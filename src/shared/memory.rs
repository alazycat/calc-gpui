//! 共享的记忆槽：MS / MR / MC / M+ / M− 背后的存储与面板开关。
//!
//! 三个模式的记忆差别只有两点：
//!
//! 1. 值从哪来（各自从显示区读，属于模式自己的事）；
//! 2. M+ / M− 怎么算（浮点直接加减；程序员模式要按位宽截断，用 `wrapping_*`）。
//!
//! 其余规则完全一样，所以收在这里：`MS` 追加一个槽、`MC` 清空并收起面板、
//! `MR` 取最新的槽、记忆列表为空时不允许打开面板。
//!
//! 求值逻辑不在这里，也不在这里格式化数字——调用方拿到 `slots()` 自己决定怎么显示。

/// 一组记忆槽。
///
/// 「槽」是计算器上每次 MS 新建的一格；点击记忆面板里的某一行取回的就是其中一格。
pub struct Memory<T> {
    /// 已存入的值，最新一个在末尾。
    slots: Vec<T>,
    /// 是否正在显示记忆列表。只有带记忆面板的模式会真正用到。
    open: bool,
}

impl<T> Memory<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            open: false,
        }
    }
}

impl<T> Default for Memory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy> Memory<T> {
    /// 全部槽位，最新一个在末尾。渲染记忆列表时通常倒序使用。
    pub fn slots(&self) -> &[T] {
        &self.slots
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// MS：新开一格存值。
    pub fn store(&mut self, value: T) {
        self.slots.push(value);
    }

    /// MC：清空所有槽，并收起面板。
    pub fn clear(&mut self) {
        self.slots.clear();
        self.open = false;
    }

    /// MR：最新一格的値。
    pub fn latest(&self) -> Option<T> {
        self.slots.last().copied()
    }

    /// 指定槽位的值（记忆面板点击某一行）。
    pub fn get(&self, index: usize) -> Option<T> {
        self.slots.get(index).copied()
    }

    /// M+ / M−：把 `update` 作用到最新一格上；没有任何槽时写入 `seed`。
    ///
    /// - M+：`update_latest(value, |last| last + value)`
    /// - M−：`update_latest(-value, |last| last - value)`
    ///
    /// 程序员模式在这里传入带位宽截断的闭包。
    pub fn update_latest(&mut self, seed: T, update: impl FnOnce(T) -> T) {
        match self.slots.last_mut() {
            Some(last) => *last = update(*last),
            None => self.slots.push(seed),
        }
    }

    /// 面板当前是否展开。
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// 展开面板（用于恢复或测试）。
    pub fn open(&mut self) {
        self.open = true;
    }

    /// 收起面板。
    pub fn close(&mut self) {
        self.open = false;
    }

    /// 在「计算器」和「记忆列表」之间切换。
    ///
    /// 没有任何记忆时不响应；返回值是切换后是否展开，调用方据此收起历史面板。
    pub fn toggle_panel(&mut self) -> bool {
        if self.slots.is_empty() {
            return false;
        }
        if self.open {
            self.close();
        } else {
            self.open();
        }
        self.open
    }
}

impl<T: Clone> Memory<T> {
    /// 复制一份槽位内容（存档用）。
    pub fn to_vec(&self) -> Vec<T> {
        self.slots.clone()
    }
}

impl<T> Memory<T> {
    /// 用存档里的槽位替换当前内容（读档用）。
    pub fn replace(&mut self, slots: Vec<T>) {
        self.slots = slots;
    }
}

#[cfg(test)]
mod tests {
    use super::Memory;

    #[test]
    fn store_latest_and_get() {
        let mut memory = Memory::new();
        assert_eq!(memory.latest(), None);
        memory.store(1.0);
        memory.store(2.0);
        assert_eq!(memory.latest(), Some(2.0));
        assert_eq!(memory.get(0), Some(1.0));
        assert_eq!(memory.get(9), None);
        assert_eq!(memory.slots(), &[1.0, 2.0]);
    }

    #[test]
    fn update_latest_seeds_when_empty() {
        let mut memory = Memory::new();
        memory.update_latest(5.0, |last| last + 1.0);
        assert_eq!(memory.slots(), &[5.0]);
        memory.update_latest(99.0, |last| last + 1.0);
        assert_eq!(memory.slots(), &[6.0]);
    }

    #[test]
    fn clear_empties_slots_and_closes_panel() {
        let mut memory = Memory::new();
        memory.store(1.0);
        memory.open();
        memory.clear();
        assert!(memory.is_empty());
        assert!(!memory.is_open());
    }

    #[test]
    fn panel_does_not_open_without_slots() {
        let mut memory = Memory::new();
        assert!(!memory.toggle_panel());
        assert!(!memory.is_open());

        memory.store(1.0);
        assert!(memory.toggle_panel());
        assert!(memory.is_open());
        assert!(!memory.toggle_panel());
        assert!(!memory.is_open());
    }

    #[test]
    fn replace_and_to_vec_round_trip() {
        let mut memory = Memory::new();
        memory.replace(vec![3, 4]);
        assert_eq!(memory.to_vec(), vec![3, 4]);
    }
}
