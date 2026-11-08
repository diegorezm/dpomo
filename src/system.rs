//! OS integrations that aren't the tray: notifications and autostart.

/// Windows toast identity. Registered in `init` so notifications show
/// "Pomolotl" and our icon instead of "Windows PowerShell".
#[cfg(windows)]
const APP_ID: &str = "pomolotl.timer";

/// One-time OS setup at startup. Failures are non-fatal.
pub fn init() {
    #[cfg(windows)]
    if let Err(e) = register_windows_app_id() {
        eprintln!("pomolotl: could not register app id: {e}");
    }
    // Without this, the first notification makes macOS look up an app named
    // "use_default" and pops a "Choose Application" dialog. Must match
    // CFBundleIdentifier in packaging/macos/Info.plist.
    #[cfg(target_os = "macos")]
    if let Err(e) = notify_rust::set_application("io.github.diegorezm.pomolotl") {
        eprintln!("pomolotl: could not set notification app: {e}");
    }
}

/// Hand idle pages back to the OS. Most of the working set right after startup
/// is the GPU driver's init work, never touched again, so this drops "Memory"
/// in Task Manager from ~110 MB to ~15 MB. Pages that are still needed come
/// back on demand; nothing is freed or lost.
pub fn trim_memory() {
    #[cfg(windows)]
    // SAFETY: plain Win32 calls on our own process pseudo-handle.
    unsafe {
        use windows_sys::Win32::System::{
            ProcessStatus::K32EmptyWorkingSet, Threading::GetCurrentProcess,
        };
        K32EmptyWorkingSet(GetCurrentProcess());
    }
}

/// Windows 11 paints title bars in the system's colors (dark mode, or the
/// accent color if "show accent color on title bars" is on). Paint every pomolotl
/// window's bar with the theme instead, so it blends in, and round the
/// borderless floating timer's corners. Older Windows ignores all of this.
#[allow(unused_variables)]
pub fn style_windows(background: egui::Color32, text: egui::Color32) {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::{HWND, LPARAM};
        use windows_sys::Win32::Graphics::Dwm::{
            DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
            DwmSetWindowAttribute,
        };
        use windows_sys::Win32::System::Threading::GetCurrentThreadId;
        use windows_sys::Win32::UI::WindowsAndMessaging::EnumThreadWindows;

        unsafe extern "system" fn apply(hwnd: HWND, colors: LPARAM) -> windows_sys::core::BOOL {
            // SAFETY: `colors` points at the array below, alive for the whole enumeration.
            let [caption, text] = unsafe { *(colors as *const [u32; 2]) };
            let round = DWMWCP_ROUND as u32;
            for (attribute, value) in [
                (DWMWA_CAPTION_COLOR, caption),
                (DWMWA_TEXT_COLOR, text),
                (DWMWA_WINDOW_CORNER_PREFERENCE, round),
            ] {
                // SAFETY: a window of our own thread and a 4-byte value.
                unsafe {
                    DwmSetWindowAttribute(
                        hwnd,
                        attribute as u32,
                        (&value as *const u32).cast(),
                        size_of::<u32>() as u32,
                    );
                }
            }
            1 // keep enumerating
        }

        // COLORREF is 0x00BBGGRR.
        let colorref = |c: egui::Color32| u32::from_le_bytes([c.r(), c.g(), c.b(), 0]);
        let colors = [colorref(background), colorref(text)];
        // All pomolotl windows (main, settings, floating timer) live on this thread.
        // SAFETY: the callback only reads `colors`, which outlives the call.
        unsafe {
            EnumThreadWindows(
                GetCurrentThreadId(),
                Some(apply),
                &colors as *const [u32; 2] as LPARAM,
            );
        }
    }
}

#[cfg(windows)]
fn register_windows_app_id() -> Result<(), Box<dyn std::error::Error>> {
    let dir = crate::config::Config::path()
        .and_then(|p| Some(p.parent()?.to_path_buf()))
        .ok_or("no config dir")?;
    std::fs::create_dir_all(&dir)?;
    let icon = dir.join("pomolotl.ico");
    std::fs::write(&icon, include_bytes!("../assets/icon.ico"))?;

    let key = windows_registry::CURRENT_USER
        .create(format!(r"Software\Classes\AppUserModelId\{APP_ID}"))?;
    key.set_string("DisplayName", "Pomolotl")?;
    key.set_string("IconUri", icon.to_string_lossy())?;
    Ok(())
}

use crate::config::TimerSettings;
use crate::i18n::{fill, tr};
use crate::timer::Phase;

/// Fire-and-forget desktop notification for a finished phase. Runs on a
/// thread because some backends (D-Bus, WinRT) can block for a moment.
pub fn notify_finished(finished: Phase, next: Phase, next_started: bool, settings: &TimerSettings) {
    let t = tr();
    let title = match finished {
        Phase::Focus => t.notify_focus_done,
        Phase::ShortBreak | Phase::LongBreak => t.notify_break_done,
    };
    let minutes = (settings.duration(next).as_secs() / 60).to_string();
    let template = match next {
        Phase::Focus => t.notify_next_focus,
        Phase::ShortBreak => t.notify_next_short,
        Phase::LongBreak => t.notify_next_long,
    };
    let what = fill(template, &[("min", &minutes)]);
    let body = if next_started {
        format!("{what} — {}.", t.notify_started)
    } else {
        format!("{what}.")
    };

    std::thread::spawn(move || {
        let mut notification = notify_rust::Notification::new();
        notification.appname("Pomolotl").summary(title).body(&body);
        #[cfg(windows)]
        notification.app_id(APP_ID);
        let result = notification.show();
        if let Err(e) = result {
            eprintln!("pomolotl: notification failed: {e}");
        }
    });
}

/// Open a file or folder with the OS default app.
pub fn open_path(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(windows)]
    let mut cmd = {
        let mut c = std::process::Command::new("explorer");
        c.arg(path);
        c
    };
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.arg(path);
        c
    };
    #[cfg(not(any(windows, target_os = "macos")))]
    let mut cmd = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(path);
        c
    };
    cmd.spawn().map(|_| ())
}

/// Register or unregister pomolotl to start (hidden, in the tray) on login.
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let launcher = auto_launch::AutoLaunchBuilder::new()
        .set_app_name("pomolotl")
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
