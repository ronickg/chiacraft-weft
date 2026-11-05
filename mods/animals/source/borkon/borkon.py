#!/usr/bin/env python3
"""Borkon, the Height-Keeper: the mountains' boss (`mods/animals/src/borkon.rs`), made with the
blocky-model skill on Drenkaun's build (`../drenkaun/drenkaun.py`, itself the reindeer's and the
deer's): laid out in the deer's units, drawn `K` times bigger (about 3.3 m at the shoulder), an
ibex titan with great ridged horns sweeping back over its neck, a shaggy pale ruff over its neck,
chest and shoulders, a goat's beard and snow lying on its back. Its colors are concept A's
(Ronald, 11-05: "A: ibex"; `README.md`); its clips are the fight's (`borkon.rs` plays them by name).

    ~/.venvs/blocky/bin/python borkon.py     # writes out/spec.json, out/texture.png

Units are 1/64 of a block, y up, it faces +z; painted at the house density, 32 texels a block.
It has no coat genes (a boss, one of its kind), so no coat layers.
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
from paint import EYES, eye_corner, occlusion, pack, paint_face_smooth, stamp_eyes, tones  # noqa: E402
from rig import fit  # noqa: E402
from ground import lowest  # noqa: E402
from house import TX  # noqa: E402

OUT = os.environ.get("BORKON_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin
K = 3.5  # the deer's layout drawn this much bigger: the withers 3.3 m up, its horns' arc 5 m up

# ------------------------------------------------------------------ materials and eyes

FUR, BACK, BELLY, DARK, EARIN, MOUTH, HORN, HOOF, TAILTOP, LEG, PALE, HORNLT, RUFF, RUFFDK, HORNDK, SNOW, LICHEN, MUZZLE, NOSE = range(19)
# From concept A (sampled): a taupe coat (FUR), darker along the back (BACK), cream under the belly
# (BELLY), a shaggy cream ruff over the neck, chest and shoulders (RUFF, its locks' tips RUFFDK),
# grey-tan face and pale muzzle (MUZZLE, PALE), dark brown shanks (LEG) and hooves; horns of
# weathered horn (HORN) to pale ridges (HORNLT), dark grooves (HORNDK) and yellow lichen (LICHEN);
# snow lying along its back (SNOW).
BASE = [(162, 130, 102), (126, 100, 78), (222, 198, 156), (48, 36, 28), (184, 142, 120), (84, 46, 44), (184, 152, 114),
        (60, 46, 36), (112, 86, 64), (104, 80, 60), (214, 192, 152), (202, 174, 136), (206, 182, 142), (164, 138, 104),
        (128, 100, 72), (236, 232, 222), (156, 150, 64), (184, 158, 126), (70, 54, 44)]
SHADOW = [0.03, 0.03, 0.06, 0.95, 0.97, 0.95, 0.05, 0.74, 0.03, 0.02, 0.06, 0.06, 0.06, 0.04, 0.04, 0.62, 0.12, 0.04, 0.03]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (FUR, BACK, BELLY, LEG, PALE, RUFF, RUFFDK)

# Its eyes are the skill's designed ones (`paint.EYES`), as Drenkaun's: the amber design Ronald
# picked for it drawn round at 12 by 10 ("A amber", "eyes a bit bigger"), stamped on the head's
# sides with the design's ring round them, mirrored on the west side so both catchlights look
# forward, and lids that blink. `ibex_amber` is the same with a goat's long pupil, across the eye.
EYES["warden_amber"] = {**EYES["hytale_amber"], "pattern": [
    "...LLLLLL...",
    ".LLAAAAAALL.",
    ".LAAAAAAAAL.",
    "LAAHHHPPPAAL",
    "LAAHHPPPPAAL",
    "LAAPPPPPPAAL",
    "LAAPPPPPPAAL",
    ".LAaPPPPaAL.",
    ".LLaaAAaaLL.",
    "...LLLLLL...",
]}
EYES["ibex_amber"] = {**EYES["hytale_amber"], "pattern": [
    "...LLLLLL...",
    ".LLAAAAAALL.",
    ".LAHHAAAAAL.",
    "LAAHHAAAAAAL",
    "LAPPPPPPPPAL",
    "LAPPPPPPPPAL",
    "LAAaaaaaaAAL",
    ".LAaaaaaaAL.",
    ".LLaaAAaaLL.",
    "...LLLLLL...",
]}
EYE_TYPES = ("warden_amber", "ibex_amber")
for _t in EYE_TYPES:
    assert all(len(r) == 12 for r in EYES[_t]["pattern"]), _t
    EYES[_t + "_m"] = {**EYES[_t], "pattern": [row[::-1] for row in EYES[_t]["pattern"]]}
EYE_STYLE = os.environ.get("BORKON_EYES", "warden_amber")

# In the deer's space (scaled by K when the model is built). The head is drawn `HS` times its
# reindeer size about its pivot, as Drenkaun's.
HEAD = np.array([0.0, 66.0, 38.0])
# The ruff (the skill's rule 6, as Drenkaun's moss): one shell a little bigger than the shoulders'
# front half, open behind, its hem and its back edge cut out in shaggy locks, so the back behind it
# shows smooth with snow on it; and a lower, shorter one over the hips, the coat shaggy down to the
# haunches (concept A).
SHELL = (-13.3, 40.0, 8.0), (13.3, 66.4, 30.6)  # 2 units out from the shoulders' front half, open behind
SHELL_HIPS = (-11.1, 47.0, -22.5), (11.1, 55.6, -8.0)  # 2 units out from the hips, short of the thighs
HS = 1.22


def hs(p):
    """A point of the head, drawn `HS` times bigger about the head's pivot."""
    return tuple(HEAD + (np.array(p, float) - HEAD) * HS)


