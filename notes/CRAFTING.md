# Crafting

Status: proposal, revised 2026-11-04 with Ronald's direction. Nothing is built.

His direction:
- "probably something in between MC and Vintage Story"
- items "broken down into pieces", "maybe up to 10 rarity variants", combined "kinda like
  Tinkers' Construct... take it even further, customization wise"
- "armor should be pieces too"

Choices 1–3 are with him now. Choice 4, how deep the parts go, is new.

- **Research, with sources:**
  - `reports/crafting/Crafting in other games.md`: Valheim, Vintage Story, TerraFirmaCraft,
    Enshrouded, V Rising, Minecraft, Factorio, Hytale.
  - `reports/crafting/Items built from parts.md`: Tinkers' Construct 3, Construct's Armory, Tetra,
    Silent Gear, Dwarf Fortress, Bannerlord.
- **Mock-ups:**
  - `notes/crafting/mockup.png`: the workbench, stations with add-ons, knapping, the anvil.
  - `notes/crafting/parts-mockup.png`: building from parts, an item's history, a mail coat from
    parts, the ten grades, one set of part art in every material.
  - The swords in the second are real output of the archived parts-sword prototype. The axes are
    today's stone axe cut into parts by colour and repainted.
- **Related plans:**
  - `notes/CONTENT.md`: the material ladder and the stations. Its open decisions 5 and 6 are the
    same questions as here.
  - `notes/SURVIVAL.md`: comfort, temperature, clothing.
  - Ronald's UI sheet: its Crafting screen.
  - Planner's concept page.

## The idea

- **Everyday things** are made in the grid, as in Minecraft. A recipe book beside it fills in as
  you find materials and fills the grid for you.
- **Stations set the tiers** and grow with add-ons built beside them. They take ingredients from
  the chests around them, and crafts can be queued and repeated.
- **Tools, weapons and armour are built from parts.**
  - Each part comes from a material with its own numbers and a trait.
  - Each part has one of ten grades, from the material, the work and the station, with a little
    luck.
  - Any item breaks back down into its parts, and parts can be swapped.
  - Parts keep their history: who made them, where the ore or stone came from, what they've done.
- **Hands-on work only the first time.** The things that really are hands-on (knapping flint,
  shaping clay, forging metal) are short games the first time you make a part. After that you make
  it from the book.
- **Slow crafts happen in the world:** tanning takes days, drying follows the weather, the forge
  needs fuel, and metal cools in the cold.

## What we have

- **The engine (Weft):**
  - Recipe kinds of `#tags`, shaped and shapeless, with time, several outputs and hidden recipes.
  - `recipe_match`, `recipe_find` and `recipe_process`.
  - The recipe viewer, R and U.
  - Data packages' TOML recipes.
  - Machines with ticks and synced components.
  - Client→server channels, so a screen can send its clicks.
  - The world reads any block entity's inventory, so nearby chests work.
  - Stack components: anything an item carries (freshness and stews already use them).
  - The `DAMAGE` and `ATTACK_DAMAGE` value chains.
- **Base:**
  - the 2 × 2 and 3 × 3 grids (`base:crafting`)
  - the furnace
  - Survival's cooking pot, drying rack and campfire
  - XP for crafting
  - the crafting table counts as "a table" for comfort
- **Earlier work:** the parts-sword prototype (archived in
  `~/Coding/chiacraft-archive/sword-generator-untracked/notes/ui-codex/procgen/`).
  - It builds 32 px swords from grip, pommel, blade and guard layers on fixed sockets.
  - Each part has a material ramp and a 1–10 quality roll painted in: crude blades are chipped.
  - Its `hybrid.py` turns a generated sprite into a map of shade steps, so any material is a
    ramp swap.
- **What parts need from the engine.** Two additions, both small and general. Everything else is
  base and a new shared crate.
  1. **An item's look from its stack.** Today the engine picks an icon by item id only. A stack
     should be able to name its look as layers (a part sprite and a material palette each). The
     UI draws them and the held model uses them. Worn armour needs the same for its texture.
  2. **Tool stats from the stack.** Mining speed, harvest level and maximum durability are fixed
     per item type today. They need a value chain each, like `ATTACK_DAMAGE`, so a stack's parts
     can set them.

