//! Bausteine des Designsytems.
//!
//! Alles Sichtbare entsteht hier: Glas-Panels mit Ring, Bento-Raster,
//! segmentierte Messbalken, Datentabellen, Timeline und Code-Blöcke.

pub mod diagnose;
pub mod gaming;
pub mod graphics;
pub mod logs;
pub mod overview;
pub mod performance;
pub mod services;
pub mod tuning;
pub mod updates;

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, Layout, Margin, Pos2, Rect, Response, RichText,
    Sense, Stroke, StrokeKind, Ui, Vec2,
};

use crate::app::ControlCenter;
use crate::icons::{self, Icon};
use crate::theme;

/// Abstand zwischen Bento-Zellen.
pub const GAP: f32 = 10.0;

// ---------------------------------------------------------------- Text

/// Technische Beschriftung in Monospace.
pub fn mono_label(ui: &mut Ui, text: &str, color: Color32, size: f32) -> Response {
    ui.label(RichText::new(text).font(theme::mono(size)).color(color))
}

/// Kapitälchen-Zeile in Monospace, wie ein Feldname in einem Terminal.
pub fn kicker(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .font(theme::mono(9.0))
            .color(theme::FAINT),
    );
}

pub fn hint(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .font(theme::sans(12.0))
            .color(theme::MUTED),
    );
}

// ---------------------------------------------------------------- Karten

/// Glas-Panel mit Haarlinie und äußerem Ring.
fn glass() -> egui::Frame {
    egui::Frame::new()
        .fill(theme::PANEL)
        .stroke(Stroke::new(1.0_f32, theme::HAIRLINE))
        .corner_radius(CornerRadius::same(theme::CARD_RADIUS))
        .inner_margin(Margin::symmetric(16, 14))
}

fn ring(ui: &Ui, rect: Rect) {
    ui.painter().rect_stroke(
        rect.expand(2.0),
        CornerRadius::same(theme::CARD_RADIUS + 2),
        Stroke::new(1.0_f32, theme::RING),
        StrokeKind::Middle,
    );
}

pub fn panel(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    let inner = glass().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        add(ui);
    });
    ring(ui, inner.response.rect);
}

/// Panel mit Titelzeile: Akzent-Marke, Titel, optionale Monospace-Notiz.
pub fn card(ui: &mut Ui, title: &str, add: impl FnOnce(&mut Ui)) {
    panel_titled(ui, title, "", add);
}

pub fn panel_titled(ui: &mut Ui, title: &str, meta: &str, add: impl FnOnce(&mut Ui)) {
    panel(ui, |ui| {
        card_header(ui, title, meta);
        add(ui);
    });
}

pub fn card_header(ui: &mut Ui, title: &str, meta: &str) {
    ui.horizontal(|ui| {
        let (mark, _) = ui.allocate_exact_size(Vec2::new(5.0, 12.0), Sense::hover());
        ui.painter()
            .rect_filled(mark, CornerRadius::same(1), theme::ACCENT);
        ui.add_space(2.0);
        ui.label(
            RichText::new(title)
                .font(theme::sans(13.0))
                .color(theme::TEXT),
        );
        if !meta.is_empty() {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(meta.to_uppercase())
                        .font(theme::mono(9.0))
                        .color(theme::FAINT),
                );
            });
        }
    });
    ui.add_space(9.0);
}

// ---------------------------------------------------------------- Raster

/// Eine Rasterzelle fester Breite; die Höhe ergibt sich aus dem Inhalt.
pub fn cell(ui: &mut Ui, width: f32, add: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(Vec2::new(width, 0.0), Layout::top_down(Align::Min), |ui| {
        ui.set_width(width);
        add(ui);
    });
}

/// Bento-Raster: teilt die Breite im Verhältnis der Gewichte auf.
pub fn grid(ui: &mut Ui, weights: &[f32], mut add: impl FnMut(&mut Ui, usize)) {
    let widths = spread(ui.available_width(), weights);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = GAP;
        for (index, width) in widths.iter().enumerate() {
            cell(ui, *width, |ui| add(ui, index));
        }
    });
}

