"""Turns a generated 1024 px texture into a 64 px game tile: averaging shrink, magenta keyed to
see-through (leaves), tinted blocks to greyscale (the game tints grass and leaves per biome)."""
import sys
import numpy as np
from PIL import Image

TINTED = {"grass_top", "leaves_oak"}
CUTOUT = {"leaves_oak"}

def tile(src, name, size=64):
    im = np.asarray(Image.open(src).convert("RGB")).astype(float)
    alpha = np.ones(im.shape[:2])
    if name in CUTOUT:
        r, g, b = im[..., 0], im[..., 1], im[..., 2]
        alpha = ~((r > 170) & (b > 150) & (g < 110))
        alpha = alpha.astype(float)
    def shrink(x):
        return np.asarray(Image.fromarray(x.astype(np.float32), "F").resize((size, size), Image.BOX)).astype(float)
    a = shrink(alpha)
    rgb = np.stack([shrink(im[..., c] * alpha) for c in range(3)], -1) / np.maximum(a, 1e-6)[..., None]
    if name in TINTED:
        lum = rgb @ [0.3, 0.55, 0.15]
        lum = (lum - lum.mean()) * 1.3 + 200
        rgb = np.stack([lum] * 3, -1)
    out = np.dstack([np.clip(rgb, 0, 255), (a >= 0.5) * 255 if name in CUTOUT else np.full(a.shape, 255)])
    return Image.fromarray(out.astype(np.uint8), "RGBA" if name in CUTOUT else None).convert("RGBA" if name in CUTOUT else "RGB")

if __name__ == "__main__":
    src, name, dst = sys.argv[1:4]
    t = tile(src, name)
    t.save(dst)
    a = np.asarray(t.convert("RGB")).astype(float)
    print(name, "seam lr %.1f tb %.1f inner %.1f" % (np.abs(a[:, 0] - a[:, -1]).mean(), np.abs(a[0] - a[-1]).mean(), np.abs(a[:, 31] - a[:, 32]).mean()))
