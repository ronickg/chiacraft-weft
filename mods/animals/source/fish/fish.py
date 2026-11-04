#!/usr/bin/env python3
"""The fish: one model for the brown trout and the pacu, made the way Hytale makes creatures (boxes,
cut-out fins and a hand-painted texture) with the blocky-model skill, from research alone
(README.md): its colors are rules (see `area`), painted in a brown trout's.

    ~/.venvs/blocky/bin/python fish.py      # writes out/spec.json, out/texture.png

Units are the house's (64 a block, 1.56 cm each), painted `TX` texels a unit (64 a block); y up, the fish faces +z. A
river brown trout is about 35 cm long: here 24 from the snout to the tail's tips, the head a little
bigger than life. The pacu is the same model drawn deeper and bigger by its genes (`build` "Gt" and
the size gene in `genetics/pacu.json`).
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
from ground import lowest  # noqa: E402
from house import SCALE, fine, texels  # noqa: E402

# Texels a unit. The eye sits on the head's side, 5 units by 4.2: at the house's 32 a block that is
# 2.5 by 2.1 texels, under `house.MIN_FACE` (3), so it paints at twice that (k = 2, 64 a block): 5 by 4.
# (It was `house.FINE`, 4 a unit, 256 a block.)
TX = texels(SCALE, fine(4.2))

OUT = os.environ.get("FISH_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

FLANK, BACK, BELLY, FIN, SPOT, RED, MOUTH, GILL = range(8)
# A brown trout's: FLANK golden bronze, BACK olive brown, BELLY creamy yellow, FIN the fins'
# brownish olive, SPOT its black spots, RED its red ones (ringed pale), MOUTH inside, GILL the gill
# cover's edge.
BASE = [(176, 146, 86), (96, 88, 58), (228, 214, 168), (128, 108, 70), (46, 38, 34), (176, 64, 48), (150, 96, 92), (120, 98, 64)]
if os.environ.get("FISH_COAT") == "pacu":  # a pacu's colors, to compare in the viewer (the game paints them from genes)
    BASE[FLANK], BASE[BACK], BASE[BELLY], BASE[FIN] = (150, 150, 146), (72, 74, 76), (214, 132, 70), (64, 62, 64)
SHADOW = [0.02, 0.05, 0.06, 0.02, 0.95, 0.97, 0.97, 0.02]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (FLANK, BACK, BELLY, FIN)
ROLES = [("coat", 0, 0), ("coat", 0, -1.5), ("coat", 1, 0), ("coat", 0.2, -1.0), ("coat", 0, -3.0), ("coat", 0.5, 0), "fixed", ("coat", 0, -0.5)]
SPOTS = os.environ.get("FISH_COAT") != "pacu"
MOUTH_LINE = (96, 60, 58)

# A fish's round eye: a gold ring round a big black pupil, a catchlight toward the front and up
# with a dimmer gloss by it (no eyelids: fish have none). `fish_gold` is the old one, at 256 a block.
EYES.update({
    "fish_gold": dict(pattern=[".AAAA.",
                               "AHhPPA",
                               "AhPPPA",
                               "APPPPA",
                               "aPPPPa",
                               ".aaaa."], colors={"P": (16, 12, 12), "H": (222, 226, 230), "h": (86, 88, 96), "A": (176, 140, 70),
                                                  "a": (206, 172, 96)}, halo="none"),
})
# At 64 a block the head's side is 5 texels by 4: the eye is 2 by 2, written for the east side
# (the snout to the left): a catchlight up and forward, the black pupil, the gold iris below and
# behind it (recolored per fish). The west side gets it mirrored (`eye_pattern`).
_GOLD, _gold, _PUP, _HI = (176, 140, 70), (206, 172, 96), (16, 12, 12), (222, 226, 230)
EYES.update({
    "fish_px": dict(pattern=["HP", "Pa"], colors={"P": _PUP, "H": _HI, "a": _gold}, halo="none", iris="a"),
    "fish_px_gold": dict(pattern=["HP", "aA"], colors={"P": _PUP, "H": _HI, "A": _GOLD, "a": _gold}, halo="none",
                         iris="Aa"),
    "fish_px_ring": dict(pattern=["PA", "Aa"], colors={"P": _PUP, "A": _GOLD, "a": _gold}, halo="none", iris="Aa"),
    "fish_px_lit": dict(pattern=["HA", "Pa"], colors={"P": _PUP, "H": _HI, "A": _GOLD, "a": _gold}, halo="none",
                        iris="Aa"),
})
EYE_STYLE = os.environ.get("FISH_EYES", "fish_px")
HEAD = ((-1.4, 2.4, 4.5), (1.4, 6.6, 9.5))
# The eye in the side's middle rows (5.1 at 256 a block): at 4 rows high it would touch the back's edge.
EYE_Z, EYE_Y = 7.4, 4.6


def side_grid():
    """The head side's texels: (width, height) and one texel's size in units (u, v)."""
    (x0, y0, z0), (x1, y1, z1) = HEAD
    w, h = max(1, round(TX * (z1 - z0))), max(1, round(TX * (y1 - y0)))
    return w, h, (z1 - z0) / w, (y1 - y0) / h


def eye_center(side):
    """The eye's center on the head's east (side > 0) or west side, in that face's texels."""
    (x0, y0, z0), (x1, y1, z1) = HEAD
    w, h, su, sv = side_grid()
    return ((z1 - EYE_Z) if side > 0 else (EYE_Z - z0)) / su, (y1 - EYE_Y) / sv