SKULL0 = (hs((-7.0, 60.0, 32.0)), hs((7.0, 72.0, 46.0)))
EYE_Z0, EYE_Y0 = hs((0, 67.6, 40.5))[2], hs((0, 67.6, 40.5))[1]
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
NECK_PITCH, NECK_LEN, NECK_W = -45.0, 30.0, 13.0  # a thick neck, carried raised (`HEAD_UP`)
NECK0 = HEAD - (NECK_LEN - 2.5) * np.array([0, math.sin(math.radians(-NECK_PITCH)), math.cos(math.radians(-NECK_PITCH))])
NECK_ROT = (NECK_PITCH + 90, 0, 0)
# A goat's ears: narrow, pointed, held out to the sides under the horns.
EAR = [(70.5, 72.5, 1.0), (72.5, 76.5, 2.1), (76.5, 79.5, 1.0)]  # y0, y1, half width
EAR_X, EAR_Z = 5.4, (33.4, 35.0)
TAIL0 = np.array([0.0, 53.5, -40.0])  # on the rump's back face: a flap hinged on the surface
HORN0 = hs((3.4, 70.6, 38.6))  # where the left horn grows from the skull's top
# The horn's lengths, from its root: pitch from upright (back is negative), how far it leans out,
# length, and width across (side to side; front to back it is 1.25 times this). A scimitar arc up
# and back over the neck, its tip turning down behind the head, as concept A's (an ibex's horns
# run about as long as its back).
HORN_ARC = [(-6, 0.08, 12.0, 6.8), (-30, 0.13, 12.5, 6.2), (-54, 0.18, 12.0, 5.6), (-78, 0.22, 11.5, 5.0),
            (-102, 0.24, 11.0, 4.4), (-126, 0.24, 10.0, 3.8), (-150, 0.22, 9.0, 3.2), (-172, 0.2, 7.5, 2.6)]


def euler_zyx(R):
    """Euler angles (degrees, x y z) of a rotation matrix, for R = Rz Ry Rx (`rotm`)."""
    ry = math.asin(max(-1.0, min(1.0, -R[2, 0])))
    rx = math.atan2(R[2, 1], R[2, 2])
    rz = math.atan2(R[1, 0], R[0, 0])
    return tuple(round(math.degrees(a), 3) for a in (rx, ry, rz))


def horn_rod(name, bone, start, pitch, lean, length, w, back=0.6):
    """A length of horn from `start`: along its pitch (0 upright, -90 straight back), leaning out
    by `lean` (x per unit along), its front face (the box's south, where the ridges are) kept on
    the arc's outer side; `w` across, 1.25 `w` deep. And its far end."""
    a = math.radians(pitch)
    d = np.array([lean, math.cos(a), math.sin(a)])
    d /= np.linalg.norm(d)
    n = np.array([0.0, -math.sin(a), math.cos(a)])  # the arc's outer side
    n -= d * (n @ d)
    n /= np.linalg.norm(n)
    x = np.cross(d, n)
    rot = euler_zyx(np.column_stack([x, d, n]))
    s = np.array(start, float)
    wz = 1.25 * w
    e = el(name, bone, (s[0] - w / 2, s[1] - back, s[2] - wz / 2), (s[0] + w / 2, s[1] + length, s[2] + wz / 2), rot=rot, origin=s)
    return e, s + rotm(rot) @ np.array([0, length - 0.6, 0])


