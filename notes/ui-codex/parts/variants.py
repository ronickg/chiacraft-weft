"""World-looks item 3 (break the tiling and the stripes): random variants of grass top, dirt, stone and
sand, a fuller grass side, and more tall-grass shapes, cut from the skill's drawings, greyed where the
game tints by biome, and shown as the game would tile them: the original alone against the original
mixed with its variants.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/variants.py   # sheets/block_variants.png, deliver/blocks/
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

BLOCK = os.path.join(ROOT, "mods", "base", "assets", "textures", "block")
OUT = os.path.join(HERE, "out")
DELIVER = os.path.join(HERE, "deliver", "blocks")
TINT = np.array([0x91, 0xBD, 0x59]) / 255  # plains grass, previews only
TINTED = {"grass_top", "tallgrass"}


def grey(rgb, mask):
    """notes/blocks/blocks.py's grey: the light grey the game multiplies by the biome colour."""
    lum = rgb @ [0.3, 0.55, 0.15]
    lum = (lum - lum[mask].mean()) * 1.3 + 185
    return np.repeat(np.clip(lum, 0, 255)[..., None], 3, -1)


def cut(name, tinted):
    a = np.array(Image.open(os.path.join(OUT, name + "_px.png")).convert("RGBA")).astype(float)
    if tinted:
        m = a[..., 3] > 0
        a[..., :3] = grey(a[..., :3], m)
    return a.clip(0, 255).astype(np.uint8)


def match_colour(v, orig):
    """Shift a variant's colours so each channel's mean and spread match the original's: neighbours
    then read as one ground, not a quilt. Only colours move; every pixel keeps its place."""
    out = v.astype(float).copy()
    o = orig.astype(float)
    for c in range(3):
        vm, vs = out[..., c].mean(), out[..., c].std() + 1e-6
        om, os_ = o[..., c].mean(), o[..., c].std()
        out[..., c] = (out[..., c] - vm) / vs * os_ + om
    return out.clip(0, 255).astype(np.uint8)


def tinted_view(a):
    v = a.astype(float).copy()
    v[..., :3] *= TINT
    return Image.fromarray(v.clip(0, 255).astype(np.uint8))


def patch(tiles, n=5, seed=1, turn=False):
    """An n x n patch of tiles picked at random (weights 4 2 2 2: the original most), turned when the
    block's .ctm allows it (symmetry all), as the game's random CTM would place them."""
    rnd = random.Random(seed)
    w = [4] + [2] * (len(tiles) - 1)
    out = Image.new("RGBA", (32 * n, 32 * n))
    for j in range(n):
        for i in range(n):
            t = rnd.choices(tiles, w)[0]
            if turn:
                t = t.rotate(90 * rnd.randrange(4))
            out.paste(t, (i * 32, j * 32))
    return out


