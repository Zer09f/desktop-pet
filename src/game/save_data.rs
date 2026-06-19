/// Save data persistence
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub affection: u32,
    pub total_feeds: u32,
    pub total_seconds: u64,
    pub last_save_timestamp: u64,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            affection: 0,
            total_feeds: 0,
            total_seconds: 0,
            last_save_timestamp: 0,
        }
    }
}

impl SaveData {
    fn save_path() -> PathBuf {
        std::env::current_dir()
            .unwrap_or_default()
            .join("data")
            .join("save.json")
    }

    pub fn load() -> Self {
        let path = Self::save_path();
        if path.exists() {
            let data = std::fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        self.last_save_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let path = Self::save_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(path, json)
    }
}
