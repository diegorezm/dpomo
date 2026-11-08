//! Commands that arrive from outside the egui loop (tray menu, tray clicks,
//! a second `dpomo` launch, the config file watcher). Producers send them
//! through an `mpsc` channel and call `ctx.request_repaint()` so `App::logic`
//! wakes up and applies them.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    ShowWindow,
    ToggleWindow,
    ToggleTimer,
    Skip,
    Reset,
    /// Show / hide the floating timer.
    ToggleMini,
    OpenSettings,
    Quit,
    /// `config.toml` changed on disk.
    ReloadConfig,
    /// Something in the user's themes/ or characters/ folder changed.
    ReloadFiles,
    /// A newer release exists (version without the `v`).
    UpdateAvailable(String),
}
