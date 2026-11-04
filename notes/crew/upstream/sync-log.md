# Weft sync: what is left after the merge (Upstream session, 11-04)

Merge e9c8405c on sync/weft-26.11.2. Things the merge dropped or left for an owner, to restore or check:

- server/liquid.rs: our held-water unit tests (freeze_pairs_wet_and_dry_states .. swimming_through_it) dropped with holds_liquid (decision 3); port the kelp/slab cases to upstream's waterlogging
- client/textures.rs: kept our Source::Look (core:look icons of parts tools) beside upstream's Source::Layers; port base parts tools to stack layers, then drop Look (keep a core:look read fallback for saved stacks)
- testkit: upstream 82ae06a8 deleted ~70 tests we never edited (gameplay battery..sprint_kicks_up_dust; ui scrollbar, pack_drag, mods_switch; world dynamite..distant_terrain_underground; extend names..facing_placement; visual use_swing..gizmos; audit liquid_surface, bodies_only; looks particles..item_model_file; redstone floater_up..saved_moving; bodies skiff..morph). Not re-listed. Restore from b5815f09 where Tester wants them in --test all.
- mods/testkit/src/tests/mod.rs:129 held_water_* to port to upstream waterlogging; gameplay.rs:163 core:equipment count with off-hand; testmod lib.rs:290 sea_slab waterlogged; testmod lib.rs:375 testmod:apple needs a FOODS row; base mobs.rs:376 blast vs waterlogged cells.
- base machines.rs keeps our free room_for (cooked stamps would break upstream's stacks_with).
- base parts tools: set stack `layers` (not only `core:look`): slot icons (ui/render.rs:783) and held items (HELD_LOOK now carries layers) read layers; keep a core:look fallback for saved stacks
- base shaderpack model.wgsl: upstream's hand and entity-effect hooks, keep fs_shadow (Lighting); mobs cast no sun shadow until then
- world_render.rs:1407 LOD fade for CubeTable-culled solid cubes (Lighting + Distant terrain)
- base: upstream renamed copper_lamp to golden_lamp (gold recipe); its golden_lamp textures are upstream's 16 px: Art to redraw at 32 px; our 32 px copper_lamp pngs are unused now
