#!/usr/bin/env python3
"""The raptor: a feathered Velociraptor the way it really looked (quill knobs prove its wing
feathers), made the way Hytale makes creatures (boxes and a few flat feather panels, a
hand-painted texture) with the blocky-model skill. No concept sheet: its colors are rules
(see `area`), its proportions come from the research in README.md.

    ../dog/chihuahua/.venv/bin/python raptor.py      # writes out/spec.json, out/texture.png

Units are its own (exported at 0.3 model pixels a unit, `build.SCALE`: 53 a block), y up, the
raptor faces +z. Painted at the house density, 32 texels a block (`TX` texels a unit), as the
blocks and items are. A real Velociraptor is about
50 cm at the hip and 1.8-2 m long, half of it tail; here the hip is about 29 units up after the
crouch, the body 23 long, the tail 44, the head 1.2x (Hytale's chunky look).
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
from paint import EYES, eye_corner, pack, paint_face_smooth, occlusion, stamp_eyes, tones  # noqa: E402
from rig import fit, swing  # noqa: E402
from ground import lowest, world  # noqa: E402
from house import texels  # noqa: E402

TX = texels(0.3)  # texels a unit: the house's 32 a block at the raptor's export scale (build.SCALE)

OUT = os.environ.get("RAPTOR_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

BACK, SIDE, BELLY, BAR, CREST, SCALE, TEETH, DARK, CLAW, TIP = range(10)
BASE = [(150, 84, 48), (196, 150, 98), (228, 210, 174), (92, 50, 34), (214, 92, 48), (150, 124, 96),
        (236, 232, 216), (70, 30, 34), (58, 46, 44), (60, 44, 40)]
SHADOW = [0.02, 0.04, 0.06, 0.0, 0.0, 0.05, 0.1, 0.95, 0.7, 0.0]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
# The second species, Velociraptor osmolskae, on the same model: it lived among the dunes of
# Bayan Mandahu, so a paler sand coat with faint bars and a dull ochre crest (its colors, like
# every raptor's, are a guess from living birds of open, dry country).
OSMOLSKAE = [(196, 168, 120), (220, 198, 156), (240, 230, 204), (156, 124, 84), (176, 132, 82), (168, 150, 122),
             (236, 232, 216), (70, 30, 34), (58, 46, 44), (140, 110, 76)]


def set_palette(base):
    """Paint with other colors (same pattern and layout)."""
    global TONES
    TONES = [tones(c, h) for c, h in zip(base, SHADOW)]


FUR = (BACK, SIDE, BELLY)  # the soft feather coat; bars, crest, teeth, claws stay crisp
ROLES = [("coat", 0, 0), ("coat", 0.5, 0), ("coat", 1, 0), ("coat", 0, 0), "fixed", "fixed",
         "fixed", "fixed", "fixed", ("coat", 0, -1.5)]  # BACK SIDE BELLY BAR CREST SCALE TEETH DARK CLAW TIP

# A hawk's eye: yellow round a black pupil, a catchlight; on the sides of the head.
EYES.update({"raptor": dict(pattern=[".OOO.",
                                     "OHPYO",
                                     "OYPYO",
                                     ".OOO."], colors={"O": (60, 36, 28), "H": (240, 238, 226), "P": (14, 10, 12),
                                                      "Y": (232, 186, 58)}, halo="round")})
# At 32 a block the head's side is 7 texels by 4: the eye is the old one's middle, 3 by 2 (a
# catchlight up and forward, the black pupil, yellow round it), the dark stripe through the eye
# its outline. Drawn for the right side (east, u running back from the snout); the left side
# gets it mirrored, so the catchlight is forward on both.
EYES.update({"raptor_px": dict(pattern=["HPY", "YPY"], colors={"H": (240, 238, 226), "P": (14, 10, 12),
                                                                "Y": (232, 186, 58)}, halo="none", iris="Y"),
             "raptor_px_round": dict(pattern=["HPY", "YYY"], colors={"H": (240, 238, 226), "P": (14, 10, 12),
                                                                      "Y": (232, 186, 58)}, halo="none", iris="Y"),
             "raptor_px2": dict(pattern=["HP", "YY"], colors={"H": (240, 238, 226), "P": (14, 10, 12),
                                                               "Y": (232, 186, 58)}, halo="none", iris="Y")})
for _k in [k for k in EYES if k.startswith("raptor")]:
    EYES[_k + "_m"] = dict(EYES[_k], pattern=[row[::-1] for row in EYES[_k]["pattern"]])
EYE_STYLE = os.environ.get("RAPTOR_EYES", "raptor_px")
EYE_Z, EYE_Y = 25.0, 39.8  # the eye's middle on the side of the head
SKULL = ((-4, 36, 18), (4, 43, 29))
LID_X = 4.4  # the eyelids' hinge plane, just outside the head's side (x = 4)

# ------------------------------------------------------------------ the model

TAIL = [(10, 8.0), (10, 6.4), (9, 5.0), (8, 3.8), (7, 3.0)]  # length, width (0.6+ a side thinner each)
# The tip flush in the one before (0.4 a side) and two texels tall (at 2.8, its sides one texel: a line; linecheck).
TAIL0 = np.array([0.0, 31.0, -5.5])


NECK_BASE = np.array([0.0, 31.0, 14.0])  # inside the chest's front top
NECK_ANGLE, NECK_LEN = 50.0, 11.0  # rising forward, its end deep in the skull


def neck_end():
    """The head's joint: the far end of the neck, inside the skull."""
    a = math.radians(NECK_ANGLE)
    return NECK_BASE + NECK_LEN * np.array([0.0, math.sin(a), math.cos(a)])


def tail_points():
    pts, p = [TAIL0.copy()], TAIL0.copy()
    for length, _ in TAIL:
        p = p + np.array([0, 0, -length])
        pts.append(p.copy())
    return pts


def el(name, bone, lo, hi, quad=None, rot=None, origin=None, cut=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad, rot=rot,
                origin=[float(v) for v in origin] if origin else None, cut=cut)


