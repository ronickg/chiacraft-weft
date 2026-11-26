# Integration review: `dev` after tonight, as one game

Researcher, 2026-11-04, round 9, for Planner and Ronald's morning.

**What was read:**
- `dev` at `4a40548e`, then `ab840c52` (about 45 merges since the evening);
- the unmerged `claude/crafting-d352ab` and `claude/cloaks`.

**Not in `dev` yet:** rooms and the cloak are on `claude/cloaks` (only the cloak's icons are in `dev`), so their stacking questions wait for that merge.

**How it was checked:**
- Read-only: no game, no tests.
- The bugs below are from reading the code. I checked the top three in the code myself, and each held.
- Strings were checked with `reports/round-3-scripts/lang.py` and `keys.py`; code size with `reports/code-health/health.py`.

## Ranked fixes

| # | kind | owner | effort | finding | fix |
|---|---|---|---|---|---|
| 1 | bug | Survival | S | **The campfire cooks one item, then stops.** Each cooked item is stamped `HOT` with the second it was made (`mods/base/src/cooking.rs:268-271`), and the output slot only takes a stack with the same stamps (`:272`). The second steak differs, doesn't fit, and cooking stops. | When the output isn't empty, give the new item the stamps already in the slot. |
| 2 | bug | Survival | S | **Every death gives a free set of clothes.** The tombstone takes the whole inventory, clothing slots included (`levels.rs:129-131`). On respawn `dress()` puts the arrival set on because nothing is worn (`clothing.rs:94-101`). Each death is another set, and the old one waits in the tombstone. | Leave the four clothing slots out of the tombstone, so the player respawns in what they wore. Or mark kit items and take them back when the tombstone is emptied. |
| 3 | bug, clash | Survival (food stacking × cooking) | S | **Opening a pot or rack merges its food stacks.** `age_inventory` runs on any block entity (`survival/food.rs:41-46`, merge at `:103`). A stew takes one item per slot, so beef in two slots becomes one slot of 2 and the stew loses an ingredient or won't cook. On the rack, a merged stack takes the first slot's drying progress. | Merge only in players' inventories and chests, never in machines. |
| 4 | bug | Survival | S | **Dying in water or lava points to the wrong tombstone.** A tombstone is placed only into air within 8 blocks up (`levels.rs:135-140`), and `GRAVE` isn't cleared. So the death screen, hint, marker and grace all use the previous, unreached tombstone, or say "You carried nothing" while the engine drops everything on the spot (`death.rs:83-86, 168`). | Clear `GRAVE` first in `tombstone()`; allow placing into water and replaceable blocks; say "dropped where you died" when none is placed. |
| 5 | before merge | Crafting | S | **The crafting branch's book could hide what the first night needs.** Discovery is "every ingredient held", so the torch stays "?" until coal is mined, and string until the first fibre. The near-miss line ("you've held 1 of its 2 materials") doesn't name what's missing. R and U still list everything, so the book and R disagree. The campfire stays findable (2×2, logs and sticks, and the night hint names it). | Starters always known: planks, sticks, table, campfire, torch, string, cloth. The near-miss names the missing material. R/U either follow the book or are labelled "all recipes". |
| 6 | open | Animals | S | **The tame-time hint never fires,** so the animal screen (sneak-use) and the whistles stay undiscovered. `base_api::hint` has no caller anywhere (`api/lib.rs:566`), and `hint.animals.tamed` (animals `en_us.lang:323`) is unused. It is also about 170 characters, too long for one line. | Call it once per player in `taming.rs:341` beside the milestone; split the text in two. |
| 7 | bug | Survival | S | **The tombstone doesn't know its dimension.** `Grave` is x, y, z only (`api/lib.rs:594`). After a bed respawn in another dimension, `watch` finds no tombstone, clears the marker and cuts the grace to 30 s (`death.rs:112-118`). | Add the dimension, and skip the check while the player is elsewhere. |
| 8 | clash | Survival | S | **A wall torch counts as a fire for rest.** It reaches the 3 °C "near a fire" line within about 3.8 blocks, so a torch-lit room gives resting, Rested, "By a fire" and faster healing (`camp.rs:15, 20, 35`). Monsters stay out only of a campfire's ring, and the hints say "fire". | Rest needs heat ≥ 0.7 (campfire, lit furnace, lava). |
| 9 | bug | Survival | S–M | **"Hot" means first seen, not just cooked.** `aged()` stamps any unstamped cooked food as hot (`survival/food.rs:161-165`). Bread crafted at a table, and furnace food left for days, come out hot: +4 °C for 4 minutes, and the Hot Meal advancement for free. | Stamp hot only where cooking happens: the furnace's done event, the campfire, the pot. |
| 10 | bug | Survival | S | **Death keeps food poisoning and Rested** (`survival/mod.rs:76-79` clears poison, bleeding and torpor only). Up to 3 minutes without healing after a respawn. | Clear `sick` and `rested` too. |
| 11 | clash | Survival | S | **The night hint ignores the seasons.** Its hours are fixed at 16–18, and it says "Night falls at 18:00" (`hints.rs:57, 169`; `en_us.lang:501`). The HUD clock and beds follow the calendar's daylight. In winter the hint comes after dark. | Use `calendar::daylight_hours` for both the trigger and the text. |
| 12 | clash | Animals, Survival | S | **How far a fire warms differs:** players 7 blocks, animals, eggs and crops 3, the rack 0 (`temperature.rs:20`, `comfort.rs:49`, `eggs.rs:381`, `grow.rs:61`). A campfire 4–7 blocks off warms the player by 8–17 °C and a tame or an egg not at all. Animals take the whole wind chill; eggs none. | One `FIRE_REACH` in `vitals`; a windproof share for fur and feathers. |
| 13 | clash | UI redo, Survival | S | **"Feels X °C" is the air after windproofing, not the body.** In fur at −10 °C it says "feels −10 °C" while the body is comfortable (`hud.rs:175`). Clothing tooltips show the whole material's wind stop (a fur hat "80%"), where a hat adds 12%. Leather armour warms but has no warmth tooltip. | Label the HUD line "air"; tooltips show the piece's share; give armour its line. |
| 14 | strings | UI redo, Phase 3 | S–M | **Russian:** base lacks the 9 new UI-redo keys (`hud.base.effects_1`…`_6`, `levels.base.badge`, `points`, `xp`) and has one stale key (`hud.base.level`). The engine's `ru` still lacks about 100 text keys (account, friends, links, join sheet, multiplayer) and `fr` 86, both unchanged since round 3. The packages mod (17 keys), animals and portals have no `ru` (Ronald's call). | Base first; the engine's screens next. |
| 15 | tidy | Survival, Voxel dog | S | **Small mismatches:** "Sheltered" is sky light < 13, "open sky" is ≥ 15, and at 13–14 you are neither (`lib.rs:228`, `camp.rs:36`). Biome flowers give no comfort: only dandelion and rose count (`camp.rs:26`). The fibre tooltip names only tall grass (`en_us.lang:533`, `ru:524`). "Grn" reads as grain, not greens (`lang:262`). The wyvern comment still says R (`breath.rs:2`). | One sky-light constant; a `flowers` tag; tooltip text. |
| 16 | clash | Worldgen, Survival | S | **Snow caps against the air.** The air cools by height at 6 m a block, so a temperate snow line at y 162 still has about 8 °C mean air: rain falls on snow (`api/land.rs:244` vs `api/lib.rs:162`). | Derive the snow lines from each climate with the same altitude rule. |
| 17 | open | engine/UI, Phase 3 | S | **What's left from the first-hour list.** Create World still has no tooltip on Survival difficulty (`saves.rs:888-918`; the settings screen has it, `screens.rs:863`). R on a held item nothing makes shows "Nothing here." instead of its uses (`app.rs:2406`). Address still sits on Select World (`saves.rs:822`). | Reuse the settings tooltip; fall back to uses; move Address. |

