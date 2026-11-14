//! 菜单里的「工具」页面：单位换算、日期计算、函数绘图。
//!
//! 与计算器模式不同，这些页面不做四则运算状态机，各自解决一个独立问题，
//! 因此和 [`crate::modes`] 分开放。

pub mod converter;
pub mod date;
pub mod graph;
