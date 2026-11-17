//! 应用根视图：页面切换、状态持久化，以及把各部分装配成一个窗口。
//!
//! 每个计算器模式自己维护状态和界面，这里只负责：
//! 1. 决定当前显示哪个模式；
//! 2. 顶栏菜单（F10 打开、方向键选择、Enter 确认）；
//! 3. 浅色 / 深色 / 跟随系统主题（含 Windows 原生标题栏）；
//! 4. 把历史、记忆、主题、上次页面和窗口尺寸保存到本地；
//! 5. 把键盘输入转交给当前模式。
//!
//! 按职责拆成子模块：
//!
//! - [`page`]：页面清单与元数据；
//! - [`theme`]：主题偏好与原生标题栏配色；
//! - [`dispatch`]：页面接口，`Page` 的唯一分发点；
//! - [`menu`]：汉堡菜单的状态机与渲染；
//! - [`keyboard`]：键盘与剪贴板分发；
//! - [`titlebar`]：`render`、标题栏与页面主体。
//!
//! 根视图 [`CalcApp`] 本身留在本文件：它的字段是各子模块共享的可变状态。

mod dispatch;
mod keyboard;
mod menu;
mod page;
mod theme;
mod titlebar;

pub use page::Page;
pub use theme::ThemePref;

use crate::modes::programmer::ProgrammerCalc;
use crate::modes::scientific::{self, ScientificCalc};
use crate::modes::standard::{self, StandardCalc};
use crate::shared::history;
use crate::shared::storage;
use crate::tools::converter::ConverterState;
use crate::tools::date::DateCalc;
use crate::tools::graph::GraphState;
use gpui_kit::*;
use std::time::{Duration, Instant};

/// 应用根视图。
pub struct CalcApp {
    /// 当前页面。
    page: Page,
    /// 标准模式状态。
    pub standard: StandardCalc,
    /// 科学模式状态。
    pub scientific: ScientificCalc,
    /// 程序员模式状态。
    pub programmer: ProgrammerCalc,
    /// 换算器状态。
    pub converter: ConverterState,
    /// 日期计算状态。
    pub date: DateCalc,
    /// 绘图状态。
    pub graph: GraphState,
    /// 用于接收键盘输入的焦点句柄（首次渲染时惰性创建）。
    focus_handle: Option<FocusHandle>,
    /// 首次渲染时把焦点交给根视图，之后不再抢焦点。
    needs_focus: bool,
    /// 上次设置的窗口标题对应的页面，避免每帧都调用系统接口。
    window_title: Option<Page>,
    /// 主题偏好。
    theme: ThemePref,
    /// 上次写入磁盘的状态。
    saved: storage::SavedState,
    /// 上次保存时间（写入节流）。
    last_save: Instant,
    /// 当前窗口尺寸（逻辑像素）。
    window_width: f32,
    window_height: f32,
    /// 汉堡菜单是否打开。
    menu_open: bool,
    /// 键盘选中的菜单项下标。
    menu_index: usize,
    /// 菜单列表的滚动句柄，键盘选择时把当前项滚进可见区域。
    menu_scroll: ScrollHandle,
    /// 原生标题栏最后一次设置的外观（None 表示还没同步过）。
    native_titlebar_dark: Option<bool>,
}

impl CalcApp {
    pub fn new() -> Self {
        Self::new_with(storage::load())
    }

    /// 用指定的存档构造（测试里传入默认存档，避免依赖本机配置文件）。
    pub fn new_with(saved: storage::SavedState) -> Self {
        let (window_width, window_height) = saved.window_size();

        // 恢复历史记录与记忆槽。
        let mut standard = StandardCalc::new();
        standard.history = history::recent(&saved.standard_history)
            .iter()
            .map(|entry| standard::HistoryEntry {
                expression: entry.expression.clone(),
                result: entry.result.clone(),
            })
            .collect();
        standard.memory.replace(saved.standard_memories.clone());

        let mut scientific = ScientificCalc::new();
        scientific.history = history::recent(&saved.scientific_history)
            .iter()
            .map(|entry| scientific::HistoryEntry {
                expression: entry.expression.clone(),
                result: entry.result.clone(),
            })
            .collect();
        scientific.memory.replace(saved.scientific_memories.clone());

        let mut programmer = ProgrammerCalc::new();
        programmer.memory.replace(saved.programmer_memories.clone());

        let mut app = Self {
            page: Page::from_saved(saved.page.as_deref()),
            standard,
            scientific,
            programmer,
            converter: ConverterState::new(),
            date: DateCalc::new(),
            graph: GraphState::new(),
            focus_handle: None,
            needs_focus: true,
            window_title: None,
            theme: ThemePref::from_saved(saved.theme.as_deref()),
            saved,
            // 允许首次渲染立即写一次，之后按 600ms 节流。
            last_save: Instant::now()
                .checked_sub(Duration::from_secs(2))
                .unwrap_or_else(Instant::now),
            window_width,
            window_height,
            menu_open: false,
            menu_index: 0,
            menu_scroll: ScrollHandle::new(),
            native_titlebar_dark: None,
        };
        app.sync_page_state();
        app
    }

