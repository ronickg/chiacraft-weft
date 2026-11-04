"""The glowing mats for Plants (notes/NIGHT_GLOW.md), built from the generated square (gen/g_glow2.png):

- The drawn threads and flecks are taken out of the square first (filled from the pixels around them), then it
  is cut to 32 px like any cube tile, so nothing of them smears into the base.
- foxfire: three long uneven runs of mycelium thread (RUNS), one texel wide, wavering and wrapping at
  the edges (the tile stays seamless). By day they are a dull pale cream, a little lighter than the wood
  (Plants: real foxfire is invisible by day); the glow is in `foxfire_e`.
- goblins_gold: the generated fleck spots, recoloured only a little lighter and more olive than the moss; the
  gold-green sheen is in `goblins_gold_e`.

An `_e` mask sets how much a pixel glows, not its colour: a glowing pixel shines in its own colour. So the
threads' cream leans faintly green-grey and the flecks olive, and their masks are strong to make up for the
dull colours. Writes the tiles and the `_e` masks.

    python3 notes/blocks/mats_snap.py      # AFTER blocks.py install (which removes variants): writes the tiles,
                                           # the game's textures, foxfire's 3 variants and .ctm, and every _e
"""
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import blocks  # noqa: E402

GEN = os.path.join(HERE, "gen", "g_glow2.png")
GAME = os.path.join(HERE, "..", "..", "mods", "base", "assets", "textures", "block")
N = 32
W = np.array([0.2126, 0.7152, 0.0722])
THREAD = lambda r, g, b: (g - r > 18) & (b - r > 4) & (g > 95)  # noqa: E731
FLECK = lambda r, g, b: (r > 140) & (g > 160) & (b < 120)  # noqa: E731


def square(g, qx, qy):
    h, w = g.shape[:2]
    inset = w // 170
    x, y = qx * w // 2, qy * h // 2
    return g[y + inset:y + h // 2 - inset, x + inset:x + w // 2 - inset].copy()


def grow(m, k):
    for _ in range(k):
        m = m | np.roll(m, 1, 0) | np.roll(m, -1, 0) | np.roll(m, 1, 1) | np.roll(m, -1, 1)
    return m


def fill(a, m):
    """Masked pixels filled from their known neighbours, ring by ring."""
    a, unknown = a.copy(), m.copy()
    while unknown.any():
        s, n = np.zeros_like(a), np.zeros(a.shape[:2])
        for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            k = ~np.roll(unknown, (dy, dx), (0, 1))
            s += np.roll(a, (dy, dx), (0, 1)) * k[..., None]
            n += k
        ring = unknown & (n > 0)
        a[ring] = s[ring] / n[ring][..., None]
        unknown &= ~ring
    return a


def cut(sq):
    return blocks.tile(sq, "cube")[..., :3]


def runs(seed):
    """Three long uneven runs of thread, one in each third of the tile, each a gently wavering one-texel line at
    its own heading and length; a run stops before it touches another, so they never knot into one shape."""
    rng = np.random.default_rng(seed)
    thread = np.zeros((N, N), bool)
    for k in rng.permutation(3):
        mine = np.zeros((N, N), bool)
        fy, fx = float(rng.integers(k * 11 + 2, k * 11 + 9)), float(rng.integers(0, N))
        head = np.radians(rng.uniform(-35, 35) + rng.choice([0, 180]))
        amp, per = rng.uniform(0.25, 0.45), rng.uniform(7, 13)
        length = int(rng.integers(20, 30))
        side = None
        for s in range(length):
            a = head + amp * np.sin(2 * np.pi * s / per)
            fy, fx = fy + np.sin(a), fx + np.cos(a)
            iy, ix = int(round(fy)) % N, int(round(fx)) % N
            near = np.roll(np.roll(thread, -iy + 1, 0), -ix + 1, 1)[:3, :3]
            if near.any():
                break
            mine[iy, ix] = True
            if side is None and s == length // 2:
                side = (fy, fx, a)
        if side and length > 16:  # one short side branch on the longer runs
            by, bx, b = side
            b += rng.choice([-1, 1]) * 1.0
            for _ in range(4):
                by, bx = by + np.sin(b), bx + np.cos(b)
                mine[int(round(by)) % N, int(round(bx)) % N] = True
        thread |= mine
    return thread


def foxfire(sq, seed):
    m = grow(THREAD(sq[..., 0], sq[..., 1], sq[..., 2]), 2)
    base = cut(fill(sq, m))
    rng = np.random.default_rng(100 + seed)
    thread = runs(seed)
    cream = np.array([140.0, 154.0, 134.0])  # dull off-white, faintly green-grey, a little lighter than the wood
    out = base.copy()
    jitter = rng.uniform(-8, 8, (N, N))[..., None]
    out[thread] = (cream + jitter[thread]) * 0.75 + base[thread] * 0.25
    e = np.where(thread, rng.uniform(0.60, 0.90, (N, N)), 0)
    return out, e


def goblins_gold(sq):
    m = FLECK(sq[..., 0], sq[..., 1], sq[..., 2])
    base = cut(fill(sq, grow(m, 2)))
    h = sq.shape[0]
    edges = np.linspace(0, h, N + 1).astype(int)
    fl = np.zeros((N, N), bool)
    for j in range(N):
        for i in range(N):
            fl[j, i] = m[edges[j]:edges[j + 1], edges[i]:edges[i + 1]].mean() > 0.06
    out = base.copy()
    out[fl] = base[fl] * 1.25 + np.array([16.0, 12.0, -2.0])  # a little lighter, toward olive
    lum = base @ W
    e = np.where(fl, 0.65 + 0.25 * (lum - lum.min()) / max(1, np.ptp(lum)), 0)
    return out, e


def save(name, rgb, e, tiles=True):
    """The tile (into notes/blocks/tiles when `tiles`) and the game's texture and `_e` mask."""
    a = np.dstack([rgb, np.full((N, N), 255.0)]).clip(0, 255).astype(np.uint8)
    os.makedirs(os.path.dirname(os.path.join(GAME, name)), exist_ok=True)
    if tiles:
        Image.fromarray(a, "RGBA").save(os.path.join(blocks.TILES, name + ".png"))
    Image.fromarray(a[..., :3], "RGB").save(os.path.join(GAME, name + ".png"))
    v = (e * 255).clip(0, 255).astype(np.uint8)
    em = np.dstack([v, v, v, np.where(e > 0, 255, 0).astype(np.uint8)])
    Image.fromarray(em, "RGBA").save(os.path.join(GAME, name + "_e.png"))
    print(f"{name}: {(e > 0).mean():.3f} of the tile glows")


def main():
    g = np.asarray(Image.open(GEN).convert("RGB")).astype(float)
    fx = square(g, 0, 0)
    save("foxfire", *foxfire(fx, 0))
    # three random variants (CTM, like stone), each with its own runs, so a floor of it doesn't repeat one shape
    for k in (1, 2, 3):
        save(f"foxfire/{k}", *foxfire(fx, k), tiles=False)
    with open(os.path.join(GAME, "foxfire.ctm"), "w") as f:
        f.write("# Four foxfire tiles, each with its own runs of thread, picked at random and turned four ways "
                "(mats_snap.py), so a floor of it doesn't show one shape repeating. Each tile has its own _e.\n"
                "method = random\ntiles = <default> foxfire/1-3\nsymmetry = all\n")
    save("goblins_gold", *goblins_gold(square(g, 1, 0)))


if __name__ == "__main__":
    main()