def eye_pattern(side):
    """The design for one side: as drawn on the east side, mirrored on the west (its u runs tail
    to snout), so the catchlight is toward the snout on both."""
    d = dict(EYES[EYE_STYLE])
    if side < 0:
        d["pattern"] = [row[::-1] for row in d["pattern"]]
    return d

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


def bones():
    B = [("root", None, (0, 0, 0)), ("body", "root", (0, 4.5, 1.0)), ("head", "body", (0, 4.5, 4.8)),
         ("tail_0", "body", (0, 4.5, -1.7)), ("tail_1", "tail_0", (0, 4.5, -6.8))]
    for s, n in ((1, "L"), (-1, "R")):
        B.append((f"fin_{n}", "body", (s * 1.7, 3.0, 3.2)))
    return B


def elements():
    E = [el("body", "body", (-1.8, 2.0, -2.0), (1.8, 7.0, 5.0)),
         el("head", "head", *HEAD),
         el("snout", "head", (-1.0, 2.9, 9.1), (1.0, 5.6, 10.6)),
         el("tail_0", "tail_0", (-1.4, 2.6, -7.2), (1.4, 6.4, -0.4)),  # reaching well into the body
         el("tail_1", "tail_1", (-0.9, 3.4, -10.6), (0.9, 5.6, -6.6)),
         # The fins: cut-out panels through the middle of the body (double-sided).
         el("caudal", "tail_1", (0, 1.4, -14.4), (0, 7.6, -9.8), quad="east", cut="caudal"),
         el("dorsal", "body", (0, 6.6, -1.6), (0, 9.6, 3.4), quad="east", cut="dorsal"),
         el("adipose", "tail_0", (0, 6.0, -6.6), (0, 7.4, -5.0), quad="east", cut="adipose"),
         el("anal", "tail_0", (0, 0.9, -6.4), (0, 3.0, -3.2), quad="east", cut="anal"),
         el("pelvic", "body", (0, 0.9, -1.2), (0, 2.4, 1.4), quad="east", cut="anal")]
    for s, n in ((1, "L"), (-1, "R")):
        x0, x1 = sorted((s * 1.7, s * 4.0))
        E.append(el(f"fin_{n.lower()}", f"fin_{n}", (x0, 3.0, 1.6), (x1, 3.0, 4.2), quad="down", rot=(0, 0, -s * 24),
                    origin=(s * 1.7, 3.0, 3.2), cut="pectoral"))
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


SPOT_R = 0.55  # texels: a spot's least radius (0.42 units at 256 a block)


