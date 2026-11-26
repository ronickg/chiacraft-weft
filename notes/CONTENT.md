# Content: tools, building, workstations, clothing

Status: draft, 2026-11-03. Everything here is proposed until Ronald says yes; the open decisions
are at the end.

`notes/PLAN.md` left game content out until the foundations stood. They stand now, and survival
and animals need things to make: tools to hunt and fell with, clothes for a taiga winter, houses
that keep a fire's heat in. Food already has its plan (`notes/ECOLOGY.md`: diets, plants, crops).
This is the plan for everything else: tools and weapons, building blocks and materials,
workstations, clothing and armor, and the progression that ties them together.

It works the way the ecology does. Each tier is a real technology, each material is where it
really is in the biome's real place, and each number comes from research, turned into the game by
one pacing. The research, with its sources:

- `reports/content/Tools and materials.md`: the first tools, weapons, the metals in order,
  fired materials, and where each material really is in each biome's place.
- `reports/content/Clothing and armor.md`: insulation in clo, what it takes at -40 °C, what
  people wore in each place, hides and fibres, armor and what it stops.
- `reports/content/Building and workstations.md`: houses in each place, how well each wall keeps
  heat, how the materials are made, the workplaces of each craft, furniture.

## The idea

1. **The ladder is real.** Wood and bone, knapped stone, fire and clay, copper, bronze, iron,
   steel. Not wood, stone, iron, gold, diamond.
2. **The biome you live in decides your first kit.** Chert on the plains, obsidian in the desert,
   basalt and bog iron in the forest, bone and caribou on the tundra. What isn't near you (tin,
   above all) is why you travel, trade, or link to another world.
3. **The seasons set the work.** Hides are prime in autumn, flax is retted in summer, charcoal is
   burnt in the dry months, and winter is for making things by the fire. A taiga winter needs a
   caribou suit made before it comes.
4. **Animals are the materials.** Hide, fur, wool, sinew, bone, antler, feathers, down and fat,
   from every species, by its real size. Hunting and herding feed the crafts, as they feed the
   stomach.
5. **Less code.** A few shared tables (materials, gear, clothing, walls) and one rule for animal
   parts; every ingredient a `#tag` (as `recipes.rs` already does); crafts that take time in real
   life (tanning, drying, charcoal) wait in the world on the growing pace.

## What exists

Weft's base game, with our survival and animals on top:

| Kind | In the game now |
|---|---|
| Tools | wood, stone, iron, gold, diamond × pickaxe, shovel, axe, sword, hoe; bucket; shears (as a tool kind for leaves) |
| Armor | leather, iron, gold, diamond × helmet, chestplate, leggings, boots; insulation per set in `vitals::temperature::set_insulation` (leather, iron, gold, diamond, and a fur row with no item) |
| Animals mod | club, tranquilizer dart, sedative, spyglass, saddles, nest, kibble, amber |
| Materials | stick, coal, raw iron and gold, iron and gold ingots, diamond, redstone, leather, bone, string, feather, gunpowder, arrow (no bow), paper, book, sugar, wheat, dung, compost, bandage |
| Blocks | dirt, grass, podzol, sand, gravel, clay, stone, cobblestone, mossy cobblestone, smooth stone, stone bricks, bricks, sandstone, obsidian, ice, snow, glass, stained glass, wool, hay bale; oak, birch, spruce and acacia logs, planks and leaves; red sand and red sandstone, fern, horsetail in the Cretaceous |
| Ores | coal, iron, gold, redstone, diamond (`worldgen.rs`, the same everywhere); amber (rare, more in the Cretaceous) |
| Workstations, furniture | crafting table (3x3), furnace, chest, bed, bookshelf, compost bin, tombstone, torch, ladder, glowstone and lamps |
| Weft's tech | the redstone set, pistons, dispenser, observer, lamps |
| Drops | Weft's mobs only (cow: leather, sheep: wool, chicken: feathers, skeleton: bones and arrows, spider: string); the animals mod's 35 species drop nothing yet |

