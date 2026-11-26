#!/usr/bin/env python3
"""Protoceratops: the Djadokhta's plant eater and the Velociraptors' prey, made the way Hytale
makes creatures (boxes and a hand-painted texture) with the blocky-model skill, from research
alone (README.md): its colors are rules (see `area`), a sandy hide pebbled with small scales, dark
bands across the back and tail.

    ~/.venvs/blocky/bin/python protoceratops.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each), y up, it faces +z; painted at the house density,
32 texels a block (`TX`, half a texel a unit), as the blocks and items are. An adult
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
from paint import EYES, eye_corner, occlusion, pack, paint_face_smooth, stamp_eyes, tones  # noqa: E402
from rig import fit  # noqa: E402
from ground import grounded, lowest  # noqa: E402
from house import TX  # noqa: E402

# TX: texels a unit, 0.5 (32 a block). The eye's face, the skull's side, is 10 by 7 texels at 32 (the
# house rule wants 3 or more), so it paints at 32.

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
# At 32 a block the skull's side is 10 texels by 7: the eye is a few texels, the same parts (a dark
# rim, the amber iris round a dark pupil, a catchlight up and forward). Each is stamped as seen on
# the left side; the right side's is its mirror, so both catchlights point forward.
_O, _P, _H, _I, _a = (42, 32, 26), (20, 16, 14), (226, 226, 222), (150, 104, 48), (190, 138, 64)
EYES.update({
    "proto_px": dict(pattern=["HP", "IP"], colors={"P": _P, "H": _H, "I": _I}, halo="none", iris="I"),
    "proto_px3": dict(pattern=["HPI", "IPa"], colors={"P": _P, "H": _H, "I": _I, "a": _a}, halo="none", iris="Ia"),
    "proto_px_rim": dict(pattern=["OHPO", "OIPO"], colors={"O": _O, "P": _P, "H": _H, "I": _I}, halo="none", iris="I"),
    "proto_px3_rim": dict(pattern=["OOO", "HPI", "OIO"], colors={"O": _O, "P": _P, "H": _H, "I": _I}, halo="none",
                          iris="I"),
})
for _k in [k for k in EYES if k.startswith("proto_px")]:
    EYES[_k + "_m"] = dict(EYES[_k], pattern=[row[::-1] for row in EYES[_k]["pattern"]])
EYE_STYLE = os.environ.get("PROTO_EYES", "proto_px_rim")  # the old ringed eye, 4 by 2: reads best from afar
SKULL = ((-6.5, 26.0, 24.0), (6.5, 40.0, 43.0))
EYE_Z, EYE_Y = 37.5, 35.0
LID_X = 6.9  # the eyelids' hinge plane, just outside the head's side
NOSTRIL = (35.0, 47.5)  # (y, z) of the nostril on the snout's sides

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


def skull_grid():
    """The skull's side in texels: (width along z, height) and the units a texel spans (u, v)."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h = max(1, round(TX * (z1 - z0))), max(1, round(TX * (y1 - y0)))
    return w, h, (z1 - z0) / w, (y1 - y0) / h


def eye_style(side):
    """The design on that side (+1 left, the east face; -1 right): the right one mirrored."""
    return EYE_STYLE + ("_m" if side < 0 and EYE_STYLE + "_m" in EYES else "")


