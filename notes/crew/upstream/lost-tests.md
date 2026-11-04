# Testkit tests lost in the Weft v26.11.2 sync (Tester, read only, nothing built or run)

Compared the `tests![ ... ]` list in `mods/testkit/src/tests/mod.rs` on dev b5815f09 (224 entries)
with sync/weft-26.11.2 at 4270141d (178 entries).

## Result in short

- **61 tests are gone.** Not 82ae06a8 as `sync-log.md` says: all 61 were taken out by one Weft commit,
  **2adf0ab1 "remove useless tests"** (VortexSo4, 2026-11-02, 3,024 lines deleted from 15 files). Found with
  `git log -S"<test>," -- mods/testkit/src/tests/mod.rs`: 61 of 61 point at it, none at us. It came in with the
  merge e9c8405c.
- **They are gone from the source, not only from the list.** None of the 61 functions exists in sync's test files,
  and the commit also deleted the support that only they used (below).
- **60 of the 61 are byte for byte Weft's old tests.** The one we edited is `gameplay::creative_middle_click`
  (we changed `slot:36` to `t.hotbar(0)`, twice).
- **15 tests are new in sync** (Weft's, from the same range): `gameplay::block_shapes, cell_parts, hitboxes,
  liquid_slot, model_bones, second_hand`, `looks::layered_icon, particle_rules`, `redstone::redstone_gate`,
  `ui::text_on_screen`, `world::camera, entity_edit, lod_columns, mob_crowd, riding`. So 224 - 61 + 15 = 178.
- `sync-log.md` says "~70": 61 listed here. dev's list has no `unify::` entries at all, so the 15 `unify`
  tests that 2adf0ab1 also removed are not part of the loss for dev.
- The `uration::from_secs` line that a naive diff shows is noise from the list's `Duration` import.

## Recommendation

- **Restore (26)**: they guard something chiacraft ships or is changing now. Every `t.*` helper and `t.op`
  command they use still exists in sync, so most are a copy and a list line.
- **Restore when cheap, or when the owner next touches that area (11)**.
- **Let go (24)**: Weft engine or mod-API checks that need deleted support, or are covered by a test that stayed.

How to restore one: copy the function from `git show b5815f09:mods/testkit/src/tests/<file>.rs`, add its line to
`tests![]` in `mod.rs`, then `cargo check -p testkit --all-targets`; a Tester run on the Mac or PC shows the
real result. I did not compile any of them (read only): what I checked is that each function's helpers
(`flat`, `look_at`, `op` commands, the `testmod:` blocks, `/morph`, `/gizmo`, `core:lod_distance`,
`lod_tiles`, `core:pick`, `mod_led`, `scrollbar`, `RESOURCE_PACKS`, `on_press_server`, `/music`) are still in sync.
Weft's waterlogging, LOD-from-saved-columns and the camera/riding rewrites changed engine behaviour under
some of them, so a restored test can fail for a real reason: expect to adjust numbers on the water and
distant terrain ones.

## Restore (26)

| Test | What it covered | Why for us | Weft replacement? | Port notes |
|---|---|---|---|---|
| gameplay::furnace_one_coal | one coal smelts exactly 8 items, no tick lost to starting on an item | base furnace/cooking (`cooking.rs`) | none | no support needed |
| gameplay::crafting_table_tags | planks (a tag) in a column of two give sticks in the 3x3 table | our recipes and crafting book changed a lot (logs give bark, planks via the book) | none | check the stick recipe and slot numbers (`slot:37` hotbar) against ours |
| gameplay::creative_middle_click | creative middle click picks a mob's egg / a full stack in the inventory; nothing outside creative | we edited it for our UI | none | take the hotbar-slot fix from dev |
| gameplay::mob_in_the_way | a mob in the way takes the right click, no block placed through it | engine, cheap, no mod | none | none |
| visual::pick_block | creative middle click on a block picks it | with `creative_middle_click` | none | none |
| visual::wall_torch | a torch on a wall hangs on it (`facing=south`) and drops when the wall goes | base torch; the night-torch rule | none | none |
| visual::ladder_placement | a ladder takes the facing of the wall | base ladder | none | none |
| ui::scrollbar | scroll bar on the controls screen scrolls on a click low on its track | UI redo rebuilt the screens | none | UI ids may have changed (`scrollbar`, `text:Walk forward`) |
| ui::pack_drag | resource packs move by their handles, every handle stays in the window | same | none | creates `testkit_a/b/c` pack dirs, needs `RESOURCE_PACKS` |
| ui::mods_switch | a mod's LED switches it for the next start, restart button shows only then | same, and our mods screen | none | ids `mod_led:testmod`, `mods_restart`, `mod:testmod` |
| looks::texture_seams | no light line at mip edges (snow-grass over dirt); a 64x64 texture among 16x16 | our 32 px block set and shader pack | none | none |
| looks::ctm_off_the_cube | random `.ctm` varies plants and box models too | 41 `.ctm` files in base | none | testkit has `varied.ctm` still |
| looks::item_model_file | an item with a model file shows its boxes in the hand and hotbar | we ship item model json (`assets/models/item`) | none | testmod totem still there |
| looks::block_model_file | a Blockbench Java model block (pedestal) is shaped right | same, for blocks | none | testmod pedestal still there |
| looks::distant_terrain_from_above | from high above, land all the way under the camera, no sky hole | LOD worker's area | `world::lod_columns` (saved columns only) | may need new LOD numbers |
| world::distant_terrain | past the view, tiles are made and drawn | same | `lod_columns` partly | `lod_tiles` and `core:lod_distance` still exist |
| world::distant_terrain_underground | none drawn deep under the land, back when coming up | same | none | uses `lod_tiles` |
| world::crater_view | the view reaches as far down as sideways; a crater's bottom stays loaded and lit beyond the rim | loading edge near water/LOD fixes | none | `crater_fast` op still there |
| world::sky_column_edit | an edit in the sky column over the view is announced, the client keeps it | same | none | none |
| bodies::natural_spawning | at night hostile mobs appear around the player; the rule switch stops it | base mobs, `mob_spawning` rule | none | the "no pig/cow" check must allow our animals mod |
| redstone::redstone_observer | an observer pulses from its back when what it looks at changes | base redstone (`redstone.rs`) | none | was in `ci-skip.txt` (a few-tick pulse missed on slow frames) |
| redstone::redstone_dispenser | a dispenser gives one item per new pulse | same | none | none |
| redstone::redstone_copper_lamp | a pulse flips a copper bulb; a comparator reads which way | same | none | none |
| redstone::redstone_spit | a sticky piston that lost power before it was out leaves its block out | same | none | none |
| redstone::redstone_saved_moving | leaving while a piston's blocks move loses none, makes none | same | `redstone_saved` (static) | none |
| redstone::redstone_floater_up | a floater goes up with what is on it | same | `redstone_floater` family | none |

## Restore when cheap (11)

| Test | What it covered | Note |
|---|---|---|
| audit::liquid_surface | `in_liquid` is false above a source's top, true inside | Weft's waterlogging may have moved the surface rule: check the numbers first |
| visual::third_person | the player model seen from behind and in front (shirt and skin colours) | our clothes and armor change the colours: adjust the probes |
| visual::crouch_pose | crouching lowers the third-person model (ratio 0.6 to 0.92) | pixel probe |
| visual::inventory_figure | the player figure in the inventory stays while mobs draw behind it | regression of a shared model buffer bug |
| visual::player_skin | a skin PNG goes to the server with the join and the model wears it | needs `png`; relevant to Profile/account work |
| gameplay::sounds | break/place/step sounds start, `/music` starts a track | needs the `play` op back in `server.rs`; better with the sound engine branch |
| redstone::redstone_first_piece | the first wire piece in a place shows | pixel probe |
| redstone::redstone_head_stays_inside | nothing of a piston head shows behind it | pixel probe |
| redstone::redstone_lift_api | a mod's structure: blocks out of the world, moved, and back | needs `putdown` and `bodygo` ops back |
| bodies::skiff_pause | pausing on a flying skiff and going on does not send it away at endless speed | engine pause bug, cheap |
| bodies::body_lit_at_once | a body is lit from the moment it exists; a broken block's place is lit at once | light, relevant to Lighting's work |

## Let go (24)

| Test | What it covered | Why |
|---|---|---|
| extend::client_entity_tick | a mod's client tick for pigs, `engine_effects` | Weft's mod API; its statics and `r.client_entity_tick` hook in `lib.rs` were deleted |
| extend::facing_placement | placing picks the allowed facing; `/setblock` with properties, `setprop` | needs `setprop` op and `testmod:vent`; our blocks have placement checks in the wall torch and ladder tests |
| extend::key_to_server | `on_press_server` keys reach the server, not while chat has input | needs the `PING` key and `pings` op, removed |
| extend::names | relative names of keys, events, items | Weft's naming rule; no support problem but no value to us |
| extend::typed_channels | typed `value!` channels both ways | the `Echo`/`GREET` channels were deleted |
| audit::bodies_only | `near_bodies` leaves out what can't be hit | op removed |
| gameplay::battery | testmod battery's energy bar | testmod only |
| gameplay::dust_rests_on_the_floor, landing_dust_scatters, sprint_kicks_up_dust | dust of broken, landed and sprinted blocks | `looks::block_dust_scatters` stayed, `looks::particle_rules` is new; `dust_rests` was already borderline on a GPU at 1280x720 |
| gameplay::creative_clicks, presses_never_wait | clicks at 1000 fps, presses never wait | the helpers `creative_clicks_fast`, `presses_fast` are gone; timing tests already in `ci-skip.txt` |
| liquids::mod_liquid_bucket, mod_sand_soil | a mod's oil in a bucket; red sand holds cactus | testmod blocks, and the liquid rules moved to waterlogging |
| looks::particles | bursts live and die out | replaced by `looks::particle_rules` |
| looks::gui_shader | the machine panel drawn by a GUI shader | engine feature we do not use |
| looks::tint_kinds | a mod's biome tint kind | engine API |
| visual::gizmos | the cube and light gizmos draw | debug aid |
| visual::use_swing | the arm swings only when a right click did something | low value |
| world::dynamite | testmod's dynamite digs a crater by area edit | testmod only |
| bodies::skiff, body_no_bob | riders move with a flying skiff; camera does not bob on a moving body | Weft's rewrite: `world::riding`, `redstone_flying_machine`, `redstone_flying_rider`, `ride_frames` |
| bodies::morph | `/morph` changes a player's box and look | `gameplay::hitboxes` and `world::entity_edit` cover the box side |
| redstone::redstone_for_mods | a mod's gate (`signal_logic`) and a wire that does not join | `signal_logic` is gone from the sdk; `redstone_gate` is its replacement |

## Support Weft deleted with them (needed only for the "when cheap" rows)

`server.rs` ops: `putdown`, `bodygo`, `pings`, `jumps`, `modes`, `bad_item`, `setprop`, `vel_y`, `play`,
`near_bodies`, `protect`. `lib.rs`: the `ECHO`/`GREET` channels, the pigs' client tick counters, the `PING`
key, and the blocks `and_gate`, `blue_wire`, `unbreakable`, `tinted_plant`. `probe.rs`: `yellow`, `teal`, `skin`
colour tests (`visual::player_skin`, `third_person` and `inventory_figure` use `skin`).

## Housekeeping found on the way

- `mods/testkit/ci-skip.txt` still names three tests that no longer exist: `presses_never_wait`,
  `dust_rests_on_the_floor`, `redstone_observer`. Drop them, or keep `redstone_observer` if it is restored.
- `notes/crew/upstream/sync-log.md` says the testkit loss was 82ae06a8 and about 70 tests; it was 2adf0ab1 and 61.
