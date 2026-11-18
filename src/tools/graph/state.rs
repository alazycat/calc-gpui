//! 绘图状态：表达式列表、当前编辑的表达式、视图切换、函数面板与视口范围。
//!
//! 一个绘图页面有 **多条表达式**：它们各自有输入文本、光标、解析结果和可见性，
//! 而视口（缩放/平移）是共用的。同一时刻只有一条表达式在被编辑（`active`），
//! 键盘和下面的数学键盘都作用在它身上。

use super::expr::{Plot, parse_plot};

/// 绘图页面的两种视图。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphView {
    /// 坐标系与曲线。
    Graph,
    /// `f(x)` 表达式列表。
    Equations,
}

/// 表达式面板的三个下拉页签。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteTab {
    Trig,
    Inequality,
    Functions,
}

/// 面板里的一个函数键。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaletteEntry {
    /// 按键上的文字。
    pub label: &'static str,
    /// 插入表达式的文本。
    pub insert: &'static str,
}

impl PaletteTab {
    pub const ALL: [PaletteTab; 3] = [
        PaletteTab::Trig,
        PaletteTab::Inequality,
        PaletteTab::Functions,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PaletteTab::Trig => "三角学",
            PaletteTab::Inequality => "不等式",
            PaletteTab::Functions => "函数",
        }
    }

    /// 这一页的按键；`second` 为真时三角学换成反函数。
    pub fn entries(self, second: bool) -> &'static [PaletteEntry] {
        match (self, second) {
            (PaletteTab::Trig, false) => &[
                PaletteEntry {
                    label: "sin",
                    insert: "sin(",
                },
                PaletteEntry {
                    label: "cos",
                    insert: "cos(",
                },
                PaletteEntry {
                    label: "tan",
                    insert: "tan(",
                },
                PaletteEntry {
                    label: "sinh",
                    insert: "sinh(",
                },
                PaletteEntry {
                    label: "cosh",
                    insert: "cosh(",
                },
                PaletteEntry {
                    label: "tanh",
                    insert: "tanh(",
                },
            ],
            (PaletteTab::Trig, true) => &[
                PaletteEntry {
                    label: "sin⁻¹",
                    insert: "asin(",
                },
                PaletteEntry {
                    label: "cos⁻¹",
                    insert: "acos(",
                },
                PaletteEntry {
                    label: "tan⁻¹",
                    insert: "atan(",
                },
                PaletteEntry {
                    label: "sinh⁻¹",
                    insert: "asinh(",
                },
                PaletteEntry {
                    label: "cosh⁻¹",
                    insert: "acosh(",
                },
                PaletteEntry {
                    label: "tanh⁻¹",
                    insert: "atanh(",
                },
            ],
            (PaletteTab::Inequality, _) => &[
                PaletteEntry {
                    label: "y >",
                    insert: "y > ",
                },
                PaletteEntry {
                    label: "y ≥",
                    insert: "y ≥ ",
                },
                PaletteEntry {
                    label: "y <",
                    insert: "y < ",
                },
                PaletteEntry {
                    label: "y ≤",
                    insert: "y ≤ ",
                },
            ],
            (PaletteTab::Functions, _) => &[
                PaletteEntry {
                    label: "|x|",
                    insert: "abs(",
                },
                PaletteEntry {
                    label: "√",
                    insert: "sqrt(",
                },
                PaletteEntry {
                    label: "∛",
                    insert: "cbrt(",
                },
                PaletteEntry {
                    label: "eˣ",
                    insert: "exp(",
                },
                PaletteEntry {
                    label: "ln",
                    insert: "ln(",
                },
                PaletteEntry {
                    label: "log",
                    insert: "log(",
                },
                PaletteEntry {
                    label: "⌊x⌋",
                    insert: "floor(",
                },
                PaletteEntry {
                    label: "⌈x⌉",
                    insert: "ceil(",
                },
                PaletteEntry {
                    label: "round",
                    insert: "round(",
                },
            ],
        }
    }
}

