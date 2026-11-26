import re, os, sys
ROOT = os.environ.get("CHIACRAFT_ROOT", ".")

def load(p):
    keys = {}
    if not os.path.exists(p):
        return keys
    lines = open(p, encoding="utf-8").read().split("\n")
    last = None
    for l in lines:
        if not l.strip() or l.lstrip().startswith("#"):
            continue
        if l.startswith((" ", "\t")) and last:
            continue
        if "=" in l:
            k, v = l.split("=", 1)
            last = k.strip()
            keys[last] = v.strip()
    return keys

L = {
    "eng_en": load(f"{ROOT}/crates/engine/assets/lang/en_us.lang"),
    "eng_ru": load(f"{ROOT}/crates/engine/assets/lang/ru_ru.lang"),
    "eng_fr": load(f"{ROOT}/crates/engine/assets/lang/fr_fr.lang"),
    "base_en": load(f"{ROOT}/mods/base/assets/lang/en_us.lang"),
    "base_ru": load(f"{ROOT}/mods/base/assets/lang/ru_ru.lang"),
    "an_en": load(f"{ROOT}/mods/animals/assets/lang/en_us.lang"),
    "po_en": load(f"{ROOT}/mods/portals/assets/lang/en_us.lang"),
}
for k, v in L.items():
    print(k, len(v))

def diff(a, b, label):
    miss = sorted(set(L[a]) - set(L[b]))
    print(f"\n== {label}: {len(miss)} in {a} not in {b}")
    print("  ", miss[:400])
    extra = sorted(set(L[b]) - set(L[a]))
    print(f"   ({len(extra)} in {b} not in {a}: {extra[:20]})")

diff("base_en", "base_ru", "base ru missing")
diff("eng_en", "eng_ru", "engine ru missing")
diff("eng_en", "eng_fr", "engine fr missing")

# registered names (base), expanded by hand from the code
foods = """apple bread porkchop cooked_porkchop beef cooked_beef rotten_flesh chicken cooked_chicken mutton cooked_mutton spider_eye cooked_egg apple_pie berries dried_meat smoked_meat salted_meat dried_apple dried_berries lingonberries cloudberries crowberries raspberries prickly_pear_fruit rose_hip hazelnut pine_nut sunflower_seeds mushroom_brown mushroom_red""".split()
ingr = "bone string sugar gunpowder feather arrow egg paper book".split()
misc = "stick coal iron_ingot gold_ingot raw_iron raw_gold diamond wheat_seeds wheat bucket water_bucket lava_bucket acorn rice leather fur plant_fibre cloth bowl stew salt dung compost bandage".split()
tools = [f"{m}_{k}" for m in "wood stone iron diamond gold".split() for k in "pickaxe shovel sword axe hoe".split()]
armor = [f"{m}_{p}" for m in "leather iron gold diamond".split() for p in "helmet chestplate leggings boots".split()]
clothes = [f"{m}_{p}" for m in "fur wool hide linen".split() for p in "hat coat trousers boots".split()]
blocks = """stone cobblestone mossy_cobblestone dirt grass snowy_grass sand gravel clay sandstone snow ice obsidian coal_ore iron_ore gold_ore diamond_ore oak_log birch_log acacia_log spruce_log oak_planks birch_planks spruce_planks acacia_planks oak_leaves birch_leaves spruce_leaves acacia_leaves water lava tall_grass grazed_grass dandelion rose sapling sugar_cane cactus wheat farmland compost_bin glass crafting_table furnace chest torch ladder glowstone bricks stone_bricks iron_block gold_block diamond_block coal_block wool bookshelf hay_bale podzol dead_bush berry_bush cooking_pot drying_rack red_stained_glass green_stained_glass blue_stained_glass red_lamp green_lamp blue_lamp bed tombstone
sedge cottongrass reindeer_lichen clover lingonberry_bush cloudberry_bush crowberry_bush hazel_bush bramble_bush prickly_pear wild_sunflower wild_rice water_hyacinth cattail""".split()
# redstone statics
rs = open(f"{ROOT}/mods/base/src/redstone.rs").read()
rsn = re.findall(r'(?:Block|Item)::named\("([a-z_]+)"\)', rs)
blocks += rsn
allnames = sorted(set(foods + ingr + misc + tools + armor + clothes + blocks))
def has(lang, n):
    return f"item.base.{n}" in L[lang] or f"block.base.{n}" in L[lang]
print("\n== base registered names missing en:", [n for n in allnames if not has("base_en", n)])
print("== base registered names missing ru:", [n for n in allnames if not has("base_ru", n)])
# lang keys for items that don't exist
named = set(allnames)
stale = [k for k in L["base_en"] if re.match(r"(item|block)\.base\.", k) and k.split(".", 2)[2] not in named]
print("== base en item/block keys with no registration I know of:", stale)

# entities
ents = re.findall(r'Entity::named\("([a-z_]+)"\)', open(f"{ROOT}/mods/base/src/mobs.rs").read())
print("== base entities", ents, "missing en:", [e for e in ents if f"entity.base.{e}" not in L["base_en"]], "missing ru:", [e for e in ents if f"entity.base.{e}" not in L["base_ru"]])
