# 🍅 Pomolotl

A **beautiful**, **lightweight** and **customizable** Pomodoro timer, with a little friend who works
and rests with you. 🦎💗 Built with Rust and [egui](https://github.com/emilk/egui).

**English** · [Português](README.pt-BR.md) · 🌐 [Website](https://diegorezm.github.io/pomolotl/)

![Pomolotl](assets/screenshot.png)

## What's Pomodoro?

Focus for **25 minutes**, rest for **5**. Every **4 sessions**, take a long **15-minute** break. That's it.
(You can change all of these numbers.)

## Download

| System | Download | How to install |
| ------ | -------- | -------------- |
| **Windows** | [pomolotl-setup.exe](https://github.com/diegorezm/pomolotl/releases/latest/download/pomolotl-setup.exe) | Open it and click *Next* until the end. No admin password needed. |
| **Linux** (Ubuntu/Debian) · *beta* | [pomolotl-linux-amd64.deb](https://github.com/diegorezm/pomolotl/releases/latest/download/pomolotl-linux-amd64.deb) | Double-click it, or `sudo apt install ./pomolotl-linux-amd64.deb` |
| **macOS** · *beta* | [pomolotl-macos.dmg](https://github.com/diegorezm/pomolotl/releases/latest/download/pomolotl-macos.dmg) | Drag Pomolotl into *Applications*. |

More options (a portable Windows version, a `.tar.gz` for other Linux distros, ARM64 Linux builds) are on the
[releases page](https://github.com/diegorezm/pomolotl/releases/latest).

> **Windows:** if you see "Windows protected your PC", click *More info* → *Run anyway*.
> That happens because the installer isn't digitally signed yet.
>
> **macOS:** the first time, **right-click** Pomolotl → *Open* → *Open*. The app isn't notarized by Apple,
> so macOS asks once.
>
> **Linux:** to see the tray icon on GNOME, install the *AppIndicator* extension (KDE works out of the box).

To uninstall on Windows: *Settings → Apps → Pomolotl → Uninstall*.

## How to use

1. Open the app.
2. Press the play button (or **Space**).
3. When time's up, you get a **sound** and a **notification**. 🔔

Your companion changes with you: it waves while idle, works while you focus, relaxes on short breaks and
naps on long ones.

### It runs in the background

Pomolotl lives in the **system tray** (the small icons next to the clock):

- Closing the window **doesn't quit**: the timer keeps going in the tray.
- The tray icon shows a live **progress ring**; hover it to see the time left.
- **Click** the icon to show/hide the window; **right-click** for the menu (start/pause, skip, restart, **quit**).
- Opened Pomolotl twice by accident? No problem, it just shows the window that's already running.
- Closed the app or restarted your computer mid-session? It **picks up where you left off** (paused).
- Want it to start with your computer? Turn on **Start when I log in** in the settings.

### Floating timer

Want the time always in sight? Turn on the **floating timer** (the small square button at the top of
the window, the `M` key, or the tray menu): a little window that stays on top of everything with the time
left, the phase and your companion. Drag it anywhere, drag its corner to make it bigger or smaller,
double-click it to open Pomolotl. It keeps working while the main window is hidden in the tray.

### Shortcuts

| Key | What it does |
| --- | ------------ |
| `Space` | Start / pause |
| `R` | Restart the phase |
| `S` | Skip to the next phase |
| `1` `2` `3` | Go to focus / short break / long break |
| `M` | Show / hide the floating timer |
| `,` | Open settings |
| `Esc` | Close settings |

Every shortcut can be changed in `config.toml` (the `[keys]` section, see [Configuration](#configuration)).

### Command line

Running `pomolotl` again with one of these talks to the Pomolotl that's already running (or starts it):

| Command | What it does |
| ------- | ------------ |
| `pomolotl --toggle` | Start / pause |
| `pomolotl --skip` | Skip to the next phase |
| `pomolotl --reset` | Restart the phase |
| `pomolotl --show` | Show the window |
| `pomolotl --mini` | Show / hide the floating timer |
| `pomolotl --settings` | Open the settings |
| `pomolotl --quit` | Quit |

Handy for a **global shortcut**: bind `pomolotl --toggle` to a key in your desktop's keyboard settings
(on Windows: a shortcut to `pomolotl.exe --toggle` with a *Shortcut key* in its properties).

## Make it yours

Click the settings icon (top-right corner). You can change:

- 🦎 **Style**: *Cozy* (with a character) or *Classic* (a minimal progress ring)
- 🐾 **Character**: axolotl, tomato or capybara — or [your own](#make-your-own-character-)
- 🎨 **Theme**: 8 built-in (Rosé Pine, Catppuccin, Tokyo Night, Nord, Gruvbox, Mono, Latte, Paper) or [your own](#make-your-own-theme-)
- ⏱️ **Durations** of focus and breaks, and how many sessions until the long break
- 🔔 **Sound**: chime, bell, soft, or any `.wav`/`.mp3`/`.ogg` file, with volume
- 📊 **Statistics**: sessions and focus time today, this week and all time
- 🌍 **Language**: English or Portuguese (follows your system by default)
- 🔍 Interface **size**, always on top, auto-start phases, tray behavior and more

## Configuration

**All** settings live in a plain text file, `config.toml`. The settings screen just edits that file for you.
It's created the first time Pomolotl opens:

| System | Location |
| ------ | -------- |
| Windows | `%APPDATA%\pomolotl\config.toml` |
| Linux | `~/.config/pomolotl/config.toml` |
| macOS | `~/Library/Application Support/pomolotl/config.toml` |

(The settings screen has **Open config.toml** and **Open folder** buttons.)

- **Edit and save: the app applies it right away**, no restart needed.
- Delete anything you don't care about: missing keys use their default.
- Made a typo? The app tells you which line is wrong and **never overwrites your file** until you fix it.

Everything you can configure, with the defaults (the [configuration guide](https://diegorezm.github.io/pomolotl/docs.html)
explains each option, plus key names and how to make themes and characters):

```toml
[timer]
focus_minutes = 25
short_break_minutes = 5
long_break_minutes = 15
sessions_before_long_break = 4
auto_start_breaks = false        # start breaks by themselves
auto_start_focus = false         # start focus by itself

[appearance]
language = "auto"                # "auto" (follow the system), "en" or "pt-BR"
layout = "cozy"                  # "cozy" (character) or "classic" (ring)
character = "axolotl"            # character folder: "axolotl", "tomato", "capybara"…
theme = "Rosé Pine"              # a theme name, or "custom" (uses [custom_theme])
ring_thickness = 10.0            # "classic" only
ui_scale = 1.0                   # interface size
smooth_animation = true          # false = uses even less CPU
glow = true                      # ring glow ("classic")
show_session_dots = true

[behavior]
always_on_top = false
notify_on_finish = true          # flash the taskbar
focus_on_finish = false          # bring the window to the front
time_in_title = true             # time left in the window title
tray = true                      # tray icon
close_to_tray = true             # closing = hiding in the tray
system_notification = true
start_with_system = false
start_hidden = false             # start hidden in the tray
check_updates = true             # see "Privacy" below

[sound]
enabled = true
kind = "chime"                   # when a break starts: "chime", "bell", "soft" or "file"
file = ""                        # path to a .wav/.mp3/.ogg when kind = "file"
# focus_kind = "bell"            # when focus starts (missing = same as kind)
# focus_file = ""                # its file, when focus_kind = "file" (empty = file)
volume = 0.6                     # 0.0 to 1.0
ticking = false                  # soft tick every second while focusing
ticking_volume = 0.3

[window]
width = 360.0
height = 500.0
remember_size = true             # keep the last size when closing

[mini]                           # floating timer: small, always on top
enabled = false
width = 260.0                    # drag it to move, drag its corner to resize
height = 96.0
show_character = true
# x = 1600.0 / y = 40.0 are added when you move it

[keys]                           # key names: "Space", "R", ",", "1", "F5", "Escape", "ArrowLeft"…
toggle = "Space"
reset = "R"
skip = "S"
settings = ","
focus = "1"
short_break = "2"
long_break = "3"
mini = "M"
back = "Escape"

[custom_theme]                   # used when theme = "custom"
background = "#191724"
surface = "#26233a"
text = "#e0def4"
muted = "#908caa"
focus = "#eb6f92"
short_break = "#9ccfd8"
long_break = "#c4a7e7"
```

Your progress and statistics are kept separately in `state.toml` (same folder), written by the app.

### Make your own theme 🎨

Themes are `.toml` files. Put yours in a `themes` folder next to `config.toml`
(e.g. `%APPDATA%\pomolotl\themes\dracula.toml`):

```toml
name = "Dracula"
background = "#282a36"
surface = "#44475a"      # cards, buttons and the ring track
text = "#f8f8f2"
muted = "#a4a8c4"        # secondary text
focus = "#ff79c6"        # focus color
short_break = "#50fa7b"  # short break color
long_break = "#bd93f9"   # long break color
```

Then click **Reload themes and characters** in the settings. Done!

Rather pick colors by clicking? Choose **Custom**, tweak the colors and click **Save as file**.
Want to share it? Just send the file to someone. 😉

### Make your own character 🦎

Create a folder in `characters`, next to `config.toml` (e.g. `%APPDATA%\pomolotl\characters\cat\`), with these SVGs:

| File | When it shows |
| ---- | ------------- |
| `idle.svg` | idle (**required**) |
| `focus.svg` | focusing |
| `short_break.svg` | short break |
| `long_break.svg` | long break |

Missing one? `idle.svg` is used instead. To name it, add a `character.toml`:

```toml
name = "Cat"

[names]
"pt-BR" = "Gato"
```

Tip: in your SVG, the color `#e05a4f` becomes the phase color (red for focus, green for breaks…) and
`#7a6464` becomes the theme's secondary text color, so your character matches every theme.

## Performance

Measured on Windows 11 (release build, 16-core CPU):

| Situation | CPU | Note |
| --------- | --- | ---- |
| Paused | **0%** | nothing runs until you click |
| Running, window open | ~0.2% of the machine | smooth 30 fps animation (turn it off for less) |
| Running, hidden in the tray | ~0.05% | wakes up once per second |

Memory (Task Manager's "Memory" column): **~12 MB** open and paused, **~45 MB** while the smooth
animation runs, **~7 MB** hidden in the tray. The Windows executable is ~13 MB.

No OpenGL on your machine (virtual machine, remote desktop, Windows on ARM)? Pomolotl switches to
Direct3D 12 by itself; it uses more memory that way, but it works.

## Privacy

Pomolotl has no accounts, no telemetry and no ads. Your settings, statistics and session stay on your
computer, in the config folder.

The only network request is the **update check**: once a day Pomolotl asks GitHub's public API
(`api.github.com`) for the number of the latest release. Nothing about you is sent, only a normal web
request (GitHub sees your IP address, see [GitHub's privacy statement](https://docs.github.com/site-policy/privacy-policies/github-general-privacy-statement)).
It never downloads or installs anything by itself. Turn it off in the settings, with
`check_updates = false`, or by unchecking *Check for new versions* in the Windows installer.

## Build it yourself

You need [Rust](https://rustup.rs):

```sh
cargo install --git https://github.com/diegorezm/pomolotl
```

> **Linux:** first install `libxkbcommon-dev`, `libgl1-mesa-dev`, `libwayland-dev` and `libasound2-dev`.

Contributing? Start with [AGENTS.md](AGENTS.md) — it explains how the project works (it's written for
AI agents, but humans like it too).

## License

[MIT](LICENSE). Do whatever you want. 🙂
