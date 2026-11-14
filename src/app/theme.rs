//! 主题：浅色 / 深色 / 跟随系统的偏好，以及 Windows 原生标题栏配色。

use super::CalcApp;
use crate::shared::ui;
use gpui_kit::assets::IconName;
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::*;

/// 主题偏好。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ThemePref {
    #[default]
    Light,
    Dark,
    System,
}

impl ThemePref {
    pub fn from_saved(value: Option<&str>) -> Self {
        match value {
            Some("dark") => ThemePref::Dark,
            Some("system") => ThemePref::System,
            _ => ThemePref::Light,
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            ThemePref::Light => "light",
            ThemePref::Dark => "dark",
            ThemePref::System => "system",
        }
    }

    pub(super) fn next(self) -> Self {
        match self {
            ThemePref::Light => ThemePref::Dark,
            ThemePref::Dark => ThemePref::System,
            ThemePref::System => ThemePref::Light,
        }
    }

    pub(super) fn icon(self) -> IconName {
        match self {
            ThemePref::Light => IconName::Sun,
            ThemePref::Dark => IconName::Moon,
            ThemePref::System => IconName::SunMoon,
        }
    }
}

impl CalcApp {
    /// 浅色 → 深色 → 跟随系统 → 浅色。
    pub(super) fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
    }

    /// 按当前偏好同步主题；切换主题后重新写回品牌蓝色。
    pub(super) fn apply_theme(&mut self, window: &mut Window, cx: &mut App) {
        let wanted = match self.theme {
            ThemePref::Light => ThemeMode::Light,
            ThemePref::Dark => ThemeMode::Dark,
            ThemePref::System => ThemeMode::from(window.appearance()),
        };
        if Theme::global(cx).mode != wanted {
            Theme::change(wanted, Some(window), cx);
            ui::apply_brand_colors(cx);
        }
    }
}

/// 让 Windows 原生标题栏跟随浅色 / 深色主题。
///
/// 使用 DWM 的 `DWMWA_USE_IMMERSIVE_DARK_MODE`（旧系统是属性 19），
/// 非 Windows 平台是空实现。
pub(super) fn apply_native_titlebar_theme(window: &Window, dark: bool) {
    #[cfg(target_os = "windows")]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};

        let Ok(handle) = HasWindowHandle::window_handle(window) else {
            return;
        };
        let RawWindowHandle::Win32(win32) = handle.as_raw() else {
            return;
        };
        let hwnd = win32.hwnd.get() as *mut core::ffi::c_void;
        let value: i32 = if dark { 1 } else { 0 };
        unsafe {
            // 20：Windows 10 2004+；19：更早的版本。
            let _ = DwmSetWindowAttribute(hwnd, 20, &value as *const i32 as *const _, 4);
            let _ = DwmSetWindowAttribute(hwnd, 19, &value as *const i32 as *const _, 4);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (window, dark);
    }
}

#[cfg(target_os = "windows")]
#[link(name = "dwmapi")]
unsafe extern "system" {
    fn DwmSetWindowAttribute(
        hwnd: *mut core::ffi::c_void,
        attribute: u32,
        value: *const core::ffi::c_void,
        size: u32,
    ) -> i32;
}
