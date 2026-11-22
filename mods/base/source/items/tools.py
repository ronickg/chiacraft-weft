"""Every tool and sword as a 3D held model (the blocky-model skill's "Items from icons", the stone
sword's way, for all five tiers): one box layout per kind (sword, pickaxe, axe, shovel, hoe), its
colors from the tier's icon, the light painted face by face, each tier with its own finish.

    ~/.venvs/blocky/bin/python mods/base/source/items/tools.py [out/]

Writes `models/item/<tier>_<kind>.json` and `textures/item/<tier>_<kind>_model.png` into the base
mod's assets (the slot keeps showing the icon, the hand and the ground the model), and, with `out/`,
`<name>.json` quads for `render.py`.

The layout, in texels at 64 a block (a texel is 1/64 m), y up along the handle, the head across x,
thickness along z; every box's sizes whole texels (the texture maps texel for texel), touching
boxes a half texel apart in every plane (no flicker), joined parts of different sizes:
- a shared handle: a darker grip end, the stick, the head clamped on its top;
- sword: pommel, corded grip, guard, a blade three thick with a ridge five thick down its middle,
  the point from squares turned 45 degrees (87 long, the blade 70% of it: an arming sword);
- pickaxe: a collar, and on each side four segments, each turned further down and thinner, so the
  pick curves to points;
- axe: a collar, a short back, a neck, the blade, and a taller, thinner cutting edge;
- shovel: a socket, the spade with a raised spine, a point from a square turned 45 degrees;
- hoe: a collar, an arm to one side, the blade hanging from its end, flaring out.

The texture ("refined" in the stone sword's build): per texel, light in ramp steps (0.25 a step),
snapped to each material's ramp (`paint.tones`, the base color the lit part of the icon):
- face light from the top left (up +0.5, west +0.25, south 0, north -0.25, east -0.5, down -1);
  a lit top row and a dark bottom row on side faces, lit and dark side columns, light falling off
  down each side; occlusion where parts meet;
- each tier's finish: wood grain (streaks along the part), stone chips and bumps on its rims,
  iron a bright cutting edge, gold glints, diamond facets and sparkles; cutting edges lighter;
- the grip end darker, a sword's grip wound with cord (rows: a groove, a lit top).
"""
import json
import math
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..", "..", "..", "..")
SKILL = os.path.join(ROOT, ".claude", "skills", "blocky-model", "scripts")
sys.path.insert(0, SKILL)
from boxcheck import check  # noqa: E402
from paint import KS, STEP, occlusion, pack, tones  # noqa: E402
from prop import FACES, el, face_frame, rotm  # noqa: E402

ASSETS = os.path.join(HERE, "..", "..", "assets")
TIERS = ["wood", "stone", "iron", "gold", "diamond"]
KINDS = ["sword", "pickaxe", "axe", "shovel", "hoe"]


# ---------------------------------------------------------------- layouts

def handle(top):
    """The shared handle: a darker grip end and the stick up to `top` (inside the head)."""
    return [el("cap", (-4, 0, -4), (4, 6, 4), mat="cap"),
            el("handle", (-3, 5.5, -3), (3, top, 3), mat="handle")]


def sword():
    T = 66
    return [
        el("pommel", (-5, 0, -5), (5, 8, 5), mat="cap"),
        el("grip", (-4, 7.5, -4), (4, 20.5, 4), mat="grip"),
        el("guard", (-14, 20, -4.5), (14, 27, 4.5), mat="guard"),
        el("blade", (-8, 26.5, -1.5), (8, T, 1.5), mat="head", blade=True),
        el("ridge", (-3, 25.5, -2.5), (3, T + 1.75, 2.5), mat="head"),
        # squares turned 45 degrees: the upper half of each is the point
        el("tip", (-6, T - 6, -1), (6, T + 6, 1), rot=(0, 0, 45), origin=(0, T, 0), mat="head", edge=True),
        el("ridge_tip", (-2, T - 0.25, -2), (2, T + 3.75, 2), rot=(0, 0, 45), origin=(0, T + 1.75, 0), mat="head"),
    ]


