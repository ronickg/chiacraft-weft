#!/usr/bin/env python3
"""Draws the chihuahua's model files the way the engine poses them (bones, the `idle`
animation's pose at a moment, clear texels cut out, vanilla's face shading), from a few sides, into
one picture: a quick look without starting the game. Needs numpy and Pillow.

    python3 mods/chihuahua/tools/preview.py out.png [coat] [model:animations ...]

e.g. `preview.py out.png 28 chihuahua:chihuahua chihuahua:chihuahua_sit chihuahua_pup:chihuahua_pup`.
"""

import json
import math
import os
import sys

import numpy as np
from PIL import Image

ASSETS = os.path.join(os.path.dirname(__file__), "..", "assets")
FACES = ["east", "west", "down", "up", "north", "south"]
CORNERS = [
    [(0, 0, 0), (0, 0, 1), (0, 1, 1), (0, 1, 0)],
    [(1, 0, 1), (1, 0, 0), (1, 1, 0), (1, 1, 1)],
    [(0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)],
    [(0, 1, 1), (1, 1, 1), (1, 1, 0), (0, 1, 0)],
    [(1, 0, 0), (0, 0, 0), (0, 1, 0), (1, 1, 0)],
    [(0, 0, 1), (1, 0, 1), (1, 1, 1), (0, 1, 1)],
]
UVS = [(0, 1), (1, 1), (1, 0), (0, 0)]
SHADE = [0.6, 0.6, 0.5, 1.0, 0.8, 0.8]


def angles(r):
    return [-math.radians(r[0]), math.radians(r[1]), -math.radians(r[2])]


def rot(rx, ry, rz):
    cx, sx, cy, sy, cz, sz = math.cos(rx), math.sin(rx), math.cos(ry), math.sin(ry), math.cos(rz), math.sin(rz)
    X = np.array([[1, 0, 0], [0, cx, -sx], [0, sx, cx]])
    Y = np.array([[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]])
    Z = np.array([[cz, -sz, 0], [sz, cz, 0], [0, 0, 1]])
    return Z @ Y @ X


def affine(m3, t):
    m = np.eye(4)
    m[:3, :3] = m3
    m[:3, 3] = t
    return m


def sample(frames, t):
    """A channel's value at t (linear; enough for a still)."""
    if isinstance(frames, list):
        return frames
    keys = sorted((float(k), v["post"] if isinstance(v, dict) else v) for k, v in frames.items())
    if t <= keys[0][0]:
        return keys[0][1]
    for (t0, a), (t1, b) in zip(keys, keys[1:]):
        if t <= t1:
            f = (t - t0) / max(t1 - t0, 1e-6)
            return [a[i] + (b[i] - a[i]) * f for i in range(3)]
    return keys[-1][1]


def pose(bones, anim, t):
    names = [b["name"] for b in bones]
    mats = []
    for b in bones:
        r = angles(b.get("rotation", [0, 0, 0]))
        off, sc = np.zeros(3), np.ones(3)
        ch = (anim or {}).get("bones", {}).get(b["name"], {})
        if "rotation" in ch:
            a = angles(sample(ch["rotation"], t))
            r = [r[i] + a[i] for i in range(3)]
        if "position" in ch:
            off = np.array(sample(ch["position"], t), dtype=float)
        if "scale" in ch:
            sc = np.array(sample(ch["scale"], t), dtype=float)
        p = np.array(b["pivot"], dtype=float)
        local = affine(np.eye(3), off + p) @ affine(rot(r[0], r[1], r[2]) @ np.diag(sc), np.zeros(3)) @ affine(np.eye(3), -p)
        parent = mats[names.index(b["parent"])] if b.get("parent") in names else np.eye(4)
        mats.append(parent @ local)
    return mats


