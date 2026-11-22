# Speed review: the CPU side of drawing a frame

Reviewer: render-CPU review, 2026-11-05, branch `perf-review` (worktree `.claude/worktrees/perf`). I only read code: no edits, no runs, no builds.

I checked the MoltenVK claims against MoltenVK **v1.4.2** source, the version brew has installed (`/opt/homebrew/Cellar/molten-vk/1.4.2`). A release that ships its own MoltenVK beside the game may be a different version: check which one before trusting a MoltenVK line below.

## What the numbers already say

- **The Mac:** the engine's own CPU work is small. In `app.rs:2345`, `record_passes` with all client rendering is `record` in the bench lines, 0.34-0.49 ms (`reports/lod/ours/2026-11-04_bench_lines.txt`). The Vulkan recording in `end_frame` is 0.3-0.7 ms. The large CPU number is `queue`, 4.5-7 ms, which is the time spent inside `vkQueueSubmit2`. So the CPU wins on the Mac are about what MoltenVK has to encode and wait for, not about our Rust.
- **The PC:** the bench lines show `record` 0.10-0.12, `submit` 0.05 and `acquire` (GPU wait) 0.75 ms. The PC is GPU-bound, and no finding below gains it more than about 0.1 ms CPU.
- **Weak 4-core CPUs:** they scale our own Rust costs by about 3-5x, so findings 7-9 matter more there.
- **Draw and pipeline counts:** the PC bench lines show 2300-3900 draws in only 16-22 draw commands and 15-20 pipeline binds a frame. Per-draw state churn is therefore not a big cost today. The draws are multi-draw indirect, and on MoltenVK each record becomes one Metal draw (`MVKCmdDraw.mm:1146`, a loop over `_drawCount` that calls `drawIndexedPrimitives` once per record). The empty records are being capped on `fix/indirect-cap`, so I don't cover them here.

### Two things that change how the Mac logs should be read

1. **`queue` is not only encoding.** MoltenVK fetches the `CAMetalDrawable` lazily, while it encodes the first pass that touches the swapchain image (`MVKImage.mm:1608-1618`, `nextDrawable`). That happens inside `vkQueueSubmit2`. So the 4.5-7 ms may include waiting for the display. See finding 2 for how to split the two.
2. **The Mac's GPU split between `world` and `translucent` is not real.** Two `Cmd::Mark`s sit inside the world render pass (`client/mod.rs:3014` and `3110`). On Apple GPUs MoltenVK samples timestamps only at the end of a Metal encoder (`MVKCommandBuffer.mm:1238-1298`). Both marks therefore read at the end of the world encoder: `world 7.6` includes the translucent, entity, sky, particle and weather draws. Each mark also adds a one-sample blit encoder that waits on a fence. To split them on the Mac, use a Metal System Trace, not these marks.

## Findings, ranked by gain per effort

