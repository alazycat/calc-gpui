//! 各模式共用的配色与基础控件。
//!
//! 这里不做业务计算，只负责「看起来像 Windows 计算器」的那一层：
//! 取主题色、画按键、画分隔线、画图标按钮。

use crate::app::CalcApp;
use gpui_kit::component::{ActiveTheme, Icon, Sizable, StyledExt, Theme};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

/// 从主题里取出的一组颜色。
///
/// 计算器的按钮、显示屏、分隔线都从这里拿颜色，这样切换到深色主题时
/// 不用改任何布局代码。
#[derive(Clone, Copy)]
pub struct Palette {
    /// 窗口背景。
    pub bg: Hsla,
    /// 主文字颜色。
    pub fg: Hsla,
    /// 次要文字颜色（进制名、小号算式等）。
    pub muted: Hsla,
    /// 边框颜色。
    pub border: Hsla,
    /// 按键表面。
    pub surface: Hsla,
    /// 稍暗的表面（禁用的功能键、记忆行背景）。
    pub surface_alt: Hsla,
    /// 悬停时的按键背景。
    pub hover: Hsla,
    /// 强调色（进制指示条、等号键、当前标签）。
    pub accent: Hsla,
    /// 强调色的悬停版本。
    pub accent_hover: Hsla,
    /// 强调色的浅色底。
    pub accent_soft: Hsla,
    /// 强调色上的文字。
    pub accent_fg: Hsla,
    /// 错误提示色。
    pub danger: Hsla,
}

/// 把品牌蓝色写回主题。
///
/// `Theme::change` 会从主题配置重新加载颜色，因此每次切换主题后都要再调用一次。
pub fn apply_brand_colors(cx: &mut App) {
    let blue: Hsla = rgb(0x0f6cbd).into();
    let hover: Hsla = rgb(0x1a7fd4).into();
    let active: Hsla = rgb(0x0c5a9e).into();
    let foreground: Hsla = rgb(0xffffff).into();

    let theme = Theme::global_mut(cx);
    theme.colors.button_primary = blue;
    theme.colors.button_primary_hover = hover;
    theme.colors.button_primary_active = active;
    theme.colors.button_primary_foreground = foreground;
    theme.tokens.button_primary = blue.into();
    theme.tokens.button_primary_hover = hover.into();
    theme.tokens.button_primary_active = active.into();
    theme.tokens.button_primary_foreground = foreground.into();
}

impl Palette {
    pub fn new(cx: &App) -> Self {
        let theme = cx.theme();
        // 计算器的强调色来自 button_primary（main 里覆盖成蓝色），
        // 而不是主题的 primary，避免等号键变成默认的黑色。
        let accent = theme.button_primary;
        Self {
            bg: theme.background,
            fg: theme.foreground,
            muted: theme.muted_foreground,
            border: theme.border,
            surface: theme.button,
            surface_alt: theme.muted,
            hover: theme.button_hover,
            accent,
            accent_hover: theme.button_primary_hover,
            accent_soft: Hsla { a: 0.12, ..accent },
            accent_fg: theme.button_primary_foreground,
            danger: theme.danger,
        }
    }

    /// 给一个颜色套上透明度（用于禁用态文字）。
    pub fn faded(&self, color: Hsla) -> Hsla {
        Hsla { a: 0.45, ..color }
    }
}

/// 点击后对 `CalcApp` 执行一个动作，并请求重绘。
///
/// GPUI 不会因为状态变化自动重绘，所以每个回调都要显式 `notify`。
pub fn press(
    cx: &mut Context<CalcApp>,
    action: impl Fn(&mut CalcApp) + 'static,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    cx.listener(move |app: &mut CalcApp, _event: &ClickEvent, _window, cx| {
        action(app);
        cx.notify();
    })
}

