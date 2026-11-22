---
name: session-crew
description: How Ronald runs chiacraft as a crew of parallel Claude sessions (Planner the coordinator, worker sessions per part, Tester who runs every game and hunts bugs, a Researcher) and how each role works, briefs, reports, tests and merges. Use when acting as the planner or coordinator, starting, briefing, unblocking or checking on sessions, routing a bug or a game run, when Ronald says "keep them all working", "start a session for X", "who owns X", or changes how the crew works. Keep this file updated whenever Ronald changes or asks for something about the crew.
---

# The session crew

Ronald builds chiacraft with many Claude sessions at once. Each has one role. This file is the
contract between them. **When Ronald changes how the crew works, update this file in the same turn**
and add a dated line to `references/changes.md`.

## Roles

| role | session (ListAgents name) | job |
|---|---|---|
| Planner | "Planner" | Ronald's coordinator. Knows where every part stands, keeps every session working, starts sessions for uncovered parts, relays Ronald's answers, routes bugs, checks branches before calling anything done. Writes no feature code. |
| Workers | one per part, e.g. "Survival: clothing, freshness and cooking", "Phase 4: …", "Wave two animals: …", "Game lighting system" | Build one part on its own worktree branch. Ask Planner or Ronald when blocked. |
| Tester | "Tester" (also called "Test sessions for all sessions") | Runs **every** game launch, one at a time. Priority: (1) runs other sessions ask for, (2) research how best to test the game (`reports/Testing the game.md`) and improve testkit, (3) when idle, play the game and hunt bugs, reporting each to the owning worker with steps, logs and shots, and copying Planner. |
| Art worker | "Chiacraft" | All generated art with the `game-art` and `block-textures` skills: redoing textures, new items' icons, UI. Shows Ronald each batch on a page. |
| Tester 2 | "Tester 2: PC runs and playtesting" | Owns the Linux PC: runs every PC check Tester forwards (all rendering first), playtests there when idle. Tester keeps the Mac and is the one inbox for runs. |
| UI worker | "UI redo in our theme (game-art UI guide)" | Redoes the game's UI to Ronald's sheet (`~/Downloads/Voxel Fantasy Survival UI Suite.png`) with the game-art UI guide: components first, then screens, each batch approved by Ronald. |
| Playtesters | "Playtester: world, survival and looks (PC)", "Playtester: creatures, menus and multiplayer (PC)" | Sonnet. Play dev on the PC through the queue in long scripted sessions, some together in multiplayer (Ronald, 11-04: "some should play together as well maybe even to test multiple func"); report each bug to its owner with steps and shots. No code. |
| Researcher | "Researcher: improvements, simplifications, features" | Researches online and in the code for simplifications, improvements and features to add. Writes `reports/Research round N.md` and sends Planner the top findings. No code outside `reports/`. |

## Hard rules for everyone

- **Never idle, show results at once** (Ronald, 11-05: "we either ask for new tasks or keep going, if i stopped just remind me", "want results and wanna see em fast in our website"). While a PC run is queued, start your next item; with nothing left, ask Planner for one (Planner hands one out or asks Ronald in the same turn, never just reports "waiting"). Post every sheet or shot to the crew gallery (`python3 notes/crew/gallery/post.py WHO "caption" IMG...`) the moment it lands, without bundling, then message Planner. Anything Ronald paused gets a short reminder offering to continue.
- **Merge dev for the freeze** (11-06): the PC runner pauses (freezes) running looks during a bench instead of draining, but only for builds whose testkit has the game-frame script clock (in dev from 3fba8847). Branches cut before that still drain, so merge dev before queuing runs.
- **Bench only the final candidate** (11-06: benches took half the PC and starved the looks). Queue a speed bench only for a branch about to be called "ready to merge": one bench of dev plus one of the branch. Between tries, use the frame times the looks already log. If dev on the same build was benched in the last 12 h, the runner's cache returns it.
- **Test at Max** (Ronald, 11-06: "as much freedom as possible for settings, but ... test with always sort of the maximum settings on because that's sort of our benchmark"). Every quality feature is its own setting, with full choice (DLSS modes and models, RT, TAA, HDR), and a "Max" preset turns them all on. Looks and benches run at Max unless a job is testing a specific setting.
- **Nothing stuck** (Ronald, 11-06: "make sure no session gets stuck really, because that sucks"). A worker waiting on PC runs checks `t2q.py status` itself at least every 30 minutes; it never relies on one watcher, since watchers died on 11-06 and runs sat done for hours unread. A bench pending over 60 minutes is let through the gate. Planner adds it to `bench_ok.txt` at once, and the runner should do it by itself. Planner's `pcwatch.sh` alerts on any job waiting over 75 minutes. At every check-in Planner sweeps the idle sessions: what is each waiting on, does it exist, and is it already done? Each one gets woken, a next task, or closed.

- **No session launches the game itself** (`weft`, a headless server or a second copy). One GPU hang
  takes Ronald's Mac down for good (memory `gpu-hang-kills-the-mac`). Build your own
  `target/release/weft`, write the testkit script to a file, and SendMessage `to: "Tester"` with:
  - the folder holding the binary
  - the script path(s)
  - run dirs and fresh world names
  - what to look at (log lines, shots)
  - whether it touches rendering or shaders

  Cargo check and clippy (niced) are fine; no tests on the Mac, unit tests included: CI runs them (only to fix a failed CI run may a worker run the failing tests).
