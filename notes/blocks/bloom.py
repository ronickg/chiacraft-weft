"""Spring bloom for leaf tiles: small flower stamps laid on a tree's leaf tile, each tree its own flower.

A biome-tinted leaf (grey tile, tinted by the game) blooms as a pair, the engine's tinted overlay (as
grass_side): `leaves_<x>_bloom` is the untinted base (alpha: the leaf silhouette plus the flowers; the
flowers in their own colours, the leaf pixels grey and hidden by the overlay) and `leaves_<x>_bloom_overlay`
is the leaf tile with the flower pixels cut out. A leaf drawn in its own colours (untinted) blooms as one
tile, `leaves_<x>_bloom`, the flowers painted in.

Flowers wrap across the tile's edges, so the tile stays seamless. Sources go to source/blocks/tiles; the
grade (regrade.py --all) builds the game's tiles from them, the leaves group with them.

    python3 notes/blocks/bloom.py <density: a|b> <out dir>        # writes the tiles and a preview sheet
    python3 notes/blocks/bloom.py bushes <out dir>                  # the fruiting bushes' stage 2, in flower

The fruiting bushes flower at their stage 2 (art only): more flowers added to the approved drawing, kept
in mods/base/source/blocks/bloom/ (not source/blocks/tiles: the bushes take no grade). Flowers sit on the
bush's own pixels only, so the outline stays, and never on a flower already drawn.
"""
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
SRC = os.path.join(ROOT, "mods", "base", "source", "blocks", "tiles")


def hexc(h):
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


# Stamps: rows of characters, '.' empty; letters index the tree's palette (h highlight, b body, s shade, c centre).
STAMPS = {
    # Vachellia tortilis: creamy-white puffballs, round, lit from the top left
    "acacia": dict(pal={"h": "FFF9E6", "b": "F1E4BE", "s": "C9B47E"},
                   stamps=[[".h.", "hbb", ".bs"], [".hb.", "hbbb", "bbbs", ".bs."], ["hb", "bs"]],
                   weights=[5, 2, 2], tinted=True),
    # mesquite: yellow catkin spikes, hanging
    "mesquite": dict(pal={"h": "F9EC86", "b": "E3C83A", "s": "AE8A22"},
                     stamps=[["h", "b", "b", "b", "s"], ["h", "b", "b", "s"], [".h", ".b", "b.", "b.", "s."], ["h.", "b.", ".b", ".s"]],
                     weights=[3, 3, 2, 2], tinted=True),
    # creosote: small yellow five-petal flowers, a darker heart
    "creosote": dict(pal={"h": "FAE36A", "b": "EEC93A", "s": "B98E1E", "c": "C77A16"},
                     stamps=[[".h.", "hcb", ".s."], ["hb", "bs"], [".h.", "bcb", ".s."]],
                     weights=[4, 3, 2], tinted=False),
    # willow: yellow-green catkins, hanging
    "willow": dict(pal={"h": "EEF0A0", "b": "CFD26A", "s": "9EA244"},
                   stamps=[["h", "b", "b", "b", "s"], ["h", "b", "b", "s"], ["h.", "b.", ".b", ".b", ".s"]],
                   weights=[3, 3, 2], tinted=False),
}
COUNT = {"a": {"acacia": 9, "mesquite": 7, "creosote": 10, "willow": 7},
         "b": {"acacia": 15, "mesquite": 12, "creosote": 16, "willow": 12}}
SEED = {"acacia": 11, "mesquite": 23, "creosote": 37, "willow": 41}


def bloom(name, density):
    spec = STAMPS[name]
    leaf = np.asarray(Image.open(os.path.join(SRC, f"leaves_{name}.png")).convert("RGBA")).copy()
    n = leaf.shape[0]
    rng = np.random.default_rng(SEED[name])
    flower = np.zeros(leaf.shape, np.uint8)
    taken = np.zeros((n, n), bool)
    w = np.array(spec["weights"], float)
    placed, tries = 0, 0
    while placed < COUNT[density][name] and tries < 4000:
        tries += 1
        st = spec["stamps"][rng.choice(len(w), p=w / w.sum())]
        y, x = rng.integers(0, n, 2)
        cells = [((y + i) % n, (x + j) % n, ch) for i, row in enumerate(st) for j, ch in enumerate(row) if ch != "."]
        # on leaves (most of the stamp on opaque pixels) and a pixel apart from other flowers
        on = sum(leaf[cy, cx, 3] > 0 for cy, cx, _ in cells)
        if on < len(cells) * 0.75:
            continue
        near = any(taken[(cy + dy) % n, (cx + dx) % n] for cy, cx, _ in cells for dy in (-1, 0, 1) for dx in (-1, 0, 1))
        if near:
            continue
        for cy, cx, ch in cells:
            flower[cy, cx, :3] = hexc(spec["pal"][ch])
            flower[cy, cx, 3] = 255
            taken[cy, cx] = True
        placed += 1
    fm = flower[..., 3] > 0
    if spec["tinted"]:
        base = leaf.copy()
        base[fm] = flower[fm]
        overlay = leaf.copy()
        overlay[fm, 3] = 0
        return dict(bloom=base, bloom_overlay=overlay), fm
    out = leaf.copy()
    out[fm] = flower[fm]
    return dict(bloom=out), fm


