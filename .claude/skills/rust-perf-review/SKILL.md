---
name: rust-perf-review
description: Review chiacraft/Weft code for speed: the Rust CPU side (meshing, worldgen, distant terrain, the per-frame client update, threads, allocation) and the Vulkan GPU side (GPU-driven terrain, passes, barriers, MoltenVK on the Mac). Use when asked to make the game faster, review hot paths, find why a frame or a far tile is slow, profile on the Mac or the PC, or check a perf change. Measure first; every finding needs a number.
---

# Speed review: Rust and Vulkan, measured

Rule one: measure, don't guess. A finding is a file:line, the evidence (a number from a log, a
profile or a bench), the change, its expected gain, and who owns that area. Rank findings by
measured gain per effort. Check every platform: Windows and Linux (NVIDIA and AMD, also weak
iGPUs), the Mac (Apple GPU through MoltenVK), and weak CPUs (4 cores). List the platform-specific
wins separately.

## 1. Where the time goes: read it before reading code

The engine reports its own time. Use it first.
- `--log frame=debug`, once a second:
  - `N fps, frame avg | cpu update, ui, acquire, record, submit | gpu upload, compute, shadows, world, translucent, effects, ui | server …`
  - `submit split`: gpu wait, acquire, record, queue, present. On the Mac "queue" is MoltenVK encoding Metal on the CPU.
  - `terrain draws`: records issued a frame, of all, how many written.
- `--log load=debug`, once a second: `distant terrain`: tiles made, wanted and asked; heights, mesh and face-call µs per tile.
- Slow frames (record over 20 ms) log their timed steps (`diag::span`).
- Counters are in `crates/engine/src/diag.rs`. Add a counter before adding a guess.
- GPU-bound versus CPU-bound: high `gpu wait` (or `acquire` on FIFO) means the GPU is the limit, and CPU work is free until it isn't. `cpu update` + `record` + `submit` near the frame time means the CPU is the limit.
- Run with the shader pack (`--shader-pack chiacraft`). Benches reset packs; see memory "benches need the shader pack".

## 2. Tools, per machine

The Mac (M3 Max, MoltenVK). Testers run every game; ask for runs, never launch one yourself.
- **CPU:** `samply record ./target/release/weft …` opens in Firefox Profiler, per thread.
- **CPU and GPU timeline:** `xcrun xctrace record --template 'Metal System Trace' --time-limit 30s --output x.trace --launch -- <weft cmd>` shows encoder boundaries, GPU gaps and drawable waits.
- **Live HUD:** `MTL_HUD_ENABLED=1 MTL_HUD_LOGGING_ENABLED=1` shows FPS, GPU ms and the encoder count.
- **MoltenVK's own costs:** `MVK_CONFIG_PERFORMANCE_TRACKING=1 MVK_CONFIG_ACTIVITY_PERFORMANCE_LOGGING_STYLE=2`, logged at exit.
- **One-frame Xcode capture:** `METAL_CAPTURE_ENABLED=1 MVK_CONFIG_AUTO_GPU_CAPTURE_SCOPE=2 MVK_CONFIG_AUTO_GPU_CAPTURE_OUTPUT_FILE=<new>.gputrace`. Scope 2 is the first frame only.

The PC (RTX 5090, Linux). Runs go through Tester 2's queue, `t2q.py build|look|bench`. Ask Planner before installing anything.
- **CPU:** samply, or perf if installed (`perf record --call-graph dwarf`, and `perf stat -e cache-misses,branch-misses` for memory- or compute-bound).
- **Allocations:** heaptrack.
- **GPU:** Nsight Graphics GPU Trace (per-pass limiter) and RenderDoc.
- **Hiding a cost:** the 5090 hides per-draw and fill costs. Re-check GPU findings on a weak GPU, or at least reason about one.

Benches:
- the testkit's `bench` lines (frame and GPU ms per leg);
- the same tree and script, run back to back, with only the change under test;
- noise is ±5-10 %, so a gain under that needs repeats.

## 3. CPU checklist (Rust)

- **Build profile.** `[profile.release]` has only `debug = "line-tables-only"`. Bench `lto = "fat"` (or "thin"), `codegen-units = 1`, then PGO (cargo-pgo, trained on a scripted flight). BOLT is Linux only. `target-cpu=native` only for local builds; the x86-64 baseline is SSE2 (no AVX2) unless dispatched at runtime.
- **Allocation in hot loops.** Look for a `Vec::new()`, `collect()`, `clone()` or `format!` per cube, per tile, per face or per frame. Reuse buffers (`clear()` keeps capacity), use `SmallVec`, or a bump arena per meshing job. dhat-rs can assert "meshing allocates 0 times".
- **Hashing.** Check for `HashMap` with SipHash on coordinates. `FxHashMap` is the norm here. A hash lookup per neighbour in a dense grid should be a flat array indexed by `coord & mask`.
- **Bounds checks and vectorisation.** Iterate over slices, `chunks_exact`, reslice once before a loop. Float reductions don't vectorise unless you split the accumulators. Confirm with cargo-show-asm.
- **Threads.**
  - The worker pool must leave the main and render threads room.
  - No blocking (recv, lock, IO) inside a pool job.
  - Batch work per tile or region, not per voxel.
  - Compare CPU time with wall time per thread count (idle spin).
  - Skewed jobs: one heavy tile idles the others.
