#!/usr/bin/env bash
# Assembles the website into _site/ (used by .github/workflows/pages.yml).
# Preview locally: bash site/build.sh && python -m http.server -d _site
set -euo pipefail
cd "$(dirname "$0")/.."
rm -rf _site
mkdir -p _site
cp site/index.html site/docs.html site/style.css site/robots.txt site/sitemap.xml _site/
cp assets/icon.svg assets/icon.png assets/screenshot.png assets/fonts/LilitaOne-Regular.ttf _site/
for dir in characters/*/; do
  id=$(basename "$dir")
  mkdir -p "_site/characters/$id"
  cp "$dir"*.svg "_site/characters/$id/"
done
echo "site built in _site/"
