//! Settings window. Edits `Config` in place; `App` decides when to save.

use chrono::Datelike;
use egui::color_picker::{Alpha, color_edit_button_srgba};
use egui::{
    Align, Align2, Color32, CornerRadius, CursorIcon, DragValue, FontId, Layout, Rect, RichText,
    Sense, Slider, Stroke, Ui, pos2, vec2,
};

use std::collections::HashMap;

use super::cozy_view::display_font;
use super::widgets;
use crate::character::{CharacterImage, Characters, Pose};
use crate::config::{self, Config};
use crate::i18n::{Language, fill, tr};
use crate::sound::SoundKind;
use crate::state::{Totals, format_minutes};
use crate::theme::{self, Theme, Themes, mix};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    ResetDefaults,
    ReloadThemes,
    /// Save the custom theme as `<themes dir>/<state.export_name>.toml`.
    ExportCustom,
    OpenConfigFile,
    OpenConfigFolder,
    /// Replace a broken config.toml with the current settings.
    OverwriteConfig,
    /// Test the sound for a break (`false`) or focus (`true`) starting.
    PlaySound {
        focus: bool,
    },
    OpenUpdatePage,
    /// Put the floating timer back in its default corner.
    MiniToCorner,
}

/// The pages listed in the sidebar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Timer,
    Appearance,
    Theme,
    Sound,
    Mini,
    Behavior,
    Tray,
    Stats,
    ConfigFile,
}

impl Section {
    const ALL: [Section; 9] = [
        Section::Timer,
        Section::Appearance,
        Section::Theme,
        Section::Sound,
        Section::Mini,
        Section::Behavior,
        Section::Tray,
        Section::Stats,
        Section::ConfigFile,
    ];

    fn title(self) -> &'static str {
        let t = tr();
        match self {
            Section::Timer => t.sec_timer,
            Section::Appearance => t.sec_appearance,
            Section::Theme => t.sec_theme,
            Section::Sound => t.sec_sound,
            Section::Mini => t.sec_mini,
            Section::Behavior => t.sec_behavior,
            Section::Tray => t.sec_tray,
            Section::Stats => t.sec_stats,
            Section::ConfigFile => t.sec_config_file,
        }
    }
}

/// Window-local state owned by `App` (it outlives a single frame).
pub struct State {
    pub section: Section,
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
            section: Section::Timer,
            export_name: tr().default_export_name.to_owned(),
            notice: None,
            system_error: None,
            config_error: None,
            thumbnails: HashMap::new(),
            update: None,
        }
    }
}

/// The whole settings window: the sections in a sidebar, the selected one on
/// the right. Bigger text than the main window, for readability.
pub fn show(
    ui: &mut Ui,
    config: &mut Config,
    themes: &Themes,
    characters: &Characters,
    totals: &Totals,
    state: &mut State,
    theme: &Theme,
) -> Option<Action> {
    readable_style(ui);
    let mut action = None;

    egui::Panel::left("settings_nav")
        .exact_size(210.0)
        .resizable(false)
        .frame(egui::Frame::new().fill(theme.surface).inner_margin(14))
        .show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(
                RichText::new(tr().settings)
                    .font(display_font(24.0))
                    .color(theme.text),
            );
            ui.add_space(10.0);
            ui.spacing_mut().item_spacing.y = 4.0;
            for section in Section::ALL {
                if nav_item(ui, theme, section.title(), state.section == section).clicked() {
                    state.section = section;
                    state.notice = None;
                }
            }
            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.label(
                    RichText::new(concat!("dpomo v", env!("CARGO_PKG_VERSION")))
                        .size(12.5)
                        .color(theme.muted),
                );
            });
        });

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(theme.background)
                .inner_margin(egui::Margin::symmetric(28, 20)),
        )
        .show(ui, |ui| {
            if let Some(a) = banners(ui, state, theme) {
                action = Some(a);
            }
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(state.section.title())
                            .font(display_font(30.0))
                            .color(theme.text),
                    );
                    ui.add_space(6.0);
                    let a = match state.section {
                        Section::Timer => timer_page(ui, config, theme),
                        Section::Appearance => {
                            appearance_page(ui, config, characters, state, theme)
                        }
                        Section::Theme => theme_page(ui, config, themes, characters, state, theme),
                        Section::Sound => sound_page(ui, config, theme),
                        Section::Mini => mini_page(ui, config, theme),
                        Section::Behavior => behavior_page(ui, config, theme),
                        Section::Tray => tray_page(ui, config, state, theme),
                        Section::Stats => stats_page(ui, totals, theme),
                        Section::ConfigFile => config_page(ui, theme),
                    };
                    if a.is_some() {
                        action = a;
                    }
                    ui.add_space(16.0);
                });
        });

    action
}

