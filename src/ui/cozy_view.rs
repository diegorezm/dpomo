//! The "cozy" main screen: phase pills, the character, a big chunky timer,
//! a slim progress bar and small controls, filling the whole window.

use std::time::Instant;

use egui::{
    Align2, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Rect, Sense, Stroke, StrokeKind,
    Ui, pos2, vec2,
};

use super::timer_view::{Action, RingAnim};
use super::widgets::{self, Icon};
use crate::character::{Character, CharacterImage, Pose};
use crate::config::Config;
use crate::i18n::{fill, tr};
use crate::theme::{Theme, mix};
use crate::timer::{Phase, Timer, format_remaining};

pub const DISPLAY_FONT: &str = "display";

pub fn display_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(DISPLAY_FONT.into()))
}

pub struct Inputs<'a> {
    pub timer: &'a Timer,
    pub config: &'a Config,
    pub theme: &'a Theme,
    pub anim: &'a RingAnim,
    pub character: Option<&'a Character>,
    pub image: &'a mut CharacterImage,
    pub now: Instant,
}

pub fn show(ui: &mut Ui, input: Inputs<'_>) -> Option<Action> {
    let Inputs {
        timer,
        config,
        theme,
        anim,
        character,
        image,
        now,
    } = input;
    let mut action = None;
    let accent = theme.accent(timer.phase());
    let time = ui.input(|i| i.time);

    // The view fills the whole window (App gives it a theme-background frame).
    let card = ui.max_rect();
    let painter = ui.painter().clone();
    let inner = card.shrink(16.0);

    // --- Pills + settings -------------------------------------------------
    let top = Rect::from_min_size(inner.min, vec2(inner.width(), 30.0));
    let pills: Vec<_> = Phase::ALL
        .iter()
        .map(|p| (*p, tr().phase_tab(*p)))
        .collect();
    let font = display_font(14.0);
    let widths: Vec<f32> = pills
        .iter()
        .map(|(_, l)| {
            ui.fonts_mut(|f| f.layout_no_wrap(l.to_string(), font.clone(), Color32::WHITE))
                .size()
                .x
                + 24.0
        })
        .collect();
    let gap = 8.0;
    let total: f32 = widths.iter().sum::<f32>() + gap * (pills.len() as f32 - 1.0);
    let mut x = top.center().x - total / 2.0;
    for ((phase, label), w) in pills.iter().zip(widths) {
        let rect = Rect::from_min_size(pos2(x, top.top()), vec2(w, 28.0));
        x += w + gap;
        let selected = *phase == timer.phase();
        let resp = ui.interact(
            rect,
            ui.make_persistent_id(("cozy_pill", *phase)),
            Sense::click(),
        );
        let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
        let (fill, text) = if selected {
            (theme.accent(*phase), theme.background)
        } else {
            (mix(theme.background, theme.text, 0.06 * hover), theme.text)
        };
        painter.rect(
            rect,
            CornerRadius::same(255),
            fill,
            Stroke::new(1.5, mix(theme.background, theme.text, 0.55)),
            StrokeKind::Inside,
        );
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            *label,
            font.clone(),
            text,
        );
        if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() && !selected {
            action = Some(Action::SwitchTo(*phase));
        }
    }
    let gear = Rect::from_center_size(
        pos2(card.right() - 26.0, card.top() + 26.0),
        vec2(28.0, 28.0),
    );
    if widgets::icon_button(ui, gear, Icon::Settings, theme.muted, Color32::TRANSPARENT)
        .on_hover_text(fill(tr().hint_settings, &[("key", &config.keys.settings)]))
        .clicked()
    {
        action = Some(Action::OpenSettings);
    }
    let mini = Rect::from_center_size(pos2(card.left() + 26.0, gear.center().y), gear.size());
    if widgets::icon_button(ui, mini, Icon::Mini, theme.muted, Color32::TRANSPARENT)
        .on_hover_text(fill(tr().hint_mini, &[("key", &config.keys.mini)]))
        .clicked()
    {
        action = Some(Action::ToggleMini);
    }

    // --- Vertical budget ---------------------------------------------------
    let controls_h = 48.0;
    let time_size = (inner.height() * 0.13).clamp(30.0, 72.0);
    let bar_h = 6.0;
    let dots_h = if config.appearance.show_session_dots {
        14.0
    } else {
        0.0
    };
    let below_char = time_size * 1.05 + 10.0 + bar_h + dots_h + 10.0 + controls_h;
    let char_area = Rect::from_min_max(
        pos2(inner.left(), top.bottom() + 8.0),
        pos2(inner.right(), inner.bottom() - below_char),
    );

    // --- Character ---------------------------------------------------------
    if let Some(character) = character {
        // Keep the 200x160 art ratio; snap the size so window resizes don't
        // re-rasterize the SVG on every pixel.
        let w = (char_area.width().min(char_area.height() * 1.25) * 0.95 / 16.0).floor() * 16.0;
        if w >= 48.0 {
            let size = vec2(w, w / 1.25);
            let bob = if timer.is_running() && config.appearance.smooth_animation {
                (time * 2.2).sin() as f32 * 2.5
            } else {
                0.0
            };
            // Soft spot so the art reads well on any card color.
            let spot = mix(theme.background, theme.text, 0.05);
            painter.circle_filled(
                char_area.center() + vec2(0.0, size.y * 0.04),
                size.y * 0.58,
                spot,
            );
            let rect = Rect::from_center_size(char_area.center() + vec2(0.0, bob), size);
            let pose = Pose::for_timer(timer);
            image
                .image(ui.ctx(), character, pose, accent, theme.muted)
                .paint_at(ui, rect);
        }
    }

    // --- Time --------------------------------------------------------------
    let mut y = char_area.bottom() + 4.0;
    let paused = !timer.is_running() && !timer.is_pristine();
    let time_color = if paused { theme.muted } else { theme.text };
    painter.text(
        pos2(inner.center().x, y),
        Align2::CENTER_TOP,
        format_remaining(timer.remaining(now)),
        display_font(time_size),
        time_color,
    );
    y += time_size * 1.05 + 10.0;

    // --- Progress bar --------------------------------------------------------
    let bar_w = (inner.width() * 0.55).min(260.0);
    let bar = Rect::from_min_size(pos2(inner.center().x - bar_w / 2.0, y), vec2(bar_w, bar_h));
    let (progress, _) = anim.value(timer.progress(now), time);
    painter.rect_filled(
        bar,
        CornerRadius::same(255),
        mix(theme.background, theme.text, 0.1),
    );
    let done = 1.0 - progress.clamp(0.0, 1.0);
    if done > 0.0 {
        let fill = Rect::from_min_size(bar.min, vec2((bar_w * done).max(bar_h), bar_h));
        painter.rect_filled(fill, CornerRadius::same(255), accent);
    }
    y += bar_h;

    // --- Session dots --------------------------------------------------------
    if config.appearance.show_session_dots {
        let total = config.timer.sessions_before_long_break.clamp(1, 12);
        let filled = match timer.phase() {
            Phase::LongBreak => total,
            _ => timer.completed_focus() % total,
        };
        let r = 3.0;
        let gap = 11.0;
        let cy = y + 9.0;
        let x0 = inner.center().x - gap * (total as f32 - 1.0) / 2.0;
        for i in 0..total {
            let p = pos2(x0 + gap * i as f32, cy);
            if i < filled {
                painter.circle_filled(p, r, accent);
            } else {
                painter.circle_stroke(
                    p,
                    r - 0.5,
                    Stroke::new(1.3, mix(theme.background, theme.muted, 0.6)),
                );
            }
        }
    }

    // --- Controls ------------------------------------------------------------
    let cy = inner.bottom() - controls_h / 2.0;
    let cx = inner.center().x;
    let fg = theme.text;
    let none = Color32::TRANSPARENT;
    let reset = Rect::from_center_size(pos2(cx - 50.0, cy), vec2(36.0, 36.0));
    let main = Rect::from_center_size(pos2(cx, cy), vec2(46.0, 46.0));
    let skip = Rect::from_center_size(pos2(cx + 50.0, cy), vec2(36.0, 36.0));
    if widgets::icon_button(ui, reset, Icon::Reset, fg, none)
        .on_hover_text(fill(tr().hint_reset, &[("key", &config.keys.reset)]))
        .clicked()
    {
        action = Some(Action::Reset);
    }
    let icon = if timer.is_running() {
        Icon::Pause
    } else {
        Icon::Play
    };
    if widgets::icon_button(ui, main, icon, fg, none)
        .on_hover_text(fill(tr().hint_toggle, &[("key", &config.keys.toggle)]))
        .clicked()
    {
        action = Some(Action::Toggle);
    }
    if widgets::icon_button(ui, skip, Icon::Skip, fg, none)
        .on_hover_text(fill(tr().hint_skip, &[("key", &config.keys.skip)]))
        .clicked()
    {
        action = Some(Action::Skip);
    }

    action
}
