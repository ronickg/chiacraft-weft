"""Puts the 64 px block tiles (tiles/, cut by grid_cut.py; and the first ones in chatgpt/ and codex/)
into the game: each over the texture of the same name in the mod that has it. A replaced texture's
old 16 px variants and connected-texture rule (`<name>.ctm`, `<name>/`) go too: mixed with the new
tile they would show as patches. Tiles for blocks the game does not have yet are left out.

    python3 notes/texture-test/install_blocks.py
"""
import os
import shutil

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..", "..")
MODS = [os.path.join(ROOT, "mods", m, "assets", "textures", "block") for m in ("base", "animals")]

# The first tiles, made before the grids: name in the game -> file here.
FIRST = {
    "stone": "chatgpt/stone_smooth_64.png",
    "cobblestone": "chatgpt/stone_64.png",
    "dirt": "codex/dirt_64.png",
    "log_oak": "codex/log_oak_64.png",
    "log_oak_top": "codex/log_oak_top_64.png",
    "leaves_oak": "codex/leaves_oak_64.png",
    "sand": "codex/sand_64.png",
    "bricks": "codex/bricks_64.png",
    "coal_ore": "codex/coal_ore_64.png",
    "grass_top": "codex/grass_top_64.png",
}


def tiles():
    out = {name: os.path.join(HERE, path) for name, path in FIRST.items()}
    folder = os.path.join(HERE, "tiles")
    for f in sorted(os.listdir(folder)) if os.path.isdir(folder) else []:
        if f.endswith(".png"):
            out[f[:-4]] = os.path.join(folder, f)
    return out


# Random variants of a texture (`<name>.ctm`, method random), from tiles/<variant>.png.
VARIANTS = {"stone": (["stone_v1", "stone_v2", "stone_v3"], "4 2 2 2")}


def variants(home, name, parts, weights):
    folder = os.path.join(home, name)
    os.makedirs(folder, exist_ok=True)
    for i, part in enumerate(parts, 1):
        shutil.copyfile(os.path.join(HERE, "tiles", part + ".png"), os.path.join(folder, f"{i}.png"))
    rule = f"method = random\ntiles = <default> {name}/1-{len(parts)}\nweights = {weights}\n"
    open(os.path.join(home, name + ".ctm"), "w").write(rule)


def glow_mask(path):
    """`<name>_e.png` beside a texture with fire in it: white where the fire is."""
    a = np.asarray(Image.open(path).convert("RGB")).astype(int)
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    fire = (r > 190) & (g > 90) & (r - b > 90)
    m = np.where(fire, 255, 0).astype(np.uint8)
    Image.fromarray(np.dstack([m, m, m, np.full_like(m, 255)]), "RGBA").save(path[:-4] + "_e.png")


def main():
    done, skipped = [], []
    for name, src in tiles().items():
        home = next((m for m in MODS if os.path.exists(os.path.join(m, name + ".png"))), None)
        if home is None:
            skipped.append(name)
            continue
        shutil.copyfile(src, os.path.join(home, name + ".png"))
        rule, old = os.path.join(home, name + ".ctm"), os.path.join(home, name)
        if os.path.exists(rule):
            os.remove(rule)
        if os.path.isdir(old):
            shutil.rmtree(old)
        if os.path.exists(os.path.join(home, name + "_e.png")):
            glow_mask(os.path.join(home, name + ".png"))
        if name in VARIANTS:
            variants(home, name, *VARIANTS[name])
        done.append(name)
    print(f"{len(done)} textures in the game: {' '.join(done)}")
    if skipped:
        print(f"no block yet for: {' '.join(skipped)}")


if __name__ == "__main__":
    main()
