#!/usr/bin/env python3
"""Builds the website into _site/ (run by .github/workflows/pages.yml).

The pages in site/ are the English ones. The Portuguese pages (pt/) are made from them here:
elements with data-i18n="key" (text), data-i18n-html="key" (markup) or data-i18n-content="key"
(the content attribute) take their text from site/pt.json. docs.html carries both languages
itself (class="en"/"pt"); its pt/ copy just gets html.lang-pt. Also adds the FAQ structured
data and writes sitemap.xml. Standard library only.

Preview: python site/build.py && python -m http.server -d _site
"""
import html
import json
import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SITE = ROOT / "site"
OUT = ROOT / "_site"
URL = "https://diegorezm.github.io/dpomo/"
PAGES = ["index.html", "docs.html"]
PT = json.loads((SITE / "pt.json").read_text(encoding="utf-8"))
OWN_PAGES = {"", "./", "index.html", "docs.html"}


def path_of(page):
    """URL path of a page, relative to the site root."""
    return "" if page == "index.html" else page


def translate(text, used):
    def value(key):
        if key not in PT:
            raise SystemExit(f"site/pt.json: missing {key!r}")
        used.add(key)
        return PT[key]

    def element(m):
        tag, attrs, kind, key, rest, _ = m.groups()
        inner = value(key) if kind else html.escape(value(key), quote=False)
        return f"<{tag}{attrs} data-i18n{kind or ''}=\"{key}\"{rest}>{inner}</{tag}>"

    text = re.sub(r'<(\w+)([^>]*?) data-i18n(-html)?="(\w+)"([^>]*)>(.*?)</\1>', element, text, flags=re.S)

    def attribute(m):
        key = m.group(1)
        return re.sub(r' content="[^"]*"', f' content="{html.escape(value(key))}"', m.group(0))

    return re.sub(r'<[^>]* data-i18n-content="(\w+)"[^>]*>', attribute, text)


def portuguese(page, text, used):
    text = translate(text, used)
    text = text.replace('<html lang="en">', '<html lang="pt-BR" class="lang-pt" data-root="../">', 1)
    text = text.replace(f'<link rel="canonical" href="{URL}{path_of(page)}">',
                        f'<link rel="canonical" href="{URL}pt/{path_of(page)}">', 1)
    text = text.replace(f'<meta property="og:url" content="{URL}">', f'<meta property="og:url" content="{URL}pt/">')
    text = text.replace('<meta property="og:locale" content="en_US">', '<meta property="og:locale" content="pt_BR">')

    def link(m):
        attr, url = m.groups()
        if re.match(r"(https?:|mailto:|data:|#|/)", url) or url.split("#")[0] in OWN_PAGES:
            return m.group(0)
        return f'{attr}="../{url}"'

    text = re.sub(r'\b(href|src)="([^"]*)"', link, text)
    # The language switch points back to the English page.
    text = re.sub(r'(<a class="chip" id="lang" href=")[^"]*(" hreflang=")pt-BR(" lang=")pt-BR(")',
                  rf'\g<1>../{path_of(page)}\g<2>en\g<3>en\g<4>', text)
    return text


def faq_data(text):
    """FAQPage structured data from the visible <details class="faq"> items."""
    plain = lambda s: html.unescape(re.sub(r"<[^>]+>", "", s)).strip()
    items = re.findall(r'<details class="faq">\s*<summary[^>]*>(.*?)</summary>\s*<p[^>]*>(.*?)</p>', text, re.S)
    if not items:
        return text
    data = {
        "@context": "https://schema.org",
        "@type": "FAQPage",
        "mainEntity": [
            {"@type": "Question", "name": plain(q), "acceptedAnswer": {"@type": "Answer", "text": plain(a)}}
            for q, a in items
        ],
    }
    script = f'<script type="application/ld+json">\n{json.dumps(data, ensure_ascii=False, indent=2)}\n</script>\n'
    return text.replace("</head>", script + "</head>", 1)


def sitemap():
    urls = []
    for page in PAGES:
        en, pt = URL + path_of(page), URL + "pt/" + path_of(page)
        alternates = (f'    <xhtml:link rel="alternate" hreflang="en" href="{en}"/>\n'
                      f'    <xhtml:link rel="alternate" hreflang="pt-BR" href="{pt}"/>\n')
        urls += [f"  <url>\n    <loc>{loc}</loc>\n{alternates}  </url>\n" for loc in (en, pt)]
    return ('<?xml version="1.0" encoding="UTF-8"?>\n'
            '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">\n'
            + "".join(urls) + "</urlset>\n")


def main():
    shutil.rmtree(OUT, ignore_errors=True)
    (OUT / "pt").mkdir(parents=True)
    used = set()
    for page in PAGES:
        text = (SITE / page).read_text(encoding="utf-8").replace("\r\n", "\n")
        (OUT / page).write_text(faq_data(text), encoding="utf-8")
        (OUT / "pt" / page).write_text(faq_data(portuguese(page, text, used)), encoding="utf-8")
    unused = set(PT) - used
    if unused:
        raise SystemExit(f"site/pt.json: unused {sorted(unused)}")
    (OUT / "sitemap.xml").write_text(sitemap(), encoding="utf-8")

    for name in ["style.css", "lang.js", "robots.txt", "llms.txt", *[p.name for p in SITE.glob("google*.html")]]:
        shutil.copy(SITE / name, OUT)
    for name in ["icon.svg", "icon.png", "screenshot.png", "fonts/LilitaOne-Regular.ttf"]:
        shutil.copy(ROOT / "assets" / name, OUT)
    for pack in (ROOT / "characters").iterdir():
        (OUT / "characters" / pack.name).mkdir(parents=True)
        for svg in pack.glob("*.svg"):
            shutil.copy(svg, OUT / "characters" / pack.name)
    print("site built in _site/")


if __name__ == "__main__":
    main()
