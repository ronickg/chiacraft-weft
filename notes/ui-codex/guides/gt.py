"""A test of Ronald's style guides (~/Downloads/Game Art Style Guides): a wide, hard spread of our own
items (two to five from each of the guide's twelve kinds), a set of blocks and three of our animals,
each made with the matching guide's master prompt, material sheet and reference image, on the exact
32-pixel grid with its guide image (the way that worked best in the item lab).

    ~/.venvs/blocky/bin/python notes/ui-codex/guides/gt.py gen [id ...]   # Codex, four at a time; skips what exists
    ~/.venvs/blocky/bin/python notes/ui-codex/guides/gt.py data          # cut each to its pixel grid, write gt.json

Round 1 (out/, px/) used the guides as Ronald wrote them (~/Downloads/Game Art Style Guides). Round 2
(out2/, px2/) uses them as kept in the game-art skill (.claude/skills/game-art/guides), with what the
item lab and the model skill learned folded in (`0 Making it.md`), a green background for purple
things, and every image checked and made again (up to twice) when a check fails.
"""
import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
ROUND = int(os.environ.get("ROUND", "2"))
OUT = os.path.join(HERE, "out" if ROUND == 1 else "out2")
PX = os.path.join(HERE, "px" if ROUND == 1 else "px2")
G = (os.path.expanduser("~/Downloads/Game Art Style Guides") if ROUND == 1 else
     os.path.join(HERE, "..", "..", "..", ".claude", "skills", "game-art", "guides"))
UI_DIR = os.path.join(HERE, "..", "..", "..", "mods", "base", "source", "ui")
CODEX = "/Applications/Codex.app/Contents/Resources/codex-cli/bin/codex"
UI_REF = f"{G}/6 UI/UI Prompt References/Crafting & Workstations UI Reference.png"


def guide_section(path, start, end):
    """A slice of a guide, from the heading `start` up to the heading `end`."""
    t = open(path).read()
    t = t[t.index(start):t.index(end)]
    t = "\n".join(l[2:] if l.startswith("> ") else ("" if l == ">" else l) for l in t.split("\n"))
    return "\n".join(l for l in t.split("\n") if not l.startswith("How to prompt, check and cut it"))


ITEM_GUIDE = f"{G}/1 Items/Item Art Style Guide.md"
BLOCK_GUIDE = f"{G}/2 Blocks/Block Art Style Guide.md"
ANIMAL_GUIDE = f"{G}/4 Animals/Animal Sprite Style Guide.md"


def item_style():
    a = guide_section(ITEM_GUIDE, "**Style:**", "### Reference images to attach").strip()
    sheet = guide_section(ITEM_GUIDE, "## B) Material sheet", "### Example item requests")
    sheet = sheet.split("```")[1].strip()
    return a + "\n\nMaterial colors (outline colors are lit side / shadow side):\n" + sheet


def block_style():
    a = guide_section(BLOCK_GUIDE, "> **Style:**", "Add **one** of these").strip()
    return a, guide_section(BLOCK_GUIDE, "| Material | Ramp", "## C)").strip()


def animal_style():
    return guide_section(ANIMAL_GUIDE, "> **Style:**", "Add **one** of these").strip()


IREF = f"{G}/1 Items/Item Prompt References/"
BREF = f"{G}/2 Blocks/Block Prompt References/"
AREF = f"{G}/4 Animals/Animal Prompt References/"
DIAG = ", lying diagonally, handle at the bottom left, tip at the top right, filling the canvas corner to corner"

