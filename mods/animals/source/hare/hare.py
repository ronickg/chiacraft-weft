#!/usr/bin/env python3
"""The hare: one model for the brown hare and the mountain hare, made the way Hytale makes creatures
(boxes and a hand-painted texture) with the blocky-model skill, from research alone (README.md):
its colors are rules (see `area`), painted in the brown hare's coat.

    ~/.venvs/blocky/bin/python hare.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each), y up, the hare faces +z. Painted at the house
density, 32 texels a block (`TX` texels a unit), as the blocks and items are. A brown
hare is 55-65 cm head and body: here 38 from the nose to the rump; the head and ears a little
bigger than life (Hytale's way), the hind feet as long as they are (15 cm, 10 units).

The rest pose stands the legs straight down from the body, as the skill builds two-legged
creatures; every pose crouches them (`crouch`: thigh forward, shin back, the long hind foot flat),
as a hare stands, with the body at its crouched height (the ground at y = 0).
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

# TX texels a unit: 32 a block (house units, 64 a block). The side of the head, which carries the
# eye, is 8 by 7 units, 4 by 3.5 texels at 32: over house.MIN_FACE, so no finer density is needed.

OUT = os.environ.get("HARE_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

FUR, BACK, RUFOUS, BELLY, PALE, TIP, EARIN, NOSE, MOUTH, TEETH, CLAW, TAIL = range(12)
# FUR the grizzled yellow-brown of the flanks, BACK darker along the spine, RUFOUS the warm neck,
# chest and legs, BELLY and PALE (the ring round the eye, the muzzle's sides) white, TIP the black
# ear tips (both hares'), TAIL the tail's top (black on a brown hare: its extremity color, `layer_props`).
BASE = [(170, 130, 84), (128, 96, 64), (176, 116, 70), (232, 224, 208), (214, 198, 172), (50, 40, 38), (196, 160, 146),
        (150, 104, 98), (84, 40, 42), (230, 220, 196), (78, 64, 56), (50, 40, 38)]
SHADOW = [0.02, 0.02, 0.02, 0.07, 0.05, 0.95, 0.97, 0.97, 0.95, 0.1, 0.02, 0.95]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (FUR, BACK, RUFOUS, BELLY, PALE)
ROLES = [("coat", 0, 0), ("coat", 0, -1.0), ("coat", 0.15, 0.25), ("coat", 1, 0), ("coat", 0.7, 0), "fixed",
         "fixed", "fixed", "fixed", "fixed", "fixed", ("coat", 1, 0)]

# Big eyes high on the sides of the head (a hare sees nearly all round): a dark rim, a golden-brown
# iris, a big pupil and a catchlight toward the front, a warmer glow below.
EYES.update({
    "hare_amber": dict(pattern=[".OOO.",
                                "OPPHO",
                                "OAPPO",
                                ".OaO."], colors={"O": (46, 30, 24), "P": (18, 12, 12), "H": (232, 230, 220),
                                                   "A": (176, 112, 44), "a": (212, 156, 70)}, halo="below"),
    "hare_dark": dict(pattern=[".OOO.",
                               "OPPHO",
                               "OPPPO",
                               ".OIO."], colors={"O": (44, 30, 26), "P": (24, 16, 16), "H": (232, 230, 220),
                                                  "I": (112, 66, 40)}, halo="below"),
    "hare_bead": dict(pattern=[".OO..",
                               "OPHO.",
                               "OPPO.",
                               ".OO.."], colors={"O": (58, 36, 26), "P": (22, 14, 14), "H": (232, 230, 220)}, halo="below"),
    "hare_gold": dict(pattern=[".AAA.",
                               "APPHA",
                               "APPPA",
                               ".aaa."], colors={"P": (20, 14, 14), "H": (236, 232, 220), "A": (168, 104, 40),
                                                 "a": (210, 152, 64)}, halo="below"),
    # At 32 a block the side of the head is 4 texels by 4: an eye is 2 by 2, the big dark pupil
    # with a catchlight toward the front above it and the amber iris before it below (`HARE_EYES`
    # picks; the other two and the big ones are for comparison). Drawn for the right side (u from
    # the front); the left gets it mirrored.
    "hare_px": dict(pattern=["HP",
                             "AP"], colors={"P": (18, 12, 12), "H": (232, 230, 220), "A": (176, 112, 44)}, halo="none",
                    iris="A"),
    "hare_px_light": dict(pattern=["HP",
                                   "AA"], colors={"P": (18, 12, 12), "H": (232, 230, 220), "A": (176, 112, 44)},
                          halo="none", iris="A"),
    "hare_px3": dict(pattern=["HPP",
                              "AAa"], colors={"P": (18, 12, 12), "H": (232, 230, 220), "A": (176, 112, 44),
                                              "a": (212, 156, 70)}, halo="none", iris="Aa"),
})
EYE_STYLE = os.environ.get("HARE_EYES", "hare_px")
EYE_RING = os.environ.get("HARE_RING", "0") == "1"  # the brown hare's pale ring round the eye
SKULL = ((-3.75, 13, 7), (3.75, 20, 15))
EYE_Z, EYE_Y = 12.0, 17.0  # high on the side, toward the front
LID_X = 4.15  # the eyelids' hinge plane, just outside the head's side (x = 3.75): two lids lying inside it never meet

# ------------------------------------------------------------------ the model


def el(name, bone, lo, hi, quad=None, rot=None, origin=None, cut=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad, rot=rot,
                origin=[float(v) for v in origin] if origin else None, cut=cut)


def skull_side():
    """The side of the skull's texels: (width, height) and the size of one texel in units (u, v)."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h = max(1, round(TX * (z1 - z0))), max(1, round(TX * (y1 - y0)))
    return w, h, (z1 - z0) / w, (y1 - y0) / h


