# Rivers like Bliss's scene

Planner, 11-05: "wide winding rivers through forest like Bliss's scene (ours show mostly lakes)". It's
built in the Minecraft forest test world (`base:mc_forest`, branch claude/mc-forest-test), and not in
the overworld. This note covers what the overworld would need, for when Ronald wants it there.

## What our land does now (seed 1, measured)

- **Rivers follow drainage.** Rain runs downhill over nodes 48 blocks apart (`base_api::land::net`).
  A node's flow is the rain of the nodes within 40 steps upstream, so a river stops growing after
  about 2 km.
  - Half the river stretches carry under 7 wet nodes of flow; only a tenth carry over 35.
  - The channel reaches full width (24 blocks) at 110. Few rivers do: most are 6-12 blocks across.
- **Lakes everywhere.**
  - Basin lakes: one per 420-block cell, kept four times in five (`LAKE_CHANCE` 0.8), 36-130 blocks
    in radius.
  - Pit lakes: rivers that end inland in a pit fill a lake of their own.
- **Result:** within 6 km of the origin, rivers cover 1.4% of the ground and lakes 2.8%. The river
  map is many short networks, each running into a round lake (`land_rivers` example).
- **Bliss's Minecraft scene is the opposite.** Rivers run on for kilometres, 20-50 blocks across,
  winding through the woods to the sea and the lakes, and there are few lakes. Minecraft doesn't
  route water at all: its river biome lies along the lines of a noise.

## What the test world does

- **`mc_rivers`: Minecraft's way.**
  - The channel runs along the zero lines of a two-octave noise 1,100 blocks across. A column's
    distance to the line is the noise over its slope (measured over 8 blocks; over one block it came
    out in steps).
  - Half width 10-26 blocks, set by a second noise; 5 blocks deep in the middle.
  - A 4-block strip level with the water, then valley sides as the drainage rivers have.
  - Faded out in the mountains, as the drainage rivers are.
  - It's cut together with the drainage rivers inside the same custom feature, so the engine, the map
    tools and every land mirror see the same land.
- **The drainage rivers keep the overworld's shape.** Making them wider and more winding only frayed
  them into lace (the first map).
- **Fewer lakes.**
  - Basin lakes one cell in seven (0.15).
  - A pit's lake only where a big flow ends (`sink_flow` 20, against 6).
- **How it's kept apart.** The differing numbers are a river style (`Rivers`: `RIVERS` for the
  overworld, `MC_RIVERS` for the test world). The land code reads the style in force on its thread
  (`with_mc_land`), and the river regions and lake sites are cached apart per style. The overworld's
  numbers and caches are as they were.

## For the overworld

Ronald's yes is needed first: it changes every world's look, and existing worlds would meet new land
at their saved chunks' edges. With that yes, the overworld's style would take `MC_RIVERS`' choices, in
one of two ways:

1. **Simplest: the overworld uses the test world's style.**
   - Set `RIVERS` to `MC_RIVERS`' numbers and run `mc_rivers` for both.
   - Then a look sweep: rivers through the desert, savanna and tundra, where Minecraft's river
     biome also runs (frozen in the cold).
2. **Ours, closer to nature.** Keep the drainage network and let it grow:
   - `RIVER_REACH` past 40, so trunks keep growing toward the sea. Costs region size and time;
     needs measuring.
   - Spill more pits instead of filling them, so networks join.
   - Fewer basin lakes.
   - Widths that grow with flow on a log scale, so mid-sized rivers are already 20-30 blocks.

   This is more work and less like Minecraft. It would keep rivers where the rain falls (dry lands
   barely have any, as now).

Either way, these need a check against the waystones and the ley lines, which stand on the land, and
against spawn (`fair_spawn` keeps plains by water).

My recommendation for the comparisons: keep rivers in the test world only (done). For the
overworld, take way 1 if Ronald likes the test world's rivers in the comparison sheets.

## Tools

On the PC, from a copy of the branch:
- `cargo run --release -p base_api --example mc_forest_map -- <seed> <x> <z> <across> <per pixel> <out.ppm>`:
  both worlds' biomes and water.
- `--example land_rivers`: the drainage network.
- `--example land_water`: water shares.
