"""Survival's tanning and charcoal art (Survival session, 11-04): the tanning frame, tanning pit and
charcoal clamp (a top and a side each, for their model boxes) and the hide, sole leather and oak bark
icons. Writes deliver/tanning/textures/{block,item}/ in base's names (over Survival's placeholders when
Ronald approves) and sheets/tanning.png: the faces flat, the three blocks on their boxes in a world
scene, the icons in slots beside our buckskin leather.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/tanningsheet.py
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
B = os.path.join(ROOT, "mods", "base", "assets", "textures")
D = os.path.join(HERE, "deliver", "tanning", "textures")
BLOCKS = {"tanning_frame": (2, 0, 2, 14, 16, 14), "tanning_pit": (0, 0, 0, 16, 12, 16), "charcoal_clamp": (0, 0, 0, 16, 12, 16)}
ITEMS = ["hide", "sole_leather", "oak_bark"]
# Survival's colour for sole leather: dark and stiff, #4b2f1b; the drawing came out orange (mean 113, 56, 24)
TARGET = {"sole_leather": np.array([0x4b, 0x2f, 0x1b]) * 1.15}  # the body's mean, a touch above the darkest tone


def load(n):
    return np.array(Image.open(os.path.join(OUT, n + "_px.png")).convert("RGBA"))


def tune(n, a):
    if n not in TARGET:
        return a
    out = a.astype(float)
    m = out[..., 3] > 0
    out[m, :3] *= TARGET[n] / out[m, :3].mean(0)
    return out.clip(0, 255).astype(np.uint8)


def save(a, rel):
    p = os.path.join(D, rel)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    Image.fromarray(a).save(p)
    parts.art.archive(p, "tanning__" + rel.replace("/", "__")[:-4], "tanning")


def main():
    tex = {}
    for b in BLOCKS:
        for f in ("top", "side"):
            tex[f"{b}_{f}"] = load(f"{b}_{f}")
            save(tex[f"{b}_{f}"], f"block/{b}_{f}.png")
    for n in ITEMS:
        tex[n] = tune(n, load(n))
        save(tex[n], f"item/{n}.png")
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    W = 1300
    im = Image.new("RGB", (W, 1500), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    d.text((14, 12), "Tanning and charcoal (Survival): the blocks' faces, the blocks on their boxes, and the new icons",
           fill=(0xf5, 0xe4, 0xbc), font=big)
    y = 48
    d.text((14, y), "1. The faces (the frame shows its middle 24 columns; the pit and clamp their lower 24 rows)",
           fill=(0xff, 0xbe, 0x45), font=big)
    y += 28
    x = 14
    for b, (x0, y0, z0, x1, y1, z1) in BLOCKS.items():
        for f in ("top", "side"):
            t = Image.fromarray(tex[f"{b}_{f}"]).resize((128, 128), Image.NEAREST)
            im.paste(t, (x, y))
            # mark what the game doesn't show
            dd = ImageDraw.Draw(im)
            if f == "side":
                cx0, cx1 = x0 * 8, x1 * 8
                cy0, cy1 = (16 - y1) * 8, (16 - y0) * 8
            else:
                cx0, cx1, cy0, cy1 = x0 * 8, x1 * 8, z0 * 8, z1 * 8
            dd.rectangle([x + cx0, y + cy0, x + cx1 - 1, y + cy1 - 1], outline=(0xff, 0xbe, 0x45))
            d.text((x, y + 130), f"{b}_{f}", fill=(0xc2, 0xac, 0x88), font=font)
            x += 140
        x += 14
    y += 170
    d.text((14, y), "2. On their boxes, on grass (frame 12 wide; pit and clamp 12 tall)", fill=(0xff, 0xbe, 0x45), font=big)
    y += 28
    g = np.array(Image.open(os.path.join(B, "block", "grass_top.png")).convert("RGBA")).astype(float)
    g[..., :3] *= np.array([0x91, 0xBD, 0x59]) / 255
    g = g.astype(np.uint8)
    dirt = Image.open(os.path.join(B, "block", "dirt.png"))
    logs = Image.open(os.path.join(B, "block", "log_oak.png"))
    s = iso.Scene()
    for xx in range(6):
        for zz in range(3):
            s.box((xx, 0, zz), top=g, side=dirt)
    s.box((0, 1, 1), top=tex["tanning_frame_top"], side=tex["tanning_frame_side"], box=BLOCKS["tanning_frame"])
    s.box((2, 1, 1), top=tex["tanning_pit_top"], side=tex["tanning_pit_side"], box=BLOCKS["tanning_pit"])
    s.box((4, 1, 1), top=tex["charcoal_clamp_top"], side=tex["charcoal_clamp_side"], box=BLOCKS["charcoal_clamp"])
    s.box((5, 1, 0), top=Image.open(os.path.join(B, "block", "log_oak_top.png")), side=logs)
    scene = s.render(3)
    im.paste(scene, (14, y), scene)
    y += scene.height + 20
    d.text((14, y), "3. The icons at 4x and slot size, beside buckskin leather (what the frame makes) and the oak log",
           fill=(0xff, 0xbe, 0x45), font=big)
    y += 28
    icons = [(n, tex[n]) for n in ITEMS] + [("leather (buckskin, now)", np.array(Image.open(os.path.join(B, "item", "leather.png")).convert("RGBA")))]
    x = 14
    for n, a in icons:
        d.rectangle([x, y, x + 135, y + 135], fill=(0x35, 0x20, 0x15), outline=(0x68, 0x3e, 0x26))
        t = Image.fromarray(a).resize((128, 128), Image.NEAREST)
        im.paste(t, (x + 4, y + 4), t)
        d.rectangle([x + 142, y, x + 177, y + 35], fill=(0x35, 0x20, 0x15))
        sm = Image.fromarray(a).resize((32, 32), Image.NEAREST)
        im.paste(sm, (x + 144, y + 2), sm)
        d.text((x, y + 140), n, fill=(0xc2, 0xac, 0x88), font=font)
        x += 200
    y += 175
    out = os.path.join(HERE, "sheets", "tanning.png")
    im = im.crop((0, 0, W, y))
    im.save(out)
    parts.art.archive(out, "sheet_tanning", "sheets")
    print(out, im.size)


if __name__ == "__main__":
    main()
