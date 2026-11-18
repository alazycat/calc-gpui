//! 绘图界面：视图切换、表达式输入 / 列表、坐标系 canvas，以及底部的数学键盘。
//!
//! 布局自上而下：视图切换 → 内容（坐标系或表达式列表）→ 函数面板 → 数学键盘。
//! 两种视图共用底部键盘，键盘永远作用在**当前选中的表达式**上。

use super::scene::{CurveShape, Viewport, build_scene, format_bound};
use super::state::{Action, Equation, GraphState, GraphView, PaletteTab};
use crate::app::CalcApp;
use crate::shared::ui::{self, Palette};
use gpui_kit::assets::IconName;
use gpui_kit::component::{Icon, Sizable, StyledExt};
use gpui_kit::*;

/// 曲线的调色板：下标就是表达式序号，超出后循环。
const CURVE_COLORS: [u32; 6] = [0x2563eb, 0xdc2626, 0x16a34a, 0xd97706, 0x9333ea, 0x0891b2];

fn curve_color(index: usize) -> Hsla {
    rgb(CURVE_COLORS[index % CURVE_COLORS.len()]).into()
}

pub fn render(state: &GraphState, cx: &mut Context<CalcApp>) -> AnyElement {
    let p = Palette::new(cx);

    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap_2()
        .overflow_hidden()
        .child(render_switch(state, cx, p))
        .child(match state.view() {
            GraphView::Graph => render_graph_pane(state, p),
            GraphView::Equations => render_equation_list(state, cx, p),
        })
        .child(render_keypad(state, cx, p))
        .into_any_element()
}

// ---- 视图切换 ----

/// 右上角的分段控件：坐标系 / 表达式列表。
fn render_switch(state: &GraphState, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .h(px(26.))
        .flex_none()
        .child(
            div()
                .flex_1()
                .text_sm()
                .font_medium()
                .text_color(p.muted)
                .child(match state.view() {
                    GraphView::Graph => "函数",
                    GraphView::Equations => "表达式",
                }),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap_1()
                .p_1()
                .rounded_md()
                .border_1()
                .border_color(p.border)
                .child(view_tab(
                    cx,
                    "graph-tab-graph",
                    p,
                    IconName::ChartLine,
                    state.view() == GraphView::Graph,
                    Action::ShowView(GraphView::Graph),
                ))
                .child(view_tab(
                    cx,
                    "graph-tab-equations",
                    p,
                    IconName::SquareFunction,
                    state.view() == GraphView::Equations,
                    Action::ShowView(GraphView::Equations),
                )),
        )
        .into_any_element()
}

fn view_tab(
    cx: &mut Context<CalcApp>,
    id: &'static str,
    p: Palette,
    icon: IconName,
    active: bool,
    action: Action,
) -> AnyElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .w(px(34.))
        .h(px(22.))
        .rounded_sm()
        .cursor_pointer()
        .bg(if active {
            p.accent_soft
        } else {
            transparent_black()
        })
        .hover(move |style| style.bg(p.hover))
        .on_click(ui::press(cx, move |app| app.graph.apply_action(action)))
        .child(
            Icon::new(icon)
                .small()
                .text_color(if active { p.accent } else { p.muted }),
        )
        .into_any_element()
}

// ---- 坐标系视图 ----

fn render_graph_pane(state: &GraphState, p: Palette) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap_2()
        .child(render_input(state, p))
        .child(render_graph(state, p))
        .child(render_status(state, p))
        .into_any_element()
}

