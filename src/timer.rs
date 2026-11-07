//! Pure Pomodoro state machine. No UI, no I/O: every method takes `now`
//! explicitly, which keeps it deterministic and easy to unit test.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::config::TimerSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub const ALL: [Phase; 3] = [Phase::Focus, Phase::ShortBreak, Phase::LongBreak];
}

/// Emitted by [`Timer::tick`] when a phase runs out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerEvent {
    Finished { finished: Phase, next: Phase },
}

#[derive(Clone, Debug)]
pub struct Timer {
    phase: Phase,
    duration: Duration,
    /// Time accumulated by previous run segments (before the last pause).
    banked: Duration,
    /// `Some` while running.
    started_at: Option<Instant>,
    completed_focus: u32,
}

impl Timer {
    pub fn new(settings: &TimerSettings) -> Self {
        Self {
            phase: Phase::Focus,
            duration: settings.duration(Phase::Focus),
            banked: Duration::ZERO,
            started_at: None,
            completed_focus: 0,
        }
    }

    /// Continue a saved session: same phase and progress, paused.
    pub fn restore(
        phase: Phase,
        elapsed: Duration,
        completed_focus: u32,
        settings: &TimerSettings,
    ) -> Self {
        let duration = settings.duration(phase);
        Self {
            phase,
            duration,
            banked: elapsed.min(duration),
            started_at: None,
            completed_focus,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn is_running(&self) -> bool {
        self.started_at.is_some()
    }

    /// Focus sessions completed since the app started.
    pub fn completed_focus(&self) -> u32 {
        self.completed_focus
    }

    /// True if the current phase has not been started yet.
    pub fn is_pristine(&self) -> bool {
        self.started_at.is_none() && self.banked.is_zero()
    }

    pub fn elapsed(&self, now: Instant) -> Duration {
        let running = self
            .started_at
            .map_or(Duration::ZERO, |t| now.saturating_duration_since(t));
        (self.banked + running).min(self.duration)
    }

    pub fn remaining(&self, now: Instant) -> Duration {
        self.duration.saturating_sub(self.elapsed(now))
    }

    /// Fraction of the phase still remaining, from 1.0 (full) to 0.0 (done).
    pub fn progress(&self, now: Instant) -> f32 {
        if self.duration.is_zero() {
            return 0.0;
        }
        self.remaining(now).as_secs_f32() / self.duration.as_secs_f32()
    }

    pub fn start(&mut self, now: Instant) {
        if self.started_at.is_none() {
            self.started_at = Some(now);
        }
    }

    pub fn pause(&mut self, now: Instant) {
        if let Some(t) = self.started_at.take() {
            self.banked += now.saturating_duration_since(t);
        }
    }

    pub fn toggle(&mut self, now: Instant) {
        if self.is_running() {
            self.pause(now);
        } else {
            self.start(now);
        }
    }

    /// Restart the current phase, stopped.
    pub fn reset(&mut self, settings: &TimerSettings) {
        self.switch_to(self.phase, settings);
    }

    /// Jump to a phase, stopped and full.
    pub fn switch_to(&mut self, phase: Phase, settings: &TimerSettings) {
        self.phase = phase;
        self.duration = settings.duration(phase);
        self.banked = Duration::ZERO;
        self.started_at = None;
    }

    /// Move to the next phase without counting the current one as completed.
    pub fn skip(&mut self, settings: &TimerSettings) {
        let next = self.next_phase(settings, self.completed_focus);
        self.switch_to(next, settings);
    }

    /// Pick up changed durations, but only if the phase hasn't started.
    pub fn sync_settings(&mut self, settings: &TimerSettings) {
        if self.is_pristine() {
            self.duration = settings.duration(self.phase);
        }
    }

    /// Advance the state machine. Call once per frame.
    pub fn tick(&mut self, now: Instant, settings: &TimerSettings) -> Option<TimerEvent> {
        if !self.is_running() || !self.remaining(now).is_zero() {
            return None;
        }

        let finished = self.phase;
        if finished == Phase::Focus {
            self.completed_focus += 1;
        }
        let next = self.next_phase(settings, self.completed_focus);
        self.switch_to(next, settings);

        let auto_start = match next {
            Phase::Focus => settings.auto_start_focus,
            Phase::ShortBreak | Phase::LongBreak => settings.auto_start_breaks,
        };
        if auto_start {
            self.start(now);
        }

        Some(TimerEvent::Finished { finished, next })
    }

    fn next_phase(&self, settings: &TimerSettings, completed: u32) -> Phase {
        match self.phase {
            Phase::Focus => {
                let every = settings.sessions_before_long_break.max(1);
                if completed > 0 && completed.is_multiple_of(every) {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            Phase::ShortBreak | Phase::LongBreak => Phase::Focus,
        }
    }
}

/// `mm:ss`, rounding up so the display hits 00:00 exactly when the phase ends.
pub fn format_remaining(d: Duration) -> String {
    let secs = d.as_secs() + u64::from(d.subsec_nanos() > 0);
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> TimerSettings {
        TimerSettings {
            focus_minutes: 1,
            short_break_minutes: 1,
            long_break_minutes: 2,
            sessions_before_long_break: 2,
            auto_start_breaks: false,
            auto_start_focus: false,
        }
    }

    const MIN: Duration = Duration::from_secs(60);

    #[test]
    fn pause_and_resume_keeps_elapsed_time() {
        let s = settings();
        let t0 = Instant::now();
        let mut t = Timer::new(&s);
        t.start(t0);
        t.pause(t0 + Duration::from_secs(10));
        assert_eq!(
            t.remaining(t0 + Duration::from_secs(100)),
            Duration::from_secs(50)
        );
        t.start(t0 + Duration::from_secs(100));
        assert_eq!(
            t.remaining(t0 + Duration::from_secs(110)),
            Duration::from_secs(40)
        );
    }

    #[test]
    fn cycles_focus_short_focus_long() {
        let s = settings();
        let mut now = Instant::now();
        let mut t = Timer::new(&s);
        let mut seen = vec![];
        for _ in 0..4 {
            t.start(now);
            now += 2 * MIN;
            let Some(TimerEvent::Finished { next, .. }) = t.tick(now, &s) else {
                panic!("phase should have finished");
            };
            seen.push(next);
        }
        assert_eq!(
            seen,
            [
                Phase::ShortBreak,
                Phase::Focus,
                Phase::LongBreak,
                Phase::Focus
            ]
        );
        assert_eq!(t.completed_focus(), 2);
    }

    #[test]
    fn auto_start_respects_settings() {
        let mut s = settings();
        s.auto_start_breaks = true;
        let now = Instant::now();
        let mut t = Timer::new(&s);
        t.start(now);
        t.tick(now + MIN, &s);
        assert!(t.is_running(), "break should auto start");
        t.tick(now + 2 * MIN, &s);
        assert!(!t.is_running(), "focus should not auto start");
    }

    #[test]
    fn skip_does_not_count_session() {
        let s = settings();
        let mut t = Timer::new(&s);
        t.skip(&s);
        assert_eq!(t.phase(), Phase::ShortBreak);
        assert_eq!(t.completed_focus(), 0);
    }

    #[test]
    fn sync_only_when_pristine() {
        let mut s = settings();
        let now = Instant::now();
        let mut t = Timer::new(&s);
        s.focus_minutes = 5;
        t.sync_settings(&s);
        assert_eq!(t.remaining(now), 5 * MIN);
        t.start(now);
        s.focus_minutes = 10;
        t.sync_settings(&s);
        assert_eq!(t.remaining(now), 5 * MIN);
    }

    #[test]
    fn restore_is_paused_with_progress() {
        let s = settings();
        let now = Instant::now();
        let t = Timer::restore(Phase::LongBreak, Duration::from_secs(30), 2, &s);
        assert!(!t.is_running());
        assert_eq!(t.remaining(now), 2 * MIN - Duration::from_secs(30));
        assert_eq!(t.completed_focus(), 2);
    }

    #[test]
    fn formats_rounding_up() {
        assert_eq!(format_remaining(Duration::from_millis(59_001)), "01:00");
        assert_eq!(format_remaining(Duration::from_secs(25 * 60)), "25:00");
        assert_eq!(format_remaining(Duration::ZERO), "00:00");
    }
}
