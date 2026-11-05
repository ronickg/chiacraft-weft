#!/usr/bin/env python3
"""The cat: an orange tabby with white chest, belly, muzzle and paws, pink nose and inner ears and
green eyes, made the way Hytale makes creatures (boxes and a few flat panels, a hand-painted
texture) with the blocky-model skill. No concept sheet: its color areas are rules (see `area`).

    ../dog/chihuahua/.venv/bin/python cat.py      # writes out/spec.json, out/texture.png

Units are its own (exported at 0.174 model pixels a unit, `build.SCALE`: 92 a block), y up, the
cat faces +z. Painted at the house density, 32 texels a block (`TX` texels a unit), as the blocks
and items are.

Proportions from research (README.md): a house cat is about 24 cm at the shoulder, 46 cm head and
body, 30 cm tail, the head about a quarter of the body. Here the back is 22 units up, the body 30
long, the tail about 25; the head is 1.3x a real cat's (cute, like Hytale's creatures), the legs
chunky (not Minecraft's two-texel sticks).
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
from rig import fit, swing  # noqa: E402
from ground import lowest  # noqa: E402
from house import texels  # noqa: E402

TX = texels(0.174)  # texels a unit: the house's 32 a block at the cat's export scale (build.SCALE)

OUT = os.environ.get("CAT_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

COAT, WHITE, STRIPE, PINK, NOSE, DARK = range(6)
BASE = [(226, 138, 62), (240, 230, 214), (168, 78, 34), (228, 150, 150), (206, 118, 124), (64, 30, 36)]
SHADOW = [0.02, 0.06, 0.0, 0.97, 0.97, 0.95]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
FUR = (COAT, WHITE)  # stripes stay crisp: thin areas would be merged into the coat
ROLES = [("coat", 0, 0), ("coat", 1, 0), ("coat", 0, 0), "fixed", "fixed", "fixed"]  # COAT WHITE STRIPE PINK NOSE DARK

# Cat eyes: a green iris round a tall pupil, a catchlight up toward the light. Choose with
# CAT_EYES=<name>; the others are there to compare in the viewer.
_O, _P, _H = (44, 36, 30), (16, 12, 14), (236, 240, 236)
EYES.update({
    "cat_green": dict(pattern=[".OOOO.",
                               "OHPPGO",
                               "OGPPGO",
                               "OGPPsO",
                               ".OggO."], colors={"O": _O, "P": _P, "H": _H, "G": (112, 168, 72), "g": (170, 198, 92),
                                                  "s": (196, 222, 200)}, halo="round"),
    "cat_amber": dict(pattern=[".OOOO.",
                               "OHPPAO",
                               "OAPPAO",
                               "OAPPsO",
                               ".OaaO."], colors={"O": _O, "P": _P, "H": _H, "A": (196, 132, 44), "a": (226, 176, 70),
                                                  "s": (236, 214, 170)}, halo="round"),
    "cat_round": dict(pattern=[".OOOO.",
                               "OHHPGO",
                               "OHPPGO",
                               "OPPPsO",
                               ".OGGO."], colors={"O": _O, "P": _P, "H": _H, "G": (112, 168, 72), "s": (196, 222, 200)},
                      halo="round"),
    "cat_blue": dict(pattern=[".OOOO.",
                              "OHPPBO",
                              "OBPPBO",
                              "OBPPsO",
                              ".ObbO."], colors={"O": _O, "P": _P, "H": _H, "B": (84, 134, 196), "b": (130, 176, 220),
                                                 "s": (210, 226, 240)}, halo="round"),
    # At 32 a block the face is 6 texels by 5: an eye is 2 by 2.
    "cat_px": dict(pattern=["HP", "GP"], colors={"P": _P, "H": _H, "G": (112, 168, 72)}, halo="none", iris="G"),
    "cat_px_slit": dict(pattern=["GP", "gP"], colors={"P": _P, "G": (112, 168, 72), "g": (170, 198, 92)}, halo="none",
                        iris="Gg"),
    "cat_px_in": dict(pattern=["GP", "GP"], colors={"P": _P, "G": (112, 168, 72)}, halo="none", iris="G", mirror=True),
    "cat_px_lit": dict(pattern=["gP", "GP"], colors={"P": _P, "G": (112, 168, 72), "g": (170, 198, 92)}, halo="none",
                       iris="Gg", mirror=True),
    "cat_px_glow": dict(pattern=["HP", "gG"], colors={"P": _P, "H": _H, "G": (112, 168, 72), "g": (170, 198, 92)},
                        halo="none", iris="Gg"),
})
EYE_STYLE = os.environ.get("CAT_EYES", "cat_px_lit")
EYE_Y, EYE_X = 29.2, 5.0  # (29.2: on the taller face, in the same rows) eye centers on the face: at 6 texels across, 2-texel eyes at its edges, 2 between
LID_Z = 24.4  # the eyelids' hinge line, in front of the face (z = 24)

# ------------------------------------------------------------------ the model

SKULL = ((-8, 19.6, 11), (8, 33.5, 24))  # its bottom flush with the two-texel jaw's (at 20.5 the jaw's back showed a strip under it; linecheck)
TAIL = [(20, 7.0, 4.2), (42, 7.0, 3.4), (72, 6.0, 2.6), (100, 5.0, 1.8)]  # pitch up (deg), length, width (0.4 a side thinner each: swaying, neighbors never share a plane)
TAIL0 = np.array([0.0, 18.0, -16.5])  # its top below the hips' top, even lifted running


def tail_points():
    """Start of each tail segment and the tip, from the base backward and up."""
    pts, p = [TAIL0.copy()], TAIL0.copy()
    for pitch, length, _ in TAIL:
        a = math.radians(pitch)
        p = p + np.array([0, math.sin(a), -math.cos(a)]) * length
        pts.append(p.copy())
    return pts


def el(name, bone, lo, hi, quad=None, rot=None, origin=None, cut=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad, rot=rot,
                origin=[float(v) for v in origin] if origin else None, cut=cut)


def skull_grid():
    """The skull front's texels: (width, height) and the size of one texel in units (u, v)."""
    sx, sy = SKULL[1][0] - SKULL[0][0], SKULL[1][1] - SKULL[0][1]
    w, h = max(1, round(TX * sx)), max(1, round(TX * sy))
    return w, h, sx / w, sy / h


