#!/usr/bin/env python3
"""The deer: one model for the white-tailed deer and the red deer, made the way Hytale makes
creatures (boxes and a hand-painted texture) with the blocky-model skill, from research alone
(README.md): its colors are rules (see `area`), painted in the white-tailed deer's summer coat.

    ~/.venvs/blocky/bin/python deer.py      # writes out/spec.json, out/texture.png

Units are 1/64 of a block (1.56 cm each), y up, the deer faces +z; painted at the house density, 32
texels a block (`TX` texels a unit), as the blocks and items are. A
white-tailed buck stands about 0.95 m at the shoulder and is 1.7 m long: here the withers are 60 up,
the body 100 from nose to tail; the head and ears a little bigger than life. A red deer is the same
model drawn bigger by its size gene (`genetics/red_deer.json`). Antlers grow on their own bones, sized
by a gene that a doe's sex turns all the way down (`build` "Ho" in the gene files): stags and bucks
carry them, does and hinds none.
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

# Texels a unit: 0.5, the house's 32 a block (a house-unit model). 32, not finer: the head's side,
# which carries the eye, is 7 texels by 6 at 32 (house.MIN_FACE is 3).

OUT = os.environ.get("DEER_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

FUR, BACK, BELLY, DARK, EARIN, MOUTH, ANTLER, HOOF, TAILTOP = range(9)
# FUR the reddish-brown summer coat, BACK darker along the spine, BELLY the white of the belly,
# throat, eye ring, muzzle band, the inside of the legs and the tail's underside, DARK the nose and
# the line round the ears, EARIN the ears' paler inside, ANTLER bone, HOOF dark horn, TAILTOP the
# tail's brown top (its extremity: the red deer's is darker).
BASE = [(164, 110, 66), (128, 84, 54), (232, 226, 214), (52, 42, 40), (196, 168, 150), (84, 42, 44), (196, 176, 140),
        (62, 52, 48), (140, 92, 56)]
if os.environ.get("DEER_COAT") == "red":  # a red deer's colors, to compare in the viewer (the game paints them from genes)
    BASE[FUR], BASE[BACK], BASE[BELLY], BASE[TAILTOP] = (134, 80, 52), (100, 62, 44), (212, 188, 146), (96, 60, 42)
SHADOW = [0.02, 0.02, 0.07, 0.95, 0.97, 0.95, 0.06, 0.74, 0.02]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (FUR, BACK, BELLY)
ROLES = [("coat", 0, 0), ("coat", 0, -1.0), ("coat", 1, 0), "fixed", "fixed", "fixed", "fixed", "fixed", ("coat", 0, -0.5)]

# Big dark eyes on the sides of the head: a near-black eye with a brown glow below and a catchlight.
EYES.update({
    "deer_dark": dict(pattern=[".OOOO.",
                               "OPPPHO",
                               "OPPPPO",
                               "OIIPPO",
                               ".OOOO."], colors={"O": (34, 24, 22), "P": (16, 12, 12), "H": (226, 228, 232),
                                                  "I": (92, 56, 38)}, halo="below"),
    # At 32 a block the head's side is 7 texels by 6: the eye is 2 by 2, near-black, the catchlight
    # up and forward (toward the light; `_m` is the other side's), the brown glow low and back.
    "deer_px": dict(pattern=["HP", "PI"], colors={"P": (16, 12, 12), "H": (226, 228, 232), "I": (92, 56, 38)}, halo="none",
                    iris="I"),
    "deer_px_m": dict(pattern=["PH", "IP"], colors={"P": (16, 12, 12), "H": (226, 228, 232), "I": (92, 56, 38)}, halo="none",
                      iris="I"),
})
EYE_STYLE = os.environ.get("DEER_EYES", "deer_px")
SKULL = ((-6.5, 62, 30), (6.5, 74, 44))
EYE_Z, EYE_Y = 39.5, 69.0
LID_X = 6.9  # the eyelids' hinge plane, just outside the head's side

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
    """The skull side's texels: (width, height) and the size of one texel in units (u, v)."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h = max(1, round(TX * (z1 - z0))), max(1, round(TX * (y1 - y0)))
    return w, h, (z1 - z0) / w, (y1 - y0) / h


