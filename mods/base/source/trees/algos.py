#!/usr/bin/env python3
"""Two tree algorithms side by side, for Ronald (11-05: "tress alg seems a little off still"):
per species a row: a photo of the real tree (Wikimedia Commons, kept outside the repo, its licence
and author in `refs/SOURCES.md`), then `branchy` (the recipe's tree, batch 1) and space colonization
(`base_api::colonize`), each in logs and with thin branch blocks, at one scale.

    ~/.venvs/blocky/bin/python mods/base/source/trees/algos.py branchy.json colonize.json refs_dir out.png [seed]
(both dumps in `tree_widths`' format; refs_dir holds <species>.jpg)
"""

import json
import os
import sys

from PIL import Image, ImageDraw, ImageFont

from audit import SPECIES, atlas, bbox
from branches import spec
from render import render, view_matrix

ROWS = ["oak", "spruce", "birch"]


def main(branchy_src, col_src, refs, out, seed="0"):
    seed = int(seed)
    a, b = json.load(open(branchy_src)), json.load(open(col_src))
    cell = (280, 330)
    font = ImageFont.load_default(17)
    small = ImageFont.load_default(14)
    cols = ["photo (Commons)", "branchy: logs", "branchy: branch blocks", "colonization: logs", "colonization: branch blocks"]
    im = Image.new("RGB", (130 + len(cols) * cell[0], 40 + len(ROWS) * (cell[1] + 20)), (236, 233, 226))
    d = ImageDraw.Draw(im)
    for c, name in enumerate(cols):
        d.text((130 + c * cell[0] + 6, 10), name, fill=(20, 20, 20), font=font)
    for r, sp in enumerate(ROWS):
        log = f"base:{sp}_log"
        y0 = 40 + r * (cell[1] + 20)
        d.text((10, y0 + cell[1] // 2), SPECIES[log][5].replace(" (", "\n("), fill=(20, 20, 20), font=small)
        p = os.path.join(refs, f"{sp}.jpg")
        if os.path.exists(p):
            ph = Image.open(p).convert("RGB")
            ph.thumbnail((cell[0] - 10, cell[1] - 10))
            im.paste(ph, (130 + 4, y0 + (cell[1] - ph.size[1]) // 2))
        ta, tb = a[log][seed], b[log][seed]
        hmax = max(bbox(t)[1][1] - bbox(t)[0][1] for t in (ta, tb))
        wmax = max(max(bbox(t)[1][[0, 2]] - bbox(t)[0][[0, 2]]) for t in (ta, tb))
        ppb = min(18.0, 300.0 / (hmax + 1), 230.0 / (wmax * 1.2 + 1))
        at = atlas(log)
        for c, (t, br) in enumerate([(ta, False), (ta, True), (tb, False), (tb, True)]):
            lo, hi = bbox(t)
            mid = (lo + hi) / 2.0
            img = Image.fromarray(render(spec(t, br, True), at, {}, 35, 15, ppb, size=cell, center=view_matrix(35, 15) @ mid))
            im.paste(img, (130 + (c + 1) * cell[0], y0), img)
            n = sum(1 for x in t if x[3] > 0)
            d.text((130 + (c + 1) * cell[0] + 6, y0 + cell[1] - 4), f"{hi[1] - lo[1]} high, {n} wood", fill=(90, 90, 90), font=small)
    im.save(out)
    print(out, im.size)


if __name__ == "__main__":
    main(*sys.argv[1:])
