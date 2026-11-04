# chiacraft: the whole project, piece by piece

Written by `notes/overview/build.py` from `notes/overview/pieces.json`; edit that, then run the script. The visual board is `notes/overview/index.html`. How the gameplay systems connect: `notes/SYSTEMS.md`.

**The goal.** Players open their own worlds, find other people's without any central service, link them with portals and walk between them; missing mods download on the way; identity and achievements live on Chia. The game in those worlds: survival in the Valheim spirit with real temperatures and seasons, and animals to tame, breed and raise, from real facts.

■ built and seen working: 62 · ◧ partly built: 19 · □ not built yet: 17 · ◌ planned for later: 6

## Core next

What's missing that the rest leans on (marked `core` in the data).

- **Account and identity: Player and world DIDs**. Phase 4: through sponsors; world ownership both ways.
- **Linked worlds: Admission rules and bans**. Phase 4: allow lists, probation, DID age, shared ban lists.
- **Running worlds safely: Entity send and activation ranges**. Research round 2, E1, in Weft's engine (to offer upstream): entities go to players within 96 blocks, far ones' moves every 2nd or 4th tick, and those no player is within 48 blocks of tick their goals and paths once a second. Built; Tester's measurement with a 200-animal pen and two players pending. The animals mod's per-tick countdowns (breeding, cat gifts, knockout anger) still to switch to `tick_elapsed`.
- **Running worlds safely: Permissions**. Operator levels exist, but any player can still /give.
- **Running worlds safely: Sandboxed code mods**. Phase 6 slice 1 built: a package's code/server.wasm (a wasmtime component, API chiacraft:mods@1.0.0 shaped on Weft's SDK) downloads with it on join and runs on the world's host under fuel, time and memory limits, per world, with storage in the save; sample clans (the animals mod's tribes). Slice 2: the join sheet marks packages that carry code (protocol 6), and a package's clashing command is left out. Left: more of the SDK (components, channels), client halves, personal mods.
- **Survival: Clothing**. Fur and cloth clothes without armor value (open question).
- **Survival: Freshness and cooking**. Freshness and hot meals. Not yet: meals from several ingredients, salting and smoking.
- **Living world: Real places and pacing**. Researched (2026-11-03): a real place and its monthly climate for every biome, 57 plants' agronomy, real diets and food webs, and the pacing (the calendar for weather, the species' growing pace for plants). Decided: the fern forest after the Yixian (cool), the dunes wet in winter and spring, Velociraptors alone, temperatures as the real places have them. The climate table follows it (vitals/tests/places.rs holds it there). Next: the plant table and the species' diets.
- **Living world: Wind and fog, snow, storms**. Each biome's real place gives its mean wind and fog days per season (reports/ecology/Wind and fog.md); vitals::air makes each day calm or windy, foggy or not: fog before dawn burning off by mid morning, sea fog, ice fog, cleared by the wind. Rain falls as snow under 1 °C, snow lies and melts by degree-days, falling snow in a strong wind is a blizzard, dry ground lifts dust storms. The weather tells the engine (set_air: snow drawn where it is cold), the pack draws fog banks glowing toward the sun, blizzards, dust, snow on tops, swaying plants in gusts, slanting rain, rings on water, wet ground. Gameplay: wind chill out in the open (the wind chill index), fog shortens how far hunters and prey see. /air shows or sets it. Light shafts: a screen effect (mods/base/assets/shaders/post/shafts.wgsl) streaks the bright sky round the sun past what stands in front of it, strongest at low sun and in fog; the engine gives screen effects the scene. Not yet: snow as blocks, wind in forests weaker, each place's own air across biome borders.
- **Living world: Plants by the place**. Wheat, berry bushes, saplings, sugar cane and cactus grow at warmth × water from their real agronomy (vitals::plants, growth.rs); wheat and berries at the growing pace (26 and 20 minutes at best), berries ripe only in summer and autumn; /growth shows the rate. Grazed grass grows back (forage). Frost kills tender crops by their real frost points (growth.rs), not yet seen in the game. Wild plants (2026-11-04, vitals::wild, mods/base/src/wild.rs): the tundra's sedges, cottongrass, reindeer lichen and northern berries, the desert's prickly pear, the wetlands' water hyacinth, wild rice and cattails, the forest's mushrooms, hazel and brambles, clover and wild sunflowers, and the Cretaceous's cycads, flowering shrubs and water lilies, each where its real place grows it, growing back, ripening, spreading or standing by its form; acorns, pine nuts and rose hips from oaks, spruces and roses; their fruit and nuts are foods. Fruit and seed feed only in their ripening season. Not yet: kelp (no block in water), the trees (mesquite, palms, the Cretaceous's), the rotten log, the crops, grazed and ripe looks; none of it seen in the game.
- **Living world: Food web per biome**. Researched: every biome's web is whole with 2 new full creatures and 14 simple ones in the overworld (wolf, jaguar; hare, rodent, small bird, deer, reindeer, fish, caiman, fox, coyote, eagle, owl, vulture, raven, gull), the farm animals as their wild forms, and Protoceratops, an iguanodont and 8 simple ones in the Cretaceous. Wave one: hare, rodent, deer, fish, fox, raven; Protoceratops, a small mammal, a lizard. Wave two: reindeer, coyote, small bird, owl, eagle, vulture, gull, caiman (13 species on 8 models), and both full creatures: the wolf (a natural tame, fed from the hand) and the jaguar (tamed by knock-out); still to come: the Cretaceous's tier 2.
- **World content: Our own content plan**. The plan leaves game content out until the foundations stand; survival and animals now need it. For food, the living world's research is the plan: the species' diets and the biomes' plants and crops.
- **Tools and checks: Tests on CI**. This branch's CI checks formatting only; a workflow that runs the tests is on the claude/ci-workflow branch, not merged.

