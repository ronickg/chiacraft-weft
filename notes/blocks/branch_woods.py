"""Branch bark, the parts made in code (Tree looks, 11-05): after `blocks.py cut/install batches/branches.tsv`,
for each wood
- branch_<wood>: its contrast lifted toward the approved oak branch's (at most MAX_LIFT) (oak's grade row, spread 36, lifted its grain
  so a 4 px strip still reads as bark), its mean kept;
- branch_<wood>_across: branch_<wood> turned 90 degrees, for the arms;
- branch_<wood>_end: branch_<wood> with log_<wood>_top shrunk to 8 x 8 in the middle (the tip's rings), ringed
  by a 1 px band of the bark's dark.

    python3 notes/blocks/branch_woods.py birch acacia ...
"""
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
GAME = os.path.join(HERE, "..", "..", "mods", "base", "assets", "textures", "block")
TILES = os.path.join(HERE, "tiles")
W = np.array([0.2126, 0.7152, 0.0722])
MAX_LIFT = 1.7  # a fine-grained bark lifted further turns to speckle (dogwood, acacia at 2.8x)


def load(n):
    return np.asarray(Image.open(os.path.join(GAME, n + ".png")).convert("RGB")).astype(float)


def spread(a):
    return float((a @ W).std())


def lift(a, target):
    """Contrast round the mean brightness to the target spread, colours kept."""
    L = a @ W
    k = min(target / max(spread(a), 1e-3), MAX_LIFT)
    if k <= 1:
        return a
    m = L.mean()
    return (a + ((L - m) * (k - 1))[..., None]).clip(0, 255)


def save(a, n, tile=True):
    im = Image.fromarray(a.clip(0, 255).astype(np.uint8), "RGB")
    im.save(os.path.join(GAME, n + ".png"))
    if tile:
        im.save(os.path.join(TILES, n + ".png"))


def main(woods):
    target = spread(load("branch_oak"))
    for w in woods:
        b = load(f"branch_{w}")
        before = spread(b)
        b = lift(b, target)
        save(b, f"branch_{w}")
        save(np.rot90(b).copy(), f"branch_{w}_across")
        top = np.asarray(Image.open(os.path.join(GAME, f"log_{w}_top.png")).convert("RGB").resize((8, 8), Image.LANCZOS)).astype(float)
        end = b.copy()
        dark = np.quantile((b @ W), 0.08)
        ring = b[(b @ W) <= dark].mean(0)
        end[11:21, 11:21] = ring
        end[12:20, 12:20] = top
        save(end, f"branch_{w}_end")
        print(f"branch_{w}: spread {before:.1f} -> {spread(b):.1f} (oak {target:.1f}); _across, _end")


if __name__ == "__main__":
    main(sys.argv[1:])
