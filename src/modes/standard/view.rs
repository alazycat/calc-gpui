//! 标准模式的界面：显示屏、记忆行、键盘、历史面板与记忆面板。

use super::{Action, Op, StandardCalc};
use crate::app::CalcApp;
use crate::shared::number::format_plain;
use crate::shared::ui::{self, Palette};
use gpui_kit::assets::IconName;
use gpui_kit::component::{Icon, Sizable, StyledExt};
use gpui_kit::*;

/// 把一次按键翻译成标准模式的语义动作。
///
/// 返回 `None` 表示该按键不归计算器处理（组合键、未知键等）。
/// 这是 UI 层唯一认识 [`gpui_kit::Keystroke`] 的地方；[`StandardCalc`]
/// 只消费纯数据 [`Action`]。
pub fn keystroke_to_action(keystroke: &Keystroke) -> Option<Action> {
    // 带 Ctrl / Alt / Cmd 的组合键交给系统处理，不当作计算器输入。
    let modifiers = &keystroke.modifiers;
    if modifiers.control || modifiers.alt || modifiers.platform {
        return None;
    }

    // 优先使用能直接打出来的字符，这样能正确处理 Shift 出来的 +、* 等。
    let typed = keystroke.key_char.as_deref().unwrap_or("");
    let mut iter = typed.chars();
    if let (Some(c), None) = (iter.next(), iter.next()) {
        let action = match c {
            '0'..='9' => Action::Digit(c),
            '.' => Action::Dot,
            '+' => Action::Operator(Op::Add),
            '-' => Action::Operator(Op::Sub),
            '*' | 'x' | 'X' => Action::Operator(Op::Mul),
            '/' => Action::Operator(Op::Div),
            '%' => Action::Percent,
            '=' => Action::Equals,
            _ => return None,
        };
        return Some(action);
    }

    // 没有对应字符的功能键。
    match keystroke.key.as_str() {
        "enter" => Some(Action::Equals),
        "escape" | "delete" => Some(Action::Clear),
        "backspace" => Some(Action::Backspace),
        _ => None,
    }
}

/// 绘制「标准」模式的主体（显示屏 + 记忆行 + 按键区）。
pub fn render(state: &StandardCalc, cx: &mut Context<CalcApp>) -> AnyElement {
    let p = Palette::new(cx);

    let body = if state.show_history {
        render_history(state, cx, p)
    } else if state.memory.is_open() {
        render_memory(state, cx, p)
    } else {
        render_keypad(cx, p)
    };

    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_2()
        .child(render_display(state, p))
        .child(render_memory_row(state, cx, p))
        .child(body)
        .into_any_element()
}

/// 显示屏：上方小号算式，下方大号数字。
fn render_display(state: &StandardCalc, p: Palette) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_end()
        .justify_end()
        .w_full()
        .h(px(112.))
        .flex_none()
        .rounded_lg()
        .border_1()
        .border_color(p.border)
        .bg(p.surface_alt)
        .px_4()
        .py_3()
        .overflow_hidden()
        .child(
            div()
                .w_full()
                .flex_none()
                .text_right()
                .text_sm()
                .line_height(relative(1.2))
                .truncate()
                .text_color(p.muted)
                .child(state.expression.clone()),
        )
        .child(
            div()
                .w_full()
                .text_right()
                .text_size(rems(2.6))
                .line_height(relative(1.1))
                .font_semibold()
                .whitespace_nowrap()
                .text_ellipsis_start()
                .text_color(if state.is_error() { p.danger } else { p.fg })
                .child(state.display.clone()),
        )
        .into_any_element()
}

/// 记忆操作行：MC MR M+ M− MS M▾。
fn render_memory_row(state: &StandardCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let has_memory = !state.memory.is_empty();

    div()
        .flex()
        .flex_row()
        .gap_1()
        .h(px(28.))
        .flex_none()
        .child(ui::flex_cell(ui::text_key(
            cx,
            "std-mc",
            p,
            "MC",
            has_memory,
            false,
            |app| app.standard.memory_clear(),
        )))
        .child(ui::flex_cell(ui::text_key(
            cx,
            "std-mr",
            p,
            "MR",
            has_memory,
            false,
            |app| app.standard.memory_recall(),
        )))
        .child(ui::flex_cell(ui::text_key(
            cx,
            "std-m-plus",
            p,
            "M+",
            true,
            false,
            |app| app.standard.memory_add(),
        )))
        .child(ui::flex_cell(ui::text_key(
            cx,
            "std-m-minus",
            p,
            "M−",
            true,
            false,
            |app| app.standard.memory_subtract(),
        )))
        .child(ui::flex_cell(ui::text_key(
            cx,
            "std-ms",
            p,
            "MS",
            true,
            false,
            |app| app.standard.memory_store(),
        )))
        .child(ui::flex_cell(ui::text_key(
            cx,
            "std-md",
            p,
            "M▾",
            has_memory,
            state.memory.is_open(),
            |app| app.standard.toggle_memory(),
        )))
        .into_any_element()
}

