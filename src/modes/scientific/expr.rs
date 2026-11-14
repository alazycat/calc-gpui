//! 科学模式的表达式层：token 类型与求值。
//!
//! 把一串 token 交给共享求值器算成一个数；运算符与函数的具体语义在
//! [`super::ops`] 里。

use super::ops::{Constant, SOp};
use crate::shared::eval::{self, CalcError, Language};
use crate::shared::number::format_value;

/// 算式里的一个片段。
#[derive(Clone, Debug, PartialEq)]
pub enum SToken {
    Num(f64),
    Const(Constant),
    Op(SOp),
    LParen,
    RParen,
}

pub fn paren_depth(tokens: &[SToken]) -> usize {
    let mut depth = 0usize;
    for token in tokens {
        match token {
            SToken::LParen => depth += 1,
            SToken::RParen => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    depth
}

/// 把 token 序列拼成算式。
pub fn tokens_to_string(tokens: &[SToken], scientific: bool) -> String {
    tokens
        .iter()
        .map(|token| match token {
            SToken::Num(value) => format_value(*value, scientific),
            SToken::Const(constant) => constant.symbol().to_string(),
            SToken::Op(op) => op.symbol().to_string(),
            SToken::LParen => "(".to_string(),
            SToken::RParen => ")".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 科学模式的语言：数字、常量 π/e、括号。
struct ScientificLanguage;

impl Language for ScientificLanguage {
    type Token = SToken;
    type Value = f64;
    type Op = SOp;

    fn is_lparen(&self, token: &SToken) -> bool {
        matches!(token, SToken::LParen)
    }

    fn is_rparen(&self, token: &SToken) -> bool {
        matches!(token, SToken::RParen)
    }

    fn operator(&self, token: &SToken) -> Option<SOp> {
        match token {
            SToken::Op(op) => Some(*op),
            _ => None,
        }
    }

    fn atom(&self, token: &SToken) -> Option<f64> {
        match token {
            SToken::Num(value) => Some(*value),
            SToken::Const(constant) => Some(constant.value()),
            _ => None,
        }
    }

    /// 科学模式缺操作数时明确报错。
    fn missing_operand(&self) -> Result<f64, CalcError> {
        Err(CalcError::InvalidInput)
    }
}

/// 求值一条科学模式算式。
pub fn evaluate(tokens: &[SToken]) -> Result<f64, CalcError> {
    eval::evaluate(ScientificLanguage, tokens)
}
