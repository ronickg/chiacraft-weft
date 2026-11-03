"""Cuts a 2 x 2 grid of generated block textures (codex/gen_grid.sh) into four 64 px game tiles:
each square a little inside its edges (the faint lines between squares), then process.tile.

    python3 notes/texture-test/grid_cut.py codex/grid_ores.png iron_ore gold_ore diamond_ore redstone_ore
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import process  # noqa: E402

INSET = 6
# Faces drawn as one picture, not a surface that runs on: their edges are not averaged.
WHOLE = {
    "grass_side_overlay", "grass_snow_side", "iron_block", "gold_block", "diamond_block", "redstone_block", "slime_block",
    "log_oak_top", "log_birch_top", "log_spruce_top", "crafting_table_top", "crafting_table_front", "crafting_table_side",
    "furnace_front", "furnace_front_lit", "furnace_side", "furnace_top", "chest_front", "chest_side", "chest_top",
    "bookshelf", "hay_bale_side", "hay_bale_top", "amber_block", "cactus_top", "smooth_stone",
}
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "tiles")


def heal(t):
    """Opposite edges averaged: no step where the tile meets itself."""
    a = np.array(t).astype(float)
    for axis in (0, 1):
        first, last = np.take(a, 0, axis), np.take(a, -1, axis)
        mean = (first + last) / 2
        if axis == 0:
            a[0], a[-1] = (a[0] + mean) / 2, (a[-1] + mean) / 2
        else:
            a[:, 0], a[:, -1] = (a[:, 0] + mean) / 2, (a[:, -1] + mean) / 2
    return Image.fromarray(a.clip(0, 255).round().astype(np.uint8), t.mode)


def main():
    src, names = sys.argv[1], sys.argv[2:6]
    im = Image.open(src).convert("RGB")
    w, h = im.size
    os.makedirs(OUT, exist_ok=True)
    for i, name in enumerate(names):
        if name == "-":
            continue
        x, y = (i % 2) * w // 2, (i // 2) * h // 2
        part = im.crop((x + INSET, y + INSET, x + w // 2 - INSET, y + h // 2 - INSET))
        t = process.tile(part, name)
        if name not in WHOLE:
            t = heal(t)
        t.save(os.path.join(OUT, name + ".png"))
        a = np.asarray(t.convert("RGB")).astype(float)
        seam = (np.abs(a[:, 0] - a[:, -1]).mean() + np.abs(a[0] - a[-1]).mean()) / 2
        inner = (np.abs(a[:, 31] - a[:, 32]).mean() + np.abs(a[31] - a[32]).mean()) / 2
        print(f"{name}: seam {seam:.1f} vs inside {inner:.1f}{'  (check the seam)' if seam > inner * 2 + 4 else ''}")


if __name__ == "__main__":
    main()