FOLIAGE = np.array(hexc("77AB2F")) / 255


def shown(name, tiles, tint):
    """How the game draws it: the base, then the tinted overlay mixed over it by the overlay's alpha."""
    if "bloom_overlay" in tiles:
        b = tiles["bloom"].astype(float)
        o = tiles["bloom_overlay"].astype(float)
        a = o[..., 3:] / 255
        rgb = b[..., :3] * (1 - a) + o[..., :3] * tint * a
        return np.dstack([rgb, b[..., 3]]).clip(0, 255).astype(np.uint8)
    return tiles["bloom"]


def plain(name, tint):
    leaf = np.asarray(Image.open(os.path.join(SRC, f"leaves_{name}.png")).convert("RGBA")).astype(float)
    if STAMPS[name]["tinted"]:
        leaf[..., :3] *= tint
    return leaf.astype(np.uint8)


def sheet(out, results):
    F = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    S = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 15)
    cell, tile3 = 192, 288
    W = 40 + 3 * (cell + tile3 + 40)
    H = 70 + len(results) * (tile3 + 60)
    im = Image.new("RGB", (W, H), (42, 32, 26))
    d = ImageDraw.Draw(im)
    d.text((14, 14), "Spring bloom on leaf tiles: today | A sparse | B dense (one tile 6x, and 3x3 as in a canopy; foliage tint #77AB2F)", font=F, fill=(245, 228, 188))
    names = {"acacia": "acacia: creamy-white puffballs (tinted: base + overlay)", "mesquite": "mesquite: yellow catkin spikes (tinted: base + overlay)",
             "creosote": "creosote: small yellow flowers (own colours: one tile)", "willow": "willow: yellow-green catkins (own colours: one tile)"}
    bg = (53, 44, 36, 255)
    for r, (name, cols) in enumerate(results):
        y = 60 + r * (tile3 + 60)
        d.text((20, y), names[name], font=F, fill=(255, 190, 69))
        for c, (label, arr) in enumerate(cols):
            x = 20 + c * (cell + tile3 + 40)
            t = Image.fromarray(arr)
            g = Image.new("RGBA", (cell, cell), bg)
            g.alpha_composite(t.resize((cell, cell), Image.NEAREST))
            im.paste(g.convert("RGB"), (x, y + 30))
            big = Image.new("RGBA", (96, 96), bg)
            for i in range(3):
                for j in range(3):
                    big.alpha_composite(t, (j * 32, i * 32))
            im.paste(big.resize((tile3, tile3), Image.NEAREST).convert("RGB"), (x + cell + 8, y + 30))
            d.text((x, y + 30 + cell + 6), label, font=S, fill=(245, 228, 188))
    im.save(out)
    return out


BUSHES = {
    # bramble: pale pink-white five-petal flowers, a yellow heart (as the drawing's)
    "bramble": dict(pal={"h": "FBE6E6", "b": "E8A9A4", "s": "B87878", "c": "E2C056"}, n=6,
                    stamps=[[".h.", "hcb", ".s."], ["hb", "bs"]], weights=[3, 2]),
    # berry bush: small white flowers
    "berry": dict(pal={"h": "FFFFFF", "b": "ECEBE0", "s": "B9B8A6", "c": "DCC24E"}, n=7,
                  stamps=[[".h.", "hcb", ".s."], ["hb", "bs"], ["h"]], weights=[3, 3, 2]),
    # (cloudberry keeps its drawing: one flower to a stem, as the real plant)
    # lingonberry: pink bells
    "lingonberry": dict(pal={"h": "F6B4C2", "b": "E58CA6", "s": "B0607E"}, n=6,
                        stamps=[["h", "s"], ["hb", "bs"], [".h.", "hbb", ".s."]], weights=[3, 3, 1]),
    # crowberry: tiny purple flowers
    "crowberry": dict(pal={"h": "9C5A92", "b": "7B3A6E", "s": "56264C"}, n=8,
                      stamps=[["h"], ["hb"], ["h", "s"]], weights=[3, 2, 2]),
}
BUSH_SRC = os.path.join(ROOT, "mods", "base", "source", "blocks", "bloom")
GAME = os.path.join(ROOT, "mods", "base", "assets", "textures", "block")


