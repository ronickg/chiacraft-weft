# Research round 12: voxel game performance

Researcher, 2026-11-04, for Planner, "Lag fixes" and "Lighting". Ronald asked: "i remember watching a lot of cosmic reach that did lots of great perf improvements maybe can find some good ideas from that and also ofc online researching voxel base games perf".

**Read:** `reports/performance.md` (round 11), `docs/architecture.html` (about the network and accounts; the render design is in the code, so I read `crates/engine/src/{lib,post,app}.rs`, `client/{world_render,mesher,terrain,terrain_gpu,lod}.rs`, `backends/vulkan/src/lib.rs`), Tester's runs `lag-*.sum` and `LAG_SUMMARY.md` (13 runs' BENCH lines, 14,195 translucent-sort lines), the Lag fixes and Lighting hand-over notes, and the branches `claude/render-scale`, `claude/mac-qos`, `claude/pipeline-cache`, `claude/frame-thread`, `perf`, `claude/entity-cull`.

**Web research** was done by three sub-agents with source links; their full notes (more detail than here) are in the shared notes folder: `cosmic-reach-perf.md`, `voxel-engines-perf.md`, `meshing-gpu-apple-perf.md`. I spot-checked: the MoltenVK deprecation text, the Vintage Story API licence and one Cosmic Reach video title. Everything marked *unconfirmed* is what the sources could not settle. Nothing was downloaded except text pages; videos were read as auto-caption transcripts only.

## The short answer

1. **Weft already does most of what the fast voxel engines do.** One draw record per cube-layer-run in big paged storage buffers with vertex pulling, 16-byte faces, per-direction face groups skipped by camera side (Sodium's back-face trick), region-then-cube frustum culling on every core, frame-budgeted mesh uploads, a GPU-made terrain, GPU-made far LOD tiles, hidden-face culling in the mesher, opaque → cutout → translucent order (Apple's rule). Greedy meshing, palettes, mesh shaders and GPU Hi-Z are not where our gain is.
2. **Our measured numbers cannot yet say which cost makes the Mac laggy.** Every Mac run is locked at 8.33 ms (120 fps) and the GPU column swings 2–5 ms between identical passes. Details in the next section. The first job is a fixed-workload measurement at Ronald's real size, before any more fixes are guessed.
3. **The new ideas that are cheap and plausible** (ranked below): skip the full-screen water copy when no water is drawn; try KosmicKrisp instead of MoltenVK; pace presents evenly and check window vs fullscreen; cap the worker threads in total; draw-side cave culling from the connectivity bits we already compute.
4. **Cosmic Reach is a small-dev project whose public perf notes are ideas we mostly have.** Its best one (batch 4×4×4 chunk meshes, keep 16³ logic) is a draw-call fix; our draws are already indirect, and our runs are not draw-bound (below).

## What our numbers say, and what they don't

Facts from `runs/*.sum` (Mac, M3 Max, Tester):

- **Window size.** Runs used the default `--size 1600x900` in *physical* pixels (1.44 Mpx), so they never saw Ronald's 3456×2234 (7.7 Mpx, 5.4× more). Lag fixes found this already (their note `lag-fixes-runs.md`) and queued rows at `--size 3456x2234`.
- **120 fps even with `--no-vsync`.** R0 (vsync) and R1 (`--no-vsync`) are both exactly 8.33 ms. So in a window the compositor still paces us to 120 Hz; `submit 7.5 ms` in the CPU column is time blocked in present/acquire, not CPU work. **`--no-vsync` cannot be used to find the headroom.** Use the GPU column or a fixed load.
- **GPU ms does not move with the things we expected.**
  - R7 (800×450, a quarter of the pixels): 1.94 / 2.07 / 4.35 / 4.06 / 3.91 ms against R0 2.05 / 2.11 / 4.09 / 3.40 / 5.05 ms. No drop.
  - R5 (view 8 against 12): same. R3 (no screen effects) and R4 (no shadows): same, and the `shadows` and `effects` parts of the BENCH line read 0.00–0.05 ms in every row.
  - R6 (LOD off) is the only row that moves (stand 1.00, settled 1.65 against 2.05 and 5.05), but `lag-lod` R6 vs R0 shows the same sort of swings in the other direction, so it may be noise.
