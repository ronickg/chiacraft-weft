#!/usr/bin/env python3
"""The jaguar (*Panthera onca*): the savanna's and the swamp's top hunter, the caiman's and the
capybara's (`notes/ECOLOGY.md`, tier 1), made the way Hytale makes creatures (boxes, a few flat
panels, a hand-painted texture) with the blocky-model skill, from research alone (README.md), on the
cat's build (`../cat/cat.py`): laid out in the cat's units with a jaguar's build (a longer, stockier
body, thick legs and big paws, a short thick tail carried low, small round ears, a broad muzzle), the
head scaled about its pivot (`HS`), then warped into the house's units (`warp`: the legs a little
shorter, everything `K` times bigger). Its colors are rules (see `area`): a golden coat with black
rosettes, painted in the most common coat (the game paints every jaguar's from its genes).

    ~/.venvs/blocky/bin/python jaguar.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each), y up, it faces +z. Painted at the house density,
32 texels a block (`TX` texels a unit), as the blocks and items are. A jaguar is about 70 cm
at the shoulder, 1.5 m from the nose to the rump and has a 60 cm tail: here the withers are 45 up,
the body 96 long, the tail 40; the head a little bigger than life (Hytale's way).
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

# TX texels a unit: 32 a block (house units, 64 a block). The face that carries the eyes is ~11 by 9
# texels at 32, well over house.MIN_FACE, so no finer density is needed.

OUT = os.environ.get("JAGUAR_OUT") or os.path.join(HERE, "out")

# The cat's units into the house's: the head scaled HS about its pivot, then below Y0 (the legs)
# squeezed to reach D lower, everything above lowered by D, all of it K times bigger.
K, Y0, D, HS = 2.25, 11.0, -2.0, 0.6
HEAD_PIVOT = np.array([0.0, 24.0, 14.0])
HEAD_BONES = set()  # the head and the bones under it, filled by `bones()`


def warp(v):
    v = np.asarray(v, float)
    y = v[..., 1]
    y2 = np.where(y < Y0, y * (Y0 + D) / Y0, y + D)
    return np.stack([v[..., 0] * K, y2 * K, v[..., 2] * K], -1)


def unwarp(v):
    v = np.asarray(v, float)
    y = v[..., 1] / K
    y2 = np.where(y < Y0 + D, y * Y0 / (Y0 + D), y - D)
    return np.stack([v[..., 0] / K, y2, v[..., 2] / K], -1)


def place(v, bone):
    """A point of `bone` from the cat's units to the model's."""
    v = np.asarray(v, float)
    if bone in HEAD_BONES:
        v = HEAD_PIVOT + (v - HEAD_PIVOT) * HS
    return warp(v)


def unplace(v, bone):
    """A point of the model's back to the cat's units (for the color rules)."""
    v = unwarp(v)
    if bone in HEAD_BONES:
        v = HEAD_PIVOT + (v - HEAD_PIVOT) / HS
    return v
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

COAT, WHITE, STRIPE, PINK, NOSE, DARK, TAN = range(7)
# COAT the golden tan, WHITE the cream underside, throat, muzzle and inner legs, STRIPE the black of
# the rosettes and spots (painted as the plain coat in the layers: the genes draw the rosettes),
# PINK the ears' insides, NOSE its pink-brown tip, DARK the mouth, TAN the rosettes' darker centers.
BASE = [(212, 158, 76), (236, 226, 204), (40, 32, 30), (196, 150, 136), (176, 116, 102), (64, 30, 36), (176, 116, 54)]
SHADOW = [0.02, 0.06, 0.0, 0.97, 0.97, 0.95, 0.02]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
FUR = (COAT, WHITE)  # spots stay crisp: thin areas would be merged into the coat
ROLES = [("coat", 0, 0), ("coat", 1, 0), ("coat", 0, 0), "fixed", "fixed", "fixed", ("coat", 0, -0.5)]

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
})
EYES.update({
    "jaguar": dict(pattern=[".OOOO.",
                            "OHPPAO",
                            "OAPPAO",
                            "OAAAsO",
                            ".OaaO."], colors={"O": _O, "P": _P, "H": _H, "A": (204, 150, 48), "a": (230, 184, 76),
                                               "s": (238, 216, 168)}, halo="round"),  # a big cat's round pupil
    # At 32 a block the face is 11 texels by 9: an eye is 3 by 3. A big cat's black lid line over
    # the outer corner, the round pupil with a catchlight toward the outside, amber round it and
    # below, the fur round it darkened (the face is big enough for a halo). JAGUAR_EYES picks; the
    # other two are for comparison.
    "jaguar_px": dict(pattern=[".OO",
                               "APH",
                               ".a."], colors={"O": _O, "P": _P, "H": _H, "A": (176, 116, 28), "a": (226, 176, 60)},
                      halo="round", iris="Aa", mirror=True),
    "jaguar_px_full": dict(pattern=["aAH",
                                    "APA",
                                    "aAa"], colors={"P": _P, "H": _H, "A": (176, 116, 28), "a": (226, 176, 60)},
                           halo="round", iris="Aa", mirror=True),
    "jaguar_px_round": dict(pattern=[".O.",
                                     "APH",
                                     ".a."], colors={"O": _O, "P": _P, "H": _H, "A": (204, 150, 48), "a": (230, 184, 76)},
                            halo="round", iris="Aa", mirror=True),
})
EYE_STYLE = os.environ.get("JAGUAR_EYES", "jaguar_px")
EYE_Y, EYE_X = 29.5, 4.0  # eye centers on the face
LID_Z = 0.0  # the eyelids' hinge line, just in front of the placed face (set below, once `place` exists)

# ------------------------------------------------------------------ the model

SKULL = ((-8, 20.5, 11), (8, 33.5, 24))
HEAD_BONES.add("head")
LID_Z = float(warp(HEAD_PIVOT + (np.array(SKULL[1]) - HEAD_PIVOT) * HS)[2]) + 0.15
# The shortest tail of the big cats, thick, carried low and turning up at the tip: pitch (up),
# length, width (0.4 a side thinner each).
TAIL = [(-40, 5.0, 4.6), (-40, 5.0, 3.8), (-40, 4.5, 3.0), (-40, 3.5, 2.2)]  # at one pitch: the bends are the poses'
TAIL0 = np.array([0.0, 17.0, -20.0])  # its top below the hips' top


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


def skull_placed():
    return [place(c, "head") for c in SKULL]


def eye_center(cx):
    """The eye's middle on the placed face (cat units in, model units out)."""
    return place((cx, EYE_Y, SKULL[1][2]), "head")