Coming from the food plan, already textured (`claude/eloquent-boyd-5b6dc9`): mesquite, acuri
palm and moriche palm wood; cattail, sedge, cottongrass; the Cretaceous woods (ginkgo, araucaria,
swamp cypress, scale conifer, tree fern, cycad) and rotten logs.

## The shared things

Most of the content is rows in a few tables. Proposed homes, beside survival's numbers:

| Shared thing | What it holds | Where (proposed) | Read by |
|---|---|---|---|
| **Materials** | each tool material (flint, obsidian, bone, copper, bronze, iron, steel...): harvest level, durability, speed, damage, edge, weight, where it's repaired | `vitals::gear` (new) | `items.rs` (tools made from rows, as `MATERIALS` is now), the load weights, the sheet |
| **Gear kinds** | axe, pick, knife, spear, bow...: the kind's share of damage and speed, its recipe shape, its stamina per swing | `vitals::gear` | `items.rs`, `recipes.rs`, stamina |
| **Clothing and armor** | each set's cold and heat insulation, what it keeps wet, windproof, protection by blow kind, mass | `vitals::temperature` (grows from `set_insulation`) | survival's body, the sheet |
| **Wall classes** | each building block's class (hide, plank, log, earth, stone, snow, thatch) and what a room of it keeps of a fire's heat | `vitals::shelter` (new) | the air (`air_at`), the sheet |
| **Animal parts** | what a body gives, from its mass and family: hide, fur, sinew, bone, antler, feathers, down, fat | `animals_core::parts` (new) | the animals' drops, the species sheet |
| **Waiting crafts** | how long tanning, retting, drying, curing, charcoal, firing take, real days through the growing pace | `vitals::gear` | the stations that wait |

**Tags are the content's glue.** Recipes ask for `#hides`, `#fur`, `#sinew`, `#fibre`, `#binder`
(sinew, rawhide, birch tar, pine pitch), `#tool_stone` (flint, chert, obsidian), `#bark_tannin`,
never a single item. A new animal or plant fits every recipe by its tags.

## The pacing

Crafts that wait in real life wait in the game, on the growing pace crops and young already use
(`animals_core::biology::GROWING`: game days = 0.0645 · real days^0.7; a game day is about 20
minutes). Work done with the hands is instant or a few seconds.

| Craft | Real time | Game time |
|---|---|---|
| Kiln firing | 1 day | ~1 minute |
| Lime burning | 2 days | ~2 minutes |
| Brain-tanning a hide | 3 days | ~3 minutes |
| Water-retting flax | 5 days | ~4 minutes |
| Charcoal clamp | 7 days | ~5 minutes |
| Drying pots before firing | 10 days | ~6 minutes |
| Smoking fish or meat | 10 days | ~6 minutes |
| Adobe drying | 25 days | ~12 minutes |
| Brick clamp, start to finish | 40 days | ~17 minutes |
| Seasoning a bow stave | 180 days | ~2.5 game days |
| Bark-tanning sole leather | 365 days | ~4 game days |

So a buckskin is ready while you eat; sole leather for boots and armor takes most of a week, and
is worth starting before autumn.

## The ladder

