---
name: weft-sync
description: Bring upstream Weft (the engine chiacraft is forked from) into our fork. The engine stays as close to Weft as it can, with our own changes on top. Use when Ronald says "merge weft", "sync with upstream", "update the engine", "what did weft fix", "port that from weft", or at the regular sync. Covers the fetch, a trial merge, which side wins each conflict, the ledger of our engine changes, checks, the runs that prove it, and the merge into dev.
---

# Syncing with Weft

Ronald, 2026-11-22: "want the engine to be as up to date from weft as possible and then just have my custom changes on top", and "ideally we should stick as close as possible to weft, and only like add modify where we really need it". So:

- **Weft is the base.** Our engine changes are a thin layer on top, and each one has a reason written down.
- **Sync often, in small steps.** A sync every week or two stays small. The first sync after 11-05 waited 17 days and conflicted in 484 files.
- **Where Weft has fixed what we fixed, take Weft's version** and drop ours.

## The pieces

| what | where |
|---|---|
| upstream remote | `upstream` = `https://github.com/VortexSo4/Weft`, branch `master` |
| sync helper | `notes/crew/weft-sync.sh [--apply]`, run during the merge |
| ledger of our engine changes | `notes/ENGINE_CHANGES.md`: files, why, and a verdict (keep in engine / move out / take upstream's / drop) |
| our diff against Weft, file by file | `notes/crew/engine-diff.sh`: also lists changed files with no ledger row |
| the last sync | the top line of `notes/crew/weft-sync-log.md`: date, upstream hash, dev merge |
| Weft's README | `docs/weft-README.md`, refreshed every sync. Ours is `README.md` |
| sorted list from the last sync | `notes/crew/upstream-<date>.md` |

## A sync, step by step

1. **Fetch and sort.**
   - `git fetch upstream`.
   - Then run `git log --no-merges <last-sync-hash>..upstream/master`.
   - Sort the commits into fixes, speed-ups, look changes, new systems and gameplay. Write them to `notes/crew/upstream-<date>.md`.
2. **Trial merge in a throwaway worktree.** `git worktree add --detach <scratch>/trial dev`, then `git merge --no-commit upstream/master`, and count the conflicts by area. Abort it and remove the worktree afterwards.
3. **Sync branch.** Cut `weft-sync-<date>` from dev in its own worktree, then `git merge upstream/master`.
   - Run `notes/crew/weft-sync.sh` (a dry run), then `notes/crew/weft-sync.sh --apply`. It takes upstream's side for conflicted engine files whose ledger **Merge** column says take upstream's or drop, or that have no row, and stages them.
   - It lists the rest with the rows to put back. Section 13 of `notes/ENGINE_CHANGES.md` is the per-file guide.
   - Resolve the rest by these rules:
   - **Engine file (`crates/`, `backends/`) with no ledger row:** take upstream's (`git checkout --theirs`). An edit with no reason has no reason to stay.
   - **Ledger verdict "take upstream's" or "drop":** take upstream's.
   - **Ledger verdict "keep in engine":** start from upstream's file and put our change back on top. Keep it as small as you can, and update the row if it moved.
   - **Ledger verdict "move out":** if the hook it needs exists, take upstream's and move our part into our crate or mod in the same branch. Until the hook exists, resolve it as keep in engine. Taking upstream's side would delete working features (11-22: ui.wgsl, looks.rs, input.rs, codec.rs, server/load.rs, the 32 px cracks).
   - **Far terrain (`client/lod.rs` and what's tied to it):** ours stays until an A/B against upstream's LOD rework picks a winner, because the stage 2 look depends on ours.
   - **Our art (`mods/*/assets/textures`, models, lang files of ours):** keep ours.
   - **Upstream's gameplay in `mods/base`** (new items, biomes, mobs): keep ours, and list theirs for Ronald. Taking their game content is his call.
   - **SDK reworks** (new APIs, renamed kinds): take upstream's and port our mods to them. Don't keep an old API alive in the engine for our sake.
   - **Generated files:** regenerate them (see `notes/crew/merge-into-dev.sh`).
4. **Ledger.**
   - Every engine file still differing from upstream must have its row. `notes/crew/engine-diff.sh` must list no file without a row.
   - Rows for fixes we dropped in favour of upstream's leave the ledger. Write them into the sync log instead.
5. **Checks.** Run niced `cargo check --workspace --all-targets` and clippy. Never tests on the Mac, never launch the game.
6. **Runs.** Hand Tester the branch, dev, and a script for the same view, comparing:
   - frame times;
   - the look at the stage 2 spots;
   - a survival smoke run.

   The rule (Ronald, 11-06): it goes in when it's equal or better. If it looks different, the change needs Ronald's ok first.
7. **Merge.** Planner merges the sync branch into dev with `notes/crew/merge-into-dev.sh`. Then:
   - add the sync log line (date, upstream hash, dev merge hash, conflicts, what we dropped in favour of Weft);
   - refresh `docs/weft-README.md` from `upstream/master:README.md`.
8. **Push.** Only with Ronald's yes, and only through `notes/crew/push-public.sh dev:dev`, which strips other people's reference images.

## Hand-porting a single fix

Use this between syncs, when one Weft fix is needed now:
- run `git cherry-pick -x <hash>`, so the next sync sees the same change and doesn't conflict;
- give it a ledger row marked "(from upstream <hash>)".

## Moving our changes out of the engine

Ronald, 11-22: "the more we can move out of the engine, the better, because the engine, I think, in Weft is becoming more flexible and allowing more mods to be its own like mod thing ... some places we just do need the custom version because it's not supported yet in Weft."

- **After every sync,** check each "keep in engine" and "move out" row against Weft's new SDK. Where Weft now offers the hook, move our code into our crate or mod and drop the engine edit.
- **Where a hook is missing,** add the smallest generic hook that any mod could use (an event, an interceptor, a value chain, a render-pass slot), not our feature. Note it in the ledger as our one engine line for that row.
- **Do the moves in their own branches, after a sync lands,** never during one. Each move is its own branch, and it must leave the game looking and playing the same.

## Keeping the layer thin between syncs

- A worker changes `crates/` only when a hook in our crates or mods can't do the job. The ledger row goes in the same branch.
- New code of ours goes into our own crates (`crates/packages`, `account`, `sandbox`, ...) or mods, not into new files inside Weft's crates.
- An engine bug that's Weft's own is marked **(Weft bug)** in the ledger. We send no pull requests to Weft (Ronald, 11-05).
