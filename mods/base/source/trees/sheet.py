#!/usr/bin/env python3
"""A sheet of trees: each species as base's `trees.rs` grows it today (on dev) next to trees grown
from branches (`base_api::branchy`), drawn with the game's 32 block textures (leaves tinted as the
biome does), from the side and from below. For the world-looks round; shared with Voxel dog, who
owns the trees.

    cargo run -p base_api --example tree_dump -- 3 > trees.json
    ~/.venvs/blocky/bin/python mods/base/source/trees/sheet.py trees.json out.png
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

BASE = os.path.join(REPO, "mods", "base", "assets", "textures", "block")
ANIMALS = os.path.join(REPO, "mods", "animals", "assets", "textures", "block")
# species: (log, leaves, the folder, leaf tint, today's crown in trees.rs)
SPECIES = {  # log: (log texture, leaves texture, folder, leaf tint, today's crown in dev's trees.rs)
    "base:oak_log": ("oak", "oak", BASE, (104, 150, 58), "round"),
    "base:birch_log": ("birch", "birch", BASE, (128, 167, 85), "round"),
    "base:spruce_log": ("spruce", "spruce", BASE, (97, 140, 97), "cone"),
    "base:acacia_log": ("acacia", "acacia", BASE, (126, 150, 52), "flat"),
    "base:mesquite_log": ("mesquite", "mesquite", BASE, (120, 140, 70), "flat"),
    "base:acuri_palm_log": ("acuri_palm", "acuri_palm", BASE, (110, 150, 60), "palm"),
    "base:moriche_palm_log": ("moriche_palm", "moriche_palm", BASE, (104, 146, 62), "palm"),
    "animals:araucaria_log": ("araucaria", "araucaria", ANIMALS, (80, 120, 60), "flat"),
    "animals:ginkgo_log": ("ginkgo", "ginkgo", ANIMALS, (150, 170, 60), "round"),
    "animals:swamp_cypress_log": ("swamp_cypress", "swamp_cypress", ANIMALS, (96, 130, 70), "cone"),
    "animals:scale_conifer_log": ("scale_conifer", "scale_conifer", ANIMALS, (84, 120, 70), "cone"),
    "animals:tempskya_log": ("tempskya", "tempskya", ANIMALS, (100, 150, 70), "palm"),
}


def today(crown, height=5):
    """dev's `tree_blocks`, for the 'today' column (parts as tree_dump: 0 leaves, 2 log)."""
    out = {}
    top = height
    put = lambda x, y, z, p: out.__setitem__((x, y, z), max(p, out.get((x, y, z), -1)))  # noqa: E731
    if crown == "palm":
        top = 7
        put(0, top, 0, 0)
        for dx, dz in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            for k in (1, 2):
                put(dx * k, top - 1, dz * k, 0)
            put(dx * 3, top - 2, dz * 3, 0)
        for dx, dz in ((1, 1), (1, -1), (-1, 1), (-1, -1)):
            put(dx, top - 1, dz, 0)
            put(dx * 2, top - 2, dz * 2, 0)
    elif crown == "flat":
        for y, r2 in ((top, 10), (top + 1, 2)):
            for dz in range(-3, 4):
                for dx in range(-3, 4):
                    d = dx * dx + dz * dz
                    if d <= r2 and (d < r2 or (dx + 3 * dz + y) % 2 == 0):
                        put(dx, y, dz, 0)
    elif crown == "cone":
        h = 8
        top = h
        for dy in range(2, h + 2):
            r = 0 if dy == h + 1 else 1 + min((h + 1 - dy) % 3, 2)
            for dz in range(-r, r + 1):
                for dx in range(-r, r + 1):
                    if dx * dx + dz * dz <= r * r + 1:
                        put(dx, dy, dz, 0)
    else:
        for y in range(top - 3, top + 1):
            r = 1 if y >= top - 1 else 2
            for dz in range(-r, r + 1):
                for dx in range(-r, r + 1):
                    corner = abs(dx) == r and abs(dz) == r
                    if corner and (y == top or (dx * 7 + dz * 3 + y) % 2 == 0):
                        continue
                    put(dx, y, dz, 0)
    for y in range(0, top):
        put(0, y, 0, 2)
    return [[x, y, z, p] for (x, y, z), p in out.items()]


