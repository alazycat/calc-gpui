//! 标准模式的界面：显示屏、记忆行、键盘、历史面板与记忆面板。

use super::{Op, StandardCalc};
use crate::app::CalcApp;
use crate::shared::number::format_plain;
use crate::shared::ui::{self, Palette};
use gpui_kit::assets::IconName;
use gpui_kit::component::{Icon, Sizable, StyledExt};
use gpui_kit::*;

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
