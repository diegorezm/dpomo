#!/usr/bin/env bash
# Smoke test of the .dmg: install the app bundle, launch it with `open`, run a
# 1-minute focus via `dpomo --toggle`, quit with `dpomo --quit`.
#   smoke-macos.sh <dpomo-macos.dmg> <out dir>
set -euo pipefail
dmg=$1
out=$(cd "$(dirname "$2")" && pwd)/$(basename "$2")
mkdir -p "$out"
cfg="$HOME/Library/Application Support/dpomo"
mkdir -p "$cfg"
cp "$(dirname "$0")/config.toml" "$cfg/config.toml"

hdiutil attach "$dmg" -mountpoint /Volumes/dpomo -nobrowse
mkdir -p "$HOME/Applications"
cp -R /Volumes/dpomo/dpomo.app "$HOME/Applications/"
hdiutil detach /Volumes/dpomo
app="$HOME/Applications/dpomo.app"
bin="$app/Contents/MacOS/dpomo"
codesign --verify --verbose "$app"
file "$bin"
shot() { screencapture -x "$out/$1.png" || echo "screencapture failed"; }

open --stdout "$out/dpomo.log" --stderr "$out/dpomo.log" "$app"
sleep 8
shot 1-started
pgrep -x dpomo || { echo "dpomo exited at startup"; cat "$out/dpomo.log"; exit 1; }

"$bin" --toggle
sleep 5
shot 2-running
sleep 60
shot 3-break-started

"$bin" --quit
sleep 3
if pgrep -x dpomo; then echo "dpomo --quit did not exit"; exit 1; fi
cp "$cfg/state.toml" "$out/"
grep -q '^sessions = 1' "$cfg/state.toml" || { echo "focus session not recorded"; cat "$cfg/state.toml"; exit 1; }
echo "smoke test passed"
