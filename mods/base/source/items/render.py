"""Renders item models (the quads `tools.py out/` writes) from a few angles into one picture, for
review without the game: orthographic, nearest texels, a soft light from the top left.

    ~/.venvs/blocky/bin/python mods/base/source/items/render.py out/ shots.png [names...] [--views flat,3/4,side,top]
"""
import json
import math
import os
import sys

import numpy as np
from PIL import Image

VIEWS = {"flat": (0, 0), "3/4": (-35, 20), "side": (-90, 0), "top": (-20, 70), "back": (180, 15)}


def camera(yaw, pitch):
    y, p = math.radians(yaw), math.radians(pitch)
    Ry = np.array([[math.cos(y), 0, math.sin(y)], [0, 1, 0], [-math.sin(y), 0, math.cos(y)]])
    Rx = np.array([[1, 0, 0], [0, math.cos(p), -math.sin(p)], [0, math.sin(p), math.cos(p)]])
    return Rx @ Ry


def render(model, tex, view, size=260, span=100):
    M = camera(*VIEWS[view])
    img = np.zeros((size, size, 3))
    img[:] = (52, 62, 72)
    zbuf = np.full((size, size), -1e9)
    k = size / span
    light = np.array([-0.45, 0.75, 0.5])
    light /= np.linalg.norm(light)
    th, tw = tex.shape[:2]
    for q in model["quads"]:
        P = np.array(q["p"]) @ M.T
        n = M @ np.array(q["n"])
        if n[2] <= 0:
            continue
        shade = 0.72 + 0.28 * max(0.0, float(np.array(q["n"]) @ light))
        s = P[:, :2] * [k, -k] + size / 2
        x0, y0 = np.floor(s.min(0)).astype(int)
        x1, y1 = np.ceil(s.max(0)).astype(int)
        x0, y0, x1, y1 = max(x0, 0), max(y0, 0), min(x1, size), min(y1, size)
        if x1 <= x0 or y1 <= y0:
            continue
        a, b = s[1] - s[0], s[3] - s[0]
        det = a[0] * b[1] - a[1] * b[0]
        if abs(det) < 1e-9:
            continue
        yy, xx = np.mgrid[y0:y1, x0:x1] + 0.5
        dx, dy = xx - s[0][0], yy - s[0][1]
        u = (dx * b[1] - dy * b[0]) / det
        v = (a[0] * dy - a[1] * dx) / det
        inside = (u >= 0) & (u <= 1) & (v >= 0) & (v <= 1)
        z = P[0][2] + u * (P[1][2] - P[0][2]) + v * (P[3][2] - P[0][2])
        u0, v0, u1, v1 = q["uv"]
        tx = np.clip(((u0 + u * (u1 - u0)) * tw).astype(int), 0, tw - 1)
        ty = np.clip(((v0 + v * (v1 - v0)) * th).astype(int), 0, th - 1)
        reg = zbuf[y0:y1, x0:x1]
        win = inside & (z > reg)
        reg[win] = z[win]
        img[y0:y1, x0:x1][win] = tex[ty[win], tx[win], :3] * shade
    return img


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    views = ["flat", "3/4", "side", "top"]
    for a in sys.argv[1:]:
        if a.startswith("--views="):
            views = a.split("=", 1)[1].split(",")
    folder, out, names = args[0], args[1], args[2:]
    if not names:
        names = sorted(f[:-5] for f in os.listdir(folder) if f.endswith(".json"))
    size = 260
    sheet = Image.new("RGB", (size * (len(views) + 1), size * len(names)), (52, 62, 72))
    for r, name in enumerate(names):
        icon = os.path.join(folder, name + "_icon.png")
        if os.path.exists(icon):  # the icon it comes from, first
            im = Image.open(icon).convert("RGBA").resize((size - 40, size - 40), Image.NEAREST)
            sheet.paste(im, (20, r * size + 20), im)
        model = json.load(open(os.path.join(folder, name + ".json")))
        tex = np.asarray(Image.open(os.path.join(folder, model["texture"])).convert("RGBA")).astype(float)
        for c, view in enumerate(views):
            im = render(model, tex, view, size)
            sheet.paste(Image.fromarray(im.clip(0, 255).astype(np.uint8)), ((c + 1) * size, r * size))
    sheet.save(out)


if __name__ == "__main__":
    main()
