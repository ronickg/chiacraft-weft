#!/usr/bin/env python3
"""An ankylosaur: one model worn by two species, *Gastonia burgei* (a polacanthid, the Cretaceous
floodplain's commonest animal) and *Pinacosaurus grangeri* (an ankylosaurid of the Gobi scrubland),
made the way Hytale makes creatures (boxes, a few flat cut-out panels, a hand-painted texture) with
the blocky-model skill, from research (README.md). Its colors are rules (see `area`): no record, so
an earthy olive-brown hide, the bony plates and spikes a paler horn, the underside a pale dun.

    ~/.venvs/blocky/bin/python ankylosaur.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each), y up, it faces +z. An adult is about 5 m long and
1.9 t, the hips about 1.1 m up: here 314 from the beak to the tail's club, the top of the hip shield
82 up; a low, very broad barrel of a body on short legs held under it, a short neck with two bony
collars, a broad, low, triangular head (horns at its back corners), rows of keeled plates over the
back, big flat spikes along the shoulders and flanks pointing out and back, a shield over the hips,
paired flat spikes down the tail and a small club at its end.
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
from house import TX  # noqa: E402

# TX: texels a unit, 0.5 (32 a block, the house density: Ronald's decision, final). The eye's face,
# the skull's side, is 14 by 12 texels at 32 (the house rule wants 3 or more), so it paints at 32:
# the eye is a 3 by 2 design on it. A texel is two units: plates are 5-6 texels across, keels and
# gaps a texel, bands at least 3 texels apart.

OUT = os.environ.get("ANKY_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

SKIN, BACK, BELLY, RIM, PLATE, KEEL, BEAK, MOUTH, CLAW, SPIKE = range(10)
# SKIN the olive-brown hide, BACK its darker brown (the gaps between the back's plates, the brow),
# BELLY the pale dun underside, RIM the dark seams between plates, PLATE the bony plates (paler
# horn), KEEL the ridge along each plate, BEAK dark horn, MOUTH the inside, CLAW the blunt hoof-like
# claws, SPIKE the spikes' pale horn toward their tips. No record of any ankylosaur's colors: these
# are plausible ones for a big plant eater of open floodplain and scrub.
BASE = [(116, 104, 70), (84, 72, 50), (198, 184, 146), (66, 56, 42), (160, 146, 106), (190, 176, 134),
        (124, 112, 92), (110, 56, 52), (80, 70, 58), (212, 198, 158)]
SHADOW = [0.05, 0.04, 0.07, 0.04, 0.07, 0.08, 0.74, 0.95, 0.74, 0.08]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (SKIN, BACK, BELLY, RIM, PLATE, KEEL)
ROLES = [("coat", 0, 0), ("coat", 0, -1.2), ("coat", 1, 0), ("coat", 0.2, -1.6), ("coat", 0.4, 1.0),
         ("coat", 0.5, 1.6), "fixed", "fixed", "fixed", "fixed"]

# At 32 a block the skull's side is 14 texels by 12: a 3 by 2 eye (a catchlight up and forward, a dark
# pupil, an amber-brown iris), small, as an ankylosaur's were. Drawn for the left side (east); the
# right gets it mirrored, so both catchlights point forward. No halo (under 8 texels).
_P, _H, _I, _a = (22, 18, 14), (226, 226, 220), (150, 104, 48), (186, 134, 64)
EYES.update({
    "anky_px3": dict(pattern=["HPI", "IPa"], colors={"P": _P, "H": _H, "I": _I, "a": _a}, halo="none", iris="Ia"),
    "anky_px": dict(pattern=["HP", "IP"], colors={"P": _P, "H": _H, "I": _I}, halo="none", iris="I"),
})
for _k in [k for k in EYES if k.startswith("anky_px")]:
    EYES[_k + "_m"] = dict(EYES[_k], pattern=[row[::-1] for row in EYES[_k]["pattern"]])
EYE_STYLE = os.environ.get("ANKY_EYES", "anky_px3")
SKULL = ((-19.0, 40.0, 94.0), (19.0, 63.0, 122.0))  # broad and low, wider than long
EYE_Z, EYE_Y = 112.0, 56.0  # high on the side, toward the front of the skull
LID_X = 19.4  # the eyelids' hinge plane, just outside the head's side

# ------------------------------------------------------------------ the model

NECK_BASE = np.array([0.0, 56.0, 68.0])  # inside the chest's front
NECK_ANGLE, NECK_LEN = -8.0, 34.0  # short, sloping a little down to a head held low
NECK_HW, NECK_HH = 14.0, 9.0
# Tail segments: (length, half width, centre height, half height), each thinner than the one before
# and reaching 4 units into it, so it can swing without a corner showing.
TAIL = [(36, 20.0, 60.0, 14.0), (34, 15.0, 58.5, 10.5), (32, 11.0, 57.0, 7.5), (28, 7.5, 55.5, 5.0)]
TAIL0_Z = -40.0
DEEP = 4.0


def neck_end():
    """The head's joint: the far end of the neck, inside the skull."""
    a = math.radians(NECK_ANGLE)
    return NECK_BASE + NECK_LEN * np.array([0.0, math.sin(a), math.cos(a)])