def skull_grid():
    """The skull side's texels: (width, height) and the units a texel spans (u along z, v)."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h = max(1, round(TX * (z1 - z0))), max(1, round(TX * (y1 - y0)))
    return w, h, (z1 - z0) / w, (y1 - y0) / h


def eye_corner_side(side):
    """The eye's top-left texel on one side of the skull (east faces run u along -z, west along
    +z), as `stamp_eyes` places it."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    cu = ((z1 - EYE_Z) if side > 0 else (EYE_Z - z0)) / su
    return eye_corner(cu, (y1 - EYE_Y) / sv, EYES[EYE_STYLE]["pattern"], w)


def eye_box(side):
    """(z0, z1, top, bottom) of the eye on one side, in units, rounded as `stamp_eyes` places it,
    so the eyelid covers it to the texel."""
    pat = EYES[EYE_STYLE]["pattern"]
    ph, pw = len(pat), len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    u0, v0 = eye_corner_side(side)
    za, zb = (z1 - (u0 + pw) * su, z1 - u0 * su) if side > 0 else (z0 + u0 * su, z0 + (u0 + pw) * su)
    return za, zb, y1 - v0 * sv, y1 - (v0 + ph) * sv


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 30, 0)), ("spine_1", "spine_0", (0, 31, 5.5)),
         ("neck", "spine_1", tuple(NECK_BASE)), ("head", "neck", tuple(neck_end())), ("jaw", "head", (0, 37.5, 28.5))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B += [(f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2)),
              (f"arm_{n}", "spine_1", (s * 6.0, 29.5, 13.5)), (f"forearm_{n}", f"arm_{n}", (s * 6.75, 24.5, 13.5))]
    pts = tail_points()
    B += [(f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", tuple(pts[k])) for k in range(len(TAIL))]
    for s, n in ((1, "L"), (-1, "R")):
        x = 4.5 * s
        B += [(f"leg_{n}_thigh", "spine_0", (x, 28, 0)), (f"leg_{n}_shin", f"leg_{n}_thigh", (x, 19, 0)),
              (f"leg_{n}_foot", f"leg_{n}_shin", (x, 8.5, -0.5)), (f"leg_{n}_toes", f"leg_{n}_foot", (x, 2.2, -0.5))]
    return B


def elements():
    E = [el("hips", "spine_0", (-5.5, 25, -6), (5.5, 35, 6)),
         el("chest", "spine_1", (-6, 24.5, 5.5), (6, 34.5, 17)),
         # The neck: a box from inside the chest out along the neck's angle to the head's joint,
         # which sits inside the skull, so head and neck share a point and never come apart; slimmer
         # than the skull and ending deep in it, so no corner of it pokes out of the head when the
         # head turns (`ground.pokes`).
         el("neck", "neck", NECK_BASE + np.array([-2.6, -2.6, -1.5]), NECK_BASE + np.array([2.6, 2.6, NECK_LEN + 1.5]),
            rot=(-NECK_ANGLE, 0, 0), origin=tuple(NECK_BASE)),
         el("skull", "head", *SKULL),
         el("snout", "head", (-3, 38, 28.5), (3, 42, 38), rot=(-6, 0, 0), origin=(0, 40, 28.5)),
         # To 0.4 short of the jaw's front (at 35.5, 1.5 units of the jaw's top showed before it as a lip; linecheck).
         el("mouth", "head", (-2, 37, 29), (2, 39.4, 36.6)),
         el("jaw", "jaw", (-2.5, 35.5, 28.5), (2.5, 38, 37)),
         # A crest of feathers along the top of the head and neck: one cut-out panel.
         el("crest", "head", (0, 41, 13), (0, 48, 29), quad="east", cut="crest")]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        x0, x1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (x0, top, za), (x1, top, zb), quad="down"))
        ax = 6.75 * s  # the shoulder (x = 6) inside both the chest and the arm
        # Out over two texels from the chest's side (at ax + 1.25, 1.2 texels of its front showed beside the chest;
        # linecheck), and the forearm two texels across (rule 7: at 1.8 units a box a texel thick, a line along
        # the arm), its sides flush: 0.35 out from the chest's, 0.4 in from the arm's.
        a0, a1 = sorted((ax - 1.25 * s, ax + 3.35 * s))
        E.append(el(f"arm_{n}", f"arm_{n}", (a0, 24, 12), (a1, 30, 15)))
        f0, f1 = sorted((s * 6.35, s * 9.7))
        E.append(el(f"forearm_{n}", f"forearm_{n}", (f0, 23, 13.5), (f1, 25.5, 21.5)))
        wx = ax + s * 1.6  # the wing feathers hang from under the forearm
        E.append(el(f"wing_{n}", f"forearm_{n}", (wx, 16.5, 11.5), (wx, 25, 21.5), quad="east" if s > 0 else "west", cut="wing"))
    pts = tail_points()
    for k, (length, w) in enumerate(TAIL):
        p = pts[k]
        hh = w * 0.85
        E.append(el(f"tail_{k}", f"tail_{k}", (-w / 2, p[1] - hh / 2, p[2] - length), (w / 2, p[1] + hh / 2, p[2] + 0.5)))
    tip = pts[-2]
    E.append(el("tail_fan", f"tail_{len(TAIL) - 1}", (-5, tip[1] + 0.2, tip[2] - 9.5), (5, tip[1] + 0.2, tip[2] - 1), quad="up", cut="fan"))
    for s, n in ((1, "L"), (-1, "R")):
        x = 4.5 * s
        # A unit wider outward: two texels past the hips' side (at x + 3.5, a 1.5-texel band beside them; linecheck).
        t0, t1 = sorted((x - 3.5 * s, x + 4.5 * s))
        E.append(el(f"thigh_{n}", f"leg_{n}_thigh", (t0, 18, -4), (t1, 30, 5)))
        E.append(el(f"shin_{n}", f"leg_{n}_shin", (x - 2, 8, -2.5), (x + 2, 19.5, 1.5)))
        E.append(el(f"foot_{n}", f"leg_{n}_foot", (x - 1.5, 2, -2), (x + 1.5, 9, 1)))
        E.append(el(f"toes_{n}", f"leg_{n}_toes", (x - 2.5, 0, -1.5), (x + 2.5, 2.2, 6)))
        cx = x - s * 1.75  # the sickle claw, on the inner toe, held up off the ground
        E.append(el(f"sickle_{n}", f"leg_{n}_toes", (cx - 0.75, 2.7, 1), (cx + 0.75, 4.7, 4)))
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