# id: (kind, guide reference image, what to draw)
ITEMS = {
    # 01 tools
    "wood_pickaxe": ("Tools", "01 Tools (diagonal).png",
                     "a wooden pickaxe: a crude carved warm orange-brown wooden head with clear grain, lashed to a "
                     "wooden handle with cream bone cord" + DIAG),
    "iron_axe": ("Tools", "01 Tools (diagonal).png",
                 "an iron axe: a bright warm-silver axe head with a near-white top bevel and a soft warm-grey "
                 "outline, a gold collar band, handle wrapped in red-brown leather with gold cross-bands, a round "
                 "gold pommel" + DIAG),
    "diamond_shovel": ("Tools", "01 Tools (diagonal).png",
                       "a diamond shovel: a faceted cyan diamond blade with white facet tips and one small sparkle, "
                       "a gold collar, red leather wrapped handle with a gold pommel" + DIAG),
    # 02 weapons
    "stone_sword": ("Weapons", "02 Weapons.png",
                    "a stone sword: a chunky chipped warm grey-tan stone blade, a plain wooden cross-guard, a wooden "
                    "grip bound with cream cord" + DIAG),
    "gold_sword": ("Weapons", "02 Weapons.png",
                   "a golden sword: a warm lemon-gold blade with orange-brown shadow and a bright highlight along "
                   "the edge, a gold guard with its own brown-gold outline, red leather grip, round gold pommel" + DIAG),
    "club": ("Weapons", "02 Weapons.png",
             "a heavy wooden club: a thick knobbly warm-brown wooden head with a few sharp stone chips set into it, "
             "narrowing to a grip wrapped in red-brown leather" + DIAG),
    # 03 armour
    "iron_chestplate": ("Armour", "03 Armour (front view).png",
                        "a plate chestplate, front view, warm-silver breastplate with rounded shoulder plates, a "
                        "gold collar at the neck, brown leather belt and straps at the waist, centred"),
    "diamond_helmet": ("Armour", "03 Armour (front view).png",
                       "a knight's great helm made of diamond, front view, rounded cyan diamond dome with a gold "
                       "crest ridge down the centre, a dark horizontal eye slit with gold trim above, short vertical "
                       "breathing slots below, centred"),
    "leather_boots": ("Armour", "03 Armour (front view).png",
                      "a pair of leather boots side by side, front view, soft warm-brown leather with stitching, a "
                      "folded cuff on top and a dark gap between them, centred"),
    # 04 ingots and raw ore
    "iron_ingot": ("Ingots", "04 Ingots and raw ores.png",
                   "an iron ingot: a warm-silver bar in three-quarter top view, tilted along the diagonal rising "
                   "to the top right, a near-white lit top face, a darker side face 2 px thick along the bottom "
                   "right, a faint embossed rune, centred"),
    "raw_gold": ("Ingots", "04 Ingots and raw ores.png",
                 "a lump of raw gold ore: a rough chunky nugget of warm lemon-gold with orange-brown shadows and "
                 "bits of grey stone stuck to it, centred"),
    # 05 gems and crystals
    "diamond": ("Gems", "05 Gems and crystals.png",
                "a diamond: a cluster of faceted cyan shards with teal shadow and white facet tips, one white "
                "plus-shaped sparkle, centred with 4 px of padding"),
    "amber": ("Gems", "05 Gems and crystals.png",
              "a chunk of amber: a rounded glossy honey-orange resin lump that looks translucent, a tiny dark "
              "insect inside, a pale yellow highlight on the upper left, centred"),
    "coal": ("Gems", "05 Gems and crystals.png",
             "a lump of coal: a chunky faceted black-brown lump, never pure black, with soft grey-blue glints on "
             "its upper-left facets, centred"),
    "redstone": ("Gems", "05 Gems and crystals.png",
                 "a pile of glowing red powder: a lumpy pile of saturated red nuggets with pink highlights, centred"),
    # 06 crafting materials
    "stick": ("Crafting materials", "06 Crafting materials.png",
              "a wooden stick: a slightly crooked warm-brown twig 3 px wide with grain and a small knot" + DIAG),
    "feather": ("Crafting materials", "06 Crafting materials.png",
                "a feather: a soft cream-white feather with a pale quill shaft, lying diagonally, shaft end at the "
                "bottom left"),
    "leather": ("Crafting materials", "06 Crafting materials.png",
                "a piece of leather hide: a soft irregular warm-brown hide shape with mottled texture, centred"),
    "string": ("Crafting materials", "06 Crafting materials.png",
               "a loose coil of pale cream string forming loops with a loose tail, centred"),
    "bone": ("Crafting materials", "06 Crafting materials.png",
             "a bone: warm cream bone with knobbed ends" + DIAG),
    # 07 food
    "apple": ("Food", "07 Food.png",
              "a red apple: round glossy vivid red with a pale streak on the upper left, a short stem and one "
              "green leaf, centred"),
    "bread": ("Food", "07 Food.png",
              "a loaf of bread in three-quarter view: a golden-brown crust with slashes on top and a cut end "
              "showing pale speckled crumb, centred"),
    "cooked_trout": ("Food", "07 Food.png",
                     "a cooked trout: a whole fish lying diagonally, head at the bottom left and tail at the top "
                     "right, golden-brown roasted skin with darker char marks, a visible fin on top"),
    "melon_slice": ("Food", "07 Food.png",
                    "a slice of melon: a crescent wedge with a green rind, a pale band and juicy pink-red flesh "
                    "with a few dark seeds, centred"),
    "apple_pie": ("Food", "07 Food.png",
                  "an apple pie in three-quarter top view: a round golden lattice crust over red-brown apple "
                  "filling, a crimped rim, centred"),
    "beef": ("Food", "07 Food.png",
             "a raw steak: a glossy red cut of meat with a cream fat edge and marbling, centred"),
    # 08 containers and liquids
    "water_bucket": ("Containers", "08 Containers and liquids.png",
                     "a metal bucket seen from the front and slightly above, slightly tapered, thick bright "
                     "warm-silver rim, vertical highlight just left of centre, a dark rope band around the middle, "
                     "small round handle lugs on both sides, filled to just below the rim with water: a flat oval "
                     "blue surface with one pale glint, centred"),
    "lava_bucket": ("Containers", "08 Containers and liquids.png",
                    "a metal bucket seen from the front and slightly above, slightly tapered, thick bright "
                    "warm-silver rim, vertical highlight just left of centre, a dark rope band around the middle, "
                    "small round handle lugs on both sides, filled to just below the rim with lava: a flat oval "
                    "red surface with bright orange swirls, centred"),
    "sedative": ("Containers", "08 Containers and liquids.png",
                 "a round glass potion flask, front view, white glass outline with a pale inner edge, transparent "
                 "glass with one curved white highlight on the upper left, small tan cork, a white cloth tied "
                 "round the neck with thin red cords, filled two-thirds with a sleepy lavender-purple liquid"),
    # 09 paper and books
    "book": ("Paper and books", "09 Paper and books.png",
             "a closed leather book lying at an angle, deep red cover with a darker border, a cream page block "
             "4 px thick showing on the lower-left edge, dark brown spine, small metal corner caps"),
    "paper": ("Paper and books", "09 Paper and books.png",
              "a slightly crumpled square sheet of parchment lying at an angle, soft cream with gentle fold "
              "shading and a warm brown edge"),
    # 10 small tricky objects
    "spyglass": ("Small tricky objects", "10 Small tricky objects.png",
                 "a spyglass lying diagonally from the bottom left (eyepiece) to the top right (lens): a "
                 "copper-and-leather tube in three segments that get wider toward the lens, a pale blue glint on "
                 "the lens"),
    "torch": ("Small tricky objects", "10 Small tricky objects.png",
              "a torch, upright and centred: a short wooden handle 3-4 px wide with a small iron collar, a "
              "teardrop flame on top, dark orange at its edge, orange in the body and yellow-white in the core"),
    "lever": ("Small tricky objects", "10 Small tricky objects.png",
              "a lever: a small grey cobblestone base plate in three-quarter top view with a wooden handle "
              "sticking up and tilted to the right, centred"),
    # 11 soft and organic
    "raptor_egg": ("Soft and organic", "11 Soft and organic.png",
                   "a large raptor egg: an upright oval, slightly narrower at the top, pale olive-cream shell with "
                   "dark brown speckles, a soft highlight on the upper left, centred"),
    "utahraptor_saddle": ("Soft and organic", "11 Soft and organic.png",
                          "a saddle seen from the side and slightly above: a dark brown leather seat with a raised "
                          "back, a stirrup hanging below with a small metal ring, a few metal studs and a tuft of "
                          "feathers tied at the back, centred"),
    "mushroom_red": ("Soft and organic", "11 Soft and organic.png",
                     "a red mushroom: a round red cap with cream spots on a short pale stem, centred"),
    "earthworm": ("Soft and organic", "11 Soft and organic.png",
                  "an earthworm: a pink-brown segmented worm curled in an S shape, faint ring segments and a "
                  "slightly paler band, centred"),
    # 12 arrows, instruments and charms
    "arrow": ("Arrows and charms", "12 Arrows, instruments and charms.png",
              "an arrow lying diagonally, thin wooden shaft, small pale steel arrowhead at the top right, "
              "cream-and-brown feather fletching at the bottom left"),
    "tranquilizer_dart": ("Arrows and charms", "12 Arrows, instruments and charms.png",
                          "a tranquilizer dart lying diagonally: a slim wooden shaft, a thin pale steel needle tip "
                          "at the top right, a small glass vial of lavender liquid behind the tip, a tuft of red "
                          "feathers at the bottom left"),
}

