#!/usr/bin/env python3
"""Kryptobaatar, the Djadokhta's commonest mammal: a multituberculate, made the way Hytale makes
creatures (boxes and a hand-painted texture) with the blocky-model skill, from research alone
(README.md), on the wood mouse's build (`source/mouse`): its colors are rules (see `area`).

    ~/.venvs/blocky/bin/python kryptobaatar.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each), y up, it faces +z. A real one was
about 10 cm head and body, its skull some 2.5 cm: too small for a face at the house density, so the
body is drawn about 1.3 times life and the head and eyes bigger still, as the mouse's: 9 units nose
to rump, the tail 7 more. Not a rodent: a blunter, broader snout over its gnawing incisors, small
round ears, a thicker tail.
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
from ground import grounded, lowest  # noqa: E402
from house import SCALE, fine, texels  # noqa: E402

OUT = os.environ.get("KRYPTO_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

FUR, BACK, BELLY, PINK, NOSE, EARIN, CLAW = range(7)
# No record of its colors (est.: a small desert mammal's): FUR grey-brown on the flanks, BACK
# darker along the spine, BELLY the pale underside, PINK the bare feet and tail's underside, NOSE
# its tip, EARIN the inside of the ears.
BASE = [(132, 108, 84), (98, 78, 62), (206, 198, 182), (194, 156, 144), (170, 116, 112), (170, 138, 128), (90, 74, 68)]
SHADOW = [0.02, 0.02, 0.62, 0.97, 0.97, 0.97, 0.02]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (FUR, BACK, BELLY)
ROLES = [("coat", 0, 0), ("coat", 0, -1.0), ("coat", 1, 0), "fixed", "fixed", "fixed", "fixed"]

# Big, black eyes (a night forager's): glossy, a catchlight toward the front and up with a dimmer
# gloss round it, a grey-blue sparkle low behind, a warm glow along the bottom, darker fur round it.
# Painted at 128 a block (`K`), as the mouse.
EYES.update({
    "mouse_black": dict(pattern=[".PPP.",
                                 "PHhPP",
                                 "PhPPP",
                                 "PPPsP",
                                 ".III."], colors={"P": (22, 15, 17), "H": (226, 228, 234), "h": (98, 96, 108), "s": (88, 98, 122),
                                                   "I": (64, 40, 32)}, halo="round"),
    # At 128 a block (mouse_black was for 256), 3 texels by 3 on a 6-by-5 side, the mouse's: the
    # catchlight in the front top corner, the grey-blue sparkle low behind, the warm glow (its iris,
    # recolored in the game) at a rounded bottom; no halo on a 6-texel face.
    "krypto_px": dict(pattern=["HPP",
                               "PPs",
                               ".I."], colors={"P": (22, 15, 17), "H": (226, 228, 234), "s": (88, 98, 122), "I": (86, 54, 40)},
                      halo="none", iris="I"),
})
EYE_STYLE = os.environ.get("KRYPTO_EYES", "krypto_px")
SKULL = ((-1.25, 1.7, 1.7), (1.25, 4.0, 4.7))
# Texels a unit. The eye sits on the skull's side, 3.0 by 2.3 units: 1.2 texels at the house's 32 a
# block, under `house.MIN_FACE`, so it paints at 128 a block (`fine` gives 4): 2 texels a unit, the
# side 6 texels by 5.
K = texels(SCALE, fine(min(SKULL[1][1] - SKULL[0][1], SKULL[1][2] - SKULL[0][2])))
EYE_Z, EYE_Y = 3.6, 2.95  # high on the side, toward the front: at 128 a block, a row of fur above it
LID_X = 1.6  # the eyelids' hinge plane, 0.35 outside the head's side: closed, they sit over the eye
LASH = (40, 26, 22)


def side_grid():
    """The skull side's texels: (width, height) and one texel's size in units (along z, along y)."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h = max(1, round(K * (z1 - z0))), max(1, round(K * (y1 - y0)))
    return w, h, (z1 - z0) / w, (y1 - y0) / h


def eye_center():
    """The eye's center on the skull's side in its texels, counted from the front edge (both sides
    are stamped from the front, so the catchlight is toward the front on both)."""
    w, h, su, sv = side_grid()
    return (SKULL[1][2] - EYE_Z) / su, (SKULL[1][1] - EYE_Y) / sv


def eye_box(side):
    """(z0, z1, top, bottom) of the eye on one side in units, rounded as `stamp_eyes` places it."""
    pat = EYES[EYE_STYLE]["pattern"]
    ph, pw = len(pat), len(pat[0])
    w, h, su, sv = side_grid()
    u0, v0 = eye_corner(*eye_center(), pat, w)
    (x0, y0, z0), (x1, y1, z1) = SKULL
    return z1 - (u0 + pw) * su, z1 - u0 * su, y1 - v0 * sv, y1 - (v0 + ph) * sv

# ------------------------------------------------------------------ the model


def el(name, bone, lo, hi, quad=None, rot=None, origin=None, cut=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad, rot=rot,
                origin=[float(v) for v in origin] if origin else None, cut=cut)


LEGS = ("FL", "FR", "BL", "BR")
FRONT_Z, BACK_Z, FRONT_X, BACK_X = 0.0, -2.2, 0.6, 1.1
# Half widths across the body (x), chosen so no two faces that can meet in a pose share a plane:
# the hips, the chest, the haunch inside and out of its leg's line, the hind shin and foot, the
# front leg and paw.
# The chest 1.9 across (1.65 left a 0.75 ring of its front round the neck; linecheck).
G = {"hips": 2.3, "chest": 1.9, "thigh_in": 0.8, "thigh_out": 1.55, "shin": 0.2, "foot": 0.85, "fleg": 0.15, "fpaw": 0.55,
     "foot_len": 1.0, "belly": 1.2, "hind": 1.5}
TAIL = [(-8, 3.0, 1.04), (-4, 4.0, 0.6)]  # pitch (deg, down), length, width (thinner to the tip)
TAIL0 = np.array([0.0, 1.9, -3.4])
EAR = [(3.7, 4.3, 0.6), (4.3, 4.7, 0.4)]  # y0, y1, half width: small and round
EAR_X, EAR_Z = 0.85, (2.8, 3.6)


