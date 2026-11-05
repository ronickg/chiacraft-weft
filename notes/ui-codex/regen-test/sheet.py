"""Comparison sheets for the art regen test (11-05): each item in the game now, from run 1 (the old
guides: our key only) and from run 2 (Ronald's guides 2: our key plus the guide's reference sheet),
in the game's inventory slot (`gui/slot.png`, the icon 2 px in) at 2x and 4x, with the approved
11-04 icon where there is one, both images as generated, and a line on what changed. A strip on top
mixes run 2's icons with established neighbours from the approved 11-04 set, as a hotbar. Blocks
show one tile at 4x and a 3 x 3 patch at 2x (leaves tinted a plains green, as the game does).

    python3 notes/ui-codex/regen-test/sheet.py CURRENT_DIR NOTES.json OUT_DIR RUN [RUN...]

Each RUN is `label=dir[+dir...]:item prefix:block prefix`, e.g. "run 2 (guides 2)=A/run2+A/run2x:n2:b2";
a run's pictures are looked up in its dirs in order.
"""
import json
import os
import sys
import textwrap

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
SLOT = Image.open(os.path.join(ROOT, "mods/base/assets/textures/gui/slot.png")).convert("RGBA")
F = "/System/Library/Fonts/Supplemental/"
font = ImageFont.truetype(F + "Arial.ttf", 15)
small = ImageFont.truetype(F + "Arial.ttf", 12)
bold = ImageFont.truetype(F + "Arial Bold.ttf", 16)
big = ImageFont.truetype(F + "Arial Bold.ttf", 24)
BG, INK, MUTED, GOLD = (26, 19, 19), (228, 218, 207), (198, 163, 131), (242, 218, 154)

cur, notes, out = sys.argv[1:4]
notes = json.load(open(notes))
RUNS = []
for r in sys.argv[4:]:
    label, rest = r.split("=", 1)
    dirs, ip, bp = rest.rsplit(":", 2)
    RUNS.append((label, dirs.split("+"), ip, bp))
os.makedirs(out, exist_ok=True)
BLOCKS = os.path.join(ROOT, "mods/base/assets/textures/block")
FOLIAGE = (92, 160, 48)


def icon(path):
    return Image.open(path).convert("RGBA") if path and os.path.exists(path) else None


def slot(ic, k):
    s = SLOT.resize((36 * k, 36 * k), Image.NEAREST)
    if ic is not None:
        s.alpha_composite(ic.resize((32 * k, 32 * k), Image.NEAREST), (2 * k, 2 * k))
    return s


def pair(ic):
    """2x and 4x slots side by side."""
    a, b = slot(ic, 2), slot(ic, 4)
    im = Image.new("RGBA", (a.width + 6 + b.width, b.height), (0, 0, 0, 0))
    im.alpha_composite(a, (0, b.height - a.height))
    im.alpha_composite(b, (a.width + 6, 0))
    return im


def tinted(ic):
    import numpy as np
    a = np.array(ic).astype(float)
    a[..., :3] = a[..., :3] * np.array(FOLIAGE) / 255
    return Image.fromarray(a.clip(0, 255).astype("uint8"), "RGBA")


def tile(ic, tint=False):
    """A block: one tile at 4x and a 3 x 3 patch at 2x, on a dark ground."""
    if ic is None:
        return None
    ic = tinted(ic) if tint else ic
    im = Image.new("RGBA", (128 + 8 + 192, 192), (0, 0, 0, 0))
    g = Image.new("RGBA", (128, 128), (40, 30, 26, 255))
    g.alpha_composite(ic.resize((128, 128), Image.NEAREST))
    im.alpha_composite(g, (0, 64))
    p = Image.new("RGBA", (96, 96), (40, 30, 26, 255))
    for j in range(3):
        for i in range(3):
            p.alpha_composite(ic, (32 * i, 32 * j))
    im.alpha_composite(p.resize((192, 192), Image.NEAREST), (136, 0))
    return im