# id: (type line from the block guide, reference image, what to draw, full tile or cutout)
BLOCKS = {
    "stone": ("Seamless tileable full-block texture, no border, evenly spread detail, no single feature that "
              "stands out when repeated.", "01 Natural cube.png",
              "large sage-grey stone slabs (#3f4341 to #a8bab1) with deep sage crevices, 6-9 slabs", True),
    "dirt": ("Seamless tileable full-block texture, no border, evenly spread detail, no single feature that "
             "stands out when repeated.", "01 Natural cube.png",
             "rounded brown dirt clods (#3a2926 to #94724b) with dark gaps between them, 10-15 clods", True),
    "cobblestone": ("Seamless tileable full-block texture, no border, evenly spread detail, no single feature "
                    "that stands out when repeated.", "01 Natural cube.png",
                    "8-12 rounded cobblestones (#2c2f2c to #97a89d) packed together with dark sage mortar", True),
    "grass_top": ("Seamless leafy grass tufts covering the whole tile.", "02 Grass block.png",
                  "muted natural green grass tufts (#4c642f #617e3b #779c49 #8fba57), in full color", True),
    "grass_side": ("Dirt clods with a grass fringe hanging 4-8 px down from the top edge, irregular tips; "
                   "seamless left to right.", "02 Grass block.png",
                   "brown dirt clods (#3a2926 to #94724b) under a muted green grass fringe (#4c642f to #8fba57)",
                   True),
    "log_oak": ("Vertical bark with deep grooves, tileable top-to-bottom and left-to-right.", "04 Logs.png",
                "oak bark (#2a2319 #362c1f #443927 #5b4b33 #7e6643), nudged warmer", True),
    "planks_oak": ("4 horizontal boards. 1-px mortar lines in the darkest shade, light top-left edge on each "
                   "piece, joints line up across tile edges so it tiles.", "05 Planks bricks built stone.png",
                   "warm walnut-to-honey oak (#221c16 to #886f4a), nudged toward the UI's wood #bc6618 for a "
                   "warmer look", True),
    "iron_ore": ("Exactly the same stone base as sage-grey stone (#3f4341 to #a8bab1, large slabs with deep "
                 "crevices), with 3-6 rounded nuggets of iron in a deep socket of the stone's own dark shade (not "
                 "black), a near-white sparkle on each, filling about 30% of the tile. Seamless.", "03 Ores.png",
                 "warm tan-copper iron nuggets (#784b38 to #e0c8a8) with pale highlights", True),
    "leaves_oak": ("Overlapping individual leaves 3-5 px long with transparent gaps covering 25-35% of the "
                   "tile, tileable, in full color.", "07 Leaves.png",
                   "oak leaves (#27380f #344b14 #49691c #669328 #74a72e)", False),
    "flower_rose": ("A single red poppy standing upright, centred, base touching the bottom edge, about 20-26 px "
                    "tall, on a transparent background, readable from its silhouette alone. A thin 1-2 px stem "
                    "and 2-4 leaves.", "08 Flowers saplings mushrooms.png",
                    "a red poppy (#d32228, shadow #80161b) with a dark centre, a green stem (#2f5b28) and two "
                    "leaves", False),
    "crafting_table_front": ("Front face of a crafting table showing what it does, framed in wood with iron "
                             "bands.", "11 Workstations.png",
                             "warm oak frame with a saw, hammer and tongs hanging on the front panel", True),
}

