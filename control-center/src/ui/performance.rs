//! Leistung: Profile, Live-Tuning und Shader-Cache.

use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Layout, RichText, Sense, Stroke, StrokeKind, Vec2,
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

    ui::card(ui, "Leistungsprofil", |ui| {
        ui.label(
            RichText::new(
                "Das Profil wird in /etc/playforge/profile gespeichert und beim Booten von \
                 playforge-profile.service angewendet.",
            )
            .size(12.5)
            .color(theme::MUTED),
        );
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            let width = ((ui.available_width() - 20.0) / 3.0).max(150.0);
            option(
                ui,
                cc,
                width,
                "performance",
                "Leistung",
                "Maximaler Takt, lauter",
                theme::AMBER,
            );
            option(
                ui,
                cc,
                width,
                "balanced",
                "Ausgewogen",
                "Leise und sparsam",
                theme::GREEN,
            );
            option(
                ui,
                cc,
                width,
                "reset",
                "Zurücksetzen",
                "Profil entfernen",
                theme::MUTED,
            );
        });

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!cc.action_pending(), |ui| {
                if ui::primary(ui, "Profil anwenden").clicked() {
                    let args = vec!["profile".to_string(), cc.profile_choice.clone()];
                    cc.apply_helper(args, "Leistungsprofil");
                }
            });
            if cc.action_pending() {
                ui.spinner();
                ui.label(
                    RichText::new("Wird angewendet …")
                        .size(12.0)
                        .color(theme::MUTED),
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui::badge(
                    ui,
                    &format!("gespeichert: {}", snap.tuning.profile_saved),
                    theme::COPPER,
                );
                ui::badge(
                    ui,
                    &format!("Governor: {}", snap.tuning.governor),
                    theme::AMBER,
                );
            });
        });
    });

    ui.add_space(14.0);

    ui::card(ui, "Live-Werte", |ui| {
        ui::kv_grid(
            ui,
            &[
                (
                    "Power-Profile-Daemon",
                    snap.tuning
                        .power_profile
                        .clone()
                        .unwrap_or_else(|| "nicht installiert".to_string()),
                ),
                ("CPU-Governor", snap.tuning.governor.clone()),
                ("Energy-Performance", snap.tuning.epp.clone()),
                ("Platform-Profil", snap.tuning.platform_profile.clone()),
                (
                    "vm.max_map_count",
                    snap.tuning
                        .max_map_count
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "—".to_string()),
                ),
                (
                    "vm.swappiness",
                    snap.tuning
                        .swappiness
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "—".to_string()),
                ),
                (
                    "TCP-Kongestion",
                    snap.tuning
                        .tcp_congestion
                        .clone()
                        .unwrap_or_else(|| "—".to_string()),
                ),
                (
                    "Inotify-Watches",
                    snap.tuning
                        .inotify_watches
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "—".to_string()),
                ),
            ],
        );

        ui.add_space(12.0);
        let max_map = snap.tuning.max_map_count.unwrap_or(0);
        ui::meter(
            ui,
            "Proton-Speicherlimit",
            &format!("{max_map} von 2 147 483 642"),
            (max_map as f32 / 2_147_483_642.0).clamp(0.0, 1.0),
            load_color(1.0 - (max_map as f32 / 2_147_483_642.0).clamp(0.0, 1.0)),
        );
        ui.add_space(10.0);
        ui::kv(
            ui,
            "zram",
            if snap.mem.zram.is_empty() {
                "nicht aktiv"
            } else {
                &snap.mem.zram
            },
        );
    });

    ui.add_space(14.0);

    ui::card(ui, "Shader-Cache", |ui| {
        ui.label(
            RichText::new(
                "MESA_SHADER_CACHE_MAX_SIZE begrenzt den Shader-Cache in /etc/environment.d. \
                 Ein größerer Cache vermeidet Nachladen bei Spielstarts.",
            )
            .size(12.5)
            .color(theme::MUTED),
        );
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            for size in ["4G", "8G", "12G", "16G", "32G"] {
                ui.radio_value(&mut cc.shader_choice, size.to_string(), size);
            }
        });
        ui.add_space(10.0);
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
                    theme::AMBER,
                );
            });
        });
        ui.add_space(8.0);
        ui::hint(
            ui,
            "Wirkt nach der nächsten Anmeldung bzw. nach einem Neustart.",
        );
    });
}

fn option(
    ui: &mut egui::Ui,
    cc: &mut ControlCenter,
    width: f32,
    value: &str,
    title: &str,
    description: &str,
    accent: Color32,
) {
    let selected = cc.profile_choice == value;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 84.0), Sense::click());
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        CornerRadius::same(12),
        if selected {
            theme::SURFACE_HI
        } else {
            Color32::from_rgb(20, 16, 12)
        },
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(
            if selected { 2.0_f32 } else { 1.0_f32 },
            if selected { accent } else { theme::BORDER },
        ),
        StrokeKind::Inside,
    );

    let dot = rect.left_top() + Vec2::new(20.0, 24.0);
    painter.circle_stroke(
        dot,
        6.0,
        Stroke::new(1.5_f32, if selected { accent } else { theme::MUTED }),
    );
    if selected {
        painter.circle_filled(dot, 3.0, accent);
    }

    painter.text(
        rect.left_top() + Vec2::new(36.0, 17.0),
        egui::Align2::LEFT_TOP,
        title,
        FontId::proportional(14.5),
        theme::TEXT,
    );
    painter.text(
        rect.left_top() + Vec2::new(36.0, 41.0),
        egui::Align2::LEFT_TOP,
        description,
        FontId::proportional(11.5),
        theme::MUTED,
    );

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if response.clicked() {
        cc.profile_choice = value.to_string();
    }
}
