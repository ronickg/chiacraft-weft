# Branch triage, 2026-11-22

Ronald, 11-22: "lets get our stuff all merged up to date". Written by the triage session for Planner. Read only: no branch was deleted, renamed or merged. `dev` was `1d0a8e75c` when this was made (223 local branches have commits that are not in `dev`; this branch is the 224th and is left out).

| bucket | branches |
|---|---|
| Ready candidates | 6 |
| Waiting on a run or a call | 31 |
| In already | 73 |
| Test only | 109 |
| Dropped by the Weft sync | 4 |
| **total** | **223** |

## Planner: merge now (no engine files)

Only notes and reports are ready; nothing that changes the game is "ready" on its own. All six merge into `dev` without conflict (checked with `git merge-tree`).

- `claude/rtx-mod-study-5939fb` `55536da5`: Report only (reports/NVIDIA RTX mod study.md); "RTX study done", session closed 11-06
- `claude/mc-shader-compare-0c5152` `91cef20d`: Reports only (reports/Minecraft shaders compared.md, mc-shader-techniques.md); session closed 11-06
- `look-valheim` `def91d4a`: Report only (reports/valheim-look.md)
- `perf-review` `5a9dd6a1`: Report addendum (reports/Perf review.md, 11 lines)
- `claude/art-regen-test-98c660` `abb9d84e`: Notes only: redo-v3 job lists, scripts and sheets (29 files, 14k lines, all new paths); art v3 itself is in dev. Optional: only if Ronald wants the job lists kept in the repo
- `sync/weft-26.11.2` `32f9df6f`: One-line sync log note (notes only); optional

Not yet, but nothing else to do after the run or the call (no engine files, and they merge into `dev` without conflict):

