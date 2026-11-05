#!/usr/bin/env python3
"""Way D: the dog the way Hytale models are made. A few dozen boxes and quads (flat planes)
on bones, and one texture at the format's fixed density (1 texel per unit, 64 units a block)
that does the detail: every texel is painted from the flat turnaround view that faces it, snapped
to the palette, with shading baked in. Fluffy edges (ear fringe, bib locks, tail plume, belly
fringe) are quads cut out by the sheet's silhouettes.

    .venv/bin/python tools/blocky.py

Writes out/d_blocky/spec.json (bones, elements with per-face UVs, animation keyframes) and
texture.png; tools/bb_build.js turns them into a Blockbench project and exports .blockymodel,
.blockyanim and .glb. Units are the other ways' voxels (56 from paw to ear tip), so the dog is
0.875 blocks tall; the game can scale it.
"""

import json
import math
import os
import sys

import numpy as np
from PIL import Image
from scipy import ndimage

sys.path.insert(0, os.path.dirname(__file__))
import voxdog as vd  # noqa: E402

sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "..", "..", "..", ".claude",
                                                 "skills", "blocky-model", "scripts")))
from layers import Recorder, bounds as layer_bounds  # noqa: E402
from paint import eye_corner, iris_keys, mark_iris, pack as skill_pack  # noqa: E402
from house import texels  # noqa: E402

# Texels a unit: the house's 32 a block at the dog's export scale (build.SCALE, 0.122 model pixels a
# unit), as the blocks and items are. The layout stays in units (56 from paw to ear tip).
TX = texels(0.122)

# Coats painted on the fly (`layers.py`): the recorder while build() paints, the face's key, and
# the last face's material weights and light from `smooth_colors`.
LAYERS = {"rec": None, "key": None, "cap": None}

ROOT = os.path.join(os.path.dirname(__file__), "..")
V = os.path.join(ROOT, "ref", "views")
OUT = os.environ.get("BLOCKY_OUT") or os.path.join(ROOT, "out", "d_blocky")
H = 56
NOSE_Z = 26  # the nose's front, where the side and top views' noses sit
SHEET = 512
# "smooth" (the default: soft painted light and edges), "painted" (v2: five hard steps) or
# "projected" (v1: the sheet's colors snapped). `blocky.py painted` writes out/d_blocky_v2/, `projected` _v1.
STYLE = "smooth"

# ------------------------------------------------------------------ the sheet's views


class View:
    """One flat view: colors filled outward past the silhouette (so boxes poking out still get a
    sensible color), the silhouette itself, and how world (x, y, z) maps to its pixels."""

    def __init__(self, name, flip=False):
        a = np.asarray(Image.open(os.path.join(V, f"{name}.png")).convert("RGBA")).astype(float)
        if flip:
            a = a[:, ::-1]
        self.mask = a[..., 3] > 127
        rgb = a[..., :3]
        _, near = ndimage.distance_transform_edt(~self.mask, return_indices=True)
        self.rgb = rgb[tuple(near)]
        self.h, self.w = self.mask.shape
        self.s = Image.open(os.path.join(V, "f_front.png")).height / H  # sheet px per unit
        self.name = name

    def sample(self, u, v, units=1.0):
        """Colors and mask at sheet pixel coords (u to the right, v down), median over each
        texel's footprint (`units` units across: 1 for the layout's grid, 1 / TX painting)."""
        foot = self.s * units  # sheet pixels a texel spans
        k = max(1, int(foot * 0.6))
        offs = np.linspace(-foot * 0.35, foot * 0.35, k)
        cols, masks = [], []
        for du in offs:
            for dv in offs:
                ui = np.clip(np.round(u + du).astype(int), 0, self.w - 1)
                vi = np.clip(np.round(v + dv).astype(int), 0, self.h - 1)
                inside = (u + du >= 0) & (u + du < self.w) & (v + dv >= 0) & (v + dv < self.h)
                cols.append(self.rgb[vi, ui])
                masks.append(self.mask[vi, ui] & inside)
        return np.median(np.stack(cols), 0), np.mean(np.stack(masks), 0) > 0.5


def legs_center(view):
    cols = np.nonzero(view.mask[-20:].any(0))[0]
    return (cols.min() + cols.max()) / 2


FRONT = View("f_front")
BACK = View("f_back", flip=True)
SIDE_R = View("f_side_r")  # head to the right: seen from -x
SIDE_L = View("f_side_l", flip=True)  # flipped so the head is right too: seen from +x
TOP = View("f_top")  # head down, image right = +x


def project(p, n, units=1.0):
    """Color and silhouette for world points p (..., 3) on faces with normal n (3,), from the
    view facing that way."""
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    ax = int(np.abs(n).argmax())
    sgn = np.sign(n[ax])
    if ax == 2 and sgn > 0:
        vw, u = FRONT, legs_center(FRONT) + x * FRONT.s
    elif ax == 2:
        vw, u = BACK, legs_center(BACK) + x * BACK.s
    elif ax == 0:
        vw = SIDE_L if sgn > 0 else SIDE_R
        nose = np.nonzero(vw.mask.any(0))[0].max()
        u = nose - (NOSE_Z - z) * vw.s
    else:
        if sgn < 0:  # the belly: the sheet's bottom view is unreliable; use the side's lowest color
            vw = SIDE_R
            nose = np.nonzero(vw.mask.any(0))[0].max()
            u = nose - (NOSE_Z - z) * vw.s
            v = np.full(x.shape, vw.h - 1 - 1.5 * vw.s)
            return vw.sample(u, v, units)
        vw = TOP
        mid = np.nonzero(TOP.mask[TOP.h // 3: 2 * TOP.h // 3].any(0))[0]
        u = (mid.min() + mid.max()) / 2 + x * TOP.s
        nose_row = np.nonzero(TOP.mask.any(1))[0].max()
        v = nose_row - (NOSE_Z - z) * TOP.s
        return vw.sample(u, v, units)
    v = vw.h - 1 - y * vw.s
    return vw.sample(u, v, units)


# ------------------------------------------------------------------ the model

L = {
    "spine_y": 19, "hip_z": -14, "mid_z": -1,
    "neck": (0, 26, 9), "head": (0, 31, 9),
    "ear_x": 9, "ear_y": 43, "ear_z": 7,
    "tail": [(0, 22, -17), (0, 24, -19.5), (0, 34, -19)],  # each joint inside both parts it joins
    "leg_x_f": 4.5, "leg_x_b": 6, "leg_z_f": 9, "leg_z_b": -15,
    "leg_top": 14, "knee": 8, "ankle": 3,
    # Sit and lie, fit with tools/ground.py so the paws and body rest on the ground.
    "sit_angle": 19, "sit_hip": -70, "sit_knee": 144, "sit_drop": 7.5, "sit_back": 0,
    "lie_front": -74, "lie_hip": -86, "lie_knee": 160, "lie_drop": 8.4, "lie_neck": 70, "lie_head": -58,
}
FACES = {"north": (2, -1), "south": (2, 1), "west": (0, -1), "east": (0, 1), "down": (1, -1), "up": (1, 1)}


def el(name, bone, lo, hi, quad=None, cut=False, rot=None, origin=None, view=None, src=None):
    """A box from lo to hi (units), or a quad: `quad` names its one face (e.g. "south" faces +z)
    and the box is flat along that axis. `cut`: transparent where the sheet's silhouette is
    empty. `rot`: (x, y, z) degrees about `origin` (voxdog's convention). `src`: (point, scale)
    for a part made bigger than the sheet's: it is painted from the sheet at its own size."""
    return dict(name=name, bone=bone, lo=list(map(float, lo)), hi=list(map(float, hi)), quad=quad, cut=cut,
                rot=rot, origin=list(origin) if origin else None, view=view,
                src=[list(map(float, src[0])), src[1]] if src else None)


def elements():
    E = []
    # Body: the front half on spine_1, the back half on spine_0.
    E.append(el("chest", "spine_1", (-8, 12, -1), (8, 26, 13)))
    E.append(el("hips", "spine_0", (-7.5, 13, -19), (7.5, 24.6, -1)))  # a bit inside the chest: the two bend apart
    E.append(el("britches", "spine_0", (-7, 14, -22), (7, 24, -19)))
    E.append(el("neck", "neck", (-7.5, 22, 1), (7.5, 31, 11)))
    # The bib: a solid block and a quad in front of it cut to the sheet's locks.
    E.append(el("bib", "spine_1", (-10, 14, 11), (10, 29, 16)))  # short: the locks' V hangs below it
    E.append(el("bib_locks", "spine_1", (-12, 4, 16.5), (12, 30, 16.5), quad="south", cut=True))
    # Belly fringe along both sides, cut from the side views.
    for side, face in ((1, "east"), (-1, "west")):
        x = side * 9  # clear of the front paws' sides, which swing in the same plane
        xb = side * 10.6  # outside the front one (9), the thigh (9.5) and the back paw (10)
        E.append(el(f"fringe_front_{'l' if side > 0 else 'r'}", "spine_1", (x, 8, -1), (x, 16, 14), quad=face, cut=True))
        E.append(el(f"fringe_back_{'l' if side > 0 else 'r'}", "spine_0", (xb, 8, -22), (xb, 16, -1), quad=face, cut=True))
    # Head.
    E.append(el("skull", "head", (-11, 28, 0), (11, 46, 20)))
    # Muzzle: the upper part on the head, the lower jaw on its own bone (hinged at the back) so
    # the mouth opens; a dark box inside is the mouth you see when it does; the tongue on its
    # own bone pivots at the lip so it can hang out over the jaw.
    E.append(el("muzzle", "head", (-5, 30, 16), (5, 35, 25)))
    E.append(el("mouth", "head", (-3.5, 28.5, 18), (3.5, 31, 21.3)))
    E.append(el("jaw", "jaw", (-4, 27, 17.5), (4, 30, 22.5)))  # shorter than the muzzle: the tongue bends out of sight
    # The tongue: a base lying in the mouth on the jaw, then two parts that bend over the lip's
    # top edge (each hinged on its top surface, so the top stays one surface), narrowing 5-4-3.
    E.append(el("tongue_base", "jaw", (-2.5, 29.4, 19.5), (2.5, 30.4, 22)))
    E.append(el("tongue", "tongue", (-2, 29.4, 22.2), (2, 30.4, 24.6)))  # its hinge (z 22.2) within reach of the base
    E.append(el("tongue_tip", "tongue_tip", (-1.5, 29.4, 24.6), (1.5, 30.4, 27.6)))
    # Eyelids: a quad per eye, hinged at the eye's top. Open (the rest pose), it lies flat inside
    # the head; a blink swings it down over the eye (-90 on X). Rotation only, so every format
    # plays it, and a missing keyframe leaves the eyes open, never shut.
    for name, x0, x1, top, bottom in eyelid_boxes():
        E.append(el(name, name, (x0, top, LID_Z - (top - bottom)), (x1, top, LID_Z), quad="down"))
    E.append(el("nose", "head", (-2, 33, 25), (2, 35, 26)))
    # No cheek fur quads: flat and facing front, they read as a second small ear from the side.
    # Ears: solid, a stack of thin boxes (2 units) following the front view's ear outline, wide at
    # the base and narrowing to the tip, and one double-sided quad down the middle cut to the
    # fringe; leaning out and turned toward the front.
    for side, bone in ((1, "ear_L"), (-1, "ear_R")):
        E.extend(ear_parts(side, bone))
    # Tail: base, plume, curl; side quads cut from the side views carry the plume's outline.
    E.append(el("tail_base", "tail_0", (-3, 19, -21), (3, 25, -15)))
    E.append(el("tail_plume", "tail_1", (-6, 22, -29), (6, 37, -18)))
    E.append(el("tail_curl", "tail_2", (-5, 32, -22), (5, 41, -10)))
    for side, face in ((1, "east"), (-1, "west")):
        x = side * 6.5
        s = "l" if side > 0 else "r"
        E.append(el(f"plume_side_{s}", "tail_1", (x, 14, -31), (x, 36, -18), quad=face, cut=True))
        E.append(el(f"curl_side_{s}", "tail_2", (side * 7, 32, -24), (side * 7, 45, -8), quad=face, cut=True))
    # Legs: upper, lower, paw; hind thighs on the upper bones.
    for leg in vd.LEGS:
        front = leg[0] == "F"
        lx = (L["leg_x_f"] if front else L["leg_x_b"]) * (1 if leg[1] == "L" else -1)
        lz = L["leg_z_f"] if front else L["leg_z_b"]
        E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (lx - 3, 8, lz - 3), (lx + 3, 14, lz + 3)))
        E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (lx - 2.5, 3, lz - 2.5), (lx + 2.5, 8, lz + 2.5)))  # thinner than the upper leg: folded, their sides never share a plane
        E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (lx - 4, 0, lz - 3.5), (lx + 4, 3, lz + 4.5)))  # 8 wide: three toes
        if not front:
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (lx - 3.5, 10, lz - 5), (lx + 3.5, 21, lz + 5)))
    return E