def tail_pivots():
    pts, z = [], TAIL0_Z
    for length, hw, cy, hh in TAIL:
        pts.append(np.array([0.0, cy, z]))
        z -= length
    return pts


def el(name, bone, lo, hi, quad=None, rot=None, origin=None, cut=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad,
                rot=[float(v) for v in rot] if rot is not None else None,
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
FRONT_X, FRONT_Z, BACK_X, BACK_Z = 38.0, 56.0, 42.0, -22.0


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 54, -10)), ("spine_1", "spine_0", (0, 52, 20)),
         ("neck", "spine_1", tuple(NECK_BASE)), ("head", "neck", tuple(neck_end())), ("jaw", "head", (0, 42, 100))]
    pts = tail_pivots()
    B += [(f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", tuple(pts[k])) for k in range(len(TAIL))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B.append((f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2)))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x = FRONT_X * s
            B += [(f"leg_{leg}_upper", "spine_1", (x, 46, FRONT_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 22, FRONT_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 8, FRONT_Z + 1))]
        else:
            x = BACK_X * s
            B += [(f"leg_{leg}_upper", "spine_0", (x, 52, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 26, BACK_Z + 1)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 10, BACK_Z + 1))]
    return B


def spike_box(name, bone, base, rot, w0, len0, w1, len1):
    """A tall spike as two boxes tapering to a point, tilted out and back from its base on the body."""
    x, y, z = base
    return [el(f"{name}_base", bone, (x - w0 / 2, y - 4, z - w0 / 2), (x + w0 / 2, y + len0, z + w0 / 2), rot=rot, origin=base),
            el(f"{name}_tip", bone, (x - w1 / 2, y + len0 - 2, z - w1 / 2), (x + w1 / 2, y + len0 + len1, z + w1 / 2), rot=rot,
               origin=base)]


def flat_spike(name, bone, side, x_body, y, zc, out, length, droop):
    """A big flat triangular spike (Gastonia's are flat, grooved behind): a cut-out panel lying out
    from the body's side, its tip out and back, drooping with the flank."""
    x0, x1 = sorted((side * (x_body - 4), side * (x_body + out)))
    return el(name, bone, (x0, y, zc - length / 2), (x1, y, zc + length / 2), quad="up", rot=(0, 0, -side * droop),
              origin=(side * x_body, y, zc), cut="spike")


def elements():
    E = [el("hips", "spine_0", (-54, 28, -46), (54, 76, 22)),  # the widest: a very broad, flat rump
         # the sacral shield over the hips, reaching just past their sides and back (at 51 and -44 their top
         # showed round it as a band a texel and a half wide; linecheck)
         el("shield", "spine_0", (-54.55, 75, -46.55), (54.55, 80.5, 16)),
         el("ribs_0", "spine_0", (-58.5, 38, -40), (58.5, 64, 14)),  # the flanks bulging out: a barrel, not a crate
         el("belly_0", "spine_0", (-46, 22, -38), (46, 32, 18)),  # the belly bulging down between the legs
         el("chest", "spine_1", (-50, 31, 16), (50, 66, 74)),
         el("backplate", "spine_1", (-42, 64, 18), (42, 70.5, 70)),  # the arched back over the ribs
         el("belly_1", "spine_1", (-40, 25, 20), (40, 34, 64)),
         el("ribs_1", "spine_1", (-53.2, 39, 22), (53.2, 60, 68)),
         # The neck: a box from inside the chest out along its slope to the head's joint, deep in
         # the skull, so they share a point and never come apart (`ground.pokes`).
         el("neck", "neck", NECK_BASE + np.array([-NECK_HW, -NECK_HH, -1.5]), NECK_BASE + np.array([NECK_HW, NECK_HH, NECK_LEN + 0.5]),
            rot=(-NECK_ANGLE, 0, 0), origin=tuple(NECK_BASE))]
    # Two bony collars (half rings of keeled plates) over the neck, tilted with it.
    for k, (z0, z1) in enumerate(((6.0, 13.0), (15.5, 22.5))):
        E.append(el(f"collar_{k}", "neck", NECK_BASE + np.array([-17.5 + k, -2.0, z0]), NECK_BASE + np.array([17.5 - k, 12.5 - 0.5 * k, z1]),
                    rot=(-NECK_ANGLE, 0, 0), origin=tuple(NECK_BASE)))
    E += [el("skull", "head", *SKULL),
          el("snout", "head", (-12, 42, 120), (12, 57, 132)),  # narrowing to the beak: a triangle from above
          el("beak", "head", (-6, 41.2, 130), (6, 49.5, 135.5)),
          el("mouth", "head", (-9, 42.6, 104), (9, 45, 130)),
          el("jaw", "jaw", (-11.5, 36, 98), (11.5, 43.5, 128))]  # shorter than the upper beak
    for s, n in ((1, "L"), (-1, "R")):
        x0, x1 = sorted((s * 16.5, s * 25.0))
        E.append(el(f"horn_{n}", "head", (x0, 52.5, 94.6), (x1, 60.5, 101.5)))  # the squamosal horn, at the skull's back corner
        x0, x1 = sorted((s * 17.0, s * 22.0))
        E.append(el(f"cheek_{n}", "head", (x0, 43.5, 100.5), (x1, 49.5, 106)))  # the jugal horn, under and behind the eye
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        lx0, lx1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down"))
    pts = tail_pivots()
    for k, (length, hw, cy, hh) in enumerate(TAIL):
        z = pts[k][2]
        E.append(el(f"tail_{k}", f"tail_{k}", (-hw, cy - hh, z - length), (hw, cy + hh, z + DEEP)))
    E.append(el("club", "tail_3", (-11, 49.5, -178), (11, 60, -164)))  # a small club (Pinacosaurus's), low and wide
    # The shoulder spikes: the biggest, rising out, up and back; then flat ones along the flanks,
    # smaller toward the hips; flat triangles in pairs down the tail.
    for s, n in ((1, "L"), (-1, "R")):
        E += spike_box(f"spike_{n}0", "spine_1", (s * 44.0, 63.0, 64.0), (-30, 0, -s * 46), 9, 14, 5, 12)
        E += spike_box(f"spike_{n}1", "spine_1", (s * 48.0, 59.0, 47.0), (-38, 0, -s * 58), 9, 15, 5, 13)
        for k, (bone, xb, y, zc, out, length) in enumerate((("spine_1", 53.2, 51, 30, 30, 22), ("spine_0", 58.5, 52, 6, 27, 20),
                                                           ("spine_0", 58.5, 52, -16, 23, 18), ("spine_0", 58.5, 52, -34, 18, 14))):
            E.append(flat_spike(f"flank_{n}{k}", bone, s, xb, y, zc, out, length, 36))
        for k, (length, hw, cy, hh) in enumerate(TAIL):
            z = pts[k][2]
            ln = min(0.7 * length, 22)
            out = (18, 15, 12, 8)[k]
            E.append(flat_spike(f"tspike_{n}{k}", f"tail_{k}", s, hw, cy, z - length + ln / 2 + 2, out, ln, 12))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z = FRONT_X * s, FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 8, 20, z - 7), (x + 8, 50, z + 7)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 7, 7, z - 6), (x + 7, 23.5, z + 6)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 9, 0, z - 8), (x + 9, 9, z + 10)))
        else:
            x, z = BACK_X * s, BACK_Z
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (x - 11, 24, z - 11), (x + 11, 60, z + 11)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 9, 9, z - 7), (x + 9, 27.5, z + 9)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 10.5, 0, z - 9), (x + 10.5, 10.5, z + 14)))
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


