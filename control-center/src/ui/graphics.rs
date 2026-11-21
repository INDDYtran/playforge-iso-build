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

    ui::card(ui, "Grafikprozessoren", |ui| {
        if snap.gpus.is_empty() {
            ui::hint(
                ui,
                "Keine GPU über lspci gefunden. Diese Anzeige benötigt Playforge Linux.",
            );
        } else {
            for gpu in &snap.gpus {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 6.0);
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
                    crate::icons::paint(ui.painter(), rect, Icon::Monitor, theme::WARN);
                    ui.label(RichText::new(&gpu.name).size(14.0).color(theme::TEXT));
                    if let Some(vendor) = gpu.vendor {
                        ui::badge(ui, vendor.label(), vendor.color());
                    }
                    if let Some(driver) = &gpu.driver {
                        ui::badge(ui, driver, theme::GREEN);
                    }
                });
                ui.label(
                    RichText::new(match &gpu.driver {
                        Some(driver) => format!("Treiber: {driver}"),
                        None => "kein Kernel-Treiber erkannt".to_string(),
                    })
                    .size(11.5)
                    .color(theme::MUTED),
                );
                ui.add_space(8.0);
            }
        }
    });

    ui.add_space(14.0);

    ui::card(ui, "Vulkan", |ui| {
        if snap.vulkan.icds.is_empty() {
            ui::notice(
                ui,
                Icon::Cross,
                theme::RED,
                "Keine Vulkan-ICD unter /usr/share/vulkan/icd.d gefunden – Steam/Proton \
                 startet ohne Vulkan nicht.",
            );
        } else {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                for icd in &snap.vulkan.icds {
                    ui::badge(ui, icd, theme::AMBER);
                }
            });
        }
        ui.add_space(10.0);
        ui::kv_grid(
            ui,
            &[
                ("Gefundene ICDs", snap.vulkan.icds.len().to_string()),
                (
                    "32-Bit-Vulkan",
                    if snap.vulkan.lib32 {
                        "vorhanden".to_string()
                    } else {
                        "fehlt".to_string()
                    },
                ),
                (
                    "vulkaninfo",
                    if snap.vulkan.summary.is_some() {
                        "läuft".to_string()
                    } else {
                        "nicht verfügbar".to_string()
                    },
                ),
                (
                    "VA-API",
                    if snap.vulkan.vainfo {
                        "vainfo vorhanden".to_string()
                    } else {
                        "vainfo fehlt".to_string()
                    },
                ),
                (
                    "[multilib]",
                    if snap.packages.multilib {
                        "aktiv".to_string()
                    } else {
                        "nicht aktiv".to_string()
                    },
                ),
            ],
        );

        if let Some(summary) = &snap.vulkan.summary {
            ui.add_space(10.0);
            egui::CollapsingHeader::new(RichText::new("vulkaninfo --summary").size(12.5))
                .id_salt("vulkan-summary")
                .show(ui, |ui| {
                    ui.add_space(6.0);
                    ui::mono_block(ui, "vulkan-summary", summary, 260.0);
                });
        }
    });

    ui.add_space(14.0);

    if let Some(nvidia) = &snap.nvidia_smi {
        ui::card(ui, "NVIDIA", |ui| {
            ui::hint(
                ui,
                "Ausgabe von nvidia-smi (Name, Treiberversion, VRAM, Temperatur).",
            );
            ui.add_space(8.0);
            ui::mono_block(ui, "nvidia-smi", nvidia, 120.0);
        });
        ui.add_space(14.0);
    }

    ui::card(ui, "Hinweise", |ui| {
        ui::kv(
            ui,
            "Treiber",
            "AMD/Intel nutzen Mesa (RADV/ANV), NVIDIA den open-dkms-Treiber.",
        );
        ui::kv(
            ui,
            "32-Bit",
            "Steam und Proton benötigen lib32-mesa und lib32-vulkan-*.",
        );
        ui::kv(
            ui,
            "Lüfterkurven",
            if snap.gaming.lact {
                "LACT ist installiert"
            } else {
                "LACT nicht gefunden"
            },
        );
        ui::kv(
            ui,
            "Live-Medium",
            "Das Live-System läuft bei NVIDIA ohne proprietären Treiber (nouveau/NVK).",
        );
    });
}
