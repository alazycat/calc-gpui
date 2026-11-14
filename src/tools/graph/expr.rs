//! 绘图表达式：词法、语法与求值。
//!
//! 支持 `+ - * / % ^`、括号、隐式乘法（`2x`、`(x+1)(x-1)`）、
//! 常量 `pi`/`e`、变量 `x`，以及 sin/cos/tan、ln/log、sqrt/abs 等函数。
//! 这里只做纯计算，不涉及任何界面。

/// 二元运算符。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

/// 一元函数。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FnKind {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Sqrt,
    Cbrt,
    Abs,
    Ln,
    Log10,
    Exp,
    Floor,
    Ceil,
    Round,
}

impl FnKind {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "sin" => FnKind::Sin,
            "cos" => FnKind::Cos,
            "tan" => FnKind::Tan,
            "asin" => FnKind::Asin,
            "acos" => FnKind::Acos,
            "atan" => FnKind::Atan,
            "sinh" => FnKind::Sinh,
            "cosh" => FnKind::Cosh,
            "tanh" => FnKind::Tanh,
            "sqrt" => FnKind::Sqrt,
            "cbrt" => FnKind::Cbrt,
            "abs" => FnKind::Abs,
            "ln" => FnKind::Ln,
            "log" | "log10" => FnKind::Log10,
            "exp" => FnKind::Exp,
            "floor" => FnKind::Floor,
            "ceil" => FnKind::Ceil,
            "round" => FnKind::Round,
            _ => return None,
        })
    }

    fn apply(self, value: f64) -> f64 {
        match self {
            FnKind::Sin => value.sin(),
            FnKind::Cos => value.cos(),
            FnKind::Tan => value.tan(),
            FnKind::Asin => value.asin(),
            FnKind::Acos => value.acos(),
            FnKind::Atan => value.atan(),
            FnKind::Sinh => value.sinh(),
            FnKind::Cosh => value.cosh(),
            FnKind::Tanh => value.tanh(),
            FnKind::Sqrt => value.sqrt(),
            FnKind::Cbrt => value.cbrt(),
            FnKind::Abs => value.abs(),
            FnKind::Ln => value.ln(),
            FnKind::Log10 => value.log10(),
            FnKind::Exp => value.exp(),
            FnKind::Floor => value.floor(),
            FnKind::Ceil => value.ceil(),
            FnKind::Round => value.round(),
        }
    }
}

/// 表达式语法树。
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f64),
    Var,
    Neg(Box<Expr>),
    Bin(BinKind, Box<Expr>, Box<Expr>),
    Func(FnKind, Box<Expr>),
}

