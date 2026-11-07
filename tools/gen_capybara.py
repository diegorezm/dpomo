# Generates characters/capybara/*.svg. Usage: python tools/gen_capybara.py characters/capybara
# Preview: cargo run --example svg_preview -- preview.png f6ecdf 2 characters/capybara/*.svg
import os, sys

sys.path.insert(0, os.path.dirname(__file__))
from gen_axolotl import OUT, ACC, INK, SW, svg, shadow, steam, heart, z, star  # noqa: E402

FUR = "#c08a5b"
FUR_D = "#a06f45"
SNOUT = "#d7aa7f"
BLUSH = "#ec9a86"
ORANGE = "#f6a23c"
LEAF = "#6cbf5b"
WOOD = "#d29b65"
WOOD_D = "#b07a48"
WATER = "#a6dcef"


def head(cx, cy, eyes="calm", glasses=False):
    """Front-facing capybara head centered at (cx, cy). ~64 wide, ~54 tall."""
    g = [
        f'<ellipse cx="{cx-25}" cy="{cy-25}" rx="6" ry="5" fill="{FUR_D}" stroke="{OUT}" stroke-width="3"/>',
        f'<ellipse cx="{cx+25}" cy="{cy-25}" rx="6" ry="5" fill="{FUR_D}" stroke="{OUT}" stroke-width="3"/>',
        f'<rect x="{cx-37}" y="{cy-28}" width="74" height="52" rx="21" fill="{FUR}" stroke="{OUT}" stroke-width="{SW}"/>',
        f'<ellipse cx="{cx-14}" cy="{cy-19}" rx="9" ry="3.5" fill="#ffffff" opacity="0.3"/>',
        # broad, boxy muzzle with the big dark nose capybaras have
        f'<rect x="{cx-25}" y="{cy-3}" width="50" height="25" rx="12" fill="{SNOUT}" stroke="{OUT}" stroke-width="2.6"/>',
        f'<rect x="{cx-10}" y="{cy}" width="20" height="9" rx="4.5" fill="#5b3b2a"/>',
        f'<path d="M{cx},{cy+9} L{cx},{cy+13} M{cx-5},{cy+13} Q{cx-2.5},{cy+16} {cx},{cy+13} Q{cx+2.5},{cy+16} {cx+5},{cy+13}" fill="none" stroke="{OUT}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>',
        f'<ellipse cx="{cx-29}" cy="{cy+1}" rx="4.5" ry="3" fill="{BLUSH}" opacity="0.85"/>',
        f'<ellipse cx="{cx+29}" cy="{cy+1}" rx="4.5" ry="3" fill="{BLUSH}" opacity="0.85"/>',
    ]
    for x in (cx - 18, cx + 18):
        y = cy - 12
        if eyes == "open":
            g.append(f'<circle cx="{x}" cy="{y}" r="3.4" fill="{OUT}"/>')
            g.append(f'<circle cx="{x+1.1}" cy="{y-1.2}" r="1.1" fill="#ffffff"/>')
        elif eyes == "calm":   # the classic unbothered capybara look
            g.append(f'<path d="M{x-4.5},{y} L{x+4.5},{y}" stroke="{OUT}" stroke-width="2.8" stroke-linecap="round"/>')
        elif eyes == "happy":
            g.append(f'<path d="M{x-4.5},{y+2} Q{x},{y-3} {x+4.5},{y+2}" fill="none" stroke="{OUT}" stroke-width="2.8" stroke-linecap="round"/>')
        elif eyes == "sleep":
            g.append(f'<path d="M{x-4.5},{y-1} Q{x},{y+4} {x+4.5},{y-1}" fill="none" stroke="{OUT}" stroke-width="2.8" stroke-linecap="round"/>')
        elif eyes == "down":
            g.append(f'<circle cx="{x}" cy="{y+2}" r="3.2" fill="{OUT}"/>')
    if glasses:
        g.append(f'<g fill="#ffffff" fill-opacity="0.25" stroke="{OUT}" stroke-width="2.6">'
                 f'<circle cx="{cx-18}" cy="{cy-11}" r="8.5"/><circle cx="{cx+18}" cy="{cy-11}" r="8.5"/></g>')
        g.append(f'<path d="M{cx-9.5},{cy-12} Q{cx},{cy-15} {cx+9.5},{cy-12}" fill="none" stroke="{OUT}" stroke-width="2.6"/>')
    return "\n  ".join(g)


