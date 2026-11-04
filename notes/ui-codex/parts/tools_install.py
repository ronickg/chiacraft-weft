"""Ronald's "Yes, replace all" (11-04, via Planner): the redrawn axes, pickaxes, shovels and hoes go
over the game's icons. Archives each installed icon first, installs the chosen design for every
material the game has, runs the no-vertical-edge check (art.py's) on what's installed, and puts the
extra materials (flint, copper, bronze, steel) beside them for crafting.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/tools_install.py
"""
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, HERE)
import parts  # noqa: E402
import toolfix as tf  # noqa: E402
from export2 import TOOLS  # noqa: E402

ITEM = os.path.join(ROOT, "mods", "base", "assets", "textures", "item")
GAME = ["wood", "stone", "iron", "gold", "diamond"]   # the materials base ships these tools in
EXTRA = ["flint", "copper", "bronze", "steel"]         # drawn for crafting, not game items yet
EXTRA_DIR = os.path.join(HERE, "deliver", "tools_extra", "item")


def main():
    bad = []
    os.makedirs(EXTRA_DIR, exist_ok=True)
    for kind, name in TOOLS.items():
        for m in GAME + EXTRA:
            a = tf.drawn(name(m))
            if a is None:
                print("missing drawing", kind, m)
                continue
            if m in GAME:
                dst = os.path.join(ITEM, f"{m}_{kind}.png")
                parts.art.archive(dst, f"replaced__{m}_{kind}", "tools_replaced")
            else:
                dst = os.path.join(EXTRA_DIR, f"{m}_{kind}.png")
            Image.fromarray(a.astype(np.uint8)).save(dst)
            parts.art.archive(dst, f"installed__{m}_{kind}", "tools_installed")
            runs = parts.art.vertical_runs(np.array(Image.open(dst).convert("RGBA")))
            if runs:
                bad.append((m, kind, runs))
    print("installed", len(GAME) * len(TOOLS), "game icons; extras in", EXTRA_DIR)
    print("vertical edges (6+ px):", bad or "none")


if __name__ == "__main__":
    main()