/// 计算器按键。
///
/// - `enabled == false` 时按钮变灰且不响应点击（例如非 HEX 进制下的 A–F）；
/// - `primary` 为真时使用强调色（等号键）。
pub fn keypad_key(
    cx: &mut Context<CalcApp>,
    id: impl Into<ElementId>,
    p: Palette,
    enabled: bool,
    primary: bool,
    content: impl IntoElement + 'static,
    action: impl Fn(&mut CalcApp) + 'static,
) -> AnyElement {
    let (bg, fg, border, hover) = if primary {
        (p.accent, p.accent_fg, p.accent, p.accent_hover)
    } else if enabled {
        (p.surface, p.fg, p.border, p.hover)
    } else {
        (p.surface_alt, p.muted, p.border, p.surface_alt)
    };

    let key = div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .flex_1()
        .h_full()
        .rounded_sm()
        .border_1()
        .border_color(border)
        .bg(bg)
        .text_color(fg)
        .text_size(rems(1.05))
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_click(press(cx, action))
        })
        .child(content);

    key.into_any_element()
}

/// 可切换状态的功能键（例如科学模式的 `2nd`）：激活时浅蓝底 + 强调色文字。
pub fn soft_key(
    cx: &mut Context<CalcApp>,
    id: impl Into<ElementId>,
    p: Palette,
    active: bool,
    content: impl IntoElement + 'static,
    action: impl Fn(&mut CalcApp) + 'static,
) -> AnyElement {
    let (bg, fg, border) = if active {
        (p.accent_soft, p.accent, p.accent)
    } else {
        (p.surface, p.fg, p.border)
    };
    let hover = if active { p.accent_soft } else { p.hover };

    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .flex_1()
        .h_full()
        .rounded_sm()
        .border_1()
        .border_color(border)
        .bg(bg)
        .text_color(fg)
        .text_size(rems(1.05))
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(press(cx, action))
        .child(content)
        .into_any_element()
}

/// 按字符数估算显示屏字号（可用宽度约 360 逻辑像素）。
///
/// 科学计数法和 64 位数字都可能很长，统一用这个函数避免溢出。
pub fn display_size(text: &str) -> Rems {
    let len = text.chars().count().max(1) as f32;
    let size = (360.0 / (len * 0.62)).clamp(8.0, 41.0);
    rems(size / 16.0)
}

/// 与 [`keypad_key`] 相同，但回调可以访问 [`Context`]（例如操作剪贴板）。
pub fn keypad_key_ctx(
    cx: &mut Context<CalcApp>,
    id: impl Into<ElementId>,
    p: Palette,
    enabled: bool,
    primary: bool,
    content: impl IntoElement + 'static,
    action: impl Fn(&mut CalcApp, &mut Context<CalcApp>) + 'static,
) -> AnyElement {
    let (bg, fg, border, hover) = if primary {
        (p.accent, p.accent_fg, p.accent, p.accent_hover)
    } else if enabled {
        (p.surface, p.fg, p.border, p.hover)
    } else {
        (p.surface_alt, p.muted, p.border, p.surface_alt)
    };

    let key = div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .flex_1()
        .h_full()
        .rounded_sm()
        .border_1()
        .border_color(border)
        .bg(bg)
        .text_color(fg)
        .text_size(rems(1.05))
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover))
                .on_click(
                    cx.listener(move |app: &mut CalcApp, _: &ClickEvent, _window, cx| {
                        action(app, cx);
                        cx.notify();
                    }),
                )
        })
        .child(content);

    key.into_any_element()
}

/// 一行 5 个（或 4 个）按键。
pub fn key_row(keys: Vec<AnyElement>) -> AnyElement {
    let mut row = div().flex().flex_row().gap_1().flex_1();
    for key in keys {
        row = row.child(key);
    }
    row.into_any_element()
}

