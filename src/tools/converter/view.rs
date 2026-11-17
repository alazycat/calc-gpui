//! 换算器的界面：显示屏、单位选择行与数字键盘。

use super::{Action, Category, ConverterState};
use crate::app::CalcApp;
use crate::shared::number::format_plain;
use crate::shared::ui::{self, Palette};
use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::component::{Icon, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

/// 绘制当前换算页面。
pub fn render(state: &ConverterState, cx: &mut Context<CalcApp>) -> AnyElement {
    let p = Palette::new(cx);
    let category = state.kind.category();

    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_2()
        .child(render_display(state, p))
        .child(render_unit_row(state, cx, p, category))
        .when_some(category.note, |this, note| {
            this.child(div().px_1().text_xs().text_color(p.muted).child(note))
        })
        .child(render_keypad(cx, p))
        .into_any_element()
}

/// 输入值与换算结果。
fn render_display(state: &ConverterState, p: Palette) -> AnyElement {
    let input = state.input_text().to_string();
    let result = format_plain(state.result());

    div()
        .flex()
        .flex_col()
        .justify_center()
        .gap_2()
        .w_full()
        .h(px(128.))
        .flex_none()
        .rounded_lg()
        .bg(p.surface_alt)
        .px_4()
        .py_3()
        .overflow_hidden()
        .child(
            div()
                .flex()
                .flex_row()
                .items_end()
                .justify_end()
                .gap_2()
                .w_full()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_right()
                        .text_size(ui::display_size(&input))
                        .line_height(relative(1.1))
                        .font_semibold()
                        .truncate()
                        .text_color(p.fg)
                        .child(input),
                )
                .child(
                    div()
                        .flex_none()
                        .pb_1()
                        .text_sm()
                        .text_color(p.muted)
                        .child(state.source_unit_name()),
                ),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_end()
                .justify_end()
                .gap_2()
                .w_full()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_right()
                        .text_size(rems(1.4))
                        .line_height(relative(1.15))
                        .font_medium()
                        .truncate()
                        .text_color(p.accent)
                        .child(result),
                )
                .child(
                    div()
                        .flex_none()
                        .pb_1()
                        .text_sm()
                        .text_color(p.muted)
                        .child(state.target_unit_name()),
                ),
        )
        .into_any_element()
}

/// 源单位 / 交换 / 目标单位。
fn render_unit_row(
    state: &ConverterState,
    cx: &mut Context<CalcApp>,
    _p: Palette,
    category: &'static Category,
) -> AnyElement {
    let entity = cx.entity();
    let units = category.units;
    let current_from = state.from();

    let from_entity = entity.clone();
    let from_menu = Button::new("conv-from")
        .label(state.source_unit_name())
        .secondary()
        .small()
        .flex_1()
        .dropdown_caret(true)
        .dropdown_menu_with_anchor(Anchor::BottomLeft, move |menu, _, _| {
            let mut menu = menu
                .min_w(px(220.))
                .max_h(px(360.))
                .scrollable(true)
                .label(category.title);
            for (index, unit) in units.iter().enumerate() {
                let app = from_entity.clone();
                menu = menu.item(
                    PopupMenuItem::new(unit.name)
                        .checked(index == current_from)
                        .on_click(move |_, _, cx| {
                            app.update(cx, |app, cx| {
                                app.converter.set_from(index);
                                cx.notify();
                            });
                        }),
                );
            }
            menu
        });

    let to_entity = entity;
    let current_to = state.to();
    let to_menu = Button::new("conv-to")
        .label(state.target_unit_name())
        .secondary()
        .small()
        .flex_1()
        .dropdown_caret(true)
        .dropdown_menu_with_anchor(Anchor::BottomLeft, move |menu, _, _| {
            let mut menu = menu
                .min_w(px(220.))
                .max_h(px(360.))
                .scrollable(true)
                .label(category.title);
            for (index, unit) in units.iter().enumerate() {
                let app = to_entity.clone();
                menu = menu.item(
                    PopupMenuItem::new(unit.name)
                        .checked(index == current_to)
                        .on_click(move |_, _, cx| {
                            app.update(cx, |app, cx| {
                                app.converter.set_to(index);
                                cx.notify();
                            });
                        }),
                );
            }
            menu
        });

    let swap = Button::new("conv-swap")
        .icon(IconName::ArrowLeftRight)
        .tooltip("交换单位")
        .secondary()
        .small()
        .on_click(ui::press(cx, |app| app.converter.swap_units()));

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .flex_none()
        .child(from_menu)
        .child(swap)
        .child(to_menu)
        .into_any_element()
}

