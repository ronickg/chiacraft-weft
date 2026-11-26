#!/usr/bin/env python3
"""The Utahraptor: the biggest raptor, feathered like its relatives (quill knobs on Dakotaraptor
show even big ones had them), built from the Velociraptor's source (`../raptor`) with the same
skill: the same skeleton, but heavier everywhere, the way an ambush hunter of 280-500 kg is
built (thick thighs and shins, a deep body, a thick neck and tail), a boxier, deeper head with a
straighter snout and big eyes, a bigger sickle claw, and an umber coat with a dark rust mane.

    ../dog/chihuahua/.venv/bin/python utahraptor.py      # writes out/spec.json, out/texture.png

Units are its own (exported at 0.83 model pixels a unit, `build.SCALE`: about 19 a block), y up,
it faces +z. A real Utahraptor is about 1.5 m at the hip and 4.7-6 m long, half of it tail; here
the hip is about 29 units up after the crouch, and the export's scale makes that 1.5 blocks.
Painted at the house density, 32 texels a block (`TX`, about 1.66 texels a unit), as the blocks
and items are.
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
from paint import EYES, eye_corner, pack, paint_face_smooth, occlusion, stamp_eyes, tones  # noqa: E402
from rig import fit, swing  # noqa: E402
from ground import lowest, world  # noqa: E402
from house import texels  # noqa: E402

TX = texels(0.83)  # texels a unit: the house's 32 a block at the Utahraptor's export scale (build.SCALE)

OUT = os.environ.get("UTAH_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials, coats and eyes

BACK, SIDE, BELLY, BAR, MARK, SCALE, TEETH, DARK, CLAW, TIP, PALE = range(11)
FUR = (BACK, SIDE, BELLY, PALE)  # the shaggy feather coat; bars, marks, teeth, claws stay crisp
SHADOW = [0.02, 0.04, 0.06, 0.0, 0.0, 0.05, 0.1, 0.95, 0.7, 0.0, 0.06]
_TEETH, _DARK, _CLAW = (236, 232, 216), (70, 30, 34), (54, 44, 42)
# Coats that differ in kind (all on the same UV layout): (pattern, colors in material order).
# Colors of big ground birds and of what large predators wear: no blues, greens or pinks.
COATS = {
    "umber": ("bars", [(96, 74, 58), (156, 128, 100), (220, 204, 174), (58, 44, 36), (58, 44, 36), (126, 110, 94),
                       _TEETH, _DARK, _CLAW, (48, 38, 34), (214, 200, 176)]),
    "emu": ("speckle", [(90, 82, 74), (130, 118, 104), (194, 184, 168), (46, 40, 36), (40, 36, 34), (104, 98, 92),
                        _TEETH, _DARK, _CLAW, (36, 32, 30), (188, 178, 164)]),
    "desert": ("spots", [(190, 158, 108), (212, 184, 136), (236, 222, 190), (118, 84, 52), (108, 74, 44), (150, 128, 100),
                         _TEETH, _DARK, _CLAW, (98, 70, 46), (232, 216, 184)]),
    "forest": ("bands", [(74, 68, 48), (114, 102, 74), (206, 196, 160), (36, 32, 24), (36, 32, 24), (96, 90, 72),
                         _TEETH, _DARK, _CLAW, (30, 28, 22), (200, 190, 150)]),
    "ash": ("plain", [(86, 86, 90), (128, 126, 126), (198, 194, 188), (44, 42, 44), (40, 38, 40), (102, 98, 94),
                      _TEETH, _DARK, _CLAW, (30, 28, 30), (234, 232, 226)]),
    "rufous": ("saddle", [(150, 84, 48), (186, 120, 72), (228, 206, 168), (80, 44, 30), (66, 38, 28), (130, 104, 84),
                          _TEETH, _DARK, _CLAW, (58, 34, 26), (232, 214, 184)]),
}
COAT = os.environ.get("UTAH_COAT", "umber")
PATTERN, BASE = COATS[COAT]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
ROLES = [("coat", 0, 0), ("coat", 0.5, 0), ("coat", 1, 0), ("coat", 0, 0), ("coat", 0, 0), "fixed",
         "fixed", "fixed", "fixed", ("coat", 0, -1.5), ("coat", 1, 0.3)]  # BACK SIDE BELLY BAR MARK SCALE TEETH DARK CLAW TIP PALE


def set_coat(name):
    """Paint with another coat (same layout): its pattern and colors."""
    global COAT, PATTERN, BASE, TONES
    COAT = name
    PATTERN, BASE = COATS[name]
    TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]


# An alligator's eye (Carpenter's restoration gave it one): an amber iris lighter below, a
# vertical slit pupil, a dark ring, a cool highlight; big, on the sides of a boxy head. Five rows
# high, so the eyelids (as wide as the eye is high) don't meet inside the head.
EYES.update({"utah": dict(pattern=[".LLLLLL.",
                                   "LAHHPAAL",
                                   "LAHAPAAL",
                                   "LaaAPaaL",
                                   ".LLLLLL."], colors={"L": (44, 30, 24), "A": (190, 132, 44), "a": (226, 176, 78),
                                                       "H": (236, 236, 228), "P": (14, 10, 10)}, halo="round")})
# The same eye at 32 a block, where the head's side is 20 texels by 15: 12 by 8 (7.2 by 4.8 units,
# about the old eye's 8 by 5), rounder at the corners, the slit pupil two texels wide in the middle
# and one at its ends, a small highlight up front. The amber (A, a) is the iris the game recolors.
EYES.update({"utah32": dict(pattern=["..LLLLLLLL..",
                                     ".LAAAAPAAAL.",
                                     "LAHHAAPPAAAL",
                                     "LAHAAAPPAAAL",
                                     "LAAAAAPPAAAL",
                                     "LaaaAAPPAaaL",
                                     ".LaaaaPaaaL.",
                                     "..LLLLLLLL.."], colors={"L": (44, 30, 24), "A": (190, 132, 44), "a": (226, 176, 78),
                                                             "H": (236, 236, 228), "P": (14, 10, 10)}, halo="round",
                            iris="Aa")})
EYE_STYLE = os.environ.get("UTAH_EYES", "utah32")
EYE_Z, EYE_Y = 25.0, 40.0  # the eye's middle on the side of the head
SKULL = ((-4.5, 35, 18), (4.5, 44, 30))
LID_X = 5.1  # the eyelids' hinge plane, just outside the head's side (x = 4.5)

# ------------------------------------------------------------------ the model

TAIL = [(10, 9.0), (10, 7.4), (9, 5.8), (8, 4.4), (7, 3.2)]  # length, width (0.6+ a side thinner each)
TAIL0 = np.array([0.0, 31.0, -5.5])


NECK_BASE = np.array([0.0, 31.0, 14.0])  # inside the chest's front top
NECK_ANGLE, NECK_LEN = 50.0, 12.0  # rising forward, its end deep in the skull


def neck_end():
    """The head's joint: the far end of the neck, inside the skull."""
    a = math.radians(NECK_ANGLE)
    return NECK_BASE + NECK_LEN * np.array([0.0, math.sin(a), math.cos(a)])


def tail_points():
    pts, p = [TAIL0.copy()], TAIL0.copy()
    for length, _ in TAIL:
        p = p + np.array([0, 0, -length])
        pts.append(p.copy())
    return pts


def el(name, bone, lo, hi, quad=None, rot=None, origin=None, cut=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad, rot=rot,
                origin=[float(v) for v in origin] if origin else None, cut=cut)


def skull_grid():
    """The skull side's texels: (width along z, height) and the units a texel spans (u, v)."""
    (_, y0, z0), (_, y1, z1) = SKULL
    w, h = max(1, round(TX * (z1 - z0))), max(1, round(TX * (y1 - y0)))
    return w, h, (z1 - z0) / w, (y1 - y0) / h