def tail_points():
    pts, p = [TAIL0.copy()], TAIL0.copy()
    for pitch, length, _ in TAIL:
        a = math.radians(pitch)
        p = p + np.array([0, math.sin(a), -math.cos(a)]) * length
        pts.append(p.copy())
    return pts


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 2.0, -1.6)), ("spine_1", "spine_0", (0, 2.0, -0.95)),
         ("head", "spine_1", (0, 2.6, 1.9))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B += [(f"ear_{n}", "head", (s * EAR_X, 3.85, sum(EAR_Z) / 2)), (f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2))]
    pts = tail_points()
    B += [(f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", tuple(pts[k])) for k in range(len(TAIL))]
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x = FRONT_X * s
            # The paw hinges at its back edge: lifting, it curls its toes up.
            B += [(f"leg_{leg}_upper", "spine_1", (x, 1.6, FRONT_Z)), (f"leg_{leg}_paw", f"leg_{leg}_upper", (x, 0.45, FRONT_Z - 0.4))]
        else:
            x = BACK_X * s
            B += [(f"leg_{leg}_upper", "spine_0", (x, 2.0, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 1.85, BACK_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 0.53, BACK_Z))]
    return B


def elements():
    E = [el("hips", "spine_0", (-G["hips"], G["belly"], -3.5), (G["hips"], 3.5, -0.9)),  # pear-shaped: wide behind
         el("rump", "spine_0", (-0.85, 1.6, -3.9), (0.85, 3.0, -3.1)),
         el("chest", "spine_1", (-G["chest"], 1.55, -1.5), (G["chest"], 3.1, 1.2)),
         # Short, deep in the head: it turns on it; its bottom half a unit over the chest's (0.6 read as a rim; linecheck).
         el("neck", "spine_1", (-0.9, 2.05, -0.5), (0.9, 3.5, 3.2)),
         el("skull", "head", *SKULL),
         # The blunt, broad snout, out two texels from the skull (at 5.4, 1.4 texels, its sides and top read as a rim; linecheck).
         el("muzzle", "head", (-0.75, 2.0, 4.3), (0.75, 3.25, 5.75))]
    for s, n in ((1, "L"), (-1, "R")):
        cx = EAR_X * s
        # Big round ears: two thin steps and a cut-out panel through the middle for the outline,
        # turned out a little.
        rot, origin = (-8, 0, -s * 18), (cx, 3.85, sum(EAR_Z) / 2)
        for k, (y0, y1, hw) in enumerate(EAR):
            E.append(el(f"ear_core_{n.lower()}{k}", f"ear_{n}", (cx - hw, y0, EAR_Z[0]), (cx + hw, y1, EAR_Z[1]), rot=rot, origin=origin))
        zc = sum(EAR_Z) / 2
        E.append(el(f"fringe_ear_{n.lower()}", f"ear_{n}", (cx - 0.8, 3.7, zc), (cx + 0.8, 5.0, zc), quad="south", rot=rot,
                    origin=origin, cut="ear"))
        # The eyelid: a quad the eye's size lying flat inside the head, hinged at the eye's top; a
        # blink swings it down over the eye. Tilted a few degrees about its hinge, so it is never in
        # a face's plane.
        za, zb, top, bottom = eye_box(s)
        lx0, lx1 = (LID_X - (top - bottom), LID_X) if s > 0 else (-LID_X, -LID_X + (top - bottom))
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down", rot=(0, 0, s * 4),
                    origin=(s * LID_X, top, (za + zb) / 2)))
    pts = tail_points()
    for k, (pitch, length, w) in enumerate(TAIL):
        p = pts[k]
        E.append(el(f"tail_{k}", f"tail_{k}", (p[0] - w / 2, p[1] - w / 2, p[2] - length), (p[0] + w / 2, p[1] + w / 2, p[2] + 0.6),
                    rot=(pitch, 0, 0), origin=tuple(p)))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z = FRONT_X * s, FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - G["fleg"], 0.35, z - 0.3), (x + G["fleg"], 2.0, z + 0.3)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - G["fpaw"], 0, z - 0.65), (x + G["fpaw"], 0.6, z + 0.9)))
        else:
            x, z = BACK_X * s, BACK_Z
            xi, xo = sorted((x - G["thigh_in"] * s, x + G["thigh_out"] * s))  # the haunch
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (xi, 1.75, z - 0.9), (xo, 2.3, z + 0.7)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - G["shin"], 0.5, z - 0.3), (x + G["shin"], 1.95, z + 0.3)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - G["foot"], 0, z - 0.7), (x + G["foot"], 0.6, z + G["foot_len"])))  # the long hind foot
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
    # Texels: K a unit, at least one for a face with any size (one rounding to 0 would vanish).
    return np.array(corner, float), du, dv, max(1, int(round(K * abs(size @ du)))), max(1, int(round(K * abs(size @ dv)))), n


def rotm(rot):
    if not rot:
        return np.eye(3)
    rx, ry, rz = (math.radians(a) for a in rot)
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


def area(e, face, p, uu, vv, h):
    """Grey-brown, darker along the back, pale below; bare pinkish feet, a tail dark above and
    pale below, the ears pinkish inside (est.)."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, FUR)
    side = face in ("east", "west")
    if name in ("hips", "rump", "chest", "neck", "skull", "muzzle"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        else:
            a = np.where(y < (2.1 if name != "muzzle" else 2.5), BELLY, FUR)
            if name == "chest" and face == "south":
                a = np.where(y < 2.4, BELLY, FUR)
    elif name.startswith("tail_"):  # dark above, pale below, as a wood mouse's
        a[:] = BELLY if face == "down" else BACK
        if side:
            a = np.where(vv > h / 2, BELLY, BACK)
    elif name.startswith("ear_core"):
        a[:] = FUR
        if face == "south":
            a[:] = EARIN
    elif name.startswith("thigh_"):
        a[:] = FUR
        if face == "down":
            a[:] = BELLY
    elif name.startswith("leg_"):
        a[:] = PINK if name.endswith("_paw") or y.max() < 1.0 else FUR
        if face == "down" and not name.endswith("_paw"):
            a[:] = BELLY
    return a


def cut_mask(e, p):
    """The ear's outline: small and round."""
    x, y = p[..., 0], p[..., 1]
    cx = e["origin"][0]
    return ((x - cx) / 0.75) ** 2 + ((y - 4.3) / 0.7) ** 2 <= 1.0


# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}  # a layers.Recorder while build() paints, and the face's key


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)


