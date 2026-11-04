#!/usr/bin/env python3
"""Iguanodont: *Iguanacolossus fortis*, the Yellow Cat floodplain's big plant eater and what the
Utahraptor was trapped hunting at Stikes, made the way Hytale makes creatures (boxes and a
hand-painted texture) with the blocky-model skill, from research alone (README.md). No record of
its colors: chosen as a big floodplain grazer's (see `area`), olive-dun above, pale buff below,
faint darker bands across the back and down the tail.

    ~/.venvs/blocky/bin/python iguanodont.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each), y up, it faces +z. An adult is about 9 m long
and a few tonnes, walking slowly on all fours: here 576 from the beak to the tail's tip, the hips
140 up; the head (long, deep and horse-like, a toothless beak in front of the cheeks) a little
bigger than life.
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
from ground import corners, lowest, world  # noqa: E402
from house import TX  # noqa: E402

# TX: texels a unit, 0.5 (32 a block, the house density, as the blocks and items). The eye's face,
# the skull's side, is 20 by 16 texels at 32 (the house rule wants 3 or more), and the smallest
# parts (the thumb spike, the eyelids) still get 2 or more, so it paints at 32.

OUT = os.environ.get("IGUANO_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

SKIN, BACK, BELLY, RIM, BEAK, MOUTH, CLAW, BAND, DBAND = range(9)
# SKIN the olive-dun hide, BACK the darker olive-grey along the top, BELLY the pale buff underside,
# RIM the faint gaps between its big scales, BEAK dark horn, MOUTH the inside, CLAW the hooves and
# the thumb spike, BAND the faint darker bands across the back and tail where they run down the
# flanks, DBAND the same bands over the darker top. No record of its colors.
BASE = [(150, 138, 100), (104, 100, 76), (208, 196, 158), (140, 128, 94), (104, 90, 74), (112, 58, 54), (84, 72, 60),
        (124, 114, 84), (86, 82, 62)]
SHADOW = [0.03, 0.03, 0.07, 0.03, 0.74, 0.95, 0.74, 0.03, 0.03]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (SKIN, BACK, BELLY, RIM, BAND, DBAND)
ROLES = [("coat", 0, 0), ("coat", 0, -1.2), ("coat", 1, 0), ("coat", 0.3, -0.4), "fixed", "fixed", "fixed",
         ("coat", 0, -0.6), ("coat", 0, -1.6)]

# At 32 a block the skull's side is 20 texels by 16: a 3 by 2 eye (a catchlight up and forward, a
# dark pupil, a brown-gold iris), as the Protoceratops's and Psittacosaurus's: at the eye's real size
# (a few centimetres on a head over a metre long) more would be cartoonish. Drawn for the left side
# (east); the right gets it mirrored, so both catchlights point forward. No record of its eye's
# color: a brown-gold.
_P, _H, _I, _a = (20, 16, 14), (226, 226, 222), (124, 88, 44), (164, 120, 58)
EYES.update({
    "iguano_px3": dict(pattern=["HPI", "IPa"], colors={"P": _P, "H": _H, "I": _I, "a": _a}, halo="none", iris="Ia"),
    "iguano_px": dict(pattern=["HP", "IP"], colors={"P": _P, "H": _H, "I": _I}, halo="none", iris="I"),
})
for _k in [k for k in EYES if k.startswith("iguano_px")]:
    EYES[_k + "_m"] = dict(EYES[_k], pattern=[row[::-1] for row in EYES[_k]["pattern"]])
EYE_STYLE = os.environ.get("IGUANO_EYES", "iguano_px3")

# ------------------------------------------------------------------ the model

# The neck: thick and short, rising forward from inside the chest's front to the head's joint, deep
# inside the skull (`ground.pokes`).
NECK_BASE = np.array([0.0, 128.0, 106.0])
NECK_ANGLE, NECK_LEN = 22.0, 75.0
NECK_HW, NECK_HH = 9.5, 13.0  # half width and height: slimmer than the skull, so its end hides inside


def neck_end():
    a = math.radians(NECK_ANGLE)
    return NECK_BASE + NECK_LEN * np.array([0.0, math.sin(a), math.cos(a)])


HEAD = tuple(float(v) for v in neck_end())  # (0, 156.1, 175.5)
# The head: a long, deep skull; the snout narrowing forward under a ridge (a horse's profile), a
# toothless beak at its front a little downturned; the lower jaw deep, hinged at the back, shorter
# than the upper (the predentary under the beak).
SKULL = ((-12.0, 140.0, 162.0), (12.0, 171.0, 202.0))
EYE_Z, EYE_Y = 190.0, 162.0  # high on the side, toward the back
LID_X = 12.4  # the eyelids' hinge plane, just outside the head's side
NOSTRIL = (155.0, 229.0)  # (y, z) of the nostril slit on the snout's sides, near the beak
JAW = (0.0, 142.0, 170.0)

TAIL0 = np.array([0.0, 138.0, -26.0])
TAIL = [(76, 44.0), (68, 36.0), (62, 28.0), (56, 20.0), (44, 12.0)]  # length, width: long, stiff, held level


def tail_points():
    pts, p = [TAIL0.copy()], TAIL0.copy()
    for length, _ in TAIL:
        p = p + np.array([0, 0, -length])
        pts.append(p.copy())
    return pts


LEGS = ("FL", "FR", "BL", "BR")
FRONT_X, FRONT_Z, BACK_X, BACK_Z = 33.0, 110.0, 24.0, -2.0


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


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 140, 0)), ("spine_1", "spine_0", (0, 130, 30)),
         ("neck", "spine_1", tuple(NECK_BASE)), ("head", "neck", HEAD), ("jaw", "head", JAW)]
    pts = tail_points()
    B += [(f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", tuple(pts[k])) for k in range(len(TAIL))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B.append((f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2)))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x = FRONT_X * s
            B += [(f"leg_{leg}_upper", "spine_1", (x, 112, FRONT_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 68, FRONT_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 21, FRONT_Z + 1))]
        else:
            x = BACK_X * s
            B += [(f"leg_{leg}_upper", "spine_0", (x, 130, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 78, BACK_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 28, BACK_Z - 1))]
    return B


def elements():
    E = [el("hips", "spine_0", (-32, 92, -42), (32, 164, 34)),  # highest over the hips
         el("belly", "spine_0", (-34, 72, -20), (34, 122, 40)),  # the big gut slung between the legs
         el("chest", "spine_1", (-37, 76, 28), (37, 152, 122)),  # deep, the back sloping down to the shoulders
         el("shoulders", "spine_1", (-39.5, 96, 84), (39.5, 147, 124)),  # the shoulder girdle and its muscle
         el("neck", "neck", NECK_BASE + np.array([-NECK_HW, -NECK_HH, -6.0]), NECK_BASE + np.array([NECK_HW, NECK_HH, NECK_LEN + 2.0]),
            rot=(-NECK_ANGLE, 0, 0), origin=tuple(NECK_BASE)),
         # Its root, deeper and wider, ending short of the head: the neck thick where it leaves the chest.
         el("neck_root", "neck", NECK_BASE + np.array([-13.0, -19.0, -4.0]), NECK_BASE + np.array([13.0, 17.0, 52.0]),
            rot=(-NECK_ANGLE, 0, 0), origin=tuple(NECK_BASE)),
         el("skull", "head", *SKULL),
         el("nasal", "head", (-6, 162, 198), (6, 167, 230)),  # the ridge down the face
         el("snout", "head", (-10, 137, 200), (10, 163, 236)),
         el("beak", "head", (-7.5, 134, 234), (7.5, 148, 243)),  # the toothless upper beak
         el("mouth", "head", (-8, 138.5, 168), (8, 141.5, 234)),
         el("jaw", "jaw", (-9, 126, 164), (9, 144, 224)),  # deep, under the cheeks
         el("predentary", "jaw", (-6.5, 130, 222), (6.5, 139, 235))]  # the lower beak, behind the upper's tip
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        lx0, lx1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down"))
    pts = tail_points()
    for k, (length, w) in enumerate(TAIL):
        p = pts[k]
        hh = w * 1.1  # a little taller than wide: the tall spines above and chevrons below
        reach = 0.5 if k == 0 else 4.0  # each reaching back into the one before, so a bend never opens the joint
        E.append(el(f"tail_{k}", f"tail_{k}", (-w / 2, p[1] - hh / 2, p[2] - length), (w / 2, p[1] + hh / 2, p[2] + reach)))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z = FRONT_X * s, FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 11, 64, z - 12), (x + 11, 118, z + 11)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 8.5, 20, z - 9), (x + 8.5, 70, z + 8.5)))
            # The hand: the three middle fingers bound into a hoof, the thumb a spike on the inside.
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 10, 0, z - 10.0), (x + 10, 23, z + 11.5)))
            xi = x - s * 10.0  # the hand's inner side
            ta, tb = sorted((xi + s * 1.0, xi - s * 4.5))
            E.append(el(f"thumb_{leg}", f"leg_{leg}_paw", (ta, 11, z + 2.0), (tb, 19, z + 8.0)))
        else:
            x, z = BACK_X * s, BACK_Z
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (x - 15, 74, z - 21), (x + 15, 140, z + 19)))  # the big drumstick
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 11, 26, z - 14), (x + 11, 80, z + 10)))
            E.append(el(f"meta_{leg}", f"leg_{leg}_paw", (x - 10, 6, z - 12), (x + 10, 30, z + 8.5)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 12.6, 0, z - 13), (x + 12.6, 9, z + 24)))  # three hoofed toes
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


BODY = ("hips", "belly", "chest", "shoulders", "neck", "neck_root")


def pebbles(e, face, p):
    """Big rounded scales with faint gaps on the body, tail and thighs: which texels are the gaps.
    At 32 a texel is two units, so the scales are 12 units (6 texels) or they turn to noise; none on
    the head and lower legs."""
    name = e["name"]
    if not (name in BODY or name.startswith(("tail_", "thigh_"))):
        return np.zeros(p.shape[:-1], bool)
    size = 12.0 if not name.startswith("tail_") else 10.0
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a, b = {"east": (z, y), "west": (z, y), "south": (x, y), "north": (x, y), "up": (x, z), "down": (x, z)}[face]
    row = size * 0.87
    j = np.floor(b / row)
    a2 = a / size + 0.5 * (j % 2)
    fa, fb = a2 - np.floor(a2) - 0.5, b / row - j - 0.5
    return (fa * size) ** 2 + (fb * row) ** 2 > (0.5 * size - 1.0) ** 2


def bands(z):
    """Faint darker bands across the back and down the tail: 12 units (6 texels) wide, every 40."""
    return np.mod(z + 6.0, 40.0) < 12.0


def area(e, face, p, uu, vv, h):
    """Olive-dun above, darker olive-grey along the top, pale buff below with the line between them
    low on the flanks; faint darker bands across the back and tail; the beak dark horn, the hooves
    and the thumb spike darker."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, SKIN)
    side = face in ("east", "west")
    t = vv / max(h, 1)
    if name in BODY or name.startswith("tail_"):
        if face == "up":
            a = np.where(bands(z) & ~np.isin(name, ("neck", "neck_root")), DBAND, BACK)
        elif face == "down":
            a[:] = BELLY
        elif side:
            # The top darker, then the bands down the upper flank, then the pale belly.
            top = (y > 150) if name not in ("belly",) else np.zeros_like(y, bool)
            if name.startswith("tail_"):
                top = t < 0.25
            band = bands(z) & (name not in ("neck", "neck_root", "belly"))
            a = np.where(top, np.where(band, DBAND, BACK), np.where(band & (y > 112) & (t < 0.6), BAND, np.where(y < 100, BELLY, SKIN)))
            if name.startswith("tail_"):
                a = np.where(t > 0.78, BELLY, a)
        elif face in ("south", "north"):
            a = np.where((y > 150) | ((t < 0.2) & (name != "belly")), BACK, np.where(y < 100, BELLY, SKIN))
            if name.startswith("tail_"):
                a = np.where(t > 0.78, BELLY, np.where(t < 0.25, BACK, SKIN))
    elif name == "skull":
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        elif side:
            a = np.where(y > 168, BACK, SKIN)
    elif name == "nasal":
        a[:] = BACK
    elif name == "snout":
        a = np.where(z > 233, BEAK, SKIN)  # the horn starts just behind the beak's box
        if face == "up":
            a = np.where(z > 233, BEAK, BACK)
        elif face == "down":
            a[:] = MOUTH
    elif name == "beak":
        a[:] = BEAK
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else BELLY if face == "down" else np.where(y < 133, BELLY, SKIN)  # pale chin and throat
    elif name == "predentary":
        a[:] = MOUTH if face == "up" else BEAK
    elif name == "mouth":
        a[:] = MOUTH
    elif name.startswith("thumb_"):
        a[:] = CLAW
    elif name.startswith("thigh_"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        else:
            a = np.where(bands(z) & (t < 0.35) & side, BAND, np.where(t > 0.82, BELLY, SKIN))  # the bands running on down
    elif name.startswith(("leg_", "meta_")):
        if name.endswith("_paw"):
            # Hooves: the front's lower part and the toes' tips dark horn.
            a = np.where(((face == "south") & (y < 6)) | (z > e["hi"][2] - 3.0), CLAW, SKIN)
            if face == "down":
                a[:] = CLAW
        elif side and ((e["lo"][0] > 0) == (face == "west")):
            a[:] = BELLY  # the inside of the legs
    gap = pebbles(e, face, p) & np.isin(a, (SKIN, BAND)) & (face != "down")
    return np.where(gap, RIM, a)


LASH = (40, 34, 26)

# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def layer_props(e, face, p, a):
    """White where this one is pale (the belly); its marks where its back and flanks are."""
    n, y = e["name"], p[..., 1]
    white = np.where(a == BELLY, 0.85, 0.15)
    ext = np.where(np.isin(a, (BEAK, CLAW)), 1.0, 0.0)
    body = n in BODY or n.startswith(("tail_", "thigh_"))
    pattern = np.full(y.shape, 1.0 if body else 0.3)
    return {"white": white, "extremity": ext, "pattern": pattern}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    if w == 0 or h == 0:
        return None
    size = np.array(e["hi"]) - np.array(e["lo"])
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel centers, in texels
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
    p = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    light_face = face
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "meta_", "tail_", "jaw", "predentary", "thumb_")):
        light_face = "north"  # seen when a pose lifts or folds it
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=False, merge=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["name"] == "skull" and face in ("east", "west"):
        side = 1 if face == "east" else -1
        stamp_eyes(img, [eye_center(side)], eye_style(side))
    if e["name"] == "snout" and face in ("east", "west"):  # the nostril: a slit, a texel high, two long
        lo, hi = e["lo"], e["hi"]
        r = min(h - 1, int((hi[1] - NOSTRIL[0]) / sv))
        c = min(w - 2, int(((hi[2] - NOSTRIL[1]) if face == "east" else (NOSTRIL[1] - lo[2])) / su))
        c = c if face == "west" else max(0, c - 1)
        img[r, c:c + 2, :3] = (52, 44, 36)
    if e["name"].startswith("eyelid"):  # a closed eye: the lash along the edge that ends up at the bottom
        col_idx = 0 if e["name"].endswith("L") else w - 1
        k = 1.0 if w >= 3 else 0.5
        img[:, col_idx, :3] = np.round(img[:, col_idx, :3] * (1 - k) + np.array(LASH) * k).astype(np.uint8)
    if e["name"].endswith("_paw") and face == "south" and w >= 5:
        # Hoofed toes (three on a foot, the hand's three middle fingers): grooves down the front.
        for c in (w // 3, (2 * w) // 3):
            img[:, c, :3] = (img[:, c, :3] * 0.74).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"lie_front": 80, "lie_elbow": -150, "lie_hip": -70, "lie_knee": 140, "lie_ankle": -70, "lie_wrist": 0, "lie_drop": 60.0,
     "tail_0": -10, "tail_1": 4, "tail_2": 4, "graze_neck": 60, "graze_head": 20}


