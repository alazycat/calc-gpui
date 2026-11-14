//! 日期计算的状态：公历换算、日期差、以及「差 / 加减」两种模式的状态机。

use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// 日期
// ---------------------------------------------------------------------------

/// 一个公历日期。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    /// 构造日期：月份、日期超出范围时收敛到最近的合法值。
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        let year = year.clamp(1, 9999);
        let month = month.clamp(1, 12);
        let day = day.clamp(1, Date::days_in_month(year, month));
        Self { year, month, day }
    }

    fn is_leap(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Date::is_leap(year) => 29,
            _ => 28,
        }
    }

    fn year_length(year: i32) -> u32 {
        if Date::is_leap(year) { 366 } else { 365 }
    }

    /// 距离 1970-01-01 的天数。
    pub fn to_days(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    pub fn from_days(days: i64) -> Self {
        let (year, month, day) = civil_from_days(days);
        Date::new(year, month, day)
    }

    pub fn today() -> Self {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or(0);
        Date::from_days(seconds.div_euclid(86_400))
    }

    pub fn add_days(self, days: i64) -> Self {
        Date::from_days(self.to_days() + days)
    }

    /// 加月份：日期超过目标月份天数时收敛到月末（1 月 31 日 + 1 月 = 2 月 28/29 日）。
    pub fn add_months(self, months: i64) -> Self {
        let total = self.year as i64 * 12 + (self.month as i64 - 1) + months;
        let year = total.div_euclid(12) as i32;
        let month = (total.rem_euclid(12) + 1) as u32;
        Date::new(year, month, self.day)
    }

    pub fn add_years(self, years: i64) -> Self {
        self.add_months(years * 12)
    }

    /// 星期几，例如「星期四」。
    pub fn weekday(self) -> &'static str {
        const NAMES: [&str; 7] = [
            "星期一",
            "星期二",
            "星期三",
            "星期四",
            "星期五",
            "星期六",
            "星期日",
        ];
        // 1970-01-01 是星期四。
        let index = (self.to_days() + 3).rem_euclid(7) as usize;
        NAMES[index]
    }

    /// 一年中的第几天（从 1 开始）。
    pub fn day_of_year(self) -> u32 {
        (self.to_days() - Date::new(self.year, 1, 1).to_days() + 1) as u32
    }

    pub fn days_in_year(self) -> u32 {
        Date::year_length(self.year)
    }

    /// `2026年11月13日`。
    pub fn format_cn(self) -> String {
        format!("{}年{}月{}日", self.year, self.month, self.day)
    }
}

/// 公历 → 距 1970-01-01 的天数。
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year as i64 - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400; // [0, 399]
    let month_index = month as i64;
    let day_of_year =
        (153 * (month_index + if month_index > 2 { -3 } else { 9 }) + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// 距 1970-01-01 的天数 → 公历。
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097; // [0, 146096]
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = month_index + if month_index < 10 { 3 } else { -9 };
    (
        (year + if month <= 2 { 1 } else { 0 }) as i32,
        month as u32,
        day as u32,
    )
}

// ---------------------------------------------------------------------------
// 计算
// ---------------------------------------------------------------------------

/// 日期间隔结果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Difference {
    pub years: i64,
    pub months: i64,
    pub days: i64,
    /// 总天数（始终为非负数）。
    pub total_days: i64,
}

/// `end - start`，自动按较早 → 较晚计算，返回正数。
pub fn difference(start: Date, end: Date) -> Difference {
    let (earlier, later) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };

    let mut years = 0i64;
    while years < 10_000 && earlier.add_years(years + 1) <= later {
        years += 1;
    }

    let base = earlier.add_years(years);
    let mut months = 0i64;
    while months < 12 && base.add_months(months + 1) <= later {
        months += 1;
    }

    let anchor = base.add_months(months);
    Difference {
        years,
        months,
        days: later.to_days() - anchor.to_days(),
        total_days: later.to_days() - earlier.to_days(),
    }
}

/// 日期计算的两个页面。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DateMode {
    /// 两个日期的差。
    Difference,
    /// 给日期加 / 减年、月、周、天。
    Offset,
}

/// 要调整哪一个日期。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DateTarget {
    Start,
    End,
}

/// 日期的年 / 月 / 日。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DatePart {
    Year,
    Month,
    Day,
}

/// 加减日期的四个字段。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AmountPart {
    Years,
    Months,
    Weeks,
    Days,
}

/// 日期计算状态。
pub struct DateCalc {
    pub mode: DateMode,
    start: Date,
    end: Date,
    years: i32,
    months: i32,
    weeks: i32,
    days: i32,
}

impl DateCalc {
    pub fn new() -> Self {
        let today = Date::today();
        Self {
            mode: DateMode::Difference,
            start: today,
            end: today,
            years: 0,
            months: 0,
            weeks: 0,
            days: 0,
        }
    }

    pub fn set_mode(&mut self, mode: DateMode) {
        self.mode = mode;
    }

    pub fn start(&self) -> Date {
        self.start
    }

    pub fn end(&self) -> Date {
        self.end
    }

    pub fn years(&self) -> i32 {
        self.years
    }

    pub fn months(&self) -> i32 {
        self.months
    }

    pub fn weeks(&self) -> i32 {
        self.weeks
    }