def horn(side):
    """Borkon's horn: seven lengths of ridged horn from a root in the skull's top, up and back in
    a long arc over its neck, the tip turning down, each length narrower than the one before.
    Each horn is its own bone, so a break can take it (`bosses::Part` left_horn, right_horn)."""
    s = 1 if side == "L" else -1
    k = side.lower()
    bone = f"horn_{side}"
    x0, y0, z0 = HORN0
    at = np.array([s * x0, y0, z0])
    E = []
    for j, (pitch, lean, length, w) in enumerate(HORN_ARC):
        e, at = horn_rod(f"horn{j}_{k}", bone, tuple(at), pitch, s * lean, length, w, back=3.0 if j == 0 else 0.6)
        E.append(e)
    return E


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 48, -22)), ("spine_1", "spine_0", (0, 46, -7)),
         ("neck", "spine_1", tuple(NECK0)), ("head", "neck", tuple(HEAD)), ("jaw", "head", hs((0, 60.1, 45.0))),
         ("tail", "spine_0", tuple(TAIL0))]
    for s, n in ((1, "L"), (-1, "R")):
        B += [(f"ear_{n}", "head", hs((s * EAR_X, 71.0, sum(EAR_Z) / 2))), (f"horn_{n}", "head", (s * HORN0[0], HORN0[1], HORN0[2]))]
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
    # Higher at the withers than at the hips, as Drenkaun.
    E = [el("hips", "spine_0", (-10.6, 35, -40), (10.6, 55.0, 0)),
         el("chest", "spine_1", (-11.4, 30, -8), (11.4, 62.5, 30)),
         el("brisket", "spine_1", (-9.6, 25.5, 12.0), (9.6, 34.0, 32.5)),
         el("shoulders", "spine_1", (-12.8, 44.0, -8.5), (12.8, 64.2, 26.5)),
         el("belly", "spine_0", (-9.2, 31.6, -31.0), (9.2, 38.0, -5.0)),
         el("skull", "head", *SKULL0),
         # A goat's muzzle: narrower than the skull (63%), straight-nosed, a dark nose at its end.
         el("muzzle", "head", hs((-4.4, 59.88, 45.5)), hs((4.4, 66.0, 51.8))),
         el("nose", "head", hs((-3.4, 59.78, 51.3)), hs((3.4, 64.0, 54.2))),  # a small dark nose, not a snout
         el("mouth", "head", hs((-2.7, 59.0, 44.5)), hs((2.7, 60.6, 50.3))),
         el("jaw", "jaw", hs((-3.1, 57.9, 44.0)), hs((3.1, 60.3, 50.6))),  # shorter than the muzzle
         # The beard from its chin (concept A): a pale tuft narrowing to a point, on the jaw, each
         # step in from the one above two texels or more (no thin lines).
         el("beard", "jaw", hs((-2.0, 51.5, 45.4)), hs((2.0, 58.4, 49.4))),
         el("beard_tip", "jaw", hs((-0.9, 46.5, 46.6)), hs((0.9, 52.0, 48.2)))]
    neck, _ = rod("neck", "neck", tuple(NECK0), NECK_ROT, NECK_LEN, NECK_W, back=6.0)  # slimmer than the head
    E.append(neck)
    n0 = NECK0
    hw = NECK_W / 2
    # The ruff (concept A): long pale hair under the throat, in locks; a crest of tufts along the
    # neck's top; the shell over the shoulders with its shaggy hem; a mat down the chest's front.
    E.append(el("throat", "neck", (-5.6, n0[1] + 3, n0[2] + hw - 1.5), (5.6, n0[1] + 24, n0[2] + hw + 3.1), rot=NECK_ROT, origin=tuple(n0)))
    E.append(el("fringe_throat", "neck", (0, n0[1] + 3.5, n0[2] + hw), (0, n0[1] + 24.5, n0[2] + hw + 21.0), quad="east", rot=NECK_ROT,
                origin=tuple(n0), cut="throat"))
    for j, (t, ln, w, d) in enumerate(((2.0, 8.0, 1.6, 3.2), (8.5, 7.0, 2.3, 2.6), (14.5, 8.5, 1.3, 3.6), (21.5, 6.5, 1.42, 2.4))):
        E.append(el(f"crest{j}", "neck", (-hw - w, n0[1] + t, n0[2] - hw - d), (hw + w, n0[1] + t + ln, n0[2] - hw + 2.0 + 1.0 * j),
                    rot=NECK_ROT, origin=tuple(n0)))
    E.append(el("ruff_shell", "spine_1", *SHELL, cut="shell"))
    E.append(el("ruff_hips", "spine_0", *SHELL_HIPS, cut="shell_hips"))
    E.append(el("ruff_chest", "spine_1", (-10.2, 34.0, 29.6), (10.2, 58.0, 34.9)))
    E.append(el("fringe_chest", "spine_1", (-11.4, 18.0, 35.4), (11.4, 58.0, 35.4), quad="south", cut="chest"))
    # A goat's short tail, held up.
    E.append(el("tail", "tail", (-2.4, 47.5, -42.8), (2.4, 54.2, -39.75), rot=(24, 0, 0), origin=tuple(TAIL0)))
    for s, n in ((1, "L"), (-1, "R")):
        cx = EAR_X * s
        rot, origin = (-10, 0, -s * 62), (cx, 71.0, sum(EAR_Z) / 2)
        ears = [el(f"ear_core_{n.lower()}{k}", f"ear_{n}", (cx - hw_, y0, EAR_Z[0]), (cx + hw_, y1, EAR_Z[1]), rot=rot, origin=origin)
                for k, (y0, y1, hw_) in enumerate(EAR)]
        zc = sum(EAR_Z) / 2
        ears.append(el(f"fringe_ear_{n.lower()}", f"ear_{n}", (cx - 2.8, 70, zc), (cx + 2.8, 80.6, zc), quad="south", rot=rot,
                       origin=origin, cut="ear"))
        for e in ears:  # the ears go with the bigger head
            e["lo"], e["hi"], e["origin"] = list(hs(e["lo"])), list(hs(e["hi"])), list(hs(e["origin"]))
        E += ears
        E += horn(n)
    # Legs: a thick shaggy upper part, a knee or hock where it bends, a slimmer dark shank, a broad
    # dark hoof (concept A).
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z = FRONT_X * s, FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 4.4, 18.5, z - 4.4), (x + 4.4, 44.6, z + 4.6)))
            E.append(el(f"knee_{leg}", f"leg_{leg}_lower", (x - 3.7, 17.0, z - 3.6), (x + 3.7, 23.5, z + 3.9)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 3.0, 4.6, z - 3.0), (x + 3.0, 20.0, z + 3.0)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 3.4, 0, z - 3.3), (x + 3.4, 5.4, z + 3.9)))
        else:
            x, z = BACK_X * s, BACK_Z
            xi, xo = sorted((x - 4.6 * s, x + 5.5 * s))
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (xi, 27.5, z - 7.0), (xo, 54.0, z + 6.5)))
            E.append(el(f"hock_{leg}", f"leg_{leg}_lower", (x - 3.5, 24.5, z - 3.4), (x + 3.5, 31.5, z + 3.6)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2.9, 5.4, z - 2.9), (x + 2.9, 26.0, z + 2.9)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 3.4, 0, z - 3.2), (x + 3.4, 6.0, z + 3.8)))
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



HORN_PARTS = ("horn",)
RUFF_PARTS = ("throat", "fringe_throat", "ruff_chest", "fringe_chest", "ruff_shell", "crest", "beard")


def local_y(e, p):
    """How far along a rod a texel is, as a share of its length (0 at its start)."""
    o = np.array(e["origin"])
    loc = (p - o) @ rotm(e["rot"])
    return (loc[..., 1] - (e["lo"][1] - o[1])) / max(e["hi"][1] - e["lo"][1], 1e-6)


def snowy(x, z):
    """Where snow lies on its back (concept A): ragged drifts along the spine, in the deer's space."""
    edge = 6.5 + 2.5 * np.sin(z * 0.45 + 1.3) + 1.5 * np.sin(z * 1.1)
    gaps = np.sin(z * 0.23 + 0.5) + 0.6 * np.sin(z * 0.71) > -0.35
    return (np.abs(x) < edge) & gaps


