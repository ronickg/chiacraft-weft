#!/usr/bin/env python3
"""Before and after for the tree-look batches: per species a row, the young and two grown trees
(seeds 0 and 1) at three quarters from the `before` dump, then the same from the `after` dump, at
one scale per row (`audit.py`'s textures and young).

    ~/.venvs/blocky/bin/python mods/base/source/trees/compare.py before.json after.json out.png oak birch ...
"""

import json
import sys

from PIL import Image, ImageDraw, ImageFont

from audit import SPECIES, atlas, bbox, draw, young


def row_trees(trees, sp):
    kind = SPECIES[sp][4]
    if kind is None:  # palms and flat-topped trees grow no understory young
        y = None
    elif trees.get("understory:" + sp):
        y = trees["understory:" + sp][0]
    elif kind:
        y = young(kind, 7)
    else:
        y = None
    return [y] + trees[sp][:2]


def main(before, after, out, *names):
    b, a = json.load(open(before)), json.load(open(after))
    sps = [sp for sp in SPECIES if sp.split(":")[1].replace("_log", "") in names]
    cell = (250, 330)
    font = ImageFont.load_default(18)
    small = ImageFont.load_default(14)
    W = 200 + 6 * cell[0] + 30
    H = 50 + len(sps) * (cell[1] + 30)
    im = Image.new("RGB", (W, H), (236, 233, 226))
    d = ImageDraw.Draw(im)
    d.text((200, 12), "BEFORE (dev): young, seed 0, seed 1", fill=(120, 30, 30), font=font)
    d.text((200 + 3 * cell[0] + 30, 12), "AFTER: young, seed 0, seed 1", fill=(20, 90, 30), font=font)
    for r, sp in enumerate(sps):
        at = atlas(sp)
        tb, ta = row_trees(b, sp), row_trees(a, sp)
        grown = [t for t in tb[1:] + ta[1:]]
        hmax = max(bbox(t)[1][1] - bbox(t)[0][1] for t in grown)
        wmax = max(max(bbox(t)[1][[0, 2]] - bbox(t)[0][[0, 2]]) for t in grown)
        ppb = min(16.0, 290.0 / (hmax + 1), 200.0 / (wmax * 1.2 + 1))
        y0 = 45 + r * (cell[1] + 30)
        d.text((12, y0 + cell[1] // 2), SPECIES[sp][5].replace(" (", "\n("), fill=(20, 20, 20), font=small)
        for c, t in enumerate(tb + ta):
            x0 = 190 + c * cell[0] + (30 if c >= 3 else 0)
            if t is None:
                d.text((x0 + 10, y0 + cell[1] // 2), "(no young)", fill=(120, 120, 120), font=small)
                continue
            img = draw(t, at, 35, 22, ppb, cell)
            im.paste(img, (x0, y0), img)
        d.line([(190 + 3 * cell[0] + 14, y0), (190 + 3 * cell[0] + 14, y0 + cell[1])], fill=(150, 150, 150), width=2)
    im.save(out)
    print(out, im.size)


if __name__ == "__main__":
    main(*sys.argv[1:])
