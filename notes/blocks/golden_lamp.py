"""The golden lamp at 32 px (Planner, 11-04: upstream Weft renamed copper_lamp to golden_lamp, a gold
recipe, with 16 px textures). Made from our approved copper lamp: its frame's pixels take the gold block's
colours by brightness rank (the darkest copper pixel gets the darkest gold, and so on), its amber glass
stays. Like upstream's, the faces tell the block's turn: the sides get one upright gold bar (two panes),
the top and bottom a cross (four panes).

    python3 notes/blocks/golden_lamp.py      # writes notes/blocks/tiles/golden_lamp*.png
then: python3 notes/blocks/blocks.py install batches/derived.tsv (rows golden_lamp*)
"""
import os

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
TILES = os.path.join(HERE, "tiles")
W = np.array([0.2126, 0.7152, 0.0722])


def load(n):
    return np.asarray(Image.open(os.path.join(TILES, n + ".png")).convert("RGB")).astype(float)


def frame_mask(a):
    """Copper is reddish with some blue; the amber glass has almost none (b/r under ~0.1)."""
    return a[..., 2] / np.maximum(a[..., 0], 1) > 0.13


def ramp(ref, bins=48):
    """The gold block's colours sorted by brightness, averaged in bins: a clean ramp without its streaks."""
    gl = ref.reshape(-1, 3)
    gl = gl[np.argsort(gl @ W)]
    return np.array([c.mean(0) for c in np.array_split(gl, bins)])


def gold(a, mask, ref):
    """Frame pixels re-coloured by brightness rank from the gold block's own pixels."""
    gl = ramp(ref)
    out = a.copy()
    lum = a[mask] @ W
    rank = np.argsort(np.argsort(lum)) / max(1, len(lum) - 1)
    # keep the frame's spread but lean light: copper's dark rim lines stay the darkest gold
    idx = np.clip((rank ** 0.85) * (len(gl) - 1), 0, len(gl) - 1).astype(int)
    out[mask] = gl[idx]
    return out


def pane_box(mask):
    inner = ~mask
    ys, xs = np.where(inner[4:28, 4:28])
    return ys.min() + 4, ys.max() + 4, xs.min() + 4, xs.max() + 4


def bars(a, box, ref, cross):
    """An upright gold bar through the pane (and a level one for the cross): dark, light, mid, dark."""
    gl = ramp(ref)
    q = lambda f: gl[int(f * (len(gl) - 1))]
    dark, mid, light = q(0.03), q(0.55), q(0.93)
    y0, y1, x0, x1 = box
    cx, cy = (x0 + x1 + 1) // 2, (y0 + y1 + 1) // 2
    out = a.copy()
    out[y0:y1 + 1, cx - 2] = dark
    out[y0:y1 + 1, cx - 1] = light
    out[y0:y1 + 1, cx] = mid
    out[y0:y1 + 1, cx + 1] = dark
    if cross:
        for x in list(range(x0, cx - 2)) + list(range(cx + 2, x1 + 1)):
            out[cy - 2, x] = dark
            out[cy - 1, x] = light
            out[cy, x] = mid
            out[cy + 1, x] = dark
        # the crossing: light top-left, mid elsewhere, dark outline kept round it
        out[cy - 1:cy + 1, cx - 1:cx + 1] = mid
        out[cy - 1, cx - 1] = light
    return out


def main():
    ref = load("gold_block") if os.path.exists(os.path.join(TILES, "gold_block.png")) else None
    if ref is None:
        ref = np.asarray(Image.open(os.path.join(HERE, "..", "..", "mods/base/assets/textures/block/gold_block.png")).convert("RGB")).astype(float)
    off = load("copper_lamp")
    m = frame_mask(off)
    box = pane_box(m)
    for src, side, top in (("copper_lamp", "golden_lamp", "golden_lamp_top"), ("copper_lamp_lit", "golden_lamp_lit", "golden_lamp_top_lit")):
        g = gold(load(src), m, ref)
        for name, cross in ((side, False), (top, True)):
            Image.fromarray(bars(g, box, ref, cross).clip(0, 255).astype(np.uint8)).save(os.path.join(TILES, name + ".png"))
            print(name, "pane", box)


if __name__ == "__main__":
    main()