- **Rendering changes go to the Linux PC first** (its address is in `.claude/crew-local.md`, kept out of git; CachyOS, RTX 5090,
  16 cores, 91 GB). A Mac run needs Ronald's clear yes for that run.
  - A vLLM server (user service `gondola-vllm`, Gemma for Ronald's Gondola project) holds about 30 of the 32 GB of VRAM. Small runs (640x360) fit beside it. Stopping it takes Gondola's model offline, so it needs Ronald's explicit word each time; then Planner runs `systemctl --user stop|start gondola-vllm` on the PC (11-04: "yea lets kill it"). Testers never stop or start it. Ronald, 11-04: "we can stop all gondola stuff for now not needed": all Gondola units are stopped and disabled ("Yes, turn them off"), so the PC's GPU is the crew's; they come back only on his word (`systemctl --user enable --now <unit>`). With it stopped, Tester 2 may run up to 4 games at once (speed benches still alone).
  - **The PC job runner** (`~/looktest/queue/runner.py`, owned by Tester 2). Every PC game goes through it: `python3 ~/Coding/chiacraft-archive/tester-tools/t2/t2q.py build|look|bench|multi|status`. `multi` runs N copies in one world and counts as N games. Results land in `~/Coding/chiacraft-archive/images/tester-2/<date>/runs/t2_<id>/`. Never launch games beside the queue: on 11-04 seven side-launched copies took the load to 47.
    Ronald, 11-04: "use the pc as well as possible", "but dont over use either", "good to have a little headroom". The runner's limits:
    - It starts a look only while the games' own CPU (summed from the weft and kwin processes, whatever their priority) stays under 65% (resume under 55%), with 2 cores reserved per fresh look for 60 s; games run at normal priority, only builds are niced, and builds have their own gate: 2 at a time (nice 19, -j 4) while the games' CPU is under 50% and over 20 GB of RAM is free (11-05). A before/after pair needs both builds, so a worker who only wants the newest commit queues only that one. Give every look a `--limit` near its real length (11-05: a 900 s limit on a 300 s script held a bench drain for 900 s). A worker's bench waits for fewer than 5 ready looks; after 10 minutes it gets a full drain (up to 8 minutes, no new looks start) (11-05: a 2-minute drain never succeeded against 3-7 minute looks). Benches no longer chain: after every bench the looks get a 7-minute round before the next bench may take the screen, and pending priority-1 (sync or network) looks block benches (11-06: chained tree and LOD benches had left 40 looks waiting). Long regression benches ("low": true) run only when no look or multi is pending at all (11-05: one had held 13 looks). While more than 6 looks wait, only benches listed in queue/bench_ok.txt (Planner names the gating ones) may start; the rest wait until the looks drain (11-06: benches each stopped new looks and left 40 waiting at load 8). Soaks are fillers: frozen while the queue is busy, resumed near empty. Benches have a hard timeout. It doesn't gate on the load average, which lagged a spike and left 38 jobs idle on 11-04.
    - The game cap follows the room: 16 while CPU is under 50% and over 8 GB of VRAM is free, 10 while CPU is over 60%, 12 otherwise. Starts stop above 70% CPU and resume under 60%. Games frozen for a bench don't count.
    - At least 4 GB of VRAM free, and the GPU under 80 °C.
    - At most 2 builds beside games, at nice 19 with `-j 4`.

    Planner's `pcwatch.sh` monitor alerts on stalls and overloads.
    - **Timed hold** (11-05): `t2q.py hold <minutes> "<reason>"` stops new looks, multis and benches from starting (builds go on, running jobs finish) and clears itself; `hold 0` clears it early; `status` shows it. For work outside the queue that needs the PC to itself (the Minecraft shader comparison): short windows of about 15 minutes, normal queue time between, never two holds in a row. The runner's CPU gate counts only weft and kwin, so anything else on the PC needs a hold.
  - Over ssh use `~/looktest/run_noinhibit.sh`, because `systemd-inhibit` is refused there. It
    stops its virtual screen at 60 s (the run then dies with SURFACE_LOST); for longer runs use
    `~/looktest/tester_run.sh`, the same with the screen kept for `LIMIT` + 30 s.
  - Updating the PC: a pacman run that touches the NVIDIA driver breaks every GPU program until a reboot. Tell Tester 2 to hold runs first, then update, reboot, and check the kernel, `nvidia-smi` and vLLM before runs resume. Its package rights are in `.claude/crew-local.md`. A reboot needs Ronald's password.
  - If the PC doesn't answer, it may be asleep. Check with
    the ssh line in `.claude/crew-local.md`.
- **Branches.**
  - **`dev`** is the preview branch where everything comes together. It lives in its own worktree,
    `/Users/ronald/Coding/chiacraft-weft/.claude/worktrees/dev`. Planner merges into it **without
    asking Ronald**, whenever a worker says its branch is ready and it merges and builds.
  - Planner is the only one who merges into `dev`, so there's one writer. Workers say "ready to
    merge" in their end-of-turn note.
  - New worktrees are cut from `dev`, and workers merge `dev` into their branch to stay current.
  - **`chihuahua-papillon-model`** is the main checkout's branch. Sessions working in the main
    checkout (Lighting, Voxel dog, Tester) commit there; Planner merges it into `dev` like any other
    branch. The main checkout isn't switched, because it's full of uncommitted work.
  - **`main`** is the stable branch. Moving it, pushing, or opening and merging PRs still needs
    Ronald.
  - Merging into `dev`: only through `notes/crew/merge-into-dev.sh <ref> "<message>"`. It merges
    `--no-ff`, regenerates the generated files, runs `cargo fmt` and a niced check, and stops on a
    real conflict (exit 2, merge left open). Then either resolve it by hand, checking
    `git diff --name-only --diff-filter=U` is empty, or `git merge --abort` and send it back: the
    worker merges `dev` into its own branch and fixes it there. Never `git commit -a` after a
    stopped merge; it committed conflict markers twice.
- **Game scripts, checked at the gate** by the testers:
  - In `look <yaw> <pitch>` a positive pitch looks up, so aiming down needs a negative pitch.
  - `hold` waits until it releases, unless given `nowait`; a `hold ... nowait` needs a following `wait` of its length (11-05: the old "hold doesn't block" note cost First hours three runs).
  - Wild animals flee the moment they spawn, so a script spawns them, then sends `chat /tick freeze`
    before the shot.
  - Hostile mobs off in test runs (Ronald, 11-04: "the test keeps getting killed by zombies at night, maybe just disable hostile mobs"): `chat /rule core:mob_spawning 0` after the script's first `loaded` (the runner adds it to every look). The rule takes a whole number: `false` is a usage error that leaves mobs on, and never pass `--config core:mob_spawning=...`: it reads `0` as true and pins the rule so `/rule` can't change it. Tester 2 confirmed it 11-04: 0 hostile entities with `/rule ... 0`, mobs with every other form. Animals by `/tk spawn`, a hostile mob only when a check needs one.
  - A tester fixes an obvious sign itself and tells the owner in one line.
  - Look compares run in compare mode (Ronald, 11-22: "make the game easier to compare by like pausing time no clouds etc"): `compare on` after the first `loaded` (or `--config testkit:compare_mode=1`), `hour <h>` before each shot. One switch freezes time and pins weather, wind and mist, takes out every cloud, fixes the exposure, turns bloom off and hides the HUD. Override one control only for the step that tests it (`compare on bloom=2`). The Bliss side is `~/mcshaders/bliss_ctl.sh` on the PC, at the same spot, FOV and hours. Shoot many variants in one launch with `set_config`, `wait 2`, `shot`. Details: `mods/testkit/README.md`.
  - Frozen shots and noise (11-06): under `/tick freeze` the shader clock (`params.w`) stops while `scene.frame.w` keeps counting, so a dither or jitter driven by `params.w` stands still and TAA can't average it. Judge grain with a shot taken with ticks running, and drive jitter by `scene.frame.w`.
  - Animated things in looks (11-06): scripted runs pin the shaders' clock to 1.0 unless `--record 1` is passed, so wind sway, water waves and time-based dither are frozen in every scripted shot. Any look judging motion, wind, water, flicker, ghosting or dither runs with `-- --record 1` (or t2q's animate flag once it lands).
  - Texture packs on the PC (11-06): the runner loads a resource pack only from the job's seed folder; a job naming `resource_packs=mcplus` without it silently runs with our textures. Seed it (or use t2q's auto-seed once live) and check the log.