- **Reading:** at a 25–60% duty cycle a GPU running far below its limit is clocked down and the timestamps include that ramp, so these GPU times are not a clean cost per pass. *Hypothesis, not proven.* It matches why a 4× pixel cut shows nothing, and it means these runs cannot rank R1–R7 either way.
- **Spikes that do show.** Frames of 30–70 ms appear in `devnow-R1/R4` and `dev-R6` while the CPU parts add up to about 9 ms: the thread waited for a core. That is the 70-worker problem (mac-qos branch). Also 85 of 14,195 translucent re-sorts took over 8 ms (max 23 ms) on the main thread for at most 16k quads, work that should take about 1 ms: again waiting for the global pool (frame-thread branch moves it).
- **Machine load** during runs was 6–20 on 14+ cores (other sessions building), which adds noise.

What to do about it is item 1 below.

## What is already in flight (not repeated as findings)

| piece | branch / commit | covers |
|---|---|---|
| render scale (auto 2/3 on Retina, UI whole) | `claude/render-scale` `73b0bf70` | round 11 #1 |
| QoS classes, bulk threads utility, frame user-interactive | `claude/mac-qos` `b3dfe105` | the "waited for a core" spikes |
| pipeline cache on disk | `claude/pipeline-cache` `b1867503` | round 11 #7 |
| frame-thread work (translucent sort etc.) | `claude/frame-thread` `ac4995f3` | the 8–23 ms sorts |
| bounded depth-only shadows, kept far map; clouds 10 samples and none behind terrain; far water out of reflections; effects in one pass; Low effects; the water's block grid only on change | `perf` `bdb745ef` (six commits) | round 11 #2, #3, #4, #5, #6 |
| entities culled to the view | `claude/entity-cull` `08b5610c` | |

None of these is on `dev` yet.

## Ranked findings (new ideas; gain, effort, risk against our measured costs)

Gain is judged against what we can measure (above), so it is stated as a range with the reason. "Owner" is the session whose area it is.

