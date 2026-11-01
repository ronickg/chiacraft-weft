#!/usr/bin/env python3
"""Builds the chihuahua's model, animations and coats: a long-haired dog of boxes for its body
and flat fur panels (fringes, cut out where the texture is clear) for its ears, ruff, cheeks,
skirt, leg feathering and plume tail.

Writes, under `assets/`:
  models/chihuahua.geo.json, models/chihuahua_pup.geo.json       (Bedrock geometry, Blockbench opens it)
  animations/chihuahua[_pup][_sit|_lie].animation.json           (`idle` holds the pose; `walk`)
  textures/entity/chihuahua/coat_<i>.png                         (8 colors x 6 patterns x 2 layouts)

Deterministic: run it again after changing it and commit what it writes.

    python3 mods/chihuahua/tools/chihuahua.py

Coordinates: model pixels before scaling (a grown dog is drawn at 0.75, a puppy at 0.375 with a
bigger head), y up from the feet, the dog looks along -Z. Rotations here are the engine's
(radians, positive x lifts the front, positive z leans the top to -x); the files get Blockbench's
degrees (x and z negated).
"""

import json
import math
import os
import random
import struct
import zlib

ROOT = os.path.join(os.path.dirname(__file__), "..", "assets")
TEXELS = 3  # texels per model pixel
SHEET_W = 128
ADULT, PUP, PUP_HEAD = 0.75, 0.375, 1.3

# ------------------------------------------------------------------ the model


class Box:
    """A box (or a flat panel when one size is 0) of a bone. `fringe`: for panels, the part of
    the panel the box underneath covers (the core, as (u0, v0, u1, v1) in the panel's own two
    axes) and how far fur may stick out of it."""

    def __init__(self, origin, size, kind, fringe=None):
        self.origin, self.size, self.kind, self.fringe = origin, size, kind, fringe


class Bone:
    def __init__(self, name, pivot, parent=None, rot=(0.0, 0.0, 0.0)):
        self.name, self.pivot, self.parent, self.rot = name, pivot, parent, rot
        self.boxes = []

    def box(self, origin, size, kind, fringe=None):
        self.boxes.append(Box(origin, size, kind, fringe))
        return self


def build():
    bones = []

    def bone(*a, **k):
        b = Bone(*a, **k)
        bones.append(b)
        return b

    body = bone("body", (0, 4.5, 0))
    body.box((-2, 3, -2.6), (4, 3, 5.2), "torso")
    body.box((-1.9, 2.7, -3.4), (3.8, 3.5, 1.0), "ruff")
    # Fur: a mane around the neck, the chest's, the belly's skirt on both sides.
    body.box((-3.7, 1.4, -2.7), (7.4, 6.4, 0), "mane", ((-2.0, 3.0, 2.0, 6.0), 1.6))
    body.box((-3.5, 1.1, -3.45), (7.0, 5.1, 0), "ruff_fringe", ((-1.9, 2.7, 1.9, 6.2), 1.6))
    for x in (-2.05, 2.05):
        body.box((x, 1.2, -2.4), (0, 2.4, 4.8), "skirt", ((3.0, -2.4, 3.6, 2.4), 1.8))

    head = bone("head", (0, 6.2, -2.8), "body")
    head.box((-2.75, 5.4, -7.1), (5.5, 4.4, 4.3), "skull")
    head.box((-2.2, 9.8, -6.6), (4.4, 0.5, 3.2), "dome")
    head.box((-0.95, 5.4, -8.2), (1.9, 1.6, 1.1), "snout")
    for x in (-2.05, 0.65):
        head.box((x, 7.0, -7.25), (1.4, 1.4, 0.15), "eye")
    # The fluff around the face (seen from the front) and on the cheeks (from the side).
    head.box((-5.2, 3.0, -5.0), (10.4, 7.2, 0), "face_mane", ((-2.75, 5.4, 2.75, 9.8), 2.4))
    for x in (-2.8, 2.8):
        head.box((x, 4.5, -6.9), (0, 4.5, 3.9), "cheek", ((5.4, -6.9, 9.0, -3.0), 0.9))

    for side, x in (("ear_r", -1.6), ("ear_l", 1.6)):
        lean = 0.42 if x < 0 else -0.42
        ear = bone(side, (x, 9.6, -4.9), "head", rot=(0.15, 0.0, lean))
        ear.box((x - 1.3, 9.6, -5.2), (2.6, 3.8, 0.6), "ear")
        ear.box((x - 2.6, 9.3, -4.9), (5.2, 5.6, 0), "ear_fringe", ((x - 1.3, 9.6, x + 1.3, 13.4), 1.3))

    tail = bone("tail", (0, 5.4, 2.4), "body", rot=(-1.35, 0.0, 0.0))
    tail.box((-0.5, 4.9, 2.4), (1, 1, 2.8), "tail")
    tail.box((0, 2.5, 2.4), (0, 5.8, 3.4), "tail_fringe", ((4.9, 2.4, 5.9, 5.2), 2.4))
    tail.box((-2.9, 5.4, 2.6), (5.8, 0, 3.2), "tail_fringe", ((-0.5, 2.6, 0.5, 5.2), 2.4))
    tip = bone("tail_tip", (0, 5.4, 5.2), "tail", rot=(-1.2, 0.0, 0.0))
    tip.box((-0.45, 4.95, 5.2), (0.9, 0.9, 2.4), "tail_tip")
    tip.box((0, 2.15, 4.9), (0, 6.5, 4.7), "tip_fringe", ((4.95, 4.9, 5.85, 7.6), 2.8))
    tip.box((-3.25, 5.4, 4.9), (6.5, 0, 4.7), "tip_fringe", ((-0.45, 4.9, 0.45, 7.6), 2.8))

    # leg0..3: back right, back left, front right, front left (the engine swings them so).
    for name, x, z, back in (("leg0", -1.35, 1.8, True), ("leg1", 1.35, 1.8, True), ("leg2", 1.35, -1.8, False), ("leg3", -1.35, -1.8, False)):
        leg = bone(name, (x, 3, z), "body")
        leg.box((x - 0.6, 0.4, z - 0.6), (1.2, 2.7, 1.2), "leg")
        leg.box((x - 0.68, 0, z - 0.78), (1.36, 0.5, 1.45), "paw")
        top, reach = (3.4, 1.3) if back else (2.9, 0.8)
        leg.box((x, 0.6, z - 0.6), (0, top - 0.6, 1.2 + reach), "feather", ((0.6, z - 0.6, top, z + 0.6), reach))
    return bones


