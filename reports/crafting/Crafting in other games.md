# Crafting in other games

Research for `notes/CRAFTING.md`, 2026-11-04. Numbers marked *unsure* could not be confirmed.

## Valheim
- **Build radius:** a workbench reaches 20 m at level 1, plus 4 m a level (36 m at level 5).
- **Station levels:** each different add-on placed within about 2 m of the bench adds one level, in
  any order. The bench goes from 1 to 5: chopping block, tanning rack, adze, tool shelf.
  The forge reaches level 7 with six add-ons.
- **Roof rule:** a station refuses to work when it's "too exposed". Players find this confusing,
  because floor tiles don't count as a roof, and a mod removes the rule.
- **Finding recipes:** a recipe appears once you've picked up one of each of its ingredients. Some
  also need their station to stand somewhere: the cauldron appears when you pick up tin while a
  forge is standing.
- **Quality and repair:** each bench level lets an item go one quality step higher (most gear goes
  to 4). Repair is free at the station type that made the item.
- **Crafting from the bag only** is the loudest complaint. One Steam poster spends about 40% of
  their play time sorting chests. Mods that craft from nearby chests (CraftFromContainers,
  ChestReach) are among the most installed.

## Vintage Story
- **The grid:** 3 × 3, and tools in a recipe lose durability (a saw for boards). The handbook (H)
  shows everything, with nothing hidden behind discovery.
- **Knapping:** a 10 × 10 flat grid. It won't let you remove voxels the tool needs, so knapping
  can't fail. Flint beats other stones, and obsidian is the best non-metal. Chalk, limestone and
  sandstone can't be knapped.
- **Clay forming:** layered voxels with 1 × 1 to 3 × 3 brushes and a "copy the layer below" mode.
  A bowl takes 1 clay, a tool mould 11–13, a storage vessel 35. Filling big sheets of clay is the
  tedious part.
- **Metals:** tier 2 is copper, gold and silver; tier 3 the bronzes; tier 4 iron; tier 5 steel.
  - Tin bronze is 88–92% copper and melts at 950 °C.
  - The crucible goes to 1340 °C in a firepit and is poured into moulds.
  - Iron needs a bloomery, which gives a bloom full of slag.
- **Smithing:**
  - Metal can be worked above half its melting point. Charcoal burns at 750 °C.
  - You pick the product once, then fill green voxels and clear purple ones with a hammer (heavy
    hit, upset, split).
  - The work cools and keeps its progress when you heat it again.
- **What's tedious:** in 1.22 big iron pieces cool in about 15 s (5–6 hits), while a pick head
  holds heat over 1.5 minutes. That's the reverse of real life, where a bigger mass holds heat
  longer. The developers call it an interim fix. Mods record and replay anvil moves, and the helve
  hammer (wind power) automates plates and ingots.

## TerraFirmaCraft
- **Anvil tiers:** stone 0, copper I, the bronzes II, wrought iron III, steel IV, black steel V,
  red and blue steel VI. An anvil works its own tier and welds one tier above.
- **Forging:** a bar with a progress mark and a target mark.

  | Move | Change |
  |---|---|
  | Punch | +2 |
  | Bend | +7 |
  | Upset | +13 |
  | Shrink | +16 |
  | Light hit | −3 |
  | Medium hit | −6 |
  | Heavy hit | −9 |
  | Draw | −15 |

  Each recipe has 1–3 rules on the last three moves ("bend second last", "hit not last"). Going
  off the bar ruins the piece. Fewer moves earn a bonus (poorly, well, expertly, perfectly
  forged), which means less wear and more speed or damage. Players install helper mods that work
  the moves out for them.
- **Welding:** two pieces and flux, both hot enough to weld, then a strike. *Unsure:* the exact
  heats.
- **Pottery:** clay knapping takes 5 clay, and a mistake can be redone. Clay fires to ceramic in
  a pit kiln.
- **Casting:** ore melts in a small vessel, 10 mB a small ore against 100 mB a cast. Alloys are
  mixed by ratio (bronze 88–92% copper, 8–12% tin).
- **Rock knapping:** two rocks in hand, then remove squares to match a pattern; where the pattern
  sits in the grid doesn't matter.

## Enshrouded, V Rising
- **Enshrouded unlocks:** stations come from craftspeople you rescue (blacksmith, carpenter,
  alchemist), and their quests unlock the recipes.
- **Enshrouded crafting:** workshop crafting is instant. Processing stations take time (a smelter
  about 4 minutes for 10 bronze bars, *unsure*).
- **Enshrouded chests:** only "magic" chests feed stations (radius *unsure*). Update 8 (April 2026)
  made building from them easier.
- **V Rising unlocks:** recipes come from bosses.
- **V Rising queues:** refining (sawmill, furnace, tannery) is a timed queue that runs while you're
  offline, so players load their stations before logging off.
- **V Rising rooms:** a station in an enclosed room with the matching floor works 25% faster.
  A later floor lets crafting use stored resources.

## Minecraft
- **The grid:** 2 × 2 in the inventory and 3 × 3 at the table, with shaped recipes.
- **Recipe book (Java 1.12):** a recipe unlocks on a trigger, usually holding an ingredient.
  Clicking a recipe fills the grid, and shift-click fills as many as you can make. A "craftable
  only" filter narrows the list. Hand-placing any recipe still works unless `doLimitedCrafting` is on.
- **Bedrock** opens the book by default, because a cursor is slow on a controller.
- **Players are split:** some feel the book makes the game easy, others learned every recipe from it.

