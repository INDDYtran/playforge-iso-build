//! Sammelt den Systemzustand – ausschließlich lesend.
//!
//! Alle Zugriffe sind best-effort: Fehlt eine Datei oder ein Befehl (z. B. beim
//! Entwickeln unter Windows), bleibt das Feld leer, statt die App zu stören.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, SystemTime};

use crate::model::*;

/// Bekannte Dienste, die das Control Center anzeigen und schalten darf.
pub const UNITS: &[(&str, &str, &str)] = &[
    (
        "playforge-profile.service",
        "Playforge-Leistungsprofil",
        "Wendet das gespeicherte Profil beim Booten an",
    ),
    (
        "NetworkManager.service",
        "Netzwerk",
        "WLAN, Ethernet und VPN",
    ),
    (
        "bluetooth.service",
        "Bluetooth",
        "BlueZ für Controller und Headsets",
    ),
    (
        "power-profiles-daemon.service",
        "Power-Profile",
        "Leistungsmodi des Systems",
    ),
    (
        "lactd.service",
        "LACT (GPU-Steuerung)",
        "Lüfterkurven und Leistungsgrenzen",
    ),
    (
        "sddm.service",
        "Anmeldebildschirm",
        "Grafischer Login (SDDM)",
    ),
    ("fstrim.timer", "SSD-Trim", "Wöchentliches TRIM für SSDs"),
    (
        "reflector.timer",
        "Spiegel-Aktualisierung",
        "Aktualisiert die Pacman-Mirrors",
    ),
    (
        "systemd-timesyncd.service",
        "Zeitsynchronisation",
        "NTP über systemd-timesyncd",
    ),
];

pub fn collect() -> Snapshot {
    let os = collect_os();
    let cpu = collect_cpu();
    let mem = collect_mem();
    let gpus = collect_gpus();
    let services = collect_services();
    let tuning = collect_tuning();
    let settings = collect_settings();

    Snapshot {
        os,
        cpu,
        mem,
        gpus,
        vulkan: collect_vulkan(),
        tuning,
        gaming: GamingInfo {
            gamemode: have("gamemoderun"),
            mangohud: have("mangohud"),
            gamescope: have("gamescope"),
            steam: have("steam"),
            lutris: have("lutris"),
            goverlay: have("goverlay"),
            wine: have("wine"),
            lact: have("lact") || have("lactd"),
        },
        packages: collect_packages(),
        services,
        settings,
        system_update: collect_system_update(),
        nvidia_smi: run(
            "nvidia-smi",
            &[
                "--query-gpu=name,driver_version,memory.total,temperature.gpu",
                "--format=csv,noheader",
            ],
        ),
    }
}

// ---------------------------------------------------------------------------
// Prozess- und Dateizugriff
// ---------------------------------------------------------------------------

