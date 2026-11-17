//! 科学模式的状态机：输入、连续等号、角度切换、记忆与历史。

use super::expr::{SToken, evaluate, paren_depth, tokens_to_string};
use super::ops::{AngleMode, Constant, MathFn, SOp, Trig, Unary};
use crate::shared::eval::{CalcError, Operator};
use crate::shared::history;
use crate::shared::memory::Memory;
use crate::shared::number::format_value;

pub use crate::shared::history::HistoryEntry;

/// 科学模式能理解的用户输入意图。
///
/// 用纯数据描述按键语义，让 [`ScientificCalc`] 不依赖任何 UI 类型；
/// 具体的 `Keystroke` → [`Action`] 转换在 [`crate::modes::scientific::view`] 里完成。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    /// 输入一个数字字符（'0'..='9'）。
    Digit(char),
    /// 小数点。
    Dot,
    /// 二元运算符。
    Operator(SOp),
    /// 左括号。
    OpenParen,
    /// 右括号。
    CloseParen,
    /// 等号 / 回车。
    Equals,
    /// 全部清除（C / Esc）。
    Clear,
    /// 清除当前输入（CE / Delete）。
    ClearEntry,
    /// 退格删除一位。
    Backspace,
    /// 正负号切换。
    Negate,
    /// 输入常量（π / e）。
    PressConst(Constant),
    /// 一元函数（x²、√x、n! 等）。
    ApplyUnary(Unary),
    /// 三角函数。
    ApplyTrig(Trig),
    /// 其他数学函数（双曲、取整等）。
    ApplyMath(MathFn),
    /// 切换 2nd 功能键。
    ToggleSecond,
    /// 切换 F-E 科学计数法显示。
    ToggleScientificNotation,
    /// 角度制在 DEG / RAD / GRAD 间循环。
    CycleAngle,
}

/// 「科学」模式的状态。
pub struct ScientificCalc {
    /// 主显示屏文本。
    pub display: String,
    /// 上方小号算式。
    pub expression: String,
    /// 已确认的算式片段。
    tokens: Vec<SToken>,
    /// 下一次输入数字时开启新的操作数。
    waiting_for_operand: bool,
    /// 连续按 = 时复用的运算符。
    last_op: Option<SOp>,
    /// 连续按 = 时复用的右操作数。
    last_operand: Option<f64>,
    /// 当前错误。
    error: Option<CalcError>,
    /// 刚按下的常量（显示 π / e，参与运算时取真实值）。
    pending_const: Option<Constant>,
    /// 角度单位。
    pub angle: AngleMode,
    /// 是否用科学计数法显示（F-E）。
    pub scientific_notation: bool,
    /// 2nd 功能键状态。
    pub second: bool,
    /// 记忆槽（MS / MR / MC / M+ / M−），面板开关也在这里。
    pub memory: Memory<f64>,
    /// 计算历史。
    pub history: Vec<HistoryEntry>,
    /// 是否显示历史面板。
    pub show_history: bool,
}

impl ScientificCalc {
    pub fn new() -> Self {
        Self {
            display: "0".to_string(),
            expression: String::new(),
            tokens: Vec::new(),
            waiting_for_operand: true,
            last_op: None,
            last_operand: None,
            error: None,
            pending_const: None,
            angle: AngleMode::Deg,
            scientific_notation: false,
            second: false,
            memory: Memory::new(),
            history: Vec::new(),
            show_history: false,
        }
    }

    /// 显示屏主数字。
    pub fn display(&self) -> String {
        self.display.clone()
    }

    pub fn expression(&self) -> &str {
        &self.expression
    }

    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// 当前显示值（常量取精确值）。
    fn current_value(&self) -> f64 {
        if let Some(constant) = self.pending_const {
            return constant.value();
        }
        self.display.parse::<f64>().unwrap_or(0.0)
    }

    fn clear_repeat(&mut self) {
        self.last_op = None;
        self.last_operand = None;
    }

