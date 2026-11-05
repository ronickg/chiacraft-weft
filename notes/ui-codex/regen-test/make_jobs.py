"""The art regen test's jobs (Ronald, 11-05: "rerun this skill ... diamond armor and diamond tools, and
maybe some food ... compare how the new generated thing will look like versus the old one"): written
from the item guide's material sheet and recipes, run with `art.py batch`.

    python3 notes/ui-codex/regen-test/make_jobs.py > jobs.json         # run 1 (11-05): the old guides
    python3 notes/ui-codex/regen-test/make_jobs.py v2 > jobs_v2.json   # run 2: guides 2, sheets attached
    python3 notes/ui-codex/regen-test/make_jobs.py blocks > blocks.json  # 4 blocks, both ways

Run 2 is the same requests with Ronald's guides 2 (11-05): each attaches the guide's sheet for its kind
and, for a whole set, the set sheet (13 tools and weapons by tier, 16 all food), named `n2_...`.
"""
import json
import sys

D = ("diamond (#48aeba base, #468594/#3a5a62 teal shadow, #c6f9fb to white facet tips, faceted with small "
     "white sparkles, #3a8490/#26555b teal outline, lighter on the upper left)")
OAK = ("a plain oak haft 3 px thick from end to end (warm orange-brown #c1600f base, #8d3e05 shadow, #ee973f "
       "highlight on its upper-left side, gentle grain streaks, #5c1e02 outline)")
IRON = "wrought iron (warm silver #c7b7a7 base, #8a6656 shadow, #f8f1e1 bevel, #695549 outline)"
LEATHER = "brown leather (#8c5236 base, #633a28 shadow, #b06e45 highlight, #4d2b20 outline)"
SCALES = (f"overlapping faceted {D} scales sewn onto a dark backing, each scale with a pale top-left facet")

jobs = []
def job(name, what, **kw):
    jobs.append(dict(kind="item", name=name, what=what, **kw))

# Armour: the shells (the diamond part) and the built pieces (shell, straps, lining).
job("rt_helmet_shell", f"the shell of a helmet, a loose armour part with no straps or lining: a rounded dome narrowing to the "
    f"cheeks, made of {SCALES}, a dark horizontal eye slit across the middle and short breathing slots below, the "
    "inside showing dark at the open bottom edge; front view, centred, about 20 x 24 px")
job("rt_chest_shell", f"the shell of a chestplate, a loose armour part with no straps or lining: a breastplate with rounded "
    f"shoulder plates and a slight V down the middle, made of {SCALES}; front view, centred, about 28 x 25 px")
job("rt_leg_shell", f"the shell of a pair of leggings, a loose armour part with no belt or lining: two plated legs with knee "
    f"plates and a dark gap between them, made of {SCALES}; front view, centred, about 24 x 26 px")
job("rt_boot_shell", f"the shells of a pair of boots, a loose armour part with no straps or lining: two plated boot shells "
    f"seen a little from the side so each foot and toe shows at the bottom, made of {SCALES}; centred, about 24 x 20 px")
job("rt_diamond_helmet", f"a diamond helmet: a knight's great helm, front view, rounded dome of {SCALES} with a raised "
    f"ridge down the centre, a dark horizontal eye slit with a {IRON} trim above it, short vertical breathing slots "
    f"below, {LEATHER} chin straps with a small iron buckle at the sides; centred, about 20 x 24 px")
job("rt_diamond_chestplate", f"a diamond chestplate, front view: a breastplate of {SCALES} with rounded shoulder plates, "
    f"a slight V down the middle, {LEATHER} belt and shoulder straps with small {IRON} buckles, a cream wool lining "
    "showing at the neck; centred, about 28 x 25 px")
job("rt_diamond_leggings", f"diamond leggings, front view: a {LEATHER} belt with a small {IRON} buckle at the top, two "
    f"legs plated with {SCALES} with knee plates, a dark gap between the legs; centred, about 24 x 24 px")
job("rt_diamond_boots", f"diamond boots: a pair seen a little from the side so each foot and toe shows at the bottom, "
    f"shin and toe plates of {SCALES} over {LEATHER} with small iron buckles; centred, about 24 x 20 px")

# Tool parts in diamond, loose (no handle).
loose = "a loose tool part with no handle, as it lies in the inventory"
job("rt_part_long_blade", f"a long sword blade of {D}, {loose}: a long straight double-edged blade with a sharp tip and a "
    "short bare tang at its base, a near-white bevel along its upper edge; lying diagonally, tang bottom left, tip top "
    "right, about 28 px corner to corner", diagonal=True)
