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
    let size = 64;
    let rgba = crate::tray::render_ring(size, 1.0, crate::theme::Theme::default().focus, false);
    std::fs::write(&icon, ico_bytes(&rgba, size))?;

    let key = windows_registry::CURRENT_USER
        .create(format!(r"Software\Classes\AppUserModelId\{APP_ID}"))?;
    key.set_string("DisplayName", "dpomo")?;
    key.set_string("IconUri", icon.to_string_lossy())?;
    Ok(())
}

/// Minimal single-image .ico (32-bit BMP payload) from square RGBA pixels.
#[cfg(any(windows, test))]
fn ico_bytes(rgba: &[u8], size: u32) -> Vec<u8> {
    let mask_row = size.div_ceil(32) * 4;
    let image_len = 40 + size * size * 4 + mask_row * size;
    let mut out = Vec::with_capacity(22 + image_len as usize);
    // ICONDIR + ICONDIRENTRY
    out.extend_from_slice(&[0, 0, 1, 0, 1, 0]);
    out.extend_from_slice(&[size as u8, size as u8, 0, 0, 1, 0, 32, 0]);
    out.extend_from_slice(&image_len.to_le_bytes());
    out.extend_from_slice(&22u32.to_le_bytes());
    // BITMAPINFOHEADER (height is doubled: color + AND mask)
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&(size * 2).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(&[0; 24]);
    // Pixels: BGRA, bottom-up.
    for y in (0..size).rev() {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            out.extend_from_slice(&[rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]]);
        }
    }
    // AND mask all zero: alpha channel does the work.
    out.resize(out.len() + (mask_row * size) as usize, 0);
    out
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
    #[test]
    fn ico_has_expected_layout() {
        let size = 16;
        let ico = super::ico_bytes(&vec![255; (size * size * 4) as usize], size);
        assert_eq!(&ico[..6], &[0, 0, 1, 0, 1, 0]);
        let declared = u32::from_le_bytes(ico[14..18].try_into().unwrap());
        assert_eq!(ico.len(), 22 + declared as usize);
    }
}

#[cfg(test)]
mod autostart_tests {
    /// Touches the real OS autostart entry, so it only runs on demand:
    /// `cargo test -- --ignored autostart`
    #[test]
    #[ignore]
    fn autostart_roundtrip() {
        super::set_autostart(true).unwrap();
        super::set_autostart(false).unwrap();
    }
}
