"""The item icon redo with the game-art skill (art.py, Ronald's style guides, our own style key):
every item icon of the base and animals mods, plus Survival's new items, in batches by kind.

    python3 notes/ui-codex/redo/queue.py            # writes batches/<n>_<name>.json and prints the counts
    ART_DIR=notes/ui-codex/redo/out ~/.venvs/blocky/bin/python .claude/skills/game-art/scripts/art.py batch notes/ui-codex/redo/batches/1_tools.json

Each `what` follows the item guide: material and colors first, then shape, then view and placement.
"""
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
DIAG = ", lying diagonally, handle at the bottom left, tip at the top right, filling the canvas corner to corner"
C = ", centred with 3-5 px of padding"  # everything that isn't a tool

# ---------------------------------------------------------------- tools and weapons (item guide: 01 Tools, 02 Weapons)

TIER = {
    "wood": ("warm orange-brown wood with clear grain, crudely carved, lashed with cream cord", "plain wooden handle"),
    "stone": ("chunky chipped warm grey-tan stone with rough bumpy edges, no shine, bound with cream cord",
              "plain wooden handle"),
    "iron": ("bright warm-silver iron with a near-white top bevel and a soft warm-grey outline, a gold collar band",
             "handle wrapped in red-brown leather with gold cross-bands and a round gold pommel"),
    "gold": ("shiny lemon-gold with orange-brown shadows and small white glints, a gold collar with its own brown-gold outline",
             "handle wrapped in red-brown leather with gold bands and a round gold pommel"),
    "diamond": ("faceted cyan diamond with teal shadows, white facet tips and one small sparkle, a gold collar",
                "handle wrapped in red-brown leather with gold bands and a round gold pommel"),
}
SHAPE = {
    "pickaxe": "pickaxe: a curved crescent head spanning about two-thirds of the canvas, made of {m}, on a {h}",
    "axe": "axe: a broad wedge-shaped blade flaring to a long straight edge, made of {m}, on a {h}",
    "shovel": "shovel: a rounded spade blade made of {m}, on a {h}",
    "hoe": "hoe: a big flat blade, about a third of the canvas wide, set at a right angle across the top of the "
           "shaft and reading clearly at slot size, made of {m}, on a {h}",
    "sword": "sword: a straight blade made of {m}, a cross-guard, on a short {h}",
}
TOOLS = [(f"{t}_{k}", f"{'an' if t == 'iron' else 'a'} {t if t != 'wood' else 'wooden'} " + SHAPE[k].format(m=m, h=h) + DIAG)
         for k in SHAPE for t, (m, h) in TIER.items()]
TOOLS += [
    ("club", "a heavy wooden club: a thick knobbly warm-brown wooden head with a few sharp stone chips set into it, "
             "narrowing to a grip wrapped in red-brown leather" + DIAG),
    ("arrow", "an arrow lying diagonally, thin wooden shaft, small pale steel arrowhead at the top right, "
              "cream-and-brown feather fletching at the bottom left"),
    ("tranquilizer_dart", "a tranquilizer dart lying diagonally: a slim wooden shaft, a thin pale steel needle tip at "
                          "the top right, a small glass vial of lavender liquid behind the tip, a tuft of red feathers "
                          "at the bottom left"),
]

# ---------------------------------------------------------------- armour (03 Armour, front view)

ARM = {
    "leather": "soft warm-brown leather with stitching",
    "iron": "warm-silver plate with a near-white highlight, gold trim",
    "gold": "ornate lemon-gold plate with orange-brown shadows and red cloth",
    "diamond": "cyan diamond plates with white facet tips over dark leather",
}
PIECE = {
    "helmet": "a {m} helmet, front view: a rounded dome narrowing to the chin, a horizontal eye slit, centred",
    "chestplate": "a {m} chestplate, front view: a breastplate with rounded shoulder plates, a collar at the neck, "
                  "a brown leather belt at the waist, centred",
    "leggings": "{m} leggings, front view: a brown leather belt with a small buckle, two plated legs with knee plates, "
                "a dark gap between the legs, centred",
    "boots": "a pair of {m} boots side by side, seen from the front and a little from the side so each boot's foot "
             "and toe show clearly at the bottom, shin pieces above, a dark gap between them, centred",
}
LEATHER_CAP = ("a soft leather cap, front view: a rounded brown leather cap with stitched panels, a short brim and "
               "ear flaps, open face (no visor, no eye slit), centred")
