//! User settings, persisted as a human-editable TOML file.
//!
//! Every struct uses `#[serde(default)]`, so adding a new field never breaks
//! old config files: missing keys just fall back to `Default`.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::theme::{self, Theme, Themes};
use crate::timer::Phase;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub timer: TimerSettings,
    pub appearance: Appearance,
    pub behavior: Behavior,
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
            theme: theme::DEFAULT_NAME.to_owned(),
            ring_thickness: 10.0,
            ui_scale: 1.0,
            smooth_animation: true,
            glow: true,
            show_session_dots: true,
        }
    }
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
        }
    }
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("dpomo").join("config.toml"))
    }

    /// Load from disk. Never fails: on any problem, falls back to defaults.
    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
                eprintln!("dpomo: invalid config at {}: {e}", path.display());
                Self::default()
            }),
            Err(_) => Self::default(),
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
        std::fs::write(path, text)
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
    fn partial_file_uses_defaults() {
        let c: Config = toml::from_str("[timer]\nfocus_minutes = 40\n").unwrap();
        assert_eq!(c.timer.focus_minutes, 40);
        assert_eq!(c.timer.short_break_minutes, 5);
        assert_eq!(c.appearance, Appearance::default());
    }
}