def eye_center(side):
    """The eye's middle on the skull's east (side > 0) or west face, in that face's texels."""
    (_, _, z0), (_, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    cu = (z1 - EYE_Z) if side > 0 else (EYE_Z - z0)  # east faces run u along -z, west along +z
    return cu / su, (y1 - EYE_Y) / sv


def eye_texel(side):
    """The eye's top-left texel (u0, v0) on its face, as `stamp_eyes` places it."""
    return eye_corner(*eye_center(side), EYES[EYE_STYLE]["pattern"], skull_grid()[0])


def eye_box(side):
    """(z0, z1, top, bottom) of the eye on one side, in units, from the texels `stamp_eyes` paints
    it on, so the eyelid covers it to the texel."""
    pat = EYES[EYE_STYLE]["pattern"]
    ph, pw = len(pat), len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    _, _, su, sv = skull_grid()
    u0, v0 = eye_texel(side)
    za, zb = (z1 - (u0 + pw) * su, z1 - u0 * su) if side > 0 else (z0 + u0 * su, z0 + (u0 + pw) * su)
    top = y1 - v0 * sv
    return za, zb, top, top - ph * sv


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 30, 0)), ("spine_1", "spine_0", (0, 31, 5.5)),
         ("neck", "spine_1", tuple(NECK_BASE)), ("head", "neck", tuple(neck_end())), ("jaw", "head", (0, 37.5, 28.5))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B += [(f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2)),
              (f"arm_{n}", "spine_1", (s * 6.5, 29.5, 13.5)), (f"forearm_{n}", f"arm_{n}", (s * 7.25, 24.5, 13.5))]
    pts = tail_points()
    B += [(f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", tuple(pts[k])) for k in range(len(TAIL))]
    for s, n in ((1, "L"), (-1, "R")):
        x = 5.2 * s
        B += [(f"leg_{n}_thigh", "spine_0", (x, 28, 0)), (f"leg_{n}_shin", f"leg_{n}_thigh", (x, 19, 0)),
              (f"leg_{n}_foot", f"leg_{n}_shin", (x, 8.5, -0.5)), (f"leg_{n}_toes", f"leg_{n}_foot", (x, 2.2, -0.5))]
    return B


def elements():
    E = [el("hips", "spine_0", (-6.5, 24, -6.5), (6.5, 35.5, 6.5)),
         el("chest", "spine_1", (-7, 23.5, 5.5), (7, 35, 17.6)),
         # The neck: a box from inside the chest out along the neck's angle to the head's joint,
         # which sits inside the skull, so head and neck share a point and never come apart; slimmer
         # than the skull and ending deep in it, so no corner of it pokes out of the head when the
         # head turns (`ground.pokes`).
         el("neck", "neck", NECK_BASE + np.array([-3.2, -3.2, -1.5]), NECK_BASE + np.array([3.2, 3.2, NECK_LEN + 1.5]),
            rot=(-NECK_ANGLE, 0, 0), origin=tuple(NECK_BASE)),
         el("skull", "head", *SKULL),
         # A deep, straight snout over a shorter jaw (an overbite hides the mouth's inside).
         el("snout", "head", (-3.5, 37.5, 29), (3.5, 42.5, 38.5), rot=(-3, 0, 0), origin=(0, 40, 29)),
         # Two texels in from the jaw's sides and front (at 2.5 wide and 36.5, 0.8- and 1.7-texel rims of the jaw's top
         # showed round it under the upturned snout; linecheck).
         el("mouth", "head", (-1.8, 36.6, 29.5), (1.8, 39.2, 36.3)),
         el("jaw", "jaw", (-3, 34.4, 28.5), (3, 37.8, 37.5))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        x0, x1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (x0, top, za), (x1, top, zb), quad="down"))
        ax = 7.25 * s  # the shoulder (x = 6.5) inside both the chest and the arm
        # Its bottom under two texels below the forearm's top (at 23.9, 0.7-texel strips of its front showed beside the
        # forearm, 2.7 long; linecheck).
        E.append(el(f"arm_{n}", f"arm_{n}", (ax - 1.5, 24.4, 12), (ax + 1.5, 30, 15.5)))
        # Two texels below the chest (at 22.5, a 1.7-texel band of its side showed under the chest; linecheck).
        E.append(el(f"forearm_{n}", f"forearm_{n}", (ax - 1.1, 22.2, 13.5), (ax + 1.1, 25.5, 21.5)))
        # Broad wing feathers, like a vulture's, from the forearm; more than 0.5 clear of the
        # thighs' side planes (legs swing about x).
        wx = 9.75 * s
        E.append(el(f"wing_{n}", f"forearm_{n}", (wx, 14.5, 10.5), (wx, 25, 22.5), quad="east" if s > 0 else "west", cut="wing"))
        # Shaggy body feathers hanging over the thighs (an emu's), from the hips.
        E.append(el(f"shag_{n}", "spine_0", (9.8 * s, 20, -6), (9.8 * s, 34, 6.5), quad="east" if s > 0 else "west", cut="shag"))
    pts = tail_points()
    for k, (length, w) in enumerate(TAIL):
        p = pts[k]
        hh = w * 0.85
        E.append(el(f"tail_{k}", f"tail_{k}", (-w / 2, p[1] - hh / 2, p[2] - length), (w / 2, p[1] + hh / 2, p[2] + 0.5)))
    tip = pts[-2]
    E.append(el("tail_fan", f"tail_{len(TAIL) - 1}", (-6, tip[1] + 0.2, tip[2] - 11), (6, tip[1] + 0.2, tip[2] - 1), quad="up", cut="fan"))
    for s, n in ((1, "L"), (-1, "R")):
        x = 5.2 * s
        E.append(el(f"thigh_{n}", f"leg_{n}_thigh", (x - 4, 17.5, -4.5), (x + 4, 30, 5.5)))
        # The shin and foot a little thicker, the foot 0.4 past the side toes (at 2, 0.3 of the toes' tops showed beside
        # it, a half-texel strip; linecheck), the shin 0.4 past the foot.
        E.append(el(f"shin_{n}", f"leg_{n}_shin", (x - 3.1, 8, -3), (x + 3.1, 19.5, 2)))
        E.append(el(f"foot_{n}", f"leg_{n}_foot", (x - 2.7, 2, -2.5), (x + 2.7, 9, 1.5)))
        # Three toes, each thinner than the foot: III long and IV outside it on the ground, II
        # inside, held up, with the sickle claw curving up from it.
        E.append(el(f"toe3_{n}", f"leg_{n}_toes", (x - 0.8, 0, -2), (x + 0.8, 2.2, 7.5)))
        # (Inside the shin's side planes by 0.3, which the folded resting legs bring them to.)
        E.append(el(f"toe4_{n}", f"leg_{n}_toes", (x + s * 1.1, 0, -2), (x + s * 2.3, 2.0, 6)) if s > 0 else
                 el(f"toe4_{n}", f"leg_{n}_toes", (x - 2.3, 0, -2), (x - 1.1, 2.0, 6)))
        ix = x - s * 1.7
        E.append(el(f"toe2_{n}", f"leg_{n}_toes", (ix - 0.6, 0.7, -2), (ix + 0.6, 2.5, 3)))
        E.append(el(f"sickle_{n}", f"leg_{n}_toes", (ix - 0.5, 2.0, 1.2), (ix + 0.5, 6.2, 3.0), rot=(-30, 0, 0), origin=(ix, 2.3, 2.0)))
    return E


# ------------------------------------------------------------------ painting

FACES = {"north": (2, -1), "south": (2, 1), "west": (0, -1), "east": (0, 1), "down": (1, -1), "up": (1, 1)}


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
    w, h = int(round(TX * abs(size @ du))), int(round(TX * abs(size @ dv)))
    if abs(size @ du) > 0 and abs(size @ dv) > 0:  # a small face is still a texel
        w, h = max(1, w), max(1, h)
    return np.array(corner, float), du, dv, w, h, n


def rotm(rot):
    if not rot:
        return np.eye(3)
    rx, ry, rz = (math.radians(a) for a in rot)
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


def bars(t, period=5.0, width=0.32):
    """Bands across `t` (units), each at least a texel wide (a narrower one aliases into noise)."""
    return ((t / period) % 1.0) < max(width, 1 / (TX * period))


def rings(i, period, width):
    """Texel rows or columns (`i`, texel centers) in bands `period` units apart and `width` units
    wide, rounded to whole texels so each band is as wide as the next (a texel at least)."""
    n = max(2, round(period * TX))
    return (np.floor(i) % n) < min(max(1, round(width * TX)), n - 1)


def teeth(uu, vv, h, down=True):
    """Teeth along a side face's edge (the bottom if `down`, else the top): slim pickets a texel
    wide and two long (0.6 by 1.2 units), two texels of gum between."""
    k = np.floor(uu) % 3
    r = (h - 1 - np.floor(vv)) if down else np.floor(vv)  # 0 at the edge
    return (r <= 1) & (k == 0)


def hashed(p, cell, seed):
    """A steady 0..1 per cell of `cell` units (the same texel always gets the same number)."""
    q = np.floor(p / cell)
    v = np.sin(q[..., 0] * 12.9898 + q[..., 1] * 78.233 + q[..., 2] * 37.719 + seed) * 43758.5453
    return v - np.floor(v)


def patterned(a, p, where):
    """The coat's pattern over `where` (the back and upper sides): bars, speckles, spots, wide
    bands, a dark saddle, or nothing (the plain coat shows its pattern on the wings)."""
    x, z = p[..., 0], p[..., 2]
    if PATTERN == "bars":
        return np.where(where & bars(z + 1.5 * np.sin(x * 0.5), 5.5, 0.3), BAR, a)
    if PATTERN == "speckle":
        return np.where(where & (hashed(p, 1.0, 3.0) < 0.14), MARK, a)
    if PATTERN == "spots":  # rosettes: a dark ring round some cells' middles
        c = (np.floor(p / 4.0) + 0.5) * 4.0
        d = np.linalg.norm((p - c)[..., [0, 2]], axis=-1)
        return np.where(where & (hashed(p, 4.0, 7.0) < 0.55) & (d > 0.7) & (d < 1.6), MARK, a)
    if PATTERN == "bands":
        return np.where(where & bars(z + 0.4 * p[..., 1], 7.0, 0.38), BAR, a)
    if PATTERN == "saddle":
        return np.where(where, MARK, a)
    return a


BODY = ("hips", "chest", "neck")


def area(e, face, p, uu, vv, h, su, sv):
    """Which material each texel is: a shaggy coat (back, flanks, belly) with the coat's pattern,
    a face with big eyes and a dark brow, scaly legs and three-toed feet with dark claws, broad
    dark-tipped wing feathers, white teeth along the jaws. `uu`, `vv` are texel centers (texel
    indices + 0.5) across the face, `su`, `sv` the units a texel spans."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, SIDE)
    side = face in ("east", "west")
    t = vv / max(h, 1)  # 0 at a side face's top
    reach = 0.7 if PATTERN in ("bands", "spots") else 0.5  # how far down the sides the pattern goes
    if name in BODY or name.startswith("tail_") and name != "tail_fan":
        if face == "up":
            a[:] = BACK
            a = patterned(a, p, np.ones(a.shape, bool) if PATTERN != "saddle" or not name.startswith("tail_") or name == "tail_0" else np.zeros(a.shape, bool))
        elif side:
            a = np.where(t < 0.4, BACK, np.where(t < 0.78, SIDE, BELLY))
            saddle_here = PATTERN != "saddle" or name in BODY or name == "tail_0"
            a = patterned(a, p, (t < reach) & saddle_here)
        elif face == "down":
            a[:] = BELLY
        elif face == "south":
            a = np.where(vv < 0.4 * h, BACK, np.where(np.abs(x) < 3, PALE, SIDE))  # a pale throat
        else:
            a[:] = BACK
        if name.startswith("tail_") and side and PATTERN in ("bars", "plain"):  # rings toward the tip
            a = np.where(bars(-z, 6, 0.28) & (t < 0.6), BAR, a)
    elif name == "skull":
        if face == "up":
            a[:] = BACK
        elif side:
            a = np.where(t < 0.3, BACK, SIDE)
            if PATTERN == "saddle":  # a pale face below the eyes
                a = np.where(t > 0.62, PALE, a)
        elif face == "down":
            a[:] = BELLY
        elif face == "north":
            a[:] = BACK
    elif name == "snout":
        if face == "up":
            a[:] = BACK
        elif side:
            a = np.where(t < 0.3, BACK, PALE if PATTERN == "saddle" else SIDE)
            a = np.where(teeth(uu, vv, h), TEETH, a)  # teeth along the edge
        elif face == "down":
            a[:] = DARK
            edge = np.minimum(uu, a.shape[1] - uu) * su < 1  # the unit outside the mouth: the teeth
            a[edge] = TEETH
        elif face == "south":
            a = np.where(vv < 0.4 * h, BACK, SIDE)
    elif name == "jaw":
        if face == "up":
            a[:] = DARK
            w = a.shape[1]
            row = np.where(rings(np.arange(a.shape[0]) + 0.5, 2.0, 1.0), TEETH, DARK)  # a tooth a unit long, every 2
            a[:, 0], a[:, w - 1] = row, row
        elif side:  # the face's color on the sides, pale below (all pale grins from the front)
            a = np.where(t < 0.55, PALE if PATTERN == "saddle" else SIDE, BELLY)
            a = np.where(teeth(uu + 1.5, vv, h, down=False), TEETH, a)
        elif face == "south":
            a = np.where(t < 0.55, SIDE, BELLY)
        else:
            a[:] = BELLY
    elif name == "mouth":
        a[:] = DARK
    elif name.startswith(("arm_", "forearm_")):
        a = np.where(t < 0.5, BACK, SIDE) if side or face == "up" else np.full(x.shape, SIDE)
        if name.startswith("forearm") and face == "south":
            a[:] = CLAW  # the hand's claws, a texel each with scales between
            a[:, 1::2] = SCALE
    elif name.startswith("wing_"):
        a = np.where(y > 22, BACK, SIDE)
        a = np.where(y < 18.5, TIP, a)  # dark primaries
        if PATTERN == "plain":  # a white bar across the wing
            a = np.where((y > 19.5) & (y < 21.5), PALE, a)
        a = np.where(bars(z, 2.0, 0.2) & (y < 22), BAR, a)  # the feathers' edges
    elif name.startswith("shag_"):
        a = np.where(y > 31, BACK, SIDE)
        a = patterned(a, p, y > 27)
        a = np.where(y < 22.2, TIP, a)  # darker lock tips
    elif name == "tail_fan":
        a[:] = BACK
        a = np.where(z < e["lo"][2] + 2.5, TIP, a)
        if PATTERN == "plain":
            a = np.where((z > e["lo"][2] + 3) & (z < e["lo"][2] + 4.5), PALE, a)
        a = np.where(bars(x + 6, 2.0, 0.25), BAR, a)
    elif name.startswith("thigh_"):
        a = np.where(t < 0.5, BACK, SIDE) if side else np.full(x.shape, SIDE)
        if face == "down":
            a[:] = BELLY
    elif name.startswith(("shin_", "foot_", "toe")):
        a[:] = SCALE
        if name.startswith("toe") and face == "south":
            a[:] = CLAW  # a claw at each toe's tip
        elif name.startswith("toe") and face == "up":
            a = np.where((h - vv) * sv < 1, CLAW, a)  # the last unit at the front
        elif side:  # scales: rings across the shins and toes (a unit wide, every 3 or 4 units)
            a = np.where(rings(vv, 3, 1), CLAW, a) if name.startswith("toe") else np.where(rings(vv, 4, 1), BAR, a)
    elif name.startswith("sickle_"):
        a[:] = CLAW
    elif name.startswith("eyelid"):
        a[:] = SIDE
    return a


def cut_mask(e, p):
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    z0, z1 = e["lo"][2], e["hi"][2]
    if e["cut"] == "wing":  # long primaries at the back, shorter forward; each lock ends in a point
        bottom = 14.5 + 6.0 * np.clip((z - z0) / (z1 - z0), 0, 1)
        lock = ((z - z0) % 2.0) / 2.0
        return y >= bottom + 1.6 * np.abs(lock - 0.5) * 2
    if e["cut"] == "shag":  # shaggy locks over the thigh, pointed tips, rounder at front and back
        lock = ((z - z0) % 2.5) / 2.5
        ends = 2.5 * (np.clip(1 - (z - z0) / 2.5, 0, 1) + np.clip(1 - (z1 - z) / 2.5, 0, 1))
        return y >= 20.2 + 1.8 * np.abs(lock - 0.5) * 2 + ends
    if e["cut"] == "fan":  # a fan of tail feathers, widest behind the tail's end, pointed tips
        half = 1.8 + 4.2 * np.clip((z1 - z) / (z1 - z0), 0, 1)
        lock = ((x + 6) % 2.0) / 2.0
        return (np.abs(x) <= half) & (z >= z0 + 1.8 * np.abs(lock - 0.5) * 2)
    return np.ones(x.shape, bool)


LASH = (54, 32, 26)


# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}  # a layers.Recorder while build() paints, and the face's key


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)
def layer_props(e, face, p, a):
    """Where white, patterns show on a raptor: white from the belly and throat up; patterns on the
    body, neck, tail, thighs and feathers, less on the head, none on the scaly legs."""
    n, y = e["name"], p[..., 1]
    lo, hi = e["lo"][1], e["hi"][1]
    t = _ramp(y, lo, max(hi, lo + 1e-3))  # 0 at the part's bottom
    if n in ("hips", "chest", "neck") or n.startswith(("tail_", "thigh_", "shag_")):
        white = 0.85 * (1 - t) + 0.15 * (face == "down")
    elif n in ("skull", "snout", "jaw"):
        white = 0.6 * (1 - t)
    else:
        white = np.full(y.shape, 0.2)
    legs = n.startswith(("shin_", "foot_", "toe", "sickle_"))
    face_part = n in ("snout", "jaw", "mouth")
    pattern = np.full(y.shape, 0.0 if legs else 0.35 if face_part else 0.8 if n == "skull" else 1.0)
    return {"white": white, "pattern": pattern}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    if w == 0 or h == 0:
        return None
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel centers, in texels
    size = np.array(e["hi"]) - np.array(e["lo"])
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
    p = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    light_face = face
    if face == "down" and e["name"].startswith(("thigh_", "shin_", "foot_", "toe", "tail_", "arm_", "forearm_", "jaw")):
        light_face = "north"  # seen when a pose lifts or folds it
    a = area(e, face, p, uu, vv, h, su, sv)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=FUR, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"]:
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "skull" and face in ("east", "west"):
        side = 1 if face == "east" else -1
        stamp_eyes(img, [eye_center(side)], EYE_STYLE)
        # A heavy brow ridge over the eye, as on its boxy skull: a unit darker, a unit less dark
        # above it, reaching a unit past the eye's ends.
        pw = len(EYES[EYE_STYLE]["pattern"][0])
        u0, v0 = eye_texel(side)
        reach = max(1, round(1 / su))
        c0, c1 = max(u0 - reach, 0), min(u0 + pw + reach, w)
        for r in range(v0 - 1, -1, -1):
            d = (v0 - r - 0.5) * sv  # units above the eye's top
            if d >= 2:
                break
            k = 0.62 if d < 1 else 0.82
            img[r, c0:c1, :3] = (img[r, c0:c1, :3] * k).astype(np.uint8)
    if e["name"] == "snout" and face in ("east", "west"):  # nostrils: slits on the sides near the tip
        # (on the front, a dark pair reads as eyes): a unit below the top, 1 to 3 units from the tip
        uf = uu * su if face == "east" else (w - uu) * su
        slit = (vv * sv >= 1) & (vv * sv < 2) & (uf >= 1) & (uf < 3)
        img[slit, :3] = (img[slit, :3] * 0.62).astype(np.uint8)
    if e["name"].startswith("eyelid"):  # a closed eye: the lash along the edge that ends up at the bottom
        lash = (uu * su < 1) if e["name"].endswith("L") else ((w - uu) * su < 1)  # a unit wide
        img[lash, :3] = LASH
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

# The stance: crouched like a bird, knees forward, ankles back, toes flat (the angles sum to 0).
L = {"thigh": -32, "shin": 58, "foot": -46, "drop": 0.0,
     "rest_thigh": -70, "rest_shin": 140, "rest_foot": -100, "rest_drop": 0.0}


def legs(base_t, base_s, base_f, swing_l=0.0, swing_r=0.0, lift_l=0.0, lift_r=0.0, lean=0.0):
    """The legs' angles: the stance, a swing and a lift per leg; `lean`, the hips' pitch (spine_0),
    taken off the thighs so the legs stand as they would and the toes stay flat."""
    q = {}
    for n, sw, lift in (("L", swing_l, lift_l), ("R", swing_r, lift_r)):
        t = base_t + sw - 0.4 * lift - lean
        s = base_s + lift
        f = base_f - 0.4 * lift
        q[f"leg_{n}_thigh"], q[f"leg_{n}_shin"], q[f"leg_{n}_foot"] = (t, 0, 0), (s, 0, 0), (f, 0, 0)
        q[f"leg_{n}_toes"] = (-(lean + t + s + f), 0, 0)  # toes flat
    return q


def tail_sway(p, n, amp, lift=0.0):
    q = {}
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = swing((0, 0, -1), amp * (0.5 if k == 0 else 1) * S(TAU * p * n - 0.6 * k), (lift if k == 0 else 0, 0, 0))
    return q


NECK = -22  # the neck raised, the head held level: an S-curve, the head above the back


def stand(t=0.0):
    q = {"root": (0, -L["drop"], 0), "neck": (NECK, 0, 0), "head": (-NECK, 0, 0)}
    q.update(legs(L["thigh"], L["shin"], L["foot"]))
    for n, s in (("L", 1), ("R", -1)):  # arms folded like a bird's wings
        q[f"arm_{n}"] = (35, 0, -s * 8)
        q[f"forearm_{n}"] = (-80, 0, 0)
    return q


def pose_idle(t):
    p = t / 3.0
    q = stand()
    br = S(TAU * p * 2)
    q["root"] = (0, q["root"][1] + 0.15 * br, 0)
    q["spine_1"] = (0.8 * br, 0, 0)
    # A bird's head: it holds still, then snaps to a new look.
    look = float(np.interp(p, [0, .2, .23, .5, .53, .78, .81, 1], [0, 0, 22, 22, -14, -14, 0, 0]))
    tilt = float(np.interp(p, [0, .5, .53, .78, .81, 1], [0, 0, 8, 8, 0, 0]))
    q["neck"] = (NECK - 4 * br, 0, 0)
    q["head"] = (-NECK, look, tilt)
    q.update(tail_sway(p, 1, 6))
    return q


def pose_walk(t):
    p = t / 0.9
    a = TAU * p
    q = stand()
    # A foot swings forward (thigh angle falling) while it's lifted: swing -sin, lift +cos.
    q.update(legs(L["thigh"], L["shin"], L["foot"], -24 * S(a), 24 * S(a), 30 * max(0.0, S(a + math.pi / 2)),
                  30 * max(0.0, S(a - math.pi / 2))))
    q["root"] = (0, q["root"][1] + 0.6 * abs(S(a)), 0)
    q["neck"] = (NECK + 6 * S(2 * a), 0, 0)  # the head bobs back and forth, like a bird's
    q["head"] = (-NECK - 6 * S(2 * a), 0, 0)
    q["spine_0"] = (0, 3 * S(a), 0)
    q.update(tail_sway(p, 1, 7))
    return q


def pose_run(t):
    p = t / 0.5
    a = TAU * p
    q = stand()
    q.update(legs(L["thigh"] - 5, L["shin"], L["foot"], -42 * S(a), 42 * S(a), 60 * max(0.0, S(a + math.pi / 2)),
                  60 * max(0.0, S(a - math.pi / 2))))
    q["root"] = (0, q["root"][1] + 1.5 * abs(S(a)) - 1.0, 0)
    q["spine_0"] = (6, 0, 0)  # leaning forward, the tail straight out behind
    q["neck"] = (NECK + 30 + 4 * S(2 * a), 0, 0)  # neck stretched forward
    q["head"] = (-NECK - 20, 0, 0)
    q.update(tail_sway(p, 1, 3, lift=-6))
    return q


def pose_rest(t, P=L):
    """Resting like a bird: legs folded under, belly and tail on the ground, the head up."""
    p = t / 4.0
    br = S(TAU * p * 2)
    q = {"root": (0, -P["rest_drop"] + 0.1 * br, 0)}
    q.update(legs(P["rest_thigh"], P["rest_shin"], P["rest_foot"]))
    for n in ("L", "R"):  # toes a little off flat, so they never lie in the belly's plane
        t = q[f"leg_{n}_toes"]
        q[f"leg_{n}_toes"] = (t[0] + 5, t[1], t[2])
    # The arms tucked up against the chest, the wings folded at its sides: hanging as when it stands,
    # their feathers went 7.3 into the ground.
    for n, s in (("L", 1), ("R", -1)):
        q[f"arm_{n}"] = (10, 0, s * 30)
        q[f"forearm_{n}"] = (-100, 0, 0)
    q["neck"] = (10, 0, 0)
    q["head"] = (-8, 8 * S(TAU * p), 0)
    q.update({f"tail_{k}": (8 if k == 0 else 0, 4 * S(TAU * p - 0.5 * k) if k else 0, 0) for k in range(len(TAIL))})
    return q


def pose_screech(t):
    """Head up, the jaw wide open, arms and wings spread."""
    p = t / 1.6
    q = stand()
    up = float(np.interp(p, [0, .15, .75, 1], [0, 1, 1, 0]))
    q["spine_0"] = (-8 * up, 0, 0)
    q["neck"] = (NECK - 20 * up, 0, 0)
    q["head"] = (-NECK - 15 * up, 0, 0)
    q["jaw"] = (float(np.interp(p, [0, .15, .25, .35, .45, .55, .75, 1], [0, 34, 26, 34, 26, 34, 0, 0])), 0, 0)
    for n, s in (("L", 1), ("R", -1)):
        q[f"arm_{n}"] = (35 - 45 * up, 0, -s * (8 + 50 * up))
        q[f"forearm_{n}"] = (-80 + 50 * up, 0, 0)
    q.update(tail_sway(p, 2, 8 * up, lift=-10 * up))
    return q


def pose_bite(t):
    """A bite: the head draws back, darts forward and down with the jaws opening, and snaps them
    shut on what it strikes; then back."""
    p = t / 0.7
    q = stand()
    d = float(np.interp(p, [0, .3, .45, .6, 1], [0, -1, 1, 1, 0]))  # -1 drawn back, 1 thrust out
    q["spine_0"] = (5 * d, 0, 0)
    q["neck"] = (NECK + 18 * d, 0, 0)
    q["head"] = (-NECK - 8 * d, 0, 0)
    q["jaw"] = (float(np.interp(p, [0, .25, .42, .5, 1], [0, 22, 34, 0, 0])), 0, 0)
    q.update(tail_sway(p, 1, 3, lift=-4 * d))
    return q


def pose_kick(t):
    """The sickle-claw kick: leaning back on one leg, the other draws up, strikes down and forward
    with the claw, and is set down again; the arms out for balance."""
    p = t / 0.8
    q = stand()
    lift = float(np.interp(p, [0, .3, .45, .6, 1], [0, 1, .5, 0, 0]))
    swing = float(np.interp(p, [0, .3, .45, .6, 1], [0, -23, -47, -18, 0]))  # negative: forward
    q.update(legs(L["thigh"], L["shin"], L["foot"], swing_l=swing, lift_l=55 * lift))
    lean = float(np.interp(p, [0, .3, .5, 1], [0, 1, 1, 0]))
    q["spine_0"] = (-8 * lean, 0, 0)
    q["neck"] = (NECK - 6 * lean, 0, 0)
    q["head"] = (-NECK + 6 * lean, 0, 0)
    for n, s in (("L", 1), ("R", -1)):
        q[f"arm_{n}"] = (35 - 30 * lean, 0, -s * (8 + 30 * lean))
        q[f"forearm_{n}"] = (-80 + 30 * lean, 0, 0)
    q.update(tail_sway(p, 1, 4, lift=-8 * lean))
    return q


# ------------------------------------------------------------------ behaviour clips (`anim.rs`)
# Ported from the Velociraptor's (`../raptor`), heavier: slower and with smaller swings. Each a
# whole-body pose over what plays below on the bones it moves (`to_bedrock.as_action`): the held
# states and the gaits loop, the rest play once, the death holds its last frame. Its height is set
# after (`ground_clips`): a standing clip's from its feet frame by frame, the fall's from its lowest
# point. No hips' pitch moves a foot: a lean is taken off the thighs (`legs`). The chest (spine_1)
# stays in line with the hips beyond a breath: bent, the hips' corners poke out of it (`ground.pokes`).

FEET = ["toe3_L", "toe3_R", "toe4_L", "toe4_R"]  # what it stands on (toe II and the sickle held up)
L.update({"low_neck": 40, "low_head": 10, "low_lean": 0, "low_crouch": 0.0, "sleep_neck": 30, "sleep_head": 10,
          "scr": [-90.0, 0.0, 60.0, -40.0, 0.0]})


def arms(out=0.0):
    """The arms folded like a bird's wings (`out` 0), or flared by `out` (1: raised, out to the
    sides and forward, the forearms open, the wing feathers spread to look big)."""
    q = {}
    for n, s in (("L", 1), ("R", -1)):
        q[f"arm_{n}"] = (35 - 65 * out, s * 30 * out, s * (-8 + 58 * out))
        q[f"forearm_{n}"] = (-80 + 60 * out, 0, 0)
    return q


def body(lean=0.0, crouch=0.0, **steps):
    """The stance, the hips pitched by `lean` (the legs standing as they were), crouched deeper by
    `crouch` (1: 14 degrees more at the hip, toes flat), the arms folded; `steps`: `legs`' swings
    and lifts."""
    q = {"root": (0, -L["drop"], 0), "spine_0": (lean, 0, 0), "neck": (NECK, 0, 0), "head": (-NECK, 0, 0)}
    q.update(legs(L["thigh"] - 14 * crouch, L["shin"] + 26 * crouch, L["foot"] - 12 * crouch, lean=lean, **steps))
    q.update(arms())
    return q


def level(q, pitch=0.0):
    """The head's pitch that holds it at `pitch` degrees in the world (0 level, down positive)."""
    return pitch - q.get("spine_0", (0, 0, 0))[0] - q.get("spine_1", (0, 0, 0))[0] - q["neck"][0]


