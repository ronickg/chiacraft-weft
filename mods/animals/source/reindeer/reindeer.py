#!/usr/bin/env python3
"""The reindeer (caribou, *Rangifer tarandus*): the tundra's big grazer, made the way Hytale makes
creatures (boxes and a hand-painted texture) with the blocky-model skill, from research alone
(README.md), on the deer's build (`../deer/deer.py`): laid out in the deer's units, then drawn
`K` times bigger, stockier, the legs shorter against the body, the muzzle broad and blunt, the
ears small and round. Its colors are rules (see `area`), painted in a wild reindeer's coat.

    ~/.venvs/blocky/bin/python reindeer.py     # writes out/spec.json, out/texture.png

Units are 1/64 of a block (1.56 cm each), y up, it faces +z; painted at the house density, 32
texels a block (`TX` texels a unit), as the blocks and items are. Laid out
in the deer's space (withers 60) and scaled by K = 1.12: the withers 67 up (1.05 m), the body 122
from the nose to the tail (1.9 m). Both sexes carry antlers (the only deer whose females do): a
bull's big, a cow's smaller, by the build gene for horns (`genetics/reindeer.json`).
"""

import json
import math
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
SKILL = os.path.normpath(os.path.join(HERE, "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts"))
sys.path.insert(0, SKILL)
from layers import Recorder, bounds as layer_bounds  # noqa: E402
from paint import EYES, eye_corner, occlusion, pack, paint_face_smooth, stamp_eyes, tones  # noqa: E402
from rig import fit  # noqa: E402
from ground import lowest  # noqa: E402
from house import TX  # noqa: E402

# Texels a unit: 0.5, the house's 32 a block (a house-unit model, scaled by K before painting). 32,
# not finer: the head's side, which carries the eye, is 8 texels by 7 at 32 (house.MIN_FACE is 3).

