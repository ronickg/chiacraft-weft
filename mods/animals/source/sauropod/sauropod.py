#!/usr/bin/env python3
"""Sauropod: *Cedarosaurus weiskopfae*, the Yellow Cat floodplain's brachiosaurid (it also stands
for *Dongbeititan* in the fern forest), made the way Hytale makes creatures (boxes and a
hand-painted texture) with the blocky-model skill, from research alone (README.md). No record of
its colors: chosen as a big browser's (see `area`), a grey-brown to olive hide, darker along the
top, pale below, faint darker bands across the back and in rings down the neck.

    ~/.venvs/blocky/bin/python sauropod.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each), y up, it faces +z. An adult is about 15 m long
along the spine and about 10 t: here about 990 units from the snout to the tail's tip measured
along the neck, body and tail, the shoulders 252 up and the hips 214 (the forelimbs longer, the back
sloping down from the shoulders), the neck raised high (the head's joint about 8.3 m up), the tail
shorter than the neck and held off the ground; the small head a little bigger than life.
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
# the skull's side, is 16 by 17 texels at 32 (the house rule wants 3 or more), and the smallest
# parts (the eyelids, the nostril slits) still get 2 or more, so it paints at 32: a 15 m animal
# needs no finer texels.

OUT = os.environ.get("SAURO_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

SKIN, BACK, BELLY, BAND, DBAND, MOUTH, CLAW = range(7)
# SKIN the grey-brown to olive hide, BACK the darker olive-grey along the top, BELLY the pale
# underside and throat, BAND the faint darker bands across the back and in rings down the neck where
# they run down the sides, DBAND the same bands over the darker top, MOUTH the inside, CLAW the
# horny soles, the thumb claw and the toe claws. No record of its colors.
BASE = [(118, 112, 86), (88, 86, 68), (172, 162, 132), (109, 103, 79), (79, 77, 61), (112, 58, 54), (72, 62, 52)]
SHADOW = [0.03, 0.03, 0.04, 0.03, 0.03, 0.95, 0.74]  # the pale belly's shadows brown, not pink
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (SKIN, BACK, BELLY, BAND, DBAND)
ROLES = [("coat", 0, 0), ("coat", 0, -1.2), ("coat", 1, 0), ("coat", 0, -0.6), ("coat", 0, -1.6), "fixed", "fixed"]

# At 32 a block the skull's side is 16 texels by 17: a 3 by 2 eye (a catchlight up and forward, a
# dark pupil, a dark amber iris), as the iguanodont's and the Protoceratops's: at the eye's real size
# (a few centimetres on a head under a metre long) more would be cartoonish. Drawn for the left side
# (east); the right gets it mirrored, so both catchlights point forward. No record of its eye's
# color: a dark amber.
_P, _H, _I, _a = (20, 16, 14), (226, 226, 222), (112, 78, 40), (152, 110, 52)
EYES.update({
    "sauro_px3": dict(pattern=["HPI", "IPa"], colors={"P": _P, "H": _H, "I": _I, "a": _a}, halo="none", iris="Ia"),
    "sauro_px": dict(pattern=["HP", "IP"], colors={"P": _P, "H": _H, "I": _I}, halo="none", iris="I"),
})
for _k in [k for k in EYES if k.startswith("sauro_px")]:
    EYES[_k + "_m"] = dict(EYES[_k], pattern=[row[::-1] for row in EYES[_k]["pattern"]])
EYE_STYLE = os.environ.get("SAURO_EYES", "sauro_px3")

# ------------------------------------------------------------------ the model

# Chains (the neck, the tail): segments of (length, angle up from the way they point, half width,
# half height), each a box laid from its joint along the chain and turned to its angle, reaching back
# into the one before (`reach`) and on past its own end (`ext`) so every joint is inside both parts.
# Each is slimmer than the one before by more than its reach can swing out (`ground.pokes`).
NECK_BASE = np.array([0.0, 222.0, 198.0])  # deep inside the chest's top front
NECK = [(100, 48.0, 27.0, 34.0), (95, 56.0, 21.0, 27.0), (90, 60.0, 16.0, 21.0), (80, 60.0, 11.0, 15.0)]
NECK_REACH, NECK_EXT = (10.0, 12.0, 12.0, 10.0), 3.0
TAIL0 = np.array([0.0, 182.0, -40.0])
TAIL = [(70, 6.0, 30.0, 25.0), (64, 9.0, 23.5, 20.0), (56, 11.0, 17.5, 15.5), (48, 12.0, 12.5, 11.0), (40, 12.0, 8.0, 7.5)]
TAIL_REACH, TAIL_EXT = (10.0, 8.0, 8.0, 8.0, 6.0), 4.0


def chain_points(base, segs, sign):
    """The joints of a chain from `base`: sign +1 runs forward (+z) and up by each angle, -1 runs
    back (-z) and down by it."""
    pts, p = [np.array(base, float)], np.array(base, float)
    for length, ang, _, _ in segs:
        a = math.radians(ang)
        p = p + length * np.array([0.0, sign * math.sin(a), sign * math.cos(a)])
        pts.append(p.copy())
    return pts


def neck_points():
    return chain_points(NECK_BASE, NECK, 1)


def tail_points():
    return chain_points(TAIL0, TAIL, -1)


HEAD = tuple(float(v) for v in neck_points()[-1])  # about (0, 522, 403): 8.2 m up
HX, HY, HZ = HEAD


def hd(lo, hi):
    """A head box given relative to the head's joint."""
    return (lo[0], HY + lo[1], HZ + lo[2]), (hi[0], HY + hi[1], HZ + hi[2])


