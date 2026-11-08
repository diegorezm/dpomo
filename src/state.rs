//! App state that isn't configuration: the current session (so closing the
//! app or rebooting doesn't lose progress) and daily focus statistics.
//! Stored next to the config as `state.toml`. Written by the app only.

use std::collections::BTreeMap;
use std::path::PathBuf;

use chrono::{Days, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::timer::Phase;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub session: Option<Session>,
    /// Completed focus sessions per local day (`YYYY-MM-DD`).
    pub history: BTreeMap<NaiveDate, DayStats>,
}

/// Where the timer was when the app last saved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub day: NaiveDate,
    pub phase: Phase,
    pub elapsed_secs: u64,
    pub completed_focus: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DayStats {
    pub sessions: u32,
    pub focus_minutes: u32,
}

/// Summary shown in settings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Totals {
    pub today: DayStats,
    pub week: DayStats,
    pub all_time: DayStats,
    /// The last 7 days, oldest first (today last), for the chart.
    pub days: [(NaiveDate, DayStats); 7],
}

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

impl State {
    fn path() -> Option<PathBuf> {
        crate::config::Config::path().and_then(|p| Some(p.parent()?.join("state.toml")))
    }

    /// Never fails: a missing or broken file is just an empty state.
    pub fn load() -> Self {
        Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = Self::path() else { return };
        let result = toml::to_string_pretty(self)
            .map_err(std::io::Error::other)
            .and_then(|text| std::fs::write(path, text));
        if let Err(e) = result {
            eprintln!("dpomo: could not save state: {e}");
        }
    }

    /// The saved session, only if it's from today (a new day starts fresh).
    pub fn todays_session(&self) -> Option<&Session> {
        self.session.as_ref().filter(|s| s.day == today())
    }

    pub fn record_focus(&mut self, day: NaiveDate, minutes: u32) {
        let entry = self.history.entry(day).or_default();
        entry.sessions += 1;
        entry.focus_minutes += minutes;
    }

    pub fn totals(&self, today: NaiveDate) -> Totals {
        let week_start = today.checked_sub_days(Days::new(6)).unwrap_or(today);
        let sum = |filter: &dyn Fn(&NaiveDate) -> bool| {
            self.history.iter().filter(|(d, _)| filter(d)).fold(
                DayStats::default(),
                |acc, (_, s)| DayStats {
                    sessions: acc.sessions + s.sessions,
                    focus_minutes: acc.focus_minutes + s.focus_minutes,
                },
            )
        };
        let days = std::array::from_fn(|i| {
            let day = week_start
                .checked_add_days(Days::new(i as u64))
                .unwrap_or(today);
            (day, self.history.get(&day).copied().unwrap_or_default())
        });
        Totals {
            today: sum(&|d| *d == today),
            week: sum(&|d| (week_start..=today).contains(d)),
            all_time: sum(&|_| true),
            days,
        }
    }
}

/// "1h 05m" / "25m".
pub fn format_minutes(minutes: u32) -> String {
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m}m"),
        (h, m) => format!("{h}h {m:02}m"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    #[test]
    fn totals_cover_today_week_and_all_time() {
        let mut st = State::default();
        st.record_focus(day("2026-11-07"), 25);
        st.record_focus(day("2026-11-07"), 25);
        st.record_focus(day("2026-11-02"), 50);
        st.record_focus(day("2026-10-01"), 25);
        let t = st.totals(day("2026-11-07"));
        assert_eq!(
            t.today,
            DayStats {
                sessions: 2,
                focus_minutes: 50
            }
        );
        assert_eq!(
            t.week,
            DayStats {
                sessions: 3,
                focus_minutes: 100
            }
        );
        assert_eq!(t.all_time.sessions, 4);
        // Chart: 7 days ending today, empty days included.
        assert_eq!(t.days[0].0, day("2026-11-01"));
        assert_eq!(t.days[1].1.focus_minutes, 50);
        assert_eq!(t.days[5].1, DayStats::default());
        assert_eq!(t.days[6], (day("2026-11-07"), t.today));
    }

    #[test]
    fn roundtrips_through_toml() {
        let mut st = State {
            session: Some(Session {
                day: day("2026-11-07"),
                phase: Phase::ShortBreak,
                elapsed_secs: 90,
                completed_focus: 3,
            }),
            ..Default::default()
        };
        st.record_focus(day("2026-11-07"), 25);
        let text = toml::to_string_pretty(&st).unwrap();
        assert_eq!(toml::from_str::<State>(&text).unwrap(), st);
    }

    #[test]
    fn formats_minutes() {
        assert_eq!(format_minutes(25), "25m");
        assert_eq!(format_minutes(65), "1h 05m");
    }
}