# ------------------------------------------------------------------ texture layout

# Faces in the engine's order (-X +X -Y +Y -Z +Z) and their names in the file.
FACE_NAMES = ["east", "west", "down", "up", "north", "south"]
# A unit cube's face corners: bottom-left, bottom-right, top-right, top-left in texture space.
CORNERS = [
    [(0, 0, 0), (0, 0, 1), (0, 1, 1), (0, 1, 0)],
    [(1, 0, 1), (1, 0, 0), (1, 1, 0), (1, 1, 1)],
    [(0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)],
    [(0, 1, 1), (1, 1, 1), (1, 1, 0), (0, 1, 0)],
    [(1, 0, 0), (0, 0, 0), (0, 1, 0), (1, 1, 0)],
    [(0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)],
]
NORMALS = [(-1, 0, 0), (1, 0, 0), (0, -1, 0), (0, 1, 0), (0, 0, -1), (0, 0, 1)]


def face_extent(size, f):
    """The face's width and height in model pixels."""
    w, h, d = size
    return [(d, h), (d, h), (w, d), (w, d), (w, h), (w, h)][f]


def layout(bones):
    """Every visible face gets its own texel rectangle, shelf packed. Returns {(bone, box, face): rect}."""
    faces = []
    for bi, b in enumerate(bones):
        for ci, c in enumerate(b.boxes):
            for f in range(6):
                fw, fh = face_extent(c.size, f)
                if fw <= 0 or fh <= 0:
                    continue
                faces.append(((bi, ci, f), max(1, round(fw * TEXELS)), max(1, round(fh * TEXELS))))
    faces.sort(key=lambda t: (-t[2], -t[1]))
    rects, x, y, row = {}, 0, 0, 0
    for key, w, h in faces:
        if x + w > SHEET_W:
            x, y, row = 0, y + row + 1, 0
        rects[key] = (x, y, w, h)
        x += w + 1
        row = max(row, h)
    height = 1 << math.ceil(math.log2(y + row + 1))
    return rects, height


def texel_point(c, f, s, t):
    """The point (model pixels, as built) a face's texture coordinate (s, t; t down) lands on."""
    c0, c1, c2, c3 = CORNERS[f]
    p = [c3[i] + s * (c2[i] - c3[i]) + t * (c0[i] - c3[i]) for i in range(3)]
    return [c.origin[i] + p[i] * c.size[i] for i in range(3)]


