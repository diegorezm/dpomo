# TODO

Things that didn't make it into 1.0, roughly in priority order. Check items off (or delete them) as
they're done, and add the change to `CHANGELOG.md`.

## Needs a human

- [ ] **Test on Linux for real** (Ubuntu/Fedora, GNOME + KDE, X11 + Wayland). The CI only proves it
      builds and packages. Checklist:
  - [ ] `.deb` installs (`sudo apt install ./dpomo-linux-amd64.deb`) and shows up in the app menu with its icon
  - [ ] tray icon appears (GNOME needs the AppIndicator extension); left click shows/hides, menu works
  - [ ] notification and sound at the end of a phase
  - [ ] closing hides to the tray; "Quit" exits; opening it again shows the running window
  - [ ] "Start when I log in" creates/removes `~/.config/autostart/dpomo.desktop`
  - [ ] `config.toml` live reload, theme/character folders
- [ ] **Test on macOS for real** (Apple Silicon + Intel). Same checklist, plus: the `.dmg` opens, the
      app runs after right-click → Open, the Dock/menu bar icon, notifications permission prompt.
- [ ] **Code signing on Windows** to remove the SmartScreen warning. SignPath Foundation signs open
      source projects for free (https://signpath.org) — requires applying with the project owner's
      account. Then add a signing step to `release.yml`.
- [ ] **macOS notarization** (needs a paid Apple Developer account).

## Code

- [ ] Memory: ~110 MB working set on Windows, mostly the OpenGL driver (`atio6axx.dll` alone was
      ~80 MB on an AMD GPU). Try eframe's `wgpu` backend (DX12) and compare RAM/binary size.
- [ ] Binary grew from ~6 MB to ~10 MB in 1.0 (rodio decoders + rustls for the update check). Options:
      drop mp3/vorbis (keep wav), or check updates via the OS (`curl`) instead of rustls.
- [ ] More languages (add a `Strings` static in `src/i18n.rs` + a `Language` variant).
- [ ] Statistics view with a small weekly chart (data is already in `state.toml`).
- [ ] Optional ticking sound during focus.
- [ ] Per-phase sounds (different sound for "break starts" and "focus starts").
- [ ] Global keyboard shortcut to start/pause while the window is hidden.
- [ ] Watch the themes/characters folders like `config.toml` (today: "Reload" button).
- [ ] Linux ARM (aarch64) builds.

## Project

- [ ] Publish to winget / Scoop / Homebrew / AUR.
- [ ] Website or GitHub Pages with screenshots and download buttons.
