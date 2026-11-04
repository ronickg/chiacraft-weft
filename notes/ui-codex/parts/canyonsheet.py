"""The badland canyon's new bands (Terrain, 11-04): cream sandstone and banded clay, each face with 3
random variants colour-matched to its drawing (rocksheet.py's way), written to deliver/canyon/ in base's
layout, and a canyon-wall mock: Terrain's strata (base_api::land::STRATA) stacked 1-3 blocks a layer,
before (today's bands only) and after (with the new ones), in sun and in a mock shade.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/canyonsheet.py
"""
import os
import random
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import rocksheet as rs  # noqa: E402

rs.FACES = {
    "cream_sandstone_top": ("rock_cream_sandstone_top", None, False),
    "cream_sandstone_side": ("rock_cream_sandstone_side", None, True),
    "banded_clay_top": ("rock_banded_clay_top", None, False),
    "banded_clay_side": ("rock_banded_clay_side", "lines", True),
}
rs.MIN_SD = {}
rs.NO_VARIANTS = set()
rs.DELIVER = os.path.join(HERE, "deliver", "canyon")
# Cream came out brighter than sand (0.86): scaled to about 0.78 so it doesn't glare in a wall.
TONE = {"cream_sandstone_top": 0.9, "cream_sandstone_side": 0.92}
BLOCK = rs.BLOCK

# Terrain's order, bottom up (STRATA), repeating with height.
AFTER = ["sandstone", "ochre_sandstone", "cream_sandstone", "red_sandstone", "banded_clay", "ochre_sandstone", "cream_sandstone", "red_sandstone"]
BEFORE = [b for b in AFTER if b not in ("cream_sandstone", "banded_clay")]


def weathered(seed):
    """The karst towers' weathered limestone (Terrain): the installed limestone side and its 3 variants
    with their level lines softened and 2-4 faint rain streaks each, 1 px wide, 8-15% darker, some a
    little grey-green. Each variant has its own columns, and each streak starts and ends inside the
    tile (14-30 px, fading at both ends), so nothing lines up from block to block (Ronald, 11-04:
    "Yes, loosen streaks"; the first version, the same columns in every tile, looked corrugated).
    Made in code from the approved limestone: the generated tries had no streaks."""
    tiles = [rs.tex("limestone_side")] + [rs.load(os.path.join(BLOCK, "limestone_side", f"{k}.png")) for k in (1, 2, 3)]
    out = []
    for i, t in enumerate(tiles):
        rnd = random.Random(seed + i)
        a = t.astype(float)
        L = rs.parts.lum(a[..., :3]).mean(1)
        for r in np.nonzero(L < L.mean() - 0.6 * L.std())[0]:  # the level lines, half way to their neighbours
            a[r, :, :3] = 0.5 * a[r, :, :3] + 0.25 * (a[r - 1, :, :3] + a[(r + 1) % 32, :, :3])
        for c in rnd.sample(range(1, 31), rnd.randint(2, 4)):
            w, dark, green = 1, rnd.uniform(0.85, 0.92), rnd.random() < 0.4
            length = rnd.randint(14, 30)
            top = rnd.randint(0, 32 - length)
            for y in range(top, top + length):
                f = min(y - top + 1, top + length - y, 4) / 4  # fades in and out over 4 px
                k = 1 - (1 - dark) * f
                for dx in range(w):
                    a[y, c + dx if c + dx < 32 else c, :3] *= k
                    if green:
                        a[y, c + dx if c + dx < 32 else c, :3] *= 1 - f * np.array((0.03, 0.0, 0.06))
        out.append(a.clip(0, 255).astype(np.uint8))
    return out


def toned(res):
    for name, g in TONE.items():
        if name in res:
            res[name] = [np.dstack([(t[..., :3] * g).round().clip(0, 255).astype(np.uint8), t[..., 3:]]) for t in res[name]]
    return res


def side_tiles(res, band):
    if band + "_side" in res:
        return res[band + "_side"]
    return [rs.tex(band + "_side")] + [rs.load(os.path.join(BLOCK, band + "_side", f"{k}.png"))
                                       for k in (1, 2, 3) if os.path.exists(os.path.join(BLOCK, band + "_side", f"{k}.png"))]


