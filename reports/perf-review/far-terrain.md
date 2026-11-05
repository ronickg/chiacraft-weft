# Far terrain tiles: where the CPU time goes (review, 2026-11-05)

Read-only review of `perf-review` (87234e26). Nothing was built, run or profiled. Every number
comes from the logs: Mac `~/Coding/chiacraft-archive/images/tester/2026-11-05/lodmac_1105-002815/run.log`,
and PC `~/Coding/chiacraft-archive/images/tester-2/2026-11-04/runs/t2_*/run.log` (the same
`distant terrain per 1.0 s` lines). Gains are estimates from code reading. Each finding names the
counter that would confirm it.

## The frame of it: fill time is CPU per tile divided by threads

- The pool is `lod.rs:119-130`, `cores/3` clamped to 3..6. This Mac (M3 Max, 10P+4E = 14) gets **4
  threads**, the PC 6, a 4-core 3. Heights, meshing and `describe_lod` (`terrain_gpu.rs:491`) all
  share this one rayon pool. They are spawned from outside it, so they go through its FIFO
  injector. Up to 256 tiles are in flight (`IN_FLIGHT`). The GPU takes 64 a frame
  (`terrain_gpu.rs:29`) and is not the limit.
- Little's law fits the logs closely:
  - Mac: 4 threads / (15 ms heights + 8 ms mesh) ≈ 175 tiles/s. Measured: 1092 tiles over
    4.79-11.02 s = **175/s**.
  - PC: 6 / (11.6 + 7.3 ms) ≈ 317/s. Measured: **323/s** at 9.2 s.
  - So every ms taken off a tile shortens the fill linearly. Mac: 879 tiles / 175 per s ≈ 5 s, plus
    a 3 s start stall (finding 4), gives the logged "9.4 s".
- Meshing outside face calls costs 30-100 µs a tile:
  - Mac 31.97 s: 421 µs mesh, 339 µs of it in faces.
  - PC 6.2 s: 3867 µs mesh, 3833 µs in faces.
  - `mesh()` allocations (`walls`, `out`) and `face_of` are therefore not worth touching.
- The face-call cost is not intrinsic. Tiles re-made near the player, where the cube cache holds
  the columns, cost **2.8-4 µs a call** (Mac 31.97 s: 339/82; 62.6 s: 465/166). Far tiles cost
  **63-155 µs** (Mac) and **63-74 µs** (PC). The difference is columns computed from scratch
  inside each call (findings 1 and 2).

## Ranked findings (gain per effort)

### 1. A steep cell asks the same column up to 9 times. Memo it in base's faces closure
- **What:** `crates/engine/src/client/lod.rs:338-354` calls `face_block` for the top, then once
  per band (`BANDS` = 8) down the wall, always at the same (x, z). Each call into
  `mods/base/src/land.rs:106-113` runs `f.surface_block()` (finding 2: about 6 column computes)
  and `l::rock_set` (climate plus `KARST_AREA`/`MOUNTAINS_AREA` fbm). Both depend only on
  (seed, x, z, height), not on `wy`.
- **Evidence:** 89-147 face calls a tile at 60-155 µs each = 5.6-22.8 ms, nearly all of the mesh
  time. A wall reaches at least the skirt (2 cells) down, so a box cell usually makes 9 calls.
  That puts about 10-16 steep cells in a tile.
- **Change:** a one-entry `thread_local` in the closure, keyed (seed, x, z, height), holding
  `rocks` (with `sandy` folded in). The 8 band calls then cost about 1-2 µs each (two `perlin3`
  for the patch and the ore). The result is exact.
- **Gain, box tiles (levels 0-2):** face time ÷ ~6-9.
  - Mac: mesh 5.5-26 → ~1-3 ms a tile (−4 to −20 ms).
  - PC: 3.9-7.3 → ~0.6-1 ms.
  - 4-core: similar share, about −35% of tile CPU.
  - Smooth tiles (levels 3-5, `mesh_smooth`, 1 call a cell) gain nothing: they need finding 2.
