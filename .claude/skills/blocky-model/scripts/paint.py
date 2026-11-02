#!/usr/bin/env python3
"""Painting helpers for block-game textures (Minecraft / Hytale / Vintage Story style).
Import from a model's build script:

    sys.path.insert(0, os.path.expanduser("~/.claude/skills/blocky-model/scripts"))
    from paint import ramp, paint_face, stamp, pack

Every face is painted as: a color-area map (which material each texel is: coat, patch, belly,
pink...) -> a light-level map (top lit, edges, ground contact, rims, fur strokes) -> a color from
that material's hue-shifted ramp at that level. Nothing is noisy; every texel is a deliberate
shade.
"""

import colorsys
import math

import numpy as np
from scipy import ndimage


def ramp(rgb, steps=2):
    """2*steps+1 shades for one base color, darkest first. Shadows shift toward purple and gain
    saturation; lights shift toward yellow and lose saturation; never pure black or white."""
    h, s, v = colorsys.rgb_to_hsv(*(np.array(rgb, float) / 255))

    def toward(h, target, amt):
        d = (target - h + 0.5) % 1.0 - 0.5
        return (h + math.copysign(min(abs(d), amt), d)) % 1.0

    out = []
    for k in range(-steps, steps + 1):
        if k < 0:
            hh, ss, vv = toward(h, 0.74, 0.03 * -k), min(1, s + 0.07 * -k), v * (1 - 0.16 * -k)
        else:
            hh, ss, vv = toward(h, 0.12, 0.015 * k), s * (1 - 0.14 * k), min(0.96, v * (1 + 0.08 * k))
        out.append(np.array(colorsys.hsv_to_rgb(hh, ss, max(vv, 0.08))) * 255)
    return np.array(out)


# How bright each face direction is, in ramp steps (light from above, a bit from the front).
FACE_LIGHT = {"up": 0.9, "south": 0.2, "east": 0.2, "west": -0.3, "north": -0.5, "down": -1.6}


def paint_face(areas, ramps, face, y_world=None, quad=False, seed=0, fur=(), rim=None):
    """Paint one face. `areas`: (h, w) ints into `ramps` (list of ramp arrays, one per material).
    `y_world`: (h, w) height of each texel above the ground (for contact shadow), or None.
    `fur`: material ids that get fur strokes. `rim`: (dark_id, light_id): the dark material is
    shaded where it meets the light one (and the light one a touch). Returns (h, w, 3)."""
    h, w = areas.shape
    steps = (len(ramps[0]) - 1) // 2
    t = (np.arange(h)[:, None] + 0.5) / h * np.ones((1, w))
    L = np.full((h, w), FACE_LIGHT[face])
    if face not in ("up", "down"):
        L += 0.5 - 1.1 * t
        if not quad:
            L[0, :] += 0.6  # top edge catches the light
            L[-1, :] -= 0.3
            if w >= 6:
                L[:, [0, -1]] -= 0.35  # sides turn away
    elif w >= 4 and h >= 4:
        L[[0, -1], :] -= 0.3
        L[:, [0, -1]] -= 0.3
    if y_world is not None:
        L -= 0.6 * (y_world < 3)
    if rim:
        dark, light = rim
        near = lambda m: ndimage.binary_dilation(m) & ~m  # noqa: E731
        L -= 0.8 * (near(areas == light) & (areas == dark)) + 0.3 * (near(areas == dark) & (areas == light))
    if fur and face not in ("up", "down") and w >= 6 and h >= 6:
        cols = (np.arange(w) * 7 + seed) % 5 == 0
        rows = ((np.arange(h)[:, None] + np.arange(w)[None, :] * 3 + seed) % 7) < 3
        L -= 0.55 * (cols[None, :] & rows & np.isin(areas, list(fur)))
    k = np.clip(np.round(L), -steps, steps).astype(int) + steps
    col = np.zeros((h, w, 3))
    for m in np.unique(areas):
        sel = areas == m
        col[sel] = ramps[m][k[sel]]
    return col


# ---------------------------------------------------------------- the smooth look (default)

STEP = 0.25  # light quantum in ramp steps: reads smooth, still clean pixels
KS = np.arange(-3, 3 + STEP / 2, STEP)


