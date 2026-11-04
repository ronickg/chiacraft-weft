#!/usr/bin/env python3
"""Render a box model (this skill's spec.json + texture.png) to a picture without Blockbench: flat
(the texture carries its light), nearest texels, cut-out alpha, a z-buffer. For review shots and
before/after sheets.

    python render.py spec.json texture.png out.png [--scale 0.25] [--ppb 256] [--yaw 35] [--pitch 20]
        [--anim sit --t 0] [--views 3q,side,front,head]

`--scale`: model pixels a unit, as exported (`house.SCALE` or a build's own): with `--ppb` (screen
pixels a block) it sets how big the model is drawn, so two builds at different densities come out
the same size and their texels can be compared. Views: `3q` (front three-quarters from above),
`side`, `front`, `back`, `top`, `head` (the head close up, 3/4).

Faces follow the painters' `face_frame` (u right and v down seen from outside the face); a quad
shows its texture on both sides, mirrored on the back, as `to_bedrock` exports it.
"""

import argparse
import json
import math

import numpy as np
from PIL import Image

from ground import rot, world

FRAMES = {"south": ("lo0 hi1 hi2", (1, 0, 0), (0, -1, 0), (0, 0, 1)),
          "north": ("hi0 hi1 lo2", (-1, 0, 0), (0, -1, 0), (0, 0, -1)),
          "east": ("hi0 hi1 hi2", (0, 0, -1), (0, -1, 0), (1, 0, 0)),
          "west": ("lo0 hi1 lo2", (0, 0, 1), (0, -1, 0), (-1, 0, 0)),
          "up": ("lo0 hi1 lo2", (1, 0, 0), (0, 0, 1), (0, 1, 0)),
          "down": ("lo0 lo1 hi2", (1, 0, 0), (0, 0, -1), (0, -1, 0))}


def face_quad(lo, hi, face):
    """(corner, U, V): the face's corner and its full u and v edges (world vectors)."""
    spec, du, dv, _ = FRAMES[face]
    pick = {"lo": lo, "hi": hi}
    corner = np.array([pick[s[:2]][int(s[2])] for s in spec.split()], float)
    size = hi - lo
    du, dv = np.array(du, float), np.array(dv, float)
    return corner, du * abs(size @ du), dv * abs(size @ dv)


def view_matrix(yaw, pitch):
    """Camera rotation: world -> view (x right, y up, z toward the viewer)."""
    return rot(-pitch, 0, 0) @ rot(0, -yaw, 0)


