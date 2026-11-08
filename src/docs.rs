//! Generates the configuration guide on the website (`site/docs.html`) from the code.
//!
//! Defaults come from `Config::default()`; the lists of themes, characters, poses, key
//! names and command-line flags come from the code too. Only the descriptions are
//! written by hand, in the tables below (English + Portuguese).
//!
//! `cargo test docs` fails when the page is out of date or a setting has no description;
//! `POMOLOTL_BLESS=1 cargo test docs` rewrites the generated parts of the page (between
//! `<!-- gen:NAME -->` and `<!-- /gen:NAME -->`). Test-only: none of this is in the binary.

use std::fmt::Write;

use egui::{Color32, Key};

use crate::character::{self, Characters, Pose};
use crate::config::{Config, Mini, Window};
use crate::instance;
use crate::theme::{Theme, Themes};

/// `[section]`, title (en, pt), intro (en, pt). In the order of `config.toml`.
const SECTIONS: &[(&str, [&str; 2], [&str; 2])] = &[
    (
        "timer",
        ["Timer", "Timer"],
        [
            "Durations are whole minutes (at least 1). A new duration applies from the next phase that hasn't started.",
            "Tempos em minutos inteiros (no mínimo 1). Um tempo novo vale a partir da próxima fase que ainda não começou.",
        ],
    ),
    ("appearance", ["Appearance", "Aparência"], ["", ""]),
    ("behavior", ["Behavior", "Comportamento"], ["", ""]),
    ("sound", ["Sound", "Som"], ["", ""]),
    ("window", ["Main window", "Janela principal"], ["", ""]),
    (
        "mini",
        ["Floating timer", "Timer flutuante"],
        [
            "A small borderless window that stays on top of everything. Drag it to move it, drag its bottom-right corner to resize it, double-click it to open Pomolotl.",
            "Uma janelinha sem borda que fica por cima de tudo. Arraste para mover, puxe o canto inferior direito para mudar o tamanho, clique duas vezes para abrir o Pomolotl.",
        ],
    ),
    (
        "keys",
        ["Shortcuts", "Atalhos"],
        [
            "Keys that work while a Pomolotl window is focused. Values are [key names](#key-names); an unknown name falls back to the default. For shortcuts that work anywhere, see [Command line](#cli).",
            "Teclas que funcionam com uma janela do Pomolotl em foco. Os valores são [nomes de teclas](#key-names); um nome desconhecido volta ao padrão. Para atalhos que funcionam em qualquer lugar, veja [Linha de comando](#cli).",
        ],
    ),
    (
        "custom_theme",
        ["Custom theme", "Tema personalizado"],
        [
            "Your own colors, used when `theme = \"custom\"`. Same keys as a [theme file](#themes); the settings window has a color picker for them.",
            "Suas próprias cores, usadas quando `theme = \"custom\"`. As mesmas chaves de um [arquivo de tema](#themes); a janela de ajustes tem um seletor de cores para elas.",
        ],
    ),
];

