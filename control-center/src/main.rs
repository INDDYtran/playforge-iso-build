//! Playforge Control Center – grafische Systemsteuerung für Playforge Linux.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod app;
mod icons;
mod model;
mod system;
mod theme;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1300.0, 880.0])
            .with_min_inner_size([1020.0, 680.0])
            .with_app_id("org.playforge.controlcenter"),
        ..Default::default()
    };

    eframe::run_native(
        "Playforge Control Center",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(app::ControlCenter::new(cc)))
        }),
    )
}
