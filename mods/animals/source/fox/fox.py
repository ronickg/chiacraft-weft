#!/usr/bin/env python3
"""The fox: one model for the red fox and the Arctic fox, made the way Hytale makes creatures (boxes
and a hand-painted texture) with the blocky-model skill, from research alone (README.md): its
colors are rules (see `area`), painted in a red fox's.

    ~/.venvs/blocky/bin/python fox.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each; painted at 32 texels a block, `TX`), y up, the fox faces +z. A red fox
stands about 38 cm at the shoulder and is 65 cm long with a 45 cm tail: here the withers are 26 up,
the body 44 from nose to rump, the brush 26; the head, eyes and ears a little bigger than life. The
Arctic fox is the same model drawn smaller with shorter ears by its genes (`genetics/arctic_fox.json`).
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
from house import TX  # noqa: E402  texels a unit: 32 a block (the face is 4 by 4 texels, over house.MIN_FACE)

OUT = os.environ.get("FOX_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

COAT, BACK, WHITE, BLACK, NOSE, MOUTH, EARIN = range(7)
# A red fox's: COAT the orange-red, BACK a little darker along the spine, WHITE the chin, throat,
# chest, belly and the brush's tip, BLACK the "stockings", the backs of the ears and the line of the
# mouth, NOSE its tip, EARIN the ears' pale fur inside.
BASE = [(206, 106, 48), (170, 84, 42), (236, 228, 214), (54, 40, 36), (44, 32, 30), (120, 60, 62), (226, 206, 184)]
if os.environ.get("FOX_COAT") == "arctic":  # an Arctic fox's winter white, to compare in the viewer (the game paints coats from genes)
    BASE[COAT], BASE[BACK], BASE[BLACK] = (230, 228, 222), (214, 212, 206), (214, 210, 202)
SHADOW = [0.02, 0.02, 0.07, 0.95, 0.95, 0.95, 0.97]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
FUR = (COAT, BACK, WHITE)
ROLES = [("coat", 0, 0), ("coat", 0, -1.0), ("coat", 1, 0), ("coat", 0, -4.0), "fixed", "fixed", "fixed"]

# A fox's eyes: amber round a tall oval pupil, a catchlight.
EYES.update({
    "fox_amber": dict(pattern=["AHA",
                               "APA",
                               "aPa"], colors={"A": (204, 136, 40), "a": (226, 170, 64), "P": (18, 12, 12), "H": (240, 236, 220)},
                      halo="round"),
    # At 32 a block the face is 4 texels by 4, two rows above the muzzle: an eye is 1 by 2 at each
    # edge, two texels of fur between: amber over the dark pupil (the iris recolored per fox).
    "fox_px": dict(pattern=["A", "P"], colors={"P": (18, 12, 12), "A": (214, 146, 44)}, halo="none", iris="A"),
    "fox_px_up": dict(pattern=["P", "A"], colors={"P": (18, 12, 12), "A": (214, 146, 44)}, halo="none", iris="A"),
    "fox_px_lit": dict(pattern=["a", "P"], colors={"P": (18, 12, 12), "a": (236, 180, 76)}, halo="none", iris="a"),
})
EYE_STYLE = os.environ.get("FOX_EYES", "fox_px_up")  # the pupil reads as an eye from afar
# Six texels wide (Ronald, 11-05: at four the cheeks beside the two-texel muzzle were one texel, a
# line; now two-texel cheek ruffs frame it).
SKULL = ((-6.0, 24.0, 12.0), (6.0, 31.0, 20.0))
EYE_X, EYE_Y = 5.0, 29.0  # on the face, above the cheeks: at 6 texels across, the outer columns
# Where the nose is painted on the muzzle (the box it was: a pad a texel proud drew a rim; rule 4).
NOSE_X, NOSE_Y, NOSE_Z = 1.1, 26.1, 25.6
LID_Z = SKULL[1][2] + 0.35  # the eyelids' hinge line, 0.35 in front of the face: closed, they sit over the eyes
LASH = (52, 30, 22)


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
    """(x0, x1, top, bottom) of the eye at `cx` on the face, rounded as `stamp_eyes` places it."""
    pat = EYES[EYE_STYLE]["pattern"]
    ph, pw = len(pat), len(pat[0])
    w, h, su, sv = skull_grid()
    u0, v0 = eye_corner(*eye_center(cx), pat, w)
    x0, top = SKULL[0][0] + u0 * su, SKULL[1][1] - v0 * sv
    return x0, x0 + pw * su, top, top - ph * sv

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


def rod(name, bone, start, rot, length, w, d=None, back=0.4):
    """A box along +y from `start` turned by `rot` (about `start`), `w` across and `d` deep."""
    s = np.array(start, float)
    d = w if d is None else d
    e = el(name, bone, (s[0] - w / 2, s[1] - back, s[2] - d / 2), (s[0] + w / 2, s[1] + length, s[2] + d / 2), rot=rot, origin=s)
    return e, s + rotm(rot) @ np.array([0, length - 0.6, 0])


LEGS = ("FL", "FR", "BL", "BR")
FRONT_X, FRONT_Z, BACK_X, BACK_Z = 2.4, 6.0, 2.8, -14.0
NECK0, NECK_PITCH, NECK_LEN = np.array([0.0, 20.0, 8.0]), -43.0, 11.0  # its end in the middle of the skull
HEAD = NECK0 + (NECK_LEN - 2.5) * np.array([0, math.sin(math.radians(-NECK_PITCH)), math.cos(math.radians(-NECK_PITCH))])
EAR = [(30.6, 33.4, 2.3), (33.4, 36.0, 1.5), (36.0, 38.0, 0.7)]  # y0, y1, half width: a tall triangle
EAR_X, EAR_Z = 2.5, (14.6, 15.8)
TAIL = [(-12, 8.0, 6.0), (-12, 9.0, 5.2), (-12, 9.0, 4.4)]  # pitch (down), length, width: the brush
TAIL0 = np.array([0.0, 23.5, -19.0])


def tail_points():
    pts, p = [TAIL0.copy()], TAIL0.copy()
    for pitch, length, _ in TAIL:
        a = math.radians(pitch)
        p = p + np.array([0, math.sin(a), -math.cos(a)]) * length
        pts.append(p.copy())
    return pts


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 22, -12)), ("spine_1", "spine_0", (0, 22, -5)),
         ("neck", "spine_1", tuple(NECK0)), ("head", "neck", tuple(HEAD)), ("jaw", "head", (0, 24.5, 19.8))]  # the hinge in the jaw's top
    for s, n in ((1, "L"), (-1, "R")):
        B.append((f"ear_{n}", "head", (s * EAR_X, 30.8, sum(EAR_Z) / 2)))
        x0, x1, top, _ = eye_box(s * EYE_X)
        B.append((f"eyelid_{n}", "head", ((x0 + x1) / 2, top, LID_Z)))
    pts = tail_points()
    B += [(f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", tuple(pts[k])) for k in range(len(TAIL))]
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x = FRONT_X * s
            B += [(f"leg_{leg}_upper", "spine_1", (x, 17, FRONT_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 9.5, FRONT_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 2.2, FRONT_Z))]
        else:
            x = BACK_X * s
            B += [(f"leg_{leg}_upper", "spine_0", (x, 21, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 11.5, BACK_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 2.2, BACK_Z))]
    return B


def elements():
    E = [el("hips", "spine_0", (-4.4, 17, -20), (4.4, 27, -3)),
         el("chest", "spine_1", (-5.5, 15, -6.5), (5.5, 27.5, 12)),  # half a unit wider than the thighs, which meet it running
         el("skull", "head", *SKULL),
         # Long and slender, to the nose's tip (painted on); muzzle and jaw each two texels tall (rule 7:
         # at 2.6 and 1.8 they were boxes a texel thick), the muzzle on the jaw's top.
         el("muzzle", "head", (-2.0, 24.6, 19.5), (2.0, 27.6, 26.6)),
         el("mouth", "head", (-1.0, 23.6, 19.9), (1.0, 25.0, 24.6)),
         el("jaw", "jaw", (-1.5, 21.6, 18.8), (1.5, 24.6, 25.2))]  # shorter than the muzzle
    neck, _ = rod("neck", "neck", tuple(NECK0), (NECK_PITCH + 90, 0, 0), NECK_LEN, 5.6, 6.2, back=3.0)
    E.append(neck)
    for s, n in ((1, "L"), (-1, "R")):
        cx = EAR_X * s
        rot, origin = (-8, 0, -s * 12), (cx, 30.8, sum(EAR_Z) / 2)
        for k, (y0, y1, hw) in enumerate(EAR):
            E.append(el(f"ear_core_{n.lower()}{k}", f"ear_{n}", (cx - hw, y0, EAR_Z[0]), (cx + hw, y1, EAR_Z[1]), rot=rot, origin=origin))
        zc = sum(EAR_Z) / 2
        # The eyelid: a quad the eye's size lying flat inside the head, hinged at the eye's top; a
        # blink swings it down over the eye (-90 about x). Tilted a few degrees into the head, so it
        # is never in the skull's top plane.
        x0, x1, top, bottom = eye_box(s * EYE_X)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (x0, top, LID_Z - (top - bottom)), (x1, top, LID_Z), quad="down", rot=(-4, 0, 0),
                    origin=((x0 + x1) / 2, top, LID_Z)))
        E.append(el(f"fringe_ear_{n.lower()}", f"ear_{n}", (cx - 2.8, 30.4, zc), (cx + 2.8, 38.6, zc), quad="south", rot=rot,
                    origin=origin, cut="ear"))
    pts = tail_points()
    for k, (pitch, length, w) in enumerate(TAIL):
        p = pts[k]
        E.append(el(f"tail_{k}", f"tail_{k}", (p[0] - w / 2, p[1] - w / 2, p[2] - length), (p[0] + w / 2, p[1] + w / 2, p[2] + 0.8),
                    rot=(pitch, 0, 0), origin=tuple(p)))  # each 0.4 a side thinner
    # Legs two texels thick or more (rule 7: at 2.2 a shank was a texel, a line under the upper leg),
    # the paw a band under the shank half a unit round it (rule 8; turning, it keeps off the shank's
    # sides), the upper leg half a unit round the shank. Their sides (inner, outer: from the middle)
    # half a unit or more from every side they meet in a pose (front against back legs running, a
    # shank against the hips lying; boxcheck) and from the chest's and hips'.
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        side = lambda i, o: sorted((s * i, s * o))  # noqa: E731
        if leg[0] == "F":
            z = FRONT_Z
            (x0, x1) = side(0.3, 4.4)
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x0, 9, z - 2.4), (x1, 18.5, z + 2.4)))
            (x0, x1) = side(0.8, 3.9)
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x0, 2.4, z - 1.9), (x1, 10, z + 1.9)))
            (x0, x1) = side(0.25, 3.35)
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x0, 0, z - 2.45), (x1, 2.4, z + 2.6)))
        else:
            z = BACK_Z
            (x0, x1) = side(0.8, 4.95)  # the haunch, its outer side half a unit past the hips'
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (x0, 11, z - 3.6), (x1, 25, z + 3.4)))
            (x0, x1) = side(1.35, 4.45)
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x0, 2.4, z - 1.95), (x1, 12, z + 1.95)))
            (x0, x1) = side(1.85, 4.95)
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x0, 0, z - 2.5), (x1, 2.4, z + 2.6)))
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
    w, h = int(round(TX * abs(size @ du))), int(round(TX * abs(size @ dv)))  # texels, at 32 a block
    if abs(size @ du) > 0 and abs(size @ dv) > 0:  # a small face is still a texel
        w, h = max(1, w), max(1, h)
    return np.array(corner, float), du, dv, w, h, n


def area(e, face, p, uu, vv, h):
    """A red fox: orange-red, a little darker along the back; white chin, throat, chest and belly;
    black "stockings" on the legs and black backs to the ears; the brush tipped white; white cheeks
    below the eyes."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, COAT)
    side = face in ("east", "west")
    t = vv / max(h, 1)
    if name in ("hips", "chest"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = WHITE
        else:
            a = np.where(t < 0.15, BACK, np.where(y < 18.4, WHITE, COAT))  # the white belly's edge: 2 texels of the chest at 32
            if name == "chest" and face == "south":
                a = np.where(np.abs(x) < 3.0 - 0.15 * np.maximum(y - 20, 0), WHITE, a)  # the white bib
    elif name == "neck":
        if face in ("south", "down"):
            a[:] = WHITE  # the throat
        elif face in ("north", "up"):
            a[:] = BACK
    elif name == "skull":
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = WHITE
        elif face == "south":
            a = np.where((y < 27.6) & (np.abs(x) > 1.8), WHITE, a)  # the white cheeks
        elif side:
            a = np.where((y < 26.4) & (z > 15.5), WHITE, a)
    elif name == "muzzle":
        if face == "up":
            a[:] = COAT
        else:
            a = np.where(y < 26.0, WHITE, COAT)
        # The nose, painted: on the front and the front edge of the top.
        if face == "south":
            a = np.where((np.abs(x) <= NOSE_X + 0.05) & (y >= NOSE_Y), NOSE, a)
        elif face == "up":
            a = np.where((np.abs(x) <= NOSE_X + 0.05) & (z >= NOSE_Z), NOSE, a)
    elif name in ("mouth",):
        a[:] = MOUTH
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else WHITE
    elif name.startswith("ear_core"):
        a[:] = BLACK if face == "north" else COAT  # black backs
        if face == "south":
            cx = e["origin"][0]
            a = np.where(np.abs(x - cx) < (37.4 - y) * 0.42, EARIN, a)
        a = np.where(y > 36.4, BLACK, a)
    elif name.startswith("tail_"):
        k = int(name.split("_")[1])
        a[:] = BACK if face == "up" else COAT
        if k == len(TAIL) - 1:
            pts = tail_points()
            along = (np.array(pts[k]) - p) @ np.array([0, -math.sin(math.radians(TAIL[k][0])), math.cos(math.radians(TAIL[k][0]))])
            a = np.where(along > TAIL[k][1] - 4.2, WHITE, a)  # the white tip
    elif name.startswith("thigh_"):
        a[:] = COAT
        if face == "up":
            a[:] = BACK
        if face == "down":
            a[:] = WHITE
    elif name.startswith("leg_"):
        a[:] = BLACK if (y.max() < 14 or name.endswith("_paw")) else COAT  # the black stockings
        if name.startswith("leg_F_") or (name.startswith("leg_F") and y.max() > 14):
            a = np.where(y > 12.5, COAT, BLACK)
    return a


def cut_mask(e, p):
    """The ear's outline: a tall triangle, a few hairs on its edge."""
    x, y = p[..., 0], p[..., 1]
    cx = e["origin"][0]
    half = np.clip((38.4 - y) * 0.36, 0, 2.7) + 0.3 * ((np.floor(y * 1.4) % 3) == 0)
    return (np.abs(x - cx) <= half) & (y >= 30.4)


# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)


def layer_props(e, face, p, a):
    """White where this red fox is white (chin, throat, bib, belly, the brush's tip) and up from it;
    the stockings and ear backs are its points (a cross or silver fox darkens them further); marks
    (the cross fox's dark cross) over the back and shoulders."""
    n, y = e["name"], p[..., 1]
    lo, hi = e["lo"][1], e["hi"][1]
    low = 1 - _ramp(y, lo, max(hi, lo + 1e-3))
    white = 0.55 * low if n in ("chest", "hips", "neck") else np.full(y.shape, 0.25)
    white = np.maximum(white, np.where(a == WHITE, 0.85, 0))
    points = np.where(a == BLACK, 1.0, 0.0)
    pattern = np.full(y.shape, 1.0 if n in ("hips", "chest", "neck") else 0.3)
    return {"white": white, "points": points, "pattern": pattern}


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
    if e["name"].startswith("eyelid"):  # seen only closed, as the face round the eyes
        a, light_face = np.full(a.shape, COAT), "south"
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    stacked = e["name"].startswith("ear_core")
    down = np.clip((38.2 - p[..., 1]) / 7.6, 0, 1) if stacked else None
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=FUR, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, down=down, tuft=e["name"] not in ("skull", "muzzle"))
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"] == "ear":
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "skull" and face == "south":
        stamp_eyes(img, [eye_center(cx) for cx in (-EYE_X, EYE_X)], EYE_STYLE)
    if e["name"].startswith("eyelid"):  # closed, the lash line is its lower edge
        img[h - 1, :, :3] = LASH
    if e["name"] == "muzzle" and face in ("east", "west") and h >= 3:  # the dark line of the mouth (1-2 texels tall at 32:
        # a darkened row is half the muzzle or all of it; the mouth box under it draws the line)
        img[h - 1, :, :3] = (img[h - 1, :, :3] * 0.6).astype(np.uint8)
    if e["name"].endswith("_paw") and face == "south" and w >= 5:  # four toes (a paw is 2 texels at 32)
        for c in range(1, w - 1, 2):
            img[:, c, :3] = (img[:, c, :3] * 0.75).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"lie_front": 80, "lie_hip": -70, "lie_knee": 140, "lie_drop": 10}