def skull_grid():
    """The placed skull front's texels: (width, height) and the size of one texel in units (u, v)."""
    lo, hi = skull_placed()
    sx, sy = hi[0] - lo[0], hi[1] - lo[1]
    w, h = max(1, round(TX * sx)), max(1, round(TX * sy))
    return w, h, sx / w, sy / h


def eye_texel(cx):
    """The eye centered at cat x = cx on the skull's front, in its texels (u, v)."""
    lo, hi = skull_placed()
    w, h, su, sv = skull_grid()
    ex, ey, _ = eye_center(cx)
    return (ex - lo[0]) / su, (hi[1] - ey) / sv


def eye_box(cx):
    """(x0, x1, top, bottom) of the eye centered at cat x = cx, on the placed face, rounded as
    `stamp_eyes` places it, so the eyelid covers it to the texel."""
    pat = EYES[EYE_STYLE]["pattern"]
    ph, pw = len(pat), len(pat[0])
    lo, hi = skull_placed()
    w, h, su, sv = skull_grid()
    u0, v0 = eye_corner(*eye_texel(cx), pat, w)
    x0, top = lo[0] + u0 * su, hi[1] - v0 * sv
    return x0, x0 + pw * su, top, top - ph * sv


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 16, -13)), ("spine_1", "spine_0", (0, 16, -2)),
         ("neck", "spine_1", (0, 20, 11)), ("head", "neck", (0, 24, 14)), ("jaw", "head", (0, 23.5, 21.5))]
    for s, n in ((1, "L"), (-1, "R")):
        B += [(f"ear_{n}", "head", (s * 5.5, 33, 17))]
    pts = tail_points()
    B += [(f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", tuple(pts[k])) for k in range(len(TAIL))]
    for leg in ("FL", "FR", "BL", "BR"):
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z, parent, top, knee = 4.6 * s, 10.25, "spine_1", 14.5, 8
        else:
            x, z, parent, top, knee = 4.6 * s, -15.5, "spine_0", 15, 8
        B += [(f"leg_{leg}_upper", parent, (x, top, z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, knee, z)),
              (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 2.5, z))]
    parent = {n: p for n, p, _ in B}
    HEAD_BONES.clear()
    for n, _, _ in B:
        b = n
        while b and b != "head":
            b = parent[b]
        if b == "head":
            HEAD_BONES.add(n)
    out = [(n, p, tuple(float(v) for v in place(pv, n if n != "head" else "root"))) for n, p, pv in B]
    for s, n in ((1, "L"), (-1, "R")):  # the eyelids, on the placed face
        x0, x1, top, _ = eye_box(s * EYE_X)
        out.append((f"eyelid_{n}", "head", ((x0 + x1) / 2, top + 0.25, LID_Z)))
        HEAD_BONES.add(f"eyelid_{n}")
    return out


def elements():
    E = [el("hips", "spine_0", (-5.9, 11, -21), (5.9, 21, 3)),  # a longer back than a cat's, deep in the chest
         el("chest", "spine_1", (-6.6, 11.5, -2.5), (6.6, 22, 12)),  # deep and broad
         el("neck", "neck", (-3.6, 17.5, 9), (3.6, 25, 15)),  # slimmer than the head it enters
         el("skull", "head", *SKULL),
         el("muzzle", "head", (-4.2, 24, 23.5), (4.2, 28, 27)),  # broad
         el("mouth", "head", (-2.5, 22.5, 21.5), (2.5, 24.5, 24.8)),
         el("jaw", "jaw", (-3, 21.5, 21), (3, 24, 25.5)),
         el("nose", "head", (-1.5, 26.5, 27), (1.5, 28, 27.6))]
    # Ears: steps narrowing to a point (solid), plus one cut-out panel through the middle for a
    # smooth triangle with a few tufts; turned out a little and leaning out.
    steps = [(33, 35, 3.6), (35, 36.6, 2.9), (36.6, 37.8, 1.8)]  # y0, y1, half width: small and round
    for s, n in ((1, "L"), (-1, "R")):
        cx = 5.5 * s
        rot, origin = (0, s * 10, -s * 14), (cx, 33, 17)
        for k, (y0, y1, hw) in enumerate(steps):
            E.append(el(f"ear_core_{n.lower()}{k}", f"ear_{n}", (cx - hw, y0, 16), (cx + hw, y1, 18), rot=rot, origin=origin))
        E.append(el(f"fringe_ear_{n.lower()}", f"ear_{n}", (cx - 4.4, 33, 17), (cx + 4.4, 38.6, 17), quad="south",
                    rot=rot, origin=origin, cut="ear"))
    pts = tail_points()
    for k, (pitch, length, w) in enumerate(TAIL):
        p = pts[k]
        E.append(el(f"tail_{k}", f"tail_{k}", (p[0] - w / 2, p[1] - w / 2, p[2] - length), (p[0] + w / 2, p[1] + w / 2, p[2] + 1.2),  # reaching into the one before
                    rot=(pitch, 0, 0), origin=tuple(p)))
    for leg in ("FL", "FR", "BL", "BR"):
        s = 1 if leg[1] == "L" else -1
        x = 4.6 * s
        if leg[0] == "F":
            z = 10.25
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 2.9, 8, z - 2.9), (x + 2.9, 15, z + 2.9)))  # stocky
        else:
            z = -15.5
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (x - 3.3, 7.5, z - 4.4), (x + 3.3, 18, z + 4.4)))  # holds the knee (y = 8)
        E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2.3, 2.5, z - 2.3), (x + 2.3, 8, z + 2.3)))
        E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 3.7, 0, z - 2.7), (x + 3.7, 2.5, z + 3.9)))  # big paws
    for e in E:  # the cat's units into the model's
        e["lo"], e["hi"] = [float(v) for v in place(e["lo"], e["bone"])], [float(v) for v in place(e["hi"], e["bone"])]
        if e["origin"]:
            e["origin"] = [float(v) for v in place(e["origin"], e["bone"])]
    for s, n in ((1, "L"), (-1, "R")):
        # The eyelid, on the placed face: a quarter texel bigger all round than the eye and just in
        # front of the face (the face isn't whole texels once scaled), so a closed lid hides it.
        x0, x1, top, bottom = eye_box(s * EYE_X)
        x0, x1, top, h = x0 - 0.3, x1 + 0.3, top + 0.25, top - bottom + 0.55
        # Tilted a few degrees into the head, so it is never in the face's plane.
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (x0, top, LID_Z - h), (x1, top, LID_Z), quad="down", rot=(-4, 0, 0),
                    origin=((x0 + x1) / 2, top, LID_Z)))
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
        w, h = max(1, w), max(1, h)  # a thin face keeps a texel, or it vanishes
    return np.array(corner, float), du, dv, w, h, n


