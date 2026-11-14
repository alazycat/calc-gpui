//! 科学模式的运算符与函数语义：角度制、常量、二元运算符、一元函数、
//! 三角函数与数学函数。
//!
//! 这里只回答「怎么算」，不保存任何状态；求值流程在 [`super::expr`]。

use crate::shared::eval::{Assoc, CalcError, Operator};

/// 三角函数使用的角度单位。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AngleMode {
    Deg,
    Rad,
    Grad,
}

impl AngleMode {
    pub fn label(self) -> &'static str {
        match self {
            AngleMode::Deg => "DEG",
            AngleMode::Rad => "RAD",
            AngleMode::Grad => "GRAD",
        }
    }

    /// DEG / GRAD → 弧度。
    fn to_radians(self, value: f64) -> f64 {
        use std::f64::consts::PI;
        match self {
            AngleMode::Deg => value * PI / 180.0,
            AngleMode::Rad => value,
            AngleMode::Grad => value * PI / 200.0,
        }
    }

    /// 弧度 → 当前角度单位。
    fn to_unit(self, value: f64) -> f64 {
        use std::f64::consts::PI;
        match self {
            AngleMode::Deg => value * 180.0 / PI,
            AngleMode::Rad => value,
            AngleMode::Grad => value * 200.0 / PI,
        }
    }

    pub fn next(self) -> Self {
        match self {
            AngleMode::Deg => AngleMode::Rad,
            AngleMode::Rad => AngleMode::Grad,
            AngleMode::Grad => AngleMode::Deg,
        }
    }
}

/// 数学常量。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Constant {
    Pi,
    E,
}

impl Constant {
    pub fn value(self) -> f64 {
        match self {
            Constant::Pi => std::f64::consts::PI,
            Constant::E => std::f64::consts::E,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Constant::Pi => "π",
            Constant::E => "e",
        }
    }
}

/// 二元运算符。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    /// `xʸ` 在 2nd 状态下的「y 次方根」。
    Root,
    /// `exp`：以 10 为底的指数输入，`2 exp 3` = 2000。
    Exp,
    /// 2nd 状态下的 `logᵧ`：`8 logᵧ 2` = 3。
    LogBase,
}

impl SOp {
    pub fn symbol(self) -> &'static str {
        match self {
            SOp::Add => "+",
            SOp::Sub => "−",
            SOp::Mul => "×",
            SOp::Div => "÷",
            SOp::Mod => "mod",
            SOp::Pow => "^",
            SOp::Root => "√",
            SOp::Exp => "exp",
            SOp::LogBase => "logᵧ",
        }
    }
}

/// 把科学运算符交给共享求值器：幂家族右结合，其余左结合。
impl Operator<f64> for SOp {
    fn precedence(self) -> u8 {
        match self {
            SOp::Pow | SOp::Root | SOp::LogBase => 4,
            SOp::Mul | SOp::Div | SOp::Mod | SOp::Exp => 3,
            SOp::Add | SOp::Sub => 2,
        }
    }

    fn assoc(self) -> Assoc {
        match self {
            SOp::Pow | SOp::Root | SOp::LogBase => Assoc::Right,
            _ => Assoc::Left,
        }
    }