def spots(p, density, size, seed):
    """Fixed round spots in 3D (so they run over edges), as a trout's are scattered."""
    rng = np.random.default_rng(seed)
    pts = np.c_[rng.uniform(-2, 2, 60), rng.uniform(2, 7.5, 60), rng.uniform(-11, 10, 60)][: int(60 * density)]
    d = np.min(np.linalg.norm(p[..., None, :] - pts[None, None], axis=-1), axis=-1)
    return d < max(size, SPOT_R / TX)  # at least a texel, or most spots fall between texel centers


def area(e, face, p, uu, vv, h):
    """A brown trout: an olive-brown back, golden-bronze flanks, a creamy belly, black spots over
    the back and flanks and red ones along the side, olive fins (the dorsal and adipose spotted)."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, FLANK)
    if e["quad"]:
        a[:] = FIN
        if SPOTS and name in ("dorsal", "adipose"):
            a = np.where(spots(p, 0.5, 0.45, 3), SPOT, a)
        if name == "adipose" and SPOTS:
            a = np.where(y > 6.6, RED, a)
        return a
    if face == "up":
        a[:] = BACK
    elif face == "down":
        a[:] = BELLY
    else:
        a = np.where(y > 5.8, BACK, np.where(y < 3.3, BELLY, FLANK))
    if SPOTS and name in ("body", "tail_0", "head") and face != "down":
        a = np.where(spots(p, 1.0, 0.42, 1) & (y > 3.6), SPOT, a)
        a = np.where(spots(p, 0.5, 0.5, 2) & (y > 3.4) & (y < 5.4) & (face in ("east", "west")), RED, a)
    if name == "head" and face in ("east", "west"):  # the gill cover's back edge, a texel wide
        a = np.where((np.abs(z - 5.2) < max(0.45, 0.5 / TX)) & (y < 6.0), GILL, a)
    return a


def cut_mask(e, p):
    """The fins' outlines: the tail a broad fan with a shallow fork, the dorsal a sail sloping
    back, the adipose a small round lobe, the anal and pelvic short fans, the pectorals paddles."""
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    kind = e["cut"]
    if kind == "caudal":
        t = np.clip((-9.8 - z) / 4.6, 0, 1)  # 0 at the stalk, 1 at the tips
        half = 1.2 + 2.0 * t
        fork = np.abs(y - 4.5) < 1.2 * np.clip((t - 0.65) / 0.35, 0, 1)
        return (np.abs(y - 4.5) <= half) & ~fork
    if kind == "dorsal":
        top = 6.6 + 3.0 * np.clip((z + 1.6) / 3.0, 0, 1) * np.clip((3.4 - z) / 2.0 + 0.2, 0, 1)
        return y <= top
    if kind == "adipose":
        return ((z + 5.8) / 0.8) ** 2 + ((y - 6.0) / 1.4) ** 2 <= 1
    if kind == "anal":
        lo, hi = e["lo"], e["hi"]
        zc = (lo[2] + hi[2]) / 2
        return np.abs(z - zc) <= (hi[2] - lo[2]) / 2 * (0.4 + 0.6 * np.clip((hi[1] - y) / (hi[1] - lo[1]), 0, 1))
    if kind == "pectoral":
        r = np.hypot(np.abs(x) - 1.7, (z - 3.2) * 1.3)
        return r <= 2.3
    return np.ones(x.shape, bool)


# ------------------------------------------------------------------ layers (coats painted on the fly)

LAYERS = {"rec": None, "key": None}


def layer_props(e, face, p, a):
    """White (silver, pale) up from the belly; spots and marks over the back and flanks and the
    dorsal fin."""
    y = p[..., 1]
    white = np.clip((3.6 - y) / 2.0, 0, 1) * 0.6
    white = np.maximum(white, np.where(a == BELLY, 0.8, 0))
    pattern = np.where((y > 3.2) & ~np.isin(a, (MOUTH,)), 1.0, 0.2) if not e["quad"] else np.full(y.shape, 0.6 if e["name"] == "dorsal" else 0.0)
    return {"white": white, "pattern": pattern}


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    size = np.array(e["hi"]) - np.array(e["lo"])
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel indices (+0.5)
    p = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    cap = {}
    col = paint_face_smooth(a, TONES, face, out=cap, coat=COAT, y_world=None, occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=False, merge=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"]:
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "head" and face in ("east", "west"):
        side = 1 if face == "east" else -1
        EYES["_fish_side"] = eye_pattern(side)
        stamp_eyes(img, [eye_center(side)], "_fish_side")
    if e["name"] == "snout" and face in ("south", "east", "west"):  # the mouth: a dark line, 2 units down
        r = int(round(2.0 / sv))  # its row, a texel index
        img[min(r, h - 1), :, :3] = BASE[MOUTH] if h >= 6 else MOUTH_LINE  # a texel-high line: the dark gap
    if e["quad"] and e["cut"] in ("caudal", "dorsal", "anal", "pectoral"):  # rays along the fin
        # A dark ray every third texel (texel indices: across the caudal's rows, along the others'
        # columns); a fin under 4 texels that way is too small for them.
        idx = vv if e["cut"] == "caudal" else uu
        if idx.max() >= 3.5:
            ray = (np.floor(idx) % 3 == 1)[..., None]
            img[..., :3] = np.where(ray, (img[..., :3] * 0.86).astype(np.uint8), img[..., :3])
    if LAYERS["rec"] is not None:
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, np.clip(np.round(col), 0, 255), img, p,
                           layer_props(e, face, p, a))
    return img


# ------------------------------------------------------------------ animation

L = {"flop_drop": 2.7}


def sway(p, amp, n=1.0):
    """The body's wave: small at the head, growing down the tail, each part behind the one before."""
    return {"head": (0, -0.25 * amp * S(TAU * p * n + 0.6), 0), "body": (0, 0.25 * amp * S(TAU * p * n), 0),
            "tail_0": (0, amp * S(TAU * p * n - 0.8), 0), "tail_1": (0, 1.4 * amp * S(TAU * p * n - 1.6), 0)}


