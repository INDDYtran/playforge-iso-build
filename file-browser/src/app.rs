//! Fenster, Navigation und Oberfläche des Dateibrowsers.

use std::path::{Component, Path, PathBuf};

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, Layout, Margin, Rect, RichText, Sense, Stroke, Vec2,
};

use crate::fs::{self, Entry, Place, PlaceKind};
use crate::icons::{self, Icon};
use crate::theme;

const ROW_HEIGHT: f32 = 30.0;
const RAIL_WIDTH: f32 = 218.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortKey {
    Name,
    Size,
    Modified,
}

enum PreviewKind {
    Directory,
    Text {
        content: String,
        truncated: bool,
    },
    Image {
        texture: egui::TextureHandle,
        size: Vec2,
    },
    Unavailable,
}

struct Preview {
    path: PathBuf,
    kind: PreviewKind,
}

enum Dialog {
    Rename { target: PathBuf },
    Delete { target: PathBuf, is_dir: bool },
    NewFolder,
}

enum Command {
    Select(PathBuf),
    Open(PathBuf, bool),
    Rename(PathBuf),
    Delete(PathBuf, bool),
    Copy(PathBuf),
}

struct RowResponse {
    clicked: bool,
    double: bool,
    secondary: bool,
    command: Option<Command>,
}

pub struct FilesApp {
    cwd: PathBuf,
    entries: Vec<Entry>,
    visible: Vec<usize>,
    selected: Option<PathBuf>,
    history: Vec<PathBuf>,
    hist_pos: usize,
    show_hidden: bool,
    query: String,
    sort: SortKey,
    ascending: bool,
    error: Option<String>,
    status: Option<(bool, String)>,
    preview: Option<Preview>,
    dialog: Option<Dialog>,
    input: String,
    focus_dialog: bool,
    focus_search: bool,
    places: Vec<Place>,
    mounts: Vec<Place>,
}