ARMOUR = [(f"{t}_{p}", PIECE[p].format(m=m) if t != "leather" else PIECE[p].format(m=m).replace("plated", "leather")
           .replace("plates", "panels").replace("plate", "panel"))
          for t, m in ARM.items() for p in PIECE]
ARMOUR = [(n, LEATHER_CAP if n == "leather_helmet" else w) for n, w in ARMOUR]

# redos Ronald asked for (11-04): the hoes with bigger heads; the leather helmet as a soft cap and
# the boots with visible feet
REDO = [f"{t}_hoe" for t in TIER] + ["leather_helmet"] + [f"{t}_boots" for t in ARM]

# ---------------------------------------------------------------- materials, ores, gems (04, 05, 06, 09)

MATERIALS = [
    ("coal", "a lump of coal: a chunky faceted black-brown lump, never pure black, soft grey-blue glints on its "
             "upper-left facets" + C),
    ("diamond", "a diamond: a cluster of faceted cyan shards with teal shadow and white facet tips, one white "
                "plus-shaped sparkle" + C),
    ("amber", "a chunk of amber: a rounded glossy honey-orange resin lump that looks translucent, a tiny dark insect "
              "inside, a pale yellow highlight on the upper left" + C),
    ("redstone", "a pile of glowing red powder: a lumpy pile of saturated red nuggets with pink highlights" + C),
    ("iron_ingot", "an iron ingot: a warm-silver bar in three-quarter top view, tilted along the diagonal rising to the "
                   "top right, a near-white lit top face, a darker side face 2 px thick along the bottom right, a faint "
                   "embossed rune" + C),
    ("gold_ingot", "a gold ingot: a lemon-gold bar in three-quarter top view, tilted along the diagonal rising to the "
                   "top right, a bright lit top face, an orange-brown side face 2 px thick along the bottom right, a "
                   "faint embossed rune" + C),
    ("raw_iron", "a lump of raw iron ore: a rough chunky rock of warm grey stone with tan-copper iron nuggets showing "
                 "on it" + C),
    ("raw_gold", "a lump of raw gold ore: a rough chunky nugget of lemon-gold with orange-brown shadows and bits of "
                 "grey stone stuck to it" + C),
    ("stick", "a wooden stick: a slightly crooked warm-brown twig 3 px wide with grain and a small knot" + DIAG),
    ("string", "a loose coil of pale cream string forming loops with a loose tail" + C),
    ("feather", "a feather: a soft cream-white feather with a pale quill shaft, lying diagonally, shaft end at the "
                "bottom left"),
    ("leather", "a piece of leather hide: a soft irregular warm-brown hide shape with mottled texture" + C),
    ("bone", "a bone: warm cream bone with knobbed ends" + DIAG),
    ("gunpowder", "a small heap of gunpowder: dark grey-black grains with a few lighter grey specks, a soft pile" + C),
    ("sugar", "a small heap of white sugar: pale cream-white crystals with tiny white sparkles and a soft blue-grey "
              "shadow" + C),
    ("paper", "a slightly crumpled square sheet of parchment lying at an angle, soft cream with gentle fold shading "
              "and a warm brown edge"),
    ("book", "a closed leather book lying at an angle, deep red cover with a darker border, a cream page block 4 px "
             "thick showing on the lower-left edge, dark brown spine, small metal corner caps"),
]

# ---------------------------------------------------------------- food (07 Food)

