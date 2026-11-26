# Tester tools (Mac game runs)

The scripts the Tester uses to run a game on the Mac one at a time, safely. They live in the repo so a cleanup of
`~/Coding/chiacraft-archive` cannot take them again (it did, before 11-22). Logs, the lock, the `PAUSED` and
`GPU_STALLED` flags and the `exp/` build folders stay in the archive: `~/Coding/chiacraft-archive/tester-tools/`
(`TESTER_RUNS` and `TESTER_EXP` override). Shots go to `~/Coding/chiacraft-archive/images/tester/<date>/`.

| File | What it does |
|---|---|
| `guard.sh` | sourced by the runners: refuses when paused (`runs/PAUSED`), after a GPU stall on this boot (`runs/GPU_STALLED`, first line = boot time), at 150 or more IOSurface clients, when the lock is held or a weft game runs |
| `gamerun.sh <dir> <script> [world] [timeout]` | one game, with the log sampled by `stuck.sh` when it goes quiet; archives shots and log after |
| `gamemulti.sh <dir> <bin> <timeout> <label> -- args1 -- args2` | several copies at once, same guards and exit codes |
| `stuck.sh <pid> <out>` | samples the main thread for 3 s: GPU / LOCK / ALIVE / UNKNOWN / GONE |
| `archive.sh <dir> <marker> <label> [logs]` | copies new shots and logs into the archive; never deletes |
| `build.sh name:commit ...` | `git archive` a commit into `exp/<name>` and build it, niced |
| `test-runners.sh` | checks every exit status with a fake game (no real launch, no GPU, about 2 minutes) |

## Exit status (gamerun.sh, gamemulti.sh)

| Code | Meaning | `GPU_STALLED` written? |
|---|---|---|
| 0 | the game(s) ended on their own | no |
| 2, 3, 4, 7 | refused by `guard.sh`: lock or a game running (2), a GPU stall on this boot (3), paused or no binary (4), too many graphics clients (7) | no |
| 5 | GPU stall: killed. Also a timeout whose log had gone quiet, or a game that would not die after SIGKILL | yes |
| 6 | lock hang (main thread on a lock): killed, a code bug, not the GPU | no |
| 8 | plain TIMEOUT: still running after the timeout, ended on SIGTERM, log still moving | no |
| 9 | STUCK: log quiet for two stall windows while the main thread kept running frames: killed | no (yes if it would not die: then 5) |

**A chain must stop on any non-zero status** (`./gamerun.sh ... || exit $?`, or `|| break`). On 11-07 a timeout exited 0, the
chain went on, and eleven more games started into a wedged GPU. Never `wait` for a game without a bound: a process
stuck in the GPU driver ignores SIGKILL (that one took 52 minutes to go), so the runners wait at most 10 s after
SIGTERM and 30 s after SIGKILL, then flag the stall.

Quiet-log rule: no new log line for `STALL` seconds (60, gamemulti 90) means `stuck.sh` samples the game. ALIVE
(frames turning) gets one more window; a second quiet window is STUCK (exit 9). GPU or UNKNOWN kills at once and
flags. `QUIETMAX` (default 30 s) is how quiet a log may be at a timeout before it counts as a stall.

Rules that stay: one game at a time on the Mac, only with Ronald's yes (Planner's relays count), never with the
argent tool-server running, never after `GPU_STALLED` on this boot (see the memory `gpu-hang-kills-the-mac`).
