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
from ground import grounded, lowest  # noqa: E402
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
# The front legs 2.85 out (2.5 before): the forearms two texels across (rule 7, Ronald 11-05) then end 0.35
# past the chest's side, not in its plane (boxcheck), and the paws keep 1.0 apart.
FRONT_Z, BACK_Z, FRONT_X, BACK_X = 2.2, -10.5, 2.85, 4.0
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
         el("chest", "spine_1", (-4.5, 6.0, -8), (4.5, 16.9, 6.6)),  # bottom 6.0 (was 5.5): with the front paws a texel tall the forearm still shows two texels (linecheck)
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
            # Its outside 0.4 in from the chest's (boxcheck), now the leg sits 2.85 out; inside the forearm below.
            xa, xb = sorted((x - 1.5 * s, x + 1.25 * s))
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (xa, 5.0, z - 1.6), (xb, 12, z + 1.6)))
            # Rule 7 (Ronald, 11-05: no boxes a texel thick): the forearm two texels square (it was 2 by 2.2 units, a
            # texel across), up into the chest (to 7.0, its face three texels tall) so it shows two of them above the
            # paw (no thin line) and hides the upper leg's stub; the paw 0.35 out from it round the sides and back
            # and 0.5 in front (flush: it was 1.5 out in front, 0.4-0.6 elsewhere), its top 1.45 (1.6 before: the
            # forearm's side showed 1.75 texels above it).
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2, 1.2, z - 2), (x + 2, 7.0, z + 2)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 2.35, 0, z - 2.35), (x + 2.35, 2.0, z + 2.5)))  # a texel tall (Planner, 11-05: at 1.45, 0.72 texel, linecheck listed it thin)
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


# ------------------------------------------------------------------ behaviour clips (`anim.rs`)
# Each a whole-body pose over what plays below on the bones it moves (`to_bedrock.as_action`): held
# states and the approach loop, the rest play once. Too small to stand its ground: no rear or charge.
# Every clip stands in the crouch (its hind feet flat) or sits up from it (`sit`: the body tipped back
# about the hips, the hind legs kept as they stand, the front paws off the ground); the clips that
# sit up, lift a foot or walk have their height set from the feet that stay down in `build`
# (`PLANTED`).


def still(t, period=4.0, sit=0.0):
    """Crouched and breathing (`sit`: tipped back that many degrees onto the haunches, the head
    and front legs kept upright): what a clip builds on."""
    br = S(TAU * t / period * 2)
    return add(crouch(arch=L["arch"] - sit), {"root": (0, 0.1 * br, 0), "spine_1": (0.5 * br, 0, 0)})


def ears(up=1.0, back=0.0, turn=0.0):
    """The ears raised straight up by `up` (0..1: laid back and splayed as they rest, to upright),
    laid back by `back` degrees, and turned (`turn` degrees, both one way)."""
    return {"ear_L": (16 * up - back, turn, 6 * up), "ear_R": (16 * up - back, turn, -6 * up)}  # (un-splayed further, a turned ear's side lay in the skull's: flicker)


def head_low(g, t=0.0, period=4.0):
    """The nose to the ground by `g` (1: the mouth half a unit over it, fitted): the neck and head
    bowed, the legs unbent (no bend is needed)."""
    q = still(t, period)
    return add(q, {"neck": (L["crop_neck"] * g, 0, 0), "head": (L["crop_head"] * g, 0, 0)})


def pose_stare(t):
    """Half aware: frozen, the head raised, both ears straight up, only the breath moving (2 s, held)."""
    q = still(t, 2.0)
    add(q, {"neck": (-8, 0, 0), "head": (8, 0, 0)})
    q.update(ears(1.0))
    return q


def pose_watch(t):
    """A sentinel: sat up tall on its haunches, the forepaws hanging, the ears straight up, the head
    turning slowly one way and the other, the nose working (4 s, held)."""
    p = t / 4.0
    yaw = 30 * S(TAU * p)
    q = still(t, 4.0, sit=L["sit"])
    add(q, {"neck": (-10, 0.6 * yaw, 0), "head": (10, 0.4 * yaw, 0)})
    for leg in ("FL", "FR"):
        add(q, {f"leg_{leg}_lower": (18, 0, 0), f"leg_{leg}_paw": (-10, 0, 0)})
    q.update(ears(1.0, turn=-0.4 * yaw))
    q.update(nose(t, 3.0))
    return q