# ------------------------------------------------------------------ coats

COLORS = [
    ("fawn", (214, 160, 102)),
    ("cream", (236, 214, 172)),
    ("chocolate", (104, 58, 34)),
    ("black", (42, 34, 32)),
    ("blue", (100, 108, 120)),
    ("red", (178, 98, 50)),
    ("white", (240, 234, 224)),
    ("golden", (206, 148, 66)),
]
PATTERNS = ["solid", "marked", "piebald", "tricolor", "sable", "patched"]
WHITE = (246, 241, 232)
TAN = (198, 134, 76)
PINK = (232, 164, 158)
NOSE = (86, 50, 46)
EYE = {"D": (40, 26, 20), "K": (14, 10, 8), "I": (112, 66, 32), "H": (252, 250, 244)}
EYE_ART = ["DDDD", "DHKI", "DKKI", "DIID"]


def h(*k):
    """A stable hash of numbers to 0..1."""
    x = 2166136261
    for v in k:
        x = ((x ^ (int(v) & 0xFFFFFFFF)) * 16777619) & 0xFFFFFFFF
    x ^= x >> 13
    x = (x * 0x5BD1E995) & 0xFFFFFFFF
    x ^= x >> 15
    return x / 0xFFFFFFFF


def mix(a, b, t):
    return tuple(a[i] + (b[i] - a[i]) * t for i in range(3))


def scale(c, f):
    return tuple(c[i] * f for i in range(3))


class Coat:
    def __init__(self, ci, pattern, marks):
        self.name, base = COLORS[ci]
        self.pattern, self.marks = pattern, marks
        if pattern == "tricolor" and sum(base) > 300:
            base = (42, 34, 32)
        self.base = base
        self.dark = scale(base, 0.7)
        rng = random.Random(f"{self.name}-{pattern}-{marks}")
        # Patches for "patched" (centers and radii in model pixels), the blaze's width.
        self.patches = [((rng.uniform(-2, 2), rng.uniform(3.5, 6.5), rng.uniform(-3, 3)), rng.uniform(1.4, 2.4)) for _ in range(2 + marks)]
        self.blaze = 0.55 if marks == 0 else 0.9
        self.eye_patch = rng.choice([-1, 1])

    def color(self, kind, p, n):
        """The coat's color at a point of a part (as built), with the face's normal."""
        x, y, z = p
        base, pat = self.base, self.pattern
        chest = kind in ("ruff", "ruff_fringe") or (kind == "mane" and y < 5.2) or (kind == "face_mane" and y < 6.0)
        belly = (kind == "torso" and n[1] < 0) or (kind == "skirt" and y < 3.0)
        paw = kind == "paw" or (kind in ("leg", "feather") and y < 1.0)
        lower_leg = kind in ("leg", "paw", "feather") and y < 2.4
        muzzle = kind == "snout" or (kind == "skull" and n[2] < 0 and y < 6.6)
        blaze = kind in ("skull", "dome") and abs(x) < self.blaze and (n[2] < 0 or n[1] > 0)
        chin = kind == "skull" and n[1] < 0
        tail_under = kind in ("tail_fringe", "tip_fringe") and y < 5.4
        tip = kind in ("tail_tip", "tip_fringe")
        white = False
        tan = False
        c = base
        if pat == "marked":
            white = chest or belly or paw or chin or tip
        elif pat == "piebald":
            white = chest or belly or lower_leg or muzzle or blaze or chin or tail_under or (tip and self.marks == 1) or (kind in ("cheek", "face_mane") and y < 6.4) or (kind == "feather")
        elif pat == "tricolor":
            white = chest and y < 5.6
            brow = kind == "skull" and n[2] < 0 and 8.4 < y < 9.0 and 0.7 < abs(x) < 1.8
            tan = (lower_leg and not paw) or muzzle or brow or (kind in ("cheek", "face_mane") and y < 6.4) or tail_under or (belly and not chest) or kind == "feather"
            if paw:
                white = True
        elif pat == "sable":
            up = n[1] > 0 or kind in ("ear", "ear_fringe") or (kind in ("torso", "mane") and y > 5.2)
            if up:
                c = self.dark
            if belly or chest:
                c = mix(base, WHITE, 0.5)
        elif pat == "patched":
            white = True
            if kind in ("ear", "ear_fringe"):
                white = False
            elif kind in ("skull", "dome", "cheek", "face_mane") and x * self.eye_patch > self.blaze:
                white = False
            elif kind in ("torso", "skirt", "tail", "tail_fringe", "mane"):
                for (cx, cy, cz), r in self.patches:
                    if (x - cx) ** 2 + (y - cy) ** 2 + (z - cz) ** 2 < r * r:
                        white = False
        if white:
            c = WHITE
        elif tan:
            c = TAN
        return c