def ear_core_bands():
    """The left ear's outline in its quad's texels, shaved of the fringe, as (top, bottom, x0,
    x1) bands four texels tall; the right ear mirrors them so the two match."""
    ox, oy, oz = L["ear_x"], L["ear_y"], L["ear_z"]
    rot, origin = (0, 18, -14), np.array((ox, oy, oz), float)
    lo, hi = (ox - 4, oy - 6, oz), (ox + 13, oy + 14, oz)
    corner, du, dv, w, h, _ = face_frame(lo, hi, "south")
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
    p = (corner + uu[..., None] * du + vv[..., None] * dv - origin) @ rotm(rot).T + origin
    _, inside = project(p, np.array([0, 0, 1.0]))
    core = ndimage.binary_erosion(inside, iterations=2)
    bands = []
    for v0 in range(0, h, 4):
        band = core[v0:v0 + 4]
        cols = np.nonzero(band.sum(0) >= 2)[0]
        if len(cols) >= 2:
            bands.append((hi[1] - v0, hi[1] - v0 - len(band), lo[0] + cols.min(), lo[0] + cols.max() + 1))
    return bands


EAR_SCALE = 1.35  # the ears, bigger than the sheet's, scaled about their base


def ear_parts(side, bone):
    s = "l" if side > 0 else "r"
    ex, oy, oz = L["ear_x"], L["ear_y"], L["ear_z"]
    ox = side * ex
    big = lambda v, o: int(round(o + (v - o) * EAR_SCALE))  # noqa: E731
    x0, x1 = sorted((side * big(ex - 4, ex), side * big(ex + 13, ex)))
    rot, origin = (0, side * 18, -side * 14), (ox, oy, oz)
    src = (origin, EAR_SCALE)  # paint from the sheet's ear at its own size
    parts = [el(f"fringe_ear_{s}", bone, (x0, big(oy - 6, oy), oz), (x1, big(oy + 14, oy), oz), quad="south", cut=True,
                rot=rot, origin=origin, view="front", src=src)]
    for k, (top, bottom, a, b) in enumerate(ear_core_bands()):
        a, b = sorted((side * big(a, ex), side * big(b, ex)))
        parts.append(el(f"ear_core_{s}{k}", bone, (a, big(bottom, oy), oz - 1), (b, big(top, oy), oz + 1), rot=rot, origin=origin, src=src))
    return parts


# ------------------------------------------------------------------ geometry and painting


def face_frame(lo, hi, face, tx=1.0):
    """A face's corner, its u and v directions (u right, v down when seen from outside, as
    Blockbench lays face UVs out) and its size in texels at `tx` texels a unit (1: the layout's
    unit grid, which the ear's bands are cut on; TX: the painted texture)."""
    lo, hi = np.array(lo), np.array(hi)
    ax, sgn = FACES[face]
    X, Y, Z = np.eye(3)
    if face == "south":
        corner, du, dv = (lo[0], hi[1], hi[2]), X, -Y
    elif face == "north":
        corner, du, dv = (hi[0], hi[1], lo[2]), -X, -Y
    elif face == "east":
        corner, du, dv = (hi[0], hi[1], hi[2]), -Z, -Y
    elif face == "west":
        corner, du, dv = (lo[0], hi[1], lo[2]), Z, -Y
    elif face == "up":
        corner, du, dv = (lo[0], hi[1], lo[2]), X, Z
    else:
        corner, du, dv = (lo[0], lo[1], hi[2]), X, -Z
    size = hi - lo
    w = abs(size @ du)
    h = abs(size @ dv)
    n = np.zeros(3)
    n[ax] = sgn
    tw, th = int(round(tx * w)), int(round(tx * h))
    if w > 0 and h > 0:  # a small face is still a texel
        tw, th = max(1, tw), max(1, th)
    return np.array(corner, float), du, dv, tw, th, n


def rotm(rot):
    return vd.qmat(vd.quat(*rot)) if rot else np.eye(3)


COAT = ["brown_dark", "brown", "brown_light", "tan", "cream", "white"]


def palette_for(e):
    """Which colors a part may use: the face may have eyes, nose and pink; ears pink; the rest
    only the coat's colors (keeps the sheet's orange from snapping to pink on the body)."""
    if e["bone"] == "head":
        return COAT + ["black", "eye", "nose", "highlight"]  # no pink: the ears behind would bleed in
    if e["bone"].startswith("ear"):
        return COAT + ["pink"]
    return COAT


KEEP = [vd.IDX[n] for n in ("black", "eye", "nose", "highlight", "pink")]


def clean(idx, passes=2):
    """Majority filter over each texel's 3x3 neighbors: clears the speckle the sheet's own cube
    shading leaves, so the coat reads as painted areas. Eyes, nose and inner ears stay."""
    for _ in range(passes):
        h, w = idx.shape
        pad = np.pad(idx, 1, mode="edge")
        votes = np.zeros((len(vd.RGB), h, w))
        for dy in range(3):
            for dx in range(3):
                n = pad[dy:dy + h, dx:dx + w]
                votes[n, np.arange(h)[:, None], np.arange(w)[None, :]] += 1 + (dy == 1 and dx == 1)
        best = votes.argmax(0)
        idx = np.where(np.isin(idx, KEEP), idx, best)
    return idx


SHADE = {"up": 1.07, "down": 0.74, "north": 0.9, "south": 1.0, "east": 0.94, "west": 0.94}


def projected_colors(idx, face, vv, h, p):
    """v1: the sheet's colors snapped to the palette, shaded by face direction in bands."""
    col = vd.RGB[idx]
    shade = SHADE[face] * np.ones(idx.shape)
    if face not in ("up", "down"):
        band = np.floor(vv / 3) * 3 / max(h, 1)
        shade = shade * (1.05 - 0.12 * band)
    shade = shade * (0.9 + 0.1 * np.clip(p[..., 1] / 20, 0, 1))
    return np.clip(col * shade[..., None], 0, 255)


# Painted style: the sheet only says which area a texel is in; each area has one base color and
# the light is painted over it.
REGION = {"brown_dark": "brown", "brown": "brown", "brown_light": "orange", "tan": "orange", "cream": "white",
          "white": "white", "grey_white": "white", "pink": "pink", "black": "eye", "eye": "brown",
          "highlight": "brown", "nose": "nose"}  # the sheet's eyes and catchlights are not projected
REGIONS = ["brown", "orange", "white", "pink", "eye", "highlight", "nose"]
BASE = {"brown": (114, 57, 30), "orange": (168, 90, 46), "white": (244, 231, 214), "pink": (224, 148, 136),
        "eye": (34, 23, 21), "highlight": (252, 250, 243), "nose": (108, 52, 52)}
FUR = {"brown", "orange", "white"}


def majority(reg, keep, passes=3):
    for _ in range(passes):
        h, w = reg.shape
        pad = np.pad(reg, 1, mode="edge")
        votes = np.zeros((len(REGIONS), h, w))
        for dy in range(3):
            for dx in range(3):
                votes[pad[dy:dy + h, dx:dx + w], np.arange(h)[:, None], np.arange(w)[None, :]] += 1 + (dy == 1 and dx == 1)
        reg = np.where(np.isin(reg, keep), reg, votes.argmax(0))
    return reg


