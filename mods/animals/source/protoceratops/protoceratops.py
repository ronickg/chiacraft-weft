#!/usr/bin/env python3
"""Protoceratops: the Djadokhta's plant eater and the Velociraptors' prey, made the way Hytale
makes creatures (boxes and a hand-painted texture) with the blocky-model skill, from research
alone (README.md): its colors are rules (see `area`), a sandy hide pebbled with small scales, dark
bands across the back and tail.

    ~/.venvs/blocky/bin/python protoceratops.py      # writes out/spec.json, out/texture.png

Units are texels at the house density (64 a block, 1.56 cm each), y up, it faces +z. An adult
*Protoceratops andrewsi* is about 1.8 m long and 0.6 m at the hips: here 126 from the beak to the
tail's tip, the hips 38 up; the head (beak, cheek flanges, the frill over the neck) a little bigger
than life.
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
from rig import fit  # noqa: E402
from ground import lowest  # noqa: E402

OUT = os.environ.get("PROTO_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

SKIN, BACK, BELLY, RIM, BEAK, MOUTH, CLAW = range(7)
# SKIN the sandy hide, BACK the darker brown of the back's bands and the frill's rim, BELLY the
# pale underside, RIM the dark gaps between its small scales, BEAK dark horn, MOUTH the inside,
# CLAW the blunt claws.
BASE = [(184, 150, 102), (112, 82, 56), (220, 204, 168), (160, 128, 90), (70, 60, 54), (110, 56, 52), (82, 70, 60)]
SHADOW = [0.03, 0.03, 0.07, 0.03, 0.74, 0.95, 0.74]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (SKIN, BACK, BELLY, RIM)
ROLES = [("coat", 0, 0), ("coat", 0, -1.2), ("coat", 1, 0), ("coat", 0.3, -0.6), "fixed", "fixed", "fixed"]

# Eyes on the sides of the head: a dark ring, an amber-brown iris round a dark pupil, a catchlight.
EYES.update({
    "proto": dict(pattern=["..OOO..",
                           ".OHPPO.",
                           "OIHPPIO",
                           "OIPPPIO",
                           ".OIaIO.",
                           "..OOO.."], colors={"O": (42, 32, 26), "P": (20, 16, 14), "H": (226, 226, 222),
                                               "I": (150, 104, 48), "a": (190, 138, 64)}, halo="below"),
})
EYE_STYLE = os.environ.get("PROTO_EYES", "proto")
SKULL = ((-6.5, 26.0, 24.0), (6.5, 40.0, 43.0))
EYE_Z, EYE_Y = 37.5, 35.0
LID_X = 6.9  # the eyelids' hinge plane, just outside the head's side

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


def eye_box(side):
    pat = EYES[EYE_STYLE]["pattern"]
    ph, pw = len(pat), len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    cu = (z1 - EYE_Z) if side > 0 else (EYE_Z - z0)
    u0 = int(round(cu - pw / 2))
    v0 = int(round(y1 - EYE_Y - ph / 2))
    za, zb = (z1 - u0 - pw, z1 - u0) if side > 0 else (z0 + u0, z0 + u0 + pw)
    return za, zb, y1 - v0, y1 - v0 - ph


LEGS = ("FL", "FR", "BL", "BR")
FRONT_X, FRONT_Z, BACK_X, BACK_Z = 9.0, 14.0, 8.5, -12.0
HEAD = (0.0, 33.0, 27.0)
FRILL0 = (0.0, 37.0, 24.0)  # where the frill rises from the back of the skull
TAIL = [((0, 37.0, -22.0), (6.2, 28.0, 44.5, -40.0, -21.0)), ((0, 35.5, -39.5), (4.2, 29.5, 41.5, -56.0, -38.5)),
        ((0, 35.0, -55.5), (2.6, 31.0, 39.0, -71.0, -54.5))]  # pivot; half width, y0, y1, z0, z1


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 38, -8)), ("spine_1", "spine_0", (0, 36, 4)),
         ("neck", "spine_1", (0, 32, 20)), ("head", "neck", HEAD), ("jaw", "head", (0, 26.6, 31.0))]
    for k, (pv, _) in enumerate(TAIL):
        B.append((f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", pv))
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B.append((f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2)))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x = FRONT_X * s
            B += [(f"leg_{leg}_upper", "spine_1", (x, 28, FRONT_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 15.5, FRONT_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 4.5, FRONT_Z + 0.5))]
        else:
            x = BACK_X * s
            B += [(f"leg_{leg}_upper", "spine_0", (x, 38, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 21, BACK_Z + 1)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 5.5, BACK_Z))]
    return B


def elements():
    E = [el("hips", "spine_0", (-9.5, 26, -24), (9.5, 46, 6)),  # highest over the hips, the back sloping to the shoulders
         el("chest", "spine_1", (-11.2, 21, 0), (11.2, 39.4, 22)),
         el("neck", "neck", (-5.5, 27, 18), (5.5, 37.5, 30)),  # short and thick, slimmer than the skull
         el("skull", "head", *SKULL),
         el("snout", "head", (-4.2, 27.8, 42.5), (4.2, 38, 51)),
         el("beak", "head", (-2.8, 25.6, 50.4), (2.8, 34.5, 55)),  # the hooked tip of the upper beak
         el("nasal", "head", (-1.8, 37.6, 44.5), (1.8, 40.4, 48.5)),  # the bump on the nose
         el("mouth", "head", (-3.4, 26.6, 40), (3.4, 28.4, 50)),
         el("jaw", "jaw", (-4.6, 22.5, 29), (4.6, 27.0, 49.5)),  # shorter than the upper beak
         # The frill: a shield rising up and back over the neck from the back of the skull, thick
         # enough to read as a plate from the side (a thin one read as a stick).
         el("frill", "head", (-10.5, 34.5, 19.6), (10.5, 50.5, 24.6), rot=(-60, 0, 0), origin=FRILL0)]
    for s, n in ((1, "L"), (-1, "R")):
        x0, x1 = sorted((s * 6.0, s * 8.4))
        E.append(el(f"jugal_{n.lower()}", "head", (x0, 24.5, 33), (x1, 30, 38)))  # the cheek flange under the eye
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        lx0, lx1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down"))
    for k, (_, (hw, y0, y1, z0, z1)) in enumerate(TAIL):
        E.append(el(f"tail_{k}", f"tail_{k}", (-hw, y0, z0), (hw, y1, z1)))  # tall: the spines along its top
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z = FRONT_X * s, FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 3.4, 14, z - 3.2), (x + 3.4, 31, z + 3.6)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2.6, 4, z - 2.6), (x + 2.6, 16.5, z + 2.8)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 3.0, 0, z - 3.4), (x + 3.0, 5.2, z + 4.8)))
        else:
            x, z = BACK_X * s, BACK_Z
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (x - 3.7, 19.5, z - 6.2), (x + 3.7, 42, z + 6.6)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2.3, 5, z - 2.4), (x + 2.3, 22, z + 3.2)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 3.3, 0, z - 3.4), (x + 3.3, 6, z + 9.4)))
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


def pebbles(e, face, p):
    """Small rounded scales with dark gaps (a lizard's hide): which texels are the gaps. Bigger on
    the back and flanks, smaller on the head, legs and toward the tail's tip."""
    name = e["name"]
    if name.startswith(("leg_", "jugal")) or name in ("skull", "snout", "jaw", "neck"):
        size = 3.2
    elif name.startswith("tail_"):
        size = 4.4 - 0.5 * int(name[-1])
    else:
        size = 5.6
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a, b = {"east": (z, y), "west": (z, y), "south": (x, y), "north": (x, y), "up": (x, z), "down": (x, z)}[face]
    row = size * 0.87
    j = np.floor(b / row)
    a2 = a / size + 0.5 * (j % 2)
    fa, fb = a2 - np.floor(a2) - 0.5, b / row - j - 0.5
    return (fa * size) ** 2 + (fb * row) ** 2 > (0.5 * size - 0.45) ** 2


def area(e, face, p, uu, vv, h):
    """Sandy above, pale below, dark bands across the back and down the tail, a dark rim to the
    frill; the beak dark horn, the claws darker."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, SKIN)
    side = face in ("east", "west")
    t = vv / max(h, 1)
    band = np.mod(z + 3.0, 15.0) < 5.5  # bands across the back, every 15 texels
    if name in ("hips", "chest", "neck") or name.startswith("tail_"):
        if face == "up":
            a = np.where(band | (np.abs(x) < 1.5), BACK, SKIN)
        elif face == "down":
            a[:] = BELLY
        elif side:
            a = np.where(t < 0.12, BACK, np.where(band & (t < 0.5), BACK, np.where(t > 0.74, BELLY, SKIN)))
        elif face in ("south", "north"):
            a = np.where(t > 0.74, BELLY, np.where(t < 0.12, BACK, SKIN))
    elif name == "skull":
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
    elif name in ("snout", "nasal"):
        a[:] = SKIN
        if face == "up":
            a[:] = BACK
    elif name == "beak":
        a[:] = BEAK
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else np.where(z > 45.5, BEAK, BELLY)  # the lower beak's horny tip
    elif name == "mouth":
        a[:] = MOUTH
    elif name == "frill":
        lo, hi = np.array(e["lo"]), np.array(e["hi"])
        q = (p - np.array(e["origin"])) @ rotm(e["rot"]) + np.array(e["origin"])  # back in the plate's own frame
        edge = np.minimum(np.minimum(q[..., 0] - lo[0], hi[0] - q[..., 0]), hi[1] - q[..., 1])
        a = np.where(edge < 2.6, BACK, SKIN)
    elif name.startswith("jugal"):
        a = np.where(y < 26.5, BACK, SKIN)
    elif name.startswith("thigh_"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        else:
            a = np.where(band & (t < 0.4), BACK, np.where(t > 0.8, BELLY, SKIN))
    elif name.startswith("leg_"):
        if name.endswith("_paw"):
            a = np.where((face == "south") & (y < 2.2), CLAW, SKIN)
            if face == "down":
                a[:] = BELLY
        elif side and ((e["lo"][0] > 0) == (face == "west")):
            a[:] = BELLY  # the inside of the legs
    if name not in ("beak", "mouth", "nasal") and not name.startswith("eyelid"):
        gap = pebbles(e, face, p) & np.isin(a, (SKIN, BELLY))
        a = np.where(gap, RIM, a)
    return a


LASH = (52, 40, 32)

# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def layer_props(e, face, p, a):
    """White where this one is white (the pale belly); the back's bands where marks show."""
    y = p[..., 1]
    white = np.where(a == BELLY, 0.85, 0.15)
    ext = np.where(a == BEAK, 1.0, 0.0)
    pattern = np.full(y.shape, 1.0 if e["name"] in ("hips", "chest") or e["name"].startswith(("tail_", "thigh_")) else 0.3)
    return {"white": white, "extremity": ext, "pattern": pattern}


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
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=False, merge=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["name"] == "skull" and face in ("east", "west"):
        (x0, y0, z0), (x1, y1, z1) = SKULL
        cu = (z1 - EYE_Z) if face == "east" else (EYE_Z - z0)
        stamp_eyes(img, [(cu, y1 - EYE_Y)], EYE_STYLE)
    if e["name"] == "snout" and face in ("east", "west"):  # the nostril, a slit high on the snout
        r, c = 3, (w - 4 if face == "east" else 3)
        img[r, c - 1:c + 1, :3] = (46, 34, 28)
    if e["name"].startswith("eyelid"):
        col_idx = 0 if e["name"].endswith("L") else w - 1
        img[:, col_idx, :3] = LASH
    if e["name"].endswith("_paw") and face == "south":  # toes: grooves on the front
        for c in range(2, w - 1, 2):
            img[:, c, :3] = (img[:, c, :3] * 0.72).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"lie_front": 80, "lie_hip": -70, "lie_knee": 140, "lie_drop": 20, "graze_neck": 30, "graze_head": 20, "graze_lean": 4}


