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
- **The look**: `notes/STYLE.md`, and the `game-art` (UI parts, item icons), `block-textures` (block
  textures, 32 px) and `blocky-model` (models) skills.

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
  - `mods/animals/web/build.sh && node mods/animals/web/sheet.mjs`

## Trying it in the game

`./target/release/weft --world <name> --script <file>` runs a testkit script (`loaded`, `idle`,
`chat`, `wait`, `key`, `hold <action> <secs>`, `cursor`, `click`, `tap <screen> <query>`, `shot`,
`where`, `leave`, `quit`; `/tk give|fill|spawn|mode|hurt_me|attr`). Worlds are saved under
`run/saves/<name>`, and a reused name keeps its inventory: use a fresh name per check.

## Text

Every player-facing string is a translation key: the engine's in `crates/engine/assets/lang`
(en, ru, fr), base's in `mods/base/assets/lang` (en, ru), animals' in
`mods/animals/assets/lang` (en).
