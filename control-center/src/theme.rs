//! Designsytem «Noir Grid» – brutalistisch-moderner Dark Mode.
//!
//! Grundlage ist reines Schwarz (#000). Darüber liegen nahezu schwarze
//! Flächen mit 1-px-Haarlinien und Glas-Ringen (Weiß, 4–7 % Alpha), die den
//! Rand einer Karte wie eine gebrochene Kante wirken lassen. Akzent ist ein
//! einzelnes Säure-Grün; Status tragen eigene, gedämpfte Signalfarben.
//!
//! Typografie in drei Rollen:
//! * `display` – Inter Display: Titel und Kennzahlen, editorial und weit
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
pub const PANEL: Color32 = Color32::from_rgb(8, 8, 10);
/// Erhöhte Fläche (Knöpfe, Hover, Zeilen).
pub const RAISED: Color32 = Color32::from_rgb(15, 15, 18);
/// Eingelassene Fläche für Code und Protokolle.
pub const WELL: Color32 = Color32::from_rgb(3, 3, 4);

// --------------------------------------------------------------- Linien
// `from_rgba_unmultiplied` ist keine const-fn; vorgemultipliziertes Weiß
// ergibt über den dunklen Flächen denselben zarten Glas-Eindruck.
pub const HAIRLINE: Color32 = Color32::from_rgba_premultiplied(18, 18, 18, 18);
pub const HAIRLINE_STRONG: Color32 = Color32::from_rgba_premultiplied(40, 40, 40, 40);
/// Äußerer Glas-Ring um Karten.
pub const RING: Color32 = Color32::from_rgba_premultiplied(10, 10, 10, 10);
/// Sehr leichte Glasfüllung (Chips, Badges).
pub const GLASS: Color32 = Color32::from_rgba_premultiplied(8, 8, 8, 8);

// --------------------------------------------------------------- Text
pub const TEXT: Color32 = Color32::from_rgb(244, 244, 245);
pub const MUTED: Color32 = Color32::from_rgb(142, 142, 152);
pub const FAINT: Color32 = Color32::from_rgb(88, 88, 98);

// --------------------------------------------------------------- Signale
/// Einziger Akzent: Säure-Grün.
pub const ACCENT: Color32 = Color32::from_rgb(198, 255, 74);
pub const OK: Color32 = Color32::from_rgb(74, 222, 128);
pub const WARN: Color32 = Color32::from_rgb(250, 204, 21);
pub const ERROR: Color32 = Color32::from_rgb(255, 95, 86);
pub const INFO: Color32 = Color32::from_rgb(125, 211, 252);

// --------------------------------------------------------------- Radien
pub const CARD_RADIUS: u8 = 14;
pub const INNER_RADIUS: u8 = 9;

const DISPLAY: &str = "display";
const SANS: &str = "geist";
const MONO: &str = "geist-mono";

/// Editorial-Schrift für Titel und Kennzahlen.
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
        "inter-display".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/InterDisplay-Medium.ttf"
        ))),
    );
    fonts.font_data.insert(
        "inter-display-regular".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/InterDisplay-Regular.ttf"
        ))),
    );
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

    let display = vec![
        "inter-display".to_owned(),
        "inter-display-regular".to_owned(),
        "geist".to_owned(),
        "Ubuntu-Light".to_owned(),
    ];
    let sans = vec![
        "geist".to_owned(),
        "inter-display-regular".to_owned(),
        "Ubuntu-Light".to_owned(),
    ];
    let mono = vec!["geist-mono".to_owned(), "Hack".to_owned()];

    fonts
        .families
        .insert(FontFamily::Name(DISPLAY.into()), display);
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
    visuals.window_stroke = Stroke::new(1.0_f32, HAIRLINE);
    visuals.window_corner_radius = CornerRadius::same(CARD_RADIUS);
    visuals.menu_corner_radius = CornerRadius::same(INNER_RADIUS);
    visuals.selection.bg_fill = alpha(ACCENT, 48);
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.hyperlink_color = ACCENT;
    visuals.striped = false;

    let radius = CornerRadius::same(INNER_RADIUS);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = radius;
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

    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(22, 22, 26);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(22, 22, 26);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, HAIRLINE_STRONG);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.hovered.expansion = 0.0;

    visuals.widgets.active.weak_bg_fill = alpha(ACCENT, 40);
    visuals.widgets.active.bg_fill = alpha(ACCENT, 40);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.active.expansion = 0.0;

    visuals.widgets.open.weak_bg_fill = RAISED;
    visuals.widgets.open.bg_fill = RAISED;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, HAIRLINE_STRONG);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, TEXT);

    style.visuals = visuals;
    style.animation_time = 0.10;
    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.spacing.button_padding = Vec2::new(12.0, 7.0);
    style.spacing.menu_margin = Margin::same(8);
    style.spacing.indent = 18.0;
    style.spacing.interact_size.y = 26.0;
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.floating = true;

    style.text_styles.insert(TextStyle::Heading, display(26.0));
    style.text_styles.insert(TextStyle::Body, sans(13.5));
    style.text_styles.insert(TextStyle::Button, sans(13.0));
    style.text_styles.insert(TextStyle::Small, sans(11.5));
    style.text_styles.insert(TextStyle::Monospace, mono(11.5));

    ctx.set_style(style);
}
