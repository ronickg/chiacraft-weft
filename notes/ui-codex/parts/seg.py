"""Part maps cut from the approved item icons (the parts test, 2026-11-04): each opaque pixel goes to
a part by its colour family (silver head or blade, gold fittings, red-brown or wood handle), then
small islands are merged into their neighbours' part."""
import colorsys
import numpy as np
from PIL import Image

ITEM = "/Users/ronald/Coding/chiacraft-weft/mods/base/assets/textures/item/"


def family(rgb):
    r, g, b = [c / 255 for c in rgb]
    h, s, v = colorsys.rgb_to_hsv(r, g, b)
    h *= 360
    if s < 0.22 or (s < 0.32 and v > 0.55):
        return "metal"
    if 32 <= h <= 62 and s > 0.35 and v > 0.3:
        return "gold"
    return "wrap"


def load(name):
    return np.array(Image.open(ITEM + name + ".png").convert("RGBA"))


def label(a):
    m = a[..., 3] > 0
    lab = np.full(m.shape, "", object)
    for y, x in zip(*np.where(m)):
        lab[y, x] = family(a[y, x, :3])
    # smooth: a pixel whose 8 neighbours mostly disagree takes their majority (outline pixels)
    for _ in range(2):
        new = lab.copy()
        for y, x in zip(*np.where(m)):
            nb = [lab[j, i] for j in range(max(0, y - 1), min(32, y + 2)) for i in range(max(0, x - 1), min(32, x + 2))
                  if (j, i) != (y, x) and lab[j, i]]
            if nb:
                best = max(set(nb), key=nb.count)
                if nb.count(best) >= 6 and best != lab[y, x]:
                    new[y, x] = best
        lab = new
    return m, lab
