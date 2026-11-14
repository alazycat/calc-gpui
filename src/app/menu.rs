//! 汉堡菜单：开关、键盘导航、确认，以及菜单面板的渲染。

use super::CalcApp;
use super::page::{MODE_COUNT, Page, menu_pages};
use crate::shared::ui::{self, Palette};
use gpui_kit::assets::IconName;
use gpui_kit::component::{Icon, Sizable};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl CalcApp {
    /// 打开 / 关闭汉堡菜单（F10 或点击菜单按钮）。
    pub(super) fn toggle_menu(&mut self) {
        if self.menu_open {
            self.close_menu();
        } else {
            self.menu_open = true;
            self.menu_index = menu_pages()
                .iter()
                .position(|page| *page == self.page)
                .unwrap_or(0);
            self.menu_scroll
                .scroll_to_item(self.menu_child_index(self.menu_index));
        }
    }

    /// 菜单选中项在滚动列表里的子节点下标（前面有分组标题）。
    fn menu_child_index(&self, index: usize) -> usize {
        if index < MODE_COUNT {
            index + 1
        } else {
            index + 3
        }
    }

    pub(super) fn close_menu(&mut self) {
        self.menu_open = false;
    }

    /// 键盘上下移动菜单选中项，到边界后循环。
    pub(super) fn move_menu(&mut self, delta: i32) {
        let len = menu_pages().len() as i32;
        self.menu_index = ((self.menu_index as i32 + delta).rem_euclid(len)) as usize;
        self.menu_scroll
            .scroll_to_item(self.menu_child_index(self.menu_index));
    }

    /// Enter / 空格确认当前菜单项。
    pub(super) fn activate_menu(&mut self) {
        if let Some(&page) = menu_pages().get(self.menu_index) {
            self.set_page(page);
            self.menu_open = false;
        }
    }

    /// 汉堡菜单：点击遮罩或按 Esc 关闭，方向键 + Enter 选择。
    pub(super) fn render_menu(&self, cx: &mut Context<Self>) -> AnyElement {
        let p = Palette::new(cx);

        let mut list = div()
            .id("app-menu-list")
            .flex()
            .flex_col()
            .gap_1()
            .w_full()
            .track_scroll(&self.menu_scroll)
            .overflow_y_scroll();

        list = list.child(menu_header(p, "计算器"));
        for (index, page) in menu_pages().iter().enumerate() {
            if index == MODE_COUNT {
                list = list
                    .child(ui::divider(p.border))
                    .child(menu_header(p, "转换器"));
            }
            list = list.child(self.menu_row(cx, p, *page, index));
        }

        let max_height = (self.window_height - 64.0).max(240.0);
        let panel = div()
            .id("app-menu-panel")
            .absolute()
            .top(px(52.))
            .left(px(8.))
            .w(px(286.))
            .max_h(px(max_height))
            .flex()
            .flex_col()
            .p_1()
            .rounded_lg()
            .border_1()
            .border_color(p.border)
            .bg(p.bg)
            .shadow_lg()
            .occlude()
            .child(list);

        div()
            .absolute()
            .inset_0()
            // 遮罩：点击菜单面板以外的地方关闭菜单，并吞掉这次点击。
            .child(div().absolute().inset_0().on_mouse_down(
                MouseButton::Left,
                cx.listener(|app, _: &MouseDownEvent, _window, cx| {
                    app.close_menu();
                    cx.notify();
                }),
            ))
            .child(panel)
            .into_any_element()
    }

    /// 一行菜单项。
    fn menu_row(&self, cx: &mut Context<Self>, p: Palette, page: Page, index: usize) -> AnyElement {
        let selected = self.menu_index == index;
        let active = self.page == page;

        div()
            .id(("app-menu-item", index))
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .h(px(26.))
            .px_2()
            .rounded_sm()
            .when(selected, |this| this.bg(p.accent_soft))
            .hover(move |style| style.bg(p.hover))
            .on_hover(cx.listener(move |app, hovered: &bool, _window, cx| {
                if *hovered && app.menu_index != index {
                    app.menu_index = index;
                    cx.notify();
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |app, _: &MouseDownEvent, _window, cx| {
                    app.set_page(page);
                    app.menu_open = false;
                    cx.notify();
                }),
            )
            .child(
                Icon::new(if active { IconName::Check } else { page.icon() })
                    .small()
                    .text_color(if active { p.accent } else { p.muted }),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(if active { p.accent } else { p.fg })
                    .child(page.title()),
            )
            .into_any_element()
    }
}

/// 菜单里的分组标题。
fn menu_header(p: Palette, label: &'static str) -> AnyElement {
    div()
        .px_2()
        .pt_2()
        .pb_1()
        .text_xs()
        .text_color(p.muted)
        .child(label)
        .into_any_element()
}