def rotm(rot):
    if not rot:
        return np.eye(3)
    rx, ry, rz = (math.radians(a) for a in rot)
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


def bars(t, period=5.0, width=0.32):
    """Bands across `t` (units): at 32 a block a texel is ~1.7 units, so a band is at least one
    texel and the period at least three."""
    period = max(period, 3 / TX)
    return ((t / period) % 1.0) < max(width, 1 / (TX * period))


def area(e, face, p, uu, vv, h):
    """Which material each texel is: a hawk-like coat (rust back with dark bars, sandy flanks,
    cream belly), a red crest, scaly legs, white teeth along the jaws, dark claws. `uu`, `vv` are
    texel indices (+0.5) on the face, `h` its height in texels; `p` the texels' points in units."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, SIDE)
    side = face in ("east", "west")
    t = vv / max(h, 1)  # 0 at a side face's top
    if name in ("hips", "chest", "neck") or name.startswith("tail_") and name != "tail_fan":
        if face == "up":
            a[:] = BACK
            a = np.where(bars(z + 1.5 * np.sin(x * 0.5), 5.5, 0.3), BAR, a)
        elif side:
            a = np.where(t < 0.4, BACK, np.where(t < 0.78, SIDE, BELLY))
            a = np.where((t < 0.4) & bars(z + 2 * t, 5.5, 0.3), BAR, a)
        elif face == "down":
            a[:] = BELLY
        elif face == "south":
            a = np.where(vv < 0.4 * h, BACK, np.where(np.abs(x) < 2.5, BELLY, SIDE))
        else:
            a[:] = BACK
        if name.startswith("tail_") and side:  # rings toward the tip
            a = np.where(bars(-z, 6, 0.28) & (t < 0.6), BAR, a)
    elif name == "skull":
        if face == "up":
            a[:] = BACK
        elif side:
            a = np.where(t < 0.35, BACK, SIDE)
            a = np.where(np.abs(y - EYE_Y) < 0.9, BAR, a)  # a dark stripe through the eye
        elif face == "down":
            a[:] = BELLY
    elif name == "snout":
        if face == "up":
            a[:] = BACK
        elif side:
            a = np.where(t < 0.3, BACK, SIDE)
            a = np.where((vv > h - 1) & ((np.floor(uu) % 2) == 0), TEETH, a)  # teeth along the edge
        elif face == "down":
            a[:] = DARK
            w = a.shape[1]
            a[:, [0, w - 1]] = TEETH
        elif face == "south":
            a[:] = SIDE
    elif name == "jaw":
        if face == "up":
            a[:] = DARK
            w = a.shape[1]
            teeth = np.where((np.arange(a.shape[0]) % 2) == 0, TEETH, DARK)
            a[:, 0], a[:, w - 1] = teeth, teeth
        elif side:
            a[:] = BELLY
            a = np.where((vv < 1) & ((np.floor(uu) % 2) == 1), TEETH, a)
        else:
            a[:] = BELLY
    elif name == "mouth":
        a[:] = DARK
    elif name == "crest":
        a[:] = CREST
        a = np.where(y > 46, TIP, a)
    elif name.startswith(("arm_", "forearm_")):
        a = np.where(t < 0.5, BACK, SIDE) if side or face == "up" else np.full(x.shape, SIDE)
        if name.startswith("forearm") and face == "south":
            a[:] = CLAW  # the hand's claws
    elif name.startswith("wing_"):
        a = np.where(y > 22, BACK, SIDE)
        a = np.where(y < 19, TIP, a)  # dark feather tips
        # The feathers' edges: a texel line every third texel (every 2 units was finer than a texel).
        a = np.where((np.floor(uu) % 3 == 1) & (y < 22), BAR, a)
    elif name == "tail_fan":
        a[:] = BACK
        a = np.where(z < e["lo"][2] + 2.5, TIP, a)
        a = np.where(np.floor(uu) % 3 == 1, BAR, a)  # the feathers' edges, a texel line every third texel
    elif name.startswith("thigh_"):
        a = np.where(t < 0.5, BACK, SIDE) if side else np.full(x.shape, SIDE)
        if face == "down":
            a[:] = BELLY
    elif name.startswith(("shin_", "foot_", "toes_")):
        a[:] = SCALE
        if name.startswith("toes_") and face == "south":
            a[-1, :] = CLAW
    elif name.startswith("sickle_"):
        a[:] = CLAW
    elif name.startswith("eyelid"):
        a[:] = SIDE
    return a


def cut_mask(e, p, uu, vv, su, sv):
    """Which texels of a feather panel are there. `p` in units; the ragged tips alternate by texel
    (`uu`, `vv` texel indices, `su`, `sv` units a texel), a texel deep."""
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    if e["cut"] == "crest":  # feathers rising from the head back over the neck
        top = 43 + 3.5 * np.clip((z - 13) / 10, 0, 1) - 2.2 * np.clip((z - 24) / 5, 0, 1)
        top = top + sv * ((np.floor(uu) % 2) == 0)
        return (y <= top) & (z >= 13) & (z <= 29)
    if e["cut"] == "wing":  # feathers fanning down and back from the forearm, ragged tips
        z0, z1 = e["lo"][2], e["hi"][2]
        bottom = 17 + 5 * np.clip((z - z0) / (z1 - z0), 0, 1) - sv * ((np.floor(uu) % 2) == 0)
        return y >= bottom
    if e["cut"] == "fan":  # a fan of tail feathers, widest behind the tail's end
        z0, z1 = e["lo"][2], e["hi"][2]
        half = 1.4 + 3.6 * np.clip((z1 - z) / (z1 - z0), 0, 1) - su * ((np.floor(vv) % 2) == 0)
        return np.abs(x) <= half
    return np.ones(x.shape, bool)


LASH = (54, 32, 26)


# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}  # a layers.Recorder while build() paints, and the face's key


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)
def layer_props(e, face, p, a):
    """Where white, patterns show on a raptor: white from the belly and throat up; patterns on the
    body, neck, tail, thighs and feathers, less on the head, none on the scaly legs."""
    n, y = e["name"], p[..., 1]
    lo, hi = e["lo"][1], e["hi"][1]
    t = _ramp(y, lo, max(hi, lo + 1e-3))  # 0 at the part's bottom
    if n in ("hips", "chest", "neck") or n.startswith(("tail_", "thigh_", "shag_")):
        white = 0.85 * (1 - t) + 0.15 * (face == "down")
    elif n in ("skull", "snout", "jaw"):
        white = 0.6 * (1 - t)
    else:
        white = np.full(y.shape, 0.2)
    legs = n.startswith(("shin_", "foot_", "toe", "sickle_"))
    face_part = n in ("snout", "jaw", "mouth")
    pattern = np.full(y.shape, 0.0 if legs else 0.35 if face_part else 0.8 if n == "skull" else 1.0)
    return {"white": white, "pattern": pattern}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    if w == 0 or h == 0:
        return None
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel centers, in texels
    size = np.array(e["hi"]) - np.array(e["lo"])
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
    p = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    light_face = face
    if face == "down" and e["name"].startswith(("thigh_", "shin_", "foot_", "tail_", "arm_", "forearm_", "jaw")):
        light_face = "north"  # seen when a pose lifts or folds it
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=FUR, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"]:
        img[..., 3] = np.where(cut_mask(e, p, uu, vv, su, sv), 255, 0)
    if e["name"] == "skull" and face in ("east", "west"):
        (x0, y0, z0), (x1, y1, z1) = SKULL
        cu = ((z1 - EYE_Z) if face == "east" else (EYE_Z - z0)) / su
        design = EYE_STYLE + "_m" if face == "west" and EYE_STYLE + "_m" in EYES else EYE_STYLE
        stamp_eyes(img, [(cu, (y1 - EYE_Y) / sv)], design)
        # A brow ridge over the eye.
        pw = len(EYES[EYE_STYLE]["pattern"][0])
        u0, v0 = eye_corner_side(1 if face == "east" else -1)
        if v0 >= 1:
            img[v0 - 1, max(u0 - 1, 0):u0 + pw + 1, :3] = (img[v0 - 1, max(u0 - 1, 0):u0 + pw + 1, :3] * 0.7).astype(np.uint8)
    if e["name"] == "snout" and face == "south" and w >= 6 and h >= 3:  # nostrils (none at 32: on a front 4 by 2 they read as eyes)
        img[1, [1, w - 2], :3] = (60, 34, 30)
    if e["name"].startswith("eyelid"):  # a closed eye: the lash along the edge that ends up at the bottom
        col_idx = 0 if e["name"].endswith("L") else w - 1
        k = 1.0 if w >= 3 else 0.5  # at 32 the lid is 2 texels: a full dark lash would be half of it
        img[:, col_idx, :3] = np.round(img[:, col_idx, :3] * (1 - k) + np.array(LASH) * k).astype(np.uint8)
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

# The stance: crouched like a bird, knees forward, ankles back, toes flat (the angles sum to 0).
L = {"thigh": -32, "shin": 58, "foot": -46, "drop": 0.0,
     "rest_thigh": -70, "rest_shin": 140, "rest_foot": -100, "rest_drop": 0.0,
     "low_neck": 40, "low_head": 10, "low_crouch": 0.0, "low_drop": 0.0, "death_drop": 10.0}


def legs(base_t, base_s, base_f, swing_l=0.0, swing_r=0.0, lift_l=0.0, lift_r=0.0):
    q = {}
    for n, sw, lift in (("L", swing_l, lift_l), ("R", swing_r, lift_r)):
        t = base_t + sw - 0.4 * lift
        s = base_s + lift
        f = base_f - 0.4 * lift
        q[f"leg_{n}_thigh"], q[f"leg_{n}_shin"], q[f"leg_{n}_foot"] = (t, 0, 0), (s, 0, 0), (f, 0, 0)
        q[f"leg_{n}_toes"] = (-(t + s + f), 0, 0)  # toes flat
    return q


def tail_sway(p, n, amp, lift=0.0):
    q = {}
    for k in range(len(TAIL)):
        q[f"tail_{k}"] = swing((0, 0, -1), amp * (0.5 if k == 0 else 1) * S(TAU * p * n - 0.6 * k), (lift if k == 0 else 0, 0, 0))
    return q


NECK = -22  # the neck raised, the head held level: an S-curve, the head above the back


def stand(t=0.0):
    q = {"root": (0, -L["drop"], 0), "neck": (NECK, 0, 0), "head": (-NECK, 0, 0)}
    q.update(legs(L["thigh"], L["shin"], L["foot"]))
    for n, s in (("L", 1), ("R", -1)):  # arms folded like a bird's wings
        q[f"arm_{n}"] = (35, 0, -s * 8)
        q[f"forearm_{n}"] = (-80, 0, 0)
    return q


def pose_idle(t):
    p = t / 3.0
    q = stand()
    br = S(TAU * p * 2)
    q["root"] = (0, q["root"][1] + 0.15 * br, 0)
    q["spine_1"] = (0.8 * br, 0, 0)
    # A bird's head: it holds still, then snaps to a new look.
    look = float(np.interp(p, [0, .2, .23, .5, .53, .78, .81, 1], [0, 0, 22, 22, -14, -14, 0, 0]))
    tilt = float(np.interp(p, [0, .5, .53, .78, .81, 1], [0, 0, 8, 8, 0, 0]))
    q["neck"] = (NECK - 4 * br, 0, 0)
    q["head"] = (-NECK, look, tilt)
    q.update(tail_sway(p, 1, 6))
    return q


def pose_walk(t):
    p = t / 0.9
    a = TAU * p
    q = stand()
    # A foot swings forward (thigh angle falling) while it's lifted: swing -sin, lift +cos.
    q.update(legs(L["thigh"], L["shin"], L["foot"], -24 * S(a), 24 * S(a), 30 * max(0.0, S(a + math.pi / 2)),
                  30 * max(0.0, S(a - math.pi / 2))))
    q["root"] = (0, q["root"][1] + 0.6 * abs(S(a)), 0)
    q["neck"] = (NECK + 6 * S(2 * a), 0, 0)  # the head bobs back and forth, like a bird's
    q["head"] = (-NECK - 6 * S(2 * a), 0, 0)
    q["spine_0"] = (0, 3 * S(a), 0)
    q.update(tail_sway(p, 1, 7))
    return q


def pose_run(t):
    p = t / 0.5
    a = TAU * p
    q = stand()
    q.update(legs(L["thigh"] - 5, L["shin"], L["foot"], -42 * S(a), 42 * S(a), 60 * max(0.0, S(a + math.pi / 2)),
                  60 * max(0.0, S(a - math.pi / 2))))
    q["root"] = (0, q["root"][1] + 1.5 * abs(S(a)) - 1.0, 0)
    q["spine_0"] = (6, 0, 0)  # leaning forward, the tail straight out behind
    q["neck"] = (NECK + 30 + 4 * S(2 * a), 0, 0)  # neck stretched forward
    q["head"] = (-NECK - 20, 0, 0)
    q.update(tail_sway(p, 1, 3, lift=-6))
    return q


def pose_rest(t, P=L):
    """Resting like a bird: legs folded under, belly and tail on the ground, the head up."""
    p = t / 4.0
    br = S(TAU * p * 2)
    q = {"root": (0, -P["rest_drop"] + 0.1 * br, 0)}
    q.update(legs(P["rest_thigh"], P["rest_shin"], P["rest_foot"]))
    for n, s in (("L", 1), ("R", -1)):
        q[f"arm_{n}"] = (35, 0, -s * 8)
        q[f"forearm_{n}"] = (-80, 0, 0)
    q["neck"] = (10, 0, 0)
    q["head"] = (-8, 8 * S(TAU * p), 0)
    q.update({f"tail_{k}": (8 if k == 0 else 0, 4 * S(TAU * p - 0.5 * k) if k else 0, 0) for k in range(len(TAIL))})
    return q


def pose_screech(t):
    """Head up, the jaw wide open, arms and wings spread."""
    p = t / 1.6
    q = stand()
    up = float(np.interp(p, [0, .15, .75, 1], [0, 1, 1, 0]))
    q["spine_0"] = (-8 * up, 0, 0)
    q["neck"] = (NECK - 20 * up, 0, 0)
    q["head"] = (-NECK - 15 * up, 0, 0)
    q["jaw"] = (float(np.interp(p, [0, .15, .25, .35, .45, .55, .75, 1], [0, 34, 26, 34, 26, 34, 0, 0])), 0, 0)
    for n, s in (("L", 1), ("R", -1)):
        q[f"arm_{n}"] = (35 - 45 * up, 0, -s * (8 + 50 * up))
        q[f"forearm_{n}"] = (-80 + 50 * up, 0, 0)
    q.update(tail_sway(p, 2, 8 * up, lift=-10 * up))
    return q


def pose_bite(t):
    """A bite: the head draws back, darts forward and down with the jaws opening, and snaps them
    shut on what it strikes; then back."""
    p = t / 0.7
    q = stand()
    d = float(np.interp(p, [0, .3, .45, .6, 1], [0, -1, 1, 1, 0]))  # -1 drawn back, 1 thrust out
    q["spine_0"] = (5 * d, 0, 0)
    q["neck"] = (NECK + 18 * d, 0, 0)
    q["head"] = (-NECK - 8 * d, 0, 0)
    q["jaw"] = (float(np.interp(p, [0, .25, .42, .5, 1], [0, 22, 34, 0, 0])), 0, 0)
    q.update(tail_sway(p, 1, 3, lift=-4 * d))
    return q


def pose_kick(t):
    """The sickle-claw kick: leaning back on one leg, the other draws up, strikes down and forward
    with the claw, and is set down again; the arms out for balance."""
    p = t / 0.8
    q = stand()
    lift = float(np.interp(p, [0, .3, .45, .6, 1], [0, 1, .5, 0, 0]))
    swing = float(np.interp(p, [0, .3, .45, .6, 1], [0, -23, -47, -18, 0]))  # negative: forward
    q.update(legs(L["thigh"], L["shin"], L["foot"], swing_l=swing, lift_l=55 * lift))
    lean = float(np.interp(p, [0, .3, .5, 1], [0, 1, 1, 0]))
    q["spine_0"] = (-8 * lean, 0, 0)
    q["neck"] = (NECK - 6 * lean, 0, 0)
    q["head"] = (-NECK + 6 * lean, 0, 0)
    for n, s in (("L", 1), ("R", -1)):
        q[f"arm_{n}"] = (35 - 30 * lean, 0, -s * (8 + 30 * lean))
        q[f"forearm_{n}"] = (-80 + 30 * lean, 0, 0)
    q.update(tail_sway(p, 1, 4, lift=-8 * lean))
    return q


# ------------------------------------------------------------------ behaviour clips (`anim.rs`)
# Each a whole-body pose over what plays below on the bones it moves (`to_bedrock.as_action`):
# held states loop, the rest play once.


def arms(fold=1.0, out=0.0):
    """The arms folded like a bird's wings (`fold` 1), or spread by `out`."""
    q = {}
    for n, s in (("L", 1), ("R", -1)):
        q[f"arm_{n}"] = (35 - 45 * out, 0, -s * (8 + 50 * out))
        q[f"forearm_{n}"] = (-80 + 50 * out, 0, 0)
    return q


