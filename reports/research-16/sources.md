# Research round 16: Voxy, Distant Horizons and Horizon, read from source

Ronald (relayed by Planner, 11-05): "u can also check out voxy and distant horzions code and also horiztion the rust mc ports code for some ideas as well".

Three sub-agents read the sources as text on the web (nothing was downloaded). Paths are given so each claim can be checked. "Inferred" means the reader could not confirm it line by line. **We take ideas only, and copy no code from any of them.**

## Licences

| project | licence | what we may do |
|---|---|---|
| Voxy (github.com/MCRcortex/voxy, dev) | `LICENSE.md`: "Copyright 2025 MCRcortex. All rights reserved. Do not redistribute." plus a no-warranty clause. Not open source. | Ideas only. No code, no shader text, no close transliteration. |
| Distant Horizons (gitlab.com/distant-horizons-team/distant-horizons-core) | LGPL v3 (`LICENSE.LESSER.txt`); logos CC BY-SA 4.0 | Ideas only (our engine is not LGPL). |
| Horizon (github.com/horizon-rust/horizon) | MIT (root `LICENSE`); players bring their own Minecraft assets | Code could be ported with the MIT notice kept. Ideas are enough here. |

**"horiztion" is Horizon**: a from-scratch Rust rewrite of Minecraft Java 26.3, checked against vanilla with differential tests. It is on wgpu 30 with Vulkan, Metal, DX12 and GL backends. Created 2026-10-03, 6.5k stars, about 19.7k commits, pushed to daily. Other names that came up but were not read: Far-Beyond-Dev/Horizon (a server framework, not Minecraft), Valence, Pumpkin, FerrumC (servers), Rezcraft and Minerust (small wgpu clones).

## How they compare to our distant terrain

Ours (`crates/engine/src/client/lod.rs`, `terrain_gpu.rs`):
- **Tiles.** Quadtree tiles of 32×32 cells; levels 1–4 are used, about 801 tiles out to 4096 blocks (counted from `pick`). Level 5 is never drawn.
- **Building.** A compute pass samples heights (64 tiles a frame at most, read back 2 frames later). 3 low-priority CPU threads then mesh them.
- **Format and drawing.** 16-byte face records; one indirect draw per tile; per-tile frustum test only.
- **Stand-ins.** A missing tile is covered by its nearest made ancestor until *all* of its children are ready.
- **Overlap with the cubes.** A discard mask where cubes stand, plus a dithered 24-block fade band.

| | Voxy | Distant Horizons | Horizon | Weft |
|---|---|---|---|---|
| Data | 32³ voxel sections, 64-bit voxels, levels 0–4 (to 512-block sections); only from chunks the player has seen | Columns of variable-length 64-bit points (`FullDataPointUtil.java`); 64×64 columns per section; levels to 16384-block roots | 16×16-column tiles, each column 1–4 material spans (min, max, state, biome, light) | heightfield sampled on the GPU from the generator: no stored data, made fresh |
| Reach | default 16 sections ≈ 8192 blocks (inferred) | default 256 chunks (4096), up to 65536 | world-wide tiles | 4096 (`core:lod_distance` 256) |
| Meshing | CPU (Java), scanline merge capped at 16; **8-byte quads** (`quad_format.glsl`) | CPU boxes, each side cut against the neighbour column; greedy merge per direction; 16-byte vertices plus index | separate sampler and mesher pools (8 + 3 beat 6 + 5: 4,336 tiles in 18.6 s vs 23.9 s) | 3 low-priority threads; 16-byte faces; no merge |
| Culling | GPU traversal with Hi-Z (5 dispatches); bounding boxes rastered against Hi-Z; camera-facing face directions only; two-phase occlusion (inferred) | per-section frustum, near-to-far sort; no occlusion; cave faces under y 60 that are unlit dropped | cave BFS on its own thread; GPU compute cull; **two-phase depth test of section and tile boxes** | per-tile frustum only |
| A missing finer tile | parent draws while children are requested; swaps only when all children are ready (`NodeManager.java`) | parent renders until all 4 children's subtrees can; "-1 tolerance" (one level coarser counts as done); old buffer kept until the new one is uploaded | a coarse "backdrop" tile while a leaf loads; old mesh kept until its replacement fades in | nearest made ancestor hides every made tile under it until all are ready |
| Load order | mesh queue by level, then retries | **coarse levels first**, then nearest (Manhattan); parents' data copied down into empty children (inferred) | priority distance×4 + bias for a column's first mesh | nearest-first, all levels mixed |
| Threads | cores/1.5 at low Java priority, weighted by queue size | BALANCED preset: 50% of cores; one shared cap over 6 executors, mesh loading highest ("Prioritize loading LODs for rendering", 3.3.0); world gen paused when the camera moves faster than 20 blocks/s | shared job pool, in flight capped at workers×32 | **3 threads** at low priority |
| Uploads | 64 MB persistent ring, fenced; requests ≤ 50 a frame, scaled by (1 − queue/4000)² | on the render thread within half a frame (`1000/fps/2` ms) | 256 MiB arena pages, GPU-copy defrag, buffers freed 4 frames later on a background thread | no cap (`update` takes every finished mesh) |
| Seam with near terrain | LOD in its own target; vanilla chunk boxes rasterised into a depth bound, LOD discards behind it; alpha fade only over the last 10% of reach | clip distance = vanilla render distance × 0.2–0.9; 4×4 Bayer dither to 1.5×; optional full-screen fade pass | backdrop tile plus near-column cutout | dithered 24-block band (`faded()`), discard mask (`lod_hidden`) |