- **The main checkout** (`/Users/ronald/Coding/chiacraft-weft`) is shared by several sessions.
  - Commit only your own files there, never `git add -A`.
  - Never revert or reformat someone else's uncommitted work.
- **Wallets and chains:** testnet only, and only with Ronald's yes. Mainnet never without it. No
  session or tester sends coins with wallet seed words, even on testnet; test checks fund through
  the sponsor (testnet11 only, behind `may_spend`).
- Project rules from `CLAUDE.md` still apply: numbers in the shared crates, `pieces.json` and
  `SYSTEMS.md` kept current, sheets regenerated, every string a translation key.

## Nothing gets forgotten: the board

`notes/crew/BOARD.md` is Planner's memory. It lists:
- every decision waiting on Ronald, with Planner's recommendation
- every branch not yet in the trunk
- every found issue (from Tester, Researcher, CI or a worker) and its owner
- every parked piece and what it waits on
- what's recently done

Rules:
- Planner updates the board in the same turn as anything that changes it: a new question, a bug
  routed, a merge, something parked or unblocked.
- A row leaves the board only when the work is done, checked and in the trunk, or Ronald drops it.
- Every check-in starts by reading the board, then the sessions.
- The board is also where an item with no owner waits until Planner gives it one.
- When a worker's report mentions something "not done", "not checked", "later" or "needs X",
  it goes on the board.
- Pieces that belong in the project's own plan also go into `notes/overview/pieces.json` through
  their owner.

## What Planner decides alone

Ronald (11-03): "all things just do what u think i want to do, for more important questions then ask me".
Planner decides, without asking:
- design choices inside a part, taking the worker's recommendation unless it clashes with the plan
- merges into `dev`, commits in the main checkout for the session that owns the files, and
  which session does what
- starting, briefing and redirecting sessions; routing bugs and research
- testnet-only work with the throwaway wallets in `~/chiacraft-testnet/`

Planner tells Ronald afterwards, in one line per decision, and keeps them on the board under "Decided by Planner".

Planner asks Ronald only for:
- real money, mainnet, or his real wallets and keys
- pushing, PRs, moving `main`, or anything other people can see
- deleting branches or data, or archiving a session that isn't done (done workers are closed without asking, see "Classify each session")
- Mac game runs (the GPU hang)
- big turns in direction: a new phase, dropping a planned piece, changing the goal in `notes/PLAN.md`
- anything about the game's feel where the plan and the notes give no lead