def rotm(rot):
    if not rot:
        return np.eye(3)
    rx, ry, rz = (math.radians(a) for a in rot)
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


def stripes(t, period=5.5, width=0.3):
    return ((t / period) % 1.0) < width


def _hash(i, j, s=0):
    h = np.sin(i * 12.9898 + j * 78.233 + s * 37.719) * 43758.5453
    return h - np.floor(h)


ROSETTE = dict(cell=7.4, inner=1.4, outer=2.6, broken=-0.8)  # the body's rosettes (cat units)


def rosettes(u, v, cell=None, solid=False):
    """The jaguar's marks on a face, from its two face coordinates (cat units): in each jittered cell
    a rosette (a broken ring of black round a darker center, often with a spot in it), or a solid
    spot where `solid` (the head, the legs, the belly). Returns STRIPE, TAN or -1 per texel.
    At 32 a block a texel is ~0.9 cat units on the body (~1.5 on the head): a rosette is ~6 texels
    across, a tan middle 3 across in a ring a texel thick and seldom broken (gaps a texel wide only
    speckle), a solid spot 1-3 texels, so they read and don't speckle."""
    cell = cell or ROSETTE["cell"]
    ci, cj = np.floor(u / cell), np.floor(v / cell)
    cu = (ci + 0.3 + 0.4 * _hash(ci, cj, 1)) * cell
    cv = (cj + 0.3 + 0.4 * _hash(ci, cj, 2)) * cell
    du, dv = u - cu, v - cv
    d = np.hypot(du, dv)
    out = np.full(u.shape, -1)
    if solid:
        r = cell * (0.17 + 0.1 * _hash(ci, cj, 3))
        return np.where(d < r, STRIPE, out)
    ang = np.arctan2(dv, du)
    ri, ro = ROSETTE["inner"], ROSETTE["outer"]
    broken = np.cos(3 * ang + 6 * _hash(ci, cj, 4)) > ROSETTE["broken"]  # the ring broken in places
    out = np.where(d < ri, TAN, out)
    out = np.where((d >= ri) & (d < ro) & broken, STRIPE, out)  # a thick ring, as a jaguar's are
    out = np.where((d < 0.6) & (_hash(ci, cj, 5) > 0.4), STRIPE, out)  # the spot in the middle
    return out


