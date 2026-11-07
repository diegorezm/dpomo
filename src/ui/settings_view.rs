//! Settings screen. Edits `Config` in place; `App` decides when to save.

use egui::color_picker::{Alpha, color_edit_button_srgba};
use egui::{
    Align, Color32, CornerRadius, CursorIcon, DragValue, Layout, RichText, Sense, Slider, Stroke,
    Ui, vec2,
};

use super::widgets::{self, Icon};
use std::collections::HashMap;

use crate::character::{CharacterImage, Characters, Pose};
use crate::config::{self, Config};
use crate::theme::{self, Theme, Themes, mix};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Back,
    ResetDefaults,
    ReloadThemes,
    /// Save the custom theme as `<themes dir>/<state.export_name>.toml`.
    ExportCustom,
    OpenConfigFile,
    OpenConfigFolder,
    /// Replace a broken config.toml with the current settings.
    OverwriteConfig,
}

/// Screen-local state owned by `App` (it outlives a single frame).
pub struct State {
    pub export_name: String,
    /// Feedback from the last action, e.g. "Tema salvo em …".
    pub notice: Option<String>,
    /// Why the tray / autostart couldn't be set up, if it failed.
    pub system_error: Option<String>,
    /// `config.toml` is invalid; saving is paused until it's fixed.
    pub config_error: Option<String>,
    /// Rasterized previews for the character picker, by character id.
    pub thumbnails: HashMap<String, CharacterImage>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            export_name: "Meu tema".to_owned(),
            notice: None,
            system_error: None,
            config_error: None,
            thumbnails: HashMap::new(),
        }
    }
}

pub fn show(
    ui: &mut Ui,
    config: &mut Config,
    themes: &Themes,
    characters: &Characters,
    state: &mut State,
    theme: &Theme,
) -> Option<Action> {
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

    if let Some(err) = &state.config_error {
        widgets::card(ui, theme, |ui| {
            ui.label(
                RichText::new("O config.toml tem um erro")
                    .strong()
                    .color(theme.focus),
            );
            ui.label(RichText::new(err).size(12.0));
            ui.label(
                RichText::new(
                    "Corrija o arquivo e salve: o app aplica na hora. Até lá, nada aqui é salvo.",
                )
                .size(11.0)
                .color(theme.muted),
            );
            ui.horizontal(|ui| {
                if ui.button("Abrir config.toml").clicked() {
                    action = Some(Action::OpenConfigFile);
                }
                if ui.button("Substituir pelos ajustes atuais").clicked() {
                    action = Some(Action::OverwriteConfig);
                }
            });
        });
        ui.add_space(8.0);
    }

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

            section(ui, theme, "Aparência", |ui| {
                let a = &mut config.appearance;
                row(ui, "Estilo", |ui| {
                    // Right-to-left layout: last option first.
                    ui.selectable_value(&mut a.layout, config::Layout::Classic, "Clássico");
                    ui.selectable_value(&mut a.layout, config::Layout::Cozy, "Aconchegante");
                });
                match a.layout {
                    config::Layout::Cozy => {
                        ui.label("Personagem");
                        character_picker(
                            ui,
                            &mut a.character,
                            characters,
                            &mut state.thumbnails,
                            theme,
                        );
                    }
                    config::Layout::Classic => {
                        row(ui, "Espessura do anel", |ui| {
                            ui.add(
                                Slider::new(&mut a.ring_thickness, 2.0..=28.0).show_value(false),
                            );
                        });
                        ui.checkbox(&mut a.glow, "Brilho no anel");
                    }
                }
                row(ui, "Escala da interface", |ui| {
                    ui.add(
                        Slider::new(&mut a.ui_scale, 0.75..=1.75)
                            .step_by(0.05)
                            .fixed_decimals(2),
                    );
                });
                ui.checkbox(&mut a.show_session_dots, "Mostrar sessões");
                ui.checkbox(&mut a.smooth_animation, "Animação suave")
                    .on_hover_text("Desligado: redesenha 1x por segundo (gasta menos CPU)");
            });

            section(ui, theme, "Tema", |ui| {
                theme_picker(ui, config, themes);
                if config.appearance.theme == theme::CUSTOM {
                    ui.add_space(6.0);
                    custom_colors(ui, &mut config.custom_theme);
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let save = ui.button("Salvar como arquivo");
                        ui.add(
                            egui::TextEdit::singleline(&mut state.export_name)
                                .hint_text("nome do tema")
                                .desired_width(f32::INFINITY),
                        );
                        if save.clicked() && !state.export_name.trim().is_empty() {
                            action = Some(Action::ExportCustom);
                        }
                    });
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui
                        .button("Recarregar temas e personagens")
                        .on_hover_text("Lê de novo os arquivos das pastas de temas e personagens")
                        .clicked()
                    {
                        action = Some(Action::ReloadThemes);
                    }
                });
                if let Some(dir) = Themes::dir() {
                    ui.label(
                        RichText::new(format!("Seus temas: {}", dir.display()))
                            .size(11.0)
                            .color(theme.muted),
                    );
                }
                if let Some(dir) = Characters::dir() {
                    ui.label(
                        RichText::new(format!("Seus personagens: {}", dir.display()))
                            .size(11.0)
                            .color(theme.muted),
                    );
                }
                for err in themes.errors.iter().chain(&characters.errors) {
                    ui.label(
                        RichText::new(format!("Erro em {err}"))
                            .size(11.0)
                            .color(theme.focus),
                    );
                }
                if let Some(notice) = &state.notice {
                    ui.label(RichText::new(notice).size(11.0).color(theme.short_break));
                }
            });

            section(ui, theme, "Comportamento", |ui| {
                let b = &mut config.behavior;
                ui.checkbox(&mut b.always_on_top, "Sempre no topo");
                ui.checkbox(
                    &mut b.notify_on_finish,
                    "Piscar na barra de tarefas ao terminar",
                );
                ui.checkbox(
                    &mut b.system_notification,
                    "Notificação do sistema ao terminar",
                );
                ui.checkbox(
                    &mut b.focus_on_finish,
                    "Trazer janela para frente ao terminar",
                );
                ui.checkbox(&mut b.time_in_title, "Tempo no título da janela");
            });

            section(ui, theme, "Segundo plano", |ui| {
                let b = &mut config.behavior;
                ui.checkbox(&mut b.tray, "Ícone na bandeja do sistema");
                ui.add_enabled_ui(b.tray, |ui| {
                    ui.checkbox(&mut b.close_to_tray, "Fechar a janela esconde na bandeja")
                        .on_hover_text(
                            "O timer continua rodando. Para sair, use \"Sair\" no menu da bandeja.",
                        );
                    ui.checkbox(&mut b.start_hidden, "Abrir escondido na bandeja");
                });
                ui.checkbox(&mut b.start_with_system, "Iniciar com o sistema")
                    .on_hover_text("Abre escondido na bandeja quando você liga o computador");
                if let Some(err) = &state.system_error {
                    ui.label(RichText::new(err).size(11.0).color(theme.focus));
                }
            });

            ui.add_space(4.0);
            section(ui, theme, "Arquivo de configuração", |ui| {
                ui.label(
                RichText::new(
                    "Tudo aqui fica salvo no config.toml, inclusive atalhos e tamanho da janela. \
                     Edite o arquivo à vontade: o app aplica na hora.",
                )
                .size(12.0),
            );
                if let Some(path) = Config::path() {
                    ui.label(
                        RichText::new(path.display().to_string())
                            .size(11.0)
                            .color(theme.muted),
                    );
                }
                ui.horizontal(|ui| {
                    if ui.button("Abrir config.toml").clicked() {
                        action = Some(Action::OpenConfigFile);
                    }
                    if ui.button("Abrir pasta").clicked() {
                        action = Some(Action::OpenConfigFolder);
                    }
                });
            });
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

