# chiacraft on Weft

A block game: a fork of the Weft voxel engine (`crates/`) with our mods (`mods/`): `base` (the
game), `animals`, `portals`, `packages`, `testkit`.

## Where to start

- **The whole project, piece by piece**: `notes/OVERVIEW.md` (and the visual board,
  `notes/overview/index.html`): every piece and its parts, what's built, partial, missing or for
  later, and what's core next. Its source is `notes/overview/pieces.json`: when you build or
  find something, update its status there and run `python3 notes/overview/build.py`.
- **The goal and the decisions**: `notes/PLAN.md` (phases 1-8: account, packages, linked worlds,
  DIDs, finding worlds, sandboxed mods, servers). Where the work stands: `notes/HANDOFF.md`.
  Research behind decisions: `reports/`.
- **Before changing gameplay**: `notes/SYSTEMS.md`: how survival, temperature, seasons, food,
  levels, animals, eggs and the HUD connect, and what to check when one changes. Why they are as
  they are: `notes/SURVIVAL.md`, `notes/ANIMALS.md`. The loop still to build (rain by season,
  plants, grazers, hunters, rot, soil), with its numbers from real places: `notes/ECOLOGY.md`. The numbers: `mods/base/vitals/SHEET.md`,
  `mods/animals/SPECIES.md` (both generated).
- **The look**: `notes/STYLE.md`, the `game-art` skill (all generated art: items, blocks, animal
  looks, UI, from the style guides in its `guides/`), `block-textures` (the 32 px block set and
  its batches) and `blocky-model` (box models).
- **Names and invented words**: `mods/base/lore/lexicon.toml` is the one source (the ancients' Old
  Eskenai, its drifts, Elvish, Dwarvish), `notes/NAMES.md` its generated sheet. Never make a name
  up freehand: `python3 tools/names/names.py gen --people <p>` offers roots, `derive <word> --steps`
  shows each tongue's form, `check` must pass. Why: `reports/Naming and invented words.md`.

## The engine's docs (Weft)

`README.md` (features), `docs/architecture.html` (how the engine is built),
`docs/modding.md` (the SDK: content, events, interceptors, value chains, settings and world
rules), `docs/ui.md`, `docs/worldgen.md`, `docs/creatures.md`, `docs/models.md`,
`docs/textures.md`, `docs/ctm.md`, `docs/shaders.md`, `docs/sky.md`, `docs/resourcepacks.md`,
`docs/machines.md`, `docs/redstone.md`, `docs/localization.md`, `docs/packages.md` (ours: data
packages). Leave Weft's engine alone where we can (`notes/PLAN.md`, rule 4): our features live
in our crates and mods.

## Working rules

- Numbers go in the shared crates, not loose in a mod's code: `mods/base/vitals` (survival:
  stats, food, temperature, climates, effects, difficulty, calendar) and `mods/animals/core`
  (species facts, life, eggs). Checks and sheets read them there.
- A new piece gets a part in `notes/overview/pieces.json`, a row in `notes/SYSTEMS.md` if it
  touches gameplay, and a check that ties it to what it touches.
- After changing numbers, regenerate the sheets (tests fail while they're stale):
  - `cargo run -p vitals --bin sheet > mods/base/vitals/SHEET.md`
  - `mods/animals/web/build.sh && node mods/animals/web/sheet.mjs` (or, with no WASM build,
    `cargo run -p animals_core --bin sheet > mods/animals/SPECIES.md`: the same page)
  - `cargo run -p progression --bin sheet > mods/base/progression/PROGRESSION_SHEET.md` (ages,
    curves, the combat matrix, every item's age and path; also after adding an item, block,
    recipe or species: the checks fail until it's tagged in `mods/base/progression`)
  - `cargo run -p crafting --bin sheet > mods/base/crafting/SHEET.md`

## Trying it in the game

`./target/release/weft --world <name> --script <file>` runs a testkit script (`loaded`, `idle`,
`chat`, `wait`, `key`, `hold <action> <secs>`, `cursor`, `click`, `tap <screen> <query>`, `shot`,
`where`, `leave`, `quit`; `/tk give|fill|spawn|mode|hurt_me|attr`). Worlds are saved under
`run/saves/<name>`, and a reused name keeps its inventory: use a fresh name per check.
Two traps that broke several scripts: `hold` waits until it releases unless given `nowait` (with `nowait`, follow it with a `wait` of its length),
and in `look <yaw> <pitch>` a positive pitch looks up (look down to aim at the ground).
Never launch the game yourself: the testers ("Tester", "Tester 2") run every script
(`.claude/skills/session-crew/SKILL.md`).

## Text

Every player-facing string is a translation key: the engine's in `crates/engine/assets/lang`
(en, ru, fr), base's in `mods/base/assets/lang` (en, ru), animals' in
`mods/animals/assets/lang` (en).
