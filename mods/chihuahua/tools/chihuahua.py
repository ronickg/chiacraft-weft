#!/usr/bin/env python3
"""Builds the chihuahua's model, animations and coats the way Hytale and Vintage Story make
creatures: a few dozen chunky boxes (tufts of fur are small boxes in tilted bones, fluffy parts
end in a zig-zag fringe cut out of the texture) and textures painted like illustrations: soft
light over each part as if it were round, shadow where parts meet, shadows tinted cool, fur
painted as layered locks, eyes painted on.

Writes, under `assets/`:
  models/chihuahua.geo.json, models/chihuahua_pup.geo.json       (Bedrock geometry, Blockbench opens it)
  animations/chihuahua[_pup][_sit|_lie].animation.json           (`idle` holds the pose; `walk`)
  textures/entity/chihuahua/coat_<i>.png                         (8 colors x 6 patterns x 2 layouts)

Deterministic: run it again after changing it and commit what it writes. Needs numpy.

    python3 mods/chihuahua/tools/chihuahua.py

Coordinates: design units (a grown dog is drawn at 0.75 model pixels each, a puppy at 0.375
with a bigger head), y up from the feet, the dog looks along -Z. Rotations here are the engine's
(radians, positive x lifts the front, positive z leans the top to -x); the files get Blockbench's
degrees (x and z negated).
"""

import json
import math
import os
import random
import struct
import zlib

import numpy as np

ROOT = os.path.join(os.path.dirname(__file__), "..", "assets")
T = 4  # texels per design unit (about 85 a block on a grown dog; Hytale's creatures have 64)
SHEET = 256
ADULT, PUP, PUP_HEAD = 0.75, 0.375, 1.3
FLUFFY = {"ruff", "skirt", "plume", "feather", "hip"}

# ------------------------------------------------------------------ the model


class Box:
    """A box; `fringe`: the side (a face index) whose edge is cut into zig-zag locks."""

    def __init__(self, origin, size, kind, fringe):
        self.origin, self.size, self.kind, self.fringe = np.array(origin, float), np.array(size, float), kind, fringe


class Bone:
    def __init__(self, name, pivot, parent=None, rot=(0.0, 0.0, 0.0)):
        self.name, self.pivot, self.parent, self.rot = name, np.array(pivot, float), parent, rot
        self.boxes = []

    def box(self, origin, size, kind, fringe=None):
        self.boxes.append(Box(origin, size, kind, fringe if fringe is not None else (2 if kind in FLUFFY else None)))
        return self


# The dog's proportions (design units, angles in radians). `shape.json` next to this script
# overrides any of them; the shape editor page writes such a file.
SHAPE = {
    "head_w": 5.6, "head_h": 4.4, "head_d": 4.4, "head_y": 0.0, "head_fwd": 0.0,
    "muzzle_len": 1.4, "muzzle_w": 2.7, "muzzle_h": 1.8,
    "jowl_w": 6.3, "jowl_h": 2.0,
    "eye_size": 1.0, "eye_gap": 1.35, "eye_y": 2.55,
    "ear_h": 4.6, "ear_w": 1.5, "ear_gap": 1.9, "ear_lean": 0.5, "ear_turn": 0.45, "ear_hair": 0.95,
    "body_len": 4.6, "body_w": 4.0, "body_h": 3.2, "chest": 1.0, "fluff": 1.0,
    "leg_len": 1.8, "leg_w": 1.2, "leg_gap": 1.25, "paw_w": 1.7, "paw_h": 0.75,
    "tail_size": 1.0, "tail_curl": 1.0,
}

# Where things ended up, for the painting (filled in by build).
G = {}