def eye_corner_side(side):
    """The eye's top-left texel (u0, v0) on one side of the skull: placed on the right side (east,
    u from the front) and mirrored onto the left (west, u from the back), so both eyes sit at the
    same place on the head."""
    pat = EYES[EYE_STYLE]["pattern"]
    pw = len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_side()
    u0, v0 = eye_corner((z1 - EYE_Z) / su - 0.01, (y1 - EYE_Y) / sv, pat)  # a half texel rounds to the front
    return (u0 if side > 0 else w - u0 - pw), v0


def eye_box(side):
    """(z0, z1, top, bottom) of the eye on one side, in units, exactly where `paint_face` stamps it."""
    pat = EYES[EYE_STYLE]["pattern"]
    ph, pw = len(pat), len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_side()
    u0, v0 = eye_corner_side(side)
    za, zb = (z1 - (u0 + pw) * su, z1 - u0 * su) if side > 0 else (z0 + u0 * su, z0 + (u0 + pw) * su)
    return za, zb, y1 - v0 * sv, y1 - (v0 + ph) * sv


LEGS = ("FL", "FR", "BL", "BR")
FRONT_Z, BACK_Z, FRONT_X, BACK_X = 2.2, -10.5, 2.5, 4.0
HIP_Y, KNEE_Y, ANKLE_Y = 11.0, 3.5, -5.7  # straight down at rest: thigh 7.5, shin 9.2
EAR = [(19.0, 23.0, 1.7), (23.0, 27.0, 1.5), (27.0, 29.0, 1.1), (29.0, 30.0, 0.5)]  # y0, y1, half width
EAR_Z, EAR_X = (9.2, 10.4), 1.9


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 11, -8)), ("spine_1", "spine_0", (0, 11, -5.2)),
         ("neck", "spine_1", (0, 14.5, 1.5)), ("head", "neck", (0, 14.6, 8)), ("jaw", "head", (0, 13.6, 13.5)),
         ("nose", "head", (0, 16.4, 18.9)), ("tail", "spine_0", (0, 14, -17))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B += [(f"ear_{n}", "head", (s * EAR_X, 19.6, sum(EAR_Z) / 2)), (f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2))]
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x = FRONT_X * s
            B += [(f"leg_{leg}_upper", "spine_1", (x, 11, FRONT_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 5, FRONT_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 1.4, FRONT_Z))]
        else:
            x = BACK_X * s
            B += [(f"leg_{leg}_upper", "spine_0", (x, HIP_Y, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, KNEE_Y, BACK_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, ANKLE_Y, BACK_Z))]
    return B


