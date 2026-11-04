#!/usr/bin/env python3
"""Paints the armour worn on the player (`items.rs`: iron, gold, diamond and leather, each a
helmet, a chestplate, leggings and boots) at 32 texels a block, like the player's skin and clothes
(Ronald 11-04, "all 32"), the way `../clothes/build.py` paints clothes: every texel from where it
sits on the body, the house's light and occlusion, each material's colours and each set's look from
the approved item icons (`textures/item/<material>_<piece>.png`):

- iron: polished plate with brass trim, a great helm with a T visor, a leather belt with a buckle;
- gold: gold plate with red cloth (a mantling on the helm, a sash and skirt, strips on the legs);
- diamond: crystal plates framed by brown leather straps;
- leather: a studded cap with ear flaps, a riveted jerkin, belted trousers, boots with a cuff.

    ~/.venvs/blocky/bin/python mods/base/source/armor/build.py [--out dir] [--sheet out.png]

Writes `mods/base/assets/textures/entity/armor/<material>_{1,2}.png` (128 x 64: layer 1 the helmet,
chestplate and boots, layer 2 the leggings, drawn closer to the body). `--sheet` draws each set on
the 32 player, front and back, next to today's.
"""

import argparse
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
sys.path.insert(0, os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts"))
sys.path.insert(0, os.path.join(HERE, "..", "clothes"))
from paint import occlusion, paint_face_smooth, tones  # noqa: E402
from render import face_quad  # noqa: E402
import build as clothes  # noqa: E402

K = 2  # texels a skin pixel: 32 a block
OUT = os.path.join(REPO, "mods", "base", "assets", "textures", "entity", "armor")
PIECES = ["helmet", "chestplate", "leggings", "boots"]
WORN = {"helmet": ["head"], "chestplate": ["body", "arm"], "leggings": ["body", "leg"], "boots": ["leg"]}  # player.rs
LAYER = {"helmet": 1, "chestplate": 1, "leggings": 2, "boots": 1}
INFLATE = {"helmet": 1.0, "chestplate": 1.0, "leggings": 0.5, "boots": 1.0}  # items.rs

NONE, MAIN, DARK, LIGHT, TRIM, STRAP, CLOTH, VISOR, SOLE = range(-1, 8)
# Per material: base colours (MAIN, DARK, LIGHT, TRIM, STRAP, CLOTH, VISOR, SOLE) and shadow hues:
# metal and crystal shade toward purple and blue, leather and cloth warm (the house's rule).
PALETTE = {
    "iron": [(196, 198, 204), (132, 134, 146), (232, 234, 238), (214, 172, 74), (112, 74, 46), (150, 40, 36), (44, 40, 48), (70, 56, 50)],
    "gold": [(236, 192, 64), (190, 128, 34), (252, 228, 132), (252, 220, 110), (112, 74, 46), (184, 44, 38), (70, 44, 26), (110, 70, 34)],
    "diamond": [(104, 214, 222), (58, 150, 172), (196, 246, 246), (226, 182, 80), (116, 74, 44), (150, 40, 36), (30, 50, 64), (84, 58, 42)],
    "leather": [(150, 94, 56), (112, 66, 38), (184, 128, 82), (214, 172, 84), (90, 56, 34), (150, 40, 36), (60, 36, 26), (66, 42, 30)],
}
HUES = {"iron": 0.74, "gold": 0.06, "diamond": 0.58, "leather": 0.02}
SOFT = {"iron": (), "gold": (), "diamond": (), "leather": ()}  # no fur strands on plate or hide


def area(m, piece, part, face, p, col, row, w, h):
    """Which material each texel is (NONE where the piece doesn't cover). Designs in skin pixels."""
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, MAIN)
    none = np.full(x.shape, NONE)
    front, back, top, bottom = (np.bool_(v) for v in (face == "south", face == "north", face == "up", face == "down"))
    side = np.bool_(face in ("east", "west"))
    ax = np.abs(x)

    if piece == "helmet":
        if bottom:
            return none
        if m == "leather":  # a cap: the crown, ear flaps, a rolled brim studded with rivets
            reach = np.where(side & (z > -2.5), 24.6, np.where(front, 28.4, 25.6))
            a = np.where(y < reach, NONE, a)
            brim = (y < reach + 1.0) & (a != NONE) & ~top
            a = np.where(brim, DARK, a)
            a = np.where(brim & ((np.floor(x * 2 + z * 2) % 3) == 0) & ~side | brim & side & ((np.floor(z * 2 + y) % 3) == 0), TRIM, a)  # rivets
            a = np.where(top & (np.abs(x) < 0.5), DARK, a)  # the crown's seam
            return a
        # a great helm round the whole head, a band at its foot
        a = np.where(~top & (y < 25), TRIM if m == "iron" else DARK, a)
        a = np.where(top & (ax < 0.5), TRIM if m != "diamond" else STRAP, a)  # the crest ridge
        if front.any() and front:
            slit = (y > 27.4) & (y < 28.4) & (ax < 3.1)
            if m in ("iron", "gold"):
                slit |= (ax < 0.5) & (y > 25.4) & (y < 28.4)  # the T
                a = np.where((ax >= 0.5) & (ax < 1.0) & (y > 25) & (y < 31), LIGHT, a)  # the nose ridge's edges
            a = np.where(slit, VISOR, a)
        if m == "diamond":  # crystal panels in a frame of straps
            a = np.where((ax < 0.5) | (np.abs(z) < 0.5) & side | (y > 29.4) & (y < 30.2), STRAP, a)
            if front:
                a = np.where((y > 27.4) & (y < 28.4) & (ax < 3.1), VISOR, a)
        if m == "gold":  # a red mantling hanging down the back and the sides behind the ears
            a = np.where((back | side & (z < -2.0)) & (y < 27.0) & (y >= 25), CLOTH, a)
        return a

    if piece == "chestplate":
        if part == "arm":
            if bottom:
                return none
            a = np.where(y < 14, NONE, a)  # the hand
            pauldron = y > 20.6
            if m == "leather":
                a = np.where(pauldron, DARK, a)
                a = np.where((y > 20.6) & (y < 21.4) & ~top, STRAP, a)
                a = np.where((y >= 14) & (y < 15) & ~top, STRAP, a)  # the cuff
                return a
            a = np.where(pauldron & ~top & (y < 21.3), TRIM if m != "diamond" else STRAP, a)  # the pauldron's rim
            a = np.where((y >= 14) & (y < 15) & ~top, TRIM if m != "diamond" else STRAP, a)  # the vambrace's
            a = np.where((y > 17.4) & (y < 18.2) & ~top, DARK, a)  # the elbow's lame
            if m == "gold":
                a = np.where((y >= 15) & (y < 17.4) & ~top & (face != "east"), CLOTH, a)  # a red sleeve under the plate
            return a
        if bottom:
            a[:] = DARK
            return a
        belt = ~top & (y < 13.6)
        if m == "leather":  # a riveted jerkin, its belt and buckle
            rivets = ((np.floor(x * 2) + np.floor(y * 2)) % 4 == 0) & (np.floor(y * 2) % 2 == 0) & (y > 14) & ~top
            a = np.where(rivets & (front | back), LIGHT, a)
            a = np.where(front & (ax < 0.5) & (y > 14), DARK, a)  # the lacing
        else:
            if front.any() and front:
                a = np.where((ax < 0.5) & (y > 14), LIGHT, a)  # the breastplate's ridge
                a = np.where((y > 22.9) & (ax < 2.5), TRIM if m != "diamond" else STRAP, a)  # the gorget's rim
                a = np.where((y > 17.4) & (y < 18.1), DARK, a)  # the plackart's edge
            if m == "diamond":  # straps from the shoulders to the belt, framing the crystal
                a = np.where(~top & (np.abs(ax - 2.25) < 0.5), STRAP, a)
            if m == "gold":
                a = np.where(~top & (y < 15.4) & (y >= 13.6), CLOTH, a)  # a red sash over the belt
            a = np.where(top & (ax > 2.5), TRIM if m != "diamond" else STRAP, a)  # the shoulders' rims
        a = np.where(belt, STRAP, a)
        a = np.where(belt & front & (ax < 0.75), TRIM, a)  # the buckle
        return a

    if piece == "leggings":
        if part == "body":
            if top:
                return none
            a = np.where(~bottom & (y > 15), NONE, a)  # the hips
            if m == "gold":
                a = np.where((front | back) & (y < 14.5) & ((np.floor(x * 2) % 3) != 0), CLOTH, a)  # a red skirt in strips
            a = np.where(~bottom & (y > 14) & (a != NONE), STRAP, a)  # the belt
            if m == "leather":
                a = np.where(front & (y > 14) & (ax < 0.75), TRIM, a)  # its buckle
            return a
        if top:
            return none
        knee = (y > 5.4) & (y < 7.0)
        if m == "leather":
            a = np.where(front & (np.abs(x + 2) < 0.25), DARK, a)  # a seam down the front
            a = np.where(knee & front, LIGHT, a)  # a worn knee
            a = np.where(~bottom & (y < 1.0), DARK, a)
            return a
        a = np.where(knee, LIGHT, a)  # the poleyn
        a = np.where((y > 6.5) & (y < 7.0), TRIM if m != "diamond" else STRAP, a)  # its rim
        if m == "diamond":
            a = np.where((y > 9.4) & (y < 10.2), STRAP, a)  # a strap round the thigh
        if m == "gold":
            a = np.where(front & (y > 7.0) & (np.abs(x + 2) < 1.0), CLOTH, a)  # a red strip down the thigh
        a = np.where(~bottom & (y < 1.0), DARK, a)
        return a

    # boots
    if top:
        return none
    if bottom:
        a[:] = SOLE
        return a
    tall = {"iron": 4.5, "gold": 5.5, "diamond": 5.0, "leather": 5.6}[m]
    a = np.where(y > tall, NONE, a)
    cuff = (y > tall - 1.0) & (y <= tall)
    a = np.where(cuff, {"iron": TRIM, "gold": CLOTH, "diamond": STRAP, "leather": LIGHT}[m], a)
    if m != "leather":
        a = np.where(front & (y < 2.0) & (y >= 0.6), LIGHT, a)  # the toe cap
    else:
        a = np.where(front & (y > 1.4) & (y < 2.0) & ((np.floor(x * 2) % 2) == 0), STRAP, a)  # laces
    a = np.where(y < 0.6, SOLE, a)
    return a