def ramp(rgb):
    """Five shades for one base color, darkest first. Shadows lean toward purple and get a bit
    more saturated, highlights lean toward yellow and a bit less; nothing reaches pure black or
    white (both break in-game lighting)."""
    import colorsys
    h, sat, v = colorsys.rgb_to_hsv(*(np.array(rgb) / 255))

    def toward(h, target, amt):
        d = (target - h + 0.5) % 1.0 - 0.5
        return (h + np.sign(d) * min(abs(d), amt)) % 1.0

    out = []
    for k in (-2, -1, 0, 1, 2):
        if k < 0:
            hh, ss, vv = toward(h, 0.74, 0.03 * -k), min(1, sat + 0.07 * -k), v * (1 - 0.16 * -k)
        else:
            hh, ss, vv = toward(h, 0.12, 0.015 * k), sat * (1 - 0.14 * k), min(0.96, v * (1 + 0.08 * k))
        out.append(np.array(colorsys.hsv_to_rgb(hh, ss, max(vv, 0.1))) * 255)
    return np.array(out)


RAMPS = {r: ramp(c) for r, c in {"brown": (118, 60, 34), "orange": (176, 98, 52), "white": (236, 224, 206),
                                  "pink": (220, 146, 138), "nose": (112, 56, 58), "eye": (44, 30, 34),
                                  "highlight": (244, 238, 226)}.items()}
FACE_LIGHT = {"up": 0.9, "south": 0.2, "east": 0.2, "west": -0.3, "north": -0.5, "down": -1.6}

# A designed eye, stamped on the face (the sheet's eyes are not projected anywhere): dark (not
# black) with a two-texel catchlight up and toward the light, a warm iris below.
EYE = [".DDD.",
       "DDDHD",
       "DDHHD",
       "DIIID",
       ".III."]
EYE_COLORS = {"D": (46, 30, 36), "H": (240, 234, 222), "I": (104, 58, 40)}


def painted_colors(idx, rgb, face, uu, vv, w, h, p, seed, quad=False):
    """Paint like a hand-painted block-game texture: clean color areas from the sheet, each
    lit from above-front-left with a five-step hue-shifted ramp: lighter tops and top edges,
    darker bottoms, outer columns and ground contact, a shadow where brown meets white, a few
    deliberate fur strokes, no noise."""
    reg = np.vectorize(lambda k: REGIONS.index(REGION[vd.NAMES[k]]))(idx)
    reg = np.where(reg == REGIONS.index("eye"), REGIONS.index("brown"), reg)  # eyes are stamped
    reg = majority(reg, [REGIONS.index(r) for r in ("highlight", "nose", "pink")])
    t = vv / max(h, 1)
    L = np.full((h, w), FACE_LIGHT[face])
    if face not in ("up", "down"):
        L += 0.5 - 1.1 * t  # light from above
        if not quad:
            L[0, :] += 0.6  # the box's top edge catches the light
            L[-1, :] -= 0.3
            if w >= 6:
                L[:, 0] -= 0.35
                L[:, -1] -= 0.35
    else:
        if w >= 4 and h >= 4:
            L[[0, -1], :] -= 0.3
            L[:, [0, -1]] -= 0.3
    L -= 0.6 * (p[..., 1] < 3)  # contact shadow near the ground
    br, wh = REGIONS.index("brown"), REGIONS.index("white")
    near = lambda m: ndimage.binary_dilation(m) & ~m  # noqa: E731
    L -= 0.8 * (near(reg == wh) & (reg == br)) + 0.3 * (near(reg == br) & (reg == wh))
    # Fur: a few short strokes one shade darker on big coat areas, staggered by column.
    if face not in ("up", "down") and w >= 6 and h >= 6:
        cols = (np.arange(w) * 7 + seed) % 5 == 0
        rows = ((np.arange(h)[:, None] + np.arange(w)[None, :] * 3 + seed) % 7) < 3
        L -= 0.55 * (cols[None, :] & rows & np.isin(reg, [br, wh, REGIONS.index("orange")]))
    k = np.clip(np.round(L), -2, 2).astype(int) + 2
    col = np.zeros((h, w, 3))
    for r, name in enumerate(REGIONS):
        m = reg == r
        if m.any():
            col[m] = RAMPS[name][k[m]]
    return col


# Smooth style (the default, like Hytale's own creatures): the same clean areas, but big and
# simple (the coat is one brown whose lights turn golden; orange only on the ear fringe), soft
# tufted edges between colors, continuous light in fine steps, real ambient occlusion where parts
# meet, and a gentle strand pattern instead of single dark strokes.
SMOOTH_BASE = {"brown": (120, 58, 32), "orange": (184, 104, 54), "white": (238, 226, 208), "pink": (222, 150, 140),
               "nose": (92, 46, 48), "eye": (44, 30, 34), "highlight": (244, 238, 226)}
SHADOW_HUE = {"white": 0.05, "pink": 0.98, "brown": 0.99, "orange": 0.02}  # warm shadows; others lean purple
STEP = 0.25  # light quantum, in ramp steps: fine enough to read as smooth, still clean pixels
KS = np.arange(-3, 3 + STEP / 2, STEP)


def tone(rgb, k, shadow_hue=0.74):
    """One shade of a base color, k ramp steps from it (negative darker), hue-shifted."""
    import colorsys
    h, s, v = colorsys.rgb_to_hsv(*(np.array(rgb) / 255))

    def toward(h, target, amt):
        d = (target - h + 0.5) % 1.0 - 0.5
        return (h + np.sign(d) * min(abs(d), amt)) % 1.0

    if k < 0:
        hh, ss, vv = toward(h, shadow_hue, 0.025 * -k), min(1, s + 0.06 * -k), v * (1 - 0.15 * -k)
    else:
        hh, ss, vv = toward(h, 0.12, 0.02 * k), s * (1 - 0.13 * k), min(0.96, v * (1 + 0.09 * k))
    return np.array(colorsys.hsv_to_rgb(hh, ss, max(vv, 0.08))) * 255


TONES = {r: np.array([tone(c, k, SHADOW_HUE.get(r, 0.74)) for k in KS]) for r, c in SMOOTH_BASE.items()}
L_ = L  # the landmarks, under a name the painter's light array doesn't shadow
BOXES = []  # (name, lo, hi) of the solid boxes, for ambient occlusion; filled by build()


def occlusion(p, n, du, dv, own):
    """How much of the space just outside each texel is taken by other boxes (0..1)."""
    du, dv = du / np.linalg.norm(du), dv / np.linalg.norm(dv)
    hits, count = 0, 0
    for d in (0.6, 1.6, 3.0):
        for a, b in ((0, 0), (1.5, 0), (-1.5, 0), (0, 1.5), (0, -1.5)):
            q = p + n * d + du * a + dv * b
            inside = np.zeros(p.shape[:2], bool)
            for name, lo, hi in BOXES:
                if name != own:
                    inside |= np.all((q > lo) & (q < hi), axis=-1)
            hits = hits + inside * (1.0 / d)
            count += 1.0 / d
    return hits / count


def smooth_colors(idx, e, face, uu, vv, w, h, p, n, du, dv, seed, quad=False):
    reg = np.vectorize(lambda k: REGIONS.index(REGION[vd.NAMES[k]]))(idx)
    br, orng, wh = (REGIONS.index(r) for r in ("brown", "orange", "white"))
    reg = np.where(reg == REGIONS.index("eye"), br, reg)  # eyes are stamped
    nose = reg == REGIONS.index("nose")
    if nose.any() and not (e["name"] == "nose" or (e["name"] == "muzzle" and face == "south")):
        if nose.all():
            reg[:] = wh
        else:  # the side and top views' noses: fill from the nearest other color
            _, near = ndimage.distance_transform_edt(nose, return_indices=True)
            reg = reg[tuple(near)]
    reg = np.where(reg == orng, br, reg)  # one coat color, ears too; its lights do the gold
    if e["name"].startswith("ear_core") and face not in ("south", "north"):
        reg[:] = br  # the ear's thin edges
    if e["name"].endswith("_paw"):
        reg[:] = wh  # white feet; the top view only sees the body above them
    coat = np.isin(reg, [br, orng, wh])
    # Big simple shapes: blur each coat area and take the strongest, then drop tiny islands (not at
    # 32 a block, where a mark one texel wide is the design).
    if min(h, w) >= 3 and TX >= 0.75:
        score = np.stack([ndimage.gaussian_filter((reg == r).astype(float), 1.1) for r in range(len(REGIONS))])
        reg = np.where(coat, score.argmax(0), reg)
        reg = majority(reg, [REGIONS.index(r) for r in ("highlight", "nose", "pink")], passes=2)
    # Tufts: on side faces the coat above hangs into the color below in short points.
    if face not in ("up", "down") and h >= 4:
        off = np.array([0, 1, 2, 1, 0, 1, 2, 2, 1])[(np.arange(w) + seed) % 9]
        if TX < 0.75:  # a point a texel long at most, at 32 a block
            off = np.minimum(off, 1)
        src = np.clip(np.arange(h)[:, None] - off[None, :], 0, h - 1)
        hang = reg[src, np.arange(w)[None, :]]
        reg = np.where(coat & np.isin(hang, [br, orng, wh]), hang, reg)
    # Light, continuous: from above, soft top edge, sides turning away, soft bottom.
    t = vv / max(h, 1)
    stacked = e["name"].startswith("ear_core")  # boxes stacked into one ear: lit as one piece
    if stacked:
        t = np.clip((L_["ear_y"] + 14 - p[..., 1]) / 20, 0, 1)
    L = np.full((h, w), FACE_LIGHT[face])
    if face not in ("up", "down"):
        bright = e["name"] == "bib_locks"  # a tall white fringe: a gentle falloff, or its lower half turns grey
        L += (0.3 - 0.5 * t) if bright else (0.6 - 1.2 * t)
        if not quad and not stacked:
            L += 0.5 * np.exp(-(vv - 0.5) / 1.0) - 0.3 * np.exp(-(h - vv - 0.5) / 1.2)
            if w >= 6:
                L -= 0.35 * (np.exp(-(uu - 0.5) / 1.3) + np.exp(-(w - uu - 0.5) / 1.3))
    elif w >= 4 and h >= 4:
        edge = np.minimum(np.minimum(uu, w - uu), np.minimum(vv, h - vv))
        L -= 0.4 * np.exp(-(edge - 0.5) / 1.3)
    L -= 1.0 * np.exp(-np.maximum(p[..., 1], 0) / 2.0)  # ground contact
    if not quad:
        L -= (1.2 if e["name"].endswith("_paw") else 2.2) * occlusion(p, n, du, dv, e["name"])
    # Strands: soft streaks along the fur (down on the sides, back on top), subtle.
    rng = np.random.default_rng(seed)
    g = ndimage.gaussian_filter(rng.standard_normal((h, w)), (1.6, 0.45))
    L += (0.1 if e["name"] == "bib_locks" else 0.22) * g / max(g.std(), 0.2) * coat  # a texel or two: not blown up
    k = np.clip(np.round(L / STEP) * STEP, KS[0], KS[-1])
    ki = np.round((k - KS[0]) / STEP).astype(int)
    # Soft edges: each texel mixes the areas around it (about one texel of blend).
    weights = np.stack([ndimage.gaussian_filter((reg == r).astype(float), 0.6) for r in range(len(REGIONS))])
    sharp = ~coat | np.isin(reg, [REGIONS.index(r) for r in ("nose", "pink", "highlight")])
    col = np.zeros((h, w, 3))
    total = np.zeros((h, w))
    used = np.zeros((len(REGIONS), h, w))
    for r, name in enumerate(REGIONS):
        if not (reg == r).any() and weights[r].max() < 1e-3:
            continue
        wr = np.where(sharp, (reg == r).astype(float), weights[r] * np.isin(r, [br, orng, wh]))
        col += wr[..., None] * TONES[name][ki]
        total += wr
        used[r] = wr
    LAYERS["cap"] = {"weights": used / np.maximum(total, 1e-6), "level": ki}
    return col / np.maximum(total, 1e-6)[..., None]