def elements():
    E = [el("hips", "spine_0", (-6.0, 5, -16), (6.0, 17.5, -5)),  # the tall round rump
         el("rump", "spine_0", (-4.0, 6.5, -17.5), (4.0, 16, -15.5)),
         # The back slopes down to the shoulders; its front 0.4 short of the skull's back (at 4.5 a texel of neck
         # showed between them; linecheck).
         el("chest", "spine_1", (-4.5, 5.5, -8), (4.5, 16.9, 6.6)),
         el("neck", "neck", (-3, 13.4, 0.5), (3, 16.4, 11)),  # inside the skull's height where it enters
         el("skull", "head", *SKULL),
         # Blunt, the split lip; out 4.2 from the skull's front, over two texels (at 17.5, out 2.5, its sides and top
         # read as a rim; linecheck). Its back in to 13.4 so its sides hold 3 texels, not 2 (clear of the mouth's back).
         # Its bottom half a unit under the skull's, so the jaw can sit half a unit under it (both 0.4 off before; linecheck).
         el("muzzle", "head", (-2.5, 12.5, 13.4), (2.5, 17.6, 19.2)),
         el("mouth", "head", (-1.5, 13.95, 14), (1.5, 14.4, 16.1)),
         el("jaw", "jaw", (-2, 12.0, 13), (2, 14, 16.6)),  # shorter than the muzzle; half a unit below it (0.9 read as a rim; linecheck)
         el("tail", "tail", (-2, 11.5, -20), (2, 15.5, -14.5), rot=(-25, 0, 0), origin=(0, 14, -17))]  # the scut, held up
    for s, n in ((1, "L"), (-1, "R")):
        cx = EAR_X * s
        # Long ears: thin steps narrowing to a rounded tip (solid), a cut-out panel through the
        # middle for the outline; laid back a little and splayed out.
        rot, origin = (-18, 0, -s * 10), (cx, 19.6, sum(EAR_Z) / 2)
        for k, (y0, y1, hw) in enumerate(EAR):
            E.append(el(f"ear_core_{n.lower()}{k}", f"ear_{n}", (cx - hw, y0, EAR_Z[0]), (cx + hw, y1, EAR_Z[1]), rot=rot, origin=origin))
        zc = sum(EAR_Z) / 2
        E.append(el(f"fringe_ear_{n.lower()}", f"ear_{n}", (cx - 2.5, 19.5, zc), (cx + 2.5, 30.8, zc), quad="south", rot=rot,
                    origin=origin, cut="ear"))
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        lx0, lx1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down"))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z = FRONT_X * s, FRONT_Z
            # The shoulder's bottom half a unit below the chest's (at 4.6, 0.9 below, it read as a rim; linecheck).
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 1.5, 5.0, z - 1.6), (x + 1.5, 12, z + 1.6)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 1, 1.2, z - 1.1), (x + 1, 5.4, z + 1.1)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 1.4, 0, z - 1.7), (x + 1.4, 1.6, z + 2.6)))
        else:
            x, z = BACK_X * s, BACK_Z
            xi, xo = sorted((x - 2.0 * s, x + 2.4 * s))  # the haunch, bulging out (its inside clear of the front legs)
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (xi, 1.0, z - 4.2), (xo, 12.6, z + 4.2)))
            # Rule 7 (Ronald, 11-05: no boxes a texel thick): the shin two texels square (it was 2 by 2.4 units, a
            # texel), 0.8 out from the knee and 0.4 past the haunch's outside, so its faces keep clear of the hips' and
            # the front legs' (boxcheck); the foot 0.35 out from it all round (narrower, the shin's edge would show).
            xa, xb = sorted((x - 1.2 * s, x + 2.8 * s))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (xa, ANKLE_Y - 0.4, z - 2.0), (xb, KNEE_Y + 0.6, z + 2.0)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (xa - 0.35, ANKLE_Y - 1.8, z - 2.35), (xb + 0.35, ANKLE_Y + 0.3, z + 8.6)))
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
    if abs(size @ du) > 0 and abs(size @ dv) > 0:
        w, h = max(1, w), max(1, h)  # a thin face (an ear's tip) keeps a texel, or it vanishes
    return np.array(corner, float), du, dv, w, h, n


def rotm(rot):
    if not rot:
        return np.eye(3)
    rx, ry, rz = (math.radians(a) for a in rot)
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