OUT = os.environ.get("REINDEER_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin
K = 1.12  # the deer's layout drawn this much bigger

# ------------------------------------------------------------------ materials and eyes

FUR, BACK, BELLY, DARK, EARIN, MOUTH, ANTLER, HOOF, TAILTOP, LEG, PALE = range(11)
# FUR the grey-brown coat, BACK darker along the spine and forehead, BELLY the white of the belly,
# rump, the tail's underside and the socks above the hooves, DARK the ears' rims and the nostrils,
# EARIN the ears' inside, ANTLER bone, HOOF dark horn, TAILTOP the tail's top, LEG the dark brown of
# the legs, face and flank band, PALE the cream of the neck, the mane and the chest.
BASE = [(150, 128, 104), (106, 86, 68), (230, 224, 212), (50, 42, 38), (178, 156, 138), (84, 42, 44), (198, 180, 146),
        (58, 50, 46), (128, 106, 84), (86, 68, 54), (214, 204, 184)]
SHADOW = [0.02, 0.02, 0.07, 0.95, 0.97, 0.95, 0.06, 0.74, 0.02, 0.02, 0.06]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (FUR, BACK, BELLY, LEG, PALE)
ROLES = [("coat", 0, 0), ("coat", 0, -1.0), ("coat", 1, 0), "fixed", "fixed", "fixed", "fixed", "fixed", ("coat", 0, -0.5),
         ("coat", 0, -1.75), ("coat", 0.75, 0)]

# A big dark eye on each side of the head: near-black, a warm brown glow below, a catchlight.
EYES.update({
    "reindeer_dark": dict(pattern=[".OOOO.",
                                   "OPPPHO",
                                   "OPPPPO",
                                   "OIIPPO",
                                   ".OOOO."], colors={"O": (32, 24, 22), "P": (16, 12, 12), "H": (226, 228, 232),
                                                      "I": (96, 62, 40)}, halo="below"),
    # At 32 a block the head's side is 8 texels by 7: the eye is 2 by 2, near-black, the catchlight
    # up and forward (toward the light; `_m` is the other side's), the brown glow low and back.
    "reindeer_px": dict(pattern=["HP", "PI"], colors={"P": (16, 12, 12), "H": (226, 228, 232), "I": (96, 62, 40)},
                        halo="none", iris="I"),
    "reindeer_px_m": dict(pattern=["PH", "IP"], colors={"P": (16, 12, 12), "H": (226, 228, 232), "I": (96, 62, 40)},
                          halo="none", iris="I"),
})
EYE_STYLE = os.environ.get("REINDEER_EYES", "reindeer_px")

# In the deer's space (scaled by K when the model is built).
SKULL0 = ((-7.0, 60.0, 32.0), (7.0, 72.0, 46.0))
EYE_Z0, EYE_Y0 = 40.5, 67.6
SKULL = tuple(tuple(v * K for v in c) for c in SKULL0)
EYE_Z, EYE_Y = EYE_Z0 * K, EYE_Y0 * K
LID_X = SKULL[1][0] + 0.4  # the eyelids' hinge plane, just outside the head's side (scaled)

# ------------------------------------------------------------------ the model


def el(name, bone, lo, hi, quad=None, rot=None, origin=None, cut=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad, rot=rot,
                origin=[float(v) for v in origin] if origin is not None else None, cut=cut)


def rotm(rot):
    if not rot:
        return np.eye(3)
    rx, ry, rz = (math.radians(a) for a in rot)
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


def rod(name, bone, start, rot, length, w, back=0.4):
    """A box along +y from `start` turned by `rot` (about `start`), `w` across, reaching `back`
    behind its start (inside what it grows from); and its far end."""
    s = np.array(start, float)
    e = el(name, bone, (s[0] - w / 2, s[1] - back, s[2] - w / 2), (s[0] + w / 2, s[1] + length, s[2] + w / 2), rot=rot, origin=s)
    return e, s + rotm(rot) @ np.array([0, length - 0.6, 0])


def eye_style(side):
    """The eye design on the head's east (side > 0) or west side: mirrored on the west, so both
    catchlights face forward."""
    return EYE_STYLE + "_m" if side < 0 and EYE_STYLE + "_m" in EYES else EYE_STYLE


def skull_grid():
    """The (scaled) skull side's texels: (width, height) and the size of one texel in units (u, v)."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h = max(1, round(TX * (z1 - z0))), max(1, round(TX * (y1 - y0)))
    return w, h, (z1 - z0) / w, (y1 - y0) / h


def eye_center(side):
    """The eye's center on the skull's east (side > 0) or west side, in that face's texels."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    return ((z1 - EYE_Z) if side > 0 else (EYE_Z - z0)) / su, (y1 - EYE_Y) / sv


def eye_box(side):
    """The painted eye on the (scaled) skull's side, in units: z from, z to, top, bottom; rounded
    exactly as `stamp_eyes` places it, so the eyelid covers it to the texel."""
    pat = EYES[eye_style(side)]["pattern"]
    ph, pw = len(pat), len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    u0, v0 = eye_corner(*eye_center(side), pat, w)
    za, zb = (z1 - (u0 + pw) * su, z1 - u0 * su) if side > 0 else (z0 + u0 * su, z0 + (u0 + pw) * su)
    top = y1 - v0 * sv
    return za, zb, top, top - ph * sv


LEGS = ("FL", "FR", "BL", "BR")
FRONT_X, FRONT_Z, BACK_X, BACK_Z = 6.4, 21.0, 6.2, -30.0
# The neck carried forward and low (45 degrees), the head held out in front: a reindeer's way.
HEAD, NECK_PITCH, NECK_LEN, NECK_W = np.array([0.0, 66.0, 38.0]), -45.0, 30.0, 10.4
NECK0 = HEAD - (NECK_LEN - 2.5) * np.array([0, math.sin(math.radians(-NECK_PITCH)), math.cos(math.radians(-NECK_PITCH))])
NECK_ROT = (NECK_PITCH + 90, 0, 0)
EAR = [(70.5, 72.5, 1.8), (72.5, 76.5, 2.6), (76.5, 78.5, 1.7)]  # y0, y1, half width: short and round
EAR_X, EAR_Z = 5.4, (34.0, 35.6)
TAIL0 = np.array([0.0, 53.5, -40.0])  # on the rump's back face: a flap hinged on the surface
ANTLER0 = (3.6, 71.4, 36.5)  # where the left antler's burr sits on the skull


# Rule 7 (Ronald, 11-05, "What the best ones do"): thin parts are cut-out planes, never boxes a
# texel thick. The antlers were a burr and eleven rods a side, most a texel thick or less; now each is
# one cut-out plane on the antler bone, as the deer's (`../deer/deer.py`), drawn in its texture a texel
# thick by an alpha mask (`cut_mask`, "antler") in the old rods' shape and size: from the burr the beam
# sweeps up and back, then forward in a C to the palm with three points; the bez tine forward from low
# on the beam; over the face the brow tine, on the left as the flat 'shovel', on the right a plain point
# (most have one shovel). A texel is 2 units drawn (2 / K in the deer's space). The plane faces sideways,
# turned out with the beam's sweep and leaning out as the beam did.
# The beam and the palm two texels thick (Ronald, 11-05: one texel read wiry, a thin outline loop from
# the side), the points and tines one or two.
ANTLER_ART = {
    "L": ["....#..#..#...",  # rows top down, columns back to front, in texels; B the burr (its lower row in the skull)
          "....#..#..#...",
          "...#########..",  # the palm
          "..##########..",
          ".####.........",
          ".###..........",
          "###...........",
          "##............",
          "##............",
          "##............",
          "###...........",
          ".###..........",
          ".#######......",  # the bez tine, forward off the beam
          "..###.........",
          "...###....##..",  # the shovel over the face
          "....###...##..",
          ".....######...",  # the beam's root and the brow tine
          ".......BB.##..",
          ".......BB....."],
    "R": ["....#..#..#...",
          "....#..#..#...",
          "...#########..",
          "..##########..",
          ".####.........",
          ".###..........",
          "###...........",
          "##............",
          "##............",
          "##............",
          "###...........",
          ".###..........",
          ".#######......",
          "..###.........",
          "...###........",
          "....###.......",
          ".....#####....",  # the plain brow point
          ".......BB.....",
          ".......BB....."]}
ANTLER_T = 2.0 / K  # a texel, in the deer's space
ANTLER_Y0, ANTLER_Z0 = -1.4, -14.3  # the drawing's bottom and back edge from the burr's pivot, in the deer's space
ANTLER_TURN = (0, 19, -20)  # the left plane's turn (rx, ry, rz) about the pivot; the right's mirrored


def antler(side):
    """The antler: one cut-out quad on its bone facing out, exactly the drawing's size, the right one
    a mirrored quad (a west quad, the turn mirrored), so both are lit alike."""
    s = 1 if side == "L" else -1
    o = np.array((s * ANTLER0[0], ANTLER0[1], ANTLER0[2]))
    art = ANTLER_ART[side]
    rows, cols = len(art), len(art[0])
    lo = (o[0], o[1] + ANTLER_Y0, o[2] + ANTLER_Z0)
    hi = (o[0], o[1] + ANTLER_Y0 + ANTLER_T * rows, o[2] + ANTLER_Z0 + ANTLER_T * cols)
    rx, ry, rz = ANTLER_TURN
    return [el(f"antler_{side.lower()}", f"antler_{side}", lo, hi, quad="east" if s > 0 else "west", rot=(rx, s * ry, s * rz),
               origin=tuple(o), cut="antler")]


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 48, -22)), ("spine_1", "spine_0", (0, 46, -7)),
         ("neck", "spine_1", tuple(NECK0)), ("head", "neck", tuple(HEAD)), ("jaw", "head", (0, 60.1, 45.0)),
         ("tail", "spine_0", tuple(TAIL0))]
    for s, n in ((1, "L"), (-1, "R")):
        B += [(f"ear_{n}", "head", (s * EAR_X, 71.0, sum(EAR_Z) / 2)), (f"antler_{n}", "head", (s * ANTLER0[0], ANTLER0[1], ANTLER0[2]))]
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x = FRONT_X * s
            B += [(f"leg_{leg}_upper", "spine_1", (x, 40, FRONT_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 21, FRONT_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 5.0, FRONT_Z))]
        else:
            x = BACK_X * s
            B += [(f"leg_{leg}_upper", "spine_0", (x, 49, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 29, BACK_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 6.0, BACK_Z))]
    B = [(n, p, tuple(v * K for v in pv)) for n, p, pv in B]
    for s, n in ((1, "L"), (-1, "R")):  # the eyelids, placed on the scaled skull with the eye's rounding
        za, zb, top, _ = eye_box(s)
        B.append((f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2)))
    return B