def tones(rgb, shadow_hue=0.74):
    """All shades of one base color, k = -3..3 ramp steps in STEP increments (darkest first).
    Pass a warm `shadow_hue` (0.0-0.05) for fur and skin; 0.74 (purple) for stone, metal."""
    h, s, v = colorsys.rgb_to_hsv(*(np.array(rgb, float) / 255))

    def toward(h, target, amt):
        d = (target - h + 0.5) % 1.0 - 0.5
        return (h + math.copysign(min(abs(d), amt), d)) % 1.0

    out = []
    for k in KS:
        if k < 0:
            hh, ss, vv = toward(h, shadow_hue, 0.025 * -k), min(1, s + 0.06 * -k), v * (1 - 0.15 * -k)
        else:
            hh, ss, vv = toward(h, 0.12, 0.02 * k), s * (1 - 0.13 * k), min(0.96, v * (1 + 0.09 * k))
        out.append(np.array(colorsys.hsv_to_rgb(hh, ss, max(vv, 0.08))) * 255)
    return np.array(out)


def occlusion(p, n, du, dv, boxes, own=None):
    """Ambient occlusion per texel: how much of the space just outside the face is taken by other
    boxes. `p`: (h, w, 3) texel centers in world units; `n`, `du`, `dv`: the face's normal and
    texel directions; `boxes`: [(name, lo, hi)] of the solid boxes. Returns (h, w) in 0..1."""
    du, dv = du / np.linalg.norm(du), dv / np.linalg.norm(dv)
    hits, count = 0, 0
    for d in (0.6, 1.6, 3.0):
        for a, b in ((0, 0), (1.5, 0), (-1.5, 0), (0, 1.5), (0, -1.5)):
            q = p + n * d + du * a + dv * b
            inside = np.zeros(p.shape[:2], bool)
            for name, lo, hi in boxes:
                if name != own:
                    inside |= np.all((q > lo) & (q < hi), axis=-1)
            hits = hits + inside / d
            count += 1 / d
    return hits / count


def paint_face_smooth(areas, tone_sets, face, coat=(), y_world=None, occ=None, quad=False, seed=0,
                      tuft=True, occ_strength=2.2, down=None, merge=True, out=None):
    """Paint one face the smooth way. `areas`: (h, w) ints into `tone_sets` (one `tones()` array
    per material). `coat`: material ids that are fur (simplified into big shapes, tufted and
    blended at their edges, streaked); others (eyes, nose, inner ear) stay crisp. `occ`: from
    `occlusion()`. `down`: (h, w) 0..1 from the top of the whole part, for boxes stacked into one
    part (an ear, a wing): it replaces each box's own top-to-bottom light and drops its edge
    highlights, so the stack is lit as one piece. `merge=False` keeps thin coat areas (stripes)
    instead of merging them into big shapes. `out`: a dict that gets each texel's material weights
    (`weights`, (n, h, w), summing to 1) and light level (`level`, index into the ramps), for
    `layers.py` (coats painted on the fly). Returns (h, w, 3)."""
    h, w = areas.shape
    n = len(tone_sets)
    coat = list(coat)
    is_coat = np.isin(areas, coat)
    if merge and min(h, w) >= 3 and coat:  # big simple shapes
        score = np.stack([ndimage.gaussian_filter((areas == m).astype(float), 1.1) for m in range(n)])
        score[[m for m in range(n) if m not in coat]] = -1
        areas = np.where(is_coat, score.argmax(0), areas)
    if tuft and face not in ("up", "down") and h >= 4:  # the color above hangs down in points
        off = np.array([0, 1, 2, 1, 0, 1, 2, 2, 1])[(np.arange(w) + seed) % 9]
        src = np.clip(np.arange(h)[:, None] - off[None, :], 0, h - 1)
        hang = areas[src, np.arange(w)[None, :]]
        areas = np.where(is_coat & np.isin(hang, coat), hang, areas)
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
    L = np.full((h, w), FACE_LIGHT[face])
    if face not in ("up", "down"):
        L += 0.6 - 1.2 * (vv / h if down is None else down)
        if not quad and down is None:
            L += 0.5 * np.exp(-(vv - 0.5)) - 0.3 * np.exp(-(h - vv - 0.5) / 1.2)
            if w >= 6:
                L -= 0.35 * (np.exp(-(uu - 0.5) / 1.3) + np.exp(-(w - uu - 0.5) / 1.3))
    elif w >= 4 and h >= 4:
        edge = np.minimum(np.minimum(uu, w - uu), np.minimum(vv, h - vv))
        L -= 0.4 * np.exp(-(edge - 0.5) / 1.3)
    if y_world is not None:
        L -= 1.0 * np.exp(-np.maximum(y_world, 0) / 2.0)
    if occ is not None:
        L -= occ_strength * occ
    g = ndimage.gaussian_filter(np.random.default_rng(seed).standard_normal((h, w)), (1.6, 0.45))
    L += 0.22 * g / (g.std() + 1e-6) * is_coat  # strands
    ki = np.round((np.clip(np.round(L / STEP) * STEP, KS[0], KS[-1]) - KS[0]) / STEP).astype(int)
    blur = np.stack([ndimage.gaussian_filter((areas == m).astype(float), 0.6) for m in range(n)])
    crisp = ~np.isin(areas, coat)
    col = np.zeros((h, w, 3))
    total = np.zeros((h, w))
    weights = []
    for m in range(n):
        wm = np.where(crisp, (areas == m).astype(float), blur[m] * (m in coat))
        col += wm[..., None] * tone_sets[m][ki]
        total += wm
        weights.append(wm)
    if out is not None:
        out["weights"] = np.stack(weights) / np.maximum(total, 1e-6)
        out["level"] = ki
    return col / np.maximum(total, 1e-6)[..., None]