def eye_center(side):
    """The eye's center on the skull's east (side > 0) or west side, in that face's texels."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    return ((z1 - EYE_Z) if side > 0 else (EYE_Z - z0)) / su, (y1 - EYE_Y) / sv


def eye_box(side):
    """(z from, z to, top, bottom) of the painted eye, in units, rounded exactly as `stamp_eyes`
    places it on the skull's side, so the eyelid covers it to the texel."""
    pat = EYES[eye_style(side)]["pattern"]
    ph, pw = len(pat), len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    u0, v0 = eye_corner(*eye_center(side), pat, w)
    za, zb = (z1 - (u0 + pw) * su, z1 - u0 * su) if side > 0 else (z0 + u0 * su, z0 + (u0 + pw) * su)
    top = y1 - v0 * sv
    return za, zb, top, top - ph * sv


LEGS = ("FL", "FR", "BL", "BR")
FRONT_X, FRONT_Z, BACK_X, BACK_Z = 6.0, 20.0, 6.0, -26.0
# The neck rises at 55 degrees from low in the chest to the head (its pivot 2.5 short of the end):
# long enough that the mouth reaches the grass.
HEAD, NECK_PITCH, NECK_LEN = np.array([0.0, 68.43, 34.91]), -55.0, 31.0
NECK0 = HEAD - (NECK_LEN - 2.5) * np.array([0, math.sin(math.radians(-NECK_PITCH)), math.cos(math.radians(-NECK_PITCH))])
EAR = [(72.0, 75.0, 1.8), (75.0, 81.0, 2.9), (81.0, 84.0, 1.9)]  # y0, y1, half width: an oval
EAR_X, EAR_Z = 5.0, (34.6, 36.2)
TAIL0 = np.array([0.0, 56.0, -36.0])  # on the rump's back face: a flap hinged on the surface
ANTLER0 = (3.9, 73.4, 35.5)  # where the left antler's burr sits on the skull


def antler(side):
    """A white-tailed buck's antler: the beam rising up, out and back, then sweeping forward, three
    tines standing up off it; the right one a mirror."""
    s = 1 if side == "L" else -1
    bone = f"antler_{side}"
    x0, y0, z0 = ANTLER0
    start = (s * x0, y0, z0)
    E = [el(f"burr_{side.lower()}", bone, (s * x0 - 1.1, y0 - 1.0, z0 - 1.3), (s * x0 + 1.1, y0 + 2.2, z0 + 1.3))]
    beam0, p1 = rod(f"beam0_{side.lower()}", bone, (start[0], y0 + 1.5, z0), (-28, 0, -s * 32), 9.0, 1.6)
    beam1, p2 = rod(f"beam1_{side.lower()}", bone, tuple(p1), (62, 0, -s * 40), 11.0, 1.4)
    E += [beam0, beam1]
    for k, (frac, length) in enumerate(((0.25, 6.0), (0.55, 7.0), (0.85, 5.0))):
        at = p1 + (p2 - p1) * frac
        tine, _ = rod(f"tine{k}_{side.lower()}", bone, tuple(at), (8 + 6 * k, 0, -s * (10 + 4 * k)), length, 1.1)
        E.append(tine)
    return E


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 50, -20)), ("spine_1", "spine_0", (0, 48, -7)),
         ("neck", "spine_1", tuple(NECK0)), ("head", "neck", tuple(HEAD)), ("jaw", "head", (0, 61.8, 43.2)),
         ("tail", "spine_0", tuple(TAIL0))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B += [(f"ear_{n}", "head", (s * EAR_X, 72.6, sum(EAR_Z) / 2)), (f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2)),
              (f"antler_{n}", "head", (s * ANTLER0[0], ANTLER0[1], ANTLER0[2]))]
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x = FRONT_X * s
            B += [(f"leg_{leg}_upper", "spine_1", (x, 42, FRONT_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 23, FRONT_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 5.5, FRONT_Z))]
        else:
            x = BACK_X * s
            B += [(f"leg_{leg}_upper", "spine_0", (x, 50, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 31, BACK_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 6.5, BACK_Z))]
    return B


