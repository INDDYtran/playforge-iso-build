//! Fenster, Navigation und Zustand des Control Centers.

use std::path::Path;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align, Align2, CornerRadius, FontId, Layout, Margin, Rect, RichText, Sense, Stroke, Vec2,
};

use crate::actions::{self, ActionResult};
use crate::icons::{self, Icon};
use crate::model::{GamemodeSettings, Snapshot};
use crate::system::{self, DoctorReport, LogMode};
use crate::theme;
use crate::ui;

const REFRESH_INTERVAL: Duration = Duration::from_secs(8);
const LOG_LINES: usize = 400;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Overview,
    Performance,
    Gaming,
    Graphics,
    Services,
    Diagnose,
    Logs,
}

impl Tab {
    pub const ALL: [Tab; 7] = [
        Tab::Overview,
        Tab::Performance,
        Tab::Gaming,
        Tab::Graphics,
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
            Tab::Services => "Systemdienste steuern und Autostart setzen",
            Tab::Diagnose => "Systemprüfung mit playforge-doctor",
            Tab::Logs => "Journal des laufenden Boot-Vorgangs",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Tab::Overview => Icon::Dashboard,
            Tab::Performance => Icon::Gauge,
            Tab::Gaming => Icon::Gamepad,
            Tab::Graphics => Icon::Monitor,
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
    drafts_ready: bool,

    pub run_options: RunOptions,

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
            drafts_ready: false,
            run_options: RunOptions::default(),
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
        self.action_rx.is_some()
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

        self.drafts_ready = true;
    }

    // -----------------------------------------------------------------
    // Aufbau der Oberfläche
    // -----------------------------------------------------------------

    fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(240.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::SIDEBAR)
                    .inner_margin(Margin::symmetric(14, 18)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), Sense::hover());
                    let painter = ui.painter();
                    painter.rect_filled(rect, CornerRadius::same(11), theme::SURFACE_HI);
                    painter.rect_stroke(
                        rect,
                        CornerRadius::same(11),
                        Stroke::new(1.0_f32, theme::BORDER),
                        egui::StrokeKind::Inside,
                    );
                    let c = rect.center();
                    let r = rect.width() * 0.30;
                    let tri = vec![
                        c + Vec2::new(-0.55 * r, -0.80 * r),
                        c + Vec2::new(0.95 * r, 0.0),
                        c + Vec2::new(-0.55 * r, 0.80 * r),
                    ];
                    painter.add(egui::Shape::convex_polygon(tri, theme::AMBER, Stroke::NONE));
                    painter.circle_filled(c + Vec2::new(0.85 * r, -0.95 * r), 2.6, theme::WARN);
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Playforge").size(16.0).strong());
                        ui.label(
                            RichText::new("Control Center")
                                .size(11.0)
                                .color(theme::MUTED),
                        );
                    });
                });

                ui.add_space(20.0);

                for tab in Tab::ALL {
                    self.nav_item(ui, tab);
                    ui.add_space(2.0);
                }

                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    ui.label(RichText::new("Version 1.0").size(10.5).color(theme::FAINT));
                    ui.add_space(8.0);
                    if let Some(snapshot) = &self.snapshot {
                        let profile = snapshot.tuning.profile_saved.to_lowercase();
                        let (label, color) = if profile.starts_with("perf") {
                            ("Profil: performance", theme::AMBER)
                        } else {
                            ("Profil: balanced", theme::GREEN)
                        };
                        ui::badge(ui, label, color);
                    }
                    ui.add_space(8.0);
                    ui::divider(ui);
                    ui.add_space(8.0);
                });
            });
    }

    fn nav_item(&mut self, ui: &mut egui::Ui, tab: Tab) {
        let selected = self.tab == tab;
        let height = 38.0;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());
        let painter = ui.painter();

        if selected {
            painter.rect_filled(
                rect,
                CornerRadius::same(10),
                theme::AMBER.gamma_multiply(0.12),
            );
            let bar = Rect::from_min_size(
                rect.left_top() + Vec2::new(0.0, 9.0),
                Vec2::new(3.0, height - 18.0),
            );
            painter.rect_filled(bar, CornerRadius::same(2), theme::AMBER);
        } else if response.hovered() {
            painter.rect_filled(rect, CornerRadius::same(10), theme::SURFACE_HI);
        }

        let icon_rect = Rect::from_min_size(
            rect.left_center() + Vec2::new(14.0, -9.0),
            Vec2::splat(18.0),
        );
        icons::paint(
            painter,
            icon_rect,
            tab.icon(),
            if selected { theme::AMBER } else { theme::MUTED },
        );
        painter.text(
            rect.left_center() + Vec2::new(44.0, 0.0),
            Align2::LEFT_CENTER,
            tab.title(),
            FontId::proportional(13.5),
            if selected { theme::TEXT } else { theme::MUTED },
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
            .exact_height(34.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::SIDEBAR)
                    .inner_margin(Margin::symmetric(18, 7)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (color, text) = match &status {
                        Some(message) if message.ok => (theme::GREEN, message.text.clone()),
                        Some(message) => (theme::RED, message.text.clone()),
                        None => (
                            theme::AMBER,
                            "Systemdaten werden automatisch alle 8 Sekunden neu eingelesen."
                                .to_string(),
                        ),
                    };
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                    ui.painter().circle_filled(rect.center(), 3.5, color);
                    ui.label(RichText::new(text).size(11.5).color(theme::MUTED));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if pending {
                            ui.spinner();
                            ui.label(
                                RichText::new(format!("{label} läuft …"))
                                    .size(11.0)
                                    .color(theme::AMBER),
                            );
                        } else {
                            ui.label(
                                RichText::new("Systemänderungen werden über pkexec autorisiert.")
                                    .size(11.0)
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
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(Margin {
                left: 26,
                right: 20,
                top: 18,
                bottom: 12,
            }))
            .show(ctx, |ui| {
                let mut refresh_now = false;
                let refreshing = self.refresh_rx.is_some();
                let last_refresh = self.last_refresh;

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(tab.title()).size(23.0).strong());
                        ui.label(RichText::new(tab.subtitle()).size(12.5).color(theme::MUTED));
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui::icon_button(ui, Icon::Refresh, "Aktualisieren", theme::AMBER)
                            .clicked()
                        {
                            refresh_now = true;
                        }
                        if refreshing {
                            ui.spinner();
                        } else if let Some(last) = last_refresh {
                            ui.label(
                                RichText::new(format!("vor {:.0} s", last.elapsed().as_secs_f32()))
                                    .size(11.0)
                                    .color(theme::FAINT),
                            );
                        }
                    });
                });

                if refresh_now {
                    self.schedule_refresh();
                }

                ui.add_space(8.0);
                ui::divider(ui);
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
        {
            ctx.request_repaint_after(Duration::from_millis(120));
        } else {
            ctx.request_repaint_after(Duration::from_secs(1));
        }
    }
}