## Engine

Weft, the voxel engine we forked: world, rendering, interface, modding. We change it only where our goals need it. 8 of 8 parts built.

| | Part | Notes |
|---|---|---|
| ■ | World and simulation | Cubic chunks, regions ticked in parallel that catch up on time away, saves, dimensions. Docs: `docs/architecture.html`. |
| ■ | Rendering | GPU terrain, coloured light, connected textures, shader packs, sky. Ours: items held as models, sharp UI scaling, and chiacraft's own shader pack (mods/base/shaderpack: a palette by time of day, golden hours, fog, storms, snow, wind), installed and turned on the first time the game is played (checked on an RTX 5090 and an M3 Max). Docs: `docs/sky.md`, `docs/shaders.md`, `docs/ctm.md`, `docs/textures.md`, `docs/models.md`, `docs/resourcepacks.md`. |
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

Always-on servers and code mods that can't hurt the player. 0 of 7 parts built.

| | Part | Notes |
|---|---|---|
| ◧ | Headless server | A dedicated mode exists; no server.toml, console or load tests (phase 7). |
| ◧ | Entity send and activation ranges (core) | Research round 2, E1, in Weft's engine (to offer upstream): entities go to players within 96 blocks, far ones' moves every 2nd or 4th tick, and those no player is within 48 blocks of tick their goals and paths once a second. Built; Tester's measurement with a 200-animal pen and two players pending. The animals mod's per-tick countdowns (breeding, cat gifts, knockout anger) still to switch to `tick_elapsed`. Docs: `docs/modding.md`. |
| ◧ | Permissions (core) | Operator levels exist, but any player can still /give. |
| □ | Movement checks | Phase 7. |
| ◧ | Sandboxed code mods (core) | Phase 6 slice 1 built: a package's code/server.wasm (a wasmtime component, API chiacraft:mods@1.0.0 shaped on Weft's SDK) downloads with it on join and runs on the world's host under fuel, time and memory limits, per world, with storage in the save; sample clans (the animals mod's tribes). Slice 2: the join sheet marks packages that carry code (protocol 6), and a package's clashing command is left out. Left: more of the SDK (components, channels), client halves, personal mods. Code: `crates/sandbox`, `crates/mod-sdk`, `mods/packages/src/sandboxed.rs`, `examples/wasm/clans`. Docs: `docs/sandboxed-mods.md`, `reports/Sandboxed code mods.md`. |
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
| ◧ | Seasons | Day length, temperatures, breeding. Rain by season and plants by season are in the living world, snow falls and lies by temperature (Wind and fog); autumn leaves later. Code: `mods/base/vitals/src/calendar.rs`. |
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

