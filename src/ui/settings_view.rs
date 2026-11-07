//! Settings screen. Edits `Config` in place; `App` decides when to save.

use egui::color_picker::{Alpha, color_edit_button_srgba};
use egui::{
    Align, Color32, CornerRadius, CursorIcon, DragValue, Layout, RichText, Sense, Slider, Stroke,
    Ui, vec2,
};

use std::collections::HashMap;

use super::widgets::{self, Icon};
use crate::character::{CharacterImage, Characters, Pose};
use crate::config::{self, Config};
use crate::i18n::{Language, fill, tr};
use crate::sound::SoundKind;
use crate::state::{Totals, format_minutes};
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
    PlaySound,
    OpenUpdatePage,
}

/// Screen-local state owned by `App` (it outlives a single frame).
pub struct State {
    pub export_name: String,
    /// Feedback from the last action, e.g. "Theme saved to …".
    pub notice: Option<String>,
    /// Why the tray / autostart couldn't be set up, if it failed.
    pub system_error: Option<String>,
    /// `config.toml` is invalid; saving is paused until it's fixed.
    pub config_error: Option<String>,
    /// Rasterized previews for the character picker, by character id.
    pub thumbnails: HashMap<String, CharacterImage>,
    /// Newer version found by the update checker.
    pub update: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            export_name: tr().default_export_name.to_owned(),
            notice: None,
            system_error: None,
            config_error: None,
            thumbnails: HashMap::new(),
            update: None,
        }
    }
}