def tail_swing(k, angle, base=(0, 0, 0)):
    pitch = math.radians(TAIL[k][0])
    return swing((0, math.sin(pitch), -math.cos(pitch)), angle, base)


def tail_sway(p, n, amp, base=None):
    base = base or [(0, 0, 0)] * len(TAIL)
    return {f"tail_{k}": tail_swing(k, amp * S(TAU * p * n - 0.7 * k), base[k]) for k in range(len(TAIL))}


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


def pose_idle(t):
    """Alert: the ears turning to sounds, the head following, the brush swaying (4 s)."""
    p = t / 4.0
    br = S(TAU * p * 2)
    turn = float(np.interp(p, [0, .2, .3, .6, .7, 1], [0, 0, 18, 18, -12, 0]))
    tw = max(0.0, S(TAU * (p - 0.45) * 10)) if 0.45 < p < 0.5 else 0.0
    q = {"root": (0, 0.1 * br, 0), "spine_1": (0.5 * br, 0, 0), "head": (2 * S(TAU * p), turn, 0),
         "ear_L": (0, -18 * tw, 0), "ear_R": (0, 0, 2 * br)}
    q.update(tail_sway(p, 1, 8))
    return q


def pose_walk(t):
    """Trotting: diagonal pairs (0.6 s)."""
    p = t / 0.6
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 22, 34)
    q.update({"root": (0, 0.3 * abs(S(TAU * p * 2)), 0), "head": (2 * S(TAU * p * 2), 0, 0)})
    q.update(tail_sway(p, 1, 6))
    return q


