"""Review sheet for the vegetation layers (python3 notes/blocks/vegetation_sheet.py <out.png>): the
floor blocks as cubes and tiled, the ground plants standing on grass (the two-block grass whole),
the shrub leaves greyed and tinted beside their natural colours, the mossy sequoia log."""
import os, sys
import numpy as np
from PIL import Image, ImageDraw, ImageFont
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import blocks as b

T = b.TILES
F = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
S = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
IVORY, AMBER = (245, 228, 188), (255, 190, 69)


def t(name, kind="cube"):
    return b.tinted_view(name, kind) if os.path.exists(os.path.join(T, name + ".png")) else Image.open(name).convert("RGBA")


def tint(img, rgb):
    a = np.asarray(img).astype(float)
    a[..., :3] *= rgb / 255
    return Image.fromarray(a.clip(0, 255).astype(np.uint8), "RGBA")


def tiled(img, k=2, n=3):
    im = Image.new("RGBA", (32 * k * n,) * 2, (156, 198, 224, 255))
    s = img.resize((32 * k, 32 * k), Image.NEAREST)
    for x in range(n):
        for y in range(n):
            im.alpha_composite(s, (x * 32 * k, y * 32 * k))
    return im


def standing(cols, k=3):
    """Side view: a row of grass blocks, on each a column of plant tiles (bottom first)."""
    h = max(len(c) for c in cols) + 1
    im = Image.new("RGBA", (len(cols) * 32 * k, h * 32 * k), (156, 198, 224, 255))
    side = Image.open(os.path.join(T, "grass_side.png")).convert("RGBA").resize((32 * k,) * 2, Image.NEAREST)
    for i, col in enumerate(cols):
        im.alpha_composite(side, (i * 32 * k, (h - 1) * 32 * k))
        for j, p in enumerate(col):
            im.alpha_composite(p.resize((32 * k,) * 2, Image.NEAREST), (i * 32 * k, (h - 2 - j) * 32 * k))
    return im


sheet = Image.new("RGBA", (1500, 1480), (42, 32, 26, 255))
d = ImageDraw.Draw(sheet)
d.text((20, 14), "Vegetation layers for Plants: floor blocks, ground plants, shrub leaves, the mossy sequoia's end", font=F, fill=IVORY)
y = 56

d.text((20, y), "1. Floor blocks: leaf litter (forest), moss block (taiga, swamp edge), coarse dirt (plains, savanna)", font=F, fill=AMBER)
x = 20
for top, side, lab in (("leaf_litter_top", "leaf_litter_side", "leaf_litter"), ("moss_block_top", "moss_block_side", "moss_block"),
                       ("coarse_dirt", "coarse_dirt", "coarse_dirt (one texture, all sides)")):
    c = b.cube(t(top), t(side), 96)
    sheet.alpha_composite(c, (x, y + 34))
    sheet.alpha_composite(tiled(t(top)), (x + 200, y + 34))
    d.text((x, y + 34 + 196), lab, font=S, fill=IVORY)
    x += 200 + 192 + 70
y += 34 + 230

d.text((20, y), "2. Ground plants on grass: bracken fern, the two-block tall grass (A my pick, B, and the first try that broke up), today's tallgrass", font=F, fill=AMBER)
g = np.array([0x7F, 0xB0, 0x48])
scene = standing([[t("fern", "plant")], [tint(t("tall_grass_lower"), g), tint(t("tall_grass_upper"), g)], [],
                  [tint(t("tall_grass_lower_b"), g), tint(t("tall_grass_upper_b"), g)], [],
                  [tint(t("tall_grass_lower_v1"), g), tint(t("tall_grass_upper_v1"), g)], [], [tint(t("tallgrass"), g)]], k=3)
sheet.alpha_composite(scene, (20, y + 34))
for i, lab in ((0, "fern"), (1, "tall grass A *"), (3, "tall grass B"), (5, "first try"), (7, "tallgrass now")):
    d.text((20 + i * 96, y + 34 + scene.height + 4), lab, font=S, fill=IVORY)
y += 34 + scene.height + 40

d.text((20, y), "3. Shrub leaves: greyed and tinted by the biome as asked (top), or kept in their own colours (bottom) *", font=F, fill=AMBER)
x = 20
fol = np.array([0x6F, 0x9A, 0x3C])
for n in ("leaves_willow", "leaves_juniper", "leaves_creosote"):
    grey = tint(t(n, "leaves").copy(), np.array([255, 255, 255]))
    tinted = b.tinted_view(n, "leaves")
    nat = Image.open(os.path.join(T, n + "_natural.png")).convert("RGBA")
    sheet.alpha_composite(b.cube(tinted, tinted, 80), (x, y + 34))
    sheet.alpha_composite(b.cube(nat, nat, 80), (x, y + 34 + 180))
    sheet.alpha_composite(tiled(nat, 2, 2), (x + 175, y + 34 + 180))
    d.text((x, y + 34 + 350), n.replace("leaves_", ""), font=S, fill=IVORY)
    x += 175 + 128 + 80
y += 34 + 380

d.text((20, y), "4. The fallen giant (mossy_sequoia_log): its new end, beside the approved sequoia log", font=F, fill=AMBER)
SQ = "/Users/ronald/Coding/chiacraft-weft/notes/ui-codex/parts/deliver/sequoia/textures/block/"
sheet.alpha_composite(b.cube(t(SQ + "log_sequoia_top.png"), t(SQ + "log_sequoia.png"), 96), (20, y + 34))
sheet.alpha_composite(b.cube(t("log_sequoia_mossy_top"), t("log_sequoia_mossy"), 96), (240, y + 34))
sheet.alpha_composite(t("log_sequoia_mossy_top").resize((192, 192), Image.NEAREST), (460, y + 34))
d.text((20, y + 34 + 200), "sequoia log (approved)", font=S, fill=IVORY)
d.text((240, y + 34 + 200), "mossy log, new end", font=S, fill=IVORY)
sheet.convert("RGB").save(sys.argv[1])
print(sys.argv[1])
