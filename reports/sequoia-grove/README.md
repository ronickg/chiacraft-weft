# Giant sequoia groves (`base:sequoia_grove`)

Worldgen, 2026-11-04. Ronald: "also want to have a biome with the massive Giant sequoia as i find them amazing".

## The real groves
- **Where:** about 75 groves along the Sierra Nevada's west slope, between 1,400 and 2,100 m. They're patches of 1 to some 1,000 ha in a belt of mixed conifers.
- **Ground:** moist draws and benches with deep, well-drained soil, where snowmelt keeps the ground wet into the dry summer.
- **Climate:** cool snowy winters and dry summers. Giant Forest (1,721 m): 10.4 °C a year, 3.3 °C in January and 20.2 °C in July. About 1,050 mm of precipitation, four fifths of it from November to March, much of it snow (Lodgepole: 5.6 m a year). The numbers are in `reports/ecology/Biome climates.md` and `vitals::climate`.
- **The trees:**
  - The giants themselves: trunks 6-8 m thick, 50-85 m tall, cinnamon-red fibrous bark, crowns only on the upper part.
  - Around them: white fir, sugar pine, incense cedar, black oak.
  - Under them: Pacific dogwood (white bracts in May), mountain misery, bracken fern, lupine.
  - Open patches: meadows, and clearings where a giant fell.

## In the game

| | |
|---|---|
| climate | `vitals::climate` `base:sequoia_grove`: Giant Forest's numbers (the research in `reports/ecology/biome_climates.json`) |
| where | `base_api::land::SEQUOIA_AREA` above `SEQUOIA_GROVE`. These are grove patches about 260 blocks across, on the mountains' front slopes: their area noise is just short of the mountains biome. It must be temperate (`SEQUOIA_TEMPERATURE`, -0.12 to 0.1), not dry (humidity `SEQUOIA_HUMIDITY` and up), and 28-85 blocks over the sea (`SEQUOIA_RISE`) |
| how much | 0.6-0.7% of the land on seeds 1-3, the nearest 600-800 blocks from spawn. Rare and grove-sized, as the real ones are, but findable |
| rule | `mods/base/src/land.rs`, after the mountains; the marker feature `sequoia_groves` has no height, only its mask |
| biome | `mods/base/src/worldgen.rs`: grass over 5 blocks of dirt (deep soil), forest and animals tags. Trees 0.12 a 5×5 cell with stands at 0.1; spruce and oak stand in until the sequoia comes |
| forage | `vitals::forage`: turf, spruce browse and grass seed like the mountains' woods, grass cover 0.06 |
| maps | `groves_seed{1,2,3}_6km.jpg` (rust red: groves; left: the land before the lakes, rivers and mesas), `grove_seed1_zoom.jpg` (one grove, a block a pixel) |

## What's still to do, and who does it

**Voxel dog: the tree and the ground.**
- A `base:sequoia_log` tree, the grove's emblem.
  - Trunk 3×3 to 5×5 blocks, flared at the foot.
  - Bare for its lower half or more, with a narrow crown high up.
  - As tall as the engine allows. Trees over about 32 blocks need Weft (`sdk/terrain.rs` scatter reach), so the full height is an engine question.
  - One giant per 2-4 sites, among firs and pines. `TREE_KINDS` stands in today.
- Clearings with ferns (bracken), Pacific dogwood (a small tree or tall shrub, white in spring), lupine, and fallen giants as long logs.
- A meadow now and then (`base_api::land::bank` and the lakes give wet ground).

**Chiacraft: the art.**
- Sequoia bark: cinnamon red-brown, deep vertical fissures.
- A log top with thick rings.
- Fine blue-green scale foliage.
- The fern, the dogwood's white bracts, lupine.

**The animals session: the food web.**
- `animals_core::web` checks every biome's web is whole: plants, plant eaters, hunters, something for the dead.
- The grove needs habitats in `species.rs`. The real ones: mule deer, Douglas squirrel, Steller's jay, great horned owl, raven, coyote, gray fox, black bear. From ours: red deer, field vole, small birds, great horned owl, raven, coyote, red fox, grey wolf, turkey vulture.
- Until then the web check fails on CI, so the grove shouldn't merge before the habitats.
- `mods/animals/SPECIES.md` is regenerated with them.