def elements():
    E = [el("hips", "spine_0", (-10.6, 35, -40), (10.6, 57.5, 0)),  # reaching deep into the chest, below the withers
         el("chest", "spine_1", (-11.4, 30, -8), (11.4, 60, 30)),  # deep and broad
         el("skull", "head", *SKULL0),
         # Broad and blunt, half the skull's width (rule 4, Ronald 11-05: 40-70%), so the skull's front shows two
         # texels each side of it; to 59.0, where the nose box ended (rule 4: the furred nose painted on its
         # front, no box; the box was 4 proud).
         el("muzzle", "head", (-3.4, 59.6, 45.5), (3.4, 67.6, 59.0)),
         el("mouth", "head", (-2.4, 58.9, 44.5), (2.4, 60.9, 53.6)),  # half a unit inside the jaw
         # Shorter than the muzzle; its chin 3.7 below the muzzle, over two texels (at 57.6, 2 below, 1.5 texels, it read as a rim; linecheck).
         el("jaw", "jaw", (-2.9, 55.9, 44.0), (2.9, 60.4, 54.0))]  # its sides half a unit in from the muzzle's
    neck, _ = rod("neck", "neck", tuple(NECK0), NECK_ROT, NECK_LEN, NECK_W, back=6.0)  # slimmer than the head
    E.append(neck)
    # The mane: long pale hair down the throat, a box with locks hanging below it.
    n0 = NECK0
    E.append(el("mane", "neck", (-4.2, n0[1] + 3, n0[2] + 4.0), (4.2, n0[1] + 24, n0[2] + 8.6), rot=NECK_ROT, origin=tuple(n0)))
    E.append(el("fringe_mane", "neck", (0, n0[1] + 3.5, n0[2] + 6.0), (0, n0[1] + 24.5, n0[2] + 14.0), quad="east", rot=NECK_ROT,
                origin=tuple(n0), cut="mane"))
    E.append(el("tail", "tail", (-2.6, 45.5, -42.8), (2.6, 54.5, -39.75), rot=(12, 0, 0), origin=tuple(TAIL0)))
    for s, n in ((1, "L"), (-1, "R")):
        cx = EAR_X * s
        rot, origin = (-14, 0, -s * 45), (cx, 71.0, sum(EAR_Z) / 2)
        for k, (y0, y1, hw) in enumerate(EAR):
            E.append(el(f"ear_core_{n.lower()}{k}", f"ear_{n}", (cx - hw, y0, EAR_Z[0]), (cx + hw, y1, EAR_Z[1]), rot=rot, origin=origin))
        zc = sum(EAR_Z) / 2
        E.append(el(f"fringe_ear_{n.lower()}", f"ear_{n}", (cx - 3.1, 70, zc), (cx + 3.1, 79.6, zc), quad="south", rot=rot,
                    origin=origin, cut="ear"))
        E += antler(n)
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z = FRONT_X * s, FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 3.6, 20, z - 3.6), (x + 3.6, 42, z + 3.8)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2.4, 4.5, z - 2.4), (x + 2.4, 22, z + 2.4)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 3.2, 0, z - 3.0), (x + 3.2, 5.6, z + 3.8)))  # the broad hoof
        else:
            x, z = BACK_X * s, BACK_Z
            xi, xo = sorted((x - 3.8 * s, x + 3.8 * s))  # the ham
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (xi, 28, z - 6.5), (xo, 55.5, z + 6)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2.5, 5.5, z - 2.5), (x + 2.5, 30, z + 2.5)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 3.3, 0, z - 3.1), (x + 3.3, 6.2, z + 3.9)))
    for e in E:  # the deer's space, drawn K times bigger
        e["lo"] = [v * K for v in e["lo"]]
        e["hi"] = [v * K for v in e["hi"]]
        if e["origin"]:
            e["origin"] = [v * K for v in e["origin"]]
    for s, n in ((1, "L"), (-1, "R")):  # the eyelids, on the scaled skull
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        lx0, lx1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down"))
    return E