def area(e, face, p, uu, vv, h):
    """Concept A's colors, in the deer's space (`p` / K): a taupe coat darker along the back, cream
    under the belly; the cream ruff over the neck, chest and shoulders, its locks darker at their
    tips; snow in drifts along its back; a grey-tan face, pale muzzle and throat, a dark nose; dark
    shanks and hooves; the horns in ridges, dark grooves between pale ridges, lichen on them."""
    name = e["name"]
    p = p / K
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, FUR)
    t = vv / max(h, 1)
    if name in ("brisket", "shoulders", "belly"):
        if face == "up":
            a = np.where(snowy(x, z), SNOW, BACK)
        elif face == "down":
            a[:] = BELLY
        elif name == "belly" and face != "up":
            a = np.where(y < 34.5, BELLY, FUR)
        elif name == "brisket" and face == "south":
            a[:] = RUFF  # behind the chest's locks
    elif name in ("hips", "chest"):
        if face == "up":
            a = np.where(snowy(x, z), SNOW, BACK)
        elif face == "down":
            a[:] = BELLY
        else:
            a = np.where(t < 0.12, BACK, np.where(y < 34.5, BELLY, FUR))
            if name == "chest" and face == "south":
                a[:] = RUFF
            if name == "hips" and face in ("east", "west"):
                # Snow run down from its back in streaks (concept A).
                # Ahead of the thighs only: over them the hips' side is a strip under two texels.
                drip = 0.06 + 0.1 * (np.sin(uu * 0.8 + 1.0) + 1) * (np.sin(uu * 0.27) > -0.2)
                a = np.where((t < drip) & (z > -22.0), SNOW, a)
            if name == "hips" and face == "north":
                a = np.where((np.abs(x) < 6.0) & (y > 40), BELLY, a)  # the pale rump round the tail
    elif name == "ruff_hips":
        # The coat over the hips: taupe, shaggy, snow on top.
        if face == "up":
            a = np.where(snowy(x, z), SNOW, BACK)
        elif face != "down":
            tongue = 0.35 + 0.25 * np.sin(uu * 1.7) + 0.15 * np.sin(uu * 0.6)
            a = np.where(t > 1 - tongue * 0.6, BACK, FUR)
    elif name.startswith(RUFF_PARTS):
        a[:] = RUFF
        if e["quad"]:
            if not e["rot"]:
                a = np.where(y < 32.0, RUFFDK, RUFF)  # the chest's locks darker toward their tips
            else:
                a = np.where(t > 0.7, RUFFDK, RUFF)
        elif face == "up":
            if name == "ruff_shell" or name.startswith("crest"):
                a = np.where(snowy(x, z), SNOW, np.where(np.abs(x) < 9.0, BACK, RUFF))
        elif face == "down":
            a[:] = RUFFDK
        else:  # shaggy: darker below, in uneven tongues
            tongue = 0.35 + 0.25 * np.sin(uu * 1.7 + len(name)) + 0.15 * np.sin(uu * 0.6)
            a = np.where(t > 1 - tongue * 0.6, RUFFDK, RUFF)
            if name == "ruff_shell" and face in ("east", "west"):
                # Snow running a little way down its sides, in streaks (concept A).
                drip = 0.05 + 0.07 * (np.sin(uu * 0.9) + 1) * (np.sin(uu * 0.31 + 2.0) > 0)
                a = np.where(t < drip, SNOW, a)
        if name == "beard_tip":
            a = np.where(t > 0.5, RUFFDK, RUFF)
    elif name == "neck":
        a[:] = FUR
        if face in ("north", "up"):
            a[:] = RUFF
        elif face in ("south", "down"):
            a[:] = PALE
    elif name == "skull":
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = PALE
        elif face in ("east", "west"):
            a = np.where(z > hs((0, 0, 44.0))[2], MUZZLE, FUR)
        else:
            a[:] = FUR
    elif name == "muzzle":
        a[:] = MUZZLE
        if face == "up":
            a[:] = FUR
        elif face == "down":
            a[:] = PALE
    elif name == "nose":
        # Pale like the muzzle, dark only at its tip (concept A), the nostrils soft slits in it
        # (`paint_face`): a goat's nose, not a snout.
        a[:] = NOSE if face == "south" else (PALE if face == "down" else MUZZLE)
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else PALE
    elif name == "mouth":
        a[:] = MOUTH
    elif name.startswith(HORN_PARTS):
        # Ridges across the horn's front and sides, every six texels: a dark groove, a pale ridge
        # behind it; its inner side plainer; lichen in patches on the middle lengths (concept A).
        a[:] = HORN
        if face not in ("up", "down"):
            ly = local_y(e, p * K)
            span = TX * (e["hi"][1] - e["lo"][1])  # texels along it
            pos = ly * span + 3.0 * int(name[4])
            r = pos % 6.0
            if face == "north":
                a = np.where(r < 1.0, HORNDK, HORN)
            else:
                a = np.where(r < 1.0, HORNDK, np.where(r < 3.0, HORNLT, HORN))
            j = int(name[4])
            if 1 <= j <= 4:
                seed = sum(map(ord, name + face))
                patch = np.sin(pos * 0.55 + seed) * np.sin(uu * 0.9 + seed * 0.3) > 0.55
                a = np.where(patch & (r >= 1.0), LICHEN, a)
    elif name == "tail":
        a[:] = TAILTOP
    elif name.startswith("ear_core"):
        a[:] = FUR
        if face == "south":
            cx = e["origin"][0] / K
            a = np.where(np.abs(x - cx) < 1.5, EARIN, FUR)
    elif name.startswith(("knee_", "hock_")):
        a[:] = LEG
    elif name.startswith("thigh_"):
        a[:] = FUR
        if face == "up":
            a[:] = BACK
        a = np.where(y < 34, LEG, a)
    elif name.startswith("leg_"):
        a[:] = LEG
        if name.endswith("_upper"):
            hem = 24.0 + 2.5 * np.sin(uu * 1.9) + 1.5 * np.sin(uu * 0.7)
            a = np.where(y > hem, RUFF, LEG)
        if name.endswith("_paw"):
            a = np.where(y < 3.0, HOOF, LEG)
            if face == "down":
                a[:] = HOOF
    return a