job("rt_part_short_blade", f"a short sword blade of {D}, {loose}: a short broad double-edged blade with a sharp tip and a "
    "short bare tang, a near-white bevel along its upper edge; lying diagonally, tang bottom left, tip top right, about "
    "20 px corner to corner, centred", diagonal=True)
job("rt_part_axe_head", f"a felling axe head of {D}, {loose}: a wide flared cutting edge on one side, a short blunt poll on "
    "the other and a dark round eye (the socket for the haft) in the middle, faceted like a cut gem; the head turned "
    "so its cutting edge runs along the corner-to-corner diagonal, never upright; centred, about 18 x 20 px")
job("rt_part_pick_head", f"a pickaxe head of {D}, {loose}: a curved crescent with two pointed ends and a dark round eye "
    "(the socket for the haft) in the middle, faceted like a cut gem, near-white highlights along its top curve; "
    "centred, the crescent spanning about 22 px")
job("rt_part_shovel_blade", f"a shovel blade of {D}, {loose}: a rounded spade blade with a short socket collar at its top "
    "(the socket for the haft), faceted like a cut gem, a near-white highlight on its upper left; tilted so the socket "
    "points to the bottom left, centred, about 16 x 20 px")
job("rt_part_hoe_blade", f"a hoe blade of {D}, {loose}: one flat blade about 10 px wide and 4-5 px tall with a short neck "
    "and a dark round eye for the haft, faceted like a cut gem; centred")
job("rt_part_spear_point", f"a leaf-shaped spear point of {D}, {loose}: a leaf-shaped double-edged point with a short "
    "socket at its base, a raised midrib and near-white facet tips; lying diagonally, socket bottom left, tip top right, "
    "centred, about 16 px corner to corner", diagonal=True)

# Built tools, as the game composes them: the diamond working part on an oak haft or grip, iron fittings.
tool = "lying diagonally, handle bottom left, tip top right, filling the canvas corner to corner"
job("rt_diamond_sword", f"a sword built from parts: a long straight double-edged blade of {D} with a near-white bevel "
    f"along its upper edge, a short bar guard and a round pommel of {IRON}, and a grip of oak wood (warm orange-brown "
    f"#c1600f, #8d3e05 shadow, #5c1e02 outline); {tool}", diagonal=True)
job("rt_diamond_pickaxe", f"a pickaxe built from parts: a curved crescent head of {D} with two pointed ends and "
    f"near-white highlights along its top, fixed by a small {IRON} wedge on {OAK}; {tool}", diagonal=True)
job("rt_diamond_axe", f"an axe built from parts: a felling head of {D}, sitting crosswise at the top of the haft, its blade "
    "sticking out to the upper left with the cutting edge parallel to the handle (along the corner-to-corner "
    f"diagonal) and a short blunt poll to the lower right, fixed by a small {IRON} wedge on {OAK}; {tool}", diagonal=True)
job("rt_diamond_shovel", f"a shovel built from parts: a rounded spade blade of {D} with a near-white highlight on its "
    f"upper left, fixed by a small {IRON} collar on {OAK}; {tool}", diagonal=True)
job("rt_diamond_hoe", f"a hoe built from parts: one flat blade of {D} about 10 px wide and 4-5 px tall, set at a right "
    f"angle across the top of the handle and pointing down-left, fixed by a small {IRON} wedge on {OAK}; {tool}",
    diagonal=True)
job("rt_diamond_spear", f"a spear built from parts: a leaf-shaped point of {D} with a raised midrib and near-white facet "
    f"tips, lashed with a small {IRON} collar to a long oak shaft 2-3 px thick (warm orange-brown #c1600f, #8d3e05 shadow, "
    f"#5c1e02 outline); {tool}", diagonal=True)

# Food, across kinds.
job("rt_bread", "a loaf of bread: crust #9a5935 with #5c3d29 shadow and a few scored diagonal cuts on top, one end cut to "
    "show the cream crumb #e9d6b0 with fine speckle #deca9d, #5b3722/#543320 outline; three-quarter top view, lying "
    "diagonally, centred, about 26 x 20 px")
