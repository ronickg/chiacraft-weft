# Moving our engine changes out of Weft's crates (plan, 2026-11-22)

Ronald, 11-22: "the more we can move out of the engine, the better ... some places we just do need
the custom version because it's not supported yet in Weft." The rule is in the weft-sync skill
("Moving our changes out of the engine"): moves happen in their own branches **after the sync lands**,
each leaves the game looking and playing the same, and where Weft lacks a hook we add the smallest
generic one any mod could use.

**What was checked:** each of the 55 "keep in engine" and "move out" rows of
`notes/ENGINE_CHANGES.md` section 11. They were checked against upstream Weft `cab2d5f72`'s SDK, engine
and docs: entries, typed tags, client hooks, events, `base_api`, shader handles and mod screens. Every hook
named below was found at the file:line given (paths in upstream's tree). "Not found" means searched
for and absent. Line counts are rough: what leaves our engine diff, and what a new hook adds.

## Totals

| Result | Rows | Engine lines saved | Hook lines added |
|---|---|---|---|
| **move out now** (Weft has the hook) | 9 | about 1,600 | 0 |
| **move out with a small hook** (80 lines or fewer each) | 13 | about 8,300 | about 590 |
| **move out with a bigger hook** (phase 3's second world host) | 1 | about 2,500 | about 200 |
| **keep** (Weft can't support it yet) | 32 | — | — |
| **all 55** | | **about 12,400 of 39,154** | **about 790** |

"Move out now" also counts rows moved only in part (`entities-animals`, `item-stacks-tools`,
`terrain-cpu-features`): their remaining part is listed under keep or a small hook.

## Results, row by row

### Move out now

| Row | Weft's hook (upstream file:line) | Saves | Risk |
|---|---|---|---|
| `phase5-finder` | Title screen part (`screen_part(&screens::TITLE, "buttons", ..)`, docs/ui.md:108-116, sdk/game.rs:1782) or replace `screens::MULTIPLAYER` (sdk/client.rs:876); join with `c.world_join(ticket)` (sdk/client.rs:360). Our tickets must parse as Weft's (engine/net/p2p.rs:177). | ~650 | low: a separate screen |
| `item-stacks-tools` | A stack's own `DURABILITY`, `MINING_SPEED`, `TOOL_LEVEL`, `ATTACK_*` win over its item's (docs/items.md:51); parts in a stack storage (items.md:53); `with_layers` (abi:1612); `condition(..)` picks model, texture and worn look per stack (docs/models.md:187, sdk/content.rs:2776). Still ours: keep-broken, and our ramp/offset layers (small hook: a `ramp` kind for `Layer.kind`, ~80). | ~400 of 708 | med: tool icons and worn armour must look the same |
| `plain-look` | A pack's `world.wgsl`/`sky.wgsl` replace the built-ins whole (engine/packs.rs:110, :121; docs/shaders.md:143). Ship a `plain` pack, picked by default. The two (Weft bug) lines and `sky.rs` stay. | ~250 | med: the default look changes unless `plain` is the default pack |
| `entities-animals` (part) | `World::set_size` (sdk/game.rs:248); per-bone scale from the client, `client_pose_bone`/`bone_scale` (sdk/client.rs:222); geo.json `set_model` (docs/entities.md:11). Still ours: mod-painted skins, any entity's UI preview, 12 layers (Weft has 8, client/models.rs:336). | ~150 of 1,124 | med: animals' look |
| `continent-scale` | `TerrainBuilder::field("continent", expr)` (sdk/terrain.rs:559). **First** check that the GPU and far path read the field, not a constant. | ~45 | med: the land changes unless the noise matches |
| `drops-at-feet` | `before_event(BREAK_BLOCK)` records the player, then `actions::SPAWN` with `spawn_cause::DROP` (abi:5355, abi:3298) moves the item with `entity_modify(id).pos(..).velocity(..)` (sdk/game.rs:572). Cleaner later: a drop-position value chain on `break_block` (~20). | ~35 | low |
| `fix-vulkan-sync` (part) | d50c23ad (faces counter at 256 bytes) is fixed upstream (engine/client/world_render.rs:1614): take theirs at the sync. The rest stays (see keep). | ~28 | low |
| `sync-port` (part) | `/kill` as a testkit command (`r.command("kill", "[who:targets]", ..)`, sdk/game.rs:1733; `world.damage`, game.rs:334); freecam on F6 from base (`r.edit(&Key::named("core:freecam"))`, docs/registration.md:510, engine/registry.rs:1664). The SDK additions stay. | ~19 | low |
| `theme-title-fonts` (art) | The `button_secondary*` and `list_row*` images go to `mods/base/assets/textures/gui/`: bare GUI classes are looked up in mods, last loaded first (engine/ui/look.rs:33, docs/ui.md:183). The font needs a small hook (below). | 0 code, 8 images | low |

### Move out with a small hook

Each hook is generic: any mod or pack could use it, and it's worth offering to Weft.

| Row | The hook (where, about how many lines) | Saves | Risk |
|---|---|---|---|
| `phase4-admission` | **A join question**: `question::JOIN` asked in `prepare_join` after the whitelist (engine/server/player_file.rs:36), with the key, name and a refusal reason; plus `w.disconnect(player, reason)`. Bans and DIDs become `SAVE` mod components (docs/modding.md:230); screens via screen parts; `player_key` (sdk/game.rs:202). ~50 lines in engine, abi, sdk. | ~4,500 | med: join order, saved data moves into components |
| `upscalers-dlss` | **`Backend::upscale`** with a default no-op, behind our `Cmd::Upscale{color, depth, motion, out}` (render/src/lib.rs:417), so `ngx.rs`, `streamline.rs` and `upscale.rs` live in a `vulkan-dlss` feature crate. ~60. | ~2,500 | med: RR guides touch many shaders; a PC DLSS bench must match |
| `wind-life` | **A particle step snippet**: a `shaders/particle_sim/` kind (`fn step(p) -> p`) chosen by a particle kind's shader slot, as engine/client/particles.rs:31 does for drawing (`particle_sim.wgsl` can't be replaced today, particles.rs:281). Wind inputs come from the scene values below. ~80. | ~250 | med: gusts and shelter |
| `phase1-account` | **`p2p::set_identity(SecretKey)`** before the endpoint is built (engine/net/p2p.rs:141; today it is read from `identity.key`, p2p.rs:37-56). The Account screen goes through a screen part. ~15. | ~230 | med: a wrong key makes the player new everywhere |
| `depth-water-shafts` | **Depth and scene in post passes**: bind `SCENE` and `WORLD_DEPTH` (sky already gets them, engine/gpu_layout.rs:1098) to the post block (gpu_layout.rs:1109), and add a world copy before translucents, both gated by `pack_uses` (packs.rs:69). ~40. | ~150 | med: the copy's timing for refraction |
| `material-maps` | **LabPBR layers**: load `_n` and `_s` into the bindless set and bind them to `world.wgsl` (today `_s` is emission only, docs/textures.md:68). ~80. | ~150 | low |
| `survival-hooks` | **World meta**: key/value data a mod reads and writes, saved with the world (`saves.rs`; `world_session` at abi:4237 is only a per-open number). `ui.wgsl` bits go to `shaders/gui/` snippets or the pack (docs/shaders.md:26, :141). ~50. | ~150 | low (no old-save logic needed) |
| `sky-events` | **Mod scene values**: `EnvironmentValues.custom: [[f32; 4]; 4]` (abi/src/environment.rs:31), mixed like the other fields, copied into `Scene` (gpu_layout.rs:616), sent with `World::environment` (sdk/environment.rs:172). ~50. | ~120 | low |
| `terrain-cpu-features` | What an `Expr` can say moves onto Weft's `field`/`landform`/`BiomeRule` (sdk/terrain.rs:563-603); the rest goes behind **one `FeatureDesc.custom` CPU callback** that the GPU path reads. ~60. | ~120 of 247 | high: near and far terrain must still match |
| `calm-pipelines` | **Pack variants**: `pack.cfg` `variants = CALM, DRY`, compiled as extra pipelines and picked by scene flags (`compose`, packs.rs:121, takes no defines today). ~60. | ~90 | low: speed only |
| `wet-snowline` | The scene values above (base sets wetness and the snow line). +0. | ~45 | low |
| `day-rate` | **`set_day_rate(f32)` per dimension**, sent to clients (Weft has a static `DAY_LENGTH`, abi:1339, and `set_day_time`; nudging time each tick stutters the sky). ~30. | ~30 | low |
| `cracks-32px` | **Crack textures looked up like bare GUI classes**, mods last loaded first (engine/content.rs:2009 reads only `core:block/destroy_N`). ~6. Plus the font: look for `fonts/default` in mods before `core:` (engine/ui/render.rs:297), ~8. | 10 images, ~0 code | low |

### Move out with a bigger hook

| Row | The hook | Saves | Risk |
|---|---|---|---|
| `phase3-links` | `links`, `friends`, `travel` and `world_address` move to our crate; messages to our own players go over mod channels (`r.packet`/`r.client_packet`, sdk/client.rs:702, :713). Missing: **a second world host** beside the joined one (Weft keeps one `server: Option<ServerHandle>`, engine/app.rs:222). That is ~150-250 lines in `app.rs` and `net`. World-to-world messages stay our protocol. | ~2,500 of 3,829 | high: two servers in one process touch the app's lifecycle and saves |

### Keep (Weft can't support it yet)

| Row | Why it stays |
|---|---|
| `phase2-packages` | Weft switches content only by restarting (`restart_pending`, engine/modloader.rs:409). Only `package_trust.rs` moves (~135). Re-check it against Weft's entries after the sync. |
| `phase6-sandbox` | The loader is libloading only (modloader.rs:472). A `ModHost` trait (~80) would move ~120 later. |
| `net-path-log`, `server-identity`, `bots` | Not in Weft; small. `server-identity` is a Weft bug (a nameless player keyed by name, player_file.rs:15). |
| `path-tracing`, `rt-shadows-reflections` | No pass hook, extra targets, G-buffer or ray-query API in the render API. A later `RenderPass` trait (~150) could carry them. |
| `sun-shadows` | Weft ray-marches `solid_at` to ~112 blocks (gpu_layout.rs:1118): no shadow maps, far cascade or mob casters. Only our gating can become `pack_uses` (~20). |
| `hdr-world`, `taa`, `froxels-mist` | Post targets are the surface format (post.rs:93); no motion vectors, history or compute pass after the shadows. |
| `frame-perf`, `diagnostics` | Engine internals; Weft has frame-level `Cmd::Mark` only (render/src/lib.rs:301). Check overlaps with Weft's pipeline cache and occlusion at the sync. |
| `biome-tints-season`, `pack-options` | Weft sets tints at registration only (sdk/game.rs:1822); `pack.cfg` has only a name and description (packs.rs:37). Ours are already the generic hooks: offer them to Weft. |
| `canopy-light` | Mesher air flags a pack can't make; its `world.wgsl` part (~15) can go to the pack. |
| `lod-fade-end`, `lod-far-look` | Weft fades LOD levels by distance only (client/world_render.rs:1149). |
| `movement-climb-glide-grapple` | Weft has `CLIMB_SPEED`, `AIR_CONTROL`, `GRAVITY` attributes (abi:5545-5565): port our speeds onto them (~60 saved). There is no mod movement mode in the client-predicted step (a ~120-line hook, high risk). |
| `entity-ranges-ai-cost` | Weft sends by view radius only (server/sync.rs:239). No activation range or column queries. |
| `felling-lean` | Weft's `body_fall` topples a felled tree under physics (sdk/world.rs:173), but has no authored posed body like our `body_turn`. A trial could save ~180 if Ronald likes the physics fall. |
| `held-item-scale` | Weft's hand placement is the item model's `display` only (docs/models.md:169); ours is a ~15-line hook already. |
| `fix-vulkan-oom` | No spill to system memory in Weft's backend. |
| `fix-vulkan-sync` (rest) | Barriers (bda04355), depth copy layout (940a1b2f), MoltenVK errors (c52818ca), early writes (3727f9a0): still as before upstream (backends/vulkan/src/lib.rs:1210, :2198, :244, :863). The shadow pass's globals are ours. |
| `fix-small` | Checked: drips (client/mod.rs:2574), particles in liquid (particles.rs:400) and other entities' poses (modui.rs:699) are still unfixed upstream; `/rule` partly (builtin.rs:83). Re-check macOS flat blocks after the sync (`world.wgsl` was reworked). |
| `sync-port` (rest) | `raining_at`, `set_rain`, `air_at`, `liquid_in_cell`, `set_biome_tints`, `body_turn` are not in Weft (only `weather_at`, sdk/game.rs:495); first-person worn bones still cut by role (entity_render.rs:1481). |
| `ui-pixel-kit` | No `at` placement, pressed offsets or pixel-grid font. Weft's `button_content` (sdk/ui.rs:658) may replace our `button_with` (~20). |
| `gui-scale-small-windows` | Weft allows any whole scale and has no scrolling for a screen taller than the window. |
| `hud-chat-crosshair-outline` | The outline still thickens with distance (entity_render.rs:2120). A `visible_when` on HUD elements (~15) would move the crosshair rules out later. |
| `load-checks`, `mac-paste`, `run-settings` | Not in Weft (missing textures only warn, assets.rs:439; paste is Ctrl only, ui/edit.rs:129; run settings are re-applied only on load, config.rs:228). |
| `sound` | Weft's `Play` has no handle or loop and an unbounded voice list (audio.rs:29, :99). Port ours onto Weft's `audio.rs`; take its `ui/click`, `item/pickup`, `weather/thunder_*` files. |
| `removed-mc-pack` | Weft still has `minecraft.rs` and calls it (app.rs:506): re-delete both at each sync. |

## The branches, in order

Each starts after the sync lands, from dev, in its own worktree. Each ends with:
- its ledger rows updated (the row leaves, or shrinks to its one hook line);
- `notes/crew/engine-diff.sh` clean;
- the same look and play, shown by Tester's runs.

They are ordered by lines saved against the risk to the look and the play.

| # | Branch | Rows | Saves / adds | Risk | Proof it's the same |
|---|---|---|---|---|---|
| 1 | `move-chiacraft-crate` | `phase5-finder`, `phase1-account`, the move of `engine/src/chiacraft/` into a `chiacraft-worlds` crate (files only, still called from the engine where they are now) | ~880 / ~15 | low-med | Find Worlds lists and joins; the account key and DID are the same after a restart |
| 2 | `move-admission` | `phase4-admission` (join question + disconnect hook) | ~4,500 / ~50 | med | two-player join checks: a ban refuses, an achievement rule admits, DIDs prove |
| 3 | `move-quick-wins` | `drops-at-feet`, `sync-port` part (`/kill`, F6), `fix-vulkan-sync` d50c23ad, `theme-title-fonts` art, `cracks-32px` + font hooks, `continent-scale` (after its check) | ~130 / ~15, plus 18 images out of the engine | low | a felling script, the testkit suite's `/kill` scripts, title and cracks shots, one terrain look at the same seed |
| 4 | `move-upscalers-crate` | `upscalers-dlss` (`Backend::upscale` hook) | ~2,500 / ~60 | med | PC DLSS bench and look, dev against branch |
| 5 | `move-item-stacks` | `item-stacks-tools` (Weft's per-stack values + a ramp layer kind) | ~400 / ~80 | med | crafting part looks, tool numbers, worn armour shots |
| 6 | `move-scene-values` | `sky-events`, `wet-snowline` (mod scene values), canopy's pack part | ~180 / ~50 | low | blood moon, rain and snow-line looks |
| 7 | `move-plain-pack` | `plain-look` as a default `plain` pack | ~250 / 0 | med | stage 1 sheets against vanilla + DH, unchanged |
| 8 | `move-render-hooks` | `depth-water-shafts`, `calm-pipelines`, `material-maps` | ~390 / ~180 | med | stage 2 look sweep and one bench |
| 9 | `move-wind-particles` | `wind-life` (particle step snippets) | ~250 / ~80 | med | wind look (with `--record`, shader time pinned) |
| 10 | `move-world-bits` | `survival-hooks` (world meta), `day-rate`, `entities-animals` part, `terrain-cpu-features` part | ~450 / ~140 | med-high (terrain) | first-day script, animal looks, far-terrain look at the same seed |
| 11 | `move-links` | `phase3-links` (second world host) | ~2,500 / ~200 | high | link, portal and travel checks across two machines |

Rows not in a branch stay "keep in engine". At each sync, check them again against Weft's SDK, as the skill says.

## Hooks worth offering to Weft

These are generic and small, and each would remove a whole kind of fork edit:
- the join question;
- mod scene values;
- the particle step snippet;
- depth and scene in post passes;
- pack variants;
- LabPBR layers;
- `set_day_rate`;
- crack and font lookup in mods;
- `Backend::upscale`;
- our pack options and runtime biome tints, which are hooks already.

Ronald sends no PRs (11-05), so they stay ours, marked in the ledger.