/// 小号文字按钮（MC / MR / M+ / M− / MS / QWORD / 按位 …）。
pub fn text_key(
    cx: &mut Context<CalcApp>,
    id: impl Into<ElementId>,
    p: Palette,
    label: impl Into<SharedString> + 'static,
    enabled: bool,
    active: bool,
    action: impl Fn(&mut CalcApp) + 'static,
) -> AnyElement {
    let label: SharedString = label.into();
    let fg = if !enabled {
        p.faded(p.muted)
    } else if active {
        p.accent
    } else {
        p.fg
    };

    let key = div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .h(px(28.))
        .px_2()
        .rounded_sm()
        .text_sm()
        .text_color(fg)
        .when(enabled, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(p.hover))
                .on_click(press(cx, action))
        })
        .child(label);

    key.into_any_element()
}

/// 顶栏 / 命令行的图标按钮。
///
/// `active` 为真时在底部画一条强调色指示条（对应图中的键盘 / 位切换页签）。
pub fn icon_key(
    cx: &mut Context<CalcApp>,
    id: impl Into<ElementId>,
    p: Palette,
    icon: impl Into<Icon> + 'static,
    active: bool,
    action: impl Fn(&mut CalcApp) + 'static,
) -> AnyElement {
    let fg = if active { p.fg } else { p.muted };
    let icon = icon.into().small().text_color(fg);

    div()
        .id(id)
        .flex()
        .flex_col()
        .items_center()
        .justify_end()
        .w(px(40.))
        .h_full()
        .cursor_pointer()
        .rounded_sm()
        .hover(move |style| style.bg(p.hover))
        .on_click(press(cx, action))
        .child(
            div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .child(icon),
        )
        .child(div().w(px(22.)).h(px(3.)).rounded_full().bg(if active {
            p.accent
        } else {
            transparent_black()
        }))
        .into_any_element()
}

/// 把子元素放进一个等宽的格子里（记忆行用）。
pub fn flex_cell(child: AnyElement) -> AnyElement {
    div()
        .flex_1()
        .flex()
        .justify_center()
        .child(child)
        .into_any_element()
}

/// 一条 1px 分隔线。
pub fn divider(color: Hsla) -> AnyElement {
    div()
        .w_full()
        .h(px(1.))
        .flex_none()
        .bg(color)
        .into_any_element()
}

/// 记忆列表面板。
///
/// 最新存入的槽排在最上面，点击任意一行取回该值。三个模式的面板长得一样，
/// 只有「值怎么格式化」不同，所以 `items` 由调用方给出已格式化的文本。
///
/// - `row_id`：每一行的 id 前缀，实际 id 是 `(row_id, 槽位下标)`；
/// - `clear_id`：「全部清除」按钮的 id。
pub fn memory_panel<R, C>(
    cx: &mut Context<CalcApp>,
    p: Palette,
    row_id: &'static str,
    clear_id: &'static str,
    items: Vec<(usize, String)>,
    on_recall: R,
    on_clear: C,
) -> AnyElement
where
    R: Fn(usize, &mut CalcApp) + Clone + 'static,
    C: Fn(&mut CalcApp) + 'static,
{
    let mut list = div()
        .id(SharedString::from(format!("{row_id}-list")))
        .flex()
        .flex_col()
        .gap_1()
        .flex_1()
        .overflow_y_scroll();

    if items.is_empty() {
        list = list.child(
            div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(p.muted)
                .child("记忆是空的"),
        );
    } else {
        for (index, text) in items {
            let recall = on_recall.clone();
            list = list.child(
                div()
                    .id((row_id, index))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_end()
                    .p_2()
                    .rounded_md()
                    .cursor_pointer()
                    .hover(move |style| style.bg(p.hover))
                    .on_click(press(cx, move |app| recall(index, app)))
                    .child(div().text_lg().font_medium().text_color(p.fg).child(text)),
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
                        .child("记忆"),
                )
                .child(text_key(cx, clear_id, p, "全部清除", true, false, on_clear)),
        )
        .child(list)
        .into_any_element()
}
