"""The block textures made in code from the approved 32 px tiles rather than generated: their
shapes must stay where the models read them, or they are another tile recoloured.

    python3 notes/blocks/derive.py        # writes tiles/, then: blocks.py install batches/derived.tsv

- Worn farmland (`farmland_worn_1..3`): the farmland, paler and duller as it wears out.
- Glass: the generated glare streaks made pale; stained glass: the glass frame recoloured, its inside the dye colour, see-through.
- Water, lava and the amber gateway: 16 frames of a seamless pattern that drifts and loops (`<name>.anim` stays).
- Mechanical faces (beds, torches, lever, button, redstone parts, the anchor, the floater): the
  old 16 px picture doubled, each of its colours drawn with the matching approved material (oak,
  wool, smooth stone) so the shapes stay where the models expect them.
"""
import os
import subprocess

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
TILES = os.path.join(HERE, "tiles")
# The old 16 px pictures, as the game had them before the 32 px redo.
OLD = "79be3ec"


def tile(name):
    return np.asarray(Image.open(os.path.join(TILES, name + ".png")).convert("RGBA")).astype(float)


def save(a, name):
    Image.fromarray(np.clip(a, 0, 255).round().astype(np.uint8), "RGBA").save(os.path.join(TILES, name + ".png"))


def old(name):
    data = subprocess.run(["git", "show", f"{OLD}:mods/base/assets/textures/block/{name}.png"], cwd=ROOT, capture_output=True, check=True).stdout
    from io import BytesIO
    return np.asarray(Image.open(BytesIO(data)).convert("RGBA")).astype(float)


def worn_farmland():
    base = tile("farmland")
    dust = np.array([0x9A, 0x86, 0x6C], float)
    for level in (1, 2, 3):
        a = base.copy()
        k = 0.17 * level
        mean = a[..., :3].mean((0, 1))
        a[..., :3] = (a[..., :3] - mean) * (1 - 0.15 * level) + mean
        a[..., :3] = a[..., :3] * (1 - k) + dust * k
        save(a, f"farmland_worn_{level}")


def clear_glass():
    """The generated glass's glare streaks pick up a pink cast from the magenta: pale blue-white."""
    glass = tile("glass")
    inside = np.zeros(glass.shape[:2], bool)
    inside[3:-3, 3:-3] = True
    glare = inside & (glass[..., 3] > 0)
    glass[glare, :3] = [0xDC, 0xEC, 0xF2]
    save(glass, "glass")


def stained_glass():
    glass = tile("glass")
    for name, rgb in (("red", (0xC0, 0x30, 0x30)), ("green", (0x3E, 0x9A, 0x40)), ("blue", (0x34, 0x5E, 0xC0))):
        c = np.array(rgb, float)
        a = glass.copy()
        lum = (a[..., :3] @ [0.3, 0.55, 0.15]) / 255
        solid = a[..., 3] > 0
        a[..., :3] = np.where(solid[..., None], c * (0.45 + 0.75 * lum[..., None]), c)
        a[..., 3] = np.where(solid, 255, 100)
        save(a, f"glass_{name}")


def drift(size=32, frames=16, seed=0, top=4):
    """A seamless pattern in [0, 1] that moves one full period over the frames, so it loops."""
    rng = np.random.default_rng(seed)
    y, x = np.mgrid[0:size, 0:size] / size * 2 * np.pi
    waves = [(rng.integers(1, top), rng.integers(1 - top, top), rng.uniform(0, 2 * np.pi), rng.uniform(0.5, 1)) for _ in range(6)]
    out = []
    for f in range(frames):
        t = f / frames * 2 * np.pi
        v = sum(amp * np.sin(kx * x + ky * y + ph + t * (1 if i % 2 else -1)) for i, (kx, ky, ph, amp) in enumerate(waves))
        out.append((v - v.min()) / (v.max() - v.min()))
    return out


def water():
    # Light grey, partly see-through: the game tints it by the biome's water colour.
    frames = []
    for v in drift(seed=3):
        g = 214 + np.round(v * 4) / 4 * 38
        frames.append(np.dstack([g, g, g, np.full_like(g, 170)]))
    save(np.vstack(frames), "water_still")


def lava():
    ramp = np.array([[0x41, 0x16, 0x12], [0x7B, 0x16, 0x13], [0xCD, 0x1C, 0x18], [0xE8, 0x5A, 0x10], [0xF3, 0x9C, 0x00], [0xFA, 0xD8, 0x6A]], float)
    frames = []
    for v in drift(seed=7, top=3):
        # Dark crust plates; where the pattern crosses its middle, a glowing crack.
        crack = np.clip(1 - np.abs(v - 0.5) * 6, 0, 1)
        i = np.clip(np.round((0.2 + 0.3 * v + 0.8 * crack) * (len(ramp) - 1)), 0, len(ramp) - 1).astype(int)
        rgb = ramp[i]
        frames.append(np.dstack([rgb, np.full(v.shape, 255)]))
    save(np.vstack(frames), "lava_still")