/// 绘图输入框可执行的语义动作（由 UI 层从按键翻译而来）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    /// 输入一个字符（表达式是文本，任意可见字符都交给 [`GraphState::input_char`] 过滤）。
    Input(char),
    Backspace,
    DeleteForward,
    /// 光标移动 `i32` 格，负数向左、正数向右。
    MoveCursor(i32),
    Home,
    End,
    /// 解析并绘制（按 Enter / 绘图按钮）。
    Plot,
    Clear,
    /// 数学键盘插入一段模板文本（例如 `^2`、`sqrt(`）。
    Insert(&'static str),
    /// `2ⁿᵈ`：切换三角函数的反函数与几个按键的幂次。
    Second,
    /// 在「坐标系」和「表达式列表」之间切换。
    ToggleView,
    /// 显示指定的视图。
    ShowView(GraphView),
    /// 打开 / 关闭某个函数面板（再点一次同一个页签即关闭）。
    TogglePalette(PaletteTab),
    AddEquation,
    RemoveEquation(usize),
    SelectEquation(usize),
    ToggleVisible(usize),
}

/// 一条表达式。
pub struct Equation {
    input: String,
    /// 光标位置（字符下标）。
    pub(super) cursor: usize,
    /// 最近一次成功解析的结果。
    pub(super) plot: Option<Plot>,
    /// 解析错误（按 Enter / 绘图时显示）。
    pub(super) error: Option<String>,
    /// 是否画在坐标系里。
    pub(super) visible: bool,
}

impl Equation {
    fn blank() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
            plot: None,
            error: None,
            visible: true,
        }
    }

    fn from_text(text: &str) -> Self {
        let mut equation = Self::blank();
        equation.input = text.to_string();
        equation.cursor = text.chars().count();
        equation.plot = parse_plot(text).ok();
        equation
    }

    pub fn text(&self) -> &str {
        &self.input
    }

    pub fn chars(&self) -> Vec<char> {
        self.input.chars().collect()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn plot(&self) -> Option<&Plot> {
        self.plot.as_ref()
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn is_empty(&self) -> bool {
        self.input.trim().is_empty()
    }

    /// 输入变化时静默尝试重新解析：解析失败时保留上一条曲线。
    fn refresh(&mut self) {
        self.error = None;
        if let Ok(plot) = parse_plot(&self.input) {
            self.plot = Some(plot);
        }
    }

    fn set_chars(&mut self, chars: Vec<char>) {
        self.input = chars.into_iter().collect();
        self.refresh();
    }

    fn insert_text(&mut self, text: &str) {
        let mut chars = self.chars();
        let mut position = self.cursor.min(chars.len());
        for ch in text.chars() {
            chars.insert(position, ch);
            position += 1;
        }
        self.cursor = position;
        self.set_chars(chars);
    }

    fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.plot = None;
        self.error = None;
    }
}

