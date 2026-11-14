//! 共享求值器：把 token 序列按运算符优先级求值的泛型核心。
//!
//! 标准 / 科学 / 程序员三个模式原本各写一套表达式求值，逻辑重复且容易各自漂移。
//! 这里把「爬升 + 括号递归 + 错误线程 + 结合性」收成一份实现，模式只需要提供：
//!
//! - [`Language`] 的 `atom`：把叶子 token（数字、常量）转成值；
//! - [`Language`] 的 `missing_operand`：缺操作数时容错还是报错；
//! - [`Operator`] 的 `precedence` / `assoc` / `apply`：运算符语义。
//!
//! 求值器本身不绑定任何模式，也不知道 token 的数字表示是文本、`f64` 还是 `u64`。
//!
//! 绘图模式的表达式（文本 → 可反复求值的语法树）是另一回事，不走这里。

/// 求值失败的类别。三个模式共用同一个错误类型。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CalcError {
    /// 除以零（含取余）。
    DivideByZero,
    /// 操作数缺失或类型不对。
    InvalidInput,
}

impl CalcError {
    /// 显示给用户的文案。
    pub fn message(self) -> &'static str {
        match self {
            CalcError::DivideByZero => "不能被零除",
            CalcError::InvalidInput => "无效输入",
        }
    }
}

/// 运算符结合性。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Assoc {
    /// 左结合：`a − b − c` = `(a − b) − c`。
    Left,
    /// 右结合：`a ^ b ^ c` = `a ^ (b ^ c)`。
    Right,
}

/// 一个运算符：优先级、结合性，以及怎么算。
pub trait Operator<Value>: Copy {
    /// 数值越大结合越紧。
    fn precedence(self) -> u8;

    /// 默认左结合，幂一类的运算符覆盖成 [`Assoc::Right`]。
    fn assoc(self) -> Assoc {
        Assoc::Left
    }

    /// 对两个操作数求值。
    fn apply(self, lhs: Value, rhs: Value) -> Result<Value, CalcError>;
}

/// 一个模式交给求值器的「语言」：怎么认括号、认运算符、认叶子值。
pub trait Language {
    /// 该模式的 token 类型。
    type Token;
    /// 求值结果的值类型（`f64` 或 `u64`）。
    type Value: Copy;
    /// 该模式的运算符类型。
    type Op: Operator<Self::Value>;

    fn is_lparen(&self, token: &Self::Token) -> bool;

    fn is_rparen(&self, token: &Self::Token) -> bool;

    /// 若 token 是运算符，返回它。
    fn operator(&self, token: &Self::Token) -> Option<Self::Op>;

    /// 若 token 是叶子值（数字、常量），返回它的值。
    fn atom(&self, token: &Self::Token) -> Option<Self::Value>;

    /// 需要操作数却拿不到时怎么办：报错，还是按某个容错值处理。
    fn missing_operand(&self) -> Result<Self::Value, CalcError>;
}

/// 按运算符优先级求值整条 token 序列。
///
/// 尾部多余的 token 会被忽略——按钮驱动的状态机保证不会产生它们。
pub fn evaluate<L: Language>(language: L, tokens: &[L::Token]) -> Result<L::Value, CalcError> {
    let mut parser = Parser {
        language,
        tokens,
        pos: 0,
    };
    parser.expr(0)
}

struct Parser<'a, L: Language> {
    language: L,
    tokens: &'a [L::Token],
    pos: usize,
}

impl<L: Language> Parser<'_, L> {
    fn peek(&self) -> Option<&L::Token> {
        self.tokens.get(self.pos)
    }

    /// 当前 token 是运算符时返回它（借出前先拷贝，避免与 `&mut self` 冲突）。
    fn peek_operator(&self) -> Option<L::Op> {
        self.peek().and_then(|token| self.language.operator(token))
    }

    /// 解析一个原子：括号表达式，或一个叶子值。
    fn atom(&mut self) -> Result<L::Value, CalcError> {
        let Some(token) = self.peek() else {
            return self.language.missing_operand();
        };
        if self.language.is_lparen(token) {
            self.pos += 1;
            let value = self.expr(0)?;
            // 右括号缺失时到此为止，交给外层判断。
            if let Some(token) = self.peek()
                && self.language.is_rparen(token)
            {
                self.pos += 1;
            }
            return Ok(value);
        }
        if let Some(value) = self.language.atom(token) {
            self.pos += 1;
            return Ok(value);
        }
        // 不是括号也不是叶子值（例如运算符出现在操作数位置）。
        self.language.missing_operand()
    }

    /// 优先级爬升：`min_prec` 以下的运算符留给外层处理。
    fn expr(&mut self, min_prec: u8) -> Result<L::Value, CalcError> {
        let mut lhs = self.atom()?;
        while let Some(op) = self.peek_operator() {
            let prec = op.precedence();
            if prec < min_prec {
                break;
            }
            self.pos += 1;
            // 左结合时右侧要求更高优先级，右结合时允许同级。
            let next_min = match op.assoc() {
                Assoc::Left => prec.saturating_add(1),
                Assoc::Right => prec,
            };
            let rhs = self.expr(next_min)?;
            lhs = op.apply(lhs, rhs)?;
        }
        Ok(lhs)
    }
}

#[cfg(test)]
mod tests {
    use super::{Assoc, CalcError, Language, Operator, evaluate};

    /// 只用来测核心的最小语言：f64 + 四则 + 幂。
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum TestOp {
        Add,
        Sub,
        Mul,
        Div,
        Pow,
    }

