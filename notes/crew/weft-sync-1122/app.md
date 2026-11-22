# Group "app": merge report

All files below are free of conflict markers. Nothing was built or run (no cargo, no tests).

## Per file

- **app.rs**: Upstream's base: async world opening (`opening_world`, `entering`, `LoadingWorld`, `enter_world`, `draw_entering`), the renderer list, the pipeline cache in the backend, `Client::send_join` + the new `Client::new`, the death screen with `entering`, Android `resumed`/`suspended`/`about_to_wait`, touch through `Touches`, `world_frame`, block previews and the menu's own player preview.
  Ours on top: links/friends/saved worlds/account start-up, packages (`content_changed`, `use_packages`), joining (`poll_join`, `enter_remote`, travel, home), the world's identity (carried through `OpenedWorld` and `OpeningWorld::Streaming` to `Game`), `host_if_listed`, `play_local` (now joins and leaves the client to `finish_world_open`), `upscale_frame`, GPU config block, R with nothing hovered, `menu_open` to the HUD, `PreviewKind::Entity(whose)`, entity previews of a menu's own entity or a kind alone, TAA/denoiser/froxel/cloud-shadow settings (as free fns), `scroll_when_taller` (inside `build_screen`), screenshot threads, the reload wait.
  `record_passes` is in the coordinator's order: world pass + EndPass, Mark, `take_depth(.., atmosphere)` + `sample_atmosphere`, with the copy on `copy_behind` + `sky.soft.passes`, then `after` in a pass with our TAA motion attachment, then the path tracer and effects. The marks still split the 8 `GPU_PARTS` (`after` ends with the client's mark). `animate_lod` is `r.entering.is_none()`, as upstream.
  Dropped: `begin_world`/`pending_world` (upstream's loading flow replaces it), our `touch`/`suspended`/`ANDROID_DESTROYED`, `save_pipeline_cache`, the fps cap, the `mc_import` call.
  SYNC: line 556 (`modloader::load` signature), line 3188 (soft clouds are forced on since the setting is gone).
- **app/api.rs**: Kept `REBUILD_SCREEN`. Upstream removed `POST_EFFECT`. The test-mc-compare fns (`mc_days`, `bliss_sun`, `sun_tilt`) are still there because their callers belong to other groups.
- **host.rs**: The Host table matches abi's `Host` exactly. The old `reg_*` fns are gone (upstream's entries rework). We keep body_turn, entity_fly, entity_ignite, set_rain, set_sky_event, sky_landing, set_air, content_tint, set_biome_tints, air_at and raining_at. `entity_effect` now uses upstream's level version.
- **screens.rs**: Upstream's Profile, Title and Pause screens and ConfigScreen. On top: account backup and account buttons, achievements, write_words, return_home and admission buttons. Our SettingsScreen and ChatSettings are dropped.
- **net/screens.rs**: Our saved worlds, links, finder, friends, Joining, JoinSheet and Links screens. Upstream's Multiplayer screen (copy and paste) and `back_button`.
- **settings.rs**: Our settings are now free fns, like upstream's. Dropped: `fps_cap`, `soft_clouds`, our `render_scale(density)`. `max_gui_scale` reads `touch_controls()`.
- **config.rs**: Upstream's entry list with ours put back in as `number/string/rule` entries. Dropped: our lod_distance/lod_detail/render_scale/fps_cap/soft_clouds/mc_leaves. Two test helpers (`num_def`, `text_def`).
- **hud.rs**: `update` has upstream's signature plus `menu_open`. Kept chat_room, the TOP overlay, the column and row ids, the first-person-only crosshair and slot numbers (`slot_icon`). `hotbar_slot_at` is gone (Android).
- **ui/render.rs**: Upstream's `plain_icon`/`item_draw` and scrollbar. Kept our sliced fill, the sink, icon-first slots and `defer_stack_sized` (icon size and slot number).
- **ui/edit.rs, ui/mod.rs, input.rs**: Upstream's, plus our `!cmd`, the root scroll rule and the freecam comment.
- **touch.rs, clipboard.rs**: Upstream's outright.
- **bench.rs**: Ours (8 GPU parts).
- **log.rs**: Android logcat plus our writer thread. **commands.rs**: Upstream's `parse_alt` plus our fallback guard. **assets.rs**: glow/mat vectors. **diag.rs**: Upstream's cfgs, our spans. **lib.rs**: Ours minus minecraft, plus upstream's exports.
- **dedicated.rs**: Identity, free settings fns, no mc_import. SYNC: line 19 (`modloader::load`).
- **audio.rs**: Upstream's acoustics (echoes, delays, filters, LocalMusic, set_active) with our categories and reach.
- **Cargo.toml**: Our serde plus upstream's reqwest/rustls/webpki/sha2 and Android section.
- **lang en/fr/ru**: Merged key by key with both sides' keys. touch.* dropped (Android) and duplicate keys removed.

SYNC comments: 3 (dedicated.rs:19, app.rs:556, app.rs:3188).

## Cross-group dependencies

- abi: `REBUILD_SCREEN` = 17 clashes with upstream's `MENU_SORT` = 17; ours needs another number (18).
- modloader: `load(mod_dirs, package_dirs, cache_dir, off, mods_off, core_assets)` is assumed.
- game_screens: `chat_lines(.., chat_room)` and `RecipeQuery::Kinds`. entity_render: `preview_kind` and `draw_preview(.., worn, subject)`.
- world_render: two `take_depth`s are still in conflict there; app.rs calls the 5-argument one (`atmosphere`).
- client: `environment.post`, `sky.soft_wanted`, `sky.soft.passes`, `late_out` gone, `send_join`.
- Callers of removed settings methods elsewhere must use free fns (`crate::settings::x()`).

## For Ronald

- Upstream's screens now: Profile, Title, Pause, and Settings as ConfigScreen pages. Ours are added onto them.
- The first-person model is on by default (upstream).
- All sound categories share one environment track, so UI sounds are now muffled underwater too.
- Worlds open the upstream way: the loading screen and the picture grow in. Our "draw the loading screen first" step is gone.
- The `core:soft_clouds` setting is gone (ledger). The soft clouds are forced on until the A/B against upstream's clouds.
- `core:fps_cap` is gone; upstream's frame limit replaces it. The pipeline cache now lives in the backend.
- Upstream's renderer choice (wgpu/vulkan) came in.
