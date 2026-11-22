//! Designsytem «ASCII Brutal» – Brutalismus, Arch-Blau und tiefes Schwarz.
//!
//! Grundfläche ist reines Schwarz (#000). Karten sind schwarze Blöcke mit
//! einer harten 1-px-Kante und Eckmarken; es gibt keine Rundungen. Der einzige
//! Akzent ist Arch-Blau (#1793D1), Status tragen eigene Signalfarben.
//!
//! Typografie in drei Rollen:
//! * `display` – Geist Mono: Überschriften und Kennzahlen, terminalartig
//! * `sans`    – Geist: Fließtext und Beschriftungen
//! * `mono`    – Geist Mono: technische Daten, Tabellen, Code, Zeitstempel

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke,
    TextStyle, Vec2,
};

// --------------------------------------------------------------- Flächen
/// Grundfläche des Fensters: reines Schwarz.
pub const INK: Color32 = Color32::BLACK;
/// Kartenfläche, einen Hauch über Schwarz.
pub const PANEL: Color32 = Color32::from_rgb(6, 6, 6);
/// Erhöhte Fläche (Knöpfe, Hover, Zeilen).
pub const RAISED: Color32 = Color32::from_rgb(16, 16, 16);
/// Eingelassene Fläche für Code und Protokolle.
pub const WELL: Color32 = Color32::BLACK;

// --------------------------------------------------------------- Linien
/// Harte 1-px-Kante statt Glas.
pub const HAIRLINE: Color32 = Color32::from_rgb(42, 42, 42);
pub const HAIRLINE_STRONG: Color32 = Color32::from_rgb(74, 74, 74);
/// Äußere Eckmarken einer Karte.
pub const RING: Color32 = Color32::from_rgb(74, 74, 74);
/// Sehr leichte Füllung (Chips, Badges).
pub const GLASS: Color32 = Color32::from_rgb(10, 10, 10);

// --------------------------------------------------------------- Text
pub const TEXT: Color32 = Color32::from_rgb(242, 242, 236);
pub const MUTED: Color32 = Color32::from_rgb(154, 154, 146);
pub const FAINT: Color32 = Color32::from_rgb(94, 94, 88);

// --------------------------------------------------------------- Signale
/// Einziger Akzent: Arch-Blau.
pub const ACCENT: Color32 = Color32::from_rgb(23, 147, 209);
pub const OK: Color32 = Color32::from_rgb(55, 224, 138);
pub const WARN: Color32 = Color32::from_rgb(255, 197, 61);
pub const ERROR: Color32 = Color32::from_rgb(255, 90, 74);
pub const INFO: Color32 = Color32::from_rgb(127, 212, 255);

// --------------------------------------------------------------- Kanten
pub const CARD_RADIUS: u8 = 0;
pub const INNER_RADIUS: u8 = 0;

const DISPLAY: &str = "display";
const SANS: &str = "geist";

/// Terminal-Schrift für Titel und Kennzahlen.
pub fn display(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(DISPLAY.into()))
}

/// Fließtext-Schrift.
pub fn sans(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SANS.into()))
}

/// Technische Schrift für Daten, Tabellen und Code.
pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

/// Setzt Alpha auf eine deckende Farbe.
pub fn alpha(color: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}

fn fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "geist".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Geist-Variable.ttf"
        ))),
    );
    fonts.font_data.insert(
        "geist-mono".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/GeistMono-Variable.ttf"
        ))),
    );

    let mono = vec!["geist-mono".to_owned(), "Hack".to_owned()];
    let sans = vec![
        "geist".to_owned(),
        "Ubuntu-Light".to_owned(),
    ];

    fonts
        .families
        .insert(FontFamily::Name(DISPLAY.into()), mono.clone());
    fonts
        .families
        .insert(FontFamily::Name(SANS.into()), sans.clone());
    fonts.families.insert(FontFamily::Proportional, sans);
    fonts.families.insert(FontFamily::Monospace, mono);

    fonts
}

pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(fonts());

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
    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.spacing.button_padding = Vec2::new(12.0, 7.0);
    style.spacing.menu_margin = Margin::same(8);
    style.spacing.indent = 18.0;
    style.spacing.interact_size.y = 26.0;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.floating = false;

    style.text_styles.insert(TextStyle::Heading, display(24.0));
    style.text_styles.insert(TextStyle::Body, sans(13.5));
    style.text_styles.insert(TextStyle::Button, mono(12.0));
    style.text_styles.insert(TextStyle::Small, sans(11.5));
    style.text_styles.insert(TextStyle::Monospace, mono(11.5));

    ctx.set_style(style);
}
