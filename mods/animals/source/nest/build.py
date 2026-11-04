#!/usr/bin/env python3
"""Builds the nest (a prop: an entity with no skeleton) with the blocky-model skill's prop tools
and the game's house style: `../../assets/models/nest.geo.json` and `textures/entity/nest/nest.png`.

    ../dog/chihuahua/.venv/bin/python build.py              (the chosen variant, into the mod)
    ../dog/chihuahua/.venv/bin/python build.py --variants   (every variant into out/<variant>/)

The real thing: Velociraptor's nests aren't known. Its close relative Troodon dug a dish about a
meter across with a pronounced raised rim of sediment, and set its eggs half into the ground,
steeply tilted, their tops toward the middle; emus line such a scrape with grass, bark and leaves.
So the nest is an earthen dish in a sloped rim (each side four stacked boxes, narrower going up,
so it slopes in and out like heaped earth), about 1 m across and 12 cm high at the crest, the dish
a little deeper in the middle where the claws dug. Variants:

- bare: the troodontid scrape: dry crust on the crest, clods and grit, claw scrapes in the dish,
  pebbles kicked out over the rim;
- lined: the same, the dish lined with dry grass and a few leaves (an emu's);
- grassy: tufts of dry grass on the rim, a forked twig, pebbles.
"""

import json
import math
import os
import subprocess
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
SKILL = os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts")
sys.path.insert(0, SKILL)
from house import SCALE, TX, material  # noqa: E402
from prop import el, entity_spec, paint, rotm  # noqa: E402

ASSETS = os.path.join(HERE, "..", "..", "assets")
OUT = os.path.join(HERE, "out")
VARIANTS = ("bare", "lined", "grassy")
CHOSEN = "bare"  # chosen by Ronald, 2026-11-02

SOIL, DARK, CRUST, STRAW, STRAW_DARK, TWIG, STONE, LEAF, GRASS, GROOVE, SOIL_LO, SOIL_HI = range(12)
MATERIALS = [material("soil"), material("soil_dark"), material("soil", (176, 140, 98)), material("straw"),
             material("straw", (150, 116, 66)), material("twig"), material("stone"), material("straw", (150, 100, 58)),
             material("straw", (176, 166, 100)), material("soil", (92, 66, 44)), material("soil", (142, 106, 72)),
             material("soil", (168, 130, 90))]
SOFT = (STRAW, STRAW_DARK, GRASS)


# ------------------------------------------------------------------ noise (organic, never a grid)

def _hash(ix, iy, iz, seed):
    h = (ix * 374761393 + iy * 668265263 + iz * 2147483647 + seed * 1274126177) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 1103515245) & 0xFFFFFFFF
    return ((h ^ (h >> 16)) & 0xFFFFFFFF) / 4294967296.0


def noise(p, scale, seed):
    """Smooth value noise 0..1 at points `p` (..., 3), features about 1/`scale` units across."""
    q = p * scale
    i = np.floor(q).astype(np.int64)
    f = q - i
    f = f * f * (3 - 2 * f)
    out = 0
    for dx in (0, 1):
        for dy in (0, 1):
            for dz in (0, 1):
                w = (f[..., 0] if dx else 1 - f[..., 0]) * (f[..., 1] if dy else 1 - f[..., 1]) * (f[..., 2] if dz else 1 - f[..., 2])
                out = out + w * _hash(i[..., 0] + dx, i[..., 1] + dy, i[..., 2] + dz, seed)
    return out


def grit(p, seed):
    """Per texel 0..1, the same wherever a texel's center falls (single specks)."""
    i = np.floor(p).astype(np.int64)
    return _hash(i[..., 0], i[..., 1], i[..., 2], seed)


def strands(p, count, seed, length=(4, 9), region=24.0):
    """Which texels lie on one of `count` straight strands (grass stems laid flat), seen from
    above: (on, darker) masks."""
    rng = np.random.default_rng(seed)
    on = np.zeros(p.shape[:-1], bool)
    dark = np.zeros(p.shape[:-1], bool)
    xz = p[..., [0, 2]]
    for _ in range(count):
        c = rng.uniform(-region, region, 2)
        a = rng.uniform(0, math.pi)
        L = rng.uniform(*length)
        if np.hypot(*c) > region:
            continue
        d = np.array([math.cos(a), math.sin(a)])
        t = np.clip((xz - c) @ d, -L / 2, L / 2)
        hit = np.linalg.norm(xz - (c + t[..., None] * d), axis=-1) < 0.55
        on |= hit
        if rng.random() < 0.35:
            dark |= hit
    return on, dark


