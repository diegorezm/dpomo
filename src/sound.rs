//! Phase-end sounds. Built-in sounds are synthesized (no audio assets in the
//! binary); custom files are decoded by rodio (wav / mp3 / ogg).
//!
//! Playback runs on a short-lived thread that opens the default output
//! device, plays, and closes it — nothing stays open while idle.

use std::f32::consts::TAU;
use std::num::NonZero;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::Duration;

use rodio::Source;

use serde::{Deserialize, Serialize};

const RATE: u32 = 44_100;

/// (start seconds, frequency Hz)
type Note = (f32, f32);
/// (frequency multiplier, amplitude)
type Partial = (f32, f32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SoundKind {
    #[default]
    Chime,
    Bell,
    Soft,
    /// `sound.file` from the config.
    File,
}

impl SoundKind {
    pub const ALL: [SoundKind; 4] = [
        SoundKind::Chime,
        SoundKind::Bell,
        SoundKind::Soft,
        SoundKind::File,
    ];
}

/// Play `kind` at `volume` (0..=1) without blocking. `rising` picks the
/// melody direction for built-in sounds: up when a break starts, down when
/// it's time to focus again.
pub fn play(kind: SoundKind, file: &str, volume: f32, rising: bool) {
    let file = file.to_owned();
    let volume = volume.clamp(0.0, 1.0);
    std::thread::spawn(move || {
        if let Err(e) = play_blocking(kind, &file, volume, rising) {
            eprintln!("dpomo: could not play sound: {e}");
        }
    });
}

fn play_blocking(
    kind: SoundKind,
    file: &str,
    volume: f32,
    rising: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut sink = rodio::DeviceSinkBuilder::open_default_sink()?;
    sink.log_on_drop(false);
    let player = rodio::Player::connect_new(sink.mixer());
    player.set_volume(volume);
    match kind {
        SoundKind::File => player.append(rodio::Decoder::try_from(std::fs::File::open(file)?)?),
        built_in => player.append(rodio::buffer::SamplesBuffer::new(
            NonZero::new(1).expect("1 channel"),
            NonZero::new(RATE).expect("non-zero rate"),
            synthesize(built_in, rising),
        )),
    }
    player.sleep_until_end();
    Ok(())
}

/// The clock tick during focus (`sound.ticking`). The output device is open
/// only while it ticks: dropping the sender stops the thread.
#[derive(Default)]
pub struct Ticker {
    running: Option<(f32, Sender<f32>)>,
}

impl Ticker {
    /// Call every pass; cheap when nothing changes.
    pub fn set(&mut self, on: bool, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        match (&mut self.running, on) {
            (None, false) => {}
            (Some(_), false) => self.running = None,
            (Some((current, tx)), true) => {
                if *current != volume && tx.send(volume).is_ok() {
                    *current = volume;
                }
            }
            (None, true) => {
                let (tx, rx) = mpsc::channel();
                std::thread::spawn(move || {
                    if let Err(e) = tick_blocking(volume, rx) {
                        eprintln!("dpomo: could not play the ticking: {e}");
                    }
                });
                self.running = Some((volume, tx));
            }
        }
    }
}

fn tick_blocking(volume: f32, rx: mpsc::Receiver<f32>) -> Result<(), Box<dyn std::error::Error>> {
    let mut sink = rodio::DeviceSinkBuilder::open_default_sink()?;
    sink.log_on_drop(false);
    let player = rodio::Player::connect_new(sink.mixer());
    player.set_volume(volume);
    player.append(
        rodio::buffer::SamplesBuffer::new(
            NonZero::new(1).expect("1 channel"),
            NonZero::new(RATE).expect("non-zero rate"),
            tick(),
        )
        .repeat_infinite(),
    );
    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(volume) => player.set_volume(volume),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
}

/// One second: a short wooden "tock" followed by silence.
pub fn tick() -> Vec<f32> {
    let mut out = vec![0.0f32; RATE as usize];
    for (i, sample) in out.iter_mut().enumerate().take(RATE as usize / 20) {
        let t = i as f32 / RATE as f32;
        let envelope = (t / 0.001).min(1.0) * (-t * 160.0).exp();
        let tone = (TAU * 1250.0 * t).sin() + 0.4 * (TAU * 2900.0 * t).sin();
        *sample = 0.35 * envelope * tone;
    }
    out
}

/// Mono samples for a built-in sound.
pub fn synthesize(kind: SoundKind, rising: bool) -> Vec<f32> {
    let (notes, partials, decay): (Vec<Note>, &[Partial], f32) = match kind {
        // Three-note arpeggio, soft sine with a little shimmer.
        SoundKind::Chime | SoundKind::File => (
            arpeggio(&[659.25, 783.99, 1046.5], 0.14, rising),
            &[(1.0, 1.0), (2.0, 0.25), (3.0, 0.08)],
            3.2,
        ),
        // Two bell strikes with inharmonic partials.
        SoundKind::Bell => (
            arpeggio(&[880.0, 659.25], 0.35, !rising),
            &[(1.0, 1.0), (2.76, 0.45), (5.4, 0.2)],
            2.2,
        ),
        // One gentle tone.
        SoundKind::Soft => (vec![(0.0, 523.25)], &[(1.0, 1.0), (2.0, 0.1)], 1.8),
    };
    let length = notes.last().map_or(0.0, |(t, _)| *t) + 1.6;
    let mut out = vec![0.0f32; (length * RATE as f32) as usize];
    for (start, freq) in notes {
        let first = (start * RATE as f32) as usize;
        for (i, sample) in out.iter_mut().enumerate().skip(first) {
            let t = (i - first) as f32 / RATE as f32;
            let attack = (t / 0.008).min(1.0);
            let envelope = attack * (-decay * t).exp();
            let tone: f32 = partials
                .iter()
                .map(|(mult, amp)| amp * (TAU * freq * mult * t).sin())
                .sum();
            *sample += 0.25 * envelope * tone;
        }
    }
    // Normalize so loud partial sums never clip.
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak > 0.9 {
        out.iter_mut().for_each(|s| *s *= 0.9 / peak);
    }
    out
}

fn arpeggio(freqs: &[f32], step: f32, rising: bool) -> Vec<Note> {
    let mut freqs = freqs.to_vec();
    if !rising {
        freqs.reverse();
    }
    freqs
        .into_iter()
        .enumerate()
        .map(|(i, f)| (i as f32 * step, f))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_is_one_quiet_second() {
        let s = tick();
        assert_eq!(s.len(), RATE as usize);
        assert!(s.iter().all(|x| x.abs() <= 0.5));
        assert!(s[RATE as usize / 10..].iter().all(|x| *x == 0.0));
    }

    #[test]
    fn built_in_sounds_are_short_and_never_clip() {
        for kind in [SoundKind::Chime, SoundKind::Bell, SoundKind::Soft] {
            for rising in [true, false] {
                let s = synthesize(kind, rising);
                let secs = s.len() as f32 / RATE as f32;
                assert!((1.0..3.0).contains(&secs), "{kind:?} lasts {secs}s");
                assert!(s.iter().all(|x| x.abs() <= 0.9001), "{kind:?} clips");
                assert!(s.iter().any(|x| x.abs() > 0.1), "{kind:?} is silent");
            }
        }
    }
}
