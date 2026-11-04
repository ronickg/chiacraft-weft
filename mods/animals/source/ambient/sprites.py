#!/usr/bin/env python3
"""The ambient life's sprites (`src/ambient.rs`, `core/src/ambient.rs`): each bird's flying model
seen from below with its wings open, the fish's from the side, rendered with the blocky-model
skill's `render.py` from the models' own builds (`../<model>/out`, made by their build scripts)
and cut to a small, crisp square, so the sky's birds are the land's.

    ~/.venvs/blocky/bin/python sprites.py      (after the models' builds)
"""

import json
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
sys.path.insert(0, os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts"))
import render  # noqa: E402  the skill's renderer, called with views of our own
OUT = os.path.join(REPO, "mods", "animals", "assets", "textures", "particle")
SIZE = 16

# Sprite: (model, spec file, view, animation, a time with the wings open).
SPRITES = {
    "ambient_eagle": ("eagle", "spec_fly.json", (0, -89), "flap", 0.0),
    "ambient_vulture": ("vulture", "spec_fly.json", (0, -89), "flap", 0.0),
    "ambient_gull": ("gull", "spec_fly.json", (0, -89), "flap", 0.0),
    "ambient_owl": ("owl", "spec_fly.json", (0, -89), "flap", 0.0),
    "ambient_fish": ("fish", "spec.json", (90, 0), None, 0.0),
}


def crisp_square(img):
    box = img.getbbox()
    img = img.crop(box)
    side = max(img.size)
    sq = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    sq.paste(img, ((side - img.width) // 2, (side - img.height) // 2))
    small = sq.resize((SIZE, SIZE), Image.BOX)
    px = small.load()
    for y in range(SIZE):
        for x in range(SIZE):
            r, g, b, a = px[x, y]
            px[x, y] = (r, g, b, 255) if a >= 110 else (0, 0, 0, 0)
    return small


if __name__ == "__main__":
    for name, (model, spec_file, (yaw, pitch), anim, t) in SPRITES.items():
        src = os.path.join(REPO, "mods", "animals", "source", model, "out")
        spec = json.load(open(os.path.join(src, spec_file)))
        tex = np.array(Image.open(os.path.join(src, "texture.png")).convert("RGBA"))
        shot = render.render(spec, tex, render.pose_at(spec, anim, t), yaw, pitch, 400 * 0.25 / 16)
        crisp_square(Image.fromarray(shot)).save(os.path.join(OUT, f"{name}.png"))
        print("wrote", name)
