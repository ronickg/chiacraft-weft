# Speed review: per-frame client update and near terrain (branch perf-review)

Read only. Nothing was run. The evidence is the code plus the Mac log
`~/Coding/chiacraft-archive/images/tester/2026-11-05/lodmac_1105-002815/run.log`
(M3 Max, view 12, fresh world, script `tester-tools/scripts/lod_loading_mac_2026-11-05.txt`).

## First: what the log numbers actually are

- **The slow frames of 400-620 ms are screenshots, not loading.** All nine frames over 400 ms
  (464, 415, 462, 451, 548, 560, 558, 451, 623, 594, 467 ms) fall in the second of an
  `[app] screenshot saved` line. The 198 ms and 369 ms frames at 2.7 s and 3.8 s are the script
  switching the shader pack (`shader pack chiacraft on` 1.82 s, `world shader reloaded` 3.24 s,
  `assets reloaded` 3.41 s). The real hitches while loading are 30-70 ms (46.4 s: 52, 90.6 s: 48,
  104.3 s: 69 ms, and that second also has one 20 ms texture load).
- **"server insert 6.61 ms" is not frame time.** It is the `insert` phase of the *last* server
  tick when the line was printed (`diag.rs` reads `phase_us`, a snapshot), and it runs on the
  server thread. The testkit summary says inserts cost 30 us each, so 2.5k cubes/s is about
  75 ms/s on that thread. It reaches the frame only by taking shared cores. The rescans around
  it are not timed at all (finding 8).
- **"cpu update" while loading is mostly the two time budgets doing their job, plus untimed work.**
  `take_messages` (`client/mod.rs:1139-1150`) gives "take terrain" (`receive.rs:404`) and
  "upload meshes" (`mod.rs:1833`) each `(FRAME_TARGET 16 ms - rest)/2`, clamped to 2-5 ms. The
  controller fills the frame up to 16 ms whenever work is waiting:
  - At spawn (frames of 11-13 ms) that gives the 3-5 ms.
  - From 7 to 11 s, cpu update stays at 4.7-5.2 ms with no new cubes. The upload queue was still
    draining: the GPU's `world` time climbs from 1.1 to 7.6 ms until 12 s, then stays flat.
  - In flight (89-92 s), frames are CPU-bound: gpu wait 0, acquire 0.04 ms, cpu update
    11.6-13.1 ms, submit 4-6 ms, frames of 17-21 ms. Both budgets cannot exceed 2 x 2-3 ms there,
    so **at least 6-8 ms a frame is outside any time check.** That is finding 2.
- Settled, cpu update is 0.07-0.3 ms. The entity, interpolation and seat_riders loops are not a
  problem in this run (no mobs). See "Checked, not the cost" at the end.

## Findings, ranked by gain per effort

### 1. The screenshot PNG is encoded on the main thread

- **What:** `app.rs:3176-3190` `save_screenshot` calls `assets::encode_png` (3456x2234 RGBA)
  inside the frame. It is called from `app.rs:2363`.
- **Evidence:** 9 of 9 frames over 400 ms sit in a second with `screenshot saved`. Each costs
  450-620 ms. They also pollute every bench's max and p99 near a `shot`.
- **Change:** move `cap` into `std::thread::spawn` (or `background()`): encode, then send the
  chat line back through a channel. The mod-path variant (`shot_path`) needs the same treatment.
- **Gain:**
  - Mac: removes 0.45-0.6 s frames.
  - PC: similar share.
  - Weak 4-core: about 1 s.
  - Loading is not affected, but every measured "slow frame while loading" was really this.
- **Effort:** about 30 minutes.
- **Measure:** frame `max` in seconds with a shot. It should drop to the 30-50 ms the seconds
  around it show.

### 2. Untimed work in `Client::update`: 6-8 ms a frame in CPU-bound loading frames