def body(cx=100, cy=120):
    return "\n  ".join([
        f'<ellipse cx="{cx}" cy="{cy}" rx="48" ry="23" fill="{FUR}" stroke="{OUT}" stroke-width="{SW}"/>',
        f'<ellipse cx="{cx-26}" cy="{cy+20}" rx="9" ry="5" fill="{FUR_D}" stroke="{OUT}" stroke-width="{SW}"/>',
        f'<ellipse cx="{cx+26}" cy="{cy+20}" rx="9" ry="5" fill="{FUR_D}" stroke="{OUT}" stroke-width="{SW}"/>',
    ])


def paw(x, y):
    return f'<ellipse cx="{x}" cy="{y}" rx="7" ry="5.5" fill="{FUR_D}" stroke="{OUT}" stroke-width="{SW}"/>'


def orange(x, y, r=11):
    return "\n  ".join([
        f'<circle cx="{x}" cy="{y}" r="{r}" fill="{ORANGE}" stroke="{OUT}" stroke-width="3"/>',
        f'<ellipse cx="{x-3.5}" cy="{y-4}" rx="3.5" ry="2" fill="#ffffff" opacity="0.5"/>',
        f'<path d="M{x},{y-r} q4,-7 10,-5 q-3,6 -10,5 Z" fill="{LEAF}" stroke="{OUT}" stroke-width="2.2" stroke-linejoin="round"/>',
    ])


poses = {}

poses["idle"] = svg("\n  ".join([
    shadow(100, 142, 56),
    body(),
    head(100, 76, "calm"),
    star(40, 44, 7), star(28, 62, 4),
]))

poses["focus"] = svg("\n  ".join([
    shadow(100, 142, 56),
    body(),
    head(100, 72, "down", glasses=True),
    # book held up, back cover facing us
    f'<rect x="72" y="100" width="56" height="36" rx="3" fill="{ACC}" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<path d="M100,100 L100,136" stroke="{OUT}" stroke-width="2.6"/>',
    f'<path d="M80,112 L94,112 M80,119 L91,119 M106,112 L120,112 M106,119 L117,119" stroke="#ffffff" stroke-width="2.6" stroke-linecap="round" opacity="0.6"/>',
    paw(72, 118), paw(128, 118),
    # mug
    f'<path d="M168,116 q9,0 9,6.5 q0,6.5 -9,6.5" fill="none" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<rect x="150" y="110" width="19" height="22" rx="4" fill="{ACC}" stroke="{OUT}" stroke-width="{SW}"/>',
    steam(156, 104),
]))

poses["short_break"] = svg("\n  ".join([
    shadow(100, 142, 56),
    body(),
    head(100, 80, "happy"),
    orange(100, 41),
    star(150, 48, 6), star(162, 66, 3.5),
]))

poses["long_break"] = svg("\n  ".join([
    shadow(100, 146, 66),
    # onsen: relaxing in a wooden tub
    steam(36, 82), steam(152, 82),
    head(100, 74, "sleep"),
    # towel folded on the head
    f'<rect x="84" y="38" width="32" height="12" rx="5" fill="{ACC}" stroke="{OUT}" stroke-width="3"/>',
    f'<path d="M90,44 L110,44" stroke="#ffffff" stroke-width="2.4" opacity="0.5" stroke-linecap="round"/>',
    f'<ellipse cx="100" cy="102" rx="62" ry="9" fill="{WATER}" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<path d="M38,102 L44,138 Q46,144 54,144 L146,144 Q154,144 156,138 L162,102 Q100,114 38,102 Z" fill="{WOOD}" stroke="{OUT}" stroke-width="{SW}" stroke-linejoin="round"/>',
    f'<path d="M66,110 L68,142 M100,113 L100,144 M134,110 L132,142" stroke="{WOOD_D}" stroke-width="2.6" stroke-linecap="round"/>',
    f'<path d="M41,120 Q100,132 159,120" fill="none" stroke="{OUT}" stroke-width="3"/>',
    z(146, 44, 0.8), z(160, 26, 1.05), z(176, 6, 1.3),
]))


def main(out_dir):
    os.makedirs(out_dir, exist_ok=True)
    for name, text in poses.items():
        with open(os.path.join(out_dir, f"{name}.svg"), "w", encoding="utf8", newline="\n") as f:
            f.write(text)
    with open(os.path.join(out_dir, "character.toml"), "w", encoding="utf8", newline="\n") as f:
        f.write('name = "Capivara"\n')
    print("ok", list(poses))


if __name__ == "__main__":
    main(sys.argv[1])
