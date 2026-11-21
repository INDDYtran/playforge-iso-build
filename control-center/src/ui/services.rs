//! Dienste: systemd-Status und Schaltflächen.

use eframe::egui::{self, Align, CornerRadius, Layout, Margin, RichText, Stroke};

use crate::app::ControlCenter;
use crate::model::{ServiceInfo, ServiceState};
use crate::theme;
use crate::ui;

pub fn show(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let Some(snap) = cc.snapshot.clone() else {
        ui::skeleton(ui);
        return;
    };

    ui::helper_notice(cc, ui);

    let mut pending_action: Option<(String, String, String)> = None;
    let pending = cc.action_pending();

    ui::card(ui, "Systemdienste", |ui| {
        ui.label(
            RichText::new(
                "Starten, stoppen und Autostart setzen. Änderungen benötigen \
                 Administratorrechte (pkexec).",
            )
            .size(12.5)
            .color(theme::MUTED),
        );
        ui.add_space(12.0);

        if snap.services.is_empty() {
            ui::notice(
                ui,
                crate::icons::Icon::Warn,
                theme::AMBER,
                "systemd ist nicht erreichbar – Dienste lassen sich nur unter Playforge Linux verwalten.",
            );
            return;
        }

        for service in &snap.services {
            if let Some(action) = row(ui, service, pending) {
                pending_action = Some((action.0, action.1, service.unit.clone()));
            }
            ui.add_space(6.0);
        }
    });

    if let Some((action, label, unit)) = pending_action {
        cc.apply_helper(vec!["service".into(), action, unit], &label);
    }
}

fn row(ui: &mut egui::Ui, service: &ServiceInfo, pending: bool) -> Option<(String, String)> {
    let mut triggered: Option<(String, String)> = None;

    egui::Frame::new()
        .fill(egui::Color32::from_rgb(20, 16, 12))
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&service.label).size(13.5).strong());
                        ui::badge(ui, service.state.label(), service.state.color());
                        if !service.installed {
                            ui::badge(ui, "nicht installiert", theme::MUTED);
                        } else if service.is_enabled() {
                            ui::badge(ui, "Autostart", theme::BLUE);
                        }
                    });
                    ui.label(
                        RichText::new(format!("{} · {}", service.unit, service.description))
                            .size(11.5)
                            .color(theme::MUTED),
                    );
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    let autostart_label = if service.is_enabled() {
                        "Autostart aus"
                    } else {
                        "Autostart an"
                    };
                    let autostart_action = if service.is_enabled() {
                        "disable"
                    } else {
                        "enable"
                    };
                    ui.add_enabled_ui(!pending && service.installed, |ui| {
                        if ui::ghost(ui, autostart_label).clicked() {
                            triggered = Some((autostart_action.to_string(), service.label.clone()));
                        }
                    });

                    let run_label = if service.state.is_active() {
                        "Stoppen"
                    } else {
                        "Starten"
                    };
                    let run_action = if service.state.is_active() {
                        "stop"
                    } else {
                        "start"
                    };
                    ui.add_enabled_ui(!pending && service.installed, |ui| {
                        if ui::ghost(ui, run_label).clicked() {
                            triggered = Some((run_action.to_string(), service.label.clone()));
                        }
                    });

                    if service.state == ServiceState::Failed {
                        ui::badge(ui, "Neustart empfohlen", theme::RED);
                    }
                });
            });
        });

    triggered
}
