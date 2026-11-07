# Generates characters/tomato/*.svg. Usage: python tools/gen_tomato.py characters/tomato
# Preview: cargo run --example svg_preview -- preview.png f6ecdf 2 characters/tomato/*.svg
import os, sys

sys.path.insert(0, os.path.dirname(__file__))
from gen_axolotl import OUT, ACC, INK, SW, svg, shadow, steam, heart, z, star  # noqa: E402

RED = "#ef5d4c"        # not the accent sentinel: the tomato stays red
RED_D = "#d9483a"
LEAF = "#6cbf5b"
LEAF_D = "#4f9a44"
BLUSH = "#ff9a8a"


def face(cx, cy, eyes="open", mouth="smile"):
    g = []
    g.append(f'<ellipse cx="{cx-19}" cy="{cy+9}" rx="5.5" ry="3.2" fill="{BLUSH}" opacity="0.9"/>')
    g.append(f'<ellipse cx="{cx+19}" cy="{cy+9}" rx="5.5" ry="3.2" fill="{BLUSH}" opacity="0.9"/>')
    for x in (cx - 13, cx + 13):
        if eyes == "open":
            g.append(f'<circle cx="{x}" cy="{cy+1}" r="3.8" fill="{OUT}"/>')
            g.append(f'<circle cx="{x+1.3}" cy="{cy-0.4}" r="1.3" fill="#ffffff"/>')
        elif eyes == "down":
            g.append(f'<circle cx="{x}" cy="{cy+4}" r="3.6" fill="{OUT}"/>')
            g.append(f'<circle cx="{x+1.2}" cy="{cy+2.8}" r="1.2" fill="#ffffff"/>')
        elif eyes == "happy":
            g.append(f'<path d="M{x-4.5},{cy+3} Q{x},{cy-3} {x+4.5},{cy+3}" fill="none" stroke="{OUT}" stroke-width="2.8" stroke-linecap="round"/>')
        elif eyes == "sleep":
            g.append(f'<path d="M{x-4.5},{cy} Q{x},{cy+5} {x+4.5},{cy}" fill="none" stroke="{OUT}" stroke-width="2.8" stroke-linecap="round"/>')
    if mouth == "smile":
        g.append(f'<path d="M{cx-5},{cy+10} Q{cx},{cy+15} {cx+5},{cy+10}" fill="none" stroke="{OUT}" stroke-width="2.4" stroke-linecap="round"/>')
    elif mouth == "flat":
        g.append(f'<path d="M{cx-3.5},{cy+11} L{cx+3.5},{cy+11}" stroke="{OUT}" stroke-width="2.4" stroke-linecap="round"/>')
    elif mouth == "o":
        g.append(f'<ellipse cx="{cx}" cy="{cy+11}" rx="2.2" ry="2.6" fill="{OUT}"/>')
    return "\n  ".join(g)


def crown(cx, top):
    """Green sepals + stem sitting on top of the tomato at y=top."""
    pts = []
    import math
    for i in range(10):
        a = -math.pi / 2 + i * math.pi / 5
        r = 17 if i % 2 == 0 else 6
        pts.append(f"{cx + r * math.cos(a):.1f},{top + 3 + r * math.sin(a) * 0.55:.1f}")
    return "\n  ".join([
        f'<polygon points="{" ".join(pts)}" fill="{LEAF}" stroke="{OUT}" stroke-width="3" stroke-linejoin="round"/>',
        f'<rect x="{cx-2.8}" y="{top-11}" width="5.6" height="12" rx="2.5" fill="{LEAF_D}" stroke="{OUT}" stroke-width="2.6"/>',
    ])


def tomato(cx, cy, eyes="open", mouth="smile", rx=40, ry=34):
    return "\n  ".join([
        f'<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{RED}" stroke="{OUT}" stroke-width="{SW}"/>',
        f'<ellipse cx="{cx-18}" cy="{cy-16}" rx="10" ry="5" fill="#ffffff" opacity="0.45" transform="rotate(-20 {cx-18} {cy-16})"/>',
        crown(cx, cy - ry + 2),
        face(cx, cy + 2, eyes, mouth),
    ])


def nub(x, y, rx=7, ry=5.5, fill=RED):
    return f'<ellipse cx="{x}" cy="{y}" rx="{rx}" ry="{ry}" fill="{fill}" stroke="{OUT}" stroke-width="{SW}"/>'


poses = {}

