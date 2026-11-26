"""Review sheet for the water plants: each option flat at 6x, and in a small scene at 2x
(seagrass and kelp under water on sand, reeds at a shore, lily pads seen from above)."""
import sys, os
from PIL import Image, ImageDraw, ImageFont
import numpy as np

W = sys.argv[1]                      # worktree (python3 notes/blocks/water_sheet.py <worktree> <out.png>)
OUT = sys.argv[2]
TILES = f"{W}/notes/blocks/tiles"
TEX = f"{W}/mods/base/assets/textures/block"
WATER = np.array([0x3d, 0x57, 0xd6]) / 255

def tex(n):
    for d in (TILES, TEX):
        p = f"{d}/{n}.png"
        if os.path.exists(p):
            return Image.open(p).convert("RGBA")
    raise SystemExit(f"missing {n}")

def water_tile():
    w = np.asarray(tex("water_still").convert("RGBA")).astype(float)[:32]
    w[..., :3] *= WATER
    w[..., 3] = 80
    return Image.fromarray(w.clip(0, 255).astype(np.uint8))

def big(im, k):
    return im.resize((im.width * k, im.height * k), Image.NEAREST)

try:
    FONT = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 22)
    SMALL = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 16)
except OSError:
    FONT = SMALL = ImageFont.load_default()

B = 64  # a block in the scenes

def side_scene(floor, plants, under_water, depth=3, cols=5, shore=False):
    """A side view: a floor row, plants (lists of texture columns, bottom first) on it, water over."""
    h = depth + 1
    im = Image.new("RGBA", (cols * B, h * B), (150, 190, 230, 255))
    for c in range(cols):
        f = floor[c] if isinstance(floor, list) else floor
        im.alpha_composite(big(tex(f), 2), (c * B, (h - 1) * B))
    for c, col in enumerate(plants):
        for i, n in enumerate(col):
            if n:
                im.alpha_composite(big(tex(n), 2), (c * B, (h - 2 - i) * B))
    if under_water:
        wt = big(water_tile(), 2)
        for c in range(cols):
            if shore and c < 2:
                continue
            for r in range(h - 1):
                im.alpha_composite(wt, (c * B, r * B))
    return im

def top_scene(pad, cols=4):
    im = Image.new("RGBA", (cols * B, cols * B), (0, 0, 0, 255))
    sand = big(tex("sand"), 2)
    wt = big(water_tile(), 2)
    p = big(tex(pad), 2)
    spots = {(0, 1), (1, 2), (2, 0), (3, 3), (2, 2)}
    for x in range(cols):
        for y in range(cols):
            im.alpha_composite(sand, (x * B, y * B))
            im.alpha_composite(wt, (x * B, y * B))
            im.alpha_composite(wt, (x * B, y * B))
            im.alpha_composite(wt, (x * B, y * B))
            if (x, y) in spots:
                im.alpha_composite(p.rotate(90 * ((x + y) % 4)), (x * B, y * B))
    return im

ROWS = [
    ("Seagrass (base:seagrass)  * A: denser, reads as eelgrass", "seagrass", "seagrass_b",
     lambda n: side_scene("sand", [[n], [], [n], [n], []], True)),
    ("Lily pad (base:lily_pad), from above  * A; B could be a 1-in-4 flowering variant", "lily_pad", "lily_pad_b", lambda n: top_scene(n)),
    ("Kelp top (column: kelp, kelp, top)  * A: the same giant kelp as the column; B is bull kelp", "kelp_top", "kelp_top_b",
     lambda n: side_scene("sand", [["kelp", "kelp", n], [], ["kelp", n], [], ["kelp", "kelp", n]], True)),
    ("Common reed (base:reed, now borrowing sugar cane)  * A: summer plumes; B autumn", "reed", "reed_b",
     lambda n: side_scene(["grass_side", "grass_side", "sand", "sand", "sand"], [[n], [n], [n], [], []], True, depth=1, shore=True)),
]
row_h = 4 * B + 70
sheet = Image.new("RGBA", (40 + 2 * (192 + 30) + 2 * (5 * B + 30), 70 + len(ROWS) * row_h), (42, 32, 26, 255))
d = ImageDraw.Draw(sheet)
d.text((20, 18), "Water plants, batch 1: option A (left) and B, each flat and in a scene. * = my pick", font=FONT, fill=(245, 228, 188))
for i, (title, a, b, scene) in enumerate(ROWS):
    y = 70 + i * row_h
    d.text((20, y), title, font=FONT, fill=(255, 190, 69))
    x = 20
    for lab, n in (("A", a), ("B", b)):
        t = big(tex(n), 6)
        bg = Image.new("RGBA", t.size, (86, 120, 150, 255))
        bg.alpha_composite(t)
        sheet.alpha_composite(bg, (x, y + 34))
        d.text((x, y + 34 + 196), f"{lab}: {n}", font=SMALL, fill=(245, 228, 188))
        x += 192 + 30
    for n in (a, b):
        s = scene(n)
        sheet.alpha_composite(s, (x, y + 34))
        x += s.width + 30
sheet.convert("RGB").save(OUT)
print(OUT)
