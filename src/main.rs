// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod character;
mod command;
mod config;
#[cfg(test)]
mod docs;
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

use std::cell::RefCell;
use std::net::TcpListener;
use std::rc::Rc;

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

    let icon = std::sync::Arc::new(window_icon());
    let viewport = egui::ViewportBuilder::default()
        .with_title("dpomo")
        .with_app_id("dpomo")
        .with_icon(icon.clone())
        .with_visible(!start_hidden)
        .with_inner_size([
            config.window.width.max(config::Window::MIN_SIZE[0]),
            config.window.height.max(config::Window::MIN_SIZE[1]),
        ])
        .with_min_inner_size(config::Window::MIN_SIZE);

    // Handed to the app by whichever renderer manages to start.
    let startup = Rc::new(RefCell::new(Some(Startup {
        config,
        listener,
        start_hidden,
        config_error,
        initial_command,
        icon,
    })));

    // OpenGL (glow) first: by far the lightest. `DPOMO_RENDERER=wgpu` skips it.
    let force_wgpu = cfg!(windows) && std::env::var("DPOMO_RENDERER").is_ok_and(|r| r == "wgpu");
    let result = if force_wgpu {
        Err(eframe::Error::AppCreation(
            "glow skipped (DPOMO_RENDERER=wgpu)".into(),
        ))
    } else {
        run(eframe::Renderer::Glow, viewport.clone(), &startup)
    };

    // No usable OpenGL (virtual machines, remote desktops, Windows on ARM
    // without the GL compatibility pack): Direct3D 12 through wgpu, which
    // also has a software fallback. Heavier, but it opens.
    #[cfg(windows)]
    if let Err(e) = &result
        && startup.borrow().is_some()
    {
        eprintln!("dpomo: OpenGL unavailable ({e}), trying Direct3D 12");
        return run(eframe::Renderer::Wgpu, viewport, &startup);
    }
    result
}

/// What `DpomoApp::new` needs besides the egui context.
struct Startup {
    config: config::Config,
    listener: Option<TcpListener>,
    start_hidden: bool,
    config_error: Option<String>,
    initial_command: Option<command::Command>,
    icon: std::sync::Arc<egui::IconData>,
}

fn run(
    renderer: eframe::Renderer,
    viewport: egui::ViewportBuilder,
    startup: &Rc<RefCell<Option<Startup>>>,
) -> eframe::Result {
    let startup = Rc::clone(startup);
    let options = eframe::NativeOptions {
        viewport,
        renderer,
        ..Default::default()
    };
    eframe::run_native(
        "dpomo",
        options,
        Box::new(move |cc| {
            let s = startup
                .borrow_mut()
                .take()
                .expect("the app is created once");
            Ok(Box::new(app::DpomoApp::new(
                cc,
                s.config,
                s.listener,
                s.start_hidden,
                s.config_error,
                s.initial_command,
                s.icon,
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
