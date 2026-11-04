"""Rounding options for Ronald ("can we make it more rounded less just square ?"): the same
button, tab, panel, bar, slot and chip at 1 px, 2 px, 3 px and pill corners, then the mix I
recommend as one small window. Written at 1x (GUI scale 2) and 2x (GUI scale 4).
Usage: python3 rounding_sheet.py OUT_DIR"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kit2 as k  # noqa: E402

OUT = sys.argv[1]
BG = (26, 19, 19)
W, H = 1060, 760
S = k.canvas(W, H, BG)
STAT = os.path.join(k.REPO, "mods/base/assets/textures/gui")


def at(img, x, y):
    assert x % 2 == 0 and y % 2 == 0, f"off the 2 px rhythm at {x},{y}"
    h, w = img.shape[:2]
    assert x + w <= W and y + h <= H, f"off the sheet at {x},{y}"
    k.blit(S, img, x, y)
    return w


def text(s, x, y, color=k.CREAM):
    return k.F.draw(S, s, x, y, color, (12, 8, 7))


COLS = [("Button", 16), ("Tab on a window", 150), ("Bar", 380), ("Slot", 510), ("Chip", 570), ("+ and -", 820)]
y = 16
text("Rounding: the same parts, four ways. One 1 px outline; corners step on the pixel grid.", 16, y, k.GOLD_TEXT)
y += 22
for name, x in COLS:
    text(name, x, y, k.DIM)
y += 20
options = [("A  1 px off each corner", 1, 1, 1, 1, 1, 1),
           ("B  2 px steps", 2, 2, 2, 2, 2, 2),
           ("C  3 px steps", 3, 3, 3, 3, 3, 3),
           ("D  pills and round", "pill", 3, "pill", "pill", 2, "pill")]
for title, rb, rt, rbar, rchip, rslot, rround in options:
    text(title, 16, y, k.GOLD_TEXT)
    y += 18
    at(k.button_r(100, 22, "main", "normal", "Craft", r=rb), 16, y)
    at(k.button_r(100, 22, "second", "hover", "Back", r=rb), 16, y + 26)
    # a tab on a bit of window
    p = k.panel_r(200, 46, rt if rt != "pill" else 3)
    at(p, 150, y + 20)
    at(k.tab_r(96, "Overview", True, "follow", r=rt if rt != "pill" else 3), 156, y)
    at(k.tab_r(76, "Genes", False, "taming", r=rt if rt != "pill" else 3), 254, y)
    at(k.bar_r(110, 0.7, "health", r=rbar), 380, y + 6)
    at(k.bar_r(110, 0.35, "q2", r=rbar), 380, y + 22)
    at(k.slot_r(36, False, r=rslot), 510, y + 4)
    cx = 570
    for t, kind in (("Adult", "plain"), ("Steady", "plain"), ("2 to place", "points")):
        cx += at(k.chip_r(t, kind, r=rchip), cx, y + 2) + 4
    cx = 570
    if rround == "pill":
        at(k.round_button(20, "main", "normal", "plus"), 820, y + 4)
        at(k.round_button(20, "second", "normal", "minus"), 844, y + 4)
        at(k.round_button(20, "main", "hover", "plus"), 868, y + 4)
    else:
        at(k.button_r(20, 20, "main", "normal", "", "plus", r=rround, pad=2), 820, y + 4)
        at(k.button_r(20, 20, "second", "normal", "", "minus", r=rround, pad=2), 844, y + 4)
        at(k.button_r(20, 20, "main", "hover", "", "plus", r=rround, pad=2), 868, y + 4)
    y += 76

# ---------------------------------------------------------------- the recommended mix
text("Recommended: 2 px on buttons, tabs and slots; 3 px on windows; pills on bars and chips; round + and -.",
     16, y, k.GOLD_TEXT)
y += 22
wx, wy = 16, y
tx = wx + 8
for t, ic, act in (("Overview", "follow", True), ("Genes", "taming", False), ("Family", "tribe", False)):
    w = k.F.width(t) + 12 + 16 + 4 + 2
    w += w % 2
    at(k.tab_r(w, t, act, ic, r=2), tx, wy)
    tx += w + 2
pw, ph = 680, 204
at(k.panel_r(pw, ph, 3), wx, wy + 20)
px, py = wx + 12, wy + 32
text("Stats", px, py, k.GOLD_TEXT)
at(k.chip_r("2 to place", "points"), px + 44, py - 4)
rows = (("health", "33.9", "q1", 0.55, "8", "+2"), ("stamina", "207", "q2", 0.6, "12", "0"),
        ("food", "140", "q0", 0.5, "5", "+1"), ("speed", "94 %", "q0", 0.4, "1", "0"))
yy = py + 22
text("Wild", px + 252, yy, k.DIM)
text("Tamed", px + 290, yy, k.DIM)
yy += 18
for name, val, q, share, wild, tamed in rows:
    at(np.array(Image.open(os.path.join(STAT, f"stat_icon_{name}.png")).convert("RGBA")), px, yy)
    text(name.capitalize(), px + 22, yy + 4)
    at(k.bar_r(100, share, q), px + 104, yy + 4)
    green = tamed == "+1"
    k.F.draw(S, val, px + 246 - k.F.width(val), yy + 4, k.GREEN if green else k.CREAM, (12, 8, 7))
    k.F.draw(S, wild, px + 264 - k.F.width(wild) // 2, yy + 4, k.BAR_FILLS[q][1], (12, 8, 7))
    k.F.draw(S, tamed, px + 306 - k.F.width(tamed) // 2, yy + 4, k.GREEN if green else k.CREAM, (12, 8, 7))
    at(k.round_button(20, "main", "hover" if name == "food" else "normal", "plus"), px + 330, yy - 2)
    yy += 26
at(k.button_r(120, 22, "main", "normal", "Keep 1 point", r=2), px, yy + 4)
at(k.button_r(84, 22, "second", "normal", "Undo", "undo", r=2), px + 126, yy + 4)
# right half of the window: orders and slots
ox = px + 370
text("Orders", ox, py, k.GOLD_TEXT)
for i, (t, ic, kind, st) in enumerate((("Follow", "follow", "main", "pressed"), ("Stay", "stay", "second", "normal"),
                                       ("Wander", "wander", "second", "normal"), ("Guard", "guard", "second", "hover"))):
    at(k.button_r(110, 22, kind, st, t, ic, r=2, align="left"), ox + (i % 2) * 116, py + 20 + (i // 2) * 26)
text("Food it carries", ox, py + 80, k.GOLD_TEXT)
for i in range(5):
    at(k.slot_r(36, i == 1, r=2), ox + i * 40, py + 98)
item = np.array(Image.open(os.path.join(k.REPO, "mods/base/assets/textures/item/beef.png")).convert("RGBA"))
at(item, ox + 2, py + 100)
cx = ox
for t in ("Adult, 22 days", "Steady", "Pack hunter"):
    cx += at(k.chip_r(t), cx, py + 142) + 4
# nothing outside its window: the frame is 6 px, the content keeps 6 px more
assert cx - 4 <= wx + pw - 12, f"chips run out of the window: {cx - 4} > {wx + pw - 12}"
assert ox + 116 + 110 <= wx + pw - 12, "orders run out of the window"
y = wy + 20 + ph + 12

out = S[:y]
os.makedirs(OUT, exist_ok=True)
one = Image.fromarray(out)
one.save(os.path.join(OUT, "rounding_1x.png"))
one.resize((one.width * 2, one.height * 2), Image.NEAREST).save(os.path.join(OUT, "rounding_2x.png"))
print(os.path.join(OUT, "rounding_2x.png"), one.size)