| # | idea | Weft today | expected gain | effort | risk | owner |
|---|---|---|---|---|---|---|
| 1 | **Measure with a fixed workload at the true size, with Apple's tools.** Run at `--size 3456x2234`; add a debug bench option that repeats the world pass N times (or `render_scale` above 1) so the GPU stays at full clock and per-pass cost scales cleanly; read the Metal HUD (`MTL_HUD_ENABLED=1`, `MTL_HUD_ENCODER_TIMING_ENABLED=1`: shows direct vs composited present and per-encoder GPU time) and MoltenVK's own timings (`MVK_CONFIG_PERFORMANCE_TRACKING=1`). | Not done; bench GPU parts exist (`bench.rs`) but at low duty cycle. | Decides every other item. Without it the Retina fix may be right or may not matter. | S | none | Lag fixes + Tester |
| 2 | **Skip `copy_behind` when no water is drawn.** Every frame with the post chain and a `water_behind` pack ends the world pass (stores colour and depth), runs a full-screen copy pass into a second colour and depth image (the copy writes depth from the shader), then starts the world pass again loading both back (`app.rs:2722-2750`, `post.rs:160`). It does not check that any refractive water is on screen. On a tile GPU that is a pass split plus about 4 full-size image moves per frame; at 7.7 Mpx colour + depth is about 62 MB stored, copied, and reloaded. Source: Apple WWDC20 10602 ("only store what you need", "avoid ... discard or update the depth"); MoltenVK also restarts passes for several reasons (D below). | Always on with that pack. | Removes a pass split and ~0.25 GB of traffic per frame in dry views (most frames); about 0.5–1 ms at 300–400 GB/s, more if the split costs more. | S: count the translucent water runs (`runs` after `self.translucent`) and set `behind` only when nonzero; an earlier test that decides it before `BeginPass` | low (water seen at the edge: use the draw list's own count, same frame) | Lag fixes (engine); Lighting reviews the pack side |
| 3 | **A/B KosmicKrisp against MoltenVK.** The MoltenVK README now says "Khronos has deprecated MoltenVK ... maintained by the community on a best-effort basis" and "Khronos now recommends KosmicKrisp on Apple Silicon Macs" (I read it). KosmicKrisp is Mesa's Vulkan-on-Metal driver, MIT, Vulkan 1.3 conformant; needs macOS 15+. No public FPS comparison exists (one emulator report: lower peak memory, more shader-compile stalls: *unconfirmed*). It is a driver swap through the Vulkan loader, no engine code. | `backends/vulkan` loads `libMoltenVK.dylib` beside the exe or the loader (`lib.rs:185-213`). | Unknown, from nothing to large; the worst case is a driver that fails. | S to try (loader + ICD file, env var) | low; do not ship until it matches on the PC-parity checks | Lag fixes + Tester |
| 4 | **Even presenting; check window vs fullscreen.** Apple (WWDC21 10147): on the adaptive-sync (40–120 Hz) display a frame may stay up 8–25 ms, "present frames at the highest rate your app can do so evenly"; on a fixed 120 Hz one a frame that misses a tick holds the old frame 16.7 ms. The Zed editor team saw a Metal 60 fps cap with 8/16 ms oscillation because they skipped presents, and fixed it by presenting every tick (zed.dev/blog/120fps). MoltenVK has no `presentAfterMinimumDuration` and ignores `desiredPresentTime`, so pacing must be done on the CPU. An Apple forum thread (733033) found direct (fullscreen) mode worse than windowed on some macOS 13–14 releases (anecdotal). | FIFO, 3 images, 2 frames in flight; no CPU pacing. | Perceived smoothness when frames run 8–16 ms: the 60↔120 flip-flop. | S: a "frame cap" setting (30/40/60/120) that sleeps to the target; test the window against fullscreen with the HUD's "direct/composited" flag | low | Lag fixes |
| 5 | **Cap the worker threads in total, not per pool.** Pools: global, background, meshing (each `cores − 2`), urgent 4, terrain pool, LOD pool: about 70 on a 14-core Mac (`lib.rs:50-110`). Veloren sizes meshing at cores − 3/4 total; Sodium at `clamp(max(cores/3, cores−6), 1, 10)` and runs workers at "normal − 2" priority. The Apple QoS facts: background QoS runs only on E cores (never promoted); 17+ prefers P cores; **a thread that ever called `pthread_setschedparam` is permanently opted out of QoS** (check no code path does it; `set_priority` is a no-op on macOS today). | `mac-qos` sets classes but keeps ~70 threads. | Fewer context switches while loading; steadier frame thread. Not measurable until item 1. | S | low | Lag fixes |
| 6 | **Cave culling for drawing, from the 15-bit face connectivity we already compute.** tomcc's visibility graph (MC PE, 2014): per 16³ chunk store which face pairs connect through air (15 bits, ~0.1–0.2 ms to build), BFS from the camera, skip chunks whose entry/exit faces don't connect: "culling ratios going from 50% to 99%" in caves; Sodium measured up to 10–15% fewer sections at high distance (0.6.0) and a refinement ("frustum clamping") culls 5–25% more at ~0.5 ms per 8,000 sections. Weft's `server::visibility` already makes a `u16` connectivity per cube (`client/terrain.rs:180`), but only to choose *what to generate*; drawing culls only by frustum and face direction. | generation only | Fewer cube draws and less vertex/binning work in caves and under mountains; little over open ground. Not measured because no run goes underground. | M | low: if wrong, ravines under-cull (tomcc's cut reason), so keep it a setting with a fallback | engine (Lag fixes or a new session) |
| 7 | **Shadows: skip casters that cannot reach a receiver, and spread near-map updates.** Veloren draws a chunk into the sun map only if its light-space box overlaps visible receivers; Luanti spreads one shadow-map update over 16 frames; Vintage Story has separate culling for near and far shadow passes. `perf` already bounds the volume and keeps the far map; the near map still draws every frame. | partly (`perf`) | Cuts the near shadow pass further; shadows read 0.0–0.05 ms in our runs, so little to gain until item 1 shows otherwise. | M | medium (flicker) | Lighting |
| 8 | **Clouds updated 1 pixel in 16 per frame with reprojection** (Guerrilla, Horizon Zero Dawn, SIGGRAPH 2015: 20 ms → about 2 ms by updating one pixel of each 4×4 block per frame, and "10× faster or more" at half resolution). Works on Metal. | `perf`: 10 samples, no march behind terrain | Only if clouds show in the Retina numbers after render scale. | M | medium (ghosting) | Lighting |
| 9 | **Fewer draw records: batch cubes** (Cosmic Reach: 16³ logic chunks, 4×4×4 batched per mesh; Sodium: 8×4×8-section region arenas, a draw-command cache while the camera is still). MoltenVK runs a multi-draw as a CPU loop of Metal indirect draws, so cost scales with records (about 1400–2400 here). But view 8 vs 12 changed nothing in the runs. | per cube-layer-run | Low until draws are shown to cost. | M–L (face format needs a cube index) | medium | engine |
| 10 | **Binary greedy meshing** (cgerikj, MIT: 50–200 µs per 64³ chunk, 8 bytes per quad). Our faces per frame are only 95–160k, and merged quads break per-corner light, AO, tint and CTM unless they match. 0fps: noisy terrain only 25–55% fewer quads, flat ground 30× fewer. | none (per-block faces) | Low | L | high (shading) | none |