def eye_center(cx):
    """The eye centered at x = cx on the skull's front, in its texels."""
    w, h, su, sv = skull_grid()
    return (cx - SKULL[0][0]) / su, (SKULL[1][1] - EYE_Y) / sv


def eye_box(cx):
    """(x0, x1, top, bottom) of the eye centered at x = cx, rounded exactly as `stamp_eyes` places
    it on the skull's front, so the eyelid covers it to the texel."""
    pat = EYES[EYE_STYLE]["pattern"]
    ph, pw = len(pat), len(pat[0])
    w, h, su, sv = skull_grid()
    u0, v0 = eye_corner(*eye_center(cx), pat, w)
    x0, top = SKULL[0][0] + u0 * su, SKULL[1][1] - v0 * sv
    return x0, x0 + pw * su, top, top - ph * sv


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 16, -12)), ("spine_1", "spine_0", (0, 16, -2)),
         ("neck", "spine_1", (0, 20, 11)), ("head", "neck", (0, 24, 14)), ("jaw", "head", (0, 23.5, 24.2))]  # the hinge at the jaw's back, on the face
    for s, n in ((1, "L"), (-1, "R")):
        x0, x1, top, _ = eye_box(s * EYE_X)
        B += [(f"ear_{n}", "head", (s * 5.5, 33, 17)), (f"eyelid_{n}", "head", ((x0 + x1) / 2, top, LID_Z))]
    pts = tail_points()
    B += [(f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", tuple(pts[k])) for k in range(len(TAIL))]
    for leg in ("FL", "FR", "BL", "BR"):
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z, parent, top, knee = 4.5 * s, 10.25, "spine_1", 14.5, 8
        else:
            x, z, parent, top, knee = 4.5 * s, -12.5, "spine_0", 15, 8
        B += [(f"leg_{leg}_upper", parent, (x, top, z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, knee, z)),
              (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 2.5, z))]
    return B


def elements():
    E = [el("hips", "spine_0", (-5.5, 11, -17), (5.5, 21, -2)),
         el("chest", "spine_1", (-6, 11.5, -2.5), (6, 22, 12)),
         el("neck", "neck", (-4, 17, 9), (4, 25, 15)),
         el("skull", "head", *SKULL),
         # Two texels narrower than the skull each side (at 3.5 the cheeks beside it were 1.7 texels; linecheck).
         # Two texels tall and two out from the skull (rule 7, and flush or two clear: at 4 tall it was a
         # box a texel thick, at 27 a texel proud), as a block-game cat's nose stands out; the nose
         # painted on its front (rule 4: the box it was, a pad half a texel proud, drew a rim).
         el("muzzle", "head", (-2.6, 23.6, 22.5), (2.6, 28, 29.9)),
         el("mouth", "head", (-1.7, 22.6, 24.2), (1.7, 24.0, 29.0)),  # inside the muzzle and jaw: seen when it opens
         # Two texels tall (rule 7), from inside the head (its side three texels long, two showing) to half a unit short of the muzzle's tip,
         # a little narrower than it.
         el("jaw", "jaw", (-2.2, 19.2, 22.2), (2.2, 23.6, 29.4))]
    # Ears: steps narrowing to a point (solid), plus one cut-out panel through the middle for a
    # smooth triangle with a few tufts; turned out a little and leaning out.
    steps = [(33, 35.5, 4), (35.5, 37.5, 3), (37.5, 39.5, 1.8), (39.5, 41, 0.8)]  # y0, y1, half width (a third of the head)
    for s, n in ((1, "L"), (-1, "R")):
        cx = 5.5 * s
        rot, origin = (0, s * 10, -s * 14), (cx, 33, 17)
        for k, (y0, y1, hw) in enumerate(steps):
            E.append(el(f"ear_core_{n.lower()}{k}", f"ear_{n}", (cx - hw, y0, 16), (cx + hw, y1, 18), rot=rot, origin=origin))
        E.append(el(f"fringe_ear_{n.lower()}", f"ear_{n}", (cx - 5, 33, 17), (cx + 5, 42, 17), quad="south",
                    rot=rot, origin=origin, cut="ear"))
        x0, x1, top, bottom = eye_box(s * EYE_X)  # exactly over the painted eye
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (x0, top, LID_Z - (top - bottom)), (x1, top, LID_Z), quad="down"))
    pts = tail_points()
    for k, (pitch, length, w) in enumerate(TAIL):
        p = pts[k]
        E.append(el(f"tail_{k}", f"tail_{k}", (p[0] - w / 2, p[1] - w / 2, p[2] - length), (p[0] + w / 2, p[1] + w / 2, p[2] + 0.4),
                    rot=(pitch, 0, 0), origin=tuple(p)))
    # Legs two texels thick or more (rule 7: at 4 a lower leg was a texel), the paw a white band a
    # little round it (rule 8), each part's sides (inner, outer: from the middle) half a unit or more
    # from the next one's and from those it meets in a pose (boxcheck): a turning joint keeps its
    # sides in their planes.
    for leg in ("FL", "FR", "BL", "BR"):
        s = 1 if leg[1] == "L" else -1
        side = lambda i, o: sorted((s * i, s * o))  # noqa: E731
        if leg[0] == "F":
            z = 10.25
            (x0, x1) = side(2.0, 7.0)
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x0, 8, z - 2.5), (x1, 15, z + 2.5)))
            (x0, x1) = side(1.5, 7.5)
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x0, 2.5, z - 3.0), (x1, 8, z + 3.0)))
            (x0, x1) = side(2.0, 7.0)
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x0, 0, z - 2.5), (x1, 2.5, z + 3.5)))
        else:
            z = -12.5
            # Its outer side flush with the hips' (at x + 3 it stood 0.7 texels out beside them; linecheck).
            (x0, x1) = side(1.5, 6.0)
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (x0, 8, z - 4), (x1, 18, z + 4)))  # down to the knee (y = 8), on the lower leg's top
            (x0, x1) = side(0.95, 6.75)
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x0, 2.5, z - 2.9), (x1, 8, z + 2.9)))
            (x0, x1) = side(1.95, 7.25)  # off the thigh's and hips' sides, which it meets sitting
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x0, 0, z - 3.4), (x1, 2.5, z + 3.5)))
    return E


