//! Designsytem «ASCII Brutal» des Dateibrowsers – dieselbe Sprache wie das
//! Control Center: reines Schwarz, harte Kanten, Arch-Blau, Monospace.
//!
//! Anders als dort werden die mitgelieferten egui-Schriften verwendet
//! (Ubuntu-Light und Hack). Das spart dem kleinen Werkzeug eigene Fontdaten.

use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Vec2,
};

// --------------------------------------------------------------- Flächen
pub const INK: Color32 = Color32::BLACK;
pub const PANEL: Color32 = Color32::from_rgb(6, 6, 6);
pub const RAISED: Color32 = Color32::from_rgb(16, 16, 16);
pub const WELL: Color32 = Color32::BLACK;

// --------------------------------------------------------------- Linien
pub const HAIRLINE: Color32 = Color32::from_rgb(42, 42, 42);
pub const HAIRLINE_STRONG: Color32 = Color32::from_rgb(74, 74, 74);

// --------------------------------------------------------------- Text
pub const TEXT: Color32 = Color32::from_rgb(242, 242, 236);
pub const MUTED: Color32 = Color32::from_rgb(154, 154, 146);
pub const FAINT: Color32 = Color32::from_rgb(94, 94, 88);

// --------------------------------------------------------------- Signale
pub const ACCENT: Color32 = Color32::from_rgb(23, 147, 209);
pub const OK: Color32 = Color32::from_rgb(55, 224, 138);
pub const WARN: Color32 = Color32::from_rgb(255, 197, 61);
pub const ERROR: Color32 = Color32::from_rgb(255, 90, 74);
pub const INFO: Color32 = Color32::from_rgb(127, 212, 255);

// --------------------------------------------------------------- Kanten
// Alle Flächen sind eckig; es gibt keine Rundungs-Token mehr.

/// Überschriften und Namen.
pub fn display(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

/// Fließtext und Beschriftungen.
pub fn sans(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

/// Technische Daten, Größen, Zeitstempel.
pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

/// Setzt Alpha auf eine deckende Farbe.
pub fn alpha(color: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}

pub fn install(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = egui::Visuals::dark();

    visuals.override_text_color = Some(TEXT);
    visuals.panel_fill = INK;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = WELL;
    visuals.faint_bg_color = RAISED;
    visuals.window_stroke = Stroke::new(1.0_f32, HAIRLINE_STRONG);
    visuals.window_corner_radius = CornerRadius::ZERO;
    visuals.menu_corner_radius = CornerRadius::ZERO;
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0_f32, INK);
    visuals.hyperlink_color = INFO;
    visuals.striped = false;

    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = CornerRadius::ZERO;
    }

    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.weak_bg_fill = PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, HAIRLINE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, MUTED);

    visuals.widgets.inactive.weak_bg_fill = RAISED;
    visuals.widgets.inactive.bg_fill = RAISED;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, HAIRLINE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.inactive.expansion = 0.0;

    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(24, 24, 24);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(24, 24, 24);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.hovered.expansion = 0.0;

    visuals.widgets.active.weak_bg_fill = ACCENT;
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, INK);
    visuals.widgets.active.expansion = 0.0;

    visuals.widgets.open.weak_bg_fill = RAISED;
    visuals.widgets.open.bg_fill = RAISED;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, HAIRLINE_STRONG);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, TEXT);

    style.visuals = visuals;
    style.animation_time = 0.08;
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(10.0, 5.0);
    style.spacing.menu_margin = Margin::same(8);
    style.spacing.indent = 18.0;
    style.spacing.interact_size.y = 26.0;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.floating = false;

    style.text_styles.insert(TextStyle::Heading, display(22.0));
    style.text_styles.insert(TextStyle::Body, sans(13.0));
    style.text_styles.insert(TextStyle::Button, mono(11.5));
    style.text_styles.insert(TextStyle::Small, sans(11.0));
    style.text_styles.insert(TextStyle::Monospace, mono(11.0));

    ctx.set_style(style);
}
