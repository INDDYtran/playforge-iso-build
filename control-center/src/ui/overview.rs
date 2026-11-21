//! Übersicht: Kennzahlen, Spielbereitschaft und Systemdaten.

use eframe::egui::{self, RichText, Sense, Vec2};

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
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        let width = ((ui.available_width() - 36.0) / 4.0).max(150.0);

        let cpu_value = match (snap.cpu.cores, snap.cpu.threads) {
            (Some(cores), Some(threads)) if threads > cores => format!("{cores}C / {threads}T"),
            (Some(cores), _) => format!("{cores} Kerne"),
            _ => "—".to_string(),
        };
        let cpu_sub = if snap.cpu.governor == "—" && snap.cpu.freq_mhz.is_none() {
            "keine CPU-Daten".to_string()
        } else {
            format!("{} · {}", snap.cpu.freq_text(), snap.cpu.governor)
        };
        ui::tile(
            ui,
            width,
            Icon::Cpu,
            "Prozessor",
            &cpu_value,
            &cpu_sub,
            theme::AMBER,
        );

        let (ram_value, ram_sub) = if snap.mem.total_kb == 0 {
            (
                "—".to_string(),
                "ohne /proc/meminfo nicht lesbar".to_string(),
            )
        } else {
            (
                format!(
                    "{} / {}",
                    fmt_kb(snap.mem.used_kb()),
                    fmt_kb(snap.mem.total_kb)
                ),
                format!("{:.0} % belegt", snap.mem.ram_fraction() * 100.0),
            )
        };
        ui::tile(
            ui,
            width,
            Icon::Memory,
            "Arbeitsspeicher",
            &ram_value,
            &ram_sub,
            load_color(snap.mem.ram_fraction()),
        );

        let (gpu_label, gpu_sub) = match snap.gpus.first() {
            Some(gpu) => (
                gpu.vendor.map(|v| v.label()).unwrap_or("GPU").to_string(),
                truncate(&gpu.name, 26),
            ),
            None => ("GPU".to_string(), "nicht erkannt".to_string()),
        };
        ui::tile(
            ui,
            width,
            Icon::Monitor,
            "Grafik",
            &gpu_label,
            gpu_sub.as_str(),
            snap.gpus
                .first()
                .and_then(|gpu| gpu.vendor)
                .map(|v| v.color())
                .unwrap_or(theme::COPPER),
        );

        let profile = snap.tuning.profile_saved.to_lowercase();
        let profile_label = if profile.starts_with("perf") {
            "Performance"
        } else {
            "Ausgewogen"
        };
        ui::tile(
            ui,
            width,
            Icon::Bolt,
            "Leistungsprofil",
            profile_label,
            &format!("Governor: {}", snap.tuning.governor),
            if profile.starts_with("perf") {
                theme::AMBER
            } else {
                theme::GREEN
            },
        );
    });

    ui.add_space(14.0);

    // --- Spielbereitschaft ------------------------------------------------
    ui::card(ui, "Spielbereitschaft", |ui| {
        ui.label(
            RichText::new("Was für Spiele, Proton und Steam bereits eingerichtet ist.")
                .size(12.5)
                .color(theme::MUTED),
        );
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);

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

            ui::chip(ui, !snap.vulkan.icds.is_empty(), "Vulkan-Treiber");
            ui::chip(ui, snap.vulkan.lib32, "32-Bit-Vulkan");
            ui::chip(ui, snap.packages.multilib, "[multilib]");
            ui::chip(
                ui,
                snap.tuning.max_map_count.unwrap_or(0) >= 2_147_483_642,
                "Proton-Speicherlimit",
            );
            ui::chip(ui, !snap.mem.zram.is_empty(), "zram-Swap");
        });
    });

    ui.add_space(14.0);

    // --- System -----------------------------------------------------------
    ui::card(ui, "System", |ui| {
        let secure_boot = match snap.os.secure_boot {
            Some(true) => "aktiv",
            Some(false) => "aus",
            None => "nicht lesbar / BIOS",
        };
        ui::kv_grid(
            ui,
            &[
                ("Distribution", snap.os.distro.clone()),
                ("Kernel", snap.os.kernel.clone()),
                ("Architektur", snap.os.arch.clone()),
                ("Hostname", snap.os.hostname.clone()),
                (
                    "Laufzeit",
                    snap.os
                        .uptime_secs
                        .map(fmt_uptime)
                        .unwrap_or_else(|| "—".to_string()),
                ),
                ("Sitzung", session_label(&snap.os.session).to_string()),
                ("Bootmodus", snap.os.boot_mode.clone()),
                ("Secure Boot", secure_boot.to_string()),
                ("systemd", snap.os.systemd.clone()),
                (
                    "Pakete",
                    match snap.packages.installed {
                        Some(count) => format!("{count} installiert"),
                        None => "—".to_string(),
                    },
                ),
                (
                    "Updates",
                    match snap.packages.updates {
                        Some(0) => "aktuell".to_string(),
                        Some(count) => format!("{count} verfügbar"),
                        None => "—".to_string(),
                    },
                ),
                (
                    "Paketstand",
                    snap.packages
                        .last_sync
                        .map(fmt_age)
                        .unwrap_or_else(|| "—".to_string()),
                ),
                (
                    "AUR-Helfer",
                    snap.packages
                        .aur_helper
                        .clone()
                        .unwrap_or_else(|| "nicht installiert".to_string()),
                ),
                (
                    "Farbschema",
                    if snap.settings.color_scheme.is_empty() {
                        "KDE-Standard".to_string()
                    } else {
                        snap.settings.color_scheme.clone()
                    },
                ),
            ],
        );
    });

    ui.add_space(14.0);

    // --- Prozessor und Speicher ------------------------------------------
    ui::card(ui, "Prozessor", |ui| {
        ui::kv(ui, "Modell", &snap.cpu.model);
        ui::kv(
            ui,
            "Kerne / Threads",
            &match (snap.cpu.cores, snap.cpu.threads) {
                (Some(cores), Some(threads)) => format!("{cores} / {threads}"),
                _ => "—".to_string(),
            },
        );
        ui::kv(ui, "Governor", &snap.cpu.governor);
        ui::kv(ui, "Energy-Performance", &snap.cpu.epp);
        ui::kv(ui, "Takt", &snap.cpu.freq_text());
        ui::kv(ui, "Temperatur", &snap.cpu.temp_text());
        if let Some(load) = snap.cpu.load {
            ui::kv(
                ui,
                "Last (1/5/15)",
                &format!("{:.2}  ·  {:.2}  ·  {:.2}", load[0], load[1], load[2]),
            );
        }
    });

    ui.add_space(14.0);

    ui::card(ui, "Arbeitsspeicher", |ui| {
        ui::meter(
            ui,
            "RAM",
            &format!(
                "{} von {}",
                fmt_kb(snap.mem.used_kb()),
                fmt_kb(snap.mem.total_kb)
            ),
            snap.mem.ram_fraction(),
            load_color(snap.mem.ram_fraction()),
        );
        ui.add_space(10.0);
        ui::meter(
            ui,
            "Swap / zram",
            &if snap.mem.swap_total_kb > 0 {
                format!(
                    "{} von {}",
                    fmt_kb(snap.mem.swap_total_kb.saturating_sub(snap.mem.swap_free_kb)),
                    fmt_kb(snap.mem.swap_total_kb)
                )
            } else {
                "kein Swap aktiv".to_string()
            },
            snap.mem.swap_fraction(),
            theme::COPPER,
        );
        if !snap.mem.zram.is_empty() {
            ui.add_space(8.0);
            ui::hint(ui, &format!("zram: {}", snap.mem.zram));
        }
    });

    ui.add_space(14.0);

    // --- Grafik -----------------------------------------------------------
    ui::card(ui, "Grafik", |ui| {
        if snap.gpus.is_empty() {
            ui::hint(
                ui,
                "Keine GPU über lspci gefunden. Auf anderen Systemen als Playforge Linux \
                 bleibt diese Anzeige leer.",
            );
        } else {
            for gpu in &snap.gpus {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                    crate::icons::paint(ui.painter(), rect, Icon::Monitor, theme::AMBER);
                    ui.label(RichText::new(&gpu.name).size(13.5).color(theme::TEXT));
                    if let Some(vendor) = gpu.vendor {
                        ui::badge(ui, vendor.label(), vendor.color());
                    }
                    if let Some(driver) = &gpu.driver {
                        ui::badge(ui, driver, theme::GREEN);
                    }
                });
            }
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                for icd in &snap.vulkan.icds {
                    ui::badge(ui, icd, theme::AMBER);
                }
                if snap.vulkan.lib32 {
                    ui::badge(ui, "lib32-Vulkan", theme::GREEN);
                }
                if snap.gaming.lact {
                    ui::badge(ui, "LACT", theme::COPPER);
                }
            });
        }
    });

    ui.add_space(14.0);

    // --- Schnellaktionen --------------------------------------------------
    ui::card(ui, "Schnellaktionen", |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            if ui::primary(ui, "Diagnose starten").clicked() {
                cc.tab = Tab::Diagnose;
                cc.start_doctor();
            }
            if ui::ghost(ui, "Theme neu anwenden").clicked() {
                cc.apply_theme();
            }
            if ui::icon_button(ui, Icon::Refresh, "Neu einlesen", theme::AMBER).clicked() {
                cc.schedule_refresh();
            }
        });
        ui.add_space(10.0);
        ui::divider(ui);
        ui.add_space(10.0);
        ui::hint(
            ui,
            "Playforge Control Center 1.0 – Systemstatus, Leistung und Gaming-Einstellungen. \
             Änderungen an Systemdateien benötigen Administratorrechte.",
        );
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