# ------------------------------------------------------------------ painting

FACES = {"north": (2, -1), "south": (2, 1), "west": (0, -1), "east": (0, 1), "down": (1, -1), "up": (1, 1)}
ANTLER_PARTS = ("antler_",)


def face_frame(lo, hi, face):
    lo, hi = np.array(lo), np.array(hi)
    X, Y, Z = np.eye(3)
    corner, du, dv = {"south": ((lo[0], hi[1], hi[2]), X, -Y), "north": ((hi[0], hi[1], lo[2]), -X, -Y),
                      "east": ((hi[0], hi[1], hi[2]), -Z, -Y), "west": ((lo[0], hi[1], lo[2]), Z, -Y),
                      "up": ((lo[0], hi[1], lo[2]), X, Z), "down": ((lo[0], lo[1], hi[2]), X, -Z)}[face]
    size = hi - lo
    ax, sgn = FACES[face]
    n = np.zeros(3)
    n[ax] = sgn
    # At TX texels a unit; never under 1 (an antler's 1-unit point is a texel, not nothing).
    return np.array(corner, float), du, dv, max(1, int(round(TX * abs(size @ du)))), max(1, int(round(TX * abs(size @ dv)))), n


def area(e, face, p, uu, vv, h):
    """A wild reindeer's coat, in the deer's space (`p` / K): grey-brown, darker along the back;
    the neck, the mane and the chest cream; a dark band along the flank above the white belly; the
    rump white round the short tail; the legs and face dark brown, white socks above the broad dark
    hooves; the nose furred grey; bone antlers. `uu`, `vv` and `h` are in texels (`t` the fraction
    down the face)."""
    name = e["name"]
    p = p / K
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, FUR)
    side = face in ("east", "west")
    t = vv / max(h, 1)
    if name in ("hips", "chest"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        else:
            a = np.where(t < 0.14, BACK, np.where(y < 33.0, BELLY, np.where(y < 37.0, LEG, FUR)))
            if name == "chest":
                a = np.where((z > 22) & (y > 37.0), PALE, a)  # the cream of the chest, under the mane
                if face == "south":
                    a = np.where(y > 33.0, PALE, BELLY)
            if name == "hips" and face == "north":  # the white rump
                a = np.where((np.abs(x) < 7.0) & (y > 39), BELLY, a)
            if name == "hips":
                a = np.where((z < -34) & (y > 41) & (y < 54), BELLY, a)  # the rump patch onto the haunches
    elif name in ("neck", "mane", "fringe_mane"):
        a[:] = PALE
        if name == "neck" and face in ("north", "up"):
            a = np.where(t < 0.5, FUR, PALE) if face == "north" else np.full(x.shape, FUR)
    elif name == "skull":
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = PALE
        elif side:
            a = np.where(z > 41.5, LEG, FUR)  # darker toward the muzzle
            za, zb, top, bottom = (v / K for v in eye_box(1 if face == "east" else -1))  # the painted eye
            d = np.hypot(np.maximum(np.abs(z - (za + zb) / 2) - (zb - za) / 2, 0),
                         np.maximum(np.abs(y - (top + bottom) / 2) - (top - bottom) / 2, 0))
            a = np.where((d > 0) & (d < 0.65 / (TX * K)), LEG, a)  # the dark round it: the texels beside it, not diagonal
        elif face == "north":
            a[:] = FUR
    elif name == "muzzle":
        if face == "up":
            a[:] = LEG
        elif face == "down":
            a[:] = PALE
        else:
            a[:] = LEG
            a = np.where(z > 52.6, FUR, a)  # the grey of the furred nose
            a = np.where(y < 61.0, PALE, a)  # the pale lip and chin
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else PALE
    elif name == "mouth":
        a[:] = MOUTH
    elif name.startswith(ANTLER_PARTS):
        a[:] = ANTLER
    elif name == "tail":
        a[:] = TAILTOP
        if face in ("north", "down"):
            a[:] = BELLY
        if side:
            a = np.where(np.abs(x) > 1.9, BELLY, a)
    elif name.startswith("ear_core"):
        a[:] = FUR
        if face == "south":
            cx = e["origin"][0] / K
            a = np.where(np.abs(x - cx) < 1.7, EARIN, np.where(np.abs(x - cx) < 2.3, FUR, DARK))
    elif name.startswith("thigh_"):
        a[:] = FUR
        if face == "up":
            a[:] = BACK
        a = np.where(y < 38, LEG, a)
        if face == "north":
            a = np.where(y > 40, BELLY, a)  # the white of the rump down the back of the thighs
    elif name.startswith("leg_"):
        a[:] = LEG
        if name.endswith("_paw"):
            a[:] = HOOF
            if face != "down":
                a = np.where(y > 3.3, BELLY, HOOF)  # the white sock above the hoof
        elif name.endswith("_lower"):
            a = np.where(y < 8.0, BELLY, LEG)
    return a


def antler_texels(e, p):
    """The antler's drawing at the face's texel centers `p`: "." none, "#" bone, "B" the burr; and
    whether each is the top of a point."""
    art = ANTLER_ART["L" if e["name"].endswith("_l") else "R"]
    R = rotm(e["rot"])
    loc = (p - np.array(e["origin"])) @ R / K  # the quad's own frame, the deer's space: y up, z forward
    r = len(art) - 1 - np.floor((loc[..., 1] - ANTLER_Y0) / ANTLER_T).astype(int)
    c = np.floor((loc[..., 2] - ANTLER_Z0) / ANTLER_T).astype(int)
    rows, cols = len(art), len(art[0])
    get = lambda rr, cc: art[rr][cc] if 0 <= rr < rows and 0 <= cc < cols else "."  # noqa: E731
    k = np.array([[get(rr, cc) for rr, cc in zip(R_, C_)] for R_, C_ in zip(r, c)])
    tip = np.array([[get(rr, cc) == "#" and get(rr - 1, cc) == "." and rr <= 1 for rr, cc in zip(R_, C_)] for R_, C_ in zip(r, c)])
    return k, tip


def cut_mask(e, p):
    """Cut-out outlines: the ear's short oval, the mane's locks (in the neck's own frame), an
    antler's drawing (`ANTLER_ART`)."""
    if e["cut"] == "antler":
        return antler_texels(e, p)[0] != "."
    if e["cut"] == "ear":
        x, y = p[..., 0] / K, p[..., 1] / K
        cx = e["origin"][0] / K
        t = (y - 74.6) / 4.6
        half = 2.9 * np.sqrt(np.clip(1 - t ** 2, 0, 1)) + 0.3 * ((np.floor(y * 1.3) % 4) == 0) * (np.abs(t) < 0.8)
        return (np.abs(x - cx) <= half) & (y >= 70)
    o = np.array(e["origin"])
    local = (p - o) @ rotm(e["rot"]) / K  # back into the neck's frame, the deer's space
    along, depth = local[..., 1], local[..., 2]
    u = np.clip((along - 3.5) / 21.0, 0, 1)
    period = max(2.5, 3 / (TX * K))  # a lock every 3 texels or more (finer ones alias at 32)
    lock = np.abs(((along / period) % 1.0) - 0.5) * 2  # points every few texels
    tip = 8.8 + 4.6 * np.sin(math.pi * u) ** 0.7 - max(1.6, 1 / (TX * K)) * lock  # notches a texel deep
    return depth <= tip


LASH = (40, 30, 28)

# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)


def layer_props(e, face, p, a):
    """White where this reindeer is white (and up from the belly; a white reindeer whitens all
    over); the legs and face its points; the back and flanks where marks show."""
    n, y = e["name"], p[..., 1]
    lo, hi = e["lo"][1], e["hi"][1]
    low = 1 - _ramp(y, lo, max(hi, lo + 1e-3))
    white = 0.5 * low if n in ("chest", "hips", "neck") else np.full(y.shape, 0.25)
    white = np.maximum(white, np.where((a == BELLY) | (a == PALE), 0.85, 0))
    points = np.where(a == LEG, 1.0, 0.0)
    ext = np.where(a == TAILTOP, 1.0, 0.0)
    pattern = np.full(y.shape, 1.0 if n in ("hips", "chest") else (0.4 if n.startswith("thigh_") else 0.0))
    return {"white": white, "points": points, "extremity": ext, "pattern": pattern}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    size = np.array(e["hi"]) - np.array(e["lo"])
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel centers, in texels
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
    p = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    light_face = face
    if e["name"].startswith("eyelid"):
        light_face = "east"  # closed, it is the head's side
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "jaw", "tail", "mane")):
        light_face = "north"
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    stacked = e["name"].startswith("ear_core")
    down = np.clip((79.0 * K - p[..., 1]) / (8.5 * K), 0, 1) if stacked else None
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT,
                            y_world=p[..., 1] if not e["name"].startswith(ANTLER_PARTS) else None,
                            occ=occ, quad=bool(e["quad"]), seed=sum(map(ord, e["name"] + face)) % 97, down=down,
                            tuft=e["name"] not in ("skull", "muzzle"), merge=e["name"] != "skull")  # keep the eye ring
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"]:
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["cut"] == "antler":  # bone; the burr a shade darker, the points' tips lighter
        k, tip = antler_texels(e, p)
        img[k == "B", :3] = (img[k == "B", :3] * 0.8).astype(np.uint8)
        img[tip, :3] = np.clip(img[tip, :3] * 1.15, 0, 255).astype(np.uint8)
    if e["name"] == "skull" and face in ("east", "west"):
        s = 1 if face == "east" else -1
        stamp_eyes(img, [eye_center(s)], eye_style(s))
    if e["name"] == "muzzle" and face == "south":  # nostrils as slits, the split lip (the nose painted here: rule 4)
        c = w // 2
        if w >= 10 and h >= 4:
            img[2, [c - 4, c - 3, c + 2, c + 3], :3] = (82, 66, 58)
            img[h - 3:h, c - 1:c + 1, :3] = (img[h - 3:h, c - 1:c + 1, :3] * 0.72).astype(np.uint8)
        elif w >= 3 and h >= 3:  # at 32 the nose is 5 texels by 3: a nostril at each edge, the lip a middle texel
            img[1, [0, w - 1], :3] = (82, 66, 58)
            if w % 2:
                img[h - 1, c, :3] = (img[h - 1, c, :3] * 0.72).astype(np.uint8)
    if e["name"].startswith("eyelid"):
        col_idx = 0 if e["name"].endswith("L") else w - 1
        img[:, col_idx, :3] = LASH
    if e["name"].endswith("_paw") and face == "south" and w >= 3 and w % 2:  # the cloven hoof: a split line, on a middle texel
        c = w // 2
        img[:, c, :3] = (img[:, c, :3] * 0.6).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"lie_front": 80, "lie_hip": -70, "lie_knee": 140, "lie_drop": 30, "graze_neck": 40, "graze_head": 30, "graze_lean": 4, "graze_splay": 0}