/// `section.key`, description (en, pt). In the order shown on the page.
/// `{mini_w}`, `{mini_h}`, `{win_w}`, `{win_h}` are filled with the minimum sizes.
const SETTINGS: &[(&str, [&str; 2])] = &[
    (
        "timer.focus_minutes",
        [
            "Length of a focus session.",
            "Duração de uma sessão de foco.",
        ],
    ),
    (
        "timer.short_break_minutes",
        ["Length of a short break.", "Duração da pausa curta."],
    ),
    (
        "timer.long_break_minutes",
        ["Length of the long break.", "Duração da pausa longa."],
    ),
    (
        "timer.sessions_before_long_break",
        [
            "Focus sessions until a long break (the dots under the timer).",
            "Sessões de foco até a pausa longa (as bolinhas embaixo do timer).",
        ],
    ),
    (
        "timer.auto_start_breaks",
        [
            "Start breaks by themselves when a focus session ends.",
            "Começar as pausas sozinhas quando uma sessão de foco termina.",
        ],
    ),
    (
        "timer.auto_start_focus",
        [
            "Start the next focus session by itself when a break ends.",
            "Começar o próximo foco sozinho quando uma pausa termina.",
        ],
    ),
    (
        "appearance.language",
        [
            "`\"auto\"` follows the system language; or `\"en\"` (English), `\"pt-BR\"` (Portuguese).",
            "`\"auto\"` segue o idioma do sistema; ou `\"en\"` (inglês), `\"pt-BR\"` (português).",
        ],
    ),
    (
        "appearance.layout",
        [
            "`\"cozy\"`: a character and a big chunky timer. `\"classic\"`: a minimal progress ring.",
            "`\"cozy\"`: um personagem e um timer grandão. `\"classic\"`: um anel de progresso minimalista.",
        ],
    ),
    (
        "appearance.character",
        [
            "Your companion (cozy layout and floating timer): the folder name of a [character](#characters), built-in or yours.",
            "Seu companheiro (layout cozy e timer flutuante): o nome da pasta de um [personagem](#characters), embutido ou seu.",
        ],
    ),
    (
        "appearance.theme",
        [
            "The `name` of a [theme](#themes), or `\"custom\"` to use [`[custom_theme]`](#sec-custom_theme).",
            "O `name` de um [tema](#themes), ou `\"custom\"` para usar o [`[custom_theme]`](#sec-custom_theme).",
        ],
    ),
    (
        "appearance.ring_thickness",
        [
            "Width of the ring (classic layout).",
            "Espessura do anel (layout classic).",
        ],
    ),
    (
        "appearance.ui_scale",
        [
            "Size of everything in the windows: `1.0` is normal, `0.75` smaller, `1.5` bigger.",
            "Tamanho de tudo nas janelas: `1.0` é o normal, `0.75` menor, `1.5` maior.",
        ],
    ),
    (
        "appearance.smooth_animation",
        [
            "Animate at ~30 fps while the timer runs. `false` repaints once per second and uses even less CPU.",
            "Animar a ~30 fps enquanto o timer corre. `false` redesenha uma vez por segundo e gasta ainda menos CPU.",
        ],
    ),
    (
        "appearance.glow",
        [
            "Soft glow around the ring (classic layout).",
            "Brilho suave em volta do anel (layout classic).",
        ],
    ),
    (
        "appearance.show_session_dots",
        [
            "One dot per focus session until the long break.",
            "Uma bolinha por sessão de foco até a pausa longa.",
        ],
    ),
    (
        "behavior.always_on_top",
        [
            "Keep the main window above other windows.",
            "Manter a janela principal por cima das outras.",
        ],
    ),
    (
        "behavior.notify_on_finish",
        [
            "Flash the taskbar / dock icon when a phase ends.",
            "Piscar o ícone na barra de tarefas / dock quando uma fase termina.",
        ],
    ),
    (
        "behavior.focus_on_finish",
        [
            "Bring the main window to the front when a phase ends.",
            "Trazer a janela principal para a frente quando uma fase termina.",
        ],
    ),
    (
        "behavior.time_in_title",
        [
            "Show the time left in the window title (and so in the taskbar).",
            "Mostrar o tempo restante no título da janela (e na barra de tarefas).",
        ],
    ),
    (
        "behavior.tray",
        [
            "Tray icon with live progress and a menu. If the system has no tray, Pomolotl turns this off by itself.",
            "Ícone na bandeja com o progresso e um menu. Se o sistema não tiver bandeja, o Pomolotl desliga isso sozinho.",
        ],
    ),
    (
        "behavior.close_to_tray",
        [
            "Closing the window hides it in the tray and the timer keeps going. `false`: closing quits.",
            "Fechar a janela esconde na bandeja e o timer continua. `false`: fechar encerra o app.",
        ],
    ),
    (
        "behavior.system_notification",
        [
            "Show a system notification when a phase ends.",
            "Mostrar uma notificação do sistema quando uma fase termina.",
        ],
    ),
    (
        "behavior.start_with_system",
        [
            "Start Pomolotl, hidden in the tray, when you log in.",
            "Abrir o Pomolotl, escondido na bandeja, quando você entra no computador.",
        ],
    ),
    (
        "behavior.start_hidden",
        [
            "Also start hidden in the tray when you open Pomolotl yourself.",
            "Abrir escondido na bandeja também quando você abre o Pomolotl.",
        ],
    ),
    (
        "behavior.check_updates",
        [
            "Once a day, ask GitHub whether there's a newer version and show a badge if so. It never downloads anything.",
            "Uma vez por dia, perguntar ao GitHub se há versão nova e mostrar um aviso. Nunca baixa nada.",
        ],
    ),
    (
        "sound.enabled",
        [
            "Play a sound when a phase ends.",
            "Tocar um som quando uma fase termina.",
        ],
    ),
    (
        "sound.kind",
        [
            "Sound when a break starts: `\"chime\"`, `\"bell\"`, `\"soft\"`, or `\"file\"` to play your own file.",
            "Som quando a pausa começa: `\"chime\"`, `\"bell\"`, `\"soft\"`, ou `\"file\"` para tocar um arquivo seu.",
        ],
    ),
    (
        "sound.file",
        [
            "Path to a .wav, .mp3 or .ogg file, used when `kind = \"file\"`. On Windows, use single quotes: `'C:\\Users\\me\\ding.mp3'`.",
            "Caminho de um arquivo .wav, .mp3 ou .ogg, usado quando `kind = \"file\"`. No Windows, use aspas simples: `'C:\\Users\\eu\\ding.mp3'`.",
        ],
    ),
    (
        "sound.focus_kind",
        [
            "Sound when focus starts again, same values as `kind`. Missing: the same sound as `kind`.",
            "Som quando o foco recomeça, os mesmos valores de `kind`. Sem isso: o mesmo som de `kind`.",
        ],
    ),
    (
        "sound.focus_file",
        [
            "Your file for `focus_kind = \"file\"`. Missing or empty: `file`.",
            "Seu arquivo para `focus_kind = \"file\"`. Sem isso ou vazio: `file`.",
        ],
    ),
    (
        "sound.volume",
        [
            "From `0.0` (silent) to `1.0` (loudest).",
            "De `0.0` (mudo) a `1.0` (mais alto).",
        ],
    ),
    (
        "sound.ticking",
        [
            "A soft clock tick every second, only while a focus session runs.",
            "Um tique suave de relógio a cada segundo, só enquanto uma sessão de foco está rodando.",
        ],
    ),
    (
        "sound.ticking_volume",
        [
            "Volume of the ticking, `0.0` to `1.0`.",
            "Volume do tique-taque, de `0.0` a `1.0`.",
        ],
    ),
    (
        "window.width",
        [
            "Width of the main window when it opens, in points (at least {win_w}).",
            "Largura da janela principal ao abrir, em pontos (no mínimo {win_w}).",
        ],
    ),
    (
        "window.height",
        [
            "Height of the main window when it opens (at least {win_h}).",
            "Altura da janela principal ao abrir (no mínimo {win_h}).",
        ],
    ),
    (
        "window.remember_size",
        [
            "Save the window's size when it closes, so it opens the same next time.",
            "Guardar o tamanho da janela ao fechar, para abrir igual da próxima vez.",
        ],
    ),
    (
        "mini.enabled",
        [
            "Show the floating timer. Also: the button at the top of the main window, the `M` key, the tray menu or `pomolotl --mini`.",
            "Mostrar o timer flutuante. Também: o botão no topo da janela principal, a tecla `M`, o menu da bandeja ou `pomolotl --mini`.",
        ],
    ),
    (
        "mini.width",
        [
            "Width in points (at least {mini_w}). Dragging its corner changes it.",
            "Largura em pontos (no mínimo {mini_w}). Puxar o canto muda isso.",
        ],
    ),
    (
        "mini.height",
        [
            "Height (at least {mini_h}). Wide shows the character too; small shows just the time.",
            "Altura (no mínimo {mini_h}). Larga mostra o personagem também; pequena mostra só o tempo.",
        ],
    ),
    (
        "mini.x",
        [
            "Screen position, added when you move it. Delete `x` and `y` to put it back in the top-right corner.",
            "Posição na tela, adicionada quando você move. Apague `x` e `y` para voltar ao canto superior direito.",
        ],
    ),
    ("mini.y", ["See `x`.", "Veja `x`."]),
    (
        "mini.show_character",
        [
            "Show the character when there's room for it.",
            "Mostrar o personagem quando houver espaço.",
        ],
    ),
    ("keys.toggle", ["Start / pause.", "Iniciar / pausar."]),
    (
        "keys.reset",
        ["Restart the current phase.", "Reiniciar a fase atual."],
    ),
    (
        "keys.skip",
        ["Skip to the next phase.", "Pular para a próxima fase."],
    ),
    (
        "keys.settings",
        ["Open the settings window.", "Abrir a janela de ajustes."],
    ),
    ("keys.focus", ["Switch to focus.", "Ir para o foco."]),
    (
        "keys.short_break",
        ["Switch to the short break.", "Ir para a pausa curta."],
    ),
    (
        "keys.long_break",
        ["Switch to the long break.", "Ir para a pausa longa."],
    ),
    (
        "keys.mini",
        [
            "Show / hide the floating timer.",
            "Mostrar / esconder o timer flutuante.",
        ],
    ),
    (
        "keys.back",
        ["Close the settings window.", "Fechar a janela de ajustes."],
    ),
];

