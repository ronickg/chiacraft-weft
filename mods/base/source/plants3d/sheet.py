"""Review sheet: each 3D plant beside today's flat one, in small scenes (a meadow, a sugar cane bank, crop rows).
    python3 mods/base/source/plants3d/sheet.py <out.png>"""
import json
import os
import sys

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import render as R  # noqa: E402

BASE = os.path.normpath(os.path.join(HERE, "..", "..", "assets", "textures", "block"))
PACK = os.path.normpath(os.path.join(HERE, "..", "..", "packs", "plants_3d", "base"))
GRASS = (0x91, 0xBD, 0x59)
F = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 18)
S = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 13)


def T(n):
    return Image.open(os.path.join(BASE, n + ".png")).convert("RGBA")


def M(n):
    m = json.load(open(os.path.join(PACK, "models", "block", n + ".json")))
    return m, Image.open(os.path.join(PACK, "textures", "block", n + ".png")).convert("RGBA")


def pick(name, x, z, n=3):
    return f"p3d/{name}_{(x * 7 + z * 13) % n}"


def meadow(three_d):
    s = R.Scene(px=110)
    for x in range(4):
        for z in range(3):
            s.cube((x, -1, z), T("grass_top"), T("grass_side"), tint_top=GRASS)
            if three_d:
                m, t = M(pick("tallgrass", x, z))
                s.model((x, 0, z), m, t, tint=GRASS)
            else:
                s.cross((x, 0, z), T("tallgrass"), tint=GRASS)
    return s.render((900, 640), origin=(0.32, 0.42))


def cane_bank(three_d):
    s = R.Scene(px=100)
    water = Image.new("RGBA", (32, 32), (52, 96, 160, 255))
    for x in range(4):
        s.cube((x, -1, 2), water, water)
        for z in range(2):
            s.cube((x, -1, z), T("sand"), T("sand"))
    cols = [(0, 1, 3), (1, 0, 2), (2, 1, 3), (3, 0, 1)]
    for x, z, h in cols:
        for y in range(h):
            if three_d:
                piece = 3 if h == 1 else (0 if y == 0 else (2 if y == h - 1 else 1))
                m, t = M(f"p3d/reeds_{piece}")
                s.model((x, y, z), m, t)
            else:
                s.cross((x, y, z), T("reeds"))
    return s.render((900, 800), origin=(0.32, 0.7))


def field(three_d):
    s = R.Scene(px=110)
    farm = T("farmland_moist") if os.path.exists(os.path.join(BASE, "farmland_moist.png")) else T("farmland")
    rows = [("wheat", [0, 2, 4, 5, 6, 7]), ("carrot", [0, 1, 2, 3, 3, 3]), ("beans", [0, 1, 2, 3, 3, 3])]
    for z, (crop, stages) in enumerate(rows):
        for x, st in enumerate(stages):
            s.cube((x, -1, z), farm, T("dirt"))
            name = f"{crop}_stage_{st}"
            if three_d:
                m, t = M(pick(name, x, z))
                s.model((x, 0, z), m, t)
            else:
                s.cross((x, 0, z), T(name))
    return s.render((1300, 800), origin=(0.22, 0.45))


def crop(im, bg=(159, 196, 222), m=14):
    import numpy as np
    a = np.asarray(im)[..., :3].astype(int)
    ys, xs = np.where(np.abs(a - bg).sum(-1) > 6)
    return im.crop((max(0, xs.min() - m), max(0, ys.min() - m), min(im.width, xs.max() + m), min(im.height, ys.max() + m)))


def strip(names, ground):
    """One plant a cell, flat over 3D, at a size where the texels show."""
    cells = []
    for tex, model, tint in names:
        pair = []
        for three_d in (False, True):
            sc = R.Scene(px=150)
            sc.cube((0, -1, 0), T(ground), T("dirt"), tint_top=GRASS if ground == "grass_top" else None)
            if three_d:
                m, t = M(model)
                sc.model((0, 0, 0), m, t, tint=tint)
            else:
                sc.cross((0, 0, 0), T(tex), tint=tint)
            pair.append(crop(sc.render((360, 420), origin=(0.38, 0.62))))
        cells.append((tex, pair))
    w = max(p[0].width for _, p in cells)
    h = max(p[0].height for _, p in cells) + max(p[1].height for _, p in cells)
    img = Image.new("RGB", (len(cells) * (w + 10) + 10, h + 40), (38, 34, 30))
    d = ImageDraw.Draw(img)
    for i, (tex, (a, b)) in enumerate(cells):
        x = 10 + i * (w + 10)
        d.text((x, 4), tex.replace("_stage_", " "), font=S, fill=(230, 215, 185))
        img.paste(a.convert("RGB"), (x, 22))
        img.paste(b.convert("RGB"), (x, 26 + a.height))
    return img


def main(out):
    rows = [("Meadow: tall grass", meadow), ("Sugar cane bank", cane_bank), ("Crops: wheat 0-2-4-5-6-7, carrots 0-3, beans 0-3", field)]
    imgs = []
    for t, f in rows:
        a, b = f(False), f(True)
        box = a.getbbox() if False else None
        imgs.append((t, crop(a), crop(b)))
    W = max(a.width + b.width for _, a, b in imgs) + 60
    H = sum(a.height + 40 for _, a, _ in imgs) + 70
    sheet = Image.new("RGB", (W, H), (38, 34, 30))
    d = ImageDraw.Draw(sheet)
    d.text((20, 12), "3D plants (a prototype): today's flat crossed pictures | box models", font=F, fill=(240, 228, 200))
    d.text((20, 38), "A rough render, not the game: the game adds its own light, shadows and the shader pack.", font=S, fill=(200, 180, 140))
    y = 66
    for t, a, b in imgs:
        d.text((20, y), t + "  (flat | 3D)", font=S, fill=(230, 215, 185))
        sheet.paste(a.convert("RGB"), (20, y + 18))
        sheet.paste(b.convert("RGB"), (40 + a.width, y + 18))
        y += a.height + 40
    strips = [
        strip([("tallgrass", "p3d/tallgrass_0", GRASS), ("tallgrass", "p3d/tallgrass_1", GRASS), ("reeds", "p3d/reeds_3", None), ("reeds", "p3d/reeds_1", None)], "grass_top"),
        strip([(f"wheat_stage_{i}", f"p3d/wheat_stage_{i}_0", None) for i in range(8)], "farmland"),
        strip([(f"carrot_stage_{i}", f"p3d/carrot_stage_{i}_0", None) for i in range(4)] + [(f"beans_stage_{i}", f"p3d/beans_stage_{i}_0", None) for i in range(4)], "farmland"),
    ]
    W2 = max(W, max(x.width for x in strips) + 40)
    H2 = H + sum(x.height + 30 for x in strips) + 30
    full = Image.new("RGB", (W2, H2), (38, 34, 30))
    full.paste(sheet, (0, 0))
    d = ImageDraw.Draw(full)
    y = H + 10
    d.text((20, y), "Each on its own: flat (top) | 3D (below)", font=F, fill=(240, 228, 200))
    y += 30
    for x in strips:
        full.paste(x, (20, y))
        y += x.height + 30
    full.save(out)


if __name__ == "__main__":
    main(sys.argv[1])