/// 绘图模式状态。
pub struct GraphState {
    equations: Vec<Equation>,
    /// 正在编辑的表达式下标。
    active: usize,
    view: GraphView,
    second: bool,
    palette: Option<PaletteTab>,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl GraphState {
    pub fn new() -> Self {
        Self {
            equations: vec![Equation::from_text("sin(x)")],
            active: 0,
            view: GraphView::Graph,
            second: false,
            palette: None,
            x_min: -10.0,
            x_max: 10.0,
            y_min: -7.0,
            y_max: 7.0,
        }
    }

    // ---- 表达式列表 ----

    pub fn equations(&self) -> &[Equation] {
        &self.equations
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn active(&self) -> &Equation {
        &self.equations[self.active]
    }

    fn active_mut(&mut self) -> &mut Equation {
        let index = self.active;
        &mut self.equations[index]
    }

    /// 正在编辑的表达式（公开给同模块的 view / 测试）。
    pub(super) fn active_plot(&self) -> Option<&Plot> {
        self.equations[self.active].plot()
    }

    pub(super) fn cursor(&self) -> usize {
        self.equations[self.active].cursor
    }

    /// 当前需要画的曲线：`(表达式, 颜色序号)`，颜色序号就是表达式下标。
    pub fn visible_plots(&self) -> Vec<(Plot, usize)> {
        self.equations
            .iter()
            .enumerate()
            .filter(|(_, equation)| equation.visible)
            .filter_map(|(index, equation)| equation.plot.clone().map(|plot| (plot, index)))
            .collect()
    }

    pub fn add_equation(&mut self) {
        self.equations.push(Equation::blank());
        self.active = self.equations.len() - 1;
        self.view = GraphView::Graph;
        self.palette = None;
    }

    pub fn remove_equation(&mut self, index: usize) {
        if index >= self.equations.len() {
            return;
        }
        self.equations.remove(index);
        if self.equations.is_empty() {
            self.equations.push(Equation::blank());
        }
        // 删掉的是当前表达式之前的，活动下标要跟着往前挪一格。
        if index < self.active {
            self.active -= 1;
        }
        if self.active >= self.equations.len() {
            self.active = self.equations.len() - 1;
        }
    }

    pub fn select_equation(&mut self, index: usize) {
        if index < self.equations.len() {
            self.active = index;
            self.view = GraphView::Graph;
            self.palette = None;
            self.refresh_active_display();
        }
    }

    pub fn toggle_visible(&mut self, index: usize) {
        if let Some(equation) = self.equations.get_mut(index) {
            equation.visible = !equation.visible;
        }
    }

    /// 切换活动表达式时清掉它残留的错误，避免状态行显示别的表达式的错误。
    fn refresh_active_display(&mut self) {
        self.active_mut().refresh();
    }

    // ---- 视图与面板 ----

    pub fn view(&self) -> GraphView {
        self.view
    }

    pub fn set_view(&mut self, view: GraphView) {
        self.view = view;
        self.palette = None;
    }

    pub fn toggle_view(&mut self) {
        self.view = match self.view {
            GraphView::Graph => GraphView::Equations,
            GraphView::Equations => GraphView::Graph,
        };
        self.palette = None;
    }

    pub fn second(&self) -> bool {
        self.second
    }

    pub fn toggle_second(&mut self) {
        self.second = !self.second;
    }

    pub fn palette(&self) -> Option<PaletteTab> {
        self.palette
    }

    pub fn toggle_palette(&mut self, tab: PaletteTab) {
        self.palette = if self.palette == Some(tab) {
            None
        } else {
            Some(tab)
        };
    }

    // ---- 文本编辑（作用于活动表达式）----

    pub fn error(&self) -> Option<&str> {
        self.active().error()
    }

    pub fn copy_text(&self) -> String {
        self.active().text().to_string()
    }

    pub fn chars(&self) -> Vec<char> {
        self.active().chars()
    }

    pub fn paste(&mut self, text: &str) {
        let text = text.trim().to_string();
        let equation = self.active_mut();
        equation.input = text;
        equation.cursor = equation.input.chars().count();
        equation.refresh();
    }

    /// 数学键盘：在光标处插入一段模板文本。
    pub fn insert(&mut self, text: &str) {
        self.active_mut().insert_text(text);
    }

    /// 按 Enter / 绘图：解析并报告错误。
    pub fn plot(&mut self) {
        let input = self.active().text().to_string();
        let equation = self.active_mut();
        match parse_plot(&input) {
            Ok(plot) => {
                equation.plot = Some(plot);
                equation.error = None;
            }
            Err(message) => equation.error = Some(message),
        }
    }

    pub fn input_char(&mut self, ch: char) {
        // 只挡控制字符：`≤`、`π` 这些非 ASCII 字符也要能敲进去。
        if ch.is_control() {
            return;
        }
        self.insert(&ch.to_string());
    }

    pub fn backspace(&mut self) {
        let equation = self.active_mut();
        if equation.cursor == 0 {
            return;
        }
        let mut chars = equation.chars();
        chars.remove(equation.cursor - 1);
        equation.cursor -= 1;
        equation.set_chars(chars);
    }

    pub fn delete_forward(&mut self) {
        let equation = self.active_mut();
        let mut chars = equation.chars();
        if equation.cursor < chars.len() {
            chars.remove(equation.cursor);
            equation.set_chars(chars);
        }
    }

    pub fn move_cursor(&mut self, delta: i32) {
        let equation = self.active_mut();
        let len = equation.input.chars().count();
        equation.cursor = (equation.cursor as i32 + delta).clamp(0, len as i32) as usize;
    }

    pub fn move_home(&mut self) {
        self.active_mut().cursor = 0;
    }

    pub fn move_end(&mut self) {
        let equation = self.active_mut();
        equation.cursor = equation.input.chars().count();
    }

    pub fn clear(&mut self) {
        self.active_mut().clear();
    }

    // ---- 视口 ----

    pub fn zoom(&mut self, factor: f64) {
        let center_x = (self.x_min + self.x_max) / 2.0;
        let center_y = (self.y_min + self.y_max) / 2.0;
        let half_x = (self.x_max - self.x_min) / 2.0 * factor;
        let half_y = (self.y_max - self.y_min) / 2.0 * factor;
        self.x_min = center_x - half_x;
        self.x_max = center_x + half_x;
        self.y_min = center_y - half_y;
        self.y_max = center_y + half_y;
    }

    /// 平移视图，`dx` / `dy` 是横纵跨度的比例。
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let shift_x = (self.x_max - self.x_min) * dx;
        let shift_y = (self.y_max - self.y_min) * dy;
        self.x_min += shift_x;
        self.x_max += shift_x;
        self.y_min += shift_y;
        self.y_max += shift_y;
    }

    pub fn reset_view(&mut self) {
        self.x_min = -10.0;
        self.x_max = 10.0;
        self.y_min = -7.0;
        self.y_max = 7.0;
    }

    /// 根据所有可见曲线在 x 范围内的取值自动适配 y 轴。
    pub fn fit_y(&mut self) {
        let plots = self.visible_plots();
        if plots.is_empty() {
            return;
        }
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for (plot, _) in &plots {
            for i in 0..=400 {
                let x = self.x_min + (self.x_max - self.x_min) * i as f64 / 400.0;
                let y = plot.eval(x);
                // 忽略爆炸性的值（例如 tan 的渐近线附近）。
                if y.is_finite() && y.abs() <= 1e6 {
                    low = low.min(y);
                    high = high.max(y);
                }
            }
        }
        if low.is_finite() && high.is_finite() {
            let span = (high - low).abs();
            let pad = if span < 1e-9 { 1.0 } else { span * 0.1 };
            self.y_min = low - pad;
            self.y_max = high + pad;
        }
    }

    /// 分发一个语义动作；按键到动作的翻译见 [`crate::tools::graph::keystroke_to_action`]。
    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::Input(ch) => self.input_char(ch),
            Action::Backspace => self.backspace(),
            Action::DeleteForward => self.delete_forward(),
            Action::MoveCursor(delta) => self.move_cursor(delta),
            Action::Home => self.move_home(),
            Action::End => self.move_end(),
            Action::Plot => self.plot(),
            Action::Clear => self.clear(),
            Action::Insert(text) => self.insert(text),
            Action::Second => self.toggle_second(),
            Action::ToggleView => self.toggle_view(),
            Action::ShowView(view) => self.set_view(view),
            Action::TogglePalette(tab) => self.toggle_palette(tab),
            Action::AddEquation => self.add_equation(),
            Action::RemoveEquation(index) => self.remove_equation(index),
            Action::SelectEquation(index) => self.select_equation(index),
            Action::ToggleVisible(index) => self.toggle_visible(index),
        }
    }
}

