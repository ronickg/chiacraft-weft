# Changes to the session crew

Why each rule exists: the history behind `SKILL.md`, newest last. Add a dated line here whenever Ronald changes how the crew works; put the rule itself in `SKILL.md`.

- 2026-11-03: written. Ronald: Planner keeps all sessions working. Tester takes other sessions' runs
  first, then researches testing, then plays and hunts bugs and reports to workers. A Researcher
  session looks for improvements, simplifications and features. Keep this skill updated as Ronald
  changes the crew.
- 2026-11-03: Planner took over the workers from "Remaining systems to build". Every worker reports
  to Planner at the end of each turn. New work goes to Planner's linked sessions; existing workers
  keep their sessions, and their next parts move to linked sessions once they finish.
- 2026-11-03: Ronald: "make sure not to forget important pieces". Added `notes/crew/BOARD.md`, kept by
  Planner and read at every check-in.
- 2026-11-03: Ronald: "a preview/dev branch we can keep merging into without asking". Added `dev`
  in its own worktree; Planner merges ready branches into it freely. `main` still needs Ronald.
- 2026-11-03: Ronald answers every session from Planner's chat; relayed answers count as his. Noted the
  app's limit of about 10 sends per message from Ronald.
- 2026-11-04: Ronald: never throw away generated images. Added the image archive rule.
- 2026-11-04: Ronald: "improve and update skills if some questions are related". Answers that show a preference now go into the matching skill.
- 2026-11-04: the Mac crawled under 13 parallel builds. Release builds moved to the testers, builds capped at 4 jobs and niced.
- 2026-11-04: Ronald asked for a calm live UI in dark mode where he can watch and answer. Added the crew board page.
- 2026-11-03: Ronald wants images with the report-backs. Added "Pictures with every report".
- 2026-11-03: Ronald: re-ask open questions in the box every turn, so nothing sits idle.
- 2026-11-03: Ronald asked for more testers. Tester 2 owns the PC and Tester the Mac, so two games run at
  once; workers still send every run to "Tester", which forwards PC runs.
- 2026-11-03: Ronald: sessions with unfinished work continue in place until done; one archived by
  mistake is brought back and resumes where it stopped.
- 2026-11-03: Ronald: workers should be faster. Added "Working fast" (background builds, subagents,
  helper sessions through Planner) and turned on sccache.
- 2026-11-03: Ronald: "just do what u think i want, ask for more important questions". Added "What
  Planner decides alone".
