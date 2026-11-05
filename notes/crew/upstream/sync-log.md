# Weft sync: what is left after the merge (Upstream session, 11-04)

Merge e9c8405c on sync/weft-26.11.2. Things the merge dropped or left for an owner, to restore or check:

- server/liquid.rs: our held-water unit tests (freeze_pairs_wet_and_dry_states .. swimming_through_it) dropped with holds_liquid (decision 3); port the kelp/slab cases to upstream's waterlogging
- client/textures.rs: kept our Source::Look (core:look icons of parts tools) beside upstream's Source::Layers; port base parts tools to stack layers, then drop Look (keep a core:look read fallback for saved stacks)
- testkit: upstream 2adf0ab1 ("remove useless tests") deleted 61 tests we never edited (one, creative_middle_click, we had). Tester's sort in notes/crew/upstream/lost-tests.md: restore 26, restore when cheap 11, let go 24. mods/testkit/ci-skip.txt still names three gone tests.
- mods/testkit/src/tests/mod.rs:129 held_water_* to port to upstream waterlogging; gameplay.rs:163 core:equipment count with off-hand; testmod lib.rs:290 sea_slab waterlogged; testmod lib.rs:375 testmod:apple needs a FOODS row; base mobs.rs:376 blast vs waterlogged cells.
- base machines.rs keeps our free room_for (cooked stamps would break upstream's stacks_with).
- base parts tools: set stack `layers` (not only `core:look`): slot icons (ui/render.rs:783) and held items (HELD_LOOK now carries layers) read layers; keep a core:look fallback for saved stacks
- base shaderpack model.wgsl: upstream's hand and entity-effect hooks, keep fs_shadow (Lighting); mobs cast no sun shadow until then
- world_render.rs:1407 LOD fade for CubeTable-culled solid cubes (Lighting + Distant terrain)
- base: upstream renamed copper_lamp to golden_lamp (gold recipe); its golden_lamp textures are upstream's 16 px: Art to redraw at 32 px; our 32 px copper_lamp pngs are unused now
- Upstream bug (a PR candidate for Ronald): client/particles.rs spawn indexed c.states with world.block() of a cell whose cube hadn't arrived (u32::MAX): a panic right after a teleport (SYNC4, world::crater). Fixed in the sync with .get().
- Upstream bug (a PR candidate): first person cut only bone roles (draw_own_hand, the body in view), not bone_mask: worn layers drawn by bone name kept every bone, so a shirt's torso and sleeves drew at the camera. Fixed in the sync (bones_with, Animals).
- Upstream candidate: a steered flying mount's flight counted as a fall on the server (mount_moved's track_fall didn't know it flew); MountMove now carries flies.
- Upstream bug (a PR candidate): sdk/ai.rs kept mob moods (anger, fear) in a process-wide static by entity id and game tick, never cleared when another world opened: a fresh world's spider took the last world's zombie's anger at the player (neutral_mobs after zombie_chases; players opening a second world too). Fixed in the sync: cleared when the seed changes or the clock goes back.
- Accepted follow-up (Ronald, revised: "Well, it's just use Weft's code then, if it's easier. Just have to make sure that our items are a bit bigger."): first person keeps Weft's hand (held items in the posed hand, Weft's hold clips; every item draws: Animals' ~/Coding/chiacraft-archive/images/animals-worker/2026-11-05/held_dev_sync.jpg). After the sync lands, Animals adds a small engine setting, a first-person scale for held items, which base sets to about dev's size, and updates held_items (and models' "first person hand waves") to that look on its branch. Both fail until then.

## Test fails that are not the sync (SYNC8, Tester)

- distant_terrain_from_above: 618 tiles shown, 198 still pending at y 1024 after 20 s: slow, not missing. Distant terrain's priority work; the test's 20 s limit may need raising after it.
- inventory_crafting and crafting_table_tags: both fail on dev b5815f09 too, before any crafting, on menu slot numbers (slot 37 isn't hotbar 0 any more), as inventory_hotkeys did; the recipes are right (one log gives 4 planks, two planks in a column 4 sticks; First hours). Tester is fixing the slot numbers.
