"""Dried mud without the grid (Planner, 11-23): from straight above, a waterhole of base:dried_mud showed
the 32 px tile's pale and dark plates repeating block by block. This makes a 96 px crust (3 x 3 blocks)
that wraps, from the approved tile's own colours and grain, and cuts it into the 9 tiles of the CTM
`repeat` method, so plates cross the block edges and the pattern comes round only every 3 blocks.

  plates   a wrapped Voronoi of jittered points, its edges wobbled (domain warp) like the tile's cracks
  colour   each plate one of the tile's plate colours, their spread cut to two thirds (the tile's pale/dark
           alternation is what read as a grid); the tile's fine grain (high-pass, cracks filled in)
           laid over it, each 32 px block flipped differently
  rims     curled plate edges paler (the tile's #c2a97e-ish rims), most on the top-left side, crack
           pixels the tile's darkest

    mudgrid.py <dried_mud.png> <out_dir> [seed]
writes out_dir/big.png (96 px), out_dir/dried_mud/0..8.png, out_dir/dried_mud/dried_mud.ctm
"""
import os, sys
import numpy as np
from PIL import Image

N = 96
lum = lambda c: 0.3 * c[..., 0] + 0.59 * c[..., 1] + 0.11 * c[..., 2]


def blur(a, r):
    """Box blur that wraps (the texture tiles)."""
    out = a.copy()
    for ax in (0, 1):
        acc = np.zeros_like(out)
        for d in range(-r, r + 1):
            acc += np.roll(out, d, ax)
        out = acc / (2 * r + 1)
    return out


def source(path):
    a = np.array(Image.open(path).convert("RGB")).astype(float)
    l = lum(a)
    crack = l < 100
    # fill the cracks from their neighbours, so the grain has no crack lines in it
    f = a.copy()
    known = ~crack
    for _ in range(12):
        s = sum(np.roll(np.roll(f * known[..., None], dy, 0), dx, 1) for dy in (-1, 0, 1) for dx in (-1, 0, 1))
        n = sum(np.roll(np.roll(known, dy, 0), dx, 1) for dy in (-1, 0, 1) for dx in (-1, 0, 1))
        fill = (~known) & (n > 0)
        f[fill] = s[fill] / n[fill][:, None]
        known = known | fill
    base = blur(f, 4)
    grain = np.clip(f - base, -9, 9) * 0.55  # the grain only, not the tile's own rims
    # plate colours: the smoothed surface's colours, sampled over the tile
    cols = base.reshape(-1, 3)
    cl = lum(cols)
    plates = cols[(cl > np.percentile(cl, 10)) & (cl < np.percentile(cl, 90))]
    crack_col = a[l <= np.percentile(l, 6)].mean(0)
    rim_col = a[l >= np.percentile(l, 96)].mean(0)
    return a, grain, plates, crack_col, rim_col


