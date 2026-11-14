//! 三种计算器模式：标准、科学、程序员。
//!
//! 每个模式都是「状态机 + 渲染」的自成一体的文件，模式之间不互相引用；
//! 共同的部分（[`crate::shared`]）通过 `shared` 访问。

pub mod programmer;
pub mod scientific;
pub mod standard;
