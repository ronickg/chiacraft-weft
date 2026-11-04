"""The labels before and after, zoomed 4x, each with what `label_check` measured (Ronald: "Pls
be sure u get text correct position, vertically, sizing etc should all be perfect").

Usage: python3 label_sheet.py RUNS_DIR OUT.png [--scale6 TESTER_DIR]

RUNS_DIR holds the PC runs (t2_<id>/), TESTER_DIR Tester's Mac runs (<name>_<time>/run/screenshots/).
With --scale6, the sheet has a second part: the same labels at GUI scale 6 (3456 x 2234)."""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kit2 as k  # noqa: E402
import label_check as lc  # noqa: E402

RUNS, OUT = sys.argv[1], sys.argv[2]
TESTER = sys.argv[sys.argv.index("--scale6") + 1] if "--scale6" in sys.argv else None
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

# At GUI scale 6, from Tester's runs: before (dev, tones A, auto scale) and after (the Valheim
# look, text fixed); places in UI pixels (3 screen pixels each).
B6, A6 = "kit2_auto2_pages_1104-093431", "val_r6b_pages_1104-104701"
BF6, AF6 = "kit2_auto2_fit_1104-093413", "val_r6b_fit_1104-104729"
PAIRS6 = [
    ("Animal: an order (icon and word)", f"{B6}/run/screenshots/pages_overview.png", (759, 310), f"{A6}/run/screenshots/pages_overview.png", (759, 310)),
    ("Animal: an order with a descender", f"{B6}/run/screenshots/pages_overview.png", (759, 338), f"{A6}/run/screenshots/pages_overview.png", (759, 338)),
    ("Animal: Keep points", f"{B6}/run/screenshots/pages_overview.png", (431, 463), f"{A6}/run/screenshots/pages_overview.png", (431, 463)),
    ("Animal: Undo (icon and word)", f"{B6}/run/screenshots/pages_overview.png", (557, 463), f"{A6}/run/screenshots/pages_overview.png", (557, 463)),
    ("Animal: a stat's +", f"{B6}/run/screenshots/pages_overview.png", (719, 260), f"{A6}/run/screenshots/pages_overview.png", (719, 260)),
    ("Animal: a chip", f"{B6}/run/screenshots/pages_overview.png", (195, 393), f"{A6}/run/screenshots/pages_overview.png", (195, 393)),
    ("Animal: the open tab", f"{B6}/run/screenshots/pages_overview.png", (195, 137), f"{A6}/run/screenshots/pages_overview.png", (195, 137)),
    ("Title: Singleplayer (Tester's \"4 px low\": one pixel of the figure counted as text)", f"{BF6}/run/screenshots/fit_title.png", (25, 418), f"{AF6}/run/screenshots/fit_title.png", (25, 418)),
    ("Title: Multiplayer", f"{BF6}/run/screenshots/fit_title.png", (25, 466), f"{AF6}/run/screenshots/fit_title.png", (25, 466)),
]


def find(path, xy, palette, lang, scale=2):
    lc.PALETTE = palette
    r = lc.measure(path, scale, lang or lc.LANG)
    best = min(r["parts"], key=lambda p: abs(p["box"][0] - xy[0]) + abs(p["box"][1] - xy[1]), default=None)
    # Only the part at that place: another one nearby is not the same part.
    if best and abs(best["box"][0] - xy[0]) + abs(best["box"][1] - xy[1]) > 24:
        return None
    return best


def crop(path, p, scale=2, z=4, margin=6):
    """The part at one UI pixel per pixel (a scale 6 shot has 3 x 3 screen pixels to one), zoomed z."""
    a = np.array(Image.open(path).convert("RGB"))
    n = max(1, scale // 2)
    im = Image.fromarray(a[::n, ::n])
    x1, y1, x2, y2 = p["box"]
    c = im.crop((x1 - margin, y1 - margin, min(x2 + margin, x1 + 260), y2 + margin))
    return c.resize((c.width * z, c.height * z), Image.NEAREST)


def text(img, s, x, y, scale=2, color=k.CREAM):
    t = k.canvas(k.F.width(s) + 4, k.F.line + 4)
    k.F.draw(t, s, 0, 2, color, (12, 8, 7))
    im = Image.fromarray(t).resize(((k.F.width(s) + 4) * scale, (k.F.line + 4) * scale), Image.NEAREST)
    img.paste(im, (x, y), im)


def verdict(p, left_aligned=False):
    v = "centred" if p["above"] == p["below"] else f"off by {abs(p['above'] - p['below'])}"
    if left_aligned == "icon":
        h = ", the icon at the left edge as designed"
    elif left_aligned:
        h = ", left-aligned as designed"
    else:
        h = "" if abs(p["left"] - p["right"]) <= 1 else f", {abs(p['left'] - p['right'])} px sideways"
    return f"above {p['above']}, below {p['below']}: {v}{h}"


def measured(pairs, root, scale):
    rows = []
    for name, bs, bxy, a_s, axy, *before_palette in pairs:
        bp, ap = os.path.join(root, bs), os.path.join(root, a_s)
        b = find(bp, bxy, None, None, scale) if before_palette else find(bp, bxy, "A", A_LANG, scale)
        a = find(ap, axy, None, None, scale)
        if not a:
            print("not found:", name)
            continue
        rows.append((name, crop(bp, b, scale) if b else None, b, crop(ap, a, scale), a))
    return rows


parts = [("Labels, zoomed 4x: before (dev, tones A) and after (the Valheim look, text fixed). PC, GUI scale 2 (1280 x 720).",
          "Measured on the game's pixels: rows of face above the capitals and below the baseline, and the room each side.",
          measured(PAIRS, RUNS, 2))]
if TESTER:
    parts.append(("The same at GUI scale 6: Ronald's Mac (3456 x 2234), Tester's runs. One UI pixel is 3 x 3 screen pixels.",
                  "Before: dev, tones A, auto scale. After: the Valheim look (83863490).",
                  measured(PAIRS6, TESTER, 6)))

every = [r for _, _, rows in parts for r in rows]
W = max(1900, 40 + 2 * max(max(r[1].width if r[1] else 0, r[3].width) for r in every) + 60)
H = sum(130 + sum(max(r[1].height if r[1] else 0, r[3].height) + 110 for r in rows) for _, _, rows in parts)
sheet = Image.new("RGB", (W, H), (26, 19, 19))
col = (W - 60) // 2
y = 0
for title, sub, rows in parts:
    text(sheet, title, 20, y + 14, 3 if len(title) < 105 else 2, k.GOLD_TEXT)
    text(sheet, sub, 20, y + 66, 2, k.DIM)
    y += 120
    for name, bi, b, ai, a in rows:
        text(sheet, name, 20, y, 2, k.GOLD_TEXT)
        left = "icon" if name.startswith("Title") else name.startswith("Mods")
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
    y += 10
sheet.crop((0, 0, W, y)).save(OUT)
print(OUT, sheet.size)
