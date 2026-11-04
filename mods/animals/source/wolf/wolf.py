#!/usr/bin/env python3
"""The grey wolf (*Canis lupus*): the top hunter of the north's forests, taiga, tundra and mountains,
and the dog's ancestor, so a natural tame (`notes/ECOLOGY.md`, tier 1), made the way Hytale makes
creatures (boxes and a hand-painted texture) with the blocky-model skill, from research alone
(README.md), on the coyote's build (`../coyote/coyote.py`, itself the fox's): the fox's layout warped
(`warp`: the legs longer, everything `K` times bigger). Its colors are rules (see `area`), painted in a
grey wolf's grizzled grey.

    ~/.venvs/blocky/bin/python wolf.py      # writes out/spec.json, out/texture.png

Units are texels at the house density (64 a block, 1.56 cm each), y up, it faces +z. A grey wolf
stands about 80 cm at the shoulder and is 1.2 m long with a 45 cm tail: here the withers are 51 up,
the body 79 from the nose to the rump, the tail 33, carried low; a broad head, shorter ears than a
coyote's, big paws.
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
from paint import EYES, occlusion, pack, paint_face_smooth, stamp_eyes, tones  # noqa: E402
from rig import fit, swing  # noqa: E402
from ground import lowest  # noqa: E402

OUT = os.environ.get("WOLF_OUT") or os.path.join(HERE, "out")

# The fox's layout warped into the wolf's: below Y0 (the legs) stretched to reach D higher,
# everything above lifted by D, then all of it K times bigger.
K, Y0, D = 1.7, 14.0, 3.0


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
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

COAT, BACK, WHITE, BLACK, NOSE, MOUTH, EARIN, TAWNY = range(8)
# A grey wolf's: COAT the grizzled grey, BACK the dark saddle along the spine (black-tipped hairs),
# WHITE the cream of the muzzle, cheeks, throat, chest, belly and inner legs, BLACK the tail's tip
# and the line of the mouth, NOSE its tip, EARIN the ears' pale fur inside, TAWNY the buff of the
# legs, the ears' backs and the muzzle's top.
BASE = [(150, 146, 136), (84, 80, 76), (228, 222, 208), (46, 42, 40), (40, 32, 30), (120, 60, 62), (222, 210, 194),
        (176, 154, 120)]
SHADOW = [0.02, 0.02, 0.07, 0.95, 0.95, 0.95, 0.97, 0.02]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
FUR = (COAT, BACK, WHITE, TAWNY)
ROLES = [("coat", 0, 0), ("coat", 0, -1.0), ("coat", 1, 0), ("coat", 0, -4.0), "fixed", "fixed", "fixed", ("coat", 0, -0.25)]

# A wolf's eyes: amber-yellow round a round pupil, a catchlight, a dark rim.
EYES.update({
    "wolf_amber": dict(pattern=[".OO.",
                                   "OPHO",
                                   "APPA",
                                   ".aa."], colors={"O": (58, 44, 32), "A": (200, 146, 44), "a": (224, 178, 72), "P": (18, 12, 12),
                                                    "H": (240, 236, 220)}, halo="round"),
})
EYE_STYLE = os.environ.get("WOLF_EYES", "wolf_amber")
SKULL0 = ((-4.0, 24.0, 12.0), (4.0, 31.0, 20.0))  # the fox's space
SKULL = tuple(tuple(float(v) for v in warp(c)) for c in SKULL0)
EYE_X, EYE_Y = 2.4 * K, (29.0 + D) * K  # on the face, above the muzzle (warped)
LID_Z = SKULL[1][2] + 0.15  # the eyelids' hinge line, just in front of the face: closed, they sit over the eyes
LASH = (52, 30, 22)


def eye_box(cx):
    """(x0, x1, top, bottom) of the eye at `cx` on the face, rounded as `stamp_eyes` places it."""
    pat = EYES[EYE_STYLE]["pattern"]
    ph, pw = len(pat), len(pat[0])
    u0, top = SKULL[0][0], SKULL[1][1]
    a = int(round((cx - u0) - pw / 2))
    v0 = int(round((top - EYE_Y) - ph / 2))
    return u0 + a, u0 + a + pw, top - v0, top - v0 - ph

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
EAR = [(30.6, 33.2, 2.3), (33.2, 35.4, 1.5), (35.4, 37.0, 0.7)]  # shorter than a coyote's  # y0, y1, half width: a tall triangle
EAR_X, EAR_Z = 2.5, (14.6, 15.8)
TAIL = [(-40, 7.0, 4.8), (-40, 7.0, 4.0), (-40, 7.0, 3.2)]  # pitch (down), length, width: bushy, carried low
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
         ("neck", "spine_1", tuple(NECK0)), ("head", "neck", tuple(HEAD)), ("jaw", "head", (0, 25.1, 19.8))]
    for s, n in ((1, "L"), (-1, "R")):
        B.append((f"ear_{n}", "head", (s * EAR_X, 30.8, sum(EAR_Z) / 2)))
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
    B = [(n, p, tuple(float(v) for v in warp(pv))) for n, p, pv in B]
    for s, n in ((1, "L"), (-1, "R")):  # the eyelids on the warped face, with the eye's rounding
        x0, x1, top, _ = eye_box(s * EYE_X)
        B.append((f"eyelid_{n}", "head", ((x0 + x1) / 2, top + 0.25, LID_Z)))
    return B


def elements():
    E = [el("hips", "spine_0", (-4.4, 17, -20), (4.4, 27, -3)),
         el("chest", "spine_1", (-5.0, 15, -6.5), (5.0, 27.5, 12)),
         el("skull", "head", *SKULL0),
         el("muzzle", "head", (-2.0, 25.0, 19.5), (2.0, 27.6, 26.0)),  # long and slender
         el("nose", "head", (-1.1, 26.1, 25.6), (1.1, 27.3, 26.6)),
         el("mouth", "head", (-1.4, 24.4, 19.9), (1.4, 25.6, 24.6)),
         el("jaw", "jaw", (-1.7, 23.4, 18.8), (1.7, 25.2, 25.2))]  # shorter than the muzzle
    neck, _ = rod("neck", "neck", tuple(NECK0), (NECK_PITCH + 90, 0, 0), NECK_LEN, 5.6, 6.2, back=3.0)
    E.append(neck)
    for s, n in ((1, "L"), (-1, "R")):
        cx = EAR_X * s
        rot, origin = (-8, 0, -s * 12), (cx, 30.8, sum(EAR_Z) / 2)
        for k, (y0, y1, hw) in enumerate(EAR):
            E.append(el(f"ear_core_{n.lower()}{k}", f"ear_{n}", (cx - hw, y0, EAR_Z[0]), (cx + hw, y1, EAR_Z[1]), rot=rot, origin=origin))
        zc = sum(EAR_Z) / 2
        E.append(el(f"fringe_ear_{n.lower()}", f"ear_{n}", (cx - 2.8, 30.4, zc), (cx + 2.8, 38.6, zc), quad="south", rot=rot,
                    origin=origin, cut="ear"))
    pts = tail_points()
    for k, (pitch, length, w) in enumerate(TAIL):
        p = pts[k]
        E.append(el(f"tail_{k}", f"tail_{k}", (p[0] - w / 2, p[1] - w / 2, p[2] - length), (p[0] + w / 2, p[1] + w / 2, p[2] + (2.4 if k == 0 else 0.8)),
                    rot=(pitch, 0, 0), origin=tuple(p)))  # each 0.4 a side thinner
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z = FRONT_X * s, FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 1.6, 9, z - 1.6), (x + 1.6, 18.5, z + 1.7)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 1.1, 2, z - 1.1), (x + 1.1, 10, z + 1.1)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 1.7, 0, z - 1.5), (x + 1.7, 2.6, z + 2.4)))
        else:
            x, z = BACK_X * s, BACK_Z
            xi, xo = sorted((x - 1.6 * s, x + 2.6 * s))  # the haunch
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (xi, 11, z - 3.6), (xo, 25, z + 3.4)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 1.0, 2, z - 1.2), (x + 1.0, 12, z + 1.2)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 1.75, 0, z - 1.6), (x + 1.75, 2.6, z + 2.6)))
    for e in E:  # the fox's space, warped into the coyote's
        e["lo"], e["hi"] = [float(v) for v in warp(e["lo"])], [float(v) for v in warp(e["hi"])]
        if e["origin"]:
            e["origin"] = [float(v) for v in warp(e["origin"])]
    for s, n in ((1, "L"), (-1, "R")):
        # The eyelid: a quad the eye's size lying flat inside the head, hinged at the eye's top; a
        # blink swings it down over the eye (-90 about x). Tilted a few degrees into the head, so it
        # is never in the skull's top plane.
        # A quarter texel bigger all round than the eye: the face isn't whole texels once warped, and
        # no edge of the eye shows past a closed lid.
        x0, x1, top, bottom = eye_box(s * EYE_X)
        x0, x1, top, h = x0 - 0.3, x1 + 0.3, top + 0.25, top - bottom + 0.55
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (x0, top, LID_Z - h), (x1, top, LID_Z), quad="down", rot=(-4, 0, 0),
                    origin=((x0 + x1) / 2, top, LID_Z)))
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
    return np.array(corner, float), du, dv, max(1, int(round(abs(size @ du)))), max(1, int(round(abs(size @ dv)))), n


def area(e, face, p, uu, vv, h):
    """A grey wolf, in the fox's space (`unwarp`): grizzled grey, a dark saddle along the back; cream
    muzzle, cheeks, throat, chest and belly; the legs, the ears' backs and the muzzle's top buff;
    the tail tipped black."""
    name = e["name"]
    p = unwarp(p)
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
            a = np.where(t < 0.3, BACK, np.where(y < 17.0, WHITE, COAT))  # the dark saddle
            if name == "chest" and face == "south":
                a = np.where(np.abs(x) < 3.2 - 0.12 * np.maximum(y - 20, 0), WHITE, a)  # the pale bib
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
            a = np.where((y < 27.4) & (np.abs(x) > 2.0), WHITE, a)  # the pale cheeks
        elif side:
            a = np.where((y < 26.2) & (z > 15.5), WHITE, a)
    elif name == "muzzle":
        if face == "up":
            a[:] = TAWNY
        else:
            a = np.where(y < 26.4, WHITE, TAWNY)  # the pale muzzle
    elif name == "nose":
        a[:] = NOSE
    elif name in ("mouth",):
        a[:] = MOUTH
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else WHITE
    elif name.startswith("ear_core"):
        a[:] = TAWNY if face == "north" else COAT  # tawny backs
        if face == "south":
            cx = e["origin"][0] / K
            a = np.where(np.abs(x - cx) < (38.0 - y) * 0.42, EARIN, a)
        a = np.where(y > 37.4, BACK, a)
    elif name.startswith("tail_"):
        k = int(name.split("_")[1])
        a[:] = BACK if face in ("up", "north") else COAT
        if k == 0 and face == "up":
            a[:] = BLACK  # the dark spot over the tail gland
        if k == len(TAIL) - 1:
            pts = tail_points()
            along = (np.array(pts[k]) - p) @ np.array([0, -math.sin(math.radians(TAIL[k][0])), math.cos(math.radians(TAIL[k][0]))])
            a = np.where(along > TAIL[k][1] - 3.6, BLACK, a)  # the black tip
    elif name.startswith("thigh_"):
        a[:] = COAT
        if face == "up":
            a[:] = BACK
        if face == "down":
            a[:] = WHITE
        a = np.where((y < 14.0) & (face != "down"), TAWNY, a)
    elif name.startswith("leg_"):
        a[:] = TAWNY
        if name.endswith("_upper") and face == "south":
            a = np.where(y < 13.0, TAWNY, COAT)
        if name.startswith("leg_F") and name.endswith("_lower") and face == "south":
            a = np.where(np.abs(x - x.mean()) < 0.4, BACK, a)  # a darker line down the foreleg
    return a


def cut_mask(e, p):
    """The ear's outline: a tall triangle, a few hairs on its edge (in the fox's space)."""
    p = unwarp(p)
    x, y = p[..., 0], p[..., 1]
    cx = e["origin"][0] / K
    half = np.clip((39.0 - y) * 0.34, 0, 2.7) + 0.3 * ((np.floor(y * 1.4) % 3) == 0)
    return (np.abs(x - cx) <= half) & (y >= 30.4)


# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)


def layer_props(e, face, p, a):
    """White where this wolf is pale (muzzle, throat, bib, belly) and up from it (an Arctic wolf is
    white all over); the buff legs, ears' backs and muzzle its points; marks (the grizzling, the
    saddle) over the back and shoulders."""
    n, y = e["name"], unwarp(p)[..., 1]
    lo, hi = unwarp(np.array(e["lo"]))[1], unwarp(np.array(e["hi"]))[1]
    low = 1 - _ramp(y, lo, max(hi, lo + 1e-3))
    white = 0.55 * low if n in ("chest", "hips", "neck") else np.full(y.shape, 0.25)
    white = np.maximum(white, np.where(a == WHITE, 0.85, 0))
    points = np.where(a == TAWNY, 1.0, 0.0)
    pattern = np.full(y.shape, 1.0 if n in ("hips", "chest", "neck") else 0.3)
    return {"white": white, "points": points, "pattern": pattern}


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
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "jaw", "tail")):
        light_face = "north"
    a = area(e, face, p, uu, vv, h)
    if e["name"].startswith("eyelid"):  # seen only closed, as the face round the eyes
        a, light_face = np.full(a.shape, COAT), "south"
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    stacked = e["name"].startswith("ear_core")
    down = np.clip((38.8 - unwarp(p)[..., 1]) / 8.2, 0, 1) if stacked else None
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=FUR, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, down=down, tuft=e["name"] not in ("skull", "muzzle"))
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"] == "ear":
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "skull" and face == "south":
        u0, top = SKULL[0][0], SKULL[1][1]
        stamp_eyes(img, [(cx - u0, top - EYE_Y) for cx in (-EYE_X, EYE_X)], EYE_STYLE)
    if e["name"].startswith("eyelid"):  # closed, the lash line is its lower edge
        img[h - 1, :, :3] = LASH
    if e["name"] == "muzzle" and face in ("east", "west"):  # the dark line of the mouth
        img[h - 1, :, :3] = (img[h - 1, :, :3] * 0.6).astype(np.uint8)
    if e["name"] == "nose" and face == "south":  # nostrils as slits
        img[h // 2, [0, w - 1], :3] = (24, 18, 18)
    if e["name"].endswith("_paw") and face == "south":  # four toes
        for c in range(1, w - 1, 2):
            img[:, c, :3] = (img[:, c, :3] * 0.75).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"lie_front": 80, "lie_hip": -70, "lie_knee": 140, "lie_drop": 10, "sit_angle": 35, "sit_hip": -80, "sit_knee": 140, "sit_lean": -6,
     "sit_drop": 6}


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
    """Alert, the ears turning to sounds and the head following, the tail swaying; then the head
    thrown back in a long howl, the mouth rounding and closing slowly (8 s)."""
    p = t / 8.0
    br = S(TAU * p * 4)
    turn = float(np.interp(p, [0, .1, .15, .3, .35, .5, 1], [0, 0, 18, 18, -12, 0, 0]))
    tw = max(0.0, S(TAU * (p - 0.22) * 20)) if 0.22 < p < 0.245 else 0.0
    howl = float(np.interp(p, [0, .58, .64, .9, .96, 1], [0, 0, 1, 1, 0, 0]))
    yip = howl * (0.7 + 0.3 * max(0.0, S(TAU * (p - 0.64) * 2)))
    q = {"root": (0, 0.1 * br, 0), "spine_1": (0.5 * br - 6 * howl, 0, 0), "neck": (-40 * howl, 0, 0),
         "head": (2 * S(TAU * p * 2) - 14 * howl, turn, 0), "jaw": (22 * yip, 0, 0),
         "ear_L": (12 * howl, -18 * tw, 0), "ear_R": (12 * howl, 0, 2 * br)}
    q.update(tail_sway(p, 2, 5 * (1 - howl)))
    return q


def pose_walk(t):
    """Trotting: diagonal pairs (0.6 s)."""
    p = t / 0.6
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 22, 34)
    q.update({"root": (0, 0.3 * abs(S(TAU * p * 2)), 0), "head": (2 * S(TAU * p * 2), 0, 0)})
    q.update(tail_sway(p, 1, 4))
    return q


def pose_run(t):
    """Galloping, the tail streaming out behind (0.4 s)."""
    p = t / 0.4
    q = gait(p, (0.0, 0.1, 0.5, 0.6), 36, 56)
    c = S(TAU * p)
    q.update({"root": (0, 1.2 * max(0.0, S(TAU * p + 0.6)), 0), "spine_0": (-5 * c, 0, 0), "spine_1": (7 * c, 0, 0),
              "neck": (-8 - 4 * c, 0, 0), "head": (8, 0, 0), "ear_L": (-25, 0, 0), "ear_R": (-25, 0, 0)})
    q.update(tail_sway(p, 1, 3, [(24, 0, 0)] + [(5, 0, 0)] * (len(TAIL) - 1)))
    return q


def pose_sit(t, P=L):
    """Sitting upright: the front legs straight, the haunches on the ground, the tail along the
    ground behind, the head turning to watch (4 s)."""
    p = t / 4.0
    br = S(TAU * p * 2)
    a = P["sit_angle"]
    q = {"root": (0, -P["sit_drop"] + 0.1 * br, 0), "spine_0": (-a, 0, 0), "spine_1": (0.8 * br, 0, 0),
         "neck": (a * 0.65, 0, 0), "head": (a * 0.35 + 2 * S(TAU * p), 8 * S(TAU * p), 0)}
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (a + P["sit_hip"], 0, 0)
        q[f"leg_{leg}_lower"] = (P["sit_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["sit_hip"] - P["sit_knee"], 0, 0)
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (a + P["sit_lean"], 0, 0)
        q[f"leg_{leg}_lower"] = (0, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["sit_lean"], 0, 0)
    for k in range(len(TAIL)):  # the tail laid back along the ground (its lift fitted, `sit_tail`)
        q[f"tail_{k}"] = tail_swing(k, 3 * S(TAU * p) if k == len(TAIL) - 1 else 0, (a + P.get("sit_tail", 40) if k == 0 else -6, 0, 0))
    return q


def pose_lie(t, P=L):
    """Lying down, the legs folded under, the head low on its paws and the tail along its side (5 s)."""
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


ANIMS = {"idle": (pose_idle, 8.0), "walk": (pose_walk, 0.6), "run": (pose_run, 0.4), "lie": (pose_lie, 5.0), "sit": (pose_sit, 4.0),
         "pounce": (pose_pounce, 1.0), "bite": (pose_bite, 0.5)}


BLINKS = {"idle": [1.3, 3.2, 6.9], "lie": [2.1, 4.2], "sit": [1.0, 3.1]}
BLINK = 0.09


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (-a, 0, 0), "eyelid_R": (-a, 0, 0)}  # eyes on the face: the lids swing down about x


def frame_times(name, dur):
    n = {"idle": 96, "lie": 40, "sit": 32, "walk": 24, "run": 20, "pounce": 30, "bite": 24}.get(name, 16)
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
          "body": lambda n: n in ("hips", "chest"), "head": lambda n: n in ("skull", "muzzle", "jaw", "nose"),
          "tail": lambda n: n.startswith("tail_"), "rump": lambda n: n == "hips",
          "back legs": lambda n: n.startswith(("thigh_B", "leg_B"))}


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

    def sit_cost(P):
        g = lowest(spec, pose_sit(0.0, dict(L, **P, sit_drop=0)), GROUPS)
        g.pop("tail", None)
        g.pop("head", None)
        floor = min(g.values())
        r = {k: v - floor for k, v in g.items()}
        return 3 * r["front paws"] ** 2 + r["back paws"] ** 2 + r["rump"] ** 2 + 0.5 * r["back legs"] ** 2
    grid = {"sit_angle": list(range(15, 61, 5)), "sit_hip": list(range(-100, -59, 5)), "sit_knee": list(range(110, 166, 5)),
            "sit_lean": [-14, -8, -2, 4]}
    best = fit(grid, sit_cost)[1]
    g = lowest(spec, pose_sit(0.0, dict(L, **best, sit_drop=0)), GROUPS)
    floor = min(v for k, v in g.items() if k not in ("tail", "head"))
    L.update(best, sit_drop=round(float(floor), 2))
    # The tail's lift: the least that keeps it on the ground, not under it.
    for lift in range(-30, 91, 5):
        t = lowest(spec, pose_sit(0.0, dict(L, sit_drop=0, sit_tail=lift)), GROUPS)["tail"] - floor
        if t >= 0.2:
            L["sit_tail"] = lift
            break
    g = lowest(spec, pose_sit(0.0, dict(L, sit_drop=0)), GROUPS)
    print("sitting, lowest:", {k: round(float(v - L["sit_drop"]), 2) for k, v in g.items()})


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
    spec = {"name": "wolf", "texture": [sheet.shape[1], sheet.shape[0]],
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