/// Führt ein Programm aus und liefert stdout, sofern es nicht leer ist.
/// Der Exit-Status wird bewusst ignoriert: `pacman -Qu` endet z. B. mit 1,
/// wenn keine Aktualisierungen anstehen, liefert aber trotzdem eine Liste.
pub fn run(program: &str, args: &[&str]) -> Option<String> {
    let output = command(program, args).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Wie `run`, gibt aber den vollständigen Output samt Status zurück.
pub fn run_full(program: &str, args: &[&str]) -> Option<Output> {
    command(program, args).output().ok()
}

fn command(program: &str, args: &[&str]) -> Command {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .env("SYSTEMD_COLORS", "0")
        .env("NO_COLOR", "1")
        .env("LC_ALL", "C");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Sucht ein Programm im PATH (schneller als ein Testaufruf).
pub fn have(program: &str) -> bool {
    if program.contains('/') || program.contains('\\') {
        return Path::new(program).is_file();
    }
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    let mut exts: Vec<String> = vec![String::new()];
    #[cfg(windows)]
    {
        exts = std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".to_string())
            .split(';')
            .filter(|ext| !ext.is_empty())
            .map(|ext| ext.to_lowercase())
            .collect();
    }
    std::env::split_paths(&paths).any(|dir| {
        exts.iter()
            .any(|ext| dir.join(format!("{program}{ext}")).is_file())
    })
}

fn read_trim(path: impl AsRef<Path>) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let text = text.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn read_u64(path: impl AsRef<Path>) -> Option<u64> {
    read_trim(path)?.parse().ok()
}

fn read_f64(path: impl AsRef<Path>) -> Option<f64> {
    read_trim(path)?.parse().ok()
}

fn os_release_value(key: &str) -> Option<String> {
    let text = fs::read_to_string("/etc/os-release").ok()?;
    text.lines().find_map(|line| {
        let (k, v) = line.split_once('=')?;
        (k.trim() == key).then(|| v.trim().trim_matches('"').to_string())
    })
}

fn sysctl(path: &str) -> Option<String> {
    read_trim(format!("/proc/sys/{path}"))
}

// ---------------------------------------------------------------------------
// Einzelne Bereiche
// ---------------------------------------------------------------------------

fn collect_os() -> OsInfo {
    let kernel = read_trim("/proc/sys/kernel/osrelease")
        .or_else(|| run("uname", &["-r"]))
        .unwrap_or_default();

    let session = std::env::var("XDG_SESSION_TYPE")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::env::var("WAYLAND_DISPLAY")
                .ok()
                .map(|_| "wayland".to_string())
        })
        .or_else(|| std::env::var("DISPLAY").ok().map(|_| "x11".to_string()))
        .unwrap_or_else(|| "—".to_string());

    let boot_mode = if Path::new("/sys/firmware/efi").exists() {
        "UEFI".to_string()
    } else if cfg!(target_os = "linux") {
        "BIOS / Legacy".to_string()
    } else {
        "—".to_string()
    };

    let secure_boot =
        fs::read("/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c")
            .ok()
            .and_then(|bytes| bytes.get(4).map(|value| *value != 0));

    let uptime_secs = read_trim("/proc/uptime")
        .and_then(|line| line.split_whitespace().next().and_then(|v| v.parse().ok()));

    OsInfo {
        distro: os_release_value("PRETTY_NAME")
            .or_else(|| os_release_value("NAME"))
            .unwrap_or_else(|| format!("Unbekannt ({})", std::env::consts::OS)),
        kernel,
        arch: std::env::consts::ARCH.to_string(),
        hostname: read_trim("/etc/hostname")
            .or_else(|| std::env::var("HOSTNAME").ok())
            .unwrap_or_else(|| "—".to_string()),
        uptime_secs,
        session,
        boot_mode,
        secure_boot,
        systemd: run("systemctl", &["--version"])
            .and_then(|out| out.lines().next().map(|line| line.trim().to_string()))
            .unwrap_or_else(|| "—".to_string()),
    }
}