ARMOUR_ROWS = [("Helmet shell", "shell_helmet_shell", "helmet_shell", None),
               ("Chest shell", "shell_chest_shell", "chest_shell", None),
               ("Leg plates", "shell_leg_shell", "leg_shell", None),
               ("Boot plates", "shell_boot_shell", "boot_shell", None),
               ("Helmet (built)", "shell_helmet_shell", "diamond_helmet", "approved_diamond_helmet"),
               ("Chestplate (built)", "shell_chest_shell", "diamond_chestplate", "approved_diamond_chestplate"),
               ("Leggings (built)", "shell_leg_shell", "diamond_leggings", "approved_diamond_leggings"),
               ("Boots (built)", "shell_boot_shell", "diamond_boots", "approved_diamond_boots")]
GROUPS = {
    "armour": dict(
        title="Diamond armour: shells and built pieces",
        sub="In game now, a diamond piece's icon is its shell's map painted with the diamond ramp (the loose shell and "
            "the built piece look the same). 11-04 is the approved whole diamond item, no longer in any list. "
            "Runs 2 and 3 attached the item guide's 03 Armour sheet.",
        neighbours=["approved_diamond", "approved_iron_chestplate", "approved_gold_helmet", "approved_iron_ingot"],
        rows=ARMOUR_ROWS,
        set_row=[("now", ["shell_helmet_shell", "shell_chest_shell", "shell_leg_shell", "shell_boot_shell"]),
                 ("11-04", ["approved_diamond_helmet", "approved_diamond_chestplate", "approved_diamond_leggings",
                            "approved_diamond_boots"]),
                 *[(f"run {k}", [f"{k}:diamond_helmet", f"{k}:diamond_chestplate", f"{k}:diamond_leggings",
                                    f"{k}:diamond_boots"]) for k in (1, 2, 3)]]),
    "tools": dict(
        title="Diamond tools: working parts and built tools",
        sub="In game now, a loose part has one grey icon in every material; its diamond drawing exists only as the "
            "painted head layer of the built tool ('on the tool'; the sword's is the straight blade). Built tools are "
            "composed as the engine does: the diamond head on the oak haft map, wrought-iron guard and pommel; the "
            "spear has no look yet, so its plain icon shows. Runs 2 and 3 attached 01 Tools or 02 Weapons plus 13 by tier.",
        neighbours=["approved_diamond", "approved_iron_sword", "approved_stick", "approved_coal"],
        rows=[("Long blade", "part_long_blade", "part_long_blade", "layer_sword"),
              ("Short blade", "part_short_blade", "part_short_blade", None),
              ("Axe head", "part_felling_axe_head", "part_axe_head", "layer_axe"),
              ("Pick head", "part_pick_head", "part_pick_head", "layer_pickaxe"),
              ("Shovel blade", "part_shovel_blade", "part_shovel_blade", "layer_shovel"),
              ("Hoe blade", "part_hoe_blade", "part_hoe_blade", "layer_hoe"),
              ("Spear point", "part_leaf_point", "part_spear_point", None),
              ("Sword", "built_sword", "diamond_sword", "approved_diamond_sword"),
              ("Pickaxe", "built_pickaxe", "diamond_pickaxe", "approved_diamond_pickaxe"),
              ("Axe", "built_axe", "diamond_axe", "approved_diamond_axe"),
              ("Shovel", "built_shovel", "diamond_shovel", "approved_diamond_shovel"),
              ("Hoe", "built_hoe", "diamond_hoe", "approved_diamond_hoe"),
              ("Spear", "built_spear", "diamond_spear", None)]),
    "food": dict(
        title="Food",
        sub="Today's food icons in the game beside the runs. Runs 2 and 3 attached 07 Food (11 Soft and organic for the "
            "egg, stew and trout) plus 16 All food.",
        neighbours=["food_berries", "food_baked_potato", "food_cooked_chicken", "food_melon_slice"],
        rows=[("Bread", "food_bread", "bread", None), ("Apple", "food_apple", "apple", None),
              ("Cooked beef", "food_cooked_beef", "cooked_beef", None), ("Raw beef", "food_beef", "beef", None),
              ("Trout", "food_trout", "trout", None), ("Stew", "food_stew", "stew", None),
              ("Egg", "food_egg", "egg", None), ("Carrot", "food_carrot", "carrot", None)]),
    "blocks": dict(
        title="Blocks: a natural cube, an ore, a log and leaves",
        sub="Made with `art.py block` from the block guide's type lines. Run 1 is the old way (our key only, "
            "`--no-ref`); runs 2 and 3 attached the block guide's sheet for the type (01 Natural cube, 03 Ores, 04 Logs, "
            "07 Leaves). Each: one tile at 4x and a 3 x 3 patch at 2x; leaves are grey in the game and shown tinted "
            "a plains green.",
        rows=[("Stone", "block:stone", "stone", None), ("Diamond ore", "block:diamond_ore", "diamond_ore", None),
              ("Oak log", "block:log_oak", "log_oak", None), ("Oak leaves", "block:leaves_oak", "leaves_oak", None)]),
    "guides3": dict(
        title="What Guides 3 changed: gems, ingots, fuel, seeds, grass",
        sub="Guides 3 added or rewrote the family notes for raw ores and ingots (a stamped bar on the diagonal, matte "
            "fuel), gems (each its own form: diamond 'a cluster of small cubes'), seeds (scattered grains), and grass "
            "(a fringe over the dirt's top edge). These are made with Guides 2 and then Guides 3, the same request and "
            "sheets; only the family notes differ. Run 1 (old guides) made none of them.",
        neighbours=["approved_iron_sword", "approved_gold_ingot", "approved_leather", "approved_stick"],
        rows=[("Diamond", "approved_diamond", "diamond", None), ("Iron ingot", "approved_iron_ingot", "iron_ingot", None),
              ("Coal", "approved_coal", "coal", None), ("Wheat seeds", "food_wheat_seeds", "wheat_seeds", None),
              ("Grass top", "block:grass_top", "@grass_top", None), ("Grass side", "block:grass_side", "@grass_side", None)]),
}


