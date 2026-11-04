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
| HUD: crosshair only in first person (6a394c0d) | It sat on the player's back in third person | #98 closed for now; clean branch ready |
| `/rule` reads values the way the rule takes them (a2f4c81c) | `false` was refused, `2` and `999` clamped silently | Clean branch ready, never opened |
| Box-model blocks' far faces use their boxes' textures (e1f48a48) | Magenta in distant terrain | Maybe |
| Setting values follow their names on a content switch; ids kept (c4a527ea, c555d7ec, 3ec8ef9c) | Switching a world's packages scrambled settings and broke an empty-name join | Maybe |
| A content switch registers the keys its content brings (47a8be49) | New keys were missing after a switch | Maybe |
| Under water the sky's gradient is drawn (2562a0cd) | Navy streaks where the sea floor ends | Maybe |
| Sky pictures upright on a leaned sun path (81ff2dcc) | The moon drew as a tilted diamond | With the leaned path only |
| clippy allows for `mut_from_ref` (0d111b9a, d7ff2255) | A newer clippy made them errors | Trivial |

## 2. Rendering, for our shader pack (Bliss-like look)

Weft's shader packs could only replace shaders. Our pack needed data the engine didn't give.

- **Sun shadows:** a shadow map from the sun for packs (48d38929), from mobs and the player (cc34e2dc), the faces the light falls on (5a096e0f), a coarse far cascade to 400 blocks (f03b29b6), distant terrain casting (c4698cc5), cloud shadows (6b63b917), and a sun path leaned 35° south (5daa6e72). *Why:* real shadows are the core of the look.
- **Depth for screen effects** (3f4035e2), **water that bends and tints what's under it** (df721f33), **light shafts, with fog kept out of caves** (4a875f7e). *Why:* water reflections, contact shadows and shafts need the depth buffer and a copy of the world.
- **Pack options, the "look knobs"** (e2c8496e): a pack names up to 16 settings and the engine passes them to its shaders. *Why:* Ronald wanted the look tunable without rebuilding.
- **Held light** (5ae27f4f, 3cdfa80d, b752319f, 3e5ce4b8): a torch in hand lights the way. **The night follows the moon** (fc1478ca), **the night sky** (104c0e39). *Why:* nights are dark by design, so the torch and moon must carry them.
- **Clouds follow the weather and the wind** (5a42062b, 1bae91a3), **biome tints a mod can change while running** (34432291, used for seasons and rain). *Why:* the living-world systems (seasons, rain by place) have to show.
- **Canopy light** (e40f323e, on look-canopy): a biome's air flags carry how closed its canopy is (`weft_abi::air_flags::CANOPY_SHIFT`, bits 16-23, 0-255). The client eases it over 10 s into a new `Scene.canopy` field, the same path the aurora takes. *Why:* the tropical forest's shade and sunflecks under closed crowns.
- **Performance:** render scale (519dbf32, 14a00a37), a pipeline cache on disk (b1867503), a bounded, depth-only shadow pass (409d70ed), screen effects in one pass (716acfd8, ce011d36), the water's grid only when changed (bdb745ef), small frame-thread steps in place (ac4995f3), the frame thread culling its own draws (c31f6d34), skin painters off the frame thread (1041e730). *Why:* Ronald's Mac lagged at 3456x2234; these took it from about 96-124 fps to 128-154 fps uncapped.
- **Diagnostics:** GPU time per pass in benches (ee147880), slow frames name their step (2394267c), where "submit" goes each second (9ab54ee5, bf07e118), the size the world draws at (ce993538), PACING lines (dcde2415). *Why:* to find the lag and the hitches.

## 3. Distant terrain (LOD) and world generation hooks