def build(P=SHAPE):
    bones = []

    def bone(*a, **k):
        b = Bone(*a, **k)
        bones.append(b)
        return b

    bw, bh, bl, c = P["body_w"], P["body_h"], P["body_len"], P["chest"]
    BY = 0.6 + P["leg_len"]  # the body's underside
    z0 = -2.0  # the body's front
    body = bone("body", (0, BY + 1.5, 0))
    body.box((-bw / 2, BY, z0), (bw, bh, bl), "torso")
    body.box((-(bw / 2 + 0.2), BY + bh - 1.0, z0 + 0.2), (bw + 0.4, 1.3, bl - 0.6), "torso")  # fur along the back
    body.box((-(bw / 2 + 0.2), BY - 0.3, z0 + bl - 0.7), (bw + 0.4, bh - 0.2, 1.4), "hip")
    # The chest: a big fluffy bib, a tuft under it, a smaller plate stepping out in front (rounder).
    rw = bw + 0.8 * c
    body.box((-rw / 2, BY, z0 - 1.9 * c), (rw, 3.9, 2.1 * c), "ruff")
    body.box((-1.5 * c, BY - 0.9 * c, z0 - 1.5 * c), (3.0 * c, 1.2 * c, 1.4 * c), "ruff")
    body.box((-(rw / 2 - 0.5), BY + 0.4, z0 - 1.9 * c - 0.4), (rw - 1.0, 3.1, 0.4), "ruff")
    body.box((-(bw / 2 + 0.3), BY + 2.1, z0 - 1.0), (bw + 0.6, 1.9, 2.2), "mane")
    body.box((-(bw / 2 + 0.1), BY - 0.7, z0 + 0.2), (bw + 0.2, 0.8, bl - 0.8), "skirt")

    hw, hh, hd = P["head_w"], P["head_h"], P["head_d"]
    HB = BY + 3.3 + P["head_y"]  # the skull's underside
    SF = -7.0 + P["head_fwd"]  # its front
    ml, mw, mh = P["muzzle_len"], P["muzzle_w"], P["muzzle_h"]
    head = bone("head", (0, HB + 0.4, SF + hd - 0.6), "body")
    head.box((-hw / 2, HB, SF), (hw, hh, hd), "skull")
    head.box((-(hw / 2 - 0.5), HB + hh, SF + 0.5), (hw - 1.0, 0.5, hd - 1.0), "skull")
    head.box((-(hw / 2 - 0.35), HB + 0.4, SF - 0.35), (hw - 0.7, hh - 0.7, 0.35), "skull")  # the face, stepping out
    head.box((-mw / 2, HB - 0.15, SF - ml), (mw, mh, ml), "muzzle")
    head.box((-(mw / 2 - 0.35), HB - 0.45, SF - ml + 0.4), (mw - 0.7, 0.4, max(ml - 0.4, 0.3)), "muzzle")
    nose_y = HB - 0.15 + mh - 0.55
    head.box((-0.65, nose_y, SF - ml - 0.35), (1.3, 0.85, 0.4), "nose")
    # The lower face: a wide, light block under the eyes, sticking out past the head (as
    # Hytale's dogs have it), the muzzle in front of it.
    head.box((-P["jowl_w"] / 2, HB - 0.3, SF - 0.25), (P["jowl_w"], P["jowl_h"], hd - 0.8), "cheek")

    eh, ew = P["ear_h"], P["ear_w"]
    for name, side in (("ear_r", -1), ("ear_l", 1)):
        # Leaned out and turned so the inside looks forward and a little out.
        x0 = side * P["ear_gap"]
        pivot = (x0, HB + hh - 0.2, SF + 2.4)
        EB = pivot[1] - 0.2
        ear = bone(name, pivot, "head", rot=(0.0, -side * P["ear_turn"], -side * P["ear_lean"]))
        # Steps wide enough for the long hair along the edges; the texture cuts them to a
        # triangle with a ragged fringe.
        y = EB - 0.3
        for k, frac in enumerate((0.37, 0.3, 0.24, 0.24)):
            h = (eh + 1.0) * frac
            half = ew * max(0.0, 1 - (y - EB) / eh) + ear_reach(P, EB, y) + 0.15
            ear.box((x0 - half, y, pivot[2] - 0.3), (2 * half, h, 0.5), "ear")
            y += h
        ear.box((x0 - 0.9, EB + 0.1, pivot[2] - 0.4), (1.8, 0.9, 0.2), "ear_fluff")

    # The tail curls up over the back: a short base, a big plume, its tip.
    ts, tc = P["tail_size"], P["tail_curl"]
    py, pz = BY + 2.9, z0 + bl + 0.2
    tail = bone("tail", (0, py, pz), "body", rot=(-1.3 * tc, 0.0, 0.0))
    tail.box((-0.8 * ts, py - 0.8 * ts, pz), (1.6 * ts, 1.6 * ts, 1.9 * ts), "tail")
    mz = pz + 1.8 * ts
    mid = bone("tail_mid", (0, py, mz), "tail", rot=(-0.8 * tc, 0.0, 0.0))
    mid.box((-1.5 * ts, py - 1.4 * ts, mz - 0.2 * ts), (3.0 * ts, 2.8 * ts, 2.8 * ts), "plume")
    mid.box((-1.25 * ts, py - 2.2 * ts, mz + 0.2 * ts), (2.5 * ts, 1.0 * ts, 2.2 * ts), "plume")
    tz = mz + 2.6 * ts
    tip = bone("tail_tip", (0, py, tz), "tail_mid", rot=(-0.9 * tc, 0.0, 0.0))
    tip.box((-1.15 * ts, py - 1.2 * ts, tz - 0.2 * ts), (2.3 * ts, 2.2 * ts, 1.7 * ts), "plume")

    # leg0..3: back right, back left, front right, front left (the engine's names).
    lw, pw, ph, gap, ll = P["leg_w"], P["paw_w"], P["paw_h"], P["leg_gap"], P["leg_len"]
    zf, zb = z0 + 0.2, z0 + bl - 0.2
    for name, x, z, back in (("leg0", -gap, zb, True), ("leg1", gap, zb, True), ("leg2", gap, zf, False), ("leg3", -gap, zf, False)):
        leg = bone(name, (x, BY + 0.3, z), "body")
        leg.box((x - lw / 2, ph - 0.15, z - lw / 2), (lw, BY + 0.5 - (ph - 0.15), lw), "leg")
        leg.box((x - pw / 2, 0.0, z - pw / 2 - 0.25), (pw, ph, pw + 0.2), "paw")
        if back:
            leg.box((x - 0.55, ph + 0.01, z + lw / 2), (1.1, 0.85 * ll, 0.6), "feather")
        else:
            leg.box((x - 0.5, ph + 0.41, z + lw / 2), (1.0, 0.58 * ll, 0.4), "feather")

    G.clear()
    G.update(P=P, BY=BY, z0=z0, HB=HB, SF=SF, hh=hh, nose_y=nose_y, muzzle_y=HB - 0.15, tail_y=py, tip_z=tz - 0.2 * ts,
             eyes=[(-P["eye_gap"], HB + P["eye_y"]), (P["eye_gap"], HB + P["eye_y"])], ear_base=HB + hh - 0.4)
    return bones