# ------------------------------------------------------------------ painting

FACES = {"north": (2, -1), "south": (2, 1), "west": (0, -1), "east": (0, 1), "down": (1, -1), "up": (1, 1)}


def face_frame(lo, hi, face):
    """A face's corner, u and v directions (u right, v down seen from outside) and texel size."""
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
    if abs(size @ du) > 0 and abs(size @ dv) > 0:
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


WAVE = 1.0 if TX >= 0.75 else 0.0  # stripes wave a little; at 32 a block a wave only staggers them a texel a row


def stripes(t, period=5.5, width=0.3):
    """Bands across `t` (units): at 32 a block a texel is ~2.9 units, so a band is at least one
    texel and two texels of coat lie between."""
    period = max(period, 3 / TX)
    return ((t / period) % 1.0) < max(width, 1 / (TX * period))


def area(e, face, p, uu, vv):
    """Which material each texel is (the coat's design, from what a tabby looks like)."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, COAT)
    side = face in ("east", "west")
    if name in ("hips", "chest", "neck"):
        if side:  # stripes down the flanks, waving a little
            a = np.where(stripes(z + WAVE * 1.6 * np.sin(y * 0.5), 5.5, 0.36) & (y > 14), STRIPE, a)
        if face == "up":  # and across the back, meeting the flank stripes
            a = np.where(stripes(z + WAVE * 1.2 * np.sin(x * 0.6), 5.5, 0.36), STRIPE, a)
        a = np.where(y < 13.5, WHITE, a)  # the belly
        if face == "down":
            a[:] = WHITE
        if face == "south" and name != "hips":
            a = np.where(np.abs(x) < 4.5 - 0.25 * np.maximum(y - 16, 0), WHITE, a)  # the bib narrows up the chest
    elif name in ("skull",):
        if face == "south":
            a = np.where((y < 27) & (np.abs(x) < 4.5), WHITE, a)  # around the muzzle
            a = np.where((y > 31) & (np.abs(x) < 3), STRIPE, a)  # the tabby's M, a dark mark on the forehead
        if side:  # two cheek stripes from the eye back
            a = np.where((np.abs(y - 27.8) < 1.3) & (z > 17), STRIPE, a)  # a cheek stripe from the eye back
            a = np.where((y < 24) & (z > 16), WHITE, a)
        if face == "up":
            a = np.where(stripes(z + 1.5, 4.0, 0.3) & (np.abs(x) < 4), STRIPE, a)
        if face == "down":
            a = np.where(z > 15, WHITE, a)
    elif name in ("muzzle", "jaw"):
        a[:] = WHITE
        if name == "muzzle" and face == "south":  # the nose, painted where its box was (rule 4)
            a = np.where((np.abs(x) <= 1.55) & (y >= 26.5), NOSE, a)
        elif name == "muzzle" and face == "up":  # and over the front texel of the top
            a = np.where((np.abs(x) <= 1.55) & (z >= 27.5), NOSE, a)
    elif name == "mouth":
        a[:] = DARK
    elif name.startswith("ear_core"):
        if face == "south":  # one pink triangle inside a coat rim, white tufts at its base
            cx = e["origin"][0]
            a = np.where(np.abs(x - cx) < (40.2 - y) * 0.42, PINK, a)
            if name.endswith("0"):
                a = np.where((np.abs(x - cx) < 1.6) & (y < 34.2), WHITE, a)
    elif name.startswith("fringe_ear"):
        pass
    elif name.startswith("tail_"):
        k = int(name.split("_")[1])
        along = sum(L for _, L, _ in TAIL[:k]) + (vv if side or face in ("up", "down") else np.zeros_like(vv))
        a = np.where(stripes(along + 1.0, 4.2, 0.36), STRIPE, a)
        if k == len(TAIL) - 1 and face == "north":
            a[:] = STRIPE  # a dark tip
    elif name.startswith(("leg_", "thigh_")):
        if "paw" in name:
            a[:] = WHITE
        else:
            a = np.where(stripes(y + 1.0, 4.5, 0.3), STRIPE, a)
            a = np.where(y < 5.5, WHITE, a)  # white socks
            if face == "south" and name.startswith("leg_F"):
                a = np.where(y < 11, WHITE, a)
    elif name.startswith("eyelid"):
        pass
    return a


def cut_mask(e, p):
    """Cut-out panels: the ear's triangle, with a few tufts along its edges."""
    x, y = p[..., 0], p[..., 1]
    cx = e["origin"][0]
    half = np.clip((41.6 - y) * 0.5, 0, 4.4) + 0.6 * ((np.floor(y) % 3) == 0)
    return (np.abs(x - cx) <= half) & (y >= 33)