FOOD = [
    ("apple", "a red apple: round glossy vivid red with a pale streak on the upper left, a short stem and one green "
              "leaf" + C),
    ("apple_pie", "an apple pie in three-quarter top view: a round golden lattice crust over red-brown apple filling, "
                  "a crimped rim" + C),
    ("bread", "a loaf of bread in three-quarter view: a golden-brown crust with slashes on top and a cut end showing "
              "pale speckled crumb" + C),
    ("carrot", "a carrot: a bright orange tapering root with faint rings, lying diagonally, a green leaf tuft at the "
               "top right"),
    ("potato", "a raw potato: a lumpy oval tuber, pale tan-brown skin with small dark eyes" + C),
    ("baked_potato", "a baked potato: a golden-brown roasted potato split open on top showing fluffy pale yellow "
                     "inside" + C),
    ("beetroot", "a beetroot: a round deep crimson-purple root with a thin tail at the bottom and short green-red "
                 "stalks on top" + C),
    ("corn", "an ear of maize: golden-yellow kernels in rows, pale green husk leaves peeled back at the bottom, lying "
             "diagonally"),
    ("roasted_corn", "a roasted ear of maize: golden kernels with dark brown char spots, husk leaves peeled back, "
                     "lying diagonally"),
    ("cassava", "a cassava root: a long tapering tuber with rough dark brown bark-like skin, one cut end showing "
                "white flesh, lying diagonally"),
    ("melon_slice", "a slice of melon: a crescent wedge with a green rind, a pale band and juicy pink-red flesh with "
                    "a few dark seeds" + C),
    ("beef", "a raw steak: a glossy red cut of meat with a cream fat edge and marbling" + C),
    ("cooked_beef", "a cooked steak: a muted brown seared steak with darker grill stripes" + C),
    ("porkchop", "a raw pork chop: pale pink meat with a cream fat rim and a small bone" + C),
    ("cooked_porkchop", "a cooked pork chop: golden-brown seared meat with a crisp fat rim and grill stripes" + C),
    ("mutton", "raw mutton: a red-pink leg cut with cream fat and a short bone end" + C),
    ("cooked_mutton", "cooked mutton: a browned roast leg cut with crisp fat and a short bone end" + C),
    ("chicken", "a raw chicken leg: pale pink skin with goosebump texture and a pale bone end" + C),
    ("cooked_chicken", "a roast chicken leg: golden-brown crisp skin and a pale bone end" + C),
    ("hare", "a raw hare: a skinned dark-red game carcass, long hind legs, lying diagonally"),
    ("cooked_hare", "a roast hare: a golden-brown roasted game carcass with long hind legs, lying diagonally"),
    ("venison", "raw venison: a deep dark-red lean steak with very little fat" + C),
    ("cooked_venison", "cooked venison: a dark brown seared lean steak with grill marks" + C),
    ("trout", "a raw trout: a whole fish lying diagonally, head at the bottom left, olive-green speckled back, a pink "
              "stripe along the side, silver belly"),
    ("cooked_trout", "a cooked trout: a whole fish lying diagonally, head at the bottom left, golden-brown roasted skin "
                     "with darker char marks, a fin on top"),
    ("pacu", "a raw pacu: a deep-bodied round Amazon fish lying diagonally, silver-grey with an orange-red chest and "
             "belly"),
    ("cooked_pacu", "a cooked pacu: a deep-bodied round fish lying diagonally, golden-brown roasted skin with char "
                    "marks"),
    ("egg", "an egg: an upright oval, slightly narrower at the top, warm cream shell with a soft highlight on the "
            "upper left, an outline no darker than mid brown" + C),
    ("cooked_egg", "a fried egg seen from above: a white with a frilled golden-brown edge and a round yellow yolk "
                   "with a glint" + C),
    ("cooked_rice", "a wooden bowl of cooked rice seen from the front and slightly above: a heap of white grains with "
                    "a soft shine" + C),
    ("rotten_flesh", "rotten flesh: a lump of grey-green spoiled meat with sickly olive patches and a dark red edge"
                     + C),
    ("spider_eye", "a spider eye: a glossy dark red eyeball with a bright highlight and a few thin veins" + C),
    ("kibble", "a small pile of kibble: chunky round brown pet-food pellets, a few lighter crumbs" + C),
]

# ---------------------------------------------------------------- plants, seeds, nuts, berries (07, 11)

def berries(name, look):
    return (name, f"a small cluster of {look}, two or three green leaves" + C)


