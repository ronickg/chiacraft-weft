"""Today's in-game icons for the art regen test (11-05), composed as the engine does
(`tools/catalogue/images.py`, `crafting::looks`, `parts::look_text`): built diamond tools with
the usual parts (oak haft or grip, wrought-iron fitting, guard and pommel), diamond armour (its icon
is its shell's map painted with the diamond ramp), the loose parts' item icons and the foods.

    python3 notes/ui-codex/regen-test/current.py OUT_DIR
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "tools", "catalogue"))
import images as I  # noqa: E402
import sources as S  # noqa: E402

out = sys.argv[1]
os.makedirs(out, exist_ok=True)
tex = I.Textures(ROOT)
kinds, shapes = S.crafting_kinds(ROOT)
looks, maps, _ = S.looks(ROOT)
L = I.Looks(looks, kinds, shapes, maps, {"copper", "bronze", "wrought_iron", "meteoric_iron", "steel", "star_steel", "gold"})

usual = {"haft": "oak", "grip": "oak", "shaft": "oak", "fitting": "wrought_iron", "guard": "wrought_iron", "pommel": "wrought_iron"}
tools = {"axe": "felling_axe_head", "pickaxe": "pick_head", "shovel": "shovel_blade", "hoe": "hoe_blade",
         "sword": "straight_blade", "spear": "leaf_point"}
made = {}
for kind, head in tools.items():
    parts = {}
    for slot, role in kinds[kind]:
        shape = head if role == "Working" else L.first_shape(kind, slot)
        parts[slot] = (shape, "diamond" if role == "Working" else usual[slot])
    look = L.text(kind, parts)
    a = I.compose(look, tex) if look else tex.get(f"base:item/{kind}")  # no look yet: its plain icon
    made[f"built_{kind}"] = (a, look or f"base:item/{kind} (no look yet)")
for shape in ["helmet_shell", "chest_shell", "leg_shell", "boot_shell"]:
    look = L.part_look(shape, "diamond")
    made[f"shell_{shape}"] = (I.compose(look, tex), look)
# The diamond working part as drawn on the tool (its painted head layer alone).
for kind, head in tools.items():
    a = tex.get(f"base:parts/{kind}/{head}/diamond")
    if a is not None:
        made[f"layer_{kind}"] = (a, f"base:parts/{kind}/{head}/diamond")
# Loose parts: one icon per shape, the same in every material.
for shape in ["long_blade", "short_blade", "straight_blade", "felling_axe_head", "pick_head", "shovel_blade",
              "hoe_blade", "leaf_point", "haft", "grip"]:
    made[f"part_{shape}"] = (tex.get(f"base:item/part_{shape}"), f"base:item/part_{shape}")
# The approved 11-04 whole items (no longer in any list; their textures stay for worn looks and history).
for n in ["diamond_helmet", "diamond_chestplate", "diamond_leggings", "diamond_boots", "diamond_sword",
          "diamond_pickaxe", "diamond_axe", "diamond_shovel", "diamond_hoe", "diamond", "iron_ingot", "gold_ingot",
          "iron_sword", "iron_chestplate", "gold_helmet", "stick", "leather", "coal"]:
    made[f"approved_{n}"] = (tex.get(f"base:item/{n}"), f"base:item/{n}")
for n in ["bread", "apple", "cooked_beef", "beef", "animals:item/trout", "stew", "egg", "carrot", "berries",
          "baked_potato", "cooked_chicken", "porkchop", "melon_slice", "potato", "animals:item/cooked_trout"]:
    a = tex.get(n if ":" in n else f"base:item/{n}")
    made[f"food_{n.split('/')[-1]}"] = (a, n)
for k, (a, look) in made.items():
    if a is None:
        print("missing", k, look)
        continue
    I.save(a, os.path.join(out, k + ".png"))
    print(k, look)
