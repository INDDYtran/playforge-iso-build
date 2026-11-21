//! Diagnose: Ergebnis von playforge-doctor als Kennzahlen und Timeline.

use eframe::egui::{self, Align, Layout};

use crate::app::ControlCenter;
use crate::icons::Icon;
use crate::system::{CheckKind, DoctorItem, DoctorReport};
use crate::theme;
use crate::ui;

pub fn show(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let report = cc.doctor.clone();
    let pending = cc.doctor_pending();
    let mut start = false;

    if let Some(report) = report.as_ref().filter(|report| report.available) {
        ui::grid(ui, &[1.0, 1.0, 1.0], |ui, column| {
            let (label, value, accent) = match column {
                0 => ("OK", report.ok, theme::OK),
                1 => ("Warnungen", report.warn, theme::WARN),
                _ => ("Fehler", report.fail, theme::ERROR),
            };
            ui::metric(ui, label, &value.to_string(), "", accent);
        });
        ui.add_space(ui::GAP);
    }

    ui::panel_titled(ui, "Systemdiagnose", "playforge-doctor", |ui| {
        ui::hint(
            ui,
            "playforge-doctor prüft Kernel, Grafiktreiber, Vulkan, 32-Bit-Unterstützung, \
             den Gaming-Stack und das Tuning.",
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
                ui::mono_label(ui, "PRÜFUNG LÄUFT …", theme::MUTED, 9.5);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui::mono_label(ui, "OHNE ADMINISTRATORRECHTE", theme::FAINT, 9.0);
            });
        });
    });

    if start {
        cc.start_doctor();
    }

    let Some(report) = report else {
        return;
    };

    ui.add_space(ui::GAP);

    if let Some(error) = &report.error {
        ui::notice(ui, Icon::Warn, theme::WARN, error);
        return;
    }

    if report.sections.is_empty() {
        ui::notice(
            ui,
            Icon::Check,
            theme::OK,
            "Keine Auffälligkeiten gemeldet.",
        );
        return;
    }

    render_report(ui, &report);
}

fn render_report(ui: &mut egui::Ui, report: &DoctorReport) {
    for section in &report.sections {
        ui::panel_titled(
            ui,
            &section.title,
            &format!("{} Punkte", section.items.len()),
            |ui| {
                let entries: Vec<ui::Entry> = section.items.iter().map(entry).collect();
                ui::timeline(ui, &section.title, &entries);
            },
        );
        ui.add_space(ui::GAP);
    }

    if !report.raw.is_empty() {
        ui::panel_titled(ui, "Rohausgabe", "stdout", |ui| {
            ui::code_block(ui, "doctor-raw", "playforge-doctor", &report.raw, 300.0);
        });
    }
}

fn entry(item: &DoctorItem) -> ui::Entry {
    let color = match item.kind {
        CheckKind::Ok => theme::OK,
        CheckKind::Warn => theme::WARN,
        CheckKind::Fail => theme::ERROR,
        CheckKind::Info => theme::FAINT,
    };
    ui::Entry::new(kind_label(item.kind), item.text.clone(), color)
}

fn kind_label(kind: CheckKind) -> &'static str {
    match kind {
        CheckKind::Ok => "OK",
        CheckKind::Warn => "WARN",
        CheckKind::Fail => "FAIL",
        CheckKind::Info => "INFO",
    }
}