EYE_DARK = (64, 34, 26)


# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}  # a layers.Recorder while build() paints, and the face's key


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)
def layer_props(e, face, p, a):
    """Where white, colorpoint and patterns show on a cat: white first where this tabby is white
    (chest, belly, muzzle, paws), then up from below; colorpoint dark on ears, face, paws and
    tail tip; tabby markings everywhere but the muzzle, paws and ears' insides."""
    n, y = e["name"], p[..., 1]
    lo, hi = e["lo"][1], e["hi"][1]
    t = _ramp(y, lo, max(hi, lo + 1e-3))
    low = 1 - t
    if n.endswith("_paw"):
        white = np.full(y.shape, 0.95)
    elif n.startswith(("leg_", "thigh_")):
        white = 0.45 + 0.4 * low
    elif n in ("muzzle", "jaw"):
        white = np.full(y.shape, 0.9)
    elif n in ("chest", "hips", "neck"):
        white = 0.75 * low + 0.2 * (face == "down") + 0.1 * (face == "south") * (n == "chest")
    elif n.startswith("tail_"):
        white = np.full(y.shape, 0.65 if n == "tail_3" else 0.2)
    else:
        white = 0.4 * low
    white = np.maximum(white * 0.85, np.where(a == WHITE, 0.9, 0))
    ext = np.zeros(y.shape)
    if n.startswith(("ear_", "muzzle", "jaw")) or n.endswith("_paw"):
        ext[:] = 1.0
    elif n == "skull":
        ext[:] = 0.75 if face == "south" else 0.35
    elif n.startswith("leg_"):
        ext = 0.5 + 0.4 * low
    elif n.startswith("tail_"):
        ext[:] = {"tail_0": 0.3, "tail_1": 0.55, "tail_2": 0.8}.get(n, 1.0)
    pattern = np.full(y.shape, 0.2 if n in ("muzzle", "jaw") or n.endswith("_paw") else 1.0)
    return {"white": white, "extremity": ext, "pattern": pattern}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    if w == 0 or h == 0:
        return None
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
    size = np.array(e["hi"]) - np.array(e["lo"])
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
    uu, vv = uu * su, vv * sv  # texel centers, in units across the face
    p = corner + uu[..., None] * du + vv[..., None] * dv
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    light_face = face
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "tail_")):
        light_face = "north"  # only seen when a pose folds or lifts it: not a near-black underside
    if e["name"].startswith("eyelid"):  # painted as it looks closed, on the face
        p = np.stack([p[..., 0], e["hi"][1] - vv, np.full(vv.shape, LID_Z)], -1)
        light_face, du, dv, n = "south", np.array([1.0, 0, 0]), np.array([0, -1.0, 0]), np.array([0, 0, 1.0])
    a = area(e, face, p, uu, vv)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    stacked = e["name"].startswith("ear_core")  # the ear's boxes, lit as one piece
    down = np.clip((41 - p[..., 1]) / 8, 0, 1) if stacked else None
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=FUR, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=e["name"] not in ("skull",), down=down)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"] == "ear":
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "skull" and face == "south":
        u0 = SKULL[0][0]
        top = SKULL[1][1]
        stamp_eyes(img, [eye_center(cx) for cx in (-EYE_X, EYE_X)], EYE_STYLE)
    if e["name"].startswith("eyelid"):
        img[-1, :, :3] = EYE_DARK
        img[-2, :, :3] = (img[-2, :, :3] * 0.85).astype(np.uint8)
    if e["name"] == "muzzle" and face == "south" and h >= 4:  # the mouth under the nose, whisker dots (too small at 32)
        for r, c in ((2, 3), (3, 2), (3, 4)):
            img[r, c, :3] = (120, 70, 74)
        for r, c in ((1, 1), (2, 0), (1, 5), (2, 6)):
            img[r, c, :3] = (img[r, c, :3] * 0.8).astype(np.uint8)
    if e["name"] == "jaw" and face == "up":
        img[..., :3] = np.array(BASE[PINK]) * 0.7
    if e["name"].endswith("_paw") and face == "south" and w >= 5:  # three toes (a paw is 2 texels at 32)
        for c in (2, 4):
            img[:, c, :3] = (img[:, c, :3] * 0.6 + np.array([150, 110, 100]) * 0.4).astype(np.uint8)
    if e["name"].endswith("_paw") and face == "down" and h >= 5:
        img[1, [0, 1, 3, 5, 6], :3] = (210, 130, 136)
        img[3:5, 2:5, :3] = (210, 130, 136)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