def cut_mask(e, p):
    """Cut-out outlines: the ears' points, the ruff's locks (in the neck's own frame)."""
    if e["cut"] == "chest":  # the ruff's locks down the chest, each its own length, longest mid-chest
        x, y = p[..., 0] / K, p[..., 1] / K
        period = max(2.0, 3 / (TX * K))
        idx = np.floor(x / period)
        rnd = (np.sin(idx * 12.9898 + 3.1) * 43758.5453) % 1.0
        lock = np.abs(((x / period) % 1.0) - 0.5) * 2
        tip = 22.0 + 7.0 * (np.abs(x) / 11.4) ** 1.5 + 9.0 * rnd + 2.0 * lock
        return y >= tip
    if e["cut"] == "shell":  # the ruff: shaggy locks along its hem, longest over the shoulders' front,
        # and its back edge (on top and down the sides) in tongues over the back; open behind
        x, y, z = p[..., 0] / K, p[..., 1] / K, p[..., 2] / K
        (x0, y0, z0), (x1, y1, z1) = SHELL
        period = max(2.0, 3 / (TX * K))
        on_side = np.abs(np.abs(x) - x1) < 0.05
        on_top = np.abs(y - y1) < 0.05
        on_open = (np.abs(z - z0) < 0.05) | (np.abs(y - y0) < 0.05)
        along = np.where(on_side, z, x)
        lock = np.abs(((along / period) % 1.0) - 0.5) * 2
        rnd = (np.sin(np.floor(along / period) * 12.9898 + 3.0 * np.sign(x) + 5.0 * on_top) * 43758.5453) % 1.0
        u = np.clip((z - z0) / (z1 - z0), 0, 1)  # 0 at the back, 1 at the front
        tip_side = 57.0 - 11.0 * u ** 0.7 + 2.0 * lock + 3.5 * rnd
        across = np.where(on_side, y, x)  # the back edge's tongues run across it
        tl = np.abs(((across / period) % 1.0) - 0.5) * 2
        tr = (np.sin(np.floor(across / period) * 78.233 + 2.0 * on_side + 3.0 * np.sign(x)) * 43758.5453) % 1.0
        back = z0 + 1.0 + 5.0 * tr + 2.5 * tl
        keep = np.where(on_side, (y >= tip_side) & (z >= back), np.where(on_top, z >= back, True))
        return keep & ~on_open
    if e["cut"] == "shell_hips":  # the hips' coat: short shaggy locks, its back edge in tongues
        x, y, z = p[..., 0] / K, p[..., 1] / K, p[..., 2] / K
        (x0, y0, z0), (x1, y1, z1) = SHELL_HIPS
        period = max(2.0, 3 / (TX * K))
        on_side = np.abs(np.abs(x) - x1) < 0.05
        on_top = np.abs(y - y1) < 0.05
        on_open = (np.abs(z - z0) < 0.05) | (np.abs(y - y0) < 0.05)
        along = np.where(on_side, z, x)
        lock = np.abs(((along / period) % 1.0) - 0.5) * 2
        rnd = (np.sin(np.floor(along / period) * 78.233 + 5.0 * on_top + 3.0 * np.sign(x)) * 43758.5453) % 1.0
        u = np.clip((z - z0) / (z1 - z0), 0, 1)
        tip_side = 50.0 - 3.0 * u + 2.0 * lock + 2.5 * rnd + 2.5 * (1 - u) ** 2
        back = z0 + 1.0 + 4.5 * rnd + 2.0 * lock
        keep = np.where(on_side, y >= tip_side, np.where(on_top, z >= back, True))
        return keep & ~on_open
    o = np.array(e["origin"])
    local = (p - o) @ rotm(e["rot"]) + o  # back into the part's own (unturned) frame
    if e["cut"] == "ear":  # a goat's ear: narrow, rounded at the root, coming to a point
        x, y = local[..., 0] / K, local[..., 1] / K
        y0, y1 = hs((0, 70.0, 0))[1], hs((0, 80.6, 0))[1]
        t = np.clip((y - y0) / (y1 - y0), 0, 1)
        half = 3.2 * HS * (1 - t) ** 0.8 * np.clip(t * 4, 0, 1) ** 0.5
        return np.abs(x - o[0] / K) <= half
    along, depth = (local - o)[..., 1] / K, (local - o)[..., 2] / K  # the neck's frame, the deer's space
    u = np.clip((along - 3.5) / 21.0, 0, 1)
    period = max(2.5, 3 / (TX * K))  # a lock every 3 texels or more
    lock = np.abs(((along / period) % 1.0) - 0.5) * 2
    tip = 12.0 + 10.0 * np.sin(math.pi * u) ** 0.7 - max(1.6, 1 / (TX * K)) * lock
    return depth <= tip



