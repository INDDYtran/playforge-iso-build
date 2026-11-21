//! Datenmodell des Control Centers.

use eframe::egui::Color32;
use std::time::{Duration, SystemTime};

use crate::theme;

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub os: OsInfo,
    pub cpu: CpuInfo,
    pub mem: MemInfo,
    pub gpus: Vec<GpuInfo>,
    pub vulkan: VulkanInfo,
    pub tuning: TuningInfo,
    pub gaming: GamingInfo,
    pub packages: PackageInfo,
    pub services: Vec<ServiceInfo>,
    pub settings: SettingsInfo,
    pub nvidia_smi: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct OsInfo {
    pub distro: String,
    pub kernel: String,
    pub arch: String,
    pub hostname: String,
    pub uptime_secs: Option<f64>,
    pub session: String,
    pub boot_mode: String,
    pub secure_boot: Option<bool>,
    pub systemd: String,
}

#[derive(Clone, Debug, Default)]
pub struct CpuInfo {
    pub model: String,
    pub cores: Option<u32>,
    pub threads: Option<u32>,
    pub governor: String,
    pub epp: String,
    pub freq_mhz: Option<f64>,
    pub temp_c: Option<f64>,
    pub load: Option<[f32; 3]>,
}

impl CpuInfo {
    pub fn freq_text(&self) -> String {
        match self.freq_mhz {
            Some(mhz) => format!("{mhz:.0} MHz"),
            None => "—".to_string(),
        }
    }

