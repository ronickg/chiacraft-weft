# Testing the game: what testkit has, what others do, what to add

Tester, 2026-11-03. How other games test (Factorio, OpenTTD, Minecraft GameTest, Veloren, Luanti,
Riot, Rare, Bevy, wgpu, Mesa, Godot), set against testkit, and against what went wrong in today's
runs. Sources are linked inline.

## Where testkit stands

testkit is already a strong base, stronger than most open-source voxel games have:

- **209 in-game tests** in Rust (`mods/testkit/src/tests/`), each in a fresh throwaway world, with
  `check`, `check_eq`, `probe`, waits on conditions (`until`, `loaded`, `idle`, `screen`, `settled`),
  pixel probes (`pixel`, `avg`, `share`, `differ`), block and inventory sync checks across two clients,
  and a report file whose exit code counts failures.
- **Parallel runs**: `--jobs N` / `--test-shard i/n`, with an `ALONE` list for timing-sensitive tests.
- **Benchmarks** by name (`--test bench`: redstone, fly_load, load_benchmark, item_farm,
  particle_crowd, crater) that print `BENCH` lines.
- **Line scripts** (`--script`): input, chat commands, waypoints, shots, `host`/`join` for two copies.
- **Same conditions** for every test (`same_conditions`: language, GUI scale, view distance, packs off).
- **Fuzzing** exists for package decoding (`crates/packages/fuzz`, cargo-fuzz).
- **CI** (branch `claude/funny-grothendieck-5aff36`, PR 1, not yet merged) builds, runs clippy and
  the unit tests, and runs `weft --test all` under xvfb with a software Vulkan.

Gaps:

- **Scripts can't assert.** A script PASSes if it reaches `quit`.
- **Scripts wait by the clock.** `wait N` is the only wait in scripts.
- **No pictures are compared** ("checks never compare pictures with references").
- **No guard for a hung client.**
- **No save/load or desync checks.**
- **No perf budget.**

## What today's runs taught

Every problem found today in other sessions' runs traces back to one of these gaps:

| what happened | cost | the gap |
|---|---|---|
| Water check "reflections off = navy screen" | 7 runs and a wrong diagnosis. The player had fallen into the lake while the area loaded, and the second `/tp` didn't take. | Fixed waits. Nothing asserted where the player stood before the shot. |
| Reflections on vs off: identical mean colours, <1% pixels differ | Only noticed because the numbers matched to the integer | No on/off comparison step. Nothing said "these two must differ here". |
| Clans pair: the invite reached nobody | A full two-copy run. The host invited at 62 s; the guest had quit at ~50 s. | Fixed waits in two scripts that must meet. No "wait until player bob is here". |
| Clans pair rerun: the guest froze with no error | Both copies killed by the outer guard after 90 s; Mac runs paused | No in-game watchdog that dumps where the main thread is stuck. |
| Winter-night playtest died at 60 s with SURFACE_LOST | One run | The PC wrapper's compositor timeout was shorter than the run. |
| Every PC run exit 1 | Two runs | A stale mod zip with an old ABI in `run/mods`. Nothing checks for that before starting. |

## What others do (the parts that fit us)

