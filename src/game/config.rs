/// Pet configuration — saved to config/config.json
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetConfig {
    pub pet_width: u32,
    pub pet_height: u32,
    pub walk_speed: f32,
    pub image_path: String,
    pub frames_per_row: u32,
    pub rows: u32,
    pub anim_fps: f32,
    pub feed_affection_gain: u32,
    pub affection_decay_per_hour: u32,
}

impl Default for PetConfig {
    fn default() -> Self {
        Self {
            pet_width: 128,
            pet_height: 128,
            walk_speed: 60.0,
            image_path: "assets/pet.png".into(),
            frames_per_row: 4,
            rows: 7,
            anim_fps: 8.0,
            feed_affection_gain: 5,
            affection_decay_per_hour: 1,
        }
    }
}

impl PetConfig {
    fn config_path() -> std::path::PathBuf {
        std::env::current_dir()
            .unwrap_or_default()
            .join("config")
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let mut data = std::fs::read_to_string(&path).unwrap_or_default();
            // Strip UTF-8 BOM if present (PowerShell Set-Content adds it)
            if data.starts_with('\u{feff}') {
                data = data[3..].to_string();
            }
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            let config = Self::default();
            let _ = config.save();
            config
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(path, json)
    }
}
