#!/usr/bin/env python3
"""The caiman: one model for the yacare caiman of the Pantanal and the spectacled caiman of the
llanos, the water's hunter of the swamp and the savanna (`notes/ECOLOGY.md`, tier 2), made the way
Hytale makes creatures (boxes and a hand-painted texture) with the blocky-model skill, from research
alone (README.md), on the lizard's build (`../lizard/lizard.py`): laid out in the lizard's units with
a crocodilian's proportions (a long flat snout and a jaw that opens, a broad flat armored body, a
long tall tail, short sprawled legs), then drawn `K` times bigger. Its colors are rules (`area`).

    ~/.venvs/blocky/bin/python caiman.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each), y up, it faces +z. A yacare caiman is about 2.2 m
long, half of it tail: here 27.4 lizard units from the snout to the tail's tip, times K = 5.2: 142
units. Its eyes and nostrils sit high on the head, so it can lie in the water with only them out.
Painted at the house density, 32 texels a block (`TX` texels a unit), as the blocks and items are.
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

# TX: texels a unit, the house's 32 a block (0.5). The eye is on the skull's side, 11 units by 6:
# 6 texels by 3 at 32, at least `house.MIN_FACE` both ways, so no finer density.

OUT = os.environ.get("CAIMAN_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin
K = 5.2  # the lizard's units drawn this much bigger

# ------------------------------------------------------------------ materials and eyes

SKIN, BACK, BELLY, PALE, MOUTH, CLAW, TOOTH = range(7)
# SKIN the olive grey-brown, BACK the dark bands across the back, the tail and the jaws, BELLY the
# pale yellowish underside, PALE the yellower flanks, MOUTH the pink inside, CLAW the claws and
# the dark of the nostrils, TOOTH its teeth along the jaws.
BASE = [(98, 100, 76), (52, 54, 44), (216, 208, 166), (148, 142, 100), (196, 130, 112), (54, 46, 40), (232, 226, 204)]
SHADOW = [0.06, 0.06, 0.09, 0.08, 0.97, 0.74, 0.08]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (SKIN, BACK, BELLY, PALE)
ROLES = [("coat", 0, 0), ("coat", 0, -1.6), ("coat", 1, 0), ("coat", 0.6, 0.6), "fixed", "fixed", "fixed"]

# A crocodilian's eye: olive-gold round a tall slit pupil, a catchlight, the dark of the lids round it.
# "caiman" is the old one at 64 a block (for comparison); at 32 the skull's side is 6 texels by 3
# and the eye 3 by 2: the slit pupil down its middle, gold either side.
EYES.update({
    "caiman": dict(pattern=["GHG",
                            "GPG",
                            "gPg"], colors={"G": (168, 150, 64), "g": (196, 176, 84), "P": (20, 18, 14), "H": (236, 232, 210)},
                   halo="round", iris="Gg"),
    "caiman_px": dict(pattern=["GPG", "gPg"], colors={"G": (168, 150, 64), "g": (196, 176, 84), "P": (20, 18, 14)},
                      halo="none", iris="Gg"),
    "caiman_px_lit": dict(pattern=["HPG", "gPg"], colors={"G": (168, 150, 64), "g": (196, 176, 84), "P": (20, 18, 14),
                                                          "H": (236, 232, 210)}, halo="none", iris="Gg"),
    "caiman_px2": dict(pattern=["GP", "gP"], colors={"G": (168, 150, 64), "g": (196, 176, 84), "P": (20, 18, 14)},
                       halo="none", iris="Gg"),
})
EYE_STYLE = os.environ.get("CAIMAN_EYES", "caiman_px")

# In the lizard's space (scaled by K when the model is built).
K_ = 5.2
# Whole texels once scaled: the eyelids line up with the eye. 8 tall (4 texels; 6 before), its top where it was:
# room for a snout and a jaw two texels tall each (rule 7, Ronald 11-05: no parts a texel thick).
SKULL0 = ((-1.45, 0.85 + 6 / K_ - 8 / K_, 2.8), (1.45, 0.85 + 6 / K_, 2.8 + 11 / K_))
EYE_Z0, EYE_Y0 = 3.75, 1.7
SKULL = tuple(tuple(v * K for v in c) for c in SKULL0)
EYE_Z, EYE_Y = EYE_Z0 * K, EYE_Y0 * K
LID_X = SKULL[1][0] + 0.2  # the eyelids' hinge plane, just outside the head's side
LASH = (40, 40, 32)


def skull_side():
    """The skull side's texels: (width, height) and one texel's size in units (u along z, v along y)."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h = max(1, round(TX * (z1 - z0))), max(1, round(TX * (y1 - y0)))
    return w, h, (z1 - z0) / w, (y1 - y0) / h


def eye_pattern(side):
    """The eye's design on one side: mirrored on the west face (its texels run back to front)."""
    pat = EYES[EYE_STYLE]["pattern"]
    return pat if side > 0 else [row[::-1] for row in pat]


