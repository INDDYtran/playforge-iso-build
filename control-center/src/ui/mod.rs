//! Gemeinsame Bausteine der Oberfläche (Karten, Kennzahlen, Abzeichen).

pub mod diagnose;
pub mod gaming;
pub mod graphics;
pub mod logs;
pub mod overview;
pub mod performance;
pub mod services;

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Margin, Rect, Response, RichText, Sense, Stroke,
    StrokeKind, Vec2,
};

use crate::app::ControlCenter;
use crate::icons::{self, Icon};
use crate::theme;

/// Breite der Beschriftungsspalte in Schlüssel-Wert-Zeilen.
const LABEL_WIDTH: f32 = 150.0;

/// Karte mit Titelzeile – der Standardcontainer aller Seiten.
pub fn card(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(theme::SURFACE)
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(CornerRadius::same(theme::CARD_RADIUS))
        .inner_margin(Margin::symmetric(18, 16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            if !title.is_empty() {
                card_title(ui, title);
                ui.add_space(10.0);
            }
            add(ui);
        });
}

pub fn card_title(ui: &mut egui::Ui, title: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(3.0, 15.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, CornerRadius::same(2), theme::AMBER);
        ui.add_space(3.0);
        ui.label(RichText::new(title).size(15.0).strong().color(theme::TEXT));
    });
}

/// Schlüssel-Wert-Zeile mit fester Beschriftungsspalte.
pub fn kv(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal_top(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(LABEL_WIDTH, 19.0), Sense::hover());
        ui.painter().text(
            rect.left_center(),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(12.5),
            theme::MUTED,
        );
        ui.add(egui::Label::new(RichText::new(value).size(13.5).color(theme::TEXT)).wrap());
    });
}

/// Zwei Schlüssel-Wert-Spalten nebeneinander.
pub fn kv_grid(ui: &mut egui::Ui, rows: &[(&str, String)]) {
    let half = rows.len().div_ceil(2);
    ui.columns(2, |columns| {
        for (index, (label, value)) in rows.iter().enumerate() {
            let column = if index < half { 0 } else { 1 };
            kv(&mut columns[column], label, value);
        }
    });
}

pub fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(12.5).color(theme::MUTED));
}

/// Abzeichen in Pillenform.
pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) -> Response {
    let font = FontId::proportional(11.5);
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, color);
    let padding = Vec2::new(8.0, 3.0);
    let size = galley.size() + padding * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        CornerRadius::same((size.y * 0.5) as u8),
        color.gamma_multiply(0.16),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same((size.y * 0.5) as u8),
        Stroke::new(1.0_f32, color.gamma_multiply(0.45)),
        StrokeKind::Inside,
    );
    painter.galley(rect.min + padding, galley, color);
    response
}

/// Punkt + Text, wie eine Statusanzeige.
///
/// Bewusst gezeichnet statt als `Frame` mit Layout: Nur so misst
/// `horizontal_wrapped` die Breite korrekt und bricht die Zeile um.
pub fn chip(ui: &mut egui::Ui, positive: bool, text: &str) {
    let color = if positive { theme::GREEN } else { theme::AMBER };
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_string(), FontId::proportional(12.0), theme::TEXT);
    let padding = Vec2::new(11.0, 5.0);
    let dot_space = 13.0;
    let size = Vec2::new(
        galley.size().x + dot_space + padding.x * 2.0,
        galley.size().y + padding.y * 2.0,
    );
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    let radius = CornerRadius::same((size.y * 0.5) as u8);
    painter.rect_filled(rect, radius, color.gamma_multiply(0.12));
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0_f32, color.gamma_multiply(0.35)),
        StrokeKind::Inside,
    );
    let dot = rect.left_center() + Vec2::new(padding.x + 3.5, 0.0);
    painter.circle_filled(dot, 3.0, color);
    painter.galley(
        rect.left_top() + Vec2::new(padding.x + dot_space, padding.y),
        galley,
        theme::TEXT,
    );
}

/// Fortschrittsbalken mit Beschriftung.
pub fn meter(ui: &mut egui::Ui, caption: &str, value: &str, fraction: f32, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(caption).size(12.0).color(theme::MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(value).size(12.0).color(theme::MUTED));
        });
    });
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 7.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(4), theme::SURFACE_HI);
    let filled_width = (rect.width() * fraction.clamp(0.0, 1.0)).max(2.0);
    painter.rect_filled(
        Rect::from_min_size(rect.min, Vec2::new(filled_width, rect.height())),
        CornerRadius::same(4),
        color,
    );
}

