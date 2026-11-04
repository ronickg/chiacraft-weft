# Performance: why it's laggy now, and what fast voxel games do

Researcher, 2026-11-04, for Planner and Ronald ("it's super laggy now").

**What was read:**
- `dev` at `c76bbdab`: the engine's frame, the Vulkan backend, the pack's shaders and post effects, and the last day's ~45 merges;
- the way Sodium, Distant Horizons, Voxy, Vintage Story, Veloren, Bedrock and Photon keep frames fast (sources at the end).

**Not yet:** Distant terrain's measurements (`claude/bench-parts`, Tester's runs R0–R7 on the Mac, then the PC at 3200×1800). The ranking below comes from the code and the research. The table at the end says which run confirms or overturns each item.

## The short answer

1. **Ronald's Mac shades 6–8 times the pixels of every test so far.**
   - His Retina window is about 3200×1800 (5.8 Mpx), against the PC tests' 1280×720 (0.9 Mpx).
   - The engine has **no render scale**: every pixel of the world, the clouds and the three screen effects is shaded at full Retina resolution.
   - Each full-screen pass at that size moves about 92 MB of colour per round trip.
   - So whatever the PC tests showed, the Mac pays roughly 6× for every per-pixel cost.
2. **The last day added several per-pixel and per-frame costs at once:**
   - ray-marched clouds;
   - far water through screen reflections;
   - the shadow pass's unbounded volume;
   - the water's block-grid upload;
   - denser leaves (drawn twice, once for shadows).

   Each is modest on the PC. Together, at Retina size, on a tile-based GPU behind MoltenVK, they can push a frame past the 16.7 ms vsync step, and with vsync the frame rate then halves (60 → 30). That would feel "super laggy" rather than "a bit slower".
3. **The biggest gains are cheap.** A render scale for the 3D world with the UI at full resolution, as Veloren, Bedrock, Vintage Story and Photon do, cuts all per-pixel work by 2–4× in one change. Half-resolution screen effects, a cached shadow map and capped cloud steps follow.

## What changed in the last day, and what it costs

Ranked by how likely each is to matter on the Mac. All confirmed in the code.

1. **The sun's shadow pass draws an unbounded volume every frame.**
   - `frustum_planes` (`crates/engine/src/client/world_render.rs:1303-1307`) builds only the four side planes of the light's box. Along the light it never ends, so every solid and cutout mesh in a 160×160-block tube is drawn into the 2048² map each frame, underground caves included.
   - At golden hour the tube lies almost flat and runs across the loaded world.
   - Every fragment runs `fs_shadow`: a texture fetch, derivatives, and `discard`, which turns off Apple's hidden-surface removal for the whole pass.
   - On MoltenVK each indirect draw record becomes one Metal draw, so the extra draws also cost CPU at submit.
   - **Commits:** `5a096e0f`, `f97aa517` and the shadows before them.
2. **Clouds now march 24 samples per pixel, at full resolution, while writing depth.**
   - `CLOUD_SAMPLES = 24` (`mods/base/shaderpack/sky.wgsl:94`). Each sample is four texture fetches, a hash and lighting.
   - The shader writes `@builtin(frag_depth)` (`:108`), so early depth testing is off: pixels behind hills and walls march too.
   - The old clouds walked a grid and stopped at the first cell.
   - **Commits:** `e2477282`, `5a42062b`, `062fa28c`.
3. **Far water now goes through screen-space reflections.**
   - `fs_lod` returns far water with alpha ≥ 0.5 (`mods/base/shaderpack/world.wgsl:642-643`, `53c718f0`), which the reflection pass treats as water.
   - So the 40-step march plus refinement runs on every far-water pixel. Looking out to sea, that's much of the screen.
4. **Near water switched on the engine's block grid.**
   - The water shading calls `solid_at`. Naming it makes the engine upload the blocks around the camera every frame: `update_solid`, `world_render.rs:805`, from `client/mod.rs:1445`.
   - That is 3,375 hash lookups a frame, and on each cube boundary crossed, a 225-cube shell rebuilt: up to 192 × 4,096 block checks plus a 64 KB upload. Roughly 1–3 ms on the main thread, in bursts while walking or flying.
   - **Commit:** `55a6524e`.
5. **Three screen effects, each its own full-screen pass:**
   - **light shafts:** 40 colour and 40 depth reads per pixel while the sun is in view;
   - **contact shadows:** about 20–25 depth reads within 64 blocks;
   - **reflections:** up to 40 steps.

   On Apple's tile-based GPU each pass stores and reloads a full Retina colour image (`crates/engine/src/post.rs:139`).
