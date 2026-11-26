"""Worldgen's rock-face stones (granite, slate, limestone, ochre and red sandstone): each face with 3
random variants, colour-matched to its first drawing, written to deliver/rocks/ in base's layout
(textures/block/<name>[_<face>].png, <dir>/1-3.png, and the random-CTM file), and a sheet of them
beside the blocks they sit next to, plus a mesa wall of stacked strata. Not installed.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/rocksheet.py
"""
import os
import random
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, HERE)
import parts  # noqa: E402
from variants import match_colour  # noqa: E402

OUT = os.path.join(HERE, "out")
BLOCK = os.path.join(ROOT, "mods", "base", "assets", "textures", "block")
ANIMALS = os.path.join(ROOT, "mods", "animals", "assets", "textures", "block")
DELIVER = os.path.join(HERE, "deliver", "rocks")
# texture name -> (first drawing, rows that must line up across neighbours, strata side)
FACES = {
    "granite": ("rock_granite", None, False),
    "granite_b": ("rock_granite_b", None, False),  # the alternative: stone-sized slabs, speckled (A read as gravel beside stone)
    "slate_top": ("rock_slate_top", None, False),
    "slate_side": ("rock_slate_side", "lines", False),
    "limestone_top": ("rock_limestone_top", None, False),
    "limestone_side": ("rock_limestone_side", "lines", False),
    "ochre_sandstone_top": ("rock_ochre_sandstone_top", None, False),
    "ochre_sandstone_side": ("rock_ochre_sandstone_side", None, True),
    "ochre_sandstone_bottom": ("rock_ochre_sandstone_bottom", None, False),
    "red_sandstone_top": (None, None, False),     # the animals mod's own, copied (it matches red sand)
    "red_sandstone_side": ("rock_red_sandstone_side", None, True),
    "red_sandstone_bottom": (None, None, False),  # the animals mod's own, copied
}
# the least contrast a face may have (brightness spread): the drawings came out flatter than today's
# stone (21); these lift them to read as rock without changing their colour
MIN_SD = {"granite": 16, "slate_top": 8, "red_sandstone_side": 10}
NO_VARIANTS = {"ochre_sandstone_bottom", "red_sandstone_bottom"}  # bottoms are rarely seen


def load(path):
    return np.array(Image.open(path).convert("RGBA"))


def lift(a, sd):
    """Stretch the brightness round its mean until its spread is at least sd; hue stays."""
    out = a.astype(float).copy()
    L = parts.lum(out[..., :3])
    if L.std() >= sd:
        return a
    k = sd / max(L.std(), 1e-6)
    target = (L - L.mean()) * k + L.mean()
    out[..., :3] *= (target / np.maximum(L, 1))[..., None]
    return out.clip(0, 255).astype(np.uint8)


def first(name):
    src, _, _ = FACES[name]
    a = load(os.path.join(ANIMALS, name + ".png")) if src is None else load(os.path.join(OUT, src + "_px.png"))
    return lift(a, MIN_SD[name]) if name in MIN_SD else a


def line_rows(a):
    """The rows that hold a level line: rows clearly darker than the tile's mean, and the row above each."""
    L = parts.lum(a[..., :3].astype(float)).mean(1)
    dark = np.nonzero(L < L.mean() - 0.6 * L.std())[0]
    rows = set(dark.tolist())
    rows |= {r - 1 for r in dark if r > 0}
    return sorted(rows)


def strata_edge(a):
    """A strata side's bottom row a little darker than the rest, so stacked layers read apart."""
    out = a.astype(float).copy()
    out[31, :, :3] *= 0.84
    return out.clip(0, 255).astype(np.uint8)


def faces():
    """{texture: [first, v1, v2, v3]}: variants colour-matched to the first, lines copied from it."""
    res = {}
    for name, (src, lines, strata) in FACES.items():
        if src and not os.path.exists(os.path.join(OUT, src + "_px.png")):
            continue
        a = first(name)
        if strata:
            a = strata_edge(a)
        tiles = [a]
        if name not in NO_VARIANTS:
            for k in (1, 2, 3):
                p = os.path.join(OUT, f"var_{name}_{k}_px.png")
                if not os.path.exists(p):
                    continue
                v = match_colour(load(p), a)
                v[..., 3] = 255
                if lines:
                    for r in line_rows(a):
                        v[r] = a[r]
                if strata:
                    v = strata_edge(v)
                tiles.append(v)
        res[name] = tiles
    return res


def deliver(res):
    for name, tiles in res.items():
        d = os.path.join(DELIVER, "textures", "block")
        os.makedirs(d, exist_ok=True)
        p = os.path.join(d, name + ".png")
        Image.fromarray(tiles[0]).save(p)
        parts.art.archive(p, "rock_" + name, "rocks")
        if len(tiles) > 1:
            os.makedirs(os.path.join(d, name), exist_ok=True)
            for k, t in enumerate(tiles[1:], 1):
                Image.fromarray(t).save(os.path.join(d, name, f"{k}.png"))
            n = len(tiles) - 1
            turn = "symmetry = all\n" if name.endswith("_top") or name in ("granite", "granite_b") else ""
            open(os.path.join(d, name + ".ctm"), "w").write(
                f"# {name} picked at random among its drawing and {n} variants, so a cliff doesn't show its tile repeating.\n"
                f"method = random\ntiles = <default> {name}/1-{n}\nweights = 4{' 2' * n}\n{turn}")


