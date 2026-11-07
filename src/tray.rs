//! System tray icon with a live progress ring and a small menu.

use std::f32::consts::TAU;
use std::sync::mpsc::Sender;

use egui::Color32;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::command::Command;

const ICON_SIZE: u32 = 32;
/// The tray ring is redrawn only when progress crosses one of these steps.
const PROGRESS_STEPS: f32 = 64.0;

/// Everything the tray shows. `Tray::update` is a no-op if this didn't change.
#[derive(Clone, Debug, PartialEq)]
pub struct TrayState {
    pub tooltip: String,
    pub progress: f32,
    pub running: bool,
    pub window_visible: bool,
    pub accent: Color32,
}

pub struct Tray {
    icon: TrayIcon,
    show_item: MenuItem,
    toggle_item: MenuItem,
    /// Last state applied, quantized, to skip redundant OS calls.
    last: Option<(TrayState, u16)>,
}

impl Tray {
    pub fn new(ctx: &egui::Context, tx: Sender<Command>) -> Result<Self, String> {
        let show_item = MenuItem::with_id("show", "Mostrar", true, None);
        let toggle_item = MenuItem::with_id("toggle", "Iniciar", true, None);
        let menu = Menu::new();
        menu.append_items(&[
            &show_item,
            &PredefinedMenuItem::separator(),
            &toggle_item,
            &MenuItem::with_id("skip", "Pular", true, None),
            &MenuItem::with_id("reset", "Reiniciar", true, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id("quit", "Sair", true, None),
        ])
        .map_err(|e| e.to_string())?;

        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip("dpomo")
            .with_icon(ring_icon(1.0, Color32::from_rgb(0xeb, 0x6f, 0x92), false))
            .build()
            .map_err(|e| e.to_string())?;

        let (menu_tx, menu_ctx) = (tx.clone(), ctx.clone());
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            let cmd = match e.id.0.as_str() {
                "show" => Command::ToggleWindow,
                "toggle" => Command::ToggleTimer,
                "skip" => Command::Skip,
                "reset" => Command::Reset,
                "quit" => Command::Quit,
                _ => return,
            };
            let _ = menu_tx.send(cmd);
            menu_ctx.request_repaint();
        }));

        let tray_ctx = ctx.clone();
        TrayIconEvent::set_event_handler(Some(move |e: TrayIconEvent| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = e
            {
                let _ = tx.send(Command::ToggleWindow);
                tray_ctx.request_repaint();
            }
        }));

        Ok(Self {
            icon,
            show_item,
            toggle_item,
            last: None,
        })
    }

    pub fn update(&mut self, state: TrayState) {
        let step = (state.progress.clamp(0.0, 1.0) * PROGRESS_STEPS).ceil() as u16;
        let (old, old_step) = match &self.last {
            Some((s, st)) => (Some(s), *st),
            None => (None, u16::MAX),
        };

        let icon_changed = old_step != step
            || old.is_none_or(|o| o.running != state.running || o.accent != state.accent);
        if icon_changed {
            let progress = f32::from(step) / PROGRESS_STEPS;
            let _ = self
                .icon
                .set_icon(Some(ring_icon(progress, state.accent, state.running)));
        }
        if old.is_none_or(|o| o.tooltip != state.tooltip) {
            let _ = self.icon.set_tooltip(Some(&state.tooltip));
        }
        if old.is_none_or(|o| o.running != state.running) {
            self.toggle_item
                .set_text(if state.running { "Pausar" } else { "Iniciar" });
        }
        if old.is_none_or(|o| o.window_visible != state.window_visible) {
            self.show_item.set_text(if state.window_visible {
                "Esconder"
            } else {
                "Mostrar"
            });
        }
        self.last = Some((state, step));
    }
}

fn ring_icon(progress: f32, accent: Color32, running: bool) -> Icon {
    let rgba = render_ring(ICON_SIZE, progress, accent, running);
    Icon::from_rgba(rgba, ICON_SIZE, ICON_SIZE).expect("valid icon size")
}

/// Anti-aliased ring as RGBA8 pixels. `progress` fills clockwise from 12
/// o'clock; the rest is a faint track. A center dot means "running".
/// Also used for the window icon.
pub fn render_ring(size: u32, progress: f32, accent: Color32, running: bool) -> Vec<u8> {
    let c = size as f32 / 2.0;
    let outer = c - 0.5;
    let inner = outer - size as f32 * 0.2;
    let dot = size as f32 * 0.16;
    let [r, g, b, _] = accent.to_array();

    let mut px = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let (dx, dy) = (x as f32 + 0.5 - c, y as f32 + 0.5 - c);
            let d = dx.hypot(dy);
            let ring = (outer - d + 0.5).clamp(0.0, 1.0) * (d - inner + 0.5).clamp(0.0, 1.0);
            let center = if running {
                (dot - d + 0.5).clamp(0.0, 1.0)
            } else {
                0.0
            };

            let t = (dx.atan2(-dy) / TAU).rem_euclid(1.0);
            let ring_alpha = if t <= progress { 1.0 } else { 0.3 };
            let alpha = (ring * ring_alpha).max(center);
            px.extend_from_slice(&[r, g, b, (alpha * 255.0).round() as u8]);
        }
    }
    px
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha(px: &[u8], size: u32, x: u32, y: u32) -> u8 {
        px[((y * size + x) * 4 + 3) as usize]
    }

    #[test]
    fn ring_fills_clockwise_from_top() {
        let size = 32;
        let px = render_ring(size, 0.25, Color32::RED, false);
        // Right side (3 o'clock region, just inside 25%) is solid.
        assert_eq!(alpha(&px, size, 29, 14), 255);
        // Left side (9 o'clock) is only the faint track.
        assert!(alpha(&px, size, 2, 16) < 100);
        // Center is empty when paused.
        assert_eq!(alpha(&px, size, 16, 16), 0);
    }

    #[test]
    fn running_shows_center_dot() {
        let px = render_ring(32, 1.0, Color32::RED, true);
        assert_eq!(alpha(&px, 32, 16, 16), 255);
    }
}
