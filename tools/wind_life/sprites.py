#!/usr/bin/env python3
"""Wind life's pollen mote and sand grain, drawn as a pixel map (the leaves and petals are Art's, generated with
the game-art skill on 11-06: ~/Coding/chiacraft-archive/images/wind-life/2026-11-06/art/; these
maps were their first placeholders). Writes mods/base/assets/textures/particle/pollen.png and
sand_grain.png (a rounded, shaded grain: the sand block's dust drew hard squares, pt4_dust)."""
import os
from PIL import Image

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
OUT = f"{ROOT}/mods/base/assets/textures/particle"
# '#' body, '+' lit edge, '-' shade, '|' the midrib, '.' clear.
SHADES = {"#": 236, "+": 255, "-": 196, "|": 168}

LEAVES = [
    # An oval broadleaf, tip up-right, stalk down-left.
    [
        "........",
        "....++#.",
        "...+##|-",
        "..+#|#-.",
        "..#|##-.",
        ".-|#--..",
        ".|--....",
        "|.......",
    ],
    # A rounder leaf (birch, cherry).
    [
        "..+++...",
        ".+####-.",
        ".##|##-.",
        ".##|##-.",
        "..#|#-..",
        "..-|-...",
        "...|....",
        "........",
    ],
    # A long pointed leaf (willow, oak's lobes read as one).
    [
        ".......+",
        "......+#",
        ".....+|-",
        "....#|-.",
        "...#|-..",
        "..#|-...",
        ".-|-....",
        "|-......",
    ],
]
PETALS = [
    [
        ".++.",
        "+##-",
        "+##-",
        ".--.",
    ],
    [
        "..+.",
        ".+#-",
        "+##-",
        ".-..",
    ],
]
POLLEN = [
    "....",
    ".++.",
    ".+-.",
    "....",
]

SAND_GRAIN = [
    ".+#.",
    "+##-",
    "##--",
    ".--.",
]


def draw(rows, name):
    h, w = len(rows), len(rows[0])
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    for y, row in enumerate(rows):
        for x, c in enumerate(row):
            if c in SHADES:
                v = SHADES[c]
                im.putpixel((x, y), (v, v, v, 255))
    im.save(f"{OUT}/{name}.png")


def main():
    os.makedirs(OUT, exist_ok=True)
    draw(POLLEN, "pollen")
    draw(SAND_GRAIN, "sand_grain")


if __name__ == "__main__":
    main()