def crouched(c, P=L):
    """The stance crouched deeper by `c` (1: a stalk's), toes flat, dropped to stay on the ground."""
    q = legs(P["thigh"] - 14 * c, P["shin"] + 26 * c, P["foot"] - 12 * c)
    return q


def head_down(P=L, d=1.0):
    """The snout to the ground by `d` (1: half a unit over it, fitted): crouched, the neck lowered."""
    q = {"root": (0, -L["drop"] - P["low_drop"] * d, 0)}
    q.update(crouched(P["low_crouch"] * d, P))
    q.update(arms())
    q.update({"neck": (NECK + P["low_neck"] * d, 0, 0), "head": (-NECK + P["low_head"] * d, 0, 0)})
    return q


def pose_stare(t):
    """Half aware: frozen, the neck up, the head locked on, only the breath moving (2 s, held)."""
    q = stand()
    br = S(TAU * t)
    q.update({"spine_1": (0.8 * br, 0, 0), "neck": (NECK - 8, 0, 0), "head": (-NECK + 4, 0, 0)})
    q.update(tail_sway(t / 2.0, 1, 1))
    return q


def pose_sniff(t):
    """The snout down to the ground, a quick sniff along, and up (2 s, once)."""
    p = t / 2.0
    d = float(np.interp(p, [0, .25, .75, 1], [0, 1, 1, 0]))
    q = head_down(d=d)
    q["head"] = (q["head"][0], 10 * d * S(TAU * p * 2), 0)
    q.update(tail_sway(p, 1, 4))
    return q