# Round, with a big catchlight up toward the light, a small second sparkle low on the other
# side and a warm iris glowing at the bottom: the cute look of Hytale's creatures.
EYE_SMOOTH = [".DDDD.",
              "DHHDDD",
              "DHHDDD",
              "DDDDSD",
              "DIIIID",
              ".IIII."]
EYE_SMOOTH_COLORS = {"D": (36, 25, 31), "H": (240, 242, 244), "S": (190, 196, 206), "I": (120, 70, 44)}


# Eye types to choose from (BLOCKY_EYES=<name>). The first ones are in the style of Hytale's creatures
# (from its corgi): big (about 8 texels wide on a 22-wide face), a dark ring around a near-black
# pupil with a warmer glow along the bottom, a cool white 2x2 highlight up top and a small
# grey-blue sparkle, set in darker fur with lighter fur below. They differ in kind (iris color,
# shape, lid, whites, fur mask), not in expression. Keys: O ring, P pupil, I/i lower iris,
# H highlight, s sparkle, L lid line, W white, A/a iris color, w lighter fur under the eye.
_P, _H, _S = (16, 12, 14), (232, 236, 240), (150, 162, 178)
_RING, _GLOW, _LID = (62, 36, 26), (98, 56, 36), (40, 26, 26)
EYES = {
    "corgi": dict(pattern=["..OOOO..",
                           ".OHHPPO.",
                           "OPHHPPPO",
                           "OPPPPPPO",
                           "OPPPPsPO",
                           ".OIIIIO.",
                           "..OOOO.."], colors={"O": _RING, "P": _P, "H": _H, "s": _S, "I": _GLOW}, halo="round"),
    "amber": dict(pattern=["..LLLL..",
                           ".LAAAAL.",
                           "LAHHPPAL",
                           "LAHPPPAL",
                           "LAPPPPAL",
                           ".LaAAaL.",
                           "..LLLL.."], colors={"L": _LID, "A": (156, 92, 38), "a": (196, 128, 58), "P": _P, "H": _H}, halo="round"),
    "oval": dict(pattern=["..OO..",
                          ".OHHO.",
                          "OPHHPO",
                          "OPPPPO",
                          "OPPPPO",
                          "OPPsPO",
                          ".OIIO.",
                          "..OO.."], colors={"O": _RING, "P": _P, "H": _H, "s": _S, "I": _GLOW}, halo="round", y=39),
    "gradient": dict(pattern=["..PPP..",
                              ".PHHPP.",
                              "PPHHPPP",
                              "PPPPPPP",
                              "PgggggP",
                              ".IIsII.",
                              "..iii.."], colors={"P": _P, "H": _H, "s": _S, "g": (52, 30, 24), "I": (104, 58, 36),
                                                  "i": (146, 88, 50)}, halo="round"),
    "whites": dict(pattern=[".LLLLLL.",
                            "WOHHPPOW",
                            "WOHPPPOW",
                            "WOPPPPOW",
                            ".LOIIOL.",
                            "..wwww.."], colors={"L": _LID, "W": (212, 202, 188), "O": _RING, "P": _P, "H": _H, "I": _GLOW,
                                                 "w": (176, 118, 80)}, halo="below"),
    "mask": dict(pattern=["..OOOO..",
                          ".OHHPPO.",
                          "OPHHPPPO",
                          "OPPPPPPO",
                          "OPPPPsPO",
                          ".OIIIIO.",
                          "..OOOO.."], colors={"O": _RING, "P": _P, "H": _H, "s": _S, "I": _GLOW}, halo="mask"),
}
# Between the Hytale corgi and the cute sparkle: sparkle's size and roundness with the corgi's
# dark ring, cool highlight and sparkle, warm glow and soft fur ring.
EYES.update({
    "between": dict(pattern=[".OOOO.",
                             "OHHPPO",
                             "OHHPPO",
                             "OPPPsO",
                             "OIIIIO",
                             ".IIII."], colors={"O": _RING, "P": _P, "H": _H, "s": _S, "I": _GLOW}, halo="round"),
    "between_wide": dict(pattern=[".OOOOO.",
                                  "OHHPPPO",
                                  "OHHPPPO",
                                  "OPPPPsO",
                                  "OIIIIIO",
                                  ".IIIII."], colors={"O": _RING, "P": _P, "H": _H, "s": _S, "I": _GLOW}, halo="round"),
    "between_soft": dict(pattern=[".PPPP.",
                                  "PHHPPP",
                                  "PHHPPP",
                                  "PPPPsP",
                                  "OIIIIO",
                                  ".IIII."], colors={"O": _RING, "P": (30, 20, 22), "H": _H, "s": _S, "I": _GLOW}, halo="below"),
})
# Cute, simpler ones (rounder, more cartoon).
EYES.update({
    "sparkle": dict(pattern=EYE_SMOOTH, colors=EYE_SMOOTH_COLORS, halo="below"),
    "glossy": dict(pattern=[".DDDD.",
                            "DHHDDD",
                            "DHHDDD",
                            "DDDDDD",
                            "DDDDHD",
                            "DIIIID",
                            ".IIII."], colors={"D": (36, 25, 31), "H": (240, 242, 244), "I": (120, 70, 44)}, halo="below"),
    "button": dict(pattern=[".DD.",
                            "DHDD",
                            "DDDD",
                            ".DD."], colors={"D": (36, 25, 31), "H": (240, 242, 244)}, halo="below"),
    "anime": dict(pattern=[".DD.",
                           "DHHD",
                           "DHDD",
                           "DDDD",
                           "DDSD",
                           "DIID",
                           ".II."], colors={"D": (36, 25, 31), "H": (240, 242, 244), "S": (190, 196, 206), "I": (120, 70, 44)},
                  halo="below"),
})
# At 32 a block the face is 5 texels by 4: an eye is 2 by 2, a catchlight up and out, the warm iris
# (recolored per dog) low and in, mirrored so the two match.
EYES.update({
    "px": dict(pattern=["HP", "PI"], colors={"P": _P, "H": _H, "I": (120, 70, 44)}, halo="none", iris="I", mirror=True),
    "px_round": dict(pattern=["HP", "IP"], colors={"P": _P, "H": _H, "I": (120, 70, 44)}, halo="none", iris="I", mirror=True),
    "px_soft": dict(pattern=["HP", "PI"], colors={"P": _P, "H": (170, 180, 196), "I": (120, 70, 44)}, halo="none", iris="I",
                    mirror=True),
    "px_in": dict(pattern=["PH", "IP"], colors={"P": _P, "H": _H, "I": (120, 70, 44)}, halo="none", iris="I", mirror=True),
    "px_dark": dict(pattern=["HP", "PP"], colors={"P": (30, 20, 22), "H": _H}, halo="none", iris="", mirror=True),
})
EYE_STYLE = os.environ.get("BLOCKY_EYES", "px_soft")
EYE_Y = 38.5  # the eyes' middle, where the sheet has them


def stamp_eyes_smooth(img, e, face):
    """The eyes: a design from EYES centered at x = +-6.5 on the skull's front, in darker fur.
    halo: "round" darkens a ring all around (lighter above), "below" only the sides and below,
    "mask" a strong two-texel patch all around with lighter fur under it."""
    if e["name"] != "skull" or face != "south":
        return img
    d = EYES[EYE_STYLE]
    for cx in (-6.5, 6.5):
        pattern = [row[::-1] for row in d["pattern"]] if (d.get("mirror") and cx > 0) else d["pattern"]
        ph, pw = len(pattern), len(pattern[0])
        u0, v0 = eye_texel(cx, d)
        m = np.zeros(img.shape[:2], bool)
        for r, row in enumerate(pattern):
            for c, ch in enumerate(row):
                if ch != ".":
                    m[v0 + r, u0 + c] = True
        ring = ndimage.binary_dilation(m) & ~m
        ring2 = ndimage.binary_dilation(m, iterations=2) & ~m & ~ring
        above = np.zeros_like(m)
        above[:v0 + 1] = True
        f = img[..., :3].astype(float)
        halo = d.get("halo", "round")
        if halo == "below":
            f[ring & ~above] *= 0.86
            f[ring2 & ~above] *= 0.95
        elif halo == "round":
            f[ring & ~above] *= 0.8
            f[ring & above] *= 0.9
            f[ring2] *= 0.94
        elif halo == "none":
            pass
        elif halo == "mask":
            f[ring] *= 0.7
            f[ring2 & ~above] *= 0.8
            f[ring2 & above] *= 0.88
            below = np.roll(ring2, 1, axis=0) & ~m & ~ring & ~ring2
            below[:v0 + ph] = False
            f[below] = np.minimum(f[below] * 1.12, 245)
        for r, row in enumerate(pattern):
            for c, ch in enumerate(row):
                if ch != ".":
                    f[v0 + r, u0 + c] = d["colors"][ch]
        img[..., :3] = np.clip(np.round(f), 0, 255).astype(np.uint8)
        mark_iris(img, v0, u0, pattern, iris_keys(d))
    return img


