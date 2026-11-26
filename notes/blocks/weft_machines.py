"""The machine faces that came with the Weft sync (11-22: assembler, rotor, floater hold, guide case and
rail, helm, thruster, wheel, music player, woodcutter) in our materials, the way `derive.py` makes the
older mechanical faces: upstream's picture doubled to 32 texels a block, so every shape stays where
the models read it, and each pixel drawn with the approved material its colour stands for (smooth
stone, dark iron, oak planks, copper, red wool, slime, glass blue), kept as light or dark as
upstream's pixel was against the rest of its material, so bevels and shadows stay.

    python3 notes/blocks/weft_machines.py OUT_DIR      # OUT_DIR/<name>.png, from dev b9b0b81f7's pictures
"""
import colorsys
import io
import os
import subprocess
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
TILES = os.path.join(HERE, "tiles")
# dev after the 11-22 sync: upstream's own pictures, before ours replaced them.
UPSTREAM = "b9b0b81f7"

FACES = [
    "assembler_front", "assembler_side", "assembler_side_going", "assembler_side_stuck",
    "floater_hold_side", "floater_hold_side_going", "floater_hold_side_stuck",
    "guide_case_arrow", "guide_case_back", "guide_case_front", "guide_case_grip", "guide_rail_end", "guide_rail_side",
    "helm_front", "helm_side", "helm_top",
    "rotor_hold_back", "rotor_hold_front", "rotor_hold_side", "rotor_hold_side_going", "rotor_hold_side_stuck",
    "rotor_side", "rotor_side_going", "rotor_side_stuck",
    "thruster_back", "thruster_back_on", "thruster_side",
    "wheel_hub", "wheel_side", "wheel_tread",
    "music_player_back", "music_player_front", "music_player_record", "music_player_side", "music_player_top",
    "woodcutter",
]


def upstream(name):
    data = subprocess.run(["git", "show", f"{UPSTREAM}:mods/base/assets/textures/block/{name}.png"], cwd=ROOT,
                          capture_output=True, check=True).stdout
    return np.asarray(Image.open(io.BytesIO(data)).convert("RGBA")).astype(float)


def tile(name):
    return np.asarray(Image.open(os.path.join(TILES, name + ".png")).convert("RGB")).astype(float)


def lum(rgb):
    return rgb @ np.array([0.3, 0.55, 0.15])


def kind(rgb):
    """The material an upstream colour stands for."""
    r, g, b = (c / 255 for c in rgb)
    h, s, v = colorsys.rgb_to_hsv(r, g, b)
    h *= 360
    if v < 0.3:
        return "socket" if v < 0.2 else "rubber"
    if s < 0.16 or (s < 0.3 and v < 0.5 and 170 <= h < 260):
        # greys, and the dark blue-grey steel of upstream's frames
        return "stone" if v > 0.42 else "iron_dark"
    if (h < 15 or h > 340) and s > 0.6 and v > 0.5:
        return "red_on" if v > 0.85 and s > 0.75 else "wool_red"
    if (h < 15 or h > 340) and s > 0.6:
        return "red_off"
    if 70 < h < 170:
        return "slime"
    if 170 <= h < 260:
        return "blue"
    if 15 <= h < 50 and s > 0.55 and v > 0.75:
        return "copper"
    if 40 <= h < 70 and v > 0.8:
        return "flame"
    if h < 60 or h > 340:
        return "wood"
    return "stone"


def materials():
    iron = tile("iron_block")
    return {
        "stone": tile("smooth_stone"),
        "iron_dark": iron * 0.42 + np.array([4, 6, 8]),
        "socket": np.full((32, 32, 3), [0x2E, 0x2A, 0x28], float),
        "rubber": np.full((32, 32, 3), [0x3A, 0x36, 0x33], float),
        "wood": tile("planks_oak"),
        "copper": tile("copper_block"),
        "wool_red": tile("wool_red"),
        "slime": tile("slime_block"),
        # glass blue, drawn bright: arrows and legs on the dark faces
        "blue": tile("glass_blue") * 0.5 + np.array([0x8A, 0xC4, 0xE0]) * 0.5,
        "red_on": np.full((32, 32, 3), [0xD8, 0x2A, 0x22], float),
        "red_off": np.full((32, 32, 3), [0x6E, 0x22, 0x1C], float),
        "flame": None,  # upstream's own colour, it is light
    }


def make(name):
    src = upstream(name)
    k = max(1, 32 // max(src.shape[:2])) if max(src.shape[:2]) < 32 else 2
    big = src.repeat(k, 0).repeat(k, 1)
    H, W = big.shape[:2]
    mats = materials()
    if name.startswith("rotor_side"):
        # the rotor's sides are built like our piston's: cobbles under an oak band
        cobble = tile("cobblestone")
        mats["stone"], mats["iron_dark"] = cobble, cobble * 0.7
    if name.startswith("wheel"):
        # a wheel's hub is dark iron, its tyre the rubber
        iron = tile("iron_block")
        mats["stone"], mats["iron_dark"] = iron * 0.62, iron * 0.45
    kinds = np.empty((H, W), object)
    for y in range(H):
        for x in range(W):
            kinds[y, x] = kind(big[y, x, :3]) if big[y, x, 3] > 0 else None
    # each material's middle brightness in this picture: a pixel keeps how far it was above or below it
    mid = {m: np.median([lum(big[y, x, :3]) for y in range(H) for x in range(W) if kinds[y, x] == m])
           for m in set(kinds.flat) if m}
    out = np.zeros((H, W, 4))
    for y in range(H):
        for x in range(W):
            m = kinds[y, x]
            if m is None:
                continue
            t = mats[m]
            if t is None:
                px = big[y, x, :3]
            else:
                base = t[y % 32, x % 32]
                rel = lum(big[y, x, :3]) / max(mid[m], 1)
                # flat materials keep upstream's shading fully; textured ones only half, so the grain shows
                strength = 1.0 if m in ("socket", "rubber", "red_on", "red_off", "blue") else 0.3 if name.startswith(("rotor_side", "wheel")) else 0.6
                px = base * (1 + (np.clip(rel, 0.5, 1.45) - 1) * strength)
            out[y, x, :3] = px
            out[y, x, 3] = big[y, x, 3]
    return out


if __name__ == "__main__":
    dst = sys.argv[1]
    os.makedirs(dst, exist_ok=True)
    for n in FACES:
        Image.fromarray(np.clip(make(n), 0, 255).round().astype(np.uint8), "RGBA").save(os.path.join(dst, n + ".png"))
    print(len(FACES), "faces to", dst)
