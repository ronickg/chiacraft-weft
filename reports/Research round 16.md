# Research round 16: where the frame goes, and what would make it faster

Researcher, 2026-11-05, for Planner, Lighting and Distant terrain. Ronald asked: "get a resercher to check our currnet shader code and lod stuff and check for speed improvements". Later, relayed by Planner: "u can also check out voxy and distant horzions code and also horiztion the rust mc ports code for some ideas as well".

**What was measured:** dev `b5815f09` (line references checked again on dev `5b239f9d`) plus one testkit verb (below), on the PC (RTX 5090) at 1280×720, 1920×1080 and 3456×2234 (Ronald's window), with the chiacraft pack, uncapped (vsync off: 300–2400 fps, so the GPU stays clocked up).

**What was read:**
- the pack's shaders (`mods/base/shaderpack/*.wgsl`) and the post effects (`mods/base/assets/shaders/post/*.wgsl`);
- `post.rs` and `post.wgsl`;
- the shadow passes (`client/mod.rs`, `world_render.rs`);
- distant terrain (`lod.rs`, `terrain_gpu.rs`, `gpu_layout.rs`, `fs_lod`);
- the source of Voxy, Distant Horizons and Horizon (that section is in [research-16/sources.md](research-16/sources.md)).

**Files:** raw bench lines in `reports/research-16/pc/` and the parser in `reports/research-16/parse.py`. Scripts in `~/Coding/chiacraft-archive/tester-tools/scripts/r15_*_2026-11-05.txt`, shots in `~/Coding/chiacraft-archive/images/researcher/2026-11-05/r15/` (`r15_scenes_3456.jpg` is the five views). Those archive files keep the `r15_` prefix they were made with: this round was drafted as 15 and renumbered when the weather Researcher's report took that number.

## The short answer

1. **A GPU hang, found by accident: changing Render scale during a game kills the GPU.**
   - Twice on the PC, 1.0 → 0.5 mid-run gave `ERROR_DEVICE_LOST` 5 s later, with `NVRM Xid 109 CTX SWITCH TIMEOUT` (19:07:41 at 720p, 19:16:26 at 1080p).
   - The Video screen offers this setting (`mods/base/src/video.rs:119`), and on Ronald's Mac a hang can take the machine down.
   - Planner was told at once. This is fix #0 below.
2. **On the Mac the frame is about 5–7 ms of GPU in clear air** (estimated from PC numbers, checked against Tester's Mac row). **In mist it is about 13–15 ms**: the fog march (12 steps of noise and a shadow tap on every pixel) adds about 7.6 ms. That is the largest cost in the game, and it would make misty mornings run under 60 fps.
   - In clear air the biggest single piece is **the light shafts while the sun is on screen: about 2.3 ms** (3 under a canopy).
   - Under trees, **contact shadows cost about 2.2 ms** for their 2–4% darkening.
   - Then come the opaque world (about 2.2 ms), sky, clouds and water together (0.5–1.4), volumetric light (0.6–0.8), bloom (0.4–0.6), the full-screen effects pass and water copy (about 0.55), reflections over water (about 0.5) and shadows (0.75–0.9).
   - No effect is expensive on the PC at 1080p: all five together cost 0.18–0.44 ms.
3. **Distant terrain's "boxes" are slow loading, not slow drawing.**
   - Tiles arrive at about 20 a second after a first burst of about 240, at 720p and at 3456×2234 alike, so the limit is the CPU, not the GPU. The view wants 801 tiles.
   - While a finer tile is missing, its coarse parent box covers its whole area, hiding every finished child under it (`lod.rs:658-695`).
   - Drawing distant terrain costs about 0.1 ms on the PC.
4. **The first fixes are cheap and in our mods:**
   - fewer mist steps;
   - shafts and contact on Low by default;
   - two bloom rings;
   - plus, in the engine, LOD with more mesh threads and coarse-first order, and one timing mark so the next round can see the water copy and clouds apart from the effects.

## How it was measured

- **A new testkit verb, `set_config <name> <value>`** (`mods/testkit/src/script.rs`, branch `claude/jovial-einstein-158b8f`, 73ee15af + d7d7dc0f), changes any setting mid-script. One run then benches each effect on and off over the same loaded world, so a difference is that effect's own cost, not run-to-run noise.
- **Each view is benched as follows** (`r15_fx_ab`): all on, then each of reflect, contact, volume, shafts and bloom off in turn, then all effects off, near shadow map only, no shadows, and all on again (the drift check). Each bench is 4 s.
- **Five views on seed 1:** ground (open land with water), sun (morning, looking at the sun), sea (a river through jungle), forest (from above), far (hills, distant terrain).
- **Noise.** Drift between the first and last "all on" was ±0.01 ms (±0.04 at 3456 looking at the sun). At 3456 the single-effect differences scatter by about ±0.15 ms (contact and shafts read −0.1 to −0.17 in views where they early-out), so differences under about 0.15 ms there are not real.
- **The engine's GPU parts are misnamed with our pack.**
  - With `water_behind`, the world pass is split for the copy, and everything after the copy goes in the "effects" part: the copy, sky, clouds, water, particles, then the effects. "translucent" reads 0.00 (the Mark at `client/mod.rs:2691` and `:2781`, with `late_out` moved after it).
  - The no-effects rows show what this hides: the sky, clouds and water alone are 0.21–0.61 ms at 3456 ("translucent" when post is off).
  - So the Mac's earlier "effects 10–14 ms after a flight over water" (BIG_PERF) included clouds and water too. Fix M1 below gives them their own part.
- **Mac estimate.**
  - Tester's Mac row at 3456×2234 with the auto render scale of 0.67 (dev 00bae2ee, `lag-big-devtip2-nov.sum`, "stand"): world 2.27 ms, effects part 3.39 ms, shadows 0.77 ms.
  - The PC at 3456 full scale scaled to 0.67's pixel count (×0.449) gives world 0.45 and effects 0.68. The Mac is about **5× the PC per pixel** (5.1 world, 5.0 effects; the views differ, so ±50%).
  - **Mac ms at Ronald's auto scale ≈ PC 3456 ms × 2.25; at scale 1.0, × 5.** Shadows don't follow pixels: the Mac's 0.77 ms shadow part is 10× the PC's 0.08.

## Where the frame goes now

PC, RTX 5090, 3456×2234 at full scale, ms of GPU per frame. "Mac" is the estimate at the auto render scale (×2.25). Each pass's cost is measured by turning it off (the difference).

| part | ground | sun | sea | forest | far | Mac (typical) |
|---|---|---|---|---|---|---|
| **whole frame** | 2.61 | 3.22 | 2.66 | 2.04 | 1.81 | 4–7 (measured 6.45 stand) |
| shadow pass (both maps, drawing) | 0.08 | 0.08 | 0.08 | 0.04 | 0.08 | 0.77 (measured) |
| opaque world (cubes, LOD tiles, entities; shading with 16–32 shadow taps) | 1.00 | 0.80 | 0.93 | 1.15 | 0.89 | about 2.2 |
| sky, clouds, water, particles (after the copy) | 0.61 | 0.48 | 0.56 | 0.22 | 0.21 | 0.5–1.4 |
| water copy plus the effects pass itself (all on, minus the effects' own costs) | about 0.25 | | about 0.25 | | | about 0.55 |
| reflect | 0.23 | 0.20 | 0.21 | 0.02 | 0.01 | 0.45–0.5 over water |
| contact | ≈0 | 0.09 | 0.16 | ≈0 | ≈0 | ≤0.35 |
| volume (Low, 8 steps, by day) | 0.26 | 0.37 | 0.25 | 0.16 | 0.11 | 0.25–0.8 |
| shafts (only with the sun on or near the screen) | ≈0 | **1.02** | ≈0 | ≈0 | ≈0 | **2.3 facing the sun** |
| bloom (every pixel, 24 taps) | 0.18 | 0.26 | 0.22 | 0.17 | 0.19 | 0.4–0.6 |
| all five effects off (removes the effects pass and the copy) | 0.93 | 1.84 | 1.07 | 0.65 | 0.64 | 1.5–4.1 |
| shadows off (pass plus all sampling) | 0.36 | 0.41 | 0.41 | 0.33 | 0.41 | 0.8–0.9 |
| of which the far map (near only instead of both) | 0.09 | 0.02 | 0.09 | 0.09 | 0.10 | |

The same at 1920×1080: whole frame 0.76–1.07 ms; all effects 0.18–0.37 (shafts 0.15 toward the sun, volume 0.04–0.10, bloom 0.03–0.05, reflect 0.02–0.06); shadows 0.14–0.19. At 1280×720: whole frame 0.51–0.69 ms, all effects 0.09–0.17, shadows 0.09–0.13. Full tables: `python3 reports/research-16/parse.py reports/research-16/pc/*.txt`.

Bad weather and a moving camera are in their own sections below.

## Ranked speed fixes

Gains are Mac ms at the auto render scale unless marked. "Engine" means `crates/` (Weft's): it needs a `notes/ENGINE_CHANGES.md` line in the same branch. "After sync" means better landed after the upstream Weft sync (which reworks `mesh_draws` culling, riding and camera modes).

| # | fix | gain (Mac) | risk to the look | files | where | owner | when |
|---|---|---|---|---|---|---|---|
| **0** | **Render scale changed in-game hangs the GPU.** Reproduced twice on the PC (Xid 109). Until it's fixed: change it only before starting, or have the Video screen apply it at the next start. | not speed: stops a Mac-killing hang | none | `crates/engine/src/app.rs` (`world_frame`, `make_depth`, `record_passes`), `post.rs` (`scene_target`, `behind_targets`); `mods/base/src/video.rs:119` for the stop-gap | engine (+ mod stop-gap) | the render-scale owner (`claude/render-scale`, Lag fixes; Planner routes) | now |
| 1 | **Cheaper mist march.** `fog_through` is 12 steps × (3 `noise3` + 1 shadow tap) on every fragment and sky pixel while there is mist (`look.wgsl:524-654`). (a) Now, in the pack: 6–8 steps (the jitter already hides banding), the 15-block octave only in the near half, the shadow tap every other step. (b) Later: the march as a half-resolution screen pass from the depth, so hidden fragments and overdraw stop paying, with a depth-aware upsample. | (a) about 3 ms, (b) about 6 ms of the +7.6 in mist | (a) low: grainier shafts in mist; (b) medium: fog over water and particles needs care | (a) `mods/base/shaderpack/look.wgsl`; (b) also `post.rs`, a new post effect | (a) pack; (b) engine + pack | Lighting (it is retuning weather now; research-weather-fx has these numbers) | (a) now, (b) after sync |
| 2 | **Shafts on Low by default** (`FX_SHAFTS` 2 → 1: 20 steps instead of 40, the Low fade 0.912 already tuned to match) | about 1.1 ms facing a low sun (half of 2.3), about 1.5 under a canopy | low: the jitter hides the steps, Low already exists | `mods/base/src/video.rs:213` | mod | Lighting | now |
| 3 | **Contact shadows on Low (or off) by default, and a shorter reach.** Ronald chose them gentle, 2–4% darkening (contact.wgsl header); they cost ≈0 in the open but **1.0 ms on the PC under a canopy** (about 2.2 on the Mac), where every pixel is within 64 blocks. Low halves the directions; ending at 40 blocks (the fade is mostly done there) cuts more. | 1.1–1.6 ms under trees, ≈0 in the open | small (his chosen strength is kept; Low is "as dark, from half the directions") | `mods/base/src/video.rs:209`, `post/contact.wgsl:49, 86` | mod + pack | Lighting (ask Ronald if turning off) | now |
| 4 | **LOD meshing: more threads at normal priority, coarse levels first, one level of tolerance** (Distant Horizons: 50% of cores, meshing highest, coarse-first; Horizon: tuned sampler and mesher split) | not ms: the far "boxes" clear in seconds instead of 20–40 s | none | `lod.rs:108-118` (pool), `pick` (sort key), `update` (stand-in rule) | engine | Distant terrain | now (independent of the sync) |
| 5 | **Shafts at quarter resolution with a bilinear upsample** (a pass of its own: a radial blur toward the sun is low-frequency), or at least skip the colour sample where the depth says not sky | about 2 ms facing the sun on top of #2's Low (85–90% of what is left); the skip alone about 0.5 | low (a soft blur anyway); halos at sharp silhouettes at quarter scale | `post.rs` (a lower-resolution target and pass), `post.wgsl`, `post/shafts.wgsl` | engine + pack | Lighting, with an engine hand | after sync |
| 6 | **Bloom from a small mip chain** (downsample the scene 3–4 times, sample 8 taps from mips 2–4) instead of 24 full-resolution taps on every pixel. Interim in the pack: two rings (16 taps) | 0.3–0.5 ms always (interim about 0.15–0.2) | low | `post.rs` (mips for the scene image), `post/bloom.wgsl` | engine (pack for the interim) | Lighting | interim now, chain after sync |
| 7 | **Volumetric light at half resolution** with a depth-aware upsample (Photon and Unity do this) | 0.3–0.4 ms by day | low–medium: beam edges soften | `post.rs`, `post/volume.wgsl` | engine + pack | Lighting | after sync |
| 8 | **The far shadow map anchored in world space**, so moving doesn't redraw it. Its box holds a sub-texel camera offset (`light_box`, `client/mod.rs:2842-2869`), so the key changes every moving frame. Also every mesh or LOD upload bumps `terrain_epoch`. | about 0.6 ms of GPU plus the CPU of about 4,500 draw records a frame, whenever the player walks or terrain loads (PC: +0.14 ms, 2,808 → 7,380 draws) | none if anchored to whole texels | `client/mod.rs:2611-2618, 2842-2869`, `world_render.rs` `terrain_epoch` | engine | Lighting (shadows) | after sync (touches `draw_shadow`) |
| 9 | **Partial stand-ins for missing LOD tiles, and keeping the old mesh until the new one is in** (Horizon's backdrop tile, Distant Horizons' swap) | not ms: no box area hiding finished tiles | none | `lod.rs:658-695` | engine | Distant terrain | now |
| 10 | **Water copy only when water is on screen**: `bc212cab` (claude/behind-copy, not merged). The clouds then march without the copied depth, so pair it with a cloud depth test (no `frag_depth` write, or test the depth buffer before marching). | up to about 0.5 ms in dry views, plus a pass split on Apple | medium (clouds behind hills) | `app.rs:2895-2937`, `post.rs` `copy_behind`, `sky.wgsl:110, 174-189` | engine + pack | Lighting + engine | after sync |
| 11 | **Per-tile camera-facing face ranges for LOD** (Voxy's cmdgen; our cubes already do this) | about half the LOD vertex work; LOD drawing is small on the PC, so about 0.1 ms on the Mac (unmeasured) | none | `lod.rs` (`mesh`: faces by direction), `world_render.rs` (LOD records) | engine | Distant terrain | after sync |
| 12 | **No `discard` in the fade band and `fs_lod`** (a separate LOD target with an alpha fade, or the cubes' depth as a clip) | unknown: Apple's hidden-surface removal is lost on the about 40% of near meshes the fade band draws with discard; not measurable on the PC | low | `world_render.rs:1357-1377`, `world.wgsl:755-785, 889-892` | engine + pack | Distant terrain + Lighting | after sync, measure on the Mac first |
| 13 | **Two-phase depth occlusion for cubes and tiles, and cave-connectivity culling for drawing** (Horizon: 15–30% fewer indices at RD12, 1.6 ms at RD32 on an M4 Pro; round 12 #6). A depth pre-pass is the cheaper cousin: the world's shading, mist included, runs on hidden fragments today. | 0.2–0.5 ms in hills, more underground and in mist (unmeasured) | none | `world_render.rs` culling, `mesh_draws`, `app.rs:2914` | engine | engine session | after sync (the sync reworks culling) |

Measurement fixes (small, engine, now):
- **M1.** A `Cmd::Mark` after `copy_behind` and one after `late_out` (`app.rs:2930-2937`), so the copy, the late pass (sky, clouds, water) and the effects each get their own part. Today "effects" holds all three.
- **M2.** LOD timing in the `load` log: microseconds per tile for the GPU sample, the custom feature heights (`custom_heights`, on the shared `pool()`) and the mesh, like the terrain's "us each on a worker" line. That shows where the 20 tiles a second go.

**Top 5 for Planner:** #0 the render-scale hang; #1 a cheaper mist march (about 3 ms on misty mornings, pack only); #2 shafts on Low (about 1.1 ms toward the sun); #3 contact on Low under trees (about 1.1 ms); #4 LOD threads and coarse-first order (the far boxes).

## Shadows: how they are drawn, and what they cost

- **Two maps:**
  - near: 2048², ±80 blocks, 400 deep, every frame;
  - far: 2048², ±400 blocks, 1200 deep, kept while its key stands still (`client/mod.rs:2570-2633, 2809-2810, 2870-2871`).
- **Culling.** Regions, then cubes, are culled against the light box's 6 planes (`world_render.rs:725-746`), and faces facing away from the light are dropped. Opaque faces draw with no fragment shader; cutout (leaves) uses `fs_shadow`, a texture read plus discard. Entities go into the near map only.
- **Sampling.** Every world pixel does 16 `textureLoad`s (a 4×4 read for a 3×3 PCF); 32 in the band from about 68 to 78 blocks, and 32 everywhere while the sun is under about 0.06 high (`look.wgsl:241-307`). Fog adds 12 more (one per fog step).
- **Measured:**
  - Turning shadows off saves 0.33–0.41 ms at 3456 on the PC. Only 0.04–0.08 of that is the drawing; the rest is sampling (in the world and in the volume effect).
  - The far map costs 0.02–0.10 standing (cached); while moving it is redrawn every frame (see "A moving camera").
  - On the Mac the shadow part measured 0.77 ms standing, 1.41 after a flight.
- **Not ranked high:** the sampling could use a comparison sampler and `textureGatherCompare` (4 depths per fetch), but the bindless table holds plain textures; on the PC the whole of the sampling is about 0.3 ms at 3456.

## Distant terrain: why far land shows as boxes

From the code (the subagent read was checked against `lod.rs`):
- **Tiles wanted.** At the default `core:lod_distance` 256 (4096 blocks) and view 12, `pick` wants **801 tiles**: L1 253, L2 192, L3 192, L4 164 (counted with the same rule in Python). L1 and L2 (64- and 128-block tiles, 2- and 4-block cells, out to about 1024 blocks) are box columns by design, with crowns as boxes. Their textures fade to a flat average colour from 220 to 520 blocks.
- **Pipeline.** A compute pass samples 34×34 columns per tile (up to 64 tiles a frame, read back 2 frames later). Then 3 low-priority threads mesh them; base's custom feature heights (1156 × features `height_at` calls per tile) run first on the shared `pool()` (`lod.rs:851-896`).
- **Uploads are not capped:** every finished mesh goes up the same frame (`lod.rs:637-644`).
- **Stand-ins.** A wanted tile that isn't made is covered by its nearest made ancestor, which hides every made tile under it until all are made (`lod.rs:658-695`). Parents are never asked for on purpose, so on a first load or a teleport an area is empty until the nearest-first queue gets there.

Measured (`load` log, PC):

| run | first seconds | then |
|---|---|---|
| 1280×720 (first load) | 223 tiles in second 2, 206 in second 4 | 15–31 a second for 20 s |
| 3456×2234 (first load) | 240, then 206 | 15–33 a second for 20 s |
| 3456×2234, each /tp of about 1000 blocks | 173–254 in the first second | 14–130, falling, for 3–5 s while the cubes load |

- The rate is the same at 720p and 3456, where the GPU frame is 4× longer, so **loading is limited by the CPU**: the 3 mesh threads, or the custom heights waiting on the shared pool while it finishes the cubes' terrain (`terrain per 1.0 s: … 3591 us each on a worker` in the same seconds). Which of the two needs M2.
- **Drawing is cheap:** about 0.1 ms on the PC (the far view with everything on is 1.81 ms; the LOD-off row was lost to the render-scale hang).
- So the boxes Ronald saw are coarse L2–L4 boxes standing in for L1 tiles still in the queue, then L1 and L2 box columns themselves at 64–1024 blocks. Fixes #4 and #7 address the first; the second is the design (DH also draws boxes there, with greedy merging and side faces cut against neighbours).

## Bad weather: mist is the most expensive thing in the game

`r15_weather_ab`, at 3456×2234 on the PC:
- **Canopy views:** Plants' rainforest interior (148.5 93 523.5), at 07:30 facing the sun, first clear, then with `/air 0.6 0` (morning mist).
- **Open land** at noon with the same mist.
- **Rain.**
- Each with the effects toggled as before. Shots: `r15_weather_3456.jpg`.

| part (ms) | clear ground (above) | open mist | canopy clear | canopy mist | rain |
|---|---|---|---|---|---|
| **whole frame** | 2.61 | **6.00** | 3.62 | **6.45** | 2.60 |
| opaque world | 1.00 | 3.05 | 0.79 | 3.26 | 1.05 |
| sky, clouds, water, particles (no-effects row) | 0.61 | 1.90 | 0.24 | 0.59 | 0.81 |
| contact | ≈0 | 0.03 | **0.99** | **1.04** | 0.00 |
| shafts | ≈0 | 0.02 | **1.24** | **1.44** | 0.00 |
| volume (Low; 16 steps under a canopy) | 0.26 | 0.30 | 0.23 | 0.29 | ≈0 |
| bloom | 0.18 | 0.22 | 0.27 | 0.24 | 0.17 |
| reflect | 0.23 | 0.26 | 0.06 | 0.08 | 0.25 |
| shadows off saves | 0.36 | **0.94** | 0.21 | 0.54 | 0.13 |
| Mac estimate (×2.25) | 5.9 | **13.5** | 8.1 | **14.5** | 5.9 |

What it shows:
- **Mist costs about +3.4 ms on the PC at 3456 (about +7.6 ms on the Mac)** over the same view in clear air.
  - About 2.05 ms of it is in the opaque world and 1.3 ms in the sky, clouds and water.
  - That is `fog_through` (`look.wgsl:614-654`): 12 steps per pixel, each with a `fog_density` noise and one shadow-map tap (`sun_reaches`). It runs for every shaded fragment through `haze_lit`, hidden ones too (there is no depth pre-pass), and for every sky and cloud pixel (`sky.wgsl:244, 326`).
  - Its shadow taps are most of why "shadows off" saves 0.94 ms in mist against 0.36 in clear air.
  - At the auto scale the Mac's misty mornings come to about 13–15 ms of GPU: under 60 fps with vsync at 120 Hz. That fits round 11's "morning fog would make lag come and go".
- **Under the canopy, contact shadows cost about 1 ms** (all pixels are within their 64-block reach, every one runs 16 depth reads plus 16 position reconstructions). The shafts through the crowns cost 1.2–1.4 ms. On the Mac that is about 2.2 ms and 3 ms.
- **Rain is cheap:** 2.60 ms, the same as clear. The rain's veil is a formula, not a march.
- At 1080p the same pattern holds: open mist 2.44 against clear 1.02, canopy contact 0.14, canopy shafts 0.14–0.17.

## A moving camera: the far shadow map is redrawn every frame

`r15_move_ab`: noon, open ground, spectator walking (`hold core:forward 5 nowait`), at 1920×1080 on the PC:

| bench | shadow part ms | draw records a frame | whole GPU ms |
|---|---|---|---|
| standing, near + far maps | 0.08 | 2,808 | 1.08 |
| **walking, near + far** | **0.22** | **7,380** | 1.23 |
| standing again after the walk (terrain still streaming in) | 0.22 | 8,437 | 1.27 |
| standing, near map only | 0.07 | 3,457 | 1.06 |
| walking, near map only | 0.07 | 3,401 | 1.05 |
| walking, near + far again | 0.22 | 7,778 | 1.25 |
| walking, no shadows | 0.00 | 1,418 | 0.87 |

- **While moving, the far map is redrawn every frame:** +0.14 ms of GPU on the PC and **+4,500 draw records a frame**. It stays redrawn after stopping while the terrain still streams in, because every mesh upload bumps `terrain_epoch`.
- The near map's cost doesn't change with movement.
- On the Mac the shadow part measured 0.77 ms standing and 1.41 after a flight (BIG_PERF), and MoltenVK turns each indirect draw record into its own Metal draw on the CPU. So this is about 0.6 ms of GPU plus the submit cost of 4,500 draws, whenever the player walks or chunks load. Fix #8 removes it.

## Ideas from Voxy, Distant Horizons and Horizon

Full comparison, licences and file paths: [research-16/sources.md](research-16/sources.md). Short version:
- **Licences.** Voxy's licence is "All rights reserved" (ideas only); Distant Horizons is LGPL v3 (ideas only); Horizon (github.com/horizon-rust/horizon, the Rust Minecraft rewrite Ronald meant) is MIT.
- **Taken into the ranking:** coarse-first loading, a level of tolerance, the old mesh kept until the new one is in, and many high-priority mesh threads (#4, #7); camera-facing face ranges per tile (#8); two-phase depth occlusion and cave culling (#12); a request budget that follows the mesher queue, and an upload time budget (both part of #4).
- **Not taken:**
  - Voxy's full GPU traversal (overkill at 801 tiles);
  - count-indirect draws (they split Metal passes under MoltenVK);
  - stored LOD data (ours is generated from the seed).
- **For the Mac's draw count:** Horizon found wgpu-on-Metal replays multi-draws one call at a time and "flattens" draws per page past 1,600 sections (0.5–1.3 ms of CPU saved on an M4 Pro). MoltenVK does the same with our indirect records (round 12), and the Mac's rows show 3,000–11,000 draws a frame. Worth a look after the sync.

## Not checked

- Nothing was run on the Mac (no game launches there). The Mac numbers are the PC's scaled by one calibration row, ±50%.
- The depth pre-pass was not tried: there is none (`app.rs:2914`), and the world's opaque shading (1.0 ms at 3456 on the PC) is the part a pre-pass would cut. How much depends on overdraw, which no counter shows. A test would need an engine branch.
- Leaves' overdraw: the forest view's opaque world is the largest (1.15 ms), but its cutout share was not separated.
- Frame pacing: covered by round 14.
