//! 换算器的数据与状态：单位表、换算公式、输入编辑与结果显示。

use crate::shared::number::format_plain;

/// 一个单位。
pub struct UnitDef {
    pub name: &'static str,
    /// 1 个该单位等于多少个基准单位（温度例外）。
    pub factor: f64,
}

const fn unit(name: &'static str, factor: f64) -> UnitDef {
    UnitDef { name, factor }
}

/// 换算方式。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scale {
    /// 线性换算：`base = value × factor`。
    Linear,
    /// 摄氏 / 华氏 / 开尔文。
    Temperature,
}

/// 一类换算。
///
/// 这是换算器分类的描述表：菜单标题、存档标识、单位表与换算方式
/// 都写在同一个 [`Category`] 里，顺序与 [`ConverterKind`] 的声明顺序一致。
/// 菜单图标属于展示层，见 [`crate::app::page`] 的映射。
pub struct Category {
    pub title: &'static str,
    /// 存档里的标识（`Page::as_str` 用它恢复页面）。
    pub slug: &'static str,
    pub units: &'static [UnitDef],
    pub scale: Scale,
    pub default_from: usize,
    pub default_to: usize,
    pub note: Option<&'static str>,
}

/// 换算器分类（顺序与菜单里转换器列表一致）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConverterKind {
    Currency,
    Volume,
    Length,
    Weight,
    Temperature,
    Energy,
    Area,
    Speed,
    Time,
    Power,
    Data,
    Pressure,
    Angle,
}

impl ConverterKind {
    pub const ALL: [ConverterKind; 13] = [
        ConverterKind::Currency,
        ConverterKind::Volume,
        ConverterKind::Length,
        ConverterKind::Weight,
        ConverterKind::Temperature,
        ConverterKind::Energy,
        ConverterKind::Area,
        ConverterKind::Speed,
        ConverterKind::Time,
        ConverterKind::Power,
        ConverterKind::Data,
        ConverterKind::Pressure,
        ConverterKind::Angle,
    ];

    pub fn index(self) -> usize {
        ConverterKind::ALL
            .iter()
            .position(|kind| *kind == self)
            .expect("ConverterKind::ALL 覆盖所有分类")
    }

    /// 这一类的完整描述（单位表、默认单位、换算方式、标题、图标、存档标识）。
    pub fn category(self) -> &'static Category {
        &CATEGORIES[self.index()]
    }

    /// 菜单与标题栏里显示的名字。
    pub fn title(self) -> &'static str {
        self.category().title
    }

    /// 存档里的标识。
    pub fn as_str(self) -> &'static str {
        self.category().slug
    }
}

impl std::str::FromStr for ConverterKind {
    type Err = ();

    /// 从存档里的标识恢复分类（`"length".parse::<ConverterKind>()`）。
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ConverterKind::ALL
            .iter()
            .copied()
            .find(|kind| kind.as_str() == value)
            .ok_or(())
    }
}

// ---- 单位表 ----

const CURRENCY_UNITS: &[UnitDef] = &[
    unit("人民币 (CNY)", 0.1385),
    unit("美元 (USD)", 1.0),
    unit("欧元 (EUR)", 1.08),
    unit("日元 (JPY)", 0.0065),
    unit("英镑 (GBP)", 1.27),
    unit("港币 (HKD)", 0.128),
    unit("韩元 (KRW)", 0.00072),
    unit("澳元 (AUD)", 0.66),
    unit("加元 (CAD)", 0.73),
    unit("瑞士法郎 (CHF)", 1.12),
];

const VOLUME_UNITS: &[UnitDef] = &[
    unit("毫升 (mL)", 0.001),
    unit("升 (L)", 1.0),
    unit("立方米 (m³)", 1000.0),
    unit("立方厘米 (cm³)", 0.001),
    unit("茶匙 (美)", 0.00492892),
    unit("汤匙 (美)", 0.0147868),
    unit("液量盎司 (美)", 0.0295735),
    unit("杯 (美)", 0.236588),
    unit("品脱 (美)", 0.473176),
    unit("夸脱 (美)", 0.946353),
    unit("加仑 (美)", 3.78541),
];

