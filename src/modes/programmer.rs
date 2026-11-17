//! 「程序员」模式：多进制显示、位宽切换、位运算与移位、位翻转面板。
//!
//! 数值统一以 `u64` 的**位模式**保存，具体含义由当前进制和位宽决定：
//! - HEX / OCT / BIN 按无符号位模式显示；
//! - DEC 按当前位宽的补码有符号数显示。
//!
//! - [`state`]：状态机与求值，不绘制任何东西，可脱离界面单独测试；
//! - [`view`]：`render` 与全部绘制函数，只读状态、通过 `cx` 回写事件。

mod state;
mod view;

pub use state::*;
pub use view::{keystroke_to_action, render};