def fur(c, kind, p, n, u, v, t, out):
    """Fur shading: strands (columns of slightly different brightness running along the hair),
    darker toward the bottom of a side, lighter tips on fringes, a little noise."""
    strand = (h(u, 7) + h(u + 1, 7)) * 0.07 - 0.07
    grain = h(u, v, 3) * 0.08 - 0.04
    f = 1.0 + strand + grain
    if n[1] == 0 and out is None:
        f *= 1.04 - 0.1 * t
    if out is not None:
        f *= 0.96 + 0.14 * out
    if kind in ("torso", "skull", "dome") and n[1] > 0:
        f *= 1.05
    return tuple(max(0, min(255, round(x * f))) for x in c)


def plane_axes(size):
    """A panel's two axes (indices into xyz) in the order its core rectangle is given."""
    if size[0] == 0:
        return 1, 2
    if size[1] == 0:
        return 0, 2
    return 0, 1


def tuft(k, period, seed):
    """Pointed locks along an edge: the length (0..1) of the lock at texel `k`."""
    best = 0.0
    c0 = math.floor(k / period)
    for cc in (c0 - 1, c0, c0 + 1):
        center = (cc + 0.2 + 0.6 * h(cc, seed)) * period
        length = 0.5 + 0.5 * h(cc, seed + 1)
        d = abs(k + 0.5 - center) / (period * 0.75)
        if d < 1:
            best = max(best, length * (1 - d) ** 0.9)
    return best


def fringe_alpha(c, p, u, v):
    """Whether a panel's texel is fur: inside its core always, outside within a lock (locks of
    two sizes overlap, so the edge is ragged but full). Returns (opaque, how far out 0..1)."""
    (a0, b0, a1, b1), reach = c.fringe
    i, j = plane_axes(c.size)
    a, b = p[i], p[j]
    da = max(a0 - a, 0, a - a1)
    db = max(b0 - b, 0, b - b1)
    out = max(da, db)
    if out <= 0:
        return True, 0.0
    along = (b if da > db else a) * TEXELS
    seed = int(abs(c.origin[0] * 7 + c.origin[1] * 13 + c.origin[2] * 17) * 10) % 997 + (7 if da > db else 3)
    length = reach * max(tuft(along, 3.6, seed), 0.75 * tuft(along + 1.3, 2.3, seed + 10), 0.3)
    return out < length, min(1.0, out / reach)


def paint(bones, rects, height, coat):
    img = [[(0, 0, 0, 0)] * SHEET_W for _ in range(height)]
    for (bi, ci, f), (rx, ry, rw, rh) in rects.items():
        b = bones[bi]
        c = b.boxes[ci]
        n = NORMALS[f]
        for ty in range(rh):
            for tx in range(rw):
                s, t = (tx + 0.5) / rw, (ty + 0.5) / rh
                p = texel_point(c, f, s, t)
                out = None
                if c.fringe is not None:
                    opaque, out = fringe_alpha(c, p, tx, ty)
                    if not opaque:
                        continue
                col = coat.color(c.kind, p, n)
                col = fur(col, c.kind, p, n, rx + tx, ry + ty, t, out)
                if c.kind == "eye":
                    col = EYE["D"]
                    if f == 4:
                        col = EYE[EYE_ART[min(3, int(t * 4))][min(3, int(s * 4))]]
                elif c.kind == "snout" and f == 4:
                    x, y = p[0], p[1]
                    if y > 6.3 and abs(x) < 0.65:
                        col = NOSE if not (y > 6.7 and x < -0.1) else (130, 84, 80)
                    elif y < 5.95 and abs(x) < 0.15:
                        col = scale(col, 0.6)
                elif c.kind == "ear" and f == 4:
                    x, y = p[0], p[1]
                    cx = b.pivot[0]
                    if 9.8 < y < 13.2 and abs(x - cx) < 1.05 * (1 - (y - 9.8) / 4.3):
                        col = mix(PINK, (250, 236, 226), 0.25 if h(rx + tx, ry + ty) > 0.7 else 0.0)
                img[ry + ty][rx + tx] = tuple(round(v) for v in col) + (255,)
    return img


