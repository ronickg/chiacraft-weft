# Tree looks: how the good block-game tree generators make trees read as their species

Research for chiacraft's tree generator, which is modelled on Vintage Story's treegen.
Date: 2026-11-04. Every claim has its source inline. Numbers come from the source code or data
files where we could read them. "px" means 1/16 of a block.

Contents:

1. Vintage Story treegen (most important for us)
2. Dynamic Trees
3. Vanilla Minecraft
4. Biomes O' Plenty
5. Terralith
6. Oh The Biomes We've Gone (BYG)
7. Regions Unexplored
8. William Wythers' Overhauled Overworld (WWOO)
9. Thin branch blocks
10. What makes a block tree read as its species
11. Common failure looks and how the good mods avoid them
12. Species references

A note on sources. Vintage Story's game assets are not on GitHub from the studio. The treegen
JSON below is from an unofficial copy of the 1.20.1 assets
(https://github.com/TheDarkLord777/vintagestory/tree/main/assets/survival/worldgen/treegen).
The engine code is the studio's own (https://github.com/anegostudios/vsessentialsmod). Minecraft's
code is not public. Its tree numbers come from the game's data files as published by misode/mcmeta
(https://github.com/misode/mcmeta/tree/data/data/minecraft/worldgen/feature), and the algorithms
from Obsidian, an open-source C# port (https://github.com/ObsidianMC/Obsidian/tree/26.3.x/Obsidian/WorldData/Generators/Features/Trees).

---

## 1. Vintage Story treegen

### How the generator works

- Each tree is one JSON file in `worldgen/treegen` holding `trunks`, `branches` and `treeBlocks`
  (https://wiki.vintagestory.at/index.php/Modding:Trees).
- A branch is a walk. Each step takes `widthloss` off the branch's width and places one block. The
  walk ends when the width falls below `dieAt`. So a branch's length is about `width / widthloss`
  blocks (https://github.com/anegostudios/vsessentialsmod/blob/master/Systems/WorldGen/Standard/ChunkGen/8.GenVegetationAndPatches/Treegen/TreeGen.cs).
- "Width" is a budget, not a drawn thickness. Every step places exactly one block. A thick trunk
  needs several trunk entries side by side (the 2x2 old oak, large kapok and redwood do this)
  (same TreeGen.cs; https://wiki.vintagestory.at/index.php/Modding:Trees).
- Wood or leaves comes from the width at that step. Width ≥ 0.3 places a log. Below 0.3 it places
  leaves. With the default `leafLevelFactor` of 5, width 0.1-0.3 gives "branchy leaves" (leaves with
  twigs) and width under 0.1 gives plain leaves
  (https://github.com/anegostudios/vsessentialsmod/blob/master/Systems/WorldGen/Standard/ChunkGen/8.GenVegetationAndPatches/Treegen/TreegenBranch.cs,
  https://github.com/anegostudios/vsessentialsmod/blob/master/Systems/WorldGen/Standard/ChunkGen/8.GenVegetationAndPatches/Treegen/TreeGenBlocks.cs).
  So every branch tip runs log → branchy leaves → leaves. Leaves only exist at the ends of wood.
- `dieAt` 0.1 removes the plain leaves; 0.3 removes all leaves
  (https://wiki.vintagestory.at/index.php/Modding:Trees).
- Angles are radians. `angleVert` 0 is straight up and 1.57 is level. `branchVerticalAngle` is
  absolute, not added to the parent's angle. `branchHorizontalAngle` is added to the parent's
  heading (https://wiki.vintagestory.at/index.php/Modding:Trees).
- `gravityDrag` pulls a branch down in proportion to how far it is from the trunk sideways. It
  does nothing to vertical branches. A negative value bends tips up (TreeGen.cs above; larch below).
- Side branches start at `branchStart` (a fraction of the parent's length). The next fork comes when
  the walked fraction passes the last fork plus `branchSpacing × (1 - fraction)`. Spacing 0 means a
  fork at every step. `branchQuantity` branches sprout at each fork, kept at least
  `var/5` radians apart (TreeGen.cs).
- `*Evolve` fields change a value along the branch. "sequence" runs 0 → 1 from base to tip.
  `linear` is `first + factor × seq`. `quadratic` is `first + sign(factor) × (factor × seq)²`.
  `sinus` is `first + sin(factor × seq)`
  (https://github.com/anegostudios/vsapi/blob/master/Math/EvolvingNatFloat.cs).
- `branchWidthLossMul` multiplies the parent's width each time it forks. Below 1 the parent thins
  at each fork, which gives forking (dichotomous) crowns (TreeGen.cs).
- The `branches` array is per depth. Entry 0 is for the trunk's children, entry 1 for theirs, and
  the last entry repeats. `branchStart` ≥ 1 stops further forking
  (https://wiki.vintagestory.at/index.php/Modding:Trees).
- Size per tree: `size = treeGenParams.size × sizeMultiplier + sizeVar`. The world picks
  `size` between `MinSize` (default 0.2) and `MaxSize` (default 1), plus up to
  `SuitabilitySizeBonus` (default 0.5) where the climate suits the species best
  (https://github.com/anegostudios/vsessentialsmod/blob/master/Systems/WorldGen/Standard/ChunkGen/8.GenVegetationAndPatches/Tree/TreeVariant.cs,
  https://github.com/anegostudios/vsessentialsmod/blob/master/Systems/WorldGen/Standard/ChunkGen/8.GenVegetationAndPatches/Treegen/TreeSupplier.cs).
- Each tree draws a `treeSubType` from 8. It picks one of up to 7 "grown" leaf variants for the
  whole tree, so neighbours differ in leaf texture (TreeGen.cs, TreeGenBlocks.cs).
- Moss decor goes on the shady side of logs: the north face in the northern hemisphere. Vines hang
  from blocks by chance, up to about 11 long (TreeGen.cs).
- A tree stops if it would grow into a non-tree, non-soil, non-leaf solid block. Logs may replace
  soil. Redwood trunks are exempt, so they do not come out as three-quarter trunks (TreeGen.cs).
- `yOffset` sinks a tree into the ground. Old oak uses -1 and redwood -2, so wide bases do not hang
  over slopes (https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/oldenglishoak.json,
  https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/redwoodpine.json).

### Real values per species (VS 1.20.1)

Abbreviations: wl = widthloss, bS = branchStart, bSp = branchSpacing, bVA = branchVerticalAngle,
bQ = branchQuantity, bWM = branchWidthMultiplier, gD = gravityDrag. "a±v" = avg ± var.

| Tree | Trunk | Branch level 1 | What it gives |
|---|---|---|---|
| English oak, sizeMult 1.2 ([file](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/englishoak.json)) | wl 0.1; bS 0.3±0.05; bSp 0; bVA 1.32; bQ 10, evolve linear +3; bWM 0.1, evolve linear 1.4 | wl 0.08; gD 0; bS 0; bSp 0.01; bVA 1.1±0.2; bQ 1; bWM 0.5. Level 2: bS 2 (no more forks) | Short bole (30%), then a fork at every step. Lower limbs are thin, upper ones near trunk-thick, so the trunk dissolves into the crown. |
| Old English oak, sizeMult 0.86, yOffset -1 ([file](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/oldenglishoak.json)) | 4 trunks in a 2x2 (dx/dz 0.5/1.5). wl 0.05; dieAt 0.14±0.04 for one, 0.2±0.1 for the others; bS 0.25; bSp 0±0.15; bVA 0.5±0.2 gaussian; bWM 0.3±0.1; bQ 1.5, evolve sinus 3; angleVertEvolve quadratic 2 | wl 0.04; bS 0.9±0.1; bVA 1.57±1; bHA 0±1.57; bQ 2; bWM 0.7. Level 2: bS 5, wl 0.1, dieAt 0.3 | Four leaders that lean further out as they rise and stop at different heights. Long, steep limbs. Size 1.4-1.8 in its own world entry ([props](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treengenproperties.json)). |
| Silver birch, sizeMult 1 ([file](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/silverbirch.json)) | wl 0.06; dieAt 0.001; bS 0.2±0.2; bSp 0.2±0.1; bVA 0.8±0.1; bQ 5; bWM 0.15, evolve linear 1.5; branchWidthLossMul 0.98 | wl 0.06; gD 0.1; dieAt 0.001; bS 0.1; bSp 0.1; bVA 0.5±0.2 inverse-gaussian; bQ 1; bWM 0.6 | Leader runs to the top (excurrent). Steep 46° branches in whorls of 5 that droop at the tips. |
| Scots pine ([file](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/scotspine.json)) | wl 0.044; dieAt 0.05; bS 0.4±0.2; bSp 0.15; bVA 1.57±0.3; bQ 4, evolve quadratic 0.2; bWM 0.4, evolve linear 0.2 | wl 0.036; gD 0.08; bS 1.25 (no sub-branches); bVA 1.57±0.5; bWM 0.4 | Tall bare bole, level whorls in the top 60%, slight droop. 1% resin logs. |
| Larch, sizeMult 0.7 ([file](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/larch.json)) | As Scots pine, but bS 0.2±0.1 and bQ 8 | As pine, but gD **-0.06** | Lower start, twice the whorl count, branch tips turn up. |
| Redwood, sizeMult 1.82, yOffset -2 ([file](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/redwoodpine.json)) | 4 trunks in a 2x2, each a quarter-section log (`logsection-grown-redwood-ne/se/sw/nw`). wl 0.047; main dieAt 0.05, others 0.2±0.1; bS 0.45±0.2; bSp 0.15; bVA 1.57±0.3; bQ 2, evolve quadratic 0.3; bWM 0.3, evolve 0.4 | wl 0.04; gD 0.04; bS 1.25; bWM 0.4 | A thick round trunk that narrows to one leader as three quarters stop early. Short level branches. |
| Kapok, sizeMult 1.4, sizeVar -0.1±0.2 ([file](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/kapok.json)) | Same numbers as silver birch | Same as birch | Birch's shape, scaled up 1.4x. |
| Large kapok ([file](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/largekapok.json)) | Kapok ×4 trunks at dx/dz 0/1; dieAt 0.001, 0.1, 0.2, 0.3 | As kapok | Four trunks that stop one after another: a flared, buttressed base under one leader. Fixed size 1.8. |
| Bald cypress ([file](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/baldcypress.json)) | wl 0.06; bS 0.3±0.05; bSp 0.1; bVA 1.2±0.1; bQ 0.5, evolve quadratic 0.1; bWM 0.1, evolve linear 1 | wl 0.03 (long); gD 0.12; bS 0.95; bSp 0.5; bVA 1.57±0.5; angleVertEvolve linear -1; bWM 0.4 | Sparse long limbs that sag, then curve up at the ends. Size 0.5-0.85. |
| Acacia ([file](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/acacia.json)) | 4 identical trunks from one spot. angleVert 0±0.1 inverse-gaussian, angleVertEvolve quadratic 2.2; wl 0.06; bS 0.5; bQ 10, evolve +3; bWM **1**; branchWidthLossMul **0.5** | wl 0.06; gD 0; bS 0.2±0.1; bSp 0.4; bVA 1.2±0.2; bHA inverse-gaussian ±π; bQ 6, evolve +1; bWM 0.8 | Stems lean out more and more with height (a vase). Each fork halves the parent and gives children of equal width, so it forks instead of branching. The crown is flat. |
| Crimson king maple ([file](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/crimsonkingmaple.json)) | Birch numbers, but bS 0.15±0.05; bVA 1.0; bWM 0.2, evolve 1.3; no branch spacing | As birch | Lower, wider-spreading crown than birch from three changed numbers. |
| Green spire cypress ([file](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treegen/greenspirecypress.json)) | widthMultiplier 0.36; randomWidthloss 0.044±0.01; widthlossCurve 0.94; bQ 0; `leavesLevels: 5` | none | One thin stem dressed in 5 leaf-block sizes chosen by width: a narrow spire with no branches. |

Lessons from these files:

- Many species share one template. Birch, kapok and crimson king maple differ in 3-5 numbers.
  A third-party Vintage Story mod makes its willow from English oak plus `gravityDrag 0.2`, and its
  ginkgo from bald cypress numbers
  (https://github.com/captanredbeard/Biodiversity/blob/master/bdtree/assets/bdtree/worldgen/treegen/willow.json,
  https://github.com/captanredbeard/Biodiversity/blob/master/bdtree/assets/bdtree/worldgen/treegen/ginkgo.json).
- Character comes from a few levers: where branching starts, branch angle, forks per step, how
  child width changes with height (`branchWidthMultiplierEvolve`), and droop (`gravityDrag`).
- Young and old: Vintage Story mostly scales one shape by `size`. Old oak is a separate generator.
  Small English oak (size 0.2-0.3) and small kapok (0.1-0.2) are extra entries for understory
  ([props](https://github.com/TheDarkLord777/vintagestory/blob/main/assets/survival/worldgen/treengenproperties.json)).

---

## 2. Dynamic Trees (Minecraft mod)

Trees grow over time. A grow "signal" walks from the root to a tip, then thickens branches on its
way back. Source: https://github.com/DynamicTreesTeam/DynamicTrees (branch `release/1.21.1`).

### Growth logic

- The signal starts with `signal_energy` and loses 1 per step. That caps the tree's reach from
  the root (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/GrowSignal.java).
- At each step it picks a direction from a weighted map. Up gets `up_probability`. Each neighbour
  adds air 1, leaves 2, and own branch "radius + 2", so signals follow existing thick limbs. The
  signal goes straight up until `lowest_branch_height`
  (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/growthlogic/GrowthLogicKit.java,
  https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/block/branch/BasicBranchBlock.java).
- Radius uses a pipe model. On the way back, each block's radius = √(sum of the squared radii of
  the branches feeding it) + `tapering`. It is clamped so it never shrinks, and capped by
  `max_branch_radius`. Radius is in px: 1 is a twig and 8 is a full block. Thick trunks go to 24,
  which is 3x3 (BasicBranchBlock.java; https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/block/branch/ThickBranchBlock.java).
- Growth logic kits per species:
  - **Conifer**: a signal that turns out of the trunk keeps only energy ÷ `energy_divisor`
    (default 3, mega spruce 5), capped at `horizontal_limiter` 16. That makes a cone. The trunk may
    turn only on odd steps where radius > 1, so whorls come every 2 blocks. Height varies by a
    position hash mod `height_variation` 5
    (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/growthlogic/ConiferLogic.java).
  - **Dark oak**: up weight is fixed at 4. Once out of the trunk, branches may not turn up or down
    (flat limbs). The current direction is weighted × 0.35, so limbs zig-zag. Sideways push grows
    with height. A branch must get 2 blocks clear of the trunk first. The trunk stops at lowest
    branch height + 5 (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/growthlogic/DarkOakLogic.java).
  - **Jungle**: the trunk branches only every `branch_out_chance` steps (5; mega jungle 3).
    Side branches get only 4 energy. Above `canopy_height` (14 + hash mod 8; mega 25) any direction
    is allowed, which makes an umbrella crown
    (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/growthlogic/JungleLogic.java).
  - **Palm**: up weight 10 and no turns. Height varies by hash mod 3
    (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/growthlogic/PalmGrowthLogic.java).
  - **Mangrove roots**: roots grow down using `up_probability`, sideways weight 1, never up, and
    never down through the trunk centre
    (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/growthlogic/MangroveRootsLogic.java).

### Species numbers (from `trees/dynamictrees/species/*.json`)

| Species | tapering | signal_energy | up_probability | lowest_branch_height | logic |
|---|---|---|---|---|---|
| oak | 0.3 | 12 | (default) | (default) | default |
| birch | 0.1 | 14 | 4 | 4 | default |
| spruce | 0.25 | 16 | 3 | 3 | conifer |
| mega spruce | 0.25 | 24 | 7 | 5 | conifer, divisor 5 |
| dark oak | 0.3 | 18 | 4 | 6 | dark_oak, max radius 24 |
| jungle | 0.2 | 24 | 3 | 2 | jungle |
| mega jungle | 0.17 | 35 | 10 | 8 | jungle, canopy 25 |
| acacia | 0.15 | 12 | 0 | 3 | default |
| cherry | 0.3 | 12 | 1 | 4 | default |
| mangrove | 0.25 (roots 0.2) | 20 | 8 | 2 | roots kit |

Source folder: https://github.com/DynamicTreesTeam/DynamicTrees/tree/release/1.21.1/common/src/main/resources/trees/dynamictrees/species.
Birch's low tapering (0.1) gives a thin trunk. Acacia's `up_probability` of 0 gives wandering
stems.

### Leaves ("cell kits")

- Leaves grow only from radius-1 twigs. A twig is a cell with "hydration" 5. Leaves spread from it
  as a cellular automaton, and a cell with no hydration dies. So leaves never float far from wood
  (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/cell/CellKits.java).
- Each kit maps hydration per face. Conifer leaf cells pass less sideways (3 → 0, 4 → 2), which makes
  thin layered skirts. Acacia cells pass almost nothing upward and pass everything sideways, which
  makes flat pads. Dark oak cells cap up and down at 3, which makes wide slabs
  (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/cell/ConiferLeafCell.java,
  https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/cell/AcaciaLeafCell.java,
  https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/cell/DarkOakLeafCell.java).
- Leaf cluster shapes used at worldgen: deciduous 5×4×5, conifer 5×2×5, acacia 7×2×7 (flat), dark
  oak 7×5×7 (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/cell/LeafClusters.java).
- `smother` limits leaf depth: spruce 3, dark oak 3, acacia 2, cherry 5
  (https://github.com/DynamicTreesTeam/DynamicTrees/tree/release/1.21.1/common/src/main/resources/trees/dynamictrees/leaves_properties).

### Worldgen with JoCodes

- A JoCode is a recorded tree shape as a base64 string. It can be rotated to any of 4 facings
  (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/worldgen/JoCode.java).
- Each species ships 16 codes for each spacing radius 2-8 (dark oak 20). Bigger radius means more
  room, so a bigger tree. Leaves are then rebuilt and "aged" on the spot
  (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/resources/trees/dynamictrees/jo_codes/oak.txt; JoCode.java).
- A "careful" mode checks for neighbouring branches so two trees do not fuse into "frankentrees"
  (JoCode.java comment).

### Roots and base

- A "rooty dirt" block under the trunk is the tree's root and soil (JoCode.java places it first).
- Surface roots, for families with `generate_surface_root` (dark oak, jungle): only on trunks of
  radius ≥ 13. At most 2 blocks per level. Root radius scales with trunk radius / 24
  (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/genfeature/RootsGenFeature.java,
  https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/resources/trees/dynamictrees/families/dark_oak.json).
- Bottom flare: if the radius 3 blocks up is above 6, the block 2 up gets +1 and the block 1 up
  gets +2 (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/genfeature/BottomFlareGenFeature.java).
- Mega spruce and dark oak also use a "mound" of soil and podzol under the canopy
  (species JSON above).

Young vs grown: the same rules run every growth tick, so a young tree is a real earlier stage, not a
scaled copy. Pictures: https://modrinth.com/mod/dynamictrees/gallery.

---

## 3. Vanilla Minecraft

Numbers are from the data files at https://github.com/misode/mcmeta/tree/data/data/minecraft/worldgen/feature.
Algorithms are from the Obsidian port linked at the top.

- **Fancy (large) oak**: trunk height = 3 + rand(0..11), then +2. The bole goes to 0.618 of that
  height. Foliage clusters sit at each height from top-5 down to 0.3 × height, one per level. Each
  cluster lies at a random angle, at distance shape × (rand + 0.328). The shape is a half-circle
  profile, so the crown is round. A limb runs from the trunk up to each cluster with slope 0.381.
  The limb is placed only if its path is clear and its base is at least 0.2 × height. Each
  cluster is a 5-row disc of radius 2/3/3/3/2, tested as (x+0.5)²+(z+0.5)² ≤ r²
  (https://github.com/misode/mcmeta/blob/data/data/minecraft/worldgen/feature/fancy_oak.json,
  https://github.com/ObsidianMC/Obsidian/blob/26.3.x/Obsidian/WorldData/Generators/Features/Trees/BranchingTrunkPlacers.cs,
  https://github.com/ObsidianMC/Obsidian/blob/26.3.x/Obsidian/WorldData/Generators/Features/Trees/FoliagePlacers.cs).
- **Dark oak**: 2×2 trunk, base 6 + rand 2 + rand 1. Near the top it leans 0-2 steps one way. Each
  of the 12 cells around the 2x2 has a 1/3 chance of a 2-4 log stub hanging from the top, each with
  its own foliage. The foliage is a flat slab with rows of radius +2, +3, +2 and sometimes a small
  top. The outer corners are cut. Dirt is always placed under the trunk
  (https://github.com/misode/mcmeta/blob/data/data/minecraft/worldgen/feature/dark_oak.json,
  https://github.com/ObsidianMC/Obsidian/blob/26.3.x/Obsidian/WorldData/Generators/Features/Trees/TrunkPlacer.cs;
  https://minecraft.wiki/w/Tree).
- **Mega jungle**: 2×2 trunk, base 10 + rand 2 + rand 19. Branches start 2-5 below the top and
  come every 2-5 blocks down to half height. Each is 5 logs out at a random angle, rising 1 per 2,
  and ends in a foliage pad whose rows widen downward. Trunk vines, and leaf vines at 25%. Small
  jungle trees get cocoa at 20%
  (https://github.com/misode/mcmeta/blob/data/data/minecraft/worldgen/feature/mega_jungle_tree.json,
  https://github.com/misode/mcmeta/blob/data/data/minecraft/worldgen/feature/jungle_tree.json; TrunkPlacer.cs).
- **Spruce**: straight trunk 5 + rand 2 + rand 1. The foliage radius climbs 0 → 1, resets, then
  climbs to one more each cycle (up to 2-3). That makes stacked skirts. Corners are cut and 1-2
  bare trunk blocks are left at the bottom. **Pine** is a thin cone of radius 1 + rand
  (https://github.com/misode/mcmeta/blob/data/data/minecraft/worldgen/feature/spruce.json,
  https://github.com/misode/mcmeta/blob/data/data/minecraft/worldgen/feature/pine.json; FoliagePlacers.cs).
- **Mega spruce / mega pine**: 2×2 trunk, base 13 + rand 2 + rand 14. The crown is 13-17 tall
  (pine 3-7) and widens by floor(depth / height × 3.5), with a bump every other row. Podzol is put
  under the tree
  (https://github.com/misode/mcmeta/blob/data/data/minecraft/worldgen/feature/mega_spruce.json,
  https://github.com/misode/mcmeta/blob/data/data/minecraft/worldgen/feature/mega_pine.json).
- **Cherry**: trunk 7 + rand 1. It has 1, 2 or 3 branches (equal odds), starting 3-4 below the top.
  The branches go opposite ways, 2-4 out, ending at the top or 1 below. Each step picks "up" with
  odds of remaining height / remaining distance, so limbs curve diagonally. Foliage: radius 4,
  height 5, hanging leaves at 1/6 with 1/3 chance to extend, 25% holes at corners and on the wide
  bottom row
  (https://github.com/misode/mcmeta/blob/data/data/minecraft/worldgen/feature/cherry.json; BranchingTrunkPlacers.cs).
- **Acacia**: forking trunk 5 + rand 2 + rand 2. It leans 1-3 steps starting 1-4 below the top. An
  optional second limb goes another way for 1-3 blocks. Foliage is a flat 2-layer pad
  (https://github.com/misode/mcmeta/blob/data/data/minecraft/worldgen/feature/acacia.json; TrunkPlacer.cs).
- **Mangrove**: trunk 2 + rand 1 + rand 4. Each log has a 50% chance of a branch that climbs
  diagonally 1-4 steps. The trunk is lifted 1-3 blocks on roots. Roots reach up to 15 long and 8
  wide, skew 20% of the time, and turn mud into muddy roots. Moss carpet at 50% on roots
  (https://github.com/misode/mcmeta/blob/data/data/minecraft/worldgen/feature/mangrove.json).
- Leaves more than 6 blocks (taxicab) from a log decay. That rule caps how far leaves can sit from
  wood (https://minecraft.wiki/w/Leaves).

---

## 4. Biomes O' Plenty

Hand-written Java shapes, one class per kind
(https://github.com/Glitchfiend/BiomesOPlenty/tree/26.3/common/src/main/java/biomesoplenty/worldgen/feature/tree).

- **BasicTreeFeature**: the classic oak. Leaf layers of radius 2, 2, 1, 1 from the top, corners cut
  at random. Optional vines up to 4 long
  ([BasicTreeFeature.java](https://github.com/Glitchfiend/BiomesOPlenty/blob/26.3/common/src/main/java/biomesoplenty/worldgen/feature/tree/BasicTreeFeature.java)).
- **BigTreeFeature**: a port of Mojang's big tree (0.618 bole, 0.381 slope) with a
  `foliageDensity` setting: clusters per level = 1.382 + (density × height / 13)²
  ([BigTreeFeature.java](https://github.com/Glitchfiend/BiomesOPlenty/blob/26.3/common/src/main/java/biomesoplenty/worldgen/feature/tree/BigTreeFeature.java)).
- **TaigaTreeFeature**: leaf layers from the top. Radius = min((i+2)/3, 3 + layers left, 6). Small
  layers alternate full and half. Wide layers become 4 branches from the trunk's corners on every
  other layer. The trunk narrows with height. Leaves stay within 4 of the trunk
  ([TaigaTreeFeature.java](https://github.com/Glitchfiend/BiomesOPlenty/blob/26.3/common/src/main/java/biomesoplenty/worldgen/feature/tree/TaigaTreeFeature.java)).
- **RedwoodTreeFeature**: the taiga logic with radius ≤ 4 and branches on every layer. The trunk is
  diamond rings. Ring 1 reaches 35-50% of the height, ring 2 7-17%, ring 3 0-6% (wider trunks
  60-80 / 10-30 / 3-12%), giving a flared, buttressed base. Small branches with leaf bushes grow
  from the trunk at 1 in 15 per log. Dirt is filled 3 blocks under the trunk so it never floats
  ([RedwoodTreeFeature.java](https://github.com/Glitchfiend/BiomesOPlenty/blob/26.3/common/src/main/java/biomesoplenty/worldgen/feature/tree/RedwoodTreeFeature.java)).
- **BayouTreeFeature**: a centre trunk plus a cross of 4 side logs to 20-35% of the height (a flared
  swamp base). On each trunk log above the base there is a 1/3 chance of a big branch (2-4 long,
  rising) with a leaf pad and Spanish moss, else a 1/3 chance of a small one
  ([BayouTreeFeature.java](https://github.com/Glitchfiend/BiomesOPlenty/blob/26.3/common/src/main/java/biomesoplenty/worldgen/feature/tree/BayouTreeFeature.java)).
- **CypressTreeFeature**: the bottom third is bare trunk. The crown radius follows
  r = maxR × 0.667 × x × (1/(x² + 0.0817) - 0.924), with maxR = 1 + 0.1 × crown height. That
  peaks a quarter of the way up and ends in a point. The outer ring is 3/4 filled
  ([CypressTreeFeature.java](https://github.com/Glitchfiend/BiomesOPlenty/blob/26.3/common/src/main/java/biomesoplenty/worldgen/feature/tree/CypressTreeFeature.java)).
- **PalmTreeFeature**: height ≥ 8. It curves toward a random side. The offset starts at 0.05-0.34
  and grows ×1.3 per block, so the bend steepens with height. 1 in 8 stand straight. Fixed 5×5 star
  crown with drooping tips
  ([PalmTreeFeature.java](https://github.com/Glitchfiend/BiomesOPlenty/blob/26.3/common/src/main/java/biomesoplenty/worldgen/feature/tree/PalmTreeFeature.java)).
- **MahoganyTreeFeature**: straight trunk to height - 3. Then 4 limbs (N/E/S/W) go out 1-2 and up
  1-2, each ending in a two-layer bush with random corners
  ([MahoganyTreeFeature.java](https://github.com/Glitchfiend/BiomesOPlenty/blob/26.3/common/src/main/java/biomesoplenty/worldgen/feature/tree/MahoganyTreeFeature.java)).
- **TwigletTreeFeature**: one log column with leaves on its sides by chance (separate odds on even
  and odd levels), plus cocoa or shelf mushrooms on the trunk
  ([TwigletTreeFeature.java](https://github.com/Glitchfiend/BiomesOPlenty/blob/26.3/common/src/main/java/biomesoplenty/worldgen/feature/tree/TwigletTreeFeature.java)).

Pictures: https://modrinth.com/mod/biomes-o-plenty/gallery (Redwood Forest, Bayou, Rainforest).

---

## 5. Terralith

- A datapack with no code. Its trees are vanilla `minecraft:tree` features with new settings. We
  found no tree structure templates (.nbt) in it; its 169 .nbt files are structures
  (https://github.com/Stardust-Labs-MC/Terralith/tree/1.20/data/terralith/worldgen/configured_feature).
- Trick 1, sparse crowns: leaves are mixed with air. Giant jungle branches use 2 jungle leaves to 1
  air, and the savanna crowns 5 acacia leaves to 2 air
  (https://github.com/Stardust-Labs-MC/Terralith/blob/1.20/1-20-5-overlay/data/terralith/worldgen/configured_feature/jungle/mega/giant_tree/branch.json,
  https://github.com/Stardust-Labs-MC/Terralith/blob/1.20/1-20-5-overlay/data/terralith/worldgen/configured_feature/savanna/baobabs.json).
- Trick 2, fat round trunks: a foliage placer filled with wood. The giant jungle trunk is a fancy
  foliage placer (height 16, radius 1) filled with jungle logs. Savanna trees use acacia wood in
  fancy or mega-pine foliage placers, which gives swollen or conical bases
  (https://github.com/Stardust-Labs-MC/Terralith/blob/1.20/data/terralith/worldgen/configured_feature/jungle/mega/giant_tree/trunk.json; baobabs.json).
- Trick 3, one tree from several features: the giant jungle tree is `trunk`, `branch`, `branch_2`
  and `top`. The top is a forking trunk 32 high with a 5-7 radius flat fancy crown. The branches
  are bending trunks with pine foliage
  (https://github.com/Stardust-Labs-MC/Terralith/blob/1.20/1-20-5-overlay/data/terralith/worldgen/configured_feature/jungle/mega/giant_tree/top.json).
- Trick 4, odd mixes: the skylands mega tree is a giant trunk 24-30 high with a 22-24 high
  mega-pine crown of dark oak leaves. Siberian taiga trees use pine foliage of radius 4-6 on 10-22
  high trunks
  (https://github.com/Stardust-Labs-MC/Terralith/blob/1.20/1-20-5-overlay/data/terralith/worldgen/configured_feature/skylands/mega_tree.json,
  https://github.com/Stardust-Labs-MC/Terralith/blob/1.20/1-20-5-overlay/data/terralith/worldgen/configured_feature/taiga/siberian/tall_tree.json).
- Pictures: https://modrinth.com/datapack/terralith.

---

## 6. Oh The Biomes We've Gone (BYG)

- Trees are built from structure templates (.nbt) by the "Oh The Trees You'll Grow" library
  (https://github.com/CorgiTaco-MC/Oh-The-Trees-Youll-Grow).
- Two templates per tree: a **trunk base** and a **canopy**. Marker wool in the templates:
  white = trunk centre or canopy anchor (exactly one), red = "log builders", yellow = where the
  canopy attaches. Red wool in the base extends logs down to the ground, up to `max_log_depth`
  (default 5), so roots meet uneven ground. Red wool in the canopy fills logs down to meet the
  trunk. A `height` number sets how much plain trunk goes between base and canopy. Templates may
  hold several palettes, picked at random. Random rotation is on by default, and leaf distances are
  recalculated
  (https://github.com/CorgiTaco-MC/Oh-The-Trees-Youll-Grow/blob/26.1.2/common/src/main/java/dev/corgitaco/ohthetreesyoullgrow/world/level/levelgen/feature/TreeFromStructureNBTFeatureV2.java,
  https://github.com/CorgiTaco-MC/Oh-The-Trees-Youll-Grow/blob/26.1.2/common/src/main/java/dev/corgitaco/ohthetreesyoullgrow/world/level/levelgen/feature/configurations/TreeFromStructureNBTConfigV2.java).
  We read the library's V2 code; BYG's configs name the `tree_from_nbt_v1` feature.
- BYG configs pair one trunk with one canopy (`baobab_tree1` = `baobab_trunk1` + `baobab_canopy1`).
  Height is "biased to bottom" 5-15 for baobab, redwood and araucaria
  (https://github.com/Potion-Studios/Oh-The-Biomes-Weve-Gone/blob/1.21.1/Common/src/main/generated/resources/data/biomeswevegone/worldgen/configured_feature/baobab_tree1.json,
  https://github.com/Potion-Studios/Oh-The-Biomes-Weve-Gone/blob/1.21.1/Common/src/main/generated/resources/data/biomeswevegone/worldgen/configured_feature/araucaria_tree1.json).
- Templates per species: oak 29, maple 15, sakura 14, spruce 11, baobab 6 (3 trunks, 3 canopies),
  redwood 6, cypress 6, mahogany 7, ebony 5, araucaria 3 (1 trunk, 2 canopies), palo verde 2
  (https://github.com/Potion-Studios/Oh-The-Biomes-Weve-Gone/tree/1.21.1/Common/src/main/resources/data/biomeswevegone/structure/features/trees).
- Lesson: hand-built templates look the most "designed". Variety comes from the template count,
  the random trunk length and rotation. A species with 2-3 templates repeats in a forest.
- Pictures: https://modrinth.com/mod/oh-the-biomes-weve-gone.

---

## 7. Regions Unexplored

Java tree classes (https://github.com/UHQ-GAMES-MODS/RegionsUnexplored/tree/1.21/src/common/main/java/net/regions_unexplored/world/level/feature/tree).
There are classes for aspen, cypress and giant cypress, eucalyptus, kapok, larch, small/medium/large
Joshua, small/large Socotra, maple, mega and ultra baobab, palm, sakura and pines. There are also
custom foliage placers for redwood, sakura, willow and pines.

- **Branch decorator**: puts a small "branch" block (or a log, by config) on a random side of
  trunk logs. Only logs at least 2 below the top get one, and only where there is free space. It
  can wrap leaves on 5 sides of the stub
  (https://github.com/UHQ-GAMES-MODS/RegionsUnexplored/blob/1.21/src/common/main/java/net/regions_unexplored/worldgen/treedecorator/BranchDecorator.java).
- **Willow**: a hand-placed 3-layer crown. From 12 rim points, curtains hang 1, 2 or 3 blocks
  (one third each) (https://github.com/UHQ-GAMES-MODS/RegionsUnexplored/blob/1.21/src/common/main/java/net/regions_unexplored/worldgen/foliageplacer/WillowFoliagePlacer.java).
- **Willow trunk decorator**: 4 side logs run down 4-5 blocks until they reach dirt (ending in
  hanging roots), and 2 blocks up beside the trunk. That gives a flared, rooted base
  (https://github.com/UHQ-GAMES-MODS/RegionsUnexplored/blob/1.21/src/common/main/java/net/regions_unexplored/worldgen/treedecorator/WillowTrunkDecorator.java).
- **Mega baobab**: a 2×2 trunk with roots at its base. The top has 4 stubby limbs, one per side,
  each one of 8 hand-written shapes (out-out-up, up-out-out-up, ...) ending in a leaf blob
  (https://github.com/UHQ-GAMES-MODS/RegionsUnexplored/blob/1.21/src/common/main/java/net/regions_unexplored/world/level/feature/tree/MegaBaobabTreeFeature.java).
- Pictures: https://modrinth.com/mod/regions-unexplored/gallery (Redwoods, Joshua Desert).

---

## 8. William Wythers' Overhauled Overworld (WWOO)

- A datapack, built only from vanilla feature types. Read here from an unofficial mirror
  (https://github.com/Jayden-Dollaga/William-Wythers-Overhauled-Overworld-Datapack-/tree/main/data/wythers/worldgen/configured_feature/vegetation/tree).
  A fan catalog shows all 149 trees with front and side pictures
  (https://github.com/mrpistachiman/wwoo_tree_catalog, https://mrpistachiman.github.io/wwoo_tree_catalog/).
- **Variation**: nested random selectors. Ancient oak picks among tiers (5%, 15%, ...). Mediterranean
  cypress has 12 variants. Bayou cypress has surface, shallow, middle and deep variants for water depth
  ([ancient_oak.json](https://github.com/Jayden-Dollaga/William-Wythers-Overhauled-Overworld-Datapack-/blob/main/data/wythers/worldgen/configured_feature/vegetation/tree/ancient_oak.json),
  [mediterranean_cypress.json](https://github.com/Jayden-Dollaga/William-Wythers-Overhauled-Overworld-Datapack-/blob/main/data/wythers/worldgen/configured_feature/vegetation/tree/mediterranean_cypress.json),
  [bayou_cypress.json](https://github.com/Jayden-Dollaga/William-Wythers-Overhauled-Overworld-Datapack-/blob/main/data/wythers/worldgen/configured_feature/vegetation/tree/bayou_cypress.json)).
- **Big oaks and birches**: "ancient" trees use Mojang's fancy trunk placer at base 22 + rand 5 with
  jungle-style foliage pads. Trunks are bark-on-all-sides "wood" blocks. They put podzol and leaf
  litter on the ground (ancient_oak.json;
  [ancient_birch.json](https://github.com/Jayden-Dollaga/William-Wythers-Overhauled-Overworld-Datapack-/blob/main/data/wythers/worldgen/configured_feature/vegetation/tree/ancient_birch.json)).
- **Huge sequoia** (53 files): a 42-block column of stripped acacia log as the bole. A fancy
  foliage placer filled with logs (radius 2, height 12) swells the base. Rooted dirt goes beneath.
  Branches are a tree of sub-features, with N/E/S/W variants that call smaller branch levels and
  leaf pads
  ([1.json](https://github.com/Jayden-Dollaga/William-Wythers-Overhauled-Overworld-Datapack-/blob/main/data/wythers/worldgen/configured_feature/vegetation/tree/huge_sequoia/1.json),
  [branch/1_east.json](https://github.com/Jayden-Dollaga/William-Wythers-Overhauled-Overworld-Datapack-/blob/main/data/wythers/worldgen/configured_feature/vegetation/tree/huge_sequoia/branch/1_east.json)).
  Huge spruce also has 53 files.
- **Swamp trees**: the swamp oak uses the mangrove's upward-branching trunk with random-spread
  leaves. Logs go below the trunk into the water
  ([swamp_forest_oak.json](https://github.com/Jayden-Dollaga/William-Wythers-Overhauled-Overworld-Datapack-/blob/main/data/wythers/worldgen/configured_feature/vegetation/tree/swamp_forest_oak.json)).
  The large willow is the cherry trunk (2 limbs, 4-5 out) with cherry foliage set to always hang,
  plus leaves added below leaves, which makes curtains
  ([willow_large.json](https://github.com/Jayden-Dollaga/William-Wythers-Overhauled-Overworld-Datapack-/blob/main/data/wythers/worldgen/configured_feature/vegetation/tree/willow_large.json)).
- **Young trees as separate features**: `young_kapok`, `young_mega_jungle` and `young_brazilwood`
  sit next to the adults (tree folder above).
- Pictures: https://modrinth.com/mod/wwoo/gallery.

---

## 9. Thin branch blocks

Ronald wants thinner branch blocks beside full logs, "a bit like a fence".

### Dynamic Trees: how its thin branches work

- **Radius in the block state.** Each branch block has an integer `radius` property, 1-8 px
  (thick trunks 1-24), plus waterlogged. So one wood type has only 8 (or 24) shapes in its state
  (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/block/branch/BasicBranchBlock.java,
  https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/block/branch/ThickBranchBlock.java).
- **Connections are not stored.** At render time the model reads the six neighbours' radii into
  model data (`ModelConnections`). Connections cost no block states
  (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/neoforge/src/main/java/com/dtteam/dynamictrees/model/baked/BasicBranchBlockBakedModel.java).
- **Radius comes from the pipe model**: √(sum of squared child radii) + tapering, never shrinking
  (section 2). Volume is radius² × 64 voxels per block; a log is 4096
  (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/systems/nodemapper/NetVolumeNode.java).
- **Model pieces** (same baked model file):
  - The **core** is a cube 2r px wide, oriented on the axis of the biggest connection, with bark on
    its sides.
  - A **sleeve** (arm) goes toward each connected neighbour. Its width is the *neighbour's*
    radius, so a thin twig meets a thick limb at the twig's width.
  - A **ring** texture caps a branch that ends, using the end-grain texture.
  - Radius 8 is a plain full block with no sleeves.
- **Bark UVs are cropped, not squashed.** Faces take the centre strip `[8-r, ..., 8+r]` of the
  full bark texture. A 2 px twig shows the middle 4 px of the log texture at the same pixel size
  (same file, `generateSleevePart`, `generateCorePart`).
- **Collision and selection** use the same shape: a centre cube of the block's radius plus a box
  toward each neighbour at min(neighbour, own) radius. Shapes are cached by connection pattern.
  Radius 8 is a full cube
  (https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/block/branch/BranchBlock.java).
- **Drops and felling**: cutting a branch collects every branch beyond the cut and adds up its
  volume. Whole logs drop for each 4096 voxels. The rest drops as sticks at 8 per log, since "a
  stick is 1/8th of a log". Thin trees give mostly sticks
  (BranchBlock.java `getPrimitiveLogs`;
  https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/tree/species/Species.java `getLogsAndSticks`).
  Axe damage also scales with radius and volume (BranchBlock.java `damageAxe`).

### Other thin connecting blocks

- **Minecraft fence**: 4 booleans (N/E/S/W) plus waterlogged, so 32 states. Its blockstate is
  "multipart". A post model always applies, and one side model applies per true side, rotated 90°
  each time (https://minecraft.wiki/w/Fence, https://minecraft.wiki/w/Tutorial:Models).
  The post is 4×16×4 px (6→10) and each side is two 2 px bars
  (https://github.com/misode/mcmeta/blob/assets/assets/minecraft/models/block/fence_post.json,
  https://github.com/misode/mcmeta/blob/assets/assets/minecraft/models/block/fence_side.json).
- **Minecraft wall**: `up` plus none/low/tall per side plus waterlogged, so 2 × 3⁴ × 2 = 324
  states (https://minecraft.wiki/w/Wall). Post 8 px (4→12), side 6 px (5→11)
  (https://github.com/misode/mcmeta/blob/assets/assets/minecraft/models/block/template_wall_post.json,
  https://github.com/misode/mcmeta/blob/assets/assets/minecraft/models/block/template_wall_side.json).
- **Artemis' Thin Logs** (Minecraft): a 6 px log (core 5→11) for every wood type, with thin trees
  in worldgen. Multipart states: 6 direction booleans, a core orientation (vertical, east-west,
  north-south, junction) and an anchor face. Each arm is a 6×6×5 box whose bark UV is cropped from
  the vanilla log texture (`uv [5,5,11,10]`). Leaves and snow can sit on it as an overlay
  (https://modrinth.com/mod/artemis-thin-logs,
  https://github.com/IntelligenceModding/ThinLogs/blob/1.21.1-neoforge/src/generated/resources/assets/thinlogs/blockstates/thin_oak_log.json,
  https://github.com/IntelligenceModding/ThinLogs/blob/1.21.1-neoforge/src/main/resources/assets/thinlogs/models/generation/branch_arm_north.json).
- **Branchy** (Minecraft): branch blocks for all vanilla woods that connect in all 6 directions,
  with leaves added by right-click (https://modrinth.com/mod/branchy).
- **Conquest Reforged** lists "Thin Branches" and mossy branches. Its notes say spruce and pine
  branches were refit to a new log pillar thickness. We found no sizes
  (https://www.conquestreforged.com/news, https://modrinth.com/mod/conquest-reforged/changelog).
- **Regions Unexplored** has a separate small "branch" block for stubs on trunks, switchable to
  full logs by config (BranchDecorator.java, section 7).
- **Vintage Story** has no thin wood. Branchy leaves (leaves with twigs) do the job: they always
  drop sticks, while plain leaves give none. Width 0.1-0.3 at branch tips places them, so they
  read as the twig layer (https://wiki.vintagestory.at/index.php/Tree/en,
  https://wiki.vintagestory.at/Special:MyLanguage/Stick; section 1).
- Not found: we found no thin-branch feature in Chipped or a Macaw's "logs and branches" mod. We
  did not confirm either.

### Costs and failure looks

- **Meshing**: Mojang's bug MC-123584 says a block change redraws the whole chunk section, and
  complex models such as fences make that worse (https://bugs.mojang.com/browse/MC-123584).
  Dynamic Trees also reads neighbours for every branch model on each rebuild (baked model above).
- **Worldgen**: a Dynamic Trees issue reports slow chunk loading with its trees
  (https://github.com/DynamicTreesTeam/DynamicTrees/issues/850). A tweaks mod says Dynamic Trees
  does "considerably more work than vanilla" in branch logic, leaf hydration, neighbour checks and
  rendering (https://www.curseforge.com/minecraft/mc-mods/dtt-dynamic-trees-tweaks).
- **Block states**: storing connections in the state multiplies states (fence 32, wall 324).
  Dynamic Trees keeps 8-24 per wood by computing connections at render time.
- **Spaghetti**: Dynamic Trees weights signals toward existing branches (radius + 2) so growth
  reuses limbs. Its dark oak logic keeps a new limb 2 blocks clear of the trunk "so it won't
  interfere with new side branches". JoCode's careful mode avoids "frankentrees" (section 2).
- **Branches that don't meet the trunk**: each sleeve takes the neighbour's radius, and the core
  turns to its biggest connection, so a thin limb always plugs into its parent (baked model).
- **Floating sticks**: Dynamic Trees grows leaves only from radius-1 twigs, and cutting drops
  everything beyond the cut. No stick is left unattached (CellKits.java; BranchBlock.java).

---

## 10. What makes a block tree read as its species

1. **Where the crown starts.** The clear-bole fraction sets the species before anything else:
   oak 0.3 and pine 0.4 vs larch 0.2 and maple 0.15 in Vintage Story
   (section 1). Brazil nut is bare for over half its height
   (https://en.wikipedia.org/wiki/Brazil_nut).
2. **Leader or no leader.** Conifers and birch keep one stem to the top (`dieAt` 0.001, conifer
   logic). Oaks hand the trunk to many equal limbs, as English oak's child width grows from 0.1 to
   1.5 up the trunk (section 1; ConiferLogic in section 2).
3. **Branch angle.** Level whorls (pine 1.57 rad) vs steep (birch 0.8, old oak 0.5) vs forking
   (acacia width ×1, parent ×0.5) are the strongest silhouette cue after height (section 1).
4. **Droop or lift at the tips.** One number changes the mood: `gravityDrag` 0.1 for birch, -0.06
   for larch, 0.2 for willow (section 1; Biodiversity willow).
5. **Height-to-width ratio.** Italian cypress is often under a tenth as wide as it is tall
   (https://en.wikipedia.org/wiki/Cupressus_sempervirens). The BOP cypress curve and Vintage
   Story's branchless spire do this; a generic leaf ball cannot (sections 1 and 4).
6. **Crown pad shape.** Flat pads (acacia cells, 7×2×7 cluster), skirts (spruce rings) and domes
   (fancy oak's half-circle profile) are kept separate per species (sections 2 and 3).
7. **Trunk taper and flare.** Buttresses and flares come from extra trunk columns that stop early.
   Large kapok's 4 trunks die at 0.001-0.3. BOP redwood's rings stop at 35-50%, 7-17% and 0-6%.
   Bayou's side logs reach 20-35% (sections 1 and 4).
8. **Roots meet the ground.** BYG red-wool builders reach down 5 blocks. BOP fills dirt 3 below.
   Vintage Story sinks big trees 1-2 blocks. RU willow roots run down to dirt (sections 1, 4, 6, 7).
9. **Wood → twig → leaf gradient.** Vintage Story's log → branchy leaves → leaves at every tip,
   and Dynamic Trees' radius-1 twigs, make crowns look built of branches, not filled (sections 1-2).
10. **Gaps in the crown.** Corner holes (cherry 25%), leaves mixed with air (Terralith 2:1) and
    smothering (Dynamic Trees) let light and sky through (sections 2, 3 and 5).
11. **Hanging parts.** Cherry hanging leaves (1/6), willow curtains (RU 1-3 blocks, WWOO always on)
    and vines and Spanish moss on swamp trees (sections 3, 4, 7 and 8).
12. **Species bark and texture on all sides.** WWOO and Terralith use bark-on-all-sides wood for
    trunks and limbs, so bent limbs show no end grain (sections 5 and 8).
13. **Ground signature.** Podzol under mega spruce and dark oak, leaf litter under WWOO ancient
    oaks, moss on the shady side in Vintage Story (sections 1, 2, 3 and 8).
14. **Habit changes with age.** Monkey puzzle is a pyramid when young and an umbrella when old.
    Ginkgo goes from slender and sparse to broad and irregular
    (https://en.wikipedia.org/wiki/Araucaria_araucana, https://en.wikipedia.org/wiki/Ginkgo_biloba).
    Dynamic Trees grows through stages; WWOO adds separate young and ancient features (sections 2, 8).
15. **Real numbers, small variation.** Vintage Story gives each tree random size, leaf variant and
    angles. Dynamic Trees adds hash-based height variation (5 for conifers). Too much variation
    breaks the species read; none makes clones (sections 1-2).

---

## 11. Common failure looks and how the good mods avoid them

- **Blobs** (a leaf ball on a stick). Leaves come only from twigs or branch tips: Vintage Story
  width < 0.3, Dynamic Trees radius-1 twigs with hydration. Crowns are several clusters on limbs,
  as in Mojang's fancy oak (sections 1, 2 and 3).
- **Floating leaf clusters.** Leaves sit within reach of wood: Minecraft's 6-block decay, Dynamic
  Trees' hydration, BOP leaves within 4 of the trunk. Fancy oak drops any cluster whose limb path
  is blocked (sections 3 and 4; https://minecraft.wiki/w/Leaves).
- **Broken or disconnected limbs.** Limbs are drawn as continuous walks from the trunk: Vintage
  Story steps, Mojang `MakeLimb`, the cherry walk that mixes horizontal and vertical steps. Canopy
  templates fill logs down to the trunk with red wool (sections 1, 3 and 6).
- **Lollipop crowns.** Branching starts low enough, and the bole length is a species trait
  (0.15-0.45 in Vintage Story). Crown width grows with height via `branchQuantityEvolve` and the
  width evolve (section 1). Minecraft's wiki itself names a "lollipop" pine as a variant
  (https://minecraft.wiki/w/Tree).
- **Clones.** Random size, angles, leaf variant per tree (Vintage Story); 16 shapes per size × 4
  rotations (Dynamic Trees); many templates and random trunk length (BYG); nested selectors and
  water-depth variants (WWOO) (sections 1, 2, 6 and 8).
- **Too-perfect symmetry.** Vintage Story's `branchHorizontalAngle` ±π with a minimum gap. Dark
  oak's lean and random 1/3 stubs. Corner holes (sections 1 and 3).
- **Leaves too far from wood.** Vintage Story cannot do this, because leaves are the ends of
  branches. Dynamic Trees caps it with smother depth (spruce 3, acacia 2) (sections 1 and 2).
- **Floating bases on slopes.** `yOffset` -1/-2 in Vintage Story; dirt fill below in BOP; log
  builders to the ground in BYG (sections 1, 4 and 6).

---

## 12. Species references

Photos were chosen from Wikimedia Commons by their titles and descriptions. We did not open the
images, so check each by eye before use. "Best block version" is our judgement from the sources above.

**Pedunculate oak (Quercus robur).** Up to 20 m (rarely 40). Single stout trunk, often a short
bole. Spreading, unevenly domed crown with massive lower branches. Grey-brown, grooved bark
(https://en.wikipedia.org/wiki/Quercus_robur).
Photos: https://commons.wikimedia.org/wiki/File:Solitary_Quercus_robur.jpg,
https://commons.wikimedia.org/wiki/File:Quercus_robur_-_alone_tree.jpg.
Best block version: Vintage Story old English oak (4 leaning leaders); WWOO ancient oak
(sections 1 and 8).

**Silver birch (Betula pendula).** 15-25 m. Slender trunk under 40 cm. White bark that peels.
Slender, often hanging twigs (https://en.wikipedia.org/wiki/Betula_pendula).
Photos: https://commons.wikimedia.org/wiki/File:Betula_pendula_Finland.jpg,
https://commons.wikimedia.org/wiki/File:B._pendula,_Koivu_Birch_end_of_Sept.jpg.
Best: Vintage Story silver birch (leader, steep whorls, drooping tips) (section 1).

**Norway spruce (Picea abies).** 35-55 m. Branchlets hang down
(https://en.wikipedia.org/wiki/Picea_abies).
Photos: https://commons.wikimedia.org/wiki/File:Kuusk_Keila-Paldiski_rdt_ääres.jpg,
https://commons.wikimedia.org/wiki/File:Landmark_spruce_by_road_162_through_Häggvall.jpg.
Best: Dynamic Trees spruce (conifer logic); WWOO huge spruce (sections 2 and 8).

**Umbrella thorn (Vachellia tortilis).** Medium to large canopied tree up to 21 m; a wiry bush when
very dry. Paired thorns (https://en.wikipedia.org/wiki/Vachellia_tortilis).
Photos: https://commons.wikimedia.org/wiki/File:Acacia_(Vachellia_tortilis),_parque_nacional_Serengueti,_Tanzania,_2024-05-26,_DD_26.jpg,
https://commons.wikimedia.org/wiki/File:Vachellia_tortilis_subsp._heteracantha_20D_4561.jpg.
Best: Vintage Story acacia (leaning multi-stem vase, forking, flat top) (section 1).

**Honey mesquite (Prosopis / Neltuma glandulosa).** Usually 6-9 m, up to 15 m. Rounded,
floppy, drooping branches with feathery leaves and paired thorns. Turns multi-trunked after cutting
(https://en.wikipedia.org/wiki/Neltuma_glandulosa).
Photos: https://commons.wikimedia.org/wiki/File:Prosopis_glandulosa_kz4.jpg,
https://commons.wikimedia.org/wiki/File:P145_Honey_mesquite_(Prosopis_glandulosa)_with_mistletoe_(15321664643).jpg.
Best: none close. A Vintage Story acacia with `gravityDrag` added would be a start.

**Acuri palm (Attalea phalerata).** Up to 18 m tall, but the trunk rarely tops 4 m. Up to 30
feather-like leaves rise from it (https://en.wikipedia.org/wiki/Attalea_phalerata).
Photos: https://commons.wikimedia.org/wiki/File:Attalea_phalerata_population.JPG,
https://commons.wikimedia.org/wiki/File:Attalea_phalerata_2zz.jpg.
Best: none. The BOP palm is a small star on a tall curved trunk, the opposite habit (section 4).

**Moriche palm (Mauritia flexuosa).** Up to 35 m (some say 50). Trunk up to 70 cm. Large leaves form
a rounded crown. Breathing roots up to 30 m out
(https://en.wikipedia.org/wiki/Mauritia_flexuosa).
Photos: https://commons.wikimedia.org/wiki/File:Moriche_Palms_(Mauritia_flexuosa)_(39236789505).jpg,
https://commons.wikimedia.org/wiki/File:Mauritia_flexuosa_(19861732855).jpg.
Best: WWOO has a `desert_fan_palm` (section 8, not inspected in detail).

**Monkey puzzle (Araucaria araucana).** 30-40 m, trunk 1-1.5 m. Pyramidal when young, umbrella
form when old. Thick, "reptilian" branches covered by leaves, very symmetrical
(https://en.wikipedia.org/wiki/Araucaria_araucana).
Photos: https://commons.wikimedia.org/wiki/File:Araucaria_en_Parque_Nacional_Conguillio.jpg,
https://commons.wikimedia.org/wiki/File:Monkey_Puzzle_tree_(Araucaria_Araucana)_on_Penryn_Campus,_Cornwall_01.jpg.
Best: BYG araucaria (1 trunk + 2 canopy templates) (section 6).

**Ginkgo (Ginkgo biloba).** 20-35 m. Conical crown when young, broader and irregular with age.
Angular crown with long, erratic branches. Young trees are tall, slender and sparsely branched.
Short "spur shoots" carry leaves (https://en.wikipedia.org/wiki/Ginkgo_biloba).
Photos: https://commons.wikimedia.org/wiki/File:GINKGOBAUM-2.jpg,
https://commons.wikimedia.org/wiki/File:Ginkgo_biloba_Frederick_MD1.jpg.
Best: the Vintage Story Biodiversity mod's ginkgo treegen (bald cypress numbers)
(https://github.com/captanredbeard/Biodiversity/blob/master/bdtree/assets/bdtree/worldgen/treegen/ginkgo.json).

**Bald cypress (Taxodium distichum).** 10-40 m, trunk 0.9-2.1 m. The trunk is often ringed by
"cypress knees" from the roots. Fibrous, stringy bark. Deciduous conifer
(https://en.wikipedia.org/wiki/Taxodium_distichum).
Photos: https://commons.wikimedia.org/wiki/File:Taxodium_distichum_NRCSMS01010.jpg,
https://commons.wikimedia.org/wiki/File:Taxodium_distichum_OB_Wrocław_2.jpg.
Best: BOP bayou tree (flared base, moss) and WWOO bayou cypress (water-depth variants); Vintage
Story bald cypress for limbs (sections 1, 4 and 8).

**Italian cypress (Cupressus sempervirens).** Up to 35 m. The wild form has a conic crown with
level branches. Most planted trees are columnar, often under a tenth as wide as tall: an
"exclamation mark" (https://en.wikipedia.org/wiki/Cupressus_sempervirens).
Photos: https://commons.wikimedia.org/wiki/File:Cupressus_sempervirens,_Saint-Rémy-de-Provence_cf01.jpg,
https://commons.wikimedia.org/wiki/File:Abbazia_di_Sant'Andrea_(Borzone,_Borzonasca)_-_cipresso_2022-06-09.jpg.
Best: BOP cypress curve; Vintage Story green spire cypress; WWOO Mediterranean cypress (12 variants)
(sections 1, 4 and 8).

**Tempskya (fossil tree fern, Cretaceous).** A "false trunk" up to 6 m by 50 cm made of many
forking stems wrapped in roots. Small leaves grew along the trunk's height, not in a crown on top
(https://en.wikipedia.org/wiki/Tempskya).
Photos (reconstructions): https://commons.wikimedia.org/wiki/File:Tempskya_sp.JPG (restoration at
MUSE, Trento), https://commons.wikimedia.org/wiki/File:Tempskya_sp._-_MUSE.jpg (1:1 reconstruction).
Best: none found.

**Giant sequoia (Sequoiadendron giganteum).** 50-85 m, trunks 6-8 m. Columnar trunk with fibrous
bark up to 90 cm thick. In groves the trunk is branch-free to 20-50 m; solitary trees keep low
branches; trees under 100 years keep their dead branches
(https://en.wikipedia.org/wiki/Sequoiadendron_giganteum).
Photos, old: https://commons.wikimedia.org/wiki/File:Grizzly_Giant_Mariposa_Grove.jpg.
Young: https://commons.wikimedia.org/wiki/File:Young_giant_sequoia_Sequoiadendron_giganteum_in_Big_Pine_CA.jpg,
https://commons.wikimedia.org/wiki/File:2013-05-03_19_06_19_Young_cultivated_Sequoiadendron_giganteum_(Giant_Sequoia)_in_Ewing,_New_Jersey.JPG.
Best: WWOO huge sequoia (53 parts); BOP redwood (ringed flare); Vintage Story redwood (2x2
sections) (sections 1, 4 and 8).

**Pacific dogwood (Cornus nuttallii).** 6-23 m, spread about 6 m. In full sun it has a short trunk
and a crown as wide as tall. Under a canopy it has a tapered trunk and a short, slender crown
(https://en.wikipedia.org/wiki/Cornus_nuttallii).
Photos (not confirmed as whole-tree): https://commons.wikimedia.org/wiki/File:Pacific_Dogwood_(Cornus_nuttallii).jpg,
https://commons.wikimedia.org/wiki/File:Cornus_nuttallii_7703404.jpg.
Best: none found.

**Andiroba (Carapa guianensis).** Usually 25-35 m (up to 55). Bole free of branches for 20-30 m.
A large, very dense crown of several erect or arching branches. Buttresses up to about 2 m
(https://sweetgum.nybg.org/science/projects/fg/monograph-details/?irn=10546,
https://costa-rica-tree-atlas.vercel.app/en/trees/cedro-macho).
Photos: https://commons.wikimedia.org/wiki/File:Starr-120606-6850-Carapa_guianensis-habit-Kahanu_Garden_NTBG_Hana-Maui_(25118063306).jpg,
https://commons.wikimedia.org/wiki/File:Andirobaamazonica.jpg.
Best: Vintage Story large kapok's staggered-trunk buttress is the closest technique (section 1).

**Matamata (Eschweilera, e.g. E. coriacea).** Canopy tree to about 35-40 m, often buttressed.
Fairly smooth brown bark that peels in plates
(https://sweetgum.nybg.org/science/vh/monographs-details/?irn=8955).
Photo: https://commons.wikimedia.org/wiki/File:Eschweilera_ovalifolia_(19095167756).jpg ("a
Matamata tree", Amacayacu, Colombia). Only one found.
Best: none found.

**Ipê (Handroanthus).** Many flowers open on still-leafless branches at the end of the dry season
(https://en.wikipedia.org/wiki/Handroanthus). Pink ipê is deciduous, up to about 15 m, with a wide,
globe-shaped crown (https://www.worldplants.ca/display.php?id=8701).
Photos: https://commons.wikimedia.org/wiki/File:055_Pink_Ipê_tree_in_Mato_Grosso_Photo_by_Giles_Laurent.jpg,
https://commons.wikimedia.org/wiki/File:Handroanthus_impetiginosus_in_China.jpg.
Best: WWOO `pink_lapacho` (fancy trunk and crown, mixed leaf blocks) (section 8).

**Brazil nut (Bertholletia excelsa).** To 50 m, trunk 1-2 m. A straight stem, usually without
branches for well over half its height. A large crown of long branches stands above the
surrounding forest (https://en.wikipedia.org/wiki/Brazil_nut).
Photos: https://commons.wikimedia.org/wiki/File:Bertholletia_excelsa_(Castanha_do_pará)_-_Reserva_de_Desenvolvimento_Sustentável_de_Rio_Negro_(Lago_do_Acajatuba)_-_Eric_Bem_dos_Santos_-_2023.jpg,
https://commons.wikimedia.org/wiki/File:Bertholletia_excelsa_compose.jpg.
Best: WWOO kapok (mega jungle trunk, base 32) for the tall clear bole
([kapok.json](https://github.com/Jayden-Dollaga/William-Wythers-Overhauled-Overworld-Datapack-/blob/main/data/wythers/worldgen/configured_feature/vegetation/tree/kapok.json)).

---

## 13. Our trees: the audit and what changed (11-04 / 11-05)

The audit sheets (`mods/base/source/trees/audit.py`: every species, young and three seeds grown,
side and 3/4) are in `~/Coding/chiacraft-archive/images/tree-looks/`. The worst first, and what
`claude/tree-looks` did about each:

1. **Every young tree was a 3x3 ball on a stick** (`young_tree_blocks`), the most-seen tree shape
   (a young tree in every 5x5 cell of a wood; the playtest's flooded-forest "lollipop"). Now each
   grows from its own species' recipe, small (`tree_recipes::understory`: a third of the height,
   clusters half as wide, branched low and leafy as young trees are, thin wood as leafy sprays;
   the scatter's reach 3). Bigger young ones closed the forest floor on the PC look.
2. **Oak**: scraggly, gappy layers, a tuft poking out of the top. Now a broad dome (Lesson 2:
   branching low, limbs out at 48 degrees, the leader ending in the crown), 2 x 2 on big ones.
3. **Spruce**: a ragged, lopsided cone. Now regular whorls of five (`branch_var`: a recipe's
   randomness; Dynamic Trees' and vanilla's spruces are the most regular trees of all).
4. **Birch**: a pile of logs in the crown. Now slim, its twigs leaves (VS: birch droop, narrow).
5. **Acacia, mesquite**: a leaf skirt under the parasol (a bug: a forked trunk still put its
   leaves at the fork). Gone.
6. **A 2 x 2 trunk narrowing on bare trunk stepped off-centre, as if broken.** Now a 2 x 2 foot
   holds to the first branch (VS builds thick trunks from columns that stop in the crown).
7. **Palms**: fronds merged into a dome. Now frond by frond (`frond_angle`, `frond_spread`).
8. **Dogwood** had no recipe (a box of leaves). Now opposite level branches in flat sprays, with
   `twig`: thin wood as leafy sprays, as VS's branchy leaves do (section 9).

Still to do: araucaria (broccoli; the monkey puzzle's tiers, section 12), ginkgo (pom-poms on
sticks), a per-tree form roll (crooked, twin, old), and the tropical canopy trees' 1-wide trunks
under 17-wide crowns (Plants' recipes; a 2 x 2 andiroba, Plants agreed).

Thin branch blocks (Ronald, 11-05): the study `mods/base/source/trees/branches.py` draws an oak
whose wood under 0.4 wide is a 4 px branch and under 0.75 an 8 px one, a post with an arm to each
wood neighbour, bark cropped as Dynamic Trees does. An oak's wood splits about a third each
(4 px, 8 px, logs). Offline estimate of faces drawn per tree, leaves included: +20-50% for oak,
birch and spruce, +70-150% for the broad tropical trees. The engine has what it needs (per-state
box models through `prop` and `states`); a block per wood and size with six connection props is
64 states, set by worldgen, which knows the neighbours.
