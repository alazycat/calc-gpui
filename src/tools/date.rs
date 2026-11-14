//! 日期计算：两个日期之间的差、以及日期加减。
//!
//! 不引入额外依赖，公历换算用 Howard Hinnant 的 `days_from_civil` /
//! `civil_from_days` 算法；「今天」按 UTC 计算。
//!
//! - [`state`]：公历换算、日期差与状态机，不绘制任何东西；
//! - [`view`]：`render` 与全部绘制函数。

mod state;
mod view;

pub use state::*;
pub use view::render;
