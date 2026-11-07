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
| `dirs` | Locates the OS config dir. |

egui 0.37 API notes (it changed a lot vs. older tutorials you may remember):
- `App::ui(&mut self, ui: &mut Ui, frame)` replaces `update(ctx, frame)`. Non-UI work goes in `App::logic(ctx, frame)`.
- Panels take a `&mut Ui`: `CentralPanel::default().show(ui, …)`, `Panel::top(id)`.
- `Frame::new()`, `corner_radius` (not `rounding`), `ui.scope_builder(UiBuilder::new().max_rect(r), …)`.
- `Id::new` is deprecated → `ui.make_persistent_id(salt)` or `Id::unique`.
- `ctx.set_visuals_of(theme, v)`, `ctx.all_styles_mut(…)`, `ui.fonts_mut(|f| …)`.
- When unsure, grep the source in `~/.cargo/registry/src/*/egui-0.37.0/` instead of guessing.

## Architecture

```
src/
├── main.rs              window options, loads Config, starts eframe
├── app.rs               DpomoApp: owns ALL state, applies Actions, repaint scheduling, saving
├── timer.rs             pure state machine (Phase, Timer, TimerEvent). No egui. Unit-tested.
├── config.rs            Config structs (serde, TOML), load/save, defaults. Unit-tested.
├── theme.rs             Theme struct, PRESETS list, apply-to-egui, hex color serde
└── ui/
    ├── mod.rs
    ├── widgets.rs       hand-painted ring, icons, icon_button, pill, card
    ├── timer_view.rs    main screen → returns Option<timer_view::Action>
    └── settings_view.rs settings screen, edits &mut Config → returns Option<settings_view::Action>
```

### Data flow (keep it this way)

```
input ─► view fn (draws, returns Action) ─► App::apply_* ─► Timer / Config mutation
                                             App::logic ─► Timer::tick ─► TimerEvent ─► App::on_timer_event
```

- **Views never mutate `Timer`.** They return an `Action` enum. Settings view is the one exception: it edits `Config` in place (it's a form).
- **`Timer` takes `now: Instant` as a parameter** everywhere. Never call `Instant::now()` inside `timer.rs`; that is what makes it testable.
- **`App` caches what it pushed to egui** (`applied_theme`, `applied_on_top`, `title`) and only re-sends on change.

### Performance rules

- Repaints are scheduled in `App::schedule_repaint`: ~30 fps while running with `smooth_animation`, otherwise exactly once per displayed second. Nothing when paused.
- Animations (`animate_bool`, `RingAnim`) request repaints only while they run.
- Don't allocate big things per frame; don't load fonts/images per frame; don't call `Theme::apply` per frame.

## Recipes

### Add a theme preset
Add one line to `PRESETS` in `src/theme.rs` (`theme(bg, surface, text, muted, focus, short, long)` as `0xRRGGBB`). That's it — the settings picker and config pick it up automatically. Check contrast for text/muted on background, and test the screen.

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

## Conventions

- UI text is **Portuguese (pt-BR)**; code, comments and docs for agents are English.
- Rust 2024 edition. Zero clippy warnings. `cargo fmt` default style.
- Draw icons with the painter (`widgets::paint_icon`), not emoji/icon fonts — they render inconsistently.
- Colors come from `Theme`, never hard-coded in views (except `Color32::TRANSPARENT`).
- Sizes in the timer view are relative to the ring radius so the UI scales with the window.
- Add a unit test when you touch `timer.rs` or `config.rs` logic.
- Keep dependencies minimal; justify any new crate in the PR description.

## Verifying visual changes

There's no automated UI test. Run the app and look at it, in a dark theme (default `Rosé Pine`) **and** a light one (`Latte`), at the minimum window size (280×380) and with `ui_scale` at 0.75 and 1.75.
Tip: set the theme by editing the config file (path is shown at the bottom of the settings screen), e.g. `%APPDATA%\dpomo\config.toml` on Windows.

## Gotchas

- The config is saved when leaving the settings screen and on window close — not on every change. If you add settings outside the settings screen, call `App::save`.
- `Timer::sync_settings` only applies new durations to a phase that hasn't started; this is intentional.
- `Timer::skip` does not count the skipped focus session as completed; `tick` (natural finish) does.
- Release builds on Windows have no console (`windows_subsystem = "windows"`), so `eprintln!` is invisible there; debug builds show it.
