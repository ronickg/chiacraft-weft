# chiacraft: the whole project, piece by piece

Written by `notes/overview/build.py` from `notes/overview/pieces.json`; edit that, then run the script. The visual board is `notes/overview/index.html`. How the gameplay systems connect: `notes/SYSTEMS.md`.

**The goal.** Players open their own worlds, find other people's without any central service, link them with portals and walk between them; missing mods download on the way; identity and achievements live on Chia. The game in those worlds: survival in the Valheim spirit with real temperatures and seasons, and animals to tame, breed and raise, from real facts.

■ built and seen working: 52 · ◧ partly built: 10 · □ not built yet: 20 · ◌ planned for later: 3

## Core next

What's missing that the rest leans on (marked `core` in the data).

- **Account and identity: Player and world DIDs**. Phase 4: through sponsors; world ownership both ways.
- **Linked worlds: Admission rules and bans**. Phase 4: allow lists, probation, DID age, shared ban lists.
- **Running worlds safely: Permissions**. Operator levels exist, but any player can still /give.
- **Running worlds safely: Sandboxed code mods**. Phase 6: wasmtime components, WIT shaped on Weft's SDK.
- **Survival: Clothing**. Fur and cloth clothes without armor value (open question).
- **Survival: Freshness and cooking**. Freshness and hot meals. Not yet: meals from several ingredients, salting and smoking.
- **Animals: Food in the world**. Grazing and hunting, food categories; a biome's food sets how well fed its animals are.
- **World content: Crops and farming**. Growing through the seasons; feeds meals and animals.
- **World content: Our own content plan**. The plan leaves game content out until the foundations stand; survival and animals now need it.
- **Tools and checks: Tests on CI**. This branch's CI checks formatting only; a workflow that runs the tests is on the claude/ci-workflow branch, not merged.

## Engine

Weft, the voxel engine we forked: world, rendering, interface, modding. We change it only where our goals need it. 8 of 8 parts built.

| | Part | Notes |
|---|---|---|
| ■ | World and simulation | Cubic chunks, regions ticked in parallel that catch up on time away, saves, dimensions. Docs: `docs/architecture.html`. |
| ■ | Rendering | GPU terrain, coloured light, connected textures, shader packs, sky. Ours: items held as models, sharp UI scaling. Docs: `docs/sky.md`, `docs/shaders.md`, `docs/ctm.md`, `docs/textures.md`, `docs/models.md`, `docs/resourcepacks.md`. |
| ■ | Interface | Screens, HUD elements, nine-slice parts. Ours: world rules on Create New World. Docs: `docs/ui.md`. |
| ■ | Modding SDK | Content builders, events, interceptors, value chains, settings and world rules, translations. Docs: `docs/modding.md`, `docs/localization.md`. |
| ■ | World generation | Docs: `docs/worldgen.md`. |
| ■ | Machines and redstone | Docs: `docs/machines.md`, `docs/redstone.md`. |
| ■ | Creatures and riding | Blockbench models with animations, per-animal scale. Ours: riding. Docs: `docs/creatures.md`. |
| ■ | Seasons | Day length through the year per dimension (`.seasons`). Docs: `docs/sky.md`. |

Code: `crates/engine`, `crates/render`, `crates/sdk`, `crates/abi`, `crates/noise`, `crates/app`. Docs: `README.md`, `docs/architecture.html`, `docs/modding.md`.

## Account and identity

Who a player is, kept by them: 24 words, keys derived from them, later a DID on Chia. 3 of 6 parts built.

| | Part | Notes |
|---|---|---|
| ■ | 24 words in the keychain | Made quietly at first start (phase 1). |
| ■ | Login key | Derived from the words; can never spend; the iroh key. |
| ■ | Content-signing key |  |
| □ | Player and world DIDs (core) | Phase 4: through sponsors; world ownership both ways. |
| □ | Achievements on chain | Phase 4: signed by the world's DID, anchored in batches, opt-in. In-game advancements exist. |
| □ | Profile view | Phase 4. |

Code: `crates/wallet`, `crates/account`. Docs: `notes/PLAN.md`, `notes/HANDOFF.md`.

## Packages

Data mods (blocks, items, art) that download when you join a world, checked by hash and signed by their makers. 6 of 8 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Local packages |  |
| ■ | Identity and store | A package is the BLAKE3 hash of its manifest; files cached by hash. |
| ■ | Content swap between worlds |  |
| ■ | Download on join | Files checked whole, not piece by piece yet. |
| ■ | Signing and trust |  |
| ■ | Hardening | Fuzz targets exist; not run in CI. |
| ◌ | Mirrors and other players | iroh-blobs, resuming downloads. |
| ◌ | Reviewer lists, key rotation |  |

Code: `crates/packages`, `mods/packages`. Docs: `docs/packages.md`, `reports/Data packages for moddable voxel games.md`.

## Linked worlds

Worlds with their own addresses, linked by both owners, with portals between them. 5 of 8 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Your world runs on while you visit |  |
| ■ | World addresses |  |
| ■ | Link requests and invites |  |
| ■ | Portal blocks and travel |  |
| ■ | Friends |  |
| ◧ | Leftovers | Arrivals in other dimensions; links carried to a world's new address. |
| □ | Admission rules and bans (core) | Phase 4: allow lists, probation, DID age, shared ban lists. |
| □ | Finding worlds | Phase 5: listings on chain and on DIG, curated lists. |

Code: `crates/engine/src/world_address.rs`, `crates/engine/src/links.rs`, `crates/engine/src/travel.rs`, `crates/engine/src/friends.rs`, `mods/portals`. Docs: `notes/PLAN.md`, `reports/World addresses links and portals.md`.

