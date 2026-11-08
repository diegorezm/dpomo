//! User settings, persisted as a human-editable TOML file.
//!
//! The file is the source of truth: it's written on first launch, the
//! settings screen edits it, and hand edits are picked up live (see `watch`).
//!
//! Every struct uses `#[serde(default)]`, so adding a new field never breaks
//! old config files: missing keys just fall back to `Default`.

use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::{Duration, SystemTime};

use egui::Key;
use serde::{Deserialize, Serialize};

use crate::command::Command;
use crate::i18n::{self, Language};
use crate::sound::SoundKind;
use crate::theme::{self, Theme, Themes};
use crate::timer::Phase;

const HEADER: &str = "\
# dpomo config. Edit freely: changes apply while the app is running.
# Every key is optional; anything missing uses its default.
# Every option explained: https://diegorezm.github.io/dpomo/docs.html

";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub timer: TimerSettings,
    pub appearance: Appearance,
    pub behavior: Behavior,
    pub sound: Sound,
    pub window: Window,
    pub mini: Mini,
    pub keys: Keys,
    /// Used when `appearance.theme = "custom"`.
    pub custom_theme: Theme,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TimerSettings {
    pub focus_minutes: u32,
    pub short_break_minutes: u32,
    pub long_break_minutes: u32,
    pub sessions_before_long_break: u32,
    pub auto_start_breaks: bool,
    pub auto_start_focus: bool,
}

impl Default for TimerSettings {
    fn default() -> Self {
        Self {
            focus_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            sessions_before_long_break: 4,
            auto_start_breaks: false,
            auto_start_focus: false,
        }
    }
}

impl TimerSettings {
    pub fn duration(&self, phase: Phase) -> Duration {
        let minutes = match phase {
            Phase::Focus => self.focus_minutes,
            Phase::ShortBreak => self.short_break_minutes,
            Phase::LongBreak => self.long_break_minutes,
        };
        Duration::from_secs(u64::from(minutes.max(1)) * 60)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    /// "auto" (follow the OS), "en" or "pt-BR".
    pub language: Language,
    pub layout: Layout,
    /// Character pack id (folder name in `characters/`). Cozy layout only.
    pub character: String,
    /// A theme name (see `themes/*.toml`), or `"custom"`.
    pub theme: String,
    pub ring_thickness: f32,
    pub ui_scale: f32,
    /// Repaint ~30 fps while running for a fluid ring. Off = 1 repaint/sec.
    pub smooth_animation: bool,
    pub glow: bool,
    pub show_session_dots: bool,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            language: Language::Auto,
            layout: Layout::Cozy,
            character: crate::character::DEFAULT_ID.to_owned(),
            theme: theme::DEFAULT_NAME.to_owned(),
            ring_thickness: 10.0,
            ui_scale: 1.0,
            smooth_animation: true,
            glow: true,
            show_session_dots: true,
        }
    }
}

/// Main screen style.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    /// Card with a character, big chunky timer, phase-colored backdrop.
    #[default]
    Cozy,
    /// Minimal progress ring.
    Classic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Behavior {
    pub always_on_top: bool,
    /// Flash the taskbar / dock when a phase ends.
    pub notify_on_finish: bool,
    /// Bring the window to front when a phase ends.
    pub focus_on_finish: bool,
    /// Show remaining time in the window title (visible in the taskbar).
    pub time_in_title: bool,
    /// System tray icon with live progress and a menu.
    pub tray: bool,
    /// Closing the window hides it to the tray instead of quitting.
    pub close_to_tray: bool,
    /// Native desktop notification when a phase ends.
    pub system_notification: bool,
    /// Launch on login (hidden, in the tray).
    pub start_with_system: bool,
    /// Start hidden in the tray when launched manually too.
    pub start_hidden: bool,
    /// Look for a newer release on GitHub (once a day). Never auto-installs.
    pub check_updates: bool,
}