fn collect_cpu() -> CpuInfo {
    let mut model = String::new();
    let mut cores = None;
    let mut threads = 0u32;

    if let Ok(text) = fs::read_to_string("/proc/cpuinfo") {
        for line in text.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            match key.trim() {
                "model name" if model.is_empty() => model = value.trim().to_string(),
                "processor" => threads += 1,
                "cpu cores" if cores.is_none() => {
                    cores = value.trim().parse::<u32>().ok();
                }
                _ => {}
            }
        }
    }

    if model.is_empty() {
        model = read_trim("/proc/device-tree/model").unwrap_or_else(|| "—".to_string());
    }

    let governor = read_trim("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .unwrap_or_else(|| "—".to_string());
    let epp = read_trim("/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_preference")
        .unwrap_or_else(|| "—".to_string());
    let freq_mhz = read_u64("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
        .map(|khz| khz as f64 / 1000.0);

    let load = read_trim("/proc/loadavg").and_then(|line| {
        let mut parts = line.split_whitespace();
        let one = parts.next()?.parse().ok()?;
        let five = parts.next()?.parse().ok()?;
        let fifteen = parts.next()?.parse().ok()?;
        Some([one, five, fifteen])
    });

    CpuInfo {
        model,
        cores: cores.or(Some(threads).filter(|t| *t > 0)),
        threads: Some(threads).filter(|t| *t > 0),
        governor,
        epp,
        freq_mhz,
        temp_c: cpu_temp(),
        load,
    }
}

fn cpu_temp() -> Option<f64> {
    let mut best: Option<(u8, f64)> = None;
    for entry in fs::read_dir("/sys/class/hwmon").ok()?.flatten() {
        let name = read_trim(entry.path().join("name")).unwrap_or_default();
        let priority = match name.as_str() {
            "coretemp" | "k10temp" | "zenpower" => 0,
            "cpu_thermal" => 1,
            "acpitz" => 2,
            _ => continue,
        };
        let Some(raw) = read_f64(entry.path().join("temp1_input")) else {
            continue;
        };
        let celsius = raw / 1000.0;
        if best.map(|(p, _)| priority < p).unwrap_or(true) {
            best = Some((priority, celsius));
        }
    }
    best.map(|(_, celsius)| celsius)
}

fn collect_mem() -> MemInfo {
    let mut values: HashMap<String, u64> = HashMap::new();
    if let Ok(text) = fs::read_to_string("/proc/meminfo") {
        for line in text.lines() {
            let Some((key, rest)) = line.split_once(':') else {
                continue;
            };
            if let Some(value) = rest.split_whitespace().next().and_then(|v| v.parse().ok()) {
                values.insert(key.trim().to_string(), value);
            }
        }
    }
    let get = |key: &str| values.get(key).copied().unwrap_or(0);

    MemInfo {
        total_kb: get("MemTotal"),
        available_kb: get("MemAvailable"),
        swap_total_kb: get("SwapTotal"),
        swap_free_kb: get("SwapFree"),
        zram: run(
            "zramctl",
            &["--noheadings", "--output", "NAME,ALGORITHM,DISKSIZE"],
        )
        .map(|out| out.split_whitespace().collect::<Vec<_>>().join(" · "))
        .unwrap_or_default(),
    }
}

fn collect_gpus() -> Vec<GpuInfo> {
    let Some(text) = run("lspci", &["-nnk"]) else {
        return Vec::new();
    };

    let mut gpus: Vec<GpuInfo> = Vec::new();
    let mut current: Option<GpuInfo> = None;

    for line in text.lines() {
        let starts_device = !line.starts_with(' ') && !line.starts_with('\t');
        if starts_device {
            if let Some(gpu) = current.take() {
                gpus.push(gpu);
            }
            let is_gpu = line.contains("VGA compatible controller")
                || line.contains("3D controller")
                || line.contains("Display controller");
            if is_gpu {
                let name = line.split_once(": ").map(|(_, rest)| rest).unwrap_or(line);
                current = Some(GpuInfo {
                    name: clean_gpu_name(name),
                    driver: None,
                    vendor: None,
                });
            }
        } else if let Some(gpu) = current.as_mut() {
            if let Some((_, value)) = line.split_once("Kernel driver in use:") {
                gpu.driver = Some(value.trim().to_string());
            }
        }
    }
    if let Some(gpu) = current {
        gpus.push(gpu);
    }

    for gpu in &mut gpus {
        gpu.vendor = Some(detect_vendor(&gpu.name, gpu.driver.as_deref()));
    }
    gpus
}

fn clean_gpu_name(raw: &str) -> String {
    let mut name = raw.to_string();
    if let Some(pos) = name.rfind(" (rev") {
        name.truncate(pos);
    }
    if let Some(pos) = name.rfind(" [") {
        if name[pos..].ends_with(']') {
            name.truncate(pos);
        }
    }
    name.trim().to_string()
}

fn detect_vendor(name: &str, driver: Option<&str>) -> GpuVendor {
    let haystack = format!(
        "{} {}",
        name.to_lowercase(),
        driver.unwrap_or_default().to_lowercase()
    );
    if haystack.contains("nvidia") {
        GpuVendor::Nvidia
    } else if haystack.contains("amd") || haystack.contains("radeon") || haystack.contains("ati ") {
        GpuVendor::Amd
    } else if haystack.contains("intel") || haystack.contains("i915") || haystack.contains(" xe ") {
        GpuVendor::Intel
    } else if haystack.contains("virtio") || haystack.contains("vmware") || haystack.contains("qxl")
    {
        GpuVendor::Virtual
    } else {
        GpuVendor::Other
    }
}

fn collect_vulkan() -> VulkanInfo {
    let mut icds = Vec::new();
    if let Ok(entries) = fs::read_dir("/usr/share/vulkan/icd.d") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                    icds.push(stem.to_string());
                }
            }
        }
    }
    icds.sort();

    VulkanInfo {
        icds,
        lib32: Path::new("/usr/lib32/libvulkan.so.1").exists(),
        summary: run("vulkaninfo", &["--summary"]),
        vainfo: have("vainfo"),
    }
}

