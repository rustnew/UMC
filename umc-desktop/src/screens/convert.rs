//! Écran « Convertir » — cœur de l'application.
//!
//! Sélection du fichier (glisser-déposer ou dialogue natif), détection
//! automatique du format, choix de la cible, options, progression, annulation.

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;
use egui::{Color32, RichText};

use umc_core::DType;
use umc_detect::FormatRegistry;
use umc_validate::ValidationMode;

use crate::history::{History, HistoryEntry};
use crate::worker::{self, ConvertOptions, WorkerEvent};

/// État de l'écran de conversion.
pub struct ConvertState {
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
    /// Buffer éditable du champ « Sortie » (vide = chemin par défaut).
    pub output_text: String,

    /// Format détecté automatiquement (affiché à l'utilisateur).
    pub detected_format: Option<String>,
    pub detected_confidence: f32,

    /// Format source forcé (None = auto).
    pub source_override: Option<String>,
    /// Format cible (None = inféré de l'extension de sortie).
    pub target_override: Option<String>,

    pub dtype_override: Option<DType>,
    pub validation_mode: ValidationMode,
    pub threads: usize,

    // ── Worker ──
    pub running: bool,
    pub cancel_token: Option<Arc<umc_pipeline::CancellationToken>>,
    pub progress_done: u64,
    pub progress_total: u64,
    pub progress_msg: String,

    // ── Résultat ──
    pub result: Option<WorkerEvent>,
    pub last_error: Option<String>,

    /// Canal de réception des événements du worker (None si inactif).
    rx: Option<std::sync::mpsc::Receiver<WorkerEvent>>,
}

impl Default for ConvertState {
    fn default() -> Self {
        Self {
            input: None,
            output: None,
            output_text: String::new(),
            detected_format: None,
            detected_confidence: 0.0,
            source_override: None,
            target_override: None,
            dtype_override: None,
            validation_mode: ValidationMode::Structural,
            threads: 0, // 0 = auto
            running: false,
            cancel_token: None,
            progress_done: 0,
            progress_total: 0,
            progress_msg: String::new(),
            result: None,
            last_error: None,
            rx: None,
        }
    }
}

impl ConvertState {
    /// Détecte le format du fichier sélectionné.
    fn detect(&mut self) {
        self.detected_format = None;
        self.detected_confidence = 0.0;
        if let Some(path) = &self.input {
            let registry = FormatRegistry::new();
            if let Ok(res) = registry.detect(path) {
                self.detected_format = Some(res.format);
                self.detected_confidence = res.confidence;
            }
        }
    }

    /// Construit le chemin de sortie par défaut (même dossier, extension cible).
    fn default_output(&self) -> Option<PathBuf> {
        let input = self.input.as_ref()?;
        let ext = self
            .target_override
            .as_deref()
            .map(|f| f.to_lowercase())
            .unwrap_or_else(|| "gguf".to_string());
        let stem = input.file_stem()?.to_string_lossy();
        Some(input.with_file_name(format!("{stem}.{ext}")))
    }

    fn start(&mut self) {
        let Some(input) = self.input.clone() else {
            return;
        };
        let output = if self.output_text.trim().is_empty() {
            self.default_output()
                .unwrap_or_else(|| PathBuf::from("output.gguf"))
        } else {
            PathBuf::from(self.output_text.trim())
        };

        let opts = ConvertOptions {
            input,
            output,
            source_format: self.source_override.clone(),
            target_format: self.target_override.clone(),
            dtype: self.dtype_override.clone(),
            validation_mode: self.validation_mode.clone(),
            threads: self.threads,
        };

        let (token, rx) = worker::spawn_conversion(opts);
        self.cancel_token = Some(token);
        self.running = true;
        self.result = None;
        self.last_error = None;
        self.progress_done = 0;
        self.progress_total = 0;
        self.progress_msg = "Démarrage…".into();

        // Poll du canal à chaque frame.
        self.rx = Some(rx);
    }