def surface(face, p):
    """(a, b): the face's two coordinates, b the one along which rows of plates are laid."""
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    return {"east": (z, y), "west": (z, y), "south": (x, y), "north": (x, y), "up": (x, z), "down": (x, z)}[face]


def scutes(a, b, sa, sb, fill=0.8, keel=None, power=4.0, fill_b=None):
    """Rows of rounded plates (spacing sa along a, sb along b, every other row offset by half; `fill`
    the share of a cell a plate takes along a, `fill_b` along b): which texels are plate, and which
    the keel along a plate's middle (`keel` "a": a line running along b, narrow in a; "b": running
    along a)."""
    j = np.floor(b / sb)
    a2 = a / sa + 0.5 * (j % 2)
    fa, fb = a2 - np.floor(a2) - 0.5, b / sb - j - 0.5
    ra, rb = fill / 2, (fill if fill_b is None else fill_b) / 2
    plate = (np.abs(fa) / ra) ** power + (np.abs(fb) / rb) ** power < 1
    if keel == "a":
        k = plate & (np.abs(fa * sa) < 1.0) & (np.abs(fb) < 0.75 * rb)
    elif keel == "b":
        k = plate & (np.abs(fb * sb) < 1.0) & (np.abs(fa) < 0.75 * ra)
    else:
        k = np.zeros_like(plate)
    return plate, k


