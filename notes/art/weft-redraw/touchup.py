"""The second pass on the Weft redraw (11-22, Ronald via Planner after the in-game look): bigger
berries on the two berry bushes and wheels that read as wheels. Made in code from approved pieces,
like the shield: the berries are painted on our plain raspberry leaves (what the snowberry bush
already shows when bare), the wheel faces drawn from the iron ramp and a rubber ramp.

The wheels are cylinders of thin boxes whose faces take their UVs from the box's place in the block
(`vehicles.rs`), so every picture here is centred on the block's middle: the small, middle and large
wheel each show the part of it their size reaches (the hub: the cap only, cap and spokes, cap,
spokes and the rim's start).

    python3 notes/art/weft-redraw/touchup.py OUT_DIR    # OUT_DIR/<name>.png, 32 x 32
"""
import math
import os
import random
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
BLOCK = os.path.join(HERE, "..", "..", "..", "mods", "base", "assets", "textures", "block")

# dark to light
RASPBERRY = [(0x5A, 0x10, 0x1C), (0x8E, 0x18, 0x2A), (0xBE, 0x26, 0x3A), (0xE0, 0x4A, 0x5A), (0xF6, 0x9A, 0xA4)]
SNOWBERRY = [(0x7C, 0x86, 0x88), (0xB4, 0xBC, 0xBE), (0xDC, 0xE2, 0xE2), (0xF2, 0xF4, 0xF0), (0xFF, 0xFF, 0xFF)]
RUBBER = [(0x24, 0x21, 0x1F), (0x33, 0x2F, 0x2C), (0x45, 0x40, 0x3C), (0x57, 0x51, 0x4C), (0x6B, 0x64, 0x5E)]
IRON = [(0x4E, 0x50, 0x52), (0x70, 0x70, 0x70), (0x97, 0x97, 0x97), (0xB8, 0xB8, 0xB8), (0xD8, 0xD8, 0xD8), (0xF4, 0xF4, 0xF4)]


def leaves():
    return Image.open(os.path.join(BLOCK, "raspberry_leaves.png")).convert("RGBA")


