#!/usr/bin/env bash
# Smoke test of the .dmg: install the app bundle, launch it with `open`, run a
# 1-minute focus via `pomolotl --toggle`, quit with `pomolotl --quit`.
#   smoke-macos.sh <pomolotl-macos.dmg> <out dir>
set -euo pipefail
dmg=$1
out=$(cd "$(dirname "$2")" && pwd)/$(basename "$2")
mkdir -p "$out"
cfg="$HOME/Library/Application Support/pomolotl"
mkdir -p "$cfg"
cp "$(dirname "$0")/config.toml" "$cfg/config.toml"

hdiutil attach "$dmg" -mountpoint /Volumes/pomolotl -nobrowse
mkdir -p "$HOME/Applications"
cp -R /Volumes/Pomolotl/Pomolotl.app "$HOME/Applications/"
hdiutil detach /Volumes/pomolotl
app="$HOME/Applications/Pomolotl.app"
bin="$app/Contents/MacOS/pomolotl"
codesign --verify --verbose "$app"
file "$bin"
shot() { screencapture -x "$out/$1.png" || echo "screencapture failed"; }

open --stdout "$out/pomolotl.log" --stderr "$out/pomolotl.log" "$app"
sleep 8
shot 1-started
pgrep -x pomolotl || { echo "pomolotl exited at startup"; cat "$out/pomolotl.log"; exit 1; }

"$bin" --toggle
sleep 5
shot 2-running
sleep 60
shot 3-break-started

"$bin" --quit
sleep 3
if pgrep -x pomolotl; then echo "pomolotl --quit did not exit"; exit 1; fi
cp "$cfg/state.toml" "$out/"
grep -q '^sessions = 1' "$cfg/state.toml" || { echo "focus session not recorded"; cat "$cfg/state.toml"; exit 1; }
echo "smoke test passed"
