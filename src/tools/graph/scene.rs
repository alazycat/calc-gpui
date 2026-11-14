//! 场景构建：把视口与曲线投影成网格、坐标轴、刻度与标签。
//!
//! 这一步是纯几何计算，在 `prepaint` 阶段算好，`paint` 只负责画。

use super::expr::Expr;
use gpui_kit::*;

/// 一次绘制所需的全部几何数据，在 prepaint 里算好，paint 里只负责画。
pub struct Scene {
    pub grid: Path<Pixels>,
    pub axes: Path<Pixels>,
    pub curve: Option<Path<Pixels>>,
    pub labels: Vec<(ShapedLine, Point<Pixels>, Pixels)>,
}

/// 取一个「好看」的刻度间隔，让纵横向大约有 8 格。
pub fn nice_step(span: f64) -> f64 {
    let raw = (span / 8.0).abs();
    if !raw.is_finite() || raw <= 0.0 {
        return 1.0;
    }
    let magnitude = 10f64.powf(raw.log10().floor());
    let normalized = raw / magnitude;
    let step = if normalized < 1.5 {
        1.0
    } else if normalized < 3.0 {
        2.0
    } else if normalized < 7.0 {
        5.0
    } else {
        10.0
    };
    step * magnitude
}

pub fn format_tick(value: f64, step: f64) -> String {
    let value = if value.abs() < step * 1e-6 {
        0.0
    } else {
        value
    };
    if value == 0.0 {
        return "0".to_string();
    }
    if value.abs() >= 1e6 || value.abs() < 1e-4 {
        return format!("{value:.1e}");
    }
    if step >= 1.0 {
        return format!("{}", value.round() as i64);
    }
    // 多留一位小数再裁掉末尾的 0，兼容 0.25 这类刻度。
    let digits = ((-step.log10()) + 1.0).ceil() as usize;
    let text = format!("{value:.digits$}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

pub fn format_bound(value: f64) -> String {
    let text = format!("{value:.2}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" { "0" } else { text }.to_string()
}

/// 在 [min, max] 内枚举刻度。
pub fn ticks(min: f64, max: f64, step: f64) -> Vec<f64> {
    let start = (min / step).ceil() * step;
    let mut values = Vec::new();
    let mut value = start;
    while value <= max + step * 1e-6 {
        values.push(value);
        value += step;
        if values.len() > 200 {
            break;
        }
    }
    values
}

/// 当前视图范围。
#[derive(Clone, Copy)]
pub struct Viewport {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

/// 把数据坐标映射到画布像素。
pub struct Projection {
    origin: Point<Pixels>,
    width: Pixels,
    height: Pixels,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

impl Projection {
    fn x(&self, value: f64) -> Pixels {
        let fraction = ((value - self.x_min) / (self.x_max - self.x_min)) as f32;
        self.origin.x + self.width * fraction
    }

    fn y(&self, value: f64) -> Pixels {
        let fraction = ((self.y_max - value) / (self.y_max - self.y_min)) as f32;
        self.origin.y + self.height * fraction
    }

    fn point(&self, x: f64, y: f64) -> Point<Pixels> {
        point(self.x(x), self.y(y))
    }
}

pub fn build_scene(
    bounds: Bounds<Pixels>,
    expr: Option<&Expr>,
    viewport: Viewport,
    window: &mut Window,
    label_color: Hsla,
) -> Scene {
    let Viewport {
        x_min,
        x_max,
        y_min,
        y_max,
    } = viewport;
    let projection = Projection {
        origin: bounds.origin,
        width: bounds.size.width,
        height: bounds.size.height,
        x_min,
        x_max,
        y_min,
        y_max,
    };

    let x_step = nice_step(x_max - x_min);
    let y_step = nice_step(y_max - y_min);

    // 网格。
    let mut grid = PathBuilder::stroke(px(1.0));
    for value in ticks(x_min, x_max, x_step) {
        if value.abs() < x_step * 1e-6 {
            continue;
        }
        let x = projection.x(value);
        grid.move_to(point(x, projection.origin.y));
        grid.line_to(point(x, projection.origin.y + projection.height));
    }
    for value in ticks(y_min, y_max, y_step) {
        if value.abs() < y_step * 1e-6 {
            continue;
        }
        let y = projection.y(value);
        grid.move_to(point(projection.origin.x, y));
        grid.line_to(point(projection.origin.x + projection.width, y));
    }
    let grid = grid.build().unwrap_or_else(|_| Path::new(bounds.origin));

    // 坐标轴。
    let mut axes = PathBuilder::stroke(px(1.5));
    if x_min <= 0.0 && x_max >= 0.0 {
        let x = projection.x(0.0);
        axes.move_to(point(x, projection.origin.y));
        axes.line_to(point(x, projection.origin.y + projection.height));
    }
    if y_min <= 0.0 && y_max >= 0.0 {
        let y = projection.y(0.0);
        axes.move_to(point(projection.origin.x, y));
        axes.line_to(point(projection.origin.x + projection.width, y));
    }
    let axes = axes.build().unwrap_or_else(|_| Path::new(bounds.origin));

    // 曲线。
    let curve = expr.and_then(|expr| {
        let mut builder = PathBuilder::stroke(px(2.0));
        let width = projection.width.as_f32().max(1.0);
        let steps = width.round() as usize;
        let mut started = false;
        let mut previous_y = 0.0f32;

        for index in 0..=steps {
            let fraction = index as f64 / steps as f64;
            let x = x_min + (x_max - x_min) * fraction;
            let y = expr.eval(x);
            if !y.is_finite() {
                started = false;
                continue;
            }
            let point = projection.point(x, y);
            let pixel_y = point.y.as_f32();
            // 渐近线两侧的值会突然跳变，断开路径避免画出一条竖线。
            if started && (pixel_y - previous_y).abs() > bounds.size.height.as_f32() {
                started = false;
            }
            if started {
                builder.line_to(point);
            } else {
                builder.move_to(point);
                started = true;
            }
            previous_y = pixel_y;
        }
        builder.build().ok()
    });

    // 刻度标签。
    let font = Font::default();
    let font_size = px(10.0);
    let line_height = px(13.0);
    let mut labels = Vec::new();

    let axis_x = if x_min <= 0.0 && x_max >= 0.0 {
        projection.x(0.0)
    } else {
        projection.origin.x + px(4.0)
    };
    let axis_y = if y_min <= 0.0 && y_max >= 0.0 {
        projection.y(0.0)
    } else {
        projection.origin.y + projection.height - line_height - px(2.0)
    };

    let shape = |text: String| {
        let run = TextRun {
            len: text.len(),
            font: font.clone(),
            color: label_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        window
            .text_system()
            .shape_line(text.into(), font_size, &[run], None)
    };

    for value in ticks(x_min, x_max, x_step) {
        if value.abs() < x_step * 1e-6 {
            continue;
        }
        let text = format_tick(value, x_step);
        let line = shape(text);
        let width = line.width();
        let center = projection.x(value);
        // 贴边刻度（例如 -10）的标签不要被画布裁掉。
        let mut label_x = center - width / 2.0;
        let min_x = projection.origin.x + px(2.0);
        let max_x = projection.origin.x + projection.width - width - px(2.0);
        if label_x < min_x {
            label_x = min_x;
        }
        if label_x > max_x {
            label_x = max_x;
        }
        let origin = point(
            label_x,
            (axis_y + px(2.0)).min(projection.origin.y + projection.height - line_height),
        );
        labels.push((line, origin, line_height));
    }

    for value in ticks(y_min, y_max, y_step) {
        if value.abs() < y_step * 1e-6 {
            continue;
        }
        let text = format_tick(value, y_step);
        let line = shape(text);
        let width = line.width();
        let center = projection.y(value);
        let mut label_y = center - line_height / 2.0;
        let min_y = projection.origin.y + px(2.0);
        let max_y = projection.origin.y + projection.height - line_height - px(2.0);
        if label_y < min_y {
            label_y = min_y;
        }
        if label_y > max_y {
            label_y = max_y;
        }
        let origin = point(
            (axis_x - width - px(4.0)).max(projection.origin.x + px(2.0)),
            label_y,
        );
        labels.push((line, origin, line_height));
    }

    Scene {
        grid,
        axes,
        curve,
        labels,
    }
}