PLANTS = [
    berries("berries", "round glossy red berries"),
    berries("blackberries", "glossy black-purple bramble berries made of tiny round drupelets"),
    berries("raspberries", "bright pink-red raspberries made of tiny round drupelets"),
    berries("cloudberries", "amber-orange cloudberries made of large soft drupelets"),
    berries("crowberries", "tiny shiny black crowberries on a sprig with needle-like leaves"),
    berries("lingonberries", "small glossy bright red lingonberries on a sprig with small oval dark-green leaves"),
    ("rose_hip", "rose hips: two glossy orange-red oval hips with dark sepals at the tip on a short green stem" + C),
    ("acorn", "an acorn: a smooth tan-brown nut in a scaly cup cap with a short stem" + C),
    ("hazelnut", "two hazelnuts: round glossy brown nuts with pale bases, one with a leafy green husk" + C),
    ("pine_nut", "a few pine nuts: small ivory teardrop-shaped kernels beside one dark brown shell" + C),
    ("acuri_nut", "acuri palm nuts: two oval hard brown nuts with fibrous husk remains and a pointed tip" + C),
    ("moriche_fruit", "a moriche palm fruit: an egg-shaped fruit covered in glossy red-brown scales like a pine cone"
                      + C),
    ("prickly_pear_fruit", "a prickly pear fruit: an oval magenta-red cactus fruit with tiny tufts of spines" + C),
    ("mesquite_pod", "mesquite pods: two long flat curved pale tan pods with bumps along the beans inside, lying "
                     "diagonally"),
    ("araucaria_cone", "an araucaria cone: a large round green-brown cone covered in pointed overlapping scales" + C),
    ("cycad_seed", "a cycad seed: a large oval glossy orange-red seed" + C),
    ("ginkgo_seed", "ginkgo seeds: two round wrinkled pale yellow-orange fleshy seeds" + C),
    ("mushroom_brown", "a brown mushroom: a flat broad tan-brown cap on a short pale stem" + C),
    ("mushroom_red", "a red mushroom: a round red cap with cream spots on a short pale stem" + C),
    ("kelp", "a strand of kelp: a wavy olive-green seaweed ribbon with small air bladders, lying diagonally"),
    ("wheat", "a bundle of ripe wheat: golden stalks with plump bearded ears, tied, lying diagonally"),
    ("barley", "a bundle of ripe barley: pale gold stalks with long-bearded nodding ears, lying diagonally"),
    ("rye", "a bundle of ripe rye: grey-gold stalks with long slender bearded ears, lying diagonally"),
    ("rice", "a bundle of ripe rice: golden drooping panicles of grain on thin stalks, lying diagonally"),
    ("millet", "a bundle of ripe millet: bushy round golden-tan seed heads on stalks, lying diagonally"),
    ("sorghum", "a head of sorghum: a dense cluster of round red-brown grains on a stalk, lying diagonally"),
    ("beans", "a handful of dry beans: speckled pink-brown and cream kidney beans" + C),
    ("sunflower_seeds", "a few sunflower seeds: teardrop seeds striped black and grey-white" + C),
]

SEED = "a small heap of seeds: {} on a tiny patch of dark soil" + C
SEEDS = [
    ("wheat_seeds", SEED.format("small oval pale tan wheat grains")),
    ("barley_seeds", SEED.format("plump pale gold barley grains with tiny husk points")),
    ("rye_seeds", SEED.format("slender grey-green rye grains")),
    ("rice_seeds", SEED.format("golden-husked rice grains")),
    ("millet_seeds", SEED.format("tiny round cream-yellow millet grains")),
    ("sorghum_seeds", SEED.format("round red-brown sorghum grains")),
    ("maize_seeds", SEED.format("flat golden-yellow maize kernels")),
    ("bean_seeds", SEED.format("speckled pink-brown bean seeds")),
    ("melon_seeds", SEED.format("flat oval dark brown melon seeds")),
    ("pumpkin_seeds", SEED.format("flat oval cream pumpkin seeds")),
]

# ---------------------------------------------------------------- containers, tricky objects, creatures, misc (08, 10, 11, 12)

BUCKET = ("a metal bucket seen from the front and slightly above, slightly tapered, thick bright warm-silver rim, "
          "vertical highlight just left of centre, a dark rope band around the middle, small round handle lugs on "
          "both sides")