def arm(side, start, segs):
    """Segments from `start`, each (length, height, thickness, degrees down), each starting just
    inside the one before; `side` +1 to +x, -1 to -x."""
    out = []
    sx, sy = start
    for k, (L, H, Z, a) in enumerate(segs):
        lo = (sx, sy - H / 2, -Z / 2) if side > 0 else (sx - L, sy - H / 2, -Z / 2)
        hi = (sx + L, sy + H / 2, Z / 2) if side > 0 else (sx, sy + H / 2, Z / 2)
        out.append(el(f"pick_{'R' if side > 0 else 'L'}{k}", lo, hi, rot=(0, 0, -a * side), origin=(sx, sy, 0),
                      mat="head", edge=k == len(segs) - 1))
        r = math.radians(a)
        sx, sy = sx + side * (L - 2) * math.cos(r), sy - (L - 2) * math.sin(r)
    return out


def pickaxe():
    segs = [(12, 10, 8, 4), (11, 9, 7, 18), (10, 7, 6, 34), (9, 5, 5, 52)]
    return handle(54) + [el("collar", (-5, 46, -4.5), (5, 60, 4.5), mat="head")] + \
        arm(1, (4.5, 53.5), segs) + arm(-1, (-4.5, 53.5), segs)


def axe():
    return handle(54) + [
        el("collar", (-4.5, 44, -4), (4.5, 60, 4), mat="head"),
        el("axe_blade", (4, 45, -1.5), (14, 59, 1.5), mat="head"),
        el("axe_plate", (13.5, 40, -1), (19.5, 64, 1), mat="head"),
        # the blade flares from its body to the plate: squares turned 45 degrees, half sunk, give
        # the slanted top and bottom
        el("flare_up", (10, 55.5, -0.75), (17, 62.5, 0.75), rot=(0, 0, 45), origin=(13.5, 59, 0), mat="head"),
        el("flare_lo", (10, 41.5, -0.75), (17, 48.5, 0.75), rot=(0, 0, 45), origin=(13.5, 45, 0), mat="head"),
        el("axe_edge", (19, 39.5, -0.5), (22, 64.5, 0.5), mat="head", edge=True),
    ]


def shovel():
    return handle(48) + [
        el("socket", (-4, 41, -3.5), (4, 51, 3.5), mat="head"),
        el("spade", (-11, 50.5, -1.5), (11, 68.5, 1.5), mat="head"),
        el("spine", (-2.5, 50, -2.5), (2.5, 66, 2.5), mat="head"),
        el("spade_tip", (-6.5, 62, -1), (6.5, 75, 1), rot=(0, 0, 45), origin=(0, 68.5, 0), mat="head", edge=True),
    ]


def hoe():
    return handle(54) + [
        el("collar", (-4.5, 46, -4), (4.5, 60, 4), mat="head"),
        el("hoe_arm", (4, 50, -3), (17, 57, 3), mat="head"),
        el("hoe_blade", (13.5, 40, -1.5), (20.5, 56, 1.5), rot=(0, 0, 22), origin=(17, 54, 0), mat="head",
           edge=True),
    ]


LAYOUTS = {"sword": sword, "pickaxe": pickaxe, "axe": axe, "shovel": shovel, "hoe": hoe}


# ---------------------------------------------------------------- colors from the icons

