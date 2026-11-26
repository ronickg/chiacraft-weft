# Performance review sources: Rust voxel engine (ash/Vulkan, MoltenVK on M3 Max, RTX 5090)
Collected 2026-11-05. Every URL was opened (HTTP 200) on that date. "Engine note" lines are
what the source means for this engine specifically. MoltenVK facts marked (src) were read from
MoltenVK `main` source and `Docs/Whats_New.md` on that date, not from docs.

## 0. Findings worth checking first (cross-cutting)
- **MoltenVK `vkCmdDraw*IndirectCount` splits the render pass.** In `MVKCmdDraw.mm`,
  `encodeIndirectCountConversion()` calls `encodeStoreActions(true)`, opens a compute encoder to
  rewrite the indirect buffer, then `beginMetalRenderPass(kMVKCommandUseRestartSubpass)` (src).
  On a TBDR GPU that is a tile store + reload of every attachment per count-draw call. It then
  loops on the CPU over `maxDrawCount`, one Metal indirect draw each (issue #1796, open).
  Engine note: on macOS, record the cull compute before the render pass and issue plain
  `vkCmdDrawIndexedIndirect` with a small fixed draw count (zero `instanceCount` for culled
  entries), or fewer, bigger draws (vertex pulling, one draw per face direction / region).
  https://github.com/KhronosGroup/MoltenVK/blob/main/MoltenVK/MoltenVK/Commands/MVKCmdDraw.mm
  https://github.com/KhronosGroup/MoltenVK/issues/1796
- **Plain multi-draw indirect on MoltenVK is also a CPU loop** over `drawCount` (src, #1796).
  Draw count, not triangle count, is the Mac CPU cost. If the pipeline reads `gl_DrawID`
  (macOS support added in 1.4.2), MoltenVK fills a draw-ID buffer on the CPU per call (src).
- **No min/max reduction samplers on an M3 Max.** MoltenVK 1.4.2 adds
  `VK_EXT_sampler_filter_minmax` only on Apple10 GPUs with macOS 26 (src); M3 is Apple9. Build
  the Hi-Z pyramid with an explicit compute min/max reduction, not a minmax sampler.
- **Geometry shaders unsupported** on MoltenVK (`maxGeometryShaderInvocations = 0`, src);
  tessellation is emulated with compute passes (src). Avoid both on the Mac path.
- **MoltenVK barriers = Metal fences** since 1.3.0; `vkCmdWaitEvents` ends the current Metal
  encoder (src, Whats_New 1.3.0). Prefer pipeline barriers over events on the Mac path, and
  avoid barriers inside render passes (MoltenVK skips them on Apple GPUs, Whats_New 1.1.2).
- **MoltenVK maps `VK_MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT` to `MTLStorageModeMemoryless`** on
  Apple GPUs (src, `MVKDevice.mm`). Engine note: depth and MSAA/transient attachments that are
  not read later should use a lazily allocated memory type plus `STORE_OP_DONT_CARE`.

## 1. Rust CPU performance

### The Rust Performance Book (Nethercote)
- **Build configuration**: https://nnethercote.github.io/perf-book/build-configuration.html
  Max speed: `codegen-units = 1`, `lto = "fat"`, `panic = "abort"`, `-C target-cpu=native`
  (only for builds that never leave the machine), PGO via cargo-pgo, an alternative allocator
  (mimalloc / tikv-jemallocator). Benchmark each change on its own.
- **Profiling**: https://nnethercote.github.io/perf-book/profiling.html
  Lists perf, Instruments, samply, Cachegrind, DHAT, heaptrack, bytehound, counts. Needs
  `debug = "line-tables-only"` (or `debug = true`) in release for usable stacks; consider
  `-C force-frame-pointers=yes` for cheap call stacks.
- **Bounds checks**: https://nnethercote.github.io/perf-book/bounds-checks.html
  Prefer iterators over indexing; reslice once (`let s = &v[..n];`) or `assert!(i < len)` before
  a hot loop so LLVM drops per-access checks; `get_unchecked` only as a last resort, measured.
- **Heap allocations**: https://nnethercote.github.io/perf-book/heap-allocations.html
  Reuse `Vec`s (`clear()` keeps capacity), `with_capacity`, `SmallVec`/`ArrayVec` for short
  lists, avoid `clone`/`to_string` in loops, `Cow`. Use DHAT to find hot allocation sites.
- **Hashing**: https://nnethercote.github.io/perf-book/hashing.html
  SipHash default is slow for small keys; switch to FxHash / ahash / foldhash where DoS
  resistance is irrelevant (chunk coordinates, block ids). nohash for already-random keys.
- **Type sizes**: https://nnethercote.github.io/perf-book/type-sizes.html
  `-Zprint-type-sizes`, static size asserts, box large rare enum variants, smaller index types
  (u32 instead of usize) for big per-voxel/per-vertex arrays.
- **Inlining**: https://nnethercote.github.io/perf-book/inlining.html
  `#[inline]` for small cross-crate hot functions (matters with many crates/mods);
  `#[inline(never)]` / `#[cold]` to split cold paths out of hot loops.
- **Machine code**: https://nnethercote.github.io/perf-book/machine-code.html
  Read the asm of hot loops (cargo-show-asm, Compiler Explorer) to confirm vectorisation and no
  bounds checks.

### Nethercote's rustc speed-up posts
- **How to speed up the Rust compiler in December 2025**:
  https://nnethercote.github.io/2025/12/05/how-to-speed-up-the-rust-compiler-in-december-2025.html
  Method: track instruction counts, cycles, wall-time and peak memory separately (they can
  disagree); fast paths for the common case; remove waste where code is generated rather than
  hoping the optimizer cleans it; cut allocations; compact data layouts.
- **counts** (ad hoc profiling: `eprintln!` events then tally): https://github.com/nnethercote/counts
  Engine note: quick way to count chunk remeshes, LOD rebuilds, allocations per frame.

### Profilers
- **samply** (macOS + Linux sampling profiler, opens in Firefox Profiler):
  https://github.com/mstange/samply  /  https://profiler.firefox.com/docs/
  `samply record ./target/profiling/weft ...`; per-thread view shows rayon workers vs main thread;
  works on the M3 without dtrace/SIP changes. Best first tool on the Mac.
- **cargo-flamegraph**: https://github.com/flamegraph-rs/flamegraph
  Uses perf on Linux, dtrace on macOS (needs root). Good for a single shareable SVG.
- **perf (Brendan Gregg's guide)**: https://www.brendangregg.com/perf.html
  On the 5090 PC: `perf record --call-graph dwarf` (or frame pointers), `perf stat -e
  cache-misses,branch-misses` to tell compute-bound from memory-bound.
- **Tracy**: https://github.com/wolfpld/tracy (manual PDF: https://github.com/wolfpld/tracy/releases)
  Rust: https://docs.rs/tracy-client/latest/tracy_client/ , https://docs.rs/tracing-tracy/latest/tracing_tracy/
  `frame_mark()` after present,
  `span!` per system, `secondary_frame_mark` for the tick loop, plot values (chunks queued,
  MB uploaded), GPU zones via Vulkan timestamp queries (`GpuContext`). Crate version must match
  the Tracy GUI protocol version (table in the crate README). Best tool for frame spikes.
- **Instruments (macOS)**: Time Profiler for CPU; Metal System Trace for CPU-GPU timeline:
  https://developer.apple.com/documentation/xcode/analyzing-the-performance-of-your-metal-app
  https://developer.apple.com/documentation/xcode/improving-your-app-s-performance
  `xcrun xctrace record --template 'Metal System Trace' --launch -- ./weft ...` shows encoder
  boundaries, GPU idle gaps, drawable waits; MoltenVK render-pass restarts show up here.

### Allocation profiling
- **dhat-rs**: https://github.com/nnethercote/dhat-rs  /  https://docs.rs/dhat/latest/dhat/
  `#[global_allocator] dhat::Alloc` behind a feature; reports bytes/blocks per call site and
  short-lived allocations; testing mode can assert allocation counts (e.g. "meshing a chunk
  allocates 0 times"). Works on macOS.
- **heaptrack** (Linux): https://github.com/KDE/heaptrack  low-overhead, flamegraph of
  allocations, temporary-allocation counts. Use on the PC.
- **bytehound** (Linux only, little recent activity): https://github.com/koute/bytehound
  timeline of memory use and leaks; good for streaming/LOD memory growth.

### Bounds checks, SIMD, vectorisation
- **cargo-show-asm**: https://github.com/pacak/cargo-show-asm  verify hot loops.
- **std::simd** (still nightly-only): https://doc.rust-lang.org/std/simd/index.html
- **wide** (stable, portable f32x4/f32x8 etc.): https://github.com/Lokathor/wide
- **fearless_simd** (Linebender, runtime dispatch, NEON+AVX2): https://github.com/linebender/fearless_simd
  background: https://raphlinus.github.io/rust/simd/2018/10/19/fearless-simd.html
  Engine note: NEON is baseline on aarch64 (M3), but x86-64 baseline is SSE2 only, so the PC
  build gets no AVX2 unless `target-feature`/`target-cpu` or runtime dispatch. Auto-vectorisation
  needs straight loops over slices (`chunks_exact`), no early exits, and fails on float
  reductions (no reassociation) unless you split accumulators by hand.

### rayon pitfalls
- **rayon FAQ**: https://github.com/rayon-rs/rayon/blob/main/FAQ.md  (Rc vs Arc, atomics
  instead of Cell, `RAYON_NUM_THREADS`, `build_global`).
- **ThreadPoolBuilder**: https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html
  Give worldgen/meshing their own pool with fewer threads than cores and a name/priority, so the
  main and render threads are never starved; don't rely on the implicit global pool.
- **with_min_len / with_max_len**:
  https://docs.rs/rayon/latest/rayon/iter/trait.IndexedParallelIterator.html#method.with_min_len
  Too-fine tasks (per voxel, per column) drown in deque traffic; batch per chunk or per region.
- **yield_now**: https://docs.rs/rayon/latest/rayon/fn.yield_now.html  Never block (channel recv,
  mutex, file IO) inside a rayon job: it stalls a worker; nested `par_iter` there can deadlock.
- **`fold` creates an identity per split** (allocation surprise): https://github.com/rayon-rs/rayon/issues/1124
- **Endignoux, "making a parallel Rust workload 10x faster with (or without) Rayon" (2024)**:
  https://gendignoux.com/blog/2024/11/18/rust-rayon-optimized.html
  Idle workers spin (`sched_yield` storms) and burn CPU; jobs can't be split once started, so
  skewed work (one heavy chunk) idles others; pinning threads gave 10-20 percent. Check CPU time
  vs wall time per thread count.

### Locks, Arc and atomics
- **Mara Bos, "Rust Atomics and Locks"** (free online): https://marabos.nl/atomics/
  Memory orderings, how Mutex/Condvar are built, cache-line effects.
- **parking_lot**: https://github.com/Amanieu/parking_lot  (smaller locks, fair unlock options;
  std Mutex is now futex-based, so measure before switching.)
- **dashmap** (sharded concurrent map): https://github.com/xacrimon/dashmap
  Or shard a chunk map yourself by `hash(coord) % N` with N locks.
- **Kołaczkowski, "How a single line of code made a 24-core server slower than a laptop"**:
  https://pkolaczk.github.io/server-slower-than-a-laptop/  `Arc::clone` refcount traffic on
  shared objects bounced a cache line between cores; fix was per-thread copies. Engine note: don't
  clone a shared `Arc<World>`/`Arc<Registry>` per block or per task; borrow or clone once per job.
- **CachePadded** (avoid false sharing of per-thread counters/queues):
  https://docs.rs/crossbeam-utils/latest/crossbeam_utils/struct.CachePadded.html

### Data layout
- **Andrew Kelley, "A Practical Guide to Applying Data-Oriented Design"**: https://www.youtube.com/watch?v=IroPQ150F6c
  Shrink structs, indices instead of pointers, SoA for hot fields, encode booleans out of structs.
- **soa-rs** (struct-of-arrays derive): https://docs.rs/soa-rs/latest/soa_rs/
- **bumpalo** (per-frame / per-task arena): https://github.com/fitzgen/bumpalo
  Engine note: a bump arena per meshing job, reset after upload, removes most mesher allocations.
- Engine note: dense chunk grids around the player are better as a flat ring-buffer array indexed
  by `coord & mask` than a HashMap lookup per neighbour.

### HashMap choice
- **rustc-hash (FxHash, v2 algorithm)**: https://github.com/rust-lang/rustc-hash
- **foldhash** (hashbrown's default hasher; fast, minimal DoS resistance): https://github.com/orlp/foldhash
- **aHash**: https://github.com/tkaitchuck/aHash
  Any of these for chunk/region coordinate keys; keep SipHash only for untrusted network input.

### Release profile, PGO, BOLT
- **Cargo profiles reference**: https://doc.rust-lang.org/cargo/reference/profiles.html
  Add `[profile.profiling] inherits = "release"`, `debug = "line-tables-only"`; per-package
  `opt-level` (e.g. `opt-level = 3` for noise/mesher crates even in dev builds).
- **cargo-pgo**: https://github.com/Kobzol/cargo-pgo  and Kobzol's walkthrough
  https://kobzol.github.io/rust/cargo/2023/07/28/rust-cargo-pgo.html
  Train with a scripted flight over generated terrain (testkit script) so worldgen and meshing
  are covered. PGO works on macOS; BOLT (https://github.com/llvm/llvm-project/tree/main/bolt) is
  Linux/ELF only. rustc chapter: https://doc.rust-lang.org/rustc/profile-guided-optimization.html

## 2. GPU and Vulkan render speed

### GPU-driven rendering, culling, Hi-Z
- **Aaltonen and Haar, "GPU-Driven Rendering Pipelines" (SIGGRAPH 2015)**:
  https://advances.realtimerendering.com/s2015/aaltonenhaar_siggraph2015_combined_final_footer_220dpi.pdf
  Cluster culling on GPU, previous-frame depth reprojected for occlusion, then re-test newly
  visible; one indirect draw for everything.
- **Karis et al., "Nanite" (SIGGRAPH 2021)**:
  https://www.advances.realtimerendering.com/s2021/Karis_Nanite_SIGGRAPH_Advances_2021_final.pdf
  Two-pass HZB occlusion: draw last frame's visible set, build HZB, test the rest, draw
  disoccluded. Avoids the one-frame popping of pure previous-frame HZB.
- **zeux/niagara** (Vulkan GPU-driven renderer, streamed): https://github.com/zeux/niagara
  Compute culling, depth pyramid build, two-phase occlusion, draw command compaction.
- **zeux, "Writing an efficient Vulkan renderer" (2020)**:
  https://zeux.io/2020/02/27/writing-an-efficient-vulkan-renderer/
  Memory suballocation, descriptor models (slot vs bindless), barrier batching, pipeline cache,
  command buffer recording across threads. The single best overview.
- **vkguide GPU-driven chapter**: https://vkguide.dev/docs/gpudriven/gpu_driven_engines/ and
  **compute culling**: https://vkguide.dev/docs/gpudriven/compute_culling/
  Frustum + depth-pyramid culling in compute, writing indirect commands; depth pyramid via
  min-reduction (needs a manual reduction on the M3, see section 0).

### Draw calls, batching, command buffers
- **NVIDIA, "Vulkan Dos and Don'ts"**: https://developer.nvidia.com/blog/vulkan-dos-donts/
  Sort by pipeline; few `vkQueueSubmit`s; no tiny command buffers; reuse pools; create pipelines
  off-thread with a pipeline cache; suballocate memory; stay under `VK_EXT_memory_budget`.
- **NVIDIA "Advanced API Performance" series**:
  command buffers https://developer.nvidia.com/blog/advanced-api-performance-command-buffers/
  memory https://developer.nvidia.com/blog/advanced-api-performance-memory-and-resources/
- **Khronos sample, pipeline cache**:
  https://docs.vulkan.org/samples/latest/samples/performance/hpp_pipeline_cache/README.html
  On MoltenVK the pipeline cache also stores SPIR-V-to-MSL conversion (the slowest load step).

### Barriers and synchronization2
- **Khronos sample, using pipeline barriers efficiently**:
  https://docs.vulkan.org/samples/latest/samples/performance/pipeline_barriers/README.html
  `ALL_COMMANDS`/`ALL_GRAPHICS` masks flush the pipe; precise stages let passes overlap (13
  percent frame-time gain in the sample).
- **Khronos sample, synchronization2**:
  https://docs.vulkan.org/samples/latest/samples/extensions/synchronization_2/README.html
  and guide https://docs.vulkan.org/guide/latest/extensions/VK_KHR_synchronization2.html
  Pitfalls: `NONE` vs `TOP_OF_PIPE`; stage and access masks now per barrier, so a lazy
  `ALL_COMMANDS` on one barrier still serialises everything.
- **Synchronization examples wiki** (copy-paste correct masks, e.g. compute-write to indirect-read
  is `COMPUTE_SHADER/SHADER_WRITE -> DRAW_INDIRECT/INDIRECT_COMMAND_READ`):
  https://github.com/KhronosGroup/Vulkan-Docs/wiki/Synchronization-Examples
- **NVIDIA, Advanced API Performance: Synchronization**:
  https://developer.nvidia.com/blog/advanced-api-performance-synchronization/
  Batch barriers into one call; avoid read-to-read barriers; `UNDEFINED` old layout when contents
  are discarded.
- Also run the Synchronization Validation layer (Vulkan Configurator preset) on the PC.

### Descriptor indexing / bindless
- **Khronos sample, descriptor indexing**:
  https://docs.vulkan.org/samples/latest/samples/extensions/descriptor_indexing/README.html
  `nonuniformEXT` on non-uniform indices (missing it is a bug, not just slow); update-after-bind.
- **NVIDIA, Advanced API Performance: Descriptors**:
  https://developer.nvidia.com/blog/advanced-api-performance-descriptors/
- **Khronos sample, descriptor management**:
  https://docs.vulkan.org/samples/latest/samples/performance/descriptor_management/README.html
  Don't allocate/update sets per frame; cache or go bindless.
- **Apple, Improving CPU performance by using argument buffers**:
  https://developer.apple.com/documentation/metal/improving-cpu-performance-by-using-argument-buffers
  MoltenVK backs descriptor sets with argument buffers by default
  (`MVK_CONFIG_USE_METAL_ARGUMENT_BUFFERS=1`). Note `MVK_CONFIG_PREFILL_METAL_COMMAND_BUFFERS`
  non-zero is ignored when update-after-bind is enabled.

### Push constants, uniforms, uploads
- **Vulkan Guide, push constants**: https://docs.vulkan.org/guide/latest/push_constants.html
  128 bytes guaranteed; use for per-draw indices (region id, LOD level), not bulk data.
- **VMA recommended usage patterns** (staging vs ReBAR vs readback):
  https://gpuopen-librariesandsdks.github.io/VulkanMemoryAllocator/html/usage_patterns.html
  On the 5090 with ReBAR, write per-frame data straight into DEVICE_LOCAL|HOST_VISIBLE memory
  (sequential writes only, never read back). On Apple unified memory, buffers need no staging
  copy; textures still go to private/optimal storage.
- **gpu-allocator** (pure-Rust allocator for ash): https://github.com/Traverse-Research/gpu-allocator
- **Khronos sample, memory budget**:
  https://docs.vulkan.org/samples/latest/samples/extensions/memory_budget/README.html
  Engine note: size LOD and chunk buffer pools from the budget, not a constant.

### Overdraw, fill rate, HDR, post, render scale
- **NVIDIA, "Peak-Performance-Percentage Analysis Method"**:
  https://developer.nvidia.com/blog/the-peak-performance-analysis-method-for-optimizing-any-gpu-workload/
  Find the top SOL unit per pass (SM, L2, DRAM, raster) before optimising; applies directly to
  Nsight GPU Trace on the 5090.
- **Khronos sample, HDR**: https://docs.vulkan.org/samples/latest/samples/api/hpp_hdr/README.html
  Engine note: `B10G11R11_UFLOAT` halves bandwidth vs `R16G16B16A16_SFLOAT` for the scene target
  if alpha isn't needed. Each full-screen post pass at Retina 3456x2234 costs ~7.7 Mpx of reads
  and writes: count passes, fuse them, and expose a render-scale knob before tuning shaders.

### Apple TBDR (M3 Max)
- **Tailor your apps for Apple GPUs and tile-based deferred rendering**:
  https://developer.apple.com/documentation/metal/tailor-your-apps-for-apple-gpus-and-tile-based-deferred-rendering
- **Setting load and store actions**: https://developer.apple.com/documentation/metal/setting-load-and-store-actions
- **Memoryless storage mode**: https://developer.apple.com/documentation/metal/mtlstoragemode/memoryless
  Checks: `LOAD_OP_CLEAR`/`DONT_CARE` unless contents are needed; `STORE_OP_DONT_CARE` for depth
  and transient targets; memoryless (lazily allocated in Vulkan) for those; no unnecessary pass
  splits (each split stores and reloads tiles).
- **Khronos sample, render passes (load/store ops)**:
  https://docs.vulkan.org/samples/latest/samples/performance/render_passes/README.html
  and **subpasses** https://docs.vulkan.org/samples/latest/samples/performance/subpasses/README.html
  (written for Mali TBDR; same rules on Apple).
- **WWDC20 "Optimize Metal Performance for Apple silicon Macs"**:
  https://developer.apple.com/videos/play/wwdc2020/10632/
  HSR order: opaque, then alpha-tested (`discard`), then blended; avoid redundant passes; load/
  store actions. Engine note: leaves/foliage with `discard` should draw after opaque terrain.
- **Tech talk "Learn performance best practices for Metal shaders"** (half precision, address
  spaces, occupancy): https://developer.apple.com/videos/play/tech-talks/111373/
  Engine note: SPIR-V `RelaxedPrecision`/fp16 reaches MSL as `half`; worth it in fragment-heavy
  post and terrain shading.

### MoltenVK specifics
- **Runtime user guide** (Performance considerations, Metal frame capture):
  https://github.com/KhronosGroup/MoltenVK/blob/main/Docs/MoltenVK_Runtime_UserGuide.md
  Use a pipeline cache (SPIR-V-to-MSL conversion is the slowest step); 3 swapchain images; turn
  off Metal API Validation for perf runs.
- **Configuration parameters**:
  https://github.com/KhronosGroup/MoltenVK/blob/main/Docs/MoltenVK_Configuration_Parameters.md
  `MVK_CONFIG_USE_METAL_ARGUMENT_BUFFERS` (default 1), `MVK_CONFIG_PREFILL_METAL_COMMAND_BUFFERS`
  (0 = encode at submit: fastest, most memory), `MVK_CONFIG_SYNCHRONOUS_QUEUE_SUBMITS`,
  `MVK_CONFIG_USE_MTLHEAP`, `MVK_CONFIG_FAST_MATH_ENABLED`, `MVK_CONFIG_PERFORMANCE_TRACKING=1` +
  `MVK_CONFIG_ACTIVITY_PERFORMANCE_LOGGING_STYLE` for MoltenVK's own timing stats (shader
  conversion, encoding), `MVK_CONFIG_SHOULD_MAXIMIZE_CONCURRENT_COMPILATION=1` for startup.
- **What's new (changelog)**: https://github.com/KhronosGroup/MoltenVK/blob/main/Docs/Whats_New.md
  1.3.0: Vulkan 1.3, barriers via Metal fences, residency sets. 1.4.2: macOS `gl_DrawID`,
  minmax samplers on Apple10 only. 1.4.3 (unreleased): `VK_EXT_multi_draw`.
- **Issue #1796, drawCount loop**: https://github.com/KhronosGroup/MoltenVK/issues/1796

### Apple GPU profiling
- **Capturing a Metal workload in Xcode**:
  https://developer.apple.com/documentation/xcode/capturing-a-metal-workload-in-xcode
  For MoltenVK without launching from Xcode: `METAL_CAPTURE_ENABLED=1
  MVK_CONFIG_AUTO_GPU_CAPTURE_SCOPE=2 MVK_CONFIG_AUTO_GPU_CAPTURE_OUTPUT_FILE=/path/x.gputrace`
  (file must not exist), then open in Xcode: per-encoder GPU time, limiter counters, shader cost
  per line, and visible render-pass restarts.
- **Optimizing GPU performance** (Xcode GPU counters, limiters):
  https://developer.apple.com/documentation/xcode/optimizing-gpu-performance
- **WWDC20 "Optimize Metal apps and games with GPU counters"**:
  https://developer.apple.com/videos/play/wwdc2020/10603/
  Read limiters (ALU, texture sample, buffer read, bandwidth, occupancy) per encoder.
- **Metal Performance HUD**: https://developer.apple.com/documentation/xcode/gaining-performance-insights-with-metal-performance-hud
  and tech talk https://developer.apple.com/videos/play/tech-talks/110339/
  `MTL_HUD_ENABLED=1` (live FPS, GPU time, memory, encoder count), `MTL_HUD_LOGGING_ENABLED=1`,
  `MTL_HUD_INSIGHTS_ENABLED=1`. Cheapest Mac check; works for a MoltenVK app launched from a shell.

### Nsight and RenderDoc (PC)
- **Nsight Graphics**: https://developer.nvidia.com/nsight-graphics  /  user guide
  https://docs.nvidia.com/nsight-graphics/UserGuide/index.html
  GPU Trace for per-pass SOL/throughput and occupancy; frame debugger for barriers and indirect
  arguments.
- **RenderDoc**: https://renderdoc.org/docs/index.html
  Inspect indirect buffers, overdraw/quad overdraw overlays, per-event timings (relative only).
  Doesn't capture through MoltenVK; use Xcode capture there.

## 3. Voxel engines and LOD

### Meshing
- **cgerikj/binary-greedy-meshing**: https://github.com/cgerikj/binary-greedy-meshing
  64x64 `u64` occupancy columns (62^3 chunk + 1 padding each side); face culling and greedy
  merge with bit ops; 50-200 us per chunk. Masks double for physics/raycast. v1 branch has AO.
- **Rust port, `binary-greedy-meshing` crate**: https://docs.rs/binary-greedy-meshing/latest/binary_greedy_meshing/
  `fast_mesh` with precomputed opacity/transparency masks (~65 us opaque chunk reported).
- **Davis Morley, "Greedy Meshing Voxels Fast" (Handmade Seattle 2022)**:
  https://www.youtube.com/watch?v=4xs66m1Of4A  origin of the binary method.
- **0fps (Mikola Lysenko), "Meshing in a Minecraft game"**:
  https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/  culled vs greedy vs monotone,
  quad-count vs mesh-time trade.
- **0fps, "Ambient occlusion for Minecraft-like worlds"**:
  https://0fps.net/2013/07/03/ambient-occlusion-for-minecraft-like-worlds/
  Per-vertex AO; flip quad diagonal by AO; greedy merge only quads with equal AO/light.
- **Vercidium, "Further Voxel World Optimisations"** (0.48 ms/chunk; bit-packed masks, avoid
  per-voxel branches): https://vercidium.com/blog/further-voxel-world-optimisations/
  and https://vercidium.com/blog/voxel-world-optimisations/

### Packed vertices and vertex pulling
- **Vercidium, "I Optimised My Game Engine Up To 12000 FPS"**:
  https://www.youtube.com/watch?v=40JzyaOYJeY  faces packed into a few bytes in a storage
  buffer, the vertex shader expands quads from `gl_VertexIndex` (vertex pulling), no per-vertex
  attributes; draws grouped by face direction so back faces are skipped per draw.
- **Sodium compact chunk vertex** (src: `CompactChunkVertex.java`, `RenderRegion.java`):
  https://github.com/CaffeineMC/sodium
  20-byte stride: 20-bit positions over a 32-block range around the section, 15-bit UVs with a
  bias sign bit, packed light. Sections batched into 8x4x8 regions sharing one buffer and
  multi-draw. Engine note: region-sized arenas make the MoltenVK draw-count loop much shorter.
- **Nick McDonald, "High Performance Voxel Engine: Vertex Pooling"**:
  https://nickmcd.me/2021/04/04/high-performance-voxel-engine/
  One persistent buffer split into fixed buckets + free list, one indirect command per bucket,
  culled by masking/ordering the command buffer (6 face-direction groups: 42 -> 27 ms in his
  test). Under-filled buckets cost nothing measurable.
- **Tommo (Minecraft PE), "Advanced cave culling"**: https://tomcc.github.io/2014/08/31/visibility-1.html
  Per-chunk 15-bit face-to-face connectivity via flood fill, traversed at render time; culls
  50-99 percent of cave chunks. Cheap CPU-side culling to complement GPU occlusion.

### LOD for distant terrain
- **Distant Horizons (source)**: https://gitlab.com/distant-horizons-team/distant-horizons
  and core https://gitlab.com/distant-horizons-team/distant-horizons-core
  From the core tree: a render quadtree (`render/QuadTree`), per-section "full data" sources
  and column render views (vertical columns of data points per x/z; detail levels step 2x,
  from memory of the code), SQLite storage (`core/sql`), generation queues and rate limiting. Low detail first,
  refined later. Engine note: store LOD data per tile in a compact column format, mesh from it on
  workers, and rate-limit both generation and upload.
- **Lengyel, Transvoxel / "Voxel-Based Terrain for Real-Time Virtual Simulations"**:
  https://transvoxel.org/  /  https://transvoxel.org/Lengyel-VoxelTerrain.pdf
  Seam-free transitions between LOD levels for smooth (marching-cubes) terrain; for blocky
  terrain use skirts or overlap instead.
- **Losasso and Hoppe, "Geometry Clipmaps" (SIGGRAPH 2004)**: https://hhoppe.com/geomclipmap.pdf
  Nested rings around the camera, each 2x coarser, updated incrementally as the camera moves;
  fixed memory and predictable per-frame update.
- **de Boer, geomipmapping**: https://www.flipcode.com/archives/article_geomipmaps.pdf (patch LOD, crack fixing)

### Streaming budgets
- No single primary source; the applicable checks come from the sources above:
  rate-limited generation and upload queues (Distant Horizons `generation/queues`,
  `util/ratelimiting`), memory pools sized from `VK_EXT_memory_budget` (Khronos sample),
  fixed-size buckets/regions (McDonald, Sodium), and Tracy plots of queue length and MB uploaded
  per frame. Engine checks: a per-frame cap on remeshes and on upload bytes; nearest-first
  priority re-sorted when the player moves; cancel stale jobs for tiles that left range.

### Noise speed
- **FastNoise2**: https://github.com/Auburn/FastNoise2
  SIMD node graph (SSE2..AVX512, NEON) with runtime dispatch; intermediates stay in registers;
  4-8x FastNoise Lite in its benchmarks (e.g. 3D Perlin 48 -> 261 M points/s on AVX2). Generate
  whole grids, not point by point. Rust binding: https://github.com/Lemonzyy/fastnoise2-rs
- **Henrik Kniberg, "Reinventing Minecraft world generation"**:
  https://www.youtube.com/watch?v=ob3VwY4JyzE  Minecraft's 1.18 pipeline: few noise fields
  (continentalness, erosion, ...) through spline curves; 3D density sampled on a coarse grid and
  interpolated (Minecraft uses 4x8x4-block cells). Engine checks: evaluate costly 3D noise on such
  cells and trilinearly interpolate, cache 2D climate fields per column, reuse samples across LOD
  levels, and sample LOD tiles at their own resolution rather than meshing full-res data.