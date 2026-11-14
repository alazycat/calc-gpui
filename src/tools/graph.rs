//! 「绘图」模式：输入 y = f(x)，在坐标系里画出函数曲线。
//!
//! - 表达式解析器支持 `+ - * / % ^`、括号、隐式乘法（`2x`、`(x+1)(x-1)`）、
//!   常量 `pi`/`e`、变量 `x`，以及 sin/cos/tan、ln/log、sqrt/abs 等函数；
//! - 曲线在 GPUI canvas 里绘制，带网格、坐标轴和刻度标签；
//! - 支持放大 / 缩小 / 平移 / 自动适配 y 轴。
//!
//! 按「解析 → 状态 → 几何 → 绘制」拆成四个子模块，依赖方向单向：
//!
//! - [`expr`]：文本 → 可反复求值的表达式树（纯计算）；
//! - [`state`]：输入、光标、已绘曲线与视口；
//! - [`scene`]：视口 + 曲线 → 网格 / 坐标轴 / 刻度（纯几何）；
//! - [`view`]：`render` 与全部绘制函数。
//!
//! 测试留在本文件：它们同时跨 [`expr`]、[`state`] 和 [`scene`] 三个子模块。

mod expr;
mod scene;
mod state;
mod view;

pub use state::GraphState;
pub use view::render;

#[cfg(test)]
mod tests {
    use super::expr::{Expr, parse};
    use super::scene::{format_tick, nice_step, ticks};
    use super::state::GraphState;

    fn eval(input: &str, x: f64) -> f64 {
        parse(input).expect("表达式应能解析").eval(x)
    }

    #[test]
    fn parses_arithmetic_and_precedence() {
        assert_eq!(eval("2 + 3 * 4", 0.0), 14.0);
        assert_eq!(eval("(2 + 3) * 4", 0.0), 20.0);
        assert_eq!(eval("2^3^2", 0.0), 512.0); // 右结合
        assert_eq!(eval("7 % 4", 0.0), 3.0);
    }

    #[test]
    fn parses_implicit_multiplication() {
        assert_eq!(eval("2x", 3.0), 6.0);
        assert_eq!(eval("2(x + 1)", 3.0), 8.0);
        assert_eq!(eval("(x + 1)(x - 1)", 3.0), 8.0);
    }

    #[test]
    fn parses_variables_constants_and_functions() {
        assert_eq!(eval("x^2 + 1", 2.0), 5.0);
        assert_eq!(eval("-x^2", 2.0), -4.0);
        assert!((eval("pi", 0.0) - std::f64::consts::PI).abs() < 1e-12);
        assert!((eval("sin(pi / 2)", 0.0) - 1.0).abs() < 1e-12);
        assert_eq!(eval("sqrt(x)", 9.0), 3.0);
        assert_eq!(eval("log(1000)", 0.0), 3.0);
        assert_eq!(eval("abs(0 - 2)", 0.0), 2.0);
        assert_eq!(eval("floor(2.9)", 0.0), 2.0);
    }

    #[test]
    fn reports_parse_errors() {
        assert!(parse("sin(").is_err());
        assert!(parse("foo(x)").is_err());
        assert!(parse("").is_err());
        assert!(parse("2 +").is_err());
        assert!(parse("x)").is_err());
    }

    #[test]
    fn nice_steps_produce_readable_ticks() {
        assert_eq!(nice_step(20.0), 2.0);
        assert_eq!(nice_step(14.0), 2.0);
        assert_eq!(nice_step(100.0), 10.0);
        assert_eq!(nice_step(1.0), 0.1);
        assert_eq!(ticks(-10.0, 10.0, 2.0).len(), 11);
        assert_eq!(format_tick(2.0, 2.0), "2");
        assert_eq!(format_tick(0.25, 0.25), "0.25");
    }

    #[test]
    fn graph_editing_moves_cursor() {
        let mut state = GraphState::new();
        state.clear();
        assert_eq!(state.copy_text(), "");
        for ch in "2x".chars() {
            state.input_char(ch);
        }
        assert_eq!(state.copy_text(), "2x");
        state.backspace();
        assert_eq!(state.copy_text(), "2");
        state.input_char('*');
        state.input_char('3');
        assert_eq!(state.copy_text(), "2*3");
        assert!(state.plotted.is_some());

        // 光标：回到开头插入。
        state.move_home();
        state.input_char('9');
        assert_eq!(state.copy_text(), "92*3");
        // 光标停在 '*' 之前，向前删除它。
        state.move_home();
        state.move_cursor(2);
        state.delete_forward();
        assert_eq!(state.copy_text(), "923");
        // 退格删掉光标左边的 '2'。
        state.backspace();
        assert_eq!(state.copy_text(), "93");
    }

    #[test]
    fn zoom_and_pan_keep_view_consistent() {
        let mut state = GraphState::new();
        state.zoom(0.5);
        assert_eq!(state.x_min, -5.0);
        assert_eq!(state.x_max, 5.0);
        state.pan(0.5, 0.0);
        assert_eq!(state.x_min, 0.0);
        assert_eq!(state.x_max, 10.0);
        state.reset_view();
        assert_eq!((state.x_min, state.x_max), (-10.0, 10.0));
    }

    #[test]
    fn fit_y_follows_the_curve() {
        let mut state = GraphState::new();
        state.paste("x^2");
        state.fit_y();
        // x^2 在 [-10, 10] 上取 0..100，留出 10% 余量。
        assert!(state.y_min <= 0.0);
        assert!(state.y_min >= -11.0);
        assert!(state.y_max >= 100.0);
        assert!(state.y_max <= 111.0);
    }

    #[test]
    fn expression_clone_evaluates_independently() {
        let expr: Expr = parse("x + 1").unwrap();
        let clone = expr.clone();
        assert_eq!(expr.eval(1.0), 2.0);
        assert_eq!(clone.eval(2.0), 3.0);
    }
}
