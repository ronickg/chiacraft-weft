"""Writes kit 2 into the game (Ronald, 2026-11-04: "Yes, build it"): every shared UI part's
texture and `.gui` in mods/base/assets/textures/gui, drawn by `kit2.py` with Ronald's pick C (every
corner steps 3 px) at the size the texture has always had, so layouts don't move.

Textures are at twice the GUI's resolution (one texel is a UI pixel, half a GUI pixel), so a
`.gui`'s size and border are half the texture's pixels.

How labels centre (UI guide, D): the layout rounds a centred thing's odd leftover down, so in a
part of even height the capitals land at (h - 10) / 2; that is the middle of kit 2's flat face
when its frame is 2 px above (outline, light) and 3 below (shade, outline). Chips and other parts
sized by their text are an odd height (17 px of line plus even padding), so they get a symmetric
frame (2 and 2).

Usage: python3 install.py [--dry]   (old textures are copied to the archive first)"""
import os
import shutil
import sys
import time

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kit2 as k  # noqa: E402

GUI = os.path.join(k.REPO, "mods/base/assets/textures/gui")
ARCHIVE = os.path.expanduser("~/Coding/chiacraft-archive/images/ui-redo/2026-11-04/gui-before-kit2")
DRY = "--dry" in sys.argv
# `--out DIR` writes there instead of the game (for a look first); sizes still come from the game.
OUT = sys.argv[sys.argv.index("--out") + 1] if "--out" in sys.argv else GUI
WRITTEN = []


def save(name, a, gui):
    """Writes `name`.png and its `.gui` (keys in GUI pixels)."""
    path = os.path.join(OUT, name + ".png")
    old = os.path.join(GUI, name + ".png")
    if os.path.exists(old) and not DRY and OUT == GUI:
        os.makedirs(ARCHIVE, exist_ok=True)
        for ext in (".png", ".gui"):
            src = os.path.join(GUI, name + ext)
            dst = os.path.join(ARCHIVE, name + ext)
            if os.path.exists(src) and not os.path.exists(dst):
                shutil.copy2(src, dst)
    h, w = a.shape[:2]
    text = "".join(f"{key} = {val}\n" for key, val in gui)
    text += f"size = {fmt(w / 2)} {fmt(h / 2)}\n"
    if not DRY:
        os.makedirs(OUT, exist_ok=True)
        Image.fromarray(a).save(path)
        os.makedirs(OUT, exist_ok=True)
        open(os.path.join(OUT, name + ".gui"), "w").write(text)
    WRITTEN.append((name, w, h, text.strip().replace("\n", "; ")))


def fmt(v):
    return str(int(v)) if v == int(v) else str(v)


NINE = ("scaling", "nine_slice")


def border(*px):
    """A `.gui` border from texture pixels: one for all sides or left, top, right, bottom."""
    return ("border", " ".join(fmt(p / 2) for p in px))


# ---------------------------------------------------------------- buttons
def faces(kind):
    base = {"main": "main", "second": "second"}
    return base[kind]


def danger_faces():
    light, face, shade = k.FACES["danger"]
    up = lambda c, f: tuple(min(255, int(v * f)) for v in c)
    return {"normal": (light, face, shade), "hover": (up(light, 1.1), up(face, 1.12), shade),
            "pressed": (shade, up(face, 0.85), face)}


def raised_part(w, h, light, face, shade, rim=k.OUTLINE, pressed=False, shade_px=2):
    a = k.canvas(w, h)
    k.raised(a, k.shape(w, h, k.R), light, face, shade, rim, pressed, shade_px)
    return a


def button_set(prefix, kind):
    for state, suffix in (("normal", ""), ("hover", "_hover"), ("pressed", "_pressed"), ("disabled", "_disabled")):
        key = {"normal": kind, "hover": kind + "_hover", "pressed": kind + "_pressed"}.get(state, kind)
        if state == "disabled":
            key = "disabled"
        light, face, shade = k.FACES[key]
        a = raised_part(64, 20, light, face, shade, k.GLOW if state == "hover" else k.OUTLINE, state == "pressed")
        save(prefix + suffix, a, [NINE, border(3)])


button_set("button", "main")
button_set("button_secondary", "second")
for state, suffix in (("normal", ""), ("hover", "_hover"), ("pressed", "_pressed")):
    light, face, shade = danger_faces()[state]
    a = raised_part(64, 20, light, face, shade, k.GLOW if state == "hover" else k.OUTLINE, state == "pressed")
    save("button_quit" + suffix, a, [NINE, border(3)])

# Categories (the recipe book's side list) and list rows (worlds, servers): secondary faces; the
# chosen one is the main colour, pressed.
for prefix, w, h in (("category", 32, 24), ("list_row", 64, 24)):
    for state, suffix in (("normal", ""), ("hover", "_hover"), ("pressed", "_pressed"), ("disabled", "_disabled")):
        if state == "pressed":
            light, face, shade = k.FACES["main_pressed"]
        elif state == "disabled":
            light, face, shade = k.FACES["disabled"]
        else:
            light, face, shade = k.FACES["second" if state == "normal" else "second_hover"]
        a = raised_part(w, h, light, face, shade, k.GLOW if state == "hover" else k.OUTLINE, state == "pressed")
        save(prefix + suffix, a, [NINE, border(3)])