impl FilesApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (start, initial_selection) = start_location();
        let cwd = std::fs::canonicalize(&start).unwrap_or(start);

        let mut app = Self {
            cwd: cwd.clone(),
            entries: Vec::new(),
            visible: Vec::new(),
            selected: None,
            history: vec![cwd],
            hist_pos: 0,
            show_hidden: false,
            query: String::new(),
            sort: SortKey::Name,
            ascending: true,
            error: None,
            status: None,
            preview: None,
            dialog: None,
            input: String::new(),
            focus_dialog: false,
            focus_search: false,
            places: Vec::new(),
            mounts: Vec::new(),
        };
        app.refresh_places();
        app.load();
        if let Some(path) = initial_selection.and_then(|path| std::fs::canonicalize(path).ok()) {
            if app.entries.iter().any(|entry| entry.path == path) {
                app.selected = Some(path);
            }
        }
        app
    }

    // -----------------------------------------------------------------
    // Zustand
    // -----------------------------------------------------------------

    fn refresh_places(&mut self) {
        self.places = fs::places();
        self.mounts = fs::mounts();
    }

    fn load(&mut self) {
        match fs::list(&self.cwd) {
            Ok(entries) => {
                self.entries = entries;
                self.error = None;
            }
            Err(err) => {
                self.entries.clear();
                self.error = Some(format!(
                    "{} lässt sich nicht öffnen: {err}",
                    self.cwd.display()
                ));
            }
        }
        if let Some(selected) = &self.selected {
            if !self.entries.iter().any(|entry| &entry.path == selected) {
                self.selected = None;
            }
        }
        self.apply_view();
    }

    fn apply_view(&mut self) {
        let query = self.query.trim().to_lowercase();
        let mut visible: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| self.show_hidden || !entry.hidden)
            .filter(|(_, entry)| query.is_empty() || entry.name.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .collect();

        let (sort, ascending) = (self.sort, self.ascending);
        visible.sort_by(|&a, &b| {
            let (left, right) = (&self.entries[a], &self.entries[b]);
            let directories = right.is_dir.cmp(&left.is_dir);
            if directories != std::cmp::Ordering::Equal {
                return directories;
            }
            let order = match sort {
                SortKey::Name => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
                SortKey::Size => left.size.cmp(&right.size),
                SortKey::Modified => left.modified.cmp(&right.modified),
            };
            if ascending {
                order
            } else {
                order.reverse()
            }
        });
        self.visible = visible;
    }

    fn selected_entry(&self) -> Option<&Entry> {
        let selected = self.selected.as_ref()?;
        self.entries.iter().find(|entry| &entry.path == selected)
    }

    fn navigate_to(&mut self, path: PathBuf) {
        let path = std::fs::canonicalize(&path).unwrap_or(path);
        if !path.is_dir() {
            self.status = Some((false, format!("Kein Ordner: {}", path.display())));
            return;
        }
        if self.cwd != path {
            self.history.truncate(self.hist_pos + 1);
            self.history.push(path.clone());
            self.hist_pos = self.history.len() - 1;
        }
        self.cwd = path;
        self.selected = None;
        self.query.clear();
        self.load();
    }

    fn go_history(&mut self, path: PathBuf) {
        self.cwd = path;
        self.selected = None;
        self.query.clear();
        self.load();
    }

    fn back(&mut self) {
        if self.hist_pos > 0 {
            self.hist_pos -= 1;
            let path = self.history[self.hist_pos].clone();
            self.go_history(path);
        }
    }

    fn forward(&mut self) {
        if self.hist_pos + 1 < self.history.len() {
            self.hist_pos += 1;
            let path = self.history[self.hist_pos].clone();
            self.go_history(path);
        }
    }

    fn up(&mut self) {
        if let Some(parent) = self.cwd.parent() {
            let parent = parent.to_path_buf();
            self.navigate_to(parent);
        }
    }

    fn refresh(&mut self) {
        self.refresh_places();
        self.load();
        self.status = Some((true, "Ordner neu eingelesen.".to_string()));
    }

    fn activate(&mut self, path: PathBuf, is_dir: bool) {
        if is_dir {
            self.navigate_to(path);
            return;
        }
        let name = file_name(&path);
        match fs::open_external(&path) {
            Ok(()) => self.status = Some((true, format!("Öffne „{name}“ …"))),
            Err(err) => {
                self.status = Some((false, format!("„{name}“ nicht geöffnet: {err}")));
            }
        }
    }

    fn apply_command(&mut self, ctx: &egui::Context, command: Command) {
        match command {
            Command::Select(path) => self.selected = Some(path),
            Command::Open(path, is_dir) => self.activate(path, is_dir),
            Command::Rename(path) => {
                self.input = file_name(&path);
                self.focus_dialog = true;
                self.dialog = Some(Dialog::Rename { target: path });
            }
            Command::Delete(path, is_dir) => {
                self.dialog = Some(Dialog::Delete {
                    target: path,
                    is_dir,
                });
            }
            Command::Copy(path) => {
                ctx.copy_text(path.display().to_string());
                self.status = Some((true, "Pfad in die Zwischenablage kopiert.".to_string()));
            }
        }
    }

    fn start_new_folder(&mut self) {
        self.input = "Neuer Ordner".to_string();
        self.focus_dialog = true;
        self.dialog = Some(Dialog::NewFolder);
    }

    // -----------------------------------------------------------------
    // Eingabe
    // -----------------------------------------------------------------

    fn handle_keys(&mut self, ctx: &egui::Context) {
        if self.dialog.is_some() {
            return;
        }

        if ctx.wants_keyboard_input() {
            if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
                self.query.clear();
                self.apply_view();
            }
            return;
        }

        let (up, down, enter, backspace, delete, rename, hidden, search, escape, reload) = ctx
            .input(|input| {
                (
                    input.key_pressed(egui::Key::ArrowUp),
                    input.key_pressed(egui::Key::ArrowDown),
                    input.key_pressed(egui::Key::Enter),
                    input.key_pressed(egui::Key::Backspace),
                    input.key_pressed(egui::Key::Delete),
                    input.key_pressed(egui::Key::F2),
                    input.modifiers.command && input.key_pressed(egui::Key::H),
                    input.modifiers.command && input.key_pressed(egui::Key::F),
                    input.key_pressed(egui::Key::Escape),
                    input.key_pressed(egui::Key::F5),
                )
            });

        if up || down {
            if !self.visible.is_empty() {
                let current = self.selected.as_ref().and_then(|path| {
                    self.visible
                        .iter()
                        .position(|&index| &self.entries[index].path == path)
                });
                let next = match current {
                    Some(index) if down => (index + 1).min(self.visible.len() - 1),
                    Some(index) => index.saturating_sub(1),
                    None => 0,
                };
                self.selected = Some(self.entries[self.visible[next]].path.clone());
            }
        }

        if enter {
            if let Some(entry) = self.selected_entry().cloned() {
                self.activate(entry.path, entry.is_dir);
            }
        }

        if backspace {
            self.up();
        }

        if reload {
            self.refresh();
        }

        if let Some(entry) = self.selected_entry().cloned() {
            if delete {
                self.dialog = Some(Dialog::Delete {
                    target: entry.path.clone(),
                    is_dir: entry.is_dir,
                });
            }
            if rename {
                self.input = entry.name.clone();
                self.focus_dialog = true;
                self.dialog = Some(Dialog::Rename { target: entry.path });
            }
        }

        if hidden {
            self.show_hidden = !self.show_hidden;
            self.apply_view();
        }

        if search {
            self.focus_search = true;
        }

        if escape && !self.query.is_empty() {
            self.query.clear();
            self.apply_view();
        }
    }

    // -----------------------------------------------------------------
    // Vorschau
    // -----------------------------------------------------------------

    fn sync_preview(&mut self, ctx: &egui::Context) {
        let selected = self.selected.clone();
        if self.preview.as_ref().map(|preview| preview.path.clone()) == selected {
            return;
        }
        let next = selected.and_then(|path| {
            let entry = self
                .entries
                .iter()
                .find(|entry| entry.path == path)?
                .clone();
            Some(Preview {
                path,
                kind: build_preview(ctx, &entry),
            })
        });
        self.preview = next;
    }

    // -----------------------------------------------------------------
    // Aufbau der Oberfläche
    // -----------------------------------------------------------------

    fn sidebar(&mut self, ctx: &egui::Context) {
        let mut target = None;

        egui::SidePanel::left("places")
            .resizable(false)
            .exact_width(RAIL_WIDTH)
            .frame(
                egui::Frame::new()
                    .fill(theme::INK)
                    .stroke(Stroke::new(1.0_f32, theme::HAIRLINE))
                    .inner_margin(Margin::symmetric(12, 16)),
            )
            .show(ctx, |ui| {
                self.brand(ui);

                ui.add_space(16.0);
                section_label(ui, "ORTE");
                ui.add_space(2.0);
                for place in &self.places {
                    if place_row(
                        ui,
                        &place.label,
                        place_icon(&place.kind),
                        self.cwd == place.path,
                    )
                    .clicked()
                    {
                        target = Some(place.path.clone());
                    }
                }

                if !self.mounts.is_empty() {
                    ui.add_space(14.0);
                    section_label(ui, "GERÄTE");
                    ui.add_space(2.0);
                    for place in &self.mounts {
                        if place_row(
                            ui,
                            &place.label,
                            place_icon(&place.kind),
                            self.cwd == place.path,
                        )
                        .clicked()
                        {
                            target = Some(place.path.clone());
                        }
                    }
                }

                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    ui.label(
                        RichText::new("PLAYFORGE FILES // ASCII BRUTAL")
                            .font(theme::mono(8.0))
                            .color(theme::FAINT),
                    );
                });
            });

        if let Some(path) = target {
            self.navigate_to(path);
        }
    }

    fn brand(&self, ui: &mut egui::Ui) {
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
            icons::paint(
                painter,
                Rect::from_center_size(rect.center(), Vec2::splat(18.0)),
                Icon::Folder,
                theme::ACCENT,
            );
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("PLAYFORGE")
                        .font(theme::mono(11.0))
                        .color(theme::TEXT),
                );
                ui.label(
                    RichText::new("// FILES")
                        .font(theme::mono(8.0))
                        .color(theme::ACCENT),
                );
            });
        });
    }

    fn toolbar(&mut self, ctx: &egui::Context) {
        let crumbs = breadcrumbs(&self.cwd);
        let (mut back, mut forward, mut up, mut reload) = (false, false, false, false);
        let mut target = None;
        let mut new_folder = false;
        let mut toggle_hidden = false;
        let mut changed_filter = false;

        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::new()
                    .fill(theme::INK)
                    .stroke(Stroke::new(1.0_f32, theme::HAIRLINE))
                    .inner_margin(Margin {
                        left: 14,
                        right: 14,
                        top: 9,
                        bottom: 9,
                    }),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if icon_button(ui, Icon::ArrowLeft, "Zurück", self.hist_pos > 0).clicked() {
                        back = true;
                    }
                    if icon_button(
                        ui,
                        Icon::ArrowRight,
                        "Vorwärts",
                        self.hist_pos + 1 < self.history.len(),
                    )
                    .clicked()
                    {
                        forward = true;
                    }
                    if icon_button(
                        ui,
                        Icon::ArrowUp,
                        "Übergeordneter Ordner",
                        self.cwd.parent().is_some(),
                    )
                    .clicked()
                    {
                        up = true;
                    }
                    if icon_button(ui, Icon::Refresh, "Neu einlesen (F5)", true).clicked() {
                        reload = true;
                    }

                    ui.add_space(8.0);
                    // Feste Breite für die Pfadleiste: die Schaltflächen rechts
                    // (Filter, versteckte Dateien, neuer Ordner) behalten so
                    // ihren Platz, statt von der Leiste verdrängt zu werden.
                    let crumb_width = (ui.available_width() - 300.0).max(120.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(crumb_width, 30.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            egui::ScrollArea::horizontal()
                                .id_salt("breadcrumbs")
                                .max_height(28.0)
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 2.0;
                                        for (index, (label, path)) in crumbs.iter().enumerate() {
                                            let last = index + 1 == crumbs.len();
                                            let button = egui::Button::new(
                                                RichText::new(label).font(theme::sans(12.0)).color(
                                                    if last { theme::TEXT } else { theme::MUTED },
                                                ),
                                            )
                                            .frame(false);
                                            if ui.add(button).clicked() {
                                                target = Some(path.clone());
                                            }
                                            if !last {
                                                ui.label(
                                                    RichText::new("/")
                                                        .font(theme::mono(10.0))
                                                        .color(theme::FAINT),
                                                );
                                            }
                                        }
                                    });
                                });
                        },
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if icon_button(ui, Icon::Plus, "Neuer Ordner", true).clicked() {
                            new_folder = true;
                        }
                        let hidden_icon = if self.show_hidden {
                            Icon::Eye
                        } else {
                            Icon::EyeOff
                        };
                        if icon_button(ui, hidden_icon, "Versteckte Dateien (Strg+H)", true)
                            .clicked()
                        {
                            toggle_hidden = true;
                        }
                        let search = ui.add(
                            egui::TextEdit::singleline(&mut self.query)
                                .hint_text("Filtern …")
                                .desired_width(170.0),
                        );
                        if self.focus_search {
                            search.request_focus();
                            self.focus_search = false;
                        }
                        if search.changed() {
                            changed_filter = true;
                        }
                    });
                });
            });

        if let Some(path) = target {
            self.navigate_to(path);
        }
        if back {
            self.back();
        }
        if forward {
            self.forward();
        }
        if up {
            self.up();
        }
        if reload {
            self.refresh();
        }
        if toggle_hidden {
            self.show_hidden = !self.show_hidden;
            self.apply_view();
        }
        if changed_filter {
            self.apply_view();
        }
        if new_folder {
            self.start_new_folder();
        }
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        let directories = self.entries.iter().filter(|entry| entry.is_dir).count();
        let files = self.entries.len() - directories;
        let (color, message) = if let Some(error) = &self.error {
            (theme::ERROR, error.clone())
        } else if let Some((ok, message)) = &self.status {
            (if *ok { theme::OK } else { theme::ERROR }, message.clone())
        } else {
            (
                theme::ACCENT,
                format!(
                    "{} Einträge · {} Ordner · {} Dateien",
                    self.entries.len(),
                    directories,
                    files
                ),
            )
        };

        egui::TopBottomPanel::bottom("status")
            .exact_height(30.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::INK)
                    .stroke(Stroke::new(1.0_f32, theme::HAIRLINE))
                    .inner_margin(Margin::symmetric(14, 5)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(7.0), Sense::hover());
                    ui.painter().rect_filled(rect, CornerRadius::ZERO, color);
                    ui.label(
                        RichText::new(message)
                            .font(theme::mono(9.5))
                            .color(theme::MUTED),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let filtered = self.visible.len() != self.entries.len();
                        let count = if filtered {
                            format!("{} VON {} SICHTBAR", self.visible.len(), self.entries.len())
                        } else {
                            format!("{} ELEMENTE", self.entries.len())
                        };
                        ui.label(
                            RichText::new(count)
                                .font(theme::mono(9.0))
                                .color(theme::FAINT),
                        );
                    });
                });
            });
    }

    fn list_header(&mut self, ui: &mut egui::Ui) {
        let mut next_sort = None;
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            if sort_label(ui, "NAME", self.sort == SortKey::Name, self.ascending).clicked() {
                next_sort = Some(SortKey::Name);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if sort_label(
                    ui,
                    "GEÄNDERT",
                    self.sort == SortKey::Modified,
                    self.ascending,
                )
                .clicked()
                {
                    next_sort = Some(SortKey::Modified);
                }
                ui.add_space(14.0);
                if sort_label(ui, "GRÖSSE", self.sort == SortKey::Size, self.ascending).clicked() {
                    next_sort = Some(SortKey::Size);
                }
            });
        });

        if let Some(key) = next_sort {
            if self.sort == key {
                self.ascending = !self.ascending;
            } else {
                self.sort = key;
                self.ascending = true;
            }
            self.apply_view();
        }
    }

    fn central(&mut self, ctx: &egui::Context) {
        let mut command: Option<Command> = None;

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::INK).inner_margin(Margin {
                left: 18,
                right: 14,
                top: 12,
                bottom: 10,
            }))
            .show(ctx, |ui| {
                self.list_header(ui);
                ui.add_space(4.0);

                if let Some(error) = &self.error {
                    ui.add_space(18.0);
                    ui.label(
                        RichText::new("Ordner nicht lesbar")
                            .font(theme::display(18.0))
                            .color(theme::TEXT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(error)
                            .font(theme::sans(12.0))
                            .color(theme::MUTED),
                    );
                    return;
                }

                if self.visible.is_empty() {
                    ui.add_space(24.0);
                    let message = if self.query.trim().is_empty() {
                        if self.entries.is_empty() {
                            "Dieser Ordner ist leer.".to_string()
                        } else {
                            "Nur versteckte Dateien – mit Strg+H einblenden.".to_string()
                        }
                    } else {
                        format!("Keine Treffer für „{}“.", self.query.trim())
                    };
                    ui.label(
                        RichText::new(message)
                            .font(theme::sans(12.5))
                            .color(theme::MUTED),
                    );
                    return;
                }

                egui::ScrollArea::vertical()
                    .id_salt("file-list")
                    .auto_shrink([false, false])
                    .show_rows(ui, ROW_HEIGHT, self.visible.len(), |ui, range| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for index in range {
                            let entry = &self.entries[self.visible[index]];
                            let selected = self.selected.as_deref() == Some(entry.path.as_path());
                            let response = draw_row(ui, entry, selected);

                            if response.clicked || response.secondary {
                                command = Some(Command::Select(entry.path.clone()));
                            }
                            if response.double {
                                command = Some(Command::Open(entry.path.clone(), entry.is_dir));
                            }
                            if response.command.is_some() {
                                command = response.command;
                            }
                        }
                    });
            });

        if let Some(command) = command {
            self.apply_command(ctx, command);
        }
    }

    fn preview_panel(&mut self, ctx: &egui::Context) {
        let entry = self.selected_entry().cloned();
        let mut command = None;

        egui::SidePanel::right("preview")
            .resizable(true)
            .default_width(320.0)
            .width_range(260.0..=460.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::INK)
                    .stroke(Stroke::new(1.0_f32, theme::HAIRLINE))
                    .inner_margin(Margin::symmetric(14, 16)),
            )
            .show(ctx, |ui| {
                section_label(ui, "AUSWAHL");
                ui.add_space(8.0);

                let Some(entry) = &entry else {
                    ui.label(
                        RichText::new("Keine Datei ausgewählt.")
                            .font(theme::sans(12.0))
                            .color(theme::MUTED),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(
                            "Doppelklick öffnet einen Ordner oder eine Datei. Rechtsklick zeigt weitere Aktionen.",
                        )
                        .font(theme::sans(11.0))
                        .color(theme::FAINT),
                    );
                    return;
                };

                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
                    ui.painter()
                        .rect_filled(rect, CornerRadius::ZERO, theme::RAISED);
                    ui.painter().rect_stroke(
                        rect,
                        CornerRadius::ZERO,
                        Stroke::new(1.0_f32, theme::HAIRLINE_STRONG),
                        egui::StrokeKind::Inside,
                    );
                    icons::paint(
                        ui.painter(),
                        Rect::from_center_size(rect.center(), Vec2::splat(22.0)),
                        icons::for_entry(entry),
                        color_for(entry),
                    );
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(&entry.name)
                                    .font(theme::sans(13.0))
                                    .color(theme::TEXT),
                            )
                            .truncate(),
                        );
                        ui.label(
                            RichText::new(entry.kind.label())
                                .font(theme::mono(9.0))
                                .color(theme::FAINT),
                        );
                    });
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(6.0);

                field_row(
                    ui,
                    "Größe",
                    if entry.is_dir {
                        "—".to_string()
                    } else {
                        fs::format_size(entry.size)
                    },
                );
                field_row(ui, "Geändert", fs::format_time(entry.modified));
                if let Some(mode) = entry.mode {
                    field_row(ui, "Rechte", format!("{mode:o}"));
                }
                field_row(
                    ui,
                    "Typ",
                    if entry.is_symlink {
                        "Verknüpfung".to_string()
                    } else {
                        entry.kind.label().to_string()
                    },
                );

                ui.add_space(8.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(entry.path.display().to_string())
                            .font(theme::mono(9.0))
                            .color(theme::FAINT),
                    )
                    .selectable(true),
                );

                ui.add_space(12.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Öffnen").clicked() {
                        command = Some(Command::Open(entry.path.clone(), entry.is_dir));
                    }
                    if ui.button("Umbenennen").clicked() {
                        command = Some(Command::Rename(entry.path.clone()));
                    }
                    if ui.button("Löschen").clicked() {
                        command = Some(Command::Delete(entry.path.clone(), entry.is_dir));
                    }
                    if ui.button("Pfad kopieren").clicked() {
                        command = Some(Command::Copy(entry.path.clone()));
                    }
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                section_label(ui, "VORSCHAU");
                ui.add_space(6.0);

                egui::ScrollArea::vertical()
                    .id_salt("preview-body")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        match self
                            .preview
                            .as_ref()
                            .filter(|preview| preview.path == entry.path)
                            .map(|preview| &preview.kind)
                        {
                            Some(PreviewKind::Directory) => {
                                if let Ok(read) = std::fs::read_dir(&entry.path) {
                                    let count = read.flatten().count();
                                    ui.label(
                                        RichText::new(format!("Ordner mit {count} Einträgen"))
                                            .font(theme::sans(11.5))
                                            .color(theme::MUTED),
                                    );
                                }
                            }
                            Some(PreviewKind::Text { content, truncated }) => {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(content)
                                            .font(theme::mono(10.0))
                                            .color(theme::TEXT),
                                    )
                                    .selectable(true)
                                    .wrap(),
                                );
                                if *truncated {
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new("Vorschau nach 96 KB gekürzt.")
                                            .font(theme::mono(9.0))
                                            .color(theme::FAINT),
                                    );
                                }
                            }
                            Some(PreviewKind::Image { texture, size }) => {
                                ui.add(egui::Image::new(egui::load::SizedTexture::new(
                                    texture.id(),
                                    *size,
                                )));
                            }
                            Some(PreviewKind::Unavailable) => {
                                ui.label(
                                    RichText::new("Für diesen Dateityp gibt es keine Vorschau.")
                                        .font(theme::sans(11.0))
                                        .color(theme::MUTED),
                                );
                            }
                            None => {
                                ui.spinner();
                            }
                        }
                    });
            });

        if let Some(command) = command {
            self.apply_command(ctx, command);
        }
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.dialog.take() else {
            return;
        };
        let mut apply = false;
        let mut cancel = false;

        let window = egui::Window::new(match dialog {
            Dialog::Rename { .. } => "Umbenennen",
            Dialog::Delete { .. } => "Löschen",
            Dialog::NewFolder => "Neuer Ordner",
        })
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO);

        window.show(ctx, |ui| {
            ui.set_min_width(340.0);

            match &dialog {
                Dialog::Rename { target } => {
                    ui.label(
                        RichText::new(format!("Neuer Name für „{}“", file_name(target)))
                            .font(theme::sans(12.0))
                            .color(theme::MUTED),
                    );
                    ui.add_space(6.0);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.input).desired_width(f32::INFINITY),
                    );
                    if self.focus_dialog {
                        response.request_focus();
                        self.focus_dialog = false;
                    }
                    if response.lost_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    {
                        apply = true;
                    }
                }
                Dialog::Delete { target, is_dir } => {
                    ui.label(
                        RichText::new(format!(
                            "„{}“ {} wird in den Papierkorb verschoben.",
                            file_name(target),
                            if *is_dir { "und ihr Inhalt" } else { "" }
                        ))
                        .font(theme::sans(12.0))
                        .color(theme::TEXT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Ohne gio oder trash-put wird die Datei endgültig gelöscht.")
                            .font(theme::sans(11.0))
                            .color(theme::FAINT),
                    );
                }
                Dialog::NewFolder => {
                    ui.label(
                        RichText::new("Name des neuen Ordners")
                            .font(theme::sans(12.0))
                            .color(theme::MUTED),
                    );
                    ui.add_space(6.0);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.input).desired_width(f32::INFINITY),
                    );
                    if self.focus_dialog {
                        response.request_focus();
                        self.focus_dialog = false;
                    }
                    if response.lost_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    {
                        apply = true;
                    }
                }
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let confirm = match &dialog {
                    Dialog::Rename { .. } => "Umbenennen",
                    Dialog::Delete { .. } => "Löschen",
                    Dialog::NewFolder => "Anlegen",
                };
                if ui.button(confirm).clicked() {
                    apply = true;
                }
                if ui.button("Abbrechen").clicked() {
                    cancel = true;
                }
            });

            if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                cancel = true;
            }
        });

        if cancel {
            self.dialog = None;
            return;
        }

        // `true` hält den Dialog offen, `false` schließt ihn. Die Aktionen
        // melden Erfolg; bei einem Fehler bleibt der Dialog zum Korrigieren
        // stehen.
        let keep = if apply {
            match &dialog {
                Dialog::Rename { target } => !self.perform_rename(target),
                Dialog::Delete { target, .. } => {
                    self.perform_delete(target);
                    false
                }
                Dialog::NewFolder => !self.perform_new_folder(),
            }
        } else {
            true
        };

        if keep {
            self.dialog = Some(dialog);
        }
    }

    fn perform_rename(&mut self, target: &Path) -> bool {
        let name = self.input.trim().to_string();
        if !valid_name(&name) {
            self.status = Some((false, "Ungültiger Dateiname.".to_string()));
            return false;
        }
        let Some(parent) = target.parent() else {
            self.status = Some((false, "Zielpfad hat keinen Ordner.".to_string()));
            return false;
        };
        let destination = parent.join(&name);
        if destination == target {
            return true;
        }
        if destination.exists() {
            self.status = Some((false, format!("„{name}“ existiert bereits.")));
            return false;
        }
        match std::fs::rename(target, &destination) {
            Ok(()) => {
                self.status = Some((true, format!("Umbenannt in „{name}“.")));
                self.selected = Some(destination);
                self.load();
                true
            }
            Err(err) => {
                self.status = Some((false, format!("Umbenennen fehlgeschlagen: {err}")));
                false
            }
        }
    }

    fn perform_delete(&mut self, target: &Path) {
        let name = file_name(target);
        match fs::trash(target) {
            Ok(()) => {
                if self.selected.as_deref() == Some(target) {
                    self.selected = None;
                }
                self.status = Some((true, format!("„{name}“ gelöscht.")));
                self.load();
            }
            Err(err) => {
                self.status = Some((false, format!("Löschen fehlgeschlagen: {err}")));
            }
        }
    }

    fn perform_new_folder(&mut self) -> bool {
        let name = self.input.trim().to_string();
        if !valid_name(&name) {
            self.status = Some((false, "Ungültiger Ordnername.".to_string()));
            return false;
        }
        let path = self.cwd.join(&name);
        if path.exists() {
            self.status = Some((false, format!("„{name}“ existiert bereits.")));
            return false;
        }
        match std::fs::create_dir(&path) {
            Ok(()) => {
                self.status = Some((true, format!("Ordner „{name}“ angelegt.")));
                self.selected = Some(path);
                self.load();
                true
            }
            Err(err) => {
                self.status = Some((false, format!("Ordner nicht angelegt: {err}")));
                false
            }
        }
    }
}