def ear_reach(P, EB, y):
    """How far the hair reaches past the ear's edge at height y: longest low on the ear."""
    eh = P["ear_h"]
    return P["ear_hair"] * (0.15 + 0.95 * np.clip((EB + 0.74 * eh - y) / (0.7 * eh), 0, 1))


# ------------------------------------------------------------------ geometry helpers

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
NORMALS = np.array([(-1, 0, 0), (1, 0, 0), (0, -1, 0), (0, 1, 0), (0, 0, -1), (0, 0, 1)], float)


def rot(r):
    """The engine's rotation matrix: Z * Y * X."""
    cx, sx, cy, sy, cz, sz = math.cos(r[0]), math.sin(r[0]), math.cos(r[1]), math.sin(r[1]), math.cos(r[2]), math.sin(r[2])
    X = np.array([[1, 0, 0], [0, cx, -sx], [0, sx, cx]])
    Y = np.array([[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]])
    Z = np.array([[cz, -sz, 0], [sz, cz, 0], [0, 0, 1]])
    return Z @ Y @ X


def rest(bones):
    """Each bone's rest transform (4x4) from its frame (as built) to the dog's."""
    names = [b.name for b in bones]
    mats = []
    for b in bones:
        m = np.eye(4)
        m[:3, :3] = rot(b.rot)
        m[:3, 3] = b.pivot - m[:3, :3] @ b.pivot
        mats.append((mats[names.index(b.parent)] if b.parent else np.eye(4)) @ m)
    return mats


def face_extent(size, f):
    """The face's width and height in design units."""
    w, h, d = size
    return [(d, h), (d, h), (w, d), (w, d), (w, h), (w, h)][f]


def layout(bones):
    """Every face gets its own texel rectangle, shelf packed. Returns {(bone, box, face): rect}."""
    faces = []
    for bi, b in enumerate(bones):
        for ci, c in enumerate(b.boxes):
            for f in range(6):
                fw, fh = face_extent(c.size, f)
                faces.append(((bi, ci, f), max(1, round(fw * T)), max(1, round(fh * T))))
    faces.sort(key=lambda t: (-t[2], -t[1]))
    rects, x, y, row = {}, 0, 0, 0
    for key, w, h in faces:
        if x + w > SHEET:
            x, y, row = 0, y + row + 1, 0
        rects[key] = (x, y, w, h)
        x += w + 1
        row = max(row, h)
    assert y + row <= SHEET, f"texture full: {y + row} rows"
    return rects


def face_points(c, f, w, h):
    """The points (as built) of a face's texels, (h, w, 3), and each texel's (s, t) in 0..1 (t down)."""
    c0, c1, c2, c3 = (np.array(k, float) for k in CORNERS[f])
    s = (np.arange(w) + 0.5) / w
    t = (np.arange(h) + 0.5) / h
    S, Tt = np.meshgrid(s, t)
    p = c3 + S[..., None] * (c2 - c3) + Tt[..., None] * (c0 - c3)
    return c.origin + p * c.size, S, Tt


# ------------------------------------------------------------------ coats

COLORS = [
    ("fawn", (212, 158, 104)),
    ("cream", (232, 208, 168)),
    ("chocolate", (122, 70, 42)),
    ("black", (52, 42, 44)),
    ("blue", (104, 110, 126)),
    ("red", (182, 100, 54)),
    ("white", (236, 228, 214)),
    ("golden", (208, 150, 70)),
]
PATTERNS = ["solid", "marked", "piebald", "tricolor", "sable", "patched"]
WHITE = np.array([242, 230, 210], float)  # never pure white (Hytale's rule: it kills the shading)
TAN = np.array([200, 136, 80], float)
PINK = np.array([226, 150, 148], float)
NOSE = np.array([74, 46, 46], float)
LIGHT = np.array([0.3, 1.0, -0.55]) / np.linalg.norm([0.3, 1.0, -0.55])
COOL = np.array([0.93, 0.92, 1.06])  # what shadow does to a color besides darkening it