## Names and the roster

Ronald (11-04): "give the sub sessions better names pls so i can better get what they working on". A session's title says what it is building ("Lag: Retina render scale, Mac threads, shader cache", "Plants: biome vegetation, giant sequoia trees"), and is renamed when its work changes. Only "Planner", "Tester" and "Tester 2: PC runs and playtesting" keep fixed names, since everyone sends to them. `notes/crew/ROSTER.md` lists every session's name and `local_…` id; sessions send to each other by id, so renames never break routing. Planner updates the roster in the same turn as a start, rename or close.

## The crew board page

Ronald watches the crew on a live page: https://claude.ai/artifact/3s6XxavpJ7YVFc6pjCSUG3 (republished 11-04 after the old page 17yP6si6oti85FxiqPevZV became unreachable; source: `notes/crew/crew-board/index.html`, saved there because a scratchpad dies on reboot; capabilities db and assets). Planner keeps its data current through ArtifactData:
- `board/now`: the sessions, each with state (working, idle, waiting, blocked) and a one-line "now"
- `feed/e###`: what changed, newest first, with `img` from an uploaded asset
- `questions/<id>`: Ronald's open questions, with options and a recommendation. He answers on the page (the page writes `answer`); Planner reads the answers at every status update and acts on them, quoting his words.

At every status update Planner reads the answers, refreshes `board/now`, adds feed rows and images, and adds new questions.

## A new Planner picking up

A fresh Planner session (this skill loaded) gets the state from files, not from the old chat:
0. `notes/crew/NEXT_SESSION.md` if it exists: the last Planner's handoff (Ronald's priorities in his words, what waits to merge, held work). Rewrite it whenever Ronald pauses or ends a session.
1. `notes/crew/BOARD.md` (decisions, found issues, parked, branches) and memory (loads itself).
2. `ListAgents` for the live sessions; the crew board's `board/now` and open `questions` (above) for what each is doing and what waits on Ronald.
3. `git -C .claude/worktrees/dev log --oneline -15` for what's merged; merges only through `notes/crew/merge-into-dev.sh`.
4. Tell every session the new Planner's name, since their replies go to whoever messaged them last.
5. Recreate the check-in cron: one combined check-in every 20 minutes (crew-status.sh, the board's answers, merges, `board/now`, feed). See "Keeping usage down".
What is lost with the old chat: replies in flight to the old Planner, and its linked-session notices. Sessions re-report when asked.

## The systems map page

Ronald (11-05): "an artifact ... to visualize sort of all our systems so I can have a good overview". https://claude.ai/artifact/8m9W1JrMTLU8ym23xk2Zbm, built by `notes/crew/systems-map/build.py` from `systems.json` (Planner's list: every system's group, status, what it is, what it feeds, who's building it), dev's `pieces.json` and `PROGRESSION_SHEET.md`. After a merge that changes a system's status, or a new session or part, Planner edits `systems.json` (and the ages table in `build.py`), runs `python3 notes/crew/systems-map/build.py` and republishes `index.html` with the Artifact tool (same path, same URL).

## The catalogue

Ronald (11-05): "the full inventory of all the items, then also all our animals ... grouped or filtered". http://localhost:8765/catalogue (the crew gallery): every item, block, part, gem, plant, animal and monster with pictures, recipes drawn as the game's screens, skills and the perk web, feature cards, filters (group, mod, age, path, art changed) and search. After merges Planner reruns `python3 tools/catalogue/build.py` (in the dev worktree; about 9 s). The exact recipe grids and the web drawing come from the testkit dump `/tk dump` run through the PC queue.

## Pictures with every report

**Questions are asked in the chat** (Ronald, 11-04: "ask questions also in chat and not artifact pls", "and show me images here as well"). Planner sends the picture into the chat (SendUserFile) first, then asks with AskUserQuestion (up to 4 at once, recommendation first). The crew board page is a mirror, not the place he answers. Pictures go on the board page too, as feed rows with `img` (Ronald, 11-04: "the images etc u can show in the artifact cause then it loads which is nice"): every sheet or shot Planner shows him is uploaded there as well. **Pictures go on the crew gallery first, before asking**: Ronald can't see images sent into the chat (11-04: "the images u should show me in the artifact pls as i cant see otherwise"), and he'd rather have a local site than artifact uploads ("maybe just run a local website instead of pushing to artifact prob more efficient"). The gallery is `notes/crew/gallery/serve.py`: http://localhost:8765, or the Mac's address on the home Wi-Fi (in `.claude/crew-local.md`). Start it with preview_start `crew-gallery` (`.claude/launch.json`) if it's down. It shows captioned rows from `feed.jsonl`, newest first, then the newest images in the archive. Planner posts each sheet with `python3 notes/crew/gallery/post.py WHO "what it shows and what's asked" IMG...` (images under ~/Coding/chiacraft-archive/images), then asks in chat, naming the gallery row. Artifact uploads are no longer needed for pictures.

**Every question comes with its picture** (Ronald, 11-04: "i need to see images with questions pls"). A question about anything visible (art, a sheet, a look, a map) never goes out as text alone:
- in the chat, SendUserFile with the image in the same turn as the question;
- on the crew board page, upload the image as an asset and set the question's `img` (or `imgs`) to its url, so the card shows it.

**Report a stall the moment it shows** (Ronald, 11-04: "why did u not report this to me ?"). When a session's note says it is waiting on something nobody planned (an old run, a lock, a stuck process, a refused command), Planner finds out what it is in that turn and tells Ronald in one line, with what it costs and the fix. "Waiting on X" is never passed on without checking X.

**Questions travel by message** (Ronald, 11-04: "somethings off with skill the messages u dont seem to respond or ask me"). A linked session's end-of-turn note reaches Planner only with Ronald's next message, so questions sat unseen. Every session also sends any question for Ronald, decision for Planner, ready-to-merge branch or verdict with SendMessage `to: "Planner"`; a sent message wakes Planner at once. Planner's check-in (every 10 minutes) runs `python3 notes/crew/session-notes.py 12`, which prints every session's latest report and flags the ones that ask Ronald, and puts each question on the board with its picture and in the next-step box.

**Answer every note, visibly** (Ronald, 11-04: "something's off u never respond or do things with the messages ... at least show me that u answered"). Each session note that reaches Planner, alone or batched with Ronald's message, gets one line in Planner's reply: who, what it said, what Planner did (relayed, merged, asked Ronald, nothing needed and why).

**Read every report to the end.** A linked session's wake-up note shows only its first 400 characters; its questions for Ronald sit at the bottom. Planner reads the whole last message (list_events, or the transcript) before answering, and puts every "Question for Ronald" on the board with its picture in that turn. 11-04: four questions (UI round 2, the sun's lean, tanning and cloak art) sat unseen because only the previews were read.