def stamp(img, pattern, colors, u0, v0):
    """Paint a small pixel design (list of strings, '.' = leave) at texel (u0, v0)."""
    for r, row in enumerate(pattern):
        for c, ch in enumerate(row):
            if ch != ".":
                img[v0 + r, u0 + c, :3] = colors[ch]
    return img


def majority(areas, keep=(), passes=3):
    """Smooth an area map into clean shapes (3x3 vote); ids in `keep` are left alone."""
    n = int(areas.max()) + 1
    for _ in range(passes):
        h, w = areas.shape
        pad = np.pad(areas, 1, mode="edge")
        votes = np.zeros((n, h, w))
        for dy in range(3):
            for dx in range(3):
                votes[pad[dy:dy + h, dx:dx + w], np.arange(h)[:, None], np.arange(w)[None, :]] += 1 + (dy == 1 and dx == 1)
        areas = np.where(np.isin(areas, list(keep)), areas, votes.argmax(0))
    return areas


def pack(tiles, width=512):
    """Shelf-pack face images [(key, (h, w, 4) uint8)] with a 1-texel gutter copied from each
    edge (no seams from filtering). Returns (atlas, {key: [u0, v0, u1, v1]}), height a multiple
    of 32."""
    tiles = sorted(tiles, key=lambda t: (-t[1].shape[0], -t[1].shape[1]))
    sheet = np.zeros((4096, width, 4), np.uint8)
    x = y = row = 0
    uv = {}
    for key, img in tiles:
        h, w = img.shape[:2]
        if x + w + 2 > width:
            x, y, row = 0, y + row, 0
        sheet[y:y + h + 2, x:x + w + 2] = np.pad(img, ((1, 1), (1, 1), (0, 0)), mode="edge")
        uv[key] = [x + 1, y + 1, x + 1 + w, y + 1 + h]
        x += w + 2
        row = max(row, h + 2)
    return sheet[:32 * math.ceil((y + row) / 32)], uv


# ---------------------------------------------------------------- eyes (types, not expressions)