def head_down(d=1.0, P=None):
    """The snout to the ground by `d` (1: half a unit over it, fitted in `fit_clips`): the body
    tipped forward over its feet, the tail up as a counterweight, the neck lowered, the legs bent no
    more than it takes. Callers sway the tail with `lift` -0.7 of the lean (`tail_lift`)."""
    P = L if P is None else P
    q = body(lean=P["low_lean"] * d, crouch=P["low_crouch"] * d)
    q.update({"neck": (NECK + P["low_neck"] * d, 0, 0), "head": (-NECK + P["low_head"] * d, 0, 0)})
    return q


def tail_lift(q, k=0.7):
    """The tail's lift that keeps it from rising all the way with a forward lean (at most 15
    degrees: more, the tail's root pokes out of the hips)."""
    return max(-15.0, -k * q["spine_0"][0])


def ease(p, ps, vs):
    return float(np.interp(p, ps, vs))


def pose_stare(t):
    """Half aware: frozen, the neck up, the head locked on, only the breath moving (2 s, held)."""
    q = body()
    br = S(TAU * t / 2.0)
    q.update({"spine_1": (0.6 * br, 0, 0), "neck": (NECK - 8, 0, 0)})
    q["head"] = (level(q, -4), 0, 0)
    q.update(tail_sway(t / 2.0, 1, 1))
    return q