def render(spec, tex, pose=None, yaw=35, pitch=20, ppu=8.0, size=None, center=None, pad=12, bg=(0, 0, 0, 0)):
    """An RGBA image of the model posed (`pose`: {bone: rot, "root": offset}), `ppu` screen pixels
    a unit. `size`/`center` fix the frame (screen pixels, view-space center), else it fits."""
    pose = pose or {}
    W = world(spec["bones"], pose)
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}
    C = view_matrix(yaw, pitch)
    tris = []
    for e in spec["elements"]:
        R, t = W[e["bone"]]
        lo, hi = np.array(e["lo"], float), np.array(e["hi"], float)
        er = rot(*e["rot"]) if e.get("rot") else np.eye(3)
        eo = np.array(e["origin"], float) if e.get("rot") else np.zeros(3)

        def to_world(p):
            p = (p - eo) @ er.T + eo
            return (p - piv[e["bone"]]) @ R.T + t

        for face, rect in (e.get("faces") or {}).items():
            if rect is None:
                continue
            corner, U, V = face_quad(lo, hi, face)
            pts = to_world(np.array([corner, corner + U, corner + V, corner + U + V]))
            tris.append((pts, rect, bool(e.get("quad"))))
    # view space
    allp = np.concatenate([p for p, _, _ in tris]) @ C.T
    if center is None:
        center = (allp.min(0) + allp.max(0)) / 2
    if size is None:
        ext = allp.max(0) - allp.min(0)
        size = (int(ext[0] * ppu) + 2 * pad, int(ext[1] * ppu) + 2 * pad)
    wpx, hpx = size
    img = np.zeros((hpx, wpx, 4), np.uint8)
    img[:] = bg
    zb = np.full((hpx, wpx), -np.inf)
    th, tw = tex.shape[:2]
    for pts, rect, quad in tris:
        q = pts @ C.T - center
        sx = q[:, 0] * ppu + wpx / 2
        sy = -q[:, 1] * ppu + hpx / 2
        o = np.array([sx[0], sy[0]])
        a = np.array([sx[1], sy[1]]) - o
        b = np.array([sx[2], sy[2]]) - o
        det = a[0] * b[1] - a[1] * b[0]
        if abs(det) < 1e-6:
            continue
        # facing: normal U x V points outward; in screen y-down, outward-facing faces have det < 0... test with view z
        n = np.cross(pts[1] - pts[0], pts[2] - pts[0]) @ C.T
        front = n[2] < 0  # U x V (u right, v down) points into the box; the outward normal is -(U x V)
        if not front and not quad:
            continue
        xs = [o[0], o[0] + a[0], o[0] + b[0], o[0] + a[0] + b[0]]
        ys = [o[1], o[1] + a[1], o[1] + b[1], o[1] + a[1] + b[1]]
        x0, x1 = max(0, int(math.floor(min(xs)))), min(wpx, int(math.ceil(max(xs))))
        y0, y1 = max(0, int(math.floor(min(ys)))), min(hpx, int(math.ceil(max(ys))))
        if x0 >= x1 or y0 >= y1:
            continue
        gx, gy = np.meshgrid(np.arange(x0, x1) + 0.5, np.arange(y0, y1) + 0.5)
        dx, dy = gx - o[0], gy - o[1]
        s = (dx * b[1] - dy * b[0]) / det
        tt = (a[0] * dy - a[1] * dx) / det
        inside = (s >= 0) & (s < 1) & (tt >= 0) & (tt < 1)
        if not inside.any():
            continue
        z = q[0, 2] + s * (q[1, 2] - q[0, 2]) + tt * (q[2, 2] - q[0, 2])
        u0, v0, u1, v1 = rect
        su = s if front else 1 - s  # the back of a quad: mirrored in u
        tu = np.clip(np.floor(u0 + su * (u1 - u0)).astype(int), 0, tw - 1)
        tv = np.clip(np.floor(v0 + tt * (v1 - v0)).astype(int), 0, th - 1)
        col = tex[tv, tu]
        ok = inside & (col[..., 3] > 127) & (z > zb[y0:y1, x0:x1])
        zb[y0:y1, x0:x1][ok] = z[ok]
        img[y0:y1, x0:x1][ok] = col[ok]
        img[y0:y1, x0:x1, 3][ok] = 255
    return img


VIEWS = {"3q": (35, 20), "side": (90, 0), "front": (0, 0), "back": (180, 10), "top": (0, 89), "head": (30, 12),
         "3q_left": (-35, 20)}


def pose_at(spec, anim, t=0.0):
    if not anim:
        return {}
    fr = spec["animations"][anim]["frames"]
    f = min(fr, key=lambda f: abs(f["t"] - t))
    return dict(f["rot"], root=f.get("root", (0, 0, 0)))


def head_center(spec, pose, yaw, pitch):
    """View-space center of the head's boxes (for a close-up)."""
    W = world(spec["bones"], pose)
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}
    names = {"head", "skull"}
    pts = []
    for e in spec["elements"]:
        if e["bone"] in names or e["name"] in ("skull", "head"):
            R, t = W[e["bone"]]
            c = (np.array(e["lo"]) + np.array(e["hi"])) / 2
            pts.append((c - piv[e["bone"]]) @ R.T + t)
    if not pts:
        return None
    return np.mean(pts, 0) @ view_matrix(yaw, pitch).T


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("spec")
    ap.add_argument("texture")
    ap.add_argument("out")
    ap.add_argument("--scale", type=float, default=0.25)
    ap.add_argument("--ppb", type=float, default=256)
    ap.add_argument("--anim")
    ap.add_argument("--t", type=float, default=0)
    ap.add_argument("--views", default="3q,side")
    a = ap.parse_args()
    spec = json.load(open(a.spec))
    tex = np.array(Image.open(a.texture).convert("RGBA"))
    ppu = a.ppb * a.scale / 16
    pose = pose_at(spec, a.anim, a.t)
    shots = []
    for v in a.views.split(","):
        yaw, pitch = VIEWS[v]
        if v == "head":
            c = head_center(spec, pose, yaw, pitch)
            shots.append(render(spec, tex, pose, yaw, pitch, ppu * 3, size=(360, 360), center=c))
        else:
            shots.append(render(spec, tex, pose, yaw, pitch, ppu))
    h = max(s.shape[0] for s in shots)
    sheet = np.zeros((h, sum(s.shape[1] for s in shots), 4), np.uint8)
    x = 0
    for s in shots:
        sheet[:s.shape[0], x:x + s.shape[1]] = s
        x += s.shape[1]
    Image.fromarray(sheet).save(a.out)