def pose_graze(t):
    """Grazing: the mouth at the grass, cropping and chewing, the head working side to side, the
    ears easy (2.5 s, held)."""
    p = t / 2.5
    q = head_low(1.0, t, 2.5)
    q["neck"] = (q["neck"][0], 5 * S(TAU * p), 0)
    q.update(chew(t, 4.0, 6.0))
    q.update(ears(0.0, back=10, turn=6 * S(TAU * p * 2)))
    return q


def pose_drink(t):
    """Drinking: the mouth at the water, lapping, the head dipping (only up from the fitted low) (1.5 s, held)."""
    q = head_low(1.0, t, 1.5)
    q["head"] = (q["head"][0] - 1.5 + 1.5 * S(TAU * t / 1.5 * 2), 0, 0)
    q.update(chew(t, 3.0, 3.0))
    q.update(ears(0.0, back=10))
    return q


def pose_sniff(t):
    """The nose down to the ground, twitching fast as it sweeps side to side, and up (2.2 s, once)."""
    p = t / 2.2
    d = float(np.interp(p, [0, .25, .75, 1], [0, 1, 1, 0]))
    q = head_low(d, t, 2.2)
    q["head"] = (q["head"][0], 10 * d * S(TAU * p * 2), 0)
    q.update(ears(0.6 * d))
    q.update(nose(t, 7.0))
    return q


def pose_look(t):
    """Head up and round one way, then the other, the ears following (3 s, once)."""
    p = t / 3.0
    yaw = float(np.interp(p, [0, .15, .4, .55, .8, 1], [0, 40, 40, -35, -35, 0]))
    up = float(np.interp(p, [0, .12, .88, 1], [0, 1, 1, 0]))
    q = still(t, 3.0)
    add(q, {"neck": (-8 * up, 0.7 * yaw, 0), "head": (8 * up, 0.3 * yaw, 0)})
    q.update(ears(up, turn=0.3 * yaw))
    q.update(nose(t))
    return q


def pose_scratch(t):
    """An itch: sat back on its haunches, the head bowed and tipped to the left, the left hind foot
    brought up to the ear's root and scratching fast, the eyes half shut (2 s, once). Fitted: the
    toe at the ear."""
    p = t / 2.0
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    k = d * S(TAU * t * 4)  # four strokes a second
    # On the way the thigh leads and the knee and foot follow, the foot lifted flat as in the thump
    # (all turned together, the foot dug up to 1.6 into the ground).
    e = 3 * math.sin(math.pi * d)
    q = still(t, 2.0, sit=L["scr_sit"] * d)
    add(q, {"neck": (L["scr_neck"] * d, L["scr_yaw"] * d, 0), "head": (0, 0, L["scr_roll"] * d),
            "leg_BL_upper": (L["scr_hip"] * d ** 0.7 - L["thump_hip"] * e - 6 * k, 0, L["scr_out"] * d),
            "leg_BL_lower": (L["scr_knee"] * d * d + L["thump_knee"] * e + 8 * k, 0, 0),
            "leg_BL_paw": (L["scr_foot"] * d * d + (L["thump_hip"] - L["thump_knee"]) * e, 0, 0)})
    q.update(ears(0.0, back=-10 * d))
    q["ear_L"] = (q["ear_L"][0], 0, q["ear_L"][2] + 2 * k)  # knocked by the foot
    q.update({"eyelid_L": (0, 0, 55 * d), "eyelid_R": (0, 0, -55 * d)})
    return q