def icon_colors(name):
    """The color of the icon's head and of its handle: the icon lies diagonally, handle at the
    bottom left, so the head is the far end along that diagonal."""
    a = np.asarray(Image.open(os.path.join(ASSETS, "textures", "item", name + ".png")).convert("RGBA")).astype(float)
    n = a.shape[0]
    y, x = np.mgrid[0:n, 0:n]
    t = (x + (n - 1 - y)) / (2 * (n - 1))  # 0 at the bottom left corner, 1 at the top right
    lum = a[..., :3].mean(-1)
    ok = (a[..., 3] > 128) & (lum > 70)  # off the dark outline

    def lit(sel):  # the mid-tones: the color the part reads as, its saturation kept
        px = a[sel & ok][:, :3]
        m = px.mean(1)
        px = px[(m >= np.percentile(m, 20)) & (m <= np.percentile(m, 65))]
        return tuple(np.median(px, 0))
    head = lit(t > (0.72 if name.endswith("sword") else 0.66))
    stick = lit((t > 0.2) & (t < 0.42) & (np.abs(x - (n - 1 - y)) < n * 0.12))
    return head, stick


# shadow hue by substance (paint.tones): warm for wood, its own brown-grey for stone (toward
# purple a warm grey passes through red), cool blue for iron and diamond, warm for gold
HUE = {"wood": 0.04, "stone": 0.07, "iron": 0.57, "gold": 0.05, "diamond": 0.58}


# ---------------------------------------------------------------- the painter

FACE_L = {"up": 0.5, "south": 0.0, "north": -0.25, "west": 0.25, "east": -0.5, "down": -1.0}


def hashf(*ks):
    """A fixed pseudo-random number in 0..1 per integer cell."""
    h = np.zeros_like(np.asarray(ks[0]), dtype=np.int64)
    for k, m in zip(ks, (73856093, 19349663, 83492791)):
        h = h ^ (np.asarray(k, np.int64) * m)
    return ((h * 2654435761) % 4093) / 4093.0


def spots(seed, n, lo, hi):
    return np.random.default_rng(seed).uniform(lo, hi, (n, 3))


def near(p, pts, r):
    d = np.min(np.linalg.norm(p[..., None, :] - pts[None, None], axis=-1), -1)
    return d < r


def finish(tier, p, face, e, uu, vv, w, h):
    """A tier's own marks, in ramp steps, from fixed points in space (so they run over edges and
    both sides agree)."""
    L = np.zeros(p.shape[:2])
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    if tier == "wood":
        along = np.argmax(np.array(e["hi"]) - np.array(e["lo"]))  # grain runs along the part
        acr = [a for a in range(3) if a != along]
        c = np.floor(p[..., acr[0]] + p[..., acr[1]] + 100).astype(int)
        seg = np.floor(p[..., along] / 6 + hashf(c) * 3).astype(int)
        r = hashf(c, seg)
        L += np.where(r < 0.3, -0.75, np.where(r > 0.82, 0.5, 0))
    elif tier == "stone":
        L += np.where(near(p, spots(3, 90, (-34, 20, -5), (34, 80, 5)), 1.5), -1.0, 0)
        L += np.where(near(p, spots(4, 60, (-34, 20, -5), (34, 80, 5)), 1.0), 0.5, 0)
        if face not in ("up", "down") and h >= 3:  # bumpy rims: notches along the top and bottom rows
            nick = hashf(np.floor(x + z * 3 + 50).astype(int), np.floor(y + 50).astype(int)) < 0.35
            L += np.where(((vv < 1) | (vv > h - 1)) & nick, -1.0, 0)
    elif tier == "iron":  # smooth, a polished streak across it
        band = np.abs(((x - y) % 24) - 12)
        L += np.where(band < 2, 0.75, np.where(band < 3.5, 0.25, -0.25))
    elif tier in ("gold", "diamond"):
        if tier == "gold":  # a warm sheen across it
            band = np.abs(((x - y + 10) % 44) - 22)
            L += np.where(band < 3, 0.75, np.where(band < 5, 0.25, 0))
        else:  # facets: irregular crystal cells (the nearest of fixed points), each lit its own way
            cells = spots(7, 120, (-26, 20, -5), (26, 80, 5))
            d = np.linalg.norm(p[..., None, :] - cells[None, None], axis=-1)
            which = np.argmin(d, -1)
            L += np.array([1.0, -0.75, 0.5, -0.25, 1.25, 0.0])[which % 6]
        lit_face = np.isin(face, ["up", "south", "west"])
        # sparkles: fixed points on the item's flat front (x, y), so every front face shows them
        star = spots(5 if tier == "gold" else 6, 22, (-24, 20, 0), (24, 78, 0))
        d = np.min(np.linalg.norm(p[..., None, :2] - star[None, None, :, :2], axis=-1), -1)
        L += np.where(lit_face & (d < 0.8), 2.5, np.where(lit_face & (d < 1.6), 1.0, 0))
    return L


