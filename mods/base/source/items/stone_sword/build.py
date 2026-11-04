"""Stone sword, a 3D item model from its icon (first try).

Boxes at 64 texels a block (a texel is 1/64 m), laid out from the icon turned upright: pommel,
grip, guard, blade with a raised ridge, and a pointed tip from a box turned 45 degrees. Three
textures on one layout:
- "refined" (the default): the icon's colors on each material's ramp, light painted face by face
  (see `refined_faces`);
- "icon": the icon's own colors, each face sampled from the upright icon (part by part), sides
  shaded from it;
- "house": the creatures' painter (paint.py: one light from the top front left, occlusion), in
  the icon's colors: stone with darker chips, an orange wooden grip wrapped in darker cord.

    ~/.venvs/blocky/bin/python build.py icon_upright.png out/
    (then copy out/*.png and out/model.json next to viewer.html, serve the folder, python3 shots.py)
"""
import json
import math
import os
import sys

import numpy as np
from PIL import Image, ImageFilter

HERE = os.path.dirname(os.path.abspath(__file__))
SKILL = os.path.join(HERE, "..", "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts")
sys.path.insert(0, SKILL)
from paint import KS, STEP, occlusion, pack, tones  # noqa: E402
from prop import FACES, el, face_frame, paint, rotm  # noqa: E402

icon_path, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)

# ---- layout, texel units, y up from the pommel's bottom, blade flat toward +z/-z
TIP = 45
T = 80  # the blade's top; the point reaches T + 7 (a sword: blade about 70% of the length)
elements = [
    el("pommel", (-5, 0, -5), (5, 8, 5)),
    el("grip", (-3.5, 7.5, -3.5), (3.5, 20.25, 3.5)),
    el("guard", (-13, 19.5, -4), (13, 26, 4)),
    el("blade", (-7, 25.5, -1.5), (7, T, 1.5)),
    el("ridge", (-3, 25.75, -2.5), (3, T + 1.5, 2.5)),
    # squares turned 45 degrees: the upper half of each is the point
    el("tip", (-4.95, T - 4.95, -1.25), (4.95, T + 4.95, 1.25), rot=(0, 0, TIP), origin=(0, T, 0)),
    el("ridge_tip", (-2.12, T + 1.5 - 2.12, -2.25), (2.12, T + 1.5 + 2.12, 2.25), rot=(0, 0, TIP), origin=(0, T + 1.5, 0)),
]
PART = {"pommel": "pommel", "grip": "grip", "guard": "guard", "blade": "blade", "ridge": "blade", "tip": "blade",
        "ridge_tip": "blade"}

# ---- the icon, upright: each part's front face in icon pixels (inside its dark outline)
# (model x0, x1, y0, y1) -> (icon x0, x1, y0, y1); icon y grows downward
ICON_MAP = {
    "pommel": ((-5, 5, 0, 8), (84, 158, 520, 468)),
    "grip": ((-3.5, 3.5, 8, 20), (86, 146, 462, 380)),
    "guard": ((-13, 13, 19.5, 26), (12, 200, 348, 300)),
    "blade": ((-7, 7, 25.5, 64), (47, 146, 288, 12)),  # the icon's blade; a longer one repeats its middle
}
src = Image.open(icon_path).convert("RGBA")
A = np.array(src).astype(float)
inner = np.array(src.split()[3].filter(ImageFilter.MinFilter(19))) > 128  # off the outline
lum = A[..., :3].mean(-1)
inner &= lum > 40
r, g, b = A[..., 0], A[..., 1], A[..., 2]
inner &= ~((r - g > 30) & (b - g > 30))  # the background's tint left by turning the icon
inner &= ~((r - g > 40) & (b >= g))  # the outline's dark red-purple
# fill outline and empty pixels with the nearest inner color, so any sample is a real color
col = np.where(inner[..., None], A[..., :3], 0)
have = inner.copy()
for _ in range(60):
    if have.all():
        break
    acc = np.zeros_like(col)
    n = np.zeros(have.shape)
    for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        acc += np.roll(np.roll(col * have[..., None], dy, 0), dx, 1)
        n += np.roll(np.roll(have, dy, 0), dx, 1)
    new = ~have & (n > 0)
    col[new] = acc[new] / n[new, None]
    have |= new