def pose_groom(t):
    """Washing the face: sat back, the head bowed to the forepaws, which wipe down over the face
    one after the other, the eyes half shut (2.5 s, once). Fitted: the paws at the cheeks."""
    p = t / 2.5
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    q = still(t, 2.5, sit=L["gr_sit"] * d)
    add(q, {"neck": (L["gr_neck"] * d, 0, 0), "head": (3 * d * S(TAU * t * 2), 0, 0)})
    for leg, ph in (("FL", 0.0), ("FR", math.pi)):
        w = d * S(TAU * t * 2 + ph)  # two wipes a second, the paws in turn
        add(q, {f"leg_{leg}_upper": (L["gr_up"] * d + 10 * w, 0, 0), f"leg_{leg}_lower": (L["gr_low"] * d - 6 * w, 0, 0),
                f"leg_{leg}_paw": (L["gr_paw"] * d, 0, 0)})
    q.update(ears(0.3 * d, back=10 * d))
    q.update({"eyelid_L": (0, 0, 50 * d), "eyelid_R": (0, 0, -50 * d)})
    return q


def pose_stretch(t):
    """Getting up: a bow, the forepaws walked out in front and the shoulders low, the head up in a
    yawn, the ears laid back; then the left hind leg pushed out straight behind (2.4 s, once).
    Fitted: the forepaws on the ground, the hind leg as far back as it goes just off it."""
    p = t / 2.4
    a = float(np.interp(p, [0, .2, .5, .6, 1], [0, 1, 1, 0, 0]))
    b = float(np.interp(p, [0, .55, .7, .9, 1], [0, 0, 1, 1, 0]))
    q = still(t, 2.4)
    reach, low = L["st_reach"] * a, L["st_chest"] * a
    add(q, {"spine_1": (low, 0, 0), "neck": (-low + 6 * a, 0, 0), "head": (-22 * a, 0, 0)})
    for leg in ("FL", "FR"):
        add(q, {f"leg_{leg}_upper": (-reach, 0, 0), f"leg_{leg}_paw": (reach - low, 0, 0)})  # flat: the turns sum to nothing
    e = 3 * math.sin(math.pi * b)  # on the way, the foot lifted flat as in the thump (straight there, it dug into the ground)
    add(q, {"leg_BL_upper": (L["st_hip"] * b - L["thump_hip"] * e, 0, 0), "leg_BL_lower": (L["st_knee"] * b + L["thump_knee"] * e, 0, 0),
            "leg_BL_paw": (L["st_foot"] * b + (L["thump_hip"] - L["thump_knee"]) * e, 0, 0)})
    q.update(chew(t, 0.42, 22 * a))  # the yawn
    q.update(ears(0.0, back=30 * a))
    return q


def pose_sleep(t, P=L):
    """Asleep in its form: the lie, the ears laid flat along the back, the eyes shut, the nose still, a slow breath
    (6 s, held)."""
    q = pose_lie(0.0, P)
    br = S(TAU * t / 6.0 * 2)
    q.update({"root": (0, q["root"][1] + 0.08 * br, 0), "spine_1": (0.4 * br, 0, 0), "neck": (-6, 0, 0), "head": (8, 0, 0),
              "ear_L": (-62, 0, 4), "ear_R": (-62, 0, -4), "nose": (0, 0, 0)})  # the lie's (pressed closer, their panels met: flicker)
    q.update({"eyelid_L": (0, 0, 90), "eyelid_R": (0, 0, -90)})
    return q


def pose_alarm(t):
    """Alarm: the head up, ears straight up, the hind feet lifted and thumped down twice, the tail
    flicked up to flash its white underside (1.2 s, once)."""
    p = t / 1.2
    up = float(np.interp(p, [0, .1, .8, 1], [0, 1, 1, 0]))
    th = float(np.interp(p, [0, 4 / 24, 7 / 24, 8 / 24, 11 / 24, 14 / 24, 15 / 24, 1], [0, 0, 1, 0, 0, 1, 0, 0]))
    q = still(t, 1.2)
    # The tail flicked up in a frame (on its way, from 20 to 40 degrees, the scut's corner pokes through the rump).
    flick = -50.0 if 1 <= round(p * 24) <= 20 else 0.0
    add(q, {"neck": (-10 * up, 0, 0), "head": (10 * up, 0, 0), "tail": (flick, 0, 0)})
    for leg in ("BL", "BR"):  # the foot lifted flat: the turns sum to nothing
        add(q, {f"leg_{leg}_upper": (-L["thump_hip"] * th, 0, 0), f"leg_{leg}_lower": (L["thump_knee"] * th, 0, 0),
                f"leg_{leg}_paw": ((L["thump_hip"] - L["thump_knee"]) * th, 0, 0)})
    q.update(ears(up))
    q.update(nose(t, 6.0))
    return q


