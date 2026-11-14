//! 程序员模式的界面：显示屏、进制切换、命令行、键盘与位翻转面板。

use super::{Base, BinOp, ProgrammerCalc, WordSize, format_value};
use crate::app::CalcApp;
use crate::shared::ui::{self, Palette};
use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::component::{Disableable, Icon, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

/// 绘制「程序员」模式的主体。
pub fn render(state: &ProgrammerCalc, cx: &mut Context<CalcApp>) -> AnyElement {
    let p = Palette::new(cx);

    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_1()
        .child(render_display(state, p))
        .child(render_bases(state, cx, p))
        .child(div().h(px(4.)).flex_none())
        .child(render_command_row(state, cx, p))
        .child(ui::divider(p.border))
        .child(render_operator_row(cx, p))
        .child(if state.show_bits {
            render_bits(state, cx, p)
        } else {
            render_keypad(state, cx, p)
        })
        .into_any_element()
}

/// 显示屏：右对齐的大号数字。
///
/// 64 位二进制有近 80 个字符，因此字号按长度自动缩小；
/// 仍然放不下时从左侧省略，保证最低位（正在输入的部分）可见。
fn render_display(state: &ProgrammerCalc, p: Palette) -> AnyElement {
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

/// 左侧的四行进制列表，点击可切换进制。
fn render_bases(state: &ProgrammerCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let mut list = div().flex().flex_col().flex_none();

    for base in Base::ALL {
        let selected = state.base == base;
        list = list.child(
            div()
                .id(("prog-base", base as usize))
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .h(px(22.))
                .w_full()
                .cursor_pointer()
                .hover(move |style| style.bg(p.hover))
                .on_click(ui::press(cx, move |app| app.programmer.set_base(base)))
                .child(div().w(px(3.)).h_full().rounded_full().bg(if selected {
                    p.accent
                } else {
                    transparent_black()
                }))
                .child(
                    div()
                        .w(px(40.))
                        .text_sm()
                        .font_medium()
                        .text_color(if selected { p.accent } else { p.muted })
                        .child(base.label()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_sm()
                        .truncate()
                        .text_color(p.fg)
                        .child(state.value_in(base)),
                ),
        );
    }

    list.into_any_element()
}

/// 命令行：键盘 / 位翻转页签 + QWORD + MS + M▾。
fn render_command_row(state: &ProgrammerCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let entity = cx.entity();
    let has_memory = !state.memory.is_empty();
    let memories = state.memory.slots().to_vec();
    let current_word = state.word;
    let word_for_memory = state.word;

    // QWORD 下拉：切换位宽。
    let word_menu_entity = entity.clone();
    let word_button = Button::new("prog-word")
        .label(state.word.label())
        .text()
        .small()
        .dropdown_caret(true)
        .dropdown_menu_with_anchor(Anchor::BottomLeft, move |menu, _, _| {
            let mut menu = menu.min_w(px(130.));
            for word in WordSize::ALL {
                let app = word_menu_entity.clone();
                menu = menu.item(
                    PopupMenuItem::new(word.label())
                        .checked(word == current_word)
                        .on_click(move |_, _, cx| {
                            app.update(cx, |app, cx| {
                                app.programmer.set_word(word);
                                cx.notify();
                            });
                        }),
                );
            }
            menu
        });

    // 记忆下拉：MC / MR / M+ / M− / MS + 记忆列表。
    let memory_entity = entity.clone();
    let memory_button = Button::new("prog-memory")
        .label("M")
        .text()
        .small()
        .dropdown_caret(true)
        .when(!has_memory, |this| this.disabled(true))
        .dropdown_menu_with_anchor(Anchor::BottomLeft, move |menu, _, _| {
            let app = memory_entity.clone();
            let app_mc = app.clone();
            let app_mr = app.clone();
            let app_plus = app.clone();
            let app_minus = app.clone();
            let app_ms = app.clone();

            let mut menu = menu
                .min_w(px(150.))
                .item(PopupMenuItem::new("MC").on_click(move |_, _, cx| {
                    app_mc.update(cx, |app, cx| {
                        app.programmer.memory_clear();
                        cx.notify();
                    });
                }))
                .item(PopupMenuItem::new("MR").on_click(move |_, _, cx| {
                    app_mr.update(cx, |app, cx| {
                        app.programmer.memory_recall();
                        cx.notify();
                    });
                }))
                .item(PopupMenuItem::new("M+").on_click(move |_, _, cx| {
                    app_plus.update(cx, |app, cx| {
                        app.programmer.memory_add();
                        cx.notify();
                    });
                }))
                .item(PopupMenuItem::new("M−").on_click(move |_, _, cx| {
                    app_minus.update(cx, |app, cx| {
                        app.programmer.memory_subtract();
                        cx.notify();
                    });
                }))
                .item(PopupMenuItem::new("MS").on_click(move |_, _, cx| {
                    app_ms.update(cx, |app, cx| {
                        app.programmer.memory_store();
                        cx.notify();
                    });
                }));

            if !memories.is_empty() {
                menu = menu.separator();
                for (index, value) in memories.iter().enumerate().rev() {
                    let app = app.clone();
                    let label = format_value(Base::Dec, *value, word_for_memory);
                    menu = menu.item(PopupMenuItem::new(label).on_click(move |_, _, cx| {
                        app.update(cx, |app, cx| {
                            app.programmer.memory_recall_at(index);
                            cx.notify();
                        });
                    }));
                }
            }

            menu
        });

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .h(px(36.))
        .flex_none()
        .child(ui::icon_key(
            cx,
            "prog-pane-keypad",
            p,
            IconName::Grid3x3,
            !state.show_bits,
            |app| app.programmer.set_show_bits(false),
        ))
        .child(ui::icon_key(
            cx,
            "prog-pane-bits",
            p,
            IconName::Binary,
            state.show_bits,
            |app| app.programmer.set_show_bits(true),
        ))
        .child(div().flex_1())
        .child(word_button)
        .child(div().w(px(6.)))
        .child(ui::text_key(cx, "prog-ms", p, "MS", true, false, |app| {
            app.programmer.memory_store()
        }))
        .child(memory_button)
        .into_any_element()
}

/// 「按位」「位移位」两个下拉按钮。
fn render_operator_row(cx: &mut Context<CalcApp>, _p: Palette) -> AnyElement {
    let entity = cx.entity();

    let bitwise_entity = entity.clone();
    let bitwise = Button::new("prog-bitwise")
        .label("按位")
        .text()
        .small()
        .dropdown_caret(true)
        .dropdown_menu_with_anchor(Anchor::BottomLeft, move |menu, _, _| {
            let app = bitwise_entity.clone();
            let app_not = app.clone();
            let mut menu = menu
                .min_w(px(120.))
                .item(PopupMenuItem::new("NOT").on_click(move |_, _, cx| {
                    app_not.update(cx, |app, cx| {
                        app.programmer.bitwise_not();
                        cx.notify();
                    });
                }));
            menu = menu.separator();
            for op in [BinOp::And, BinOp::Or, BinOp::Xor, BinOp::Nand, BinOp::Nor] {
                let app = app.clone();
                menu = menu.item(PopupMenuItem::new(op.symbol()).on_click(move |_, _, cx| {
                    app.update(cx, |app, cx| {
                        app.programmer.set_operator(op);
                        cx.notify();
                    });
                }));
            }
            menu
        });

    let shift_entity = entity;
    let shift = Button::new("prog-shift")
        .label("位移位")
        .text()
        .small()
        .dropdown_caret(true)
        .dropdown_menu_with_anchor(Anchor::BottomLeft, move |menu, _, _| {
            let mut menu = menu.min_w(px(120.));
            for op in [BinOp::Lsh, BinOp::Rsh] {
                let app = shift_entity.clone();
                menu = menu.item(PopupMenuItem::new(op.symbol()).on_click(move |_, _, cx| {
                    app.update(cx, |app, cx| {
                        app.programmer.set_operator(op);
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
        .child(bitwise)
        .child(shift)
        .child(div().flex_1())
        .into_any_element()
}

/// 6 行 × 5 列按键区。
fn render_keypad(state: &ProgrammerCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let radix = state.base.radix();
    let hex_enabled = state.base == Base::Hex;

    // 生成一个数字键（0–9 / A–F）。
    let digit_key = |cx: &mut Context<CalcApp>, ch: char, enabled: bool| {
        let label = ch.to_string();
        let id: ElementId = SharedString::from(format!("prog-key-{ch}")).into();
        ui::keypad_key(cx, id, p, enabled, false, label, move |app| {
            app.programmer.input_digit(ch)
        })
    };

    // 生成一个运算符 / 功能键。
    let op_key = |cx: &mut Context<CalcApp>, id: &'static str, label: &'static str, op: BinOp| {
        ui::keypad_key(cx, id, p, true, false, label, move |app| {
            app.programmer.set_operator(op)
        })
    };

    let func_key = |cx: &mut Context<CalcApp>,
                    id: &'static str,
                    label: &'static str,
                    action: fn(&mut ProgrammerCalc)| {
        ui::keypad_key(cx, id, p, true, false, label, move |app| {
            action(&mut app.programmer)
        })
    };

    div()
        .flex()
        .flex_col()
        .gap_1()
        .flex_1()
        // 第 1 行：A  <<  >>  C  ⌫
        .child(ui::key_row(vec![
            digit_key(cx, 'A', hex_enabled),
            op_key(cx, "prog-lsh", "<<", BinOp::Lsh),
            op_key(cx, "prog-rsh", ">>", BinOp::Rsh),
            func_key(cx, "prog-clear", "C", ProgrammerCalc::clear),
            ui::keypad_key(
                cx,
                "prog-backspace",
                p,
                true,
                false,
                Icon::new(IconName::Delete).small(),
                |app| app.programmer.backspace(),
            ),
        ]))
        // 第 2 行：B  (  )  %  ÷
        .child(ui::key_row(vec![
            digit_key(cx, 'B', hex_enabled),
            func_key(cx, "prog-open-paren", "(", ProgrammerCalc::open_paren),
            func_key(cx, "prog-close-paren", ")", ProgrammerCalc::close_paren),
            op_key(cx, "prog-mod", "%", BinOp::Mod),
            op_key(cx, "prog-div", "÷", BinOp::Div),
        ]))
        // 第 3 行：C  7  8  9  ×
        .child(ui::key_row(vec![
            digit_key(cx, 'C', hex_enabled),
            digit_key(cx, '7', 7 < radix),
            digit_key(cx, '8', 8 < radix),
            digit_key(cx, '9', 9 < radix),
            op_key(cx, "prog-mul", "×", BinOp::Mul),
        ]))
        // 第 4 行：D  4  5  6  −
        .child(ui::key_row(vec![
            digit_key(cx, 'D', hex_enabled),
            digit_key(cx, '4', true),
            digit_key(cx, '5', true),
            digit_key(cx, '6', true),
            op_key(cx, "prog-sub", "−", BinOp::Sub),
        ]))
        // 第 5 行：E  1  2  3  +
        .child(ui::key_row(vec![
            digit_key(cx, 'E', hex_enabled),
            digit_key(cx, '1', true),
            digit_key(cx, '2', true),
            digit_key(cx, '3', true),
            op_key(cx, "prog-add", "+", BinOp::Add),
        ]))
        // 第 6 行：F  +/−  0  .  =
        .child(ui::key_row(vec![
            digit_key(cx, 'F', hex_enabled),
            func_key(cx, "prog-negate", "+/−", ProgrammerCalc::negate),
            digit_key(cx, '0', true),
            ui::keypad_key(cx, "prog-dot", p, false, false, ".", |_| {}),
            ui::keypad_key(cx, "prog-equals", p, true, true, "=", |app| {
                app.programmer.equals()
            }),
        ]))
        .into_any_element()
}

/// 位翻转面板：每个按钮对应一位，点击即可翻转。
fn render_bits(state: &ProgrammerCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let bits = state.word.bits();
    let bits_per_row = 16.min(bits);
    let rows = bits / bits_per_row;
    let value = state.value();

    let mut panel = div().flex().flex_col().gap_1().flex_1().pt_1();

    for row in 0..rows {
        let mut line = div().flex().flex_row().gap_1().flex_1();
        let start = bits - 1 - row * bits_per_row;
        for column in 0..bits_per_row {
            let bit = start - column;
            let set = value & (1u64 << bit) != 0;
            line = line.child(
                div()
                    .id(("prog-bit", bit as usize))
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_1()
                    .h_full()
                    .rounded_sm()
                    .border_1()
                    .border_color(p.border)
                    .bg(if set { p.accent_soft } else { p.surface })
                    .text_sm()
                    .text_color(if set { p.accent } else { p.muted })
                    .cursor_pointer()
                    .hover(move |style| style.bg(p.hover))
                    .on_click(ui::press(cx, move |app| app.programmer.toggle_bit(bit)))
                    .child(if set { "1" } else { "0" }),
            );
            // 每 4 位留一点空隙，和图里的分组一致。
            if (column + 1) % 4 == 0 && column + 1 < bits_per_row {
                line = line.child(div().w(px(6.)).flex_none());
            }
        }
        panel = panel.child(line);
    }

    panel
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .pt_2()
                .text_xs()
                .text_color(p.muted)
                .child(format!("{bits} 位 · 点击任意位翻转", bits = bits)),
        )
        .into_any_element()
}
