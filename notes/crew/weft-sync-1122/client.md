# Group "client": report

Files: crates/engine/src/client/mod.rs, receive.rs, world.rs, modui.rs, README.md (+ docs/weft-README.md). No conflict markers left. I checked that the four .rs files parse (rustfmt on copies). Nothing was compiled.

## `Client::render` signature (for the app group)

It keeps upstream's shape: `render(&mut self, gpu, list: &mut CommandList, after: &mut CommandList, early: &mut CommandList, frame: &FrameInfo, textures: &GameTextures, animate_lod: bool) -> bool`.
- Our `late_out` field is gone. The late draws (sky, translucent, wake, particles, weather, hand) go into `after`, followed by `Cmd::Mark`, as upstream does.
- The app must run our world copy (`behind`) before it records `after`.
- `animate_lod` is ignored: our far terrain still updates from `upload_meshes`.
- `behind`, `taa_out` and `frame_camera` stay as they were.

## Per file

- **mod.rs**
  - **Upstream's, taken in:** environment, `lightning::LightningEffects`, WeatherField `weather_at` with `weather_override`, ItemStack `worn`/`held`/`off_held`, `FieldComps` struct, `Stuck`, `dry_boxes`, `physics::Moves`, `send_join`, `preparing_since`/`ready_sent`/`has_world`, `update_glow`, `motion_effects`, server-driven `body_target`, the clip clocks (`age`/`arm_swing` by ticks), `physics::seat_at`, pose conditions, `content::swims_in`, `equipped`, the new Scene fields (environment, cloud, cloud_lod, atmosphere caches, `lightning_start`/`end`), the atmosphere wiring of `sky.draw`, `wake_render`, `prepare_uploads`, `fog_color` from the environment.
  - **Ours, kept on top:**
    - Look: hand light, TAA (`taa_frame`, `taa_last`, `taa_out`, `frame_camera`), RT `rt_frame`, the calm pipelines, sun shadows, our leaned sun path (now times the environment's sunlight), soft clouds, froxels, frame_look, cloud_shadow, pack options, `behind`.
    - Weather and air: rain by biome (`rains_at` share, `gloom`), wind, fog, snow, dust, aurora, canopy, wet, puddles, snow line. `fog_color.w` is still our `sky_rain()`.
    - Movement and entities: climb, glide and rope (now read from the attributes component by name, `our_moves`), entity scale (seats scaled through `scaled_seat`), felling `lean`.
    - Other: worn and held looks, the account and DID in `Join`, diag parts and spans, the drip guard (now `states.get().is_some_and(solid)`), block texture preload.
    - Far terrain: ours. `Lod::new(lod_tx)`, `lod.update` back in `upload_meshes`, `column_shown(.., None)`.
  - **Dropped:**
    - Our client-side random lightning. Our pack's `Scene::lightning` (`lightning_seen`, `lightnings`) is now set from upstream's real strikes in receive.rs.
    - `lightmap_day` (`--mc-days`, test-mc-compare drop); `params.z` is the environment's daylight.
    - Our client `turn_body` and the "mob standing still turns its body" tweak (upstream's server now turns bodies).
    - `creature_sounds` (upstream removed it); the local `swims_in` and `words`; `ToClient::Attributes`.
  - **SYNC comments:**
    - Line 2225: `lod_ready_for_entry` is `lod.pending() == 0` (our far terrain has no `ready_for_entry`).
    - Line 3096: `held_look` and `worn_looks` must be in `registry::FieldComps`. The registry was still unmerged when I finished.
    - Line 3543: no `lod.save_view` in our far terrain.
    - Line 4622: our climb, glide and rope poses need those states in upstream's pose conditions (`conditions::Body`). Until then those poses never hold.
- **receive.rs**
  - Upstream's `Clock` (with our `day_rate` added by the net group), `BodyAir`, `Lightning`.
  - Ours: `BodyLean`, `Air`, `SkyEvent`, `SkyLanding`, `Tints`; `Weather { only }` sets `rain_in` (its id comes through Clock).
  - `TickRate` and `Attributes` are gone (upstream's Clock and the attributes component replace them).
  - `LodColumns` (upstream's `LodColumn`) is mapped onto our tuples. The Lightning handler now also sets the pack's flash.
  - SYNC at line 113: our far terrain doesn't take `lod.set_edits`.
- **world.rs**
  - Upstream's `ticks`. Ours: `rain_in`, `air`, `tints`.
  - `column_shown` keeps our signature (`ground: Option<i32>`, liquid counted as see-through), because our lod.rs calls it. Upstream's `through: bool` (translucent counted as air) is not in.
- **modui.rs**
  - Ours: `at` from `UiNode::at`, plus upstream's new `at_from: [0, 0]`.
  - Upstream's `using`/`using_ticks` in the client state.
- **README.md**: ours (the chiacraft README). **docs/weft-README.md** now holds upstream's README.md as it is (it lost our older Android lines, which upstream's doesn't have).

## Dependencies on other groups (sent to main while working)

- The net group added `ShapeField.scale`, `Clock.day_rate` and kept `Weather { id, only }`.
- The abi group added `attribute::WALL_CLIMB_SPEED`.
- Still needed from the registry group: `FieldComps.held_look` and `FieldComps.worn_looks`.
- Still needed from the world_render group: `WorldRenderer::draw(.., &leans, &dry)`. Its current text already has both.
- I kept `Client::worn()` and `worn_looks()` for app.rs. If app.rs takes upstream's `preload(.., &me)`, `worn()` becomes dead code.

## For Ronald

- **Weather is now upstream's regional weather** (WeatherField by position and seed, server override through Clock).
  - Our rain-by-biome share and slow gathering (`gloom`) still ease `self.look`.
  - Upstream's sky no longer takes `self.look`: it draws from its environment values. So our slow gathering of the sky may not show in the clouds any more.
  - The rain particles now go by upstream's weather field, not our biome share.
- **Lightning is upstream's**: server bolts, flash lights in the scene, thunder from the server. Our random client flashes are gone. The pack's flash (`Scene::lightning`) and the mods' thunder counter now follow upstream's real strikes, with the direction to the strike.
- **Mob bodies turn by the server** (upstream). Our client "mob standing still turns its body to its look" tweak is gone; check that mobs still face what they look at.
- **The sun**: our leaned path stays, now dimmed by upstream's environment sunlight (`sun.w`). The scene's daylight (`params.z`) and night vision (`light.w`) are upstream's environment values.
- **Climbing, gliding and rope poses**: they don't show until the pose conditions learn those states (SYNC). The movement itself still works.
- **Soft clouds and the held light** stay in for now (the ledger asks for an A/B against upstream's volumetric clouds and dynamic lights first).