### Skip (reasons)

- **Mesh shaders / meshlets:** not in MoltenVK or KosmicKrisp (the latter lists them as future); on an M1 an Apple forum poster measured the mesh path 2× slower than plain draws.
- **MetalFX upscaling:** not reachable through Vulkan on macOS (`VK_EXT_metal_objects` gives a Metal object but not the command buffer; no example found). Use our own upscale in the render-scale pass. MoltenVK defaults `SWAPCHAIN_MIN_MAG_FILTER_USE_NEAREST=1`: if anything is ever scaled by the swapchain itself, it will look blocky, so keep the filtering in our own pass (as `render-scale` does).
- **GPU Hi-Z / compute culling:** possible (compute before the pass), but our CPU culling costs 0.4–0.9 ms, and `vkCmdDraw*IndirectCount` splits the Metal render pass in MoltenVK. If ever done, use a fixed count with zeroed entries. Weft's engine never uses `DrawIndirectCount`.
- **Software occlusion (Intel MSOC):** x86 SIMD only, repo archived.
- **Palettes and layer compression** (Cosmic Reach 1.3 GB → 200 MB on a flat world): memory is not our problem.
- **Distant Horizons / Voxy designs:** we already make far tiles on the GPU in levels (`lod.rs`); nothing to add.
- **Hytale, Teardown:** no usable public engine notes found (below).

## Technique catalogue

"Weft?" says whether we do it, with the file.

### Cosmic Reach (closed source, Java/libGDX, OpenGL; ideas only)

The developer is **FinalForEach** (itch.io, YouTube @finalforeach), solo. His real name is unconfirmed; I found nothing linking the names in Planner's brief ("Barchard", "Finji") to the game. It is not on Steam. The technical content is in his YouTube devlogs (timestamps below are from auto-captions and may be a few seconds off). His 94 itch.io changelogs (0.1.6 to 0.6.4) are one-liners with no numbers. The video Ronald most likely remembers is *I Remade Minecraft But It's Optimized* (892K views, 2023-08-20).