def eye_center(side):
    """The eye's center on the skull's side (east face for side > 0, west otherwise), in its texels."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    cu = (z1 - EYE_Z) if side > 0 else (EYE_Z - z0)
    return cu / su, (y1 - EYE_Y) / sv


def eye_box(side):
    """(z0, z1, top, bottom) of the painted eye, from the same corner `stamp_eyes` uses, so the
    eyelid covers it to the texel."""
    pat = EYES[eye_style(side)]["pattern"]
    ph, pw = len(pat), len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    u0, v0 = eye_corner(*eye_center(side), pat, w)
    za, zb = (z1 - (u0 + pw) * su, z1 - u0 * su) if side > 0 else (z0 + u0 * su, z0 + (u0 + pw) * su)
    return za, zb, y1 - v0 * sv, y1 - (v0 + ph) * sv


LEGS = ("FL", "FR", "BL", "BR")
# The front legs at 7.35 (at 9 the shoulders stood 1.2 out of the chest: now 0.45 in; linecheck).
FRONT_X, FRONT_Z, BACK_X, BACK_Z = 7.35, 14.0, 8.5, -12.0
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
    # Highest over the hips, the back sloping to the shoulders; half a unit wider than the chest, the thighs half a
    # unit past them (at 9.5 a 1.7 strip of the chest's top showed beside them; linecheck).
    E = [el("hips", "spine_0", (-11.7, 26, -24), (11.7, 46, 6)),
         # Its front half a unit short of the skull's back (at 22 a texel of neck showed between them; linecheck).
         el("chest", "spine_1", (-11.2, 21, 0), (11.2, 39.4, 23.6)),
         el("neck", "neck", (-5.5, 27, 18), (5.5, 37.5, 30)),  # short and thick, slimmer than the skull
         el("skull", "head", *SKULL),
         el("snout", "head", (-4.2, 27.8, 42.5), (4.2, 38, 51)),
         # The hooked tip of the upper beak: out 5 from the snout, its top 4.1 under the snout's, its sides half
         # a unit in from the snout's and its hook 4 below it (out 4, 3.5 under, 1.4 in and 2.2 below, each read
         # as a rim; linecheck).
         el("beak", "head", (-3.8, 23.8, 50.4), (3.8, 33.5, 56)),
         # The bump on the nose, over the snout's front: as wide as the snout and half a unit past its front, its
         # back 2 texels clear of the skull (a narrower bump mid-snout left a 1.2-texel ring of snout top; linecheck).
         # 4.8 proud of the snout, its back three texels tall (2.4, a texel, read as a lip; linecheck).
         el("nasal", "head", (-4.7, 37.6, 47.4), (4.7, 42.8, 51.5)),
         el("mouth", "head", (-3.4, 26.6, 40), (3.4, 28.4, 50)),
         el("jaw", "jaw", (-4.6, 21.5, 29), (4.6, 27.0, 49.5)),  # shorter than the upper beak; down 1 to show 2 texels under the skull (1.5 read as a rim; linecheck)
         # The frill: a shield rising up and back over the neck from the back of the skull, thick
         # enough to read as a plate from the side (a thin one read as a stick).
         el("frill", "head", (-10.5, 34.5, 19.6), (10.5, 50.5, 24.6), rot=(-60, 0, 0), origin=FRILL0)]
    for s, n in ((1, "L"), (-1, "R")):
        # The cheek flange under the eye, flaring 4.2 out from the skull's side (its faces three texels across),
        # its bottom flush with the skull's (out 1.9 and 1.5 below, its top, back and inside read as rims; linecheck).
        x0, x1 = sorted((s * 5.6, s * 10.7))
        E.append(el(f"jugal_{n.lower()}", "head", (x0, 25.6, 33), (x1, 30, 38)))
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
            # Its sides and back half a unit out from the shank's (rule 8: at 1 out its top showed a texel-thin rim
            # round the shank; linecheck).
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 2.8, 0, z - 2.9), (x + 2.8, 6, z + 9.4)))
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
    # Its size in texels at TX (a face of any size is at least a texel, or it would vanish).
    return np.array(corner, float), du, dv, max(1, int(round(TX * abs(size @ du)))), max(1, int(round(TX * abs(size @ dv)))), n


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
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=False, merge=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["name"] == "skull" and face in ("east", "west"):
        side = 1 if face == "east" else -1
        stamp_eyes(img, [eye_center(side)], eye_style(side))
    if e["name"] == "snout" and face in ("east", "west"):  # the nostril, a texel high on the snout, near its front
        lo, hi = e["lo"], e["hi"]
        r = min(h - 1, int((hi[1] - NOSTRIL[0]) / sv))
        c = min(w - 1, int(((hi[2] - NOSTRIL[1]) if face == "east" else (NOSTRIL[1] - lo[2])) / su))
        img[r, c, :3] = (46, 34, 28)
    if e["name"].startswith("eyelid"):
        col_idx = 0 if e["name"].endswith("L") else w - 1
        img[:, col_idx, :3] = LASH
    if e["name"].endswith("_paw") and face == "south" and w >= 5:  # toes: grooves on the front (a paw is 3 texels at 32: none)
        for c in range(2, w - 1, 2):
            img[:, c, :3] = (img[:, c, :3] * 0.72).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"lie_front": 80, "lie_hip": -70, "lie_knee": 140, "lie_drop": 20, "graze_neck": 30, "graze_head": 20, "graze_lean": 4,
     "clip_chew": 6.0, "sleep_neck": 8, "sleep_head": 3.15,
     "charge_neck": 14, "charge_head": 18, "charge_level": 0.0, "death_drop": 26.0}


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
    # The head down 3.15 (6 before the jaw went a unit deeper): the chin as low as before, so the legs fold as they did (the fit is touchy).
    q = {"root": (0, -P["lie_drop"] + 0.2 * br, 0), "spine_1": (0.6 * br, 0, 0), "neck": (8, 0, 0), "head": (3.15, 6 * S(TAU * p), 0)}
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


# ------------------------------------------------------------------ behaviour clips (`anim.rs`)
# Each a whole-body pose over what plays below on the bones it moves (`to_bedrock.as_action`): held
# states and the gaits loop, the rest play once. A mother (83 kg, over `roles::GUARDS_KG`) stands her
# ground for her young: she rears (a threat display, not up on her hind legs: a heavy four-footed
# animal), stares, then charges.


def still(t, period=4.0, tail=1.5):
    """Standing still and breathing, the tail swaying a little: what a clip builds on. (No spine
    pitch: the front legs hang from the chest, and a tilt lifts or sinks their feet.)"""
    br = S(TAU * t / period * 2)
    q = {"root": (0, 0.15 * br, 0), "spine_1": (0.4 * br, 0, 0)}
    q.update(tail_sway(t, tail, 1 / period))
    return q


def pose_stare(t):
    """Half aware: frozen, the head up and level, the frill raised behind it, only the breath moving (2 s, held)."""
    q = still(t, 2.0, tail=0.5)
    q.update({"neck": (-12, 0, 0), "head": (4, 0, 0), "jaw": (0.5, 0, 0)})
    return q


def pose_graze(t):
    """Cropping ferns: the beak at the ground, biting and chewing, the head working side to side (2.5 s, held)."""
    p = t / 2.5
    q = still(t, 2.5, tail=2.0)
    q.update(graze_pose(1.0, L))
    # (the head works side to side, not the neck: turned while it is down, its corner pokes out of the chest)
    q["head"] = (q["head"][0] + 2 * max(0.0, S(TAU * p * 2)), 8 * S(TAU * p), 0)
    q.update(chew(t, 2.0, L["clip_chew"]))
    return q


def pose_drink(t):
    """Drinking: the beak in the water, dipping and sipping (1.5 s, held)."""
    q = still(t, 1.5, tail=1.0)
    q.update(graze_pose(1.0, L))
    q["head"] = (q["head"][0] + 2 * S(TAU * t / 1.5 * 2), 0, 0)
    q.update(chew(t, 2.0, 0.6 * L["clip_chew"]))
    return q


def pose_sniff(t):
    """The beak down to the ground, a sniff along it side to side, and up (2.2 s, once)."""
    p = t / 2.2
    d = float(np.interp(p, [0, .25, .75, 1], [0, 1, 1, 0]))
    q = still(t, 2.2)
    q.update(graze_pose(d, L))
    q["head"] = (q["head"][0], 10 * d * S(TAU * p * 2), 0)
    return q


def pose_look(t):
    """The head up and round one way, then the other (3 s, once)."""
    p = t / 3.0
    yaw = float(np.interp(p, [0, .15, .4, .55, .8, 1], [0, 40, 40, -35, -35, 0]))
    up = float(np.interp(p, [0, .12, .88, 1], [0, 1, 1, 0]))
    q = still(t, 3.0)
    q.update({"neck": (-12 * up, 0.65 * yaw, 0), "head": (4 * up, 0.35 * yaw, 0)})
    return q


def pose_scratch(t):
    """An itch: a hind foot brought forward under the belly, raking the flank, the head turned to
    it (2 s, once)."""
    p = t / 2.0
    d = float(np.interp(p, [0, .15, .85, 1], [0, 1, 1, 0]))
    rake = d * S(TAU * p * 6)
    q = still(t, 2.0)
    q.update({"leg_BL_upper": (-34 * d + 7 * rake, 0, -6 * d), "leg_BL_lower": (46 * d - 8 * rake, 0, 0),
              "leg_BL_paw": (-12 * d, 0, 0), "neck": (6 * d, 24 * d, 0), "head": (2 * d, 10 * d, 6 * d)})
    return q


def pose_groom(t):
    """Grooming: the left forefoot lifted a little and the beak brought down to it, nibbling (2.5 s,
    once). (The chest turned toward it, not the neck: the short neck turned while it is down pokes
    out of the chest.)"""
    p = t / 2.5
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    q = still(t, 2.5)
    q.update({"spine_1": (q["spine_1"][0], 6 * d, 0), "neck": (30 * d, 2 * d, 0),
              "head": (14 * d + 3 * d * S(TAU * p * 3), 14 * d, 0),
              "leg_FL_upper": (-16 * d, 0, 0), "leg_FL_lower": (36 * d, 0, 0), "leg_FL_paw": (-10 * d, 0, 0)})
    q.update(chew(t, 4.0, 7.0 * d))
    return q


def pose_stretch(t):
    """Getting up: the neck stretched long and up, a yawn, then a hind leg pushed out behind (2.4 s, once)."""
    p = t / 2.4
    a = float(np.interp(p, [0, .2, .5, .6, 1], [0, 1, 1, 0, 0]))
    b = float(np.interp(p, [0, .55, .7, .9, 1], [0, 0, 1, 1, 0]))
    q = still(t, 2.4)
    q.update({"neck": (-16 * a, 0, 0), "head": (-6 * a, 0, 0),
              # pushed out behind, the knee lifting it clear on the way (swung straight back, its toe dips into the ground)
              "leg_BR_upper": (30 * b, 0, 0), "leg_BR_lower": (30 * S(math.pi * b), 0, 0),
              "leg_BR_paw": (-10 * b - 24 * S(math.pi * b), 0, 0)})
    q.update(chew(t, 0.5, 20 * a))  # the yawn
    q["tail_0"] = (q["tail_0"][0] - 6 * b, q["tail_0"][1], 0)
    return q


def pose_sleep(t, P=L):
    """Asleep, lying: the head laid on the ground, turned a little, the tail curled round, eyes shut,
    a slow breath (6 s, held)."""
    q = pose_lie(0.0, P)
    br = S(TAU * t / 6.0 * 2)
    q.update({"root": (0, -P["lie_drop"] + 0.12 * br, 0), "spine_1": (0.5 * br, 0, 0), "neck": (P["sleep_neck"], 10, 0),
              "head": (P["sleep_head"], 6, 0), "jaw": (0.01, 0, 0)})
    for k, a in enumerate((12, 6, 6)):  # curled round to the left, each segment a little (more pokes out of the one before)
        q[f"tail_{k}"] = (6 + k, a, 0)
    q.update({"eyelid_L": (0, 0, 90), "eyelid_R": (0, 0, -90)})
    return q


def pose_alarm(t):
    """Alarm: the head flung up, tossed and brought down hard with a snap of the beak, twice, the
    tail raised (1.2 s, once)."""
    p = t / 1.2
    up = float(np.interp(p, [0, .1, .82, 1], [0, 1, 1, 0]))
    toss = float(np.interp(p, [0, .12, .22, .3, .45, .55, .63, 1], [0, 0, -1, 1, 0, -1, 1, 0]))
    snap = float(np.interp(p, [0, .2, .26, .3, .53, .59, .63, 1], [0, 0, 1, 0, 0, 1, 0, 0]))
    q = still(t, 1.2, tail=1.0)
    q.update({"neck": (-18 * up + 6 * toss, 0, 0), "head": (6 * up + 10 * toss, 0, 0), "jaw": (22 * snap, 0, 0)})
    q["tail_0"] = (q["tail_0"][0] - 10 * up, q["tail_0"][1], 0)
    return q


def pose_rear(t):
    """A mother's threat display, as the stand begins: the forequarters lifted a little off the
    ground, the head tipped down and round to turn the frill's face broadside at the threat, a gape;
    the hind feet planted (its height set from them frame by frame, `build`) (1.4 s, once)."""
    p = t / 1.4
    r = float(np.interp(p, [0, .25, .7, 1], [0, 1, 1, 0]))
    gape = float(np.interp(p, [0, .3, .4, .55, .65, 1], [0, 0, 1, 1, 0, 0]))
    tip = r * S(TAU * min(1.0, max(0.0, (p - .25) / .45)) * 0.5)  # round to one side and back
    q = still(t, 1.4, tail=0.6)
    # (the head tipped mostly at the neck: more than about 12 degrees against it, its corner pokes out of the skull)
    q.update({"spine_0": (-9 * r, 0, 0), "neck": (18 * r, 14 * tip, 0), "head": (11 * r, 6 * tip, -10 * tip), "jaw": (18 * gape, 0, 0)})
    for leg in ("BL", "BR"):  # the hind legs kept upright under the tilt
        q[f"leg_{leg}_upper"] = (9 * r, 0, 0)
    for leg in ("FL", "FR"):  # the forefeet lifted, reaching forward
        q[f"leg_{leg}_upper"] = (-6 * r, 0, 0)
        q[f"leg_{leg}_lower"] = (14 * r, 0, 0)
        q[f"leg_{leg}_paw"] = (-8 * r, 0, 0)
    q["tail_0"] = (q["tail_0"][0] - 8 * r, q["tail_0"][1], 0)
    return q


def pose_play(t):
    """A young one's play: two bounces off all four feet, the head tossed side to side, the tail
    wagging (1.2 s, once; it lands on the ground, `build`)."""
    p = t / 1.2
    hop = max(0.0, S(TAU * p * 2))
    q = still(t, 1.2)
    q.update({"root": (0, 4.0 * hop, 0), "neck": (-8 * hop, 12 * S(TAU * p), 0), "head": (4 * hop, 8 * S(TAU * p), 6 * S(TAU * p)),
              "jaw": (8 * hop, 0, 0)})
    for leg in LEGS:
        q[f"leg_{leg}_upper"] = ((-8 if leg[0] == "F" else 8) * hop, 0, 0)
        q[f"leg_{leg}_lower"] = (10 * hop, 0, 0)
    q.update(tail_sway(t, 10, 2 / 1.2))
    return q


def pose_nuzzle(t):
    """A mother nuzzling her young: the head low and forward, a gentle push, a nibble (2 s, once)."""
    p = t / 2.0
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    push = max(0.0, S(TAU * p * 2))
    q = still(t, 2.0)
    q.update(graze_pose(0.55 * d, L))
    q["head"] = (q["head"][0] - 8 * push, 0, 0)
    q.update(chew(t, 2.0, 4.0 * d))
    return q


def pose_flinch(t):
    """Hit: a jerk aside, the head ducked and turned away, the tail clamped (0.45 s, once). (Turned,
    not rolled: a roll at the chest dips one forefoot into the ground.)"""
    p = t / 0.45
    d = float(np.interp(p, [0, .15, .45, 1], [0, 1, 0.8, 0]))
    q = still(t, 0.45, tail=0.0)
    q.update({"spine_1": (0, 4 * d, 0), "neck": (14 * d, -10 * d, 0), "head": (10 * d, -6 * d, 0), "jaw": (6 * d, 0, 0)})
    for k in range(3):
        q[f"tail_{k}"] = (q[f"tail_{k}"][0] + 6 * d, -4 * d, 0)
    return q


def pose_watch(t):
    """A herd's sentinel: standing tall, the head held high with the chin tucked so the frill stands
    up behind it, turning slowly one way and the other, a chew now and then (4 s, held)."""
    p = t / 4.0
    yaw = 30 * S(TAU * p)
    q = still(t, 4.0, tail=1.0)
    q.update({"neck": (-22, 0.6 * yaw, 0), "head": (12, 0.4 * yaw, 0)})
    q.update(chew(t, 2.0, 2.5 * max(0.0, S(TAU * p * 2))))
    return q


def pose_approach(t):
    """Curious, coming to look: slow careful steps, the neck stretched out low and the beak forward,
    the head cocked one way and the other (1.8 s a cycle, in place of the walk; its height set from
    its feet, `build`)."""
    p = t / 1.8
    q = gait(p, (0.0, 0.5, 0.75, 0.25), 13, 28)
    q.update({"neck": (12 + 2 * S(TAU * p * 2), 0, 0), "head": (-10 - 2 * S(TAU * p * 2), 0, 6 * S(TAU * p)),
              "spine_0": (0, 2 * S(TAU * p), 0)})
    q.update(tail_sway(t, 4, 1 / 1.8))
    return q


def pose_charge(t, P=L):
    """Charging a hunter: the heavy trot driven hard, the head down so the frill stands up in front
    like a shield, the beak forward, the tail out straight (0.55 s a cycle, in place of the walk and
    the run). No lean but `charge_level`, fitted so all four feet come down to the ground (the
    trot's hind feet reach half a unit lower than its forefeet); `build` sets its height from its
    feet, keeping the bounce."""
    p = t / 0.55
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 28, 54)
    for leg in ("BL", "BR"):  # the hind legs kept on their stride under the level
        q[f"leg_{leg}_upper"] = (q[f"leg_{leg}_upper"][0] - P["charge_level"], 0, 0)
    c = S(TAU * p * 2)
    q.update({"root": (0, 1.6 * abs(S(TAU * p)), 0), "spine_0": (P["charge_level"], 0, 0), "neck": (P["charge_neck"], 0, 0),
              "head": (P["charge_head"] + 2 * c, 0, 0)})
    q.update(tail_sway(t, 5, 1 / 0.55))
    for k in range(3):
        q[f"tail_{k}"] = (q[f"tail_{k}"][0] - 4, q[f"tail_{k}"][1], 0)
    return q


def pose_death(t, P=L):
    """Dying: the legs give, it rolls onto its side, the head and tail fall slack, the eyes shut; it
    stays (1.6 s, holds; each frame rests on its lowest point, `build`)."""
    p = t / 1.6
    fall = float(np.interp(p, [0, .35, .7, 1], [0, 0.35, 1, 1]))
    q = {"root": (0, -P["death_drop"] * fall, 0), "spine_0": (0, 0, 84 * fall), "spine_1": (0, 0, 4 * fall),
         "neck": (16 * fall, 0, -12 * fall), "head": (10 * fall, 0, -8 * fall), "jaw": (10 * fall, 0, 0)}
    for leg in LEGS:
        q[f"leg_{leg}_upper"] = ((14 if leg[0] == "F" else -14) * fall, 0, 0)
        q[f"leg_{leg}_lower"] = (12 * fall, 0, 0)
    for k in range(3):
        q[f"tail_{k}"] = (2 * fall, 0, -4 * fall)
    q.update({"eyelid_L": (0, 0, 90 * fall), "eyelid_R": (0, 0, -90 * fall)})
    return q


BLINKS = {"idle": [2.0, 6.4], "lie": [1.5, 4.4], "stare": [1.3], "graze": [1.1], "drink": [0.9], "look": [1.6], "watch": [1.2, 3.1]}
BLINK = 0.12
ANIMS = {"idle": (pose_idle, 8.0), "walk": (pose_walk, 1.2), "run": (pose_run, 0.6), "lie": (pose_lie, 6.0),
         "stare": (pose_stare, 2.0), "graze": (pose_graze, 2.5), "drink": (pose_drink, 1.5), "sniff": (pose_sniff, 2.2),
         "look": (pose_look, 3.0), "scratch": (pose_scratch, 2.0), "stretch": (pose_stretch, 2.4), "sleep": (pose_sleep, 6.0),
         "alarm": (pose_alarm, 1.2), "rear": (pose_rear, 1.4), "play": (pose_play, 1.2), "groom": (pose_groom, 2.5),
         "nuzzle": (pose_nuzzle, 2.0), "flinch": (pose_flinch, 0.45), "death": (pose_death, 1.6),
         "watch": (pose_watch, 4.0), "approach": (pose_approach, 1.8), "charge": (pose_charge, 0.55)}
# Only the held states and the gaits loop (the engine loops a clip whose file says so, whatever the
# game asks); the one-offs play once, death holds its last frame.
LOOPS = {"stare": True, "graze": True, "drink": True, "sleep": True, "watch": True, "approach": True, "charge": True,
         "death": "hold_on_last_frame"}
BLENDS = {"flinch": (0.05, 0.2), "death": (0.1, 0.0), "sleep": (0.6, 0.6), "graze": (0.4, 0.4), "watch": (0.4, 0.4),
          "approach": (0.3, 0.3), "charge": (0.15, 0.3)}
STANDING = ("idle", "walk", "run", "lie")


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}


def frame_times(name, dur):
    n = {"idle": 96, "lie": 60, "walk": 24, "run": 20, "sleep": 36, "flinch": 12, "watch": 40, "approach": 24, "charge": 20}.get(name, 24)
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
                pose.update(eyelids(name, tt))
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames}
        if name not in STANDING:
            out[name].update(loop=loops, blend=BLENDS.get(name, (0.25, 0.3)))
    # A clip goes over what plays below only on the bones it writes, and the exporter leaves out a bone
    # that stays at 0: kept at a hundredth of a degree, so the idle's cropping neck and head, chewing
    # jaw and swaying tail don't show through a clip that holds them straight. The eyelids are left to
    # the idle's blinks.
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
          "body": lambda n: n in ("hips", "chest"), "muzzle": lambda n: n in ("beak", "jaw"),
          "legs": lambda n: n.startswith(("leg_", "thigh_")), "frill": lambda n: n == "frill"}


def fit_poses(spec):
    """Lying (the belly on the ground, the folded legs under it, nothing below) and cropping (the
    beak at the ferns), by forward kinematics."""
    def lie_cost(P):
        g = lowest(spec, pose_lie(0.0, dict(L, **P, lie_drop=0)), GROUPS)
        f = min(g.values())
        # Near-ties (a few thousandths, as when the hind feet narrowed) go to the hips folded deep under (-85),
        # the fold it was drawn with, not to whichever leg angle wins by a hair.
        return (g["body"] - f - 0.8) ** 2 + 0.3 * (g["front paws"] - f) ** 2 + 0.3 * (g["back paws"] - f) ** 2 \
            + 1e-4 * (P["lie_hip"] + 85) ** 2
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

    # The clips' cropping chews no wider than keeps the open jaw out of the ground (the beak is fitted closed).
    def jaw_low(a):
        q = graze_pose(1.0)
        q["head"] = (q["head"][0] + 2, 0, 0)
        q.update(chew(0.125, 2.0, a))
        return lowest(spec, q, GROUPS)["muzzle"]
    L["clip_chew"] = max([a for a in (1, 2, 3, 4, 5, 6) if jaw_low(a) > 0.1], default=0.5)
    # Charging: the level that brings all four feet down to the ground over the stride.
    feet = {f: (lambda n, f=f: n == f) for f in FEET}

    def feet_reach(P):
        lows = [lowest(spec, pose_charge(0.55 * k / 20, P), feet) for k in range(20)]
        m = {f: min(g[f] for g in lows) for f in FEET}
        return max(m.values()) - min(m.values())
    L["charge_level"] = min((round(0.2 * k, 1) for k in range(16)), key=lambda v: feet_reach(dict(L, charge_level=v)))
    print("charging, level", L["charge_level"], "; the feet's lowest within", round(feet_reach(L), 2))
    # Asleep: the head laid on the ground (its lowest a quarter unit over it).
    head = {"head": lambda n: n in HEAD_PARTS or n.startswith("jugal")}

    def sleep_cost(P):
        g = lowest(spec, pose_sleep(0.0, dict(L, **P)), head)
        return (g["head"] - 0.25) ** 2 + 0.002 * ((P["sleep_neck"] - 8) ** 2 + (P["sleep_head"] - 3) ** 2)
    L.update(fit({"sleep_neck": list(range(0, 25, 2)), "sleep_head": list(range(-8, 17, 1))}, sleep_cost)[1])
    # Dying on its side: dropped until nothing is below the ground (each frame of the fall rests on its
    # lowest point: `ground_clips`).
    g = lowest(spec, pose_death(1.6, dict(L, death_drop=0)), {"all": lambda n: True})
    L["death_drop"] = round(float(g["all"]), 2)
    print("cropping: chew", L["clip_chew"], "; asleep, the head", round(float(lowest(spec, pose_sleep(0.0), head)["head"]), 2),
          "over the ground", {k: L[k] for k in ("sleep_neck", "sleep_head")}, "; dead, dropped", L["death_drop"])


HEAD_PARTS = ("skull", "snout", "beak", "nasal", "mouth", "jaw", "frill")


def ground_clips(spec):
    """The clips written as angles, stood on the ground: the gaits' height from their feet frame by
    frame (the approach) or once over the loop (the charge and the play's bounces, kept); rearing on its
    hind feet, frame by frame; dying, each frame of the fall on its lowest point (mid-roll the legs
    swing out below where it ends up lying)."""
    A = spec["animations"]
    A["approach"] = grounded(spec, A["approach"], FEET)
    A["charge"] = grounded(spec, A["charge"], FEET, True)
    A["play"] = grounded(spec, A["play"], FEET, True)
    A["rear"] = grounded(spec, A["rear"], ["leg_BL_paw", "leg_BR_paw"])
    A["death"] = grounded(spec, A["death"], [e["name"] for e in spec["elements"]])


FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]


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
    ground_clips(spec)
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