pub fn show(
    ui: &mut Ui,
    config: &mut Config,
    themes: &Themes,
    characters: &Characters,
    totals: &Totals,
    state: &mut State,
    theme: &Theme,
) -> Option<Action> {
    let t = tr();
    let mut action = None;

    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::hover());
        if widgets::icon_button(ui, rect, Icon::Back, theme.text, Color32::TRANSPARENT)
            .on_hover_text(fill(t.hint_back, &[("key", &config.keys.back)]))
            .clicked()
        {
            action = Some(Action::Back);
        }
        ui.label(RichText::new(t.settings).size(20.0).strong());
    });
    ui.add_space(8.0);

    if let Some(err) = &state.config_error {
        widgets::card(ui, theme, |ui| {
            ui.label(
                RichText::new(t.config_error_title)
                    .strong()
                    .color(theme.focus),
            );
            ui.label(RichText::new(err).size(12.0));
            ui.label(
                RichText::new(t.config_error_help)
                    .size(11.0)
                    .color(theme.muted),
            );
            ui.horizontal(|ui| {
                if ui.button(t.open_config).clicked() {
                    action = Some(Action::OpenConfigFile);
                }
                if ui.button(t.overwrite_config).clicked() {
                    action = Some(Action::OverwriteConfig);
                }
            });
        });
        ui.add_space(8.0);
    }

    if let Some(version) = &state.update {
        let text = fill(t.update_badge, &[("version", version)]);
        if ui
            .add(
                egui::Button::new(RichText::new(text).color(theme.background))
                    .fill(theme.short_break),
            )
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            action = Some(Action::OpenUpdatePage);
        }
        ui.add_space(8.0);
    }

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            section(ui, theme, t.sec_timer, |ui| {
                let tm = &mut config.timer;
                minutes_row(ui, t.focus, &mut tm.focus_minutes);
                minutes_row(ui, t.short_break, &mut tm.short_break_minutes);
                minutes_row(ui, t.long_break, &mut tm.long_break_minutes);
                row(ui, t.sessions_before_long_break, |ui| {
                    ui.add(DragValue::new(&mut tm.sessions_before_long_break).range(1..=12));
                });
                ui.checkbox(&mut tm.auto_start_breaks, t.auto_start_breaks);
                ui.checkbox(&mut tm.auto_start_focus, t.auto_start_focus);
            });

            section(ui, theme, t.sec_appearance, |ui| {
                let a = &mut config.appearance;
                row(ui, t.language, |ui| {
                    // Right-to-left layout: last option first.
                    ui.selectable_value(&mut a.language, Language::Portuguese, "Português");
                    ui.selectable_value(&mut a.language, Language::English, "English");
                    ui.selectable_value(&mut a.language, Language::Auto, t.language_auto);
                });
                row(ui, t.style, |ui| {
                    ui.selectable_value(&mut a.layout, config::Layout::Classic, t.style_classic);
                    ui.selectable_value(&mut a.layout, config::Layout::Cozy, t.style_cozy);
                });
                match a.layout {
                    config::Layout::Cozy => {
                        ui.label(t.character);
                        character_picker(
                            ui,
                            &mut a.character,
                            characters,
                            &mut state.thumbnails,
                            theme,
                        );
                    }
                    config::Layout::Classic => {
                        row(ui, t.ring_thickness, |ui| {
                            ui.add(
                                Slider::new(&mut a.ring_thickness, 2.0..=28.0).show_value(false),
                            );
                        });
                        ui.checkbox(&mut a.glow, t.ring_glow);
                    }
                }
                row(ui, t.ui_scale, |ui| {
                    ui.add(
                        Slider::new(&mut a.ui_scale, 0.75..=1.75)
                            .step_by(0.05)
                            .fixed_decimals(2),
                    );
                });
                ui.checkbox(&mut a.show_session_dots, t.show_sessions);
                ui.checkbox(&mut a.smooth_animation, t.smooth_animation)
                    .on_hover_text(t.smooth_animation_hint);
            });

            section(ui, theme, t.sec_theme, |ui| {
                theme_picker(ui, config, themes);
                if config.appearance.theme == theme::CUSTOM {
                    ui.add_space(6.0);
                    custom_colors(ui, &mut config.custom_theme);
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let save = ui.button(t.save_as_file);
                        ui.add(
                            egui::TextEdit::singleline(&mut state.export_name)
                                .hint_text(t.theme_name_hint)
                                .desired_width(f32::INFINITY),
                        );
                        if save.clicked() && !state.export_name.trim().is_empty() {
                            action = Some(Action::ExportCustom);
                        }
                    });
                }
                ui.add_space(4.0);
                if ui
                    .button(t.reload_files)
                    .on_hover_text(t.reload_files_hint)
                    .clicked()
                {
                    action = Some(Action::ReloadThemes);
                }
                let small = |ui: &mut Ui, text: String, color: Color32| {
                    ui.label(RichText::new(text).size(11.0).color(color));
                };
                if let Some(dir) = Themes::dir() {
                    small(
                        ui,
                        fill(t.your_themes, &[("path", &dir.display().to_string())]),
                        theme.muted,
                    );
                }
                if let Some(dir) = Characters::dir() {
                    small(
                        ui,
                        fill(t.your_characters, &[("path", &dir.display().to_string())]),
                        theme.muted,
                    );
                }
                for err in themes.errors.iter().chain(&characters.errors) {
                    small(ui, fill(t.error_in, &[("error", err)]), theme.focus);
                }
                if let Some(notice) = &state.notice {
                    small(ui, notice.clone(), theme.short_break);
                }
            });

            section(ui, theme, t.sec_sound, |ui| {
                let s = &mut config.sound;
                ui.checkbox(&mut s.enabled, t.sound_enabled);
                ui.add_enabled_ui(s.enabled, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for kind in SoundKind::ALL {
                            let label = match kind {
                                SoundKind::Chime => t.sound_chime,
                                SoundKind::Bell => t.sound_bell,
                                SoundKind::Soft => t.sound_soft,
                                SoundKind::File => t.sound_file,
                            };
                            ui.selectable_value(&mut s.kind, kind, label);
                        }
                    });
                    if s.kind == SoundKind::File {
                        ui.add(
                            egui::TextEdit::singleline(&mut s.file)
                                .hint_text(t.sound_file_hint)
                                .desired_width(f32::INFINITY),
                        );
                    }
                    row(ui, t.volume, |ui| {
                        if ui.button(t.play_test).clicked() {
                            action = Some(Action::PlaySound);
                        }
                        ui.add(Slider::new(&mut s.volume, 0.0..=1.0).show_value(false));
                    });
                });
            });

            section(ui, theme, t.sec_stats, |ui| {
                for (label, stats) in [
                    (t.stats_today, totals.today),
                    (t.stats_week, totals.week),
                    (t.stats_total, totals.all_time),
                ] {
                    row(ui, label, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} · {}",
                                t.sessions(stats.sessions),
                                format_minutes(stats.focus_minutes)
                            ))
                            .color(theme.muted),
                        );
                    });
                }
            });

            section(ui, theme, t.sec_behavior, |ui| {
                let b = &mut config.behavior;
                ui.checkbox(&mut b.always_on_top, t.always_on_top);
                ui.checkbox(&mut b.notify_on_finish, t.flash_taskbar);
                ui.checkbox(&mut b.system_notification, t.system_notification);
                ui.checkbox(&mut b.focus_on_finish, t.focus_on_finish);
                ui.checkbox(&mut b.time_in_title, t.time_in_title);
                ui.checkbox(&mut b.check_updates, t.check_updates);
            });

            section(ui, theme, t.sec_tray, |ui| {
                let b = &mut config.behavior;
                ui.checkbox(&mut b.tray, t.tray_icon);
                ui.add_enabled_ui(b.tray, |ui| {
                    ui.checkbox(&mut b.close_to_tray, t.close_to_tray)
                        .on_hover_text(t.close_to_tray_hint);
                    ui.checkbox(&mut b.start_hidden, t.start_hidden);
                });
                ui.checkbox(&mut b.start_with_system, t.start_with_system)
                    .on_hover_text(t.start_with_system_hint);
                if let Some(err) = &state.system_error {
                    ui.label(RichText::new(err).size(11.0).color(theme.focus));
                }
            });

            section(ui, theme, t.sec_config_file, |ui| {
                ui.label(RichText::new(t.config_file_help).size(12.0));
                if let Some(path) = Config::path() {
                    ui.label(
                        RichText::new(path.display().to_string())
                            .size(11.0)
                            .color(theme.muted),
                    );
                }
                ui.horizontal(|ui| {
                    if ui.button(t.open_config).clicked() {
                        action = Some(Action::OpenConfigFile);
                    }
                    if ui.button(t.open_folder).clicked() {
                        action = Some(Action::OpenConfigFolder);
                    }
                });
            });
            ui.horizontal(|ui| {
                if ui.button(t.reset_defaults).clicked() {
                    action = Some(Action::ResetDefaults);
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(concat!("dpomo v", env!("CARGO_PKG_VERSION")))
                            .size(11.0)
                            .color(theme.muted),
                    );
                });
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
                tr().custom_theme
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
                c.display_name(),
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
    let s = tr();
    let fields: [(&str, &mut Color32); 7] = [
        (s.color_background, &mut t.background),
        (s.color_surface, &mut t.surface),
        (s.color_text, &mut t.text),
        (s.color_muted, &mut t.muted),
        (s.focus, &mut t.focus),
        (s.short_break, &mut t.short_break),
        (s.long_break, &mut t.long_break),
    ];
    for (label, color) in fields {
        row(ui, label, |ui| {
            color_edit_button_srgba(ui, color, Alpha::Opaque);
        });
    }
}