/// 6 行 × 4 列按键区。
fn render_keypad(cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let digit = |cx: &mut Context<CalcApp>, id: &'static str, label: &'static str, ch: char| {
        ui::keypad_key(cx, id, p, true, false, label, move |app| {
            app.standard.input_digit(ch)
        })
    };
    let op = |cx: &mut Context<CalcApp>, id: &'static str, label: &'static str, op: Op| {
        ui::keypad_key(cx, id, p, true, false, label, move |app| {
            app.standard.set_operator(op)
        })
    };
    let func = |cx: &mut Context<CalcApp>,
                id: &'static str,
                label: &'static str,
                action: fn(&mut StandardCalc)| {
        ui::keypad_key(cx, id, p, true, false, label, move |app| {
            action(&mut app.standard)
        })
    };

    div()
        .flex()
        .flex_col()
        .gap_1()
        .flex_1()
        // 第 1 行：%  CE  C  ⌫
        .child(ui::key_row(vec![
            func(cx, "std-percent", "%", StandardCalc::percent),
            func(cx, "std-ce", "CE", StandardCalc::clear_entry),
            func(cx, "std-clear", "C", StandardCalc::clear),
            ui::keypad_key(
                cx,
                "std-backspace",
                p,
                true,
                false,
                Icon::new(IconName::Delete).small(),
                |app| app.standard.backspace(),
            ),
        ]))
        // 第 2 行：1/x  x²  √x  ÷
        .child(ui::key_row(vec![
            func(cx, "std-reciprocal", "1/x", StandardCalc::reciprocal),
            func(cx, "std-square", "x²", StandardCalc::square),
            func(cx, "std-sqrt", "²√x", StandardCalc::sqrt),
            op(cx, "std-div", "÷", Op::Div),
        ]))
        // 第 3 行：7 8 9 ×
        .child(ui::key_row(vec![
            digit(cx, "std-7", "7", '7'),
            digit(cx, "std-8", "8", '8'),
            digit(cx, "std-9", "9", '9'),
            op(cx, "std-mul", "×", Op::Mul),
        ]))
        // 第 4 行：4 5 6 −
        .child(ui::key_row(vec![
            digit(cx, "std-4", "4", '4'),
            digit(cx, "std-5", "5", '5'),
            digit(cx, "std-6", "6", '6'),
            op(cx, "std-sub", "−", Op::Sub),
        ]))
        // 第 5 行：1 2 3 +
        .child(ui::key_row(vec![
            digit(cx, "std-1", "1", '1'),
            digit(cx, "std-2", "2", '2'),
            digit(cx, "std-3", "3", '3'),
            op(cx, "std-add", "+", Op::Add),
        ]))
        // 第 6 行：+/−  0  .  =
        .child(ui::key_row(vec![
            func(cx, "std-negate", "+/−", StandardCalc::negate),
            digit(cx, "std-0", "0", '0'),
            ui::keypad_key(cx, "std-dot", p, true, false, ".", |app| {
                app.standard.input_dot()
            }),
            ui::keypad_key(cx, "std-equals", p, true, true, "=", |app| {
                app.standard.equals()
            }),
        ]))
        .into_any_element()
}

/// 历史记录面板。
fn render_history(state: &StandardCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let mut list = div()
        .id("std-history-list")
        .flex()
        .flex_col()
        .gap_1()
        .flex_1()
        .overflow_y_scroll();

    if state.history.is_empty() {
        list = list.child(
            div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(p.muted)
                .child("还没有计算历史"),
        );
    } else {
        // 最新的排在最上面。
        for (index, entry) in state.history.iter().enumerate().rev() {
            let expression = entry.expression.clone();
            let result = entry.result.clone();
            list = list.child(
                div()
                    .id(("std-history", index))
                    .flex()
                    .flex_col()
                    .items_end()
                    .gap_1()
                    .p_2()
                    .rounded_md()
                    .cursor_pointer()
                    .hover(move |style| style.bg(p.hover))
                    .on_click(ui::press(cx, move |app| app.standard.recall_history(index)))
                    .child(div().text_sm().text_color(p.muted).child(expression))
                    .child(div().text_lg().font_medium().text_color(p.fg).child(result)),
            );
        }
    }

    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_2()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .flex_none()
                .child(
                    div()
                        .text_base()
                        .font_medium()
                        .text_color(p.fg)
                        .child("历史记录"),
                )
                .child(ui::text_key(
                    cx,
                    "std-clear-history",
                    p,
                    "清空",
                    true,
                    false,
                    |app| app.standard.clear_history(),
                )),
        )
        .child(list)
        .into_any_element()
}