## What the player does

1. **Hands first.** The inventory's Crafting tab has the 2 × 2 grid and the book's hand recipes:
   sticks, cordage, a campfire, a flake struck off a flint nodule.
2. **A workbench**, which is the crafting table renamed. It has the 3 × 3 grid, the book and an
   Assemble tab for parts.
3. **Add-ons raise a station's level.** Each different add-on within 4 blocks counts, in any
   order:
   - workbench: knapping stone, chopping block, tanning frame, sewing table
   - forge: bellows, anvil, quench trough
   - kiln: chimney, drying shelf
   - also the loom and the bloomery

   There's no roof rule. Content owns which stations exist; this plan owns how they work.
4. **The book.** Picking a recipe fills the grid from your bag and from the chests within 7
   blocks. Craft makes one; Craft ×N or shift-click queues more.
   - The queue runs while you're within 7 blocks of the station.
   - Cancelling refunds the ingredients.
   - Sub-parts are made one level deep: sticks from planks.
   - You can still place items in the grid by hand, and a shape nobody has shown you is how you
     find a new recipe.
5. **Slow crafts are machines in the world** that run while you're away, on Content's real times:
   the tanning frame, the drying rack, the kiln, the charcoal clamp, the bloomery. They report what
   happened ("fired 4 bowls, 1 cracked").

## Items built from parts

### Kinds and their parts

Real tools are built this way. A stone head is lashed to its haft with sinew. A bronze head is cast
with a socket. An iron head is forged with an eye and held by a wedge.

| Kind | Parts | Modifier slots from |
|---|---|---|
| Axe, adze, pick, hammer, hoe, shovel | head, haft, fitting (lashing, glue, socket rivets, wedge) | the head's grade |
| Knife | blade, grip, fitting (lashing, rivets) | the blade's grade |
| Spear, harpoon | point, shaft, fitting | the point's grade |
| Sword | blade, guard, grip, pommel | the blade's grade |
| Bow | stave (limbs), string, grip | the stave's grade |
| Arrow, dart | point, shaft, fletching (they stack: one make for 8) | none |
| Shield | boards or hide, rim, boss and grip | the face's grade |
| Armour (cap or helmet, coat, leggings, boots) | shell, fastening, lining | the shell's grade |

- **Armour:**
  - **The shell** stops blows: rawhide, leather, bone or antler slats, bronze scales, iron mail,
    plate.
  - **The fastening** holds it together: lacing, straps, rivets. It wears first and is cheap to
    redo.
  - **The lining** pads and warms: wool, linen, fur, quilted down. It's Survival's clothing in
    numbers: insulation in clo, how much it keeps when wet, blunt padding.
  - So the same mail coat takes a fur lining for a taiga winter and a linen one for the desert.

### Making parts

The first time you make a part shape, you make it by hand. After that it's in your book.

| Part from | How | Where |
|---|---|---|
| Flint, chert, obsidian | knapped: a 7 × 7 core, strike off what's outside the outline | knapping stone |
| Bone, antler | ground and carved | workbench |
| Wood (haft, shaft, stave, grip) | split and carved from a log of a kind; staves season for days | chopping block; seasoning is a slow craft |
| Copper | cold-hammered, softened in the fire | anvil |
| Bronze | cast into a mould, then the edge hammered | crucible and moulds, then the anvil |
| Iron, steel | forged at heat, then quenched | forge and anvil |
| Hide, leather, cloth | cut and sewn | sewing table; tanning is a slow craft |
| Cordage, sinew, rawhide, birch tar | by hand, or cooked over the fire (tar) | hands, campfire |

- **Moulds** are reusable stone or fired-clay blocks at the crucible, not items to carry. Tinkers'
  bags of casts and patterns are a known complaint.
- **Stacking:** parts with the same shape, material, grade and maker stack, so making eight
  arrowheads doesn't fill the bag.

### Assembling, swapping, breaking down

- **Assembling:** the workbench's Assemble tab. A kind's slots appear when you put a part in.
  - Metal fittings (rivets, a wedge, a socket) need the anvil nearby.
  - Armour needs the sewing table nearby (leather, cloth), or the anvil for mail and plate.