| # | Tier | Tools and weapons | Building | Workstations | Clothing and armor | Pushes toward |
|---|---|---|---|---|---|---|
| 0 | **Hands and wood** | digging stick, club, fire-hardened spear | lean-to of branches and leaves | campfire | none | anywhere |
| 1 | **Stone and bone** | knapped knife, hand axe, hafted stone axe and adze, stone-tipped spear, atlatl, bone awl and needle, bone harpoon | hide tent, sod, thatch, wattle | knapping by hand, tanning frame, drying rack | rawhide, buckskin, furs, the caribou suits, bison robe; rawhide and slat armor | chert (plains), obsidian (desert, ocean), birch tar (birch forest); bone where there's no stone (tundra, savanna, swamp) |
| 1b | **Bow and sling** | self bow, arrows (stone, bone points), sling | | fletcher's bench | | yew and elm (forest), Osage orange (plains); the sling needs no wood (desert, beach, savanna) |
| 2 | **Fire and clay** | ground stone axe and adze (tougher, faster) | adobe, fired brick, clay tile, lime plaster, whitewash | kiln, charcoal clamp, grinding slab; spindle and loom | wool, linen, loden; gambeson | clay (forest, birch forest, swamp, savanna), limestone (birch forest, beach), flax and sheep |
| 3 | **Copper** | copper knife, awl, axe, points | | anvil stone (cold hammering); a kiln with bellows for smelting | | native copper (taiga), malachite (desert, beach) |
| 4 | **Bronze** | bronze axe, pick, sword, spear | | crucible and moulds | bronze armor | **tin**: rare, only near beaches and in mountain streams: travel or trade |
| 5 | **Iron** | iron tools, arrowheads, nails | nails, iron fittings | bloomery with bellows, forge and anvil | mail over a gambeson | bog iron (forest, birch forest, taiga, swamp); a mountain of charcoal, so forests |
| 6 | **Steel** | steel tools and weapons | | forge with a quench trough | plate | wherever there's iron and fuel |
| side | **Gold and silver** | none (too soft) | gilding, ornaments | | jewellery | placer gold (taiga, desert), silver and lead (mountains) |

What the research says about the numbers (Tools and materials, A3, C4):

- **Stone works.** A stone axe fells a thin tree almost as fast as steel, a thick one 4-6 times
  slower. Stone is sharp but brittle: short life, cheap to replace.
- **The big jump is stone to metal.** Bronze fells as fast as steel, and hammered bronze is harder
  than wrought iron. Iron won on supply (bog ore is everywhere, tin is 30 times rarer than
  copper), and because it becomes steel.
- **Charcoal is the cost of iron.** A kilogram of bloom takes about 7 kg of charcoal, about 60 kg
  of wood. The iron age eats forests.

Proposed numbers (durability against today's iron tool, 250 uses; speed against today's
wood 2 to diamond 8):

| Material | Harvest level | Durability | Speed | Edge | Repaired at |
|---|---|---|---|---|---|
| Wood | 0 | 60 | 2 | - | - |
| Bone, antler | 50 | 90 | 2.5 | tough | - |
| Knapped stone (flint, chert) | 100 | 75 | 3 | very sharp, brittle | knapping, cheap |
| Obsidian | 100 | 40 | 3 | the sharpest, shortest life | - |
| Ground stone | 120 | 125 | 4 | tough | grinding slab |
| Copper | 150 | 150 | 5 | soft | anvil, free |
| Bronze | 200 | 250 | 7 | hard | anvil, recast |
| Wrought iron | 220 | 275 | 7 | softer than bronze, tougher body | forge |
| Steel | 300 | 500 | 8 | the best | forge |

The engine's harvest levels are numbers a hundred apart so mods fit between them
(`abi::tool_tier`); the ladder needs no engine change. Ores' harvest levels move with it: a stone
pick for copper and tin ore, bronze for iron ore, iron for gold and silver.

### Tools and weapons

| Kind | Tiers | Used for |
|---|---|---|
| Knife | stone, obsidian, bone, every metal | butchering (parts from a kill), cutting fibre, skinning; the first tool |
| Axe | hand axe, hafted stone, ground stone, every metal | felling, chopping; a slow strong weapon |
| Adze | ground stone, metal | planks and shingles from logs |
| Pick, mattock | antler mattock, stone, every metal | stone and ore |
| Shovel, spade | wood, bone (shoulder blade), metal | earth, sod, clay, snow |
| Hoe | wood, stone, metal | tilling (as now) |
| Spear | fire-hardened wood, stone, bone, every metal | thrust or throw; big game at bay |
| Atlatl and darts | wood, stone or bone points | before the bow: stronger, shorter |
| Bow and arrows | self bow; stone, bone, copper, iron points | deer, birds, hunting at range |
| Sling | fibre or wool, stones | birds, small game; no wood needed |
| Harpoon | bone or ivory toggling head | fish, caiman; seals if they come |
| Club | wood (exists in the animals mod) | knocking out (taming) |
| Sword | bronze, iron, steel | fighting; no earlier sword is real |
| Snow knife | bone, metal | cutting snow blocks |