def gait(p, phases, swing_amp, lift):
    """A foot swings forward while it's lifted, back while it's down."""
    out = {}
    for leg, ph in zip(LEGS, phases):
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))
        out[f"leg_{leg}_upper"] = (-swing_amp * S(a), 0, 0)
        knee = lift * up
        if leg[0] == "F":
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (knee, 0, 0), (-0.5 * knee, 0, 0)
        else:
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (-knee, 0, 0), (0.6 * knee, 0, 0)
    return out


def chew(t, rate=2.5, amp=7.0):
    return {"jaw": (amp * max(0.0, S(TAU * t * rate)), 0, 0)}


def graze_pose(g, P=L):
    """Neck and head down toward the lichen by `g` (0..1); the front legs straddled forward a
    little to lower the shoulders, the hooves kept flat on the ground."""
    sp = P["graze_splay"] * g
    q = {"spine_1": (P["graze_lean"] * g, 0, 0), "neck": (P["graze_neck"] * g, 0, 0), "head": (P["graze_head"] * g, 0, 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (-sp - P["graze_lean"] * g, 0, 0)
        q[f"leg_{leg}_paw"] = (sp, 0, 0)
    return q


def pose_idle(t):
    """Grazing, chewing, then the head up to look about, an ear flick and a tail flick (8 s)."""
    p = t / 8.0
    br = S(TAU * p * 3)
    g = float(np.interp(p, [0, .05, .12, .55, .62, 1], [1, 1, 1, 1, 0, 0])) if p < 0.62 else float(np.interp(p, [.62, .9, .97, 1], [0, 0, 1, 1]))
    look = (1 - g) * 14 * S(TAU * (p - 0.6) * 1.5)
    fl = max(0.0, S(TAU * (p - 0.3) * 12)) if 0.3 < p < 0.34 else 0.0
    tf = max(0.0, S(TAU * (p - 0.7) * 10)) if 0.7 < p < 0.75 else 0.0
    q = graze_pose(g)
    q.update({"root": (0, 0.15 * br, 0), "head": (q["head"][0], look, 0), "ear_L": (0, 0, -15 * fl), "ear_R": (0, 0, 2 * br),
              "tail": (-25 * tf, 0, 0)})
    q.update(chew(t, 2.5 if g > 0.5 else 1.2, 6 if g > 0.5 else 2))
    return q


def pose_walk(t):
    """The long-striding walk, the head nodding low (1.5 s)."""
    p = t / 1.5
    q = gait(p, (0.0, 0.5, 0.75, 0.25), 20, 36)
    q.update({"root": (0, 0.6 * abs(S(TAU * p * 2)), 0), "neck": (4 + 3 * S(TAU * p * 2), 0, 0), "head": (-3 * S(TAU * p * 2), 0, 0),
              "tail": (-4, 0, 0)})
    return q


def pose_run(t):
    """The gallop, the head out low and the short tail up (0.55 s)."""
    p = t / 0.55
    q = gait(p, (0.0, 0.1, 0.5, 0.6), 32, 60)
    c = S(TAU * p)
    q.update({"root": (0, 2.4 * max(0.0, S(TAU * p + 0.6)), 0), "spine_0": (-4 * c, 0, 0), "spine_1": (6 * c, 0, 0),
              "neck": (6 - 5 * c, 0, 0), "head": (-4, 0, 0), "ear_L": (-20, 0, 8), "ear_R": (-20, 0, -8),
              "tail": (-40, 0, 0)})
    return q


def pose_lie(t, P=L):
    """Lying down, the legs folded under, the head up and chewing the cud (6 s)."""
    p = t / 6.0
    br = S(TAU * p * 2)
    q = {"root": (0, -P["lie_drop"] + 0.15 * br, 0), "spine_1": (0.5 * br, 0, 0), "head": (0, 8 * S(TAU * p), 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (P["lie_front"], 0, 0)
        q[f"leg_{leg}_lower"] = (-150, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_front"] + 150 - 40, 0, 0)
    for leg, s in (("BL", 1), ("BR", -1)):
        q[f"leg_{leg}_upper"] = (P["lie_hip"], 0, s * 5)
        q[f"leg_{leg}_lower"] = (P["lie_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_hip"] - P["lie_knee"], 0, 0)
    q.update(chew(t, 1.0, 4.0))
    return q


BLINKS = {"idle": [2.0, 5.9], "lie": [1.5, 4.4]}
BLINK = 0.12
ANIMS = {"idle": (pose_idle, 8.0), "walk": (pose_walk, 1.5), "run": (pose_run, 0.55), "lie": (pose_lie, 6.0)}


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}


def frame_times(name, dur):
    n = {"idle": 96, "lie": 60, "walk": 24, "run": 20}.get(name, 16)
    ts = {round(dur * k / n, 4) for k in range(n + 1)}
    for b in BLINKS.get(name, []):
        ts |= {round(b + d, 4) for d in (-BLINK, -BLINK / 2, 0, BLINK / 2, BLINK)}
    return sorted(ts)


def animations(bone_names):
    out = {}
    for name, (fn, dur) in ANIMS.items():
        frames = []
        for t in frame_times(name, dur):
            tt = t % dur if t < dur else 0.0
            pose = fn(tt)
            pose.update(eyelids(name, tt))
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames}
    return out


# ------------------------------------------------------------------ fitting poses to the ground

GROUPS = {"front paws": lambda n: n in ("leg_FL_paw", "leg_FR_paw"), "back paws": lambda n: n in ("leg_BL_paw", "leg_BR_paw"),
          "body": lambda n: n in ("hips", "chest"), "muzzle": lambda n: n in ("muzzle", "jaw"),
          "legs": lambda n: n.startswith(("leg_", "thigh_")), "antlers": lambda n: n.startswith(ANTLER_PARTS),
          "mane": lambda n: n in ("mane", "fringe_mane")}


def fit_poses(spec):
    """Lying (the belly on the ground, the folded legs under it, nothing below) and grazing (the
    mouth at the ground), by forward kinematics."""
    def lie_cost(P):
        g = lowest(spec, pose_lie(0.0, dict(L, **P, lie_drop=0)), GROUPS)
        f = min(v for k, v in g.items() if k != "mane")
        return (g["body"] - f - 0.8) ** 2 + 0.3 * (g["front paws"] - f) ** 2 + 0.3 * (g["back paws"] - f) ** 2
    grid = {"lie_front": list(range(60, 101, 5)), "lie_hip": list(range(-100, -49, 5)), "lie_knee": list(range(115, 166, 5))}
    best = fit(grid, lie_cost)[1]
    g = lowest(spec, pose_lie(0.0, dict(L, **best, lie_drop=0)), GROUPS)
    L.update(best, lie_drop=round(float(min(g.values())), 2))

    def graze_cost(P):
        q = graze_pose(1.0, dict(L, **P))
        g = lowest(spec, q, GROUPS)
        # The mouth at the ground, the antlers and the mane's locks clear of it.
        return (g["muzzle"] - 0.5) ** 2 + 0.002 * (P["graze_neck"] ** 2 + P["graze_head"] ** 2) + 0.05 * P["graze_lean"] ** 2 \
            + 20 * max(0.0, 1.0 - g["antlers"]) ** 2 + 20 * max(0.0, 0.5 - g["mane"]) ** 2 + 40 * (g["front paws"] - g["back paws"]) ** 2 + 60 * min(0.0, g["front paws"]) ** 2
    best = fit({"graze_neck": list(range(30, 126, 5)), "graze_head": list(range(-45, 61, 5)), "graze_lean": [0, 4, 8, 12, 16],
                "graze_splay": [0, 6, 12, 18]}, graze_cost)[1]
    L.update(best)
    gl = lowest(spec, graze_pose(1.0), GROUPS)
    print("grazing, lowest:", {k: round(v, 2) for k, v in gl.items()})
    ll = lowest(spec, pose_lie(0.0), GROUPS)
    print("lying, lowest:", {k: round(v, 2) for k, v in ll.items()})


# ------------------------------------------------------------------ build


def build():
    B = bones()
    E = elements()
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"] and not e["rot"]]
    LAYERS["rec"] = Recorder(layer_bounds(E, 3.0))
    tiles = []
    for i, e in enumerate(E):
        for f in ([e["quad"]] if e["quad"] else list(FACES)):
            LAYERS["key"] = (i, f)
            tiles.append(((i, f), paint_face(e, f, boxes)))
    sheet, uv = pack(tiles, 512)  # the layers pack at the same width (`Recorder.save`)
    LAYERS["rec"].save(OUT, [k for k, _ in tiles], pack)
    LAYERS["rec"] = None
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "reindeer", "texture": [sheet.shape[1], sheet.shape[0]],
            "bones": [{"name": n, "parent": p, "pivot": [float(v) for v in pv]} for n, p, pv in B], "elements": E}
    fit_poses(spec)
    spec["animations"] = animations([n for n, _, _ in B])
    spec["fitted"] = dict(L)
    return spec, sheet


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    spec, sheet = build()
    Image.fromarray(sheet, "RGBA").save(os.path.join(OUT, "texture.png"))
    with open(os.path.join(OUT, "spec.json"), "w") as f:
        json.dump(spec, f)
    print(len(spec["elements"]), "elements;", len(spec["bones"]), "bones; texture", sheet.shape[1], "x", sheet.shape[0],
          "; fitted", spec["fitted"])
