# Our changes to Weft's core, and why

Written 2026-11-04 from dev's history. "Core" here means Weft's own crates: `crates/engine`, `sdk`,
`abi`, `render`, `noise`, `app`, the `mod-sdk*` crates and `backends/vulkan`. Our own new crates
(`sandbox`, `packages`, `account`, `bridge`) are additions on top, not core changes, and are
listed at the end only. The crypto side (wallet, DIDs, the chain) is left out of this file.

Since our fork's base with upstream (2026-11-01, 6e4e1685), 214 commits touched the core (of 223 in
`crates/` and `backends/`). PLAN.md rule 4 says to leave the engine alone where we can, so each change
below is here because a feature or a fix needed something only the engine could give.

Each line says **why** we needed it. **Upstream?** marks what might go to Weft one day. Ronald's rule
for now is bug fixes only, written and opened by him.

## 1. Bug fixes (upstream candidates)

| Change | Why | Upstream? |
|---|---|---|
| Flat blocks on macOS: texture sizes without `arrayLength` | Blocks drew flat on Apple GPUs | Merged upstream (#67) |
| Vulkan: free GPU objects only once no frame in flight uses them | GPU device-lost on asset reload | Merged upstream (#78) |
| Hold back asset reloads until the player has joined | Same hang, from a reload before the join | Merged upstream (#79) |
| Vulkan: drop uploads queued for a destroyed buffer or texture (9b919255) | Copy into a reused slot, then GPU crash (Xid 109) | #83 closed; upstream re-implemented it (1b9b2774). Take theirs on the next sync |
| Vulkan: order upload copies that write the same bytes (0b06e383) | Overlapping writes in the wrong order | #84 closed; upstream re-implemented it (1049ba95). Take theirs |
| Vulkan: GPU-only memory spills to system memory when full (a3606d48, f88e5a7c) | Two games on one GPU panicked with "Out of memory" | #97 closed for now; clean branch ready for Ronald |
| Far terrain: a scatter that forces all its blocks takes its commonest block (414e4e88) | Boulders drew as purple missing-texture blocks in the distance | #96 closed for now; clean branch ready |
| Vulkan: `drawIndirectCount` optional (`Backend::indirect_count`, `Cmd::FillBuffer`): without it the table's count draws draw all their records, zeroed before the culling writes them; `--no-indirect-count` forces it | Since Weft's GPU terrain culling (c376cf9c) the game wouldn't start on any Mac: "Apple M3 Max lacks what the game needs: drawIndirectCount" (MoltenVK has none) | Candidate 6: clean branch for Ronald |
| HUD: crosshair only in first person (6a394c0d) | It sat on the player's back in third person | #98 closed for now; clean branch ready |
| `/rule` reads values the way the rule takes them (a2f4c81c) | `false` was refused, `2` and `999` clamped silently | Clean branch ready, never opened |
| Box-model blocks' far faces use their boxes' textures (e1f48a48) | Magenta in distant terrain | Maybe |
| Setting values follow their names on a content switch; ids kept (c4a527ea, c555d7ec, 3ec8ef9c) | Switching a world's packages scrambled settings and broke an empty-name join | Maybe |
| A content switch registers the keys its content brings (47a8be49) | New keys were missing after a switch | Maybe |
| Under water the sky's gradient is drawn (2562a0cd) | Navy streaks where the sea floor ends | Maybe |
| Sky pictures upright on a leaned sun path (81ff2dcc) | The moon drew as a tilted diamond | With the leaned path only |
| Paths for bodies wider than a block: room across the whole footprint, at the entity's own size and scale (`server/path.rs` `footprint`, `find(.., width)`; a899c90e) | The A* gave every walker a one-cell body, so wide mobs were led between trunks they can't pass and jammed (our big animals, 2-4.5 wide, barely moved in forests) | Yes, a bug fix: vanilla's pathing sizes nodes by the mob's width. With a unit test (`a_wide_body_goes_round_a_gap_it_cannot_pass`) |
| clippy allows for `mut_from_ref` (0d111b9a, d7ff2255) | A newer clippy made them errors | Trivial |
| Client: water drips (a solid block under water, air under that) read the block under with `get`: a cube not arrived yet is `u32::MAX` (`client/mod.rs`) | It indexed past the block states and crashed the client a second after arriving by a tarn (water above the sea's level, near the top of what had arrived) | Yes: a plain bug fix |

## 2. Rendering, for our shader pack (Bliss-like look)

Weft's shader packs could only replace shaders. Our pack needed data the engine didn't give.

- **Sun shadows:** a shadow map from the sun for packs (48d38929), from mobs and the player (cc34e2dc), the faces the light falls on (5a096e0f), a coarse far cascade to 400 blocks (f03b29b6), distant terrain casting (c4698cc5), cloud shadows (6b63b917), and a sun path leaned 35° south (5daa6e72). *Why:* real shadows are the core of the look.
- **Depth for screen effects** (3f4035e2), **water that bends and tints what's under it** (df721f33), **light shafts, with fog kept out of caves** (4a875f7e). *Why:* water reflections, contact shadows and shafts need the depth buffer and a copy of the world.
- **Pack options, the "look knobs"** (e2c8496e): a pack names up to 32 settings (16 until the fantasy settings, branch `options-32`, 11-05: `Scene.options` `[[f32; 4]; 8]`) and the engine passes them to its shaders. *Why:* Ronald wanted the look tunable without rebuilding, and every fantasy effect gets its own setting ("we can also have the option to turn on off parts").
- **Held light** (5ae27f4f, 3cdfa80d, b752319f, 3e5ce4b8): a torch in hand lights the way. **The night follows the moon** (fc1478ca), **the night sky** (104c0e39). *Why:* nights are dark by design, so the torch and moon must carry them.
- **Clouds follow the weather and the wind** (5a42062b, 1bae91a3), **biome tints a mod can change while running** (34432291, used for seasons and rain). *Why:* the living-world systems (seasons, rain by place) have to show.
- **Canopy light** (e40f323e, on look-canopy): a biome's air flags carry how closed its canopy is (`weft_abi::air_flags::CANOPY_SHIFT`, bits 16-23, 0-255). The client eases it over 10 s into a new `Scene.canopy` field, the same path the aurora takes. *Why:* the tropical forest's shade and sunflecks under closed crowns.
- **Soft clouds** (Lighting, branch `soft-clouds` from the sync tip, 11-05): `client/soft_clouds.rs`, a setting `core:soft_clouds` (off: the pack's blocky clouds). When the pack's `sky.wgsl` has `fs_cloud_trace` and `fs_cloud_resolve`, the engine traces them into a quarter-size target each frame after the world is copied (`app.rs`, after `copy_behind`), blends that into one of two histories, and the sky's `fs_clouds` draws from it. Three new `Scene` fields: `soft_clouds` (the history's, last frame's and the noise volume's indices, the history's age), `prev_view_proj`, `cloud_moved` (camera move less the clouds' drift, frame number); a 64³ tiling noise volume made once on the CPU (`noise_atlas`); old targets are destroyed only after the frames that used them (a resize must not hang the GPU). *Why:* Ronald, 11-05, "Both, a setting" and "Soft clouds first": Bliss/Photon soft clouds at a sixteenth of the pixels (Research round 15, rec. 7).
- **Screen effects at half size** (Lighting, branch `half-res`, 11-05): a second post hook, `post_low` (`shaders/post_low/<name>.wgsl`, `fn low(p: PostPixel) -> vec4<f32>`): an effect's half-size part, drawn before the effects into its own Rgba16Float image a quarter the pixels, which the effect of the same name reads (`Pass.low`, `PostPixel.low`, `post_low_upsampled`: the four nearest texels weighted by depth). *Why:* Research round 16 #7, #5 and #1b: volumetric light, shafts and the mist march need far fewer pixels than the screen's; the Mac pays 5× the PC per pixel.
- **Sky events** (Lighting, branch `sky-events`, 11-05): `Host::set_sky_event(kind, strength)` and `Host::sky_landing(pos, seconds)` (sdk `Game::set_sky_event`, `Game::sky_landing`, kinds in `weft_abi::sky_event`), a dimension's state sent to its players on a change (`ToClient::SkyEvent`, `ToClient::SkyLanding`), into two `Scene` fields, `sky_event` and `sky_landing`. One value for the whole world, not air-flag bits (those stay for enchanted air by biome). *Why:* Ronald, round 17: blood moons, meteor showers, fireballs and aurora storms, "Gameplay too"; the base mod's schedule (First hours) says when, the pack draws them.
- **Two-channel entity glow masks** (Lighting, branch `sky-events`, 11-05): an entity's `_e.png` mask goes to the GPU as Rgba8 (`assets::emission_mask_rg`): r its glow as before, g the picture's green × alpha for packs (the blood moon's eyes, Art's masks with red 0). Blocks keep their one-byte masks. *Why:* the mask was one byte (red × alpha), so the eyes painted in green never reached the shader.
- **Performance:** render scale (519dbf32, 14a00a37), a pipeline cache on disk (b1867503), a bounded, depth-only shadow pass (409d70ed), screen effects in one pass (716acfd8, ce011d36), the water's grid only when changed (bdb745ef), small frame-thread steps in place (ac4995f3), the frame thread culling its own draws (c31f6d34), skin painters off the frame thread (1041e730). *Why:* Ronald's Mac lagged at 3456x2234; these took it from about 96-124 fps to 128-154 fps uncapped.
- **Diagnostics:** GPU time per pass in benches (ee147880), slow frames name their step (2394267c), where "submit" goes each second (9ab54ee5, bf07e118), the size the world draws at (ce993538), PACING lines (dcde2415). *Why:* to find the lag and the hitches.
- **Breaking cracks at 32 px** (Art, claude/art-cracks 3fa5964f, merged 0800ffb5): `crates/engine/assets/textures/block/destroy_0-9.png` redrawn 32x32 in place (grown by `notes/blocks/destroy.py`), same names, so `core:block/destroy_N` is unchanged. The pack darkens the face with them (`entity.wgsl` `fs_overlay`). *Why:* the 16 px near-black cracks stamped a chunky black maze on our 32 px blocks (Ronald: "glitchy").

## 3. Distant terrain (LOD) and world generation hooks

- **Trees per site, rock on steep faces, stable levels; smooth, morphing far ground; skirts under edges; a tall crown tapers** (747f25bc, 87aa8332, e2c0d171, 0399162b). *Why:* distant forests and mountains looked blocky or wrong, and giant sequoias showed flat-topped.
- **Far trees as crowns over trunks** (branch lod-crowns). Each scatter's shape is sampled 16 times per biome and 8 of the runs kept as crowns (radius, top, lowest leaves, widest layer, how round its top is, trunk width and block, leaves); each far site takes one by its hash and lays it over the cells round or tapering, over the real ground, with a trunk at levels 1-2. A sample comes back as 4 words, not 2, and the LOD compute pass uses 18.5 KB of workgroup memory (was 13.9; Apple GPUs allow 32). The old `TALL_CROWN` taper is gone, the profile covers it. *Why:* Ronald, 11-05: "why the lods still boxes ?": far forests were flat-topped leaf pillars down to the ground, one median tree per scatter.
- **Far steps keep their top's colour down a cell's height** (66a12a45, `lod::mesh`). *Why:* at 3456 px (Ronald's Mac) far terraces showed rows of dirt dashes, one block of grass over dirt per 2- or 4-block step.
- **A dithered fade where the cubes end** (branch lod-fade). The world renderer draws the cube meshes near the view's end with the pack's `fs_opaque_fade`/`fs_cutout_fade`/`fs_translucent_fade` (six more pipelines, made only when the pack has them; the translucent ones since lod-polish, so near water fades too), which dither pixels away over the last 24 blocks; `fs_lod` draws exactly those pixels; meshes past the end aren't drawn (the view's square corners). `Globals.extra.y` carries where the cubes end. *Why:* Ronald chose all three LOD fixes on 11-05; the cube edge showed as a hard line (reports/lod.md:89, Distant Horizons' fade).
- **Far tiles load faster and stand in one level coarser** (branch lod-loading, `lod::pool`, `Lod::request`, `Lod::update`). LOD's own pool has a third of the cores (3 to 6, was 3) at the near-cube pool's low priority (half the cores at normal priority starved the near cubes: 7 s to come in after a teleport, not 1.5); tiles are asked for nearest first in tile widths (coarse before fine at the same reach, the land by the camera before the horizon), each missing tile's parent before it, so a stand-in hides at most its three siblings. Inside `terrain::one_column` (distant terrain's face calls) `Terrain::surface_block` works out only the column asked for unless its cube's are cached (it computed all 256 of the cube column, 1-2 ms, for each far steep cell the dimension's faces asked about: most of a far tile's meshing); elsewhere it is as before (doing that everywhere slowed the near cubes' scattered things, 1.5 to 4-5 s after a teleport). The `load` log times each tile's heights, mesh, face calls and wait, and says how long until every wanted tile was made. *Why:* Research round 16: the far boxes were slow loading (about 20 tiles a second of the 800 a view wants, CPU-bound), and a missing tile's coarse ancestor hid every finished tile under it.
- **Scatters up to 96 blocks high** (3cfc8a9c). *Why:* giant sequoias didn't fit the old cap.
- **A continent scale per dimension** (587208ff, f081de3b). *Why:* Ronald picked 1800-block land masses for the overworld (the default is 900).
- **Blocks that stand in still liquid, `holds_liquid`** (c1ee7fe0, cbbeeb9b). *Why:* water plants, kelp and lily pads need water in the same block.

## 4. Gameplay the engine had to allow

- **Riding** (2ab12f97, 868b4fad), **a size per entity** (f3d8a0fe), **skins painted by mods** (92315d8d), **up to 12 drawn layers** (d1c06be7, for the cloak), **the inventory's 3D preview for any entity** (7bc57d30). *Why:* the animals mod: rideable raptors and wyverns, per-animal coats, the animal screen's portrait.
- **Held items bigger in the own first-person hand** (`comp::FIRST_PERSON_HELD_SCALE` on an entity kind, `f32`, 1 without it; 7295827c): `place_carried` scales the display of items in the player's own first-person hand about where they're held; base sets it on `core:player`. *Why:* after the Weft sync the items sit in Weft's posed hand, smaller than our old corner; Ronald kept Weft's hand but wanted "our items a bit bigger". **Upstream?** Maybe (a game-wide knob).
- **Entity send and activation ranges** (249afe81). *Why:* far animals cost the server too much (Research round 2, E1).
- **Items:** a stack's own tool numbers win over its item's (a9fcb688), items that keep when broken (4e03dc83), a stack's look from layers (988c4077). *Why:* tools built from parts (crafting).
- **A player's break drops at their feet** (8cb72890, 6eaf25b5). *Why:* chopped logs landed out of reach (first-day test).
- **A time-of-day rate per dimension, sent to clients** (1e138839, protocol 11 to 12). *Why:* Ronald's slow first day, starting at 10:00.
- **A moving structure can lean** (416f25cf, protocol 12 to 13): `World::body_turn(entity, pivot, rot, ticks)` gives a body a pose, a pivot and a rotation the clients ease to (`ToClient::BodyLean`); `world.wgsl` (the engine's and the base pack's) turns its vertices and the pack its normals (`offsets` is three vec4 a structure: offset and light, rotation, pivot); a leaning body is only drawn: the server's physics and picking and the client's collision and raycast skip it. *Why:* Ronald, 11-05: trees "fall down sideways and like the leaves to sort of start breaking while it's falling": the tree's own blocks, lifted (`World::lift`), lean over the stump and shed leaves as they go (`mods/base/src/fall.rs`). Ronald chose this over a box-model tree ("Yes, the real tree falls").
- **A leaning structure casts its shadow** (70f1b361): the sun's shadow pass draws leaning structures (`WorldRenderer::draw_shadow` takes them with their pose; its offsets as `draw`'s), turned like their blocks. *Why:* a falling tree threw no shadow (Planner's follow-up, 11-05; Lighting agreed the pack needs nothing, its shadow vertex goes through `finish()`). Other moving structures still cast none.
- **The free camera on F6** (`input.rs`, `core:freecam`; Weft has P). *Why:* P is the base game's Stats screen, and the two on one key opened Stats instead of the camera (Ronald, 11-04: freecam to F6, P stays Stats). Camera keys don't fire while F3 is held (`client/camera.rs`), so F3+F6 only opens the debug options.
- **A world never takes a nameless player** (dc1d8b7b); **a headless server has its own identity** (432889c9). *Why:* bugs found in multiplayer checks.

## 5. UI the engine had to allow

- **Pixel fonts on the UI grid; placement on the pixel grid** (7efecb0d, 818614e8, b0976672), **`at` placement** (b2877719), **UI kit 2's shared parts** (2b35d9f9), **pressed and selected offsets, the label shadow straight down, labels centred even when taller than their room** (5e38fcae, 650ea4c9, 83863490, be22b7f6). *Why:* Ronald's Valheim-style UI, "text position, vertical and sizing perfect".
- **GUI scale even only, an auto scale that fits, small windows scroll** (22960101, b94cf94b, 361bea1f, 7376e8e2). *Why:* at scale 1 text drew twice its box size (playtest B9), and small screens cut menus off.
- **Video and Quality presets** (bb03aa57). *Why:* Ronald's video settings screen.
- **Text shrunk to fit says so in the log** (4765e3b2), **a missing block texture stops the game at startup** (e2d63707, 3ff06554). *Why:* catch squeezed Russian text and silent magenta early.
- **Russian:** the Cyrillic glyphs lifted, then drawn for our main font (a5351840, cyrillic branch), 100 engine strings translated (c902d251), plural forms (bb4de9b4). *Why:* Russian is our second language.
- **Smooth text: Alegreya Sans** (0e1ba79e); **slots with frames** (06bd9414); **the title screen in our theme** (e0edc189); **Select World as a list** (bf9427c5). *Why:* the UI redo.
- **Chat fits small windows; no crosshair over a screen** (claude/audit-rest, merged a0fc9a41; `hud.rs`, `game_screens.rs`). *Why:* at 760x600 the chat covered the Inventory/Recipes/Stats buttons (B9c), and the crosshair stood over the sleeping screen.
- **Block outline thin and faint** (look-outline ba37dc6d, merged 9a5398eb): `client/entity_render.rs` draws the selection outline's 12 edge boxes see-through (alpha `OUTLINE_DARK` 0.4) in the overlay pass, each sized to `OUTLINE_PX` 1.25 screen pixels from its distance, the camera's fov and the frame height, instead of solid black boxes 0.012 blocks thick. *Why:* Ronald: "thin and faint like vanilla"; the old frame was about 3 px solid black.
- **Mods' screen text can wrap** (claude/hint-wrap; `app.rs`, `app/api.rs`, sdk `Client::screen_text_wrapped`): a screen label may carry a width it wraps to, its lines centred; zero keeps one line. *Why:* the base game's long hint ("Press [E]: logs make planks…") ran over the HUD's top-left column in a narrow window (760x600).
- **…and sit on the HUD's plate** (claude/hint-plate; `app.rs`, `app/api.rs`, sdk `Client::screen_text_plated`): a negative wrap width draws the label inside a `hud_plate` panel (no new host function). *Why:* the first hint read as "a thin small line mid-screen" (World playtest round 13).
- **A UI preview of an entity kind alone** (claude/journal-ui; `client/modui.rs`, `client/entity_render.rs`, `app.rs`, sdk `Ui::kind_model`): a 3D preview node may name an entity kind (`kind:<name>`), drawn with its kind's own texture (loaded for it, as none may be in the world) and no entity behind it, turning with the mouse like the player's figure. *Why:* the Field Journal's page shows each animal turning, also ones not in the world (Ronald, 11-05: "Yes, turning 3D").
- **The GPU's kind and memory for mods** (claude/gpu-preset; render `Backend::device_class`/`device_memory_mb`, the Vulkan backend's device type and largest device-local heap, `config.rs`, `app.rs`): put at start into `core:gpu_name`, `core:gpu_class` (1 integrated, 2 discrete, 3 virtual or CPU) and `core:gpu_memory_mb`, never saved by a player. *Why:* the base game picks its first quality preset by the GPU (Research round 19, 7: integrated GPUs got unplayable defaults).

## 6. Sound

- **Handles, loops, a voice cap and safe file reading; weather sound** (415a0fe8); **block groups by data, voices for every mob and 42 species, load-time checks** (9871a15c); **sounds for 7 materials and the UI** (83019cb9); **`/sounds recent`** (869ce80c). *Why:* the game had almost no sound, and testers needed to see what played.

## 7. Our project's phases (the network and content layer)

These are the biggest changes. They live mostly in our crates, but each needed hooks in the engine.

- **Account (phase 1):** the network key follows the account's 24 words; joining proves the account (170fff57, 308a0544).
- **Data packages (phase 2):** local packages, package identity, switching content between worlds without a restart, downloading a world's packages on join, signing and trust (d2b75419 to 91f907c3).
- **Linked worlds (phase 3):** your world runs while you visit others, every world has its own address, link requests, portal blocks, travel, invites, friends (94a0f394 to b15162ec).
- **Admission (phase 4):** admission rules, bans and shared ban lists (bd9b4aa9 onward).
- **Finding worlds (phase 5):** Find Worlds lists other players' worlds and joins the online ones (65045b6e, a774a47e).
- **Sandboxed mods (phase 6):** wasmtime components for server halves, API 1.1 to 1.3, client halves, personal mods, write caps and call cost (ed5ff455 to 0276a52c).
- *Why:* PLAN.md's goal: a player-owned network of worlds, with mods that can't harm players.

## 8. Test kit and settings for runs

- **Settings set for a run hold for the whole run** (2fa0beff), **and in the worlds it opens** (b091ac8b), **`--config` and `--no-shadows`** (ee147880). *Why:* scripted PC looks and benches need fixed settings.
- **The network path in the log** (0a11b0ee): `net: path to <id> relay|direct|mixed <kind>` on every join, portal arrival and player coming in, and again on each change. **A relay-only test setting** (`core:net_relay_only`, on no screen): the endpoint gets no IP transport, so every connection goes through n0's relay. *Why:* Ronald asked whether joins really cross the network, and every two-machine check ran on one home network.

- **Load-test hooks for many players** (Research round 18, branch `claude/jovial-einstein-158b8f`):
  - `net/p2p.rs`: `ALPN`, `PROTOCOL`, `MAX_FRAME`, `MAX_HELLO`, `MAX_OFFER`, `write_frame` and `read_frame_max` are made public;
  - `diag::take_net` reads the traffic counters without a frame report;
  - `weft --server --stats [secs]` logs a stats line: ticks per second, tick avg/p50/p99/max and phases, players, cubes, entities, traffic by message kind each way (with `--log net=debug`), and memory, threads and CPU by thread name from `/proc` (`dedicated.rs`, `Shared::stats` and `tick_samples` in `server/mod.rs`).

  *Why:* our bot client (`crates/bots`, `weft-bots`) joins over the real protocol by the hundred, and the server had no tick timing without a game window. Off unless asked for. *Upstream?* Maybe the stats line.

## 9. Removed

- **The texture pack built from a player's Minecraft copy** (159b507c). *Why:* Mojang's assets must not ship or be generated from; Weft's AI_POLICY also forbids "generated look-alikes".

## What this means for syncing with upstream

- **Synced with Weft v26.11.2** (upstream `2db18c23`) on `sync/weft-26.11.2`, merge `e9c8405c`
  (plan: `notes/crew/upstream/sync-plan.md`, Ronald's calls 1-6). Upstream's engine is the base;
  what stays ours on top of it:
  - **Terrain:** CPU callback features beside upstream's expression features
    (`FeatureDesc.custom`, `Feature::custom`, `FeatureArgs.continent_scale`), `FACES` (16),
    `CONTINENT_SCALE` (17), scatter `thin` as an expression mask; the GPU cube path takes the CPU
    features' heights (`gpu_custom`, `custom_cache`).
  - **Distant terrain:** our smooth tiles, wall bands and faces, with upstream's saved columns.
  - **Weather:** upstream's weather ids, with our rain by biome on them (`set_rain(biomes,
    weather)`, `weather_in`, `ToClient::Weather { id, only }`), air, tints and the `/weather`
    override; packs' rain/thunder inputs come from the weather's look.
  - **Kept as before:** sun and moon shadows (`mesh_draws` with all layers for the shadow passes,
    `table_only` meshes cast too), painted skins, bone builds, mob shadows, day rate, drops at the
    feet, felling's leaning bodies (`body_turn`), probation and the chiacraft layers, protocol 14.
  - **Taken from upstream instead of ours:** waterlogging (cell parts; our `holds_liquid` is
    gone), riding (seats, steering), camera modes (`camera_flags` for the crosshair, the hand and
    one's own shadow), the Vulkan upload fixes, mining on the server (our broken-tool rule is in
    `mining_rate`, keep-broken in `stack::worn`).
  - **Fixed in upstream's code (PR candidates):** `sdk/ai.rs` mob moods cleared when another
    world opens (they were process-wide by entity id and tick, so a zombie's anger landed on the
    next world's spider: `neutral_mobs`); particles' spawn on a cell not arrived yet; first person
    cutting worn layers by bone mask; a ridden flyer's flight counted as a fall (MountMove `flies`).
- The bug fixes in section 1 are the natural PRs. Upstream already has its own versions of #83 and #84. The clean branches for #96 to #98 and `/rule` are in `notes/crew/upstream/for-ronald.md`.
- Everything else is ours to keep in the fork unless Ronald decides to offer it.

## Our own crates (additions, not core changes)

`sandbox` (wasmtime mod host), `packages` (data packages), `account`, `bridge`, `bots` (`weft-bots`, headless load-test players).
