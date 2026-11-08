# TODO

Things that didn't make it into 1.0, roughly in priority order. Check items off as they're done
(keep them, so it's clear what happened), and add the change to `CHANGELOG.md`.

## Needs a human

The release workflow now installs every package on fresh Windows, Ubuntu 22.04/24.04 and macOS
(arm64 + Intel) runners and runs a real pomodoro (see AGENTS.md → Smoke tests). What CI can't check:

- [x] **Linux, automated** (smoke test on Ubuntu 22.04 + 24.04): the `.deb` installs with apt (its
      dependencies resolve), the window renders, a phase ends, the notification shows, statistics are
      saved, `dpomo --quit` exits.
- [x] **macOS, automated** (smoke test on arm64 + Intel): the `.dmg` mounts, the signed app launches,
      menu bar icon shows, a phase ends, statistics are saved. Found and fixed a "Choose Application"
      dialog on the first notification.
- [x] **Windows, automated**: silent install, a full phase, quit, uninstall leaves nothing behind.
      Found and fixed: dpomo closed at startup on machines without OpenGL (now falls back to Direct3D 12).
- [ ] **Linux on a real desktop** (GNOME + KDE, X11 + Wayland):
  - [ ] tray icon appears (GNOME needs the AppIndicator extension); left click shows/hides, menu works
  - [ ] sound plays (CI has no sound card)
  - [ ] "Start when I log in" creates/removes `~/.config/autostart/dpomo.desktop`
  - [ ] window decorations / HiDPI look right on Wayland
- [ ] **macOS on a real Mac**: first launch after downloading (Gatekeeper: right-click → Open), menu bar
      icon, notification permission prompt, sound.
- [x] Code signing wired into `release.yml` (SignPath for Windows, Developer ID + notarization for
      macOS), off until configured. The installer can turn the update check off.
- [ ] **Windows code signing (free)**: apply at https://signpath.org/apply with the owner's GitHub
      account (needs 2FA on GitHub and SignPath). After acceptance, follow AGENTS.md → "Code signing"
      (create the SignPath project + set `SIGNPATH_ORGANIZATION_ID` / `SIGNPATH_API_TOKEN`). The
      workflow and installer are ready. The "Code signing policy" section was removed from the
      README and site until then (it read as if releases were already signed): put it back when
      applying (text in AGENTS.md).
- macOS notarization: **not planned** (the Apple Developer Program costs US$ 99/year and dpomo is a
  free hobby project with no budget). The workflow supports it if that ever changes (AGENTS.md).

## Code

- [x] Memory: tried eframe's `wgpu` backend (300–400 MB, worse); kept glow and trim the working set
      after startup / when hiding: ~110 MB → ~12 MB open, ~7 MB in the tray.
- [x] Binary size: update check uses the OS's `curl` instead of ureq/rustls (−1.6 MB). The Windows
      build is ~12.7 MB because of the Direct3D fallback (worth it).
- [x] Command line: `dpomo --toggle/--skip/--reset/--show/--quit` (also usable as global shortcuts).
- [x] Windows 11 title bar uses the theme's colors.
- [x] Settings in their own bigger window (sidebar, larger text, explanations under options).
- [x] Floating timer: borderless always-on-top window, movable and resizable, remembers its place.
- [ ] More languages (add a `Strings` static in `src/i18n.rs` + a `Language` variant).
- [ ] Statistics view with a small weekly chart (data is already in `state.toml`).
- [ ] Optional ticking sound during focus.
- [ ] Per-phase sounds (different sound for "break starts" and "focus starts").
- [ ] Built-in global shortcut while the window is hidden (today: bind `dpomo --toggle` in the OS).
- [ ] Watch the themes/characters folders like `config.toml` (today: "Reload" button).
- [ ] Linux ARM (aarch64) builds.

## Project

- [x] Website on GitHub Pages: https://diegorezm.github.io/dpomo/ (free, no custom domain).
- [ ] Publish to winget / Scoop / Homebrew / AUR.
- [x] Website: Google Search Console ownership file (`site/google73dd98d454ee7b5e.html`).
- [ ] Search Console: click Verify, then submit `sitemap.xml` (owner's Google account).
- [x] SEO on the site: search phrases in titles, FAQ (+ FAQPage data), Portuguese pages at `/pt/`
      with hreflang, richer SoftwareApplication data, `llms.txt`.
- [ ] Be found off-site (what search engines and AI assistants actually learn from), after v1.0.0:
      English repo description + topics; AlternativeTo listing; winget / Scoop / Flathub / AUR;
      awesome lists (awesome-rust, awesome-pomodoro); a post on r/productivity / r/rust / Show HN.
