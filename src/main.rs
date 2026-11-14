//! 一个使用 Rust + GPUI Kit 编写的桌面计算器。
//!
//! 目前包含：
//! - **标准**：四则运算、百分比、一元运算、记忆与历史记录；
//! - **科学**：三角函数、对数、幂/根、阶乘、括号，DEG/RAD/GRAD 与 F-E；
//! - **程序员**：HEX / DEC / OCT / BIN 四种进制、QWORD/DWORD/WORD/BYTE 位宽、
//!   位运算（AND/OR/NOT/XOR/NAND/NOR）、移位（`<<` / `>>`）、位翻转面板；
//! - **换算器**：货币、体积、长度、重量、温度、能量、面积、速度、时间、功率、
//!   数据、压力、角度 13 类单位；
//! - **日期计算**：两个日期的差、日期加减年/月/周/天；
//! - **绘图**：输入 y = f(x)，绘制函数曲线，支持缩放、平移、自动适配。
//!
//! 顶栏的汉堡菜单可以在模式和换算器之间切换：F10 打开菜单，方向键选择，
//! Enter 确认，Esc 关闭。除了鼠标点击，也支持直接敲键盘
//! （Ctrl+C 复制结果、Ctrl+V 粘贴数值、Ctrl+H 历史）。
//!
//! 右上角的主题按钮在浅色 / 深色 / 跟随系统之间切换，原生标题栏也会跟着变深；
//! 历史记录、记忆槽、主题、上次页面和窗口尺寸会保存到用户配置目录
//! （`calc-gpui/state.json`），下次启动恢复。窗口另有 360×520 的最小尺寸限制。
//!
//! 代码结构（依赖方向单向：`app` → `modes`/`tools` → `shared`）：
//! - [`main`]：初始化主题、打开窗口；
//! - [`app`]：根视图，标题栏 / 菜单 / 页面切换 / 键盘分发；
//! - [`modes`]：三个计算器模式
//!   - [`standard`](modes::standard)：标准模式的状态与界面；
//!   - [`scientific`](modes::scientific)：科学模式的状态与界面；
//!   - [`programmer`](modes::programmer)：程序员模式的状态与界面；
//! - [`tools`]：三个独立工具页面
//!   - [`converter`](tools::converter)：单位换算（含离线的示例汇率）；
//!   - [`date`](tools::date)：日期计算；
//!   - [`graph`](tools::graph)：函数绘图与表达式解析；
//! - [`shared`]：跨页面共享的内核
//!   - [`eval`](shared::eval)：三个模式共享的表达式求值器（优先级爬升）；
//!   - [`memory`](shared::memory)：三个模式共享的记忆槽；
//!   - [`number`](shared::number)：共享的数值格式化；
//!   - [`storage`](shared::storage)：本地持久化；
//!   - [`ui`](shared::ui)：共享的配色与按键组件。

mod app;
mod modes;
mod shared;
mod tools;

use app::{CalcApp, ThemePref};
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