/// Larger text and roomier controls than egui's defaults.
fn readable_style(ui: &mut Ui) {
    let style = ui.style_mut();
    for (text_style, size) in [
        (egui::TextStyle::Body, 15.5),
        (egui::TextStyle::Button, 15.5),
        (egui::TextStyle::Small, 12.5),
        (egui::TextStyle::Monospace, 14.0),
    ] {
        if let Some(font) = style.text_styles.get_mut(&text_style) {
            font.size = size;
        }
    }
    style.spacing.item_spacing = vec2(10.0, 10.0);
    style.spacing.button_padding = vec2(10.0, 5.0);
    style.spacing.interact_size.y = 28.0;
    style.spacing.icon_width = 18.0;
    style.spacing.icon_width_inner = 10.0;
    style.spacing.slider_width = 180.0;
}

fn nav_item(ui: &mut Ui, theme: &Theme, title: &str, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
    let hover = ui.ctx().animate_bool(response.id, response.hovered());
    let fill = if selected {
        mix(theme.surface, theme.focus, 0.2)
    } else {
        mix(theme.surface, theme.text, 0.06 * hover)
    };
    ui.painter().rect_filled(rect, CornerRadius::same(10), fill);
    ui.painter().text(
        rect.left_center() + vec2(12.0, 0.0),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(15.5),
        if selected { theme.text } else { theme.muted },
    );
    response.on_hover_cursor(CursorIcon::PointingHand)
}