# ------------------------------------------------------------------ files


def deg(r):
    """Engine rotation (radians) to the file's degrees."""
    return [round(-math.degrees(r[0]), 3), round(math.degrees(r[1]), 3), round(-math.degrees(r[2]), 3)]


def r3(v):
    return [round(x, 4) for x in v]


def geo(bones, rects, height, s, head_scale, ident):
    def subtree(b, root):
        while b is not None:
            if b.name == root:
                return True
            b = next((x for x in bones if x.name == b.parent), None)
        return False

    hp = next(b for b in bones if b.name == "head").pivot

    def at(b, v):
        if head_scale != 1.0 and subtree(b, "head"):
            v = [hp[i] + (v[i] - hp[i]) * head_scale for i in range(3)]
        return [x * s for x in v]

    out = []
    for bi, b in enumerate(bones):
        k = head_scale if (head_scale != 1.0 and subtree(b, "head")) else 1.0
        cubes = []
        for ci, c in enumerate(b.boxes):
            uv = {}
            for f in range(6):
                rect = rects.get((bi, ci, f))
                name = FACE_NAMES[f]
                if rect is None:
                    uv[name] = {"uv": [0, 0], "uv_size": [0, 0]}
                    continue
                x, y, w, hh = rect
                # The engine reads "down" flipped in v, as Blockbench writes it.
                uv[name] = {"uv": [x, y + hh], "uv_size": [w, -hh]} if name == "down" else {"uv": [x, y], "uv_size": [w, hh]}
            cubes.append({"origin": r3(at(b, c.origin)), "size": r3([v * s * k for v in c.size]), "uv": uv})
        bone = {"name": b.name, "pivot": r3(at(b, b.pivot)), "cubes": cubes}
        if b.parent:
            bone["parent"] = b.parent
        if any(b.rot):
            bone["rotation"] = deg(b.rot)
        out.append(bone)
    return {
        "format_version": "1.12.0",
        "minecraft:geometry": [{"description": {"identifier": ident, "texture_width": SHEET_W, "texture_height": height}, "bones": out}],
    }


def keyframes(times_values, smooth=True):
    frames = {}
    for t, v in times_values:
        frames[f"{t:g}"] = {"post": v, "lerp_mode": "catmullrom"} if smooth else v
    return frames


def wave(length, period, amp, axis, phase=0.0, steps_per_period=4):
    """A sine as smooth keyframes over `length` seconds (whole periods)."""
    n = round(length / period) * steps_per_period
    out = []
    for i in range(n + 1):
        t = length * i / n
        a = amp * math.sin(2 * math.pi * (t / period) + phase)
        v = [0.0, 0.0, 0.0]
        v[axis] = round(a, 3)
        out.append((round(t, 4), v))
    return out


def rot_file(v):
    """A rotation keyframe from engine radians."""
    return deg(v)


