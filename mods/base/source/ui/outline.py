"""Item outlines in each part's own color (Minecraft's way, a "selective outline"): image models
draw one saturated red-brown line round every item; this finds that line and recolors each of its
pixels a darker shade of the part it borders (dark red round an apple, blue-grey round iron, teal
round diamond, dark brown round wood). Works on a whole generated sheet or on one icon.

    python mods/base/source/ui/outline.py sheet.png out.png

`red_line(rgb, keep)` finds the outline (dark, saturated, red to orange-brown); `own_color(rgb,
keep)` recolors it; `share(rgb, keep)` is how much of an item's edge is still that red line (the
checker's test).
"""
import colorsys
import sys

import numpy as np
from PIL import Image
from scipy import ndimage


def hsv(rgb):
    rgb = np.asarray(rgb, float) / 255
    mx, mn = rgb.max(-1), rgb.min(-1)
    d = mx - mn + 1e-9
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    h = np.where(mx == r, ((g - b) / d) % 6, np.where(mx == g, (b - r) / d + 2, (r - g) / d + 4)) * 60
    return h, np.where(mx > 0, d / np.maximum(mx, 1e-9), 0), mx


def red_line(rgb, keep):
    """The model's outline: dark, saturated, from red to orange-brown."""
    h, s, v = hsv(rgb)
    return keep & (v < 0.36) & (s > 0.5) & ((h > 340) | (h < 32))


def own_color(rgb, keep, dark=0.38):
    """Each outline pixel the color of the nearest part it borders, at `dark` of its brightness
    and a little more saturated."""
    line = red_line(rgb, keep)
    inner = keep & ~line
    out = np.array(rgb, float)
    if not inner.any() or not line.any():
        return out
    _, (iy, ix) = ndimage.distance_transform_edt(~inner, return_indices=True)
    near = out[iy, ix]
    h, s, v = hsv(near)
    ys, xs = np.where(line)
    for y, x in zip(ys, xs):
        r, g, b = colorsys.hsv_to_rgb(h[y, x] / 360, min(1, s[y, x] * 1.1 + 0.12), v[y, x] * dark)
        out[y, x] = (r * 255, g * 255, b * 255)
    return out


def share(rgb, keep):
    """How much of the items' outer edge (a band a few pixels deep) is the red outline."""
    d = ndimage.distance_transform_edt(keep)
    band = keep & (d <= max(2, 0.01 * max(keep.shape)))
    return float(red_line(rgb, keep)[band].mean()) if band.any() else 0.0


def main():
    sys.path.insert(0, __file__.rsplit("/", 1)[0])
    import sheet as sheets
    src = np.array(Image.open(sys.argv[1]).convert("RGBA"))
    keep = ~sheets.background(src)
    out = src.copy()
    out[..., :3] = own_color(src[..., :3], keep).clip(0, 255).astype(np.uint8)
    Image.fromarray(out).save(sys.argv[2])
    print(f"red outline on the edge: {share(src[..., :3], keep):.0%} before, "
          f"{share(out[..., :3], keep):.0%} after")


if __name__ == "__main__":
    main()
