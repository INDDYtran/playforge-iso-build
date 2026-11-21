//! Gaming: GameMode, MangoHud und Startoptionen.

use eframe::egui::{self, Align, Layout, RichText, Vec2};

use crate::app::ControlCenter;
use crate::theme;
use crate::ui;

pub fn show(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let Some(snap) = cc.snapshot.clone() else {
        ui::skeleton(ui);
        return;
    };

    ui::helper_notice(cc, ui);

    // --- GameMode ---------------------------------------------------------
    ui::card(ui, "GameMode", |ui| {
        ui.horizontal_wrapped(|ui| {
            ui::badge(
                ui,
                if snap.gaming.gamemode {
                    "GameMode installiert"
                } else {
                    "GameMode fehlt"
                },
                if snap.gaming.gamemode {
                    theme::GREEN
                } else {
                    theme::RED
                },
            );
        });
        ui.label(
            RichText::new(
                "GameMode hebt während des Spiels Priorität und Governor an. \
                 Die Werte stehen in /etc/gamemode.ini.",
            )
            .size(12.5)
            .color(theme::MUTED),
        );
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("Governor").size(12.5).color(theme::MUTED));
            egui::ComboBox::from_id_salt("gamemode-gov")
                .width(170.0)
                .selected_text(cc.gamemode_draft.desired_gov.clone())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut cc.gamemode_draft.desired_gov,
                        "performance".to_string(),
                        "performance",
                    );
                    ui.selectable_value(
                        &mut cc.gamemode_draft.desired_gov,
                        "powersave".to_string(),
                        "powersave",
                    );
                });

            ui.add_space(14.0);
            ui.label(RichText::new("softrealtime").size(12.5).color(theme::MUTED));
            egui::ComboBox::from_id_salt("gamemode-rt")
                .width(110.0)
                .selected_text(cc.gamemode_draft.softrealtime.clone())
                .show_ui(ui, |ui| {
                    for value in ["auto", "on", "off"] {
                        ui.selectable_value(
                            &mut cc.gamemode_draft.softrealtime,
                            value.to_string(),
                            value,
                        );
                    }
                });
        });

        ui.add_space(8.0);
        ui.spacing_mut().slider_width = 260.0;
        ui.add(
            egui::Slider::new(&mut cc.gamemode_draft.renice, 0..=19)
                .text("renice")
                .min_decimals(0),
        );
        ui.add_space(4.0);
        ui::hint(
            ui,
            "renice 10 ist der Playforge-Standard; höhere Werte bevorzugen das Spiel stärker.",
        );

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!cc.action_pending(), |ui| {
                if ui::primary(ui, "Übernehmen").clicked() {
                    let args = vec![
                        "gamemode".to_string(),
                        format!("desiredgov={}", cc.gamemode_draft.desired_gov),
                        format!("renice={}", cc.gamemode_draft.renice),
                        format!("softrealtime={}", cc.gamemode_draft.softrealtime),
                    ];
                    cc.apply_helper(args, "GameMode");
                }
            });
            if cc.action_pending() {
                ui.spinner();
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui::badge(
                    ui,
                    &format!(
                        "aktiv: {} / {} / renice {}",
                        snap.settings.gamemode.desired_gov,
                        snap.settings.gamemode.softrealtime,
                        snap.settings.gamemode.renice
                    ),
                    theme::COPPER,
                );
            });
        });
    });

    ui.add_space(14.0);

    // --- MangoHud ---------------------------------------------------------
    ui::card(ui, "MangoHud", |ui| {
        ui.label(
            RichText::new(
                "Overlay-Profil für FPS, Frametimes und Temperaturen. Die Konfiguration \
                 landet im Benutzerprofil und in /etc/skel für neue Konten.",
            )
            .size(12.5)
            .color(theme::MUTED),
        );
        ui.add_space(10.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            for (value, label) in [
                ("standard", "Standard"),
                ("compact", "Kompakt"),
                ("detailed", "Ausführlich"),
                ("off", "Aus"),
            ] {
                ui.radio_value(&mut cc.mangohud_choice, value.to_string(), label);
            }
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui::hint(ui, "Vorschau");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui::badge(
                    ui,
                    &match snap.settings.mangohud_preset.as_str() {
                        "custom" => "eigene Konfiguration".to_string(),
                        "missing" => "keine Konfiguration".to_string(),
                        preset => format!("aktiv: {preset}"),
                    },
                    theme::AMBER,
                );
            });
        });
        ui.add_space(6.0);
        ui::mono_block(
            ui,
            "mangohud-preview",
            preset_conf(&cc.mangohud_choice),
            160.0,
        );

        if !snap.settings.mangohud_conf.trim().is_empty() {
            ui.add_space(8.0);
            egui::CollapsingHeader::new(RichText::new("Aktive Konfiguration").size(12.0))
                .id_salt("mangohud-active")
                .show(ui, |ui| {
                    ui.add_space(6.0);
                    ui::mono_block(ui, "mangohud-active", &snap.settings.mangohud_conf, 180.0);
                });
        }

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!cc.action_pending(), |ui| {
                if ui::primary(ui, "Overlay-Profil anwenden").clicked() {
                    let args = vec!["mangohud".to_string(), cc.mangohud_choice.clone()];
                    cc.apply_helper(args, "MangoHud-Profil");
                }
            });
            if cc.action_pending() {
                ui.spinner();
            }
        });
    });

    ui.add_space(14.0);

    // --- Startoptionen ----------------------------------------------------
    ui::card(ui, "Startoptionen für Spiele", |ui| {
        ui.label(
            RichText::new(
                "Diesen Befehl in Steam unter „Eigenschaften → Startoptionen“ eintragen. \
                 %command% wird von Steam ersetzt.",
            )
            .size(12.5)
            .color(theme::MUTED),
        );
        ui.add_space(10.0);

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            ui.checkbox(&mut cc.run_options.hud, "MangoHud-Overlay");
            ui.checkbox(&mut cc.run_options.gamemode, "GameMode");
            ui.checkbox(&mut cc.run_options.gamescope, "Gamescope");
            ui.add_enabled_ui(cc.run_options.gamescope, |ui| {
                ui.checkbox(&mut cc.run_options.windowed, "Fenstermodus");
            });
        });

        if cc.run_options.gamescope {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Auflösung").size(12.5).color(theme::MUTED));
                egui::ComboBox::from_id_salt("gamescope-res")
                    .width(140.0)
                    .selected_text(cc.run_options.resolution.clone())
                    .show_ui(ui, |ui| {
                        for value in ["1280x720", "1920x1080", "2560x1440", "3840x2160"] {
                            ui.selectable_value(
                                &mut cc.run_options.resolution,
                                value.to_string(),
                                value,
                            );
                        }
                    });
            });
        }

        let command = cc.run_options.command();
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui::hint(ui, "Fertiger Befehl");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui::copy_button(ui, &command, "Kopieren");
            });
        });
        ui.add_space(6.0);
        ui::mono_block(ui, "run-command", &command, 70.0);

        if !snap.gaming.gamemode && cc.run_options.gamemode {
            ui.add_space(10.0);
            ui::notice(
                ui,
                crate::icons::Icon::Warn,
                theme::WARN,
                "GameMode ist nicht installiert – playforge-run bricht mit Hinweis ab.",
            );
        }
    });

    ui.add_space(14.0);

    ui::card(ui, "Gaming-Stack", |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            ui::chip(ui, snap.gaming.gamemode, "GameMode");
            ui::chip(ui, snap.gaming.mangohud, "MangoHud");
            ui::chip(ui, snap.gaming.gamescope, "Gamescope");
            ui::chip(ui, snap.gaming.steam, "Steam");
            ui::chip(ui, snap.gaming.lutris, "Lutris");
            ui::chip(ui, snap.gaming.wine, "Wine");
            ui::chip(ui, snap.gaming.goverlay, "GOverlay");
            ui::chip(ui, snap.gaming.lact, "LACT");
        });
        ui.add_space(10.0);
        ui::divider(ui);
        ui.add_space(10.0);
        ui::kv(
            ui,
            "Vulkan",
            &if snap.vulkan.icds.is_empty() {
                "keine ICD gefunden".to_string()
            } else {
                snap.vulkan.icds.join(", ")
            },
        );
        ui::kv(
            ui,
            "32-Bit",
            if snap.vulkan.lib32 {
                "lib32-Vulkan vorhanden (Steam/Proton)"
            } else {
                "lib32-Vulkan fehlt – Steam benötigt lib32-vulkan-*"
            },
        );
        ui::kv(
            ui,
            "Multi-Monitor",
            "Gamescope startet Spiele mit eigener Auflösung",
        );
        ui::kv(
            ui,
            "Shader-Cache",
            if snap.settings.shader_cache.is_empty() {
                "Standard"
            } else {
                &snap.settings.shader_cache
            },
        );
    });
}

