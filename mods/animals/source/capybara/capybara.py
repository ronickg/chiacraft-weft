#!/usr/bin/env python3
"""The capybara: the world's largest rodent, calm, always chewing. Made the way Hytale makes
creatures (boxes and a hand-painted texture) with the blocky-model skill, from research alone
(README.md): its colors are rules (see `area`).

    ../dog/chihuahua/.venv/bin/python capybara.py      # writes out/spec.json, out/texture.png

Units are its own (exported at 0.34 model pixels a unit, `build.SCALE`: 47 a block), y up, the
capybara faces +z. Painted at the house density, 32 texels a block (`TX` texels a unit), as the
blocks and items are. A real one is 50-62 cm at the
shoulder and 100-130 cm long: here the back is 25.5 up, the whole animal 54 long; the head 1.15x
(Hytale's chunky look).
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
from ground import lowest, pokes, world  # noqa: E402
from house import texels  # noqa: E402

TX = texels(0.34)  # texels a unit: the house's 32 a block at the capybara's export scale (build.SCALE)

OUT = os.environ.get("CAPYBARA_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

FUR, BACK, BELLY, DARK, SKIN, MOUTH, TEETH, EARIN = range(8)
ROLES = [("coat", 0, 0), ("coat", 0, -1.0), ("coat", 1, 0), "fixed", "fixed", "fixed", "fixed", "fixed"]  # FUR BACK BELLY DARK SKIN MOUTH TEETH EARIN
BASE = [(160, 104, 64), (124, 78, 48), (204, 162, 118), (54, 38, 32), (96, 72, 58), (84, 36, 40), (226, 212, 170),
        (150, 98, 84)]
SHADOW = [0.02, 0.02, 0.05, 0.0, 0.03, 0.95, 0.1, 0.98]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (FUR, BACK, BELLY)  # the coarse coat; the snout's patch, claws and mouth stay crisp

# Small dark eyes high on the sides of the head, a catchlight; calm.
EYES.update({"capybara": dict(pattern=[".DD.",
                                       "DHDD",
                                       "DDDD",
                                       ".DD."], colors={"D": (34, 22, 20), "H": (226, 220, 206)}, halo="round")})
# At 32 a block the head's side is 7 texels by 7: an eye is 2 by 2, dark, with a catchlight up and
# toward the snout. `mirror` is the west side's copy (its texels run front to back the other way).
_D, _H = (34, 22, 20), (226, 220, 206)
EYES.update({"capybara_px": dict(pattern=["DH", "DD"], colors={"D": _D, "H": _H}, halo="none", iris=""),
             "capybara_px_m": dict(pattern=["HD", "DD"], colors={"D": _D, "H": _H}, halo="none", iris="")})
EYE_STYLE = os.environ.get("CAPYBARA_EYES", "capybara_px")
SKULL = ((-6, 17, 15), (6, 28, 26))
EYE_Z, EYE_Y = 22.5, 25.0  # high on the head, toward the front
LID_X = 6.4  # the eyelids' hinge plane, just outside the head's side (x = 6)

# ------------------------------------------------------------------ the model


def el(name, bone, lo, hi, quad=None, rot=None, origin=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad, rot=rot,
                origin=[float(v) for v in origin] if origin else None, cut=None)


def skull_grid():
    """The skull side's texels: (width along z, height) and the units a texel spans (u, v)."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    sz, sy = z1 - z0, y1 - y0
    w, h = max(1, round(TX * sz)), max(1, round(TX * sy))
    return w, h, sz / w, sy / h