- **Trees per site, rock on steep faces, stable levels; smooth, morphing far ground; skirts under edges; a tall crown tapers** (747f25bc, 87aa8332, e2c0d171, 0399162b). *Why:* distant forests and mountains looked blocky or wrong, and giant sequoias showed flat-topped.
- **Far trees as crowns over trunks** (branch lod-crowns). Each scatter's shape is sampled 16 times per biome and 8 of the runs kept as crowns (radius, top, lowest leaves, widest layer, how round its top is, trunk width and block, leaves); each far site takes one by its hash and lays it over the cells round or tapering, over the real ground, with a trunk at levels 1-2. A sample comes back as 4 words, not 2, and the LOD compute pass uses 18.5 KB of workgroup memory (was 13.9; Apple GPUs allow 32). The old `TALL_CROWN` taper is gone, the profile covers it. *Why:* Ronald, 11-05: "why the lods still boxes ?": far forests were flat-topped leaf pillars down to the ground, one median tree per scatter.
- **Scatters up to 96 blocks high** (3cfc8a9c). *Why:* giant sequoias didn't fit the old cap.
- **A continent scale per dimension** (587208ff, f081de3b). *Why:* Ronald picked 1800-block land masses for the overworld (the default is 900).
- **Blocks that stand in still liquid, `holds_liquid`** (c1ee7fe0, cbbeeb9b). *Why:* water plants, kelp and lily pads need water in the same block.

## 4. Gameplay the engine had to allow

- **Riding** (2ab12f97, 868b4fad), **a size per entity** (f3d8a0fe), **skins painted by mods** (92315d8d), **up to 12 drawn layers** (d1c06be7, for the cloak), **the inventory's 3D preview for any entity** (7bc57d30). *Why:* the animals mod: rideable raptors and wyverns, per-animal coats, the animal screen's portrait.
- **Entity send and activation ranges** (249afe81). *Why:* far animals cost the server too much (Research round 2, E1).
- **Items:** a stack's own tool numbers win over its item's (a9fcb688), items that keep when broken (4e03dc83), a stack's look from layers (988c4077). *Why:* tools built from parts (crafting).
- **A player's break drops at their feet** (8cb72890, 6eaf25b5). *Why:* chopped logs landed out of reach (first-day test).
- **A time-of-day rate per dimension, sent to clients** (1e138839, protocol 11 to 12). *Why:* Ronald's slow first day, starting at 10:00.
- **A world never takes a nameless player** (dc1d8b7b); **a headless server has its own identity** (432889c9). *Why:* bugs found in multiplayer checks.

## 5. UI the engine had to allow

- **Pixel fonts on the UI grid; placement on the pixel grid** (7efecb0d, 818614e8, b0976672), **`at` placement** (b2877719), **UI kit 2's shared parts** (2b35d9f9), **pressed and selected offsets, the label shadow straight down, labels centred even when taller than their room** (5e38fcae, 650ea4c9, 83863490, be22b7f6). *Why:* Ronald's Valheim-style UI, "text position, vertical and sizing perfect".
- **GUI scale even only, an auto scale that fits, small windows scroll** (22960101, b94cf94b, 361bea1f, 7376e8e2). *Why:* at scale 1 text drew twice its box size (playtest B9), and small screens cut menus off.
- **Video and Quality presets** (bb03aa57). *Why:* Ronald's video settings screen.
- **Text shrunk to fit says so in the log** (4765e3b2), **a missing block texture stops the game at startup** (e2d63707, 3ff06554). *Why:* catch squeezed Russian text and silent magenta early.
- **Russian:** the Cyrillic glyphs lifted, then drawn for our main font (a5351840, cyrillic branch), 100 engine strings translated (c902d251), plural forms (bb4de9b4). *Why:* Russian is our second language.
- **Smooth text: Alegreya Sans** (0e1ba79e); **slots with frames** (06bd9414); **the title screen in our theme** (e0edc189); **Select World as a list** (bf9427c5). *Why:* the UI redo.

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

## 9. Removed

- **The texture pack built from a player's Minecraft copy** (159b507c). *Why:* Mojang's assets must not ship or be generated from; Weft's AI_POLICY also forbids "generated look-alikes".

## What this means for syncing with upstream

- Upstream (VortexSo4/Weft master 10fdced5) is 173 commits ahead of our base. The next sync will conflict where we both changed the same code: Vulkan uploads (take upstream's versions), item stacks and icons (upstream now has "stack values over item values" and layered item icons, close to ours), and anything in `server/dim.rs`, `config.rs` and the UI.
- The bug fixes in section 1 are the natural PRs. Upstream already has its own versions of #83 and #84. The clean branches for #96 to #98 and `/rule` are in `notes/crew/upstream/for-ronald.md`.
- Everything else is ours to keep in the fork unless Ronald decides to offer it.

## Our own crates (additions, not core changes)

`sandbox` (wasmtime mod host), `packages` (data packages), `account`, `bridge`.