LASH = (40, 30, 28)


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
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "jaw", "tail", "throat", "beard")):
        light_face = "north"
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    stacked = e["name"].startswith("ear_core")
    down = np.clip((80.0 * K - p[..., 1]) / (8.5 * K), 0, 1) if stacked else None
    cap = {}
    horn = e["name"].startswith(HORN_PARTS)
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT,
                            y_world=p[..., 1] if not horn else None,
                            occ=occ, quad=bool(e["quad"]), seed=sum(map(ord, e["name"] + face)) % 97, down=down,
                            tuft=e["name"] not in ("skull", "muzzle", "nose") and not horn,
                            merge=e["name"] != "skull" and not horn)  # keep the eye ring and the horns' grooves
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"]:
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "skull" and face in ("east", "west"):
        s = 1 if face == "east" else -1
        stamp_eyes(img, [eye_center(s)], eye_style(s))
    if e["name"] == "nose" and face == "south":
        # Nostrils as short soft slits low on the nose, the lip a line below them: never two dark
        # marks high on the face, which read as a second pair of eyes (the skill's face rule).
        r = max(1, int(h * 0.62))
        soft = (44, 32, 26)
        if w >= 10:
            for c0 in (int(w * 0.18), int(w * 0.70)):
                img[r, c0:c0 + max(2, w // 8), :3] = soft
            img[h - 2, w // 2 - 1:w // 2 + 1, :3] = (img[h - 2, w // 2 - 1:w // 2 + 1, :3] * 0.8).astype(np.uint8)
    if e["name"].startswith("eyelid"):
        col_idx = 0 if e["name"].endswith("L") else w - 1
        img[:, col_idx, :3] = LASH
    if e["name"].endswith("_paw") and face == "south" and w >= 3 and w % 2:  # the cloven hoof: a split line, on a middle texel
        c = w // 2
        img[:, c, :3] = (img[:, c, :3] * 0.6).astype(np.uint8)
    return img


# ------------------------------------------------------------------ animation

L = {"lie_front": 80, "lie_hip": -70, "lie_knee": 140, "lie_drop": 30, "crouch_tilt": 0, "crouch_drop": 0}



# Standing, its legs bend where a deer's do (Ronald's round: "the legs bend visibly at knee and
# hock"): the hind leg's thigh back and its cannon forward, so the hock points back; the foreleg a
# little at the knee; each hoof kept flat (the angles of a leg sum to 0). The drop it takes to
# keep the hooves on the ground is fitted (`fit_poses`, `stance_drop`).
STANCE = {"F": (-10.0, 18.0, -8.0), "B": (10.0, -17.0, 7.0)}  # fitted so both pairs of hooves stand level


def with_stance(q):
    """`q` with the standing bend added to each leg's three bones."""
    q = dict(q)
    for leg in LEGS:
        for part, a in zip(("upper", "lower", "paw"), STANCE[leg[0]]):
            k = f"leg_{leg}_{part}"
            x, y, z = q.get(k, (0, 0, 0))
            q[k] = (x + a, y, z)
    return q


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
    return with_stance(out)


def chew(t, rate=2.5, amp=7.0):
    return {"jaw": (amp * max(0.0, S(TAU * t * rate)), 0, 0)}



HEAD_UP = (-30, 0, 0), (20, 0, 0)  # neck raised, head held high: an ibex's carriage (concept A)


def stand(q=None):
    """Its standing carriage under every clip: the neck raised, the head level."""
    q = dict(q or {})
    q.setdefault("neck", HEAD_UP[0])
    q.setdefault("head", HEAD_UP[1])
    if not any(k.startswith("leg_") for k in q):
        q = with_stance(q)
    return q


def pose_idle(t):
    """Standing guard: breathing deep, looking slowly about, an ear and a tail flick (8 s)."""
    p = t / 8.0
    br = S(TAU * p * 2)
    look = 18 * S(TAU * p)
    fl = max(0.0, S(TAU * (p - 0.3) * 12)) if 0.3 < p < 0.34 else 0.0
    tf = max(0.0, S(TAU * (p - 0.7) * 10)) if 0.7 < p < 0.75 else 0.0
    return stand({"root": (0, 0.6 * br, 0), "spine_1": (0.6 * br, 0, 0), "neck": (HEAD_UP[0][0] + 2 * br, 0.3 * look, 0),
                  "head": (HEAD_UP[1][0], 0.7 * look, 0), "ear_L": (0, 0, -15 * fl), "ear_R": (0, 0, 2 * br),
                  "tail": (-25 * tf, 0, 0)})


def pose_walk(t):
    """The long-striding walk, the head nodding low (1.5 s)."""
    p = t / 1.5
    q = gait(p, (0.0, 0.5, 0.75, 0.25), 20, 36)
    q.update({"root": (0, 0.6 * abs(S(TAU * p * 2)), 0), "neck": (HEAD_UP[0][0] + 3 * S(TAU * p * 2), 0, 0),
              "head": (HEAD_UP[1][0] - 3 * S(TAU * p * 2), 0, 0), "tail": (-4, 0, 0)})
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


def ease(p, a, b):
    """0 before `a`, 1 after `b`, smooth between."""
    u = min(1.0, max(0.0, (p - a) / max(b - a, 1e-6)))
    return u * u * (3 - 2 * u)


def pose_rise(t):
    """Out of the ground (3 s): sunk to its back, the body pitched up as it climbs."""
    p = t / 3.0
    u = ease(p, 0.0, 0.85)
    q = stand({"root": (0, -62.0 * K * (1 - u), 0), "spine_0": (-14 * (1 - u), 0, 0)})
    q["neck"] = (HEAD_UP[0][0] - 10 * S(math.pi * u), 0, 0)
    return q



def pose_charge_tell(t):
    """Before the charge (1.4 s): the head down, the horns brought forward, a forehoof scraping twice."""
    p = t / 1.4
    d = ease(p, 0.0, 0.3)
    scrape = S(TAU * p * 2) if p < 0.9 else 0.0
    return stand({"neck": (HEAD_UP[0][0] + 45 * d, 0, 0), "head": (HEAD_UP[1][0] + 28 * d, 0, 0), "spine_1": (4 * d, 0, 0),
                  "leg_FR_upper": (-23 * max(0.0, scrape), 0, 0), "leg_FR_lower": (30 * max(0.0, scrape), 0, 0),
                  "tail": (-30 * d, 0, 0)})


def pose_charge(t):
    """The charge (2.5 s): a gallop with the head down, the horns levelled at what it runs at."""
    q = pose_run(t % 0.55)
    q.update({"neck": (HEAD_UP[0][0] + 48, 0, 0), "head": (HEAD_UP[1][0] + 26, 0, 0), "tail": (-45, 0, 0)})
    return q


def pose_toss_tell(t):
    """Before the toss (0.9 s): the head low and turned a little, the horns hooked under."""
    d = ease(t / 0.9, 0.0, 0.7)
    return stand({"neck": (HEAD_UP[0][0] + 46 * d, -12 * d, 0), "head": (HEAD_UP[1][0] + 16 * d, -4 * d, -4 * d),
                  "spine_1": (3 * d, 0, 0), "tail": (-20 * d, 0, 0)})


def pose_toss(t):
    """The toss (0.6 s): the head flung up and round, the horns hooking up through whoever is
    there, then back to its carriage."""
    p = t / 0.6
    up = ease(p, 0.0, 0.3) - ease(p, 0.6, 1.0)
    low = 1 - ease(p, 0.0, 0.3)
    return stand({"neck": (HEAD_UP[0][0] + 46 * low - 26 * up, -12 * low + 14 * up, 0),
                  "head": (HEAD_UP[1][0] + 16 * low - 10 * up, -4 * low + 4 * up, -4 * low + 6 * up),
                  "spine_1": (3 * low - 4 * up, 0, 0), "spine_0": (-3 * up, 0, 0)})


def front_up(r):
    """The front legs drawn up while it rears by `r` (0..1)."""
    q = {}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (-55 * r, 0, 0)
        q[f"leg_{leg}_lower"] = (95 * r, 0, 0)
    return q


def pose_rockfall_tell(t):
    """Before the rockfall (1.6 s): rearing high on its hind legs, the head thrown back, as ibex
    rear before they clash."""
    r = ease(t / 1.6, 0.0, 0.6)
    q = stand({"spine_0": (-28 * r, 0, 0), "neck": (HEAD_UP[0][0] - 10 * r, 0, 0), "head": (HEAD_UP[1][0] - 6 * r, 0, 0),
               "tail": (-20 * r, 0, 0)})
    q.update(front_up(r))
    for leg in ("BL", "BR"):  # the hind legs stay planted as the body tips back over them
        q[f"leg_{leg}_upper"] = (STANCE["B"][0] + 28 * r, 0, 0)
    return q


def pose_rockfall(t):
    """The rockfall (0.7 s): down hard on both forefeet, the head and horns thrown down, the
    mountain shaking stones loose; then up."""
    p = t / 0.7
    r = 1 - ease(p, 0.0, 0.3)
    hit = (1 - r) * (1 - ease(p, 0.6, 1.0))
    q = stand({"spine_0": (-28 * r, 0, 0), "neck": (HEAD_UP[0][0] + 30 * hit, 0, 0), "head": (HEAD_UP[1][0] + 12 * hit, 0, 0),
               "root": (0, -2.0 * K * (1 - r) * (1 - ease(p, 0.4, 1.0)), 0)})
    q.update(front_up(r))
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (STANCE["B"][0] + 28 * r, 0, 0)
    return q


# Crouched to leap: the hind legs folded under it (the thigh forward, the cannon back, the hoof
# flat), the body tipped back over them; the tilt and the drop are fitted so all four hooves stand
# on the ground (`fit_poses`).
CROUCH = (-52.0, 98.0, -46.0)


def crouch(c, P=L):
    """Crouched by `c` (0..1), the head raised toward where it will land."""
    q = stand({"spine_0": (P["crouch_tilt"] * c, 0, 0), "neck": (HEAD_UP[0][0] - 12 * c, 0, 0), "head": (HEAD_UP[1][0] + 4 * c, 0, 0),
               "root": (0, P["crouch_drop"] * c, 0)})
    for leg in ("BL", "BR"):
        for part, a, st in zip(("upper", "lower", "paw"), CROUCH, STANCE["B"]):
            q[f"leg_{leg}_{part}"] = (st + a * c, 0, 0)
    for leg in ("FL", "FR"):  # the front legs bend a little at the knee to keep level
        q[f"leg_{leg}_upper"] = (STANCE["F"][0] - 8 * c, 0, 0)
        q[f"leg_{leg}_lower"] = (STANCE["F"][1] + 16 * c, 0, 0)
        q[f"leg_{leg}_paw"] = (STANCE["F"][2] - 8 * c, 0, 0)
    return q


def pose_leap_tell(t):
    """Before the leap (1.0 s): crouching on its haunches, the head up at where it will land."""
    return crouch(ease(t / 1.0, 0.0, 0.8))


def pose_leap(t):
    """The leap (1.2 s): springing off its haunches, the forelegs tucked and the hind legs
    stretched back in the air, then reaching down to land."""
    p = t / 1.2
    off = 1 - ease(p, 0.0, 0.15)
    fly = ease(p, 0.05, 0.25) * (1 - ease(p, 0.7, 0.95))
    q = crouch(off)
    q.update(front_up(fly))
    for leg in ("BL", "BR"):
        st = STANCE["B"]
        q[f"leg_{leg}_upper"] = (st[0] + CROUCH[0] * off + 48 * fly, 0, 0)
        q[f"leg_{leg}_lower"] = (st[1] + CROUCH[1] * off - 10 * fly, 0, 0)
        q[f"leg_{leg}_paw"] = (st[2] + CROUCH[2] * off + 25 * fly, 0, 0)
    q["neck"] = (HEAD_UP[0][0] - 12 * off - 6 * fly, 0, 0)
    q["spine_0"] = (L["crouch_tilt"] * off - 6 * fly, 0, 0)
    q["tail"] = (-35 * fly, 0, 0)
    return q



def pose_stunned(t):
    """Stunned after charging into rock (2 s, looped): the head low, swaying."""
    p = t / 2.0
    sw = S(TAU * p)
    return stand({"neck": (HEAD_UP[0][0] + 38, 10 * sw, 0), "head": (HEAD_UP[1][0] + 10, 12 * sw, 8 * sw),
                  "spine_1": (0, 0, 2.5 * sw), "root": (0, -0.8 * K * abs(sw), 0)})


def pose_down(t, P=L):
    """Knocked down (4 s, looped): on its belly, the legs folded, the head low, heaving."""
    q = pose_lie(t * 1.5 % 6.0, P)
    br = S(TAU * t / 2.0)
    q.update({"neck": (20 + 2 * br, 0, 0), "head": (10, 0, 0), "spine_1": (1.5 * br, 0, 0), "jaw": (6, 0, 0)})
    return q


BLINKS = {"idle": [2.0, 5.9]}
BLINK = 0.12
ANIMS = {"idle": (pose_idle, 8.0), "walk": (pose_walk, 1.5), "run": (pose_run, 0.55), "lie": (pose_lie, 6.0),
         "rise": (pose_rise, 3.0), "charge_tell": (pose_charge_tell, 1.4), "charge": (pose_charge, 2.5),
         "toss_tell": (pose_toss_tell, 0.9), "toss": (pose_toss, 0.6), "rockfall_tell": (pose_rockfall_tell, 1.6),
         "rockfall": (pose_rockfall, 0.7), "leap_tell": (pose_leap_tell, 1.0), "leap": (pose_leap, 1.2),
         "stunned": (pose_stunned, 2.0), "down": (pose_down, 4.0)}
# One-off clips play once (an action that loops would play on: `borkon.rs` asks for each by name).
LOOPED = ("idle", "walk", "run", "lie", "stunned", "down")
CLIPS = [n for n in ANIMS if n not in ("idle", "walk", "run", "lie")]


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}


def frame_times(name, dur):
    n = {"idle": 96, "lie": 60, "walk": 24, "run": 20, "charge": 100, "down": 60, "stunned": 40, "leap": 36}.get(name, 30)
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
            root = list(pose.get("root", (0, 0, 0)))
            if name not in ("lie", "down"):  # standing on bent legs, the hooves on the ground
                root[1] -= L.get("stance_drop", 0.0)
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in root]})
        out[name] = {"length": dur, "frames": frames, "loop": name in LOOPED}
    return out


