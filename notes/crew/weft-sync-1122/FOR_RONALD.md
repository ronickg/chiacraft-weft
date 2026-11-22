# Weft sync 11-22: what changed, for Ronald

Branch `claude/upstream-fixes-1122-e85f21`, merge commit 2dd1fdb62 (upstream master cab2d5f72), with dev merged in.

## Where it stands
Tester 2 compared it with dev ae261d346 on the PC, and it all passes:
- **Looks:** og, vista, forest and the no-pack forest match dev. Near land is within 1 luma and 2 % contrast, and the canopy holes and sun patches are back.
- **Other checks:** the bench passes on the final head. Smoke, join, portals, creeper, water plants, menus and broken tool passed on earlier heads and are rerun on the final one.
- **Speed:** frame times are within about 3 % of dev's, with equal draws per frame.

What it took to match dev:
- **Settings through the catalog:** upstream stopped filling `config_names`. Our lookups now go through the catalog, which brings back the pack options, shadow size, RT and the `--config` overrides at join. This was the haze and the flat look.
- **`core:mc_leaves` ported back:** the solid canopy.
- **Generated sea plants stand in their water:** the bench never went idle.
- **The pack's screen effects in their old order:** bloom last.

Left: the pack gets ambient 0.04 where dev had 0 (upstream's floor), so it is slightly lighter in the dark, and clouds sit a little differently at the same hour. The points below are what changes against dev. Some need his call.

## What we lose or what changes
- **Android** is now upstream's: our pre-rotation and Pixel fixes are gone. Planner approved this.
- **The Vulkan backend** uses upstream's Vulkan 1.1 render passes. It needs a PC look and a bench.
  - The Mac still starts: drawIndirectCount is optional, and draws fall back to a compacted buffer without it.
  - The pipeline cache now lives in the backend.
- **The command line** uses upstream's subcommands (`server`, `package`, ...). The scripts that call it need updating.
- **Weather** is upstream's moving weather field, with regional rain and lightning.
  - The base mod's rain by biome is now a forced weather in the mod's biomes.
  - `/weather` given without a time lasts until it is changed.
  - The plain sky gets upstream's volumetric clouds, rain, lightning and atmosphere.
  - Our cloud wind drift is gone.
- **Soft clouds:** `core:soft_clouds` is a setting again (default on, off for the look tests), until the A/B against upstream's clouds.
- **Frame cap:** `core:fps_cap` is gone. Upstream's frame limit replaces it.
- **New worlds and world select** use upstream's screens: game mode, hardcore, cheats and world rules, with a cheats toggle per world.
  - Our rule toggles are gone. The "new address" button stays.
- **Permissions:** upstream's now need cheats on, even for the owner. Our admission gate replaces upstream's whitelist.
- **Screens:** upstream's Profile, Title, Pause and Settings screens are in, with our buttons added. Worlds open the upstream way, so the "loading screen first" step is gone.
- **First person:** the first-person model is on by default.
- **Sound:** all sounds go through one environment track, so menu sounds are muffled underwater too.
- **Generation** of ores, structures and trees uses upstream's feature code, so our scatter-site code is gone.
  - **Scatter thinning has no home yet** (`FeatureBuilder::thin` is missing). Until it does: no forest clearings or stands, cherries all through their biome, no shrub thickets, and far forests may differ.
  - Deep cave fill has changed. Far trees may be off: they need a look.
- **Waterlogging:** ours gave way to upstream's own (`.shares_cell_with("#water")`), and the mods are ported to it.
- **Max health:** back to 100000 on top of upstream's code (Planner: bosses need it).
- **Scatter thinning** comes back as a `thin` hook on upstream's feature builder (Planner).
- **Climbing, gliding and ropes:**
  - Wall climbing has its own attribute, `core:wall_climb_speed`. `core:climb_speed` is now upstream's ladder speed.
  - The climb, glide and rope poses may not show yet.
- **Mob bodies** are turned by the server now.
- **Big animals' pathing** is upstream's. It needs a look with Drenkaun and the titans.
- **Animals' knockback resistance and attack speed** move to upstream's attributes.
- **Broken tools** mine and attack through upstream's code. Check that a broken tool still works like the bare hand.
- **Block textures:**
  - Rotated block textures turn the other way (upstream's fix). Our shader pack still uses the old direction.
  - Grass-side overlays are face layers now. Far-terrain grass sides lose their overlay, and the upper layers don't fade.
- **Lighting and outlines:**
  - The lightmap is upstream's (night vision, sunlight), under our vanilla colour shading.
  - The block outline uses upstream's merged edges, in our thin dark style.
- **Test switches:** mc-moon, mc-days, mc-clouds and `core:mc_leaves` (with `face::INNER`) are back, as the stage 1/2 looks use them. The Minecraft-import spawn is gone.
- **Screen effects** are upstream environment layers now, run by priority then name; base gives its six their old order (reflect, contact, far_ao, volume, shafts, bloom last).
- **Network:** moves are sent every tick, empty ones too.

## Upstream's new content we took: each piece in its own file or block, easy to cut
- doors, potions (toadstool and snowberry use them), ships and vehicles with physics, a music player, a woodcutter (with beams and eighths), storage sort, deposit and trash, target info, camera zoom
- a shield and off-hand slot, clay balls, cactus pricks, arrows that stick, a sheep music track
- the `upstream_blocks` function in blocks.rs: bark-all-round wood, hollow logs, hand-picked berry bushes, new grasses, tulips, inula, verbascum, lavender, moss, four forest mushrooms
- the engine: sound absorption and sound environments, thunder, an editor, logic gates, a screen of mods that failed to load with the reason, `/clear entity`, `/structure`
- the renderer choice (wgpu or Vulkan)

## Upstream content we didn't take, because it clashes with ours: his call
- upstream's worldgen: lavender fields, hills, two meadows, snowy peaks, its trees with roots and stumps, its bush, berry and fungus features
- tinted glass and wool, the grass side overlay, hoe tilling, flint from gravel, clay balls from clay, its clover, fern and coarse dirt
- upstream's removal of the creeper: we keep ours. It now has a geo model in upstream's format, and its walk needs a look.

## Kept ours
- far terrain (LOD), until the A/B
- the mods' art and game
- the protocol is 25: upstream's 24 plus our day rate and shape scale

Helper reports, file by file: the session scratchpad's `sync/*.md`. They will be copied to `notes/crew/weft-sync-1122/` before the branch is ready.

## Upstream content kept for now (Planner, 11-22): open questions
All 71 of upstream's new items and blocks are tagged in `mods/base/progression` (one block, "kept until Ronald's call"); the four new foods have vitals rows.
1. **Nothing grows the new plants:** upstream's plants, berry bushes and forest mushrooms (raspberry, snowberry, toadstool, dryad's saddle included) aren't placed by our worldgen, so for now they're creative only. Should our worldgen place them?
2. **No shears:** several upstream blocks need `#tools/shears` to drop themselves (bush leaves, inula, verbascum, lavender, sheared grasses).
3. **Wheels can't be made:** their recipe takes dyed wool, and we make none. `vehicles.rs` builds with `base:blue_wool`, which isn't registered.
4. **No clay balls:** our clay drops clay, so the clay ball has no source.
5. **Potions:** there's no brewing, so they're creative only.
6. **Grab tool:** it has no recipe. The rotor and assembler do (`redstone.rs`).
7. **Shield:** in the Iron Age, because its recipe needs iron. Should it be an early armour piece, or built from parts?
8. **Snowberry:** upstream made it poisonous; ours gives a chance of sickness instead.
9. Upstream's names for vanilla tools, armour and wool we never register (47 lang keys) were taken out.

## Startup fixes (11-22): what changed in upstream's content
- **Shears:** with no shears and no `tools/shears` tag the game stopped at load, so upstream's "cut with shears" condition never holds for now (`sheared()` in `blocks.rs`): bush leaves, inula, verbascum, lavender and the sheared grasses drop what they drop unsheared. To undo: give a tool of ours the `tools/shears` tag, or make shears, and put `Cond::tool("#tools/shears")` back.
- **Wheels:** their recipe (dyed wool, `#wool/*`) is left out in `vehicles.rs`, the line kept in a comment, so the three wheels are creative only.
- **The car's stripe** (`/spawncar`): upstream's blue wool isn't registered, so it is our white wool.
- **Potion of poison:** upstream's own poison damage (it can't kill) clashed with our survival's (it can, with our death message); ours is kept, so the potion's poison can now kill.