    fn cancel(&mut self) {
        if let Some(tok) = &self.cancel_token {
            tok.cancel();
        }
    }

    fn poll(&mut self, history: &mut History) {
        // On draine le canal dans un vecteur d'abord (pour libérer l'emprunt
        // de `self.rx` avant de muter `self`).
        let events: Vec<WorkerEvent> = {
            let Some(rx) = &self.rx else { return };
            std::iter::from_fn(|| rx.try_recv().ok()).collect()
        };

        for ev in events {
            match ev {
                WorkerEvent::Progress(done, total, msg) => {
                    self.progress_done = done;
                    self.progress_total = total;
                    self.progress_msg = msg;
                }
                WorkerEvent::Done {
                    source_format,
                    target_format,
                    elapsed_ms,
                    tensor_count,
                    output,
                    warnings,
                } => {
                    self.running = false;
                    self.cancel_token = None;
                    self.rx = None;
                    self.result = Some(WorkerEvent::Done {
                        source_format: source_format.clone(),
                        target_format: target_format.clone(),
                        elapsed_ms,
                        tensor_count,
                        output: output.clone(),
                        warnings: warnings.clone(),
                    });
                    history.push(HistoryEntry {
                        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        input: self.input.clone().unwrap_or_default(),
                        output: output.clone(),
                        source_format: source_format.clone(),
                        target_format: target_format.clone(),
                        elapsed_ms,
                        tensor_count,
                        status: "ok".into(),
                        message: if warnings.is_empty() {
                            "Conversion réussie".into()
                        } else {
                            format!("{} avertissement(s)", warnings.len())
                        },
                    });
                }
                WorkerEvent::Error(e) => {
                    self.running = false;
                    self.cancel_token = None;
                    self.rx = None;
                    self.last_error = Some(e.clone());
                    history.push(HistoryEntry {
                        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        input: self.input.clone().unwrap_or_default(),
                        output: self.output.clone().unwrap_or_default(),
                        source_format: self.detected_format.clone().unwrap_or_else(|| "?".into()),
                        target_format: self.target_override.clone().unwrap_or_else(|| "?".into()),
                        elapsed_ms: 0,
                        tensor_count: 0,
                        status: "error".into(),
                        message: e,
                    });
                }
            }
        }
    }
}

