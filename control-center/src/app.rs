//! Fenster, Navigation und Zustand des Control Centers.

use std::path::Path;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align, Align2, CornerRadius, Layout, Margin, Rect, RichText, Sense, Stroke, Vec2,
};

use crate::actions::{self, ActionEvent, ActionResult};
use crate::icons::{self, Icon};
use crate::model::{GamemodeSettings, Snapshot, UpdateReport};
use crate::system::{self, DoctorReport, LogMode};
use crate::theme;
use crate::ui;

const REFRESH_INTERVAL: Duration = Duration::from_secs(8);
const LOG_LINES: usize = 400;
const RAIL_WIDTH: f32 = 236.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Overview,
    Performance,
    Gaming,
    Graphics,
    Tuning,
    Updates,
    Services,
    Diagnose,
    Logs,
}

impl Tab {
    pub const ALL: [Tab; 9] = [
        Tab::Overview,
        Tab::Performance,
        Tab::Gaming,
        Tab::Graphics,
        Tab::Tuning,
        Tab::Updates,
        Tab::Services,
        Tab::Diagnose,
        Tab::Logs,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Overview => "Übersicht",
            Tab::Performance => "Leistung",
            Tab::Gaming => "Gaming",
            Tab::Graphics => "Grafik",
            Tab::Tuning => "Feintuning",
            Tab::Updates => "Updates",
            Tab::Services => "Dienste",
            Tab::Diagnose => "Diagnose",
            Tab::Logs => "Protokolle",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Tab::Overview => "Systemstatus, Spielbereitschaft und Schnellaktionen",
            Tab::Performance => "Leistungsprofile, Tuning und Shader-Cache",
            Tab::Gaming => "GameMode, MangoHud und Startoptionen",
            Tab::Graphics => "Grafikprozessoren, Treiber und Vulkan",
            Tab::Tuning => "CPU, Kernel, Datenträger und Grafik-Umgebung",
            Tab::Updates => "System, Flatpak und automatische Aktualisierung",
            Tab::Services => "Systemdienste steuern und Autostart setzen",
            Tab::Diagnose => "Systemprüfung mit playforge-doctor",
            Tab::Logs => "Journal des laufenden Boot-Vorgangs",
        }
    }

    /// Laufende Nummer in der Navigationsleiste.
    pub fn index(self) -> &'static str {
        match self {
            Tab::Overview => "01",
            Tab::Performance => "02",
            Tab::Gaming => "03",
            Tab::Graphics => "04",
            Tab::Tuning => "05",
            Tab::Updates => "06",
            Tab::Services => "07",
            Tab::Diagnose => "08",
            Tab::Logs => "09",
        }
    }

    /// Technische Kennung für die Kopfzeile.
    pub fn slug(self) -> &'static str {
        match self {
            Tab::Overview => "OVERVIEW",
            Tab::Performance => "PERFORMANCE",
            Tab::Gaming => "GAMING",
            Tab::Graphics => "GRAPHICS",
            Tab::Tuning => "TUNING",
            Tab::Updates => "UPDATES",
            Tab::Services => "SERVICES",
            Tab::Diagnose => "DIAGNOSE",
            Tab::Logs => "LOGS",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Tab::Overview => Icon::Dashboard,
            Tab::Performance => Icon::Gauge,
            Tab::Gaming => Icon::Gamepad,
            Tab::Graphics => Icon::Monitor,
            Tab::Tuning => Icon::Bolt,
            Tab::Updates => Icon::Refresh,
            Tab::Services => Icon::Sliders,
            Tab::Diagnose => Icon::Shield,
            Tab::Logs => Icon::Terminal,
        }
    }
}

/// Optionen für den Befehl, den `playforge-run` erzeugen soll.
#[derive(Clone, Debug)]
pub struct RunOptions {
    pub hud: bool,
    pub gamemode: bool,
    pub gamescope: bool,
    pub windowed: bool,
    pub resolution: String,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            hud: true,
            gamemode: true,
            gamescope: false,
            windowed: false,
            resolution: "1920x1080".to_string(),
        }
    }
}

