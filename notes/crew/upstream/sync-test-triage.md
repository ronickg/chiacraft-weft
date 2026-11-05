# Sync test triage: SYNC5 (35454f1e) against dev b5815f09 (Tester, read only)

Sources (Tester 2, PC): SYNC5 `--test all` 126 pass / 41 fail; dev b5815f09 `--test all` 188 / 216 with 28 fails
(`~/Coding/chiacraft-archive/images/tester-2/2026-11-04/runs/sync_tests/`: `sync5_all_fails.txt`,
`dev_b5815f09_all_fails.txt`, `sync5_only_vs_dev.txt`, `sync5_restored_fails.txt`, `shots/`, `inv_check/`).
The old "12 fails" dev baseline predates First hours and is not the comparison. Nothing here was run by me; the causes
are from reading the tests and the engine, and each says how sure it is.

Classes: **(a)** a real game regression or clash, to fix or to accept as a follow-up; **(b)** a test that needs
updating (for Weft's new behaviour we chose, for our own changes, or a test-only mod); **(c)** flaky; **dev** = fails
on dev too, so not caused by the sync.

## Summary

| class | rows |
|---|---|
| (a) real, to fix or accept | 8: camera (high), held_items, kill_and_void, respawn_keeps_mode, neutral_mobs, block_shapes, third_person_shakes, redstone_on_a_structure (the last seven to confirm) |
| (b) test to update | 38 (second_hand is (a)-looking but is a layout constant in the test, see below) |
| (c) flaky | 1: spawn_eggs |
| failing on dev too (inside the (b) rows) | 23 of the 47 rows |

Rows: the 41 SYNC5 fails, `creative_middle_click`, and the 5 other restored tests that failed (47 in all).
One cause explains many: **Weft numbers the new off-hand slot LAST in the inventory menu** (`registry.rs` ~2420,
"as in vanilla"), so `T::hotbar()` (last nine slots) clicked one slot too far. Fixed in `runner.rs` (uncommitted
until Upstream takes it; the SYNC8 run checks it).

## (a) Real: fix, or accept as a follow-up

| Test | Fail line | Cause | Sure? | Owner | Suggested fix |
|---|---|---|---|---|---|
| camera | free camera in creative: 0 want 3; a mod's camera on its key: 0 | Weft's `core:freecam` default key is `P`, which is our Stats key (base `hud.rs`, "P Stats"): the press opens Stats, so the camera key never acts, and the mod camera key probably meets an open screen | high (key tables read, shots not) | Upstream (key) + UI | rebind freecam in our defaults (a free key) or Stats; Ronald's call, since P is a shortcut he sees |
| held_items | pickaxe, sword, flower, torch colours in the hand: 0 (block colours fine) | flat items not drawn in the first-person hand; `sync-log.md` already notes parts tools must set stack `layers` and "HELD_LOOK now carries layers"; torch and flower are not parts tools, so it is wider than parts | medium | Upstream (engine, base items) | check on the PC with a shot of a held torch; if the hand is empty it is real and bad |
| kill_and_void, respawn_keeps_mode | timed out waiting for screen `core:death` after `/kill` | the death screen did not open (or its name changed under our `base` `screens::DEATH`); both fail only on sync | medium | Upstream (death.rs was edited in the sync) | check what `/kill` leaves in the log; try `/kill` by hand in the sync |
| neutral_mobs | by day a spider leaves you alone: health 12.15 of 17.95 | a spider hurt a player by day: Weft's AI goals rewrite may have changed the neutral-by-day behaviour of our base mobs | medium | Animals / Upstream (base mobs) | check `mods/base/src/mobs.rs` spider goals |
| block_shapes (Weft's new test) | breaking a double slab gives back 13 want 15; 7 want 8 | a double slab drops one slab instead of two for our `base:sandstone_slab` (our loot / block drop) | medium-low | Upstream (base blocks) | check base slab loot against Weft's "give back two" |
| third_person_shakes | walking sways the view and a hit jolts it: both identical (0.0018) | Weft's camera rewrite (our test was already borderline on a GPU, `ci-skip.txt`); may be Weft's chosen behaviour | low | Upstream | decide: accept as Weft's, then update the test, or the view shake is lost |
| redstone_on_a_structure | wire on it shows red: fails | a pixel probe of wire colour on a moving structure; could be Weft's wire/shader rework or our shader pack | low | Lighting / Upstream | needs a shot |
| second_hand (Weft's new test) | 5 FAIL + hands-swapped timeout | **mostly (b)**: the test's `const OFF: u32 = 40` is Weft's inventory index of the off-hand, but our clothing takes 40-43, so it puts torches in the hat slot; fix OFF from the layout and rerun before calling it real | high that it is the index | Upstream (testkit) | move to (b) when the rerun passes |

## (b) Test needs updating

| Test | Fail line | Cause | Owner | dev? |
|---|---|---|---|---|
| armor | timed out waiting for armor worn | `hotbar()` shift (off-hand slot) | Upstream (testkit) | no |
| creative_inventory | bricks in the hotbar | same | Upstream | no |
| death_is_a_state | on the cursor | same | Upstream | no |
| leaving_keeps_cursor | on the cursor | same | Upstream | no |
| creative_middle_click (restored) | whole stack on the cursor | same | Upstream | no (new here) |
| tooltips | chestplate/diamond block/lines drawn 0 | hovers hotbar slots: likely the same shift | Upstream | no |
| crafting_table_tags (restored) | planks on the cursor | hotbar slot 37 stale since the clothing slots; fixed in 0a00f546 and by the hotbar fix | Upstream | yes |
| inventory_crafting | sync: logs on the cursor; dev: planks in the result | sync: the shift. dev: our hand 2x2 gives bark, not planks (planks only via the book): the test is stale | Upstream, Plants (recipes) | yes |
| inventory_hotkeys | number key 3 got linen_hat | the swap hits the worn outfit slots: slot numbers stale since clothes (same line on dev) | Upstream | yes |
| held_water_place / break / piston / swim / mesh (5) | sea_slab waterlogged=false/true, weed gone, no air loss | our old `held_water` tests for a design we dropped for Weft's waterlogging (`sync-log.md`: "port to upstream waterlogging"); the testmod's `sea_slab` and `waterweed` need Weft's waterlogging flags | Upstream (testmod), Plants or Terrain (water) | no |
| every_food_has_nutrition | `testmod:apple` has no FOODS row | testmod:apple is Weft's new test item; add a row or skip testmod's | Upstream / Plants (vitals FOODS) | no |
| broken_shader_pack | the texture table still reads as packs read it | Weft's texture layers (`Source::Layers`) changed how the table reads | Upstream | no |
| rich_text | {if:testmod:hurt} line light red 0.0001 | probe on tooltip text colour after Weft's text rewrite | Upstream | no |
| spawn_eggs | two pigs: 2:base:pig | counts pigs near; passes on SYNC4 and in the batch: **(c) flaky** | Animals | no |
| third_person, crouch_pose, player_skin (restored) | shirt 0 / -1.1574 / 0.0203 | the player wears our starter outfit (hood, wool coat, orange hide trousers, boots); probes look for the default cyan shirt / blue trousers / red skin; same on dev | Upstream (testkit), UI | yes |
| distant_terrain_from_above (restored) | timed out waiting for distant terrain made | fails on dev too; not a sync regression: the other distant-terrain tests pass on sync (distant_terrain failed on dev) | Distant terrain | yes |

## Failing on dev too (not the sync): 23 rows

account, beds_far, calendar_follows_time, clouds, edit_while_loading, f3, food, inventory_crafting, inventory_hotkeys,
liquid_layers, models, names_translated, own_names, packages, redstone_clock, scene (tool_tiers), skiff_blocks, sky,
plus the restored crafting_table_tags, distant_terrain_from_above, third_person, crouch_pose, player_skin. Their
causes (all test or content drift of ours, none from Weft):

| Test | Cause (read from the fail line) | Owner |
|---|---|---|
| scene / tool_tiers | the first hours: bare tools and parts tools give no drops, the table of what a wooden pickaxe drops is stale | First hours / Upstream |
| account | the account screen's "marked as written down" text changed with the UI redo | UI |
| names_translated, own_names | five stripped tropical logs had no names (fixed b3c14ac4); own_names: the broken chest's item name | Upstream |
| packages | gems:ruby package: the package mod moved on | Packages |
| f3 | 1600x900 text on a window that is 1280x720 | UI |
| calendar_follows_time, beds_far, food | timeouts on survival changes (hunger, respawn, `mode creative` answer) | Plants / Upstream |
| clouds, sky, liquid_layers | pixel probes after the HDR/shader work | Lighting |
| models | `animate me wave` and the idol bow: the first-person hand test | Animals |
| redstone_clock | 14 flips in four seconds: clock speed | Upstream |
| skiff_blocks | a skiff carrying a stone | Upstream |
| edit_while_loading | a placed block shows after 5 s with ~190k meshes waiting: a real loading cost, on dev too | Lighting / Distant terrain |

Fixed by the sync (fail on dev, pass now): dimensions, distant_terrain, foreign_terrain, multiplayer (a segfault on dev),
item_model_file, break_refused, lod_columns, riding.

## What I'd do before landing

1. Rerun on SYNC8: the 8 cursor tests (hotbar fix), `second_hand` with a layout-derived OFF, `crafting_table_tags`.
2. Decide the freecam key (Ronald): P clashes with Stats.
3. Look at `held_items` and the two death tests on the PC with shots or a log (these are the only ones that could
   hurt a player), then `neutral_mobs` and `block_shapes`.
4. The (b) rows are test work, soon after landing: held_water x5 and the starter-outfit probes are the biggest.
