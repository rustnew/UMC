//! Persistance de l'historique de conversion (JSON dans le répertoire de données).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Une entrée d'historique de conversion.
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

/// Historique complet, persisté en JSON.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct History {
    pub entries: Vec<HistoryEntry>,
}

impl History {
    /// Répertoire de données de l'application (~/.local/share/umc ou équivalent).
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

    /// Charge l'historique depuis le disque (vide si absent).
    pub fn load() -> Self {
        let path = Self::history_path();
        match std::fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Sauvegarde l'historique sur le disque.
    pub fn save(&self) {
        let path = Self::history_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(raw) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, raw);
        }
    }

    /// Ajoute une entrée en tête de liste (la plus récente d'abord), bornée à 200.
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