def pose_watch(t):
    """The sentinel's look round: drawn up tall, the chest up, the head high, turning in a bird's
    snaps to one side, the other and back (4 s, held)."""
    p = t / 4.0
    br = S(TAU * p * 2)
    look = ease(p, [0, .1, .14, .4, .44, .7, .74, 1], [0, 0, 32, 32, -28, -28, 0, 0])
    q = body(lean=-20)
    q.update({"spine_1": (0.6 * br, 0, 0), "neck": (NECK - 2, 0.5 * look, 0)})
    q["head"] = (level(q, 0), 0.35 * look, 0)
    q.update(tail_sway(p, 1, 3, lift=12))
    return q


def pose_drink(t):
    """Drinking like a bird: the snout dipped, lapping, then the head tipped up to swallow (2 s, held)."""
    p = t / 2.0
    d = ease(p, [0, .15, .45, .6, .85, 1], [0.25, 1, 1, 0.35, 0.35, 0.25])
    lap = max(0.0, S(TAU * p * 6)) * float(np.clip((d - 0.9) / 0.1, 0, 1))
    q = head_down(d - 0.08 * lap)  # the snout lifts a little as the jaw drops
    q["head"] = (q["head"][0] - 16 * (1 - d), 0, 0)
    q["jaw"] = (6 * lap + 0.01, 0, 0)
    q.update(tail_sway(p, 1, 2, lift=tail_lift(q)))
    return q


