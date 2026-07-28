use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    pub cursor_style: String,
    pub line_numbers: String,
    pub wrap: bool,
    pub autosave: String,
    pub default_folder: String,
    pub timestamp_filenames: bool,
    pub use_tabs: bool,
    pub tab_spaces: usize,
    pub show_status: bool,
    pub startup_behavior: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            cursor_style: "block".to_string(),
            line_numbers: "off".to_string(),
            wrap: true,
            autosave: "disabled".to_string(),
            default_folder: String::new(),
            timestamp_filenames: false,
            use_tabs: false,
            tab_spaces: 4,
            show_status: false,
            startup_behavior: "menu".to_string(),
        }
    }
}

impl Config {
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("mute").join("config.toml"))
    }

    pub fn load() -> Result<Self, String> {
        let path = Self::config_path().ok_or("No config directory found")?;
        if path.exists() {
            let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            toml::from_str(&content).map_err(|e| e.to_string())
        } else {
            let config = Config::default();
            config.save().map_err(|e| e.to_string())?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path().ok_or("No config directory found")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())
    }
}