impl eframe::App for FilesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keys(ctx);
        self.sync_preview(ctx);

        self.sidebar(ctx);
        self.toolbar(ctx);
        self.status_bar(ctx);
        self.preview_panel(ctx);
        self.central(ctx);
        self.dialogs(ctx);
    }
}

// ---------------------------------------------------------------------
// Zeichnen
// ---------------------------------------------------------------------

fn build_preview(ctx: &egui::Context, entry: &Entry) -> PreviewKind {
    if entry.is_dir {
        return PreviewKind::Directory;
    }
    match entry.kind {
        fs::Kind::Text | fs::Kind::Code => match fs::text_preview(&entry.path) {
            Some(preview) => PreviewKind::Text {
                content: preview.content,
                truncated: preview.truncated,
            },
            None => PreviewKind::Unavailable,
        },
        fs::Kind::Image => match fs::decode_image(&entry.path) {
            Some(image) => {
                let thumbnail = image.thumbnail(1400, 1400);
                let rgba = thumbnail.to_rgba8();
                let (width, height) = rgba.dimensions();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    rgba.as_raw(),
                );
                let texture = ctx.load_texture(
                    format!("preview-{}", entry.path.display()),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                let scale = (280.0 / width.max(1) as f32)
                    .min(320.0 / height.max(1) as f32)
                    .min(1.0);
                PreviewKind::Image {
                    texture,
                    size: Vec2::new(width as f32 * scale, height as f32 * scale),
                }
            }
            None => PreviewKind::Unavailable,
        },
        _ => PreviewKind::Unavailable,
    }
}