# id: (type line, reference card, what to draw); all 48 x 32 sprites, side view facing right
ANIMALS = {
    "capybara": ("Stocky rectangular body, short sturdy legs, friendly face with a lighter muzzle.",
                 "03 Wild mammals.png",
                 "a capybara: a barrel-shaped body with coarse warm reddish-brown fur (#5a3a25 to #a8744a), a big "
                 "blunt square head, small round ears, tiny eye high on the head, no tail"),
    "chihuahua": ("Small and agile, a large head and ears relative to the body, a bushy or long tail, cream "
                  "countershaded belly and muzzle.", "03 Wild mammals.png",
                  "a chihuahua: a tiny dog with a fawn coat (#b9875a to #e8c497), huge upright ears with pink "
                  "insides, a round apple head, big dark eyes, thin legs and a curled tail, standing small in "
                  "the canvas (about 20 px tall)"),
    "utahraptor": ("Large reptile with a long tail balancing a heavy head, scales as a subtle 2-shade pebble "
                   "pattern, a lighter belly, bold back stripes or spots, plates or feathers in an accent color.",
                   "01 Farm animals.png",
                   "a utahraptor: olive-green scales (#3e5a2a to #8fb24d), a cream belly, rust-red back stripes, "
                   "a curved claw on each foot, feathered arms and a quill-feather crest in burnt orange, the "
                   "long tail held out level behind it"),
}