const LENGTH_UNITS: &[UnitDef] = &[
    unit("微米 (µm)", 1e-6),
    unit("毫米 (mm)", 0.001),
    unit("厘米 (cm)", 0.01),
    unit("米 (m)", 1.0),
    unit("千米 (km)", 1000.0),
    unit("英寸 (in)", 0.0254),
    unit("英尺 (ft)", 0.3048),
    unit("码 (yd)", 0.9144),
    unit("英里 (mi)", 1609.344),
    unit("海里 (nmi)", 1852.0),
];

const WEIGHT_UNITS: &[UnitDef] = &[
    unit("毫克 (mg)", 1e-6),
    unit("克 (g)", 0.001),
    unit("千克 (kg)", 1.0),
    unit("吨 (t)", 1000.0),
    unit("盎司 (oz)", 0.0283495),
    unit("磅 (lb)", 0.453592),
    unit("英石 (st)", 6.35029),
    unit("短吨 (US ton)", 907.185),
];

const TEMPERATURE_UNITS: &[UnitDef] = &[
    unit("摄氏度 (°C)", 1.0),
    unit("华氏度 (°F)", 1.0),
    unit("开尔文 (K)", 1.0),
];

const ENERGY_UNITS: &[UnitDef] = &[
    unit("焦耳 (J)", 1.0),
    unit("千焦 (kJ)", 1000.0),
    unit("卡 (cal)", 4.184),
    unit("千卡 (kcal)", 4184.0),
    unit("瓦时 (Wh)", 3600.0),
    unit("千瓦时 (kWh)", 3.6e6),
    unit("电子伏 (eV)", 1.602176634e-19),
    unit("BTU", 1055.06),
];

const AREA_UNITS: &[UnitDef] = &[
    unit("平方毫米 (mm²)", 1e-6),
    unit("平方厘米 (cm²)", 1e-4),
    unit("平方米 (m²)", 1.0),
    unit("平方千米 (km²)", 1e6),
    unit("公顷 (ha)", 1e4),
    unit("平方英寸 (in²)", 0.00064516),
    unit("平方英尺 (ft²)", 0.092903),
    unit("平方码 (yd²)", 0.836127),
    unit("英亩 (ac)", 4046.86),
];

const SPEED_UNITS: &[UnitDef] = &[
    unit("米/秒 (m/s)", 1.0),
    unit("千米/小时 (km/h)", 0.277778),
    unit("英里/小时 (mph)", 0.44704),
    unit("英尺/秒 (ft/s)", 0.3048),
    unit("节 (kn)", 0.514444),
];

const TIME_UNITS: &[UnitDef] = &[
    unit("毫秒 (ms)", 0.001),
    unit("秒 (s)", 1.0),
    unit("分钟 (min)", 60.0),
    unit("小时 (h)", 3600.0),
    unit("天 (d)", 86400.0),
    unit("周 (wk)", 604800.0),
    unit("年 (365 天)", 31_536_000.0),
];

const POWER_UNITS: &[UnitDef] = &[
    unit("瓦 (W)", 1.0),
    unit("千瓦 (kW)", 1000.0),
    unit("兆瓦 (MW)", 1e6),
    unit("英制马力 (hp)", 745.7),
    unit("公制马力 (PS)", 735.49875),
];

const DATA_UNITS: &[UnitDef] = &[
    unit("位 (bit)", 0.125),
    unit("字节 (B)", 1.0),
    unit("千字节 (KB)", 1024.0),
    unit("兆字节 (MB)", 1_048_576.0),
    unit("吉字节 (GB)", 1_073_741_824.0),
    unit("太字节 (TB)", 1_099_511_627_776.0),
];

const PRESSURE_UNITS: &[UnitDef] = &[
    unit("帕 (Pa)", 1.0),
    unit("千帕 (kPa)", 1000.0),
    unit("兆帕 (MPa)", 1e6),
    unit("巴 (bar)", 100_000.0),
    unit("大气压 (atm)", 101_325.0),
    unit("毫米汞柱 (mmHg)", 133.322),
    unit("磅力/平方英寸 (psi)", 6894.76),
];

const ANGLE_UNITS: &[UnitDef] = &[
    unit("度 (°)", 1.0),
    unit("弧度 (rad)", 57.295_779_513_082_3),
    unit("百分度 (grad)", 0.9),
    unit("圈 (turn)", 360.0),
];