- **What:** these steps run outside both budget checks:
  - `world.end_batch()` runs after the budget has already run out (`receive.rs:442`, body at
    `world.rs:420-481`). Each batch sorts 27 x cubes positions and runs `neighbors_known`
    (27 lookups) per member, through `par_iter` on the global pool, and the main thread waits.
  - `render.remove` for every removed cube (`receive.rs:19-21`).
  - The `now` uploads (`mod.rs:1827-1831`, from `world.rs:953-994`): everything within 2 cubes
    of the player, plus finished whole batches, with no budget.
  - `schedule_meshing` / `start_jobs` (`world.rs:849-949`):
    - the in-flight cap is meshing threads x 512 = 6144 on the Mac;
    - each job does 27 hash lookups and 27 `Arc` clones;
    - the main thread does one `pool.spawn` per job;
    - all of `ready` is re-sorted whenever the player moved more than 1 cube (every ~0.2 s at
      140 blocks/s).
  - `update_solid` (finding 10), `take_tints` (finding 7), and `lod.update`: `upload_lod` takes
    every finished tile with no budget (`lod.rs:737-745`), then `update_mask`.
- **Evidence:** the arithmetic above (89-92 s: cpu update 12-13 ms, frames 19-21 ms, budget
  floored near 2 ms each). The same holds at 46-48 s after the teleport (cpu update 7-11.5 ms,
  gpu wait 0-0.8 ms). Which step dominates is **not known**: `update` has no `diag::span` at all,
  and `slow_record` only logs spans when record is over 20 ms.
- **Change:**
  1. Add `diag::span`s, plus a "slow update" log line like `slow_record` (update over 8 ms):
     - `update/receive`
     - `update/take_terrain`
     - `update/end_batch`
     - `update/removed`
     - `update/tints`
     - `update/schedule_meshing`
     - `update/solid`
     - `update/uploads_now`
     - `update/uploads`
     - `update/lod`
     - `update/lod_mask`
  2. Then put the biggest under the budget. For example: count `end_batch` and the removals into
     `spent`, split very large batches, cap `start_jobs` per frame (about 1-2 frames of work, not
     6144), and give `upload_lod` the upload budget.
  3. Second lever: `FRAME_TARGET` is a fixed 16 ms (`mod.rs:473`). On the 120 Hz Mac the
     controller deliberately trades the frame down to 60 fps while anything loads. Derive it from
     the display interval or the settled frame time.
- **Gain:**
  - Mac flight and teleport: frames from 17-21 ms toward 12-14 ms, if the untimed 6-8 ms is
    capped. Near terrain fills a little slower; LOD covers that.
  - PC: smaller (faster CPU).
  - Weak 4-core: the largest, because every untimed step is ~3x slower there.
  - This is a trade-off, not free speed.
- **Effort:** spans 1-2 h. Capping is small to medium per step.
- **Measure:** the new spans. Also cpu update versus `submit` in the `[frame]` line during
  `travel` and after `/tp`.

### 3. The client's terrain finishing waits in the same pool as the server's generation

- **What:**
  - `terrain_gpu.rs:563` (`background().spawn(finish)`) and `client/terrain.rs:831` (the GPU jobs'
    `custom_heights`) both use `crate::background()`.
  - So does `server/load.rs:257` (`spawn_generation`, CPU worldgen of the simulation distance).
  - All three go into one FIFO injector.
- **Evidence:**
  - First second: the server generated 2521 cubes at 3524 us each, about 8.8 core-s/s. The client
    finished 8846 at 483 us, about 4.3 core-s/s. That is 13 core-s/s of demand on a 12-thread pool:
    saturated.
  - Finish time per cube swings from 278 us to 3434 us (46 s) and 10915 us (97.7 s) while the
    pool is shared.
  - The integrated server's cubes are not what the player sees: the client makes its own.
- **Change:** give client terrain its own lane:
  - a `client_terrain()` pool of `workers()` threads; or
  - finish inline in the unpack job, with a larger unpack pool; or
  - a small server-generation pool (`workers()/2`) at a lower priority than the client
    (finding 4).
  - Keep the GPU `custom_heights` on the client lane.
- **Gain:**
  - Mac: near terrain at spawn and teleport fills maybe 1.5-2x sooner. Frame time is unchanged
    (the work is time-budgeted).
  - PC (many cores): small.
  - Weak 4-core: large. `background()` is 2 threads there. The server's ~7.5k cubes per place at
    several ms each would sit ahead of the client's finishing for tens of seconds. This is an
    estimate, not measured.
- **Effort:** small (a few hours).
- **Measure:** existing `[load] terrain per 1.0 s: … finished (X us each)`, the testkit
  `load:` summary (`server generated … us each`), and time to `pending 0`. On the PC, check a
  4-core case with `taskset -c 0-3`.