/// 数字键盘：7-9 / 4-6 / 1-3 / 0 . ⇄。
fn render_keypad(cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let digit = |cx: &mut Context<CalcApp>, id: &'static str, ch: char| {
        let label = ch.to_string();
        ui::keypad_key(cx, id, p, true, false, label, move |app| {
            app.converter.input_digit(ch)
        })
    };
    let action = |cx: &mut Context<CalcApp>,
                  id: &'static str,
                  label: &'static str,
                  f: fn(&mut ConverterState)| {
        ui::keypad_key(cx, id, p, true, false, label, move |app| {
            f(&mut app.converter)
        })
    };

    div()
        .flex()
        .flex_col()
        .gap_1()
        .flex_1()
        .child(ui::key_row(vec![
            digit(cx, "conv-7", '7'),
            digit(cx, "conv-8", '8'),
            digit(cx, "conv-9", '9'),
            ui::keypad_key(
                cx,
                "conv-backspace",
                p,
                true,
                false,
                Icon::new(IconName::Delete).small(),
                |app| app.converter.backspace(),
            ),
        ]))
        .child(ui::key_row(vec![
            digit(cx, "conv-4", '4'),
            digit(cx, "conv-5", '5'),
            digit(cx, "conv-6", '6'),
            action(cx, "conv-clear", "C", ConverterState::clear),
        ]))
        .child(ui::key_row(vec![
            digit(cx, "conv-1", '1'),
            digit(cx, "conv-2", '2'),
            digit(cx, "conv-3", '3'),
            action(cx, "conv-negate", "+/−", ConverterState::negate),
        ]))
        .child(ui::key_row(vec![
            digit(cx, "conv-0", '0'),
            ui::keypad_key(cx, "conv-dot", p, true, false, ".", |app| {
                app.converter.input_dot()
            }),
            ui::keypad_key(cx, "conv-swap-key", p, true, false, "⇄", |app| {
                app.converter.swap_units()
            }),
            ui::keypad_key_ctx(cx, "conv-copy", p, true, false, "复制", |app, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(app.converter.copy_text()));
            }),
        ]))
        .into_any_element()
}

/// 把按键翻译成语义动作。换算器键盘之外能用的键很少：
/// 数字、小数点、负号，以及清零（Esc / Delete）与退格。
pub fn keystroke_to_action(keystroke: &Keystroke) -> Option<Action> {
    let modifiers = &keystroke.modifiers;
    if modifiers.control || modifiers.alt || modifiers.platform {
        return None;
    }

    let typed = keystroke.key_char.as_deref().unwrap_or("");
    let mut iter = typed.chars();
    if let (Some(c), None) = (iter.next(), iter.next()) {
        let action = match c {
            '0'..='9' => Action::Digit(c),
            '.' => Action::Dot,
            '-' => Action::Negate,
            _ => return None,
        };
        return Some(action);
    }

    match keystroke.key.as_str() {
        "escape" | "delete" => Some(Action::Clear),
        "backspace" => Some(Action::Backspace),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, ConverterState, keystroke_to_action};
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
    fn digits_dot_and_negate_map_to_actions() {
        assert_eq!(keystroke_to_action(&typed('7')), Some(Action::Digit('7')));
        assert_eq!(keystroke_to_action(&typed('0')), Some(Action::Digit('0')));
        assert_eq!(keystroke_to_action(&typed('.')), Some(Action::Dot));
        assert_eq!(keystroke_to_action(&typed('-')), Some(Action::Negate));
    }

    #[test]
    fn clear_keys_and_backspace_map_to_actions() {
        assert_eq!(keystroke_to_action(&named("escape")), Some(Action::Clear));
        assert_eq!(keystroke_to_action(&named("delete")), Some(Action::Clear));
        assert_eq!(
            keystroke_to_action(&named("backspace")),
            Some(Action::Backspace)
        );
    }

    #[test]
    fn unknown_and_modifier_keys_are_ignored() {
        // 换算器不接受的字符。
        assert_eq!(keystroke_to_action(&typed('+')), None);
        assert_eq!(keystroke_to_action(&typed('a')), None);
        // 不在映射里的功能键。
        assert_eq!(keystroke_to_action(&named("enter")), None);
        assert_eq!(keystroke_to_action(&named("up")), None);
        // 任何组合键都不参与输入。
        let mut ctrl = typed('c');
        ctrl.modifiers.control = true;
        assert_eq!(keystroke_to_action(&ctrl), None);
        let mut alt = typed('1');
        alt.modifiers.alt = true;
        assert_eq!(keystroke_to_action(&alt), None);
    }

    #[test]
    fn keystrokes_drive_the_state_end_to_end() {
        let mut state = ConverterState::new();
        for keystroke in [typed('1'), typed('2'), typed('.'), typed('5'), typed('-')] {
            if let Some(action) = keystroke_to_action(&keystroke) {
                state.apply_action(action);
            }
        }
        assert_eq!(state.input_text(), "-12.5");
        assert!((state.value() + 12.5).abs() < 1e-9);

        if let Some(action) = keystroke_to_action(&named("backspace")) {
            state.apply_action(action);
        }
        assert_eq!(state.input_text(), "-12.");

        if let Some(action) = keystroke_to_action(&named("escape")) {
            state.apply_action(action);
        }
        assert_eq!(state.input_text(), "0");
    }
}