/// 换算器分类表（顺序与 [`ConverterKind`] 一致）。
pub const CATEGORIES: [Category; 13] = [
    Category {
        title: "货币",
        slug: "currency",
        units: CURRENCY_UNITS,
        scale: Scale::Linear,
        default_from: 0,
        default_to: 1,
        note: Some("汇率为内置示例（离线），仅用于演示。"),
    },
    Category {
        title: "体积",
        slug: "volume",
        units: VOLUME_UNITS,
        scale: Scale::Linear,
        default_from: 1,
        default_to: 10,
        note: None,
    },
    Category {
        title: "长度",
        slug: "length",
        units: LENGTH_UNITS,
        scale: Scale::Linear,
        default_from: 3,
        default_to: 6,
        note: None,
    },
    Category {
        title: "重量",
        slug: "weight",
        units: WEIGHT_UNITS,
        scale: Scale::Linear,
        default_from: 2,
        default_to: 5,
        note: None,
    },
    Category {
        title: "温度",
        slug: "temperature",
        units: TEMPERATURE_UNITS,
        scale: Scale::Temperature,
        default_from: 0,
        default_to: 1,
        note: None,
    },
    Category {
        title: "能量",
        slug: "energy",
        units: ENERGY_UNITS,
        scale: Scale::Linear,
        default_from: 1,
        default_to: 3,
        note: None,
    },
    Category {
        title: "面积",
        slug: "area",
        units: AREA_UNITS,
        scale: Scale::Linear,
        default_from: 2,
        default_to: 6,
        note: None,
    },
    Category {
        title: "速度",
        slug: "speed",
        units: SPEED_UNITS,
        scale: Scale::Linear,
        default_from: 1,
        default_to: 2,
        note: None,
    },
    Category {
        title: "时间",
        slug: "time",
        units: TIME_UNITS,
        scale: Scale::Linear,
        default_from: 3,
        default_to: 2,
        note: None,
    },
    Category {
        title: "功率",
        slug: "power",
        units: POWER_UNITS,
        scale: Scale::Linear,
        default_from: 1,
        default_to: 3,
        note: None,
    },
    Category {
        title: "数据",
        slug: "data",
        units: DATA_UNITS,
        scale: Scale::Linear,
        default_from: 4,
        default_to: 3,
        note: None,
    },
    Category {
        title: "压力",
        slug: "pressure",
        units: PRESSURE_UNITS,
        scale: Scale::Linear,
        default_from: 1,
        default_to: 6,
        note: None,
    },
    Category {
        title: "角度",
        slug: "angle",
        units: ANGLE_UNITS,
        scale: Scale::Linear,
        default_from: 0,
        default_to: 1,
        note: None,
    },
];

// ---- 换算 ----

fn unit_to_celsius(unit: usize, value: f64) -> f64 {
    match unit {
        0 => value,
        1 => (value - 32.0) * 5.0 / 9.0,
        _ => value - 273.15,
    }
}

fn celsius_to_unit(unit: usize, value: f64) -> f64 {
    match unit {
        0 => value,
        1 => value * 9.0 / 5.0 + 32.0,
        _ => value + 273.15,
    }
}

fn convert(category: &Category, from: usize, to: usize, value: f64) -> f64 {
    match category.scale {
        Scale::Linear => {
            let base = value * category.units[from].factor;
            base / category.units[to].factor
        }
        Scale::Temperature => celsius_to_unit(to, unit_to_celsius(from, value)),
    }
}

// ---------------------------------------------------------------------------
// 状态
// ---------------------------------------------------------------------------

/// 换算器状态。
pub struct ConverterState {
    pub(super) kind: ConverterKind,
    /// 源单位下标。
    from: usize,
    /// 目标单位下标。
    to: usize,
    /// 正在输入的文本。
    input: String,
}

/// 换算器可执行的语义动作（由 UI 层从按键翻译而来）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Digit(char),
    Dot,
    Negate,
    Clear,
    Backspace,
}

impl ConverterState {
    pub fn new() -> Self {
        let mut state = Self {
            kind: ConverterKind::Length,
            from: 0,
            to: 0,
            input: "0".to_string(),
        };
        state.set_kind(ConverterKind::Length);
        state
    }

    /// 切换换算分类（从菜单进入某个转换器页面时调用）。
    pub fn set_kind(&mut self, kind: ConverterKind) {
        self.kind = kind;
        let category = self.category();
        self.from = category.default_from;
        self.to = category.default_to;
        self.input = "0".to_string();
    }

