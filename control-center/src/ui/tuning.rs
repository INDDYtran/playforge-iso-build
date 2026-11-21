//! Feintuning: CPU, Kernel, Datenträger und Grafik-Umgebung.

use eframe::egui::{self, RichText};

use crate::app::ControlCenter;
use crate::model::Snapshot;
use crate::theme;
use crate::ui;

const MAX_MAP_STANDARD: u32 = 65_530;
const MAX_MAP_PROTON: u32 = 2_147_483_642;

pub fn show(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    let Some(snap) = cc.snapshot.clone() else {
        ui::skeleton(ui);
        return;
    };

    ui::helper_notice(cc, ui);

    ui::grid(ui, &[1.0, 1.0], |ui, column| {
        if column == 0 {
            cpu_card(cc, ui, &snap);
        } else {
            memory_card(cc, ui);
        }
    });

    ui.add_space(ui::GAP);

    ui::grid(ui, &[1.0, 1.0], |ui, column| {
        if column == 0 {
            io_card(cc, ui, &snap);
        } else {
            graphics_card(cc, ui);
        }
    });
}

fn cpu_card(cc: &mut ControlCenter, ui: &mut egui::Ui, snap: &Snapshot) {
    let mut governors = snap.tuning.governors.clone();
    if governors.is_empty() {
        governors = [
            "performance",
            "powersave",
            "schedutil",
            "ondemand",
            "conservative",
        ]
        .iter()
        .map(|value| value.to_string())
        .collect();
    }
    let mut epps = snap.tuning.epps.clone();
    if epps.is_empty() {
        epps = [
            "default",
            "performance",
            "balance_performance",
            "balance_power",
            "power",
        ]
        .iter()
        .map(|value| value.to_string())
        .collect();
    }

    ui::panel_titled(ui, "Prozessor", "cpufreq", |ui| {
        ui::hint(
            ui,
            "Governor und Energy-Performance-Preference werden sofort gesetzt und von \
             playforge-cpu-tune.service bei jedem Boot wiederhergestellt.",
        );
        ui.add_space(12.0);

        ui::kicker(ui, "Governor");
        ui.add_space(4.0);
        egui::ComboBox::from_id_salt("tune-governor")
            .width(ui.available_width())
            .selected_text(cc.tune.governor.clone())
            .show_ui(ui, |ui| {
                for value in &governors {
                    ui.selectable_value(&mut cc.tune.governor, value.clone(), value);
                }
            });

        ui.add_space(10.0);
        ui::kicker(ui, "Energy-Performance-Preference");
        ui.add_space(4.0);
        egui::ComboBox::from_id_salt("tune-epp")
            .width(ui.available_width())
            .selected_text(cc.tune.epp.clone())
            .show_ui(ui, |ui| {
                for value in &epps {
                    ui.selectable_value(&mut cc.tune.epp, value.clone(), value);
                }
            });

        ui.add_space(8.0);
        ui::mono_label(
            ui,
            "EPP IST NICHT AUF JEDER CPU VERFÜGBAR",
            theme::FAINT,
            9.0,
        );

        ui.add_space(12.0);
        ui::hairline(ui);
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!cc.action_pending(), |ui| {
                if ui::primary(ui, "Übernehmen").clicked() {
                    let args = vec![
                        "cpu".to_string(),
                        cc.tune.governor.clone(),
                        cc.tune.epp.clone(),
                    ];
                    cc.apply_helper(args, "CPU-Tuning");
                }
            });
            if cc.action_pending() {
                ui.spinner();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui::badge(
                    ui,
                    &format!("{} / {}", snap.tuning.governor, snap.tuning.epp),
                    theme::ACCENT,
                );
            });
        });
    });
}

fn memory_card(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    ui::panel_titled(ui, "Kernel und Speicher", "sysctl", |ui| {
        ui::hint(
            ui,
            "Werte landen in /etc/sysctl.d/99-playforge-zz-tuning.conf und gelten sofort.",
        );
        ui.add_space(12.0);
        ui.spacing_mut().slider_width = ui.available_width().min(320.0);

        ui::kicker(ui, "vm.swappiness");
        ui.add_space(2.0);
        ui.add(
            egui::Slider::new(&mut cc.tune.swappiness, 0..=200)
                .min_decimals(0)
                .show_value(true),
        );

        ui.add_space(7.0);
        ui::kicker(ui, "vm.vfs_cache_pressure");
        ui.add_space(2.0);
        ui.add(
            egui::Slider::new(&mut cc.tune.cache_pressure, 0..=1000)
                .min_decimals(0)
                .show_value(true),
        );

        ui.add_space(7.0);
        ui::kicker(ui, "fs.inotify.max_user_watches");
        ui.add_space(2.0);
        ui.add(
            egui::Slider::new(&mut cc.tune.inotify, 8192..=1_048_576)
                .logarithmic(true)
                .min_decimals(0)
                .show_value(true),
        );

        ui.add_space(10.0);
        ui::kicker(ui, "vm.max_map_count");
        ui.add_space(4.0);
        egui::ComboBox::from_id_salt("tune-max-map")
            .width(ui.available_width())
            .selected_text(max_map_label(cc.tune.max_map))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut cc.tune.max_map, MAX_MAP_STANDARD, "Standard (65 530)");
                ui.selectable_value(
                    &mut cc.tune.max_map,
                    MAX_MAP_PROTON,
                    "Proton / Wine (2 147 483 642)",
                );
            });

        ui.add_space(12.0);
        ui::hairline(ui);
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!cc.action_pending(), |ui| {
                if ui::primary(ui, "Sysctl übernehmen").clicked() {
                    let args = vec![
                        "sysctl".to_string(),
                        format!("vm.swappiness={}", cc.tune.swappiness),
                        format!("vm.vfs_cache_pressure={}", cc.tune.cache_pressure),
                        format!("fs.inotify.max_user_watches={}", cc.tune.inotify),
                        format!("vm.max_map_count={}", cc.tune.max_map),
                    ];
                    cc.apply_helper(args, "Kernel-Tuning");
                }
            });
            if cc.action_pending() {
                ui.spinner();
            }
        });
    });
}