def eye_center(side):
    """The eye's center on the skull's east (side > 0) or west face, in that face's texels."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_side()
    return ((z1 - EYE_Z) if side > 0 else (EYE_Z - z0)) / su, (y1 - EYE_Y) / sv


def eye_box(side):
    """(z0, z1, top, bottom) of the eye on one side (scaled), rounded as `stamp_eyes` places it."""
    pat = eye_pattern(side)
    ph, pw = len(pat), len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_side()
    u0, v0 = eye_corner(*eye_center(side), pat, w)
    za, zb = (z1 - (u0 + pw) * su, z1 - u0 * su) if side > 0 else (z0 + u0 * su, z0 + (u0 + pw) * su)
    return za, zb, y1 - v0 * sv, y1 - (v0 + ph) * sv

# ------------------------------------------------------------------ the model


def el(name, bone, lo, hi, quad=None, rot=None, origin=None, cut=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad, rot=rot,
                origin=[float(v) for v in origin] if origin else None, cut=cut)


LEGS = ("FL", "FR", "BL", "BR")
# Each leg (the lizard's, set out to the caiman's broader body): the shoulder or hip, the elbow or
# knee out to the side, the wrist or ankle; the upper leg runs out sideways, the lower one down.
# Rule 7 and "No thin lines" (Ronald, 11-05): the lower legs were rods 0.8 texel deep (front 1.8 x 1.6
# x 0.8 texels, hind 2.3 x 2.3 x 0.8), their sides and the feet's tops a texel or so out from each other.
# Now each lower leg is the elbow or knee coming down at the upper leg's end: as deep as the upper leg
# (0.07 inside its front and back: flush; 2.2 and 2.5 texels deep), its top 0.07 under the upper's, its
# inner and outer sides 0.07 inside the foot's, so it shows as a column two texels across (0.78 past
# the upper's end) and 2.2 tall (above the foot). The feet's heels flush behind it (0.105 out, and
# 0.035 behind the upper's back, which they'd share a plane with as the leg lifts: boxcheck), their
# toes 2.1 texels in front (were 1.2-1.4 texels out on every side, a thin line round the foot's top).
# The hind one's bottom 0.47, inside the foot, so it doesn't meet the chest's bottom plane in the run.
LEG = {"F": dict(hip=(1.6, 1.1, 1.2), knee=(2.75, 0.95, 1.2), ankle=(2.75, 0.55, 1.2), upper=(1.4, 2.8, 0.75, 1.55, 0.7, 1.7),
                 lower=(2.12, 3.58, 0.5, 1.48, 0.77, 1.63), foot=(2.05, 3.65, 0.15, 0.65, 0.665, 2.45)),
       "B": dict(hip=(1.35, 1.35, -1.9), knee=(2.85, 1.3, -1.9), ankle=(2.85, 0.55, -1.9), upper=(1.15, 3.0, 0.95, 1.75, -2.45, -1.35),
                 lower=(2.12, 3.78, 0.47, 1.68, -2.38, -1.42), foot=(2.05, 3.85, 0.0, 0.85, -2.485, -0.6))}
# The upper legs 0.8 tall, two texels (0.6 and 0.5 before, a texel: rule 7, Ronald 11-05).
# The tail: four segments, tall and flattened from the side (it swims with it), each smaller, each nested 0.06
# inside the one before (boxcheck, pokes) and all as tall as the hips allow, so the last two are two texels
# tall and the tip two wide (rule 7, Ronald 11-05: the tip was a texel each way).
TAIL = [((0, 1.3, -2.9), (1.15, 0.76, 1.84, -6.4, -2.0)), ((0, 1.3, -6.0), (0.8, 0.82, 1.78, -10.4, -4.8)),
        ((0, 1.3, -10.0), (0.5, 0.88, 1.72, -14.5, -8.8)), ((0, 1.3, -14.1), (0.41, 0.94, 1.66, -18.0, -13.7))]  # each reaching deep into the one before (the tip less: wider now, it would poke out of tail_2 as it swings)
JAW0 = (0, 1.1, 3.3)  # the jaw's hinge, at the back of the head


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 1.3, -1.0)), ("spine_1", "spine_0", (0, 1.3, -0.6)),
         ("neck", "spine_1", (0, 1.4, 2.0)), ("head", "neck", (0, 1.4, 3.0)), ("jaw", "head", JAW0)]
    for k, (pv, _) in enumerate(TAIL):
        B.append((f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", pv))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        g = LEG[leg[0]]
        parent = "spine_1" if leg[0] == "F" else "spine_0"
        mx = lambda p: (s * p[0], p[1], p[2])  # noqa: E731
        B += [(f"leg_{leg}_upper", parent, mx(g["hip"])), (f"leg_{leg}_lower", f"leg_{leg}_upper", mx(g["knee"])),
              (f"leg_{leg}_paw", f"leg_{leg}_lower", mx(g["ankle"]))]
    B = [(n, p, tuple(v * K for v in pv)) for n, p, pv in B]
    for s, n in ((1, "L"), (-1, "R")):  # the eyelids, on the scaled skull with the eye's rounding
        za, zb, top, _ = eye_box(s)
        B.append((f"eyelid_{n}", "head", (s * LID_X, top + 0.25, (za + zb) / 2)))
    return B


def elements():
    E = [el("hips", "spine_0", (-1.6, 0.7, -3.2), (1.6, 1.9, -0.4)),
         el("chest", "spine_1", (-2.0, 0.38, -0.8), (2.0, 2.2, 2.5)),  # broad and flat; its belly 0.07 lower, clear of the deeper skull's in the bite (boxcheck)
         el("neck", "neck", (-1.15, 1.15, 0.9), (1.15, 1.65, 3.9)),  # deep in the chest and the skull: it turns on them
         el("skull", "head", *SKULL0),
         # Long and flat, two texels tall (rule 7: one before), its top 0.07 under the skull's; to 9.4, where the
         # nose box ended (rule 4, Ronald 11-05: the nostrils painted on its tip, no box).
         el("snout", "head", (-1.05, 1.186, 4.6), (1.05, 1.934, 9.4)),
         el("mouth", "head", (-0.88, 1.1, 4.0), (0.88, 1.26, 8.93)),  # the pink inside, seen when it gapes; its sides and front 0.07 in from the jaw's (linecheck)
         # Shorter than the snout: an overbite; two texels tall (one before), its bottom 0.07 under the skull's.
         el("jaw", "jaw", (-0.95, 0.396, 3.0), (0.95, 1.166, 9.0))]
    for k, (_, (hw, y0, y1, z0, z1)) in enumerate(TAIL):
        # Straight, each a size smaller than the one before (no two faces share a plane).
        E.append(el(f"tail_{k}", f"tail_{k}", (-hw, y0, z0), (hw, y1, z1)))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        g = LEG[leg[0]]
        for part, key in (("upper", "upper"), ("lower", "lower"), ("paw", "foot")):
            x0, x1, y0, y1, z0, z1 = g[key]
            xa, xb = sorted((s * x0, s * x1))
            E.append(el(f"leg_{leg}_{part}", f"leg_{leg}_{part}", (xa, y0, z0), (xb, y1, z1)))
    for e in E:  # the lizard's space, drawn K times bigger
        e["lo"], e["hi"] = [v * K for v in e["lo"]], [v * K for v in e["hi"]]
        if e["origin"]:
            e["origin"] = [v * K for v in e["origin"]]
    for s, n in ((1, "L"), (-1, "R")):
        # The eyelid: a quad the eye's size lying flat inside the head, hinged at the eye's top; a
        # blink swings it down over the eye. Tilted a few degrees about its hinge.
        za, zb, top, bottom = eye_box(s)
        lx0, lx1 = (LID_X - (top - bottom), LID_X) if s > 0 else (-LID_X, -LID_X + (top - bottom))
        # A quarter unit bigger all round than the eye, so no edge of it shows past a closed lid.
        za, zb, lx0, lx1 = za - 0.25, zb + 0.25, (lx0 - 0.25 if s > 0 else lx0), (lx1 if s > 0 else lx1 + 0.25)
        top, lx0, lx1 = top + 0.25, (lx0 - 0.25 if s > 0 else lx0), (lx1 if s > 0 else lx1 + 0.25)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down", rot=(0, 0, s * 4),
                    origin=(s * LID_X, top, (za + zb) / 2)))
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
    return np.array(corner, float), du, dv, max(1, int(round(TX * abs(size @ du)))), max(1, int(round(TX * abs(size @ dv)))), n


def area(e, face, p, uu, vv, h):
    """A yacare caiman, in the lizard's space (`p` / K): olive grey-brown, rows of dark armor
    plates down the back, dark bands round the tail, the flanks yellower, pale yellow below; dark
    bars along the jaws, the inside of the mouth pink."""
    name = e["name"]
    p = p / K
    U = 1 / (TX * K)  # a texel, in the lizard's units (0.38): plates, bars and bands at least 1 by 2 of them
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, SKIN)
    side = face in ("east", "west")
    if name in ("hips", "chest", "neck"):
        if face == "up":
            plates = (np.floor(z / (2 * U)) % 2 == 0) & (np.floor(np.abs(x) / U) % 2 == 0)  # the osteoderms in rows: a plate 1 texel across, 2 long
            a = np.where(plates, BACK, SKIN)
            a = np.where(np.abs(x) > 1.55, PALE, a)
        elif face == "down":
            a[:] = BELLY
        elif side:
            a = np.where(y < 0.95, BELLY, np.where(y < 1.35, PALE, SKIN))
    elif name.startswith("tail_"):
        a = np.where(np.mod(-z, 2.4) < 0.9, BACK, SKIN)  # the dark bands
        if face == "down":
            a[:] = BELLY
        elif side:
            a = np.where(y < 1.0, np.where(a == BACK, BACK, PALE), a)
    elif name in ("skull", "snout", "nose"):
        if face == "down":
            a[:] = BELLY
        elif face == "up" and name == "skull":
            a = np.where(np.abs(z - 4.5) < max(0.18, U / 2), BACK, SKIN)  # the bony 'spectacles' ridge before the eyes
        elif side and name == "snout" and h >= 3:
            a = np.where(np.mod(z, 1.4) < 0.45, BACK, SKIN)  # the dark bars along the jaws
            # (at 32 the snout's side is one texel tall: it carries the teeth, the jaw the bars)
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else BELLY
        if side:
            a = np.where((np.mod(z, 1.4) < 0.45) & (y > 0.95), BACK, PALE)
    elif name == "mouth":
        a[:] = MOUTH
    elif name.startswith("eyelid"):
        a[:] = SKIN
    elif name.startswith("leg_"):
        a = np.where((np.floor(z / (2 * U)) % 2 == 0) & (face == "up"), BACK, SKIN)  # bands 2 texels wide
        if face == "down":
            a[:] = BELLY
    return a


# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def layer_props(e, face, p, a):
    """White where it is pale below; the dark plates and bands are its points (darker or fainter by
    its genes); marks over the back and the tail."""
    white = np.where(a == BELLY, 0.85, 0.1)
    points = np.where(a == BACK, 1.0, 0.0)
    pattern = np.full(p.shape[:2], 1.0 if e["name"] in ("hips", "chest") or e["name"].startswith("tail_") else 0.3)
    return {"white": white, "points": points, "pattern": pattern}


def rotm(rot):
    if not rot:
        return np.eye(3)
    rx, ry, rz = (math.radians(a) for a in rot)
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    size = np.array(e["hi"]) - np.array(e["lo"])
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
    p = corner + uu[..., None] * du * (abs(size @ du) / w) + vv[..., None] * dv * (abs(size @ dv) / h)
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    light_face = face
    if face == "down" and e["name"].startswith(("leg_", "tail", "jaw")):
        light_face = "north"
    if e["name"].startswith("eyelid"):
        light_face = "east"  # seen only closed, as the side of the head
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=None, occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=False, merge=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["name"] == "skull" and face in ("east", "west"):
        side = 1 if face == "east" else -1
        EYES["_caiman_side"] = dict(EYES[EYE_STYLE], pattern=eye_pattern(side))
        stamp_eyes(img, [eye_center(side)], "_caiman_side")
    if e["name"] in ("snout", "nose") and face in ("east", "west"):  # teeth along the lip, every few texels
        t = np.arange(w) % 3 == 1
        if h >= 3:
            img[h - 1, t, :3] = BASE[TOOTH]
        else:  # one texel tall at 32: a whole tooth-white texel reads as a picket fence, so a softer one
            img[h - 1, t, :3] = (img[h - 1, t, :3] * 0.35 + np.array(BASE[TOOTH]) * 0.65).astype(np.uint8)
    if e["name"] == "jaw" and face in ("east", "west") and h >= 3:  # (one texel tall at 32: its bars, no teeth)
        cols = np.arange(w)
        img[0, cols % 3 == 2, :3] = BASE[TOOTH]
    if e["name"] == "snout" and face == "up":  # the nostrils: two slits on its tip (rule 4: painted, no nose box)
        if h >= 6:
            img[h - 3, [w // 2 - 2, w // 2 + 1], :3] = BASE[CLAW]
        else:  # 4 texels by 4 at 32: the two side by side, toward the front
            img[h - 2, [w // 2 - 1, w // 2], :3] = BASE[CLAW]
    if e["name"].startswith("eyelid"):  # closed, the lash line is its lower edge
        c = 0 if e["name"].endswith("L") else w - 1
        if w >= 3:
            img[:, c, :3] = LASH
        else:  # a 2-texel lid: a full dark lash would cover half the eye
            img[:, c, :3] = (img[:, c, :3] * 0.5 + np.array(LASH) * 0.5).astype(np.uint8)
    if e["name"].endswith("_paw") and face == "south":  # the claws
        img[:, ::2, :3] = (img[:, ::2, :3] * 0.7).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"bask_drop": 2.0, "bask_splay": 20}


def gait(p, phases, swing_amp, lift):
    """The high walk, sprawling: each upper leg swings forward and back about the vertical, lifted
    (raised out to the side) while it swings forward; the lower leg kept upright, the foot flat."""
    out = {}
    for leg, ph in zip(LEGS, phases):
        s = 1 if leg[1] == "L" else -1
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))
        raise_ = s * lift * up
        out[f"leg_{leg}_upper"] = (0, -s * swing_amp * S(a), raise_)
        out[f"leg_{leg}_lower"] = (0, 0, -raise_)
    return out


def body_wave(p, amp):
    """The body's S-curve walking, the tail following."""
    w = S(TAU * p)
    q = {"spine_0": (0, amp * w, 0), "spine_1": (0, -amp * w, 0), "neck": (0, 0.5 * amp * w, 0)}
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = (0, -amp * (0.5 + 0.12 * k) * S(TAU * p - 0.4 * (k + 1)), 0)  # each a few degrees on the one before
    return q


