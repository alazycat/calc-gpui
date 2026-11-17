//! 计算历史：标准与科学模式共用的条目结构与容量上限。
//!
//! 两个模式只在「何时记录」上不同（标准模式要求算式含运算符，科学模式直接记录），
//! 条目结构与裁剪规则完全一致，因此收在这里，避免上限常量各写一份后漂移。

/// 历史记录最多保留的条数。
pub const MAX_HISTORY: usize = 50;

/// 一条计算历史。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HistoryEntry {
    /// 带等号的算式，例如 `2 + 3 × 4 =`。
    pub expression: String,
    /// 计算结果。
    pub result: String,
}

/// 追加一条历史；超过上限时丢弃最旧的一条。
pub fn push(entries: &mut Vec<HistoryEntry>, entry: HistoryEntry) {
    entries.push(entry);
    if entries.len() > MAX_HISTORY {
        entries.remove(0);
    }
}

/// 从存档恢复时只保留最近的 [`MAX_HISTORY`] 条。
pub fn recent<T>(entries: &[T]) -> &[T] {
    let start = entries.len().saturating_sub(MAX_HISTORY);
    &entries[start..]
}

#[cfg(test)]
mod tests {
    use super::{HistoryEntry, MAX_HISTORY, push, recent};

    fn entry(index: usize) -> HistoryEntry {
        HistoryEntry {
            expression: format!("{index} + 1 ="),
            result: (index + 1).to_string(),
        }
    }

    #[test]
    fn push_keeps_only_latest_entries() {
        let mut entries = Vec::new();
        for index in 0..MAX_HISTORY + 5 {
            push(&mut entries, entry(index));
        }

        assert_eq!(entries.len(), MAX_HISTORY);
        assert_eq!(entries.first().unwrap().expression, "5 + 1 =");
        assert_eq!(
            entries.last().unwrap().expression,
            format!("{} + 1 =", MAX_HISTORY + 4)
        );
    }

    #[test]
    fn recent_drops_oldest_entries() {
        let saved: Vec<HistoryEntry> = (0..MAX_HISTORY + 3).map(entry).collect();
        let trimmed = recent(&saved);

        assert_eq!(trimmed.len(), MAX_HISTORY);
        assert_eq!(trimmed.first().unwrap().expression, "3 + 1 =");
    }

    #[test]
    fn recent_keeps_short_history_unchanged() {
        let saved = vec![entry(0), entry(1)];
        assert_eq!(recent(&saved), &saved[..]);
    }
}