def marks(a, m, where):
    return np.where(where & (m >= 0), m, a)


SPOT = 4.6  # the solid spots' cell (cat units): 3 texels on the head, 5 on the body and legs


def area(e, face, p, uu, vv, along=None):
    """A jaguar, in the cat's units: golden tan, black rosettes with darker centers on the back and
    flanks, solid black spots on the head, the legs and the cream belly; the throat, the muzzle and
    the inner legs cream; the ears black behind with a pale spot; the tail spotted, ringed toward
    its black tip."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, COAT)
    side = face in ("east", "west")
    if name in ("hips", "chest", "neck"):
        if side:
            a = marks(a, rosettes(z, y), y > 14)
        elif face == "up":
            a = marks(a, rosettes(z, x), np.ones(x.shape, bool))
        a = np.where(y < 13.5, WHITE, a)  # the belly
        if face == "down":
            a[:] = WHITE
            a = marks(a, rosettes(z, x, SPOT, True), np.ones(x.shape, bool))
        if face == "south" and name != "hips":
            a = np.where(np.abs(x) < 4.8 - 0.25 * np.maximum(y - 16, 0), WHITE, a)  # the pale throat and chest
        if face == "north" and name == "hips":
            a = marks(a, rosettes(x, y), y > 14)
    elif name == "skull":
        if face == "south":
            a = np.where((y < 26.5) & (np.abs(x) < 4.8), WHITE, a)  # round the muzzle
            a = marks(a, rosettes(x, y, SPOT, True), (y > 30.5) & (np.abs(x) < 2.2))  # small spots on the forehead, between the eyes
        elif side or face == "up":
            a = marks(a, rosettes(z if side else x, y if side else z, SPOT, True), np.ones(x.shape, bool))
            if side:
                a = np.where((y < 24) & (z > 16), WHITE, a)
        if face == "down":
            a = np.where(z > 15, WHITE, a)
    elif name in ("muzzle", "jaw"):
        a[:] = WHITE
        if name == "muzzle" and face in ("east", "west") and x.shape[1] >= 4:
            a = marks(a, rosettes(z, y, 1.8, True), y > 26.5)  # the whisker spots (a muzzle side is 2 texels at 32: none)
    elif name == "nose":
        a[:] = NOSE
    elif name == "mouth":
        a[:] = DARK
    elif name.startswith("ear_core"):
        cx = e["origin"][0]
        if face == "south":  # pale fur inside a coat rim
            a = np.where(np.abs(x - cx) < (37.6 - y) * 0.8, PINK, a)
        elif face == "north":  # black behind with a pale spot
            a[:] = STRIPE
            a = np.where((np.abs(x - cx) < 1.1) & (y > 34.2) & (y < 36.0), WHITE, a)
    elif name.startswith("tail_"):
        k = int(name.split("_")[1])
        along = sum(L for _, L, _ in TAIL[:k]) + along  # from the base, down the tail (cat units)
        if k < 2:
            a = marks(a, rosettes(along, x if face in ("up", "down") else y, SPOT, True), np.ones(x.shape, bool))
        else:  # rings toward the tip: a ring 1.4 texels wide every 3.4
            a = np.where(((along / 3.0) % 1.0) < 0.4, STRIPE, a)
        if face == "down":
            a = np.where(a == COAT, WHITE, a)
        if k == len(TAIL) - 1 and face == "north":
            a[:] = STRIPE  # the black tip
    elif name.startswith(("leg_", "thigh_")):
        a = marks(a, rosettes(z if side else x, y, 3.4, True), np.ones(x.shape, bool))
        if face == "south" and name.startswith("leg_F"):
            a = np.where(y < 11, np.where(a == STRIPE, STRIPE, WHITE), a)
        if side and ((e["lo"][0] > 0) == (face == "west")):
            a = np.where(a == STRIPE, STRIPE, WHITE)  # the inside of the legs
        if "paw" in name:
            a = np.where(a == STRIPE, STRIPE, COAT)
    elif name.startswith("eyelid"):
        pass
    return a


def cut_mask(e, p):
    """The ear's cut-out panel: small and round, a few hairs on its edge."""
    x, y = p[..., 0], p[..., 1]
    cx = e["origin"][0]
    t = np.clip((y - 33.0) / 5.4, 0, 1)
    half = 4.2 * np.sqrt(np.clip(1 - t ** 2.4, 0, 1)) + 0.4 * ((np.floor(y * 1.5) % 3) == 0) * (t < 0.85)
    return (np.abs(x - cx) <= half) & (y >= 33)


