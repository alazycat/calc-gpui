//! 标准模式的状态机：token、运算符语义、求值，以及历史与记忆。

use crate::shared::eval::{self, Assoc, CalcError, Language, Operator};
use crate::shared::memory::Memory;
use crate::shared::number::format_plain;
use gpui_kit::Keystroke;

/// 四则运算的运算符。
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    /// 运算符在按钮 / 算式里显示的符号。
    fn symbol(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "−",
            Op::Mul => "×",
            Op::Div => "÷",
        }
    }
}

/// 把四则运算符交给共享求值器：乘除高于加减，全部左结合。
impl Operator<f64> for Op {
    fn precedence(self) -> u8 {
        match self {
            Op::Add | Op::Sub => 1,
            Op::Mul | Op::Div => 2,
        }
    }

    fn assoc(self) -> Assoc {
        Assoc::Left
    }

    fn apply(self, a: f64, b: f64) -> Result<f64, CalcError> {
        match self {
            Op::Add => Ok(a + b),
            Op::Sub => Ok(a - b),
            Op::Mul => Ok(a * b),
            Op::Div => {
                if b == 0.0 {
                    Err(CalcError::DivideByZero)
                } else {
                    Ok(a / b)
                }
            }
        }
    }
}

/// 算式里的一个片段：数字或运算符。
///
/// 保存用户输入的**原始文本**而不是 f64，这样显示算式时可以原样还原
/// （例如 `0.50` 不会被格式化成 `0.5`）。
#[derive(Clone)]
pub enum Token {
    Num(String),
    Op(Op),
}

impl Token {
    fn text(&self) -> &str {
        match self {
            Token::Num(s) => s,
            Token::Op(op) => op.symbol(),
        }
    }
}

/// 把 token 序列拼成给人看的算式，例如 `2 + 3 ×`。
fn tokens_to_string(tokens: &[Token]) -> String {
    tokens.iter().map(Token::text).collect::<Vec<_>>().join(" ")
}

/// 按「先乘除后加减」对算式求值。标准模式没有括号，括号判定恒为 false。
struct StandardLanguage;

impl Language for StandardLanguage {
    type Token = Token;
    type Value = f64;
    type Op = Op;

    fn is_lparen(&self, _token: &Token) -> bool {
        false
    }

    fn is_rparen(&self, _token: &Token) -> bool {
        false
    }

    fn operator(&self, token: &Token) -> Option<Op> {
        match token {
            Token::Op(op) => Some(*op),
            _ => None,
        }
    }

