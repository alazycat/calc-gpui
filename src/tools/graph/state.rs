//! 绘图状态：表达式输入、光标位置、最近一次成功的曲线与视口范围。

use super::expr::{Expr, parse};
use gpui_kit::Keystroke;

/// 绘图模式状态。
pub struct GraphState {
    /// 正在编辑的表达式。
    input: String,
    /// 光标位置（字符下标）。
    pub(super) cursor: usize,
    /// 最近一次成功解析的表达式。
    pub(super) plotted: Option<Expr>,
    /// 解析错误（按 Enter / 绘图时显示）。
    pub(super) error: Option<String>,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl GraphState {
    pub fn new() -> Self {
        let input = "sin(x)".to_string();
        Self {
            cursor: input.chars().count(),
            plotted: parse(&input).ok(),
            input,
            error: None,
            x_min: -10.0,
            x_max: 10.0,
            y_min: -7.0,
            y_max: 7.0,
        }
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn copy_text(&self) -> String {
        self.input.clone()
    }

    pub fn paste(&mut self, text: &str) {
        self.input = text.trim().to_string();
        self.cursor = self.input.chars().count();
        self.refresh();
    }

    pub fn chars(&self) -> Vec<char> {
        self.input.chars().collect()
    }

    fn set_chars(&mut self, chars: Vec<char>) {
        self.input = chars.into_iter().collect();
        self.refresh();
    }

    /// 输入变化时静默尝试重新解析：解析失败时保留上一条曲线。
    fn refresh(&mut self) {
        self.error = None;
        if let Ok(expr) = parse(&self.input) {
            self.plotted = Some(expr);
        }
    }

    /// 按 Enter / 绘图：解析并报告错误。
    pub fn plot(&mut self) {
        match parse(&self.input) {
            Ok(expr) => {
                self.plotted = Some(expr);
                self.error = None;
            }
            Err(message) => self.error = Some(message),
        }
    }

    pub fn input_char(&mut self, ch: char) {
        if !(ch.is_ascii_graphic() || ch == ' ') {
            return;
        }
        let mut chars = self.chars();
        chars.insert(self.cursor.min(chars.len()), ch);
        self.cursor = (self.cursor + 1).min(chars.len());
        self.set_chars(chars);
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let mut chars = self.chars();
        chars.remove(self.cursor - 1);
        self.cursor -= 1;
        self.set_chars(chars);
    }

    pub fn delete_forward(&mut self) {
        let mut chars = self.chars();
        if self.cursor < chars.len() {
            chars.remove(self.cursor);
            self.set_chars(chars);
        }
    }

    pub fn move_cursor(&mut self, delta: i32) {
        let len = self.input.chars().count();
        self.cursor = (self.cursor as i32 + delta).clamp(0, len as i32) as usize;
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.input.chars().count();
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.plotted = None;
        self.error = None;
    }

    pub fn zoom(&mut self, factor: f64) {
        let center_x = (self.x_min + self.x_max) / 2.0;
        let center_y = (self.y_min + self.y_max) / 2.0;
        let half_x = (self.x_max - self.x_min) / 2.0 * factor;
        let half_y = (self.y_max - self.y_min) / 2.0 * factor;
        self.x_min = center_x - half_x;
        self.x_max = center_x + half_x;
        self.y_min = center_y - half_y;
        self.y_max = center_y + half_y;
    }

    /// 平移视图，`dx` / `dy` 是横纵跨度的比例。
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let shift_x = (self.x_max - self.x_min) * dx;
        let shift_y = (self.y_max - self.y_min) * dy;
        self.x_min += shift_x;
        self.x_max += shift_x;
        self.y_min += shift_y;
        self.y_max += shift_y;
    }

    pub fn reset_view(&mut self) {
        self.x_min = -10.0;
        self.x_max = 10.0;
        self.y_min = -7.0;
        self.y_max = 7.0;
    }

    /// 根据曲线在 x 范围内的取值自动适配 y 轴。
    pub fn fit_y(&mut self) {
        let Some(expr) = &self.plotted else {
            return;
        };
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for i in 0..=400 {
            let x = self.x_min + (self.x_max - self.x_min) * i as f64 / 400.0;
            let y = expr.eval(x);
            // 忽略爆炸性的值（例如 tan 的渐近线附近）。
            if y.is_finite() && y.abs() <= 1e6 {
                low = low.min(y);
                high = high.max(y);
            }
        }
        if low.is_finite() && high.is_finite() {
            let span = (high - low).abs();
            let pad = if span < 1e-9 { 1.0 } else { span * 0.1 };
            self.y_min = low - pad;
            self.y_max = high + pad;
        }
    }

    pub fn handle_key(&mut self, keystroke: &Keystroke) {
        let modifiers = &keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform {
            return;
        }

        let typed = keystroke.key_char.as_deref().unwrap_or("");
        let mut iter = typed.chars();
        if let (Some(c), None) = (iter.next(), iter.next()) {
            self.input_char(c);
            return;
        }

        match keystroke.key.as_str() {
            "backspace" => self.backspace(),
            "delete" => self.delete_forward(),
            "left" => self.move_cursor(-1),
            "right" => self.move_cursor(1),
            "home" => self.move_home(),
            "end" => self.move_end(),
            "enter" => self.plot(),
            "escape" => self.clear(),
            _ => {}
        }
    }
}

impl Default for GraphState {
    fn default() -> Self {
        Self::new()
    }
}
