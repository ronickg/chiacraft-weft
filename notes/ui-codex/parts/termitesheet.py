"""Termite mound options for wave two's Cretaceous insect eaters (animals:termite_mound, a full cube,
1 or 2 blocks tall, rare on the red dunes and the scrubland): a pale clay mound, a red laterite mound
and a fluted cathedral spire (two tries), each with a side, a top and the upper block's side (its
crown). Writes deliver/termite/<option>/textures/block/termite_mound_{side,top,upper_side}.png and
sheets/termite_mounds.png: each option flat and standing in both biomes beside the approved blocks.
Nothing installed: Ronald picks one.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/termitesheet.py
"""
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, HERE)
import iso  # noqa: E402
import parts  # noqa: E402

OUT = os.path.join(HERE, "out")
BASE = os.path.join(ROOT, "mods", "base", "assets", "textures", "block")
ANIM = os.path.join(ROOT, "mods", "animals", "assets", "textures", "block")
OPTIONS = [("clay", "A. pale clay mound (African savanna domes)"),
           ("laterite", "B. red laterite mound (knobbly red earth)"),
           ("cathedral", "C. cathedral spire, first try (reads as wood)"),
           ("cathedral_b", "C2. cathedral spire, mud buttresses")]
SCRUB = np.array([0xb5, 0xad, 0x5a]) / 255  # scrubland's grass tint


def load(n):
    p = os.path.join(OUT, n + "_px.png")
    return np.array(Image.open(p).convert("RGBA")) if os.path.exists(p) else None


def tinted(path, tint):
    a = np.array(Image.open(path).convert("RGBA")).astype(float)
    a[..., :3] *= tint
    return a.clip(0, 255).astype(np.uint8)


def main():
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    W = 1500
    im = Image.new("RGB", (W, 4000), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    d.text((14, 12), "Termite mounds for the Cretaceous insect eaters: pick one (a full cube, 1 or 2 tall; side, top, and the upper block's side)",
           fill=(0xf5, 0xe4, 0xbc), font=big)
    y = 48
    red_sand = Image.open(os.path.join(ANIM, "red_sand.png"))
    rss, rst = Image.open(os.path.join(ANIM, "red_sandstone_side.png")), Image.open(os.path.join(ANIM, "red_sandstone_top.png"))
    log, logt = Image.open(os.path.join(ANIM, "log_scale_conifer.png")), Image.open(os.path.join(ANIM, "log_scale_conifer_top.png"))
    grass = tinted(os.path.join(BASE, "grass_top.png"), SCRUB)
    dirt = Image.open(os.path.join(BASE, "dirt.png"))
    for key, title in OPTIONS:
        side, top, cap = load(f"termite_{key}_side"), load(f"termite_{key}_top"), load(f"termite_{key}_cap")
        if side is None:
            continue
        dd = os.path.join(HERE, "deliver", "termite", key, "textures", "block")
        os.makedirs(dd, exist_ok=True)
        for n, a in (("side", side), ("top", top), ("upper_side", cap)):
            p = os.path.join(dd, f"termite_mound_{n}.png")
            Image.fromarray(a).save(p)
            parts.art.archive(p, f"termite_{key}_{n}", "termite")
        d.text((14, y), title, fill=(0xff, 0xbe, 0x45), font=big)
        y += 28
        x = 14
        for n, a in (("side", side), ("upper side (crown)", cap), ("top", top)):
            t = Image.fromarray(a).resize((112, 112), Image.NEAREST)
            im.paste(t, (x, y))
            d.text((x, y + 114), n, fill=(0xc2, 0xac, 0x88), font=font)
            x += 122
        # the side and the upper side stacked, flat: does the pair read as one mound?
        st = Image.new("RGBA", (32, 64))
        st.paste(Image.fromarray(cap), (0, 0))
        st.paste(Image.fromarray(side), (0, 32))
        st = st.resize((80, 160), Image.NEAREST)
        im.paste(st, (x + 10, y), st)
        d.text((x + 10, y + 162), "stacked", fill=(0xc2, 0xac, 0x88), font=font)
        x += 120
        # the two biomes
        for ground, gside, label, extra in ((red_sand, red_sand, "red dunes", (rst, rss)), (grass, dirt, "scrubland", (logt, log))):
            s = iso.Scene()
            for gx in range(4):
                for gz in range(3):
                    s.box((gx, 0, gz), top=ground, side=gside)
            s.box((0, 1, 2), top=top, side=side)                       # a lone one-block mound
            s.box((2, 1, 1), top=top, side=side)                       # a two-block mound
            s.box((2, 2, 1), top=top, side=cap)
            s.box((3, 1, 0), top=extra[0], side=extra[1])              # an approved block beside
            img = s.render(2)
            im.paste(img, (x, y - 10), img)
            d.text((x + 10, y + img.height - 14), label, fill=(0xc2, 0xac, 0x88), font=font)
            x += img.width + 10
        y += 360
    out = os.path.join(HERE, "sheets", "termite_mounds.png")
    im = im.crop((0, 0, W, y))
    im.save(out)
    parts.art.archive(out, "sheet_termite_mounds", "sheets")
    print(out, im.size)


if __name__ == "__main__":
    main()
