foods = """apple bread porkchop cooked_porkchop beef cooked_beef rotten_flesh chicken cooked_chicken mutton cooked_mutton spider_eye cooked_egg apple_pie berries dried_meat smoked_meat salted_meat dried_apple dried_berries lingonberries cloudberries crowberries raspberries prickly_pear_fruit rose_hip hazelnut pine_nut sunflower_seeds mushroom_brown mushroom_red""".split()
ingr = "bone string sugar gunpowder feather arrow egg paper book".split()
misc = "stick coal iron_ingot gold_ingot raw_iron raw_gold diamond wheat_seeds wheat bucket water_bucket lava_bucket acorn rice leather fur plant_fibre cloth bowl stew salt dung compost bandage".split()
tools = [f"{m}_{k}" for m in "wood stone iron diamond gold".split() for k in "pickaxe shovel sword axe hoe".split()]
armor = [f"{m}_{p}" for m in "leather iron gold diamond".split() for p in "helmet chestplate leggings boots".split()]
clothes = [f"{m}_{p}" for m in "fur wool hide linen".split() for p in "hat coat trousers boots".split()]
ITEMS = foods + ingr + misc + tools + armor + clothes
BLOCKS = """stone cobblestone mossy_cobblestone dirt grass snowy_grass sand gravel clay sandstone snow ice obsidian coal_ore iron_ore gold_ore diamond_ore glass glowstone bricks iron_block gold_block diamond_block coal_block cooking_pot drying_rack compost_bin bed tombstone sedge cottongrass reindeer_lichen clover water_hyacinth cattail""".split()