def fins(p, amp, n=2.0):
    return {"fin_L": (0, 0, amp * S(TAU * p * n)), "fin_R": (0, 0, -amp * S(TAU * p * n))}


def pose_idle(t):
    """Holding in the current: a slow tail beat, the pectoral fins sculling (2 s)."""
    p = t / 2.0
    q = sway(p, 8)
    q.update(fins(p, 14))
    q["root"] = (0, 0.15 * S(TAU * p), 0)
    return q


def pose_walk(t):
    """Swimming: a full tail beat (0.6 s)."""
    p = t / 0.6
    q = sway(p, 16)
    q.update(fins(p, 8, 1.0))
    return q


def pose_run(t):
    """Darting: a quick, strong beat, the fins laid back (0.3 s)."""
    p = t / 0.3
    q = sway(p, 24)
    q.update({"fin_L": (0, 0, 18), "fin_R": (0, 0, -18)})
    return q


def pose_flop(t, P=L):
    """Stranded: on its side, the tail slapping (0.8 s)."""
    p = t / 0.8
    c = S(TAU * p)
    # On its side the tail bends about its own x (the body rolled, x now points up): along the ground.
    q = {"root": (0, -P["flop_drop"] + 0.6 * max(0.0, c), 0), "body": (0, 10 * c, 88), "tail_0": (22 * c, 0, 0),
         "tail_1": (30 * S(TAU * p - 0.7), 0, 0)}
    return q


ANIMS = {"idle": (pose_idle, 2.0), "walk": (pose_walk, 0.6), "run": (pose_run, 0.3), "flop": (pose_flop, 0.8)}


def frame_times(name, dur):
    n = {"idle": 40, "walk": 24, "run": 20, "flop": 24}.get(name, 16)
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


def fit_poses(spec):
    """Stranded on its side, resting on the ground."""
    g = lowest(spec, pose_flop(0.0, dict(L, flop_drop=0)), {"all": lambda n: not n.startswith(("dorsal", "caudal", "anal", "pelvic", "adipose", "fin_"))})
    L["flop_drop"] = round(float(g["all"]), 2)


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
    spec = {"name": "fish", "texture": [sheet.shape[1], sheet.shape[0]],
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