fn draw_row(ui: &mut egui::Ui, entry: &Entry, selected: bool) -> RowResponse {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), Sense::click());
    let painter = ui.painter();

    if selected {
        painter.rect_filled(rect, CornerRadius::ZERO, theme::alpha(theme::ACCENT, 26));
        let bar = Rect::from_min_size(
            rect.left_top() + Vec2::new(0.0, 0.0),
            Vec2::new(3.0, ROW_HEIGHT),
        );
        painter.rect_filled(bar, CornerRadius::ZERO, theme::ACCENT);
    } else if response.hovered() {
        painter.rect_filled(rect, CornerRadius::ZERO, theme::alpha(theme::TEXT, 8));
    }

    icons::paint(
        painter,
        Rect::from_min_size(
            rect.left_center() + Vec2::new(10.0, -8.0),
            Vec2::splat(16.0),
        ),
        icons::for_entry(entry),
        color_for(entry),
    );

    let date_right = rect.right() - 10.0;
    let size_right = rect.right() - 150.0;
    let name_right = size_right - 78.0;
    let name_rect = Rect::from_min_max(
        rect.left_top() + Vec2::new(34.0, 0.0),
        egui::pos2(name_right, rect.bottom()),
    );

    painter.with_clip_rect(name_rect).text(
        name_rect.left_center(),
        Align2::LEFT_CENTER,
        &entry.name,
        theme::sans(12.0),
        if entry.hidden {
            theme::MUTED
        } else {
            theme::TEXT
        },
    );

    painter.text(
        egui::pos2(size_right, rect.center().y),
        Align2::RIGHT_CENTER,
        if entry.is_dir {
            "—".to_string()
        } else {
            fs::format_size(entry.size)
        },
        theme::mono(10.0),
        theme::MUTED,
    );

    painter.text(
        egui::pos2(date_right, rect.center().y),
        Align2::RIGHT_CENTER,
        fs::format_time(entry.modified),
        theme::mono(9.5),
        theme::FAINT,
    );

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let mut command = None;
    response.context_menu(|ui| {
        if ui.button("Öffnen").clicked() {
            command = Some(Command::Open(entry.path.clone(), entry.is_dir));
            ui.close_menu();
        }
        if ui.button("Umbenennen").clicked() {
            command = Some(Command::Rename(entry.path.clone()));
            ui.close_menu();
        }
        if ui.button("Löschen").clicked() {
            command = Some(Command::Delete(entry.path.clone(), entry.is_dir));
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Pfad kopieren").clicked() {
            command = Some(Command::Copy(entry.path.clone()));
            ui.close_menu();
        }
    });

    RowResponse {
        clicked: response.clicked(),
        double: response.double_clicked(),
        secondary: response.secondary_clicked(),
        command,
    }
}

