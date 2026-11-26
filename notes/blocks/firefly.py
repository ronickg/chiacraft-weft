"""Firefly particle sprites (research round 17, rec. 1): hard 1-2 texel squares, no soft halo (the pack's
bloom gives the glow). 8 x 8 like the engine's particles; at `.size(0.25)` one texel is one block texel
(1/32 block). firefly_0 is the bright blink (a 2 x 2 core with four 1 px arms set as a pinwheel, since a 2 x 2 core has no middle), firefly_1 the core alone,
firefly_2 one dim texel, so `.textures([...])` can blink through them; the colour and fade come from `.color`.

    python3 notes/blocks/firefly.py
"""
import numpy as np
from PIL import Image

OUT = "mods/base/assets/textures/particle/"
CORE = (244, 255, 150, 255)   # near-white yellow-green
ARM = (184, 224, 90, 255)     # the glow's edge, yellow-green
DIM = (168, 208, 80, 255)


def main():
    frames = []
    a = np.zeros((8, 8, 4), np.uint8)
    a[3:5, 3:5] = CORE
    for y, x in ((2, 3), (2, 4), (5, 3), (5, 4), (3, 2), (4, 2), (3, 5), (4, 5)):
        a[y, x] = ARM
    # arms only on the middle of each side: a plus, one texel each way, rounded by leaving corners empty
    for y, x in ((2, 3), (5, 4), (3, 5), (4, 2)):
        a[y, x] = 0
    frames.append(a)
    b = np.zeros((8, 8, 4), np.uint8)
    b[3:5, 3:5] = CORE
    frames.append(b)
    c = np.zeros((8, 8, 4), np.uint8)
    c[3:5, 3:5] = DIM
    c[3, 3] = ARM
    frames.append(c)
    for i, f in enumerate(frames):
        Image.fromarray(f).save(OUT + f"firefly_{i}.png")
        print(OUT + f"firefly_{i}.png")


if __name__ == "__main__":
    main()
