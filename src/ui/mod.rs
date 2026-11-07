//! Views. Each view is a function that draws itself and returns an `Action`
//! enum describing what the user did; `App` applies it. Views never mutate
//! the timer directly, which keeps all state changes in one place.

pub mod cozy_view;
pub mod mini_view;
pub mod settings_view;
pub mod timer_view;
pub mod widgets;