# ------------------------------------------------------------------ fitting poses to the ground

GROUPS = {"front paws": lambda n: n in ("leg_FL_paw", "leg_FR_paw"), "back paws": lambda n: n in ("leg_BL_paw", "leg_BR_paw"),
          "body": lambda n: n in ("hips", "chest", "brisket", "belly"), "muzzle": lambda n: n in ("nose", "jaw"),
          "legs": lambda n: n.startswith(("leg_", "thigh_", "knee_", "hock_")), "horns": lambda n: n.startswith(HORN_PARTS),
          "mane": lambda n: n in ("throat", "fringe_throat", "beard", "beard_tip")}


def fit_poses(spec):
    """Standing on its bent legs (the drop that keeps the hooves on the ground), lying (the belly
    on the ground, the folded legs under it, nothing below) and crouching to leap (all four hooves
    on the ground), by forward kinematics."""
    g = lowest(spec, stand(), GROUPS)
    L["stance_drop"] = round(float(min(g["front paws"], g["back paws"])), 2)
    print("standing, lowest:", {k: round(v, 2) for k, v in g.items()})

    def lie_cost(P):
        g = lowest(spec, pose_lie(0.0, dict(L, **P, lie_drop=0)), GROUPS)
        f = min(v for k, v in g.items() if k != "mane")
        return (g["body"] - f - 0.8) ** 2 + 0.3 * (g["front paws"] - f) ** 2 + 0.3 * (g["back paws"] - f) ** 2
    grid = {"lie_front": list(range(60, 101, 5)), "lie_hip": list(range(-100, -49, 5)), "lie_knee": list(range(115, 166, 5))}
    best = fit(grid, lie_cost)[1]
    g = lowest(spec, pose_lie(0.0, dict(L, **best, lie_drop=0)), GROUPS)
    L.update(best, lie_drop=round(float(min(g.values())), 2))

    def crouch_cost(P):
        g = lowest(spec, crouch(1.0, dict(L, **P, crouch_drop=0)), GROUPS)
        return (g["front paws"] - g["back paws"]) ** 2
    best = fit({"crouch_tilt": [x * 0.5 for x in range(-40, 21)]}, crouch_cost)[1]
    g = lowest(spec, crouch(1.0, dict(L, **best, crouch_drop=0)), GROUPS)
    L.update(best, crouch_drop=round(float(L["stance_drop"] - min(g["front paws"], g["back paws"])), 2))
    print("crouching, lowest:", {k: round(v, 2) for k, v in lowest(spec, crouch(1.0), GROUPS).items()})
    ll = lowest(spec, pose_lie(0.0), GROUPS)
    print("lying, lowest:", {k: round(v, 2) for k, v in ll.items()})


# ------------------------------------------------------------------ build


def build():
    B = bones()
    E = elements()
    # A shell with a cut-out hem casts no occlusion: what shows through its hem is lit as open fur.
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"] and not e["rot"] and not str(e["cut"]).startswith("shell")]
    tiles = []
    for i, e in enumerate(E):
        for f in ([e["quad"]] if e["quad"] else list(FACES)):
            tiles.append(((i, f), paint_face(e, f, boxes)))
    sheet, uv = pack(tiles, 1024)
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "borkon", "texture": [sheet.shape[1], sheet.shape[0]],
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
