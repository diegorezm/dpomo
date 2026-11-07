//! Glue between config, timer and views. Owns all mutable state.

use std::time::{Duration, Instant};

use egui::{Key, Modifiers, ViewportCommand};

use crate::config::Config;
use crate::theme::Theme;
use crate::timer::{Timer, TimerEvent, format_remaining};
use crate::ui::{settings_view, timer_view};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum View {
    Timer,
    Settings,
}

pub struct DpomoApp {
    config: Config,
    /// Last state written to disk, to avoid pointless writes.
    saved: Config,
    timer: Timer,
    view: View,
    ring: timer_view::RingAnim,
    /// Cache of what was pushed to egui / the window, so we only resend on change.
    applied_theme: Option<(Theme, f32)>,
    applied_on_top: Option<bool>,
    title: String,
}

impl DpomoApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        Self {
            timer: Timer::new(&config.timer),
            saved: config.clone(),
            config,
            view: View::Timer,
            ring: timer_view::RingAnim::default(),
            applied_theme: None,
            applied_on_top: None,
            title: String::new(),
        }
    }

    fn save(&mut self) {
        if self.config != self.saved {
            if let Err(e) = self.config.save() {
                eprintln!("dpomo: could not save config: {e}");
            }
            self.saved = self.config.clone();
        }
    }

    /// Run a timer mutation and animate the ring from where it was.
    fn with_ring_anim(
        &mut self,
        ctx: &egui::Context,
        now: Instant,
        f: impl FnOnce(&mut Timer, &Config),
    ) {
        let before = self.timer.progress(now);
        f(&mut self.timer, &self.config);
        let time = ctx.input(|i| i.time);
        self.ring.restart(before, time);
    }

    fn apply_timer_action(
        &mut self,
        ctx: &egui::Context,
        action: timer_view::Action,
        now: Instant,
    ) {
        use timer_view::Action;
        match action {
            Action::Toggle => self.timer.toggle(now),
            Action::Reset => self.with_ring_anim(ctx, now, |t, c| t.reset(&c.timer)),
            Action::Skip => self.with_ring_anim(ctx, now, |t, c| t.skip(&c.timer)),
            Action::SwitchTo(p) => self.with_ring_anim(ctx, now, |t, c| t.switch_to(p, &c.timer)),
            Action::OpenSettings => self.view = View::Settings,
        }
    }

    fn on_timer_event(&mut self, ctx: &egui::Context, event: TimerEvent) {
        let TimerEvent::Finished { .. } = event;
        let time = ctx.input(|i| i.time);
        self.ring.restart(0.0, time);
        if self.config.behavior.notify_on_finish {
            ctx.send_viewport_cmd(ViewportCommand::RequestUserAttention(
                egui::UserAttentionType::Informational,
            ));
        }
        if self.config.behavior.focus_on_finish {
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context, now: Instant) {
        let pressed = |key| ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key));
        match self.view {
            View::Timer => {
                let action = if pressed(Key::Space) {
                    Some(timer_view::Action::Toggle)
                } else if pressed(Key::R) {
                    Some(timer_view::Action::Reset)
                } else if pressed(Key::S) {
                    Some(timer_view::Action::Skip)
                } else if pressed(Key::Comma) {
                    Some(timer_view::Action::OpenSettings)
                } else {
                    None
                };
                if let Some(a) = action {
                    self.apply_timer_action(ctx, a, now);
                }
            }
            View::Settings => {
                if pressed(Key::Escape) {
                    self.leave_settings();
                }
            }
        }
    }

    fn leave_settings(&mut self) {
        self.view = View::Timer;
        self.save();
    }

    /// Push theme / window settings to egui only when they change.
    fn sync_environment(&mut self, ctx: &egui::Context) {
        let theme = self.config.theme();
        let scale = self.config.appearance.ui_scale.clamp(0.5, 3.0);
        if self.applied_theme != Some((theme, scale)) {
            theme.apply(ctx);
            ctx.set_zoom_factor(scale);
            self.applied_theme = Some((theme, scale));
        }

        let on_top = self.config.behavior.always_on_top;
        if self.applied_on_top != Some(on_top) {
            let level = if on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            };
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(level));
            self.applied_on_top = Some(on_top);
        }
    }

    fn update_title(&mut self, ctx: &egui::Context, now: Instant) {
        let title = if self.config.behavior.time_in_title && !self.timer.is_pristine() {
            format!(
                "{} · {}",
                format_remaining(self.timer.remaining(now)),
                self.timer.phase().label()
            )
        } else {
            "dpomo".to_owned()
        };
        if title != self.title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    /// The app is idle (zero CPU) unless the timer runs or an animation plays.
    fn schedule_repaint(&self, ctx: &egui::Context, now: Instant) {
        if !self.timer.is_running() {
            return;
        }
        let visible = !ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        if visible && self.config.appearance.smooth_animation {
            ctx.request_repaint_after(Duration::from_millis(33));
        } else {
            // Wake exactly when the displayed second changes.
            let sub = self.timer.remaining(now).subsec_nanos();
            let wait = if sub == 0 {
                Duration::from_secs(1)
            } else {
                Duration::from_nanos(u64::from(sub))
            };
            ctx.request_repaint_after(wait + Duration::from_millis(1));
        }
    }
}

impl eframe::App for DpomoApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        self.timer.sync_settings(&self.config.timer);
        if let Some(event) = self.timer.tick(now, &self.config.timer) {
            self.on_timer_event(ctx, event);
        }
        self.update_title(ctx, now);
        self.schedule_repaint(ctx, now);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let now = Instant::now();
        self.sync_environment(&ctx);
        self.handle_shortcuts(&ctx, now);

        if ctx.input(|i| i.viewport().close_requested()) {
            self.save();
        }

        let theme = self.config.theme();
        let frame = egui::Frame::new().fill(theme.background).inner_margin(16);
        egui::CentralPanel::default()
            .frame(frame)
            .show(ui, |ui| match self.view {
                View::Timer => {
                    if let Some(a) =
                        timer_view::show(ui, &self.timer, &self.config, &theme, &self.ring, now)
                    {
                        self.apply_timer_action(&ctx, a, now);
                    }
                }
                View::Settings => match settings_view::show(ui, &mut self.config, &theme) {
                    Some(settings_view::Action::Back) => self.leave_settings(),
                    Some(settings_view::Action::ResetDefaults) => self.config = Config::default(),
                    None => {}
                },
            });

        // Keep the ring easing animation going.
        let (_, animating) = self
            .ring
            .value(self.timer.progress(now), ctx.input(|i| i.time));
        if animating {
            ctx.request_repaint();
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.config.theme().background.to_normalized_gamma_f32()
    }
}
