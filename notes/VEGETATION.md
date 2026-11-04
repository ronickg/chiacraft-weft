# Vegetation by biome: the layers

Vegetation, 2026-11-04. Ronald: "vegetation and how it's done is important too for the biomes, esp
when comparing to the reference images". Researcher's `reports/world-looks.md` item 2 ("Ground
cover per biome and by rule") and item 4 (groves and clearings) are this.

## What a real wood has, and what ours has

The references (`~/Coding/chiacraft-archive/world-looks/refs/`) read as places because they are
layered: WWOO's taiga clearing has a moss and grass floor, berry bushes, a fallen log, flowers,
shrubs at the wood's edge, then the spruces; the redwoods stand over ferns and low green. Ours
(Tester 2's `biome_looks1`, seed 1) jumps from one tall-grass shape straight to the canopy.

| layer | what it is | ours today |
|---|---|---|
| **Canopy** | the trees | built: branchy trees, stands and clearings, big, crooked, twin, fallen and stump forms |
| **Understory** | young and small trees under and between the big ones | none |
| **Shrubs** | woody bushes, 1-2 blocks: hazel, willow, juniper, creosote | only the six fruit bushes (berry, hazel, bramble, lingon-, cloud-, crowberry), as single cross plants at 0.4-1.2% |
| **Herbs** | grass, ferns, flowers, sedges | one tall-grass shape, 45 biome flowers, sedges, clover, lichen, mushrooms |
| **Floor** | what the ground itself shows: moss, leaf litter, needles, bare dirt | grass or podzol everywhere, gravel on steps |
| **Deadwood and rock** | fallen logs, snags, boulders | fallen logs and stumps (8% of forest trees), boulders |

## The layers per biome

Each from the real place its climate comes from (`notes/ECOLOGY.md`, "Each biome's real place").
*Placeholder* means it ships with an existing block's look until Chiacraft draws it.

| biome (place) | floor | herbs | shrubs | understory | canopy, deadwood |
|---|---|---|---|---|---|
| **plains** (Konza prairie) | bare dirt and grazed patches | **tall two-block grass** in patches (big bluestem), the prairie flowers | **thickets** of dogwood and sumac in patches (oak leaves, placeholder) | none | bur oaks, few; more along water (later) |
| **forest** (Białowieża) | **leaf litter** under the crowns, **moss** on the wet side | **ferns** (bracken) under trees, wood anemone in spring, mushrooms **round trunks** | hazel (bush exists), bramble, **leaf bushes** (hornbeam) | **young oaks and hornbeams** in gaps and at edges | oaks, birch; fallen logs, stumps, **snags** |
| **birch forest** (Moscow) | leaf litter, grass | ferns, lily of the valley, fireweed in clearings | **rowan and raspberry** (birch leaves, placeholder) | **young birches**, thick in clearings | birches; fallen logs |
| **taiga** (Fairbanks) | **moss carpet** and needles (podzol), lichen | lingon-, crowberry, few ferns, twinflower | **willow and alder** at edges and by water, **juniper** mats | **young spruces** 2-4 tall, many | spruces; **snags**, fallen spruce |
| **snowy plains** (Utqiaġvik) | lichen, moss, snowy turf | sedge, cottongrass, poppies | **dwarf willow and birch**, knee-high mats (placeholder leaves) | none | a few stunted spruces at the south edge |
| **desert** (Hermosillo) | sand, **gravel and coarse patches** | brittlebush, poppies, prickly pear | **creosote bush**, spaced evenly (mesquite leaves, placeholder) | none | mesquite and palo verde in washes; saguaro (cactus) |
| **savanna** (llanos) | bare dirt patches | **tall two-block grass**, thick | scattered **shrubs** (acacia leaves) | none | acacia, mesquite, moriche palms by streams |
| **swamp** (Pantanal) | moss and mud at the edges | cattail, wild rice, water hyacinth, reed (water plants branch) | **shrubs** on the higher ground | none | palm groves (capões) |
| **mountains** (Davos) | moss, gravel, rock | alpine flowers, alpenrose | **dwarf mountain pine** and **juniper** mats (spruce leaves) | young spruces at the tree line | spruces thinning with height; boulders |
| **sequoia grove** (Giant Forest, Worldgen's biome) | **needle litter** (podzol) | **sword fern**, lupine (placeholders) | **dogwood** (placeholder) | young sequoias and firs | giant sequoias; a **fallen giant** (later) |

**Bold** is new.

## How it's built (one machine, data per biome)

Everything is data on the biome, as `TREES` and `FLOWERS` are, so another mod sets it on its own
biomes. No engine change for any of it.

1. **Floor** (decoration): patches of a floor block by a noise, thicker under the crowns: `FLOOR`
   (a `BlockList`) and `FLOOR_COVER`. New full blocks `leaf_litter`, `moss`, `coarse_dirt`
   (plant soil, so grass and ferns still grow on them). **No extra faces**: they replace the top
   block.
2. **Herbs** (decoration): `base:fern` and a two-block `tall_grass` (lower and upper state), in
   the biome's `HERBS` with covers, taking part of the grass's share, so the plant count stays
   as it is. Ferns and mushrooms gather **round trunks** (the decoration sees the logs the trees
   put in the cube).
3. **Shrubs** (a scatter): `SHRUB_SITES`, a site per 4 x 4 cell at the biome's `SHRUBS` chance,
   the kind from `SHRUB_KINDS` (leaf blocks, weighted); a clump 1-2 high and 1-3 across, on a
   log stub or not. Thickets in patches by a noise (`.thin()`, so far terrain agrees).
4. **Understory** (a scatter): `young_tree_blocks` in `base_api::trees` (a trunk 2-4 tall under
   a small crown of the same leaves, a spire for spruces), a site per 5 x 5 cell at the biome's
   `YOUNG_TREES`, none deep in a stand: in clearings and at stand edges, where the light comes in.
5. **Deadwood:** a `Snag` form (a bare trunk 3-6 high with a branch stub), 3% of wild forest
   trees, beside the fallen and stump forms.

Built (2026-11-04, branch `vegetation-layers`): 1, 2 (ferns; the two-block grass waits for its
art), 3, 4, 5. Mushrooms round trunks are still to do.

## Cost

| part | frame cost | worldgen cost | art |
|---|---|---|---|
| floor blocks | none (a top block swapped) | one noise per column | 3 top faces: leaf litter, moss, coarse dirt (placeholders: podzol, mossy look, dirt) |
| ferns, tall grass | about none: they take part of the grass's share (each a cross of 2 quads) | none | fern (placeholder: the Cretaceous fern), two-block grass, sword fern, lupine |
| shrubs | small: a forest gets about 5 clumps per 16 x 16 at 4-12 leaf faces each (with leaves-cull), next to some 2,000 faces of crowns | one scatter (cell 4) | none first; willow, alder, juniper, creosote, dogwood leaves later |
| understory | small: 2-4 young trees per 16 x 16 in forests, about 20-40 faces each | rides the tree scatter | none |
| snags | none to speak of | none | none |
| distant terrain | shrubs and young trees stay out of far canopy (they would average down the trees' height) | | |

## Order

1. Floor, ferns and shrubs in the forest, birch forest and taiga first: they fill most shots.
   Tester 2 retakes `biome_looks1`'s spots for before and after.
2. Understory and snags.
3. Tall grass for the plains and savanna, the desert's creosote, the north's dwarf shrubs, the
   mountains' pine mats.
4. Chiacraft's art replaces the placeholders, batch by batch.
5. Later: trees thicker along water (gallery woods), meadows in dips (Hytale's rule), seasonal
   ground colour.

## Checks

- Every biome's covers leave room: herbs, wild plants and flowers sum under 0.9 of the soil.
- Every floor block is plant soil, and every shrub kind is a leaves block with its tint.
- A shrub stays inside its reach; a young tree inside the tree scatter's.