EYE_DARK = (64, 34, 26)


# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}  # a layers.Recorder while build() paints, and the face's key


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)
def layer_props(e, face, p, a):
    """White where this jaguar is cream (the throat, belly, muzzle and inner legs) and up from below
    (a rare white-spotted one); the rosettes drawn by the genes (`rosettes`) over the back, flanks,
    legs and tail, smaller marks on the head, none on the muzzle and the ears' insides."""
    n, y = e["name"], p[..., 1]
    lo = unplace(np.array(e["lo"]), e["bone"])[1]
    hi = unplace(np.array(e["hi"]), e["bone"])[1]
    low = 1 - _ramp(y, lo, max(hi, lo + 1e-3))
    if n in ("chest", "hips", "neck"):
        white = 0.6 * low + 0.2 * (face == "down")
    elif n.startswith(("leg_", "thigh_")):
        white = 0.3 + 0.3 * low
    else:
        white = np.full(y.shape, 0.2)
    white = np.maximum(white * 0.85, np.where(a == WHITE, 0.9, 0))
    pattern = np.full(y.shape, 0.15 if n in ("muzzle", "jaw", "nose", "mouth") or n.startswith("ear_") else
                      (0.5 if n == "skull" else 1.0))
    return {"white": white, "pattern": pattern}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    if w == 0 or h == 0:
        return None
    size = np.array(e["hi"]) - np.array(e["lo"])
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel centers, in texels
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
    uu, vv = uu * su, vv * sv  # texel centers, in units across the face
    p = corner + uu[..., None] * du + vv[..., None] * dv
    along = (e["hi"][2] - p[..., 2]) / K - 1.2  # a tail segment's own length, from its base (cat units)
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    pc = unplace(p, e["bone"])  # the cat's units, for the color rules
    light_face = face
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "tail_")):
        light_face = "north"  # only seen when a pose folds or lifts it: not a near-black underside
    if e["name"].startswith("eyelid"):  # painted as it looks closed, on the face
        p = np.stack([p[..., 0], e["hi"][1] - vv, np.full(vv.shape, LID_Z)], -1)
        pc = unplace(p, e["bone"])
        light_face, du, dv, n = "south", np.array([1.0, 0, 0]), np.array([0, -1.0, 0]), np.array([0, 0, 1.0])
    a = area(e, face, pc, uu, vv, along)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    stacked = e["name"].startswith("ear_core")  # the ear's boxes, lit as one piece
    down = np.clip((38 - pc[..., 1]) / 5, 0, 1) if stacked else None
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=FUR, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=e["name"] not in ("skull",), down=down)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"] == "ear":
        img[..., 3] = np.where(cut_mask(e, pc), 255, 0)
    if e["name"] == "skull" and face == "south":
        stamp_eyes(img, [eye_texel(cx) for cx in (-EYE_X, EYE_X)], EYE_STYLE)
    if e["name"].startswith("eyelid"):
        img[-1, :, :3] = EYE_DARK
        if h >= 5:  # a 3-texel lid at 32: the lash line alone
            img[-2, :, :3] = (img[-2, :, :3] * 0.85).astype(np.uint8)
    if e["name"] == "muzzle" and face == "south":  # the mouth's line under the nose
        c0 = w // 2
        img[h - 2:h, c0 - 1:c0 + 1, :3] = (110, 66, 62)
        img[h - 1, max(0, c0 - 4):c0 + 4, :3] = (img[h - 1, max(0, c0 - 4):c0 + 4, :3] * 0.7).astype(np.uint8)
    if e["name"] == "jaw" and face == "up":
        img[..., :3] = np.array(BASE[PINK]) * 0.7
    if e["name"].endswith("_paw") and face == "south":  # the toes: three at 32 (a paw is 8 texels), four from 11
        n_toes = 4 if w >= 11 else 3
        for k in range(1, n_toes):
            c = int(round(w * k / n_toes - 0.5))
            img[:, c, :3] = (img[:, c, :3] * 0.7).astype(np.uint8)
    if e["name"].endswith("_paw") and face == "down":  # dark pads
        img[h // 2:, w // 4:3 * w // 4, :3] = (70, 52, 50)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, pc, a))
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
    q.update({"root": (0, 1.0 * max(0.0, S(TAU * p + 0.6)), 0), "spine_0": (-3 * c, 0, 0), "spine_1": (5 * c, 0, 0),
              "neck": (-5 * c, 0, 0), "ear_L": (15, 0, -8), "ear_R": (15, 0, 8)})
    q.update({"tail_0": tail_swing(0, 0, (26 + 4 * c, 0, 0)), "tail_1": tail_swing(1, 4 * c, (4, 0, 0)),
              "tail_2": tail_swing(2, 4 * c, (4, 0, 0)), "tail_3": tail_swing(3, 3 * c, (10, 0, 0))})  # streaming out level
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


def pose_roar(t):
    """The jaguar's call, a deep sawing grunt: the head up a little, the jaws opening on each rasp."""
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


BLINKS = {"idle": [1.2], "sit": [1.8], "lie": [1.0, 3.0], "roar": [0.4]}
BLINK = 0.09
# The behaviour clips (`mods/animals/src/anim.rs`), the canids' (`../_clips/canid.py`) on a cat's
# body: the same rig, and an ambusher's stalk is the one the creature AI calls (`hunt::chase`).
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "_clips"))
import canid  # noqa: E402

