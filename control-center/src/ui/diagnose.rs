//! Diagnose: Ergebnis von playforge-doctor.

use eframe::egui::{self, Align, Layout, RichText, Sense, Vec2};

use crate::app::ControlCenter;
use crate::icons::Icon;
use crate::system::{CheckKind, DoctorItem, DoctorReport};
use crate::theme;
use crate::ui;

pub fn show(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let report = cc.doctor.clone();
    let pending = cc.doctor_pending();
    let mut start = false;

    ui::card(ui, "Systemdiagnose", |ui| {
        ui.label(
            RichText::new(
                "playforge-doctor prüft Kernel, Grafiktreiber, Vulkan, 32-Bit-Unterstützung, \
                 den Gaming-Stack und das Tuning.",
            )
            .size(12.5)
            .color(theme::MUTED),
        );
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!pending, |ui| {
                if ui::primary(ui, "Prüfung starten").clicked() {
                    start = true;
                }
            });
            if pending {
                ui.spinner();
                ui.label(
                    RichText::new("Prüfung läuft …")
                        .size(12.0)
                        .color(theme::MUTED),
                );
            }
            if let Some(report) = report.as_ref().filter(|report| report.available) {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui::badge(ui, &format!("{} Fehler", report.fail), theme::RED);
                    ui::badge(ui, &format!("{} Warnungen", report.warn), theme::AMBER);
                    ui::badge(ui, &format!("{} OK", report.ok), theme::GREEN);
                });
            }
        });
        ui.add_space(8.0);
        ui::hint(
            ui,
            "Die Prüfung läuft ohne Administratorrechte und verändert nichts.",
        );
    });

    if start {
        cc.start_doctor();
    }

    let Some(report) = report else {
        return;
    };

    if let Some(error) = &report.error {
        ui.add_space(14.0);
        ui::notice(ui, Icon::Warn, theme::AMBER, error);
        return;
    }

    if report.sections.is_empty() {
        ui.add_space(14.0);
        ui::notice(
            ui,
            Icon::Check,
            theme::GREEN,
            "Keine Auffälligkeiten gemeldet.",
        );
        return;
    }

    ui.add_space(14.0);
    render_report(ui, &report);
}

fn render_report(ui: &mut egui::Ui, report: &DoctorReport) {
    for section in &report.sections {
        ui::card(ui, &section.title, |ui| {
            for item in &section.items {
                item_row(ui, item);
            }
        });
        ui.add_space(12.0);
    }

    if !report.raw.is_empty() {
        ui::card(ui, "Rohausgabe", |ui| {
            ui.horizontal(|ui| {
                ui::hint(ui, "Vollständige Ausgabe von playforge-doctor");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui::copy_button(ui, &report.raw, "Kopieren");
                });
            });
            ui.add_space(8.0);
            ui::mono_block(ui, "doctor-raw", &report.raw, 320.0);
        });
    }
}

fn item_row(ui: &mut egui::Ui, item: &DoctorItem) {
    let (icon, color) = match item.kind {
        CheckKind::Ok => (Some(Icon::Check), theme::GREEN),
        CheckKind::Warn => (Some(Icon::Warn), theme::AMBER),
        CheckKind::Fail => (Some(Icon::Cross), theme::RED),
        CheckKind::Info => (None, theme::MUTED),
    };

    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(15.0), Sense::hover());
        match icon {
            Some(icon) => crate::icons::paint(ui.painter(), rect, icon, color),
            None => {
                ui.painter().circle_filled(rect.center(), 2.5, theme::FAINT);
            }
        }
        ui.add_space(2.0);
        ui.label(
            RichText::new(&item.text)
                .size(12.5)
                .color(if item.kind == CheckKind::Info {
                    theme::MUTED
                } else {
                    theme::TEXT
                }),
        );
    });
}