def pose_sleep(t, P=None):
    """Asleep, resting: the legs folded under, the head turned back and tucked along the body,
    eyes shut, slow breath (6 s, held). The neck and head fitted to lie by the body (`fit_clips`)."""
    P = L if P is None else P
    q = pose_rest(0.0, P)
    br = S(TAU * t / 6.0 * 2)
    q["root"] = (0, -P["rest_drop"] + 0.12 * br, 0)
    q.update({"spine_1": (0.6 * br, 0, 0), "neck": (P["sleep_neck"], 60, 0), "head": (P["sleep_head"], 30, 0), "jaw": (0.01, 0, 0)})
    q.update({f"tail_{k}": (8 if k == 0 else 0.01, 0, 0) for k in range(len(TAIL))})
    q.update({"eyelid_L": (0, 0, 90), "eyelid_R": (0, 0, -90)})
    return q


def steps(a, swing, lift):
    """A walk's leg swings and lifts at phase `a`: a foot swings forward while it is lifted."""
    return dict(swing_l=-swing * S(a), swing_r=swing * S(a), lift_l=lift * max(0.0, S(a + math.pi / 2)),
                lift_r=lift * max(0.0, S(a - math.pi / 2)))


def pose_stalk(t):
    """Stalking: crouched low, the chest down, the neck out low and the head level and still, aimed;
    the tail held level and stiff; slow, high, careful steps (1.6 s a cycle, in place of the walk)."""
    p = t / 1.6
    a = TAU * p
    q = body(lean=6, crouch=1.8, **steps(a, 14, 30))
    q.update({"neck": (NECK + 32, 0, 0)})
    q["head"] = (level(q), 0, 0)
    q.update(tail_sway(p, 1, 1.5, lift=-6))  # level
    return q