def layer_props(e, face, p, a):
    """White where the belly is (and up from it), the tail's tip and the feet; the back's darker
    stripe is the pattern's place (an agouti coat has none to speak of)."""
    n, y = e["name"], p[..., 1]
    lo, hi = e["lo"][1], e["hi"][1]
    low = 1 - _ramp(y, lo, max(hi, lo + 1e-3))
    white = 0.6 * low if n in ("chest", "hips", "rump", "skull") else np.full(y.shape, 0.3)
    white = np.maximum(white, np.where(a == BELLY, 0.85, 0))
    pattern = np.full(y.shape, 1.0 if n in ("hips", "rump", "chest", "skull") else 0.3)
    return {"white": white, "pattern": pattern}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
    p = corner + uu[..., None] * du * (abs((np.array(e["hi"]) - np.array(e["lo"])) @ du) / w) \
        + vv[..., None] * dv * (abs((np.array(e["hi"]) - np.array(e["lo"])) @ dv) / h)
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    light_face = face
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "tail")):
        light_face = "north"  # seen when a pose lifts or folds it
    if e["name"].startswith("eyelid"):
        light_face = "east"  # seen only closed, as the side of the head
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    stacked = e["name"].startswith("ear_core")  # the ear's boxes, lit as one piece
    down = np.clip((4.9 - p[..., 1]) / 1.4, 0, 1) if stacked else None
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=p[..., 1] if not e["name"].startswith("leg_") else None,
                            occ=occ, quad=bool(e["quad"]), seed=sum(map(ord, e["name"] + face)) % 97, down=down, tuft=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"] == "ear":
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "skull" and face in ("east", "west"):
        # Both sides counted from the front: the west face's texels run back to front, so stamp it
        # mirrored (the catchlight toward the front on both).
        stamp_eyes(img if face == "east" else img[:, ::-1], [eye_center()], EYE_STYLE)
    if e["name"] == "muzzle" and face == "south":  # the pink nose at the tip, the mouth below it
        # At 128 a block the snout's tip is 3 texels by 2: the nose the middle of its top row, the mouth's
        # shadow and the incisors (one texel, in the middle) the bottom one (the nostril slit and the
        # split lip need 6 by 5: dropped).
        c0, c1 = (w - 1) // 2, w // 2 + 1
        img[0, c0:c1, :3] = BASE[NOSE]
        if h >= 3:  # the split lip under the nose
            img[1:h - 1, c0:c1, :3] = (img[1:h - 1, c0:c1, :3] * 0.8).astype(np.uint8)
        if h >= 2:
            img[h - 1, :, :3] = (img[h - 1, :, :3] * 0.82).astype(np.uint8)
        if h >= 2:
            img[h - 1, c0:c1, :3] = (222, 206, 170)  # the gnawing incisors
    if e["name"] == "muzzle" and face in ("east", "west"):  # the nose's pink carries round the tip
        tip = 0 if face == "east" else w - 1
        img[0, tip, :3] = BASE[NOSE]
    if e["name"].startswith("eyelid"):  # closed, the lash line is its lower edge
        img[:, 0 if e["name"].endswith("L") else w - 1, :3] = LASH
    if e["name"].endswith("_paw") and face == "south" and w >= 3:  # tiny dark claws
        # At 128 a block the hind foot's front is 3 texels by 1 and the front paw's 2 by 1 (no
        # claws): half-dark, so the toes hint and the foot doesn't turn black.
        img[h - 1, ::2, :3] = ((img[h - 1, ::2, :3] + np.array(BASE[CLAW])) / 2).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"sit_angle": 60, "sit_hip": -60, "sit_knee": 120, "sit_drop": 1.0, "sit_front": -70}


def tail_swing(k, angle, base=(0, 0, 0)):
    pitch = math.radians(TAIL[k][0])
    return swing((0, math.sin(pitch), -math.cos(pitch)), angle, base)


def gait(p, phases, swing_amp, lift, hind=None):
    """Scurrying: a foot swings forward while it's lifted, back while it's down."""
    out = {}
    for leg, ph in zip(LEGS, phases):
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))
        out[f"leg_{leg}_upper"] = (-swing_amp * S(a), 0, 0)
        knee = lift * up
        if leg[0] == "F":
            out[f"leg_{leg}_paw"] = (-knee, 0, 0)  # the toes curl up
        else:
            sw = out[f"leg_{leg}_upper"][0]
            knee *= G["hind"] if hind is None else hind  # the longer hind leg folds more
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (knee, 0, 0), (-knee - sw, 0, 0)  # the hock folds back, the foot flat
    return out


def tail_sway(p, n, amp):
    return {f"tail_{k}": tail_swing(k, amp * S(TAU * p * n - 0.8 * k)) for k in range(len(TAIL))}


def pose_idle(t):
    """Sniffing: quick nods and turns of the head, the ears turning, the tail swaying (3 s)."""
    p = t / 3.0
    sniff = 3 * max(0.0, S(TAU * t * 6)) * (0.3 < p < 0.6)
    turn = float(np.interp(p, [0, .15, .25, .6, .7, 1], [0, 0, 10, 10, -8, 0]))
    tw = max(0.0, S(TAU * (p - 0.75) * 10)) if 0.75 < p < 0.85 else 0.0
    q = {"root": (0, 0.06 * S(TAU * p * 4), 0), "spine_1": (0.6 * S(TAU * p * 4), 0, 0),
         "head": (sniff + 4 * S(TAU * p), turn, 0), "ear_L": (0, 0, -20 * tw), "ear_R": (0, 0, 4 * S(TAU * p * 2))}
    q.update(tail_sway(p, 1, 8))
    return q


def pose_walk(t):
    p = t / 0.4
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 26, 45)  # tiny legs: a big swing and lift
    q.update({"root": (0, 0.15 * abs(S(TAU * p)), 0), "head": (2 * S(TAU * p * 2), 0, 0)})
    q.update(tail_sway(p, 1, 5))
    return q


def pose_run(t):
    """A bounding dash, the tail streaming behind (0.24 s)."""
    p = t / 0.24
    q = gait(p, (0.0, 0.08, 0.5, 0.58), 40, 70, hind=0.9)
    c = S(TAU * p)
    q.update({"root": (0, 0.4 * max(0.0, S(TAU * p + 0.6)), 0), "spine_0": (-5 * c, 0, 0), "spine_1": (7 * c, 0, 0),
              "ear_L": (-25, 0, 0), "ear_R": (-25, 0, 0), "tail_0": tail_swing(0, 0, (-6, 0, 0))})
    return q


