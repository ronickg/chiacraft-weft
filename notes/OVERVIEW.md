# chiacraft: the whole project, piece by piece

Written by `notes/overview/build.py` from `notes/overview/pieces.json`; edit that, then run the script. The visual board is `notes/overview/index.html`. How the gameplay systems connect: `notes/SYSTEMS.md`.

**The goal.** Players open their own worlds, find other people's without any central service, link them with portals and walk between them; missing mods download on the way; identity and achievements live on Chia. The game in those worlds: survival in the Valheim spirit with real temperatures and seasons, and animals to tame, breed and raise, from real facts.

■ built and seen working: 54 · ◧ partly built: 16 · □ not built yet: 25 · ◌ planned for later: 6

## Core next

What's missing that the rest leans on (marked `core` in the data).

- **Account and identity: Player and world DIDs**. Phase 4: through sponsors; world ownership both ways.
- **Linked worlds: Admission rules and bans**. Phase 4: allow lists, probation, DID age, shared ban lists.
- **Running worlds safely: Permissions**. Operator levels exist, but any player can still /give.
- **Running worlds safely: Sandboxed code mods**. Phase 6: wasmtime components, WIT shaped on Weft's SDK.
- **Survival: Clothing**. Fur and cloth clothes without armor value (open question).
- **Survival: Freshness and cooking**. Freshness and hot meals. Not yet: meals from several ingredients, salting and smoking.
- **Living world: Real places and pacing**. Researched (2026-11-03): a real place and its monthly climate for every biome, 57 plants' agronomy, real diets and food webs, and the pacing (the calendar for weather, the species' growing pace for plants). Decided: the fern forest after the Yixian (cool), the dunes wet in winter and spring, Velociraptors alone, temperatures as the real places have them. Next: the tables from it.
- **Living world: Rain by season**. Rain per season in the climate table (the savanna wet in summer, dry in winter); rain falls in a biome as often as its season says. Today rain is random and everywhere. Needs one engine flag: no rain drawn in a dry biome.
- **Living world: Plants by the place**. One plant table and one growth rule (warmth, water, light, fertility) for crops, wild plants and grass. Today wheat, berries and saplings grow on a timer, whatever the season.
- **Living world: Food categories and diets**. Grass, leaves, fruit, seeds, roots, insects, meat, fish, carrion, water, on every food, plant and species. Today a species is carnivore or herbivore with a list of items.
- **Living world: Wild hunger and foraging**. Wild animals get hungry like tames; one forage goal finds plants, food on the ground, water or prey. Condition (built) turns it into stats.
- **Living world: Food web per biome**. Researched: every biome's web is whole with 2 new full creatures and 14 simple ones in the overworld (wolf, jaguar; hare, rodent, small bird, deer, reindeer, fish, caiman, fox, coyote, eagle, owl, vulture, raven, gull), the farm animals as their wild forms, and Protoceratops, an iguanodont and 8 simple ones in the Cretaceous. Wave one: hare, rodent, deer, fish, fox, raven; Protoceratops, a small mammal, a lizard.
- **Living world: Loop balance checks**. Every species finds its food in each habitat every season; every plant grows somewhere; a year in each biome holds (no starving season, no eating it bare).
- **World content: Our own content plan**. The plan leaves game content out until the foundations stand; survival and animals now need it. For food, the living world's research is the plan: the species' diets and the biomes' plants and crops.
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
| ■ | 24 words in the keychain | Made quietly at first start (phase 1). Code: `crates/wallet`. |
| ■ | Login key | Derived from the words; can never spend; the iroh key. Code: `crates/account`. |
| ■ | Content-signing key |  |
| □ | Player and world DIDs (core) | Phase 4: through sponsors; world ownership both ways. |
| □ | Achievements on chain | Phase 4: signed by the world's DID, anchored in batches, opt-in. In-game advancements exist. |
| □ | Profile view | Phase 4. |

Code: `crates/wallet`, `crates/account`, `crates/engine/src/chiacraft/account_screen.rs`. Docs: `notes/PLAN.md`, `notes/HANDOFF.md`.

## Packages

Data mods (blocks, items, art) that download when you join a world, checked by hash and signed by their makers. 6 of 8 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Local packages |  |
| ■ | Identity and store | A package is the BLAKE3 hash of its manifest; files cached by hash. |
| ■ | Content swap between worlds |  |
| ■ | Download on join | Files checked whole, not piece by piece yet. |
| ■ | Signing and trust | Code: `crates/engine/src/chiacraft/package_trust.rs`. |
| ■ | Hardening | Fuzz targets exist; not run in CI. |
| ◌ | Mirrors and other players | iroh-blobs, resuming downloads. |
| ◌ | Reviewer lists, key rotation |  |

