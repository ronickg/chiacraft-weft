"""Giant sequoia art for Ronald's new biome ("the massive Giant sequoia as i find them amazing"):
bark (with variants for a huge trunk), the cut log end, foliage (greyed for the biome tint, like every
leaves texture), redwood planks, the sapling and the cone. Writes deliver/sequoia/textures/ in base's
names and sheets/sequoia.png: the faces, the sequoia beside our woods and leaves, a 5 x 5 mock trunk
so the bark reads at scale, the sapling on grass and the cone in a slot. Nothing installed.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/sequoiasheet.py
"""
import os
import random
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, HERE)
import iso  # noqa: E402
import parts  # noqa: E402
from variants import grey, match_colour  # noqa: E402

OUT = os.path.join(HERE, "out")
BLOCK = os.path.join(ROOT, "mods", "base", "assets", "textures", "block")
D = os.path.join(HERE, "deliver", "sequoia", "textures")
# log_sequoia_top_2: the first log end showed no rings at all
# The log end: three drawn tries (_px, _2, _3) never showed clear rings; _4 is our approved mesquite log
# end (dense rings) repainted in sequoia colours by brightness, so its rings are drawn ones.
TOP = "log_sequoia_top_4"
TINT = 0x5f8f78   # proposed sequoia foliage tint: blue-green, bluer and greyer than the spruce's
WOODS = ["oak", "birch", "spruce", "acacia", "mesquite"]
FOLIAGE = {"oak": 0x77ab2f, "birch": 0x80a755, "spruce": 0x59ae30, "acacia": 0xaea42a, "mesquite": 0x8f9a3c}


def rgb(h):
    return np.array([(h >> 16) & 255, (h >> 8) & 255, h & 255]) / 255


def load(n):
    p = os.path.join(OUT, n + "_px.png")
    return np.array(Image.open(p).convert("RGBA")) if os.path.exists(p) else None


def block(n):
    return np.array(Image.open(os.path.join(BLOCK, n + ".png")).convert("RGBA"))


def tint(a, h):
    out = a.astype(float).copy()
    out[..., :3] *= rgb(h)
    return out.clip(0, 255).astype(np.uint8)


def save(a, rel):
    p = os.path.join(D, rel)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    Image.fromarray(a).save(p)
    parts.art.archive(p, "sequoia__" + rel.replace("/", "__")[:-4], "sequoia")