GRID = ("Draw it on an exact {w} x {h} pixel grid: the image is {W} x {H}, so every sprite pixel is exactly one "
        "32 x 32 block of a single flat color, all blocks aligned to a grid starting at the top-left corner of "
        "the image. No anti-aliasing, no blur, no soft edges, no gradients inside a block, no half blocks. The "
        "last attached image is that grid (cells on magenta): put every pixel exactly in one of its cells, but "
        "do not draw the grid lines.")
TAIL = ("Use your image generation tool to make ONE image, then save it as a PNG file named {name}.png in the "
        "current folder. Do not write any code or edit any other files. When done, reply with the file name.")
# things that are purple or pink themselves: on green, so the background can be told apart (round 2)
GREEN = {"sedative", "tranquilizer_dart"}


def corner_green(path):
    """Whether the image's background is green rather than magenta: the model picks green for pink and
    purple things by itself (the guides tell it to), so the cutter looks instead of assuming."""
    import numpy as np
    from PIL import Image
    a = np.array(Image.open(path).convert("RGB")).astype(int)
    c = np.concatenate([a[:16, :16], a[:16, -16:], a[-16:, :16], a[-16:, -16:]]).reshape(-1, 3)
    green = ((c[:, 1] > 190) & (c[:, 0] < 90) & (c[:, 2] < 90)).mean()
    magenta = ((c[:, 0] > 190) & (c[:, 1] < 90) & (c[:, 2] > 190)).mean()
    return bool(green > magenta)