    /// 切换页面（菜单项点击时调用）。
    pub fn set_page(&mut self, page: Page) {
        self.page = page;
        self.sync_page_state();
    }

    /// 页面切换后的收尾：收起面板、同步换算器分类。
    fn sync_page_state(&mut self) {
        // 切换模式时收起面板，回到按键区。
        self.standard.show_history = false;
        self.standard.memory.close();
        self.scientific.show_history = false;
        self.scientific.memory.close();
        // 进入某个换算器页面时切换分类并重置输入。
        if let Some(kind) = self.page.converter_kind()
            && self.converter.kind() != kind
        {
            self.converter.set_kind(kind);
        }
    }

    /// 当前页面对应的标题。
    pub(super) fn title(&self) -> &'static str {
        self.page.title()
    }

    // ---- 持久化 ----

    fn snapshot(&self) -> storage::SavedState {
        storage::SavedState {
            standard_history: self
                .standard
                .history
                .iter()
                .map(|entry| storage::SavedEntry {
                    expression: entry.expression.clone(),
                    result: entry.result.clone(),
                })
                .collect(),
            standard_memories: self.standard.memory.to_vec(),
            scientific_history: self
                .scientific
                .history
                .iter()
                .map(|entry| storage::SavedEntry {
                    expression: entry.expression.clone(),
                    result: entry.result.clone(),
                })
                .collect(),
            scientific_memories: self.scientific.memory.to_vec(),
            programmer_memories: self.programmer.memory.to_vec(),
            theme: Some(self.theme.as_str().to_string()),
            page: Some(self.page.as_str().to_string()),
            window_width: Some(self.window_width),
            window_height: Some(self.window_height),
        }
    }

    /// 状态有变化时写入磁盘，最多每 600ms 写一次。
    fn persist(&mut self) {
        let snapshot = self.snapshot();
        if snapshot != self.saved && self.last_save.elapsed() >= Duration::from_millis(600) {
            storage::save(&snapshot);
            self.saved = snapshot;
            self.last_save = Instant::now();
        }
    }
}

