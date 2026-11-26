#!/usr/bin/env python3
"""Builds every egg look (props with no skeleton) with the blocky-model skill's prop tools: one
model and texture per family of eggs, so a raptor's egg reads as a raptor's at a glance
(Ronald, 2026-11-06: "I don't want them to all look the same"). The research behind each look is
`reports/Eggs by species.md`; which species lays which look, and how long its egg is, are
`animals_core::eggs` (the game scales a look to each species' egg).

    ~/.venvs/blocky/bin/python build.py            (every look into the mod: models/egg_<look>.geo.json,
                                                    textures/entity/egg/<texture>.png)
    ~/.venvs/blocky/bin/python build.py --out DIR  (only into DIR, for review: spec, texture, renders)

Each look: the egg's real length and width (units, 1/64 m), its shape (`eggshape.profile`: how
much wider the blunt end is, how blunt the ends are), how it lies (`tilt`: 0 standing on its
pointed end, 90 lying), its colors and its shell's marks (`area`), and for roe a clump of eggs.
Eggs sit with Minecraft-style blocks and items as its own and its mods' eggs do
(`reports/Eggs in Minecraft mods.md`; Ronald, 2026-11-06: "a bit too different"): painted at the
house's 32 texels a block, drawn about five times life (1.6 px a real cm, 5 to 28 px long), built
of 1 to 6 stacked layers stepping a pixel a side; species differ by color and marks on these few
shapes.
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
sys.path.insert(0, HERE)
from house import SCALE, TX, material  # noqa: E402
from paint import tones  # noqa: E402
from prop import entity_spec, paint  # noqa: E402

import eggshape as es  # noqa: E402

ASSETS = os.path.join(HERE, "..", "..", "assets")
OUT = os.path.join(HERE, "out")
CM = 0.64  # units a centimetre


def shell(rgb):
    return material("shell", rgb)


def stony(rgb):
    return tones(rgb, 0.74)


# ------------------------------------------------------------------ the looks' shell marks
# Each `area(e, u, v, th, x, z)` gives a material per texel (0 the shell), from where it sits on
# its egg: u round it (units at its widest), v along it from the pointed end, th the angle round,
# x and z across it.


def grid(e, u, v):
    """The texel's column round the egg and row along it (whole texels)."""
    tx = e["egg"]["tx"]
    return np.floor(u * tx).astype(int), np.floor(v * tx).astype(int)


def noise(c, r, seed):
    return np.vectorize(lambda a, b: es.hashed(a, b, seed))(c, r)


def speckled(e, u, v, th, x, z):
    """A dromaeosaur's: pale sky blue-green (Deinonychus's shell held both of birds' egg
    pigments), a lumpy net of paler bumps, brown blotches gathered toward the blunt end over
    lilac-grey ones lying deeper, and brown speckles all over."""
    g = e["egg"]
    L, W = g["L"], g["W"]
    rng = np.random.default_rng(3)
    c, r = grid(e, u, v)
    a = np.where(noise(c, r, 9) > 0.8, 3, 0)
    a = es.stamp(a, u, v, W, es.blobs(rng, 4, (0.6 * L, 0.88 * L), (0.32 * W / 4.7 * 2, 0.45 * W / 4.7 * 2), W), 1)
    a = es.stamp(a, u, v, W, es.blobs(rng, 6, (0.64 * L, 0.95 * L), (0.26 * W / 4.7 * 2, 0.4 * W / 4.7 * 2), W), 2)
    a = np.where(noise(c, r, 4) > 0.93, 2, a)
    return a