fn spread(total: f32, weights: &[f32]) -> Vec<f32> {
    let gaps = GAP * weights.len().saturating_sub(1) as f32;
    let usable = (total - gaps).max(80.0);
    let sum: f32 = weights.iter().sum::<f32>().max(0.001);
    weights
        .iter()
        .map(|weight| (usable * weight / sum).max(60.0))
        .collect()
}

// ---------------------------------------------------------------- Kennzahlen

/// Kennzahlenkachel: Monospace-Label, Zahl in Inter Display, Fußnote.
pub fn metric(ui: &mut Ui, label: &str, value: &str, sub: &str, accent: Color32) {
    panel(ui, |ui| {
        ui.set_min_height(72.0);
        ui.horizontal(|ui| {
            let (mark, _) = ui.allocate_exact_size(Vec2::splat(5.0), Sense::hover());
            ui.painter()
                .rect_filled(mark, CornerRadius::same(1), accent);
            ui.add_space(2.0);
            ui.label(
                RichText::new(label.to_uppercase())
                    .font(theme::mono(9.0))
                    .color(theme::MUTED),
            );
        });
        ui.add_space(9.0);
        ui.label(
            RichText::new(value)
                .font(theme::display(22.0))
                .color(theme::TEXT),
        );
        ui.add_space(1.0);
        ui.add(
            egui::Label::new(
                RichText::new(sub)
                    .font(theme::mono(9.5))
                    .color(theme::FAINT),
            )
            .wrap(),
        );
    });
}

// ---------------------------------------------------------------- Zeilen

const LABEL_WIDTH: f32 = 158.0;

/// Schlüssel-Wert-Zeile: Monospace-Schlüssel, Sans-Wert.
pub fn kv(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal_top(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(LABEL_WIDTH, 18.0), Sense::hover());
        ui.painter().text(
            rect.left_center(),
            Align2::LEFT_CENTER,
            label,
            theme::mono(10.0),
            theme::FAINT,
        );
        ui.add(
            egui::Label::new(
                RichText::new(value)
                    .font(theme::sans(12.5))
                    .color(theme::TEXT),
            )
            .wrap(),
        );
    });
}

/// Zwei Schlüssel-Wert-Spalten nebeneinander.
pub fn kv_grid(ui: &mut Ui, rows: &[(&str, String)]) {
    let half = rows.len().div_ceil(2);
    ui.columns(2, |columns| {
        for (index, (label, value)) in rows.iter().enumerate() {
            let column = if index < half { 0 } else { 1 };
            kv(&mut columns[column], label, value);
        }
    });
}

// ---------------------------------------------------------------- Marken

/// Statusplakette in Monospace, eckig, mit Glasfüllung.
pub fn badge(ui: &mut Ui, text: &str, color: Color32) -> Response {
    let label = text.to_uppercase();
    let galley = ui.painter().layout_no_wrap(label, theme::mono(9.0), color);
    let padding = Vec2::new(7.0, 3.0);
    let size = galley.size() + padding * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    let radius = CornerRadius::same(4);
    painter.rect_filled(rect, radius, theme::alpha(color, 24));
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0_f32, theme::alpha(color, 90)),
        StrokeKind::Inside,
    );
    painter.galley(rect.min + padding, galley, color);
    response
}

/// Prüfpunkt als Quadrat + Monospace-Text.
///
/// Gezeichnet statt als Frame, damit `horizontal_wrapped` die Breite misst
/// und die Zeile korrekt umbricht.
pub fn chip(ui: &mut Ui, positive: bool, text: &str) {
    let color = if positive {
        theme::ACCENT
    } else {
        theme::FAINT
    };
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        theme::mono(10.0),
        if positive { theme::TEXT } else { theme::MUTED },
    );
    let padding = Vec2::new(9.0, 5.0);
    let glyph = 12.0;
    let size = Vec2::new(
        galley.size().x + glyph + padding.x * 2.0,
        galley.size().y + padding.y * 2.0,
    );
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    let radius = CornerRadius::same(6);
    painter.rect_filled(rect, radius, theme::GLASS);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0_f32, theme::HAIRLINE),
        StrokeKind::Inside,
    );

    let square = Rect::from_center_size(
        Pos2::new(rect.left() + padding.x + 3.0, rect.center().y),
        Vec2::splat(5.0),
    );
    if positive {
        painter.rect_filled(square, CornerRadius::same(1), color);
    } else {
        painter.rect_stroke(
            square,
            CornerRadius::same(1),
            Stroke::new(1.0_f32, color),
            StrokeKind::Inside,
        );
    }
    painter.galley(
        rect.left_top() + Vec2::new(padding.x + glyph, padding.y),
        galley,
        theme::TEXT,
    );
}