class Coat:
    def __init__(self, ci, pattern, marks):
        self.name, base = COLORS[ci]
        self.pattern, self.marks = pattern, marks
        if pattern == "tricolor" and sum(base) > 300:
            base = (52, 42, 44)
        self.base = np.array(base, float)
        rng = random.Random(f"{self.name}-{pattern}-{marks}")
        # Patches for "patched": centers above the body's underside, radii.
        self.patches = [((rng.uniform(-2, 2), rng.uniform(0.7, 3.2), rng.uniform(-1.5, 2.5)), rng.uniform(1.3, 2.1)) for _ in range(2 + marks)]
        self.blaze = 0.45 if marks == 0 else 0.8
        self.eye_patch = rng.choice([-1, 1])

    def color(self, kind, p, n):
        """The coat's base colors at points p (as built, ..., 3) of one part, face normal n (built)."""
        x, y, z = p[..., 0], p[..., 1], p[..., 2]
        BY, HB, SF = G["BY"], G["HB"], G["SF"]
        c = np.broadcast_to(self.base, p.shape).copy()
        false = np.zeros(x.shape, bool)
        K = lambda *ks: np.bool_(kind in ks)
        chest = (K("ruff") & (y < HB + 0.4)) | (K("mane") & (z < G["z0"] - 0.6) & (y < HB)) | (K("skull") & (y < HB + 0.7)) | (K("muzzle") & (y < HB + 0.3))
        belly = K("skirt") | (K("torso") & (n[1] < 0))
        paw = K("paw") | (K("leg") & (y < G["P"]["paw_h"] + 0.45))
        lower_leg = K("leg", "paw", "feather") & (y < BY + 0.2)
        muzzle = K("muzzle")
        blaze = K("skull") & (np.abs(x) < self.blaze + 0.12 * np.clip(y - (HB + 2.4), 0, None)) & ((z < SF + 0.1) | (y > HB + G["hh"] - 0.1))
        cheek = K("cheek")
        plume = K("plume")
        tip = plume & (z > G["tip_z"])
        plume_under = plume & ((n[1] < 0) | (y < G["tail_y"] - 0.6 * G["P"]["tail_size"]))
        white, tan = false, false
        pat = self.pattern
        if pat == "marked":
            white = chest | belly | paw | tip | (muzzle & (y < HB + 0.8))
        elif pat == "piebald":
            white = chest | belly | lower_leg | muzzle | blaze | cheek | plume_under | (tip & (self.marks == 1)) | K("feather") | K("hip") & (y < BY + 0.7)
        elif pat == "tricolor":
            white = (chest & (y < HB - 0.2)) | paw
            ex, ey = G["eyes"][1]
            brow = K("skull") & (z < SF + 0.1) & (np.abs(y - (ey + 1.15 * G["P"]["eye_size"] + 0.35)) < 0.3) & (np.abs(np.abs(x) - ex) < 0.45)
            tan = ((lower_leg & ~paw) | muzzle | brow | cheek | plume_under | K("feather") | (belly & ~chest)) & ~white
        elif pat == "sable":
            dark = (n[1] > 0) | K("ear", "ear_fringe") | (K("torso", "mane", "tail") & (y > BY + 2.5))
            c[dark] *= 0.66
            c[belly | chest] = (self.base + WHITE) / 2
        elif pat == "patched":
            white = np.ones(x.shape, bool) & ~K("ear", "ear_fringe")
            white &= ~(K("skull", "muzzle", "cheek") & (x * self.eye_patch > self.blaze) & (y > HB + 1.0))
            for (cx, cy, cz), r in self.patches:
                white &= ~(K("torso", "hip", "tail", "plume") & ((x - cx) ** 2 + (y - BY - cy) ** 2 + (z - cz) ** 2 < r * r))
        c[white] = WHITE
        c[tan] = TAN
        if K("ear_fluff"):
            c[:] = (WHITE + PINK) / 2 if pat != "sable" else (self.base + WHITE) / 2
        if K("nose"):
            c[:] = NOSE
        return c


# ------------------------------------------------------------------ painting


def hash1(k, seed):
    return (np.sin(k * 12.9898 + seed * 78.233) * 43758.5453) % 1.0


def fur_tufts(u, up, height, kind, top):
    """Fur painted the way Hytale paints it: soft, lighter tufts shaped like flames rising from
    the bottom edge of a side, darker hollows between them, a second row higher up, fainter.
    u runs across the face, up from its bottom edge (design units). A brightness factor."""
    if kind in ("nose", "eye"):
        return np.ones_like(u)
    if top:
        return 1.0 + 0.04 * np.sin(u * 2 * math.pi / 1.1 + 1.5 * np.sin(up * 1.7))
    out = np.ones_like(u)
    for row, (w, tall, amount) in enumerate(((1.1, 1.3, 0.2), (0.9, 1.0, 0.1))):
        base = row * min(1.4, height * 0.5)
        k = u / w + row * 0.5
        lu = (k % 1.0) - 0.5
        tall_here = tall * (0.65 + 0.35 * hash1(np.floor(k), 9 + row))
        rise = (up - base) / tall_here
        flame = np.clip(1 - np.abs(lu) * 2 - rise, -1, 1)
        inside = (rise > 0) & (rise < 1.2)
        out = out + np.where(inside, amount * np.clip(flame * 2.5, -1, 1), 0)
    # Darker towards the top of a side, where the next part's shadow and the fur's roots are.
    return out * (1.04 - 0.1 * np.clip(up / max(height, 0.5), 0, 1))


