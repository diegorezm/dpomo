//! UI translations.
//!
//! Every user-facing string lives in [`Strings`], with one `static` per
//! language. Adding a field forces every language to translate it (it won't
//! compile otherwise). Placeholders like `{n}` are filled with
//! [`fill`]/`str::replace` at the call site.
//!
//! The active language is global (set by `App` from the config), so views and
//! background threads just call [`tr()`].

use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

use crate::timer::Phase;

/// `appearance.language` in the config.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    /// Follow the operating system (Portuguese if it's `pt*`, else English).
    #[default]
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "pt-BR")]
    Portuguese,
}

impl Language {
    /// The concrete language `Auto` stands for on this machine.
    pub fn resolve(self) -> Language {
        match self {
            Language::Auto => {
                let os = sys_locale::get_locale().unwrap_or_default().to_lowercase();
                if os.starts_with("pt") {
                    Language::Portuguese
                } else {
                    Language::English
                }
            }
            other => other,
        }
    }

    /// BCP-47-ish code, used for per-language names in `character.toml`.
    pub fn code(self) -> &'static str {
        match self.resolve() {
            Language::Portuguese => "pt-BR",
            _ => "en",
        }
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

/// Make `lang` the active language.
pub fn set(lang: Language) {
    let id = match lang.resolve() {
        Language::Portuguese => 1,
        _ => 0,
    };
    CURRENT.store(id, Ordering::Relaxed);
}

pub fn current() -> Language {
    match CURRENT.load(Ordering::Relaxed) {
        1 => Language::Portuguese,
        _ => Language::English,
    }
}

/// Strings for the active language.
pub fn tr() -> &'static Strings {
    match current() {
        Language::Portuguese => &PT_BR,
        _ => &EN,
    }
}

/// Replace `{key}` placeholders: `fill(t.saved_to, &[("path", &p)])`.
pub fn fill(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = template.to_owned();
    for (key, value) in args {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

pub struct Strings {
    // Phases
    pub focus: &'static str,
    pub short_break: &'static str,
    pub long_break: &'static str,
    /// Compact labels for the phase tabs.
    pub tab_focus: &'static str,
    pub tab_short_break: &'static str,
    pub tab_long_break: &'static str,
    pub paused: &'static str,

    // Main screen tooltips ({key} = configured shortcut)
    pub hint_settings: &'static str,
    pub hint_reset: &'static str,
    pub hint_toggle: &'static str,
    pub hint_skip: &'static str,
    pub hint_mini: &'static str,
    pub config_error_badge: &'static str,
    /// {version}
    pub update_badge: &'static str,

    // Settings
    pub settings: &'static str,
    pub config_error_title: &'static str,
    pub config_error_help: &'static str,
    pub open_config: &'static str,
    pub overwrite_config: &'static str,

    pub sec_timer: &'static str,
    pub sessions_before_long_break: &'static str,
    pub auto_start_breaks: &'static str,
    pub auto_start_focus: &'static str,

    pub sec_appearance: &'static str,
    pub language: &'static str,
    pub language_auto: &'static str,
    pub style: &'static str,
    pub style_cozy: &'static str,
    pub style_classic: &'static str,
    pub style_hint: &'static str,
    pub character: &'static str,
    pub ring_thickness: &'static str,
    pub ring_glow: &'static str,
    pub ui_scale: &'static str,
    pub show_sessions: &'static str,
    pub smooth_animation: &'static str,
    pub smooth_animation_hint: &'static str,

    pub sec_theme: &'static str,
    pub custom_theme: &'static str,
    pub default_export_name: &'static str,
    pub save_as_file: &'static str,
    pub theme_name_hint: &'static str,
    pub reload_files: &'static str,
    pub reload_files_hint: &'static str,
    /// {path}
    pub your_themes: &'static str,
    /// {path}
    pub your_characters: &'static str,
    /// {error}
    pub error_in: &'static str,
    pub color_background: &'static str,
    pub color_surface: &'static str,
    pub color_text: &'static str,
    pub color_muted: &'static str,

    pub sec_sound: &'static str,
    pub sound_enabled: &'static str,
    pub sound_chime: &'static str,
    pub sound_bell: &'static str,
    pub sound_soft: &'static str,
    pub sound_file: &'static str,
    pub sound_file_hint: &'static str,
    pub volume: &'static str,
    pub play_test: &'static str,

    pub sec_stats: &'static str,
    pub stats_today: &'static str,
    pub stats_week: &'static str,
    pub stats_total: &'static str,
    pub session_one: &'static str,
    /// {n}
    pub session_many: &'static str,

    pub sec_mini: &'static str,
    pub mini_enabled: &'static str,
    pub mini_help: &'static str,
    pub mini_show_character: &'static str,
    pub mini_to_corner: &'static str,
    /// {key}
    pub mini_shortcut: &'static str,
    pub mini_open_main: &'static str,
    pub mini_close: &'static str,

    pub sec_behavior: &'static str,
    pub always_on_top: &'static str,
    pub flash_taskbar: &'static str,
    pub system_notification: &'static str,
    pub focus_on_finish: &'static str,
    pub time_in_title: &'static str,
    pub check_updates: &'static str,

    pub sec_tray: &'static str,
    pub tray_icon: &'static str,
    pub close_to_tray: &'static str,
    pub close_to_tray_hint: &'static str,
    pub start_hidden: &'static str,
    pub start_with_system: &'static str,
    pub start_with_system_hint: &'static str,

    pub sec_config_file: &'static str,
    pub config_file_help: &'static str,
    pub open_folder: &'static str,
    pub reset_defaults: &'static str,

    // Notices / errors ({e}, {path}, {themes}, {characters})
    pub open_failed: &'static str,
    pub reloaded: &'static str,
    pub theme_saved: &'static str,
    pub save_failed: &'static str,
    pub autostart_failed: &'static str,
    pub tray_unavailable: &'static str,
    pub builtin: &'static str,
    /// {file}
    pub not_utf8: &'static str,
    pub missing_idle: &'static str,
    /// {n}, {msg}
    pub config_line: &'static str,
    /// {value}
    pub invalid_color: &'static str,

    // Tray menu
    pub tray_show: &'static str,
    pub tray_hide: &'static str,
    pub tray_start: &'static str,
    pub tray_pause: &'static str,
    pub tray_skip: &'static str,
    pub tray_reset: &'static str,
    pub tray_mini: &'static str,
    pub tray_settings: &'static str,
    pub tray_quit: &'static str,

    // Notifications ({min})
    pub notify_focus_done: &'static str,
    pub notify_break_done: &'static str,
    pub notify_next_focus: &'static str,
    pub notify_next_short: &'static str,
    pub notify_next_long: &'static str,
    pub notify_started: &'static str,
}

impl Strings {
    pub fn phase(&self, phase: Phase) -> &'static str {
        match phase {
            Phase::Focus => self.focus,
            Phase::ShortBreak => self.short_break,
            Phase::LongBreak => self.long_break,
        }
    }

    pub fn phase_tab(&self, phase: Phase) -> &'static str {
        match phase {
            Phase::Focus => self.tab_focus,
            Phase::ShortBreak => self.tab_short_break,
            Phase::LongBreak => self.tab_long_break,
        }
    }

    pub fn sessions(&self, n: u32) -> String {
        if n == 1 {
            self.session_one.to_owned()
        } else {
            fill(self.session_many, &[("n", &n.to_string())])
        }
    }
}