def stems(p, count, seed, region=(0, 21), length=(7, 13)):
    """Grass stems laid round a dish, seen from above: each runs roughly along the circle through
    its middle (as a bird or dinosaur turns while lining a scrape)."""
    rng = np.random.default_rng(seed)
    on = np.zeros(p.shape[:-1], bool)
    xz = p[..., [0, 2]]
    for _ in range(count):
        r = rng.uniform(*region)
        th = rng.uniform(0, 2 * math.pi)
        c = np.array([r * math.cos(th), r * math.sin(th)])
        a = th + math.pi / 2 + rng.uniform(-0.5, 0.5)
        d = np.array([math.cos(a), math.sin(a)])
        L = rng.uniform(*length)
        t = np.clip((xz - c) @ d, -L / 2, L / 2)
        on |= np.linalg.norm(xz - (c + t[..., None] * d), axis=-1) < 0.6
    return on


# ------------------------------------------------------------------ layout (units: texels, 64 a block)

# A rim's layers from the outside in: (outer edge, inner edge, bottom, top) as distances on the
# side's own axis (negative: toward its outside); each narrower and higher than the one below, so
# the rim slopes in and out like heaped earth.
PROFILE = [(-31, -17, 0.0, 3.0), (-29.5, -19, 0.5, 4.5), (-28, -21, 1.0, 6.0), (-26.5, -22.5, 1.5, 7.5)]


def rim_side(name, axis, sign, half, rise):
    """A rim side: north/south (`axis` "x", running along x) or west/east, `sign` -1 on the
    negative side; its crest `rise` higher or lower than the others'."""
    E = []
    for k, (o, i, b, t) in enumerate(PROFILE):
        h = half - k
        lo, hi = (o, i) if sign < 0 else (-i, -o)
        top = t + rise * (k + 1) / len(PROFILE)
        box = ((-h, b, lo), (h, top, hi)) if axis == "x" else ((lo, b, -h), (hi, top, h))
        E.append(el(f"rim_{name}{k}", *box, part="rim", layer=k))
    return E


def rim_corner(name, sx, sz, rise):
    """A corner of the octagon: the same profile on the diagonal (`sx`, `sz`), laid along x and
    turned so it runs along the rim."""
    E = []
    ux, uz = sx * math.sqrt(0.5), sz * math.sqrt(0.5)
    angle = -45 if sx * sz < 0 else 45
    for k, (o, i, b, t) in enumerate(PROFILE):
        r, w, h = -(o + i) / 2, (i - o) / 2, 8.5 - k
        cx, cz = ux * r, uz * r
        top = t + rise * (k + 1) / len(PROFILE)
        E.append(el(f"rim_{name}{k}", (cx - h, b + 0.25, cz - w), (cx + h, top + 0.25, cz + w), rot=(0, angle, 0),
                    origin=(cx, 0, cz), part="rim", layer=k))
    return E