| # | What | Gain (Mac / PC / weak) | Effort |
|---|---|---|---|
| 1 | `memory_barrier` has a HOST destination, so MoltenVK walks every resource, about 24 times a frame | Mac 0.2-2 ms CPU (uncertain) / ~0 / ~0 | 30 min |
| 2 | `queue` hides drawable waits; split the submit | measurement now; Mac GPU starts sooner | 1 h to measure, ~1 day to split |
| 3 | The occlusion depth chain: 31 MB copy, 13 barriers, 12 compute encoders a frame | Mac 0.1-0.3 CPU + 0.1-0.3 GPU / small / iGPU small-moderate | 0.5-1 day |
| 4 | Bug: `CopyDepthToBuffer` uses the wrong old layout and leaves the depth layout untracked | correctness (AMD/Intel risk) | 15 min |
| 5 | Exact barrier masks (the code-level detail behind Research 19's row) and the acquire wait stage | Mac GPU overlap (unmeasured) / PC acquire overlap | 2-4 h |
| 6 | Texture uploads: two full barriers per mip, each ending a Metal blit encoder | load and reload time; small per frame | 1-2 h |
| 7 | O(n²) structure lookups in `draw` and `draw_shadow` | grows with structures (felled trees) | 30 min |
| 8 | Shadow pass culls every opaque cube on the CPU every frame | 0.05-0.3 ms Mac/PC, about 3x on weak | 2-4 h |
| 9 | Per-frame allocations in `draw`, `draw_shadow` and `render` | 20-100 µs, about 3x on weak | 1-2 h |
| 10 | Translucent runs split by page, and `bind_run` re-sends all its state per run | scene-dependent: water views | 2-4 h |
| 11 | `end_frame` (MoltenVK encoding) on the main thread | Mac CPU-bound cases: up to 4-7 ms overlap | days |

---

### 1. `memory_barrier` asks for the host, so MoltenVK walks every buffer and image about 24 times a frame

- **What:** `backends/vulkan/src/lib.rs:1008-1018`. Its destination is `ALL_COMMANDS | HOST` with `HOST_READ`. It is used for every `Cmd::Barrier` (`lib.rs:1885`) and twice in `record_uploads` (`lib.rs:1038`, `1118`).
- **Evidence:** MoltenVK 1.4.2, `MVKDevice::applyMemoryBarrier` (`MVKDevice.mm:4650-4659`). For any global memory barrier whose destination is HOST stage with HOST_READ access, it takes `_rezLock` and calls `applyMemoryBarrier` on every `MVKResource` the device has (`_resources`, which holds every `MVKBuffer` and every image memory binding). On Apple Silicon each call does nothing (`needsHostReadSync` is false on unified memory, `MVKBuffer.mm:103-130`), but the walk still runs. It happens at encode time, which is inside `vkQueueSubmit2`, so it counts in `queue`.
  - **Barriers a frame** (from the code):
    - uploads: 2;
    - `CubeTable::cull`: 3 on the Mac (`world_render.rs:1935`, `1938`, `1952`);
    - `Occlusion::test`: 2 (`world_render.rs:2052`, `2060`);
    - `take_depth`: 1 plus one per level (`world_render.rs:2092`, `2101`), about 13 at 3456x2234;
    - terrain GPU: up to 2;
    - particles: 2.
    - That makes about 20-24 a frame.
  - **Resources:** each texture and animation frame is its own image (`render/src/lib.rs:31`). The mods hold about 1,600-2,400 texture PNGs, plus face and quad pages and the per-slot frame buffers, so roughly 3,000-5,000 resources.
  - **Cost:** 24 x about 4,000 pointer-chasing virtual calls is about 100k calls a frame. At 5-20 ns each that is about 0.5-2 ms. This is my estimate, not a measurement.
- **Change:** drop `HOST` and `HOST_READ` from `memory_barrier`. Add one host barrier (`ALL_COMMANDS`/`MEMORY_WRITE` to `HOST`/`HOST_READ`) in `end_frame`, after the command loop and before the closing timestamp (around `lib.rs:1960`). That is the only one the CPU readbacks need: the terrain `out`/`lod_out` buffers, `table buckets` and the mapped counts are all read after the slot's fence.
- **Gain:**
  - Mac: removes about 23 of about 24 resource walks. My guess is 0.2-2 ms of `queue`, and the range is wide.
  - PC and weak machines: none (NVIDIA, AMD and Mesa have no such walk). The barrier also becomes slightly narrower.
- **Effort:** 30 minutes.
- **Measure:**
  - `submit split` "queue" before and after, same script.
  - Better: `samply record` on the Mac and look at `MVKDevice::applyMemoryBarrier` under `vkQueueSubmit2`. MoltenVK's symbols are exported.
- **Owner:** Upstream (backend).

### 2. `queue` hides drawable waits; measure it, then split the submit

- **What:** `lib.rs:1979-1987`, a single `queue_submit2` per frame. The acquire semaphore waits at `ALL_COMMANDS` (`lib.rs:1967-1969`).
- **Evidence:**
  - MoltenVK calls `nextDrawable` while it encodes the first pass that uses the swapchain image, inside the submit (`MVKImage.mm:1608-1618`).
  - MoltenVK also commits the whole `MTLCommandBuffer` only at the end of encoding. So the GPU cannot start on any of this frame's work (terrain compute, shadows, world) until all 4.5-7 ms of encoding plus the drawable wait are done.
  - The game is GPU-bound at 16.2 ms with `FRAMES = 2` (`lib.rs:12`), so part of `queue` may be time waiting for the display.
- **Change:**
  - **(a) Measure first.** Run once with `MVK_CONFIG_PERFORMANCE_TRACKING=1 MVK_CONFIG_ACTIVITY_PERFORMANCE_LOGGING_STYLE=2`. MoltenVK logs "vkQueueSubmit() encoding to MTLCommandBuffers" and "Retrieve a CAMetalDrawable" separately (`MVKDevice.mm:4764-4766`). Encoding alone is the first minus the second. This tells us whether `queue` is CPU work at all.
  - **(b) Then split the submit.** Add a `Cmd::Submit` (or `Cmd::Flush`) the app pushes before the first pass that targets `Target::Surface`. The backend ends the command buffer there and submits it with no semaphore, then records the rest into a second command buffer per slot, which waits on `acquired` at `COLOR_ATTACHMENT_OUTPUT` and signals `rendered` and the fence. On MoltenVK the first `MTLCommandBuffer` commits and starts on the GPU while the CPU encodes the effects and UI and waits for the drawable.
- **Gain:**
  - Mac: when CPU-limited (weak Macs, or after the GPU work shrinks), the GPU starts each frame several milliseconds sooner.
  - PC: the semaphore wait at `COLOR_ATTACHMENT_OUTPUT` alone lets the compute, shadow and offscreen world passes run before the swapchain image is free (FIFO). That is small on the 5090.
  - The measurement itself is worth the most: it decides whether to chase encoding costs at all.
- **Effort:** measuring, 1 hour (a tester run). Splitting, about a day (backend plus one push in `record_passes`).
- **Measure:** the MoltenVK performance log, `submit split` "queue", and a Metal System Trace (does the command buffer start before the frame's submit ends?).
- **Owner:** Upstream.

### 3. The occlusion depth chain: a 31 MB copy, 13 full barriers and 12 compute encoders a frame

- **What:** `world_render.rs:2064-2105` (`take_depth`) and `lib.rs:1890-1918` (`CopyDepthToBuffer`).
- **Evidence:**
  - Every frame the full-resolution depth (3456 x 2234 x 4 B = 30.9 MB) is copied into a buffer.
  - It is then reduced one level per dispatch, with a `Cmd::Barrier` after each: about 12 levels.
  - With residency sets, MoltenVK 1.4.2 ends the current Metal encoder at every barrier outside a render pass (`MVKCmdPipeline.mm:160-167`), and the next encoder waits on all four stage fences (`MVKCommandBuffer.mm:651-676`). So the chain is about 12 compute encoders.
  - Each of those barriers also does finding 1's resource walk.
- **Change:**
  - Read level 0 straight from the depth texture in the `down` shader (it is bindless already: `r.gpu.texture_index(r.depth.0)`). Drop `CopyDepthToBuffer` and write level 1 first.
  - Reduce several levels per dispatch: a single-pass downsampler (AMD SPD-style) does 6 levels per workgroup in shared memory, so the chain becomes 2 dispatches.
  - Use compute-to-compute barriers: `COMPUTE_SHADER`/`SHADER_STORAGE_WRITE` to `COMPUTE_SHADER`/`SHADER_STORAGE_READ`.
  - Check whether `test`/`cull` ever read level 0. If not, never build it.
- **Gain:**
  - Mac: about 10 fewer encoders and fence waits, plus 31 MB less copied and about 31 MB less read. That is roughly 0.1-0.3 ms CPU and 0.1-0.3 ms GPU.
  - PC: small (the 5090 has the bandwidth).
  - Weak iGPU at 1080p (8 MB): about 0.1-0.3 ms GPU.
- **Effort:** half a day to a day (WGSL plus about 30 lines of Rust).
- **Measure:** a Metal System Trace for the encoder count and the gap between the world pass and the effects. The GPU "effects" part includes this today (it sits after the world mark), so effects should drop.
- **Owner:** Upstream (occlusion is engine core).

### 4. Bug: `CopyDepthToBuffer` uses the wrong old layout and leaves the depth layout untracked

- **What:** `lib.rs:1894-1917`, together with `EndPass` at `lib.rs:1776-1786`.
- **Evidence (code reading):**
  - `EndPass` moves the world depth to `SHADER_READ_ONLY_OPTIMAL` and records that.
  - `take_depth` (`app.rs:3100`) then issues `CopyDepthToBuffer`, whose barrier claims the image is in `DEPTH_ATTACHMENT_OPTIMAL` (wrong). After the copy it moves the image to `DEPTH_ATTACHMENT_OPTIMAL` without updating `tex.layout`.
  - The effects then sample depth that is really in `DEPTH_ATTACHMENT_OPTIMAL`, while the tracked layout says `SHADER_READ_ONLY`. That is undefined behaviour, which validation flags.
  - It is harmless on MoltenVK and probably on NVIDIA. On AMD and Intel, depth layouts can switch compression, so this risks broken depth reads in the effects.
- **Change:** use `tex.layout` as the old layout, return the image to `SHADER_READ_ONLY_OPTIMAL` after the copy, and keep `tex.layout` true. Finding 3 removes this copy entirely.
- **Gain:** correctness on AMD and Intel. No speed gain.
- **Effort:** 15 minutes.
- **Measure:** run with the validation layers (`desc.debug`); the layout error should go.
- **Owner:** Upstream. This could become a Weft PR candidate.

### 5. Exact barrier masks (the code-level detail behind Research 19's "exact stage masks" row) and the acquire wait stage

- **What:**
  - `barrier_image`, `lib.rs:985-1006`: `ALL_COMMANDS`/`MEMORY_WRITE` to `ALL_COMMANDS`/`MEMORY_READ|WRITE`. It runs at every `BeginPass` (`1710`, `1733`) and `EndPass` (`1778`, `1789`), about 2-4 per pass and about 15 passes, so roughly 40-50 a frame.
  - `memory_barrier`, `lib.rs:1008`.
  - The acquire and render semaphores, `lib.rs:1967-1972`.
- **Evidence:** with residency sets, MoltenVK turns each barrier into fence updates and waits per stage (`MVKCommandBuffer.mm:708-723`, and `mvkBarrierStagesFromPipelineStageFlags`, `mvk_datatypes.mm:859-886`). `ALL_COMMANDS` maps to all four stages (vertex, fragment, compute, copy). So every pass's vertex stage waits for the previous pass's fragment stage, and Apple's TBDR cannot overlap the next pass's geometry with the last one's shading. On desktop, `ALL_COMMANDS` to `ALL_COMMANDS` drains the pipe at every pass edge.
- **Change** (sync2 masks):
  - **Colour target, start of a pass:**
    - from `FRAGMENT_SHADER | COMPUTE_SHADER` / none, that is, after earlier sampling (add `COLOR_ATTACHMENT_OUTPUT`/`COLOR_ATTACHMENT_WRITE` when it loads);
    - to `COLOR_ATTACHMENT_OUTPUT` / `COLOR_ATTACHMENT_WRITE` (plus `COLOR_ATTACHMENT_READ` with LOAD).
  - **Colour target, end of a pass:** from `COLOR_ATTACHMENT_OUTPUT`/`COLOR_ATTACHMENT_WRITE` to `FRAGMENT_SHADER | COMPUTE_SHADER`/`SHADER_SAMPLED_READ`.
  - **Depth, start of a pass:** from `FRAGMENT_SHADER | COMPUTE_SHADER | COPY` / none to `EARLY_FRAGMENT_TESTS | LATE_FRAGMENT_TESTS` / `DEPTH_STENCIL_ATTACHMENT_READ | WRITE`.
  - **Depth, end of a pass:** from `LATE_FRAGMENT_TESTS`/`DEPTH_STENCIL_ATTACHMENT_WRITE` to `FRAGMENT_SHADER | COMPUTE_SHADER | COPY`/`SHADER_SAMPLED_READ | TRANSFER_READ`.
  - **`Cmd::Barrier`:** from `COMPUTE_SHADER | COPY | CLEAR` / `SHADER_STORAGE_WRITE | TRANSFER_WRITE` to `DRAW_INDIRECT | VERTEX_SHADER | FRAGMENT_SHADER | COMPUTE_SHADER` / `INDIRECT_COMMAND_READ | SHADER_STORAGE_READ | SHADER_STORAGE_WRITE`. Keep HOST out (finding 1).
  - **Present:** from `COLOR_ATTACHMENT_OUTPUT` to none.
  - **Acquire semaphore:** wait at `COLOR_ATTACHMENT_OUTPUT`. **Rendered semaphore:** signal at `COLOR_ATTACHMENT_OUTPUT`.
  - **Optional:** batch a pass's image barriers into one `vkCmdPipelineBarrier2` call (one `DependencyInfo` with several image barriers) instead of one call each.
- **Gain:**
  - Mac GPU: overlap at about 15 pass edges. There is no Apple number; my guess is 0.2-1 ms of the 16.2. CPU: fewer, smaller barrier commands.
  - PC: the acquire stage lets the frame's offscreen work start before the image is free.
  - All GPUs: no pipe drain at pass edges.
- **Effort:** 2-4 hours. The backend needs the previous use of each image, which it partly tracks already (`tex.layout`).
- **Measure:** a Metal System Trace (vertex and fragment overlap across encoders), the bench GPU total, and validation layers with synchronization validation on.
- **Owner:** Upstream / Lighting. This overlaps Research 19's "tile-memory pass structure" row; these are its exact masks.

### 6. Texture uploads: two full barriers per mip, each ending a Metal blit encoder

- **What:** `lib.rs:1070-1100`, the `Upload::Texture` path.
- **Evidence:**
  - Every mip of every texture written gets an `ALL_COMMANDS` barrier into `TRANSFER_DST`, a copy, and another into `SHADER_READ_ONLY`.
  - In MoltenVK each of those barriers ends the blit encoder (`MVKCmdPipeline.mm:160-167`).
  - Loading a pack (thousands of textures x mips) therefore builds thousands of blit encoders in one submit.
  - Per frame it is small: skins, icons, previews.
- **Change:** collect all of a frame's texture uploads. Issue one `vkCmdPipelineBarrier2` with every image barrier into `TRANSFER_DST`, then all copies, then one barrier into `SHADER_READ_ONLY` (`COPY` to `FRAGMENT_SHADER | COMPUTE_SHADER`). Do the same for `InitTexture`.
- **Gain:** load and reload, and the first frame after a pack change. My guess is tens to hundreds of milliseconds on the Mac (unmeasured). Little per frame.
- **Effort:** 1-2 hours.
- **Measure:** the time of the first frame after load (`slow_record` spans), and `queue` max in `submit split` during a reload.
- **Owner:** Upstream.

### 7. O(n²) structure lookups in `draw` and `draw_shadow`

- **What:**
  - `world_render.rs:1354`, `frames.iter().position(...)` for every structure cube with light;
  - `world_render.rs:1402`, the same per mesh inside the cull, on every core;
  - `world_render.rs:782-795`, for every leaning structure, a walk over **all** `regions`;
  - `client/mod.rs:3001-3010`, `world_light.values().max()` per structure per frame.
- **Evidence (code reading):** the cost is structures x meshes x structures. Chiacraft makes a structure per felled tree (lying logs stay structures?), so the count can grow during play. It is cheap at 5 structures; at 300 it is about 300 x 5 x 300, or 450k compares a frame, roughly 0.2-0.5 ms.
- **Change:** build `FxHashMap<u64, usize>` (structure id to index into `frames`) once per frame and use it in all three places. Key `regions` by structure id first (`FxHashMap<u64, Vec<region>>`) so a leaning one finds its own regions directly.
- **Gain:** about 0 today. It grows with structure count, about 3x on weak CPUs.
- **Effort:** 30 minutes.
- **Measure:** the `world/cull` span, and a bench after felling 100+ trees. Add a `structures` count to the frame line.
- **Owner:** Upstream / Terrain (felling).

### 8. The shadow pass culls every opaque cube on the CPU every frame

- **What:** `world_render.rs:749-822`. Every frame it walks every region's meshes **and every `table_only` mesh**, which is every opaque cube in the world, with no region pre-cull. It runs serially on the frame thread, then sorts, and allocates a fresh `draws` Vec (finding 9).
- **Evidence (code reading):** the main view moved opaque cubes to the GPU (`CubeTable`) so the CPU cost wouldn't grow with view distance, but the near cascade still pays it. At view 12, `table_only` is about 10-40k meshes, each tested against 6 planes and then `mesh_draws`. The `shadows` span in `app.rs:3072` covers it, and the whole `record` is only 0.34-0.49 ms on the Mac, so this is at most a part of that. The PC bench shows shadows double the draw count (1,141 draws with shadows off, about 2,300 on): that is the records MoltenVK encodes, not this CPU loop.
- **Change:**
  - **Cheap:** keep `table_only` in region buckets like `regions` and test each region's box (`side * 0.8661`) first, as `draw` does.
  - **Better:** cache the near cascade's `draws` while `cam_cube`, the light matrix (it moves in whole texels) and `terrain_epoch` all stay the same. Only the camera-relative `origins` and `cam_frac` change; `cam_frac` is in `Globals` and the origins are relative to `cam_cube`.
  - **Later:** cull shadows on the GPU with `CubeTable` and the light's planes (a second `cull` dispatch). On the Mac that needs the count cap from `fix/indirect-cap`.
- **Gain:** about 0.05-0.3 ms CPU on Mac and PC, about 3x on weak CPUs. Caching also lets the frames when nothing moves skip the sort and the writes.
- **Effort:** 2-4 hours (region buckets plus the cache).
- **Measure:** add `diag::span("shadow/cull")` around `world_render.rs:749-822`, and a counter of the near cascade's records.
- **Owner:** Lighting (shadows) / Upstream.

### 9. Per-frame allocations in `draw`, `draw_shadow` and `render`

- **What:**
  - `world_render.rs:1454-1464`: `counts: Vec<Vec<u32>>`, one `vec![0; 8 * pages]` per seen region.
  - `1477-1483`: `starts`, one `next.clone()` per region. With 300-1,000 seen regions that is about 600-2,000 allocations a frame, on rayon workers.
  - `749`: `draws` is a fresh Vec per cascade per frame (thousands of 44-byte `DrawRecord`s, about 12 regrowths).
  - `1339`, `1347`, `1352`, `1364`, `1494`, `1646`: `frame_offsets`, `picks`, `cube_pick`, `seen`, `lod`, `groups`.
  - `client/mod.rs:2977-2994`, `2999`, `3056`, `2906`: `frames`, `lights`, `leans`, `lit`, `cracks`, `leaning`.
  - `models.rs` `draw_list`, `insts` collected per batch (it could write straight into the mapped buffer).
  - `lib.rs:1039`, `1051`, `1680`, `1749`: `chunks`, the `written` BTreeMap, `in_pass_targets`, `colors`.
- **Evidence (code reading):** the code already reuses `parts`, `trans` and `runs` because "allocating them anew on every core each frame cost more than the culling" (comment at `world_render.rs:1380`). The same applies to `counts` and `starts`.
- **Change:** keep `counts` and `starts` as one flat `Vec<u32>` of `seen.len() * nb` in `self`, indexed by region. Keep `draws`, `seen`, `lod` and the maps in `self` and `clear()` them. Have `groups()` return a slice cached when `per_bucket` changes.
- **Gain:** about 20-100 µs a frame on Mac and PC, more on weak CPUs and with a fragmented allocator.
- **Effort:** 1-2 hours.
- **Measure:** `samply` (malloc/free under `WorldRender::draw`), or a counting global allocator in a bench build (`dhat`).
- **Owner:** Upstream.

### 10. Translucent runs split by page, and `bind_run` re-sends all its state per run

- **What:**
  - `world_render.rs:1602-1609`: translucent runs are cut at every change of (kind, layer, page) in far-to-near order.
  - `bind_run`, `world_render.rs:1693-1710`: per run it records an `IndexBuffer`, a `SetPipeline` (which in the backend rebinds descriptor set 0 every time, `lib.rs:1799-1805`), and 12 push-descriptor calls (`lib.rs:1806-1821`, each a separate `vkCmdPushDescriptorSetKHR`).
- **Evidence:**
  - The PC benches show only 15-20 pipeline binds a frame on land. Over water or glass, quads from cubes in different pages interleave by distance, so runs can approach one per translucent draw. Each run is then about 14 Vulkan calls, plus MoltenVK re-binding 12 buffers for two stages.
  - That has not been measured in a water scene.
  - The push-constant range differs per pipeline layout (`lib.rs:1267`), which makes set 0 incompatible between layouts, so it really must be rebound.
- **Change:**
  - In the backend, skip a `SetPipeline` to the pipeline already bound.
  - Cache the last buffer per (bind point, binding) and skip identical `BindBuffer`s. Then a run that changes only the page re-pushes just `RECORDS`.
  - Batch a run's 12 writes into one `cmd_push_descriptor_set` call (one array of `WriteDescriptorSet`).
  - Give every layout the same push-constant range (for example 128 bytes, `ShaderStageFlags::ALL`), so set 0 is bound once per bind point per command buffer.
  - In the engine: within a translucent run, merge across pages when the distance order allows. Or, for translucency, bind both face and quad pages and pick by a per-draw page index.
- **Gain:**
  - Land scenes: small.
  - Water or glass views on the Mac: possibly 0.1-0.5 ms of `queue` (unmeasured).
  - The bind-batching part helps every platform a little.
- **Effort:** 2-4 hours (backend caching plus one batched write).
- **Measure:** the bench's "pipeline binds" and "commands" in a lake or ocean script. Add a counter of `BindBuffer`s actually pushed after the cache.
- **Owner:** Upstream.

### 11. MoltenVK's encoding happens on the main thread, serial with the game update

- **What:** `app.rs:2321-2350`: `begin_frame`, update, `record_passes` and `end_frame` all run one after another on one thread.
- **Evidence:** `queue` is 4.5-7 ms of `vkQueueSubmit2` on the main thread (minus any drawable wait, finding 2). While the Mac is GPU-bound it costs nothing. Once the GPU work shrinks (TAAU and the cap) the Mac becomes CPU-bound at roughly update + ui + record + 5 ms.
- **Change:** a render thread that owns the backend's submit: the main thread hands it the finished `CommandList` (double-buffered) and goes on to the next update. `CommandList` is plain data, so this is mostly ownership plumbing (`Backend: Send` already holds). A cheaper, partial version is finding 2's split submit.
- **Gain:** Mac, when CPU-bound: up to the full 4-7 ms per frame overlapped. PC: about 0. Weak CPUs: whatever the driver's submit costs.
- **Effort:** several days. It touches the frame loop, uploads (`write_buffer` from the main thread) and captures.
- **Measure:** frame time versus `queue` once GPU time falls below CPU time. Or, now, a `--render-scale 0.5` run to make the Mac CPU-bound.
- **Owner:** Upstream.

## Looked at, no real cost found

- **`entity_render.rs` `draw` and `models.rs`:** poses run in parallel (`add_many`, `with_min_len(64)`), far poses are kept for 2-8 frames, and models draw instanced per kind (one `Draw` per kind), so items draw in runs. The cost per entity is flat. There is no spatial index (every entity is distance-checked), which is fine up to thousands. The only waste is `insts` (finding 9).
- **`CubeTable::upload`:** it uploads sorted, deduplicated runs of neighbours only. That is good.
- **`frame_buffer`:** grows by powers of two and is per slot; no per-frame allocation.
- **`emit` and the main view's draw building:** they write straight into mapped memory from every core, with no sort for opaque. That is good.

## Sources

The MoltenVK v1.4.2 source:

- `MVKDevice.mm` (`applyMemoryBarrier`, the activity names)
- `MVKCmdPipeline.mm` (`MVKCmdPipelineBarrier::encode`)
- `MVKCommandBuffer.mm` (`setBarrier`, `barrierWait`, `markTimestamp`, `encodeTimestampStageCounterSamples`)
- `mvk_datatypes.mm` (`mvkBarrierStagesFromPipelineStageFlags`)
- `MVKCmdDraw.mm` (the indirect loop)
- `MVKImage.mm` (`getCAMetalDrawable`)
- `MVKDescriptorSet.mm:324` (with residency sets, bindless textures need no per-encoder `useResource`, so our 65,536-entry table costs nothing per draw on macOS 15+ when MoltenVK is built with Xcode 16; without residency sets it would cost a lot)

All from https://github.com/KhronosGroup/MoltenVK/tree/v1.4.2/MoltenVK/MoltenVK
