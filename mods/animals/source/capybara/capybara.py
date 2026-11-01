#!/usr/bin/env python3
"""The capybara: the world's largest rodent, calm, always chewing. Made the way Hytale makes
creatures (boxes and a hand-painted texture) with the blocky-model skill, from research alone
(README.md): its colors are rules (see `area`).

    ../chihuahua/.venv/bin/python capybara.py      # writes out/spec.json, out/texture.png

Units are Blockbench's (1 texel each), y up, the capybara faces +z. A real one is 50-62 cm at the
shoulder and 100-130 cm long: here the back is 25.5 up, the whole animal 54 long; the head 1.15x
(Hytale's chunky look).
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
from paint import EYES, occlusion, pack, paint_face_smooth, stamp_eyes, tones  # noqa: E402
from rig import fit  # noqa: E402
from ground import lowest  # noqa: E402

OUT = os.environ.get("CAPYBARA_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin

# ------------------------------------------------------------------ materials and eyes

FUR, BACK, BELLY, DARK, SKIN, MOUTH, TEETH, EARIN = range(8)
BASE = [(160, 104, 64), (124, 78, 48), (204, 162, 118), (54, 38, 32), (96, 72, 58), (84, 36, 40), (226, 212, 170),
        (150, 98, 84)]
SHADOW = [0.02, 0.02, 0.05, 0.0, 0.03, 0.95, 0.1, 0.98]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (FUR, BACK, BELLY)  # the coarse coat; the snout's patch, claws and mouth stay crisp

# Small dark eyes high on the sides of the head, a catchlight; calm.
EYES.update({"capybara": dict(pattern=[".DD.",
                                       "DHDD",
                                       "DDDD",
                                       ".DD."], colors={"D": (34, 22, 20), "H": (226, 220, 206)}, halo="round")})
EYE_STYLE = os.environ.get("CAPYBARA_EYES", "capybara")
SKULL = ((-6, 17, 15), (6, 28, 26))
EYE_Z, EYE_Y = 22.5, 25.0  # high on the head, toward the front
LID_X = 6.4  # the eyelids' hinge plane, just outside the head's side (x = 6)

# ------------------------------------------------------------------ the model


def el(name, bone, lo, hi, quad=None, rot=None, origin=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad, rot=rot,
                origin=[float(v) for v in origin] if origin else None, cut=None)


def eye_box(side):
    """(z0, z1, top, bottom) of the eye on one side, rounded as `stamp_eyes` places it."""
    pat = EYES[EYE_STYLE]["pattern"]
    ph, pw = len(pat), len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    cu = (z1 - EYE_Z) if side > 0 else (EYE_Z - z0)
    u0 = int(round(cu - pw / 2))
    v0 = int(round(y1 - EYE_Y - ph / 2))
    za, zb = (z1 - u0 - pw, z1 - u0) if side > 0 else (z0 + u0, z0 + u0 + pw)
    return za, zb, y1 - v0, y1 - v0 - ph


LEGS = ("FL", "FR", "BL", "BR")
FRONT_Z, BACK_Z, LEG_X = 11.0, -13.0, 5.5


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 17, -10)), ("spine_1", "spine_0", (0, 17, -2)),
         ("neck", "spine_1", (0, 20, 12)), ("head", "neck", (0, 22, 16.5)), ("jaw", "head", (0, 15.75, 25.75))]
    for s, n in ((1, "L"), (-1, "R")):
        za, zb, top, _ = eye_box(s)
        B += [(f"ear_{n}", "head", (s * 4.25, 28, 18.25)), (f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2))]
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        x = LEG_X * s
        if leg[0] == "F":
            B += [(f"leg_{leg}_upper", "spine_1", (x, 11, FRONT_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 5.5, FRONT_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 1.6, FRONT_Z))]
        else:
            B += [(f"leg_{leg}_upper", "spine_0", (x, 13, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 7, BACK_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 1.6, BACK_Z))]
    return B


def elements():
    E = [el("hips", "spine_0", (-8.5, 9, -20), (8.5, 25, -2)),
         el("rump", "spine_0", (-7.5, 11, -22.5), (7.5, 23, -19.5)),  # the round back end (no tail)
         el("chest", "spine_1", (-9, 8.5, -2.5), (9, 25.5, 14)),
         el("neck", "neck", (-6.5, 15, 11), (6.5, 25, 18)),
         el("skull", "head", *SKULL),
         el("muzzle", "head", (-5, 15.5, 25.5), (5, 24, 32)),  # long and blunt
         el("mouth", "head", (-3, 15, 25), (3, 17, 30.5)),
         el("jaw", "jaw", (-4, 14, 24), (4, 16, 31))]  # shorter than the muzzle
    for s, n in ((1, "L"), (-1, "R")):
        x0, x1 = (3, 5.5) if s > 0 else (-5.5, -3)
        E.append(el(f"ear_{n}", f"ear_{n}", (x0, 27.5, 17), (x1, 30.5, 19.5)))  # small and round
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        lx0, lx1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down"))
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        x = LEG_X * s
        if leg[0] == "F":
            z = FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 2.5, 5, z - 2.5), (x + 2.5, 12, z + 2.5)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2, 1.5, z - 2), (x + 2, 5.5, z + 2)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 2.75, 0, z - 2.5), (x + 2.75, 1.8, z + 4)))
        else:
            z = BACK_Z
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (x - 3.5, 6, z - 5), (x + 3.5, 17, z + 4)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2, 1.5, z - 2), (x + 2, 7, z + 2)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 2.75, 0, z - 2.5), (x + 2.75, 1.8, z + 4)))
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
    return np.array(corner, float), du, dv, int(round(abs(size @ du))), int(round(abs(size @ dv))), n


def area(e, face, p, uu, vv, h):
    """A capybara's coat: reddish brown, darker along the back, lighter underneath; the dark bald
    patch on top of the snout; dark nostrils; darker, bare feet with claws."""
    name = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, FUR)
    side = face in ("east", "west")
    t = vv / max(h, 1)
    if name in ("hips", "rump", "chest", "neck"):
        if face == "up":
            a[:] = BACK
        elif side or face in ("south", "north"):
            a = np.where(t < 0.25, BACK, np.where(t > 0.8, BELLY, FUR))
        elif face == "down":
            a[:] = BELLY
    elif name in ("skull", "muzzle"):
        if face == "up":
            a[:] = BACK
            if name == "muzzle":  # the morrillo: a dark bald patch on top of the snout
                a = np.where((np.abs(x) < 2.6) & (np.abs(z - 28.2) < 1.8), DARK, a)
        elif face == "down":
            a[:] = BELLY
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else FUR
        if face == "down":
            a[:] = BELLY
    elif name == "mouth":
        a[:] = MOUTH
    elif name.startswith("ear_"):
        a[:] = BACK
        if face == "south":
            a = np.where((uu > 0.6) & (uu < a.shape[1] - 0.6) & (vv > 0.6), EARIN, a)
    elif name.startswith(("leg_", "thigh_")):
        a = np.where(y < 4.5, SKIN, FUR) if not name.endswith("_paw") else np.full(x.shape, SKIN)
        if face == "up" and name.startswith("thigh_"):
            a[:] = BACK
    return a


LASH = (40, 28, 24)


def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    if w == 0 or h == 0:
        return None
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
    p = corner + uu[..., None] * du + vv[..., None] * dv
    light_face = face
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "jaw")):
        light_face = "north"  # seen when a pose lifts or folds it
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, n, du, dv, boxes, own=e["name"])
    col = paint_face_smooth(a, TONES, light_face, coat=COAT, y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["name"] == "skull" and face in ("east", "west"):
        (x0, y0, z0), (x1, y1, z1) = SKULL
        cu = (z1 - EYE_Z) if face == "east" else (EYE_Z - z0)
        stamp_eyes(img, [(cu, y1 - EYE_Y)], EYE_STYLE)
    if e["name"] == "muzzle" and face == "south":  # big nostrils high on the blunt snout, a split lip
        c = w // 2  # slits, not holes: big round nostrils read as a second pair of eyes
        img[1, [c - 3, c - 2, c + 1, c + 2], :3] = (72, 48, 40)
        img[3:h - 1, c, :3] = (img[3:h - 1, c, :3] * 0.75).astype(np.uint8)
        img[h - 1, c - 1:c + 1, :3] = (img[h - 1, c - 1:c + 1, :3] * 0.5 + np.array(BASE[TEETH]) * 0.5).astype(np.uint8)
    if e["name"].startswith("eyelid"):
        col_idx = 0 if e["name"].endswith("L") else w - 1
        img[:, col_idx, :3] = LASH
    if e["name"].endswith("_paw") and face == "south":  # webbed toes: 4 in front, 3 behind, dark claws
        toes = (1, 2, 3) if e["name"].startswith("leg_F") else (1, 3)
        step = w / (len(toes) + 1)
        for k in range(len(toes)):
            c = int(round(step * (k + 1))) - 1
            img[:, c, :3] = (img[:, c, :3] * 0.7).astype(np.uint8)
        img[h - 1, :, :3] = BASE[DARK]
    return img


# ------------------------------------------------------------------ animation

L = {"sit_angle": 30, "sit_hip": -80, "sit_knee": 140, "sit_lean": -4, "sit_drop": 5,
     "lie_front": 80, "lie_hip": -80, "lie_knee": 150, "lie_drop": 6, "graze_neck": 40, "graze_head": 20}


def gait(p, phases, swing_amp, lift):
    """Legs in a gait: a foot swings forward (upper angle falling) while it's lifted."""
    out = {}
    for leg, ph in zip(LEGS, phases):
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))
        out[f"leg_{leg}_upper"] = (-swing_amp * S(a), 0, 0)
        knee = lift * up
        if leg[0] == "F":
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (knee, 0, 0), (-0.6 * knee, 0, 0)
        else:
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (-knee, 0, 0), (0.7 * knee, 0, 0)
    return out


