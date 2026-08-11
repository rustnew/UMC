//! Écran « Historique » — liste des conversions passées.

use eframe::egui;
use egui::{Color32, RichText};

use crate::history::History;

pub fn show(ui: &mut egui::Ui, history: &mut History) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.heading("Historique");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Vider").clicked() {
                    history.clear();
                }
            });
        });
        ui.add_space(8.0);

        if history.entries.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Aucune conversion pour l'instant.")
                        .weak()
                        .size(15.0),
                );
                ui.label(
                    RichText::new("Lancez une conversion depuis l'écran « Convertir ».")
                        .weak()
                        .size(12.0),
                );
            });
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("history_grid")
                .striped(true)
                .min_col_width(90.0)
                .show(ui, |ui| {
                    // En-tête
                    for h in [
                        "Date", "Source", "Cible", "Tenseurs", "Durée", "Statut", "Fichier",
                    ] {
                        ui.label(RichText::new(h).strong());
                    }
                    ui.end_row();

                    for e in &history.entries {
                        ui.label(RichText::new(&e.timestamp).weak());
                        ui.label(&e.source_format);
                        ui.label(&e.target_format);
                        ui.label(e.tensor_count.to_string());
                        ui.label(format!("{:.1}s", e.elapsed_ms as f64 / 1000.0));
                        if e.status == "ok" {
                            ui.label(
                                RichText::new("✓")
                                    .color(Color32::from_rgb(0x4c, 0xaf, 0x50))
                                    .strong(),
                            );
                        } else {
                            ui.label(
                                RichText::new("✗")
                                    .color(Color32::from_rgb(0xe0, 0x5a, 0x5a))
                                    .strong(),
                            );
                        }
                        ui.label(
                            RichText::new(
                                e.output
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string(),
                            )
                            .weak(),
                        );
                        ui.end_row();
                    }
                });
        });
    });
}
