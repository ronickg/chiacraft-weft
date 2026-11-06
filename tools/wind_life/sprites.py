#!/usr/bin/env python3
"""Wind life's first particle sprites, drawn as pixel maps: three leaves, two petals, a pollen
mote. Near white, so a leaf takes its tree's colour (`wind_life_colours.rs`) and season tint.
Writes mods/base/assets/textures/particle/{leaf_0..2,petal_0..1,pollen}.png."""
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
    for i, rows in enumerate(LEAVES):
        draw(rows, f"leaf_{i}")
    for i, rows in enumerate(PETALS):
        draw(rows, f"petal_{i}")
    draw(POLLEN, "pollen")


if __name__ == "__main__":
    main()