def glints(m, face, a, col, row, w, h):
    """Metal and crystal catch the light: on plate (MAIN) a thin diagonal glint across each face's
    upper part (iron, gold), facets on crystal (diamond), a texel thick at any K."""
    if m == "leather" or face == "down":
        return a
    if m == "diamond":
        facet = ((col - row) % (4 * K) == 0) | ((col + row) % (4 * K) == 0)
        return np.where((a == MAIN) & facet, LIGHT, a)
    glint = (col + row) % (6 * K) == 2 * K
    upper = row < h * 0.6
    return np.where((a == MAIN) & glint & upper, LIGHT, a)


def paint_piece(m, piece, sheet):
    tone = [tones(c, HUES[m] if k in (MAIN, DARK, LIGHT) else (0.06 if k == TRIM else 0.02)) for k, c in enumerate(PALETTE[m])]
    for part in WORN[piece]:
        lo, hi, (u, v) = clothes.PARTS[part]
        lo, hi = np.array(lo, float), np.array(hi, float)
        size = hi - lo
        for face, (fu, fv, fw, fh) in clothes.rects(u, v, int(size[0]), int(size[1]), int(size[2])).items():
            w, h = fw * K, fh * K
            corner, Uv, Vv = face_quad(lo, hi, face)
            cc, rr = np.meshgrid(np.arange(w), np.arange(h))
            p = corner + ((cc + 0.5) / w)[..., None] * Uv + ((rr + 0.5) / h)[..., None] * Vv
            a = glints(m, face, area(m, piece, part, face, p, cc, rr, w, h), cc, rr, w, h)
            if (a == NONE).all():
                continue
            occ = occlusion(p * clothes.U, np.array(clothes.NORMAL[face], float), Uv, Vv, clothes.BOXES, own=part)
            down = np.full((h, w), 0.3) if piece == "helmet" and face not in ("up", "down") else None
            col = paint_face_smooth(np.where(a == NONE, MAIN, a), tone, "north" if face == "down" else face, coat=SOFT[m],
                                    y_world=p[..., 1] * clothes.U, occ=occ * 0.6, down=down,
                                    seed=sum(map(ord, m + piece + part + face)) % 97, merge=False, tuft=False)
            img = np.dstack([np.clip(np.round(col), 0, 255), np.where(a == NONE, 0, 255)]).astype(np.uint8)
            region = sheet[fv * K:fv * K + h, fu * K:fu * K + w]
            region[:] = np.where(img[..., 3:] > 0, img, region)