job("rt_apple", "a red apple: round glossy #d8221e with #be1916 shadow (#a21310 deepest), #f36555 highlight and a short "
    "pale streak on the upper left, a small brown stem and one green leaf (#5e9a2c, #395123 outline) on top, "
    "#85251a/#5e1811 outline; centred, about 22 x 24 px")
job("rt_cooked_beef", "a cooked steak: warm brown #764e3d with #48352d shadow and #9d7553 highlights, darker diagonal grill "
    "marks, a thin cream fat edge along one side, #38251e outline; three-quarter top view, centred, about 26 x 22 px")
job("rt_beef", "a raw beef steak: vivid red #d9443e meat with #a21310 shadow and #f36555 highlight, white-pink marbling "
    "streaks, a cream fat rim along one side, #85251a/#5e1811 outline; three-quarter top view, centred, about 26 x 22 px")
job("rt_trout", "a raw trout lying diagonally, head at the bottom left and tail at the top right, about 26 x 28 px: olive-"
    "green back (#6f7a3c) with small dark spots, a pink-red stripe along its side (#d9706a), a pale silver-cream belly "
    "(#e6dcc4), a fin on top, a small dark eye, scales suggested by soft speckle, outlined in a darker shade of each part")
job("rt_stew", "a wide wooden bowl (#86673c to #3f3023) seen from the front and slightly above, oval opening showing the "
    "inside, bright rim, filled with a hearty meat stew: orange-red broth with brown meat chunks, orange carrot pieces and "
    "a few green herb flecks, a wooden spoon handle sticking out to the upper left; centred, about 26 x 18 px")
job("rt_egg", "a chicken's egg: an upright oval slightly narrower at the top, warm cream #f9d6a7 shading to #bb905c on the "
    "lower right, a soft highlight on the upper left, plain, outline no darker than #6b4e35; centred, about 14 x 18 px "
    "(a small thing, about 60-70% of the canvas)", pale=True)
job("rt_carrot", "a carrot: bright orange #e77f0d with #cc6c0b shadow (#7f3906 deepest), #e9b32a highlight and a few "
    "small ring lines across it, tapering to a point at the bottom left, a tuft of green leaves (#5e9a2c, #395123 "
    "outline) at the top right, #7f3906 outline; lying diagonally, centred, about 24 x 24 px")

# Sheets for run 2: the kind's sheet first (its family notes go in the prompt), then the set sheet.
SHEETS = {"shell": "03", "helmet": "03", "chestplate": "03", "leggings": "03", "boots": "03",
          "axe": "01,13", "pick": "01,13", "shovel": "01,13", "hoe": "01,13",
          "blade": "02,13", "spear": "02,13", "sword": "02,13", "point": "02,13"}
FOOD = {"n2_egg": "11,16", "n2_stew": "11,16", "n2_trout": "11,16"}


def sheet(name):
    if name in FOOD:
        return FOOD[name]
    for k, v in SHEETS.items():
        if k in name:
            return v
    return "07,16"


B = [("stone", "natural cube", False,
      "large sage-grey stone slabs (#3f4341, #616661, #81938a, #90a398, #a8bab1) with deep sage crevices, 6-9 slabs"),
     ("diamond_ore", "ore", False,
      "the same sage-grey slab stone as plain stone (#3f4341 to #a8bab1), with 2-3 faceted diamond crystals (#48aeba base, "
      "#468594/#3a5a62 teal shadow, #c6f9fb to white facet tips) in deep sockets of the stone's own dark shade, a near-white "
      "sparkle on each"),
     ("log_oak", "log side", False,
      "oak log bark (#2a2319, #362c1f, #443927, #5b4b33, #7e6643): vertical bark with deep grooves and ridges, tileable top "
      "to bottom"),
     ("leaves_oak", "leaves", True,
      "oak leaves: overlapping rounded oak leaves 3-5 px long with transparent gaps covering 25-35% of the tile, tileable, "
      "drawn in light greyscale for biome tinting (the shading of #27380f to #74a72e turned grey)")]

if sys.argv[1:] == ["v2"]:
    for j in jobs:
        j["name"] = "n2_" + j["name"][3:]
        j["ref"] = sheet(j["name"])
elif sys.argv[1:] == ["blocks"]:
    jobs = []
    for name, t, cut, what in B:
        jobs.append(dict(kind="block", name="b1_" + name, type=t, cutout=cut, what=what, no_ref=True))
        jobs.append(dict(kind="block", name="b2_" + name, type=t, cutout=cut, what=what))
print(json.dumps(jobs, indent=1))
