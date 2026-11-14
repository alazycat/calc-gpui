//! 科学模式的界面：显示屏、状态行、函数下拉、键盘、历史与记忆面板。

use super::{Constant, MathFn, SOp, ScientificCalc, Trig, Unary};
use crate::app::CalcApp;
use crate::shared::number::format_value;
use crate::shared::ui::{self, Palette};
use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::component::{Icon, Sizable, StyledExt};
use gpui_kit::*;

/// 「三角函数」下拉的菜单项。
fn trig_items(second: bool) -> Vec<(Trig, &'static str)> {
    let items = if second {
        [
            Trig::Asin,
            Trig::Acos,
            Trig::Atan,
            Trig::Asec,
            Trig::Acsc,
            Trig::Acot,
        ]
    } else {
        [
            Trig::Sin,
            Trig::Cos,
            Trig::Tan,
            Trig::Sec,
            Trig::Csc,
            Trig::Cot,
        ]
    };
    items.iter().map(|func| (*func, func.label())).collect()
}

/// 「函数」下拉的菜单项。
fn math_items(second: bool) -> Vec<(MathFn, &'static str)> {
    let mut items: Vec<MathFn> = if second {
        vec![
            MathFn::Asinh,
            MathFn::Acosh,
            MathFn::Atanh,
            MathFn::Asech,
            MathFn::Acsch,
            MathFn::Acoth,
        ]
    } else {
        vec![
            MathFn::Sinh,
            MathFn::Cosh,
            MathFn::Tanh,
            MathFn::Sech,
            MathFn::Csch,
            MathFn::Coth,
        ]
    };
    items.extend([MathFn::Floor, MathFn::Ceil, MathFn::Round]);
    items.iter().map(|func| (*func, func.label())).collect()
}

/// 绘制「科学」模式的主体。
pub fn render(state: &ScientificCalc, cx: &mut Context<CalcApp>) -> AnyElement {
    let p = Palette::new(cx);

    let body = if state.show_history {
        render_history(state, cx, p)
    } else if state.memory.is_open() {
        render_memory(state, cx, p)
    } else {
        render_keypad(state, cx, p)
    };

    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_1()
        .child(render_display(state, p))
        .child(render_status_row(state, cx, p))
        .child(render_memory_row(state, cx, p))
        .child(render_function_row(state, cx, p))
        .child(ui::divider(p.border))
        .child(body)
        .into_any_element()
}

/// 显示屏（表达式 + 大号数字）。
fn render_display(state: &ScientificCalc, p: Palette) -> AnyElement {
    let text = state.display();

    div()
        .flex()
        .flex_col()
        .items_end()
        .justify_end()
        .w_full()
        .h(px(84.))
        .flex_none()
        .px_2()
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
                .child(state.expression().to_string()),
        )
        .child(
            div()
                .w_full()
                .text_right()
                .text_size(ui::display_size(&text))
                .line_height(relative(1.1))
                .font_semibold()
                .whitespace_nowrap()
                .text_ellipsis_start()
                .text_color(if state.is_error() { p.muted } else { p.fg })
                .child(text),
        )
        .into_any_element()
}

/// DEG / F-E 状态行。
fn render_status_row(state: &ScientificCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_4()
        .h(px(28.))
        .flex_none()
        .px_1()
        .child(ui::text_key(
            cx,
            "sci-angle",
            p,
            state.angle.label(),
            true,
            false,
            |app| app.scientific.cycle_angle(),
        ))
        .child(ui::text_key(
            cx,
            "sci-fe",
            p,
            "F-E",
            true,
            state.scientific_notation,
            |app| app.scientific.toggle_scientific_notation(),
        ))
        .into_any_element()
}

/// 记忆操作行：MC MR M+ M− MS M▾。
fn render_memory_row(state: &ScientificCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let has_memory = !state.memory.is_empty();

    div()
        .flex()
        .flex_row()
        .gap_1()
        .h(px(28.))
        .flex_none()
        .child(ui::flex_cell(ui::text_key(
            cx,
            "sci-mc",
            p,
            "MC",
            has_memory,
            false,
            |app| app.scientific.memory_clear(),
        )))
        .child(ui::flex_cell(ui::text_key(
            cx,
            "sci-mr",
            p,
            "MR",
            has_memory,
            false,
            |app| app.scientific.memory_recall(),
        )))
        .child(ui::flex_cell(ui::text_key(
            cx,
            "sci-m-plus",
            p,
            "M+",
            true,
            false,
            |app| app.scientific.memory_add(),
        )))
        .child(ui::flex_cell(ui::text_key(
            cx,
            "sci-m-minus",
            p,
            "M−",
            true,
            false,
            |app| app.scientific.memory_subtract(),
        )))
        .child(ui::flex_cell(ui::text_key(
            cx,
            "sci-ms",
            p,
            "MS",
            true,
            false,
            |app| app.scientific.memory_store(),
        )))
        .child(ui::flex_cell(ui::text_key(
            cx,
            "sci-md",
            p,
            "M▾",
            has_memory,
            state.memory.is_open(),
            |app| app.scientific.toggle_memory(),
        )))
        .into_any_element()
}