def mosaic(p, cell, seed=0, seam=1.1):
    """A mosaic of fused bony tiles (the hip shield, the skull's tiles): cells round jittered points
    in 3D, so it runs on over a box's edges. Returns (seam texels, a boss in some cells' middles)."""
    g = np.floor(p / cell)
    d1 = np.full(p.shape[:-1], np.inf)
    d2 = np.full(p.shape[:-1], np.inf)
    h1 = np.zeros(p.shape[:-1])
    for ox in (-1, 0, 1):
        for oy in (-1, 0, 1):
            for oz in (-1, 0, 1):
                c = g + np.array([ox, oy, oz])
                hsh = np.sin(c @ np.array([12.9898, 78.233, 37.719]) + seed) * 43758.5453
                hsh = hsh - np.floor(hsh)
                hsh2 = np.sin(c @ np.array([39.346, 11.135, 83.155]) + seed) * 24634.6345
                hsh2 = hsh2 - np.floor(hsh2)
                q = (c + 0.2 + 0.6 * np.stack([hsh, hsh2, (hsh + hsh2) % 1.0], -1)) * cell
                d = np.linalg.norm(p - q, axis=-1)
                closer = d < d1
                d2 = np.where(closer, d1, np.minimum(d2, d))
                h1 = np.where(closer, hsh, h1)
                d1 = np.where(closer, d, d1)
    return (d2 - d1) < seam, (d1 < 1.3) & (h1 < 0.35)


def local(e, p):
    """A point back in the element's own unrotated frame."""
    if not e["rot"]:
        return p
    o = np.array(e["origin"])
    return (p - o) @ rotm(e["rot"]) + o


