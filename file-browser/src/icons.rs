//! Selbst gezeichnete Symbole – keine Icon-Dateien, keine externen Assets.
//!
//! Alle Formen liegen in einem 0…1-Raster und werden auf das Zielrechteck
//! skaliert. Dadurch bleibt die Strichstärke bei jeder Fenstergröße lesbar.

use eframe::egui::{Color32, CornerRadius, Painter, Pos2, Rect, Shape, Stroke};

use crate::fs::{Entry, Kind};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Folder,
    File,
    Image,
    Code,
    Archive,
    Audio,
    Video,
    Pdf,
    Home,
    Disk,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Refresh,
    Plus,
    Eye,
    EyeOff,
}

pub fn for_entry(entry: &Entry) -> Icon {
    if entry.is_dir {
        return Icon::Folder;
    }
    match entry.kind {
        Kind::Folder => Icon::Folder,
        Kind::Image => Icon::Image,
        Kind::Code => Icon::Code,
        Kind::Archive => Icon::Archive,
        Kind::Audio => Icon::Audio,
        Kind::Video => Icon::Video,
        Kind::Pdf => Icon::Pdf,
        Kind::Text | Kind::Other => Icon::File,
    }
}

pub fn paint(painter: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let point = |x: f32, y: f32| {
        Pos2::new(
            rect.left() + rect.width() * x,
            rect.top() + rect.height() * y,
        )
    };
    let stroke = Stroke::new((rect.width() * 0.075).clamp(1.0, 2.4), color);
    let line = |points: &[(f32, f32)], stroke: Stroke| {
        let points: Vec<Pos2> = points.iter().map(|&(x, y)| point(x, y)).collect();
        painter.add(Shape::line(points, stroke));
    };
    let polygon = |points: &[(f32, f32)], fill: Color32| {
        let points: Vec<Pos2> = points.iter().map(|&(x, y)| point(x, y)).collect();
        painter.add(Shape::convex_polygon(points, fill, Stroke::NONE));
    };
    let radius = |_factor: f32| CornerRadius::ZERO;

    match icon {
        Icon::Folder => {
            polygon(
                &[(0.08, 0.22), (0.40, 0.22), (0.50, 0.30), (0.08, 0.30)],
                color,
            );
            painter.rect_filled(
                Rect::from_min_max(point(0.08, 0.30), point(0.92, 0.86)),
                radius(0.08),
                color,
            );
        }
        Icon::File => {
            painter.rect_filled(
                Rect::from_min_max(point(0.26, 0.08), point(0.78, 0.92)),
                radius(0.05),
                color,
            );
            let ink = crate::theme::INK;
            for y in [0.34, 0.50, 0.66] {
                line(
                    &[(0.36, y), (0.68, y)],
                    Stroke::new(stroke.width * 0.8, ink),
                );
            }
        }
        Icon::Image => {
            painter.rect_filled(
                Rect::from_min_max(point(0.14, 0.16), point(0.86, 0.84)),
                radius(0.07),
                color,
            );
            let ink = crate::theme::INK;
            painter.circle_filled(point(0.36, 0.36), rect.width() * 0.07, ink);
            polygon(
                &[
                    (0.20, 0.78),
                    (0.46, 0.48),
                    (0.62, 0.66),
                    (0.72, 0.54),
                    (0.82, 0.78),
                ],
                ink,
            );
        }
        Icon::Code => {
            line(&[(0.35, 0.30), (0.15, 0.50), (0.35, 0.70)], stroke);
            line(&[(0.65, 0.30), (0.85, 0.50), (0.65, 0.70)], stroke);
            line(&[(0.56, 0.22), (0.44, 0.78)], stroke);
        }
        Icon::Archive => {
            painter.rect_filled(
                Rect::from_min_max(point(0.18, 0.18), point(0.82, 0.86)),
                radius(0.05),
                color,
            );
            let ink = crate::theme::INK;
            line(
                &[(0.50, 0.18), (0.50, 0.86)],
                Stroke::new(stroke.width, ink),
            );
            for y in [0.34, 0.46, 0.58] {
                line(
                    &[(0.50, y), (0.62, y)],
                    Stroke::new(stroke.width * 0.8, ink),
                );
            }
        }
        Icon::Audio => {
            painter.circle_filled(point(0.36, 0.74), rect.width() * 0.13, color);
            line(
                &[(0.48, 0.72), (0.48, 0.22), (0.78, 0.30), (0.78, 0.64)],
                stroke,
            );
            painter.circle_filled(point(0.68, 0.66), rect.width() * 0.10, color);
        }
        Icon::Video => {
            painter.rect_filled(
                Rect::from_min_max(point(0.12, 0.22), point(0.88, 0.78)),
                radius(0.09),
                color,
            );
            polygon(
                &[(0.42, 0.36), (0.42, 0.64), (0.68, 0.50)],
                crate::theme::INK,
            );
        }
        Icon::Pdf => {
            painter.rect_filled(
                Rect::from_min_max(point(0.26, 0.08), point(0.78, 0.92)),
                radius(0.05),
                color,
            );
            polygon(
                &[(0.62, 0.08), (0.78, 0.24), (0.62, 0.24)],
                crate::theme::INK,
            );
            let ink = crate::theme::INK;
            line(
                &[(0.36, 0.44), (0.68, 0.44)],
                Stroke::new(stroke.width, ink),
            );
            line(
                &[(0.36, 0.58), (0.60, 0.58)],
                Stroke::new(stroke.width, ink),
            );
            line(
                &[(0.36, 0.72), (0.66, 0.72)],
                Stroke::new(stroke.width, ink),
            );
        }
        Icon::Home => {
            polygon(&[(0.50, 0.14), (0.90, 0.48), (0.10, 0.48)], color);
            painter.rect_filled(
                Rect::from_min_max(point(0.22, 0.46), point(0.78, 0.88)),
                radius(0.04),
                color,
            );
            painter.rect_filled(
                Rect::from_min_max(point(0.44, 0.64), point(0.58, 0.88)),
                CornerRadius::ZERO,
                crate::theme::INK,
            );
        }
        Icon::Disk => {
            painter.rect_filled(
                Rect::from_min_max(point(0.14, 0.24), point(0.86, 0.76)),
                radius(0.09),
                color,
            );
            painter.circle_filled(point(0.70, 0.50), rect.width() * 0.07, crate::theme::INK);
            line(
                &[(0.24, 0.62), (0.56, 0.62)],
                Stroke::new(stroke.width, crate::theme::INK),
            );
        }
        Icon::ArrowLeft => {
            line(&[(0.82, 0.50), (0.22, 0.50)], stroke);
            line(&[(0.42, 0.30), (0.20, 0.50), (0.42, 0.70)], stroke);
        }
        Icon::ArrowRight => {
            line(&[(0.18, 0.50), (0.78, 0.50)], stroke);
            line(&[(0.58, 0.30), (0.80, 0.50), (0.58, 0.70)], stroke);
        }
        Icon::ArrowUp => {
            line(&[(0.50, 0.82), (0.50, 0.22)], stroke);
            line(&[(0.30, 0.42), (0.50, 0.20), (0.70, 0.42)], stroke);
        }
        Icon::Refresh => {
            let mut points = Vec::new();
            let start = 0.35_f32;
            let end = 1.85_f32;
            for step in 0..=28 {
                let angle = start + (end - start) * step as f32 / 28.0;
                points.push(point(0.50 + 0.30 * angle.cos(), 0.50 + 0.30 * angle.sin()));
            }
            painter.add(Shape::line(points, stroke));
            line(&[(0.20, 0.30), (0.22, 0.55), (0.46, 0.50)], stroke);
        }
        Icon::Plus => {
            line(&[(0.50, 0.20), (0.50, 0.80)], stroke);
            line(&[(0.20, 0.50), (0.80, 0.50)], stroke);
        }
        Icon::Eye | Icon::EyeOff => {
            let mut points = Vec::new();
            for step in 0..=20 {
                let t = step as f32 / 20.0;
                let x = 0.10 + 0.80 * t;
                let y = 0.50 - (std::f32::consts::PI * t).sin() * 0.26;
                points.push(point(x, y));
            }
            for step in (0..=20).rev() {
                let t = step as f32 / 20.0;
                let x = 0.10 + 0.80 * t;
                let y = 0.50 + (std::f32::consts::PI * t).sin() * 0.26;
                points.push(point(x, y));
            }
            painter.add(Shape::closed_line(points, stroke));
            painter.circle_filled(point(0.50, 0.50), rect.width() * 0.11, color);
            if icon == Icon::EyeOff {
                line(&[(0.18, 0.18), (0.82, 0.82)], stroke);
            }
        }
    }
}
