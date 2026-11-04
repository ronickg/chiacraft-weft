# How the world looks: what pretty worlds do, and what we change

Researcher, 2026-11-04, round 7, for Planner and Ronald ("the biomes, trees etc all kinda still suck").

**What was read:**
- `dev` at `3bd02ddf` (worldgen, trees, wild plants, textures, the shader pack, LOD);
- the main checkout's trunk, whose world files match `dev`;
- the mods' and games' own pages (sources at the end).

**Pictures:**
- **References:** `world-looks/refs/` holds small copies of 12 reference pictures. The originals (28 MB) are archived in `~/Coding/chiacraft-archive/world-looks/refs/`. They are for internal reference only; no image licence was stated.
- **Ours:** `world-looks/ours/` holds shots from 3 November, as an interim "before". Tester 2 has a request for fixed-spot shots of six biomes at noon and at sunset, looking slightly down (`biome-shots.script` in the shared notes folder). They go in that folder when taken, and the same spots serve for every "after".

## Why their worlds look good, by technique

1. **Trees grown from branches, not stamped.**
   - **The recipe.** Vintage Story's trees are JSON recipes:
     - a trunk whose width shrinks each block (log while wide, then "branchy" leaves, then leaves at the tips);
     - angles that drift as it grows, and gravity that droops branches;
     - child branches at set spacings;
     - random size spread.

     A 2×2 trunk is just four trunk entries ([VS modding: trees](https://wiki.vintagestory.at/index.php/Modding:Trees)).
   - **Species by place.** Each species has a climate range and its own spacing: oak about 5 blocks, redwood 7 or more ([VS trees](https://wiki.vintagestory.at/Trees)).
   - **Dynamic Trees** grows trunks thick at the base, thinner when crowded, and lets leafless branches rot ([Dynamic Trees](https://modrinth.com/mod/dynamictrees)).
   - **The result:** no two silhouettes alike, and forests that read as forests.
   - Pictures: `refs/vintage_story_seasonal_foliage.jpg` (a thick, branching oak), `refs/regions_unexplored_redwoods.jpg`, `refs/dynamic_trees_oak_forest.jpg`.
2. **Ground that looks placed by a designer.** Hytale's new worldgen places props on rules:
   - meadows in dips ringed by small trees and boulders;
   - mushrooms and sticks round cedar trunks;
   - log bridges across streams;
   - grass only where there is open air above;
   - rocky surfaces on cliffs ([Hytale worldgen V2](https://hytale.com/news/2026/1/the-future-of-world-generation)).

   WWOO adds taiga clearings and transition bands between biomes ([WWOO](https://modrinth.com/datapack/wwoo)).
   - Pictures: `refs/wwoo_taiga_clearings.jpg` (fallen log, berry bushes, flowers, boulders, a lake, a rocky hill behind), `refs/hytale_worldgen_v2.jpg`.
3. **Terrain with large readable shapes.**
   - Tectonic shapes the land at continent scale: ranges tens of thousands of blocks long, plateaus in 2–3 tiers with walkable ramps, valleys, dunes, and wide rivers that join underground ones ([Tectonic](https://modrinth.com/datapack/tectonic)).
   - Terralith keeps to vanilla blocks and reviewers praise its cohesion. Two biome mods together give a "patchwork quilt" ([comparison](https://www.mc-mod.net/biomes-o-plenty-vs-terralith-world-generation-mods-compared/), a weak source).
   - Pictures: `refs/tectonic_range.jpg` (a snow-capped range beyond a taiga and meadow), `refs/tectonic_plateaus.jpg` (a red-desert butte and terraces), `refs/wwoo_forest_to_rocky_hills.jpg` (cliffs, a waterfall, a river).
4. **Breaking up the cube.**
   - Better Foliage puts extra leaf quads at 45° so leaf blocks lose their square outline, and short grass tufts on grass tops ([features](https://legacy.curseforge.com/minecraft/mc-mods/better-foliage/pages/features)).
   - Random texture variants hide tiling.
   - Pictures: `refs/better_foliage_leaves.jpg`, `refs/better_foliage_grass.jpg`.
5. **One restrained palette and graded light.**
   - Hytale's art guide asks for no pure black or white, coloured shadows (purple in their example), and painted AO rather than noise ([Hytale models](https://hytale.com/news/2025/12/an-introduction-to-making-models-for-hytale)).
   - Complementary Reimagined dulled its hues to keep the vanilla feel ([Complementary](https://www.complementary.dev/shaders/)).
   - Pictures: `refs/complementary_old_growth.jpg` (sunset through spruces, rays, reflections), `refs/complementary_style_comparison.jpg`.
6. **Seasons and motion.**
   - Vintage Story shifts foliage colour through the year, starting a month before each season and depending on latitude ([VS seasons](https://wiki.vintagestory.at/Seasons)).
   - Falling Leaves and Particle Rain add falling leaves, blown rain, dust and fog ([Falling Leaves](https://modrinth.com/mod/fallingleaves), [Particle Rain](https://modrinth.com/resourcepack/particle-rain)).

## Where we are

Our light is the strongest part: sun shadows, light shafts, reflections, ground fog, plants swaying in gusts. The world under it is what's weak:

| area | ours | file |
|---|---|---|
| **Trees** | All 13 kinds are one straight 1×1 trunk under one of 4 canopy shapes (round, cone, flat, palm). The only variety is height (4–9). Birch is drawn exactly like oak. No branches, thick trunks, big trees, dead or fallen trees. Crowns are at most 5–7 blocks across. | `mods/base/api/src/trees.rs:39-115` |
| **Tree placement** | One site per 5×5 cell, sideways reach 3, each cell rolling on its own. Forest density 0.4 is about 160 trees a hectare, which is close to a real mature stand (170–244/ha for Norway spruce, [study](https://annforsci.biomedcentral.com/articles/10.1007/s13595-011-0149-0/tables/1)). The count is right; what's missing is crown size and clumping (groves, clearings, edges). | `mods/base/src/worldgen.rs:229-243` |
| **Ground cover** | One tall grass shape. Flowers are only dandelion and rose: no biome sets `FLOWERS` (`worldgen.rs:28`, read at `:350`). 19 wild plants at low cover. No ferns, moss, leaf litter, rocks, boulders or overworld fallen logs (rotten logs exist only in the Cretaceous). | `worldgen.rs:250-382`, `vitals/src/wild.rs` |
| **Textures** | 270 tiles at 32 px in one style. No random variants, so grass tops visibly repeat (`ours/2026-11-03_grass_tiling_32px.jpg`). Grass sides are a thin lip over dirt, so every one-block step shows a red-brown stripe (`ours/2026-11-03_plains_noon_pack.jpg`). Weft's random grass is switched off (`mods/base/assets/minecraft/textures/block/tallgrass.ctm`: `# off`), and `notes/HANDOFF.md:56` notes it drew as white sticks. | `mods/base/assets/textures/block/` |
| **Terrain** | A heightmap on the GPU: continents, ±16-block hills, dunes, ridged mountains. Most land lies within about 34 blocks of sea level. No rivers (`canyons`, `ridge` and `plateaus` exist in `crates/noise/src/fixed.rs` but the overworld uses none). No snow caps (grass to y130, then bare stone). One stone. Biome borders are hard lines on climate contours, softened only by colour blending. Only 4 of 11 biomes differ in shape. | `terrain.wgsl:254-307`, `worldgen.rs:190-228` |
| **Colour and light** | A soft tone curve. No colour grade per biome or hour, no bloom. Shadows reach 80 blocks. Distant LOD terrain is flat average colours. Golden hour is a strong magenta sky over unchanged green ground (`ours/2026-11-03_plains_golden_hour_pack.jpg`). | `mods/base/shaderpack/look.wgsl:55-78` |
| **Seasons and motion** | No seasonal foliage colour, though the calendar exists. No falling leaves, pollen or fireflies. The engine already has per-block `ambient_particles` (torches use them). | `blocks.rs:532` |

## Ten changes, ranked by how much better it looks for the work

| # | change | owner | effort | engine? |
|---|---|---|---|---|
| 1 | **Trees grown from branches (Vintage Story's recipe).** A trunk width that falls each block, drifting angles, droop, child branches by level, size spread. Per species: oak broad and branchy, birch narrow and pale-trunked, spruce and fir conical with drooping tiers, acacia flat on a bent trunk, palms leaning. 2×2 trunks on big species, and a few big old trees per forest (Central European old growth has 10–17 large trees a hectare, [Serrahn](https://fis.leuphana.de/en/publications/structural-pattern-of-a-near-natural-beech-forest-fagus-sylvatica/)). Wider reach so crowns close. | Voxel dog | M | Trees over about 32 blocks need Weft (`sdk/terrain.rs:558`); the rest is base |
| 2 | **Ground cover per biome and by rule.** Flower sets per biome (`FLOWERS`); ferns and moss under trees; boulders and fallen logs (reuse the Cretaceous rotten log); patches of coarse dirt, gravel and moss by noise and slope; meadows in dips with saplings at forest edges; berries and mushrooms around trunks. | Voxel dog | S–M | no |
| 3 | **Break the tiling and the stripes.** 3–4 random variants of grass top, dirt, stone and sand. 2–3 tall-grass shapes and short tufts. A grass side whose lip covers more, or follows the top on steps (Better Foliage's "connected grass"). The art is Chiacraft's (block-textures). Random variants need Weft's CTM white-sticks bug checked, or variants placed as separate blocks in decorate. | Chiacraft, + Weft if CTM | S–M | maybe (the CTM bug) |
| 4 | **Groves, clearings and edges.** A low-frequency noise on tree density, so forests clump, open into glades, and thin to scattered trees at their edges and on slopes. Young and dead trees for age. | Voxel dog | S | no |
| 5 | **Rivers, plateaus, snow caps, rock faces.** Use the unused `canyons`/`ridge` noise for rivers, and `plateaus` for mesas in the dry lands. Snow on mountains by height and temperature. Bare rock on steep faces. Two or three stone kinds in patches. Hills up to ±30 in the right biomes. | **a new worldgen session** | M | no (`Feature::new` in base); 3D overhangs would be Weft |
| 6 | **Colour grade.** Shadows tinted cool and never black. A warm grade at golden hour that reaches the ground, not only the sky. Saturation held back. A little bloom on the sun and lit blocks, if a cheap post pass allows. | Lighting | S (grade), M (bloom) | bloom done properly is a Weft pass |
| 7 | **Seasonal foliage.** Foliage and grass tint follow the calendar: spring fresh, summer deeper, autumn yellow, orange and red by species (birch yellow, oak brown, maple red), winter bare or grey. Uses the existing calendar and tint blending. | Lighting (tint) + Voxel dog (per-species colours) | M | no |
| 8 | **Motion: falling leaves and small life.** `ambient_particles` on leaf blocks (more in autumn and wind), pollen over meadows, fireflies at dusk in summer, dust in dry wind. Mostly data. | Voxel dog (data), Lighting (looks) | S | no |
| 9 | **Leaves that aren't cubes.** Extra cut-out quads at 45° on leaf blocks (Better Foliage), or a fluffy leaf model, to lose the square outline. Watch vertex counts. | Chiacraft (art) + Weft mesher | M | yes |
| 10 | **The far distance.** Shadows past 80 blocks (cascades), and LOD tiles that carry light and some texture instead of flat average colours. | Lighting + Weft | L | yes |

**Order.** Do 1, 2, 3 and 4 together: they are the forest and the ground, which fill every shot, and only 3 might touch the engine. Then 6 and 7 make the same world read better through the day and the year. Then 5, for the land itself, when a worldgen session starts. 9 and 10 last.

**The check.** Tester 2's six-biome, noon-and-sunset shots from fixed spots are the "before". The same script and spots give the "after" for each change.

## Sources

- Vintage Story: [modding: trees](https://wiki.vintagestory.at/index.php/Modding:Trees), [trees](https://wiki.vintagestory.at/Trees), [seasons](https://wiki.vintagestory.at/Seasons), [v1.13 blog](https://www.vintagestory.at/blog.html/news/the-seasons-update-v113-pre1-r251/).
- Mods: [Dynamic Trees](https://modrinth.com/mod/dynamictrees), [Tectonic](https://modrinth.com/datapack/tectonic), [Terralith](https://modrinth.com/datapack/terralith), [WWOO](https://modrinth.com/datapack/wwoo), [Regions Unexplored](https://modrinth.com/mod/regions-unexplored), [Better Foliage](https://legacy.curseforge.com/minecraft/mc-mods/better-foliage/pages/features), [Falling Leaves](https://modrinth.com/mod/fallingleaves), [Particle Rain](https://modrinth.com/resourcepack/particle-rain), [Complementary](https://www.complementary.dev/shaders/).
- Hytale: [worldgen V2](https://hytale.com/news/2026/1/the-future-of-world-generation), [models and art](https://hytale.com/news/2025/12/an-introduction-to-making-models-for-hytale).
- Forests: [Norway spruce stand densities](https://annforsci.biomedcentral.com/articles/10.1007/s13595-011-0149-0/tables/1), [Serrahn beech old growth](https://fis.leuphana.de/en/publications/structural-pattern-of-a-near-natural-beech-forest-fagus-sylvatica/).