def main():
    os.makedirs(DELIVER, exist_ok=True)
    S = 4  # patch scale on the sheet
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    W = 1400
    im = Image.new("RGB", (W, 2600), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    d.text((14, 12), "Breaking the tiling: each block alone (left) against mixed with its new variants (right)", fill=(0xf5, 0xe4, 0xbc), font=big)
    y = 48
    for name in ("grass_top", "dirt", "stone", "sand"):
        tinted = name in TINTED
        orig = np.array(Image.open(os.path.join(BLOCK, name + ".png")).convert("RGBA"))
        vars_ = [match_colour(cut(f"var_{name}_{k}", tinted), orig) for k in (1, 2, 3)]
        for k, v in enumerate(vars_, 1):
            os.makedirs(os.path.join(DELIVER, name), exist_ok=True)
            p = os.path.join(DELIVER, name, f"{k}.png")
            Image.fromarray(v).save(p)
            parts.art.archive(p, f"variant_{name}_{k}", "blocks")
        view = (lambda a: tinted_view(a)) if tinted else (lambda a: Image.fromarray(a))
        tiles = [view(orig)] + [view(v) for v in vars_]
        alone = patch([tiles[0]], turn=(name == "grass_top"))
        mixed = patch(tiles, turn=(name == "grass_top"))
        d.text((14, y), f"{name}: alone" + (" (turned four ways, as now)" if name == "grass_top" else ""), fill=(0xff, 0xbe, 0x45), font=big)
        d.text((14 + 5 * 32 * S // 2 + 40, y), f"{name}: mixed with 3 variants", fill=(0xff, 0xbe, 0x45), font=big)
        y += 28
        a2, m2 = alone.resize((alone.width * S // 2, alone.height * S // 2), Image.NEAREST), mixed.resize((mixed.width * S // 2, mixed.height * S // 2), Image.NEAREST)
        im.paste(a2, (14, y), a2)
        im.paste(m2, (14 + a2.width + 40, y), m2)
        # the three variants on their own
        x = 14 + a2.width * 2 + 80
        for k, t in enumerate(tiles):
            tt = t.resize((96, 96), Image.NEAREST)
            im.paste(tt, (x + (k % 2) * 104, y + (k // 2) * 120), tt)
            d.text((x + (k % 2) * 104, y + (k // 2) * 120 + 98), "now" if k == 0 else f"variant {k}", fill=(0xc2, 0xac, 0x88), font=font)
        y += a2.height + 30
    # the grass side: old against fuller, on a one-block step
    d.text((14, y), "Grass side on a one-block step: now (left) against the fuller lip (right)", fill=(0xff, 0xbe, 0x45), font=big)
    y += 28
    old_side = Image.open(os.path.join(BLOCK, "grass_side.png")).convert("RGBA")
    new_side = Image.fromarray(np.array(Image.open(os.path.join(OUT, "var_grass_side_full_px.png")).convert("RGBA")))
    p = os.path.join(DELIVER, "grass_side_full.png")
    new_side.save(p)
    parts.art.archive(p, "grass_side_full", "blocks")
    # its overlay: the green fringe greyed for the tint, as blocks.py makes grass_side_overlay
    ns = np.array(new_side).astype(float)
    green = (ns[..., 1] > ns[..., 0] + 8) & (ns[..., 1] > ns[..., 2] + 8)
    ov = np.dstack([grey(ns[..., :3], green), green * 255.0]).clip(0, 255).astype(np.uint8)
    Image.fromarray(ov).save(os.path.join(DELIVER, "grass_side_full_overlay.png"))
    for k, side in enumerate((old_side, new_side)):
        step = Image.new("RGBA", (32 * 6, 32 * 3), (0, 0, 0, 0))
        dirt = Image.open(os.path.join(BLOCK, "dirt.png")).convert("RGBA")
        for i in range(6):
            top = 0 if i < 3 else 1
            step.paste(side, (i * 32, top * 32))
            for j in range(top + 1, 3):
                step.paste(dirt, (i * 32, j * 32))
        st = step.resize((step.width * S // 2, step.height * S // 2), Image.NEAREST)
        im.paste(st, (14 + k * (st.width + 40), y), st)
    y += 32 * 3 * S // 2 + 30
    # tall grass shapes
    d.text((14, y), "Tall grass: now, then 3 new shapes and 2 tufts (tinted as in plains, on dirt)", fill=(0xff, 0xbe, 0x45), font=big)
    y += 28
    shapes = [np.array(Image.open(os.path.join(BLOCK, "tallgrass.png")).convert("RGBA"))] + [cut(f"var_tallgrass_{k}", True) for k in range(1, 6)]
    for k in range(1, 6):
        os.makedirs(os.path.join(DELIVER, "tallgrass"), exist_ok=True)
        p = os.path.join(DELIVER, "tallgrass", f"{k}.png")
        Image.fromarray(shapes[k]).save(p)
        parts.art.archive(p, f"tallgrass_{k}", "blocks")
    dirt = Image.open(os.path.join(BLOCK, "dirt.png")).convert("RGBA").resize((128, 128), Image.NEAREST)
    for k, s in enumerate(shapes):
        x = 14 + k * 150
        im.paste(dirt, (x, y))
        t = tinted_view(s).resize((128, 128), Image.NEAREST)
        im.paste(t, (x, y), t)
        d.text((x, y + 132), "now" if k == 0 else (f"shape {k}" if k <= 3 else f"tuft {k - 3}"), fill=(0xc2, 0xac, 0x88), font=font)
    y += 170
    out = os.path.join(HERE, "sheets", "block_variants.png")
    im = im.crop((0, 0, W, y))
    im.save(out)
    parts.art.archive(out, "sheet_block_variants", "sheets")
    print(out, im.size)


if __name__ == "__main__":
    main()