/// Settings that aren't written until they have a value.
const UNSET: &[&str] = &["mini.x", "mini.y", "sound.focus_kind", "sound.focus_file"];

/// Keys of a theme file (and of `[custom_theme]`).
const THEME_FIELDS: &[(&str, [&str; 2])] = &[
    (
        "name",
        [
            "Shown in the theme picker; use it in `theme = \"…\"`. Same name as a built-in theme = replaces it. Missing: the file name.",
            "Aparece no seletor de temas; use em `theme = \"…\"`. Mesmo nome de um tema embutido = substitui ele. Sem nome: o nome do arquivo.",
        ],
    ),
    (
        "background",
        [
            "Window background. It also decides whether the theme is dark or light.",
            "Fundo da janela. Também decide se o tema é escuro ou claro.",
        ],
    ),
    (
        "surface",
        [
            "Cards, buttons and the empty part of the ring.",
            "Cartões, botões e a parte vazia do anel.",
        ],
    ),
    ("text", ["Main text.", "Texto principal."]),
    (
        "muted",
        [
            "Secondary text, hints and the character's details.",
            "Texto secundário, dicas e detalhes do personagem.",
        ],
    ),
    (
        "focus",
        [
            "Focus color: progress, buttons, the character's accents.",
            "Cor do foco: progresso, botões, detalhes do personagem.",
        ],
    ),
    ("short_break", ["Short break color.", "Cor da pausa curta."]),
    ("long_break", ["Long break color.", "Cor da pausa longa."]),
];