    pub fn kind(&self) -> ConverterKind {
        self.kind
    }

    fn category(&self) -> &'static Category {
        self.kind.category()
    }

    pub fn from(&self) -> usize {
        self.from
    }

    pub fn to(&self) -> usize {
        self.to
    }

    pub fn set_from(&mut self, index: usize) {
        if index < self.category().units.len() {
            self.from = index;
        }
    }

    pub fn set_to(&mut self, index: usize) {
        if index < self.category().units.len() {
            self.to = index;
        }
    }

    pub fn swap_units(&mut self) {
        std::mem::swap(&mut self.from, &mut self.to);
    }

    /// 输入框里的原始文本。
    pub fn input_text(&self) -> &str {
        &self.input
    }

    /// 当前输入值。
    pub fn value(&self) -> f64 {
        self.input.parse::<f64>().unwrap_or(0.0)
    }

    /// 换算结果。
    pub fn result(&self) -> f64 {
        convert(self.category(), self.from, self.to, self.value())
    }

    pub fn source_unit_name(&self) -> &'static str {
        self.category().units[self.from].name
    }

    pub fn target_unit_name(&self) -> &'static str {
        self.category().units[self.to].name
    }

    pub fn copy_text(&self) -> String {
        format_plain(self.result())
    }

    // ---- 输入 ----

    pub fn input_digit(&mut self, digit: char) {
        if self.input == "0" {
            self.input.clear();
        }
        if self.input.len() < 20 {
            self.input.push(digit);
        }
    }

    pub fn input_dot(&mut self) {
        if !self.input.contains('.') {
            self.input.push('.');
        }
    }

    pub fn backspace(&mut self) {
        self.input.pop();
        if self.input.is_empty() || self.input == "-" {
            self.input = "0".to_string();
        }
    }

    pub fn clear(&mut self) {
        self.input = "0".to_string();
    }

    pub fn negate(&mut self) {
        if self.input == "0" {
            return;
        }
        if let Some(rest) = self.input.strip_prefix('-') {
            self.input = rest.to_string();
        } else {
            self.input.insert(0, '-');
        }
    }

    pub fn paste(&mut self, text: &str) {
        let cleaned: String = text
            .chars()
            .filter(|c| !c.is_whitespace() && *c != ',' && *c != '_')
            .collect();
        if let Ok(value) = cleaned.parse::<f64>() {
            self.input = format_plain(value);
        }
    }

    /// 分发一个语义动作；按键到动作的翻译见 [`crate::tools::converter::keystroke_to_action`]。
    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::Digit(digit) => self.input_digit(digit),
            Action::Dot => self.input_dot(),
            Action::Negate => self.negate(),
            Action::Clear => self.clear(),
            Action::Backspace => self.backspace(),
        }
    }
}

