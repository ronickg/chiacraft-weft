#!/usr/bin/env python3
"""The thin-branch-block study (Ronald, 11-05: "we also have branches ... a bit like a fence ... just
a narrower thing"): a tree's wood drawn as today's full logs next to the same tree whose thin wood is
branch blocks, a post 4 or 8 px thick at the block's middle with an arm to each wood neighbour,
like a fence, in the log's bark. With and without leaves, so the wood shows.

    cargo run -p base_api --example tree_widths -- 3 oak > widths.json
    ~/.venvs/blocky/bin/python mods/base/source/trees/branches.py widths.json out.png [base:oak_log]
"""

import json
import sys

from PIL import Image, ImageDraw, ImageFont

from audit import SPECIES, atlas, bbox
from render import render, view_matrix
import numpy as np

THIN, MID = 0.4, 0.75  # wood under THIN wide is a 4 px branch, under MID an 8 px one, else a log
END = {1: ("east", "west"), 2: ("up", "down"), 3: ("south", "north")}
DIRS = {"east": (1, 0, 0), "west": (-1, 0, 0), "up": (0, 1, 0), "down": (0, -1, 0), "south": (0, 0, 1), "north": (0, 0, -1)}


def size_of(w):
    return 4 if w < THIN else 8 if w < MID else 16


def bark(lo, hi, f):
    """A face's uv on the bark (atlas 0..32 across), as big as the face, so thin boxes show bark
    at the scale of a log's."""
    a = {"east": (2, 1), "west": (2, 1), "south": (0, 1), "north": (0, 1), "up": (0, 2), "down": (0, 2)}[f]
    u = (hi[a[0]] - lo[a[0]]) * 32
    v = (hi[a[1]] - lo[a[1]]) * 32
    u0 = (lo[a[0]] % 1) * 32
    v0 = (1 - (hi[a[1]] % 1 or 1)) * 32
    return [u0, v0, min(u0 + u, 32), min(v0 + v, 32)]


def spec(blocks, branches, leaves=True):
    """Elements for `render`: logs as cubes (hidden faces dropped), leaves as cubes, branch wood
    as a post and arms."""
    wood = {(b[0], b[1], b[2]): b for b in blocks if b[3] > 0}
    size = {k: (size_of(b[4]) if branches else 16) for k, b in wood.items()}
    els = []
    for x, y, z, p, w in blocks:
        k = (x, y, z)
        if p == 0:
            if leaves:
                els.append({"name": f"l{x}_{y}_{z}", "bone": "root", "lo": [x, y, z], "hi": [x + 1, y + 1, z + 1],
                            "faces": {f: [64, 0, 96, 32] for f in DIRS}})
            continue
        s = size[k]
        if s == 16:
            faces = {}
            for f, d in DIRS.items():
                n = (x + d[0], y + d[1], z + d[2])
                if n in wood and size[n] == 16:
                    continue
                faces[f] = [32, 0, 64, 32] if f in END[p] else [0, 0, 32, 32]
            els.append({"name": f"b{x}_{y}_{z}", "bone": "root", "lo": [x, y, z], "hi": [x + 1, y + 1, z + 1], "faces": faces})
            continue
        h = s / 32.0  # half its thickness in blocks
        c = np.array([x + 0.5, y + 0.5, z + 0.5])
        lo, hi = c - h, c + h
        els.append({"name": f"p{x}_{y}_{z}", "bone": "root", "lo": list(lo), "hi": list(hi),
                    "faces": {f: bark(lo, hi, f) for f in DIRS}})
        for f, d in DIRS.items():
            n = (x + d[0], y + d[1], z + d[2])
            if n not in wood:
                continue
            a_lo, a_hi = lo.copy(), hi.copy()
            for i in range(3):
                if d[i] > 0:
                    a_lo[i], a_hi[i] = hi[i], x + 1 if i == 0 else (y + 1 if i == 1 else z + 1)
                elif d[i] < 0:
                    a_lo[i], a_hi[i] = (x, y, z)[i], lo[i]
            els.append({"name": f"a{x}_{y}_{z}{f}", "bone": "root", "lo": list(a_lo), "hi": list(a_hi),
                        "faces": {g: bark(a_lo, a_hi, g) for g in DIRS if g != f and DIRS[g] != tuple(-v for v in d)}})
    return {"bones": [{"name": "root", "parent": None, "pivot": [0, 0, 0]}], "elements": els}


def main(src, out, sp="base:oak_log"):
    trees = json.load(open(src))[sp]
    at = atlas(sp)
    cell = (330, 380)
    font = ImageFont.load_default(18)
    small = ImageFont.load_default(14)
    cols = [("logs (today)", False, True), ("branch blocks", True, True), ("logs, leaves off", False, False), ("branch blocks, leaves off", True, False)]
    im = Image.new("RGB", (40 + len(cols) * cell[0], 50 + len(trees) * (cell[1] + 30)), (236, 233, 226))
    d = ImageDraw.Draw(im)
    d.text((14, 10), f"{SPECIES[sp][5]}: thin wood as branch blocks (4 px under {THIN} wide, 8 px under {MID}), three seeds", fill=(20, 20, 20), font=font)
    for r, t in enumerate(trees):
        lo, hi = bbox(t)
        ppb = min(20.0, 330.0 / (hi[1] - lo[1] + 1), 260.0 / (max(hi[0] - lo[0], hi[2] - lo[2]) * 1.2 + 1))
        mid = (lo + hi) / 2.0
        n = sum(1 for b in t if b[3] > 0)
        nb = [sum(1 for b in t if b[3] > 0 and size_of(b[4]) == s) for s in (4, 8, 16)]
        for c, (label, br, lv) in enumerate(cols):
            x0, y0 = 20 + c * cell[0], 45 + r * (cell[1] + 30)
            img = Image.fromarray(render(spec(t, br, lv), at, {}, 35, 20, ppb, size=cell, center=view_matrix(35, 20) @ mid))
            im.paste(img, (x0, y0 + 18), img)
            d.text((x0 + 6, y0), f"seed {r}: {label}", fill=(60, 60, 60), font=small)
        d.text((20 + 1 * cell[0] + 6, 45 + r * (cell[1] + 30) + cell[1] + 6),
               f"{n} wood: {nb[0]} at 4 px, {nb[1]} at 8 px, {nb[2]} logs", fill=(90, 90, 90), font=small)
    im.save(out)
    print(out, im.size)


if __name__ == "__main__":
    main(*sys.argv[1:])