## Techniques worth taking, ranked for us

| # | technique (from) | what it buys us | cost | where |
|---|---|---|---|---|
| L1 | **More, higher-priority LOD mesh threads** (DH: 50% of cores, meshing first; Horizon: tuned sampler/mesher split). Our 3 low-priority threads make about 20 tiles a second once the first burst is done (PC run log, below), so the 801 tiles take about 40 s. | Tiles arrive several times faster, so the coarse boxes go away sooner. This is the main lever for the "boxes" Ronald saw. | S | `lod.rs:108-118` pool size and priority; engine |
| L2 | **Coarse levels first** (DH's load order) **plus one level of tolerance** (a tile one level coarser counts as done) | The whole view gets a stand-in at once, and it refines without holes or a parent hiding 4–64 finished children | S | `lod.rs` `pick` sort key and the stand-in rule; engine |
| L3 | **Keep the old mesh until the new one is in** (DH, Horizon) and a **partial stand-in**: draw a parent only over its missing children's area (clip by a per-tile mask, or draw the made children and the parent's missing quarter) | Ends the "a single slow L1 tile keeps a whole L2 box area showing" case | M | `lod.rs:658-695`; engine |
| L4 | **Draw only the face directions that face the camera, per tile** (Voxy's cmdgen; our cubes already do this with `camera_sides`) | Roughly half the LOD vertex work. Back faces are culled after the vertex shader today. | S–M (faces sorted by direction in `lod::mesh`, 6 ranges per tile) | `lod.rs` mesh, `world_render.rs` LOD records; engine |
| L5 | **A request budget that follows the mesher queue** (Voxy), and an **upload time budget** (DH: half a frame) | No burst of 200 tile uploads in one frame (we saw 223 made in one second after a /tp) | S | `lod.rs:637-644, 851-878`; engine |
| L6 | **Two-phase depth occlusion of tiles and cubes** (Horizon: 15–30% fewer solid indices at RD12, 1.6 ms saved at RD32 on an M4 Pro; Voxy) | Hills hide what is behind them: fewer tiles and cubes drawn and shaded | M–L | engine; after the upstream sync reworks `mesh_draws` |
| L7 | **Cave connectivity BFS for drawing** (Horizon on its own thread, with a sight line past 60 blocks; also round 12 #6) | Underground views stop drawing the surface's caves and vice versa | M | engine; after the sync |
| L8 | **8-byte quads with a per-tile y base** (Voxy) | Half the LOD memory and upload | M | `gpu_layout.rs`, `lod.rs`, `terrain.wgsl`; engine |
| L9 | **A separate LOD target with an alpha fade**, or clipping the LOD by the cubes' depth instead of `discard` (Voxy, DH's fade pass) | No `discard` in `fs_lod` and the fade pipelines: keeps Apple's hidden-surface removal for about 40% of near meshes | M | engine plus the pack's `fs_lod` |
| L10 | **Background buffer freeing with a graveyard** (Horizon: 8–20 ms free spikes on lavapipe) | Removes stalls when many tiles are evicted at once (unmeasured for us) | S | `backends/vulkan` defer list already does part of this |

Skip:
- **Voxy's full GPU traversal.** With 801 tiles, CPU selection is cheap.
- **Count-indirect draws.** They split Metal passes in MoltenVK; Horizon's version needs `MULTI_DRAW_INDIRECT_COUNT` and does not run on Metal either.
- **Stored LOD data on disk** (Voxy, DH). Ours is generated: no import, no storage, and the world's seed is the source.

## Horizon's numbers (self-reported)

- RTX 5080 at 720p, RD32 forest: 1389 fps (1% low 504), against 1203 for vanilla ([benchmarks/perf/windows-9800x3d-5080-2026-10-29.md](https://github.com/horizon-rust/horizon/blob/main/benchmarks/perf/windows-9800x3d-5080-2026-10-29.md)).
- M4 Pro (loaded) at 1080p, RD32: 165 fps, GPU 2.9–5.7 ms, GPU-bound ([benchmarks/perf/macmini-m4pro-loaded-2026-11-01.md](https://github.com/horizon-rust/horizon/blob/main/benchmarks/perf/macmini-m4pro-loaded-2026-11-01.md)).
- On Metal, wgpu replays a multi-draw as one call per entry. So past 1,600 sections Horizon "flattens" its draws: compute writes one index list per page and it issues one draw per page run (`gfx/flat.rs`). On the M4 Pro that cost 0.4 ms more GPU at RD16 and saved 0.5–0.8 ms of CPU, and 1.3 ms of CPU at RD32. MoltenVK also turns indirect multi-draws into a CPU loop (round 12), so this is relevant to our Mac's 3–11k draw records.

## Sources (main files)

- **Voxy:**
  - Traversal: `src/main/java/me/cortex/voxy/client/core/rendering/hierachical/HierarchicalOcclusionTraverser.java`, `.../shaders/lod/hierarchical/traversal_dev.comp`, `screenspace.glsl`.
  - Hi-Z: `HiZBuffer2.java`, `hiz/hiz.comp`.
  - Drawing: `MDICSectionRenderer.java`, `lod/gl46/cull/raster.frag`.
  - Quads and meshing: `lod/quad_format.glsl`, `RenderDataFactory.java`, `ScanMesher2D.java`.
  - Memory and uploads: `UploadStream.java`, `AllocationArena.java`, `NodeManager.java`, `NodeCleaner.java`.
  - Threads: `UnifiedServiceThreadPool.java`.
- **Distant Horizons** (core, `core/src/main/java/com/seibel/distanthorizons/core/`):
  - Data: `pos/DhSectionPos.java`, `util/FullDataPointUtil.java`, `util/RenderDataPointUtil.java`.
  - Meshing: `dataObjects/render/bufferBuilding/ColumnBox.java`, `LodQuadBuilder.java`, `LodBufferContainer.java`.
  - Quadtree: `render/QuadTree/LodQuadTree.java`, `LodRenderSection.java`.
  - Threads: `config/eventHandlers/presets/ThreadPresetConfigEventHandler.java`, `util/threading/RateLimitedThreadPoolExecutor.java`, `PriorityTaskPicker.java`.
  - Uploads and drawing: `render/RenderThreadTaskHandler.java`, `util/RenderUtil.java`, `render/RenderBufferHandler.java`.
  - Shaders: `shaders/terrain/gl/frag.frag`, `shaders/fade/gl/vanilla_fade.frag`.
  - Changelogs: [releases](https://gitlab.com/api/v4/projects/distant-horizons-team%2Fdistant-horizons/releases).
- **Horizon** (`crates/horizon-client-render/src/`): `mesher.rs`, `mesher/packed.rs`, `streaming.rs`, `gfx/occlusion.rs`, `gfx/gpu_culling.rs`, `gfx/cull.rs`, `gfx/lod.rs`, `lod.rs`, `gfx/arena.rs`, `gfx/flat.rs`.
