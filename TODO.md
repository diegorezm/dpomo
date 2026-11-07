# TODO

Things that didn't make it into 1.0, roughly in priority order. Check items off (or delete them) as
they're done, and add the change to `CHANGELOG.md`.

## Needs a human

The release workflow now installs every package on fresh Windows, Ubuntu 22.04/24.04 and macOS
(arm64 + Intel) runners and runs a real pomodoro (see AGENTS.md → Smoke tests). What CI can't check:

- [ ] **Linux on a real desktop** (GNOME + KDE, X11 + Wayland):
  - [ ] tray icon appears (GNOME needs the AppIndicator extension); left click shows/hides, menu works
  - [ ] sound plays (CI has no sound card)
  - [ ] "Start when I log in" creates/removes `~/.config/autostart/dpomo.desktop`
  - [ ] window decorations / HiDPI look right on Wayland
- [ ] **macOS on a real Mac**: first launch after downloading (Gatekeeper: right-click → Open), menu bar
      icon, notification permission prompt, sound.
- [ ] **Windows code signing (free)**: apply at https://signpath.org/apply with the owner's GitHub
      account (needs 2FA on GitHub and SignPath). After acceptance, follow AGENTS.md → "Code signing"
      (create the SignPath project + set `SIGNPATH_ORGANIZATION_ID` / `SIGNPATH_API_TOKEN`). The
      workflow, installer and README policy are ready.
- [ ] **macOS notarization**: needs the Apple Developer Program (US$ 99/year). Then set the five
      `APPLE_*` secrets listed in AGENTS.md; the workflow is ready.

## Code

- [ ] More languages (add a `Strings` static in `src/i18n.rs` + a `Language` variant).
- [ ] Statistics view with a small weekly chart (data is already in `state.toml`).
- [ ] Optional ticking sound during focus.
- [ ] Per-phase sounds (different sound for "break starts" and "focus starts").
- [ ] Built-in global shortcut while the window is hidden (today: bind `dpomo --toggle` in the OS).
- [ ] Watch the themes/characters folders like `config.toml` (today: "Reload" button).
- [ ] Linux ARM (aarch64) builds.

## Project

- [ ] Publish to winget / Scoop / Homebrew / AUR.
- [ ] Website or GitHub Pages with screenshots and download buttons.
