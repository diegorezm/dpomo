//! Glue between config, timer and views. Owns all mutable state.

use std::net::TcpListener;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use egui::{Modifiers, ViewportCommand};

use crate::character::{CharacterImage, Characters};
use crate::command::Command;
use crate::config::{Config, Keys, Layout, LoadError};
use crate::i18n::{self, Language, fill, tr};
use crate::state::{self, Session, State};
use crate::theme::{Theme, Themes};
use crate::timer::{Phase, Timer, TimerEvent, format_remaining};
use crate::tray::{Tray, TrayState};
use crate::ui::{cozy_view, settings_view, timer_view};
use crate::{instance, sound, system, updates};

/// How often a running session is written to `state.toml` (crash safety).
const STATE_SAVE_INTERVAL: Duration = Duration::from_secs(30);

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
    /// Session progress + statistics (`state.toml`).
    app_state: State,
    last_state_save: Instant,
    applied_language: Option<Language>,
    update_checker_started: bool,
}

impl DpomoApp {
    /// `listener` is the single-instance socket, if we own it.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        config: Config,
        listener: Option<TcpListener>,
        start_hidden: bool,
        config_error: Option<String>,
    ) -> Self {
        install_fonts(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let commands = channel();
        if let Some(listener) = listener {
            instance::listen(listener, cc.egui_ctx.clone(), commands.0.clone());
        }
        Config::watch(cc.egui_ctx.clone(), commands.0.clone());

        // Pick up where the last run left off (same day only), paused.
        let app_state = State::load();
        let timer = match app_state.todays_session() {
            Some(s) => Timer::restore(
                s.phase,
                Duration::from_secs(s.elapsed_secs),
                s.completed_focus,
                &config.timer,
            ),
            None => Timer::new(&config.timer),
        };
        Self {
            app_state,
            last_state_save: Instant::now(),
            applied_language: None,
            update_checker_started: false,
            commands,
            tray: None,
            window_visible: !start_hidden,
            themes: Themes::load(),
            characters: Characters::load(),
            character_image: CharacterImage::default(),
            timer,
            settings: settings_view::State {
                config_error,
                ..Default::default()
            },
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
        // Never overwrite a file the user is still fixing by hand.
        if self.settings.config_error.is_some() {
            return;
        }
        if self.config != self.saved {
            if let Err(e) = self.config.save() {
                eprintln!("dpomo: could not save config: {e}");
            }
            self.saved = self.config.clone();
        }
    }

    /// Persist the current session so a restart/reboot doesn't lose it.
    fn save_state(&mut self, now: Instant) {
        self.app_state.session = Some(Session {
            day: state::today(),
            phase: self.timer.phase(),
            elapsed_secs: self.timer.elapsed(now).as_secs(),
            completed_focus: self.timer.completed_focus(),
        });
        self.app_state.save();
        self.last_state_save = now;
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
        if action != Action::OpenSettings {
            self.save_state(now);
        }
    }

    fn on_timer_event(&mut self, ctx: &egui::Context, event: TimerEvent, now: Instant) {
        let TimerEvent::Finished { finished, next } = event;
        if finished == Phase::Focus {
            let minutes = self.config.timer.focus_minutes.max(1);
            self.app_state.record_focus(state::today(), minutes);
        }
        self.save_state(now);
        let s = &self.config.sound;
        if s.enabled {
            // Rising melody into a break, falling back into focus.
            sound::play(s.kind, &s.file, s.volume, next != Phase::Focus);
        }
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
        let k = &self.config.keys;
        let d = Keys::default();
        let pressed = |name: &str, default: &str| {
            ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Keys::key(name, default)))
        };
        match self.view {
            View::Timer => {
                use timer_view::Action;
                let bindings = [
                    (&k.toggle, &d.toggle, Action::Toggle),
                    (&k.reset, &d.reset, Action::Reset),
                    (&k.skip, &d.skip, Action::Skip),
                    (&k.settings, &d.settings, Action::OpenSettings),
                    (&k.focus, &d.focus, Action::SwitchTo(Phase::Focus)),
                    (
                        &k.short_break,
                        &d.short_break,
                        Action::SwitchTo(Phase::ShortBreak),
                    ),
                    (
                        &k.long_break,
                        &d.long_break,
                        Action::SwitchTo(Phase::LongBreak),
                    ),
                ];
                let action = bindings
                    .into_iter()
                    .find(|(name, default, _)| pressed(name, default))
                    .map(|(_, _, action)| action);
                if let Some(a) = action {
                    self.apply_timer_action(ctx, a, now);
                }
            }
            View::Settings => {
                if pressed(&k.back, &d.back) {
                    self.leave_settings();
                }
            }
        }
    }

    fn apply_settings_action(&mut self, action: settings_view::Action) {
        use settings_view::Action;
        match action {
            Action::Back => self.leave_settings(),
            Action::OpenConfigFile | Action::OpenConfigFolder => {
                let path = Config::path().and_then(|p| match action {
                    Action::OpenConfigFile => Some(p),
                    _ => p.parent().map(|d| d.to_path_buf()),
                });
                if let Some(path) = path
                    && let Err(e) = system::open_path(&path)
                {
                    self.settings.notice = Some(fill(tr().open_failed, &[("e", &e.to_string())]));
                }
            }
            Action::OpenUpdatePage => {
                if let Err(e) = system::open_path(std::path::Path::new(updates::RELEASES_URL)) {
                    self.settings.notice = Some(fill(tr().open_failed, &[("e", &e.to_string())]));
                }
            }
            Action::PlaySound => {
                let s = &self.config.sound;
                sound::play(s.kind, &s.file, s.volume, true);
            }
            Action::OverwriteConfig => {
                self.settings.config_error = None;
                self.saved = Config::default();
                self.save();
            }
            Action::ResetDefaults => self.config = Config::default(),
            Action::ReloadThemes => {
                self.themes = Themes::load();
                self.characters = Characters::load();
                self.settings.notice = Some(fill(
                    tr().reloaded,
                    &[
                        ("themes", &self.themes.iter().count().to_string()),
                        ("characters", &self.characters.iter().count().to_string()),
                    ],
                ));
            }
            Action::ExportCustom => {
                let name = self.settings.export_name.trim().to_owned();
                self.settings.notice =
                    Some(match Themes::export(&name, &self.config.custom_theme) {
                        Ok(path) => {
                            self.themes = Themes::load();
                            self.config.appearance.theme = name;
                            fill(tr().theme_saved, &[("path", &path.display().to_string())])
                        }
                        Err(e) => fill(tr().save_failed, &[("e", &e.to_string())]),
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
            self.settings.system_error = Some(fill(tr().autostart_failed, &[("e", &e)]));
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
                Command::ReloadConfig => self.reload_config(),
                Command::UpdateAvailable(version) => {
                    if self.config.behavior.check_updates {
                        self.settings.update = Some(version);
                    }
                }
            }
        }
    }

    /// Apply hand edits to `config.toml`.
    fn reload_config(&mut self) {
        match Config::read() {
            Ok(config) => {
                self.settings.config_error = None;
                if config != self.saved {
                    let autostart_before = self.saved.behavior.start_with_system;
                    self.config = config.clone();
                    self.saved = config;
                    if self.config.behavior.start_with_system != autostart_before
                        && let Err(e) =
                            system::set_autostart(self.config.behavior.start_with_system)
                    {
                        self.settings.system_error =
                            Some(fill(tr().autostart_failed, &[("e", &e)]));
                    }
                }
            }
            // Deleted: write what we have back, so there's always a file to edit.
            Err(LoadError::Missing) => {
                self.saved = Config::default();
                self.save();
            }
            Err(LoadError::Invalid(e)) => self.settings.config_error = Some(e),
        }
    }

    /// Store the current window size in the config (if enabled).
    fn remember_window_size(&mut self, ctx: &egui::Context) {
        if !self.config.window.remember_size || !self.window_visible {
            return;
        }
        // Viewport rects are in ui points (affected by zoom); the config stores
        // OS logical points, which is what `with_inner_size` expects.
        let zoom = ctx.zoom_factor();
        if let Some(rect) = ctx.input(|i| i.viewport().inner_rect) {
            self.config.window.width = (rect.width() * zoom).round();
            self.config.window.height = (rect.height() * zoom).round();
        }
    }

    fn quit(&mut self) -> ! {
        self.apply_autostart();
        self.save();
        self.save_state(Instant::now());
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
                    self.settings.system_error = Some(fill(tr().tray_unavailable, &[("e", &e)]));
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
        let label = tr().phase(phase);
        let tooltip = if self.timer.is_pristine() {
            format!("dpomo · {label}")
        } else if self.timer.is_running() {
            format!(
                "dpomo · {} {label}",
                format_remaining(self.timer.remaining(now))
            )
        } else {
            format!(
                "dpomo · {} {label} {}",
                format_remaining(self.timer.remaining(now)),
                tr().paused
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
        let language = self.config.appearance.language;
        if self.applied_language != Some(language) {
            i18n::set(language);
            // Rebuild the tray so its menu picks up the new labels.
            if self.applied_language.is_some() {
                self.tray = None;
            }
            self.applied_language = Some(language);
        }

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
                tr().phase(self.timer.phase())
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

impl DpomoApp {
    /// Small clickable notes at the bottom-left of the main screen.
    fn badges(&mut self, ctx: &egui::Context, theme: &Theme) {
        let t = tr();
        let mut notes = Vec::new();
        if self.settings.config_error.is_some() {
            notes.push((t.config_error_badge.to_owned(), theme.focus, false));
        }
        if let Some(version) = &self.settings.update {
            notes.push((
                fill(t.update_badge, &[("version", version)]),
                theme.short_break,
                true,
            ));
        }
        if notes.is_empty() {
            return;
        }
        let clicked = egui::Area::new(egui::Id::unique("badges"))
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(8.0, -6.0))
            .show(ctx, |ui| {
                let mut clicked = None;
                for (text, color, is_update) in notes {
                    let label = egui::Label::new(egui::RichText::new(text).size(11.0).color(color))
                        .sense(egui::Sense::click());
                    if ui
                        .add(label)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        clicked = Some(is_update);
                    }
                }
                clicked
            })
            .inner;
        match clicked {
            Some(true) => self.apply_settings_action(settings_view::Action::OpenUpdatePage),
            Some(false) => self.view = View::Settings,
            None => {}
        }
    }
}

impl eframe::App for DpomoApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        self.handle_commands(ctx, now);
        self.timer.sync_settings(&self.config.timer);
        if let Some(event) = self.timer.tick(now, &self.config.timer) {
            self.on_timer_event(ctx, event, now);
        }
        if self.timer.is_running()
            && now.duration_since(self.last_state_save) >= STATE_SAVE_INTERVAL
        {
            self.save_state(now);
        }
        if self.config.behavior.check_updates && !self.update_checker_started {
            updates::spawn_checker(ctx.clone(), self.commands.0.clone());
            self.update_checker_started = true;
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
            self.remember_window_size(&ctx);
            if self.tray.is_some() && self.config.behavior.close_to_tray {
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                self.save();
                self.save_state(now);
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
        // The cozy view applies its own padding.
        let frame = egui::Frame::new()
            .fill(theme.background)
            .inner_margin(if cozy { 0 } else { 16 });
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
                        &self.app_state.totals(state::today()),
                        &mut self.settings,
                        &theme,
                    );
                    if let Some(a) = action {
                        self.apply_settings_action(a);
                    }
                }
            });

        if self.view == View::Timer {
            self.badges(&ctx, &theme);
        }

        // Keep the ring easing animation going.
        let (_, animating) = self
            .ring
            .value(self.timer.progress(now), ctx.input(|i| i.time));
        if animating {
            ctx.request_repaint();
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.config
            .theme(&self.themes)
            .background
            .to_normalized_gamma_f32()
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
