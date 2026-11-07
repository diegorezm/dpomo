# AGENTS.md

Guide for AI agents (and humans who like detail) working on **dpomo**, a Pomodoro timer in Rust + egui.
Read this whole file before changing code. It is short on purpose.

## Product priorities (in order)

1. **Aesthetics.** Every pixel matters. Check spacing, alignment and both dark *and* light themes.
2. **Customizability.** New visual/behavior knobs go in `Config` and the settings screen.
3. **Performance.** Idle = zero CPU. Never call `request_repaint()` unconditionally.
4. **Simplicity / extensibility.** Small modules, no clever abstractions, few dependencies.

## Commands

```sh
cargo run                      # debug build
cargo run --release            # what users get (glow backend, LTO, stripped)
cargo test                     # unit tests (timer, config, theme)
cargo clippy --all-targets     # must have zero warnings
cargo fmt                      # must be clean
```

CI (`.github/workflows/ci.yml`) runs fmt, clippy (`-D warnings`) and tests on Linux and Windows.
Run all four locally before you commit.

## Stack

| Crate | Why |
| ----- | --- |
| `eframe` 0.37 / `egui` 0.37 | GUI. **glow** backend, `default-features = false`. Don't switch backends without measuring (see TODO.md). |
| `serde` + `toml` | Config file. |
| `egui_extras` (`svg`) | Rasterizes character SVGs (resvg). Only the `svg` feature. |
| `dirs` | Locates the OS config dir. |
| `tray-icon` | Tray icon + menu. `default-features = false, features = ["ksni"]`: Linux uses D-Bus (KSNI), **no GTK**. |
| `notify-rust` | Native notifications (WinRT toast / D-Bus / macOS). |
| `auto-launch` | "Start with the system" (registry Run key / `.desktop` / LaunchAgent). |
| `windows-registry` | Windows only: registers the toast AppUserModelID. |
| `rodio` | Phase-end sounds. Only `playback` + `wav`/`mp3`/`vorbis`. Linux builds need `libasound2-dev`. |
| `chrono` (`clock`, `serde`) | Local dates for statistics / "same day" session restore. |
| `serde_json` | Parses the daily update check answer. The request itself runs the OS's `curl` (no HTTP/TLS crate: −1.6 MB). |
| `wgpu` (Windows only, `dx12`) | Fallback renderer when OpenGL can't start (VMs, RDP, CI runners, Windows on ARM). `main.rs` tries glow first, then reruns `run_native` with wgpu. `DPOMO_RENDERER=wgpu` forces it. ~+3.8 MB on Windows only. |
| `windows-sys` | Windows only: trims the working set (`system::trim_memory`). Already in the tree via winit. |
| `sys-locale` | OS language for `language = "auto"`. |

egui 0.37 API notes (it changed a lot vs. older tutorials you may remember):
- `App::ui(&mut self, ui: &mut Ui, frame)` replaces `update(ctx, frame)`. Non-UI work goes in `App::logic(ctx, frame)`.
- Panels take a `&mut Ui`: `CentralPanel::default().show(ui, …)`, `Panel::top(id)`.
- `Frame::new()`, `corner_radius` (not `rounding`), `ui.scope_builder(UiBuilder::new().max_rect(r), …)`.
- `Id::new` is deprecated → `ui.make_persistent_id(salt)` or `Id::unique`.
- `ctx.set_visuals_of(theme, v)`, `ctx.all_styles_mut(…)`, `ui.fonts_mut(|f| …)`.
- When unsure, grep the source in `~/.cargo/registry/src/*/egui-0.37.0/` instead of guessing.

## Architecture

