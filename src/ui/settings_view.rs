//! Settings window. Edits `Config` in place; `App` decides when to save.
//!
//! Layout: a sidebar (search + pages), a top bar (breadcrumb + restore defaults) and
//! the page: titled groups of rows, each row = title and explanation on the left,
//! its control on the right. Every row goes through `Form`, which is what makes
//! search work: with a query, all pages are drawn and only matching rows show.

use chrono::Datelike;
use egui::color_picker::{Alpha, color_edit_button_srgba};
use egui::epaint::PathShape;
use egui::{
    Align, Align2, Color32, CornerRadius, CursorIcon, DragValue, FontId, Key, Layout, Margin,
    Painter, Pos2, Rect, RichText, Sense, Slider, Stroke, Ui, UiBuilder, pos2, vec2,
};

use std::collections::HashMap;

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
    Sound,
    Mini,
    Appearance,
    Theme,
    Behavior,
    Tray,
    Stats,
    ConfigFile,
}

impl Section {
    /// Sidebar order, in groups (the first one has no heading).
    fn groups() -> [(Option<&'static str>, &'static [Section]); 3] {
        let t = tr();
        [
            (None, &[Section::Timer, Section::Sound, Section::Mini]),
            (Some(t.nav_look), &[Section::Appearance, Section::Theme]),
            (
                Some(t.nav_app),
                &[
                    Section::Behavior,
                    Section::Tray,
                    Section::Stats,
                    Section::ConfigFile,
                ],
            ),
        ]
    }

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
    /// Text in the search box; empty = show the selected page.
    pub query: String,
    /// When "Restore defaults" was clicked once (it needs a second click).
    pub confirm_reset_at: Option<f64>,
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
            query: String::new(),
            confirm_reset_at: None,
            export_name: tr().default_export_name.to_owned(),
            notice: None,
            system_error: None,
            config_error: None,
            thumbnails: HashMap::new(),
            update: None,
        }
    }
}

/// Everything a page needs besides the config.
struct Ctx<'a> {
    themes: &'a Themes,
    characters: &'a Characters,
    totals: &'a Totals,
    state: &'a mut State,
}

/// The whole settings window.
pub fn show(
    ui: &mut Ui,
    config: &mut Config,
    themes: &Themes,
    characters: &Characters,
    totals: &Totals,
    state: &mut State,
    theme: &Theme,
) -> Option<Action> {
    readable_style(ui, theme);
    let mut action = None;

    egui::Panel::left("settings_nav")
        .exact_size(236.0)
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(theme.surface)
                .inner_margin(Margin::symmetric(14, 16)),
        )
        .show(ui, |ui| sidebar(ui, state, theme));

    egui::Panel::top("settings_top")
        .exact_size(54.0)
        .frame(
            egui::Frame::new()
                .fill(theme.background)
                .inner_margin(Margin::symmetric(22, 0)),
        )
        .show(ui, |ui| {
            if top_bar(ui, state, theme) {
                action = Some(Action::ResetDefaults);
            }
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(theme.background))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    let width = ui.available_width().min(760.0) - 48.0;
                    let side = (ui.available_width() - width) / 2.0;
                    ui.horizontal(|ui| {
                        ui.add_space(side);
                        ui.vertical(|ui| {
                            ui.set_width(width);
                            ui.add_space(22.0);
                            if let Some(a) = banners(ui, state, theme) {
                                action = Some(a);
                            }
                            let mut cx = Ctx {
                                themes,
                                characters,
                                totals,
                                state,
                            };
                            if let Some(a) = pages(ui, config, &mut cx, theme) {
                                action = Some(a);
                            }
                            ui.add_space(28.0);
                        });
                    });
                });
        });

    action
}

/// The selected page, or every page's matching rows while searching.
fn pages(ui: &mut Ui, config: &mut Config, cx: &mut Ctx, theme: &Theme) -> Option<Action> {
    let mut form = Form::new(theme, &cx.state.query);
    let sections: Vec<Section> = if form.searching() {
        Section::groups()
            .iter()
            .flat_map(|(_, s)| s.iter().copied())
            .collect()
    } else {
        vec![cx.state.section]
    };
    let mut action = None;
    for section in sections {
        form.page = section.title();
        let a = match section {
            Section::Timer => timer_page(ui, &mut form, config),
            Section::Sound => sound_page(ui, &mut form, config),
            Section::Mini => mini_page(ui, &mut form, config),
            Section::Appearance => appearance_page(ui, &mut form, config, cx),
            Section::Theme => theme_page(ui, &mut form, config, cx),
            Section::Behavior => behavior_page(ui, &mut form, config),
            Section::Tray => tray_page(ui, &mut form, config, cx),
            Section::Stats => stats_page(ui, &mut form, cx),
            Section::ConfigFile => config_page(ui, &mut form),
        };
        action = action.or(a);
    }
    if form.searching() && form.groups == 0 {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(fill(tr().no_results, &[("query", cx.state.query.trim())]))
                    .color(theme.muted),
            );
        });
    }
    action
}

// ---------------------------------------------------------------- frame

