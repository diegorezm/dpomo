//! Settings screen. Edits `Config` in place; `App` decides when to save.

use egui::color_picker::{Alpha, color_edit_button_srgba};
use egui::{
    Align, Color32, CornerRadius, CursorIcon, DragValue, Layout, RichText, Sense, Slider, Stroke,
    Ui, vec2,
};

use super::widgets::{self, Icon};
use crate::config::Config;
use crate::theme::{self, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Back,
    ResetDefaults,
}

pub fn show(ui: &mut Ui, config: &mut Config, theme: &Theme) -> Option<Action> {
    let mut action = None;

    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::hover());
        if widgets::icon_button(ui, rect, Icon::Back, theme.text, Color32::TRANSPARENT)
            .on_hover_text("Voltar (Esc)")
            .clicked()
        {
            action = Some(Action::Back);
        }
        ui.label(RichText::new("Ajustes").size(20.0).strong());
    });
    ui.add_space(8.0);

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            section(ui, theme, "Timer", |ui| {
                let t = &mut config.timer;
                minutes_row(ui, "Foco", &mut t.focus_minutes);
                minutes_row(ui, "Pausa curta", &mut t.short_break_minutes);
                minutes_row(ui, "Pausa longa", &mut t.long_break_minutes);
                row(ui, "Sessões até a pausa longa", |ui| {
                    ui.add(DragValue::new(&mut t.sessions_before_long_break).range(1..=12));
                });
                ui.checkbox(&mut t.auto_start_breaks, "Iniciar pausas automaticamente");
                ui.checkbox(&mut t.auto_start_focus, "Iniciar foco automaticamente");
            });

            section(ui, theme, "Tema", |ui| {
                theme_picker(ui, config);
                if config.appearance.theme == theme::CUSTOM {
                    ui.add_space(6.0);
                    custom_colors(ui, &mut config.custom_theme);
                }
            });

            section(ui, theme, "Aparência", |ui| {
                let a = &mut config.appearance;
                row(ui, "Espessura do anel", |ui| {
                    ui.add(Slider::new(&mut a.ring_thickness, 2.0..=28.0).show_value(false));
                });
                row(ui, "Escala da interface", |ui| {
                    ui.add(
                        Slider::new(&mut a.ui_scale, 0.75..=1.75)
                            .step_by(0.05)
                            .fixed_decimals(2),
                    );
                });
                ui.checkbox(&mut a.glow, "Brilho no anel");
                ui.checkbox(&mut a.show_session_dots, "Mostrar sessões");
                ui.checkbox(&mut a.smooth_animation, "Animação suave")
                    .on_hover_text("Desligado: redesenha 1x por segundo (gasta menos CPU)");
            });

            section(ui, theme, "Comportamento", |ui| {
                let b = &mut config.behavior;
                ui.checkbox(&mut b.always_on_top, "Sempre no topo");
                ui.checkbox(
                    &mut b.notify_on_finish,
                    "Piscar na barra de tarefas ao terminar",
                );
                ui.checkbox(
                    &mut b.focus_on_finish,
                    "Trazer janela para frente ao terminar",
                );
                ui.checkbox(&mut b.time_in_title, "Tempo no título da janela");
            });

            ui.add_space(4.0);
            if let Some(path) = Config::path() {
                ui.label(
                    RichText::new(format!("Config: {}", path.display()))
                        .size(11.0)
                        .color(theme.muted),
                );
            }
            ui.horizontal(|ui| {
                if ui.button("Restaurar padrões").clicked() {
                    action = Some(Action::ResetDefaults);
                }
            });
            ui.add_space(12.0);
        });

    action
}

fn section(ui: &mut Ui, theme: &Theme, title: &str, add: impl FnOnce(&mut Ui)) {
    ui.label(
        RichText::new(title.to_uppercase())
            .size(11.0)
            .color(theme.muted),
    );
    widgets::card(ui, theme, add);
    ui.add_space(10.0);
}

fn row(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(Layout::right_to_left(Align::Center), add);
    });
}

fn minutes_row(ui: &mut Ui, label: &str, value: &mut u32) {
    row(ui, label, |ui| {
        ui.add(DragValue::new(value).range(1..=180).suffix(" min"));
    });
}

fn theme_picker(ui: &mut Ui, config: &mut Config) {
    let current = config.theme();
    let entries = theme::PRESETS
        .iter()
        .map(|(n, t)| (*n, *t))
        .chain(std::iter::once((theme::CUSTOM, config.custom_theme)));

    let spacing = 8.0;
    let width = ((ui.available_width() - spacing) / 2.0).floor();
    let mut chosen = None;
    ui.spacing_mut().item_spacing = vec2(spacing, spacing);
    ui.horizontal_wrapped(|ui| {
        for (name, t) in entries {
            let selected = config.appearance.theme == name;
            let label = if name == theme::CUSTOM {
                "Personalizado"
            } else {
                name
            };
            if swatch(ui, label, &t, selected, width).clicked() {
                chosen = Some(name);
            }
        }
    });

    if let Some(name) = chosen {
        // Starting a custom theme from whatever is on screen is friendlier
        // than starting from scratch.
        if name == theme::CUSTOM && config.appearance.theme != theme::CUSTOM {
            config.custom_theme = current;
        }
        config.appearance.theme = name.to_owned();
    }
}

fn swatch(ui: &mut Ui, name: &str, t: &Theme, selected: bool, width: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 44.0), Sense::click());
    let hover = ui.ctx().animate_bool(response.id, response.hovered());
    let painter = ui.painter();
    let radius = CornerRadius::same(10);
    let border = if selected {
        Stroke::new(2.0, t.focus)
    } else {
        Stroke::new(1.0, t.surface.gamma_multiply(0.6 + 0.4 * hover))
    };
    painter.rect(rect, radius, t.background, border, egui::StrokeKind::Inside);
    painter.text(
        rect.left_center() + vec2(12.0, 0.0),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::proportional(13.0),
        t.text,
    );
    for (i, c) in [t.focus, t.short_break, t.long_break]
        .into_iter()
        .enumerate()
    {
        painter.circle_filled(
            rect.right_center() - vec2(14.0 + 12.0 * i as f32, 0.0),
            4.5,
            c,
        );
    }
    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn custom_colors(ui: &mut Ui, t: &mut Theme) {
    let fields: [(&str, &mut Color32); 7] = [
        ("Fundo", &mut t.background),
        ("Superfície", &mut t.surface),
        ("Texto", &mut t.text),
        ("Texto secundário", &mut t.muted),
        ("Foco", &mut t.focus),
        ("Pausa curta", &mut t.short_break),
        ("Pausa longa", &mut t.long_break),
    ];
    for (label, color) in fields {
        row(ui, label, |ui| {
            color_edit_button_srgba(ui, color, Alpha::Opaque);
        });
    }
}