fn theme_picker(ui: &mut Ui, config: &mut Config, themes: &Themes) {
    let current = config.theme(themes);
    let entries = themes
        .iter()
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

/// Cards with each character's idle pose, in the current theme's colors.
fn character_picker(
    ui: &mut Ui,
    current: &mut String,
    characters: &Characters,
    thumbnails: &mut HashMap<String, CharacterImage>,
    theme: &Theme,
) {
    let spacing = 8.0;
    let columns = 3.0;
    let width = ((ui.available_width() - spacing * (columns - 1.0)) / columns).floor();
    let art = vec2(width - 12.0, (width - 12.0) / 1.25);
    ui.spacing_mut().item_spacing = vec2(spacing, spacing);
    ui.horizontal_wrapped(|ui| {
        for c in characters.iter() {
            let selected = *current == c.id;
            let (rect, response) =
                ui.allocate_exact_size(vec2(width, art.y + 34.0), Sense::click());
            let hover = ui.ctx().animate_bool(response.id, response.hovered());
            let border = if selected {
                Stroke::new(2.0, theme.focus)
            } else {
                Stroke::new(1.0, mix(theme.surface, theme.text, 0.12 + 0.2 * hover))
            };
            ui.painter().rect(
                rect,
                CornerRadius::same(10),
                theme.background,
                border,
                egui::StrokeKind::Inside,
            );
            let art_rect = egui::Rect::from_min_size(rect.min + vec2(6.0, 6.0), art);
            thumbnails
                .entry(c.id.clone())
                .or_default()
                .image(ui.ctx(), c, Pose::Idle, theme.focus, theme.muted)
                .paint_at(ui, art_rect);
            let name_color = if selected { theme.text } else { theme.muted };
            ui.painter().text(
                egui::pos2(rect.center().x, rect.bottom() - 14.0),
                egui::Align2::CENTER_CENTER,
                &c.name,
                egui::FontId::proportional(13.0),
                name_color,
            );
            if response.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                *current = c.id.clone();
            }
        }
    });
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