col = np.array(Image.fromarray(col.clip(0, 255).astype(np.uint8)).filter(ImageFilter.BoxBlur(3))).astype(float)


def icon_color(part, x, y):
    (mx0, mx1, my0, my1), (ix0, ix1, iy0, iy1) = ICON_MAP[part]
    u = ix0 + (x - mx0) / (mx1 - mx0) * (ix1 - ix0)
    if part == "blade":
        # icon pixels a texel as drawn, counted down from the point; past the icon's blade the
        # middle of it repeats back and forth, so the stone isn't stretched
        k = (iy0 - iy1) / (my1 - my0)
        v = iy1 + (T + 7 - y) * k
        lo, hi = 70, iy0
        span = hi - lo
        t = np.maximum(v - lo, 0) % (2 * span)
        v = np.where(v > hi, np.where(t < span, lo + t, hi - (t - span)), v)
        u = u + (np.minimum(v, hi) - 150) * 0.07  # the icon's blade leans a little
    else:
        v = iy0 + (y - my0) / (my1 - my0) * (iy1 - iy0)
    u = np.clip(np.round(u).astype(int), 0, col.shape[1] - 1)
    v = np.clip(np.round(v).astype(int), 0, col.shape[0] - 1)
    return col[v, u]


SIDE_LIGHT = {"south": 1.0, "north": 0.82, "west": 0.92, "east": 0.72, "up": 1.12, "down": 0.6}


def icon_faces():
    """Each face's texels from the icon: the front and back from the front view at that x, y;
    the sides from the color just inside that edge; shaded a little by which way they face."""
    tiles = []
    for i, e in enumerate(elements):
        for face in FACES:
            corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
            if w == 0 or h == 0:
                continue
            uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
            p = corner + uu[..., None] * du + vv[..., None] * dv
            if e["rot"]:
                o = np.array(e["origin"], float)
                p = (p - o) @ rotm(e["rot"]).T + o
            x, y = p[..., 0], p[..., 1]
            part = PART[e["name"]]
            if face in ("east", "west"):  # just inside the edge
                x = x - np.sign(x) * 1.0
            if face in ("up", "down"):
                y = y - (1.0 if face == "up" else -1.0)
            c = icon_color(part, x, y) * SIDE_LIGHT[face]
            if face in ("east", "west", "up", "down") and h > 2 and w > 2:  # darker toward the far edge
                c *= (0.92 + 0.08 * (1 - np.abs(vv / h - 0.5) * 2))[..., None]
            img = np.dstack([c.clip(0, 255), np.full((h, w), 255)]).astype(np.uint8)
            tiles.append(((i, face), img))
    return tiles


# ---- the house painter, in the icon's colors
def median(part, sel=None):
    (mx0, mx1, my0, my1), (ix0, ix1, iy0, iy1) = ICON_MAP[part]
    region = A[min(iy0, iy1):max(iy0, iy1), min(ix0, ix1):max(ix0, ix1)]
    ok = inner[min(iy0, iy1):max(iy0, iy1), min(ix0, ix1):max(ix0, ix1)]
    px = region[ok][:, :3]
    if sel == "dark":
        px = px[px.mean(1) < np.percentile(px.mean(1), 25)]
    if sel == "lit":  # the lit side, the color a part reads as
        px = px[px.mean(1) > np.percentile(px.mean(1), 55)]
    return tuple(int(v) for v in np.median(px, 0))


STONE, CHIP, WOOD, CORD = range(4)
MATS = [tones(median("blade"), 0.74), tones(median("blade", "dark"), 0.74), tones(median("pommel"), 0.04),
        tones(median("grip", "dark"), 0.03)]


def area(e, face, p, uu, vv, h):
    part = PART[e["name"]]
    if part in ("blade", "guard"):
        # chips: fixed spots in space, so they run over edges
        rng = np.random.default_rng(7)
        spots = rng.uniform((-13, 20, -3), (13, T + 8, 3), (90, 3))
        d = np.min(np.linalg.norm(p[..., None, :] - spots[None, None], axis=-1), -1)
        return np.where(d < 1.4, CHIP, STONE)
    if part == "grip":  # cord wound round, leaning a little
        t = (p[..., 1] + 0.35 * (p[..., 0] + p[..., 2])) % 2.5
        return np.where(t < 0.9, CORD, WOOD)
    return np.full(p.shape[:2], WOOD)