fn icon_button(ui: &mut egui::Ui, icon: Icon, tooltip: &str, enabled: bool) -> egui::Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(30.0), sense);
    let hovered = enabled && response.hovered();

    if hovered {
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, theme::RAISED);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let color = if !enabled {
        theme::FAINT
    } else if hovered {
        theme::TEXT
    } else {
        theme::MUTED
    };
    icons::paint(
        ui.painter(),
        Rect::from_center_size(rect.center(), Vec2::splat(16.0)),
        icon,
        color,
    );

    response.on_hover_text(tooltip)
}

fn place_row(ui: &mut egui::Ui, label: &str, icon: Icon, active: bool) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
    let painter = ui.painter();

    if active {
        painter.rect_filled(rect, CornerRadius::ZERO, theme::alpha(theme::ACCENT, 26));
        let bar = Rect::from_min_size(rect.left_top() + Vec2::new(0.0, 0.0), Vec2::new(3.0, 30.0));
        painter.rect_filled(bar, CornerRadius::ZERO, theme::ACCENT);
    } else if response.hovered() {
        painter.rect_filled(rect, CornerRadius::ZERO, theme::alpha(theme::TEXT, 8));
    }

    icons::paint(
        painter,
        Rect::from_min_size(
            rect.left_center() + Vec2::new(12.0, -8.0),
            Vec2::splat(16.0),
        ),
        icon,
        if active { theme::ACCENT } else { theme::MUTED },
    );

    let label_rect = Rect::from_min_max(
        rect.left_top() + Vec2::new(36.0, 0.0),
        rect.right_bottom() - Vec2::new(4.0, 0.0),
    );
    painter.with_clip_rect(label_rect).text(
        label_rect.left_center(),
        Align2::LEFT_CENTER,
        label,
        theme::sans(12.0),
        if active { theme::TEXT } else { theme::MUTED },
    );

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    response
}

