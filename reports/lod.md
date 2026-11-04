# Distant terrain (LOD): why it looks off, and what to change

Researcher, 2026-11-04, round 10, for Planner and Ronald ("LODs etc seem broken... It would always seem off, slow").

**What was read:**
- Weft's LOD code in `dev` at `82cf20a3`: `crates/engine/src/client/lod.rs`, `terrain_gpu.rs`, the `lod` entry in `terrain.wgsl`, base's `fs_lod`;
- Distant Horizons' and Voxy's source, and Veloren, CDLOD and clipmaps (sources at the end).

**Tested** by Tester 2 on the PC (RTX 5090, `dev` `6955aacb`, seed 1, 1280×720), with LOD at 256 (A) and off (B). The shots and bench lines are in `lod/ours/`, the originals and the script in `~/Coding/chiacraft-archive/researcher/lod/`.

**References** (downloaded with Ronald's yes, internal only): `lod/refs/` holds 5 Distant Horizons and Voxy pictures, 12.8 MB of originals in the same archive folder.

**Who wrote it:** it is upstream Weft, not ours: VortexSo4, `f0efa0ac` "distant terrain on gpu" and `c9416f41` "distant terrain fills gaps", 2026-10-29. So any fix in the engine is also a candidate to offer upstream.

## The short answer

**Nothing is outright broken, and the GPU cost is small.**
- **Cost:** on the 5090, LOD costs 0.15–0.35 ms of GPU a frame (0.49 against 0.33 ms high, 0.78 against 0.43 after a flight) and about 270 extra draws.
- **Spikes:** the 80–87 ms spikes after each teleport are the same with LOD on or off, so those aren't LOD.

**It looks off** because, at about 192 blocks, everything changes at once and nothing hides the change:
- textures become one flat colour;
- blended biome tints become hard ones;
- trees become leaf pillars;
- water becomes an opaque pale sheet;
- rock faces become dirt;
- hills become stairs of 2-, 4-, 8- and 16-block steps.

**It feels slow** for two likely reasons:
1. **Tile work at join and while flying:** several milliseconds of CPU per tile for base's rivers, mesas and highlands, on the same thread pool that finishes the near cubes, so each slows the other. Plus GPU generation spikes, and tiles regenerated when you move back and forth.
2. **The Mac:** MoltenVK, and a shader that discards pixels while writing depth, which costs more on Apple GPUs. That hasn't been measured; the PC is far from the Mac.

**Fix it in place, don't rebuild it.** Weft builds far terrain on the GPU from the *same* height function as the near world. That is Veloren's and CDLOD's approach, and it suits a world that is mostly generated and visited only where people go. What it lacks is what Distant Horizons and Voxy add on top: real colour, light, water, a soft border, stable detail levels. Each of those is a change to Weft's LOD, not a new system. The Voxy-style layer, built from real chunks for players' builds, is a later addition, not a replacement.

## What we see (Tester 2's shots)

- **After a flight, LOD on** (`lod/ours/2026-11-04_a_lod_after_flight.jpg`):
  - the land runs to the horizon, with no crack at the cube edge;
  - but the far land is a flat-coloured carpet of boxes;
  - lakes are uniform pale-blue sheets with hard edges, lit like land, nothing like the near water.
- **Same view, LOD off** (`b_...`): nothing beyond the cubes but a pale haze. LOD is clearly worth having.
- **On the ground:**
  - with LOD on, far hills read a little lighter and flatter green than near ones;
  - with LOD off, the haze closes in much nearer, because the fog end follows the LOD distance (`client/mod.rs:2222`).
- **Seen in passing, cause not known:** trees with **magenta leaves** on the snowy slope near the camera, in real cubes, not LOD. They look like a missing or broken tint. Lighting or Voxel dog should look.

## Why, from the code

1. **One hard ring, nothing hiding it.**
   - The fog end becomes 4096 blocks whenever LOD is on (`client/mod.rs:2222`), so haze at the cube edge (about 192 blocks) is only 1–3% (`look.wgsl:401-418`).
   - The edge is a stair-stepped circle of 16-block columns (the mask, `lod.rs:353`), and nothing fades or dithers.
   - Distant Horizons hides it with a full-screen fade of near pixels into far ones over a distance band ([vanilla_fade.frag](https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/resources/assets/distanthorizons/shaders/fade/gl/vanilla_fade.frag)). Voxy masks the far terrain exactly behind the loaded chunks with a depth bound ([BoundRenderer](https://github.com/MCRcortex/voxy/blob/dev/src/main/java/me/cortex/voxy/client/core/rendering/bounding/BoundRenderer.java)).
2. **Colour.**
   - Each face is the texture's 1×1 average with a hard per-biome tint (`fs_lod`, base `world.wgsl:622-640`), while near cubes blend tints across columns.
   - Walls are the top block over the "under" block all the way down (`lod.rs:154-158`). Base's rock faces are a decorate pass, which never reaches LOD (`mods/base/src/land.rs:76, 151`). So far cliffs show green tops over brown walls, while the same faces near are grey rock.
   - Distant Horizons since 3.2 and Voxy draw real textures on far faces.
3. **Trees placed by a test at the world origin.**
   - `describe_lod` runs each scatter shape 8 times at `pos [0, 64, 0]` (`terrain_gpu.rs:147-160`, checked). Base's tree shapes depend on where they stand (stands, clearings, `site_on_rock`, `worldgen.rs:234-252, 317`).
   - So far forests are full density everywhere or nowhere, depending on the seed, with no clearings.
   - Canopies become solid leaf columns, and trees float or sink on slopes.
   - This is a real bug.
4. **Shape.**
   - Each cell is a box with vertical walls at the height sampled at its middle: far hills are stairs.
   - Rivers 10–30 blocks wide break into dotted pools at levels 3–4.
   - Detail changes at fixed rings (about 512, 1024, 2048 blocks), and a tile's outline jumps when it swaps level: popping.
   - CDLOD and clipmaps morph between levels instead ([CDLOD](https://aggrobird.com/files/cdlod_latest.pdf), [GPU Gems 2](https://developer.nvidia.com/gpugems/gpugems2/part-i-geometric-complexity/chapter-2-terrain-rendering-using-gpu-based-geometry)).
5. **Water** is the opaque average colour in the opaque pass: no reflection, fresnel or depth colour. Lighting already has a fix on test (far water, `53c718f0`).
6. **Churn.**
   - Tiles that are no longer wanted are freed at once (`lod.rs:319-328`), with no hysteresis and no cache.
   - Work already in flight is never cancelled.
   - A child tile shows over its parent before its three siblings are ready (`lod.rs:292-311`).
   - Simulated: about 11 new tiles per 16 blocks walked, and 6–10 regenerated pacing over a threshold.
7. **Light, and the brightness step at the edge.**
   - `vs_lod` gives every far face full sky light and no AO: `out.light = vec4(1.0, 0.0, 0.0, 0.0)` and `out.ao = 1.0` (`mods/base/shaderpack/world.wgsl:604, 608`).
   - Near cubes at the same distance keep their real sky light and corner AO. Shadows end at 80 blocks for both, so that part isn't the difference.
   - So the far land reads lighter and flatter than the cubes beside it, as Tester 2 saw: a brightness step at the edge.
   - Valleys and forest floors never darken.
   - Distant Horizons and Voxy keep sky and block light per voxel and add SSAO. The quick fix is in our pack: give far faces the average AO and side-face darkening near terrain has at that distance.
8. **Cost at default settings** (`lod_distance` 256 cubes, view 12):
   - **Tiles:** 801, about 830k cells, about 1.2–4 M quads before culling.
   - **The level-4 ring** is mostly drawn and then nearly covered by haze (full haze from about 2,950 blocks).
   - **CPU per tile:** 1,156 samples × 3 base features, several ms, on the cube finishers' thread pool (`terrain_gpu.rs:427`).
   - **GPU per frame:** up to 64 tiles, with level-4 tiles scanning about 47k emulated 64-bit hashes for trees and boulders.

## Changes, ranked by how much better for the work

| # | change | effort | owner | why |
|---|---|---|---|---|
| 1 | **Hide the ring.** Haze that starts near the cube edge, not tied to the LOD distance. A fade band over the last 16–32 blocks of cubes, dithered against LOD, like Distant Horizons' fade. Stop drawing the level-4 ring where haze is already full. | S–M | Lighting (haze, fade in the pack), engine (the fog end in `client/mod.rs`) | The single most visible thing: the place where everything changes at once. |
| 2 | **Far water through the water shader:** Lighting's `53c718f0` (water-tinted faces through `water()` in `fs_lod`). Then far lakes keep their depth colour and reflect the sky. | S | Lighting | The worst-looking part of the shots, and the fix exists. |
| 3 | **Trees placed per site.** Evaluate the scatter shape's stand, clearing and rock tests at each real site, or scale chance by many sampled positions, instead of 8 runs at the origin. Canopies as a crown over a trunk, not pillars to the ground; trees on the sampled ground under them. | S–M | engine (`terrain_gpu.rs:147`), with Voxel dog for base's shapes | A real bug: forest density depends on the seed. |
| 4 | **Colour that matches near.** Biome tint blended per corner from neighbour samples. Walls of the filler block for its real depth, then stone, and rock where base's `rock_faces` would put it: an output word or a rule the GPU can apply. Average colours in linear space, not sRGB. Later, textured faces for levels 1–2 (Distant Horizons 3.2's method: the texture as a ratio of its average colour, so it fades into the flat colour at distance). | S (tint, linear), M (walls, rock), M (textures) | engine, Lighting, Worldgen for the rock rule | Far cliffs and mesas today are green over brown, near ones grey. |
| 5 | **Stable detail levels.** Hysteresis (split at 4× tile width, merge at 4.5×). Keep the parent until all four children are ready. Cancel stale jobs. A small cache of recently dropped tiles. LOD's CPU work on its own thread pool, so it doesn't slow the near cubes. | S | engine (`lod.rs`) | Less popping, less regeneration when moving, faster joins. |
| 6 | **Shape: a smooth heightmap mesh with morphing for the coarse levels.** Levels 3–4 as a grid from the same height function, morphing to the next level over a band (CDLOD), or Veloren's single stretched grid with bicubic heights. Keep boxes for levels 1–2, where blocky detail still reads. | M–L | **a new engine session** | Ends the stairs and the level rings at a distance where boxes add nothing. |
| 7 | **Light on far terrain.** Darken by slope and by a cheap horizon or occlusion term from the heightmap (valleys, forest floors). Far shadows from a horizon map later (Veloren does this). | M | Lighting | Far land stops looking uniformly lit and flat. |
| 8 | **Measure on the Mac, then trim.** The PC numbers say the GPU cost is small. Ronald's "slow" needs a Mac bench of LOD on and off: Tester, once the Mac's IOSurface count is clear after a reboot. If Apple GPUs suffer from `fs_lod`'s discard with depth writes, mask by depth bound (Voxy) instead. `SPLIT` 3 instead of 4 halves the finest ring's geometry. | S | Tester, then engine | Find out what "slow" is before optimising the wrong thing. |

**Later, not now: a layer built from real chunks** for what players build. Voxy's 32³ mip sections fit our cubic chunks, built from visited chunks and edits, and sent with the world like Distant Horizons' server streaming. That's how far-away bases, roads and structures would show. It is a new engine system and a session of its own, after the eight above.

**Where it goes:**
- **Lighting:** 1 (pack side), 2, 4 (tint, colour space, textures), 7.
- **A new engine session** ("Distant terrain"): 3, 4 (walls), 5, 6, then the real-chunk layer. It works in Weft's own files, so it should offer each fix upstream.
- **Voxel dog and Worldgen:** the tree shapes and the rock rule that LOD must read.
- **Tester:** 8, plus the fixes for the script below.

**For the next LOD run** (Tester 2's notes):
- creative players fall after `/tp`, so the high views need `/gamemode spectator` first;
- testkit pins the view distance at 12, so a view-24 comparison needs that setting changed;
- the virtual compositor caps at 60 fps even with `--no-vsync`, so compare GPU ms, not FPS.

## Sources

- **Distant Horizons:**
  - stored data [FullDataSourceV2](https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/dataObjects/fullData/sources/FullDataSourceV2.java) and [FullDataPointUtil](https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/util/FullDataPointUtil.java);
  - [ColumnBox](https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/dataObjects/render/bufferBuilding/ColumnBox.java) (border faces kept against holes);
  - textures [BlockTextureRegistry](https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/dataObjects/render/textures/BlockTextureRegistry.java);
  - [problems and solutions](https://gitlab.com/distant-horizons-team/distant-horizons/-/wikis/1-user-guide/1-frequently-asked-questions/2-problems-and-solutions/Problems-and-Solutions), [server owners](https://gitlab.com/distant-horizons-team/distant-horizons/-/wikis/1-user-guide/1-frequently-asked-questions/5-server-owners/Server-Owners);
  - [3.3.0](https://modrinth.com/mod/distanthorizons/version/3.3.0-26.3) (reverse-Z, TAA);
  - Iris's DH contract: [DHCompatInternal](https://github.com/IrisShaders/Iris/blob/HEAD/common/src/main/java/net/irisshaders/iris/compat/dh/DHCompatInternal.java).
- **Voxy:**
  - [WorldSection](https://github.com/MCRcortex/voxy/blob/dev/src/main/java/me/cortex/voxy/common/world/WorldSection.java), [Mipper](https://github.com/MCRcortex/voxy/blob/dev/src/main/java/me/cortex/voxy/common/world/other/Mipper.java);
  - GPU traversal [traversal_dev.comp](https://github.com/MCRcortex/voxy/blob/dev/src/main/resources/assets/voxy/shaders/lod/hierarchical/traversal_dev.comp);
  - [ModelFactory](https://github.com/MCRcortex/voxy/blob/dev/src/main/java/me/cortex/voxy/client/core/model/ModelFactory.java) (textured LOD);
  - no Mac ([#208](https://github.com/MCRcortex/voxy/issues/208)).
- **Veloren:** [lod.glsl](https://gitlab.com/veloren/veloren/-/blob/master/assets/voxygen/shaders/include/lod.glsl).
- **Terrain LOD in general:** [CDLOD](https://aggrobird.com/files/cdlod_latest.pdf), [geometry clipmaps](https://hhoppe.com/proj/geomclipmap/), [0fps on blocky LOD](https://0fps.net/2018/03/03/a-level-of-detail-method-for-blocky-voxels/).