    pub fn days(&self) -> i32 {
        self.days
    }

    pub fn difference(&self) -> Difference {
        difference(self.start, self.end)
    }

    /// 结果摘要（Ctrl+C）。
    pub fn summary(&self) -> String {
        match self.mode {
            DateMode::Difference => {
                let diff = self.difference();
                format!("{} 年 {} 个月 {} 天", diff.years, diff.months, diff.days)
            }
            DateMode::Offset => self.result_date().format_cn(),
        }
    }

    /// 加减模式的结果日期。
    pub fn result_date(&self) -> Date {
        self.start
            .add_years(self.years as i64)
            .add_months(self.months as i64)
            .add_days(self.weeks as i64 * 7 + self.days as i64)
    }

    pub fn set_today(&mut self, target: DateTarget) {
        let today = Date::today();
        match target {
            DateTarget::Start => self.start = today,
            DateTarget::End => self.end = today,
        }
    }

    /// 调整开始 / 结束日期的一个字段。
    pub fn step_date(&mut self, target: DateTarget, part: DatePart, delta: i32) {
        let date = match target {
            DateTarget::Start => self.start,
            DateTarget::End => self.end,
        };
        let updated = match part {
            DatePart::Year => date.add_years(delta as i64),
            DatePart::Month => date.add_months(delta as i64),
            DatePart::Day => date.add_days(delta as i64),
        };
        match target {
            DateTarget::Start => self.start = updated,
            DateTarget::End => self.end = updated,
        }
    }

    /// 调整加 / 减量。
    pub fn step_amount(&mut self, part: AmountPart, delta: i32) {
        let value = match part {
            AmountPart::Years => &mut self.years,
            AmountPart::Months => &mut self.months,
            AmountPart::Weeks => &mut self.weeks,
            AmountPart::Days => &mut self.days,
        };
        *value = (*value + delta).clamp(-9999, 9999);
    }
}

impl Default for DateCalc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AmountPart, Date, DateCalc, DateMode, DatePart, DateTarget, civil_from_days,
        days_from_civil, difference,
    };

    #[test]
    fn epoch_conversion() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 1, 1), 10_957);
        assert_eq!(days_from_civil(1969, 12, 31), -1);
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(10_957), (2000, 1, 1));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
    }

    #[test]
    fn roundtrip_days() {
        for (year, month, day) in [
            (1900, 2, 28),
            (2000, 2, 29),
            (2024, 12, 31),
            (1, 1, 1),
            (9999, 12, 31),
        ] {
            let days = days_from_civil(year, month, day);
            assert_eq!(civil_from_days(days), (year, month, day));
        }
    }

    #[test]
    fn add_days_across_leap_day() {
        let date = Date::new(2024, 2, 28);
        assert_eq!(date.add_days(1), Date::new(2024, 2, 29));
        assert_eq!(date.add_days(2), Date::new(2024, 3, 1));
        assert_eq!(Date::new(2023, 2, 28).add_days(1), Date::new(2023, 3, 1));
    }

    #[test]
    fn add_months_clamps_to_month_end() {
        assert_eq!(Date::new(2024, 1, 31).add_months(1), Date::new(2024, 2, 29));
        assert_eq!(Date::new(2023, 1, 31).add_months(1), Date::new(2023, 2, 28));
        assert_eq!(
            Date::new(2023, 12, 31).add_months(1),
            Date::new(2024, 1, 31)
        );
        assert_eq!(
            Date::new(2024, 3, 31).add_months(-1),
            Date::new(2024, 2, 29)
        );
    }

    #[test]
    fn differences() {
        let diff = difference(Date::new(2024, 1, 31), Date::new(2024, 3, 1));
        assert_eq!((diff.years, diff.months, diff.days), (0, 1, 1));
        assert_eq!(diff.total_days, 30);

        let diff = difference(Date::new(2020, 2, 29), Date::new(2021, 3, 1));
        assert_eq!((diff.years, diff.months, diff.days), (1, 0, 1));
        assert_eq!(diff.total_days, 366);

        // 反向也按正数计算。
        let diff = difference(Date::new(2024, 3, 1), Date::new(2024, 1, 31));
        assert_eq!(diff.total_days, 30);
    }

    #[test]
    fn weekday_and_day_of_year() {
        assert_eq!(Date::new(2024, 1, 1).weekday(), "星期一");
        assert_eq!(Date::new(2024, 2, 29).day_of_year(), 60);
        assert_eq!(Date::new(2024, 2, 29).days_in_year(), 366);
    }

    #[test]
    fn date_calc_steppers() {
        let mut calc = DateCalc::new();
        calc.step_date(DateTarget::Start, DatePart::Year, 1);
        assert_eq!(calc.start().year, (Date::today().year + 1).min(9999));

        calc.set_mode(DateMode::Offset);
        calc.step_amount(AmountPart::Weeks, 2);
        calc.step_amount(AmountPart::Days, 3);
        assert_eq!(calc.result_date(), calc.start().add_days(17));
    }

    #[test]
    fn format_cn_and_clamping() {
        assert_eq!(Date::new(2024, 2, 31).format_cn(), "2024年2月29日");
        assert_eq!(Date::new(2024, 13, 1).month, 12);
    }
}