/// Auswahlfläche in Monospace: ausgewählt füllt Säure-Grün.
pub fn choice(ui: &mut Ui, selected: bool, label: &str) -> Response {
    let foreground = if selected { theme::INK } else { theme::MUTED };
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), theme::mono(10.0), foreground);
    let padding = Vec2::new(10.0, 5.0);
    let size = galley.size() + padding * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    let painter = ui.painter();
    let radius = CornerRadius::same(5);
    let fill = if selected {
        theme::ACCENT
    } else if response.hovered() {
        theme::RAISED
    } else {
        theme::GLASS
    };
    painter.rect_filled(rect, radius, fill);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(
            1.0_f32,
            if selected {
                theme::ACCENT
            } else {
                theme::HAIRLINE
            },
        ),
        StrokeKind::Inside,
    );
    painter.galley(rect.min + padding, galley, foreground);

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

// ---------------------------------------------------------------- Messbalken

/// Segmentierter Balken – technischer als ein durchgehender Verlauf.
pub fn meter(ui: &mut Ui, caption: &str, value: &str, fraction: f32, color: Color32) {
    ui.horizontal(|ui| {
        mono_label(ui, caption, theme::MUTED, 10.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            mono_label(ui, value, theme::FAINT, 10.0);
        });
    });
    ui.add_space(4.0);

    let width = ui.available_width();
    let height = 9.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    if rect.width() <= 1.0 {
        return;
    }

    let segments = 44.0_f32;
    let gap = 2.0;
    let segment_width = ((rect.width() - gap * (segments - 1.0)) / segments).max(1.0);
    if segment_width <= gap {
        return;
    }
    let filled = (fraction.clamp(0.0, 1.0) * segments).round();
    let painter = ui.painter();
    let mut x = rect.left();
    for index in 0..segments as usize {
        if x + segment_width > rect.right() + 0.5 {
            break;
        }
        let segment =
            Rect::from_min_size(Pos2::new(x, rect.top()), Vec2::new(segment_width, height));
        let fill = if index as f32 + 0.5 <= filled {
            color
        } else {
            theme::HAIRLINE
        };
        painter.rect_filled(segment, CornerRadius::ZERO, fill);
        x += segment_width + gap;
    }
}

// ---------------------------------------------------------------- Tabelle

/// Kopfzeile einer Datentabelle: Monospace-Kapitälchen und Haarlinie.
pub fn table_header(ui: &mut Ui, columns: &[(&str, f32)]) {
    let weights: Vec<f32> = columns.iter().map(|(_, weight)| *weight).collect();
    let widths = spread(ui.available_width(), &weights);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = GAP;
        for (index, (label, _)) in columns.iter().enumerate() {
            cell(ui, widths[index], |ui| {
                ui.label(
                    RichText::new(label.to_uppercase())
                        .font(theme::mono(9.0))
                        .color(theme::FAINT),
                );
            });
        }
    });
    ui.add_space(5.0);
    hairline(ui);
}

/// Datenzeile mit Spaltengewichten; `add` füllt die Zelle `index`.
pub fn table_row(ui: &mut Ui, weights: &[f32], mut add: impl FnMut(&mut Ui, usize)) {
    let widths = spread(ui.available_width(), weights);
    ui.add_space(7.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = GAP;
        for (index, width) in widths.iter().enumerate() {
            cell(ui, *width, |ui| add(ui, index));
        }
    });
    ui.add_space(7.0);
    hairline(ui);
}

// ---------------------------------------------------------------- Timeline

/// Eine Zeile der Timeline.
pub struct Entry {
    pub time: String,
    pub text: String,
    pub color: Color32,
    /// Monospace für Protokollzeilen, Sans für Prüfergebnisse.
    pub mono: bool,
}

impl Entry {
    pub fn new(time: impl Into<String>, text: impl Into<String>, color: Color32) -> Self {
        Self {
            time: time.into(),
            text: text.into(),
            color,
            mono: false,
        }
    }

    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }
}