**Top five for the morning:** 1, 2, 3, 4 (all Survival, all S, together an afternoon), and 5 (Crafting, before its merge).

## The first hour now

Since round 6 nearly everything landed:
- **Death:** the death screen has the cause and the tombstone's place, a marker with distance, and a monsters-off walk back (180 s, cut to 30 s at the grave).
- **Respawn:** full stamina, the meal and plain clothes.
- **Keys and HUD:** R and U work with nothing hovered; the HUD names its shortcuts; the wyvern's breath moved to V; "feels" and the effect names are on the HUD.
- **Food in the right order:** cattail root and wild rice are food, and sedge and cattail give fibre, before ripe-only drops. The spring plains keep plant food.
- **Animals:** tooltips on the spyglass, club, dart, sedative, saddle and amber; the `tame` milestone fires.
- **Create World and the title:** "Survival difficulty", Gentle's three monster-free nights, the chiacraft footer.

What a new player still meets, in order:
1. Address beside Delete on Select World.
2. No tooltip on Create World's difficulty.
3. R on logs says "Nothing here."
4. Food stacks that move by themselves: they merge into earlier slots, hot food stays apart until it cools, and different ages never merge. Players will ask where their berries went. A tooltip line, "merges with the same food of the same age", would explain it.
5. The campfire cooking once and stopping (1).
6. A torch-lit room counting as a fire (8).
7. A free set of clothes at each death (2).
8. Never learning the animal screen (6).