    fn set_value(&mut self, value: f64) {
        self.pending_const = None;
        if value.is_finite() {
            self.display = format_value(value, self.scientific_notation);
            self.error = None;
        } else {
            self.set_error(CalcError::InvalidInput);
        }
    }

    fn set_error(&mut self, error: CalcError) {
        self.pending_const = None;
        self.display = error.message().to_string();
        self.error = Some(error);
    }

    // ---- 开关 ----

    pub fn toggle_second(&mut self) {
        self.second = !self.second;
    }

    pub fn toggle_scientific_notation(&mut self) {
        self.scientific_notation = !self.scientific_notation;
        // 把当前显示值换成另一种写法。
        if self.error.is_none() && self.pending_const.is_none() {
            self.set_value(self.current_value());
        }
    }

    pub fn cycle_angle(&mut self) {
        self.angle = self.angle.next();
    }

    /// 输入一个常量（π / e）。
    pub fn press_const(&mut self, constant: Constant) {
        if self.error.is_some() {
            self.clear();
        }
        self.clear_repeat();
        if self.waiting_for_operand && self.tokens.is_empty() {
            self.expression.clear();
        }
        self.display = constant.symbol().to_string();
        self.pending_const = Some(constant);
        self.waiting_for_operand = false;
    }

    // ---- 输入 ----

    pub fn input_digit(&mut self, digit: char) {
        if self.error.is_some() {
            self.clear();
        }
        self.clear_repeat();
        self.pending_const = None;

        if self.waiting_for_operand {
            if self.tokens.is_empty() {
                self.expression.clear();
            }
            self.display.clear();
            self.waiting_for_operand = false;
        }
        if self.display == "0" {
            self.display.clear();
        }
        self.display.push(digit);
    }

    pub fn input_dot(&mut self) {
        if self.error.is_some() {
            self.clear();
        }
        self.clear_repeat();
        self.pending_const = None;

        if self.waiting_for_operand {
            if self.tokens.is_empty() {
                self.expression.clear();
            }
            self.display = "0".to_string();
            self.waiting_for_operand = false;
        }
        if !self.display.contains('.') {
            self.display.push('.');
        }
    }

    pub fn backspace(&mut self) {
        self.clear_repeat();
        if self.waiting_for_operand || self.error.is_some() {
            return;
        }
        self.pending_const = None;
        self.display.pop();
        if self.display.is_empty() || self.display == "-" {
            self.display = "0".to_string();
        }
    }

    /// 全部清除（C）。
    pub fn clear(&mut self) {
        self.display = "0".to_string();
        self.expression.clear();
        self.tokens.clear();
        self.waiting_for_operand = true;
        self.error = None;
        self.pending_const = None;
        self.clear_repeat();
    }

    /// 清除当前输入（CE / Delete），保留已确认的算式。
    pub fn clear_entry(&mut self) {
        self.clear_repeat();
        self.display = "0".to_string();
        self.waiting_for_operand = true;
        self.error = None;
        self.pending_const = None;
    }

    pub fn negate(&mut self) {
        if self.error.is_some() {
            return;
        }
        self.clear_repeat();
        if self.pending_const.is_some() {
            let value = -self.current_value();
            self.set_value(value);
            self.waiting_for_operand = false;
            return;
        }
        if self.display == "0" {
            return;
        }
        if let Some(rest) = self.display.strip_prefix('-') {
            self.display = rest.to_string();
        } else {
            self.display.insert(0, '-');
        }
        self.waiting_for_operand = false;
    }

    // ---- 一元函数 ----

    fn apply_function(&mut self, f: impl Fn(f64) -> f64) {
        if self.error.is_some() {
            self.clear();
        }
        self.clear_repeat();
        let value = f(self.current_value());
        self.set_value(value);
        self.waiting_for_operand = false;
    }

    pub fn apply_unary(&mut self, unary: Unary) {
        self.apply_function(|value| unary.apply(value));
    }