def pose_idle(t):
    """Still in the sun, the jaws slowly gaping to cool and closing, the head lifting a little, a
    slow sweep of the tail (8 s)."""
    p = t / 8.0
    gape = float(np.interp(p, [0, .15, .3, .65, .8, 1], [0, 0, 1, 1, 0, 0]))
    sweep = 5 * S(TAU * p)
    # The head lifts as the jaw drops, so the lower jaw stays clear of the ground.
    q = {"root": (0, 0.03 * S(TAU * p * 2), 0), "head": (-8 * gape, 3 * S(TAU * p), 0), "jaw": (12 * gape, 0, 0)}
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = (0, sweep * (0.5 + 0.3 * k), 0)
    return q


def pose_walk(t):
    """The high walk: the belly off the ground, the legs under it, the tail dragging and swaying (1.2 s)."""
    p = t / 1.2
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 22, 18)
    q.update(body_wave(p, 5))
    return q


def pose_run(t):
    """A rush, the body low and whipping side to side (0.5 s)."""
    p = t / 0.5
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 30, 22)
    q.update(body_wave(p, 8))
    q["head"] = (-3, 0, 0)
    return q


def pose_bite(t):
    """A bite: the jaws opening wide, a lunge with the head snapping sideways, the jaws slamming
    shut (0.6 s)."""
    p = t / 0.6
    open_ = float(np.interp(p, [0, .25, .45, .52, 1], [0, 1, 1, 0, 0]))
    lunge = float(np.interp(p, [0, .3, .5, .7, 1], [0, -0.4, 1, 1, 0]))
    q = pose_idle(0)
    q.update({"jaw": (40 * open_, 0, 0), "head": (-6 * open_, 14 * lunge, 0), "neck": (0, 8 * lunge, 0), "spine_1": (0, 6 * lunge, 0)})
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = (0, -8 * lunge * (0.5 + 0.3 * k), 0)
    return q


