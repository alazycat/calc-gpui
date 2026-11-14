//! 本地持久化：历史记录、记忆槽、主题偏好和窗口尺寸。
//!
//! 保存到用户配置目录下的 `calc-gpui/state.json`：
//! - Windows：`%APPDATA%\calc-gpui\state.json`
//! - 其它：`$XDG_CONFIG_HOME/calc-gpui/state.json` 或 `~/.config/calc-gpui/state.json`
//!
//! 读写失败时静默忽略，计算器本身仍然可用。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 一条历史记录。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SavedEntry {
    pub expression: String,
    pub result: String,
}

/// 需要跨启动保留的状态。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedState {
    pub standard_history: Vec<SavedEntry>,
    pub standard_memories: Vec<f64>,
    pub scientific_history: Vec<SavedEntry>,
    pub scientific_memories: Vec<f64>,
    pub programmer_memories: Vec<u64>,
    /// `light` / `dark` / `system`
    pub theme: Option<String>,
    /// 上次所在的页面，例如 `standard` / `length`
    pub page: Option<String>,
    pub window_width: Option<f32>,
    pub window_height: Option<f32>,
}

impl SavedState {
    /// 恢复窗口尺寸，并限制在合理范围内，避免上次最大化导致下次窗口超出屏幕。
    pub fn window_size(&self) -> (f32, f32) {
        let width = self.window_width.unwrap_or(400.0).clamp(360.0, 900.0);
        let height = self.window_height.unwrap_or(600.0).clamp(520.0, 1100.0);
        (width, height)
    }
}

/// 状态文件路径。
pub fn config_path() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
    }?;
    Some(base.join("calc-gpui").join("state.json"))
}

/// 读取上次保存的状态；文件不存在或损坏时返回默认值。
pub fn load() -> SavedState {
    let Some(path) = config_path() else {
        return SavedState::default();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return SavedState::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

/// 写入状态：先写临时文件再重命名，避免中途退出留下半个 JSON。
pub fn save(state: &SavedState) {
    let Some(path) = config_path() else {
        return;
    };
    if let Some(parent) = path.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return;
    }
    let Ok(text) = serde_json::to_string_pretty(state) else {
        return;
    };
    let temp = path.with_extension("json.tmp");
    if fs::write(&temp, text).is_ok() {
        let _ = fs::rename(&temp, &path);
    }
}

#[cfg(test)]
mod tests {
    use super::{SavedEntry, SavedState};

    #[test]
    fn state_roundtrips_through_json() {
        let state = SavedState {
            standard_history: vec![SavedEntry {
                expression: "2 + 3 =".into(),
                result: "5".into(),
            }],
            standard_memories: vec![1.5, 2.0],
            scientific_memories: vec![3.5],
            programmer_memories: vec![0xF0],
            theme: Some("dark".into()),
            page: Some("graphing".into()),
            window_width: Some(420.0),
            window_height: Some(640.0),
            ..Default::default()
        };
        let text = serde_json::to_string(&state).unwrap();
        let parsed: SavedState = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed, state);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let parsed: SavedState = serde_json::from_str("{\"theme\":\"dark\"}").unwrap();
        assert_eq!(parsed.theme.as_deref(), Some("dark"));
        assert!(parsed.standard_memories.is_empty());
    }

    #[test]
    fn window_size_is_clamped() {
        let mut state = SavedState::default();
        assert_eq!(state.window_size(), (400.0, 600.0));

        state.window_width = Some(5000.0);
        state.window_height = Some(100.0);
        assert_eq!(state.window_size(), (900.0, 520.0));
    }
}
