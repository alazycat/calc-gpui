# `app` 里只允许一个 `match self.page`

`src/app/dispatch.rs` 定义 `PageView` trait：`render` 必须实现，`handle_key` / `copy_text` / `paste` / `toggle_history` / `history_open` 都有默认空实现（默认值表达的是页面之间的能力差异：日期计算只有 `render` 和 `copy_text`，程序员模式没有历史面板）。`CalcApp::page_view() -> &mut dyn PageView` 是 `app` 模块里**唯一**的 `match self.page`，渲染、按键、剪贴板都走这一条路径。

同时 `Page` 只有 6 个变体：五个模式加 `Page::Converter(ConverterKind)`。换算器的 13 个分类不再是 `Page` 里手抄的一遍枚举，标题、图标、存档标识都由 `ConverterKind` 自己回答（`CATEGORIES` 一行描述一个分类），菜单由 `MODE_PAGES` 拼 `ConverterKind::ALL` 生成。

这么做是因为原先的 18 变体 `Page` 有 5 处 13 臂的镜像 match 要靠人同步，而 `page_view` 用兜底 `_` 把 13 个换算器分支吃掉，新增页面**不会有任何编译错误**。现在 `page_view` 是穷尽匹配，漏了就是编译失败。

代价：`CalcApp::copy_text`、`title_bar` 从 `&self` 变成 `&mut self`（`page_view()` 同时服务于渲染和输入，需要 `&mut`）；`CalcApp` 结构体本身留在 `app.rs`，没搬进子模块，因为 Rust 的可见性按**声明所在模块**计算——搬进 `app/state.rs` 之后，同级的 `app/keyboard.rs` 就拿不到私有字段了。所以 `app.rs` 的角色是「共享可变状态的持有者」，子模块只是行为的分区。

新增一个页面仍然是 3 处：`app/page.rs`（变体 + 名字 + 图标）、`app/dispatch.rs`（`PageView` 实现 + `page_view` 分支）、一个新模块。新增一个换算器分类只需 3 处，且菜单、标题栏、存档读写、页面分发都自动跟上。
