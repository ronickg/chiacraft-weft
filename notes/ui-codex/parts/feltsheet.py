"""The axes redrawn with the working part on the diagonal (Ronald, 11-04: "its not allowed to be
vertical like that the front part" / "Yes, that"): the game's axes with their vertical cutting
edges marked, the nine new axes (edge across the handle, one handle line), the new heads on the
other handle style, and the game's five offered as replacements.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/diagsheet.py   # sheets/axes_diagonal.png
"""
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import heads  # noqa: E402
import toolfix as tf  # noqa: E402

MATS = ["wood", "stone", "flint", "copper", "bronze", "iron", "steel", "gold", "diamond"]
CRUDE = {"wood", "stone", "flint"}
GAME = ["wood", "stone", "iron", "gold", "diamond"]


def vruns(a):
    """Straight vertical edges 5+ px long on the right side of opaque pixels."""
    m = a[..., 3] > 0
    edge = m & ~np.pad(m, ((0, 0), (0, 1)))[:, 1:]
    out = []
    for x in range(32):
        y = 0
        while y < 32:
            if edge[y, x]:
                y0 = y
                while y < 32 and edge[y, x]:
                    y += 1
                if y - y0 >= 5:
                    out.append((x, y0, y - 1))
            else:
                y += 1
    return out


def angle(a):
    _, _, (c, d) = heads.cut(a)
    return float(np.degrees(np.arctan2(-d[1], d[0])))


NEW = lambda m: "axe_C_felling_1" if m == "iron" else f"felling_{m}_axe"
OUTNAME = "axes_felling.png"


def main():
    S, pad = 112, 14
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    W = pad + len(MATS) * (S + pad) + 10
    im = Image.new("RGB", (W, 900), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    d.text((pad, 12), "Axes redesigned (felling axe, picked overnight): the game's axes, the new set, the new heads on the other handle",
           fill=(0xf5, 0xe4, 0xbc), font=big)
    y = 48

    def tile(img, x, y, label, runs=()):
        d.rectangle([x, y, x + S - 1, y + S - 1], fill=(0x35, 0x20, 0x15), outline=(0x68, 0x3e, 0x26))
        if img is not None:
            t = img.resize((S - 8, S - 8), Image.NEAREST)
            im.paste(t, (x + 4, y + 4), t)
            k = (S - 8) / 32
            for rx, y0, y1 in runs:
                sx = x + 4 + (rx + 1) * k
                d.line([(sx, y + 4 + y0 * k), (sx, y + 4 + (y1 + 1) * k)], fill=(255, 40, 40), width=4)
        else:
            d.text((x + 24, y + S // 2 - 8), "none", fill=(0xc2, 0xac, 0x88), font=font)
        for i, part in enumerate(label.split(", ")):
            d.text((x, y + S + 3 + i * 16), part, fill=(0xc2, 0xac, 0x88), font=font)

    def row(title, items):
        nonlocal y
        d.text((pad, y), title, fill=(0xff, 0xbe, 0x45), font=big)
        y += 28
        for k, it in enumerate(items):
            tile(*((it[0], pad + k * (S + pad), y) + it[1:]))
        y += S + 48

    old = []
    for m in MATS:
        if m in GAME:
            a = tf.drawn(f"{m}_axe")
            old.append((Image.fromarray(a), f"{m}, in the game", vruns(a)))
        else:
            old.append((None, m))
    row("1. The game's axes: the cutting edge a straight vertical line (red)", old)
    new = []
    angles = {}
    for m in MATS:
        a = tf.drawn(NEW(m))
        angles[m] = angle(a)
        new.append((Image.fromarray(a), f"{m}" + (" (new)" if m not in GAME else ""), vruns(a)))
    row("2. New felling axes: blade to the upper left, edge parallel to the handle, a flat poll", new)
    other = []
    for m in MATS:
                other.append((tf.anchored(NEW(m), NEW("iron") if m in CRUDE else NEW("wood"), "axe"), f"{m}, {'wrapped' if m in CRUDE else 'wood'} handle"))
    row("3. New heads on the other handle style (the handle moves to meet the head)", other)
    lines = [f"Handle line, degrees above horizontal: " + ", ".join(f"{m} {angles[m]:.0f}" for m in MATS) + " (45 is the canvas diagonal).",
             "Offer: the new wood, stone, iron, gold and diamond axes (row 2) as replacements for the game's five (row 1).",
             "Red marks any straight vertical edge 5 px or longer; row 2 should have none, or only a corner tip."]
    for i, line in enumerate(lines):
        d.text((pad, y + i * 22), line, fill=(0xc2, 0xac, 0x88), font=font)
    out = os.path.join(HERE, "sheets", OUTNAME)
    im = im.crop((0, 0, W, y + 22 * len(lines) + 14))
    im.save(out)
    tf.parts.art.archive(out, "sheet_" + OUTNAME[:-4], "sheets")
    print(out, im.size, {m: round(a) for m, a in angles.items()})


if __name__ == "__main__":
    main()