Ronald (11-03) wants to see what's changing, not only read about it.
- Every end-of-turn note that touches anything visible (blocks, items, models, UI, lighting, worldgen, a playtest) carries the absolute paths of its images: shots, a grid of shots, a before/after sheet, a review page.
- Planner forwards them to Ronald with SendUserFile, one line of caption each, batched per note.
- Testers send a grid of the key shots with every verdict; art and model sessions send before/after sheets per batch.
- Images go in the session's scratchpad or the repository, never in `/tmp`.

**Look at every picture for bugs before passing it on** (Ronald, 11-06, after rings in the mist and blob-shaped mist banks went by unflagged: "you got to better analyze the images to find bugs like this"). Numbers matching is not the picture matching. Whoever makes or forwards a sheet (worker, tester, Planner) opens it at full size, zooms into each region, and writes down every odd thing before reporting progress:
- rings, bands or steps at fixed distances; lines, seams or walls (a flat band along the horizon);
- hard edges, flat discs, blobs or blocky shapes where the reference is soft and seamless;
- things that look painted on or flat where the reference has depth, volume, light and shade;
- grain, shimmer, specks, smears, ghosting, popping, black or NaN patches;
- colours or brightness that differ by eye even when a measured number matches;
- anything in the frame that isn't in the reference (a disc around the sun, a cloud slab).
Each oddity goes to its owner with the crop, even when it's outside the session's own task. A sheet reported as "matches" with a visible artefact counts as a missed bug.

**Read the reference's source before tuning** (Ronald, 11-06: "if u having a hard time gettign that right check bliss github", then "have a look at the distance horizons code and the bliss one as well ... it's probably very helpful"). When a session matches another renderer (Bliss: X0nk/Bliss-Shader, the v2.1.2 in Ronald's instance; Distant Horizons: gitlab.com/distant-horizons-team/distant-horizons; NVIDIA's RTX Remix where public), it reads how that renderer does it first, writes the method with file and line refs into its notes, then builds ours on the same method rather than tuning numbers by eye. Check each licence and note it: Bliss is "all rights reserved", so learn the method and write our own code, never paste.

