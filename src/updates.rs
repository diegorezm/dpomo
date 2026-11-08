//! "A new version is available" check against GitHub releases. Read-only:
//! it never downloads or installs anything, it only tells the user.

use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::command::Command;

pub const RELEASES_URL: &str = "https://github.com/diegorezm/pomolotl/releases/latest";
const API_URL: &str = "https://api.github.com/repos/diegorezm/pomolotl/releases/latest";

/// Check shortly after startup and then once a day, on a background thread.
pub fn spawn_checker(ctx: egui::Context, tx: Sender<Command>) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(5));
        loop {
            match latest_version() {
                Ok(latest) if is_newer(&latest, env!("CARGO_PKG_VERSION")) => {
                    if tx.send(Command::UpdateAvailable(latest)).is_err() {
                        return;
                    }
                    ctx.request_repaint();
                }
                Ok(_) => {}
                Err(e) => eprintln!("pomolotl: update check failed: {e}"),
            }
            std::thread::sleep(Duration::from_secs(24 * 60 * 60));
        }
    });
}

/// Asks the OS's own `curl` (built into Windows 10+, macOS and most Linux
/// distros) instead of bundling an HTTP + TLS stack: that kept ~2 MB out of
/// the binary. No curl = no update check, silently.
fn latest_version() -> Result<String, Box<dyn std::error::Error>> {
    #[cfg(windows)]
    let mut cmd = {
        use std::os::windows::process::CommandExt;
        // Absolute path: never pick up a `curl.exe` from the working directory.
        let root = std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
        let mut cmd =
            std::process::Command::new(std::path::Path::new(&root).join(r"System32\curl.exe"));
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: no console flash
        cmd
    };
    #[cfg(not(windows))]
    let mut cmd = std::process::Command::new("curl");
    let out = cmd
        .args([
            "--silent",
            "--show-error",
            "--fail",
            "--location",
            "--max-time",
            "10",
        ])
        .args([
            "--user-agent",
            concat!("pomolotl/", env!("CARGO_PKG_VERSION")),
        ])
        .args(["--header", "Accept: application/vnd.github+json", API_URL])
        .stdin(std::process::Stdio::null())
        .output()?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().into());
    }
    let body: serde_json::Value = serde_json::from_slice(&out.stdout)?;
    let tag = body["tag_name"].as_str().ok_or("no tag_name")?;
    Ok(tag.trim_start_matches('v').to_owned())
}

fn parse(v: &str) -> Option<(u64, u64, u64)> {
    let mut parts = v.trim_start_matches('v').split(['.', '-', '+']);
    Some((
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next().unwrap_or("0").parse().ok()?,
    ))
}

/// Is `latest` strictly newer than `current`? Unparseable versions are never newer.
pub fn is_newer(latest: &str, current: &str) -> bool {
    matches!((parse(latest), parse(current)), (Some(l), Some(c)) if l > c)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hits the real GitHub API: `cargo test -- --ignored latest_release`.
    #[test]
    #[ignore]
    fn latest_release_is_reachable() {
        let v = latest_version().unwrap();
        assert!(parse(&v).is_some(), "{v}");
    }

    #[test]
    fn compares_versions() {
        assert!(is_newer("1.0.1", "1.0.0"));
        assert!(is_newer("v1.2.0", "1.1.9"));
        assert!(is_newer("2.0.0", "1.9.9"));
        assert!(!is_newer("1.0.0", "1.0.0"));
        assert!(!is_newer("0.9.0", "1.0.0"));
        assert!(!is_newer("banana", "1.0.0"));
    }
}
