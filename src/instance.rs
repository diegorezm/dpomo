//! Single instance: the first `dpomo` listens on a localhost port; launching
//! it again sends that one a request (show its window, or a timer command
//! like `dpomo --toggle`), then exits.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::command::Command;

const ADDR: &str = "127.0.0.1:47613";
const PREFIX: &str = "dpomo:";
const ACK: &[u8] = b"dpomo:ok";

/// Command-line requests for the running instance: `dpomo --<flag>`.
/// Launching without one of these flags means "show".
pub const FLAGS: [(&str, &str); 7] = [
    ("--show", "show"),
    ("--toggle", "toggle"),
    ("--skip", "skip"),
    ("--reset", "reset"),
    ("--mini", "mini"),
    ("--settings", "settings"),
    ("--quit", "quit"),
];

/// The request named on the command line, or "show".
pub fn request_from_args() -> &'static str {
    std::env::args()
        .find_map(|a| {
            FLAGS
                .iter()
                .find(|(flag, _)| *flag == a)
                .map(|(_, verb)| *verb)
        })
        .unwrap_or("show")
}

/// What the running app should do for a request verb.
pub fn command_for(verb: &str) -> Option<Command> {
    Some(match verb {
        "show" => Command::ShowWindow,
        "toggle" => Command::ToggleTimer,
        "skip" => Command::Skip,
        "reset" => Command::Reset,
        "mini" => Command::ToggleMini,
        "settings" => Command::OpenSettings,
        "quit" => Command::Quit,
        _ => return None,
    })
}

pub enum Instance {
    /// We are the only one running; keep this to receive requests.
    Primary(TcpListener),
    /// Another dpomo answered and got the request.
    AlreadyRunning,
    /// Couldn't tell (port taken by something else). Run normally.
    Unknown,
}

pub fn acquire(verb: &str) -> Instance {
    if let Ok(listener) = TcpListener::bind(ADDR) {
        return Instance::Primary(listener);
    }
    if send(verb).unwrap_or(false) {
        Instance::AlreadyRunning
    } else {
        Instance::Unknown
    }
}

fn send(verb: &str) -> std::io::Result<bool> {
    let mut stream = TcpStream::connect_timeout(
        &ADDR.parse().expect("valid addr"),
        Duration::from_millis(300),
    )?;
    stream.set_read_timeout(Some(Duration::from_millis(500)))?;
    stream.write_all(format!("{PREFIX}{verb}\n").as_bytes())?;
    let mut buf = [0u8; ACK.len()];
    stream.read_exact(&mut buf)?;
    Ok(buf == ACK)
}

/// Read one `dpomo:<verb>\n` request (at most 64 bytes, 500 ms).
fn read_request(stream: &mut TcpStream) -> Option<Command> {
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .ok()?;
    let mut buf = Vec::with_capacity(64);
    let mut chunk = [0u8; 64];
    while !buf.contains(&b'\n') && buf.len() < 64 {
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
    }
    let text = std::str::from_utf8(&buf).ok()?;
    command_for(text.trim().strip_prefix(PREFIX)?)
}

/// Answer requests from later launches on a background thread.
pub fn listen(listener: TcpListener, ctx: egui::Context, tx: Sender<Command>) {
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            if let Some(cmd) = read_request(&mut stream) {
                let _ = stream.write_all(ACK);
                let _ = tx.send(cmd);
                ctx.request_repaint();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_flag_maps_to_a_command() {
        for (_, verb) in FLAGS {
            assert!(command_for(verb).is_some(), "{verb}");
        }
        assert_eq!(command_for("nope"), None);
    }
}
