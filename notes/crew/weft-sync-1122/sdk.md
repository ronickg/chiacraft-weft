# Group sdk: merge report

All files below have no conflict markers left. Nothing was built, checked or run (no cargo, no
git writes). 11 `// SYNC:` comments.

## Per file

- **crates/abi/src/lib.rs**: Upstream's rework is the base: entries, `comp::<kind>` components, tags, conditions, chain questions and `Modifier` steps. Every old descriptor is gone (`SoundDesc`, `ParticleDesc`, `DimensionDesc`, `ScatterDesc`, `PoseDesc`, `MusicDesc`, `TerrainDesc`, `terrain_op`, `FeatureDesc`, `entity_flags`, `particle_flags`, `game_mode_flags`, `client_held_item`). Ours that stay, as components: `comp::sound::PRIORITY/VOICES`, `comp::particle::AIR/SINK/FLUTTER/LIFT/STREAK` and tags `FLOATS/BLOCK_TINT`, `comp::terrain::{CONTINENT_SCALE, FACES, FACE_FALL, FAR_LEAVES, CPU_HEIGHT}`, `comp::dimension::{YEAR_DAYS, DAY_SWING}`, `comp::entity::ALWAYS_ACTIVE`, `comp::feature::{THIN, THIN_KEEP, THIN_BIOMES}`. Ours that stay as they were: GRIP, LOOK, WORN_LOOK, KEEP_BROKEN, FIRST_PERSON_HELD_SCALE, RecipeInfo width/height + RecipeInput, SoundStart/SoundChange, set_rain/raining_at, set_air/air_at, body_turn, sky events, set_day_rate, SkinArgs/BoneScale, FeatureArgs/FaceArgs/FarLeavesArgs, client_image_set, MENU_SORT=17 upstream + our REBUILD_SCREEN=18. Conditions `core:climbing/gliding/rope` = 25..27. Question NATURAL_SPAWN comes after CRIT/SWEEP/RECOVERING. Attributes: upstream's list plus WALL_CLIMB_SPEED, GLIDE_*, WIND_*, LIFT, ROPE_*. HOLDS_LIQUID is dropped (let go).
  SYNC lines: 1129 (feature THIN), 1431 (YEAR_DAYS/DAY_SWING), 4789 (set_rain), 4801 (set_air), 5269 (pose conditions), 5722 (NATURAL_SPAWN as chain), 5934 (WALL_CLIMB_SPEED).