# ---- refined: the icon's colors on each material's own ramp, with the light painted as the
# Hytale and Blockbench style guides do it: face by face from the top left (the icon's light),
# a lit rim on each face's top and left edges and a dark one on its bottom and right, contact
# shadow where parts meet, the blade brightest at the point, sharpened lighter cutting edges
# with chips, cord wound round the grip; few deliberate shades, hue-shifted shadows, no noise.
# stone: shadows stay in its own brown-grey (toward purple they pass through red on a warm grey)
RAMP = {"blade": tones(median("blade"), 0.07), "guard": tones(median("blade"), 0.07),
        "grip": tones(median("grip", "lit"), 0.03), "pommel": tones(median("pommel", "lit"), 0.04)}
BASE_LUM = {k: np.mean(median("blade" if k == "guard" else k)) for k in RAMP}
FACE_L = {"up": 0.5, "south": 0.0, "north": -0.25, "west": 0.25, "east": -0.5, "down": -1.0}
SOLID = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in elements if not e["rot"]]


def chip(y, side):
    """Whether the cutting edge has a chip at this height (fixed, so both faces agree)."""
    k = np.floor(y / 1.0).astype(int) * 7 + side * 13
    return ((k * 2654435761) % 1000) / 1000 < 0.22


def refined_faces():
    tiles = []
    for i, e in enumerate(elements):
        for face in FACES:
            corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
            if w == 0 or h == 0:
                continue
            uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
            p = corner + uu[..., None] * du + vv[..., None] * dv
            R = rotm(e["rot"])
            if e["rot"]:
                o = np.array(e["origin"], float)
                p = (p - o) @ R.T + o
            part = PART[e["name"]]
            x, y = p[..., 0], p[..., 1]
            sx, sy = x.copy(), y.copy()
            if face in ("east", "west"):
                sx = x - np.sign(x) * 1.0
            if face in ("up", "down"):
                sy = y - (1.0 if face == "up" else -1.0)
            c = icon_color(PART[e["name"]], sx, sy)
            key = e["name"] if e["name"] in ("guard", "grip", "pommel") else "blade"
            # the icon's own light and marks, in half steps (clean clusters, not noise)
            L = np.round(np.clip((c.mean(-1) / BASE_LUM[key] - 1) / 0.15, -1.5, 1.5) * 2) / 2 * (key not in ("grip", "pommel"))
            L = L + FACE_L[face]
            # painted bevels on every face's edges
            if face in ("up", "down"):
                if w >= 3 and h >= 3:
                    rim = (uu < 1) | (vv < 1) | (uu > w - 1) | (vv > h - 1)
                    L = L + 0.25 * rim * (face == "up")
            else:
                if h >= 3:
                    L = L + 0.5 * (vv < 1) - 0.5 * (vv > h - 1)
                if h >= 4:  # light falls off down each side
                    L = L + 0.6 * (0.5 - vv / h) * (key != "blade")
                if w >= 3:
                    L = L + 0.25 * (uu < 1) - 0.25 * (uu > w - 1)
            if key == "blade":
                L = L + (y - 26) / (T + 7 - 26) * 0.75 - 0.25
                if e["name"] == "blade" and face in ("south", "north"):
                    edge = 7 - np.abs(x)
                    nick = chip(y, (x > 0).astype(int))
                    L = L + np.where(edge < 1, np.where(nick, -0.75, 0.75), 0) + np.where((edge >= 1) & (edge < 2) & nick, -0.5, 0)
            if key == "grip" and face not in ("up", "down"):
                row = np.floor(y).astype(int) % 3  # cord: a groove, a lit top, the round of it
                L = L + np.where(row == 0, -1.0, np.where(row == 2, 0.5, 0))
            if not e["rot"]:
                L = L - 1.25 * occlusion(p, R @ n, R @ du, R @ dv, SOLID, own=e["name"])
            L = np.clip(L, -2, 2)
            k = np.clip(np.round((np.round(L / STEP) * STEP - KS[0]) / STEP), 0, len(KS) - 1).astype(int)
            col = RAMP[key][k]
            img = np.dstack([col.clip(0, 255), np.full((h, w), 255)]).astype(np.uint8)
            tiles.append(((i, face), img))
    return tiles