def key_color(key):
    return "green" if ROUND > 1 and key in GREEN else "magenta"


def bg_words(key):
    return "solid flat #00FF00 green" if key_color(key) == "green" else "solid flat #FF00FF magenta"


def text_for(kind, key):
    bg = bg_words(key)
    if kind == "item":
        k, ref, what = ITEMS[key]
        t = (f"Make one item icon for my block game.\n\n{item_style()}\n\n"
             f"{GRID.format(w=32, h=32, W=1024, H=1024)} The item sits inside the 32 x 32 grid; everything "
             f"outside it is {bg} (it will be cut out). Use about 16-32 colors.\n\n"
             "Attached: first, reference items of this kind: take their size, outline, light and level of "
             "detail, not their exact designs. Second, the game's UI: its colors and the dark brown slot "
             "(#352015) the icon must read on.\n\n"
             f"The item: {what}.\n\n" + TAIL.format(name=key))
        return t, [IREF + ref, UI_REF, os.path.join(HERE, "guide32.png")], (32, 32)
    if kind == "block":
        line, ref, what, full = BLOCKS[key]
        style, ramps = block_style()
        fill = ("Every one of the 32 x 32 cells is filled: the texture covers the whole image edge to edge and "
                "tiles seamlessly." if full else
                f"Where there is nothing (gaps, around the plant) the image is {bg}, which stands for "
                "transparent.")
        t = (f"Make one block texture for my block game.\n\n{style}\n\n{line}\n\nMeasured color ramps:\n{ramps}\n\n"
             f"{GRID.format(w=32, h=32, W=1024, H=1024)} {fill}\n\n"
             "Attached: first, reference block textures of this kind (cubes are shown with a 3 x 3 tiled patch "
             "under them): take their look, not their exact designs. Second, the game's UI, for its colors.\n\n"
             f"The block: {what}.\n\n" + TAIL.format(name=key))
        if ROUND > 1:
            t = t.replace("in full color", "in full color (for this look test; in the game it is drawn grey "
                                           "and tinted)")
        return t, [BREF + ref, UI_REF, os.path.join(HERE, "guide32.png")], (32, 32)
    line, ref, what = ANIMALS[key]
    style = animal_style().replace("[animal]", key).replace("[W×H]", "48 x 32")
    t = (f"Make one animal sprite for my block game.\n\n{style}\n\n{line}\n\n"
         f"{GRID.format(w=48, h=32, W=1536, H=1024)} Make the image landscape, 1536 x 1024. The animal stands on "
         f"the bottom row of the grid; everything around it is {bg} (it will be cut out).\n\n"
         "Attached: first, faces, fur and markings cut flat from the reference's animal skins (take colors, fur "
         "texture and faces from them, not the pose). Second, reference item icons, for the outline and light. "
         "Third, the game's UI, for its colors.\n\n"
         f"The animal: {what}.\n\n" + TAIL.format(name=key))
    return t, [AREF + ref, IREF + "07 Food.png", UI_REF, os.path.join(HERE, "guide48.png")], (48, 32)


def all_jobs():
    return ([("item", k) for k in ITEMS] + [("block", k) for k in BLOCKS] + [("animal", k) for k in ANIMALS])


def guides():
    from PIL import Image, ImageDraw
    for name, (w, h) in (("guide32.png", (1024, 1024)), ("guide48.png", (1536, 1024))):
        im = Image.new("RGB", (w, h), (255, 0, 255))
        d = ImageDraw.Draw(im)
        for k in range(0, w + 1, 32):
            d.line([(k, 0), (k, h - 1)], fill=(200, 0, 200))
        for k in range(0, h + 1, 32):
            d.line([(0, k), (w - 1, k)], fill=(200, 0, 200))
        im.save(os.path.join(HERE, name))


# ---------------------------------------------------------------- checks (round 2)