def main():
    bark = load("log_sequoia")
    # The drawn variants (var_log_sequoia_1-3, kept in out/ and the archive) came out as fine noisy
    # streaks, far busier than the bark's bold furrows. The bark tiles both ways, so shifted copies stay
    # seamless and match it exactly while moving every furrow: those are the variants.
    barks = [bark] + [np.roll(np.roll(bark, dx, 1), dy, 0) for dx, dy in ((11, 16), (21, 7), (5, 23))]
    top, leaves_c, planks, sapling, cone = (load(n) for n in (TOP, "leaves_sequoia", "planks_sequoia", "sapling_sequoia", "sequoia_cone"))
    m = leaves_c[..., 3] > 0
    leaves = leaves_c.copy()
    leaves[..., :3] = grey(leaves_c[..., :3].astype(float), m).clip(0, 255).astype(np.uint8)
    leaves[~m] = 0
    save(bark, "block/log_sequoia.png")
    for k, b in enumerate(barks[1:], 1):
        save(b, f"block/log_sequoia/{k}.png")
    if len(barks) > 1:
        p = os.path.join(D, "block", "log_sequoia.ctm")
        open(p, "w").write(f"# The sequoia's bark picked at random, so a trunk five blocks wide doesn't show its tile repeating.\n"
                           f"method = random\ntiles = <default> log_sequoia/1-{len(barks) - 1}\nweights = 4{' 2' * (len(barks) - 1)}\n")
    save(top, "block/log_sequoia_top.png")
    save(leaves, "block/leaves_sequoia.png")
    save(planks, "block/planks_sequoia.png")
    save(sapling, "block/sapling_sequoia.png")
    save(cone, "item/sequoia_cone.png")
    leaves_t = tint(leaves, TINT)

    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    W = 1500
    im = Image.new("RGB", (W, 4200), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    d.text((14, 12), "Giant sequoia: bark, log end, foliage, redwood planks, sapling and cone (for Ronald's pick)",
           fill=(0xf5, 0xe4, 0xbc), font=big)
    y = 48

    def tile(a, x, y, label, s=112, bg=(0x35, 0x20, 0x15)):
        d.rectangle([x, y, x + s - 1, y + s - 1], fill=bg)
        t = Image.fromarray(a).resize((s, s), Image.NEAREST)
        im.paste(t, (x, y), t)
        d.text((x, y + s + 2), label, fill=(0xc2, 0xac, 0x88), font=font)

    d.text((14, y), "1. The faces", fill=(0xff, 0xbe, 0x45), font=big)
    y += 28
    row = [(b, "bark" if k == 0 else f"bark variant {k}") for k, b in enumerate(barks)]
    row += [(top, "log end"), (planks, "planks"), (leaves_t, f"leaves (tint #{TINT:06x})"), (leaves, "leaves as stored (grey)")]
    for k, (a, lab) in enumerate(row):
        tile(a, 14 + k * 124, y, lab)
    y += 150
    tile(sapling, 14, y, "sapling", bg=(0x55, 0x80, 0x40))
    tile(cone, 138, y, "cone (item)")
    sm = Image.fromarray(cone).resize((32, 32), Image.NEAREST)
    d.rectangle([262, y, 297, y + 35], fill=(0x35, 0x20, 0x15))
    im.paste(sm, (264, y + 2), sm)
    d.text((262, y + 40), "slot size", fill=(0xc2, 0xac, 0x88), font=font)
    y += 160

    d.text((14, y), "2. Beside our woods: bark, log end, planks, leaves (each tinted as in its biome)", fill=(0xff, 0xbe, 0x45), font=big)
    y += 28
    cols = WOODS + ["sequoia"]
    for c, w in enumerate(cols):
        x = 14 + c * 130
        d.text((x, y), w, fill=(0xff, 0xbe, 0x45) if w == "sequoia" else (0xc2, 0xac, 0x88), font=font)
        faces = ([bark, top, planks, leaves_t] if w == "sequoia" else
                 [block(f"log_{w}"), block(f"log_{w}_top"), block(f"planks_{w}"), tint(block(f"leaves_{w}"), FOLIAGE[w])])
        for r, a in enumerate(faces):
            t = Image.fromarray(a).resize((112, 112), Image.NEAREST)
            d.rectangle([x, y + 20 + r * 118, x + 111, y + 20 + r * 118 + 111], fill=(0x35, 0x20, 0x15))
            im.paste(t, (x, y + 20 + r * 118), t)
    y += 20 + 4 * 118 + 20

    d.text((14, y), "3. A trunk 5 blocks wide, as the tree will stand: the bark at scale (variants mixed), foliage above",
           fill=(0xff, 0xbe, 0x45), font=big)
    y += 28
    rnd = random.Random(5)
    pick = lambda: rnd.choices(barks, [4] + [2] * (len(barks) - 1))[0]
    g = tint(block("grass_top"), 0x6ba941)
    dirt = block("dirt")
    s = iso.Scene()
    for gx in range(-1, 7):
        for gz in range(-1, 7):
            s.box((gx, 0, gz), top=g, side=dirt)
    for yy in range(1, 6):
        for gx in range(5):
            for gz in range(5):
                if gx in (0, 4) or gz in (0, 4):
                    s.box((gx, yy, gz), top=top, side=pick())
    for gx in range(-1, 6):
        for gz in range(-1, 6):
            if (gx - 2) ** 2 + (gz - 2) ** 2 <= 12:
                s.box((gx, 6, gz), top=leaves_t, side=leaves_t)
    img = s.render(2)
    im.paste(img, (14, y), img)
    # the same trunk face-on, 5 x 6 blocks, at 2x
    face = Image.new("RGBA", (5 * 32, 6 * 32))
    for j in range(6):
        for i in range(5):
            face.paste(Image.fromarray(pick()), (i * 32, j * 32))
    face = face.resize((face.width * 2, face.height * 2), Image.NEAREST)
    im.paste(face, (img.width + 50, y), face)
    d.text((img.width + 50, y + face.height + 4), "one face of the trunk, 5 wide x 6 tall", fill=(0xc2, 0xac, 0x88), font=font)
    y += max(img.height, face.height + 30) + 20
    out = os.path.join(HERE, "sheets", "sequoia.png")
    im = im.crop((0, 0, W, y))
    im.save(out)
    parts.art.archive(out, "sheet_sequoia", "sheets")
    print(out, im.size, len(barks) - 1, "bark variants")


if __name__ == "__main__":
    main()
