#!/usr/bin/env python3
"""The wood mouse, made the way Hytale makes creatures (boxes and a hand-painted texture) with the
blocky-model skill, from research alone (README.md): its colors are rules (see `area`).

    ~/.venvs/blocky/bin/python mouse.py      # writes out/spec.json, out/texture.png

Units are texels at the house density (64 a block, 1.56 cm each), y up, the mouse faces +z. A
wood mouse is 8-11 cm head and body with a tail as long: a real one is six texels long, too small
for a face. Here the body is about 1.3 times life and the head, eyes and ears bigger still (cute
exaggeration, Hytale's way), so the face has room for a big black eye: 9 units nose to rump, the
tail 8 more.
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

OUT = os.environ.get("MOUSE_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

FUR, BACK, BELLY, PINK, NOSE, EARIN, CLAW = range(7)
# FUR the warm sandy brown of the flanks, BACK darker along the spine, BELLY the grey-white
# underside (sharply divided from the brown, as a wood mouse's is), PINK the bare feet and tail's
# underside, NOSE its tip, EARIN the thin pinkish-grey inside of the ears.
BASE = [(150, 104, 66), (116, 78, 52), (214, 208, 196), (206, 160, 150), (178, 118, 116), (178, 140, 130), (96, 76, 70)]
SHADOW = [0.02, 0.02, 0.62, 0.97, 0.97, 0.97, 0.02]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (FUR, BACK, BELLY)
ROLES = [("coat", 0, 0), ("coat", 0, -1.0), ("coat", 1, 0), "fixed", "fixed", "fixed", "fixed"]

# Big, bulging, black eyes (a nocturnal seed eater's): glossy, a catchlight toward the front.
EYES.update({
    "mouse_black": dict(pattern=["PH",
                                 "PP"], colors={"P": (20, 14, 16), "H": (222, 224, 230)}, halo="none"),
    "mouse_bead": dict(pattern=["P"], colors={"P": (20, 14, 16)}, halo="none"),
})
EYE_STYLE = os.environ.get("MOUSE_EYES", "mouse_black")
SKULL = ((-1.25, 1.7, 1.7), (1.25, 4.0, 4.7))
EYE_Z, EYE_Y = 3.7, 3.0  # high on the side, toward the front

# ------------------------------------------------------------------ the model


def el(name, bone, lo, hi, quad=None, rot=None, origin=None, cut=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad, rot=rot,
                origin=[float(v) for v in origin] if origin else None, cut=cut)


LEGS = ("FL", "FR", "BL", "BR")
FRONT_Z, BACK_Z, FRONT_X, BACK_X = 0.0, -2.2, 0.6, 1.1
# Half widths across the body (x), chosen so no two faces that can meet in a pose share a plane:
# the hips, the chest, the haunch inside and out of its leg's line, the hind shin and foot, the
# front leg and paw.
G = {"hips": 2.3, "chest": 1.65, "thigh_in": 0.8, "thigh_out": 1.55, "shin": 0.2, "foot": 0.85, "fleg": 0.15, "fpaw": 0.55,
     "foot_len": 1.0, "belly": 1.2, "hind": 1.5}
TAIL = [(-8, 3.2, 0.95), (-4, 4.6, 0.55)]  # pitch (deg, down), length, width (thinner to the tip)
TAIL0 = np.array([0.0, 1.9, -3.4])
EAR = [(3.7, 4.6, 0.75), (4.6, 5.2, 0.5)]  # y0, y1, half width
EAR_X, EAR_Z = 0.85, (2.8, 3.6)


def tail_points():
    pts, p = [TAIL0.copy()], TAIL0.copy()
    for pitch, length, _ in TAIL:
        a = math.radians(pitch)
        p = p + np.array([0, math.sin(a), -math.cos(a)]) * length
        pts.append(p.copy())
    return pts


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 2.0, -1.6)), ("spine_1", "spine_0", (0, 2.0, -0.95)),
         ("head", "spine_1", (0, 2.6, 1.9))]
    for s, n in ((1, "L"), (-1, "R")):
        B.append((f"ear_{n}", "head", (s * EAR_X, 3.85, sum(EAR_Z) / 2)))
    pts = tail_points()
    B += [(f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", tuple(pts[k])) for k in range(len(TAIL))]
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x = FRONT_X * s
            # The paw hinges at its back edge: lifting, it curls its toes up.
            B += [(f"leg_{leg}_upper", "spine_1", (x, 1.6, FRONT_Z)), (f"leg_{leg}_paw", f"leg_{leg}_upper", (x, 0.45, FRONT_Z - 0.4))]
        else:
            x = BACK_X * s
            B += [(f"leg_{leg}_upper", "spine_0", (x, 2.0, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 1.85, BACK_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 0.53, BACK_Z))]
    return B


def elements():
    E = [el("hips", "spine_0", (-G["hips"], G["belly"], -3.5), (G["hips"], 3.5, -0.9)),  # pear-shaped: wide behind
         el("rump", "spine_0", (-0.85, 1.6, -3.9), (0.85, 3.0, -3.1)),
         el("chest", "spine_1", (-G["chest"], 1.55, -1.5), (G["chest"], 3.1, 1.2)),
         el("neck", "spine_1", (-0.9, 2.15, -0.5), (0.9, 3.5, 3.2)),  # short, deep in the head: it turns on it
         el("skull", "head", *SKULL),
         el("muzzle", "head", (-0.6, 2.1, 4.3), (0.6, 3.3, 6.0))]  # the pointed snout, the nose at its tip
    for s, n in ((1, "L"), (-1, "R")):
        cx = EAR_X * s
        # Big round ears: two thin steps and a cut-out panel through the middle for the outline,
        # turned out a little.
        rot, origin = (-8, 0, -s * 18), (cx, 3.85, sum(EAR_Z) / 2)
        for k, (y0, y1, hw) in enumerate(EAR):
            E.append(el(f"ear_core_{n.lower()}{k}", f"ear_{n}", (cx - hw, y0, EAR_Z[0]), (cx + hw, y1, EAR_Z[1]), rot=rot, origin=origin))
        zc = sum(EAR_Z) / 2
        E.append(el(f"fringe_ear_{n.lower()}", f"ear_{n}", (cx - 1.0, 3.7, zc), (cx + 1.0, 5.6, zc), quad="south", rot=rot,
                    origin=origin, cut="ear"))
    pts = tail_points()
    for k, (pitch, length, w) in enumerate(TAIL):
        p = pts[k]
        E.append(el(f"tail_{k}", f"tail_{k}", (p[0] - w / 2, p[1] - w / 2, p[2] - length), (p[0] + w / 2, p[1] + w / 2, p[2] + 0.6),
                    rot=(pitch, 0, 0), origin=tuple(p)))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z = FRONT_X * s, FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - G["fleg"], 0.35, z - 0.3), (x + G["fleg"], 2.0, z + 0.3)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - G["fpaw"], 0, z - 0.65), (x + G["fpaw"], 0.6, z + 0.9)))
        else:
            x, z = BACK_X * s, BACK_Z
            xi, xo = sorted((x - G["thigh_in"] * s, x + G["thigh_out"] * s))  # the haunch
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (xi, 1.75, z - 0.9), (xo, 2.3, z + 0.7)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - G["shin"], 0.5, z - 0.3), (x + G["shin"], 1.95, z + 0.3)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - G["foot"], 0, z - 0.7), (x + G["foot"], 0.6, z + G["foot_len"])))  # the long hind foot
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


def rotm(rot):
    if not rot:
        return np.eye(3)
    rx, ry, rz = (math.radians(a) for a in rot)
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


def area(e, face, p, uu, vv, h):
    """A wood mouse's coat: warm sandy brown, darker along the back, sharply grey-white below;
    bare pinkish feet, a tail dark above and pale below, thin ears pinkish inside."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, FUR)
    side = face in ("east", "west")
    if name in ("hips", "rump", "chest", "neck", "skull", "muzzle"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        else:
            a = np.where(y < (2.1 if name != "muzzle" else 2.5), BELLY, FUR)
            if name == "chest" and face == "south":
                a = np.where(y < 2.4, BELLY, FUR)
    elif name.startswith("tail_"):  # dark above, pale below, as a wood mouse's
        a[:] = BELLY if face == "down" else BACK
        if side:
            a = np.where(vv > h / 2, BELLY, BACK)
    elif name.startswith("ear_core"):
        a[:] = FUR
        if face == "south":
            a[:] = EARIN
    elif name.startswith("thigh_"):
        a[:] = FUR
        if face == "down":
            a[:] = BELLY
    elif name.startswith("leg_"):
        a[:] = PINK if name.endswith("_paw") or y.max() < 1.0 else FUR
        if face == "down" and not name.endswith("_paw"):
            a[:] = BELLY
    return a


def cut_mask(e, p):
    """The ear's outline: round, a little taller than wide."""
    x, y = p[..., 0], p[..., 1]
    cx = e["origin"][0]
    return ((x - cx) / 0.95) ** 2 + ((y - 4.6) / 0.95) ** 2 <= 1.0


# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}  # a layers.Recorder while build() paints, and the face's key


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)


