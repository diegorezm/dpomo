# Generates characters/axolotl/*.svg. Usage: python tools/gen_axolotl.py characters/axolotl
# Edit this file (not the SVGs) to tweak the axolotl, then preview with:
#   cargo run --example svg_preview -- preview.png f6ecdf 2 characters/axolotl/*.svg
import os, sys

OUT = "#3b2a2a"      # outline sentinel  -> theme text
ACC = "#e05a4f"      # accent sentinel   -> phase accent
INK = "#7a6464"      # loose marks (zzz, steam) -> theme muted
PINK = "#f7b6c2"
PINK_D = "#ef9fb1"
GILL = "#e7708f"
BELLY = "#fcd6de"
BLUSH = "#f2879e"
SW = 3.5

def head(eyes="open", mouth="smile"):
    g = []
    # gills (left), mirrored for the right side
    gills = f'''
      <ellipse cx="-44" cy="-19" rx="13" ry="5.5" transform="rotate(32 -44 -19)"/>
      <ellipse cx="-48" cy="-5" rx="13.5" ry="5.5" transform="rotate(6 -48 -5)"/>
      <ellipse cx="-45" cy="9" rx="12" ry="5" transform="rotate(-24 -45 9)"/>'''
    g.append(f'<g fill="{GILL}" stroke="{OUT}" stroke-width="{SW}">{gills}</g>')
    g.append(f'<g fill="{GILL}" stroke="{OUT}" stroke-width="{SW}" transform="scale(-1 1)">{gills}</g>')
    g.append(f'<ellipse cx="0" cy="0" rx="36" ry="28" fill="{PINK}" stroke="{OUT}" stroke-width="{SW}"/>')
    g.append('<ellipse cx="-14" cy="-15" rx="9" ry="4" fill="#ffffff" opacity="0.45"/>')
    g.append(f'<ellipse cx="-22" cy="10" rx="5.5" ry="3.2" fill="{BLUSH}" opacity="0.8"/>')
    g.append(f'<ellipse cx="22" cy="10" rx="5.5" ry="3.2" fill="{BLUSH}" opacity="0.8"/>')
    if eyes == "open":
        for x in (-16, 16):
            g.append(f'<circle cx="{x}" cy="2" r="3.8" fill="{OUT}"/>')
            g.append(f'<circle cx="{x+1.3}" cy="0.6" r="1.3" fill="#ffffff"/>')
    elif eyes == "happy":   # ^ ^
        for x in (-16, 16):
            g.append(f'<path d="M{x-4.5},4 Q{x},-2 {x+4.5},4" fill="none" stroke="{OUT}" stroke-width="2.8" stroke-linecap="round"/>')
    elif eyes == "sleep":   # u u
        for x in (-16, 16):
            g.append(f'<path d="M{x-4.5},1 Q{x},6 {x+4.5},1" fill="none" stroke="{OUT}" stroke-width="2.8" stroke-linecap="round"/>')
    elif eyes == "down":    # looking at the screen
        for x in (-16, 16):
            g.append(f'<circle cx="{x}" cy="5" r="3.6" fill="{OUT}"/>')
            g.append(f'<circle cx="{x+1.2}" cy="3.8" r="1.2" fill="#ffffff"/>')
    if mouth == "smile":
        g.append(f'<path d="M-5,11 Q-2.5,14 0,11 Q2.5,14 5,11" fill="none" stroke="{OUT}" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"/>')
    elif mouth == "o":
        g.append(f'<ellipse cx="0" cy="12" rx="2.2" ry="2.6" fill="{OUT}"/>')
    elif mouth == "flat":
        g.append(f'<path d="M-3.5,12 L3.5,12" stroke="{OUT}" stroke-width="2.4" stroke-linecap="round"/>')
    return "\n    ".join(g)

def svg(body, w=200, h=160):
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">
  <!-- dpomo character. {ACC} becomes the phase color and {INK} the theme's muted text color. -->
  {body}