def elements(variant):
    E = [
        # The dish: four floor slabs around a lower middle where the claws dug.
        # Every overlapping pair differs by half a unit or more at top and bottom (bottoms may sink
        # below the ground, which hides them).
        el("floor_n", (-17, -0.5, -21), (17, 2.5, -9), part="floor"),
        el("floor_s", (-16.5, -0.5, 9), (16.5, 2.5, 21), part="floor"),
        el("floor_w", (-21, -1.0, -9.5), (-9, 2.0, 9.5), part="floor"),
        el("floor_e", (9, -1.0, -9.5), (21, 1.5, 9.5), part="floor"),
        el("floor_mid", (-10, 0.0, -10), (10, 1.0, 10), part="floor"),
    ]
    E += rim_side("n", "x", -1, 13.0, 0.5)
    E += rim_side("s", "x", 1, 12.5, 0.0)
    E += rim_side("w", "z", -1, 12.8, -0.5)
    E += rim_side("e", "z", 1, 13.2, 0.25)
    for name, sx, sz, rise in (("ne", 1, -1, 0.0), ("nw", -1, -1, 0.75), ("se", 1, 1, -0.25), ("sw", -1, 1, 0.4)):
        E += rim_corner(name, sx, sz, rise)
    # Pebbles: a wider bottom and a narrower top, so they read round.
    pebbles = {"bare": [(35, -6, 0.0), (-8, 35, 0.0), (-35, 14, 0.0)], "lined": [(35, 9, 0.0)],
               "grassy": [(35, -3, 0.0), (-34, -16, 0.0)]}[variant]  # kicked out over the rim
    for k, (x, z, y) in enumerate(pebbles):
        E += [el(f"pebble{k}_low", (x - 2.5, y - 0.5, z - 2), (x + 2.5, y + 2, z + 2), part="pebble"),
              el(f"pebble{k}_top", (x - 1.5, y + 1.5, z - 1.25), (x + 1.75, y + 3.75, z + 1.5), part="pebble")]
    if variant == "lined":
        # Dry leaves lying in the dish (well above the floor, so no faces meet).
        for k, (x, z, a) in enumerate(((-6, 4, 30), (7, -5, -50), (2, 13, 80))):
            E.append(el(f"leaf{k}", (x - 3, 2.9, z - 2), (x + 3, 2.9, z + 2), quad="up", rot=(0, a, 0), origin=(x, 2.9, z),
                        cut="leaf", part="leaf", seed=k))
    if variant == "grassy":
        for k, (x, z, a) in enumerate(((-24, 4, 20), (17, -18, -35), (6, 25, 70), (-15, -22, 10), (25, 13, 50))):
            for j, b in enumerate((0, 90)):
                E.append(el(f"tuft{k}_{j}", (x - 7, 4, z), (x + 7, 16, z), quad="south", rot=(0, a + b, 0), origin=(x, 4, z),
                            cut="tuft", part="tuft", seed=k * 2 + j))
        # A forked twig lying over the rim.
        E += [el("twig", (-14, 7.75, -25), (12, 9.75, -23), rot=(0, 12, 0), origin=(0, 7.75, -24), part="twig"),
              el("twig_fork", (2, 8.0, -27), (12, 9.5, -25.75), rot=(0, -28, 0), origin=(2, 8.0, -26.5), part="twig")]
    return E


# ------------------------------------------------------------------ areas (designed, per texel)

def area(e, face, p, uu, vv, h, variant):
    part = e.get("part")
    shape = p.shape[:-1]
    if part == "pebble":
        return np.where(grit(p, 31) < 0.12, SOIL, STONE)  # earth stuck to it
    if part == "twig":
        streak = noise(p * np.array([0.15, 2.0, 2.0]), 1.0, 41) > 0.62  # bark runs along it
        return np.where(streak, DARK, TWIG)
    if part == "leaf":
        return np.where(np.abs(uu - uu.max() / 2) < 0.6, TWIG, LEAF)  # the midrib
    if part == "tuft":
        t = vv / max(h, 1)  # 0 at the top
        return np.where(t > 0.75, STRAW_DARK, np.where(t > 0.4, GRASS, STRAW))
    # Earth like the house's dirt: small soft clumps two to four texels across in shades just
    # either side of the base (never camouflage blotches), a dry crust on the crest, the dish
    # darker toward its damp middle with claw scrapes, now and then a tiny stone; lined, a mat of
    # grass stems laid round the dish.
    clump = noise(p, 0.38, 7)
    a = np.full(shape, SOIL)
    a = np.where(clump > 0.66, SOIL_HI, np.where(clump < 0.32, SOIL_LO, a))
    up = face == "up"
    if part == "rim" and up and e["layer"] == len(PROFILE) - 1:
        a = np.where(clump > 0.45, CRUST, SOIL_HI)
    if part == "floor" and up:
        r = np.hypot(p[..., 0], p[..., 2]) + 2.5 * (noise(p, 0.25, 21) - 0.5)
        a = np.where(r < 12, np.where(clump > 0.6, SOIL_LO, DARK), a)
        for cx, cz, ang in (() if variant == "lined" else ((-5, -3, 0.5), (6, 4, 2.2))):  # claw scrapes
            dx, dz = math.cos(ang), math.sin(ang)
            u = (p[..., 0] - cx) * dx + (p[..., 2] - cz) * dz
            v = -(p[..., 0] - cx) * dz + (p[..., 2] - cz) * dx
            groove = (np.abs(u) < 3.5 - 0.3 * np.abs(v)) & (np.abs(((v + 4.5) % 3) - 1.5) < 0.5) & (np.abs(v) < 4.5)
            a = np.where(groove, GROOVE, a)
        if variant == "lined":
            # A mat: darker straw under, stems on top running round the dish.
            on = stems(p, 60, 5)
            a = np.where(on, STRAW, STRAW_DARK)
    if variant == "lined" and part == "floor" and not up:
        a = np.full(shape, STRAW_DARK)  # the lining runs down the dish's little steps
    if variant == "lined" and part == "rim" and up and e["layer"] < len(PROFILE) - 1:
        a = np.where(stems(p, 50, 6, region=(20, 30)) & (noise(p, 0.2, 3) > 0.4), STRAW, a)
    g = grit(p, 13)
    return np.where(g > 0.994, STONE, a)


