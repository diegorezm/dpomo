//! The floating timer: a small borderless always-on-top window with the time
//! left, the phase and (optionally) the character. `App::show_mini` puts it in
//! its own viewport; this only draws it. Everything scales with the window,
//! from a 120×44 sliver up to as big as the user drags it.

use std::time::Instant;

use egui::{
    Align2, Color32, CornerRadius, CursorIcon, Rect, ResizeDirection, Sense, Stroke, Ui,
    ViewportCommand, pos2, vec2,
};

use super::cozy_view::display_font;
use super::widgets::{self, Icon};
use crate::character::{Character, CharacterImage, Pose};
use crate::config::Config;
use crate::i18n::{fill, tr};
use crate::theme::{Theme, mix};
use crate::timer::{Timer, format_remaining};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Toggle,
    Skip,
    OpenMain,
    Close,
}

pub struct Inputs<'a> {
    pub timer: &'a Timer,
    pub config: &'a Config,
    pub theme: &'a Theme,
    pub character: Option<&'a Character>,
    pub image: &'a mut CharacterImage,
    pub now: Instant,
}

pub fn show(ui: &mut Ui, input: Inputs<'_>) -> Option<Action> {
    let Inputs {
        timer,
        config,
        theme,
        character,
        image,
        now,
    } = input;
    let mut action = None;
    let rect = ui.max_rect();
    let painter = ui.painter().clone();
    let accent = theme.accent(timer.phase());
    let t = tr();

    // The whole window: drag to move it, double-click to open pomolotl.
    // Registered first so the buttons below win over it.
    let body = ui.interact(
        rect,
        ui.scope_id().with("mini_body"),
        Sense::click_and_drag(),
    );
    if body.drag_started() {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }
    if body.double_clicked() {
        action = Some(Action::OpenMain);
    }
    let hovered = ui
        .ctx()
        .input(|i| i.pointer.hover_pos())
        .is_some_and(|p| rect.contains(p));
    let hover = ui
        .ctx()
        .animate_bool(ui.scope_id().with("mini_hover"), hovered);

    let h = rect.height();
    let pad = (h * 0.12).clamp(6.0, 14.0);

    // --- Progress bar along the bottom ----------------------------------------
    let bar_h = (h * 0.045).clamp(3.0, 5.0);
    let bar = Rect::from_min_size(
        pos2(rect.left() + pad, rect.bottom() - pad * 0.7 - bar_h),
        vec2(rect.width() - 2.0 * pad, bar_h),
    );
    painter.rect_filled(
        bar,
        CornerRadius::same(255),
        mix(theme.background, theme.text, 0.1),
    );
    let done = 1.0 - timer.progress(now).clamp(0.0, 1.0);
    if done > 0.0 {
        let fill = Rect::from_min_size(bar.min, vec2((bar.width() * done).max(bar_h), bar_h));
        painter.rect_filled(fill, CornerRadius::same(255), accent);
    }
    let content = Rect::from_min_max(
        rect.min + vec2(pad, pad * 0.8),
        pos2(rect.right() - pad, bar.top() - pad * 0.5),
    );

    // --- Character on the left (when there's room) ------------------------------
    let mut text_area = content;
    if config.mini.show_character
        && let Some(character) = character
        && content.width() > content.height() * 2.4
    {
        // 200x160 art; snap to 8 px so resizing doesn't re-rasterize every frame.
        let art_h = ((content.height() * 1.05) / 8.0).floor() * 8.0;
        if art_h >= 24.0 {
            let size = vec2(art_h * 1.25, art_h);
            let art = Rect::from_min_size(
                pos2(content.left() - pad * 0.3, content.center().y - art_h / 2.0),
                size,
            );
            image
                .image(
                    ui.ctx(),
                    character,
                    Pose::for_timer(timer),
                    accent,
                    theme.muted,
                )
                .paint_at(ui, art);
            text_area.min.x = art.right() + pad * 0.4;
        }
    }

    // --- Time + phase -----------------------------------------------------------
    let show_label = text_area.height() >= 38.0;
    let label_size = (text_area.height() * 0.2).clamp(10.0, 18.0);
    let time_text = format_remaining(timer.remaining(now));
    let mut time_size = if show_label {
        text_area.height() - label_size * 1.4
    } else {
        text_area.height()
    }
    .max(10.0);
    // Shrink to fit the width too.
    let width = ui
        .fonts_mut(|f| f.layout_no_wrap(time_text.clone(), display_font(time_size), Color32::WHITE))
        .size()
        .x;
    if width > text_area.width() {
        time_size *= text_area.width() / width;
    }
    let block_h = time_size + if show_label { label_size * 1.4 } else { 0.0 };
    let top = text_area.center().y - block_h / 2.0;
    let paused = !timer.is_running() && !timer.is_pristine();
    let fade = 1.0 - 0.85 * hover;
    let time_color = if paused { theme.muted } else { theme.text };
    painter.text(
        pos2(text_area.center().x, top),
        Align2::CENTER_TOP,
        time_text,
        display_font(time_size),
        time_color.gamma_multiply(fade),
    );
    if show_label {
        let label = if paused {
            format!("{} {}", t.phase(timer.phase()), t.paused)
        } else {
            t.phase(timer.phase()).to_owned()
        };
        painter.text(
            pos2(text_area.center().x, top + time_size * 1.02),
            Align2::CENTER_TOP,
            label,
            egui::FontId::proportional(label_size),
            accent.gamma_multiply(fade),
        );
    }

    // --- Controls on hover --------------------------------------------------------
    if hover > 0.0 {
        let size = (h * 0.34).clamp(20.0, 34.0);
        let gap = size * 0.25;
        let mut buttons = vec![
            (
                if timer.is_running() {
                    Icon::Pause
                } else {
                    Icon::Play
                },
                Action::Toggle,
                fill(t.hint_toggle, &[("key", &config.keys.toggle)]),
            ),
            (
                Icon::Skip,
                Action::Skip,
                fill(t.hint_skip, &[("key", &config.keys.skip)]),
            ),
            (Icon::Expand, Action::OpenMain, t.mini_open_main.to_owned()),
            (Icon::Close, Action::Close, t.mini_close.to_owned()),
        ];
        let fits = |n: usize| n as f32 * size + (n as f32 - 1.0) * gap <= text_area.width();
        while buttons.len() > 1 && !fits(buttons.len()) {
            buttons.remove(buttons.len() - 2); // keep "close" last
        }
        let n = buttons.len() as f32;
        let x0 = text_area.center().x - (n * size + (n - 1.0) * gap) / 2.0 + size / 2.0;
        let fg = theme.text.gamma_multiply(hover);
        let bg = mix(theme.background, theme.text, 0.08).gamma_multiply(hover);
        for (i, (icon, act, hint)) in buttons.into_iter().enumerate() {
            let c = pos2(x0 + i as f32 * (size + gap), text_area.center().y);
            let r = Rect::from_center_size(c, vec2(size, size));
            if widgets::icon_button(ui, r, icon, fg, bg)
                .on_hover_text(hint)
                .clicked()
            {
                action = Some(act);
            }
        }

        // Resize grip in the bottom-right corner.
        let grip = Rect::from_min_max(rect.max - vec2(16.0, 16.0), rect.max);
        let resp = ui
            .interact(grip, ui.scope_id().with("mini_grip"), Sense::drag())
            .on_hover_cursor(CursorIcon::ResizeSouthEast);
        if resp.drag_started() {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::BeginResize(ResizeDirection::SouthEast));
        }
        let stroke = Stroke::new(1.5, theme.muted.gamma_multiply(hover));
        for k in [4.0, 8.0] {
            painter.line_segment(
                [
                    pos2(rect.right() - 3.0 - k, rect.bottom() - 3.0),
                    pos2(rect.right() - 3.0, rect.bottom() - 3.0 - k),
                ],
                stroke,
            );
        }
    }

    action
}