**When rooms and the cloak merge:** the room line ("Sheltered · 3 walls, 14 °C") will sit beside the existing "Sheltered" effect, one word with two meanings. Rename one. The wool cloak and fur robe need tooltips, and the rain cape is made of leather, which its name doesn't suggest.

## Numbers outside the shared crates

`base_api::land`'s terrain constants are world shape and fine where they are. These are balance and belong in `vitals` or `animals_core`, worst first:
1. **The heat table** (campfire and lava 1.0, lit furnace 0.7, torch 0.25), which `vitals::preserve` already relies on: `api/lib.rs:339-341`.
2. **Metres per block (6.0)** and the "underground" line at sea level − 6: `api/lib.rs:162, 201`.
3. **A second fallback climate,** `TEMPERATE`: `api/lib.rs:146`.
4. **Fire reach** 7 / 3 / 3 / 3 (item 12).
5. **Wetness:** how long it lasts (120 s) and drying by a fire (÷3): `temperature.rs:18, 68`.
6. **Camp:** `NEAR_FIRE` 3 °C and `REST_SECS` 20: `camp.rs:13-15`.
7. **The campfire's monster ring,** 8 / 4 blocks: `mobs.rs:495-496`.
8. **The night hint's hours,** 16–18 and 5–19: `hints.rs:57, 169`.
9. **The wet hint's 15 °C:** `hints.rs:34`.
10. **Gentle's floor** of a tenth of max health, written twice: `health.rs:48`, `comfort.rs:57`.
11. **Tames' hunger out of comfort** (×1.5, ×2) and fat as food (×0.5): `comfort.rs:87, 136-140`.
12. **An egg in a nest** develops twice as fast: `eggs.rs:384-385`.
13. **Hungry and full at 0.6 / 0.95,** written twice: `forage.rs:44-45`, `hunt.rs:34-35`.
14. **Wounds:** share 0.2, bleeding 20 s, a bandage heals 10: `survival/body.rs:17-20`.
15. **Wild plant growth** 600 / 300 s and `BLOOM_SECS` 300: `wild.rs:50-70, 139`.

Also: "what is a fire" is three lists that must agree by hand (`heat_by_name`, `camp::comfort_of`, the monsters' `campfires` tag); one tag would do. The fibre sources are two tables in `vitals` (`clothing.rs:73`, `wild.rs:218`).

## Code size since round 5

From `reports/code-health/health.py`, against `2026-11-04 dev 30de857.md`:

| measure | round 5 | now |
|---|---|---|
| code lines | 105.8k | 114.8k (+9k in a day) |
| test lines | 16.0k | 17.9k |
| duplicate blocks | 33 | 38 |
| functions written more than once | 21 | 33 |

- **Gone:** the animals' seven copies of `wake` and `settle`, round 5's quick win.
- **New copies:**
  - Phase 6's client SDK copies the guest SDK's `wire_args`, `push` and `__log` (`crates/mod-sdk-client/src/lib.rs:121, 241, 273` and `crates/mod-sdk/src/lib.rs:151, 278, 318`);
  - Phase 4 wrote `short_did` twice (`chiacraft/did.rs:168`, `worlds/world_did.rs:247`);
  - the land-map example tools copy `old_height`.
- **The check:** it now skips `examples/` for `println!`, since those are tools.

## Where each goes

| owner | items |
|---|---|
| Survival | 1, 2, 3, 4, 7, 8, 9, 10, 11, 15 (most), 16 (with Worldgen), the stacking tooltip, the numbers list |
| Crafting | 5, before its merge |
| Animals | 6, 12 (with Survival), numbers 11–13 |
| UI redo | 13 (the HUD line), 14 (base `ru`) |
| Phase 3 | 14 (engine `ru`), Address |
| engine/UI | 17 |
| Voxel dog | 15 (flowers' comfort, fibre tooltip), number 15 |
| Phase 6, Phase 4 | the new copies in code size |
