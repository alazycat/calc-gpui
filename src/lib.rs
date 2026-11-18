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
//! - **绘图**：多条表达式同时绘制（`y = f(x)` 与 `y > f(x)` 这类不等式），
//!   支持函数 / 不等式 / 三角学面板、`2ⁿᵈ`、缩放平移、自动适配，
//!   并可在「坐标系」和「表达式列表」两种视图之间切换。
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
//! - 二进制入口：初始化主题、打开窗口；
//! - [`app`]：根视图，标题栏 / 菜单 / 页面切换 / 键盘分发；
//! - [`modes`]：三个计算器模式
//!   - [`standard`](modes::standard)：标准模式的状态与界面；
//!   - [`scientific`](modes::scientific)：科学模式的状态与界面；
//!   - [`programmer`](modes::programmer)：程序员模式的状态与界面；
//! - [`tools`]：三个独立工具页面
//!   - [`converter`](tools::converter)：单位换算（含离线的示例汇率）；
//!   - [`date`](tools::date)：日期计算；
//!   - [`graph`](tools::graph)：多条函数 / 不等式的绘图与表达式解析；
//! - [`shared`]：跨页面共享的内核
//!   - [`eval`](shared::eval)：三个模式共享的表达式求值器（优先级爬升）；
//!   - [`history`](shared::history)：标准与科学模式共享的历史条目与容量上限；
//!   - [`memory`](shared::memory)：三个模式共享的记忆槽；
//!   - [`number`](shared::number)：共享的数值格式化；
//!   - [`storage`](shared::storage)：本地持久化；
//!   - [`ui`](shared::ui)：共享的配色与按键组件。
//!
//! 所有页面状态机（standard / scientific / programmer / converter / graph）都不认识
//! `Keystroke`：状态层只暴露语义方法与纯数据的 `Action` 枚举（`apply_action` 分发），
//! 按键 → `Action` 的翻译集中在各自 view 层的 `keystroke_to_action` 适配器里，
//! [`app::dispatch`] 负责把两者串起来。这使状态机可以脱离 UI 框架单测。

pub mod app;
pub mod modes;
pub mod shared;
pub mod tools;
