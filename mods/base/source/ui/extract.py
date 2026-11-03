"""Turns a generated UI image (one component, its states in a row; `notes/STYLE.md`) into the
game's GUI parts: finds each state on the background (transparent, flat magenta, white, or a
painted-in checkerboard), reads its pixel grid, takes its true pixels, cleans them (a palette per
state, straight edges that repeat so the frame stretches, a flat inside), and rebuilds it at the
size asked for by keeping the corners as drawn and taking or adding the difference in the middle.

    python mods/base/source/ui/extract.py image.png out/ --names slot,slot_hover,... --size 24 --corner 8

writes `<name>.png` (true pixels) and `<name>.gui` for each state, and `preview.png`: the source
beside the result, enlarged, on the game's dark brown.

Generated images draw details finer than their own pixel grid (the facets of a corner block fall
between "pixels"), which snapping loses. `--smooth P` keeps the image as drawn instead: each state
shrunk smoothly to `size` x `P` screen pixels (what it covers at GUI scale `P`), the background's
color pulled out of its edges first, its `.gui` saying it stands for `size` GUI pixels.
"""
import argparse
import os

import numpy as np
from PIL import Image


def background(im, tight=False):
    """Pixels that are background: transparent, magenta, or light and grey (white, a checkerboard).
    `tight`: only colors within a short distance of the background's own (read at the corner), for
    an asset with colors near magenta (a purple fill)."""
    rgb, a = im[..., :3].astype(float), im[..., 3]
    if tight:
        key = np.median(rgb[:8, :8].reshape(-1, 3), 0)
        return (a < 128) | (np.abs(rgb - key).sum(-1) < 45)
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    # magenta, and anything tinted with it (where compression mixed it into the outline): red and
    # blue well above green and close to each other; no color of the style looks like that
    magenta = ((r > 200) & (b > 200) & (g < 80)) | ((r - g > 50) & (b - g > 50) & (np.abs(r - b) < 70))
    lum = rgb.mean(-1)
    grey = rgb.max(-1) - rgb.min(-1) < 18
    return (a < 128) | magenta | ((lum > 190) & grey)


def boxes(fg, rows=False):
    """Each state's box: runs of columns with enough foreground, then their rows (or the other way
    round, for states stacked top to bottom)."""
    if rows:
        return [(b[1], b[0], b[3], b[2]) for b in boxes(fg.T)]
    cols = fg.sum(0) > 3
    out, start = [], None
    for x, on in enumerate(list(cols) + [False]):
        if on and start is None:
            start = x
        elif not on and start is not None:
            if x - start > 8:
                sub = fg[:, start:x]
                ys = np.where(sub.sum(1) > 3)[0]
                out.append((start, int(ys.min()), x, int(ys.max()) + 1))
            start = None
    return out


def sample(rgb, box, n, m):
    """The `n` x `m` art pixels in `box`: each the median of its cell's middle."""
    x0, y0, x1, y1 = box
    cw, ch = (x1 - x0) / n, (y1 - y0) / m
    art = np.zeros((m, n, 3))
    spread = 0.0
    for j in range(m):
        for i in range(n):
            cell = rgb[int(y0 + (j + 0.3) * ch):int(y0 + (j + 0.7) * ch) + 1, int(x0 + (i + 0.3) * cw):int(x0 + (i + 0.7) * cw) + 1].reshape(-1, 3)
            art[j, i] = np.median(cell, 0)
            spread += cell.std(0).sum()
    return art, spread / (n * m)


def grid(rgb, box, lo=12, hi=48):
    """The art pixel count that makes cells most even, preferring fewer pixels when close."""
    best = None
    for n in range(lo, hi + 1):
        _, s = sample(rgb, box, n, n)
        score = s * (1 + 0.004 * n)
        if best is None or score < best[0]:
            best = (score, n)
    return best[1]