/// Command-line flags (`instance::FLAGS`).
const FLAGS: &[(&str, [&str; 2])] = &[
    (
        "--show",
        [
            "Show the main window (same as opening Pomolotl again).",
            "Mostrar a janela principal (igual a abrir o Pomolotl de novo).",
        ],
    ),
    ("--toggle", ["Start / pause.", "Iniciar / pausar."]),
    (
        "--skip",
        ["Skip to the next phase.", "Pular para a próxima fase."],
    ),
    (
        "--reset",
        ["Restart the current phase.", "Reiniciar a fase atual."],
    ),
    (
        "--mini",
        [
            "Show / hide the floating timer.",
            "Mostrar / esconder o timer flutuante.",
        ],
    ),
    (
        "--settings",
        ["Open the settings window.", "Abrir a janela de ajustes."],
    ),
    ("--quit", ["Quit Pomolotl.", "Fechar o Pomolotl."]),
];

/// Character pose files (`Pose::file_name`).
const POSES: &[(&str, [&str; 2])] = &[
    (
        "idle.svg",
        [
            "Before starting and while paused. Required: a missing pose uses this one.",
            "Antes de começar e pausado. Obrigatório: uma pose que faltar usa esta.",
        ],
    ),
    ("focus.svg", ["While focusing.", "Durante o foco."]),
    ("short_break.svg", ["Short break.", "Pausa curta."]),
    ("long_break.svg", ["Long break.", "Pausa longa."]),
];

/// The generated parts of the page: `(marker name, html)`.
fn blocks() -> Vec<(&'static str, String)> {
    vec![
        ("settings", settings()),
        ("keys", key_names()),
        ("cli", cli()),
        ("theme-fields", described(THEME_FIELDS, |_| None)),
        ("theme-example", theme_example()),
        ("themes", themes()),
        ("poses", described(POSES, |_| None)),
        ("colors", colors()),
        ("characters", characters()),
    ]
}

fn settings() -> String {
    let defaults = toml::Table::try_from(Config::default()).unwrap();
    let mut out = String::new();
    for (section, title, intro) in SECTIONS {
        let table = defaults[*section].as_table().unwrap();
        writeln!(
            out,
            "<div class=\"ref\" id=\"sec-{section}\">\n<h3><code>[{section}]</code> {}</h3>",
            both(*title)
        )
        .unwrap();
        if !intro[0].is_empty() {
            writeln!(out, "<p>{}</p>", both(*intro)).unwrap();
        }
        out.push_str("<dl>\n");
        if *section == "custom_theme" {
            let fields = THEME_FIELDS.iter().filter(|(k, _)| *k != "name");
            for (key, text) in fields {
                setting(&mut out, section, key, table.get(*key), *text);
            }
        } else {
            let prefix = format!("{section}.");
            for (key, text) in SETTINGS {
                if let Some(key) = key.strip_prefix(&prefix) {
                    setting(&mut out, section, key, table.get(key), *text);
                }
            }
        }
        out.push_str("</dl>\n</div>\n");
    }
    out
}