def berry(p, cx, cy, ramp, r, drupelets):
    """One round berry, lit from the top left, outlined in its own darkest shade."""
    for y in range(cy - r - 1, cy + r + 2):
        for x in range(cx - r - 1, cx + r + 2):
            d = math.hypot(x - cx, y - cy)
            if not (0 <= x < 32 and 0 <= y < 32) or d > r + 0.5:
                continue
            if d > r - 0.5:
                p[x, y] = ramp[0] + (255,)
                continue
            light = -((x - cx) + (y - cy)) / (2 * r)  # top left +, bottom right -
            i = 2 + round(light * 2)
            if drupelets and (x + y) % 2 == 0 and i > 1:
                i -= 1
            p[x, y] = ramp[max(1, min(len(ramp) - 2, i))] + (255,)
    # one glint
    p[cx - max(1, r // 2), cy - max(1, r // 2)] = ramp[-1] + (255,)


def bush(ramp, spots, drupelets):
    im = leaves()
    p = im.load()
    for cx, cy, r in spots:
        berry(p, cx, cy, ramp, r, drupelets)
    return im


def scatter(seed, clusters, per, sizes):
    """Loose clusters at random places, a few berries each, not too close to another cluster, every
    berry whole inside the tile (a neighbour is another variant, so one cut at a seam shows):
    (x, y, r) for each berry."""
    rng = random.Random(seed)
    centres = []
    out = []
    while len(centres) < clusters:
        c = (rng.randrange(5, 27), rng.randrange(5, 27))
        if all(math.hypot(c[0] - o[0], c[1] - o[1]) > 12 for o in centres):
            centres.append(c)
    for cx, cy in centres:
        n = rng.choice(per)
        for k in range(n):
            for _ in range(50):
                a = rng.uniform(0, 2 * math.pi)
                d = 0 if k == 0 else rng.uniform(5.5, 7.5)
                x, y, r = round(cx + d * math.cos(a)), round(cy + d * math.sin(a)), rng.choice(sizes)
                if r + 1 <= x <= 30 - r and r + 1 <= y <= 30 - r and all(math.hypot(x - o[0], y - o[1]) > r + o[2] + 0.5 for o in out):
                    out.append((x, y, r))
                    break
    return out


def ring_shade(ramp, d, rings):
    """Rubber shade at distance d from the middle: grooves at each ring, lighter just inside it."""
    for at in rings:
        if abs(d - at) < 0.6:
            return ramp[0]
        if 0.6 <= at - d < 1.6:
            return ramp[3]
    return ramp[2]


def wheel_side():
    im = Image.new("RGBA", (32, 32))
    p = im.load()
    for y in range(32):
        for x in range(32):
            d = math.hypot(x - 15.5, y - 15.5)
            c = ring_shade(RUBBER, d, (7.0, 11.0, 15.0))  # a lip just inside each wheel's edge (8, 12, 16 texels)
            # a soft light from the top left on the sidewall
            if x + y < 22 and c == RUBBER[2]:
                c = RUBBER[3]
            p[x, y] = c + (255,)
    return im


def wheel_tread():
    """The rolling face: u across the tyre, v around it. Blocks of tread with grooves across, a
    channel down the middle, every 4 texels around."""
    im = Image.new("RGBA", (32, 32))
    p = im.load()
    for y in range(32):
        for x in range(32):
            c = RUBBER[3]
            if y % 8 in (0, 4):
                c = RUBBER[0]  # groove across
            elif y % 8 in (1, 5):
                c = RUBBER[4] if y % 8 == 1 else RUBBER[3]
            elif y % 8 in (3, 7):
                c = RUBBER[2]
            if x in (15, 16):
                c = RUBBER[1]  # channel down the middle
            if x in (0, 31):
                c = RUBBER[1]
            p[x, y] = c + (255,)
    return im


def wheel_hub():
    im = Image.new("RGBA", (32, 32))
    p = im.load()
    for y in range(32):
        for x in range(32):
            dx, dy = x - 15.5, y - 15.5
            d = math.hypot(dx, dy)
            a = math.atan2(dy, dx)
            lit = -(dx + dy) / 32  # top left lighter
            # the hub box is a third of the wheel's radius: 2.6, 4 and 5.3 texels from the middle
            if d < 1.6:  # the cap
                c = IRON[5] if lit >= 0 else IRON[4]
            elif d < 2.3:
                c = IRON[1]
            elif d < 4.4:  # five spokes over a dark well
                on = abs(((a / (2 * math.pi) * 5) % 1) - 0.5) > 0.5 - 0.8 / d
                c = (IRON[4] if lit > 0 else IRON[3]) if on else IRON[0]
            elif d < 5.6:
                c = IRON[4] if lit > 0.03 else IRON[2]  # the rim
            else:
                c = IRON[1]
            p[x, y] = c + (255,)
    return im


def main(out):
    os.makedirs(out, exist_ok=True)
    # three random variants a bush (upstream's folder form, `method = random`), each its own count:
    # raspberries alone or in twos, snowberries in twos and threes, as they hang
    kinds = {
        "raspberry_leaves_berried": (RASPBERRY, True, [(11, 3, [1, 1, 2]), (12, 2, [1, 2]), (13, 4, [1, 1, 2])], [2, 3]),
        "snowberry_leaves_berried": (SNOWBERRY, False, [(21, 2, [2, 3]), (22, 3, [1, 2, 2]), (23, 2, [2])], [3]),
    }
    for name, (ramp, drupelets, variants, sizes) in kinds.items():
        folder = os.path.join(out, name)
        os.makedirs(folder, exist_ok=True)
        for i, (seed, clusters, per) in enumerate(variants, 1):
            bush(ramp, scatter(seed, clusters, per, sizes), drupelets).save(os.path.join(folder, f"{i}.png"))
        with open(os.path.join(folder, name + ".ctm"), "w") as f:
            f.write(f"method = random\ntiles = 1-{len(variants)}\n")
    wheel_side().save(os.path.join(out, "wheel_side.png"))
    wheel_tread().save(os.path.join(out, "wheel_tread.png"))
    wheel_hub().save(os.path.join(out, "wheel_hub.png"))
    print("touch-up to", out)


if __name__ == "__main__":
    main(sys.argv[1])
