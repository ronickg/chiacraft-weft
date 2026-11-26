# Group "mods": conflict resolution report

Every file on my list has no conflict markers left. Nothing was built, run or tested, and no git writes were made.
There are 14 `// SYNC:` comments (listed per file below).

## Files

- **Cargo.lock**: ours (`git show :2:Cargo.lock`), whole file. Cargo will update it at the compile step.
- **mods/base/api/src/lib.rs**: upstream's module split (components, events, tags, attributes, chains, hud, dimensions, entities, sky_objects, blocks, menus, items, slots, recipes, tabs) is the base. Our modules (branchy, colonize, grow, land, skills, tree_recipes, trees), our Vitals/Climate block, our food/eat block, ITEM_LEVEL, the clothes slots (HAT..CLOAK) and MC_FOREST are on top. Dropped: our duplicates that upstream now has in its modules (SMELTED, Food, FOOD, FUEL, FLAMMABLE, SMELT_TIME, CRAFTING, SMELTING, HEAD..FEET, OVERWORLD, FLATLANDS, our inline tabs). There are no top-level re-exports, so callers must use the module paths (see "For the compile step").
  - SYNC line 490: `components::FoodItem::food` still eats with upstream's hunger `components::eat`.
  - SYNC line 710: our `Armor` stack component (`ARMOR`, "armor") is kept next to upstream's armor attributes, because parts.rs makes armour from parts.