def elements():
    E = [el("hips", "spine_0", (-9.5, 38, -36), (9.5, 58.0, 0)),  # reaching deep into the chest, below the withers
         el("chest", "spine_1", (-10.2, 33, -8), (10.2, 60, 28)),
         el("skull", "head", *SKULL),
         el("muzzle", "head", (-4.6, 61.4, 43), (4.6, 69.2, 52.5)),  # tapering to the nose
         el("nose", "head", (-3.7, 62.6, 52), (3.7, 67.8, 55.4)),
         el("mouth", "head", (-3.0, 61.0, 42.5), (3.0, 62.4, 51.0)),
         el("jaw", "jaw", (-3.4, 59.6, 41.5), (3.4, 62.0, 51.5))]  # shorter than the muzzle
    neck, _ = rod("neck", "neck", tuple(NECK0), (NECK_PITCH + 90, 0, 0), NECK_LEN, 9.6, back=6.0)  # slimmer than the head
    E.append(neck)
    tail = el("tail", "tail", (-3.0, 43, -39.0), (3.0, 57.0, -35.75), rot=(12, 0, 0), origin=tuple(TAIL0))
    E.append(tail)
    for s, n in ((1, "L"), (-1, "R")):
        cx = EAR_X * s
        rot, origin = (-14, 0, -s * 38), (cx, 72.6, sum(EAR_Z) / 2)
        for k, (y0, y1, hw) in enumerate(EAR):
            E.append(el(f"ear_core_{n.lower()}{k}", f"ear_{n}", (cx - hw, y0, EAR_Z[0]), (cx + hw, y1, EAR_Z[1]), rot=rot, origin=origin))
        zc = sum(EAR_Z) / 2
        E.append(el(f"fringe_ear_{n.lower()}", f"ear_{n}", (cx - 3.4, 72, zc), (cx + 3.4, 85, zc), quad="south", rot=rot,
                    origin=origin, cut="ear"))
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        lx0, lx1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down"))
        E += antler(n)
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z = FRONT_X * s, FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 3.2, 22, z - 3.2), (x + 3.2, 44, z + 3.4)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2.0, 5, z - 2.0), (x + 2.0, 24, z + 2.0)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 2.5, 0, z - 2.5), (x + 2.5, 6.4, z + 3.1)))
        else:
            x, z = BACK_X * s, BACK_Z
            xi, xo = sorted((x - 3.6 * s, x + 4.2 * s))  # the ham
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (xi, 30, z - 6.5), (xo, 56.8, z + 6)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2.2, 6, z - 2.2), (x + 2.2, 32, z + 2.2)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 2.7, 0, z - 2.7), (x + 2.7, 7.2, z + 3.3)))
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
    # At TX texels a unit; never under 1 (an antler's 1.1-unit tine is a texel, not nothing).
    return np.array(corner, float), du, dv, max(1, int(round(TX * abs(size @ du)))), max(1, int(round(TX * abs(size @ dv)))), n


def area(e, face, p, uu, vv, h):
    """A white-tailed deer's summer coat: reddish brown, darker along the back; white belly, the
    inside of the legs, the throat patch, the eye ring and the band behind the black nose; the tail
    brown above and white below; the ears dark-edged, pale inside; dark hooves, bone antlers. `p` is
    in units; `uu`, `vv` and `h` in texels (`t` the fraction down the face)."""
    name = e["name"]
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
            a = np.where(t < 0.14, BACK, np.where(y < 37.5, BELLY, FUR))
            if name == "hips" and face == "north":  # the white under the tail
                a = np.where((np.abs(x) < 4.5) & (y > 44), BELLY, a)
    elif name == "neck":
        if face in ("north", "up"):
            a[:] = BACK
        elif face in ("south", "down"):
            a = np.where(y > 54, BELLY, FUR)  # the white throat patch, high on the neck
    elif name == "skull":
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        elif side:  # the white eye ring: a texel round the painted eye, its corners left brown
            za, zb, top, bottom = eye_box(1 if face == "east" else -1)
            d = np.hypot(np.maximum(np.abs(z - (za + zb) / 2) - (zb - za) / 2, 0),
                         np.maximum(np.abs(y - (top + bottom) / 2) - (top - bottom) / 2, 0))
            a = np.where((d > 0) & (d < 0.65 / TX), BELLY, a)  # the texels beside it (0.5 / TX away), not diagonal
    elif name == "muzzle":
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        else:
            a = np.where(z > 50.2, BELLY, a)  # the white band behind the nose
            a = np.where(y < 62.6, BELLY, a)  # and the chin
    elif name == "nose":
        a[:] = DARK
        if face in ("down",) or (face != "up" and False):
            a[:] = BELLY
        if face != "south":
            a = np.where(y < 63.6, BELLY, DARK)
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else BELLY
    elif name == "mouth":
        a[:] = MOUTH
    elif name.startswith(("burr_", "beam", "tine")):
        a[:] = ANTLER
    elif name == "tail":
        a[:] = TAILTOP
        if face in ("north", "down"):  # its underside (it hangs down: the back face) is white
            a[:] = BELLY
        if side:
            a = np.where(np.abs(x) > 2.6, BELLY, a)
    elif name.startswith("ear_core"):
        a[:] = FUR
        if face == "south":
            cx = e["origin"][0]
            a = np.where(np.abs(x - cx) < 2.0, EARIN, DARK)
    elif name.startswith("thigh_"):
        a[:] = FUR
        if face == "up":
            a[:] = BACK
        if face == "down" or face == "north" or (side and ((e["lo"][0] > 0) == (face == "west"))):
            a = np.where(y < 50, BELLY, a)  # the white of the inner thigh and rump
    elif name.startswith("leg_"):
        a[:] = FUR
        if name.endswith("_paw"):
            a[:] = HOOF
            a = np.where(y > 4.6, np.where(side, BELLY, FUR), a) if face != "down" else a
        elif side and ((e["lo"][0] > 0) == (face == "west")):
            a[:] = BELLY  # the inside of the legs
    return a


