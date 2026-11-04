#!/usr/bin/env python3
"""The tree-look audit: every species grown from branches (`base_api::branchy`), one sheet each,
its young (the understory's `young_tree_blocks`, or its young recipe) and three seeds grown, each
from the side and at three quarters, drawn with the game's 32 px textures (leaves tinted as the
biome does); and a line-up of all species at one scale. References (photos, mods) are pasted in
from `refs/<species>.png` when there.

    cargo run -p base_api --example tree_audit -- 3 > trees.json
    ~/.venvs/blocky/bin/python mods/base/source/trees/audit.py trees.json OUTDIR [refs_dir]
"""

import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
sys.path.insert(0, os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts"))
sys.path.insert(0, HERE)
from render import render, view_matrix  # noqa: E402
from sheet import spec_of  # noqa: E402

BASE = os.path.join(REPO, "mods", "base", "assets", "textures", "block")
ANIMALS = os.path.join(REPO, "mods", "animals", "assets", "textures", "block")
# log: (log texture, leaves texture, folder, leaf tint or None (own colours), young: "ball"/"cone"/None, name)
SPECIES = {
    "base:oak_log": ("oak", "oak", BASE, (104, 150, 58), "ball", "oak (Quercus robur)"),
    "base:birch_log": ("birch", "birch", BASE, (128, 167, 85), "ball", "birch (Betula pendula)"),
    "base:spruce_log": ("spruce", "spruce", BASE, (97, 140, 97), "cone", "spruce (Picea abies)"),
    "base:acacia_log": ("acacia", "acacia", BASE, (126, 150, 52), None, "acacia (Vachellia tortilis)"),
    "base:mesquite_log": ("mesquite", "mesquite", BASE, (120, 140, 70), None, "mesquite (Prosopis glandulosa)"),
    "base:acuri_palm_log": ("acuri_palm", "acuri_palm", BASE, (110, 150, 60), None, "acuri palm (Attalea phalerata)"),
    "base:moriche_palm_log": ("moriche_palm", "moriche_palm", BASE, (104, 146, 62), None, "moriche palm (Mauritia flexuosa)"),
    "animals:araucaria_log": ("araucaria", "araucaria", ANIMALS, (80, 120, 60), None, "araucaria (Cretaceous)"),
    "animals:ginkgo_log": ("ginkgo", "ginkgo", ANIMALS, (150, 170, 60), "ball", "ginkgo (Cretaceous)"),
    "animals:swamp_cypress_log": ("swamp_cypress", "swamp_cypress", ANIMALS, (96, 130, 70), "cone", "swamp cypress (Cretaceous)"),
    "animals:scale_conifer_log": ("scale_conifer", "scale_conifer", ANIMALS, (84, 120, 70), "cone", "scale conifer (Cretaceous)"),
    "animals:tempskya_log": ("tempskya", "tempskya", ANIMALS, (100, 150, 70), None, "tempskya tree fern (Cretaceous)"),
    "base:sequoia_log": ("sequoia", "sequoia", BASE, (77, 138, 60), "cone", "giant sequoia"),
    "base:dogwood_log": ("dogwood", "dogwood_green", BASE, None, "ball", "Pacific dogwood (no recipe: trees.rs round crown)"),
}
# The tropical trees (Plants' branch until it lands): textures from `AUDIT_TEXTURES` (that tree's
# block folder) when they aren't here yet.
TROP = os.environ.get("AUDIT_TEXTURES", BASE)
SPECIES.update({
    "base:cacao_log": ("cacao", "cacao", TROP, (96, 140, 60), "ball", "cacao (Theobroma cacao)"),
    "base:andiroba_log": ("andiroba", "andiroba", TROP, (90, 140, 60), "ball", "andiroba (Carapa guianensis)"),
    "base:matamata_log": ("matamata", "matamata", TROP, (90, 140, 60), "ball", "matamata (Eschweilera)"),
    "base:ipe_log": ("ipe", "ipe", TROP, (96, 140, 60), "ball", "ipê (Handroanthus)"),
    "base:brazil_nut_log": ("brazil_nut", "brazil_nut", TROP, (90, 140, 60), "ball", "Brazil nut (Bertholletia excelsa)"),
})


def atlas(sp):
    log, leaves, folder, tint = SPECIES[sp][:4]
    side = Image.open(os.path.join(folder, f"log_{log}.png")).convert("RGBA").resize((32, 32), Image.NEAREST)
    top_p = os.path.join(folder, f"log_{log}_top.png")
    top = Image.open(top_p if os.path.exists(top_p) else os.path.join(folder, f"log_{log}.png")).convert("RGBA").resize((32, 32), Image.NEAREST)
    lv = np.array(Image.open(os.path.join(folder, f"leaves_{leaves}.png")).convert("RGBA").resize((32, 32), Image.NEAREST)).astype(float)
    if tint:
        lv[..., :3] = lv[..., :3] * np.array(tint) / 255.0
    a = np.zeros((32, 96, 4), np.uint8)
    a[:, 0:32] = np.array(side)
    a[:, 32:64] = np.array(top)
    a[:, 64:96] = np.clip(lv, 0, 255).astype(np.uint8)
    return a


def h3(seed, x, y, z):
    v = (seed * 0x9E3779B1 ^ x * 73856093 ^ y * 19349663 ^ z * 83492791) & 0xFFFFFFFF
    v ^= v >> 13
    v = (v * 0x5BD1E995) & 0xFFFFFFFF
    return v ^ (v >> 15)


def young(kind, h):
    """dev's `young_tree_blocks` (the understory's young): a trunk 2-4 high under a ragged ball,
    or rings round it for the cone trees."""
    out = {}
    height = 2 + h % 3
    top = height
    if kind == "cone":
        for k in range(1, height):
            for dx, dz in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                if k % 2 == 1 or h3(h, dx, k, dz) & 1 == 0:
                    out[(dx, k, dz)] = 0
        out[(0, top, 0)] = 0
        out[(0, top + 1, 0)] = 0
    else:
        for cy in (top - 1, top):
            for dz in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    if abs(dx) + abs(dz) == 2 and h3(h, dx, cy, dz) & 3 != 0:
                        continue
                    out[(dx, cy, dz)] = 0
        out[(0, top + 1, 0)] = 0
    for k in range(height):
        out[(0, k, 0)] = 2
    return [[x, y, z, p] for (x, y, z), p in out.items()]


def round_crown(h, form):
    """dev's plain round crown (`trees.rs` `Crown::Round`), for a log with no recipe."""
    big = form == "big"
    height = 5 + h % 3 + 2 * big
    top = height
    out = {}
    wide = 4 if big else 3
    depth = 3 + big
    for y in range(top - depth, top + 1):
        r = wide - 1 if y >= top - 1 else wide
        for dz in range(-r, r + 1):
            for dx in range(-r, r + 1):
                corner = abs(dx) == r and abs(dz) == r
                if corner and (y == top or h3(0, dx, y, dz) & 1 == 0):
                    continue
                out[(dx, y, dz)] = 0
    for y in range(top):
        out[(0, y, 0)] = 2
    return [[x, y, z, p] for (x, y, z), p in out.items()]


def bbox(t):
    a = np.array([b[:3] for b in t])
    return a.min(0), a.max(0) + 1


def draw(t, at, yaw, pitch, ppb, cell):
    lo, hi = bbox(t)
    mid = (lo + hi) / 2.0
    mid[1] = (lo[1] + hi[1]) / 2.0
    img = render(spec_of(t), at, {}, yaw, pitch, ppb, size=cell, center=view_matrix(yaw, pitch) @ mid)
    return Image.fromarray(img)


VIEWS = [(0, 6, "side"), (35, 24, "3/4")]


def sheet(sp, trees, refs, out):
    at = atlas(sp)
    young_kind = SPECIES[sp][4]
    grown = trees[sp]
    # columns: the understory's young (its own recipe, small, when the dump has it; else dev's
    # ball on a stick), a young recipe's tree, then the grown ones
    cols = []
    if trees.get("understory:" + sp):
        cols += [(f"young (understory), seed {i}", t) for i, t in enumerate(trees["understory:" + sp][:2])]
    elif young_kind:
        cols += [("young (dev: ball on a stick)", young(young_kind, 7))]
    else:
        cols += [("young: none", None)]
    if trees.get("young:" + sp):
        cols += [("young (YOUNG recipe)", trees["young:" + sp][0])]
    cols += [(f"grown, seed {i}", t) for i, t in enumerate(grown)]
    hmax = max(bbox(t)[1][1] - bbox(t)[0][1] for t in grown)
    wmax = max(max(bbox(t)[1][[0, 2]] - bbox(t)[0][[0, 2]]) for t in grown)
    cell = (300, 460)
    ppb = min(18.0, 400.0 / (hmax + 1), 250.0 / (wmax * 1.25 + 1))
    font = ImageFont.load_default(17)
    small = ImageFont.load_default(14)
    ref = None
    if refs and os.path.exists(os.path.join(refs, sp.split(":")[1] + ".png")):
        ref = Image.open(os.path.join(refs, sp.split(":")[1] + ".png")).convert("RGB")
        ref.thumbnail((520, cell[1] * 2 + 30))
    W = 20 + len(cols) * cell[0] + (ref.size[0] + 20 if ref else 0)
    H = 50 + 2 * (cell[1] + 24)
    im = Image.new("RGB", (W, H), (236, 233, 226))
    d = ImageDraw.Draw(im)
    d.text((14, 10), f"{SPECIES[sp][5]}   ({sp}, {ppb:.1f} px a block)", fill=(20, 20, 20), font=font)
    for row, (yaw, pitch, vname) in enumerate(VIEWS):
        y0 = 40 + row * (cell[1] + 24)
        for c, (label, t) in enumerate(cols):
            x0 = 10 + c * cell[0]
            d.text((x0 + 6, y0), f"{label}, {vname}", fill=(60, 60, 60), font=small)
            if t is None:
                continue
            # the young at the grown trees' scale, so sizes compare
            r = draw(t, at, yaw, pitch, ppb, cell)
            im.paste(r, (x0, y0 + 20), r)
            lo, hi = bbox(t)
            d.text((x0 + 6, y0 + cell[1] + 2), f"{hi[1] - lo[1]} high, {hi[0] - lo[0]}x{hi[2] - lo[2]} wide, "
                   f"{sum(1 for b in t if b[3] > 0)} logs", fill=(90, 90, 90), font=small)
    if ref:
        im.paste(ref, (20 + len(cols) * cell[0], 40))
    im.save(out)
    return out


def lineup(trees, out):
    """All species, seed 0, at three quarters, at one scale (a block 8 px), sequoias apart."""
    ppb = 12.0
    items = [sp for sp in SPECIES if sp in trees]
    rendered = []
    for sp in items:
        t = trees[sp][0]
        lo, hi = bbox(t)
        s = ppb if hi[1] - lo[1] < 40 else ppb / 3  # the giants at a third
        r = draw(t, atlas(sp), 35, 12, s, (int(s * 30) + 40, int(s * 80) + 40))
        r = r.crop(r.getbbox())
        rendered.append((sp, r, s))
    H = max(r.size[1] for _, r, _ in rendered) + 70
    W = sum(max(r.size[0], 110) + 20 for _, r, _ in rendered) + 20
    im = Image.new("RGB", (W, H), (236, 233, 226))
    d = ImageDraw.Draw(im)
    font = ImageFont.load_default(14)
    x = 20
    for sp, r, s in rendered:
        w = max(r.size[0], 110)
        im.paste(r, (x + (w - r.size[0]) // 2, H - 50 - r.size[1]), r)
        name = SPECIES[sp][5].split(" (")[0] + (" (1/3 scale)" if s < ppb else "")
        d.text((x, H - 40), name, fill=(20, 20, 20), font=font)
        x += w + 20
    im.save(out)


def main(src, outdir, refs=None):
    trees = json.load(open(src))
    os.makedirs(outdir, exist_ok=True)
    # dogwood has no recipe: three of its plain crowns
    trees["base:dogwood_log"] = [round_crown(h, f) for h, f in ((0, "standing"), (1, "standing"), (2, "big"))]
    for sp in SPECIES:
        if sp in trees:
            p = sheet(sp, trees, refs, os.path.join(outdir, sp.split(":")[1] + ".png"))
            print(p)
    lineup(trees, os.path.join(outdir, "lineup.png"))


if __name__ == "__main__":
    main(*sys.argv[1:])