/// Vorschau der MangoHud-Konfiguration – identisch zum Helfer-Skript.
pub fn preset_conf(preset: &str) -> &'static str {
    match preset {
        "compact" => {
            "# Playforge-Preset: compact\n\
             gpu_stats\n\
             cpu_stats\n\
             fps\n\
             frametime\n\
             position=top-right\n\
             font_size=14\n\
             background_alpha=0.25\n\
             round_corners=6\n"
        }
        "detailed" => {
            "# Playforge-Preset: detailed\n\
             gpu_stats\n\
             gpu_temp\n\
             gpu_power\n\
             gpu_clock\n\
             cpu_stats\n\
             cpu_temp\n\
             cpu_power\n\
             cpu_mhz\n\
             ram\n\
             vram\n\
             fps\n\
             frametime\n\
             frame_timing\n\
             position=top-left\n\
             font_size=16\n\
             background_alpha=0.4\n\
             round_corners=8\n\
             toggle_hud=Shift_R+F12\n"
        }
        "off" => "# Playforge-Preset: off\nno_display\n",
        _ => {
            "# Playforge-Preset: standard\n\
             gpu_stats\n\
             cpu_stats\n\
             ram\n\
             vram\n\
             fps\n\
             frametime\n\
             frame_timing\n\
             gpu_temp\n\
             cpu_temp\n\
             position=top-left\n\
             font_size=18\n\
             background_alpha=0.35\n\
             round_corners=8\n"
        }
    }
}