def area(e, face, p, uu, vv, h):  # uu, vv texel centers in texels (h of them down the face)
    """A brown hare's coat: grizzled yellow-brown, darker along the back; the neck, chest and legs
    warmer; the belly and the inside of the legs white; a pale ring round the eye and pale muzzle
    sides; black ear tips; the tail black on top, white below."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, FUR)
    side = face in ("east", "west")
    t = vv / max(h, 1)
    if name in ("hips", "rump", "chest"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        else:
            a = np.where(t < 0.22, BACK, np.where(y < 7.5, BELLY, FUR))
            if name == "chest":
                a = np.where((a == FUR) & (z > 0), RUFOUS, a)
                if face == "south":
                    a = np.where(y < 9, BELLY, RUFOUS)
    elif name == "neck":
        a[:] = RUFOUS
        if face == "up":
            a[:] = BACK
        if face in ("down", "south"):
            a = np.where(y < 12.5, BELLY, RUFOUS)
    elif name == "skull":
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        elif side:
            d = np.hypot(z - EYE_Z, (y - EYE_Y) * 1.15)
            if EYE_RING:
                a = np.where((d > 2.4) & (d < 3.6), PALE, a)  # the pale ring round the eye
            a = np.where((y < 14.6) & (z > 11), PALE, a)  # pale cheeks toward the mouth
        elif face == "south":
            a = np.where(y > 18.4, BACK, a)
    elif name == "muzzle":
        if face == "up":
            a[:] = FUR
        elif face == "down":
            a[:] = BELLY
        else:
            a = np.where(y < 15.4, PALE, a)  # pale lips and whisker pads
        # Rule 4 (Ronald, 11-05: the nose painted on the muzzle, never a pad proud of it): the pink nose where
        # its box was, on the front and the front edge of the top.
        if face == "south":
            a = np.where((np.abs(x) < 1.6) & (y > 15.8), NOSE, a)  # the muzzle's front is two texels wide
        elif face == "up":
            a = np.where((np.abs(x) < 1.6) & (z > 18.2), NOSE, a)
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else BELLY
    elif name == "mouth":
        a[:] = MOUTH
    elif name == "nose":
        a[:] = NOSE
    elif name == "tail":
        a[:] = BELLY
        if face in ("up", "north") or side:
            a = np.where(y > 13.6, TAIL, BELLY) if side else np.full(x.shape, TAIL)
    elif name.startswith("ear_core"):
        a[:] = FUR
        if face == "south":  # the inside: pale, furred at the rim
            cx = e["origin"][0]
            a = np.where((np.abs(x - cx) < 1.0) & (y < 27.6), EARIN, a)
        a = np.where(y > 27.6, TIP, a)  # black tips
    elif name.startswith(("leg_", "thigh_")):
        a[:] = RUFOUS
        if name.startswith("thigh_"):
            a[:] = FUR
            if face == "up":
                a[:] = BACK
            if face == "down" or (side and ((e["lo"][0] > 0) == (face == "west"))):
                a[:] = BELLY  # the inside of the haunch
        if name.endswith("_paw"):
            a[:] = PALE if face in ("up", "down") else RUFOUS
            if name.startswith("leg_B"):
                a[:] = PALE if face == "down" else RUFOUS
        if face == "down" and not name.startswith("thigh_"):
            a[:] = PALE
    return a


def cut_mask(e, p):
    """The ear's outline: a long leaf, rounded at the tip, with a few hairs along its edges."""
    x, y = p[..., 0], p[..., 1]
    cx = e["origin"][0]
    t = np.clip((y - 19.5) / 11.0, 0, 1)
    half = 1.9 * np.sqrt(np.clip(1 - t ** 3, 0, 1)) + 0.4 * ((np.floor(y * 1.3) % 4) == 0) * (t < 0.9)
    return (np.abs(x - cx) <= half) & (y >= 19.5)


LASH = (42, 30, 26)

# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}  # a layers.Recorder while build() paints, and the face's key


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)