def pose_sit(t, P=L):
    """Up on its haunches, the front paws held to the mouth, nibbling (3 s)."""
    p = t / 3.0
    a = P["sit_angle"]
    nib = 4 * max(0.0, S(TAU * t * 5))
    q = {"root": (0, -P["sit_drop"] + 0.05 * S(TAU * p * 3), 0), "spine_0": (-a, 0, 0), "head": (a * 0.5 + nib, 3 * S(TAU * p), 0)}
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (a + P["sit_hip"], 0, 0)
        q[f"leg_{leg}_lower"] = (P["sit_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["sit_hip"] - P["sit_knee"], 0, 0)
    for leg, s in (("FL", 1), ("FR", -1)):
        q[f"leg_{leg}_upper"] = (P["sit_front"] + nib, 0, s * 8)
        q[f"leg_{leg}_paw"] = (40, 0, 0)
    for k in range(len(TAIL)):  # the tail laid out behind on the ground
        q[f"tail_{k}"] = (a * 0.9 if k == 0 else 4, 0, 0)
    return q


# ------------------------------------------------------------------ the behaviour clips

# The behaviour clips the game asks for by name (`mods/animals/src/anim.rs`), on the mouse's poses:
# held (stare, graze, drink, sleep, watch), the approach gait, the ones played once over what plays
# below (alarm, sniff, look, scratch, stretch, play, groom, nuzzle, flinch) and the death. Too small to
# stand its ground: no rear or charge. The mouse, the vole and kryptobaatar share them (one layout).
# At this size a degree is a lot: the head is a third of the body, the legs a unit or two long. And
# the neck is a box of the chest deep in the skull: the head turns 20 degrees at most (less bowed),
# bows 30 and lifts 15, the chest turns 15 on the hips, or a corner shows through (`pokes`); a
# bigger turn is shared between the chest and the head.


def add(q, more):
    for k, v in more.items():
        a = q.get(k, (0, 0, 0))
        q[k] = tuple(a[i] + v[i] for i in range(3))
    return q


def mix(a, b, d):
    """Pose a turned toward pose b by d (0..1), bone by bone."""
    out = {}
    for k in set(a) | set(b):
        u, v = a.get(k, (0, 0, 0)), b.get(k, (0, 0, 0))
        out[k] = tuple(u[i] + (v[i] - u[i]) * d for i in range(3))
    return out


def stand(t, period=3.0):
    """Standing still on its four feet, breathing: what a clip builds on."""
    br = S(TAU * t / period * 2)
    return {"root": (0, 0.03 * br, 0), "spine_1": (0.4 * br, 0, 0)}


def seated(d=1.0, P=None, front=None):
    """Sat up on its haunches by d (0..1) of a fitted sit (`P`: the sit's or the watch's angles): the
    body tipped back, the hind legs folded under it with the feet flat, the forepaws raised to the
    chest (`front`: the shoulder's angle when sat), the tail laid along the ground. No root: the
    clip stands it on its hind feet (`PLANTED`)."""
    P = P or L
    a = P["sit_angle"] * d
    q = {"spine_0": (-a, 0, 0)}
    r = math.sqrt(max(d, 0.0))
    for leg, s in (("BL", 1), ("BR", -1)):  # the thighs splayed a little, the feet kept flat (on the way, square, a
        # thigh's underside lay in the chest's: flicker)
        q[f"leg_{leg}_upper"] = ((P["sit_angle"] + P["sit_hip"]) * d, 0, s * 5 * r)
        q[f"leg_{leg}_lower"] = (P["sit_knee"] * d, 0, 0)
        q[f"leg_{leg}_paw"] = ((-P["sit_hip"] - P["sit_knee"]) * d, 0, -s * 5 * r)
    f = P["sit_front"] if front is None else front
    for leg, s in (("FL", 1), ("FR", -1)):
        q[f"leg_{leg}_upper"] = (f * d, 0, s * 8 * d)
        q[f"leg_{leg}_paw"] = (40 * d, 0, 0)
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = (a * 0.9 if k == 0 else 4 * d, 0, 12 * r * (1 - d) if k == 0 else 0)  # rolled a little: on the way, level with the thighs' tops it flickered
    return q


def ears(up=1.0, back=0.0, turn=0.0):
    """The ears pricked up and forward by `up` (0..1), laid back by `back` degrees, turned by `turn`."""
    # (turned out 8 degrees: square to the head, a bowing head brought an ear's panel into the neck's front)
    return {"ear_L": (6 * up - back, turn + 8, 10 * up), "ear_R": (6 * up - back, turn - 8, -10 * up)}


def nose(t, rate=6.0, amp=2.5):
    """The twitching nose and whiskers: quick small nods of the head."""
    return {"head": (amp * max(0.0, S(TAU * t * rate)), 0, 0)}


def head_low(g, t=0.0, period=3.0):
    """The nose to the ground by g (1: the muzzle a little over it, fitted): the head bowed, the
    shoulders a little lowered, the forelegs set back just enough to keep the paws on the ground
    (the head alone can't reach it without showing the neck through the skull)."""
    c, b = L["crop_chest"] * g, L["crop_leg"] * g
    q = add(stand(t, period), {"head": (L["crop_head"] * g, 0, 0), "spine_1": (c, 0, 0)})
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (b - c, 0, 0)
        q[f"leg_{leg}_paw"] = (-b, 0, 0)  # flat: the turns sum to nothing
    return q


def pose_stare(t):
    """Frozen, the head a little raised, the ears straight up, only the breath moving (2 s, held)."""
    q = add(stand(t, 2.0), {"head": (-6, 0, 0)})
    q.update(ears(1.0))
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = (0.01, 0, 0)
    return q


def pose_watch(t):
    """A sentinel: sat up tall on its haunches, the forepaws tucked to the chest, the ears up, the
    head turning slowly one way and the other, the nose twitching (3 s, held)."""
    p = t / 3.0
    yaw = 24 * S(TAU * p)
    q = seated(1.0, front=-40)
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_paw"] = (75, 0, 0)  # curled, the paws held in
    br = S(TAU * p * 2)
    add(q, {"spine_1": (0.6 * br, 0.5 * yaw, 0), "head": (10, 0.5 * yaw, 0)})  # the nose up, sniffing the air
    add(q, nose(t, 5.0, 2.0))
    q.update(ears(1.0, turn=-0.3 * yaw))
    return q


def pose_graze(t):
    """Feeding: nibbling at the ground (the head working in small nods and turns), then sat up with a
    seed in the forepaws at the mouth, nibbling it, and down again (4 s, held)."""
    p = t / 4.0
    up = float(np.interp(p, [0, .45, .55, .88, .98, 1], [0, 0, 1, 1, 0, 0]))
    nib = 3.0 * max(0.0, S(TAU * t * 5))
    low = head_low(1.0, t, 4.0)
    add(low, {"head": (nib - 2 * (1 - abs(S(TAU * p * 2))), 0, 0), "spine_1": (0, 8 * S(TAU * p * 3), 0)})
    sit = seated(1.0)
    add(sit, {"head": (L["sit_angle"] * 0.5 + nib, 0, 0)})
    for leg in ("FL", "FR"):
        add(sit, {f"leg_{leg}_upper": (nib, 0, 0)})
    q = mix(low, sit, up)
    q.update(ears(0.4))
    return q


def pose_drink(t):
    """Drinking: the nose at the water, lapping in quick small dips (1.5 s, held)."""
    q = head_low(1.0, t, 1.5)
    add(q, {"head": (L["drink_dip"] * max(0.0, S(TAU * t * 4)), 0, 0)})
    q.update(ears(0.3))
    return q


def pose_sniff(t):
    """The nose down low, sweeping side to side, the whiskers twitching fast, and up (1.8 s, once)."""
    p = t / 1.8
    d = float(np.interp(p, [0, .2, .8, 1], [0, 1, 1, 0]))
    q = head_low(0.8 * d, t, 1.8)
    sw = d * S(TAU * p * 2)
    add(q, {"spine_1": (0, 12 * sw, 0), "head": (0, 4 * sw, 0)})
    add(q, nose(t, 9.0, 2.0 * d))
    q.update(ears(0.7 * d))
    return q


def pose_look(t):
    """The head up and round one way, then the other, the ears following (2.4 s, once)."""
    p = t / 2.4
    yaw = float(np.interp(p, [0, .15, .4, .55, .8, 1], [0, 40, 40, -38, -38, 0]))
    up = float(np.interp(p, [0, .12, .88, 1], [0, 1, 1, 0]))
    q = add(stand(t, 2.4), {"spine_1": (0, 0.35 * yaw, 0), "head": (-8 * up, 0.45 * yaw, 0)})
    add(q, nose(t, 5.0, 2.0))
    q.update(ears(up, turn=0.3 * yaw))
    return q


def pose_scratch(t):
    """An itch: sat back, the head bent round and tipped toward the left hind foot, which comes up to
    the ear and scratches fast, the eyes half shut (1.4 s, once). Fitted: the toe at the ear."""
    p = t / 1.4
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    k = d * S(TAU * t * 7)  # seven strokes a second
    q = scratch_pose([L[n] * d for n in SCRATCH])
    add(q, {"leg_BL_upper": (-7 * k, 0, 0), "leg_BL_lower": (9 * k, 0, 0)})
    q.update(ears(0.0, back=15 * d))
    q["ear_L"] = (q["ear_L"][0], 0, q["ear_L"][2] + 4 * k)  # knocked by the foot
    q.update({"eyelid_L": (0, 0, 60 * d), "eyelid_R": (0, 0, -60 * d)})
    return q


def pose_groom(t):
    """Washing: sat up, the forepaws wiping down over the face in turn, then up behind the ears with
    the head bowed, the eyes half shut (2.6 s, once). Fitted: the paws at the cheeks, then the ears."""
    p = t / 2.6
    d = float(np.interp(p, [0, .15, .88, 1], [0, 1, 1, 0]))
    e = float(np.interp(p, [0, .5, .58, 1], [0, 0, 1, 1]))  # from the face to behind the ears
    q = seated(d)
    face = {n: L["gf_" + n] * (1 - e) + L["ge_" + n] * e for n in GROOM}
    add(q, {"head": ((L["sit_angle"] * 0.5 + face["head"]) * d, 0, 0)})
    for leg, s, ph in (("FL", 1, 0.0), ("FR", -1, math.pi)):
        w = d * S(TAU * t * 3 + ph)  # three wipes a second, the paws in turn
        # (splayed by under a degree, raised the leg's side lay in the neck's: flicker)
        q[f"leg_{leg}_upper"] = ((face["up"] + 10 * w) * d, s * 6 * d, s * face["out"] * math.sqrt(d))
        q[f"leg_{leg}_paw"] = ((face["paw"] - 8 * w) * d, 0, 0)
    q.update(ears(0.2 * d, back=(10 + 25 * e) * d))
    q.update({"eyelid_L": (0, 0, 55 * d), "eyelid_R": (0, 0, -55 * d)})
    return q


def pose_stretch(t):
    """A bow, the forepaws reached out in front and the shoulders low, the head up; then the left hind
    leg pushed out straight behind (2.0 s, once). Fitted: the forepaws on the ground, the hind foot
    just off it."""
    p = t / 2.0
    a = float(np.interp(p, [0, .2, .5, .6, 1], [0, 1, 1, 0, 0]))
    b = float(np.interp(p, [0, .5, .65, .88, 1], [0, 0, 1, 1, 0]))
    q = stand(t, 2.0)
    reach, low = L["st_reach"] * a, L["st_chest"] * a
    add(q, {"spine_1": (low, 0, 0), "head": (-13 * a, 0, 0)})
    for leg in ("FL", "FR"):
        add(q, {f"leg_{leg}_upper": (-reach - low, 0, 0), f"leg_{leg}_paw": (reach, 0, 0)})  # flat: the turns sum to nothing
    add(q, {"leg_BL_upper": (L["st_hip"] * b, 0, 0), "leg_BL_lower": (L["st_knee"] * b, 0, 0), "leg_BL_paw": (L["st_foot"] * b, 0, 0)})
    q.update(ears(0.0, back=25 * a))
    q.update({"eyelid_L": (0, 0, 55 * a), "eyelid_R": (0, 0, -55 * a)})  # the yawn (shut 70, a lid lay in the laid-back ear's side)
    return q


def pose_sleep(t, P=None):
    """Asleep curled in a ball: the legs folded under, the back rounded, the nose tucked down to the
    chest, the tail wrapped round its side, the eyes shut, a slow breath (4 s, held)."""
    br = S(TAU * t / 4.0 * 2)
    q = curl(P or L)
    add(q, {"root": (0, 0.04 * br, 0), "spine_1": (0.6 * br, 0, 0)})
    q.update(ears(0.0, back=30))
    q.update({"eyelid_L": (0, 0, 90), "eyelid_R": (0, 0, -90)})
    return q


def pose_alarm(t):
    """A start: a jump straight back, landing frozen, the head up, the ears up, the tail raised;
    then it eases (1.0 s, once). Its height from its feet in `build`."""
    p = t / 1.0
    h = math.sin(math.pi * float(np.clip((p - 0.05) / 0.3, 0, 1)))  # the leap
    back = float(np.interp(p, [0, .05, .35, .7, 1], [0, 0, -0.9, -0.9, 0]))
    up = float(np.interp(p, [0, .06, .75, 1], [0, 1, 1, 0]))
    q = stand(t, 1.0)
    add(q, {"root": (0, 1.1 * h, back), "spine_0": (-8 * h, 0, 0), "spine_1": (4 * h, 0, 0), "head": (-10 * up, 0, 0)})
    for leg, s in (("FL", 1), ("FR", -1)):  # splayed a little: swung forward square, the leg's side lay in the neck's
        add(q, {f"leg_{leg}_upper": (-20 * h, 0, s * 6 * h), f"leg_{leg}_paw": (30 * h, 0, 0)})
    for leg in ("BL", "BR"):
        add(q, {f"leg_{leg}_upper": (20 * h, 0, 0), f"leg_{leg}_lower": (25 * h, 0, 0), f"leg_{leg}_paw": (-45 * h, 0, 0)})
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = tail_swing(k, 0, ((24 if k == 0 else 12) * up + (6 * h if k == 0 else 0), 0, 5 * up if k == 0 else 0))
    q.update(ears(up))
    return q


def pose_play(t):
    """Play: a little hop and spin one way, and a hop back round, the legs kicked out (1.2 s, once).
    Its height from its feet in `build`."""
    p = t / 1.2
    h1 = math.sin(math.pi * float(np.clip((p - 0.05) / 0.35, 0, 1)))
    h2 = math.sin(math.pi * float(np.clip((p - 0.55) / 0.35, 0, 1)))
    h = h1 + h2
    spin = float(np.interp(p, [0, .05, .4, .55, .9, 1], [0, 0, 110, 110, 0, 0]))
    spin = 110 * 0.5 * (1 - math.cos(math.pi * spin / 110))  # eased
    q = stand(t, 1.2)
    add(q, {"root": (0, 1.3 * h, 0), "spine_0": (-6 * h, spin, 0), "head": (-8 * h, 10 * (h1 - h2), 0)})
    for leg, s in (("BL", 1), ("BR", -1)):
        add(q, {f"leg_{leg}_upper": (35 * h, 0, s * 12 * h), f"leg_{leg}_paw": (-20 * h, 0, 0)})
    for leg, s in (("FL", 1), ("FR", -1)):
        add(q, {f"leg_{leg}_upper": (-30 * h, 0, s * 6 * h), f"leg_{leg}_paw": (40 * h, 0, 0)})
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = tail_swing(k, -25 * (h1 - h2), ((10 if k == 0 else 0) * h, 0, 0))
    q.update(ears(0.5 * h, back=15 * h))
    return q


def pose_nuzzle(t):
    """Nuzzling a young one: the head low and forward, two gentle pushes, the ears soft (1.6 s, once)."""
    p = t / 1.6
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    push = max(0.0, S(TAU * p * 2))
    q = head_low(0.6 * d, t, 1.6)
    add(q, {"head": (-6 * push * d, 5 * d * S(TAU * p), 0), "spine_1": (0, 5 * d * S(TAU * p), 0)})
    add(q, nose(t, 4.0, 2.0 * d))
    q.update(ears(0.4 * d))
    return q


def pose_flinch(t):
    """Hit: a jerk aside, the head ducked, the ears flattened, the tail clamped, the eyes squeezed
    (0.4 s, once)."""
    p = t / 0.4
    d = float(np.interp(p, [0, .15, .45, 1], [0, 1, 0.8, 0]))
    q = add(stand(t, 0.4), {"spine_1": (0, 6 * d, 4 * d), "head": (10 * d, -8 * d, 0)})
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = tail_swing(k, 18 * d)
    q.update(ears(0.0, back=40 * d))
    q.update({"eyelid_L": (0, 0, 75 * d), "eyelid_R": (0, 0, -75 * d)})
    return q


def pose_approach(t):
    """Curious, coming to look: slow careful steps, the head low and stretched forward, the nose
    twitching, the ears up (0.8 s a cycle, in place of the walk). Its height from its feet in `build`."""
    p = t / 0.8
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 16, 30)
    add(q, {"head": (L["crop_head"] * 0.4 + 1.5 * S(TAU * p * 2), 0, 0)})
    add(q, nose(t, 6.0, 2.0))
    q.update(ears(0.8))
    q.update(tail_sway(p, 1, 4))
    return q


def pose_death(t):
    """Dying: the legs give, it rolls onto its side, the legs go slack, the head drops, the eyes shut;
    it stays (1.2 s, holds). Each frame rests on its lowest point (`build`)."""
    p = t / 1.2
    fall = float(np.interp(p, [0, .3, .65, 1], [0, 0.35, 1, 1]))
    q = {"spine_0": (0, 0, 85 * fall), "spine_1": (6 * fall, 0, 0), "head": (14 * fall, 0, -10 * fall)}
    for leg, s in (("FL", 1), ("FR", -1)):  # splayed a little: swung forward square, the leg's side lay in the neck's
        add(q, {f"leg_{leg}_upper": (-25 * fall, 0, s * 8 * fall), f"leg_{leg}_paw": (20 * fall, 0, 0)})
    for leg in ("BL", "BR"):
        add(q, {f"leg_{leg}_upper": (30 * fall, 0, 0), f"leg_{leg}_lower": (-20 * fall, 0, 0), f"leg_{leg}_paw": (25 * fall, 0, 0)})
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = tail_swing(k, 15 * fall)
    q.update(ears(0.0, back=30 * fall))
    q.update({"eyelid_L": (0, 0, 90 * fall), "eyelid_R": (0, 0, -90 * fall)})
    return q


ANIMS = {"idle": (pose_idle, 3.0), "walk": (pose_walk, 0.4), "run": (pose_run, 0.24), "sit": (pose_sit, 3.0),
         "stare": (pose_stare, 2.0), "graze": (pose_graze, 4.0), "drink": (pose_drink, 1.5), "sniff": (pose_sniff, 1.8),
         "look": (pose_look, 2.4), "scratch": (pose_scratch, 1.4), "stretch": (pose_stretch, 2.0), "sleep": (pose_sleep, 4.0),
         "alarm": (pose_alarm, 1.0), "play": (pose_play, 1.2), "groom": (pose_groom, 2.6), "nuzzle": (pose_nuzzle, 1.6),
         "flinch": (pose_flinch, 0.4), "death": (pose_death, 1.2), "watch": (pose_watch, 3.0), "approach": (pose_approach, 0.8)}
CLIPS = [c for c in ANIMS if c not in ("idle", "walk", "run", "sit")]
STANDING = ("idle", "walk", "run", "sit")
# The held ones and the approach loop; a clip played once must not (the engine loops a clip whose
# file says so, whatever the game asks); the death holds its last frame.
LOOPS = {"stare": True, "graze": True, "drink": True, "sleep": True, "watch": True, "approach": True, "death": "hold_on_last_frame"}
BLENDS = {"flinch": (0.05, 0.2), "death": (0.1, 0.0), "sleep": (0.5, 0.5), "graze": (0.3, 0.3), "watch": (0.4, 0.4),
          "approach": (0.25, 0.25), "alarm": (0.05, 0.3)}
PAWS = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
# The feet each clip stands on, its height set from them frame by frame (`build`): sat up, a hind foot
# raised, walking. `None`: every part (dying, and curled asleep, it rests on its lowest point).
PLANTED = {"watch": ["leg_BL_paw", "leg_BR_paw"], "groom": ["leg_BL_paw", "leg_BR_paw"], "graze": ["leg_BL_paw", "leg_BR_paw"],
           "scratch": ["leg_BR_paw"], "stretch": ["leg_FL_paw", "leg_FR_paw", "leg_BR_paw"], "approach": PAWS, "death": None,
           "sleep": None}
# Leaps: shifted once so they take off from and land on the ground.
LEAPS = ("alarm", "play")
BLINKS = {"idle": [1.1], "sit": [2.2], "stare": [1.3], "graze": [1.0, 2.9], "look": [1.2], "watch": [0.9, 2.3], "drink": [0.8]}
BLINK = 0.09


def eyelids(name, t, pose=None):
    """The blinks, over a clip's own lids (half shut, squeezed) where it has them."""
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    a = max(a, abs((pose or {}).get("eyelid_L", (0, 0, 0))[2]))
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}  # side eyes: the lids swing down about z


