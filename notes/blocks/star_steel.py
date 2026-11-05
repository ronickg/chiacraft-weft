"""Star materials, the parts made in code (First hours, 11-05):
- star_steel_ore: its etched crosshatch (the Widmanstatten pattern, the material's signature) painted back on
  the nickel-iron masses, which the 32 px cut averaged away: thin lighter lines crossing in two directions, a
  texel wide, 5-6 texels apart, only on the metal;
- star_lantern_e: the Starheart and the lit panes glow (blue-white pixels, by brightness), the frame doesn't.

    python3 notes/blocks/star_steel.py      # after blocks.py install batches/stars.tsv
"""
import os

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
B = os.path.join(HERE, "..", "..", "mods", "base", "assets", "textures", "block")
W = np.array([0.2126, 0.7152, 0.0722])


def ore():
    p = os.path.join(B, "star_steel_ore.png")
    a = np.asarray(Image.open(p).convert("RGB")).astype(float)
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    lum = a @ W
    metal = (b - r > 8) & (lum < 100)
    # keep only solid masses (a texel with at least 3 metal neighbours)
    n = sum(np.roll(np.roll(metal, dy, 0), dx, 1) for dy in (-1, 0, 1) for dx in (-1, 0, 1)) - metal
    metal &= n >= 4
    yy, xx = np.mgrid[0:32, 0:32]
    lines = ((xx + yy) % 5 == 0) | ((xx - yy) % 6 == 0)
    m = metal & lines
    a[m] = a[m] * 0.55 + np.array([150, 160, 176.0]) * 0.45
    Image.fromarray(a.clip(0, 255).astype(np.uint8)).save(p)
    Image.fromarray(a.clip(0, 255).astype(np.uint8)).save(os.path.join(HERE, "tiles", "star_steel_ore.png"))
    print(f"star_steel_ore: {metal.mean():.2f} metal, {m.mean():.3f} etched")


def lantern():
    a = np.asarray(Image.open(os.path.join(B, "star_lantern.png")).convert("RGB")).astype(float)
    lum = a @ W
    glow = ((a[..., 2] - a[..., 0] > 25) & (lum > 120)) | (lum > 200)
    k = np.clip((lum - 120) / (245 - 120), 0, 1)
    e = np.where(glow, 0.35 + 0.65 * k, 0)
    v = (e * 255).astype(np.uint8)
    Image.fromarray(np.dstack([v, v, v, np.where(glow, 255, 0).astype(np.uint8)]), "RGBA").save(os.path.join(B, "star_lantern_e.png"))
    print(f"star_lantern_e: {glow.mean():.2f} glows")


if __name__ == "__main__":
    ore()
    lantern()