poses["idle"] = svg("\n  ".join([
    shadow(100, 142, 44),
    nub(86, 138, 9, 5, RED_D), nub(114, 138, 9, 5, RED_D),
    # waving arm
    f'<path d="M134,104 Q146,96 148,84" fill="none" stroke="{OUT}" stroke-width="12" stroke-linecap="round"/>',
    f'<path d="M134,104 Q146,96 148,84" fill="none" stroke="{RED}" stroke-width="5.5" stroke-linecap="round"/>',
    nub(149, 82, 6.5, 6),
    f'<g fill="none" stroke="{INK}" stroke-width="2.4" stroke-linecap="round" opacity="0.8"><path d="M160,76 q5,6 2,13"/><path d="M167,71 q8,9 3,20"/></g>',
    tomato(100, 100),
    nub(62, 112),
    star(40, 46, 7), star(28, 64, 4),
]))

poses["focus"] = svg("\n  ".join([
    tomato(100, 66, "down", "flat", 38, 32),
    nub(72, 117, 7.5, 5.5), nub(128, 117, 7.5, 5.5),
    f'<rect x="74" y="90" width="52" height="34" rx="5" fill="#dde3ea" stroke="{OUT}" stroke-width="{SW}"/>',
    heart(100, 106, 1.2),
    f'<rect x="12" y="124" width="176" height="11" rx="5.5" fill="#e8c6a0" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<rect x="68" y="121" width="64" height="5" rx="2.5" fill="#c7cfd9" stroke="{OUT}" stroke-width="3"/>',
    f'<path d="M26,122 L30,104 L46,96" fill="none" stroke="{OUT}" stroke-width="9" stroke-linecap="round" stroke-linejoin="round"/>',
    '<path d="M26,122 L30,104 L46,96" fill="none" stroke="#f4e4c9" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>',
    f'<ellipse cx="26" cy="122" rx="12" ry="4" fill="#f4e4c9" stroke="{OUT}" stroke-width="3"/>',
    f'<path d="M40,90 Q58,82 66,104 Z" fill="{ACC}" stroke="{OUT}" stroke-width="{SW}" stroke-linejoin="round"/>',
    f'<path d="M168,108 q9,0 9,6.5 q0,6.5 -9,6.5" fill="none" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<rect x="150" y="102" width="19" height="22" rx="4" fill="{ACC}" stroke="{OUT}" stroke-width="{SW}"/>',
    steam(156, 96),
]))

poses["short_break"] = svg("\n  ".join([
    shadow(110, 142, 60),
    nub(74, 138, 9, 5, RED_D), nub(102, 138, 9, 5, RED_D),
    tomato(88, 100, "happy", "smile"),
    # glass of juice with a straw
    f'<path d="M150,90 L160,70 L168,72" fill="none" stroke="{OUT}" stroke-width="7" stroke-linecap="round" stroke-linejoin="round"/>',
    '<path d="M150,90 L160,70 L168,72" fill="none" stroke="#ffffff" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"/>',
    f'<path d="M136,94 L139,130 Q139,134 143,134 L157,134 Q161,134 161,130 L164,94 Z" fill="#eef6fb" stroke="{OUT}" stroke-width="{SW}" stroke-linejoin="round"/>',
    f'<path d="M138,106 L162,106 L160,130 Q160,132 157,132 L143,132 Q141,132 141,130 Z" fill="{ACC}"/>',
    f'<circle cx="160" cy="96" r="7" fill="#ffd36b" stroke="{OUT}" stroke-width="2.6"/>',
    nub(133, 116, 6.5, 5.5), nub(167, 116, 6.5, 5.5),
]))

poses["long_break"] = svg("\n  ".join([
    shadow(100, 144, 56),
    # sleeping in a salad bowl
    tomato(100, 72, "sleep", "o", 38, 32),
    f'<path d="M50,104 Q60,96 70,104 Q80,94 92,104" fill="{LEAF}" stroke="{OUT}" stroke-width="3" stroke-linejoin="round"/>',
    f'<path d="M108,104 Q120,94 130,104 Q140,96 150,104" fill="{LEAF}" stroke="{OUT}" stroke-width="3" stroke-linejoin="round"/>',
    f'<path d="M42,104 L158,104 Q156,142 100,142 Q44,142 42,104 Z" fill="{ACC}" stroke="{OUT}" stroke-width="{SW}" stroke-linejoin="round"/>',
    f'<path d="M58,118 Q100,128 142,118" fill="none" stroke="#ffffff" stroke-width="3" opacity="0.4" stroke-linecap="round"/>',
    z(140, 56, 0.8), z(156, 36, 1.05), z(174, 12, 1.35),
]))


def main(out_dir):
    os.makedirs(out_dir, exist_ok=True)
    for name, text in poses.items():
        with open(os.path.join(out_dir, f"{name}.svg"), "w", encoding="utf8", newline="\n") as f:
            f.write(text)
    with open(os.path.join(out_dir, "character.toml"), "w", encoding="utf8", newline="\n") as f:
        f.write('name = "Tomate"\n')
    print("ok", list(poses))


if __name__ == "__main__":
    main(sys.argv[1])