/// Config error / update notices, shown above every page.
fn banners(ui: &mut Ui, state: &State, theme: &Theme) -> Option<Action> {
    let t = tr();
    let mut action = None;
    if let Some(err) = &state.config_error {
        widgets::card(ui, theme, |ui| {
            ui.label(
                RichText::new(t.config_error_title)
                    .strong()
                    .color(theme.focus),
            );
            ui.label(RichText::new(err).monospace());
            hint(ui, t.config_error_help, theme);
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
    action
}

fn timer_page(ui: &mut Ui, config: &mut Config, theme: &Theme) -> Option<Action> {
    let t = tr();
    let tm = &mut config.timer;
    widgets::card(ui, theme, |ui| {
        minutes_row(ui, t.focus, &mut tm.focus_minutes);
        minutes_row(ui, t.short_break, &mut tm.short_break_minutes);
        minutes_row(ui, t.long_break, &mut tm.long_break_minutes);
        row(ui, t.sessions_before_long_break, |ui| {
            ui.add(DragValue::new(&mut tm.sessions_before_long_break).range(1..=12));
        });
    });
    ui.add_space(8.0);
    widgets::card(ui, theme, |ui| {
        ui.checkbox(&mut tm.auto_start_breaks, t.auto_start_breaks);
        ui.checkbox(&mut tm.auto_start_focus, t.auto_start_focus);
    });
    None
}

fn appearance_page(
    ui: &mut Ui,
    config: &mut Config,
    characters: &Characters,
    state: &mut State,
    theme: &Theme,
) -> Option<Action> {
    let t = tr();
    let a = &mut config.appearance;
    widgets::card(ui, theme, |ui| {
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
        hint(ui, t.style_hint, theme);
    });
    ui.add_space(8.0);
    match a.layout {
        config::Layout::Cozy => {
            ui.label(RichText::new(t.character).strong());
            character_picker(
                ui,
                &mut a.character,
                characters,
                &mut state.thumbnails,
                theme,
            );
        }
        config::Layout::Classic => {
            widgets::card(ui, theme, |ui| {
                row(ui, t.ring_thickness, |ui| {
                    ui.add(Slider::new(&mut a.ring_thickness, 2.0..=28.0).show_value(false));
                });
                ui.checkbox(&mut a.glow, t.ring_glow);
            });
        }
    }
    ui.add_space(8.0);
    widgets::card(ui, theme, |ui| {
        row(ui, t.ui_scale, |ui| {
            ui.add(
                Slider::new(&mut a.ui_scale, 0.75..=1.75)
                    .step_by(0.05)
                    .fixed_decimals(2),
            );
        });
        ui.checkbox(&mut a.show_session_dots, t.show_sessions);
        ui.checkbox(&mut a.smooth_animation, t.smooth_animation);
        hint_indented(ui, t.smooth_animation_hint, theme);
    });
    None
}

fn theme_page(
    ui: &mut Ui,
    config: &mut Config,
    themes: &Themes,
    characters: &Characters,
    state: &mut State,
    theme: &Theme,
) -> Option<Action> {
    let t = tr();
    let mut action = None;
    theme_picker(ui, config, themes);
    if config.appearance.theme == theme::CUSTOM {
        ui.add_space(8.0);
        widgets::card(ui, theme, |ui| {
            custom_colors(ui, &mut config.custom_theme);
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
        });
    }
    ui.add_space(8.0);
    widgets::card(ui, theme, |ui| {
        if ui.button(t.reload_files).clicked() {
            action = Some(Action::ReloadThemes);
        }
        hint(ui, t.reload_files_hint, theme);
        if let Some(dir) = Themes::dir() {
            hint(
                ui,
                &fill(t.your_themes, &[("path", &dir.display().to_string())]),
                theme,
            );
        }
        if let Some(dir) = Characters::dir() {
            hint(
                ui,
                &fill(t.your_characters, &[("path", &dir.display().to_string())]),
                theme,
            );
        }
        for err in themes.errors.iter().chain(&characters.errors) {
            ui.label(RichText::new(fill(t.error_in, &[("error", err)])).color(theme.focus));
        }
        if let Some(notice) = &state.notice {
            ui.label(RichText::new(notice).color(theme.short_break));
        }
    });
    action
}

fn sound_page(ui: &mut Ui, config: &mut Config, theme: &Theme) -> Option<Action> {
    let t = tr();
    let mut action = None;
    let s = &mut config.sound;
    widgets::card(ui, theme, |ui| {
        ui.checkbox(&mut s.enabled, t.sound_enabled);
        ui.add_enabled_ui(s.enabled, |ui| {
            row(ui, t.sound_break_starts, |ui| {
                if ui.button(t.play_test).clicked() {
                    action = Some(Action::PlaySound { focus: false });
                }
            });
            ui.horizontal_wrapped(|ui| {
                for kind in SoundKind::ALL {
                    ui.selectable_value(&mut s.kind, kind, kind_label(kind));
                }
            });
            if s.kind == SoundKind::File {
                file_field(ui, &mut s.file, t.sound_file_hint);
            }
            ui.add_space(6.0);
            row(ui, t.sound_focus_starts, |ui| {
                if ui.button(t.play_test).clicked() {
                    action = Some(Action::PlaySound { focus: true });
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut s.focus_kind, None, t.sound_same);
                for kind in SoundKind::ALL {
                    ui.selectable_value(&mut s.focus_kind, Some(kind), kind_label(kind));
                }
            });
            if s.focus_kind == Some(SoundKind::File) {
                file_field(ui, &mut s.focus_file, t.sound_focus_file_hint);
            }
            ui.add_space(6.0);
            row(ui, t.volume, |ui| {
                ui.add(Slider::new(&mut s.volume, 0.0..=1.0).show_value(false));
            });
        });
    });
    widgets::card(ui, theme, |ui| {
        ui.add_enabled_ui(s.enabled, |ui| {
            ui.checkbox(&mut s.ticking, t.ticking);
            hint_indented(ui, t.ticking_hint, theme);
            ui.add_enabled_ui(s.ticking, |ui| {
                row(ui, t.volume, |ui| {
                    ui.add(Slider::new(&mut s.ticking_volume, 0.0..=1.0).show_value(false));
                });
            });
        });
    });
    action
}

fn kind_label(kind: SoundKind) -> &'static str {
    let t = tr();
    match kind {
        SoundKind::Chime => t.sound_chime,
        SoundKind::Bell => t.sound_bell,
        SoundKind::Soft => t.sound_soft,
        SoundKind::File => t.sound_file,
    }
}

fn file_field(ui: &mut Ui, path: &mut String, hint: &str) {
    ui.add(
        egui::TextEdit::singleline(path)
            .hint_text(hint)
            .desired_width(f32::INFINITY),
    );
}

fn mini_page(ui: &mut Ui, config: &mut Config, theme: &Theme) -> Option<Action> {
    let t = tr();
    let mut action = None;
    let key = config.keys.mini.clone();
    let m = &mut config.mini;
    widgets::card(ui, theme, |ui| {
        ui.checkbox(&mut m.enabled, t.mini_enabled);
        hint_indented(ui, t.mini_help, theme);
        ui.add_enabled_ui(m.enabled, |ui| {
            ui.checkbox(&mut m.show_character, t.mini_show_character);
            if ui.button(t.mini_to_corner).clicked() {
                action = Some(Action::MiniToCorner);
            }
        });
        hint(ui, &fill(t.mini_shortcut, &[("key", &key)]), theme);
    });
    action
}

fn behavior_page(ui: &mut Ui, config: &mut Config, theme: &Theme) -> Option<Action> {
    let t = tr();
    let b = &mut config.behavior;
    widgets::card(ui, theme, |ui| {
        ui.checkbox(&mut b.system_notification, t.system_notification);
        ui.checkbox(&mut b.notify_on_finish, t.flash_taskbar);
        ui.checkbox(&mut b.focus_on_finish, t.focus_on_finish);
    });
    ui.add_space(8.0);
    widgets::card(ui, theme, |ui| {
        ui.checkbox(&mut b.always_on_top, t.always_on_top);
        ui.checkbox(&mut b.time_in_title, t.time_in_title);
        ui.checkbox(&mut b.check_updates, t.check_updates);
    });
    None
}

fn tray_page(ui: &mut Ui, config: &mut Config, state: &State, theme: &Theme) -> Option<Action> {
    let t = tr();
    let b = &mut config.behavior;
    widgets::card(ui, theme, |ui| {
        ui.checkbox(&mut b.tray, t.tray_icon);
        ui.add_enabled_ui(b.tray, |ui| {
            ui.checkbox(&mut b.close_to_tray, t.close_to_tray);
            hint_indented(ui, t.close_to_tray_hint, theme);
            ui.checkbox(&mut b.start_hidden, t.start_hidden);
        });
    });
    ui.add_space(8.0);
    widgets::card(ui, theme, |ui| {
        ui.checkbox(&mut b.start_with_system, t.start_with_system);
        hint_indented(ui, t.start_with_system_hint, theme);
        if let Some(err) = &state.system_error {
            ui.label(RichText::new(err).color(theme.focus));
        }
    });
    None
}

fn stats_page(ui: &mut Ui, totals: &Totals, theme: &Theme) -> Option<Action> {
    let t = tr();
    widgets::card(ui, theme, |ui| {
        ui.label(RichText::new(t.stats_chart).strong());
        ui.add_space(6.0);
        week_chart(ui, totals, theme);
    });
    ui.add_space(8.0);
    widgets::card(ui, theme, |ui| {
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
    None
}

/// Focus minutes of the last 7 days as bars, today on the right.
fn week_chart(ui: &mut Ui, totals: &Totals, theme: &Theme) {
    let t = tr();
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 150.0), Sense::hover());
    let painter = ui.painter_at(rect);
    // A full bar is the best day, or one default session if that's more.
    let max = totals
        .days
        .iter()
        .map(|(_, s)| s.focus_minutes)
        .max()
        .unwrap_or(0)
        .max(25) as f32;
    let (top, bottom) = (rect.top() + 18.0, rect.bottom() - 20.0);
    let slot = rect.width() / 7.0;
    let width = (slot * 0.55).min(34.0);
    for (i, (day, stats)) in totals.days.iter().enumerate() {
        let x = rect.left() + slot * (i as f32 + 0.5);
        let track = Rect::from_x_y_ranges(x - width / 2.0..=x + width / 2.0, top..=bottom);
        painter.rect_filled(track, 6.0, mix(theme.surface, theme.muted, 0.15));
        if stats.focus_minutes > 0 {
            let height = ((bottom - top) * stats.focus_minutes as f32 / max).max(8.0);
            let bar = Rect::from_x_y_ranges(track.x_range(), bottom - height..=bottom);
            painter.rect_filled(bar, 6.0, theme.focus);
            painter.text(
                pos2(x, bar.top() - 3.0),
                Align2::CENTER_BOTTOM,
                format_minutes(stats.focus_minutes),
                FontId::proportional(12.0),
                theme.text,
            );
        }
        let today = i == totals.days.len() - 1;
        painter.text(
            pos2(x, rect.bottom()),
            Align2::CENTER_BOTTOM,
            t.weekdays[day.weekday().num_days_from_monday() as usize],
            FontId::proportional(12.5),
            if today { theme.text } else { theme.muted },
        );
    }
}

fn config_page(ui: &mut Ui, theme: &Theme) -> Option<Action> {
    let t = tr();
    let mut action = None;
    widgets::card(ui, theme, |ui| {
        ui.add(egui::Label::new(t.config_file_help).wrap());
        if let Some(path) = Config::path() {
            ui.label(
                RichText::new(path.display().to_string())
                    .monospace()
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
    ui.add_space(8.0);
    if ui.button(t.reset_defaults).clicked() {
        action = Some(Action::ResetDefaults);
    }
    action
}

/// A muted explanation under a setting.
fn hint(ui: &mut Ui, text: &str, theme: &Theme) {
    ui.add(egui::Label::new(RichText::new(text).size(13.5).color(theme.muted)).wrap());
}

/// Like `hint`, lined up with a checkbox's text.
fn hint_indented(ui: &mut Ui, text: &str, theme: &Theme) {
    ui.horizontal(|ui| {
        let indent = ui.spacing().icon_width + ui.spacing().icon_spacing;
        ui.add_space(indent);
        hint(ui, text, theme);
    });
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

    let spacing = 10.0;
    let columns = if ui.available_width() > 520.0 {
        3.0
    } else {
        2.0
    };
    let width = ((ui.available_width() - spacing * (columns - 1.0)) / columns).floor();
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
    let spacing = 10.0;
    let columns = 3.0;
    let width = ((ui.available_width() - spacing * (columns - 1.0)) / columns)
        .min(180.0)
        .floor();
    let art = vec2(width - 12.0, (width - 12.0) / 1.25);
    ui.spacing_mut().item_spacing = vec2(spacing, spacing);
    ui.horizontal_wrapped(|ui| {
        for c in characters.iter() {
            let selected = *current == c.id;
            let (rect, response) =
                ui.allocate_exact_size(vec2(width, art.y + 36.0), Sense::click());
            let hover = ui.ctx().animate_bool(response.id, response.hovered());
            let border = if selected {
                Stroke::new(2.0, theme.focus)
            } else {
                Stroke::new(1.0, mix(theme.surface, theme.text, 0.12 + 0.2 * hover))
            };
            ui.painter().rect(
                rect,
                CornerRadius::same(12),
                theme.surface,
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
                egui::pos2(rect.center().x, rect.bottom() - 15.0),
                egui::Align2::CENTER_CENTER,
                c.display_name(),
                egui::FontId::proportional(14.5),
                name_color,
            );
            if response.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                *current = c.id.clone();
            }
        }
    });
}

fn swatch(ui: &mut Ui, name: &str, t: &Theme, selected: bool, width: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 50.0), Sense::click());
    let hover = ui.ctx().animate_bool(response.id, response.hovered());
    let painter = ui.painter();
    let radius = CornerRadius::same(12);
    let border = if selected {
        Stroke::new(2.0, t.focus)
    } else {
        Stroke::new(1.0, t.surface.gamma_multiply(0.6 + 0.4 * hover))
    };
    painter.rect(rect, radius, t.background, border, egui::StrokeKind::Inside);
    painter.text(
        rect.left_center() + vec2(14.0, 0.0),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::proportional(14.5),
        t.text,
    );
    for (i, c) in [t.focus, t.short_break, t.long_break]
        .into_iter()
        .enumerate()
    {
        painter.circle_filled(
            rect.right_center() - vec2(16.0 + 13.0 * i as f32, 0.0),
            5.0,
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