</svg>
'''

def shadow(cx=100, cy=142, rx=48):
    return f'<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="6" fill="{OUT}" opacity="0.12"/>'

def tail():
    return (f'<path d="M124,128 C150,134 166,118 160,98 C152,110 142,118 122,114 Z" '
            f'fill="{PINK_D}" stroke="{OUT}" stroke-width="{SW}" stroke-linejoin="round"/>')

def sitting_body():
    return "\n  ".join([
        tail(),
        f'<ellipse cx="100" cy="118" rx="31" ry="24" fill="{PINK}" stroke="{OUT}" stroke-width="{SW}"/>',
        f'<ellipse cx="100" cy="124" rx="18" ry="14" fill="{BELLY}"/>',
        f'<ellipse cx="83" cy="140" rx="10" ry="5.5" fill="{PINK}" stroke="{OUT}" stroke-width="{SW}"/>',
        f'<ellipse cx="117" cy="140" rx="10" ry="5.5" fill="{PINK}" stroke="{OUT}" stroke-width="{SW}"/>',
    ])

def steam(x, y):
    return (f'<g fill="none" stroke="{INK}" stroke-width="2.4" stroke-linecap="round" opacity="0.7">'
            f'<path d="M{x},{y} q-4,-5 0,-10 q4,-5 0,-10"/>'
            f'<path d="M{x+8},{y+2} q-4,-5 0,-10 q4,-5 0,-10"/></g>')

def heart(x, y, s=1.0, fill=ACC):
    return (f'<path transform="translate({x} {y}) scale({s})" d="M0,4 C-7,-1 -5,-8 0,-4.5 C5,-8 7,-1 0,4 Z" '
            f'fill="{fill}" stroke="{OUT}" stroke-width="{2/s}" stroke-linejoin="round"/>')

def z(x, y, s):
    return (f'<path transform="translate({x} {y}) scale({s})" d="M0,0 H10 L0,12 H10" fill="none" '
            f'stroke="{INK}" stroke-width="{3/s}" stroke-linecap="round" stroke-linejoin="round"/>')

def star(x, y, r, fill=ACC):
    i = r * 0.38
    d = f"M{x},{y-r} Q{x+i*0.3},{y-i*0.3} {x+r},{y} Q{x+i*0.3},{y+i*0.3} {x},{y+r} Q{x-i*0.3},{y+i*0.3} {x-r},{y} Q{x-i*0.3},{y-i*0.3} {x},{y-r} Z"
    return f'<path d="{d}" fill="{fill}" stroke="{OUT}" stroke-width="2" stroke-linejoin="round"/>'

poses = {}

poses["focus"] = svg("\n  ".join([
    # lamp light
    # body + head behind the laptop
    f'<ellipse cx="100" cy="114" rx="32" ry="20" fill="{PINK}" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<g transform="translate(100 64)">\n    {head("down", "flat")}\n  </g>',
    # hands on the keyboard, peeking from the sides
    f'<ellipse cx="72" cy="117" rx="7.5" ry="5.5" fill="{PINK}" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<ellipse cx="128" cy="117" rx="7.5" ry="5.5" fill="{PINK}" stroke="{OUT}" stroke-width="{SW}"/>',
    # laptop (back of the lid)
    f'<rect x="74" y="90" width="52" height="34" rx="5" fill="#dde3ea" stroke="{OUT}" stroke-width="{SW}"/>',
    heart(100, 106, 1.2),
    # desk
    f'<rect x="12" y="124" width="176" height="11" rx="5.5" fill="#e8c6a0" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<rect x="68" y="121" width="64" height="5" rx="2.5" fill="#c7cfd9" stroke="{OUT}" stroke-width="3"/>',
    # lamp
    f'<path d="M26,122 L30,104 L46,96" fill="none" stroke="{OUT}" stroke-width="9" stroke-linecap="round" stroke-linejoin="round"/>',
    '<path d="M26,122 L30,104 L46,96" fill="none" stroke="#f4e4c9" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>',
    f'<ellipse cx="26" cy="122" rx="12" ry="4" fill="#f4e4c9" stroke="{OUT}" stroke-width="3"/>',
    f'<circle cx="30" cy="104" r="3" fill="{OUT}"/>',
    f'<path d="M40,90 Q58,82 66,104 Z" fill="{ACC}" stroke="{OUT}" stroke-width="{SW}" stroke-linejoin="round"/>',
    # mug
    f'<path d="M168,108 q9,0 9,6.5 q0,6.5 -9,6.5" fill="none" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<rect x="150" y="102" width="19" height="22" rx="4" fill="{ACC}" stroke="{OUT}" stroke-width="{SW}"/>',
    steam(156, 96),
]))

poses["short_break"] = svg("\n  ".join([
    shadow(),
    sitting_body(),
    f'<g transform="translate(100 68)">\n    {head("happy", "smile")}\n  </g>',
    # mug held with both hands
    f'<path d="M112,118 q8,0 8,6 q0,6 -8,6" fill="none" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<rect x="88" y="110" width="25" height="24" rx="5" fill="{ACC}" stroke="{OUT}" stroke-width="{SW}"/>',
    heart(100, 122, 0.9, "#ffffff"),
    f'<ellipse cx="86" cy="121" rx="7" ry="5.5" fill="{PINK}" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<ellipse cx="114" cy="124" rx="6" ry="5" fill="{PINK}" stroke="{OUT}" stroke-width="{SW}"/>',
]))

poses["long_break"] = svg("\n  ".join([
    shadow(104, 142, 70),
    # pillow
    f'<rect x="24" y="108" width="72" height="32" rx="15" fill="#f4e4c9" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<path d="M36,116 Q60,122 84,116" fill="none" stroke="{OUT}" stroke-width="2" opacity="0.3" stroke-linecap="round"/>',
    # tail fin sticking out of the blanket
    f'<path d="M160,124 C176,118 186,126 182,136 C174,134 166,136 158,134 Z" fill="{PINK_D}" stroke="{OUT}" stroke-width="{SW}" stroke-linejoin="round"/>',
    # body under a blanket
    f'<ellipse cx="122" cy="126" rx="42" ry="15" fill="{PINK}" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<path d="M86,140 L86,124 Q96,112 108,120 Q120,110 132,120 Q146,110 164,122 L166,140 Z" fill="{ACC}" stroke="{OUT}" stroke-width="{SW}" stroke-linejoin="round"/>',
    f'<path d="M100,138 L100,124 M118,138 L118,122 M136,138 L136,122 M152,138 L152,124" stroke="#ffffff" stroke-width="3" opacity="0.35" stroke-linecap="round"/>',
    f'<g transform="translate(66 102) rotate(-10)">\n    {head("sleep", "o")}\n  </g>',
    z(128, 66, 0.8), z(146, 46, 1.05), z(166, 22, 1.35),
]))

poses["idle"] = svg("\n  ".join([
    shadow(),
    sitting_body(),
    # waving arm (behind the head's gills is fine)
    f'<path d="M124,114 Q140,108 146,98" fill="none" stroke="{OUT}" stroke-width="13" stroke-linecap="round"/>',
    f'<path d="M124,114 Q140,108 146,98" fill="none" stroke="{PINK}" stroke-width="6" stroke-linecap="round"/>',
    f'<ellipse cx="147" cy="96" rx="7" ry="6" fill="{PINK}" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<g fill="none" stroke="{INK}" stroke-width="2.4" stroke-linecap="round" opacity="0.8"><path d="M158,90 q5,6 2,13"/><path d="M165,85 q8,9 3,20"/></g>',
    # resting arm
    f'<ellipse cx="78" cy="122" rx="7" ry="5.5" fill="{PINK}" stroke="{OUT}" stroke-width="{SW}"/>',
    f'<g transform="translate(100 70)">\n    {head("open", "smile")}\n  </g>',
    star(30, 26, 7), star(18, 44, 4),
]))

out_dir = sys.argv[1]
os.makedirs(out_dir, exist_ok=True)
for name, text in poses.items():
    with open(os.path.join(out_dir, f"{name}.svg"), "w", encoding="utf8", newline="\n") as f:
        f.write(text)
with open(os.path.join(out_dir, "character.toml"), "w", encoding="utf8", newline="\n") as f:
    f.write('name = "Axolote"\n')
print("ok", list(poses))
