"""An ore's mineral lifted onto the approved stone tile (art stand-ins, 11-26), so its base is
exactly `stone.png` (the block guide's rule for ores): the generated tile's mineral pixels (by a
colour rule), grown by a pixel to keep their dark rim, pasted over the stone.

Usage: python3 ore_on_stone.py GENERATED_TILE OUT [--rule malachite]"""
import os
import sys

import numpy as np
from PIL import Image
from scipy import ndimage

HERE = os.path.dirname(os.path.abspath(__file__))
STONE = os.path.join(HERE, "..", "..", "blocks", "tiles", "stone.png")
RULES = {
    # green copper mineral: green over red and blue
    "malachite": lambda r, g, b: (g - r > 40) & (g - b > 10),
}


def main():
    src, out = sys.argv[1], sys.argv[2]
    rule = RULES[sys.argv[sys.argv.index("--rule") + 1] if "--rule" in sys.argv else "malachite"]
    a = np.asarray(Image.open(src).convert("RGB")).astype(int)
    stone = np.asarray(Image.open(STONE).convert("RGB")).copy()
    m = rule(a[..., 0], a[..., 1], a[..., 2])
    m = ndimage.binary_opening(m, iterations=1) | (m & ndimage.binary_dilation(ndimage.binary_opening(m), iterations=1))
    rim = ndimage.binary_dilation(m, iterations=1) & ~m
    stone[m] = a[m]
    # the crevice round each crust: the stone a step darker, not the generated stone
    stone[rim] = (stone[rim] * 0.62).astype(np.uint8)
    Image.fromarray(stone).save(out)


if __name__ == "__main__":
    main()