def pose_bask(t, P=L):
    """Flat on the bank, the legs splayed, the jaws agape in the sun (6 s)."""
    p = t / 6.0
    # The head up 12.5 (9 before the jaw grew two texels tall: the jaw's tip clear of the bank as it was).
    q = {"root": (0, -P["bask_drop"] + 0.1 * S(TAU * p), 0), "head": (-12.5, 2 * S(TAU * p), 0), "jaw": (13, 0, 0)}
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        sp = s * P["bask_splay"]
        q[f"leg_{leg}_upper"] = (0, -s * (14 if leg[0] == "F" else -14), sp)
        q[f"leg_{leg}_lower"] = (0, 0, -sp)
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = (0, 4 + 2 * k, 0)  # curving gently to one side
    return q


# ------------------------------------------------------------------ behaviour clips (`anim.rs`)
# Each a whole-body pose over what plays below on the bones it moves (`to_bedrock.as_action`): held
# states and the gaits loop, the rest play once. An ambusher: it stalks (a belly-low creep) to its
# burst range and pounces as it breaks cover (`hunt::chase`). Its height in the gaits, the alarm and
# the pounce is set from its feet in the build (`ground.grounded`), the death's from its lowest point.
# Fitted in `fit_poses`: the jaw's dip to drink, the snout's to sniff, the head laid down asleep, the
# stalk's splay (the belly just off the ground).
L.update({"drink_head": 0.0, "sniff_head": 0.0, "sleep_head": 0.0, "stalk_splay": 0.0})
DRINK_JAW, SLEEP_JAW = 10, 7  # the jaw a little open, drinking; asleep a little agape, as basking