def pose_run(t):
    """Galloping, the brush streaming level (0.36 s)."""
    p = t / 0.36
    q = gait(p, (0.0, 0.1, 0.5, 0.6), 36, 56)
    c = S(TAU * p)
    q.update({"root": (0, 1.2 * max(0.0, S(TAU * p + 0.6)), 0), "spine_0": (-5 * c, 0, 0), "spine_1": (7 * c, 0, 0),
              "neck": (-8 - 4 * c, 0, 0), "head": (8, 0, 0), "ear_L": (-25, 0, 0), "ear_R": (-25, 0, 0)})
    q.update(tail_sway(p, 1, 3, [(5, 0, 0)] * len(TAIL)))
    return q


def pose_lie(t, P=L):
    """Lying down, the legs folded under, the head low on its paws and the brush along its side (5 s)."""
    p = t / 5.0
    br = S(TAU * p * 2)
    q = {"root": (0, -P["lie_drop"] + 0.1 * br, 0), "spine_1": (0.5 * br, 0, 0), "neck": (22, 0, 0), "head": (-16, 6 * S(TAU * p), 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (P["lie_front"], 0, 0)
        q[f"leg_{leg}_lower"] = (-74, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_front"] + 74 - 4, 0, 0)
    for leg, s in (("BL", 1), ("BR", -1)):
        q[f"leg_{leg}_upper"] = (P["lie_hip"], 0, s * 6)
        q[f"leg_{leg}_lower"] = (P["lie_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_hip"] - P["lie_knee"], 0, 0)
    curl = [(8, 28, 0), (4, 12, 0), (2, 10, 0)]  # the brush brought round along one side
    q.update({f"tail_{k}": tail_swing(k, 4 * S(TAU * p) if k == 2 else 0, curl[k]) for k in range(len(TAIL))})
    return q


def pose_pounce(t):
    """The mousing pounce: crouched and still, a leap up and over, the front paws stabbing down (1 s)."""
    p = t / 1.0
    crouch = float(np.interp(p, [0, .15, .4, .45, 1], [0, 1, 1, 0, 0]))
    up = float(np.interp(p, [0, .45, .6, .75, .85, 1], [0, 0, 1, 0.3, 0, 0]))
    dive = float(np.interp(p, [0, .55, .72, .85, 1], [0, 0, 1, 0.6, 0]))
    q = pose_idle(0)
    q.update({"root": (0, -2.0 * crouch + 6.0 * up, 0), "spine_0": (-18 * up + 10 * dive, 0, 0), "spine_1": (6 * crouch + 22 * dive, 0, 0),
              "neck": (10 * crouch + 20 * dive, 0, 0), "head": (6 * dive, 0, 0), "ear_L": (6, 0, 0), "ear_R": (6, 0, 0)})
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (-30 * up - 20 * dive + 20 * crouch, 0, 0)
        q[f"leg_{leg}_lower"] = (40 * crouch + 30 * up, 0, 0)
        q[f"leg_{leg}_paw"] = (-20 * crouch, 0, 0)
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (-30 * crouch + 30 * up, 0, 0)
        q[f"leg_{leg}_lower"] = (-30 * crouch - 20 * up, 0, 0)
        q[f"leg_{leg}_paw"] = (40 * crouch, 0, 0)
    q.update(tail_sway(p, 1, 2, [(-10 * up, 0, 0)] * len(TAIL)))
    q["jaw"] = (16 * dive, 0, 0)
    return q


def pose_bite(t):
    """A bite: drawn back, then a quick lunge, the jaws snapping shut (0.5 s)."""
    p = t / 0.5
    q = pose_idle(0)
    d = float(np.interp(p, [0, .25, .45, .6, 1], [0, -1, 1, 1, 0]))
    q.update({"spine_1": (4 * d, 0, 0), "neck": (10 * d, 0, 0), "head": (4 * d, 0, 0),
              "jaw": (float(np.interp(p, [0, .2, .4, .48, 1], [0, 18, 26, 0, 0])), 0, 0)})
    return q


# The behaviour clips (`mods/animals/src/anim.rs`), shared by the canids (`../_clips/canid.py`).
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "_clips"))
import canid  # noqa: E402

canid.bind(sys.modules[__name__], scratch=0.4, wag=0.5)  # a short neck, a big level brush

ANIMS = {"idle": (pose_idle, 4.0), "walk": (pose_walk, 0.6), "run": (pose_run, 0.36), "lie": (pose_lie, 5.0),
         "pounce": (pose_pounce, 1.0), "bite": (pose_bite, 0.5)}
ANIMS.update(canid.anims(howl=False))
LOOPS, BLENDS = canid.LOOPS, canid.BLENDS


BLINKS = {"idle": [1.3, 3.2], "lie": [2.1, 4.2]}
BLINKS.update(canid.BLINKS)
BLINK = 0.09


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (-a, 0, 0), "eyelid_R": (-a, 0, 0)}  # eyes on the face: the lids swing down about x


