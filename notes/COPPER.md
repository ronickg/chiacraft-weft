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

## Slice 2: the ore, the smelt and casting (11-26)

Planner, 11-26: "the next copper step from COPPER.md: ore, the smelt and the casting path, as
planned". Numbers in `vitals::smelt`; code in `metals.rs`, `block_entities.rs`, `casting.rs`.

What real people did (`reports/content/Tools and materials.md`, C1): copper melts at 1,085 °C and
malachite reduces with charcoal from about 700 °C, but a wood fire reaches 600-800 °C. Forced air
over charcoal (blowpipes, then bellows) got past 1,100 °C. Molten copper was poured into open or
two-part moulds of stone or fired clay, which were used again.

1. **Malachite.** In the dry lands and on beaches (`MALACHITE_LANDS`), copper's shallow veins are
   `base:malachite_ore`, green-stained stone, which breaks into `base:malachite` with a stone pick.
   It is tagged `raw_ores/copper`, so it smelts like raw copper. Copper ore stays as it was in the
   cold lands and deep down.
2. **The hot fire.** Copper of any kind (raw copper, malachite, native copper: `HOT_METALS`) melts
   only in a hot furnace. A hot furnace burns fuel of `HOT_FUEL_TICKS` (1,600) ticks or more
   (charcoal, `base:coal`, from the clamp or a log in the furnace; a coal block) and has
   `base:bellows` on any of its six sides. Bellows are made at the table from 5 planks, 3 leather and
   a stick. A furnace short of that won't light, or stops cooking. Anyone within 8 blocks gets
   `hint.base.copper_heat` once. Tin and iron smelt as before. Iron's own heat waits for the
   bloomery (CONTENT step 7).
3. **Moulds.** There are three fired-clay mould blocks (`MOULDS`): felling axe head, pick head and
   shovel blade. To make one, lay clay blocks in the head's shape in the grid to get an unfired
   mould, then fire it in the furnace.
   - Placed beside a hot furnace (any side), a mould takes 3 copper ingots (`CAST_INGOTS`, the
     same as the book) and casts them into its head in `cast_secs` (about 10 s).
   - The head is graded as a mould's work (`CAST_WORK` 22 plus `CAST_STATION` 10 on copper's 20),
     which makes it Good: a grade above a cold-hammered head at skill 0
     (`casting_beats_cold_hammering`).
   - The mould stays. Its screen has one slot in, one out and the progress arrow.
   - The crucible is not used, because the crucible is locked behind a ruin blueprint (the bronze
     gate). Bronze casting at the crucible comes later.
4. **Hints.** `copper_ore` now says how to make the furnace hot and names moulds. `casting` shows
   once a copper ingot is in the bag.

The book's three-ingot copper heads stay for now, so ore copper has two ways to a head.

Check: `mods/testkit/scripts/cov_copper2_1126.txt`.

## Later (not in this slice)

- Art: `native_copper` block tile and item icon (today recoloured from the flint nodule's), the
  knapping screen's squares in copper colours (today flint's).
- The stone anvil; annealing; bronze cast at the crucible into the same moulds; iron's heat (the
  bloomery); `notes/CONTENT.md` tier 3 and `notes/CRAFTING.md` slice 11. Whether the book's
  three-ingot copper heads go once casting is played. Art for malachite, bellows and the moulds,
  each mould's top showing its head.
- Settled 11-26 (Planner): copper's harvest is one number, `vitals::gear::COPPER_HARVEST` 170,
  which felling's copper tier reads; the notes say 170.

## Check

`mods/testkit/scripts/cov_copper_1126.txt`: a census of loose copper in a cold and a dry land,
then on a pad in survival a lump picked up, put down, struck (a knock), hammered cold on the
screen into a pick head (a strike inside first: no loss), hafted into a copper pick, the clock's
`copper` and `copper_tool`, and raw copper's hint.
