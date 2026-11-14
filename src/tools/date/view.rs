//! 日期计算的界面：模式页签、日期卡片、步进器与结果展示。

use super::{AmountPart, Date, DateCalc, DateMode, DatePart, DateTarget};
use crate::app::CalcApp;
use crate::shared::ui::{self, Palette};
use gpui_kit::component::StyledExt;
use gpui_kit::*;

/// 绘制日期计算页面。
pub fn render(state: &DateCalc, cx: &mut Context<CalcApp>) -> AnyElement {
    let p = Palette::new(cx);

    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_2()
        .child(render_mode_tabs(state, cx, p))
        .child(match state.mode {
            DateMode::Difference => render_difference(state, cx, p),
            DateMode::Offset => render_offset(state, cx, p),
        })
        .child(
            div()
                .pt_1()
                .text_xs()
                .text_color(p.muted)
                .child("日期按公历计算，「今天」取自 UTC 时间。"),
        )
        .into_any_element()
}

/// 日期差异 / 加减小日期 两个页签。
fn render_mode_tabs(state: &DateCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .h(px(30.))
        .flex_none()
        .child(ui::text_key(
            cx,
            "date-tab-diff",
            p,
            "日期差异",
            true,
            state.mode == DateMode::Difference,
            |app| app.date.set_mode(DateMode::Difference),
        ))
        .child(ui::text_key(
            cx,
            "date-tab-offset",
            p,
            "加/减日期",
            true,
            state.mode == DateMode::Offset,
            |app| app.date.set_mode(DateMode::Offset),
        ))
        .child(div().flex_1())
        .into_any_element()
}

fn render_difference(state: &DateCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let diff = state.difference();
    let weeks = diff.total_days / 7;
    let days = diff.total_days % 7;

    div()
        .flex()
        .flex_col()
        .gap_2()
        .flex_1()
        .child(date_card(
            cx,
            p,
            "date-start",
            "开始日期",
            state.start(),
            DateTarget::Start,
        ))
        .child(date_card(
            cx,
            p,
            "date-end",
            "结束日期",
            state.end(),
            DateTarget::End,
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .p_3()
                .rounded_lg()
                .bg(p.surface_alt)
                .flex_none()
                .child(div().text_sm().text_color(p.muted).child("相差"))
                .child(
                    div()
                        .text_size(rems(1.8))
                        .line_height(relative(1.2))
                        .font_semibold()
                        .text_color(p.fg)
                        .child(format!(
                            "{} 年 {} 个月 {} 天",
                            diff.years, diff.months, diff.days
                        )),
                )
                .child(div().text_sm().text_color(p.muted).child(format!(
                    "共 {} 天 · {} 周 {} 天",
                    diff.total_days, weeks, days
                )))
                .child(div().text_xs().text_color(p.muted).child(format!(
                    "{} {} → {} {}",
                    state.start().format_cn(),
                    state.start().weekday(),
                    state.end().format_cn(),
                    state.end().weekday()
                ))),
        )
        .into_any_element()
}

fn render_offset(state: &DateCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    let result = state.result_date();

    div()
        .flex()
        .flex_col()
        .gap_2()
        .flex_1()
        .child(date_card(
            cx,
            p,
            "date-offset",
            "起始日期",
            state.start(),
            DateTarget::Start,
        ))
        .child(amount_card(state, cx, p))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .p_3()
                .rounded_lg()
                .bg(p.surface_alt)
                .flex_none()
                .child(div().text_sm().text_color(p.muted).child("结果日期"))
                .child(
                    div()
                        .text_size(rems(1.8))
                        .line_height(relative(1.2))
                        .font_semibold()
                        .text_color(p.fg)
                        .child(result.format_cn()),
                )
                .child(div().text_sm().text_color(p.muted).child(format!(
                    "{} · 第 {} 天 / 共 {} 天",
                    result.weekday(),
                    result.day_of_year(),
                    result.days_in_year()
                ))),
        )
        .into_any_element()
}

