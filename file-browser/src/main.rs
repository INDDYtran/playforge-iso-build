//! Playforge Files – Dateibrowser für Playforge Linux.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod fs;
mod icons;
mod theme;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([980.0, 620.0])
            .with_app_id("org.playforge.files"),
        ..Default::default()
    };

    eframe::run_native(
        "Playforge Files",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(app::FilesApp::new(cc)))
        }),
    )
}