impl Default for Behavior {
    fn default() -> Self {
        Self {
            always_on_top: false,
            notify_on_finish: true,
            focus_on_finish: false,
            time_in_title: true,
            tray: true,
            close_to_tray: true,
            system_notification: true,
            start_with_system: false,
            start_hidden: false,
            check_updates: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sound {
    pub enabled: bool,
    /// "chime", "bell", "soft" or "file" (uses `file`).
    pub kind: SoundKind,
    /// Path to a .wav/.mp3/.ogg, when `kind = "file"`.
    pub file: String,
    /// 0.0 to 1.0.
    pub volume: f32,
}

impl Default for Sound {
    fn default() -> Self {
        Self {
            enabled: true,
            kind: SoundKind::Chime,
            file: String::new(),
            volume: 0.6,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Window {
    /// Initial size, in points.
    pub width: f32,
    pub height: f32,
    /// Update `width`/`height` with the last size when the window closes.
    pub remember_size: bool,
}

impl Window {
    pub const MIN_SIZE: [f32; 2] = [280.0, 380.0];
}

impl Default for Window {
    fn default() -> Self {
        Self {
            width: 360.0,
            height: 500.0,
            remember_size: true,
        }
    }
}

/// The floating timer: a small borderless always-on-top window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mini {
    pub enabled: bool,
    /// Size in points; changed by dragging its corner.
    pub width: f32,
    pub height: f32,
    /// Screen position, set when the user moves it. Missing = top-right corner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,
    pub show_character: bool,
}

impl Default for Mini {
    fn default() -> Self {
        Self {
            enabled: false,
            width: 260.0,
            height: 96.0,
            x: None,
            y: None,
            show_character: true,
        }
    }
}

impl Mini {
    pub const MIN_SIZE: [f32; 2] = [120.0, 44.0];
}

/// Keyboard shortcuts. Values are egui key names: "Space", "R", ",", "1",
/// "F5", "Escape", "ArrowLeft"… An unknown name falls back to the default.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Keys {
    pub toggle: String,
    pub reset: String,
    pub skip: String,
    pub settings: String,
    pub focus: String,
    pub short_break: String,
    pub long_break: String,
    /// Show / hide the floating timer.
    pub mini: String,
    /// Close the settings window.
    pub back: String,
}

impl Default for Keys {
    fn default() -> Self {
        Self {
            toggle: "Space".into(),
            reset: "R".into(),
            skip: "S".into(),
            settings: ",".into(),
            focus: "1".into(),
            short_break: "2".into(),
            long_break: "3".into(),
            mini: "M".into(),
            back: "Escape".into(),
        }
    }
}

impl Keys {
    /// Parse `name`, or use `default` if it isn't a key name.
    pub fn key(name: &str, default: &str) -> Key {
        Key::from_name(name.trim())
            .or_else(|| Key::from_name(default))
            .unwrap_or(Key::Space)
    }
}

/// Why the config file couldn't be used.
#[derive(Debug, PartialEq)]
pub enum LoadError {
    Missing,
    /// Short human-readable message (line + reason).
    Invalid(String),
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("dpomo").join("config.toml"))
    }

    pub fn read() -> Result<Self, LoadError> {
        let path = Self::path().ok_or(LoadError::Missing)?;
        let text = std::fs::read_to_string(&path).map_err(|_| LoadError::Missing)?;
        Self::parse(&text)
    }

    fn parse(text: &str) -> Result<Self, LoadError> {
        toml::from_str(text).map_err(|e| {
            let line = e
                .span()
                .map(|s| text[..s.start.min(text.len())].lines().count().max(1));
            LoadError::Invalid(match line {
                Some(n) => i18n::fill(
                    i18n::tr().config_line,
                    &[("n", &n.to_string()), ("msg", e.message())],
                ),
                None => e.message().to_owned(),
            })
        })
    }

    /// Startup load. Writes the defaults on first launch so the file is there
    /// to edit. Also returns the error message if the file is broken.
    pub fn load() -> (Self, Option<String>) {
        match Self::read() {
            Ok(config) => (config, None),
            Err(LoadError::Missing) => {
                let config = Self::default();
                if let Err(e) = config.save() {
                    eprintln!("dpomo: could not write default config: {e}");
                }
                (config, None)
            }
            Err(LoadError::Invalid(e)) => (Self::default(), Some(e)),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let Some(path) = Self::path() else {
            return Ok(());
        };
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, format!("{HEADER}{text}"))
    }

    fn modified() -> Option<SystemTime> {
        std::fs::metadata(Self::path()?).ok()?.modified().ok()
    }

    /// Poll the file once a second and send `Command::ReloadConfig` when it
    /// changes. One `stat` per second, no extra dependencies.
    pub fn watch(ctx: egui::Context, tx: Sender<Command>) {
        std::thread::spawn(move || {
            let mut last = Self::modified();
            loop {
                std::thread::sleep(Duration::from_secs(1));
                let now = Self::modified();
                if now != last {
                    last = now;
                    if tx.send(Command::ReloadConfig).is_err() {
                        return;
                    }
                    ctx.request_repaint();
                }
            }
        });
    }

    /// The theme currently in effect.
    pub fn theme(&self, themes: &Themes) -> Theme {
        if self.appearance.theme == theme::CUSTOM {
            return self.custom_theme;
        }
        themes.get(&self.appearance.theme).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_toml() {
        let mut c = Config::default();
        c.timer.focus_minutes = 50;
        c.appearance.theme = "Nord".into();
        let text = toml::to_string_pretty(&c).unwrap();
        assert_eq!(toml::from_str::<Config>(&text).unwrap(), c);
    }

    #[test]
    fn saved_file_with_header_parses() {
        let text = format!(
            "{HEADER}{}",
            toml::to_string_pretty(&Config::default()).unwrap()
        );
        assert_eq!(Config::parse(&text), Ok(Config::default()));
    }

    #[test]
    fn invalid_file_reports_line() {
        let err = Config::parse("[timer]\nfocus_minutes = \"muito\"\n").unwrap_err();
        let LoadError::Invalid(msg) = err else {
            panic!("expected Invalid")
        };
        assert!(msg.starts_with("line 2:"), "{msg}");
    }

    /// Both READMEs document every setting with its default; keep them in sync.
    /// Compares raw TOML tables, so a missing key fails too.
    #[test]
    fn readme_reference_matches_defaults() {
        let defaults = toml::Table::try_from(Config::default()).unwrap();
        for (name, readme) in [
            ("README.md", include_str!("../README.md")),
            ("README.pt-BR.md", include_str!("../README.pt-BR.md")),
        ] {
            // Normalize CRLF (Windows checkouts) so the search works everywhere.
            let readme = readme.replace("\r\n", "\n");
            let start = readme
                .find("```toml\n[timer]")
                .unwrap_or_else(|| panic!("config reference in {name}"))
                + 8;
            let end = start + readme[start..].find("```").unwrap();
            let block = &readme[start..end];
            // Same values (compared as Config, so f32 rounding doesn't matter)…
            let documented: Config = toml::from_str(block).unwrap();
            assert_eq!(documented, Config::default(), "{name}: wrong default value");
            // …and every key is documented.
            let table: toml::Table = toml::from_str(block).unwrap();
            assert_eq!(
                keys(&table),
                keys(&defaults),
                "{name}: missing or extra keys"
            );
        }
    }

    fn keys(table: &toml::Table) -> Vec<String> {
        let mut out = Vec::new();
        for (section, value) in table {
            match value.as_table() {
                Some(inner) => out.extend(inner.keys().map(|k| format!("{section}.{k}"))),
                None => out.push(section.clone()),
            }
        }
        out.sort();
        out
    }

    #[test]
    fn unknown_key_names_fall_back() {
        assert_eq!(Keys::key("F5", "Space"), Key::F5);
        assert_eq!(Keys::key(",", "Space"), Key::Comma);
        assert_eq!(Keys::key("banana", "R"), Key::R);
    }

    #[test]
    fn partial_file_uses_defaults() {
        let c: Config = toml::from_str("[timer]\nfocus_minutes = 40\n").unwrap();
        assert_eq!(c.timer.focus_minutes, 40);
        assert_eq!(c.timer.short_break_minutes, 5);
        assert_eq!(c.appearance, Appearance::default());
    }
}