def cur_icon(name):
    if name.startswith("block:"):
        return icon(os.path.join(BLOCKS, name[6:] + ".png"))
    return icon(os.path.join(cur, name + ".png"))


def is_block(g, n):
    return g == "blocks" or n.startswith("@")


def find(g, k, n, ext):
    """Run k's file for item or block n (its prefix, looked up in its dirs in order), or None."""
    _, dirs, ip, bp = RUNS[k - 1]
    name = f"{bp if is_block(g, n) else ip}_{n.lstrip('@')}{ext}"
    for d in dirs:
        if os.path.exists(os.path.join(d, name)):
            return os.path.join(d, name)
    return None


def run_icon(g, k, n):
    return icon(find(g, k, n, "_px.png"))


def text(d, xy, s, f, fill, width):
    y = xy[1]
    for para in s.split("\n"):
        for line in textwrap.wrap(para, width) or [""]:
            d.text((xy[0], y), line, font=f, fill=fill)
            y += f.size + 4
    return y


def tries(g, k, n):
    chk = find(g, k, n, "_checks.json")
    if not chk:
        return ""
    t = json.load(open(chk))["tries"]
    last = t[-1] if t else {}
    return f"run {k}: {len(t)} tr{'y' if len(t) == 1 else 'ies'}, {'passed' if last.get('passed') else 'FAILED'}"


