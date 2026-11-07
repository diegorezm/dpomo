//! "A new version is available" check against GitHub releases. Read-only:
//! it never downloads or installs anything, it only tells the user.

use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::command::Command;

pub const RELEASES_URL: &str = "https://github.com/diegorezm/dpomo/releases/latest";
const API_URL: &str = "https://api.github.com/repos/diegorezm/dpomo/releases/latest";

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
                Err(e) => eprintln!("dpomo: update check failed: {e}"),
            }
            std::thread::sleep(Duration::from_secs(24 * 60 * 60));
        }
    });
}

fn latest_version() -> Result<String, Box<dyn std::error::Error>> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .new_agent();
    let body: serde_json::Value = agent
        .get(API_URL)
        .header("User-Agent", concat!("dpomo/", env!("CARGO_PKG_VERSION")))
        .header("Accept", "application/vnd.github+json")
        .call()?
        .body_mut()
        .read_json()?;
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