def tails(yaws, pitch=0.0):
    """The tail's segments swung by `yaws` (each on the one before), the first raised by `pitch`."""
    return {f"tail_{k}": ((pitch if k == 0 else 0), y, 0) for k, y in enumerate(yaws)}


def still(t, period=4.0, tail=2.0):
    """Lying still on its legs, breathing, the tail swaying a little: what a clip builds on."""
    br = S(TAU * t / period)
    return {"root": (0, 0.08 * br, 0), **tails([tail * br * (0.5 + 0.3 * k) for k in range(len(TAIL))])}


def legs_up(lift, legs=LEGS):
    """The legs straightened under it by `lift` degrees: each knee or elbow lowered, the lower leg kept
    upright, so the body rises off the ground (the high stance; set on its feet in the build)."""
    out = {}
    for leg in legs:
        s = 1 if leg[1] == "L" else -1
        out[f"leg_{leg}_upper"] = (0, 0, -s * lift)
        out[f"leg_{leg}_lower"] = (0, 0, s * lift)
    return out


def pose_stare(t):
    """Half aware: frozen, the head raised a little and still, only the breath moving (2 s, held)."""
    q = still(t, 2.0, tail=0.4)
    q.update({"neck": (-3, 0, 0), "head": (-6, 0, 0), "jaw": (0.5, 0, 0)})
    return q


def pose_watch(t):
    """On guard: the head up, its eyes above the level, turning slowly one way and the other (4 s, held)."""
    p = t / 4.0
    yaw = 24 * S(TAU * p)
    q = still(t, 4.0, tail=1.0)
    q.update({"neck": (-6, 0.55 * yaw, 0), "head": (-10, 0.45 * yaw, 0), "jaw": (0.5, 0, 0)})
    return q


def pose_drink(t, P=L):
    """Drinking: the head tipped down, the lower jaw dipped to the water, lifting a little to swallow
    (1.5 s, held)."""
    sip = max(0.0, S(TAU * t / 1.5))
    q = still(t, 1.5, tail=0.5)
    q.update({"neck": (0.01, 0, 0), "head": (P["drink_head"] - 2 * sip, 0, 0), "jaw": (DRINK_JAW + sip, 0, 0)})
    return q


def pose_sleep(t, P=L):
    """Asleep on its belly as basking: the legs splayed, the chin laid low, the jaws a little agape,
    eyes shut, a slow breath (6 s, held)."""
    q = pose_bask(0.0, P)
    br = S(TAU * t / 6.0)
    q.update({"root": (0, -P["bask_drop"] + 0.06 * br, 0), "head": (P["sleep_head"], 0.01, 0), "jaw": (SLEEP_JAW, 0, 0),
              "spine_1": (0.4 * br, 0, 0), "eyelid_L": (0, 0, 90), "eyelid_R": (0, 0, -90)})
    return q


def stalk_legs(p, sp, swing, lift, phases=(0.0, 0.5, 0.5, 0.0)):
    """The legs sprawled out by `sp` (the body sinks between them) and stepping: the high walk's
    swing and lift on top of the splay."""
    q = gait(p, phases, swing, lift)
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        u, w = q[f"leg_{leg}_upper"], q[f"leg_{leg}_lower"]
        q[f"leg_{leg}_upper"] = (u[0], u[1], u[2] + s * sp)
        q[f"leg_{leg}_lower"] = (w[0], w[1], w[2] - s * sp)
    return q


def pose_stalk(t, P=L):
    """Stalking: the belly a hand off the ground, the legs sprawled, slow creeping steps, the head low
    and level and pointed ahead, the tail straight and barely moving (2 s a cycle, in place of the walk)."""
    p = t / 2.0
    q = stalk_legs(p, P["stalk_splay"], 14, 10)
    q.update(body_wave(p, 2.5))
    q.update({"neck": (1, -1.25 * S(TAU * p), 0), "head": (-1, 0.01, 0), "jaw": (0.01, 0, 0)})
    return q


def pose_approach(t):
    """Curious, coming to look: the high walk slowed, the head raised a little and turning one way
    and the other (1.6 s a cycle, in place of the walk)."""
    p = t / 1.6
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 18, 17)
    q.update(body_wave(p, 3))
    q.update({"neck": (-3, 4 * S(TAU * p / 2) - 1.5 * S(TAU * p), 0), "head": (-5, 6 * S(TAU * p / 2), 0), "jaw": (0.5, 0, 0)})
    return q