/// Larger text and roomier controls than egui's defaults.
fn readable_style(ui: &mut Ui, theme: &Theme) {
    let style = ui.style_mut();
    for (text_style, size) in [
        (egui::TextStyle::Body, 15.0),
        (egui::TextStyle::Button, 15.0),
        (egui::TextStyle::Small, 12.5),
        (egui::TextStyle::Monospace, 13.5),
    ] {
        if let Some(font) = style.text_styles.get_mut(&text_style) {
            font.size = size;
        }
    }
    style.spacing.item_spacing = vec2(8.0, 8.0);
    style.spacing.button_padding = vec2(12.0, 5.0);
    style.spacing.interact_size.y = 30.0;
    style.spacing.icon_width = 18.0;
    style.spacing.icon_width_inner = 10.0;
    style.spacing.slider_width = 200.0;
    style.spacing.combo_width = 200.0;
    // Buttons and dropdowns: a quiet field with a hairline border, like the cards.
    let field = mix(theme.background, theme.surface, 0.5);
    let line = line_color(theme);
    let w = &mut style.visuals.widgets;
    w.inactive.weak_bg_fill = field;
    w.inactive.bg_stroke = Stroke::new(1.0, line);
    w.inactive.corner_radius = CornerRadius::same(8);
    w.hovered.weak_bg_fill = mix(field, theme.text, 0.06);
    w.hovered.corner_radius = CornerRadius::same(8);
    w.active.corner_radius = CornerRadius::same(8);
    w.open.corner_radius = CornerRadius::same(8);
}

fn line_color(theme: &Theme) -> Color32 {
    mix(theme.surface, theme.text, 0.1)
}

fn sidebar(ui: &mut Ui, state: &mut State, theme: &Theme) {
    search_box(ui, state, theme);
    ui.add_space(14.0);
    ui.spacing_mut().item_spacing.y = 2.0;
    for (heading, sections) in Section::groups() {
        if let Some(heading) = heading {
            ui.add_space(14.0);
            ui.label(RichText::new(heading).size(13.0).color(theme.muted));
            ui.add_space(2.0);
        }
        for &section in sections {
            let selected = state.query.is_empty() && state.section == section;
            if nav_item(ui, theme, section, selected).clicked() {
                state.section = section;
                state.query.clear();
                state.notice = None;
            }
        }
    }
    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        ui.label(
            RichText::new(concat!("dpomo v", env!("CARGO_PKG_VERSION")))
                .size(12.5)
                .color(theme.muted),
        );
    });
}

/// The search field. `/` focuses it from anywhere in the window.
fn search_box(ui: &mut Ui, state: &mut State, theme: &Theme) {
    let id = ui.make_persistent_id("settings_search");
    let focused = ui.memory(|m| m.has_focus(id));
    if !focused
        && ui.memory(|m| m.focused().is_none())
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Slash))
    {
        ui.memory_mut(|m| m.request_focus(id));
    }
    let border = Stroke::new(
        1.0,
        if focused {
            theme.focus
        } else {
            line_color(theme)
        },
    );
    egui::Frame::new()
        .fill(mix(theme.surface, theme.background, 0.5))
        .stroke(border)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(16.0, 26.0), Sense::hover());
                search_icon(ui.painter(), rect.center(), theme.muted);
                let edit = egui::TextEdit::singleline(&mut state.query)
                    .id(id)
                    .frame(egui::Frame::NONE)
                    .hint_text(tr().search_hint)
                    .desired_width(ui.available_width() - 26.0)
                    .min_size(vec2(0.0, 26.0))
                    .align(Align2::LEFT_CENTER)
                    .margin(Margin::symmetric(2, 0));
                ui.add(edit);
                let (rect, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
                if !focused && state.query.is_empty() {
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(5),
                        mix(theme.surface, theme.text, 0.08),
                        Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        "/",
                        FontId::proportional(12.0),
                        theme.muted,
                    );
                }
            });
        });
}

fn nav_item(ui: &mut Ui, theme: &Theme, section: Section, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::click());
    let hover = ui.ctx().animate_bool(response.id, response.hovered());
    let fill = if selected {
        mix(theme.surface, theme.focus, 0.18)
    } else {
        mix(theme.surface, theme.text, 0.06 * hover)
    };
    ui.painter().rect_filled(rect, CornerRadius::same(9), fill);
    let color = if selected { theme.text } else { theme.muted };
    nav_icon(
        ui.painter(),
        section,
        rect.left_center() + vec2(20.0, 0.0),
        color,
    );
    ui.painter().text(
        rect.left_center() + vec2(38.0, 0.0),
        Align2::LEFT_CENTER,
        section.title(),
        FontId::proportional(15.0),
        if selected {
            theme.text
        } else {
            mix(theme.muted, theme.text, 0.35 * hover)
        },
    );
    response.on_hover_cursor(CursorIcon::PointingHand)
}