- **Locks and Arc.** Look for a `Mutex` taken per block or per cube, an `Arc::clone` per item on shared objects (cache-line ping-pong), and false sharing of per-thread counters (`CachePadded`).
- **Mod callbacks.** CPU callbacks (FFI or wasm) per column or per face, such as the far tiles' "face calls", cost per call. Batch them per tile, cache them per column, or evaluate coarsely and interpolate.
- **Noise and worldgen.** Evaluate at coarse cells and interpolate, as Minecraft does. Cache shared fields per column. SIMD noise (FastNoise2 is 4-8x faster than scalar).
- **Data layout.** Small structs, `u32` indices instead of pointers, SoA for hot fields, box big rare enum variants.

## 4. GPU checklist (Vulkan, all GPUs)

- **Pass count at full resolution.** At 3456x2234 a full-screen pass is about 7.7 Mpx read and written. Count the passes, fuse them, use half-resolution effects and a render-scale knob. Use `B10G11R11` instead of `RGBA16F` where alpha isn't needed.
- **Overdraw and order.** Draw opaque, then alpha-tested (`discard`: leaves, plants), then blended. Leaves with `discard` early in the pass break Apple's hidden-surface removal and cost elsewhere too.
- **Vertices.** Use a packed vertex format (Sodium's 20 bytes), vertex pulling, greedy or binary meshing, and LOD tiles at the right density for their distance.
- **Draw count.** It matters on the CPU everywhere and a lot on MoltenVK. Look for one draw per cube or per face group, and for draws of empty records.
- **Barriers.** `ALL_COMMANDS` to `ALL_COMMANDS` (`memory_barrier` in `backends/vulkan`) serialises everything. Use precise stages, for example compute-write to `DRAW_INDIRECT`/`INDIRECT_COMMAND_READ`, and batch them.
- **Descriptors.** No per-frame set allocations; bindless with `nonuniformEXT` on non-uniform indices.
- **Uploads.** Per-frame data belongs in host-visible memory (ReBAR on NVIDIA, unified on Apple). Watch per-frame `write_buffer` traffic in `gpu upload`.
- **Pipeline cache.** Kept on disk; on the Mac it also caches the SPIR-V-to-MSL step.

## 5. The Mac (MoltenVK, Apple TBDR)

- **No `drawIndirectCount`.** The terrain's count draws then issue a capped number of records (`capped_draws`, `world_render.rs`). Each record is one Metal draw, a CPU loop (MoltenVK issue #1796). Check the `terrain draws` line.
- **Avoid `vkCmdDraw*IndirectCount`** even where present on MoltenVK: it splits the render pass, which stores and reloads tiles.
- **Load and store actions.** Clear or don't-care loads; don't-care stores for depth and transient targets; lazily allocated memory becomes Metal memoryless. Every extra render pass or pass split costs a tile store and reload.
- **No geometry shaders.** Tessellation is emulated with compute.
- **No min/max reduction samplers on the M3 (Apple9).** Build Hi-Z with a compute reduction.
- **Precision.** `RelaxedPrecision`/fp16 reaches MSL as `half`; use it in fragment-heavy post passes.
- **Barriers.** These are Metal fences; avoid `vkCmdWaitEvents`, which ends the Metal encoder.

## 6. Writing it up

For each finding:
- **What:** file:line.
- **Evidence:** the measured number and where it came from.
- **Change:** the proposed change.
- **Gain:** the expected gain, on which platform.
- **Effort.**
- **Owner:** the session that owns the area (Lighting: effects and HDR; Distant terrain: LOD drawing; Terrain: land-feature speed; Upstream: the engine core).

Fix only small, proven things yourself, each on its own branch with a bench. Anything bigger goes to its owner. A real engine bug can become a Weft PR candidate, which Ronald writes.

## Sources

The full list, about 110 links, is in `sources.md` next to this file. The best starting points:
- **CPU:**
  - The Rust Performance Book: https://nnethercote.github.io/perf-book/
  - samply: https://github.com/mstange/samply
  - Tracy, through the tracy-client crate: https://docs.rs/tracy-client
  - dhat-rs: https://docs.rs/dhat
  - rayon tuning (Endignoux): https://gendignoux.com/blog/2024/11/18/rust-rayon-optimized.html
  - cargo-pgo: https://github.com/Kobzol/cargo-pgo
- **GPU:**
  - zeux, "Writing an efficient Vulkan renderer": https://zeux.io/2020/02/27/writing-an-efficient-vulkan-renderer/
  - niagara (GPU-driven culling): https://github.com/zeux/niagara
  - Aaltonen and Haar, GPU-Driven Rendering (SIGGRAPH 2015)
  - Khronos performance samples: https://docs.vulkan.org/samples/latest/
  - NVIDIA, Vulkan Dos and Don'ts: https://developer.nvidia.com/blog/vulkan-dos-donts/
- **Mac:**
  - Apple's TBDR guide: https://developer.apple.com/documentation/metal/tailor-your-apps-for-apple-gpus-and-tile-based-deferred-rendering
  - MoltenVK's user guide and configuration parameters: https://github.com/KhronosGroup/MoltenVK/tree/main/Docs
  - MoltenVK issue #1796: https://github.com/KhronosGroup/MoltenVK/issues/1796
- **Voxels:**
  - Binary greedy meshing: https://github.com/cgerikj/binary-greedy-meshing
  - Sodium's vertex format
  - Distant Horizons (its source)
  - FastNoise2: https://github.com/Auburn/FastNoise2
