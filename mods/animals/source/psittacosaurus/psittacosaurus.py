#!/usr/bin/env python3
"""Psittacosaurus: the Jehol fern forest's small, common plant eater and the Utahraptor's prey,
made the way Hytale makes creatures (boxes, a few flat panels, a hand-painted texture) with the
blocky-model skill, from research (README.md). Its colors are the one dinosaur's known from its
skin (Vinther et al. 2016): dark brown above, pale below (countershading for a shaded forest
floor), dark cheek horns, darker flecks on the back, a fringe of long bristles along the top of
its tail.

    ~/.venvs/blocky/bin/python psittacosaurus.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each), y up, it faces +z. An adult *Psittacosaurus
lujiatunensis* is about 1.6 m long and 0.5 m at the hips, walking on its hind legs: here 101 from
the beak to the tail's tip, the hips about 31 up in its stance; the head (a deep, short, parrot-
beaked skull with horns out of its cheeks) a little bigger than life.
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
from house import TX  # noqa: E402

# TX: texels a unit, 0.5 (32 a block). The eye's face, the skull's side, is 6 by 6 texels at 32 (the
# house rule wants 3 or more), so it paints at 32.

OUT = os.environ.get("PSITTA_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

SKIN, BACK, BELLY, RIM, BEAK, MOUTH, CLAW, HORN, BRISTLE = range(9)
# SKIN the brown of the flanks, BACK the dark brown above (and its flecks), BELLY the pale
# underside, RIM the dark gaps between its small scales, BEAK dark horn, MOUTH the inside, CLAW the
# claws, HORN the cheek horns (dark, as in the fossil's skin), BRISTLE the tail's long bristles.
BASE = [(136, 98, 64), (80, 56, 40), (214, 194, 156), (110, 78, 52), (104, 94, 82), (110, 56, 52), (70, 60, 52),
        (48, 40, 36), (92, 70, 52)]
SHADOW = [0.03, 0.03, 0.07, 0.03, 0.74, 0.95, 0.74, 0.74, 0.03]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (SKIN, BACK, BELLY, RIM)
ROLES = [("coat", 0, 0), ("coat", 0, -1.2), ("coat", 1, 0), ("coat", 0.3, -0.6), "fixed", "fixed", "fixed", "fixed",
         ("coat", 0, -1.5)]

# At 32 a block the skull's side is 6 texels by 6: a 3 by 2 eye (a catchlight up and forward, a dark
# pupil, a dark-brown iris), as the Protoceratops's. Drawn for the left side (east); the right gets it
# mirrored, so both catchlights point forward. No record of its eye's color: a dark brown.
_P, _H, _I, _a = (20, 16, 14), (226, 226, 222), (104, 70, 40), (140, 96, 54)
EYES.update({
    "psitta_px3": dict(pattern=["HPI", "IPa"], colors={"P": _P, "H": _H, "I": _I, "a": _a}, halo="none", iris="Ia"),
    "psitta_px": dict(pattern=["HP", "IP"], colors={"P": _P, "H": _H, "I": _I}, halo="none", iris="I"),
})
for _k in [k for k in EYES if k.startswith("psitta_px")]:
    EYES[_k + "_m"] = dict(EYES[_k], pattern=[row[::-1] for row in EYES[_k]["pattern"]])
EYE_STYLE = os.environ.get("PSITTA_EYES", "psitta_px3")
SKULL = ((-5.5, 36.0, 23.5), (5.5, 47.0, 36.0))  # deep and short, as wide as it is tall
EYE_Z, EYE_Y = 31.5, 43.0  # high on the side, toward the back
LID_X = 5.9  # the eyelids' hinge plane, just outside the head's side

# ------------------------------------------------------------------ the model

NECK_BASE = np.array([0.0, 32.0, 19.0])  # inside the chest's front top
NECK_ANGLE, NECK_LEN = 42.0, 10.0  # rising forward, its end deep in the skull
TAIL = [(12, 9.0), (11, 7.4), (10, 5.8), (9, 4.4), (8, 3.0)]  # length, width
TAIL0 = np.array([0.0, 33.0, -8.0])


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
                origin=[float(v) for v in origin] if origin is not None else None, cut=cut)


def rotm(rot):
    if not rot:
        return np.eye(3)
    rx, ry, rz = (math.radians(a) for a in rot)
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


def skull_grid():
    """The skull's side in texels: (width along z, height) and the units a texel spans (u, v)."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h = max(1, round(TX * (z1 - z0))), max(1, round(TX * (y1 - y0)))
    return w, h, (z1 - z0) / w, (y1 - y0) / h


