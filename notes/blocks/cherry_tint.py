"""The wild cherry's leaves as tinted tiles (Planner, 11-05: autumn colour, the summer leaves less
olive): made in code from Ronald's approved tiles (`batches/cherry.tsv`, "b: separate flowers").

- `leaves_cherry`: the approved summer leaves in the game's grey (`blocks.grey`), tinted by the
  biome's foliage colour and by the season (`vitals::wild::LEAVES`: it turns russet in autumn).
- `leaves_cherry_bloom` stays the approved blossom in its own colours; `leaves_cherry_bloom_overlay`
  is its small bronze leaves alone (not pink, green well over blue), greyed: with an overlay the engine draws
  the face untinted and the overlay tinted (as the acacia's bloom), so the pink stays pink and its
  young leaves take the forest's green.

    python3 notes/blocks/cherry_tint.py
"""
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from blocks import grey  # noqa: E402

TILES = os.path.join(HERE, "tiles")
GAME = os.path.join(HERE, "..", "..", "mods", "base", "assets", "textures", "block")


def load(name):
    return np.asarray(Image.open(os.path.join(TILES, name + ".png")).convert("RGBA")).astype(float)


def save(a, name):
    Image.fromarray(np.clip(a, 0, 255).astype(np.uint8), "RGBA").save(os.path.join(GAME, name + ".png"))


leaf = load("leaves_cherry")
mask = leaf[..., 3] > 0
out = leaf.copy()
out[..., :3] = grey(leaf[..., :3], mask)
save(out, "leaves_cherry")

bloom = load("leaves_cherry_bloom")
r, g, b = bloom[..., 0], bloom[..., 1], bloom[..., 2]
green = (bloom[..., 3] > 0) & ~((r > g) & (b > g)) & (g > b * 1.4)
over = np.zeros_like(bloom)
over[..., :3] = grey(bloom[..., :3], green)
over[..., 3] = np.where(green, 255, 0)
save(over, "leaves_cherry_bloom_overlay")
save(bloom, "leaves_cherry_bloom")
print(f"leaves_cherry greyed; bloom overlay: {int(green.sum())} leaf pixels of {int((bloom[..., 3] > 0).sum())}")