fn collect_tuning() -> TuningInfo {
    let env = environment_values();
    TuningInfo {
        profile_saved: read_trim("/etc/playforge/profile")
            .unwrap_or_else(|| "balanced".to_string()),
        governor: read_trim("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
            .unwrap_or_else(|| "—".to_string()),
        epp: read_trim("/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_preference")
            .unwrap_or_else(|| "—".to_string()),
        platform_profile: read_trim("/sys/firmware/acpi/platform_profile")
            .unwrap_or_else(|| "—".to_string()),
        power_profile: run("powerprofilesctl", &["get"]),
        max_map_count: read_u64("/proc/sys/vm/max_map_count"),
        swappiness: read_u64("/proc/sys/vm/swappiness"),
        vfs_cache_pressure: read_u64("/proc/sys/vm/vfs_cache_pressure"),
        tcp_congestion: sysctl("net/ipv4/tcp_congestion_control"),
        inotify_watches: read_u64("/proc/sys/fs/inotify/max_user_watches"),
        governors: sysfs_list("/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors"),
        epps: sysfs_list(
            "/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_available_preferences",
        ),
        io_devices: collect_io_devices(),
        graphics_env: GraphicsEnv {
            mesa_glthread: env
                .get("MESA_GLTHREAD")
                .map(|value| value.eq_ignore_ascii_case("true")),
            amd_icd: env
                .get("AMD_VULKAN_ICD")
                .filter(|value| !value.is_empty())
                .cloned(),
            radv_gpl: env
                .get("RADV_PERFTEST")
                .map(|value| value.split(',').any(|part| part.trim() == "gpl"))
                .unwrap_or(false),
        },
        autoupdate: collect_autoupdate(),
    }
}

/// Liste aus einer Sysfs-Datei; der aktive Eintrag steht dort in `[...]`.
fn sysfs_list(path: &str) -> Vec<String> {
    read_trim(path)
        .map(|text| {
            text.split_whitespace()
                .map(|value| value.trim_matches(|c| c == '[' || c == ']').to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn collect_io_devices() -> Vec<IoDevice> {
    let mut devices = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/block") else {
        return devices;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if ["loop", "ram", "zram", "dm-", "sr"]
            .iter()
            .any(|prefix| name.starts_with(prefix))
        {
            continue;
        }
        let Some(text) = read_trim(entry.path().join("queue/scheduler")) else {
            continue;
        };
        let supported: Vec<String> = text
            .split_whitespace()
            .map(|value| value.trim_matches(|c| c == '[' || c == ']').to_string())
            .collect();
        let scheduler = text
            .split_whitespace()
            .find(|value| value.starts_with('['))
            .map(|value| value.trim_matches(|c| c == '[' || c == ']').to_string())
            .unwrap_or_default();
        devices.push(IoDevice {
            name,
            scheduler,
            supported,
        });
    }
    devices.sort_by(|a, b| a.name.cmp(&b.name));
    devices
}

fn environment_values() -> HashMap<String, String> {
    let mut values = HashMap::new();
    let Ok(text) = fs::read_to_string("/etc/environment.d/90-playforge.conf") else {
        return values;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            values.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    values
}

fn collect_autoupdate() -> AutoUpdate {
    let mode = fs::read_to_string("/etc/playforge/updates.conf")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.trim()
                    .strip_prefix("MODE=")
                    .map(|value| value.trim().to_string())
            })
        })
        .filter(|mode| matches!(mode.as_str(), "daily" | "weekly"))
        .unwrap_or_else(|| "off".to_string());

    let timer_active = run("systemctl", &["is-active", "playforge-autoupdate.timer"])
        .map(|out| out.trim() == "active")
        .unwrap_or(false);

    let last_update = read_u64("/var/lib/playforge/last-update")
        .and_then(|secs| SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(secs)));

    AutoUpdate {
        mode,
        timer_active,
        last_update,
    }
}

