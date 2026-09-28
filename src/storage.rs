use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEntry {
    pub path: String,
    pub display_name: String,
    pub accessed: String, // ISO timestamp
}

/// One day of quiet writing statistics, as sketched in `PLAN.txt`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SessionStats {
    /// Local date, `YYYY-MM-DD`.
    pub date: String,
    /// Words added — the sum of the document's word count growing.
    pub words: usize,
    /// Seconds spent with a document open.
    pub seconds: u64,
    /// Files opened today, so the same document is never counted twice.
    pub files: Vec<String>,
}

/// Days of history kept before the oldest is dropped.
const SESSION_LOG_DAYS: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Storage {
    pub recent_files: Vec<RecentEntry>,
    pub last_session_file: Option<String>,
    pub session_log: Vec<SessionStats>,
}

/// Today's local date, the key the session log is rolled over on.
pub fn today_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// A quiet label for a day: `today`, `yesterday`, or `MM-DD`.
pub fn day_label(date: &str) -> String {
    let today = chrono::Local::now().date_naive();
    match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(parsed) if parsed == today => "today".to_string(),
        Ok(parsed) if parsed == today.pred_opt().unwrap_or(today) => "yesterday".to_string(),
        Ok(parsed) => parsed.format("%m-%d").to_string(),
        Err(_) => date.to_string(),
    }
}

impl Storage {
    fn storage_path() -> Option<PathBuf> {
        dirs::data_dir().map(|p| p.join("microwriter").join("storage.json"))
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
        // Write beside the target and rename over it. The statistics are now
        // flushed while the app runs, so an interrupted write must not be able
        // to leave a half-written storage.json behind.
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, content)?;
        fs::rename(&tmp, &path)
    }

    /// Today's record, created (and rolled over) as needed.
    pub fn today(&mut self) -> &mut SessionStats {
        let today = today_date();
        if self
            .session_log
            .last()
            .is_none_or(|entry| entry.date != today)
        {
            self.session_log.push(SessionStats {
                date: today,
                ..SessionStats::default()
            });
            let excess = self.session_log.len().saturating_sub(SESSION_LOG_DAYS);
            self.session_log.drain(..excess);
        }
        let last = self.session_log.len() - 1;
        &mut self.session_log[last]
    }

    /// Today's record, without creating one if the day has no writing yet.
    pub fn today_stats(&self) -> Option<&SessionStats> {
        let today = today_date();
        self.session_log.iter().rev().find(|e| e.date == today)
    }

    /// Note that a document was opened today, counting it once.
    pub fn track_document(&mut self, path: &str) {
        let stats = self.today();
        if !stats.files.iter().any(|f| f == path) {
            stats.files.push(path.to_string());
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn today_record_is_created_once_and_reused() {
        let mut storage = Storage::default();
        assert!(storage.today_stats().is_none());

        storage.today().words = 10;
        storage.today().words += 5;
        assert_eq!(storage.today_stats().map(|stats| stats.words), Some(15));
        assert_eq!(storage.session_log.len(), 1);
    }

    #[test]
    fn an_earlier_day_is_left_untouched() {
        let mut storage = Storage::default();
        storage.session_log.push(SessionStats {
            date: "1999-01-01".to_string(),
            words: 99,
            ..SessionStats::default()
        });

        storage.today().words = 7;
        assert_eq!(storage.today_stats().map(|stats| stats.words), Some(7));
        assert_eq!(storage.session_log.len(), 2);
        assert_eq!(storage.session_log[0].words, 99);
    }

    #[test]
    fn days_are_labelled_by_how_recent_they_are() {
        let today = chrono::Local::now().date_naive();
        assert_eq!(day_label(&today.format("%Y-%m-%d").to_string()), "today");

        let yesterday = today.pred_opt().unwrap_or(today);
        assert_eq!(
            day_label(&yesterday.format("%Y-%m-%d").to_string()),
            "yesterday"
        );

        let older = today - chrono::Days::new(5);
        assert_eq!(
            day_label(&older.format("%Y-%m-%d").to_string()),
            older.format("%m-%d").to_string()
        );
        assert_eq!(day_label("not a date"), "not a date");
    }

    #[test]
    fn documents_are_counted_once_each() {
        let mut storage = Storage::default();
        storage.track_document("/notes/a.txt");
        storage.track_document("/notes/b.txt");
        storage.track_document("/notes/a.txt");
        assert_eq!(
            storage.today_stats().map(|stats| stats.files.len()),
            Some(2)
        );
    }

    #[test]
    fn the_session_log_is_trimmed() {
        let mut storage = Storage::default();
        for day in 0..(SESSION_LOG_DAYS + 5) {
            storage.session_log.push(SessionStats {
                date: format!("2000-01-{day:02}"),
                ..SessionStats::default()
            });
        }
        // Loading a fresh day trims the log back to the retained window.
        storage.today();
        assert_eq!(storage.session_log.len(), SESSION_LOG_DAYS);
    }
}
