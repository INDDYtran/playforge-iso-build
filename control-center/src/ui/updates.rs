//! Updates: pacman, Flatpak und automatische Aktualisierung.

use eframe::egui::{self, Align, Layout, RichText};

use crate::app::ControlCenter;
use crate::icons::Icon;
use crate::model::{fmt_age, Snapshot};
use crate::theme;
use crate::ui;

pub fn show(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let Some(snap) = cc.snapshot.clone() else {
        ui::skeleton(ui);
        return;
    };

    ui::helper_notice(cc, ui);

    status_card(cc, ui);
    ui.add_space(ui::GAP);

    ui::grid(ui, &[1.0, 1.0], |ui, column| {
        if column == 0 {
            auto_card(cc, ui, &snap);
        } else {
            manual_card(cc, ui);
        }
    });
}

fn status_card(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let report = cc.update_report.clone();
    let pending = cc.update_check_pending();

    ui::panel_titled(ui, "Verfügbare Updates", "pacman · aur · flatpak", |ui| {
        ui.horizontal(|ui| {
            ui::hint(
                ui,
                "Prüft pacman, den AUR-Helfer und Flatpak, ohne etwas zu installieren.",
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_enabled_ui(!pending, |ui| {
                    if ui::icon_button(ui, Icon::Refresh, "Jetzt prüfen", theme::ACCENT).clicked()
                    {
                        cc.start_update_check();
                    }
                });
                if pending {
                    ui.spinner();
                }
            });
        });

        ui.add_space(12.0);

        match &report {
            None => {
                ui::mono_label(ui, "NOCH KEINE PRÜFUNG DURCHGEFÜHRT", theme::FAINT, 9.5);
            }
            Some(report) => {
                if let Some(error) = &report.error {
                    ui::notice(ui, Icon::Warn, theme::WARN, error);
                    ui.add_space(10.0);
                }

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 6.0);
                    count_badge(ui, "pacman", report.pacman.len(), theme::ACCENT);
                    count_badge(
                        ui,
                        report.aur_helper.as_deref().unwrap_or("AUR"),
                        report.aur.len(),
                        theme::INFO,
                    );
                    if report.flatpak_available {
                        count_badge(ui, "Flatpak", report.flatpak, theme::OK);
                    } else {
                        ui::badge(ui, "Flatpak nicht installiert", theme::MUTED);
                    }
                    if let Some(checked) = report.checked {
                        ui::badge(ui, &format!("geprüft {}", fmt_age(checked)), theme::FAINT);
                    }
                });

                if !report.pacman.is_empty() {
                    ui.add_space(11.0);
                    egui::CollapsingHeader::new(
                        RichText::new(format!("pacman-Pakete ({})", report.pacman.len()))
                            .font(theme::mono(10.0))
                            .color(theme::MUTED),
                    )
                    .id_salt("update-pacman")
                    .show(ui, |ui| {
                        ui.add_space(6.0);
                        ui::code_block(
                            ui,
                            "update-pacman-list",
                            "pacman -Qu",
                            &report.pacman.join("\n"),
                            200.0,
                        );
                    });
                }

                if !report.aur.is_empty() {
                    ui.add_space(9.0);
                    egui::CollapsingHeader::new(
                        RichText::new(format!("AUR-Pakete ({})", report.aur.len()))
                            .font(theme::mono(10.0))
                            .color(theme::MUTED),
                    )
                    .id_salt("update-aur")
                    .show(ui, |ui| {
                        ui.add_space(6.0);
                        ui::code_block(
                            ui,
                            "update-aur-list",
                            "aur -Qua",
                            &report.aur.join("\n"),
                            200.0,
                        );
                    });
                    if let Some(helper) = &report.aur_helper {
                        ui.add_space(9.0);
                        ui::notice(
                            ui,
                            Icon::Warn,
                            theme::WARN,
                            &format!(
                                "AUR-Pakete lassen sich nicht als root bauen. Im Terminal \
                                 aktualisieren: {helper} -Sua"
                            ),
                        );
                    }
                }

                if report.pacman.is_empty()
                    && report.aur.is_empty()
                    && report.flatpak == 0
                    && report.error.is_none()
                {
                    ui.add_space(11.0);
                    ui::notice(
                        ui,
                        Icon::Check,
                        theme::OK,
                        "Alles aktuell – keine Updates gefunden.",
                    );
                }
            }
        }
    });
}