def pose_approach(t):
    """Curious, coming to look: slow steps a little crouched, the neck stretched forward and low, the
    head cocked one way and the other as a bird's does, the tail level (1.4 s a cycle, in place of
    the walk)."""
    p = t / 1.4
    a = TAU * p
    q = body(lean=4, crouch=0.45, **steps(a, 16, 26))
    q.update({"neck": (NECK + 20, 0, 0)})
    q["head"] = (level(q, 6), 0, 12 * S(a))
    q.update(tail_sway(p, 1, 3, lift=-5))
    return q


def pose_alarm(t):
    """Alarm: a hiss, the jaws wide, the head drawn low and forward, the feathered arms flared out
    to look big, the tail up and lashing (1.2 s, once)."""
    p = t / 1.2
    up = ease(p, [0, .12, .8, 1], [0, 1, 1, 0])
    q = body(lean=4 * up, crouch=0.3 * up)
    q.update(arms(out=up * (1 + 0.04 * S(TAU * p * 8))))
    q.update({"neck": (NECK + 14 * up, 0, 0), "jaw": (30 * up * (1 + 0.08 * S(TAU * p * 9)), 0, 0)})
    q["head"] = (level(q, -6 * up), 0, 0)
    q.update(tail_sway(p, 2, 6 * up, lift=10 * up))
    return q


def pounce_rise(t):
    """How high the pounce's leap carries it over its feet's height (0 on the ground)."""
    return ease(t / 1.0, [0, .4, .5, .6, .7, .78, 1], [0, 0, 6, 9, 6, 0, 0])


LEAP = (0.4, 0.78)  # the pounce leaves the ground only between these times (s)


def pose_pounce(t):
    """The pounce: a deep crouch, a leap up and onto the prey feet first, the body tipped back, both
    sickle claws swung forward, the arms spread to grab, the jaws open; landing on bent legs (1.0
    s, once). In place: the game moves it forward."""
    p = t / 1.0
    c = ease(p, [0, .3, .4, .78, .86, 1], [0, 1, 0, 0, .7, 0])
    air = ease(p, [0, .36, .46, .7, .78, 1], [0, 0, 1, 1, 0, 0])
    q = body(lean=6 * c, crouch=1.6 * c)
    # In the air the hips tip back, the thighs swing up and forward, the shins and feet reach out in
    # front, the toes tipped up: the sickle claws out ahead (the world angles: thigh -75, shin -50,
    # foot -95, toes 30 up).
    fly = legs(-75, 25, -45, lean=-14)
    for n in ("L", "R"):
        fly[f"leg_{n}_toes"] = (fly[f"leg_{n}_toes"][0] - 30, 0, 0)
    fly["spine_0"] = (-14, 0, 0)
    q.update({k: tuple(np.array(q.get(k, (0, 0, 0)), float) + (np.array(v, float) - np.array(q.get(k, (0, 0, 0)), float)) * air)
              for k, v in fly.items()})
    q.update(arms(out=air))
    q.update({"neck": (NECK + 12 * c + 6 * air, 0, 0), "jaw": (30 * air, 0, 0)})
    q["head"] = (level(q, 4 * air), 0, 0)
    q.update(tail_sway(p, 1, 2, lift=6 * air - 2 * c))
    return q


def pose_sniff(t):
    """The snout down to the ground, a sniff along it side to side, and up (2.2 s, once)."""
    p = t / 2.2
    d = ease(p, [0, .22, .78, 1], [0, 1, 1, 0])
    q = head_down(d)
    q["head"] = (q["head"][0], 10 * d * S(TAU * p * 2), 0)
    q.update(tail_sway(p, 1, 3, lift=tail_lift(q)))
    return q


def pose_look(t):
    """Looking round, a bird's way: still, snap to one side, a tilt, snap to the other, back (2.8 s, once)."""
    p = t / 2.8
    look = ease(p, [0, .1, .13, .4, .43, .7, .73, 1], [0, 0, 40, 40, -36, -36, 0, 0])
    tilt = ease(p, [0, .2, .23, .4, .43, 1], [0, 0, 12, 12, 0, 0])
    q = body()
    q.update({"neck": (NECK, 0.5 * look, 0), "head": (-NECK, 0.5 * look, tilt)})
    q.update(tail_sway(p, 1, 3))
    return q


def pose_groom(t):
    """Grooming: preening the right arm's feathers, the arm held out, the neck curled round to it and
    the jaws working through the feathers (2.6 s, once)."""
    p = t / 2.6
    d = ease(p, [0, .2, .85, 1], [0, 1, 1, 0])
    q = body()
    q.update({"neck": (NECK + 30 * d, -44 * d, 0), "head": (-NECK + 6 * d, -16 * d, 0),
              "jaw": (10 * d * max(0.0, S(TAU * p * 6)) + 0.01, 0, 0), "arm_R": (35 - 30 * d, 0, 8 + 40 * d),
              "forearm_R": (-80 + 40 * d, 0, 0)})
    q.update(tail_sway(p, 1, 3))
    return q


SCRATCH_TIP = (-5.2, 1.1, 7.5)  # the right foot's middle claw (on the toes' bone, at rest)


def scratch_neck(d):
    """The scratch's body: the body tipped forward, the neck bent down and round to the right, the head low
    beside the right knee (as high as a 300 kg theropod's foot comes up), tilted to offer the throat."""
    q = body(lean=25 * d)
    q.update({"neck": (NECK + 90 * d, -60 * d, 0), "head": (-NECK - 15 * d, -8 * d, -12 * d)})
    return q


def scratch_target(q):
    """The point the claw scratches: the throat behind the jaw, from below and outside (the neck's
    end, the head's joint, out and down from it)."""
    return world(RIG, q)["head"][1] + np.array([-2.0, -4.5, 0.0])



def pose_scratch(t, P=None):
    """An itch: on the left leg, the right foot drawn up and forward to the neck, bent down and round
    to meet it, the claws raking it; then the foot set down (2.0 s, once). The leg's reach fitted
    in `fit_clips`."""
    P = L if P is None else P
    p = t / 2.0
    d = ease(p, [0, .25, .8, 1], [0, 1, 1, 0])
    on = ease(p, [0, .25, .3, .75, .8, 1], [0, 0, 1, 1, 0, 0])
    rake = 6 * S(TAU * p * 7) * on
    q = scratch_neck(d)
    st = q["leg_R_thigh"][0], q["leg_R_shin"][0], q["leg_R_foot"][0], q["leg_R_toes"][0]
    T, Z, Sh, F, To = P["scr"]
    fold = 4 * d * (1 - d)  # on the way up and down the knee bends more, so the foot clears the ground
    q.update({"leg_R_thigh": (st[0] + (T - st[0]) * d, 0, Z * d), "leg_R_shin": (st[1] + (Sh - st[1]) * d + 50 * fold + rake, 0, 0),
              "leg_R_foot": (st[2] + (F - st[2]) * d - 25 * fold - rake, 0, 0), "leg_R_toes": (st[3] + (To - st[3]) * d, 0, 0)})
    q.update(arms(out=0.15 * d))
    q.update(tail_sway(p, 1, 3, lift=tail_lift(q)))
    q.update({"eyelid_L": (0, 0, 60 * on), "eyelid_R": (0, 0, -60 * on)})  # eyes half shut, enjoying it
    return q


def pose_stretch(t):
    """Getting up: both arms spread wide and shaken, the neck long, a yawn; then the right leg
    stretched out behind (2.5 s, once)."""
    p = t / 2.5
    a = ease(p, [0, .2, .55, .65, 1], [0, 1, 1, 0, 0])
    b = ease(p, [0, .6, .72, .9, 1], [0, 0, 1, 1, 0])
    yawn = ease(p, [0, .15, .25, .45, .55, 1], [0, 0, 1, 1, 0, 0])
    q = body(swing_r=34 * b, lift_r=8 * b)
    q.update(arms(out=a * (1 + 0.06 * S(TAU * p * 8))))
    q.update({"spine_1": (-4 * a, 0, 0), "neck": (NECK - 10 * a, 0, 0), "jaw": (26 * yawn + 0.01, 0, 0)})
    q["head"] = (level(q, -14 * a), 0, 0)
    q.update(tail_sway(p, 3, 6 * a))
    q.update({"eyelid_L": (0, 0, 80 * yawn), "eyelid_R": (0, 0, -80 * yawn)})
    return q


