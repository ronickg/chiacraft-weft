"""Ten brown looks for Ronald to choose from ("I do kind of want that brown sort of look ... give me
like 10 variants"), each rendered on the same in-game shots. Kit 2 draws exact colours, so every
part's colour (panel, frame, slots, wells, buttons, chips, accent, text) is swapped for the
variant's, and a variant can lay a pixel texture (wood grain, leather, stone, parchment) on its
panel faces. Every variant is measured: text against its panel, slots against the panel, button
text against its face.

Usage: python3 variants.py SHOTS_DIR OUT_DIR"""
import colorsys
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kit2 as k  # noqa: E402

# Read only when run; label_check imports this for tones A.
SHOTS, OUT = (sys.argv[1], sys.argv[2]) if __name__ == "__main__" else (None, None)

# Kit 2's colours with tones A (what the shots were taken with), frozen here so the sheet stays
# reproducible after kit 2 takes a variant's palette.
A = dict(PANEL=(29, 25, 24), PANEL_DEEP=(19, 16, 15), BRONZE_D=(92, 46, 20), BRONZE=(137, 69, 30), BRONZE_HI=(205, 120, 60), OUTLINE=(40, 24, 12), SLOT=(52, 44, 40), SLOT_D=(40, 33, 30), SLOT_L=(76, 64, 57), WELL=(44, 37, 34), WELL_D=(33, 28, 26), WELL_L=(64, 54, 49), SLIDER=(74, 56, 48), SLIDER_D=(56, 42, 36), SLIDER_L=(98, 76, 64), GLOW=(255, 221, 32), GLOW_D=(243, 171, 31),
         FACES={'main': ((236, 150, 64), (200, 108, 22), (138, 70, 12)), 'main_hover': ((248, 176, 92), (222, 128, 34), (150, 78, 14)), 'main_pressed': ((150, 78, 14), (170, 88, 16), (200, 108, 22)), 'second': ((122, 82, 56), (90, 58, 38), (58, 36, 24)), 'second_hover': ((146, 100, 68), (110, 72, 46), (66, 42, 28)), 'second_pressed': ((58, 36, 24), (70, 46, 30), (90, 58, 38)), 'disabled': ((86, 78, 72), (70, 63, 58), (54, 48, 44)), 'danger': ((222, 90, 70), (176, 44, 34), (112, 24, 18))},
         CHIPS={'plain': ((122, 82, 56), (52, 38, 30), (72, 50, 36)), 'points': ((220, 130, 60), (150, 80, 24), (110, 56, 16)), 'dim': ((86, 62, 46), (36, 27, 22), (50, 36, 28))})

# ---------------------------------------------------------------- the colours kit 2 draws now
NOW = {
    "panel": A["PANEL"], "groove": A["PANEL_DEEP"], "frame_d": A["BRONZE_D"], "frame": A["BRONZE"], "frame_hi": A["BRONZE_HI"],
    "outline": A["OUTLINE"], "slot": A["SLOT"], "slot_d": A["SLOT_D"], "slot_l": A["SLOT_L"],
    "well": A["WELL"], "well_d": A["WELL_D"], "well_l": A["WELL_L"],
    "slider": A["SLIDER"], "slider_d": A["SLIDER_D"], "slider_l": A["SLIDER_L"],
    "glow": A["GLOW"], "glow_d": A["GLOW_D"],
    # text, as the game draws it (one level over the theme's colours)
    "text": (229, 219, 208), "dim": (199, 164, 132), "button_text": (244, 235, 217), "title": (242, 219, 155),
    "on_orange": (255, 244, 220),
}
for name, (l, f, s) in A["FACES"].items():
    NOW[f"{name}_l"], NOW[f"{name}_f"], NOW[f"{name}_s"] = l, f, s
for name, (l, f, s) in A["CHIPS"].items():
    NOW[f"chip_{name}_l"], NOW[f"chip_{name}_f"], NOW[f"chip_{name}_s"] = l, f, s