def pose_play(t):
    """A leveret's binky: a leap straight up, twisting in the air, the hind legs kicked out to the
    side and the head tossed, and down again (1.0 s, once). Its height from its feet in `build`."""
    p = t / 1.0
    h = math.sin(math.pi * float(np.clip((p - 0.1) / 0.7, 0, 1)))
    tw = S(TAU * float(np.clip((p - 0.15) / 0.6, 0, 1))) * h  # one twist each way while up
    q = still(t, 1.0)
    add(q, {"root": (0, 10 * h, 0), "spine_0": (0, 22 * tw, 18 * tw), "spine_1": (0, -14 * tw, -8 * tw),
            "neck": (-8 * h, -16 * tw, 0), "head": (6 * h, -14 * tw, 10 * tw), "tail": (-10 * h, 0, 0)})  # (raised 20-40 the scut's corner pokes through the rump)
    for leg, s in (("BL", 1), ("BR", -1)):  # kicked out, opened
        add(q, {f"leg_{leg}_upper": (30 * h, 0, s * 22 * h * h), f"leg_{leg}_lower": (-45 * h, 0, 0), f"leg_{leg}_paw": (20 * h, 0, 0)})
    for leg in ("FL", "FR"):  # tucked
        add(q, {f"leg_{leg}_upper": (-25 * h, 0, 0), f"leg_{leg}_lower": (40 * h, 0, 0), f"leg_{leg}_paw": (-15 * h, 0, 0)})
    q.update(ears(0.5 * h, back=20 * h))
    q["ear_L"] = (q["ear_L"][0], 0, q["ear_L"][2] + 14 * tw)
    q["ear_R"] = (q["ear_R"][0], 0, q["ear_R"][2] + 14 * tw)
    return q


def pose_nuzzle(t):
    """Nuzzling a leveret: the head low and forward, a gentle push, a lick (2 s, once)."""
    p = t / 2.0
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    push = max(0.0, S(TAU * p * 2))
    q = head_low(0.55 * d, t, 2.0)
    q["head"] = (q["head"][0] - 8 * push * d, 0, 0)
    q.update(chew(t, 2.0, 4.0 * d))
    q.update(ears(0.5 * d))
    return q


def pose_flinch(t):
    """Hit: a jerk aside, the head ducked, the ears flattened, the tail clamped, the eyes squeezed
    (0.45 s, once)."""
    p = t / 0.45
    d = float(np.interp(p, [0, .15, .45, 1], [0, 1, 0.8, 0]))
    q = still(t, 0.45)
    add(q, {"spine_1": (0, 0, 3 * d), "neck": (12 * d, 0, 0), "head": (8 * d, 0, 0), "tail": (14 * d, 0, 0),
            "leg_FL_upper": (0, 0, -3 * d), "leg_FR_upper": (0, 0, -3 * d)})
    q.update(ears(0.0, back=45 * d))
    q.update({"eyelid_L": (0, 0, 70 * d), "eyelid_R": (0, 0, -70 * d)})
    return q


def pose_approach(t):
    """Curious, coming to look: slow careful hops, the neck stretched out low and the nose forward
    and twitching, the ears up and cupped ahead (1.2 s a cycle, in place of the walk). Its height from
    its feet in `build`."""
    p = t / 1.2
    q = gait(p, (0.0, 0.12, 0.5, 0.5), 10, 16, front=(10, 20))
    c = S(TAU * p)
    add(q, {"spine_1": (2 * c, 0, 0), "neck": (16, 0, 0), "head": (-6 - 2 * c, 0, 0)})
    q.update(ears(0.8, back=-8))
    q.update(nose(t, 5.0))
    return q


