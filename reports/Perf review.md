# Perf review: the game's hot paths, ranked by gain per effort (11-05)

By Upstream, with the `rust-perf-review` skill (`.claude/skills/rust-perf-review/`). The base is
dev at 18e325a0 and later. Three read-only reviews make up the detail, in `reports/perf-review/`:
`render-cpu.md` (the Vulkan backend and frame recording), `client-update.md` (the per-frame
update, near terrain, the server feeding a client) and `far-terrain.md` (distant tiles and land
features).

**Status: draft.** The findings come from the engine's own logs and from reading the code. The
runs that confirm them are under way (see "Measurements"). Each finding says whether its number
is measured or estimated. The ranking changes once samply profiles and the Mac A/B are in.

Round 19 (`reports/Research round 19.md`) already ranks the GPU side from the literature: TAAU,
froxel mist, effect specialisation, cloud reprojection, draw order, 8-byte faces and the water
copy. This review doesn't repeat those rows. It adds what the code itself shows.

## What the numbers say first

- **The Mac is GPU-bound in play.** The M3 Max at 3456x2234 shows GPU 16.2 of 16.7 ms: world 7.6, effects 6.8. Our own Rust per frame is small: `record` takes 0.3-0.5 ms. Two numbers there need care:
  - **"queue" (4.5-7 ms):** it is `vkQueueSubmit2`, which is MoltenVK encoding Metal, but it also includes fetching the drawable from the display.
  - **"world":** it includes the translucent, entity, sky and particle draws, because MoltenVK takes timestamps only at encoder ends. Use a Metal System Trace to split them.
- **The PC is GPU-bound at about 0.72 ms a frame.** `record` is 0.1 ms. No CPU finding gains it more than about 0.1 ms in play.
- **Far terrain fills at a rate set by threads divided by CPU per tile.** The far-tile pool has `cores/3` threads, clamped to 3..6.
  - Mac: 4 threads, 15 ms heights + 8 ms mesh a tile, about 175 tiles/s predicted. Measured: 175/s, 9.4 s to fill.
  - PC: 6 threads, 18.9 ms a tile, 317/s predicted. Measured: 323/s.
  - A weak 4-core: 3 threads, about 15 s by the same rule.
  - Every ms cut from a tile shortens the fill in proportion.
- **The long frames are mostly our own scripts.** The frames of 400-620 ms in the Mac logs are screenshots (the PNG is encoded on the main thread). The 200-370 ms ones are shader-pack reloads. Real loading hitches are 30-70 ms.
- **The 3-5 ms "cpu update" while loading is the frame budgets at work.** In flight, though, 6-8 ms a frame runs outside any budget (finding 9).

## Ranked findings

