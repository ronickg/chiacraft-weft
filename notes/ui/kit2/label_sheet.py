"""The labels before and after, zoomed 4x, each with what `label_check` measured (Ronald: "Pls
be sure u get text correct position, vertically, sizing etc should all be perfect").

Usage: python3 label_sheet.py RUNS_DIR OUT.png [--scale6 DIR]"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kit2 as k  # noqa: E402
import label_check as lc  # noqa: E402

RUNS, OUT = sys.argv[1], sys.argv[2]
A_LANG = os.environ.get("A_LANG")

# (name, before shot, after shot, where the part is in each, roughly: x, y)
PAIRS = [
    ("Pause: a menu button", "t2_k2a_world/12_pause.png", (441, 219), "t2_val4_world/12_pause.png", (441, 219)),
    ("Animal: an order (icon and word)", "t2_k2a_pages/pages_overview.png", (823, 297), "t2_val4_pages/pages_overview.png", (823, 297)),
    ("Animal: Keep points", "t2_k2a_pages/pages_overview.png", (495, 450), "t2_val4_pages/pages_overview.png", (495, 450)),
    ("Animal: Undo (icon and word)", "t2_k2a_pages/pages_overview.png", (621, 450), "t2_val4_pages/pages_overview.png", (621, 450)),
    ("Animal: a stat's +", "t2_k2a_pages/pages_overview.png", (783, 247), "t2_val4_pages/pages_overview.png", (783, 247)),
    ("Recipes: the chosen category (before: the Valheim build without the text fixes)", "t2_val2_world/03_inventory.png", (520, 287),
     "t2_val4_world/03_inventory.png", (520, 287), "valheim"),
    ("Inventory: the open tab", "t2_k2a_world/03_inventory.png", (116, 165), "t2_val4_world/03_inventory.png", (116, 165)),
    ("Animal: a chip", "t2_k2a_pages/pages_overview.png", (259, 380), "t2_val4_pages/pages_overview.png", (259, 380)),
    ("Mods: the chosen row", "t2_k2a_menus/30_mods.png", (159, 128), "t2_val4_menus/30_mods.png", (159, 128)),
]


def find(shot, xy, palette, lang):
    lc.PALETTE = palette
    r = lc.measure(os.path.join(RUNS, shot), 2, lang or lc.LANG)
    best = min(r["parts"], key=lambda p: abs(p["box"][0] - xy[0]) + abs(p["box"][1] - xy[1]), default=None)
    # Only the part at that place: another one nearby is not the same part.
    if best and abs(best["box"][0] - xy[0]) + abs(best["box"][1] - xy[1]) > 24:
        return None
    return best


def crop(shot, p, z=4, margin=6):
    im = Image.open(os.path.join(RUNS, shot)).convert("RGB")
    x1, y1, x2, y2 = p["box"]
    box = (x1 - margin, y1 - margin, min(x2 + margin, x1 + 260), y2 + margin)
    c = im.crop(box)
    return c.resize((c.width * z, c.height * z), Image.NEAREST)


def text(img, s, x, y, scale=2, color=k.CREAM):
    t = k.canvas(k.F.width(s) + 4, k.F.line + 4)
    k.F.draw(t, s, 0, 2, color, (12, 8, 7))
    im = Image.fromarray(t).resize(((k.F.width(s) + 4) * scale, (k.F.line + 4) * scale), Image.NEAREST)
    img.paste(im, (x, y), im)


def verdict(p, left_aligned=False):
    v = "centred" if p["above"] == p["below"] else f"off by {abs(p['above'] - p['below'])}"
    if left_aligned:
        h = ", left-aligned as designed"
    else:
        h = "" if abs(p["left"] - p["right"]) <= 1 else f", {abs(p['left'] - p['right'])} px sideways"
    return f"above {p['above']}, below {p['below']}: {v}{h}"


rows = []
for name, bs, bxy, a_s, axy, *before_palette in PAIRS:
    b = find(bs, bxy, None, None) if before_palette else find(bs, bxy, "A", A_LANG)
    a = find(a_s, axy, None, None)
    if not a:
        print("not found:", name)
        continue
    rows.append((name, crop(bs, b) if b else None, b, crop(a_s, a), a))

W = 40 + 2 * max(max(r[1].width if r[1] else 0, r[3].width) for r in rows) + 60
H = 120 + sum(max(r[1].height if r[1] else 0, r[3].height) + 110 for r in rows)
sheet = Image.new("RGB", (W, H), (26, 19, 19))
text(sheet, "Labels, zoomed 4x: before (dev, tones A) and after (the Valheim look, text fixed). PC, GUI scale 2.", 20, 14, 3, k.GOLD_TEXT)
text(sheet, "Measured on the game's pixels: rows of face above the capitals and below the baseline, and the room each side.", 20, 66, 2, k.DIM)
y = 120
col = (W - 60) // 2
for name, bi, b, ai, a in rows:
    text(sheet, name, 20, y, 2, k.GOLD_TEXT)
    left = name.startswith("Mods")
    if bi:
        sheet.paste(bi, (20, y + 40))
    sheet.paste(ai, (40 + col, y + 40))
    hb = max(bi.height if bi else 0, ai.height)
    if b:
        text(sheet, "before: " + verdict(b, left), 20, y + 50 + hb, 2, (240, 150, 90) if b["above"] != b["below"] else k.DIM)
    else:
        text(sheet, "before: not found in that shot", 20, y + 50 + hb, 2, k.DIM)
    text(sheet, "after: " + verdict(a, left), 40 + col, y + 50 + hb, 2, k.GREEN if a["above"] == a["below"] else (240, 150, 90))
    y += hb + 110
sheet.crop((0, 0, W, y + 10)).save(OUT)
print(OUT, sheet.size)
