"""The UI kit at the item pixel (the UI redo, 2026-11-04): one UI pixel is one item pixel, half a
GUI pixel, so an 18 GUI px slot is 36 pixels with a 32 px icon in it and nothing is resampled.

Parts come from `art.py ui` (the game-art skill, Ronald's UI sheets attached) as exact pixel
sprites (`NAME_px.png`); this cuts them apart, fits each to its size by nine-slice (corners kept,
plain edges repeated) and makes the states in code (hover, pressed, disabled, selected).

    ~/.venvs/blocky/bin/python mods/base/source/ui/pxkit.py parts SPRITE_px.png   # list the parts found
    ~/.venvs/blocky/bin/python mods/base/source/ui/pxkit.py build                 # every part in PARTS into px/

`px/NAME.png` is drawn at 2 image pixels a GUI pixel; `px/NAME.gui` says so (`size`), with its
nine-slice border in GUI pixels.
"""
import colorsys
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "px")


# ---------------------------------------------------------------- cutting

def load(path):
    return np.array(Image.open(path).convert("RGBA"))


def parts(a, min_px=12):
    """The opaque islands of a sprite, as (y0, x0, y1, x1), in reading order (rows by their top)."""
    m = a[..., 3] > 0
    seen = np.zeros_like(m)
    boxes = []
    H, W = m.shape
    for y in range(H):
        for x in range(W):
            if m[y, x] and not seen[y, x]:
                stack = [(y, x)]
                seen[y, x] = True
                ys, xs = [], []
                while stack:
                    cy, cx = stack.pop()
                    ys.append(cy)
                    xs.append(cx)
                    for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)):
                        ny, nx = cy + dy, cx + dx
                        if 0 <= ny < H and 0 <= nx < W and m[ny, nx] and not seen[ny, nx]:
                            seen[ny, nx] = True
                            stack.append((ny, nx))
                if len(ys) >= min_px:
                    boxes.append((min(ys), min(xs), max(ys) + 1, max(xs) + 1))
    rows = []
    for b in sorted(boxes):
        for r in rows:
            if b[0] < r[0][2] and b[2] > r[0][0]:
                r.append(b)
                break
        else:
            rows.append([b])
    return [b for r in rows for b in sorted(r, key=lambda b: b[1])]


def crop(a, box):
    y0, x0, y1, x1 = box
    return a[y0:y1, x0:x1].copy()


def fit9(a, w, h, border):
    """`a` at w x h: the corners (border pixels; one number or (l, t, r, b)) kept, the rest made by
    repeating the middle row and column, which on a plain edge is exact."""
    l, t, r, b = (border,) * 4 if np.isscalar(border) else border
    H, W = a.shape[:2]

    def axis(n, lo, hi, N):
        mid = list(range(lo, N - hi))
        core = [mid[len(mid) // 2]] * max(0, n - lo - hi)
        idx = list(range(lo)) + core + list(range(N - hi, N))
        if n < lo + hi:  # smaller than its corners: keep the outer pixels of each side
            idx = list(range(n - n // 2)) + list(range(N - n // 2, N))
        return idx

    ys, xs = axis(h, t, b, H), axis(w, l, r, W)
    return a[np.ix_(ys, xs)]


# ---------------------------------------------------------------- states

def hsv_map(a, f):
    out = a.copy()
    rgb = a[..., :3].reshape(-1, 3) / 255.0
    res = np.array([colorsys.hsv_to_rgb(*f(*colorsys.rgb_to_hsv(*p))) for p in rgb])
    out[..., :3] = np.clip(res * 255 + 0.5, 0, 255).reshape(a.shape[:2] + (3,)).astype(np.uint8)
    return out


def brighter(a, k=1.15):
    return hsv_map(a, lambda h, s, v: (h, s, min(1.0, v * k)))


def darker(a, k=0.82):
    return hsv_map(a, lambda h, s, v: (h, s, v * k))


def tone(a, to=0.072, pull=0.55, sat=0.78, val=0.84, lo=0.068, hi=0.19, min_s=0.3):
    """Ronald (2026-11-04): "too yellow everywhere". Gold and yellow are for state (hover,
    selected), never at rest: every warm pixel from orange to yellow (hue lo..hi, saturated) is
    pulled toward bronze (`to`, about #b35a26's 22-27 degrees), a little less saturated and darker.
    Reds, browns and greens are left alone."""
    def f(h, s, v):
        if lo <= h <= hi and s >= min_s:
            return (h + (to - h) * pull, s * sat, v * val)
        return (h, s, v)
    return hsv_map(a, f)


def disabled(a):
    """Grey-brown and flat: most of the color out, darker, the light kept."""
    return hsv_map(a, lambda h, s, v: (0.07, s * 0.25, v * 0.72))


def pressed(a):
    """Darker and one pixel lower: the top row of the inside goes to the outline's color."""
    d = darker(a, 0.85)
    out = d.copy()
    out[2:-1, 1:-1] = d[1:-2, 1:-1]
    return out


def glow(a, colors=((255, 221, 32), (243, 171, 31)), width=None):
    """The selected/hover rim: the outer pixels of the part (its outline first) recolored gold,
    outer to inner, keeping the shape (so it needs no outset)."""
    out = a.copy()
    m = a[..., 3] > 0
    ring = m.copy()
    for c in colors[: width or len(colors)]:
        pad = np.pad(ring, 1)
        inner = pad[2:, 1:-1] & pad[:-2, 1:-1] & pad[1:-1, 2:] & pad[1:-1, :-2]
        edge = ring & ~inner
        out[edge, :3] = c
        ring = ring & inner
    return out


# ---------------------------------------------------------------- writing

def save(name, a, border=None, scaling=None):
    os.makedirs(OUT, exist_ok=True)
    Image.fromarray(a).save(os.path.join(OUT, name + ".png"))
    h, w = a.shape[:2]
    lines = []
    if border is not None:
        b = (border,) * 4 if np.isscalar(border) else border
        g = [v / 2 for v in b]
        lines += ["scaling = nine_slice", "border = " + (fmt(g[0]) if len(set(g)) == 1 else " ".join(map(fmt, g)))]
    elif scaling:
        lines.append(f"scaling = {scaling}")
    lines.append(f"size = {fmt(w / 2)} {fmt(h / 2)}")
    open(os.path.join(OUT, name + ".gui"), "w").write("\n".join(lines) + "\n")


def fmt(v):
    return str(int(v)) if float(v).is_integer() else str(v)


def sheet(names, scale=6, bg=(26, 19, 19)):
    """Every part side by side, enlarged, for looking at."""
    ims = [Image.open(os.path.join(OUT, n + ".png")) for n in names]
    W = sum(i.width for i in ims) * scale + 8 * (len(ims) + 1)
    H = max(i.height for i in ims) * scale + 16
    s = Image.new("RGBA", (W, H), bg + (255,))
    x = 8
    for i in ims:
        s.alpha_composite(i.resize((i.width * scale, i.height * scale), Image.NEAREST), (x, 8))
        x += i.width * scale + 8
    return s


if __name__ == "__main__":
    if sys.argv[1] == "parts":
        a = load(sys.argv[2])
        for i, b in enumerate(parts(a)):
            print(i, "box", b, "size", b[3] - b[1], "x", b[2] - b[0])
