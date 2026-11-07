//! Commands that arrive from outside the egui loop (tray menu, tray clicks,
//! a second `dpomo` launch). Producers send them through an `mpsc` channel and
//! call `ctx.request_repaint()` so `App::logic` wakes up and applies them.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    ShowWindow,
    ToggleWindow,
    ToggleTimer,
    Skip,
    Reset,
    Quit,
}
