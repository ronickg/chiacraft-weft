# The copper tier's first steps (design, 11-26)

Planner, 11-26: "the copper tier's first steps. Design note and numbers in vitals first ...
then build". What a player does after the knapped stone pick to hold the first copper tool.

## Where it stood

The vanilla-like path already worked, with nothing copper about it: a stone pick mines hidden
`base:copper_ore` veins (`metals.rs`: shallow in the cold, dry and beach biomes, below y 0
elsewhere), the cobblestone furnace smelts raw copper to an ingot in 10 s, and three ingots make
a pick or axe head in the table's book. No sign on the surface, no hint, no clock step.

## What real people did first (`reports/first-hours-research/prehistory.md`, `reports/content/Tools and materials.md`)

Copper came before smelting: native copper, lumps of the pure metal weathered out of the rock,
picked up and hammered cold into awls, points and axes (the Old Copper Complex round Lake
Superior from about 7,000 years ago; the Copper Inuit of the Arctic). Hammered cold it hardens.
Smelting malachite with charcoal and bellows came later. TerraFirmaCraft and Vintage Story both
start the metal age with surface nuggets.

## The design: two ways to the first copper head

1. **Native copper, hammered cold (new).** Lumps of native copper (`base:loose_copper`, picked up
   as `base:native_copper`) lie loose on open ground in the copper lands (`COPPER_LANDS`: the
   biome tags `cold` and `dry`, where `metals.rs` brings copper ore near the surface; the snowy
   plains, a bone land, among them). They are a find: `COPPER` 0.0003 of open columns, three
   times on gravel and slopes (placer copper), about two in a 64-block walk
   (`copper_lands_have_some`), under a quarter of flint's share. Put down on stone and struck
   sneaking with a rock (any hammer: rock, flint, bone), a lump opens the knapping screen as
   **Hammering Copper**: the squares outside the head's outline are beaten flat into it; a strike
   inside only dents it (copper bends, it doesn't break, so nothing is lost). The head is copper
   (axe head, pick head or shovel blade), graded at `COPPER_HEAD_MATERIAL_POINTS` 20 (copper's
   own, as from ingots) with the smithing skill. It hafts in the 2x2 with a stick and string as
   the stone heads do. A plain strike only knocks. A lump also melts in the furnace like raw
   copper (its `raw_ores/copper` tag).
2. **Ore and the furnace (as built).** Elsewhere copper is the ore, mined with the stone pick and
   melted in the furnace; three ingots make a head in the book. Unchanged.

So a player in the cold or the dry lands can hold a copper axe within minutes of finding a lump,
and one anywhere else after the furnace. Copper's tier (item level 12, harvest 170 by
`vitals::gear`) is unchanged.

## Hints and the clock

- `hint.base.native_copper`: native copper in the bag: put it down on stone and sneak-strike it.
- `hint.base.copper_ore`: raw copper in the bag and no furnace or ingot: the furnace and the book.
- The clock gains `copper` (any copper in the bag: a lump, ore, an ingot, a copper part) and
  `copper_tool` (a tool whose working part is copper), so playtests time the tier too.

## Later (not in this slice)

- Art: `native_copper` block tile and item icon (today recoloured from the flint nodule's), the
  knapping screen's squares in copper colours (today flint's).
- Malachite (green-stained stone in the dry lands and on beaches) and smelting it in a charcoal
  pit with bellows; the stone anvil; annealing; moulds and casting (bronze); `notes/CONTENT.md`
  tier 3 and `notes/CRAFTING.md` slice 11.
- A note to settle: copper's harvest is 170 in `vitals::gear` and 150 in `vitals::felling` and
  the notes (`FIRST_HOURS.md`, `CONTENT.md`).

## Check

`mods/testkit/scripts/cov_copper_1126.txt`: a census of loose copper in a cold and a dry land,
then on a pad in survival a lump picked up, put down, struck (a knock), hammered cold on the
screen into a pick head (a strike inside first: no loss), hafted into a copper pick, the clock's
`copper` and `copper_tool`, and raw copper's hint.
