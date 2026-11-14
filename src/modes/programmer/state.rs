//! 程序员模式的状态机：进制、位宽、运算符语义、求值、记忆与位翻转。

use crate::shared::eval::{self, Assoc, CalcError, Language, Operator};
use crate::shared::memory::Memory;
use gpui_kit::Keystroke;

// ---------------------------------------------------------------------------
// 进制 / 位宽 / 运算符
// ---------------------------------------------------------------------------

/// 显示与输入使用的进制。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Base {
    Hex,
    Dec,
    Oct,
    Bin,
}

impl Base {
    pub const ALL: [Base; 4] = [Base::Hex, Base::Dec, Base::Oct, Base::Bin];

    pub fn radix(self) -> u64 {
        match self {
            Base::Hex => 16,
            Base::Dec => 10,
            Base::Oct => 8,
            Base::Bin => 2,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Base::Hex => "HEX",
            Base::Dec => "DEC",
            Base::Oct => "OCT",
            Base::Bin => "BIN",
        }
    }

    /// 把字符解析成该进制下的一位数字。
    fn digit(self, c: char) -> Option<u64> {
        let d = c.to_digit(16)? as u64;
        (d < self.radix()).then_some(d)
    }
}

/// 位宽（QWORD / DWORD / WORD / BYTE）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WordSize {
    Qword,
    Dword,
    Word,
    Byte,
}

impl WordSize {
    pub const ALL: [WordSize; 4] = [
        WordSize::Qword,
        WordSize::Dword,
        WordSize::Word,
        WordSize::Byte,
    ];

    pub fn bits(self) -> u32 {
        match self {
            WordSize::Qword => 64,
            WordSize::Dword => 32,
            WordSize::Word => 16,
            WordSize::Byte => 8,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WordSize::Qword => "QWORD",
            WordSize::Dword => "DWORD",
            WordSize::Word => "WORD",
            WordSize::Byte => "BYTE",
        }
    }

    fn mask(self) -> u64 {
        match self.bits() {
            64 => u64::MAX,
            bits => (1u64 << bits) - 1,
        }
    }
}

/// 二元运算符：四则运算 + 位运算 + 移位。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Nand,
    Nor,
    Lsh,
    Rsh,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "−",
            BinOp::Mul => "×",
            BinOp::Div => "÷",
            BinOp::Mod => "%",
            BinOp::And => "AND",
            BinOp::Or => "OR",
            BinOp::Xor => "XOR",
            BinOp::Nand => "NAND",
            BinOp::Nor => "NOR",
            BinOp::Lsh => "<<",
            BinOp::Rsh => ">>",
        }
    }
}

/// 把程序员运算符交给共享求值器：全部左结合，优先级仿 C。
impl Operator<u64> for BinOp {
    fn precedence(self) -> u8 {
        match self {
            BinOp::Mul | BinOp::Div | BinOp::Mod => 6,
            BinOp::Add | BinOp::Sub => 5,
            BinOp::Lsh | BinOp::Rsh => 4,
            BinOp::And | BinOp::Nand => 3,
            BinOp::Xor | BinOp::Nor => 2,
            BinOp::Or => 1,
        }
    }

    fn assoc(self) -> Assoc {
        Assoc::Left
    }

    fn apply(self, a: u64, b: u64) -> Result<u64, CalcError> {
        match self {
            BinOp::Add => Ok(a.wrapping_add(b)),
            BinOp::Sub => Ok(a.wrapping_sub(b)),
            BinOp::Mul => Ok(a.wrapping_mul(b)),
            BinOp::Div => a.checked_div(b).ok_or(CalcError::DivideByZero),
            BinOp::Mod => a.checked_rem(b).ok_or(CalcError::DivideByZero),
            BinOp::And => Ok(a & b),
            BinOp::Or => Ok(a | b),
            BinOp::Xor => Ok(a ^ b),
            BinOp::Nand => Ok(!(a & b)),
            BinOp::Nor => Ok(!(a | b)),
            BinOp::Lsh => Ok(a.wrapping_shl(b as u32)),
            BinOp::Rsh => Ok(a.wrapping_shr(b as u32)),
        }
    }
}

// ---------------------------------------------------------------------------
// 表达式
// ---------------------------------------------------------------------------

/// 程序员模式算式里的一个片段。
#[derive(Clone, Debug, PartialEq)]
pub enum PToken {
    Num(u64),
    Op(BinOp),
    LParen,
    RParen,
}

