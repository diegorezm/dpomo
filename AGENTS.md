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
| `eframe` 0.37 / `egui` 0.37 | GUI. **glow** backend, `default-features = false` → ~5 MB binary. Don't switch to wgpu. |
| `serde` + `toml` | Config file. |
| `egui_extras` (`svg`) | Rasterizes character SVGs (resvg). Only the `svg` feature. |
| `dirs` | Locates the OS config dir. |
| `tray-icon` | Tray icon + menu. `default-features = false, features = ["ksni"]`: Linux uses D-Bus (KSNI), **no GTK**. |
| `notify-rust` | Native notifications (WinRT toast / D-Bus / macOS). |
| `auto-launch` | "Start with the system" (registry Run key / `.desktop` / LaunchAgent). |
| `windows-registry` | Windows only: registers the toast AppUserModelID. |

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
tools/gen_axolotl.py     generates characters/axolotl/*.svg — edit this, not the SVGs
tools/gen_icon.py        generates assets/icon.{svg,png,ico,icns} from the axolotl head
packaging/               installer sources (Inno Setup .iss, Linux .desktop, macOS Info.plist)
.github/workflows/       ci.yml (fmt/clippy/test), release.yml (installers on v* tags)
examples/svg_preview.rs  renders SVGs to a PNG grid (check art without running the app)
src/
├── main.rs              window options, loads Config, starts eframe
├── app.rs               DpomoApp: owns ALL state, applies Actions/Commands, repaint scheduling, saving
├── command.rs           Command enum: requests from outside egui (tray, second launch)
├── tray.rs              tray icon + menu, live ring icon (render_ring is also the window icon)
├── instance.rs          single instance over 127.0.0.1:47613 (second launch → "show" → exit)
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

## Recipes

### Add a built-in theme
Drop a file in `themes/` (copy an existing one). Fields: `name`, `background`, `surface`, `text`, `muted`, `focus`, `short_break`, `long_break`, all `"#rrggbb"`. No code changes: `build.rs` embeds every `themes/*.toml` and `cargo test` checks they parse and have unique names. The picker sorts dark themes first, then light, alphabetically. Check contrast for `text`/`muted` on `background` and look at it in the app.

Users add themes the same way in `<config dir>/dpomo/themes/`; a user theme with the same `name` as a built-in replaces it. Bad files don't crash: they're skipped and listed in the settings screen (`Themes::errors`).

### Add a theme color
Add the field to `Theme` (with `#[serde(with = "hex")]`) **and to every file in `themes/`** (tests fail otherwise), then use it in views. Consider `#[serde(default = …)]` so users' existing theme files keep working.

### Add a setting
1. Add the field to the right struct in `config.rs` (`TimerSettings`, `Appearance`, `Behavior`) **and** to its `Default` impl. All structs are `#[serde(default)]`, so old config files keep working.
2. Add a control in `settings_view.rs` inside the matching `section(…)`.
3. Read it where needed. If it changes egui/window state, apply it in `App::sync_environment` with a change cache.

### Add a timer action / shortcut
1. Add a variant to `timer_view::Action`.
2. Emit it from `timer_view::show` (button) and/or `App::handle_shortcuts` (key).
3. Handle it in `App::apply_timer_action`. Use `with_ring_anim` if the ring jumps.

### Add a new screen
Add a `View` variant in `app.rs`, a `ui/<name>_view.rs` with `pub fn show(…) -> Option<Action>`, and route it in `App::ui`.

### React to a phase finishing (sound, notification, stats…)
Hook into `App::on_timer_event`. Put any heavy work (audio decoding, HTTP) on a thread; never block the UI thread.

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

### Tweak the axolotl
Edit `tools/gen_axolotl.py`, run `python tools/gen_axolotl.py characters/axolotl`, preview with the example
on a light (`f6ecdf`) and a dark (`191724`) background.

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

To release: bump `version` in `Cargo.toml`, commit, `git tag vX.Y.Z && git push origin vX.Y.Z`.
To test packaging without publishing: run the workflow manually (workflow_dispatch) and download the artifacts.

- File names have **no version** on purpose: README links use `releases/latest/download/<file>`. Don't rename them.
- Never change `AppId` in the `.iss` file (breaks upgrades/uninstall).
- The installer is per-user (`PrivilegesRequired=lowest`), kills a running dpomo before installing, and on
  uninstall removes the autostart `Run` value and the toast AUMID key (both created by the app at runtime).
- Icons: `assets/icon.svg` is the source (generated by `tools/gen_icon.py` from the axolotl head).
  `icon.png/.ico/.icns` are generated from it — rerun `python tools/gen_icon.py` (needs Pillow) after changes.
  The `.ico` is embedded into the `.exe` by `build.rs` (`winresource`); the window icon is rasterized from
  `icon.svg` at startup (`main.rs::window_icon`).
- Nothing is code-signed/notarized (costs money). README tells users how to get past SmartScreen/Gatekeeper.

## Conventions

- UI text is **Portuguese (pt-BR)**; code, comments and docs for agents are English.
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