    fn atom(&self, token: &Token) -> Option<f64> {
        match token {
            Token::Num(text) => text.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// 按键序列保证不会缺操作数；万一出现就按 0 处理。
    fn missing_operand(&self) -> Result<f64, CalcError> {
        Ok(0.0)
    }
}

/// 求值一条标准模式算式。
pub fn evaluate(tokens: &[Token]) -> Result<f64, CalcError> {
    eval::evaluate(StandardLanguage, tokens)
}

/// 一条计算历史。
#[derive(Clone)]
pub struct HistoryEntry {
    /// 带等号的算式，例如 `2 + 3 × 4 =`。
    pub expression: String,
    /// 计算结果。
    pub result: String,
}

/// 「标准」模式的状态。
pub struct StandardCalc {
    /// 主显示屏上的数字（当前正在输入 / 上一次结果）。
    pub display: String,
    /// 显示屏上方的小号算式，例如 `2 + 3 ×`。
    pub expression: String,
    /// 已确认的算式片段，例如 `[2, +, 3, ×]`。正在输入的数字不在里面。
    tokens: Vec<Token>,
    /// 为 true 表示：刚按下运算符或 `=`，下一次输入数字将开启一个新操作数。
    waiting_for_operand: bool,
    /// 连续按 = 时复用的运算符。
    last_operator: Option<Op>,
    /// 连续按 = 时复用的右操作数（例如 12 + 7 = = =）。
    last_operand: Option<f64>,
    /// 记忆槽（MS / MR / MC / M+ / M−），面板开关也在这里。
    pub memory: Memory<f64>,
    /// 当前错误（例如除以零）。
    error: Option<CalcError>,
    /// 计算历史（最新一条在末尾）。
    pub history: Vec<HistoryEntry>,
    /// 是否正在显示历史记录面板。
    pub show_history: bool,
}

impl StandardCalc {
    pub fn new() -> Self {
        Self {
            display: "0".to_string(),
            expression: String::new(),
            tokens: Vec::new(),
            waiting_for_operand: false,
            last_operator: None,
            last_operand: None,
            memory: Memory::new(),
            error: None,
            history: Vec::new(),
            show_history: false,
        }
    }

    /// 是否处于错误状态（除零等）。
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// 清除「连续按 =」的记忆，用户开始编辑数字后旧链条即失效。
    fn clear_repeat(&mut self) {
        self.last_operator = None;
        self.last_operand = None;
    }

    /// 输入一个数字。
    pub fn input_digit(&mut self, digit: char) {
        if self.error.is_some() {
            self.clear();
        }
        self.clear_repeat();
        if self.waiting_for_operand {
            // 刚按过运算符或 =：开启一个新操作数。
            // 若 tokens 为空（刚按完 =），说明要开始一道全新的算式，清掉旧算式。
            if self.tokens.is_empty() {
                self.expression.clear();
            }
            self.display.clear();
            self.waiting_for_operand = false;
        }
        // 避免出现 "07" 这种前导零。
        if self.display == "0" {
            self.display = digit.to_string();
        } else {
            self.display.push(digit);
        }
    }

    /// 输入小数点。
    pub fn input_dot(&mut self) {
        if self.error.is_some() {
            self.clear();
        }
        self.clear_repeat();
        if self.waiting_for_operand {
            if self.tokens.is_empty() {
                self.expression.clear();
            }
            self.display = "0".to_string();
            self.waiting_for_operand = false;
            self.display.push('.');
        } else if !self.display.contains('.') {
            self.display.push('.');
        }
    }

    /// 按下运算符。只把「数字 + 运算符」记入算式，真正的计算留到按 =。
    pub fn set_operator(&mut self, op: Op) {
        if self.error.is_some() {
            self.clear();
        }
        self.clear_repeat();

        if self.waiting_for_operand {
            if matches!(self.tokens.last(), Some(Token::Op(_))) {
                // 连按运算符：替换掉末尾那个运算符。
                *self.tokens.last_mut().expect("已判断末尾是运算符") = Token::Op(op);
            } else {
                // tokens 为空（例如刚按过 =）：以当前显示值为左操作数。
                self.tokens.push(Token::Num(self.display.clone()));
                self.tokens.push(Token::Op(op));
            }
        } else {
            // 把正在输入的数字和刚按下的运算符追加到算式末尾。
            self.tokens.push(Token::Num(self.display.clone()));
            self.tokens.push(Token::Op(op));
        }

        self.expression = tokens_to_string(&self.tokens);
        self.waiting_for_operand = true;
    }

    /// 按下等号：对整条算式按标准优先级求值。
    pub fn equals(&mut self) {
        // 没有算式：可能只是对单个数字按 =，也可能是连续按 =。
        if self.tokens.is_empty() {
            if let (Some(op), Some(right)) = (self.last_operator, self.last_operand) {
                // 连续按 =：用上一次的结果继续套用运算。
                let left = self.display.parse::<f64>().unwrap_or(0.0);
                self.expression = format!(
                    "{} {} {} =",
                    format_plain(left),
                    op.symbol(),
                    format_plain(right)
                );
                self.set_result(op.apply(left, right));
            } else {
                self.expression = format!("{} =", self.display);
            }
            self.waiting_for_operand = true;
            self.record_history();
            return;
        }

        let mut tokens = self.tokens.clone();

        // 补齐右操作数：末尾运算符悬空时，复用算式里最后一个数字。
        let right_text = if self.waiting_for_operand {
            tokens
                .iter()
                .rev()
                .find_map(|t| match t {
                    Token::Num(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "0".to_string())
        } else {
            self.display.clone()
        };
        tokens.push(Token::Num(right_text));

        // 记住最后一个运算符和操作数，供连续按 = 使用。
        let last_op = tokens.iter().rev().find_map(|t| match t {
            Token::Op(op) => Some(*op),
            _ => None,
        });
        let last_num = tokens.iter().rev().find_map(|t| match t {
            Token::Num(s) => s.parse::<f64>().ok(),
            _ => None,
        });

        let result = evaluate(&tokens);

        self.expression = format!("{} =", tokens_to_string(&tokens));
        self.set_result(result);

        self.tokens.clear();
        self.last_operator = last_op;
        self.last_operand = last_num;
        self.waiting_for_operand = true;
        self.record_history();
    }

    /// 写入求值结果；失败时显示错误文案并进入错误状态。
    fn set_result(&mut self, result: Result<f64, CalcError>) {
        match result {
            Ok(value) => self.set_value(value),
            Err(error) => self.set_error(error),
        }
    }

    /// 写入一个正常的显示值。非有限值（溢出、负数开方）转成错误状态。
    fn set_value(&mut self, value: f64) {
        if value.is_finite() {
            self.error = None;
            self.display = format_plain(value);
        } else {
            self.set_error(CalcError::InvalidInput);
        }
    }

    /// 进入错误状态：显示屏只放错误文案，后续按键会先清掉它。
    fn set_error(&mut self, error: CalcError) {
        self.display = error.message().to_string();
        self.error = Some(error);
    }

    // ---- 历史记录 ----

    /// 把当前算式与结果记入历史（只记录真正包含运算符、且没有出错的算式）。
    fn record_history(&mut self) {
        if self.error.is_some() {
            return;
        }
        let has_operator = self
            .expression
            .chars()
            .any(|c| matches!(c, '+' | '−' | '×' | '÷'));
        if !has_operator {
            return;
        }
        self.history.push(HistoryEntry {
            expression: self.expression.clone(),
            result: self.display.clone(),
        });
        // 最多保留 50 条，避免无限增长。
        if self.history.len() > 50 {
            self.history.remove(0);
        }
    }

    /// 点击历史条目：把结果回填到显示区，并关闭历史面板。
    pub fn recall_history(&mut self, index: usize) {
        let Some(entry) = self.history.get(index) else {
            return;
        };
        let result = entry.result.clone();
        self.clear();
        self.display = result;
        self.waiting_for_operand = true;
        self.show_history = false;
    }

    /// 清空全部历史。
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// 在「计算器」和「历史记录」之间切换。
    pub fn toggle_history(&mut self) {
        self.show_history = !self.show_history;
        if self.show_history {
            self.memory.close();
        }
    }

    /// 清空（C）。
    pub fn clear(&mut self) {
        self.display = "0".to_string();
        self.expression.clear();
        self.tokens.clear();
        self.waiting_for_operand = false;
        self.error = None;
        self.clear_repeat();
    }

    /// 清除当前输入（CE），保留已确认的算式。
    pub fn clear_entry(&mut self) {
        self.clear_repeat();
        self.display = "0".to_string();
        self.waiting_for_operand = false;
        self.error = None;
    }

    /// 退格删除一位。
    pub fn backspace(&mut self) {
        if self.error.is_some() {
            self.clear();
            return;
        }
        self.clear_repeat();
        if self.waiting_for_operand {
            return; // 结果刚出来时不响应退格
        }
        self.display.pop();
        if self.display.is_empty() || self.display == "-" {
            self.display = "0".to_string();
        }
    }

    /// 取相反数（正负号切换）。
    pub fn negate(&mut self) {
        if self.error.is_some() {
            return;
        }
        self.clear_repeat();
        if self.display == "0" {
            return;
        }
        if let Some(rest) = self.display.strip_prefix('-') {
            self.display = rest.to_string();
        } else {
            self.display.insert(0, '-');
        }
    }

    /// 百分比：除以 100。
    pub fn percent(&mut self) {
        if self.error.is_some() {
            return;
        }
        self.clear_repeat();
        let value = self.current_value() / 100.0;
        self.set_value(value);
    }

    /// 当前显示的数字。
    fn current_value(&self) -> f64 {
        self.display.parse::<f64>().unwrap_or(0.0)
    }

    /// 用一元函数处理当前显示值，结果仍显示在屏幕上。
    ///
    /// 一元运算后把 `waiting_for_operand` 置为 false：这样 `2 + 3 x² =`
    /// 会用平方后的结果当右操作数（`2 + 9 = 11`）。
    fn apply_unary(&mut self, f: impl Fn(f64) -> f64) {
        if self.error.is_some() {
            return;
        }
        self.clear_repeat();
        self.set_value(f(self.current_value()));
        self.waiting_for_operand = false;
    }

    /// 倒数 1/x。
    pub fn reciprocal(&mut self) {
        self.apply_unary(|v| 1.0 / v);
    }

    /// 平方 x²。
    pub fn square(&mut self) {
        self.apply_unary(|v| v * v);
    }

    /// 平方根 √x。
    pub fn sqrt(&mut self) {
        self.apply_unary(f64::sqrt);
    }

    // ---- 记忆 ----

    /// MS：把当前显示值存入一个新的记忆槽。
    pub fn memory_store(&mut self) {
        let value = self.current_value();
        self.memory.store(value);
    }

    /// MC：清空所有记忆。
    pub fn memory_clear(&mut self) {
        self.memory.clear();
    }

    /// MR：把最新一个记忆值取回到显示区。
    pub fn memory_recall(&mut self) {
        if let Some(value) = self.memory.latest() {
            self.load_value(value);
        }
    }

    /// 把指定记忆槽的值取回到显示区，并关闭记忆面板。
    pub fn memory_recall_at(&mut self, index: usize) {
        if let Some(value) = self.memory.get(index) {
            self.load_value(value);
            self.memory.close();
        }
    }

    /// M+：最新一个记忆槽加上当前显示值。
    pub fn memory_add(&mut self) {
        let value = self.current_value();
        self.memory.update_latest(value, |last| last + value);
    }

    /// M−：最新一个记忆槽减去当前显示值。
    pub fn memory_subtract(&mut self) {
        let value = self.current_value();
        self.memory.update_latest(-value, |last| last - value);
    }

    /// 把某个值载入显示区（从记忆取回）。
    fn load_value(&mut self, value: f64) {
        self.clear();
        self.set_value(value);
        self.waiting_for_operand = true;
    }

    /// 在「计算器」和「记忆列表」之间切换。
    pub fn toggle_memory(&mut self) {
        if self.memory.toggle_panel() {
            self.show_history = false;
        }
    }

    /// 处理一次按键。
    pub fn handle_key(&mut self, keystroke: &Keystroke) {
        // 带 Ctrl / Alt / Cmd 的组合键交给系统处理，不当作计算器输入。
        let modifiers = &keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform {
            return;
        }

        // 优先使用能直接打出来的字符，这样能正确处理 Shift 出来的 +、* 等。
        let typed = keystroke.key_char.as_deref().unwrap_or("");
        let mut iter = typed.chars();
        if let (Some(c), None) = (iter.next(), iter.next()) {
            match c {
                '0'..='9' => self.input_digit(c),
                '.' => self.input_dot(),
                '+' => self.set_operator(Op::Add),
                '-' => self.set_operator(Op::Sub),
                '*' | 'x' | 'X' => self.set_operator(Op::Mul),
                '/' => self.set_operator(Op::Div),
                '%' => self.percent(),
                '=' => self.equals(),
                _ => {}
            }
            return;
        }

        // 没有对应字符的功能键。
        match keystroke.key.as_str() {
            "enter" => self.equals(),
            "escape" | "delete" => self.clear(),
            "backspace" => self.backspace(),
            _ => {}
        }
    }

    /// 显示屏文本（Ctrl+C）。
    pub fn copy_text(&self) -> String {
        self.display.clone()
    }

    /// 从剪贴板粘贴一个数（Ctrl+V）。
    pub fn paste(&mut self, text: &str) {
        let cleaned: String = text
            .chars()
            .filter(|c| !c.is_whitespace() && *c != ',' && *c != '_')
            .collect();
        if let Ok(value) = cleaned.parse::<f64>() {
            self.clear();
            self.set_value(value);
            self.waiting_for_operand = true;
        }
    }
}

#[cfg(test)]
mod tests {
    // 注意：这里不能写 `use super::*;`，因为 `gpui_kit::*` 导出的 `test`
    // 宏会遮蔽内置的 `#[test]`，导致递归展开报错。
    use super::{CalcError, Op, StandardCalc, Token, evaluate};
    use crate::shared::number::format_plain;
    use gpui_kit::{Keystroke, Modifiers};

    /// 方便在测试里构造 token。
    fn num(value: impl ToString) -> Token {
        Token::Num(value.to_string())
    }

    fn op(op: Op) -> Token {
        Token::Op(op)
    }

    /// 把一组数字/运算符求值，失败则 panic（测试只关心成功路径）。
    fn eval(tokens: &[Token]) -> f64 {
        evaluate(tokens).expect("求值不应失败")
    }

    #[test]
    fn single_number() {
        assert_eq!(eval(&[num("42")]), 42.0);
    }

    #[test]
    fn addition_and_subtraction_left_to_right() {
        // 10 − 3 + 2 = 9（同级从左到右）
        let tokens = [num("10"), op(Op::Sub), num("3"), op(Op::Add), num("2")];
        assert_eq!(eval(&tokens), 9.0);
    }

    #[test]
    fn multiplication_before_addition() {
        // 2 + 3 × 4 = 14（旧实现会算成 20）
        let tokens = [num("2"), op(Op::Add), num("3"), op(Op::Mul), num("4")];
        assert_eq!(eval(&tokens), 14.0);
    }

    #[test]
    fn division_before_subtraction() {
        // 20 − 8 ÷ 2 = 16
        let tokens = [num("20"), op(Op::Sub), num("8"), op(Op::Div), num("2")];
        assert_eq!(eval(&tokens), 16.0);
    }

    #[test]
    fn mixed_chain() {
        // 2 + 3 × 4 − 6 ÷ 3 = 2 + 12 − 2 = 12
        let tokens = [
            num("2"),
            op(Op::Add),
            num("3"),
            op(Op::Mul),
            num("4"),
            op(Op::Sub),
            num("6"),
            op(Op::Div),
            num("3"),
        ];
        assert_eq!(eval(&tokens), 12.0);
    }

    #[test]
    fn division_by_zero_is_an_error() {
        let tokens = [num("1"), op(Op::Div), num("0")];
        assert_eq!(evaluate(&tokens), Err(CalcError::DivideByZero));
    }

    #[test]
    fn division_by_zero_shows_error_then_resets_on_input() {
        let mut calc = StandardCalc::new();
        calc.input_digit('1');
        calc.set_operator(Op::Div);
        calc.input_digit('0');
        calc.equals();
        assert!(calc.is_error());
        assert_eq!(calc.display, "不能被零除");
        assert!(calc.history.is_empty(), "出错的算式不进历史");

        // 再按数字会从错误状态恢复，而不是把文案当数字拼接。
        calc.input_digit('7');
        assert!(!calc.is_error());
        assert_eq!(calc.display, "7");
    }

    /// 显示规则来自 `shared::number`，与科学模式完全一致。
    #[test]
    fn display_uses_shared_number_rule() {
        for value in [0.1 + 0.2, 1.0 / 3.0, 1.5e20, 1.23e-12, 1024.0, -2.5] {
            let mut calc = StandardCalc::new();
            calc.paste(&value.to_string());
            assert_eq!(calc.display, format_plain(value), "值 {value}");
        }
    }

    /// 极大/极小数也用科学计数法，不再铺成一长串数字。
    #[test]
    fn large_and_small_values_use_scientific_notation() {
        let mut calc = StandardCalc::new();
        calc.paste("1e20");
        assert_eq!(calc.display, "1e+20");
        calc.paste("1e-12");
        assert_eq!(calc.display, "1e-12");
    }

    #[test]
    fn floating_point_is_cleaned_up_on_display() {
        // 0.1 + 0.2 在显示层被抹平
        let tokens = [num("0.1"), op(Op::Add), num("0.2")];
        assert_eq!(format_plain(eval(&tokens)), "0.3");
    }

    // ---- 以下是直接驱动 StandardCalc 状态机的“整机”测试 ----

    /// 模拟依次按下 `2 + 3 × 4 =`，返回最终显示。
    #[test]
    fn calculator_respects_precedence() {
        let mut calc = StandardCalc::new();
        calc.input_digit('2');
        calc.set_operator(Op::Add);
        calc.input_digit('3');
        calc.set_operator(Op::Mul);
        calc.input_digit('4');
        calc.equals();
        assert_eq!(calc.display, "14");
        assert_eq!(calc.expression, "2 + 3 × 4 =");
    }

    #[test]
    fn consecutive_equals_repeat_last_operation() {
        let mut calc = StandardCalc::new();
        calc.input_digit('2');
        calc.set_operator(Op::Add);
        calc.input_digit('3');
        calc.equals(); // 5
        assert_eq!(calc.display, "5");
        calc.equals(); // 8
        assert_eq!(calc.display, "8");
        calc.equals(); // 11
        assert_eq!(calc.display, "11");
    }

    #[test]
    fn pressing_operator_twice_replaces_it() {
        let mut calc = StandardCalc::new();
        calc.input_digit('6');
        calc.set_operator(Op::Add);
        calc.set_operator(Op::Mul); // 应替换成 ×
        calc.input_digit('3');
        calc.equals();
        assert_eq!(calc.display, "18");
    }

    #[test]
    fn new_expression_after_equals() {
        let mut calc = StandardCalc::new();
        calc.input_digit('2');
        calc.set_operator(Op::Add);
        calc.input_digit('3');
        calc.equals(); // 5
        // 直接接着按运算符，应以上一次结果为左操作数。
        calc.set_operator(Op::Mul);
        calc.input_digit('4');
        calc.equals();
        assert_eq!(calc.display, "20");
    }

    // ---- 一元运算 / CE ----

    #[test]
    fn unary_operations() {
        let mut calc = StandardCalc::new();
        calc.input_digit('4');
        calc.reciprocal();
        assert_eq!(calc.display, "0.25");

        calc.clear();
        calc.input_digit('4');
        calc.square();
        assert_eq!(calc.display, "16");

        calc.clear();
        calc.input_digit('9');
        calc.sqrt();
        assert_eq!(calc.display, "3");
    }

    /// 负数开方不是「显示成一个字符串」，而是真正的错误状态：再按数字会重新开始。
    #[test]
    fn sqrt_of_negative_is_error() {
        let mut calc = StandardCalc::new();
        calc.input_digit('9');
        calc.negate();
        calc.sqrt();
        assert!(calc.is_error());
        assert_eq!(calc.display, "无效输入");

        calc.input_digit('4');
        assert!(!calc.is_error());
        assert_eq!(calc.display, "4");
    }

    #[test]
    fn unary_operation_feeds_following_equals() {
        // 2 + 3 x² = 2 + 9 = 11
        let mut calc = StandardCalc::new();
        calc.input_digit('2');
        calc.set_operator(Op::Add);
        calc.input_digit('3');
        calc.square();
        calc.equals();
        assert_eq!(calc.display, "11");
    }

    #[test]
    fn clear_entry_only_clears_current_operand() {
        let mut calc = StandardCalc::new();
        calc.input_digit('1');
        calc.set_operator(Op::Add);
        calc.input_digit('2');
        calc.clear_entry();
        assert_eq!(calc.display, "0");
        calc.input_digit('3');
        calc.equals();
        assert_eq!(calc.display, "4"); // 1 + 3
    }

    // ---- 记忆功能 ----

    #[test]
    fn memory_operations_round_trip() {
        let mut calc = StandardCalc::new();
        calc.input_digit('5');
        calc.memory_store();
        assert_eq!(calc.memory.to_vec(), vec![5.0]);

        calc.clear();
        calc.memory_recall();
        assert_eq!(calc.display, "5");

        calc.memory_add(); // 5 + 5
        assert_eq!(calc.memory.to_vec(), vec![10.0]);
        calc.memory_subtract(); // 10 − 5
        assert_eq!(calc.memory.to_vec(), vec![5.0]);

        calc.memory_clear();
        assert!(calc.memory.is_empty());
    }

    #[test]
    fn memory_supports_multiple_slots() {
        let mut calc = StandardCalc::new();
        calc.input_digit('3');
        calc.memory_store();
        calc.clear();
        calc.input_digit('4');
        calc.memory_store();
        assert_eq!(calc.memory.to_vec(), vec![3.0, 4.0]);

        // MR 取回最新一个
        calc.clear();
        calc.memory_recall();
        assert_eq!(calc.display, "4");

        // 取回较早的槽，并关闭面板
        calc.memory.open();
        calc.memory_recall_at(0);
        assert_eq!(calc.display, "3");
        assert!(!calc.memory.is_open());
    }

    #[test]
    fn memory_add_without_slot_creates_one() {
        let mut calc = StandardCalc::new();
        calc.input_digit('7');
        calc.memory_add();
        assert_eq!(calc.memory.to_vec(), vec![7.0]);
    }

    #[test]
    fn toggle_memory_ignores_empty_list() {
        let mut calc = StandardCalc::new();
        calc.toggle_memory();
        assert!(!calc.memory.is_open());

        calc.input_digit('1');
        calc.memory_store();
        calc.toggle_memory();
        assert!(calc.memory.is_open());
    }

    // ---- 历史记录 ----

    #[test]
    fn history_records_calculation() {
        let mut calc = StandardCalc::new();
        calc.input_digit('2');
        calc.set_operator(Op::Add);
        calc.input_digit('3');
        calc.equals();
        assert_eq!(calc.history.len(), 1);
        assert_eq!(calc.history[0].expression, "2 + 3 =");
        assert_eq!(calc.history[0].result, "5");
    }

    #[test]
    fn plain_number_equals_is_not_recorded() {
        let mut calc = StandardCalc::new();
        calc.input_digit('7');
        calc.equals();
        assert!(calc.history.is_empty());
    }

    #[test]
    fn recall_history_loads_result_and_closes_panel() {
        let mut calc = StandardCalc::new();
        calc.input_digit('2');
        calc.set_operator(Op::Add);
        calc.input_digit('3');
        calc.equals();
        calc.clear();
        calc.show_history = true;
        calc.recall_history(0);
        assert_eq!(calc.display, "5");
        assert!(!calc.show_history);
    }

    #[test]
    fn clear_history_empties_list() {
        let mut calc = StandardCalc::new();
        calc.input_digit('1');
        calc.set_operator(Op::Add);
        calc.input_digit('1');
        calc.equals();
        calc.clear_history();
        assert!(calc.history.is_empty());
    }

    // ---- 键盘输入 ----

    /// 构造一个只带 `key` 与 `key_char` 的按键事件。
    fn key(key: &str, key_char: Option<&str>) -> Keystroke {
        Keystroke {
            modifiers: Modifiers::default(),
            key: key.to_string(),
            key_char: key_char.map(|s| s.to_string()),
        }
    }

    #[test]
    fn keyboard_typing_respects_precedence() {
        let mut calc = StandardCalc::new();
        // 依次敲 1 + 2 * 3，然后回车
        for k in ["1", "+", "2", "*", "3"] {
            calc.handle_key(&key(k, Some(k)));
        }
        calc.handle_key(&key("enter", None));
        assert_eq!(calc.display, "7");
    }

    #[test]
    fn keyboard_escape_clears() {
        let mut calc = StandardCalc::new();
        calc.handle_key(&key("5", Some("5")));
        assert_eq!(calc.display, "5");
        calc.handle_key(&key("escape", None));
        assert_eq!(calc.display, "0");
        assert!(calc.expression.is_empty());
    }

    #[test]
    fn keyboard_trailing_operator_reuses_operand() {
        let mut calc = StandardCalc::new();
        calc.handle_key(&key("7", Some("7")));
        calc.handle_key(&key("+", Some("+")));
        calc.handle_key(&key("enter", None));
        // 末尾运算符悬空，复用最后一个数字：7 + 7 = 14
        assert_eq!(calc.display, "14");
    }

    #[test]
    fn keyboard_ignores_ctrl_combos() {
        let mut calc = StandardCalc::new();
        let mut keystroke = key("5", Some("5"));
        keystroke.modifiers.control = true;
        calc.handle_key(&keystroke);
        assert_eq!(calc.display, "0");
    }
}