def quantize(art, alpha, k=10):
    """A small palette per state (k-means), so cell noise becomes the drawn colors."""
    px = art[alpha].reshape(-1, 3)
    if len(px) == 0:
        return art
    rng = np.random.default_rng(1)
    centers = px[rng.choice(len(px), min(k, len(px)), replace=False)]
    for _ in range(20):
        d = ((px[:, None, :] - centers[None]) ** 2).sum(-1)
        lab = d.argmin(1)
        for c in range(len(centers)):
            if (lab == c).any():
                centers[c] = px[lab == c].mean(0)
    out = art.copy()
    d = ((art[..., None, :] - centers[None, None]) ** 2).sum(-1)
    q = centers[d.argmin(-1)]
    out[alpha] = q[alpha]
    return out


def resize_middle(art, alpha, size, corner):
    """`size` x `size` from a square `art`: corners of `corner` pixels kept, the middle rows and
    columns made even (each edge row and column the median along it) and repeated or dropped."""
    n = art.shape[0]
    c = corner

    def axis(idx_len):
        mid = list(range(c, n - c))
        want = size - 2 * c
        pick = [mid[int(i * len(mid) / want)] for i in range(want)] if mid else []
        return list(range(c)) + pick + list(range(n - c, n))

    rows, cols = axis(n), axis(n)
    # even out the straight parts: every row of the top and bottom bars is one color along its
    # middle, every column of the side bars too; the inside is its median
    a = art.copy()
    inner = alpha.copy()
    for j in list(range(c)) + list(range(n - c, n)):
        seg = a[j, c:n - c][alpha[j, c:n - c]]
        if len(seg):
            a[j, c:n - c] = np.median(seg, 0)
    for i in list(range(c)) + list(range(n - c, n)):
        seg = a[c:n - c, i][alpha[c:n - c, i]]
        if len(seg):
            a[c:n - c, i] = np.median(seg, 0)
    middle = a[c:n - c, c:n - c][alpha[c:n - c, c:n - c]]
    if len(middle):
        a[c:n - c, c:n - c] = np.median(middle, 0)
    out = a[np.ix_(rows, cols)]
    al = alpha[np.ix_(rows, cols)]
    return out, al


def defringe(rgb, keep, rounds=6):
    """Edge pixels next to the background take their colors from inside, so no magenta (or white)
    bleeds into the edge when the image is shrunk."""
    rgb, keep = rgb.copy(), keep.copy()
    for _ in range(rounds):
        acc, n = np.zeros_like(rgb), np.zeros(keep.shape)
        for dy, dx in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
            sk = np.roll(np.roll(keep, dy, 0), dx, 1)
            acc += np.roll(np.roll(rgb, dy, 0), dx, 1) * sk[..., None]
            n += sk
        grow = ~keep & (n > 0)
        rgb[grow] = acc[grow] / n[grow][:, None]
        keep |= grow
    return rgb