PAD = np.array((96, 56, 60))
GROOVE = np.array((150, 96, 82))  # a warm shadow between toes, not grey
TOE_GROOVES = (2, 5)  # columns between the three 2-texel toes of an 8-wide paw


def stamp_paw(img, e, face):
    """Toes on the paws: grooves on the front running a little way back over the top, a crease
    behind the toes, and pads (three toe beans and a main pad) underneath."""
    if not e["name"].endswith("_paw"):
        return img
    f = img[..., :3].astype(float)
    h, w = f.shape[:2]
    if w < 8 or h < 6 and face == "down":  # at 32 a block a paw is 2 texels: no room for toes
        return img
    if face == "south":
        for u in TOE_GROOVES:
            f[:, u] = f[:, u] * 0.5 + GROOVE * 0.5
        f[0, [0, 1, 3, 4, 6, 7]] = np.minimum(f[0, [0, 1, 3, 4, 6, 7]] * 1.04, 245)  # rounded toe tops
    elif face == "up":  # v runs toward the front
        for u in TOE_GROOVES:
            f[h - 3:, u] = f[h - 3:, u] * 0.6 + GROOVE * 0.4
        f[h - 4, :] *= 0.9
    elif face == "down":  # v runs toward the back
        for u0 in (0, 3, 6):
            f[1, u0:u0 + 2] = PAD
        f[3:6, 2:6] = PAD
        f[3, [2, 5]] = f[3, [2, 5]] * 0.5 + PAD * 0.5
    img[..., :3] = np.clip(np.round(f), 0, 255).astype(np.uint8)
    return img


MOUTH_BASE = {"tongue": (214, 86, 98), "mouth": (84, 32, 42), "palate": (150, 70, 82)}
MOUTH_TONES = {r: np.array([tone(c, k, 0.95) for k in KS]) for r, c in MOUTH_BASE.items()}
LIP = np.array((46, 30, 34))
TOOTH = np.array((240, 234, 220))


def shade(name, k):
    """A mouth color at ramp step(s) k (array or number)."""
    ki = np.round((np.clip(np.round(np.asarray(k) / STEP) * STEP, KS[0], KS[-1]) - KS[0]) / STEP).astype(int)
    return MOUTH_TONES[name][ki]


def paint_mouth(img, e, face):
    """The mouth, designed like the eyes: a tongue with a center groove, a dark mouth inside, a
    pink roof and floor, black lips and a line under the nose, and two small lower fangs."""
    name = e["name"]
    if not (name.startswith("tongue") or name in ("mouth", "muzzle", "jaw")):
        return img
    h, w = img.shape[:2]
    uu, vv = np.meshgrid(np.arange(w), np.arange(h))
    f = img[..., :3].astype(float)
    if name.startswith("tongue"):
        # Darker inside the mouth, lighter toward the tip, a groove down the middle of the top.
        k = np.full((h, w), FACE_LIGHT[face] * 0.6 + {"tongue_base": -0.6, "tongue": 0.0, "tongue_tip": 0.35}[name])
        if face in ("up", "down"):
            along = vv / max(h - 1, 1) if face == "up" else 1 - vv / max(h - 1, 1)
            k += 0.3 * along
            mid = [w // 2] if w % 2 else [w // 2 - 1, w // 2]
            k -= (0.9 if w % 2 else 0.55) * np.isin(uu, mid) * (face == "up")
            k -= 0.35 * ((uu == 0) | (uu == w - 1))
        f = shade("tongue", k)
    elif name == "mouth":
        f = shade("mouth", np.full((h, w), -0.5 + 0.3 * FACE_LIGHT[face]))
    elif name == "muzzle" and face == "down":  # the roof of the mouth, front row at v = 0
        f = shade("palate", -0.6 - 0.5 * (vv % 2 == 1) - 0.08 * vv)
        f[0] = LIP
    elif name == "muzzle" and face == "south" and h >= 4:  # upper lip, and a line from the nose down to it
        f[h - 1] = LIP
        mid = [w // 2 - 1, w // 2]
        f[3:h - 1, mid] = f[3:h - 1, mid] * 0.45 + LIP * 0.55
    elif name == "jaw" and face == "up" and w >= 5:  # the floor of the mouth, front row at v = h - 1
        f = shade("palate", -0.9 - 0.06 * (h - 1 - vv))
        f[h - 1] = LIP
        f[h - 2:, [1, w - 2]] = TOOTH
        f[h - 2:, 2:w - 2] = shade("tongue", -0.5)  # under the tongue, where it goes over the lip
    elif name == "jaw" and face == "south" and w >= 5:  # lower lip and the fangs' tips
        f[0] = LIP
        f[0, [1, w - 2]] = TOOTH
    img[..., :3] = np.clip(np.round(f), 0, 255).astype(np.uint8)
    return img


def stamp_eyes(img, e, face):
    """The eyes on the skull's front face, where the sheet has them (x = +-6.5, top at y = 40)."""
    if e["name"] != "skull" or face != "south":
        return img
    for cx in (-6.5, 6.5):
        for r, row in enumerate(EYE):
            for c, ch in enumerate(row):
                if ch == ".":
                    continue
                u = int(cx - 2.5 + c - e["lo"][0])
                v = int(e["hi"][1] - 40 + r)
                img[v, u, :3] = EYE_COLORS[ch]
        # A lid shadow above.
        v = int(e["hi"][1] - 41)
        for c in range(5):
            u = int(cx - 2.5 + c - e["lo"][0])
            img[v, u, :3] = (img[v, u, :3] * 0.78).astype(np.uint8)
    return img


def bib_outline(p, inside):
    """The bib's fur hanging at the front: locks three texels wide whose bottoms come together in a
    V, the point in the middle hanging between the front legs (y = 5), the sides rising steeply
    so the legs show below; above, the sheet's outline."""
    x, y = p[..., 0], p[..., 1]
    ax = np.abs(x)
    lock = np.abs((np.floor(ax) + 1) % 3 - 1)  # 0 at each lock's point, 1 at its sides
    bottom = 5 + 1.5 * ax + 2.2 * lock
    return ndimage.binary_fill_holes((ax <= 11.5) & (y >= bottom) & (inside | (y < 22)))


# The coat's roles (REGIONS order): brown, orange and white are all coat (white spotting is a
# gene: the painted white becomes where it shows first); pink, eyes, catchlights, nose are fixed.
ROLES = [("coat", 0, 0), ("coat", 0, 0), ("coat", 0, 0), "fixed", "fixed", "fixed", "fixed"]


def _ramp(v, a, b):
    return np.clip((v - a) / (b - a), 0, 1)


def layer_props(e, face, p, white_w):
    """Where genes show on a chihuahua: white spotting first where this parti dog is white (paws,
    chest, muzzle, plume), tan points on the brows, cheeks, muzzle, chest, legs and under the
    tail, a dark mask on the muzzle, colorpoint-like extremities, a lighter underside."""
    n, x, y = e["name"], p[..., 0], p[..., 1]
    lo, hi = e["lo"][1], e["hi"][1]
    low = 1 - _ramp(y, lo, max(hi, lo + 1e-3))
    z = np.zeros(y.shape)
    if n.endswith("_paw"):
        lowness = np.ones(y.shape)
    elif n.startswith("leg_"):
        lowness = 0.6 + 0.35 * low
    elif n in ("chest", "bib", "bib_locks", "muzzle", "jaw", "britches"):
        lowness = 0.85 + 0.0 * low
    elif n in ("hips",) or n.startswith("thigh_"):
        lowness = 0.6 * low + 0.25 * (face == "down")
    elif n.startswith(("tail_plume", "tail_curl")):
        lowness = np.full(y.shape, 0.6)
    elif n == "skull" and face == "south":
        lowness = 0.75 * _ramp(4 - np.abs(x), 0, 4)  # a blaze up the middle of the face
    else:
        lowness = 0.15 * low
    white = white_w * (0.55 + 0.45 * lowness) + (1 - white_w) * 0.35 * lowness
    points = z.copy()
    if n == "skull" and face == "south":
        brows = (np.abs(np.abs(x) - 5.5) < 1.6) & (y > 41) & (y < 43.5)
        cheeks = (y < 33.5) & (np.abs(x) > 4)
        points = np.where(brows | cheeks, 1.0, 0.0)
    elif n in ("muzzle", "jaw"):
        points[:] = 1.0 if face != "up" else 0.4
    elif n in ("chest", "bib", "bib_locks"):
        points[:] = 0.8 if face in ("south", "down") else 0.3
    elif n.startswith("leg_"):
        points = np.where(y < 9, 1.0, 0.35)
    elif n.endswith("_paw") or n == "britches":
        points[:] = 1.0
    elif n == "tail_base" and face == "down":
        points[:] = 1.0
    mask = np.full(y.shape, 1.0 if n in ("muzzle", "jaw", "nose") else 0.0)
    if n == "skull" and face == "south":
        mask = _ramp(5 - np.abs(x), 0, 3) * _ramp(36 - y, 0, 4)
    ext = np.full(y.shape, 1.0 if n.startswith(("ear_", "fringe_ear", "muzzle", "nose", "tail_plume")) or n.endswith("_paw") else 0.0)
    under = np.zeros(y.shape)
    if n in ("chest", "hips", "neck") or n.startswith(("thigh_", "leg_")):
        under = 0.7 * low + 0.3 * (face == "down")
    elif n in ("bib", "bib_locks", "britches", "jaw"):
        under[:] = 0.8
    head = n in ("skull", "muzzle", "jaw", "nose") or n.startswith(("ear_", "fringe_ear"))
    pattern = np.full(y.shape, 0.5 if head else 0.6 if n.startswith("leg_") or n.endswith("_paw") else 1.0)
    return {"white": white, "points": points, "mask": mask, "extremity": ext, "under": under, "pattern": pattern}


def paint_face(e, face):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face, TX)
    if w == 0 or h == 0:
        return None
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
    size = np.array(e["hi"]) - np.array(e["lo"])
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
    p = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    nn = R @ n
    lid = e["name"].startswith("eyelid")
    if lid:  # painted as it looks closed: on the face, hinge at the top
        p = np.stack([p[..., 0], e["hi"][1] - vv * sv, np.full(vv.shape, LID_Z)], -1)
        nn, du, dv, face = np.array([0, 0, 1.0]), np.array([1.0, 0, 0]), np.array([0, -1.0, 0]), "south"
    if e.get("src"):
        o, k = np.array(e["src"][0]), e["src"][1]
        p = o + (p - o) / k
    if e["view"] == "front":
        nn = np.array([0, 0, 1.0])
    elif e["view"] == "back":
        nn = np.array([0, 0, -1.0])
    rgb, inside = project(p, nn, max(1.0, 1 / TX))
    if e["name"] == "bib_locks":
        inside = bib_outline(p, inside)
    idx = vd.snap(rgb, palette_for(e))
    if TX >= 0.75:  # at 32 a block each texel is already its footprint's median: a 3x3 vote would erase the blaze
        idx = clean(idx)
    seed = sum(map(ord, e["name"] + face)) % 97
    if STYLE == "projected":
        col = projected_colors(idx, face, vv, h, p)
    elif STYLE == "painted":
        col = painted_colors(idx, rgb, face, uu, vv, w, h, p, seed=seed, quad=bool(e["quad"]))
    else:
        col = smooth_colors(idx, e, face, uu, vv, w, h, p, nn, R @ du, R @ dv, seed, quad=bool(e["quad"]))
    alpha = np.where(inside, 255, 0) if e["cut"] else np.full((h, w), 255)
    if lid:  # a closed eye: a dark lash line along the bottom, a soft crease above it
        col[-1] = EYES[EYE_STYLE]["colors"].get("O", (40, 26, 26))
        col[-2] = col[-2] * 0.85
        alpha = np.full((h, w), 255)
    if e["name"] == "bib_locks":  # the locks' tips a little darker, so the V reads over the white legs
        tip = inside & ~np.vstack([inside[1:], np.zeros((1, w), bool)]) & (p[..., 1] < 20)
        col = np.where(tip[..., None], col * 0.88, col)
        above = np.vstack([tip[1:], np.zeros((1, w), bool)]) & inside & ~tip
        col = np.where(above[..., None], col * 0.95, col)
    img = np.dstack([np.round(col), alpha]).astype(np.uint8)
    if STYLE == "projected":
        return img
    if STYLE == "painted":
        return stamp_eyes(img, e, face)
    smooth = img[..., :3].copy()
    img = paint_mouth(stamp_paw(stamp_eyes_smooth(img, e, face), e, face), e, face)
    cap = LAYERS["cap"]
    if LAYERS["rec"] is not None and cap is not None and cap["level"].shape == img.shape[:2]:
        white_w = cap["weights"][REGIONS.index("white")]
        LAYERS["rec"].face(LAYERS["key"], cap["weights"], cap["level"], ROLES, smooth, img, p,
                           layer_props(e, face, p, white_w))
    return img


def build():
    E = elements()
    BOXES[:] = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E
                if not e["quad"] and not e["rot"] and not e["name"].startswith("tongue")]  # it moves: no baked shadow
    tiles = []
    LAYERS["rec"] = Recorder(layer_bounds(E, 3.0)) if STYLE == "smooth" else None
    for i, e in enumerate(E):
        faces = [e["quad"]] if e["quad"] else list(FACES)
        for f in faces:
            LAYERS["key"], LAYERS["cap"] = (i, f), None
            img = paint_face(e, f)
            if img is not None:
                tiles.append((i, f, img))
    # Shelf packing, tallest first, each tile with a one-texel gutter copied from its edge so
    # filtering and mipmaps never pull a neighbor's colors in (the seams that flicker).
    tiles.sort(key=lambda t: (-t[2].shape[0], -t[2].shape[1]))
    sheet = np.zeros((SHEET, SHEET, 4), np.uint8)
    x = y = row = 0
    uv = {}
    for i, f, img in tiles:
        h, w = img.shape[:2]
        if x + w + 2 > SHEET:
            x, y, row = 0, y + row, 0
        assert y + h + 2 <= SHEET, "texture full"
        sheet[y:y + h + 2, x:x + w + 2] = np.pad(img, ((1, 1), (1, 1), (0, 0)), mode="edge")
        uv[(i, f)] = [x + 1, y + 1, x + 1 + w, y + 1 + h]
        x += w + 2
        row = max(row, h + 2)
    used = y + row
    tex_h = 32 * int(math.ceil(used / 32))  # sizes are multiples of 32
    sheet = sheet[:tex_h]
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    if LAYERS["rec"] is not None:  # the same packing as above (tallest first, stable)
        LAYERS["rec"].save(OUT, [(i, f) for i, f, _ in tiles], skill_pack, SHEET)
        LAYERS["rec"] = None
    return E, sheet