    pub fn temp_text(&self) -> String {
        match self.temp_c {
            Some(c) => format!("{c:.1} °C"),
            None => "—".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MemInfo {
    pub total_kb: u64,
    pub available_kb: u64,
    pub swap_total_kb: u64,
    pub swap_free_kb: u64,
    pub zram: String,
}

impl MemInfo {
    pub fn used_kb(&self) -> u64 {
        self.total_kb.saturating_sub(self.available_kb)
    }

    pub fn ram_fraction(&self) -> f32 {
        percent(self.used_kb(), self.total_kb)
    }

    pub fn swap_fraction(&self) -> f32 {
        percent(
            self.swap_total_kb.saturating_sub(self.swap_free_kb),
            self.swap_total_kb,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Virtual,
    Other,
}

impl GpuVendor {
    pub fn label(self) -> &'static str {
        match self {
            GpuVendor::Nvidia => "NVIDIA",
            GpuVendor::Amd => "AMD",
            GpuVendor::Intel => "Intel",
            GpuVendor::Virtual => "Virtuell",
            GpuVendor::Other => "GPU",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            GpuVendor::Nvidia => Color32::from_rgb(118, 185, 0),
            GpuVendor::Amd => Color32::from_rgb(237, 28, 36),
            GpuVendor::Intel => theme::BLUE,
            GpuVendor::Virtual => theme::MUTED,
            GpuVendor::Other => theme::AMBER,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct GpuInfo {
    pub name: String,
    pub driver: Option<String>,
    pub vendor: Option<GpuVendor>,
}

#[derive(Clone, Debug, Default)]
pub struct VulkanInfo {
    pub icds: Vec<String>,
    pub lib32: bool,
    pub summary: Option<String>,
    pub vainfo: bool,
}

#[derive(Clone, Debug, Default)]
pub struct TuningInfo {
    pub profile_saved: String,
    pub governor: String,
    pub epp: String,
    pub platform_profile: String,
    pub power_profile: Option<String>,
    pub max_map_count: Option<u64>,
    pub swappiness: Option<u64>,
    pub tcp_congestion: Option<String>,
    pub inotify_watches: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct GamingInfo {
    pub gamemode: bool,
    pub mangohud: bool,
    pub gamescope: bool,
    pub steam: bool,
    pub lutris: bool,
    pub goverlay: bool,
    pub wine: bool,
    pub lact: bool,
}

#[derive(Clone, Debug, Default)]
pub struct PackageInfo {
    pub installed: Option<usize>,
    pub updates: Option<usize>,
    pub last_sync: Option<SystemTime>,
    pub multilib: bool,
    pub aur_helper: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ServiceState {
    Active,
    Inactive,
    Failed,
    #[default]
    Unknown,
}

impl ServiceState {
    pub fn from_systemd(value: &str) -> Self {
        match value {
            "active" | "reloading" | "activating" => ServiceState::Active,
            "failed" => ServiceState::Failed,
            "inactive" | "deactivating" => ServiceState::Inactive,
            _ => ServiceState::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ServiceState::Active => "aktiv",
            ServiceState::Inactive => "inaktiv",
            ServiceState::Failed => "Fehler",
            ServiceState::Unknown => "unbekannt",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            ServiceState::Active => theme::GREEN,
            ServiceState::Inactive => theme::MUTED,
            ServiceState::Failed => theme::RED,
            ServiceState::Unknown => theme::AMBER,
        }
    }

    pub fn is_active(self) -> bool {
        self == ServiceState::Active
    }
}

#[derive(Clone, Debug, Default)]
pub struct ServiceInfo {
    pub unit: String,
    pub label: String,
    pub description: String,
    pub state: ServiceState,
    /// `enabled`, `disabled`, `static`, … wie von systemd gemeldet.
    pub enabled: Option<String>,
    pub installed: bool,
}

impl ServiceInfo {
    pub fn is_enabled(&self) -> bool {
        matches!(
            self.enabled.as_deref(),
            Some("enabled") | Some("enabled-runtime")
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct SettingsInfo {
    pub shader_cache: String,
    pub gamemode: GamemodeSettings,
    pub mangohud_preset: String,
    pub mangohud_conf: String,
    pub color_scheme: String,
}

#[derive(Clone, Debug)]
pub struct GamemodeSettings {
    pub desired_gov: String,
    pub renice: u32,
    pub softrealtime: String,
}

impl Default for GamemodeSettings {
    fn default() -> Self {
        Self {
            desired_gov: "performance".to_string(),
            renice: 10,
            softrealtime: "auto".to_string(),
        }
    }
}

/// Formatiert eine Kilobyte-Angabe als GiB bzw. MiB.
pub fn fmt_kb(kb: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0;
    let kb_f = kb as f64;
    if kb_f >= GIB {
        format!("{:.1} GiB", kb_f / GIB)
    } else if kb_f >= 1024.0 {
        format!("{:.0} MiB", kb_f / 1024.0)
    } else {
        format!("{kb} KiB")
    }
}

/// `1 Tag, 2 Std., 5 Min.` – kompakt genug für eine Statuszeile.
pub fn fmt_uptime(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    let days = total / 86_400;
    let hours = (total % 86_400) / 3_600;
    let minutes = (total % 3_600) / 60;
    if days > 0 {
        format!("{days} T {hours} Std {minutes} Min")
    } else if hours > 0 {
        format!("{hours} Std {minutes} Min")
    } else {
        format!("{minutes} Min")
    }
}

pub fn fmt_age(then: SystemTime) -> String {
    let diff = SystemTime::now()
        .duration_since(then)
        .unwrap_or(Duration::ZERO);
    let secs = diff.as_secs();
    match secs {
        0..=59 => "gerade eben".to_string(),
        60..=3_599 => format!("vor {} Min", secs / 60),
        3_600..=86_399 => format!("vor {} Std", secs / 3_600),
        _ => format!("vor {} Tagen", secs / 86_400),
    }
}

pub fn percent(part: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (part as f32 / total as f32).clamp(0.0, 1.0)
    }
}

/// Wählt eine Farbe für einen Füllstand (grün → gelb → rot).
pub fn load_color(fraction: f32) -> Color32 {
    if fraction >= 0.9 {
        theme::RED
    } else if fraction >= 0.7 {
        theme::AMBER
    } else {
        theme::GREEN
    }
}
