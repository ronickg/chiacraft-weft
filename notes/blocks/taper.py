"""A bracket fungus's side, tapered (Plants, 11-04: the full band read as a disc on a slab): each column
of the approved side texture squeezed into a band that is the full height in the middle and 14 of 32 rows
at the ends (a rounded curve), flat on top, its cream rim following the curved lower edge; below it transparent.

    python3 notes/blocks/taper.py notes/blocks/tiles/bracket_fungus_side_v1.png out.png
"""
import math
import sys

import numpy as np
from PIL import Image

MIN = 14  # rows at the very ends


def taper(src):
    a = np.asarray(Image.open(src).convert("RGBA"))
    n = a.shape[0]
    out = np.zeros_like(a)
    for x in range(n):
        h = round(MIN + (n - MIN) * math.sin(math.pi * (x + 0.5) / n) ** 0.5)
        rows = [min(n - 1, int((y + 0.5) * n / h)) for y in range(h)]
        out[:h, x] = a[rows, x]
        out[:h, x, 3] = 255
    return Image.fromarray(out, "RGBA")


if __name__ == "__main__":
    taper(sys.argv[1]).save(sys.argv[2])
    print(sys.argv[2])
