//! 页面清单：菜单里每一项的名字、图标、存档标识。
//!
//! 页面只有两种：计算器模式（各自独立的状态），和换算器（13 个分类共用一份状态）。
//! 换算器的分类不是页面自己的枚举，而是 [`ConverterKind`]——页面只是「当前是哪个
//! 分类」的持有者，名字、图标、存档标识都由分类自己回答。
//!
//! 这里只有「有哪些页面」和「页面怎么描述自己」，不含任何状态。

use crate::tools::converter::ConverterKind;
use gpui_kit::assets::IconName;
use std::sync::OnceLock;

/// 计算器的一个页面。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Standard,
    Scientific,
    Graphing,
    Programmer,
    DateCalculation,
    /// 换算器页面，携带具体分类。
    Converter(ConverterKind),
}

/// 菜单里「计算器」模式的个数，用来插入「转换器」分组标题。
pub(super) const MODE_COUNT: usize = 5;

/// 菜单「计算器」分组里的页面，顺序与菜单显示一致。
pub(super) const MODE_PAGES: [Page; MODE_COUNT] = [
    Page::Standard,
    Page::Scientific,
    Page::Graphing,
    Page::Programmer,
    Page::DateCalculation,
];

static MENU_PAGES: OnceLock<Vec<Page>> = OnceLock::new();

/// 菜单里可点击的页面，顺序与菜单显示一致：先是模式，再是全部换算器分类。
///
/// 换算器部分由 [`ConverterKind::ALL`] 生成，新增一个分类时菜单自动跟上。
pub(super) fn menu_pages() -> &'static [Page] {
    MENU_PAGES.get_or_init(|| {
        MODE_PAGES
            .iter()
            .copied()
            .chain(ConverterKind::ALL.iter().map(|kind| Page::Converter(*kind)))
            .collect()
    })
}

impl Page {
    /// 标题栏与菜单里显示的名字。
    pub fn title(self) -> &'static str {
        match self {
            Page::Standard => "标准",
            Page::Scientific => "科学",
            Page::Graphing => "绘图",
            Page::Programmer => "程序员",
            Page::DateCalculation => "日期计算",
            Page::Converter(kind) => kind.title(),
        }
    }

    /// 菜单图标。
    pub fn icon(self) -> IconName {
        match self {
            Page::Standard => IconName::Calculator,
            Page::Scientific => IconName::SquareFunction,
            Page::Graphing => IconName::ChartLine,
            Page::Programmer => IconName::Binary,
            Page::DateCalculation => IconName::CalendarClock,
            Page::Converter(kind) => kind.icon(),
        }
    }

    /// 页面在存档里的标识。
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Page::Standard => "standard",
            Page::Scientific => "scientific",
            Page::Graphing => "graphing",
            Page::Programmer => "programmer",
            Page::DateCalculation => "date",
            Page::Converter(kind) => kind.as_str(),
        }
    }

    /// 从存档里的标识恢复页面，未知值回到标准模式。
    pub fn from_saved(value: Option<&str>) -> Self {
        let Some(value) = value else {
            return Page::Standard;
        };
        MODE_PAGES
            .iter()
            .copied()
            .find(|page| page.as_str() == value)
            .or_else(|| ConverterKind::from_str(value).map(Page::Converter))
            .unwrap_or(Page::Standard)
    }

    /// 该页面是否属于换算器，以及对应的分类。
    pub(super) fn converter_kind(self) -> Option<ConverterKind> {
        match self {
            Page::Converter(kind) => Some(kind),
            _ => None,
        }
    }
}