impl RunOptions {
    pub fn command(&self) -> String {
        let mut parts = vec!["playforge-run".to_string()];
        if self.hud {
            parts.push("--hud".to_string());
        }
        if !self.gamemode {
            parts.push("--no-gamemode".to_string());
        }
        if self.gamescope {
            parts.push("--gamescope".to_string());
            if self.windowed {
                parts.push("--windowed".to_string());
            }
            parts.push("--resolution".to_string());
            parts.push(self.resolution.clone());
        }
        parts.push("--".to_string());
        parts.push("%command%".to_string());
        parts.join(" ")
    }
}

#[derive(Clone, Debug)]
pub struct TuneDraft {
    pub governor: String,
    pub epp: String,
    pub swappiness: u32,
    pub cache_pressure: u32,
    pub max_map: u32,
    pub inotify: u32,
    pub io_scheduler: String,
    pub mesa_glthread: bool,
    pub amd_icd: String,
    pub radv_gpl: bool,
}

impl Default for TuneDraft {
    fn default() -> Self {
        Self {
            governor: "performance".to_string(),
            epp: "default".to_string(),
            swappiness: 10,
            cache_pressure: 50,
            max_map: 65_530,
            inotify: 524_288,
            io_scheduler: "mq-deadline".to_string(),
            mesa_glthread: true,
            amd_icd: "RADV".to_string(),
            radv_gpl: false,
        }
    }
}

#[derive(Clone)]
struct StatusMessage {
    ok: bool,
    text: String,
}

pub struct ControlCenter {
    pub tab: Tab,
    pub snapshot: Option<Arc<Snapshot>>,
    refresh_rx: Option<Receiver<Snapshot>>,
    last_refresh: Option<Instant>,

    pub doctor: Option<DoctorReport>,
    doctor_rx: Option<Receiver<DoctorReport>>,
    doctor_started: bool,

    pub logs: Option<Result<String, String>>,
    logs_rx: Option<Receiver<Result<String, String>>>,
    pub log_mode: LogMode,
    pub log_filter: String,

    pub profile_choice: String,
    pub shader_choice: String,
    pub gamemode_draft: GamemodeSettings,
    pub mangohud_choice: String,
    pub tune: TuneDraft,
    pub autoupdate_choice: String,
    drafts_ready: bool,

    pub run_options: RunOptions,

    pub update_report: Option<UpdateReport>,
    update_attempted: bool,
    update_check_rx: Option<Receiver<UpdateReport>>,
    update_rx: Option<Receiver<ActionEvent>>,
    pub update_log: Vec<String>,
    pub update_done: Option<ActionResult>,
    pub update_flatpak: bool,
    pub update_mirrors: bool,

    action_rx: Option<Receiver<ActionResult>>,
    action_label: String,
    status: Option<StatusMessage>,

    pub helper_available: bool,
}