impl Expr {
    /// 在 `x` 处求值；定义域外返回 NaN / 无穷，绘制时会自动断开。
    pub fn eval(&self, x: f64) -> f64 {
        match self {
            Expr::Num(value) => *value,
            Expr::Var => x,
            Expr::Neg(inner) => -inner.eval(x),
            Expr::Bin(kind, left, right) => {
                let left = left.eval(x);
                let right = right.eval(x);
                match kind {
                    BinKind::Add => left + right,
                    BinKind::Sub => left - right,
                    BinKind::Mul => left * right,
                    BinKind::Div => left / right,
                    BinKind::Mod => left % right,
                    BinKind::Pow => left.powf(right),
                }
            }
            Expr::Func(kind, inner) => kind.apply(inner.eval(x)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
}

/// 把文本切成 token。
fn tokenize(input: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;
    let mut tokens = Vec::new();

    while pos < chars.len() {
        let ch = chars[pos];
        match ch {
            ' ' | '\t' | '\n' | '\r' => pos += 1,
            '0'..='9' | '.' => {
                let start = pos;
                let mut seen_dot = false;
                while pos < chars.len() {
                    match chars[pos] {
                        '0'..='9' => pos += 1,
                        '.' if !seen_dot => {
                            seen_dot = true;
                            pos += 1;
                        }
                        _ => break,
                    }
                }
                let text: String = chars[start..pos].iter().collect();
                let value = text
                    .parse::<f64>()
                    .map_err(|_| format!("无法识别的数字：{text}"))?;
                tokens.push(Tok::Num(value));
            }
            'a'..='z' | 'A'..='Z' => {
                let start = pos;
                while pos < chars.len() && chars[pos].is_ascii_alphabetic() {
                    pos += 1;
                }
                let name: String = chars[start..pos].iter().collect();
                tokens.push(Tok::Ident(name.to_ascii_lowercase()));
            }
            'π' => {
                tokens.push(Tok::Ident("pi".to_string()));
                pos += 1;
            }
            '+' => {
                tokens.push(Tok::Plus);
                pos += 1;
            }
            '-' | '−' => {
                tokens.push(Tok::Minus);
                pos += 1;
            }
            '*' | '×' | '·' => {
                tokens.push(Tok::Star);
                pos += 1;
            }
            '/' | '÷' => {
                tokens.push(Tok::Slash);
                pos += 1;
            }
            '%' => {
                tokens.push(Tok::Percent);
                pos += 1;
            }
            '^' => {
                tokens.push(Tok::Caret);
                pos += 1;
            }
            '(' => {
                tokens.push(Tok::LParen);
                pos += 1;
            }
            ')' => {
                tokens.push(Tok::RParen);
                pos += 1;
            }
            other => return Err(format!("不支持的字符：{other}")),
        }
    }

    Ok(tokens)
}

struct Parser {
    tokens: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Tok> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    /// 加减。
    fn expr(&mut self) -> Result<Expr, String> {
        let mut left = self.term()?;
        loop {
            match self.peek() {
                Some(Tok::Plus) => {
                    self.pos += 1;
                    let right = self.term()?;
                    left = Expr::Bin(BinKind::Add, Box::new(left), Box::new(right));
                }
                Some(Tok::Minus) => {
                    self.pos += 1;
                    let right = self.term()?;
                    left = Expr::Bin(BinKind::Sub, Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// 乘除、取余，以及隐式乘法（`2x`、`2(x+1)`、`(x+1)(x-1)`）。
    fn term(&mut self) -> Result<Expr, String> {
        let mut left = self.unary()?;
        loop {
            let implicit = matches!(
                self.peek(),
                Some(Tok::Num(_)) | Some(Tok::Ident(_)) | Some(Tok::LParen)
            );
            if implicit {
                let right = self.unary()?;
                left = Expr::Bin(BinKind::Mul, Box::new(left), Box::new(right));
                continue;
            }
            let kind = match self.peek() {
                Some(Tok::Star) => BinKind::Mul,
                Some(Tok::Slash) => BinKind::Div,
                Some(Tok::Percent) => BinKind::Mod,
                _ => break,
            };
            self.pos += 1;
            let right = self.unary()?;
            left = Expr::Bin(kind, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// 一元正负号。
    fn unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Tok::Minus) => {
                self.pos += 1;
                Ok(Expr::Neg(Box::new(self.unary()?)))
            }
            Some(Tok::Plus) => {
                self.pos += 1;
                self.unary()
            }
            _ => self.power(),
        }
    }

    /// 幂（右结合）。
    fn power(&mut self) -> Result<Expr, String> {
        let base = self.atom()?;
        if matches!(self.peek(), Some(Tok::Caret)) {
            self.pos += 1;
            let exponent = self.unary()?;
            Ok(Expr::Bin(BinKind::Pow, Box::new(base), Box::new(exponent)))
        } else {
            Ok(base)
        }
    }

    fn atom(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Tok::Num(value)) => Ok(Expr::Num(value)),
            Some(Tok::LParen) => {
                let inner = self.expr()?;
                if !matches!(self.peek(), Some(Tok::RParen)) {
                    return Err("缺少右括号".to_string());
                }
                self.pos += 1;
                Ok(inner)
            }
            Some(Tok::Ident(name)) => self.ident(&name),
            _ => Err("表达式中缺少操作数".to_string()),
        }
    }

    fn ident(&mut self, name: &str) -> Result<Expr, String> {
        match name {
            "x" => return Ok(Expr::Var),
            "pi" => return Ok(Expr::Num(std::f64::consts::PI)),
            "e" => return Ok(Expr::Num(std::f64::consts::E)),
            _ => {}
        }

        let Some(kind) = FnKind::from_name(name) else {
            return Err(format!("未知的名称：{name}"));
        };

        // 函数可以写成 sin(x)，也可以写 sin x。
        let arg = if matches!(self.peek(), Some(Tok::LParen)) {
            self.pos += 1;
            let inner = self.expr()?;
            if !matches!(self.peek(), Some(Tok::RParen)) {
                return Err("缺少右括号".to_string());
            }
            self.pos += 1;
            inner
        } else {
            self.unary()?
        };

        Ok(Expr::Func(kind, Box::new(arg)))
    }
}

/// 解析 `y = ...` 右边的表达式。
pub fn parse(input: &str) -> Result<Expr, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("请输入表达式".to_string());
    }
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.expr()?;
    if parser.pos != parser.tokens.len() {
        return Err("表达式末尾有多余的内容".to_string());
    }
    Ok(expr)
}