def paint_material(m):
    layers = {1: np.zeros((32 * K, 64 * K, 4), np.uint8), 2: np.zeros((32 * K, 64 * K, 4), np.uint8)}
    for piece in ("helmet", "chestplate", "boots", "leggings"):
        paint_piece(m, piece, layers[LAYER[piece]])
    return layers


def sheet_png(path, painted, before_dir):
    from PIL import ImageDraw, ImageFont
    from render import render
    clothes.K = K
    skin = np.array(Image.open(os.path.join(REPO, "crates", "engine", "assets", "textures", "entity", "player.png")).convert("RGBA")
                    .resize((64 * K, 32 * K), Image.NEAREST))
    font = ImageFont.load_default(18)
    cw, ch = 220, 450
    out = Image.new("RGB", (100 + 4 * cw, 40 + len(painted) * ch), (236, 233, 226))
    d = ImageDraw.Draw(out)
    for k, t in enumerate(("today: front", "today: back", "at 32: front", "at 32: back")):
        d.text((100 + k * cw + 50, 10), t, fill=(20, 20, 20), font=font)
    rows = [(0, list(clothes.PARTS), 0.0)] + [(32 * LAYER[p], WORN[p], INFLATE[p]) for p in ("boots", "leggings", "chestplate", "helmet")]
    spec = clothes.player_spec(rows)
    for r, (m, layers) in enumerate(painted.items()):
        old = [np.array(Image.open(os.path.join(before_dir, f"{m}_{n}.png")).convert("RGBA").resize((64 * K, 32 * K), Image.NEAREST)) for n in (1, 2)]
        d.text((10, 40 + r * ch + ch // 2), m, fill=(20, 20, 20), font=font)
        for j, lay in enumerate((old, [layers[1], layers[2]])):
            atlas = np.concatenate([skin, lay[0], lay[1]], 0)
            for i, yaw in enumerate((25, 205)):
                img = render(spec, atlas, {}, yaw, 8, 12.5, size=(cw, ch), center=np.array([0, 16.0, 0]))
                im = Image.fromarray(img)
                out.paste(im, (100 + (2 * j + i) * cw, 40 + r * ch), im)
    out.save(path)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=OUT)
    ap.add_argument("--sheet")
    ap.add_argument("--before", default=OUT, help="today's textures, for the sheet")
    a = ap.parse_args()
    painted = {m: paint_material(m) for m in PALETTE}
    if a.sheet:
        sheet_png(a.sheet, painted, a.before)
        print("sheet", a.sheet)
    os.makedirs(a.out, exist_ok=True)
    for m, layers in painted.items():
        for n, img in layers.items():
            Image.fromarray(img, "RGBA").save(os.path.join(a.out, f"{m}_{n}.png"))
    print("wrote", ", ".join(f"{m}_1/2" for m in painted), "into", a.out)