/// 一张日期卡片：年 / 月 / 日三个步进器 + 「今天」。
fn date_card(
    cx: &mut Context<CalcApp>,
    p: Palette,
    prefix: &'static str,
    title: &'static str,
    date: Date,
    target: DateTarget,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .p_3()
        .rounded_lg()
        .bg(p.surface_alt)
        .flex_none()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(div().text_sm().text_color(p.muted).child(title))
                .child(ui::text_key(
                    cx,
                    (prefix, 9usize),
                    p,
                    "今天",
                    true,
                    false,
                    move |app| app.date.set_today(target),
                )),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap_2()
                .child(stepper(
                    cx,
                    p,
                    prefix,
                    0,
                    "年",
                    date.year.to_string(),
                    (target, DatePart::Year),
                ))
                .child(stepper(
                    cx,
                    p,
                    prefix,
                    1,
                    "月",
                    date.month.to_string(),
                    (target, DatePart::Month),
                ))
                .child(stepper(
                    cx,
                    p,
                    prefix,
                    2,
                    "日",
                    date.day.to_string(),
                    (target, DatePart::Day),
                )),
        )
        .into_any_element()
}

/// 一个字段的步进器：− 值 +。
fn stepper(
    cx: &mut Context<CalcApp>,
    p: Palette,
    prefix: &'static str,
    slot: usize,
    label: &'static str,
    value: String,
    step: (DateTarget, DatePart),
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_center()
                .text_color(p.muted)
                .child(label),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(ui::text_key(
                    cx,
                    (prefix, slot * 2),
                    p,
                    "−",
                    true,
                    false,
                    move |app| app.date.step_date(step.0, step.1, -1),
                ))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_center()
                        .text_lg()
                        .font_medium()
                        .text_color(p.fg)
                        .child(value),
                )
                .child(ui::text_key(
                    cx,
                    (prefix, slot * 2 + 1),
                    p,
                    "+",
                    true,
                    false,
                    move |app| app.date.step_date(step.0, step.1, 1),
                )),
        )
        .into_any_element()
}

/// 加减量卡片：年 / 月 / 周 / 天。
fn amount_card(state: &DateCalc, cx: &mut Context<CalcApp>, p: Palette) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_3()
        .rounded_lg()
        .bg(p.surface_alt)
        .flex_none()
        .child(div().text_sm().text_color(p.muted).child("加减"))
        .child(
            div()
                .flex()
                .flex_row()
                .gap_2()
                .child(amount_stepper(
                    cx,
                    p,
                    0,
                    "年",
                    state.years(),
                    AmountPart::Years,
                ))
                .child(amount_stepper(
                    cx,
                    p,
                    1,
                    "月",
                    state.months(),
                    AmountPart::Months,
                )),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap_2()
                .child(amount_stepper(
                    cx,
                    p,
                    2,
                    "周",
                    state.weeks(),
                    AmountPart::Weeks,
                ))
                .child(amount_stepper(
                    cx,
                    p,
                    3,
                    "天",
                    state.days(),
                    AmountPart::Days,
                )),
        )
        .into_any_element()
}

fn amount_stepper(
    cx: &mut Context<CalcApp>,
    p: Palette,
    slot: usize,
    label: &'static str,
    value: i32,
    part: AmountPart,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_center()
                .text_color(p.muted)
                .child(label),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(ui::text_key(
                    cx,
                    ("date-amount", slot * 2),
                    p,
                    "−",
                    true,
                    false,
                    move |app| app.date.step_amount(part, -1),
                ))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_center()
                        .text_lg()
                        .font_medium()
                        .text_color(p.fg)
                        .child(value.to_string()),
                )
                .child(ui::text_key(
                    cx,
                    ("date-amount", slot * 2 + 1),
                    p,
                    "+",
                    true,
                    false,
                    move |app| app.date.step_amount(part, 1),
                )),
        )
        .into_any_element()
}