fn io_card(cc: &mut ControlCenter, ui: &mut egui::Ui, snap: &Snapshot) {
    ui::panel_titled(ui, "Datenträger", "io-scheduler", |ui| {
        ui::hint(
            ui,
            "Der Scheduler wird per udev-Regel dauerhaft gesetzt und sofort angewendet.",
        );
        ui.add_space(11.0);

        if snap.tuning.io_devices.is_empty() {
            ui::mono_label(ui, "KEINE DATENTRÄGER GEFUNDEN", theme::FAINT, 9.5);
            return;
        }

        for device in &snap.tuning.io_devices {
            ui.horizontal(|ui| {
                ui::mono_label(ui, &device.name, theme::TEXT, 10.5);
                ui::badge(ui, &device.scheduler, theme::ACCENT);
            });
            ui::mono_label(ui, &device.supported.join(" · "), theme::FAINT, 9.0);
            ui.add_space(5.0);
        }

        let mut options: Vec<String> = Vec::new();
        for device in &snap.tuning.io_devices {
            for scheduler in &device.supported {
                if !options.contains(scheduler) {
                    options.push(scheduler.clone());
                }
            }
        }
        options.sort();

        ui.add_space(7.0);
        ui::hairline(ui);
        ui.add_space(11.0);
        ui::kicker(ui, "Scheduler");
        ui.add_space(5.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 6.0);
            for scheduler in &options {
                if ui::choice(ui, cc.tune.io_scheduler == *scheduler, scheduler).clicked() {
                    cc.tune.io_scheduler = scheduler.clone();
                }
            }
        });
        ui.add_space(6.0);
        ui::mono_label(
            ui,
            "NONE/MQ-DEADLINE FÜR NVME · BFQ FÜR SATA",
            theme::FAINT,
            9.0,
        );

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!cc.action_pending(), |ui| {
                if ui::primary(ui, "Scheduler setzen").clicked() {
                    let args = vec!["io-scheduler".to_string(), cc.tune.io_scheduler.clone()];
                    cc.apply_helper(args, "I/O-Scheduler");
                }
            });
            if cc.action_pending() {
                ui.spinner();
            }
        });
    });
}

fn graphics_card(cc: &mut ControlCenter, ui: &mut egui::Ui) {
    ui::panel_titled(ui, "Grafik-Umgebung", "/etc/environment.d", |ui| {
        ui::hint(
            ui,
            "Mesa- und Vulkan-Variablen gelten für neue Sitzungen und Spiele.",
        );
        ui.add_space(12.0);

        ui.checkbox(
            &mut cc.tune.mesa_glthread,
            RichText::new("MESA_GLTHREAD")
                .font(theme::mono(10.5))
                .color(theme::TEXT),
        );
        ui::mono_label(ui, "MESA-AUFRUFE IN EIGENEN THREADS", theme::FAINT, 9.0);

        ui.add_space(8.0);
        ui.checkbox(
            &mut cc.tune.radv_gpl,
            RichText::new("RADV_PERFTEST=gpl")
                .font(theme::mono(10.5))
                .color(theme::TEXT),
        );
        ui::mono_label(ui, "AMD: GRAFIK-PIPELINE STATT COMPUTE", theme::FAINT, 9.0);

        ui.add_space(10.0);
        ui::kicker(ui, "AMD_VULKAN_ICD");
        ui.add_space(4.0);
        egui::ComboBox::from_id_salt("tune-amd-icd")
            .width(ui.available_width())
            .selected_text(cc.tune.amd_icd.clone())
            .show_ui(ui, |ui| {
                for value in ["RADV", "AMDVLK"] {
                    ui.selectable_value(&mut cc.tune.amd_icd, value.to_string(), value);
                }
            });

        ui.add_space(12.0);
        ui::hairline(ui);
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!cc.action_pending(), |ui| {
                if ui::primary(ui, "Umgebung übernehmen").clicked() {
                    let args = vec![
                        "graphics-env".to_string(),
                        format!("MESA_GLTHREAD={}", cc.tune.mesa_glthread),
                        format!("AMD_VULKAN_ICD={}", cc.tune.amd_icd),
                        format!(
                            "RADV_PERFTEST={}",
                            if cc.tune.radv_gpl { "gpl" } else { "" }
                        ),
                    ];
                    cc.apply_helper(args, "Grafik-Umgebung");
                }
            });
            if cc.action_pending() {
                ui.spinner();
            }
        });
    });
}

fn max_map_label(value: u32) -> String {
    match value {
        MAX_MAP_STANDARD => "Standard (65 530)".to_string(),
        MAX_MAP_PROTON => "Proton / Wine (2 147 483 642)".to_string(),
        other => format!("eigener Wert: {other}"),
    }
}