def cut_mask(e, p):
    """The ear's outline: a long oval, a few hairs on its edge."""
    x, y = p[..., 0], p[..., 1]
    cx = e["origin"][0]
    t = (y - 78.2) / 6.4
    half = 3.2 * np.sqrt(np.clip(1 - t ** 2, 0, 1)) + 0.3 * ((np.floor(y * 1.3) % 4) == 0) * (np.abs(t) < 0.8)
    return (np.abs(x - cx) <= half) & (y >= 72)


LASH = (40, 30, 28)

# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)


def layer_props(e, face, p, a):
    """White where this deer is white (and up from the belly); the tail's top its extremity; the
    back and flanks where marks show (a fawn's spots: `young` in the gene files)."""
    n, y = e["name"], p[..., 1]
    lo, hi = e["lo"][1], e["hi"][1]
    low = 1 - _ramp(y, lo, max(hi, lo + 1e-3))
    white = 0.5 * low if n in ("chest", "hips", "neck") else np.full(y.shape, 0.2)
    white = np.maximum(white, np.where(a == BELLY, 0.85, 0))
    ext = np.where(a == TAILTOP, 1.0, 0.0)
    pattern = np.full(y.shape, 1.0 if n in ("hips", "chest") else (0.4 if n.startswith("thigh_") else 0.0))
    return {"white": white, "extremity": ext, "pattern": pattern}


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
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "jaw", "tail")):
        light_face = "north"
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    stacked = e["name"].startswith("ear_core")
    down = np.clip((84.5 - p[..., 1]) / 12.5, 0, 1) if stacked else None
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=p[..., 1] if not e["name"].startswith(("beam", "tine")) else None,
                            occ=occ, quad=bool(e["quad"]), seed=sum(map(ord, e["name"] + face)) % 97, down=down,
                            tuft=e["name"] not in ("skull", "muzzle", "nose"), merge=e["name"] != "skull")  # keep the eye ring
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"] == "ear":
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "skull" and face in ("east", "west"):
        s = 1 if face == "east" else -1
        stamp_eyes(img, [eye_center(s)], eye_style(s))
    if e["name"] == "nose" and face == "south" and h >= 2:  # nostrils as slits, the split lip
        c = w // 2
        if w >= 8:
            img[1, [c - 3, c - 2, c + 1, c + 2], :3] = (24, 18, 18)
            img[2:h, c - 1:c + 1, :3] = (img[2:h, c - 1:c + 1, :3] * 0.7).astype(np.uint8)
        elif w >= 3:  # at 32 the nose is 4 texels: a nostril at each edge, no split lip
            img[1, [0, w - 1], :3] = (24, 18, 18)
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