/// 记忆列表面板。
fn render_memory(state: &StandardCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    // 最新存入的值排在最上面。
    let items = state
        .memory
        .slots()
        .iter()
        .enumerate()
        .rev()
        .map(|(index, value)| (index, format_plain(*value)))
        .collect();

    ui::memory_panel(
        cx,
        p,
        "std-memory",
        "std-clear-memory",
        items,
        |index, app| app.standard.memory_recall_at(index),
        |app| app.standard.memory_clear(),
    )
}

#[cfg(test)]
mod tests {
    use super::{Action, Op, StandardCalc, keystroke_to_action};
    use gpui_kit::{Keystroke, Modifiers};

    /// 构造一个只带 `key` 与 `key_char` 的按键事件。
    fn key(key: &str, key_char: Option<&str>) -> Keystroke {
        Keystroke {
            modifiers: Modifiers::default(),
            key: key.to_string(),
            key_char: key_char.map(|s| s.to_string()),
        }
    }

    #[test]
    fn digits_map_to_digit_action() {
        for c in '0'..='9' {
            let s = c.to_string();
            assert_eq!(
                keystroke_to_action(&key(&s, Some(&s))),
                Some(Action::Digit(c)),
                "数字 {c}"
            );
        }
    }

    #[test]
    fn operators_map_correctly() {
        assert_eq!(
            keystroke_to_action(&key("+", Some("+"))),
            Some(Action::Operator(Op::Add))
        );
        assert_eq!(
            keystroke_to_action(&key("-", Some("-"))),
            Some(Action::Operator(Op::Sub))
        );
        // Shift+8 打出 *，以及 x / X 都当乘号。
        assert_eq!(
            keystroke_to_action(&key("*", Some("*"))),
            Some(Action::Operator(Op::Mul))
        );
        assert_eq!(
            keystroke_to_action(&key("x", Some("x"))),
            Some(Action::Operator(Op::Mul))
        );
        assert_eq!(
            keystroke_to_action(&key("/", Some("/"))),
            Some(Action::Operator(Op::Div))
        );
    }

    #[test]
    fn dot_percent_and_equals_map_correctly() {
        assert_eq!(keystroke_to_action(&key(".", Some("."))), Some(Action::Dot));
        assert_eq!(
            keystroke_to_action(&key("%", Some("%"))),
            Some(Action::Percent)
        );
        assert_eq!(
            keystroke_to_action(&key("=", Some("="))),
            Some(Action::Equals)
        );
    }

    #[test]
    fn function_keys_map_correctly() {
        assert_eq!(
            keystroke_to_action(&key("enter", None)),
            Some(Action::Equals)
        );
        assert_eq!(
            keystroke_to_action(&key("escape", None)),
            Some(Action::Clear)
        );
        assert_eq!(
            keystroke_to_action(&key("delete", None)),
            Some(Action::Clear)
        );
        assert_eq!(
            keystroke_to_action(&key("backspace", None)),
            Some(Action::Backspace)
        );
    }

    #[test]
    fn unknown_keys_return_none() {
        assert_eq!(keystroke_to_action(&key("a", Some("a"))), None);
        assert_eq!(keystroke_to_action(&key("f1", None)), None);
    }

    #[test]
    fn modifier_combos_are_ignored() {
        let mut keystroke = key("5", Some("5"));
        keystroke.modifiers.control = true;
        assert_eq!(keystroke_to_action(&keystroke), None);

        let mut keystroke = key("5", Some("5"));
        keystroke.modifiers.alt = true;
        assert_eq!(keystroke_to_action(&keystroke), None);
    }

    // ---- 端到端：适配器 + 状态机 ----

    #[test]
    fn typing_respects_precedence() {
        let mut calc = StandardCalc::new();
        // 依次敲 1 + 2 * 3，然后回车
        for k in ["1", "+", "2", "*", "3"] {
            if let Some(action) = keystroke_to_action(&key(k, Some(k))) {
                calc.apply_action(action);
            }
        }
        if let Some(action) = keystroke_to_action(&key("enter", None)) {
            calc.apply_action(action);
        }
        assert_eq!(calc.display, "7");
    }

    #[test]
    fn escape_clears_state() {
        let mut calc = StandardCalc::new();
        if let Some(action) = keystroke_to_action(&key("5", Some("5"))) {
            calc.apply_action(action);
        }
        assert_eq!(calc.display, "5");
        if let Some(action) = keystroke_to_action(&key("escape", None)) {
            calc.apply_action(action);
        }
        assert_eq!(calc.display, "0");
        assert!(calc.expression.is_empty());
    }

    #[test]
    fn trailing_operator_reuses_operand() {
        let mut calc = StandardCalc::new();
        for k in ["7", "+"] {
            if let Some(action) = keystroke_to_action(&key(k, Some(k))) {
                calc.apply_action(action);
            }
        }
        if let Some(action) = keystroke_to_action(&key("enter", None)) {
            calc.apply_action(action);
        }
        // 末尾运算符悬空，复用最后一个数字：7 + 7 = 14
        assert_eq!(calc.display, "14");
    }
}
