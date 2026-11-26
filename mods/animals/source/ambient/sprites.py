#!/usr/bin/env python3
"""The ambient life's sprites (`src/ambient.rs`, `core/src/ambient.rs`): each bird's flying model
seen from below with its wings open, level and banked into a turn either way, and the fish's from
the side, rendered with the blocky-model skill's `render.py` from the models' own builds
(`../<model>/out`, made by their build scripts) and cut to a small, crisp square, so the sky's
birds are the land's.

The game draws each on a card turned along its flight (`particle.wgsl`'s streak): the picture's
bottom row leads and its right edge is the bird's right wing. So a bird is drawn as seen from
below with its head down the picture, and the fish with its nose down.

    ~/.venvs/blocky/bin/python sprites.py      (after the models' builds)

render.py's pitch +89 looks up from below (its -89, which the first sprites used, looks down on
the back): head up the picture and the bird's left wing on the right, so turned half round.
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
from ground import rot  # noqa: E402

OUT = os.path.join(REPO, "mods", "animals", "assets", "textures", "particle")
# Texels across: 16 a block for the birds (their spans 1.2-2 blocks), 32 for the fish (half a block).
BIRD, FISH = 32, 16
# The roll of a bird circling a thermal (a soaring eagle's turns are banked 20-30 degrees).
BANK = 25.0

# Sprite: (model, spec file, animation, a time with the wings open, banked too).
BIRDS = {
    "ambient_eagle": ("eagle", True),
    "ambient_vulture": ("vulture", True),
    "ambient_gull": ("gull", True),
    "ambient_owl": ("owl", False),
}


def crisp_square(img, size):
    box = img.getbbox()
    img = img.crop(box)
    side = max(img.size)
    sq = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    sq.paste(img, ((side - img.width) // 2, (side - img.height) // 2))
    small = sq.resize((size, size), Image.BOX)
    px = small.load()
    for y in range(size):
        for x in range(size):
            r, g, b, a = px[x, y]
            px[x, y] = (r, g, b, 255) if a >= 110 else (0, 0, 0, 0)
    return small


def shot(model, spec_file, view, roll=0.0, anim=None):
    src = os.path.join(REPO, "mods", "animals", "source", model, "out")
    spec = json.load(open(os.path.join(src, spec_file)))
    tex = np.array(Image.open(os.path.join(src, "texture.png")).convert("RGBA"))
    pose = render.pose_at(spec, anim, 0.0)
    yaw, pitch = view
    # The bird rolled about its length (+z, the way it faces): + lowers its right wing (-x).
    plain = render.view_matrix
    render.view_matrix = lambda y, p: plain(y, p) @ rot(0, 0, roll)
    try:
        return Image.fromarray(render.render(spec, tex, pose, yaw, pitch, 400 * 0.25 / 16))
    finally:
        render.view_matrix = plain


if __name__ == "__main__":
    for name, (model, banks) in BIRDS.items():
        rolls = {"": 0.0}
        if banks:
            # A turn to its left lowers the left wing.
            rolls.update({"_bank_l": -BANK, "_bank_r": BANK})
        for suffix, roll in rolls.items():
            img = shot(model, "spec_fly.json", (0, 89), roll, "flap").rotate(180)
            crisp_square(img, BIRD).save(os.path.join(OUT, f"{name}{suffix}.png"))
            print("wrote", name + suffix)
    # The fish from its side, its nose (+z) to the left at yaw 90; turned so it points down.
    img = shot("fish", "spec.json", (90, 0)).rotate(90, expand=True)
    crisp_square(img, FISH).save(os.path.join(OUT, "ambient_fish.png"))
    print("wrote ambient_fish")