Gain per effort, all platforms. "Mac", "PC" and "weak" mean the M3 Max, the RTX 5090 PC, and a 4-core CPU with an integrated GPU. Owners: Terrain (base's land features), Distant terrain (LOD), Lighting (effects and HDR), Upstream (engine core).

| # | Finding | Where | Evidence | Change | Gain | Effort | Owner |
|---|---|---|---|---|---|---|---|
| 1 | Without `drawIndirectCount` (Macs) every terrain record is drawn: about 35k mostly empty Metal draws a frame | `world_render.rs` draw loop | Round 19; MoltenVK loops on the CPU, one Metal draw per record (`MVKCmdDraw.mm`) | **Done and merged:** `fix/indirect-cap` 09dc1c68. PC check: no missing terrain after fast turns or a teleport; about 16 % of the records issued. The PC flight is unchanged. **Mac (Tester, 3456 auto scale, creative sprint):** 100 against 93 fps; world GPU 5.23 against 5.78 ms; queue 1.3-1.6 against 2.5 ms; 2190 of 11950 records issued | **Measured, Mac:** −0.55 ms world GPU, −1.0 ms queue, about 0.7 ms a frame (smaller than Round 19's hope) | done | Upstream |
| 2 | Base's faces callback recomputes the same column for the top block and each wall band of a steep cell, up to 9 times | `client/lod.rs:338-354`, `mods/base/src/land.rs:106-113` | Log: 63-155 µs a face call on far tiles, 2.8-4 µs where columns are cached; 89-147 calls a tile | **In progress:** Terrain, `claude/river-region-lru` 76b47a14: a one-entry thread-local memo keyed (seed, x, z, height) | Mesh 5-26 → ~1-3 ms a tile: fill about −30 % on all CPUs | 0.5 h | Terrain |
| 3 | `tarns` and `tarn_streams` evaluate the whole land at every column before checking for a tarn | `mods/base/api/src/land.rs:829, 847` | Estimate: heights rose from ~1.8 µs to 13-18 µs a column since tarns; `land_time` doesn't time them | **In progress:** Terrain 3c84d1a3: the site is looked up first. Timed per far tile: tarns 3.95 → 0.03 ms, tarn_streams 4.21 → 0.22 ms, values identical | Heights 15 → ~6-8 ms a tile (±50 %), near generation too | 1-2 h | Terrain |
| 4 | Meshing and `describe_lod` wait in the pool's FIFO behind up to 256 heights jobs | `client/lod.rs:149, 1031`; `terrain_gpu.rs:491` | Log: 3 s of "0 tiles made" at load and after a teleport; 1-1.8 s from asked to made | **In progress:** Distant terrain, on its own branch: heights in flight capped at about 3× the LOD threads, `describe_lod` started on a dimension change; benched on time to first tiles | First far tiles 3 s → ~0.3-0.6 s; CPU per tile unchanged | 2-3 h | Distant terrain |
| 5 | `surface_block` under `ONE_COLUMN` computes 6 columns per face call (column, 4 corners, column again) | `server/terrain.rs:891, 910-929` | Code; matches the 63-155 µs a call | Compute once, skip the carve test for far faces, or pass the GPU's top block in `FaceArgs` | With #2, mesh ~0.3-1 ms a tile, smooth tiles included | 1-3 h | Upstream |
| 6 | Every `Cmd::Barrier` targets the host stage. MoltenVK then goes over every buffer and image, about 24 times a frame | `backends/vulkan/src/lib.rs` `memory_barrier` | MoltenVK source | Tried: `fix/host-barrier-once` ff78b5d0, one host barrier at frame end. **No gain on the Mac** (A/E: queue 1.27 vs 1.26 ms, frame +0.14 ms). Reverted on `revert/host-barrier` 26890a0c, so the engine stays Weft's there | none | — | Upstream |
| 7 | The screenshot PNG is encoded on the main thread | `app.rs:3176-3190` | Log: every frame over 400 ms is a screenshot second | **Done and merged** (f8a9b4e8): `fix/shot-encode-thread`, joined on quit. PC: a shot's frame ~70 → ~22 ms at 1280x720 | Players' F2 no longer stalls 0.5 s; test runs get honest frame times | 0.5 h | Upstream |
| 8 | Thread priorities: `lower_priority` does nothing on macOS, and nice +1 on Linux is too weak | `crates/engine/src/lib.rs:125-146` | Code | **Done:** `fix/thread-priorities` 04424bf0: QoS utility on macOS, nice 10 on Linux (Windows already below normal); env vars to compare. Distant terrain: on the PC, half the cores at normal priority took near cubes from 1.5 to 4.7 s after a teleport, so the LOD count stays cores/3 until per-tile CPU drops | Frame stability while loading; most on weak CPUs | 1 h | Upstream |
| 9 | 6-8 ms a frame of untimed work in `Client::update` during flight and after a teleport | `client/mod.rs`, `receive.rs:442`, `world.rs:420, 849-949`, `mod.rs:1827` | Log: cpu update 11.6-13.1 ms in CPU-bound flight frames, while the budgets allow 2 × 2-3 ms | **Spans done:** `spans/update-split` 84b9d31c (`update split` line); then budget the biggest. Suspects: `end_batch`, `start_jobs`, uploads near the player | Mac flight frames 17-21 → ~12-14 ms; more on weak CPUs | 1 h to measure, then 2-6 h | Upstream |
| 10 | Client terrain finishing shares `background()` with the server's CPU generation (**tried:** `fix/client-terrain-lane`, no change on the PC: near cubes +1.3-1.4 s and far tiles +2.3-2.4 s after a teleport on both; unmerged, a 4-core test would need `taskset`) | `terrain_gpu.rs:563`, `client/terrain.rs:831`, `server/load.rs:257` | Estimate: the server took ~8.8 of 12 threads' time in the first second, the client 4.3 | A separate lane, or priority, for the client's finishing | Near terrain ~1.5-2× sooner on the Mac, more on a 4-core (2 threads there) | 3-6 h | Upstream |
| 11 | Release profile: no LTO, 16 codegen units | `Cargo.toml` `[profile.release]` | **Measured** (PC, Tester 2): far tiles' heights 13.2 → 10.9 ms (thin) / 10.6 (fat), mesh 4.4 → 3.9 / 3.8 ms, fill done 5.2 → 4.4 / 4.3 s; flights unchanged (GPU-bound). Builds 0:54 → 5:39 / ~7 min | **Done:** a `dist` profile (thin LTO, one codegen unit) for what players get, `dist/profile` 9ee4ce62; the crew keeps `release`. CI's release jobs should build `--profile dist` (the GitHub Actions session) | Far tiles −17 % heights, −10 % mesh on every platform for players | 0 (CI builds slower) | Upstream |
| 12 | Each of the 9 feature callbacks rebuilds the column's climate and ground (`Here::new`, ~10 noise octaves). `custom_heights` loops feature by feature, so no memo can help | `mods/base/src/land.rs:116`; `client/lod.rs:1054-1058` | Code | **In progress:** Terrain d25ef0d3 makes the nine callbacks one (`customs`, "land") with the shared ground worked out once a column. All CPU features per far tile on the Mac: 12.25 → 3.56 ms (Terrain's timed example); PC bench rivlru_faces queued | After #3, heights ~6-8 → 3-4 ms a tile | 3-5 h | Terrain + Distant terrain |
| 13 | Texture loads block the frame, once per mesh | `textures.rs:286-322` | Log: 97 ms in the first second, single loads of 15-32 ms | Coalesce to once a frame, preload at join; async later | Loading hitches 15-32 ms gone | 1-2 h | Upstream |
| 14 | The occlusion's depth chain: a 31 MB depth copy, ~12 one-level dispatches each behind a full barrier, so ~12 Metal compute encoders | `world_render.rs:2064-2105` | Code and MoltenVK source | **Done:** `fix/hiz-chain` bad29cbd: level 1 read straight from the depth texture (no copy), 6 levels a dispatch in workgroup memory (3 instead of 12). **PC measured:** flight frames 0.684 → 0.613 ms (−10 %), GPU 0.644 → 0.576; no missing terrain after spins and a teleport | PC −10 % frame; more expected on the Mac and iGPUs (bandwidth, encoders) | done | Upstream |
| 16 | **(Weft bug)** The culling's faces counter bound at `buckets × 8` bytes: a storage-buffer offset off `minStorageBufferOffsetAlignment` for some bucket counts | `world_render.rs` `cull` | Validation: `VUID-VkWriteDescriptorSet-descriptorType-00328` on newer dev (Tester 2) | **Done:** `fix/faces-counter-align`, a 256-byte boundary; the validation re-run is clean | Correctness on every GPU | done | Upstream |
| 15 | Bug: the depth copy claimed the attachment layout after `EndPass` had made the image shader-readable, then left it as an attachment untracked; the effects sampled it in the wrong layout | `backends/vulkan/src/lib.rs` `CopyDepthToBuffer` | Code; a validation-layer run is queued | **Done and merged:** `fix/depth-copy-layout` 940a1b2f; the validation layer's 10× oldLayout-01197 errors are gone. Marked (Weft bug) | Correctness on AMD/Intel (undefined behaviour today) | done | Upstream |

**Smaller, in the appendices:**
- **Render CPU:** texture upload barriers per mip; O(n²) structure lookups in `draw`; the shadow pass culling every cube on the CPU; per-frame allocations in `draw`; translucent runs re-sending all state; MoltenVK encoding on the main thread.
- **Client update:** mesh upload copies, tint blends ×9, the server rescanning its backlog per cube, `ColumnCache` waits, per-cell rescans near the camera, the storage lock held across disk reads.
- **Far terrain:** `host::invoke` overhead; cave interpolation twice per node; site caches per thread; river regions built 16×; the Mac's 4 LOD threads.

## Measured gains so far (PC flight bench, 1280x720)

- `fix/hiz-chain` (#14): 0.684 → 0.613 ms a frame (−10 %).
- `fix/barrier-masks` (render #5, b2d9b08c): image barriers on the stages their layouts use, and the acquire at colour output. 0.612 → 0.579 ms (−5.4 %), GPU 0.574 → 0.540. Sync validation: no hazards seen. `WEFT_ALL_BARRIERS=1` compares.
- Together: 0.684 → 0.579 ms, −15 %. The Mac gains are to measure, with both in its next build.
- The `dist` profile: far tiles' heights −17 %, mesh −10 %.

## What the update split showed on the Mac (Tester, 179 s run, dev 9ae94af9)

The client update's parts, added up over the run (the sum of per-second averages, in ms a frame), with each part's worst frame:

| Part | Sum | Worst frame |
|---|---|---|
| solid cells (`update_solid`) | 114 | 32 ms |
| uploads (budget) | 87 | 30 ms |
| tints | 46 | 38 ms |
| take terrain / end batch | 16 | 40 ms |
| uploads near the player | 7 | 28 ms |

- **Correction (LOADCPU on the PC):** "solid cells" is not CPU work. 95 % of it is the cube bits' `write_buffer`, and the bits themselves take 0.02 ms. The Vulkan backend's first write of a frame waits for that frame slot's fence (`ensure_slot_ready`). The client update writes first, so it carries the frame's GPU wait and can't overlap the GPU. That is why the Mac's flight frames looked CPU-bound (gpu wait 0, cpu update 12 ms). The tints' and uploads' worst frames are likely the same wait.
- **Fix:** `fix/early-writes` 3727f9a0. Writes made before the slot is ready stay on the CPU and are staged right after its wait, in order. `WEFT_EARLY_WAIT=1` brings back the old behaviour for comparison. Expected: the update overlaps the GPU, gaining up to its real CPU time a frame on the Mac while loading and in flight. To measure on the Mac.
- `fix/texture-loads-once` and `fix/tints-once` showed no gain on the PC (within noise; the colours are fine). Their Mac evidence was that wait in disguise, so they stay unmerged until a Mac run with early-writes shows they matter.
- MoltenVK's own log says encoding a command buffer averages 1.63 ms (max 14.6), and its pipeline compiles 0.62 ms (74 of them).
- The 400-600 ms screenshot frames are gone, now that the PNG is encoded on its own thread.

## Platform-specific wins

- **The Mac (MoltenVK, Apple GPU):**
  - #1 the draw cap and #6 the host barrier, both done and awaiting the Mac A/B.
  - #14 the occlusion's depth chain.
  - The Mac items in Round 19 (draw order for HSR, the water copy).
  - Split the submit before the first pass that draws to the screen, so the GPU starts before all encoding is done (render-cpu #2).
  - QoS classes for threads (#8).
- **Linux:** nice +10 for the bulk pools (#8); PGO and BOLT on top of #11.
- **Windows:** thread priority (#8). Nothing else here is Windows-only.
- **Weak CPUs (4 cores):** #2, #3, #5 and #12 matter most there: the far fill is about 15 s on 3 threads today, about 2.5 s if all land. Also #8 and #10, and the small per-frame allocations.
- **Weak GPUs (integrated):** Round 19's TAAU and first-start quality (the latter now in dev as `gpu-preset`), plus #14 (bandwidth).

## Done on branches (small and proven, or proving)

- `fix/indirect-cap` 09dc1c68: #1.
- `fix/host-barrier-once` ff78b5d0: #6.
- `fix/depth-copy-layout` 940a1b2f: #15.
- All three together: `macrun/combined` 8d0a3913. The PC correctness run is MACRUN; the one Mac session follows.
- `lto-bench/thin` 1c8b7854 and `lto-bench/fat` 26b0ec22: #11, bench only.

## Measurements

- **PC, Tester 2:**
  - MACRUN correctness: no missing terrain after fast turns or a teleport with `--no-indirect-count`; validation layers.
  - The LTO A/B: the flight bench twice per tree, and far-fill time.
  - samply profiles of the far fill and the flight bench on dev, once samply is installed (Planner's yes). These confirm or rank #2, #3, #5, #9 and #12.
- **Mac, Tester's one session:**
  - A: default; B: `--no-indirect-cap`; C: Metal System Trace plus MoltenVK's performance log; D: Lighting's fx-half; E: `--host-barrier-each`.
