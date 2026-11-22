//! Leistung: Profile, Live-Tuning und Shader-Cache.

use eframe::egui::{
    self, Align, Color32, CornerRadius, Layout, Rect, Sense, Stroke, StrokeKind, Vec2,
};

use crate::app::ControlCenter;
use crate::model::load_color;
use crate::theme;
use crate::ui;

pub fn show(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let Some(snap) = cc.snapshot.clone() else {
        ui::skeleton(ui);
        return;
    };

    ui::helper_notice(cc, ui);

    ui::panel_titled(ui, "Leistungsprofil", "/etc/playforge/profile", |ui| {
        ui::hint(
            ui,
            "Das Profil wird beim Booten von playforge-profile.service angewendet.",
        );
        ui.add_space(12.0);

        ui::grid(ui, &[1.0, 1.0, 1.0], |ui, column| {
            let (value, title, description, accent) = match column {
                0 => (
                    "performance",
                    "Leistung",
                    "Maximaler Takt, lauter",
                    theme::ACCENT,
                ),
                1 => ("balanced", "Ausgewogen", "Leise und sparsam", theme::OK),
                _ => ("reset", "Zurücksetzen", "Profil entfernen", theme::MUTED),
            };
            option(ui, cc, value, title, description, accent);
        });

        ui.add_space(14.0);
        ui::hairline(ui);
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!cc.action_pending(), |ui| {
                if ui::primary(ui, "Profil anwenden").clicked() {
                    let args = vec!["profile".to_string(), cc.profile_choice.clone()];
                    cc.apply_helper(args, "Leistungsprofil");
                }
            });
            if cc.action_pending() {
                ui.spinner();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui::badge(
                    ui,
                    &format!("Governor {}", snap.tuning.governor),
                    theme::ACCENT,
                );
                ui::badge(
                    ui,
                    &format!("gespeichert: {}", snap.tuning.profile_saved),
                    theme::MUTED,
                );
            });
        });
    });

    ui.add_space(ui::GAP);

    ui::grid(ui, &[1.0, 1.0], |ui, column| {
        if column == 0 {
            ui::panel_titled(ui, "Live-Werte", "sysctl", |ui| {
                for (label, value) in [
                    (
                        "POWER-PROFILE",
                        snap.tuning
                            .power_profile
                            .clone()
                            .unwrap_or_else(|| "nicht installiert".to_string()),
                    ),
                    ("GOVERNOR", snap.tuning.governor.clone()),
                    ("EPP", snap.tuning.epp.clone()),
                    ("PLATFORM-PROFIL", snap.tuning.platform_profile.clone()),
                    (
                        "VM.MAX_MAP_COUNT",
                        snap.tuning
                            .max_map_count
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                    (
                        "VM.SWAPPINESS",
                        snap.tuning
                            .swappiness
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                    (
                        "TCP-KONGESTION",
                        snap.tuning
                            .tcp_congestion
                            .clone()
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                    (
                        "INOTIFY-WATCHES",
                        snap.tuning
                            .inotify_watches
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                ] {
                    ui::kv(ui, label, &value);
                }
            });
        } else {
            ui::panel_titled(ui, "Proton-Speicherlimit", "vm.max_map_count", |ui| {
                let max_map = snap.tuning.max_map_count.unwrap_or(0);
                let fraction = (max_map as f32 / 2_147_483_642.0).clamp(0.0, 1.0);
                ui::meter(
                    ui,
                    "LIMIT",
                    &format!("{max_map} / 2 147 483 642"),
                    fraction,
                    load_color(1.0 - fraction),
                );
                ui.add_space(14.0);
                ui::meter(
                    ui,
                    "RAM-AUSLASTUNG",
                    &format!("{:.0} %", snap.mem.ram_fraction() * 100.0),
                    snap.mem.ram_fraction(),
                    load_color(snap.mem.ram_fraction()),
                );
                ui.add_space(14.0);
                ui::meter(
                    ui,
                    "SWAP",
                    &format!("{:.0} %", snap.mem.swap_fraction() * 100.0),
                    snap.mem.swap_fraction(),
                    theme::INFO,
                );
                ui.add_space(14.0);
                ui::hairline(ui);
                ui.add_space(10.0);
                ui::kv(
                    ui,
                    "ZRAM",
                    if snap.mem.zram.is_empty() {
                        "nicht aktiv"
                    } else {
                        &snap.mem.zram
                    },
                );
            });
        }
    });

    ui.add_space(ui::GAP);

    ui::panel_titled(ui, "Shader-Cache", "mesa_shader_cache_max_size", |ui| {
        ui::hint(
            ui,
            "Begrenzt den Shader-Cache in /etc/environment.d. Ein größerer Cache \
             vermeidet Nachladen bei Spielstarts.",
        );
        ui.add_space(11.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
            for size in ["4G", "8G", "12G", "16G", "32G"] {
                if ui::choice(ui, cc.shader_choice == size, size).clicked() {
                    cc.shader_choice = size.to_string();
                }
            }
        });
        ui.add_space(12.0);
        ui::hairline(ui);
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!cc.action_pending(), |ui| {
                if ui::primary(ui, "Übernehmen").clicked() {
                    let args = vec!["shader-cache".to_string(), cc.shader_choice.clone()];
                    cc.apply_helper(args, "Shader-Cache");
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui::badge(
                    ui,
                    &format!(
                        "aktiv: {}",
                        if snap.settings.shader_cache.is_empty() {
                            "Standard"
                        } else {
                            &snap.settings.shader_cache
                        }
                    ),
                    theme::ACCENT,
                );
            });
        });
        ui.add_space(9.0);
        ui::mono_label(ui, "WIRKT NACH DER NÄCHSTEN ANMELDUNG", theme::FAINT, 9.0);
    });
}

fn option(
    ui: &mut egui::Ui,
    cc: &mut ControlCenter,
    value: &str,
    title: &str,
    description: &str,
    accent: Color32,
) {
    let selected = cc.profile_choice == value;
    let height = 74.0;
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
    let painter = ui.painter();

    painter.rect_filled(
        rect,
        CornerRadius::same(theme::INNER_RADIUS),
        if selected {
            theme::alpha(accent, 16)
        } else if response.hovered() {
            theme::RAISED
        } else {
            theme::PANEL
        },
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(theme::INNER_RADIUS),
        Stroke::new(
            1.0_f32,
            if selected {
                theme::alpha(accent, 170)
            } else {
                theme::HAIRLINE
            },
        ),
        StrokeKind::Inside,
    );

    let square = Rect::from_min_size(rect.left_top() + Vec2::new(14.0, 16.0), Vec2::splat(7.0));
    if selected {
        painter.rect_filled(square, CornerRadius::ZERO, accent);
    } else {
        painter.rect_stroke(
            square,
            CornerRadius::ZERO,
            Stroke::new(1.0_f32, theme::FAINT),
            StrokeKind::Inside,
        );
    }

    painter.text(
        rect.left_top() + Vec2::new(30.0, 10.0),
        egui::Align2::LEFT_TOP,
        title,
        theme::sans(13.0),
        if selected { theme::TEXT } else { theme::MUTED },
    );
    painter.text(
        rect.left_top() + Vec2::new(30.0, 31.0),
        egui::Align2::LEFT_TOP,
        description,
        theme::mono(9.0),
        theme::FAINT,
    );
    painter.text(
        rect.left_top() + Vec2::new(30.0, 48.0),
        egui::Align2::LEFT_TOP,
        value.to_string(),
        theme::mono(9.0),
        if selected { accent } else { theme::FAINT },
    );

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if response.clicked() {
        cc.profile_choice = value.to_string();
    }
}