### 4. Thread priorities: none on the Mac, and too weak on Linux

- **What:** `lib.rs:125-146` `set_priority` only acts on Windows and Linux.
  - **On macOS it does nothing.** Every bulk pool runs at the same priority as the main thread and
    the global pool it waits on. On the M3 Max (`workers()` = 12) that is background 12, mesh 12,
    terrain light 12, LOD 4, unpack 4.
  - On Linux the bulk pools get nice +1. The -1 for the main thread and global pool fails without
    privilege. That is a CFS weight of 820 against 1024. With about 13 runnable bulk threads on a
    4-core, the main thread gets about 35% of its core.
  - The fixed-size pools ignore `workers()`: LOD min 3, unpack 4, urgent 4, skin 2. A 4-core has
    about 13 bulk threads plus main, terrain and server.
- **Evidence:**
  - Comments record main-thread waits on a busy Mac of "milliseconds" (`world.rs:187-191`) and
    "up to 15 ms" (`world_render.rs`, translucent sort).
  - A LOD tile's mesh takes 539 us idle (62.6 s) but 20-60 ms while loading, and face calls go
    from 2.8 us to 100-300 us each. Part of that may be blocking (finding 9), not only CPU.
- **Change:**
  - macOS: `libc::pthread_set_qos_class_self_np(QOS_CLASS_UTILITY, 0)` in `lower_priority`;
    `QOS_CLASS_USER_INTERACTIVE` for main and the global pool; `USER_INITIATED` for the server.
    Utility still uses P-cores when they are free.
  - Linux: nice +10 (or `SCHED_BATCH`) for bulk pools.
  - Size the fixed pools by `workers()`.
- **Gain:**
  - Mac: steadier load frames and shorter main-thread waits in `end_batch`, `start_jobs`,
    `take_tints` and culling. Perhaps 1-2 ms off the untimed part and a lower frame max (a guess).
  - Linux 4-core: main share ~35% to ~80% under contention, the biggest frame-stability win there.
  - Windows: already strict priorities.
- **Effort:** small.
- **Measure:** the same lodmac script before and after: frame max, cpu update, `finished (us each)`.
  samply per thread shows run-queue gaps on the main thread.

### 5. Lazy texture loading blocks the frame

- **What:** `world_render.rs:1254` (`upload`) and `:1137` (`upload_lod`) call
  `textures.ensure_blocks` → `load_requested` (`textures.rs:286-322`).
  - It reads and decodes PNGs on the global pool while the main thread waits, then creates GPU
    textures.
  - It runs once per mesh that needs a new texture, not once per frame.
  - Each load bumps `generation`, so `set_textures` (`world_render.rs:559`) rebuilds `tex_info` and
    `anim` over every texture id, and `flush_glow` clones and rewrites the whole glow map.
- **Evidence:** `[load] loaded N textures in X ms`:
  - second 2: 52 loads, 96.7 ms, max 15 ms;
  - second 5: 45 ms, one load of 31.7 ms;
  - 48 s (new place): 26 ms;
  - 104 s (new biome in flight): 36 ms, one load of 20.1 ms. That is the 69 ms frame.
- **Change:**
  - Collect requests for the frame and load once (`ensure_blocks` only marks; one
    `load_requested` at the end of `upload_meshes`).
  - Preload the dimension's surface blocks during world open.
  - Later, decode asynchronously and draw with the blank texture until it is ready.
- **Gain:** about 1 ms/frame on average in the first second, and 15-35 ms single-frame hitches gone
  at join and in new biomes. All platforms; on a weak 4-core the decode runs on 2 workers, so the
  hitch is longer.
- **Effort:** coalescing and preloading are small; async is medium.
- **Measure:** the existing `loaded N textures` lines; a counter of `load_requested` calls per frame.

### 6. Mesh uploads repeat work on the main thread that the mesher could do

- **What:** `world_render.rs:1233-1305` `upload`, per mesh:
  - collects every face's and quad's texture id and `sort_unstable` + `dedup`s it
    (O(F log F) for about 10 distinct ids);
  - recounts faces per direction (the mesher already sorted by direction, `mesher.rs:118-120`);
  - `mesh.faces.concat()` and `mesh.quads.concat()` (an allocation and a full copy, then
    `write_buffer` copies again into staging);
  - `mesh.quads[2].clone()`.
  - `upload_lod` (`:1118-1140`) sorts every quad of a tile the same way, with no budget.
