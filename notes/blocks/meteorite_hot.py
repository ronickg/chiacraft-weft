"""The meteorite while it cools (research round 17, addendum): an `_e` glows in the pixel's own colour, so the
cool meteorite's black cracks can't glow orange. The cool tile gets its cracks back first (the cut lost them);
meteorite_hot.png is the meteorite with its crust's cracks
(and the crust round them, a texel) turned to glowing orange, hottest in the middle of each crack, and
meteorite_hot_e.png the mask over those pixels. The cracks are found in the generated square
(gen/s_impact.png, its top-left quarter): the darkest thin lines, a texel at 32 px wherever they cross.

    ~/.venvs/blocky/bin/python notes/blocks/meteorite_hot.py   # after blocks.py install batches/sky.tsv (re-cuts the cool
                                                               # tile's cracks, rust and glints too)
"""
import os

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
GEN = os.path.join(HERE, "gen", "s_impact.png")
GAME = os.path.join(HERE, "..", "..", "mods", "base", "assets", "textures", "block")
N = 32
W = np.array([0.2126, 0.7152, 0.0722])
HOT = [np.array(c, float) for c in ((120, 30, 8), (210, 80, 20), (255, 150, 50), (255, 210, 120))]


def main():
    g = np.asarray(Image.open(GEN).convert("RGB")).astype(float)
    h = g.shape[0] // 2
    inset = g.shape[1] // 170
    sq = g[inset:h - inset, inset:h - inset]
    from scipy import ndimage
    lum = sq @ W
    # thin dark lines (the cracks), not the broad thumbprint hollows: a black top-hat
    hat = ndimage.grey_closing(lum, size=(9, 9)) - lum
    crack = hat > np.quantile(hat, 0.955)
    r, gg, b = sq[..., 0], sq[..., 1], sq[..., 2]
    rust = (r - b > 22) & (r > 70)
    glint = lum > np.quantile(lum, 0.997)
    edges = np.linspace(0, sq.shape[0], N + 1).astype(int)
    pool = lambda m: np.array([[m[edges[j]:edges[j + 1], edges[i]:edges[i + 1]].mean() for i in range(N)] for j in range(N)])
    share, rshare, gshare = pool(crack), pool(rust), pool(glint)
    core = share > 0.19
    rim = np.zeros_like(core)
    for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        rim |= np.roll(np.roll(core, dy, 0), dx, 1)
    rim &= ~core
    t = np.asarray(Image.open(os.path.join(GAME, "meteorite.png")).convert("RGB")).astype(float)
    # the cool tile gets its cracks, rust and glints back (the cut averaged them away)
    t[rshare > 0.35] = t[rshare > 0.35] * 0.4 + np.array([116, 72, 42.0]) * 0.6
    t[core] = np.array([22, 22, 26.0])
    t[(gshare > 0.10) & ~core] = np.array([196, 200, 210.0])
    Image.fromarray(t.clip(0, 255).astype(np.uint8)).save(os.path.join(GAME, "meteorite.png"))
    Image.fromarray(t.clip(0, 255).astype(np.uint8)).save(os.path.join(HERE, "tiles", "meteorite.png"))
    rng = np.random.default_rng(5)
    hot = t.copy()
    k = np.clip(share / max(share.max(), 1e-3), 0, 1)
    for j, i in zip(*np.where(core)):
        f = 0.55 + 0.45 * k[j, i] + rng.uniform(-0.1, 0.1)
        idx = min(3, int(f * 3.2))
        hot[j, i] = HOT[idx]
    hot[rim] = t[rim] * 0.75 + HOT[0] * 0.25
    e = np.zeros((N, N))
    e[core] = 0.7 + 0.3 * k[core]
    e[rim] = 0.18
    Image.fromarray(hot.clip(0, 255).astype(np.uint8)).save(os.path.join(GAME, "meteorite_hot.png"))
    v = (e * 255).astype(np.uint8)
    Image.fromarray(np.dstack([v, v, v, np.where(e > 0, 255, 0).astype(np.uint8)]), "RGBA").save(os.path.join(GAME, "meteorite_hot_e.png"))
    print(f"meteorite_hot: {core.mean():.3f} of the face in cracks, {rim.mean():.3f} rim")


if __name__ == "__main__":
    main()
