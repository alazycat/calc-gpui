//! 页面分发：把「当前页面」映射成一个统一的页面接口。
//!
//! 这里是 [`Page`] 枚举**唯一**的消费点。新增一种页面只需要三步：
//!
//! 1. 在 [`super::page`] 里加一个 `Page` 变体和对应的 `MENU_PAGES` 项；
//! 2. 给该页面的状态类型写一个 [`PageView`] 实现（通常只有 `render` 是必填的）；
//! 3. 在 [`CalcApp::page_view`] 里加一条 arm。
//!
//! 窗口渲染、键盘分发、剪贴板都只认 [`PageView`]，不再认识任何具体模式。
//! 页面之间的差异（日期页没有键盘处理、程序员模式没有历史面板）由默认实现
//! 表达，而不是让调用方去 `match`。

use super::CalcApp;
use super::page::Page;
use crate::modes::programmer::{self, ProgrammerCalc};
use crate::modes::scientific::{self, ScientificCalc};
use crate::modes::standard::{self, StandardCalc};
use crate::tools::converter::{self, ConverterState};
use crate::tools::date::{self, DateCalc};
use crate::tools::graph::{self, GraphState};
use gpui_kit::*;

/// 一种页面必须能做的事。
///
/// 只有 [`render`](PageView::render) 是必填的；其余方法默认什么都不做。
pub(super) trait PageView {
    /// 画出页面主体。
    fn render(&self, cx: &mut Context<CalcApp>) -> AnyElement;

    /// 处理一次按键。
    fn handle_key(&mut self, _keystroke: &Keystroke) {}

    /// 当前页面的结果文本（Ctrl+C）。
    fn copy_text(&self) -> String {
        String::new()
    }

    /// 从剪贴板粘贴数值（Ctrl+V）。
    fn paste(&mut self, _text: &str) {}

    /// 切换历史面板（Ctrl+H）。
    fn toggle_history(&mut self) {}

    /// 标题栏是否要显示历史面板按钮；`Some(是否展开)` 表示显示。
    ///
    /// 只有标准与科学模式有历史面板，程序员模式没有，日期/换算器也没有。
    fn history_open(&self) -> Option<bool> {
        None
    }
}

impl CalcApp {
    /// 当前页面对应的 [`PageView`]。
    ///
    /// 整个 `app` 模块里只有这一个 `match self.page`；返回 `&mut dyn` 是为了让
    /// 渲染、按键、剪贴板共用同一条路径，只读的调用方直接在上面读即可。
    pub(super) fn page_view(&mut self) -> &mut dyn PageView {
        match self.page {
            Page::Standard => &mut self.standard,
            Page::Scientific => &mut self.scientific,
            Page::Programmer => &mut self.programmer,
            Page::DateCalculation => &mut self.date,
            Page::Graphing => &mut self.graph,
            // 13 个换算器分类共用一个状态：具体分类由 `Page` 携带，
            // 状态由 `ConverterState` 持有。
            Page::Converter(_) => &mut self.converter,
        }
    }
}

impl PageView for StandardCalc {
    fn render(&self, cx: &mut Context<CalcApp>) -> AnyElement {
        standard::render(self, cx)
    }

    fn handle_key(&mut self, keystroke: &Keystroke) {
        StandardCalc::handle_key(self, keystroke);
    }

    fn copy_text(&self) -> String {
        StandardCalc::copy_text(self)
    }

    fn paste(&mut self, text: &str) {
        StandardCalc::paste(self, text);
    }

    fn toggle_history(&mut self) {
        StandardCalc::toggle_history(self);
    }

    fn history_open(&self) -> Option<bool> {
        Some(self.show_history)
    }
}

impl PageView for ScientificCalc {
    fn render(&self, cx: &mut Context<CalcApp>) -> AnyElement {
        scientific::render(self, cx)
    }

    fn handle_key(&mut self, keystroke: &Keystroke) {
        ScientificCalc::handle_key(self, keystroke);
    }

    fn copy_text(&self) -> String {
        ScientificCalc::copy_text(self)
    }

    fn paste(&mut self, text: &str) {
        ScientificCalc::paste(self, text);
    }

    fn toggle_history(&mut self) {
        ScientificCalc::toggle_history(self);
    }

    fn history_open(&self) -> Option<bool> {
        Some(self.show_history)
    }
}

impl PageView for ProgrammerCalc {
    fn render(&self, cx: &mut Context<CalcApp>) -> AnyElement {
        programmer::render(self, cx)
    }

    fn handle_key(&mut self, keystroke: &Keystroke) {
        ProgrammerCalc::handle_key(self, keystroke);
    }

    fn copy_text(&self) -> String {
        ProgrammerCalc::copy_text(self)
    }

    fn paste(&mut self, text: &str) {
        ProgrammerCalc::paste(self, text);
    }
}

impl PageView for GraphState {
    fn render(&self, cx: &mut Context<CalcApp>) -> AnyElement {
        graph::render(self, cx)
    }

    fn handle_key(&mut self, keystroke: &Keystroke) {
        GraphState::handle_key(self, keystroke);
    }

    fn copy_text(&self) -> String {
        GraphState::copy_text(self)
    }

    fn paste(&mut self, text: &str) {
        GraphState::paste(self, text);
    }
}

impl PageView for ConverterState {
    fn render(&self, cx: &mut Context<CalcApp>) -> AnyElement {
        converter::render(self, cx)
    }

    fn handle_key(&mut self, keystroke: &Keystroke) {
        ConverterState::handle_key(self, keystroke);
    }

    fn copy_text(&self) -> String {
        ConverterState::copy_text(self)
    }

    fn paste(&mut self, text: &str) {
        ConverterState::paste(self, text);
    }
}

impl PageView for DateCalc {
    fn render(&self, cx: &mut Context<CalcApp>) -> AnyElement {
        date::render(self, cx)
    }

    /// 日期页没有键盘输入，只把结果交给 Ctrl+C。
    fn copy_text(&self) -> String {
        self.summary()
    }
}