MISC = [
    ("bucket", BUCKET + ", the oval opening showing the dark inside" + C),
    ("water_bucket", BUCKET + ", filled to just below the rim with water: a flat oval blue surface with one pale glint"
     + C),
    ("lava_bucket", BUCKET + ", filled to just below the rim with lava: a flat oval red surface with bright orange "
                             "swirls" + C),
    ("sedative", "a round glass potion flask, front view, white glass outline with a pale inner edge, transparent "
                 "glass with one curved white highlight on the upper left, small tan cork, a white cloth tied round "
                 "the neck with thin red cords, filled two-thirds with a sleepy lavender-purple liquid"),
    ("bandage", "a rolled bandage: a cream linen roll with a loose end trailing, faint weave texture" + C),
    ("compost", "a heap of compost: dark crumbly brown soil with bits of green leaf and straw" + C),
    ("dung", "a dung pat: a round flattened dark brown dropping with a lighter rim, a few straw bits" + C),
    ("earthworm", "an earthworm: a pink-brown segmented worm curled in an S shape, faint ring segments and a slightly "
                  "paler band" + C),
    ("grub", "a beetle grub: a plump curled C-shaped cream-white larva with a small orange-brown head" + C),
    ("insects", "a few small insects: two beetles and a cricket in browns and greens, seen from above" + C),
    ("beetle", "a beetle seen from above: a shiny dark green-bronze shell with a highlight, six thin legs and short "
               "antennae" + C),
    ("torch", "a torch, upright and centred: a short wooden handle 3-4 px wide with a small iron collar, a teardrop "
              "flame on top, dark orange at its edge, orange in the body and yellow-white in the core"),
    ("redstone_torch", "a redstone torch, upright and centred: a short wooden handle with a small iron collar, a "
                       "glowing round red crystal tip with a bright pink highlight"),
    ("lever", "a lever: a small grey cobblestone base plate in three-quarter top view with a wooden handle sticking up "
              "and tilted to the right" + C),
    ("repeater", "a repeater seen from slightly above: a thin smooth grey stone slab with two small red torches on it "
                 "and a faint red arrow line" + C),
    ("comparator", "a comparator seen from slightly above: a thin smooth grey stone slab with three small red torches "
                   "in a triangle" + C),
    ("spyglass", "a spyglass lying diagonally from the bottom left (eyepiece) to the top right (lens): a "
                 "copper-and-leather tube in three segments that get wider toward the lens, a pale blue glint on the "
                 "lens"),
    ("nest", "a bird nest: a round bowl of woven twigs and dry grass seen from the front and slightly above, empty "
             "inside" + C),
    ("raptor_egg", "a large raptor egg: an upright oval, slightly narrower at the top, pale olive-cream shell with "
                   "dark brown speckles, a soft highlight on the upper left" + C),
    ("utahraptor_saddle", "a saddle seen from the side and slightly above, reading clearly as a saddle: a dark brown "
                          "leather seat with a raised back, one stirrup hanging below with a metal ring, a few metal "
                          "studs" + C),
    ("wyvern_saddle", "a riding saddle for a flying beast, seen from the side and slightly above: a dark red-brown "
                      "leather seat with a high back, a gold buckle strap and a stirrup hanging below" + C),
]
EGG = "a spawn egg: an upright oval egg, {}, a soft highlight on the upper left" + C
SPAWN = [
    ("chicken_spawn_egg", EGG.format("cream shell with red speckles")),
    ("cow_spawn_egg", EGG.format("dark brown shell with big cream patches")),
    ("pig_spawn_egg", EGG.format("pale pink shell with darker rose spots")),
    ("sheep_spawn_egg", EGG.format("pale cream-white shell with soft pink spots")),
    ("zombie_spawn_egg", EGG.format("teal-green shell with dark green blotches")),
    ("skeleton_spawn_egg", EGG.format("pale bone-grey shell with darker grey spots")),
    ("spider_spawn_egg", EGG.format("dark brown shell with red speckles")),
    ("creeper_spawn_egg", EGG.format("bright green shell with dark green-black blotches")),
]

# ---------------------------------------------------------------- Survival's new items (claude/survival-clothing-cooking-f0c469)