Code: `crates/packages`, `mods/packages`, `crates/engine/src/chiacraft/package_trust.rs`. Docs: `docs/packages.md`, `reports/Data packages for moddable voxel games.md`.

## Linked worlds

Worlds with their own addresses, linked by both owners, with portals between them. 5 of 8 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Your world runs on while you visit |  |
| ■ | World addresses | Code: `crates/engine/src/chiacraft/worlds/world_address.rs`, `crates/engine/src/chiacraft/worlds/saved_worlds.rs`. |
| ■ | Link requests and invites | Code: `crates/engine/src/chiacraft/worlds/links.rs`. |
| ■ | Portal blocks and travel | Code: `crates/engine/src/chiacraft/worlds/travel.rs`, `mods/portals`. |
| ■ | Friends | Code: `crates/engine/src/chiacraft/worlds/friends.rs`. |
| ◧ | Leftovers | Arrivals in other dimensions; links carried to a world's new address. |
| □ | Admission rules and bans (core) | Phase 4: allow lists, probation, DID age, shared ban lists. |
| □ | Finding worlds | Phase 5: listings on chain and on DIG, curated lists. |

Code: `crates/engine/src/chiacraft/worlds`, `mods/portals`, `crates/engine/src/chiacraft/bridge.rs`. Docs: `notes/PLAN.md`, `reports/World addresses links and portals.md`.

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

One body for players and animals: food sets how much of your stats you reach, temperature from a real climate, seasons, levels. 9 of 13 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Stats and levels | Nine stats, a point a level, XP from everything. Code: `mods/base/vitals/src/stats.rs`, `mods/base/vitals/src/levels.rs`, `mods/base/src/levels.rs`. |
| ■ | Stomach and food | Real nutrients, well fed from three foods, water as a nutrient. Code: `mods/base/vitals/src/food.rs`, `mods/base/vitals/src/stomach.rs`, `mods/base/src/survival/food.rs`. |
| ■ | Health | Max health from the stomach, healing from protein and greens. Code: `mods/base/src/survival/health.rs`. |
| ■ | Stamina | Climbing's cost waits for ladders. Code: `mods/base/vitals/src/stamina.rs`, `mods/base/src/survival/stamina.rs`. |
| ■ | Breath | Air under water; the oxygen stat doesn't lengthen it yet. Code: `mods/base/src/survival/breath.rs`. |
| ■ | Body temperature | Climate, season, hour, height, weather, fires, armor. Code: `mods/base/vitals/src/temperature.rs`, `mods/base/vitals/src/climate.rs`, `mods/base/src/survival/temperature.rs`. |
| □ | Clothing (core) | Fur and cloth clothes without armor value (open question). |
| ◧ | Seasons | Day length, temperatures, breeding. Rain by season and plants by season are in the living world; snow and autumn leaves later. Code: `mods/base/vitals/src/calendar.rs`. |
| ■ | Rested and comfort | Which furniture counts is still open. Code: `mods/base/src/survival/camp.rs`. |
| ◧ | Freshness and cooking (core) | Freshness and hot meals. Not yet: meals from several ingredients, salting and smoking. Code: `mods/base/src/survival/food.rs`. |
| ■ | Body effects | Poison, bleeding, sedation, carried weight. Code: `mods/base/vitals/src/effects.rs`, `mods/base/src/survival/body.rs`, `mods/base/src/survival/load.rs`. |
| ◧ | Death | Tombstone and XP lost; the tombstone doesn't crumble yet. Code: `mods/base/src/survival/mod.rs`, `mods/base/src/levels.rs`. |
| ■ | Difficulty | Code: `mods/base/vitals/src/difficulty.rs`. |

Code: `mods/base/vitals`, `mods/base/src/survival`, `mods/base/src/levels.rs`, `mods/base/src/hud.rs`, `mods/base/api`. Docs: `notes/SURVIVAL.md`, `notes/SYSTEMS.md`, `mods/base/vitals/SHEET.md`.

## Animals

