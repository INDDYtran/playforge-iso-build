//! Protokolle: journalctl-Auszüge des laufenden Boots als Timeline.

use eframe::egui::{self, Align, Color32, Layout};

use crate::app::ControlCenter;
use crate::system::LogMode;
use crate::theme;
use crate::ui;

pub fn show(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let logs = cc.logs.clone();
    let pending = cc.logs_pending();
    let mut reload = false;

    ui::panel_titled(ui, "Protokoll", "journalctl -b", |ui| {
        ui.horizontal(|ui| {
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

            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::singleline(&mut cc.log_filter)
                    .hint_text("Filtern …")
                    .desired_width(200.0),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui::icon_button(ui, crate::icons::Icon::Refresh, "Neu laden", theme::ACCENT)
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
        ui::hairline(ui);
        ui.add_space(10.0);

        match &logs {
            None => {
                if !pending {
                    ui::mono_label(ui, "PROTOKOLL WIRD GELADEN …", theme::FAINT, 9.5);
                }
            }
            Some(Err(error)) => {
                ui::notice(ui, crate::icons::Icon::Warn, theme::WARN, error);
            }
            Some(Ok(text)) => {
                let filter = cc.log_filter.to_lowercase();
                let filtered: Vec<&str> = text
                    .lines()
                    .filter(|line| filter.is_empty() || line.to_lowercase().contains(&filter))
                    .collect();

                ui.horizontal(|ui| {
                    ui::mono_label(ui, &format!("{} ZEILEN", filtered.len()), theme::MUTED, 9.0);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui::badge(
                            ui,
                            &format!("{} · aktueller Boot", cc.log_mode.label()),
                            theme::ACCENT,
                        );
                    });
                });
                ui.add_space(10.0);

                let entries: Vec<ui::Entry> = filtered
                    .iter()
                    .map(|line| {
                        let (time, rest) = split_journal_line(line);
                        ui::Entry::new(time, rest, line_color(line)).mono()
                    })
                    .collect();

                egui::Frame::new()
                    .fill(theme::WELL)
                    .stroke(egui::Stroke::new(1.0_f32, theme::HAIRLINE))
                    .corner_radius(egui::CornerRadius::same(theme::INNER_RADIUS))
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui::timeline(ui, "journal", &entries);
                    });
            }
        }
    });

    ui.add_space(ui::GAP);

    ui::panel_titled(ui, "Hinweis", "", |ui| {
        ui::kv(
            ui,
            "JOURNALCTL",
            "Die Anzeige nutzt journalctl -b des laufenden Boot-Vorgangs.",
        );
        ui::kv(
            ui,
            "RECHTE",
            "Eigene Dienste sind lesbar; systemweite Protokolle können eine Anmeldung verlangen.",
        );
        ui::kv(
            ui,
            "TIPP",
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

/// Zerlegt `Nov 21 20:31:05 host unit: Meldung` in Zeit und Rest.
fn split_journal_line(line: &str) -> (String, String) {
    let mut parts = line.splitn(4, ' ');
    let month = parts.next().unwrap_or("");
    let day = parts.next().unwrap_or("");
    let time = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("");

    let looks_like_time = time.len() == 8 && time.matches(':').count() == 2;
    if month.is_empty() || day.is_empty() || !looks_like_time || rest.is_empty() {
        (String::new(), line.to_string())
    } else {
        (time.to_string(), rest.to_string())
    }
}

fn line_color(line: &str) -> Color32 {
    let lower = line.to_lowercase();
    if lower.contains(" error") || lower.contains("failed") || lower.contains("panic") {
        theme::ERROR
    } else if lower.contains("warn") || lower.contains("deprecated") {
        theme::WARN
    } else if lower.contains(" ok") || lower.contains("playforge_live_ready") {
        theme::OK
    } else {
        theme::MUTED
    }
}
