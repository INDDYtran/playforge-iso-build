//! Protokolle: journalctl-Auszüge des laufenden Boots.

use eframe::egui::{self, Align, Color32, Layout, RichText};

use crate::app::ControlCenter;
use crate::system::LogMode;
use crate::theme;
use crate::ui;

pub fn show(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let logs = cc.logs.clone();
    let pending = cc.logs_pending();
    let mut reload = false;

    ui::card(ui, "Protokoll", |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Quelle").size(12.5).color(theme::MUTED));
            let mut mode = cc.log_mode;
            egui::ComboBox::from_id_salt("log-mode")
                .width(230.0)
                .selected_text(mode.label())
                .show_ui(ui, |ui| {
                    for option in LogMode::ALL {
                        if ui
                            .selectable_value(&mut mode, option, option.label())
                            .clicked()
                        {
                            reload = true;
                        }
                    }
                });
            if mode != cc.log_mode {
                cc.log_mode = mode;
                reload = true;
            }

            ui.add_space(8.0);
            ui.add(
                egui::TextEdit::singleline(&mut cc.log_filter)
                    .hint_text("Filtern …")
                    .desired_width(200.0),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui::icon_button(ui, crate::icons::Icon::Refresh, "Neu laden", theme::AMBER)
                    .clicked()
                {
                    reload = true;
                }
                if pending {
                    ui.spinner();
                }
                ui::copy_button(ui, &raw_text(&logs), "Kopieren");
            });
        });

        ui.add_space(12.0);

        match &logs {
            None => {
                if !pending {
                    ui::hint(ui, "Protokoll wird geladen …");
                }
            }
            Some(Err(error)) => {
                ui::notice(ui, crate::icons::Icon::Warn, theme::AMBER, error);
            }
            Some(Ok(text)) => {
                let filter = cc.log_filter.to_lowercase();
                let filtered: Vec<&str> = text
                    .lines()
                    .filter(|line| filter.is_empty() || line.to_lowercase().contains(&filter))
                    .collect();

                ui.horizontal(|ui| {
                    ui::hint(ui, &format!("{} Zeilen", filtered.len()));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui::badge(
                            ui,
                            &format!("{} · aktueller Boot", cc.log_mode.label()),
                            theme::COPPER,
                        );
                    });
                });
                ui.add_space(8.0);

                egui::Frame::new()
                    .fill(Color32::from_rgb(8, 6, 4))
                    .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
                    .corner_radius(egui::CornerRadius::same(9))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        egui::ScrollArea::both()
                            .id_salt("journal-view")
                            .max_height(430.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                for line in &filtered {
                                    ui.label(
                                        RichText::new(*line)
                                            .monospace()
                                            .size(11.5)
                                            .color(line_color(line)),
                                    );
                                }
                            });
                    });
            }
        }
    });

    ui.add_space(12.0);
    ui::card(ui, "Hinweis", |ui| {
        ui::kv(
            ui,
            "journalctl",
            "Die Anzeige nutzt journalctl -b des laufenden Boot-Vorgangs.",
        );
        ui::kv(
            ui,
            "Rechte",
            "Eigene Dienste sind lesbar; systemweite Protokolle können eine Anmeldung verlangen.",
        );
        ui::kv(
            ui,
            "Tipp",
            "Für mehr Kontext: journalctl -b -u playforge-profile.service im Terminal.",
        );
    });

    if reload {
        cc.load_logs();
    }
}

fn raw_text(logs: &Option<Result<String, String>>) -> String {
    match logs {
        Some(Ok(text)) => text.clone(),
        Some(Err(error)) => error.clone(),
        None => String::new(),
    }
}

fn line_color(line: &str) -> Color32 {
    let lower = line.to_lowercase();
    if lower.contains(" error") || lower.contains("failed") || lower.contains("panic") {
        theme::RED
    } else if lower.contains("warn") || lower.contains("deprecated") {
        theme::AMBER
    } else if lower.contains(" ok") || lower.contains("playforge_live_ready") {
        theme::GREEN
    } else {
        theme::MUTED
    }
}