/// Liest das Ergebnis der täglichen Prüfung durch playforge-update-check.timer.
fn collect_system_update() -> SystemUpdate {
    let text = fs::read_to_string("/var/lib/playforge/update-status").unwrap_or_default();
    let mut status = SystemUpdate::default();

    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "CHECKED" => {
                status.checked = value
                    .parse::<u64>()
                    .ok()
                    .and_then(|secs| SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(secs)));
            }
            "INSTALLED" if !value.is_empty() => status.installed = Some(value.to_string()),
            "AVAILABLE" if !value.is_empty() => status.available = Some(value.to_string()),
            "UPDATE" => status.update_available = value == "1",
            "STATE" => status.failed = value == "error",
            _ => {}
        }
    }

    if status.installed.is_none() {
        status.installed = read_trim("/etc/playforge/version");
    }

    status.timer_active = run("systemctl", &["is-active", "playforge-update-check.timer"])
        .map(|out| out.trim() == "active")
        .unwrap_or(false);

    status
}

/// Prüft verfügbare Updates (ohne etwas zu installieren).
pub fn collect_updates() -> UpdateReport {
    let mut report = UpdateReport {
        checked: Some(SystemTime::now()),
        ..Default::default()
    };

    if !cfg!(target_os = "linux") {
        report.error = Some(
            "Updates lassen sich nur unter Playforge Linux prüfen – ohne pacman und flatpak \
             bleibt die Seite leer."
                .to_string(),
        );
        return report;
    }

    report.aur_helper = ["paru", "yay"]
        .iter()
        .find(|tool| have(tool))
        .map(|tool| tool.to_string());

    // `checkupdates` prüft gegen eine temporäre Datenbank, ohne `pacman -Sy`
    // und damit ohne Risiko eines Teil-Upgrades.
    let pacman = if have("checkupdates") {
        run("checkupdates", &[])
    } else {
        run("pacman", &["-Qu"])
    };
    report.pacman = split_lines(pacman);

    if let Some(helper) = &report.aur_helper {
        report.aur = split_lines(run(helper, &["-Qua"]));
    }

    report.flatpak_available = have("flatpak");
    if report.flatpak_available {
        report.flatpak = run(
            "flatpak",
            &["remote-ls", "--updates", "--columns=application"],
        )
        .map(|out| out.lines().filter(|line| !line.trim().is_empty()).count())
        .unwrap_or(0);
    }

    if !have("pacman") && report.aur.is_empty() && !report.flatpak_available {
        report.error = Some(
            "Weder pacman noch flatpak gefunden – es gibt auf diesem System nichts zu \
             aktualisieren."
                .to_string(),
        );
    }

    report
}

fn split_lines(text: Option<String>) -> Vec<String> {
    text.map(|out| {
        out.lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect()
    })
    .unwrap_or_default()
}

fn collect_packages() -> PackageInfo {
    let installed = run("pacman", &["-Q"]).map(|out| out.lines().count());
    let updates = run("pacman", &["-Qu"])
        .map(|out| out.lines().count())
        .or(Some(0));

    let last_sync = fs::read_dir("/var/lib/pacman/sync")
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.metadata().ok()?.modified().ok())
        .max();

    let multilib = fs::read_to_string("/etc/pacman.conf")
        .map(|text| text.lines().any(|line| line.trim() == "[multilib]"))
        .unwrap_or(false);

    PackageInfo {
        installed,
        updates,
        last_sync,
        multilib,
        aur_helper: ["paru", "yay"]
            .iter()
            .find(|tool| have(tool))
            .map(|s| s.to_string()),
    }
}