/// 三角函数 / 函数 两个下拉菜单。
fn render_function_row(
    state: &ScientificCalc,
    cx: &mut Context<CalcApp>,
    _p: Palette,
) -> AnyElement {
    let entity = cx.entity();
    let second = state.second;

    let trig_entity = entity.clone();
    let trig_label = if second {
        "反三角函数"
    } else {
        "三角函数"
    };
    let trig = Button::new("sci-trig")
        .label(trig_label)
        .icon(IconName::Triangle)
        .text()
        .small()
        .dropdown_caret(true)
        .dropdown_menu_with_anchor(Anchor::BottomLeft, move |menu, _, _| {
            let mut menu = menu.min_w(px(160.));
            for (func, label) in trig_items(second) {
                let app = trig_entity.clone();
                menu = menu.item(PopupMenuItem::new(label).on_click(move |_, _, cx| {
                    app.update(cx, |app, cx| {
                        app.scientific.apply_trig(func);
                        cx.notify();
                    });
                }));
            }
            menu
        });

    let math_entity = entity;
    let math = Button::new("sci-func")
        .label("函数")
        .icon(IconName::SquareFunction)
        .text()
        .small()
        .dropdown_caret(true)
        .dropdown_menu_with_anchor(Anchor::BottomLeft, move |menu, _, _| {
            let mut menu = menu.min_w(px(160.));
            for (func, label) in math_items(second) {
                let app = math_entity.clone();
                menu = menu.item(PopupMenuItem::new(label).on_click(move |_, _, cx| {
                    app.update(cx, |app, cx| {
                        app.scientific.apply_math(func);
                        cx.notify();
                    });
                }));
            }
            menu
        });

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .h(px(30.))
        .flex_none()
        .px_1()
        .child(trig)
        .child(math)
        .child(div().flex_1())
        .into_any_element()
}

