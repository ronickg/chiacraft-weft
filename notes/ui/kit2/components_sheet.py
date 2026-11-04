"""The component sheet for Ronald: every shared UI part from `kit2.py`, in its states, laid out on
a 2 px rhythm, written at 1x (1 UI px = 1 screen px: GUI scale 2) and 2x (GUI scale 4).
Usage: python3 components_sheet.py OUT_DIR [OLD_SHEET.png]"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kit2 as k  # noqa: E402

OUT = sys.argv[1]
OLD = sys.argv[2] if len(sys.argv) > 2 else None
BG = (26, 19, 19)
W, H = 1340, 820
S = k.canvas(W, H, BG)
PLACED = []


def at(img, x, y):
    """Places a part; every position sits on the 2 px rhythm."""
    assert x % 2 == 0 and y % 2 == 0, f"off the 2 px rhythm at {x},{y}"
    k.blit(S, img, x, y)
    PLACED.append((x, y, img.shape[1], img.shape[0]))
    return img.shape[1]


def text(s, x, y, color=k.CREAM):
    """Text on the page, its capitals' top at `y`, with the page's shadow."""
    return k.F.draw(S, s, x, y, color, (12, 8, 7))


def heading(s, x, y):
    text(s, x, y, k.GOLD_TEXT)


# ---------------------------------------------------------------- 1. icons
x0, y = 16, 16
heading("Icons: 16 x 16, one outline, light from the top left, one palette", x0, y)
names = ["follow", "stay", "wander", "guard", "passive", "neutral", "aggressive", "assist", "ride", "tribe",
         "sniff", "release", "undo", "rename", "male", "female", "taming", "close", "plus", "minus", "check"]
labels = {"follow": "Follow", "stay": "Stay", "wander": "Wander", "guard": "Guard", "passive": "Passive",
          "neutral": "Neutral", "aggressive": "Attack", "assist": "Assist", "ride": "Ride", "tribe": "Tribe",
          "sniff": "Sniff", "release": "Release", "undo": "Undo", "rename": "Rename", "male": "Male",
          "female": "Female", "taming": "Taming", "close": "Close", "plus": "Plus", "minus": "Minus", "check": "Done"}
y += 20
for i, n in enumerate(names):
    cx = x0 + i * 62
    at(k.slot(24), cx + 18, y)
    at(k.icon(n), cx + 22, y + 4)
    lw = k.F.width(labels[n])
    k.F.draw(S, labels[n], cx + 30 - lw // 2, y + 30, k.DIM, (12, 8, 7))
# the same on a light ground: the outline must hold there too
y += 46
light = k.canvas(len(names) * 62 - 6, 24, (214, 196, 170))
at(light, x0 + 10, y)
for i, n in enumerate(names):
    at(k.icon(n), x0 + i * 62 + 22, y + 4)
y += 34

# ---------------------------------------------------------------- 2. buttons
heading("Buttons: normal, hover, pressed, disabled", x0, y)
y += 20
for row, kind in enumerate(("main", "second")):
    for col, state in enumerate(("normal", "hover", "pressed", "disabled")):
        at(k.button(96, 22, kind, state, "Craft" if kind == "main" else "Back"), x0 + col * 104, y + row * 28)
at(k.button(96, 22, "danger", "normal", "Release", "release"), x0 + 4 * 104, y)
at(k.button(96, 22, "second", "normal", "Undo", "undo"), x0 + 4 * 104, y + 28)
yb = y + 62
heading("A toggle group: Neutral chosen (pressed, orange), Attack under the pointer", x0, yb)
yb += 20
group = (("Passive", "passive", "second", "normal"), ("Neutral", "neutral", "main", "pressed"),
         ("Attack", "aggressive", "second", "hover"), ("Assist", "assist", "second", "normal"))
for i, (t, ic, kind, st) in enumerate(group):
    at(k.button(100, 22, kind, st, t, ic, align="left"), x0 + (i % 2) * 104, yb + (i // 2) * 26)
orders = (("Follow", "follow", "main", "pressed"), ("Stay", "stay", "second", "normal"),
          ("Wander", "wander", "second", "normal"), ("Guard", "guard", "second", "normal"))
for i, (t, ic, kind, st) in enumerate(orders):
    at(k.button(100, 22, kind, st, t, ic, align="left"), x0 + 216 + (i % 2) * 104, yb + (i // 2) * 26)
at(k.button(100, 22, "second", "disabled", "Ride", "ride", align="left"), x0 + 432, yb)
text("Ride: needs a saddle on it", x0 + 432, yb + 32, k.DIM)
y = yb + 60

# ---------------------------------------------------------------- 3. tabs on a panel
heading("Tabs and a window", x0, y)
y += 20
p = k.panel(560, 200)
tx = x0 + 8
for t, ic, act in (("Overview", "follow", True), ("Genes", "taming", False), ("Family", "tribe", False)):
    w = k.F.width(t) + 16 + 4 + 16
    w += w % 2
    at(k.tab(w, t, act, ic), tx, y)
    tx += w + 2
at(p, x0, y + 20)
px, py = x0 + k.INNER + 6, y + 20 + k.INNER + 6
heading("Stats", px, py)
at(k.chip("2 to place", "points"), px + 44, py - 4)
rows = (("health", "33.9", "q1", 0.55, "8", "+2"), ("stamina", "207", "q2", 0.6, "12", "0"),
        ("food", "140", "q0", 0.5, "5", "+1"), ("speed", "94 %", "q0", 0.4, "1", "0"))
yy = py + 24
text("Wild", px + 252, yy, k.DIM)
text("Tamed", px + 290, yy, k.DIM)
yy += 18
stat_icons = os.path.join(k.REPO, "mods/base/assets/textures/gui")
for name, val, q, share, wild, tamed in rows:
    ic = np.array(Image.open(os.path.join(stat_icons, f"stat_icon_{name}.png")).convert("RGBA"))
    at(ic, px, yy)
    text(name.capitalize(), px + 22, yy + 4)
    at(k.bar(100, share, q), px + 104, yy + 4)
    vw = k.F.width(val)
    k.F.draw(S, val, px + 246 - vw, yy + 4, k.GREEN if tamed == "+1" else k.CREAM, (12, 8, 7))
    ww = k.F.width(wild)
    k.F.draw(S, wild, px + 264 - ww // 2, yy + 4, k.BAR_FILLS[q][1], (12, 8, 7))
    tw = k.F.width(tamed)
    k.F.draw(S, tamed, px + 306 - tw // 2, yy + 4, k.GREEN if tamed == "+1" else k.CREAM, (12, 8, 7))
    at(k.button(20, 20, "main", "hover" if name == "food" else "normal", "", "plus", pad=2), px + 330, yy - 2)
    yy += 26
at(k.divider(360), px, yy)
yy += 8
at(k.button(120, 22, "main", "normal", "Keep 1 point"), px, yy)
at(k.button(84, 22, "second", "normal", "Undo", "undo"), px + 126, yy)
# right of the window: bars, slots, chips, keys
rx = x0 + 600
by = y
heading("Bars: the value beside, never on it", rx, by)
by += 20
for fill, share, val in (("health", 0.97, "33 / 34"), ("food", 0.98, "132 / 134"), ("water", 1.0, "100 / 100"),
                         ("taming", 0.68, "68 %"), ("torpor", 0.8, "2:40"), ("xp", 0.74, "108 / 146")):
    at(k.bar(160, share, fill), rx, by + 2)
    text("XP" if fill == "xp" else fill.capitalize(), rx + 168, by + 2, k.DIM)
    vw = k.F.width(val)
    text(val, rx + 300 - vw, by + 2)
    by += 18
for i in range(5):
    at(k.bar(56, 0.3 + i * 0.15, f"q{i}"), rx + i * 64, by + 2)
text("Wild points: grey few, then green, blue, purple, gold", rx, by + 18, k.DIM)
by += 38
heading("Slots, chips, keys", rx, by)
by += 20
at(k.slot(36), rx, by)
at(k.slot(36, True), rx + 42, by)
s3 = k.slot(36)
item = os.path.join(k.REPO, "mods/base/assets/textures/item/beef.png")
at(s3, rx + 84, by)
at(np.array(Image.open(item).convert("RGBA")), rx + 86, by + 2)
cx = rx + 130
for t, kind in (("Adult, 22 days", "plain"), ("Steady", "plain"), ("4 to place", "points")):
    cx += at(k.chip(t, kind), cx, by) + 4
cx = rx + 130
for i, (key, ic) in enumerate((("J", "follow"), ("K", "stay"), ("H", "wander"), ("G", "assist"), ("B", "passive"))):
    at(k.keycap(key), cx + i * 40, by + 22)
    at(k.icon(ic), cx + i * 40 + 20, by + 22)
by += 50
heading("Text: one size; the shadow 1 px down, a tone under its ground", rx, by)
by += 20
row_bg = k.canvas(300, 40, k.PANEL)
at(row_bg, rx, by)
k.F.draw(S, "Before: 1 px down-right, near black", rx + 6, by + 6, k.CREAM, k.OUTLINE, shadow_dx=1)
for xx in range(rx + 6, rx + 290):
    pass
k.F.draw(S, "After: 1 px down, panel's dark", rx + 6, by + 24, k.CREAM, k.PANEL_DEEP)
y = max(yy + 30, by + 50) - 40

# ---------------------------------------------------------------- 4. before and after
if OLD:
    heading("Before (the fixed round-2 sheet, Stance) and after", x0, y)
    old = Image.open(OLD).convert("RGBA")
    # Stance block on the 2x sheet: crop and halve back to 1 UI px.
    crop = old.crop((1186, 344, 1574, 484)).resize((194, 70), Image.NEAREST)
    at(np.array(crop), x0, y + 20)
    for i, (t, ic, kind, st) in enumerate(group):
        at(k.button(100, 22, kind, st, t, ic, align="left"), x0 + 220 + (i % 2) * 104, y + 24 + (i // 2) * 26)
    y += 96

S2 = S[:y + 10]
os.makedirs(OUT, exist_ok=True)
one = Image.fromarray(S2)
one.save(os.path.join(OUT, "components_1x.png"))
one.resize((one.width * 2, one.height * 2), Image.NEAREST).save(os.path.join(OUT, "components_2x.png"))
print(os.path.join(OUT, "components_1x.png"), one.size)