def pose_play(t):
    """A young one's play: bouncing on bent legs, the chest wiggling side to side, the head tilting,
    mock snaps, the arms flicking out, the tail wagging (1.6 s, once). The feet stay down."""
    p = t / 1.6
    bob = max(0.0, S(TAU * p * 2))
    q = body(crouch=0.6 * bob)
    q.update({"neck": (NECK + 8 * bob, 14 * S(TAU * p), 0), "jaw": (20 * max(0.0, S(TAU * p * 3)) + 0.01, 0, 0)})
    q["head"] = (level(q, -4 * bob), 6 * S(TAU * p), 10 * bob)
    q.update(arms(out=0.3 * bob))
    q.update(tail_sway(p, 2, 8))
    return q


def pose_nuzzle(t):
    """Nuzzling a young one: the head low, a gentle nudge and a nibble (2 s, once)."""
    p = t / 2.0
    d = ease(p, [0, .2, .85, 1], [0, 1, 1, 0])
    push = max(0.0, S(TAU * p * 2))
    q = head_down(0.6 * d)
    q["head"] = (q["head"][0] - 8 * push, 0, 0)
    q["jaw"] = (8 * d * push + 0.01, 0, 0)
    q.update(tail_sway(p, 2, 3, lift=tail_lift(q)))
    return q


def pose_flinch(t):
    """Hit: a jerk back and up, the head ducked, the arms snapped in, the eyes squeezed, the tail
    lashing (0.45 s, once)."""
    p = t / 0.45
    d = ease(p, [0, .15, .45, 1], [0, 1, 0.8, 0])
    q = body(lean=-8 * d)
    q.update({"neck": (NECK + 16 * d, 0, 0), "head": (-NECK + 10 * d, 0, 0), "jaw": (12 * d + 0.01, 0, 0)})
    for n, s in (("L", 1), ("R", -1)):  # snapped in (a forearm folded to -90 lies in the arm's underside: flicker)
        q[f"arm_{n}"] = (35 + 8 * d, 0, -s * (8 - 4 * d))
        q[f"forearm_{n}"] = (-80 - 5 * d, 0, 0)
    q.update(tail_sway(p, 2, 7 * d, lift=5 * d))
    q.update({"eyelid_L": (0, 0, 70 * d), "eyelid_R": (0, 0, -70 * d)})
    return q


def pose_death(t):
    """Dying: the legs give, it topples onto its side, the neck and tail fall slack; it stays (1.4 s,
    holds). Each frame rests on its lowest point (`ground_clips`)."""
    p = t / 1.4
    fall = ease(p, [0, .3, .7, 1], [0, 0.3, 1, 1])
    q = body()
    q.update({"spine_0": (0, 0, 84 * fall), "neck": (NECK + 30 * fall, 0, -10 * fall), "head": (-NECK + 10 * fall, 0, 0),
              "jaw": (10 * fall + 0.01, 0, 0)})
    q.update(legs(L["thigh"] + 20 * fall, L["shin"] - 30 * fall, L["foot"] + 10 * fall))
    q.update({f"tail_{k}": (0.01, 0, 0) for k in range(len(TAIL))})
    q.update({"eyelid_L": (0, 0, 90 * fall), "eyelid_R": (0, 0, -90 * fall)})
    return q


BLINKS = {"idle": [1.5], "rest": [1.2, 3.0], "stare": [1.2], "watch": [1.1, 3.1], "look": [1.4], "drink": [1.4], "approach": [0.9],
          "sniff": [1.9], "nuzzle": [1.0]}
BLINK = 0.09
ANIMS = {"idle": (pose_idle, 3.0), "walk": (pose_walk, 0.9), "run": (pose_run, 0.5), "rest": (pose_rest, 4.0),
         "screech": (pose_screech, 1.6), "bite": (pose_bite, 0.7), "kick": (pose_kick, 0.8)}
OWN = tuple(ANIMS)  # its own clips, exported as they were (no loop or blend of their own)
CLIPS = {"stare": (pose_stare, 2.0), "watch": (pose_watch, 4.0), "drink": (pose_drink, 2.0), "sleep": (pose_sleep, 6.0),
         "stalk": (pose_stalk, 1.6), "approach": (pose_approach, 1.4), "alarm": (pose_alarm, 1.2), "pounce": (pose_pounce, 1.0),
         "sniff": (pose_sniff, 2.2), "look": (pose_look, 2.8), "scratch": (pose_scratch, 2.0), "stretch": (pose_stretch, 2.5),
         "play": (pose_play, 1.6), "groom": (pose_groom, 2.6), "nuzzle": (pose_nuzzle, 2.0), "flinch": (pose_flinch, 0.45),
         "death": (pose_death, 1.4)}
ANIMS.update(CLIPS)
# The held ones and the gaits loop; a clip played once must not (the engine loops a clip whose file
# says so, whatever the game asks); the death holds its last frame.
LOOPS = {"stare": True, "watch": True, "drink": True, "sleep": True, "stalk": True, "approach": True, "death": "hold_on_last_frame"}
BLENDS = {"flinch": (0.05, 0.2), "death": (0.1, 0.0), "stalk": (0.3, 0.3), "approach": (0.3, 0.3), "sleep": (0.6, 0.6),
          "watch": (0.4, 0.4), "pounce": (0.1, 0.2)}
SHUT = ("sleep", "death", "scratch", "stretch", "flinch")  # their own lids: shut, half shut, a yawn's, squeezed
FRAMES = {"watch": 40, "sleep": 36, "stalk": 32, "approach": 28, "alarm": 30, "pounce": 40, "sniff": 30, "look": 36, "scratch": 50,
          "stretch": 30, "play": 32, "groom": 40, "flinch": 12}


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}  # they swing down over the side of the head


def frame_times(name, dur):
    if name in OWN:
        n = 30 if name in ("idle", "screech", "bite", "kick") else (12 if dur < 1 else 16)
    else:
        n = FRAMES.get(name, 24)
    ts = {round(dur * k / n, 4) for k in range(n + 1)}
    for b in BLINKS.get(name, []):
        ts |= {round(b + d, 4) for d in (-BLINK, -BLINK / 2, 0, BLINK / 2, BLINK)}
    return sorted(ts)


def animations(bone_names):
    out = {}
    for name, (fn, dur) in ANIMS.items():
        frames = []
        loops = LOOPS.get(name, False)
        for t in frame_times(name, dur):
            if name in OWN:
                tt = t % dur if t < dur else 0.0
            else:  # a loop's last frame is its first; a clip played once ends where it ends
                tt = t % dur if t < dur else (0.0 if loops is True else dur)
            pose = fn(tt)
            if name not in SHUT:
                pose.update(eyelids(name, tt))
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames}
        if name not in OWN:
            out[name].update(loop=loops, blend=BLENDS.get(name, (0.25, 0.3)))
    # A clip goes over what plays below only on the bones it writes, and the exporter leaves out a bone
    # that stays at 0: kept at a hundredth of a degree, so the idle's (or resting's) head turn and tail
    # don't show through a clip that holds them still. The eyelids are left to the idle's blinks.
    for name in CLIPS:
        below = ("idle", "rest") if name == "sleep" else ("idle",)
        moved = {b for a in below for f in out[a]["frames"] for b, r in f["rot"].items() if any(abs(v) > 1e-6 for v in r)}
        for b in moved - {"eyelid_L", "eyelid_R"}:
            if all(not any(abs(v) > 1e-6 for v in f["rot"][b]) for f in out[name]["frames"]):
                for f in out[name]["frames"]:
                    f["rot"][b] = [0.01, 0.0, 0.0]
    return out


RIG = [{"name": n, "parent": p, "pivot": [float(v) for v in pv]} for n, p, pv in bones()]
PIV = {b["name"]: np.array(b["pivot"], float) for b in RIG}


# ------------------------------------------------------------------ fitting poses to the ground

GROUPS = {"toes": lambda n: n.startswith(("toe3_", "toe4_")), "body": lambda n: n in ("hips", "chest"),
          "legs": lambda n: n.startswith(("thigh_", "shin_", "foot_")), "tail": lambda n: n.startswith("tail_") and n != "tail_fan",
          "sickle": lambda n: n.startswith(("sickle_", "toe2_"))}


