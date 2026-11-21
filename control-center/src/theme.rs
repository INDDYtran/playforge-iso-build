//! Farben und Stil des Control Centers.
//!
//! Die Palette stammt aus `overlay/usr/share/color-schemes/Playforge.colors`,
//! damit die App neben Plasma wie ein Teil des Systems wirkt.

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Vec2};

// Hintergründe (warmes Eisen)
pub const SIDEBAR: Color32 = Color32::from_rgb(12, 10, 8);
pub const BG: Color32 = Color32::from_rgb(16, 14, 12);
pub const SURFACE: Color32 = Color32::from_rgb(26, 22, 18);
pub const SURFACE_HI: Color32 = Color32::from_rgb(36, 30, 22);
pub const BORDER: Color32 = Color32::from_rgb(62, 50, 34);

// Text
pub const TEXT: Color32 = Color32::from_rgb(245, 239, 228);
pub const MUTED: Color32 = Color32::from_rgb(160, 148, 126);
pub const FAINT: Color32 = Color32::from_rgb(110, 100, 82);

// Akzente
pub const AMBER: Color32 = Color32::from_rgb(245, 158, 11);
pub const COPPER: Color32 = Color32::from_rgb(196, 120, 56);
pub const GREEN: Color32 = Color32::from_rgb(90, 185, 110);
pub const RED: Color32 = Color32::from_rgb(220, 90, 75);
pub const WARN: Color32 = Color32::from_rgb(245, 190, 80);
pub const BLUE: Color32 = Color32::from_rgb(150, 160, 175);

pub const CARD_RADIUS: u8 = 12;

pub fn install(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(TEXT);
    visuals.panel_fill = BG;
    visuals.window_fill = SURFACE;
    visuals.extreme_bg_color = Color32::from_rgb(8, 6, 4);
    visuals.faint_bg_color = SURFACE_HI;
    visuals.window_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.menu_corner_radius = CornerRadius::same(10);
    visuals.selection.bg_fill = AMBER.gamma_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0_f32, TEXT);
    visuals.hyperlink_color = AMBER;
    visuals.striped = true;

    let radius = CornerRadius::same(9);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = radius;
    }

    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.weak_bg_fill = SURFACE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT);

    visuals.widgets.inactive.weak_bg_fill = SURFACE_HI;
    visuals.widgets.inactive.bg_fill = SURFACE_HI;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.inactive.expansion = 0.0;

    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(46, 38, 26);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(46, 38, 26);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, AMBER.gamma_multiply(0.7));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.hovered.expansion = 1.0;

    visuals.widgets.active.weak_bg_fill = AMBER.gamma_multiply(0.35);
    visuals.widgets.active.bg_fill = AMBER.gamma_multiply(0.35);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, AMBER);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.active.expansion = 0.0;

    visuals.widgets.open.weak_bg_fill = SURFACE;
    visuals.widgets.open.bg_fill = SURFACE;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, TEXT);

    style.visuals = visuals;
    style.animation_time = 0.12;
    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.spacing.button_padding = Vec2::new(12.0, 7.0);
    style.spacing.menu_margin = egui::Margin::same(8);
    style.spacing.indent = 18.0;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.floating = true;

    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(23.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(11.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(12.5, FontFamily::Monospace),
    );

    ctx.set_style(style);
}