def smooth(src, bg, found, names, args):
    """Each state as drawn, shrunk to its on-screen size (see the module's notes)."""
    w_gui, h_gui = args.size, args.height or args.size
    side_w, side_h = w_gui * args.smooth, h_gui * args.smooth
    # one box size for all states, so they line up: the largest, centered on each
    bw = max(b[2] - b[0] for b in found)
    bh = max(b[3] - b[1] for b in found)
    results = []
    for name, (x0, y0, x1, y1) in zip(names, found):
        cx, cy = (x0 + x1) // 2, (y0 + y1) // 2
        box = (cx - bw // 2, cy - bh // 2, cx - bw // 2 + bw, cy - bh // 2 + bh)
        crop = src[box[1]:box[3], box[0]:box[2]]
        keep = ~bg[box[1]:box[3], box[0]:box[2]]
        rgb = defringe(crop[..., :3].astype(float), keep)
        alpha = keep.astype(float) * 255
        rgba = np.concatenate([rgb, alpha[..., None]], -1).clip(0, 255).astype(np.uint8)
        im = Image.fromarray(rgba, "RGBA").resize((side_w, side_h), Image.LANCZOS)
        a = np.array(im)
        a[..., 3] = np.where(a[..., 3] > 127, 255, 0)  # hard edges, as pixel art has
        Image.fromarray(a, "RGBA").save(os.path.join(args.out, name + ".png"))
        border = args.corner
        with open(os.path.join(args.out, name + ".gui"), "w") as f:
            f.write(f"scaling = nine_slice\nborder = {border}\nsize = {w_gui} {h_gui}\n")
        results.append((box, a))
    z = 4
    pv = Image.new("RGBA", (len(results) * (side_w * z + 24) + 24, side_h * z * 2 + 72), (52, 40, 36, 255))
    for k, (box, a) in enumerate(results):
        x = 24 + k * (side_w * z + 24)
        srcim = Image.fromarray(src[box[1]:box[3], box[0]:box[2]]).resize((side_w * z, side_h * z), Image.LANCZOS)
        pv.alpha_composite(srcim, (x, 24))
        pv.alpha_composite(Image.fromarray(a).resize((side_w * z, side_h * z), Image.NEAREST), (x, side_h * z + 48))
    pv.save(os.path.join(args.out, "preview.png"))
    print(f"{len(results)} states as drawn, {side_w} x {side_h} image pixels for {w_gui} x {h_gui} GUI pixels, to {args.out}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("image")
    ap.add_argument("out")
    ap.add_argument("--names", required=True, help="the states left to right, comma separated")
    ap.add_argument("--size", type=int, required=True, help="art pixels each state should be (its width)")
    ap.add_argument("--height", type=int, default=0, help="its height, if not square")
    ap.add_argument("--corner", type=int, default=8, help="art pixels of corner kept as drawn (and the 9-slice border)")
    ap.add_argument("--grid", type=int, default=0, help="art pixels across in the image, if known")
    ap.add_argument("--colors", type=int, default=16, help="colors kept per state")
    ap.add_argument("--smooth", type=int, default=0, help="keep the image as drawn, at this many image pixels a GUI pixel")
    ap.add_argument("--rows", action="store_true", help="states stacked top to bottom")
    ap.add_argument("--tight", action="store_true", help="only the background's own color is background (an asset with purple)")
    args = ap.parse_args()
    names = args.names.split(",")
    src = np.array(Image.open(args.image).convert("RGBA"))
    rgb = src[..., :3].astype(float)
    bg = background(src, args.tight)
    found = boxes(~bg, args.rows)
    if len(found) != len(names):
        raise SystemExit(f"found {len(found)} states, named {len(names)}")
    os.makedirs(args.out, exist_ok=True)
    if args.smooth:
        smooth(src, bg, found, names, args)
        return
    # one grid for all states (they are drawn alike): from the first, or as given
    n = args.grid or grid(rgb, found[0])
    results = []
    for name, box in zip(names, found):
        art, _ = sample(rgb, box, n, n)
        x0, y0, x1, y1 = box
        cw, ch = (x1 - x0) / n, (y1 - y0) / n
        alpha = np.array([[not bg[int(y0 + (j + 0.5) * ch), int(x0 + (i + 0.5) * cw)] for i in range(n)] for j in range(n)])
        art = quantize(art, alpha, args.colors)
        out, al = resize_middle(art, alpha, args.size, min(args.corner, n // 2 - 1))
        img = np.concatenate([out, al[..., None] * 255.0], -1).astype(np.uint8)
        Image.fromarray(img, "RGBA").save(os.path.join(args.out, name + ".png"))
        with open(os.path.join(args.out, name + ".gui"), "w") as f:
            f.write(f"scaling = nine_slice\nborder = {args.corner}\nsize = {args.size} {args.size}\n")
        results.append((box, img))
    # preview: the source states, then the parts at 12x, on the game's dark brown
    z = 12
    w = sum(r[1].shape[1] * z + 24 for r in results) + 24
    h = args.size * z + 48
    pv = Image.new("RGBA", (w, h * 2), (52, 40, 36, 255))
    x = 24
    for box, img in results:
        part = Image.fromarray(img).resize((img.shape[1] * z, img.shape[0] * z), Image.NEAREST)
        srcim = Image.fromarray(src[box[1]:box[3], box[0]:box[2]]).resize((img.shape[1] * z, img.shape[0] * z), Image.LANCZOS)
        pv.alpha_composite(srcim, (x, 24))
        pv.alpha_composite(part, (x, h + 24))
        x += img.shape[1] * z + 24
    pv.save(os.path.join(args.out, "preview.png"))
    print(f"{len(results)} states, read at {n} x {n} art pixels, written at {args.size} x {args.size} to {args.out}")


if __name__ == "__main__":
    main()