- **mods/base/assets/lang/en_us.lang, ru_ru.lang**: ours, with our wording, plus every upstream key ours lacked (143 in en, 154 in ru). These cover the woodcutter, effects and potions, storage sort/deposit/trash, the new plants and mushrooms, shears, wood blocks, the shield and the zoom keys.
- **advancements.rs**: ours (EatHot uses `u.entity()`). The screen became `base_api::menus::ADVANCEMENTS`. `ARMOR.on_item(..).is_some()` was kept over upstream's `ARMOR_PIECE`. `screen_open` was changed to `menu_open`.
- **block_entities.rs** (upstream's rename of machines.rs): upstream's chest block and menu (`.opens(&CHEST_MENU)`, `Slots::chest(3).holder()`) with our window screen on top. Our `cooked` and `room_for` stay. Our local `ratio` was dropped because the SDK now has it.
- **blocks.rs**: ours, ported to the new API (`mined_by`, `Cond::chance/state`, `shares_cell_with`, texture builders, `tint_kind`, `r.edit(..)`, damage causes). Upstream's changes on top:
  - `pub use base_api::blocks::*` with only our own statics in the file
  - ladder `on_sides`
  - the cactus `Hazard::touch` with `PRICKED` and its death message
  - liquids in upstream's `.block(|b| ..)` form, keeping our `light_opacity(1)`, plus upstream's water acoustics
  - the `#cools_into_stone` reaction
  - beams and eighths (lines 443-445)

  Upstream's new blocks are all in one function, `upstream_blocks` (line 1312, called at line 1189), with `wood_block`, `berry_leaves` and `pick_berries` under it. Hollow logs are limited to oak, birch and spruce, since only those have `_inner` textures. These were left out of upstream's version: tinted glass and wool (`COLORS`), the grass side overlay, random offsets on our plants, hoe tilling and the `tillable` tag, gravel's flint loot, clay dropping clay balls, CLOVER, FERN and COARSE_DIRT (ours registers base:clover via `wild.rs`, and has its own fern and coarse dirt). The `axis_placement` we use stays.
- **items.rs**: our food loop and parts. Upstream's shield (line 17 + its recipe, line 28) and clay_ball are in. Upstream's mushrooms and berries are in one block (lines 140-172). SYNC line 143: these need rows in `vitals::food::FOODS`. Until they have them, `base_api::eat` refuses them. Upstream's vanilla tools and armor loops were dropped (ours come from parts). Upstream's flint was dropped (first.rs owns it).
- **lib.rs**: our mods plus upstream's new files. Upstream's new content is registered in one commented block (lines 194-201). camera and block_entities are registered at lines 144-145.
- **mobs.rs**: our creeper stays (the blood moon uses it). Upstream's arrow projectile and hitboxes come in, with our sounds. SYNC line 58: the creeper needs `models/creeper.geo.json` and `textures/entity/creeper.png` restored from HEAD.
- **player.rs**: ours (vitals, our HUD, clothes slots, armour cut). Upstream's off-hand slot and storage trash slot come in. Upstream's hunger, tick_players and feed were dropped. SYNC line 148: the grids keep our crafting (`on_change(craft_result).on_take(consume_grid)`), not upstream's `Slots::crafting()`. `set_on(Target::Entity("core:player"))` became `r.edit(&PLAYER).set(..)`.
- **recipes.rs**: ours (bread, paper, book, eggs, cacao, hay, tools from parts). Upstream's tool and armor recipes were dropped. Upstream's recipes for its new pieces were kept (lines 16-19, 32).
- **redstone.rs**: `pub use base_api::blocks::*` and `menus::DISPENSER`, with our COPPER_LAMP on top. Upstream's dispenser block and menu come in with our window screen.
- **sounds.rs**: our sound groups. Upstream's `SHEEP` music (`music/418DreamingSheep`, line 55) comes in, because our calm1 track was deleted upstream.
- **testmod/src/lib.rs**: ported (`mined_by`, `shares_cell_with`). SYNC line 447: testmod:apple needs a vitals food row.
- **testkit/script.rs**: both sides' docs. Our `chatf` plus upstream's `op` command. `Menu::generated`, `menu_open`.
- **testkit/server.rs**: ours (`r.edit(..).set_text`, `before_event`, `world_tick`). Upstream's PIG, light, lit and pedestal setup, count, prop, parts and hit_from come in.
- **tests**:
  - gameplay.rs: ours, plus upstream's critical-hit and torch tests and the `--port` server args. SYNC line 183: check the core:equipment count now that the off-hand is a slot.
  - looks.rs: ours, plus upstream's `environment_sources`.
  - mod.rs: our audit list, plus upstream's tests.
  - redstone.rs: ours.
  - scene.rs: ours, plus upstream's explanation check.
  - ui.rs: the crafting test was adapted to upstream's flow using our helpers; our chat test is kept.
  - world.rs: ours, plus `big_blocks` and `generation_reads`.
- **Worldgen**: upstream deleted `mods/base/src/worldgen.rs` and split its own into `mods/base/src/worldgen/`. **Ours is now in that layout, and ours wins on content**:
  - mod.rs: the doc, the imports, `register` and `flat`
  - consts.rs: SEA, the water tints, gap, stand and clearing measures, clay and wild water depths
  - statics.rs: the biome components, tags, our biomes, and the feature handles
  - tree_algorythms.rs: the plans for trees, giants, young trees, shrubs, boulders, gaps, lianas and fallen giants
  - decor.rs: `decorate`, bed, clay, ponds, shore
  - util.rs: the stand and cherry masks, `flower_kind`, `wild_roll`, `wild_at`

  These changes were ported:
  - `Scatter` became `Feature`, `.cell` became `.spacing`, and `.shape` became `.plan`
  - `Site` became `Plan`, and `put_force` became `set` (put now returns bool, so the closures end in `;`)
  - ores: `.per_cube`, `.replaces`
  - `set_on/set_text_on(Target::Biome)` became `r.edit(Biome::generated(..)).set/set_text`

  SYNC in mod.rs lines 371, 387, 398, 414, 424: **thinning (`thin`, `thin_in`) has no home on the feature builder.** The calls are commented out, so for now the trees, stands, cherry groves, shrubs and young trees are not thinned: no clearings, cherries everywhere in their biomes, shrubs not in thickets. `stand_over` and `cherry_grove` are kept with `allow(dead_code)` (SYNC, util.rs). This needs `FeatureBuilder::thin/thin_in` in sdk/feature.rs, which the terrain group's SYNC at sdk/terrain.rs line 689 also points to.

  **Still to do:**
  - `mods/base/src/worldgen.rs` (UD) must be removed.
  - `mods/base/src/worldgen/tree_kinds.rs` is upstream's and is not declared in mod.rs, so it is not compiled. Its `TREE_KINDS` clashes with ours by name and type. Remove it, or keep it for later.
  - There is no `mods/base/src/tree_kinds.rs`, so there was nothing to compare.

Extra, from the coordinator's SDK notes:
- mods/animals/src/voices.rs now uses `r.sounds_on_entities(..)`.
- mods/base/src/climbing.rs now uses `attribute::WALL_CLIMB_SPEED`. The doc lines in climbing.rs and vitals/src/climb.rs now say `core:wall_climb_speed`.
- `.holds_liquid()` became `.shares_cell_with("#water")` in mods/base/src/wild.rs, and the docs in testkit/tests/held.rs and testmod were updated.
- No `liquid_in()` uses were found in mods.

## For the compile step (other files, not mine)
- Callers of base_api top-level names must use the modules: `base_api::components::FOOD`, `dimensions::OVERWORLD`, `menus::CRAFTING`, `slots::HEAD`, and so on.
- `.mineable` (about 45 uses), `r.intercept`, `screen_open/close`, `Screen` and `set_on(Target::..)` remain in other mods/base files and in account.rs, audit.rs and packages.rs. For example, levels.rs `SCREEN: Screen` must become `Menu`.
- mods/base/src/land.rs, mods/animals/core/src/nests.rs and mods/animals/src/eggs.rs still use `Site`/`Scatter`, and must move to `Plan`/`Feature`.
- The creeper's model and texture must be restored from HEAD.

## For Ronald: upstream's new content (each with what cutting it removes)
- **Doors**: mods/base/src/doors.rs. Cut `mod doors;` and `doors::register(r);` (lib.rs line 194).
- **Effects and potions** (speed, nausea, poison, glow, ...): mods/base/src/effects.rs. Cut `effects::register(r);` (lib.rs line 195). Upstream's toadstool and snowberry in items.rs use `crate::effects::{NAUSEA, POISON}`, so cut those lines too.
- **Ships and vehicles** (assembler, thrusters, rotors, wheels, helm, floaters, anchors): mods/base/src/assembly.rs and vehicles.rs. Cut `assembly::register`, `vehicles::register` (lib.rs lines 196-197) and the assembly_tool, thruster and helm recipes (recipes.rs lines 16-18).
- **Music player block**: mods/base/src/music_player.rs. Cut `music_player::register` (lib.rs line 198) and its recipe (recipes.rs line 19).
- **Woodcutter, beams and eighths**: mods/base/src/woodcutter.rs. Cut `woodcutter::register` (lib.rs line 199), its recipe (recipes.rs line 32) and `r.beams`/`r.eighths` (blocks.rs lines 443-445).
- **Storage sort, deposit and trash**: mods/base/src/storage.rs. Cut `storage::register` (lib.rs line 200) and the trash slot and group in player.rs (around lines 160 and 206).
- **Target info** (what you look at): mods/base/src/target_info.rs. Cut `target_info::register` (lib.rs line 201).
- **Camera** (zoom keys): mods/base/src/camera.rs. Cut `camera::register` (lib.rs line 144).
- **Shield and off-hand**: items.rs line 17 (shield) and its recipe (line 28), plus the off-hand slot in player.rs (lines 74, 203).
- **Clay ball item**: items.rs line 65. Clay doesn't drop it (ours).
- **Cactus pricks on touch**: blocks.rs `PRICKED` (line 86), its `damage_cause` and `.hazard(..)` (around line 852).
- **Arrows stick in place** (arrow projectile): mobs.rs around line 126.
- **Sheep music** (418 Dreaming Sheep): sounds.rs lines 11 and 54-56.
- **Wood blocks** (bark all round: oak, birch, spruce): blocks.rs `upstream_blocks` (three `wood_block` lines) and the `wood_block` fn.
- **Hollow logs** (oak, birch, spruce): blocks.rs `upstream_blocks` (the `shapes_of("hollow_log")` lines).
- **Bushes and berries** (bush stem, bush leaves, raspberry and snowberry bushes picked by hand): blocks.rs `upstream_blocks`, plus `berry_leaves`/`pick_berries`, plus items.rs lines 140-172 (raspberry, snowberry).
- **New grasses** (short grass, tall tallgrass, bluegrass): blocks.rs `upstream_blocks`, plus their shears and seed loot loop at its end.
- **Flowers** (6 tulips, inula, verbascum, lavender): blocks.rs `upstream_blocks`.
- **Moss** (on any side, in water too): blocks.rs `upstream_blocks`.
- **Forest mushrooms** (porcini, chanterelle, toadstool, dryad's saddle shelf): blocks.rs `upstream_blocks`, plus items.rs lines 140-172 (cooked dryad's saddle, toadstool, dryad's saddle).
- **Water acoustics** (muffled sound under water): blocks.rs, the `.environment(..)` on water.
- **Upstream's worldgen, not taken** (it clashes with ours): lavender fields, hills, tallgrass meadow, clover meadow and snowy peaks biomes; its tree kinds with roots, mossy trunks and stumps; bushes, berry and fungus features. It now exists only in git (MERGE_HEAD) and in the unused `worldgen/tree_kinds.rs`.
- **Upstream's creeper removal, not taken**: we keep the creeper.
- **Tests**: new are environment_sources, big_blocks, generation_reads, a critical hit and the torch. Upstream's riding and flying_rider tests were dropped. Our chat and broken_shader_pack tests were kept.