- **Evidence:** this cost spends the upload budget. From 5 to 12 s, cubes made by 4.8 s were still
  reaching the screen (GPU `world` time climbing). There is no per-upload counter yet, so the
  per-mesh cost is an estimate: 30-100 us for a surface cube.
- **Change:**
  - `mesher::build` returns, with the mesh, its distinct texture ids, its per-direction counts and
    one contiguous record buffer with layer offsets.
  - `upload` writes that buffer directly and moves `quads[2]` instead of cloning it (take
    `MeshResult` by value).
  - Do the same in `lod::made` for tiles.
- **Gain:** maybe 1.5-2x more meshes per millisecond of budget, so a loaded area shows sooner
  (the 5-12 s tail could roughly halve). Per-frame cpu update is unchanged while work waits
  (budgeted). All platforms, most on a weak CPU.
- **Effort:** small.
- **Measure:** new `diag` Counters `UPLOADS`, `UPLOAD_NS` (main thread), `UPLOAD_RECORDS`; time from
  the last `put in` to GPU `world` going flat.

### 7. Biome tints are blended about 9 times per column while loading

- **What:** `world.rs:518-529`: every new column marks its 3x3 neighbourhood dirty. `take_tints`
  (`world.rs:727-741`, up to `TINTS_PER_FRAME` = 256 a frame, about 9 us each, `par_iter` on the
  global pool from the main thread) blends again as each neighbour arrives. `take_tints` also
  copies the whole dirty set to a `Vec` every frame.
- **Evidence:** code (the comment at `world.rs:184-186` says as much). Not measured: 256 x 9 us is
  about 2.3 ms of CPU a frame at worst. The wall time is short on 12 threads, about 1.2 ms with
  the main thread waiting on a 4-core.
- **Change:** blend a column once all 8 neighbour columns are present, or the view edge is reached.
  Until then keep the first blend, or leave it dirty and do no work. Iterate the dirty set without
  collecting.
- **Gain:** about 5-9x fewer blends. Mac small; weak 4-core about 1 ms/frame while loading.
- **Effort:** small.
- **Measure:** new Counters `TINT_BLENDS` and `TINT_NS`, against new columns per second.

### 8. The server rescans its whole backlog for every cube that arrives

- **What:** `server/load.rs:87-119` `receive_generated`:
  - drains the backlog, sorts it, builds a hash set of it, and pushes back everything whose cube
    above is still coming;
  - `load_until` (`:7-43`) calls it again after each single `recv_timeout`;
  - `request_generation` rebuilds a `queued` set of the backlog every 5 ms.
  - `insert_ns` (the 30 us/cube) times only `Dim::insert`, so this cost is invisible.
- **Evidence:** code. With a backlog of B (up to 768 on the Mac, waiting for the cube above), each
  arrival costs O(B log B), roughly 40-80 us at B=500. At 2.5k cubes/s that is 10-20% of the
  server thread. This is an estimate.
- **Change:** keep the backlog keyed by position, with a "waiting for the cube above" map. When a
  cube is inserted, release only the one below it. No re-sort.
- **Gain:** server-thread CPU. It reaches the frame only on a weak 4-core, where it frees part of a
  core during loading. Mac and PC: none visible.
- **Effort:** small to medium.
- **Measure:** new Counter `SERVER_RECEIVE_NS` (the whole function), next to `insert_ns`.

### 9. `ColumnCache` makes pool workers wait on each other

- **What:** `server/terrain.rs:1155-1173`: `get_or_init` blocks every thread asking for a column
  while one computes it. A blocked rayon worker steals nothing. The cubes of a column are spawned
  together, one job each:
  - `client/terrain.rs:834-841` (CPU path);
  - `server/load.rs:257` (server generation, column by column top-down);
  - LOD face calls also look up columns (`lod.rs:192`, `one_column`).
- **Evidence:** indirect only:
  - server generation at 3524 us per cube on a worker;
  - LOD face calls at 100-300 us each while loading against 2.8 us idle;
  - pool waits do not show as CPU in any counter.