pub static EN: Strings = Strings {
    focus: "Focus",
    short_break: "Short break",
    long_break: "Long break",
    tab_focus: "Focus",
    tab_short_break: "Break",
    tab_long_break: "Long",
    paused: "(paused)",

    hint_settings: "Settings ({key})",
    hint_reset: "Restart ({key})",
    hint_toggle: "Start / pause ({key})",
    hint_skip: "Skip ({key})",
    hint_mini: "Floating timer ({key})",
    config_error_badge: "config.toml has an error (see settings)",
    update_badge: "Version {version} is available — click to download",

    settings: "Settings",
    config_error_title: "config.toml has an error",
    config_error_help: "Fix the file and save it: the app picks it up right away. Until then, nothing here is saved.",
    open_config: "Open config.toml",
    overwrite_config: "Replace it with the current settings",

    sec_timer: "Timer",
    sessions_before_long_break: "Sessions before a long break",
    auto_start_breaks: "Start breaks automatically",
    auto_start_focus: "Start focus automatically",

    sec_appearance: "Appearance",
    language: "Language",
    language_auto: "System",
    style: "Style",
    style_cozy: "Cozy",
    style_classic: "Classic",
    style_hint: "Cozy shows your character; Classic shows a progress ring.",
    character: "Character",
    ring_thickness: "Ring thickness",
    ring_glow: "Ring glow",
    ui_scale: "Interface scale",
    show_sessions: "Show sessions",
    smooth_animation: "Smooth animation",
    smooth_animation_hint: "Off: redraws once per second (uses less CPU)",

    sec_theme: "Theme",
    custom_theme: "Custom",
    default_export_name: "My theme",
    save_as_file: "Save as file",
    theme_name_hint: "theme name",
    reload_files: "Reload themes and characters",
    reload_files_hint: "Read the theme and character folders again",
    your_themes: "Your themes: {path}",
    your_characters: "Your characters: {path}",
    error_in: "Error in {error}",
    color_background: "Background",
    color_surface: "Surface",
    color_text: "Text",
    color_muted: "Secondary text",

    sec_sound: "Sound",
    sound_enabled: "Play a sound when a phase ends",
    sound_chime: "Chime",
    sound_bell: "Bell",
    sound_soft: "Soft",
    sound_file: "File",
    sound_file_hint: "path to a .wav, .mp3 or .ogg file",
    volume: "Volume",
    play_test: "Test",

    sec_stats: "Statistics",
    stats_today: "Today",
    stats_week: "Last 7 days",
    stats_total: "All time",
    session_one: "1 session",
    session_many: "{n} sessions",

    sec_mini: "Floating timer",
    mini_enabled: "Show the floating timer",
    mini_help: "A small window that stays on top of everything with the time left. Drag it to move it, drag its bottom-right corner to resize it, double-click it to open dpomo.",
    mini_show_character: "Show the character",
    mini_to_corner: "Move it back to the corner",
    mini_shortcut: "Shortcut in the main window: {key} · command line: dpomo --mini",
    mini_open_main: "Open dpomo",
    mini_close: "Hide the floating timer",

    sec_behavior: "Behavior",
    always_on_top: "Always on top",
    flash_taskbar: "Flash the taskbar when a phase ends",
    system_notification: "System notification when a phase ends",
    focus_on_finish: "Bring the window to the front when a phase ends",
    time_in_title: "Show the time in the window title",
    check_updates: "Check for new versions",

    sec_tray: "Tray & startup",
    tray_icon: "System tray icon",
    close_to_tray: "Closing the window hides it in the tray",
    close_to_tray_hint: "The timer keeps running. To quit, use \"Quit\" in the tray menu.",
    start_hidden: "Start hidden in the tray",
    start_with_system: "Start when I log in",
    start_with_system_hint: "Opens hidden in the tray when you turn on the computer",

    sec_config_file: "Configuration file",
    config_file_help: "Everything here is saved in config.toml, including shortcuts and window size. Edit it freely: the app picks up changes right away.",
    open_folder: "Open folder",
    reset_defaults: "Restore defaults",

    open_failed: "Couldn't open it: {e}",
    reloaded: "{themes} themes and {characters} characters loaded",
    theme_saved: "Theme saved to {path}",
    save_failed: "Couldn't save: {e}",
    autostart_failed: "Couldn't set up start on login: {e}",
    tray_unavailable: "Tray unavailable: {e}",
    builtin: "built-in",
    not_utf8: "{file}: not UTF-8",
    missing_idle: "idle.svg is missing",
    config_line: "line {n}: {msg}",
    invalid_color: "invalid color {value}, use \"#rrggbb\"",

    tray_show: "Show",
    tray_hide: "Hide",
    tray_start: "Start",
    tray_pause: "Pause",
    tray_skip: "Skip",
    tray_reset: "Restart",
    tray_mini: "Floating timer",
    tray_settings: "Settings…",
    tray_quit: "Quit",

    notify_focus_done: "Focus done! 🍅",
    notify_break_done: "Break's over",
    notify_next_focus: "Time to focus ({min} min)",
    notify_next_short: "Time for a short break ({min} min)",
    notify_next_long: "Time for a long break ({min} min)",
    notify_started: "already started",
};

