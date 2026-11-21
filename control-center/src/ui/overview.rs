//! Übersicht: Kennzahlen, Spielbereitschaft und Systemdaten.

use eframe::egui::{self, Align, Layout, RichText, Sense, Vec2};

use crate::app::{ControlCenter, Tab};
use crate::icons::Icon;
use crate::model::{fmt_age, fmt_kb, fmt_uptime, load_color};
use crate::theme;
use crate::ui::{self, truncate};

pub fn show(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let Some(snap) = cc.snapshot.clone() else {
        ui::skeleton(ui);
        return;
    };

    // --- Kennzahlen -------------------------------------------------------
    ui::grid(ui, &[1.0, 1.0, 1.0, 1.0], |ui, column| match column {
        0 => {
            let value = match (snap.cpu.cores, snap.cpu.threads) {
                (Some(cores), Some(threads)) if threads > cores => format!("{cores}C/{threads}T"),
                (Some(cores), _) => format!("{cores} Kerne"),
                _ => "—".to_string(),
            };
            let sub = if snap.cpu.governor == "—" && snap.cpu.freq_mhz.is_none() {
                "keine CPU-Daten".to_string()
            } else {
                format!("{} · {}", snap.cpu.freq_text(), snap.cpu.governor)
            };
            ui::metric(ui, "Prozessor", &value, &sub, theme::ACCENT);
        }
        1 => {
            let (value, sub) = if snap.mem.total_kb == 0 {
                (
                    "—".to_string(),
                    "ohne /proc/meminfo nicht lesbar".to_string(),
                )
            } else {
                (
                    fmt_kb(snap.mem.used_kb()),
                    format!(
                        "{} · {:.0} %",
                        fmt_kb(snap.mem.total_kb),
                        snap.mem.ram_fraction() * 100.0
                    ),
                )
            };
            ui::metric(
                ui,
                "Speicher",
                &value,
                &sub,
                load_color(snap.mem.ram_fraction()),
            );
        }
        2 => {
            let (label, sub) = match snap.gpus.first() {
                Some(gpu) => (
                    gpu.vendor.map(|v| v.label()).unwrap_or("GPU").to_string(),
                    truncate(&gpu.name, 24),
                ),
                None => ("GPU".to_string(), "nicht erkannt".to_string()),
            };
            ui::metric(
                ui,
                "Grafik",
                &label,
                &sub,
                snap.gpus
                    .first()
                    .and_then(|gpu| gpu.vendor)
                    .map(|v| v.color())
                    .unwrap_or(theme::INFO),
            );
        }
        _ => {
            let profile = snap.tuning.profile_saved.to_lowercase();
            let (label, accent) = if profile.starts_with("perf") {
                ("Performance", theme::ACCENT)
            } else if profile.starts_with("reset") {
                ("Zurückgesetzt", theme::MUTED)
            } else {
                ("Ausgewogen", theme::OK)
            };
            ui::metric(
                ui,
                "Profil",
                label,
                &format!("GOV {}", snap.tuning.governor.to_uppercase()),
                accent,
            );
        }
    });

    ui.add_space(ui::GAP);

    // --- Spielbereitschaft und System -------------------------------------
    ui::grid(ui, &[1.0, 1.3], |ui, column| {
        if column == 0 {
            ui::panel_titled(ui, "Spielbereitschaft", "", |ui| {
                ui::hint(
                    ui,
                    "Was für Spiele, Proton und Steam bereits eingerichtet ist.",
                );
                ui.add_space(11.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);

                    let zen = snap.os.kernel.contains("zen") || snap.os.kernel.contains("cachyos");
                    ui::chip(
                        ui,
                        zen,
                        &format!("Kernel {}", short_kernel(&snap.os.kernel)),
                    );
                    ui::chip(ui, snap.gaming.gamemode, "GameMode");
                    ui::chip(ui, snap.gaming.mangohud, "MangoHud");
                    ui::chip(ui, snap.gaming.gamescope, "Gamescope");
                    ui::chip(ui, snap.gaming.steam, "Steam");
                    ui::chip(ui, snap.gaming.lutris, "Lutris");
                    ui::chip(ui, snap.gaming.wine, "Wine");
                    ui::chip(ui, !snap.vulkan.icds.is_empty(), "Vulkan");
                    ui::chip(ui, snap.vulkan.lib32, "32-Bit-Vulkan");
                    ui::chip(ui, snap.packages.multilib, "[multilib]");
                    ui::chip(
                        ui,
                        snap.tuning.max_map_count.unwrap_or(0) >= 2_147_483_642,
                        "Proton-Limit",
                    );
                    ui::chip(ui, !snap.mem.zram.is_empty(), "zram-Swap");
                });
            });
        } else {
            let secure_boot = match snap.os.secure_boot {
                Some(true) => "aktiv",
                Some(false) => "aus",
                None => "n/a",
            };
            ui::panel_titled(ui, "System", "live", |ui| {
                for (label, value) in [
                    ("DISTRIBUTION", snap.os.distro.clone()),
                    ("KERNEL", snap.os.kernel.clone()),
                    ("ARCHITEKTUR", snap.os.arch.clone()),
                    ("HOSTNAME", snap.os.hostname.clone()),
                    (
                        "LAUFZEIT",
                        snap.os
                            .uptime_secs
                            .map(fmt_uptime)
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                    ("SITZUNG", session_label(&snap.os.session).to_string()),
                    ("BOOTMODUS", snap.os.boot_mode.clone()),
                    ("SECURE BOOT", secure_boot.to_string()),
                    ("SYSTEMD", snap.os.systemd.clone()),
                    (
                        "PAKETE",
                        match snap.packages.installed {
                            Some(count) => format!("{count} installiert"),
                            None => "—".to_string(),
                        },
                    ),
                    (
                        "UPDATES",
                        match snap.packages.updates {
                            Some(0) => "aktuell".to_string(),
                            Some(count) => format!("{count} verfügbar"),
                            None => "—".to_string(),
                        },
                    ),
                    (
                        "PAKETSTAND",
                        snap.packages
                            .last_sync
                            .map(fmt_age)
                            .unwrap_or_else(|| "—".to_string()),
                    ),
                    (
                        "AUR-HELFER",
                        snap.packages
                            .aur_helper
                            .clone()
                            .unwrap_or_else(|| "nicht installiert".to_string()),
                    ),
                    (
                        "FARBSCHEMA",
                        if snap.settings.color_scheme.is_empty() {
                            "KDE-Standard".to_string()
                        } else {
                            snap.settings.color_scheme.clone()
                        },
                    ),
                ] {
                    ui::kv(ui, label, &value);
                }
            });
        }
    });

    ui.add_space(ui::GAP);

    // --- Prozessor und Speicher ------------------------------------------
    ui::grid(ui, &[1.0, 1.0], |ui, column| {
        if column == 0 {
            ui::panel_titled(ui, "Prozessor", "", |ui| {
                ui::kv(ui, "MODELL", &snap.cpu.model);
                ui::kv(
                    ui,
                    "KERNE / THREADS",
                    &match (snap.cpu.cores, snap.cpu.threads) {
                        (Some(cores), Some(threads)) => format!("{cores} / {threads}"),
                        _ => "—".to_string(),
                    },
                );
                ui::kv(ui, "GOVERNOR", &snap.cpu.governor);
                ui::kv(ui, "EPP", &snap.cpu.epp);
                ui::kv(ui, "TAKT", &snap.cpu.freq_text());
                ui::kv(ui, "TEMPERATUR", &snap.cpu.temp_text());
                if let Some(load) = snap.cpu.load {
                    ui::kv(
                        ui,
                        "LAST 1/5/15",
                        &format!("{:.2} · {:.2} · {:.2}", load[0], load[1], load[2]),
                    );
                }
            });
        } else {
            ui::panel_titled(ui, "Arbeitsspeicher", "kernel", |ui| {
                ui::meter(
                    ui,
                    "RAM",
                    &format!(
                        "{} / {}",
                        fmt_kb(snap.mem.used_kb()),
                        fmt_kb(snap.mem.total_kb)
                    ),
                    snap.mem.ram_fraction(),
                    load_color(snap.mem.ram_fraction()),
                );
                ui.add_space(14.0);
                ui::meter(
                    ui,
                    "SWAP / ZRAM",
                    &if snap.mem.swap_total_kb > 0 {
                        format!(
                            "{} / {}",
                            fmt_kb(snap.mem.swap_total_kb.saturating_sub(snap.mem.swap_free_kb)),
                            fmt_kb(snap.mem.swap_total_kb)
                        )
                    } else {
                        "kein Swap aktiv".to_string()
                    },
                    snap.mem.swap_fraction(),
                    theme::INFO,
                );
                if !snap.mem.zram.is_empty() {
                    ui.add_space(12.0);
                    ui::hairline(ui);
                    ui.add_space(10.0);
                    ui::kv(ui, "ZRAM", &snap.mem.zram);
                }
            });
        }
    });

    ui.add_space(ui::GAP);

    // --- Grafik -----------------------------------------------------------
    ui::panel_titled(
        ui,
        "Grafik",
        &format!("{} ICD", snap.vulkan.icds.len()),
        |ui| {
            if snap.gpus.is_empty() {
                ui::hint(
                    ui,
                    "Keine GPU über lspci gefunden. Auf anderen Systemen als Playforge Linux \
                 bleibt diese Anzeige leer.",
                );
                return;
            }
            for gpu in &snap.gpus {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                    crate::icons::paint(ui.painter(), rect, Icon::Monitor, theme::ACCENT);
                    ui.label(
                        RichText::new(&gpu.name)
                            .font(theme::sans(12.5))
                            .color(theme::TEXT),
                    );
                    if let Some(vendor) = gpu.vendor {
                        ui::badge(ui, vendor.label(), vendor.color());
                    }
                    if let Some(driver) = &gpu.driver {
                        ui::badge(ui, driver, theme::OK);
                    }
                });
            }
            ui.add_space(10.0);
            ui::hairline(ui);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                for icd in &snap.vulkan.icds {
                    ui::badge(ui, icd, theme::ACCENT);
                }
                if snap.vulkan.lib32 {
                    ui::badge(ui, "lib32-Vulkan", theme::OK);
                }
                if snap.gaming.lact {
                    ui::badge(ui, "LACT", theme::INFO);
                }
            });
        },
    );

    ui.add_space(ui::GAP);

    // --- Schnellaktionen --------------------------------------------------
    ui::panel_titled(ui, "Schnellaktionen", "", |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if ui::primary(ui, "Diagnose starten").clicked() {
                cc.tab = Tab::Diagnose;
                cc.start_doctor();
            }
            if ui::ghost(ui, "Theme neu anwenden").clicked() {
                cc.apply_theme();
            }
            if ui::icon_button(ui, Icon::Refresh, "Neu einlesen", theme::ACCENT).clicked() {
                cc.schedule_refresh();
            }
        });
        ui.add_space(12.0);
        ui::hairline(ui);
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui::mono_label(ui, "PLAYFORGE CONTROL CENTER 1.0", theme::FAINT, 9.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui::mono_label(
                    ui,
                    "SYSTEMÄNDERUNGEN NUR MIT ADMINISTRATORRECHTEN",
                    theme::FAINT,
                    9.0,
                );
            });
        });
    });
}

fn short_kernel(kernel: &str) -> String {
    if kernel.is_empty() {
        "—".to_string()
    } else {
        truncate(kernel, 18)
    }
}

fn session_label(session: &str) -> &str {
    match session {
        "wayland" => "Wayland",
        "x11" => "X11",
        "tty" => "Konsole",
        other => other,
    }
}