LID_Z = 20.4  # the eyelids' hinge line, just in front of the face (z = 20)
SKULL_TOP, SKULL_X0 = 46, -11


def eyelid_boxes():
    """(bone name, x0, x1, top, bottom) of each eyelid: exactly over the stamped eye. BLOCKY_LID
    sizes them for another eye type, so textures painted with other eyes keep this layout."""
    d = EYES[os.environ.get("BLOCKY_LID", EYE_STYLE)]
    ph, pw = len(d["pattern"]), len(d["pattern"][0])
    _, _, su, sv = skull_grid()
    out = []
    for name, cx in (("eyelid_R", -6.5), ("eyelid_L", 6.5)):
        u0, v0 = eye_texel(cx, d)
        x0, top = SKULL_X0 + u0 * su, SKULL_TOP - v0 * sv
        out.append((name, float(x0), float(x0 + pw * su), float(top), float(top - ph * sv)))
    return out


def skull_grid():
    """The skull front's texels: (width, height) and the units a texel spans (u, v)."""
    sx, sy = 22, 18  # the skull's front: x -11..11, y 28..46
    w, h = max(1, round(TX * sx)), max(1, round(TX * sy))
    return w, h, sx / w, sy / h


def eye_texel(cx, d):
    """The top-left texel of the eye centered at x = cx on the skull's front (as `paint.eye_corner`
    places it: two eyes land mirror-symmetric)."""
    w, h, su, sv = skull_grid()
    return eye_corner((cx - SKULL_X0) / su, (SKULL_TOP - d.get("y", EYE_Y)) / sv, d["pattern"], w)


def skeleton(L=L):
    """voxdog's bones plus the jaw (hinged at the back of the mouth), the tongue (pivots at the
    lip, then its tip) and the eyelids (hinged at the top of each eye)."""
    lids = [(name, "head", ((x0 + x1) / 2, top, LID_Z)) for name, x0, x1, top, _ in eyelid_boxes()]
    return vd.skeleton(L) + [("jaw", "head", (0.0, 29.0, 17.75)), ("tongue", "jaw", (0.0, 30.2, 22.2)),
                             ("tongue_tip", "tongue", (0.0, 30.4, 24.6))] + lids


def mouth(name, t, dur):
    """Jaw and tongue per animation: panting with the tongue out, slower and nearly shut lying
    down. +X opens the jaw and drops the tongue's tip."""
    p = t / dur
    if name == "lick":
        k = lick_keys(p)
        return {"jaw": (k["jaw"], 0, 0), "tongue": (k["tongue"], 0, 0), "tongue_tip": (k["tip_x"], k["tip_y"], 0)}
    if name == "bite":  # the tongue folded away under the muzzle
        return {"jaw": (bite_keys(p)["jaw"], 0, 0), "tongue": (0, 0, 0), "tongue_tip": (180, 0, 0)}
    if name in CLIP_MOUTHS:  # the behaviour clips' own (`behaviour_mouth`)
        return behaviour_mouth(name, p)
    pant = {"idle": (20, 4, 8, 55, 6, 25), "sit": (20, 4, 8, 55, 6, 25), "walk": (18, 3, 2, 55, 8, 25),
            "run": (26, 4, 1, 40, 12, 15), "lie": (8, 2, 2, 70, 4, 15)}[name]
    jaw, jaw_amp, n, tongue, tongue_amp, tip = pant
    a = TAU * p * n
    return {"jaw": (jaw + jaw_amp * math.sin(a), 0, 0), "tongue": (tongue + tongue_amp * math.sin(a - 0.6), 0, 0),
            "tongue_tip": (tip + 0.8 * tongue_amp * math.sin(a - 1.2), 0, 0)}


TAU = 2 * math.pi
BLINKS = {"idle": [1.0], "sit": [1.6], "lie": [0.8, 2.2], "lick": []}  # when each animation blinks
BLINK = 0.09  # half a blink, seconds


def lick_keys(p):
    """The lick, as (phase, value) keys: the jaw opens, the tongue comes out level, its tip curls
    up over the nose and sweeps across it, then folds back in under the muzzle."""
    K = {"jaw": [(0, 0), (.1, 0), (.22, 14), (.75, 14), (.88, 0), (1, 0)],
         "tongue": [(0, 0), (.1, 0), (.22, -14), (.3, -22), (.6, -22), (.72, -14), (.88, 0), (1, 0)],
         "tip_x": [(0, 180), (.1, 180), (.24, 0), (.34, -75), (.62, -75), (.72, 0), (.86, 180), (1, 180)],
         "tip_y": [(0, 0), (.36, 0), (.44, -22), (.54, 22), (.6, 0), (1, 0)],
         "lids": [(0, 0), (.3, 0), (.36, -90), (.56, -90), (.62, 0), (1, 0)],
         "head": [(0, 0), (.15, -8), (.75, -8), (.9, 0), (1, 0)]}
    return {k: float(np.interp(p, [a for a, _ in v], [b for _, b in v])) for k, v in K.items()}