def cut(e, p):
    """Quads' outlines, in each quad's own frame (its tilt undone): a tuft fans out from its base
    in blades that narrow and droop at the tips; a leaf is a pointed oval."""
    R = rotm(e["rot"])
    o = np.array(e["origin"], float)
    q = (p - o) @ R  # untilted, relative to the origin
    x, y, z = q[..., 0], q[..., 1], q[..., 2]
    if e["cut"] == "leaf":
        return (x / 3.0) ** 2 + (z / np.maximum(1.9 * (1 - 0.6 * np.abs(x) / 3), 0.2)) ** 2 <= 1
    rng = np.random.default_rng(e["seed"])
    keep = np.zeros(x.shape, bool)
    for _ in range(7):
        base = rng.uniform(-1.8, 1.8)
        lean = rng.uniform(-0.55, 0.55)  # sideways per unit up
        height = rng.uniform(7, 11.5)
        bend = rng.uniform(-0.04, 0.04)  # the tip droops sideways
        t = np.clip(y / height, 0, 1)
        center = base + lean * y + bend * y * y
        width = 1.1 * (1 - t) + 0.35
        keep |= (y >= 0) & (y <= height) & (np.abs(x - center) <= width)
    return keep


# ------------------------------------------------------------------ build

def egg_module():
    import importlib.util

    spec = importlib.util.spec_from_file_location("egg_build", os.path.join(HERE, "..", "egg", "build.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def with_eggs(E, egg):
    """Four eggs in the dish as the game lays them (`eggs.rs`): the middle and round it, a quarter
    sunk, leaning in toward the middle; for looking at the nest as it is used."""
    spots = [(0.0, 0.0)] + [(12.8 * math.cos(a), 12.8 * math.sin(a)) for a in (0.4, 2.5, 4.4)]
    for k, (x, z) in enumerate(spots):
        yaw = math.degrees(math.atan2(-x, -z)) if k else 30.0
        E += egg.elements(yaw=yaw, at=(x, 1.0 - 2.5, z), name=f"egg_{k}_")
    return E


def build(variant, out, eggs=False):
    os.makedirs(out, exist_ok=True)
    E = elements(variant)
    egg = egg_module() if eggs else None
    if eggs:
        E = with_eggs(E, egg)
    # The rim's steps lit as one sloped mound: light falls off from its crest to the ground.
    def down(e, face, p):
        if e.get("part") == "egg":
            return np.clip((e["at"][1] + 10 - p[..., 1]) / 10, 0, 1)
        return np.clip((8.5 - p[..., 1]) / 8.5, 0, 1) if e.get("part") in ("rim", "floor") else None

    mats = MATERIALS + (egg.materials(egg.CHOSEN) if eggs else [])

    def paint_area(e, f, p, uu, vv, h):
        if e.get("part") == "egg":
            return egg.area(p, egg.CHOSEN, origin=e["at"], yaw=e["yaw"]) + len(MATERIALS)
        return area(e, f, p, uu, vv, h, variant)

    sheet, uv = paint(E, paint_area, mats, coat=SOFT, cut=cut, width=256, seed=3, down=down, smooth=1, tx=TX)
    Image.fromarray(sheet, "RGBA").save(os.path.join(out, "texture.png"))
    spec = entity_spec(E, uv, sheet, "nest")
    with open(os.path.join(out, "spec.json"), "w") as f:
        json.dump(spec, f)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(out, "spec.json")], check=True)
    return spec


if __name__ == "__main__":
    from to_bedrock import export  # noqa: E402

    if "--variants" in sys.argv:
        for v in VARIANTS:
            build(v, os.path.join(OUT, v))
            build(v, os.path.join(OUT, f"{v}_eggs"), eggs=True)
            print("built", v)
    else:
        spec = build(CHOSEN, OUT)
        made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "nest", SCALE, "nest/nest", pose_files={"": {}})
        print("wrote", made, "(", CHOSEN, ")")