def layer_props(e, face, p, a):
    """White where the belly is (and up from it), the tail's tip and the feet; the back's darker
    stripe is the pattern's place (an agouti coat has none to speak of)."""
    n, y = e["name"], p[..., 1]
    lo, hi = e["lo"][1], e["hi"][1]
    low = 1 - _ramp(y, lo, max(hi, lo + 1e-3))
    white = 0.6 * low if n in ("chest", "hips", "rump", "skull") else np.full(y.shape, 0.3)
    white = np.maximum(white, np.where(a == BELLY, 0.85, 0))
    pattern = np.full(y.shape, 1.0 if n in ("hips", "rump", "chest", "skull") else 0.3)
    return {"white": white, "pattern": pattern}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
    p = corner + uu[..., None] * du * (abs((np.array(e["hi"]) - np.array(e["lo"])) @ du) / w) \
        + vv[..., None] * dv * (abs((np.array(e["hi"]) - np.array(e["lo"])) @ dv) / h)
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    light_face = face
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "tail")):
        light_face = "north"  # seen when a pose lifts or folds it
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    stacked = e["name"].startswith("ear_core")  # the ear's boxes, lit as one piece
    down = np.clip((5.4 - p[..., 1]) / 1.8, 0, 1) if stacked else None
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT, y_world=p[..., 1] if not e["name"].startswith("leg_") else None,
                            occ=occ, quad=bool(e["quad"]), seed=sum(map(ord, e["name"] + face)) % 97, down=down, tuft=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"] == "ear":
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "skull" and face in ("east", "west"):
        (x0, y0, z0), (x1, y1, z1) = SKULL
        cu = (z1 - EYE_Z) if face == "east" else (EYE_Z - z0)
        stamp_eyes(img, [(cu, y1 - EYE_Y)], EYE_STYLE)
    if e["name"] == "muzzle" and face == "south":  # the pink nose at the tip, the mouth a line below
        c = w // 2
        img[0, max(0, c - 1):c + 1, :3] = BASE[NOSE]
        img[h - 1, :, :3] = (img[h - 1, :, :3] * 0.8).astype(np.uint8)
    if e["name"].endswith("_paw") and face == "south":  # tiny dark claws
        img[h - 1, :, :3] = BASE[CLAW]
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"sit_angle": 60, "sit_hip": -60, "sit_knee": 120, "sit_drop": 1.0, "sit_front": -70}