def frame_times(name, dur):
    # nibbling, the nose and the fast scratch need fine keys
    n = {"idle": 72, "sit": 60, "walk": 20, "run": 20, "graze": 80, "watch": 48, "scratch": 42, "groom": 52, "sniff": 36,
         "drink": 30, "flinch": 12, "approach": 20, "sleep": 24, "look": 36, "play": 30, "alarm": 30}.get(name, 24)
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
            if name in STANDING:
                pose.update(eyelids(name, tt))
            elif name not in ("sleep", "death"):  # those keep their eyes shut
                pose.update(eyelids(name, tt, pose))
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames}
        if name not in STANDING:
            out[name].update(loop=loops, blend=BLENDS.get(name, (0.2, 0.25)))
    # A clip goes over what plays below only on the bones it writes, and the exporter leaves out a bone
    # that stays at 0: kept at a hundredth of a degree, so the idle's and the sit's head, ears, tail and
    # legs don't show through a clip that holds them still. The eyelids are left to the idle's blinks.
    moved = {b for a in ("idle", "sit") for f in out[a]["frames"] for b, r in f["rot"].items() if any(abs(v) > 1e-6 for v in r)}
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
          "rump": lambda n: n in ("hips", "rump"), "tail": lambda n: n.startswith("tail_"),
          "front legs": lambda n: n.startswith("leg_F"), "body": lambda n: n in ("hips", "rump", "chest", "skull", "muzzle")}