def crust(path, seed=7):
    rng = np.random.default_rng(seed)
    a, grain, plates, crack_col, rim_col = source(path)
    # plate centres: a jittered 7 x 7 grid over 96 px (about 5 plates a block, as the tile)
    g = int(os.environ.get('MUD_G', '5'))
    pts = np.array([((i + 0.5 + rng.uniform(-0.38, 0.38)) * N / g, (j + 0.5 + rng.uniform(-0.38, 0.38)) * N / g)
                    for i in range(g) for j in range(g)])
    yy, xx = np.mgrid[0:N, 0:N].astype(float)
    # wobble: smooth wrapped noise moves where each pixel samples the cells
    def noise(r):
        return blur(rng.normal(size=(N, N)), r)
    wx, wy = noise(8), noise(8)
    wx, wy = wx / wx.std() * 2.6, wy / wy.std() * 2.6
    px, py = xx + wx, yy + wy
    d = np.full((len(pts), N, N), 1e9)
    for k, (cx, cy) in enumerate(pts):
        dx = (px - cx + N / 2) % N - N / 2
        dy = (py - cy + N / 2) % N - N / 2
        d[k] = np.hypot(dx, dy)
    order = np.argsort(d, 0)
    near, second = order[0], order[1]
    d1 = np.take_along_axis(d, order[:1], 0)[0]
    d2 = np.take_along_axis(d, order[1:2], 0)[0]
    edge = d2 - d1  # 0 on a crack
    # plate colours: the tile's, their spread cut to two thirds around the mean
    pl = plates[rng.integers(0, len(plates), len(pts))]
    m = plates.mean(0)
    pl = m + (pl - m) * 0.65
    # tidy the plates' outlines: each pixel takes its 3 x 3 neighbourhood's commonest plate, twice
    # (no one-pixel teeth or islands along the cracks)
    for _ in range(2):
        votes = np.zeros((len(pts), N, N), np.int16)
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                nb = np.roll(np.roll(near, dy, 0), dx, 1)
                np.add.at(votes, (nb, *np.mgrid[0:N, 0:N]), 1)
        votes[near, *np.mgrid[0:N, 0:N]] += 1  # ties stay
        near = votes.argmax(0)
    # plate crumbs (a piece under 12 px, cut off from its plate): given to the plate around them, so no
    # crack specks float inside a plate
    from scipy import ndimage
    big3 = np.tile(near, (3, 3))  # pieces that wrap round the edge count whole
    for _ in range(3):
        changed = False
        for k in range(len(pts)):
            lab, n = ndimage.label(big3 == k)
            for c in range(1, n + 1):
                sel = lab == c
                if sel.sum() >= 12:
                    continue
                ring = ndimage.binary_dilation(sel) & ~sel
                vals, cnt = np.unique(big3[ring], return_counts=True)
                big3[sel] = vals[cnt.argmax()]
                changed = True
        if not changed:
            break
    near = big3[N:2 * N, N:2 * N]
    img = pl[near].copy()
    for by in range(3):
        for bx in range(3):
            gr = grain
            if (bx + by) % 2:
                gr = gr.transpose(1, 0, 2)
            if (bx * 2 + by) % 3 == 1:
                gr = gr[::-1, ::-1]
            img[by * 32:(by + 1) * 32, bx * 32:(bx + 1) * 32] += gr
    # cracks: an unbroken 1 px line wherever a pixel's plate differs from the one right of or below it
    sh = lambda a, dy, dx: np.roll(np.roll(a, dy, 0), dx, 1)
    crack = (near != sh(near, 0, -1)) | (near != sh(near, -1, 0))
    # rims (light from the top left): a crack up or left of a pixel -> its curled rim, pale; two px in,
    # a little; a crack down or right of it -> in shade
    lit1 = ~crack & (sh(crack, 1, 0) | sh(crack, 0, 1))
    lit2 = ~crack & ~lit1 & (sh(crack, 2, 0) | sh(crack, 0, 2) | sh(crack, 1, 1))
    dark1 = ~crack & ~lit1 & (sh(crack, -1, 0) | sh(crack, 0, -1))
    for mask, k in ((lit1, 0.55), (lit2, 0.22)):
        img[mask] = img[mask] * (1 - k) + rim_col * k
    img[dark1] = img[dark1] * 0.86
    img[crack] = crack_col + rng.normal(0, 3, (crack.sum(), 1))
    return np.clip(img, 0, 255).astype(np.uint8)


def main():
    src, out = sys.argv[1], sys.argv[2]
    seed = int(sys.argv[3]) if len(sys.argv) > 3 else 7
    big = crust(src, seed)
    os.makedirs(os.path.join(out, "dried_mud"), exist_ok=True)
    Image.fromarray(big).save(os.path.join(out, "big.png"))
    for i in range(9):
        by, bx = divmod(i, 3)
        Image.fromarray(big[by * 32:(by + 1) * 32, bx * 32:(bx + 1) * 32]).save(os.path.join(out, "dried_mud", f"{i}.png"))
    with open(os.path.join(out, "dried_mud", "dried_mud.ctm"), "w") as f:
        f.write("# One crust over 3 x 3 blocks (plates cross the block edges), so a dried pond shows no grid.\n"
                "method = repeat\ntiles = 0-8\nwidth = 3\nheight = 3\n")
    print(out)


if __name__ == "__main__":
    main()