def tail_swing(k, angle, base=(0, 0, 0)):
    pitch = math.radians(TAIL[k][0])
    return swing((0, math.sin(pitch), -math.cos(pitch)), angle, base)


def gait(p, phases, swing_amp, lift, hind=None):
    """Scurrying: a foot swings forward while it's lifted, back while it's down."""
    out = {}
    for leg, ph in zip(LEGS, phases):
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))
        out[f"leg_{leg}_upper"] = (-swing_amp * S(a), 0, 0)
        knee = lift * up
        if leg[0] == "F":
            out[f"leg_{leg}_paw"] = (-knee, 0, 0)  # the toes curl up
        else:
            sw = out[f"leg_{leg}_upper"][0]
            knee *= G["hind"] if hind is None else hind  # the longer hind leg folds more
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (knee, 0, 0), (-knee - sw, 0, 0)  # the hock folds back, the foot flat
    return out


def tail_sway(p, n, amp):
    return {f"tail_{k}": tail_swing(k, amp * S(TAU * p * n - 0.8 * k)) for k in range(len(TAIL))}


def pose_idle(t):
    """Sniffing: quick nods and turns of the head, the ears turning, the tail swaying (3 s)."""
    p = t / 3.0
    sniff = 3 * max(0.0, S(TAU * t * 6)) * (0.3 < p < 0.6)
    turn = float(np.interp(p, [0, .15, .25, .6, .7, 1], [0, 0, 10, 10, -8, 0]))
    tw = max(0.0, S(TAU * (p - 0.75) * 10)) if 0.75 < p < 0.85 else 0.0
    q = {"root": (0, 0.06 * S(TAU * p * 4), 0), "spine_1": (0.6 * S(TAU * p * 4), 0, 0),
         "head": (sniff + 4 * S(TAU * p), turn, 0), "ear_L": (0, 0, -20 * tw), "ear_R": (0, 0, 4 * S(TAU * p * 2))}
    q.update(tail_sway(p, 1, 8))
    return q