# A cat's body: its chest and hips poke at any sideways turn of the spine, its thick neck out of
# its chest past 85 degrees down, and its tail root out of the hips raised far (ground.pokes).
canid.bind(sys.modules[__name__], scratch=0.4, neck=0.7, bend=0.7, lift=0.7, wag=0.6, twist=0, neck_max=85)

ANIMS = {"idle": (pose_idle, 3.0), "walk": (pose_walk, 0.9), "run": (pose_run, 0.5), "sit": (pose_sit, 3.0),
         "lie": (pose_lie, 4.0), "roar": (pose_roar, 1.6), "swipe": (pose_swipe, 0.6), "bite": (pose_bite, 0.5)}
ANIMS.update(canid.anims())
LOOPS, BLENDS = canid.LOOPS, canid.BLENDS


def eyelids(name, t):
    a = -90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (a, 0, 0), "eyelid_R": (a, 0, 0)}


def frame_times(name, dur):
    n = 24 if name in ("roar", "swipe", "bite") else canid.FRAMES.get(name, 12 if dur < 1 else 16 if name in ("idle", "walk", "run", "sit", "lie") else 24)
    ts = {round(dur * k / n, 4) for k in range(n + 1)}
    for b in BLINKS.get(name, []):
        ts |= {round(b + d, 4) for d in (-BLINK, -BLINK / 2, 0, BLINK / 2, BLINK)}
    return sorted(ts)


