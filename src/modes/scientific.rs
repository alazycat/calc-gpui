//! 「科学」模式：三角函数、对数、幂、阶乘、括号，
//! 支持 DEG / RAD / GRAD 角度制和 F-E 科学计数法显示。
//!
//! 与标准模式一样，这里只保存状态和算法，界面在 [`render`] 里绘制。
//! 求值交给共享的 [`eval`](crate::shared::eval) 核心，这里只描述运算符语义。
//!
//! 自下而上分四层，依赖方向单向（`ops` ← `expr` ← `state` ← `view`）：
//!
//! - [`ops`]：运算符、常量、一元/三角函数/数学函数的语义；
//! - [`expr`]：token 与求值；
//! - [`state`]：状态机，不绘制任何东西，可脱离界面单独测试；
//! - [`view`]：`render` 与全部绘制函数。

mod expr;
mod ops;
mod state;
mod view;

pub use ops::*;
pub use state::*;
pub use view::{keystroke_to_action, render};