fn setting(
    out: &mut String,
    section: &str,
    key: &str,
    value: Option<&toml::Value>,
    text: [&str; 2],
) {
    let line = match value {
        // Settings are f32: print 0.6, not the f64 widening 0.6000000238418579.
        Some(toml::Value::Float(f)) => format!("{key} = {:?}", *f as f32),
        Some(v) => esc(&format!("{key} = {v}")),
        None => format!(
            "{key} <span class=\"unset\">{}</span>",
            both(["(not set)", "(vazio)"])
        ),
    };
    let text = text.map(|t| {
        t.replace("{mini_w}", &Mini::MIN_SIZE[0].to_string())
            .replace("{mini_h}", &Mini::MIN_SIZE[1].to_string())
            .replace("{win_w}", &Window::MIN_SIZE[0].to_string())
            .replace("{win_h}", &Window::MIN_SIZE[1].to_string())
    });
    writeln!(
        out,
        "<div id=\"{section}.{key}\"><dt><code>{line}</code></dt><dd>{}</dd></div>",
        both([&text[0], &text[1]])
    )
    .unwrap();
}

/// A `<dl>` of `name → description`, with an optional extra after the name.
fn described(items: &[(&str, [&str; 2])], extra: impl Fn(&str) -> Option<String>) -> String {
    let mut out = String::from("<dl>\n");
    for (name, text) in items {
        writeln!(
            out,
            "<div><dt><code>{}</code>{}</dt><dd>{}</dd></div>",
            esc(name),
            extra(name).unwrap_or_default(),
            both(*text)
        )
        .unwrap();
    }
    out.push_str("</dl>");
    out
}

fn key_names() -> String {
    let names: Vec<String> = Key::ALL
        .iter()
        .map(|k| format!("<code>{}</code>", esc(k.name())))
        .collect();
    format!("<p class=\"keys\">{}</p>", names.join(" "))
}

fn cli() -> String {
    described(FLAGS, |_| None).replace("<code>--", "<code>pomolotl --")
}

fn theme_example() -> String {
    let colors = toml::to_string(&Theme::default()).unwrap();
    format!(
        "<pre><code>name = \"My theme\"\n{}</code></pre>",
        esc(colors.trim_end())
    )
}

fn themes() -> String {
    let mut out = String::from("<div class=\"themes\">\n");
    for (name, t) in Themes::builtin().iter() {
        writeln!(
            out,
            "<div class=\"theme\" style=\"background:{};color:{}\"><b>{}</b>\
             <small style=\"color:{}\">{}</small>\
             <i style=\"background:{}\"></i><i style=\"background:{}\"></i><i style=\"background:{}\"></i></div>",
            hex(t.background),
            hex(t.text),
            esc(name),
            hex(t.muted),
            both(if t.is_dark() { ["dark", "escuro"] } else { ["light", "claro"] }),
            hex(t.focus),
            hex(t.short_break),
            hex(t.long_break),
        )
        .unwrap();
    }
    out.push_str("</div>");
    out
}

fn colors() -> String {
    format!(
        "<dl>\n<div><dt><code>{}</code></dt><dd>{}</dd></div>\n<div><dt><code>{}</code></dt><dd>{}</dd></div>\n</dl>",
        character::ACCENT_SENTINEL,
        both([
            "becomes the phase color (focus, short break, long break).",
            "vira a cor da fase (foco, pausa curta, pausa longa).",
        ]),
        character::INK_SENTINEL,
        both([
            "becomes the theme's `muted` color.",
            "vira a cor `muted` do tema.",
        ]),
    )
}

fn characters() -> String {
    let mut out = String::from("<div class=\"chars\">\n");
    for c in Characters::builtin().iter() {
        writeln!(
            out,
            "<figure><img src=\"characters/{id}/{file}\" alt=\"\" width=\"100\" height=\"80\" loading=\"lazy\">\
             <figcaption>{}<br><code>character = \"{id}\"</code></figcaption></figure>",
            both([&c.name, c.name_in("pt-BR")]),
            id = esc(&c.id),
            file = Pose::Idle.file_name(),
        )
        .unwrap();
    }
    out.push_str("</div>");
    out
}

