// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod character;
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

    let (config, config_error) = config::Config::load();
    system::init();
    // `--hidden` is what the autostart entry passes.
    let start_hidden = config.behavior.tray
        && (config.behavior.start_hidden || std::env::args().any(|a| a == "--hidden"));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("dpomo")
            .with_app_id("dpomo")
            .with_icon(window_icon())
            .with_visible(!start_hidden)
            .with_inner_size([
                config.window.width.max(280.0),
                config.window.height.max(380.0),
            ])
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
                config_error,
            )))
        }),
    )
}

/// The app icon (assets/icon.svg), rasterized once for the window/taskbar.
fn window_icon() -> egui::IconData {
    let size = egui::SizeHint::Size {
        width: 128,
        height: 128,
        maintain_aspect_ratio: true,
    };
    let image = egui_extras::image::load_svg_bytes_with_size(
        include_bytes!("../assets/icon.svg"),
        size,
        &Default::default(),
    )
    .expect("bundled icon is valid");
    egui::IconData {
        rgba: image
            .pixels
            .iter()
            .flat_map(|c| c.to_srgba_unmultiplied())
            .collect(),
        width: image.size[0] as u32,
        height: image.size[1] as u32,
    }
}
