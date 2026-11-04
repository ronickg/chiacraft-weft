# Upstream sync: a dry run (Upstream session, 11-04)

A plan, not a merge. Upstream Weft master `10fdced5` ("stack values over item values, parts in stack
storages, block and item info, shared block models, layered item icons") was merged with `--no-commit`
into a scratch branch of dev (`scratch/upstream-sync-dry`, dev `26969dd2`, worktree
`.claude/worktrees/syncdry`), the conflicts read, then aborted (`git merge --no-commit 10fdced5`
there shows them again). Nothing went into dev.

## Refreshed (11-04, before the freeze)

The dry merge again, on dev `af16f181` (241 merges past the first run's `26969dd2`) with upstream
`ca172ab8` (11 commits past `10fdced5`): scratch branch `scratch/upstream-sync-dry2`, worktree
`.claude/worktrees/syncdry2`, aborted afterwards. Nothing went into dev.

- **263 files conflict** (was 259): the same 174 textures and **89 code and doc files** (was 85).
  Dev's 241 merges added no new conflicted file. The 4 new ones come from upstream's new commits:
  `docs/redstone.md`, base's `en_us.lang` and `ru_ru.lang`, and `mods/testmod/src/lib.rs`.
- **Hunks grew** where upstream's new features landed: abi lib.rs 12→16, client/mod.rs 26→30,
  hud.rs 1→5, host.rs 11→12, registry.rs 11→12, dim.rs 25→26, sync.rs 4→5, terrain_gpu.rs 8→9,
  content.rs 8→9, receive.rs 3→4, world_render.rs 2→3, base lib.rs 1→3, testkit gameplay.rs
  10→11, visual.rs 1→2. Every other file is as in the tables below.
- **Upstream's new commits:**
  - riding: seats, chains, mob and player steering with presets, exits, rider rules, a riding
    pose carried through teleports and dimensions;
  - camera modes: freecam and eased transitions; `Client::view` is gone, replaced by registered
    camera modes and `camera_flags`;
  - ore variants and exposure, HUD icon effects;
  - `/attribute`, homes, `/back` (PR #72).
- **Two new calls these bring** (Planner or Ronald):
  5. **Riding.** Upstream now has its own riding, broader than ours: seats, chains, steering
     presets, exits, rules. Ours (2ab12f97 onward) carries the wyverns' flight and the rider's sit
     pose. **Recommendation: take upstream's riding** and put our flight steering and the wyvern
     onto its steering presets (Animals).
  6. **Camera.** **Take upstream's camera modes.** Our view-based code (the crosshair, the sun
     shadow of one's own body, `shadow_me`, the first-person hand) reads `camera_flags`
     (HAND, BODY) instead. The crosshair fix for Weft is already reworked that way
     (`fix/crosshair-third-person-clean` 93227e10, see for-ronald.md).
- **The Vulkan upload fixes** are still upstream's own, merged cleanly with ours (no change).

### Who works on which area

The order stays as below. Each area is done on one sync branch, built and looked at on the PC
before the next one starts.

| # | area | files (main ones) | who |
|---|---|---|---|
| 1 | abi, sdk, host: both sides' host functions, renamed ones chased through the mods | abi lib.rs, host.rs, sdk/*, registry.rs, content.rs | Upstream |
| 2 | Positions as `Vec3` (~200 sites), questions and events as `actions::*` (~86), SDK renames | all mods | Upstream for base and testkit; **Animals** for mods/animals; **Terrain** for base_api::land; **Plants** for base_api::trees, branchy and vitals' callers |
| 3 | Terrain (decision 1: our callbacks beside upstream's expressions), distant terrain (2: our LOD tiles) | server/terrain.rs, sdk/terrain.rs, terrain.wgsl, noise/fixed.rs, client/lod.rs, terrain_gpu.rs, gpu_layout.rs | **Terrain** (worldgen) with **Distant terrain**; looks on seeds 1-3 against dev's |
| 4 | Weather by id (4), ours registered on it | dim.rs, sync.rs, weather.rs, builtin.rs, regions.rs, base weather and weather_sound | Upstream, with **Lighting** for the sky, fog and pack side |
| 5 | Cell parts and waterlogging (3), liquids, mining on the server, drops at the feet | dim.rs, liquid.rs, mesher.rs, actions.rs | Upstream |
| 6 | Entities: upstream's animation clips, riding (5) and camera (6) under our painted skins, bone builds, sit pose, flight, mob sun shadows | entity_render.rs, models.rs, model.rs, client/mod.rs | **Animals** (models, riding, the wyvern), **Lighting** (shadows) |
| 7 | The rest: app.rs and client/mod.rs (`self.me`, the early list, occlusion), UI, HUD, lang, docs, testkit | app.rs, ui/*, hud.rs, screens, lang, docs | **UI** for screens and the HUD, Upstream for the rest |
| 8 | All of it: `cargo test --release --workspace`, `--test all` against today's baseline, the sheets, a playtest | | Tester 2 (PC), Tester, the playtesters |

---

## The size of it

- Merge base `6e4e1685`: upstream is **173 commits** past it, dev **1,294**.
- **259 files conflict**: 174 textures and **85 code and doc files** (84 in the tables below, plus Vulkan).
  Everything else merged by itself, but much of it **won't compile without follow-ups** (see "Beyond the
  conflicts").
- The biggest single change: upstream rewrote parts of the engine that our features build on
  (terrain features, weather, cell parts and waterlogging, entity animation, positions as `Vec3`,
  questions and events as `actions::*`). This isn't an afternoon's merge. My estimate is several days for
  one session, or one session per area (below) with a person checking each in the game.

## Decided (Ronald, 11-04)

- **When:** "Later, after the tropical round". The sync runs on a quiet dev, area by area, in the
  order below.
- **The four calls:** "Yes, all four":
  1. Terrain: keep our callback path beside upstream's expression features.
  2. Distant terrain: keep our LOD tiles.
  3. Waterlogging: take upstream's, and move base's water-holding blocks onto it.
  4. Weather: take upstream's weather by id, with ours registered on it in base.

## The decisions as they were put (with the reasons)

1. **Terrain.** Upstream turned custom terrain features into expression trees and dropped the
   callback features, `Fields` and `shape` that all of `base_api::land` (rivers, continents, coasts,
   lakes, canyons, karst, craters) and our LOD heights use. Either:
   - (a) port our worldgen onto upstream's expression features (a rewrite of `base_api::land`, some of
     it maybe not expressible), or
   - (b) keep our callback path beside upstream's (a lasting divergence in `server/terrain.rs`,
     `sdk/terrain.rs`, `terrain.wgsl`, `noise/fixed.rs`).
   **Recommendation: (b) for this sync.** Keep ours working, and port feature by feature later where
   upstream's form fits.
2. **Distant terrain.** Our smooth, morphing LOD tiles against upstream's simpler interpolation
   (`client/lod.rs`, `terrain_gpu.rs`, `gpu_layout.rs`). **Recommendation: keep ours** (an exception to
   "take upstream").
3. **Waterlogging.** Upstream re-implemented our blocks that hold water (`holds_liquid`) with its own SDK
   names. **Recommendation: take upstream's** and move base's water-holding blocks onto it.
4. **Weather.** Upstream's registered weathers (ids, `WeatherLook`) replace rain/thunder booleans
   under our biome rain, `/weather` override, air, snow and weather sound. **Recommendation: take
   upstream's registry**, and register our weathers in base on it.

## How every conflict resolves (the rules)

- Upstream's engine code is the base; our features go back on top.
- Where upstream re-implemented one of ours (the Vulkan upload fixes, waterlogging), upstream's wins.
- Our game-only parts (account, packages, worlds, survival hooks, shader-pack hooks, testkit lines,
  lang keys) stay.
- Both sides' new fields, functions, host-table entries, match arms and lang keys are all kept. The
  abi `Host` struct lists both sides' new pointers, and `host.rs` fills each.
- **Protocol: 13** (ours 12, upstream 2). `ABI_VERSION` is a hash of abi/lib.rs, so it changes by
  itself and every mod is rebuilt.
- **Damage causes:** our FREEZE id collides with upstream's HEAL (11). Register ours (freeze, heat,
  poison, bleed) through upstream's damage-cause registration.

## Vulkan (backends/vulkan/src/lib.rs): 2 conflicts, low risk

Upstream re-implemented our two upload fixes (#83, #84) as `1b9b2774` "drop uploads of destroyed
resources" and `1049ba95` "order overlapping upload copies", nearly word for word. Git merged both
with ours as one: one `copy_barrier`, one `uploads.retain`, no duplicates. Left over:
- the comment above destroy_buffer's retain: **take upstream's**;
- the depth image's usage: ours `DEPTH_STENCIL_ATTACHMENT | SAMPLED` (shader packs read depth),
  upstream's `| TRANSFER_SRC`: **keep both**, `DEPTH_STENCIL_ATTACHMENT | SAMPLED | TRANSFER_SRC`.
Our OOM spill (`allocate`, `spill_types`) merged by itself.

## Textures: 174 files, keep ours

Upstream redrew items and blocks at 16 px ("Redraw all items", "redraw all swords", ...); ours are the
32 px set (aba488ab and on).
- **155 both changed:** keep ours.
- **10 upstream deleted, ours changed:** bed_top_foot/head_90/180/270, copper_lamp(_lit),
  dispenser_front_vertical, stonebrick. Our beds.rs, redstone.rs and blocks.rs still use them (copper
  lamp: its lang and sounds), so **keep ours**.
- **5 we deleted, upstream changed:** glass/0-4. Ours dropped them, so **keep deleted**.
- **4 both added:** grass_top.ctm and grass_top/1-3. **Keep ours.**
- After the merge, Plants' missing-texture check (e2d63707) stops the game at startup naming any
  texture the merged code asks for and doesn't have: run it first on the PC.

## Order of work (one area at a time, each checked on the PC before the next)

1. abi, sdk and host (the tables' "ABI and SDK"): both sides' host functions, the renamed ones
   chased through base, animals and testkit. Builds nothing else until done.
2. Positions as `Vec3` (about 200 `pos[n]`/`size[n]` sites in our mods) and questions/events as
   `actions::*` (about 86): mechanical, by compiler errors.
3. Terrain (decision 1), distant terrain (2): worldgen looks on the PC, seeds 1, 2, 3, against dev's.
4. Weather (4): rain by season, snow, wind, weather sound.
5. Cell parts and waterlogging (3), liquids, mining (server-side timing now), drops at the feet.
6. Entity rendering and animation: painted skins, bone builds, sit and riding poses, mob shadows.
7. The rest (client/mod.rs and app.rs: `self.me`, the early list, occlusion), UI, lang, docs, testkit.
8. Then the whole: `cargo test --release --workspace`, `--test all` against today's baseline, the
   vitals and animals sheets, a playtest.

---

## Sync plan, list A (42 files): upstream 10fdced5 into dev, base 6e4e1685

Read-only pass over the conflict hunks in `.claude/worktrees/syncdry`, compared with `git show` on both sides. "Ours" means HEAD (dev). Hunk counts are `<<<<<<<` markers.

Five upstream rewrites cause most of the conflicts. The tables refer to them by these names:
- **W (weather registry):** rain and thunder booleans became weather ids from `reg_weather`/`reg_precipitation`. This came in `set_weather(weather, ticks)`, `weather_now`, `ToClient::Weather(u32)` and `Dim.weather: u16`. Our place-and-season weather (`set_rain`/`rain_in`/`raining_at`, `set_manual_weather`, `SNOWING` air flag) is built on the old booleans.
- **E (terrain expressions):** custom features stopped being Rust callbacks over `Fields`. Upstream made them `Expr`/`ExprStep` trees with named fields (`FIELD`, `SITE`), and the same trees run on the CPU (`Expr::eval`) and the GPU (`terrain.wgsl eval`). `Fields`, `shape::*` and `terrain_preset::CUSTOM` are gone. Our land (`base_api::land`: mesas, canyons, karst, lakes, rivers, craters), the `faces` callback, `continent_scale`, `thin` masks and the LOD `jobs` heights all depend on what was removed.
- **P (cell parts):** a cell now holds a main state plus `extras`, through `parts`, `has_part`, `as_part`, `replace_part`, `set_cell`, `shares_cell`, `liquid_view` and `fluid()`. Upstream waterlogs blocks this way. That does the same job as our `holds_liquid` (`holds`, `liquid_in`, `placed_into`, `filled`/`drained`, `left`, `source_of`).
- **R (render cleanup):** `entity_render` was split into `clips_of`, `pose_bones`, `place_carried`, `view_motion`, `pose_ground_item` and `draw_item_runs`. Poses became clip-based (`Motion { aim_root, clips }`, stock animations). Upstream also added avatars, first-person hands (`model_hand_pipe`), entity effects and layered stack icons. Opaque world meshes are culled and drawn on the GPU (`CubeTable`, occlusion).
- **S (struct ABI):** lists became structs (`Vec3`, `BlockPos`, `Rect`, `EntitySize`, `Dims`). Enums became consts (`GameMode::SURVIVAL`, `Pose::SWIM`). Many host and SDK functions were renamed (listed in the follow-ups).

### Engine client (rendering, shaders, net)

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| crates/engine/src/client/entity_render.rs | 28 | Mod-painted skins on worker threads (`painted`, `painters`, `paint_skins`, `Skin` bone `builds`), sun-shadow pipe with `draw_shadow`, stack looks (`model_key`, `HELD_LOOK`), riding seat offset, gaits by speed, swim limbs, worn layers by skin layout (`worn_tex_size`), inventory portrait. | R: avatars (`set_avatar`), `model_hand_pipe` with first-person `own_hand`, `effect_slots`, icon sheets, `item_models` keyed by `(item, layers)`, clip-based posing with pose LOD, `pose_ground_item`, `draw_item_runs`. | Take upstream's structure and layered stacks (`layers` replaces our `look`). Re-add painted skins and `builds` (scales into `pose_bones`), `shadow_pipe` with `draw_shadow`, `worn_tex_size`, and riding/gaits as clips or a seat offset in `clips_of`. | high |
| crates/engine/src/client/world_render.rs | 2 | `camera_sides`, and `mesh_draws(.., seen)` used by our sun-shadow pass (light-facing sides). `terrain_epoch += 1` in `remove` (shadow cache). | GPU-driven culling: the CPU's `mesh_draws` draws only structures and the translucent layer (`first_layer = 2`), the rest goes through `CubeTable`. `remove` also checks `table_only` and calls `table.remove(slots)`. | Take upstream's `remove` and add `terrain_epoch += 1`. Keep `seen` in upstream's `mesh_draws`, plus an all-layers mode for the shadow pass, or opaque terrain stops casting shadows. | high |
| crates/engine/src/client/mesher.rs | 3 | Held liquids: draws the liquid source around a block that `holds` one; `liquid(.., drawn)` compares with `liquid_in()`. | P: `draw_block` per part plus `n.extras`; `liquid(.., s, biome)` reads `n.liquid_view`. | Take upstream's `draw_block`/extras and `liquid_view`, and draw `c.source_of(l)` in `draw_block` when `info.holds`. Drops out if holds_liquid moves to parts. | medium |
| crates/engine/src/client/weather.rs | 1 | Snow wherever a mod's air says `air_flags::SNOWING` (`cold` flag). | W: `kind` from `c.biome_precipitation[biome]`; columns per `PrecipitationDef`. | Take upstream's `kind`, then `if world.air_at(cell).3 & SNOWING != 0 { kind = <core:snow id> }`. The snow id comes from the registry, not a literal 1. | medium |
| crates/engine/src/ui/font.rs | 5 | Smooth font: per-font cell size (`cell_origin(cell, w, h)`, `cell_w/cell_h/unit/top`), `unifont()` helper, `mono_width`. | Latin diacritics: `ABOVE` rows (`ROWS = CELL + ABOVE`), `STRIDE`/`STRIDE_Y`, inline hex parse with per-glyph metrics, drawn missing box, new `uv`. | Keep our parametrized cells and smooth path. Fold in upstream's `ABOVE` (the bitmap font's `cell_h = ROWS`), diacritic metrics and `missing_box` at `cell_origin(0)`. | medium |
| crates/engine/src/ui/text.rs | 1 | An empty key with no args shows nothing (Playtest B2). | `template_of(key, cx)`, which handles `TEMPLATE`-prefixed keys. | Take upstream's `template_of`, and in `message` return early when `key.is_empty() && args.is_empty()`. | low |
| crates/engine/src/audio.rs | 1 | `Voice::make_track` with handles, voice cap and `self.reach`. | `SoundDesc.full_within` and `falloff` easing curve (and music `holds` conditions, merged cleanly). | Keep `Voice`. Store `near` and the curve on `Voice` at play time and use `.distances((near, self.reach)).attenuation_function(curve)`. Make `audible()` honor `full_within`. | medium |
| crates/engine/src/model.rs | 4 | `Motion` gained `swim`, `sit` (riding pose), `holding`, `build` (bone scales; parent carries joint), with test literal. | R: `Motion { pitch, head_yaw, aim_root, clips }`; procedural vanilla swing/crouch replaced by clips (`replaces`, mirror, aim); `euler()`. | Take upstream's `Motion` and add `build: &[[f32; 3]]` with our parent-offset code. Make riding a sit clip (or apply it after the clips). Drop `swim`/`holding` if stock clips cover them. | high |
| crates/engine/assets/shaders/terrain.wgsl | 2 | `ridge`/`continent` (per-dimension `D_CONT_SCALE`)/`temperature`/`humidity`/`craters`. LOD: morph offset `- off` and `column(job, ..)`, which reads CPU-made custom heights from `jobs`. | E: everything is `eval()` over `EXPR_*` steps; `craters` became `site()`; `column(bx, bz)`. | Take upstream's evaluator and `site`. Re-add `- off` and a feature kind that reads `jobs[...]` for our CPU-callback features. Drop `D_CONT_SCALE` (the `continent` field takes the scale). | high |
| crates/engine/src/net/p2p.rs | 3 | chiacraft ALPNs (status, links, worlds, bans), `PROTOCOL = 12` with history, `MAX_FRAME`, `identity()` from `chiacraft_account`, `read_frame(.., max)`. | `PROTOCOL = 2`, file-based `identity.key`, `super::MAX_MESSAGE`. | Keep ours and bump `PROTOCOL` to 13, because upstream's wire changes (S, `Use.part/hit`, `Weather(u32)`) break the old one. Keep the `max` parameter; check that `MAX_FRAME` covers `MAX_MESSAGE`. | low |
| crates/engine/src/net/codec.rs | 3 | Tuples up to 6; test `Join { account, did }`, `Pose::Sit`. | Tuples up to 7 (sprinting in moves); test `Use { part, hit }`, `Pose::SWIM`. | List both tuple arities (5, 6 and 7). Test: `Join` with our fields, upstream's `Use`, pose `SWIM` or our sit as the new pose const. | low |

### Engine server

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| crates/engine/src/server/dim.rs | 25 | `SetRain`/`SetAir`/`SetTints`/`Flash` events; `raining`/`thunder`/`rain_in`/`air`; `SCATTER_UP = 96`; `mining_time`/`break_time` (broken tools mine like the hand); `HELD_LOOK`/`mount` mirroring, 20-byte shape (scale); `c.left()` on set; holds-liquid scheduled and neighbor ticks; `liquid_in` in spawn spots; liquid temperatures; `seat_riders`; `cell_has` near-support; `drop_mined` (drops at the player's feet, `own_name`); `placed_into` in `place`. | W: `weather: u16`, `SetWeather`. `reach_y` per scatter. `mining_rate` (per-tick share, creative). Off-hand and `using` fields, `EntitySize`. P: extras on set, `parts`/`as_part` ticks, `neighbor_changed` per part, `place` with `merge`/`replaced`/shape boxes. `spawn_place` rules, `burns()`, `fall_effects`, `drop_block_loot(.., carried)`, `loot_rolls`. | Take upstream throughout and re-add ours on top. Keep `air`/`rain_in` beside `weather` (drop `raining`/`thunder`). Use `reach_y` instead of `SCATTER_UP` (base sets `reach_y(96, 16)` on giant trees). Add `c.broken(tool)` inside `mining_rate`. Extend `field_comps` with `mount`/`held_look` and the scale bytes. Holds-liquid ticks go inside the parts loop and `neighbor_changed`. `placed_into` goes before `parts` in `place`. Re-add `drop_block_loot_at` and `drop_mined` with `carried`. Keep both temperature and `burns`. Keep `seat_riders` and `hurt_at`. | high |
| crates/engine/src/server/terrain.rs | 9 | `Height::Custom(ModId, Callback)` with `FeatureArgs`, MaskStep `Mask::from_steps`, `faces` callback, `continent_scale`, `self.fields(seed)`. | E: `RawStep`/`Expr::resolve`/`Step`, named `Field`s (`standard_fields`), `needed`-field pruning, `Terrain::new(.., fields, ..)`. | Take upstream's `Expr`/`Field` and keep, as additions, a custom-callback height on `Feature` (CPU only, its heights uploaded for LOD), `faces`, and the face args. Drop `Mask` (port `thin` to `Expr`) and `continent_scale` (redefine the `continent` field). | high |
| crates/engine/src/server/liquid.rs | 5 | Held liquids in `spread_to` (`liquid_in`), reactions drop the block's loot first, bucket fill into `c.filled`, bucket drain to `c.drained`; `BURNING` const; held-water unit tests. | P: `self.fluid()`, `extras`, `replace_part`; `action::BREAK_BLOCK`, `Vec3`/`BlockPos` query; `BURNING` removed (`LiquidDef::burns`). | Take upstream's parts calls and keep our held-liquid branches next to them (fill/drain via `filled`/`drained` when the target `holds`). Drop `BURNING` (fix `path.rs`). Keep the tests and update `RawState`/`RawBlock` literals to upstream's fields. | high |
| crates/engine/src/server/sync.rs | 4 | `core:entity_send_distance` (send radius in blocks), per-entity `moved_at` throttling (`Moved`), `ToClient::Weather { raining, thunder, only }` + `Air` + `Tints`, `mining_now` with `mining_time`. | `GameMode::SURVIVAL`, `send_entities(.., view.radius, ..)`. Mining progress per tick (`Mining { progress, part }`, `Crack`, `mining_rate_of`). `sprinting` in moves. `ToClient::Weather(u32)`. | Take upstream's mining and `GameMode` consts. Keep our send distance and `moved_at`, adding `e.sprinting` to `mv`. Send `Weather(dim.weather)` plus our rain-biome list (a new message or field), `Air` and `Tints`. | high |
| crates/engine/src/server/player_file.rs | 2 | Join log with account identity, probation notice; saves `identity`/`did`/`played`/`probation`. | `p.glow_sent = false`; `GameMode::SURVIVAL`. | Keep both, with `glow_sent` before our block re-borrows `p`. Use `GameMode::SURVIVAL` in our `if`. | low |
| crates/engine/src/server/builtin.rs | 2 | `/rule` value as `word` (true/false names); admission commands (ban, tempban, unban, bans, allow, probation, revoke, achievementrule, didrule, banlist); `/weather` through `set_manual_weather(raining, thunder)`. | `title`/`say`/`msg`/`tellraw` commands; `/weather` by name to weather id, `d.set_weather(w as u16, ticks)`. | Keep both command lists with our `value:word`. Make it `d.set_manual_weather(w as u16, ticks)`, with `regions.rs::set_manual_weather` taking the id. | medium |
| crates/engine/src/content.rs | 8 | `DimensionDef::fields`, `daylight_clock` (seasons: `year_days`, `day_swing`), `ids.keep_broken`, `filled`/`drained`/`state_names`, `harvests` with broken tools, `freeze_held_liquids`, `HELD_LOOK` field comp. | `has_weather()`, a per-dimension daylight curve, `use_priority`/`two_handed`/`one_handed`, `fluid_items`/`item_fluid`/`biome_precipitation`/`damage_immune`, `shares_cell`, field comps `OFF_HELD`, `USING`, `SWAPS`, `LOOPING_ACTION`, `EFFECT`, `FORCED_POSE`. | Keep both sides' fields, ids, maps and freeze steps. Drop `fields()` with `Fields`. Apply `daylight_clock` before upstream's curve. Put `HELD_LOOK` after upstream's comps so `field_comps[7..10]` keep their indices (registry `[u32; 13]`). | medium |
| crates/engine/src/loot.rs | 1 | `stack_value`/`harvests(&tool)`, the same as upstream. | New `lc::CONDITION` arm (`ctx.holds`); `Ctx` loses `Copy`. | Take upstream. | low |
| crates/engine/src/config.rs | 1 | `core:entity_send_distance`, `entity_activation_distance`, `inactive_entity_interval`. | `first_person_model`, `show_avatars`, `occlusion_culling`, `allow_avatars`, `keep_inventory`. | Keep both. | low |

### ABI and SDK (shared crates)

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| crates/abi/src/lib.rs | 12 | `ScatterDesc.thin/thin_keep/thin_biomes`; damage causes `FREEZE`/`HEAT`/`POISON`/`BLEED` (old `\| BYPASSES_ARMOR` ids); `entities_near`, `entity_item`, `entity_set_item`; W-era `set_weather(raining, thunder)`, `raining`, `thundering`, `set_rain`, `raining_at`, `set_air`, `air_at`, `entity_flash`; `set_entity_look`, riding (`entity_mount`, `_of`, `_rider`, `_keys`, `_mount_seat`), `reg_entity_skin`, `set_entity_scale`, `entity_fly`, `entity_ignite`; `dim_surface`, `entity_tick_elapsed`, `set_biome_tints`, `content_tint`, `client_sound_start/set/stop/playing`, `set_day_rate`; `SoundDesc.priority/voices`; `NATURAL_SPAWN`; `FeatureDesc` callback and `FeatureArgs`; `terrain_op::FACES = 15`, `CONTINENT_SCALE = 16`, `FaceArgs`, `face`. | `reach_up`/`reach_down`/`ground`. Registered causes (`DamageCauseDesc`, `damage_flags`, `HEAL = 11`, `NAMES`). `entities_in(shape)`. `set_weather(weather)`, `weather_now`, `client_weather`, `weather_flags`, `reg_weather`, `reg_precipitation`, `entity_effect`, `reg_entity_effect`, `reg_text_place`, `reg_game_mode`, `reg_pose`, `entity_set_pose`, `game_mode_flags`. `set_entity_appearance`, `dim_surface_block`. `full_within`/`falloff`. `HURT`/`INTERACT_ENTITY`/`USE_ITEM`/`DIE`. `FeatureDesc` as `ExprStep`. `CAVE_BIOME_REMOVE`, `FIELD = 15`. | `Host` lists both sides' new pointers and upstream's renames: drop the base-era `raining`, `thundering`, `entities_near`, `entity_flash`, `set_entity_look` and `dim_surface`. Keep `set_rain`, `raining_at`, `set_air`, `air_at` and the rest of ours, switching `[i32;3]`/`[f64;3]` arguments to `BlockPos`/`Vec3`. Our damage causes are removed from `damage_cause` and registered by base with `BYPASSES_ARMOR` (their ids clash with `HEAL = 11`). Renumber `FACES` to 16 and drop `CONTINENT_SCALE`. Add `custom: Callback` beside the `ExprStep` height. Keep `FaceArgs`/`face` with `field`, and both default sets. | high |
| crates/engine/src/host.rs | 11 | Table and bodies for the ours-new functions above, plus base-era names (`entity_look_at`, `entities_near_bodies`, `structure_block`, `set_structure_block`, `lift_blocks`, `put_down_body`, `body_go`, `dim_carved`, `dim_surface`, `area_fill`). `SoundDef.priority/voices`. | Renames and replacements: `entity_in_reach`, `raycast`, `entities_in`, `body_from_blocks`, `body_to_blocks`, `body_fall`, `push_blocks`, `world_enter_body`, `world_leave_body`, `body_move_to`, `dim_surface_carved`, `dim_surface_block`, `fill`. New: `weather_now`, `reg_weather`, `reg_precipitation`, `weather_flags`, `entity_effect`, `reg_entity_effect`, `reg_text_place`, `reg_game_mode`, `reg_pose`, `entity_set_pose`, `game_mode_flags`, `set_entity_appearance`; `BlockPos` in `biome_at`; `full_within`/`falloff`. | Fill every `Host` field. Use upstream's names for the base-era functions, and keep only our new ones: `entity_item`, `entity_set_item`, `set_rain`, `raining_at`, `set_air`, `air_at`, `set_biome_tints`, `content_tint`, riding (`entity_mount`, `_of`, `_rider`, `_keys`, `_mount_seat`), `reg_entity_skin`, `set_entity_scale`, `entity_fly`, `entity_ignite`, `entity_tick_elapsed`, `client_sound_*`, `set_day_rate`. Drop `entity_flash` (`entity_effect` with `core:flash`). Rewrite `set_rain`/`raining_at` on weather ids. Set both `SoundDef` field pairs. | high |
| crates/sdk/src/terrain.rs | 8 | `Feature::new(closure over &Fields)` with `CUSTOM` preset and `params`; TerrainOp `face`; scatter `thin`/`thin_biomes` (MaskStep); doc "up 96". | E: `Feature::new(height: Expr)`, TerrainOp `field`, scatter `ground`, `reach_y` ("32 and 16 unless `reach_y`"). | Take upstream's `Expr` API. Add `Feature::custom(closure)` for the CPU callbacks our land needs (or port `base_api::land` to `Expr`). Keep `faces`, and port `thin(..)` to `Expr` masks next to `ground`. Take upstream's doc. | high |
| crates/sdk/src/game.rs | 6 | `Skin` with `scale_bone` (skin painters), `EYE_SHARE`, `damage` re-export; `fly`; `look_at` (base); `dropped_item`/`set_dropped_item`; `component_mut` doc; `MenuCtx::slot`, `MenuGroup` (`index_of`, `cols`, `rows`); recipe `hidden`/`shape`/`inputs`/`accepts`. | Re-exports `EYE_SHARE`, `damage_cause`, `damage_flags` from abi. `raycast(from, dir, ..)` replaces `look_at` (`LookAt.body`). `set_component`. `SlotGroup` with `find_group`. Liquid results doc. | Keep `Skin`, `fly`, `dropped_item`, `set_dropped_item`, `slot()` and the recipe info. Take upstream's re-exports, `raycast`, `set_component` and `SlotGroup`, adding `index_of` to `SlotGroup` (base `parts.rs`/`player.rs` use it). Drop `look_at` (callers use `raycast` from the eyes). | medium |
| crates/sdk/src/content.rs | 2 | `ItemBuilder::keeps_broken`; `DimensionBuilder::seasons(days, day_swing)`. | `use_ticks` doc tweak; `sky` renamed `sky_color`. | Keep both and use upstream's doc and `sky_color`. | low |
| crates/sdk/src/lib.rs | 1 | Re-exports `air_flags`, `held_keys`, `worn_parts`, `BiomeAir`. | Re-exports `BlockPos`, `EntitySize`, `Holder`, `OpenMenu`, `Rect`, `UV_*`, `Vec3`, `Who`, `Hand`. | Use the union of both lists. | low |
| crates/sdk/src/effects.rs | 1 | `SoundBuilder::priority`, `voices`. | `SoundBuilder::falloff(full_within, Falloff)` and the `Falloff` enum. | Keep both. | low |
| crates/noise/src/fixed.rs | 1 | `CONTINENT_SCALE`, `Fields` (`with_continent_scale`, `fbm`, `ridge`, `continent`, `temperature`, `humidity`), the `shape` module (`continents`, `hills`, `mountains`, `plateaus`, `canyons`, `dunes`, `craters`). | E: removed `Fields` and `shape`, added `site(seed, x, z, spacing, chance, radius)`. | Add upstream's `site`. Keep `Fields` and `shape` only while `base_api::land` still uses them (CPU callbacks); delete them once land is ported to `Expr`. | medium |

### Base

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| mods/base/src/worldgen.rs | 3 | Forest layers from `vitals::layers`, overworld shape moved to `crate::land::terrain`; stand, giant, boulder, shrub and young-tree scatters with `thin`/`thin_in`; `tree()` via `wild_tree_at`, `site_open`, `FLOWER_MEADOW`, `CLEARING`. | The shape inline in `Expr` terms (`preset::*`, `.mask(Expr::...)`, `field("base:mountains")`); tree scatter `.ground("#plant_soil")` with an inline shape; `grow_tree(w, p)`. | Keep ours. Port `thin(..)` to `Expr` masks, add `.ground("#plant_soil")`, give giant trees `.reach_y(96, 16)`, use `BlockPos` from `s.pos()` and `put_force`. Upstream's inline overworld is not wanted (land.rs replaces it). | high |
| mods/base/src/blocks.rs | 7 | Log `bark` axis, mossy sequoia log, planks helper, leaves with autumn tint (`leaf_tint`) and `cull_same`, `opacity(1)`, dogwood loot; extra tint kinds; plant growth by `growth::step_secs`/`growth::grow`, `needs_near_support`. | `face_turns` on log sides, `light_opacity`, `&tints::FOLIAGE`, `has_shapes` tag with stairs, slabs and vertical slabs; `grows`, `needs_near`, `e.pos().to_array()`. | Keep ours and add upstream's `face_turns` and `has_shapes`. Rename to `light_opacity`, `needs_near` and `grows` (our step times go in `grows`). Pass `e.pos()` to `grow_tree` as `BlockPos` (or `.to_array()`). | medium |
| mods/base/src/machines.rs | 5 | Furnace and chest screens through `crate::window`; `cooked()` stamps food made and hot; `room_for`; `use_one` (remainder). | `ui.panel` with `ui.title()`, `.default_screen()`, `ItemStack::merged(out, n)`, `room_for(out)`, `consume_one`-style `shrink`. | Keep our screens and `cooked()`. Use `output.merged(out, n)` when the slot isn't empty and upstream's `room_for` method in `fits`. `use_one` becomes `consume_one`. | low |
| mods/base/src/advancements.rs | 3 | Vitals effects and milestones in the check; toast at `TOAST_TOP`; screen through `window_titled` with a stats tab (`open_screen`). | `st.pos.y`; `draw_screen_text`, `TOAST_MARGIN`; `ui.view()`; `ui.panel`. | Keep ours, using `pos.y`, `draw_screen_text`, `ui.view()` and `screen_open`. | low |
| mods/base/src/redstone.rs | 1 | Dispenser screen through `crate::window`. | `.default_screen()`. | Keep ours (`MenuBuilder::screen` still exists). | low |
| mods/base/api/src/lib.rs | 1 | `eat()` goes through the vitals stomach, with poison and sickness resisted by fortitude. | `h.food`/`saturation` with `consume_one(stack)`. | Keep ours, with `consume_one(stack)` (a bowl's remainder) in place of `stack.shrink(1)`. | low |

### Testkit

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| mods/testkit/src/tests/mod.rs | 3 | `mod held` and the list entries `account`, `packages`, `package_pins`, `held::held_water_*`; keeps `mod liquids`, `ui::scrollbar`, `extend::client_entity_tick`, `extend::facing_placement`. | Pruned list: `liquids.rs` deleted, those test functions removed, many others dropped. | Keep `mod held`, account, packages and held water. Drop `mod liquids`, `liquids::*`, `ui::scrollbar`, `extend::client_entity_tick` and `extend::facing_placement` (their functions are gone). Re-add the tests whose bodies we keep (follow-ups). | medium |
| mods/testkit/src/tests/looks.rs | 4 | Fall check against `max_health`, `t.hotbar(0)`, sky test hides the HUD, sunset at `time 180100` (seasons); `attack_indicator` (amber fill) and more. | `pos.x`, `Rect`, `slot:36`, `input_cursor`, `set_look`; deleted `attack_indicator`, `third_person_shakes`, `block_dust_scatters`, `translucent_order`; new light test. | Keep ours on upstream's API (`pos.x`, `Rect { .. }` with `t.hotbar`, `input_cursor`, `set_look`). Keep our kept tests and upstream's new light test. | medium |
| mods/testkit/src/tests/audit.rs | 1 | Kept and edited 15 tests (`respawn_keeps_mode` … `creative_sword_no_flicker`). | Deleted those tests. | Keep ours, ported to S (`pos.x`, `Rect`, `input_cursor`, `set_look`), and listed again in mod.rs. Or drop the ones we never touched; ask Planner. | medium |
| mods/testkit/src/tests/ui.rs | 2 | `ui_rect(&t.hotbar(i))`, `cursor`. | `Rect { .. }`, `slot:36/37`, `input_cursor`. | `let Some(Rect { x, y, w, h }) = t.c().ui_rect(&t.hotbar(i))` with `input_cursor`. | low |
| mods/testkit/src/tests/visual.rs | 1 | `ui_rect(&t.hotbar(slot))` for slots 0 to 4. | `slot:36..40` and `Rect`. | Ours, with `Rect { x, y, w: sw, h: sh }`. | low |

### Docs

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| docs/ui.md | 1 | `open_screen` and a paragraph on `rebuild_screen`. | Renamed to `screen_open`. | Upstream's line plus our `rebuild_screen` paragraph. | low |
| docs/models.md | 1 | "the item's own model file (above)". | Link to `#blocks-and-items`. | Upstream's (the anchor exists). | low |

### Lang

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| crates/engine/assets/lang/fr_fr.lang | 2 | `command.sounds.*`; the chiacraft block (admission, DID, achievements, listing, finder). | `command.title.done`; `chat.screenshot*`. | Keep both. | low |

### Follow-ups outside the conflicts

Renamed or replaced APIs to chase (callers counted in the merged tree):
- Upstream renamed or replaced these, so every caller has to change:
  - `open_screen` becomes `screen_open`, about 22 calls in packages, portals, base and testkit.
  - `ui.client()` becomes `ui.view()` (packages, base `book`/`player`).
  - `screen_text` becomes `draw_screen_text` (base `levels`/`hints`, `app/api.rs`).
  - `cursor` becomes `input_cursor`, `look` becomes `set_look`, and `ui_rect` now returns `Rect`.
  - `pos[0..2]` becomes `pos.x/.y/.z` throughout testkit and base.
- Host and SDK functions upstream replaced:
  - `set_entity_look` becomes `set_entity_appearance`.
  - `entity_flash` becomes `entity_effect(core:flash)` (sdk `client.rs`, base `mobs.rs`).
  - `entities_near` becomes `entities_in` (sdk `ai.rs`/`shape.rs`, base `mobs.rs`, testkit `server.rs`).
  - `raining()`/`thundering()` become `weather_now` plus `weather_flags` (sdk `client.rs`/`view.rs`, packages `client_half.rs`).
  - `look_at` becomes `raycast`, about 60 calls in testkit and sdk.
  - `dim_carved`/`dim_surface` become `dim_surface_carved`/`dim_surface_block` (sdk `terrain.rs`).
  - `area_fill` becomes `fill`.
  - Bodies: `structure_block`, `lift_blocks`, `put_down_body` and `body_go` become `body_*`, `push_blocks` and `world_enter_body`/`world_leave_body`.
- Builders: `opacity` becomes `light_opacity`, `needs_near_support` becomes `needs_near`, `growth` becomes `grows`, `sky` becomes `sky_color`, all in every mod (animals and packages too). `InventoryGroup`/`MenuGroup` become `SlotGroup`, which needs `index_of` (base `parts.rs`, `player.rs`).
- Enums became consts:
  - `GameMode::Survival` and similar become `GameMode::SURVIVAL` (`actions.rs`, `client/mod.rs`, `sync.rs`, `player_file.rs`).
  - `Pose::Sit`/`Stand`/... become consts (about 23 uses in `physics.rs`, `world/mod.rs`, `receive.rs`, `entity_render`, `host`). Our riding `Sit` needs a registered pose (`reg_pose`) or a const.
- Damage causes: base `survival/body.rs` and animals `comfort.rs` use `damage_cause::FREEZE`, `POISON` and so on. These become causes registered by base with `damage_flags::BYPASSES_ARMOR`. Check `vitals`' death-message keys.
- Mining: `dim::mining_time`/`break_time` become `mining_rate` (`actions.rs`, `client/mod.rs`, `sync.rs`). Our broken-tool rule moves inside `mining_rate`.
- `liquid::BURNING` is gone, so `server/path.rs` uses `LiquidDef::burns()`.
- `loot::Ctx` lost `Copy`; check our copies of it (and `drop_block_loot`'s new `carried` argument everywhere we call it).
- `regions.rs` (UU, not in this list): `set_weather`/`set_manual_weather`/`set_rain`/`rain_here` must move to weather ids (`Dim.weather`), and `ToClient::Weather` plus the client's `rains_at`/`air_at` follow. That is one piece of work with W across `dim.rs`, `sync.rs`, `host.rs`, `abi`, `builtin.rs` and `weather.rs`. Base's seasonal weather (`set_rain(biomes, thunder)`) must pick a weather id (rain or thunder).
- `registry.rs`/`client/mod.rs` (`field_comps`): grow to 13 with upstream's 11 first, then `MOUNT`, `HELD_LOOK`. `client/mod.rs:2147` destructures our 7.
- Terrain (E) is one piece of work: `client/terrain_gpu.rs`, `lod.rs` and `gpu_layout.rs` carry our `jobs` heights and morphing. `base_api/src/land.rs` uses `Feature::new(|f, x, z| ..)`, `Fields` and `shape::*` (about 12 features). Either keep a CPU-callback feature kind, or port land to `Expr`, which is the cleaner end state and is needed for upstream's GPU-only LOD. `mods/testmod/src/lib.rs:362` also uses `Feature::new(closure)`.
- holds_liquid against cell parts (P): the merge can keep both. Deciding whether to move our waterlogging (`holds`, `filled`/`drained`, `placed_into`, `left`) onto upstream's extras (liquid as an extra part) is for Planner or Ronald. Until then, `support_near` (`cell_has` vs `block_has` + extras), the mesher and the liquid ticks each check two mechanisms.
- `entity_flash` users get `entity_effect`. Upstream's `glow`/`hurt` effects may duplicate our glow masks; check that our base mobs' flash still shows.
- Things that compile but behave differently:
  - Default scatter reach upward drops from 96 to 32 (`reach_y`), so giant sequoias are clipped unless base sets `reach_y`.
  - Opaque terrain leaves the CPU draw list (GPU culling), so our sun-shadow and far-shadow passes may lose terrain casters.
  - `core:show_avatars`/`allow_avatars` default to on.
  - `core:occlusion_culling` defaults to on (check it with the shader pack).
  - `keep_inventory` is a new world rule; check it against our respawn kit and death rules.
  - Music conditions are now `holds` ids.
  - The spawn rules' `spawn_place` changes where our animals spawn (water and ceiling).
- Testkit: re-list the tests we keep but upstream unlisted:
  - `audit::` `respawn_keeps_mode`, `kill_and_void`, `leaving_keeps_cursor`, `containers`, `water_keeps_flowing`, `ladder_no_fall`, `bucket_not_wasted`, `shift_click_skips_grid`, `no_block_into_mob`, `tp_nan`, `changes_in_one_tick`, `no_ghost_light`, `chat_down_keeps_text`, `break_refused`, `creative_sword_no_flicker`.
  - `looks::` `attack_indicator`, `third_person_shakes`, `block_dust_scatters`, `translucent_order`.
  - Check `gameplay.rs`/`visual.rs`/`bodies.rs` the same way. There are 16 old-API calls (`.look(`, `.cursor(`, `pos[i]`) inside our sides of the testkit hunks.
- Tests to rerun on the PC after the merge:
  - Engine unit tests: `liquid.rs` (held water), `codec` roundtrip, `model` pose, `terrain` (Expr against GPU).
  - testkit `held::held_water_*`, `gameplay::mining_and_wear`/`armor`/`food`, `looks::sky` (seasons sunset), `ui::inventory_*`, `account`/`packages`.
  - Worldgen looks (land shape, sequoias, LOD from above).
  - Weather and snow runs, mob shadows, multiplayer (PROTOCOL bump).
  - The vitals and animals sheets.
- Bump `p2p::PROTOCOL` (ours 12 → 13). `ABI_VERSION` is a hash of the file, so it changes by itself and every mod must be rebuilt.

### Riskiest files (summary)

1. Terrain (E), the worst: `server/terrain.rs`, `sdk/terrain.rs`, `terrain.wgsl`, `noise/fixed.rs` and abi `FeatureDesc` dropped the callback, `Fields` and `shape` features that all of `base_api::land` and the LOD `jobs` path depend on.
2. Weather (W): `dim.rs`, `sync.rs`, `host.rs`/abi, `builtin.rs` and `weather.rs` replace rain/thunder booleans with weather ids, under our biome rain, `/weather` override, air and snow; `regions.rs` has to be rewritten with them.
3. `server/dim.rs` (25 hunks) and `liquid.rs`/`mesher.rs`: upstream's cell parts re-did placement, ticks, neighbor updates and waterlogging, where our holds_liquid, `drop_mined`, mining and riding sit.
4. `client/entity_render.rs` (28 hunks) and `model.rs`: an upstream rewrite (clips, avatars, first-person hands, layered icons) to re-apply painted skins, bone builds, mob sun shadows and riding onto.
5. `abi/lib.rs` and `host.rs`: about 25 of our new host pointers to keep, about 12 base-era names upstream renamed (`set_entity_appearance`, `entities_in`, `raycast`, `dim_surface_block`, `body_*`), and damage causes moving to registration; `world_render.rs` GPU culling may also drop terrain from our shadow passes.

---

## Sync plan, half B (42 files): upstream Weft `10fdced5` into dev, base `6e4e1685`

Hunk counts are the worktree's conflict blocks. "Ours" = chiacraft dev (HEAD), "upstream" = Weft master.
Most of upstream's side comes from five commits: `35549b38` (big rename and registry pass: actions, weathers, poses, damage causes, game modes and entity effects become registered values), `1e41c99c` (SDK unification), `9ea027e2`/`2d028b6c`/`a7a3d5d8` (arrays become `BlockPos`/`Vec3`/`Dims`/`EntitySize`/`Rect` structs), `1b8b8fd2` (GPU worldgen rewritten as expression fields), `0158b408` (cells hold several parts: "extras"), plus `10fdced5` (stack values, parts in stack storages, layered item icons) and `1dbceaf3`/`2fe06fcb` (hands, first-person model, avatars, stock animations).

### Engine client

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| crates/engine/src/client/mod.rs | 26 | Rain by biome, `Air` (fog, wind, snow, dust, aurora), `day_rate`, riding (`mount`, `seat_forward`, `keys`), `held_look`, `liquid_in()` for eye and drips, sun and moon shadow maps (`prepare_shadows`, `render_shadows`, `moon()`), `hand_light`, `shadow_me`, `diag::span`s. | Local player becomes a full `ClientEntity` (`self.me`); `rain`/`thunder` become `look: WeatherLook` driven by `weather_id`; `since_time: f64` plus `tick_rate`; off-hand, `using`, `swaps`, `effect`, `forced` pose field components; `render(.., list, early, ..)`; `Pose::def().bob`; `Dims`; `mining_rate`. | Take upstream's structure (`me`, `look`, `render` with `early`, the field components). Put our mount/held_look/scale fields on `ClientEntity` and our mount and shape handling into upstream's `mine` branch. Keep our air/fog/wind/aurora easing next to upstream's look easing, and gate rain by `world.rains_at`. Keep the shadow functions whole, with `shadow_me` drawing `self.me`. | high |
| crates/engine/src/client/models.rs | 13 | One shared geometry per model file (`Shared`, `anim_files`), a `run` clip, a `sit` pose, `Batch::Shadow` and `shadow_only`, per-entity bone builds (`build`, `scale_bone`), up to `MAX_ANIMS` 3 blended clips. | Animation system rewritten: per-`Act` clips (`actions`, `engine`, `items`), stock animations per body shape, `Posed.clips`, `PoseLod` kept bones, `Batch::Hand`/`Icon`, item models, avatars, bone motions (`live_bones`). | Take upstream's clip system and `push_with_stock`. Re-add our per-file sharing in `push_with_stock`, `Batch::Shadow` (instances 5, bones 4), `shadow_only`, and `Motion.build` for `scale_bone`. Map our idle/walk/run to upstream acts (run as a speed-picked walk act), and sit to the new registered pose's act. | high |
| crates/engine/src/client/terrain_gpu.rs | 8 | `describe_lod` on LOD threads (`LodDesc` OnceLock), steep-face fall (`FACE_FALL`), scatter thinning masks (`postfix` with `Mask`), sample runs on real ground (`RUN_SPOTS`), `custom_heights` per cube, smooth flag in jobs. | `1b8b8fd2`: features and fields become expression steps (`expr as ek`), mod "custom" features and masks removed from the GPU path, `describe_lod` returns `Vec<i32>`, `ScatterBlock.offset` and `pos` take `.into()`. | Take upstream's expression description as the base. Rebuild our scatter thinning as expression steps, not `Mask` postfix. Keep the async `LodDesc`, `FACE_FALL`/`CONT_SCALE`, `RUN_SPOTS` and the smooth job word. Drop `custom_heights` only if our worldgen features (rivers, lakes, continents) all express as fields; otherwise keep a CPU fallback. | high |
| crates/engine/src/client/lod.rs | 6 | Smooth morphing far levels (`87aa8332`, `TileKey::smooth`), steep faces drawn as wall bands (`fall`, `BANDS`, `FILLER`, `tops`/`walls`), box-model face textures (`face_of`), custom heights on `pool()` with a `live` flag. | PR #52 "smoother LODs": bilinear `corner_y`/`sample_y` and a separate `y_at`; `face_paint(..).base_tinted` for tint; custom heights removed (jobs sent straight). | Keep our smooth tiles and faces, a superset of upstream's interpolation (decision: this departs from the "take upstream" rule). Take upstream's `face_paint` tint inside our `face_of`. Follow `terrain_gpu`'s choice on custom heights. Keep `live` cancelling. | high |
| crates/engine/src/client/textures.rs | 5 | `Source::Look`: a stack's `core:look` composed at runtime (`looks`, `looks_full`). | `Source::Layers(Vec<(String,u32)>)` with a `layered` map, for stacks' `layers` (layered item icons), the same idea. | Take upstream's `Layers`/`layered` and drop `Source::Look`. Port our parts tools from `core:look` to `layers`, keeping a `core:look` read fallback for saved stacks. | medium |
| crates/engine/src/client/receive.rs | 3 | `Time { rate }` and `day_rate`; `Air`, `Tints`, `Weather { raining, thunder, only }` arms; `Flash`; `ClientEntity` literal with `held_look`, `scale`, `mount`, `seat_forward`. | `BodyGlow` arm, `since_time = 0.0`, `Using((ticks, off))`, `Weather(id)`, `Flash` gone (entity effects), `ClientEntity::new(..)`. | Keep `rate` and `day_rate` with f64 `since_time`, plus all arms: `BodyGlow`, upstream's `Using`, `Air`, `Tints` and `Weather { id, only }`. Drop `Flash`. Use `ClientEntity::new` with our fields given defaults inside it. | medium |
| crates/engine/src/client/modui.rs | 2 | `uk::ENTITY` with text "menu" shows the menu's own entity (data 1); `client_raining` asks `rains_at` the player's biome. | `uk::ENTITY` renamed `uk::PLAYER_MODEL`; `client_raining` replaced by `client_weather` (weather id). | Use `uk::PLAYER_MODEL` with our `data: (text=="menu")`. Keep upstream's `client_weather` and keep `client_raining` as a second host entry (falls and `rains_at`). | low |
| crates/engine/src/client/sky.rs | 1 | `Sky::update` gets `rain, thunder, wind, dt`: clouds drift with the wind, and pack weather `cover` comes from rain and thunder. | `rain, thunder` become `look: &WeatherLook`. | Signature `look, wind, dt`. Derive our pack `cover` and `cloud_shade` from `look.falls` and `look.cloud_dark`; the merged body still reads `rain`/`thunder` at lines 460-471. | medium |
| crates/engine/src/ui/render.rs | 5 | Font metrics `g.bearing`, `font.top`, `font.height` (smooth fonts, count drawn 1.3x); `slot_draw`; `stack_item_icon` from a stack's look; `icon()` helper; `max_durability`. | `font::ABOVE`/`CELL` (Latin diacritics); layered icons via `stack::layers`; liquid items tinted, counts shown in B/mB; `model_icon` for 3D items; `c.durability`. | Keep our font metrics, folding `ABOVE` into `font.top`. Keep `slot_draw`. Take upstream's layers branch, liquid tint, mB counts and `model_icon`, with `stack_item_icon` reduced to layers. Use `c.durability`. | medium |
| crates/engine/src/hud.rs | 1 | `c.max_durability(&stack)` (a parts tool's own durability). | `c.durability(&stack)`: stack value over item value, the same thing. | Take upstream's `durability`. | low |
| crates/engine/src/app.rs | 7 | `reload_assets` became a `Running` method; `content_changed`, `poll_join`, `enter_remote`; async join with portals/travel and home return; reload waits for join and first terrain (15 s cap); `world_info` split (render scale); shadows before the world pass; `copy_behind`; `diag::span`s; `Cmd::Mark` GPU parts. | `grab_cursor`/`center_cursor`; `read_avatar` passed to `Client::new`; reload waits for the join; `r.quads_dirty = true` on reload; `world_list` built with `render(.., &mut world, &mut r.list, ..)`; `occlusion.take_depth` after the pass. | Keep our functions and add upstream's two cursor helpers. Pass `read_avatar` in `enter_remote`'s `Client::new`. Add `quads_dirty` to our method. Order the frame as preload, `render(world, early=r.list)`, `render_shadows`, `BeginPass`, world with `copy_behind`, `EndPass`, then `take_depth`, keeping both `Mark`s. | high |
| crates/engine/src/gpu_layout.rs | 4 | Terrain header `FACE_FALL` 30, `CONT_SCALE` 31, `LEN` 32; `SCATTER` 8, `SCATTER_BIOME` 6 (thinned), `CUSTOM`; UI `tex` word's smooth-font flag; `lod_corner_compiles` test. | Header renumbered (`FIELDS`, `FIELD_OFF`..`SCATTER_OFF` 22, `LEN` 23), `STEP`/`FIELD`/`COND` words, `CUSTOM` and `MASK` gone; `ui_strip_record` animation word. | Take upstream's numbering and append ours after `SCATTER_OFF`: `FACE_FALL` 23, `CONT_SCALE` 24, `LEN` 25. Keep `SCATTER`/`SCATTER_BIOME` 6 and drop `CUSTOM` per terrain_gpu. Fit the smooth-font flag into a free bit of `ui_strip_record`. Keep our test. | high |

### Engine server and shared

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| crates/engine/src/server/actions.rs | 8 | `Join` with account vouch, DID check, achievement proofs (`Proofs`, `AskAchievement`, `waiting`); probation (`may`/`barred`, `Did::Barred`, `tell_barred`); `keys`, seated riders don't fall; `mining_time(&held)`; `leave()` as a method with a last message; `keep_broken` wear. | Server-side mining progress (`Mining { frame, pos, part, progress }`, `aimed_part`, `rate`, `no_break`); `SwapHands`; `Move` with `jumped` and `physics::around`, `splashes`, `fall_effects`; `Use` rewritten (no `Did`, block first then hands, `block_reach`); `Show`/`PlayerGone` on leave; `stack::worn`. | Take upstream's mining, Use and Move bodies. Re-add our Join/Proofs arms; the probation check in StartBreak and Use (place only if not barred); `keys` plus a seated `on_ground`. Keep our `leave()` method with upstream's `Show`+`PlayerGone` inside. Move `keep_broken` into `stack::worn`. | high |
| crates/engine/src/server/mod.rs | 5 | `mod achievements; mod admission;`; `run_day_clock()`; `SetAir`, `SetTints`, `SetRain` events; `Air` sent on dimension change; death keys `freeze`, `heat`, `poison`, `bleed`. | `pub(crate) mod actions`; `has_weather()`, `tick_weather(c)`; `SetWeather { weather }`; `Flash` removed; `EntitySwing { off }`; death messages from registered `damage_causes`. | Keep our modules with `pub(crate) mod actions`, `run_day_clock` plus upstream's weather tick, all event arms (`SetRain` on the new weather ids), and upstream's death messages. Register our four causes as damage causes with `death.freeze`.. keys. | medium |
| crates/engine/src/server/entity.rs | 4 | Tick-skipping fields (`moved_at`, `hurt_at`, `resting`, `ticked_at`, `steering`); `dims()` times `self.scale`; no gravity while flying (`fly`); no fall damage while flying. | `sent` gets a `bool`; `dims()` returns `physics::Dims` from `EntitySize`; `notice_hand`; `STEP_UP`, `gravity_per_tick`, `e.falls`; fall check by `physics::around`/`same_around`. | Keep both field sets and upstream's `dims` with the size scaled by `self.scale`. Gravity is 0 when (`NO_GRAVITY` && !`falls`) or `fly`. Add `|| fly.is_some()` to upstream's `here` check. | medium |
| crates/engine/src/server/regions.rs | 2 | `Dim` weather by place: `rain_in`, `manual_until`, `set_rain`, `set_manual_weather`, `rain_here`, `set_air`, `set_tints`; `tick_weather` defers to a mod. | `raining`/`thunder` become `weather: u16` (registered weathers, weights and durations); `tick_weather(c)` emits `WEATHER_TICK`. | Take upstream's `weather` id. Re-express `set_rain(biomes, thunder)` as "weather id X only in biomes"; keep `set_air`/`set_tints`/`manual_until`, and a mod's rain still overrides upstream's random pick. | high |
| crates/engine/src/server/path.rs | 2 | Held liquids count (`liquid_in()`) for free cells, burning, ground and wet. | Cells' `main` + `extras` checked; `liquids[..].burns()`; `block.liquid(c, p)`. | Take upstream's extras form and `burns()`. Use `liquid_in()` on each part only if our `holds` survives the waterlogging decision. | medium |
| crates/engine/src/server/area.rs | 1 | `wet`: a block standing in a kept liquid leaves the liquid. | `keep` becomes `self.keeps(c, s)`, radius check `self.covers(cp)`. | Take upstream's helpers and add `|| wet(s)` to the `any`, if `holds` survives. | low |
| crates/engine/src/registry.rs | 11 | `skin_painters`; `MOUNT`, `HELD_LOOK` field components (7); sound `priority`, `voices`; `command_permissions` with `ModId`; packages' settings sorted last (`from_packages`). | `chat_format`; `carried` components; `OFF_HELD`, `USING`, `SWAPS`, `LOOPING_ACTION`, `EFFECT`, `FORCED_POSE` (11); sound `full_within`, `falloff`; `LiquidChange` renamed `LiquidEdit`; `entity_watchers`, `attached_models`, `item_models`. | Keep both everywhere: 13 field components `[HEALTH..FORCED_POSE, MOUNT, HELD_LOOK]`, all sound fields, `(ModId, String, u32)` permissions with `LiquidEdit`, and both sets of local collections. | medium |
| crates/engine/src/physics.rs | 3 | `posed(pose, h, eye)` with `Pose::Sit` (1.15/0.97); `in_liquid` and `liquid_at` count held liquids (`liquid_in`); `drag_of` (held liquid's drag). | `posed(size, pose) -> Dims` by `PoseDef`; `Cells` trait (main + extras, `liquid()`); `Around`, `around`, `body_level`, `same_around`; walk reads `around`. | Take upstream's form. Sit becomes a registered `core:sit` pose (height 1.15/1.8, eye 0.97/1.62). Make `Cells::liquid` use `liquid_in()` and the walker's drag use `drag_of`, if `holds` survives. | high |
| crates/engine/src/saves.rs | 2 | `key_index`, `key_generation`, `created` in world meta; a `kept` id map when writing cubes. | Weather saved as `weather.{d} = {id} {until}`; cube palettes include `all_extras()`. | Write both: our key lines and the new weather line. Chain extras into the palette loop with our `kept` lookup. | medium |
| crates/engine/src/world/mod.rs | 1 | `Pose::Sit` (5) in the enum and `from_u8`. | `Pose` becomes `Pose(u16)` consts plus `from_id` and `def()` from the registry. | Take upstream's form and register `SIT` (abi `pose::SIT = 5`) in `engine_poses`. | medium |
| crates/engine/src/components.rs | 1 | `KEEP_BROKEN` component. | `USE_PRIORITY` component. | Keep both. | low |

### Net

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| crates/engine/src/net/mod.rs | 3 | ToClient `Weather { raining, thunder, only }`, `Air`, `Tints`, `Refused`, `Achievement`, `AchievementAnchor`, `AskAchievement`, `Revocations`; ToServer `Move.keys`; protocol 12 in p2p.rs. | `Using(Option<(u32,bool)>)`, `Weather(u32)`, `Flash` removed, `Avatar`, `PlayerGone`; `Move.jumped`; `StartBreak.part`; plus outside the hunks `Show`, `BodyGlow`, `Knock`, `Velocity`, `SetVelocity`, `Look`, `Flying`, `Selected`, `TickRate`, `SwapHands`, `Blocks.extras`; protocol still 2. | Keep every message of both. Weather becomes `Weather { id: u32, only: Option<Vec<u32>> }`. `Move` carries both `jumped` and `keys`; `StartBreak { pos, part, frame }`. The merged `PROTOCOL` in p2p.rs must be **13** (above our 12 and upstream's 2), since both sides' wire formats changed. | high |

### ABI and SDK

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| crates/sdk/src/extend.rs | 1 | `question::NATURAL_SPAWN`. | Questions with an event after them move to `actions::*` (`Action<T>` pairs question and event); `question` keeps only `ATTACK`. | Take upstream and add `NATURAL_SPAWN` to `question` as `Question::named(crate::abi::question::NATURAL_SPAWN)`. | low |
| crates/sdk/src/client.rs | 1 | On `World`: `raining_at`, `set_rain`, `set_air`, `set_biome_tints`, `content_tint`, `air_at`, `set_day_rate`. | All those `World` methods move out of client.rs into `game.rs` (`set_weather(impl Into<Id<Weather>>)`, `teleport(impl Into<Vec3>)`, `open_menu(.., Holder)`); `s()` renamed `abi_str()`. | Take upstream here, which deletes the block. Move our seven methods into `game.rs` next to upstream's, with `impl Into<BlockPos>` and the weather id. | medium |

### Base mod

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| mods/base/src/player.rs | 8 | Hunger removed for the vitals; clothing slots (hat, coat, trousers, shoes, cloak); HUD armor and air top-left from `VITALS`; window tabs, recipe book and parts panel; `survival::crafted`; crafting XP. | Renames: `actions::USE_ITEM/BREAK_BLOCK/HURT`, `icons(..).shaking()/.from_right()`, `outside_survival`, `player_state`, `S::column`, `recipe_grid_output`, `consume_one`, `damage_cause::`, `w.heal`, `attribute_add_modifier`, `block_pos_of`, `st.eye()`; adds `OFF_HAND` slot. | Keep ours (hunger stays deleted) and add `r.equipment_slot(&OFF_HAND, &[], "slot_offhand")`. Apply upstream's renames to our surviving code (`S::column`, `outside_survival`, `recipe_grid_output`, `consume_one`). | medium |
| mods/base/src/mobs.rs | 3 | Campfire ring keeps monsters off (`keep_from_fire`, `fire_for`, `CAMPFIRES`); explosions keep held liquids (`held_liquid`). | AI triggers (`.aggressive(16.0)`, `hostile(goals)`); `fill(Shape::blast(..)).keep(Filter::..)`; `ai::melee()`, `chase_target`; `block_flags`, `block_pos_of`. | Take upstream's AI and fill API, and keep `keep_from_fire` first in creeper and hostile goals. Add our `held_liquid` test to the blast condition. Port our `[f64;3]` positions to `Vec3`/`BlockPos`. | medium |
| mods/base/src/lib.rs | 1 | `.seasons(YEAR_DAYS, FOUR_SWING).set(&SEASONS, FOUR).weather()`. | `.weathers(&[&weathers::RAIN, &weathers::THUNDER])`. | `.seasons(..).set(..).weathers(&[&weathers::RAIN, &weathers::THUNDER])`. | low |

### Testkit

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| mods/testkit/src/tests/gameplay.rs | 10 | Hotbar found by `t.hotbar(0)` (equipment shifted it to 41-49); tests for the vitals (`in_stomach`, `max_health`, `every_food_has_nutrition`); `own_names`, `creative_sword`, `advancements_by_tags`, `air_in_creative`, `swimming_no_bob`, `toast_with_the_gui`, `spawn_eggs` kept; deop check; join by p2p hosting address. | Renames `input_key`, `select_slot`, `set_look`, `say`, `world_join`, `world_leave`; `cell_parts` grows slab water, stair-shape and piston steps; "remove useless tests" deleted the tests listed under ours. | Keep our tests and upstream's new `cell_parts` steps, both halves of hunk 2 in sequence. Apply the client renames. Join by `world_join(&hosting_address(..))`. Keep our deop lines with `world_leave()`. | medium |
| mods/testkit/src/server.rs | 3 | `players` and `setprop` ops kept (with `pings`); `feet` op (felling); crystal check by `Option`. | `pings` and `setprop` gone; `w.get` renamed `w.block`; adds `crowd` and `swap` ops. | Keep every op: ours with `w.block`, `setprop` through `w.prop_set`, plus `crowd` and `swap`. | low |
| mods/testkit/src/script.rs | 2 | `entities` script command; `chatf` (chat a file's text). | Doc text for `rec`; `set_look`, `say`; `view`, `vsync`, `occlusion` commands. | Keep both, using `say` and `set_look` in ours, and our `rec` doc line naming tools/rec.ps1. | low |
| mods/testkit/src/tests/bodies.rs | 1 | Health checked by `super::full` (vitals scale). | `me.y` (Vec3). | Ours with `me.y`. | low |
| mods/testkit/src/runner.rs | 1 | `let sent = Instant::now();` before chatting. | `chat` renamed `say`. | `let sent = Instant::now(); self.c().say(line);` | low |

### Docs

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| docs/modding.md | 4 | "Far from players" note; riding (`mount`, `keys`); `set_scale`; `dropped_item`; world rules on Create New World; skins with `scale_bone`. | Rewritten for actions, triggers and reactions, damage causes, shapes, conditions, poses, game modes, storages, GPU particle rules, sound falloff, `client_entity_cosmetics`, `st.pos.y`. | Take upstream's text and re-add our paragraphs in upstream's names (`actions::`, `on_died`, `st.eye()`, `client_entity_cosmetics`). | medium |
| docs/worldgen.md | 1 | `seasons(days, day_swing)` bullet. | `daylight(&[(turn, level)])` bullet. | Keep both bullets. | low |
| docs/sky.md | 1 | `seasons` in the daylight sentence. | `daylight` curve in the same sentence. | List all three: `day_length`, `fixed_time`, `seasons`, `daylight`. | low |
| docs/machines.md | 1 | `recipes(&KIND)` and `Recipe::at(..)` (recipe book). | Liquids are items (no `fluid:` syntax), liquid slots. | Take upstream's paragraph and keep our recipe-book line after it. | low |
| README.md | 1 | Removed the Minecraft resource pack import. | Explains `MINECRAFT_DIR_LOOKUP`. | Keep ours ("Weft ships with its own textures."). | low |

### Lang (en_us, ru_ru; fr_fr is in the other half)

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| crates/engine/assets/lang/en_us.lang | 2 | `command.sounds.*` and 166 other engine keys (172 in the hunks). | `command.title.done`, `chat.screenshot` (now `{0}, {1}`), `.picture`, `.folder`. | Keep every key of both: no key is in both hunks, so take the union. | low |
| crates/engine/assets/lang/ru_ru.lang | 2 | 273 keys in the hunks. | The same 4 keys in Russian. | Union. Note that `mods.title` is already in HEAD twice (lines 145, 227): rename one. | low |

### Shaders

| file | hunks | ours (what we changed) | upstream (what they changed) | resolve | risk |
|---|---|---|---|---|---|
| crates/engine/assets/shaders/ui.wgsl | 3 | `sharp_uv`: sharp bilinear icons at any GUI scale (`linear_clamp`). | `strip_pixel`: animated strips blended between frames (`uv_next`, `blend`). | Keep both functions: `strip_pixel` samples through `sharp_uv` with `linear_clamp` for both frames, used for icons and the default path. | medium |

### Follow-ups outside the conflicts

Renamed APIs to chase (compile errors in clean-merged files, mostly our mods):
- **Arrays become structs** (`9ea027e2`, `2d028b6c`, `a7a3d5d8`): `EntityState.pos` is `Vec3` and `.size` is `EntitySize`, with no `Index` impl, so every `st.pos[1]` and `size[2]` breaks: about 163 `pos[n]` and 38 `size[n]` in mods/ (base, animals, packages). Use `.x/.y/.z`, `.eye`, `st.eye()`, `block_pos_of`. `[i32;3]` still converts into `BlockPos` (`From`), but results come back as `BlockPos`.
- **Questions and events become `actions::*`**: `question::{BREAK_BLOCK, PLACE_BLOCK, USE_BLOCK, ENTER_DIMENSION, PICKUP_ITEM, HURT, INTERACT_ENTITY, USE_ITEM}` (45 uses) and events `BLOCK_BROKEN`, `BLOCK_PLACED`, `ENTITY_DIED`, `ITEM_PICKED_UP`, `ENTITY_HURT`, `ITEM_USED` (41 uses in mods). `mods/packages/src/sandboxed.rs` maps sandbox questions to these, so it needs the same change.
- **SDK World renames**: `w.get` to `w.block` (80), `w.set` to `set_block`, `schedule` to `schedule_tick`, `add_modifier`/`remove_modifier` to `attribute_*` (9), `after` to `emit_after`, `compute`/`explain` to `chain_compute`/`chain_explain`, `explain_attribute` to `attribute_explain`, `loot_modify` to `loot_edit`, `replace`/`remove` on loot to `*_item`, `fill_sphere*` to `fill(Shape::ball/blast).keep(Filter::..)`, `position_seed`/`cube_seed` to `seed_at`/`seed_of_cube`, `near` to `entities_near`, `on_death` to `on_died`, `client_entity_tick` to `client_entity_cosmetics`, `engine_effects` to `run_engine_effects`.
- **UI and client**: `S::col` to `S::column` (73), `icons_rtl`/`icons_shaking` to `icons().from_right()/.shaking()`, `c.me()` to `player_state()`. Testkit client `chat`/`select`/`look`/`key`/`join`/`leave_world` become `say`/`select_slot`/`set_look`/`input_key`/`world_join`/`world_leave` (151 calls).
- **Gameplay helpers**: `recipe_match` to `recipe_grid_output`, `use_one` to `consume_one`, `damage::X` to `damage_cause::X`, `Content::max_durability` to `durability` (5 callers), `LiquidChange` to `LiquidEdit`.
- **Creative**: `w.creative(id)` still exists, but upstream's own code moved to `outside_survival` (spectator included). Our 36 `creative()` checks for vitals and hunger-like costs should probably follow.
- **Weather is now registered values** (`weathers(&[..])`, `Weather` ids, `WeatherLook`). Our weather-by-place stack needs rewriting onto it:
  - host `set_weather`/`set_rain`, `DimEvent::SetWeather`/`SetRain`, client `weather.rs` (`rain_in`, `rains_at`);
  - the base vitals and seasons code calling `set_rain`/`raining_at`;
  - shader-pack `scene.weather`.
- **Sit pose**: `Pose` is now registered (`PoseDef`). Our riding `Sit` must be registered as `core:sit`, and our 19 `Pose::Sit`/`Pose::Stand`/`from_u8` uses become consts and `from_id`.
- **Damage causes are registered**: `FREEZE`, `HEAT`, `POISON`, `BLEED` (abi `damage` consts, our vitals) need `r.damage_cause(..).death_message("death.freeze")` or the engine-side registration, or deaths read "death.generic".
- **Flash**: `ToClient::Flash`, `DimEvent::Flash` and host `entity_flash` are gone; `w.flash` now goes through `entity_effect(FLASH)`. Our `e.flash` field and its renderer use go.
- **Toggle sneak/sprint settings** were removed upstream (per-binding toggle and hold instead). `crates/engine/src/app.rs:2633-2634` (outside the hunks) still sets `r.input.toggle_sneak/sprint`, and `client/controls.rs` is deleted upstream.
- **Waterlogging, a decision for Ronald or Planner**: upstream re-implemented it (bf `HOLDS_LIQUID` 4096, SDK `.holds_liquid()` with no argument, the liquid as an extra part in the cell). Ours is `StateInfo::holds`, `liquid_in()`, `.holds_liquid(&WATER)` and `drag_of` (`c1ee7fe0`). The SDK names clash.
  - By the rule we take upstream's version and port `mods/base/src/wild.rs` (kelp, eelgrass), the testmod's `sea_slab`/`waterweed` and the `held_water_*` testkit tests to it. Then `liquid_in()` and `holds` go from physics.rs, path.rs, area.rs, client/mod.rs and actions.rs.
  - If we keep ours, every extras-aware helper needs `liquid_in`.
- **Tools from parts**: upstream added stack values over item values, parts in stack storages and `layers` icons (`10fdced5`), overlapping our `core:look`, `keep_broken` and `max_durability`. Port base `parts.rs` and crafting to upstream's stack storages and `layers`. Keep a loader for saved stacks that carry `core:look`.
- **GPU worldgen rewrite** (`1b8b8fd2`):
  - `mods/base/src/worldgen.rs` and our noise features (rivers, lakes, continent scale, mesas, sequoia scatter heights) must be re-expressed as SDK expression fields.
  - `Terrain::fields(seed)` and `Mask` are gone in upstream's sdk/terrain.rs (other half), and the server `terrain.rs` changed by 666 lines.
  - The memory note's SCATTER_UP 32 and LOD scatter changes need re-checking there.
- **Calendar and daylight**: our `seasons(days, swing)` and upstream's `daylight(&[(turn, level)])` both shape daylight. Decide the composition (seasons move sunrise and sunset, the curve maps turn to level) in `content.rs`/`server/dim.rs` day functions.
- **ClientEntity::new** (upstream) must initialize our added fields (`held_look`, `scale`, `mount`, `seat_forward`). The local player is now `self.me`, so our `self_entity()` callers (shadows, hand light) switch to it.
- **Saves**: worlds saved by chiacraft write `weather.{d} = r t until` (three numbers). Upstream's loader reads `{id} {until}`, so add a fallback for the old form.
- **Behaviour changes that still compile**:
  - mining is now timed on the server (`Mining.progress`), so our `mining_time(&held)` with parts tools must feed `mining_rate`;
  - Move's fall damage now goes through `physics::around` (seated riders must still not fall);
  - `durability()` now falls back to the item's own value, so check repair and keep-broken at 0.
- **Tests to rerun on the PC** (Tester 2, after the merge builds):
  - testkit `gameplay` (`cell_parts`, `own_names`, `creative_sword`, `advancements_by_tags`, `air_in_creative`, `swimming_no_bob`, `toast_with_the_gui`, `spawn_eggs`, eating and vitals, `dedicated_server` join and deop), `bodies` (lift ride);
  - riding, `held_water_*`, felling (`feet`), explosions;
  - LOD and shadow looks (far terrain faces, smooth tiles, sun and moon shadows), weather and fog look shots, sky and clouds drift;
  - `cargo test -p weft-engine gpu_layout` (header and records, `lod_corner_compiles`), the unit tests in physics/liquid/models (avatar limits), and the vitals sheet regeneration.

### Riskiest files

1. `client/terrain_gpu.rs`, `client/lod.rs`, `gpu_layout.rs`: upstream rewrote GPU worldgen as expression fields and dropped masks and custom heights, all of which our rivers, continents, scatter thinning, steep faces and async LOD build on.
2. `client/models.rs`: upstream's clip, act and stock-animation system replaces the idle/walk slots our shared geometry, run clip, sit pose, shadow batch and `scale_bone` builds extend.
3. `client/mod.rs` and `app.rs`: the local player becomes `self.me`, `render` gets an `early` list plus occlusion, and weather becomes `WeatherLook`, under our shadows, riding, air/fog/wind and async joins.
4. `server/actions.rs` and `server/regions.rs`: upstream's server-side mining, the Use/Move rewrite and registered weathers cut across our identity/DID/achievement joins, probation and rain-by-biome weather.
5. `net/mod.rs`: keep every message of both, `Weather { id, only }`, and protocol 13. Waterlogging (physics.rs, path.rs) is a decision first: upstream re-implemented our `holds_liquid`.