def blur(img, sigma):
    """A separable gaussian over a face's texels (edges clamp)."""
    r = max(1, int(round(sigma * 2.5)))
    k = np.exp(-0.5 * (np.arange(-r, r + 1) / sigma) ** 2)
    k /= k.sum()
    for axis in (0, 1):
        pad = [(0, 0)] * img.ndim
        pad[axis] = (r, r)
        q = np.pad(img, pad, mode="edge")
        img = sum(k[i] * np.take(q, range(i, i + img.shape[axis]), axis=axis) for i in range(2 * r + 1))
    return img


def fringe_alpha(u, v_bottom, kind):
    """Zig-zag points along an edge (v_bottom: distance from it): whether a texel is left."""
    period, depth = 0.6, 0.35 * G["P"]["fluff"]
    k = u / period
    tri = np.clip(np.abs((k % 1.0) - 0.5) * 3 - 0.5, 0, 1)  # 0 on a (flat) tip, 1 between tips
    jitter = 0.5 + 0.5 * np.abs(np.sin(np.floor(k) * 12.9898) * 43758.5453 % 1.0)
    return v_bottom > depth * tri * jitter


def occlusion(points, normals, world_boxes):
    """How much of the space in front of each point is outside every box (1 open)."""
    rng = np.random.default_rng(7)
    dirs = rng.normal(size=(24, 3))
    dirs /= np.linalg.norm(dirs, axis=1, keepdims=True)
    blocked = np.zeros(points.shape[:-1])
    total = np.zeros(points.shape[:-1])
    for d in dirs:
        for dist in (0.35, 0.8):
            dd = np.where((normals @ d)[..., None] < 0, -d, d)
            q = points + normals * 0.05 + dd * dist
            inside = np.zeros(points.shape[:-1], bool)
            for inv, lo, hi in world_boxes:
                ql = q @ inv[:3, :3].T + inv[:3, 3]
                inside |= np.all((ql > lo) & (ql < hi), axis=-1)
            w = 1.0 / dist
            blocked += inside * w
            total += w
    return 1.0 - blocked / total


_AO = {}


def paint(bones, rects, coat):
    img = np.zeros((SHEET, SHEET, 4), np.uint8)
    mats = rest(bones)
    world_boxes = [(np.linalg.inv(m), c.origin + 0.02, c.origin + c.size - 0.02) for b, m in zip(bones, mats) for c in b.boxes]
    for key, (rx, ry, rw, rh) in rects.items():
        bi, ci, f = key
        b, m = bones[bi], mats[bi]
        c = b.boxes[ci]
        p, S, Tt = face_points(c, f, rw, rh)
        n = NORMALS[f]
        pw = p @ m[:3, :3].T + m[:3, 3]
        R = m[:3, :3]
        nw = R @ n
        # Light as if the box were rounded: the normal leans towards the box's own edges.
        center, half = c.origin + c.size / 2, np.maximum(c.size / 2, 0.3)
        bulge = (p - center) / half
        bulge[..., np.abs(n) > 0] = 0
        nr = n + 0.75 * bulge
        nr /= np.linalg.norm(nr, axis=-1, keepdims=True)
        nr = nr @ R.T
        light = 0.84 + 0.24 * np.clip(nr @ LIGHT, -1, 1)
        # The same for every coat: worked out once.
        if key not in _AO:
            _AO[key] = 0.45 + 0.55 * occlusion(pw, np.broadcast_to(nw, pw.shape), world_boxes) ** 1.3
        ao = _AO[key]
        col = coat.color(c.kind, p, n)
        # Fur locks, across and along the hair: on the sides the hair hangs down.
        top = abs(nw[1]) > 0.7
        axis_u = 2 if abs(n[0]) > 0 else 0
        u = p[..., axis_u]
        if top:
            up = p[..., 2]
            height = 1.0
        else:
            up = (1 - Tt) * face_extent(c.size, f)[1]
            height = face_extent(c.size, f)[1]
        col = col * fur_tufts(u, up, height, c.kind, top)[..., None]
        # The bottom of a side a little darker; edges catch light at the top, go dark at the
        # bottom (painted bevels make boxes look soft).
        if not top and abs(n[1]) == 0:
            col *= (0.94 + 0.08 * (1 - Tt))[..., None]
            col[0, :] *= 1.08
            col[-1, :] *= 0.9
        elif top:
            col[0, :] *= 1.04
            col[-1, :] *= 1.04
            col[:, 0] *= 1.04
            col[:, -1] *= 1.04
        shade = light * ao
        col = col * shade[..., None] * (1 + (1 - shade)[..., None] * (COOL - 1) * 2.5)
        alpha = np.ones(S.shape, bool)
        fr = c.fringe
        if fr is not None and NORMALS[fr] @ n == 0:
            # Distance from the cut edge, and along it.
            ax = fr // 2
            o = c.origin
            d = p[..., ax] - o[ax] if fr % 2 == 0 else o[ax] + c.size[ax] - p[..., ax]
            along = next(i for i in range(3) if i != ax and n[i] == 0)
            alpha = fringe_alpha(p[..., along] + 0.37 * f, d, c.kind)
        # Soft brushwork: colors and light melt into each other.
        col = blur(col, 0.6)
        col = details(c, b, f, p, n, col)
        if c.kind == "ear" and f == 4:
            col = inner_ear(b, p, col)
        if c.kind == "ear":
            # A flat cut-out: the steps' other sides would show as slivers.
            alpha &= ear_alpha(b, p) & (f in (4, 5))
            # The hair beyond the skin is lighter at its ends.
            out = np.clip(np.abs(p[..., 0] - b.pivot[0]) - ear_core(p[..., 1]), 0, 1)
            col = col * (1 + 0.12 * out)[..., None]
        img[ry : ry + rh, rx : rx + rw, :3] = np.round(np.clip(col, 0, 255)).astype(np.uint8)
        img[ry : ry + rh, rx : rx + rw, 3] = np.where(alpha, 255, 0)
    return bleed(img)


