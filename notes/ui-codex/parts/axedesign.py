"""The axe redesign (Ronald, 11-04: "maybe for axes need a better design, also check online for some
reference ones"): what reads well at 32 px, from looking at Minecraft's axes and at real axe heads,
drawn here as our own flat schematics (no other game's pixels are copied or attached), beside three
iron design directions to pick from.

What the references taught (minecraft.wiki/w/Axe, read as a 16 x 16 brightness grid in the browser;
real axe anatomy from awesomeaxes.com and Wikipedia's Axe):
- the head sits crosswise at the top of the handle; the blade sticks out to the UPPER LEFT and its
  cutting edge runs PARALLEL to the handle; a short poll sticks out to the lower right;
- the head spans about half the canvas (8-9 of 16 px in Minecraft), the handle about 2 px wide;
- the bright highlight runs along the cutting edge, the head darkens toward the handle.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/axedesign.py   # sheets/axe_redesign.png
"""
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import toolfix as tf  # noqa: E402

# Minecraft's iron axe as measured (16 x 16, brightness 0-9, '.' empty), kept only as a flat layout silhouette
MC = """................
.........22.....
........2992....
.......29782....
......2977723...
......0987571...
.......0027570..
........231770..
.......241.00...
......231.......
.....231........
....241.........
...231..........
..241...........
..11............
................""".split("\n")

U = np.array([1.0, -1.0]) / np.sqrt(2)   # along the handle, toward the head
N = np.array([-1.0, -1.0]) / np.sqrt(2)  # across it, toward the upper left (where the blade goes)


def schematic(neck, edge_up, edge_down, length, poll, poll_w):
    """A flat axe silhouette on 32 px: handle on the diagonal, head crosswise at its top, blade to the
    upper left with its edge parallel to the handle (from edge_up along the handle to edge_down back)."""
    im = Image.new("RGBA", (32, 32))
    d = ImageDraw.Draw(im)
    top = np.array([22.0, 9.0])
    d.line([(3, 29), tuple(top)], fill=(150, 110, 70, 255), width=3)  # the handle
    p = lambda v: tuple(v)
    blade = [top + U * neck / 2, top + U * edge_up + N * length, top - U * edge_down + N * length, top - U * neck / 2]
    d.polygon([p(v) for v in blade], fill=(205, 205, 210, 255))
    pl = [top + U * poll_w / 2, top + U * poll_w / 2 - N * poll, top - U * poll_w / 2 - N * poll, top - U * poll_w / 2]
    d.polygon([p(v) for v in pl], fill=(170, 170, 178, 255))
    # the highlight along the cutting edge
    d.line([p(top + U * edge_up + N * length), p(top - U * edge_down + N * length)], fill=(255, 255, 255, 255), width=1)
    return im


def mc_silhouette():
    im = Image.new("RGBA", (16, 16))
    for y, row in enumerate(MC):
        for x, ch in enumerate(row):
            if ch != ".":
                v = 90 + int(ch) * 18
                im.putpixel((x, y), (v, v, v, 255))
    return im.resize((32, 32), Image.NEAREST)


def main():
    S, pad = 120, 14
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    W = pad + 6 * (S + pad) + 40
    im = Image.new("RGB", (W, 1000), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    d.text((pad, 12), "Axe redesign: what reads at 32 px, and three iron directions to pick from", fill=(0xf5, 0xe4, 0xbc), font=big)
    y = 46

    def tile(img, x, y, label, bad=False):
        d.rectangle([x, y, x + S - 1, y + S - 1], fill=(0x35, 0x20, 0x15), outline=(0xe0, 0x57, 0x4a) if bad else (0x68, 0x3e, 0x26), width=3 if bad else 1)
        if img is not None:
            t = img.resize((S - 10, S - 10), Image.NEAREST)
            im.paste(t, (x + 5, y + 5), t)
        for i, part in enumerate(label.split("|")):
            d.text((x, y + S + 3 + i * 16), part, fill=(0xe0, 0x57, 0x4a) if bad else (0xc2, 0xac, 0x88), font=font)

    d.text((pad, y), "1. References, as flat schematics (layout only)", fill=(0xff, 0xbe, 0x45), font=big)
    y += 28
    refs = [(mc_silhouette(), "Minecraft's layout|(measured 16 px, x2)"),
            (schematic(4, 5, 6, 9, 3, 4), "real felling axe|broad bit, flat poll"),
            (schematic(3, 3, 10, 8, 3, 3), "real bearded axe|beard runs down|beside the handle"),
            (schematic(3, 3, 4, 6, 4, 4), "real hatchet|small head,|hammer poll"),
            (tf.drawn("iron_axe") is not None and Image.fromarray(tf.drawn("iron_axe")), "ours, in the game:|edge vertical (wrong)", True),
            (Image.fromarray(tf.drawn("diag_iron_axe")), "last try: edge|across the handle (wrong)", True)]
    for k, r in enumerate(refs):
        tile(r[0], pad + k * (S + pad), y, r[1], len(r) > 2)
    y += S + 62
    rules = ["What reads at 32 px:",
             "- the head sits crosswise at the handle's top; the blade sticks out to the upper left, its cutting edge PARALLEL to the handle;",
             "- a short poll sticks out on the other side (lower right); the head spans about half the canvas, the handle 2-3 px wide;",
             "- the brightest pixels run along the cutting edge; the head darkens toward the handle; one outline round the whole tool."]
    for i, line in enumerate(rules):
        d.text((pad, y + i * 20), line, fill=(0xf5, 0xe4, 0xbc) if i == 0 else (0xc2, 0xac, 0x88), font=font)
    y += 20 * len(rules) + 16
    d.text((pad, y), "2. Three iron directions (two samples each), drawn by the skill on that layout", fill=(0xff, 0xbe, 0x45), font=big)
    y += 28
    names = [("A_classic", "A. classic"), ("B_bearded", "B. bearded"), ("C_felling", "C. felling")]
    k = 0
    for key, lbl in names:
        for v in (1, 2):
            a = tf.drawn(f"axe_{key}_{v}")
            tile(Image.fromarray(a) if a is not None else None, pad + k * (S + pad), y, f"{lbl}, sample {v}")
            k += 1
    y += S + 30
    # the same at the size they're played at
    d.text((pad, y), "At slot size (32 px at 1x and 2x):", fill=(0xc2, 0xac, 0x88), font=font)
    y += 22
    x = pad
    for key, _ in names:
        for v in (1, 2):
            a = tf.drawn(f"axe_{key}_{v}")
            if a is None:
                continue
            img = Image.fromarray(a)
            for sc in (1, 2):
                d.rectangle([x, y, x + 32 * sc + 7, y + 32 * sc + 7], fill=(0x35, 0x20, 0x15))
                t = img.resize((32 * sc, 32 * sc), Image.NEAREST)
                im.paste(t, (x + 4, y + 4), t)
                x += 32 * sc + 14
            x += 10
    y += 64 + 20
    out = os.path.join(HERE, "sheets", "axe_redesign.png")
    im = im.crop((0, 0, W, y))
    im.save(out)
    tf.parts.art.archive(out, "sheet_axe_redesign", "sheets")
    print(out, im.size)


if __name__ == "__main__":
    main()