- **Swapping:** put a new part on a slot and the old one comes back to the bag. Lashings and glue
  are cut and lost; wedges and rivets come back.
- **Breaking down:** gives every part back with its grade, wear and history.
- **Broken items** don't vanish. A worn-out part leaves a "broken" item to repair, rebind or
  rebuild.

### How stats combine

One rule, shown in the tooltip, so nothing is hidden:

- **The working part sets the numbers** (head, blade, point, stave, shell): what it harvests, its
  speed or damage or protection, and its own wear. The material and the shape set them; the grade
  multiplies them.
- **The holding part multiplies them** (haft, grip, shaft): a material and shape bonus, such as
  +10% durability for oak, or +5% speed for birch.
  - It sums with the other holding parts like Tinkers' handles: total = working part ×
    (1 + the bonuses).
  - Its length sets reach and swing speed; its mass adds to the weight.
- **The fitting and extras add a trait and have their own wear.** No numbers of their own, so the
  sum stays readable. A stone axe's lashing goes long before its head does, as it really did.
- **Weight is every part's mass:** density × the shape's volume. It sets stamina per swing
  (Survival) and the armour's load.
- **Example**, as the mock-up shows it:

  | Stat | Value |
  |---|---|
  | Chops | flint head 3.0 × Superior 1.20 = 3.6 |
  | Wears out | binding first: 41, then the head 82, then the haft 96 |

### Materials and traits

Materials are Content's rows (`vitals::gear`), from real properties: density, hardness,
toughness, edge, flex, insulation, what water does to them. Each material has one trait, and the
trait can depend on the part it's in. Proposed:

| Material | As | Trait |
|---|---|---|
| Flint | head, point, blade | **Keen**: +15% on thin trees and hide; brittle on stone |
| Chert | same | **Common**: knapping and repair cost half |
| Obsidian | same | **Razor**: +25% damage, wears twice as fast |
| Bone, antler | head, point, grip, haft | **Tough**: never chips; as a grip, takes the sting out of blows |
| Copper | head, blade | **Soft**: hammered back to shape for free, but wears fast |
| Bronze | head, blade, scales | **Recastable**: a broken bronze part can be melted and cast whole |
| Wrought iron | head, blade, rings | **Weldable**: forge-welding repairs it fully |
| Steel | head, blade, plate | **Tempered**: holds its edge (wear −30%) |
| Oak, mesquite | haft | **Tough**: +10% or +15% durability, a bit slower |
| Birch, spruce | haft, shaft | **Light**: +5% speed; spruce gives a long, light spear shaft |
| Sinew, rawhide | fitting | **Shrinks tight**: wears half as fast while dry |
| Birch tar | fitting | **Glued**: doesn't loosen in the rain |
| Plant cordage | fitting | **Cheap**: rots when wet |
| Wool, felt | lining | **Warm when wet**: keeps 60% of its warmth soaked |
| Fur | lining | **Warmest**: 2.4 clo in a coat, heavy and slow to dry |
| Linen | lining | **Cool**: lets heat out in the sun |

- **Where a material came from matters:** fine flint from chalk against river cobble, a hide
  taken in autumn (prime) or summer, seasoned or green wood, the ore's deposit, bronze inside its
  real 88–92% copper or outside it. That's the "material" share of the grade below.
- **Traits are effects base already knows:** a stat change under a condition (wet, cold, against
  a kind of block or animal), wear speed, a repair rule. So a package can give a new material a
  trait from the list, with its own numbers, and write no code.

### The ten grades

| # | Grade | × main stat and durability | Modifier slots |
|---|---|---|---|
| 1 | Crude | 0.70 | 0 |
| 2 | Rough | 0.80 | 0 |
| 3 | Plain | 0.90 | 0 |
| 4 | Sound | 1.00 | 1 |
| 5 | Good | 1.06 | 1 |
| 6 | Fine | 1.12 | 1 |
| 7 | Superior | 1.20 | 2 |
| 8 | Exquisite | 1.28 | 2 |
| 9 | Masterwork | 1.38 | 3 |
| 10 | Legendary | 1.50 | 3 |

