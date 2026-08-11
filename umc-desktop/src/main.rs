//! UMC Desktop — Universal Model Converter, application de bureau.
//!
//! Interface native (egui/eframe) pour convertir des modèles entre formats
//! (GGUF, SafeTensors, ONNX, PyTorch, …) en local, sans serveur.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod history;
mod screens;
mod worker;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("UMC — Universal Model Converter")
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "UMC — Universal Model Converter",
        options,
        Box::new(|cc| Ok(Box::new(app::UmcApp::new(cc)))),
    )
}
