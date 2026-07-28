use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEntry {
    pub path: String,
    pub display_name: String,
    pub accessed: String, // ISO timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub date: String,
    pub total_words: usize,
    pub total_minutes: usize,
    pub documents: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Storage {
    pub recent_files: Vec<RecentEntry>,
    pub last_session_file: Option<String>,
    pub session_log: Vec<SessionStats>,
}

impl Storage {
    fn storage_path() -> Option<PathBuf> {
        dirs::data_dir().map(|p| p.join("mute").join("storage.json"))
    }

    pub fn load() -> Result<Self, String> {
        let path = Self::storage_path().ok_or("No data directory found")?;
        if path.exists() {
            let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            serde_json::from_str(&content).map_err(|e| e.to_string())
        } else {
            Ok(Storage::default())
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::storage_path().ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No data directory",
        ))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)
    }

    pub fn track_file(&mut self, path: &str) {
        let display_name = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        // Remove existing entry
        self.recent_files.retain(|r| r.path != path);

        self.recent_files.insert(
            0,
            RecentEntry {
                path: path.to_string(),
                display_name,
                accessed: chrono::Utc::now().to_rfc3339(),
            },
        );

        // Keep only last 50
        self.recent_files.truncate(50);
    }
}