def chew(t, rate=3.0, amp=6.0):
    """Always chewing: the jaw works up and down."""
    return {"jaw": (amp * max(0.0, S(TAU * t * rate)), 0, 0)}


def pose_idle(t):
    """Chewing and looking about, then down to graze, then up again (6 s)."""
    p = t / 6.0
    br = S(TAU * p * 2)
    graze = float(np.interp(p, [0, .4, .5, .85, .95, 1], [0, 0, 1, 1, 0, 0]))
    tw = max(0.0, S(TAU * (p - 0.2) * 8)) if 0.2 < p < 0.26 else 0.0
    q = {"root": (0, 0.12 * br, 0), "spine_1": (0.6 * br, 0, 0),
         "neck": (L["graze_neck"] * graze, 0, 0),
         "head": (L["graze_head"] * graze + 2 * S(TAU * p), 10 * S(TAU * p * 2) * (1 - graze), 0),
         "ear_L": (0, 0, -18 * tw), "ear_R": (0, 0, 3 * br)}
    q.update(chew(t, 3.0 if graze > 0.5 else 2.0))
    return q


def pose_walk(t):
    p = t / 1.1
    q = gait(p, (0.0, 0.5, 0.5, 0.0), 18, 45)  # short legs: a small swing, a clear lift
    q.update({"root": (0, 0.35 * abs(S(TAU * p)), 0), "head": (2 * S(TAU * p * 2), 3 * S(TAU * p), 0)})
    q.update(chew(t, 2.0, 4.0))
    return q