def pose_pounce(t):
    """Breaking cover: a moment gathered, the tail cocked to one side, then a lunge forward on
    straightened legs, the jaws flung open and snapped shut, the tail sweeping hard to drive it (0.8 s, once)."""
    p = t / 0.8
    crouch = float(np.interp(p, [0, .2, .32, 1], [0, 1, 0, 0]))
    lunge = float(np.interp(p, [0, .2, .38, .6, 1], [0, 0, 1, 1, 0]))
    open_ = float(np.interp(p, [0, .22, .36, .5, .56, 1], [0, 0, 1, 1, 0, 0]))
    sweep = float(np.interp(p, [0, .2, .4, .55, .8, 1], [1, 1, -1, -0.4, 0, 0]))
    q = legs_up(26 * lunge)
    for leg in LEGS:  # the feet pushing back as the body goes forward
        s = 1 if leg[1] == "L" else -1
        u = q[f"leg_{leg}_upper"]
        q[f"leg_{leg}_upper"] = (u[0], s * (16 if leg[0] == "B" else 10) * lunge, u[2])
    q.update({"root": (0, 0, 5 * lunge), "spine_0": (-4 * lunge, 0, 0), "spine_1": (-2 * lunge, 0, 0),
              "neck": (-3 * lunge + 2 * crouch, 0, 0), "head": (2 * crouch - 14 * open_, 0.01, 0), "jaw": (34 * open_, 0, 0)})
    q.update(tails([(4 + 2.5 * k) * sweep * max(crouch, lunge, 0.4 if p > .55 else 0) for k in range(len(TAIL))]))
    return q


def pose_alarm(t):
    """A hiss: up on straight legs, the head raised and the jaws wide, the body swelling with each
    hiss, the tail curled round to one side (1.4 s, once)."""
    p = t / 1.4
    up = float(np.interp(p, [0, .12, .8, 1], [0, 1, 1, 0]))
    hiss = float(np.interp(p, [0, .12, .3, .45, .6, .8, 1], [0, 1, 0.8, 1, 0.8, 1, 0]))
    q = legs_up(30 * up)
    q.update({"spine_1": (-2 * up, 0, 0), "neck": (-6 * up, 0, 0), "head": (-18 * up, 0.01, 0), "jaw": (30 * hiss * up, 0, 0)})
    q.update(tails([(5 + 3 * k) * up for k in range(len(TAIL))], pitch=-3 * up))
    return q


def pose_sniff(t, P=L):
    """The snout down near the ground, swinging slowly side to side, and up (2.2 s, once)."""
    p = t / 2.2
    d = float(np.interp(p, [0, .2, .8, 1], [0, 1, 1, 0]))
    sw = d * S(TAU * p * 2)
    q = still(t, 2.2, tail=1.0)
    q.update({"neck": (0.01, 8 * sw, 0), "head": (P["sniff_head"] * d, 12 * sw, 0), "jaw": (0.01, 0, 0)})
    return q


def pose_look(t):
    """The head turning round one way, then the other, and back (3 s, once)."""
    p = t / 3.0
    yaw = float(np.interp(p, [0, .15, .4, .55, .8, 1], [0, 34, 34, -30, -30, 0]))
    up = float(np.interp(p, [0, .12, .88, 1], [0, 1, 1, 0]))
    q = still(t, 3.0, tail=1.0)
    q.update({"spine_1": (0, 0.15 * yaw, 0), "neck": (-2 * up, 0.4 * yaw, 0), "head": (-4 * up, 0.45 * yaw, 0), "jaw": (0.01, 0, 0)})
    return q


def pose_scratch(t):
    """An itch: the left hind foot raised and scratching at the flank in quick strokes, the body bent
    round toward it (2 s, once). (A caiman's hind foot can't reach its jaw: the flank.)"""
    p = t / 2.0
    d = float(np.interp(p, [0, .15, .85, 1], [0, 1, 1, 0]))
    stroke = d * S(TAU * p * 6)
    q = still(t, 2.0, tail=0.0)
    # (the foot clear of the ground through every stroke: 0.6 to 1.4 over it)
    q.update({"leg_BL_upper": (0, -(40 + 8 * stroke) * d, 34 * d), "leg_BL_lower": (0, 0, -30 * d), "leg_BL_paw": (0, 0, 8 * stroke)})
    q.update({"spine_1": (0, 5 * d, 0), "neck": (0, 10 * d, 0), "head": (0, 12 * d, 0), "jaw": (0.01, 0, 0)})
    q.update(tails([-(3 + 1.5 * k) * d for k in range(len(TAIL))]))
    return q


def pose_stretch(t):
    """Waking: a long gape with the head raised, then the front legs reached forward, the hind ones
    back and the tail arched and flexed side to side (2.4 s, once)."""
    p = t / 2.4
    a = float(np.interp(p, [0, .15, .42, .55, 1], [0, 1, 1, 0, 0]))
    b = float(np.interp(p, [0, .45, .6, .88, 1], [0, 0, 1, 1, 0]))
    flex = b * S(TAU * (p - .45) * 2)
    q = still(t, 2.4, tail=0.0)
    q.update({"neck": (-7 * a, 0, 0), "head": (-20 * a, 0.01, 0), "jaw": (30 * a, 0, 0)})
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        q[f"leg_{leg}_upper"] = (0, -s * (22 if leg[0] == "F" else -18) * b, 0)
    q.update(tails([(3 + 2 * k) * flex for k in range(len(TAIL))], pitch=4 * b))
    return q


def pose_play(t):
    """A young one's play: the tail swished hard side to side, the head tossed, the jaws snapping (1.2 s, once)."""
    p = t / 1.2
    d = float(np.interp(p, [0, .1, .9, 1], [0, 1, 1, 0]))
    sw = d * S(TAU * p * 2)
    snap = d * max(0.0, S(TAU * p * 4))
    q = still(t, 1.2, tail=0.0)
    q.update({"spine_0": (0, -3 * sw, 0), "spine_1": (0, 4 * sw, 0), "neck": (-3 * d, 6 * sw, 0), "head": (-10 * d, 10 * sw, 0),
              "jaw": (14 * snap, 0, 0)})
    q.update(tails([(5 + 3 * k) * d * S(TAU * p * 2 - 0.5 * (k + 1)) for k in range(len(TAIL))]))
    return q