def frame_times(name, dur):
    n = {"idle": 48, "lie": 40, "walk": 24, "run": 20, "pounce": 30, "bite": 24}.get(name, canid.FRAMES.get(name, 24))
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
            if name not in canid.SHUT:
                pose.update(eyelids(name, tt))
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames, "loop": loops, "blend": BLENDS.get(name, (0.25, 0.3))}
    return out


# ------------------------------------------------------------------ fitting poses to the ground

GROUPS = {"front paws": lambda n: n in ("leg_FL_paw", "leg_FR_paw"), "back paws": lambda n: n in ("leg_BL_paw", "leg_BR_paw"),
          "body": lambda n: n in ("hips", "chest"), "head": lambda n: n in ("skull", "muzzle", "jaw"),
          "tail": lambda n: n.startswith("tail_")}


def fit_poses(spec):
    """Lying: the belly on the ground, the folded legs under it, the head on the paws, nothing
    below."""
    def lie_cost(P):
        g = lowest(spec, pose_lie(0.0, dict(L, **P, lie_drop=0)), GROUPS)
        f = min(g.values())
        return (g["body"] - f - 0.6) ** 2 + 0.3 * (g["front paws"] - f) ** 2 + 0.3 * (g["back paws"] - f) ** 2 + 0.2 * (g["head"] - f - 1) ** 2
    grid = {"lie_front": list(range(60, 101, 5)), "lie_hip": list(range(-100, -49, 5)), "lie_knee": list(range(115, 166, 5))}
    best = fit(grid, lie_cost)[1]
    g = lowest(spec, pose_lie(0.0, dict(L, **best, lie_drop=0)), GROUPS)
    L.update(best, lie_drop=round(float(min(g.values())), 2))
    # Head down to the ground (sniffing, drinking) and the death fall (`canid.fit`).
    canid.fit(spec, GROUPS)


# ------------------------------------------------------------------ build


def build():
    B = bones()
    E = elements()
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"] and not e["rot"]]
    LAYERS["rec"] = Recorder(layer_bounds(E, 2.0))
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
    spec = {"name": "fox", "texture": [sheet.shape[1], sheet.shape[0]],
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