/// 表达式输入框（带光标），左边的 `f` 是当前表达式的颜色标记。
fn render_input(state: &GraphState, p: Palette) -> AnyElement {
    let chars = state.chars();
    let cursor = state.cursor().min(chars.len());
    let before: String = chars[..cursor].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let is_error = state.error().is_some();
    let empty = chars.is_empty();
    let accent = curve_color(state.active_index());

    let mut text = div()
        .flex()
        .flex_row()
        .flex_1()
        .min_w_0()
        .items_center()
        .overflow_hidden();

    if empty {
        text = text
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
                    .text_size(rems(1.05))
                    .text_color(p.muted)
                    .child("输入表达式"),
            );
    } else {
        text = text
            .child(
                div()
                    .flex_none()
                    .whitespace_nowrap()
                    .text_size(rems(1.05))
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
                    .text_size(rems(1.05))
                    .font_medium()
                    .text_color(p.fg)
                    .child(after),
            );
    }

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .h(px(38.))
        .flex_none()
        .rounded_lg()
        .border_1()
        .border_color(if is_error { p.danger } else { p.border })
        .bg(p.surface_alt)
        .px_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w(px(28.))
                .h(px(28.))
                .flex_none()
                .rounded_sm()
                .bg(accent)
                .child(
                    div()
                        .italic()
                        .text_base()
                        .font_medium()
                        .text_color(p.surface_alt)
                        .child("f"),
                ),
        )
        .child(text)
        .into_any_element()
}

/// 坐标系画布。
fn render_graph(state: &GraphState, p: Palette) -> AnyElement {
    let plots = state.visible_plots();
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
    let label_color = p.muted;

    let graph = canvas(
        move |bounds, window, _cx| build_scene(bounds, &plots, viewport, window, label_color),
        move |_bounds, scene, window, cx| {
            window.paint_path(scene.grid, grid_color);
            window.paint_path(scene.axes, axis_color);
            for (path, index) in scene.fills {
                // 半透明填充：看得见下面的网格，又能分辨是哪个表达式。
                window.paint_path(
                    path,
                    Hsla {
                        a: 0.16,
                        ..curve_color(index)
                    },
                );
            }
            for CurveShape { path, index } in scene.curves {
                window.paint_path(path, curve_color(index));
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
        .min_h(px(90.))
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
        // 还没有可画的曲线时，状态行当提示用。
        None if state.active_plot().is_none() => {
            ("输入表达式，按 Enter 或 ↵ 绘图".to_string(), p.muted)
        }
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
        .h(px(16.))
        .flex_none()
        .px_1()
        .child(div().text_xs().text_color(color).child(text))
        .child(
            div()
                .text_xs()
                .text_color(p.muted)
                .child("Enter 绘图    Esc 清空"),
        )
        .into_any_element()
}

// ---- 表达式列表视图 ----

fn render_equation_list(state: &GraphState, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let mut list = div()
        .id("graph-equation-list")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap_1()
        .overflow_y_scroll();

    for (index, equation) in state.equations().iter().enumerate() {
        list = list.child(equation_row(
            cx,
            p,
            index,
            equation,
            index == state.active_index(),
        ));
    }

    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap_2()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .text_color(p.muted)
                        .child(format!("共 {} 条", state.equations().len())),
                )
                .child(ui::text_key(
                    cx,
                    "graph-add-equation",
                    p,
                    "+ 添加表达式",
                    true,
                    false,
                    |app| app.graph.apply_action(Action::AddEquation),
                )),
        )
        .child(list)
        .into_any_element()
}

fn equation_row(
    cx: &mut Context<CalcApp>,
    p: Palette,
    index: usize,
    equation: &Equation,
    active: bool,
) -> AnyElement {
    let visible = equation.is_visible();
    let text = equation.text().to_string();
    let has_error = equation.error().is_some();

    div()
        .id(("graph-equation", index))
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .h(px(38.))
        .flex_none()
        .px_2()
        .rounded_sm()
        .border_1()
        .border_color(if active {
            p.border
        } else {
            transparent_black()
        })
        .bg(if active {
            p.surface_alt
        } else {
            transparent_black()
        })
        .cursor_pointer()
        .hover(move |style| style.bg(p.hover))
        .on_click(ui::press(cx, move |app| {
            app.graph.apply_action(Action::SelectEquation(index))
        }))
        .child(
            div()
                .w(px(8.))
                .h(px(8.))
                .flex_none()
                .rounded_full()
                .bg(if visible {
                    curve_color(index)
                } else {
                    p.faded(p.muted)
                }),
        )
        .child(div().flex_none().text_sm().text_color(p.muted).child(
            if equation.plot().is_some_and(|plot| plot.is_inequality()) {
                ""
            } else {
                "y ="
            },
        ))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_sm()
                .text_color(if has_error {
                    p.danger
                } else if visible {
                    p.fg
                } else {
                    p.muted
                })
                .child(if text.is_empty() {
                    "空表达式".to_string()
                } else {
                    text
                }),
        )
        .child(row_icon(
            cx,
            ("graph-equation-eye", index),
            p,
            if visible {
                IconName::Eye
            } else {
                IconName::EyeOff
            },
            move |app| app.graph.apply_action(Action::ToggleVisible(index)),
        ))
        .child(row_icon(
            cx,
            ("graph-equation-remove", index),
            p,
            IconName::Trash,
            move |app| app.graph.apply_action(Action::RemoveEquation(index)),
        ))
        .into_any_element()
}

