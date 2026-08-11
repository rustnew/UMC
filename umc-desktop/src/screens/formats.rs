//! Écran « Formats » — tableau des formats supportés.

use eframe::egui;
use egui::{Color32, RichText};

/// Catalogue statique des formats (aligné sur le CLI `umc formats`).
const FORMATS: &[(&str, &str, &str, &str)] = &[
    (
        "GGUF",
        "✓ natif",
        "prévu",
        "GGUF v1/v2/v3, tous types de quant",
    ),
    (
        "SafeTensors",
        "✓ natif",
        "✓ natif",
        "SafeTensors HuggingFace",
    ),
    ("ONNX", "prévu", "prévu", "ONNX opset 13-21"),
    ("PyTorch", "prévu", "prévu", "PyTorch .pt/.pth (pickle sûr)"),
    ("TFLite", "prévu", "prévu", "TFLite FlatBuffers"),
    ("KerasH5", "prévu", "—", "Keras H5 (lecture seule)"),
    ("GGML", "prévu", "—", "GGML hérité (lecture seule)"),
    ("SentencePiece", "✓ natif", "—", "Tokenizers .model"),
];

pub fn show(ui: &mut egui::Ui) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.add_space(6.0);
        ui.heading("Formats supportés");
        ui.label(
            RichText::new("Détection automatique par signature binaire, extension et analyse de contenu.")
                .weak(),
        );
        ui.add_space(12.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("formats_grid")
                .striped(true)
                .min_col_width(110.0)
                .show(ui, |ui| {
                    for h in ["Format", "Lecture", "Écriture", "Notes"] {
                        ui.label(RichText::new(h).strong());
                    }
                    ui.end_row();

                    for (name, load, save, notes) in FORMATS {
                        ui.label(RichText::new(*name).strong());
                        ui.label(colored_bool(load));
                        ui.label(colored_bool(save));
                        ui.label(RichText::new(*notes).weak());
                        ui.end_row();
                    }
                });
        });

        ui.add_space(16.0);
        ui.separator();
        ui.label(
            RichText::new(
                "Légende : ✓ natif = implémenté en Rust pur · prévu = sur la feuille de route · — = non supporté.",
            )
            .weak()
            .size(11.0),
        );
    });
}

fn colored_bool(s: &str) -> RichText {
    if s.starts_with('✓') {
        RichText::new(s).color(Color32::from_rgb(0x4c, 0xaf, 0x50))
    } else if s.starts_with("prévu") {
        RichText::new(s).color(Color32::from_rgb(0xe0, 0xa0, 0x3c))
    } else {
        RichText::new(s).weak()
    }
}
