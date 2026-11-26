# The first 30 minutes, played blind: brief (for Ronald or a playtester)

What it's for: the first hours are built (rock, flake, knapped head, string, axe, fire, logs,
table, stone pick) and every step logs when it first comes true. Scripts can't tell how long a
**new** player takes, so this needs a person who hasn't read `notes/FIRST_HOURS.md`. The plan
and the targets are there under "Timing it"; this file is just what to do.

## Before

- A build from current `dev` (release), the chiacraft look as installed. No mods or packs changed.
- Three fresh worlds, one each: seeds **1**, **7**, **42**. Survival, normal difficulty, mobs on.
  Name them `timing_1`, `timing_7`, `timing_42` (a new name per run: a reused one keeps its bag).
- Don't read `FIRST_HOURS.md`, the recipe book's notes or the hints file first.

## Playing

- 30 minutes of play per world, from the first moment in the world. No commands, no creative,
  no `/tk`, no wiki. Hints on screen are fine: they're part of what's being timed.
- Play as you would: try things, get stuck, look around. Dying is fine; keep going.
- If stuck for more than 3 minutes, say out loud (or jot down) what you're trying and which hint
  you last read. That note matters more than the times.
- At 30 minutes type `/firstclock` once (the only command), take a screenshot, quit.

## After (one short note per world)

1. The `first-clock:` lines from `run.log` (or the `/firstclock` reply): the second each step
   came, of `rock` (`bone` in snowy, savanna and swamp spawns), `edge`, `head`, `string`, `axe`,
   `fire`, `logs`, `planks`, `table`, `pickaxe`, `stone`.
2. Deaths, and what killed you.
3. Where you got stuck: what you tried, what you expected, which hint you read.
4. Anything that felt like a chore, or too easy.

## What we look at

The targets (minutes): rock 2.5, edge 4, head 7, string 9, **axe 13** (Ronald's "13-15"),
fire 16, logs 17, table/pick/stone 22-25. A step past its "too slow" mark (FIRST_HOURS.md's
table) in two of the three worlds gets a fix (a number in `vitals::first` or a hint), then the
same three seeds again. Notes go to Planner; the First hours session reads them.