/// Breadcrumb and "Restore defaults". Returns true when the reset is confirmed.
fn top_bar(ui: &mut Ui, state: &mut State, theme: &Theme) -> bool {
    let t = tr();
    let rect = ui.max_rect().expand2(vec2(22.0, 0.0));
    ui.painter().hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, line_color(theme)),
    );
    let mut reset = false;
    ui.horizontal_centered(|ui| {
        let page = if state.query.is_empty() {
            state.section.title()
        } else {
            t.search_title
        };
        ui.label(RichText::new(t.settings).color(theme.muted));
        ui.label(RichText::new("/").color(mix(theme.muted, theme.background, 0.4)));
        ui.label(RichText::new(page).color(theme.text));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let now = ui.input(|i| i.time);
            let armed = state.confirm_reset_at.is_some_and(|at| now - at < 3.0);
            let (label, color) = if armed {
                (t.confirm_restore, theme.focus)
            } else {
                (t.reset_defaults, theme.muted)
            };
            let button = egui::Button::new(RichText::new(label).color(color))
                .frame(false)
                .min_size(vec2(0.0, 30.0));
            let response = ui.add(button).on_hover_cursor(CursorIcon::PointingHand);
            let (icon, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
            reset_icon(ui.painter(), icon.center(), color);
            if response.clicked() {
                if armed {
                    state.confirm_reset_at = None;
                    reset = true;
                } else {
                    state.confirm_reset_at = Some(now);
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_secs(3));
                }
            }
        });
    });
    reset
}

/// Config error / update notices, shown above every page.
fn banners(ui: &mut Ui, state: &State, theme: &Theme) -> Option<Action> {
    let t = tr();
    let mut action = None;
    if let Some(err) = &state.config_error {
        egui::Frame::new()
            .fill(mix(theme.surface, theme.focus, 0.12))
            .stroke(Stroke::new(1.0, mix(theme.surface, theme.focus, 0.5)))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
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
        ui.add_space(18.0);
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
        ui.add_space(18.0);
    }
    action
}

// ---------------------------------------------------------------- form

/// Draws groups and rows, and filters them while searching.
struct Form<'a> {
    theme: &'a Theme,
    /// Folded search text; empty = not searching.
    query: String,
    /// Page being drawn (shown before group titles in search results).
    page: &'static str,
    /// First pass of a group while searching: count matches, draw nothing.
    counting: bool,
    count: usize,
    /// The group (or its page) matched the query: show all its rows.
    all: bool,
    /// No row drawn yet in the current card (no divider above the next one).
    first: bool,
    /// Groups drawn so far.
    groups: usize,
}

impl<'a> Form<'a> {
    fn new(theme: &'a Theme, query: &str) -> Self {
        Self {
            theme,
            query: fold(query.trim()),
            page: "",
            counting: false,
            count: 0,
            all: false,
            first: true,
            groups: 0,
        }
    }

    fn searching(&self) -> bool {
        !self.query.is_empty()
    }

    /// Every word of the query starts some word of `texts` ("not sis" finds
    /// "Notificação do sistema", "tema" doesn't).
    fn matches(&self, texts: &[&str]) -> bool {
        if !self.searching() || self.all {
            return true;
        }
        let words: Vec<String> = texts.iter().flat_map(|t| words(&fold(t))).collect();
        words_of(&self.query).all(|q| words.iter().any(|w| w.starts_with(q)))
    }

    /// A titled card of rows. `add` runs twice while searching (count, then draw).
    fn group(&mut self, ui: &mut Ui, title: &str, mut add: impl FnMut(&mut Ui, &mut Form)) {
        if self.searching() {
            self.all = false;
            self.all = self.matches(&[title, self.page]);
            if !self.all {
                // Counting draws nothing, but scopes still take space: use a
                // detached, invisible child so the page layout isn't touched.
                let mut scratch = ui.new_child(UiBuilder::new().invisible());
                self.counting = true;
                self.count = 0;
                add(&mut scratch, self);
                self.counting = false;
                if self.count == 0 {
                    return;
                }
            }
        }
        if self.groups > 0 {
            ui.add_space(22.0);
        }
        self.groups += 1;
        let heading = if self.searching() && self.page != title {
            format!("{} · {title}", self.page)
        } else {
            title.to_owned()
        };
        ui.label(
            RichText::new(heading)
                .size(16.5)
                .strong()
                .color(self.theme.text),
        );
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(self.theme.surface)
            .stroke(Stroke::new(1.0, line_color(self.theme)))
            .corner_radius(CornerRadius::same(12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 0.0;
                self.first = true;
                add(ui, self);
            });
        self.all = false;
    }

    /// Whether to draw an item with these texts (and counts it while counting).
    fn take(&mut self, texts: &[&str]) -> bool {
        if !self.matches(texts) {
            return false;
        }
        if self.counting {
            self.count += 1;
            return false;
        }
        true
    }

    fn divider(&mut self, ui: &mut Ui) {
        if !self.first {
            let x = ui.max_rect().x_range();
            ui.painter().hline(
                x,
                ui.cursor().top(),
                Stroke::new(1.0, line_color(self.theme)),
            );
        }
        self.first = false;
    }

