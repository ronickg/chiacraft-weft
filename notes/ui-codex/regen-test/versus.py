"""Every item and block of the regen test, run 1 (old guides) beside run 3 (Guides 3), with today's
game icon small for reference, on one sheet (Ronald, 11-05: "i wnan see all the items version 3 did
vs 1").

    python3 notes/ui-codex/regen-test/versus.py CURRENT_DIR RUN1 RUN2 RUN3 OUT.png
"""
import os
import sys

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.argv, args = sys.argv[:1], sys.argv[1:]
cur, run1, run2, run3, out = args
SLOT = Image.open(os.path.join(ROOT, "mods/base/assets/textures/gui/slot.png")).convert("RGBA")
BLOCKS = os.path.join(ROOT, "mods/base/assets/textures/block")
F = "/System/Library/Fonts/Supplemental/"
font, bold, big = (ImageFont.truetype(F + "Arial.ttf", 14), ImageFont.truetype(F + "Arial Bold.ttf", 16),
                   ImageFont.truetype(F + "Arial Bold.ttf", 26))
BG, INK, MUTED, GOLD = (26, 19, 19), (228, 218, 207), (198, 163, 131), (242, 218, 154)
FOLIAGE = (92, 160, 48)

ITEMS = [("Helmet shell", "shell_helmet_shell", "helmet_shell"), ("Chest shell", "shell_chest_shell", "chest_shell"),
         ("Leg plates", "shell_leg_shell", "leg_shell"), ("Boot plates", "shell_boot_shell", "boot_shell"),
         ("Helmet", "shell_helmet_shell", "diamond_helmet"),
         ("Chestplate", "shell_chest_shell", "diamond_chestplate"),
         ("Leggings", "shell_leg_shell", "diamond_leggings"), ("Boots", "shell_boot_shell", "diamond_boots"),
         ("Long blade", "layer_sword", "part_long_blade"), ("Short blade", "part_short_blade", "part_short_blade"),
         ("Axe head", "part_felling_axe_head", "part_axe_head"), ("Pick head", "part_pick_head", "part_pick_head"),
         ("Shovel blade", "part_shovel_blade", "part_shovel_blade"), ("Hoe blade", "part_hoe_blade", "part_hoe_blade"),
         ("Spear point", "part_leaf_point", "part_spear_point"), ("Sword", "built_sword", "diamond_sword"),
         ("Pickaxe", "built_pickaxe", "diamond_pickaxe"), ("Axe", "built_axe", "diamond_axe"),
         ("Shovel", "built_shovel", "diamond_shovel"), ("Hoe", "built_hoe", "diamond_hoe"),
         ("Spear", "built_spear", "diamond_spear"), ("Bread", "food_bread", "bread"), ("Apple", "food_apple", "apple"),
         ("Cooked beef", "food_cooked_beef", "cooked_beef"), ("Raw beef", "food_beef", "beef"),
         ("Trout", "food_trout", "trout"), ("Stew", "food_stew", "stew"), ("Egg", "food_egg", "egg"),
         ("Carrot", "food_carrot", "carrot")]
BLOCK_ROWS = [("Stone", "stone"), ("Diamond ore", "diamond_ore"), ("Oak log", "log_oak"), ("Oak leaves", "leaves_oak")]


def icon(p):
    return Image.open(p).convert("RGBA") if p and os.path.exists(p) else None


def slot(ic, k):
    s = SLOT.resize((36 * k, 36 * k), Image.NEAREST)
    if ic is not None:
        s.alpha_composite(ic.resize((32 * k, 32 * k), Image.NEAREST), (2 * k, 2 * k))
    return s


def tile(ic, k, tint):
    if tint:
        import numpy as np
        a = np.array(ic).astype(float)
        a[..., :3] *= np.array(FOLIAGE) / 255
        ic = Image.fromarray(a.clip(0, 255).astype("uint8"), "RGBA")
    g = Image.new("RGBA", (64, 64), (40, 30, 26, 255))
    for j in range(2):
        for i in range(2):
            g.alpha_composite(ic, (32 * i, 32 * j))
    return g.resize((64 * k // 2 * 2 // 2 * 1, 64 * k // 2 * 2 // 2 * 1), Image.NEAREST) if False else g.resize((32 * k, 32 * k), Image.NEAREST)


CW, CH, COLS = 430, 200, 4
rows = (len(ITEMS) + COLS - 1) // COLS + 1
im = Image.new("RGBA", (24 + COLS * CW, 110 + rows * CH + 20), BG + (255,))
d = ImageDraw.Draw(im)
d.text((24, 18), "Every item: version 1 (no reference sheets) beside version 3 (Guides 3 + reference sheets)", font=big, fill=GOLD)
d.text((24, 56), "Each card: today's game icon small on the left, then version 1 and version 3 at 4x on the game's slot. "
       "Blocks (last row) as a 2 x 2 patch, leaves tinted green as in the game.", font=font, fill=MUTED)
for i, (label, c, n) in enumerate(ITEMS + [(l, "block:" + n, "@" + n) for l, n in BLOCK_ROWS]):
    x, y = 24 + (i % COLS if i < len(ITEMS) else (i - len(ITEMS))) * CW, 100 + (i // COLS if i < len(ITEMS) else rows - 1) * CH
    d.rounded_rectangle([x, y, x + CW - 14, y + CH - 14], 8, fill=(36, 27, 24))
    d.text((x + 10, y + 8), label, font=bold, fill=INK)
    if n.startswith("@"):
        b = n[1:]
        pics = [icon(os.path.join(BLOCKS, b + ".png")), icon(os.path.join(run2, f"b1_{b}_px.png")),
                icon(os.path.join(run3, f"b3_{b}_px.png"))]
        draw = [tile(p, 2, "leaves" in b) if p else None for p in pics[:1]] + \
               [tile(p, 4, "leaves" in b) if p else None for p in pics[1:]]
    else:
        pics = [icon(os.path.join(cur, c + ".png")), icon(os.path.join(run1, f"rt_{n}_px.png")),
                icon(os.path.join(run3, f"n3_{n}_px.png"))]
        draw = [slot(pics[0], 2), slot(pics[1], 4), slot(pics[2], 4)]
    px = x + 10
    for k, (pic, cap) in enumerate(zip(draw, ("now", "version 1", "version 3"))):
        if pic is None:
            continue
        top = y + 32 + (pic.height and (144 - pic.height) if k == 0 else 0)
        im.alpha_composite(pic, (px, top))
        d.text((px, top - 0 + pic.height + 1) if k else (px, top - 18), cap, font=font, fill=MUTED if k == 0 else GOLD)
        px += pic.width + 8
im.convert("RGB").save(out)
print(out)
