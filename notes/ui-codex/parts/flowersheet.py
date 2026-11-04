"""The 45 biome flowers (Voxel dog's list, the Researcher's species): each beside the placeholder it
replaces, on a grass tile, grouped by biome. Also writes deliver/flowers/<name>.png (32 px, the game's
names) for the Voxel dog handoff.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/flowersheet.py   # sheets/flowers.png
"""
import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, HERE)
import parts  # noqa: E402

BLOCK = os.path.join(ROOT, "mods", "base", "assets", "textures", "block")
BIOMES = [("plains", "purple_coneflower black_eyed_susan butterfly_milkweed leadplant blazing_star"),
          ("forest", "wood_anemone lesser_celandine lungwort sweet_woodruff foxglove"),
          ("birch forest", "lily_of_the_valley wood_cranesbill may_lily bellflower fireweed"),
          ("taiga", "twinflower bunchberry prickly_rose labrador_tea"),
          ("snowy plains", "arctic_poppy lousewort saxifrage arctic_coltsfoot pygmy_buttercup"),
          ("desert", "brittlebush globemallow gold_poppy desert_marigold ocotillo"),
          ("savanna", "sensitive_plant senna pencil_flower morning_glory"),
          ("swamp", "water_primrose alligator_flag"),
          ("mountains", "alpenrose gentian edelweiss globeflower alpine_aster"),
          ("beach", "sea_daffodil sea_holly horned_poppy sea_rocket hottentot_fig")]
GROUND = {"desert": "sand", "beach": "sand", "snowy plains": "snow"}


def ground(biome):
    n = GROUND.get(biome, "grass_top")
    g = Image.open(os.path.join(BLOCK, n + ".png")).convert("RGBA")
    if n == "grass_top":
        a = np.array(g).astype(float); a[..., :3] *= np.array([0x91, 0xBD, 0x59]) / 255
        g = Image.fromarray(a.clip(0, 255).astype(np.uint8))
    return g


def main():
    out_dir = os.path.join(HERE, "deliver", "flowers")
    os.makedirs(out_dir, exist_ok=True)
    S, pad = 96, 10
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 13)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 18)
    W = 14 + 5 * (2 * S + pad + 24)
    im = Image.new("RGB", (W, 60 + len(BIOMES) * (S + 70)), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    d.text((14, 12), "The 45 biome flowers: placeholder (left of each pair) and the new flower (right), on their ground",
           fill=(0xf5, 0xe4, 0xbc), font=big)
    y = 46
    checks = {}
    for biome, names in BIOMES:
        d.text((14, y), biome, fill=(0xff, 0xbe, 0x45), font=big)
        y += 24
        g = ground(biome).resize((S, S), Image.NEAREST)
        for k, n in enumerate(names.split()):
            x = 14 + k * (2 * S + pad + 24)
            px = os.path.join(HERE, "out", f"flower_{n}_px.png")
            new = Image.open(px).convert("RGBA") if os.path.exists(px) else None
            old = Image.open(os.path.join(BLOCK, n + ".png")).convert("RGBA")
            for c, img in enumerate((old, new)):
                im.paste(g, (x + c * (S + 4), y))
                if img is not None:
                    t = img.resize((S, S), Image.NEAREST)
                    im.paste(t, (x + c * (S + 4), y), t)
            d.text((x, y + S + 2), n.replace("_", " "), fill=(0xc2, 0xac, 0x88), font=font)
            if new is not None:
                p = os.path.join(out_dir, n + ".png")
                new.save(p)
                parts.art.archive(p, f"flower_{n}", "flowers")
                cj = os.path.join(HERE, "out", f"flower_{n}_checks.json")
                if os.path.exists(cj):
                    t = json.load(open(cj))["tries"]
                    checks[n] = bool(t and t[-1].get("passed"))
        y += S + 46
    out = os.path.join(HERE, "sheets", "flowers.png")
    im = im.crop((0, 0, W, y))
    im.save(out)
    parts.art.archive(out, "sheet_flowers", "sheets")
    print(out, im.size, "checks failed:", [n for n, ok in checks.items() if not ok])


if __name__ == "__main__":
    main()