def eye_style(side):
    """The design on that side (+1 left, the east face; -1 right): the right one mirrored."""
    return EYE_STYLE + ("_m" if side < 0 and EYE_STYLE + "_m" in EYES else "")


def eye_center(side):
    """The eye's center on the skull's side (east face for side > 0, west otherwise), in its texels."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    cu = (z1 - EYE_Z) if side > 0 else (EYE_Z - z0)
    return cu / su, (y1 - EYE_Y) / sv


def eye_box(side):
    """(z0, z1, top, bottom) of the painted eye, from the same corner `stamp_eyes` uses, so the
    eyelid covers it to the texel."""
    pat = EYES[eye_style(side)]["pattern"]
    ph, pw = len(pat), len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    u0, v0 = eye_corner(*eye_center(side), pat, w)
    za, zb = (z1 - (u0 + pw) * su, z1 - u0 * su) if side > 0 else (z0 + u0 * su, z0 + (u0 + pw) * su)
    return za, zb, y1 - v0 * sv, y1 - (v0 + ph) * sv


# The arm's middle: its outside half a unit past the chest's side (at 8.4 it stood out of it by
# a texel, a rim; linecheck), so it comes out from under the chest.
ARM_X = 6.4


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 33, 0)), ("spine_1", "spine_0", (0, 33, 5)),
         ("neck", "spine_1", tuple(NECK_BASE)), ("head", "neck", tuple(neck_end())), ("jaw", "head", (0, 38.5, 29.0))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B += [(f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2)),
              (f"arm_{n}", "spine_1", (s * 8.0, 29.5, 17.0)), (f"forearm_{n}", f"arm_{n}", (s * ARM_X, 22.5, 17.0)),
              (f"hand_{n}", f"forearm_{n}", (s * ARM_X, 16.5, 17.5))]
    pts = tail_points()
    B += [(f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", tuple(pts[k])) for k in range(len(TAIL))]
    for s, n in ((1, "L"), (-1, "R")):
        x = 5.0 * s
        B += [(f"leg_{n}_thigh", "spine_0", (x, 31, 0)), (f"leg_{n}_shin", f"leg_{n}_thigh", (x, 21, 0)),
              (f"leg_{n}_foot", f"leg_{n}_shin", (x, 10.5, -0.5)), (f"leg_{n}_toes", f"leg_{n}_foot", (x, 2.2, -0.5))]
    return B


def elements():
    # highest over the hips; out over the thighs' tops (at 6.5 the thighs' backs showed beside them as
    # rims a texel wide; linecheck), 0.3 past the chest's sides and 0.7 past the thighs', so no face meets
    # another in any pose (boxcheck)
    E = [el("hips", "spine_0", (-8.3, 27, -10), (8.3, 39, 6)),
         el("chest", "spine_1", (-8, 25, 4), (8, 36.0, 22)),  # a barrel of a body, slung under the back
         # The neck: a box from inside the chest out along the neck's angle to the head's joint,
         # inside the skull, so they share a point and never come apart (`ground.pokes`).
         el("neck", "neck", NECK_BASE + np.array([-3.0, -3.0, -1.5]), NECK_BASE + np.array([3.0, 3.0, NECK_LEN + 0.5]),
            rot=(-NECK_ANGLE, 0, 0), origin=tuple(NECK_BASE)),
         el("skull", "head", *SKULL),
         # the upper beak's base, out over two texels from the skull (at 40, 1.8; linecheck)
         el("snout", "head", (-3.6, 38.5, 35.5), (3.6, 45.5, 41.0)),
         # its hooked tip, a parrot's: out over two texels from the snout (at 2.2 units its sides read as a rim; linecheck)
         el("beak", "head", (-2.2, 36.4, 40.4), (2.2, 42.4, 45.5)),
         el("mouth", "head", (-3.0, 37.6, 30), (3.0, 39.0, 38.8)),
         # deep, ending in the lower beak; down and out over two texels from the skull (at 34 and 39.5 it showed
         # under and before it by 1.8 texels, a rim; linecheck)
         el("jaw", "jaw", (-4.2, 31.5, 27.5), (4.2, 38.6, 40.5))]
    for s, n in ((1, "L"), (-1, "R")):
        x0, x1 = sorted((s * 5.0, s * 8.0))
        E.append(el(f"horn_{n.lower()}", "head", (x0, 36.5, 29.0), (x1, 38.6, 31.0)))  # the cheek horn, sticking out
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        lx0, lx1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down"))
        ax = ARM_X * s  # the shoulder (x = 8) inside both the chest and the arm
        # down over two texels under the chest (at 22, 1.5; linecheck)
        # The arm, forearm and hand each half a unit round the one below: the forearm 3 units a side, two
        # texels (at 2, one texel, its whole face was a line beside the arm; linecheck).
        E.append(el(f"arm_{n}", f"arm_{n}", (ax - 2.1, 20.5, 15.0), (ax + 2.1, 31, 19.0)))
        E.append(el(f"forearm_{n}", f"forearm_{n}", (ax - 1.5, 16, 15.5), (ax + 1.5, 23.5, 18.5)))
        E.append(el(f"hand_{n}", f"hand_{n}", (ax - 2.0, 13, 15.0), (ax + 2.0, 16.5, 20.0)))  # a grasping hand, off the ground
    pts = tail_points()
    for k, (length, w) in enumerate(TAIL):
        p = pts[k]
        hh = max(w * 0.95, 3.2)  # the tip at least 2 texels high (at 2.85 its sides were a 1-texel line; linecheck)
        E.append(el(f"tail_{k}", f"tail_{k}", (-w / 2, p[1] - hh / 2, p[2] - length), (w / 2, p[1] + hh / 2, p[2] + 0.5)))
    # The bristles: a fringe of long quills standing up along the top of the tail's base (the
    # Senckenberg specimen's, up to 16 cm), one cut-out panel on each of the first two segments.
    for k in (0, 1):
        length, w = TAIL[k]
        p = pts[k]
        top = p[1] + w * 0.95 / 2
        E.append(el(f"bristles_{k}", f"tail_{k}", (0, top - 1.0, p[2] - length + 0.5), (0, top + (9.5 if k == 0 else 7.0), p[2] - 0.5),
                    quad="east", cut="bristles"))
    for s, n in ((1, "L"), (-1, "R")):
        x = 5.0 * s
        # the big drumstick, its top inside the hips (at x + 3.4 its back showed beside them a texel wide;
        # linecheck), 0.4 inside the chest's side and 0.5 outside the shin's
        xi, xo = sorted((x - 3.4 * s, x + 2.6 * s))
        E.append(el(f"thigh_{n}", f"leg_{n}_thigh", (xi, 20, -5), (xo, 34, 5.5)))
        E.append(el(f"shin_{n}", f"leg_{n}_shin", (x - 2.1, 10, -2.6), (x + 2.1, 22, 1.8)))
        E.append(el(f"foot_{n}", f"leg_{n}_foot", (x - 1.6, 2, -2.2), (x + 1.6, 11, 1.2)))
        E.append(el(f"toes_{n}", f"leg_{n}_toes", (x - 2.8, 0, -1.6), (x + 2.8, 2.4, 6.0)))
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


def pebbles(e, face, p):
    """Small rounded scales with dark gaps: which texels are the gaps. At 32 a texel is two units,
    so the scales are big (6+ units) or they turn to noise; none on the head and legs."""
    name = e["name"]
    if name.startswith(("leg_", "shin_", "foot_", "toes_", "arm", "forearm", "hand", "horn")) or name in ("skull", "snout", "jaw", "neck"):
        return np.zeros(p.shape[:-1], bool)
    size = 7.0 if not name.startswith("tail_") else 6.0
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a, b = {"east": (z, y), "west": (z, y), "south": (x, y), "north": (x, y), "up": (x, z), "down": (x, z)}[face]
    row = size * 0.87
    j = np.floor(b / row)
    a2 = a / size + 0.5 * (j % 2)
    fa, fb = a2 - np.floor(a2) - 0.5, b / row - j - 0.5
    return (fa * size) ** 2 + (fb * row) ** 2 > (0.5 * size - 1.0) ** 2


def flecks(p, seed=7):
    """Darker flecks on the back (the fossil's skin has dark spots on the shoulders and back): one
    a cell of 6 units, at most."""
    x, z = p[..., 0], p[..., 2]
    cx, cz = np.floor(x / 6.0), np.floor(z / 6.0)
    h = np.sin(cx * 12.9898 + cz * 78.233 + seed) * 43758.5453
    h = h - np.floor(h)
    return h < 0.3


def area(e, face, p, uu, vv, h):
    """Dark brown above, the flanks a warmer brown, pale below with a soft line between (its
    countershading); darker flecks over the back; the beak dark horn, the cheek horns darker."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, SKIN)
    side = face in ("east", "west")
    t = vv / max(h, 1)
    if name in ("hips", "chest", "neck") or name.startswith("tail_"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        elif side or face in ("south", "north"):
            a = np.where(t < 0.34, BACK, np.where(t > 0.66, BELLY, SKIN))
            a = np.where((t < 0.55) & flecks(p, 3), BACK, a)
    elif name == "skull":
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        elif side:
            a = np.where(t < 0.3, BACK, SKIN)
    elif name == "snout":
        # Skin over the snout, the beak's horn only on its front (a parrot's cere and bill).
        a = np.where(z < 38.6, BACK if face == "up" else SKIN, BEAK)
        if face == "down":
            a[:] = MOUTH
    elif name == "beak":
        a[:] = BEAK
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else np.where(z > 37.5, BEAK, BELLY)  # the lower beak's horny front
    elif name == "mouth":
        a[:] = MOUTH
    elif name.startswith("horn"):
        a[:] = HORN
    elif name.startswith("bristles"):
        a[:] = BRISTLE
    elif name.startswith("thigh_"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        else:
            a = np.where(t < 0.3, BACK, np.where(t > 0.8, BELLY, SKIN))
            a = np.where((t < 0.6) & flecks(p, 5), BACK, a)
    elif name.startswith(("shin_", "foot_", "arm_", "forearm_")):
        if side and ((e["lo"][0] > 0) == (face == "west")):
            a[:] = BELLY  # the inside of the limbs
    elif name.startswith(("toes_", "hand_")):
        a = np.where((face == "south") | (y < e["lo"][1] + 1.0), CLAW, SKIN)
        if face == "down":
            a[:] = BELLY
    if not name.startswith(("beak", "mouth", "eyelid", "horn", "bristles")):
        gap = pebbles(e, face, p) & np.isin(a, (SKIN, BELLY))
        a = np.where(gap, RIM, a)
    return a


def cut_mask(e, p, uu, vv, su, sv):
    """Which texels of the bristle panel are there: separate quills a texel wide, every other
    column, longest over the tail's base and shorter back along it."""
    y, z = p[..., 1], p[..., 2]
    lo, hi = e["lo"], e["hi"]
    k = np.clip((hi[2] - z) / max(hi[2] - lo[2], 1e-3), 0, 1)  # 0 at the panel's front
    tall = (hi[1] - lo[1]) * (1.0 - 0.45 * k) * (0.75 + 0.25 * ((np.floor(uu) % 4) == 1))
    return (y <= lo[1] + tall) & ((np.floor(uu) % 2) == 1)


LASH = (40, 30, 24)

# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def layer_props(e, face, p, a):
    """White where this one is white (the pale belly); its marks where its back is."""
    n, y = e["name"], p[..., 1]
    white = np.where(a == BELLY, 0.85, 0.15)
    ext = np.where(np.isin(a, (BEAK, HORN)), 1.0, 0.0)
    body = n in ("hips", "chest", "neck") or n.startswith(("tail_", "thigh_"))
    pattern = np.full(y.shape, 1.0 if body else 0.3)
    return {"white": white, "extremity": ext, "pattern": pattern}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    if w == 0 or h == 0:
        return None
    size = np.array(e["hi"]) - np.array(e["lo"])
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel centers, in texels
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
    p = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    light_face = face
    if face == "down" and e["name"].startswith(("thigh_", "shin_", "foot_", "tail_", "arm_", "forearm_", "hand_", "jaw")):
        light_face = "north"  # seen when a pose lifts or folds it
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=False, merge=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"]:
        img[..., 3] = np.where(cut_mask(e, p, uu, vv, su, sv), 255, 0)
    if e["name"] == "skull" and face in ("east", "west"):
        side = 1 if face == "east" else -1
        stamp_eyes(img, [eye_center(side)], eye_style(side))
    if e["name"].startswith("eyelid"):  # a closed eye: the lash along the edge that ends up at the bottom
        col_idx = 0 if e["name"].endswith("L") else w - 1
        k = 1.0 if w >= 3 else 0.5
        img[:, col_idx, :3] = np.round(img[:, col_idx, :3] * (1 - k) + np.array(LASH) * k).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

# The stance: on its hind legs, the knees forward and the ankles back as a bird's, the toes flat
# (the angles sum to 0); `lean` tips the body forward at the hips (the legs keep their place).
L = {"thigh": -26, "shin": 50, "foot": -40, "drop": 0.0, "graze_lean": 20, "graze_neck": 30, "graze_head": 20,
     "rest_thigh": -70, "rest_shin": 140, "rest_foot": -100, "rest_drop": 0.0}


def legs(base_t, base_s, base_f, swing_l=0.0, swing_r=0.0, lift_l=0.0, lift_r=0.0, lean=0.0):
    q = {}
    for n, sw, lift in (("L", swing_l, lift_l), ("R", swing_r, lift_r)):
        t = base_t + sw - 0.4 * lift - lean
        s = base_s + lift
        f = base_f - 0.4 * lift
        q[f"leg_{n}_thigh"], q[f"leg_{n}_shin"], q[f"leg_{n}_foot"] = (t, 0, 0), (s, 0, 0), (f, 0, 0)
        q[f"leg_{n}_toes"] = (-(t + s + f + lean), 0, 0)  # toes flat on the ground
    return q


def tail_sway(p, n, amp, lift=0.0):
    q = {}
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = swing((0, 0, -1), amp * (0.5 if k == 0 else 1) * S(TAU * p * n - 0.6 * k), (lift if k == 0 else 0, 0, 0))
    return q


def arms(fold=1.0):
    """The arms held in front of the chest, the hands turned in (it never walked on them grown)."""
    q = {}
    for n, s in (("L", 1), ("R", -1)):
        q[f"arm_{n}"] = (-10 * fold, 0, -s * 4)
        q[f"forearm_{n}"] = (-30 * fold, 0, 0)
        q[f"hand_{n}"] = (10 * fold, 0, s * 10)
    return q


def stand(lean=0.0):
    q = {"root": (0, -L["drop"], 0), "spine_0": (lean, 0, 0), "neck": (0, 0, 0), "head": (0, 0, 0)}
    q.update(legs(L["thigh"], L["shin"], L["foot"], lean=lean))
    q.update(arms())
    return q


def chew(t, rate=2.0, amp=8.0):
    return {"jaw": (amp * max(0.0, S(TAU * t * rate)), 0, 0)}


def graze_pose(g, P=L):
    """Tipped forward at the hips, the neck and head down to the ferns, by `g` (0..1)."""
    q = stand(P["graze_lean"] * g)
    q["neck"] = (P["graze_neck"] * g, 0, 0)
    q["head"] = (P["graze_head"] * g, 0, 0)
    q["tail_0"] = (-0.8 * P["graze_lean"] * g, 0, 0)  # the tail kept about level
    return q


def pose_idle(t):
    """Cropping ferns and chewing, then up to look about, the tail swaying slowly (8 s)."""
    p = t / 8.0
    br = S(TAU * p * 3)
    g = float(np.interp(p, [0, .5, .6, .86, .96, 1], [1, 1, 0, 0, 1, 1]))
    look = (1 - g) * 18 * S(TAU * (p - 0.6) * 1.6)
    q = graze_pose(g)
    q["root"] = (0, q["root"][1] + 0.2 * br, 0)
    q["spine_1"] = (0.6 * br, 0, 0)
    q["head"] = (q["head"][0], look, 0)
    q.update(tail_sway(p, 2, 4, lift=-0.8 * L["graze_lean"] * g))
    q.update(chew(t, 2.0 if g > 0.5 else 0.6, 8 if g > 0.5 else 2))
    return q


def pose_walk(t):
    p = t / 1.0
    a = TAU * p
    q = stand(4)
    # A foot swings forward (thigh angle falling) while it's lifted: swing -sin, lift +cos.
    q.update(legs(L["thigh"], L["shin"], L["foot"], -22 * S(a), 22 * S(a), 28 * max(0.0, S(a + math.pi / 2)),
                  28 * max(0.0, S(a - math.pi / 2)), lean=4))
    q["root"] = (0, q["root"][1] + 0.6 * abs(S(a)), 0)
    q["neck"] = (-4 + 3 * S(2 * a), 0, 0)
    q["head"] = (4 - 3 * S(2 * a), 0, 0)
    q["spine_1"] = (0, 2 * S(a), 0)
    q.update(tail_sway(p, 1, 7))
    return q


def pose_run(t):
    """A quick run, leaning forward, the tail straight out behind (0.55 s)."""
    p = t / 0.55
    a = TAU * p
    q = stand(12)
    q.update(legs(L["thigh"] - 4, L["shin"], L["foot"], -38 * S(a), 38 * S(a), 55 * max(0.0, S(a + math.pi / 2)),
                  55 * max(0.0, S(a - math.pi / 2)), lean=12))
    q["root"] = (0, q["root"][1] + 1.4 * abs(S(a)) - 1.0, 0)
    q["neck"] = (-6, 0, 0)
    q["head"] = (-6, 0, 0)
    q.update(tail_sway(p, 1, 3, lift=-12))
    return q


def pose_lie(t, P=L):
    """Lying down like a bird: the legs folded under, belly and tail on the ground, the head up (6 s)."""
    p = t / 6.0
    br = S(TAU * p * 2)
    q = {"root": (0, -P["rest_drop"] + 0.15 * br, 0), "spine_1": (0.6 * br, 0, 0)}
    q.update(legs(P["rest_thigh"], P["rest_shin"], P["rest_foot"]))
    q.update(arms(1.4))
    q["neck"] = (12, 0, 0)
    q["head"] = (-6, 8 * S(TAU * p), 0)
    q.update({f"tail_{k}": (6 if k == 0 else 0, 4 * S(TAU * p - 0.5 * k) if k else 0, 0) for k in range(len(TAIL))})
    return q


BLINKS = {"idle": [2.6, 6.8], "lie": [1.5, 4.4]}
BLINK = 0.12
ANIMS = {"idle": (pose_idle, 8.0), "walk": (pose_walk, 1.0), "run": (pose_run, 0.55), "lie": (pose_lie, 6.0)}


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}


def frame_times(name, dur):
    n = {"idle": 96, "lie": 60, "walk": 20, "run": 16}.get(name, 16)
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

GROUPS = {"toes": lambda n: n.startswith("toes_"), "body": lambda n: n in ("hips", "chest"),
          "tail": lambda n: n.startswith("tail_"), "muzzle": lambda n: n in ("beak", "jaw"),
          "hands": lambda n: n.startswith("hand_")}


def fit_poses(spec):
    """The stance's drop (toes on the ground); cropping (the beak at the ferns, the toes still on
    the ground, nothing below them); lying (belly and toes on the ground, nothing below)."""
    g = lowest(spec, stand(), GROUPS)
    L["drop"] = round(float(g["toes"]), 2)

    def graze_cost(P):
        g = lowest(spec, graze_pose(1.0, dict(L, **P)), GROUPS)
        floor = g["toes"]
        return (g["muzzle"] - floor - 0.5) ** 2 + 0.002 * (P["graze_neck"] ** 2 + P["graze_head"] ** 2) \
            + 0.01 * P["graze_lean"] ** 2 + 40 * min(0.0, g["hands"] - floor) ** 2 + 40 * min(0.0, g["tail"] - floor) ** 2
    best = fit({"graze_lean": list(range(0, 46, 5)), "graze_neck": list(range(0, 71, 5)), "graze_head": list(range(-10, 61, 5))},
               graze_cost)[1]
    L.update(best)

    def rest_cost(P):
        g = lowest(spec, pose_lie(0.0, P), GROUPS)
        g.pop("muzzle", None)
        floor = min(g.values())
        r = {k: v - floor for k, v in g.items()}
        return r["body"] ** 2 + r["toes"] ** 2 + 0.3 * r["tail"] ** 2
    grid = {"rest_thigh": list(range(-95, -44, 5)), "rest_shin": list(range(100, 166, 5)), "rest_foot": list(range(-130, -59, 10))}
    best = fit(grid, lambda P: rest_cost(dict(L, **P, rest_drop=0)))[1]
    P = dict(L, **best, rest_drop=0)
    g = lowest(spec, pose_lie(0.0, P), GROUPS)
    g.pop("muzzle", None)
    L.update(best, rest_drop=round(float(min(g.values())), 2))


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
    sheet, uv = pack(tiles, 256)  # the layers pack at the same width (`Recorder.save`)
    LAYERS["rec"].save(OUT, [k for k, _ in tiles], pack)
    LAYERS["rec"] = None
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "psittacosaurus", "texture": [sheet.shape[1], sheet.shape[0]],
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
