//! Dienste: systemd-Status als Datentabelle mit Aktionen.

use eframe::egui::{self, RichText, Stroke, Vec2};

use crate::app::ControlCenter;
use crate::model::ServiceState;
use crate::theme;
use crate::ui;

const COLUMNS: [f32; 4] = [2.1, 0.85, 0.95, 1.9];

pub fn show(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let Some(snap) = cc.snapshot.clone() else {
        ui::skeleton(ui);
        return;
    };

    ui::helper_notice(cc, ui);

    let mut pending_action: Option<(String, String, String)> = None;
    let pending = cc.action_pending();
    let running = snap
        .services
        .iter()
        .filter(|service| service.state.is_active())
        .count();

    ui::panel_titled(
        ui,
        "Systemdienste",
        &format!("{running}/{} aktiv", snap.services.len()),
        |ui| {
            ui::hint(
                ui,
                "Starten, stoppen und Autostart setzen. Änderungen benötigen \
                 Administratorrechte (pkexec).",
            );
            ui.add_space(12.0);

            if snap.services.is_empty() {
                ui::notice(
                    ui,
                    crate::icons::Icon::Warn,
                    theme::WARN,
                    "systemd ist nicht erreichbar – Dienste lassen sich nur unter \
                     Playforge Linux verwalten.",
                );
                return;
            }

            ui::table_header(
                ui,
                &[
                    ("Dienst", COLUMNS[0]),
                    ("Status", COLUMNS[1]),
                    ("Autostart", COLUMNS[2]),
                    ("Aktion", COLUMNS[3]),
                ],
            );

            for service in &snap.services {
                ui::table_row(ui, &COLUMNS, |ui, column| match column {
                    0 => {
                        ui.label(
                            RichText::new(&service.label)
                                .font(theme::sans(12.5))
                                .color(theme::TEXT),
                        );
                        ui.add(
                            egui::Label::new(
                                RichText::new(&service.unit)
                                    .font(theme::mono(9.0))
                                    .color(theme::FAINT),
                            )
                            .wrap(),
                        );
                    }
                    1 => {
                        if service.installed {
                            ui::badge(ui, service.state.label(), service.state.color());
                        } else {
                            ui::badge(ui, "nicht installiert", theme::FAINT);
                        }
                    }
                    2 => {
                        if service.is_enabled() {
                            ui::badge(ui, "enabled", theme::INFO);
                        } else if service.enabled.is_some() {
                            ui::badge(ui, "disabled", theme::FAINT);
                        } else {
                            ui::mono_label(ui, "—", theme::FAINT, 10.0);
                        }
                    }
                    _ => {
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
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                if small(ui, run_label).clicked() {
                                    pending_action = Some((
                                        run_action.to_string(),
                                        service.label.clone(),
                                        service.unit.clone(),
                                    ));
                                }
                                if small(ui, autostart_label).clicked() {
                                    pending_action = Some((
                                        autostart_action.to_string(),
                                        service.label.clone(),
                                        service.unit.clone(),
                                    ));
                                }
                            });
                        });

                        if service.state == ServiceState::Failed {
                            ui.add_space(4.0);
                            ui::mono_label(ui, "NEUSTART EMPFOHLEN", theme::ERROR, 8.5);
                        }
                    }
                });
            }

            ui.add_space(4.0);
            ui::mono_label(
                ui,
                "ÄNDERUNGEN LAUFEN ÜBER /usr/local/lib/playforge/control-helper.sh",
                theme::FAINT,
                9.0,
            );
        },
    );

    if let Some((action, label, unit)) = pending_action {
        cc.apply_helper(vec!["service".into(), action, unit], &label);
    }
}

fn small(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            RichText::new(text)
                .font(theme::sans(11.0))
                .color(theme::TEXT),
        )
        .fill(theme::RAISED)
        .stroke(Stroke::new(1.0_f32, theme::HAIRLINE))
        .min_size(Vec2::new(0.0, 24.0)),
    )
}