# Hytale-style creature eyes (from its corgi): big (~8 texels wide on a 22-wide face), a dark
# ring (O) around a near-black pupil (P), a warmer glow along the bottom (I), a cool white 2x2
# highlight (H) and a small grey-blue sparkle (s), set in darker fur ("round" halo). Recolor O/I
# per creature (amber wolf, green cat, blue-black bird). Then simpler cute types. '.' = face.
_P, _H, _s = (16, 12, 14), (232, 236, 240), (150, 162, 178)
_O, _I, _L = (62, 36, 26), (98, 56, 36), (40, 26, 26)
_D, _W = (36, 25, 31), (240, 242, 244)
_ROUND8 = ["..OOOO..", ".OHHPPO.", "OPHHPPPO", "OPPPPPPO", "OPPPPsPO", ".OIIIIO.", "..OOOO.."]
EYES = {
    "hytale": dict(pattern=_ROUND8, colors={"O": _O, "P": _P, "H": _H, "s": _s, "I": _I}, halo="round"),
    "hytale_amber": dict(pattern=["..LLLL..", ".LAAAAL.", "LAHHPPAL", "LAHPPPAL", "LAPPPPAL", ".LaAAaL.", "..LLLL.."],
                         colors={"L": _L, "A": (156, 92, 38), "a": (196, 128, 58), "P": _P, "H": _H}, halo="round"),
    "hytale_oval": dict(pattern=["..OO..", ".OHHO.", "OPHHPO", "OPPPPO", "OPPPPO", "OPPsPO", ".OIIO.", "..OO.."],
                        colors={"O": _O, "P": _P, "H": _H, "s": _s, "I": _I}, halo="round"),
    "hytale_gradient": dict(pattern=["..PPP..", ".PHHPP.", "PPHHPPP", "PPPPPPP", "PgggggP", ".IIsII.", "..iii.."],
                            colors={"P": _P, "H": _H, "s": _s, "g": (52, 30, 24), "I": (104, 58, 36), "i": (146, 88, 50)},
                            halo="round"),
    "hytale_whites": dict(pattern=[".LLLLLL.", "WOHHPPOW", "WOHPPPOW", "WOPPPPOW", ".LOIIOL.", "..wwww.."],
                          colors={"L": _L, "W": (212, 202, 188), "O": _O, "P": _P, "H": _H, "I": _I, "w": (176, 118, 80)},
                          halo="below"),
    "hytale_mask": dict(pattern=_ROUND8, colors={"O": _O, "P": _P, "H": _H, "s": _s, "I": _I}, halo="mask"),
    # Between Hytale's and the cute sparkle: sparkle's size (6 wide) with Hytale's ring, cool
    # highlight and sparkle, warm glow and soft fur ring. A good default for cute creatures.
    "hytale_cute": dict(pattern=[".OOOO.", "OHHPPO", "OHHPPO", "OPPPsO", "OIIIIO", ".IIII."],
                        colors={"O": _O, "P": _P, "H": _H, "s": _s, "I": _I}, halo="round"),
    "sparkle": dict(pattern=[".DDDD.", "DHHDDD", "DHHDDD", "DDDDSD", "DIIIID", ".IIII."],
                    colors={"D": _D, "H": _W, "S": (190, 196, 206), "I": (120, 70, 44)}, halo="below"),
    "glossy": dict(pattern=[".DDDD.", "DHHDDD", "DHHDDD", "DDDDDD", "DDDDHD", "DIIIID", ".IIII."],
                   colors={"D": _D, "H": _W, "I": (120, 70, 44)}, halo="below"),
    "button": dict(pattern=[".DD.", "DHDD", "DDDD", ".DD."], colors={"D": _D, "H": _W}, halo="below"),
    "anime": dict(pattern=[".DD.", "DHHD", "DHDD", "DDDD", "DDSD", "DIID", ".II."],
                  colors={"D": _D, "H": _W, "S": (190, 196, 206), "I": (120, 70, 44)}, halo="below"),
}


def stamp_eyes(img, centers, design="hytale", colors=None):
    """Stamp a pair of designed eyes on a face image (h, w, 3|4) at texel centers [(u, v), (u, v)]
    (left one first, as seen from the front). A design may set mirror=True (an eyelid wing that
    sweeps outward). halo: "round" darker fur all around (lighter above), "below" only sides and
    below (shading above alone reads as a frown), "mask" a strong patch with lighter fur under it.
    `colors` overrides keys. Only ever on the front of the head."""
    d = EYES[design]
    cols = {**d["colors"], **(colors or {})}
    for i, (cu, cv) in enumerate(centers):
        pat = [row[::-1] for row in d["pattern"]] if (d.get("mirror") and i == 1) else d["pattern"]
        ph, pw = len(pat), len(pat[0])
        u0, v0 = int(round(cu - pw / 2)), int(round(cv - ph / 2))
        m = np.zeros(img.shape[:2], bool)
        for r, row in enumerate(pat):
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
        elif halo == "mask":
            f[ring] *= 0.7
            f[ring2 & ~above] *= 0.8
            f[ring2 & above] *= 0.88
            below = np.roll(ring2, 1, axis=0) & ~m & ~ring & ~ring2
            below[:v0 + ph] = False
            f[below] = np.minimum(f[below] * 1.12, 245)
        for r, row in enumerate(pat):
            for c, ch in enumerate(row):
                if ch != ".":
                    f[v0 + r, u0 + c] = cols[ch]
        img[..., :3] = np.clip(np.round(f), 0, 255).astype(img.dtype)
    return img