/// Kennzahlenkachel für die Übersicht.
pub fn tile(
    ui: &mut egui::Ui,
    width: f32,
    icon: Icon,
    label: &str,
    value: &str,
    sub: &str,
    accent: Color32,
) {
    let height = 94.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(theme::CARD_RADIUS), theme::SURFACE);
    painter.rect_stroke(
        rect,
        CornerRadius::same(theme::CARD_RADIUS),
        Stroke::new(1.0_f32, theme::BORDER),
        StrokeKind::Inside,
    );

    let strip = Rect::from_min_size(
        rect.min + Vec2::new(0.0, 16.0),
        Vec2::new(3.0, height - 32.0),
    );
    painter.rect_filled(strip, CornerRadius::same(2), accent);

    let icon_rect = Rect::from_min_size(rect.min + Vec2::new(18.0, 17.0), Vec2::splat(17.0));
    icons::paint(painter, icon_rect, icon, accent);
    painter.text(
        icon_rect.right_top() + Vec2::new(2.0, 0.0),
        Align2::LEFT_TOP,
        label,
        FontId::proportional(12.0),
        theme::MUTED,
    );
    painter.text(
        rect.min + Vec2::new(18.0, 50.0),
        Align2::LEFT_CENTER,
        value,
        FontId::proportional(19.0),
        theme::TEXT,
    );
    painter.text(
        rect.min + Vec2::new(18.0, 74.0),
        Align2::LEFT_CENTER,
        sub,
        FontId::proportional(11.5),
        theme::FAINT,
    );
}

/// Hinweisbox mit Symbol.
pub fn notice(ui: &mut egui::Ui, icon: Icon, color: Color32, text: &str) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.10))
        .stroke(Stroke::new(1.0_f32, color.gamma_multiply(0.35)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 11))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_top(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(17.0), Sense::hover());
                icons::paint(ui.painter(), rect, icon, color);
                ui.add_space(2.0);
                ui.add(egui::Label::new(RichText::new(text).size(12.5).color(theme::TEXT)).wrap());
            });
        });
}

/// Hinweis, wenn der privilegierte Helfer fehlt (z. B. außerhalb von Playforge).
pub fn helper_notice(cc: &ControlCenter, ui: &mut egui::Ui) {
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

/// Kräftiger Hauptknopf.
pub fn primary(ui: &mut egui::Ui, text: &str) -> Response {
    let enabled = ui.is_enabled();
    let fill = if enabled {
        theme::AMBER
    } else {
        theme::SURFACE_HI
    };
    let text_color = if enabled {
        Color32::from_rgb(6, 14, 24)
    } else {
        theme::FAINT
    };
    ui.add(
        egui::Button::new(RichText::new(text).color(text_color).strong())
            .fill(fill)
            .stroke(Stroke::NONE)
            .min_size(Vec2::new(0.0, 30.0)),
    )
}

/// Zurückhaltender Knopf.
pub fn ghost(ui: &mut egui::Ui, text: &str) -> Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(13.0).color(theme::TEXT))
            .fill(theme::SURFACE_HI)
            .min_size(Vec2::new(0.0, 28.0)),
    )
}

/// Knopf mit Symbol und optionaler Beschriftung.
pub fn icon_button(ui: &mut egui::Ui, icon: Icon, label: &str, accent: Color32) -> Response {
    let font = FontId::proportional(12.5);
    let galley = if label.is_empty() {
        None
    } else {
        Some(
            ui.painter()
                .layout_no_wrap(label.to_string(), font, theme::TEXT),
        )
    };
    let icon_size = 16.0;
    let text_width = galley.as_ref().map(|g| g.size().x + 8.0).unwrap_or(0.0);
    let size = Vec2::new(icon_size + text_width + 20.0, 30.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hovered = response.hovered();
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        CornerRadius::same(9),
        if hovered {
            theme::SURFACE_HI
        } else {
            theme::SURFACE
        },
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(9),
        Stroke::new(
            1.0_f32,
            if hovered {
                accent.gamma_multiply(0.7)
            } else {
                theme::BORDER
            },
        ),
        StrokeKind::Inside,
    );
    let icon_rect = Rect::from_center_size(
        rect.left_center() + Vec2::new(10.0 + icon_size * 0.5, 0.0),
        Vec2::splat(icon_size),
    );
    icons::paint(painter, icon_rect, icon, accent);
    if let Some(galley) = galley {
        let position = icon_rect.right_center() + Vec2::new(8.0, -galley.size().y * 0.5);
        painter.galley(position, galley, theme::TEXT);
    }
    response
}

/// Monospace-Block für Rohausgaben (scrollbar, mit Kopierknopf).
pub fn mono_block(ui: &mut egui::Ui, id: &str, text: &str, max_height: f32) {
    egui::Frame::new()
        .fill(Color32::from_rgb(8, 6, 4))
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(CornerRadius::same(9))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::ScrollArea::both()
                .id_salt(id)
                .max_height(max_height)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    ui.label(
                        RichText::new(text)
                            .monospace()
                            .size(11.5)
                            .color(theme::MUTED),
                    );
                });
        });
}

pub fn copy_button(ui: &mut egui::Ui, text: &str, label: &str) {
    if icon_button(ui, Icon::Copy, label, theme::AMBER).clicked() {
        ui.ctx().copy_text(text.to_string());
    }
}

pub fn divider(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, theme::BORDER);
}

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

pub fn skeleton(ui: &mut egui::Ui) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.spinner();
        ui.add_space(8.0);
        hint(ui, "Systemdaten werden gelesen …");
    });
}