def pose_drink(t):
    """Drinking like a bird: the snout dipped, then the head tipped up to swallow (2 s, held)."""
    p = t / 2.0
    d = float(np.interp(p, [0, .15, .45, .6, .85, 1], [0.2, 1, 1, 0.3, 0.3, 0.2]))
    q = head_down(d=d)
    q["head"] = (q["head"][0] - 14 * (1 - d), 0, 0)
    q["jaw"] = (6 * max(0.0, S(TAU * p * 4)) * d, 0, 0)
    q.update(tail_sway(p, 1, 2))
    return q


def pose_look(t):
    """Looking round, a bird's way: still, snap to one side, a tilt, snap to the other, back (2.5 s, once)."""
    p = t / 2.5
    look = float(np.interp(p, [0, .1, .13, .4, .43, .7, .73, 1], [0, 0, 45, 45, -40, -40, 0, 0]))
    tilt = float(np.interp(p, [0, .2, .23, .4, .43, 1], [0, 0, 12, 12, 0, 0]))
    q = stand()
    q.update({"neck": (NECK, 0.6 * look, 0), "head": (-NECK, 0.4 * look, tilt)})
    q.update(tail_sway(p, 1, 4))
    return q


def preen(t, dur, side=1):
    """Preening the feathers of one arm: the arm out, the snout working through it."""
    p = t / dur
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    n = "L" if side > 0 else "R"
    q = stand()
    q.update({"neck": (NECK + 30 * d, 44 * side * d, 0), "head": (-NECK + 10 * d, 20 * side * d, 0),
              "jaw": (10 * d * max(0.0, S(TAU * p * 6)), 0, 0), f"arm_{n}": (35 - 30 * d, 0, -side * (8 + 40 * d)),
              f"forearm_{n}": (-80 + 40 * d, 0, 0)})
    q.update(tail_sway(p, 1, 3))
    return q


