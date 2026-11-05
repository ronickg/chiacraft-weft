"""Item pictures for the flat mats (Plants, 11-05: the slot showed the thin mat model): a lumpy patch cut from
the block's own tile, so the item matches the block (generated pictures came out brighter, and the threads and
flecks were lost at 32 px), with a darker front edge two texels deep so it reads as a thin mat, and a dark
outline in the block's own deepest colour, like our other items.

    python3 notes/blocks/mat_items.py      # item/foxfire.png, item/goblins_gold.png
"""
import math
import os

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
B = os.path.join(HERE, "..", "..", "mods", "base", "assets", "textures")
W = np.array([0.2126, 0.7152, 0.0722])


def blob(seed, n=32):
    rng = np.random.default_rng(seed)
    yy, xx = np.mgrid[0:n, 0:n] + 0.5
    cx, cy = n / 2, n / 2 - 1.5
    a = np.arctan2(yy - cy, xx - cx)
    r = 12.5 + sum(rng.uniform(0.4, 1.2) * np.sin(k * a + rng.uniform(0, 6.3)) for k in (2, 3, 5))
    # wider than tall: a mat seen a little from above
    d = np.hypot((xx - cx) / 1.0, (yy - cy) / 0.78)
    return d < r


def main():
    for name, seed in (("foxfire", 3), ("goblins_gold", 8)):
        t = np.asarray(Image.open(os.path.join(B, "block", name + ".png")).convert("RGB")).astype(float)
        top = blob(seed)
        # the front edge: the patch shifted down two texels, where the top doesn't cover it
        edge = (np.roll(top, 1, 0) | np.roll(top, 2, 0)) & ~top
        shape = top | edge
        out = np.zeros((32, 32, 4))
        out[top, :3] = t[top]
        out[edge, :3] = t[edge] * 0.55
        dark = t[(t @ W) <= np.quantile(t @ W, 0.04)].mean(0) * 0.75
        ring = np.zeros_like(shape)
        for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            ring |= np.roll(np.roll(shape, dy, 0), dx, 1)
        ring &= ~shape
        out[shape | ring, 3] = 255
        out[ring, :3] = dark
        Image.fromarray(out.clip(0, 255).astype(np.uint8), "RGBA").save(os.path.join(B, "item", name + ".png"))
        print(f"item/{name}.png")


if __name__ == "__main__":
    main()