def eye_center(side):
    """The eye's center on one side of the skull (east, side > 0: u runs from the front back;
    west: from the back forward), in that face's texels."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    return ((z1 - EYE_Z) if side > 0 else (EYE_Z - z0)) / su, (y1 - EYE_Y) / sv


def eye_box(side):
    """(z0, z1, top, bottom) of the eye on one side, rounded as `stamp_eyes` places it, so the
    eyelid covers it to the texel."""
    pat = EYES[EYE_STYLE]["pattern"]
    ph, pw = len(pat), len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    u0, v0 = eye_corner(*eye_center(side), pat, w)
    za, zb = (z1 - (u0 + pw) * su, z1 - u0 * su) if side > 0 else (z0 + u0 * su, z0 + (u0 + pw) * su)
    return za, zb, y1 - v0 * sv, y1 - (v0 + ph) * sv


LEGS = ("FL", "FR", "BL", "BR")
AXIS = ["neck", "skull", "chest", "hips"]  # the body's axis: no corner of one poking through the next (`ground.pokes`)
FRONT_Z, BACK_Z, LEG_X = 11.0, -13.0, 5.5


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 17, -10)), ("spine_1", "spine_0", (0, 17, -2)),
         ("neck", "spine_1", (0, 20, 12)), ("head", "neck", (0, 22, 16.5)), ("jaw", "head", (0, 15.75, 25.75))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B += [(f"ear_{n}", "head", (s * 4.25, 28, 18.25)), (f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2))]
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        x = LEG_X * s
        if leg[0] == "F":
            B += [(f"leg_{leg}_upper", "spine_1", (x, 11, FRONT_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 5.5, FRONT_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 1.6, FRONT_Z))]
        else:
            B += [(f"leg_{leg}_upper", "spine_0", (x, 13, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 7, BACK_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 1.6, BACK_Z))]
    return B


def elements():
    E = [el("hips", "spine_0", (-8.5, 9, -20), (8.5, 25, -2)),
         # The round back end (no tail): its sides and bottom flush with the hips' (half a unit in), its top 3.5
         # under theirs and out 3.5 behind, two texels each (in by 1 and 2, out 2.5, a ring of hips showed round it; linecheck).
         el("rump", "spine_0", (-8.0, 9.5, -23.5), (8.0, 21.5, -19.5)),
         el("chest", "spine_1", (-9, 8.5, -2.5), (9, 25.5, 14)),
         el("neck", "neck", (-6.5, 15, 11), (6.5, 25, 18)),
         el("skull", "head", *SKULL),
         # Long and blunt; its sides half a unit in from the skull's, its bottom half a unit over the jaw's (at 5
         # and 15.5 the skull's front showed a rim beside it and the jaw 1.5 under it; linecheck).
         el("muzzle", "head", (-5.5, 14.5, 25.5), (5.5, 24, 32)),
         el("mouth", "head", (-3, 15, 25.9), (3, 17, 30.5)),  # its back clear of the jaw's, moved back to 25
         el("jaw", "jaw", (-4, 14, 25), (4, 16, 31))]  # shorter than the muzzle; its back half a unit behind the muzzle's (at 24 its top showed; linecheck)
    for s, n in ((1, "L"), (-1, "R")):
        x0, x1 = (3, 5.5) if s > 0 else (-5.5, -3)
        # Small and round; 3.3 above the skull, over two texels (at 30.5, 1.7 texels, it read as a sliver; linecheck).
        E.append(el(f"ear_{n}", f"ear_{n}", (x0, 27.5, 17), (x1, 31.3, 19.5)))
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        lx0, lx1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down"))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        x = LEG_X * s
        if leg[0] == "F":
            z = FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 2.5, 5, z - 2.5), (x + 2.5, 12, z + 2.5)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2, 1.5, z - 2), (x + 2, 5.5, z + 2)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 2.75, 0, z - 2.5), (x + 2.75, 1.8, z + 4)))
        else:
            z = BACK_Z
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (x - 3.5, 6, z - 5), (x + 3.5, 17, z + 4)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2, 1.5, z - 2), (x + 2, 7, z + 2)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 2.75, 0, z - 2.5), (x + 2.75, 1.8, z + 4)))
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


def area(e, face, p, uu, vv, h):
    """A capybara's coat: reddish brown, darker along the back, lighter underneath; the dark bald
    patch on top of the snout; dark nostrils; darker, bare feet with claws. `p` is in units, `uu`,
    `vv` are texel centers (indices + 0.5)."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, FUR)
    side = face in ("east", "west")
    t = vv / max(h, 1)
    if name in ("hips", "rump", "chest", "neck"):
        if face == "up":
            a[:] = BACK
        elif side or face in ("south", "north"):
            a = np.where(t < 0.25, BACK, np.where(t > 0.8, BELLY, FUR))
        elif face == "down":
            a[:] = BELLY
    elif name in ("skull", "muzzle"):
        if face == "up":
            a[:] = BACK
            if name == "muzzle":  # the morrillo: a dark bald patch on top of the snout
                a = np.where((np.abs(x) < 2.6) & (np.abs(z - 28.2) < 1.8), DARK, a)
        elif face == "down":
            a[:] = BELLY
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else FUR
        if face == "down":
            a[:] = BELLY
    elif name == "mouth":
        a[:] = MOUTH
    elif name.startswith("ear_"):
        a[:] = BACK
        if face == "south" and a.shape[1] >= 3:  # the ear's inside (an ear is 2 texels at 32)
            a = np.where((uu > 0.6) & (uu < a.shape[1] - 0.6) & (vv > 0.6), EARIN, a)
    elif name.startswith(("leg_", "thigh_")):
        a = np.where(y < 4.5, SKIN, FUR) if not name.endswith("_paw") else np.full(x.shape, SKIN)
        if face == "up" and name.startswith("thigh_"):
            a[:] = BACK
    return a


LASH = (40, 28, 24)


# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}  # a layers.Recorder while build() paints, and the face's key


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)
def layer_props(e, face, p, a):
    """A capybara's coat varies little: a slightly lighter belly, darker extremities (muzzle,
    feet); white is very rare."""
    n, y = e["name"], p[..., 1]
    lo, hi = e["lo"][1], e["hi"][1]
    low = 1 - _ramp(y, lo, max(hi, lo + 1e-3))
    white = 0.7 * low if n in ("chest", "hips", "rump") else np.full(y.shape, 0.25)
    ext = np.full(y.shape, 1.0 if n in ("muzzle", "jaw") or n.endswith("_paw") else 0.0)
    return {"white": white, "extremity": ext, "pattern": np.ones(y.shape)}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    if w == 0 or h == 0:
        return None
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel centers, in texels
    size = np.array(e["hi"]) - np.array(e["lo"])
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
    p = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv
    light_face = face
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "jaw")):
        light_face = "north"  # seen when a pose lifts or folds it
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, n, du, dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["name"] == "skull" and face in ("east", "west"):
        design = EYE_STYLE + "_m" if face == "west" and EYE_STYLE + "_m" in EYES else EYE_STYLE
        stamp_eyes(img, [eye_center(1 if face == "east" else -1)], design)
    if e["name"] == "muzzle" and face == "south" and w >= 5 and h >= 4:  # big nostrils high on the blunt snout, a split lip
        # Slits, not holes: big round nostrils read as a second pair of eyes. At 32 the snout's front
        # is 7 by 6: one texel a nostril, the lip line down the middle, the incisors under it.
        mid = [w // 2] if w % 2 else [w // 2 - 1, w // 2]
        img[1, [mid[0] - 2, mid[-1] + 2], :3] = (72, 48, 40)
        img[3:h - 1, mid, :3] = (img[3:h - 1, mid, :3] * 0.75).astype(np.uint8)
        img[h - 1, mid, :3] = (img[h - 1, mid, :3] * 0.5 + np.array(BASE[TEETH]) * 0.5).astype(np.uint8)
    if e["name"].startswith("eyelid"):
        col_idx = 0 if e["name"].endswith("L") else w - 1
        img[:, col_idx, :3] = LASH
    if e["name"].endswith("_paw") and face == "south" and h >= 2:  # webbed toes: 4 in front, 3 behind, dark claws
        toes = (1, 2, 3) if e["name"].startswith("leg_F") else (1, 3)
        step = w / (len(toes) + 1)
        for k in range(len(toes)):
            c = int(round(step * (k + 1))) - 1
            img[:, c, :3] = (img[:, c, :3] * 0.7).astype(np.uint8)
        img[h - 1, :, :3] = BASE[DARK]
    elif e["name"].endswith("_paw") and face == "south":  # at 32 the paw's front is one texel high: claws between the toes
        img[0, 1::2, :3] = BASE[DARK]
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"sit_angle": 30, "sit_hip": -80, "sit_knee": 140, "sit_lean": -4, "sit_drop": 5,
     "lie_front": 80, "lie_hip": -80, "lie_knee": 150, "lie_drop": 6, "graze_neck": 40, "graze_head": 20}


def gait(p, phases, swing_amp, lift):
    """Legs in a gait: a foot swings forward (upper angle falling) while it's lifted."""
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


def chew(t, rate=3.0, amp=6.0):
    """Always chewing: the jaw works up and down."""
    return {"jaw": (amp * max(0.0, S(TAU * t * rate)), 0, 0)}


def pose_idle(t):
    """Chewing and looking about, then down to graze, then up again (6 s)."""
    p = t / 6.0
    br = S(TAU * p * 2)
    graze = float(np.interp(p, [0, .4, .5, .85, .95, 1], [0, 0, 1, 1, 0, 0]))
    tw = max(0.0, S(TAU * (p - 0.2) * 8)) if 0.2 < p < 0.26 else 0.0
    q = {"root": (0, 0.12 * br, 0), "spine_1": (0.6 * br, 0, 0),
         "neck": (L["graze_neck"] * graze, 0, 0),
         "head": (L["graze_head"] * graze + 2 * S(TAU * p), 10 * S(TAU * p * 2) * (1 - graze), 0),
         "ear_L": (0, 0, -18 * tw), "ear_R": (0, 0, 3 * br)}
    q.update(chew(t, 3.0 if graze > 0.5 else 2.0))
    return q


def pose_walk(t):
    p = t / 1.1
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 18, 45)  # short legs: a small swing, a clear lift
    q.update({"root": (0, 0.35 * abs(S(TAU * p)), 0), "head": (2 * S(TAU * p * 2), 3 * S(TAU * p), 0)})
    q.update(chew(t, 2.0, 4.0))
    return q


def pose_run(t):
    p = t / 0.55
    q = gait(p, (0.0, 0.08, 0.5, 0.58), 30, 62)
    c = S(TAU * p)
    q.update({"root": (0, 1.2 * max(0.0, S(TAU * p + 0.6)), 0), "spine_0": (-4 * c, 0, 0), "spine_1": (6 * c, 0, 0),
              "neck": (-4 * c, 0, 0), "ear_L": (10, 0, -6), "ear_R": (10, 0, 6)})
    return q