## Running worlds safely

Always-on servers and code mods that can't hurt the player. 0 of 6 parts built.

| | Part | Notes |
|---|---|---|
| ◧ | Headless server | A dedicated mode exists; no server.toml, console or load tests (phase 7). |
| ◧ | Permissions (core) | Operator levels exist, but any player can still /give. |
| □ | Movement checks | Phase 7. |
| □ | Sandboxed code mods (core) | Phase 6: wasmtime components, WIT shaped on Weft's SDK. |
| □ | Personal mods per world | Phase 6, owner's choice. |
| □ | Our own relay | Phase 7. |

Code: `crates/engine/src/dedicated.rs`. Docs: `notes/PLAN.md`.

## Survival

One body for players and animals: food sets how much of your stats you reach, temperature from a real climate, seasons, levels. 7 of 11 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Stats and levels | Nine stats, a point a level, XP from everything. |
| ■ | Stomach and food | Real nutrients, well fed from three foods, water as a nutrient. |
| ■ | Stamina | Climbing's cost waits for ladders. |
| ■ | Body temperature | Climate, season, hour, height, weather, fires, armor. |
| □ | Clothing (core) | Fur and cloth clothes without armor value (open question). |
| ◧ | Seasons | Day length, temperatures, breeding. Not yet: snow and leaves, crops by season. |
| ■ | Rested and comfort | Which furniture counts is still open. |
| ◧ | Freshness and cooking (core) | Freshness and hot meals. Not yet: meals from several ingredients, salting and smoking. |
| ■ | Body effects | Poison, bleeding, sedation, carried weight. |
| ◧ | Death | Tombstone and XP lost; the tombstone doesn't crumble yet. |
| ■ | Difficulty |  |

Code: `mods/base/vitals`, `mods/base/src/survival.rs`, `mods/base/src/levels.rs`, `mods/base/src/hud.rs`, `mods/base/api`. Docs: `notes/SURVIVAL.md`, `notes/SYSTEMS.md`, `mods/base/vitals/SHEET.md`.

## Animals

Real species (and a wyvern) to tame, command, ride, breed and raise, with genes, seasons and bodies on the survival rules. 10 of 16 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Species as data | Seven species from real facts. |
| ■ | Stats, levels, XP |  |
| ■ | Taming | Feeding for the small ones, knocking out the raptors. |
| ■ | Commands, stances, tribes |  |
| ■ | Signatures | Barking, gifts, calm aura. |
| ■ | Riding | Utahraptor. |
| ■ | Breeding, eggs, seasons |  |
| ■ | Genetics, coats, evolution |  |
| ■ | Habitats and the Cretaceous |  |
| ■ | Bodies | Comfort ranges, seeking warmth or shade, condition from food. |
| □ | Food in the world (core) | Grazing and hunting, food categories; a biome's food sets how well fed its animals are. |
| □ | Imprinting babies | The value exists; nothing raises it yet. |
| □ | Care for tames | Barns, coats, troughs (shelter and fires work already). |
| □ | Sedatives from plants | By dose and body mass. |
| □ | Allen's and Gloger's rules |  |
| ◌ | Capsules, jobs, journal |  |

Code: `mods/animals`, `mods/animals/core`. Docs: `notes/ANIMALS.md`, `mods/animals/SPECIES.md`, `docs/creatures.md`.

## World content

What the world is made of: blocks, items, recipes, biomes, mobs. 4 of 6 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Blocks, items, crafting | Weft's base game. |
| ■ | Biomes with real climates |  |
| ■ | Mobs | Weft's. |
| ■ | Advancements |  |
| □ | Crops and farming (core) | Growing through the seasons; feeds meals and animals. |
| □ | Our own content plan (core) | The plan leaves game content out until the foundations stand; survival and animals now need it. |

Code: `mods/base/src`, `mods/base/assets`. Docs: `docs/worldgen.md`, `notes/PLAN.md`.

## Look and feel

One warm pixel style: a copper-bronze UI kit, generated icons, box models painted like illustrations. 4 of 9 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Style guide |  |
| ■ | UI kit in the game |  |
| ■ | HUD |  |
| ◧ | Remaining UI art | Menu background, logo, menu icons, armor points and more: the list is in STYLE.md. |
| □ | Fonts | Alegreya needs the smooth-font change. |
| ◧ | Item icons | Sheets made in the house style; the game still shows the old ones. |
| ◧ | Held item models | The stone sword only. |
| ■ | Creature models |  |
| □ | Palette per material | So every wooden tool shares its wood's colours. |

Code: `mods/base/source/ui`, `mods/base/source/items`, `mods/animals/source`. Docs: `notes/STYLE.md`, `.claude/skills/game-art/SKILL.md`.

## Tools and checks

What keeps it all working: game scripts and tests, balance and cross-system checks, generated sheets, this map. 5 of 7 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Game scripts and tests |  |
| ■ | Balance checks |  |
| ■ | Cross-system checks |  |
| ■ | Generated sheets |  |
| ◧ | Tests on CI (core) | This branch's CI checks formatting only; a workflow that runs the tests is on the claude/ci-workflow branch, not merged. |
| □ | Fuzzing on CI |  |
| ■ | Species website | The animals' logic as WASM. |

Code: `mods/testkit`, `mods/animals/core/tests`, `mods/base/vitals/tests`, `.github/workflows`. Docs: `notes/SYSTEMS.md`, `CLAUDE.md`.
