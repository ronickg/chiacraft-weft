"""Egg shapes and shell patterns for `build.py`: an egg laid out as stacked square layers along
its long axis (pointed end at the bottom), tilted as it lies in the nest, and its shell painted
by where each texel sits on the egg (round it and along it), so marks wrap round the corners.

Units are the house style's layout units (1/64 m); `tx` is texels a unit.
"""

import math

import numpy as np

from prop import el, rotm


def profile(x, asym=0.0, blunt=2.0):
    """The egg's half-width at `x` (-1 pointed end .. 1 blunt end), 0..1: a superellipse
    (`blunt` 2 is an ellipse, more is blunter, a crocodile's barrel), wider toward the blunt end
    by `asym` (0 symmetric, 0.3 a hen's egg)."""
    x = np.clip(np.asarray(x, float), -1, 1)
    r = (1 - np.abs(x) ** blunt) ** (1 / blunt) * (1 + asym * x)
    xs = np.linspace(-1, 1, 801)
    top = ((1 - np.abs(xs) ** blunt) ** (1 / blunt) * (1 + asym * xs)).max()
    return r / top


MIN_STEP = 0.65  # units neighbors differ in width at least: 0.325 a side (boxcheck flags faces under 0.3 apart)


def layers(L, W, asym=0.0, blunt=2.0, n=None, ov=0.25, step=None):
    """[(width, y0, y1)] bottom to top: `n` slices, each as wide as the profile at its middle,
    snapped to `step`; neighbors overlap by `ov`, and slices within MIN_STEP of the one below
    merge with it (the wider), so no two boxes have faces close enough to flicker."""
    step = step or (1.0 if W >= 4 else 0.75)
    n = n or int(np.clip(round(L / 0.9), 3, 18))
    h = L / n
    ov = max(0.05, min(ov, 0.3 * h, h - 0.35))  # tiny eggs: bottoms 0.35 apart
    out = []
    for i in range(n):
        mid = -1 + (2 * i + 1) / n
        w = max(step, round(W * float(profile(mid, asym, blunt)) / step) * step)
        y0, y1 = max(0.0, i * h - ov), min(L, (i + 1) * h + ov)
        if out and abs(out[-1][0] - w) < MIN_STEP - 1e-6:
            out[-1] = (max(w, out[-1][0]), out[-1][1], y1)
        else:
            out.append((w, y0, y1))
    return out


def egg_elements(name, L, W, asym=0.0, blunt=2.0, tilt=0.0, yaw=0.0, at=(0.0, 0.0, 0.0), n=None, sink=0.0, step=None):
    """One egg's boxes, standing on its pointed end and tilted by `tilt` degrees (its top toward
    +z, 90: lying), turned by `yaw`, centered over `at`; lifted so its lowest corner touches the
    ground (`sink`: units pushed into it). Each element remembers its egg (for painting)."""
    ls = layers(L, W, asym, blunt, n, step=step)
    R = rotm((tilt, yaw, 0))
    # Lowest corner after the tilt: lift by it so the egg rests on y = at.y.
    pts = np.array([R @ np.array([sx * w / 2, y, sz * w / 2]) for w, y0, y1 in ls for y in (y0, y1)
                    for sx in (-1, 1) for sz in (-1, 1)])
    lo_y = pts[:, 1].min()
    # Centered over `at` across the ground, however it leans (its box and its clicks fit it).
    cx, cz = (pts[:, 0].min() + pts[:, 0].max()) / 2, (pts[:, 2].min() + pts[:, 2].max()) / 2
    x, y, z = at
    o = (x - cx, y - lo_y - sink, z - cz)
    egg = {"L": L, "W": W, "asym": asym, "blunt": blunt, "tilt": tilt, "yaw": yaw, "o": o, "name": name}
    return [el(f"{name}{k}", (o[0] - w / 2, o[1] + y0, o[2] - w / 2), (o[0] + w / 2, o[1] + y1, o[2] + w / 2),
               rot=(tilt, yaw, 0) if tilt or yaw else None, origin=o, egg=egg, layer=w)
            for k, (w, y0, y1) in enumerate(ls)]


def local(e, p):
    """Texels `p` (world units) on egg element `e` in the egg's own frame: (u, v, th, x, z): v
    along it from the pointed end, th the angle round it, u how far round it (0 to 4 W, a W a
    face, whatever the layer's width), x and z across it."""
    g = e["egg"]
    R = rotm((g["tilt"], g["yaw"], 0))
    q = (p - np.array(g["o"], float)) @ R  # undo the tilt (R is orthonormal: p @ R = R^T p)
    x, z = q[..., 0], q[..., 2]
    th = np.arctan2(z, x)
    # Round the square layers by their faces, evenly along each (an angle bunches texels at the
    # corners): 0..4, a face each, scaled to the egg's widest so marks line up layer to layer.
    h = np.maximum(np.maximum(np.abs(x), np.abs(z)), 1e-6)
    qx = np.where(x > 0, (z / h + 1) / 2, 2 + (-z / h + 1) / 2)
    qz = np.where(z > 0, 1 + (-x / h + 1) / 2, 3 + (x / h + 1) / 2)
    u = np.where(np.abs(x) >= np.abs(z), qx, qz) % 4 * g["W"]
    return u, q[..., 1], th, x, z


# ------------------------------------------------------------------ marks

def wrap(d, period):
    return (d + period / 2) % period - period / 2


def blobs(rng, n, v_range, size, W, rough=0.35):
    """Blotches: (u, v, radius, phase), u anywhere round the egg."""
    per = 4 * W
    return [(rng.uniform(0, per), rng.uniform(*v_range), rng.uniform(*size), rng.uniform(0, 6), rough) for _ in range(n)]


def stamp(a, u, v, W, marks, mat):
    """Paint `mat` inside each blotch of `marks` (ragged edges by their phase)."""
    per = 4 * W
    for bu, bv, r, ph, rough in marks:
        du, dv = wrap(u - bu, per), v - bv
        ang = np.arctan2(dv, du)
        edge = r * (1 + rough * np.sin(3 * ang + ph))
        a = np.where(np.hypot(du, dv) < edge, mat, a)
    return a


def hashed(ix, iy, seed):
    """A stable pseudo-random 0..1 per integer cell."""
    h = (ix * 374761393 + iy * 668265263 + seed * 2147483647) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 1274126177) & 0xFFFFFFFF
    return (h & 0xFFFF) / 65535.0