house_sheet, uv = paint(elements, area, MATS, ground=False, width=128)
icon_sheet, uv2 = pack(icon_faces(), 128)
refined_sheet, uv3 = pack(refined_faces(), 128)
assert uv == uv2 == uv3, "the textures must share a layout"


def square(sheet):
    """The engine takes square textures only (or vertical strips, for animations)."""
    side = max(sheet.shape[:2])
    sq = np.zeros((side, side, 4), np.uint8)
    sq[:sheet.shape[0], :sheet.shape[1]] = sheet
    return sq


house_sheet, icon_sheet, refined_sheet = square(house_sheet), square(icon_sheet), square(refined_sheet)
Image.fromarray(house_sheet).save(f"{out}/house.png")
Image.fromarray(icon_sheet).save(f"{out}/icon.png")
Image.fromarray(refined_sheet).save(f"{out}/refined.png")
W, H = house_sheet.shape[1], house_sheet.shape[0]
print("texture", W, "x", H, "texels; length", max(e["hi"][1] for e in elements[:5]) + 7, "texels")

# ---- quads for the viewer (world texel units) and a Java item model for the engine
quads = []
for i, e in enumerate(elements):
    for face in FACES:
        corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
        if (i, face) not in uv:
            continue
        pts = [corner, corner + du * w, corner + du * w + dv * h, corner + dv * h]
        if e["rot"]:
            o = np.array(e["origin"], float)
            pts = [(q - o) @ rotm(e["rot"]).T + o for q in pts]
        u0, v0, u1, v1 = uv[(i, face)]
        quads.append({"p": [list(map(float, q)) for q in pts], "uv": [u0 / W, v0 / H, u1 / W, v1 / H],
                      "n": list(map(float, rotm(e["rot"]) @ n))})
json.dump({"quads": quads, "size": [W, H]}, open(f"{out}/model.json", "w"))

# The game's item: laid in the item's square like its sprite (handle bottom left, point top
# right, turned -45 degrees about its middle), so the engine holds it as it holds a sword sprite
# (`handheld`). The points, turned +45 already, come out straight.
K = 16 / 64  # Java units (16 a block) per texel
C = np.array([0.0, (T + 7) / 2, 0.0])  # the sword's middle, at the square's middle
TURN = -45.0
Rz = rotm((0, 0, TURN))


def j(v):
    return [round(float(a), 4) for a in (np.array(v) - C) * K + 8]


java = {"textures": {"sword": "base:item/stone_sword_model", "particle": "base:item/stone_sword"}, "elements": []}
for i, e in enumerate(elements):
    faces = {}
    for face in FACES:
        if (i, face) in uv:
            u0, v0, u1, v1 = uv[(i, face)]
            faces[face] = {"texture": "#sword", "uv": [round(u0 / W * 16, 4), round(v0 / H * 16, 4),
                                                       round(u1 / W * 16, 4), round(v1 / H * 16, 4)]}
    lo, hi = np.array(e["lo"]), np.array(e["hi"])
    d = {"name": e["name"], "faces": faces}
    if e["rot"] and e["rot"][2] + TURN == 0:
        mid = C + Rz @ ((lo + hi) / 2 - C)
        d["from"], d["to"] = j(mid - (hi - lo) / 2), j(mid + (hi - lo) / 2)
    else:
        assert not e["rot"], "only the points are turned"
        d["from"], d["to"] = j(lo), j(hi)
        d["rotation"] = {"angle": TURN, "axis": "z", "origin": [8.0, 8.0, 8.0]}
    java["elements"].append(d)
json.dump(java, open(f"{out}/stone_sword.json", "w"), indent=1)
ASSETS = os.path.join(HERE, "..", "..", "..", "assets")
json.dump(java, open(os.path.join(ASSETS, "models", "item", "stone_sword.json"), "w"), indent=1)
Image.fromarray(refined_sheet).save(os.path.join(ASSETS, "textures", "item", "stone_sword_model.png"))
spec = {"elements": [{**e, "bone": "body", "faces": {f: uv.get((i, f)) for f in FACES}} for i, e in enumerate(elements)]}
json.dump(spec, open(f"{out}/spec.json", "w"))
