# Profiling the server's tick (CPU sampler)

Where an integrated server's tick goes, system by system, without touching the GPU: macOS's
`/usr/bin/sample` reads call stacks once a millisecond. Testers run it (never launch the game
yourself); nothing here needs a debug build: release keeps line tables.

1. Start the game with a script whose windows are marked `rec <label> start` / `rec <label> end`
   (`mods/testkit/scripts/profile_busy.txt`, `profile_world.txt`), and right after it:

   ```
   tools/profile/sample_windows.sh <run dir> <out prefix> stand walk
   ```

   It waits for each `REC <label> start` in the run's newest log and samples `weft` for 28 s
   into `<out prefix>-<label>.sample.txt`.
2. Read one:

   ```
   python3 tools/profile/sample_top.py <out prefix>-stand.sample.txt
   ```

   It sums the `server` thread and its pool (`server N`): samples busy and waiting (blocked, or the
   pool looking for work), CPU ms per tick, then the top functions by self and inclusive samples.
   `--threads all` reads every thread (the client's too).
3. Put it next to the run's BENCH line: `server avg/max ms` is the tick's wall time, split into
   `insert, messages, regions, request, sync, rest`; `mods per tick` is each mod's callback time.

The two scripts:
- `profile_world.txt`: a copy of Ronald's world (`dev/run/saves/New_World`, copied into the run
  dir's `saves/`, never the original), standing still and walking, 30 s each.
- `profile_busy.txt`: a fresh world (`--seed 1`) with 120 animals of 12 species around the
  player and a wheat field, the same two windows.