def pose_groom(t):
    """Grooming: the body bent round, the snout rubbing along the right flank, the jaws working a
    little (2.5 s, once)."""
    p = t / 2.5
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    rub = d * S(TAU * p * 3)
    q = still(t, 2.5, tail=0.0)
    q.update({"spine_1": (0, -7 * d, 0), "neck": (0.01, -12 * d, 0), "head": (-5 * d, -14 * d + 4 * rub, 0),
              "jaw": (4 * d * max(0.0, S(TAU * p * 6)), 0, 0)})
    q.update(tails([(2 + 1.5 * k) * d for k in range(len(TAIL))]))
    return q


def pose_nuzzle(t, P=L):
    """Nudging a young one: the snout low and forward, gentle pushes up with it (2 s, once)."""
    p = t / 2.0
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    push = d * max(0.0, S(TAU * p * 2))
    q = still(t, 2.0, tail=0.8)
    q.update({"neck": (0.01, 0, 0), "head": (P["sniff_head"] * 0.7 * d - 7 * push, 0.01, 4 * d), "jaw": (0.01, 0, 0)})
    return q


def pose_flinch(t):
    """Hit: a jerk aside, the head up and back, the jaws starting open, the tail lashing (0.45 s, once)."""
    p = t / 0.45
    d = float(np.interp(p, [0, .15, .45, 1], [0, 1, 0.8, 0]))
    q = still(t, 0.45, tail=0.0)
    q.update({"spine_1": (0, 5 * d, 0), "neck": (-4 * d, 7 * d, 0), "head": (-10 * d, 9 * d, 0), "jaw": (10 * d, 0, 0)})
    q.update(tails([-(4 + 2.5 * k) * d for k in range(len(TAIL))]))
    return q


def pose_death(t):
    """Dying: the legs give and fold in under it, it rolls over onto its back, the jaws fall open, the
    eyes shut; it stays (1.6 s, holds). Its height is set on its lowest point in the build."""
    p = t / 1.6
    fall = float(np.interp(p, [0, .25, .7, 1], [0, 0.4, 1, 1]))
    roll = float(np.interp(p, [0, .2, .8, 1], [0, 0, 1, 1]))
    # (the jaws fall open as it goes over, then slack shut again under their weight; the head lolls
    # back to the ground)
    gape = float(np.interp(p, [0, .25, .55, .85, 1], [0, 0.6, 1, 0.3, 0.3]))
    q = {"root": (-12 * roll, 0, 0), "spine_0": (0, 0, 180 * roll), "spine_1": (0, 0, 0.01), "neck": (-3 * roll, 0, 0),
         "head": (4 * fall - 10 * roll, 0.01, 0), "jaw": (18 * gape, 0, 0)}
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        q[f"leg_{leg}_upper"] = (0, -s * (10 if leg[0] == "F" else -10) * fall, -s * 40 * fall)
        q[f"leg_{leg}_lower"] = (0, 0, -s * 30 * fall)
    q.update(tails([(2 + 1.5 * k) * fall for k in range(len(TAIL))]))
    # The lids shut to 84, their free edges tipped in to the skull: on its back, seen from above, a lid
    # at 90 (0.2 out from the skull's side) let a strip of the eye show past its free edge.
    q.update({"eyelid_L": (0, 0, 84 * fall), "eyelid_R": (0, 0, -84 * fall)})
    return q


ANIMS = {"idle": (pose_idle, 8.0), "walk": (pose_walk, 1.2), "run": (pose_run, 0.5), "bask": (pose_bask, 6.0),
         "bite": (pose_bite, 0.6),
         "stare": (pose_stare, 2.0), "watch": (pose_watch, 4.0), "drink": (pose_drink, 1.5), "sleep": (pose_sleep, 6.0),
         "stalk": (pose_stalk, 2.0), "approach": (pose_approach, 1.6), "pounce": (pose_pounce, 0.8), "alarm": (pose_alarm, 1.4),
         "sniff": (pose_sniff, 2.2), "look": (pose_look, 3.0), "scratch": (pose_scratch, 2.0), "stretch": (pose_stretch, 2.4),
         "play": (pose_play, 1.2), "groom": (pose_groom, 2.5), "nuzzle": (pose_nuzzle, 2.0), "flinch": (pose_flinch, 0.45),
         "death": (pose_death, 1.6)}
# The clips it had before the behaviour ones, exported as they were.
OLD = ("idle", "walk", "run", "bask", "bite")
# The held ones and the gaits loop; a clip played once must not (the engine loops a clip whose file
# says so, whatever the game asks); the death holds its last frame.
LOOPS = {"stare": True, "watch": True, "drink": True, "sleep": True, "stalk": True, "approach": True, "death": "hold_on_last_frame"}
BLENDS = {"flinch": (0.05, 0.2), "death": (0.1, 0.0), "sleep": (0.6, 0.6), "watch": (0.4, 0.4), "stalk": (0.3, 0.3),
          "approach": (0.3, 0.3), "pounce": (0.1, 0.25)}
BLINKS = {"idle": [1.0, 5.2], "bask": [2.5],  # basking, a slow blink
          "stare": [1.2], "watch": [1.1, 3.0], "drink": [0.8], "approach": [0.9], "look": [1.6], "sniff": [1.1], "nuzzle": [1.4]}
BLINK = {"idle": 0.1, "bask": 0.35}