def pose_walk(t):
    p = t / 0.4
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 26, 45)  # tiny legs: a big swing and lift
    q.update({"root": (0, 0.15 * abs(S(TAU * p)), 0), "head": (2 * S(TAU * p * 2), 0, 0)})
    q.update(tail_sway(p, 1, 5))
    return q


def pose_run(t):
    """A bounding dash, the tail streaming behind (0.24 s)."""
    p = t / 0.24
    q = gait(p, (0.0, 0.08, 0.5, 0.58), 40, 70, hind=0.9)
    c = S(TAU * p)
    q.update({"root": (0, 0.4 * max(0.0, S(TAU * p + 0.6)), 0), "spine_0": (-5 * c, 0, 0), "spine_1": (7 * c, 0, 0),
              "ear_L": (-25, 0, 0), "ear_R": (-25, 0, 0), "tail_0": tail_swing(0, 0, (-6, 0, 0))})
    return q


def pose_sit(t, P=L):
    """Up on its haunches, the front paws held to the mouth, nibbling (3 s)."""
    p = t / 3.0
    a = P["sit_angle"]
    nib = 4 * max(0.0, S(TAU * t * 5))
    q = {"root": (0, -P["sit_drop"] + 0.05 * S(TAU * p * 3), 0), "spine_0": (-a, 0, 0), "head": (a * 0.5 + nib, 3 * S(TAU * p), 0)}
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (a + P["sit_hip"], 0, 0)
        q[f"leg_{leg}_lower"] = (P["sit_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["sit_hip"] - P["sit_knee"], 0, 0)
    for leg, s in (("FL", 1), ("FR", -1)):
        q[f"leg_{leg}_upper"] = (P["sit_front"] + nib, 0, s * 8)
        q[f"leg_{leg}_paw"] = (40, 0, 0)
    for k in range(len(TAIL)):  # the tail laid out behind on the ground
        q[f"tail_{k}"] = (a * 0.9 if k == 0 else 4, 0, 0)
    return q


ANIMS = {"idle": (pose_idle, 3.0), "walk": (pose_walk, 0.4), "run": (pose_run, 0.24), "sit": (pose_sit, 3.0)}


def frame_times(name, dur):
    n = {"idle": 72, "sit": 60, "walk": 20, "run": 20}.get(name, 16)
    return sorted({round(dur * k / n, 4) for k in range(n + 1)})


def animations(bone_names):
    out = {}
    for name, (fn, dur) in ANIMS.items():
        frames = []
        for t in frame_times(name, dur):
            tt = t % dur if t < dur else 0.0
            pose = fn(tt)
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames}
    return out


# ------------------------------------------------------------------ fitting poses to the ground

GROUPS = {"front paws": lambda n: n in ("leg_FL_paw", "leg_FR_paw"), "back paws": lambda n: n in ("leg_BL_paw", "leg_BR_paw"),
          "rump": lambda n: n in ("hips", "rump"), "tail": lambda n: n.startswith("tail_"),
          "front legs": lambda n: n.startswith("leg_F"), "body": lambda n: n in ("hips", "rump", "chest", "skull", "muzzle")}


def fit_poses(spec):
    """Sitting up: the hind feet and the rump on the ground, the tail laid behind, nothing below."""
    def cost(P):
        g = lowest(spec, pose_sit(0.0, dict(L, **P, sit_drop=0)), GROUPS)
        f = min(g["back paws"], g["rump"])
        return (g["back paws"] - f) ** 2 + (g["rump"] - f) ** 2 + 4 * max(0.0, f - g["tail"]) ** 2 + 2 * max(0.0, f + 0.3 - g["front legs"]) ** 2
    grid = {"sit_angle": list(range(40, 81, 5)), "sit_hip": list(range(-100, -29, 5)), "sit_knee": list(range(60, 161, 10))}
    best = fit(grid, cost)[1]
    g = lowest(spec, pose_sit(0.0, dict(L, **best, sit_drop=0)), GROUPS)
    L.update(best, sit_drop=round(float(min(g["back paws"], g["rump"])), 2))


# ------------------------------------------------------------------ build


def build():
    B = bones()
    E = elements()
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"] and not e["rot"]]
    LAYERS["rec"] = Recorder(layer_bounds(E, 1.0))
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
    spec = {"name": "mouse", "texture": [sheet.shape[1], sheet.shape[0]],
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