def animations(bone_names):
    out = {}
    for name, (fn, dur) in ANIMS.items():
        frames = []
        loops = LOOPS.get(name, name in ("idle", "walk", "run", "sit", "lie"))
        for t in frame_times(name, dur):
            # A loop's last frame is its first; a clip played once ends where it ends.
            tt = t % dur if t < dur else (0.0 if loops is True else dur)
            pose = fn(tt)
            if name not in canid.SHUT:
                pose.update(eyelids(name, tt))
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames, "loop": loops, "blend": BLENDS.get(name, (0.25, 0.3))}
    return out


# ------------------------------------------------------------------ fitting poses to the ground

GROUPS = {"front paws": lambda n: n in ("leg_FL_paw", "leg_FR_paw"), "back paws": lambda n: n in ("leg_BL_paw", "leg_BR_paw"),
          "back legs": lambda n: n.startswith("thigh_") or n.startswith("leg_B") and not n.endswith("_paw"),
          "rump": lambda n: n == "hips", "chest": lambda n: n == "chest", "tail": lambda n: n.startswith("tail_"),
          "front legs": lambda n: n.startswith("leg_F") and not n.endswith("_paw")}


# Where the tail's segment ends should lie in a resting pose: on the ground, curling forward
# along the cat's right side toward its front paws.
# (In the cat's units, placed into the model's by `fit_tail`.)
TAIL_TARGETS = {"sit": [(-2.5, 1.7, -23.5), (-7, 1.7, -21.5), (-9.5, 1.7, -18), (-10, 1.7, -15)],
                "lie": [(-3, 1.7, -24), (-7.5, 1.7, -21.5), (-9.5, 1.7, -17.5), (-9.5, 1.7, -14)]}


