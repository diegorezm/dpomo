//! Small hand-painted widgets. Everything is drawn with the painter so the
//! look is identical on every platform and doesn't depend on icon fonts.

use std::f32::consts::TAU;

use egui::epaint::PathShape;
use egui::{
    Button, Color32, CornerRadius, CursorIcon, Painter, Pos2, Rect, Response, RichText, Sense,
    Shape, Stroke, Ui, pos2, vec2,
};

use crate::theme::{Theme, mix};

/// Progress ring. `progress` is the visible fraction, starting at 12 o'clock.
#[allow(clippy::too_many_arguments)]
pub fn ring(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    thickness: f32,
    progress: f32,
    track: Color32,
    color: Color32,
    glow: bool,
) {
    painter.circle_stroke(center, radius, Stroke::new(thickness, track));

    let p = progress.clamp(0.0, 1.0);
    if p <= 0.001 {
        return;
    }
    let segments = ((p * 128.0).ceil() as usize).max(2);
    let start = -TAU / 4.0;
    let points: Vec<Pos2> = (0..=segments)
        .map(|i| {
            let a = start + TAU * p * i as f32 / segments as f32;
            center + radius * vec2(a.cos(), a.sin())
        })
        .collect();

    if glow {
        // No round caps here: overlapping translucent caps would show as blobs.
        for (scale, alpha) in [(2.6, 0.05), (1.8, 0.10)] {
            let stroke = Stroke::new(thickness * scale, color.gamma_multiply(alpha));
            if p >= 0.999 {
                // Closed path so the two ends don't leave a seam at 12 o'clock.
                let closed = points[..points.len() - 1].to_vec();
                painter.add(PathShape::closed_line(closed, stroke));
            } else {
                painter.add(PathShape::line(points.clone(), stroke));
            }
        }
    }
    arc(painter, &points, thickness, color);
}

/// Polyline with round caps.
fn arc(painter: &Painter, points: &[Pos2], width: f32, color: Color32) {
    painter.add(PathShape::line(points.to_vec(), Stroke::new(width, color)));
    for p in [points[0], points[points.len() - 1]] {
        painter.circle_filled(p, width / 2.0, color);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Play,
    Pause,
    Reset,
    Skip,
    Settings,
    Back,
}

/// Round icon button painted inside `rect`.
pub fn icon_button(ui: &mut Ui, rect: Rect, icon: Icon, fg: Color32, bg: Color32) -> Response {
    let id = ui.make_persistent_id(("icon_button", icon, rect.center().x as i32));
    let response = ui.interact(rect, id, Sense::click());
    let hover = ui.ctx().animate_bool(id, response.hovered());
    let pressed = response.is_pointer_button_down_on();

    let painter = ui.painter();
    let c = rect.center();
    let r = rect.width().min(rect.height()) / 2.0;
    let grow = if pressed { 0.96 } else { 1.0 + 0.05 * hover };
    if bg != Color32::TRANSPARENT {
        painter.circle_filled(c, r * grow, bg);
    } else if hover > 0.0 {
        painter.circle_filled(c, r * grow, fg.gamma_multiply(0.08 * hover));
    }
    paint_icon(painter, icon, c, r * 0.42, fg);

    response.on_hover_cursor(CursorIcon::PointingHand)
}

pub fn paint_icon(painter: &Painter, icon: Icon, c: Pos2, s: f32, color: Color32) {
    let stroke = Stroke::new((s * 0.2).max(1.5), color);
    match icon {
        Icon::Play => {
            let x = c.x - s * 0.4;
            let tri = vec![
                pos2(x, c.y - s * 0.7),
                pos2(x + s * 1.15, c.y),
                pos2(x, c.y + s * 0.7),
            ];
            painter.add(Shape::convex_polygon(
                tri,
                color,
                Stroke::new(s * 0.12, color),
            ));
        }
        Icon::Pause => {
            let size = vec2(s * 0.36, s * 1.3);
            for dx in [-s * 0.32, s * 0.32] {
                let rect = Rect::from_center_size(c + vec2(dx, 0.0), size);
                painter.rect_filled(rect, CornerRadius::same((s * 0.12) as u8), color);
            }
        }
        Icon::Skip => {
            let x = c.x - s * 0.6;
            let tri = vec![
                pos2(x, c.y - s * 0.6),
                pos2(x + s * 0.95, c.y),
                pos2(x, c.y + s * 0.6),
            ];
            painter.add(Shape::convex_polygon(
                tri,
                color,
                Stroke::new(s * 0.1, color),
            ));
            let bar = Rect::from_center_size(pos2(c.x + s * 0.55, c.y), vec2(s * 0.24, s * 1.25));
            painter.rect_filled(bar, CornerRadius::same((s * 0.1) as u8), color);
        }
        Icon::Reset => {
            let r = s * 0.72;
            let (a0, a1) = (-TAU * 0.20, TAU * 0.62);
            let points: Vec<Pos2> = (0..=32)
                .map(|i| {
                    let a = a0 + (a1 - a0) * i as f32 / 32.0;
                    c + r * vec2(a.cos(), a.sin())
                })
                .collect();
            painter.add(PathShape::line(points, stroke));
            // Arrow head at the start of the arc, pointing "backwards".
            let tip = c + r * vec2(a0.cos(), a0.sin());
            let h = s * 0.42;
            let head = vec![
                tip + vec2(-h * 0.2, -h),
                tip + vec2(h * 0.95, h * 0.05),
                tip + vec2(-h * 0.35, h * 0.6),
            ];
            painter.add(Shape::convex_polygon(head, color, Stroke::NONE));
        }
        Icon::Settings => {
            let w = s * 0.95;
            for (i, knob) in [(-1.0, 0.35), (0.0, -0.4), (1.0, 0.1)] {
                let y = c.y + i * s * 0.6;
                painter.line_segment(
                    [pos2(c.x - w, y), pos2(c.x + w, y)],
                    Stroke::new(stroke.width * 0.75, color),
                );
                painter.circle_filled(pos2(c.x + knob * w, y), s * 0.22, color);
            }
        }
        Icon::Back => {
            let pts = vec![
                pos2(c.x + s * 0.3, c.y - s * 0.7),
                pos2(c.x - s * 0.4, c.y),
                pos2(c.x + s * 0.3, c.y + s * 0.7),
            ];
            painter.add(PathShape::line(pts, stroke));
        }
    }
}

/// Rounded tab-like toggle.
pub fn pill(ui: &mut Ui, text: &str, selected: bool, accent: Color32, theme: &Theme) -> Response {
    let (fill, color) = if selected {
        (mix(theme.background, accent, 0.18), accent)
    } else {
        (Color32::TRANSPARENT, theme.muted)
    };
    ui.add(
        Button::new(RichText::new(text).size(13.0).color(color))
            .fill(fill)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(255)),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
}

/// A padded rounded card used to group settings.
pub fn card<R>(ui: &mut Ui, theme: &Theme, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::new()
        .fill(theme.surface)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(14)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}
