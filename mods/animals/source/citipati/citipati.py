#!/usr/bin/env python3
"""Citipati: the oviraptorid of the Gobi's red dunes that sat on its nests like a bird, made the
way Hytale makes creatures (boxes, a few flat feather panels, a hand-painted texture) with the
blocky-model skill, from research (README.md). No record of its colors: a dune bird's, chosen as
rules in `area` (sandy buff with a rufous back, a pale throat and breast, darker barred wings and
tail fan, a red crest and a dark slate beak).

    ~/.venvs/blocky/bin/python citipati.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each), y up, it faces +z. A grown *Citipati osmolskae*
is about 2.9 m long and 75-110 kg, emu-sized, on long legs: here the hips about 58 up in its
stance (0.9 m), the crest's top about 105 (1.65 m), 175 along the spine from the beak to the tail
fan's tip (2.7 m; 2.3 m as it stands, the neck raised); a
short, deep, toothless beak under a tall rounded crest, a long neck, long feathered arms, a short
tail ending in a fan. The head a little bigger than life.
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
from rig import euler, fit, qmul, quat, swing  # noqa: E402
from ground import grounded, lowest  # noqa: E402
from house import TX  # noqa: E402

# TX: texels a unit, 0.5 (32 a block). The eye's face, the skull's side, is 7 by 6 texels at 32 (the
# house rule wants 3 or more), so it paints at 32.

OUT = os.environ.get("CITIPATI_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

BODY, BACK, PALE, WING, BAR, TIP, CREST, BEAK, MOUTH, SCALE, CLAW = range(11)
# BODY the sandy-buff body feathers, BACK the rufous back, PALE the throat, breast and belly, WING
# the darker wing and tail feathers, BAR their dark bars, TIP the darkest feather tips, CREST the
# crest (red), BEAK the slate-grey horn of the beak, MOUTH the inside, SCALE the scaly feet, CLAW
# the claws.
BASE = [(200, 162, 112), (170, 104, 62), (228, 212, 176), (128, 88, 60), (80, 56, 42), (96, 68, 50),
        (172, 66, 50), (84, 82, 86), (120, 58, 54), (128, 114, 98), (58, 48, 46)]
SHADOW = [0.03, 0.02, 0.07, 0.02, 0.0, 0.0, 0.98, 0.74, 0.95, 0.05, 0.74]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (BODY, BACK, PALE)  # the soft body feathers; the wing bars, crest, beak, feet stay crisp
ROLES = [("coat", 0.5, 0), ("coat", 0, 0), ("coat", 1, 0), ("coat", 0, -1.0), ("coat", 0, -2.0), ("coat", 0, -2.2),
         "fixed", "fixed", "fixed", "fixed", "fixed"]

# At 32 a block the skull's side is 7 texels by 6: a 3 by 2 eye (a catchlight up and forward, a dark
# pupil, an amber iris), as the Psittacosaurus's. Drawn for the left side (east); the right gets it
# mirrored, so both catchlights point forward. No record of its eye's color: amber, a ground bird's.
_P, _H, _I, _a = (22, 16, 14), (230, 228, 220), (190, 128, 46), (214, 160, 70)
EYES.update({
    "citi_px3": dict(pattern=["HPI", "IPa"], colors={"P": _P, "H": _H, "I": _I, "a": _a}, halo="none", iris="Ia"),
})
for _k in [k for k in EYES if k.startswith("citi_px")]:
    EYES[_k + "_m"] = dict(EYES[_k], pattern=[row[::-1] for row in EYES[_k]["pattern"]])
EYE_STYLE = os.environ.get("CITIPATI_EYES", "citi_px3")
SKULL = ((-6.4, 92.0, 38.0), (6.4, 104.0, 52.0))  # short and deep, behind the beak and under the crest
# Its sides two texels out from the crest and the snout (at 5.0, its top and front showed 1.5-texel rims beside them; linecheck).
EYE_Z, EYE_Y = 47.0, 99.0  # high on the side, behind the crest's foot
LID_X = 6.65  # the eyelids' hinge plane, just outside the head's side
LID_UP = 0.3  # the hinge a little over the eye's top, so a turned head shows no rim of it

# ------------------------------------------------------------------ the model

# The neck: two segments, a bird's S, rising from inside the chest's front top to the head's joint
# inside the skull; each slimmer than the part it leaves, its end deep in the next.
NECK_BASE = np.array([0.0, 64.0, 27.0])
NECK = [(48.0, 19.0, 4.5), (78.0, 19.0, 3.4)]  # angle above level, length, half width
TAIL = [(16, 10.0), (14, 8.6), (12, 7.2), (10, 5.8)]  # length, width (0.7 a side thinner each)
TAIL0 = np.array([0.0, 66.0, -13.0])
LEG_X = 6.2
SHOULDER = (10.0, 63.0, 25.0)
ARM_X = 11.4  # the arm's middle, outside the chest's side (10.5)


def neck_points():
    pts, p = [NECK_BASE.copy()], NECK_BASE.copy()
    for ang, length, _ in NECK:
        a = math.radians(ang)
        p = p + length * np.array([0.0, math.sin(a), math.cos(a)])
        pts.append(p.copy())
    return pts


def tail_points():
    pts, p = [TAIL0.copy()], TAIL0.copy()
    for length, _ in TAIL:
        p = p + np.array([0, 0, -length])
        pts.append(p.copy())
    return pts


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
    npts = neck_points()
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 64, 0)), ("spine_1", "spine_0", (0, 64, 8)),
         ("neck_0", "spine_1", tuple(npts[0])), ("neck_1", "neck_0", tuple(npts[1])), ("head", "neck_1", tuple(npts[2])),
         ("jaw", "head", (0, 92.0, 43.0))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        sx, sy, sz = SHOULDER
        B += [(f"eyelid_{n}", "head", (s * LID_X, top + LID_UP, (za + zb) / 2)),
              (f"arm_{n}", "spine_1", (s * sx, sy, sz)), (f"forearm_{n}", f"arm_{n}", (s * ARM_X, 62.5, 14.0)),
              (f"hand_{n}", f"forearm_{n}", (s * ARM_X, 62.5, 1.5))]
    pts = tail_points()
    B += [(f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", tuple(pts[k])) for k in range(len(TAIL))]
    for s, n in ((1, "L"), (-1, "R")):
        x = LEG_X * s
        B += [(f"leg_{n}_thigh", "spine_0", (x, 62, 0)), (f"leg_{n}_shin", f"leg_{n}_thigh", (x, 41, 0)),
              (f"leg_{n}_foot", f"leg_{n}_shin", (x, 15, -0.5)), (f"leg_{n}_toes", f"leg_{n}_foot", (x, 2.6, -0.5))]
    return B


def elements():
    npts = neck_points()
    # The hips half a unit inside the chest's sides, the thighs 0.4 inside theirs (at 8.5, 1.7 units of the thighs'
    # backs showed past them; linecheck).
    E = [el("hips", "spine_0", (-10.0, 55, -14), (10.0, 72, 10)),
         el("chest", "spine_1", (-10.5, 48, 6), (10.5, 72.6, 30)),  # a deep, bird's body
         # The breast, lower and forward: two texels in from the chest's sides and out in front (at 8 wide and 3.5 out,
         # 1.2-1.8-texel bands of the chest beside it and of it before the chest; linecheck).
         el("breast", "spine_1", (-6.5, 44, 15.5), (6.5, 62, 34.2))]
    for k, ((ang, length, hw), p) in enumerate(zip(NECK, npts)):
        back, front = (3.0, 0.5) if k == 0 else (2.0, 4.0)  # the last reaches deep into the skull
        E.append(el(f"neck_{k}", f"neck_{k}", p + np.array([-hw, -hw, -back]), p + np.array([hw, hw, length + front]),
                    rot=(-ang, 0, 0), origin=tuple(p)))
    E += [el("skull", "head", *SKULL),
          # The upper beak's deep base: flush under the crest, two texels in from the skull (at 3.8 a rim; linecheck).
          el("snout", "head", (-2.3, 91.0, 50.5), (2.3, 99.5, 58.2)),  # its front 0.4 behind the crest's (flush)
          # Its hooked tip, a parrot's: narrower than the snout, 2.2 texels below it, its front half a unit past it and
          # its top inside it (at 2.6 wide, 88-96.5 and 55.6-59.4, bands of it showed beside the snout and the crest; linecheck).
          el("beak", "head", (-1.5, 86.6, 55.4), (1.5, 95.9, 58.7)),
          el("mouth", "head", (-3.0, 90.4, 42.0), (3.0, 92.2, 54.4)),
          el("jaw", "jaw", (-4.3, 87.0, 41.5), (4.3, 92.6, 55.0)),  # deep, shorter than the upper beak
          # The crest: a tall, rounded casque over the beak, as a cassowary's, rising straight from the
          # beak in front and stepping down behind; two boxes, the upper narrower and shorter than the
          # one below, so it reads round.
          # Each 0.4 narrower than the one below and its front flush, its back two texels short (at 2.1 wide, rims of
          # skull showed beside it; at 1.5-texel steps behind, bands; linecheck).
          # Two texels over the skull (at 106, one texel of its side showed above it; linecheck).
          el("crest", "head", (-1.9, 96.0, 44.0), (1.9, 108.0, 58.6)),
          # Deep in the crest, showing over two texels above it (at 105.5, 1.8; linecheck); the top tier, ending the
          # crest (rule 7: a third tier, a texel across, read as a line on top).
          el("crest_1", "head", (-1.5, 104.5, 48.0), (1.5, 115.0, 58.2))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, bottom = eye_box(s)
        h = top + LID_UP - bottom + 0.4  # a little over the eye all round, so no rim of it shows when shut
        lx0, lx1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top + LID_UP, za - 0.5), (lx1, top + LID_UP, zb + 0.5), quad="down"))
        # The wing, folded along the flank: the upper arm back from the shoulder, the forearm and hand
        # on back, each a little thinner than the last and 0.4 inside it (flush); the feathers hang
        # from forearm and hand as panels. Every part two texels across or more (rule 7: no boxes a
        # texel thick; the forearm and hand were 1.1 and 0.6 texels, lines beside the arm), the arm
        # out over two texels from the chest's side.
        ax = ARM_X * s
        a0, a1 = sorted((ax - 1.6 * s, ax + 4.7 * s))
        E.append(el(f"arm_{n}", f"arm_{n}", (a0, 60.0, 12.5), (a1, 65.5, 27.5)))
        f0, f1 = sorted((s * 10.9, s * 15.7))
        E.append(el(f"forearm_{n}", f"forearm_{n}", (f0, 60.5, 0.0), (f1, 65.1, 15.0)))
        h0, h1 = sorted((s * 11.3, s * 15.3))
        E.append(el(f"hand_{n}", f"hand_{n}", (h0, 61.0, -11.0), (h1, 64.7, 2.5)))
        q = "east" if s > 0 else "west"
        E.append(el(f"wing_{n}", f"forearm_{n}", (s * 13.6, 46.0, -1.0), (s * 13.6, 64.0, 15.5), quad=q, cut="wing"))
        E.append(el(f"primaries_{n}", f"hand_{n}", (s * 14.6, 46.0, -24.0), (s * 14.6, 64.5, 2.5), quad=q, cut="primaries"))
    pts = tail_points()
    for k, (length, w) in enumerate(TAIL):
        p = pts[k]
        hh = w * 0.9
        E.append(el(f"tail_{k}", f"tail_{k}", (-w / 2, p[1] - hh / 2, p[2] - length), (w / 2, p[1] + hh / 2, p[2] + (3.0 if k == 0 else 0.5))))
    # The fan of tail feathers on the last segment (a pygostyle's): two halves meeting along the
    # tail, a little raised and their outer edges lifted in a shallow V, so it shows from the side.
    tip = pts[-2]
    o = (0, tip[1] + 0.2, tip[2] - 1.0)
    for s, n in ((1, "L"), (-1, "R")):
        x0, x1 = sorted((0.0, s * 13.0))
        E.append(el(f"tail_fan_{n}", f"tail_{len(TAIL) - 1}", (x0, o[1], tip[2] - 30.0), (x1, o[1], o[2]),
                    quad="up", cut="fan", rot=(10, 0, s * 26), origin=o))
    for s, n in ((1, "L"), (-1, "R")):
        x = LEG_X * s
        # A feathered drumstick, 0.4 inside the hips' side (flush; linecheck), half a unit clear of the chest's.
        t0, t1 = sorted((x - 4.0 * s, x + 3.4 * s))
        E.append(el(f"thigh_{n}", f"leg_{n}_thigh", (t0, 40, -6), (t1, 66, 6)))
        E.append(el(f"shin_{n}", f"leg_{n}_shin", (x - 2.8, 14, -3.2), (x + 2.8, 42.5, 2.4)))
        E.append(el(f"foot_{n}", f"leg_{n}_foot", (x - 2.1, 2, -2.6), (x + 2.1, 16.5, 1.4)))
        E.append(el(f"toes_{n}", f"leg_{n}_toes", (x - 3.6, 0, -2.2), (x + 3.6, 2.8, 9.5)))
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


def bars(t, period, width=0.34):
    """Bands across `t` (units): a band at least a texel, the period at least three."""
    period = max(period, 3 / TX)
    return ((t / period) % 1.0) < max(width, 1 / (TX * period))


BODY_PARTS = ("hips", "chest", "breast", "neck_0", "neck_1")


def area(e, face, p, uu, vv, h):
    """A dune bird's coat: rufous above, sandy buff on the flanks, pale throat, breast and belly;
    darker wings and tail fan with dark bars; a red crest over a slate beak; scaly grey-brown feet."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, BODY)
    side = face in ("east", "west")
    t = vv / max(h, 1)
    if name in BODY_PARTS or name.startswith("tail_") and not name.startswith("tail_fan"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = PALE
        elif side:
            a = np.where(t < 0.3, BACK, np.where(t > 0.72, PALE, BODY))
        elif face == "south":
            a = np.where(t < 0.25, BACK, PALE) if name in ("breast", "chest", "neck_0", "neck_1") else np.full(x.shape, BODY)
        if name == "breast" and face != "up":
            a = np.where(t > 0.2, PALE, a)
        if name.startswith("neck_"):
            # the throat pale down the neck's front, the nape rufous: by the side of the box
            # the texel is on, in the neck's own frame (its 'up' is the nape)
            if face == "down":
                a[:] = PALE
            elif face == "up":
                a[:] = BACK
            elif side:
                a = np.where(t < 0.35, BACK, np.where(t > 0.6, PALE, BODY))
    elif name == "skull":
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = PALE
        elif side:
            a = np.where(t < 0.25, BACK, np.where(t > 0.8, PALE, BODY))
    elif name in ("snout", "beak"):
        a[:] = BEAK
        if name == "snout" and face == "down":
            a[:] = MOUTH
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else np.where(z > 48.5, BEAK, PALE)  # the lower beak's horn in front
    elif name == "mouth":
        a[:] = MOUTH
    elif name.startswith("crest"):
        a[:] = CREST
    elif name.startswith(("arm_", "forearm_", "hand_")):
        a = np.where(t < 0.5, BACK, BODY) if side or face == "up" else np.full(x.shape, BODY)
        if face == "up":
            a[:] = BACK
        if name.startswith("hand_") and face == "north":
            a[:] = CLAW
    elif name.startswith(("wing_", "primaries_")):
        a[:] = WING
        top = e["hi"][1]
        a = np.where(y > top - 4.0, BACK, a)  # the coverts over the flight feathers
        a = np.where((y <= top - 4.0) & bars(y, 7.0), BAR, a)
        a = np.where(y < e["lo"][1] + 5.0 + (2.0 if name.startswith("wing_") else 0.0), TIP, a)
    elif name.startswith("tail_fan"):
        a[:] = WING
        r = np.hypot(x, z - e["hi"][2])  # bands round the fan's root
        a = np.where(bars(r, 7.0), BAR, a)
        a = np.where(r > 25.0, TIP, a)
    elif name.startswith("thigh_"):
        if face == "up":
            a[:] = BODY
        elif face == "down":
            a[:] = PALE
        else:
            a = np.where(t < 0.35, BACK, np.where(t > 0.85, PALE, BODY))
    elif name.startswith("shin_"):
        a = np.where(y > 33, BODY, SCALE)  # feathered down to the middle of the shin
    elif name.startswith(("foot_", "toes_")):
        a[:] = SCALE
        if name.startswith("toes_"):
            if face == "south":
                a = np.where((np.floor(uu) % 2) == 0, CLAW, SCALE)  # three toes' claws, grooves between
            elif face == "up":
                a = np.where(((np.floor(uu) % 2) == 1) & (vv > a.shape[0] - 3), CLAW, SCALE)
    return a


def cut_mask(e, p, uu, vv, su, sv):
    """Which texels of a feather panel are there; the ragged tips alternate by texel, a texel deep."""
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    lo, hi = e["lo"], e["hi"]
    rag = sv * np.array([0, 1, 0, 2, 1, 0, 1])[np.floor(uu).astype(int) % 7]  # ragged, not a comb
    if e["cut"] == "wing":  # the secondaries, longest at the back of the forearm
        k = np.clip((hi[2] - z) / (hi[2] - lo[2]), 0, 1)
        bottom = lo[1] + 6.0 * (1 - k) + rag
        return y >= bottom
    if e["cut"] == "primaries":  # the hand's long feathers, sweeping back to a rounded point
        k = np.clip((hi[2] - z) / (hi[2] - lo[2]), 0, 1)  # 0 at the wrist, 1 at the tip
        bottom = lo[1] + 3.0 * (1 - k) + 13.0 * np.clip((k - 0.62) / 0.38, 0, 1) ** 1.6 + rag
        top = hi[1] - 9.0 * np.clip((k - 0.55) / 0.45, 0, 1) ** 1.4
        return (y >= bottom) & (y <= top)
    if e["cut"] == "fan":  # a fan, widest behind the tail's end, its tips ragged
        z0, z1 = lo[2], hi[2]
        d = np.clip((z1 - z) / (z1 - z0), 0, 1)
        half = 2.5 + 10.5 * np.sqrt(d) - su * ((np.floor(vv) % 2) == 0)
        round_end = np.hypot(x / 13.0, (z1 - z) / (z1 - z0)) <= 1.0 + 0.04 * ((np.floor(uu) % 2) == 0)
        return (np.abs(x) <= half) & round_end
    return np.ones(x.shape, bool)


LASH = (44, 32, 28)

# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def layer_props(e, face, p, a):
    """White where it is pale (throat, breast, belly); its marks on the body, neck, tail and wings."""
    n, y = e["name"], p[..., 1]
    white = np.where(a == PALE, 0.85, 0.15)
    ext = np.where(np.isin(a, (CREST, BEAK)), 1.0, 0.0)
    body = n in BODY_PARTS or n.startswith(("tail_", "thigh_", "wing_", "primaries_", "arm_", "forearm_"))
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
    pr = p
    if e["rot"]:
        o = np.array(e["origin"])
        pr = (p - o) @ R.T + o
    light_face = face
    if face == "down" and e["name"].startswith(("thigh_", "shin_", "foot_", "tail_", "arm_", "forearm_", "hand_", "jaw", "neck_")):
        light_face = "north"  # seen when a pose lifts or folds it
    # Colors from the box's own frame (the fan's bands, the neck's throat), light and occlusion from
    # where the texel really is.
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(pr, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    if occ is not None and (e["name"] in BODY_PARTS or e["name"].startswith(("arm_", "forearm_", "hand_", "thigh_"))):
        occ = np.minimum(occ, 0.3)  # the flank behind the folded wing, the thigh's top: shaded, not a dark red hole
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=pr[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"]:
        img[..., 3] = np.where(cut_mask(e, p, uu, vv, su, sv), 255, 0)
    if e["name"] == "skull" and face in ("east", "west"):
        side = 1 if face == "east" else -1
        stamp_eyes(img, [eye_center(side)], eye_style(side))
    if e["name"].startswith("eyelid"):  # a closed eye: the lash along the edge that ends up at the bottom
        col_idx = 0 if e["name"].endswith("L") else w - 1
        k = 1.0 if w >= 3 else 0.5
        img[:, col_idx, :3] = np.round(img[:, col_idx, :3] * (1 - k) + np.array(LASH) * k).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, pr,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

# The stance: on its long hind legs, the knees forward and the ankles back as a bird's, the toes flat
# (the angles sum to 0). Pecking: tipped forward at the hips, the neck and head down to the ground.
L = {"thigh": -28, "shin": 54, "foot": -42, "drop": 0.0, "peck_lean": 15, "peck_n0": 40, "peck_n1": 60, "peck_head": 30,
     "rest_thigh": -70, "rest_shin": 140, "rest_foot": -100, "rest_drop": 0.0, "rest_pitch": 0.0, "wing_roll": 40.0, "wing_pitch": 0.0, "run_lift": 0.0,
     "graze_lean": 20, "graze_n0": 40, "graze_n1": 20, "graze_head": 10,
     "sleep_n0": 40, "sleep_yaw": 120, "sleep_n1": -60, "sleep_yaw1": -15, "sleep_head": 15, "sleep_yaw2": 90}


def legs(base_t, base_s, base_f, swing_l=0.0, swing_r=0.0, lift_l=0.0, lift_r=0.0, lean=0.0):
    q = {}
    for n, sw, lift in (("L", swing_l, lift_l), ("R", swing_r, lift_r)):
        t = base_t + sw - 0.4 * lift - lean
        s = base_s + lift
        f = base_f - 0.4 * lift
        q[f"leg_{n}_thigh"], q[f"leg_{n}_shin"], q[f"leg_{n}_foot"] = (t, 0, 0), (s, 0, 0), (f, 0, 0)
        q[f"leg_{n}_toes"] = (-(t + s + f + lean), 0, 0)  # toes flat on the ground
    return q


def tail_sway(p, n, amp, lift=0.0, fan=0.0):
    q = {}
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = swing((0, 0, -1), amp * (0.5 if k == 0 else 1) * S(TAU * p * n - 0.6 * k),
                               (lift if k == 0 else fan if k == len(TAIL) - 1 else 0, 0, 0))
    return q


def wings(out=0.0, lift=0.0):
    """The wings folded along the flanks; `out` lifts them a little away from the body (degrees)."""
    q = {}
    for n, s in (("L", 1), ("R", -1)):
        q[f"arm_{n}"] = (lift, 0, s * out)
        q[f"forearm_{n}"] = (0, 0, 0)
        q[f"hand_{n}"] = (0, 0, 0)
    return q


def stand(lean=0.0):
    q = {"root": (0, -L["drop"], 0), "spine_0": (lean, 0, 0), "neck_0": (0, 0, 0), "neck_1": (0, 0, 0), "head": (0, 0, 0)}
    q.update(legs(L["thigh"], L["shin"], L["foot"], lean=lean))
    q.update(wings())
    return q


def peck_pose(g, P=L):
    """Tipped forward at the hips, the neck and head down to the ground, by `g` (0..1)."""
    q = stand(P["peck_lean"] * g)
    q["neck_0"] = (P["peck_n0"] * g, 0, 0)
    q["neck_1"] = (P["peck_n1"] * g, 0, 0)
    q["head"] = (P["peck_head"] * g, 0, 0)
    q["tail_0"] = (-0.8 * P["peck_lean"] * g, 0, 0)  # the tail kept about level
    return q


def keys(t, pts):
    """Piecewise-linear keys [(t, v)]: holds where two keys share a value, snaps where they're close."""
    ts, vs = zip(*pts)
    return float(np.interp(t, ts, vs))


def pose_idle(t):
    """Pecking at the ground (seeds, a lizard, an egg), then up to look about with a bird's head
    snaps (hold, jump, hold), two blinks, then down again (8 s)."""
    br = S(TAU * t / 8.0 * 4)
    # down for 3.2 s with quick jabs at the ground, up and looking about, down again
    g = keys(t, [(0, 1), (3.2, 1), (3.7, 0), (7.3, 0), (7.8, 1), (8, 1)])
    jab = max(0.0, S(TAU * t * 1.25)) ** 3 if t < 3.2 else 0.0
    q = peck_pose(g * (0.86 + 0.14 * jab))
    yaw = keys(t, [(0, 0), (3.9, 0), (3.98, 30), (4.9, 30), (4.98, -24), (5.9, -24), (5.98, 12), (6.8, 12), (6.88, 0), (8, 0)])
    roll = keys(t, [(0, 0), (3.9, 0), (3.98, 10), (4.9, 10), (4.98, -6), (5.9, -6), (5.98, 0), (8, 0)])
    q["head"] = (q["head"][0] + keys(t, [(0, 0), (5.98, 0), (6.06, -8), (6.8, -8), (6.88, 0), (8, 0)]), yaw, roll)
    q["neck_1"] = (q["neck_1"][0], 0.3 * yaw, 0)
    q["root"] = (0, q["root"][1] + 0.12 * (1 + br), 0)  # breathing, the toes never below the ground
    q["spine_1"] = (0.6 * br, 0, 0)
    q.update(tail_sway(t / 8.0, 2, 4, lift=-0.8 * L["peck_lean"] * g, fan=-4))
    q["jaw"] = (6 * jab if t < 3.2 else 0, 0, 0)
    return q


def pose_walk(t):
    p = t / 1.0
    a = TAU * p
    q = stand(4)
    # A foot swings forward (thigh angle falling) while it's lifted: swing -sin, lift +cos.
    q.update(legs(L["thigh"], L["shin"], L["foot"], -24 * S(a), 24 * S(a), 30 * max(0.0, S(a + math.pi / 2)),
                  30 * max(0.0, S(a - math.pi / 2)), lean=4))
    q["root"] = (0, q["root"][1] + 0.9 * abs(S(a)), 0)
    # A bird's head bob: thrust forward, then held still while the body catches up (twice a cycle).
    bob = keys(p % 0.5, [(0, 0), (0.15, 1), (0.42, 1), (0.5, 0)])
    q["neck_0"] = (6 * bob, 0, 0)
    q["neck_1"] = (-4 * bob, 0, 0)
    q["head"] = (-2 * bob, 0, 0)
    q["spine_1"] = (0, 2 * S(a), 0)
    q.update(tail_sway(p, 1, 6, fan=-4))
    return q


def pose_run(t):
    """A long-legged run, leaning forward, the neck out, the wings a little out for balance (0.6 s)."""
    p = t / 0.6
    a = TAU * p
    q = stand(12)
    q.update(legs(L["thigh"] - 4, L["shin"], L["foot"], -38 * S(a), 38 * S(a), 58 * max(0.0, S(a + math.pi / 2)),
                  58 * max(0.0, S(a - math.pi / 2)), lean=12))
    q["root"] = (0, q["root"][1] + 1.8 * abs(S(a)) + L["run_lift"], 0)
    q["neck_0"] = (16, 0, 0)
    q["neck_1"] = (-6, 0, 0)
    q["head"] = (-20, 0, 0)
    q.update(wings(out=10 + 4 * S(2 * a), lift=-6))
    q.update(tail_sway(p, 1, 3, lift=-12, fan=-6))
    return q


def brood_wings(P=L):
    """The arms spread to the sides and back round the nest, the feathers fanned down to the ground
    on both sides (the Ukhaa Tolgod brooders: Norell et al. 1995, 2018)."""
    q = {}
    for n, s in (("L", 1), ("R", -1)):
        # out and back from the shoulder, then turned about its own length so the feathers fan out
        q[f"arm_{n}"] = euler(qmul(quat(P["wing_pitch"], -s * 34, 0), quat(0, 0, s * P["wing_roll"])))
        q[f"forearm_{n}"] = (0, s * 22, 0)
        q[f"hand_{n}"] = (0, s * 16, 0)
    return q


def pose_lie(t, P=L):
    """Brooding: sat down on its folded legs, the breast low, the arms and their feathers spread out
    to both sides, the neck up and the head turning now and then (6 s)."""
    p = t / 6.0
    br = S(TAU * p * 2)
    q = {"root": (0, -P["rest_drop"] + 0.1 * (1 + br), 0), "spine_0": (P["rest_pitch"], 0, 0), "spine_1": (0.8 * br, 0, 0)}
    q.update(legs(P["rest_thigh"], P["rest_shin"], P["rest_foot"], lean=P["rest_pitch"]))
    q.update(brood_wings(P))
    q["neck_0"] = (-8 - P["rest_pitch"], 0, 0)
    yaw = keys(t, [(0, 0), (1.9, 0), (1.98, 22), (3.2, 22), (3.28, -16), (4.6, -16), (4.68, 0), (6, 0)])
    q["neck_1"] = (4, 0.3 * yaw, 0)
    q["head"] = (4, yaw, 0)
    q.update({f"tail_{k}": (8 if k == 0 else 0, 3 * S(TAU * p - 0.5 * k) if k else 0, 0) for k in range(len(TAIL))})
    return q


# ------------------------------------------------------------------ behaviour clips (`anim.rs`)
# Each a whole-body pose over what plays below on the bones it moves (`to_bedrock.as_action`): held
# states and the gaits loop, the rest play once. A mother (90 kg, over `roles::GUARDS_KG`) stands her
# ground for her young: rear (a display), then stare, then charge. No stalk or pounce: not a hunter.


def still(t, period=4.0, lean=0.0, tail=1.5):
    """Standing and breathing, tipped at the hips by `lean` (the legs keep their place), the tail held
    against it and swaying a little: what a clip builds on."""
    br = S(TAU * t / period * 2)
    q = stand(lean)
    q["root"] = (0, q["root"][1] + 0.12 * (1 + br), 0)  # the toes never below the ground
    q["spine_1"] = (0.6 * br, 0, 0)
    q.update(tail_sway(t / period, 1, tail, lift=-0.8 * lean, fan=-4))
    return q


def upright(q, lean, n0, n1, head=None, yaw=0.0, roll=0.0):
    """The neck raised over a body tipped by `lean`: neck_0 and neck_1 pitched by n0, n1 (negative is
    back), the head kept level unless `head` is given; `yaw` turned mostly at the head, a bird's way."""
    h = -(lean + n0 + n1) if head is None else head
    # (turned at neck_1 by a quarter at most: past 15 degrees there its corners poke out of neck_0)
    q.update({"neck_0": (n0, 0, 0), "neck_1": (n1, 0.25 * yaw, 0), "head": (h, 0.75 * yaw, roll)})
    return q


def spread(q, out, up=0.0, fold=0.0, open_=0.0, sides=("L", "R")):
    """The wings spread from the flanks by `out` degrees (yawed out from the shoulder; 75: straight out
    to the side, the feather panels turned to face forward), raised by `up`, the forearm and hand
    opened on by `open_` (or folded back in by `fold`)."""
    for n, s in (("L", 1), ("R", -1)):
        if n not in sides:
            continue
        q[f"arm_{n}"] = euler(qmul(quat(0, 0, s * up), quat(0, -s * out, 0)))
        q[f"forearm_{n}"] = (0, s * (fold - open_), 0)
        q[f"hand_{n}"] = (0, s * (fold - 0.6 * open_), 0)
    return q


def head_down(g, P=L):
    """Tipped forward at the hips, the neck and head down by `g` (1: the beak's tip half a unit over the
    ground, the lowest of the head, fitted; the legs straight under it), the tail kept about level. (The
    idle's pecking tips the head past the vertical, its crest lowest and the beak 7 units up: its own.)"""
    lean = P["graze_lean"] * g
    q = stand(lean)
    # the hips' joints, two units under the spine's pivot, rise as it tips: the body let down to keep the toes on the ground
    q["root"] = (0, q["root"][1] - 2.0 * (1 - math.cos(math.radians(lean))), 0)
    q["neck_0"] = (P["graze_n0"] * g, 0, 0)
    q["neck_1"] = (P["graze_n1"] * g, 0, 0)
    q["head"] = (P["graze_head"] * g, 0, 0)
    return q


def peck_still(g, t, period, P=L):
    """The beak toward the ground by `g` (`head_down`), breathing, the tail kept level."""
    br = S(TAU * t / period * 2)
    q = head_down(g, P)
    q["root"] = (0, q["root"][1] + 0.12 * (1 + br), 0)
    q["spine_1"] = (0.6 * br, 0, 0)
    q.update(tail_sway(t / period, 1, 2, lift=-0.8 * P["graze_lean"] * g, fan=-4))
    return q


def pose_stare(t):
    """Half aware: frozen tall, the head high and level, locked on, only the breath moving (2 s, held)."""
    q = still(t, 2.0, lean=-4, tail=0.6)
    return upright(q, -4, -12, 6)


def pose_graze(t):
    """Pecking at the ground (seeds, insects, fallen fruit): quick jabs, the beak snapping at each, the
    head working side to side (2.4 s, held)."""
    p = t / 2.4
    jab = max(0.0, S(TAU * p * 3)) ** 3
    q = peck_still(0.84 + 0.16 * jab, t, 2.4)
    q["neck_1"] = (q["neck_1"][0], 8 * S(TAU * p), 0)
    q["head"] = (q["head"][0], 6 * S(TAU * p), 0)
    q["jaw"] = (7 * max(0.0, S(TAU * p * 3 - 0.9)), 0, 0)
    return q


def pose_drink(t):
    """Drinking a bird's way: the beak dipped into the water, then raised and tipped up to let it run
    down (2 s, held)."""
    p = t / 2.0
    d = keys(p, [(0, 1), (0.3, 1), (0.45, 0.35), (0.85, 0.35), (1, 1)])
    up = keys(p, [(0, 0), (0.35, 0), (0.5, 1), (0.8, 1), (0.95, 0), (1, 0)])
    q = peck_still(d, t, 2.0)
    q["head"] = (q["head"][0] - 26 * up, 0, 0)
    q["neck_1"] = (q["neck_1"][0] - 10 * up, 0, 0)
    q["jaw"] = (4 * max(0.0, S(TAU * p * 4)) * (1 - up), 0, 0)
    return q


def pose_sniff(t):
    """Down to the ground, a few pecks and a look along it side to side, and up (2.2 s, once)."""
    p = t / 2.2
    d = keys(p, [(0, 0), (0.25, 1), (0.75, 1), (1, 0)])
    jab = max(0.0, S(TAU * p * 4)) ** 2
    q = peck_still(d * (0.88 + 0.12 * jab), t, 2.2)
    q["neck_1"] = (q["neck_1"][0], 14 * d * S(TAU * p * 1.5), 0)
    q["head"] = (q["head"][0], 10 * d * S(TAU * p * 1.5), 8 * d * S(TAU * p * 1.5))
    q["jaw"] = (5 * d * jab, 0, 0)
    return q


LOOK = [(0, 0), (0.1, 0), (0.13, 42), (0.38, 42), (0.41, -38), (0.66, -38), (0.69, 14), (0.84, 14), (0.87, 0), (1, 0)]
TILT = [(0, 0), (0.1, 0), (0.13, 12), (0.38, 12), (0.41, -8), (0.66, -8), (0.69, 0), (1, 0)]


def pose_look(t):
    """Looking round a bird's way: up, then the head snapped to one side and held, cocked, snapped to
    the other, back (2.6 s, once)."""
    p = t / 2.6
    up = keys(p, [(0, 0), (0.1, 1), (0.9, 1), (1, 0)])
    q = still(t, 2.6, lean=-3 * up)
    return upright(q, -3 * up, -10 * up, 4 * up, yaw=keys(p, LOOK), roll=keys(p, TILT))


WATCH = [(0, 0), (0.12, 0), (0.14, 34), (0.36, 34), (0.38, 12), (0.5, 12), (0.52, -30), (0.76, -30), (0.78, -8), (0.88, -8),
         (0.9, 0), (1, 0)]


def pose_watch(t):
    """A sentinel: standing tall, the head high, turning in a bird's jerks (snapped round and held) one
    way and the other (4 s, held)."""
    p = t / 4.0
    q = still(t, 4.0, lean=-7, tail=1.0)
    return upright(q, -7, -16, 6, yaw=keys(p, WATCH), roll=0.2 * keys(p, WATCH))


def preen(t, dur, side, reach):
    """The beak working through one wing's feathers, the wing lifted out from the flank to meet it;
    `reach` how far along the wing the head runs (a quick nibble or a long grooming)."""
    p = t / dur
    d = keys(p, [(0, 0), (0.18, 1), (0.85, 1), (1, 0)])
    n = "L" if side > 0 else "R"
    along = d * reach * S(TAU * p * 1.5) if reach else 0.0
    q = still(t, dur / 2)
    # (turned at neck_0, pitched forward with it: turned 70 or more, or less pitched, its corners poke out of
    # the chest; neck_1 bent forward, not turned)
    q.update({"spine_1": (0, side * 6 * d, 0), "neck_0": (40 * d, side * 60 * d, 0), "neck_1": (30 * d + 6 * along, 0, 0),
              "head": (-10 * d, side * 40 * d + side * 8 * along, side * 10 * d)})
    q["jaw"] = (8 * d * max(0.0, S(TAU * p * 7)), 0, 0)
    spread(q, 26 * d, up=10 * d, open_=8 * d, sides=(n,))
    return q


def pose_scratch(t):
    """An itch: a quick nibble in the left wing's feathers, the wing lifted (1.8 s, once)."""
    return preen(t, 1.8, 1, 0.0)


def pose_groom(t):
    """Preening: the right wing held out, the beak run along its feathers (2.6 s, once)."""
    return preen(t, 2.6, -1, 1.0)


def pose_stretch(t):
    """Waking up: both wings spread and raised, the neck long and up, a yawn; then a bird's wing-and-leg
    stretch, the left wing and leg pushed out behind together (2.4 s, once)."""
    p = t / 2.4
    a = keys(p, [(0, 0), (0.18, 1), (0.45, 1), (0.55, 0), (1, 0)])
    b = keys(p, [(0, 0), (0.5, 0), (0.65, 1), (0.88, 1), (1, 0)])
    q = still(t, 2.4)
    upright(q, 0, -14 * a, 4 * a, head=-6 * a)
    q["jaw"] = (22 * a * keys(p, [(0, 0), (0.15, 0), (0.25, 1), (0.4, 1), (0.48, 0), (1, 0)]), 0, 0)
    spread(q, 60 * a, up=24 * a, open_=20 * a)
    if b > 0:
        q["arm_L"] = euler(qmul(quat(0, 0, -6 * b), quat(14 * b, -20 * b, 0)))
        q["forearm_L"] = (0, -10 * b, 0)
        q["hand_L"] = (0, -6 * b, 0)
    q.update(legs(L["thigh"], L["shin"], L["foot"], swing_l=34 * b, lift_l=-14 * b))
    q["tail_0"] = (q["tail_0"][0] + 10 * b, q["tail_0"][1], 0)
    return q


def pose_alarm(t):
    """Startled: drawn up tall, the neck pulled back, a hiss with the beak open, the wings flared out
    from the flanks, the tail raised (1.2 s, once)."""
    p = t / 1.2
    up = keys(p, [(0, 0), (0.1, 1), (0.75, 1), (1, 0)])
    hiss = keys(p, [(0, 0), (0.12, 0), (0.2, 1), (0.6, 1), (0.7, 0), (1, 0)])
    q = still(t, 1.2, lean=-8 * up, tail=1.0)
    upright(q, -8 * up, -14 * up, -2 * up, head=14 * up)
    q["jaw"] = (24 * hiss, 0, 0)
    spread(q, 24 * up, up=26 * up, open_=10 * up)
    q["tail_0"] = (q["tail_0"][0] + 12 * up, q["tail_0"][1], 0)
    q["tail_3"] = (q["tail_3"][0] + 14 * up, q["tail_3"][1], 0)
    return q


def pose_rear(t):
    """A mother's threat display, as the stand begins: drawn up to her full height, the wings spread
    wide to the sides with their feathers opened toward the threat, the crest held high, the tail raised
    and its fan tipped up behind, a hiss; the feet planted (its height set from them frame by frame,
    `ground_clips`) (1.4 s, once)."""
    p = t / 1.4
    r = keys(p, [(0, 0), (0.22, 1), (0.78, 1), (1, 0)])
    hiss = keys(p, [(0, 0), (0.2, 0), (0.28, 1), (0.62, 1), (0.7, 0), (1, 0)])
    sway = r * S(TAU * min(1.0, max(0.0, (p - 0.22) / 0.56)))
    q = still(t, 1.4, lean=-12 * r, tail=0.6)
    upright(q, -12 * r, -14 * r, 2 * r, head=8 * r, yaw=6 * sway)
    q["jaw"] = (26 * hiss, 0, 0)
    spread(q, 72 * r, up=22 * r + 4 * sway, open_=26 * r)
    q["tail_0"] = (q["tail_0"][0] + 24 * r, q["tail_0"][1], 0)
    q["tail_3"] = (q["tail_3"][0] + 12 * r, q["tail_3"][1], 0)  # (more, and its corners poke out of tail_2)
    return q


def pose_play(t):
    """A young one's play: two little hops off both feet, the wings flapped out, the head tossed
    (1.4 s, once; it takes off from and lands on the ground, `ground_clips`)."""
    p = t / 1.4
    hop = max(0.0, S(TAU * p * 2))
    q = still(t, 1.4, lean=4 * hop)
    q["root"] = (0, q["root"][1] + 7.0 * hop, 0)
    q.update(legs(L["thigh"], L["shin"], L["foot"], -8 * hop, -8 * hop, 26 * hop, 26 * hop, lean=4 * hop))
    upright(q, 4 * hop, -6 * hop, 0, head=4 * hop + 8 * S(TAU * p), yaw=18 * S(TAU * p), roll=10 * S(TAU * p))
    spread(q, 40 * hop, up=30 * hop, open_=10 * hop)
    q.update(tail_sway(p, 2, 5, lift=-3 * hop, fan=6 * hop))
    return q


def pose_nuzzle(t):
    """A brooding parent's nudge: the beak lowered to a chick's height, two gentle pushes, a nibble
    (2 s, once)."""
    p = t / 2.0
    d = keys(p, [(0, 0), (0.2, 1), (0.85, 1), (1, 0)])
    push = max(0.0, S(TAU * p * 2)) * d
    q = peck_still(0.6 * d, t, 2.0)
    q["neck_1"] = (q["neck_1"][0] + 8 * push, 0, 0)
    q["head"] = (q["head"][0] - 10 * push, 0, 6 * d * S(TAU * p))
    q["jaw"] = (5 * d * max(0.0, S(TAU * p * 4)), 0, 0)
    return q


def pose_flinch(t):
    """Hit: a jerk aside, the neck pulled down and back, the head ducked and turned away, the wings
    clapped to the flanks, the tail clamped (0.45 s, once)."""
    p = t / 0.45
    d = keys(p, [(0, 0), (0.15, 1), (0.45, 0.8), (1, 0)])
    q = still(t, 0.45, tail=0.0)
    q.update({"spine_1": (0, 5 * d, 0), "neck_0": (14 * d, -8 * d, 0), "neck_1": (8 * d, 0, 0), "head": (-12 * d, -14 * d, 0),
              "jaw": (8 * d, 0, 0)})
    q.update(wings(out=-2 * d, lift=-4 * d))
    q["tail_0"] = (q["tail_0"][0] - 8 * d, 0, 0)
    return q


def pose_approach(t):
    """Curious, coming to look: slow careful steps, the neck stretched forward and low, the beak level,
    the head cocked one way and the other (1.6 s a cycle, in place of the walk; its height set from its
    feet, `ground_clips`)."""
    p = t / 1.6
    a = TAU * p
    lean = 8
    q = stand(lean)
    q.update(legs(L["thigh"] - 4, L["shin"] + 8, L["foot"] - 4, -18 * S(a), 18 * S(a), 24 * max(0.0, S(a + math.pi / 2)),
                  24 * max(0.0, S(a - math.pi / 2)), lean=lean))
    bob = keys(p % 0.5, [(0, 0), (0.15, 1), (0.42, 1), (0.5, 0)])
    upright(q, lean, 26 + 4 * bob, 12, roll=10 * S(a))
    q["spine_1"] = (0, 2 * S(a), 0)
    q.update(tail_sway(p, 1, 4, lift=-0.8 * lean, fan=-4))
    return q


def pose_charge(t):
    """Charging a hunter: the run driven hard, the neck stretched straight out ahead and the head low
    and forward, the beak open, the wings a little out (0.6 s a cycle, in place of the walk and the run;
    `ground_clips` sets its height from its feet, keeping the bounce)."""
    p = t / 0.6
    a = TAU * p
    q = pose_run(t)
    upright(q, 12, 30, 38, head=-70 + 3 * S(2 * a))  # (at -77 the neck's top lies in the eyelids' plane: they flicker)
    q["jaw"] = (10, 0, 0)
    q.update(wings(out=16 + 4 * S(2 * a), lift=-10))
    return q


def pose_sleep(t, P=L):
    """Asleep on the nest: the brooding pose, the head turned back over the shoulder and laid on the
    back, the beak tucked in, eyes shut, a slow breath (6 s, held)."""
    q = pose_lie(0.0, P)
    br = S(TAU * t / 6.0 * 2)
    q.update({"root": (0, -P["rest_drop"] + 0.1 * (1 + br), 0), "spine_1": (0.8 * br, 0, 0),
              "neck_0": (P["sleep_n0"], P["sleep_yaw"], 0), "neck_1": (P["sleep_n1"], P["sleep_yaw1"], 0),
              "head": (P["sleep_head"], P["sleep_yaw2"], 0), "jaw": (0.01, 0, 0)})
    q.update({"eyelid_L": (0, 0, 90), "eyelid_R": (0, 0, -90)})
    return q


def pose_death(t):
    """Dying: the legs give, it topples onto its side, the neck and tail fall slack, the wings loose,
    the eyes shut; it stays (1.4 s, holds; each frame on its lowest point, `ground_clips`)."""
    p = t / 1.4
    fall = keys(p, [(0, 0), (0.35, 0.35), (0.7, 1), (1, 1)])
    q = stand()
    q["root"] = (0, -L["drop"] - 40 * fall, 0)
    q.update({"spine_0": (0, 0, 84 * fall), "neck_0": (30 * fall, 0, -20 * fall), "neck_1": (30 * fall, 0, -10 * fall),
              "head": (10 * fall, 0, -10 * fall), "jaw": (12 * fall, 0, 0)})
    q.update(legs(L["thigh"] + 24 * fall, L["shin"] - 30 * fall, L["foot"] + 6 * fall))
    q.update(wings(out=8 * fall))
    q.update({f"tail_{k}": (0, 0, (-6 if k == 0 else -3) * fall) for k in range(len(TAIL))})
    q.update({"eyelid_L": (0, 0, 90 * fall), "eyelid_R": (0, 0, -90 * fall)})
    return q


BLINKS = {"idle": [4.4, 6.3], "lie": [1.2, 4.0], "stare": [1.3], "graze": [1.1], "look": [1.7], "watch": [1.0, 3.3],
          "drink": [1.3], "approach": [0.9]}
BLINK = 0.09
ANIMS = {"idle": (pose_idle, 8.0), "walk": (pose_walk, 1.0), "run": (pose_run, 0.6), "lie": (pose_lie, 6.0),
         "stare": (pose_stare, 2.0), "graze": (pose_graze, 2.4), "drink": (pose_drink, 2.0), "sniff": (pose_sniff, 2.2),
         "look": (pose_look, 2.6), "scratch": (pose_scratch, 1.8), "groom": (pose_groom, 2.6), "stretch": (pose_stretch, 2.4),
         "sleep": (pose_sleep, 6.0), "alarm": (pose_alarm, 1.2), "rear": (pose_rear, 1.4), "play": (pose_play, 1.4),
         "nuzzle": (pose_nuzzle, 2.0), "flinch": (pose_flinch, 0.45), "death": (pose_death, 1.4), "watch": (pose_watch, 4.0),
         "approach": (pose_approach, 1.6), "charge": (pose_charge, 0.6)}
SNAPS = {"idle": [3.9, 3.98, 4.9, 4.98, 5.9, 5.98, 6.06, 6.8, 6.88], "lie": [1.9, 1.98, 3.2, 3.28, 4.6, 4.68],
         "walk": [0.15, 0.42, 0.5, 0.65, 0.92], "look": [2.6 * k for k, _ in LOOK[1:-1]], "watch": [4.0 * k for k, _ in WATCH[1:-1]],
         "approach": [1.6 * (h + k) for h in (0, 0.5) for k in (0.15, 0.42)]}
# The held ones and the gaits loop; a clip played once must not (the engine loops a clip whose file
# says so, whatever the game asks); the death holds its last frame.
LOOPS = {"stare": True, "graze": True, "drink": True, "sleep": True, "watch": True, "approach": True, "charge": True,
         "death": "hold_on_last_frame"}
BLENDS = {"flinch": (0.05, 0.2), "death": (0.1, 0.0), "sleep": (0.6, 0.6), "graze": (0.4, 0.4), "watch": (0.4, 0.4),
          "approach": (0.3, 0.3), "charge": (0.2, 0.3)}
STANDING = ("idle", "walk", "run", "lie")


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}


def frame_times(name, dur):
    n = {"idle": 96, "lie": 60, "walk": 20, "run": 16, "sleep": 36, "flinch": 12, "watch": 48, "approach": 24, "charge": 16,
         "look": 40}.get(name, 16 if name in STANDING else 24)
    ts = {round(dur * k / n, 4) for k in range(n + 1)} | {round(s, 4) for s in SNAPS.get(name, [])}
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
    # that stays at 0: kept at a hundredth of a degree, so the idle's pecking lean, neck and tail don't
    # show through a clip that holds them straight. The eyelids are left to the idle's blinks.
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

GROUPS = {"toes": lambda n: n.startswith("toes_"), "body": lambda n: n in ("hips", "chest", "breast"),
          "tail": lambda n: n.startswith("tail_") and not n.startswith("tail_fan"), "fan": lambda n: n.startswith("tail_fan"),
          "beak": lambda n: n in ("beak", "jaw"), "wings": lambda n: n.startswith(("wing_", "primaries_")),
          "arms": lambda n: n.startswith(("arm_", "forearm_", "hand_")), "head": lambda n: n in ("skull", "snout") or n.startswith("crest"),
          "legs": lambda n: n.startswith(("thigh_", "shin_", "foot_"))}


def fit_poses(spec):
    """The stance's drop (toes on the ground); pecking (the beak at the ground, the toes still on it,
    nothing below them); brooding (breast and toes on the ground, the feathers' tips just at it)."""
    g = lowest(spec, stand(), GROUPS)
    L["drop"] = round(float(g["toes"]), 2)
    # the run's bob: its lowest toe just on the ground
    L["run_lift"] = round(-min(float(lowest(spec, pose_run(t), GROUPS)["toes"]) for t in np.linspace(0, 0.6, 25)), 2)

    def peck_cost(P):
        g = lowest(spec, peck_pose(1.0, dict(L, **P)), GROUPS)
        floor = g["toes"]
        return (g["beak"] - floor - 0.5) ** 2 + 0.001 * (P["peck_n0"] ** 2 + P["peck_n1"] ** 2 + P["peck_head"] ** 2) \
            + 0.004 * P["peck_lean"] ** 2 + 40 * min(0.0, g["wings"] - floor - 1) ** 2 + 40 * min(0.0, g["fan"] - floor) ** 2 \
            + 40 * min(0.0, g["head"] - floor - 1) ** 2 \
            + 0.004 * (P["peck_lean"] + P["peck_n0"] + P["peck_n1"] + P["peck_head"] - 80) ** 2  # the beak points down
    best = fit({"peck_lean": list(range(0, 26, 5)), "peck_n0": list(range(10, 71, 5)), "peck_n1": list(range(0, 91, 10)),
                "peck_head": list(range(-10, 71, 10))}, peck_cost)[1]
    L.update(best)

    def rest_cost(P):
        g = lowest(spec, pose_lie(0.0, P), GROUPS)
        for k in ("beak", "wings", "arms", "head", "fan"):
            g.pop(k, None)
        floor = min(g.values())
        r = {k: v - floor for k, v in g.items()}
        return r["body"] ** 2 + r["toes"] ** 2 + 0.3 * r["tail"] ** 2 + 0.02 * P["rest_pitch"] ** 2 + 0.3 * r["legs"] ** 2
    grid = {"rest_thigh": list(range(-95, -44, 5)), "rest_shin": list(range(110, 181, 5)), "rest_foot": list(range(-160, -79, 10)),
            "rest_pitch": [-12, -9, -6, -3, 0, 3]}  # the tail and breast both down
    best = fit(grid, lambda P: rest_cost(dict(L, **P, rest_drop=0)))[1]
    P = dict(L, **best, rest_drop=0)
    g = lowest(spec, pose_lie(0.0, P), GROUPS)
    floor = min(v for k, v in g.items() if k in ("body", "toes", "tail", "legs"))
    P["rest_drop"] = round(float(floor), 2)

    # The wings rolled out until the feathers' tips just reach the ground, none of them through it.
    def wing_cost(Q):
        g = lowest(spec, pose_lie(0.0, dict(P, **Q)), GROUPS)
        return (g["wings"] - 0.4) ** 2 + 50 * min(0.0, g["arms"] - 1.0) ** 2 - 0.01 * Q["wing_roll"]
    best_w = fit({"wing_roll": list(range(20, 71, 2)), "wing_pitch": list(range(-30, 11, 2))}, wing_cost)[1]
    L.update(best, rest_drop=P["rest_drop"], **best_w)

    # The behaviour clips' pecking (graze, drink, sniff): the beak's tip to the ground, lower than the
    # rest of the head, the legs straight, nothing else near the ground.
    head = {"tip": lambda n: n in ("beak", "jaw"), "head": lambda n: n in ("skull", "snout") or n.startswith("crest")}

    def graze_cost(Q):
        g = lowest(spec, head_down(1.0, dict(L, **Q)), {**GROUPS, **head})
        floor = g["toes"]
        return (g["tip"] - floor - 0.5) ** 2 + 40 * min(0.0, g["head"] - g["tip"] - 1.5) ** 2 \
            + 40 * min(0.0, g["wings"] - floor - 1) ** 2 + 40 * min(0.0, g["fan"] - floor) ** 2 \
            + 0.001 * (Q["graze_n0"] ** 2 + Q["graze_n1"] ** 2 + Q["graze_head"] ** 2) + 0.004 * Q["graze_lean"] ** 2
    best_g = fit({"graze_lean": list(range(0, 51, 5)), "graze_n0": list(range(0, 121, 10)), "graze_n1": list(range(-30, 61, 10)),
                  "graze_head": list(range(-40, 21, 5))}, graze_cost)[1]
    L.update(best_g)
    g = lowest(spec, head_down(1.0), {**GROUPS, **head})
    print("pecking: the beak's tip", round(float(g["tip"] - g["toes"]), 2), "over the ground, the head",
          round(float(g["head"] - g["toes"]), 2), best_g)


FEET = ["toes_L", "toes_R"]


def ground_clips(spec):
    """The clips written as angles, stood on the ground: the approach's height from its feet frame by
    frame, the charge's and the play's once over the clip (keeping their bounce and hops); rearing on
    its planted feet, frame by frame; dying, each frame of the fall on its lowest point (mid-fall a leg
    swings out below where it ends up lying)."""
    A = spec["animations"]
    A["approach"] = grounded(spec, A["approach"], FEET)
    A["charge"] = grounded(spec, A["charge"], FEET, True)
    A["play"] = grounded(spec, A["play"], FEET, True)
    A["rear"] = grounded(spec, A["rear"], FEET)
    A["death"] = grounded(spec, A["death"], [e["name"] for e in spec["elements"]])


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
    sheet, uv = pack(tiles, 256)  # the layers pack at the same width (`Recorder.save`)
    LAYERS["rec"].save(OUT, [k for k, _ in tiles], pack)
    LAYERS["rec"] = None
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "citipati", "texture": [sheet.shape[1], sheet.shape[0]],
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