def render(geo, tex, anim, t, yaw, pitch, size=360, zoom=None):
    g = geo["minecraft:geometry"][0]
    tw, th = g["description"]["texture_width"], g["description"]["texture_height"]
    bones = g["bones"]
    mats = pose(bones, anim, t)
    view = rot(math.radians(pitch), 0, 0) @ rot(0, math.radians(yaw), 0)
    tris = []
    for b, m in zip(bones, mats):
        pm = b.get("poly_mesh")
        if pm:
            pos, uv = pm["positions"], pm["uvs"]
            for poly in pm["polys"]:
                pts = [view @ (m @ np.append(np.array(pos[c[0]], float), 1.0))[:3] for c in poly]
                uvs = [(uv[c[2]][0] / tw, uv[c[2]][1] / th) for c in poly]
                n = np.cross(pts[1] - pts[0], pts[2] - pts[0])
                n = view.T @ (n / (np.linalg.norm(n) + 1e-9))
                shade = n[0] ** 2 * 0.6 + n[2] ** 2 * 0.8 + n[1] ** 2 * (1.0 if n[1] > 0 else 0.5)
                for a, bb, cc in ((0, 1, 2), (0, 2, 3)):
                    tris.append((pts[a], pts[bb], pts[cc], uvs[a], uvs[bb], uvs[cc], shade))
        for c in b.get("cubes", []):
            o, s = np.array(c["origin"], float), np.array(c["size"], float)
            for f, name in enumerate(FACES):
                fu = c["uv"][name]
                u, v = fu["uv"]
                w, hh = fu["uv_size"]
                if w == 0 or hh == 0:
                    continue
                rect = (u, v + hh, w, -hh) if name == "down" else (u, v, w, hh)
                pts, uvs = [], []
                for k in range(4):
                    p = o + np.array(CORNERS[f][k]) * s
                    q = view @ (m @ np.append(p, 1.0))[:3]
                    pts.append(q)
                    uvs.append(((rect[0] + UVS[k][0] * rect[2]) / tw, (rect[1] + UVS[k][1] * rect[3]) / th))
                for a, bb, cc in ((0, 1, 2), (0, 2, 3)):
                    tris.append((pts[a], pts[bb], pts[cc], uvs[a], uvs[bb], uvs[cc], SHADE[f]))
    allp = np.array([p for t_ in tris for p in t_[:3]])
    lo, hi = allp.min(0), allp.max(0)
    span = zoom or max(hi[0] - lo[0], hi[1] - lo[1]) * 1.15
    cx, cy = (lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2
    k = size / span
    img = np.zeros((size, size, 3)) + np.array([150, 190, 120])
    depth = np.full((size, size), -1e9)
    texa = np.asarray(tex.convert("RGBA"), dtype=float)
    TH, TW = texa.shape[:2]
    for p0, p1, p2, t0, t1, t2, shade in tris:
        P = np.array([[(p[0] - cx) * k + size / 2, (cy - p[1]) * k + size / 2, p[2]] for p in (p0, p1, p2)])
        x0, x1 = int(max(0, math.floor(P[:, 0].min()))), int(min(size - 1, math.ceil(P[:, 0].max())))
        y0, y1 = int(max(0, math.floor(P[:, 1].min()))), int(min(size - 1, math.ceil(P[:, 1].max())))
        if x1 < x0 or y1 < y0:
            continue
        area = (P[1, 0] - P[0, 0]) * (P[2, 1] - P[0, 1]) - (P[2, 0] - P[0, 0]) * (P[1, 1] - P[0, 1])
        # Back faces are culled, as in the engine.
        if area > -1e-9:
            continue
        xs, ys = np.meshgrid(np.arange(x0, x1 + 1) + 0.5, np.arange(y0, y1 + 1) + 0.5)
        w0 = ((P[1, 0] - xs) * (P[2, 1] - ys) - (P[2, 0] - xs) * (P[1, 1] - ys)) / area
        w1 = ((P[2, 0] - xs) * (P[0, 1] - ys) - (P[0, 0] - xs) * (P[2, 1] - ys)) / area
        w2 = 1 - w0 - w1
        inside = (w0 >= 0) & (w1 >= 0) & (w2 >= 0)
        if not inside.any():
            continue
        z = w0 * P[0, 2] + w1 * P[1, 2] + w2 * P[2, 2]
        u = w0 * t0[0] + w1 * t1[0] + w2 * t2[0]
        v = w0 * t0[1] + w1 * t1[1] + w2 * t2[1]
        tx = np.clip((u * TW).astype(int), 0, TW - 1)
        ty = np.clip((v * TH).astype(int), 0, TH - 1)
        col = texa[ty, tx]
        sub = depth[y0 : y1 + 1, x0 : x1 + 1]
        ok = inside & (col[..., 3] >= 128) & (z > sub)
        sub[ok] = z[ok]
        img[y0 : y1 + 1, x0 : x1 + 1][ok] = col[..., :3][ok] * shade
    return Image.fromarray(img.clip(0, 255).astype(np.uint8))


def main():
    out = sys.argv[1]
    coat = sys.argv[2] if len(sys.argv) > 2 else "28"
    pairs = sys.argv[3:] or ["chihuahua:chihuahua", "chihuahua:chihuahua_sit", "chihuahua:chihuahua_lie", "chihuahua_pup:chihuahua_pup"]
    tex = Image.open(os.path.join(ASSETS, "textures", "entity", "chihuahua", f"coat_{coat}.png"))
    views = [(180, -12), (220, -20), (270, -8), (330, -20), (0, -15)]
    size = 300
    sheet = Image.new("RGB", (size * len(views), size * len(pairs)))
    for r, pair in enumerate(pairs):
        model, anims = pair.split(":")
        geo = json.load(open(os.path.join(ASSETS, "models", f"{model}.geo.json")))
        anim_file = json.load(open(os.path.join(ASSETS, "animations", f"{anims}.animation.json")))
        idle = next((a for n, a in anim_file["animations"].items() if n.endswith(".idle")), None)
        for c, (yaw, pitch) in enumerate(views):
            sheet.paste(render(geo, tex, idle, 0.0, yaw, pitch, size, zoom=22 if "pup" not in model else 14), (c * size, r * size))
    sheet.save(out)


if __name__ == "__main__":
    main()
