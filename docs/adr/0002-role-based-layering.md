# 角色分层：`shared` 不被模式依赖

目录按**角色**分，不按功能分：`src/modes/`（标准 / 科学 / 程序员，同一件事的三种方言）、`src/tools/`（换算器 / 日期 / 绘图，三个互不相干的问题）、`src/shared/`（求值器、记忆槽、数值格式化、持久化、配色与按键）。硬规则只有一条：**`shared` 不得 `use crate::modes` 或 `crate::tools`**，依赖方向永远是 `app → modes/tools → shared`。

每个页面拆成 `<name>.rs`（声明）+ `<name>/state.rs` + `<name>/view.rs`；科学模式再多一层 `ops.rs`（运算符语义）+ `expr.rs`（token 与求值），绘图再多 `expr.rs`（文本 → AST）+ `scene.rs`（坐标映射）+ `view.rs`。拆的位置不是新发明的——绘图原本就用 `// ---` 把文件分成了四段，顺着作者的边界切。

代价：拆分时需要放宽一批可见性（多为 `pub(super)`，外层模块仍然是私有的）；`graph` 的测试留在 `graph.rs` 而不是分散到四个子模块，因为一组测试会同时跨 expr / state / scene。

判断一个模块该不该在 `shared` 里，标准是「它认不认识某个模式的名字」。记忆槽 `Memory<T>` 只认「一格值」和「面板开关」，M+ 的算术由调用方以闭包传入（程序员模式要按位宽截断），所以它在 `shared`；标准模式原来自带的 `format_number` 因为违反这条规则而被删除（结果见 ADR-0004）。