def pose_death(t):
    """Dying: the legs give, it rolls onto its side, the legs go slack and straighten, the head and
    ears drop, the eyes shut; it stays (1.4 s, holds). Each frame rests on its lowest point (`build`)."""
    p = t / 1.4
    fall = float(np.interp(p, [0, .35, .7, 1], [0, 0.35, 1, 1]))
    q = crouch()
    add(q, {"spine_0": (-L["arch"] * fall, 0, 84 * fall), "neck": (L["arch"] + 14 * fall - L["arch"] * (1 - fall), 0, -8 * fall),
            "head": (-L["raise"] * fall + 6 * fall, 0, 0), "jaw": (6 * fall, 0, 0), "tail": (10 * fall, 0, 0)})
    for leg in ("FL", "FR"):
        add(q, {f"leg_{leg}_upper": (L["arch"] * fall - 30 * fall, 0, 0), f"leg_{leg}_lower": (10 * fall, 0, 0)})
    for leg in ("BL", "BR"):  # unfolded, trailing
        add(q, {f"leg_{leg}_upper": (L["arch"] * fall + 40 * fall, 0, 0), f"leg_{leg}_lower": (-60 * fall, 0, 0), f"leg_{leg}_paw": (30 * fall, 0, 0)})
    q.update(ears(0.0, back=40 * fall))
    q.update({"eyelid_L": (0, 0, 90 * fall), "eyelid_R": (0, 0, -90 * fall)})
    return q


BLINKS = {"idle": [1.4, 4.6], "lie": [1.2, 3.4], "stare": [1.3], "graze": [1.1], "look": [1.6], "watch": [1.2, 3.1],
          "drink": [0.9]}
BLINK = 0.1
ANIMS = {"idle": (pose_idle, 6.0), "walk": (pose_walk, 0.8), "run": (pose_run, 0.36), "lie": (pose_lie, 5.0),
         "stare": (pose_stare, 2.0), "graze": (pose_graze, 2.5), "drink": (pose_drink, 1.5), "sniff": (pose_sniff, 2.2),
         "look": (pose_look, 3.0), "scratch": (pose_scratch, 2.0), "stretch": (pose_stretch, 2.4), "sleep": (pose_sleep, 6.0),
         "alarm": (pose_alarm, 1.2), "play": (pose_play, 1.0), "groom": (pose_groom, 2.5), "nuzzle": (pose_nuzzle, 2.0),
         "flinch": (pose_flinch, 0.45), "death": (pose_death, 1.4), "watch": (pose_watch, 4.0), "approach": (pose_approach, 1.2)}
# The held ones and the approach loop; a clip played once must not (the engine loops a clip whose
# file says so, whatever the game asks); the death holds its last frame.
LOOPS = {"stare": True, "graze": True, "drink": True, "sleep": True, "watch": True, "approach": True, "death": "hold_on_last_frame"}
BLENDS = {"flinch": (0.05, 0.2), "death": (0.1, 0.0), "sleep": (0.6, 0.6), "graze": (0.4, 0.4), "watch": (0.4, 0.4),
          "approach": (0.3, 0.3)}
STANDING = ("idle", "walk", "run", "lie")
PAWS = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
# The feet each clip stands on, its height set from them frame by frame (`build`): sat up, a hind foot
# raised, the shoulders lowered, walking. `None`: every part (dying, it rests on its lowest point).
PLANTED = {"watch": ["leg_BL_paw", "leg_BR_paw"], "groom": ["leg_BL_paw", "leg_BR_paw"], "scratch": ["leg_BR_paw"],
           "stretch": ["leg_FL_paw", "leg_FR_paw", "leg_BR_paw"], "approach": PAWS, "death": None}


def eyelids(name, t, pose=None):
    """The blinks, over a clip's own lids (half shut, squeezed) where it has them."""
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    own = abs((pose or {}).get("eyelid_L", (0, 0, 0))[2])
    a = max(a, own)
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}  # side eyes: the lids swing down about z


def frame_times(name, dur):
    # chewing, the nose and short hops need fine keys; the scratch and the face-wash fast strokes
    n = {"idle": 96, "lie": 60, "walk": 24, "run": 20, "sleep": 36, "flinch": 12, "watch": 40, "approach": 24, "scratch": 48,
         "groom": 50}.get(name, 16 if name in STANDING else 24)
    ts = {round(dur * k / n, 4) for k in range(n + 1)}
    for b in BLINKS.get(name, []):
        ts |= {round(b + d, 4) for d in (-BLINK, -BLINK / 2, 0, BLINK / 2, BLINK)}
    return sorted(ts)