/// Timeline mit durchgehender Schiene, Zeitstempel und farbigem Knoten.
pub fn timeline(ui: &mut Ui, id: &str, entries: &[Entry]) {
    if entries.is_empty() {
        hint(ui, "Keine Einträge.");
        return;
    }

    let mut first: Option<Pos2> = None;
    let mut last: Option<Pos2> = None;

    for entry in entries {
        let mut rail = Rect::NOTHING;
        let inner = ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 9.0;
            let (gutter, _) = ui.allocate_exact_size(Vec2::new(54.0, 15.0), Sense::hover());
            ui.painter().text(
                gutter.right_top(),
                Align2::RIGHT_TOP,
                &entry.time,
                theme::mono(9.0),
                theme::FAINT,
            );
            let (rect, _) = ui.allocate_exact_size(Vec2::new(14.0, 15.0), Sense::hover());
            rail = rect;
            let font = if entry.mono {
                theme::mono(10.5)
            } else {
                theme::sans(12.0)
            };
            ui.add(
                egui::Label::new(RichText::new(&entry.text).font(font).color(theme::TEXT)).wrap(),
            );
        });

        let rect = inner.response.rect;
        let rail_x = rail.center().x;
        let top = Pos2::new(rail_x, rect.top() + 5.0);
        let bottom = Pos2::new(rail_x, rect.bottom() + 6.0);
        first.get_or_insert(top);
        last = Some(bottom);

        ui.painter()
            .line_segment([top, bottom], Stroke::new(1.0_f32, theme::HAIRLINE));
        ui.painter().circle_filled(top, 2.6, entry.color);
        let _ = id;
    }

    if let (Some(top), Some(bottom)) = (first, last) {
        ui.painter()
            .line_segment([top, bottom], Stroke::new(1.0_f32, theme::HAIRLINE));
    }
}

// ---------------------------------------------------------------- Code

/// Code-Block mit Kopfzeile, Zeilennummern und Kopierknopf.
pub fn code_block(ui: &mut Ui, id: &str, label: &str, text: &str, max_height: f32) {
    egui::Frame::new()
        .fill(theme::WELL)
        .stroke(Stroke::new(1.0_f32, theme::HAIRLINE))
        .corner_radius(CornerRadius::same(theme::INNER_RADIUS))
        .inner_margin(Margin::ZERO)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            egui::Frame::new()
                .fill(theme::PANEL)
                .inner_margin(Margin::symmetric(11, 6))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(label.to_uppercase())
                                .font(theme::mono(9.0))
                                .color(theme::FAINT),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_button(ui, Icon::Copy, "", theme::ACCENT)
                                .on_hover_text("In die Zwischenablage kopieren")
                                .clicked()
                            {
                                ui.ctx().copy_text(text.to_string());
                            }
                        });
                    });
                });

            hairline(ui);

            egui::Frame::new()
                .inner_margin(Margin::symmetric(11, 8))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    egui::ScrollArea::both()
                        .id_salt(id)
                        .max_height(max_height)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                            if text.is_empty() {
                                mono_label(ui, "—", theme::FAINT, 10.5);
                                return;
                            }
                            for (index, line) in text.lines().enumerate() {
                                ui.horizontal_top(|ui| {
                                    let (gutter, _) = ui
                                        .allocate_exact_size(Vec2::new(22.0, 14.0), Sense::hover());
                                    ui.painter().text(
                                        gutter.right_top(),
                                        Align2::RIGHT_TOP,
                                        format!("{:>2}", index + 1),
                                        theme::mono(9.0),
                                        theme::FAINT,
                                    );
                                    ui.label(
                                        RichText::new(line)
                                            .font(theme::mono(10.5))
                                            .color(theme::MUTED),
                                    );
                                });
                            }
                        });
                });
        });
}

// ---------------------------------------------------------------- Knöpfe

/// Hauptaktion: Säure-Grün auf Schwarz.
pub fn primary(ui: &mut Ui, text: &str) -> Response {
    let enabled = ui.is_enabled();
    let (fill, foreground) = if enabled {
        (theme::ACCENT, theme::INK)
    } else {
        (theme::RAISED, theme::FAINT)
    };
    ui.add(
        egui::Button::new(
            RichText::new(text)
                .font(theme::sans(12.5))
                .color(foreground),
        )
        .fill(fill)
        .stroke(Stroke::NONE)
        .min_size(Vec2::new(0.0, 30.0)),
    )
}

