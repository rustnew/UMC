//! Écran « Réglages » — préférences persistées.

use eframe::egui;
use egui::RichText;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Réglages persistés de l'application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsState {
    pub theme_dark: bool,
    pub default_threads: usize,
    pub default_validation: String,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            theme_dark: true,
            default_threads: 0,
            default_validation: "structural".into(),
        }
    }
}

impl SettingsState {
    fn path() -> PathBuf {
        crate::history::History::data_dir().join("settings.json")
    }

    pub fn load() -> Self {
        match std::fs::read_to_string(Self::path()) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(raw) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, raw);
        }
    }
}

pub fn show(ui: &mut egui::Ui, settings: &mut SettingsState, theme_dark: &mut bool) {
    let mut changed = false;

    egui::CentralPanel::default().show(ui, |ui| {
        ui.add_space(6.0);
        ui.heading("Réglages");
        ui.add_space(12.0);

        ui.group(|ui| {
            ui.label(RichText::new("Apparence").strong());
            ui.add_space(4.0);
            if ui.selectable_label(*theme_dark, "Thème sombre").clicked() {
                *theme_dark = true;
                settings.theme_dark = true;
                changed = true;
            }
            if ui.selectable_label(!*theme_dark, "Thème clair").clicked() {
                *theme_dark = false;
                settings.theme_dark = false;
                changed = true;
            }
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label(RichText::new("Conversion").strong());
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Threads par défaut :");
                if ui
                    .selectable_label(settings.default_threads == 0, "Auto")
                    .clicked()
                {
                    settings.default_threads = 0;
                    changed = true;
                }
                for n in [1usize, 2, 4, 8, 16] {
                    let selected = settings.default_threads == n;
                    if ui.selectable_label(selected, n.to_string()).clicked() {
                        settings.default_threads = if selected { 0 } else { n };
                        changed = true;
                    }
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Validation par défaut :");
                for (label, val) in [
                    ("Aucune", "none"),
                    ("Structurelle", "structural"),
                    ("Numérique", "numeric"),
                    ("Stricte", "strict"),
                ] {
                    let selected = settings.default_validation == val;
                    if ui.selectable_label(selected, label).clicked() {
                        settings.default_validation = val.into();
                        changed = true;
                    }
                }
            });
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label(RichText::new("Données").strong());
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "Historique et réglages : {}",
                    crate::history::History::data_dir().display()
                ))
                .weak()
                .size(11.0),
            );
        });

        if changed {
            settings.save();
        }
    });
}