    pub fn apply_trig(&mut self, trig: Trig) {
        let angle = self.angle;
        self.apply_function(|value| trig.apply(value, angle));
    }

    pub fn apply_math(&mut self, function: MathFn) {
        self.apply_function(|value| function.apply(value));
    }

    // ---- 表达式 ----

    /// 把当前输入（数字或常量）追加到 token 序列里。
    fn push_operand(&mut self) {
        if let Some(constant) = self.pending_const.take() {
            self.tokens.push(SToken::Const(constant));
        } else {
            self.tokens.push(SToken::Num(self.current_value()));
        }
    }

    pub fn set_operator(&mut self, op: SOp) {
        if self.error.is_some() {
            return;
        }
        self.clear_repeat();

        if self.waiting_for_operand {
            match self.tokens.last() {
                Some(SToken::Op(_)) => {
                    *self.tokens.last_mut().expect("已判断末尾是运算符") = SToken::Op(op);
                }
                // 左括号后不能直接跟运算符。
                Some(SToken::LParen) => return,
                Some(SToken::RParen) => self.tokens.push(SToken::Op(op)),
                _ => {
                    self.push_operand();
                    self.tokens.push(SToken::Op(op));
                }
            }
        } else {
            self.push_operand();
            self.tokens.push(SToken::Op(op));
        }

        self.expression = tokens_to_string(&self.tokens, self.scientific_notation);
        self.waiting_for_operand = true;
    }

    /// 左括号。数字后面直接按 `(` 时补一个乘号。
    pub fn open_paren(&mut self) {
        if self.error.is_some() {
            self.clear();
        }
        self.clear_repeat();

        if !self.waiting_for_operand {
            self.push_operand();
            self.tokens.push(SToken::Op(SOp::Mul));
        }
        self.tokens.push(SToken::LParen);
        self.expression = tokens_to_string(&self.tokens, self.scientific_notation);
        self.waiting_for_operand = true;
    }

    /// 右括号。没有未闭合的左括号时忽略。
    pub fn close_paren(&mut self) {
        if paren_depth(&self.tokens) == 0 {
            return;
        }
        self.clear_repeat();

        if !self.waiting_for_operand
            || matches!(
                self.tokens.last(),
                Some(SToken::Op(_)) | Some(SToken::LParen)
            )
        {
            self.push_operand();
        }
        self.tokens.push(SToken::RParen);
        self.expression = tokens_to_string(&self.tokens, self.scientific_notation);
        self.waiting_for_operand = true;
    }

    pub fn equals(&mut self) {
        if self.error.is_some() {
            return;
        }

        // 没有算式：连续按 = 或对单个数字按 =。
        if self.tokens.is_empty() {
            if let (Some(op), Some(operand)) = (self.last_op, self.last_operand) {
                let left = self.current_value();
                let expression = format!(
                    "{} {} {} =",
                    format_value(left, self.scientific_notation),
                    op.symbol(),
                    format_value(operand, self.scientific_notation)
                );
                match op.apply(left, operand) {
                    Ok(value) => self.set_value(value),
                    Err(error) => self.set_error(error),
                }
                self.expression = expression;
            } else {
                self.expression = format!("{} =", self.display());
            }
            self.waiting_for_operand = true;
            return;
        }

        let mut tokens = self.tokens.clone();
        let last_number = tokens.iter().rev().find_map(|token| match token {
            SToken::Num(value) => Some(*value),
            SToken::Const(constant) => Some(constant.value()),
            _ => None,
        });

        if self.waiting_for_operand {
            // 末尾运算符悬空时复用最后一个数字。
            if matches!(tokens.last(), Some(SToken::Op(_))) {
                tokens.push(SToken::Num(last_number.unwrap_or(self.current_value())));
            }
        } else if let Some(constant) = self.pending_const.take() {
            tokens.push(SToken::Const(constant));
        } else {
            tokens.push(SToken::Num(self.current_value()));
        }

        // 自动补齐没有闭合的括号。
        let mut depth = paren_depth(&tokens);
        while depth > 0 {
            tokens.push(SToken::RParen);
            depth -= 1;
        }

        let last_op = tokens.iter().rev().find_map(|token| match token {
            SToken::Op(op) => Some(*op),
            _ => None,
        });
        let last_operand = tokens.iter().rev().find_map(|token| match token {
            SToken::Num(value) => Some(*value),
            SToken::Const(constant) => Some(constant.value()),
            _ => None,
        });
        let has_operator = tokens.iter().any(|token| matches!(token, SToken::Op(_)));

        match evaluate(&tokens) {
            Ok(value) => self.set_value(value),
            Err(error) => self.set_error(error),
        }

        self.expression = format!("{} =", tokens_to_string(&tokens, self.scientific_notation));
        self.tokens.clear();
        self.last_op = last_op;
        self.last_operand = last_operand;
        self.waiting_for_operand = true;

        if has_operator && self.error.is_none() {
            let entry = HistoryEntry {
                expression: self.expression.clone(),
                result: self.display(),
            };
            history::push(&mut self.history, entry);
        }
    }