for g, G in GROUPS.items():
    blocks = g == "blocks"
    nr = len(RUNS)
    cell = 336 if blocks or g == "guides3" else 230
    W = 170 + (nr + 1) * cell + (0 if blocks else 230) + 200 + 540
    rowh = (200 if blocks or g == "guides3" else 144) + 40 + (30 if nr > 2 else 0)
    X = [24] + [170 + i * cell for i in range(nr + 2)]
    gen_x = X[nr + 2] if blocks else X[nr + 2] + 0
    note_x = None
    H = 500 + len(G["rows"]) * rowh + (5 * 90 + 60 if "set_row" in G else 0)
    im = Image.new("RGBA", (W, H), BG + (255,))
    d = ImageDraw.Draw(im)
    d.text((24, 18), f"Art regen test 11-05: {G['title']}", font=big, fill=GOLD)
    y = text(d, (24, 54), G["sub"] + ("" if blocks else " Slots are the game's slot.png at 2x and 4x (the game "
             "shows 2x at GUI scale 2)."), font, MUTED, 230)
    if not blocks:
        y += 8
        d.text((24, y), f"In context at 2x: {RUNS[-1][0]}'s icons (marked) among approved 11-04 neighbours", font=bold, fill=INK)
        y += 26
        mix, nb = [], G["neighbours"]
        for i, r in enumerate(G["rows"]):
            if r[2].startswith("@"):
                continue
            mix.append(("new", r[2]))
            if i % 3 == 2:
                mix.append(("old", nb[(i // 3) % len(nb)]))
        x = 24
        for kind, n in mix[:24]:
            sl = slot(run_icon(g, nr, n) if kind == "new" else cur_icon(n), 2)
            im.alpha_composite(sl, (x, y))
            if kind == "new":
                d.rectangle([x + 30, y + 74, x + 42, y + 78], fill=GOLD)
            x += sl.width + 2
        y += 100
    heads = ["", "in game now"] + [r[0] for r in RUNS]
    if not blocks:
        heads.append("11-04 / on the tool" if g == "tools" else ("approved 11-04" if g == "armour" else ""))
    for cx, t in zip(X, heads):
        d.text((cx, y), t, font=bold, fill=GOLD)
    gen_x = X[nr + 2] if blocks or g == "guides3" else X[nr + 2]
    gen_x = gen_x if blocks or g == "guides3" else X[nr + 1] + cell
    d.text((gen_x, y), "as generated", font=bold, fill=GOLD)
    note_x = gen_x + 170
    d.text((note_x, y), "what changed", font=bold, fill=GOLD)
    y += 30
    for label, c, n, third in G["rows"]:
        d.line([(24, y - 6), (W - 24, y - 6)], fill=(60, 44, 38))
        d.text((24, y + 50), label, font=bold, fill=INK)
        tint = "leaves" in n or n == "@grass_top"
        pics = [cur_icon(c)] + [run_icon(g, k, n) for k in range(1, nr + 1)]
        for x, ic in zip(X[1:nr + 2], pics):
            if ic is None:
                d.text((x, y + 60), "not made", font=font, fill=(150, 120, 110))
                continue
            im.alpha_composite(tile(ic, tint) if is_block(g, n) else pair(ic), (x, y))
        if third:
            im.alpha_composite(pair(cur_icon(third)), (X[nr + 2], y))
        for k in range(1, nr + 1):
            oy = (k - 1) * 70 if nr <= 2 else (k - 1) * 56
            gen = find(g, k, n, ".png")
            if gen:
                im.alpha_composite(Image.open(gen).convert("RGBA").resize((52, 52), Image.LANCZOS), (gen_x, y + oy))
                d.text((gen_x + 58, y + oy + 10), tries(g, k, n).replace(" tr", "\ntr", 1), font=small, fill=MUTED)
        text(d, (note_x, y + 4), notes.get(f"{g}:{n.lstrip('@')}", ""), font, INK, 62)
        y += rowh
    if "set_row" in G:
        y += 10
        d.text((24, y), "The full diamond set together (helmet, chestplate, leggings, boots) at 2x", font=bold, fill=GOLD)
        y += 30
        for label, names in G["set_row"]:
            d.text((24, y + 28), label, font=bold, fill=INK)
            x = 170
            for n in names:
                ic = run_icon(g, int(n[0]), n[2:]) if n[1:2] == ":" else cur_icon(n)
                sl = slot(ic, 2)
                im.alpha_composite(sl, (x, y))
                x += sl.width + 2
            y += 90
    if notes.get(f"{g}:verdict"):
        y += 10
        d.text((24, y), "Verdict", font=bold, fill=GOLD)
        y = text(d, (24, y + 26), notes[f"{g}:verdict"], font, INK, 230)
    im = im.crop((0, 0, W, y + 24))
    if not any(find(g, k, r[2], "_px.png") for k in range(1, nr + 1) for r in G["rows"]):
        continue  # no run made this group
    im.convert("RGB").save(os.path.join(out, f"regen{nr + 1}_{g}.png"))
    print(os.path.join(out, f"regen{nr + 1}_{g}.png"))