/// Affiche l'écran de conversion.
pub fn show(ui: &mut egui::Ui, state: &mut ConvertState, history: &mut History) {
    state.poll(history);

    egui::CentralPanel::default().show(ui, |ui| {
        ui.add_space(6.0);
        ui.heading("Convertir un modèle");
        ui.label(
            RichText::new("Convertissez un fichier de modèle entre formats supportés.").weak(),
        );
        ui.add_space(12.0);

        // ── Zone de dépôt / sélection ─────────────────────────────────────
        let dropped = ui.ctx().input(|i| i.raw.dropped_files.clone());
        if let Some(file) = dropped.into_iter().next() {
            let path = file.path().to_path_buf();
            state.input = Some(path);
            state.output = None;
            state.output_text.clear();
            state.detect();
        }

        ui.group(|ui| {
            ui.set_min_size(egui::vec2(ui.available_width(), 90.0));
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                match &state.input {
                    Some(path) => {
                        ui.label(
                            RichText::new(path.file_name().unwrap_or_default().to_string_lossy())
                                .size(16.0)
                                .strong(),
                        );
                        ui.label(RichText::new(path.display().to_string()).weak().size(11.0));
                        if let Some(fmt) = &state.detected_format {
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(format!(
                                    "Format détecté : {fmt} (confiance {:.0}%)",
                                    state.detected_confidence * 100.0
                                ))
                                .color(Color32::from_rgb(0x4f, 0x9d, 0xe9)),
                            );
                        } else {
                            ui.label(
                                RichText::new("Format non reconnu — précisez-le manuellement.")
                                    .color(Color32::from_rgb(0xe0, 0x8a, 0x3c)),
                            );
                        }
                    }
                    None => {
                        ui.label(
                            RichText::new("Glissez-déposez un fichier de modèle ici")
                                .size(16.0)
                                .weak(),
                        );
                        ui.label(RichText::new("ou").weak());
                    }
                }
                ui.add_space(6.0);
                if ui.button("Parcourir…").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter(
                            "Modèles",
                            &[
                                "gguf",
                                "safetensors",
                                "bin",
                                "pt",
                                "pth",
                                "onnx",
                                "tflite",
                                "h5",
                                "ggml",
                                "model",
                            ],
                        )
                        .pick_file()
                    {
                        state.input = Some(path);
                        state.output = None;
                        state.output_text.clear();
                        state.detect();
                    }
                }
                ui.add_space(8.0);
            });
        });

        ui.add_space(12.0);

        // ── Options ───────────────────────────────────────────────────────
        ui.collapsing("Options de conversion", |ui| {
            ui.add_space(4.0);

            // Format source
            ui.horizontal(|ui| {
                ui.label("Format source :");
                if ui
                    .selectable_label(state.source_override.is_none(), "Auto")
                    .clicked()
                {
                    state.source_override = None;
                }
                let formats = FormatRegistry::new().format_names();
                for f in formats {
                    let selected = state.source_override.as_deref() == Some(f);
                    if ui.selectable_label(selected, f).clicked() {
                        state.source_override = if selected { None } else { Some(f.to_string()) };
                    }
                }
            });

            ui.add_space(6.0);

            // Format cible
            ui.horizontal(|ui| {
                ui.label("Format cible :");
                if ui
                    .selectable_label(state.target_override.is_none(), "Auto (extension)")
                    .clicked()
                {
                    state.target_override = None;
                }
                let formats = FormatRegistry::new().format_names();
                for f in formats {
                    let selected = state.target_override.as_deref() == Some(f);
                    if ui.selectable_label(selected, f).clicked() {
                        state.target_override = if selected { None } else { Some(f.to_string()) };
                    }
                }
            });

            ui.add_space(6.0);

            // DType
            ui.horizontal(|ui| {
                ui.label("DType cible :");
                if ui
                    .selectable_label(state.dtype_override.is_none(), "Conserver")
                    .clicked()
                {
                    state.dtype_override = None;
                }
                let dtypes = [
                    DType::F32,
                    DType::F16,
                    DType::BF16,
                    DType::F8E4M3,
                    DType::Q8_0,
                    DType::Q6K,
                    DType::Q5KM,
                    DType::Q4KM,
                    DType::Q4_0,
                    DType::Q4_1,
                    DType::NF4,
                    DType::FP4,
                ];
                for dt in dtypes {
                    let selected = state.dtype_override.as_ref() == Some(&dt);
                    if ui.selectable_label(selected, dt.as_str()).clicked() {
                        state.dtype_override = if selected { None } else { Some(dt.clone()) };
                    }
                }
            });

            ui.add_space(6.0);

            // Validation
            ui.horizontal(|ui| {
                ui.label("Validation :");
                let modes = [
                    ("Aucune", ValidationMode::None),
                    ("Structurelle", ValidationMode::Structural),
                    ("Numérique", ValidationMode::Numeric),
                    ("Stricte", ValidationMode::Strict),
                ];
                for (label, mode) in modes {
                    let selected = state.validation_mode == mode;
                    if ui.selectable_label(selected, label).clicked() {
                        state.validation_mode = mode;
                    }
                }
            });

            ui.add_space(6.0);

            // Threads
            ui.horizontal(|ui| {
                ui.label("Threads :");
                if ui.selectable_label(state.threads == 0, "Auto").clicked() {
                    state.threads = 0;
                }
                for n in [1usize, 2, 4, 8, 16] {
                    let selected = state.threads == n;
                    if ui.selectable_label(selected, n.to_string()).clicked() {
                        state.threads = if selected { 0 } else { n };
                    }
                }
            });
        });

        ui.add_space(12.0);

        // ── Sortie ────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Sortie :");
            let width = (ui.available_width() - 40.0).max(120.0);
            ui.add(
                egui::TextEdit::singleline(&mut state.output_text)
                    .hint_text("(par défaut : à côté de l'entrée)")
                    .desired_width(width),
            );
            if ui.button("…").clicked() {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    state.output_text = path.display().to_string();
                    state.output = Some(path);
                }
            }
        });
        if state.output_text.trim().is_empty() {
            if let Some(def) = state.default_output() {
                ui.label(
                    RichText::new(format!("Par défaut : {}", def.display()))
                        .weak()
                        .size(11.0),
                );
            }
        }

        ui.add_space(16.0);

        // ── Progression / résultat ────────────────────────────────────────
        if state.running {
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.label(RichText::new("Conversion en cours…").strong());
                ui.add_space(4.0);
                let progress = if state.progress_total > 0 {
                    state.progress_done as f32 / state.progress_total as f32
                } else {
                    0.0
                };
                ui.add(
                    egui::ProgressBar::new(progress)
                        .show_percentage()
                        .desired_height(18.0),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!(
                        "{}  —  {}/{} tenseurs",
                        state.progress_msg, state.progress_done, state.progress_total
                    ))
                    .weak(),
                );
                ui.add_space(6.0);
                if ui
                    .button(RichText::new("Annuler").color(Color32::from_rgb(0xe0, 0x5a, 0x5a)))
                    .clicked()
                {
                    state.cancel();
                }
            });
        } else if let Some(WorkerEvent::Done {
            source_format,
            target_format,
            elapsed_ms,
            tensor_count,
            output,
            warnings,
        }) = &state.result
        {
            let source_format = source_format.clone();
            let target_format = target_format.clone();
            let elapsed_ms = *elapsed_ms;
            let tensor_count = *tensor_count;
            let output = output.clone();
            let warnings = warnings.clone();
            ui.group(|ui| {
                ui.label(
                    RichText::new("✓ Conversion terminée")
                        .strong()
                        .color(Color32::from_rgb(0x4c, 0xaf, 0x50)),
                );
                ui.add_space(4.0);
                ui.label(format!(
                    "{source_format} → {target_format}  |  {tensor_count} tenseurs  |  {:.1}s",
                    elapsed_ms as f64 / 1000.0
                ));
                ui.label(RichText::new(output.display().to_string()).weak());
                if !warnings.is_empty() {
                    ui.add_space(4.0);
                    for w in warnings {
                        ui.label(
                            RichText::new(format!("⚠ {w}"))
                                .color(Color32::from_rgb(0xe0, 0xa0, 0x3c)),
                        );
                    }
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Ouvrir le dossier").clicked() {
                        if let Some(dir) = output.parent() {
                            let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
                        }
                    }
                    if ui.button("Nouvelle conversion").clicked() {
                        *state = ConvertState::default();
                    }
                });
            });
        } else if let Some(err) = &state.last_error {
            let err = err.clone();
            ui.group(|ui| {
                ui.label(
                    RichText::new("✗ Échec de la conversion")
                        .strong()
                        .color(Color32::from_rgb(0xe0, 0x5a, 0x5a)),
                );
                ui.add_space(4.0);
                ui.label(RichText::new(err).weak());
                ui.add_space(6.0);
                if ui.button("Réessayer").clicked() {
                    state.last_error = None;
                    state.start();
                }
            });
        } else {
            ui.label(
                RichText::new("Sélectionnez un fichier puis lancez la conversion.")
                    .weak()
                    .italics(),
            );
        }

        ui.add_space(12.0);

        // ── Bouton principal ──────────────────────────────────────────────
        let can_start = state.input.is_some() && !state.running;
        let btn = egui::Button::new(RichText::new("▶  Convertir").size(16.0).strong())
            .fill(Color32::from_rgb(0x4f, 0x9d, 0xe9))
            .corner_radius(8.0)
            .min_size(egui::vec2(ui.available_width(), 42.0));
        if ui.add_sized([ui.available_width(), 42.0], btn).clicked() && can_start {
            state.start();
        }
    });
}