| technique | what and why | source | Weft? |
|---|---|---|---|
| Chunk batching | 64³ chunks: few draws but up to ~0.5 s remesh per edit; 16³: fast remesh but too many draws. Fix: 16³ logic chunks keep their own vertex data; 4×4×4 of them are combined into one batch mesh (same draw count as a 64³ chunk); an edit remeshes one chunk, then the batch is rebuilt from stored data; one shared index buffer. Cost: RAM, ~4 GB vertex data at 32-chunk radius. | *The Algorithm Behind Breaking Blocks*, https://www.youtube.com/watch?v=q1wu_T-wRJM, 2:51–5:50 | No: per-cube records (item 9). We have a shared quad index buffer (`quad_index`). |
| Compact chunk memory | Flat byte arrays; a uniform layer collapses to one reference; per-chunk palettes with nibble indices. 1.3 GB → 200 MB on a flat world. | https://www.youtube.com/watch?v=fz3Td2zlgro 1:33–3:16; palettes https://www.youtube.com/watch?v=nbI3MdvoHQI 1:37–2:18 | Not needed. |
| Vertex shrinking | Index buffers, three 10-bit coordinates in one float, partial greedy meshing with face size in the UV. VRAM 4 GB → 2 GB. Packing later dropped because of batching. | https://www.youtube.com/watch?v=2xbjP8XbVFY 0:19–4:13 | We pull 16-byte faces from storage buffers (`mesher.rs`). |
| Two-level frustum culling | Test regions of 16³ chunks (4096 chunks), then octants. Cave occlusion culling was only planned (shipped: unconfirmed). | fz3Td2zlgro 3:19–4:29; q1wu_T-wRJM 4:39–4:55 | Yes: region then cube (`world_render.rs:166, 1060`). Cave culling: item 6. |
| Hidden-face culling in the mesher | Skip a face whose neighbour is opaque or the same type. | fz3Td2zlgro | Yes (`mesher.rs`, row bitmasks). |
| Lighting | BFS flood fill 0–15, a second "darkness" queue for removal, coloured light as 3×4 bits in a short, same layered storage as blocks. Itch notes: 0.4.0 "improved lighting memory allocation", 0.6.0 "optimized sky lighting", 0.6.4 "optimized lighting algorithm and rendering" (no numbers). | https://www.youtube.com/watch?v=edaaFUflusk 0:14–3:37; removal q1wu_T-wRJM 1:29–2:49 | Yes, our own light (`world/light.rs`). |
| Threads | Main, world-gen and mesh threads; later a tick thread (0.1.37), sleeping idle threads (0.3.5, 0.4.2), object pooling (0.5.26), "optimized the meshing thread" (0.6.0). | https://www.youtube.com/watch?v=Wm5zPRTq5hc 1:46–3:48; itch changelogs | Yes, more of it (item 5 is about too many). |
| Opaque and transparent meshes split | Fixes depth artefacts. | Wm5zPRTq5hc 0:17–1:39 | Yes (layers). |
| World gen | Sample 3D noise every 4 blocks and interpolate: 125 samples instead of 4096, "30× faster"; biome tie-break noise only when it could change the result. | https://www.youtube.com/watch?v=wd2R4xOw6cM ~0:19–0:51 | We generate on the GPU (`terrain_gpu.rs`); caves use a 4-block grid (`terrain.rs` 5×5 heights). |
| Saving | Region files with an offset table, saved off the main thread; re-encoding each chunk to its smallest form took one save from 250 MB to 25 MB. | https://www.youtube.com/watch?v=okvk5qQEAhI 1:46–5:03 | Not render. |
| Simulation load | Random ticks only near the player (0.1.43), cut to 1/16 (0.4.13); piston remesh skips air chunks (0.5.14). | itch changelogs | Not render. |

**Not found** (so do not credit Cosmic Reach with them): any LOD (he says he is "not a fan of level of detail", Wm5zPRTq5hc 7:14), shipped Vulkan, numbers for any 2024–2026 optimisation, GC tuning, Reddit/Discord dev statements, Steam news.

### Other voxel engines (licence column says whether code may be reused)