Today's sword, hoe and shovel stay as kinds; the pick and axe gain the earlier tiers. The arrow
exists without a bow; the bow comes in tier 1b.

## Where the materials are

From the research (Tools and materials, E1). Stone is under every biome in the world; what
differs is what lies loose at the surface and what the rock holds.

| Biome | Loose and surface | In the rock | Wood | Notes |
|---|---|---|---|---|
| Plains (Konza) | **chert** in gravel and limestone | limestone, shale | few trees; Osage orange for bows | the easiest stone start; sod for houses |
| Forest (Kassel) | **bog iron** in wet ground, clay | **basalt**, red sandstone, lignite (coal) | oak (tannin bark), yew, elm, hazel, willow | the iron-age heartland |
| Birch forest (Moscow) | bog iron, clay | **limestone** (with chert) | birch (**tar**, bark), linden (bast) | |
| Taiga (Fairbanks) | **native copper**, **placer gold** in streams | quartzite, schist, coal | spruce (**pitch**, logs) | cold: fur is everything |
| Snowy plains (Utqiaġvik) | no stone; driftwood, bone | (stone deep under permafrost) | driftwood only | bone, antler and caribou; the hardest start |
| Desert (Hermosillo) | **obsidian**, **malachite**, placer gold | granite, rhyolite, caliche (lime) | mesquite | adobe, cotton |
| Savanna (llanos) | no stone; clay | (deep) | moriche palm (fibre), acacia | bone, clay and palm |
| Swamp (Pantanal) | no stone; clay, bog iron | rock iron at the edges (Urucum) | acuri palm | bone, clay and palm |
| Mountains (Davos) | quartzite | gneiss, dolomite, **silver and lead** (galena), soapstone | spruce, larch | stone slabs, wool sheep |
| Ocean (Azores) | **obsidian**, pumice | **basalt** | | islands |
| Beach (Faro) | sea salt, shell | limestone, red sandstone, **copper and tin** (rare) | | the only tin |
| Deep caves | | the deep ores: iron, copper, galena, **tin** (rare), coal, diamond | | |

Tin is the one material the ladder hangs on that most players won't have near. That's real
(Bronze Age Europe traded Cornish and Iberian tin across a continent), and it's what linked worlds
are for.

## Building

What people really built in each place (Building and workstations, A), and what keeps heat:

- **Thatch and logs keep heat in; stone and brick don't.** A stone wall loses three to four times
  the heat of a log wall. Stone is for mass and fire safety.
- **Earth walls (sod, adobe, cob) hold the day's average.** An adobe room at Hermosillo's 40 °C
  afternoon stays near 32 °C, and is warmer than outside before dawn.
- **A snow house is the best-measured shelter**: 5 °C over the outside air empty, 20-45 °C over it
  with seal-oil lamps.
- **A hide tent stops wind and little else**; its liner makes it livable.

New blocks, by biome (all with their wall class):

| Block | Biome | From | Class |
|---|---|---|---|
| Chinked log wall | birch forest, taiga | logs + moss | log |
| Sod, turf | plains, tundra, taiga roofs | cut from grass with a spade | earth |
| Adobe brick | desert | clay + sand + straw, sun-dried | earth (erodes in rain) |
| Rammed earth | beach, desert | earth in a form | earth |
| Wattle and daub, half-timber frame | forest, swamp, savanna | sticks, clay, straw in a log frame | plank |
| Thatch: reed, straw, palm | forest, birch forest, savanna, swamp | cattail and reeds, wheat straw, palm fronds | thatch (burns) |
| Wood shingles | mountains, taiga | split spruce | roof |
| Stone slabs | mountains | split stone | stone |
| Clay roof tile | forest, ocean, beach | clay, kiln | roof |
| Birch bark | birch forest, taiga | birch logs | under sod roofs |
| Basalt, basalt bricks | ocean, forest | basalt | stone |
| Lime plaster, whitewash | beach, ocean, desert | lime, kiln | a coat on any wall; white keeps it cooler |
| Snow block | tundra, mountains | snow, cut with a snow knife | snow (melts above 0) |
| Hide cover (tent, tipi) | plains, tundra | sewn hides on poles | hide |
| Gut window, mica window | tundra; birch forest | seal gut; mica | dim window |
| Raised floor posts | swamp, savanna | logs | keeps floors over floods |
| Ramada | desert, savanna | mesquite poles, brush | roof only: shade |

