#!/usr/bin/env python3
"""Paints the default player skin at 32 texels a block (128 x 64, Minecraft's classic 64 x 32
layout at twice the texels; Ronald 11-04, "C now: all 32"), the `blocky-model` way, like the
clothes worn over it (`../clothes/build.py`): every texel from where it sits on the body, the
house's light and occlusion, designed features (the face) stamped on. A friendly person in what
they wear under their clothes: brown hair, an oatmeal linen undershirt with the forearms bare, a
belt, brown breeches and shoes (the `game-art` guide `5 Monsters and Characters`: warm skin tones,
an expressive face, eyes 1-2 px with a white highlight, no black outline).

    ~/.venvs/blocky/bin/python mods/base/source/player/build.py [--out player.png] [--sheet out.png]

Writes `crates/engine/assets/textures/entity/player.png` (the engine's own default skin: a mod can't
replace an engine asset). The engine maps the player's model by its declared 64 x 32 layout, so a
128 x 64 skin needs no engine change; worn layers map by the same layout (`worn_tex_size`).
`--sheet` draws it bare and in each material's whole outfit, front and back.
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
OUT = os.path.join(REPO, "crates", "engine", "assets", "textures", "entity", "player.png")

SKIN, HAIR, SHIRT, BREECHES, BELT, SHOE, LIP = range(7)
BASE = [(204, 146, 108), (92, 60, 40), (212, 196, 164), (118, 86, 60), (124, 82, 48), (82, 58, 42), (170, 104, 88)]
HUES = [0.02, 0.02, 0.06, 0.02, 0.02, 0.02, 0.97]
TONES = [tones(c, h) for c, h in zip(BASE, HUES)]
SOFT = (HAIR,)  # strands along the hair

# The face's designed parts (texels on the 16 x 16 front of the head), stamped after the light.
WHITE, IRIS, PUPIL, GLINT, BROW = (236, 230, 220), (104, 70, 44), (40, 28, 24), (246, 244, 236), (70, 46, 32)


def area(part, face, p, col, row, w, h):
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, SKIN)
    front, back, top, bottom = face == "south", face == "north", face == "up", face == "down"
    side = face in ("east", "west")
    if part == "head":
        if top:
            a[:] = HAIR
        elif back:
            a = np.where(y > 25.5, HAIR, a)  # down to the nape
        elif side:
            # over the ears and behind them, a sideburn in front; the ear a bump of skin
            line = np.where(z < -0.5, 25.5, np.where(z < 2.5, 28.5, 29.5))
            a = np.where(y > line, HAIR, a)
        elif front:
            # a fringe, jagged: locks hanging a texel lower here and there
            jag = np.array([0, 1, 1, 0, 2, 1, 0, 0, 1, 2, 1, 0, 0, 1, 1, 0])[col % 16] * 0.5 / K
            a = np.where(y > 30.0 - jag, HAIR, a)
            a = np.where((np.abs(x) > 3.5) & (y > 28.5), HAIR, a)  # the hair at the temples
    elif part == "body":
        a[:] = SHIRT
        if front:
            a = np.where((np.abs(x) < 1.0) & (y > 22.5) & (np.abs(x) < 1.0 - (23.9 - y) * 0.6), SKIN, a)  # the neckline's V
        if top:
            a = np.where((np.abs(x) < 1.6) & (np.abs(z) < 1.2), SKIN, a)  # the neck
        a = np.where(~np.bool_(top) & ~np.bool_(bottom) & (y < 13.5), BELT, a)
        if bottom:
            a[:] = BREECHES
    elif part == "arm":
        a = np.where(y > 19, SHIRT, a)  # short sleeves; bare forearms and hands below
        if top:
            a[:] = SHIRT
    elif part == "leg":
        a[:] = BREECHES
        a = np.where(y < 2.2, SHOE, a)
        if top:
            a[:] = BREECHES
        if bottom:
            a[:] = SHOE
    return a


def stamp_face(img):
    """The face on the head's front (16 x 16 texels at K = 2): brows, eyes with whites and a
    catchlight up and in toward the light, a nose shaded below, a mouth a little darker."""
    f = img[..., :3].astype(float)
    for cols in ([3, 4, 5], [12, 11, 10]):  # each eye from its outer corner in: white, iris, pupil
        for c in cols:
            f[6, c] = BROW  # the brow
            f[7, c] = f[7, c] * 0.88  # the lid's shade under it
        f[8, cols[0]], f[8, cols[1]], f[8, cols[2]] = WHITE, GLINT, PUPIL
        f[9, cols[0]], f[9, cols[1]], f[9, cols[2]] = WHITE, IRIS, PUPIL
    f[10:12, 7:9] *= 0.9  # the nose's sides
    f[12, 7:9] *= 0.82  # its shadow below
    f[13, 6:10] = np.array(BASE[LIP]) * 0.95  # the mouth
    img[..., :3] = np.clip(np.round(f), 0, 255).astype(np.uint8)


def paint_skin():
    sheet = np.zeros((32 * K, 64 * K, 4), np.uint8)
    for part, (lo, hi, (u, v)) in clothes.PARTS.items():
        lo, hi = np.array(lo, float), np.array(hi, float)
        size = hi - lo
        for face, (fu, fv, fw, fh) in clothes.rects(u, v, int(size[0]), int(size[1]), int(size[2])).items():
            w, h = fw * K, fh * K
            corner, Uv, Vv = face_quad(lo, hi, face)
            cc, rr = np.meshgrid(np.arange(w), np.arange(h))
            p = corner + ((cc + 0.5) / w)[..., None] * Uv + ((rr + 0.5) / h)[..., None] * Vv
            a = area(part, face, p, cc, rr, w, h)
            occ = occlusion(p * clothes.U, np.array(clothes.NORMAL[face], float), Uv, Vv, clothes.BOXES, own=part)
            down = np.full((h, w), 0.35) if part == "head" and face not in ("up", "down") else None  # a face lit evenly
            col = paint_face_smooth(a, TONES, "north" if face == "down" else face, coat=SOFT, y_world=p[..., 1] * clothes.U,
                                    occ=occ * 0.6, down=down, seed=sum(map(ord, part + face)) % 97, merge=False, tuft=False)
            img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
            if part == "head" and face == "south":
                stamp_face(img)
            sheet[fv * K:fv * K + h, fu * K:fu * K + w] = img
    return sheet


def sheet_png(path, skin):
    from PIL import ImageDraw, ImageFont
    from render import render
    clothes.K = K
    font = ImageFont.load_default(18)
    outfits = [("bare", None)] + [(m, clothes.paint_material(m)) for m in clothes.PALETTE]
    cw, ch = 230, 470
    out = Image.new("RGB", (len(outfits) * 2 * cw, ch + 30), (236, 233, 226))
    d = ImageDraw.Draw(out)
    empty = np.zeros_like(skin)
    for k, (name, layers) in enumerate(outfits):
        atlas = np.concatenate([skin, layers[1] if layers else empty, layers[2] if layers else empty], 0)
        pieces = [] if layers is None else ["shoes", "trousers", "coat", "hat"]
        rows = [(0, list(clothes.PARTS), 0.0)] + [(32 * clothes.LAYER[p], clothes.WORN[p], clothes.INFLATE[p]) for p in pieces]
        spec = clothes.player_spec(rows)
        d.text((k * 2 * cw + 10, 6), name, fill=(20, 20, 20), font=font)
        for j, yaw in enumerate((25, 205)):
            img = render(spec, atlas, {}, yaw, 8, 13.0, size=(cw, ch), center=np.array([0, 16.0, 0]))
            im = Image.fromarray(img)
            out.paste(im, ((2 * k + j) * cw, 30), im)
    out.save(path)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=OUT)
    ap.add_argument("--sheet")
    a = ap.parse_args()
    skin = paint_skin()
    Image.fromarray(skin, "RGBA").save(a.out)
    print("wrote", a.out, skin.shape[1], "x", skin.shape[0])
    if a.sheet:
        sheet_png(a.sheet, skin)
        print("sheet", a.sheet)