def layer_props(e, face, p, a):
    """A hare's coat: white where this brown hare is white (belly, inner legs) and up from there;
    the tail's top is its extremity (black on a brown hare, white on a mountain hare); fine
    grizzling (the pattern) on the back and flanks."""
    n, y = e["name"], p[..., 1]
    lo, hi = e["lo"][1], e["hi"][1]
    low = 1 - _ramp(y, lo, max(hi, lo + 1e-3))
    white = 0.6 * low if n in ("chest", "hips", "rump", "neck") else np.full(y.shape, 0.3)
    white = np.maximum(white, np.where(np.isin(a, (BELLY, PALE, TAIL)), 0.85, 0))
    ext = np.where(a == TAIL, 1.0, 0.0)
    pattern = np.full(y.shape, 1.0 if n in ("hips", "rump", "chest", "neck", "skull") or n.startswith("thigh_") else 0.3)
    return {"white": white, "extremity": ext, "pattern": pattern}


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
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "jaw", "tail")):
        light_face = "north"  # seen when a pose lifts or folds it
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    stacked = e["name"].startswith("ear_core")  # the ear's boxes, lit as one piece
    down = np.clip((30.5 - p[..., 1]) / 11, 0, 1) if stacked else None
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=p[..., 1] if not e["name"].startswith("leg_B") else None,
                            occ=occ, quad=bool(e["quad"]), seed=sum(map(ord, e["name"] + face)) % 97, down=down,
                            tuft=e["name"] not in ("skull", "muzzle"))
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"] == "ear":
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "skull" and face in ("east", "west"):
        pat = EYES[EYE_STYLE]["pattern"]
        u0, v0 = eye_corner_side(1 if face == "east" else -1)
        design = EYE_STYLE if face == "east" else EYE_STYLE + "_m"  # the catchlight toward the front on both
        EYES.setdefault(EYE_STYLE + "_m", dict(EYES[EYE_STYLE], pattern=[row[::-1] for row in pat]))
        stamp_eyes(img, [(u0 + len(pat[0]) / 2, v0 + len(pat) / 2)], design)
    if e["name"] == "muzzle" and face == "south" and w >= 4:  # the split lip: a line down from the nose, a hint of the incisors (2 texels at 32: none)
        c = w // 2
        img[2:h - 1, c - 1:c + 1, :3] = (img[2:h - 1, c - 1:c + 1, :3] * 0.78).astype(np.uint8)
        img[h - 1, c - 1:c + 1, :3] = BASE[TEETH]
    if e["name"].startswith("eyelid"):
        col_idx = 0 if e["name"].endswith("L") else w - 1
        img[:, col_idx, :3] = LASH
    if e["name"].endswith("_paw") and face == "south" and w >= 4 and h >= 2:  # toes: four short claws (a paw's front is 1-2 by 1 texels at 32: none)
        step = w / 4
        for k in range(1, 4):
            c = int(round(step * k)) - 1
            img[:, c, :3] = (img[:, c, :3] * 0.78).astype(np.uint8)
        img[h - 1, :, :3] = BASE[CLAW]
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

# The crouch every pose stands in (fitted: `fit_poses`): thigh forward, shin back, foot flat.
L = {"hip": -55, "knee": 112, "arch": 14, "lie_arch": 5, "raise": 20, "lift": 0.0, "lie_drop": 3.0, "lie_hip": -70, "lie_knee": 140, "lie_front": 70, "graze_neck": 30,
     "graze_head": 20}