impl Default for ConverterState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Action, CATEGORIES, ConverterKind, ConverterState, Scale, celsius_to_unit, convert,
        unit_to_celsius,
    };
    use crate::shared::number::format_plain;

    fn category(kind: ConverterKind) -> &'static super::Category {
        kind.category()
    }

    #[test]
    fn categories_are_in_menu_order() {
        assert_eq!(CATEGORIES.len(), 13);
        assert_eq!(CATEGORIES[0].title, "货币");
        assert_eq!(CATEGORIES[2].title, "长度");
        assert_eq!(CATEGORIES[12].title, "角度");
        assert!(CATEGORIES[0].note.is_some());
    }

    /// 每个分类在分类表里都有自己的一行，且描述自己的方法都指向那一行。
    #[test]
    fn every_kind_is_described_by_categories() {
        assert_eq!(ConverterKind::ALL.len(), CATEGORIES.len());
        for (index, kind) in ConverterKind::ALL.iter().enumerate() {
            assert_eq!(kind.index(), index, "{kind:?} 与 CATEGORIES 顺序不一致");
            assert_eq!(kind.category().title, CATEGORIES[index].title);
            assert_eq!(kind.title(), CATEGORIES[index].title);
            assert!(!kind.title().is_empty());
        }
    }

    /// 存档标识唯一，且能原样恢复成分类。
    #[test]
    fn kinds_roundtrip_through_saved_names() {
        let mut seen = Vec::new();
        for kind in ConverterKind::ALL {
            let slug = kind.as_str();
            assert!(!slug.is_empty());
            assert!(!seen.contains(&slug), "存档标识 {slug} 重复");
            seen.push(slug);
            assert_eq!(slug.parse::<ConverterKind>(), Ok(kind));
        }
        assert_eq!("unknown".parse::<ConverterKind>(), Err(()));
    }

    #[test]
    fn linear_conversions() {
        let length = category(ConverterKind::Length);
        // 1 米 = 100 厘米
        assert!((convert(length, 3, 2, 1.0) - 100.0).abs() < 1e-9);
        // 1 英里 = 1609.344 米
        assert!((convert(length, 8, 3, 1.0) - 1609.344).abs() < 1e-9);

        let data = category(ConverterKind::Data);
        // 1 GB = 1024 MB
        assert_eq!(convert(data, 4, 3, 1.0), 1024.0);
    }

    #[test]
    fn temperature_conversions() {
        assert_eq!(unit_to_celsius(1, 32.0), 0.0);
        assert_eq!(celsius_to_unit(1, 100.0), 212.0);
        assert!((celsius_to_unit(2, 0.0) - 273.15).abs() < 1e-9);
        let temperature = category(ConverterKind::Temperature);
        // 300 K = 26.85 °C = 80.33 °F
        assert!((convert(temperature, 2, 1, 300.0) - 80.33).abs() < 0.01);
    }

    #[test]
    fn default_units_and_switch() {
        let mut state = ConverterState::new();
        assert_eq!(state.kind(), ConverterKind::Length);
        // 长度默认 米 → 英尺
        assert_eq!(state.source_unit_name(), "米 (m)");
        assert_eq!(state.target_unit_name(), "英尺 (ft)");

        state.input_digit('1');
        assert!((state.result() - 3.280_839_895_013_1).abs() < 1e-9);

        state.swap_units();
        assert_eq!(state.source_unit_name(), "英尺 (ft)");
        assert_eq!(state.target_unit_name(), "米 (m)");

        state.set_kind(ConverterKind::Temperature);
        assert_eq!(state.source_unit_name(), "摄氏度 (°C)");
        assert_eq!(state.input_text(), "0");
    }

    #[test]
    fn input_editing() {
        let mut state = ConverterState::new();
        state.input_digit('1');
        state.input_digit('2');
        state.input_dot();
        state.input_digit('5');
        assert_eq!(state.input_text(), "12.5");

        state.backspace();
        assert_eq!(state.input_text(), "12.");
        state.clear();
        assert_eq!(state.input_text(), "0");

        state.input_digit('5');
        state.negate();
        assert_eq!(state.input_text(), "-5");
        state.negate();
        assert_eq!(state.input_text(), "5");
    }

    #[test]
    fn paste_accepts_grouped_numbers() {
        let mut state = ConverterState::new();
        state.paste("1,234.5");
        assert_eq!(state.input_text(), "1234.5");
        state.paste("  42 ");
        assert_eq!(state.input_text(), "42");
    }

    #[test]
    fn linear_scale_marker() {
        assert_eq!(category(ConverterKind::Length).scale, Scale::Linear);
        assert_eq!(
            category(ConverterKind::Temperature).scale,
            Scale::Temperature
        );
    }

    #[test]
    fn result_formatting() {
        assert_eq!(format_plain(3.2808398950131), "3.2808398950131");
    }

    #[test]
    fn apply_action_dispatches_to_semantic_methods() {
        let mut state = ConverterState::new();
        state.apply_action(Action::Digit('1'));
        state.apply_action(Action::Digit('2'));
        state.apply_action(Action::Dot);
        state.apply_action(Action::Digit('5'));
        assert_eq!(state.input_text(), "12.5");

        state.apply_action(Action::Negate);
        assert_eq!(state.input_text(), "-12.5");
        state.apply_action(Action::Negate);
        assert_eq!(state.input_text(), "12.5");
    }

    #[test]
    fn apply_action_clear_and_backspace() {
        let mut state = ConverterState::new();
        state.apply_action(Action::Digit('7'));
        state.apply_action(Action::Digit('8'));
        state.apply_action(Action::Backspace);
        assert_eq!(state.input_text(), "7");

        state.apply_action(Action::Clear);
        assert_eq!(state.input_text(), "0");

        // 空输入上退格回到 0，而不是变负。
        state.apply_action(Action::Backspace);
        assert_eq!(state.input_text(), "0");
    }
}