- 2026-11-04: Testers check every script at the gate before running it: a `look` meant to aim down must have a negative pitch, and every `hold` needs a following `wait`. They fix an obvious sign themselves and tell the owner in one line (a positive pitch tripped 5 scripts in a day).
- 2026-11-04: Merging: after a merge that stops on conflicts, never `git commit -a` (it commits the conflict markers; it happened once, 1e0b5584, fixed forward in 3bd02ddf). Check `git diff --name-only --diff-filter=U` is empty, resolve real code conflicts by hand (or send them back to the owner), regenerate generated files, then commit. Fix forward if any branch already merged the bad commit.
- 2026-11-04: Planner merges into dev only through `notes/crew/merge-into-dev.sh <ref> "<message>"`: it regenerates the generated files, stops on any real conflict (exit 2, merge left open) and never commits markers. Chained `git commit -am` after a merge committed conflict markers twice in one night.
- 2026-11-04: Animal shots: wild animals flee the moment they spawn, so spawn them, then `chat /tick freeze` before the shot (the server still sends while frozen). A night of "animals not drawn" turned out to be this.
- 2026-11-05: No session or tester sends coins with wallet seed words, even on testnet; test checks fund through the sponsor (testnet11 only, behind may_spend), as the DID check does.
- 2026-11-04: Ronald: "check online for ways to further improve this skill". Following Anthropic's skill guide (keep SKILL.md lean, history in a reference file, rules not buried in a log) and multi-agent lessons (explicit ownership so agents don't duplicate work, handoff notes, a single integrator): the change log moved here, rules that lived only in it (script gate, seed words, merge script) moved into the body, and "One owner per piece", "Report format" and "When Ronald is low on usage" were added.
- 2026-11-04: Ronald: close done workers and start fresh sessions for different tasks; keep a handoff (`notes/crew/NEXT_SESSION.md`) ready for a fresh Planner.
- 2026-11-04: Ronald: "lets do all of this, so for next fresh sessions it's better": added "Keeping usage down" (6-8 workers, fresh short sessions, Sonnet for testers and Researcher, a 20-minute Planner check-in, notes/crew/crew-status.sh).
- 2026-11-04: Ronald: "Do them now": added the crew guard hook (no direct game launches, no `git commit -a` during a merge) and Planner's Monitor on Tester's runs. A Stop hook enforcing the report format was left out: it would cost a model call on every turn of every session.
- 2026-11-04: Ronald: "do they even need polling ... make sure the skill isn't outdated". Added "No polling" (linked sessions and SendMessage report on their own; the cron is only for the board page's answers), fixed "Messaging" (names work, not only ids), noted that start_session needs Ronald's own words in that turn, dropped the sccache restart steps (sccache is off), and the loop reads crew-status.sh instead of every transcript.
- 2026-11-04: Ronald: "Start fresh crew" after the old sessions stopped at 07:38; "max 10" sessions; "give the sub sessions better names". Crew board page republished (3s6XxavpJ7YVFc6pjCSUG3); titles say what each builds; `notes/crew/ROSTER.md` holds names and ids, and sessions send by id.
- 2026-11-04: Ronald: "i need to see images with qurstions pls, and also why did u not report this to me ?" (the old PC queue blocking the lag benches, mentioned twice by Tester 2 as "waiting on the old lag runs"). Every question now carries its picture, in chat and on the board page (question cards show `img`), and Planner checks any unplanned wait at once and tells Ronald.
- 2026-11-04: Ronald: "something's off ... u dont seem to respond or ask me". End-of-turn notes only reach Planner with his next message, so: sessions also SendMessage Planner their questions and verdicts, Planner checks in every 10 minutes with `notes/crew/session-notes.py`, and every note gets a visible line in Planner's reply.
- 2026-11-04: Ronald: "ask questions also in chat and not artifact pls" and "show me images here as well": Planner asks with AskUserQuestion after sending the picture into the chat; the board page mirrors.
- 11-04: Ronald can't see images sent in chat: "the images u should show me in the artifact pls as i cant see otherwise". Every image goes on the crew board page (feed row with img) before the question.
- 11-04: Pictures go on a local crew gallery (notes/crew/gallery, port 8765) instead of artifact uploads (Ronald: "maybe just run a local website instead of pushing to artifact prob more efficient").
- 11-04: The PC runner's limits are in the skill: it gates on live CPU (under 65%, not the load average), with at most 12 games, 4 GB of VRAM free, the GPU under 80 °C and 2 niced builds. Every game goes through t2q, multiplayer included (`multi`).
- 11-04: Mobs off = `/rule core:mob_spawning 0` after `loaded`; `false` is a usage error and `--config` can't reach the world-scope rule (Tester 2's night checks). This reverses the hour-old `false` line.
- 11-04: Weft PRs are Ronald's to write and open; the crew prepares the branch, test, results and a 3-line note (after VortexSo4 closed #83/#84; Ronald: "Draft, you rewrite", "Yes, I write and open").
- 11-04: No Co-Authored-By trailer or 🤖 line on Weft-bound commits (Ronald: it lists Claude among Weft's contributors).
- 11-04: every core change gets a line in notes/ENGINE_CHANGES.md (Ronald: "Yes, commit it"), and Planner checks for it before merging.
- 11-05: PC bench rules: worker benches wait at most 10 min; low regression benches only on an empty look queue (a reg bench had held 13 looks).
- 11-05: testkit hold waits until released unless nowait (CLAUDE.md and the skill corrected).
- 11-06: PC benches no longer chain; a 7-minute look round follows each bench; priority-1 looks block benches; soaks are frozen fillers while the queue is busy (40 looks had waited behind chained benches).
- 11-06: while more than 6 looks wait, only gating benches in queue/bench_ok.txt may start; the rest wait for the looks to drain.
- 2026-11-05: Ronald stopped Weft PRs ("from now on we not going to make any prs with fixes etc upstream. Just note them in some md file"). Engine bug fixes are noted in notes/ENGINE_CHANGES.md marked (Weft bug); the Upstream PRs section is now history.
- 11-06: Ronald: engine changes come in when they improve or measure the same ("only if it improves waht we have or stays same"). Added to the Upstream section; replaces the 'no gain, keep Weft's engine' reading of rule 4.
- 11-07: briefs carry the progression tagging rule (content.rs / danger.rs, curves, bounded tweaks) and the naming-tool rule.

- 11-05: briefs say no logic for old saves while the game is being built (Ronald: "we don't need any logic for old saves at the moment because we're still building the game").
- 11-05: the PC runner's timed hold (`t2q.py hold`), for work outside the queue such as Minecraft; short windows, never two in a row.
- 11-05: Never idle, results to the gallery at once, remind Ronald of paused work (Ronald, late 11-05).
- 2026-11-06: Nothing stuck rule (Ronald: "make sure no session gets stuck"): self-check runs every 30 min, benches >60 min let through, pcwatch alerts >75 min, Planner sweeps idle sessions each check-in.
- 2026-11-06: Test at Max rule (Ronald): every feature its own setting; Max preset is the benchmark default.
- 2026-11-06: Bench only the final candidate (benches took ~50% of PC time).
- 2026-11-06: Merge dev for the freeze (testkit clock 3fba8847).
- 2026-11-06: Look at every picture for bugs before passing it on (Ronald, after the mist rings and blob banks went unflagged): artefact checklist in 'Pictures with every report'.
- 2026-11-06: Read the reference's source before tuning (Ronald: Bliss and Distant Horizons code).
- 2026-11-06: Script gate notes: frozen-shot shader clock (params.w) and PC texture-pack seeding.
- 2026-11-06: Script gate: scripted runs pin shader clock (wind/water frozen) unless --record 1.
- 2026-11-22: Stay close to Weft: engine edits only where needed, every one in ENGINE_CHANGES with files, why and a verdict; engine-diff.sh checked at merges.
- 2026-11-22: Repo public; pushes only via push-public.sh; weft-sync skill for upstream merges.
