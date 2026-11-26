#!/usr/bin/env python3
"""The variety sheet (tree_recipes::Variant): per species a row, a tree in each form (standing,
big, crooked, twin) at one scale, three quarters.

    cargo run -p base_api --example tree_forms -- 3 > forms.json
    ~/.venvs/blocky/bin/python mods/base/source/trees/forms.py forms.json out.png oak birch ...
"""

import json
import sys

from PIL import Image, ImageDraw, ImageFont

from audit import SPECIES, atlas, bbox, draw

FORMS = ["Standing", "Big", "Crooked", "Twin"]


def main(src, out, *names, seed="0"):
    d = json.load(open(src))
    sps = [sp for sp in SPECIES if sp.split(":")[1].replace("_log", "") in names]
    cell = (260, 320)
    f, small = ImageFont.load_default(18), ImageFont.load_default(14)
    im = Image.new("RGB", (180 + len(FORMS) * cell[0], 40 + len(sps) * (cell[1] + 10)), (236, 233, 226))
    dr = ImageDraw.Draw(im)
    for c, name in enumerate(FORMS):
        dr.text((180 + c * cell[0] + 8, 10), name.lower(), fill=(20, 20, 20), font=f)
    for r, sp in enumerate(sps):
        trees = [d[f"{sp}|{v}"][int(seed)] for v in FORMS]
        hmax = max(bbox(t)[1][1] - bbox(t)[0][1] for t in trees)
        wmax = max(max(bbox(t)[1][[0, 2]] - bbox(t)[0][[0, 2]]) for t in trees)
        ppb = min(16.0, 290.0 / (hmax + 1), 220.0 / (wmax * 1.2 + 1))
        y0 = 40 + r * (cell[1] + 10)
        dr.text((10, y0 + cell[1] // 2), SPECIES[sp][5].replace(" (", "\n("), fill=(20, 20, 20), font=small)
        at = atlas(sp)
        for c, t in enumerate(trees):
            img = draw(t, at, 35, 18, ppb, cell)
            im.paste(img, (180 + c * cell[0], y0), img)
    im.save(out)
    print(out, im.size)


if __name__ == "__main__":
    main(*sys.argv[1:])