fn field_row(ui: &mut egui::Ui, label: &str, value: String) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label.to_uppercase())
                .font(theme::mono(8.5))
                .color(theme::FAINT),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(value)
                        .font(theme::sans(11.5))
                        .color(theme::TEXT),
                )
                .truncate(),
            );
        });
    });
}

fn sort_label(ui: &mut egui::Ui, label: &str, active: bool, ascending: bool) -> egui::Response {
    let text = if active {
        format!("{label} {}", if ascending { "↑" } else { "↓" })
    } else {
        label.to_string()
    };
    ui.add(
        egui::Button::new(RichText::new(text).font(theme::mono(9.0)).color(if active {
            theme::TEXT
        } else {
            theme::FAINT
        }))
        .frame(false),
    )
}

fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .font(theme::mono(8.5))
            .color(theme::FAINT),
    );
}

fn breadcrumbs(path: &Path) -> Vec<(String, PathBuf)> {
    let mut crumbs = Vec::new();
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        let label = match component {
            Component::RootDir => "/".to_string(),
            Component::Prefix(prefix) => prefix.as_os_str().to_string_lossy().into_owned(),
            Component::Normal(name) => name.to_string_lossy().into_owned(),
            Component::CurDir => ".".to_string(),
            Component::ParentDir => "..".to_string(),
        };
        crumbs.push((label, current.clone()));
    }
    crumbs
}