def fit_poses(spec):
    """Sitting up: the hind feet and the rump on the ground, the tail laid behind, nothing below."""
    def cost(P):
        g = lowest(spec, pose_sit(0.0, dict(L, **P, sit_drop=0)), GROUPS)
        f = min(g["back paws"], g["rump"])
        return (g["back paws"] - f) ** 2 + (g["rump"] - f) ** 2 + 4 * max(0.0, f - g["tail"]) ** 2 + 2 * max(0.0, f + 0.3 - g["front legs"]) ** 2
    grid = {"sit_angle": list(range(40, 81, 5)), "sit_hip": list(range(-100, -29, 5)), "sit_knee": list(range(60, 161, 10))}
    best = fit(grid, cost)[1]
    g = lowest(spec, pose_sit(0.0, dict(L, **best, sit_drop=0)), GROUPS)
    L.update(best, sit_drop=round(float(min(g["back paws"], g["rump"])), 2))
    fit_clips(spec)


# The behaviour clips' fits (`fit_clips`): the curl asleep, and the parts the scratch, groom and
# stretch bring together.
SCRATCH = ["scr_sit", "scr_yaw", "scr_hyaw", "scr_roll", "scr_hip", "scr_out", "scr_knee", "scr_foot"]
GROOM = ["head", "up", "paw", "out"]