def pose_scratch(t):
    """An itch: preening the left arm's feathers (1.8 s, once)."""
    return preen(t, 1.8, 1)


def pose_groom(t):
    """Grooming: preening the right arm's feathers, longer (2.6 s, once)."""
    return preen(t, 2.6, -1)


def pose_stretch(t):
    """Getting up: both arms spread wide and shaken, the neck long, a leg stretched back (2.4 s, once)."""
    p = t / 2.4
    a = float(np.interp(p, [0, .2, .55, .65, 1], [0, 1, 1, 0, 0]))
    b = float(np.interp(p, [0, .6, .72, .9, 1], [0, 0, 1, 1, 0]))
    q = stand()
    q.update(arms(out=a))
    q.update({"neck": (NECK - 10 * a, 0, 0), "head": (-NECK - 10 * a, 0, 0), "jaw": (20 * a * max(0.0, S(TAU * p * 2)), 0, 0)})
    q.update(legs(L["thigh"], L["shin"], L["foot"], swing_r=24 * b, lift_r=-10 * b))
    q.update(tail_sway(p, 3, 8 * a))
    return q


def pose_sleep(t, P=L):
    """Asleep, resting: the legs folded under, the head turned back and tucked along the body,
    eyes shut, slow breath (6 s, held)."""
    q = pose_rest(0.0, P)
    br = S(TAU * t / 6.0 * 2)
    q["root"] = (0, -P["rest_drop"] + 0.12 * br, 0)
    q.update({"spine_1": (0.6 * br, 0, 0), "neck": (30, 60, 0), "head": (10, 30, 0), "jaw": (0, 0, 0)})
    q.update({"eyelid_L": (0, 0, 90), "eyelid_R": (0, 0, -90)})
    return q


def pose_stalk(t):
    """Stalking: crouched low, the neck out level, the head still and aimed, the tail stiff behind,
    slow high careful steps (1.3 s a cycle, in place of the walk)."""
    p = t / 1.3
    a = TAU * p
    q = {"root": (0, -L["drop"] - 4.0, 0)}
    q.update(legs(L["thigh"] - 14, L["shin"] + 26, L["foot"] - 12, -16 * S(a), 16 * S(a), 34 * max(0.0, S(a + math.pi / 2)),
                  34 * max(0.0, S(a - math.pi / 2))))
    q.update(arms())
    q.update({"spine_0": (8, 0, 0), "neck": (NECK + 26, 0, 0), "head": (-NECK - 34, 0, 0)})
    q.update(tail_sway(p, 1, 1, lift=-8))
    return q


