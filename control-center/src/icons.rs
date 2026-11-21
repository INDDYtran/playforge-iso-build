//! Kleine Vektor-Symbole, mit dem egui-Painter gezeichnet.
//!
//! Bewusst keine Icon-Schrift: Die Symbole skalieren so mit der Schriftgröße
//! und es gibt keine fehlenden Glyphen auf dem Live-System.

use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Shape, Stroke, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Dashboard,
    Gauge,
    Gamepad,
    Monitor,
    Sliders,
    Shield,
    Terminal,
    Refresh,
    Bolt,
    Cpu,
    Memory,
    Copy,
    Check,
    Warn,
    Cross,
}

/// Zeichnet `icon` zentriert in `rect`.
pub fn paint(painter: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let c = rect.center();
    let r = rect.width().min(rect.height()) * 0.41;
    let w = (r * 0.20).clamp(1.3, 2.4);
    let st = Stroke::new(w, color);

    match icon {
        Icon::Dashboard => {
            let sz = 0.40 * r;
            for (i, (dx, dy)) in [(-0.47, -0.47), (0.47, -0.47), (-0.47, 0.47), (0.47, 0.47)]
                .iter()
                .enumerate()
            {
                let center = c + Vec2::new(dx * r, dy * r);
                let tile = Rect::from_center_size(center, Vec2::splat(sz * 2.0));
                let fill = if i == 0 {
                    color
                } else {
                    color.gamma_multiply(0.55)
                };
                painter.rect_filled(tile, CornerRadius::same((sz * 0.45) as u8), fill);
            }
        }
        Icon::Gauge => {
            painter.add(Shape::line(arc(c, r * 0.86, 165.0, 385.0, 40), st));
            let needle = c + Vec2::new(-0.30 * r, -0.34 * r);
            painter.line_segment([c, needle], st);
            painter.circle_filled(c, w * 1.2, color);
        }
        Icon::Gamepad => {
            let body = Rect::from_center_size(c, Vec2::new(1.72 * r, 1.02 * r));
            painter.rect_filled(
                body,
                CornerRadius::same((0.34 * r) as u8),
                color.gamma_multiply(0.16),
            );
            painter.rect_stroke(
                body,
                CornerRadius::same((0.34 * r) as u8),
                st,
                egui::StrokeKind::Inside,
            );
            let pad = c + Vec2::new(-0.44 * r, 0.0);
            painter.line_segment(
                [
                    pad + Vec2::new(-0.15 * r, 0.0),
                    pad + Vec2::new(0.15 * r, 0.0),
                ],
                st,
            );
            painter.line_segment(
                [
                    pad + Vec2::new(0.0, -0.15 * r),
                    pad + Vec2::new(0.0, 0.15 * r),
                ],
                st,
            );
            painter.circle_filled(c + Vec2::new(0.36 * r, -0.14 * r), 0.075 * r, color);
            painter.circle_filled(c + Vec2::new(0.54 * r, 0.10 * r), 0.075 * r, color);
        }
        Icon::Monitor => {
            let screen = Rect::from_center_size(
                c + Vec2::new(0.0, -0.16 * r),
                Vec2::new(1.62 * r, 1.06 * r),
            );
            painter.rect_stroke(
                screen,
                CornerRadius::same((0.14 * r) as u8),
                st,
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [c + Vec2::new(0.0, 0.42 * r), c + Vec2::new(0.0, 0.70 * r)],
                st,
            );
            painter.line_segment(
                [
                    c + Vec2::new(-0.34 * r, 0.74 * r),
                    c + Vec2::new(0.34 * r, 0.74 * r),
                ],
                st,
            );
        }
        Icon::Sliders => {
            for (dy, knob) in [(-0.56, -0.22), (0.0, 0.34), (0.56, -0.42)] {
                let y = c.y + dy * r;
                painter.line_segment(
                    [Pos2::new(c.x - 0.82 * r, y), Pos2::new(c.x + 0.82 * r, y)],
                    Stroke::new(w * 0.85, color.gamma_multiply(0.6)),
                );
                painter.circle_filled(Pos2::new(c.x + knob * r, y), 0.14 * r, color);
            }
        }
        Icon::Shield => {
            let pts = poly(
                &[
                    (0.0, -0.92),
                    (0.76, -0.62),
                    (0.76, 0.10),
                    (0.0, 0.88),
                    (-0.76, 0.10),
                    (-0.76, -0.62),
                ],
                c,
                r,
            );
            painter.add(Shape::closed_line(pts, st));
            painter.add(Shape::line(
                poly(&[(-0.30, 0.02), (-0.04, 0.30), (0.40, -0.26)], c, r),
                st,
            ));
        }
        Icon::Terminal => {
            let frame = Rect::from_center_size(c, Vec2::new(1.78 * r, 1.36 * r));
            painter.rect_stroke(
                frame,
                CornerRadius::same((0.16 * r) as u8),
                st,
                egui::StrokeKind::Inside,
            );
            painter.add(Shape::line(
                poly(&[(-0.46, -0.30), (-0.14, 0.02), (-0.46, 0.34)], c, r),
                st,
            ));
            painter.line_segment(
                [
                    c + Vec2::new(0.16 * r, 0.36 * r),
                    c + Vec2::new(0.56 * r, 0.36 * r),
                ],
                st,
            );
        }
        Icon::Refresh => {
            let pts = arc(c, 0.82 * r, 30.0, 320.0, 34);
            painter.add(Shape::line(pts, st));
            let tip = c + Vec2::new(
                0.82 * r * 30f32.to_radians().cos(),
                0.82 * r * 30f32.to_radians().sin(),
            );
            painter.add(Shape::line(
                vec![
                    tip + Vec2::new(-0.30 * r, -0.06 * r),
                    tip,
                    tip + Vec2::new(-0.02 * r, -0.34 * r),
                ],
                st,
            ));
        }
        Icon::Bolt => {
            painter.add(Shape::convex_polygon(
                poly(
                    &[
                        (0.18, -0.95),
                        (-0.50, 0.08),
                        (-0.08, 0.08),
                        (-0.18, 0.95),
                        (0.50, -0.10),
                        (0.08, -0.10),
                    ],
                    c,
                    r,
                ),
                color,
                Stroke::NONE,
            ));
        }
        Icon::Cpu => {
            let square = Rect::from_center_size(c, Vec2::splat(1.18 * r));
            painter.rect_stroke(
                square,
                CornerRadius::same((0.14 * r) as u8),
                st,
                egui::StrokeKind::Inside,
            );
            painter.rect_filled(
                Rect::from_center_size(c, Vec2::splat(0.40 * r)),
                CornerRadius::ZERO,
                color.gamma_multiply(0.85),
            );
            for i in [-0.34_f32, 0.0, 0.34] {
                painter.line_segment(
                    [
                        c + Vec2::new(i * r, -0.59 * r),
                        c + Vec2::new(i * r, -0.90 * r),
                    ],
                    st,
                );
                painter.line_segment(
                    [
                        c + Vec2::new(i * r, 0.59 * r),
                        c + Vec2::new(i * r, 0.90 * r),
                    ],
                    st,
                );
                painter.line_segment(
                    [
                        c + Vec2::new(-0.59 * r, i * r),
                        c + Vec2::new(-0.90 * r, i * r),
                    ],
                    st,
                );
                painter.line_segment(
                    [
                        c + Vec2::new(0.59 * r, i * r),
                        c + Vec2::new(0.90 * r, i * r),
                    ],
                    st,
                );
            }
        }
        Icon::Memory => {
            for (i, dx) in [-0.56_f32, 0.0, 0.56].iter().enumerate() {
                let height = [0.9_f32, 1.2, 0.7][i];
                let bar = Rect::from_min_size(
                    Pos2::new(c.x + dx * r - 0.16 * r, c.y + 0.55 * r - height * r),
                    Vec2::new(0.32 * r, height * r),
                );
                painter.rect_filled(
                    bar,
                    CornerRadius::same((0.10 * r) as u8),
                    color.gamma_multiply(if i == 1 { 1.0 } else { 0.55 }),
                );
            }
            painter.line_segment(
                [
                    c + Vec2::new(-0.86 * r, 0.66 * r),
                    c + Vec2::new(0.86 * r, 0.66 * r),
                ],
                st,
            );
        }
        Icon::Copy => {
            let back =
                Rect::from_min_size(c + Vec2::new(-0.72 * r, -0.72 * r), Vec2::splat(0.88 * r));
            let front =
                Rect::from_min_size(c + Vec2::new(-0.20 * r, -0.20 * r), Vec2::splat(0.88 * r));
            painter.rect_stroke(
                back,
                CornerRadius::same((0.14 * r) as u8),
                Stroke::new(w * 0.85, color.gamma_multiply(0.6)),
                egui::StrokeKind::Inside,
            );
            painter.rect_stroke(
                front,
                CornerRadius::same((0.14 * r) as u8),
                st,
                egui::StrokeKind::Inside,
            );
        }
        Icon::Check => {
            painter.add(Shape::line(
                poly(&[(-0.72, 0.02), (-0.16, 0.56), (0.74, -0.52)], c, r),
                Stroke::new(w * 1.15, color),
            ));
        }
        Icon::Warn => {
            painter.add(Shape::closed_line(
                poly(&[(0.0, -0.86), (0.88, 0.72), (-0.88, 0.72)], c, r),
                st,
            ));
            painter.line_segment(
                [c + Vec2::new(0.0, -0.34 * r), c + Vec2::new(0.0, 0.22 * r)],
                st,
            );
            painter.circle_filled(c + Vec2::new(0.0, 0.46 * r), w * 0.75, color);
        }
        Icon::Cross => {
            painter.line_segment(
                [
                    c + Vec2::new(-0.62 * r, -0.62 * r),
                    c + Vec2::new(0.62 * r, 0.62 * r),
                ],
                Stroke::new(w * 1.1, color),
            );
            painter.line_segment(
                [
                    c + Vec2::new(0.62 * r, -0.62 * r),
                    c + Vec2::new(-0.62 * r, 0.62 * r),
                ],
                Stroke::new(w * 1.1, color),
            );
        }
    }
}

fn poly(points: &[(f32, f32)], c: Pos2, r: f32) -> Vec<Pos2> {
    points
        .iter()
        .map(|(x, y)| c + Vec2::new(x * r, y * r))
        .collect()
}

fn arc(c: Pos2, radius: f32, start_deg: f32, end_deg: f32, segments: usize) -> Vec<Pos2> {
    let segments = segments.max(8);
    (0..=segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            let angle = (start_deg + (end_deg - start_deg) * t).to_radians();
            c + Vec2::new(radius * angle.cos(), radius * angle.sin())
        })
        .collect()
}