def wall(res, order, rnd, w=24, h=22):
    rows, k = [], 0
    while len(rows) < h:
        tiles = side_tiles(res, order[k % len(order)])
        for _ in range(rnd.randint(1, 3)):
            rows.append([rnd.choices(tiles, [4] + [2] * (len(tiles) - 1))[0] for _ in range(w)])
        k += 1
    return rs.grid(list(reversed(rows[:h])), 1)


def shade(im, k=0.42, tint=(0.92, 0.96, 1.08)):
    a = np.asarray(im.convert("RGB")).astype(float) * k * np.array(tint)
    return Image.fromarray(a.clip(0, 255).astype(np.uint8))


def main():
    res = toned(rs.faces())
    res["weathered_limestone_side"] = weathered(11)
    rs.deliver(res)
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 15)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    W = 1640
    im = Image.new("RGB", (W, 4600), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    d.text((14, 12), "Badland canyon bands: cream sandstone and banded clay, each face with 3 variants (not installed)", fill=(0xf5, 0xe4, 0xbc), font=big)
    y, rnd = 46, random.Random(5)
    for name, tiles in res.items():
        L = rs.parts.lum(tiles[0][..., :3].astype(float)) / 255
        d.text((14, y), f"{name}  (lum {L.mean():.2f})", fill=(0xff, 0xbe, 0x45), font=big)
        y += 28
        x = 14
        for k, t in enumerate(tiles):
            im.paste(Image.fromarray(t).resize((96, 96), Image.NEAREST), (x, y))
            d.text((x, y + 98), "drawing" if k == 0 else f"variant {k}", fill=(0xc2, 0xac, 0x88), font=font)
            x += 104
        x += 20
        g = rs.grid(rs.patch(tiles, 6, rnd), 1).resize((384, 192), Image.NEAREST) if name.endswith("_side") else rs.grid(rs.patch(tiles, 4, rnd), 1).resize((192, 192), Image.NEAREST)
        im.paste(g, (x, y))
        d.text((x, y + 196), "a long band (6 wide, as tiled)" if name.endswith("_side") else "mixed 4 x 4", fill=(0xc2, 0xac, 0x88), font=font)
        y += 230
    d.text((14, y), "A canyon wall, Terrain's strata bottom up (1-3 blocks a layer): before (today's bands) | after (with cream and banded clay)",
           fill=(0xff, 0xbe, 0x45), font=big)
    y += 30
    for label, fn in (("in sun", lambda i: i), ("in shade (a mock: 42% light, slightly cool; the game's shade is Lighting's)", shade)):
        x = 14
        for order in (BEFORE, AFTER):
            g = fn(wall(res, order, random.Random(9)).resize((768, 704), Image.NEAREST))
            im.paste(g, (x, y))
            x += g.width + 24
        d.text((14, y + 708), label, fill=(0xc2, 0xac, 0x88), font=font)
        y += 740
    # A karst tower, 4 wide and 14 tall: today's limestone side | weathered, mixed as the game would
    d.text((14, y), "A karst tower, 4 x 14: limestone today | weathered limestone (rain streaks, grey-green in the wet), in sun and the mock shade",
           fill=(0xff, 0xbe, 0x45), font=big)
    y += 30
    lime = [rs.tex("limestone_side")] + [rs.load(os.path.join(BLOCK, "limestone_side", f"{k}.png")) for k in (1, 2, 3)]
    x = 14
    for tiles in (lime, res["weathered_limestone_side"]):
        g = rs.grid(rs.patch(tiles, 14, random.Random(2))[:14], 1).crop((0, 0, 4 * 32, 14 * 32)).resize((4 * 48, 14 * 48), Image.NEAREST)
        for f in (lambda i: i, shade):
            im.paste(f(g), (x, y))
            x += g.width + 16
        x += 30
    y += 14 * 48 + 30
    out = os.path.join(HERE, "sheets", "canyon_bands.png")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    im = im.crop((0, 0, W, y))
    im.save(out)
    for name, tiles in res.items():
        L = rs.parts.lum(tiles[0][..., :3].astype(float)) / 255
        print(f"{name}: lum {L.mean():.2f}, {len(tiles) - 1} variants")
    print(out, im.size)


if __name__ == "__main__":
    main()