def pose_run(t):
    p = t / 0.55
    q = gait(p, (0.0, 0.08, 0.5, 0.58), 30, 62)
    c = S(TAU * p)
    q.update({"root": (0, 1.2 * max(0.0, S(TAU * p + 0.6)), 0), "spine_0": (-4 * c, 0, 0), "spine_1": (6 * c, 0, 0),
              "neck": (-4 * c, 0, 0), "ear_L": (10, 0, -6), "ear_R": (10, 0, 6)})
    return q


def pose_sit(t, P=L):
    """Sitting on its haunches, front legs straight, chewing."""
    p = t / 4.0
    br = S(TAU * p * 2)
    a = P["sit_angle"]
    q = {"root": (0, -P["sit_drop"] + 0.1 * br, 0), "spine_0": (-a, 0, 0), "spine_1": (0.6 * br, 0, 0),
         "neck": (a * 0.5, 0, 0), "head": (a * 0.5, 6 * S(TAU * p), 0)}
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (a + P["sit_hip"], 0, 0)
        q[f"leg_{leg}_lower"] = (P["sit_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["sit_hip"] - P["sit_knee"], 0, 0)
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (a + P["sit_lean"], 0, 0)
        q[f"leg_{leg}_lower"] = (0, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["sit_lean"], 0, 0)
    q.update(chew(t, 2.0))
    return q


def pose_lie(t, P=L):
    """Flopped on its belly, legs tucked, the head held up (as capybaras rest), chewing slowly."""
    p = t / 5.0
    br = S(TAU * p * 2)
    q = {"root": (0, -P["lie_drop"] + 0.12 * br, 0), "spine_1": (0.6 * br, 0, 0), "head": (0, 5 * S(TAU * p), 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (P["lie_front"], 0, 0)
        q[f"leg_{leg}_lower"] = (-70, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_front"] + 70, 0, 0)
    for leg, s in (("BL", 1), ("BR", -1)):
        q[f"leg_{leg}_upper"] = (P["lie_hip"], 0, s * 6)
        q[f"leg_{leg}_lower"] = (P["lie_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_hip"] - P["lie_knee"], 0, 0)
    q.update(chew(t, 1.5, 4.0))
    return q


BLINKS = {"idle": [1.2, 4.5], "sit": [1.6], "lie": [1.0, 2.6, 4.2]}
BLINK = 0.1
ANIMS = {"idle": (pose_idle, 6.0), "walk": (pose_walk, 1.1), "run": (pose_run, 0.55), "sit": (pose_sit, 4.0),
         "lie": (pose_lie, 5.0)}


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}  # side eyes: the lids swing down about z


def frame_times(name, dur):
    n = {"idle": 72, "sit": 48, "lie": 60, "walk": 24, "run": 20}.get(name, 16)  # chewing and short steps need fine keys
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
          "back legs": lambda n: n.startswith("thigh_") or n.startswith("leg_B") and not n.endswith("_paw"),
          "front legs": lambda n: n.startswith("leg_F") and not n.endswith("_paw"),
          "rump": lambda n: n in ("hips", "rump"), "chest": lambda n: n == "chest",
          "muzzle": lambda n: n in ("muzzle", "jaw")}


def fit_poses(spec):
    """Sit and lie on the ground (paws, rump, belly), and grazing with the mouth at the grass."""
    def low(fn, P):
        g = lowest(spec, fn(0.0, P), GROUPS)
        g.pop("muzzle", None)
        return g

    def sit_cost(P):
        g = low(pose_sit, P)
        f = min(g.values())
        r = {k: v - f for k, v in g.items()}
        return 3 * r["front paws"] ** 2 + r["back paws"] ** 2 + r["rump"] ** 2 + 0.5 * r["back legs"] ** 2
    grid = {"sit_angle": list(range(10, 46, 5)), "sit_hip": list(range(-100, -59, 5)), "sit_knee": list(range(110, 166, 5)),
            "sit_lean": [-10, -4, 2]}
    best = fit(grid, lambda P: sit_cost(dict(L, **P, sit_drop=0)))[1]
    L.update(best, sit_drop=round(float(min(low(pose_sit, dict(L, **best, sit_drop=0)).values())), 2))

    def lie_cost(P):
        g = low(pose_lie, P)
        f = min(g.values())
        r = {k: v - f for k, v in g.items()}
        return r["chest"] ** 2 + r["rump"] ** 2 + 0.5 * r["front paws"] ** 2 + 0.5 * r["back paws"] ** 2
    grid = {"lie_front": list(range(60, 111, 5)), "lie_hip": list(range(-104, -63, 8)), "lie_knee": list(range(116, 166, 8))}
    best = fit(grid, lambda P: lie_cost(dict(L, **P, lie_drop=0)))[1]
    L.update(best, lie_drop=round(float(min(low(pose_lie, dict(L, **best, lie_drop=0)).values())), 2))

    # Grazing: neck and head angles that bring the mouth down to the grass (about 0.5 up).
    def graze_cost(P):
        L2 = dict(L, **P)
        g = lowest(spec, {"neck": (P["graze_neck"], 0, 0), "head": (P["graze_head"], 0, 0)}, GROUPS)
        return (g["muzzle"] - 0.5) ** 2 + 0.002 * (P["graze_neck"] ** 2 + P["graze_head"] ** 2)
    best = fit({"graze_neck": list(range(20, 71, 5)), "graze_head": list(range(0, 61, 5))}, graze_cost)[1]
    L.update(best)


# ------------------------------------------------------------------ build


def build():
    B = bones()
    E = elements()
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"]]
    tiles = []
    for i, e in enumerate(E):
        for f in ([e["quad"]] if e["quad"] else list(FACES)):
            img = paint_face(e, f, boxes)
            if img is not None:
                tiles.append(((i, f), img))
    sheet, uv = pack(tiles, 512)
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "capybara", "texture": [sheet.shape[1], sheet.shape[0]],
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