    /// Title and explanation on the left, `control` on the right (`width` wide).
    fn row(
        &mut self,
        ui: &mut Ui,
        title: &str,
        hint: &str,
        width: f32,
        control: impl FnOnce(&mut Ui),
    ) {
        if !self.take(&[title, hint]) {
            return;
        }
        self.divider(ui);
        let theme = self.theme;
        egui::Frame::new()
            .inner_margin(Margin::symmetric(18, 12))
            .show(ui, |ui| {
                let full = ui.available_width();
                let top_left = ui.cursor().min;
                let text_width = (full - width - 24.0).max(140.0);
                // One line: centered on the control's height. With a hint: from the top.
                let (height, layout) = if hint.is_empty() {
                    (30.0, Layout::left_to_right(Align::Center))
                } else {
                    (1000.0, Layout::top_down(Align::Min))
                };
                let text = ui
                    .scope_builder(
                        UiBuilder::new()
                            .max_rect(Rect::from_min_size(top_left, vec2(text_width, height)))
                            .layout(layout),
                        |ui| {
                            ui.spacing_mut().item_spacing.y = 3.0;
                            ui.add(egui::Label::new(RichText::new(title).color(theme.text)).wrap());
                            if !hint.is_empty() {
                                self::hint(ui, hint, theme);
                            }
                        },
                    )
                    .response
                    .rect;
                let height = text.height().max(30.0);
                let area = Rect::from_min_max(
                    pos2(top_left.x + full - width, top_left.y),
                    pos2(top_left.x + full, top_left.y + height),
                );
                let used = ui
                    .scope_builder(
                        UiBuilder::new()
                            .max_rect(area)
                            .layout(Layout::right_to_left(Align::Center)),
                        control,
                    )
                    .response
                    .rect;
                let bottom = text.bottom().max(used.bottom()).max(area.bottom());
                ui.advance_cursor_after_rect(Rect::from_min_max(
                    top_left,
                    pos2(top_left.x + full, bottom),
                ));
            });
    }

    /// Free-form content (pickers, charts), found by `keywords` when searching.
    fn block(&mut self, ui: &mut Ui, keywords: &str, add: impl FnOnce(&mut Ui)) {
        if !self.take(&[keywords]) {
            return;
        }
        self.divider(ui);
        egui::Frame::new()
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                add(ui);
            });
    }

    fn toggle(&mut self, ui: &mut Ui, title: &str, hint: &str, value: &mut bool) {
        let accent = self.theme.focus;
        self.row(ui, title, hint, 50.0, |ui| {
            toggle(ui, value, accent);
        });
    }

    fn choice<T: PartialEq + Copy>(
        &mut self,
        ui: &mut Ui,
        title: &str,
        hint: &str,
        value: &mut T,
        options: &[(T, &str)],
    ) {
        self.row(ui, title, hint, 220.0, |ui| {
            combo(ui, title, value, options, 200.0);
        });
    }

    fn slider(
        &mut self,
        ui: &mut Ui,
        title: &str,
        hint: &str,
        slider: impl FnOnce(&mut Ui) -> egui::Response,
    ) {
        self.row(ui, title, hint, 240.0, |ui| {
            slider(ui);
        });
    }

    fn minutes(&mut self, ui: &mut Ui, title: &str, value: &mut u32) {
        self.row(ui, title, "", 120.0, |ui| {
            ui.add(DragValue::new(value).range(1..=180).suffix(" min"));
        });
    }
}

fn words(text: &str) -> Vec<String> {
    words_of(text).map(str::to_owned).collect()
}

fn words_of(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
}

/// Lowercase without accents, so "aparencia" finds "Aparência".
fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            c => c,
        })
        .collect()
}

/// A muted explanation under a setting.
fn hint(ui: &mut Ui, text: &str, theme: &Theme) {
    ui.add(egui::Label::new(RichText::new(text).size(13.0).color(theme.muted)).wrap());
}

