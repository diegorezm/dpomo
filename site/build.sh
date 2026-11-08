#!/usr/bin/env bash
# Builds the website into _site/ (see site/build.py). Used by .github/workflows/pages.yml.
set -euo pipefail
cd "$(dirname "$0")/.."
if command -v python3 >/dev/null && python3 -c "" 2>/dev/null; then python3 site/build.py; else python site/build.py; fi
