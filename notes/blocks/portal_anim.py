"""The portal's 8-frame loop from one seamless 32 px tile: the tile drifting one diagonal way and a
quarter-turned copy drifting the other, 4 px a frame (8 frames make 32 px, so the loop closes),
blended, with a slight pulse. Writes a 32 x 256 strip, frames top to bottom, as `portal.anim` reads.

    python3 notes/blocks/portal_anim.py notes/blocks/tiles/portal.png out/portal.png
"""
import sys

import numpy as np
from PIL import Image

FRAMES, STEP = 8, 4


def strip(tile):
    a = np.asarray(Image.open(tile).convert("RGB")).astype(float)
    b = np.rot90(a)
    out = []
    for i in range(FRAMES):
        one = np.roll(a, (STEP * i, STEP * i), (0, 1))
        two = np.roll(b, (STEP * i, -STEP * i), (0, 1))
        pulse = 1 + 0.06 * np.sin(2 * np.pi * i / FRAMES)
        out.append(np.clip((0.62 * one + 0.38 * two) * pulse, 0, 255))
    return Image.fromarray(np.concatenate(out).round().astype(np.uint8), "RGB")


if __name__ == "__main__":
    strip(sys.argv[1]).save(sys.argv[2])
    print(sys.argv[2])
