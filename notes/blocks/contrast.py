"""Set a tile's in-tile contrast: scale its luminance spread around its mean until the p10-p90 spread is
`target` sRGB codes (reports/Daytime look targets.md (e): quiet ground 20-30, busy blocks 45-80), hue
and mean kept. Variants take the factor measured on the main tile, so they stay matched.

    python3 notes/blocks/contrast.py <target codes> <main.png> [variant.png ...]   # in place
"""
import sys

import numpy as np
from PIL import Image

W = np.array([0.2126, 0.7152, 0.0722])


def spread(a):
    s = a[..., 3] > 127
    L = a[..., :3][s] @ W
    return np.percentile(L, 90) - np.percentile(L, 10), L.mean()


def apply(a, k, mean):
    b = a.astype(float).copy()
    L = b[..., :3] @ W
    t = mean + (L - mean) * k
    b[..., :3] = np.clip(b[..., :3] * (t / np.maximum(L, 1))[..., None], 0, 255)
    return b.round().astype(np.uint8)


def factor(main, target):
    a = np.asarray(Image.open(main).convert("RGBA")).astype(float)
    sp, mean = spread(a)
    k = target / sp
    for _ in range(4):  # clipping shrinks the spread a little; nudge until it lands
        got, _ = spread(apply(a, k, mean).astype(float))
        k *= target / got
    return k


if __name__ == "__main__":
    target, files = float(sys.argv[1]), sys.argv[2:]
    k = factor(files[0], target)
    for f in files:
        im = Image.open(f)
        a = np.asarray(im.convert("RGBA")).astype(float)
        _, mean = spread(a)
        out = Image.fromarray(apply(a, k, mean), "RGBA")
        (out if im.mode == "RGBA" else out.convert(im.mode)).save(f)
        print(f"{f}: x{k:.2f}, spread {spread(np.asarray(out).astype(float))[0]:.0f}")