```
build.rs                 embeds themes/*.toml and characters/<id>/* into the binary
themes/*.toml            built-in themes, one file each
characters/<id>/         built-in character packs (idle/focus/short_break/long_break .svg + character.toml)
assets/fonts/            Lilita One (display font for the cozy layout) + its OFL license
tools/gen_axolotl.py     generates characters/axolotl/*.svg — edit this, not the SVGs (shared helpers live here)
tools/gen_tomato.py      generates characters/tomato/*.svg
tools/gen_capybara.py    generates characters/capybara/*.svg
tools/gen_icon.py        generates assets/icon.{svg,png,ico,icns} from the axolotl head
packaging/               installer sources (Inno Setup .iss, Linux .desktop, macOS Info.plist)
packaging/smoke/         smoke tests run by release.yml on the real packages (one script per OS)
site/                    website (https://diegorezm.github.io/dpomo/), deployed by .github/workflows/pages.yml
.github/workflows/       ci.yml (fmt/clippy/test), release.yml (installers on v* tags)
examples/svg_preview.rs  renders SVGs to a PNG grid (check art without running the app)
src/
├── main.rs              window options, loads Config, starts eframe
├── app.rs               DpomoApp: owns ALL state, applies Actions/Commands, repaint scheduling, saving
├── command.rs           Command enum: requests from outside egui (tray, second launch, file watcher, updates)
├── i18n.rs              ALL user-facing strings (Strings struct, EN + PT_BR statics), active language
├── sound.rs             synthesized chime/bell/soft + custom files, played on a short-lived thread
├── state.rs             state.toml: current session (resume after restart) + daily statistics
├── updates.rs           daily "new version?" check against GitHub releases (notify only)
├── tray.rs              tray icon + menu, live ring icon (render_ring is also the window icon)
├── instance.rs          single instance over 127.0.0.1:47613; later launches send a request
│                        ("show", or `--toggle/--skip/--reset/--quit`) and exit
├── system.rs            notifications, autostart, Windows AUMID registration
├── timer.rs             pure state machine (Phase, Timer, TimerEvent). No egui. Unit-tested.
├── config.rs            Config structs (serde, TOML), load/save, defaults. Unit-tested.
├── theme.rs             Theme struct, Themes registry (built-in + user files), apply-to-egui, hex serde
├── character.rs         Character packs registry, Pose, recoloring, CharacterImage (texture cache)
└── ui/
    ├── mod.rs
    ├── widgets.rs       hand-painted ring, icons, icon_button, pill, card
    ├── cozy_view.rs     main screen, "cozy" layout (default): card + character + chunky timer
    ├── timer_view.rs    main screen, "classic" layout (ring); also owns Action + RingAnim shared by both
    └── settings_view.rs settings screen, edits &mut Config → returns Option<settings_view::Action>
```

### Data flow (keep it this way)

```
input ─► view fn (draws, returns Action) ─► App::apply_* ─► Timer / Config mutation
tray / 2nd launch ─► mpsc Command + ctx.request_repaint() ─► App::handle_commands (in logic)
                                             App::logic ─► Timer::tick ─► TimerEvent ─► App::on_timer_event
```

### Background / daemon model

- Closing the window (with tray enabled) sends `CancelClose` + `Visible(false)`. The process keeps running.
- **While hidden, eframe never calls `App::ui`, only `App::logic`.** So everything that must work in the
  background (ticking, commands, tray updates, notifications, title) lives in `logic`. `ui` is drawing only.