impl ControlCenter {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            tab: Tab::Overview,
            snapshot: None,
            refresh_rx: None,
            last_refresh: None,
            doctor: None,
            doctor_rx: None,
            doctor_started: false,
            logs: None,
            logs_rx: None,
            log_mode: LogMode::Playforge,
            log_filter: String::new(),
            profile_choice: "balanced".to_string(),
            shader_choice: "12G".to_string(),
            gamemode_draft: GamemodeSettings::default(),
            mangohud_choice: "standard".to_string(),
            tune: TuneDraft::default(),
            autoupdate_choice: "off".to_string(),
            drafts_ready: false,
            run_options: RunOptions::default(),
            update_report: None,
            update_attempted: false,
            update_check_rx: None,
            update_rx: None,
            update_log: Vec::new(),
            update_done: None,
            update_flatpak: true,
            update_mirrors: false,
            action_rx: None,
            action_label: String::new(),
            status: None,
            helper_available: cfg!(target_os = "linux") && Path::new(actions::HELPER).exists(),
        };
        app.schedule_refresh();
        app
    }

    // -----------------------------------------------------------------
    // Hintergrundarbeit
    // -----------------------------------------------------------------

    pub fn schedule_refresh(&mut self) {
        if self.refresh_rx.is_some() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(system::collect());
        });
        self.refresh_rx = Some(rx);
    }

    pub fn apply_helper(&mut self, args: Vec<String>, label: &str) {
        if self.action_rx.is_some() {
            return;
        }
        self.action_label = label.to_string();
        self.action_rx = Some(actions::spawn_helper(args, label.to_string()));
    }

    pub fn apply_theme(&mut self) {
        if self.action_rx.is_some() {
            return;
        }
        self.action_label = "Theme".to_string();
        self.action_rx = Some(actions::spawn_command(
            "playforge-theme".to_string(),
            Vec::new(),
        ));
    }

    pub fn start_doctor(&mut self) {
        if self.doctor_rx.is_some() {
            return;
        }
        self.doctor_started = true;
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(system::run_doctor());
        });
        self.doctor_rx = Some(rx);
    }

    pub fn doctor_pending(&self) -> bool {
        self.doctor_rx.is_some()
    }

    /// Prüft im Hintergrund, ob pacman, AUR und Flatpak Updates anbieten.
    pub fn start_update_check(&mut self) {
        if self.update_check_rx.is_some() {
            return;
        }
        self.update_attempted = true;
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(system::collect_updates());
        });
        self.update_check_rx = Some(rx);
    }

    pub fn update_check_pending(&self) -> bool {
        self.update_check_rx.is_some()
    }

    /// Startet das Systemupdate und zeigt die Ausgabe live an.
    pub fn start_update(&mut self) {
        if self.update_rx.is_some() {
            return;
        }
        let mut args = vec!["updates".to_string(), "apply".to_string()];
        if self.update_flatpak {
            args.push("--flatpak".to_string());
        }
        if self.update_mirrors {
            args.push("--mirrors".to_string());
        }
        self.update_log.clear();
        self.update_done = None;
        self.update_rx = Some(actions::spawn_stream(args, "Systemupdate".to_string()));
    }

    pub fn update_pending(&self) -> bool {
        self.update_rx.is_some()
    }

    /// Schaltet den systemd-Timer für automatische Updates.
    pub fn apply_autoupdate(&mut self) {
        let args = vec!["autoupdate".to_string(), self.autoupdate_choice.clone()];
        self.apply_helper(args, "Auto-Update");
    }

    fn push_update_line(&mut self, line: String) {
        if line.trim().is_empty() {
            return;
        }
        self.update_log.push(line);
        if self.update_log.len() > 400 {
            let excess = self.update_log.len() - 400;
            self.update_log.drain(..excess);
        }
    }

    pub fn load_logs(&mut self) {
        if self.logs_rx.is_some() {
            return;
        }
        let mode = self.log_mode;
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(system::read_logs(mode, LOG_LINES));
        });
        self.logs_rx = Some(rx);
        self.logs = None;
    }

    pub fn logs_pending(&self) -> bool {
        self.logs_rx.is_some()
    }

    pub fn action_pending(&self) -> bool {
        self.action_rx.is_some() || self.update_rx.is_some()
    }

    fn poll(&mut self) {
        if let Some(rx) = &self.refresh_rx {
            match rx.try_recv() {
                Ok(snapshot) => {
                    let snapshot = Arc::new(snapshot);
                    if !self.drafts_ready {
                        self.init_drafts(&snapshot);
                    }
                    self.snapshot = Some(snapshot);
                    self.last_refresh = Some(Instant::now());
                    self.refresh_rx = None;
                }
                Err(TryRecvError::Disconnected) => {
                    self.refresh_rx = None;
                    self.last_refresh = Some(Instant::now());
                }
                Err(TryRecvError::Empty) => {}
            }
        }

        if let Some(rx) = &self.action_rx {
            match rx.try_recv() {
                Ok(result) => {
                    let succeeded = result.ok;
                    self.status = Some(StatusMessage {
                        ok: succeeded,
                        text: result.message,
                    });
                    self.action_rx = None;
                    if succeeded {
                        self.drafts_ready = false;
                        self.schedule_refresh();
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    self.status = Some(StatusMessage {
                        ok: false,
                        text: "Die Aktion wurde unerwartet beendet.".to_string(),
                    });
                    self.action_rx = None;
                }
                Err(TryRecvError::Empty) => {}
            }
        }

        if let Some(rx) = self.update_check_rx.take() {
            match rx.try_recv() {
                Ok(report) => {
                    self.update_report = Some(report);
                }
                Err(TryRecvError::Disconnected) => {
                    self.status = Some(StatusMessage {
                        ok: false,
                        text: "Die Update-Prüfung wurde unerwartet beendet.".to_string(),
                    });
                }
                Err(TryRecvError::Empty) => {
                    self.update_check_rx = Some(rx);
                }
            }
        }

        if let Some(rx) = self.update_rx.take() {
            let mut finished = false;
            loop {
                match rx.try_recv() {
                    Ok(ActionEvent::Line(line)) => self.push_update_line(line),
                    Ok(ActionEvent::Done(result)) => {
                        let ok = result.ok;
                        self.status = Some(StatusMessage {
                            ok,
                            text: result.message.clone(),
                        });
                        self.update_done = Some(result);
                        finished = true;
                        self.drafts_ready = false;
                        self.start_update_check();
                        self.schedule_refresh();
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        if self.update_done.is_none() {
                            self.status = Some(StatusMessage {
                                ok: false,
                                text: "Die Aktualisierung wurde unerwartet beendet.".to_string(),
                            });
                        }
                        finished = true;
                        break;
                    }
                }
            }
            if !finished {
                self.update_rx = Some(rx);
            }
        }

        if let Some(rx) = &self.doctor_rx {
            match rx.try_recv() {
                Ok(report) => {
                    self.doctor = Some(report);
                    self.doctor_rx = None;
                }
                Err(TryRecvError::Disconnected) => {
                    self.doctor_rx = None;
                }
                Err(TryRecvError::Empty) => {}
            }
        }

        if let Some(rx) = &self.logs_rx {
            match rx.try_recv() {
                Ok(result) => {
                    self.logs = Some(result);
                    self.logs_rx = None;
                }
                Err(TryRecvError::Disconnected) => {
                    self.logs = Some(Err("Protokoll konnte nicht gelesen werden.".to_string()));
                    self.logs_rx = None;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    }

    fn init_drafts(&mut self, snapshot: &Snapshot) {
        let profile = snapshot.tuning.profile_saved.to_lowercase();
        self.profile_choice = if profile.starts_with("perf") {
            "performance".to_string()
        } else if profile.starts_with("reset") {
            "reset".to_string()
        } else {
            "balanced".to_string()
        };

        self.shader_choice = if snapshot.settings.shader_cache.is_empty() {
            "12G".to_string()
        } else {
            snapshot.settings.shader_cache.clone()
        };

        self.gamemode_draft = snapshot.settings.gamemode.clone();

        self.mangohud_choice = match snapshot.settings.mangohud_preset.as_str() {
            "compact" => "compact",
            "detailed" => "detailed",
            "off" => "off",
            _ => "standard",
        }
        .to_string();

        let governor = if snapshot.tuning.governor == "—" {
            snapshot
                .tuning
                .governors
                .first()
                .cloned()
                .unwrap_or_else(|| "performance".to_string())
        } else {
            snapshot.tuning.governor.clone()
        };
        let epp = if snapshot.tuning.epp == "—" {
            snapshot
                .tuning
                .epps
                .first()
                .cloned()
                .unwrap_or_else(|| "default".to_string())
        } else {
            snapshot.tuning.epp.clone()
        };
        let io_scheduler = snapshot
            .tuning
            .io_devices
            .iter()
            .map(|device| device.scheduler.clone())
            .find(|scheduler| !scheduler.is_empty())
            .unwrap_or_else(|| "mq-deadline".to_string());

        self.tune = TuneDraft {
            governor,
            epp,
            swappiness: snapshot.tuning.swappiness.unwrap_or(10).min(200) as u32,
            cache_pressure: snapshot.tuning.vfs_cache_pressure.unwrap_or(50).min(1000) as u32,
            max_map: snapshot
                .tuning
                .max_map_count
                .filter(|value| *value > 0)
                .unwrap_or(65_530)
                .min(u32::MAX as u64) as u32,
            inotify: snapshot
                .tuning
                .inotify_watches
                .unwrap_or(524_288)
                .clamp(8192, 1_048_576) as u32,
            io_scheduler,
            mesa_glthread: snapshot.tuning.graphics_env.mesa_glthread.unwrap_or(true),
            amd_icd: snapshot
                .tuning
                .graphics_env
                .amd_icd
                .clone()
                .unwrap_or_else(|| "RADV".to_string()),
            radv_gpl: snapshot.tuning.graphics_env.radv_gpl,
        };

        self.autoupdate_choice = match snapshot.tuning.autoupdate.mode.as_str() {
            "daily" => "daily",
            "weekly" => "weekly",
            _ => "off",
        }
        .to_string();

        self.drafts_ready = true;
    }

    // -----------------------------------------------------------------
    // Aufbau der Oberfläche
    // -----------------------------------------------------------------

    fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(RAIL_WIDTH)
            .frame(
                egui::Frame::new()
                    .fill(theme::INK)
                    .stroke(Stroke::new(1.0_f32, theme::HAIRLINE))
                    .inner_margin(Margin::symmetric(14, 16)),
            )
            .show(ctx, |ui| {
                self.brand(ui);

                ui.add_space(18.0);
                ui::hairline(ui);
                ui.add_space(14.0);

                for tab in Tab::ALL {
                    self.nav_item(ui, tab);
                    ui.add_space(2.0);
                }

                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    ui::mono_label(ui, "V1.0.0 // ASCII BRUTAL", theme::FAINT, 8.5);
                    ui.add_space(9.0);
                    ui::hairline(ui);
                    ui.add_space(9.0);
                    if let Some(snapshot) = &self.snapshot {
                        let profile = snapshot.tuning.profile_saved.to_lowercase();
                        let (label, color) = if profile.starts_with("perf") {
                            ("PROFIL · PERFORMANCE", theme::ACCENT)
                        } else if profile.starts_with("reset") {
                            ("PROFIL · RESET", theme::MUTED)
                        } else {
                            ("PROFIL · BALANCED", theme::OK)
                        };
                        ui::badge(ui, label, color);
                    }
                });
            });
    }

    fn brand(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::hover());
            let painter = ui.painter();
            painter.rect_filled(rect, CornerRadius::ZERO, theme::INK);
            painter.rect_stroke(
                rect,
                CornerRadius::ZERO,
                Stroke::new(1.0_f32, theme::TEXT),
                egui::StrokeKind::Inside,
            );
            let c = rect.center();
            let apex = c + Vec2::new(0.0, -7.0);
            for leg in [-8.0_f32, 8.0] {
                painter.line_segment(
                    [apex, c + Vec2::new(leg, 6.0)],
                    Stroke::new(2.0_f32, theme::ACCENT),
                );
            }
            painter.rect_filled(
                Rect::from_center_size(c + Vec2::new(0.0, 9.0), Vec2::new(5.0, 5.0)),
                CornerRadius::ZERO,
                theme::TEXT,
            );

            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("PLAYFORGE")
                        .font(theme::mono(10.5))
                        .color(theme::TEXT),
                );
                ui.label(
                    RichText::new("// CONTROL")
                        .font(theme::mono(8.0))
                        .color(theme::ACCENT),
                );
            });
        });
    }

    fn nav_item(&mut self, ui: &mut egui::Ui, tab: Tab) {
        let selected = self.tab == tab;
        let height = 34.0;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
        let painter = ui.painter();

        if selected {
            painter.rect_filled(rect, CornerRadius::ZERO, theme::ACCENT);
        } else if response.hovered() {
            painter.rect_filled(rect, CornerRadius::ZERO, theme::alpha(theme::TEXT, 10));
        }

        painter.text(
            rect.left_center() + Vec2::new(12.0, 0.0),
            Align2::LEFT_CENTER,
            tab.index(),
            theme::mono(8.5),
            if selected { theme::INK } else { theme::FAINT },
        );

        let icon_rect = Rect::from_min_size(
            rect.left_center() + Vec2::new(34.0, -8.0),
            Vec2::splat(16.0),
        );
        icons::paint(
            painter,
            icon_rect,
            tab.icon(),
            if selected { theme::INK } else { theme::MUTED },
        );

        painter.text(
            rect.left_center() + Vec2::new(58.0, 0.0),
            Align2::LEFT_CENTER,
            tab.title().to_uppercase(),
            theme::mono(10.5),
            if selected { theme::INK } else { theme::MUTED },
        );

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if response.clicked() {
            self.tab = tab;
        }
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        let status = self.status.clone();
        let pending = self.action_pending();
        let label = self.action_label.clone();

        egui::TopBottomPanel::bottom("status")
            .exact_height(32.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::INK)
                    .stroke(Stroke::new(1.0_f32, theme::HAIRLINE))
                    .inner_margin(Margin::symmetric(18, 6)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (color, text) = match &status {
                        Some(message) if message.ok => (theme::OK, message.text.clone()),
                        Some(message) => (theme::ERROR, message.text.clone()),
                        None => (
                            theme::ACCENT,
                            "Systemdaten werden alle 8 Sekunden neu eingelesen.".to_string(),
                        ),
                    };
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(7.0), Sense::hover());
                    ui.painter().rect_filled(rect, CornerRadius::ZERO, color);
                    ui.label(
                        RichText::new(text)
                            .font(theme::mono(9.5))
                            .color(theme::MUTED),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if pending {
                            ui.spinner();
                            ui.label(
                                RichText::new(format!("{} LÄUFT …", label.to_uppercase()))
                                    .font(theme::mono(9.0))
                                    .color(theme::ACCENT),
                            );
                        } else {
                            ui.label(
                                RichText::new("PKEXEC · SYSTEMÄNDERUNGEN AUTORISIERT")
                                    .font(theme::mono(9.0))
                                    .color(theme::FAINT),
                            );
                        }
                    });
                });
            });
    }

    fn content(&mut self, ui: &mut egui::Ui) {
        match self.tab {
            Tab::Overview => ui::overview::show(self, ui),
            Tab::Performance => ui::performance::show(self, ui),
            Tab::Gaming => ui::gaming::show(self, ui),
            Tab::Graphics => ui::graphics::show(self, ui),
            Tab::Tuning => ui::tuning::show(self, ui),
            Tab::Updates => {
                if !self.update_attempted
                    && self.update_report.is_none()
                    && !self.update_check_pending()
                {
                    self.start_update_check();
                }
                ui::updates::show(self, ui);
            }
            Tab::Services => ui::services::show(self, ui),
            Tab::Diagnose => {
                if !self.doctor_started && self.doctor_rx.is_none() {
                    self.start_doctor();
                }
                ui::diagnose::show(self, ui);
            }
            Tab::Logs => {
                if self.logs.is_none() && self.logs_rx.is_none() {
                    self.load_logs();
                }
                ui::logs::show(self, ui);
            }
        }
    }

    fn central(&mut self, ctx: &egui::Context) {
        let tab = self.tab;
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::INK).inner_margin(Margin {
                left: 24,
                right: 18,
                top: 18,
                bottom: 12,
            }))
            .show(ctx, |ui| {
                let mut refresh_now = false;
                let refreshing = self.refresh_rx.is_some();
                let last_refresh = self.last_refresh;

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui::kicker(ui, &format!("PLAYFORGE :: {} / {}", tab.index(), tab.slug()));
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(tab.title().to_uppercase())
                                .font(theme::display(28.0))
                                .color(theme::TEXT),
                        );
                        ui.add_space(1.0);
                        ui.label(
                            RichText::new(tab.subtitle())
                                .font(theme::sans(12.0))
                                .color(theme::MUTED),
                        );
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui::icon_button(ui, Icon::Refresh, "Neu einlesen", theme::ACCENT)
                            .clicked()
                        {
                            refresh_now = true;
                        }
                        if refreshing {
                            ui.spinner();
                        } else if let Some(last) = last_refresh {
                            ui.label(
                                RichText::new(format!("VOR {:.0} S", last.elapsed().as_secs_f32()))
                                    .font(theme::mono(9.0))
                                    .color(theme::FAINT),
                            );
                        }
                    });
                });

                if refresh_now {
                    self.schedule_refresh();
                }

                ui.add_space(14.0);
                ui::hairline(ui);
                ui.add_space(14.0);

                egui::ScrollArea::vertical()
                    .id_salt("page-content")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(2.0);
                        self.content(ui);
                        ui.add_space(16.0);
                    });
            });
    }
}

impl eframe::App for ControlCenter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();

        let due = self
            .last_refresh
            .map(|last| last.elapsed() >= REFRESH_INTERVAL)
            .unwrap_or(true);
        if due {
            self.schedule_refresh();
        }

        self.sidebar(ctx);
        self.status_bar(ctx);
        self.central(ctx);

        if self.refresh_rx.is_some()
            || self.action_rx.is_some()
            || self.doctor_rx.is_some()
            || self.logs_rx.is_some()
            || self.update_rx.is_some()
            || self.update_check_rx.is_some()
        {
            ctx.request_repaint_after(Duration::from_millis(120));
        } else {
            ctx.request_repaint_after(Duration::from_secs(1));
        }
    }
}
