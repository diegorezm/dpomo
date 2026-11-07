// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod command;
mod config;
mod instance;
mod system;
mod theme;
mod timer;
mod tray;
mod ui;

use instance::Instance;

fn main() -> eframe::Result {
    // A second launch just shows the window of the one already running.
    let listener = match instance::acquire() {
        Instance::Primary(listener) => Some(listener),
        Instance::AlreadyRunning => return Ok(()),
        Instance::Unknown => None,
    };

    let config = config::Config::load();
    system::init();
    // `--hidden` is what the autostart entry passes.
    let start_hidden = config.behavior.tray
        && (config.behavior.start_hidden || std::env::args().any(|a| a == "--hidden"));

    let icon_size = 64;
    let icon = egui::IconData {
        rgba: tray::render_ring(icon_size, 1.0, theme::Theme::default().focus, false),
        width: icon_size,
        height: icon_size,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("dpomo")
            .with_app_id("dpomo")
            .with_icon(icon)
            .with_visible(!start_hidden)
            .with_inner_size([360.0, 500.0])
            .with_min_inner_size([280.0, 380.0]),
        ..Default::default()
    };

    eframe::run_native(
        "dpomo",
        options,
        Box::new(move |cc| {
            Ok(Box::new(app::DpomoApp::new(
                cc,
                config,
                listener,
                start_hidden,
            )))
        }),
    )
}
