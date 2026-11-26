"""After `blocks.py cut batches/weft_plants.tsv`: upstream's plant textures (Weft sync 11-22) laid out the
way the game loads them, from our cut tiles.

- magenta fringe off every cutout tile's edge;
- plants that stand on the ground dropped to the tile's bottom row (a few were drawn floating);
- the random-variant folders upstream uses (`short_grass/1..4.png` and the rest, `<name>.ctm` kept);
- `inula.png`, `verbascum.png`, `lavender.png`: the top over the bottom, 32 x 64, as upstream's
  16 x 32 picture of the whole plant;
- `moss.png`: our moss block's top (upstream's moss layer is that moss);
- the three hollow logs' insides: our stripped log of the same wood in the shade.

    python3 notes/blocks/weft_plants_post.py OUT_DIR     # OUT_DIR mirrors textures/block
"""
import colorsys
import os
import shutil
import sys

from PIL import Image, ImageEnhance

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
TILES = os.path.join(HERE, "tiles")
BLOCK = os.path.join(ROOT, "mods", "base", "assets", "textures", "block")

FOLDERS = {
    "short_grass": 4, "tall_tallgrass_top": 4, "bluegrass": 2,
    "porcini_mushroom": 3, "chanterelle_mushroom": 2, "toadstool_mushroom": 3, "dryads_saddle_mushroom": 2,
}
SINGLE = ["inula_top", "inula_bottom", "verbascum_top", "verbascum_bottom", "lavender_top", "lavender_bottom",
          "tall_tallgrass_bottom", "bush_stem", "raspberry_leaves", "raspberry_leaves_berried", "snowberry_leaves_berried",
          "tulip_red", "tulip_orange", "tulip_yellow", "tulip_pink", "tulip_purple", "tulip_white"]
# standing on the ground: dropped to the bottom row
GROUNDED = ("short_grass", "bluegrass", "porcini_mushroom", "chanterelle_mushroom", "toadstool_mushroom", "tulip_")


def magenta(p):
    if p[3] == 0:
        return False
    h, s, v = colorsys.rgb_to_hsv(*(c / 255 for c in p[:3]))
    return 285 <= h * 360 <= 340 and s > 0.3 and v > 0.3


def defringe(im):
    p = im.load()
    W, H = im.size
    for _ in range(2):
        for y in range(H):
            for x in range(W):
                if magenta(p[x, y]):
                    nb = [p[x + dx, y + dy] for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))
                          if 0 <= x + dx < W and 0 <= y + dy < H]
                    inner = [q for q in nb if q[3] and not magenta(q)]
                    edge = any(q[3] == 0 for q in nb)
                    p[x, y] = (0, 0, 0, 0) if edge or not inner else inner[0]
    return im


def drop(im):
    box = im.getbbox()
    if not box or box[3] >= im.height:
        return im
    out = Image.new("RGBA", im.size)
    out.alpha_composite(im, (0, im.height - box[3]))
    return out


def tile(name):
    im = Image.open(os.path.join(TILES, name + ".png")).convert("RGBA")
    # pink and purple petals share magenta's hue: those two are recoloured from the clean red one
    if name not in ("tulip_pink", "tulip_purple"):
        im = defringe(im)
    return drop(im) if name.startswith(GROUNDED) else im


def main(out):
    os.makedirs(out, exist_ok=True)
    for folder, n in FOLDERS.items():
        os.makedirs(os.path.join(out, folder), exist_ok=True)
        for i in range(1, n + 1):
            tile(f"{folder}_{i}").save(os.path.join(out, folder, f"{i}.png"))
        shutil.copy(os.path.join(BLOCK, folder, folder + ".ctm"), os.path.join(out, folder, folder + ".ctm"))
    for name in SINGLE:
        tile(name).save(os.path.join(out, name + ".png"))
    for plant in ("inula", "verbascum", "lavender"):
        whole = Image.new("RGBA", (32, 64))
        whole.alpha_composite(tile(plant + "_top"), (0, 0))
        whole.alpha_composite(tile(plant + "_bottom"), (0, 32))
        whole.save(os.path.join(out, plant + ".png"))
    shutil.copy(os.path.join(BLOCK, "moss_block_top.png"), os.path.join(out, "moss.png"))
    for wood in ("oak", "birch", "spruce"):
        im = ImageEnhance.Brightness(Image.open(os.path.join(BLOCK, f"log_{wood}_stripped.png")).convert("RGB")).enhance(0.82)
        r, g, b = im.split()
        Image.merge("RGB", (r, g, b.point(lambda v: int(v * 0.92)))).save(os.path.join(out, f"{wood}_log_inner.png"))
    print("plants to", out)


if __name__ == "__main__":
    main(sys.argv[1])
