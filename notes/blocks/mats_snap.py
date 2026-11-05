"""The glowing mats (foxfire's threads, goblin's gold's flecks) were drawn on a finer grid than 32 px, so the
cut averaged their one-pixel threads and flecks away. This paints them back: in the generated square, the
thread or fleck pixels are found by colour, and each 32 px texel a thread crosses (a share of its pixels)
takes their mean colour, over the cut (healed) tile. Then emissive.py makes the masks.

    python3 notes/blocks/mats_snap.py      # rewrites notes/blocks/tiles/{foxfire,goblins_gold}.png
"""
import os

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
GEN = os.path.join(HERE, "gen", "g_glow2.png")
TILES = os.path.join(HERE, "tiles")
N = 32
# name: (quadrant (col, row), the feature by colour, share of a texel's pixels that makes it a feature)
MATS = {
    "foxfire": ((0, 0), lambda r, g, b: (g - r > 18) & (b - r > 4) & (g > 95), 0.12),
    "goblins_gold": ((1, 0), lambda r, g, b: (r > 140) & (g > 160) & (b < 120), 0.06),
}


def main():
    g = np.asarray(Image.open(GEN).convert("RGB")).astype(float)
    h = g.shape[0] // 2
    for name, ((qx, qy), pick, share) in MATS.items():
        sq = g[qy * h:(qy + 1) * h, qx * h:(qx + 1) * h]
        m = pick(sq[..., 0], sq[..., 1], sq[..., 2])
        tile = np.asarray(Image.open(os.path.join(TILES, name + ".png")).convert("RGB")).astype(float).copy()
        edges = np.linspace(0, h, N + 1).astype(int)
        n = 0
        for j in range(N):
            for i in range(N):
                blk = m[edges[j]:edges[j + 1], edges[i]:edges[i + 1]]
                if blk.mean() > share:
                    tile[j, i] = sq[edges[j]:edges[j + 1], edges[i]:edges[i + 1]][blk].mean(0)
                    n += 1
        Image.fromarray(tile.clip(0, 255).astype(np.uint8)).save(os.path.join(TILES, name + ".png"))
        print(f"{name}: {n} texels of threads or flecks")


if __name__ == "__main__":
    main()