def pose_approach(t):
    """Curious, coming to look: slow steps a little crouched, the neck stretched forward and low, the
    head cocked one way and the other as a bird's does, the tail level (1.2 s a cycle, in place of the
    walk). Its height is set from its feet in the build (`ground.grounded`)."""
    p = t / 1.2
    a = TAU * p
    q = {"root": (0, -L["drop"], 0)}
    q.update(legs(L["thigh"] - 6, L["shin"] + 12, L["foot"] - 6, -18 * S(a), 18 * S(a), 28 * max(0.0, S(a + math.pi / 2)),
                  28 * max(0.0, S(a - math.pi / 2))))
    q.update(arms())
    q.update({"spine_0": (5, 0, 0), "neck": (NECK + 18, 0, 0), "head": (-NECK - 22, 0, 14 * S(a))})
    q.update(tail_sway(p, 1, 3, lift=-4))
    return q


def pose_alarm(t):
    """Alarm: the head snapped up, a sharp double chirp, the arms half spread (1.0 s, once)."""
    p = t / 1.0
    up = float(np.interp(p, [0, .1, .8, 1], [0, 1, 1, 0]))
    call = float(np.interp(p, [0, .2, .28, .36, .5, .58, .66, 1], [0, 0, 1, 0, 0, 1, 0, 0]))
    q = stand()
    q.update(arms(out=0.4 * up))
    q.update({"spine_0": (-6 * up, 0, 0), "neck": (NECK - 18 * up, 0, 0), "head": (-NECK - 6 * up, 0, 0), "jaw": (26 * call, 0, 0)})
    q.update(tail_sway(p, 1, 2, lift=-8 * up))
    return q


def pose_pounce(t):
    """The pounce: a crouch, a leap up and onto the prey feet first, both sickle claws forward, the
    arms spread for balance, the jaws open (1.0 s, once)."""
    p = t / 1.0
    c = float(np.interp(p, [0, .25, .35, 1], [0, 1, 0, 0]))
    up = float(np.interp(p, [0, .3, .5, .75, .9, 1], [0, 0, 1, .8, 0, 0]))
    q = {"root": (0, -L["drop"] - 4.0 * c + 9.0 * up, 0)}
    q.update(legs(L["thigh"] - 14 * c, L["shin"] + 26 * c, L["foot"] - 12 * c, -40 * up, -40 * up, 30 * up, 30 * up))
    q.update(arms(out=up))
    q.update({"spine_0": (-14 * up, 0, 0), "neck": (NECK + 10 * c, 0, 0), "head": (-NECK + 14 * up, 0, 0), "jaw": (30 * up, 0, 0)})
    q.update(tail_sway(p, 1, 2, lift=-6 * up))
    return q


def pose_play(t):
    """A young one's play: a bob and a hop to the side, the head tilting, a mock snap (1.4 s, once)."""
    p = t / 1.4
    hop = max(0.0, S(TAU * p * 2))
    q = stand()
    q["root"] = (0, q["root"][1] + 5.0 * hop, 0)
    q.update({"spine_0": (0, 20 * S(TAU * p), 0), "neck": (NECK + 6 * hop, 0, 0), "head": (-NECK, 20 * S(TAU * p), 14 * hop),
              "jaw": (20 * max(0.0, S(TAU * p * 3)), 0, 0)})
    q.update(arms(out=0.3 * hop))
    q.update(tail_sway(p, 2, 7))
    return q


def pose_nuzzle(t):
    """Nuzzling a young one: the head low, a gentle nudge and a nibble (2 s, once)."""
    p = t / 2.0
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    push = max(0.0, S(TAU * p * 2))
    q = head_down(d=0.6 * d)
    q["head"] = (q["head"][0] - 8 * push, 0, 0)
    q["jaw"] = (8 * d * push, 0, 0)
    q.update(tail_sway(p, 2, 3))
    return q


def pose_flinch(t):
    """Hit: a jerk back, the head ducked, the arms snapped in, the tail lashing (0.4 s, once)."""
    p = t / 0.4
    d = float(np.interp(p, [0, .15, .45, 1], [0, 1, 0.8, 0]))
    q = stand()
    q.update({"spine_0": (-6 * d, 0, 4 * d), "neck": (NECK + 16 * d, 0, 0), "head": (-NECK + 10 * d, 0, 0), "jaw": (12 * d, 0, 0)})
    q.update(tail_sway(p, 2, 7 * d))
    return q


def pose_death(t, P=L):
    """Dying: the legs give, it topples onto its side, the neck and tail fall slack; it stays (1.2 s, holds)."""
    p = t / 1.2
    fall = float(np.interp(p, [0, .35, .7, 1], [0, 0.35, 1, 1]))
    q = stand()
    q["root"] = (0, -L["drop"] - P["death_drop"] * fall, 0)
    q.update({"spine_0": (0, 0, 84 * fall), "neck": (NECK + 30 * fall, 0, -10 * fall), "head": (-NECK + 10 * fall, 0, 0), "jaw": (10 * fall, 0, 0)})
    q.update(legs(L["thigh"] + 20 * fall, L["shin"] - 30 * fall, L["foot"] + 10 * fall))
    q.update({f"tail_{k}": (0, 0, 0) for k in range(len(TAIL))})
    q.update({"eyelid_L": (0, 0, 90 * fall), "eyelid_R": (0, 0, -90 * fall)})
    return q


BLINKS = {"idle": [1.5], "rest": [1.2, 3.0], "stare": [1.2], "look": [1.2]}
BLINK = 0.09
ANIMS = {"idle": (pose_idle, 3.0), "walk": (pose_walk, 0.9), "run": (pose_run, 0.5), "rest": (pose_rest, 4.0),
         "screech": (pose_screech, 1.6), "bite": (pose_bite, 0.7), "kick": (pose_kick, 0.8),
         "stare": (pose_stare, 2.0), "sniff": (pose_sniff, 2.0), "drink": (pose_drink, 2.0), "look": (pose_look, 2.5),
         "scratch": (pose_scratch, 1.8), "groom": (pose_groom, 2.6), "stretch": (pose_stretch, 2.4), "sleep": (pose_sleep, 6.0),
         "stalk": (pose_stalk, 1.3), "alarm": (pose_alarm, 1.0), "pounce": (pose_pounce, 1.0), "play": (pose_play, 1.4),
         "nuzzle": (pose_nuzzle, 2.0), "flinch": (pose_flinch, 0.4), "death": (pose_death, 1.2), "approach": (pose_approach, 1.2)}