def fit_poses(spec):
    """The stance's drop (toes on the ground), and the rest pose's leg angles and drop (belly and
    toes on the ground, nothing below it)."""
    g = lowest(spec, stand(), GROUPS)
    L["drop"] = round(float(g["toes"]), 2)

    def rest_cost(P):
        g = lowest(spec, pose_rest(0.0, P), GROUPS)
        g.pop("sickle", None)
        floor = min(g.values())
        r = {k: v - floor for k, v in g.items()}
        return r["body"] ** 2 + r["toes"] ** 2 + 0.3 * r["tail"] ** 2
    grid = {"rest_thigh": list(range(-115, -44, 5)), "rest_shin": list(range(100, 176, 5)), "rest_foot": list(range(-155, -59, 10))}
    best = fit(grid, lambda P: rest_cost(dict(L, **P, rest_drop=0)))[1]
    P = dict(L, **best, rest_drop=0)
    g = lowest(spec, pose_rest(0.0, P), GROUPS)
    g.pop("sickle", None)
    L.update(best, rest_drop=round(float(min(g.values())), 2))
    fit_clips(spec)


def fit_clips(spec):
    """The behaviour clips' reaches. Head down (sniffing, drinking): the snout half a unit over the
    ground, with the body tipped forward and the neck lowered before the legs bend. Asleep: the head laid
    by the body, just over the ground. Scratching: the right foot's claw at the neck."""
    heads = {"head": lambda n: n in ("skull", "snout", "jaw", "mouth"), "toes": lambda n: n.startswith(("toe3_", "toe4_"))}

    def nose(P):
        g = lowest(spec, head_down(1.0, dict(L, **P)), heads)
        return g["head"] - g["toes"]

    def low_cost(P):  # the snout pointing down and forward, not tucked back past the vertical
        pitch = P["low_lean"] + P["low_neck"] + P["low_head"]
        return ((nose(P) - 0.5) ** 2 + 0.0002 * (P["low_neck"] ** 2 + P["low_head"] ** 2) + 0.0008 * P["low_lean"] ** 2
                + 1.0 * P["low_crouch"] ** 2 + 0.01 * max(0.0, pitch - 70) ** 2)
    best = fit({"low_neck": list(range(20, 141, 5)), "low_head": list(range(-60, 31, 5)), "low_lean": [0, 10, 20, 30, 40, 50],
                "low_crouch": [0, .25, .5, .75, 1, 1.5]}, low_cost)[1]
    L.update(best)
    print("head down, snout over the ground:", round(float(nose({})), 2), best)

    head = {"head": lambda n: n in ("skull", "snout", "jaw", "mouth")}

    def sleep_cost(P):
        g = lowest(spec, pose_sleep(0.0, dict(L, **P)), head)
        return (g["head"] - 0.3) ** 2 + 0.0005 * (P["sleep_neck"] ** 2 + P["sleep_head"] ** 2)
    best = fit({"sleep_neck": list(range(-10, 61, 5)), "sleep_head": list(range(-20, 41, 5))}, sleep_cost)[1]
    L.update(best)
    print("asleep, the head over the ground:", round(float(lowest(spec, pose_sleep(0.0), head)["head"]), 2), best)

    from scipy.optimize import minimize
    q0 = scratch_neck(1.0)
    target = scratch_target(q0)

    def tip(v):
        q = dict(q0, leg_R_thigh=(v[0], 0, v[1]), leg_R_shin=(v[2], 0, 0), leg_R_foot=(v[3], 0, 0), leg_R_toes=(v[4], 0, 0))
        R, tt = world(RIG, q)["leg_R_toes"]
        return R @ (np.array(SCRATCH_TIP) - PIV["leg_R_toes"]) + tt

    def inside(v):  # how far the raised knee and ankle come in under the body (outside x = -9, the thighs' side)
        q = dict(q0, leg_R_thigh=(v[0], 0, v[1]), leg_R_shin=(v[2], 0, 0), leg_R_foot=(v[3], 0, 0), leg_R_toes=(v[4], 0, 0))
        W = world(RIG, q)
        return sum(max(0.0, float(W[b][1][0]) + 9) ** 2 for b in ("leg_R_shin", "leg_R_foot") if W[b][1][1] > 14)

    def cost(v):
        return float(np.sum((tip(v) - target) ** 2)) + 3 * inside(v) + 0.00005 * float(np.sum(np.square(v)))
    bounds = [(-150, 0), (-60, 60), (0, 160), (-160, 40), (-90, 90)]
    best = min((minimize(cost, np.array(s, float), method="L-BFGS-B", bounds=bounds)
                for s in ([-90, -30, 60, -40, 0], [-110, -20, 90, -80, 30], [-70, -35, 40, -20, -20], [-60, -40, 110, -90, 20])),
               key=lambda r: r.fun)
    L["scr"] = [round(float(v), 2) for v in best.x]
    print("scratching, the claw off the neck by", round(float(np.linalg.norm(tip(best.x) - target)), 2), "the leg in the body by",
          round(inside(best.x), 2), L["scr"])


PLANTED = {"scratch": ["toe3_L", "toe4_L"], "stretch": ["toe3_L", "toe4_L"]}  # standing on one leg


def ground_clips(spec):
    """Each standing clip's height from its feet, frame by frame (`ground.grounded`): crouches,
    leans and steps stand on the ground; the pounce's leap added over it; the fall each frame on its
    lowest point. Asleep it lies as it rests. The strikes, the walk and the run set the same way, the
    run keeping its bounce: the hips' pitch carries the legs, and the bite and run sank their feet
    0.8 and 1.8 units, the screech and kick lifted them 0.3; the walk floated 2.7 at mid-stride (both
    legs swung apart, the root bobbing up), now each frame on its planted foot."""
    from ground import grounded
    A = spec["animations"]
    everything = [e["name"] for e in spec["elements"]]
    for name in CLIPS:
        if name == "sleep":
            continue
        A[name] = grounded(spec, A[name], everything if name == "death" else PLANTED.get(name, FEET))
        if name == "pounce":
            for f in A[name]["frames"]:
                f["root"][1] = round(f["root"][1] + pounce_rise(f["t"]), 4)
        for f in A[name]["frames"]:
            f["root"] = [round(float(v), 4) for v in f["root"]]
    for name in ("walk", "screech", "bite", "kick"):
        A[name] = grounded(spec, A[name], FEET)
    A["run"] = grounded(spec, A["run"], FEET, bounds=True)
    for name in ("walk", "screech", "bite", "kick", "run"):
        for f in A[name]["frames"]:
            f["root"] = [round(float(v), 4) for v in f["root"]]


# ------------------------------------------------------------------ build


def paint_sheet(E):
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"] and not e["rot"]]
    tiles = []
    for i, e in enumerate(E):
        for f in ([e["quad"]] if e["quad"] else list(FACES)):
            LAYERS["key"] = (i, f)
            img = paint_face(e, f, boxes)
            if img is not None:
                tiles.append(((i, f), img))
    sheet, uv = pack(tiles, 512)
    return sheet, uv, [k for k, _ in tiles]


def build():
    B = bones()
    E = elements()
    LAYERS["rec"] = Recorder(layer_bounds(E, 3.0))
    sheet, uv, keys = paint_sheet(E)
    LAYERS["rec"].save(OUT, keys, pack)
    LAYERS["rec"] = None
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "utahraptor", "texture": [sheet.shape[1], sheet.shape[0]],
            "bones": [{"name": n, "parent": p, "pivot": [float(v) for v in pv]} for n, p, pv in B], "elements": E}
    fit_poses(spec)
    spec["animations"] = animations([n for n, _, _ in B])
    ground_clips(spec)
    spec["fitted"] = dict(L)
    return spec, sheet


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    spec, sheet = build()
    Image.fromarray(sheet, "RGBA").save(os.path.join(OUT, "texture.png"))
    # Every coat on the same layout (out/coats/<coat>.png), to choose from in the viewer.
    os.makedirs(os.path.join(OUT, "coats"), exist_ok=True)
    chosen = COAT
    for name in COATS:
        set_coat(name)
        other, _, _ = paint_sheet(elements())
        Image.fromarray(other, "RGBA").save(os.path.join(OUT, "coats", f"{name}.png"))
    set_coat(chosen)
    with open(os.path.join(OUT, "spec.json"), "w") as f:
        json.dump(spec, f)
    print(len(spec["elements"]), "elements;", len(spec["bones"]), "bones; texture", sheet.shape[1], "x", sheet.shape[0],
          "; fitted", spec["fitted"])
