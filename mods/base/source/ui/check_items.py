"""Checks a generated item sheet before it is cut (the master prompt's rules, `notes/ui-codex/
items_style.txt`): image generators drift from a prompt now and then, so every sheet is measured,
and one that breaks a rule is made again rather than cut.

    python mods/base/source/ui/check_items.py sheet.png --count 16 [--grid 4x4]

- the right number of items, in the asked grid (rows x columns), none touching;
- chunky pixels: the picture is a small sprite enlarged with hard square pixels, so its color
  edges fall on a regular grid (`grid_strength`: every pixel-art sheet so far 0.16-0.40, the old
  smooth, high-res tool and material references 0.06);
- a flat background: the corner color all round, nothing else on it;
- even sizes: every item fills about the same share of its cell (a stray tiny or huge one shows);
- an outline round each item, darker than its inside, in each part's own color: not the one saturated red-brown line
  image models fall back to (`outline.py`'s `share`; the approved sheets had 55-82% of their edge
  red, recolored ones 26-40%, the rest from red parts like apples).

Prints one line per rule and exits 1 if any fails. Then look at the sheet as well: whether each
item is the thing asked for is for eyes, not numbers.
"""
import argparse
import sys

import numpy as np
from PIL import Image

import extract as ex
import outline
import sheet as sheets


def grid_strength(rgb, keep):
    """How strongly the color edges repeat at one pitch (along rows and columns): a small sprite
    enlarged with square pixels puts every edge on its grid; a painting doesn't. The strength of
    the best pitch's repeat over all edge energy, and that pitch in image pixels."""
    out = []
    for ax in (1, 0):
        g = (np.abs(np.diff(rgb, axis=ax)).sum(-1) * (keep[:, 1:] if ax == 1 else keep[1:])).sum(1 - ax)
        prof = g - g.mean()
        k = np.arange(len(prof))
        best = max((abs((prof * np.exp(2j * np.pi * k / p)).sum()), p) for p in np.arange(6, 40, 0.1))
        out.append((best[0] / max(np.abs(prof).sum(), 1e-9), best[1]))
    return float(np.mean([o[0] for o in out])), float(out[0][1])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("image")
    ap.add_argument("--count", type=int, required=True)
    ap.add_argument("--grid", help="rows x columns, e.g. 4x5")
    ap.add_argument("--min-grid", type=float, default=0.12, help="grid strength (smooth paintings 0.06, pixel art 0.16+)")
    args = ap.parse_args()
    src = np.array(Image.open(args.image).convert("RGBA"))
    bg = sheets.background(src)
    fg = ~bg
    rgb = src[..., :3].astype(float)
    ok = True

    def rule(good, text):
        nonlocal ok
        ok &= bool(good)
        print(("ok    " if good else "FAIL  ") + text)

    rows = ex.boxes(fg, rows=True)
    icons = []
    per_row = []
    for (_, ry0, _, ry1) in rows:
        found = ex.boxes(fg[ry0:ry1])
        per_row.append(len(found))
        for (cx0, _, cx1, _) in found:
            ys = np.where(fg[ry0:ry1, cx0:cx1].sum(1) > 0)[0]
            icons.append((cx0, ry0 + ys.min(), cx1, ry0 + ys.max() + 1))
    rule(len(icons) == args.count, f"{len(icons)} items (asked {args.count}); per row {per_row}")
    if args.grid:
        r, c = map(int, args.grid.lower().split("x"))
        rule(per_row == [c] * r, f"grid {args.grid}")

    strength, pitch = grid_strength(rgb, fg)
    rule(strength >= args.min_grid, f"chunky pixels: edges on a {pitch:.1f} px grid, strength {strength:.2f} (want >= {args.min_grid})")

    # the background: pixels far from the key color and from every item
    key = np.median(rgb[:8, :8].reshape(-1, 3), 0)
    near = np.zeros_like(fg)
    for x0, y0, x1, y1 in icons:
        near[max(0, y0 - 6):y1 + 6, max(0, x0 - 6):x1 + 6] = True
    stray = (np.abs(rgb - key).sum(-1) > 60) & ~near
    rule(stray.mean() < 0.002, f"flat background ({stray.mean() * 100:.2f}% stray)")

    if icons:
        sizes = np.array([max(x1 - x0, y1 - y0) for x0, y0, x1, y1 in icons], float)
        spread = sizes.max() / sizes.min()
        rule(spread < 1.8, f"even sizes: largest {sizes.max():.0f} px, smallest {sizes.min():.0f} px")
        dark = []  # each item's edge darker than its inside (an outline of its own color, not black)
        for x0, y0, x1, y1 in icons:
            m = fg[y0:y1, x0:x1]
            edge = m & ~np.pad(m, 1)[2:, 1:-1] | m & ~np.pad(m, 1)[:-2, 1:-1]
            lum = rgb[y0:y1, x0:x1].mean(-1)
            inside = m & ~edge
            dark.append(np.median(lum[edge]) / max(np.median(lum[inside]), 1) if edge.any() and inside.any() else 1)
        # pale things (a feather, a fried egg) have light outlines of their own: the median item
        # must be clearly outlined, and none may have no edge at all
        rule(np.median(dark) < 0.8 and max(dark) < 0.97,
             f"outlines darker than the inside (median item's edge at {np.median(dark) * 100:.0f}% of its inside, palest {max(dark) * 100:.0f}%)")
        red = outline.share(rgb, fg)
        rule(red < 0.45, f"outlines in their parts' colors ({red * 100:.0f}% of the edge the red-brown line, want < 45%)")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