/// 括号深度（左括号数 − 右括号数）。
fn paren_depth(tokens: &[PToken]) -> usize {
    let mut depth = 0usize;
    for token in tokens {
        match token {
            PToken::LParen => depth += 1,
            PToken::RParen => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    depth
}

/// 把 token 序列拼成算式，数字按当前进制显示。
fn tokens_to_string(tokens: &[PToken], base: Base, word: WordSize) -> String {
    tokens
        .iter()
        .map(|token| match token {
            PToken::Num(v) => format_value(base, *v, word),
            PToken::Op(op) => op.symbol().to_string(),
            PToken::LParen => "(".to_string(),
            PToken::RParen => ")".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 程序员模式的语言：数字、括号。
pub struct ProgrammerLanguage;

impl Language for ProgrammerLanguage {
    type Token = PToken;
    type Value = u64;
    type Op = BinOp;

    fn is_lparen(&self, token: &PToken) -> bool {
        matches!(token, PToken::LParen)
    }

    fn is_rparen(&self, token: &PToken) -> bool {
        matches!(token, PToken::RParen)
    }

    fn operator(&self, token: &PToken) -> Option<BinOp> {
        match token {
            PToken::Op(op) => Some(*op),
            _ => None,
        }
    }

    fn atom(&self, token: &PToken) -> Option<u64> {
        match token {
            PToken::Num(value) => Some(*value),
            _ => None,
        }
    }

    /// 程序员模式容错：缺操作数时按 0 处理，避免整条算式失效。
    fn missing_operand(&self) -> Result<u64, CalcError> {
        Ok(0)
    }
}

/// 求值一条程序员模式算式。
fn evaluate(tokens: &[PToken]) -> Result<u64, CalcError> {
    eval::evaluate(ProgrammerLanguage, tokens)
}

/// 把原始位模式格式化成适合显示的字符串。
pub fn format_value(base: Base, value: u64, word: WordSize) -> String {
    let value = value & word.mask();
    match base {
        Base::Hex => group_digits(format!("{value:X}"), 4),
        Base::Dec => signed_value(value, word).to_string(),
        Base::Oct => group_digits(format!("{value:o}"), 3),
        Base::Bin => group_digits(format!("{value:b}"), 4),
    }
}

/// 按补码把位模式解释成有符号整数。
fn signed_value(value: u64, word: WordSize) -> i64 {
    let bits = word.bits();
    let value = value & word.mask();
    if bits >= 64 {
        value as i64
    } else {
        let shift = 64 - bits;
        ((value << shift) as i64) >> shift
    }
}

/// 从右往左每 `size` 位加一个空格：`FFFF FFFF`、`1111 0000`。
fn group_digits(digits: String, size: usize) -> String {
    let chars: Vec<char> = digits.chars().collect();
    let mut out = String::with_capacity(chars.len() + chars.len() / size);
    for (index, ch) in chars.iter().enumerate() {
        out.push(*ch);
        let remaining = chars.len() - index - 1;
        if remaining > 0 && remaining.is_multiple_of(size) {
            out.push(' ');
        }
    }
    out
}

// ---------------------------------------------------------------------------
// 状态
// ---------------------------------------------------------------------------

/// 「程序员」模式的状态。
pub struct ProgrammerCalc {
    /// 当前输入 / 显示的进制。
    pub base: Base,
    /// 位宽。
    pub word: WordSize,
    /// 当前显示的位模式（已按位宽截断）。
    value: u64,
    /// 已确认的算式片段。
    tokens: Vec<PToken>,
    /// 显示屏上方的小号算式。
    expression: String,
    /// 为 true 表示下一次输入数字要开启新的操作数。
    waiting_for_operand: bool,
    /// 连续按 = 时复用的运算符。
    last_op: Option<BinOp>,
    /// 连续按 = 时复用的右操作数。
    last_operand: Option<u64>,
    /// 当前错误（例如除以零）。
    error: Option<CalcError>,
    /// 记忆槽（最新在末尾）。
    pub memory: Memory<u64>,
    /// 键盘 / 位翻转面板的切换：true 表示显示位翻转。
    pub show_bits: bool,
}

impl ProgrammerCalc {
    pub fn new() -> Self {
        Self {
            base: Base::Dec,
            word: WordSize::Qword,
            value: 0,
            tokens: Vec::new(),
            expression: String::new(),
            waiting_for_operand: true,
            last_op: None,
            last_operand: None,
            error: None,
            memory: Memory::new(),
            show_bits: false,
        }
    }

    /// 显示屏上的主数字。
    pub fn display(&self) -> String {
        match self.error {
            Some(error) => error.message().to_string(),
            None => format_value(self.base, self.value, self.word),
        }
    }

    /// 显示屏上方的小号算式。
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// 当前位模式在指定进制下的文本（用于左侧进制列表）。
    pub fn value_in(&self, base: Base) -> String {
        format_value(base, self.value, self.word)
    }

    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// 当前显示值的原始位模式。
    pub fn value(&self) -> u64 {
        self.value
    }

    fn mask(&self, value: u64) -> u64 {
        value & self.word.mask()
    }

    fn clear_repeat(&mut self) {
        self.last_op = None;
        self.last_operand = None;
    }

    // ---- 进制 / 位宽 ----

    pub fn set_base(&mut self, base: Base) {
        self.base = base;
    }

    /// 切换位宽：保留低位，超出部分截断。
    pub fn set_word(&mut self, word: WordSize) {
        self.word = word;
        self.value &= word.mask();
        if let Some(operand) = self.last_operand.as_mut() {
            *operand &= word.mask();
        }
    }

    /// 在键盘面板与位翻转面板之间切换。
    pub fn set_show_bits(&mut self, show: bool) {
        self.show_bits = show;
    }

    // ---- 输入 ----

    /// 输入一位数字（0–9 / A–F），不在当前进制里的会被忽略。
    pub fn input_digit(&mut self, ch: char) {
        let Some(digit) = self.base.digit(ch) else {
            return;
        };

        if self.error.is_some() {
            self.clear();
        }
        self.clear_repeat();

        if self.waiting_for_operand {
            // 刚按过运算符或 =：开启新的操作数。
            if self.tokens.is_empty() {
                self.expression.clear();
            }
            self.value = 0;
            self.waiting_for_operand = false;
        }

        let radix = self.base.radix();
        self.value = self.mask(self.value.wrapping_mul(radix).wrapping_add(digit));
    }

    /// 清除当前输入（CE）：保留已确认的算式。
    pub fn clear_entry(&mut self) {
        self.clear_repeat();
        self.value = 0;
        self.waiting_for_operand = true;
        self.error = None;
    }

    /// 全部清除（C）。
    pub fn clear(&mut self) {
        self.value = 0;
        self.tokens.clear();
        self.expression.clear();
        self.waiting_for_operand = true;
        self.error = None;
        self.clear_repeat();
    }

    /// 退格：按当前进制去掉最后一位。
    pub fn backspace(&mut self) {
        self.clear_repeat();
        if self.waiting_for_operand {
            return;
        }
        self.value /= self.base.radix();
    }

    /// 取相反数（按当前位宽求补码）。
    pub fn negate(&mut self) {
        self.clear_repeat();
        self.value = self.mask(self.value.wrapping_neg());
        self.waiting_for_operand = false;
        self.error = None;
    }

    /// 按位取反（NOT）。
    pub fn bitwise_not(&mut self) {
        self.clear_repeat();
        self.value = self.mask(!self.value);
        self.waiting_for_operand = false;
        self.error = None;
    }

    /// 翻转某一位。
    pub fn toggle_bit(&mut self, bit: u32) {
        if self.error.is_some() {
            self.clear();
        }
        self.clear_repeat();
        self.value = self.mask(self.value ^ (1u64 << bit));
        self.waiting_for_operand = false;
    }

    // ---- 运算 ----

    /// 按下一个二元运算符。
    pub fn set_operator(&mut self, op: BinOp) {
        self.error = None;
        self.clear_repeat();

        if self.waiting_for_operand {
            match self.tokens.last() {
                // 连按运算符：替换掉末尾那个。
                Some(PToken::Op(_)) => {
                    *self.tokens.last_mut().expect("已判断末尾是运算符") = PToken::Op(op);
                }
                // 左括号后不能直接跟运算符。
                Some(PToken::LParen) => return,
                // 右括号或空算式：直接追加运算符。
                Some(PToken::RParen) => self.tokens.push(PToken::Op(op)),
                _ => {
                    self.tokens.push(PToken::Num(self.value));
                    self.tokens.push(PToken::Op(op));
                }
            }
        } else {
            self.tokens.push(PToken::Num(self.value));
            self.tokens.push(PToken::Op(op));
        }

        self.expression = tokens_to_string(&self.tokens, self.base, self.word);
        self.waiting_for_operand = true;
    }

    /// 左括号。数字后面直接按 `(` 时补一个乘号（`2(3+4)`）。
    pub fn open_paren(&mut self) {
        if self.error.is_some() {
            self.clear();
        }
        self.clear_repeat();

        if !self.waiting_for_operand {
            self.tokens.push(PToken::Num(self.value));
            self.tokens.push(PToken::Op(BinOp::Mul));
        }

        self.tokens.push(PToken::LParen);
        self.expression = tokens_to_string(&self.tokens, self.base, self.word);
        self.value = 0;
        self.waiting_for_operand = true;
    }

    /// 右括号。没有未闭合的左括号时忽略。
    pub fn close_paren(&mut self) {
        if paren_depth(&self.tokens) == 0 {
            return;
        }
        self.clear_repeat();

        // 括号内末尾悬空的运算符。
        if !self.waiting_for_operand || matches!(self.tokens.last(), Some(PToken::Op(_))) {
            self.tokens.push(PToken::Num(self.value));
        }
        self.tokens.push(PToken::RParen);
        self.expression = tokens_to_string(&self.tokens, self.base, self.word);
        self.waiting_for_operand = true;
    }

    /// 按下等号：对整条算式求值。
    pub fn equals(&mut self) {
        if self.error.is_some() {
            return;
        }

        // 没有算式：可能是连续按 =，也可能只是给单个数字加等号。
        if self.tokens.is_empty() {
            if let (Some(op), Some(operand)) = (self.last_op, self.last_operand) {
                let left = self.value;
                self.expression = format!(
                    "{} {} {} =",
                    format_value(self.base, left, self.word),
                    op.symbol(),
                    format_value(self.base, operand, self.word)
                );
                match op.apply(left, operand) {
                    Ok(result) => self.value = self.mask(result),
                    Err(error) => self.error = Some(error),
                }
            } else {
                self.expression = format!("{} =", self.display());
            }
            self.waiting_for_operand = true;
            return;
        }

        let mut tokens = self.tokens.clone();
        let last_number = tokens.iter().rev().find_map(|token| match token {
            PToken::Num(v) => Some(*v),
            _ => None,
        });

        if self.waiting_for_operand {
            // 末尾运算符悬空时复用最后一个数字，`7 + =` → 14。
            if matches!(tokens.last(), Some(PToken::Op(_))) {
                tokens.push(PToken::Num(last_number.unwrap_or(self.value)));
            }
        } else {
            tokens.push(PToken::Num(self.value));
        }

        // 自动补齐没有闭合的括号。
        let mut depth = paren_depth(&tokens);
        while depth > 0 {
            tokens.push(PToken::RParen);
            depth -= 1;
        }

        let last_op = tokens.iter().rev().find_map(|token| match token {
            PToken::Op(op) => Some(*op),
            _ => None,
        });
        let last_operand = tokens.iter().rev().find_map(|token| match token {
            PToken::Num(v) => Some(*v),
            _ => None,
        });

        match evaluate(&tokens) {
            Ok(result) => {
                self.value = self.mask(result);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }

        self.expression = format!("{} =", tokens_to_string(&tokens, self.base, self.word));
        self.tokens.clear();
        self.last_op = last_op;
        self.last_operand = last_operand;
        self.waiting_for_operand = true;
    }

    // ---- 记忆 ----

    pub fn memory_store(&mut self) {
        self.memory.store(self.value);
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
        }
    }

    /// M+：累加后按当前位宽截断。
    pub fn memory_add(&mut self) {
        let value = self.value;
        let mask = self.word.mask();
        self.memory
            .update_latest(value, |last| last.wrapping_add(value) & mask);
    }

    /// M−：没有记忆槽时存的是 `-value` 的位模式。
    pub fn memory_subtract(&mut self) {
        let value = self.value;
        let mask = self.word.mask();
        self.memory
            .update_latest(value.wrapping_neg() & mask, |last| {
                last.wrapping_sub(value) & mask
            });
    }

    fn load_value(&mut self, value: u64) {
        self.clear();
        self.value = self.mask(value);
        self.waiting_for_operand = true;
    }

    // ---- 键盘 ----

    pub fn handle_key(&mut self, keystroke: &Keystroke) {
        let modifiers = &keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform {
            return;
        }

        let typed = keystroke.key_char.as_deref().unwrap_or("");
        let mut iter = typed.chars();
        if let (Some(c), None) = (iter.next(), iter.next()) {
            match c {
                '0'..='9' => self.input_digit(c),
                'a'..='f' | 'A'..='F' => self.input_digit(c),
                '+' => self.set_operator(BinOp::Add),
                '-' => self.set_operator(BinOp::Sub),
                '*' | 'x' | 'X' => self.set_operator(BinOp::Mul),
                '/' => self.set_operator(BinOp::Div),
                '%' => self.set_operator(BinOp::Mod),
                '&' => self.set_operator(BinOp::And),
                '|' => self.set_operator(BinOp::Or),
                '^' => self.set_operator(BinOp::Xor),
                '~' => self.bitwise_not(),
                '(' => self.open_paren(),
                ')' => self.close_paren(),
                '<' => self.set_operator(BinOp::Lsh),
                '>' => self.set_operator(BinOp::Rsh),
                '=' => self.equals(),
                _ => {}
            }
            return;
        }

        match keystroke.key.as_str() {
            "enter" => self.equals(),
            "escape" => self.clear(),
            "delete" => self.clear_entry(),
            "backspace" => self.backspace(),
            _ => {}
        }
    }

    /// 显示屏文本（Ctrl+C）。
    pub fn copy_text(&self) -> String {
        self.display()
    }

    /// 从剪贴板粘贴一个数（Ctrl+V），按当前进制解析。
    pub fn paste(&mut self, text: &str) {
        let cleaned: String = text
            .chars()
            .filter(|c| !c.is_whitespace() && *c != ',' && *c != '_')
            .collect();
        let radix = self.base.radix() as u32;
        let parsed = u64::from_str_radix(&cleaned, radix).or_else(|_| cleaned.parse::<u64>());
        if let Ok(value) = parsed {
            if self.error.is_some() {
                self.clear();
            }
            self.clear_repeat();
            self.error = None;
            self.value = self.mask(value);
            if self.waiting_for_operand && self.tokens.is_empty() {
                self.expression.clear();
            }
            self.waiting_for_operand = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Base, BinOp, PToken, ProgrammerCalc, WordSize, evaluate, format_value};
    use gpui_kit::{Keystroke, Modifiers};

    fn num(v: u64) -> PToken {
        PToken::Num(v)
    }

    fn op(op: BinOp) -> PToken {
        PToken::Op(op)
    }

    fn key(key: &str, key_char: Option<&str>) -> Keystroke {
        Keystroke {
            modifiers: Modifiers::default(),
            key: key.to_string(),
            key_char: key_char.map(|s| s.to_string()),
        }
    }

    #[test]
    fn formats_each_base() {
        let value = 0xDEAD_BEEFu64;
        assert_eq!(format_value(Base::Hex, value, WordSize::Qword), "DEAD BEEF");
        assert_eq!(
            format_value(Base::Dec, value, WordSize::Qword),
            "3735928559"
        );
        assert_eq!(
            format_value(Base::Oct, value, WordSize::Qword),
            "33 653 337 357"
        );
        assert_eq!(
            format_value(Base::Bin, value, WordSize::Qword),
            "1101 1110 1010 1101 1011 1110 1110 1111"
        );
    }

    #[test]
    fn decimal_is_signed_per_word_size() {
        assert_eq!(format_value(Base::Dec, 0xFF, WordSize::Byte), "-1");
        assert_eq!(format_value(Base::Dec, 0x7F, WordSize::Byte), "127");
        assert_eq!(format_value(Base::Hex, 0x1FF, WordSize::Byte), "FF");
    }

    #[test]
    fn precedence_follows_c_like_rules() {
        // 2 + 3 × 4 = 14
        let tokens = [num(2), op(BinOp::Add), num(3), op(BinOp::Mul), num(4)];
        assert_eq!(evaluate(&tokens), Ok(14));
    }

    #[test]
    fn bitwise_and_shift() {
        // 12 AND 10 = 8
        let tokens = [num(12), op(BinOp::And), num(10)];
        assert_eq!(evaluate(&tokens), Ok(8));

        // 1 << 4 = 16
        let tokens = [num(1), op(BinOp::Lsh), num(4)];
        assert_eq!(evaluate(&tokens), Ok(16));
    }

    #[test]
    fn parentheses_override_precedence() {
        // (2 + 3) × 4 = 20
        let tokens = [
            PToken::LParen,
            num(2),
            op(BinOp::Add),
            num(3),
            PToken::RParen,
            op(BinOp::Mul),
            num(4),
        ];
        assert_eq!(evaluate(&tokens), Ok(20));
    }

    #[test]
    fn division_by_zero_is_an_error() {
        let tokens = [num(1), op(BinOp::Div), num(0)];
        assert!(evaluate(&tokens).is_err());
    }

    #[test]
    fn hex_input_only_accepts_hex_digits() {
        let mut calc = ProgrammerCalc::new();
        calc.set_base(Base::Hex);
        calc.input_digit('A');
        calc.input_digit('F');
        assert_eq!(calc.display(), "AF");

        calc.clear();
        calc.set_base(Base::Dec);
        calc.input_digit('A'); // 十进制下忽略
        assert_eq!(calc.display(), "0");
    }

    #[test]
    fn word_size_truncates_result() {
        let mut calc = ProgrammerCalc::new();
        calc.set_word(WordSize::Byte);
        calc.set_base(Base::Hex);
        calc.input_digit('F');
        calc.input_digit('F');
        calc.set_operator(BinOp::Add);
        calc.input_digit('1');
        calc.equals();
        assert_eq!(calc.display(), "0");
    }

    #[test]
    fn equals_uses_precedence_and_records_expression() {
        let mut calc = ProgrammerCalc::new();
        calc.input_digit('2');
        calc.set_operator(BinOp::Add);
        calc.input_digit('3');
        calc.set_operator(BinOp::Mul);
        calc.input_digit('4');
        calc.equals();
        assert_eq!(calc.value(), 14);
        assert_eq!(calc.expression(), "2 + 3 × 4 =");
    }

    #[test]
    fn consecutive_equals_repeats_operation() {
        let mut calc = ProgrammerCalc::new();
        calc.input_digit('2');
        calc.set_operator(BinOp::Add);
        calc.input_digit('3');
        calc.equals();
        assert_eq!(calc.value(), 5);
        calc.equals();
        assert_eq!(calc.value(), 8);
    }

    #[test]
    fn parentheses_work_through_the_state_machine() {
        let mut calc = ProgrammerCalc::new();
        calc.open_paren();
        calc.input_digit('2');
        calc.set_operator(BinOp::Add);
        calc.input_digit('3');
        calc.close_paren();
        calc.set_operator(BinOp::Mul);
        calc.input_digit('4');
        calc.equals();
        assert_eq!(calc.value(), 20);
    }

    #[test]
    fn open_paren_after_number_inserts_multiply() {
        let mut calc = ProgrammerCalc::new();
        calc.input_digit('2');
        calc.open_paren();
        calc.input_digit('3');
        calc.set_operator(BinOp::Add);
        calc.input_digit('4');
        calc.close_paren();
        calc.equals();
        assert_eq!(calc.value(), 14);
    }

    #[test]
    fn negate_uses_twos_complement() {
        let mut calc = ProgrammerCalc::new();
        calc.set_word(WordSize::Byte);
        calc.input_digit('5');
        calc.negate();
        assert_eq!(calc.display(), "-5");
        assert_eq!(calc.value_in(Base::Hex), "FB");
    }

    #[test]
    fn backspace_removes_one_digit_in_current_base() {
        let mut calc = ProgrammerCalc::new();
        calc.set_base(Base::Hex);
        calc.input_digit('1');
        calc.input_digit('2');
        calc.input_digit('3');
        calc.backspace();
        assert_eq!(calc.display(), "12");
    }

    #[test]
    fn keyboard_accepts_hex_letters() {
        let mut calc = ProgrammerCalc::new();
        calc.set_base(Base::Hex);
        for ch in ["A", "B", "C"] {
            calc.handle_key(&key(ch, Some(ch)));
        }
        assert_eq!(calc.display(), "ABC");
    }

    #[test]
    fn memory_round_trip() {
        let mut calc = ProgrammerCalc::new();
        calc.input_digit('7');
        calc.memory_store();
        calc.clear();
        calc.memory_recall();
        assert_eq!(calc.value(), 7);

        calc.memory_add();
        assert_eq!(calc.memory.to_vec(), vec![14]);
        calc.memory_clear();
        assert!(calc.memory.is_empty());
    }

    #[test]
    fn toggle_bit_flips_only_the_target_bit() {
        let mut calc = ProgrammerCalc::new();
        calc.toggle_bit(3);
        assert_eq!(calc.value(), 8);
        calc.toggle_bit(3);
        assert_eq!(calc.value(), 0);
    }
}