def pose_lick(t, L):
    """Lick: standing, the head up a little, licking its nose (1.6 s loop)."""
    pose = vd.pose_stand(t * 1.5, L)
    k = lick_keys(t / 1.6)
    hx, hy, hz = pose["head"]
    pose["head"] = (hx + k["head"], hy * 0.3, hz * 0.3)
    return pose


def bite_keys(p):
    """The bite, as (phase, value) keys: drawn back with the ears flat, a quick lunge, the jaw
    opening on the way and snapping shut."""
    K = {"lunge": [(0, 0), (.25, -1), (.45, 1), (.6, 1), (1, 0)],
         "jaw": [(0, 0), (.2, 22), (.4, 30), (.48, 0), (1, 0)],
         "ears": [(0, 0), (.15, 1), (.75, 1), (1, 0)]}
    return {k: float(np.interp(p, [a for a, _ in v], [b for _, b in v])) for k, v in K.items()}


def pose_bite(t, L):
    """Bite: standing, a quick lunge and snap (0.5 s)."""
    pose = vd.pose_stand(0, L)
    k = bite_keys(t / 0.5)
    d, e = k["lunge"], k["ears"]
    pose["spine_1"] = (4 * d, 0, 0)
    pose["neck"] = (10 * d, 0, 0)
    pose["head"] = (4 * d, 0, 0)
    pose["ear_L"] = (-24 * e, 0, -10 * e)  # flat back
    pose["ear_R"] = (-24 * e, 0, 10 * e)
    return pose


# ------------------------------------------------------------------ behaviour clips (`anim.rs`)
# The dog's and the chihuahua's (one model): each a whole-body pose over what plays below on the
# bones it moves (`to_bedrock.as_action`); held states loop, the rest play once. The fitted
# numbers (`low_*`, `death_drop`) come from `fit_clips` once the model is built.

FIT = {"low_neck": 50, "low_head": 20, "low_crouch": 0.0, "low_drop": 0.0, "low_chest": 0, "death_drop": 10.0}


def still(t, L):
    """Standing still, breathing, the head steady: what a clip builds on."""
    q = vd.pose_stand(0, L)
    br = math.sin(TAU * t / 2.0)
    q.update({"root": (0, 0.12 * br, 0), "spine_1": (1.0 * br, 0, 0), "neck": (-1.5 * br, 0, 0), "head": (0, 0, 0),
              "ear_L": (0, 0, 0), "ear_R": (0, 0, 0)})
    return q


def crouch(k):
    q = {}
    for leg in ("FL", "FR"):
        q.update({f"leg_{leg}_upper": (22 * k, 0, 0), f"leg_{leg}_lower": (-30 * k, 0, 0), f"leg_{leg}_paw": (8 * k, 0, 0)})
    for leg in ("BL", "BR"):
        q.update({f"leg_{leg}_upper": (-18 * k, 0, 0), f"leg_{leg}_lower": (28 * k, 0, 0), f"leg_{leg}_paw": (-10 * k, 0, 0)})
    return q


def head_down(d=1.0, P=None):
    """The nose to the ground by `d` (fitted): legs bent, body dropped, neck and head lowered."""
    P = P or FIT
    q = crouch(P["low_crouch"] * d)
    q.update({"root": (0, -P["low_drop"] * d, 0), "spine_1": (P["low_chest"] * d, 0, 0), "neck": (P["low_neck"] * d, 0, 0),
              "head": (P["low_head"] * d, 0, 0)})
    # The front legs kept upright under a chest pitched down.
    for leg in ("FL", "FR"):
        u = q.get(f"leg_{leg}_upper", (0, 0, 0))[0]
        q[f"leg_{leg}_upper"] = (u - P["low_chest"] * d, 0, 0)
    return q


def wag(q, L, amp, rate, base=(0, 0, 0)):
    for k in range(3):
        q[f"tail_{k}"] = tail_wag(L, k, amp * math.sin(rate - 0.8 * k), base if k == 0 else (0, 0, 0))
    return q


def tail_wag(L, k, a, base=(0, 0, 0)):
    return vd.tail_wag(L, k, a, base)


def pose_stare(t, L):
    q = still(t, L)
    q.update({"neck": (-8, 0, 0), "head": (-4, 0, 0), "ear_L": (-16, 0, 0), "ear_R": (-16, 0, 0)})
    return wag(q, L, 0, 0)


def pose_sniff(t, L):
    p = t / 2.0
    d = float(np.interp(p, [0, .25, .75, 1], [0, 1, 1, 0]))
    q = still(t, L)
    q.update(head_down(d))
    q["head"] = (q["head"][0], 10 * d * math.sin(TAU * p * 2), 0)
    return wag(q, L, 10, TAU * p * 3)


def pose_drink(t, L):
    q = still(t, L)
    q.update(head_down(1.0))
    return wag(q, L, 4, TAU * t)


def pose_look(t, L):
    p = t / 2.6
    yaw = float(np.interp(p, [0, .15, .4, .55, .8, 1], [0, 45, 45, -40, -40, 0]))
    q = still(t, L)
    q.update({"neck": (-4, 0.8 * yaw, 0), "head": (0, 0.15 * yaw, 0),
              "ear_L": (-10, 0, 0), "ear_R": (-10, 0, 0)})
    return wag(q, L, 6, TAU * p * 2)


def pose_scratch(t, L):
    """Nibbling an itch on its flank (1.8 s)."""
    p = t / 1.8
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    q = still(t, L)
    q.update({"neck": (16 * d, 64 * d, 0), "head": (6 * d, 4 * d, 0), "ear_L": (10 * d, 0, 0), "ear_R": (10 * d, 0, 0)})
    return wag(q, L, 4, TAU * p)


def pose_stretch(t, L):
    """The bow stretch, then a back leg out behind (2.2 s)."""
    p = t / 2.2
    bow = float(np.interp(p, [0, .15, .45, .55, 1], [0, 1, 1, 0, 0]))
    back = float(np.interp(p, [0, .55, .7, .9, 1], [0, 0, 1, 1, 0]))
    q = still(t, L)
    q.update({"root": (0, -1.2 * bow, 0), "spine_0": (-10 * bow + 4 * back, 0, 0), "spine_1": (14 * bow, 0, 0), "neck": (-10 * bow, 0, 0),
              "head": (-8 * bow, 0, 0)})
    for leg in ("FL", "FR"):
        q.update({f"leg_{leg}_upper": (-50 * bow, 0, 0), f"leg_{leg}_lower": (8 * bow, 0, 0), f"leg_{leg}_paw": (28 * bow, 0, 0)})
    for leg in ("BL", "BR"):
        q.update({f"leg_{leg}_upper": (10 * bow + 28 * back, 0, 0), f"leg_{leg}_lower": (-8 * back, 0, 0),
                  f"leg_{leg}_paw": (-10 * bow - 20 * back, 0, 0)})
    return wag(q, L, 10 * bow, TAU * p * 3)


def pose_sleep(t, L):
    """Asleep, lying: the head on its paws, eyes shut, mouth shut, slow breath (5 s, held)."""
    q = vd.pose_lie(t * 3.0 / 5.0, L)
    br = math.sin(TAU * t / 5.0 * 2)
    q["root"] = (0, -L["lie_drop"] + 0.14 * br, 0)
    q["head"] = (L["lie_head"] + 4, 0, 0)
    return q


def pose_stalk(t, L):
    """Creeping low, head level and forward, tail low and still (1.1 s a cycle, in place of the walk)."""
    p = t / 1.1
    q = vd.legs_gait(p, (0.0, 0.5, 0.75, 0.25), 14, 26, L)
    for leg in ("FL", "FR", "BL", "BR"):
        f = leg[0] == "F"
        q[f"leg_{leg}_upper"] = (q[f"leg_{leg}_upper"][0] + (22 if f else -18), 0, 0)
        q[f"leg_{leg}_lower"] = (q[f"leg_{leg}_lower"][0] + (-30 if f else 28), 0, 0)
        q[f"leg_{leg}_paw"] = (q[f"leg_{leg}_paw"][0] + (8 if f else -10), 0, 0)
    q.update({"root": (0, -1.5, 0), "neck": (14, 0, 0), "head": (-12, 0, 0), "ear_L": (-20, 0, 0), "ear_R": (-20, 0, 0)})
    return wag(q, L, 0, 0, (30, 0, 0))


def pose_alarm(t, L):
    """Barking: head up, ears high, two sharp barks (1.0 s)."""
    p = t / 1.0
    up = float(np.interp(p, [0, .1, .8, 1], [0, 1, 1, 0]))
    q = still(t, L)
    q.update({"spine_1": (-3 * up, 0, 0), "neck": (-14 * up, 0, 0), "head": (-6 * up, 0, 0), "ear_L": (-22 * up, 0, 0),
              "ear_R": (-22 * up, 0, 0)})
    return wag(q, L, 4, TAU * p * 2, (-14 * up, 0, 0))


def pose_howl(t, L):
    p = t / 3.0
    h = float(np.interp(p, [0, .12, .85, 1], [0, 1, 1, 0]))
    q = still(t, L)
    q.update({"spine_1": (-4 * h, 0, 0), "neck": (-30 * h, 0, 0), "head": (-16 * h, 0, 0), "ear_L": (14 * h, 0, 0), "ear_R": (14 * h, 0, 0)})
    return wag(q, L, 2, TAU * p)


