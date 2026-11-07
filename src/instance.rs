//! Single instance: the first `dpomo` listens on a localhost port; launching
//! it again just asks that one to show its window, then exits.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::command::Command;

const ADDR: &str = "127.0.0.1:47613";
const HELLO: &[u8] = b"dpomo:show";
const ACK: &[u8] = b"dpomo:ok";

pub enum Instance {
    /// We are the only one running; keep this to receive "show" requests.
    Primary(TcpListener),
    /// Another dpomo answered and is now showing its window.
    AlreadyRunning,
    /// Couldn't tell (port taken by something else). Run normally.
    Unknown,
}

pub fn acquire() -> Instance {
    if let Ok(listener) = TcpListener::bind(ADDR) {
        return Instance::Primary(listener);
    }
    if ping_existing().unwrap_or(false) {
        Instance::AlreadyRunning
    } else {
        Instance::Unknown
    }
}

fn ping_existing() -> std::io::Result<bool> {
    let mut stream = TcpStream::connect_timeout(
        &ADDR.parse().expect("valid addr"),
        Duration::from_millis(300),
    )?;
    stream.set_read_timeout(Some(Duration::from_millis(500)))?;
    stream.write_all(HELLO)?;
    let mut buf = [0u8; ACK.len()];
    stream.read_exact(&mut buf)?;
    Ok(buf == ACK)
}

/// Answer "show" requests from later launches on a background thread.
pub fn listen(listener: TcpListener, ctx: egui::Context, tx: Sender<Command>) {
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
            let mut buf = [0u8; HELLO.len()];
            if stream.read_exact(&mut buf).is_ok() && buf == HELLO {
                let _ = stream.write_all(ACK);
                let _ = tx.send(Command::ShowWindow);
                ctx.request_repaint();
            }
        }
    });
}
