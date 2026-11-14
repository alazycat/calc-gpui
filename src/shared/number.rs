//! 各模式共用的数值格式化——全项目只有这一套规则。
//!
//! - 保留 14 位有效数字，去掉浮点误差（`0.1 + 0.2` 显示为 `0.3`）；
//! - 极大或极小的数自动使用科学计数法；
//! - 非有限值显示为「无效输入」。
//!
//! 两个入口，区别只在「谁决定用不用科学计数法」：
//!
//! - [`format_plain`]：没有 F-E 开关的调用方（标准模式、换算器），固定走小数显示；
//! - [`format_value`]：有 F-E 开关的调用方（科学模式），由开关决定。
//!
//! 程序员模式的进制显示（十六/八/二进制）与绘图坐标轴的刻度格式化是另两件事，
//! 刻度还要考虑步长，所以它们各自留在自己的模块里。

/// 按「小数优先」规则格式化：有 F-E 开关的调用方请用 [`format_value`]。
pub fn format_plain(value: f64) -> String {
    format_value(value, false)
}

/// 按 F-E 开关格式化：`scientific` 为 true 时强制科学计数法。
pub fn format_value(value: f64, scientific: bool) -> String {
    if !value.is_finite() {
        return "无效输入".to_string();
    }
    if value == 0.0 {
        if scientific {
            return "0e+0".to_string();
        }
        return "0".to_string();
    }
    if scientific {
        return format_scientific(value);
    }

    let abs = value.abs();
    // 太大或太小的数自动用科学计数法，避免显示成几十位。
    if abs >= 1e15 || abs < 1e-9 {
        return format_scientific(value);
    }

    let rounded = round_significant(value, 14);
    let mut text = format!("{rounded}");
    if text == "-0" {
        text = "0".to_string();
    }
    text
}

/// 保留 `digits` 位有效数字后四舍五入。
fn round_significant(value: f64, digits: i32) -> f64 {
    if value == 0.0 || !value.is_finite() {
        return value;
    }
    let exponent = value.abs().log10().floor() as i32;
    let factor = 10f64.powi(digits - 1 - exponent);
    (value * factor).round() / factor
}

/// `1234` → `1.234e+3`。极大/极小数与 F-E 开关都走这里。
fn format_scientific(value: f64) -> String {
    if !value.is_finite() {
        return "无效输入".to_string();
    }
    if value == 0.0 {
        return "0e+0".to_string();
    }

    let formatted = format!("{value:.10e}");
    let (mantissa, exponent) = formatted.split_once('e').unwrap_or((&formatted, "0"));
    let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
    let exponent: i32 = exponent.parse().unwrap_or(0);
    format!(
        "{mantissa}e{}{}",
        if exponent < 0 { '-' } else { '+' },
        exponent.abs()
    )
}

#[cfg(test)]
mod tests {
    use super::{format_plain, format_value};

    #[test]
    fn cleans_up_floating_point_noise() {
        assert_eq!(format_value(0.1 + 0.2, false), "0.3");
        assert_eq!(format_value(1.0 / 3.0, false), "0.33333333333333");
        assert_eq!(format_value(-2.5, false), "-2.5");
        assert_eq!(format_value(1024.0, false), "1024");
    }

    #[test]
    fn handles_large_and_small_values() {
        assert_eq!(format_value(1.5e20, false), "1.5e+20");
        assert_eq!(format_value(1.23e-12, false), "1.23e-12");
        assert_eq!(format_value(f64::NAN, false), "无效输入");
        assert_eq!(format_value(f64::INFINITY, false), "无效输入");
    }

    #[test]
    fn scientific_notation() {
        assert_eq!(format_value(1234.5, true), "1.2345e+3");
        assert_eq!(format_value(-0.00042, true), "-4.2e-4");
        assert_eq!(format_value(0.0, true), "0e+0");
        assert_eq!(format_plain(12.0), "12");
        assert_eq!(format_plain(1234.5), "1234.5");
    }
}
