//! 桌面应用入口：初始化主题与窗口，然后把交互交给 [`calc_gpui::app::CalcApp`]。
//!
//! 模块结构与功能说明见库文档 [`calc_gpui`]。

use calc_gpui::app::{CalcApp, ThemePref};
use calc_gpui::shared;
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::*;

fn main() {
    application().with_assets(assets::AllAssets).run(|cx| {
        init(cx);

        // 恢复上次的主题偏好和窗口尺寸。
        let saved = shared::storage::load();
        match ThemePref::from_saved(saved.theme.as_deref()) {
            ThemePref::Light => Theme::change(ThemeMode::Light, None, cx),
            ThemePref::Dark => Theme::change(ThemeMode::Dark, None, cx),
            // 跟随系统：等窗口打开后由 CalcApp 读取系统外观再同步。
            ThemePref::System => {}
        }
        shared::ui::apply_brand_colors(cx);

        let (width, height) = saved.window_size();
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(width), px(height)), cx)),
            window_min_size: Some(size(px(360.), px(520.))),
            titlebar: Some(TitlebarOptions {
                title: Some("计算器".into()),
                ..Default::default()
            }),
            ..WindowOptions::default()
        };

        open_window(options, cx, |_, cx| cx.new(|_| CalcApp::new())).expect("无法打开窗口");
    });
}