def is_key(rgb, key):
    """The background color's pixels (magenta, or green for purple things; `key` may be True/False for
    a background already looked at)."""
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    if key is True or (key is not False and key_color(key) == "green"):
        return (g > 190) & (r < 90) & (b < 90)
    return (r > 190) & (g < 90) & (b > 190)


def check(kind, key, path):
    """The rules from `0 Making it.md`, measured: chunky pixels on the grid, an outline darker than the
    inside and not the red-brown line, a flat background, and for full blocks no empty cell and no
    seam. Returns (passed, {rule: value})."""
    import numpy as np
    from PIL import Image
    sys.path.insert(0, UI_DIR)
    import check_items
    import outline
    _, _, (w, h) = text_for(kind, key)
    rgb = np.array(Image.open(path).convert("RGB").resize((w * 32, h * 32), Image.NEAREST)).astype(float)
    full = kind == "block" and BLOCKS[key][3]
    keep = np.ones(rgb.shape[:2], bool) if full else ~is_key(rgb, corner_green(path))
    res = {}
    strength, _ = check_items.grid_strength(rgb, keep)
    res["grid"] = round(strength, 3)
    ok = strength >= 0.12
    sprite, _ = snap(path, w, h, full, key)
    if full:
        res["empty"] = int((sprite[..., 3] == 0).sum())
        a = sprite[..., :3].astype(float)
        inner = (np.abs(np.diff(a, axis=1)).mean() + np.abs(np.diff(a, axis=0)).mean()) / 2
        seam = (np.abs(a[:, 0] - a[:, -1]).mean() + np.abs(a[0] - a[-1]).mean()) / 2
        res["seam"] = round(seam / max(inner, 1e-9), 2)  # 1 = the edge looks like any other neighbour pair
        ok &= res["empty"] == 0 and res["seam"] < 2.2
    else:
        m = sprite[..., 3] > 0
        if m.any():
            lum = sprite[..., :3].astype(float).mean(-1)
            edge = m & ~(np.pad(m, 1)[2:, 1:-1] & np.pad(m, 1)[:-2, 1:-1] & np.pad(m, 1)[1:-1, 2:] & np.pad(m, 1)[1:-1, :-2])
            inside = m & ~edge
            res["outline"] = round(float(np.median(lum[edge]) / max(np.median(lum[inside]), 1)), 2) if inside.any() else 1
            res["red_line"] = round(outline.share(sprite[..., :3].astype(float), m), 2)
            pale = kind == "item" and key in ("feather", "string", "paper", "bone", "sedative", "earthworm")
            ok &= (res["outline"] < 0.9 or pale) and res["red_line"] < 0.45
            fill = m.mean()
            res["fill"] = round(float(fill), 2)
            ok &= 0.04 < fill < 0.85
        else:
            ok = False
    return bool(ok), res


def run(job):
    kind, key = job
    out = os.path.join(OUT, key + ".png")
    if os.path.exists(out):
        return f"{key} exists"
    text, refs, _ = text_for(kind, key)
    tries = []
    for attempt in range(3 if ROUND > 1 else 1):
        args = [CODEX, "exec", "--skip-git-repo-check", "--sandbox", "workspace-write", "--ephemeral"]
        for r in refs:
            args += ["-i", r]
        args += ["-o", os.path.join(OUT, f"last_{key}.txt"), text]
        with open(os.path.join(OUT, f"run_{key}.log"), "a") as log:
            subprocess.run(args, cwd=OUT, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT)
        if not os.path.exists(out):
            tries.append({"made": False})
            continue
        if ROUND == 1:
            break
        ok, res = check(kind, key, out)
        tries.append(dict(res, passed=ok))
        if ok:
            break
        os.rename(out, os.path.join(OUT, f"{key}__try{attempt + 1}.png"))
    if ROUND > 1:
        json.dump(tries, open(os.path.join(OUT, f"checks_{key}.json"), "w"))
        if not os.path.exists(out):  # none passed: keep the last one made, marked
            made = [k for k in range(len(tries), 0, -1) if os.path.exists(os.path.join(OUT, f"{key}__try{k}.png"))]
            if made:
                import shutil
                shutil.copy(os.path.join(OUT, f"{key}__try{made[0]}.png"), out)
    return f"{key} {'ok' if os.path.exists(out) else 'FAILED'} after {len(tries)} " + \
        ("" if ROUND == 1 else str([t.get("passed") for t in tries]))