- `claude/look-hill-risers` `ff697fa2`: Waits on Ronald: HDR world with no far_soft (hill risers like Bliss); his 11-06 far-look pick turned far_soft on (Ronald (call))
- `claude/far-sharpen` `0a3fe8d9`: Waits on a PC A/B: far-sharpen checker before/after (TAA background job queued, NEXT_SESSION) (PC A/B)
- `claude/sky-horizon-haze` `692ad3be`: Waits on a PC run (shhz0/1 on dev fd687748) (PC run)
- `claude/sky-predawn-teal` `7451851e`: Waits on a PC run (blue-hour teal, "run pending") (PC run)
- `claude/sky-bodies` `8d67e7f0`: Waits on a PC run (shbd2: sun and moon bodies at Bliss's size) (PC run)
- `claude/sky-stars-fade` `c9fa6b8d`: Waits on a PC run (stars after sun -3 degrees; built 11-06, no result recorded) (PC run)
- `claude/sky-dawn-pink` `e37be2f4`: Waits on a PC run (shdp2: dawn pink) (PC run)
- `claude/look-twilight2` `df81e01b`: Held for a stage-2 check run (board 11-06): 19:00 land 63 vs Bliss 86 (PC run)
- `claude/rtx-showcase-blocks-272636` `a1158562`: RTX showcase notes: "merge with its next code" (next PT milestone) (Next PT milestone)
- `claude/easy-portals-d5e7c8` `d0056356`: Design note, linking at the portal: options A-C, B recommended; Ronald chooses. The session is working now (Ronald (call))
- `claude/animals-wave-two-0005d7` `cf11c312`: WIP: ambient life numbers and sprites, "not wired in yet" (Animals)
- `claude/post-sync-audit` `d2c78566`: Video screen defaults in Russian: not in dev; likely covered by the later UI audit (479f72802) and Russian pass (c902d251c): check, then drop (Planner call)

No engine files but the merge conflicts with `dev` (the owner rebases first): `claude/far-shadow`, `claude/sky-cloud-layer2`, `claude/fossils-phase-1-d14597`, `claude/tests-after-32`.

## After the Weft sync (they touch `crates/` or `backends/`)

Wait for `claude/upstream-fixes-1122-e85f21`, then rebase on the new `dev`. Each also waits on the run or call shown.

- `claude/mist-bliss-dawn-2f90b6` `a379d767`, 10 engine files: Waits on its final PC benches ("merges after its final benches", m16 on dev 45d49dea) (PC bench)
- `claude/rt-cone-mip` `557bd3b0`, 1 engine file: Waits on the PC A/B pairs (pool speckle fix, adds engine rt_hit_cone; ENGINE_CHANGES line in) (PC A/B)
- `claude/animate-time` `cdad0519`, 1 engine file: Waits on a proof run (--animate-time v2, "didn't separate cleanly") (PC run)
- `claude/pt-denoisers-13c78d` `1df5df64`, 6 engine files: Waits on a cost run at 1440p and 4K (pt_dn_grow default 1: blotches -8%) (PC run (cost))
- `claude/rt-path-tracing` `40b341fa`, 7 engine files: Not ready (board 11-06 22:05: M2 light-pick fix, test scene queued) (PC run)
- `claude/server-scale` `2913fafa`, 7 engine files: Parked on Ronald's word: "Lets pake the srvrer stuff for now and do it later" (11-05) (Ronald (call))
- `claude/phase3-cleanup-519f6e` `73d41973`, 36 engine files: Phase 3 leftovers (links follow moves, arrivals in any dimension, three-copy check, crates/store, outbox lanes) plus letters through portals and the mailbox: NOT in dev (board 11-04: "waits on an owner"); needs Tester 2 runs and a Weft-sync rebase, and overlaps easy-portals (Needs an owner (Phase 3 session gone))
- `claude/phase3-leftovers-519f6e` `98fffa5c`, 15 engine files: Ancestor of phase3-cleanup (same work, 2 commits fewer): merge the cleanup branch, not this (Needs an owner (Phase 3 session gone))
- `claude/phase5-atlas-wip` `81a0413c`, 6 engine files: WIP, unbuilt and "on hold": network atlas status v2 plan and first engine parts (Phase 5) (No owner (on hold))
- `claude/plurals-519f6e` `9c5ce2cc`, 2 engine files: WIP plurals tag in the text formatter: "a small engine item; owner when one is free" (board, parked) (No owner)
- `claude/behind-copy` `bc212cab`, 2 engine files: Waits on a Mac pair ("behind-copy waits on Tester's Mac pair; merge as is if no room", board 11-04); no result recorded (Mac run (Ronald's yes))
- `claude/air-biome-high` `b4474070`, 3 engine files: Real client fix not in dev (a cube without biome data takes the first cube's biome); no merge call recorded: a Weft bug, note only in ENGINE_CHANGES (Planner call)
- `claude/lod-fixes` `29d49d97`, 1 engine file: Real LOD fix not in dev (no air canopies, the magenta boxes were boulders); no merge call recorded (Planner call)
- `fix/tints-once` `618979ef`, 1 engine file: Biome tints blend once the 8 neighbours are in: not in dev; the join-stall it targeted was mostly the GPU fence (board 11-05 22:35); drop or keep is the Upstream call (Planner call)

The sync branch itself, `claude/upstream-fixes-1122-e85f21` (tip `3ca22600`), is listed in the waiting table: it is not a candidate to merge now.

Dropped by the sync: the four Android branches. The `in already` and `test only` branches that touch the engine need nothing from the sync; they are what Ronald may delete (his yes needed).

## How each branch was checked

- `git rev-list --count dev..<b>` > 0 picks the branches; `git cherry dev <b>` lists the commits with no patch twin in `dev` ("own commits"). A branch with none is in `dev` already; a branch whose own commits are all test commits is test only.
- `git merge-tree --write-tree dev <b>`: 21 branches change nothing when merged; the "merge" column says clean or conflict for those that are not.
- For each own commit: its subject searched in the `dev` log, its added lines searched in `dev`'s version of the file (so a cherry-pick with a new hash still shows), and `dev` merge messages (they name the source branch and hash).
- Board and handoff: `notes/crew/BOARD.md` and `NEXT_SESSION.md` in the main checkout (the newest entries are 11-07 02:40); `notes/ENGINE_CHANGES.md` and `notes/crew/move-out-plan.md` for the sync.
- "Engine" is yes when an own commit touches `crates/` or `backends/`; the number is the files. "Files" is the number of files in the own commits. A branch with no own commits shows 0 and engine no.
- Not run: the game, any test, any build.

Weakest calls (check these first): `claude/air-biome-high`, `claude/lod-fixes`, `fix/tints-once`, `claude/tests-after-32`, `claude/post-sync-audit` (real changes not found in `dev`, with no board decision); the "in already" calls for zero-own-commit branches rest on patch ids only.

## Ready candidates (6)

| branch | tip | date | files | engine | merge | reason | owner |
|---|---|---|---|---|---|---|---|
| `claude/art-regen-test-98c660` | `abb9d84e` | 2026-11-06 | 38 | no | clean | Notes only: redo-v3 job lists, scripts and sheets (29 files, 14k lines, all new paths); art v3 itself is in dev. Optional: only if Ronald wants the job lists kept in the repo | Art test (local_d1c79691) |
| `claude/mc-shader-compare-0c5152` | `91cef20d` | 2026-11-06 | 4 | no | clean | Reports only (reports/Minecraft shaders compared.md, mc-shader-techniques.md); session closed 11-06 | MC shader compare (closed) |
| `claude/rtx-mod-study-5939fb` | `55536da5` | 2026-11-06 | 4 | no | clean | Report only (reports/NVIDIA RTX mod study.md); "RTX study done", session closed 11-06 | RTX study (closed) |
| `look-valheim` | `def91d4a` | 2026-11-04 | 1 | no | clean | Report only (reports/valheim-look.md) | Look |
| `perf-review` | `5a9dd6a1` | 2026-11-05 | 2 | no | clean | Report addendum (reports/Perf review.md, 11 lines) | Upstream |
| `sync/weft-26.11.2` | `32f9df6f` | 2026-11-04 | 1 | no | clean | One-line sync log note (notes only); optional | Upstream |

## Waiting on a run or a call (31)

| branch | tip | date | files | engine | merge | reason | owner |
|---|---|---|---|---|---|---|---|
| `claude/air-biome-high` | `b4474070` | 2026-11-04 | 4 | yes (3) | conflict | Real client fix not in dev (a cube without biome data takes the first cube's biome); no merge call recorded: a Weft bug, note only in ENGINE_CHANGES | Planner call |
| `claude/animals-wave-two-0005d7` | `cf11c312` | 2026-11-04 | 7 | no | clean | WIP: ambient life numbers and sprites, "not wired in yet" | Animals |
| `claude/animate-time` | `cdad0519` | 2026-11-06 | 2 | yes (1) | clean | Waits on a proof run (--animate-time v2, "didn't separate cleanly") | PC run |
| `claude/behind-copy` | `bc212cab` | 2026-11-04 | 2 | yes (2) | clean | Waits on a Mac pair ("behind-copy waits on Tester's Mac pair; merge as is if no room", board 11-04); no result recorded | Mac run (Ronald's yes) |
| `claude/easy-portals-d5e7c8` | `d0056356` | 2026-11-22 | 1 | no | clean | Design note, linking at the portal: options A-C, B recommended; Ronald chooses. The session is working now | Ronald (call) |
| `claude/far-shadow` | `761d9e10` | 2026-11-06 | 6 | no | conflict | Waits on an A/B: far sun shadow built, PC A/B pending (NEXT_SESSION) | PC A/B |
| `claude/far-sharpen` | `0a3fe8d9` | 2026-11-06 | 1 | no | clean | Waits on a PC A/B: far-sharpen checker before/after (TAA background job queued, NEXT_SESSION) | PC A/B |
| `claude/fossils-phase-1-d14597` | `3f5c4980` | 2026-11-06 | 87 | no | conflict | Paused clean at 3f5c4980 (WIP): on resume merge dev, all-targets, sheets, then ask Ronald batch 1 and 4 decisions; in-game features on hold | Ronald (go) |
| `claude/lod-fixes` | `29d49d97` | 2026-11-04 | 1 | yes (1) | conflict | Real LOD fix not in dev (no air canopies, the magenta boxes were boulders); no merge call recorded | Planner call |
| `claude/look-hill-risers` | `ff697fa2` | 2026-11-07 | 1 | no | clean | Waits on Ronald: HDR world with no far_soft (hill risers like Bliss); his 11-06 far-look pick turned far_soft on | Ronald (call) |
| `claude/look-twilight2` | `df81e01b` | 2026-11-06 | 1 | no | clean | Held for a stage-2 check run (board 11-06): 19:00 land 63 vs Bliss 86 | PC run |
| `claude/mist-bliss-dawn-2f90b6` | `a379d767` | 2026-11-06 | 15 | yes (10) | clean | Waits on its final PC benches ("merges after its final benches", m16 on dev 45d49dea) | PC bench |
| `claude/phase3-cleanup-519f6e` | `73d41973` | 2026-11-04 | 59 | yes (36) | conflict | Phase 3 leftovers (links follow moves, arrivals in any dimension, three-copy check, crates/store, outbox lanes) plus letters through portals and the mailbox: NOT in dev (board 11-04: "waits on an owner"); needs Tester 2 runs and a Weft-sync rebase, and overlaps easy-portals | Needs an owner (Phase 3 session gone) |
| `claude/phase3-leftovers-519f6e` | `98fffa5c` | 2026-11-04 | 38 | yes (15) | conflict | Ancestor of phase3-cleanup (same work, 2 commits fewer): merge the cleanup branch, not this | Needs an owner (Phase 3 session gone) |
| `claude/phase5-atlas-wip` | `81a0413c` | 2026-11-04 | 6 | yes (6) | conflict | WIP, unbuilt and "on hold": network atlas status v2 plan and first engine parts (Phase 5) | No owner (on hold) |
| `claude/plurals-519f6e` | `9c5ce2cc` | 2026-11-04 | 2 | yes (2) | clean | WIP plurals tag in the text formatter: "a small engine item; owner when one is free" (board, parked) | No owner |
| `claude/post-sync-audit` | `d2c78566` | 2026-11-04 | 1 | no | clean | Video screen defaults in Russian: not in dev; likely covered by the later UI audit (479f72802) and Russian pass (c902d251c): check, then drop | Planner call |
| `claude/pt-denoisers-13c78d` | `1df5df64` | 2026-11-06 | 7 | yes (6) | clean | Waits on a cost run at 1440p and 4K (pt_dn_grow default 1: blotches -8%) | PC run (cost) |
| `claude/rt-cone-mip` | `557bd3b0` | 2026-11-06 | 3 | yes (1) | clean | Waits on the PC A/B pairs (pool speckle fix, adds engine rt_hit_cone; ENGINE_CHANGES line in) | PC A/B |
| `claude/rt-path-tracing` | `40b341fa` | 2026-11-06 | 8 | yes (7) | conflict | Not ready (board 11-06 22:05: M2 light-pick fix, test scene queued) | PC run |
| `claude/rtx-showcase-blocks-272636` | `a1158562` | 2026-11-06 | 1 | no | clean | RTX showcase notes: "merge with its next code" (next PT milestone) | Next PT milestone |
| `claude/server-scale` | `2913fafa` | 2026-11-05 | 9 | yes (7) | conflict | Parked on Ronald's word: "Lets pake the srvrer stuff for now and do it later" (11-05) | Ronald (call) |
| `claude/sky-bodies` | `8d67e7f0` | 2026-11-06 | 1 | no | clean | Waits on a PC run (shbd2: sun and moon bodies at Bliss's size) | PC run |
| `claude/sky-cloud-layer2` | `cc52112f` | 2026-11-06 | 10 | no | conflict | Waits on a cost bench and a look: Bliss's second cloud layer as a costed Max/Ultra row (NEXT_SESSION); no result on the board | PC run (cost) + look |
| `claude/sky-dawn-pink` | `e37be2f4` | 2026-11-06 | 2 | no | clean | Waits on a PC run (shdp2: dawn pink) | PC run |
| `claude/sky-horizon-haze` | `692ad3be` | 2026-11-06 | 1 | no | clean | Waits on a PC run (shhz0/1 on dev fd687748) | PC run |
| `claude/sky-predawn-teal` | `7451851e` | 2026-11-06 | 2 | no | clean | Waits on a PC run (blue-hour teal, "run pending") | PC run |
| `claude/sky-stars-fade` | `c9fa6b8d` | 2026-11-06 | 1 | no | clean | Waits on a PC run (stars after sun -3 degrees; built 11-06, no result recorded) | PC run |
| `claude/tests-after-32` | `d7059b29` | 2026-11-04 | 4 | no | conflict | Testkit test fixes after the 32 px skin (undress before looking at the skin, etc.); not in dev, conflicts with dev: owner rebases or drops | Planner call |
| `claude/upstream-fixes-1122-e85f21` | `3ca22600` | 2026-11-22 | 13 | yes (9) | clean | The Weft sync itself (hand ports of upstream fixes, 11 so far; PR #98): not a candidate; merges when its owner says | Upstream-port (local_9d0a754b) |
| `fix/tints-once` | `618979ef` | 2026-11-05 | 2 | yes (1) | conflict | Biome tints blend once the 8 neighbours are in: not in dev; the join-stall it targeted was mostly the GPU fence (board 11-05 22:35); drop or keep is the Upstream call | Planner call |

## In already (73)

| branch | tip | date | files | engine | reason | owner |
|---|---|---|---|---|---|---|
| `art/look-foliage-tints` | `61256d89` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `claude/art-branches` | `0b50645f` | 2026-11-04 | 11 | no | Branch bark was ported into dev with the branch blocks (14b9dedfd: "Art's young oak bark") | Art |
| `claude/art-tropical` | `6457dfb4` | 2026-11-04 | 17 | no | Tropical wood pass: dev has it (be8b2febb cites 6457dfb4) | Art |
| `claude/ci-workflow` | `41cf1452` | 2026-11-05 | 8 | yes (1) | PR #1 CI workflow: 95% of its lines are in dev; board: merged 11-03 | CI |
| `claude/compassionate-booth-38c571` | `1863ab02` | 2026-11-04 | 6 | no | Release builds workflow: dev has it (same subject, 97% of the lines) | CI |
| `claude/content-plan-d6bc49` | `ba8e6d7d` | 2026-11-03 | 43 | no | Only commit is a copy of a shader commit already in dev; the content plan itself was cherry-picked (ba8e6d7) 11-03 | Content plan |
| `claude/empty-join-fallback` | `5d7ed082` | 2026-11-04 | 0 | no | merging it changes nothing: its whole tree is in dev | — |
| `claude/haze-bliss-world` | `a53e929a` | 2026-11-06 | 1 | no | HDR haze noon: dev has the commit under another hash (same subject) | Look |
| `claude/headless-server-stall` | `0a0caedc` | 2026-11-04 | 3 | yes (1) | Headless server account: dev has Phase 3's "headless server's own account" (88c5b0bc0) | Phase 3 |
| `claude/journal-clip` | `475a3795` | 2026-11-05 | 1 | no | Journal scroll-bar clip: dev has the fix (561907ec8 "values clear of the scroll bar") | UI |
| `claude/journal-screen` | `a5b8fddb` | 2026-11-04 | 5 | no | Field Journal is in dev (journal-ui merged, 0b4289374); this tip was declared void by Animals | Animals / UI |
| `claude/knap-ui` | `80f6c742` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `claude/look-dawn-lift` | `9b9801ac` | 2026-11-06 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `claude/look-grade-mcf` | `35d1955c` | 2026-11-06 | 1 | no | Merging it changes nothing (tree already in dev): soft clouds to 9000 blocks, its revert, and the grade test | Look |
| `claude/look-leaf-range` | `ce7daabd` | 2026-11-06 | 3 | no | Leaf holes at a distance merged (Look); the second commit is test knobs for it | Look |
| `claude/look-skyao-h2` | `7a811978` | 2026-11-06 | 0 | no | merging it changes nothing: its whole tree is in dev | — |
| `claude/look-volume-clear` | `3effab53` | 2026-11-06 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `claude/look-volume-filter` | `e37674d0` | 2026-11-06 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `claude/models-32-45aa98` | `6905dfc0` | 2026-11-04 | 1 | no | Cherry-picked cleanly: all its changes are in dev (models-32 fmt) | Models |
| `claude/sad-cori-ce5f14` | `1863ab02` | 2026-11-04 | 6 | no | Release builds workflow: dev has it (same subject, 97% of the lines) | CI |
| `claude/sequoia-habitats` | `03c36174` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `claude/sky-birch` | `d9523b69` | 2026-11-05 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `claude/sky-bolts` | `3aaab3e7` | 2026-11-05 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `claude/sky-cloud-glow` | `2fb0c270` | 2026-11-05 | 1 | no | Clouds glow at sunrise/sunset: dev has it (same subject) | Shader |
| `claude/sky-dawn-mist` | `3559e90f` | 2026-11-05 | 2 | no | Dawn mist: dev has it (same subject) | Shader |
| `claude/sky-forest-sky` | `434a0258` | 2026-11-05 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `claude/sky-golden-b` | `85348540` | 2026-11-05 | 1 | no | Golden hours B: Planner 11-06 "keep golden hours B"; 80% of lines in dev | Shader |
| `claude/sky-mc-water` | `4bbcc46d` | 2026-11-06 | 0 | no | merging it changes nothing: its whole tree is in dev | — |
| `claude/sky-predawn-blue` | `76102671` | 2026-11-06 | 1 | no | Pre-dawn blue (P1): merged as predawn c2a1d364 (a614bd8a) 11-06 | Shader |
| `claude/sky-predawn-j7` | `9d334724` | 2026-11-06 | 0 | no | merging it changes nothing: its whole tree is in dev | — |
| `claude/sky-rain-readable` | `68b4af18` | 2026-11-05 | 0 | no | merging it changes nothing: its whole tree is in dev | — |
| `claude/sky-soft-clouds` | `f2cf4418` | 2026-11-05 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `claude/sky-soft-shadows` | `f5f77eac` | 2026-11-05 | 2 | no | Soft clouds are on by default and the cloud shadow map is in dev (cloud_shadow.rs); this shadow variant is superseded | Shader |
| `claude/sky-soft-shadows-dev` | `09967858` | 2026-11-05 | 2 | no | Soft clouds are on by default and the cloud shadow map is in dev (cloud_shadow.rs); this shadow variant is superseded | Shader |
| `claude/sky-torch` | `b19a6e6d` | 2026-11-05 | 0 | no | merging it changes nothing: its whole tree is in dev | — |
| `claude/sky-tuning` | `3bb06d7d` | 2026-11-05 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `claude/sky-wet-ground` | `fa63ae20` | 2026-11-05 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `claude/terrain-lakes` | `fca95fac` | 2026-11-04 | 0 | no | merging it changes nothing: its whole tree is in dev | — |
| `claude/tree-branches` | `b318128c` | 2026-11-04 | 12 | no | Branch blocks and space-colonized trees ported onto the Weft sync (14b9dedfd, cafa7dcb8); 88% of lines in dev | Trees / Voxel dog |
| `claude/trees-branchy` | `14716e3a` | 2026-11-04 | 6 | no | Branch blocks and space-colonized trees ported onto the Weft sync (14b9dedfd, cafa7dcb8); 88% of lines in dev | Trees / Voxel dog |
| `draft/frame-thread` | `8147d5f4` | 2026-11-04 | 0 | no | merging it changes nothing: its whole tree is in dev | — |
| `draft/frame-timing` | `86df8c14` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `draft/pipeline-cache` | `7794deea` | 2026-11-04 | 3 | yes (3) | Pipeline cache on disk: dev has it (same subject, 100% of lines); Weft has its own, taken at the sync | Lag |
| `draft/render-scale` | `bb5fb844` | 2026-11-04 | 9 | yes (8) | Render scale: tip already in dev (27330639); testkit config line also in dev | Lag |
| `draft/sound-handles` | `f96b4167` | 2026-11-04 | 12 | yes (11) | Sound handles: 94% of its lines are in dev | Sound |
| `fix/crosshair-third-person` | `a9d4e5e3` | 2026-11-04 | 1 | yes (1) | No crosshair in third person: in dev as 6a394c0d (ENGINE_CHANGES row), Weft PR #98 closed | Upstream |
| `fix/crosshair-third-person-clean` | `93227e10` | 2026-11-04 | 2 | yes (2) | No crosshair in third person: in dev as 6a394c0d (ENGINE_CHANGES row), Weft PR #98 closed | Upstream |
| `fix/far-terrain-forced-scatter` | `849ad8e9` | 2026-11-04 | 1 | yes (1) | Far terrain forced scatter: dev has it (same subject); Weft PR #96 | Upstream |
| `fix/far-terrain-forced-scatter-clean` | `944fcbbb` | 2026-11-04 | 1 | yes (1) | Far terrain forced scatter: dev has it (same subject); Weft PR #96 | Upstream |
| `fix/first-person-worn-bones` | `7e6e0eba` | 2026-11-04 | 1 | yes (1) | First-person worn bones: in dev as 58562745 (ENGINE_CHANGES row, Weft bug) | Upstream |
| `fix/flying-mount-no-fall` | `db56836c` | 2026-11-04 | 5 | yes (5) | Flying mount no fall: in dev as 6617ffe1 (ENGINE_CHANGES row, Weft bug) | Upstream |
| `fix/indirect-count-optional` | `b14dabbc` | 2026-11-04 | 4 | yes (4) | drawIndirectCount optional: dev has it (same subject); Weft has its own, taken at the sync | Upstream |
| `fix/mob-moods-new-world` | `d818e756` | 2026-11-04 | 1 | yes (1) | Mob moods per world: dev has it (ai.rs); the sync takes upstream's (6bc563297) | Upstream |
| `fix/particles-unloaded-cell` | `03c3cff0` | 2026-11-04 | 1 | yes (1) | Particles in an unloaded cell: in dev (ENGINE_CHANGES row, Weft bug) | Upstream |
| `fix/rule-values` | `3a596654` | 2026-11-04 | 5 | yes (5) | /rule values: dev has it (same subject, 100% of lines) | Upstream |
| `fix/rule-values-clean` | `410aa971` | 2026-11-04 | 5 | yes (5) | /rule values: dev has it (same subject, 100% of lines) | Upstream |
| `fix/texture-loads-once` | `ef30f342` | 2026-11-05 | 4 | yes (3) | Textures loaded once: superseded by preload-all (ENGINE_CHANGES: every texture loaded once after joining) | Upstream |
| `fix/vulkan-oom-system-memory` | `37b6b6e8` | 2026-11-04 | 1 | yes (1) | Vulkan OOM spill: dev has it (same subject, 100% of lines) | Upstream |
| `fix/vulkan-oom-system-memory-clean` | `e0861edb` | 2026-11-04 | 1 | yes (1) | Vulkan OOM spill: dev has it (same subject, 100% of lines) | Upstream |
| `glider-art-preview` | `815d9679` | 2026-11-05 | 5 | no | Glider art: installed in dev (100% of lines); preview only | Glider |
| `lighting-wip` | `cb713824` | 2026-11-01 | 18 | yes (18) | HDR world stage 1 WIP: superseded by the HDR world merged 11-05/06 | Lighting |
| `look-canopy2-trees` | `3ea3b7dc` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `look-dev-trees` | `4f30282d` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `look-fell-light` | `c58325df` | 2026-11-04 | 1 | no | Falling tree light: dev has it (same subject) | Look |
| `look-tropical` | `463a46bc` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `look-volume` | `bb242cc1` | 2026-11-04 | 2 | no | Volumetric light merged (dev has "Volumetric light" and froxels) | Look |
| `measure/upload-count` | `16f906a0` | 2026-11-05 | 2 | yes (2) | Mesh upload counter: 100% of its lines are in dev | Upstream |
| `release-builds-main` | `1863ab02` | 2026-11-04 | 6 | no | Release builds workflow: dev has it (same subject, 97% of the lines) | CI |
| `sequoia-try` | `a730e76f` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |
| `sky-tuning-old-order` | `afd880dd` | 2026-11-05 | 1 | no | Moonlit nights: dev has it (same subject) | Shader |
| `spans/solid-split` | `2214dd4e` | 2026-11-05 | 1 | yes (1) | update_solid split diagnostics: merged as spans/update-split 84b9d31c (board 11-05) | Upstream |
| `taau` | `0a93b783` | 2026-11-05 | 13 | yes (8) | Temporal AA (core:aa, on by default 11-06) and DLSS/GSR upscalers are in dev; TAAU is superseded | Lighting |
| `worktree-agent-a419f66e3f05057ce` | `5bc2a2ab` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev | — |

## Test only (109)

| branch | tip | date | files | engine | reason | owner |
|---|---|---|---|---|---|---|
| `bench/riv-after` | `efb96a8d` | 2026-11-05 | 0 | no | merging it changes nothing: its whole tree is in dev; name says A/B or test side | — |
| `bench/riv-before` | `e19e7b1d` | 2026-11-04 | 0 | no | merging it changes nothing: its whole tree is in dev; name says A/B or test side | — |
| `claude/armour-cloaks-run` | `d09e788e` | 2026-11-04 | 1 | no | Armour/cloak test branches ("not for merging") | Gear |
| `claude/armour-try` | `e8289eb1` | 2026-11-05 | 14 | no | Armour/cloak test branches ("not for merging") | Gear |
| `claude/churn-probe` | `c6b2b72d` | 2026-11-06 | 1 | yes (1) | PROBE: what changes with the light (not to merge) | Look |
| `claude/entity-cull` | `08b5610c` | 2026-11-04 | 4 | yes (3) | PARKED: no gain (the herd stays in the shadow box), pop-in risk (board 11-04 12:05) | Lag |
| `claude/far-ao` | `e814573a` | 2026-11-06 | 6 | no | Far AO (DH screen-space occlusion): "barely moves the far band, NOT merged" (board 11-07 00:15) | Far view |
| `claude/far-canopy-rough` | `585189d0` | 2026-11-06 | 5 | yes (3) | Far crown tiers / canopy / slopes: parked by Far view ("no visible change at view 24"), one commit set on three branches; the sync takes upstream's far crowns after an A/B | Far view |
| `claude/far-crown-tiers` | `30bb8d9a` | 2026-11-06 | 5 | yes (3) | Far crown tiers / canopy / slopes: parked by Far view ("no visible change at view 24"), one commit set on three branches; the sync takes upstream's far crowns after an A/B | Far view |
| `claude/far-end-fade` | `edb65c5d` | 2026-11-06 | 1 | yes (1) | Probe, no change to keep: PROBE + its revert; the real fix merged as far-end-fade2 / plain-stage1b | Far view |
| `claude/far-slopes` | `731158d1` | 2026-11-06 | 4 | yes (2) | Far crown tiers / canopy / slopes: parked by Far view ("no visible change at view 24"), one commit set on three branches; the sync takes upstream's far crowns after an A/B | Far view |
| `claude/far-wall-ao` | `17990716` | 2026-11-06 | 3 | yes (1) | Far wall AO: no effect on the dashes (pixel-identical), "NOT for merge" (board 11-06 21:15) | Far view |
| `claude/fh3-impact-test` | `9680264c` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `claude/gpu-oom-repro` | `fd7f8f13` | 2026-11-04 | 1 | yes (1) | REPRO ONLY, not to merge: fake GPU memory limits (the fix is in dev) | Upstream |
| `claude/gpu-oom-repro-nospill` | `efac34de` | 2026-11-04 | 1 | yes (1) | REPRO ONLY, not to merge: fake GPU memory limits (the fix is in dev) | Upstream |
| `claude/hitbox-before` | `f1df90bc` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `claude/look-canopy-beams` | `0c03a275` | 2026-11-06 | 1 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-canopy-beams2` | `8c1ac8c0` | 2026-11-06 | 1 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-canopy-test` | `f03e3ed3` | 2026-11-06 | 0 | no | merging it changes nothing: its whole tree is in dev; name says A/B or test side | — |
| `claude/look-cloud-b` | `0e6e6bf9` | 2026-11-06 | 1 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-far-blue` | `e49b0ee1` | 2026-11-06 | 1 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-far-blue2` | `d0715ab3` | 2026-11-06 | 1 | no | Far haze bluer: variants of one idea (same commit on 5 branches, joint test merges); no merge call recorded | Look |
| `claude/look-fogday-base` | `1f1b84ed` | 2026-11-06 | 15 | yes (10) | Mist m-series on top of look-fogday: test merge; the real branch is claude/mist-bliss-dawn (the fog-day fade merged as aee4bdf9) | Mist |
| `claude/look-fogday-volume` | `3d936d3b` | 2026-11-06 | 15 | yes (10) | Mist m-series on top of look-fogday: test merge; the real branch is claude/mist-bliss-dawn (the fog-day fade merged as aee4bdf9) | Mist |
| `claude/look-glare-b` | `b1826b6a` | 2026-11-06 | 1 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-golden-mist` | `c58534d8` | 2026-11-06 | 2 | no | Golden mist / grade: mixed test commits with a revert (soft cloud reach and shade fill tries) | Look |
| `claude/look-haze-facing-blue` | `1968b126` | 2026-11-06 | 1 | no | Far haze bluer: variants of one idea (same commit on 5 branches, joint test merges); no merge call recorded | Look |
| `claude/look-horizon` | `299d6abc` | 2026-11-07 | 1 | no | Far haze bluer: variants of one idea (same commit on 5 branches, joint test merges); no merge call recorded | Look |
| `claude/look-leaf-through` | `f488a5b2` | 2026-11-06 | 1 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-sky-facing` | `e9011cd9` | 2026-11-06 | 1 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-skyao-h1` | `e409233e` | 2026-11-06 | 1 | no | Variant of the merged far shade (dev took the 0.6 variant, 7590c6ca) | Look |
| `claude/look-skyao-third` | `56efdeff` | 2026-11-06 | 1 | no | Variant of the merged far shade (dev took the 0.6 variant, 7590c6ca) | Look |
| `claude/look-split-tone` | `8ca9c49f` | 2026-11-06 | 2 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-toe-b` | `0a70b2fb` | 2026-11-06 | 1 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-trees` | `02a9955a` | 2026-11-07 | 2 | no | Sky light by day in the sky's hue: disproved (board 11-07 00:25: vista navy is air in-scatter); carries the same far_soft commit as look-hill-risers | Trees |
| `claude/look-uw-test` | `6d7dcb06` | 2026-11-06 | 2 | yes (1) | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-volume-dither` | `d80d936e` | 2026-11-06 | 1 | no | Jitter-per-frame commit lives on in the look-volume-v tests; board: Mist's volume carries the fog-day fade | Look |
| `claude/look-volume-v2` | `0298b173` | 2026-11-06 | 1 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-volume-v3` | `392371f8` | 2026-11-06 | 1 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-volume-v4` | `e6e6eecd` | 2026-11-06 | 1 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/look-volume-v5` | `c497b804` | 2026-11-06 | 1 | no | Look test variant ("test:" commit): a sweep point, not for merge | Look |
| `claude/mac-qos` | `b3dfe105` | 2026-11-04 | 3 | yes (3) | Neutral on the Mac: "mac-qos NOT to merge" (board 11-04); the Linux nice-10 part merged separately | Lag |
| `claude/mcf-sheet-test` | `dd1daf7f` | 2026-11-05 | 0 | no | merging it changes nothing: its whole tree is in dev; name says A/B or test side | — |
| `claude/plain-stage1` | `edb65c5d` | 2026-11-06 | 1 | yes (1) | Probe, no change to keep: PROBE + its revert; the real fix merged as far-end-fade2 / plain-stage1b | Far view |
| `claude/probe-far-normals` | `589d83b3` | 2026-11-06 | 1 | yes (1) | PROBE far normals: not to merge | Far view |
| `claude/pt-denoisers-rr-test` | `1e5b144a` | 2026-11-06 | 3 | yes (3) | TEST ONLY, not to merge: RR guides wired by hand | pt-denoisers (local_7cea0fe3) |
| `claude/pt-denoisers-test2` | `b643adea` | 2026-11-06 | 11 | yes (10) | Test merge of the denoiser and path-tracer branches (M2 step 1 etc.): real work is on rt-path-tracing and pt-denoisers-13c78d | pt-denoisers (local_7cea0fe3) |
| `claude/rtx-showcase-m0-test` | `74eb9654` | 2026-11-06 | 0 | no | merging it changes nothing: its whole tree is in dev; name says A/B or test side | — |
| `claude/rtx-showcase-rt-test` | `7562b480` | 2026-11-06 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `claude/sky-band-morning` | `84a86fed` | 2026-11-06 | 1 | no | Dropped by Planner/Shader 11-06: "claude/sky-band-morning DROPPED (don't merge; Shader agrees)" | Shader |
| `claude/sky-blocky-low-sun` | `56d9d88a` | 2026-11-05 | 1 | no | Blocky clouds near a low sun: soft clouds are on by default, blocky is the off mode | Shader |
| `claude/sky-bodies-stars-test` | `9f6888c2` | 2026-11-06 | 1 | no | Merge of sky-bodies + sky-stars (test build); both live on their own branches | Shader |
| `claude/sky-cloud-cut` | `2eccf41a` | 2026-11-06 | 1 | no | Dropped: clouds-behind-far-land cut ("sky-cloud-cut dropped", board 11-06) | Shader |
| `claude/sky-cloud-sparse` | `2b42b4aa` | 2026-11-06 | 1 | no | Clear-day cover: Ronald picked "B: smaller puffs" (sky-cloud-cover f404757c); the 0.25 variants are not it | Shader |
| `claude/sky-evening-blue` | `1531996c` | 2026-11-06 | 1 | no | Blue evening: "not merged (keep golden hours B)" (Planner 11-06) | Shader |
| `claude/sky-evening-on-grade` | `1966aab9` | 2026-11-06 | 1 | no | Blue evening: "not merged (keep golden hours B)" (Planner 11-06) | Shader |
| `claude/sky-gold-land` | `683d5245` | 2026-11-05 | 1 | no | Shader dropped gold-land ("only +2-3 warmth", board 11-05) | Shader |
| `claude/sky-horizon-h3` | `ff3e5f60` | 2026-11-06 | 1 | no | Horizon band variant H3; the horizon fix to try is sky-horizon-haze | Shader |
| `claude/sky-joint-horizon-test` | `9fe1c4a4` | 2026-11-06 | 1 | no | Far haze bluer: variants of one idea (same commit on 5 branches, joint test merges); no merge call recorded | Look |
| `claude/sky-light-low-sun` | `68d035d5` | 2026-11-06 | 1 | no | shls "barely moves 18:00: NOT merged" (board 11-06 21:45) | Shader |
| `claude/sky-predawn-joint` | `9f7d09c8` | 2026-11-06 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `claude/sky-predawn-joint2` | `84f3798f` | 2026-11-06 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `claude/sky-predawn-joint3` | `6f1cd84d` | 2026-11-06 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `claude/sky-predawn-joint6` | `1dbe21e3` | 2026-11-06 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `claude/sky-puddles-debug` | `bc9ee730` | 2026-11-06 | 1 | no | DEBUG puddles (not for merge) | Shader |
| `claude/sky-shadow-sparse25` | `71300b29` | 2026-11-06 | 1 | no | Clear-day cover: Ronald picked "B: smaller puffs" (sky-cloud-cover f404757c); the 0.25 variants are not it | Shader |
| `claude/terrain-next` | `78599739` | 2026-11-04 | 4 | no | WIP prototypes for Ronald's pick (river terraces, karst, canyons): "do not merge"; karst clusters and the rest merged separately | Terrain |
| `claude/tree-crowns` | `7badc4fc` | 2026-11-06 | 1 | no | Try: smaller canopy crowns (a try beside Trees' crown work) | Trees |
| `combo/loading-cpu` | `80747ec2` | 2026-11-05 | 5 | yes (4) | Combo of three loading fixes for one LOADCPU measurement; the "solid cells" cost was the GPU fence (board 11-05 22:35); textures superseded by preload-all | Upstream |
| `cracks-both` | `7341825a` | 2026-11-04 | 0 | no | merging it changes nothing: its whole tree is in dev; name says A/B or test side | — |
| `fix/test-wire-on-a-lift` | `21ab7e28` | 2026-11-04 | 0 | no | merging it changes nothing: its whole tree is in dev; name says A/B or test side | — |
| `fix/thread-priorities` | `5a103fab` | 2026-11-05 | 2 | yes (1) | Old thread-priorities: "NOT to be merged" (board 11-05 21:05), replaced by fix/thread-priorities-linux, in dev (4d3e6477a) | Upstream |
| `giant-test` | `b4c96e77` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `lighting-mac` | `af41366c` | 2026-11-05 | 14 | yes (9) | Test build: Lighting's Mac session integration (dev + TAAU + early writes) | Lighting |
| `lod-gpu-slice1` | `9d9bd8ae` | 2026-11-05 | 2 | yes (1) | LOD probes / experiments (core:lod_probe, smooth, split): PROBE or EXPERIMENT, "not for merging"; one probe chain on five branches | Far view |
| `lod-hiz` | `26545eb2` | 2026-11-05 | 5 | yes (4) | Hi-Z far culling DROPPED (board 11-06 00:35: at most 0.05 ms, "kept for the record") | Far view |
| `lod-probe` | `e20423be` | 2026-11-05 | 5 | yes (3) | LOD probes / experiments (core:lod_probe, smooth, split): PROBE or EXPERIMENT, "not for merging"; one probe chain on five branches | Far view |
| `lod-probe2` | `2efe4abd` | 2026-11-05 | 5 | yes (3) | LOD probes / experiments (core:lod_probe, smooth, split): PROBE or EXPERIMENT, "not for merging"; one probe chain on five branches | Far view |
| `lod-probe3` | `475c43de` | 2026-11-05 | 5 | yes (3) | LOD probes / experiments (core:lod_probe, smooth, split): PROBE or EXPERIMENT, "not for merging"; one probe chain on five branches | Far view |
| `lod-probe4` | `136038bb` | 2026-11-05 | 5 | yes (3) | LOD probes / experiments (core:lod_probe, smooth, split): PROBE or EXPERIMENT, "not for merging"; one probe chain on five branches | Far view |
| `lod-probe5` | `6b8d8430` | 2026-11-05 | 5 | yes (3) | LOD probes / experiments (core:lod_probe, smooth, split): PROBE or EXPERIMENT, "not for merging"; one probe chain on five branches | Far view |
| `lod-smooth2` | `b62e3513` | 2026-11-05 | 1 | yes (1) | LOD probes / experiments (core:lod_probe, smooth, split): PROBE or EXPERIMENT, "not for merging"; one probe chain on five branches | Far view |
| `lod-split3` | `26380ff1` | 2026-11-05 | 1 | yes (1) | LOD probes / experiments (core:lod_probe, smooth, split): PROBE or EXPERIMENT, "not for merging"; one probe chain on five branches | Far view |
| `look-bisA` | `8b379a36` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `look-bisB` | `3cd028de` | 2026-11-04 | 2 | yes (1) | Look-only bisect point (11-04) | Look |
| `look-bisC` | `9955928b` | 2026-11-04 | 1 | no | Look-only bisect point (11-04) | Look |
| `look-canopy-trees` | `8a22ef26` | 2026-11-04 | 2 | no | Canopy check test scripts on Plants' rainforest | Look |
| `look-storm2` | `01690543` | 2026-11-04 | 1 | no | Two Look storm tries from 11-04, before the HDR world | Look |
| `lto-bench/fat` | `26b0ec22` | 2026-11-05 | 1 | no | LTO bench variants (thin / fat), for a PC bench | Perf |
| `lto-bench/thin` | `1c8b7854` | 2026-11-05 | 1 | no | LTO bench variants (thin / fat), for a PC bench | Perf |
| `pr-shots` | `003222f0` | 2026-11-04 | 3 | no | Screenshots branch for Weft PRs #96-98 (images only, linked by raw URL from the PRs): keep, never merge | Upstream |
| `r7` | `d9523b69` | 2026-11-05 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `reflect-faint` | `73c3a016` | 2026-11-05 | 1 | no | Reflections: parked, "no saving" on the PC (board 11-06 07:15 / 08:30) | Lighting |
| `reflect-half` | `382f6a8a` | 2026-11-05 | 2 | no | Reflections: parked, "no saving" on the PC (board 11-06 07:15 / 08:30) | Lighting |
| `scratch/weft-prs-all` | `9b613e57` | 2026-11-04 | 8 | yes (8) | Scratch merge of the Weft bug-fix PR branches (scratch) | Upstream |
| `shafts-after-trees` | `9e647b31` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `shafts-before-trees` | `ff97c1a9` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `snow-debug` | `04c38138` | 2026-11-04 | 1 | yes (1) | DEBUG: log the client's air (snow ring), not for merging | Upstream |
| `test/crown-tiers-on-trees` | `9050949b` | 2026-11-05 | 5 | yes (3) | Far crown tiers / canopy / slopes: parked by Far view ("no visible change at view 24"), one commit set on three branches; the sync takes upstream's far crowns after an A/B | Far view |
| `test/crowns-shadowfar` | `e4660541` | 2026-11-05 | 0 | no | merging it changes nothing: its whole tree is in dev; name says A/B or test side | — |
| `test/haze-noon-on-look` | `bcafd932` | 2026-11-06 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `test/mcpack` | `23f11c74` | 2026-11-05 | 0 | no | merging it changes nothing: its whole tree is in dev; name says A/B or test side | — |
| `test/og-canopy` | `666132da` | 2026-11-05 | 0 | no | merging it changes nothing: its whole tree is in dev; name says A/B or test side | — |
| `test/simple-mc` | `94a974d1` | 2026-11-05 | 0 | no | merging it changes nothing: its whole tree is in dev; name says A/B or test side | — |
| `test/survival-hang-a` | `c08f0405` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |
| `test/survival-hang-b` | `588fc3bc` | 2026-11-04 | 2 | no | Bisect only: 47a9ad60 without the four clothing slots | Survival |
| `test/tree-shapes-far` | `5bf66aee` | 2026-11-06 | 5 | yes (3) | Far crown tiers / canopy / slopes: parked by Far view ("no visible change at view 24"), one commit set on three branches; the sync takes upstream's far crowns after an A/B | Far view |
| `translucent-split` | `684e74ee` | 2026-11-05 | 6 | yes (5) | Measuring flag core:skip_parts (translucent part by part), for a Lighting bench | Lighting |
| `uw-coral-test` | `3eabb281` | 2026-11-04 | 0 | no | no commit of its own: every non-merge commit has a patch twin in dev; name says A/B or test side | — |

## Dropped by the Weft sync (4)

| branch | tip | date | files | engine | reason | owner |
|---|---|---|---|---|---|---|
| `claude/android-f16` | `09955c95` | 2026-11-06 | 4 | yes (3) | Android: half precision was slower, "parked" (board 11-06); Android port goes to upstream's | Android (paused) |
| `claude/android-port-3dedde` | `016f5cf8` | 2026-11-06 | 13 | yes (7) | Android port: "take upstream's" (ENGINE_CHANGES row 4, c95781e6f, 91a213775). Branch also holds the 120 fps notes (ANDROID_120FPS) | Android (paused) |
| `claude/android-taau` | `7d6a31c5` | 2026-11-06 | 10 | yes (6) | Android temporal upscale (GSR1 was the pick, 24 vs 25 fps); Android port goes to upstream's | Android (paused) |
| `claude/android-vrs` | `cb0eae05` | 2026-11-06 | 3 | yes (3) | Android 2x2 shading rate: "VRS (no effect) parked"; Android port goes to upstream's | Android (paused) |
