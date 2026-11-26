# How Minecraft, Distant Horizons and Bliss colour leaves (11-05)

For the comparison test switch `base:mc_leaf_colours` (Ronald, 11-05: "lets make sure our tree
colors match the bliss/dh"). Read off Ronald's own Minecraft 26.3 client jar and Bliss 2.1.2 on the
PC; no Minecraft or Bliss files are in this repository.

**Minecraft.** Leaf textures are grey, and the game tints them per block.
- Oak, dark oak, jungle, acacia and mangrove-style leaves take the biome's foliage colour: the
  256×256 `colormap/foliage.png` at x = (1 − temperature) × 255 and
  y = (1 − downfall × temperature) × 255, both clamped to 0-1.
- The swamp's foliage colour is fixed, 0x6a7039.
- Spruce leaves are always 0x619961 and birch leaves always 0x80a755, whatever the biome.
- Grass works the same way from `colormap/grass.png`.

The base biomes' nearest Minecraft biomes, from the colormap:

| base biome | Minecraft biome | foliage | grass |
|---|---|---|---|
| plains, beach | plains | 77ab2f | 91bd59 |
| forest | forest | 59ae30 | 79c05a |
| birch forest | birch forest | 6ba941 | 88bb67 |
| taiga | taiga | 68a464 | 86b783 |
| snowy | snowy plains | 60a17b | 80b497 |
| desert, savanna | desert, savanna | aea42a | bfb755 |
| swamp | swamp | 6a7039 (fixed) | |
| tropical forest | jungle | 30bb0b | 59c93c |
| mountains | windswept hills | 6da36b | 8ab689 |
| sequoia grove | old growth spruce taiga | 68a464 | 86b783 |
| old growth forest | dark forest | 59ae30 | 79c05a |
| ocean | ocean | 71a74d | 8eb971 |

Base's own biome tints already start from these foliage values, but pass them through `muted()`
(less saturated, same brightness). Our spruce-like and birch-like leaves take their biome's colour
rather than a fixed one.

**Distant Horizons.** Its far LOD columns take each block's colour from the game: the texture's
average times the block's tint from the same biome colour resolvers, sampled from the real biome. So
far leaves get the same biome colours as near ones, with no seasons or colour change of its own.

**Bliss 2.1.2, as Ronald has it** (`SATURATION=0.3`, `SSS_TYPE=3`, seasons off):
- Leaves keep vanilla's tint (`gl_Color`). Its season colouring (`YearCycleColor`, which would
  multiply leaves by per-season colours) is off by default (`#define Seasons` commented out), and
  for DH's far leaves too (`DH_SEASONS`).
- What it adds to leaves:
  - subsurface scattering: leaves `SSSAMOUNT` 1.0, DH far leaves too (`DH_SUBSURFACE_SCATTERING`);
  - waving leaves;
  - its global saturation setting, on everything, not leaves only.
- So Bliss's canopy colour is vanilla's biome foliage colour, lit with SSS and its grade.

**Our test switch.** `/rule base:mc_leaf_colours 1` (`mods/base/src/seasons.rs`). It sets every base
biome's leaves to the table above, unmuted, with the fixed spruce and birch colours and no seasonal
turn. It goes through the season tints, so near and far tiles change alike. Turned off, base's leaf
colours come back. Other mods' biomes keep their own.
