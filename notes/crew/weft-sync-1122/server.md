# Group "server": merge resolution report

Every file below is free of conflict markers. Nothing was compiled (the brief says no cargo), so the notes on other groups' files at the end are the likely compile breaks.

## Per file

- **server/dim.rs**: Upstream's code is the base: features.rs worldgen, `space_id` everywhere, `WeatherField`, upstream's path API, attributes, knockback and mirror_fields.
  - Ours kept on top:
    - `rain_in`/`air` fields and `weather_in(biome, pos)` over `weather.at_pos`.
    - DimEvents for sky events, landings, air, tints, rain and day rate.
    - No collision for leaning bodies (`physics_body`).
    - `ShapeField.scale`.
    - The players' held and worn looks.
    - No falling onto unloaded land.
    - `hurt_at`.
    - `mark_resting` (it now uses `vehicle`, and the ALWAYS_ACTIVE tag through components), plus the resting walk carry-over on upstream's `Walk`.
    - `index_cells`.
    - The `NATURAL_SPAWN` question.
    - Drops at the player's feet (`drop_mined`/`drop_block_loot_at`).
  - Dropped:
    - Our ores, veins, structures and scatters (upstream moved them into features.rs; lod-trees-crowns verdict).
    - holds-liquid water fill, our `mining_rate`, our knockback resistance and our attack code (upstream's attributes).
    - `seat_ahead`, which nothing calls.
  - SYNC at line 567: `registry::FieldComps` needs `held_look` and `worn_looks`.
- **server/actions.rs**: Upstream's Ready, ControllerSave, HelmHeld, Grab* and Keys came in, plus its mining-on-entity and leave flow.
  - Ours kept on top:
    - Probation checks in place, StartBreak and Break.
    - Join and Proofs going through `prepare_join` and `.account`.
    - The held-pose no-fall.
    - `mount_moved` with `flies`.
- **server/player_file.rs**: Rebuilt from upstream's Arrival/prepare_join/finish_joins.
  - Ours kept on top:
    - The admission gate, which replaces the whitelist.
    - Portal arrival.
    - `name_for`.
    - played/lifted/since and ARRIVAL_PROTECTION.
    - The account-aware join log and DID and probation notes.
    - The identity/did/probation lines in the save.
  - Dropped: the mc_import spawn.
- **server/regions.rs**: Upstream's `WeatherField`, `set_weather(Option<u16>)`, timers and `tick_weather` came in.
  - Ours kept on top:
    - Day rate and its clock, sky events, landings, tints and air.
    - Rain by biome (`set_rain`), now a forced weather that `rain_in` limits.
    - `set_manual_weather(Option<u16>, ticks)`.
- **server/entity.rs**: Upstream's `Walk`, moves, keys, mount, anchors and `sent: Option<Move>` came in.
  - Ours kept on top:
    - `fly`, scale (`hit_body`, `dims`), moved_at, hurt_at, resting, ticked_at and steering.
    - Cell `moved_to` in `attach`.
  - Dropped: our attack_cooldown, worn, old `mount`, `seat_forward` and `keys: u8` fields.
- **server/sync.rs**: Upstream's space loop, `Clock` and mount came in.
  - Ours kept on top:
    - FAR_EVERY and move thinning by distance (`Moved`, move_due) with their tests.
    - BodyLean in the body loop.
    - Clock with `day_rate`, then Weather{id, only}, Air, SkyEvent, SkyLanding and Tints.
- **server/builtin.rs**: Upstream's `/clear` syntax, `/structure` and the `rules_locked` check came in.
  - Ours kept on top:
    - `/kill` (as VOID damage).
    - `/rule` value names with `held_for_run`.
    - The admission commands.
    - `/weather` through `set_manual_weather`.
- **server/load.rs**: Upstream's bodies (cave biomes, serialize_tree, timers, `save(.., &around, ..)`) came in. Our linked-world `Scope` wraps them.
- **server/path.rs**: Upstream's outright (`git show :3:`), as the coordinator said. Our wide-body pathing and its test are gone (fix-wide-paths verdict).
- **server/mod.rs**: Upstream's `Player.identity: Arc<PlayerIdentity>` came in.
  - Ours kept on top:
    - `account: Option<String>`.
    - The tick runs `run_day_clock`, `tick_timers` and `tick_weather`.
    - Our DimEvents.
    - The dimension change sends Clock (day_rate), Weather, Air and SkyEvent.
    - The LodFar and `lod_columns` helpers (our tuple form).
- **server/riding.rs**: Upstream's `seats_for`/`seat_at` came in, scaled by our mount scale when the mount has no helm. `mount_moved` keeps our `flies`.
- **server/extend.rs**: Upstream's outright. Our attribute constants are gone, since upstream's live in `registry::attr`.
- **server/space.rs**: Upstream's struct plus our `lean` and `lean_moved`.
- **server/liquid.rs**: Upstream's skip filter plus ours: leaning bodies are left alone.
- **saves.rs**:
  - Upstream's came in:
    - `game_mode` and `cheats` in WorldMeta.
    - core:covered→AIR.
    - Server-side saved LOD: `save_cube(.., Some(around))`, and `lod_tops` only when Some.
    - The new world-select sidebar.
    - CreateWorld (game modes, cheats, the world rules screen).
  - Ours kept: packages, key_index/generation, created, the "world.new_address" button, the translated default name, and the "create" id.
- **net/mod.rs**: Both sides' messages are in.
  - `Clock { dim, time, ticks, weather, rate, day_rate }`. `Time`, `TickRate` and `Attributes` are gone.
  - Ours kept: Weather{id, only}, Air, SkyEvent, SkyLanding, Tints, BodyLean, LodFar, `Join{name, key, account, did}` plus `Ready`, Proofs, Refused, achievements, revocations, and tuple LodColumns.
  - Upstream's BodyAir and Grab* came in.
  - `ShapeField` gets `scale` (default 1).
- **net/p2p.rs**: Ours kept (Who endpoints, ALPNs, world-key hosting, start_join). Upstream's `relay_mode()` came in. **PROTOCOL = 25**: upstream's 24 plus our `Clock.day_rate` and `ShapeField.scale`.
- **net/codec.rs**: The test uses `Join{name, key, account, did}` and `ToServer::Ready`.
- **physics.rs**: Upstream's moves came in. Our wall climbing rides on `f.moves.climb`.
  - SYNC at line 1700: upstream's `core:climb_speed` is a ladder multiplier (default 1), while ours meant wall climbing (default 0). It needs its own attribute name.

SYNC comments: **2** (server/dim.rs:567, physics.rs:1700).

## For Ronald

- Weather is now upstream's moving weather field. Our mod rain-by-biome is a forced weather on top of it, limited to the mod's biomes.
- `/weather` without a time now lasts until it is changed (upstream's default).
- New worlds use upstream's Create World screen: game mode, hardcore, cheats, and a world rules screen. Our rule toggle buttons are gone.
- World select uses upstream's new sidebar, with a cheats toggle per world. Our "new address" button is kept there.
- Upstream's permission now needs cheats on, even for the owner.
- Our admission gate replaces upstream's whitelist.
- Pathing for big animals is upstream's (`79dbcda93`, large-hitbox mobs). Ours is gone and needs a look with Drenkaun and the titans.
- Knockback resistance and attack speed come from upstream's attributes. The animals mod's values need to move to `core:knockback_resistance` in upstream's form.
- Ores, structures and scatters (trees) are generated by upstream's feature system (features.rs). Our per-site scatter code and its 96-high reach are gone (lod-trees-crowns verdict). Far crowns and the far terrain that read our scatter sites need checking.
- Water plants in generated seas take upstream's waterlogging (our holds-liquid fill is gone).
- The Minecraft-import spawn is gone.
- Moves are sent every tick, empty ones too (upstream's).
- New from upstream: `/clear entity`, `/structure`.
- Broken tools: their mining (content.rs `mining_rate`) and their attack damage now go through upstream's attribute and modifier code. That needs a check that a broken tool still works like the hand (content group).

## For other groups (likely compile breaks in files outside this group)

- **server/admission.rs, server/achievements.rs**: they still use `p.key`, `p.name`, `p.identity` (as the account Option) and `self.join(c, pi, name, key)`. They need `p.identity.key`, `p.identity.name`, `p.account` and `self.prepare_join(..)`.
- **registry.rs**:
  - `FieldComps` needs `held_look` and `worn_looks` (from `HELD_LOOK` and `WORN_LOOKS`).
  - `registry::attr` needs our glide, wind, lift and rope ids. Callers of `extend::ATTACK_SPEED` and `extend::KNOCKBACK_RESISTANCE` need `registry::attr::*`.
  - `tag_bits` has no ALWAYS_ACTIVE bit; dim.rs reads the tag through components instead.
- **host.rs**:
  - `Dim::weather_in(biome)` is now `weather_in(biome, pos)` (`raining_at`).
  - `set_manual_weather` takes `Option<u16>`, and the DimSet weather is `Option<u16>`.
  - `d.weather` is a `WeatherField` (`.at_pos(pos)`).
- **client**:
  - Receive needs `Clock.day_rate`, `ShapeField.scale` and our tuple LodColumns.
  - Upstream removed `seat_at`/`riding::turned`.
  - `ToClient::Time`, `TickRate` and `Attributes` are gone (replaced by `Clock` and the attributes field).
- **client/terrain_gpu.rs:157**: a comment names `dim::scatter_site`, which is gone.