The loop: seasons bring heat and rain, rain grows plants, plants feed grazers, grazers feed hunters, and what dies goes back to the soil. Each part reads five shared things (the place, the climate table, the plant table, food categories, the forage goal) instead of rules of its own. 8 of 18 parts built.

| | Part | Notes |
|---|---|---|
| ◧ | Real places and pacing (core) | Researched (2026-11-03): a real place and its monthly climate for every biome, 57 plants' agronomy, real diets and food webs, and the pacing (the calendar for weather, the species' growing pace for plants). Decided: the fern forest after the Yixian (cool), the dunes wet in winter and spring, Velociraptors alone, temperatures as the real places have them. The climate table follows it (vitals/tests/places.rs holds it there). Next: the plant table and the species' diets. Code: `mods/base/vitals/src/climate.rs`, `mods/animals/core/src/biology.rs`. Docs: `notes/ECOLOGY.md`, `reports/ecology/Biome climates.md`, `reports/ecology/Plants and crops.md`, `reports/ecology/Overworld food webs.md`, `reports/ecology/Cretaceous food webs.md`. |
| ■ | Rain by season | Each biome rains on its real place's share of each season's days, in spells of a few hours (mods/base/src/weather.rs); the engine draws rain only there (set_rain, raining_at); /rain tells today's spell. Not yet: snow drawn by temperature (a taiga winter spell shows rain), the Cretaceous's wet-season label per biome. Code: `mods/base/vitals/src/climate.rs`, `mods/base/src/weather.rs`, `crates/engine/src/server/regions.rs`, `crates/engine/src/client/weather.rs`. Docs: `notes/ECOLOGY.md`. |
| ◧ | Wind and fog, snow, storms (core) | Each biome's real place gives its mean wind and fog days per season (reports/ecology/Wind and fog.md); vitals::air makes each day calm or windy, foggy or not: fog before dawn burning off by mid morning, sea fog, ice fog, cleared by the wind. Rain falls as snow under 1 °C, snow lies and melts by degree-days, falling snow in a strong wind is a blizzard, dry ground lifts dust storms. The weather tells the engine (set_air: snow drawn where it is cold), the pack draws fog banks glowing toward the sun, blizzards, dust, snow on tops, swaying plants in gusts, slanting rain, rings on water, wet ground. Gameplay: wind chill out in the open (the wind chill index), fog shortens how far hunters and prey see. /air shows or sets it. Light shafts: a screen effect (mods/base/assets/shaders/post/shafts.wgsl) streaks the bright sky round the sun past what stands in front of it, strongest at low sun and in fog; the engine gives screen effects the scene. Not yet: snow as blocks, wind in forests weaker, each place's own air across biome borders. Code: `mods/base/vitals/src/air.rs`, `mods/base/src/weather.rs`, `crates/engine/src/server/regions.rs`, `mods/base/shaderpack/look.wgsl`. Docs: `reports/ecology/Wind and fog.md`, `docs/shaders.md`. |
| ■ | Soil water | Worked out from the biome's last 8 days of rain, drying faster in the heat (vitals::plants::soil_water), so nothing is stored; ground within 4 blocks of water is soaked (base_api::ground_at). Code: `mods/base/vitals/src/plants.rs`, `mods/base/api/src/lib.rs`. Docs: `notes/ECOLOGY.md`. |
| ◧ | Plants by the place (core) | Wheat, berry bushes, saplings, sugar cane and cactus grow at warmth × water from their real agronomy (vitals::plants, growth.rs); wheat and berries at the growing pace (26 and 20 minutes at best), berries ripe only in summer and autumn; /growth shows the rate. Grazed grass grows back (forage). Frost kills tender crops by their real frost points (growth.rs), not yet seen in the game. Wild plants (2026-11-04, vitals::wild, mods/base/src/wild.rs): the tundra's sedges, cottongrass, reindeer lichen and northern berries, the desert's prickly pear, the wetlands' water hyacinth, wild rice and cattails, the forest's mushrooms, hazel and brambles, clover and wild sunflowers, and the Cretaceous's cycads, flowering shrubs and water lilies, each where its real place grows it, growing back, ripening, spreading or standing by its form; acorns, pine nuts and rose hips from oaks, spruces and roses; their fruit and nuts are foods. Fruit and seed feed only in their ripening season. Not yet: kelp (no block in water), the trees (mesquite, palms, the Cretaceous's), the rotten log, the crops, grazed and ripe looks; none of it seen in the game. Code: `mods/base/vitals/src/plants.rs`, `mods/base/src/growth.rs`, `mods/base/src/blocks.rs`, `mods/base/vitals/src/wild.rs`, `mods/base/src/wild.rs`, `mods/base/src/worldgen.rs`. Docs: `notes/ECOLOGY.md`. |
| ■ | Food categories and diets | Twelve kinds of food (vitals::food::Category) on every food item, and every species' researched diet as shares (Species::eats), shown in the sheets. Not yet: taming foods from the categories. Code: `mods/base/vitals/src/food.rs`, `mods/animals/core/src/species.rs`. Docs: `notes/ECOLOGY.md`. |
| ■ | Wild hunger and foraging | Wild plant eaters, and hunters with prey at home, get hungry and thirsty. One forage goal drinks first, then eats: thirsty, an animal walks to open water within 16 blocks and drinks until slaked (each species its own need by mass, and its own share drunk rather than eaten: a deer 70%, a vole 10%, fish none; mods/animals/src/drink.rs, animals_core::needs); hungry, it eats dropped food of its diet within 12 blocks first (food items by kind and portion, dropped seeds, spoiled food for carrion eaters; the engine's new World::dropped_item), then tall grass (grazed short, growing back), ripe berries and leaves. Every meal fills by mass, a tenth of its own filling it (a kill, an item, needs::fills), and a hunter arrives as fed as its prey are this season. Parched is as weak as starving, and kills a wild one in a game day. Checks: hunters fill up on their prey at home, most of every diet is in the game, a drink lasts 10 minutes or more, desert animals barely drink. Seen: a fox killing a mouse (4% of its food), a deer eating dropped berries, a hungry fox taking a dropped porkchop over mice. Not yet: hunger and thirst while an area is unloaded, remembered waterholes. Code: `mods/base/vitals/src/forage.rs`, `mods/animals/src/forage.rs`, `mods/animals/src/drink.rs`, `mods/animals/src/hunt.rs`, `mods/animals/core/src/needs.rs`, `crates/engine/src/host.rs`. Docs: `notes/ECOLOGY.md`. |
| ◧ | Food web per biome (core) | Researched: every biome's web is whole with 2 new full creatures and 14 simple ones in the overworld (wolf, jaguar; hare, rodent, small bird, deer, reindeer, fish, caiman, fox, coyote, eagle, owl, vulture, raven, gull), the farm animals as their wild forms, and Protoceratops, an iguanodont and 8 simple ones in the Cretaceous. Wave one: hare, rodent, deer, fish, fox, raven; Protoceratops, a small mammal, a lizard. Wave two: reindeer, coyote, small bird, owl, eagle, vulture, gull, caiman (13 species on 8 models), and both full creatures: the wolf (a natural tame, fed from the hand) and the jaguar (tamed by knock-out); still to come: the Cretaceous's tier 2. Code: `mods/animals/core/src/species.rs`. Docs: `notes/ECOLOGY.md`, `reports/ecology/Overworld food webs.md`, `reports/ecology/Cretaceous food webs.md`. |
| ■ | New species, wave one | The smallest set that makes hunting visible: hare, rodent, deer, fish, fox and raven; Protoceratops, a small mammal and a lizard. Simple creatures first (one model, few animations), wild only (Taming::Wild: they live, flee, breed and get eaten). Built: the hare, as two species on one model (the brown hare of open country and the mountain hare of the north, white in the snow), the wood mouse (fields, woods, the desert, the llanos, the dunes) the field vole (taiga, birch woods, tundra, mountains, marshes), and the deer as two species on one model (the white-tailed deer of the Americas and the red deer of Europe, antlers on males through a gene their sex sets), and the fish as two species on one model (the brown trout of cool rivers and lakes, the pacu of the swamp and llanos), each with coat genes, spawning in its habitats and running from hunters; their shared behaviour is mods/animals/src/wild.rs, and fish swim and come by the mod's own water spawner (mods/animals/src/fish.rs: the engine spawns nothing in water), not breeding yet; and the foxes, the first new hunters (the red fox of the north and the Arctic fox of the tundra on one model: hungry, they hunt voles, mice, hares and birds by size and eat their kill; seen taking a mouse and a hare), and the raven, the carrion bird of the cold biomes (on the ground and in the air: it takes off from walkers and hunters on its flying model and lands again; no nests yet, its young come out as fledglings); and Protoceratops, the Cretaceous's plant eater and the raptors' prey (herds on the red dunes and the scrubland, cropping ferns: the Cretaceous ferns are browse now, mods/base/vitals/src/forage.rs; eggs in nests in the wet season). and Kryptobaatar, the Cretaceous's small mammal (a multituberculate on the wood mouse's build). and the lizard, Priscagama (an early agamid: sprawling, bobbing its head and doing push-ups, basking flat). All ten are built, each seen in the game; still for later: nests for the raven, breeding for the fish. Reviewed against the blocky-model checklist (2026-11-03): the mouse, vole, Kryptobaatar, lizard and fish are painted finer (house.FINE, 256 texels a block: at 64 their heads were 2-3 texels, the eyes black squares), every one with eyes that blink, the raven's eye redrawn, Protoceratops' eyes bigger and its frill a plate. Code: `mods/animals/src/wild.rs`, `mods/animals/src/hare.rs`, `mods/animals/src/mouse.rs`, `mods/animals/src/vole.rs`, `mods/animals/src/deer.rs`, `mods/animals/src/fish.rs`, `mods/animals/src/fox.rs`, `mods/animals/src/raven.rs`, `mods/animals/src/protoceratops.rs`, `mods/animals/src/kryptobaatar.rs`, `mods/animals/src/lizard.rs`, `mods/animals/core/src/species.rs`, `mods/animals/source/hare`, `mods/animals/source/mouse`, `mods/animals/source/vole`, `mods/animals/source/deer`, `mods/animals/source/fish`, `mods/animals/source/fox`, `mods/animals/source/raven`, `mods/animals/source/protoceratops`, `mods/animals/source/kryptobaatar`, `mods/animals/source/lizard`. Docs: `notes/ECOLOGY.md`, `reports/ecology/Overworld food webs.md`, `reports/ecology/Cretaceous food webs.md`. |
| ■ | New species, wave two | The overworld's remaining simple creatures (notes/ECOLOGY.md, tier 2), each built with the blocky-model skill, reviewed against its checklist (head close-ups, idle, the resting pose, a blink frame) and seen in the game: the reindeer (on the deer's build: antlers on both sexes by the horns gene, a cow's smaller), the coyote (on the fox's: the legs longer, a howl in its idle, pounce and bite), the small bird as the house sparrow and the snow bunting (painted FINE, its own layout), the owl as the great horned owl and the snowy owl (eyes on the face, ear tufts on a build gene, the head turned far round), the eagle as the golden eagle and the savanna hawk (the llanos' and the Pantanal's), the turkey vulture (a bald red head, painted FINE), the herring gull, and the caiman as the yacare and the spectacled caiman (on the lizard's build: a jaw that gapes and bites, eggs in mounds in the rains). The birds share one build (mods/animals/source/bird: the raven's layout with knobs) and one flight (mods/animals/src/birds.rs, the raven's, which the raven uses now); their young come out as fledglings (no nests yet). With them tier 1's two full creatures: the grey wolf (on the coyote's build: tamed by feeding as the cat, commands, sitting then lying as looks; black and white wolves by their genes) and the jaguar (on the cat's build at the house density: rosettes from its genes, black jaguars by a dominant allele, tamed by knock-out as the raptors). Still for later: hunting from the air (birds hunt on the ground, as the raven does), nests, carcasses for the vulture, a young gull's brown plumage. Code: `mods/animals/src/birds.rs`, `mods/animals/src/reindeer.rs`, `mods/animals/src/coyote.rs`, `mods/animals/src/small_bird.rs`, `mods/animals/src/owl.rs`, `mods/animals/src/eagle.rs`, `mods/animals/src/vulture.rs`, `mods/animals/src/gull.rs`, `mods/animals/src/caiman.rs`, `mods/animals/core/src/species.rs`, `mods/animals/source/bird`, `mods/animals/source/reindeer`, `mods/animals/source/coyote`, `mods/animals/source/small_bird`, `mods/animals/source/owl`, `mods/animals/source/eagle`, `mods/animals/source/vulture`, `mods/animals/source/gull`, `mods/animals/source/caiman`, `mods/animals/source/wolf`, `mods/animals/source/jaguar`. Docs: `notes/ECOLOGY.md`, `reports/ecology/Overworld food webs.md`. |
| ◧ | Hunters and prey | Wild hunters get hungry with prey at home and hunt by their researched prey sizes (never their own family or players), a kill fills them by the prey's mass against their own, food lying near is eaten before a hunt, prey run from hunters that can take them (mods/animals/src/hunt.rs). Seen: a chihuahua took a chicken, a wyvern a raptor, a pig and a cat. Not yet: packs taking big prey together, carcasses of our own species (they drop nothing yet), hunting players (a decision). Code: `mods/animals/src/hunt.rs`, `mods/animals/core/src/species.rs`. Docs: `notes/ECOLOGY.md`. |
| ◧ | Capacity and populations | Plant eaters come by natural spawning as often as their food grows this season (never under a fifth; the engine's new natural-spawn question), arrive as fed as the place feeds them, a quarter young in their breeding season, and starve to death on an empty stomach in a game day (wild only). Dry-season grass is poor fodder. Not yet: hunters' capacity, standing water in swamps. Code: `mods/base/vitals/src/forage.rs`, `mods/animals/src/forage.rs`, `mods/animals/src/animal.rs`, `crates/engine/src/server/dim.rs`. Docs: `notes/ECOLOGY.md`. |
| ◧ | Death, dung and rot | Grazers leave dung; a compost bin takes spoiled food, dung, rotten flesh and plant waste and rots it into compost (vitals::soil, farming.rs); kills' drops stay as carcasses. Scavengers and hungry animals eat food and carrion off the ground (forage.rs). Not yet: food and carcasses rotting away on the ground, a carcass for our own species, the generated art. Code: `mods/base/vitals/src/soil.rs`, `mods/base/src/farming.rs`, `mods/animals/src/forage.rs`. Docs: `notes/ECOLOGY.md`. |
| ■ | Fertile soil | Farmland's fertility 3 to 0 sets crops' growth 100% to 20%; a ripe wheat harvest wears a level, dung and compost give levels back, a season unplanted one; hoes till grass and dirt (new). Not yet: clover and beans. Code: `mods/base/vitals/src/soil.rs`, `mods/base/src/farming.rs`, `mods/base/src/growth.rs`. Docs: `notes/ECOLOGY.md`. |
| ■ | Loop balance checks | Built: animals_core::web checks every food in a diet is in the world, every plant eater eats in each habitat every season, every hunter has prey where it lives, every biome has plants, plant eaters, hunters and a carrion eater, and everything but the top hunters is prey; the species sheet shows each biome's web and the gaps the plan fills (web::WAITING, 14 now: wave two and the Cretaceous tier 2). It found winter bare, so the world gained winter hay, twigs and mast, lichen, fungi, roots, insects and water plants (vitals::forage). Every plant grows somewhere (vitals/tests/places.rs). A year holds (web::grazing): in each biome's best season its grazers' full crowds eat at most 18% of what its tall grass grows back (far under real grazers' quarter to half: a choice for later); the grass cover moved to vitals::forage::GRASS_COVER, which worldgen reads. Code: `mods/animals/core/src/web.rs`, `mods/base/vitals/src/forage.rs`, `mods/animals/core/tests/places.rs`. Docs: `notes/ECOLOGY.md`. |
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