def area(e, face, p, uu, vv, h):
    """Olive-brown above, a pale dun below; rows of keeled plates over the back and down the flanks
    (bigger on the back, smaller lower down), a mosaic of small plates in the hip shield, tiles on
    the skull; the spikes pale horn toward their tips, the beak dark horn."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, SKIN)
    side = face in ("east", "west")
    t = vv / max(h, 1)
    sa, sb = surface(face, p)

    def put(mask, keel, base=None):
        nonlocal a
        if base is not None:
            a = np.where(mask, base, a)
        a = np.where(mask, PLATE, a)
        a = np.where(keel, KEEL, a)

    if name in ("hips", "chest", "backplate", "ribs_0", "ribs_1"):
        if face == "up":
            # transverse rows of big oval plates, a keel down each, darker skin between the rows
            a[:] = BACK
            put(*scutes(sa, sb, 14, 16, 0.8, "a", 3.0, fill_b=0.56))
        elif face == "down":
            a[:] = BELLY
        elif side:
            lo, hi = e["lo"][1], e["hi"][1]
            a = np.where(y < 36, BELLY, SKIN)
            a = np.where(y > 66, BACK, a)
            pl, kl = scutes(sa, sb, 13, 11, 0.8, "b", 2.2, fill_b=0.66)
            put(pl & (y > 46), kl & (y > 46))
            pl, _ = scutes(sa, sb, 8, 8, 0.6, None, 3.0)
            put(pl & (y <= 46) & (y > 38), pl & False)
        else:
            a = np.where(y < 36, BELLY, np.where(y > 66, BACK, SKIN))
            pl, kl = scutes(sa, sb, 12, 11, 0.76, "b", 3.0, fill_b=0.62)
            put(pl & (y > 46), kl & (y > 46))
    elif name == "shield":
        # The polacanthid shield: small plates fused into one mosaic, a boss in some of them.
        seam, _ = mosaic(p, 12.0, 3, 1.0)
        a = np.where(seam, BACK, PLATE)
        if face not in ("up", "down"):
            a = np.where(t > 0.7, BACK, a)
    elif name.startswith("belly"):
        a[:] = BELLY
        if side or face in ("south", "north"):
            a = np.where(t < 0.3, SKIN, BELLY)
    elif name == "neck":
        if face == "down":
            a[:] = BELLY
        elif face == "up":
            a[:] = SKIN
        else:
            a = np.where(t > 0.62, BELLY, np.where(t < 0.3, BACK, SKIN))
    elif name.startswith("collar"):
        if face == "down":
            a[:] = BELLY
        else:
            q = local(e, p)
            lo = np.array(e["lo"])
            # one ring of plates across the neck: a keel along each, seams between them
            across = q[..., 0] if face in ("up", "south", "north") else q[..., 1]
            seam = np.mod(across - lo[0 if face in ("up", "south", "north") else 1] + 3.0, 7.0) < 1.6
            a = np.where(seam, RIM, PLATE)
            if face == "up":
                mid = np.abs(q[..., 2] - (e["lo"][2] + e["hi"][2]) / 2) < 1.2
                a = np.where(mid & ~seam, KEEL, a)
            if side:
                a = np.where(t > 0.7, SKIN, a)
    elif name == "skull":
        if face == "up":
            seam, _ = mosaic(p, 11.0, 5, 1.0)  # the head's bony tiles (caputegulae)
            a = np.where(seam, BACK, PLATE)
        elif face == "down":
            a[:] = BELLY
        elif side:
            a = np.where(t < 0.3, BACK, SKIN)
        else:
            a = np.where(t < 0.3, BACK, SKIN)
    elif name == "snout":
        if face == "up":
            seam, _ = mosaic(p, 11.0, 5, 1.0)
            a = np.where(seam, BACK, PLATE)
        elif face == "down":
            a[:] = MOUTH
        else:
            a = np.where(t < 0.25, BACK, SKIN)
    elif name == "beak":
        a[:] = BEAK
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else np.where(z > 124.5, BEAK, BELLY)
        if side:
            a = np.where((t < 0.5) & (z <= 124.5), SKIN, a)
    elif name == "mouth":
        a[:] = MOUTH
    elif name.startswith("horn"):
        a[:] = SPIKE if face != "up" else PLATE
    elif name.startswith("cheek"):
        a[:] = PLATE
    elif name.startswith("spike_"):
        q = local(e, p)
        ry = q[..., 1] - e["origin"][1]
        a = np.where(ry > 9 if name.endswith("_base") else ry > -100, SPIKE, PLATE)
    elif name.startswith(("flank_", "tspike_")):
        q = local(e, p)
        s = 1 if e["lo"][0] + e["hi"][0] > 0 else -1
        f = (s * q[..., 0] - abs(e["origin"][0])) / max(abs(e["hi"][0] - e["lo"][0]) - 4, 1)
        a = np.where(f < 0.3, PLATE, SPIKE)
    elif name.startswith("tail_"):
        if face == "down":
            a[:] = BELLY
        elif face == "up":
            a[:] = BACK
            pl, kl = scutes(sa, sb, 10, 10, 0.8, "a")  # oval plates on top
            put(pl, kl)
        elif side:
            a = np.where(t > 0.72, BELLY, np.where(t < 0.3, BACK, SKIN))
            pl, kl = scutes(sa, sb, 9, 8, 0.7, "b")
            put(pl & (t < 0.55), kl & (t < 0.55))
        else:
            a = np.where(t > 0.72, BELLY, np.where(t < 0.3, BACK, SKIN))
    elif name == "club":
        a[:] = RIM
        pl, kl = scutes(sa, sb, 10, 8, 0.85, "a", power=3.0)
        a = np.where(pl, PLATE, a)
        a = np.where(kl, KEEL, a)
    elif name.startswith("thigh_"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        else:
            a = np.where(t > 0.85, BELLY, SKIN)
            pl, _ = scutes(sa, sb, 8, 8, 0.6)  # small round scutes on the thigh
            a = np.where(pl & (t < 0.7), PLATE, a)
    elif name.startswith("leg_"):
        if name.endswith("_paw"):
            a = np.where((face == "south") & (y < 3.2), CLAW, SKIN)
            if face == "down":
                a[:] = BELLY
        elif side and ((e["lo"][0] > 0) == (face == "west")):
            a[:] = BELLY  # the inside of the legs
        elif name.endswith("_upper") and not side and face not in ("up", "down"):
            pl, _ = scutes(sa, sb, 8, 8, 0.6)
            a = np.where(pl & (t < 0.6), PLATE, a)
    return a


def cut_mask(e, p):
    """Which texels of a flat spike are there: a triangle from a broad base on the body to a point
    out and back, the front edge sloping, the back one nearly square to the body."""
    q = local(e, p)
    s = 1 if e["lo"][0] + e["hi"][0] > 0 else -1
    x_in = min(abs(e["lo"][0]), abs(e["hi"][0]))
    x_out = max(abs(e["lo"][0]), abs(e["hi"][0]))
    f = np.clip((s * q[..., 0] - x_in) / (x_out - x_in), 0, 1)
    z0, z1 = e["lo"][2], e["hi"][2]
    zf = z1 - f * (z1 - z0)
    zb = z0 + 0.06 * (1 - f) * (z1 - z0)  # the back edge nearly straight out: a broad blade
    return (q[..., 2] >= zb - 0.5) & (q[..., 2] <= zf + 0.5)


LASH = (44, 36, 28)

# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def layer_props(e, face, p, a):
    """White where this one is pale (the belly); marks over the back, flanks and tail."""
    n, y = e["name"], p[..., 1]
    white = np.where(a == BELLY, 0.85, 0.15)
    ext = np.where(np.isin(a, (BEAK, CLAW)), 1.0, 0.0)
    body = n in ("hips", "chest", "backplate", "neck", "ribs_0", "ribs_1", "shield") or n.startswith(("tail_", "thigh_"))
    pattern = np.full(y.shape, 1.0 if body else 0.3)
    return {"white": white, "extremity": ext, "pattern": pattern}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    size = np.array(e["hi"]) - np.array(e["lo"])
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel centers, in texels
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel (texel centres scaled to units)
    p = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    light_face = face
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "jaw", "tail", "club")):
        light_face = "north"  # seen when a pose lifts or folds it
    if e["name"].startswith("eyelid"):
        light_face = "east"  # closed, it hangs on the skull's side: lit as that side is
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=False, merge=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"]:
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "skull" and face in ("east", "west"):
        side = 1 if face == "east" else -1
        stamp_eyes(img, [eye_center(side)], eye_style(side))
    if e["name"] == "snout" and face in ("east", "west"):  # the nostril: a slit, a texel high, near the front
        r = 2
        c = 1 if face == "east" else w - 3
        img[r, c:c + 2, :3] = (img[r, c:c + 2, :3] * 0.55).astype(np.uint8)
    if e["name"].startswith("eyelid"):  # a closed eye: the lash along the edge that ends up at the bottom
        col_idx = 0 if e["name"].endswith("L") else w - 1
        img[:, col_idx, :3] = LASH
    if e["name"].endswith("_paw") and face == "south" and w >= 7:  # toes: grooves on the front, 2 texels apart
        for c in range(2, w - 1, 3):
            img[:, c, :3] = (img[:, c, :3] * 0.74).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"lie_front": 80, "lie_hip": -70, "lie_knee": 140, "lie_drop": 20, "lie_tail": -4, "lie_neck": 10, "lie_head": 4,
     "graze_neck": 20, "graze_head": 20, "graze_lean": 4}


def gait(p, phases, swing_amp, lift):
    """A foot swings forward while it's lifted, back while it's down; the knees bend, the feet kept
    about flat."""
    out = {}
    for leg, ph in zip(LEGS, phases):
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))
        swing = -swing_amp * S(a)
        knee = lift * up
        upper = swing - (0.3 if leg[0] == "F" else 0.4) * knee  # the thigh drawn forward as the knee bends
        out[f"leg_{leg}_upper"], out[f"leg_{leg}_lower"] = (upper, 0, 0), (knee, 0, 0)
        out[f"leg_{leg}_paw"] = (-(upper + knee), 0, 0)  # the foot kept flat
    return out


def chew(t, rate=2.0, amp=5.0):
    return {"jaw": (amp * max(0.0, S(TAU * t * rate)), 0, 0)}


def graze_pose(g, P=L):
    """Neck and head down to the low plants by `g` (0..1), the chest a little lower."""
    q = {"spine_1": (P["graze_lean"] * g, 0, 0), "neck": (P["graze_neck"] * g, 0, 0), "head": (P["graze_head"] * g, 0, 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (-P["graze_lean"] * g, 0, 0)
    return q


def tail_sway(t, amp, rate, pitch=0.0):
    return {f"tail_{k}": (pitch + (1.0 if k else 0.0), amp * (0.6 + 0.4 * k) * S(TAU * (t * rate - 0.12 * k)), 0)
            for k in range(len(TAIL))}


def pose_idle(t):
    """Cropping low plants and chewing, then the head up to look about, the tail swaying (8 s)."""
    p = t / 8.0
    br = S(TAU * p * 3)
    g = float(np.interp(p, [0, .45, .55, .84, .94, 1], [1, 1, 0, 0, 1, 1]))
    look = (1 - g) * 18 * S(TAU * (p - 0.55) * 1.6)
    q = graze_pose(g)
    q.update({"root": (0, 0.15 * (1 + br), 0)})  # breathing lifts it, never sinks it
    q["neck"] = (q["neck"][0] - 6 * (1 - g), 0, 0)  # up to look about
    q["head"] = (q["head"][0] - 4 * (1 - g), look, 0)
    q.update(tail_sway(t, 3, 0.25))
    q.update(chew(t, 2.0 if g > 0.5 else 0.8, 5 if g > 0.5 else 1.2))
    return q


def pose_walk(t):
    p = t / 1.4
    q = gait(p, (0.0, 0.5, 0.75, 0.25), 14, 36)
    q.update({"root": (0, 0.6 * abs(S(TAU * p * 2)), 0), "spine_0": (0, 1.5 * S(TAU * p), 0),
              "spine_1": (0, -1.5 * S(TAU * p), 0), "head": (-1.5 * S(TAU * p * 2), -2 * S(TAU * p), 0)})
    q.update(tail_sway(t, 6, 1 / 1.4))
    return q


def pose_run(t):
    """A heavy trot, the head held low and forward (0.8 s)."""
    p = t / 0.8
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 20, 50)
    q.update({"root": (0, 1.6 * abs(S(TAU * p)), 0), "neck": (4, 0, 0), "head": (-3, 0, 0),
              "spine_1": (1.2 * S(TAU * p * 2), 0, 0)})
    q.update(tail_sway(t, 7, 1 / 0.8, pitch=2.0))
    return q


def pose_lie(t, P=L):
    """Lying on its belly, the legs folded under (as the juvenile Pinacosaurus were found), the
    tail and chin on the ground (6 s)."""
    p = t / 6.0
    br = S(TAU * p * 2)
    q = {"root": (0, -P["lie_drop"] + 0.15 * (1 + br), 0), "spine_1": (0.6 * br, 0, 0), "neck": (P["lie_neck"], 0, 0),
         "head": (P["lie_head"], 5 * S(TAU * p), 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (P["lie_front"], 0, 0)
        q[f"leg_{leg}_lower"] = (-150, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_front"] + 150 - 40, 0, 0)
    for leg, s in (("BL", 1), ("BR", -1)):
        q[f"leg_{leg}_upper"] = (P["lie_hip"], 0, s * 6)
        q[f"leg_{leg}_lower"] = (P["lie_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_hip"] - P["lie_knee"], 0, 0)
    q.update(tail_sway(t, 2, 1 / 6.0))
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = (P["lie_tail"] if k == 0 else 0.5 * P["lie_tail"], q[f"tail_{k}"][1], 0)
    return q


BLINKS = {"idle": [2.0, 5.4], "lie": [1.5, 4.4]}
BLINK = 0.12
ANIMS = {"idle": (pose_idle, 8.0), "walk": (pose_walk, 1.4), "run": (pose_run, 0.8), "lie": (pose_lie, 6.0)}


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
          "body": lambda n: n in ("hips", "chest") or n.startswith("belly"), "muzzle": lambda n: n in ("beak", "jaw", "snout"),
          "legs": lambda n: n.startswith(("leg_", "thigh_")), "tail": lambda n: n.startswith(("tail_", "club")),
          "spikes": lambda n: n.startswith(("spike_", "flank_", "tspike_", "cheek", "horn")),
          "head": lambda n: n in ("skull",)}


def fit_poses(spec):
    """Lying (the belly on the ground, the folded legs under it, the tail and chin down, nothing
    below) and cropping (the beak at the plants, the front feet still on the ground), by forward
    kinematics."""
    def lie_cost(P):
        g = lowest(spec, pose_lie(0.0, dict(L, **P, lie_drop=0)), GROUPS)
        f = min(g["body"], g["legs"])
        return (g["body"] - f - 0.8) ** 2 + 0.3 * (g["front paws"] - f) ** 2 + 0.3 * (g["back paws"] - f) ** 2
    grid = {"lie_front": list(range(20, 101, 5)), "lie_hip": list(range(-110, -24, 5)), "lie_knee": list(range(60, 166, 5))}
    best = fit(grid, lie_cost)[1]
    g = lowest(spec, pose_lie(0.0, dict(L, **best, lie_drop=0)), GROUPS)
    L.update(best, lie_drop=round(float(min(g["body"], g["legs"])), 2))

    def tail_cost(P):  # the tail's club down to the ground, nothing through it
        g = lowest(spec, pose_lie(0.0, dict(L, **P)), GROUPS)
        return (g["tail"] - 0.6) ** 2 + 50 * min(0.0, g["spikes"]) ** 2
    L.update(fit({"lie_tail": [x / 2 for x in range(-40, 11)]}, tail_cost)[1])

    def chin_cost(P):  # the chin resting on the ground
        g = lowest(spec, pose_lie(0.0, dict(L, **P)), GROUPS)
        return (g["muzzle"] - 1.0) ** 2 + 50 * min(0.0, g["spikes"]) ** 2 + 50 * min(0.0, g["head"]) ** 2 \
            + 0.001 * (P["lie_neck"] ** 2 + P["lie_head"] ** 2)
    L.update(fit({"lie_neck": list(range(-10, 41, 2)), "lie_head": list(range(-20, 31, 2))}, chin_cost)[1])

    def graze_cost(P):
        g = lowest(spec, graze_pose(1.0, dict(L, **P)), GROUPS)
        return (g["muzzle"] - 0.5) ** 2 + 0.002 * (P["graze_neck"] ** 2 + P["graze_head"] ** 2) + 0.05 * P["graze_lean"] ** 2 \
            + 40 * min(0.0, g["front paws"]) ** 2 + 40 * min(0.0, g["spikes"]) ** 2
    best = fit({"graze_neck": list(range(0, 61, 2)), "graze_head": list(range(-10, 61, 2)), "graze_lean": [0, 2, 4, 6, 8, 10]},
               graze_cost)[1]
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
    spec = {"name": "ankylosaur", "texture": [sheet.shape[1], sheet.shape[0]],
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