def atlas(sp):
    log, leaves, folder, tint, _ = SPECIES[sp]
    side = Image.open(os.path.join(folder, f"log_{log}.png")).convert("RGBA").resize((32, 32), Image.NEAREST)
    top = Image.open(os.path.join(folder, f"log_{log}_top.png")).convert("RGBA").resize((32, 32), Image.NEAREST)
    lv = np.array(Image.open(os.path.join(folder, f"leaves_{leaves}.png")).convert("RGBA").resize((32, 32), Image.NEAREST)).astype(float)
    lv[..., :3] = lv[..., :3] * np.array(tint) / 255.0
    a = np.zeros((32, 96, 4), np.uint8)
    a[:, 0:32] = np.array(side)
    a[:, 32:64] = np.array(top)
    a[:, 64:96] = np.clip(lv, 0, 255).astype(np.uint8)
    return a


END = {1: ("east", "west"), 2: ("up", "down"), 3: ("south", "north")}  # a log's cut ends, by its axis


def spec_of(blocks):
    solid = {(x, y, z) for x, y, z, p in blocks if p > 0}
    els = []
    for x, y, z, p in blocks:
        faces = {}
        for f, (dx, dy, dz) in {"south": (0, 0, 1), "north": (0, 0, -1), "east": (1, 0, 0), "west": (-1, 0, 0),
                                "up": (0, 1, 0), "down": (0, -1, 0)}.items():
            if p > 0 and (x + dx, y + dy, z + dz) in solid:
                continue  # wood against wood: hidden
            if p == 0:
                faces[f] = [64, 0, 96, 32]
            else:
                faces[f] = [32, 0, 64, 32] if f in END[p] else [0, 0, 32, 32]
        if faces:
            els.append({"name": f"b{x}_{y}_{z}", "bone": "root", "lo": [x, y, z], "hi": [x + 1, y + 1, z + 1], "faces": faces})
    return {"bones": [{"name": "root", "parent": None, "pivot": [0, 0, 0]}], "elements": els}


def main(src, out):
    grown = json.load(open(src))
    ppb = 16.0  # screen pixels a block
    cell = (290, 420)
    font = ImageFont.load_default(18)
    n = max(len(v) for v in grown.values())
    cols = 2 + n
    sheet = Image.new("RGB", (120 + cols * cell[0], 40 + len(SPECIES) * cell[1]), (236, 233, 226))
    d = ImageDraw.Draw(sheet)
    d.text((120 + 10, 8), "today (dev's trees.rs)", fill=(20, 20, 20), font=font)
    d.text((120 + cell[0] + 10, 8), "grown from branches (base_api::branchy), three seeds", fill=(20, 20, 20), font=font)
    d.text((120 + (1 + n) * cell[0] + 10, 8), "the first from below", fill=(20, 20, 20), font=font)
    for r, sp in enumerate(SPECIES):
        at = atlas(sp)
        d.text((10, 40 + r * cell[1] + cell[1] // 2), sp.split(":")[1].replace("_log", "").replace("_", " "), fill=(20, 20, 20), font=font)
        trees = [today(SPECIES[sp][4])] + grown.get(sp, [])
        views = [(30, 12)] * len(trees) + [(30, -70)]  # and the first grown one from below
        for k, (t, (yaw, pitch)) in enumerate(zip(trees + trees[1:2], views)):
            spec = spec_of(t)
            mid = np.array([0.5, 7.5 if pitch > 0 else 4.0, 0.5])
            img = render(spec, at, {}, yaw, pitch, ppb, size=cell, center=view_matrix(yaw, pitch) @ mid)
            im = Image.fromarray(img)
            sheet.paste(im, (120 + k * cell[0], 40 + r * cell[1]), im)
    sheet.save(out)
    print(out, sheet.size)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
