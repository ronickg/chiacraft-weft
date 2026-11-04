# Terrain like Terralith, Tectonic and Still Life: what they do, what we take

Worldgen, 2026-11-04, for Ronald via Planner ("lakes, rivers etc.", and "Still Life mod as well pls, very important").

**What was read:**
- **Tectonic** and **Lithostitched**: their source on GitHub.
- **Terralith**: its source on GitHub, read once before we saw that its licence bans AI use (below). Only general ideas from it are used here, and none of its numbers.
- **Still Life** and **Lithosphere**: their Modrinth pages and gallery captions only. They publish no source.
- **Ours**: `dev` at `99f901df`, measured with `base_api`'s examples (no game runs).

## Licences first

| mod | licence | what we may do |
|---|---|---|
| [Tectonic](https://modrinth.com/mod/tectonic) (Apollo) | MIT (mod metadata and Modrinth; older branches carry MIT or CC0 files; the default branch has no LICENSE file) | reuse with credit; we reimplement anyway, since our terrain isn't Minecraft's density functions |
| [Lithostitched](https://github.com/Apollounknowndev/lithostitched) (Apollo) | MIT | the same; it's Tectonic's plumbing, nothing we need |
| [Terralith](https://github.com/Stardust-Labs-MC/Terralith) (Stardust Labs) | Stardust Labs License: no reuse of code or JSON, even with credit; reading to learn is allowed; **forbids use "to configure, test, debug, or augment any artificial intelligence"** | general ideas only, most of them vanilla Minecraft's own mechanisms; no numbers. Planner has flagged it for Ronald |
| [Still Life](https://modrinth.com/datapack/still-life) and its terrain pack [Lithosphere](https://modrinth.com/datapack/lithosphere) (Jayzet) | All Rights Reserved, no public source | ideas from their own descriptions and pictures, nothing more |

## What makes them good

**Tectonic (the land's shape).**
- **One distance from the river shapes everything near it.** A river is the zero line of a "ridges" noise. The same `|ridge|` value then decides:
  - the riverbed (`|ridge|` < 0.03, about 8 blocks under the sea)
  - the banks (out to 0.1)
  - the steps of the terraces: tiered plateaus at 0.3, 0.4 and 0.6, each tier's height varied by a noise

  So valleys, banks and plateaus always line up with the rivers.
- **Rivers reach the sea without any flow simulation**: the zero line keeps going through the coast's shapes.
- **Height comes from nested lookups** (curves of inland-ness × erosion × ridge), not from adding features. Mountains rise the further inland they are (offset 0.65 at the coast, 1.1 deep inland), so ranges have a continental logic.
- **Coasts by erosion**: low erosion makes cliffs, high erosion makes flat beaches. Oceans run along a noise's zero line too (`abs(noise)`), so the seas connect.
- **The climate picks the terrain kind**: a temperature band picks mesas, badland canyons, dunes or tiered plateaus, so the land and the biome agree.
- **Rivers carry on under the mountains as tunnels** at sea level, where the surface has none.
- **Detail is cheap and erosion-like**: the ridge detail noise is shifted by the slope, which gives gullies.
- **No dedicated lakes.** Only wetland flats with shallow ponds. Lakes are where Tectonic is weakest.

**Lithosphere + Still Life (the look Ronald named "very important").** From their pages:
- **Land (Lithosphere):**
  - ranges instead of pointy peaks, and much higher
  - rivers rarer than vanilla's but never cut off, carving valleys through the mountains
  - **big lakes anywhere, often with islands**
  - wide beaches that slope gently into the sea
  - deep oceans between continents, with archipelagos
  - realistic cliffs near mountains
  - karst towers (Monument Valley, Ha Long Bay)
  - smoother height changes
- **Biomes (Still Life):** a natural counterpart of every surface biome and many transition biomes, much denser plants, and no new blocks. Its gallery reads like a climate atlas: savanna river, old-growth temperate forest, cold steppe, desert wetlands, bog, lush and wooded tundra, warm temperate clearings, xeric shrubland, tropical wetlands, mountains by climate (cold temperate, warm temperate, arid).

**Terralith (biome placement), as general ideas** (vanilla's multi-noise system, extended):
- Each biome keeps to one temperature band, so cold land only touches cold.
- Variants split off on humidity, or on a "weirdness" noise within the band; rare biomes sit at its extremes.
- Rivers follow a weirdness zero line, and take their banks' temperature (frozen, temperate, warm).
- A thin coast band picks rocky shore, gravel beach or sand by erosion.
- Oases are the wet corner of the hot band.
- Steep slopes go bare, snow lies by biome, and badlands are banded by height.

## Other mods (Ronald: "have a look at other mods maybe too"; our scale, not Terralith's)

Licences are as each page states; ideas only, from the pages' own descriptions.

| mod | licence | what's worth taking |
|---|---|---|
| [William Wythers' Overhauled Overworld](https://modrinth.com/datapack/wwoo) | CC BY-NC-ND 4.0 (no derivatives) | sub-biomes inside every biome and transition biomes "to eliminate jarring shifts"; add-ons for navigable rivers and coves on cliff coasts |
| [Regions Unexplored](https://modrinth.com/mod/regions-unexplored) | MIT, source public | 64 biomes with many plants and blocks; more than we want (Ronald: not that massive) |
| [Geophilic](https://modrinth.com/datapack/geophilic) | All Rights Reserved (source viewable) | the closest to our scale: no new biomes, only better ones. Fallen trees, clearings, boulders, stumps, bushes and moss, and trees kept low enough to walk under. Most of it Voxel dog has, or is doing |
| Vintage Story's river mods: [Rivers](https://mods.vintagestory.at/rivers), [Watersheds](https://mods.vintagestory.at/show/mod/42299) | per mod, not checked | **rivers as a network of nodes**: segments with set sizes, linked to the sea, lakes fading into rivers. Watersheds paths streams uphill from lakes and seas over eroded ground. Our zero lines can't make tributaries meet; a node network could (step 2) |

## What we have (dev, seed 1, 8 km around spawn)

| | ours |
|---|---|
| land, sea | 79%, 20% |
| rivers | 1.3% of the area: valleys cut to the sea along a wandering noise's zero line, wider at the coast, narrowing and ending where they fade (inland, dry, near mountains); often loops; no tributaries |
| **lakes** | **0%**: all water sits at sea level, and inland basins are rare |
| land heights | half under y 85, nine tenths under y 137, peaks to about 250 |
| mountains | ridged noise in patches (16% of the land); snow caps above 146 / 162 / 180 by temperature; bare stone above y 130 |
| shapes | mesas (3 tiers) in the hot, dry land, highlands (+24) in temperate patches, dunes; rock faces in strata, slate, granite or limestone |
| biomes | 11, each now 2-20% of the land and within 700 blocks of spawn (`base_api::land::biome_shares`) |
| coasts | beaches where the land is within 1 block of the sea near the coast; no cliffs, no islands to speak of |

Maps (`cargo run -p base_api --example land_map`; left: the land before this autumn's terrain work, right: now):
- `terrain-terralith-tectonic/now_seed1_4096.jpg`: 4 km around spawn
- `terrain-terralith-tectonic/now_seed1_1024.jpg`: 1 km, a block a pixel

## Eight steps, ranked (lakes and rivers first)

Each step is ours, in `base_api::land` (numbers, shapes) and `mods/base/src/land.rs` (registration), built like this year's rivers: a `Feature::new` the engine sums, checked on the CPU first with the map tools.

| # | step | the idea from | effort | engine? |
|---|---|---|---|---|
| **1** | **Lakes in basins, with islands.** One lake site per 480-block cell, its chance set by the climate (many in the wet and the cold, few in the dry, oases rare). Its shoreline is a noise-wobbled blob 40-140 blocks across. The bed is carved below the sea's level and the banks rise smoothly to the land, so the water is the sea's own fill and costs nothing on the GPU or in distant terrain. Islands come where an island noise lifts the bed back above the water, and rivers passing near join them (both lie at the sea's level). **1b, mountain lakes** above the sea's level: the same basin with its water level set by the lowest point of its rim, filled by a decoration. | Lithosphere (big lakes with islands, anywhere); Tectonic's weakness | M (1), M (1b) | 1: no. 1b: distant terrain shows the basin dry until the engine has water levels per area (the LOD session could add a hook like `faces`) |
| **2** | **A river network that shapes its valleys.** Rivers rarer and longer, and tributaries that meet them: either a finer line that only exists near a main river (cheap), or Vintage Story's way, a network of nodes per large region linked down to the sea (truer, more work). Mountains leave room for them (the mountains' height falls toward the river line), so rivers run through valleys between ranges instead of ending at their feet. The same river distance shapes terraces and plateaus beside the river. | Tectonic (one river distance shapes banks and tiers); Lithosphere (rivers through mountains, never cut off) | M | no |
| **3** | **Coasts: wide gentle beaches, sea cliffs, islands.** A wider shelf, so beaches slope into the sea. Where an erosion field is low the coast is a cliff into the water (rock faces do the rest). Archipelagos in the seas where the continent noise dips to islands. | Tectonic (coasts by erosion), Lithosphere (beaches, archipelagos), Terralith (cliffs as a capped cut) | S-M | no |
| **4** | **An erosion field and continental ranges.** One large noise that says flat or rugged; it scales the hills, the mountains and the valley widths instead of today's separate patch masks. Mountains rise further from the coast, so ranges form a continent's spine rather than scattered patches. | Tectonic (erosion × inland-ness), Lithosphere ("cinematic" ranges) | M | no |
| **5** | **Shapes by climate band.** Tiered plateaus along rivers in the dry and temperate lands, badland canyons in the hottest, dunes with a steep lee side, karst towers in the warm and wet. | Tectonic (temperature band picks the shape), Lithosphere (karst) | M | no |
| **6** | **Transition and variant biomes.** A natural set near Still Life's atlas: steppe between plains and desert, bog and wetland in the wet lowlands, old-growth forest and clearings, lush tundra, desert wetlands (oases), warm and cold mountains. Each needs a real-place climate (`vitals::climate`), plants and some art. | Still Life (transitions, natural biomes), Terralith (variants within a band) | L | no |
| **7** | **Rivers under the mountains.** Where a river line meets a range, a tunnel at the sea's level keeps it going: a cave river with pillars. | Tectonic | M | maybe (carving under a surface is the engine's caves) |
| **8** | **Softer biome borders.** A small noise on the climate values when a rule picks a biome, so borders wander instead of following one smooth line. | Terralith and vanilla (nearest-box picking) | S | yes: the rules read the engine's climate fields |

**Order.** Do 1 and 2 first: they are what Ronald named, and they work together (lakes along rivers). Then 3 and 4 for the land's big shape. Then 5 and 6, which need art and climates. 7 and 8 last.

## Step 1 in more detail (lakes)

**Built** (claude/terrain-lakes, not yet seen in the game). Lakes are about 1% of the area on seeds 1-3 (1.15, 0.97, 1.13%), about as much as the rivers (1.25-1.3%). They are long and ragged, lie every way, and carry small islands. On the CPU they cost 0.3-0.4 µs a column (about 0.1 ms a chunk column), on top of about 0.5 ms for the other mods' features; timed on a loaded Mac, so roughly. Maps (left as before, right with lakes):
- `terrain-terralith-tectonic/lakes_seed1_4096.jpg`: 4 km around spawn
- `terrain-terralith-tectonic/lake_seed1_zoom.jpg`: one lake, a block a pixel

The numbers as built: a site per 420-block cell, kept with 0.8 times its climate's share; radius 36-130 blocks; the shore wobbled by up to the radius; 1-2.1 times longer than wide; 3-9 blocks deep. These differ from the plan below, which was the first guess.


- **Where:**
  - A site per `LAKE_CELL` cell (480 blocks), jittered.
  - Its chance is `LAKE_CHANCE` times a climate ramp: humid and cold more, dry less.
  - None in the sea, the mountains, or within a river's channel.
- **Shape:**
  - A radius between `LAKE_RADIUS` bounds, wobbled by a noise so no lake is round.
  - The bed falls to `LAKE_DEPTH` under the sea's level in a bowl.
  - The shore rises from the water's edge to the land over a band that widens the higher the land stands, the same rule as the river banks.
- **Islands:** inside the shore, an island noise above `LAKE_ISLANDS` lifts the bed back over the water (1-6 blocks), some with a tree.
- **Water:** the sea fills everything under its level, so a lake is wet from the first frame, on the GPU and in distant terrain. Its water is the land's biome, on its bed (`RIVER_BEDS`).
- **Cost:** a column's three-by-three cells are a few hashes, and the land's height is only rebuilt inside a lake, as the rivers do now.
- **Checks:**
  - A unit test that lakes cut only downward and that some reach under the sea.
  - `land_water` reporting their share. My aim is 1-2% of the land, about the rivers' share.
  - A before/after map.
  - A testkit script with `expect block` water at a lake and its island.

Sources: [Tectonic source](https://github.com/Apollounknowndev/tectonic) (default branch `rewrite-squared`, files under `src/common/main/resources/resourcepacks/tectonic/`), [Lithostitched](https://github.com/Apollounknowndev/lithostitched), [Terralith licence](https://github.com/Stardust-Labs-MC/Terralith/blob/1.20/license.txt), [Still Life](https://modrinth.com/datapack/still-life) and its [gallery](https://modrinth.com/datapack/still-life/gallery), [Lithosphere](https://modrinth.com/datapack/lithosphere).
