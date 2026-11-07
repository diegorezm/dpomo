//! OS integrations that aren't the tray: notifications and autostart.

/// Windows toast identity. Registered in `init` so notifications show
/// "dpomo" and our icon instead of "Windows PowerShell".
#[cfg(windows)]
const APP_ID: &str = "dpomo.timer";

/// One-time OS setup at startup. Failures are non-fatal.
pub fn init() {
    #[cfg(windows)]
    if let Err(e) = register_windows_app_id() {
        eprintln!("dpomo: could not register app id: {e}");
    }
}

#[cfg(windows)]
fn register_windows_app_id() -> Result<(), Box<dyn std::error::Error>> {
    let dir = crate::config::Config::path()
        .and_then(|p| Some(p.parent()?.to_path_buf()))
        .ok_or("no config dir")?;
    std::fs::create_dir_all(&dir)?;
    let icon = dir.join("dpomo.ico");
    std::fs::write(&icon, include_bytes!("../assets/icon.ico"))?;

    let key = windows_registry::CURRENT_USER
        .create(format!(r"Software\Classes\AppUserModelId\{APP_ID}"))?;
    key.set_string("DisplayName", "dpomo")?;
    key.set_string("IconUri", icon.to_string_lossy())?;
    Ok(())
}

use crate::config::TimerSettings;
use crate::timer::Phase;

/// Fire-and-forget desktop notification for a finished phase. Runs on a
/// thread because some backends (D-Bus, WinRT) can block for a moment.
pub fn notify_finished(finished: Phase, next: Phase, next_started: bool, settings: &TimerSettings) {
    let title = match finished {
        Phase::Focus => "Foco concluído! 🍅",
        Phase::ShortBreak | Phase::LongBreak => "Pausa terminada",
    };
    let minutes = settings.duration(next).as_secs() / 60;
    let what = match next {
        Phase::Focus => format!("Hora de focar ({minutes} min)"),
        Phase::ShortBreak => format!("Hora de uma pausa curta ({minutes} min)"),
        Phase::LongBreak => format!("Hora de uma pausa longa ({minutes} min)"),
    };
    let body = if next_started {
        format!("{what} — já começou.")
    } else {
        format!("{what}.")
    };

    std::thread::spawn(move || {
        let mut notification = notify_rust::Notification::new();
        notification.appname("dpomo").summary(title).body(&body);
        #[cfg(windows)]
        notification.app_id(APP_ID);
        let result = notification.show();
        if let Err(e) = result {
            eprintln!("dpomo: notification failed: {e}");
        }
    });
}

/// Register or unregister dpomo to start (hidden, in the tray) on login.
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let launcher = auto_launch::AutoLaunchBuilder::new()
        .set_app_name("dpomo")
        .set_app_path(&exe.to_string_lossy())
        .set_args(&["--hidden"])
        .build()
        .map_err(|e| e.to_string())?;
    let result = if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    };
    result.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    /// Touches the real OS autostart entry, so it only runs on demand:
    /// `cargo test -- --ignored autostart`
    #[test]
    #[ignore]
    fn autostart_roundtrip() {
        super::set_autostart(true).unwrap();
        super::set_autostart(false).unwrap();
    }
}