def bleed(img, steps=8):
    """Spreads the colors of painted texels into the clear ones around them (still clear):
    smaller copies of the texture average neighbours, and black there shows as dark seams."""
    rgb = img[..., :3].astype(float)
    have = img[..., 3] > 0
    for _ in range(steps):
        acc = np.zeros_like(rgb)
        cnt = np.zeros(have.shape)
        for dy, dx in ((0, 1), (0, -1), (1, 0), (-1, 0)):
            h2 = np.roll(have, (dy, dx), (0, 1))
            acc += np.roll(rgb, (dy, dx), (0, 1)) * h2[..., None]
            cnt += h2
        grow = ~have & (cnt > 0)
        rgb[grow] = acc[grow] / cnt[grow, None]
        have = have | grow
    img[..., :3] = np.round(rgb).astype(np.uint8)
    return img


def ear_core(y):
    """Half the width of the ear's skin (without hair) at height y."""
    P = G["P"]
    return P["ear_w"] * (1 - (y - G["ear_base"]) / P["ear_h"])


def ear_alpha(b, p):
    """The ear: a triangle with a ragged fringe of hair along its sides."""
    x, y = p[..., 0], p[..., 1]
    EB = G["ear_base"]
    # Like real long-haired chihuahuas: a fan of hair off the edges, longest low on the ear,
    # in a few soft locks.
    k = (y - EB + 0.2) / 0.75
    lock = np.clip(1.3 - np.abs((k % 1.0) - 0.5) * 2.2, 0, 1) ** 0.6
    hair = ear_reach(G["P"], EB, y) * (0.55 + 0.45 * lock * (0.7 + 0.3 * hash1(np.floor(k), 5)))
    return (np.abs(x - b.pivot[0]) < ear_core(y) + hair) & (y < EB + G["P"]["ear_h"] + 0.1)


def inner_ear(b, p, col):
    x0 = b.pivot[0]
    x, y = p[..., 0], p[..., 1]
    P, EB = G["P"], G["ear_base"]
    y0, span, w = EB + 0.3, P["ear_h"] - 0.7, P["ear_w"] / 1.5
    k = 1 - (y - y0) / span
    inside = (np.abs(x - x0) < 1.0 * w * k) & (y > y0)
    deep = (np.abs(x - x0) < 0.55 * w * k) & (y > y0 + 0.3)
    pink = PINK * np.where(deep, 0.86, 1.0)[..., None] * (0.92 + 0.1 * (y[..., None] - y0) / span)
    return np.where(inside[..., None], pink, col)


