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

### Changed
- The cozy layout fills the whole window (no card border).

## [0.1.0] - 2026-11-07

First public release: cozy layout with the axolotl, classic ring layout, 8 themes + custom themes as
TOML files, system tray, notifications, single instance, start with the system.

[1.0.0]: https://github.com/diegorezm/dpomo/releases/tag/v1.0.0
[0.1.0]: https://github.com/diegorezm/dpomo/releases/tag/v0.1.0
