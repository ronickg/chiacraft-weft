#!/usr/bin/env python3
"""The animals' want bubbles (`taming.rs`: a bubble with a food in it above an animal asking for
it): the bubble frame of `want_apple.png`, its inside cleared, and the food's own item icon (or
its block's side) shrunk into it, crisp (no half-clear edges). For every taming food with no
bubble yet, or the ones named.

    ~/.venvs/blocky/bin/python bubbles.py [food ...]      (any Python with Pillow)
"""

import os
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
PARTICLES = os.path.join(REPO, "mods", "animals", "assets", "textures", "particle")
# Inside the frame (24 x 24): x 2-21, y 1-18; the icon 14 x 14, centered there.
INSIDE = (2, 1, 22, 19)
ICON = 14


def icon_of(name):
    """The food's picture: its item icon, or (a block) its block's side or face."""
    for mod in ("base", "animals"):
        for rel in (f"item/{name}.png", f"block/{name}.png", f"block/{name}_side.png"):
            p = os.path.join(REPO, "mods", mod, "assets", "textures", rel)
            if os.path.exists(p):
                return Image.open(p).convert("RGBA")
    raise SystemExit(f"no picture of {name}")


def bubble(name):
    frame = Image.open(os.path.join(PARTICLES, "want_apple.png")).convert("RGBA")
    white = frame.getpixel((12, 3))
    x0, y0, x1, y1 = INSIDE
    for y in range(y0, y1):
        for x in range(x0, x1):
            frame.putpixel((x, y), white)
    small = icon_of(name).resize((ICON, ICON), Image.BOX)
    # Crisp: a texel is the icon's or the bubble's, nothing between.
    px = small.load()
    for y in range(ICON):
        for x in range(ICON):
            r, g, b, a = px[x, y]
            px[x, y] = (r, g, b, 255) if a >= 128 else (0, 0, 0, 0)
    ox, oy = x0 + ((x1 - x0) - ICON) // 2, y0 + ((y1 - y0) - ICON) // 2
    frame.alpha_composite(small, (ox, oy))
    frame.save(os.path.join(PARTICLES, f"want_{name}.png"))
    return f"want_{name}.png"


if __name__ == "__main__":
    for n in sys.argv[1:]:
        print("wrote", bubble(n))