impl Default for CalcApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::page::{MODE_COUNT, MODE_PAGES, Page, menu_pages};
    use super::theme::ThemePref;
    use super::{CalcApp, storage};
    use crate::tools::converter::ConverterKind;

    /// 菜单切换页面时，应该收起上一个模式的面板。
    #[test]
    fn set_page_switches_mode_and_closes_panels() {
        let mut app = CalcApp::new_with(Default::default());
        app.standard.show_history = true;
        app.standard.memory.open();

        app.set_page(Page::Programmer);
        assert_eq!(app.page, Page::Programmer);
        assert!(!app.standard.show_history);
        assert!(!app.standard.memory.is_open());

        app.set_page(Page::Standard);
        assert_eq!(app.page, Page::Standard);
    }

    /// 每个模式都有名字，换算器页面能映射到分类。
    #[test]
    fn pages_have_titles_and_converter_kinds() {
        for page in [Page::Standard, Page::Programmer, Page::Graphing] {
            assert!(!page.title().is_empty());
            assert!(page.converter_kind().is_none());
        }
        for kind in [
            ConverterKind::Currency,
            ConverterKind::Length,
            ConverterKind::Angle,
        ] {
            let page = Page::Converter(kind);
            assert!(!page.title().is_empty());
            assert!(page.converter_kind().is_some());
        }
    }

    /// 切换换算器页面会同步换算器分类。
    #[test]
    fn set_page_syncs_converter_kind() {
        let mut app = CalcApp::new_with(Default::default());
        app.set_page(Page::Converter(ConverterKind::Length));
        assert_eq!(app.converter.kind(), ConverterKind::Length);
        app.set_page(Page::Converter(ConverterKind::Angle));
        assert_eq!(app.converter.kind(), ConverterKind::Angle);
    }

    /// 主题偏好按 浅色 → 深色 → 跟随系统 循环，并能从存档恢复。
    #[test]
    fn theme_preference_cycles_and_parses() {
        assert_eq!(ThemePref::from_saved(None), ThemePref::Light);
        assert_eq!(ThemePref::from_saved(Some("dark")), ThemePref::Dark);
        assert_eq!(ThemePref::from_saved(Some("system")), ThemePref::System);

        assert_eq!(ThemePref::Light.next(), ThemePref::Dark);
        assert_eq!(ThemePref::Dark.next(), ThemePref::System);
        assert_eq!(ThemePref::System.next(), ThemePref::Light);
    }

    /// 菜单键盘导航：打开时定位当前页，上下循环，Enter 切换页面。
    #[test]
    fn menu_keyboard_navigation() {
        let mut app = CalcApp::new_with(Default::default());
        let length = Page::Converter(ConverterKind::Length);
        let weight = Page::Converter(ConverterKind::Weight);

        app.set_page(length);
        app.toggle_menu();
        assert!(app.menu_open);
        assert_eq!(menu_pages()[app.menu_index], length);

        app.move_menu(1);
        assert_eq!(menu_pages()[app.menu_index], weight);
        app.move_menu(-1);
        assert_eq!(menu_pages()[app.menu_index], length);

        // 从第一项往上应回绕到最后一项。
        app.menu_index = 0;
        app.move_menu(-1);
        assert_eq!(app.menu_index, menu_pages().len() - 1);

        app.menu_index = 1;
        app.activate_menu();
        assert_eq!(app.page, Page::Scientific);
        assert!(!app.menu_open);
    }

    /// 打开菜单时按 Esc 关闭，不影响当前页面。
    #[test]
    fn closing_menu_keeps_page() {
        let mut app = CalcApp::new_with(Default::default());
        let angle = Page::Converter(ConverterKind::Angle);
        app.set_page(angle);
        app.toggle_menu();
        app.close_menu();
        assert!(!app.menu_open);
        assert_eq!(app.page, angle);
    }

    /// 用存档构造时恢复历史、记忆、主题和窗口尺寸。
    #[test]
    fn new_with_restores_saved_state() {
        let saved = storage::SavedState {
            standard_history: vec![storage::SavedEntry {
                expression: "2 + 2 =".to_string(),
                result: "4".to_string(),
            }],
            standard_memories: vec![9.0],
            scientific_memories: vec![1.5],
            programmer_memories: vec![255],
            theme: Some("dark".to_string()),
            page: Some("graphing".to_string()),
            window_width: Some(420.0),
            window_height: Some(700.0),
            ..Default::default()
        };

        let app = CalcApp::new_with(saved);
        assert_eq!(app.standard.history.len(), 1);
        assert_eq!(app.standard.history[0].result, "4");
        assert_eq!(app.standard.memory.to_vec(), vec![9.0]);
        assert_eq!(app.scientific.memory.to_vec(), vec![1.5]);
        assert_eq!(app.programmer.memory.to_vec(), vec![255]);
        assert_eq!(app.theme, ThemePref::Dark);
        assert_eq!(app.page, Page::Graphing);
        assert_eq!(app.window_width, 420.0);
        assert_eq!(app.window_height, 700.0);
    }

    /// 菜单来自「模式 + 全部换算器分类」，新增分类会自动出现在菜单里。
    #[test]
    fn menu_pages_cover_modes_and_every_converter_kind() {
        let pages = menu_pages();
        assert_eq!(pages.len(), MODE_COUNT + ConverterKind::ALL.len());
        assert_eq!(&pages[..MODE_COUNT], &MODE_PAGES);
        for kind in ConverterKind::ALL {
            assert!(
                pages.contains(&Page::Converter(kind)),
                "{kind:?} 不在菜单里"
            );
        }
    }

    /// 页面标识能完整往返。
    #[test]
    fn page_names_roundtrip() {
        for &page in menu_pages() {
            assert_eq!(Page::from_saved(Some(page.as_str())), page);
        }
        assert_eq!(Page::from_saved(None), Page::Standard);
        assert_eq!(Page::from_saved(Some("unknown")), Page::Standard);
    }

    /// 存档快照包含历史、记忆、主题和窗口尺寸。
    #[test]
    fn snapshot_includes_persistent_state() {
        let mut app = CalcApp::new_with(Default::default());
        app.set_page(Page::Standard);
        app.standard.display = "7".to_string();
        app.standard.memory_store();
        app.cycle_theme();

        let snapshot = app.snapshot();
        assert_eq!(snapshot.standard_memories, vec![7.0]);
        assert_eq!(snapshot.theme.as_deref(), Some("dark"));
        assert_eq!(snapshot.page.as_deref(), Some("standard"));
        assert!(snapshot.window_width.is_some());
        assert!(snapshot.window_height.is_some());
    }
}
