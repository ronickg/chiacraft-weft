# Weft sync: resolving merge conflicts (shared brief for every helper)

Repo worktree: /Users/ronald/Coding/chiacraft-weft/.claude/worktrees/mist-bliss-dawn-2f90b6 (branch claude/upstream-fixes-1122-e85f21).
A merge of `upstream/master` (Weft, cab2d5f72) into our fork is OPEN there (`git merge --no-commit`). Our side (HEAD, "ours") is chiacraft:
a block game on a fork of the Weft engine (`crates/`, `backends/`) with our mods (`mods/`). The merge base is 2db18c238 (our 11-05 sync).
Upstream since then: 492 commits, including an SDK rework (kinds as "entries", tags instead of flag bits, unified content identities, `base_api` modules).

Ronald (owner), 11-22: "want the engine to be as up to date from weft as possible and then just have my custom changes on top",
"stick as close as possible to weft, and only add/modify where we really need it".

## Your job
Resolve every conflict (`<<<<<<<` / `=======` / `>>>>>>>`) in YOUR files only (listed in your prompt), editing the working-tree files.
When you finish, none of your files may contain a conflict marker (check with `grep -n '^<<<<<<<\|^=======$\|^>>>>>>>' FILE`).

Look at each side with read-only git:
- `git show :1:PATH` base, `git show :2:PATH` ours, `git show :3:PATH` upstream's (for a file in conflict).
- `git log --oneline 2db18c238..upstream/master -- PATH` upstream's commits on it, `git show <hash> -- PATH` one of them.
- `git log --oneline 2db18c238..HEAD -- PATH` ours; `git show <hash>` to see why.

## Rules (from .claude/skills/weft-sync/SKILL.md)
- **Upstream's code is the base.** Start from upstream's version of each conflicting hunk and put our change back on top, as small as you can.
- **Our engine changes stay** where they have a reason in the ledger. The ledger: `notes/ENGINE_CHANGES.md` (committed), and the newer
  per-row draft with verdicts at `/Users/ronald/Coding/chiacraft-weft/.claude/worktrees/trees-bliss-canopy-1d047c/notes/ENGINE_CHANGES.md` (read only).
  Verdicts: "keep in engine" and "move out" => keep ours on top of upstream's for now (moving out is Ronald's call later).
  "take upstream's" / "drop" => take upstream's. An engine edit of ours with NO reason anywhere (a pure leftover) => upstream's.
  Many of our changes are the look (HDR, TAA, shadows, RT, path tracing, pack options, our shader pack hooks), our far terrain,
  sound, animals' entity needs, data packages, linked worlds, testkit hooks, felling, Android pre-rotation (this one: take upstream's
  Android where it re-implemented the same). When unsure whether a bit of ours matters: keep it.
- **Far terrain (`client/lod.rs` and what is tied to it) stays OURS for now** (ledger: take upstream's only after an A/B). Upstream code that
  calls into their LOD: adapt it to ours, or keep ours at that spot.
- **Android: take upstream's** (ledger verdict, upstream c95781e6f, 91a213775).
- **SDK rework wins**: where upstream renamed or reworked an API (entries, tags, `abi::Modifier`, content identities), take upstream's API and
  write OUR additions against it. Don't keep an old API alive for our sake. If you can't see how ours maps onto theirs, keep ours
  in a form that is closest to theirs and put a `// SYNC: <what needs doing>` comment on it.
- **`minecraft.rs` (the Minecraft texture pack generator) stays deleted**: drop upstream's calls to `crate::minecraft::...`.
- **Our art and our game in `mods/` stay ours** (items, blocks, recipes, worldgen numbers, our lang strings). Upstream's new gameplay
  content in mods/base (new items, biomes, mobs, potions, ships, fallen trees...): keep it in for now where it doesn't clash, and LIST it
  for Ronald. Where it clashes with ours, ours wins and you list theirs.
- **Generated files** (sheets): leave; they get regenerated.
- Translation keys: every player-facing string is a key; keep both sides' keys in lang files.
- Comments in this codebase are plain English full sentences, our style; keep both sides' comments where both still apply.

## Do NOT
- Do not run git commands that write: no add, commit, checkout, restore, reset, stash, merge, rm. Read-only git only.
- Do not run cargo (other files still have markers; and the Mac must stay usable). No tests, never launch the game.
- Do not touch files outside your list (others are resolving them at the same time).

## Report
Write `/private/tmp/claude-501/-Users-ronald-Coding-chiacraft-weft--claude-worktrees-mist-bliss-dawn-2f90b6/3be3ec3b-66bd-4510-b30c-c5e5c507c000/scratchpad/sync/<your group>.md`:
per file, 1-3 lines: what of ours stayed, what of upstream's came in, any `// SYNC:` left (with line), and a section
"For Ronald" (upstream gameplay content, look changes, anything that changes how the game plays or looks).
Your final message: the group name, files done, count of SYNC comments, and the For Ronald items (short).