pub static PT_BR: Strings = Strings {
    focus: "Foco",
    short_break: "Pausa curta",
    long_break: "Pausa longa",
    tab_focus: "Foco",
    tab_short_break: "Pausa",
    tab_long_break: "Longa",
    paused: "(pausado)",

    hint_settings: "Ajustes ({key})",
    hint_reset: "Reiniciar ({key})",
    hint_toggle: "Iniciar / pausar ({key})",
    hint_skip: "Pular ({key})",
    hint_mini: "Timer flutuante ({key})",
    config_error_badge: "Erro no config.toml (ver ajustes)",
    update_badge: "Versão {version} disponível — clique para baixar",

    settings: "Ajustes",
    config_error_title: "O config.toml tem um erro",
    config_error_help: "Corrija o arquivo e salve: o app aplica na hora. Até lá, nada aqui é salvo.",
    open_config: "Abrir config.toml",
    overwrite_config: "Substituir pelos ajustes atuais",

    sec_timer: "Timer",
    sessions_before_long_break: "Sessões até a pausa longa",
    auto_start_breaks: "Iniciar pausas automaticamente",
    auto_start_focus: "Iniciar foco automaticamente",

    sec_appearance: "Aparência",
    language: "Idioma",
    language_auto: "Sistema",
    style: "Estilo",
    style_cozy: "Aconchegante",
    style_classic: "Clássico",
    style_hint: "Aconchegante mostra seu personagem; Clássico mostra um anel de progresso.",
    character: "Personagem",
    ring_thickness: "Espessura do anel",
    ring_glow: "Brilho no anel",
    ui_scale: "Escala da interface",
    show_sessions: "Mostrar sessões",
    smooth_animation: "Animação suave",
    smooth_animation_hint: "Desligado: redesenha 1x por segundo (gasta menos CPU)",

    sec_theme: "Tema",
    custom_theme: "Personalizado",
    default_export_name: "Meu tema",
    save_as_file: "Salvar como arquivo",
    theme_name_hint: "nome do tema",
    reload_files: "Recarregar temas e personagens",
    reload_files_hint: "Lê de novo as pastas de temas e personagens",
    your_themes: "Seus temas: {path}",
    your_characters: "Seus personagens: {path}",
    error_in: "Erro em {error}",
    color_background: "Fundo",
    color_surface: "Superfície",
    color_text: "Texto",
    color_muted: "Texto secundário",

    sec_sound: "Som",
    sound_enabled: "Tocar um som quando a fase terminar",
    sound_chime: "Sininho",
    sound_bell: "Sino",
    sound_soft: "Suave",
    sound_file: "Arquivo",
    sound_file_hint: "caminho de um arquivo .wav, .mp3 ou .ogg",
    volume: "Volume",
    play_test: "Testar",

    sec_stats: "Estatísticas",
    stats_today: "Hoje",
    stats_week: "Últimos 7 dias",
    stats_total: "Total",
    session_one: "1 sessão",
    session_many: "{n} sessões",

    sec_mini: "Timer flutuante",
    mini_enabled: "Mostrar o timer flutuante",
    mini_help: "Uma janelinha que fica por cima de tudo com o tempo que falta. Arraste para mover, puxe o canto inferior direito para redimensionar, clique duas vezes para abrir o dpomo.",
    mini_show_character: "Mostrar o personagem",
    mini_to_corner: "Voltar para o canto",
    mini_shortcut: "Atalho na janela principal: {key} · linha de comando: dpomo --mini",
    mini_open_main: "Abrir o dpomo",
    mini_close: "Esconder o timer flutuante",

    sec_behavior: "Comportamento",
    always_on_top: "Sempre no topo",
    flash_taskbar: "Piscar na barra de tarefas ao terminar",
    system_notification: "Notificação do sistema ao terminar",
    focus_on_finish: "Trazer a janela para frente ao terminar",
    time_in_title: "Tempo no título da janela",
    check_updates: "Procurar novas versões",

    sec_tray: "Bandeja e inicialização",
    tray_icon: "Ícone na bandeja do sistema",
    close_to_tray: "Fechar a janela esconde na bandeja",
    close_to_tray_hint: "O timer continua rodando. Para sair, use \"Sair\" no menu da bandeja.",
    start_hidden: "Abrir escondido na bandeja",
    start_with_system: "Iniciar com o sistema",
    start_with_system_hint: "Abre escondido na bandeja quando você liga o computador",

    sec_config_file: "Arquivo de configuração",
    config_file_help: "Tudo aqui fica salvo no config.toml, inclusive atalhos e tamanho da janela. Edite à vontade: o app aplica na hora.",
    open_folder: "Abrir pasta",
    reset_defaults: "Restaurar padrões",

    open_failed: "Não foi possível abrir: {e}",
    reloaded: "{themes} temas e {characters} personagens carregados",
    theme_saved: "Tema salvo em {path}",
    save_failed: "Não foi possível salvar: {e}",
    autostart_failed: "Iniciar com o sistema falhou: {e}",
    tray_unavailable: "Bandeja indisponível: {e}",
    builtin: "embutido",
    not_utf8: "{file}: não é UTF-8",
    missing_idle: "falta idle.svg",
    config_line: "linha {n}: {msg}",
    invalid_color: "cor inválida {value}, use \"#rrggbb\"",

    tray_show: "Mostrar",
    tray_hide: "Esconder",
    tray_start: "Iniciar",
    tray_pause: "Pausar",
    tray_skip: "Pular",
    tray_reset: "Reiniciar",
    tray_mini: "Timer flutuante",
    tray_settings: "Ajustes…",
    tray_quit: "Sair",

    notify_focus_done: "Foco concluído! 🍅",
    notify_break_done: "Pausa terminada",
    notify_next_focus: "Hora de focar ({min} min)",
    notify_next_short: "Hora de uma pausa curta ({min} min)",
    notify_next_long: "Hora de uma pausa longa ({min} min)",
    notify_started: "já começou",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_replaces_placeholders() {
        assert_eq!(fill("{a} and {b}", &[("a", "1"), ("b", "2")]), "1 and 2");
    }

    #[test]
    fn plural_sessions() {
        assert_eq!(EN.sessions(1), "1 session");
        assert_eq!(EN.sessions(3), "3 sessions");
        assert_eq!(PT_BR.sessions(0), "0 sessões");
    }
}