def drawn_flower(px):
    """A pixel of a flower already in the drawing: not green, not bark."""
    import colorsys
    r, g, b, a = px
    if a == 0:
        return False
    h, l, s = colorsys.rgb_to_hls(r / 255, g / 255, b / 255)
    return h * 360 < 25 or h * 360 > 200 or l > 0.75


def bush_bloom(name):
    spec = BUSHES[name]
    src = os.path.join(BUSH_SRC, f"{name}_bush_stage_2.png")
    if not os.path.exists(src):  # the first run keeps the approved drawing
        os.makedirs(BUSH_SRC, exist_ok=True)
        Image.open(os.path.join(GAME, f"{name}_bush_stage_2.png")).save(src)
    tile = np.asarray(Image.open(src).convert("RGBA")).copy()
    n = tile.shape[0]
    taken = np.array([[drawn_flower(tile[y, x]) for x in range(n)] for y in range(n)])
    rng = np.random.default_rng(SEED.get(name, sum(map(ord, name))))
    w = np.array(spec["weights"], float)
    out = tile.copy()
    placed, tries = 0, 0
    while placed < spec["n"] and tries < 6000:
        tries += 1
        st = spec["stamps"][rng.choice(len(w), p=w / w.sum())]
        y, x = rng.integers(0, n, 2)
        cells = [(y + i, x + j, ch) for i, row in enumerate(st) for j, ch in enumerate(row) if ch != "."]
        if any(cy >= n or cx >= n or tile[cy, cx, 3] == 0 for cy, cx, _ in cells):
            continue
        if any(taken[min(max(cy + dy, 0), n - 1), min(max(cx + dx, 0), n - 1)] for cy, cx, _ in cells for dy in (-1, 0, 1) for dx in (-1, 0, 1)):
            continue
        for cy, cx, ch in cells:
            out[cy, cx, :3] = hexc(spec["pal"][ch])
            taken[cy, cx] = True
        placed += 1
    return tile, out, placed


def bushes(outdir):
    os.makedirs(outdir, exist_ok=True)
    F = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    S = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 15)
    cell = 224
    im = Image.new("RGB", (40 + len(BUSHES) * (2 * cell + 40), 70 + cell + 60), (42, 32, 26))
    d = ImageDraw.Draw(im)
    d.text((14, 14), "Fruiting bushes in flower (stage 2): the drawing today | with more flowers (dense, as B)", font=F, fill=(245, 228, 188))
    for i, name in enumerate(BUSHES):
        before, after, placed = bush_bloom(name)
        Image.fromarray(after).save(os.path.join(outdir, f"{name}_bush_stage_2.png"))
        x = 20 + i * (2 * cell + 40)
        for j, arr in enumerate((before, after)):
            g = Image.new("RGBA", (cell, cell), (53, 44, 36, 255))
            g.alpha_composite(Image.fromarray(arr).resize((cell, cell), Image.NEAREST))
            im.paste(g.convert("RGB"), (x + j * (cell + 6), 60))
        d.text((x, 60 + cell + 8), f"{name}: +{placed} flowers", font=S, fill=(255, 190, 69))
    out = os.path.join(outdir, "bushes_sheet.png")
    im.save(out)
    print(out)


def main(density, outdir):
    os.makedirs(outdir, exist_ok=True)
    results = []
    for name in STAMPS:
        cols = [("today", plain(name, FOLIAGE))]
        for dn in ("a", "b"):
            tiles, fm = bloom(name, dn)
            cols.append((f"{dn.upper()}: {int(fm.sum())} flower px", shown(name, tiles, FOLIAGE)))
            if dn == density:
                for k, arr in tiles.items():
                    Image.fromarray(arr).save(os.path.join(outdir, f"leaves_{name}_{k}.png"))
        results.append((name, cols))
    print(sheet(os.path.join(outdir, "bloom_sheet.png"), results))


if __name__ == "__main__":
    (bushes(sys.argv[2]) if sys.argv[1] == "bushes" else main(sys.argv[1], sys.argv[2]))