LEGS = ("FL", "FR", "BL", "BR")
L = {"sit_angle": 45, "sit_hip": -80, "sit_knee": 140, "sit_lean": -6, "sit_drop": 6,
     "lie_front": 95, "lie_hip": -80, "lie_knee": 150, "lie_drop": 8}


def tail_swing(k, angle, base=(0, 0, 0)):
    pitch = math.radians(TAIL[k][0])
    return swing((0, math.sin(pitch), -math.cos(pitch)), angle, base)


def gait(p, phases, swing_amp, lift):
    out = {}
    for leg, ph in zip(LEGS, phases):
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))
        out[f"leg_{leg}_upper"] = (-swing_amp * S(a), 0, 0)
        knee = lift * up
        if leg[0] == "F":
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (knee, 0, 0), (-0.6 * knee, 0, 0)
        else:
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (-knee, 0, 0), (0.7 * knee, 0, 0)
    return out


def tail_sway(p, n, amp, base0=(0, 0, 0)):
    q = {"tail_0": tail_swing(0, amp * 0.4 * S(TAU * p * n), base0)}
    for k in range(1, len(TAIL)):
        q[f"tail_{k}"] = tail_swing(k, amp * S(TAU * p * n - 0.7 * k))
    return q


def pose_idle(t):
    p = t / 3.0
    br = S(TAU * p * 2)
    tw = max(0.0, S(TAU * (p - 0.6) * 5)) if 0.6 < p < 0.7 else 0.0
    q = {"root": (0, 0.12 * br, 0), "spine_1": (0.8 * br, 0, 0), "head": (2 * S(TAU * p), 6 * S(TAU * p), 3 * S(TAU * p)),
         "ear_L": (0, 0, -12 * tw), "ear_R": (0, 0, 2 * br)}
    q.update(tail_sway(p, 1, 9))
    return q