def animations(pose, s):
    """`idle` (holding the pose, with a little life) and `walk` for one pose; `s` scales moves."""
    name = f"animation.chihuahua.{pose}"
    anims = {}
    bones = {}

    def rot(bone, frames):
        bones.setdefault(bone, {})["rotation"] = keyframes([(t, rot_file(v)) for t, v in frames])

    def const_rot(bone, v):
        bones.setdefault(bone, {})["rotation"] = rot_file(v)

    def const_pos(bone, v):
        bones.setdefault(bone, {})["position"] = r3([x * s for x in v])

    if pose == "stand":
        length = 4.0
        rot("tail", wave(length, 0.5, 0.35, 1))
        rot("ear_r", [(0, [0, 0, 0]), (2.6, [0, 0, 0]), (2.7, [0, 0, 0.25]), (2.85, [0, 0, 0]), (4, [0, 0, 0])])
        rot("ear_l", [(0, [0, 0, 0]), (1.1, [0, 0, 0]), (1.2, [0, 0, -0.25]), (1.35, [0, 0, 0]), (4, [0, 0, 0])])
        bones.setdefault("body", {})["scale"] = keyframes([(t, [1.0, round(1.0 + 0.025 * math.sin(math.pi * t), 4), 1.0]) for t in [0, 0.5, 1, 1.5, 2, 2.5, 3, 3.5, 4]])
    elif pose == "sit":
        length = 4.0
        const_pos("body", [0, -1.2, 0])
        bones["body"]["rotation"] = rot_file([0.55, 0, 0])
        for leg in ("leg0", "leg1"):
            const_rot(leg, [0.95, 0, 0])
        for leg in ("leg2", "leg3"):
            const_rot(leg, [-0.55, 0, 0])
        rot("head", [(0, [-0.45, 0, 0]), (1.5, [-0.45, 0, 0]), (1.8, [-0.45, 0, 0.18]), (2.8, [-0.45, 0, 0.18]), (3.1, [-0.45, 0, 0]), (4, [-0.45, 0, 0])])
        rot("tail", [(t, [1.05, v[1], 0]) for t, v in wave(length, 1.0, 0.3, 1)])
    else:  # lie
        length = 6.0
        const_pos("body", [0, -2.4, 0])
        for leg in ("leg0", "leg1", "leg2", "leg3"):
            const_rot(leg, [1.45, 0, 0])
        const_pos("head", [0, -0.4, 0])
        const_rot("head", [-0.2, 0, 0])
        rot("tail", [(t, [1.25, 0.9 + v[1], 0]) for t, v in wave(length, 3.0, 0.12, 1)])
        rot("ear_r", [(0, [0, 0, 0]), (4.0, [0, 0, 0]), (4.1, [0, 0, 0.3]), (4.3, [0, 0, 0]), (6, [0, 0, 0])])
        bones["body"]["scale"] = keyframes([(t, [1.0, round(1.0 + 0.03 * math.sin(2 * math.pi * t / 3.0), 4), 1.0]) for t in [0, 0.75, 1.5, 2.25, 3, 3.75, 4.5, 5.25, 6]])
    anims[f"{name}.idle"] = {"loop": True, "animation_length": length, "bones": bones}

    # Walking: diagonal pairs (back right with front left), a bob and a wagging tail; one stride
    # per 1.4 (the engine runs walk animations by distance).
    stride = 1.4
    walk = {}
    for leg, phase in (("leg0", 0.0), ("leg3", 0.0), ("leg1", math.pi), ("leg2", math.pi)):
        walk[leg] = {"rotation": keyframes([(t, rot_file(v)) for t, v in wave(stride, stride, 0.6, 0, phase)])}
    walk["body"] = {"position": keyframes([(t, r3([0, abs(v[0]) * s, 0])) for t, v in wave(stride, stride, 0.35, 0)])}
    walk["tail"] = {"rotation": keyframes([(t, rot_file(v)) for t, v in wave(stride, stride / 2, 0.3, 1)])}
    walk["head"] = {"rotation": keyframes([(t, rot_file(v)) for t, v in wave(stride, stride / 2, 0.06, 0)])}
    anims[f"{name}.walk"] = {"loop": True, "animation_length": stride, "bones": walk}
    return {"format_version": "1.8.0", "animations": anims}


def png(path, img):
    w, hh = len(img[0]), len(img)
    raw = b"".join(b"\x00" + bytes(c for px in row for c in px) for row in img)

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    data = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, hh, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    with open(path, "wb") as f:
        f.write(data)


def write_json(path, value):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        json.dump(value, f, indent=1)
        f.write("\n")


def main():
    bones = build()
    rects, height = layout(bones)
    write_json(os.path.join(ROOT, "models", "chihuahua.geo.json"), geo(bones, rects, height, ADULT, 1.0, "geometry.chihuahua"))
    write_json(os.path.join(ROOT, "models", "chihuahua_pup.geo.json"), geo(bones, rects, height, PUP, PUP_HEAD, "geometry.chihuahua_pup"))
    for pose in ("stand", "sit", "lie"):
        suffix = "" if pose == "stand" else f"_{pose}"
        write_json(os.path.join(ROOT, "animations", f"chihuahua{suffix}.animation.json"), animations(pose, ADULT))
        write_json(os.path.join(ROOT, "animations", f"chihuahua_pup{suffix}.animation.json"), animations(pose, PUP))
    out = os.path.join(ROOT, "textures", "entity", "chihuahua")
    os.makedirs(out, exist_ok=True)
    i = 0
    for ci in range(len(COLORS)):
        for pattern in PATTERNS:
            for marks in range(2):
                png(os.path.join(out, f"coat_{i}.png"), paint(bones, rects, height, Coat(ci, pattern, marks)))
                i += 1
    print(f"model, 6 animation files and {i} coats ({SHEET_W}x{height}) in {os.path.normpath(ROOT)}")


if __name__ == "__main__":
    main()