# The head: small and tall; the domed nasal arch rising over and in front of the eyes, the nostrils
# high on it; a long, blunt snout sloping down in front; the lower jaw hinged at the back, shorter
# than the upper (an overbite). The skull holds a circle of 16 around the neck's joint, so the
# neck's end stays inside it however the head turns (`ground.pokes`).
SKULL = hd((-14.0, -18.0, -16.0), (14.0, 16.0, 16.0))
DOME = hd((-10.0, 2.0, 0.0), (10.0, 28.0, 23.0))
SNOUT = hd((-12.75, -13.0, 14.0), (12.75, 4.5, 44.0))
JAW_BOX = hd((-9.5, -21.0, -10.0), (9.5, -11.0, 40.0))
MOUTH_BOX = hd((-8.0, -15.5, -4.0), (8.0, -12.5, 38.0))
JAW = (0.0, HY - 15.0, HZ - 8.0)
EYE_Z, EYE_Y = HZ + 5.0, HY + 8.0  # high on the side, behind the dome
LID_X = 14.4  # the eyelids' hinge plane, just outside the head's side
NOSTRILS = (HZ + 17.0, 4.5)  # (z, x) of the nostril slits on the dome's top, near its front

LEGS = ("FL", "FR", "BL", "BR")
FRONT_X, FRONT_Z, BACK_X, BACK_Z = 50.0, 180.0, 44.0, 0.0
FRONT_TOP, BACK_TOP = 210.0, 168.0  # the shoulder and hip joints: the forelimbs longer


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
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 190, 0)), ("spine_1", "spine_0", (0, 192, 40))]
    npts = neck_points()
    B += [(f"neck_{k}", "spine_1" if k == 0 else f"neck_{k - 1}", tuple(npts[k])) for k in range(len(NECK))]
    B += [("head", f"neck_{len(NECK) - 1}", HEAD), ("jaw", "head", JAW)]
    tpts = tail_points()
    B += [(f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", tuple(tpts[k])) for k in range(len(TAIL))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B.append((f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2)))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x = FRONT_X * s
            B += [(f"leg_{leg}_upper", "spine_1", (x, FRONT_TOP, FRONT_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 116, FRONT_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 28, FRONT_Z))]
        else:
            x = BACK_X * s
            B += [(f"leg_{leg}_upper", "spine_0", (x, BACK_TOP, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 98, BACK_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 28, BACK_Z + 2))]
    return B


def chain_elements(prefix, base, segs, sign, reach, ext):
    out = []
    pts = chain_points(base, segs, sign)
    for k, (length, ang, hw, hh) in enumerate(segs):
        p = pts[k]
        if sign > 0:
            lo, hi = p + np.array([-hw, -hh, -reach[k]]), p + np.array([hw, hh, length + ext])
        else:
            lo, hi = p + np.array([-hw, -hh, -length - ext]), p + np.array([hw, hh, reach[k]])
        out.append(el(f"{prefix}_{k}", f"{prefix}_{k}", lo, hi, rot=(-ang, 0, 0), origin=tuple(p)))
    return out


def elements():
    E = [el("hips", "spine_0", (-56, 138, -48), (56, 214, 46)),
         # The barrel: the gut slung low between the legs, the deep ribcage in front, the back a
         # slab sloping up from the hips to the shoulders, and the shoulder girdle's muscle widening
         # the top in front.
         el("belly", "spine_1", (-62, 104, 20), (62, 204, 160)),
         el("chest", "spine_1", (-65.5, 138, 138), (65.5, 258, 234)),
         el("back", "spine_1", (-52, 176, 4), (52, 222, 212), rot=(-14, 0, 0), origin=(0, 200, 40)),
         el("shoulders", "spine_1", (-70, 204, 166), (70, 262.5, 218))]
    E += chain_elements("neck", NECK_BASE, NECK, 1, NECK_REACH, NECK_EXT)
    E += [el("skull", "head", *SKULL),
          el("dome", "head", *DOME),  # the nasal arch
          el("snout", "head", *SNOUT),
          el("mouth", "head", *MOUTH_BOX),
          el("jaw", "jaw", *JAW_BOX)]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        lx0, lx1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down"))
    E += chain_elements("tail", TAIL0, TAIL, -1, TAIL_REACH, TAIL_EXT)
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            # Pillars: the long upper arm, the forearm, the hand a round column (the fingers bound
            # into a horseshoe), each wider than the one above where they meet.
            x, z = FRONT_X * s, FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 17, 110, z - 18), (x + 17, 218, z + 17)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 14, 24, z - 14.5), (x + 14, 120, z + 14)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 16, 0, z - 15.5), (x + 16, 30, z + 16.5)))
        else:
            x, z = BACK_X * s, BACK_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 21, 92, z - 24), (x + 21, 184, z + 22)))  # the thigh
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 16.5, 26, z - 18), (x + 16.5, 102, z + 17)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 19, 0, z - 19), (x + 19, 30, z + 25)))  # the foot, toes forward
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