def crouch(P=L, hip=None, knee=None, arch=None):
    """A hare's stance: the back arched up to the high rump (the body tipped forward, the head and
    front legs turned back level and upright), the hind legs folded under it, the foot's pitch
    cancelling its parents' (it stays flat), the whole raised to stand on the ground (`lift`)."""
    hip = P["hip"] if hip is None else hip
    knee = P["knee"] if knee is None else knee
    a = P["arch"] if arch is None else arch  # the head held up on a raised neck, turned back level
    q = {"root": (0, P.get("lift", 0.0), 0), "spine_0": (a, 0, 0), "neck": (-a - P["raise"], 0, 0), "head": (P["raise"], 0, 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (-a, 0, 0)
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (hip - a, 0, 0)
        q[f"leg_{leg}_lower"] = (knee, 0, 0)
        q[f"leg_{leg}_paw"] = (-hip - knee, 0, 0)
    return q


def add(q, more):
    for k, v in more.items():
        a = q.get(k, (0, 0, 0))
        q[k] = tuple(a[i] + v[i] for i in range(3))
    return q


def chew(t, rate=4.0, amp=7.0):
    return {"jaw": (amp * max(0.0, S(TAU * t * rate)), 0, 0)}


def nose(t, rate=5.0):
    """The twitching nose: quick small nods."""
    return {"nose": (6 * max(0.0, S(TAU * t * rate)), 0, 0)}


def pose_idle(t):
    """Crouched, the nose twitching, ears turning one at a time; then down to graze, chewing, and up
    to look about (6 s)."""
    p = t / 6.0
    br = S(TAU * p * 3)
    graze = float(np.interp(p, [0, .35, .42, .78, .86, 1], [0, 0, 1, 1, 0, 0]))
    turnL = float(np.interp(p, [0, .1, .14, .26, .3, 1], [0, 0, 1, 1, 0, 0]))
    turnR = float(np.interp(p, [0, .86, .9, .96, 1], [0, 0, 1, 1, 0]))
    q = crouch()
    add(q, {"root": (0, 0.1 * br, 0), "spine_1": (0.5 * br, 0, 0), "neck": (L["graze_neck"] * graze, 0, 0),
            "head": (L["graze_head"] * graze, 8 * S(TAU * p * 2) * (1 - graze), 0),
            "ear_L": (-10 * graze, -30 * turnL, 0), "ear_R": (-10 * graze, 25 * turnR, 0)})
    q.update(chew(t, 4.0) if graze > 0.5 else nose(t))
    return q


def gait(p, phases, swing_amp, lift, front=(18, 30)):
    """The legs in a hop: a foot swings forward while it's lifted, back while it's down; the hind
    legs on top of the crouch, opening the knee as they push."""
    out = crouch()
    for leg, ph in zip(LEGS, phases):
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))
        if leg[0] == "F":
            k = front[1] * up
            add(out, {f"leg_{leg}_upper": (-front[0] * S(a), 0, 0), f"leg_{leg}_lower": (k, 0, 0), f"leg_{leg}_paw": (-0.6 * k, 0, 0)})
        else:
            sw = -swing_amp * S(a)
            k = lift * up - 0.5 * lift * max(0.0, -S(a + math.pi / 2))  # folds lifted, opens pushing off
            add(out, {f"leg_{leg}_upper": (sw, 0, 0), f"leg_{leg}_lower": (k, 0, 0), f"leg_{leg}_paw": (-sw - (1 - 0.4 * up) * k, 0, 0)})  # flat down, trailing up
    return out


def pose_walk(t):
    """Lolloping: the front paws step, then the hind feet come up together (0.8 s)."""
    p = t / 0.8
    q = gait(p, (0.0, 0.12, 0.5, 0.5), 17, 26)
    c = S(TAU * p)
    add(q, {"root": (0, 0.6 * max(0.0, S(TAU * p - 0.6)), 0), "spine_0": (-3 * c, 0, 0), "spine_1": (4 * c, 0, 0),
            "head": (-3 * c, 0, 0), "ear_L": (-6, 0, 0), "ear_R": (-6, 0, 0)})
    q.update(nose(t, 2.5))
    return q


def pose_run(t):
    """Bounding flat out, the ears laid back (0.36 s: up to 70 km/h)."""
    p = t / 0.36
    q = gait(p, (0.0, 0.1, 0.5, 0.55), 33, 41, front=(36, 50))
    c = S(TAU * p)
    add(q, {"root": (0, 1.6 * max(0.0, S(TAU * p + 0.5)), 0), "spine_0": (-9 * c, 0, 0), "spine_1": (12 * c, 0, 0),
            "neck": (-6 * c - 8, 0, 0), "head": (6, 0, 0), "ear_L": (-55, 0, 6), "ear_R": (-55, 0, -6),
            "tail": (-10, 0, 0)})
    return q