## Connected textures moved to upstream's folder form (11-22)
Upstream keeps a texture's rules in `block/<name>/<name>.ctm` beside its tiles, and stops the game when a `<name>.png` sits next to that folder. Our 44 rules files and the 3D plants pack's 16 are moved into that form, with the same rules as on dev:
- The plain picture is now tile `0.png` in the folder: the first tile, so items and far terrain still show it. Where dev kept it out of the random set (cobblestone, mossy cobblestone, redstone ore) it has weight 0.
- Reeds: our four tiles are back (the merge had taken upstream's). The picture an item shows is now our top tile (3), since the vertical method takes its alone picture from there.
- Obsidian, glowstone and tall grass: upstream's variant folders are dropped. Ours are single pictures, as on dev.

## Our shader pack and upstream's new sky passes (11-22)
With our pack on, upstream's atmosphere pass and lightning bolts don't run, because the pack has no `vs_atmosphere`, `fs_atmosphere` or `fs_lightning`. The engine treats them as optional, so nothing breaks; the bolts and the atmosphere just don't show. Upstream's face layers (grass-side overlays) draw with our `fs_translucent`. Bringing them into the pack is his call, after the look A/B.

## Upstream's new art redrawn in our style (11-22, approved "Yes, approve them")
157 of the 187 pictures the sync added are ours now, same names, sizes doubled to our 32 a block where upstream's were 16 (models read UVs in block units, so they map the same). **At the next sync, keep ours** for all of these ("our art: keep ours"):
- **Potions:** `item/potion_*` (24) and `glass_bottle`. One flask, the liquid in each effect's colour from `effects.rs`. One shape for all, as upstream has; a shape per effect would be new flasks drawn and a different base picked per potion in `notes/art/weft-redraw/potions.py`, nothing else changes.
- **Items (17):** assembly_tool, grab_tool, clay_ball, iron/gold/diamond shears, the plant and mushroom items (chanterelle, porcini, toadstool, dryad's saddle and its cooked one, inula, verbascum, lavender, raspberry, snowberry, reeds). The **shield** model's texture `item/shield.png`, painted in its geo layout from our oak, iron and gold.
- **Effect icons:** `mob_effect/*` (24), now 48 px like our own status icons, each in its potion's colour.
- **Plants:** short grass, tall grass, bluegrass, the four forest mushrooms (in upstream's variant folders, `.ctm` kept), tulips (6), inula, verbascum, lavender (top, bottom and the 32 x 64 whole), bush stem, raspberry and snowberry leaves, moss (our moss block's top), the three hollow logs' insides. Made with `notes/blocks/batches/weft_plants.tsv`, `weft_tulips.py`, `weft_plants_post.py`.
- **Machine faces (36):** assembler, rotor hold, floater hold, guide case and rail, helm, thruster, wheel, music player (and record), woodcutter. Upstream's picture doubled and drawn in our materials by `notes/blocks/weft_machines.py`; rerun it after a material tile changes.
- **UI (10):** icon_put_away, icon_sort, music_play/pause/stop, music_track rows, slot_fuel and slot_trash hints.

Left as upstream's because the game never loads them: `block/stained_glass/`, `clover_top/`, `clover_bottom/`, `fern.png`, `lamp_white`, `rotor_front`, `rotor_back`; `gui/heart_hardcore_*`, `heart_poisoned_*`, `heart_withered_*` (12) and `food_hunger_*` (3), since our HUD draws only the armour and air rows. Redraw them if they come into use.
Every try and sheet: `~/Coding/chiacraft-archive/images/art-weft-content/2026-11-22/`.
