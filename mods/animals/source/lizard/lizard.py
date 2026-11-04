#!/usr/bin/env python3
"""Priscagama, the Djadokhta's agamid lizard, made the way Hytale makes creatures (boxes and a
hand-painted texture) with the blocky-model skill, from research alone (README.md): its colors are
rules (see `area`).

    ~/.venvs/blocky/bin/python lizard.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each), y up, it faces +z. A real one was about 20 cm
long, the tail more than half of it: here about 1.3 times life, as the mouse: 9 units from the
snout to the hips, the tail 10 more. Its head is still only 2.4 units across and 1.5 tall, so it is
painted 4 times the house density (`TX`, 128 a block): a designed eye that blinks, nostrils and lips. Its legs sprawl out
to the sides, the body carried low; it bobs its head and pushes up on its front legs, as agamas do.
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
from house import texels  # noqa: E402

# Texels a unit: 128 a block (k = 4, `house.fine`). The eye sits on the head's side, 2 units by 1.5:
# at the house's 32 a block that is 1 texel by 0.75, at 64 still 1.5 tall; at 128 it is 4 by 3.
TX = texels(0.25, 4)

OUT = os.environ.get("LIZARD_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

SKIN, BACK, BELLY, PALE, MOUTH, CLAW = range(6)
# No record of its colors (est.: a desert agama's): SKIN sandy grey-brown, BACK the darker blotches
# across the back and the tail's bands, BELLY the pale underside, PALE the light stripe along each
# side of the back, MOUTH the line of the lips, CLAW the claws.
BASE = [(170, 146, 110), (112, 90, 66), (218, 208, 184), (206, 190, 150), (92, 66, 56), (78, 64, 56)]
SHADOW = [0.03, 0.03, 0.07, 0.07, 0.95, 0.74]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (SKIN, BACK, BELLY, PALE)
ROLES = [("coat", 0, 0), ("coat", 0, -1.2), ("coat", 1, 0), ("coat", 0.6, 0.6), "fixed", "fixed"]

# An agama's eye: a dark pupil in a gold-brown ring, a catchlight toward the front, darker scales
# round it. "lizard" is the old one at 256 a block (for comparison); at 128 the head's side is 4
# texels by 3 and the eye 2 by 2: the catchlight in front of the pupil, the gold ring under them.
EYES.update({
    "lizard": dict(pattern=[".GG.",
                            "GHPG",
                            "GPPG",
                            ".gg."], colors={"P": (28, 22, 16), "H": (232, 224, 196), "G": (150, 112, 58), "g": (188, 146, 76)},
                   halo="round", iris="Gg"),
    "lizard_px": dict(pattern=["HP", "Gg"], colors={"P": (28, 22, 16), "H": (232, 224, 196), "G": (150, 112, 58),
                                                    "g": (188, 146, 76)}, halo="none", iris="Gg"),
    "lizard_px_ring": dict(pattern=["GP", "gG"], colors={"P": (28, 22, 16), "G": (150, 112, 58), "g": (188, 146, 76)},
                           halo="none", iris="Gg"),
    "lizard_px_dark": dict(pattern=["HP", "PG"], colors={"P": (28, 22, 16), "H": (232, 224, 196), "G": (150, 112, 58)},
                           halo="none", iris="G"),
})
EYE_STYLE = os.environ.get("LIZARD_EYES", "lizard_px")
SKULL = ((-1.2, 0.8, 2.8), (1.2, 2.3, 4.8))
EYE_Z, EYE_Y = 4.0, 1.6
LID_X = 1.55  # the eyelids' hinge plane, 0.35 outside the head's side: closed, they sit over the eye
LASH = (70, 54, 40)


def skull_side():
    """The skull side's texels: (width, height) and one texel's size in units (u along z, v along y)."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h = max(1, round(TX * (z1 - z0))), max(1, round(TX * (y1 - y0)))
    return w, h, (z1 - z0) / w, (y1 - y0) / h


def eye_pattern(side):
    """The eye's design on one side: mirrored on the west face (its texels run back to front), so
    the catchlight is toward the snout on both."""
    pat = EYES[EYE_STYLE]["pattern"]
    return pat if side > 0 else [row[::-1] for row in pat]