- **Effort:** 0.5 h. **Owner:** Terrain.
- **Measure:** the existing `face calls` µs. Add `diag::LOD_FACE_CELLS` (steep cells a tile) beside
  `LOD_FACES` to confirm calls per cell.

### 2. `surface_block` under `ONE_COLUMN` still computes 6 columns, not 1
- **What:** `crates/engine/src/server/terrain.rs:910-929` calls `surface_carved` first (`:876-893`):
  - `self.column(x, z)`, which is a compute on a cache miss;
  - 16 `cave_node` calls;
  - **4 corner columns** (`:891`), each a full compute;
  - then the `ONE_COLUMN` branch computes (x, z) **again** (`:921-924`).
  - That is 6 full columns (every feature, 9 mod callbacks each) per face call. e597eaf4 and
    4f4afade fixed the 256-column case but left this path.
- **Evidence:**
  - e597eaf4's note puts 256 columns at 1-2 ms, which is 4-8 µs a column.
  - `custom_heights` puts it at about 13 µs (15 ms / 1156).
  - 6 columns ≈ 25-80 µs, plus caves and rocks, against the measured 63-155 µs a call. Cached
    columns (near the player) give 3-4 µs.
- **Change:**
  - Smallest: under `ONE_COLUMN`, compute (col, rule) once and skip the carve test. Far faces
    don't show cave mouths, and the result changes only in rare columns where caves breach the
    surface.
  - Cleaner (Upstream ABI, chiacraft's `FACES` addition): add `surface` to `abi::FaceArgs`, filled
    from the GPU sample's `top` (it already has it), and have `sdk::terrain::Face::surface_block`
    (`crates/sdk/src/terrain.rs:928-931`) return it. Zero CPU columns.
- **Gain:**
  - With 1 done: the remaining ~90 µs a steep cell → ~15 µs (or ~0 with the ABI field).
  - Without 1: face time ÷ ~5. Smooth tiles gain the same factor.
  - Together with 1: Mac mesh 5.5-26 → ~0.3-1 ms a tile; PC 4-7 → ~0.3 ms.
- **Effort:** 1 h (skip carve), 2-3 h (ABI field). **Owner:** Upstream (engine FFI / terrain).
- **Measure:** `face calls` µs. Add `diag::LOD_FACE_COLUMNS`, counting `compute()` calls made
  while `ONE_COLUMN` is set (should drop from ~6 to ≤1 a call).

### 3. `tarns` and `tarn_streams` compute the whole land eagerly at every column
- **What:** `mods/base/api/src/land.rs`:
  - `:847` `tarns()` = `tarns::cut(f, x, z, before_tarns(f, x, z), …)`;
  - `:829` `tarn_streams()` = `stream_cut(f, x, z, with_tarns(f, x, z))`;
  - `:811-813` `with_tarns`, `:841-844` `before_tarns`.
  - The `base` argument is computed **before** `cut` or `stream_bed` look for a tarn. Yet
    `tarns::cut` (`land/tarns.rs:126-134`) returns 0 unless the column is in a bowl, and
    `stream_bed` (`tarns.rs:249-256`) returns `None` almost everywhere.
  - So every column, near or far, pays `before_tarns` twice:
    - `land_height`, which is `base_ground` (≈30-40 `perlin2` octaves: continent, hills twice,
      dunes, ridge, mesa, area noises) plus canyons, karst, reefs and `lakes_on`;
    - plus `rivers`, which recomputes `land_height_at` again when near a river;
    - plus craters.
- **Evidence:**
  - Heights are 15-21 ms a tile (Mac) and 11.6-15 ms (PC). That is 13-18 µs a column over the 9
    callbacks (1156 samples).
  - `reports/terrain-terralith-tectonic.md:104` (11-04, before tarns at 16:45 and streams at
    17:04) had every mod feature at ~0.5 ms a chunk column, ~1.8 µs a column, plus 0.3-0.4 µs of
    lakes.
  - `mods/base/api/examples/land_time.rs` does not time `tarns` or `tarn_streams` at all, so the
    jump went unseen.
  - Inference, not a profile: these two are likely the largest single cost in heights.
- **Change:** look up first, compute the ground after.
  - `tarns`: `tarns::at` first; `before_tarns` only inside a bowl.
  - `tarn_streams`: `stream_bed` first; `with_tarns` only on a stream column.
  - Pure functions, the same integers: exact.
- **Gain (estimate, ±50%):** removes 2 of ~3 full land evaluations a column.
  - Mac heights 15 → ~6-8 ms a tile.
  - PC 12 → ~5-6.
  - 4-core ~25 → ~11.
  - Near cube generation (`Terrain::compute`) gains the same per column, and it shares the CPU.
  - Face calls get cheaper too, since each computes columns.
- **Effort:** 1-2 h, including two `land_time` lines. **Owner:** Terrain.
- **Measure:** add `tarns` and `tarn_streams` to `land_time` (before and after). Then the
  `heights … us` per tile in the load log.

### 4. Meshing and `describe_lod` wait in FIFO behind up to 256 heights jobs
- **What:**
  - `lod.rs:1031` spawns up to 256 heights jobs.
  - `lod.rs:149` (`made`) spawns each mesh job onto the **same** pool from the render thread,
    into the injector's FIFO, so a mesh waits for every heights job queued before it.
  - `terrain_gpu.rs:488-491` spawns `describe_lod` only when the first finished heights job
    reaches the GPU side, and it also queues behind the rest.
- **Evidence:**
  - Mac 2.76-4.79 s: "0 tiles made … 256 asked" for 3 s, then the first 255 tiles at **3645 ms**
    from asked to made.
  - Mac after the teleport (46.4-48.4 s): 0 tiles for 3 s again, then 3528 ms. At 52-65 ms of
    heights a tile, 256 × 60 / 4 ≈ 3.8 s: it is the queue.
  - PC: 3 s of 0, then 2643 ms.
  - Steady state: 1.0-1.8 s asked-to-made = 256 in flight ÷ 175/s. So a new nearest tile after
    a turn waits behind ~255 others.
- **Change:**
  - Cap heights in flight at ~3 × threads, separately from `IN_FLIGHT`.
  - Give meshes priority: a crossbeam queue of mesh jobs that every LOD job drains first.
  - Spawn `describe_lod` at the dim/seed change, before any heights. `Lod::update` already runs
    `describe` there (`lod.rs:724`).
- **Gain:**
  - CPU per tile is unchanged.
  - First far tiles after load or teleport: ~3-3.6 s → ~0.3-0.6 s.
  - Asked-to-made: 1-1.8 s → ~0.2-0.4 s, and nearest-first really holds.
  - Largest on a 4-core (3 threads: 256 × ~45 ms / 3 ≈ 3.8 s of queue).
- **Effort:** 2-3 h. **Owner:** Distant terrain.
- **Measure:** the existing "ms from asked to made" and "every wanted tile made … after" lines.
  Add `diag::LOD_MESH_WAIT_NS` (from `made()` to mesh start) and `diag::LOD_FIRST_MS` (from reset
  to first tile made).

### 5. Each of the 9 callbacks rebuilds the same column's climate and ground; the LOD loop order defeats a memo
- **What:**
  - Every custom feature starts from a fresh `Here::new` (`land.rs:116`), so the climate `Here`
    caches (continent fbm4, temperature fbm3, humidity fbm3 = 10 `perlin2`) is recomputed in
    ~8-10 callbacks a column. `Ramp::Noise` areas (`land.rs:140-147`) are never cached.
    `base_ground` evaluates `shape::hills` twice with the same noise (`land.rs:793-794`), and the
    engine does too, for its two `preset::hills` features.
  - `custom_heights` (`lod.rs:1054-1058`) loops **feature-major** (every column for feature 1,
    then feature 2…), so no per-column memo can hit there.
- **Change:**
  - (a) Distant terrain: swap the loops to column-major, writing `out[f * SAMPLES + k]` (5 min).
  - (b) Terrain: a one-entry per-thread memo keyed (seed, x, z), holding the climate fields,
    `base_ground`, `ground_at`, `land_height`, `rivers` and `before_tarns`/`with_tarns`. The
    callbacks run in order on one column (`Terrain::compute` already does that, and (a) makes
    the LOD path do it too), so each callback adds only its own delta. Exact.
  - Upstream alternative: pass the engine's running sum into `FeatureArgs`. By construction it
    equals `base_ground`/`ground_at` ("the same integers the engine sums", `land.rs:767`), so
    no callback recomputes the ground. In the LOD path this needs the Expr features on the CPU,
    ~1-2 µs.
- **Gain, after 3:** heights ~6-8 → ~3-4 ms a tile (Mac), PC ~5 → ~2.5, 4-core ~11 → ~5.
  Near generation gains similarly.
- **Effort:** 3-5 h. **Owner:** Terrain, with Distant terrain for (a).
- **Measure:** a new `land_time` line, "all 9 features at one column in engine order". Then
  `heights` µs per tile.

### 6. `host::invoke` does 2 clock reads and 2 contended atomics per call, ~10.4k calls a tile
- **What:** `crates/engine/src/host.rs:595-616`:
  - `Instant::now()` twice;
  - TLS enter and leave;
  - `fetch_add` on `STATS[owner].nanos` and `.calls`, one cache line shared by every thread
    running base callbacks (4-6 LOD threads plus the near-generation pool).
  - Heights make 9 × 1156 = 10.4k calls a tile; face calls add 9 × 6 more each (finding 2).
- **Evidence:** none measured. At an estimated 50-200 ns a call under contention that is 0.5-2 ms
  a tile: ~5-13% of heights now, 20-40% once 3 and 5 land.
- **Change:**
  - Per-thread stat accumulators, flushed every ~1k calls or on the diag tick.
  - Or no per-call timing for pure terrain callbacks (`FeatureArgs`, `FaceArgs`): their callers
    already time the batch.
  - Longer term: one batched call per column (all of a mod's custom features) or per tile.
- **Gain:** Mac/PC −0.5-2 ms a tile (high uncertainty). The most on the Mac, where lines bounce
  between P-clusters. Small on a 4-core.
- **Effort:** 1-2 h. **Owner:** Upstream.
- **Measure:** samply on a Mac LOD fill, self time in `invoke`, `Instant::now` and the atomics
  (Tester runs it). Then compare `heights` µs per tile.

### 7. The engine's cave interpolation evaluates every node twice
- **What:** `terrain.rs:1048-1062`. `interpolate` calls `field(0)` and then `field(1)`, each
  calling `node()` 8 times, so `cave_node` runs 16 times (48 `perlin3`) instead of 8 (24).
  `surface_carved` uses it (face calls until 2 lands, and near decorations: trees, spawn).
- **Change:** evaluate the 8 corners once into an array, then lerp both fields. Exact.
- **Gain:** ~1-3 µs a `surface_carved` call. About −0.1-0.4 ms a far tile until 2 lands, more for
  near scatter.
- **Effort:** 15 min. **Owner:** Upstream.
- **Measure:** `face calls` µs. The near side through scatter timing.

### 8. Thread-local site caches: each thread rebuilds every tarn, stream, lake, karst, crater and reef site, behind SipHash
- **What:**
  - `land/tarns.rs:47,168` (`SITES`, `STREAMS`), `land.rs:898-903` (`LAKE_SITES`), and karst,
    craters and reefs (`SHORES`): `thread_local` std `HashMap`s, each cleared whole at 4096.
  - A tarn site (`tarns.rs:66-117`) tries up to `TARN_TRIES` points, each costing a ring and a
    bowl of full-land `ground()` evaluations (hundreds). A stream `trace` walks with 4-8
    `with_tarns` per step.
  - Each LOD thread and each near-generation thread does this separately.
  - Per column, `stream_bed` makes 9 SipHash lookups plus one `beds.get`, and the other caches
    add about 5-10 more.
- **Evidence:** cold tiles cost 36-64 ms of heights (Mac 2.76-4.79 s and 46-48 s; PC 38 ms at
  3.2 s) against 15 steady. River regions (finding 9) also go into that cold cost; the split is
  unknown.
- **Change:** a process-wide sharded cache (like `server::terrain::ColumnCache`) or per-region
  precompute beside `net::Region`. `FxHashMap` everywhere (`rustc-hash` is already in the tree).
- **Gain:** cold tiles −10-40 ms each for the first 2-3 s after load or teleport (uncertain).
  Steady state −0.3-0.6 ms a tile from FxHash.
- **Effort:** 15 min (FxHash), 2-4 h (shared). **Owner:** Terrain.
- **Measure:** static `AtomicU64` build counters per site kind in `base_api`, printed by a
  `land_time` multi-thread cold run (4 threads over the same tile).

### 9. River regions: built 16× over, single-threaded, with other pool threads blocked waiting
- **What:** `land/net.rs:96-130`. A region of 32 × 32 nodes is built over 130 × 130 nodes
  (margin 47 = border 6 + `RIVER_REACH` 40 + 1), each with `route_height` and humidity. So every
  node is computed in ~16.5 regions. `region_at` (`net.rs:340-360`) builds it under a `OnceLock`
  with the other askers blocked: that is blocking inside a pool job. The global lookup on a
  thread-local miss is a std `Mutex<HashMap>` with SipHash, cheap (~100 misses a tile).
- **Evidence:** `land_time` prints "river network: X ms a region". Its number is not in the logs
  read here. LOD reach 4096 blocks spans ~36-49 regions of 1536 blocks.
- **Change:**
  - A world-scale node cache (per 32×32 node block), so a region's margin reuses its neighbours'
    nodes. This is Round 19 #9's first step, made concrete.
  - Or split the node loop with `rayon::join` inside the build.
- **Gain:** at load or teleport only: −(15/16 of region build) × regions. Perhaps 0.5-1.5 s of
  CPU at load (uncertain). None in steady state.
- **Effort:** 3-6 h. **Owner:** Terrain (feeds Round 19 #9, not a duplicate).
- **Measure:** `land_time`'s per-region ms. Add a regions-built counter.

### 10. Threads: Mac LOD threads aren't lowered; the Mac gets only 4
- **What:** `crate::lower_priority` (`crates/engine/src/lib.rs:125-145`) is a no-op on macOS. The
  LOD pool runs at normal priority beside the main and render threads, while on Windows and Linux
  it is below normal. `cores/3` gives this Mac 4 threads.
- **Change (needs a bench):** QoS `UTILITY` on Mac LOD threads. Risk: it steers them to the 4
  E-cores. Only after 1-5 shrink the work, consider `cores/3` rounding up on the Mac.
- **Gain:** frame smoothness during fills, not tile speed. Uncertain.
- **Effort:** 1 h plus a bench. **Owner:** Distant terrain / Upstream.
- **Measure:** `--log frame=debug` cpu update and record during a fill, with and without.

## If 1-5 land (rough)

| | Mac | PC | 4-core |
|---|---|---|---|
| heights a tile, now → after | 15 → ~3-4 ms | 12 → ~2.5-3 | ~25 → ~5-6 |
| mesh a tile, now → after | 8 → ~0.5-1 ms | 6 → ~0.3 | ~13 → ~1 |
| tiles/s | 175 → ~800-1100 | 320 → ~1700-2000 | ~80 → ~450 |
| full fill (≈880 tiles) | 9.4 s → ~1.5 s | ~9 s → <1 s | ~15 s → ~2.5 s |

Treat these numbers as hypotheses. The 4-core column assumes cores ~1.7× slower than the M3's P
cores and 3 threads. Confirm 3 and 5 with `land_time` before trusting them.

## Checked, not findings
- `mesh()` and `mesh_smooth()` themselves (allocations, `face_of`, run merging): ≤100 µs a tile.
- `take_columns`: 1156 Fx lookups, only when the server sent columns.
- `region_at`'s global mutex: rare misses thanks to `LAST`.
- `ColumnCache` shard locks in `column()`: 6 short `parking_lot` locks a face call, small next to
  the computes.
- The `words.to_vec()` per tile (18 KB) and the GPU batch of 64 a frame.
- `Fields::with_continent_scale` per callback: two words.