def patch(tiles, n, rnd):
    w = [4] + [2] * (len(tiles) - 1)
    return [[rnd.choices(tiles, w)[0] for _ in range(n)] for _ in range(n)]


def grid(rows, s=3):
    h, w = len(rows), len(rows[0])
    im = Image.new("RGBA", (w * 32, h * 32))
    for j, row in enumerate(rows):
        for i, t in enumerate(row):
            if t is not None:
                im.paste(Image.fromarray(t) if isinstance(t, np.ndarray) else t, (i * 32, j * 32))
    return im.resize((w * 32 * s, h * 32 * s), Image.NEAREST)


def tex(n, mod=BLOCK):
    return load(os.path.join(mod, n + ".png"))


def main():
    res = faces()
    deliver(res)
    rnd = random.Random(3)
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    W = 1500
    im = Image.new("RGB", (W, 6000), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    d.text((14, 12), "Rock-face stones for Worldgen: each face with its 3 variants, mixed as the game would, beside its neighbours",
           fill=(0xf5, 0xe4, 0xbc), font=big)
    y = 48
    stone, gravel, snow, grass_side = tex("stone"), tex("gravel"), tex("snow"), tex("grass_side")
    sand, sst, sss = tex("sand"), tex("sandstone_top"), tex("sandstone_side")
    red_sand = tex("red_sand", ANIMALS)
    for name, tiles in res.items():
        d.text((14, y), name.replace("_", " ") + f"  ({len(tiles) - 1} variants)", fill=(0xff, 0xbe, 0x45), font=big)
        y += 28
        x = 14
        for k, t in enumerate(tiles):  # the drawing and its variants, alone
            g = Image.fromarray(t).resize((96, 96), Image.NEAREST)
            im.paste(g, (x, y))
            d.text((x, y + 98), "drawing" if k == 0 else f"variant {k}", fill=(0xc2, 0xac, 0x88), font=font)
            x += 104
        x += 20
        mixed = grid(patch(tiles, 4, rnd), 1).resize((256, 256), Image.NEAREST)  # 4 x 4 at 2x
        im.paste(mixed, (x, y))
        d.text((x, y + 258), "mixed, 4 x 4", fill=(0xc2, 0xac, 0x88), font=font)
        y += 290
    # the neighbours
    def wall_of(t, n):
        if t not in res:
            return [None] * n
        return [rnd.choices(res[t], [4] + [2] * (len(res[t]) - 1))[0] for _ in range(n)]
    d.text((14, y), "Beside their neighbours (side faces)", fill=(0xff, 0xbe, 0x45), font=big)
    y += 28
    scenes = [
        ("granite B cliff by stone, gravel and grass", [[grass_side] * 2 + [None] * 4, wall_of("granite_b", 3) + [stone] * 3,
                                                       wall_of("granite_b", 4) + [stone, gravel], [gravel] + wall_of("granite_b", 5)]),
        ("granite cliff by stone, gravel and grass", [[grass_side] * 2 + [None] * 4, wall_of("granite", 3) + [stone] * 3,
                                                       wall_of("granite", 4) + [stone, gravel], [gravel] + wall_of("granite", 5)]),
        ("slate under snow", [[snow] * 6, wall_of("slate_side", 6), wall_of("slate_side", 6), [stone, stone] + wall_of("slate_side", 2) + [gravel] * 2]),
        ("limestone river cliff, grass on top, sand and water below", [[grass_side] * 6, wall_of("limestone_side", 6), wall_of("limestone_side", 6), [sand] * 6]),
    ]
    x, row_h = 14, 0
    for title, rows in scenes:
        if x + len(rows[0]) * 64 > W:
            x, y = 14, y + 4 * 64 + 40
        g = grid(rows, 2)
        im.paste(g, (x, y), g)
        d.text((x, y + g.height + 2), title, fill=(0xc2, 0xac, 0x88), font=font)
        x += g.width + 30
    y += 4 * 64 + 40
    # the mesa: layers 1-3 blocks thick, by height, as Worldgen will stack them
    d.text((14, y), "A mesa wall: sandstone, ochre, sandstone, red ... in layers 1-3 blocks thick, red sand and sand at its foot",
           fill=(0xff, 0xbe, 0x45), font=big)
    y += 28
    order, rows = ["sandstone", "ochre_sandstone_side", "sandstone", "red_sandstone_side"], []
    top = [sst] * 12
    rows.append(top)
    k = 0
    while len(rows) < 11:
        kind = order[k % 4]
        for _ in range(rnd.randint(1, 3)):
            rows.append([sss] * 12 if kind == "sandstone" else wall_of(kind, 12))
        k += 1
    rows = rows[:11] + [[red_sand] * 6 + [sand] * 6]
    g = grid(rows, 2)
    im.paste(g, (14, y))
    y += g.height + 30
    out = os.path.join(HERE, "sheets", "rock_faces.png")
    im = im.crop((0, 0, W, y))
    im.save(out)
    parts.art.archive(out, "sheet_rock_faces", "sheets")
    for name, tiles in res.items():
        L = parts.lum(tiles[0][..., :3].astype(float))
        print(f"{name}: lum {L.mean():.0f} sd {L.std():.0f}, {len(tiles) - 1} variants")
    print(out, im.size)


if __name__ == "__main__":
    main()