def gen(keys):
    os.makedirs(OUT, exist_ok=True)
    guides()
    jobs = [j for j in all_jobs() if not keys or j[1] in keys]
    with ThreadPoolExecutor(4) as ex:
        for r in ex.map(run, jobs):
            print(r, flush=True)


# ---------------------------------------------------------------- cut to the pixel grid

def snap(path, w, h, full, key=""):
    """The image as an exact w x h sprite: the 32-px grid's phase from where color edges fall, each cell
    the most common color in its middle, transparent where the middle is the background color."""
    import numpy as np
    from PIL import Image
    src = np.array(Image.open(path).convert("RGB").resize((w * 32, h * 32), Image.NEAREST)).astype(float)
    if not full:
        key = corner_green(path)
    phase = []
    for ax in (1, 0):
        g = np.abs(np.diff(src, axis=ax)).sum(-1).sum(1 - ax)
        fold = np.bincount(np.arange(len(g)) % 32, weights=g, minlength=32)
        phase.append(((int(np.argmax(fold)) + 1) % 32, fold.max() / max(fold.sum(), 1e-9)))
    (px, sx), (py, sy) = phase
    px, py = (px if px < 16 else px - 32), (py if py < 16 else py - 32)
    out = np.zeros((h, w, 4), np.uint8)
    H, W = src.shape[:2]
    for j in range(h):
        for i in range(w):
            y0, x0 = py + j * 32 + 8, px + i * 32 + 8
            cell = src[max(0, y0):min(H, y0 + 16), max(0, x0):min(W, x0 + 16)].reshape(-1, 3)
            if not len(cell):
                continue
            mag = is_key(cell, key)
            if not full and mag.mean() > 0.5:
                continue
            c = cell[~mag] if (~mag).any() else cell
            cols, counts = np.unique((c // 4).astype(int), axis=0, return_counts=True)
            out[j, i, :3] = cols[counts.argmax()] * 4 + 2
            out[j, i, 3] = 255
    return out, float((sx + sy) / 2)


def data():
    from PIL import Image
    rows = []
    os.makedirs(PX, exist_ok=True)
    for kind, key in all_jobs():
        src = os.path.join(OUT, key + ".png")
        if not os.path.exists(src):
            continue
        _, _, (w, h) = text_for(kind, key)
        full = kind == "block" and BLOCKS[key][3]
        sprite, share = snap(src, w, h, full, key)
        Image.fromarray(sprite).save(os.path.join(PX, key + ".png"))
        group = ITEMS[key][0] if kind == "item" else kind
        colors = len({tuple(p[:3]) for p in sprite.reshape(-1, 4) if p[3]})
        row = {"id": key, "kind": kind, "group": group, "w": w, "h": h, "grid": round(share, 3), "colors": colors}
        cpath = os.path.join(OUT, f"checks_{key}.json")
        if os.path.exists(cpath):
            row["tries"] = json.load(open(cpath))
        rows.append(row)
    json.dump(rows, open(os.path.join(HERE, f"gt{ROUND}.json"), "w"), indent=1)
    print(len(rows), "cut")


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "data"
    if cmd == "gen":
        gen(sys.argv[2:])
    elif cmd == "prompt":
        kind, key = next(j for j in all_jobs() if j[1] == sys.argv[2])
        print(text_for(kind, key)[0])
    elif cmd == "check":
        kind, key = next(j for j in all_jobs() if j[1] == sys.argv[2])
        print(check(kind, key, sys.argv[3]))
    else:
        data()
