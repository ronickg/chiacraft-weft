"""Blood-moon eye masks (research round 17, addendum; First hours; Lighting's format, 11-05): entity/<skin>_e.png
next to each monster skin, the same size. R = glow every night (0 here), G = blood-moon-only glow, alpha 255 on
the eye texels only. The shader paints G a deep ember red, so the skins keep their own eyes. One texel per eye
pixel, on the face's front, from base's 64 x 32 skins (dev):

- zombie, skeleton, creeper: the head's front (8..16, 8..16), the two eyes;
- spider: the head's front (40..48, 12..20), all eight red eyes, the front median pair strongest.

    python3 notes/blocks/blood_eyes.py
"""
import os

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
E = os.path.join(HERE, "..", "..", "mods", "base", "assets", "textures", "entity")
# skin: [(x, y, G strength 0..255)]
EYES = {
    "zombie": [(9, 11, 255), (10, 11, 255), (13, 11, 255), (14, 11, 255)],
    "skeleton": [(x, y, 255) for y in (11, 12) for x in (9, 10, 13, 14)],
    "creeper": [(x, y, 255) for y in (10, 11) for x in (9, 10, 13, 14)],
    "spider": [(43, 14, 255), (44, 14, 255),                                   # the big front median pair
               (41, 15, 200), (42, 15, 200), (45, 15, 200), (46, 15, 200),     # the side pairs
               (42, 17, 150), (45, 17, 150)],                                  # the small lower two
}


def main():
    for skin, eyes in EYES.items():
        w, h = Image.open(os.path.join(E, skin + ".png")).size
        a = np.zeros((h, w, 4), np.uint8)
        for x, y, g in eyes:
            a[y, x] = (0, g, 0, 255)
        Image.fromarray(a, "RGBA").save(os.path.join(E, skin + "_e.png"))
        print(f"{skin}_e.png: {len(eyes)} eye texels on a {w} x {h} skin")


if __name__ == "__main__":
    main()