/// Nebenaktion: erhöhte Fläche mit Haarlinie.
pub fn ghost(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        egui::Button::new(
            RichText::new(text)
                .font(theme::sans(12.5))
                .color(theme::TEXT),
        )
        .fill(theme::RAISED)
        .stroke(Stroke::new(1.0_f32, theme::HAIRLINE))
        .min_size(Vec2::new(0.0, 30.0)),
    )
}

/// Knopf mit Symbol und optionaler Beschriftung.
pub fn icon_button(ui: &mut Ui, icon: Icon, label: &str, accent: Color32) -> Response {
    let galley = if label.is_empty() {
        None
    } else {
        Some(
            ui.painter()
                .layout_no_wrap(label.to_string(), theme::sans(12.0), theme::TEXT),
        )
    };
    let icon_size = 15.0;
    let text_width = galley.as_ref().map(|g| g.size().x + 7.0).unwrap_or(0.0);
    let size = Vec2::new(icon_size + text_width + 18.0, 28.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hovered = response.hovered();
    let painter = ui.painter();

    painter.rect_filled(
        rect,
        CornerRadius::same(7),
        if hovered {
            theme::RAISED
        } else {
            Color32::TRANSPARENT
        },
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(7),
        Stroke::new(
            1.0_f32,
            if hovered {
                theme::alpha(accent, 120)
            } else {
                theme::HAIRLINE
            },
        ),
        StrokeKind::Inside,
    );

    let icon_rect = Rect::from_center_size(
        rect.left_center() + Vec2::new(9.0 + icon_size * 0.5, 0.0),
        Vec2::splat(icon_size),
    );
    icons::paint(painter, icon_rect, icon, accent);
    if let Some(galley) = galley {
        let position = icon_rect.right_center() + Vec2::new(7.0, -galley.size().y * 0.5);
        painter.galley(position, galley, theme::TEXT);
    }
    response
}

pub fn copy_button(ui: &mut Ui, text: &str, label: &str) {
    if icon_button(ui, Icon::Copy, label, theme::ACCENT).clicked() {
        ui.ctx().copy_text(text.to_string());
    }
}

// ---------------------------------------------------------------- Hinweise

pub fn notice(ui: &mut Ui, icon: Icon, color: Color32, text: &str) {
    egui::Frame::new()
        .fill(theme::alpha(color, 12))
        .stroke(Stroke::new(1.0_f32, theme::alpha(color, 56)))
        .corner_radius(CornerRadius::same(theme::INNER_RADIUS))
        .inner_margin(Margin::symmetric(13, 10))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_top(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                icons::paint(ui.painter(), rect, icon, color);
                ui.add_space(2.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(text)
                            .font(theme::sans(12.0))
                            .color(theme::TEXT),
                    )
                    .wrap(),
                );
            });
        });
}

/// Hinweis, wenn der privilegierte Helfer fehlt (z. B. außerhalb von Playforge).
pub fn helper_notice(cc: &ControlCenter, ui: &mut Ui) {
    if cc.helper_available {
        return;
    }
    notice(
        ui,
        Icon::Warn,
        theme::WARN,
        "Systemänderungen sind nur unter Playforge Linux möglich – der Helfer \
         /usr/local/lib/playforge/control-helper.sh ist nicht vorhanden. Anzeigen funktioniert weiterhin.",
    );
    ui.add_space(12.0);
}

// ---------------------------------------------------------------- Linien

pub fn hairline(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, theme::HAIRLINE);
}

pub fn divider(ui: &mut Ui) {
    hairline(ui);
}

// ---------------------------------------------------------------- Sonstiges

/// Kürzt Text auf `max` Zeichen (mit Auslassungszeichen).
pub fn truncate(text: &str, max: usize) -> String {
    let count = text.chars().count();
    if count <= max {
        text.to_string()
    } else {
        let cut: String = text.chars().take(max.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}

pub fn skeleton(ui: &mut Ui) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.spinner();
        ui.add_space(9.0);
        mono_label(ui, "SYSTEMDATEN WERDEN GELESEN …", theme::FAINT, 9.5);
    });
}