def eye_center(side):
    """The eye's center on the skull's east (side > 0) or west face, in that face's texels."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_side()
    return ((z1 - EYE_Z) if side > 0 else (EYE_Z - z0)) / su, (y1 - EYE_Y) / sv


def eye_box(side):
    """(z0, z1, top, bottom) of the eye on one side in units, rounded as `stamp_eyes` places it."""
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
# Each leg: the shoulder or hip (on the body), the elbow or knee out to the side, the wrist or ankle
# (x out from the middle, y, z); the upper leg runs out sideways, the lower one down to the foot.
# Every face at least 0.35 from its neighbors' parallel faces (`boxcheck`), the front feet a hair up
# so their soles never meet the hind feet's in a stride.
LEG = {"F": dict(hip=(1.3, 1.15, 1.2), knee=(2.4, 1.0, 1.2), ankle=(2.4, 0.6, 1.2), upper=(1.1, 2.45, 0.9, 1.5, 0.7, 1.7),
                 lower=(2.1, 2.8, 0.55, 1.15, 1.05, 1.35), foot=(1.75, 3.2, 0.2, 0.7, 0.3, 2.05)),
       "B": dict(hip=(1.1, 1.45, -1.9), knee=(2.45, 1.4, -1.9), ankle=(2.45, 0.6, -1.9), upper=(0.9, 2.6, 1.2, 1.7, -2.4, -1.4),
                 lower=(2.1, 3.0, 0.45, 1.35, -2.05, -1.75), foot=(1.7, 3.4, 0.0, 0.9, -2.8, -1.0))}
TAIL = [((0, 1.35, -2.8), (0.9, 0.9, 1.8, -6.0, -2.6)), ((0, 1.3, -5.9), (0.55, 1.0, 1.6, -9.6, -5.4)),
        ((0, 1.2, -9.5), (0.3, 1.0, 1.3, -13.0, -9.0))]  # pivot; half width, y0, y1, z0, z1


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 1.4, -1.0)), ("spine_1", "spine_0", (0, 1.4, -0.7)),
         ("neck", "spine_1", (0, 1.5, 2.0)), ("head", "neck", (0, 1.5, 3.0))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B.append((f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2)))
    for k, (pv, _) in enumerate(TAIL):
        B.append((f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", pv))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        g = LEG[leg[0]]
        parent = "spine_1" if leg[0] == "F" else "spine_0"
        mx = lambda p: (s * p[0], p[1], p[2])  # noqa: E731
        B += [(f"leg_{leg}_upper", parent, mx(g["hip"])), (f"leg_{leg}_lower", f"leg_{leg}_upper", mx(g["knee"])),
              (f"leg_{leg}_paw", f"leg_{leg}_lower", mx(g["ankle"]))]
    return B


def elements():
    E = [el("hips", "spine_0", (-1.3, 0.8, -3.0), (1.3, 2.1, -0.6)),
         el("chest", "spine_1", (-1.7, 0.4, -0.8), (1.7, 2.5, 2.5)),
         el("neck", "neck", (-0.8, 1.2, 0.9), (0.8, 1.9, 4.2)),  # deep in the chest and the skull: it turns on them
         el("skull", "head", *SKULL),
         el("snout", "head", (-0.8, 1.2, 4.6), (0.8, 1.9, 6.0))]
    for s, n in ((1, "L"), (-1, "R")):
        # The eyelid: a quad the eye's size lying flat inside the head, hinged at the eye's top; a
        # blink swings it down over the eye. Tilted a few degrees about its hinge, so it is never
        # in a face's plane.
        za, zb, top, bottom = eye_box(s)
        lx0, lx1 = (LID_X - (top - bottom), LID_X) if s > 0 else (-LID_X, -LID_X + (top - bottom))
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down", rot=(0, 0, s * 4),
                    origin=(s * LID_X, top, (za + zb) / 2)))
    for k, (_, (hw, y0, y1, z0, z1)) in enumerate(TAIL):
        # Each segment a little more down than the one before: never parallel to its neighbors.
        E.append(el(f"tail_{k}", f"tail_{k}", (-hw, y0, z0), (hw, y1, z1), rot=(-2.0 * (k + 1), 0, 0), origin=TAIL[k][0]))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        g = LEG[leg[0]]
        for part, key in (("upper", "upper"), ("lower", "lower"), ("paw", "foot")):
            x0, x1, y0, y1, z0, z1 = g[key]
            xa, xb = sorted((s * x0, s * x1))
            E.append(el(f"leg_{leg}_{part}", f"leg_{leg}_{part}", (xa, y0, z0), (xb, y1, z1)))
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
    """Sandy grey-brown, dark blotches across the back and bands round the tail, a pale stripe
    along each side of the back, pale below; a dark line along the lips."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, SKIN)
    side = face in ("east", "west")
    blotch = np.mod(z + 0.5, 2.6) < 1.1
    if name in ("hips", "chest", "neck"):
        if face == "up":
            a = np.where(np.abs(x) > 0.9, PALE, np.where(blotch, BACK, SKIN))
        elif face == "down":
            a[:] = BELLY
        elif side:
            a = np.where(y < 1.1, BELLY, SKIN)
    elif name.startswith("tail_"):
        a = np.where(blotch, BACK, SKIN)
        if face == "down":
            a[:] = BELLY
    elif name in ("skull", "snout"):
        if face == "down":
            a[:] = BELLY
        elif face == "up":
            a[:] = SKIN
        elif side:
            a = np.where(y < 1.2, BELLY, SKIN)
    elif name.startswith("eyelid"):
        a[:] = SKIN
    elif name.startswith("leg_"):
        a = np.where(blotch & (face == "up"), BACK, SKIN)
        if face == "down":
            a[:] = BELLY
    return a


# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def layer_props(e, face, p, a):
    """White where it is pale below; the back's blotches and the tail's bands where marks show."""
    y = p[..., 1]
    white = np.where(a == BELLY, 0.85, 0.15)
    pattern = np.full(y.shape, 1.0 if e["name"] in ("hips", "chest") or e["name"].startswith("tail_") else 0.3)
    return {"white": white, "pattern": pattern}


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
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel centers, in texels
    p = corner + uu[..., None] * du * (abs(size @ du) / w) + vv[..., None] * dv * (abs(size @ dv) / h)  # ... in units
    light_face = face
    if face == "down" and e["name"].startswith(("leg_", "tail")):
        light_face = "north"
    if e["name"].startswith("eyelid"):
        light_face = "east"  # seen only closed, as the side of the head
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, n, du, dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=None, occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=False, merge=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["name"] == "skull" and face in ("east", "west"):
        side = 1 if face == "east" else -1
        EYES["_lizard_side"] = dict(EYES[EYE_STYLE], pattern=eye_pattern(side))
        stamp_eyes(img, [eye_center(side)], "_lizard_side")
    if e["name"] == "snout" and face in ("east", "west"):  # the lips' line, rising a little at the back
        if h >= 3:
            img[h - 1, :, :3] = BASE[MOUTH]
            back = slice(w - 2, w) if face == "east" else slice(0, 2)
            img[h - 2, back, :3] = BASE[MOUTH]
        else:  # at 128 a block the snout's side is one texel tall: the lips a darker side, not a line
            img[..., :3] = (img[..., :3] * 0.55 + np.array(BASE[MOUTH]) * 0.45).astype(np.uint8)
    if e["name"] == "snout" and face == "south":  # the nostrils, two slits at the top corners
        if h >= 3:
            for c in (1, w - 2):  # faint: dark spots on a face read as eyes
                img[0, c, :3] = (img[0, c, :3] * 0.8).astype(np.uint8)
            img[h - 1, :, :3] = (img[h - 1, :, :3] * 0.8).astype(np.uint8)
        else:  # 3 texels by 1: the outer two a little darker
            for c in (0, w - 1):
                img[0, c, :3] = (img[0, c, :3] * 0.82).astype(np.uint8)
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

L = {"bask_drop": 0.6, "bask_splay": 20}


def gait(p, phases, swing_amp, lift):
    """Sprawling: each upper leg swings forward and back about the vertical, lifted (raised out
    to the side) while it swings forward; the lower leg kept upright, the foot flat."""
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
    for k in range(3):
        q[f"tail_{k}"] = (0, -amp * (0.8 + 0.3 * k) * S(TAU * p - 0.6 * (k + 1)), 0)
    return q