    impl Operator<f64> for TestOp {
        fn precedence(self) -> u8 {
            match self {
                TestOp::Add | TestOp::Sub => 1,
                TestOp::Mul | TestOp::Div => 2,
                TestOp::Pow => 3,
            }
        }

        fn assoc(self) -> Assoc {
            match self {
                TestOp::Pow => Assoc::Right,
                _ => Assoc::Left,
            }
        }

        fn apply(self, lhs: f64, rhs: f64) -> Result<f64, CalcError> {
            match self {
                TestOp::Add => Ok(lhs + rhs),
                TestOp::Sub => Ok(lhs - rhs),
                TestOp::Mul => Ok(lhs * rhs),
                TestOp::Div => {
                    if rhs == 0.0 {
                        Err(CalcError::DivideByZero)
                    } else {
                        Ok(lhs / rhs)
                    }
                }
                TestOp::Pow => Ok(lhs.powf(rhs)),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestToken {
        Num(f64),
        Op(TestOp),
        LParen,
        RParen,
    }

    struct TestLanguage;

    impl Language for TestLanguage {
        type Token = TestToken;
        type Value = f64;
        type Op = TestOp;

        fn is_lparen(&self, token: &TestToken) -> bool {
            matches!(token, TestToken::LParen)
        }

        fn is_rparen(&self, token: &TestToken) -> bool {
            matches!(token, TestToken::RParen)
        }

        fn operator(&self, token: &TestToken) -> Option<TestOp> {
            match token {
                TestToken::Op(op) => Some(*op),
                _ => None,
            }
        }

        fn atom(&self, token: &TestToken) -> Option<f64> {
            match token {
                TestToken::Num(value) => Some(*value),
                _ => None,
            }
        }

        fn missing_operand(&self) -> Result<f64, CalcError> {
            Err(CalcError::InvalidInput)
        }
    }

    fn num(value: f64) -> TestToken {
        TestToken::Num(value)
    }

    fn op(op: TestOp) -> TestToken {
        TestToken::Op(op)
    }

    fn eval(tokens: &[TestToken]) -> Result<f64, CalcError> {
        evaluate(TestLanguage, tokens)
    }

    #[test]
    fn multiplication_binds_tighter_than_addition() {
        let tokens = [
            num(2.0),
            op(TestOp::Add),
            num(3.0),
            op(TestOp::Mul),
            num(4.0),
        ];
        assert_eq!(eval(&tokens), Ok(14.0));
    }

    #[test]
    fn equal_precedence_is_left_associative() {
        // 10 − 3 + 2 = 9
        let tokens = [
            num(10.0),
            op(TestOp::Sub),
            num(3.0),
            op(TestOp::Add),
            num(2.0),
        ];
        assert_eq!(eval(&tokens), Ok(9.0));
        // 8 ÷ 4 × 2 = 4（不是 1）
        let tokens = [
            num(8.0),
            op(TestOp::Div),
            num(4.0),
            op(TestOp::Mul),
            num(2.0),
        ];
        assert_eq!(eval(&tokens), Ok(4.0));
    }

    #[test]
    fn right_associative_operator_groups_to_the_right() {
        // 2 ^ 3 ^ 2 = 2 ^ (3 ^ 2) = 512
        let tokens = [
            num(2.0),
            op(TestOp::Pow),
            num(3.0),
            op(TestOp::Pow),
            num(2.0),
        ];
        assert_eq!(eval(&tokens), Ok(512.0));
    }

    #[test]
    fn parentheses_override_precedence() {
        // (2 + 3) × 4 = 20
        let tokens = [
            TestToken::LParen,
            num(2.0),
            op(TestOp::Add),
            num(3.0),
            TestToken::RParen,
            op(TestOp::Mul),
            num(4.0),
        ];
        assert_eq!(eval(&tokens), Ok(20.0));
    }

    #[test]
    fn nested_parentheses() {
        // ((1 + 2) × (3 + 4)) = 21
        let tokens = [
            TestToken::LParen,
            TestToken::LParen,
            num(1.0),
            op(TestOp::Add),
            num(2.0),
            TestToken::RParen,
            op(TestOp::Mul),
            TestToken::LParen,
            num(3.0),
            op(TestOp::Add),
            num(4.0),
            TestToken::RParen,
            TestToken::RParen,
        ];
        assert_eq!(eval(&tokens), Ok(21.0));
    }

    #[test]
    fn operator_errors_thread_through() {
        let tokens = [
            num(1.0),
            op(TestOp::Add),
            num(2.0),
            op(TestOp::Div),
            num(0.0),
        ];
        assert_eq!(eval(&tokens), Err(CalcError::DivideByZero));
    }

    #[test]
    fn missing_operand_uses_language_policy() {
        // 末尾悬空的运算符：TestLanguage 选择报错。
        let tokens = [num(1.0), op(TestOp::Add)];
        assert_eq!(eval(&tokens), Err(CalcError::InvalidInput));
        // 空表达式同样走容错策略。
        assert_eq!(eval(&[]), Err(CalcError::InvalidInput));
    }

    #[test]
    fn single_number_evaluates_to_itself() {
        assert_eq!(eval(&[num(42.0)]), Ok(42.0));
    }

    #[test]
    fn error_messages_are_human_readable() {
        assert_eq!(CalcError::DivideByZero.message(), "不能被零除");
        assert_eq!(CalcError::InvalidInput.message(), "无效输入");
    }
}