def pose_play(t, L):
    """The play bow, a bounce and a spin of the head, the tail wagging hard (1.4 s)."""
    p = t / 1.4
    bow = float(np.interp(p, [0, .12, .4, .5, 1], [0, 1, 1, 0, 0]))
    hop = max(0.0, math.sin(TAU * (p - 0.5) * 2)) if p > 0.5 else 0.0
    q = still(t, L)
    q.update({"root": (0, -1.2 * bow + 2.0 * hop, 0), "spine_0": (-10 * bow, 0, 0), "spine_1": (14 * bow, 0, 0),
              "neck": (-10 * bow, 16 * hop, 0), "head": (-8 * bow, 0, 12 * hop)})
    for leg in ("FL", "FR"):
        q.update({f"leg_{leg}_upper": (-50 * bow - 20 * hop, 0, 0), f"leg_{leg}_lower": (8 * bow + 26 * hop, 0, 0),
                  f"leg_{leg}_paw": (28 * bow, 0, 0)})
    return wag(q, L, 26, TAU * p * 6)


def pose_groom(t, L):
    """Licking a front paw (2.4 s)."""
    p = t / 2.4
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    q = still(t, L)
    q.update({"neck": (34 * d, 8 * d, 0), "head": (16 * d, 4 * d, 0), "leg_FL_upper": (-24 * d, 0, 0), "leg_FL_lower": (30 * d, 0, 0)})
    return wag(q, L, 4, TAU * p)


def pose_nuzzle(t, L):
    p = t / 2.0
    d = float(np.interp(p, [0, .2, .85, 1], [0, 1, 1, 0]))
    push = max(0.0, math.sin(TAU * p * 2))
    q = still(t, L)
    q.update(head_down(0.55 * d))
    q["head"] = (q["head"][0] - 6 * push, 0, 0)
    q.update({"ear_L": (10 * d, 0, -8 * d), "ear_R": (10 * d, 0, 8 * d)})
    return wag(q, L, 12, TAU * p * 4)


def pose_flinch(t, L):
    p = t / 0.4
    d = float(np.interp(p, [0, .15, .45, 1], [0, 1, 0.8, 0]))
    q = still(t, L)
    q.update({"root": (0, -0.6 * d, 0), "spine_1": (4 * d, 0, 2 * d), "neck": (12 * d, 0, 0), "head": (10 * d, 0, 0),
              "ear_L": (-24 * d, 0, -10 * d), "ear_R": (-24 * d, 0, 10 * d)})
    return wag(q, L, 0, 0, (30 * d, 0, 0))


def pose_death(t, L):
    p = t / 1.2
    fall = float(np.interp(p, [0, .35, .7, 1], [0, 0.35, 1, 1]))
    q = still(0, L)
    q.update({"root": (0, -FIT["death_drop"] * fall, 0), "spine_0": (0, 0, 84 * fall), "neck": (14 * fall, 0, -10 * fall),
              "head": (8 * fall, 0, 0), "ear_L": (14 * fall, 0, 0), "ear_R": (14 * fall, 0, 0)})
    for leg in ("FL", "FR", "BL", "BR"):
        q[f"leg_{leg}_upper"] = ((10 if leg[0] == "F" else -10) * fall, 0, 0)
    return wag(q, L, 0, 0)


CLIP_LENGTHS = {"stare": 2.0, "sniff": 2.0, "drink": 1.0, "look": 2.6, "scratch": 1.8, "stretch": 2.2, "sleep": 5.0, "stalk": 1.1,
                "alarm": 1.0, "howl": 3.0, "play": 1.4, "groom": 2.4, "nuzzle": 2.0, "flinch": 0.4, "death": 1.2}
CLIP_MOUTHS = set(CLIP_LENGTHS)
LOOPS = {"stare": True, "drink": True, "sleep": True, "stalk": True, "death": "hold_on_last_frame"}
BLENDS = {"flinch": (0.05, 0.2), "death": (0.1, 0.0), "stalk": (0.3, 0.3), "sleep": (0.6, 0.6)}


def behaviour_mouth(name, p):
    """The jaw and tongue in a behaviour clip: shut and tucked mostly, lapping when it drinks,
    licking when it grooms or nuzzles, barking, howling, panting at play."""
    shut = {"jaw": (2, 0, 0), "tongue": (0, 0, 0), "tongue_tip": (180, 0, 0)}
    a = TAU * p
    if name == "drink":
        return {"jaw": (14 + 4 * math.sin(a * 2), 0, 0), "tongue": (60 + 20 * math.sin(a * 2), 0, 0), "tongue_tip": (40 * math.sin(a * 2), 0, 0)}
    if name in ("groom", "nuzzle"):
        lick = max(0.0, math.sin(a * 5))
        return {"jaw": (10 * lick, 0, 0), "tongue": (-10 * lick, 0, 0), "tongue_tip": (180 - 150 * lick, 0, 0)}
    if name == "alarm":
        bark = float(np.interp(p, [0, .2, .28, .36, .5, .58, .66, 1], [0, 0, 1, 0, 0, 1, 0, 0]))
        return {"jaw": (26 * bark, 0, 0), "tongue": (0, 0, 0), "tongue_tip": (180, 0, 0)}
    if name == "howl":
        h = float(np.interp(p, [0, .15, .85, 1], [0, 1, 1, 0]))
        return {"jaw": (20 * h, 0, 0), "tongue": (0, 0, 0), "tongue_tip": (180, 0, 0)}
    if name in ("play", "stretch"):
        return {"jaw": (20 + 4 * math.sin(a * 6), 0, 0), "tongue": (50, 0, 0), "tongue_tip": (25, 0, 0)}
    if name == "death":
        return {"jaw": (8 * min(1.0, p * 2), 0, 0), "tongue": (30 * min(1.0, p * 2), 0, 0), "tongue_tip": (60, 0, 0)}
    return shut


ANIMS = dict(vd.ANIMS, lick=(pose_lick, 1.6), bite=(pose_bite, 0.5),
             **{n: (globals()[f"pose_{n}"], d) for n, d in CLIP_LENGTHS.items()})


def eyelids(name, t, dur):
    """The lids' angle: 0 open, -90 closed. A blink closes and opens them in about 0.18 s."""
    if name == "lick":
        a = lick_keys(t / dur)["lids"]
    elif name == "sleep":
        a = -90
    elif name == "death":
        a = -90 * min(1.0, t / dur * 1.5)
    else:
        a = -90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (a, 0, 0), "eyelid_R": (a, 0, 0)}


def frame_times(name, dur):
    """Keyframe times: even steps, plus the edges and middle of every blink so they stay crisp;
    the lick, all quick moves, gets fine steps."""
    n = 32 if name == "lick" else 24 if name in ("bite", *CLIP_LENGTHS) else (12 if dur < 1 else 16)
    ts = {round(dur * k / n, 4) for k in range(n + 1)}
    for b in BLINKS.get(name, []):
        ts |= {round(b + d, 4) for d in (-BLINK, -BLINK / 2, 0, BLINK / 2, BLINK)}
    return sorted(ts)


def animations():
    out = {}
    names = [n for n, _, _ in skeleton(L)]
    for name, (fn, dur) in ANIMS.items():
        frames = []
        # The standing clips loop; an action plays once unless `LOOPS` holds it (a clip with
        # "loop": true never ends, whatever `action_play` asks).
        loops = LOOPS.get(name, name in vd.ANIMS)
        for t in frame_times(name, dur):
            # A loop's last frame is its first; a clip played once ends where it ends.
            tt = t % dur if t < dur else (0.0 if loops is True else dur)
            pose = fn(tt, L)
            pose.update(mouth(name, tt, dur))
            pose.update(eyelids(name, tt, dur))
            frames.append({"t": round(t, 4), "rot": {b: list(pose.get(b, (0, 0, 0))) for b in names if b != "root"},
                           "root": list(pose.get("root", (0, 0, 0)))})
        out[name] = {"length": dur, "frames": frames}
        if name not in vd.ANIMS:
            out[name].update(loop=loops, blend=BLENDS.get(name, (0.25, 0.3)))
    return out


def fit_clips(spec):
    """The behaviour clips' fitted numbers (`FIT`): the head brought down until the nose is half
    a unit over the ground, the legs bent as little as it takes; how far it drops dying on its side."""
    from ground import lowest
    from rig import fit
    paws = {"paws": lambda n: "paw" in n, "head": lambda n: n.startswith(("muzzle", "nose", "jaw", "head", "skull", "snout"))}
    def low(P):
        g = lowest(spec, dict(vd.pose_stand(0, L), **head_down(1.0, dict(FIT, **P, low_drop=0))), paws)
        return g["head"] - g["paws"], g["paws"]
    best = fit({"low_neck": list(range(20, 101, 10)), "low_head": list(range(0, 61, 10)), "low_crouch": [0, 1, 2, 3],
                "low_chest": [0, 10, 20, 30]},
               lambda P: (low(P)[0] - 0.5) ** 2 + 0.0002 * (P["low_neck"] ** 2 + P["low_head"] ** 2) + 0.5 * P["low_crouch"] ** 2
               + 0.002 * P["low_chest"] ** 2)[1]
    nose, drop = low(best)
    FIT.update(best, low_drop=round(float(drop), 2))
    g = lowest(spec, pose_death(1.2, dict(L)) | {"root": (0, 0, 0)}, {"all": lambda n: True})
    FIT["death_drop"] = round(float(g["all"]), 2)
    print("clips: head down, nose over the ground", round(float(nose), 2), FIT)


def main():
    global STYLE, OUT
    if len(sys.argv) > 1 and sys.argv[1] == "projected":
        STYLE, OUT = "projected", OUT + "_v1"
    elif len(sys.argv) > 1 and sys.argv[1] == "painted":
        STYLE, OUT = "painted", OUT + "_v2"
    os.makedirs(OUT, exist_ok=True)
    E, sheet = build()
    Image.fromarray(sheet, "RGBA").save(os.path.join(OUT, "texture.png"))
    skel = [{"name": n, "parent": p, "pivot": list(map(float, v))} for n, p, v in skeleton(L)]
    spec = {"name": "voxel_dog_blocky", "texture": [SHEET, sheet.shape[0]], "bones": skel, "elements": E}
    fit_clips(spec)
    spec["animations"] = animations()
    with open(os.path.join(OUT, "spec.json"), "w") as f:
        json.dump(spec, f)
    with open(os.path.join(OUT, "landmarks.json"), "w") as f:
        json.dump(L, f, indent=1)
    print(len(E), "elements;", len(skel), "bones; texture", sheet.shape[1], "x", sheet.shape[0])


if __name__ == "__main__":
    main()