def pose_lie(t, P=L):
    """In its form: pressed low, the ears laid flat along the back, still but for breathing (5 s)."""
    p = t / 5.0
    br = S(TAU * p * 2)
    q = crouch(P, P["lie_hip"], P["lie_knee"], P["lie_arch"])
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (P["lie_front"] - P["lie_arch"], 0, 0)
        q[f"leg_{leg}_lower"] = (-70, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_front"] + 60, 0, 0)  # tipped a little: never flat with the hind feet
    add(q, {"root": (0, -P["lie_drop"] + 0.1 * br, 0), "spine_1": (0.5 * br, 0, 0), "neck": (-8, 0, 0), "head": (8, 0, 0),
            "ear_L": (-62, 0, 4), "ear_R": (-62, 0, -4)})
    q.update(nose(t, 1.0))
    return q


BLINKS = {"idle": [1.4, 4.6], "lie": [1.2, 3.4]}
BLINK = 0.1
ANIMS = {"idle": (pose_idle, 6.0), "walk": (pose_walk, 0.8), "run": (pose_run, 0.36), "lie": (pose_lie, 5.0)}


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}  # side eyes: the lids swing down about z


def frame_times(name, dur):
    n = {"idle": 96, "lie": 60, "walk": 24, "run": 20}.get(name, 16)  # chewing, the nose and short hops need fine keys
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
          "front legs": lambda n: n.startswith("leg_F") and not n.endswith("_paw"),
          "rump": lambda n: n in ("hips", "rump"), "chest": lambda n: n == "chest", "muzzle": lambda n: n in ("muzzle", "jaw"),
          "body": lambda n: n in ("hips", "rump", "chest", "neck")}


def fit_poses(spec):
    """The crouch (hind feet flat on the ground, the heel a little behind the hip), the form (low,
    belly near the ground) and grazing (the mouth at the grass), by forward kinematics."""
    from ground import world
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}

    def crouch_cost(P):
        q = crouch(dict(L, **P))
        g = lowest(spec, q, GROUPS)
        W = world(spec["bones"], q)
        R, t = W["leg_BL_paw"]
        heel = R @ (np.array([BACK_X, ANKLE_Y, BACK_Z]) - piv["leg_BL_paw"]) + t
        knee = W["leg_BL_lower"][1]
        # Feet on the ground with the front paws, the heel 1-2 behind the hip, the knee clear of the ground.
        return (g["back paws"] - g["front paws"]) ** 2 * 4 + (heel[2] - (BACK_Z - 1.5)) ** 2 + 4 * max(0.0, 5 - knee[1]) ** 2
    L["lift"] = 0.0
    best = fit({"hip": list(range(-75, -29, 3)), "knee": list(range(70, 151, 3))}, crouch_cost)[1]
    L.update(best)
    L["lift"] = round(-float(lowest(spec, crouch(), GROUPS)["front paws"]), 2)

    def lie_cost(P):
        q = pose_lie(0.0, dict(L, **P, lie_drop=0))
        g = lowest(spec, q, GROUPS)
        f = min(g["back paws"], g["front paws"], g["body"])
        # The belly just off the ground (its faces never in the feet's plane), the feet on it.
        return (g["body"] - f - 0.6) ** 2 + 0.5 * (g["back paws"] - f) ** 2 + 0.5 * (g["front paws"] - f) ** 2
    grid = {"lie_hip": list(range(-90, -44, 5)), "lie_knee": list(range(100, 166, 5)), "lie_front": list(range(40, 101, 10))}
    best = fit(grid, lie_cost)[1]
    q = pose_lie(0.0, dict(L, **best, lie_drop=0))
    g = lowest(spec, q, GROUPS)
    L.update(best, lie_drop=round(float(min(g["back paws"], g["front paws"], g["body"])), 2))

    def graze_cost(P):
        q = add(crouch(), {"neck": (P["graze_neck"], 0, 0), "head": (P["graze_head"], 0, 0)})
        g = lowest(spec, q, GROUPS)
        return (g["muzzle"] - 0.5) ** 2 + 0.002 * (4 * P["graze_neck"] ** 2 + P["graze_head"] ** 2)  # mostly the head: a bent neck cuts through the chest
    best = fit({"graze_neck": list(range(10, 71, 5)), "graze_head": list(range(0, 61, 5))}, graze_cost)[1]
    L.update(best)


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
    sheet, uv = pack(tiles, 512)  # the layers pack at the same width (`Recorder.save`)
    LAYERS["rec"].save(OUT, [k for k, _ in tiles], pack)
    LAYERS["rec"] = None
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "hare", "texture": [sheet.shape[1], sheet.shape[0]],
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
