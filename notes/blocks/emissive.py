"""Emission masks (`<texture>_e.png`: red x alpha is how much a pixel glows) for "glow in the dark"
(research round 17, rec. 4; Ronald's fantasy picks, 11-05). Restraint: the gem ores and redstone, their flecks only,
dimly; plain stone never glows. The pack scales emission by darkness, so by day they look as before.

    python3 notes/blocks/emissive.py ores      # diamond_ore_e.png and redstone_ore_e.png (Ronald, 11-05: "Redstone too")
    python3 notes/blocks/emissive.py plant <rule> <tile> <out _e.png>   # a plant's mask (rules in PLANTS)
"""
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
GAME = os.path.join(HERE, "..", "..", "mods", "base", "assets", "textures", "block")
W = np.array([0.2126, 0.7152, 0.0722])

# which pixels are the gem, and how strong its glow (low, high: the darkest gem pixel to the brightest)
ORES = {
    "diamond_ore": (lambda r, g, b: (b - r > 35) & (g - r > 25), (0.30, 0.65)),
    "redstone_ore": (lambda r, g, b: (r - g > 60) & (r - b > 40), (0.25, 0.55)),
}
# plants: the pixels that glow (threads, caps, flecks), and a faint rest (the mushroom stems)
# that glows at FAINT so the shape shows in the dark
FAINT = 0.12
NONE = lambda r, g, b, a: np.zeros_like(a, bool)
PLANTS = {
    # Plants' names (notes/NIGHT_GLOW.md); the foxfire and goblins_gold mats make their own masks (mats_snap.py)
    "glow_mycena": (lambda r, g, b, a: (a > 0) & (g > 120) & (g >= r) & (g - b > 25),
                    lambda r, g, b, a: (a > 0) & (g > 120) & (abs(g - b) <= 25), (0.35, 0.80)),
    # round 1, approved by Ronald (glow_mycena_small uses glow_mycena's rule)
    "glow_lichen": (lambda r, g, b, a: (a > 0) & (b > 110) & (g > 120) & (b - r > 25),
                    lambda r, g, b, a: (a > 0) & (g > 90) & (g - r > 5), (0.40, 0.80)),
    "glow_crystal": (lambda r, g, b, a: (a > 0) & ((r + g + b) / 3 > 120) & ((b - r > 15) | ((r + g + b) / 3 > 200)), NONE, (0.15, 0.75)),
}


def mask(tile, pick, strength, plant=False, faint=None):
    a = np.asarray(Image.open(tile).convert("RGBA")).astype(float)
    r, g, b, al = a[..., 0], a[..., 1], a[..., 2], a[..., 3]
    m = pick(r, g, b, al) if plant else pick(r, g, b) & (al > 0)
    lum = a[..., :3] @ W
    lo, hi = strength
    if m.any():
        l0, l1 = lum[m].min(), lum[m].max()
        k = lo + (hi - lo) * (lum - l0) / max(1, l1 - l0)
    else:
        k = np.zeros_like(lum)
    v = np.where(m, k, 0)
    if faint is not None:
        f = faint(r, g, b, al) & ~m
        v = np.where(f, FAINT, v)
        m = m | f
    v = v * 255
    out = np.zeros(a.shape, np.uint8)
    out[..., 0] = out[..., 1] = out[..., 2] = v.clip(0, 255).astype(np.uint8)
    out[..., 3] = np.where(m, 255, 0)
    return out, m.mean()


def main():
    if sys.argv[1] == "ores":
        names = list(ORES)
        for n in names:
            pick, s = ORES[n]
            out, share = mask(os.path.join(GAME, n + ".png"), pick, s)
            Image.fromarray(out).save(os.path.join(GAME, n + "_e.png"))
            print(f"{n}_e.png: {share:.3f} of the face glows")
    elif sys.argv[1] == "plant":
        n, tile, dst = sys.argv[2], sys.argv[3], sys.argv[4]
        pick, faint, s = PLANTS[n]
        out, share = mask(tile, pick, s, plant=True, faint=faint)
        Image.fromarray(out).save(dst)
        print(f"{dst}: {share:.3f} of the square glows")


if __name__ == "__main__":
    main()
