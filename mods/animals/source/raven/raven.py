#!/usr/bin/env python3
"""The common raven, made the way Hytale makes creatures (boxes, cut-out feather panels and a
hand-painted texture) with the blocky-model skill, from research alone (README.md).

    ~/.venvs/blocky/bin/python raven.py      # writes out/spec.json (on the ground), out/spec_fly.json
                                            # (flying) and out/texture.png

Units are texels at the house density (64 a block, 1.56 cm each), y up, the raven faces +z. A raven
is about 63 cm from the bill to the tail's tip with a 1.2 m span: here 40 long and 76 across the
open wings, the head a little bigger than life. One build paints both of its models on one texture:
standing with the wings folded along its sides, and flying with them open (`raven_fly`, a look of
its own that `raven.rs` puts on while it flies), so no wing has to fold through the body.
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
from ground import lowest  # noqa: E402

OUT = os.environ.get("RAVEN_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

BLACK, GLOSS, DULL, BILL = range(4)
# BLACK the plumage, GLOSS its blue-purple sheen on the back, wings and crown, DULL the softer
# black of the belly and under the wings, BILL the bill, legs and feet.
BASE = [(32, 31, 38), (46, 48, 74), (28, 27, 32), (26, 24, 26)]
SHADOW = [0.72, 0.7, 0.72, 0.74]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
FEATHER = (BLACK, GLOSS, DULL)
ROLES = [("coat", 0, 0), ("coat", 0, 1.0), ("coat", 0.4, -0.5), "fixed"]

# A raven's eye: dark brown, a bright catchlight (an all-black bird needs one to have a face).
EYES.update({
    "raven": dict(pattern=["IH",
                           "II"], colors={"I": (58, 40, 30), "H": (232, 230, 224)}, halo="none"),
})
EYE_STYLE = os.environ.get("RAVEN_EYES", "raven")
SKULL = ((-2.6, 15.5, 8.5), (2.6, 20.5, 14.0))
EYE_Z, EYE_Y = 12.4, 18.6

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


NECK0 = (0.0, 13.6, 6.6)
HEAD = (0.0, 17.0, 9.6)


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 12.5, -2.0)), ("spine_1", "spine_0", (0, 12.5, 0.6)),
         ("neck", "spine_1", NECK0), ("head", "neck", HEAD), ("jaw", "head", (0, 16.6, 13.6)), ("tail", "spine_0", (0, 13.2, -6.4))]
    for s, n in ((1, "L"), (-1, "R")):
        B += [(f"wingf_{n}", "spine_1", (s * 3.9, 14.6, 5.0)),  # folded
              (f"wing_{n}", "spine_1", (s * 3.6, 14.9, 5.0)), (f"hand_{n}", f"wing_{n}", (s * 17.8, 14.9, 5.0)),  # open
              (f"leg_{n}_thigh", "spine_0", (s * 1.9, 10.0, -0.6)), (f"leg_{n}_shin", f"leg_{n}_thigh", (s * 1.9, 6.4, -0.6)),
              (f"leg_{n}_toes", f"leg_{n}_shin", (s * 1.9, 1.0, -0.6))]
    return B


def elements():
    E = [el("hips", "spine_0", (-3.4, 9.0, -7.2), (3.4, 16.0, 1.4)),
         el("chest", "spine_1", (-4.0, 8.4, -0.4), (4.0, 16.6, 8.4)),
         el("neck", "neck", (-2.0, 12.4, 5.0), (2.0, 18.2, 10.4)),  # the shaggy throat, deep in the head
         el("skull", "head", *SKULL),
         el("bill", "head", (-1.0, 16.7, 13.5), (1.0, 19.0, 19.2), rot=(6, 0, 0), origin=(0, 17.8, 13.5)),  # big, a little curved
         el("jaw", "jaw", (-0.8, 15.9, 13.2), (0.8, 16.9, 18.0)),  # shorter than the bill
         el("tail", "tail", (-2.2, 12.6, -19.6), (2.2, 13.8, -5.0)),
         el("tail_fan", "tail", (-3.6, 13.2, -21.0), (3.6, 13.2, -8.0), quad="up", cut="wedge")]
    for s, n in ((1, "L"), (-1, "R")):
        x0, x1 = sorted((s * 3.6, s * 4.9))
        # Folded along the side, the primaries crossing over the tail's base.
        E.append(el(f"wingf_{n.lower()}", f"wingf_{n}", (x0, 10.6, -6.0), (x1, 15.9, 7.0)))
        px = s * 4.3
        E.append(el(f"primariesf_{n.lower()}", f"wingf_{n}", (px, 10.4, -15.0), (px, 14.4, -4.0), quad="east" if s > 0 else "west",
                    cut="folded"))
        # Open: the arm, the hand, and the flight feathers behind them as one cut-out panel.
        ax0, ax1 = sorted((s * 3.2, s * 18.4))
        E.append(el(f"wing_{n.lower()}", f"wing_{n}", (ax0, 14.0, 1.8), (ax1, 15.8, 8.0)))
        hx0, hx1 = sorted((s * 17.4, s * 38.0))
        E.append(el(f"hand_{n.lower()}", f"hand_{n}", (hx0, 14.45, 2.2), (hx1, 15.35, 7.0)))
        fx0, fx1 = sorted((s * 3.6, s * 17.4))
        E.append(el(f"flight_{n.lower()}", f"wing_{n}", (fx0, 14.9, -7.5), (fx1, 14.9, 2.4), quad="down", cut="secondaries"))
        gx0, gx1 = sorted((s * 17.6, s * 39.0))
        E.append(el(f"primaries_{n.lower()}", f"hand_{n}", (gx0, 14.9, -6.5), (gx1, 14.9, 2.8), quad="down", cut="primaries"))
        x = s * 1.9
        E.append(el(f"thigh_{n.lower()}", f"leg_{n}_thigh", (x - 1.1, 5.9, -2.0), (x + 1.1, 11.0, 0.8)))  # feathered
        E.append(el(f"shin_{n.lower()}", f"leg_{n}_shin", (x - 0.5, 0.8, -1.1), (x + 0.5, 6.6, -0.1)))
        E.append(el(f"toes_{n.lower()}", f"leg_{n}_toes", (x - 1.1, 0.0, -2.6), (x + 1.1, 1.2, 2.6)))
    return E


FOLDED = lambda n: n.startswith(("wingf_", "primariesf_"))  # noqa: E731
OPEN = lambda n: n.startswith(("wing_", "hand_", "flight_", "primaries_"))  # noqa: E731

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


def area(e, face, p, uu, vv, h):
    """All black: the sheen on top (back, crown, the folded and open wings' upper faces), duller
    below; the bill, legs and feet a harder black."""
    name = e["name"]
    a = np.full(p.shape[:2], BLACK)
    if name in ("bill", "jaw") or name.startswith(("shin_", "toes_")):
        a[:] = BILL
        return a
    if face == "up" or (e["quad"] and name.startswith(("primaries", "flight"))):
        a[:] = GLOSS
    elif face == "down":
        a[:] = DULL
    if name in ("hips", "chest", "neck") and face in ("east", "west", "south"):
        a = np.where(vv < h * 0.3, GLOSS, a)
    return a


def cut_mask(e, p):
    """Feather outlines: the open wing's fingered primaries, the secondaries' scalloped trailing
    edge, the folded primaries' tapering tips, the tail's wedge."""
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    kind = e["cut"]
    if kind == "wedge":
        t = np.clip((-8.0 - z) / 13.0, 0, 1)  # along the tail, 1 at the tip
        return np.abs(x) <= 2.0 + 1.4 * np.sin(np.pi * np.clip(t * 0.85, 0, 1)) - 1.6 * np.clip(t - 0.8, 0, 1) * 5
    if kind == "secondaries":
        scallop = 0.7 * np.abs(np.sin(np.abs(x) * 1.3))
        return z >= -7.5 + 2.0 * np.clip((np.abs(x) - 12.0) / 6.0, 0, 1) + scallop  # broad by the body
    if kind == "primaries":
        ax = np.abs(x)
        t = np.clip((ax - 17.6) / 21.4, 0, 1)
        trail = -5.5 + 3.0 * t + 0.6 * np.abs(np.sin(ax * 1.1))
        # The fingers: gaps between the outermost feathers near the tip.
        fingers = (t > 0.72) & (np.floor((z + 6.5) / 1.6) % 2 == 1) & (ax > 39.0 - 5.0 * (z + 6.5) / 9.3)
        tip = ax <= 39.0 - 3.0 * np.clip((z - 1.0) / 1.8, 0, 1)
        return (z >= trail) & tip & ~fingers
    if kind == "folded":
        t = np.clip((-4.0 - z) / 11.0, 0, 1)
        return np.abs(y - 12.0) <= 2.0 * (1 - 0.6 * t)
    return np.ones(x.shape, bool)


LAYERS = {"rec": None, "key": None}


def layer_props(e, face, p, a):
    """A raven barely varies: its sheen and how black; a leucistic one is white all over."""
    return {"white": np.full(p.shape[:2], 0.2), "pattern": np.zeros(p.shape[:2])}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    size = np.array(e["hi"]) - np.array(e["lo"])
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
    p = corner + uu[..., None] * du * (abs(size @ du) / w) + vv[..., None] * dv * (abs(size @ dv) / h)
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] or OPEN(e["name"]) else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, face, out=cap, coat=FEATHER, y_world=None, occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"]:
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "skull" and face in ("east", "west"):
        (x0, y0, z0), (x1, y1, z1) = SKULL
        cu = (z1 - EYE_Z) if face == "east" else (EYE_Z - z0)
        stamp_eyes(img, [(cu, y1 - EYE_Y)], EYE_STYLE)
    if e["quad"] and e["cut"] in ("primaries", "secondaries", "folded", "wedge"):  # the feathers' shafts and edges
        along = np.abs(p[..., 0]) if e["cut"] in ("primaries", "secondaries") else p[..., 2]
        img[..., :3] = np.where((np.floor(along * 0.9) % 2 == 0)[..., None], (img[..., :3] * 0.86).astype(np.uint8), img[..., :3])
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"drop": 0.0, "tilt": -18}


def stand(P=L):
    """Standing: the body tipped up at the front, the head held level, the legs straight under it."""
    t = P["tilt"]
    q = {"root": (0, -P["drop"], 0), "spine_0": (t, 0, 0), "neck": (-t * 0.4, 0, 0), "head": (-t * 0.6, 0, 0), "tail": (-t * 0.8, 0, 0)}
    for n in ("L", "R"):
        q[f"leg_{n}_thigh"] = (-t, 0, 0)
        q[f"leg_{n}_shin"] = (0, 0, 0)
        q[f"leg_{n}_toes"] = (0, 0, 0)
    return q


def add(q, more):
    for k, v in more.items():
        a = q.get(k, (0, 0, 0))
        q[k] = tuple(a[i] + v[i] for i in range(3))
    return q


def pose_idle(t):
    """Looking about in holds and snaps, the throat working, a tail flick (3 s)."""
    p = t / 3.0
    look = float(np.interp(p, [0, .2, .23, .5, .53, .78, .81, 1], [0, 0, 18, 18, -14, -14, 0, 0]))
    tilt = float(np.interp(p, [0, .5, .53, .78, .81, 1], [0, 0, 4, 4, 0, 0]))
    flick = max(0.0, S(TAU * (p - 0.6) * 8)) if 0.6 < p < 0.66 else 0.0
    q = stand()
    add(q, {"root": (0, 0.08 * S(TAU * p * 2), 0), "head": (0, look, tilt), "tail": (-12 * flick, 0, 0)})
    return q


def pose_walk(t):
    """Walking with a swagger, the head bobbing (0.7 s)."""
    p = t / 0.7
    a = TAU * p
    q = stand()
    for n, ph in (("L", 0.0), ("R", 0.5)):
        b = TAU * (p + ph)
        up = max(0.0, S(b + math.pi / 2))
        add(q, {f"leg_{n}_thigh": (-26 * S(b), 0, 0), f"leg_{n}_shin": (24 * up, 0, 0), f"leg_{n}_toes": (26 * S(b) - 24 * up, 0, 0)})
    add(q, {"root": (0, 0.3 * abs(S(a)), 0), "spine_0": (0, 4 * S(a), 0), "neck": (5 * S(2 * a), 0, 0), "head": (-5 * S(2 * a), 0, 0)})
    return q


def pose_run(t):
    """Hopping along fast, both feet together (0.4 s)."""
    p = t / 0.4
    a = TAU * p
    hop = max(0.0, S(a))
    q = stand()
    for n in ("L", "R"):
        add(q, {f"leg_{n}_thigh": (-30 * S(a), 0, 0), f"leg_{n}_shin": (34 * hop, 0, 0), f"leg_{n}_toes": (30 * S(a) - 34 * hop, 0, 0)})
    add(q, {"root": (0, 1.4 * hop, 0), "tail": (-10 * hop, 0, 0)})
    return q


def pose_fly(t):
    """Flying: a deep, rowing wingbeat, the hand following the arm, the legs tucked back (0.5 s)."""
    p = t / 0.5
    c = S(TAU * p)
    q = {"spine_0": (2, 0, 0), "neck": (-4, 0, 0), "head": (4, 0, 0), "root": (0, 2.0 + 0.8 * c, 0)}
    for n, s in (("L", 1), ("R", -1)):
        q[f"wing_{n}"] = (0, 0, s * 34 * c)
        q[f"hand_{n}"] = (0, 0, s * 18 * S(TAU * p - 0.6))
        q[f"leg_{n}_thigh"] = (70, 0, 0)
        q[f"leg_{n}_toes"] = (40, 0, 0)
    return q


ANIMS = {"idle": (pose_idle, 3.0), "walk": (pose_walk, 0.7), "run": (pose_run, 0.4)}
FLY_ANIMS = {"flap": (pose_fly, 0.5)}


def frame_times(name, dur):
    n = {"idle": 48, "walk": 20, "run": 16, "flap": 20}.get(name, 16)
    return sorted({round(dur * k / n, 4) for k in range(n + 1)})


def animations(anims, bone_names):
    out = {}
    for name, (fn, dur) in anims.items():
        frames = []
        for t in frame_times(name, dur):
            tt = t % dur if t < dur else 0.0
            pose = fn(tt)
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames}
    return out


def fit_poses(spec):
    """Standing: the toes on the ground."""
    g = lowest(spec, stand(dict(L, drop=0)), {"toes": lambda n: n.startswith("toes_")})
    L["drop"] = round(float(g["toes"]), 2)


# ------------------------------------------------------------------ build


def build():
    B = bones()
    E = elements()
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"] and not e["rot"] and not OPEN(e["name"])]
    LAYERS["rec"] = Recorder(layer_bounds(E, 2.0))
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
    names = [n for n, _, _ in B]
    bones_json = [{"name": n, "parent": p, "pivot": [float(v) for v in pv]} for n, p, pv in B]
    ground = {"name": "raven", "texture": [sheet.shape[1], sheet.shape[0]], "bones": bones_json,
              "elements": [e for e in E if not OPEN(e["name"])]}
    fit_poses(ground)
    ground["animations"] = animations(ANIMS, names)
    ground["fitted"] = dict(L)
    fly = {"name": "raven_fly", "texture": ground["texture"], "bones": bones_json, "elements": [e for e in E if not FOLDED(e["name"])]}
    fly["animations"] = animations(FLY_ANIMS, names)
    return ground, fly, sheet


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    ground, fly, sheet = build()
    Image.fromarray(sheet, "RGBA").save(os.path.join(OUT, "texture.png"))
    for name, spec in (("spec.json", ground), ("spec_fly.json", fly)):
        with open(os.path.join(OUT, name), "w") as f:
            json.dump(spec, f)
    print(len(ground["elements"]), "elements on the ground,", len(fly["elements"]), "flying;", len(ground["bones"]), "bones; texture",
          sheet.shape[1], "x", sheet.shape[0], "; fitted", ground["fitted"])
