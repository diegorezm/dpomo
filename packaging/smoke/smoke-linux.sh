#!/usr/bin/env bash
# Smoke test of the installed .deb on a virtual X11 desktop (Xvfb) with a real
# D-Bus session and notification daemon (dunst). Run under dbus-run-session.
#   smoke-linux.sh <out dir>
# Starts dpomo, starts a 1-minute focus with `dpomo --toggle`, waits for it to
# finish (notification + sound + stats), quits with `dpomo --quit`.
set -euo pipefail
out=$(realpath -m "$1")
mkdir -p "$out"
cfg="$HOME/.config/dpomo"
mkdir -p "$cfg"
cp "$(dirname "$0")/config.toml" "$cfg/config.toml"

Xvfb :99 -screen 0 1280x800x24 &
export DISPLAY=:99
sleep 2
dunst >"$out/dunst.log" 2>&1 &
sleep 1
shot() { import -window root "$out/$1.png"; }

dpomo >"$out/dpomo.log" 2>&1 &
pid=$!
sleep 8
shot 1-started
kill -0 "$pid" || { echo "dpomo exited at startup"; cat "$out/dpomo.log"; exit 1; }

dpomo --toggle
sleep 5
shot 2-running
sleep 60
shot 3-break-started

dpomo --quit
sleep 3
if kill -0 "$pid" 2>/dev/null; then echo "dpomo --quit did not exit"; exit 1; fi
cp "$cfg/state.toml" "$out/"
grep -q '^sessions = 1' "$cfg/state.toml" || { echo "focus session not recorded"; cat "$cfg/state.toml"; exit 1; }
echo "smoke test passed"