/// 7 行 × 5 列按键区。
fn render_keypad(state: &ScientificCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let second = state.second;

    let digit = |cx: &mut Context<CalcApp>, id: &'static str, label: &'static str, ch: char| {
        ui::keypad_key(cx, id, p, true, false, label, move |app| {
            app.scientific.input_digit(ch)
        })
    };
    let unary = |cx: &mut Context<CalcApp>, id: &'static str, label: &'static str, u: Unary| {
        ui::keypad_key(cx, id, p, true, false, label, move |app| {
            app.scientific.apply_unary(u)
        })
    };
    let binary = |cx: &mut Context<CalcApp>, id: &'static str, label: &'static str, op: SOp| {
        ui::keypad_key(cx, id, p, true, false, label, move |app| {
            app.scientific.set_operator(op)
        })
    };
    let action = |cx: &mut Context<CalcApp>,
                  id: &'static str,
                  label: &'static str,
                  f: fn(&mut ScientificCalc)| {
        ui::keypad_key(cx, id, p, true, false, label, move |app| {
            f(&mut app.scientific)
        })
    };

    div()
        .flex()
        .flex_col()
        .gap_1()
        .flex_1()
        // 第 1 行：2nd  π  e  C  ⌫
        .child(ui::key_row(vec![
            ui::soft_key(cx, "sci-2nd", p, second, "2nd", |app| {
                app.scientific.toggle_second()
            }),
            ui::keypad_key(cx, "sci-pi", p, true, false, "π", |app| {
                app.scientific.press_const(Constant::Pi)
            }),
            ui::keypad_key(cx, "sci-e", p, true, false, "e", |app| {
                app.scientific.press_const(Constant::E)
            }),
            action(cx, "sci-clear", "C", ScientificCalc::clear),
            ui::keypad_key(
                cx,
                "sci-backspace",
                p,
                true,
                false,
                Icon::new(IconName::Delete).small(),
                |app| app.scientific.backspace(),
            ),
        ]))
        // 第 2 行：x²/x³  1/x  |x|  exp  mod
        .child(ui::key_row(vec![
            ui::keypad_key(
                cx,
                "sci-square",
                p,
                true,
                false,
                if second { "x³" } else { "x²" },
                move |app| {
                    app.scientific
                        .apply_unary(if second { Unary::Cube } else { Unary::Square })
                },
            ),
            unary(cx, "sci-reciprocal", "1/x", Unary::Reciprocal),
            unary(cx, "sci-abs", "|x|", Unary::Abs),
            binary(cx, "sci-exp", "exp", SOp::Exp),
            binary(cx, "sci-mod", "mod", SOp::Mod),
        ]))
        // 第 3 行：√x/∛x  (  )  n!  ÷
        .child(ui::key_row(vec![
            ui::keypad_key(
                cx,
                "sci-sqrt",
                p,
                true,
                false,
                if second { "³√x" } else { "²√x" },
                move |app| {
                    app.scientific
                        .apply_unary(if second { Unary::Cbrt } else { Unary::Sqrt })
                },
            ),
            action(cx, "sci-open-paren", "(", ScientificCalc::open_paren),
            action(cx, "sci-close-paren", ")", ScientificCalc::close_paren),
            unary(cx, "sci-factorial", "n!", Unary::Factorial),
            binary(cx, "sci-div", "÷", SOp::Div),
        ]))
        // 第 4 行：xʸ/ʸ√x  7  8  9  ×
        .child(ui::key_row(vec![
            ui::keypad_key(
                cx,
                "sci-pow",
                p,
                true,
                false,
                if second { "ʸ√x" } else { "xʸ" },
                move |app| {
                    if second {
                        app.scientific.set_operator(SOp::Root);
                    } else {
                        app.scientific.set_operator(SOp::Pow);
                    }
                },
            ),
            digit(cx, "sci-7", "7", '7'),
            digit(cx, "sci-8", "8", '8'),
            digit(cx, "sci-9", "9", '9'),
            binary(cx, "sci-mul", "×", SOp::Mul),
        ]))
        // 第 5 行：10ˣ/2ˣ  4  5  6  −
        .child(ui::key_row(vec![
            ui::keypad_key(
                cx,
                "sci-ten-pow",
                p,
                true,
                false,
                if second { "2ˣ" } else { "10ˣ" },
                move |app| {
                    app.scientific
                        .apply_unary(if second { Unary::TwoPow } else { Unary::TenPow })
                },
            ),
            digit(cx, "sci-4", "4", '4'),
            digit(cx, "sci-5", "5", '5'),
            digit(cx, "sci-6", "6", '6'),
            binary(cx, "sci-sub", "−", SOp::Sub),
        ]))
        // 第 6 行：log/logᵧ  1  2  3  +
        .child(ui::key_row(vec![
            ui::keypad_key(
                cx,
                "sci-log",
                p,
                true,
                false,
                if second { "logᵧ" } else { "log" },
                move |app| {
                    if second {
                        app.scientific.set_operator(SOp::LogBase);
                    } else {
                        app.scientific.apply_unary(Unary::Log10);
                    }
                },
            ),
            digit(cx, "sci-1", "1", '1'),
            digit(cx, "sci-2", "2", '2'),
            digit(cx, "sci-3", "3", '3'),
            binary(cx, "sci-add", "+", SOp::Add),
        ]))
        // 第 7 行：ln/eˣ  +/−  0  .  =
        .child(ui::key_row(vec![
            ui::keypad_key(
                cx,
                "sci-ln",
                p,
                true,
                false,
                if second { "eˣ" } else { "ln" },
                move |app| {
                    app.scientific
                        .apply_unary(if second { Unary::Exp } else { Unary::Ln })
                },
            ),
            action(cx, "sci-negate", "+/−", ScientificCalc::negate),
            digit(cx, "sci-0", "0", '0'),
            ui::keypad_key(cx, "sci-dot", p, true, false, ".", |app| {
                app.scientific.input_dot()
            }),
            ui::keypad_key(cx, "sci-equals", p, true, true, "=", |app| {
                app.scientific.equals()
            }),
        ]))
        .into_any_element()
}

/// 历史记录面板。
fn render_history(state: &ScientificCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let mut list = div()
        .id("sci-history-list")
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
        for (index, entry) in state.history.iter().enumerate().rev() {
            let expression = entry.expression.clone();
            let result = entry.result.clone();
            list = list.child(
                div()
                    .id(("sci-history", index))
                    .flex()
                    .flex_col()
                    .items_end()
                    .gap_1()
                    .p_2()
                    .rounded_md()
                    .cursor_pointer()
                    .hover(move |style| style.bg(p.hover))
                    .on_click(ui::press(cx, move |app| {
                        app.scientific.recall_history(index)
                    }))
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
                    "sci-clear-history",
                    p,
                    "清空",
                    true,
                    false,
                    |app| app.scientific.clear_history(),
                )),
        )
        .child(list)
        .into_any_element()
}

/// 记忆列表面板。
fn render_memory(state: &ScientificCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    // 最新存入的值排在最上面；格式化跟随当前的科学计数法开关。
    let items = state
        .memory
        .slots()
        .iter()
        .enumerate()
        .rev()
        .map(|(index, value)| (index, format_value(*value, state.scientific_notation)))
        .collect();

    ui::memory_panel(
        cx,
        p,
        "sci-memory",
        "sci-clear-memory",
        items,
        |index, app| app.scientific.memory_recall_at(index),
        |app| app.scientific.memory_clear(),
    )
}