def gait(p, phases, swing_amp, lift):
    """A foot swings forward while it's lifted, back while it's down; the knees bend forward, the
    feet kept about flat."""
    out = {}
    for leg, ph in zip(LEGS, phases):
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))
        swing = -swing_amp * S(a)
        out[f"leg_{leg}_upper"] = (swing, 0, 0)
        knee = lift * up
        if leg[0] == "F":
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (knee, 0, 0), (-0.6 * knee, 0, 0)
        else:
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (knee, 0, 0), (-0.8 * knee, 0, 0)
    return out


def chew(t, rate=2.0, amp=6.0):
    return {"jaw": (amp * max(0.0, S(TAU * t * rate)), 0, 0)}


def graze_pose(g, P=L):
    """Neck and head down to the ferns by `g` (0..1), the chest a little lower."""
    q = {"spine_1": (P["graze_lean"] * g, 0, 0), "neck": (P["graze_neck"] * g, 0, 0), "head": (P["graze_head"] * g, 0, 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (-P["graze_lean"] * g, 0, 0)
    return q


def tail_sway(t, amp, rate):
    return {f"tail_{k}": (2 + k, amp * (0.6 + 0.4 * k) * S(TAU * (t * rate - 0.12 * k)), 0) for k in range(3)}


def pose_idle(t):
    """Cropping ferns and chewing, then the head up to look about, the tail swaying slowly (8 s)."""
    p = t / 8.0
    br = S(TAU * p * 3)
    g = float(np.interp(p, [0, .5, .58, .86, .94, 1], [1, 1, 0, 0, 1, 1]))
    look = (1 - g) * 16 * S(TAU * (p - 0.58) * 1.6)
    q = graze_pose(g)
    q.update({"root": (0, 0.2 * br, 0), "head": (q["head"][0], look, 0)})
    q.update(tail_sway(t, 4, 0.25))
    q.update(chew(t, 2.0 if g > 0.5 else 0.8, 6 if g > 0.5 else 1.5))
    return q


def pose_walk(t):
    p = t / 1.2
    q = gait(p, (0.0, 0.5, 0.75, 0.25), 18, 34)
    q.update({"root": (0, 0.5 * abs(S(TAU * p * 2)), 0), "spine_0": (0, 2.5 * S(TAU * p), 0), "head": (-2 * S(TAU * p * 2), -2 * S(TAU * p), 0)})
    q.update(tail_sway(t, 7, 1 / 1.2))
    return q


def pose_run(t):
    """A heavy trot, the head held low and forward (0.6 s)."""
    p = t / 0.6
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 28, 54)
    q.update({"root": (0, 1.6 * abs(S(TAU * p)), 0), "neck": (6, 0, 0), "head": (-4, 0, 0)})
    q.update(tail_sway(t, 9, 1 / 0.6))
    return q


def pose_lie(t, P=L):
    """Lying on its belly, the legs folded under, the head resting low (6 s)."""
    p = t / 6.0
    br = S(TAU * p * 2)
    q = {"root": (0, -P["lie_drop"] + 0.2 * br, 0), "spine_1": (0.6 * br, 0, 0), "neck": (8, 0, 0), "head": (6, 6 * S(TAU * p), 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (P["lie_front"], 0, 0)
        q[f"leg_{leg}_lower"] = (-150, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_front"] + 150 - 40, 0, 0)
    for leg, s in (("BL", 1), ("BR", -1)):
        q[f"leg_{leg}_upper"] = (P["lie_hip"], 0, s * 6)
        q[f"leg_{leg}_lower"] = (P["lie_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_hip"] - P["lie_knee"], 0, 0)
    q.update(tail_sway(t, 3, 1 / 6.0))
    for k in range(3):
        q[f"tail_{k}"] = (q[f"tail_{k}"][0] + 4, q[f"tail_{k}"][1], 0)
    return q


BLINKS = {"idle": [2.0, 6.4], "lie": [1.5, 4.4]}
BLINK = 0.12
ANIMS = {"idle": (pose_idle, 8.0), "walk": (pose_walk, 1.2), "run": (pose_run, 0.6), "lie": (pose_lie, 6.0)}


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}


def frame_times(name, dur):
    n = {"idle": 96, "lie": 60, "walk": 24, "run": 20}.get(name, 16)
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
          "body": lambda n: n in ("hips", "chest"), "muzzle": lambda n: n in ("beak", "jaw"),
          "legs": lambda n: n.startswith(("leg_", "thigh_")), "frill": lambda n: n == "frill"}


def fit_poses(spec):
    """Lying (the belly on the ground, the folded legs under it, nothing below) and cropping (the
    beak at the ferns), by forward kinematics."""
    def lie_cost(P):
        g = lowest(spec, pose_lie(0.0, dict(L, **P, lie_drop=0)), GROUPS)
        f = min(g.values())
        return (g["body"] - f - 0.8) ** 2 + 0.3 * (g["front paws"] - f) ** 2 + 0.3 * (g["back paws"] - f) ** 2
    grid = {"lie_front": list(range(20, 101, 5)), "lie_hip": list(range(-110, -24, 5)), "lie_knee": list(range(60, 166, 5))}
    best = fit(grid, lie_cost)[1]
    g = lowest(spec, pose_lie(0.0, dict(L, **best, lie_drop=0)), GROUPS)
    L.update(best, lie_drop=round(float(min(g.values())), 2))

    def graze_cost(P):
        g = lowest(spec, graze_pose(1.0, dict(L, **P)), GROUPS)
        return (g["muzzle"] - 0.5) ** 2 + 0.002 * (P["graze_neck"] ** 2 + P["graze_head"] ** 2) + 0.05 * P["graze_lean"] ** 2 \
            + 40 * min(0.0, g["front paws"]) ** 2
    best = fit({"graze_neck": list(range(0, 71, 5)), "graze_head": list(range(-10, 61, 5)), "graze_lean": [0, 3, 6, 9, 12]}, graze_cost)[1]
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
            tiles.append(((i, f), paint_face(e, f, boxes)))
    sheet, uv = pack(tiles, 512)  # the layers pack at the same width (`Recorder.save`)
    LAYERS["rec"].save(OUT, [k for k, _ in tiles], pack)
    LAYERS["rec"] = None
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "protoceratops", "texture": [sheet.shape[1], sheet.shape[0]],
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