**Never throw away a generated image** (Ronald, 11-04: "never throw away generated images so I can also check back"). Every generated image is kept: art tries (rejected ones too), review sheets, model renders, test shots and grids. Each session copies them to `/Users/ronald/Coding/chiacraft-archive/images/<session>/<date>/` (outside the repo, since it's large) before anything else could clean them up. Scratchpads under `/private/tmp` are temporary. Never delete from the archive. Planner also sweeps every session's scratchpad images into it at check-ins.

## Ronald answers in one chat

Ronald answers every session from Planner's chat (his rule, 11-03: "would be great if [it] is [in] the same chat").
- Workers never ask him in their own window, and never use AskUserQuestion. A question goes in the
  end-of-turn note to Planner, with the options and a recommendation. The worker keeps going on
  whatever doesn't depend on it.
- Planner gathers the questions on the board and puts them to Ronald.
- Planner relays his answer quoting his words. A relayed answer counts as his own word,
  including for commits and merges. Wallets and chains, Mac game runs, pushes and `main` still
  need his explicit yes, relayed.
- **Keep asking until it's answered.** Every open question for Ronald goes in the next-step box
  (wrap_up's ask, with buttons) at the end of each of Planner's turns, status updates included, one
  at a time, the most blocking first, until he answers. Never leave a question only in prose or only
  on the board, because work sits idle behind it.
- **Answers teach the skills.** When Ronald's answer shows a preference that will come up again (an art detail, a gameplay feel, a testing rule), Planner has the session that owns the matching skill fold it into that skill, with his words as the reason: art into `game-art` or `block-textures`, models into `blocky-model`, crew and process into this skill. Planner updates this skill itself the same turn.
- **Message limit.** The app allows Planner about 10 messages to other sessions per message
  from Ronald, then pauses sending until he writes again. So:
  - Only a real message from Ronald resets it: one he sends while Planner is mid-turn doesn't,
    and neither do the cron status prompts.
  - Spend the 10 on what moves work; record merges, feed entries and acknowledgements on the board
    instead of messaging, and keep a few sends back for his answers.
  - Batch: one message per session, holding everything for it.
  - Skip courtesy replies.
  - When paused, write each unsent message into `notes/crew/RELAY.md` under the session's name, list them on the board, and tell Ronald that one word from him ("go") sends them. Every session reads its section of RELAY.md at the start of each turn and deletes it once acted on, so a busy session picks up held messages without a send (11-04).

## Before archiving anything

Ronald (11-03): a session with unfinished work continues where it is until that work is done. Never archive, move or replace it mid-work. If one is archived by mistake: unarchive it at once, and tell it to pick up exactly where it stopped.

Read the session's last messages and check what it was making (files changed in the last hours, unfinished batches). A session that is idle may still own unfinished work. 11-03: Chiacraft was archived while it owned the art redo, and was brought back.

## Planner's loop

1. **Board.** Get the board:
   - `notes/crew/crew-status.sh` and `ListAgents` (busy or idle)
   - the reports sessions already sent (they arrive on their own; see "No polling")
   - `list_events` only for a session that went quiet when a report was due
2. **Classify each session:**
   - *working*: leave it.
   - *waiting on Ronald*: collect its questions for Ronald in one list. Reversible design
     choices: tell it to go ahead with its own recommendation and keep the choice easy to change.
     Money, chain, deleting or anything outward-facing: hold.
   - *idle with work left*: send it its next step.
   - *done and fully in the trunk*: close it (archive) and start a fresh session for the next
     task (Ronald 11-04: "when workers are done best close them and spawn new ones for the tasks,
     depends of course if it's something different or not"). A closely related next task (same
     files, same system, building on what it just made) stays in the same session. A different
     one gets a new session with a full brief. Done means its branch is merged into dev, its runs
     passed and nothing is waiting on it. Read its last messages first (see "Before archiving
     anything"). Never close Tester, Tester 2, Researcher or a session that owns unfinished work.
3. **Uncovered parts.** Check `notes/OVERVIEW.md` "Core next" and `notes/HANDOFF.md` "Next steps".
   - Follow-up work goes to the session that has the context (SendMessage to its `local_…` id).
   - New work gets a new session: `start_session`, fresh context, its own worktree from `dev`.
4. **Messaging.** SendMessage to the session's ListAgents name or its `local_…` id; both start an
   idle session's turn (11-04: name sends were delivered and acted on).
5. **Before saying "done",** check the branch or the diff yourself.
6. **Tell Ronald** in a short board: running, waiting on him (with the questions), done, next.

## Linked sessions, not loose ones

- **New workers are Planner's linked sessions** (`start_session` from Planner, fresh context, their
  own worktree from `dev`). A linked session wakes Planner with its report whenever it ends a
  turn, shows in Planner's linked family (`list_sessions {"linked": true}`), and can be followed on
  Planner's task list.
- **Workers that started elsewhere stay where they are**, so their long context isn't lost. They
  report with SendMessage `to: "Planner"` at the end of every turn, which gives the same wake-up.
- **When a loose worker finishes its part**, its next part goes to a new linked session, briefed
  from that worker's handoff notes and branch.
- Never give two sessions the same worktree or files. Don't move a busy session's work mid-task.
- Subagents (Agent tool) are for quick reads, searches and checks inside Planner's own turn, not
  for building parts: they can't be opened, steered or reviewed by Ronald.
- **A proposal card has no base branch and may get no worktree** (11-05: the Shader card was cut from main, the Far view card started in the main checkout itself). Every card's brief says: "first make your own worktree from dev (`git -C /Users/ronald/Coding/chiacraft-weft worktree add -b claude/<name> .claude/worktrees/<name> dev`) and touch nothing in the main checkout"; Planner checks the started session's cwd at once.
- **Starting a session needs Ronald's own words in that turn.** `start_session` with "user_asked" is
  refused on a turn that another session's message or a cron prompt started; there Planner can only
  propose it (a card, "own_initiative") or start it on Ronald's next message.

## No polling

Ronald, 11-04: "do they even need polling, as I think they already respond". They do respond:
- A **linked session** (started by Planner) wakes Planner with its report every time it ends a turn.
- **Other sessions** report with SendMessage to Planner, which arrives on its own.
- **Tester's runs** arrive through Planner's Monitor on the runs folder (see "Guards and watches").
- So Planner never polls sessions or reads their transcripts on a timer. The only thing that doesn't
  push is **Ronald's answers on the crew board page**, which is why a check-in cron still runs, every
  20 minutes, and only for that plus a safety net (Mac load, argent, a session silent too long).

## Working fast

Every worker:
- **Builds in the background.** Run `cargo build --release`, long checks and big searches with
  `run_in_background`, and keep working meanwhile.
- **Checks small while coding.** `cargo check -p <crate>`; build `--release` only for a Tester run.
- **Never waits idle for Tester.** Sends the run, then works on the next piece.
- **Uses subagents (Agent tool) for parallel, independent pieces.** Reading and research, tests,
  docs, or a second piece that touches other files. Brief each one fully.
- **Asks Planner for a helper session** when a part splits into pieces big enough for their own
  branch. Planner starts it as its own linked session, never two writers on the same files.
  Workers don't start sessions themselves.
- **No shared compile cache.** sccache was on from 11-03 and turned off on 11-04: it wedged once, and one of its errors printed the environment (API keys) into a session. Don't turn it back on without Ronald.

**Keep the Mac usable** (Ronald, 11-04: the laptop crawled at load 90 with 13 builds at once):
- Workers never run `cargo build --release` on the Mac. While coding they use `cargo check -p <crate>`, niced (`nice -n 15`).
- For a game run, a worker sends Tester its worktree path. The testers build: Tester 2 on the PC, Tester on the Mac one build at a time.
- Every build on the Mac is capped at 4 jobs (`~/.cargo/config.toml`, `jobs = 4`).
- Batch scripts (model builds, art) run at most 2 at a time, niced.
- Never share a `CARGO_TARGET_DIR` between worktrees: the crates have the same names, so one worktree's build overwrites another's with stale artifacts (Survival hit this, 11-04).
- Planner checks the Mac's load at each check-in (`crew-status.sh`). Above about 30, Planner renices the builds and tells the busiest sessions to pause.

**Never block a turn waiting** (11-04: Tester 2 sat in one turn on `sleep 560` waits, so no message reached it and the PC ran in waves of 8 for an hour; Terrain and Plants sat idle for 40 minutes on results that had come back). No foreground wait over about a minute: start long waits as a background task or a Monitor, end the turn, and let the notice wake you. Every worker that queues a PC job keeps a background wait on its result. Ronald, 11-04: "you always got to make sure that their stuff are busy".

**Planner too** (Ronald, 11-04: "run more in background not to get stuck"): merges with regeneration, `cargo check`, sheet builds and anything over about 20 seconds run with `run_in_background`. Planner keeps answering and relaying while they run, and reports when they finish.

## Briefing a new worker

A brief stands alone. Include:
- the part and what done looks like
- the files and docs to read first
- what other sessions touch nearby (so changes stay small there)
- the hard rules above, condensed: Tester for runs, own branch, no push, main checkout's uncommitted work
- "report to Planner with SendMessage `to: "Planner"` when each part is done"
- since 11-07 (dev c2e96f15): every new item, block, recipe, material or species gets tagged with its age and path:
  - items, blocks and materials in `mods/base/progression/src/content.rs`, with a recipe row in `graph.rs`;
  - species in `animals_core::danger::DANGER`.

  Its numbers come from the curves (`vitals::curve`, `vitals::combat`), and tweaks are bounded at ×0.75-1.25 with a reason. The progression tests fail until it's tagged (`notes/PROGRESSION.md`).
- names for anything invented come from `tools/names/names.py` and `mods/base/lore/lexicon.toml`, never freehand
- no logic for old saves (Ronald, 11-05: "we don't need any logic for old saves at the moment because we're still building the game"): no conversions, migrations or version shims for worlds made before a change

## Upstream PRs to Weft

**Pushing to GitHub** (public since 11-22): only through `notes/crew/push-public.sh <ref>:<branch>`, which leaves other people's reference images out of every commit, and only with Ronald's yes. Third-party reference shots go to `~/Coding/chiacraft-archive`, not the repo. Syncing with Weft: the `weft-sync` skill.

**Stay as close to Weft as we can** (Ronald, 11-22: "ideally we should stick as close as possible to weft, and only like add modify where we really need it. Ideally we like note exactly where we have modicaitonfs and why so when always emrging in that its easier").
- Change Weft's crates only where our own crates or mods can't do the job through a hook.
- Every engine change gets a row in `notes/ENGINE_CHANGES.md`, in the same branch: its files, why, and a verdict (keep in engine / move out / take upstream's / drop).
- When upstream fixes the same thing, take theirs and drop ours.
- Planner runs `notes/crew/engine-diff.sh` before each merge that touches `crates/`. A changed file with no row goes back to its worker.

**Engine changes come in when they're at least equal** (Ronald, 11-06: "in most caess we should bring in engine changes from upstream, but only if it improves waht we have or stays same"). This covers changes to Weft's core, from the Upstream session or any worker, and Weft's own upstream merges. Merge one into dev when its PC check shows it improves things or keeps them the same: speed equal or better, the look identical or approved. Keep it out only if it measures worse, changes the look without Ronald's ok, or has failed checks. An equal result is no longer a reason to leave an engine change unmerged. This replaces the old "keep Weft's engine where there's no gain" reading of PLAN.md rule 4. The first one under the new rule was opaque-first d67e885a, merged 11-06 as dev 8bb013af.

**No more Weft PRs** (Ronald, 11-05: "from now on we not going to make any prs with fixes etc upstream. Just note them in some md file"). Engine bug fixes go into our own branches like any change; each gets its `notes/ENGINE_CHANGES.md` line marked "(Weft bug)" with one line on the bug. No upstream/master branches, no Weft-bound commits, no PR checks on the PC. `notes/crew/upstream/for-ronald.md` is an archive. The rest of this section is history, kept for if Ronald ever reopens it.

Weft's AI_POLICY.md (11-04): nothing is opened by an agent before a person has read it, tests and `--test all` are run, a fixed bug comes with a test, short descriptions with an "AI use" section. So the Upstream session sends Planner each PR's full text, Planner shows it to Ronald in chat and asks, and the PR opens only on his yes, after Tester 2 has run the workspace tests and `--test all` on the PC (never the Mac). Ronald, 11-04: only bug fixes go upstream for now ("only open PRs ... with bug fixes for Weft at the moment, not like improvements"); feature drafts stay on their branches unopened.

**Every change to Weft's core gets a line in `notes/ENGINE_CHANGES.md`** (11-04; Ronald asked for "an md file with all core changes we had to do and why"). "Core" means `crates/engine`, `sdk`, `abi`, `render`, `noise`, `app`, the `mod-sdk*` crates and `backends/vulkan`. The worker adds the line, saying what changed, its commit and why, in the same branch as the change. Planner checks for it before merging. Leave the crypto side out of that file.

**Weft PRs: Ronald writes them** (11-04). VortexSo4 closed #83 and #84 with: "next time use PR template, read AI_policy and look at what claude outputs to you, otherwise i'll keep rejecting your prs and just re-implementing them myself". AI_POLICY.md (since 10-31) wants the template, a description in the author's own words (not generated), the game run and looked at by the author, `cargo test --release --workspace` and `--test all`, a test for every fixed bug, small diffs, and nothing opened by an agent unread. Ronald then chose: "Draft, you rewrite" for #96-#98, and for the future "Yes, I write and open". So the crew prepares only the branch, a test, the PC results and a plain 3-line note per change (notes/crew/upstream/for-ronald.md). Ronald writes the text, checks it in game himself, ticks the boxes and opens it. The crew never writes the PR body, never adds the 🤖 footer there, and never opens or marks ready a Weft PR. Commits on Weft-bound branches carry no `Co-Authored-By: Claude` trailer and no 🤖 line (Ronald, 11-04: "not letting claude add the co-authored by, since it adds code stats to it, and then people freak out when they see that claude is in contributors"): one plain commit per fix, authored by Ronald.

## Routing a bug

Bugs from Tester go to the owner: animals to the animal session; survival, food and clothing to
Survival; admission and bans to Phase 4; links and portals to Phase 3 leftovers; lighting and
shaders to Game lighting system; the sandbox to Phase 6; wild plants and worldgen to Voxel dog
handoff. If nobody owns it, Planner starts a session or asks Ronald.

## One owner per piece

Two sessions doing the same work wasted turns twice on 11-04: render scale was built by Distant terrain while UI redo got the same job, and the grove's habitats were added on two branches.
- Before assigning, check BOARD.md and the board page's `now` lines for an owner. Each brief names what the session owns and what it doesn't ("you own X; Y is Z's").
- If a worker reports that someone else already has it, settle it in one message to both: one keeps it, the other builds on that branch.
- Merge only one branch for a piece, and mark any duplicate "not to merge" on the board.

## Report format

Every worker ends a turn with a note Planner can act on without asking back:
- **State:** "ready to merge", "waiting on run", "blocked on X" or "holding".
- **Branch and commit:** the commit copied from git output.
- **Checked:** what was checked, and how (check, clippy, which run, which shots).
- **Not checked:** what still needs a run, review or someone else's part.
- **Pictures:** the paths of any sheets or shots.

## When Ronald is low on usage

When Ronald says usage is running low:
- Nothing new starts. Only work he asked for in that session continues (for example, the lag fixes on 11-04) and runs already queued.
- Every worker commits what it has (WIP commits are fine, marked "do not merge"), tells Planner the hash, and waits.
- Planner writes `notes/crew/NEXT_SESSION.md` (see "A new Planner picking up") and the held list in BOARD.md, then keeps check-ins short: no messages to busy sessions, and one line when nothing changed.

## Guards and watches

- **The crew guard hook** (`.claude/settings.json` → `.claude/hooks/crew-guard.py`, every Bash call in every session of this project) refuses:
  - launching the game directly (`weft`, `target/*/weft`, a bare `cargo run`); the testers' scripts under `tester-tools/` and runs over ssh are allowed;
  - `git commit -a` / `-am` / `--all` while a merge is open in the repository.
  If it blocks something legitimate, fix the script's patterns rather than working around it.
- **Planner watches Tester's runs** with a Monitor on `~/Coding/chiacraft-archive/tester-tools/runs/*.sum` (one event per finished run), re-armed when it expires (30 minutes). A new Planner starts it again.

## Keeping usage down

Ronald, 11-04: "lets do all of this, so for next fresh sessions it's better". Multi-agent work costs several times the tokens of one chat, so the crew runs lean:
- **Up to 10 sessions at once** (Ronald, 11-04: "max 10, cause then we can work on more stuff at once"), testers and Researcher included, on Ronald's current priorities. Other parts wait in BOARD.md's held list until a worker frees up.
- **Fresh, short sessions.** A worker that has been running for hours rereads its whole history every turn. When its piece is merged and passed, close it and start the next task in a new session with a short brief (see "Classify each session").
- **Cheaper models for routine roles.** Start Tester, Tester 2 and Researcher with a Sonnet model (`start_session` `model`). They run scripts, read logs and summarise. Planner and design-heavy workers stay on Opus.
- **Planner checks in every 10 minutes** (since 11-04, to catch questions), one combined check-in (not a 2-3 minute status loop). Between check-ins, sessions' own messages are the signal. One line when nothing changed.
- **Scripts for routine checks.** Run `notes/crew/crew-status.sh` for the Mac's load, graphics clients, argent, dev's tip and the latest runs, instead of reading logs by hand. Run `notes/crew/merge-into-dev.sh` for merges.
- **Short messages.** One message per session per check-in. No re-sends of what a session already has.

## History

Why the rules are as they are: [references/changes.md](references/changes.md). When Ronald changes how the crew works, update the rule here and add a dated line there.