How warm a room is (proposed, Building and workstations, B3): a roof alone keeps the night sky off
(+2 °C) and the sun (-4 felt); three walls stop the wind; a closed room with a fire adds by its
walls' class (hide +12, plank or stone +16, log, earth or snow +25, thatch roof +3 more); earth
and stone even out the day's swing. This needs survival's air to know a room (an enclosure check),
which is a decision below.

## Workstations

The real workplaces (Building and workstations, D), as few stations as the crafts allow:

| Station | Makes | Tier | New or has |
|---|---|---|---|
| Campfire, then hearth | cooking (food plan), heat, light, birch tar, fire-hardening, bonfire pottery | 0 | new (today fires are torches, lava and lit furnaces) |
| Crafting table, then carpenter's bench | wood and stone tools, furniture, doors | 0 | has |
| Tanning frame | rawhide, buckskin, dressed fur | 1 | new |
| Tanning pit | sole leather (bark-tanned, slow) | 1 | new |
| Drying rack, smokehouse | dried and smoked meat and fish (food plan), seasoned bow staves | 1 | new |
| Fletcher's bench | bows, arrows, atlatls | 1b | new |
| Sewing kit (an item) | clothes, covers, bags from hides and cloth | 1 | new |
| Spindle (an item), spinning wheel | yarn from wool, flax, cotton, nettle | 2 | new |
| Loom | cloth, blankets, gambeson cloth | 2 | new |
| Kiln | pottery, bricks, tiles, lime, crucibles | 2 | new (the furnace does some of it today) |
| Charcoal clamp | charcoal: a mound of logs covered with earth, lit, left | 2 | new (built, like the amber gateway's frame) |
| Grinding slab, quern | ground stone tools; flour (food plan) | 2 | new |
| Bellows | a kiln or forge add-on: hot enough for metal | 3 | new |
| Anvil (stone, then iron) | cold-hammered copper, forging, repair | 3 | new |
| Bloomery | iron blooms from bog ore and charcoal | 5 | new (built of clay) |
| Quench trough | steel (quench and temper) | 6 | new |

Crafts by the fire (spinning, sewing, carving, fletching) are home work; tanning, charcoal, kilns
and smelting are outdoor work. Winter evenings were craft time; working at a station inside a
comfortable home could go faster (proposed).

Furniture for comfort (survival's rested), each biome's own counting as the general kind: fur bed
and sleeping platform, hammock, bench and stool, table, rug (wool, hide, rush), wall hanging,
tallow candle, rushlight, oil lamp, seal-oil lamp (heat and light at once), masonry stove (a
Russian or tile stove, warm long after the fire dies), clay water jar.

## Clothing and armor

The numbers (Clothing and armor, A and E):

- **1 clo is about 12 °C of cold insulation for someone walking**, 7 °C at rest. A 4 clo double
  caribou suit keeps a walker comfortable at -30 °C; sitting still they'd need 8, which is why
  people slept under robes in a shelter by a lamp.
- **The game's own climates need much more than today's sets.** Utqiaġvik's winter night with wind
  feels about -41 °C and needs about 50 °C of cold insulation; Fairbanks about 45, Moscow 29,
  Kassel 23. Today's leather (8) and fur (16) cover Kassel at best.
- **Wet costs by material.** Wool keeps most of its warmth, soaked fur loses about 80%, down
  collapses, sealskin and gut keep you dry.
- **Clothing was always worn under armor.** Mail over a gambeson, plate over a padded doublet,
  Arctic bone armor over the parka.

### Two layers

Clothing slots (head, chest, legs, feet) for warmth and sun, armor slots (the same four) for
protection, and one cloak slot over both (a robe for warmth, a gut parka or oiled cape for rain).
Survival builds the slots and the rules (`notes/SURVIVAL.md`); this plan lists the items and
their materials.

### Clothing (warmth, no protection)

Full-set numbers; each piece gives its share (head 0.15, chest 0.4, legs 0.3, feet 0.15).

| Set | Made from | Cold | Heat | Wet keeps | For |
|---|---|---|---|---|---|
| Linen | flax, linden bast shoes | 6 | 0 | 0.2 | summer anywhere |
| Desert robe | cotton or linen, palm-straw hat, rawhide sandals | 5 | **+6** | 0.2 | desert, savanna, swamp |
| Buckskin | deer, elk or caribou leather, brain-tanned | 9 | -1 | 0.3 (smoked 0.5) | spring, autumn |
| Wool | spun and woven wool, leather soles | 14 | -2 | **0.6** | the forest's, ocean's and beach's winter |
| Loden | fulled wool over wool, felt | 24 | -4 | 0.6, windproof | the forest's winter, the mountains' autumn |
| Plains winter | bison robe, buckskin, fur-in moccasins | 30 | -8 | 0.4 | the plains' winter |
| Sheepskin | sheepskin coat, wool, felt boots | 32 | -8 | 0.5 | the birch forest's winter |
| Bird-skin | bird skins with down, caribou trousers | 28 | -8 | 0.15 | the coast without caribou |
| Caribou, single | ~2 caribou, sinew | 30 | -9 | 0.3 | the taiga's autumn |
| Caribou, double | ~4 caribou, wolf ruff, fox lining, sinew | **48** | -14 | 0.25, windproof | the taiga's and tundra's winter |
| Sealskin | seal, gut, sinew | 8 | -2 | 1.0, stays dry | the Arctic summer, rain, boats |

Cloaks: wool cloak (+4 cold), hare-skin robe (+8; about 85 hares), bison robe (+10, also
bedding), gut parka or oiled cape (+1, rain doesn't make you wet), down cloak (+12), feather
cloak from the Cretaceous's raptors (+8).

### Armor (protection, weight, heat)

Protection out of 10 against cuts, piercing (arrows, stabs), blunt blows and bites:

| Armor | From | Cut | Pierce | Blunt | Bite | Cold | Heat | Mass |
|---|---|---|---|---|---|---|---|---|
| Gambeson | linen or wool cloth, 16-30 layers | 4 | 3 | 4 | 4 | +10 | -6 | 3 kg |
| Rawhide, hardened leather | cow, bison, boar hide | 4 | 3 | 5 | 6 | +2 | -3 | 4 kg |
| Hide and wooden slats | elk or bison hide, birch slats | 5 | 5 | 5 | 6 | +2 | -4 | 7 kg |
| Bone or antler lamellar | antler, bone, sinew lacing | 6 | 6 | 4 | 6 | 0 | -4 | 8 kg |
| Caiman plate | caiman back hide with its bony plates | 5 | 5 | 4 | 7 | 0 | -4 | 6 kg |
| Protoceratops hide | Cretaceous | 5 | 4 | 6 | 6 | +1 | -3 | 5 kg |
| Bronze | copper and tin | 7 | 7 | 6 | 8 | -1 | -6 | 20 kg |
| Mail (over a gambeson) | iron rings | 8 | 6 | 6 | 8 | -2 | -5 | 10 kg |
| Plate | steel | 10 | 10 | 8 | 10 | -2 | -8 | 25 kg |
| Shield | linden boards, rawhide rim, iron boss | blocks | | | | 0 | 0 | 4 kg |

Corrections to today's numbers: leather's heat insulation is the wrong sign (thick leather traps
heat: -3, not +3); fur is clothing, not armor; iron's +4 cold belongs to the gambeson under it.
Heavy armor costs stamina by its weight (survival's carried weight and swings) and heat: plate in
a desert summer leaves no comfortable place in the sun, as in the real tests.

## Animal parts

The animals mod's 35 species drop nothing yet. One rule gives every body its parts, from its mass
and family, so a new species has parts the moment it's added:

| Part | From | Amount (real) | Used for |
|---|---|---|---|
| Hide | mammals, caiman, dinosaurs | about 7% of the body's mass green; area grows as mass^(2/3) (a deer about 1 m², a bison 4 m²) | rawhide, leather, buckskin, tent covers, armor |
| Fur | mammals with winter coats (prime in autumn and winter) | the hide, hair on | clothing, robes, trim (wolf and fox ruffs) |
| Sinew | big mammals' legs and backs | a few strands a deer | thread, bowstrings, binding |
| Bone | every vertebrate | | awls, needles, points, harpoon heads, the shoulder-blade spade, glue, lamellar |
| Antler | deer, reindeer; shed in late winter (found on the ground) | | flakers, mattocks, handles, lamellar |
| Feathers | birds, the feathered dinosaurs | | fletching, stuffing, cloaks |
| Down | birds; most from ducks (15-20 g a nest) | | down cloaks and quilts |
| Fat, tallow | every body by its condition | | candles, lamp oil, waterproofing, cooking (food plan) |
| Wool | sheep, sheared once a year (1-2 kg a primitive sheep) | | yarn, felt, cloth |
| Gut | big mammals, seals | | rain parkas, windows, bowstrings |

What a body gives also follows its life: a thin animal gives little fat, a winter coat makes the
best fur, and a young one a small hide. A kill's parts come with a knife; without one, only meat
and a torn hide (proposed).

## Plants for materials

Food's plant table (`vitals::plants`) grows these too; some are in it already, some are new rows:

| Plant | Gives | Biome | In the plant table |
|---|---|---|---|
| Flax | linen fibre, linseed oil | forest, birch forest | new |
| Cotton | cotton fibre | desert, savanna | new |
| Nettle | coarse fibre | forest | new |
| Moriche palm | leaf fibre for hats, hammocks, cord | savanna | textured, to register |
| Acuri palm, palm fronds | thatch | swamp, savanna | textured, to register |
| Yucca, agave | cord, sandals | desert | new |
| Reeds, cattail | thatch, rush mats, rushlights | swamp, forest, savanna | cattail textured |
| Wheat straw | thatch, adobe, bedding | wherever wheat grows | from wheat |
| Oak bark | tannin | forest | from oak logs |
| Birch bark | tar, roofing, containers | birch forest, taiga | from birch logs |
| Linden bast | cord, bast shoes | birch forest, forest | new tree (or from oak) |
| Spruce and pine resin | pitch | taiga, mountains | from spruce logs |
| Yew, elm, Osage orange | bow staves | forest, plains | new trees |
| Hazel, willow | wattle, baskets | forest, swamp | hazel textured |
| Moss | chinking | taiga, birch forest | new block |

## The Cretaceous

No people lived there, so it gives rocks, woods and animal parts, not metals:

- **Rocks:** chert in the floodplain's conglomerate (knappable), basalt in the fern forest,
  sandstone in the dunes and scrubland, limestone and quartzite in the crags, caliche for lime.
- **Woods:** araucaria, ginkgo, swamp cypress, scale conifer, tree fern.
- **Animal parts are its riches:** big hides (Protoceratops), feathers (the raptors: the best
  fletching and a feather cloak), large bones, claws and teeth for points.
- **Amber** is a game choice: none of our five formations has it in the sources (real Early
  Cretaceous amber is Lebanese and Spanish). It stays as the gateway's material, rarest in the
  dunes and commonest in the fern forest's wet conifer woods.
- The research also found that the dunes' and scrubland's formations (Djadokhta, Bayan Mandahu)
  are Late Cretaceous, 75-71 million years old, and the others Early. The dimension mixes the two
  already; worth saying so where it's described.

## What the other tracks give

| From | What this plan needs |
|---|---|
| **Survival** | clothing slots apart from armor, and a cloak slot; insulation per set and wet per material (the tables above); sleeping halves worn insulation (bedding and shelter make up the rest); armor's weight in stamina and heat; a room check for shelter warmth by wall class; repair at stations |
| **Animals** | `animals_core::parts` and drops for every species and Weft's farm animals; shearing, plucking down, shed antlers; butchering with a knife |
| **Living world** | rows for the fibre and material plants (flax, cotton, nettle, yucca, yew, elm, Osage orange, linden, moss); bark from logs |
| **World generation** | loose stone by biome (chert, flint, obsidian), bog iron in wet ground, native copper and placer gold in streams, ores by biome instead of one list for all |
| **Look** | icons (game-art) and held models (blocky-model) for every new item; block textures; wearable models for each clothing set and armor |

## Building it, in order

Each step can be played on its own:

1. **The stone start and animal parts.** Loose stones and flint by biome; knapping; hafted stone
   tools with a binder; bone tools; the campfire; `animals_core::parts` and drops for every
   species; the knife for butchering. Wood and stone tools of today become the stone age.
2. **Hides and the first clothes.** The tanning frame (rawhide, buckskin, dressed fur), sinew
   thread, the bone needle and sewing kit; buckskin, fur and the caribou suits, the bison robe;
   rawhide armor. Needs survival's clothing slots.
3. **Shelter.** Sod, thatch, wattle and daub, chinked logs, hide tents, snow blocks; wall classes
   and the room check.
4. **Fire and clay.** The kiln, pottery, bricks, tiles, lime, the charcoal clamp, ground stone.
5. **Fibre.** Flax, cotton and wool to yarn and cloth; spindle, wheel, loom; linen, wool, loden,
   the gambeson; desert robes.
6. **Copper and bronze.** Native copper and the stone anvil; bellows; crucibles and moulds; tin.
7. **Iron and steel.** Bog iron, the bloomery, the forge and anvil, quenching; mail, plate.
8. **The rest of building and furniture.** Shingles, tiles, whitewash, stoves, lamps, the
   furniture kinds for comfort.

**The first slice (proposed):** step 1. It changes how every game starts, needs nothing from
the other tracks, and the animal parts it adds are what clothing (survival) and every later step
build on.

## The checks

Like `places.rs` and `web.rs`, reading the tables, so a new item is checked when it's added:

- **Every ingredient has a source.** Each `#tag` a recipe asks for is a block in some biome, an
  animal part, or a plant's harvest.
- **Every biome reaches tier 1 at home**, with its own stone or bone; the later tiers name the
  biomes that hold their materials.
- **Every biome's winter can be dressed for.** For each biome, some set (plus fat and a hot meal)
  covers its coldest night while active, and some set its hottest day.
- **The ladder is in order.** Durability and speed rise tier by tier; bronze is near iron; steel
  is best.
- **A house works.** A closed log room with a fire keeps a Fairbanks winter night livable; a snow
  house with a lamp an Utqiaġvik one.
- **The sheet.** Materials, gear, clothing and wall classes, generated like the survival sheet.

## Open decisions

For Ronald:

1. **The ladder replaces Weft's tiers?** Wood, stone, iron, gold, diamond tools and armor become
   the real ladder (wood and bone, stone, copper, bronze, iron, steel). Proposed: yes.
2. **Gold and diamond.** Gold is too soft for tools (ornaments, trade, gilding). Diamond: drop it,
   keep it as a rare gem (trade, decoration), or keep a top tier above steel?
3. **Two layers of clothing.** Clothing under armor plus a cloak (nine slots), or one set of four
   slots where a fur coat and mail compete? Proposed: two layers and a cloak.
4. **How hard the biomes gate.** Tin only near beaches, in mountain streams and deep down: you
   travel or trade for bronze. The tundra, savanna and swamp have no loose stone: bone and trade
   instead. Proposed: yes, as real.
5. **Stations.** A few stations that grow with add-ons placed near them (Valheim: bellows on the
   kiln, an anvil by the forge), or a separate block for every craft? Proposed: a few with
   add-ons.
6. **Crafts that wait.** Tanning, retting, drying and charcoal take their real time on the growing
   pace (a buckskin 3 minutes, sole leather 4 game days), or are instant? Proposed: they wait.
7. **Shelter warmth by material.** A room check, and walls warming by their class (logs and earth
   warm, stone cold), or keep today's roof and fire? Proposed: the room check.
8. **Weft's redstone and its tech.** Keep as they are, outside this plan? Proposed: keep, untouched.