/// An on/off switch.
fn toggle(ui: &mut Ui, on: &mut bool, accent: Color32) -> egui::Response {
    let (rect, mut response) = ui.allocate_exact_size(vec2(38.0, 22.0), Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let t = ui.ctx().animate_bool(response.id, *on);
    let visuals = ui.style().interact(&response);
    let off = ui.visuals().widgets.inactive.bg_fill;
    let enabled = ui.is_enabled();
    let fill = mix(off, accent, t);
    let painter = ui.painter();
    painter.rect(
        rect,
        CornerRadius::same(11),
        if enabled {
            fill
        } else {
            fill.gamma_multiply(0.5)
        },
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );
    let x = egui::lerp(rect.left() + 11.0..=rect.right() - 11.0, t);
    painter.circle(
        pos2(x, rect.center().y),
        8.0,
        Color32::WHITE,
        Stroke::new(1.0, Color32::from_black_alpha(30)),
    );
    response.on_hover_cursor(CursorIcon::PointingHand)
}

// ---------------------------------------------------------------- pages

fn timer_page(ui: &mut Ui, f: &mut Form, config: &mut Config) -> Option<Action> {
    let t = tr();
    let tm = &mut config.timer;
    f.group(ui, t.grp_durations, |ui, f| {
        f.minutes(ui, t.focus, &mut tm.focus_minutes);
        f.minutes(ui, t.short_break, &mut tm.short_break_minutes);
        f.minutes(ui, t.long_break, &mut tm.long_break_minutes);
        f.row(
            ui,
            t.sessions_before_long_break,
            t.sessions_hint,
            120.0,
            |ui| {
                ui.add(DragValue::new(&mut tm.sessions_before_long_break).range(1..=12));
            },
        );
    });
    f.group(ui, t.grp_auto_start, |ui, f| {
        f.toggle(
            ui,
            t.auto_start_breaks,
            t.auto_start_breaks_hint,
            &mut tm.auto_start_breaks,
        );
        f.toggle(
            ui,
            t.auto_start_focus,
            t.auto_start_focus_hint,
            &mut tm.auto_start_focus,
        );
    });
    None
}

fn sound_page(ui: &mut Ui, f: &mut Form, config: &mut Config) -> Option<Action> {
    let t = tr();
    let mut action = None;
    let s = &mut config.sound;
    let kinds = [
        (SoundKind::Chime, t.sound_chime),
        (SoundKind::Bell, t.sound_bell),
        (SoundKind::Soft, t.sound_soft),
        (SoundKind::File, t.sound_file),
    ];
    f.group(ui, t.grp_sounds, |ui, f| {
        f.toggle(ui, t.sound_enabled, "", &mut s.enabled);
        ui.add_enabled_ui(s.enabled, |ui| {
            f.row(ui, t.sound_break_starts, "", 300.0, |ui| {
                if ui.button(t.play_test).clicked() {
                    action = Some(Action::PlaySound { focus: false });
                }
                combo(ui, "break_sound", &mut s.kind, &kinds, 170.0);
            });
            if s.kind == SoundKind::File {
                f.row(ui, t.sound_file, t.sound_file_hint, 300.0, |ui| {
                    file_field(ui, &mut s.file, "");
                });
            }
            let focus_kinds: Vec<(Option<SoundKind>, &str)> = std::iter::once((None, t.sound_same))
                .chain(kinds.iter().map(|(k, l)| (Some(*k), *l)))
                .collect();
            f.row(ui, t.sound_focus_starts, "", 300.0, |ui| {
                if ui.button(t.play_test).clicked() {
                    action = Some(Action::PlaySound { focus: true });
                }
                combo(ui, "focus_sound", &mut s.focus_kind, &focus_kinds, 170.0);
            });
            if s.focus_kind == Some(SoundKind::File) {
                f.row(ui, t.sound_file, t.sound_focus_file_hint, 300.0, |ui| {
                    file_field(ui, &mut s.focus_file, "");
                });
            }
            f.slider(ui, t.volume, "", |ui| {
                ui.add(Slider::new(&mut s.volume, 0.0..=1.0).show_value(false))
            });
        });
    });
    f.group(ui, t.grp_ticking, |ui, f| {
        ui.add_enabled_ui(s.enabled, |ui| {
            f.toggle(ui, t.ticking, t.ticking_hint, &mut s.ticking);
            ui.add_enabled_ui(s.ticking, |ui| {
                f.slider(ui, t.volume, "", |ui| {
                    ui.add(Slider::new(&mut s.ticking_volume, 0.0..=1.0).show_value(false))
                });
            });
        });
    });
    action
}

/// A dropdown with a small chevron.
fn combo<T: PartialEq + Copy>(
    ui: &mut Ui,
    id: &str,
    value: &mut T,
    options: &[(T, &str)],
    width: f32,
) {
    let selected = options
        .iter()
        .find(|(v, _)| v == value)
        .map_or("", |(_, label)| *label);
    egui::ComboBox::from_id_salt(id)
        .selected_text(selected)
        .width(width)
        .icon(|ui, rect, visuals, open| {
            let c = rect.center();
            let (w, h) = (3.5, if open { -2.0 } else { 2.0 });
            let points = vec![c + vec2(-w, -h), c + vec2(0.0, h), c + vec2(w, -h)];
            ui.painter().add(PathShape::line(
                points,
                Stroke::new(1.5, visuals.fg_stroke.color),
            ));
        })
        .show_ui(ui, |ui| {
            for (v, label) in options {
                ui.selectable_value(value, *v, *label);
            }
        });
}

fn file_field(ui: &mut Ui, path: &mut String, hint: &str) {
    ui.add(
        egui::TextEdit::singleline(path)
            .hint_text(hint)
            .desired_width(ui.available_width()),
    );
}

fn mini_page(ui: &mut Ui, f: &mut Form, config: &mut Config) -> Option<Action> {
    let t = tr();
    let mut action = None;
    let shortcut = fill(t.mini_shortcut, &[("key", &config.keys.mini)]);
    let m = &mut config.mini;
    f.group(ui, t.sec_mini, |ui, f| {
        f.toggle(ui, t.mini_enabled, t.mini_help, &mut m.enabled);
        ui.add_enabled_ui(m.enabled, |ui| {
            f.toggle(
                ui,
                t.mini_show_character,
                t.mini_show_character_hint,
                &mut m.show_character,
            );
            f.row(ui, t.mini_to_corner, &shortcut, 220.0, |ui| {
                if ui.button(t.mini_to_corner_button).clicked() {
                    action = Some(Action::MiniToCorner);
                }
            });
        });
    });
    action
}

fn appearance_page(ui: &mut Ui, f: &mut Form, config: &mut Config, cx: &mut Ctx) -> Option<Action> {
    let t = tr();
    let a = &mut config.appearance;
    f.group(ui, t.grp_general, |ui, f| {
        f.choice(
            ui,
            t.language,
            "",
            &mut a.language,
            &[
                (Language::Auto, t.language_auto),
                (Language::English, "English"),
                (Language::Portuguese, "Português"),
            ],
        );
        f.choice(
            ui,
            t.style,
            t.style_hint,
            &mut a.layout,
            &[
                (config::Layout::Cozy, t.style_cozy),
                (config::Layout::Classic, t.style_classic),
            ],
        );
        f.slider(ui, t.ui_scale, t.ui_scale_hint, |ui| {
            ui.add(
                Slider::new(&mut a.ui_scale, 0.75..=1.75)
                    .step_by(0.05)
                    .fixed_decimals(2),
            )
        });
    });
    if a.layout == config::Layout::Cozy || f.searching() {
        f.group(ui, t.character, |ui, f| {
            let thumbnails = &mut cx.state.thumbnails;
            let characters = cx.characters;
            let theme = f.theme;
            f.block(ui, t.character, |ui| {
                character_picker(ui, &mut a.character, characters, thumbnails, theme);
            });
        });
    }
    if a.layout == config::Layout::Classic || f.searching() {
        f.group(ui, t.grp_ring, |ui, f| {
            f.slider(ui, t.ring_thickness, "", |ui| {
                ui.add(Slider::new(&mut a.ring_thickness, 2.0..=28.0).show_value(false))
            });
            f.toggle(ui, t.ring_glow, "", &mut a.glow);
        });
    }
    f.group(ui, t.grp_details, |ui, f| {
        f.toggle(
            ui,
            t.show_sessions,
            t.sessions_hint,
            &mut a.show_session_dots,
        );
        f.toggle(
            ui,
            t.smooth_animation,
            t.smooth_animation_hint,
            &mut a.smooth_animation,
        );
    });
    None
}

fn theme_page(ui: &mut Ui, f: &mut Form, config: &mut Config, cx: &mut Ctx) -> Option<Action> {
    let t = tr();
    let mut action = None;
    let themes = cx.themes;
    f.group(ui, t.sec_theme, |ui, f| {
        f.block(ui, t.sec_theme, |ui| theme_picker(ui, config, themes));
    });
    if config.appearance.theme == theme::CUSTOM {
        let export_name = &mut cx.state.export_name;
        let custom = &mut config.custom_theme;
        f.group(ui, t.grp_custom_colors, |ui, f| {
            let s = tr();
            let fields: [(&str, &mut Color32); 7] = [
                (s.color_background, &mut custom.background),
                (s.color_surface, &mut custom.surface),
                (s.color_text, &mut custom.text),
                (s.color_muted, &mut custom.muted),
                (s.focus, &mut custom.focus),
                (s.short_break, &mut custom.short_break),
                (s.long_break, &mut custom.long_break),
            ];
            for (label, color) in fields {
                f.row(ui, label, "", 60.0, |ui| {
                    color_edit_button_srgba(ui, color, Alpha::Opaque);
                });
            }
            f.row(ui, t.save_as_file, t.save_as_file_hint, 320.0, |ui| {
                let save = ui.button(t.save_as_file);
                ui.add(
                    egui::TextEdit::singleline(export_name)
                        .hint_text(t.theme_name_hint)
                        .desired_width(ui.available_width()),
                );
                if save.clicked() && !export_name.trim().is_empty() {
                    action = Some(Action::ExportCustom);
                }
            });
        });
    }
    let (notice, errors) = (
        cx.state.notice.clone(),
        themes
            .errors
            .iter()
            .chain(&cx.characters.errors)
            .cloned()
            .collect::<Vec<_>>(),
    );
    f.group(ui, t.grp_your_files, |ui, f| {
        let mut where_ = Vec::new();
        if let Some(dir) = Themes::dir() {
            where_.push(fill(t.your_themes, &[("path", &dir.display().to_string())]));
        }
        if let Some(dir) = Characters::dir() {
            where_.push(fill(
                t.your_characters,
                &[("path", &dir.display().to_string())],
            ));
        }
        let hint_text = format!("{}\n{}", t.reload_files_hint, where_.join("\n"));
        f.row(ui, t.reload_files, &hint_text, 130.0, |ui| {
            if ui.button(t.reload_button).clicked() {
                action = Some(Action::ReloadThemes);
            }
        });
        let theme = f.theme;
        if !errors.is_empty() || notice.is_some() {
            f.block(ui, t.reload_files, |ui| {
                for err in &errors {
                    ui.label(RichText::new(fill(t.error_in, &[("error", err)])).color(theme.focus));
                }
                if let Some(notice) = &notice {
                    ui.label(RichText::new(notice).color(theme.short_break));
                }
            });
        }
    });
    action
}

fn behavior_page(ui: &mut Ui, f: &mut Form, config: &mut Config) -> Option<Action> {
    let t = tr();
    let b = &mut config.behavior;
    f.group(ui, t.grp_phase_end, |ui, f| {
        f.toggle(ui, t.system_notification, "", &mut b.system_notification);
        f.toggle(ui, t.flash_taskbar, "", &mut b.notify_on_finish);
        f.toggle(ui, t.focus_on_finish, "", &mut b.focus_on_finish);
    });
    f.group(ui, t.grp_window, |ui, f| {
        f.toggle(ui, t.always_on_top, "", &mut b.always_on_top);
        f.toggle(
            ui,
            t.time_in_title,
            t.time_in_title_hint,
            &mut b.time_in_title,
        );
    });
    f.group(ui, t.grp_updates, |ui, f| {
        f.toggle(
            ui,
            t.check_updates,
            t.check_updates_hint,
            &mut b.check_updates,
        );
    });
    None
}

fn tray_page(ui: &mut Ui, f: &mut Form, config: &mut Config, cx: &mut Ctx) -> Option<Action> {
    let t = tr();
    let b = &mut config.behavior;
    f.group(ui, t.grp_tray, |ui, f| {
        f.toggle(ui, t.tray_icon, t.tray_icon_hint, &mut b.tray);
        ui.add_enabled_ui(b.tray, |ui| {
            f.toggle(
                ui,
                t.close_to_tray,
                t.close_to_tray_hint,
                &mut b.close_to_tray,
            );
            f.toggle(ui, t.start_hidden, "", &mut b.start_hidden);
        });
    });
    let error = cx.state.system_error.clone();
    f.group(ui, t.grp_startup, |ui, f| {
        let hint = match &error {
            Some(err) => format!("{}\n{err}", t.start_with_system_hint),
            None => t.start_with_system_hint.to_owned(),
        };
        f.toggle(ui, t.start_with_system, &hint, &mut b.start_with_system);
    });
    None
}

fn stats_page(ui: &mut Ui, f: &mut Form, cx: &mut Ctx) -> Option<Action> {
    let t = tr();
    let totals = cx.totals;
    f.group(ui, t.stats_chart, |ui, f| {
        let theme = f.theme;
        f.block(ui, t.stats_chart, |ui| week_chart(ui, totals, theme));
    });
    f.group(ui, t.grp_totals, |ui, f| {
        let theme = f.theme;
        for (label, stats) in [
            (t.stats_today, totals.today),
            (t.stats_week, totals.week),
            (t.stats_total, totals.all_time),
        ] {
            f.row(ui, label, "", 260.0, |ui| {
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

fn config_page(ui: &mut Ui, f: &mut Form) -> Option<Action> {
    let t = tr();
    let mut action = None;
    let path = Config::path()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    f.group(ui, "config.toml", |ui, f| {
        let hint = format!(
            "{}
{path}",
            t.config_file_help
        );
        f.row(ui, t.config_file_title, &hint, 300.0, |ui| {
            if ui.button(t.open_folder).clicked() {
                action = Some(Action::OpenConfigFolder);
            }
            if ui.button(t.open_config).clicked() {
                action = Some(Action::OpenConfigFile);
            }
        });
    });
    action
}

// ---------------------------------------------------------------- pickers and charts

/// Focus minutes of the last 7 days as bars, today on the right.
fn week_chart(ui: &mut Ui, totals: &Totals, theme: &Theme) {
    let t = tr();
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 160.0), Sense::hover());
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
                mix(theme.surface, theme.background, 0.4),
                border,
                egui::StrokeKind::Inside,
            );
            let art_rect = Rect::from_min_size(rect.min + vec2(6.0, 6.0), art);
            thumbnails
                .entry(c.id.clone())
                .or_default()
                .image(ui.ctx(), c, Pose::Idle, theme.focus, theme.muted)
                .paint_at(ui, art_rect);
            let name_color = if selected { theme.text } else { theme.muted };
            ui.painter().text(
                pos2(rect.center().x, rect.bottom() - 15.0),
                Align2::CENTER_CENTER,
                c.display_name(),
                FontId::proportional(14.5),
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
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(14.5),
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

// ---------------------------------------------------------------- icons

/// Points on a circle arc (angles in radians, 0 = right, clockwise on screen).
fn arc(c: Pos2, r: f32, from: f32, to: f32) -> Vec<Pos2> {
    let steps = 16;
    (0..=steps)
        .map(|i| {
            let a = from + (to - from) * i as f32 / steps as f32;
            c + r * vec2(a.cos(), a.sin())
        })
        .collect()
}

fn line(p: &Painter, points: Vec<Pos2>, color: Color32) {
    p.add(PathShape::line(points, Stroke::new(1.5, color)));
}

fn search_icon(p: &Painter, c: Pos2, color: Color32) {
    p.circle_stroke(c - vec2(1.5, 1.5), 5.0, Stroke::new(1.5, color));
    line(p, vec![c + vec2(2.2, 2.2), c + vec2(5.5, 5.5)], color);
}

fn reset_icon(p: &Painter, c: Pos2, color: Color32) {
    use std::f32::consts::PI;
    line(p, arc(c, 5.5, -PI * 0.5, PI * 1.25), color);
    let tip = c + vec2(0.0, -5.5);
    line(
        p,
        vec![tip + vec2(-3.0, -2.5), tip, tip + vec2(-2.5, 3.0)],
        color,
    );
}

/// Small line icons for the sidebar, drawn in a 16×16 box around `c`.
fn nav_icon(p: &Painter, section: Section, c: Pos2, color: Color32) {
    use std::f32::consts::PI;
    let stroke = Stroke::new(1.5, color);
    let r =
        |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(c + vec2(x0, y0), c + vec2(x1, y1));
    match section {
        Section::Timer => {
            p.circle_stroke(c + vec2(0.0, 1.0), 6.5, stroke);
            line(p, vec![c + vec2(0.0, 1.0), c + vec2(0.0, -2.5)], color);
            line(p, vec![c + vec2(0.0, 1.0), c + vec2(2.8, 2.5)], color);
            line(p, vec![c + vec2(-2.0, -7.5), c + vec2(2.0, -7.5)], color);
        }
        Section::Sound => {
            let body = vec![
                c + vec2(-7.0, -2.5),
                c + vec2(-4.0, -2.5),
                c + vec2(0.0, -6.0),
                c + vec2(0.0, 6.0),
                c + vec2(-4.0, 2.5),
                c + vec2(-7.0, 2.5),
            ];
            p.add(PathShape::closed_line(body, stroke));
            line(p, arc(c + vec2(1.0, 0.0), 3.5, -PI / 3.5, PI / 3.5), color);
            line(p, arc(c + vec2(1.0, 0.0), 6.5, -PI / 3.5, PI / 3.5), color);
        }
        Section::Mini => {
            p.rect_stroke(
                r(-7.0, -5.5, 7.0, 5.5),
                2.0,
                stroke,
                egui::StrokeKind::Middle,
            );
            p.rect_filled(r(0.5, 0.5, 5.0, 3.5), 1.0, color);
        }
        Section::Appearance => {
            for (y, knob) in [(-4.5, 2.5), (0.0, -2.5), (4.5, 1.0)] {
                line(p, vec![c + vec2(-7.0, y), c + vec2(7.0, y)], color);
                p.circle_filled(c + vec2(knob, y), 2.2, color);
            }
        }
        Section::Theme => {
            p.circle_stroke(c, 6.5, stroke);
            let mut half = arc(c, 6.5, PI * 0.5, PI * 1.5);
            half.push(c);
            p.add(PathShape::convex_polygon(half, color, Stroke::NONE));
        }
        Section::Behavior => {
            let mut bell = arc(c + vec2(0.0, -1.0), 4.5, PI, PI * 2.0);
            bell.insert(0, c + vec2(-6.5, 4.0));
            bell.insert(1, c + vec2(-4.5, 2.0));
            bell.push(c + vec2(4.5, 2.0));
            bell.push(c + vec2(6.5, 4.0));
            bell.push(c + vec2(-6.5, 4.0));
            line(p, bell, color);
            p.circle_filled(c + vec2(0.0, 6.2), 1.4, color);
        }
        Section::Tray => {
            line(
                p,
                vec![
                    c + vec2(-7.0, 1.0),
                    c + vec2(-3.0, 1.0),
                    c + vec2(-2.0, 3.0),
                    c + vec2(2.0, 3.0),
                    c + vec2(3.0, 1.0),
                    c + vec2(7.0, 1.0),
                ],
                color,
            );
            p.rect_stroke(
                r(-7.0, -6.0, 7.0, 6.0),
                2.0,
                stroke,
                egui::StrokeKind::Middle,
            );
        }
        Section::Stats => {
            for (x, top) in [(-5.0, 0.0), (0.0, -6.0), (5.0, -3.0)] {
                p.rect_filled(r(x - 1.6, top, x + 1.6, 6.5), 1.0, color);
            }
        }
        Section::ConfigFile => {
            line(
                p,
                vec![
                    c + vec2(-5.5, -7.0),
                    c + vec2(2.0, -7.0),
                    c + vec2(5.5, -3.5),
                    c + vec2(5.5, 7.0),
                    c + vec2(-5.5, 7.0),
                    c + vec2(-5.5, -7.0),
                ],
                color,
            );
            for y in [0.0, 3.5] {
                line(p, vec![c + vec2(-3.0, y), c + vec2(3.0, y)], color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_matches_word_starts_without_accents() {
        let theme = Theme::default();
        let f = Form::new(&theme, "  NOTIF sis ");
        assert!(f.matches(&["Notificação do sistema"]));
        assert!(!Form::new(&theme, "tema").matches(&["Notificação do sistema"]));
        assert!(Form::new(&theme, "aparencia").matches(&["Aparência"]));
        assert!(Form::new(&theme, "").matches(&["anything"]));
    }
}