- **Change:** one job per column that computes the column data once, then the cubes, either in the
  same job or as follow-up spawns. Or compute the shared column data first, before spawning the
  cube jobs.
- **Gain:** uncertain; perhaps 10-30% generation throughput on the Mac, more on 12+ threads.
  Weak 4-core: little (2 workers).
- **Effort:** medium.
- **Measure:** new Counter `COLUMN_WAIT_NS` (time in `get_or_init` when the cell was already there
  but empty) and `COLUMN_WAITS`.

### 10. Per-cell rescans of cubes near the camera, every frame while anything loads

- **What:**
  - `world_render.rs:1055-1113` `update_solid` reruns whenever `world.revision()` changes, which is
    every insert anywhere. That is 15³ = 3375 `cube_rev` hash lookups a frame, plus up to 32 cubes
    x 4096 `cube.get(i)` (a bit-unpack each).
  - `particles.rs:442-470` `update_cells` does 16 cubes x 4096 x (`get` + `light`) when stamps
    change.
- **Evidence:** code. Estimated 0.3-0.6 ms/frame while loading near the player.
- **Change:**
  - Decode once (`cube.decode` into a reused buffer), map through a per-palette table, and build
    the bit words from that.
  - For `update_solid`, rerun only when a cube within 7 changed: keep a "near revision" set in
    `insert`, `remove` and `set_block`.
- **Gain:** about 0.3-0.5 ms/frame while loading, all platforms.
- **Effort:** small.
- **Measure:** spans `update/solid` and `render/particle cells`.

### 11. The storage lock is held across disk read and decode in every generation job

- **What:**
  - `server/load.rs:273` takes `s.lock().load(dim, pos)`.
  - `saves.rs:588-614` does the region read, lz4 decompress and `Cube::read` under one `Mutex`,
    on every background worker.
  - Unload saves take the same lock (`load.rs:330`).
  - `send_far` reads up to 256 cubes a tick from disk on the server thread (`load.rs:461`).
- **Evidence:** not in this run (fresh world: `region_exists` is false, so it returns at once).
  On a revisited world every loading worker queues on one lock.
- **Change:** read the raw bytes under the lock (or keep a region handle per thread), and
  decompress and parse outside it. Move `send_far` reads to a worker.
- **Gain:** loading a saved world: up to about `workers()`x on the decode part. Fresh worlds: none.
  All platforms.
- **Effort:** small.
- **Measure:** `gen_ns` per cube on a revisited world; new Counter `STORAGE_LOCK_NS`.

## Checked, not the cost (in this run)

- **Entities, interpolation, `seat_riders`, `chip_block`** (`mod.rs:1636-1794`): O(entities) a
  frame. `seat_riders` makes 4 passes over riders; `Despawn` does `retain` over
  components x ids (`receive.rs:120`). These are small with no mobs (cpu update 0.07 ms settled).
  They could grow with crowds. `animate_entities` already goes parallel at 512 or more.
- **The mesher** (`mesher.rs`): already a padded 18³ array, bit rows for enclosed blocks, no hash
  lookups in the inner loop. The remaining cost per job is the two 23 KB boxes and the `extras`
  map in `neighborhood` (`world.rs:1019`), and `corner_data`'s per-corner `lets_into`. It runs on
  workers, not the frame. Turn on `--log mesh=debug` to get "us each on a worker" before touching it.
- **Light:** the terrain thread is 19-77% busy, never saturated. The server's `Dim::insert`
  (lighting included) is 30 us per cube. Neither limits loading here.
- **Locks between the terrain thread and the main thread:** none. They talk through channels
  (`terrain::Batch`, `BACKLOG` 8). The main thread's waits are on rayon pools (findings 2 and 4),
  not on the terrain thread.
- **The meshing in-flight cap** (6144 on the Mac, `mod.rs:1822`): likely deeper than needed (about
  12 frames of work queued), which can waste meshes on cubes that change meanwhile. This is a
  hypothesis: add a Counter `MESH_STALE` for results dropped in `take_meshes` (`world.rs:958`)
  before changing it.
- **Context, not in this area:**
  - The fly_creative bench shows `base 310 ms/52984 calls` of mod callback time per tick. That is
    about 6 cores busy in worldgen callbacks while flying (Terrain session).
  - MoltenVK `queue` (submit) is 4-6 ms of every loading frame.