/// English and Portuguese side by side; the page shows one (`html.lang-pt`).
fn both(text: [&str; 2]) -> String {
    format!(
        "<span class=\"en\">{}</span><span class=\"pt\" lang=\"pt-BR\">{}</span>",
        inline(text[0]),
        inline(text[1])
    )
}

/// Escapes HTML and turns `code` and [text](#link) into tags.
fn inline(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find(['`', '[']) {
        out.push_str(&esc(&rest[..i]));
        rest = &rest[i..];
        if let Some(code) = rest.strip_prefix('`') {
            let end = code.find('`').expect("closing backtick");
            write!(out, "<code>{}</code>", esc(&code[..end])).unwrap();
            rest = &code[end + 1..];
        } else if let Some((label, after)) = rest[1..].split_once("](") {
            let (href, after) = after.split_once(')').expect("closing parenthesis");
            write!(out, "<a href=\"{}\">{}</a>", esc(href), inline(label)).unwrap();
            rest = after;
        } else {
            out.push('[');
            rest = &rest[1..];
        }
    }
    out.push_str(&esc(rest));
    out
}

fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn hex(c: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

/// Replace what's between `<!-- gen:NAME -->` and `<!-- /gen:NAME -->`.
fn splice(page: &str, name: &str, html: &str) -> String {
    let open = format!("<!-- gen:{name} -->");
    let close = format!("<!-- /gen:{name} -->");
    let start = page
        .find(&open)
        .unwrap_or_else(|| panic!("{open} missing in site/docs.html"));
    let start = start + open.len();
    let end = start
        + page[start..]
            .find(&close)
            .unwrap_or_else(|| panic!("{close} missing"));
    format!("{}\n{}\n{}", &page[..start], html.trim_end(), &page[end..])
}

#[test]
fn website_docs_are_up_to_date() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("site/docs.html");
    let page = std::fs::read_to_string(&path)
        .unwrap()
        .replace("\r\n", "\n");
    let mut fresh = page.clone();
    for (name, html) in blocks() {
        fresh = splice(&fresh, name, &html);
    }
    if fresh != page {
        if std::env::var_os("POMOLOTL_BLESS").is_some() {
            std::fs::write(&path, fresh).unwrap();
        } else {
            panic!("site/docs.html is out of date: run `POMOLOTL_BLESS=1 cargo test docs`");
        }
    }
}

#[test]
fn docs_describe_every_setting() {
    let defaults = toml::Table::try_from(Config::default()).unwrap();
    let mut real: Vec<String> = Vec::new();
    for (section, value) in &defaults {
        let table = value.as_table().expect("every setting is in a [section]");
        if section != "custom_theme" {
            real.extend(table.keys().map(|k| format!("{section}.{k}")));
        }
    }
    real.extend(UNSET.iter().map(|k| k.to_string()));
    real.sort();
    let mut documented: Vec<String> = SETTINGS.iter().map(|(k, _)| k.to_string()).collect();
    documented.sort();
    assert_eq!(
        documented, real,
        "SETTINGS must describe every key of Config"
    );

    let sections: Vec<&str> = SECTIONS.iter().map(|(s, ..)| *s).collect();
    let mut real_sections: Vec<&str> = defaults.keys().map(String::as_str).collect();
    let mut sorted = sections.clone();
    sorted.sort();
    real_sections.sort();
    assert_eq!(sorted, real_sections, "SECTIONS must list every [section]");

    let theme = toml::Table::try_from(Theme::default()).unwrap();
    let mut fields: Vec<&str> = theme.keys().map(String::as_str).chain(["name"]).collect();
    let mut documented: Vec<&str> = THEME_FIELDS.iter().map(|(k, _)| *k).collect();
    fields.sort();
    documented.sort();
    assert_eq!(
        documented, fields,
        "THEME_FIELDS must describe every theme color"
    );

    let flags: Vec<&str> = instance::FLAGS.iter().map(|(f, _)| *f).collect();
    let documented: Vec<&str> = FLAGS.iter().map(|(f, _)| *f).collect();
    assert_eq!(
        documented, flags,
        "FLAGS must match instance::FLAGS (same order)"
    );

    let poses: Vec<&str> = Pose::ALL.iter().map(|p| p.file_name()).collect();
    let documented: Vec<&str> = POSES.iter().map(|(f, _)| *f).collect();
    assert_eq!(documented, poses, "POSES must match Pose::ALL");
}

#[test]
fn inline_markup() {
    assert_eq!(
        inline("a `b<c` [d](#e) [f"),
        "a <code>b&lt;c</code> <a href=\"#e\">d</a> [f"
    );
}