fn collect_services() -> Vec<ServiceInfo> {
    if !have("systemctl") {
        return Vec::new();
    }

    // `list-unit-files` liefert den Aktivierungszustand, `list-units` den
    // Laufzeitzustand. Beide werden einmalig für alle Units abgefragt.
    let mut enabled: HashMap<String, String> = HashMap::new();
    if let Some(text) = run(
        "systemctl",
        &[
            "list-unit-files",
            "--no-legend",
            "--no-pager",
            "--plain",
            "--type=service",
            "--type=timer",
        ],
    ) {
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            let (Some(unit), Some(state)) = (parts.next(), parts.next()) else {
                continue;
            };
            enabled.insert(unit.to_string(), state.to_string());
        }
    }

    let mut runtime: HashMap<String, (ServiceState, String)> = HashMap::new();
    if let Some(text) = run(
        "systemctl",
        &[
            "list-units",
            "--all",
            "--no-legend",
            "--no-pager",
            "--plain",
            "--type=service",
            "--type=timer",
        ],
    ) {
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            let (Some(unit), Some(_load), Some(active)) =
                (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let description = parts.collect::<Vec<_>>().join(" ");
            runtime.insert(
                unit.to_string(),
                (ServiceState::from_systemd(active), description),
            );
        }
    }

    if runtime.is_empty() && enabled.is_empty() {
        return Vec::new();
    }

    UNITS
        .iter()
        .map(|(unit, label, description)| {
            let runtime_entry = runtime.get(*unit);
            let installed = runtime_entry.is_some() || enabled.contains_key(*unit);
            ServiceInfo {
                unit: unit.to_string(),
                label: label.to_string(),
                description: runtime_entry
                    .map(|(_, desc)| desc.clone())
                    .filter(|desc| !desc.is_empty())
                    .unwrap_or_else(|| description.to_string()),
                state: runtime_entry
                    .map(|(state, _)| *state)
                    .unwrap_or(ServiceState::Unknown),
                enabled: enabled.get(*unit).cloned(),
                installed,
            }
        })
        .collect()
}

fn collect_settings() -> SettingsInfo {
    let shader_cache = fs::read_to_string("/etc/environment.d/90-playforge.conf")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.trim()
                    .strip_prefix("MESA_SHADER_CACHE_MAX_SIZE=")
                    .map(|value| value.trim().to_string())
            })
        })
        .unwrap_or_default();

    let gamemode_text = fs::read_to_string("/etc/gamemode.ini").unwrap_or_default();
    let gamemode = GamemodeSettings {
        desired_gov: ini_value(&gamemode_text, "general", "desiredgov")
            .unwrap_or_else(|| "performance".to_string()),
        renice: ini_value(&gamemode_text, "general", "renice")
            .and_then(|value| value.parse().ok())
            .unwrap_or(10),
        softrealtime: ini_value(&gamemode_text, "general", "softrealtime")
            .unwrap_or_else(|| "auto".to_string()),
        inhibit_screensaver: ini_value(&gamemode_text, "general", "inhibit_screensaver")
            .map(|value| value.trim() != "0")
            .unwrap_or(true),
        reaper_freq: ini_value(&gamemode_text, "general", "reaper_freq")
            .and_then(|value| value.parse().ok())
            .unwrap_or(5),
    };

    let mangohud_path = std::env::var("HOME")
        .ok()
        .map(|home| format!("{home}/.config/MangoHud/MangoHud.conf"))
        .filter(|path| Path::new(path).exists())
        .unwrap_or_else(|| "/etc/skel/.config/MangoHud/MangoHud.conf".to_string());
    let mangohud_conf = fs::read_to_string(&mangohud_path).unwrap_or_default();
    let mangohud_preset = mangohud_conf
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("# Playforge-Preset:")
                .map(|value| value.trim().to_string())
        })
        .unwrap_or_else(|| {
            if mangohud_conf.trim().is_empty() {
                "missing".to_string()
            } else {
                "custom".to_string()
            }
        });

    let color_scheme = fs::read_to_string("/etc/xdg/kdeglobals")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.trim()
                    .strip_prefix("ColorScheme=")
                    .map(|value| value.trim().to_string())
            })
        })
        .unwrap_or_default();

    SettingsInfo {
        shader_cache,
        gamemode,
        mangohud_preset,
        mangohud_conf: mangohud_conf.chars().take(4000).collect(),
        color_scheme,
    }
}