fn count_badge(ui: &mut egui::Ui, label: &str, count: usize, color: egui::Color32) {
    if count == 0 {
        ui::badge(ui, &format!("{label}: aktuell"), theme::OK);
    } else {
        ui::badge(ui, &format!("{label}: {count}"), color);
    }
}

fn auto_card(cc: &mut ControlCenter, ui: &mut egui::Ui, snap: &Snapshot) {
    let auto = snap.tuning.autoupdate.clone();

    ui::panel_titled(ui, "Automatische Updates", "systemd-timer", |ui| {
        ui::hint(
            ui,
            "Legt playforge-autoupdate.service und einen Timer an, der pacman und Flatpak \
             ohne geöffnetes Control Center aktualisiert.",
        );
        ui.add_space(12.0);

        ui::kicker(ui, "Intervall");
        ui.add_space(5.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 6.0);
            for (value, label) in [
                ("off", "Aus"),
                ("daily", "Täglich"),
                ("weekly", "Wöchentlich"),
            ] {
                if ui::choice(ui, cc.autoupdate_choice == value, label).clicked() {
                    cc.autoupdate_choice = value.to_string();
                }
            }
        });

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 6.0);
            if auto.timer_active {
                ui::badge(ui, "Timer aktiv", theme::OK);
            } else {
                ui::badge(ui, "Timer aus", theme::MUTED);
            }
            match auto.last_update {
                Some(time) => ui::badge(ui, &format!("zuletzt {}", fmt_age(time)), theme::INFO),
                None => ui::badge(ui, "noch nie gelaufen", theme::FAINT),
            };
        });

        ui.add_space(12.0);
        ui::hairline(ui);
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!cc.action_pending(), |ui| {
                if ui::primary(ui, "Automatik übernehmen").clicked() {
                    cc.apply_autoupdate();
                }
            });
            if cc.action_pending() {
                ui.spinner();
            }
        });

        ui.add_space(9.0);
        ui::mono_label(
            ui,
            "FLATPAK SYSTEMWEIT · BENUTZER-PAKETE ÜBER FLATPAK UPDATE",
            theme::FAINT,
            9.0,
        );
    });
}

fn manual_card(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let pending = cc.update_pending();
    let log = cc.update_log.join("\n");
    let done = cc.update_done.clone();

    ui::panel_titled(ui, "Jetzt aktualisieren", "pacman -syu", |ui| {
        ui::hint(
            ui,
            "Führt pacman -Syu aus und aktualisiert auf Wunsch Flatpak und die Paketspiegel.",
        );
        ui.add_space(10.0);

        ui.checkbox(&mut cc.update_flatpak, "Flatpak aktualisieren");
        ui.checkbox(&mut cc.update_mirrors, "Spiegel mit reflector erneuern");

        ui.add_space(11.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!pending && cc.helper_available, |ui| {
                if ui::primary(ui, "System aktualisieren").clicked() {
                    cc.start_update();
                }
            });
            if pending {
                ui.spinner();
                ui::mono_label(ui, "LÄUFT …", theme::ACCENT, 9.5);
            } else if let Some(result) = &done {
                if result.ok {
                    ui::badge(ui, "abgeschlossen", theme::OK);
                } else {
                    ui::badge(ui, "mit Fehler beendet", theme::ERROR);
                }
            }
        });

        if !log.is_empty() {
            ui.add_space(11.0);
            ui::code_block(ui, "update-log", "ausgabe", &log, 230.0);
        }

        if let Some(result) = &done {
            if !result.ok {
                ui.add_space(9.0);
                ui::notice(ui, Icon::Warn, theme::WARN, &result.message);
            }
        }
    });
}
