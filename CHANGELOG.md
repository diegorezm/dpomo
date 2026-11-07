# Changelog

All notable changes to dpomo. Format based on [Keep a Changelog](https://keepachangelog.com/),
versions follow [Semantic Versioning](https://semver.org/).

## [1.0.0] - 2026-11-07

### Added
- **English UI** (now the main language) with Portuguese available; follows the OS by default
  (`appearance.language`).
- **Sounds** when a phase ends: chime, bell, soft (synthesized, no audio files) or your own
  `.wav`/`.mp3`/`.ogg`, with volume and a test button (`[sound]`).
- **Resume after restart**: the current session is saved to `state.toml` and restored (paused) the
  next time dpomo opens on the same day.
- **Statistics**: focus sessions and focus time for today, the last 7 days and all time.
- **Update notice**: checks GitHub releases once a day and shows a link when a new version exists.
  Never downloads anything; turn it off with `behavior.check_updates = false`.
- Tomato and capybara characters, with a visual character picker.
- `config.toml` is the single source of truth: created on first launch, applied live when edited,
  and never overwritten while it has an error. Shortcuts (`[keys]`) and window size (`[window]`)
  are configurable.
- Installers: Windows (Inno Setup + portable zip), Linux (`.deb`, `.tar.gz`) and macOS (`.dmg`).
- App icon (the axolotl's face) for the executable, window, installer and notifications.
- Command line: `dpomo --toggle`, `--skip`, `--reset`, `--show`, `--quit` control the running app
  (bind them to a global shortcut).
- Release pipeline smoke tests: every package is installed on fresh Windows, Ubuntu 22.04/24.04 and
  macOS (arm64 + Intel) machines and runs a real pomodoro before anything is published.
- Windows installer option to turn off the update check; code signing (SignPath) and macOS
  notarization are wired in the release workflow, ready to switch on.

### Changed
- The cozy layout fills the whole window (no card border).
- Much lower memory use: ~12 MB open and ~7 MB in the tray (was ~110 MB), by handing the GPU
  driver's unused startup memory back to Windows.
- Smaller executable (~9 MB): the update check uses the system's `curl` instead of a bundled
  HTTPS stack.

## [0.1.0] - 2026-11-07

First public release: cozy layout with the axolotl, classic ring layout, 8 themes + custom themes as
TOML files, system tray, notifications, single instance, start with the system.

[1.0.0]: https://github.com/diegorezm/dpomo/releases/tag/v1.0.0
[0.1.0]: https://github.com/diegorezm/dpomo/releases/tag/v0.1.0