## Factorio
- **Hand crafting:** left click makes 1, right click 5, shift-click as many as you can. The
  ingredients are taken when you queue the craft.
- **Sub-parts:** parts you can make are crafted automatically (shown orange); missing raw
  materials show red.
- **Cancelling** refunds the materials and cascades to the queued sub-parts. Crafting pauses when
  the bag is full.

## Hytale, Necesse, Core Keeper
- **Hytale:** a three-tier workbench plus specialist benches, which craft from nearby chests (6–7
  blocks, *unsure*). Players ask which chest gets used first.
- **Necesse:** stations have a "use nearby inventories" switch (±5 tiles), and settlers craft from
  your chests.
- **Core Keeper:** pulls from chests at a small range, and players ask for more.

## What it means for us
- **Feels good:**
  - recipes appearing as you find materials (Valheim, Minecraft)
  - one click to repeat or batch a craft, with sub-parts made for you (Factorio)
  - stations that grow by placing things beside them, which reads in the world (Valheim)
  - processing that runs while you're away (V Rising)
  - hands-on work that can't fail (Vintage Story knapping)
  - visible quality for doing it well (TerraFirmaCraft)
- **Tedious:**
  - carrying materials from chests to the bench, the most-modded complaint in Valheim
  - heat running out before the work is done (Vintage Story 1.22)
  - filling big voxel sheets by hand
  - rules that need a calculator (TerraFirmaCraft's helper mods)
  - unclear station rules ("too exposed")
  - memorising grid shapes
- **So:**
  - a recipe list filtered to what you can make
  - recipes found by touching their materials
  - nearby-chest crafting, with the chests in use shown
  - a short queue that makes sub-parts for you
  - real timers for the crafts that wait
  - hands-on making only for what really is hands-on (knapping, clay, the anvil), which can't
    fail and has a "from memory" repeat after the first few times
  - pieces that cool by their real mass
  - repair that costs a little material

## Sources
- Valheim:
  - radius and add-ons: https://www.gamespot.com/articles/valheim-workbench-upgrades-how-to-enhance-your-crafting-station-to-level-5/1100-6488182/, https://thunderstore.io/c/valheim/p/TheShackDev/Shack_Workbench_Pack/
  - the roof rule: https://www.ggrecon.com/guides/valheim-station-too-exposed-how-to-fix/, https://www.nexusmods.com/valheim/mods/381
  - recipes from materials: https://www.supercheats.com/valheim/how-to-learn-new-crafting-recipes
  - forge, repair, quality: https://game8.co/games/Valheim/archives/320293, https://allthings.how/valheim-how-to-level-up-your-forge-to-level-3/
  - chest crafting complaints and mods: https://steamcommunity.com/app/892970/discussions/0/3825284575376745010, https://thunderstore.io/c/valheim/p/Lemonandmassiw/CraftFromContainers/
- Vintage Story:
  - crafting: https://wiki.vintagestory.at/Crafting/en
  - knapping: https://wiki.vintagestory.at/Knapping/en
  - clay forming: https://wiki.vintagestory.at/Clay_forming/en
  - smithing: https://wiki.vintagestory.at/Smithing/en
  - metals: https://wiki.vintagestory.at/Metal/en
  - crucible: https://wiki.vintagestory.at/Crucible/en
  - helve hammer: https://wiki.vintagestory.at/Helve_hammer/en
  - cooling complaint: https://www.vintagestory.at/forums/topic/21042-122-smithing-feels-so-overtuned-to-the-point-i-feel-like-i-need-to-rush-my-work/
  - clay tedium: https://www.vintagestory.at/forums/topic/1974-auto-filling-for-chiselingclay-molds
  - smithing mods: https://mods.vintagestory.at/automaticforging, https://mods.vintagestory.at/smithingplus
- TerraFirmaCraft:
  - anvils: https://terrafirmacraft.github.io/Field-Guide/en_us/metalworking/anvils.html
  - moves and rules: https://wiki.terrafirmagreg.team/modern/field-guide/es_es/tfg_tips/anvil_working_guide.html, https://terrafirmacraft.github.io/Documentation/1.12.x/crafttweaker/anvil
  - pottery: https://terrafirmacraft.github.io/Field-Guide/en_us/getting_started/pottery.html
  - alloys: https://terrafirmacraft.github.io/Field-Guide/en_us/getting_started/primitive_alloys.html
- Enshrouded:
  - crafting: https://enshrouded.wiki.gg/wiki/Crafting
  - magic chests: https://pindrop.gg/enshrouded/guides/magic-chest
  - Update 8: https://www.bisecthosting.com/blog/enshrouded-update-8-patch-notes-forging-the-path-new-skills-attacks-features
- V Rising:
  - refining: https://guildorder.com/games/v_rising/wiki/resource-logistics-and-refinement
  - floors: https://exputer.com/guides/v-rising-workshop-floors/
  - queue: https://vrising.wiki.spellsandguns.com/Inventory_Crafting_Tab
- Minecraft:
  - recipe book: https://minecraft.wiki/w/Recipe_book
  - opinions: https://feedback.minecraft.net/hc/en-us/community/posts/360043346971-Craftable-Recipe-Book
- Factorio: https://wiki.factorio.com/Crafting
- Hytale: https://www.4netplayers.com/en/blog/hytale/hytale-workbenches-functions-overview, https://www.curseforge.com/hytale/mods/increase-crafting-inventory-radius
- Necesse: https://necessewiki.com/Carpenters_Bench
- Core Keeper: https://steamcommunity.com/app/1621690/discussions/0/4428814417441784520