def faces(elements, ramps, tier):
    solid = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in elements if not e["rot"]]
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
            mat = e["mat"]
            L = np.full((h, w), FACE_L[face])
            if face in ("up", "down"):
                if w >= 3 and h >= 3 and face == "up":
                    L += 0.25 * ((uu < 1) | (vv < 1) | (uu > w - 1) | (vv > h - 1))
            else:
                if h >= 3:
                    L += 0.5 * (vv < 1) - 0.75 * (vv > h - 1)
                if h >= 4 and not e.get("blade"):
                    L += 0.6 * (0.5 - vv / h)
                if w >= 3:
                    L += 0.25 * (uu < 1) - 0.25 * (uu > w - 1)
            if mat == "head":
                L += finish(tier, p, face, e, uu, vv, w, h)
                if e.get("edge"):
                    L += 0.5
                if e.get("blade"):
                    L += (p[..., 1] - 27) / 45 * 0.75 - 0.25  # brightest toward the point
                    if face in ("south", "north"):
                        L += np.where(8 - np.abs(p[..., 0]) < 1, 0.75 + 0.25 * (tier == "iron"), 0)
            elif mat in ("handle", "cap", "guard"):
                L += finish("wood", p, face, e, uu, vv, w, h)
            if mat == "grip" and face not in ("up", "down"):
                row = np.floor(p[..., 1]).astype(int) % 3
                L += np.where(row == 0, -1.0, np.where(row == 2, 0.5, 0))
            if not e["rot"]:
                L -= 1.25 * occlusion(p, R @ n, R @ du, R @ dv, solid, own=e["name"])
            L = np.clip(L, -2, 2)
            k = np.clip(np.round((np.round(L / STEP) * STEP - KS[0]) / STEP), 0, len(KS) - 1).astype(int)
            col = ramps[mat][k]
            tiles.append(((i, face), np.dstack([col.clip(0, 255), np.full((h, w), 255)]).astype(np.uint8)))
    return tiles


def square(sheet):
    """The engine takes square textures (or vertical strips, for animations)."""
    side = max(sheet.shape[:2])
    sq = np.zeros((side, side, 4), np.uint8)
    sq[:sheet.shape[0], :sheet.shape[1]] = sheet
    return sq


def placed(e):
    """The element's corners in the item's own frame (its tilt applied)."""
    lo, hi = np.array(e["lo"]), np.array(e["hi"])
    return lo, hi, rotm(e["rot"]), np.array(e["origin"] or (0, 0, 0), float)


# ---------------------------------------------------------------- one item