6. **Per-pixel world shading.**
   - Sixteen shadow-map reads for every pixel within 80 blocks (`look.wgsl:150`).
   - On foggy days, a 12-step fog march of about 290 hashes per pixel, on terrain, water, sky and clouds (`look.wgsl:362`). Morning fog would make lag come and go.
7. **Denser, bigger trees.**
   - Grown from branches, wider crowns, 3×3 stands. Leaves don't cull faces between leaf blocks (`blocks.rs:168`), so more cutout faces with `discard`, drawn twice (main and shadow pass).
   - Each branchy tree is grown about 4 times across neighbouring columns at worldgen (`server/dim.rs:420`), which slows chunk arrival.
8. **No pipeline cache.**
   - Every pipeline is created with `vk::PipelineCache::null()` (`backends/vulkan/src/lib.rs:1447, 1475`). So each launch and each pack reload converts SPIR-V to Metal and compiles again.
   - MoltenVK's own guide says a serialized pipeline cache removes these "glitches and hiccups".
9. **Distant terrain's CPU work** per tile (base's rivers, mesas and highlands) competes with chunk finishing on the same pool. This slows loading, not steady frames (round 10's `lod.md`).

**Low impact, checked:**
- seasonal tints: a re-blend of column colours, capped per frame, 4 times a game day, no re-meshing;
- the colour grade: about 20 math operations a pixel;
- the night sky: 3 noise calls a sky pixel at night;
- rooms: a flood fill once a second, at most 400 blocks;
- the live portrait: only while its screen is open.

**Backend facts:**
- **Fine:** 3 swapchain images (the minimum + 1), no `vkQueueWaitIdle` in the frame, mapped ring buffers.
- **Questionable:** a full barrier at every pass start, and timestamp marks inside the world pass. Apple only samples timestamps between encoders, so MoltenVK may split the world pass there.

## What the fast ones do

- **Render the 3D world below native resolution, the UI at native.**
  - Veloren: a "non-UI graphics upscaling" factor of 0.25–2.0.
  - Bedrock: a render resolution with TAAU or bilinear.
  - Vintage Story: a Resolution slider rated "Medium FPS cost".
  - Photon: `TAAU_RENDER_SCALE 0.75` by default.
  - On Mac, Java Minecraft has no such option; the popular fix (retiNO, 56k downloads) just turns Retina off, UI included.
  - Apple recommends keeping the drawable at the screen's size, which lets Metal present directly to the display, and drawing the UI after any scaler. MetalFX's spatial and temporal upscalers exist, but reaching them through MoltenVK has no shipped example.
- **Screen effects at half resolution with a depth-aware upsample.**
  - Photon: ambient occlusion and fog at half resolution, volumetric light at half, clouds at a quarter with 16× checkerboard temporal upscaling.
  - Unity HDRP's SSAO runs at half resolution by default.
  - The cost of half resolution is halos at depth edges and some ghosting.
- **Cache the shadow map.**
  - Unity HDRP keeps static casters in a cached map and re-renders when the light's angle passes a threshold.
  - CryEngine's cached cascades cost no draw calls once built.
  - Cyberpunk 2077 refreshes distant shadow levels every few seconds.
  - Complementary's and Photon's low profiles shorten the shadow distance and turn entity shadows off.
- **Budget the work per frame.**
  - Sodium: meshing threads `clamp(max(cores/3, cores-6), 1, 10)`, job admission by estimated cost, uploads capped at `max(30% of the frame, 10 ms)`, culling on its own thread.
  - Sodium 0.7 made scheduling frame-rate independent.
  - Distant Horizons: thread presets down to 10% of cores, and pauses generation when the server lags.
  - Voxy: at most 50 node loads requested per frame, through a 64 MB fenced ring.
- **Draw less.**
  - Sodium: cave-culling by section connectivity (10–15% fewer sections), 20-byte vertices, shared index buffers (up to 25% less bandwidth), and skipping back-facing face groups.
  - Weft already has the cave-culling fill and big face pages.
- **On Apple GPUs:**
  - draw opaque, then alpha-tested, then translucent;
  - load and store only what's needed, with memoryless transient attachments;
  - merge passes so data stays in tile memory;
  - avoid `discard` and depth writes where possible, because both disable hidden-surface removal (WWDC20 10602).

## Changes, ranked by likely gain for us

| # | change | gain | effort | where | owner |
|---|---|---|---|---|---|
| 1 | **A render scale for the 3D world.** Render the world, sky, clouds and screen effects into an offscreen target at a scale, default about 0.6–0.7 on Retina, then upscale (bilinear, or FXAA-aware like Veloren) into the full-resolution swapchain, and draw the UI on top at native. A setting `core:render_scale`, 0.25–1.0. | All per-pixel work ×0.36–0.5 at 0.6–0.7. The single biggest change for the Mac. | M | `crates/engine` (frame, `post.rs`), `crates/render`, `backends/vulkan` (offscreen targets, blit) | a new "Performance" engine session, or Distant terrain |
| 2 | **Bound the shadow pass, and stop redrawing it every frame.** Add the light's near and far planes (`frustum_planes`), so the box ends. Give opaque shadow casters a pipeline with no fragment shader. Skip meshes with no sky light. Re-render the terrain shadow map only when the sun has moved about a texel or shadowed chunks changed. | The shadow pass drops from every frame to rarely; golden hour stops being the worst time. | S (planes, depth-only), M (caching) | `world_render.rs:594, 1303`, `client/mod.rs:2287-2367`, the pack's `fs_shadow` | Lighting, with an engine session for caching |
| 3 | **Clouds cheaper.** Fewer samples (8–12), and no `frag_depth` write: test scene depth first, so hidden pixels don't march. Later, clouds at a quarter resolution with checkerboard reprojection (Photon). | The cloud pass ÷2–3 now, ÷10+ with reprojection. | S, then M | `mods/base/shaderpack/sky.wgsl` | Lighting |
| 4 | **Screen effects at half resolution with a bilateral upsample**, and fewer passes: contact shadows folded into the reflection pass. Light shafts at half resolution with 20 steps (Photon uses 20). | Those passes ÷3–4. | M | `post.rs`, `mods/base/assets/shaders/post/*` | Lighting (shaders), engine (half-resolution targets) |
| 5 | **Far water out of the reflection march** (keep its Fresnel and sky colour, but return `SOLID_ALPHA` so `reflect.wgsl` skips it), or reflections only within 128 blocks. | Large looking at sea. | S | `world.wgsl:642` | Lighting |
| 6 | **The water's block grid at a budget.** Rebuild the `solid_at` shell incrementally over several frames, or only where water is in view. | Removes 1–3 ms bursts while moving. | S–M | `world_render.rs:805`, `client/mod.rs:1445` | engine |
| 7 | **A pipeline cache saved to disk** (`VkPipelineCache` loaded at start, saved at exit), plus prewarming the pack's pipelines at load. | Removes compile hitches on first sight of each shader and on reload. | S | `backends/vulkan/src/lib.rs:1447, 1475` | engine |
| 8 | **Quality profiles and a settings screen for them:** Low, Medium, High for render scale, shadow reach, cloud steps, effects on or off, and fog steps. Medium is the Mac default. | Lets Ronald and players choose; a measuring tool too. | S–M | pack config, engine settings | Lighting + UI redo |
| 9 | **Leaves cull faces between leaf blocks**, at least in the shadow pass, and grow each tree once per worldgen, not 4 times. | Fewer cutout faces; faster chunk arrival. | S | `mods/base/src/blocks.rs:168`, `server/dim.rs:420` | Voxel dog, engine |
| 10 | **Measure on the Mac with the right tools:** the Metal HUD frame-interval histogram (`MTL_HUD_ENABLED=1`), an Xcode Metal capture per pass, Tracy for CPU, and p99 rather than the average. | Tells which of 1–9 matters most, before more guessing. | S | — | Tester |

**Do first:** 1, 2 (planes and depth-only), 3 (samples and depth) and 5. Together they're about a day's work in three places and should bring the Mac back under the vsync step. Then 4, 6 and 7. The rest as the measurements say.

## What Distant terrain's runs will tell us

| run | changes | confirms, if it moves a lot |
|---|---|---|
| R0 | as Ronald plays (pack, vsync, view 12, LOD 256) | the baseline |
| vsync off | — | how close frames are to the 16.7 ms step (item 7 of the causes) |
| no pack | the chiacraft pack off | the pack's per-pixel cost overall: changes 1, 3, 4, 5 |
| no screen effects | `--no-reflect --no-contact --no-shafts` | change 4 (and 5 for reflections) |
| no shadows | `--no-shadows` | change 2 |
| view 8 | a smaller view | geometry and draw count: changes 2 and 9 |
| LOD 0 | distant terrain off | round 10's LOD costs, and far water (change 5) |
| R7 | a quarter of the pixels, vsync off | **fill rate: if frame time falls about 4×, change 1 is the answer** |

When the numbers land, this table gets a column of results and the ranking above gets re-ordered by them.

## Sources

- **Resolution:**
  - [Veloren's renderer](https://raw.githubusercontent.com/veloren/veloren/master/voxygen/src/render/mod.rs) (upscale factor);
  - [Bedrock's Vibrant Visuals settings](https://www.sportskeeda.com/minecraft/every-minecraft-vibrant-visual-setting-explained), [MCPE-229723](https://mojira.dev/MCPE-229723);
  - [Vintage Story settings](https://wiki.vintagestory.at/Settings);
  - [retiNO](https://github.com/juliand665/retiNO), [MC-156077](https://bugs-legacy.mojang.com/browse/MC-156077);
  - [Photon settings](https://github.com/sixthsurge/photon/blob/main/shaders/settings.glsl);
  - Apple: [managing your game window](https://developer.apple.com/documentation/metal/managing-your-game-window-for-metal-in-macos.md), [MetalFX (WWDC22)](https://developer.apple.com/videos/play/wwdc2022/10103/), [WWDC25 211](https://developer.apple.com/videos/play/wwdc2025/211/).
- **Sodium:**
  - [ChunkBuilder](https://raw.githubusercontent.com/CaffeineMC/sodium/dev/common/src/main/java/net/caffeinemc/mods/sodium/client/render/chunk/compile/executor/ChunkBuilder.java), [RenderSectionManager](https://raw.githubusercontent.com/CaffeineMC/sodium/dev/common/src/main/java/net/caffeinemc/mods/sodium/client/render/chunk/RenderSectionManager.java);
  - [0.5.0](https://github.com/CaffeineMC/sodium/releases/tag/mc1.20.1-0.5.0), [0.7.0](https://modrinth.com/mod/sodium/version/mc1.21.8-0.7.0-fabric);
  - [CompactChunkVertex](https://raw.githubusercontent.com/CaffeineMC/sodium/dev/common/src/main/java/net/caffeinemc/mods/sodium/client/render/chunk/vertex/format/impl/CompactChunkVertex.java);
  - [visibility (tomcc)](https://tomcc.github.io/2014/08/31/visibility-2.html).
- **Distant Horizons and Voxy:**
  - [DH thread presets](https://gitlab.com/distant-horizons-team/distant-horizons-core/-/raw/main/core/src/main/java/com/seibel/distanthorizons/core/config/eventHandlers/presets/ThreadPresetConfigEventHandler.java);
  - [Voxy traversal](https://github.com/MCRcortex/voxy/blob/dev/src/main/java/me/cortex/voxy/client/core/rendering/hierachical/HierarchicalOcclusionTraverser.java), [Voxy upload stream](https://github.com/MCRcortex/voxy/blob/dev/src/main/java/me/cortex/voxy/client/core/rendering/util/UploadStream.java).
- **Shadows:**
  - [HDRP shadows](https://docs.unity3d.com/Packages/com.unity.render-pipelines.high-definition@13.1/manual/Shadows-in-HDRP.html);
  - [CryEngine cached shadows](https://www.cryengine.com/docs/static/engines/cryengine-3/categories/1114113/pages/21267738);
  - [Cyberpunk 2077 shadows (SIGGRAPH '21)](https://history.siggraph.org/wp-content/uploads/2022/06/2021-Talks-Dybisz_Shadows-Optimizations-in-Cyberpunk-2077.pdf);
  - [Complementary's profiles](https://github.com/ComplementaryDevelopment/ComplementaryReimagined/blob/main/shaders/shaders.properties).
- **Half resolution:**
  - [HDRP ambient occlusion](https://docs.unity3d.com/Packages/hdrp/manual/Override-Ambient-Occlusion.html);
  - [GTAO](https://research.activision.com/publications/archives/practical-real-time-strategies-for-accurate-indirect-occlusion), [XeGTAO](https://github.com/GameTechDev/XeGTAO);
  - [joint bilateral upsampling](https://johanneskopf.de/publications/jbu/).
- **Apple and MoltenVK:**
  - [WWDC20 10602](https://developer.apple.com/videos/play/wwdc2020/10602/), [tile-based deferred rendering](https://developer.apple.com/documentation/metal/tailor-your-apps-for-apple-gpus-and-tile-based-deferred-rendering);
  - [MoltenVK runtime guide](https://github.com/KhronosGroup/MoltenVK/blob/main/Docs/MoltenVK_Runtime_UserGuide.md), [MoltenVK config](https://github.com/KhronosGroup/MoltenVK/blob/main/Docs/MoltenVK_Configuration_Parameters.md);
  - [Metal HUD](https://developer.apple.com/documentation/xcode/understanding-metal-performance-hud-metrics.md).
- **Frame pacing:** [Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/), [frame timing (Ladavac)](https://medium.com/@alen.ladavac/the-elusive-frame-timing-168f899aec92), [Tracy](https://github.com/wolfpld/tracy).