def hx(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def shade(c, f):
    """Lighter (f > 1) or darker (f < 1) in value, a touch warmer when darker (hue toward red)."""
    r, g, b = (v / 255 for v in c)
    h, s, v = colorsys.rgb_to_hsv(r, g, b)
    v = max(0.0, min(1.0, v * f))
    if f < 1:
        s = min(1.0, s * (1 + (1 - f) * 0.25))
    else:
        s = s * (1 - min(0.3, (f - 1) * 0.25))
    return tuple(int(round(x * 255)) for x in colorsys.hsv_to_rgb(h, s, v))


def grey(c, keep=0.25):
    r, g, b = c
    m = (r * 0.3 + g * 0.59 + b * 0.11)
    return tuple(int(m + (v - m) * keep) for v in c)


def palette(v):
    """Every role's colour from a variant's few base colours."""
    p = {}
    p["panel"], p["groove"] = v["panel"], shade(v["panel"], 0.62)
    p["frame"], p["frame_d"], p["frame_hi"] = v["frame"], shade(v["frame"], 0.66), shade(v["frame"], 1.45)
    p["outline"] = v.get("outline", shade(v["frame"], 0.28))
    p["slot"], p["slot_d"], p["slot_l"] = v["slot"], shade(v["slot"], 0.78), shade(v["slot"], 1.42)
    well = v.get("well", tuple((a + b) // 2 for a, b in zip(v["panel"], v["slot"])))
    p["well"], p["well_d"], p["well_l"] = well, shade(well, 0.76), shade(well, 1.4)
    sl = v.get("slider", shade(v["slot"], 1.4))
    p["slider"], p["slider_d"], p["slider_l"] = sl, shade(sl, 0.76), shade(sl, 1.3)
    p["glow"], p["glow_d"] = v["accent"], shade(v["accent"], 0.85)
    for name, base in (("main", v["main"]), ("second", v["second"]), ("danger", v.get("danger", hx("#b02c22")))):
        p[f"{name}_l"], p[f"{name}_f"], p[f"{name}_s"] = shade(base, 1.18), base, shade(base, 0.68)
        hov = shade(base, 1.12)
        p[f"{name}_hover_l"], p[f"{name}_hover_f"], p[f"{name}_hover_s"] = shade(hov, 1.15), hov, shade(base, 0.72)
        p[f"{name}_pressed_l"], p[f"{name}_pressed_f"], p[f"{name}_pressed_s"] = shade(base, 0.7), shade(base, 0.85), base
    d = grey(v["second"])
    p["disabled_l"], p["disabled_f"], p["disabled_s"] = shade(d, 1.2), d, shade(d, 0.75)
    chip = v.get("chip", shade(v["slot"], 1.1))
    for name, base in (("plain", chip), ("points", v["main"]), ("dim", shade(chip, 0.8))):
        p[f"chip_{name}_l"], p[f"chip_{name}_f"], p[f"chip_{name}_s"] = shade(base, 1.6), base, shade(base, 1.25)
    p["text"], p["dim"], p["title"] = v["text"], v["dim"], v["title"]
    p["button_text"] = v.get("button_text", v["text"])
    p["on_orange"] = v.get("on_main", p["button_text"])
    return p


# ---------------------------------------------------------------- textures, at the UI's pixel
def noise(h, w, seed, cell):
    rng = np.random.default_rng(seed)
    g = rng.random((h // cell + 2, w // cell + 2))
    return np.kron(g, np.ones((cell, cell)))[:h, :w]


def texture(kind, h, w):
    """-1, 0 or 1 per pixel: a darker, the base or a lighter tone of the panel."""
    y, x = np.mgrid[0:h, 0:w]
    if kind == "wood":
        # long grain along x: bands that wander a little, with a few knots of darker streaks
        n = noise(h, w, 3, 16)
        bands = np.sin((y + n * 6) * 0.55 + np.sin(x * 0.02) * 2)
        t = np.where(bands > 0.86, 1, np.where(bands < -0.9, -1, 0))
        streak = (noise(h, w, 5, 1) > 0.985)
        return np.where(streak, -1, t)
    if kind == "leather":
        n1, n2 = noise(h, w, 7, 2), noise(h, w, 8, 1)
        return np.where(n1 + n2 * 0.4 > 1.18, 1, np.where(n1 + n2 * 0.4 < 0.22, -1, 0))
    if kind == "stone":
        n = noise(h, w, 11, 6) + noise(h, w, 12, 2) * 0.35
        return np.where(n > 1.08, 1, np.where(n < 0.28, -1, 0))
    if kind == "parchment":
        n = noise(h, w, 13, 10) * 0.7 + noise(h, w, 14, 3) * 0.3
        return np.where(n > 0.8, -1, 0)
    return np.zeros((h, w), int)


def apply(shot, v):
    p = palette(v)
    out = shot.copy()
    masks = {name: np.all(shot == np.array(c), axis=-1) for name, c in NOW.items()}
    for name, m in masks.items():
        if name in p:
            out[m] = p[name]
    if v.get("texture"):
        t = texture(v["texture"], *shot.shape[:2])
        m = masks["panel"]
        out[m & (t == 1)] = shade(p["panel"], v.get("tex_light", 1.14))
        out[m & (t == -1)] = shade(p["panel"], v.get("tex_dark", 0.86))
    return out, p


# ---------------------------------------------------------------- the checklist, measured
def lum(c):
    def ch(v):
        v /= 255
        return v / 12.92 if v <= 0.03928 else ((v + 0.055) / 1.055) ** 2.4
    r, g, b = (ch(x) for x in c)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast(a, b):
    la, lb = sorted((lum(a), lum(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


def check(p):
    """What the checklist measures, and what it finds."""
    rows = [("text on panel", contrast(p["text"], p["panel"]), 4.5),
            ("dim text on panel", contrast(p["dim"], p["panel"]), 3.0),
            ("title on panel", contrast(p["title"], p["panel"]), 3.0),
            ("button text on main", contrast(p["on_orange"], p["main_f"]), 3.0),
            ("text on second button", contrast(p["button_text"], p["second_f"]), 3.0),
            ("slot against panel", contrast(p["slot"], p["panel"]), 1.15),
            ("frame against panel", contrast(p["frame"], p["panel"]), 1.5)]
    return [(n, v, need, v >= need) for n, v, need in rows]


# ---------------------------------------------------------------- the ten
V = [
    dict(name="Vintage Story: warm brown, wood slots, cyan pick", game="Vintage Story (its own GuiStyle values)",
         panel=hx("#403529"), frame=hx("#2b2219"), outline=hx("#17120d"), slot=hx("#6e4f3b"), well=hx("#4a3d30"),
         main=hx("#5a4330"), second=hx("#453424"), accent=hx("#62c5db"), text=hx("#e9ddce"), dim=hx("#bba78e"),
         title=hx("#edce98"), on_main=hx("#e0cfbb"), button_text=hx("#e0cfbb"), slider=hx("#845c43"), chip=hx("#5a4636")),
    dict(name="Valheim: quiet dark wood, orange accent", game="Valheim",
         panel=hx("#1f1712"), frame=hx("#6e4c30"), outline=hx("#0e0a07"), slot=hx("#33281f"),
         well=hx("#291f18"), main=hx("#7a4a24"), second=hx("#3a2a1e"), accent=hx("#ffa13c"),
         text=hx("#e8d6b0"), dim=hx("#b49c76"), title=hx("#ffb84a"), on_main=hx("#fff0cf"), slider=hx("#4a3524")),
    dict(name="Hytale: warm wood and gold", game="Hytale",
         panel=hx("#4a3322"), texture="wood", tex_light=1.1, tex_dark=0.9, frame=hx("#d1a04c"), outline=hx("#24160a"),
         slot=hx("#33231a"), main=hx("#c98a32"), second=hx("#5e432e"), accent=hx("#fff0b0"),
         text=hx("#fbeed2"), dim=hx("#d2b88e"), title=hx("#ffd77a"), on_main=hx("#2c1a08"), button_text=hx("#fbeed2")),
    dict(name="Minecraft Dungeons: chunky stone and gold", game="Minecraft Dungeons",
         panel=hx("#2f2a25"), texture="stone", tex_light=1.1, tex_dark=0.9, frame=hx("#c9962e"), outline=hx("#21170a"),
         slot=hx("#221e1a"), main=hx("#d7a536"), second=hx("#4c4036"), accent=hx("#ffe27a"),
         text=hx("#f5eedf"), dim=hx("#c4b59a"), title=hx("#ffd65c"), on_main=hx("#2a1c0a"), button_text=hx("#f5eedf")),
    dict(name="Stardew: warm wood and parchment", game="Stardew Valley",
         panel=hx("#f1d39a"), texture="parchment", tex_dark=0.95, frame=hx("#b0642c"), outline=hx("#4a2210"),
         slot=hx("#d8a861"), well=hx("#e4bd7d"), main=hx("#c2552b"), second=hx("#9b6a3a"), accent=hx("#e8402a"),
         text=hx("#4a2614"), dim=hx("#7a5030"), title=hx("#8a2c16"), button_text=hx("#fff1d6"), on_main=hx("#fff1d6"),
         chip=hx("#e2b878"), slider=hx("#c89452")),
    dict(name="Dwarf Fortress: red-brown stone, gem green", game="Dwarf Fortress (Steam)",
         panel=hx("#4a3428"), texture="stone", tex_light=1.12, tex_dark=0.86, frame=hx("#b0843e"), slot=hx("#5e4434"),
         well=hx("#553d30"), main=hx("#9a6a2a"), second=hx("#5c4233"), accent=hx("#4fd07e"), text=hx("#f2e4c6"),
         dim=hx("#c9b08a"), title=hx("#ecc870"), on_main=hx("#fff4d8")),
    dict(name="Terraria: leather tan and sky", game="Terraria",
         panel=hx("#6a4a2f"), texture="leather", tex_light=1.08, tex_dark=0.92, frame=hx("#3d2716"), outline=hx("#1c120a"),
         slot=hx("#4c341f"), well=hx("#58402a"), main=hx("#a8692c"), second=hx("#563a22"), accent=hx("#6cc4ff"),
         text=hx("#fff4dc"), dim=hx("#e2c9a2"), title=hx("#ffe08a"), on_main=hx("#fff6e2"), chip=hx("#7c5838")),
    dict(name="Enshrouded: near-black leather, brass, teal", game="Enshrouded",
         panel=hx("#1b1613"), texture="leather", tex_light=1.18, tex_dark=0.9, frame=hx("#9c7c48"), outline=hx("#0a0806"),
         slot=hx("#2e2620"), well=hx("#241d18"), main=hx("#4a3a2a"), second=hx("#2c231c"), accent=hx("#5fd6cc"),
         text=hx("#e6d8bf"), dim=hx("#ae9a7c"), title=hx("#e2c27e"), on_main=hx("#f0dcb4"), button_text=hx("#e6d8bf")),
    dict(name="Kenshi: rust and bone, iron trim", game="Kenshi",
         panel=hx("#4a2c20"), frame=hx("#6d655c"), outline=hx("#1a1310"), slot=hx("#3a231a"), well=hx("#43281d"),
         main=hx("#a04a28"), second=hx("#5a3a2c"), accent=hx("#e0603c"), text=hx("#e4d9c2"), dim=hx("#b9a184"),
         title=hx("#efcfa6"), on_main=hx("#f8ecd6")),
    dict(name="Grounded: cardboard and bold outline", game="Grounded",
         panel=hx("#c7a273"), texture="parchment", tex_dark=0.94, frame=hx("#7a4e28"), outline=hx("#22140a"),
         slot=hx("#a98454"), well=hx("#b8925f"), main=hx("#5f8f2c"), second=hx("#8a6236"), accent=hx("#a8dc4a"),
         text=hx("#26170b"), dim=hx("#5a4026"), title=hx("#3a2008"), button_text=hx("#fff6e4"), on_main=hx("#fff6e4"),
         chip=hx("#b48d5e"), slider=hx("#9a744a")),
]

SHOT_FILES = [("Inventory", "t2_k2a_world/03_inventory.png", (400, 140, 880, 540)),
              ("Animal: Overview", "t2_k2a_pages/pages_overview.png", (240, 110, 1040, 600)),
              ("Settings", "t2_k2a_menus/23_settings.png", (320, 80, 960, 520))]


def main():
    shots = [(n, np.array(Image.open(os.path.join(SHOTS, f)).convert("RGB")), box) for n, f, box in SHOT_FILES]
    os.makedirs(OUT, exist_ok=True)
    results = []
    for i, v in enumerate(V):
        crops = []
        for n, a, box in shots:
            out, p = apply(a, v)
            crops.append(Image.fromarray(out).crop(box))
        results.append((v, p, crops, check(p)))
        print(f"{i + 1:2d} {v['name']:42s}", "  ".join(f"{n} {val:.1f}{'' if ok else ' LOW'}" for n, val, _, ok in check(p)))
    return results


if __name__ == "__main__":
    main()


def sheet(results, path):
    """The ten on one sheet: number, name, the game that inspired it, swatches, what the checklist
    measured, then the three screens at 1:1."""
    def label(img, s, x, y, scale=2, color=k.GOLD_TEXT):
        t = k.canvas(k.F.width(s) + 4, k.F.line + 4)
        k.F.draw(t, s, 0, 2, color, (12, 8, 7))
        im = Image.fromarray(t).resize(((k.F.width(s) + 4) * scale, (k.F.line + 4) * scale), Image.NEAREST)
        img.paste(im, (x, y), im)

    widths = [c.width for c in results[0][2]]
    row_h = max(c.height for c in results[0][2]) + 150
    W = 20 + sum(widths) + 20 * len(widths)
    out = Image.new("RGB", (W, 140 + len(results) * row_h), (26, 19, 19))
    label(out, "Ten brown looks for the UI, on the same in-game shots at 1:1. The first three are my picks.", 20, 14, 3)
    label(out, "Inventory, the animal screen, settings. Swatches: panel, frame, slot, main button, accent, text. Then what the checklist measured.", 20, 70, 2, k.DIM)
    y = 140
    for i, (v, p, crops, chk) in enumerate(results):
        pick = "  (pick %d)" % (i + 1) if i < 3 else ""
        label(out, f"{i + 1}  {v['name']}{pick}", 20, y, 3, k.GOLD_TEXT if i < 3 else k.CREAM)
        label(out, f"inspired by {v['game']}", 20, y + 50, 2, k.DIM)
        sx = 20 + k.F.width(f"inspired by {v['game']}") * 2 + 40
        for j, role in enumerate(("panel", "frame", "slot", "main_f", "glow", "text")):
            sw = Image.new("RGB", (32, 32), p[role])
            out.paste(sw, (sx + j * 38, y + 46))
        bad = [n for n, val, need, ok in chk if not ok]
        short = {"text on panel": "text", "dim text on panel": "dim", "title on panel": "title",
                 "button text on main": "button", "text on second button": "button 2", "slot against panel": "slots",
                 "frame against panel": "frame"}
        line = "contrast " + ", ".join(f"{short[n]} {val:.1f}" for n, val, need, ok in chk) + (
            ": all pass" if not bad else ": low " + ", ".join(short[n] for n in bad) + (
                " (its frame is a shadow line, by design)" if "Vintage" in v["name"] else ""))
        label(out, line, 20, y + 88, 2, k.DIM if not bad else (240, 150, 90))
        x = 20
        for c in crops:
            out.paste(c, (x, y + 126))
            x += c.width + 20
        y += row_h
    out.save(path)
    return out.size


if __name__ != "__main__" and False:
    pass