def pose_sit(t, P=L):
    """Sitting on its haunches, front legs straight, chewing."""
    p = t / 4.0
    br = S(TAU * p * 2)
    a = P["sit_angle"]
    q = {"root": (0, -P["sit_drop"] + 0.1 * br, 0), "spine_0": (-a, 0, 0), "spine_1": (0.6 * br, 0, 0),
         "neck": (a * 0.5, 0, 0), "head": (a * 0.5, 6 * S(TAU * p), 0)}
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (a + P["sit_hip"], 0, 0)
        q[f"leg_{leg}_lower"] = (P["sit_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["sit_hip"] - P["sit_knee"], 0, 0)
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (a + P["sit_lean"], 0, 0)
        q[f"leg_{leg}_lower"] = (0, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["sit_lean"], 0, 0)
    q.update(chew(t, 2.0))
    return q


def pose_lie(t, P=L):
    """Flopped on its belly, legs tucked, the head held up (as capybaras rest), chewing slowly."""
    p = t / 5.0
    br = S(TAU * p * 2)
    q = {"root": (0, -P["lie_drop"] + 0.12 * br, 0), "spine_1": (0.6 * br, 0, 0), "head": (0, 5 * S(TAU * p), 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (P["lie_front"], 0, 0)
        q[f"leg_{leg}_lower"] = (-70, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_front"] + 70, 0, 0)
    for leg, s in (("BL", 1), ("BR", -1)):
        q[f"leg_{leg}_upper"] = (P["lie_hip"], 0, s * 6)
        q[f"leg_{leg}_lower"] = (P["lie_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_hip"] - P["lie_knee"], 0, 0)
    q.update(chew(t, 1.5, 4.0))
    return q


# ------------------------------------------------------------------ behaviour clips (`anim.rs`)
# Each a whole-body pose over what plays below on the bones it moves (`to_bedrock.as_action`): held
# states and the gaits loop, the rest play once. The capybara's neck is wider than its skull and
# sits 3 units into the chest, so a neck turned more than 5 degrees, or a head turned past 11, pokes a
# corner out of the skull or chest (`ground.pokes`): it looks about with its head (its eyes are on
# the sides: a real one hardly turns) and lowers it along clean angles (`DOWN`), a step a key.

C = {"down_neck": 57, "down_head": 8, "sleep_neck": 35, "sleep_head": 0, "rear_lift": 0, "death_drop": 10,
     "back_tilt": 8, "back_hip": -20, "back_knee": 30, "back_drop": 0}


def still(t, period=4.0):
    """Standing still and breathing (as the idle): what a clip builds on."""
    br = S(TAU * t / period * 2)
    return {"root": (0, 0.12 * br, 0), "spine_1": (0.6 * br, 0, 0)}


def key(p, n, pts):
    """A value keyed on the clip's frames: `pts` (frame index, value) over its `n` frames, linear
    between, so a key lands on each point and a step between two keys is never sampled half way."""
    return float(np.interp(p * n, [a for a, _ in pts], [b for _, b in pts]))


def down(k, P=C):
    """The head lowered along the neck's clean angles: k 0 up, 1 and 2 on the way, 3 at the ground
    (fitted: the mouth at the grass, the legs straight)."""
    return {"neck": (float(np.interp(k, [0, 1, 2, 3], [0, 20, 35, P["down_neck"]])), 0, 0),
            "head": (float(np.interp(k, [0, 1, 2, 3], [0, P["down_head"] / 3, 2 * P["down_head"] / 3, P["down_head"]])), 0, 0)}


def sit_back(s, P=C):
    """Sitting back a little by `s` (0..1): the forequarters up, the haunches lowered on bent hind
    legs, all four feet flat on the ground (fitted)."""
    a = P["back_tilt"] * s
    hip, knee = P["back_hip"] * s, P["back_knee"] * s
    q = {"root": (0, -P["back_drop"] * s, 0), "spine_0": (-a, 0, 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (a, 0, 0)
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (a + hip, 0, 0)
        q[f"leg_{leg}_lower"] = (knee, 0, 0)
        q[f"leg_{leg}_paw"] = (-hip - knee, 0, 0)
    return q


def pose_stare(t):
    """Half aware: frozen, the chewing stopped, the head up, the small ears pricked (2 s, held)."""
    q = still(t, 2.0)
    q.update({"neck": (-12, 0, 0), "head": (4, 0, 0), "ear_L": (14, 0, 6), "ear_R": (14, 0, -6)})
    return q


def pose_graze(t):
    """Grazing: the mouth at the grass, cropping and chewing, the head working a little (2.5 s, held)."""
    p = t / 2.5
    q = still(t)
    q.update(down(3))
    q["head"] = (q["head"][0] + 3 * S(TAU * p * 2), 0, 0)
    q.update(chew(t, 3.2, 6.0))
    q["ear_L"] = (0, 0, -6 * max(0.0, S(TAU * p)))
    return q


def pose_drink(t):
    """Drinking: the mouth in the water, lapping (1.5 s, held)."""
    q = still(t)
    q.update(down(3))
    q["head"] = (q["head"][0] + 2 * S(TAU * t / 1.5 * 3), 0, 0)
    q.update(chew(t, 2.0, 3.0))
    return q


def pose_sniff(t):
    """Nose down to the ground, sniffing along it, and up again (2.2 s, once)."""
    p = t / 2.2
    k = key(p, 22, [(0, 0), (2, 0), (5, 3), (17, 3), (20, 0), (22, 0)])
    q = still(t)
    q.update(down(k))
    q["head"] = (q["head"][0] - (k / 3) * 3 * max(0.0, S(TAU * p * 5)), 0, 0)  # the nose working
    q.update({"ear_L": (6 * k / 3, 0, 0), "ear_R": (6 * k / 3, 0, 0)})
    return q


def pose_look(t):
    """Head up, looking about with its side eyes: a little one way, then the other (3 s, once)."""
    p = t / 3.0
    up = key(p, 24, [(0, 0), (3, 1), (21, 1), (24, 0)])
    yaw = key(p, 24, [(0, 0), (3, 1), (9, 1), (12, -1), (18, -1), (21, 0), (24, 0)])
    q = still(t)
    q.update({"neck": (-14 * up, 4 * yaw, 0), "head": (6 * up, 10 * yaw, 0), "ear_L": (10 * up, 0, -6 * max(0.0, yaw)),
              "ear_R": (10 * up, 0, -6 * min(0.0, yaw))})
    return q


def pose_scratch(t):
    """An itch: a hind foot up to the flank, scratching fast, the head tipped toward it, eyes half
    shut (2 s, once)."""
    p = t / 2.0
    d = key(p, 32, [(0, 0), (5, 1), (27, 1), (32, 0)])
    sc = d * S(TAU * p * 7)
    q = still(t)
    q.update({"leg_BL_upper": (-48 * d - 8 * sc, 0, 10 * d), "leg_BL_lower": (-40 * d + 10 * sc, 0, 0), "leg_BL_paw": (30 * d, 0, 0),
              "neck": (0, 4 * d, 0), "head": (6 * d, 10 * d, 0), "ear_L": (-10 * d, 0, 0),
              "eyelid_L": (0, 0, 50 * d), "eyelid_R": (0, 0, -50 * d)})
    return q


def pose_stretch(t):
    """Getting up: the neck and head stretched up and out, a wide yawn, then a hind leg pushed out
    behind (2.4 s, once)."""
    p = t / 2.4
    a = key(p, 24, [(0, 0), (5, 1), (12, 1), (14, 0), (24, 0)])
    b = key(p, 24, [(0, 0), (13, 0), (16, 1), (21, 1), (24, 0)])
    q = still(t)
    q.update({"neck": (-16 * a, 0, 0), "head": (-14 * a, 0, 0), "ear_L": (-14 * a, 0, -10 * a), "ear_R": (-14 * a, 0, 10 * a),
              "jaw": (22 * a, 0, 0), "eyelid_L": (0, 0, 70 * a), "eyelid_R": (0, 0, -70 * a),
              "leg_BR_upper": (34 * b, 0, 0), "leg_BR_lower": (-8 * b, 0, 0), "leg_BR_paw": (-20 * b, 0, 0)})
    return q


def pose_sleep(t, P=L, Q=C):
    """Asleep, lying (its rest pose), the chin laid on the ground, eyes shut, slow breath (6 s, held)."""
    q = pose_lie(0.0, P)
    br = S(TAU * t / 6.0 * 2)
    q.update({"root": (0, -P["lie_drop"] + 0.12 * br, 0), "spine_1": (0.8 * br, 0, 0), "neck": (Q["sleep_neck"], 0, 0),
              "head": (Q["sleep_head"], 0, 0), "jaw": (0, 0, 0), "ear_L": (-12, 0, 10), "ear_R": (-12, 0, -10),
              "eyelid_L": (0, 0, 90), "eyelid_R": (0, 0, -90)})
    return q


def pose_alarm(t):
    """Alarm: the head jerked up, a sharp bark (two), the ears up, then back down (1.4 s, once)."""
    p = t / 1.4
    up = key(p, 28, [(0, 0), (2, 1), (22, 1), (28, 0)])
    bark = key(p, 28, [(0, 0), (4, 0), (5, 1), (7, 0), (11, 0), (12, 1), (14, 0), (28, 0)])
    q = still(t)
    q.update({"neck": (-20 * up, 0, 0), "head": (-8 * up - 6 * bark, 0, 0), "jaw": (18 * bark, 0, 0),
              "ear_L": (16 * up, 0, 8 * up), "ear_R": (16 * up, 0, -8 * up)})
    return q


def pose_rear(t, P=C):
    """Standing its ground: a short lift of the heavy forequarters, the head and chest up, the hind
    feet planted, the teeth chattering (1.4 s, once)."""
    p = t / 1.4
    r = key(p, 28, [(0, 0), (6, 1), (18, 1), (26, 0), (28, 0)])
    q = still(t)
    a = 16 * r
    q.update({"root": (0, P["rear_lift"] * r, 0), "spine_0": (-a, 0, 0), "neck": (-8 * r, 0, 0), "head": (4 * r, 0, 0),
              "jaw": (6 * r * max(0.0, S(TAU * p * 9)), 0, 0), "ear_L": (-16 * r, 0, 0), "ear_R": (-16 * r, 0, 0)})
    for leg in ("BL", "BR"):  # kept upright under the tilt, flat on the ground
        q[f"leg_{leg}_upper"] = (a, 0, 0)
    for leg in ("FL", "FR"):  # lifted, folded a little
        q[f"leg_{leg}_upper"] = (a - 18 * r, 0, 0)
        q[f"leg_{leg}_lower"] = (36 * r, 0, 0)
        q[f"leg_{leg}_paw"] = (-14 * r, 0, 0)
    return q


def pose_play(t):
    """A young one's play: two little bounces, the head tossed (1.2 s, once)."""
    p = t / 1.2
    hop = max(0.0, S(TAU * p * 2))
    q = still(t)
    q.update({"root": (0, 3.0 * hop, 0), "spine_0": (-4 * hop, 0, 0), "neck": (-10 * hop, 0, 0),
              "head": (4 * hop, 9 * S(TAU * p), 0), "ear_L": (-10 * hop, 0, -8 * hop), "ear_R": (-10 * hop, 0, 8 * hop)})
    for leg in LEGS:
        f = leg[0] == "F"
        q[f"leg_{leg}_upper"] = ((-14 if f else 14) * hop, 0, 0)
        q[f"leg_{leg}_lower"] = ((20 if f else -20) * hop, 0, 0)
    return q


def pose_groom(t):
    """Grooming: a forefoot lifted to the mouth, nibbling over it, the head tucked (2.5 s, once)."""
    p = t / 2.5
    d = key(p, 25, [(0, 0), (1, 0), (3, 1), (22, 1), (24, 0), (25, 0)])
    k = key(p, 25, [(0, 0), (1, 1), (3, 1), (22, 1), (24, 1), (25, 0)])
    q = still(t)
    q.update({"neck": (20 * k, 0, 0), "head": (48 * d + 4 * d * S(TAU * p * 6), 0, 0),
              "leg_FL_upper": (-34 * d, 0, -4 * d), "leg_FL_lower": (56 * d, 0, 0), "leg_FL_paw": (-20 * d, 0, 0)})
    q.update(chew(t, 5.0, 5.0 * d))
    return q


def pose_nuzzle(t):
    """A mother nuzzling her young: the head low and forward, a gentle push, a lick (2 s, once)."""
    p = t / 2.0
    k = key(p, 20, [(0, 0), (1, 0), (3, 2), (17, 2), (19, 0), (20, 0)])
    d = k / 2
    push = d * max(0.0, S(TAU * p * 2))
    q = still(t)
    q.update(down(k))
    q["head"] = (q["head"][0] - 8 * push, 0, 0)
    q.update(chew(t, 2.0, 4.0 * d))
    q.update({"ear_L": (10 * d, 0, 0), "ear_R": (10 * d, 0, 0)})
    return q


def pose_flinch(t):
    """Hit: a jerk aside, the head ducked, ears back, eyes squeezed (0.45 s, once)."""
    p = t / 0.45
    d = key(p, 9, [(0, 0), (2, 1), (4, 0.8), (9, 0)])
    q = still(t)
    q.update({"spine_1": (0, 4 * d, 0), "neck": (14 * d, 0, 0), "head": (10 * d, 0, 0),
              "ear_L": (-24 * d, 0, 0), "ear_R": (-24 * d, 0, 0), "eyelid_L": (0, 0, 60 * d), "eyelid_R": (0, 0, -60 * d)})
    return q


def pose_watch(t):
    """On watch: sitting back a little, the head high, looking slowly one way and the other with its
    side eyes, the ears turning, a chew now and then (4 s, held)."""
    p = t / 4.0
    yaw = S(TAU * p)
    q = still(t, 4.0)
    q.update(sit_back(1.0))
    q.update({"neck": (-16, 4 * yaw, 0), "head": (4, 10 * yaw, 0),
              "ear_L": (12, 0, -6 * yaw), "ear_R": (12, 0, -6 * yaw)})
    q["spine_1"] = still(t, 4.0)["spine_1"]
    q.update(chew(t, 2.0, 3.0 * max(0.0, S(TAU * p * 2))))
    return q


def pose_approach(t):
    """Curious, coming to look: slow careful steps, the head low and the nose out, ears forward
    (1.6 s a cycle, in place of the walk)."""
    p = t / 1.6
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 14, 42)
    q.update({"root": (0, 0.3 * abs(S(TAU * p)), 0), "neck": (18, 0, 0), "head": (-12 + 3 * S(TAU * p * 2), 0, 0),
              "ear_L": (12, 0, 0), "ear_R": (12, 0, 0)})
    return q


def pose_charge(t):
    """Charging: a heavy, head-low trot-run, the ears back (0.6 s a cycle, in place of the walk and
    the run). The spine only rocks with the stride, no lean."""
    p = t / 0.6
    q = gait(p, (0.0, 0.5, 0.55, 0.05), 26, 60)
    c = S(TAU * p * 2)
    q.update({"root": (0, 1.0 * max(0.0, c), 0), "spine_0": (-2 * c, 0, 0), "spine_1": (3 * c, 0, 0),
              "neck": (18, 0, 0), "head": (14, 0, 0), "ear_L": (-24, 0, 6), "ear_R": (-24, 0, -6)})
    return q


def pose_death(t, P=C):
    """Dying: the legs give, it rolls onto its side, the head drops; it stays (1.6 s, holds)."""
    p = t / 1.6
    fall = float(np.interp(p, [0, .35, .7, 1], [0, 0.35, 1, 1]))
    q = {"root": (0, -P["death_drop"] * fall, 0), "spine_0": (0, 0, 84 * fall), "neck": (20 * fall, 0, 0),
         "head": (10 * fall, 0, 0), "jaw": (8 * fall, 0, 0), "ear_L": (-20 * fall, 0, 0), "ear_R": (-20 * fall, 0, 0)}
    for leg in LEGS:
        q[f"leg_{leg}_upper"] = ((12 if leg[0] == "F" else -12) * fall, 0, 0)
    q.update({"eyelid_L": (0, 0, 90 * fall), "eyelid_R": (0, 0, -90 * fall)})
    return q


BLINKS = {"idle": [1.2, 4.5], "sit": [1.6], "lie": [1.0, 2.6, 4.2], "stare": [1.3], "graze": [1.1], "drink": [0.6], "look": [1.6],
          "watch": [1.2, 3.1], "nuzzle": [1.4]}
BLINK = 0.1
STANDING = ("idle", "walk", "run", "sit", "lie")  # the looks the engine plays by itself: written as they were
ANIMS = {"idle": (pose_idle, 6.0), "walk": (pose_walk, 1.1), "run": (pose_run, 0.55), "sit": (pose_sit, 4.0),
         "lie": (pose_lie, 5.0),
         "stare": (pose_stare, 2.0), "graze": (pose_graze, 2.5), "drink": (pose_drink, 1.5), "sniff": (pose_sniff, 2.2),
         "look": (pose_look, 3.0), "scratch": (pose_scratch, 2.0), "stretch": (pose_stretch, 2.4), "sleep": (pose_sleep, 6.0),
         "alarm": (pose_alarm, 1.4), "rear": (pose_rear, 1.4), "play": (pose_play, 1.2), "groom": (pose_groom, 2.5),
         "nuzzle": (pose_nuzzle, 2.0), "flinch": (pose_flinch, 0.45), "death": (pose_death, 1.6),
         "watch": (pose_watch, 4.0), "approach": (pose_approach, 1.6), "charge": (pose_charge, 0.6)}
# Only the held states and the gaits loop (the engine loops a clip whose file says so, whatever the
# game asks); the one-offs play once, death holds its last frame.
LOOPS = {"stare": True, "graze": True, "drink": True, "sleep": True, "watch": True, "approach": True, "charge": True,
         "death": "hold_on_last_frame"}
BLENDS = {"flinch": (0.05, 0.2), "death": (0.1, 0.0), "sleep": (0.6, 0.6), "graze": (0.4, 0.4), "watch": (0.4, 0.4),
          "approach": (0.3, 0.3), "charge": (0.15, 0.3)}
# Keys a clip: those stepping through the neck's clean angles (`key`) are keyed on these counts.
FRAMES = {"idle": 72, "sit": 48, "lie": 60, "walk": 24, "run": 20,  # chewing and short steps need fine keys
          "stare": 24, "graze": 40, "drink": 24, "sniff": 22, "look": 24, "scratch": 32, "stretch": 24, "sleep": 36, "alarm": 28,
          "rear": 42, "play": 24, "groom": 50, "nuzzle": 20, "flinch": 9, "death": 32, "watch": 40, "approach": 24, "charge": 20}


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}  # side eyes: the lids swing down about z


def frame_times(name, dur):
    n = FRAMES.get(name, 16)
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
            if name in STANDING or name in BLINKS:  # the clips that shut or squint their own eyes keep them
                pose.update(eyelids(name, tt))
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames}
        if name not in STANDING:
            out[name].update(loop=loops, blend=BLENDS.get(name, (0.25, 0.3)))
    # A clip goes over what plays below only on the bones it writes, and the exporter leaves out a bone
    # that stays at 0: kept at a hundredth of a degree, so the idle's grazing neck and chewing jaw don't
    # show through a clip that holds them still. The eyelids are left to the idle's blinks.
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
          "rump": lambda n: n in ("hips", "rump"), "chest": lambda n: n == "chest",
          "muzzle": lambda n: n in ("muzzle", "jaw")}


def fit_poses(spec):
    """Sit and lie on the ground (paws, rump, belly), and grazing with the mouth at the grass."""
    def low(fn, P):
        g = lowest(spec, fn(0.0, P), GROUPS)
        g.pop("muzzle", None)
        return g

    def sit_cost(P):
        g = low(pose_sit, P)
        f = min(g.values())
        r = {k: v - f for k, v in g.items()}
        return 3 * r["front paws"] ** 2 + r["back paws"] ** 2 + r["rump"] ** 2 + 0.5 * r["back legs"] ** 2
    grid = {"sit_angle": list(range(10, 46, 5)), "sit_hip": list(range(-100, -59, 5)), "sit_knee": list(range(110, 166, 5)),
            "sit_lean": [-10, -4, 2]}
    best = fit(grid, lambda P: sit_cost(dict(L, **P, sit_drop=0)))[1]
    L.update(best, sit_drop=round(float(min(low(pose_sit, dict(L, **best, sit_drop=0)).values())), 2))

    def lie_cost(P):
        g = low(pose_lie, P)
        f = min(g.values())
        r = {k: v - f for k, v in g.items()}
        return r["chest"] ** 2 + r["rump"] ** 2 + 0.5 * r["front paws"] ** 2 + 0.5 * r["back paws"] ** 2
    grid = {"lie_front": list(range(60, 111, 5)), "lie_hip": list(range(-104, -63, 8)), "lie_knee": list(range(116, 166, 8))}
    best = fit(grid, lambda P: lie_cost(dict(L, **P, lie_drop=0)))[1]
    L.update(best, lie_drop=round(float(min(low(pose_lie, dict(L, **best, lie_drop=0)).values())), 2))

    # Grazing: neck and head angles that bring the mouth down to the grass (about 0.5 up).
    def graze_cost(P):
        L2 = dict(L, **P)
        g = lowest(spec, {"neck": (P["graze_neck"], 0, 0), "head": (P["graze_head"], 0, 0)}, GROUPS)
        return (g["muzzle"] - 0.5) ** 2 + 0.002 * (P["graze_neck"] ** 2 + P["graze_head"] ** 2)
    best = fit({"graze_neck": list(range(20, 71, 5)), "graze_head": list(range(0, 61, 5))}, graze_cost)[1]
    L.update(best)

    # The behaviour clips (`C`): fitted with the idle's own numbers left as they are.
    def poked(pose):
        """How many of the body's axis parts poke a corner through a neighbour in this pose."""
        one = dict(spec, animations={"p": {"frames": [{"t": 0, "rot": {k: v for k, v in pose.items() if k != "root"},
                                                       "root": pose.get("root", (0, 0, 0))}]}})
        return sum(1 for _, (_, where) in pokes(one, AXIS).items() if any(w != "rest pose" for w in where))

    # Head down (graze, drink, sniff): the mouth at the grass with the jaw open in a chew, the legs
    # straight (the neck reaches), nothing poking through.
    def down_cost(P):
        q = dict(down(3, dict(C, **P)), jaw=(6, 0, 0))
        g = lowest(spec, q, GROUPS)
        return (g["muzzle"] - 0.2) ** 2 + 0.0005 * (P["down_neck"] ** 2 + P["down_head"] ** 2) + 100 * poked(q)
    C.update(fit({"down_neck": list(range(52, 76)), "down_head": list(range(-10, 31, 2))}, down_cost)[1])

    # Asleep: lying, the chin laid on the ground, nothing poking through.
    def sleep_cost(P):
        q = pose_sleep(0.0, L, dict(C, **P))
        g = lowest(spec, q, GROUPS)
        return (g["muzzle"] - 0.2) ** 2 + 0.0005 * (P["sleep_neck"] ** 2 + P["sleep_head"] ** 2) + 100 * poked(q)
    C.update(fit({"sleep_neck": list(range(0, 21, 2)) + [35] + list(range(55, 71, 3)), "sleep_head": list(range(-30, 31, 2))}, sleep_cost)[1])

    # Watching, sat back a little: the hind legs bent so all four feet stay flat on the ground.
    paw = next(e for e in spec["elements"] if e["name"] == "leg_BL_paw")
    pv = next(np.array(b["pivot"], float) for b in spec["bones"] if b["name"] == "leg_BL_paw")
    mid = (np.array(paw["lo"]) + np.array(paw["hi"])) / 2

    def back_cost(P):
        q = sit_back(1.0, dict(C, **P, back_drop=0))
        g = lowest(spec, q, GROUPS)
        R, t = world(spec["bones"], q)["leg_BL_paw"]
        dz = (R @ (mid - pv) + t)[2] - mid[2]  # the hind feet stay where they stood
        return (g["back paws"] - g["front paws"]) ** 2 + 0.1 * dz ** 2 + 0.0001 * (P["back_hip"] ** 2 + P["back_knee"] ** 2)
    C.update(fit({"back_hip": list(range(-50, 1, 2)), "back_knee": list(range(0, 91, 2))}, back_cost)[1])
    g = lowest(spec, sit_back(1.0, dict(C, back_drop=0)), GROUPS)
    C["back_drop"] = round(float(min(g["front paws"], g["back paws"])), 2)

    # Rearing: lifted so the hind feet stay on the ground (at the top of the lift).
    g = lowest(spec, pose_rear(0.3, dict(C, rear_lift=0)), GROUPS)
    C["rear_lift"] = round(-float(g["back paws"]), 2)
    # Dying on its side: dropped until nothing is below the ground (each frame of the fall rests on its
    # lowest point: `build.py`).
    g = lowest(spec, pose_death(1.6, dict(C, death_drop=0)), {"all": lambda n: True})
    C["death_drop"] = round(float(g["all"]), 2)
    print("clips fitted:", C)


# ------------------------------------------------------------------ build


def build():
    B = bones()
    E = elements()
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"]]
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
    spec = {"name": "capybara", "texture": [sheet.shape[1], sheet.shape[0]],
            "bones": [{"name": n, "parent": p, "pivot": [float(v) for v in pv]} for n, p, pv in B], "elements": E}
    fit_poses(spec)
    spec["animations"] = animations([n for n, _, _ in B])
    spec["fitted"] = dict(L, **C)
    return spec, sheet


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    spec, sheet = build()
    Image.fromarray(sheet, "RGBA").save(os.path.join(OUT, "texture.png"))
    with open(os.path.join(OUT, "spec.json"), "w") as f:
        json.dump(spec, f)
    print(len(spec["elements"]), "elements;", len(spec["bones"]), "bones; texture", sheet.shape[1], "x", sheet.shape[0],
          "; fitted", spec["fitted"])