def details(c, b, f, p, n, col):
    """Eyes, nose and mouth painted on."""
    x, y = p[..., 0], p[..., 1]
    if c.kind == "skull" and f == 4:
        es = G["P"]["eye_size"]
        for ex, ey in G["eyes"]:
            dx, dy = (x - ex) / (1.05 * es), (y - ey) / (1.15 * es)
            r = np.sqrt(dx * dx + dy * dy)
            eye = r < 1.0
            iris = np.array([118, 70, 36], float) * (1.15 - 0.45 * np.clip(-dy, 0, 1))[..., None]
            e = np.where((r < 0.62)[..., None], np.array([22, 16, 16], float), iris)
            e = np.where(((r > 0.86) | ((dy > 0.55) & (r > 0.7)))[..., None], np.array([44, 28, 24], float), e)
            # The shine on the inner upper side and a small one low on the outer side.
            inner = -np.sign(ex)
            big = ((dx - 0.3 * inner) ** 2 + (dy - 0.34) ** 2) < 0.035
            small = ((dx + 0.3 * inner) ** 2 + (dy + 0.36) ** 2) < 0.012
            e = np.where(big[..., None], np.array([250, 246, 240], float), e)
            e = np.where(small[..., None], np.array([214, 196, 182], float), e)
            col = np.where(eye[..., None], e, col)
            # Painted sockets: the fur darker towards the eye.
            ring = np.clip((1.45 - r) / 0.45, 0, 1) * (r >= 1.0)
            col = col * (1 - 0.3 * ring)[..., None]
    if c.kind == "nose":
        ny = G["nose_y"]
        hi = (y > ny + 0.55) & (np.abs(x + 0.15) < 0.3) & ((f == 4) | (f == 3))
        col = np.where(hi[..., None], NOSE * 1.7, col)
        nostril = (f == 4) & (y < ny + 0.35) & (np.abs(np.abs(x) - 0.3) < 0.13)
        col = np.where(nostril[..., None], NOSE * 0.55, col)
    if c.kind == "paw" and f in (3, 4):
        cx = c.origin[0] + c.size[0] / 2
        # Three toes: two soft grooves across the front, down to the floor.
        groove = np.abs(np.abs(x - cx) - 0.29)
        front = (f == 4) & (p[..., 1] < c.origin[1] + c.size[1] - 0.12) | (f == 3) & (p[..., 2] < c.origin[2] + 0.55)
        col = np.where((front & (groove < 0.07))[..., None], col * 0.7, col)
        col = np.where((front & (groove >= 0.07) & (groove < 0.16))[..., None], col * 0.88, col)
        # Claws: small light marks low on the front of each toe.
        toe = np.minimum(np.abs(x - cx), np.abs(np.abs(x - cx) - 0.58))
        claw = (f == 4) & (toe < 0.07) & (p[..., 1] < c.origin[1] + 0.22)
        col = np.where(claw[..., None], np.array([236, 226, 210], float) * 0.9, col)
    if c.kind == "muzzle" and f == 4 and c.size[1] > 1:
        my = G["muzzle_y"]
        line = (np.abs(x) < 0.13) & (y > my + 0.55) & (y < G["nose_y"])
        smile = (np.abs(y - (my + 0.6 - 0.25 * (1 - np.abs(x) / 0.8) ** 2)) < 0.12) & (np.abs(x) < 0.8) & (np.abs(x) > 0.05)
        col = np.where((line | smile)[..., None], col * 0.62, col)
    return col


# ------------------------------------------------------------------ files


def deg(r):
    """Engine rotation (radians) to the file's degrees."""
    return [round(-math.degrees(r[0]), 3), round(math.degrees(r[1]), 3), round(-math.degrees(r[2]), 3)]


def r3(v):
    return [round(float(x), 4) for x in v]


