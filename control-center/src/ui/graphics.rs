//! Grafik: GPUs, Vulkan und NVIDIA.

use eframe::egui::{self, RichText, Sense, Vec2};

use crate::app::ControlCenter;
use crate::icons::Icon;
use crate::theme;
use crate::ui;

pub fn show(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let Some(snap) = cc.snapshot.clone() else {
        ui::skeleton(ui);
        return;
    };

    ui::panel_titled(
        ui,
        "Grafikprozessoren",
        &format!("{}", snap.gpus.len()),
        |ui| {
            if snap.gpus.is_empty() {
                ui::hint(
                    ui,
                    "Keine GPU über lspci gefunden. Diese Anzeige benötigt Playforge Linux.",
                );
                return;
            }

            ui::table_header(ui, &[("Gerät", 2.6), ("Hersteller", 0.9), ("Treiber", 1.1)]);
            for gpu in &snap.gpus {
                ui::table_row(ui, &[2.6, 0.9, 1.1], |ui, column| match column {
                    0 => {
                        ui.horizontal(|ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::splat(15.0), Sense::hover());
                            crate::icons::paint(ui.painter(), rect, Icon::Monitor, theme::ACCENT);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&gpu.name)
                                        .font(theme::sans(12.5))
                                        .color(theme::TEXT),
                                )
                                .wrap(),
                            );
                        });
                    }
                    1 => {
                        if let Some(vendor) = gpu.vendor {
                            ui::badge(ui, vendor.label(), vendor.color());
                        } else {
                            ui::mono_label(ui, "—", theme::FAINT, 10.0);
                        }
                    }
                    _ => {
                        if let Some(driver) = &gpu.driver {
                            ui::badge(ui, driver, theme::OK);
                        } else {
                            ui::mono_label(ui, "kein Treiber", theme::ERROR, 9.0);
                        }
                    }
                });
            }
        },
    );

    ui.add_space(ui::GAP);

    ui::panel_titled(ui, "Vulkan", "icd.d", |ui| {
        if snap.vulkan.icds.is_empty() {
            ui::notice(
                ui,
                Icon::Cross,
                theme::ERROR,
                "Keine Vulkan-ICD unter /usr/share/vulkan/icd.d gefunden – Steam/Proton \
                 startet ohne Vulkan nicht.",
            );
            ui.add_space(12.0);
        } else {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                for icd in &snap.vulkan.icds {
                    ui::badge(ui, icd, theme::ACCENT);
                }
            });
            ui.add_space(12.0);
        }

        ui::table_header(ui, &[("Prüfung", 1.6), ("Ergebnis", 1.4)]);
        for (label, value, ok) in [
            (
                "ICD-DATEIEN",
                snap.vulkan.icds.len().to_string(),
                !snap.vulkan.icds.is_empty(),
            ),
            (
                "32-BIT-VULKAN",
                if snap.vulkan.lib32 {
                    "vorhanden".to_string()
                } else {
                    "fehlt".to_string()
                },
                snap.vulkan.lib32,
            ),
            (
                "VULKANINFO",
                if snap.vulkan.summary.is_some() {
                    "läuft".to_string()
                } else {
                    "nicht verfügbar".to_string()
                },
                snap.vulkan.summary.is_some(),
            ),
            (
                "VA-API",
                if snap.vulkan.vainfo {
                    "vainfo vorhanden".to_string()
                } else {
                    "vainfo fehlt".to_string()
                },
                snap.vulkan.vainfo,
            ),
            (
                "[MULTILIB]",
                if snap.packages.multilib {
                    "aktiv".to_string()
                } else {
                    "nicht aktiv".to_string()
                },
                snap.packages.multilib,
            ),
        ] {
            ui::table_row(ui, &[1.6, 1.4], |ui, column| {
                if column == 0 {
                    ui::mono_label(ui, label, theme::MUTED, 10.0);
                } else {
                    ui::badge(ui, &value, if ok { theme::OK } else { theme::ERROR });
                }
            });
        }

        if let Some(summary) = &snap.vulkan.summary {
            ui.add_space(12.0);
            egui::CollapsingHeader::new(
                RichText::new("vulkaninfo --summary")
                    .font(theme::mono(10.0))
                    .color(theme::MUTED),
            )
            .id_salt("vulkan-summary")
            .show(ui, |ui| {
                ui.add_space(6.0);
                ui::code_block(ui, "vulkan-summary", "vulkaninfo", summary, 240.0);
            });
        }
    });

    if let Some(nvidia) = &snap.nvidia_smi {
        ui.add_space(ui::GAP);
        ui::panel_titled(ui, "NVIDIA", "nvidia-smi", |ui| {
            ui::code_block(ui, "nvidia-smi", "nvidia-smi", nvidia, 120.0);
        });
    }

    ui.add_space(ui::GAP);

    ui::panel_titled(ui, "Hinweise", "", |ui| {
        ui::kv(
            ui,
            "TREIBER",
            "AMD/Intel nutzen Mesa (RADV/ANV), NVIDIA den open-dkms-Treiber.",
        );
        ui::kv(
            ui,
            "32-BIT",
            "Steam und Proton benötigen lib32-mesa und lib32-vulkan-*.",
        );
        ui::kv(
            ui,
            "LÜFTERKURVEN",
            if snap.gaming.lact {
                "LACT ist installiert"
            } else {
                "LACT nicht gefunden"
            },
        );
        ui::kv(
            ui,
            "LIVE-MEDIUM",
            "Das Live-System läuft bei NVIDIA ohne proprietären Treiber (nouveau/NVK).",
        );
    });
}