def ridged(e, u, v, th, x, z):
    """An oviraptorid's (elongatoolithid): fine ridges running along the egg, broken into dashes
    and nodes (lineartuberculate), lighter crests beside darker grooves; plainer at both ends."""
    g = e["egg"]
    L = g["L"]
    c, r = grid(e, u, v)
    dash = noise(c, r // 3, 11) > 0.22
    mid = (v > 0.1 * L) & (v < 0.9 * L)
    a = np.where((c % 3 == 0) & dash & mid, 1, 0)
    return np.where((c % 3 == 1) & dash & mid, 2, a)


def leathery(e, u, v, th, x, z):
    """A soft-shelled egg (Protoceratops, Norell et al. 2020): pale buff and matt, a dent where
    the thin shell gave (a crescent of shadow with a lit lip), a crease or two, sand stuck to the
    side it lay on."""
    g = e["egg"]
    L, W = g["L"], g["W"]
    rng = np.random.default_rng(7)
    a = np.zeros(u.shape, int)
    a = es.stamp(a, u, v, W, es.blobs(rng, 4, (0.2 * L, 0.8 * L), (0.5, 0.8), W, rough=0.6), 1)
    # The dent: on the side facing up and out, a crescent.
    du, dv = es.wrap(u - W * 0.9, 4 * W), v - 0.55 * L
    d = np.hypot(du / 1.0, dv / 1.6)
    a = np.where((d < 1.1) & (d > 0.75) & (dv > -0.2), 2, a)
    a = np.where((d <= 0.75) & (dv > 0.3), 1, a)
    for _ in range(2):
        bu, bv, ln, ang = rng.uniform(0, 4 * W), rng.uniform(0.25 * L, 0.75 * L), rng.uniform(0.8, 1.4), rng.uniform(0, math.pi)
        du, dv = es.wrap(u - bu, 4 * W), v - bv
        along = du * math.cos(ang) + dv * math.sin(ang)
        across = -du * math.sin(ang) + dv * math.cos(ang)
        a = np.where((np.abs(along) < ln) & (np.abs(across) < 0.5 / g["tx"]), 2, a)
    c, r = grid(e, u, v)
    low = np.cos(th) < -0.6
    return np.where(low & (noise(c, r, 5) > 0.4), 3, a)


def mottled(e, u, v, th, x, z):
    """An ankylosaur's (no egg known: our pick, `reports/Eggs by species.md`): round, leathery,
    dull olive-grey, with dark earth mottling and paler patches, earthier toward the bottom."""
    g = e["egg"]
    L, W = g["L"], g["W"]
    rng = np.random.default_rng(31)
    a = np.zeros(u.shape, int)
    a = es.stamp(a, u, v, W, es.blobs(rng, 6, (0.1 * L, 0.95 * L), (0.6, 1.0), W, rough=0.6), 2)
    a = es.stamp(a, u, v, W, es.blobs(rng, 7, (0.0, 0.85 * L), (0.4, 0.7), W, rough=0.6), 1)
    c, r = grid(e, u, v)
    a = np.where((v < 0.3 * L) & (noise(c, r, 6) > 0.55), 3, a)
    return a


def netted(e, u, v, th, x, z):
    """An ornithopod's (hadrosaur-like, spheroolithid): big, near round, warm ivory, its shell a
    fine net of small raised bumps, each lit above and shadowed below."""
    c, r = grid(e, u, v)
    a = np.zeros(u.shape, int)
    cc, rr = (c + (r // 2) % 2) % 2, r % 2
    keep = noise(c // 2, r // 2, 23) > 0.15
    a = np.where(keep & (cc == 0) & (rr == 1), 1, a)  # the bump, lit
    return np.where(keep & (cc == 0) & (rr == 0), 2, a)  # its shadow under it


def knobbed(e, u, v, th, x, z):
    """A sauropod's (megaloolithid, Auca Mahuevo): a ball, chalk white, covered in coarse round
    knobs set close on offset rows: each a lit texel, a shadow under it."""
    c, r = grid(e, u, v)
    cell_r = r // 3
    off = cell_r % 2
    cell_c = (c + off) // 3
    cc, rr = (c + off) % 3, r % 3
    keep = noise(cell_c, cell_r, 21) > 0.06
    a = np.where(keep & (cc == 1) & (rr == 2), 1, 0)  # the knob
    a = np.where(keep & (cc == 1) & (rr == 1), 2, a)  # its shadow under it
    return a


def banded(e, u, v, th, x, z):
    """A crocodilian's: hard, smooth, both ends alike; the chalky white band round its middle
    that a living egg shows, creamier ends, brown stains from the nest's rotting plants."""
    g = e["egg"]
    L, W = g["L"], g["W"]
    rng = np.random.default_rng(13)
    a = np.where(np.abs(v - 0.5 * L) < 0.21 * L, 1, 0)
    stains = [b for b in es.blobs(rng, 6, (0.04 * L, 0.96 * L), (0.18, 0.3), W, rough=0.25) if abs(b[1] - 0.5 * L) > 0.2 * L]
    return es.stamp(a, u, v, W, stains, 2)


def plain(e, u, v, th, x, z):
    """A lizard's: tiny, white, matt, with one dent."""
    g = e["egg"]
    L, W = g["L"], g["W"]
    du, dv = es.wrap(u - W * 0.9, 4 * W), v - 0.5 * L
    return np.where(np.hypot(du, dv / 1.4) < 0.35, 1, 0)


def bead(e, u, v, th, x, z):
    """Roe: round and glassy, its top lit; a dot on the side facing out: an eyed trout egg's
    young's two eyes, a pacu's yellow yolk, a lungfish's dark egg in its jelly."""
    g = e["egg"]
    L = g["L"]
    a = np.where(v > 0.6 * L, 1, 0)
    dot = g.get("dot")
    if dot:
        f = g["face"]
        along, side = x * math.cos(f) + z * math.sin(f), -x * math.sin(f) + z * math.cos(f)
        mid = np.abs(v - 0.48 * L) < 0.5 / g["tx"] + 0.01
        if dot == "eyes":  # one dark texel on a 3 px bead (the eyed egg's young)
            on = mid & (along > 0) & (np.abs(side) < 0.5 / g["tx"])
        else:
            on = (np.abs(v - 0.45 * L) < 0.12 * L + 0.5 / g["tx"]) & (along > 0) & (np.abs(side) < 0.12 * L + 0.5 / g["tx"])
        a = np.where(on, 2, a)
    return a


def scaled(e, u, v, th, x, z):
    """The wyvern's (fantasy, our own): a heavy, stone-like egg in overlapping round scales that
    point to its pointed end, each row lying over the top of the row below and offset by half a
    scale, like a pine cone's; each scale's lower edge a bright metal rim, a dark shadow under the
    edge of the scales above, a few scales flecked with a second jewel color; the tip and the crown
    in dark small scales."""
    g = e["egg"]
    L, tx = g["L"], g["tx"]
    per = 4 * g["W"] * tx  # texels round the egg
    sw = per / max(4, round(per / 11.0))  # a scale's width: a whole number of them round
    sh = sw * 0.62  # rows' spacing
    R = sw * 0.62  # the scale's radius
    col, row = u * tx, v * tx
    j = np.floor(row / sh).astype(int)
    cands = []  # (row, scale id, distance from its middle) of every scale that may cover a texel
    for dk in (0, 1, 2, -1):
        k = j - dk
        off = (k % 2) * 0.5
        ci = np.round(col / sw - off).astype(int)
        for dc in (-1, 0, 1):
            cc = ci + dc
            dx = es.wrap(col - (cc + off) * sw, per)
            cands.append((k, cc * 7919 + k, np.hypot(dx, row - (k * sh + R)), row < k * sh + R))
    # A texel shows the highest row's scale over it (each row lies over the one below).
    owner = np.full(u.shape, -10**6)
    dist = np.zeros(u.shape)
    lower = np.zeros(u.shape, bool)
    cid = np.zeros(u.shape, int)
    for k, i, d, lo in cands:
        take = (d < R) & (k > owner)
        owner, dist = np.where(take, k, owner), np.where(take, R - d, dist)
        lower, cid = np.where(take, lo, lower), np.where(take, i, cid)
    # How close the edge of a scale above is (its shadow falls on this one).
    above = np.full(u.shape, 99.0)
    for k, i, d, lo in cands:
        above = np.where(k > owner, np.minimum(above, d - R), above)
    a = np.zeros(u.shape, int)
    rim = (dist < 1.0) & lower
    a = np.where(rim, 1, a)
    a = np.where(~rim & (above < 0.9), 2, a)
    fleck = noise(cid, cid // 3, 41) > 0.9
    a = np.where(fleck & (a == 0), 3, a)
    c, r = grid(e, u, v)  # stone grain: a few darker texels in the scales
    a = np.where((a == 0) & (noise(c, r, 43) > 0.9), 4, a)
    return np.where((v < 0.05 * L) | (v > 0.96 * L), 2, a)


def bird(e, u, v, th, x, z):
    """A bird's egg, by its look's `marks`: `blotch` (count, size in units, how much they gather
    at the blunt end 0..1), `deep` (paler lilac-grey marks under them), `speck` (the share of
    texels speckled), `cap` (a wreath of marks round the blunt end). Materials: shell, deep
    marks, marks, a second mark color."""
    g = e["egg"]
    L, W, m = g["L"], g["W"], g["marks"]
    rng = np.random.default_rng(m.get("seed", 5))
    a = np.zeros(u.shape, int)
    c, r = grid(e, u, v)

    def spots(n, size, gather):
        lo = 0.05 + 0.6 * gather
        return es.blobs(rng, n, (lo * L, 0.97 * L), size, W, rough=0.45)

    if m.get("deep"):
        a = es.stamp(a, u, v, W, spots(m["deep"], m.get("deep_size", (0.35, 0.6)), m.get("gather", 0.3)), 1)
    if m.get("blotch"):
        a = es.stamp(a, u, v, W, spots(m["blotch"], m.get("size", (0.3, 0.55)), m.get("gather", 0.3)), 2)
    if m.get("cap"):
        ring = (v > 0.72 * L) & (v < 0.9 * L) & (noise(c, r, 17) > 1 - m["cap"])
        a = np.where(ring, 2, a)
    if m.get("speck"):
        a = np.where(noise(c, r, 19) > 1 - m["speck"], np.where(noise(c, r, 23) > 0.5, 2, 3 if m.get("two") else 2), a)
    return a


# ------------------------------------------------------------------ the looks

LOOKS = {
    # Velociraptor (both species) and Utahraptor (scaled up): a long oval, one end a little
    # narrower, stood nearly upright pointed end down, leaning toward the nest's middle.
    "dromaeosaur": dict(L=9 * CM, W=4.5 * CM, asym=0.18, tilt=20, area=speckled,
                        mats=[shell((150, 198, 196)), shell((158, 150, 170)), material("shell", (110, 72, 48)),
                              shell((176, 214, 210))]),
    # Citipati: the longest, ends alike, dark teal, ridged; in pairs in a ring, lying inclined,
    # the top end toward the middle.
    "oviraptorid": dict(L=18 * CM, W=6.5 * CM, blunt=2.3, tilt=65, area=ridged,
                        mats=[shell((52, 108, 106)), shell((92, 148, 140)), shell((32, 70, 74))]),
    # Protoceratops (Psittacosaurus scaled down): soft, slim, pale buff, buried lying in sand.
    "ceratopsian": dict(L=10.1 * CM, W=5.8 * CM, tilt=88, area=leathery, sink=0.3,
                        mats=[shell((226, 210, 174)), shell((208, 190, 152)), shell((154, 134, 104)),
                              material("soil", (200, 172, 124))]),
    # Gastonia and Pinacosaurus (our pick): near round, leathery, dull olive-grey, mottled.
    "ankylosaur": dict(L=12 * CM, W=10.5 * CM, asym=0.05, tilt=30, area=mottled, sink=0.3,
                       mats=[shell((136, 138, 112)), shell((114, 110, 86)), shell((152, 154, 128)), material("soil", (126, 106, 78))]),
    # Iguanacolossus: big, near round, warm ivory, a fine net of bumps.
    "ornithopod": dict(L=16 * CM, W=14 * CM, asym=0.05, tilt=20, area=netted,
                       mats=[shell((228, 212, 176)), shell((244, 234, 208)), shell((180, 160, 124))]),
    # Cedarosaurus: a ball, chalk white, coarse knobs, half sunk in its pit.
    "sauropod": dict(L=13 * CM, W=13 * CM, tilt=0, area=knobbed, sink=0.6,
                     mats=[stony((222, 220, 210)), stony((242, 240, 232)), stony((168, 162, 152))]),
    # Caimans and the neosuchian: small, smooth, both ends alike, a chalky band, lying.
    "crocodilian": dict(L=6.8 * CM, W=4.3 * CM, blunt=2.0, tilt=90, area=banded,
                        mats=[shell((212, 194, 164)), shell((248, 246, 238)), material("soil", (140, 102, 70))]),
    # Priscagama: tiny, white, matt, dented, lying.
    "lizard": dict(L=1.6 * CM, W=1.0 * CM, tilt=90, area=plain, sink=0.1,
                   mats=[shell((236, 230, 214)), shell((196, 188, 168))]),
    # Fish: a clutch of a few beads (each 3 px, a lungfish's jelly ball 5: no egg under 2 x 3
    # px, `reports/Eggs in Minecraft mods.md`); the four tell apart by color and how they lie
    # (a heap in gravel, a loose scatter, jelly balls).
    "roe_trout": dict(roe=dict(n=4, d=1.5, px=3, dot="eyes"), area=bead,
                      mats=[shell((232, 124, 50)), shell((250, 172, 104)), material("shell", (40, 30, 36))]),
    "roe_pacu": dict(roe=dict(n=4, d=1.25, px=3, dot="yolk", spread=2.6), area=bead,
                     mats=[shell((202, 214, 196)), shell((232, 240, 230)), shell((228, 196, 80))]),
    "roe_lycoptera": dict(roe=dict(n=4, d=1.25, px=3, spread=2.6), area=bead,
                          mats=[shell((222, 200, 136)), shell((242, 228, 182)), shell((150, 130, 80))]),
    "roe_lungfish": dict(roe=dict(n=2, d=2.4, px=5, dot="yolk", spread=1.9), area=bead,
                         mats=[shell((186, 208, 210)), shell((222, 236, 236)), material("shell", (50, 78, 46))]),
    # Birds (`reports/Eggs by species.md`, Birds): shapes by group, a texture per species; all
    # lie on their side in the nest's cup. Owls: plain white, nearly round, glossy.
    "bird_owl": dict(L=5.6 * CM, W=4.7 * CM, asym=0.08, tilt=80, area=bird, marks={},
                     mats=[shell((238, 236, 228)), shell((214, 210, 200)), shell((200, 196, 186)), shell((200, 196, 186))]),
    # Songbirds (at the smallest an egg is drawn, 5 px): the house sparrow greyish
    # white, densely speckled grey-brown; the snow bunting bluish white with red-brown and lilac
    # spots and scrawls gathered at the blunt end.
    "bird_sparrow": dict(L=2.2 * CM, W=1.6 * CM, asym=0.22, tilt=80, area=bird,
                         marks=dict(speck=0.4, two=True, blotch=3, size=(0.2, 0.3), gather=0.5),
                         mats=[shell((214, 214, 204)), shell((168, 164, 160)), material("shell", (112, 98, 84)), shell((150, 146, 138))]),
    "bird_bunting": dict(L=2.2 * CM, W=1.6 * CM, asym=0.22, tilt=80, area=bird,
                         marks=dict(deep=4, deep_size=(0.2, 0.3), blotch=7, size=(0.15, 0.25), gather=0.75, speck=0.06),
                         mats=[shell((220, 230, 226)), shell((176, 160, 184)), material("shell", (132, 70, 50)), shell((132, 70, 50))]),
    # The raven: green (kept off blue, so it doesn't read as a raptor's), blotched and streaked
    # olive-brown, more at the blunt end.
    "bird_raven": dict(L=5.0 * CM, W=3.4 * CM, asym=0.22, tilt=80, area=bird,
                       marks=dict(deep=5, blotch=10, size=(0.3, 0.5), gather=0.4, speck=0.08),
                       mats=[shell((138, 170, 122)), shell((130, 140, 128)), material("shell", (92, 80, 52)), shell((92, 80, 52))]),
    # The herring gull: olive-brown, boldly blotched dark brown-black.
    "bird_gull": dict(L=7.1 * CM, W=4.9 * CM, asym=0.22, tilt=80, area=bird,
                      marks=dict(deep=4, blotch=9, size=(0.35, 0.6), gather=0.35, speck=0.05),
                      mats=[shell((164, 150, 108)), shell((136, 120, 98)), material("shell", (58, 44, 36)), shell((58, 44, 36))]),
    # The golden eagle: whitish, blotched and washed red-brown.
    "bird_eagle": dict(L=7.5 * CM, W=5.8 * CM, asym=0.22, tilt=80, area=bird,
                       marks=dict(deep=6, blotch=8, size=(0.4, 0.7), gather=0.3, speck=0.06),
                       mats=[shell((232, 224, 208)), shell((190, 160, 150)), material("shell", (150, 76, 46)), shell((150, 76, 46))]),
    # The savanna hawk: bluish white, a few faint pale brown spots.
    "bird_hawk": dict(L=5.7 * CM, W=4.5 * CM, asym=0.22, tilt=80, area=bird,
                      marks=dict(blotch=4, size=(0.2, 0.35), gather=0.3),
                      mats=[shell((226, 234, 234)), shell((206, 200, 190)), shell((196, 168, 132)), shell((196, 168, 132))]),
    # The turkey vulture: creamy white, heavy dark brown and lavender blotches wreathing the
    # blunt end.
    "bird_vulture": dict(L=7.1 * CM, W=4.9 * CM, asym=0.22, tilt=80, area=bird,
                         marks=dict(deep=6, blotch=10, size=(0.35, 0.6), gather=0.75, cap=0.45),
                         mats=[shell((236, 228, 210)), shell((170, 156, 180)), material("shell", (92, 56, 40)), shell((92, 56, 40))]),
    # The Cretaceous birds (Gobioolithus, the eggs once taken for Gobipteryx's; Confuciusornis's by
    # inference): long, slim, plain pale, each stood upright and half buried, as those were found.
    "bird_mesozoic": dict(L=3.6 * CM, W=2 * CM, asym=0.06, tilt=8, sink=0.45 * 3.6 * CM, area=bird,
                          marks=dict(deep=3, deep_size=(0.2, 0.35), gather=0.0),
                          mats=[shell((226, 220, 204)), shell((204, 196, 178)), shell((204, 196, 178)), shell((204, 196, 178))]),
    # The wyvern (fantasy): big, heavy, stone-like, scaled, metal-rimmed; one, on bare rock.
    "wyvern": dict(L=50 * CM, W=34 * CM, asym=0.25, tilt=12, area=scaled, px=28,
                   mats=None),
}

# The wyvern egg's colorings to choose from (scale, rim, shadow, fleck, grain): our own jewel tones.
WYVERN = {
    "teal_silver": [stony((40, 92, 96)), stony((206, 210, 204)), stony((20, 34, 40)), stony((150, 60, 120)), stony((32, 78, 82))],
    "garnet_gold": [stony((112, 30, 38)), stony((226, 182, 92)), stony((40, 16, 22)), stony((214, 120, 40)), stony((94, 24, 32))],
    "amethyst_copper": [stony((86, 52, 112)), stony((214, 132, 82)), stony((32, 20, 44)), stony((60, 140, 130)), stony((72, 42, 96))],
    "charcoal_rust": [stony((50, 46, 54)), stony((190, 110, 66)), stony((24, 20, 26)), stony((176, 50, 40)), stony((40, 36, 44))],
}
WYVERN_CHOSEN = "teal_silver"  # Ronald, 2026-11-06: "Teal-silver"
LOOKS["wyvern"]["mats"] = WYVERN[WYVERN_CHOSEN]
# Species whose egg is far bigger or smaller than its family's get the family's look at their own
# size (scaling a model would change its texel density, `reports/Eggs in Minecraft mods.md`).
LOOKS["dromaeosaur_large"] = dict(LOOKS["dromaeosaur"], L=18 * CM, W=8 * CM, px=24)  # Utahraptor (under the wyvern's 28)
LOOKS["ceratopsian_small"] = dict(LOOKS["ceratopsian"], L=7 * CM, W=4 * CM)  # Psittacosaurus


def roe_spots(n, d, spread=None, seed=1):
    """Where a clutch's beads lie: a heap (one in the middle, a ring round it touching, any more on
    top), or with `spread` a loose scatter on the ground."""
    rng = np.random.default_rng(seed)
    if spread:
        spots = []
        for k in range(n):
            a = k / n * 2 * math.pi + rng.uniform(-0.3, 0.3)
            r = spread * (0.45 + 0.55 * ((k * 5) % 3) / 2) if n > 2 else spread / 2
            spots.append((math.cos(a) * r, 0.0, math.sin(a) * r))
        return spots
    spots = [(0.0, 0.0, 0.0)]
    for k in range(min(n - 1, 6)):
        a = k / 6 * 2 * math.pi + rng.uniform(-0.2, 0.2)
        spots.append((math.cos(a) * d * 0.95 + rng.uniform(-0.07, 0.07), 0.0, math.sin(a) * d * 0.95 + rng.uniform(-0.07, 0.07)))
    for k in range(n - 7):
        a = (k + 0.5) / max(1, n - 7) * 2 * math.pi
        spots.append((math.cos(a) * d * 0.5, d * 0.7, math.sin(a) * d * 0.5))
    return spots


PX_PER_CM, PX_MIN, PX_MAX = 1.6, 5, 28  # drawn length: about 5x life, 5 to 28 px (`reports/Eggs in Minecraft mods.md`)


def px_of(look):
    """The egg's drawn length in pixels (texels at the house's 32 a block): 1.6 a real cm, 5 to 28,
    or the look's own `px` (a roe bead's size)."""
    if "roe" in look:
        return look["roe"]["px"]
    return look.get("px") or int(min(PX_MAX, max(PX_MIN, round(PX_PER_CM * look["L"] / CM))))


def mult(look):
    """Layout units a real one: eggs are drawn about five times life (as Minecraft's and its mods'
    are), at the house density, so the marks (laid out in real units) scale with them."""
    real = look["roe"]["d"] if "roe" in look else look["L"]
    return px_of(look) / TX / real


def tx_of(look):
    return TX  # the house density: a texel is a pixel, 32 a block


def slices(px):
    """How many layers an egg `px` long is built of: 1-2 small, 3 medium, 4-6 large (the mods')."""
    return 1 if px <= 3 else 2 if px <= 9 else 3 if px <= 14 else 4 if px <= 20 else 6


def egg_of(look, name="egg", at=(0.0, 0.0, 0.0), yaw=0.0):
    """One egg: whole pixels, steps of a pixel a side, tips at least 2 px, no chamfers."""
    m, px = mult(look), px_of(look)
    W = max(2, round(px * look["W"] / look["L"])) / TX  # units, a whole number of pixels
    return es.egg_elements(name, px / TX, W, look.get("asym", 0.0), look.get("blunt", 2.0), look["tilt"],
                           yaw=yaw, at=at, sink=look.get("sink", 0.0) * m, n=slices(px), step=2 / TX,
                           tip=2 / TX, m=m)


def elements(look):
    if "roe" in look:
        r = look["roe"]
        m, tx = mult(look), tx_of(look)
        E = []
        for i, (x, y, z) in enumerate(roe_spots(r["n"], r["d"], r.get("spread"))):
            d = r["px"] / TX
            # Each bead turned its own way, so no two share a face.
            egg = es.egg_elements(f"egg{i}_", d, d, tilt=0, yaw=17 * i + 5, at=(x * m, (y + 0.013 * i) * m, z * m),
                                  n=slices(r["px"]), step=2 / TX, m=m)
            g = egg[0]["egg"]
            g["dot"] = r.get("dot")
            # The side facing out of the heap (the first bead: toward the viewer's side, +x +z).
            g["face"] = math.atan2(z, x) if (x or z) else math.pi / 4
            g["face"] -= math.radians(g["yaw"]) * -1  # into the bead's own (turned) frame
            E += egg
        return E
    return egg_of(look)


def clutch(look, n=5):
    """For review: `n` eggs as a nest holds them (`eggs.rs`: one in the middle, the rest in a
    ring), each leaning its top toward the middle."""
    m = mult(look)
    E = egg_of(look, "c0_")
    r = (max(0.55 * look["L"], 0.75 * look["W"]) + 0.6) * m
    for k in range(1, n):
        a = (k - 1) / (n - 1) * 2 * math.pi + 0.3
        yaw = math.degrees(math.atan2(-math.cos(a), -math.sin(a))) + 0.7 * k
        E += egg_of(look, f"c{k}_", at=(math.cos(a) * r, 0.011 * k * m, math.sin(a) * r), yaw=yaw)
    return E


def build(name, look, out, mats=None, E=None):
    """Lays out, paints and checks one look into `out` (spec.json, texture.png)."""
    os.makedirs(out, exist_ok=True)
    E = E or elements(look)
    m, tx = mult(look), tx_of(look)
    W = look["roe"]["d"] if "roe" in look else look["W"]
    for e in E:
        e["egg"]["tx"] = tx * m  # texels a real unit (the marks are laid out in real units)
    for e in E:
        e["egg"]["marks"] = look.get("marks", {})

    def area(e, face, p, uu, vv, h):
        u, v, th, x, z = es.local(e, p)
        return look["area"](e, u, v, th, x, z)

    def down(e, face, p):
        u, v, th, x, z = es.local(e, p)
        return np.clip((e["egg"]["L"] - v) / e["egg"]["L"], 0, 1)

    width = 64 if tx * W * m < 24 and len(E) < 12 else 128
    sheet, uv = paint(E, area, mats or look["mats"], ground=False, width=width, down=down, tx=tx)
    Image.fromarray(sheet, "RGBA").save(os.path.join(out, "texture.png"))
    spec = entity_spec(E, uv, sheet, name)
    with open(os.path.join(out, "spec.json"), "w") as f:
        json.dump(spec, f)
    r = subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(out, "spec.json")],
                       capture_output=True, text=True)
    if r.returncode:
        sys.exit(f"{name}: boxcheck failed\n{r.stdout}")
    no_thin_lines(spec, sheet, E, name)
    return spec, tx


def no_thin_lines(spec, sheet, E, name=""):
    """No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another.
    Meant (the planner's ruling, 11-05, `linecheck_allow.txt`): an egg's round taper, each layer
    narrower than the next toward both ends, so the middle layers' sides show a narrow band above
    or below their neighbours; squaring them off would square the egg. Each egg's tips stay
    checked."""
    from linecheck import boxes_from_spec, lines  # noqa: E402

    eggs = {}
    for e in E:
        eggs.setdefault(id(e["egg"]), []).append(e["name"])
    # The tips' inner ends (under the next slice) show a sliver at the octagon's corners: meant too.
    allow = [n for names in eggs.values() for n in names[1:-1]]
    allow += [f"{names[0]}.up" for names in eggs.values()] + [f"{names[-1]}.down" for names in eggs.values()]
    # An octagonal tip's pulled-in sides show a sliver where its crossing box stops short: meant.
    tips = [t for names in eggs.values() for t in {names[0], names[-1]}]
    allow += [f"{t}.{f}" for t in tips if f"{t}x" in sum(eggs.values(), []) for f in ("north", "south")]
    allow += [f"{t[:-1]}.{f}" for t in tips if t.endswith("x") for f in ("north", "south")]
    thin = [h for h in lines(boxes_from_spec(spec), sheet, density=E[0]["egg"]["tx"] / E[0]["egg"]["m"], allow=allow) if h["kind"] == "line"]
    assert not thin, f"{name}: thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"


def model_of(name):
    return f"egg_{name}"  # each look its own model (linecheck finds `source/egg` and its texture by the name)


def box_of(spec):
    """The hitbox for `animals_core::bodies::BOXES` (blocks): as wide as the geometric mean of
    the model's width and length, as high as its top, the small ones a few cm more to click (the
    rules `core/tests/boxes.rs` checks)."""
    from prop import rotm

    pts = []
    for e in spec["elements"]:
        lo, hi = np.array(e["lo"]), np.array(e["hi"])
        R = rotm(e["rot"]) if e.get("rot") else np.eye(3)
        o = np.array(e["origin"]) if e.get("rot") else np.zeros(3)
        pts += [(np.array([hi[i] if k >> i & 1 else lo[i] for i in range(3)]) - o) @ R.T + o for k in range(8)]
    p = np.array(pts) / 64
    mean, top = math.sqrt(np.ptp(p[:, 0]) * np.ptp(p[:, 2])), p[:, 1].max()
    return round(float(mean + 0.06 if mean < 0.15 else mean), 2), round(float(top + 0.04 if top < 0.15 else top), 2)


if __name__ == "__main__":
    if "--boxes" in sys.argv:  # each model's box, from a review build (`--out DIR` first)
        d = sys.argv[sys.argv.index("--boxes") + 1]
        done = set()
        for name in LOOKS:
            if model_of(name) not in done and os.path.exists(os.path.join(d, name, "spec.json")):
                done.add(model_of(name))
                print(f'    ("{model_of(name)}", {box_of(json.load(open(os.path.join(d, name, "spec.json"))))}),')
        sys.exit()
    out = sys.argv[sys.argv.index("--out") + 1] if "--out" in sys.argv else None
    only = [a for a in sys.argv[1:] if a in LOOKS]
    for name, look in LOOKS.items():
        if only and name not in only:
            continue
        d = os.path.join(out or OUT, name)
        spec, tx = build(name, look, d)
        print(f"{name}: {len(spec["elements"])} boxes, {px_of(look)} px long, texture {spec["texture"]}")
        if out:
            if "roe" not in look:
                build(name, look, os.path.join(out, "clutch", name), E=clutch(look))
            if name == "wyvern":
                for k, m in WYVERN.items():
                    build(name, look, os.path.join(out, "wyvern_colors", k), mats=m)
        else:
            from to_bedrock import export  # noqa: E402

            export(spec, os.path.join(d, "texture.png"), ASSETS, model_of(name), SCALE, f"egg/{name}", pose_files={"": {}})
