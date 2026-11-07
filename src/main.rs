// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod theme;
mod timer;
mod ui;

fn main() -> eframe::Result {
    let config = config::Config::load();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("dpomo")
            .with_app_id("dpomo")
            .with_inner_size([360.0, 500.0])
            .with_min_inner_size([280.0, 380.0]),
        ..Default::default()
    };

    eframe::run_native(
        "dpomo",
        options,
        Box::new(|cc| Ok(Box::new(app::DpomoApp::new(cc, config)))),
    )
}