def pose_idle(t):
    """Basking alert, then the display: two push-ups on the front legs and quick head bobs, a look
    round, a flick of the tail's tip (4 s)."""
    p = t / 4.0
    push = 0.0
    if 0.3 < p < 0.6:
        push = max(0.0, S(TAU * (p - 0.3) / 0.15))
    bob = 6 * max(0.0, S(TAU * t * 4)) * (0.3 < p < 0.6)
    turn = float(np.interp(p, [0, .65, .72, .9, .95, 1], [0, 0, 18, 18, 0, 0]))
    flick = 12 * max(0.0, S(TAU * (p - 0.8) * 8)) if 0.8 < p < 0.86 else 0.0
    q = {"root": (0, 0.25 * push, 0), "spine_1": (-6 * push, 0, 0), "head": (-bob + 5 * push, turn, 0),
         "tail_2": (0, flick, 0)}
    for leg, s in (("FL", 1), ("FR", -1)):
        q[f"leg_{leg}_upper"] = (0, 0, -s * 12 * push)  # straightening, the shoulders up
        q[f"leg_{leg}_lower"] = (0, 0, s * 12 * push)
    return q


def pose_walk(t):
    p = t / 0.5
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 26, 22)
    q.update(body_wave(p, 7))
    return q


def pose_run(t):
    """A dash, the body low and whipping side to side (0.3 s)."""
    p = t / 0.3
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 34, 26)
    q.update(body_wave(p, 10))
    q["head"] = (-4, 0, 0)
    return q


def pose_bask(t, P=L):
    """Flat on the warm ground, the legs splayed, the head up a little (4 s)."""
    p = t / 4.0
    q = {"root": (0, -P["bask_drop"] + 0.04 * S(TAU * p), 0), "head": (-6, 4 * S(TAU * p), 0)}
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        sp = s * P["bask_splay"]
        q[f"leg_{leg}_upper"] = (0, -s * (12 if leg[0] == "F" else -12), sp)
        q[f"leg_{leg}_lower"] = (0, 0, -sp)
    for k in range(3):
        q[f"tail_{k}"] = (0, 6 * (k + 1) * (1 if k != 1 else -1), 0)
    return q


ANIMS = {"idle": (pose_idle, 4.0), "walk": (pose_walk, 0.5), "run": (pose_run, 0.3), "bask": (pose_bask, 4.0)}
BLINKS = {"idle": [0.6, 3.3], "bask": [2.0]}  # basking, a slow blink
BLINK = {"idle": 0.08, "bask": 0.3}


def eyelids(name, t):
    d = BLINK.get(name, 0.08)
    a = 90 * max([0.0] + [1 - abs(t - b) / d for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}  # side eyes: the lids swing down about z


def frame_times(name, dur):
    n = {"idle": 96, "bask": 48, "walk": 20, "run": 20}.get(name, 16)
    ts = {round(dur * k / n, 4) for k in range(n + 1)}
    d = BLINK.get(name, 0.08)
    for b in BLINKS.get(name, []):
        ts |= {round(b + x, 4) for x in (-d, -d / 2, 0, d / 2, d)}
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

GROUPS = {"feet": lambda n: n.endswith("_paw"), "body": lambda n: n in ("hips", "chest"),
          "tail": lambda n: n.startswith("tail_")}


def fit_poses(spec):
    """Basking: the belly on the ground, the feet flat beside it, nothing below."""
    def cost(P):
        g = lowest(spec, pose_bask(0.0, dict(L, **P, bask_drop=0)), GROUPS)
        f = min(g.values())
        return (g["body"] - f) ** 2 + (g["feet"] - f) ** 2 + 0.5 * max(0.0, g["tail"] - f - 0.3) ** 2
    best = fit({"bask_splay": list(range(0, 61, 4))}, cost)[1]
    g = lowest(spec, pose_bask(0.0, dict(L, **best, bask_drop=0)), GROUPS)
    L.update(best, bask_drop=round(float(min(g.values())), 2))


# ------------------------------------------------------------------ build


def build():
    B = bones()
    E = elements()
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"]]
    LAYERS["rec"] = Recorder(layer_bounds(E, 1.0))
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
    spec = {"name": "lizard", "texture": [sheet.shape[1], sheet.shape[0]],
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
