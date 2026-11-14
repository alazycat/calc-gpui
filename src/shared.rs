//! 跨页面共享的内核：求值器、记忆槽、数值格式化、持久化、配色与按键组件。
//!
//! 这里的模块都不能引用 [`crate::modes`] 或 [`crate::tools`] —— 依赖方向
//! 是单向的：模式/工具 → 共享内核，反过来不成立。

pub mod eval;
pub mod memory;
pub mod number;
pub mod storage;
pub mod ui;
