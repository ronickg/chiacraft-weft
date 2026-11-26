#!/usr/bin/env python3
"""A review sheet of the dead bodies (`body.py`): each model from two angles, and the quadruped repainted in two other
coats the way the game's painter does it (`recolor`, as `mods/animals/src/carcass.rs` `paint_body`). Writes
out/sheet.png; `--before <png> ...` puts earlier in-game shots above it."""

import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts"))
from render import render  # noqa: E402

OUT = os.path.join(HERE, "out")
from body import COAT_RGB, UNDER_RGB  # noqa: E402


def recolor(tex, mask, coat, under):
    out = tex.astype(float).copy()
    for m, (now, to) in ((0, (COAT_RGB, coat)), (1, (UNDER_RGB, under))):
        sel = mask == m
        out[sel, :3] = np.clip(out[sel, :3] * (np.array(to, float) / np.array(now, float)), 0, 255)
    return out.astype(np.uint8)


def main():
    tex = np.array(Image.open(os.path.join(OUT, "body.png")).convert("RGBA"))
    raw = open(os.path.join(HERE, "body_mask.bin"), "rb").read()
    w, h = np.frombuffer(raw[:4], "<u2")
    mask = np.frombuffer(raw[4:], np.uint8).reshape(h, w)
    names = ["quadruped", "bird", "biped", "reptile", "fish"]
    cell = 300
    rows = []
    for n in names:
        row = []
        for suffix in ("", "_open"):
            spec = json.load(open(os.path.join(OUT, f"carcass_{n}{suffix}.spec.json")))
            for yaw, pitch in ((35, -30), (150, -42)):
                ppu = {"quadruped": 2.2, "bird": 5.5, "biped": 2.2, "reptile": 2.2, "fish": 7}[n]
                img = Image.fromarray(render(spec, tex, yaw=yaw, pitch=pitch, ppu=ppu))
                img.thumbnail((cell, cell))
                row.append((f"{n}{suffix} {yaw}/{pitch}", img))
        rows.append(row)
    spec = json.load(open(os.path.join(OUT, "carcass_quadruped_open.spec.json")))
    row = []
    for label, coat, under in (("grey wolf coat", (118, 112, 104), (206, 200, 190)),
                               ("capybara coat", (140, 96, 60), (156, 112, 76)),
                               ("jaguar coat", (196, 144, 70), (232, 220, 196)),
                               ("black coat", (52, 46, 44), (70, 62, 58))):
        img = Image.fromarray(render(spec, recolor(tex, mask, coat, under), yaw=35, pitch=-30, ppu=2.2))
        img.thumbnail((cell, cell))
        row.append((label, img))
    rows.append(row)
    befores = [Image.open(p).convert("RGBA") for p in sys.argv[sys.argv.index("--before") + 1:]] if "--before" in sys.argv else []
    top = 0
    for im in befores:
        im.thumbnail((cell * 2, cell))
    top = max((im.height for im in befores), default=0) + (30 if befores else 0)
    sheet = Image.new("RGBA", (cell * 4, top + len(rows) * (cell + 20)), (120, 132, 110, 255))
    d = ImageDraw.Draw(sheet)
    x = 0
    for im in befores:
        sheet.paste(im, (x, 20))
        x += im.width + 10
    if befores:
        d.text((4, 4), "before (r6 in the game): the living model rolled onto its side", fill=(255, 255, 255, 255))
    for r, row in enumerate(rows):
        for c, (label, img) in enumerate(row):
            x0, y0 = c * cell, top + r * (cell + 20)
            sheet.alpha_composite(img, (x0 + (cell - img.width) // 2, y0 + 18 + (cell - img.height) // 2))
            d.text((x0 + 4, y0 + 2), label, fill=(255, 255, 255, 255))
    sheet.save(os.path.join(OUT, "sheet.png"))
    print(os.path.join(OUT, "sheet.png"))


if __name__ == "__main__":
    main()