CLOTH = {
    "fur": "thick shaggy grey-brown fur with a cream trim",
    "wool": "soft knitted cream-white wool with a visible stitch pattern",
    "hide": "soft tan buckskin leather with fringed seams",
    "linen": "light undyed beige linen with a coarse weave",
}
WEAR = {
    "hat": "a {m} hat, front view, centred",
    "coat": "a {m} coat, front view, sleeves out a little, centred",
    "trousers": "a pair of {m} trousers, front view, centred",
    "boots": "a pair of {m} boots side by side, front view, centred",
}
WORDS = {("wool", "coat"): "sweater", ("wool", "boots"): "socks", ("hide", "coat"): "shirt",
         ("hide", "boots"): "moccasins", ("linen", "hat"): "hood", ("linen", "coat"): "shirt",
         ("linen", "boots"): "foot wraps"}
SURVIVAL = []
for mat, m in CLOTH.items():
    for piece, pat in WEAR.items():
        word = WORDS.get((mat, piece))
        t = pat.format(m=m)
        if word:
            t = t.replace(piece, word)
        SURVIVAL.append((f"{mat}_{piece}", t))
SURVIVAL += [
    ("fur", "a pelt: a rolled-up shaggy grey-brown fur pelt with a lighter cream underside showing" + C),
    ("plant_fibre", "a bundle of plant fibre: pale green-tan long fibres twisted together and tied in the middle, "
                    "lying diagonally"),
    ("cloth", "a folded piece of linen cloth: two neat folds of light undyed beige fabric with a coarse weave" + C),
    ("salt", "a small heap of coarse salt: white-grey crystals with tiny sparkles, a soft blue-grey shadow" + C),
    ("bowl", "a wide wooden bowl seen from the front and slightly above, oval opening showing the empty inside, bright "
             "rim" + C),
    ("stew", "a wide wooden bowl seen from the front and slightly above, bright rim, filled with a thick orange-brown "
             "meat stew with chunks of meat and vegetables, a wooden spoon handle sticking out to the upper left" + C),
    ("dried_meat", "strips of dried meat: three dark red-brown leathery jerky strips, wrinkled, lying across each "
                   "other" + C),
    ("smoked_meat", "smoked meat: a dark mahogany-brown smoked cut with a glossy crust and a pale fat edge, tied with "
                    "string" + C),
    ("salted_meat", "salted meat: a red cut of meat crusted with white salt crystals" + C),
    ("dried_apple", "dried apple rings: three wrinkled pale golden apple rings with red edges, overlapping" + C),
    ("dried_berries", "a small heap of dried berries: wrinkled dark red and purple raisins" + C),
    ("cooking_pot", "a cooking pot: a round black-iron cauldron pot seen from the front and slightly above, a thick "
                    "rim, two small handles, a wisp of steam" + C),
    ("drying_rack", "a drying rack: a small wooden A-frame rack with crossbars, three strips of meat hanging from it"
                    + C),
]

# second redos (Ronald 11-04: "Approve, redo those 3"; and one more try at the hoes that read as hammers)
HOE2 = ("a {t} hoe: a long plain {h_short} handle and, across its top end, one thin flat blade like a garden hoe, "
        "about a third of the canvas wide and only 3 px thick, set at a right angle to the handle and bent "
        "downward, made of {m}; not a hammer, no block-shaped head" + DIAG)
REDO2 = {
    "coal": "a lump of coal: a chunky faceted lump of deep glossy black-brown (the darkest shade of its own brown, never "
            "pure black) with bright blue-grey glints along its upper-left facet edges, strong contrast" + C,
    "gunpowder": "a small heap of gunpowder: a soft cone of near-black charcoal-grey grains with clearly lighter grey "
                 "and a few sulphur-yellow specks, a light top-left side and a dark lower-right side, strong contrast" + C,
    "stick": "a wooden stick: a slightly crooked warm-brown twig a full 3 px thick from end to end, with grain, a small "
             "knot and a lighter top edge" + DIAG,
    "wood_hoe": HOE2.format(t="wooden", h_short="wooden", m="warm orange-brown carved wood"),
    "stone_hoe": HOE2.format(t="stone", h_short="wooden", m="chipped warm grey-tan stone bound with cream cord"),
    "diamond_hoe": HOE2.format(t="diamond", h_short="leather-wrapped", m="faceted cyan diamond with white facet tips"),
    # batch 4 (Ronald: "Approve, redo those 5")
    "venison": "raw venison: a lean deep dark-red steak with a clear darker red-brown outline, a thin cream fat edge "
               "and soft marbling, glossy highlight on the upper left" + C,
    "spider_eye": "a spider eye: a glossy dark red eyeball seen from the front, a big black slit pupil in an amber "
                  "iris, a white glint on the upper left and a few thin dark red veins, clearly an eye" + C,
    "kibble": "a small sack of kibble: a tied cream-tan burlap sack, open at the top showing round brown pet-food "
              "pellets, a few pellets spilled in front" + C,
    "hare": "a raw hare: a whole skinned hare carcass lying diagonally, head at the top right with long ears laid "
            "back, long hind legs at the bottom left, dark red meat with a clear outline, reading as a hare",
    "cooked_beef": "a cooked steak: a warm rich brown seared steak with darker brown grill stripes, a crisp golden "
                   "fat edge and a glossy highlight, never grey" + C,
}