LOOPS = {"stare": True, "drink": True, "sleep": True, "stalk": True, "approach": True, "death": "hold_on_last_frame"}
BLENDS = {"flinch": (0.05, 0.2), "death": (0.1, 0.0), "stalk": (0.3, 0.3), "approach": (0.3, 0.3), "sleep": (0.6, 0.6)}


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}  # they swing down over the side of the head


def frame_times(name, dur):
    if name in ("walk", "run", "rest"):
        n = 12 if dur < 1 else 16
    else:
        n = 30 if name in ("idle", "screech", "bite", "kick", "look", "pounce", "play", "stalk", "approach") else 24
    ts = {round(dur * k / n, 4) for k in range(n + 1)}
    for b in BLINKS.get(name, []):
        ts |= {round(b + d, 4) for d in (-BLINK, -BLINK / 2, 0, BLINK / 2, BLINK)}
    return sorted(ts)


def animations(bone_names):
    out = {}
    for name, (fn, dur) in ANIMS.items():
        frames = []
        loops = LOOPS.get(name, name in ("idle", "walk", "run", "rest"))
        for t in frame_times(name, dur):
            # A loop's last frame is its first; a clip played once ends where it ends.
            tt = t % dur if t < dur else (0.0 if loops is True else dur)
            pose = fn(tt)
            if name not in ("sleep", "death"):  # those keep their eyes shut
                pose.update(eyelids(name, tt))
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames, "loop": loops, "blend": BLENDS.get(name, (0.25, 0.3))}
    return out


# ------------------------------------------------------------------ fitting poses to the ground

GROUPS = {"toes": lambda n: n.startswith("toes_"), "body": lambda n: n in ("hips", "chest"),
          "legs": lambda n: n.startswith(("thigh_", "shin_", "foot_")), "tail": lambda n: n.startswith("tail_") and n != "tail_fan",
          "sickle": lambda n: n.startswith("sickle_")}


def fit_poses(spec):
    """The stance's drop (toes on the ground), and the rest pose's leg angles and drop (belly and
    toes on the ground, nothing below it)."""
    g = lowest(spec, stand(), GROUPS)
    L["drop"] = round(float(g["toes"]), 2)

    def rest_cost(P):
        g = lowest(spec, pose_rest(0.0, P), GROUPS)
        g.pop("sickle", None)
        floor = min(g.values())
        r = {k: v - floor for k, v in g.items()}
        return r["body"] ** 2 + r["toes"] ** 2 + 0.3 * r["tail"] ** 2
    grid = {"rest_thigh": list(range(-95, -44, 5)), "rest_shin": list(range(100, 166, 5)), "rest_foot": list(range(-130, -59, 10))}
    best = fit(grid, lambda P: rest_cost(dict(L, **P, rest_drop=0)))[1]
    P = dict(L, **best, rest_drop=0)
    g = lowest(spec, pose_rest(0.0, P), GROUPS)
    g.pop("sickle", None)
    L.update(best, rest_drop=round(float(min(g.values())), 2))

    # Head down (sniffing, drinking): the snout half a unit over the ground, the toes on it.
    heads = {"head": lambda n: n in ("skull", "snout", "jaw", "nose", "muzzle")}
    def low_pose(P):
        q = head_down(dict(L, **P, low_drop=0))
        g = lowest(spec, q, {**GROUPS, **heads})
        drop = g["toes"] + L["drop"] * 0
        return g.get("head", 99) - g["toes"], g["toes"]
    def low_cost(P):
        nose, _ = low_pose(P)
        return (nose - 0.5) ** 2 + 0.0002 * (P["low_neck"] ** 2 + P["low_head"] ** 2) + 1.0 * P["low_crouch"] ** 2
    best = fit({"low_neck": list(range(20, 121, 5)), "low_head": list(range(0, 61, 5)), "low_crouch": [0, .5, 1, 1.5, 2]}, low_cost)[1]
    nose, toes = low_pose(best)
    L.update(best, low_drop=round(float(toes + L["drop"]), 2))
    print("head down, snout over the ground:", round(float(nose), 2), best)
    # Dying on its side: dropped until nothing is below the ground.
    g = lowest(spec, pose_death(1.2, dict(L, death_drop=0)), {"all": lambda n: True})
    L["death_drop"] = round(float(g["all"]) - 0.0, 2)
    print("dead, dropped", L["death_drop"])


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
    sheet, uv = pack(tiles, 512)
    LAYERS["rec"].save(OUT, [k for k, _ in tiles], pack)
    LAYERS["rec"] = None
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "raptor", "texture": [sheet.shape[1], sheet.shape[0]],
            "bones": [{"name": n, "parent": p, "pivot": [float(v) for v in pv]} for n, p, pv in B], "elements": E}
    fit_poses(spec)
    spec["animations"] = animations([n for n, _, _ in B])
    spec["fitted"] = dict(L)
    return spec, sheet


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    spec, sheet = build()
    Image.fromarray(sheet, "RGBA").save(os.path.join(OUT, "texture.png"))
    # Velociraptor osmolskae: the same sheet in its colors.
    set_palette(OSMOLSKAE)
    E = elements()
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"] and not e["rot"]]
    tiles = [((i, f), img) for i, e in enumerate(E) for f in ([e["quad"]] if e["quad"] else list(FACES))
             if (img := paint_face(e, f, boxes)) is not None]
    other, _ = pack(tiles, 512)
    assert other.shape == sheet.shape
    Image.fromarray(other, "RGBA").save(os.path.join(OUT, "osmolskae.png"))
    set_palette(BASE)
    with open(os.path.join(OUT, "spec.json"), "w") as f:
        json.dump(spec, f)
    print(len(spec["elements"]), "elements;", len(spec["bones"]), "bones; texture", sheet.shape[1], "x", sheet.shape[0],
          "; fitted", spec["fitted"])
