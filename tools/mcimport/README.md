# Minecraft world import (test only)

Loads a Minecraft world into ours block for block, so our look can be compared with Minecraft's
(vanilla, Bliss, Distant Horizons) in the identical scene from the identical camera. Ronald's world
is his own and stays local: never commit it, build it in or publish it.

## Use

1. Copy the world's overworld `region/` folder (Minecraft 26.x: `dimensions/minecraft/overworld/region`)
   into one folder, e.g. `~/mc-world/` (on the PC it is there already).
2. For the far view, export Distant Horizons' data next to it:
   `python3 tools/mcimport/dh_export.py <.../data/DistantHorizons.sqlite> ~/mc-world/dh0.bin`.
3. `weft --world <new name> --import-mc ~/mc-world` imports before the world opens (once per world,
   about 5 s for 15k chunks), then plays it. `mc_import.txt` in the world's folder lists what was
   made and every Minecraft block that stood in for one we lack.

In the comparison preset (`tester-tools/look-bliss/bliss_env`): spots `mc_world` (Bliss's vista),
`mc_world_forest` and `mc_world_far0`-`3`, which pass `--import-mc` themselves. Our yaw turns the
other way from Minecraft's: ours = 180 - Minecraft's; its pitch down is positive, ours negative.

## How

- `crates/mcimport`: our own reader of Anvil chunks (NBT, 1.18+ palettes and biomes, 26.3's
  mixed palette lists) and of DH's format-2 columns; the block and biome mapping (`map.rs`).
- `chiacraft::mc_import` (engine): names to our states with fallbacks, cubes saved as if edited
  (they load instead of generating), the far grid `far.bin` that distant tiles take over the
  terrain formula, mob spawning off, new players on the ground under the vista (`mc_spawn.txt`).

## Known limits

- **Poplars** (26.3's dappled forest): their red, orange and yellow leaves have no block of ours, so
  they come in as birch leaves, green. Vanilla's red dots across the far forests are missing.
- **Leaf litter, pink petals, vines, lichen, fences, rails and other small things** are air or a
  plant of ours: we have no thin floor cover block. Waterlogged ones are water.
- **Blocks we lack** (tuff and deepslate variants, amethyst, sulfur, cinnabar, copper, terracotta
  colours, prismarine) are stone or the nearest of ours; all in `mc_import.txt`.
- **Biomes** are per column, the one at the column's surface (ours have no 3D biomes).
- **Leaves** come in as placed (no decay), and cherry leaves in bloom (Minecraft's are pink all
  year).
- Beyond DH's range the far view is our own terrain again.