fn ini_value(text: &str, section: &str, key: &str) -> Option<String> {
    let mut in_section = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed[1..trimmed.len() - 1].trim() == section;
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Diagnose und Protokolle
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckKind {
    Ok,
    Warn,
    Fail,
    Info,
}

#[derive(Clone, Debug)]
pub struct DoctorItem {
    pub kind: CheckKind,
    pub text: String,
}

#[derive(Clone, Debug, Default)]
pub struct DoctorSection {
    pub title: String,
    pub items: Vec<DoctorItem>,
}

#[derive(Clone, Debug, Default)]
pub struct DoctorReport {
    pub sections: Vec<DoctorSection>,
    pub ok: usize,
    pub warn: usize,
    pub fail: usize,
    pub raw: String,
    pub available: bool,
    pub error: Option<String>,
}

pub fn run_doctor() -> DoctorReport {
    let mut report = DoctorReport::default();

    if !have("playforge-doctor") && !Path::new("/usr/local/bin/playforge-doctor").exists() {
        report.error = Some(
            "playforge-doctor ist nicht installiert. Die Diagnose läuft nur unter Playforge Linux."
                .to_string(),
        );
        return report;
    }

    let Some(output) = run_full("playforge-doctor", &[]) else {
        report.error = Some("Die Diagnose konnte nicht gestartet werden.".to_string());
        return report;
    };

    let raw = String::from_utf8_lossy(&output.stdout).to_string();
    report.available = true;
    report.raw = raw.clone();

    let mut sections: Vec<DoctorSection> = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() {
            continue;
        }
        let no_indent = !trimmed.starts_with(' ');
        let content = trimmed.trim();

        let item = if let Some(rest) = content.strip_prefix("[OK]") {
            Some((CheckKind::Ok, rest.trim().to_string()))
        } else if let Some(rest) = content.strip_prefix("[WARN]") {
            Some((CheckKind::Warn, rest.trim().to_string()))
        } else if let Some(rest) = content.strip_prefix("[FAIL]") {
            Some((CheckKind::Fail, rest.trim().to_string()))
        } else {
            None
        };

        if let Some((kind, text)) = item {
            match kind {
                CheckKind::Ok => report.ok += 1,
                CheckKind::Warn => report.warn += 1,
                CheckKind::Fail => report.fail += 1,
                CheckKind::Info => {}
            }
            if sections.is_empty() {
                sections.push(DoctorSection::default());
            }
            sections
                .last_mut()
                .expect("Abschnitt vorhanden")
                .items
                .push(DoctorItem { kind, text });
        } else if no_indent {
            sections.push(DoctorSection {
                title: content.to_string(),
                items: Vec::new(),
            });
        } else if !sections.is_empty() {
            sections
                .last_mut()
                .expect("Abschnitt vorhanden")
                .items
                .push(DoctorItem {
                    kind: CheckKind::Info,
                    text: content.to_string(),
                });
        }
    }

    sections.retain(|section| !section.items.is_empty());
    report.sections = sections;
    report
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogMode {
    Playforge,
    System,
    Warnings,
}

impl LogMode {
    pub const ALL: [LogMode; 3] = [LogMode::Playforge, LogMode::System, LogMode::Warnings];

    pub fn label(self) -> &'static str {
        match self {
            LogMode::Playforge => "Playforge-Dienste",
            LogMode::System => "Gesamtes Boot-Protokoll",
            LogMode::Warnings => "Nur Warnungen und Fehler",
        }
    }

    fn args(self, lines: usize) -> Vec<String> {
        let count = lines.to_string();
        let mut args = vec![
            "-b".to_string(),
            "--no-pager".to_string(),
            "-n".to_string(),
            count,
            "-o".to_string(),
            "short-iso".to_string(),
        ];
        match self {
            LogMode::Playforge => {
                args.push("-u".to_string());
                args.push("playforge-profile.service".to_string());
                args.push("-u".to_string());
                args.push("playforge-live-setup.service".to_string());
            }
            LogMode::System => {}
            LogMode::Warnings => {
                args.push("-p".to_string());
                args.push("warning".to_string());
            }
        }
        args
    }
}

pub fn read_logs(mode: LogMode, lines: usize) -> Result<String, String> {
    if !have("journalctl") {
        return Err(
            "journalctl ist nicht verfügbar. Protokolle lassen sich nur unter Playforge Linux lesen."
                .to_string(),
        );
    }
    let args = mode.args(lines);
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = command("journalctl", &arg_refs)
        .output()
        .map_err(|err| format!("journalctl konnte nicht gestartet werden: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "journalctl meldet einen Fehler.".to_string()
        } else {
            stderr
        });
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Err("Keine Einträge im aktuellen Boot-Vorgang.".to_string())
    } else {
        Ok(text)
    }
}
