#!/usr/bin/env python3
"""Paints the cloaks worn over clothes and armour (`clothing.rs`: the cloak slot, body and arms,
1.2 out from the body) at 32 texels a block, with the clothes painter (`build.py`): each texel from
where it sits on the body, the house's light and occlusion.

- wool cloak: loden green wool hanging from the shoulders down the back and over the arms, open at
  the front, a lighter woven border and a brass clasp at the collar;
- fur robe: a heavy brown fur robe closed down the front, a deep cream fur collar and cuffs;
- rain cape: an ochre oilskin cape to the waist at the front and down the back, its seams darker,
  a waxy sheen; no hood (the cloak slot doesn't cover the head).

    ~/.venvs/blocky/bin/python mods/base/source/clothes/cloaks.py [--out dir] [--sheet out.png]

Writes `mods/base/assets/textures/entity/clothes/{wool_cloak,fur_robe,rain_cape}.png` (128 x 64).
"""

import argparse
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import build as clothes  # noqa: E402
from paint import occlusion, paint_face_smooth, tones  # noqa: E402  (the skill's, on build's path)
from render import face_quad  # noqa: E402

K = 2
OUT = clothes.OUT
INFLATE = 1.2  # clothing.rs CLOAK_INFLATE
PARTS = ["body", "arm"]  # player.rs: the cloak slot's parts

NONE, MAIN, DARK, LIGHT, TRIM, LINING = range(-1, 5)
# Per cloak: MAIN, DARK, LIGHT, TRIM, LINING, and which are soft (fur strands).
PALETTE = {
    "wool_cloak": ([(98, 112, 80), (70, 82, 58), (150, 156, 112), (214, 172, 84), (120, 92, 66)], ()),
    "fur_robe": ([(126, 92, 66), (92, 64, 46), (236, 214, 176), (214, 172, 84), (104, 74, 54)], (MAIN, LIGHT)),
    "rain_cape": ([(170, 140, 72), (126, 100, 50), (214, 190, 120), (98, 76, 46), (140, 112, 60)], ()),
}
HUES = [0.04, 0.04, 0.08, 0.06, 0.02]


def area(m, part, face, p, col, row, w, h):
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, MAIN)
    front, back, top, bottom = (np.bool_(v) for v in (face == "south", face == "north", face == "up", face == "down"))
    side = np.bool_(face in ("east", "west"))
    ax = np.abs(x)
    pc = col // K

    if part == "arm":
        if bottom:
            return np.full(x.shape, NONE)
        # draped over the shoulder and down the outside of the arm, open on its inner side
        lowest = {"wool_cloak": 14.5, "fur_robe": 14.0, "rain_cape": 16.0}[m]
        a = np.where(y < lowest, NONE, a)
        if face == "east":  # the arm's inner side (the right arm's +x faces the body)
            a = np.where(y < 22.5, NONE, a)
        if m == "fur_robe":
            a = np.where((y >= lowest) & (y < lowest + 1.0) & ~top, LIGHT, a)  # a fur cuff
        else:
            a = np.where((y >= lowest) & (y < lowest + 0.5) & ~top, DARK if m == "rain_cape" else LIGHT, a)  # the hem
        if m == "rain_cape" and not top:
            a = np.where((a == MAIN) & ((col + row) % (10 * K) == K) & (row < h * 0.5), LIGHT, a)  # the oilskin's sheen
        return a

    # the body
    if bottom:
        return np.full(x.shape, NONE) if m == "rain_cape" else np.full(x.shape, LINING)
    if m == "wool_cloak":
        if front:
            a = np.where(ax < 2.2, NONE, a)  # open down the front
            a = np.where((ax >= 2.2) & (ax < 2.7), LIGHT, a)  # the woven border
            a = np.where((y > 22.6) & (ax < 2.7), np.where(ax < 0.75, TRIM, LIGHT), a)  # the collar and its clasp
            a = np.where((y > 22.6) & (ax < 0.75), TRIM, a)
        if top:
            a = np.where(ax < 1.6, NONE, a)  # the neck
        if back:
            a = np.where(((pc % 3) == 1) & (y < 21), DARK, a)  # folds hanging down the back
        a = np.where(~top & (y < 12.6) & (a != NONE), LIGHT, a)  # the border at the hem
    elif m == "fur_robe":
        if front:
            a = np.where((ax < 0.3) & (y < 17.4), DARK, a)  # the overlap down the front, below the sash
            a = np.where(y > 21.4, LIGHT, a)  # a deep fur collar
            a = np.where((y > 17.4) & (y < 18.2), DARK, a)  # a sash
        if top:
            a = np.where(ax < 1.6, NONE, LIGHT)  # the collar round the neck
        if back:
            a = np.where(y > 22.4, LIGHT, a)
        a = np.where(~top & (y < 13.0) & (a != NONE), LIGHT, a)  # the fur hem
    else:  # rain cape
        if front:
            a = np.where(ax < 1.4, NONE, a)  # open at the throat and down the middle
            a = np.where((ax >= 1.4) & (ax < 1.9), DARK, a)  # its seamed edge
            a = np.where((y > 22.6) & (ax < 1.9), DARK, a)  # the collar
            a = np.where(front & (y < 17.0), NONE, a)  # to the waist at the front
        if top:
            a = np.where(ax < 1.6, NONE, a)
        if back or side:
            a = np.where((pc % 4) == 0, DARK, a)  # the seams between its panels
        a = np.where(~front & ~top & (y < 12.6) & (a != NONE), DARK, a)
        if not top:
            a = np.where((a == MAIN) & ((col + row) % (10 * K) == K) & (row < h * 0.5), LIGHT, a)  # the sheen, high up
    return a