| engine | technique | numbers quoted | licence | Weft? |
|---|---|---|---|---|
| Sodium | section graph with a 6×6 face-visibility bitfield in one `long`, BFS walk | up to 10–15% fewer sections (0.6.0-beta.3) | **PolyForm Shield 1.0.0 from 0.6.0** (non-compete; was LGPL-3.0 up to 0.5.x): ideas only | No (item 6) |
| Sodium | async culling thread (0.9), falls back to sync when the camera steps 32 blocks | none | same | No |
| Sodium | mesh scheduler: a mesh task costs 10 effort units, a re-sort 1, 11 units per thread per frame; ordered by age, distance and frustum; uploads capped by size; workers at normal − 2 | none | same | Partly: uploads are time-budgeted (`client/mod.rs:267, 857, 1472`) |
| Sodium | region arenas (8×4×8 sections), staging uploads, one multi-draw per region, draw-command cache while the camera is still | "up to +115%" on some machines (0.8.12 backport claim) | same | Paged buffers yes; per-region draw no |
| Sodium | skip back-facing face groups (7 ranges per section) | none | same | **Yes** (`camera_sides`, `world_render.rs:176`) |
| Sodium | 20-byte vertices | — | same | We use 16 B per face |
| Distant Horizons | quad-tree LOD, SQLite + ZSTD storage; thread presets | ~5× faster loading, 40% smaller databases, runs in 2–4 GB | LGPL-3 headers, repo also ships GPL-3 text: check per file | Own design (`lod.rs`) |
| Voxy | 32³ mip-chained sections, GPU BFS with frustum, Hi-Z and screen-size tests, 64 MB fenced upload ring | none | **All rights reserved**: ideas only | No |
| Nvidium | mesh shaders (NVIDIA only, needs Sodium) | none | LGPL-3.0 | Not portable |
| Vintage Story | mesh array recycling in size buckets; pools of 400k vertices / 900 chunks, defragment above 3%; separate near/far shadow culling | a large mesh took ~10 ms to allocate; heap grew over 1 GB after 40–50 block edits | `vsapi` is read-only proprietary ("All rights reserved", I read it): ideas only | We use free-range pages with best-fit (`world_render.rs:39`) |
| Luanti | mapblock BFS culler (falls back to a loop culler above range 500); shadow map update spread over 16 frames | none | LGPL-2.1+ | No (items 6, 7) |
| Veloren | meshing threads = CPUs − 3 (4–7 CPUs) or − 4 (8+); nearest-first meshing; 8-byte vertices with colour/light/AO in a per-chunk atlas; shadow casters only if they overlap visible receivers; render scale 0.25–2 | none | GPL-3.0: code only if we accept GPL | Render scale in flight; rest no |
| Hytale | no chunk, culling or instancing posts exist; only requirement pages ("each doubling of view distance covers roughly four times the area") | — | closed | — |
| Teardown | blog and third-party breakdowns only: voxel 3D textures as boxes ray-marched in the shader | — | closed | Different renderer |
| Vercidium voxel-mesh-generation | ~20% more triangles than greedy but "~390% faster" meshing, 4-byte voxel data | Ryzen 5 1600, 32³ | Apache-2.0 | No |
| cgerikj binary-greedy-meshing | see item 10 | 50–200 µs / chunk | **MIT** | No |
| block-mesh-rs | `visible_block_faces` ~40 M quads/s/core; greedy 3× slower, 1/3 the quads on a sphere | i7 2.5 GHz | MIT or Apache-2.0 | No |
| Embeddium | not checked | — | unverified | — |

### Meshing, GPU-driven drawing, Apple and MoltenVK