def geo(bones, rects, s, head_scale, ident):
    names = [b.name for b in bones]

    def in_head(b):
        while b is not None:
            if b.name == "head":
                return True
            b = bones[names.index(b.parent)] if b.parent else None
        return False

    hp = bones[names.index("head")].pivot

    def at(b, v):
        v = np.array(v, float)
        if head_scale != 1.0 and in_head(b):
            v = hp + (v - hp) * head_scale
        return v * s

    out = []
    for bi, b in enumerate(bones):
        k = head_scale if (head_scale != 1.0 and in_head(b)) else 1.0
        cubes = []
        for ci, c in enumerate(b.boxes):
            uv = {}
            for f in range(6):
                x, y, w, hh = rects[(bi, ci, f)]
                name = FACE_NAMES[f]
                # The engine reads "down" flipped in v, as Blockbench writes it.
                uv[name] = {"uv": [x, y + hh], "uv_size": [w, -hh]} if name == "down" else {"uv": [x, y], "uv_size": [w, hh]}
            cubes.append({"origin": r3(at(b, c.origin)), "size": r3(c.size * s * k), "uv": uv})
        bone = {"name": b.name, "pivot": r3(at(b, b.pivot)), "cubes": cubes}
        if b.parent:
            bone["parent"] = b.parent
        if any(b.rot):
            bone["rotation"] = deg(b.rot)
        out.append(bone)
    return {
        "format_version": "1.12.0",
        "minecraft:geometry": [{"description": {"identifier": ident, "texture_width": SHEET, "texture_height": SHEET}, "bones": out}],
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


def animations(pose, s):
    """`idle` (holding the pose, with a little life) and `walk` for one pose; `s` scales moves."""
    name = f"animation.chihuahua.{pose}"
    anims = {}
    bones = {}

    def rot_(bone, frames):
        bones.setdefault(bone, {})["rotation"] = keyframes([(t, deg(v)) for t, v in frames])

    def const_rot(bone, v):
        bones.setdefault(bone, {})["rotation"] = deg(v)

    def const_pos(bone, v):
        bones.setdefault(bone, {})["position"] = r3([x * s for x in v])

    if pose == "stand":
        length = 4.0
        rot_("tail", wave(length, 0.5, 0.3, 1))
        rot_("ear_r", [(0, [0, 0, 0]), (2.6, [0, 0, 0]), (2.7, [0, 0, 0.2]), (2.85, [0, 0, 0]), (4, [0, 0, 0])])
        rot_("ear_l", [(0, [0, 0, 0]), (1.1, [0, 0, 0]), (1.2, [0, 0, -0.2]), (1.35, [0, 0, 0]), (4, [0, 0, 0])])
        bones.setdefault("body", {})["scale"] = keyframes([(t, [1.0, round(1.0 + 0.02 * math.sin(math.pi * t), 4), 1.0]) for t in [0, 0.5, 1, 1.5, 2, 2.5, 3, 3.5, 4]])
    elif pose == "sit":
        length = 4.0
        const_pos("body", [0, -0.8, 0.4])
        bones["body"]["rotation"] = deg([0.5, 0, 0])
        for leg in ("leg0", "leg1"):
            const_rot(leg, [1.0, 0, 0])
        for leg in ("leg2", "leg3"):
            const_rot(leg, [-0.5, 0, 0])
        rot_("head", [(0, [-0.4, 0, 0]), (1.5, [-0.4, 0, 0]), (1.8, [-0.4, 0, 0.16]), (2.8, [-0.4, 0, 0.16]), (3.1, [-0.4, 0, 0]), (4, [-0.4, 0, 0])])
        rot_("tail", [(t, [0.5, v[1], 0]) for t, v in wave(length, 1.0, 0.25, 1)])
    else:  # lie
        length = 6.0
        const_pos("body", [0, -1.6, 0])
        for leg in ("leg0", "leg1"):
            const_rot(leg, [1.5, 0, 0])
        for leg in ("leg2", "leg3"):
            const_rot(leg, [-1.5, 0, 0])
        const_pos("head", [0, -0.5, -0.3])
        const_rot("head", [-0.15, 0, 0])
        rot_("tail", [(t, [0.5, 0.6 + v[1], 0]) for t, v in wave(length, 3.0, 0.1, 1)])
        rot_("ear_r", [(0, [0, 0, 0]), (4.0, [0, 0, 0]), (4.1, [0, 0, 0.25]), (4.3, [0, 0, 0]), (6, [0, 0, 0])])
        bones["body"]["scale"] = keyframes([(t, [1.0, round(1.0 + 0.025 * math.sin(2 * math.pi * t / 3.0), 4), 1.0]) for t in [0, 0.75, 1.5, 2.25, 3, 3.75, 4.5, 5.25, 6]])
    anims[f"{name}.idle"] = {"loop": True, "animation_length": length, "bones": bones}

    # Walking: diagonal pairs (each back leg with the front leg on the other side), a bob and a
    # wagging tail; one stride per 1.4 (the engine runs walk animations by distance).
    stride = 1.4
    walk = {}
    for leg, phase in (("leg0", 0.0), ("leg2", 0.0), ("leg1", math.pi), ("leg3", math.pi)):
        walk[leg] = {"rotation": keyframes([(t, deg(v)) for t, v in wave(stride, stride, 0.55, 0, phase)])}
    walk["body"] = {"position": keyframes([(t, r3([0, abs(v[0]) * s, 0])) for t, v in wave(stride, stride, 0.3, 0)])}
    walk["tail"] = {"rotation": keyframes([(t, deg(v)) for t, v in wave(stride, stride / 2, 0.3, 1)])}
    walk["head"] = {"rotation": keyframes([(t, deg(v)) for t, v in wave(stride, stride / 2, 0.05, 0)])}
    anims[f"{name}.walk"] = {"loop": True, "animation_length": stride, "bones": walk}
    return {"format_version": "1.8.0", "animations": anims}


def png(path, img):
    hh, w = img.shape[:2]
    raw = b"".join(b"\x00" + img[r].tobytes() for r in range(hh))

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
    shape = dict(SHAPE)
    path = os.path.join(os.path.dirname(__file__), "shape.json")
    if os.path.exists(path):
        shape.update(json.load(open(path)))
    bones = build(shape)
    rects = layout(bones)
    write_json(os.path.join(ROOT, "models", "chihuahua.geo.json"), geo(bones, rects, ADULT, 1.0, "geometry.chihuahua"))
    write_json(os.path.join(ROOT, "models", "chihuahua_pup.geo.json"), geo(bones, rects, PUP, PUP_HEAD, "geometry.chihuahua_pup"))
    for pose in ("stand", "sit", "lie"):
        suffix = "" if pose == "stand" else f"_{pose}"
        write_json(os.path.join(ROOT, "animations", f"chihuahua{suffix}.animation.json"), animations(pose, ADULT))
        write_json(os.path.join(ROOT, "animations", f"chihuahua_pup{suffix}.animation.json"), animations(pose, PUP))
    out = os.path.join(ROOT, "textures", "entity", "chihuahua")
    os.makedirs(out, exist_ok=True)
    i = 0
    # CHI_COATS=28,3 paints only those (quicker while trying things).
    only = os.environ.get("CHI_COATS")
    for ci in range(len(COLORS)):
        for pattern in PATTERNS:
            for marks in range(2):
                if only is None or str(i) in only.split(","):
                    png(os.path.join(out, f"coat_{i}.png"), paint(bones, rects, Coat(ci, pattern, marks)))
                i += 1
    print(f"{sum(len(b.boxes) for b in bones)} boxes in {len(bones)} bones, {i} coats in {os.path.normpath(ROOT)}")


if __name__ == "__main__":
    main()
