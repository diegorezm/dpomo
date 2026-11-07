//! The main screen: phase tabs, progress ring and playback controls.

use std::time::Instant;

use egui::text::{LayoutJob, TextFormat};
use egui::{Align, Align2, FontId, Layout, Rect, Ui, UiBuilder, pos2, vec2};

use super::widgets::{self, Icon};
use crate::config::Config;
use crate::theme::{Theme, mix};
use crate::timer::{Phase, Timer, format_remaining};

/// What the user asked for this frame. Applied by `App`, never here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Toggle,
    Reset,
    Skip,
    SwitchTo(Phase),
    OpenSettings,
}

/// Eases the ring between two values after a jump (reset, phase change).
#[derive(Default)]
pub struct RingAnim {
    from: f32,
    start: f64,
}

impl RingAnim {
    const SECS: f64 = 0.6;

    pub fn restart(&mut self, from: f32, now: f64) {
        self.from = from;
        self.start = now;
    }

    pub fn value(&self, target: f32, now: f64) -> (f32, bool) {
        let t = ((now - self.start) / Self::SECS).clamp(0.0, 1.0) as f32;
        let eased = 1.0 - (1.0 - t).powi(3);
        (self.from + (target - self.from) * eased, t < 1.0)
    }
}

pub fn show(
    ui: &mut Ui,
    timer: &Timer,
    config: &Config,
    theme: &Theme,
    anim: &RingAnim,
    now: Instant,
) -> Option<Action> {
    let mut action = None;
    let full = ui.max_rect();
    let accent = theme.accent(timer.phase());

    // --- Top bar ---------------------------------------------------------
    let top = Rect::from_min_size(full.min, vec2(full.width(), 32.0));
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(top)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for phase in Phase::ALL {
                let selected = phase == timer.phase();
                if widgets::pill(
                    ui,
                    phase.short_label(),
                    selected,
                    theme.accent(phase),
                    theme,
                )
                .clicked()
                    && !selected
                {
                    action = Some(Action::SwitchTo(phase));
                }
            }
        },
    );
    let gear = Rect::from_center_size(pos2(top.right() - 16.0, top.center().y), vec2(32.0, 32.0));
    if widgets::icon_button(
        ui,
        gear,
        Icon::Settings,
        theme.muted,
        egui::Color32::TRANSPARENT,
    )
    .on_hover_text("Ajustes")
    .clicked()
    {
        action = Some(Action::OpenSettings);
    }

    // --- Ring ------------------------------------------------------------
    let controls_h = 96.0;
    let area = Rect::from_min_max(
        pos2(full.left(), top.bottom() + 8.0),
        pos2(full.right(), full.bottom() - controls_h),
    );
    let thickness = config.appearance.ring_thickness;
    let radius = (area.width().min(area.height()) / 2.0 - thickness * 1.5).max(40.0);
    let center = area.center();
    let time = ui.input(|i| i.time);
    let (progress, _) = anim.value(timer.progress(now), time);

    let painter = ui.painter();
    let track = mix(theme.background, theme.surface, 0.9);
    widgets::ring(
        painter,
        center,
        radius,
        thickness,
        progress,
        track,
        accent,
        config.appearance.glow,
    );

    // Phase label (letter-spaced).
    let mut job = LayoutJob::default();
    job.append(
        &timer.phase().label().to_uppercase(),
        0.0,
        TextFormat {
            font_id: FontId::proportional((radius * 0.085).max(10.0)),
            color: theme.muted,
            extra_letter_spacing: 2.0,
            ..Default::default()
        },
    );
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let label_pos = center - vec2(galley.size().x / 2.0, radius * 0.42);
    painter.galley(label_pos, galley, theme.muted);

    // Remaining time.
    let paused = !timer.is_running() && !timer.is_pristine();
    let text_color = if paused { theme.muted } else { theme.text };
    painter.text(
        center,
        Align2::CENTER_CENTER,
        format_remaining(timer.remaining(now)),
        FontId::proportional(radius * 0.42),
        text_color,
    );

    // Session dots.
    if config.appearance.show_session_dots {
        let total = config.timer.sessions_before_long_break.clamp(1, 12);
        let done = match timer.phase() {
            Phase::LongBreak => total,
            _ => timer.completed_focus() % total,
        };
        let dot = (radius * 0.035).clamp(3.0, 6.0);
        let gap = dot * 3.2;
        let y = center.y + radius * 0.36;
        let x0 = center.x - gap * (total as f32 - 1.0) / 2.0;
        for i in 0..total {
            let p = pos2(x0 + gap * i as f32, y);
            if i < done {
                painter.circle_filled(p, dot, accent);
            } else {
                painter.circle_stroke(
                    p,
                    dot - 0.75,
                    egui::Stroke::new(1.5, mix(theme.background, theme.muted, 0.5)),
                );
            }
        }
    }

    // --- Controls --------------------------------------------------------
    let cy = full.bottom() - controls_h / 2.0;
    let cx = full.center().x;
    let side_bg = theme.surface;
    let main = Rect::from_center_size(pos2(cx, cy), vec2(64.0, 64.0));
    let left = Rect::from_center_size(pos2(cx - 76.0, cy), vec2(44.0, 44.0));
    let right = Rect::from_center_size(pos2(cx + 76.0, cy), vec2(44.0, 44.0));

    if widgets::icon_button(ui, left, Icon::Reset, theme.muted, side_bg)
        .on_hover_text("Reiniciar (R)")
        .clicked()
    {
        action = Some(Action::Reset);
    }
    let main_icon = if timer.is_running() {
        Icon::Pause
    } else {
        Icon::Play
    };
    if widgets::icon_button(ui, main, main_icon, theme.background, accent)
        .on_hover_text("Iniciar / pausar (Espaço)")
        .clicked()
    {
        action = Some(Action::Toggle);
    }
    if widgets::icon_button(ui, right, Icon::Skip, theme.muted, side_bg)
        .on_hover_text("Pular (S)")
        .clicked()
    {
        action = Some(Action::Skip);
    }

    action
}
