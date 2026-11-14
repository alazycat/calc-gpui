//! 「标准」模式：四则运算 + 记忆 + 历史记录。
//!
//! 这一部分只保存状态与算法，界面在 [`render`] 里绘制；
//! 顶层视图 [`CalcApp`](crate::app::CalcApp) 负责标题栏、菜单和键盘事件分发。
//!
//! - [`state`]：状态机与求值，不绘制任何东西，可脱离界面单独测试；
//! - [`view`]：`render` 与全部绘制函数，只读状态、通过 `cx` 回写事件。

mod state;
mod view;

pub use state::*;
pub use view::render;