fn place_icon(kind: &PlaceKind) -> Icon {
    match kind {
        PlaceKind::Home => Icon::Home,
        PlaceKind::Music => Icon::Audio,
        PlaceKind::Root | PlaceKind::Disk => Icon::Disk,
        _ => Icon::Folder,
    }
}

fn color_for(entry: &Entry) -> Color32 {
    match entry.kind {
        fs::Kind::Folder => theme::ACCENT,
        fs::Kind::Image => theme::INFO,
        fs::Kind::Video => theme::INFO,
        fs::Kind::Audio => theme::OK,
        fs::Kind::Archive => theme::WARN,
        fs::Kind::Code => theme::ACCENT,
        fs::Kind::Pdf => Color32::from_rgb(255, 138, 128),
        fs::Kind::Text | fs::Kind::Other => theme::MUTED,
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// Startordner: optional ein Pfad von der Kommandozeile (die Desktop-Datei
/// übergibt `%f`). Bei einer Datei öffnet der Browser deren Ordner.
fn start_location() -> (PathBuf, Option<PathBuf>) {
    if let Some(raw) = std::env::args_os().nth(1) {
        let path = PathBuf::from(raw);
        if path.is_file() {
            let parent = path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| path.clone());
            return (parent, Some(path));
        }
        if path.is_dir() {
            return (path, None);
        }
    }

    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("/"));
    (home, None)
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
}