- **What sets a part's grade:**
  - **Material:** up to 40 points, from its kind and where it came from (above).
  - **The work:** up to 40.
    - Made by hand, it's how well you did: strikes inside the outline, moves in the right order,
      metal worked at the right heat.
    - From the book, it's your skill in that craft (knapping, carving, smithing, sewing), which
      rises by making things. From the book it stops at 30.
  - **The station:** up to 20, from its level and its tools.
  - **In tens:** 72 points is grade 7.
  - **A little luck:** one grade down 10% of the time, one up 17%, two up 3%.
  - **Grade 10** needs 90+ points: hands-on work with the best material at the best station.
    Grade 9 can roll up to it.
  - **There are no re-rolls to buy.** Silent Gear's catalyst grinding is a known complaint.
- **An item's grade is its working part's.** Every part keeps its own, shown in the item's
  tooltip.
- **How grades show:**
  - Five colour bands of two grades each. The second of each pair has a mark (+), and 10 has ☼,
    so grades read without colour, the way Dwarf Fortress marks quality.
  - The sprite shows it too: crude edges are chipped, masterwork ones have a glint (the prototype
    already paints this).
  - UI redo picks the colours.
- **Masterwork and Legendary** are announced in chat, can be named by their maker, and get a line
  in the world's chronicle.

### Modifiers and upgrades over time