- Viewport commands sent from `logic` are processed even when hidden (that's how "show window" works).
- Hidden + running = one wake-up per second (see `schedule_repaint`). Paused = no wake-ups at all.
- Quitting = `App::quit`: save config, drop the tray (avoids a ghost icon on Windows), `process::exit(0)`.
- If the tray can't be created (e.g. Linux without a StatusNotifier host) the app disables `behavior.tray`
  and never hides the window, so the user can't get stuck with an invisible app.

- **Views never mutate `Timer`.** They return an `Action` enum. Settings view is the one exception: it edits `Config` in place (it's a form).
- **`Timer` takes `now: Instant` as a parameter** everywhere. Never call `Instant::now()` inside `timer.rs`; that is what makes it testable.
- **`App` owns the `Themes` registry**; `Config` only stores the theme *name*. Resolve with `config.theme(&themes)`.
- **`App` caches what it pushed to egui** (`applied_theme`, `applied_on_top`, `title`) and only re-sends on change.

### Performance rules

- Repaints are scheduled in `App::schedule_repaint`: ~30 fps while running with `smooth_animation`, otherwise exactly once per displayed second. Nothing when paused.
- Animations (`animate_bool`, `RingAnim`) request repaints only while they run.
- Don't allocate big things per frame; don't load fonts/images per frame; don't call `Theme::apply` per frame.
- Memory: the OpenGL driver's startup leaves ~100 MB of pages that are never touched again.
  `App::trim_memory_when_due` calls `system::trim_memory` (Windows `EmptyWorkingSet`) 2 s after startup
  and 2 s after hiding: ~110 MB → ~12 MB open, ~7 MB in the tray. Keep glow: wgpu was measured at
  300–400 MB (DX12/Vulkan drivers) and +3–5 MB of binary (measured on an AMD GPU, Nov 2026).

## Recipes

### Add a built-in theme
Drop a file in `themes/` (copy an existing one). Fields: `name`, `background`, `surface`, `text`, `muted`, `focus`, `short_break`, `long_break`, all `"#rrggbb"`. No code changes: `build.rs` embeds every `themes/*.toml` and `cargo test` checks they parse and have unique names. The picker sorts dark themes first, then light, alphabetically. Check contrast for `text`/`muted` on `background` and look at it in the app.

Users add themes the same way in `<config dir>/dpomo/themes/`; a user theme with the same `name` as a built-in replaces it. Bad files don't crash: they're skipped and listed in the settings screen (`Themes::errors`).

### Add a theme color
Add the field to `Theme` (with `#[serde(with = "hex")]`) **and to every file in `themes/`** (tests fail otherwise), then use it in views. Consider `#[serde(default = …)]` so users' existing theme files keep working.

### Config file model
`config.toml` is the single source of truth for **every** setting (the user asked for this explicitly).
- `Config::load` writes defaults on first launch so the file always exists.
- `Config::watch` polls the file's mtime once a second and sends `Command::ReloadConfig`;
  `App::reload_config` applies it live. Our own saves also trigger it — harmless, the content is equal.
- If the file is invalid: the app keeps running on the last good config, shows the error (line + reason) as a
  badge on the main screen and a card in settings, and **`App::save` refuses to write** until the file is fixed
  (or the user clicks "Substituir pelos ajustes atuais"). Never silently overwrite a user's file.
- Saved files start with a short comment header (`config::HEADER`).
- Both READMEs have a full annotated `config.toml` reference ("Configuration" / "Configuração"): **update both when you add a setting** (a test fails otherwise).

### Add a setting
1. Add the field to the right struct in `config.rs` (`TimerSettings`, `Appearance`, `Behavior`, `Window`, `Keys`) **and** to its `Default` impl. All structs are `#[serde(default)]`, so old config files keep working.
2. Add a control in `settings_view.rs` inside the matching `section(…)` (optional for power-user knobs).
3. Document it in the `config.toml` reference of `README.md` and `README.pt-BR.md`, add UI strings to `i18n.rs`.
4. Read it where needed. If it changes egui/window state, apply it in `App::sync_environment` with a change cache.

### Add a timer action / shortcut
1. Add a variant to `timer_view::Action`.
2. Emit it from the views (button) and/or add a binding: a field in `config::Keys` (+ default) and a row in
   the `bindings` table in `App::handle_shortcuts`. Shortcuts are never hard-coded.
3. Handle it in `App::apply_timer_action`. Use `with_ring_anim` if the ring jumps.

### Add a new screen
Add a `View` variant in `app.rs`, a `ui/<name>_view.rs` with `pub fn show(…) -> Option<Action>`, and route it in `App::ui`.

### React to a phase finishing (sound, notification, stats…)
Hook into `App::on_timer_event` (it already records stats, saves state, plays the sound and notifies).
Put any heavy work (audio decoding, HTTP) on a thread; never block the UI thread.

### Translations (i18n)
- English is the main language; Portuguese (pt-BR) is the second. **Every** user-facing string is a field of
  `i18n::Strings` with a value in both `EN` and `PT_BR` — the compiler rejects a missing translation.
- Use `i18n::tr().field`; placeholders like `{key}` are filled with `i18n::fill(template, &[("key", value)])`.
- Phase names: `tr().phase(p)` / `tr().phase_tab(p)`. Plurals: add a `_one`/`_many` pair (see `sessions`).
- The active language is global (`i18n::set`, called by `App::sync_environment` when the config changes).
  Changing it recreates the tray (menu labels are set at creation).
- Character names: `character.toml` has `name` (English) + `[names] "pt-BR" = …`; use `Character::display_name()`.
- To add a language: a `Language` variant (+ serde rename), a `Strings` static, and the matches in `i18n.rs`.
- Code, comments, AGENTS.md, CHANGELOG and TODO are English. Keep `README.md` (English) and
  `README.pt-BR.md` in sync — a test checks both config references against the defaults.

### State file (`state.toml`)
Not configuration — written only by the app, next to `config.toml`. `App::save_state` stores the current
session (phase, elapsed, completed sessions, date) on every timer action, on phase end, when hiding/quitting
and every 30 s while running. On startup a session from **today** is restored paused (`Timer::restore`);
older ones are ignored. `history` keeps per-day focus sessions/minutes for the statistics section.

### Sounds
Built-ins are synthesized in `sound::synthesize` (no audio assets). `rising = true` when a break starts,
`false` when focus starts. Each play opens the default output device on a thread and closes it after —
nothing stays open while idle. Custom files go through `rodio::Decoder` (wav/mp3/ogg).

### Update check
`updates::spawn_checker` runs once at startup (+5 s) and every 24 h, only if `behavior.check_updates`.
The request is the OS's `curl` (`%SystemRoot%\System32\curl.exe` on Windows, absolute path on purpose,
`CREATE_NO_WINDOW`); no curl = no check, silently. The `.deb` *recommends* curl.
It sends `Command::UpdateAvailable(version)`; the app shows a badge (main screen) and a button (settings)
that open the releases page. It must never download or install anything. README "Privacy" documents it.

### Add a new phase
Add it to `Phase` + `Phase::ALL`, then fix every `match` the compiler flags (labels, `TimerSettings::duration`, `Theme::accent`, `Timer::next_phase`).

### Layouts
`config.appearance.layout` picks `cozy_view` (default) or `timer_view` (classic). Both return the same
`timer_view::Action`, so actions/shortcuts work in both. The cozy view fills the whole window edge to
edge (no card/border — the user prefers it that way); `App::ui` gives it a zero-margin frame and the view
pads itself. The display font family is `cozy_view::DISPLAY_FONT`
(`cozy_view::display_font(size)`), installed in `app::install_fonts`.

### Characters
- Pose per state: `Pose::for_timer` → focus running = `Focus`, focus paused/not started = `Idle`, breaks = their pose.
- SVG art uses a 200×160 viewBox. The cozy view keeps that 1.25 ratio and snaps the size to 16 px steps so
  window resizes don't re-rasterize constantly.
- Recolor sentinels (exact lowercase hex, replaced in `Character::svg`): `#e05a4f` → phase accent,
  `#7a6464` → theme `muted`. The outline color `#3b2a2a` is **not** replaced on purpose (dark outlines look
  right on both light and dark cards).
- `CharacterImage` builds a new `bytes://` URI only when character/pose/colors change and calls
  `ctx.forget_image` on the old one. Don't build `Image::from_bytes` with fresh bytes every frame.
- resvg is built **without** the `text` feature: `<text>` in SVGs renders nothing. Draw letters as paths
  (see the `z()` helper in `tools/gen_axolotl.py`).
- `cargo test` rasterizes every built-in pose, so broken SVGs fail CI.

### Add a character
Create `characters/<id>/` with at least `idle.svg` (+ optional poses and `character.toml` with `name`).
`build.rs` embeds it automatically. Preview: `cargo run --example svg_preview -- out.png f6ecdf 2 characters/<id>/*.svg`.

### Tweak a built-in character
Built-ins: `axolotl` (default), `tomato`, `capybara`. Edit `tools/gen_<name>.py`, run
`python tools/gen_<name>.py characters/<id>`, preview with the example on a light (`f6ecdf`) and a dark
(`191724`) background. New generators should import the shared helpers (`svg`, `shadow`, `steam`, `z`,
`star`, `heart`, color sentinels) from `gen_axolotl.py` so all characters share one style: 200×160 canvas,
`#3b2a2a` outlines at 3.5 px, ground shadow at y≈142, faces readable (don't cover them with props).

### Command-line requests
`dpomo --toggle|--skip|--reset|--show|--quit` (`instance::FLAGS`). If dpomo is running, the request goes
over the single-instance socket as `dpomo:<verb>\n` and the new process exits; if not, dpomo starts and
applies it (`--quit` just exits). Each verb maps to a `Command` in `instance::command_for`. Users bind
these to global shortcuts; the smoke tests use them to drive the app.

### Add a tray menu item
Add a `Command` variant, a `MenuItem::with_id("<id>", …)` in `Tray::new`, map the id in the `MenuEvent`
handler, and handle the variant in `App::handle_commands`.

## Releases & installers

`.github/workflows/release.yml` builds everything when a `v*` tag is pushed and attaches it to a GitHub Release:

| File | Built with | Source |
| ---- | ---------- | ------ |
| `dpomo-setup.exe` | Inno Setup (`iscc`) | `packaging/windows/dpomo.iss` |
| `dpomo-windows-portable.zip` | `Compress-Archive` | release `.exe` |
| `dpomo-linux-amd64.deb` | `cargo deb` | `[package.metadata.deb]` in `Cargo.toml`, `packaging/linux/dpomo.desktop` |
| `dpomo-linux-x86_64.tar.gz` | `tar` | binary + `.desktop` + icon |
| `dpomo-macos.dmg` | `lipo` + `hdiutil` (universal, ad-hoc signed) | `packaging/macos/Info.plist` |

To release: bump `version` in `Cargo.toml`, add a `CHANGELOG.md` entry, commit, `git tag vX.Y.Z && git push origin vX.Y.Z`.
To test packaging without publishing: run the workflow manually (workflow_dispatch) and download the artifacts.

**Smoke tests** (`smoke-*` jobs, scripts in `packaging/smoke/`): each package is installed on fresh runners
(Windows installer; `.deb` with apt on Ubuntu 22.04 + 24.04 under Xvfb + D-Bus + dunst; `.dmg` on macOS
arm64 + Intel), dpomo runs a real 1-minute focus driven by `dpomo --toggle`, quits with `--quit`, and the
script checks `state.toml` recorded the session (Windows also uninstalls and checks the registry cleanup).
Screenshots and logs are uploaded as `smoke-*` artifacts — **look at them** after packaging changes.
The release job only publishes if every smoke test passes.

- File names have **no version** on purpose: README links use `releases/latest/download/<file>`. Don't rename them.
- Never change `AppId` in the `.iss` file (breaks upgrades/uninstall).
- The installer is per-user (`PrivilegesRequired=lowest`), kills a running dpomo before installing, and on
  uninstall removes the autostart `Run` value and the toast AUMID key (both created by the app at runtime).
- Icons: `assets/icon.svg` is the source (generated by `tools/gen_icon.py` from the axolotl head).
  `icon.png/.ico/.icns` are generated from it — rerun `python tools/gen_icon.py` (needs Pillow) after changes.
  The `.ico` is embedded into the `.exe` by `build.rs` (`winresource`); the window icon is rasterized from
  `icon.svg` at startup (`main.rs::window_icon`).
- **Code signing** is wired in `release.yml` but stays off until configured:
  - Windows (free, SignPath Foundation): after the project is accepted at https://signpath.org/apply, create
    in SignPath a project `dpomo`, a signing policy `release-signing`, artifact configurations `exe`
    (zip root containing `dpomo.exe`) and `installer` (zip root containing `dpomo-setup.exe`), link the
    GitHub connector, then add the repo variable `SIGNPATH_ORGANIZATION_ID` and secret `SIGNPATH_API_TOKEN`.
    Every release then waits for a manual approval in SignPath. Update README "Code signing policy" (drop
    "application pending") and the SmartScreen notes.
  - SignPath rules we must keep: README "Code signing policy" section (attribution line, roles, privacy),
    update check disclosed + optional at install time (the installer's *Check for new versions* task),
    uninstaller, MFA on GitHub and SignPath, product name/version in file metadata.
  - macOS (needs the paid Apple Developer Program): secrets `APPLE_CERTIFICATE_P12` (base64 Developer ID
    Application .p12), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_PASSWORD`
    (app-specific password). With them set, the app is signed with hardened runtime and the dmg notarized
    + stapled; without them it stays ad-hoc signed.

## Website

`site/index.html` is one self-contained page (inline CSS/JS, no build tools, no CDN). `site/build.sh`
copies it plus the app's own art (character SVGs, icon, screenshot, Lilita One) into `_site/`;
`pages.yml` deploys it on pushes that touch `site/`, `characters/` or `assets/`. English is the HTML
text (what Google indexes); Portuguese lives in the `PT` object in the script, keyed by `data-i18n`
(text) / `data-i18n-html` (markup). New characters show up only if added to the `.friends` buttons.
Keep the "Code signing policy" section and the privacy text in sync with the README (SignPath requires
both on the download page). Preview: `bash site/build.sh && python -m http.server -d _site`.
The project has **no budget**: free hosting only, no custom domain.

## Conventions

- UI text lives in `src/i18n.rs` only (English + Portuguese); never hard-code a user-facing string elsewhere.
- Code, comments and docs for agents are English.
- Rust 2024 edition. Zero clippy warnings. `cargo fmt` default style.
- Draw icons with the painter (`widgets::paint_icon`), not emoji/icon fonts — they render inconsistently.
- Illustrations are SVG character packs; UI chrome is painted. Don't add raster images.
- Colors come from `Theme`, never hard-coded in views (except `Color32::TRANSPARENT`).
- Sizes in the timer view are relative to the ring radius so the UI scales with the window.
- Add a unit test when you touch `timer.rs` or `config.rs` logic.
- Keep dependencies minimal; justify any new crate in the PR description.

## Verifying visual changes

There's no automated UI test. Run the app and look at it in **both layouts** (cozy and classic), in a dark theme (default `Rosé Pine`) **and** a light one (`Latte`), at the minimum window size (280×380) and with `ui_scale` at 0.75 and 1.75.
Tip: set the theme by editing the config file (path is shown at the bottom of the settings screen), e.g. `%APPDATA%\dpomo\config.toml` on Windows.

## Gotchas

- Unfinished work and ideas live in `TODO.md`; user-visible changes go in `CHANGELOG.md`.

- Only one dpomo runs at a time. When testing, kill the old one first or your new build will just
  "show" the old one and exit immediately. (`Get-Process dpomo | Stop-Process` / `pkill dpomo`)
- Tests that touch the OS (autostart) are `#[ignore]`: `cargo test -- --ignored`.
- Windows toasts need the AUMID registered in `system::init` (HKCU, no admin). Without it they show as "Windows PowerShell".

- The config is saved when leaving the settings screen and on window close — not on every change. If you add settings outside the settings screen, call `App::save`.
- Theme names are display names (`"Rosé Pine"`), not file names. `theme::DEFAULT_NAME` must match the `name` in `themes/rose-pine.toml` (tested).
- `"custom"` is reserved: it means `Config::custom_theme`, edited in the settings screen.
- `Timer::sync_settings` only applies new durations to a phase that hasn't started; this is intentional.
- `Timer::skip` does not count the skipped focus session as completed; `tick` (natural finish) does.
- Release builds on Windows have no console (`windows_subsystem = "windows"`), so `eprintln!` is invisible there; debug builds show it.
