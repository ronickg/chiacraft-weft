#!/usr/bin/env python3
"""Paints the clothes worn on the player (`clothing.rs`: fur, wool, hide and linen, each a hat, a
coat, trousers and shoes) into the armour layout the player's skin uses (64 x 32: layer 1 the hat,
coat and shoes, layer 2 the trousers), the `blocky-model` way: every texel from where it sits on the
body, its material's colours, the house's light (one light from above and a bit front-left, hue-
shifted ramps) and occlusion from the body's boxes. The colours and each piece's look come from
Chiacraft's approved icons (`notes/ui-codex/redo/deliver/survival/<material>_<piece>.png`), so
what is worn matches what is held.

    ~/.venvs/blocky/bin/python mods/base/source/clothes/build.py [--sheet out.png]

Writes `mods/base/assets/textures/entity/clothes/<material>_{1,2}.png`; `--sheet` also draws the
player wearing each piece (front and back) and each whole outfit.

`K` is texels a skin pixel: 1 for the 64 x 32 layout (16 a block, as the player's own skin).
The engine maps a worn texture's pixels by its own size, so a finer one (K = 2, 128 x 64, 32 a
block) needs the engine to map worn layers by the skin's layout instead, and a 32 skin to match.
"""

import argparse
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
sys.path.insert(0, os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts"))
from paint import occlusion, paint_face_smooth, tones  # noqa: E402
from render import face_quad  # noqa: E402

K = 1
OUT = os.path.join(REPO, "mods", "base", "assets", "textures", "entity", "clothes")
SKIN = os.path.join(REPO, "crates", "engine", "assets", "textures", "entity", "player.png")
U = 4  # house units (1/64 block) a skin pixel: the painter's light and occlusion work in them

# The player (skin pixels, feet at 0, facing +z) and where each part sits in the 64 x 32 layout.
PARTS = {
    "head": ((-4, 24, -4), (4, 32, 4), (0, 0)),
    "body": ((-4, 12, -2), (4, 24, 2), (16, 16)),
    "arm": ((-8, 12, -2), (-4, 24, 2), (40, 16)),  # the right arm; the left draws it mirrored
    "leg": ((-4, 0, -2), (0, 12, 2), (0, 16)),
}
MIRRORED = {"armL": ("arm", (4, 12, -2), (8, 24, 2)), "legL": ("leg", (0, 0, -2), (4, 12, 2))}
BOXES = [(n, np.array(lo, float) * U, np.array(hi, float) * U) for n, (lo, hi, _) in PARTS.items()] + \
        [(n, np.array(lo, float) * U, np.array(hi, float) * U) for n, (_, lo, hi) in MIRRORED.items()]
INFLATE = {"hat": 0.45, "coat": 0.3, "trousers": 0.25, "shoes": 0.35}  # clothing.rs
WORN = {"hat": ["head"], "coat": ["body", "arm"], "trousers": ["body", "leg"], "shoes": ["leg"]}  # player.rs
LAYER = {"hat": 1, "coat": 1, "trousers": 2, "shoes": 1}
NORMAL = {"up": (0, 1, 0), "down": (0, -1, 0), "south": (0, 0, 1), "north": (0, 0, -1), "east": (1, 0, 0), "west": (-1, 0, 0)}


def rects(u, v, w, h, d):
    """A box's faces in the layout (Minecraft's box unwrap), in skin pixels: (u0, v0, width, height)."""
    return {"up": (u + d, v, w, d), "down": (u + d + w, v, w, d), "west": (u, v + d, d, h),
            "south": (u + d, v + d, w, h), "east": (u + d + w, v + d, d, h), "north": (u + 2 * d + w, v + d, w, h)}


# ------------------------------------------------------------------ materials, from the icons

NONE, MAIN, DARK, LIGHT, INNER, SOLE, LACE = range(-1, 6)
# Per material, the colours of MAIN, DARK, LIGHT, INNER, SOLE and LACE (the icons' clusters), and
# which are soft (fur and knit: strands and tufted edges).
PALETTE = {
    "fur": ([(150, 117, 95), (108, 80, 62), (246, 228, 194), (104, 74, 54), (92, 66, 50), (70, 50, 40)], (MAIN, LIGHT)),
    "wool": ([(226, 196, 150), (186, 152, 108), (247, 230, 192), (168, 128, 96), (150, 112, 84), (150, 112, 84)], (MAIN, LIGHT)),
    "hide": ([(212, 138, 60), (160, 90, 36), (240, 174, 92), (140, 74, 34), (108, 56, 26), (98, 50, 24)], ()),
    "linen": ([(228, 202, 166), (184, 152, 116), (246, 230, 200), (150, 116, 84), (126, 96, 72), (150, 116, 84)], ()),
}
HUES = [0.02, 0.02, 0.06, 0.02, 0.02, 0.02]  # warm shadows: things made from animals and plants


# ------------------------------------------------------------------ each piece's design


def bands(t, period, width=1):
    """Every `period`th texel across `t` (texel indices), `width` wide."""
    return (t % period) < width


def area(m, piece, part, face, p, col, row, w, h):
    """Which material each texel is (NONE where the piece doesn't cover). `p`: texel centres in
    skin pixels (x, y, z); `col`, `row`: texel indices on the face (u right, v down from outside)."""
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, MAIN)
    none = np.full(x.shape, NONE)
    # Designs are in skin pixels (pc, pr: which pixel of the 64 x 32 layout a texel is in), so a
    # rib, a fringe or a wrap keeps its size at any K; a line (a cable, a fold, a seam) keeps its
    # spacing and stays one texel thick (`ln`: the first texel of each pixel column).
    pc, pr, wp = col // K, row // K, w // K
    ln = col % K == 0
    # numpy booleans: `~` on a Python bool is -2, not False
    front, side, top, bottom = (np.bool_(v) for v in (face == "south", face in ("east", "west"), face == "up", face == "down"))

    if piece == "hat":
        if bottom:
            return none
        drop = {"south": 4, "east": 5, "west": 5, "north": 6, "up": 8}[face]  # how far down from the crown
        if m == "fur":  # a round fur cap, a thick cream fur brim all round (half its height in front)
            a = np.where(32 - y > drop, NONE, a)
            a = np.where(~top & (32 - y > drop - 2.1) & (a != NONE), LIGHT, a)
        elif m == "wool":  # a beanie: cables up the crown, a ribbed brim
            a = np.where(32 - y > drop, NONE, a)
            a = np.where(bands(pc, 3) & ln & ~top & (a != NONE), DARK, a)
            brim = ~top & (32 - y > drop - 2.1) & (a != NONE)
            a = np.where(brim, np.where(bands(pc, 2), LIGHT, MAIN), a)
        elif m == "hide":  # a trapper: the crown, ear flaps down the sides, a fleece edge
            reach = np.where(side & (z > -1.5), 7.5, drop - 1)
            a = np.where(32 - y > reach, NONE, a)
            a = np.where(~top & (32 - y > reach - 1.1) & (a != NONE), LIGHT, a)
            a = np.where(top & bands(pc, 4) & ln & (pc > 0), DARK, a)  # the crown's seams
        else:  # linen: a hood round the face
            if front:
                opening = (np.abs(x) < 3) & (y < 30)
                a = np.where(opening, NONE, a)
                a = np.where(~opening & (y < 31), LIGHT, a)  # its hem round the face
            if face == "north":
                a = np.where(bands(pc, 4) & ln & (pc > 0), DARK, a)  # the back seam's folds
    elif piece == "coat":
        if part == "arm":
            if bottom:
                return none  # the hand
            a = np.where(y < 14, NONE, a)  # the hand shows below the cuff
            cuff = (y >= 14) & (y < 15) & ~top
            a = np.where(cuff, {"fur": LIGHT, "wool": MAIN, "hide": DARK, "linen": LIGHT}[m], a)
            if m == "wool":
                a = np.where(cuff & bands(pc, 2), DARK, a)
            if m == "hide" and face == "west":
                a = np.where(bands(pr, 2) & (pc == wp // 2) & (y >= 15), LIGHT, a)  # fringe down the outer seam
        else:
            if bottom:
                a[:] = DARK if m in ("hide", "linen") else MAIN
            if m == "fur":  # open at the front: the brown lining between cream fur edges, a collar
                if front:
                    a = np.where(np.abs(x) < 1, INNER, a)
                    a = np.where((np.abs(x) >= 1) & (np.abs(x) < 2), LIGHT, a)
                    a = np.where(y > 22.9, LIGHT, a)
                if top:
                    a = np.where(np.abs(x) > 1.9, LIGHT, a)
                a = np.where(~top & ~bottom & (y < 13), LIGHT, a)  # the hem
            elif m == "wool":  # a jumper: cables, a ribbed hem and neck
                if front or side:  # the back a plain panel
                    cable = ((pc % 4 == 1) if wp >= 6 else (pc % 3 == 1)) & ln
                    a = np.where(cable, np.where(pr % 3 == 1, LIGHT, DARK), a)  # a cable and its twists
                if not (top or bottom):
                    a = np.where(y < 13, np.where(bands(pc, 2), DARK, MAIN), a)
                if front:
                    a = np.where((y > 22.9) & (np.abs(x) < 2), DARK, a)
            elif m == "hide":  # buckskin: a laced collar, a fringed yoke across the chest
                yoke = (y > 19.9) & (y < 21)
                if front:
                    a = np.where(yoke & (np.abs(x) >= 1), np.where(bands(pc, 2), LIGHT, DARK), a)
                    a = np.where((np.abs(x) < 1) & (y > 19.9), LACE, a)
                if face == "north":
                    a = np.where(yoke, np.where(bands(pc, 2), LIGHT, DARK), a)
                a = np.where(~top & ~bottom & (y < 13), DARK, a)
            else:  # linen: a shirt with a slit at the collar, soft folds
                if not (top or bottom):
                    a = np.where(bands(pc, 3) & ln & (pc > 0) & (y < 20), DARK, a)
                    a = np.where(y < 13, LIGHT, a)
                if front:
                    a = np.where((np.abs(x) < 1) & (y > 20.9), INNER, a)
                    a = np.where((np.abs(x) < 2) & (np.abs(x) >= 1) & (y > 22.9), LIGHT, a)
    elif piece == "trousers":
        if part == "body":
            if top:
                return none
            a = np.where(~bottom & (y > 15), NONE, a)  # only the hips
            band = ~bottom & (y > 14) & (a != NONE)
            a = np.where(band, {"fur": LIGHT, "wool": LIGHT, "hide": LACE, "linen": DARK}[m], a)  # the waistband, a belt
            if m == "hide" and front:
                a = np.where(band & (np.abs(x) < 1), LIGHT, a)  # its buckle
        else:
            if top:
                return none
            hem = ~bottom & (y < 1.5)
            if m == "fur":
                a = np.where(hem, LIGHT, a)
            elif m == "wool":
                a = np.where(bands(pc, 3) & ln & ~bottom, DARK, a)
                a = np.where(hem, np.where(bands(pc, 2), DARK, MAIN), a)
            elif m == "hide":
                if face == "west":  # fringe down the outer seam
                    a = np.where(pc == wp - 1, np.where(bands(pr, 2), LIGHT, DARK), a)
                if front:
                    a = np.where((pc == 0) & ln, DARK, a)  # the seam
                a = np.where(hem, DARK, a)
            else:
                if not bottom:
                    a = np.where(bands(pc, 3) & ln & (pc > 0) & (y > 3), DARK, a)  # folds
                a = np.where(hem, LIGHT, a)
    else:  # shoes
        if top:
            return none
        tall = {"fur": 5.5, "wool": 6.5, "hide": 3.2, "linen": 6.5}[m]
        if bottom:
            a[:] = SOLE
        else:
            a = np.where(y > tall, NONE, a)
            if m == "fur":
                a = np.where((y > tall - 1.2) & (y <= tall), LIGHT, a)  # the fur cuff
            elif m == "wool":  # socks: a ribbed top, a heel and toe
                a = np.where((y > tall - 2) & (y <= tall), np.where(bands(pc, 2), DARK, LIGHT), a)
                a = np.where((y < 2) & (face in ("south", "north")), DARK, a)
            elif m == "hide":  # moccasins: a stitched vamp, a turned cuff
                if front:
                    a = np.where((y > 1) & (y < 2) & bands(pc, 2), LACE, a)
                a = np.where((y > tall - 1.2) & (y <= tall), LIGHT, a)
            else:  # linen wraps, wound round and round
                a = np.where(((row + col) % (3 * K) < K) & (y >= 1) & (a != NONE), DARK, a)  # a band K texels thick
                a = np.where((y > tall - 1) & (y <= tall), LIGHT, a)
            a = np.where((y < 1) & (a != NONE), SOLE if m in ("fur", "hide") else DARK, a)
    return a


# ------------------------------------------------------------------ painting


def paint_piece(m, piece, sheet):
    """Paints one piece into its layer's sheet (H x W x 4, transparent where it doesn't cover)."""
    base, soft = PALETTE[m]
    tone = [tones(c, hh) for c, hh in zip(base, HUES)]
    for part in WORN[piece]:
        lo, hi, (u, v) = PARTS[part]
        lo, hi = np.array(lo, float), np.array(hi, float)
        size = hi - lo
        for face, (fu, fv, fw, fh) in rects(u, v, int(size[0]), int(size[1]), int(size[2])).items():
            w, h = fw * K, fh * K
            corner, Uv, Vv = face_quad(lo, hi, face)
            cc, rr = np.meshgrid(np.arange(w), np.arange(h))
            p = corner + ((cc + 0.5) / w)[..., None] * Uv + ((rr + 0.5) / h)[..., None] * Vv
            a = area(m, piece, part, face, p, cc, rr, w, h)
            if (a == NONE).all():
                continue
            occ = occlusion(p * U, np.array(NORMAL[face], float), Uv, Vv, BOXES, own=part)
            light_face = "north" if face == "down" else face  # seen only from below: not near-black
            # A hat is lit as one piece down its sides (its brim is at the bottom of each face,
            # where a face's own light runs darkest: in the game a cream brim read tan).
            down = np.full((h, w), 0.3) if piece == "hat" else None
            col = paint_face_smooth(np.where(a == NONE, MAIN, a), tone, light_face, coat=soft, y_world=p[..., 1] * U,
                                    occ=occ * 0.6, down=down, seed=sum(map(ord, m + piece + part + face)) % 97, merge=False,
                                    tuft=False)  # tufts hang two texels: on a 16-a-block hat they eat its brim
            img = np.dstack([np.clip(np.round(col), 0, 255), np.where(a == NONE, 0, 255)]).astype(np.uint8)
            region = sheet[fv * K:fv * K + h, fu * K:fu * K + w]
            region[:] = np.where(img[..., 3:] > 0, img, region)


def paint_material(m):
    layers = {1: np.zeros((32 * K, 64 * K, 4), np.uint8), 2: np.zeros((32 * K, 64 * K, 4), np.uint8)}
    for piece in ("hat", "coat", "shoes", "trousers"):
        paint_piece(m, piece, layers[LAYER[piece]])
    return layers


# ------------------------------------------------------------------ the player wearing them


def player_spec(layer_rows):
    """A render.py spec of the player in its skin (rows 0-31 of the atlas) and, for each worn
    piece, the parts it covers, inflated (`layer_rows`: [(atlas row offset, parts, inflate)])."""
    elements = []

    def add(name, src, lo, hi, off, grow, mirror):
        slo, shi, (u, v) = PARTS[src]
        size = np.array(shi) - np.array(slo)
        faces = {}
        for f, (fu, fv, fw, fh) in rects(u, v, int(size[0]), int(size[1]), int(size[2])).items():
            r = [fu * K, (fv + off) * K, (fu + fw) * K, (fv + off + fh) * K]
            faces[f] = [r[2], r[1], r[0], r[3]] if mirror else r
        if mirror:
            faces["east"], faces["west"] = faces["west"], faces["east"]
        elements.append({"name": f"{name}{off}", "bone": "root", "lo": (np.array(lo, float) - grow).tolist(),
                         "hi": (np.array(hi, float) + grow).tolist(), "faces": faces})

    for off, parts, grow in layer_rows:
        for name, (lo, hi, _) in PARTS.items():
            if name in parts:
                add(name, name, lo, hi, off, grow, False)
        for name, (src, lo, hi) in MIRRORED.items():
            if src in parts:
                add(name, src, lo, hi, off, grow, True)
    return {"bones": [{"name": "root", "parent": None, "pivot": [0, 0, 0]}], "elements": elements}


def sheet(path, painted):
    from PIL import ImageDraw, ImageFont
    from render import render
    skin = np.array(Image.open(SKIN).convert("RGBA").resize((64 * K, 32 * K), Image.NEAREST))
    font = ImageFont.load_default(18)
    cols = ["hat", "coat", "trousers", "shoes", "all four"]
    cw, ch = 130, 300
    out = Image.new("RGB", (70 + len(cols) * 2 * cw, 40 + len(painted) * ch), (236, 233, 226))
    d = ImageDraw.Draw(out)
    for k, c in enumerate(cols):
        d.text((70 + k * 2 * cw + 60, 10), c, fill=(20, 20, 20), font=font)
    for r, (m, layers) in enumerate(painted.items()):
        atlas = np.concatenate([skin, layers[1], layers[2]], 0)
        d.text((8, 40 + r * ch + 140), m, fill=(20, 20, 20), font=font)
        for k, c in enumerate(cols):
            pieces = ["shoes", "trousers", "coat", "hat"] if c == "all four" else [c if c != "shoes" else "shoes"]
            rows = [(0, list(PARTS), 0.0)] + [(32 * LAYER[p], WORN[p], INFLATE[p]) for p in pieces]
            spec = player_spec(rows)
            for j, yaw in enumerate((25, 205)):
                img = render(spec, atlas, {}, yaw, 8, 8.0, size=(cw, ch), center=np.array([0, 16.0, 0]))
                im = Image.fromarray(img)
                out.paste(im, (70 + (2 * k + j) * cw, 40 + r * ch), im)
    out.save(path)
    # Close up: each whole outfit from the front three-quarters and the back.
    big = Image.new("RGB", (4 * 2 * 260, 560), (236, 233, 226))
    d = ImageDraw.Draw(big)
    for r, (m, layers) in enumerate(painted.items()):
        atlas = np.concatenate([skin, layers[1], layers[2]], 0)
        rows = [(0, list(PARTS), 0.0)] + [(32 * LAYER[p], WORN[p], INFLATE[p]) for p in ("shoes", "trousers", "coat", "hat")]
        spec = player_spec(rows)
        d.text((r * 520 + 10, 6), m, fill=(20, 20, 20), font=font)
        for j, yaw in enumerate((30, 215)):
            img = render(spec, atlas, {}, yaw, 12, 15.0, size=(260, 540), center=np.array([0, 16.0, 0]))
            im = Image.fromarray(img)
            big.paste(im, (r * 520 + j * 260, 24), im)
    big.save(path.replace(".png", "_close.png"))


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--sheet")
    ap.add_argument("--out", default=OUT, help="where the textures go (default: the game's)")
    ap.add_argument("--k", type=int, default=K, help="texels a skin pixel (1: 64 x 32; 2: 128 x 64, 32 a block)")
    ap.add_argument("--skin", default=SKIN, help="the skin the sheet draws them on (any multiple of 64 x 32)")
    a = ap.parse_args()
    K, SKIN = a.k, a.skin
    os.makedirs(a.out, exist_ok=True)
    painted = {}
    for m in PALETTE:
        painted[m] = paint_material(m)
        for n, img in painted[m].items():
            Image.fromarray(img, "RGBA").save(os.path.join(a.out, f"{m}_{n}.png"))
    print("wrote", ", ".join(f"{m}_1/2" for m in painted), "into", a.out)
    if a.sheet:
        sheet(a.sheet, painted)
        print("sheet", a.sheet)