L = {"lie_front": 80, "lie_hip": -70, "lie_knee": 140, "lie_drop": 30, "graze_neck": 40, "graze_head": 30, "graze_lean": 4, "graze_splay": 0,
     "death_drop": 30, "rear_lift": 0}


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
    """Neck and head down toward the grass by `g` (0..1); the front legs straddled forward a
    little to lower the shoulders, the hooves kept flat on the ground."""
    sp = P["graze_splay"] * g
    q = {"spine_1": (P["graze_lean"] * g, 0, 0), "neck": (P["graze_neck"] * g, 0, 0), "head": (P["graze_head"] * g, 0, 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (-sp - P["graze_lean"] * g, 0, 0)
        q[f"leg_{leg}_paw"] = (sp, 0, 0)
    return q


def pose_idle(t):
    """Grazing and chewing, then the head up to look about, an ear flick and a tail flick (8 s)."""
    p = t / 8.0
    br = S(TAU * p * 3)
    g = float(np.interp(p, [0, .05, .12, .55, .62, 1], [1, 1, 1, 1, 0, 0])) if p < 0.62 else float(np.interp(p, [.62, .9, .97, 1], [0, 0, 1, 1]))
    look = (1 - g) * 14 * S(TAU * (p - 0.6) * 1.5)
    fl = max(0.0, S(TAU * (p - 0.3) * 12)) if 0.3 < p < 0.34 else 0.0
    tf = max(0.0, S(TAU * (p - 0.7) * 10)) if 0.7 < p < 0.75 else 0.0
    q = graze_pose(g)
    q.update({"root": (0, 0.15 * br, 0), "head": (q["head"][0], look, 0), "ear_L": (0, 0, -17 * fl), "ear_R": (0, 0, 2 * br),
              "tail": (-25 * tf, 0, 0)})
    q.update(chew(t, 2.5 if g > 0.5 else 1.2, 6 if g > 0.5 else 2))
    return q


def pose_walk(t):
    p = t / 1.4
    q = gait(p, (0.0, 0.5, 0.75, 0.25), 20, 38)
    q.update({"root": (0, 0.6 * abs(S(TAU * p * 2)), 0), "neck": (3 * S(TAU * p * 2), 0, 0), "head": (-3 * S(TAU * p * 2), 0, 0),
              "tail": (-6, 0, 0)})
    return q


def pose_run(t):
    """Bounding, the white tail raised as a flag (0.5 s)."""
    p = t / 0.5
    q = gait(p, (0.0, 0.1, 0.5, 0.6), 34, 64)
    c = S(TAU * p)
    q.update({"root": (0, 3.0 * max(0.0, S(TAU * p + 0.6)), 0), "spine_0": (-5 * c, 0, 0), "spine_1": (7 * c, 0, 0),
              "neck": (-12 - 6 * c, 0, 0), "head": (14, 0, 0), "ear_L": (-25, 0, 10), "ear_R": (-25, 0, -10),
              "tail": (-68, 0, 0)})  # the flag
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


# ------------------------------------------------------------------ behaviour clips (`anim.rs`)
# Each a whole-body pose over what plays below on the bones it moves (`to_bedrock.as_action`):
# held states loop, the rest play once.


def still(t, period=4.0):
    """Standing still and breathing: what a clip builds on."""
    br = S(TAU * t / period * 2)
    return {"root": (0, 0.15 * br, 0), "spine_1": (0.5 * br, 0, 0)}


def pose_stare(t):
    """Half aware: frozen, the head high and pointed, both ears cupped forward, the tail still (2 s, held)."""
    q = still(t, 2.0)
    q.update({"neck": (-14, 0, 0), "head": (6, 0, 0), "ear_L": (-10, 0, -12), "ear_R": (-10, 0, 12), "tail": (-8, 0, 0)})
    return q


def pose_graze(t):
    """Grazing: the head down at the grass, cropping and chewing, a step of the head now and then (2.5 s, held)."""
    p = t / 2.5
    q = still(t)
    q.update(graze_pose(1.0))
    q["neck"] = (q["neck"][0], 5 * S(TAU * p), 0)
    q.update(chew(t, 2.5, 6.0))
    q["ear_L"] = (0, 0, 4 * S(TAU * p * 2))
    return q


def pose_drink(t):
    """Drinking: the muzzle in the water, sipping (1.5 s, held)."""
    q = still(t)
    q.update(graze_pose(1.0))
    q["head"] = (q["head"][0] + 2 * S(TAU * t / 1.5 * 2), 0, 0)
    q.update(chew(t, 1.3, 3.0))
    return q


def pose_sniff(t):
    """Nose down to the ground and up again (2.2 s, once)."""
    p = t / 2.2
    d = float(np.interp(p, [0, .25, .75, 1], [0, 1, 1, 0]))
    q = still(t)
    q.update(graze_pose(d))
    q["head"] = (q["head"][0], 8 * d * S(TAU * p * 2), 0)
    return q


def pose_look(t):
    """Head up and round one way, then the other (3 s, once)."""
    p = t / 3.0
    yaw = float(np.interp(p, [0, .15, .4, .55, .8, 1], [0, 40, 40, -35, -35, 0]))
    q = still(t)
    q.update({"neck": (-10, 0.7 * yaw, 0), "head": (4, 0.3 * yaw, 0), "ear_L": (-0.15 * abs(yaw), 0, 0), "ear_R": (-0.15 * abs(yaw), 0, 0)})
    return q


def pose_scratch(t):
    """An itch: the head round to the flank, nibbling (2 s, once)."""
    p = t / 2.0
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    q = still(t)
    q.update({"spine_1": (0, 5 * d, 0), "neck": (30 * d, 66 * d, 0), "head": (8 * d, 10 * d, 0)})
    q.update(chew(t, 5.0, 6.0 * d))
    return q


def pose_stretch(t):
    """Getting up: the neck stretched long and low, then a hind leg pushed out behind (2.4 s, once)."""
    p = t / 2.4
    a = float(np.interp(p, [0, .2, .5, .6, 1], [0, 1, 1, 0, 0]))
    b = float(np.interp(p, [0, .55, .7, .9, 1], [0, 0, 1, 1, 0]))
    q = still(t)
    q.update({"spine_1": (6 * a, 0, 0), "neck": (20 * a, 0, 0), "head": (-16 * a, 0, 0), "ear_L": (0, 0, -20 * a), "ear_R": (0, 0, 20 * a),
              "leg_BL_upper": (30 * b, 0, 0), "leg_BL_lower": (-10 * b, 0, 0), "leg_BL_paw": (-20 * b, 0, 0)})
    q.update(chew(t, 0.5, 14 * a))  # a yawn
    return q


def pose_sleep(t, P=L):
    """Asleep, lying: the head turned back and laid along the flank, eyes shut, slow breath (6 s, held)."""
    q = pose_lie(0.0, P)
    br = S(TAU * t / 6.0 * 2)
    q.update({"root": (0, -P["lie_drop"] + 0.15 * br, 0), "spine_1": (0.8 * br, 0, 0), "neck": (40, 74, 0), "head": (8, 10, 0),
              "jaw": (0, 0, 0), "ear_L": (0, 0, 20), "ear_R": (0, 0, -20)})
    q.update({"eyelid_L": (0, 0, 90), "eyelid_R": (0, 0, -90)})
    return q


def pose_alarm(t):
    """Alarm: head snapped high, ears out, the white tail flagged straight up, a forehoof stamped twice (1.4 s, once)."""
    p = t / 1.4
    up = float(np.interp(p, [0, .1, .8, 1], [0, 1, 1, 0]))
    stamp = float(np.interp(p, [0, .25, .32, .4, .5, .57, .65, 1], [0, 0, 1, 0, 0, 1, 0, 0]))
    q = still(t)
    q.update({"spine_1": (-3 * up, 0, 0), "neck": (-20 * up, 0, 0), "head": (8 * up, 0, 0), "ear_L": (-10 * up, 0, -18 * up),
              "ear_R": (-10 * up, 0, 18 * up), "tail": (-70 * up, 0, 0),
              "leg_FR_upper": (-24 * stamp, 0, 0), "leg_FR_lower": (40 * stamp, 0, 0), "leg_FR_paw": (-16 * stamp, 0, 0)})
    q.update(chew(t, 4.0, 6.0 * up))  # a snort
    return q


def pose_rear(t, P=L):
    """Rearing (a buck's fight, a startle): up on the hind legs, the forelegs tucked and striking (1.4 s, once)."""
    p = t / 1.4
    r = float(np.interp(p, [0, .3, .65, 1], [0, 1, 1, 0]))
    strike = r * max(0.0, S(TAU * p * 3))
    q = still(t)
    q.update({"root": (0, P["rear_lift"] * r, 0), "spine_0": (-38 * r, 0, 0), "neck": (16 * r, 0, 0), "head": (14 * r, 0, 0),
              "ear_L": (24 * r, 0, 0), "ear_R": (24 * r, 0, 0), "tail": (-20 * r, 0, 0)})
    for leg in ("BL", "BR"):  # the hind legs kept upright under the tilt
        q[f"leg_{leg}_upper"] = (38 * r - 8 * r, 0, 0)
        q[f"leg_{leg}_lower"] = (10 * r, 0, 0)
        q[f"leg_{leg}_paw"] = (-2 * r, 0, 0)
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (-50 * r - 20 * strike, 0, 0)
        q[f"leg_{leg}_lower"] = (90 * r - 30 * strike, 0, 0)
        q[f"leg_{leg}_paw"] = (-30 * r, 0, 0)
    return q


def pose_play(t):
    """A fawn's play: a stiff-legged bound straight up (a pronk), landing, and another (1.2 s, once)."""
    p = t / 1.2
    hop = max(0.0, S(TAU * p * 2))
    q = still(t)
    q.update({"root": (0, 7.0 * hop, 0), "spine_0": (-4 * hop, 0, 0), "neck": (-10 * hop, 10 * S(TAU * p), 0), "head": (6 * hop, 0, 0),
              "tail": (-60 * hop, 0, 0), "ear_L": (-10, 0, -10 * hop), "ear_R": (-10, 0, 10 * hop)})
    for leg in ("FL", "FR", "BL", "BR"):
        q[f"leg_{leg}_upper"] = ((-6 if leg[0] == "F" else 6) * hop, 0, 0)
    return q


def pose_groom(t):
    """Grooming: licking the flank, the head round and down (2.5 s, once)."""
    p = t / 2.5
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    q = still(t)
    q.update({"spine_1": (0, 4 * d, 0), "neck": (44 * d, 58 * d, 0), "head": (8 * d, 8 * d, 0)})
    q.update(chew(t, 3.0, 5.0 * d))
    return q


def pose_nuzzle(t):
    """A doe nuzzling her fawn: the head low and forward, a gentle push, a lick (2 s, once)."""
    p = t / 2.0
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    push = max(0.0, S(TAU * p * 2))
    q = still(t)
    q.update(graze_pose(0.55 * d))
    q["head"] = (q["head"][0] - 8 * push, 0, 0)
    q.update(chew(t, 2.0, 4.0 * d))
    q.update({"ear_L": (10 * d, 0, 0), "ear_R": (10 * d, 0, 0)})
    return q


def pose_flinch(t):
    """Hit: a jerk aside, the head ducked, ears back, the tail clamped (0.45 s, once)."""
    p = t / 0.45
    d = float(np.interp(p, [0, .15, .45, 1], [0, 1, 0.8, 0]))
    q = still(t)
    q.update({"root": (0, -1.5 * d, 0), "spine_1": (4 * d, 0, 2 * d), "neck": (16 * d, 0, 0), "head": (10 * d, 0, 0),
              "ear_L": (24 * d, 0, 0), "ear_R": (24 * d, 0, 0), "tail": (10 * d, 0, 0)})
    return q


def pose_death(t, P=L):
    """Dying: the legs fold, it rolls onto its side, the head drops; it stays (1.4 s, holds)."""
    p = t / 1.4
    fall = float(np.interp(p, [0, .35, .7, 1], [0, 0.35, 1, 1]))
    q = {"root": (0, -P["death_drop"] * fall, 0), "spine_0": (0, 0, 84 * fall), "neck": (24 * fall, 0, -10 * fall), "head": (10 * fall, 0, 0),
         "jaw": (6 * fall, 0, 0), "ear_L": (20 * fall, 0, 0), "ear_R": (20 * fall, 0, 0)}
    for leg in ("FL", "FR", "BL", "BR"):
        q[f"leg_{leg}_upper"] = ((10 if leg[0] == "F" else -10) * fall, 0, 0)
    q.update({"eyelid_L": (0, 0, 90 * fall), "eyelid_R": (0, 0, -90 * fall)})
    return q


BLINKS = {"idle": [2.0, 5.9], "lie": [1.5, 4.4], "stare": [1.3], "graze": [1.1], "look": [1.6]}
BLINK = 0.12
ANIMS = {"idle": (pose_idle, 8.0), "walk": (pose_walk, 1.4), "run": (pose_run, 0.5), "lie": (pose_lie, 6.0),
         "stare": (pose_stare, 2.0), "graze": (pose_graze, 2.5), "drink": (pose_drink, 1.5), "sniff": (pose_sniff, 2.2),
         "look": (pose_look, 3.0), "scratch": (pose_scratch, 2.0), "stretch": (pose_stretch, 2.4), "sleep": (pose_sleep, 6.0),
         "alarm": (pose_alarm, 1.4), "rear": (pose_rear, 1.4), "play": (pose_play, 1.2), "groom": (pose_groom, 2.5),
         "nuzzle": (pose_nuzzle, 2.0), "flinch": (pose_flinch, 0.45), "death": (pose_death, 1.4)}
LOOPS = {"stare": True, "graze": True, "drink": True, "sleep": True, "death": "hold_on_last_frame"}
BLENDS = {"flinch": (0.05, 0.2), "death": (0.1, 0.0), "sleep": (0.6, 0.6), "graze": (0.4, 0.4)}


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}


def frame_times(name, dur):
    n = {"idle": 96, "lie": 60, "walk": 24, "run": 20, "sleep": 36, "flinch": 12}.get(name, 24)
    ts = {round(dur * k / n, 4) for k in range(n + 1)}
    for b in BLINKS.get(name, []):
        ts |= {round(b + d, 4) for d in (-BLINK, -BLINK / 2, 0, BLINK / 2, BLINK)}
    return sorted(ts)


def animations(bone_names):
    out = {}
    for name, (fn, dur) in ANIMS.items():
        frames = []
        loops = LOOPS.get(name, name in ("idle", "walk", "run", "lie"))
        for t in frame_times(name, dur):
            # A loop's last frame is its first; a clip played once ends where it ends.
            tt = t % dur if t < dur else (0.0 if loops is True else dur)
            pose = fn(tt)
            if name not in ("sleep", "death"):  # those keep their eyes shut
                pose.update(eyelids(name, tt))
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames, "loop": loops, "blend": BLENDS.get(name, (0.25, 0.3))}
    return out


