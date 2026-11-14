//! 绘图界面：表达式输入框、工具栏、坐标系 canvas 与状态行。

use super::scene::{Viewport, build_scene, format_bound};
use super::state::GraphState;
use crate::app::CalcApp;
use crate::shared::ui::{self, Palette};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Sizable, StyledExt};
use gpui_kit::*;

pub fn render(state: &GraphState, cx: &mut Context<CalcApp>) -> AnyElement {
    let p = Palette::new(cx);

    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_2()
        .child(render_input(state, p))
        .child(render_toolbar(cx, p))
        .child(render_graph(state, p))
        .child(render_status(state, p))
        .into_any_element()
}

/// 表达式输入框（带光标）。
fn render_input(state: &GraphState, p: Palette) -> AnyElement {
    let chars = state.chars();
    let cursor = state.cursor.min(chars.len());
    let before: String = chars[..cursor].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let focus = state.error.is_none();

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .h(px(44.))
        .flex_none()
        .rounded_lg()
        .border_1()
        .border_color(if focus { p.border } else { p.danger })
        .bg(p.surface_alt)
        .px_3()
        .child(div().text_sm().text_color(p.muted).child("y ="))
        .child(
            div()
                .flex()
                .flex_1()
                .min_w_0()
                .items_center()
                .overflow_hidden()
                .child(
                    div()
                        .flex_none()
                        .whitespace_nowrap()
                        .text_size(rems(1.1))
                        .font_medium()
                        .text_color(p.fg)
                        .child(before),
                )
                .child(
                    div()
                        .flex_none()
                        .w(px(2.))
                        .h(px(18.))
                        .rounded_full()
                        .bg(p.accent),
                )
                .child(
                    div()
                        .flex_none()
                        .whitespace_nowrap()
                        .text_size(rems(1.1))
                        .font_medium()
                        .text_color(p.fg)
                        .child(after),
                ),
        )
        .into_any_element()
}

/// 绘图 / 缩放按钮。
fn render_toolbar(cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .flex_none()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    Button::new("graph-plot")
                        .label("绘图")
                        .primary()
                        .small()
                        .on_click(ui::press(cx, |app| app.graph.plot())),
                )
                .child(ui::text_key(
                    cx,
                    "graph-fit",
                    p,
                    "自动适配",
                    true,
                    false,
                    |app| {
                        app.graph.plot();
                        app.graph.fit_y();
                    },
                ))
                .child(ui::text_key(
                    cx,
                    "graph-reset",
                    p,
                    "重置视图",
                    true,
                    false,
                    |app| app.graph.reset_view(),
                ))
                .child(div().flex_1())
                .child(ui::text_key(
                    cx,
                    "graph-clear",
                    p,
                    "清空",
                    true,
                    false,
                    |app| app.graph.clear(),
                )),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(ui::text_key(
                    cx,
                    "graph-zoom-in",
                    p,
                    "放大",
                    true,
                    false,
                    |app| app.graph.zoom(0.8),
                ))
                .child(ui::text_key(
                    cx,
                    "graph-zoom-out",
                    p,
                    "缩小",
                    true,
                    false,
                    |app| app.graph.zoom(1.25),
                ))
                .child(div().flex_1())
                .child(ui::text_key(
                    cx,
                    "graph-left",
                    p,
                    "←",
                    true,
                    false,
                    |app| app.graph.pan(-0.15, 0.0),
                ))
                .child(ui::text_key(
                    cx,
                    "graph-right",
                    p,
                    "→",
                    true,
                    false,
                    |app| app.graph.pan(0.15, 0.0),
                ))
                .child(ui::text_key(cx, "graph-up", p, "↑", true, false, |app| {
                    app.graph.pan(0.0, 0.15)
                }))
                .child(ui::text_key(
                    cx,
                    "graph-down",
                    p,
                    "↓",
                    true,
                    false,
                    |app| app.graph.pan(0.0, -0.15),
                )),
        )
        .into_any_element()
}

/// 坐标系画布。
fn render_graph(state: &GraphState, p: Palette) -> AnyElement {
    let expr = state.plotted.clone();
    let viewport = Viewport {
        x_min: state.x_min,
        x_max: state.x_max,
        y_min: state.y_min,
        y_max: state.y_max,
    };
    let grid_color = Hsla {
        a: 0.45,
        ..p.border
    };
    let axis_color = Hsla { a: 0.9, ..p.muted };
    let curve_color = p.accent;
    let label_color = p.muted;

    let graph = canvas(
        move |bounds, window, _cx| {
            build_scene(bounds, expr.as_ref(), viewport, window, label_color)
        },
        move |_bounds, scene, window, cx| {
            window.paint_path(scene.grid, grid_color);
            window.paint_path(scene.axes, axis_color);
            if let Some(curve) = scene.curve {
                window.paint_path(curve, curve_color);
            }
            for (line, origin, line_height) in scene.labels {
                let _ = line.paint(origin, line_height, TextAlign::Left, None, window, cx);
            }
        },
    )
    .size_full();

    div()
        .relative()
        .flex_1()
        .min_h(px(220.))
        .rounded_lg()
        .border_1()
        .border_color(p.border)
        .bg(p.surface_alt)
        .overflow_hidden()
        .child(graph)
        .into_any_element()
}

/// 底部状态：错误信息或当前坐标范围。
fn render_status(state: &GraphState, p: Palette) -> AnyElement {
    let (text, color) = match state.error() {
        Some(error) => (error.to_string(), p.danger),
        None => (
            format!(
                "x: {} … {}    y: {} … {}",
                format_bound(state.x_min),
                format_bound(state.x_max),
                format_bound(state.y_min),
                format_bound(state.y_max)
            ),
            p.muted,
        ),
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .h(px(18.))
        .flex_none()
        .px_1()
        .child(div().text_xs().text_color(color).child(text))
        .child(div().text_xs().text_color(p.muted).child("Enter 绘图"))
        .into_any_element()
}