def build(tier, kind, out=None):
    name = f"{tier}_{kind}"
    elements = LAYOUTS[kind]()
    hits = check(elements)
    assert not hits, f"{name}: faces in one plane {hits}"
    head, stick = icon_colors(name)
    wood_hue = HUE["wood"]
    if tier == "diamond":  # near white already: a darker base leaves its highlights room
        head = tuple(np.array(head) * 0.9)
    ramps = {"head": tones(head, HUE[tier]), "guard": tones(np.array(stick) * 0.78, wood_hue),
             "handle": tones(stick, wood_hue), "cap": tones(np.array(stick) * 0.72, wood_hue),
             "grip": tones(np.array(stick) * 0.8, wood_hue)}
    sheet, uv = pack(faces(elements, ramps, tier), 128)
    sheet = square(sheet)
    W = sheet.shape[1]
    assert W <= 128, f"{name}: texture {W} wide, the engine maps only 128 texel for texel"

    # Laid in the item's square like its sprite: handle bottom left, head top right, the whole
    # turned -45 degrees about its middle, so the engine holds it as it holds a tool sprite.
    corners = []
    for e in elements:
        lo, hi, R, o = placed(e)
        for c in np.array(np.meshgrid(*zip(lo, hi))).T.reshape(-1, 3):
            corners.append((c - o) @ R.T + o)
    corners = np.array(corners)
    C = (corners.min(0) + corners.max(0)) / 2
    C[2] = 0
    TURN = -45.0
    Rz = rotm((0, 0, TURN))
    K = 16 / 64  # Java units (16 a block) per texel

    java = {"textures": {"t": f"base:item/{name}_model", "particle": f"base:item/{name}"}, "elements": []}
    quads = []
    for i, e in enumerate(elements):
        lo, hi, R, o = placed(e)
        mid = (lo + hi) / 2
        world_mid = C + Rz @ (o + R @ (mid - o) - C)
        half = (hi - lo) / 2
        angle = (e["rot"][2] if e["rot"] else 0) + TURN
        j = lambda v: [round(float(a), 4) for a in (np.array(v) - C) * K + 8]  # noqa: E731
        fs = {}
        for face in FACES:
            if (i, face) in uv:
                u0, v0, u1, v1 = uv[(i, face)]
                fs[face] = {"texture": "#t", "uv": [round(c / W * 16, 4) for c in (u0, v0, u1, v1)]}
        d = {"name": e["name"], "from": j(world_mid - half), "to": j(world_mid + half), "faces": fs}
        if abs(angle) > 1e-6:
            d["rotation"] = {"angle": round(angle, 4), "axis": "z", "origin": j(world_mid)}
        java["elements"].append(d)
        if out:
            Rw = Rz @ R
            for face in FACES:
                if (i, face) not in uv:
                    continue
                corner, du, dv, w, h, nrm = face_frame(lo, hi, face)
                pts = [corner, corner + du * w, corner + du * w + dv * h, corner + dv * h]
                pts = [C + Rz @ (o + R @ (q - o) - C) for q in pts]
                u0, v0, u1, v1 = uv[(i, face)]
                quads.append({"p": [list(map(float, q - C)) for q in pts], "uv": [u0 / W, v0 / W, u1 / W, v1 / W],
                              "n": list(map(float, Rw @ nrm))})
    json.dump(java, open(os.path.join(ASSETS, "models", "item", name + ".json"), "w"), indent=1)
    Image.fromarray(sheet).save(os.path.join(ASSETS, "textures", "item", name + "_model.png"))
    if out:
        Image.fromarray(sheet).save(os.path.join(out, name + ".png"))
        Image.open(os.path.join(ASSETS, "textures", "item", name + ".png")).save(os.path.join(out, name + "_icon.png"))
        json.dump({"quads": quads, "texture": name + ".png"}, open(os.path.join(out, name + ".json"), "w"))
        spec = {"size": W, "elements": [{k: e[k] for k in ("name", "lo", "hi", "rot", "origin")} |
                                        {"faces": {f: uv[(i, f)] for f in FACES if (i, f) in uv}}
                                        for i, e in enumerate(elements)]}
        json.dump(spec, open(os.path.join(out, name + "_spec.json"), "w"))
    return name, len(elements), W


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else None
    if out:
        os.makedirs(out, exist_ok=True)
    for tier in TIERS:
        for kind in KINDS:
            name, n, W = build(tier, kind, out)
            print(f"{name}: {n} boxes, {W}x{W} texture")
