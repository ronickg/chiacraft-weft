"""The three fired-clay mould tops (art stand-ins, 11-26): the generated blank mould top
(`mould_blank_top`, a fired-clay slab with its rim) with each head's hollow carved into it, its
shape the head's own drawn loose picture (`parts/loose/<shape>/copper.png`), so the mould and
what it casts match. A hollow is shaded as a recess lit from the top left: dark along its
upper-left wall, lit along its lower-right one, its floor a step darker than the slab.

Usage: python3 mould_tops.py BLANK_TILE OUT_DIR"""
import os
import sys

import numpy as np
from PIL import Image
from scipy import ndimage

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
PARTS = os.path.join(REPO, "mods/base/assets/textures/parts/loose")
SHAPES = ["pick_head", "felling_axe_head", "shovel_blade"]


def carve(blank, shape):
    a = np.asarray(Image.open(blank).convert("RGB")).astype(float)
    px = np.asarray(Image.open(os.path.join(PARTS, shape, "copper.png")).convert("RGBA")).astype(int)
    r, g, b = px[..., 0], px[..., 1], px[..., 2]
    # the head's metal only, not its binding (gold-cream) or haft stub: copper's orange, with its dark outline
    metal = (px[..., 3] > 0) & (r - b > 70) & (r - g > 45)
    head = ndimage.binary_closing(ndimage.binary_dilation(metal) & (px[..., 3] > 0) & (r - g > 25), iterations=1)
    head = ndimage.binary_fill_holes(head)
    lab, n = ndimage.label(head)
    if n > 1:
        head = lab == 1 + np.argmax(ndimage.sum(head, lab, range(1, n + 1)))
    ys, xs = np.nonzero(head)
    head = head[ys.min():ys.max() + 1, xs.min():xs.max() + 1]
    h, w = head.shape
    m = np.zeros(a.shape[:2], bool)
    # inside the 3 px rim, centred
    room = 32 - 8
    if h > room or w > room:
        k = room / max(h, w)
        head = np.asarray(Image.fromarray(head).resize((max(1, round(w * k)), max(1, round(h * k))), Image.NEAREST))
        h, w = head.shape
    y0, x0 = (32 - h) // 2, (32 - w) // 2
    m[y0:y0 + h, x0:x0 + w] = head
    out = a.copy()
    out[m] *= 0.72  # the floor
    # walls: a hollow pixel whose up or left neighbour is slab is in the shadow of the wall above it
    up = np.zeros_like(m); up[1:] = m[:-1]
    lf = np.zeros_like(m); lf[:, 1:] = m[:, :-1]
    dn = np.zeros_like(m); dn[:-1] = m[1:]
    rt = np.zeros_like(m); rt[:, :-1] = m[:, 1:]
    shadow = m & (~up | ~lf)
    out[shadow] = a[shadow] * 0.5
    lit = m & (~dn | ~rt) & ~shadow
    out[lit] = np.minimum(255, a[lit] * 1.05)
    # the slab's edge just past the hollow's lower right catches light
    lip = ~m & ndimage.binary_dilation(m) & (np.roll(m, -1, 0) | np.roll(m, -1, 1)) & ~(np.roll(m, 1, 0) | np.roll(m, 1, 1))
    out[lip] = np.minimum(255, a[lip] * 1.12)
    return Image.fromarray(out.clip(0, 255).astype(np.uint8))


def main():
    blank, out = sys.argv[1], sys.argv[2]
    os.makedirs(out, exist_ok=True)
    for s in SHAPES:
        carve(blank, s).save(os.path.join(out, f"mould_{s}_top.png"))
        print(os.path.join(out, f"mould_{s}_top.png"))


if __name__ == "__main__":
    main()