def pose_walk(t):
    p = t / 0.9
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 22, 30)
    bob = abs(S(TAU * p))
    q.update({"root": (0, 0.3 * bob, 0), "head": (2 * S(TAU * p * 2), 3 * S(TAU * p), 0)})
    q.update(tail_sway(p, 1, 6))
    return q


def pose_run(t):
    p = t / 0.5
    q = gait(p, (0.0, 0.08, 0.5, 0.58), 38, 50)
    c = S(TAU * p)
    q.update({"root": (0, 1.0 * max(0.0, S(TAU * p + 0.6)), 0), "spine_0": (-5 * c, 0, 0), "spine_1": (8 * c, 0, 0),
              "neck": (-5 * c, 0, 0), "ear_L": (15, 0, -8), "ear_R": (15, 0, 8)})
    q.update({"tail_0": tail_swing(0, 0, (-25 + 6 * c, 0, 0)), "tail_1": tail_swing(1, 4 * c, (25, 0, 0)),
              "tail_2": tail_swing(2, 4 * c, (25, 0, 0)), "tail_3": tail_swing(3, 3 * c)})
    return q


def pose_sit(t, P=L):
    """Upright: front legs straight, haunches on the ground, the tail curled round the paws."""
    p = t / 3.0
    br = S(TAU * p * 2)
    a = P["sit_angle"]
    q = {"root": (0, -P["sit_drop"] + 0.1 * br, 0), "spine_0": (-a, 0, 0), "spine_1": (0.8 * br, 0, 0),
         "neck": (a * 0.5, 0, 0), "head": (a * 0.5 + 2 * S(TAU * p), 8 * S(TAU * p), 4 * S(TAU * p))}
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (a + P["sit_hip"], 0, 0)
        q[f"leg_{leg}_lower"] = (P["sit_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["sit_hip"] - P["sit_knee"], 0, 0)
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (a + P["sit_lean"], 0, 0)
        q[f"leg_{leg}_lower"] = (0, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["sit_lean"], 0, 0)
    # The tail lies on the ground and curls forward along one side (fitted, `fit_tail`); the tip flicks.
    for k, r in enumerate(P.get("sit_tail") or [(0, 0, 0)] * len(TAIL)):
        q[f"tail_{k}"] = tail_swing(k, 8 * S(TAU * p * 2), r) if k == len(TAIL) - 1 else tuple(r)
    return q


def pose_lie(t, P=L):
    """A loaf: belly down, front paws tucked under the chest, tail along one side."""
    p = t / 4.0
    br = S(TAU * p * 2)
    q = {"root": (0, -P["lie_drop"] + 0.12 * br, 0), "spine_1": (0.8 * br, 0, 0), "head": (0, 4 * S(TAU * p), 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (P["lie_front"], 0, 0)
        q[f"leg_{leg}_lower"] = (-80, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_front"] + 80, 0, 0)
    for leg, s in (("BL", 1), ("BR", -1)):
        q[f"leg_{leg}_upper"] = (P["lie_hip"], 0, s * 8)
        q[f"leg_{leg}_lower"] = (P["lie_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_hip"] - P["lie_knee"], 0, 0)
    for k, r in enumerate(P.get("lie_tail") or [(0, 0, 0)] * len(TAIL)):
        q[f"tail_{k}"] = tail_swing(k, 6 * S(TAU * p), r) if k == len(TAIL) - 1 else tuple(r)
    return q


def pose_meow(t):
    """Standing, the head comes up and the mouth opens twice."""
    p = t / 1.6
    q = pose_idle(t * 3.0 / 1.6 * 0.5)
    lift = float(np.interp(p, [0, .1, .8, 1], [0, -10, -10, 0]))
    hx, hy, hz = q["head"]
    q["head"] = (hx + lift, hy * 0.3, hz * 0.3)
    q["jaw"] = (float(np.interp(p, [0, .15, .3, .4, .55, .7, 1], [0, 24, 6, 0, 26, 0, 0])), 0, 0)
    return q


def pose_swipe(t):
    """A paw swipe: the weight back, a forepaw drawn up and out, then raked down and across, claws
    first; a hiss."""
    p = t / 0.6
    q = pose_idle(0)
    up = float(np.interp(p, [0, .35, .5, .62, 1], [0, 1, .4, 0, 0]))
    rake = float(np.interp(p, [0, .35, .5, .62, 1], [0, 0, 1, .6, 0]))
    q["leg_FL_upper"] = (-105 * up - 40 * rake, 18 * up - 22 * rake, 0)  # -X reaches forward and up
    q["leg_FL_lower"] = (15 * up + 10 * rake, 0, 0)  # nearly straight: the paw up by the face, then raked down
    q["leg_FL_paw"] = (-45 * up, 0, 0)  # curled, claws out
    q["spine_1"] = (-10 * up, 0, 0)  # rearing a little
    q["head"] = (6 * rake, 0, 0)
    q["jaw"] = (14 * up, 0, 0)
    return q


def pose_bite(t):
    """A bite: drawn back, then a quick lunge with the jaws opening, snapping shut."""
    p = t / 0.5
    q = pose_idle(0)
    d = float(np.interp(p, [0, .25, .45, .6, 1], [0, -1, 1, 1, 0]))  # -1 drawn back, 1 lunging
    q["spine_1"] = (4 * d, 0, 0)
    q["neck"] = (10 * d, 0, 0)
    q["head"] = (4 * d, 0, 0)
    q["jaw"] = (float(np.interp(p, [0, .2, .4, .48, 1], [0, 18, 26, 0, 0])), 0, 0)
    return q


BLINKS = {"idle": [1.2], "sit": [1.8], "lie": [1.0, 3.0], "meow": [0.4]}
BLINK = 0.09
ANIMS = {"idle": (pose_idle, 3.0), "walk": (pose_walk, 0.9), "run": (pose_run, 0.5), "sit": (pose_sit, 3.0),
         "lie": (pose_lie, 4.0), "meow": (pose_meow, 1.6), "swipe": (pose_swipe, 0.6), "bite": (pose_bite, 0.5)}


def eyelids(name, t):
    a = -90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (a, 0, 0), "eyelid_R": (a, 0, 0)}


def frame_times(name, dur):
    n = 24 if name in ("meow", "swipe", "bite") else (12 if dur < 1 else 16)
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
          "back legs": lambda n: n.startswith("thigh_") or n.startswith("leg_B") and not n.endswith("_paw"),
          "rump": lambda n: n == "hips", "chest": lambda n: n == "chest", "tail": lambda n: n.startswith("tail_"),
          "front legs": lambda n: n.startswith("leg_F") and not n.endswith("_paw")}


# Where the tail's segment ends should lie in a resting pose: on the ground, curling forward
# along the cat's right side toward its front paws.
TAIL_TARGETS = {"sit": [(-3, 1.7, -22), (-8.5, 1.7, -18), (-10.5, 1.7, -12), (-9.5, 1.7, -7)],
                "lie": [(-3.5, 1.7, -20.5), (-8.5, 1.7, -15.5), (-9.5, 1.7, -9.5), (-9, 1.7, -4.5)]}


def fit_tail(spec, pose_fn, key, P):
    """The tail's rotations (pitch and yaw per segment) that put each segment's end on its target
    without going below the ground: forward kinematics plus a small optimizer."""
    from scipy.optimize import minimize
    from ground import corners, world
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}
    pts = tail_points()
    els = {e["name"]: e for e in spec["elements"]}
    targets = np.array(TAIL_TARGETS[key], float)

    def cost(v):
        pose = pose_fn(0.0, dict(P, **{f"{key}_tail": [(v[2 * k], v[2 * k + 1], 0) for k in range(len(TAIL))]}))
        W = world(spec["bones"], pose)
        c = 0.0
        for k in range(len(TAIL)):
            R, t = W[f"tail_{k}"]
            end = R @ (pts[k + 1] - piv[f"tail_{k}"]) + t
            c += np.sum((end - targets[k]) ** 2)
            low = ((corners(els[f"tail_{k}"]) - piv[f"tail_{k}"]) @ R.T + t)[:, 1].min()
            c += 50 * max(0.0, 0.2 - low) ** 2
        return c + 0.0005 * np.sum(np.square(v))
    best = None
    for start in ([60, 0, -40, 40, -40, 40, -40, 30], [90, -20, -60, 50, -50, 40, -40, 20], [40, 0, 0, 0, 0, 0, 0, 0]):
        r = minimize(cost, np.array(start, float), method="Powell", options={"maxiter": 20000, "xtol": 0.01, "ftol": 1e-6})
        if best is None or r.fun < best.fun:
            best = r
    return [[round(float(best.x[2 * k]), 2), round(float(best.x[2 * k + 1]), 2), 0.0] for k in range(len(TAIL))], round(float(best.fun), 2)


def fit_poses(spec):
    """Sit and lie: angles and drops that put the paws (and the body, lying) on the ground."""
    def low(fn, P):
        return lowest(spec, fn(0.0, P), GROUPS)

    def sit_cost(P):
        g = low(pose_sit, P)
        g.pop("tail", None)
        floor = min(g.values())
        r = {k: v - floor for k, v in g.items()}
        P["_drop"] = floor
        return 3 * r["front paws"] ** 2 + r["back paws"] ** 2 + r["rump"] ** 2 + 0.5 * r["back legs"] ** 2
    grid = {"sit_angle": list(range(15, 51, 5)), "sit_hip": list(range(-100, -59, 5)), "sit_knee": list(range(110, 166, 5)),
            "sit_lean": [-14, -8, -2, 4]}
    best = fit(grid, lambda P: sit_cost(dict(L, **P, sit_drop=0)))[1]
    P = dict(L, **best, sit_drop=0)
    L.update(best, sit_drop=round(min(low(pose_sit, P).values()), 2))

    def lie_cost(P):
        g = low(pose_lie, P)
        g.pop("tail", None)
        floor = min(g.values())
        r = {k: v - floor for k, v in g.items()}
        return r["chest"] ** 2 + r["rump"] ** 2 + 0.5 * r["front paws"] ** 2 + 0.5 * r["back paws"] ** 2
    grid = {"lie_front": list(range(70, 121, 5)), "lie_hip": list(range(-110, -59, 10)), "lie_knee": list(range(110, 171, 10))}
    best = fit(grid, lambda P: lie_cost(dict(L, **P, lie_drop=0)))[1]
    P = dict(L, **best, lie_drop=0)
    L.update(best, lie_drop=round(min(low(pose_lie, P).values()), 2))
    L["sit_tail"], L["sit_tail_err"] = fit_tail(spec, pose_sit, "sit", L)
    L["lie_tail"], L["lie_tail_err"] = fit_tail(spec, pose_lie, "lie", L)


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
            img = paint_face(e, f, boxes)
            if img is not None:
                tiles.append(((i, f), img))
    sheet, uv = pack(tiles, 512)
    LAYERS["rec"].save(OUT, [k for k, _ in tiles], pack)
    LAYERS["rec"] = None
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "cat", "texture": [sheet.shape[1], sheet.shape[0]],
            "bones": [{"name": n, "parent": p, "pivot": [float(v) for v in pv]} for n, p, pv in B], "elements": E}
    fit_poses(spec)
    spec["animations"] = animations([n for n, _, _ in B])
    spec["fitted"] = {k: v for k, v in L.items()}
    return spec, sheet


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    spec, sheet = build()
    Image.fromarray(sheet, "RGBA").save(os.path.join(OUT, "texture.png"))
    with open(os.path.join(OUT, "spec.json"), "w") as f:
        json.dump(spec, f)
    print(len(spec["elements"]), "elements;", len(spec["bones"]), "bones; texture", sheet.shape[1], "x", sheet.shape[0],
          "; fitted", {k: v for k, v in L.items()})