def fit_tail(spec, pose_fn, key, P):
    """The tail's rotations (pitch and yaw per segment) that put each segment's end on its target
    without going below the ground: forward kinematics plus a small optimizer."""
    from scipy.optimize import minimize
    from ground import corners, world
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}
    pts = tail_points()
    els = {e["name"]: e for e in spec["elements"]}
    targets = np.array([place(t, "root") for t in TAIL_TARGETS[key]], float)
    pts = [place(pt, "tail_0") for pt in pts]

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
    grid = {"sit_angle": list(range(15, 61, 5)), "sit_hip": list(range(-110, -59, 5)), "sit_knee": list(range(110, 171, 5)),
            "sit_lean": [-24, -16, -8, 0, 8]}
    best = fit(grid, lambda P: sit_cost(dict(L, **P, sit_drop=0)))[1]
    P = dict(L, **best, sit_drop=0)
    L.update(best, sit_drop=round(min(v for k, v in low(pose_sit, P).items() if k != "tail"), 2))


    def lie_cost(P):
        g = low(pose_lie, P)
        g.pop("tail", None)
        floor = min(g.values())
        r = {k: v - floor for k, v in g.items()}
        return r["chest"] ** 2 + r["rump"] ** 2 + 0.5 * r["front paws"] ** 2 + 0.5 * r["back paws"] ** 2
    grid = {"lie_front": list(range(70, 121, 5)), "lie_hip": list(range(-110, -59, 10)), "lie_knee": list(range(110, 171, 10))}
    best = fit(grid, lambda P: lie_cost(dict(L, **P, lie_drop=0)))[1]
    P = dict(L, **best, lie_drop=0)
    L.update(best, lie_drop=round(min(v for k, v in low(pose_lie, P).items() if k != "tail"), 2))
    # Head down to the ground (sniffing, drinking) and the death fall (`canid.fit`).
    canid.fit(spec, dict(GROUPS, head=lambda n: n in ("skull", "muzzle", "jaw", "nose")))
    L["sit_tail"] = rest_tail(spec, pose_sit, "sit")
    L["lie_tail"] = rest_tail(spec, pose_lie, "lie")
    for key, fn in (("sit", pose_sit), ("lie", pose_lie)):
        g = low(fn, L)
        print(f"{key}, lowest:", {k: round(float(v), 2) for k, v in g.items()})


def rest_tail(spec, pose_fn, key):
    """A resting tail: laid back along the ground curving gently to one side, each segment a few
    degrees on the one before (a short tail can't wrap round the paws as a cat's does); the first
    segment lifted the least that keeps it all on the ground, not under it."""
    curl = [(0, 12, 0), (4, 9, 0), (4, 9, 0), (6, 8, 0)]
    floor = None
    for lift in range(-30, 121, 5):
        tail = [(lift + curl[0][0], curl[0][1], 0)] + curl[1:]
        g = lowest(spec, pose_fn(0.0, dict(L, **{f"{key}_tail": tail})), GROUPS)
        if floor is None:
            floor = min(v for k, v in g.items() if k != "tail")
        if g["tail"] - floor >= 0.3:
            return tail
    return tail


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
    spec = {"name": "jaguar", "texture": [sheet.shape[1], sheet.shape[0]],
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