fn row_icon(
    cx: &mut Context<CalcApp>,
    id: (&'static str, usize),
    p: Palette,
    icon: IconName,
    action: impl Fn(&mut CalcApp) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .w(px(26.))
        .h(px(26.))
        .flex_none()
        .rounded_sm()
        .cursor_pointer()
        .hover(move |style| style.bg(p.hover))
        .on_click(ui::press(cx, action))
        .child(Icon::new(icon).xsmall().text_color(p.muted))
        .into_any_element()
}

// ---- 数学键盘 ----

/// 键盘上的一个按键。
#[derive(Clone, Copy)]
enum Key {
    /// 普通插入键。
    Insert(&'static str, &'static str),
    /// 有 `2ⁿᵈ` 变体的插入键：`(标签, 插入文本, 2nd 标签, 2nd 插入文本)`。
    Toggle(&'static str, &'static str, &'static str, &'static str),
    Second,
    Clear,
    Delete,
    /// 等号：同样是绘图，但不用强调色（强调色留给右下的回车键）。
    Equals,
    Plot,
}

/// 键盘布局：7 行 × 5 列。
const KEYS: [[Key; 5]; 7] = [
    [
        Key::Second,
        Key::Insert("π", "pi"),
        Key::Insert("e", "e"),
        Key::Clear,
        Key::Delete,
    ],
    [
        Key::Toggle("x²", "^2", "x³", "^3"),
        Key::Insert("1/x", "1/("),
        Key::Insert("|x|", "abs("),
        Key::Insert("x", "x"),
        Key::Insert("y", "y "),
    ],
    [
        Key::Toggle("√x", "sqrt(", "∛x", "cbrt("),
        Key::Insert("(", "("),
        Key::Insert(")", ")"),
        Key::Equals,
        Key::Insert("÷", "/"),
    ],
    [
        Key::Insert("xʸ", "^"),
        Key::Insert("7", "7"),
        Key::Insert("8", "8"),
        Key::Insert("9", "9"),
        Key::Insert("×", "*"),
    ],
    [
        Key::Toggle("10ˣ", "10^", "2ˣ", "2^"),
        Key::Insert("4", "4"),
        Key::Insert("5", "5"),
        Key::Insert("6", "6"),
        Key::Insert("−", "-"),
    ],
    [
        Key::Insert("log", "log("),
        Key::Insert("1", "1"),
        Key::Insert("2", "2"),
        Key::Insert("3", "3"),
        Key::Insert("+", "+"),
    ],
    [
        Key::Insert("ln", "ln("),
        Key::Insert("(-)", "-"),
        Key::Insert("0", "0"),
        Key::Insert(".", "."),
        Key::Plot,
    ],
];

fn render_keypad(state: &GraphState, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let mut column = div().flex().flex_col().gap_1().flex_none();

    // 三个函数面板页签。
    column = column.child(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .h(px(28.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .flex_1()
                    .child(palette_tab(cx, p, PaletteTab::Trig, state))
                    .child(palette_tab(cx, p, PaletteTab::Inequality, state))
                    .child(palette_tab(cx, p, PaletteTab::Functions, state)),
            )
            .child(ui::text_key(
                cx,
                "graph-fit",
                p,
                "适配",
                true,
                false,
                |app| {
                    app.graph.plot();
                    app.graph.fit_y();
                },
            )),
    );

    // 面板打开时插在页签与键盘之间。
    if let Some(tab) = state.palette() {
        column = column.child(render_palette(cx, p, tab, state.second()));
    }

    for (row_index, row) in KEYS.iter().enumerate() {
        let mut keys = Vec::with_capacity(5);
        for (column_index, key) in row.iter().enumerate() {
            keys.push(render_key(cx, p, state, *key, row_index * 5 + column_index));
        }
        column = column.child(div().h(px(30.)).w_full().child(ui::key_row(keys)));
    }

    column.into_any_element()
}

fn palette_tab(
    cx: &mut Context<CalcApp>,
    p: Palette,
    tab: PaletteTab,
    state: &GraphState,
) -> AnyElement {
    let active = state.palette() == Some(tab);
    let label = format!("{} ▾", tab.label());
    let id = match tab {
        PaletteTab::Trig => "graph-palette-trig",
        PaletteTab::Inequality => "graph-palette-inequality",
        PaletteTab::Functions => "graph-palette-functions",
    };
    ui::text_key(cx, id, p, label, true, active, move |app| {
        app.graph.apply_action(Action::TogglePalette(tab))
    })
}

/// 面板里的一格格函数键。
fn render_palette(
    cx: &mut Context<CalcApp>,
    p: Palette,
    tab: PaletteTab,
    second: bool,
) -> AnyElement {
    let entries = tab.entries(second);
    let mut grid = div()
        .flex()
        .flex_col()
        .gap_1()
        .flex_none()
        .p_1()
        .rounded_md();

    for (row_index, chunk) in entries.chunks(3).enumerate() {
        let mut row = Vec::with_capacity(3);
        for (offset, entry) in chunk.iter().enumerate() {
            let position = row_index * 3 + offset;
            row.push(ui::keypad_key(
                cx,
                tab_id(tab, position),
                p,
                true,
                false,
                entry.label,
                move |app| app.graph.apply_action(Action::Insert(entry.insert)),
            ));
        }
        grid = grid.child(div().h(px(28.)).w_full().child(ui::key_row(row)));
    }

    grid.bg(p.surface_alt)
        .border_1()
        .border_color(p.border)
        .into_any_element()
}

fn render_key(
    cx: &mut Context<CalcApp>,
    p: Palette,
    state: &GraphState,
    key: Key,
    position: usize,
) -> AnyElement {
    let id = ("graph-key", position);
    match key {
        Key::Insert(label, insert) => ui::keypad_key(cx, id, p, true, false, label, move |app| {
            app.graph.apply_action(Action::Insert(insert))
        }),
        Key::Toggle(label, insert, second_label, second_insert) => {
            let (shown, text) = if state.second() {
                (second_label, second_insert)
            } else {
                (label, insert)
            };
            ui::keypad_key(cx, id, p, true, false, shown, move |app| {
                app.graph.apply_action(Action::Insert(text))
            })
        }
        Key::Second => ui::soft_key(cx, id, p, state.second(), "2nd", |app| {
            app.graph.apply_action(Action::Second)
        }),
        Key::Clear => ui::keypad_key(cx, id, p, true, false, "C", |app| {
            app.graph.apply_action(Action::Clear)
        }),
        Key::Delete => ui::keypad_key(cx, id, p, true, false, "⌫", |app| {
            app.graph.apply_action(Action::Backspace)
        }),
        Key::Equals => ui::keypad_key(cx, id, p, true, false, "=", |app| {
            app.graph.apply_action(Action::Plot)
        }),
        Key::Plot => ui::keypad_key(cx, id, p, true, true, "↵", |app| {
            app.graph.apply_action(Action::Plot)
        }),
    }
}

/// 面板按键的 id：同一页签内按位置唯一，跨页签也不冲突。
fn tab_id(tab: PaletteTab, position: usize) -> (&'static str, usize) {
    let base = match tab {
        PaletteTab::Trig => 0,
        PaletteTab::Inequality => 16,
        PaletteTab::Functions => 32,
    };
    ("graph-palette-key", base + position)
}

/// 把按键翻译成语义动作。表达式输入框接受任意可见字符，
/// 其余是文本编辑常用的光标移动与删除键，外加 Enter 绘图、Esc 清空。
pub fn keystroke_to_action(keystroke: &Keystroke) -> Option<Action> {
    let modifiers = &keystroke.modifiers;
    if modifiers.control || modifiers.alt || modifiers.platform {
        return None;
    }

    let typed = keystroke.key_char.as_deref().unwrap_or("");
    let mut iter = typed.chars();
    if let (Some(c), None) = (iter.next(), iter.next()) {
        return Some(Action::Input(c));
    }

    match keystroke.key.as_str() {
        "backspace" => Some(Action::Backspace),
        "delete" => Some(Action::DeleteForward),
        "left" => Some(Action::MoveCursor(-1)),
        "right" => Some(Action::MoveCursor(1)),
        "home" => Some(Action::Home),
        "end" => Some(Action::End),
        "enter" => Some(Action::Plot),
        "escape" => Some(Action::Clear),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, GraphState, keystroke_to_action};
    use gpui_kit::{Keystroke, Modifiers};

    fn key(key: &str, key_char: Option<&str>) -> Keystroke {
        Keystroke {
            modifiers: Modifiers::default(),
            key: key.to_string(),
            key_char: key_char.map(|s| s.to_string()),
        }
    }

    fn typed(ch: char) -> Keystroke {
        key("a", Some(&ch.to_string()))
    }

    fn named(key_name: &str) -> Keystroke {
        key(key_name, None)
    }

    #[test]
    fn every_typed_character_becomes_input() {
        // 表达式是文本：字母、运算符、空格都原样交给 input_char 过滤。
        assert_eq!(keystroke_to_action(&typed('x')), Some(Action::Input('x')));
        assert_eq!(keystroke_to_action(&typed('+')), Some(Action::Input('+')));
        assert_eq!(keystroke_to_action(&typed('(')), Some(Action::Input('(')));
        assert_eq!(keystroke_to_action(&typed(' ')), Some(Action::Input(' ')));
        // 不等号也是普通字符，由解析器决定能不能用。
        assert_eq!(keystroke_to_action(&typed('>')), Some(Action::Input('>')));
    }

    #[test]
    fn editing_and_plot_keys_map_to_actions() {
        assert_eq!(
            keystroke_to_action(&named("backspace")),
            Some(Action::Backspace)
        );
        assert_eq!(
            keystroke_to_action(&named("delete")),
            Some(Action::DeleteForward)
        );
        assert_eq!(
            keystroke_to_action(&named("left")),
            Some(Action::MoveCursor(-1))
        );
        assert_eq!(
            keystroke_to_action(&named("right")),
            Some(Action::MoveCursor(1))
        );
        assert_eq!(keystroke_to_action(&named("home")), Some(Action::Home));
        assert_eq!(keystroke_to_action(&named("end")), Some(Action::End));
        assert_eq!(keystroke_to_action(&named("enter")), Some(Action::Plot));
        assert_eq!(keystroke_to_action(&named("escape")), Some(Action::Clear));
    }

    #[test]
    fn unknown_and_modifier_keys_are_ignored() {
        // 不在映射里的功能键。
        assert_eq!(keystroke_to_action(&named("up")), None);
        assert_eq!(keystroke_to_action(&named("tab")), None);
        // 任何组合键都不参与编辑。
        let mut ctrl = typed('c');
        ctrl.modifiers.control = true;
        assert_eq!(keystroke_to_action(&ctrl), None);
        let mut alt = named("left");
        alt.modifiers.alt = true;
        assert_eq!(keystroke_to_action(&alt), None);
    }

    #[test]
    fn keystrokes_drive_the_state_end_to_end() {
        let mut state = GraphState::new();
        state.clear();
        for keystroke in [typed('2'), typed('x'), named("left"), named("backspace")] {
            if let Some(action) = keystroke_to_action(&keystroke) {
                state.apply_action(action);
            }
        }
        // "2x" → 光标左移 → 退格删掉 '2'，剩 "x"。
        assert_eq!(state.copy_text(), "x");

        if let Some(action) = keystroke_to_action(&named("end")) {
            state.apply_action(action);
        }
        assert_eq!(state.cursor(), 1);

        if let Some(action) = keystroke_to_action(&named("enter")) {
            state.apply_action(action);
        }
        assert!(state.active_plot().is_some());
        assert!(state.error().is_none());
    }
}