- **crates/engine/src/content.rs**: Upstream's freeze (entries, `blocks::freeze`, features, Interner) is the base. Kept: `ModMeta.package` (beside upstream's revision/after/before/source), Clone on `RawRegistrar`/`ModMeta`/`BiomeDef`, StateInfo `grip` (beside upstream's sound_absorption), year_days/day_swing + `daylight_clock`, `DimensionDef::fields(seed)`, `custom_cache` (now a `weft_noise::cache::ColumnCache`), `Ids.look/worn_look/keep_broken`, `broken`/`max_durability`/broken tool in `harvests`, `source_of`, `state_names`, our switchable `CONTENT` (AtomicPtr + per-thread, `set(&'static)`), `check_texture_files` (ported to `Interner`, called after `number_items`), item icon = model when it has no picture, items of packages numbered last, package command clash + permission owner rule and its test, and the "block sounds:" log (now reads `effects[..].sound`).
  Dropped: holds-liquid (`liquid_in`), ScatterDef/ScatterThin, all old Raw* and number_blocks code. SYNC 818 (scatter thinning has no home on features yet), 2563 (block ids of packages: `blocks::freeze` must sort packages last).
- **crates/engine/src/registry.rs**: Upstream is the base (entries baked by `Tables`, `FieldComps`, `engine_components`, `engine_attributes`). Ours on it: `FieldComps.held_look/worn_looks` with HELD_LOOK/WORN_LOOKS in `engine_components`; `attr` ids 33..43 and `engine_attributes` rows for wall climb, glide, wind, lift and rope (old ranges); `RawGame.package` and `skin_painters`, owner in `command_permissions`; package configs and keys sorted last via `bake_by`; `Game.config_cells` filled while configs bake; entity send/activation/interval config ids; sound priority/voices baked from components (click 100/2, pickup voices 4); particle air/sink/flutter/lift/streak from components. Max health 100000 was let go (upstream's 1024).
- **crates/engine/src/modloader.rs**: Upstream's discover (`DiscoveredMod`, `paths::libraries`), rejected-mod errors (`load_error`), ABI_VERSION and also_off are in. The signature is `load(mod_dirs, package_dirs, cache, off_file, also_off, core_assets)`. Our package Loader stays: it now keeps `step_order` (upstream's `step_order.txt`) and `core_assets`, `build()` calls the 4-argument `content::freeze`, and package ModMetas carry `source` = their store folder. Names go through our `set_names` (our RwLock NAMES). Every mod is logged as loaded.
- **crates/engine/src/stack.rs**: upstream's `embedded` and `layers`, then our own look as a fallback (`own_look`, renamed from `look`), `worn_look`, `is_look`, `with_name`, keep_broken wear.
- **crates/engine/src/components.rs**: upstream's, plus KEEP_BROKEN, GRIP, FIRST_PERSON_HELD_SCALE, LOOK/WORN_LOOK. HOLDS_LIQUID is gone.
- **crates/engine/src/loot.rs**: upstream's (identical).
- **crates/sdk/src/ai.rs**: upstream's outright.
- **crates/sdk/src/lib.rs**: upstream's, plus `air_flags`, `held_keys` and `BiomeAir` re-exports.
- **crates/sdk/src/client.rs**: upstream's, plus rebuild_screen, set_image, screen_text_wrapped/plated and BiomeAir.
- **crates/sdk/src/effects.rs**: particle and sound builders set our components through a private `mod chia` of `Component` statics. SYNC 36: these belong in comps.rs. `Client::start_sound` + SoundHandle kept. Play sounds through upstream's `Output::play_sound`.
- **crates/sdk/src/extend.rs**: upstream's attributes/Modifier, plus our attributes and the NATURAL_SPAWN question.
- **crates/sdk/src/component.rs**: upstream's matchers, plus FIRST_PERSON_HELD_SCALE, GRIP, `Tag::at` on `World::parts`, stack_look/with_look/with_worn_look.
- **crates/sdk/src/content.rs**: `sound_group_on` is now an edit, `sounds_on_entities` is new, `seasons()` uses local components (SYNC 3210: belongs in comps.rs), `grip()` is `self.set`. keeps_broken is kept.
- **crates/sdk/src/game.rs**: Upstream's World API (walk(Walk), raycast(Ray), RecipeView). Ours on it: tick_elapsed, rain/air/sky events/biome tints/day rate, set_scale, ignite, dropped_item, entity_skin; `recipes()`/`RecipeView::at`/shape/inputs/accepts on upstream's RecipeView, `hidden()` via the recipe HIDDEN tag, `always_active()` as a tag.
- **docs/modding.md**: upstream's index page. Our SDK text moved into a closing section, "chiacraft's additions": stacks and looks, set_scale, entity skins, far-from-players settings, movement attributes, sound handles and voices, and create-world rules. The holds-liquid text is gone. It could move into upstream's detail pages later; those aren't my files.
- **docs/ui.md**: upstream's `menu_open`, plus our rebuild_screen sentence and our corner note for the HUD.
- **docs/redstone.md**: upstream's bodies/ships list, plus our `body_turn` line.
- **docs/machines.md**: deleted upstream and handled by the lead (now docs/block_entities.md). I fixed our line there: `Recipe::at` -> `RecipeView::at`.

## Needs in files that aren't mine

- blocks.rs: StateInfo literals (around lines 900, 970) need `grip: 1.0`. `blocks::freeze` should sort a package's blocks after code mods' (SYNC content.rs:2563).
- tag_bits.rs: `particle` tags need `FLOATS` and `BLOCK_TINT` (abi `comp::particle::FLOATS/BLOCK_TINT`). gpu_layout.rs and client/particles.rs read `pf::FLOATS/BLOCK_TINT`.
- features.rs / sdk feature builder: scatter thinning (comp::feature::THIN*) is unported.
- host.rs: `client_held_item` is gone from abi Host. `reg_command_permission` must push the owner. `reg_entity_skin` pushes to `RawGame.skin_painters` (kept).
- host.rs, server/area.rs, client/world_render.rs still call `StateInfo::liquid_in()` (holds-liquid, dropped). Use `.liquid`.
- crates/bots/src/main.rs:201 and engine lib.rs:79 call the old `modloader::load` (bots with 4 args, lib.rs without package_dirs).
- Something must evaluate pose conditions 25..27 and ask NATURAL_SPAWN as a chain. The server has to read `comp::entity::ALWAYS_ACTIVE` as a tag.
- mods/animals/src/voices.rs uses `sounds_on(Target::Entity(..))`, which is now `sounds_on_entities(kinds, prefix)`.
- docs/details/camera.md says the free camera key is P; ours was F6 (input.rs decides).

## For Ronald

- Upstream content: ships and bodies with physics (thrusters, body mounts, helm/controller, flooding, collision events), sound absorption per block, sound environments (lowpass, liquids muffle), thunder sounds, spawn eggs drawn generically when a mod has no picture, an in-game editor, logic gates, rejected mods shown with their reason in the mods screen, and a `step_order.txt` for modifier steps.
- Lost or waiting on our side: scatter thinning (groves and glades far forests copy) has no home on upstream's features yet. Waterlogged blocks (holds_liquid) are gone, as the ledger said. Max health is capped at 1024 again, which matters for later bosses. Mods must use `core:wall_climb_speed`, because `core:climb_speed` is now upstream's ladder multiplier.
- Look: nothing changed by this group except item icons (unchanged rule) and particles' wind (same numbers, read from components).

## SDK map (our addition -> on upstream's API)

| Ours before | Now |
|---|---|
| `attribute::CLIMB_SPEED` "core:climb_speed" (wall climbing) | `attribute::WALL_CLIMB_SPEED` "core:wall_climb_speed" (SYNC); upstream's CLIMB_SPEED is a ladder multiplier |
| GLIDE_SPEED, GLIDE_SINK, WIND_X/Z, LIFT, ROPE_* attributes | unchanged names; engine `registry::attr` ids 33..43 |
| `pose_when::CLIMBING/GLIDING/ROPE` | conditions `core:climbing`, `core:gliding`, `core:rope` (abi::condition 25..27, SYNC) |
| NATURAL_SPAWN question with verdicts | chain question answered yes/no (SYNC) |
| `entity_flags::ALWAYS_ACTIVE` | tag `comp::entity::ALWAYS_ACTIVE`; `.always_active()` unchanged |
| ParticleDesc air/sink/flutter/lift/streak, flags FLOATS/BLOCK_TINT | `comp::particle::AIR/SINK/FLUTTER/LIFT/STREAK`, tags `FLOATS`, `BLOCK_TINT`; builder methods unchanged (SYNC: belong in comps.rs) |
| SoundDesc priority/voices | `comp::sound::PRIORITY/VOICES`; `.priority()`, `.voices()` unchanged |
| DimensionDesc year_days/day_swing | `comp::dimension::YEAR_DAYS/DAY_SWING`; `.seasons(days, swing)` unchanged (SYNC) |
| ScatterDesc thin/thin_keep/thin_biomes | `comp::feature::THIN/THIN_KEEP/THIN_BIOMES`; not applied yet (SYNC) |
| terrain_op FACES/CONTINENT_SCALE/FAR_LEAVES, FeatureDesc::custom | `comp::terrain::FACES` (+FACE_FALL), `CONTINENT_SCALE`, `FAR_LEAVES`, `CPU_HEIGHT.<landform>`; `Landform::custom` |
| TerrainDesc, terrain_op, FeatureDesc | dropped |
| HOLDS_LIQUID, `.holds_liquid()` | dropped (let go); `Tag::at` kept on `World::parts` |
| KNOCKBACK_RESISTANCE (ours) | upstream's |
| `client_op::REBUILD_SCREEN` 17 | 18 (upstream's MENU_SORT is 17) |
| SDK `Recipe` struct (`recipes`, `Recipe::at`, `hidden`, `shape`, `inputs`, `accepts`) | same methods on upstream's `RecipeView`; `RecipeView::at(&KIND, n)`; `hidden()` reads the recipe HIDDEN tag |
| `r.sounds_on(Target::Entity(k), p)` | `r.sounds_on_entities(kinds, prefix)` |
| `r.sound_group_on(block, group)` | same signature, now an entry edit |
| `World/Client::play_sound`, Hearer | upstream's `Output::play_sound`; `Client::start_sound` + `SoundHandle` kept |
| `World::raycast(..)`, `walk(..)` | upstream's `raycast(Ray)`, `walk(Walk)` |
| shader `World::entity_effect` | gone; upstream's status entity effects |
| stack looks `core:look`, `core:worn_look` | kept: `stack_look`, `with_look`, `with_worn_look`; engine `stack::own_look` is the fallback after a stack's LAYERS |
| `reg_entity_skin`, `Skin`, `scale_bone` | kept |
| `set_rain`/`raining_at`, `set_air`/`air_at`, `BiomeAir`, `air_flags` | kept (SYNC); weather-clouds was let go, but canopy, aurora, snowline and drying ride on set_air |
| `client_held_item` | removed (upstream's `client_displayed`) |
| `grip()`, `keeps_broken()`, `first_person_held_scale` | unchanged (`self.set` on upstream's components) |
| `ModMeta.package`, `modloader::load(.., package_dirs, ..)` | kept on upstream's ModMeta (source, revision, after, before) and load signature |