    fn apply(self, a: f64, b: f64) -> Result<f64, CalcError> {
        match self {
            SOp::Add => Ok(a + b),
            SOp::Sub => Ok(a - b),
            SOp::Mul => Ok(a * b),
            SOp::Div => {
                if b == 0.0 {
                    Err(CalcError::DivideByZero)
                } else {
                    Ok(a / b)
                }
            }
            SOp::Mod => {
                if b == 0.0 {
                    Err(CalcError::DivideByZero)
                } else {
                    Ok(a % b)
                }
            }
            SOp::Pow => Ok(a.powf(b)),
            SOp::Root => {
                if b == 0.0 {
                    Err(CalcError::InvalidInput)
                } else {
                    Ok(a.powf(1.0 / b))
                }
            }
            SOp::Exp => Ok(a * 10f64.powf(b)),
            SOp::LogBase => {
                if a <= 0.0 || b <= 0.0 || b == 1.0 {
                    Err(CalcError::InvalidInput)
                } else {
                    Ok(a.ln() / b.ln())
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 一元函数
// ---------------------------------------------------------------------------

/// 即时作用在显示值上的一元函数。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unary {
    Square,
    Cube,
    Reciprocal,
    Abs,
    Sqrt,
    Cbrt,
    Factorial,
    TenPow,
    TwoPow,
    Log10,
    Ln,
    Exp,
}

impl Unary {
    pub fn apply(self, value: f64) -> f64 {
        match self {
            Unary::Square => value * value,
            Unary::Cube => value * value * value,
            Unary::Reciprocal => 1.0 / value,
            Unary::Abs => value.abs(),
            Unary::Sqrt => value.sqrt(),
            Unary::Cbrt => value.cbrt(),
            Unary::Factorial => factorial(value),
            Unary::TenPow => 10f64.powf(value),
            Unary::TwoPow => 2f64.powf(value),
            Unary::Log10 => value.log10(),
            Unary::Ln => value.ln(),
            Unary::Exp => value.exp(),
        }
    }
}

/// 阶乘：只接受 0..=170 的整数。
fn factorial(value: f64) -> f64 {
    if value < 0.0 || value > 170.0 || value.fract() != 0.0 {
        return f64::NAN;
    }
    (1..=value as u64).map(|i| i as f64).product()
}

/// 「三角函数」下拉里的函数。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trig {
    Sin,
    Cos,
    Tan,
    Sec,
    Csc,
    Cot,
    Asin,
    Acos,
    Atan,
    Asec,
    Acsc,
    Acot,
}

impl Trig {
    pub fn label(self) -> &'static str {
        match self {
            Trig::Sin => "sin",
            Trig::Cos => "cos",
            Trig::Tan => "tan",
            Trig::Sec => "sec",
            Trig::Csc => "csc",
            Trig::Cot => "cot",
            Trig::Asin => "sin⁻¹",
            Trig::Acos => "cos⁻¹",
            Trig::Atan => "tan⁻¹",
            Trig::Asec => "sec⁻¹",
            Trig::Acsc => "csc⁻¹",
            Trig::Acot => "cot⁻¹",
        }
    }

    pub fn apply(self, value: f64, angle: AngleMode) -> f64 {
        match self {
            Trig::Sin => angle.to_radians(value).sin(),
            Trig::Cos => angle.to_radians(value).cos(),
            Trig::Tan => angle.to_radians(value).tan(),
            Trig::Sec => 1.0 / angle.to_radians(value).cos(),
            Trig::Csc => 1.0 / angle.to_radians(value).sin(),
            Trig::Cot => 1.0 / angle.to_radians(value).tan(),
            Trig::Asin => angle.to_unit(value.asin()),
            Trig::Acos => angle.to_unit(value.acos()),
            Trig::Atan => angle.to_unit(value.atan()),
            Trig::Asec => angle.to_unit((1.0 / value).acos()),
            Trig::Acsc => angle.to_unit((1.0 / value).asin()),
            Trig::Acot => angle.to_unit((1.0 / value).atan()),
        }
    }
}

/// 「函数」下拉里的函数。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MathFn {
    Sinh,
    Cosh,
    Tanh,
    Sech,
    Csch,
    Coth,
    Asinh,
    Acosh,
    Atanh,
    Asech,
    Acsch,
    Acoth,
    Floor,
    Ceil,
    Round,
}

impl MathFn {
    pub fn label(self) -> &'static str {
        match self {
            MathFn::Sinh => "sinh",
            MathFn::Cosh => "cosh",
            MathFn::Tanh => "tanh",
            MathFn::Sech => "sech",
            MathFn::Csch => "csch",
            MathFn::Coth => "coth",
            MathFn::Asinh => "sinh⁻¹",
            MathFn::Acosh => "cosh⁻¹",
            MathFn::Atanh => "tanh⁻¹",
            MathFn::Asech => "sech⁻¹",
            MathFn::Acsch => "csch⁻¹",
            MathFn::Acoth => "coth⁻¹",
            MathFn::Floor => "⌊x⌋",
            MathFn::Ceil => "⌈x⌉",
            MathFn::Round => "round",
        }
    }

    pub fn apply(self, value: f64) -> f64 {
        match self {
            MathFn::Sinh => value.sinh(),
            MathFn::Cosh => value.cosh(),
            MathFn::Tanh => value.tanh(),
            MathFn::Sech => 1.0 / value.cosh(),
            MathFn::Csch => 1.0 / value.sinh(),
            MathFn::Coth => 1.0 / value.tanh(),
            MathFn::Asinh => value.asinh(),
            MathFn::Acosh => value.acosh(),
            MathFn::Atanh => value.atanh(),
            MathFn::Asech => (1.0 / value).acosh(),
            MathFn::Acsch => (1.0 / value).asinh(),
            MathFn::Acoth => (1.0 / value).atanh(),
            MathFn::Floor => value.floor(),
            MathFn::Ceil => value.ceil(),
            MathFn::Round => value.round(),
        }
    }
}