# third try at three hoes (Ronald 11-04: "Yes, retry hoes"): between the hammer and the hook
HOE3 = ("a {t} hoe: a long plain {h} handle; across its top end one flat {m} blade shaped like a garden hoe, "
        "about 10 px wide and 4-5 px tall, flat and plain, set at a right angle to the handle so the blade points "
        "down-left; the blade clearly bigger than the handle is thick, but no block-shaped hammer head" + DIAG)
REDO3 = {
    "wood_hoe": HOE3.format(t="wooden", h="wooden", m="warm orange-brown carved wooden"),
    "stone_hoe": HOE3.format(t="stone", h="wooden", m="chipped warm grey-tan stone, bound on with cream cord,"),
    "diamond_hoe": HOE3.format(t="diamond", h="leather-wrapped", m="faceted cyan diamond"),
}

# new wild foods from the Voxel dog handoff's wild plants (Planner 11-04): new item ids, handed to that session
WILD = [
    ("cattail_root", "a cattail root: a pale cream-tan starchy rhizome lying diagonally, segmented with faint rings, "
                     "a few thin brown root hairs, one cut end showing white starchy flesh"),
    ("cooked_wild_rice", "a wooden bowl of cooked wild rice seen from the front and slightly above: long slender dark "
                         "brown-black grains, some split open showing pale insides, a soft shine" + C),
]

# batch 5 redos and the diamond hoe once more (Ronald 11-04: "Yes, as suggested")
REDO5 = {
    "crowberries": "a small cluster of six glossy jet-black crowberries, each round berry big enough to see on its own "
                   "with a blue-grey bloom and a white glint, on a sprig of short needle-like green leaves" + C,
    "beans": "a handful of five dry kidney beans, clearly bean-shaped and glossy: speckled cream-and-pink-brown ones "
             "and two plain dark red ones, each with a small pale eye, a soft shadow under them" + C,
    "barley": "a bundle of ripe barley lying diagonally: three full golden ears with long dark-tipped bristly awns, "
              "tied with a cord, strong contrast between the golden grains and their darker outline",
    "rye": "a bundle of ripe rye lying diagonally: three long slender grey-gold ears with fine bristly awns, tied with "
           "a cord, the grains clearly drawn with a light top and a darker outline, never washed out",
    "melon_seeds": "a few big flat oval melon seeds, glossy dark brown with a paler rim and a white glint, five seeds "
                   "spread a little apart on a tiny patch of dark soil" + C,
    "diamond_hoe": HOE3.format(t="diamond", h="leather-wrapped", m="faceted cyan diamond, shaped like the stone one,"),
}

# batch 6: redos, and creature icons scaled a little by real size (Ronald 11-04: "maybe size should kinda
# better scale to what animal it is as well a little"): the longest side in px of the 32
def sized(what, px):
    return (what.replace(C, "") + f", drawn small: its longest side only about {px} px of the 32 x 32 grid, centred, "
            "everything round it the flat background color, still clearly readable")


