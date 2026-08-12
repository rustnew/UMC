//! Persistence of the conversion history (JSON in the data directory).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A conversion history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub input: PathBuf,
    pub output: PathBuf,
    pub source_format: String,
    pub target_format: String,
    pub elapsed_ms: u64,
    pub tensor_count: usize,
    pub status: String, // "ok" | "error"
    pub message: String,
}

/// Full history, persisted as JSON.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct History {
    pub entries: Vec<HistoryEntry>,
}

impl History {
    /// Application data directory (~/.local/share/umc or equivalent).
    pub fn data_dir() -> PathBuf {
        if let Some(dirs) = directories::ProjectDirs::from("", "", "umc") {
            dirs.data_dir().to_path_buf()
        } else {
            PathBuf::from(".")
        }
    }

    pub fn history_path() -> PathBuf {
        Self::data_dir().join("history.json")
    }

    /// Loads the history from disk (empty if absent).
    pub fn load() -> Self {
        let path = Self::history_path();
        match std::fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Saves the history to disk.
    pub fn save(&self) {
        let path = Self::history_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(raw) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, raw);
        }
    }

    /// Adds an entry at the head of the list (most recent first), capped at 200.
    pub fn push(&mut self, entry: HistoryEntry) {
        self.entries.insert(0, entry);
        self.entries.truncate(200);
        self.save();
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.save();
    }
}