def gait(p, phases, swing_amp, lift):
    """A foot swings forward while it's lifted, back while it's down; the knees bend forward, the
    feet kept about flat."""
    out = {}
    for leg, ph in zip(LEGS, phases):
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))
        swing_ = -swing_amp * S(a)
        out[f"leg_{leg}_upper"] = (swing_, 0, 0)
        knee = lift * up
        if leg[0] == "F":
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (knee, 0, 0), (-0.6 * knee, 0, 0)
        else:
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (knee, 0, 0), (-0.8 * knee, 0, 0)
    return out


def chew(t, rate=1.6, amp=5.0):
    return {"jaw": (amp * max(0.0, S(TAU * t * rate)), 0, 0)}


def graze_pose(g, P=L):
    """The neck and head down to the low plants by `g` (0..1); the legs stay where they stand."""
    return {"neck": (P["graze_neck"] * g, 0, 0), "head": (P["graze_head"] * g, 0, 0)}


def tail_sway(p, n, amp, lift=0.0):
    """The tail swung from its base, stiff (its ossified tendons): the segments after the first barely
    bend on their own."""
    q = {}
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = swing((0, 0, -1), amp * (1.0 if k == 0 else 0.25) * S(TAU * p * n - 0.3 * k),
                               (lift if k == 0 else 0, 0, 0))
    return q


