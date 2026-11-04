"""Glacier ice sides with irregular layers (Ronald, 11-04: "Break up the bands first"; streaks and stripes
never repeat in step). The generated sides had one dark band at the same height in every variant and a
light-top, dark-bottom gradient, so a cliff striped once per block. This flattens each side's rows to
one level (the band and the gradient go), turns each variant by its own amount, and lays new dust
layers per variant: their own heights, thicknesses and gaps, lens-shaped so they fade before the tile's
edge (a neighbour never shows a cut-off stripe), and some variants with none.

    python3 notes/blocks/glacier_bands.py      # rewrites the sources, then: regrade.py --all
Originals: ~/Coding/chiacraft-archive/images/chiacraft/2026-11-04/ice/sides_before/.
"""
import os

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "..", "..", "mods", "base", "source", "blocks", "tiles")
ARCHIVE = os.path.expanduser("~/Coding/chiacraft-archive/images/chiacraft/2026-11-04/ice/sides_before")
W = np.array([0.2126, 0.7152, 0.0722])
DUST = np.array([0x7F, 0x98, 0xAC], float)
LIGHT = np.array([0xE6, 0xF0, 0xF6], float)
# per tile: the turn (rows), and layers: (centre row, thickness, from x, to x, strength; negative = lighter)
PLAN = {
    "glacier_ice_side.png": (0, [(9, 2, 3, 27, 0.30), (25, 1, 10, 22, 0.18)]),
    "1.png": (11, [(20, 3, 6, 29, -0.35)]),
    "2.png": (5, [(4, 1, 2, 14, 0.22), (15, 2, 12, 30, 0.28), (27, 1, 5, 20, -0.25)]),
    "3.png": (19, []),
}


def flatten(a):
    """Each row scaled to the tile's mean brightness: no band, no gradient, the texture kept."""
    L = a @ W
    r = L.mean(1, keepdims=True)
    return (a * (L.mean() / r)[..., None]).clip(0, 255)


def layer(a, y0, th, x0, x1, k, seed):
    rng = np.random.default_rng(seed)
    n = a.shape[0]
    wave = rng.uniform(0, 2 * np.pi)
    for x in range(x0, x1 + 1):
        t = (x - x0) / max(1, x1 - x0)
        lens = np.sin(np.pi * t) ** 0.7  # fades to nothing at both ends
        yc = y0 + 0.9 * np.sin(2 * np.pi * x / 16 + wave)
        for dy in range(-1, th + 1):
            y = int(round(yc)) + dy
            edge = 0.45 if dy in (-1, th) else 1.0  # a soft row above and below
            m = abs(k) * lens * edge
            col = DUST if k > 0 else LIGHT
            a[y % n, x] = a[y % n, x] * (1 - m) + col * m
    return a


def main():
    for name, (turn, layers) in PLAN.items():
        src = os.path.join(ARCHIVE, name)
        a = np.asarray(Image.open(src).convert("RGBA")).astype(float)
        rgb = a[..., :3].copy()
        rgb[15:22] = rgb[4:11]  # the generated dusty band (rows 16-20) out: clean ice from higher up
        rgb = np.roll(flatten(rgb), turn, axis=0)
        for i, (y0, th, x0, x1, k) in enumerate(layers):
            rgb = layer(rgb, y0, th, x0, x1, k, seed=sum(map(ord, name)) + i)
        out = np.dstack([rgb, a[..., 3]]).clip(0, 255).astype(np.uint8)
        dst = os.path.join(SRC, "glacier_ice_side.png" if name == "glacier_ice_side.png" else os.path.join("glacier_ice_side", name))
        Image.fromarray(out).save(dst)
        print(dst)


if __name__ == "__main__":
    main()