def paint_cloak(m):
    base, soft = PALETTE[m]
    tone = [tones(c, hh) for c, hh in zip(base, HUES)]
    sheet = np.zeros((32 * K, 64 * K, 4), np.uint8)
    for part in PARTS:
        lo, hi, (u, v) = clothes.PARTS[part]
        lo, hi = np.array(lo, float), np.array(hi, float)
        size = hi - lo
        for face, (fu, fv, fw, fh) in clothes.rects(u, v, int(size[0]), int(size[1]), int(size[2])).items():
            w, h = fw * K, fh * K
            corner, Uv, Vv = face_quad(lo, hi, face)
            cc, rr = np.meshgrid(np.arange(w), np.arange(h))
            p = corner + ((cc + 0.5) / w)[..., None] * Uv + ((rr + 0.5) / h)[..., None] * Vv
            a = area(m, part, face, p, cc, rr, w, h)
            if (a == NONE).all():
                continue
            occ = occlusion(p * clothes.U, np.array(clothes.NORMAL[face], float), Uv, Vv, clothes.BOXES, own=part)
            col = paint_face_smooth(np.where(a == NONE, MAIN, a), tone, "north" if face == "down" else face, coat=soft,
                                    y_world=p[..., 1] * clothes.U, occ=occ * 0.6,
                                    seed=sum(map(ord, m + part + face)) % 97, merge=False, tuft=False)
            img = np.dstack([np.clip(np.round(col), 0, 255), np.where(a == NONE, 0, 255)]).astype(np.uint8)
            sheet[fv * K:fv * K + h, fu * K:fu * K + w] = img
    return sheet


def sheet_png(path, cloaks):
    from PIL import ImageDraw, ImageFont
    from render import render
    clothes.K = K
    skin = np.array(Image.open(os.path.join(clothes.REPO, "crates", "engine", "assets", "textures", "entity", "player.png"))
                    .convert("RGBA").resize((64 * K, 32 * K), Image.NEAREST))
    tex = lambda n: np.array(Image.open(os.path.join(OUT, n + ".png")).convert("RGBA").resize((64 * K, 32 * K), Image.NEAREST))  # noqa: E731
    under = {"over the linen clothes": ("linen", None), "over clothes and iron armour": ("wool", "iron")}
    font = ImageFont.load_default(18)
    cw, ch = 210, 450
    cols = [("bare", None, None)] + [(k, c, a) for k, (c, a) in under.items()]
    out = Image.new("RGB", (120 + len(cols) * 2 * cw, 40 + len(cloaks) * ch), (236, 233, 226))
    d = ImageDraw.Draw(out)
    for k, (label, _, _) in enumerate(cols):
        d.text((120 + k * 2 * cw + 20, 10), f"{label}: front, back", fill=(20, 20, 20), font=font)
    for r, (m, cloak) in enumerate(cloaks.items()):
        d.text((10, 40 + r * ch + ch // 2), m.replace("_", " "), fill=(20, 20, 20), font=font)
        for k, (label, cl, armour) in enumerate(cols):
            layers = [skin]
            rows = [(0, list(clothes.PARTS), 0.0)]
            if cl:
                layers += [tex(f"{cl}_1"), tex(f"{cl}_2")]
                rows += [(32 * clothes.LAYER[p], clothes.WORN[p], clothes.INFLATE[p]) for p in ("shoes", "trousers", "coat", "hat")]
            if armour:
                arm = [np.array(Image.open(os.path.join(clothes.REPO, "mods", "base", "assets", "textures", "entity", "armor", f"{armour}_{n}.png"))
                                .convert("RGBA").resize((64 * K, 32 * K), Image.NEAREST)) for n in (1, 2)]
                base = 32 * (len(layers) - 1)
                layers += arm
                rows += [(base + 32, ["head"], 1.0), (base + 32, ["body", "arm"], 1.0), (base + 64, ["body", "leg"], 0.5), (base + 32, ["leg"], 1.0)]
            rows += [(32 * len(layers), PARTS, INFLATE)]
            layers.append(cloak)
            atlas = np.concatenate(layers, 0)
            spec = clothes.player_spec(rows)
            for j, yaw in enumerate((25, 205)):
                img = render(spec, atlas, {}, yaw, 8, 12.0, size=(cw, ch), center=np.array([0, 16.0, 0]))
                im = Image.fromarray(img)
                out.paste(im, (120 + (2 * k + j) * cw, 40 + r * ch), im)
    out.save(path)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=OUT)
    ap.add_argument("--sheet")
    a = ap.parse_args()
    cloaks = {m: paint_cloak(m) for m in PALETTE}
    os.makedirs(a.out, exist_ok=True)
    for m, img in cloaks.items():
        Image.fromarray(img, "RGBA").save(os.path.join(a.out, f"{m}.png"))
    print("wrote", ", ".join(cloaks), "into", a.out)
    if a.sheet:
        sheet_png(a.sheet, cloaks)
        print("sheet", a.sheet)
