# Crafting

Status: decided 2026-11-04. Slice 1 is built and waiting for a game run; the rest follows the
slices at the end.

**Ronald's decision:** "Yes, 1A 2A 3A 4B". So:
- the grid with a book that fills it
- recipes found by touching their materials
- hands-on making only the first time
- parts at Tinkers' depth and further

His words along the way:
- "probably something in between MC and Vintage Story"
- items "broken down into pieces", "maybe up to 10 rarity variants", combined "kinda like
  Tinkers' Construct... take it even further, customization wise"
- "armor should be pieces too"
- On the depth: "i do like the depth if that makes sense".
- On the first part icons: "pieces also look weird, maybe need to be a few less pieces", then
  "not 100 percent about less tbh, maybe it can be fixed with textures". So the parts stay, and
  the icons' look is fixed in the art (below, "Fewer pieces in the picture").
- "the system should be automatable later on" (below, "Automation later").

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

## Content's decisions, as they touch crafting (11-04)

- **The ladder stays open at the top.** Ronald: "we will probably have more ores / items with
  higher tiers". Wood and bone, stone, copper, bronze, iron, steel, then diamond as a real tier
  above steel, and more later.
  - A new tier is a material row (a package's too).
  - Grades and stats scale past steel by the same rule, with no code change.
  - Diamond's harvest level is 400, above steel's 300.
- **Taken as agreed unless he objects** (Planner): gold for ornaments, two clothing layers plus
  a cloak, biomes gating materials, stations with add-ons, crafts that take real time, and the
  room check.

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
  - **Grade 10** needs 92+ points, more than work from memory can reach (40 + 30 + 20): hands-on
    work with the best material at the best station. Luck can lift a 92-point part from 9.
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
- **Chiacraft's work** (its answer, 11-04):
  - **Eight steps per part, not six:**

    | Step | What it is |
    |---|---|
    | 0 | outline, shadow side |
    | 1 | outline, lit side (the guide's two-tone outline) |
    | 2–6 | the fill |
    | 7 | the highlight: metal's near-white bevel, gem sparkles |

    Hue shift lives in each ramp (gold's orange shadows and lemon highlights), so the map holds
    only brightness. Each layer brings its own outline steps, which gives the guide's "outline per
    part".
  - **Detail is the risk, not colour.** Approved icons have 50–150 colours of painted
    micro-variation, and an earlier palette "cleaning" mangled them. So:
    - each part's steps are fitted to its own reference drawing, not to one fixed palette
    - grain stays an overlay
    - composed icons are compared with the approved full icons before the method is adopted
  - **Overrides are needed for more than obsidian:**
    - knapped stone and flint (chipped edges, not a smooth bevel)
    - gems and diamond (facets and sparkle)
    - wooden heads against metal ones (grain direction, thickness)
    - glass
    - Ronald approved crude-looking wood and stone tools beside polished metal, so those get their
      own shape variants.
  - **Sockets** are fixed per kind on the 32 px diagonal. Heads must read at slot size (Ronald's
    hoe lesson: a blade about 10 × 4–5 px), and the guide's "Learned from Ronald's answers"
    applies to every part.
  - **Ramps** come from the 161 approved icons (iron, gold, diamond, wood, stone, leather, bone).
    New materials (bronze, copper, flint, obsidian, antler, sinew) get a reference icon first, and
    the ramp comes from that.
  - **Part maps:** `art.py` draws each part alone, in its socket, in one reference material, and
    it is snapped the `hybrid.py` way, with the usual checks and the archive.
  - **Full icons stay** for every fixed item, and as the target the composed tools must match.
  - **First, one test:** an axe (3 parts) and a sword (4 parts) in 6 materials, composed and shown
    beside the approved icons for Ronald to judge. Then the full run of about 40 part maps and 25
    ramps, the armour regions (blocky-model, painted in code with the same maps) and the grade
    overlays.

### The art format (for Chiacraft)

- **Parts in the data are not layers in the picture.**
  - An item keeps all its parts: an axe is head, haft and fitting; a sword is blade, guard,
    grip and pommel.
  - Its icon is drawn from at most 2–3 layers, whatever reads at 32 px: an axe as head and haft,
    with the lashing painted inside the head layer; a sword as blade and hilt, with guard, grip
    and pommel in one hilt layer.
  - A layer's map says, for each pixel, which part's material paints it, so the lashing still
    shows sinew or rawhide, and the hilt still shows a bronze guard on a leather grip.
- **One layer** is a 32 × 32 PNG on the kind's canvas, at the kind's sockets, plus a small
  `.json` beside it:
  - **Red** is the shade step, 0–7, stored as step × 36 (0, 36, ... 252):
    - 0: the outline, shadow side
    - 1: the outline, lit side
    - 2–6: the fill
    - 7: the highlight
  - **Green** is the region: which of the item's parts paints the pixel (0 the layer's own part,
    1 the fitting or the next part, and so on, as the `.json` lists them).
  - **Alpha** 255 where the layer is, 0 elsewhere.
  - **The `.json`:** `{ "kind": "base:axe", "layer": "head", "regions": ["head", "fitting"],
    "steps": 8 }`.
- **A material** is a strip of 8 colours, one per step, as an 8 × 1 PNG (the guide's ramp with
  its two outline tones and highlight). An optional grain is a 32 × 32 greyscale PNG where 128
  leaves a step alone, and darker or lighter moves it one step down or up.
- **Overrides:** a material can bring a whole painted layer for a shape (flint, obsidian, gems,
  glass, wooden heads), used instead of the map painted with its ramp.
- **Grades:** craft quality within one material, never a material tier. 1–2 rough and chipped,
  3–8 plain, a cleaner edge from 7, 9–10 a glint. Material tiers are the materials themselves.
- **Sockets**, settled by Chiacraft and Ronald 11-04 ("Yes, do the rest": all pieces, made
  readable through texture). On the 32 × 32 grid, handle bottom left, boxes x0, y0 – x1, y1:

  | Kind | Socket (where they meet) | Layers |
  |---|---|---|
  | Axe | 14.9, 14.8 | head 13, 0 – 31, 20, with the joint band (lashing or collar) 11, 11 – 18, 16 as its region 1; haft 0, 10 – 17, 31 |
  | Sword | 8.1, 23.4 (the guard) | blade 6, 0 – 31, 26; hilt: guard 3, 17 – 15, 28 as a region, grip and pommel 0, 23 – 8, 31 |

  Pickaxe, shovel and hoe come next; `crafting::looks` holds each kind's as it's fixed.
- **How the files come:**
  - **Working parts:** each material's working part is a fully painted layer (an override),
    because a recoloured map lost crude flint and stone, gold's shape and diamond's facets.
    Ronald approved drawn heads per material.
  - **Handles:** both painted (leather-wrapped with gold fittings, plain wood) and as maps, so
    oak, birch and antler are ramp materials.
  - **Every layer** has its `.json`.
  - **The first materials:** wood, stone, flint, copper, bronze, iron, steel, gold, diamond (the
    sword without flint).
  - **Grade overlays** come after the tools.
- **First test** (Chiacraft, now):
  - an axe and a sword
  - 6 materials: flint, copper, bronze, iron, steel, and gold for the sword
  - grades 1, 5 and 10
  - shown beside the approved full icons

### The engine hook for looks (built 11-04, waiting for a PC run)

One stack component, `core:look`, says how a stack is drawn: its layers back to front, each
`<texture>` (painted, used as it is) or `<map>@<ramp>,<ramp>...` (a shade map painted with one
ramp per region), separated by `;`. Base writes it when it assembles an item (`crafting::looks`
names the textures). The engine:
1. **Composes the picture once per distinct look.** A texture entry of a new kind, built from its
   layer files and ramps (red / 36 is the step, green the region), cached by the look's text,
   loaded on first use like any texture.
2. **Draws icons by the stack:** `item_icon` and `slot_draw` take the stack, not the item id, and
   use the composed texture when the stack has a look. They take the item's own icon otherwise.
3. **Builds held models by the stack:** the texel boxes are keyed by (item, look) and read the
   composed pixels.
4. **Syncs the look for other players:** today only their held item id travels; the look's text
   goes with it.

It's rendering, so it goes to the PC first. It's general: any mod can draw a stack from layers.
An upstream candidate (PLAN.md rule 4).

How it was built:
- **Composing:** `client/stack_look.rs` (`compose`, a pure function with a test).
- **Textures:** `Source::Look` entries in `client/textures.rs`, lazy, cached by text, at most
  4096.
- **Icons:** `stack_item_icon` in `ui/render.rs`.
- **Held and dropped models:** keyed by (item, look) in `client/entity_render.rs`.
- **Other players:** their held look travels as `core:held_look`.
- **SDK:** `with_look`, `stack_look`.
- **Base:** writes a tool's look when it's assembled (`parts::look_text`). Placeholder axe layers
  stand in for Chiacraft's.
- **Not yet:** looks in rich-text item icons (they have only an item id), and a look on loose
  part items.

### Fewer pieces in the picture

Ronald found the first part icons too busy at 32 px. The parts in the data stay as they are; the
picture shows fewer pieces (above).

If the art can't make 3–4 parts read and he wants fewer to craft too, the fallback is 2 crafted
parts per item:
- tools: a head and a handle, with the lashing, rivets or wedge an ingredient of assembly that
  sets a trait and its wear, but isn't a slot of its own
- swords: a blade and a hilt
- armour: a shell and a lining, with the fastening part of the shell

It's a data change only: kinds lose a slot, the fitting becomes an assembly ingredient with its
own wear, and nothing else moves. Not taken unless he asks.

### Automation later

Ronald: "the system should be automatable later on". Not built now, but the design leaves room:
- **One function for every craft.** Crafting and assembly go through one function in base
  (`book.rs`'s `craft` today, the parts' assembly later) that takes its inputs from a list of
  slots and gives its outputs to slots. The player's screen and a machine both call it.
- **Machines feed stations.** Stations are machines (`docs/machines.md`) with input and output
  slot groups, so hoppers feed them and empty them today, and pipes or conveyors later.
- **Queues and fuel.** A queue runs without a player nearby when its station has fuel or power
  (`docs/machines.md`, energy).
- **An autocrafter block** holds a recipe or a part shape from its owner's book and makes it on
  a redstone signal or while powered (`docs/redstone.md`).
- **Machines and grades.** A machine's work is a fixed score, about Good (grade 5) with good
  materials. Only hands-on work reaches the top grades. So automation saves time without making
  craftsmanship pointless. A machine-made part's history says so: "made by the autocrafter at
  ...".

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

## The choices (decided: 1A 2A 3A 4B)

Kept as they were put to Ronald, for the reasons.

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

1. **The screen and nearby chests** (built 11-04, `mods/base/src/book.rs`, waiting for a game
   run):
   - The book sits beside the inventory's 2 × 2 (hand recipes only) and the crafting table.
   - It has tabs by creative tab, pages, and the recipes you can make first.
   - Picking one shows its ingredients with have/need.
   - Craft, Craft 5 and All take from the bag, and at a table also from every chest within 7
     blocks of it. The grid still works by hand.
   - The book lists every mod's crafting recipes from the engine, so nothing moved into a crate
     yet; that comes with parts.
   - Engine additions: `recipes(kind)`, `Recipe::inputs`, `shape` and `accepts` (host
     `recipe_input` and `recipe_accepts`), `Client::rebuild_screen` (`client_op::REBUILD_SCREEN`),
     `Ui::button_with` and `InventoryGroup::slot`.
   - Not yet: filling the grid itself from the book (Minecraft's ghost placement), and the
     queue.
2. **Parts, first piece** (built 11-04, `mods/base/src/parts.rs`, waiting for a game run):
   - **Part items:** one per shape (`base:part_<shape>`), carrying material, grade and wear.
   - **Making them:** from the book's Parts tab at a crafting table (`base:parts` recipes, each
     naming its material). The grade is rolled from material, work from memory and the station.
     Wooden heads come from planks; hafts, grips and shafts are split from a log of their wood.
   - **Kind items** (`base:axe` ...) carry their parts and their own tool numbers on the stack
     (the engine hook).
   - **The table's row:** four assembly slots, the tool they make, and an "apart" slot that
     takes a tool back into its parts. Lashings are cut; wedges and rivets come back.
   - **Not yet:**
     - the look hook (every part and kind shows a plain icon for now)
     - broken items that stay
     - history
     - per-part repair
   - **Today's materials only:** wood, stone, bone, iron, gold, diamond, string, leather.
     Content's flint, sinew and bronze come with its step 1.
   - **Test:** `mods/testkit/scripts/crafting_parts.txt`.
3. **Skills** (built 11-04, `crafting::skills`, `parts.rs`):
   - One skill per craft (knapping, carving, casting, smithing, handwork, sewing), rising with
     every part made.
   - A skill's level is work from memory, capped at 30.
   - The book shows the skill and level for a part.
4. **Discovery** (built 11-04, `mods/base/src/discovery.rs`, waiting for a game run):
   - **Held items:** saved per player as a bitset, kept through death.
   - **The book:** shows a recipe once all its ingredients have been held, and a "?" with "2 of
     3" once some have. Creative shows all.
   - **Not yet:** R and U filtered, and picking a near miss to see which ingredients you lack.
   - **Test:** `crafting_discovery.txt`.

   The plan it follows:
   - A recipe unlocks once you've touched all its ingredients, as Valheim does. Minecraft's book,
     which unlocks on the first, floods (the Researcher, 11-04).
   - Near misses show as "2 of 3", with the missing ingredient as "?".
   - R and U show only what you know.
   - Slice 1's book shows every recipe until then.
5. **Stations, add-ons and the queue:** the workbench levels; the queue with sub-parts and
   refunds; cold hands and the warm room.
6. **The rest of the parts core** (4A):
   - kinds, part shapes, the grade rules and traits in the crate
   - part items with their components
   - the Assemble tab: build, swap, break down
   - stats from parts through the two engine hooks (Planner's yes): an item's look from its
     stack, and per-stack chains for mining speed, harvest level and maximum durability
   - assembly through the same one craft function a machine will call (automation later)
   - wear and repair per part
   - history (who, when, where)
   - Today's tools become assembled kinds from wood and stone parts. Art: Chiacraft's first part
     maps and ramps.
7. **Knapping**, the first hands-on game: flint, chert and obsidian heads and points (needs
   Content's stone items).
8. **Armour from parts:** shell, fastening and lining, with linings as Survival's clothing
   warmth. Worn region textures.
9. **Further** (4B, Ronald's pick), in this order:
   1. **Part shapes with trade-offs:** a narrow felling head or a bearded one; a long or short
      blade.
   2. **Origin and condition count:** seasoned or green wood, prime hides, ore deposits, the
      bronze ratio.
   3. **Parts improve with use:** counts per part, worn-in hafts, honed edges that need
      re-honing.
   4. **Masterwork naming and the world's chronicle**, deeds in the history.
   5. **Signed history through portals**, with Phase 4's world DIDs.
10. **Slow crafts** as machines (the tanning frame first), then **smithing and casting** with
   Content's copper, bronze and iron.

## Who does what

- **Content plan:** which items, materials, stations and add-ons exist, and their numbers in
  `vitals::gear`. The material rows above are proposals to agree with it.
- **Survival:** the pot, rack and campfire stay its own. The lining's warmth uses its clothing
  numbers, and stamina per swing uses its stamina.
- **UI redo:** styles every screen here: the book, Assemble, the tooltip, the grade colours.
  Its notes on the mock-up (11-04):
  - **Classes:** top tabs use class `tab` and sidebar categories class `category`, with
    `selected(true)` for the open one.
  - **Font:** PixelOperator Bold, with Pixeloid Sans Bold for Russian.
  - **Colours:** text is cream `#e4dacf`, dim notes `#c6a383`, and only the recipe name is
    heading gold `#f2da9a`. Gold is for state only ("can make now"). Have/need is green
    `#8ac80f` and red `#d8221e`.
  - **Buttons:** Craft is the main (orange) button; −, +, max and "From memory" are the dark
    secondary one. "To the forge" and "Quench" are main only when they're the step to take now.
  - **Placement:** slots are 18 GUI and touch, with no gaps. Labels sit 3 GUI from the frame.
  - **Heat and shape bars:** `progress_bind` with a frame and a `_fill`. The marks go inside the
    fill picture, or on a second thin bar, until the `at` ABI change (Phase 6's API 1.2).
- **Chiacraft (art):** part maps, material ramps and grains, armour regions, grade overlays,
  icons for stations and add-ons.
- **Phase 4 (DIDs):** signing history entries with world DIDs.
- **Portals:** carrying items with their history; the world rules for foreign items.
- **Tester:** every slice in the game.