def animations(bone_names):
    out = {}
    for name, (fn, dur) in ANIMS.items():
        frames = []
        loops = LOOPS.get(name, name in STANDING)
        for t in frame_times(name, dur):
            # A loop's last frame is its first; a clip played once ends where it ends.
            tt = t % dur if t < dur else (0.0 if loops is True else dur)
            pose = fn(tt)
            if name not in ("sleep", "death"):  # those keep their eyes shut
                pose.update(eyelids(name, tt, pose))
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames}
        if name not in STANDING:
            out[name].update(loop=loops, blend=BLENDS.get(name, (0.25, 0.3)))
    # A clip goes over what plays below only on the bones it writes, and the exporter leaves out a bone
    # that stays at 0: kept at a hundredth of a degree, so the idle's grazing, chewing, nose and ear
    # turns don't show through a clip that holds them still. The eyelids are left to the idle's blinks.
    moved = {b for a in ("idle", "lie") for f in out[a]["frames"] for b, r in f["rot"].items() if any(abs(v) > 1e-6 for v in r)}
    for name, anim in out.items():
        if name in STANDING:
            continue
        for b in moved - {"eyelid_L", "eyelid_R"}:
            if all(not any(abs(v) > 1e-6 for v in f["rot"][b]) for f in anim["frames"]):
                for f in anim["frames"]:
                    f["rot"][b] = [0.01, 0.0, 0.0]
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

    # The floor is every part's lowest, not only the feet's and the body's: fitted on those alone, the
    # folded haunches went 2.2 into the ground (hip -50, knee 135, arch 5).
    every = {"all": lambda n: True}

    def lie_cost(P):
        q = pose_lie(0.0, dict(L, **P, lie_drop=0))
        g = lowest(spec, q, GROUPS)
        f = lowest(spec, q, every)["all"]
        # The belly just off the ground (its faces never in the feet's plane), the feet on it.
        # The forearms (at lie_front - 70) never level with the belly (at lie_arch): their undersides flicker.
        flat = 1.0 if abs(P["lie_front"] - 70 - P["lie_arch"]) < 2 else 0.0
        return (g["body"] - f - 0.6) ** 2 + 0.5 * (g["back paws"] - f) ** 2 + 0.5 * (g["front paws"] - f) ** 2 + flat
    grid = {"lie_hip": list(range(-130, -29, 5)), "lie_knee": list(range(90, 181, 5)), "lie_front": list(range(40, 101, 5)),
            "lie_arch": [-5, 0, 5, 10, 15]}
    best = fit(grid, lie_cost)[1]
    q = pose_lie(0.0, dict(L, **best, lie_drop=0))
    L.update(best, lie_drop=round(float(lowest(spec, q, every)["all"]), 2))

    def graze_cost(P):
        q = add(crouch(), {"neck": (P["graze_neck"], 0, 0), "head": (P["graze_head"], 0, 0)})
        g = lowest(spec, q, GROUPS)
        return (g["muzzle"] - 0.5) ** 2 + 0.002 * (4 * P["graze_neck"] ** 2 + P["graze_head"] ** 2)  # mostly the head: a bent neck cuts through the chest
    best = fit({"graze_neck": list(range(10, 71, 5)), "graze_head": list(range(0, 61, 5))}, graze_cost)[1]
    L.update(best)
    fit_clips(spec)