def pose_idle(t):
    """Cropping low plants and chewing, then the head up to look about, chewing slowly, the tail
    swaying a little (10 s)."""
    p = t / 10.0
    br = S(TAU * p * 3)
    g = float(np.interp(p, [0, .45, .55, .85, .95, 1], [1, 1, 0, 0, 1, 1]))
    look = (1 - g) * 18 * S(TAU * (p - 0.55) * 1.8)
    q = graze_pose(g)
    q.update({"root": (0, 0.4 * br, 0), "spine_1": (0.5 * br, 0, 0), "head": (q["head"][0], look, 0)})
    q.update(tail_sway(p, 2, 2.5))
    q.update(chew(t, 1.6 if g > 0.5 else 0.7, 5 if g > 0.5 else 2))
    return q


def pose_walk(t):
    """On all fours, a slow lateral walk (the feet in turn: back left, front left, back right, front
    right), the head nodding with the steps (1.6 s)."""
    p = t / 1.6
    q = gait(p, (0.0, 0.5, 0.75, 0.25), 16, 30)
    q.update({"root": (0, 1.2 * abs(S(TAU * p * 2)), 0), "spine_0": (0, 1.5 * S(TAU * p), 0),
              "neck": (1.5 * S(TAU * p * 2), 0, 0), "head": (-1.5 * S(TAU * p * 2), -2 * S(TAU * p), 0)})
    q.update(tail_sway(p, 1, 3))
    return q


