"""Crafting's fallback icons: the icon a part (or a tool kind) shows when nothing more exact is drawn.
Writes deliver/fallback/item/part_<shape>.png and item/<kind>.png (the crafting hook's names) and a
sheet of them all at 4x and slot size. The axe, pickaxe, shovel, hoe and sword kind icons are the
chosen iron designs; knife and spear are new.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/fallbacksheet.py   # sheets/fallback_icons.png
"""
import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import parts  # noqa: E402
import toolfix as tf  # noqa: E402

OUT = os.path.join(HERE, "out")
DELIVER = os.path.join(HERE, "deliver", "fallback", "item")
KINDS = {"axe": "axe_C_felling_1", "pickaxe": "pick2_iron_pickaxe", "shovel": "fix_iron_shovel",
         "hoe": "hoeL_iron_1", "sword": "iron_sword", "knife": "fb_knife", "spear": "fb_spear"}


def load(name):
    return Image.open(os.path.join(OUT, name + "_px.png")).convert("RGBA")


def main():
    os.makedirs(DELIVER, exist_ok=True)
    icons = []
    for j in json.load(open(os.path.join(HERE, "fallback_jobs.json"))):
        if j["name"].startswith("fb_part_"):
            icons.append(("part_" + j["name"][8:], load(j["name"])))
    icons += [(k, Image.fromarray(tf.drawn(v).astype(np.uint8))) for k, v in KINDS.items()]
    for n, im in icons:
        p = os.path.join(DELIVER, n + ".png")
        im.save(p)
        parts.art.archive(p, "fallback_" + n, "fallback")
    S, pad, cols = 128, 16, 7
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 13)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    rows = (len(icons) + cols - 1) // cols
    W = pad + cols * (S + 32 + pad)
    sheet = Image.new("RGB", (W, 50 + rows * (S + 36)), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(sheet)
    d.text((pad, 12), f"Crafting's fallback icons ({len(icons)}): parts, then tool kinds, at 4x and slot size",
           fill=(0xf5, 0xe4, 0xbc), font=big)
    for i, (n, im) in enumerate(icons):
        x, y = pad + (i % cols) * (S + 32 + pad), 46 + (i // cols) * (S + 36)
        d.rectangle([x, y, x + S - 1, y + S - 1], fill=(0x35, 0x20, 0x15), outline=(0x68, 0x3e, 0x26))
        t = im.resize((S - 8, S - 8), Image.NEAREST)
        sheet.paste(t, (x + 4, y + 4), t)
        d.rectangle([x + S + 2, y, x + S + 29, y + 27], fill=(0x35, 0x20, 0x15))
        small = im.resize((24, 24), Image.NEAREST)
        sheet.paste(small, (x + S + 4, y + 2), small)
        d.text((x, y + S + 3), n, fill=(0xc2, 0xac, 0x88), font=font)
    out = os.path.join(HERE, "sheets", "fallback_icons.png")
    sheet.save(out)
    parts.art.archive(out, "sheet_fallback_icons", "sheets")
    print(out, len(icons))


if __name__ == "__main__":
    main()