def fit_clips(spec):
    """The behaviour clips' fits (the idle's own above stay as they were): the mouth at the grass
    (graze, drink, sniff, nuzzle: the idle's graze stops 2 units over it), the hind feet's thump, the
    toe at the ear (scratch), the forepaws at the cheeks (groom), the forepaws on the ground in the
    bow (stretch)."""
    from ground import world
    from scipy.optimize import minimize
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}

    def at(W, bone, p):
        R, t = W[bone]
        return R @ (np.array(p, float) - piv[bone]) + t

    def bounded(x, lim):
        return sum(max(0.0, lo - v, v - hi) ** 2 for v, (lo, hi) in zip(x, lim))

    # Cropping: the mouth half a unit over the ground by the neck and head alone (the neck at most
    # 50, past that its corner shows through the chest; the head at most 50, through the skull).
    def crop_cost(P):
        q = add(crouch(), {"neck": (P["crop_neck"], 0, 0), "head": (P["crop_head"], 0, 0)})
        return (lowest(spec, q, GROUPS)["muzzle"] - 0.5) ** 2 + 2e-5 * (P["crop_neck"] ** 2 + P["crop_head"] ** 2)
    L.update(fit({"crop_neck": list(range(30, 51, 2)), "crop_head": list(range(0, 51, 5))}, crop_cost)[1])
    L["sit"] = 50  # the sentinel: the body tipped back 50 degrees onto its haunches

    # The thump: both hind feet lifted 2.5 units, flat, by the hip and knee.
    feet = {"back paws": GROUPS["back paws"]}

    def thump_cost(P):
        q = crouch()
        for leg in ("BL", "BR"):
            add(q, {f"leg_{leg}_upper": (-P["thump_hip"], 0, 0), f"leg_{leg}_lower": (P["thump_knee"], 0, 0),
                    f"leg_{leg}_paw": (P["thump_hip"] - P["thump_knee"], 0, 0)})
        return (lowest(spec, q, feet)["back paws"] - 2.5) ** 2 + 1e-4 * (P["thump_hip"] ** 2 + P["thump_knee"] ** 2)
    L.update(fit({"thump_hip": list(range(-30, 31, 2)), "thump_knee": list(range(-30, 41, 2))}, thump_cost)[1])

    # The scratch: the left hind foot's toe at the left ear's root, the head bowed and tipped toward
    # it, the leg kept outside the chest and front legs, the head off the ground, sat back on the
    # haunches as far as needed.
    toe0 = (BACK_X + 0.8, ANKLE_Y - 0.75, BACK_Z + 8.6)
    names = ["scr_sit", "scr_neck", "scr_yaw", "scr_roll", "scr_hip", "scr_out", "scr_knee", "scr_foot"]
    # (sat back past 38 or bowed past 25, the neck's corner shows through the chest)
    lim = [(0, 38), (-10, 25), (0, 60), (-40, 0), (-150, 30), (0, 40), (-160, 160), (-120, 120)]
    parts = {"head": lambda n: n in ("muzzle", "jaw", "skull"), "BR": lambda n: n == "leg_BR_paw",
             "leg": lambda n: n in ("leg_BL_paw", "leg_BL_lower")}

    def scratch_pose(x):
        sit, neck, yaw, roll, hip, out, knee, foot = x
        return add(crouch(arch=L["arch"] - sit), {"neck": (neck, yaw, 0), "head": (0, 0, roll), "leg_BL_upper": (hip, 0, out),
                                                  "leg_BL_lower": (knee, 0, 0), "leg_BL_paw": (foot, 0, 0)})

    def scratch_cost(x):
        q = scratch_pose(x)
        W = world(spec["bones"], q)
        toe, ear = at(W, "leg_BL_paw", toe0), at(W, "ear_L", (EAR_X + 1.6, 19.6, sum(EAR_Z) / 2))
        knee = at(W, "leg_BL_lower", (BACK_X + 0.8, KNEE_Y, BACK_Z))
        ankle = at(W, "leg_BL_paw", (BACK_X + 0.8, ANKLE_Y, BACK_Z))
        g = lowest(spec, q, parts)
        pen = sum(max(0.0, 6.5 - p[0]) ** 2 for p in (knee, ankle, (knee + ankle) / 2, (ankle + toe) / 2)) + bounded(x, lim)
        pen += max(0.0, g["BR"] + 3 - g["head"]) ** 2 + max(0.0, g["BR"] + 1 - g["leg"]) ** 2
        return float(np.sum((toe - ear) ** 2) + 3 * pen + 0.0002 * np.sum(np.square(x)))
    best = min((minimize(scratch_cost, g, method="Nelder-Mead", options={"maxiter": 5000, "maxfev": 5000})
                for g in ([40, 25, 23, -26, -52, 40, -47, 20], [35, 25, 21, -40, -47, 40, -45, 19])), key=lambda r: r.fun)
    L.update({k: round(float(v), 1) for k, v in zip(names, best.x)})

    # The face-wash: the left forepaw's front at the left cheek (the right mirrors it), sat back.
    paw0, cheek0 = (FRONT_X, 1.0, FRONT_Z + 1.5), (2.6, 14.5, 16.0)
    names = ["gr_sit", "gr_neck", "gr_up", "gr_low", "gr_paw"]
    lim = [(0, 38), (-10, 30), (-150, 0), (-20, 140), (-90, 90)]  # (sat back or bowed further, the neck shows through the chest)

    def groom_pose(x):
        sit, neck, up, low, paw = x
        q = add(crouch(arch=L["arch"] - sit), {"neck": (neck, 0, 0)})
        for leg in ("FL", "FR"):
            add(q, {f"leg_{leg}_upper": (up, 0, 0), f"leg_{leg}_lower": (low, 0, 0), f"leg_{leg}_paw": (paw, 0, 0)})
        return q

    def groom_cost(x):
        q = groom_pose(x)
        W = world(spec["bones"], q)
        g = lowest(spec, q, {"head": parts["head"], "back paws": GROUPS["back paws"]})
        pen = bounded(x, lim) + max(0.0, g["back paws"] + 3 - g["head"]) ** 2
        return float(np.sum((at(W, "leg_FL_paw", paw0) - at(W, "head", cheek0)) ** 2) + 3 * pen + 0.0003 * np.sum(np.square(x)))
    best = minimize(groom_cost, [37, 30, -45, -3, 19], method="Nelder-Mead", options={"maxiter": 4000, "maxfev": 4000})
    L.update({k: round(float(v), 1) for k, v in zip(names, best.x)})

    # The bow: the forepaws reached forward 35 degrees, the shoulders lowered (the chest tipped) until
    # they are on the ground with the hind feet.
    L["st_reach"] = 35

    def bow_cost(P):
        q = crouch()
        add(q, {"spine_1": (P["st_chest"], 0, 0)})
        for leg in ("FL", "FR"):
            add(q, {f"leg_{leg}_upper": (-L["st_reach"], 0, 0), f"leg_{leg}_paw": (L["st_reach"] - P["st_chest"], 0, 0)})
        g = lowest(spec, q, GROUPS)
        return (g["front paws"] - g["back paws"]) ** 2
    L.update(fit({"st_chest": [x / 2 for x in range(0, 61)]}, bow_cost)[1])
    # The hind leg's push: the ankle as far back as it goes with the leg clear of the ground (0.8 to
    # 3 over it), the foot near in line with the shin.
    leg = {"paw": lambda n: n == "leg_BL_paw", "shin": lambda n: n == "leg_BL_lower"}

    def push_cost(P):
        q = add(crouch(), {"leg_BL_upper": (P["st_hip"], 0, 0), "leg_BL_lower": (P["st_knee"], 0, 0), "leg_BL_paw": (P["st_foot"], 0, 0)})
        g = lowest(spec, q, leg)
        ankle = at(world(spec["bones"], q), "leg_BL_paw", (BACK_X, ANKLE_Y, BACK_Z))
        return ankle[2] + 0.0005 * (P["st_hip"] + P["st_knee"] + P["st_foot"]) ** 2 + 100 * (max(0.0, 0.8 - min(g.values())) + max(0.0, g["paw"] - 3))
    L.update(fit({"st_hip": list(range(0, 101, 10)), "st_knee": list(range(-120, 1, 10)), "st_foot": list(range(-60, 91, 10))}, push_cost)[1])
    print("clips fitted:", {k: v for k, v in L.items() if k.startswith(("crop", "thump", "scr", "gr", "st_", "sit"))})


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
    # The clips that sit up, lift a foot or walk stand on the feet that stay down, frame by frame;
    # dying, each frame rests on its lowest point (mid-roll its legs swing out below where it ends
    # up lying); the binky's leap shifted once, so it takes off from and lands on the ground.
    everything = [e["name"] for e in E]
    for clip, feet in PLANTED.items():
        spec["animations"][clip] = grounded(spec, spec["animations"][clip], feet or everything)
    spec["animations"]["play"] = grounded(spec, spec["animations"]["play"], PAWS, True)
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