def pose_run(t):
    """A heavy trot on all fours, the head held low and forward, the tail straight out (1.0 s)."""
    p = t / 1.0
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 24, 46)
    q.update({"root": (0, 3.0 * abs(S(TAU * p)), 0), "spine_1": (1.5 * S(TAU * p * 2), 0, 0), "neck": (8, 0, 0),
              "head": (-6 + 2 * S(TAU * p * 2), 0, 0)})
    q.update(tail_sway(p, 1, 2, lift=1.0))
    return q


def lie_body(P=L):
    """The legs folded under, the neck lowered: the pose before its drop to the ground."""
    q = {"neck": (14, 0, 0), "head": (2, 0, 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (P["lie_front"], 0, 0)
        q[f"leg_{leg}_lower"] = (P["lie_elbow"], 0, 0)
        q[f"leg_{leg}_paw"] = (P["lie_wrist"] - P["lie_front"] - P["lie_elbow"], 0, 0)  # the hand's own angle, folded under
    for leg, s in (("BL", 1), ("BR", -1)):
        q[f"leg_{leg}_upper"] = (P["lie_hip"], 0, s * 6)
        q[f"leg_{leg}_lower"] = (P["lie_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (P["lie_ankle"], 0, 0)  # the foot along the ground, as a bird's
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = (P.get(f"tail_{min(k, 2)}", 0) if k < 3 else P["tail_2"] * 0.5, 0, 0)
    return q


def pose_lie(t, P=L):
    """Lying on its belly, the legs folded under, the head held low, the tail along the ground (8 s)."""
    p = t / 8.0
    br = S(TAU * p * 2)
    q = lie_body(P)
    q.update({"root": (0, -P["lie_drop"] + 0.15 * (1 + br), 0), "spine_1": (0.6 * br, 0, 0), "head": (2, 8 * S(TAU * p), 0)})
    for k in range(1, len(TAIL)):
        rx = q[f"tail_{k}"][0]
        q[f"tail_{k}"] = swing((0, 0, -1), 2.0 * S(TAU * p - 0.6 * k), (rx, 0, 0))
    q.update(chew(t, 0.5, 2))
    return q


BLINKS = {"idle": [3.0, 7.4], "lie": [1.8, 5.6]}
BLINK = 0.12
ANIMS = {"idle": (pose_idle, 10.0), "walk": (pose_walk, 1.6), "run": (pose_run, 1.0), "lie": (pose_lie, 8.0)}


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}


def frame_times(name, dur):
    n = {"idle": 100, "lie": 64, "walk": 24, "run": 20}.get(name, 16)
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


def extent(spec, pose, test):
    """(lowest, highest) world corner of the elements `test(name)` accepts, in a pose."""
    W = world(spec["bones"], pose)
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}
    pts = []
    for e in spec["elements"]:
        if test(e["name"]):
            R, t = W[e["bone"]]
            pts.append((corners(e) - piv[e["bone"]]) @ R.T + t)
    pts = np.concatenate(pts)
    return pts.min(0), pts.max(0)

GROUPS = {"front paws": lambda n: n in ("leg_FL_paw", "leg_FR_paw", "thumb_FL", "thumb_FR"),
          "back paws": lambda n: n in ("leg_BL_paw", "leg_BR_paw", "meta_BL", "meta_BR"),
          "body": lambda n: n in ("hips", "belly", "chest", "shoulders"),
          "muzzle": lambda n: n in ("beak", "jaw", "predentary", "snout"),
          "legs": lambda n: n.startswith(("leg_", "thigh_", "meta_", "thumb_")),
          "tail": lambda n: n.startswith("tail_")}
GROUPS.update({f"tail_{k}": (lambda n, k=k: n == f"tail_{k}") for k in range(5)})


def fit_poses(spec):
    """Lying (the belly on the ground, the folded legs under it, the tail along the ground, nothing
    below) and cropping (the beak at the plants), by forward kinematics."""
    top = max(e["hi"][1] for e in spec["elements"] if e["name"] in ("hips", "chest"))

    front = lambda n: n.startswith(("leg_FL", "leg_FR", "thumb_"))  # noqa: E731
    back = lambda n: n.startswith(("leg_BL", "leg_BR", "thigh_", "meta_"))  # noqa: E731
    paw = {front: GROUPS["front paws"], back: GROUPS["back paws"]}

    def lie_cost(P, side):
        # The legs folded under the belly with their feet on the ground beside it; no folded leg
        # standing up over the back.
        pose = lie_body(dict(L, **P))
        g = lowest(spec, dict(pose), {"body": GROUPS["body"], "leg": side, "paw": paw[side]})
        f = g["body"] - 0.8
        lo, hi = extent(spec, pose, side)
        # Kept under and beside the body: not over the back, not out in front of the chest or
        # behind the hips.
        out = max(0.0, hi[1] - (top - 30)) + max(0.0, hi[2] - 126) + max(0.0, -46 - lo[2])
        return (g["leg"] - f) ** 2 + 1.0 * (g["paw"] - f) ** 2 + 2 * out ** 2
    best = fit({"lie_front": list(range(-60, 101, 10)), "lie_elbow": list(range(-160, 41, 10)),
                "lie_wrist": list(range(-100, 101, 25))}, lambda P: lie_cost(P, front))[1]
    best = fit({k: [v - 5, v, v + 5] for k, v in best.items()}, lambda P: lie_cost(P, front))[1]  # then finer
    L.update(best)
    best = fit({"lie_hip": list(range(-90, -9, 10)), "lie_knee": list(range(100, 171, 10)), "lie_ankle": list(range(-170, -39, 10))},
               lambda P: lie_cost(P, back))[1]
    best = fit({k: [v - 5, v, v + 5] for k, v in best.items()}, lambda P: lie_cost(P, back))[1]
    L.update(best)
    g = lowest(spec, lie_body(L), GROUPS)
    L["lie_drop"] = round(float(min(g["body"], g["legs"])), 2)

    def tail_cost(P):
        g = lowest(spec, lie_body(dict(L, **P)), GROUPS)
        f = L["lie_drop"]
        # The back half along the ground, the root sloping down to it from the hips; nothing below.
        return sum((g[f"tail_{k}"] - f - 0.5) ** 2 for k in (2, 3, 4)) + 0.2 * (g["tail_1"] - f - 3.0) ** 2 \
            + 2000 * min(0.0, g["tail"] - f) ** 2 + 0.002 * (P["tail_0"] ** 2 + P["tail_1"] ** 2 + P["tail_2"] ** 2)
    best = fit({"tail_0": list(range(-44, 1, 2)), "tail_1": list(range(-20, 31, 2)), "tail_2": list(range(-10, 31, 2))}, tail_cost)[1]
    L.update(best)
    g = lowest(spec, lie_body(L), GROUPS)
    assert g["muzzle"] >= L["lie_drop"] - 0.05, f"the head sinks into the ground lying: {g}"

    def graze_cost(P):
        g = lowest(spec, graze_pose(1.0, dict(L, **P)), GROUPS)
        return (g["muzzle"] - 0.5) ** 2 + 0.0005 * (P["graze_neck"] ** 2 + P["graze_head"] ** 2)
    # The neck no further down than 70: past that its root's lower corner sweeps through the
    # chest's front (`ground.pokes`); the head bends down the rest, as a horse's does.
    best = fit({"graze_neck": list(range(20, 71, 2)), "graze_head": list(range(-20, 111, 3))}, graze_cost)[1]
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
            img = paint_face(e, f, boxes)
            if img is not None:
                tiles.append(((i, f), img))
    sheet, uv = pack(tiles, 512)  # the layers pack at the same width (`Recorder.save`)
    LAYERS["rec"].save(OUT, [k for k, _ in tiles], pack)
    LAYERS["rec"] = None
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "iguanodont", "texture": [sheet.shape[1], sheet.shape[0]],
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