def eyelids(name, t):
    d = BLINK.get(name, 0.1)
    a = 90 * max([0.0] + [1 - abs(t - b) / d for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}  # side eyes: the lids swing down about z


def frame_times(name, dur):
    n = {"idle": 96, "bask": 48, "walk": 24, "run": 20, "bite": 24,
         "stare": 16, "watch": 40, "drink": 24, "sleep": 36, "stalk": 32, "approach": 24, "pounce": 24, "alarm": 28,
         "sniff": 26, "look": 30, "scratch": 48, "stretch": 36, "play": 30, "groom": 36, "nuzzle": 24, "flinch": 12,
         "death": 32}.get(name, 16)
    ts = {round(dur * k / n, 4) for k in range(n + 1)}
    d = BLINK.get(name, 0.1)
    for b in BLINKS.get(name, []):
        ts |= {round(b + x, 4) for x in (-d, -d / 2, 0, d / 2, d)}
    return sorted(ts)


def animations(bone_names):
    out = {}
    for name, (fn, dur) in ANIMS.items():
        frames = []
        loops = LOOPS.get(name, False)
        for t in frame_times(name, dur):
            # A loop's last frame is its first; a clip played once ends where it ends.
            tt = t % dur if t < dur else (0.0 if name in OLD or loops is True else dur)
            pose = fn(tt)
            if name not in ("sleep", "death"):  # those keep their eyes shut
                pose.update(eyelids(name, tt))
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames}
        if name not in OLD:
            out[name].update(loop=loops, blend=BLENDS.get(name, (0.25, 0.3)))
    # A clip goes over what plays below only on the bones it writes, and the exporter leaves out a bone
    # that stays at 0: kept at a hundredth of a degree, so the idle's head lift, gape and tail sweep
    # don't show through a clip that holds them still. The eyelids are left to the idle's blinks, the
    # legs to the walk below (the idle doesn't move them).
    moved = {b for f in out["idle"]["frames"] for b, r in f["rot"].items() if any(abs(v) > 1e-6 for v in r)}
    for name, anim in out.items():
        if name in OLD:
            continue
        for b in moved - {"eyelid_L", "eyelid_R"}:
            if all(not any(abs(v) > 1e-6 for v in f["rot"][b]) for f in anim["frames"]):
                for f in anim["frames"]:
                    f["rot"][b] = [0.01, 0.0, 0.0]
    return out


# ------------------------------------------------------------------ fitting poses to the ground

GROUPS = {"feet": lambda n: n.endswith("_paw"), "body": lambda n: n in ("hips", "chest"),
          "tail": lambda n: n.startswith("tail_"), "jaw": lambda n: n == "jaw"}


def fit_poses(spec):
    """Basking: the belly on the ground, the feet flat beside it, the jaw clear of it, nothing below."""
    def cost(P):
        g = lowest(spec, pose_bask(0.0, dict(L, **P, bask_drop=0)), GROUPS)
        f = min(g["body"], g["feet"], g["tail"])
        return (g["body"] - f) ** 2 + (g["feet"] - f) ** 2 + 0.5 * max(0.0, g["tail"] - f - 1.5) ** 2 + 10 * max(0.0, f - g["jaw"]) ** 2
    best = fit({"bask_splay": list(range(0, 61, 4))}, cost)[1]
    g = lowest(spec, pose_bask(0.0, dict(L, **best, bask_drop=0)), GROUPS)
    L.update(best, bask_drop=round(float(min(g.values())), 2))
    gi = lowest(spec, pose_idle(4.0), GROUPS)
    print("gaping, lowest:", {k: round(float(v), 2) for k, v in gi.items()})
    print("basking, lowest:", {k: round(float(v - min(g.values())), 2) for k, v in g.items()})

    # Drinking: the jaw a little open, its lowest a third of a unit over the ground (the water).
    hd = [x / 4 for x in range(-80, 41)]
    L.update(fit({"drink_head": hd}, lambda P: (lowest(spec, pose_drink(0.0, dict(L, **P)), GROUPS)["jaw"] - 0.3) ** 2)[1])
    # Sniffing: the closed jaw near the ground (0.6 over it).
    L.update(fit({"sniff_head": hd}, lambda P: (lowest(spec, pose_sniff(1.1, dict(L, **P)), GROUPS)["jaw"] - 0.6) ** 2)[1])
    # Asleep: on the basking belly, the chin laid low, the jaw 0.3 over the ground.
    L.update(fit({"sleep_head": hd}, lambda P: (lowest(spec, pose_sleep(0.0, dict(L, **P)), GROUPS)["jaw"] - 0.3) ** 2)[1])

    # Stalking: the legs sprawled until the belly is 0.6 over the ground, standing on its feet.
    def stalk_cost(P):
        g = lowest(spec, stalk_legs(0.0, P["stalk_splay"], 0, 0), GROUPS)
        return (g["body"] - g["feet"] - 0.6) ** 2
    L.update(fit({"stalk_splay": [x / 2 for x in range(0, 61)]}, stalk_cost)[1])
    print("drinking, sniffing, asleep, stalking:", {k: L[k] for k in ("drink_head", "sniff_head", "sleep_head", "stalk_splay")})


# ------------------------------------------------------------------ build


def build():
    B = bones()
    E = elements()
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"] and not e["rot"]]
    LAYERS["rec"] = Recorder(layer_bounds(E, 3.0))
    tiles = []
    for i, e in enumerate(E):
        for f in ([e["quad"]] if e["quad"] else FACES):
            LAYERS["key"] = (i, f)
            tiles.append(((i, f), paint_face(e, f, boxes)))
    sheet, uv = pack(tiles, 512)  # the layers pack at the same width (`Recorder.save`)
    LAYERS["rec"].save(OUT, [k for k, _ in tiles], pack)
    LAYERS["rec"] = None
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "caiman", "texture": [sheet.shape[1], sheet.shape[0]],
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