- **Modifier slots** come from the grade. A modifier is a small addition made at a station:
  - a leather grip wrap (−10% stamina)
  - a fire-hardened point
  - a quench temper (steel)
  - an inlay of gold, silver or amber (trade value, looks)
  - fur trim on armour (+warmth)
  - waxed or oiled leather (sheds rain)
  - riveted reinforcement
  - Each kind of modifier once per item.
  - An integrity budget (Tetra's) caps how much a light item can carry.
- **Use makes an item yours** (Tetra's honing, Tinkers' tool leveling):
  - Every part counts what it does: trees felled, ore broken, animals killed, blows taken.
  - At set counts a part earns one improvement: a haft "worn in" (+handling), an edge "honed" at
    the whetstone (+keenness, which needs re-honing as it wears), armour "fitted" to its wearer.
  - Each part has its own few steps, capped.

### Wear and repair, per part

- **Each part wears by what it does.** The head wears on every use, a haft rarely, a fitting with
  hard blows and wet weather, an armour shell with every blow it stops. The item's bar shows its
  weakest part.
- **Repair is at the station of the part's tier,** with a little of its own material:
  - a few strikes at the knapping stone for a stone edge
  - a hammer for copper
  - recasting for bronze
  - welding for iron
  - Each repair takes a little off the part's maximum, unless a skilled hand does it.
  - Repair is never free. Valheim's free repair makes wear meaningless.
- **Some parts are replaced, not repaired:** a cracked haft, a frayed lashing. The rest of the
  item keeps its grade and history.

### History

- **What each part records:**
  - who made it: name, and the player DID once Phase 4 has them
  - when: game day, season and year
  - where: the world, the place and biome, and where its material came from
  - how: by hand or from the book, and its work score
  - what it has done: the counts above
  - its repairs and swaps
- **The tooltip** tells it in a few lines, as in the mock-up's "Wolfbiter". The rest is in the
  item's page in the journal.
- **Size:** each part keeps its last 8 entries plus its counts, so an item stays small.

### Through portals and in multiplayer

- **An item carries everything it needs.** Each part holds its material's id and a snapshot of
  its numbers. A world without that material (another package set) still shows and uses it right,
  with the material's name and a plain look.
- **Proof of history:**
  - Each world signs the history entries it writes with its world DID.
  - Another world shows them with their world's name, and "unverified" unless that DID signed
    them and you trust it.
  - Items made in creative say so.
  - A world can refuse foreign items above a grade (its rules), like the portal admission rules.
- **Gear never changes on arrival.** Parts are only ever changed by the world you're in.

### How parts look

As Tinkers' Construct 3 does it, which our prototype already proved at 32 px:

- **Icons:**
  - Each part shape is drawn once, at the kind's fixed sockets on a 32 × 32 canvas, as a map of
    shade steps (about 6) rather than colours.
  - Each material is a ramp of 5–8 colours from the item style guide, plus an optional grain (wood
    grain, flint's ripples, mail rings, wool).
  - An item's icon is its part layers, back to front, each painted with its material's ramp in the
    UI shader.
  - New materials cost a ramp, not new art.
  - A material can bring a hand-painted sprite for any shape where a ramp isn't enough (obsidian's
    glassy shine).
  - The grade adds a small overlay: chips at 1–2, a glint at 9–10.
- **Held:** the held model is the same layers, extruded, as items are held today.
- **Worn armour:**
  - Each armour piece is a box model (blocky-model) whose texture has regions per part: the shell
    over most of it, straps and rivets as strips, the lining showing at the collar and cuffs.
  - Each region is a map of shade steps painted with its material's ramp, the same way.
- **Chiacraft's work:**
  - one shade-step map per part shape, about 40 to start
  - one ramp and an optional grain per material, about 25
  - the armour region maps
  - the grade overlays
  - The parts are made with Codex at the kind's sockets and turned into maps with `hybrid.py`'s
    method, then checked like any icon batch.

### Data, so packages can add to it

Base's own rows live in Rust tables in the shared crates, for checks and sheets: Content's
materials in `vitals::gear`; kinds, part shapes, grades and traits in `mods/base/crafting`.
Packages add rows of the same shape as TOML:

```toml
# defs/materials/meteoric_iron.toml
name = "material.mypack.meteoric_iron"
tier = 5                       # harvests like wrought iron
density = 7.9
hardness = 6.2
toughness = 7.0
edge = 0.8
parts = ["head", "blade", "point", "rings"]
made_at = "base:forge"
trait = { effect = "wear", factor = 0.7, when = "always" }
ramp = ["#2a2a33", "#45454f", "#6a6a78", "#9090a0", "#c0c0cc", "#ecece8"]
grain = "mypack:grain/meteoric"   # optional
source = "#meteorites"            # what it's made from, for the checks
```

```toml
# defs/parts/bearded_axe_head.toml
kind = "base:axe"
role = "working"
volume = 160                   # cm³, for its mass
stats = { chop = 1.0, damage = 1.15, speed = 0.95 }
sprite = "mypack:parts/bearded_axe_head"   # a shade-step map at the axe's sockets
pattern = "knap:bearded_axe"   # or "cast:...", "forge:...": the hands-on shape, the first time
```

`defs/kinds/` is the same idea for a new kind: its slots, its sockets, its stats. Recipes for
everything else stay `defs/recipes`, with a station's kind (`kind = "base:workbench"`).

## How it ties to the real world

- **Materials decide the numbers:** real hardness, toughness and density; flint against chert;
  a prime autumn hide; bronze in its real ratio.
- **Heat is real at the forge and kiln:**
  - Metal cools by its mass and the air (`air_at`): cold, wind and rain cool it faster, and bigger
    pieces hold heat longer.
  - Iron is worked at about 900–1250 °C; good pottery needs about 900 °C.
  - All these numbers go in the data with sources.
- **Clay dries** where it stands, as the rack dries food.
- **Cold hands:** being Freezing doubles hand-work time. A warm, comfortable room (comfort 3+)
  makes crafting 25% faster.
- **Seasons set the work** through the materials: hides in autumn, flax in summer, charcoal in the
  dry months.
- **Linings are clothing:** an armour lining's warmth is Survival's clo, so the parts system and
  the cold are one system.

## The choices for Ronald

Choices 1–3 are as he has them now, written to his direction.

### 1. The everyday screen

| | What it is | For | Against |
|---|---|---|---|
| **A. The grid, with a book that fills it** (recommended, his direction) | Minecraft's 2 × 2 and 3 × 3 grids stay. Beside them is a book of known recipes; picking one fills the grid from your bag and nearby chests, with Craft ×N and a queue. | Familiar and quick. The book removes the chores. | The most screen to build. |
| B. A list only (Valheim) | No grid. | Simplest. | Loses Minecraft's grid. |

### 2. How recipes are found

| | What it is | For | Against |
|---|---|---|---|
| **A. By touching materials** (recommended) | A recipe appears once you've held one of each ingredient and its station exists. "?" hints show what's next. R and U show only what you know. | Ties progress to exploring; seasons and places matter. | A recipe can be missed. |
| B. Everything in the handbook | All visible, R as today. | Easy to plan. | No discovery. |
| C. A, plus knowledge found in the world | Some recipes come from an old smith's notes in a ruin, a trader, or a linked world. | Gives exploring and portals a reason. | Needs ruins and traders first. |

### 3. Hands-on making

| | What it is | For | Against |
|---|---|---|---|
| **A. The first time only** (recommended, his direction) | Knapping, clay shaping and a few anvil moves the first time you make a part shape; after that, from the book. Hands-on work can reach a higher grade. | Feels earned; never a chore. | Three small games to build. |
| B. None | List crafting with times. | Least work. | Stone and metal feel the same. |
| C. Always by hand | Vintage Story, TerraFirmaCraft. | Most real. | The known tedium. |

### 4. How deep the parts go (new)

| | What it is | For | Against |
|---|---|---|---|
| A. Tinkers' depth | 3–4 parts per kind; materials with numbers and a trait; ten grades; modifier slots; swap and break down; wear and repair per part; history (who, when, where); armour from shell, fastening and lining. | A known, loved shape; the least to build of these. | Ronald said "take it even further". |
| **B. Further** (recommended) | A, and: a few shapes per part with trade-offs (a narrow felling head or a bearded one, a long or short blade); the material's origin and condition count (seasoned wood, prime hides, ore deposits, the bronze ratio); parts improve with use (worn-in hafts, honed edges); naming and the chronicle at Masterwork; deeds in the history; armour linings swapped by season. | Every item tells where it's from and what it's done. It ties parts to our real materials and seasons. Each step builds on A. | More rows, more art (part shapes), more to balance. |
| C. Furthest | B, and the shape you make is the part: a knapped or forged head's own shape sets its numbers (length for reach, mass for force, Bannerlord style), and its sprite is drawn from that shape. | Every item unique. | Far more to build and balance; the art is generated from shapes, not painted. |

My recommendation: 1A, 2A, 3A, 4B, built so that A's core comes first.

## What gets built, in slices

Each slice is seen in the game through Tester. Strings are keys; recipes, materials and parts
are rows.

1. **The screen and nearby chests:** the grid with the book, filling from the bag and chests
   within 7 blocks, Craft ×N, on today's recipes. The `crafting` crate holds today's recipes.
2. **Discovery:** known recipes per player, "?" hints, R and U showing only what you know.
3. **Stations, add-ons and the queue:** the workbench levels; the queue with sub-parts and
   refunds; cold hands and the warm room.
4. **Parts, core** (4A):
   - kinds, part shapes, the grade rules and traits in the crate
   - part items with their components
   - the Assemble tab: build, swap, break down
   - stats from parts through the two engine additions
   - wear and repair per part
   - history (who, when, where)
   - Today's tools become assembled kinds from wood and stone parts. Art: Chiacraft's first part
     maps and ramps.
5. **Knapping**, the first hands-on game: flint, chert and obsidian heads and points (needs
   Content's stone items).
6. **Armour from parts:** shell, fastening and lining, with linings as Survival's clothing
   warmth. Worn region textures.
7. **Further** (4B): part shapes, origin and condition, use improvements, naming and the
   chronicle, signed history through portals.
8. **Slow crafts** as machines (the tanning frame first), then **smithing and casting** with
   Content's copper, bronze and iron.

## Who does what

- **Content plan:** which items, materials, stations and add-ons exist, and their numbers in
  `vitals::gear`. The material rows above are proposals to agree with it.
- **Survival:** the pot, rack and campfire stay its own. The lining's warmth uses its clothing
  numbers, and stamina per swing uses its stamina.
- **UI redo:** styles every screen here: the book, Assemble, the tooltip, the grade colours.
- **Chiacraft (art):** part maps, material ramps and grains, armour regions, grade overlays,
  icons for stations and add-ons.
- **Phase 4 (DIDs):** signing history entries with world DIDs.
- **Portals:** carrying items with their history; the world rules for foreign items.
- **Tester:** every slice in the game.