# ---------------------------------------------------------------- tabs
# A tab stands on the window's frame (the window draws over its bottom 4 px), so it has no bottom
# edge of its own; the active one is the frame's bronze, the others darker.
TAB_FACES = {"": ((110, 72, 46), (78, 50, 32), (58, 36, 24)), "_hover": ((140, 96, 62), (98, 64, 40), (66, 42, 28)),
             "_pressed": (k.BRONZE_HI, k.BRONZE, k.BRONZE_D), "_disabled": k.FACES["disabled"]}
for suffix, (light, face, shade) in TAB_FACES.items():
    w, h = 48, 26
    a = k.canvas(w, h)
    m = k.shape(w, h + 6, k.R, "top")[:h]
    # no bottom edge: the shape carries on under the window's frame
    ring = k.edge(np.pad(m, ((0, 1), (0, 0)), constant_values=True))[:h]
    rim = k.GLOW if suffix == "_hover" else k.OUTLINE
    k.paint(a, m & ring, rim)
    k.paint(a, m & ~ring, face)
    up = np.zeros_like(m)
    up[1:] = ring[:-1]
    k.paint(a, up & m & ~ring, light)
    save("tab" + suffix, a, [NINE, border(3, 4, 3, 4)])

# ---------------------------------------------------------------- chips
# Sized by their text (17 px of line and even padding): an odd height, so the frame is
# symmetric, 2 px above and 2 below, and the capitals centre exactly.
for name, kind in (("chip", "plain"), ("chip_points", "points")):
    light, face, shade, _ = k.CHIPS[kind]
    a = raised_part(16, 16, light, face, shade, shade_px=1)
    save(name, a, [NINE, border(3)])

# ---------------------------------------------------------------- windows and boxes
a = k.panel_r(64, 64, k.R)
save("panel", a, [NINE, border(6)])
save("news", a, [NINE, border(6)])


def sunken(w, h, face, dark, light, rim=k.OUTLINE):
    a = k.canvas(w, h)
    m = k.shape(w, h, k.R)
    ring = k.edge(m)
    inner = m & ~ring
    k.paint(a, ring, rim)
    k.paint(a, inner, face)
    sh = lambda dy, dx: np.roll(np.roll(ring, dy, 0), dx, 1)
    k.paint(a, (sh(-1, 0) | sh(0, -1)) & inner, light)
    k.paint(a, (sh(1, 0) | sh(0, 1)) & inner, dark)
    return a


save("well", sunken(32, 24, k.WELL, k.WELL_D, k.WELL_L), [NINE, border(3)])
save("portrait", sunken(64, 64, (116, 101, 87), (90, 78, 66), (138, 122, 106)), [NINE, border(3)])
for name in ("tooltip", "hud_plate"):
    a = k.canvas(32, 24)
    m = k.shape(32, 24, k.R)
    ring = k.edge(m)
    k.paint(a, ring, k.OUTLINE)
    m2 = m & ~ring
    r2 = k.edge(m2)
    k.paint(a, r2, k.BRONZE_D)
    k.paint(a, m2 & ~r2, k.PANEL)
    save(name, a, [NINE, border(3)])

# ---------------------------------------------------------------- slots, inputs, scroll, slider
for name in ("slot", "recipe_slot"):
    save(name, k.slot_r(36, False, k.R), [NINE, border(4), ("icon", "16")])
hl = k.canvas(36, 36)
m = k.shape(36, 36, k.R)
ring = k.edge(m)
k.paint(hl, ring, k.GLOW)
k.paint(hl, k.edge(m & ~ring), k.GLOW_D)
save("slot_highlight", hl, [NINE, border(4)])
save("input", sunken(64, 18, k.WELL, k.WELL_D, k.WELL_L), [NINE, border(3)])
save("input_focused", sunken(64, 18, k.WELL, k.WELL_D, k.WELL_L, k.GLOW), [NINE, border(3)])
save("scrollbar", sunken(12, 40, k.WELL, k.WELL_D, k.WELL_L), [NINE, border(4)])
for suffix, key in (("", "second"), ("_hover", "second_hover")):
    light, face, shade = k.FACES[key]
    save("scrollbar_handle" + suffix, raised_part(12, 28, light, face, shade, k.GLOW if suffix else k.OUTLINE, shade_px=1),
         [NINE, border(4)])
# The slider's track fills its whole row (20 GUI px in the settings): a mid-tone groove at the
# sheet's own value (V 0.29), so it reads on the panel (dark, it vanished: PC shots, 11-04).
save("slider", sunken(62, 10, k.SLIDER, k.SLIDER_D, k.SLIDER_L), [NINE, border(3)])
for suffix, key in (("", "main"), ("_hover", "main_hover")):
    light, face, shade = k.FACES[key]
    save("slider_knob" + suffix, raised_part(14, 20, light, face, shade, k.GLOW if suffix else k.OUTLINE), [NINE, border(4)])

