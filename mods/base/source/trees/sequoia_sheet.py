#!/usr/bin/env python3
"""The giant sequoia's shapes sheet: grown trees from branches (`tree_recipes::SEQUOIA`) and young
ones (`YOUNG_SEQUOIA`), from the side and the first from below, each with a 2-block player at its
foot for scale. Chiacraft's bark, log end and foliage (the foliage in the grove's tint).

    cargo run -p base_api --example sequoia_dump -- 4 > sequoias.json
    ~/.venvs/blocky/bin/python mods/base/source/trees/sequoia_sheet.py sequoias.json out.png
"""

import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
sys.path.insert(0, os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts"))
from render import render, view_matrix  # noqa: E402

BLOCKS = os.path.join(REPO, "mods", "base", "assets", "textures", "block")
BARK = (176, 96, 66)  # the spruce's log tinted toward a sequoia's red-brown
LEAF = (0x4D, 0x8A, 0x3C)  # the grove's evergreen tint (worldgen.rs, sequoia_grove)
PLAYER = (40, 90, 190)


def tex(name, tint=None):
    im = np.array(Image.open(os.path.join(BLOCKS, name)).convert("RGBA").resize((32, 32), Image.NEAREST)).astype(float)
    if tint:
        grey = im[..., :3].mean(axis=2, keepdims=True)
        im[..., :3] = np.clip(grey * np.array(tint) / 128.0, 0, 255)
    return np.clip(im, 0, 255).astype(np.uint8)


def atlas():
    a = np.zeros((32, 128, 4), np.uint8)
    a[:, 0:32] = tex("log_sequoia.png")
    a[:, 32:64] = tex("log_sequoia_top.png")
    lv = tex("leaves_sequoia.png").astype(float)
    lv[..., :3] = lv[..., :3] * np.array(LEAF) / 255.0
    a[:, 64:96] = lv.astype(np.uint8)
    a[:, 96:128] = (*PLAYER, 255)
    return a


END = {1: ("east", "west"), 2: ("up", "down"), 3: ("south", "north")}
DIRS = {"south": (0, 0, 1), "north": (0, 0, -1), "east": (1, 0, 0), "west": (-1, 0, 0), "up": (0, 1, 0), "down": (0, -1, 0)}


def spec_of(blocks, player_x):
    solid = {(x, y, z) for x, y, z, p in blocks}
    els = []
    for x, y, z, p in blocks:
        faces = {}
        for f, (dx, dy, dz) in DIRS.items():
            n = (x + dx, y + dy, z + dz)
            if n in solid and (p > 0 or any(b == n for b in ())):
                continue  # hidden against wood or leaves behind wood
            if p == 0:
                faces[f] = [64, 0, 96, 32]
            else:
                faces[f] = [32, 0, 64, 32] if f in END[p] else [0, 0, 32, 32]
        if faces:
            els.append({"name": f"b{x}_{y}_{z}", "bone": "root", "lo": [x, y, z], "hi": [x + 1, y + 1, z + 1], "faces": faces})
    # A player: 0.6 x 1.8 x 0.3, beside the trunk.
    pf = {f: [96, 0, 128, 32] for f in DIRS}
    els.append({"name": "player", "bone": "root", "lo": [player_x, 0, 0], "hi": [player_x + 0.6, 1.8, 0.3], "faces": pf})
    return {"bones": [{"name": "root", "parent": None, "pivot": [0, 0, 0]}], "elements": els}


def main(src, out):
    trees = json.load(open(src))
    at = atlas()
    font = ImageFont.load_default(18)
    ppb = 6.0
    cell = (200, 520)
    rows = [("grown", trees["grown"]), ("young", trees["young"])]
    cols = max(len(v) for _, v in rows) + 1
    sheet = Image.new("RGB", (120 + cols * cell[0], 50 + len(rows) * cell[1]), (236, 233, 226))
    d = ImageDraw.Draw(sheet)
    d.text((10, 10), "giant sequoia grown from branches (Chiacraft's bark and foliage); blue: a 2-block player", fill=(20, 20, 20), font=font)
    for r, (label, ts) in enumerate(rows):
        d.text((10, 50 + r * cell[1] + cell[1] // 2), label, fill=(20, 20, 20), font=font)
        views = [(t, 25, 6, ppb) for t in ts] + [(ts[0], 35, 18, 16.0)]
        for k, (t, yaw, pitch, scale) in enumerate(views):
            wood0 = [b[0] for b in t if b[1] == 0 and b[3] > 0]
            px = (max(wood0) if wood0 else 1) + 1.5
            spec = spec_of(t, px)
            top = max(b[1] for b in t)
            close = scale > ppb
            mid = np.array([0.5, 12.0 if close else top / 2, 0.5])
            img = render(spec, at, {}, yaw, pitch, scale, size=cell, center=view_matrix(yaw, pitch) @ mid)
            im = Image.fromarray(img)
            sheet.paste(im, (120 + k * cell[0], 50 + r * cell[1]), im)
            h = top + 1
            d.text((120 + k * cell[0] + 8, 50 + r * cell[1] + cell[1] - 26), "the foot, closer" if close else f"{h} tall", fill=(40, 40, 40), font=font)
    sheet.save(out)
    print(out, sheet.size)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
