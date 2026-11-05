"""Every item and block of the regen test, run 1 (old guides) beside run 3 (Guides 3), with today's
game icon small for reference, on one sheet (Ronald, 11-05: "i wnan see all the items version 3 did
vs 1").

    python3 notes/ui-codex/regen-test/versus.py CURRENT_DIR RUN1 RUN2 RUN3 OUT.png [RAW3]

With RAW3 (the raw run: Ronald's Guides 3 as sent, its sheets, none of our key or lessons) a third
version shows, and the Guides 3 extras (diamond, ingot, coal, seeds, grass) get cards too.
"""
import os
import sys

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.argv, args = sys.argv[:1], sys.argv[1:]
cur, run1, run2, run3, out = args[:5]
raw = args[5] if len(args) > 5 else None
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
if raw:
    ITEMS += [("Diamond (gem)", "approved_diamond", "diamond"), ("Iron ingot", "approved_iron_ingot", "iron_ingot"),
              ("Coal", "approved_coal", "coal"), ("Wheat seeds", "food_wheat_seeds", "wheat_seeds")]
    BLOCK_ROWS += [("Grass top", "grass_top"), ("Grass side", "grass_side")]


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


CW, CH, COLS = (580, 200, 3) if raw else (430, 200, 4)
nb = (len(BLOCK_ROWS) + COLS - 1) // COLS
rows = (len(ITEMS) + COLS - 1) // COLS + nb
im = Image.new("RGBA", (24 + COLS * CW, 110 + rows * CH + 20), BG + (255,))
d = ImageDraw.Draw(im)
d.text((24, 18), "Every item: version 1 (no reference sheets) beside version 3 (Guides 3 + reference sheets)"
       + (" and raw version 3 (Guides 3 as sent, its sheets, none of our key or lessons)" if raw else ""),
       font=big if not raw else bold, fill=GOLD)
d.text((24, 56), "Each card: today's game icon small on the left, then version 1 and version 3 at 4x on the game's slot. "
       "Blocks (last row) as a 2 x 2 patch, leaves tinted green as in the game.", font=font, fill=MUTED)
for i, (label, c, n) in enumerate(ITEMS + [(l, "block:" + n, "@" + n) for l, n in BLOCK_ROWS]):
    if i < len(ITEMS):
        x, y = 24 + (i % COLS) * CW, 100 + (i // COLS) * CH
    else:
        b = i - len(ITEMS)
        x, y = 24 + (b % COLS) * CW, 100 + ((len(ITEMS) + COLS - 1) // COLS + b // COLS) * CH
    d.rounded_rectangle([x, y, x + CW - 14, y + CH - 14], 8, fill=(36, 27, 24))
    d.text((x + 10, y + 8), label, font=bold, fill=INK)
    if n.startswith("@"):
        b = n[1:]
        pics = [icon(os.path.join(BLOCKS, b + ".png")), icon(os.path.join(run2, f"b1_{b}_px.png")),
                icon(os.path.join(run3, f"b3_{b}_px.png"))]
        if raw:
            pics.append(icon(os.path.join(raw, f"br_{b}_px.png")))
        tint = "leaves" in b or b == "grass_top"
        draw = [tile(p, 2, tint) if p else None for p in pics[:1]] + \
               [tile(p, 4, tint) if p else None for p in pics[1:]]
    else:
        pics = [icon(os.path.join(cur, c + ".png")), icon(os.path.join(run1, f"rt_{n}_px.png")),
                icon(os.path.join(run3, f"n3_{n}_px.png"))]
        if raw:
            pics.append(icon(os.path.join(raw, f"r3_{n}_px.png")))
        draw = [slot(pics[0], 2)] + [slot(p, 4) if p else None for p in pics[1:]]
    px = x + 10
    for k, (pic, cap) in enumerate(zip(draw, ("now", "version 1", "version 3", "raw v3"))):
        if pic is None:
            px += 152
            continue
        top = y + 32 + (pic.height and (144 - pic.height) if k == 0 else 0)
        im.alpha_composite(pic, (px, top))
        d.text((px, top - 0 + pic.height + 1) if k else (px, top - 18), cap, font=font, fill=MUTED if k == 0 else GOLD)
        px += pic.width + 8
im.convert("RGB").save(out)
print(out)