- **0fps (Mikola Lysenko):** greedy meshing and the AO rule. Per-vertex AO from three voxels (both sides solid → 0, else 3 − (side1 + side2 + corner)); flip the quad's diagonal by comparing the AO sums so shading doesn't skew; merge greedy quads only when AO is equal at all four corners. Weft?: AO exists; no greedy. https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/, https://0fps.net/2013/07/03/ambient-occlusion-for-minecraft-like-worlds/. The atlas article returned 404 (not read).
- **tomcc visibility:** https://tomcc.github.io/2014/08/31/visibility-1.html, `-2.html` (blog, no code). 15-bit connectivity, BFS also gives front-to-back order and rebuild priority; about 14% of frame time for those CPU costs. Known flaw: under-culling in ravines.
- **GPU-driven pipelines** (Haar and Aaltonen, Ubisoft, SIGGRAPH 2015): clusters of 64 triangles, two-phase Hi-Z culling; 2.3 ms for 250,000 objects on an Xbox One. Works through MoltenVK only as compute before the render pass. Not worth it at our about 1,400–2,400 records.
- **MoltenVK facts verified in its source** (by the sub-agent): `multiDrawIndirect` and `drawIndirectCount` are on; a multi-draw is a CPU loop of Metal indirect draws; `*IndirectCount` ends and restarts the render pass; memory typed transient + lazily-allocated becomes Metal memoryless; changing colour-attachment locations or input-attachment indices mid-rendering restarts the pass; `VK_KHR_dynamic_rendering_local_read` exists (1.4.0) for merging post passes into one. Swapchain: 2 or 3 images, 3 recommended; `displaySyncEnabled` is false only for `IMMEDIATE`; `VK_GOOGLE_display_timing` records but does not act. Pipeline cache stores MoltenVK's converted MSL (the slow SPIR-V→MSL step), not Metal's own binary.
- **Apple tile GPU rules** (WWDC20 10602, https://developer.apple.com/videos/play/wwdc2020/10602/): "First, opaque. Second, alpha test, discard or depth feedback. And third, and finally, translucent." Don't interleave. "If you don't need any data, don't load anything and clear instead." Only store what you need. Weft's draw order already follows this (layer-major buckets, then distant tiles, then translucent; `world_render.rs:1122`); the depth must be stored (post effects read it), so memoryless depth is not available while effects run.
- **Post pass bandwidth** (sub-agent's arithmetic): an RGBA16F full-screen pass at 3456×2234 moves ~62 MB out and ~62 MB in. Our scene images are the surface format (BGRA8, ~31 MB each way). `perf` already merges effects into one pass.
- **Frame pacing:** Apple WWDC21 10147, Zed's post, Apple forum 733033 (all in item 4). `CAMetalDisplayLink` (macOS 14+) is what an Apple engineer recommended after drawable-size-change stalls; MoltenVK never uses it.
- **Threads and QoS:** Eclectic Light's QoS table (background 9, utility 17, user-initiated 25, user-interactive 33; background never runs on P cores); JUCE forum: on M1 POSIX priorities below about 5 kept threads on E cores (60× slower jobs), anecdotal. The `libc` crate (MIT or Apache-2.0) exposes `pthread_set_qos_class_self_np`. I could not read what the `thread-priority` crate does on macOS (*unconfirmed*).
- **MoltenVK config worth trying** (env vars): `MVK_CONFIG_PREFILL_METAL_COMMAND_BUFFERS=1` encodes at end-of-command-buffer (a tuning knob; no measurement), `MVK_CONFIG_SHOULD_MAXIMIZE_CONCURRENT_COMPILATION=1` (macOS 13.3+, parallel pipeline compiles), `MVK_CONFIG_FAST_MATH_ENABLED` (default already on). Not measured here.

## Licences (what may be copied)

- **Code that may be used:** binary-greedy-meshing (MIT), block-mesh-rs (MIT or Apache-2.0), Vercidium (Apache-2.0), KosmicKrisp (MIT), the `libc` crate (MIT or Apache-2.0). Anything taken goes in `LICENSES`/NOTICE with its text.
- **Copyleft:** Veloren (GPL-3.0), Luanti (LGPL-2.1+), Distant Horizons (LGPL-3 / GPL-3 mixed), Nvidium (LGPL-3.0): ideas only unless Ronald decides on a copyleft dependency.
- **Ideas only:** Sodium 0.6+ (PolyForm Shield), Voxy (all rights reserved), Vintage Story's API (read-only proprietary), Cosmic Reach, Hytale, Teardown (closed source), Intel MSOC (Apache-2.0 but unusable on ARM).
- No code was taken for this report.

## Not found / not checked

- Hytale: no engine posts. Teardown: could not find Gustafsson's talk, so no timestamps.
- Shadow-cascade caching and section-impostor numbers: no authoritative source.
- KosmicKrisp's Vulkan 1.4 claim is from a search snippet (Phoronix returned 403).
- Whether Cosmic Reach's cave culling shipped.
- Whether `Retina` windows of 3456×2234 are real surfaces (macOS may clamp): Tester's size probe is queued by Lag fixes.
- My reading that GPU times are clock-ramp-inflated is a hypothesis; item 1 tests it.
- No code, game run or build was done; the report only reads.

## Sources

- **Cosmic Reach:** [itch.io devlog](https://finalforeach.itch.io/cosmic-reach/devlog); videos [fz3Td2zlgro](https://www.youtube.com/watch?v=fz3Td2zlgro), [Wm5zPRTq5hc](https://www.youtube.com/watch?v=Wm5zPRTq5hc), [2xbjP8XbVFY](https://www.youtube.com/watch?v=2xbjP8XbVFY), [edaaFUflusk](https://www.youtube.com/watch?v=edaaFUflusk), [nbI3MdvoHQI](https://www.youtube.com/watch?v=nbI3MdvoHQI), [q1wu_T-wRJM](https://www.youtube.com/watch?v=q1wu_T-wRJM), [okvk5qQEAhI](https://www.youtube.com/watch?v=okvk5qQEAhI), [wd2R4xOw6cM](https://www.youtube.com/watch?v=wd2R4xOw6cM).
- **Engines:** [Sodium](https://github.com/CaffeineMC/sodium), [Voxy](https://modrinth.com/mod/voxy), [Nvidium](https://github.com/MCRcortex/nvidium), [vsapi](https://github.com/anegostudios/vsapi), [Veloren](https://github.com/veloren/veloren), [Luanti](https://github.com/luanti-org/luanti), [Distant Horizons](https://gitlab.com/distant-horizons-team/distant-horizons).
- **Voxel visibility:** [tomcc part 1](https://tomcc.github.io/2014/08/31/visibility-1.html), [part 2](https://tomcc.github.io/2014/08/31/visibility-2.html), [cod.ifies.com](https://cod.ifies.com/voxel-visibility).
- **Meshing:** [0fps](https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/), [binary greedy](https://github.com/cgerikj/binary-greedy-meshing), [block-mesh-rs](https://github.com/bonsairobo/block-mesh-rs), [Vercidium](https://github.com/Vercidium/voxel-mesh-generation), [Exile](https://thenumb.at/Voxel-Meshing-in-Exile).
- **GPU-driven:** [Haar and Aaltonen 2015](https://advances.realtimerendering.com/s2015/aaltonenhaar_siggraph2015_combined_final_footer_220dpi.pdf), [Tellusim Metal MDI](https://tellusim.com/metal-mdi/), [Apple forum 722047](https://developer.apple.com/forums/thread/722047).
- **Apple and MoltenVK:** [MoltenVK](https://github.com/KhronosGroup/MoltenVK) (README, user guide, configuration parameters), [WWDC20 10602](https://developer.apple.com/videos/play/wwdc2020/10602/), [WWDC21 10147](https://developer.apple.com/videos/play/wwdc2021/10147/), [Metal HUD](https://developer.apple.com/tutorials/data/documentation/xcode/monitoring-your-metal-apps-graphics-performance.md), [KosmicKrisp XDC 2025](https://www.lunarg.com/wp-content/uploads/2025/10/XDC-2025-KosmicKrisp-Overview.pdf), [Eclectic Light on QoS](https://eclecticlight.co/2025/05/09/what-is-quality-of-service-and-how-does-it-matter/), [JUCE thread](https://forum.juce.com/t/fr-thread-priority-vs-efficiency-performance-cores/49025), [Zed 120 fps](https://zed.dev/blog/120fps), [Apple forum 733033](https://developer.apple.com/forums/thread/733033).
- **Clouds:** [Horizon Zero Dawn, SIGGRAPH 2015](https://www.advances.realtimerendering.com/s2015/The%20Real-time%20Volumetric%20Cloudscapes%20of%20Horizon%20-%20Zero%20Dawn%20-%20ARTR.pdf).