# ------------------------------------------------------------------ fitting poses to the ground

GROUPS = {"front paws": lambda n: n in ("leg_FL_paw", "leg_FR_paw"), "back paws": lambda n: n in ("leg_BL_paw", "leg_BR_paw"),
          "body": lambda n: n in ("hips", "chest"), "muzzle": lambda n: n in ("nose", "jaw"),
          "legs": lambda n: n.startswith(("leg_", "thigh_")), "antlers": lambda n: n.startswith(("burr_", "beam", "tine"))}


def fit_poses(spec):
    """Lying (the belly on the ground, the folded legs under it, nothing below) and grazing (the
    mouth at the grass), by forward kinematics."""
    def lie_cost(P):
        g = lowest(spec, pose_lie(0.0, dict(L, **P, lie_drop=0)), GROUPS)
        f = min(g.values())
        return (g["body"] - f - 0.8) ** 2 + 0.3 * (g["front paws"] - f) ** 2 + 0.3 * (g["back paws"] - f) ** 2
    grid = {"lie_front": list(range(60, 101, 5)), "lie_hip": list(range(-100, -49, 5)), "lie_knee": list(range(115, 166, 5))}
    best = fit(grid, lie_cost)[1]
    g = lowest(spec, pose_lie(0.0, dict(L, **best, lie_drop=0)), GROUPS)
    L.update(best, lie_drop=round(float(min(g.values())), 2))

    def graze_cost(P):
        q = graze_pose(1.0, dict(L, **P))
        g = lowest(spec, q, GROUPS)
        # The mouth at the grass, the antlers clear of the ground.
        return (g["muzzle"] - 0.5) ** 2 + 0.002 * (P["graze_neck"] ** 2 + P["graze_head"] ** 2) + 0.05 * P["graze_lean"] ** 2 \
            + 20 * max(0.0, 1.0 - g["antlers"]) ** 2 + 40 * (g["front paws"] - g["back paws"]) ** 2
    best = fit({"graze_neck": list(range(40, 126, 5)), "graze_head": list(range(-20, 51, 5)), "graze_lean": [0, 4, 8, 12, 16],
                "graze_splay": [0, 6, 12, 18]}, graze_cost)[1]
    L.update(best)
    # Dying on its side: dropped until nothing is below the ground.
    g = lowest(spec, pose_death(1.4, dict(L, death_drop=0)), {"all": lambda n: True})
    L["death_drop"] = round(float(g["all"]), 2)
    # Rearing: lifted or dropped so the hind hooves stay on the ground.
    g = lowest(spec, pose_rear(0.6, dict(L, rear_lift=0)), GROUPS)
    L["rear_lift"] = round(-float(g["back paws"]), 2)
    print("dead, dropped", L["death_drop"], "; rearing, lifted", L["rear_lift"])


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
    spec = {"name": "deer", "texture": [sheet.shape[1], sheet.shape[0]],
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
