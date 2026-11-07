// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod character;
mod command;
mod config;
mod i18n;
mod instance;
mod sound;
mod state;
mod system;
mod theme;
mod timer;
mod tray;
mod ui;
mod updates;

use instance::Instance;

fn main() -> eframe::Result {
    // A second launch hands its request (default: "show") to the one already
    // running and exits. `dpomo --toggle` etc. work for OS-level shortcuts.
    let request = instance::request_from_args();
    let listener = match instance::acquire(request) {
        Instance::Primary(listener) => Some(listener),
        Instance::AlreadyRunning => return Ok(()),
        Instance::Unknown => None,
    };
    // Not running yet: `--quit` has nothing to do; other requests run after startup.
    if request == "quit" {
        return Ok(());
    }
    let initial_command =
        instance::command_for(request).filter(|c| *c != command::Command::ShowWindow);

    // Messages produced while loading follow the OS language until the
    // config says otherwise.
    i18n::set(i18n::Language::Auto);
    let (config, config_error) = config::Config::load();
    i18n::set(config.appearance.language);
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
                initial_command,
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
