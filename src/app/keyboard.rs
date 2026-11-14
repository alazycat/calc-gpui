//! 键盘分发：先处理菜单与 Ctrl 快捷键，其余交给当前页面自己处理。

use super::CalcApp;
use gpui_kit::*;

impl CalcApp {
    /// 处理一次按键：先处理菜单和 Ctrl 快捷键，再分发给当前模式。
    pub(super) fn handle_key(&mut self, keystroke: &Keystroke, cx: &mut Context<Self>) {
        // F10 打开 / 关闭菜单；菜单打开时用方向键选择、Enter 确认、Esc 关闭。
        if keystroke.key == "f10" {
            self.toggle_menu();
            return;
        }
        if self.menu_open {
            match keystroke.key.as_str() {
                "up" => self.move_menu(-1),
                "down" => self.move_menu(1),
                "enter" | "space" => self.activate_menu(),
                "escape" => self.close_menu(),
                _ => {}
            }
            return;
        }

        let modifiers = &keystroke.modifiers;
        if modifiers.control {
            match keystroke.key.as_str() {
                "c" => {
                    cx.write_to_clipboard(ClipboardItem::new_string(self.copy_text()));
                }
                "v" => {
                    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                        self.paste_text(&text);
                    }
                }
                "h" => self.page_view().toggle_history(),
                _ => {}
            }
            return;
        }
        if modifiers.alt || modifiers.platform {
            return;
        }

        self.page_view().handle_key(keystroke);
    }

    /// 当前页面的结果文本（Ctrl+C）。
    fn copy_text(&mut self) -> String {
        self.page_view().copy_text()
    }

    /// 从剪贴板粘贴数值（Ctrl+V）。
    fn paste_text(&mut self, text: &str) {
        self.page_view().paste(text);
    }
}