BODY = ("hips", "belly", "chest", "back", "shoulders")
BAND_EVERY, BAND_W = 44.0, 14.0  # 7 texels wide, so they read as bands, not lines


def bands(s):
    return np.mod(s + 6.0, BAND_EVERY) < BAND_W


def neck_s(e, pl):
    """Distance along the neck from its base, from a neck box's unturned texel positions."""
    k = int(e["name"].split("_")[1])
    before = sum(seg[0] for seg in NECK[:k])
    return before + (pl[..., 2] - neck_points()[k][2])


def area(e, face, p, pl, uu, vv, h):
    """Grey-brown to olive above, a darker olive-grey along the top, pale below with the line
    between them low on the flanks; faint darker bands across the back and in rings down the neck;
    the soles, the thumb claw and the toe claws dark horn."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, SKIN)
    side = face in ("east", "west")
    t = vv / max(h, 1)
    if name in BODY:
        band = bands(z) & (z < 214)
        if face == "up":
            a = np.where(band, DBAND, BACK)
        elif face == "down":
            a[:] = BELLY
        elif side:
            a = np.where(y > 214 + (z - 40) * 0.24, np.where(band, DBAND, BACK),
                         np.where(band & (y > 168), BAND, np.where(y < 142, BELLY, SKIN)))
        else:
            a = np.where(y > 240, BACK, np.where(y < 142, BELLY, SKIN))
    elif name.startswith("neck_"):
        band = bands(neck_s(e, pl))
        if face == "up":
            a = np.where(band, DBAND, BACK)
        elif face == "down":
            hw = (e["hi"][0] - e["lo"][0]) / 2
            a = np.where(np.abs(x) < 0.55 * hw, BELLY, SKIN)  # a pale stripe down the throat
        elif side:
            a = np.where(t < 0.3, np.where(band, DBAND, BACK), np.where(t > 0.82, BELLY, np.where(band & (t < 0.6), BAND, SKIN)))
        else:
            a[:] = SKIN
    elif name.startswith("tail_"):
        band = bands(z) & (z > -260)
        if face == "up":
            a = np.where(band, DBAND, BACK)
        elif face == "down":
            a[:] = BELLY
        elif side:
            a = np.where(t < 0.28, np.where(band, DBAND, BACK), np.where(t > 0.75, BELLY, np.where(band & (t < 0.55), BAND, SKIN)))
        else:
            a = np.where(t > 0.75, BELLY, np.where(t < 0.28, BACK, SKIN))
    elif name in ("skull", "dome"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        elif side:
            a = np.where(y > HY + 12, BACK, SKIN)
    elif name == "snout":
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a = np.where(z > JAW_BOX[1][2] - 1.0, SKIN, MOUTH)  # the roof of the mouth, the lip's edge in front of the jaw
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else BELLY if face == "down" else np.where(y < HY - 16, BELLY, SKIN)  # pale chin
    elif name == "mouth":
        a[:] = MOUTH
    elif name.startswith("leg_"):
        lo, hi = e["lo"], e["hi"]
        if name.endswith("_paw"):
            if face == "down":
                a[:] = CLAW
            else:
                a = np.where(y < 3.0, CLAW, SKIN)  # the horny sole's rim
                if name[4] == "B" and face == "south":
                    a = np.where(y < 12.0, CLAW, a)  # the three big toe claws (split below)
                if name[4] == "F" and face == "south":
                    inner = (x - (lo[0] + hi[0]) / 2) * (1 if lo[0] < 0 else -1) > 6.0
                    a = np.where(inner & (y < 14.0) & (y > 2.0), CLAW, a)  # the thumb claw, on the inside
        elif face == "up":
            a[:] = BACK
        elif side and ((lo[0] > 0) == (face == "west")):
            a = np.where(t > 0.5, BELLY, SKIN) if name.endswith("_upper") else a  # the inside of the upper legs
    return a


LASH = (40, 34, 26)

# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def layer_props(e, face, p, a):
    """White where this one is pale (the belly and throat); its marks where its back, flanks, neck
    and tail are."""
    n, y = e["name"], p[..., 1]
    white = np.where(a == BELLY, 0.85, 0.15)
    ext = np.where(a == CLAW, 1.0, 0.0)
    body = n in BODY or n.startswith(("tail_", "neck_")) or n.endswith("_upper")
    pattern = np.full(y.shape, 1.0 if body else 0.3)
    return {"white": white, "extremity": ext, "pattern": pattern}


MOVING = ("leg_", "tail_", "jaw")


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    if w == 0 or h == 0:
        return None
    size = np.array(e["hi"]) - np.array(e["lo"])
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel centers, in texels
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
    pl = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv  # unturned
    R = rotm(e["rot"])
    p = pl
    if e["rot"]:
        o = np.array(e["origin"])
        p = (pl - o) @ R.T + o
    light_face = face
    if face == "down" and e["name"].startswith(MOVING):
        light_face = "north"  # seen when a pose lifts or folds it
    if face == "down" and e["rot"] and (R @ n)[2] > 0.5:
        light_face = "south"  # a raised neck's underside faces forward
    a = area(e, face, p, pl, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=False, merge=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["name"] == "skull" and face in ("east", "west"):
        side = 1 if face == "east" else -1
        stamp_eyes(img, [eye_center(side)], eye_style(side))
    if e["name"] == "dome" and face == "up":
        # The nostrils high on the arch: two slits along the snout, a texel wide and three long.
        lo = e["lo"]
        r = int((NOSTRILS[0] - lo[2]) / sv)
        for sx in (-1, 1):
            c = min(w - 1, max(0, int((sx * NOSTRILS[1] - lo[0]) / su)))
            img[max(0, r - 1):r + 2, c, :3] = (56, 48, 40)
    if e["name"].startswith("eyelid"):  # a closed eye: the lash along the edge that ends up at the bottom
        col_idx = 0 if e["name"].endswith("L") else w - 1
        k = 1.0 if w >= 3 else 0.5
        img[:, col_idx, :3] = np.round(img[:, col_idx, :3] * (1 - k) + np.array(LASH) * k).astype(np.uint8)
    if e["name"].startswith("leg_B") and e["name"].endswith("_paw") and face == "south" and w >= 5:
        # Three big toe claws: two grooves splitting the dark front, up into the hide a little.
        for c in (w // 3, (2 * w) // 3):
            img[:, c, :3] = (img[:, c, :3] * 0.78).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"lie_front": 80, "lie_elbow": -150, "lie_hip": -70, "lie_knee": 140, "lie_ankle": -70, "lie_wrist": 0, "lie_drop": 100.0,
     "tail_0": -10, "tail_1": 4, "tail_2": 4, "drink_neck0": 30, "drink_neck": 30, "drink_head": 40}
NECK_DIRS = [(0.0, math.sin(math.radians(a)), math.cos(math.radians(a))) for _, a, _, _ in NECK]
WALK, RUN = 2.4, 2.0
LEG_LEN = {"F": FRONT_TOP, "B": BACK_TOP}  # the joint's height: the swing's radius


def stride_amp(dur, leg):
    """The swing (degrees) that moves a planted foot back at the ground's speed: Weft plays a walk
    0.6 clip seconds a block, so a planted foot goes back 64 / 0.6 units a clip second, for half the
    cycle."""
    back = 64.0 / 0.6 * dur / 2
    return math.degrees(back / (2 * LEG_LEN[leg[0]]))


def gait(p, dur, phases, lift):
    """A foot swings forward while it's lifted, back while it's down; the knees bend forward, the
    feet kept about flat."""
    out = {}
    for leg, ph in zip(LEGS, phases):
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))
        out[f"leg_{leg}_upper"] = (-stride_amp(dur, leg) * S(a), 0, 0)
        knee = lift[leg[0]] * up
        out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (knee, 0, 0), (-0.7 * knee, 0, 0)
    return out


def chew(t, rate=1.4, amp=5.0):
    return {"jaw": (amp * max(0.0, S(TAU * t * rate)), 0, 0)}


def neck_pose(pitch, side=0.0, head=0.0, head_yaw=0.0, first=None):
    """The neck bent by `pitch` at each joint (`first` at the base, if given; + lowers it) and swung
    `side` degrees across, shared between its joints; the head pitched and turned on top."""
    q = {}
    for k in range(len(NECK)):
        pk = first if (k == 0 and first is not None) else pitch
        q[f"neck_{k}"] = swing(NECK_DIRS[k], side / len(NECK), (pk, 0, 0))
    q["head"] = (head, head_yaw, 0)
    return q


def drink_pose(g, P=L):
    """The neck and head down to the water by `g` (0..1); the legs stay where they stand."""
    return neck_pose(P["drink_neck"] * g, 0, P["drink_head"] * g, first=P["drink_neck0"] * g)


def tail_sway(p, n, amp, lift=0.0):
    """The tail swung from its base, each segment a little more."""
    q = {}
    for k in range(len(TAIL)):
        a = TAIL[k][1]
        d = (0.0, -math.sin(math.radians(a)), -math.cos(math.radians(a)))
        q[f"tail_{k}"] = swing(d, amp * (1.0 if k == 0 else 0.5) * S(TAU * p * n - 0.4 * k), (lift if k == 0 else 0, 0, 0))
    return q


IDLE = 16.0


def pose_idle(t):
    """Browsing high: the neck up and reaching, swung from side to side across the trees, the head
    tipped up, chewing; then down to drink, a slow gulp or two, and up again (16 s)."""
    p = t / IDLE
    br = S(TAU * p * 4)
    g = float(np.interp(t, [0, 6.0, 7.6, 10.4, 12.0, IDLE], [0, 0, 1, 1, 0, 0]))
    g = g * g * (3 - 2 * g)  # eased
    reach = (1 - g) * (0.5 + 0.5 * S(TAU * t / 5.0))
    side = (1 - g) * 16 * S(TAU * t / 8.0)
    q = neck_pose(-4.0 * reach + L["drink_neck"] * g, side, -14.0 * reach + L["drink_head"] * g,
                  head_yaw=(1 - g) * 6 * S(TAU * t / 8.0 + 0.8), first=-4.0 * reach + L["drink_neck0"] * g)
    q.update({"root": (0, 0.4 * br, 0), "spine_1": (0.4 * br, 0, 0)})
    q.update(tail_sway(p, 2, 2.0))
    if g < 0.5:
        q.update(chew(t, 1.2, 4.0 * (1 - 2 * g)))
    else:
        q.update({"jaw": (3.0 * max(0.0, S(TAU * t * 0.8)), 0, 0)})  # gulping
    return q


def pose_walk(t):
    """On all fours, a slow lateral walk (back left, front left, back right, front right), the neck
    nodding a little with the steps (2.4 s)."""
    p = t / WALK
    q = gait(p, WALK, (0.0, 0.5, 0.75, 0.25), {"F": 26, "B": 30})
    q.update(neck_pose(1.0 * S(TAU * p * 2), 3 * S(TAU * p), -1.0 * S(TAU * p * 2)))
    q.update({"root": (0, 1.5 * abs(S(TAU * p * 2)), 0), "spine_0": (0, 1.2 * S(TAU * p), 0)})
    q.update(tail_sway(p, 1, 3))
    return q


def pose_run(t):
    """A faster amble, still a walk's footfalls: higher steps, the body rolling, the neck a little
    lower and forward (2.0 s)."""
    p = t / RUN
    q = gait(p, RUN, (0.0, 0.5, 0.75, 0.25), {"F": 40, "B": 44})
    q.update(neck_pose(4.0 + 1.5 * S(TAU * p * 2), 3 * S(TAU * p), -14.0 - 1.5 * S(TAU * p * 2), first=4.0))
    q.update({"root": (0, 3.0 * abs(S(TAU * p * 2)), 0), "spine_0": (0, 2.0 * S(TAU * p), 1.0 * S(TAU * p)),
              "spine_1": (0.8 * S(TAU * p * 2), 0, 0)})
    q.update(tail_sway(p, 1, 4, lift=-2.0))
    return q


def lie_body(P=L):
    """The legs folded under, the neck lowered to a rest: the pose before its drop to the ground."""
    q = neck_pose(LIE_NECK, 0, LIE_HEAD, first=LIE_NECK0)
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (P["lie_front"], 0, 0)
        q[f"leg_{leg}_lower"] = (P["lie_elbow"], 0, 0)
        q[f"leg_{leg}_paw"] = (P["lie_wrist"] - P["lie_front"] - P["lie_elbow"], 0, 0)  # the hand's own angle, folded under
    for leg, s in (("BL", 1), ("BR", -1)):
        q[f"leg_{leg}_upper"] = (P["lie_hip"], 0, s * 6)
        q[f"leg_{leg}_lower"] = (P["lie_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (P["lie_ankle"], 0, 0)
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = (P.get(f"tail_{min(k, 2)}", 0) if k < 3 else P["tail_2"] * 0.5, 0, 0)
    return q


LIE = 10.0
# Lying, the neck lowered to a rest (its joints turned down) and the head held level on it.
LIE_NECK0, LIE_NECK = 26.0, 12.0
LIE_HEAD = -(LIE_NECK0 + 3 * LIE_NECK) + 6.0


def pose_lie(t, P=L):
    """Lying on its belly, the legs folded under, the neck lowered and the head turning slowly, the
    tail along the ground (10 s)."""
    p = t / LIE
    br = S(TAU * p * 2)
    q = lie_body(P)
    q.update(neck_pose(LIE_NECK, 10 * S(TAU * p), LIE_HEAD, head_yaw=5 * S(TAU * p + 1.0), first=LIE_NECK0))
    q.update({"root": (0, -P["lie_drop"] + 0.2 * (1 + br), 0), "spine_1": (0.5 * br, 0, 0)})
    for k in range(1, len(TAIL)):
        rx = q[f"tail_{k}"][0]
        q[f"tail_{k}"] = swing((0, 0, -1), 2.0 * S(TAU * p - 0.6 * k), (rx, 0, 0))
    q.update(chew(t, 0.5, 2))
    return q


BLINKS = {"idle": [2.5, 13.0], "lie": [3.0, 7.5]}
BLINK = 0.12
ANIMS = {"idle": (pose_idle, IDLE), "walk": (pose_walk, WALK), "run": (pose_run, RUN), "lie": (pose_lie, LIE)}


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}


def frame_times(name, dur):
    n = {"idle": 128, "lie": 80, "walk": 24, "run": 24}.get(name, 16)
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


GROUPS = {"front paws": lambda n: n in ("leg_FL_paw", "leg_FR_paw"),
          "back paws": lambda n: n in ("leg_BL_paw", "leg_BR_paw"),
          "body": lambda n: n in BODY,
          "muzzle": lambda n: n in ("snout", "jaw", "skull", "dome"),
          "neck": lambda n: n.startswith("neck_"),
          "legs": lambda n: n.startswith("leg_"),
          "tail": lambda n: n.startswith("tail_")}
GROUPS.update({f"tail_{k}": (lambda n, k=k: n == f"tail_{k}") for k in range(len(TAIL))})


def fit_poses(spec):
    """Lying (the belly on the ground, the folded legs under it, the tail along the ground, nothing
    below) and drinking (the mouth at the water), by forward kinematics."""
    top = max(e["hi"][1] for e in spec["elements"] if e["name"] in ("hips", "chest"))
    front = lambda n: n.startswith(("leg_FL", "leg_FR"))  # noqa: E731
    back = lambda n: n.startswith(("leg_BL", "leg_BR"))  # noqa: E731
    paw = {front: GROUPS["front paws"], back: GROUPS["back paws"]}

    def lie_cost(P, side):
        # The legs folded under the belly with their feet on the ground beside it; no folded leg
        # standing up over the back, out in front of the chest or behind the hips.
        pose = lie_body(dict(L, **P))
        g = lowest(spec, dict(pose), {"body": GROUPS["body"], "leg": side, "paw": paw[side]})
        f = g["body"] - 0.8
        lo, hi = extent(spec, pose, side)
        out = max(0.0, hi[1] - (top - 40)) + max(0.0, hi[2] - 234) + max(0.0, -60 - lo[2])
        return (g["leg"] - f) ** 2 + 1.0 * (g["paw"] - f) ** 2 + 2 * out ** 2
    # The front legs camel-fashion: the upper arm about upright, the forearm folded back along the
    # ground under the chest and belly, where the body hides it.
    # (odd steps: no exact right angles in a resting pose, which line faces up)
    best = fit({"lie_front": list(range(-38, 61, 6)), "lie_elbow": list(range(51, 172, 8)),
                "lie_wrist": list(range(-173, 176, 24))}, lambda P: lie_cost(P, front))[1]
    best = fit({k: [v - 3, v, v + 3] for k, v in best.items()}, lambda P: lie_cost(P, front))[1]  # then finer
    L.update(best)
    best = fit({"lie_hip": list(range(-97, -9, 6)), "lie_knee": list(range(101, 172, 6)), "lie_ankle": list(range(-169, -39, 6))},
               lambda P: lie_cost(P, back))[1]
    best = fit({k: [v - 3, v, v + 3] for k, v in best.items()}, lambda P: lie_cost(P, back))[1]
    L.update(best)
    g = lowest(spec, lie_body(L), GROUPS)
    L["lie_drop"] = round(float(min(g["body"], g["legs"])), 2)

    def tail_cost(P):
        g = lowest(spec, lie_body(dict(L, **P)), GROUPS)
        f = L["lie_drop"]
        # The back half along the ground, the root sloping down to it from the hips; nothing below.
        return sum((g[f"tail_{k}"] - f - 0.5) ** 2 for k in (2, 3, 4)) + 0.2 * (g["tail_1"] - f - 3.0) ** 2 \
            + 2000 * min(0.0, g["tail"] - f) ** 2 + 0.002 * (P["tail_0"] ** 2 + P["tail_1"] ** 2 + P["tail_2"] ** 2)
    best = fit({"tail_0": list(range(-30, 11, 2)), "tail_1": list(range(-30, 21, 2)), "tail_2": list(range(-20, 21, 2))}, tail_cost)[1]
    L.update(best)
    g = lowest(spec, lie_body(L), GROUPS)
    assert g["muzzle"] >= L["lie_drop"] - 0.05 and g["neck"] >= L["lie_drop"] - 0.05, f"the head sinks into the ground lying: {g}"

    def drink_cost(P):
        g = lowest(spec, drink_pose(1.0, dict(L, **P)), GROUPS)
        return (g["muzzle"] - 0.5) ** 2 + 400 * min(0.0, g["neck"] - 4.0) ** 2 \
            + 0.0005 * (P["drink_neck0"] ** 2 + 4 * P["drink_neck"] ** 2 + P["drink_head"] ** 2)
    best = fit({"drink_neck0": list(range(10, 61, 4)), "drink_neck": list(range(10, 41, 2)),
                "drink_head": list(range(0, 91, 5))}, drink_cost)[1]
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
    sheet, uv = pack(tiles, PACK)  # the layers pack at the same width (`Recorder.save`)
    LAYERS["rec"].save(OUT, [k for k, _ in tiles], pack, PACK)
    LAYERS["rec"] = None
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "sauropod", "texture": [sheet.shape[1], sheet.shape[0]],
            "bones": [{"name": n, "parent": p, "pivot": [float(v) for v in pv]} for n, p, pv in B], "elements": E}
    fit_poses(spec)
    spec["animations"] = animations([n for n, _, _ in B])
    spec["fitted"] = dict(L)
    return spec, sheet


PACK = 512

if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    spec, sheet = build()
    Image.fromarray(sheet, "RGBA").save(os.path.join(OUT, "texture.png"))
    with open(os.path.join(OUT, "spec.json"), "w") as f:
        json.dump(spec, f)
    print(len(spec["elements"]), "elements;", len(spec["bones"]), "bones; texture", sheet.shape[1], "x", sheet.shape[0],
          "; fitted", spec["fitted"])
