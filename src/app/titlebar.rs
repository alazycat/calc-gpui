//! 根视图装配：每帧同步主题 / 窗口标题 / 窗口尺寸，再画标题栏与页面主体。

use super::CalcApp;
use super::theme::apply_native_titlebar_theme;
use crate::shared::ui::{self, Palette};
use gpui_kit::assets::IconName;
use gpui_kit::component::{StyledExt, Theme};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl Render for CalcApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 按偏好同步主题（含「跟随系统」）。
        self.apply_theme(window, cx);
        let bg = Palette::new(cx).bg;

        // 原生标题栏也跟随浅色 / 深色（启动时也要同步一次）。
        let dark = Theme::global(cx).mode.is_dark();
        if self.native_titlebar_dark != Some(dark) {
            apply_native_titlebar_theme(window, dark);
            self.native_titlebar_dark = Some(dark);
        }

        // 窗口标题跟随当前模式。
        if self.window_title != Some(self.page) {
            window.set_window_title(&format!("计算器 · {}", self.title()));
            self.window_title = Some(self.page);
        }

        // 记住窗口尺寸：只在非最大化状态下保存。
        if let WindowBounds::Windowed(bounds) = window.window_bounds() {
            self.window_width = bounds.size.width.as_f32();
            self.window_height = bounds.size.height.as_f32();
        }

        // 惰性创建焦点句柄，并在首次渲染时把焦点交给根视图。
        let focus_handle = self
            .focus_handle
            .get_or_insert_with(|| cx.focus_handle())
            .clone();
        if self.needs_focus {
            focus_handle.focus(window, cx);
            self.needs_focus = false;
        }

        let root = div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(bg)
            .p_3()
            .gap_2()
            // 让根视图能获得焦点，并接收键盘输入。
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(|app, event: &KeyDownEvent, _window, cx| {
                app.handle_key(&event.keystroke, cx);
                // GPUI 不会因为状态改变自动重绘，必须显式通知。
                cx.notify();
            }))
            .child(self.title_bar(cx))
            .child(self.body(cx))
            .when(self.menu_open, |this| this.child(self.render_menu(cx)));

        // 有变化时（历史、记忆、主题、窗口尺寸）写入磁盘。
        self.persist();
        root
    }
}

impl CalcApp {
    /// 顶栏：汉堡菜单 · 模式名称 · 历史记录 / 主题按钮。
    fn title_bar(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let p = Palette::new(cx);

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .h(px(44.))
            .flex_none()
            .child(ui::icon_key(
                cx,
                "main-menu",
                p,
                IconName::Menu,
                self.menu_open,
                |app| app.toggle_menu(),
            ))
            .child(
                div()
                    .px_1()
                    .text_xl()
                    .line_height(relative(1.1))
                    .font_semibold()
                    .text_color(p.fg)
                    .child(self.title()),
            )
            .child(div().flex_1())
            // 只有带历史面板的页面才显示这个按钮；展开状态由页面自己回答。
            .when_some(self.page_view().history_open(), |this, open| {
                this.child(ui::icon_key(
                    cx,
                    "title-history",
                    p,
                    IconName::Clock,
                    open,
                    |app| app.page_view().toggle_history(),
                ))
            })
            .child(ui::icon_key(
                cx,
                "theme-toggle",
                p,
                self.theme.icon(),
                false,
                |app| app.cycle_theme(),
            ))
            .into_any_element()
    }

    /// 页面主体：交给当前页面的 [`PageView`](super::dispatch::PageView) 实现。
    fn body(&mut self, cx: &mut Context<Self>) -> AnyElement {
        self.page_view().render(cx)
    }
}