def scratch_pose(x):
    """Sat back by `sit` (0..1) of the fitted sit (the right hind foot planted, the forepaws off the
    ground), the chest and head bent round to the left as far as they go (`pokes`), the head tipped
    toward the left hind foot, which is brought forward and up (hip, out, knee, foot). (Bowing the
    shoulders to bring the head down to the foot gained half a unit and showed the chest through
    the hips.)"""
    sit, yaw, hyaw, roll, hip, out, knee, foot = x
    sit = min(max(sit, 0.0), 1.0)  # (sat back further, the tail's root shows through the hips)
    q = seated(sit)
    add(q, {"spine_1": (4 * sit, yaw, 0), "head": (L["sit_angle"] * 0.25 * sit, hyaw, roll),
            "leg_BL_upper": (hip, 0, out), "leg_BL_lower": (knee, 0, 0), "leg_BL_paw": (foot, 0, 0)})
    return q


def curl(P):
    """Curled asleep (`P`: the fit): the legs folded under, the back rounded, the head tucked down,
    the tail wrapped round the left side; the root down so it rests on the ground."""
    q = {"root": (0, -P["curl_drop"], 0), "spine_0": (P["curl_back"], 0, 0), "spine_1": (P["curl_chest"], 0, 0),
         "head": (P["curl_head"], 0, 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (P["curl_front"] - P["curl_back"] - P["curl_chest"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["curl_front"], 0, 0)
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (P["curl_hip"] - P["curl_back"], 0, 0)
        q[f"leg_{leg}_lower"] = (P["curl_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["curl_hip"] - P["curl_knee"], 6 if leg == "BL" else -6, 0)  # (square, folded under the thigh, their sides met: flicker)
    for k in range(len(TAIL)):  # round the side, along the ground
        q[f"tail_{k}"] = tail_swing(k, 35 if k == 0 else 20, (P.get("curl_tail", 0.0) - P["curl_back"] if k == 0 else 0, 0, 0))
    return q


def fit_clips(spec):
    """The clips' fits: the muzzle at the ground (graze, drink, sniff, nuzzle, the approach's head), the
    sentinel's sit, the toe at the ear (scratch), the forepaws at the cheeks and ears (groom), the bow
    and the hind leg's push (stretch), the curl asleep."""
    from ground import world
    from scipy.optimize import minimize
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}
    every = {"all": lambda n: True}

    def at(W, bone, p):
        R, t = W[bone]
        return R @ (np.array(p, float) - piv[bone]) + t

    def bounded(x, lim):
        return sum(max(0.0, lo - v, v - hi) ** 2 for v, (lo, hi) in zip(x, lim))

    snout = {"muzzle": lambda n: n in ("muzzle", "skull"), "front paws": GROUPS["front paws"]}
    # The muzzle 0.3 over the ground: the head bowed at most 24 (past that a nibble or a turn shows
    # the neck through the skull), the shoulders lowered as needed, the forelegs set back to keep
    # the paws on the ground; drinking, the head dipped to 0.05.
    def crop_pose(P, dip=0.0):
        keep = dict(L)
        L.update(P)
        q = add(head_low(1.0), {"head": (dip, 0, 0)})
        L.clear()
        L.update(keep)
        return q

    def crop_cost(P):
        g = lowest(spec, crop_pose(P), snout)
        return (g["muzzle"] - 0.3) ** 2 + 10 * g["front paws"] ** 2 + 1e-4 * (P["crop_chest"] ** 2 + P["crop_leg"] ** 2)
    L.update(fit({"crop_head": list(range(14, 25)), "crop_chest": list(range(0, 15)), "crop_leg": list(range(0, 41, 2))}, crop_cost)[1])
    L["drink_dip"] = fit({"drink_dip": [x / 4 for x in range(0, 25)]},
                         lambda P: (lowest(spec, crop_pose({}, P["drink_dip"]), snout)["muzzle"] - 0.05) ** 2)[1]["drink_dip"]

    # The scratch: the left hind foot's toe behind the left ear, at the back of the head's side (the
    # chest and head bend round only so far: the ear's root itself is out of reach); the leg kept
    # outside the hips and chest, the head off the ground.
    toe0 = (BACK_X, 0.3, BACK_Z + G["foot_len"])
    ear0 = (SKULL[1][0] + 0.3, 3.0, SKULL[0][2] + 0.4)
    lim = [(0, 1), (0, 14), (0, 7), (-14, 0), (-170, 40), (0, 45), (-160, 160), (-120, 120)]
    parts = {"head": lambda n: n in ("muzzle", "skull"), "BR": lambda n: n == "leg_BR_paw", "front": GROUPS["front paws"],
             "leg": lambda n: n in ("leg_BL_paw", "leg_BL_lower")}

    def scratch_cost(x):
        q = scratch_pose(x)
        W = world(spec["bones"], q)
        toe, ear = at(W, "leg_BL_paw", toe0), at(W, "ear_L", ear0)
        knee = at(W, "leg_BL_lower", (BACK_X, 1.85, BACK_Z))
        ankle = at(W, "leg_BL_paw", (BACK_X, 0.53, BACK_Z))
        g = lowest(spec, q, parts)
        pen = sum(max(0.0, G["hips"] + 0.3 - p[0]) ** 2 for p in (knee, ankle, (knee + ankle) / 2)) + bounded(x, lim)
        pen += max(0.0, g["BR"] + 1.0 - g["head"]) ** 2 + max(0.0, g["BR"] + 0.9 - g["leg"]) ** 2 + max(0.0, g["BR"] - g["front"]) ** 2
        return float(np.sum((toe - ear) ** 2) + 10 * pen + 0.00005 * np.sum(np.square(x)))
    best = min((minimize(scratch_cost, g, method="Nelder-Mead", options={"maxiter": 6000, "maxfev": 6000})
                for g in ([1, 10, 5, -15, -100, 30, 60, 20], [0.4, 14, 7, -20, -60, 40, -40, 60], [0, 14, 7, -18, -44, 45, 20, 23])),
               key=lambda r: r.fun)
    L.update({k: round(float(v), 1) for k, v in zip(SCRATCH, best.x)})
    W = world(spec["bones"], scratch_pose(best.x))
    L["scr_miss"] = round(float(np.linalg.norm(at(W, "leg_BL_paw", toe0) - at(W, "ear_L", ear0))), 2)

    # The face-wash: sat, the forepaw's tip at the cheek (gf_), then over the ear's root (ge_).
    paw0 = (FRONT_X, 0.3, FRONT_Z + 0.9)
    lim = [(-10, 8), (-170, 0), (-90, 120), (8, 30)]  # (splayed under 8, the paw's side lay in the skull's: flicker)

    def groom_fit(target, guess):
        def cost(x):
            hd, up, paw, out = x
            q = seated(1.0)
            add(q, {"head": (L["sit_angle"] * 0.5 + hd, 0, 0)})
            for leg, s in (("FL", 1), ("FR", -1)):
                q[f"leg_{leg}_upper"] = (up, s * 6, s * out)
                q[f"leg_{leg}_paw"] = (paw, 0, 0)
            W = world(spec["bones"], q)
            return float(np.sum((at(W, "leg_FL_paw", paw0) - at(W, "head", target)) ** 2) + 10 * bounded(x, lim) + 0.0001 * np.sum(np.square(x)))
        return minimize(cost, guess, method="Nelder-Mead", options={"maxiter": 4000, "maxfev": 4000}).x
    for pre, target, guess in (("gf_", (SKULL[1][0] + 0.1, 2.6, 4.2), [5, -90, 40, 10]),
                               ("ge_", (EAR_X + 0.4, 3.9, sum(EAR_Z) / 2 - 0.5), [8, -130, 40, 15])):
        L.update({pre + n: round(float(v), 1) for n, v in zip(GROOM, groom_fit(target, guess))})

    # The bow: the shoulders lowered, the forepaws reached forward until they are on the ground with
    # the hind feet (a long paw reached out a little dips its heel: the reach alone can't lower them).
    L["st_chest"] = 14

    def bow_cost(P):
        q = {"spine_1": (L["st_chest"], 0, 0)}
        for leg in ("FL", "FR"):
            q[f"leg_{leg}_upper"] = (-P["st_reach"] - L["st_chest"], 0, 0)
            q[f"leg_{leg}_paw"] = (P["st_reach"], 0, 0)
        g = lowest(spec, q, GROUPS)
        return (g["front paws"] - g["back paws"]) ** 2
    L.update(fit({"st_reach": list(range(0, 81))}, bow_cost)[1])
    # The hind leg's push: the ankle as far back as it goes with the leg just clear of the ground.
    leg = {"paw": lambda n: n == "leg_BL_paw", "shin": lambda n: n == "leg_BL_lower"}

    def push_cost(P):
        q = {"leg_BL_upper": (P["st_hip"], 0, 0), "leg_BL_lower": (P["st_knee"], 0, 0), "leg_BL_paw": (P["st_foot"], 0, 0)}
        g = lowest(spec, q, leg)
        ankle = at(world(spec["bones"], q), "leg_BL_paw", (BACK_X, 0.53, BACK_Z))
        return ankle[2] + 0.0005 * (P["st_hip"] + P["st_knee"] + P["st_foot"]) ** 2 + 100 * (max(0.0, 0.1 - min(g.values())) + max(0.0, g["paw"] - 0.6))
    L.update(fit({"st_hip": list(range(0, 91, 5)), "st_knee": list(range(-60, 61, 10)), "st_foot": list(range(-60, 91, 10))}, push_cost)[1])

    # Curled asleep: the belly just off the ground, the folded legs and the tucked head on it.
    def curl_cost(P):
        P = dict(P, curl_drop=0.0)
        q = curl(P)
        g = lowest(spec, q, GROUPS)
        f = lowest(spec, q, every)["all"]
        return (g["body"] - f - 0.15) ** 2 + 0.3 * (g["back paws"] - f) ** 2 + 0.3 * (g["front paws"] - f) ** 2
    grid = {"curl_back": [-10, -5, 0, 5], "curl_chest": [10, 15, 20, 25], "curl_head": [15, 20, 25, 30], "curl_front": [40, 60, 80, 100],
            "curl_hip": [-90, -70, -50, -30], "curl_knee": [65, 85, 105, 125, 145]}  # (no right angles: a shin square to the thigh lay in its plane)
    best = fit(grid, curl_cost)[1]
    L.update(best, curl_drop=0.0)
    L["curl_drop"] = round(float(lowest(spec, curl(L), every)["all"]), 2)
    tail = {"tail": GROUPS["tail"]}  # the tail lowered to lie along the ground
    L.update(fit({"curl_tail": list(range(-40, 21))}, lambda P: (lowest(spec, curl(dict(L, **P)), tail)["tail"] - 0.05) ** 2)[1])
    print("clips fitted:", {k: v for k, v in L.items() if k.startswith(("crop", "drink", "scr", "gf", "ge", "st_", "curl"))},
)


# ------------------------------------------------------------------ build


def build():
    B = bones()
    E = elements()
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"] and not e["rot"]]
    LAYERS["rec"] = Recorder(layer_bounds(E, 1.0))
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
    spec = {"name": "kryptobaatar", "texture": [sheet.shape[1], sheet.shape[0]],
            "bones": [{"name": n, "parent": p, "pivot": [float(v) for v in pv]} for n, p, pv in B], "elements": E}
    fit_poses(spec)
    spec["animations"] = animations([n for n, _, _ in B])
    # The clips that sit up, lift a foot or walk stand on the feet that stay down, frame by frame;
    # dying and curled asleep, each frame rests on its lowest point; the leaps shifted once, so they
    # take off from and land on the ground.
    everything = [e["name"] for e in E]
    for clip, feet in PLANTED.items():
        spec["animations"][clip] = grounded(spec, spec["animations"][clip], feet or everything)
    for clip in LEAPS:
        spec["animations"][clip] = grounded(spec, spec["animations"][clip], PAWS, True)
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