# The close button: a small danger button with a cross drawn on its face.
CROSS = ["#......#", ".#....#.", "..#..#..", "...##...", "...##...", "..#..#..", ".#....#.", "#......#"]
for suffix, state in (("", "normal"), ("_hover", "hover"), ("_pressed", "pressed")):
    light, face, shade = danger_faces()[state]
    a = raised_part(14, 14, light, face, shade, k.GLOW if state == "hover" else k.OUTLINE, state == "pressed", shade_px=1)
    dy = 1 if state == "pressed" else 0
    for y, row in enumerate(CROSS):
        for x, ch in enumerate(row):
            if ch == "#":
                a[3 + y + dy, 3 + x] = k.ON_ORANGE + (255,)
    save("close" + suffix, a, [NINE, border(3)])

# ---------------------------------------------------------------- bars
# A frame and a fill, both nine-sliced: the engine lays a sliced fill over the whole bar and cuts
# it to the share, so the rounded ends keep their pixels at any width.
FILLS = {"life_health": "health", "life_food": "food", "life_water": "water", "life_bond": "taming",
         "life_taming": "taming", "life_torpor": "torpor",
         "stat_health": "health", "stat_stamina": "stamina", "stat_food": "food", "stat_damage": "health",
         "stat_speed": "q1", "stat_weight": "food", "stat_oxygen": "water", "stat_fortitude": "q0",
         "stat_torpor": "torpor", "stat_xp": "xp",
         "stat_q0": "q0", "stat_q1": "q1", "stat_q2": "q2", "stat_q3": "q3", "stat_q4": "q4"}
BOND = ((252, 170, 198), (220, 110, 150), (150, 56, 92))
for name, fill in FILLS.items():
    old = Image.open(os.path.join(GUI, name + ".png"))
    w, h = old.size
    frame = k.bar_r(w, 0.0, fill, h, k.R)
    save(name, frame, [NINE, border(3)])
    full = k.bar_r(w, 1.0, fill, h, k.R)
    if name == "life_bond":
        k.BAR_FILLS["bond"] = BOND
        full = k.bar_r(w, 1.0, "bond", h, k.R)
    # The fill alone: its pixels where they differ from the empty frame.
    only = full.copy()
    only[np.all(full == frame, axis=-1)] = 0
    save(name + "_fill", only, [NINE, border(3)])

# ---------------------------------------------------------------- icons and a divider
# The 16 px icons (8 GUI px), for labels: `{img:base:icon_follow 8x8} Follow` (the space between is
# the font's 4 px, the icon-to-text gap of the guide).
for name in k.ICONS:
    save("icon_" + name, k.icon(name), [("scaling", "stretch")])
div = k.canvas(4, 2)
k.rect(div, 0, 0, 4, 1, k.PANEL_DEEP)
k.rect(div, 0, 1, 4, 1, (52, 36, 28))
save("divider", div, [("scaling", "stretch")])

# ---------------------------------------------------------------- the crosshair and the recipe arrow
# The crosshair's box is 9 GUI px (18 UI px): drawn at that, a cream plus, its arms 4 px, a 1 px
# outline (the old one was 36 px, 4 texels a GUI pixel: 1.5 screen px each at GUI scale 6).
xh = k.canvas(18, 18)
plus = np.zeros((18, 18), bool)
plus[7:11, 2:16] = True
plus[2:16, 7:11] = True
ring = np.zeros_like(plus)
for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
    ring |= np.roll(np.roll(plus, dy, 0), dx, 1)
k.paint(xh, ring & ~plus, k.OUTLINE)
k.paint(xh, plus, (240, 230, 214))
k.paint(xh, plus & np.roll(~plus, 1, 0), (255, 250, 240))
save("crosshair", xh, [("scaling", "stretch")])

# The recipe arrow's box is 24 x 16 GUI px (`ui::RECIPE_ARROW`): its 48 x 28 picture and 50 x 28
# fill are drawn into 48 x 32, so they were squashed. Same pictures, 2 empty rows above and below,
# the fill trimmed to the arrow's 48 columns.
def padded(name, w=48, h=32):
    src = np.array(Image.open(os.path.join(ARCHIVE if os.path.exists(os.path.join(ARCHIVE, name + ".png")) else GUI, name + ".png")).convert("RGBA"))
    sh, sw = src.shape[:2]
    if (sw, sh) == (w, h):
        return src
    out = k.canvas(w, h)
    x0 = (sw - w) // 2
    y0 = (h - sh) // 2
    out[y0:y0 + sh] = src[:, x0:x0 + w]
    return out


save("arrow", padded("arrow"), [("scaling", "stretch")])
save("arrow_fill", padded("arrow_fill"), [("scaling", "stretch")])

for name, w, h, gui in WRITTEN:
    print(f"{name:28s} {w}x{h}  {gui}")
print(len(WRITTEN), "textures", "(dry run)" if DRY else f"written; old ones in {ARCHIVE}")
