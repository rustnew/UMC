//! "Formats" screen — table of supported formats.

use eframe::egui;
use egui::{Color32, RichText};

/// Static catalog of formats (aligned with the `umc formats` CLI).
const FORMATS: &[(&str, &str, &str, &str)] = &[
    (
        "GGUF",
        "✓ native",
        "planned",
        "GGUF v1/v2/v3, all quant types",
    ),
    (
        "SafeTensors",
        "✓ native",
        "✓ native",
        "SafeTensors HuggingFace",
    ),
    ("ONNX", "planned", "planned", "ONNX opset 13-21"),
    (
        "PyTorch",
        "planned",
        "planned",
        "PyTorch .pt/.pth (safe pickle)",
    ),
    ("TFLite", "planned", "planned", "TFLite FlatBuffers"),
    ("KerasH5", "planned", "—", "Keras H5 (read-only)"),
    ("GGML", "planned", "—", "GGML legacy (read-only)"),
    ("SentencePiece", "✓ native", "—", "Tokenizers .model"),
];

pub fn show(ui: &mut egui::Ui) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.add_space(6.0);
        ui.heading("Supported formats");
        ui.label(
            RichText::new("Automatic detection by binary signature, extension and content analysis.")
                .weak(),
        );
        ui.add_space(12.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("formats_grid")
                .striped(true)
                .min_col_width(110.0)
                .show(ui, |ui| {
                    for h in ["Format", "Read", "Write", "Notes"] {
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
                "Legend: ✓ native = implemented in pure Rust · planned = on the roadmap · — = not supported.",
            )
            .weak()
            .size(11.0),
        );
    });
}

fn colored_bool(s: &str) -> RichText {
    if s.starts_with('✓') {
        RichText::new(s).color(Color32::from_rgb(0x4c, 0xaf, 0x50))
    } else if s.starts_with("planned") {
        RichText::new(s).color(Color32::from_rgb(0xe0, 0xa0, 0x3c))
    } else {
        RichText::new(s).weak()
    }
}