def gateway():
    """The amber gateway's surface (animals): a slow amber swirl, partly see-through."""
    ramp = np.array([[0x96, 0x32, 0x06], [0xB8, 0x52, 0x10], [0xD8, 0x7A, 0x1E], [0xF0, 0xA8, 0x34], [0xFF, 0xC4, 0x40]], float)
    frames = []
    for v in drift(seed=11, top=3):
        i = np.clip(np.round(v * (len(ramp) - 1)), 0, len(ramp) - 1).astype(int)
        frames.append(np.dstack([ramp[i], np.full(v.shape, 215)]))
    Image.fromarray(np.vstack(frames).round().astype(np.uint8), "RGBA").save(os.path.join(TILES, "gateway.png"))


# Each old colour family -> how to draw it at 32 px.
def material(rgb, a):
    r, g, b = rgb
    if a == 0:
        return None
    if (r, g, b) == (0x46, 0x46, 0x46):
        return "frame"
    if r > 200 and g < 60:
        return "red_on"
    if 0x50 < r < 0x70 and g < 0x30:
        return "red_off"
    if r > 200 and g > 200 and b > 200:
        return "white"
    if r > 140 and g < 60:
        return "wool_red"
    if 0x70 < r < 0x80 and g < 0x20:
        return "wool_red"
    if r > 0xE0 and g > 0xA0 and b < 0x80:
        return "flame"
    if g > r + 20 and g > b:
        return "green"
    if r < 0x40 and abs(r - g) < 12 and abs(g - b) < 12:
        return "socket"
    if r > g + 18 and g > b + 8:
        return "wood"
    return "stone"


def remap(name, wood="planks_oak"):
    src = old(name)
    big = src.repeat(32 // src.shape[0], 0).repeat(32 // src.shape[1], 1) if src.shape[0] == 16 else src
    stone, oak, red, white, slime = tile("smooth_stone"), tile(wood), tile("wool_red"), tile("wool_white"), tile("slime_block")
    out = np.zeros((32, 32, 4))
    kinds = np.empty((32, 32), object)
    for y in range(32):
        for x in range(32):
            kinds[y, x] = material(tuple(int(c) for c in big[y, x, :3]), big[y, x, 3])
    for y in range(32):
        for x in range(32):
            k = kinds[y, x]
            if k is None:
                continue
            # Symbols are lit from the top left: lighter on an edge facing up or left, darker down or right.
            edge = 0
            if k in ("red_on", "red_off", "flame", "green", "socket"):
                if y == 0 or kinds[y - 1, x] != k or x == 0 or kinds[y, x - 1] != k:
                    edge = 1
                elif y == 31 or kinds[y + 1, x] != k or x == 31 or kinds[y, x + 1] != k:
                    edge = -1
            px = {
                "frame": stone[y, x, :3] * 0.55,
                "stone": stone[y, x, :3],
                "wood": oak[y, x, :3],
                "wool_red": red[y, x, :3],
                "white": white[y, x, :3] if name.startswith("bed") else big[y, x, :3],
                "red_on": np.array([0xD8, 0x2A, 0x22]),
                "red_off": np.array([0x6E, 0x22, 0x1C]),
                "flame": big[y, x, :3],
                "green": slime[y, x, :3],
                "socket": np.array([0x2E, 0x2A, 0x28]),
            }[k]
            out[y, x, :3] = px * (1.15 if edge > 0 else 0.8 if edge < 0 else 1)
            out[y, x, 3] = big[y, x, 3]
    save(out, name)


MECHANICAL = [
    "anchor", "anchor_on", "button", "comparator", "comparator_on", "repeater", "repeater_on", "dispenser_front_vertical",
    "observer_back_on", "piston_inner", "floater_back", "floater_front", "floater_side", "floater_side_going", "floater_side_stuck",
    "lever", "torch", "redstone_torch", "redstone_torch_off",
    "bed_foot_east", "bed_foot_south", "bed_foot_west", "bed_head_east", "bed_head_north", "bed_head_west",
    "bed_top_foot", "bed_top_foot_90", "bed_top_foot_180", "bed_top_foot_270",
    "bed_top_head_0", "bed_top_head_90", "bed_top_head_180", "bed_top_head_270",
]
# Redstone dust is grey (the game tints it by power): its old picture doubled.
GREY = ["redstone_dust_dot", "redstone_dust_line"]


if __name__ == "__main__":
    worn_farmland()
    clear_glass()
    stained_glass()
    water()
    lava()
    gateway()
    for n in MECHANICAL:
        remap(n, "log_oak" if n in ("lever", "torch", "redstone_torch", "redstone_torch_off") else "planks_oak")
    for n in GREY:
        a = old(n)
        save(a.repeat(2, 0).repeat(2, 1), n)
    print("derived:", "farmland_worn_1-3 glass_red/green/blue water_still lava_still", " ".join(MECHANICAL + GREY))