Real species (and a wyvern) to tame, command, ride, breed and raise, with genes, seasons and bodies on the survival rules. 10 of 15 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Species as data | Seven species from real facts. Code: `mods/animals/core/src/species.rs`, `mods/animals/core/src/biology.rs`. |
| ■ | Stats, levels, XP | Code: `mods/animals/src/animal.rs`, `mods/animals/src/xp.rs`. |
| ■ | Taming | Feeding for the small ones, knocking out the raptors. Code: `mods/animals/src/taming.rs`, `mods/animals/src/knockout.rs`. |
| ■ | Commands, stances, tribes | Code: `mods/animals/src/commands.rs`. |
| ■ | Signatures | Barking, gifts, calm aura. Code: `mods/animals/src/signatures.rs`. |
| ■ | Riding | Utahraptor. Code: `mods/animals/src/riding.rs`. |
| ■ | Breeding, eggs, seasons | Code: `mods/animals/src/breeding.rs`, `mods/animals/src/eggs.rs`, `mods/animals/core/src/life.rs`. |
| ■ | Genetics, coats, evolution | Code: `mods/animals/src/coats.rs`, `mods/animals/core/src/coats.rs`, `mods/animals/src/evolution.rs`. |
| ■ | Habitats and the Cretaceous | Code: `mods/animals/src/habitats.rs`, `mods/animals/src/cretaceous.rs`, `mods/animals/src/gateway.rs`. |
| ■ | Bodies | Comfort ranges, seeking warmth or shade, condition from food. Code: `mods/animals/src/comfort.rs`. |
| □ | Imprinting babies | The value exists; nothing raises it yet. |
| □ | Care for tames | Barns, coats, troughs (shelter and fires work already). |
| □ | Sedatives from plants | By dose and body mass. |
| □ | Allen's and Gloger's rules |  |
| ◌ | Capsules, jobs, journal |  |

Code: `mods/animals`, `mods/animals/core`. Docs: `notes/ANIMALS.md`, `mods/animals/SPECIES.md`, `docs/creatures.md`.

## Living world

The loop: seasons bring heat and rain, rain grows plants, plants feed grazers, grazers feed hunters, and what dies goes back to the soil. Each part reads five shared things (the place, the climate table, the plant table, food categories, the forage goal) instead of rules of its own. 0 of 16 parts built.

| | Part | Notes |
|---|---|---|
| ◧ | Real places and pacing (core) | Researched (2026-11-03): a real place and its monthly climate for every biome, 57 plants' agronomy, real diets and food webs, and the pacing (the calendar for weather, the species' growing pace for plants). Decided: the fern forest after the Yixian (cool), the dunes wet in winter and spring, Velociraptors alone, temperatures as the real places have them. Next: the tables from it. Code: `mods/base/vitals/src/climate.rs`, `mods/animals/core/src/biology.rs`. Docs: `notes/ECOLOGY.md`, `reports/ecology/Biome climates.md`, `reports/ecology/Plants and crops.md`, `reports/ecology/Overworld food webs.md`, `reports/ecology/Cretaceous food webs.md`. |
| □ | Rain by season (core) | Rain per season in the climate table (the savanna wet in summer, dry in winter); rain falls in a biome as often as its season says. Today rain is random and everywhere. Needs one engine flag: no rain drawn in a dry biome. Code: `mods/base/vitals/src/climate.rs`, `crates/engine/src/client/weather.rs`. Docs: `notes/ECOLOGY.md`. |
| □ | Soil water | One number per biome from the last days' rain and heat; ground near water always wet. Docs: `notes/ECOLOGY.md`. |
| ◧ | Plants by the place (core) | One plant table and one growth rule (warmth, water, light, fertility) for crops, wild plants and grass. Today wheat, berries and saplings grow on a timer, whatever the season. Code: `mods/base/src/blocks.rs`. Docs: `notes/ECOLOGY.md`. |
| ◧ | Food categories and diets (core) | Grass, leaves, fruit, seeds, roots, insects, meat, fish, carrion, water, on every food, plant and species. Today a species is carnivore or herbivore with a list of items. Code: `mods/animals/core/src/species.rs`, `mods/base/vitals/src/food.rs`. Docs: `notes/ECOLOGY.md`. |
| □ | Wild hunger and foraging (core) | Wild animals get hungry like tames; one forage goal finds plants, food on the ground, water or prey. Condition (built) turns it into stats. Code: `mods/animals/src/comfort.rs`. Docs: `notes/ECOLOGY.md`. |
| ◧ | Food web per biome (core) | Researched: every biome's web is whole with 2 new full creatures and 14 simple ones in the overworld (wolf, jaguar; hare, rodent, small bird, deer, reindeer, fish, caiman, fox, coyote, eagle, owl, vulture, raven, gull), the farm animals as their wild forms, and Protoceratops, an iguanodont and 8 simple ones in the Cretaceous. Wave one: hare, rodent, deer, fish, fox, raven; Protoceratops, a small mammal, a lizard. Code: `mods/animals/core/src/species.rs`. Docs: `notes/ECOLOGY.md`, `reports/ecology/Overworld food webs.md`, `reports/ecology/Cretaceous food webs.md`. |
| □ | New species, wave one | The smallest set that makes hunting visible: hare, rodent, deer, fish, fox and raven; Protoceratops, a small mammal and a lizard. Simple creatures first (one model, few animations), the rest biome by biome. Docs: `notes/ECOLOGY.md`, `reports/ecology/Overworld food webs.md`, `reports/ecology/Cretaceous food webs.md`. |
| □ | Hunters and prey | Carnivores hunt animals of a size they can take, prey flee, kills leave carcasses. Docs: `notes/ECOLOGY.md`. |
| ◧ | Capacity and populations | Spawning by how much a place feeds this season; young in the breeding season; starving ones die. Today habitats set spawning at fixed rates. Code: `mods/animals/src/habitats.rs`. Docs: `notes/ECOLOGY.md`. |
| ◧ | Death, dung and rot | Carcasses and dropped food rot, grazers leave dung, a compost bin turns them into compost. Today food items go stale, but rot has no use. Code: `mods/base/src/survival/food.rs`. Docs: `notes/ECOLOGY.md`. |
| □ | Fertile soil | Farmland fertility 0-3: crops tire it; compost, dung, beans or a fallow season restore it. Docs: `notes/ECOLOGY.md`. |
| □ | Loop balance checks (core) | Every species finds its food in each habitat every season; every plant grows somewhere; a year in each biome holds (no starving season, no eating it bare). Docs: `notes/ECOLOGY.md`. |
| ◌ | Migration and waterholes | Herds move to wetter land in the dry season; waterholes shrink. Docs: `notes/ECOLOGY.md`. |
| ◌ | The seasons' look | Grass colour by soil water, autumn leaves, winter snow. Docs: `notes/ECOLOGY.md`. |
| ◌ | Wildfire | Dry grass burns in the dry season and grows back greener. Docs: `notes/ECOLOGY.md`. |