    // ---- 历史 ----

    pub fn toggle_history(&mut self) {
        self.show_history = !self.show_history;
        if self.show_history {
            self.memory.close();
        }
    }

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

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    // ---- 记忆 ----

    pub fn memory_store(&mut self) {
        self.memory.store(self.current_value());
    }

    pub fn memory_clear(&mut self) {
        self.memory.clear();
    }

    pub fn memory_recall(&mut self) {
        if let Some(value) = self.memory.latest() {
            self.load_value(value);
        }
    }

    pub fn memory_recall_at(&mut self, index: usize) {
        if let Some(value) = self.memory.get(index) {
            self.load_value(value);
            self.memory.close();
        }
    }

    pub fn memory_add(&mut self) {
        let value = self.current_value();
        self.memory.update_latest(value, |last| last + value);
    }

    pub fn memory_subtract(&mut self) {
        let value = self.current_value();
        self.memory.update_latest(-value, |last| last - value);
    }

    pub fn toggle_memory(&mut self) {
        if self.memory.toggle_panel() {
            self.show_history = false;
        }
    }

    fn load_value(&mut self, value: f64) {
        self.clear();
        self.display = format_value(value, self.scientific_notation);
        self.waiting_for_operand = true;
    }

    // ---- 输入动作 ----

    /// 执行一个语义动作。
    ///
    /// 所有键盘 / 按钮输入最终都通过这里驱动状态机；状态机本身不认识任何 UI 类型，
    /// 按键到 [`Action`] 的翻译由 UI 层负责。
    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::Digit(d) => self.input_digit(d),
            Action::Dot => self.input_dot(),
            Action::Operator(op) => self.set_operator(op),
            Action::OpenParen => self.open_paren(),
            Action::CloseParen => self.close_paren(),
            Action::Equals => self.equals(),
            Action::Clear => self.clear(),
            Action::ClearEntry => self.clear_entry(),
            Action::Backspace => self.backspace(),
            Action::Negate => self.negate(),
            Action::PressConst(constant) => self.press_const(constant),
            Action::ApplyUnary(unary) => self.apply_unary(unary),
            Action::ApplyTrig(trig) => self.apply_trig(trig),
            Action::ApplyMath(function) => self.apply_math(function),
            Action::ToggleSecond => self.toggle_second(),
            Action::ToggleScientificNotation => self.toggle_scientific_notation(),
            Action::CycleAngle => self.cycle_angle(),
        }
    }

    /// 显示屏文本（Ctrl+C）。
    pub fn copy_text(&self) -> String {
        self.display()
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

impl Default for ScientificCalc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Action, AngleMode, CalcError, Constant, MathFn, SOp, SToken, ScientificCalc, Trig, Unary,
        evaluate, format_value,
    };
    use crate::shared::number::format_plain;

    fn num(value: f64) -> SToken {
        SToken::Num(value)
    }

    fn op(op: SOp) -> SToken {
        SToken::Op(op)
    }

    fn approx(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    /// 显示走共享规则，所以和标准模式给出同一段文本（规则本身在 `shared::number` 里测）。
    #[test]
    fn display_uses_shared_number_rule() {
        let mut calc = ScientificCalc::new();
        calc.input_digit('1');
        calc.set_operator(SOp::Div);
        calc.input_digit('3');
        calc.equals();

        assert_eq!(calc.display(), format_plain(1.0 / 3.0));
        assert_eq!(calc.display(), "0.33333333333333");
    }

    /// F-E 开关只切换显示形式，不改变数值。
    #[test]
    fn scientific_notation_toggle_only_changes_display() {
        let mut calc = ScientificCalc::new();
        calc.input_digit('1');
        calc.input_digit('2');
        calc.input_digit('3');
        calc.input_digit('4');

        calc.toggle_scientific_notation();
        assert_eq!(calc.display(), format_value(1234.0, true));
        assert_eq!(calc.display(), "1.234e+3");
    }

    #[test]
    fn power_binds_tighter_than_multiplication() {
        // 2 + 3 ^ 2 = 11
        let tokens = [num(2.0), op(SOp::Add), num(3.0), op(SOp::Pow), num(2.0)];
        assert_eq!(evaluate(&tokens), Ok(11.0));
    }

    #[test]
    fn power_is_right_associative() {
        // 2 ^ 3 ^ 2 = 2 ^ (3 ^ 2) = 512（见 ADR-0001）
        let tokens = [num(2.0), op(SOp::Pow), num(3.0), op(SOp::Pow), num(2.0)];
        assert_eq!(evaluate(&tokens), Ok(512.0));
    }

    #[test]
    fn parentheses_override_precedence() {
        // (2 + 3) ^ 2 = 25
        let tokens = [
            SToken::LParen,
            num(2.0),
            op(SOp::Add),
            num(3.0),
            SToken::RParen,
            op(SOp::Pow),
            num(2.0),
        ];
        assert_eq!(evaluate(&tokens), Ok(25.0));
    }

    #[test]
    fn exp_and_log_base_are_binary() {
        // 2 exp 3 = 2000
        assert_eq!(evaluate(&[num(2.0), op(SOp::Exp), num(3.0)]), Ok(2000.0));
        // 8 logᵧ 2 = 3
        assert_eq!(evaluate(&[num(8.0), op(SOp::LogBase), num(2.0)]), Ok(3.0));
    }

    #[test]
    fn division_by_zero_is_an_error() {
        assert_eq!(
            evaluate(&[num(1.0), op(SOp::Div), num(0.0)]),
            Err(CalcError::DivideByZero)
        );
    }

    #[test]
    fn unary_functions() {
        assert_eq!(Unary::Square.apply(4.0), 16.0);
        assert_eq!(Unary::Sqrt.apply(9.0), 3.0);
        assert_eq!(Unary::Factorial.apply(5.0), 120.0);
        assert!(Unary::Factorial.apply(-1.0).is_nan());
        assert_eq!(Unary::Log10.apply(100.0), 2.0);
        assert_eq!(Unary::TenPow.apply(2.0), 100.0);
    }

    #[test]
    fn trig_respects_angle_mode() {
        assert!(approx(Trig::Sin.apply(30.0, AngleMode::Deg), 0.5));
        assert!(approx(Trig::Asin.apply(0.5, AngleMode::Deg), 30.0));
        assert!(approx(Trig::Cos.apply(0.0, AngleMode::Rad), 1.0));
        assert!(approx(Trig::Sin.apply(100.0, AngleMode::Grad), 1.0));
    }

    #[test]
    fn math_functions() {
        assert_eq!(MathFn::Floor.apply(-1.2), -2.0);
        assert_eq!(MathFn::Ceil.apply(-1.2), -1.0);
        assert_eq!(MathFn::Round.apply(1.6), 2.0);
        assert!(approx(MathFn::Sinh.apply(0.0), 0.0));
        assert!(approx(MathFn::Atanh.apply(0.5), 0.5493061443340549));
    }

    /// 依次按下一串按键（数字和运算符），最后按 =。
    fn press_keys(calc: &mut ScientificCalc, keys: &[char]) {
        for key in keys {
            match key {
                '0'..='9' => calc.input_digit(*key),
                '.' => calc.input_dot(),
                '+' => calc.set_operator(SOp::Add),
                '-' => calc.set_operator(SOp::Sub),
                '*' => calc.set_operator(SOp::Mul),
                '/' => calc.set_operator(SOp::Div),
                '^' => calc.set_operator(SOp::Pow),
                '(' => calc.open_paren(),
                ')' => calc.close_paren(),
                _ => {}
            }
        }
        calc.equals();
    }

    #[test]
    fn state_machine_uses_precedence() {
        let mut calc = ScientificCalc::new();
        press_keys(&mut calc, &['2', '+', '3', '*', '4']);
        assert_eq!(calc.display(), "14");
        assert_eq!(calc.expression(), "2 + 3 × 4 =");
    }

    #[test]
    fn state_machine_supports_parentheses() {
        let mut calc = ScientificCalc::new();
        press_keys(&mut calc, &['(', '2', '+', '3', ')', '^', '2']);
        assert_eq!(calc.display(), "25");
    }

    #[test]
    fn square_root_and_square() {
        let mut calc = ScientificCalc::new();
        calc.input_digit('9');
        calc.apply_unary(Unary::Sqrt);
        assert_eq!(calc.display(), "3");
        calc.apply_unary(Unary::Square);
        assert_eq!(calc.display(), "9");
    }

    #[test]
    fn constants_are_exact_in_expressions() {
        let mut calc = ScientificCalc::new();
        calc.press_const(Constant::Pi);
        assert_eq!(calc.display(), "π");
        calc.set_operator(SOp::Mul);
        calc.input_digit('2');
        calc.equals();
        assert_eq!(calc.display(), "6.2831853071796");
    }

    #[test]
    fn angle_mode_cycles_deg_rad_grad() {
        let mut calc = ScientificCalc::new();
        assert_eq!(calc.angle, AngleMode::Deg);
        calc.cycle_angle();
        assert_eq!(calc.angle, AngleMode::Rad);
        calc.cycle_angle();
        assert_eq!(calc.angle, AngleMode::Grad);
        calc.cycle_angle();
        assert_eq!(calc.angle, AngleMode::Deg);
    }

    #[test]
    fn fe_toggle_reformats_display() {
        let mut calc = ScientificCalc::new();
        for ch in ['1', '2', '3', '4', '5'] {
            calc.input_digit(ch);
        }
        assert_eq!(calc.display(), "12345");
        calc.toggle_scientific_notation();
        assert_eq!(calc.display(), "1.2345e+4");
        calc.toggle_scientific_notation();
        assert_eq!(calc.display(), "12345");
    }

    #[test]
    fn second_toggle_changes_behavior() {
        let mut calc = ScientificCalc::new();
        calc.toggle_second();
        assert!(calc.second);
        // 9 的 2nd + √ 是立方根。
        calc.input_digit('8');
        calc.apply_unary(Unary::Cbrt);
        assert_eq!(calc.display(), "2");
    }

    #[test]
    fn inverse_trig_through_state_machine() {
        let mut calc = ScientificCalc::new();
        calc.input_digit('0');
        calc.input_dot();
        calc.input_digit('5');
        calc.apply_trig(Trig::Asin);
        assert_eq!(calc.display(), "30");
    }

    #[test]
    fn memory_round_trip() {
        let mut calc = ScientificCalc::new();
        calc.input_digit('7');
        calc.memory_store();
        calc.clear();
        calc.memory_recall();
        assert_eq!(calc.display(), "7");
        calc.memory_add();
        assert_eq!(calc.memory.to_vec(), vec![14.0]);
        calc.memory_clear();
        assert!(calc.memory.is_empty());
    }

    #[test]
    fn history_records_operators_only() {
        let mut calc = ScientificCalc::new();
        press_keys(&mut calc, &['2', '+', '2']);
        assert_eq!(calc.history.len(), 1);
        assert_eq!(calc.history[0].expression, "2 + 2 =");
        assert_eq!(calc.history[0].result, "4");
    }

    // ---- 动作分发 ----

    /// [`Action`] 必须能正确路由到对应的语义方法。
    #[test]
    fn apply_action_dispatches_to_semantic_methods() {
        let mut calc = ScientificCalc::new();

        // 2 + 3 * 4 = 14
        calc.apply_action(Action::Digit('2'));
        assert_eq!(calc.display(), "2");
        calc.apply_action(Action::Operator(SOp::Add));
        calc.apply_action(Action::Digit('3'));
        calc.apply_action(Action::Operator(SOp::Mul));
        calc.apply_action(Action::Digit('4'));
        calc.apply_action(Action::Equals);
        assert_eq!(calc.display(), "14");

        calc.apply_action(Action::Clear);
        assert_eq!(calc.display(), "0");
        assert!(calc.expression().is_empty());

        // 括号与幂：(2 + 3) ^ 2 = 25
        calc.apply_action(Action::OpenParen);
        calc.apply_action(Action::Digit('2'));
        calc.apply_action(Action::Operator(SOp::Add));
        calc.apply_action(Action::Digit('3'));
        calc.apply_action(Action::CloseParen);
        calc.apply_action(Action::Operator(SOp::Pow));
        calc.apply_action(Action::Digit('2'));
        calc.apply_action(Action::Equals);
        assert_eq!(calc.display(), "25");
    }

    #[test]
    fn apply_action_unary_and_constants() {
        let mut calc = ScientificCalc::new();

        // π * 2
        calc.apply_action(Action::PressConst(Constant::Pi));
        assert_eq!(calc.display(), "π");
        calc.apply_action(Action::Operator(SOp::Mul));
        calc.apply_action(Action::Digit('2'));
        calc.apply_action(Action::Equals);
        assert_eq!(calc.display(), "6.2831853071796");

        // 一元函数：9 的平方根
        calc.apply_action(Action::Clear);
        calc.apply_action(Action::Digit('9'));
        calc.apply_action(Action::ApplyUnary(Unary::Sqrt));
        assert_eq!(calc.display(), "3");

        // 三角函数（DEG 模式）：sin(30)
        calc.apply_action(Action::Clear);
        calc.apply_action(Action::Digit('3'));
        calc.apply_action(Action::Digit('0'));
        calc.apply_action(Action::ApplyTrig(Trig::Sin));
        assert_eq!(calc.display(), "0.5");
    }

    #[test]
    fn apply_action_toggles() {
        let mut calc = ScientificCalc::new();

        calc.apply_action(Action::CycleAngle);
        assert_eq!(calc.angle, AngleMode::Rad);

        calc.apply_action(Action::ToggleSecond);
        assert!(calc.second);

        calc.apply_action(Action::ToggleScientificNotation);
        assert!(calc.scientific_notation);
    }

    #[test]
    fn apply_action_clear_entry_preserves_expression() {
        let mut calc = ScientificCalc::new();
        calc.apply_action(Action::Digit('1'));
        calc.apply_action(Action::Operator(SOp::Add));
        calc.apply_action(Action::Digit('2'));
        calc.apply_action(Action::ClearEntry);
        assert_eq!(calc.display(), "0");
        calc.apply_action(Action::Digit('3'));
        calc.apply_action(Action::Equals);
        assert_eq!(calc.display(), "4");
    }
}