- **Wait on conditions, never on seconds.** Riot's test system for League of Legends polls the game
  until a condition holds, with a timeout; plain sleeps aren't allowed, and new tests sit in a
  staging set for a week before they can fail anyone else's build
  ([Riot](https://www.riotgames.com/en/news/automated-testing-league-legends)). Fabric's client tests
  use `waitFor(predicate, timeout)`, `waitForScreen`, and clicks by translation key, not by pixel
  ([Fabric](https://docs.fabricmc.net/develop/automatic-testing)).
- **Fixtures in isolated plots** (Minecraft GameTest). A test names a saved structure. The framework
  pastes it into its own plot with a test environment (rules, time, weather), runs it with a
  `max_ticks` limit, and supports `max_attempts`/`required_successes` for flaky cases and
  `/test verify` to run every rotation
  ([1.21.5 notes](https://www.minecraft.net/article/minecraft-java-edition-1-21-5)). Bedrock runs
  tagged sets (`/gametest runset`) and advises Normal difficulty, because mobs behave differently on
  Peaceful ([Microsoft](https://learn.microsoft.com/en-us/minecraft/creator/documents/gametestgettingstarted)).
- **Screenshot comparison with a tolerance.** Fabric has `assertScreenshotEquals` (fuzzy) and
  `assertScreenshotContains` (finds a template inside the frame). wgpu compares with NVIDIA's FLIP
  metric and writes an error map on failure
  ([wgpu_test](https://wgpu.rs/doc/src/wgpu_test/image.rs.html)). Bevy runs examples with a fixed
  frame time and sends the screenshots to a service that diffs them against main
  ([bevy_ci_testing](https://docs.rs/crate/bevy_dev_tools/latest/source/src/ci_testing/config.rs)).
- **Known failures in a file** (Mesa). Mesa keeps `fails.txt`, `flakes.txt` and `skips.txt` per
  driver. CI fails only on surprises, including a listed failure that now passes
  ([deqp-runner](https://gitlab.freedesktop.org/mesa/deqp-runner/-/merge_requests/74)).
- **Two processes, marker files, merged logs** (Luanti). `test_multiplayer.sh` starts a server and
  a headless client and waits for `startup`/`done` files. It fails on a `test_failure` file or a
  crash signal, and merges the logs with `(server)`/`(client)` prefixes
  ([Luanti](https://github.com/luanti-org/luanti/blob/master/util/test_multiplayer.sh)).
- **A software-rendered smoke test with every effect on** (Luanti). It runs a tiny window at 5 fps
  under xvfb with `LIBGL_ALWAYS_SOFTWARE`, GL debug on, and bloom, shadows, volumetrics and
  reflections all enabled, with a 25 s limit, to "catch shader errors and other obvious bugs"
  ([Luanti](https://github.com/luanti-org/luanti/blob/master/util/test_singleplayer.sh)).
- **Error paths must exit cleanly** (Luanti). Each scripted mod error must exit with code 1 within
  10 s. The sanitizer is given its own exit code so the two can't be confused
  ([Luanti](https://github.com/luanti-org/luanti/blob/master/util/test_error_cases.sh)).
- **Checksums each tick and replays** (Factorio heavy mode). It CRCs the map every tick and saves a
  readable map each tick. On replay it stops at the first tick that differs, so you diff two saves
  ([FFF 47](https://direct.factorio.com/blog/post/fff-47)). OpenTTD's `desync=2` recomputes every
  cache each tick and logs any mismatch
  ([OpenTTD](https://github.com/OpenTTD/OpenTTD/blob/master/docs/desync.md)).
- **Physics independent of step size** (Veloren). The same fall or walk is run as 20 small steps
  and as 2 steps ten times larger, and the results must agree
  ([Veloren](https://gitlab.com/veloren/veloren/-/blob/master/common/systems/tests/phys/basic.rs)).
- **A bot swarm** (Veloren). `swarm N` connects N headless clients that move around, to load the
  server ([swarm.rs](https://gitlab.com/veloren/veloren/-/blob/master/client/src/bin/swarm.rs)).
- **Benchmarks with history.** Factorio's `--benchmark save --benchmark-ticks N` prints per-tick
  timings ([wiki](https://wiki.factorio.com/Command_line_parameters)). Veloren loads Criterion CSVs
  into Postgres. Godot runs daily on one fixed machine
  ([Godot](https://contributing.godotengine.org/en/latest/other/benchmarks.html)). For noisy CI,
  count instructions instead of time (gungraun, formerly iai-callgrind).
- **Every fixed bug gets a test** (Factorio, [FFF 366](https://factorio.com/blog/post/fff-366)).
  Factorio also orders tests by dependency, so the first failure shown is the root cause.
- **A bot that plays the whole game** (Croteam, The Talos Principle). It ran without rendering at
  high speed after every level change, filed bugs through the in-game reporter and cheated past them
  ([PlayStation Blog](https://blog.playstation.com/archive/2015/09/25/how-croteam-built-a-bot-to-beat-its-ps4-puzzler-the-talos-principle/)).

## What testkit should gain, in order

1. **Script assertions and condition waits.** This is the cheapest change and would have saved most
   of today's reruns.
   - Add to `script.rs`:
     - `expect_block x y z name`
     - `expect_pos x y z r`
     - `expect_on_ground`
     - `expect_chat <text>`: a chat line containing the text arrived since the last mark
     - `expect_log <text>`
     - `expect_attr <attr> <op> <value>`
     - `expect_health <op> <value>`
   - Add `until <one of the above> [timeout]` as the wait form, plus `until_player <name>` for pair
     scripts.
   - Any failed `expect` fails the test, and the report shows the line. The `T` helpers already
     exist (`until`, `block`, `state`, `check`); the script only needs words for them.
   - Then retire `wait N` before shots: `until settled` plus `expect_pos`.
   - **Built** (branch `tester-testkit`, 2026-11-03), as two words over one set of conditions:
     - `expect <condition>` checks now. A miss fails the run, naming the line and what was there
       instead, and the script goes on.
     - `until <condition> [within N]` waits, 30 s by default, and stops the run if the condition
       isn't met in time.
     - Conditions:
       - `block x y z name`, with `~`/`~n` off the feet
       - `near x y z [r]`
       - `on_ground`, `alive`, `dead`, `flying`
       - `mode survival|creative|spectator`
       - `health <op> n`, `fps <op> n`
       - `chat <text>`
       - `screen <name|none>`
       - `players n`
     - Not built: log lines and attributes.
2. **An in-game hang watchdog.** A thread that checks a per-frame heartbeat. If the main thread
   misses frames for 15 s, it logs a backtrace of every thread, the last 20 log lines and the GPU
   submit state, then exits with a dedicated code (e.g. 70). Today's frozen guest left nothing to go
   on. On the Mac this matters most, because a run killed from outside never says why.
   - **Built**: `--watchdog <secs>`, off without it. It writes the step the run was on, plus each
     thread's state and kernel wait on Linux, to stderr and `test-reports/watchdog.txt`, then exits
     with code 70. Full backtraces of other threads would need a platform crate, so it doesn't
     print them yet.
3. **Paired-run markers.** Give `host`/`join` an `expect_peer` step. Write
   `run/test-reports/<name>.done` or `.failed` files. Add a small `testkit pair` runner (or extend
   `--jobs`) that starts both copies, merges their logs with `[host]`/`[guest]` prefixes, and fails
   if either fails or goes quiet. That is the Luanti pattern; today it is a shell script in
   Tester's scratchpad (`gamepair.sh`).
4. **On/off and golden image comparison.**
   - `shot name` gains `compare name <golden|other-shot> [max% at >8/255]`, and `differ name a b
     <min%>` for "these must differ" (the reflections case). It writes a diff image next to the shot.
   - Keep goldens per backend (Vulkan/Linux, Metal/Mac), refreshed by hand with
     `--test-bless`.
   - A list of known differences, in the shape of Mesa's `fails.txt`, keeps CI green while a look is
     still being worked on.
   - This turns the shader pack's checklist (`mods/base/shaderpack/checks/`) from "Tester looks at
     grids" into checks that fail.
5. **The everything-on shader smoke test in CI.** CI already has software Vulkan under xvfb. Add a
   job that runs one scene with `--shader-pack chiacraft`, every screen effect on, Vulkan validation
   on (`--gpu-debug`) and a 60 s limit, failing on any validation error, `ERROR` line or NaN pixel.
   It catches shader and binding bugs before they reach the Mac's GPU.
6. **A preflight check before every run.** Before the world opens:
   - refuse a mod in `run/mods` whose ABI differs (with the file name and a hint)
   - refuse a `--script` path that doesn't exist
   - print the seed and the git hash
   Today's stale `testmod.zip` cost two runs.
   - **Built in part**: the whole script is read before its first step. Unknown commands, missing
     arguments and unreadable conditions fail the run at once, listing every bad line. Every failure
     names its line.
   - Already covered: the engine refuses a mod with the wrong ABI (it exits 1 and names the mod),
     and a missing script exits 2.
   - Still to do: print the seed and the git hash.
7. **Save/load round trip and cache checks.** A `roundtrip` test step, or `--heavy`, does this every
   N ticks:
   - Hash the world (blocks, entities, vitals, animal state), save, reload, and hash again.
   - Recompute light and heightmaps for loaded chunks and compare them with the caches.

   Survival state (stomach, freshness, temperature, clothing) and animal state (tame, hunger, eggs)
   are the likeliest to be lost on save.
8. **Perf budgets.**
   - Make `BENCH` lines machine-readable: `BENCH label p50 p95 p99 ms`.
   - Run the existing benchmarks nightly on the Linux PC (one fixed machine) and append to a CSV in
     `run/bench/`.
   - Fail when p95 drops more than 15% below the median of the last 10 runs.
   - Hot paths (meshing, worldgen, light) get instruction-count benches in CI.
9. **Long runs and bots.**
   - A `soak` script on the PC: survival, 3 in-game days, random walking, eating when hungry, fps and
     memory logged every minute.
   - Later, a `swarm N` of headless clients for the server (phase 8).
   - Fails: memory growth over 10%, any frame over 250 ms, any ERROR line.
10. **Habits.**
    - Every bug Tester reports gets a script in `mods/testkit/scripts/bugs/`.
    - New tests sit in an `unstable` tag for a week before CI fails on them.
    - The report also comes out as JUnit XML, so CI can show it.
    - Fuzz the network packet decoder, alongside the package fuzzing that already exists.

## Start with

1, 2 and 6 are small, live entirely in testkit, and would have saved most of today's wasted runs.
4 and 5 turn the lighting work's "look at the grid" into checks that fail. 7 guards the survival
and animal work landing now.