Code: `mods/base/vitals/src/climate.rs`, `mods/base/api`, `mods/animals/src/comfort.rs`, `mods/animals/src/habitats.rs`. Docs: `notes/ECOLOGY.md`, `notes/ANIMALS.md`, `notes/SYSTEMS.md`.

## World content

What the world is made of: blocks, items, recipes, biomes, mobs. 4 of 5 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Blocks, items, crafting | Weft's base game. |
| ■ | Biomes with real climates |  |
| ■ | Mobs | Weft's. |
| ■ | Advancements |  |
| □ | Our own content plan (core) | The plan leaves game content out until the foundations stand; survival and animals now need it. For food, the living world's research is the plan: the species' diets and the biomes' plants and crops. Docs: `notes/ECOLOGY.md`. |

Code: `mods/base/src`, `mods/base/assets`. Docs: `docs/worldgen.md`, `notes/PLAN.md`.

## Look and feel

One warm pixel style: a copper-bronze UI kit, generated icons, box models painted like illustrations. 4 of 9 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Style guide |  |
| ■ | UI kit in the game | Code: `mods/base/source/ui`. |
| ■ | HUD | Code: `mods/base/src/hud.rs`. |
| ◧ | Remaining UI art | Menu background, logo, menu icons, armor points and more: the list is in STYLE.md. |
| □ | Fonts | Alegreya needs the smooth-font change. |
| ◧ | Item icons | Sheets made in the house style; the game still shows the old ones. |
| ◧ | Held item models | The stone sword only. Code: `mods/base/source/items`. |
| ■ | Creature models |  |
| □ | Palette per material | So every wooden tool shares its wood's colours. |

Code: `mods/base/source/ui`, `mods/base/source/items`, `mods/animals/source`. Docs: `notes/STYLE.md`, `.claude/skills/game-art/SKILL.md`.

## Tools and checks

What keeps it all working: game scripts and tests, balance and cross-system checks, generated sheets, this map. 5 of 7 parts built.

| | Part | Notes |
|---|---|---|
| ■ | Game scripts and tests | Code: `mods/testkit`. |
| ■ | Balance checks | Code: `mods/base/vitals/src/balance.rs`, `mods/animals/core/src/balance.rs`. |
| ■ | Cross-system checks | Code: `mods/animals/core/tests/places.rs`. |
| ■ | Generated sheets | Code: `mods/base/vitals/src/sheet.rs`, `mods/animals/core/src/sheet.rs`. |
| ◧ | Tests on CI (core) | This branch's CI checks formatting only; a workflow that runs the tests is on the claude/ci-workflow branch, not merged. |
| □ | Fuzzing on CI |  |
| ■ | Species website | The animals' logic as WASM. Code: `mods/animals/web`. |

Code: `mods/testkit`, `mods/animals/core/tests`, `mods/base/vitals/tests`, `.github/workflows`. Docs: `notes/SYSTEMS.md`, `CLAUDE.md`.