impl Default for GraphState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, GraphState, GraphView, PaletteTab};

    #[test]
    fn apply_action_dispatches_text_editing() {
        let mut state = GraphState::new();
        state.clear();
        for ch in "2x".chars() {
            state.apply_action(Action::Input(ch));
        }
        assert_eq!(state.copy_text(), "2x");
        assert_eq!(state.cursor(), 2);

        state.apply_action(Action::Backspace);
        assert_eq!(state.copy_text(), "2");

        state.apply_action(Action::Home);
        state.apply_action(Action::Input('9'));
        assert_eq!(state.copy_text(), "92");

        // 光标停在 '9' 与 '2' 之间，向前删除 '2'。
        state.apply_action(Action::DeleteForward);
        assert_eq!(state.copy_text(), "9");
    }

    #[test]
    fn apply_action_end_plot_and_clear() {
        let mut state = GraphState::new();
        state.clear();

        // 在空输入上绘图应报告解析错误。
        state.apply_action(Action::Plot);
        assert!(state.error().is_some());
        assert!(state.active_plot().is_none());

        state.apply_action(Action::Input('x'));
        state.apply_action(Action::End);
        assert_eq!(state.cursor(), 1);

        state.apply_action(Action::Plot);
        assert!(state.error().is_none());
        assert!(state.active_plot().is_some());

        state.apply_action(Action::Clear);
        assert_eq!(state.copy_text(), "");
        assert_eq!(state.cursor(), 0);
        assert!(state.active_plot().is_none());
    }

    #[test]
    fn insert_replaces_nothing_and_keeps_cursor_after_text() {
        let mut state = GraphState::new();
        state.clear();
        state.apply_action(Action::Insert("sqrt("));
        state.apply_action(Action::Insert("x"));
        assert_eq!(state.copy_text(), "sqrt(x");
        assert_eq!(state.cursor(), 6);
    }

    #[test]
    fn multiple_equations_are_independent() {
        let mut state = GraphState::new();
        state.clear();
        state.insert("x");
        state.add_equation();
        assert_eq!(state.equations().len(), 2);
        // 新表达式是空的，且立刻可编辑。
        assert_eq!(state.copy_text(), "");
        assert_eq!(state.view(), GraphView::Graph);

        state.insert("x^2");
        assert_eq!(state.visible_plots().len(), 2);

        // 回到第一条，内容还在。
        state.apply_action(Action::SelectEquation(0));
        assert_eq!(state.copy_text(), "x");

        // 隐藏第一条后只剩一条曲线。
        state.apply_action(Action::ToggleVisible(0));
        assert_eq!(state.visible_plots().len(), 1);

        state.apply_action(Action::RemoveEquation(0));
        assert_eq!(state.equations().len(), 1);
        assert_eq!(state.copy_text(), "x^2");
    }

    #[test]
    fn removing_the_last_equation_leaves_one_blank() {
        let mut state = GraphState::new();
        state.apply_action(Action::RemoveEquation(0));
        assert_eq!(state.equations().len(), 1);
        assert_eq!(state.copy_text(), "");
    }

    /// 删掉当前表达式之前的一条，选中的内容不能跟着索引漂移。
    #[test]
    fn removing_an_earlier_equation_keeps_the_selection() {
        let mut state = GraphState::new();
        state.clear();
        state.insert("x");
        state.add_equation();
        state.insert("x^2");
        state.add_equation();
        state.insert("x^3");
        assert_eq!(state.active_index(), 2);

        state.apply_action(Action::RemoveEquation(0));
        assert_eq!(state.active_index(), 1);
        assert_eq!(state.copy_text(), "x^3");

        state.apply_action(Action::RemoveEquation(0));
        assert_eq!(state.active_index(), 0);
        assert_eq!(state.copy_text(), "x^3");
    }

    #[test]
    fn view_and_palette_toggles() {
        let mut state = GraphState::new();
        assert_eq!(state.view(), GraphView::Graph);

        state.apply_action(Action::ToggleView);
        assert_eq!(state.view(), GraphView::Equations);

        // 切换视图会收起函数面板。
        state.apply_action(Action::TogglePalette(PaletteTab::Trig));
        assert_eq!(state.palette(), Some(PaletteTab::Trig));
        state.apply_action(Action::TogglePalette(PaletteTab::Trig));
        assert_eq!(state.palette(), None);
        state.apply_action(Action::TogglePalette(PaletteTab::Functions));
        state.apply_action(Action::ShowView(GraphView::Graph));
        assert_eq!(state.palette(), None);

        state.apply_action(Action::Second);
        assert!(state.second());
        state.apply_action(Action::Second);
        assert!(!state.second());
    }

    /// 键盘和面板能插入的每一段文本，都必须能组成一个合法表达式。
    ///
    /// 这条约束保证数学键盘不会塞进解析器不认识的字符。
    #[test]
    fn every_palette_entry_can_be_completed_into_a_valid_expression() {
        use super::super::expr::parse_plot;

        for tab in PaletteTab::ALL {
            for second in [false, true] {
                for entry in tab.entries(second) {
                    // 函数类插入的是左括号，补一个 `x)` 收尾；不等式自己就是完整开头。
                    let candidate = match tab {
                        PaletteTab::Inequality => format!("{}2x", entry.insert),
                        _ => format!("{}x)", entry.insert),
                    };
                    assert!(
                        parse_plot(&candidate).is_ok(),
                        "{:?} 的 {:?} 插入后无法解析：{}",
                        tab,
                        entry.label,
                        candidate
                    );
                }
            }
        }
    }
}
