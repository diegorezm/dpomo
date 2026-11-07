//! Glue between config, timer and views. Owns all mutable state.

use std::net::TcpListener;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use egui::{Key, Modifiers, ViewportCommand};

use crate::character::{CharacterImage, Characters};
use crate::command::Command;
use crate::config::{Config, Layout};
use crate::theme::{Theme, Themes};
use crate::timer::{Phase, Timer, TimerEvent, format_remaining};
use crate::tray::{Tray, TrayState};
use crate::ui::{cozy_view, settings_view, timer_view};
use crate::{instance, system};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum View {
    Timer,
    Settings,
}

pub struct DpomoApp {
    config: Config,
    /// Last state written to disk, to avoid pointless writes.
    saved: Config,
    themes: Themes,
    characters: Characters,
    character_image: CharacterImage,
    timer: Timer,
    view: View,
    settings: settings_view::State,
    ring: timer_view::RingAnim,
    /// Cache of what was pushed to egui / the window, so we only resend on change.
    applied_theme: Option<(Theme, f32)>,
    applied_on_top: Option<bool>,
    title: String,
    /// Commands from the tray and from later launches (see `command.rs`).
    commands: (Sender<Command>, Receiver<Command>),
    tray: Option<Tray>,
    window_visible: bool,
}

impl DpomoApp {
    /// `listener` is the single-instance socket, if we own it.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        config: Config,
        listener: Option<TcpListener>,
        start_hidden: bool,
    ) -> Self {
        install_fonts(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let commands = channel();
        if let Some(listener) = listener {
            instance::listen(listener, cc.egui_ctx.clone(), commands.0.clone());
        }
        Self {
            commands,
            tray: None,
            window_visible: !start_hidden,
            themes: Themes::load(),
            characters: Characters::load(),
            character_image: CharacterImage::default(),
            timer: Timer::new(&config.timer),
            settings: settings_view::State::default(),
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
        let TimerEvent::Finished { finished, next } = event;
        if self.config.behavior.system_notification {
            let started = self.timer.is_running();
            system::notify_finished(finished, next, started, &self.config.timer);
        }
        let time = ctx.input(|i| i.time);
        self.ring.restart(0.0, time);
        if self.config.behavior.notify_on_finish {
            ctx.send_viewport_cmd(ViewportCommand::RequestUserAttention(
                egui::UserAttentionType::Informational,
            ));
        }
        if self.config.behavior.focus_on_finish {
            self.set_window_visible(ctx, true);
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
                } else if pressed(Key::Num1) {
                    Some(timer_view::Action::SwitchTo(Phase::Focus))
                } else if pressed(Key::Num2) {
                    Some(timer_view::Action::SwitchTo(Phase::ShortBreak))
                } else if pressed(Key::Num3) {
                    Some(timer_view::Action::SwitchTo(Phase::LongBreak))
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

    fn apply_settings_action(&mut self, action: settings_view::Action) {
        use settings_view::Action;
        match action {
            Action::Back => self.leave_settings(),
            Action::ResetDefaults => self.config = Config::default(),
            Action::ReloadThemes => {
                self.themes = Themes::load();
                self.characters = Characters::load();
                self.settings.notice = Some(format!(
                    "{} temas e {} personagens carregados",
                    self.themes.iter().count(),
                    self.characters.iter().count()
                ));
            }
            Action::ExportCustom => {
                let name = self.settings.export_name.trim().to_owned();
                self.settings.notice =
                    Some(match Themes::export(&name, &self.config.custom_theme) {
                        Ok(path) => {
                            self.themes = Themes::load();
                            self.config.appearance.theme = name;
                            format!("Tema salvo em {}", path.display())
                        }
                        Err(e) => format!("Não foi possível salvar: {e}"),
                    });
            }
        }
    }

    fn leave_settings(&mut self) {
        self.settings.notice = None;
        self.view = View::Timer;
        self.apply_autostart();
        self.save();
    }

    /// Touch the OS autostart entry only when the setting actually changed.
    fn apply_autostart(&mut self) {
        let wanted = self.config.behavior.start_with_system;
        if wanted == self.saved.behavior.start_with_system {
            return;
        }
        if let Err(e) = system::set_autostart(wanted) {
            self.config.behavior.start_with_system = !wanted;
            self.settings.system_error = Some(format!("Iniciar com o sistema falhou: {e}"));
        }
    }

    fn set_window_visible(&mut self, ctx: &egui::Context, visible: bool) {
        self.window_visible = visible;
        ctx.send_viewport_cmd(ViewportCommand::Visible(visible));
        if visible {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }
    }

    fn handle_commands(&mut self, ctx: &egui::Context, now: Instant) {
        while let Ok(cmd) = self.commands.1.try_recv() {
            match cmd {
                Command::ShowWindow => self.set_window_visible(ctx, true),
                Command::ToggleWindow => self.set_window_visible(ctx, !self.window_visible),
                Command::ToggleTimer => {
                    self.apply_timer_action(ctx, timer_view::Action::Toggle, now)
                }
                Command::Skip => self.apply_timer_action(ctx, timer_view::Action::Skip, now),
                Command::Reset => self.apply_timer_action(ctx, timer_view::Action::Reset, now),
                Command::Quit => self.quit(),
            }
        }
    }

    fn quit(&mut self) -> ! {
        self.apply_autostart();
        self.save();
        // Drop the tray first so Windows doesn't leave a ghost icon behind.
        self.tray = None;
        std::process::exit(0);
    }

    /// Create / remove the tray icon to match the config, and refresh it.
    fn sync_tray(&mut self, ctx: &egui::Context, now: Instant) {
        if !self.config.behavior.tray {
            self.tray = None;
            return;
        }
        if self.tray.is_none() {
            match Tray::new(ctx, self.commands.0.clone()) {
                Ok(tray) => {
                    self.tray = Some(tray);
                    self.settings.system_error = None;
                }
                Err(e) => {
                    eprintln!("dpomo: tray unavailable: {e}");
                    self.settings.system_error = Some(format!("Bandeja indisponível: {e}"));
                    self.config.behavior.tray = false;
                    // Never stay hidden without a way back.
                    if !self.window_visible {
                        self.set_window_visible(ctx, true);
                    }
                    return;
                }
            }
        }

        let phase = self.timer.phase();
        let tooltip = if self.timer.is_pristine() {
            format!("dpomo · {}", phase.label())
        } else {
            let paused = if self.timer.is_running() {
                ""
            } else {
                " (pausado)"
            };
            format!(
                "dpomo · {} {}{paused}",
                format_remaining(self.timer.remaining(now)),
                phase.label()
            )
        };
        let state = TrayState {
            tooltip,
            progress: self.timer.progress(now),
            running: self.timer.is_running(),
            window_visible: self.window_visible,
            accent: self.config.theme(&self.themes).accent(phase),
        };
        if let Some(tray) = &mut self.tray {
            tray.update(state);
        }
    }

    /// Push theme / window settings to egui only when they change.
    fn sync_environment(&mut self, ctx: &egui::Context) {
        let theme = self.config.theme(&self.themes);
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
        let visible =
            self.window_visible && !ctx.input(|i| i.viewport().minimized.unwrap_or(false));
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
        self.handle_commands(ctx, now);
        self.timer.sync_settings(&self.config.timer);
        if let Some(event) = self.timer.tick(now, &self.config.timer) {
            self.on_timer_event(ctx, event);
        }
        self.sync_tray(ctx, now);
        self.update_title(ctx, now);
        self.schedule_repaint(ctx, now);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let now = Instant::now();
        self.sync_environment(&ctx);
        self.handle_shortcuts(&ctx, now);

        if ctx.input(|i| i.viewport().close_requested()) {
            if self.tray.is_some() && self.config.behavior.close_to_tray {
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                self.set_window_visible(&ctx, false);
                if self.view == View::Settings {
                    self.leave_settings();
                }
            } else {
                self.quit();
            }
        }

        let theme = self.config.theme(&self.themes);
        let cozy = self.config.appearance.layout == Layout::Cozy && self.view == View::Timer;
        let fill = if cozy {
            cozy_view::backdrop(&theme, self.timer.phase())
        } else {
            theme.background
        };
        let frame = egui::Frame::new()
            .fill(fill)
            .inner_margin(if cozy { 14 } else { 16 });
        egui::CentralPanel::default()
            .frame(frame)
            .show(ui, |ui| match self.view {
                View::Timer => {
                    let action = match self.config.appearance.layout {
                        Layout::Cozy => cozy_view::show(
                            ui,
                            cozy_view::Inputs {
                                timer: &self.timer,
                                config: &self.config,
                                theme: &theme,
                                anim: &self.ring,
                                character: self.characters.get(&self.config.appearance.character),
                                image: &mut self.character_image,
                                now,
                            },
                        ),
                        Layout::Classic => {
                            timer_view::show(ui, &self.timer, &self.config, &theme, &self.ring, now)
                        }
                    };
                    if let Some(a) = action {
                        self.apply_timer_action(&ctx, a, now);
                    }
                }
                View::Settings => {
                    let action = settings_view::show(
                        ui,
                        &mut self.config,
                        &self.themes,
                        &self.characters,
                        &mut self.settings,
                        &theme,
                    );
                    if let Some(a) = action {
                        self.apply_settings_action(a);
                    }
                }
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
        let theme = self.config.theme(&self.themes);
        let color = match self.config.appearance.layout {
            Layout::Cozy => cozy_view::backdrop(&theme, self.timer.phase()),
            Layout::Classic => theme.background,
        };
        color.to_normalized_gamma_f32()
    }
}

/// Adds the chunky display font (Lilita One, OFL) as the `display` family,
/// falling back to the default fonts for missing glyphs.
fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "lilita".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/LilitaOne-Regular.ttf"
        ))),
    );
    let mut family = vec!["lilita".to_owned()];
    family.extend(
        fonts.families[&egui::FontFamily::Proportional]
            .iter()
            .cloned(),
    );
    fonts.families.insert(
        egui::FontFamily::Name(cozy_view::DISPLAY_FONT.into()),
        family,
    );
    ctx.set_fonts(fonts);
}