EGG_PX = {"chicken": 17, "pig": 22, "sheep": 22, "cow": 26, "zombie": 23, "skeleton": 23, "spider": 21, "creeper": 22}
SIZE6 = {
    "repeater": "a repeater seen from slightly above: a thick smooth grey stone slab filling most of the canvas width, "
                "two small red torches standing on it and a red arrow line between them" + C,
    "insects": sized("a few insects seen from above: two clear beetles and one cricket, fewer and bigger than a swarm, "
                     "browns and greens with a highlight each", 19),
    "wyvern_saddle": "a big riding saddle for a flying wyvern, seen from the side and slightly above, reading clearly "
                     "as a saddle: a dark red-brown leather seat with a high back and a front horn, a gold buckle strap, "
                     "one stirrup hanging below, its longest side about 26 px of the 32",
    "utahraptor_saddle": sized(dict(MISC)["utahraptor_saddle"], 22),
    "earthworm": sized(dict(MISC)["earthworm"], 16),
    "grub": sized(dict(MISC)["grub"], 16),
    "egg": sized(dict(FOOD)["egg"], 18),
}
SIZE6.update({f"{a}_spawn_egg": sized(dict(SPAWN)[f"{a}_spawn_egg"], px) for a, px in EGG_PX.items()})

BATCHES = [
    ("1_tools", TOOLS, "base"),
    ("2_armour", ARMOUR, "base"),
    ("3_materials", MATERIALS, "base"),
    ("4_food", FOOD, "base"),
    ("5_plants", PLANTS + SEEDS, "base"),
    ("6_misc", MISC + SPAWN, "base"),
    ("7_survival", SURVIVAL, "base"),
    ("5c_wild", WILD, "base"),
]
GREEN = {"sedative", "tranquilizer_dart", "spider_eye", "beetroot", "blackberries", "prickly_pear_fruit",
         "pig_spawn_egg", "sheep_spawn_egg", "earthworm", "dried_berries", "porkchop", "chicken"}
PALE = {"feather", "string", "paper", "bone", "sugar", "egg", "salt", "cloth", "bandage", "wool_hat", "wool_coat",
        "wool_trousers", "wool_boots", "linen_hat", "linen_coat", "linen_trousers", "linen_boots", "cooked_rice",
        "cooked_egg", "pine_nut", "earthworm", "grub"}


def mod_of(name):
    for m in ("base", "animals"):
        if os.path.exists(os.path.join(HERE, "..", "..", "..", "mods", m, "assets", "textures", "item", name + ".png")):
            return m
    return "base"


def main():
    os.makedirs(os.path.join(HERE, "batches"), exist_ok=True)
    seen = set()
    for name, items, _ in BATCHES:
        jobs = []
        for n, what in items:
            assert n not in seen, n
            seen.add(n)
            jobs.append({"kind": "item", "name": n, "what": what, "green": n in GREEN, "pale": n in PALE,
                         "mod": mod_of(n)})
        json.dump(jobs, open(os.path.join(HERE, "batches", name + ".json"), "w"), indent=1)
        print(f"{name}: {len(jobs)}")
    byname = {j["name"]: j for b, _, _ in BATCHES for j in json.load(open(os.path.join(HERE, "batches", b + ".json")))}
    json.dump([byname[n] for n in REDO], open(os.path.join(HERE, "batches", "2b_redo.json"), "w"), indent=1)
    print(f"2b_redo: {len(REDO)}")
    json.dump([dict(byname[n], what=w) for n, w in REDO2.items()],
              open(os.path.join(HERE, "batches", "3b_redo.json"), "w"), indent=1)
    print(f"3b_redo: {len(REDO2)}")
    json.dump([dict(byname[n], what=w) for n, w in REDO3.items()],
              open(os.path.join(HERE, "batches", "5b_hoes.json"), "w"), indent=1)
    print(f"5b_hoes: {len(REDO3)}")
    json.dump([dict(byname[n], what=w) for n, w in REDO5.items()],
              open(os.path.join(HERE, "batches", "5d_redo.json"), "w"), indent=1)
    print(f"5d_redo: {len(REDO5)}")
    json.dump([dict(byname[n], what=w) for n, w in SIZE6.items()],
              open(os.path.join(HERE, "batches", "6b_size.json"), "w"), indent=1)
    print(f"6b_size: {len(SIZE6)}")
    root = os.path.join(HERE, "..", "..", "..", "mods")
    have = {f[:-4] for m in ("base", "animals") for f in os.listdir(os.path.join(root, m, "assets", "textures", "item"))}
    print("in the game but not queued:", sorted(have - seen) or "none")
    print("queued, not in the game (new):", sorted(seen - have))


if __name__ == "__main__":
    main()
