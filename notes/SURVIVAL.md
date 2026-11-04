# Survival: vitals, food, temperature, levels

One system for players and animals: the same nine stats, the same stomach and nutrients, the same
body temperature and effects. Valheim's feel (food makes you strong, no starving to death, rest at
a warm home), with our own touch (a stomach and nutrients instead of three slots, real
temperatures, levels and points like the animals).

## The decisions (Ronald, 2026-11-02)

- **Food**: no hunger meter and no food slots; a **stomach** sized by the food stat, and every
  food has **nutrients** (protein, energy, fat, greens, water); a balanced diet gives a bonus;
  food **goes stale and spoils**; cooked and multi-ingredient meals are much stronger; **hot and
  cold meals** change body temperature.
- **Thirst**: water is a nutrient (drinks, soups, fruit), no meter, no dying of thirst.
- **Progression**: **levels and stat points like the animals**, the animals' nine stats, level
  cap 100, one point a level.
- **Temperature**: a real **body temperature**; gear insulation, fortitude and camp protect.
- **Stamina** pays for sprinting and jumping, fighting, working (mining, chopping), swimming and
  climbing.
- **Effects**: weather (wet, cold, freezing, hot, heatstroke), body (poisoned, bleeding, food
  poisoning, sedated), load (encumbered), camp (near a fire, sheltered, rested, well fed).
- **Rested and comfort** (Valheim): resting by a fire under a roof; furniture adds comfort.
- **XP** from fighting and hunting, gathering and making, taming and breeding, exploring.
- **Death**: a tombstone with everything, part of the XP toward the next level lost, no food in
  the stomach after.
- **Animals**: comfort ranges from real facts (seek shade, water, shelter; avoid biomes they
  can't stand), tames need care (barns, fires, coats), eggs need warmth, cold-blooded ones slow
  in the cold.
- **HUD**: health and stamina bars, XP with level, the stomach's foods with timers, the
  temperature bar, effect icons.
- **Difficulty** set per world: gentle, normal, harsh.

## How it fits together

**The stat sets the ceiling, food sets how much of it you reach.** A level-30 player with 12
points in health has a high maximum health; it's reached only well fed. With an empty stomach
max health and stamina fall to a floor (proposed 40% of the stat), never to death. This is the
animals' condition (decided: "well fed it reaches the strength its genes allow") applied to
players: one rule for both.

```
max health  = health stat  * (floor + (1 - floor) * protein fullness)   (proposed floor 0.4)
max stamina = stamina stat * (floor + (1 - floor) * energy fullness)
```

**The nine stats** (as `mods/animals/core/src/species.rs`), for players too, value =
`base * (1 + points * step)`:

| stat | player at level 1 (proposed) | a point (proposed) | what it does |
|---|---|---|---|
| health | 100 | +10% | ceiling of max health |
| stamina | 100 | +10% | ceiling of max stamina |
| food | 100 units | +10% | stomach size: how much you can have eaten at once |
| damage | 100% | +5% | melee and tool damage |
| speed | 100% | +2% | walking and sprinting |
| weight | 100 kg | +10% | carried before encumbered |
| oxygen | 100 | +20% | breath under water |
| torpor | 200 | +10% | how much it takes to knock you out (sedation, blows) |
| fortitude | 0 | +1 | widens the comfort range 0.5 °C each way, resists poison and sedation 2% |

Animals keep their own bases and steps per species; the formula and names are shared.

**Levels**: XP to the next level grows with the level (proposed `next = 12 + 8 * level^1.45`: 20
for the first, 2 300 at level 50, 6 300 from 99 to 100, about 260 000 in all); a point per
level; cap 100 (a world setting).

**The stomach.** Every food has a size (units, roughly grams / 10) and nutrients per unit; eating
puts it in the stomach if there's room; it digests over a time set by the food (a berry minutes,
a roast an hour, proposed), releasing its nutrients steadily while it's there. Fullness of each
nutrient: what's being digested against what the stats ask for.

| nutrient | from (examples) | does |
|---|---|---|
| protein | meat, fish, eggs, beans | max health, health regeneration, damage +5% when full |
| energy | bread, grain, roots, honey, fruit | max stamina, stamina regeneration |
| fat | meat fat, nuts, butter, fish | warmth: cold insulation, slower cooling |
| greens | vegetables, herbs, fruit, mushrooms | regeneration, poison and sickness resistance |
| water | water, milk, soup, fruit | stamina regeneration; low water: overheating sooner |

**Well fed**: four or five nutrients over half full, from at least three different foods: +10%
regeneration and stamina (proposed). Living on one food never makes it, however balanced (a loaf
nearly is).

Nutrients come from real nutrition facts per 100 g (protein, carbohydrate, fat, fiber and
vitamins, water), like the animals' facts: a table per food item, checked like `balance.rs`.

**Freshness**: a food stack knows when it was made; fresh, then stale (half the nutrients),
then spoiled (eating it: chance of food poisoning). Cooking resets and multiplies (proposed
x1.5), salted, smoked and dried foods keep much longer; meals from several ingredients add theirs
and keep the best freshness.

**Body temperature.** The air temperature where you stand, in °C:

- the biome's temperature (the engine's climate) mapped to °C;
- height: -0.65 °C per 100 m above sea level;
- the day: night colder (more in deserts), noon warmer in the sun;
- weather: rain -3 °C and wet; snow colder;
- water: swimming cools fast and makes you wet;
- shade and shelter: no sun, no rain, no wind;
- a fire: warm within a few blocks (more close by).

The body moves toward what it feels (proposed: half way in a minute); what it feels is the air
shifted by insulation (each armor piece and clothing has cold and heat insulation: fur warm,
leather mild, metal cold in snow and hot in sun), fat in the stomach, a hot or cold meal (for a
while), and fortitude. Comfort range for a person, unclothed: 15 to 28 °C air (proposed).

| feels | effect (proposed) |
|---|---|
| a bit below comfort | **Chilly**: stamina regeneration -25% |
| well below | **Cold**: no health regeneration |
| far below, or cold and wet | **Freezing**: loses health |
| a bit above | **Warm**: water used faster |
| well above | **Hot**: stamina regeneration -25% |
| far above | **Heatstroke**: loses stamina, then health |

Animals use the same with their species' comfort range from facts (a capybara 20 to 35 °C, an
arctic animal -30 to 15), and the effects drive their behavior: seek shade, water or shelter;
tames lose food and health faster out of range; eggs incubate only within their range;
cold-blooded species slower below theirs.

**Effects**, with icons and timers (the kit's status icons): wet, chilly, cold, freezing, warm,
hot, heatstroke; poisoned, bleeding, food poisoning, sedated; encumbered; near a fire,
sheltered, rested, well fed. Sedated is torpor: the animals' darts and plants work on players
too, by dose and body mass.

**Rested and comfort**: under a roof near a fire, resting (sitting, a bed) makes you Rested
(stamina regeneration +50%, XP +25%, proposed); the comfort level around you (furniture, beds,
banners, the fire itself; Valheim's) sets how long it lasts (8 minutes + 1 a comfort level).

**XP**: killing (by the creature's level and danger), gathering and mining (by the block's
hardness), crafting and cooking (by what it took), building (a little a block), taming (by the
animal's level), raising young and hatching, first visits to biomes and dimensions, advancements.

**Death**: a tombstone where you died holds everything; you lose part of the XP toward your next
level (proposed a quarter at normal); you respawn with an empty stomach.

**Difficulty per world** (gentle, normal, harsh): how fast food digests and spoils, the
temperature effects' strength (gentle never kills by cold), the XP lost at death.

## Where it lives (proposed)

- `mods/base/vitals`: a pure Rust crate, like `mods/animals/core`: the stats and their values,
  levels and XP, the stomach and nutrients, freshness, body temperature, the effects and their
  rules, the difficulty presets. No engine calls, so it is tested on its own and shared.
- `mods/base`: the player: its vitals component (replacing `Hunger`), stamina costs, eating,
  temperature from the world, fires and shelter, insulation on armor, rested, tombstone, XP
  sources, the HUD and a stats screen to place points.
- `mods/animals`: its stats on the shared ones, condition from the stomach, comfort ranges as
  species facts, behavior by temperature, eggs by temperature.

## Building it (proposed order)

1. **The vitals core**: stats, levels, stomach and nutrients, body temperature, effects; the food
   table from nutrition facts; balance checks.
2. **The player's vitals**: replace hunger; stamina costs; eating into the stomach; max health and
   stamina from food; regeneration; the HUD bars, stomach and effect icons.
3. **Temperature in the world**: air temperature from biome, height, time, weather, water,
   shade; fires and shelter; insulation on armor; the temperature bar; wet, cold, hot effects.
4. **Levels**: XP from every source, level ups, a stats screen to place points, death's tombstone
   and XP loss.
5. **Camp**: rested and comfort.
6. **Freshness and cooking**: spoilage on stacks, cooking, meals from several ingredients, hot
   and cold meals.
7. **Body effects**: poison, bleeding, food poisoning, sedation on players, encumbered.
8. **Animals**: the shared stats, condition from the stomach, comfort ranges, seeking comfort,
   tames' care, eggs' warmth, cold-blooded.
9. **Difficulty presets** per world.

## Open questions

- Comfort items list and values (which furniture counts).
- Which food items exist and their recipes (bread, stews, roasts, salted and smoked meat...):
  stews and the preserved foods are built (step 6b); roasts and pies of several ingredients wait.
- Clothing: decided (2026-11-03, via the Planner): clothes of their own, without armor value, in
  four slots under armor (step 10). Easy to change: the numbers are `vitals::clothing`.
- A stats screen of its own, or a tab in the inventory.

## Built so far

**1. The vitals core** (2026-11-02): `mods/base/vitals`, a pure crate in the workspace.

- `stats`: the nine stats (the animals' order), a player's bases and what a point adds, growth by
  share or by a fixed amount (fortitude).
- `levels`: XP to the next level (`12 + 8 * level^1.45`, about 257 000 to level 100), the level
  from XP, XP after dying, and the XP each source gives.
- `food`: every food the game has, from USDA nutrition per 100 g and a real portion's weight
  (cooked meat lighter and richer than raw); the five nutrients per unit; digestion pace (a steak
  or a loaf about half an hour, an apple ten minutes, a whole chicken forty to fifty); freshness
  (fresh, stale half, spoiled a quarter) and food poisoning chances (raw meat 15%, spoiled half);
  hot cooked meals and cooling fruit.
- `stomach`: portions digesting and fading, fullness of each nutrient, well fed (four nutrients
  over half, from three different foods), and max health and stamina between the 40% floor and the
  stats. The smallest well-fed meal for a new player: an apple, a loaf and a cooked steak (59 of
  100 units).
- `temperature`: the air from the biome, humidity's day and night swing, height, sun, rain, snow,
  water and fires (20 °C beside one, nothing past 8 blocks); comfort from insulation (armor sets),
  fortitude, fat and meals, mostly lost when wet; the core moving toward its target (half way in a
  minute for 70 kg, slower for bigger bodies); feelings from medicine's lines (hypothermia under
  35 °C, heatstroke over 40); cold-blooded speed.
- `effects`: the seventeen effects in four groups, what each changes, their icons in the kit.
- `stamina`: costs (sprint 7 a second, jump 8, a swing by the weapon's weight, a block by its
  hardness) and regeneration from the stomach.
- `difficulty`: gentle, normal, harsh.
- `balance`: checks that all of the above holds together (tests ask for no problems).

**2. The player's vitals** (2026-11-02): base's hunger is gone (`Hunger` in `base_api` replaced by
`Vitals`, kept through death).

- `mods/base/src/survival/` (one file per part since 2026-11-03): each tick the stomach digests; max health and stamina follow it
  (the engine keeps the base game's 20 health points, a point 5 of the vitals', so creatures' damage
  stays in balance; max health is a `base:fed` modifier on it); stamina pays for sprinting,
  jumping, swimming, blows that land (a weapon or tool 1.5 kg, a fist 0.5) and every block broken
  (by its hardness), comes back a second after spending from energy and water; no sprinting under
  10 once run out; health comes back from protein and greens; breath under water as before. Eating
  puts the food in the stomach (`base_api::eat`, by the item's name in the food table) and is
  refused without room. Dying empties the stomach, costs a quarter of the XP into the level, and
  sets max health again after respawning. `/feed [units]` fills the stomach with an apple, a loaf
  and a steak in turn (0 empties it).
- The HUD (`hud.rs`): health and stamina bars from the kit with their numbers, the stomach's
  foods with minutes left, the five nutrients' fullness, an icon per effect on; armor and breath
  under them.
- Seen in the game: a new player 40/40 health and stamina; an apple, a loaf and a steak raise them
  to 79 and 83 and make one well fed; six seconds of sprinting took 28 stamina.
- The tests that read hunger (`testkit` gameplay: food, respawn, advancements by tags, air in
  creative; ui: the `/feed` reply) are rewritten for the vitals; not run here (CI runs them).

Not yet: effects other than well fed (steps 3, 5, 7), freshness of what's eaten (step 6: all food
is fresh for now), climbing's cost (no ladders' state yet), foods other mods add without a row in
the food table (refused for now).

**3. Temperature in the world** (2026-11-02):

- Every biome has a `base:climate` (`base_api::CLIMATE`): its year's mean °C, humidity and half
  its summer-to-winter range, from a real place of its kind (plains 12 °C like a prairie, taiga 0,
  snowy plains -10 like tundra, desert 25, savanna 26, swamp 19, caves keep 12 all year; the
  Cretaceous's hot: dunes 28, fern forest 23). Other mods set it on theirs.
- Once a second (`survival/temperature.rs`, `feel`): the air where one stands from the climate, the hour,
  height (a block stands for 6 m), sun, rain or snow (rain under 1 °C falls as snow), water, caves
  (no day or weather deep down), and heat from lava, lit furnaces and torches within 7 blocks;
  wet from rain on open sky or water, drying in two minutes (faster by a fire); armor's insulation
  per piece; the core temperature moving toward its target. Effects: chilly to freezing, warm to
  heatstroke, wet, near a fire, sheltered (a roof).
- Freezing and heatstroke take health (new death messages: froze to death, died of heatstroke);
  heatstroke takes stamina.
- The HUD: an icon for how the body feels, the kit's temperature bar filling from cold up to the
  core temperature, and the air's °C.
- Seen in the game: plains at noon 20 °C, comfortable; snowy plains at midnight -20 °C, freezing,
  dead in a few minutes unclothed; two blocks from lava there 0 °C, cold but no longer freezing;
  savanna at noon 36 °C, hot.

## Seasons (Ronald, 2026-11-02: "seasons as well, tied in with mating and breeding")

The temperature already takes a season (`temperature::season`: each biome's own swing, big in the
taiga and snow, small in the tropics, none in caves); a spring or autumn day for now. To decide:
a year's length, what changes with the seasons, and how breeding follows them.

Decided:

- **A year is 56 days**: spring, summer, autumn and winter of 14 days each (about 19 hours of
  play): slow and realistic, a winter a real stretch to plan for.
- **Breeding follows each species' real season**, from facts as the rest of their biology: most
  mammals mate in spring so their young come in summer, cats with the long days, some all year
  (dogs; capybaras in the tropics); the dinosaurs by the Cretaceous's wet season. Out of season
  wild ones don't come into heat; tames can be bred out of season with comfort and food.
- **Day length** follows the season: long summer days and short winter ones, more in the north's
  biomes.
- **Snow and leaves**: snow covers the ground in winter where it's cold enough, leaves turn in
  autumn and fall in winter, grass browns.
- **Crops and food**: crops grow in their seasons and not in winter (a greenhouse aside), berries
  and fruit in summer and autumn, so storing food matters (and freshness, step 6).
- **Animals' lives**: some migrate or hibernate through winter, coats thicken in winter (the coat
  genes' shift), young are born in spring.
- **The Cretaceous** has a wet and a dry season instead: warm all year, floods and green in the
  wet, drought and spreading dunes in the dry.

Proposed order: the calendar first (a world's day count, the season and day of the year, shown on
the clock; the temperature's season term from it; day length), then breeding seasons in the
species' facts, then snow, leaves and grass, then crops, then migration, hibernation and coats.

**The calendar** (2026-11-02):

- The engine: a dimension can have a year (`.seasons(days, day_swing)`): `day_turn` stretches the
  clock so the sun rises earlier and sets later in summer (and the sky and daylight with it).
- `vitals::calendar`: the date from the world's day time (a year of 56 days, spring first, a day
  starting at 06:00), how far the year is from midwinter, daylight hours, jumping to a day.
- Each dimension says its calendar (`base_api::SEASONS`): the overworld four seasons with
  sixteen hours of light at midsummer and eight at midwinter; the Cretaceous a wet and a dry season
  whose days barely change; the flatlands none.
- The air's temperature takes the season (each biome's own swing); the clock shows the day of the
  season ("Summer 7", "Wet season 3") and its sun from the real sunrise; `/season <season> [day]`
  jumps there, keeping the hour.
- Seen in the game, a taiga: summer 05:00 already light, 11 °C, noon 26 °C; winter 07:00 still
  dark with stars out, -22 °C, noon -10 °C.

Next for seasons: breeding seasons in the species' facts, snow, leaves and grass, crops, migration,
hibernation and coats; rain by the wet and dry seasons.

**4. Levels** (2026-11-02): `mods/base/src/levels.rs`.

- XP (`base_api::give_xp`, which other mods call for their own deeds; rested raises it, none in
  creative): mining and chopping by the block's hardness, building a little a block, crafting by
  ingredients, killing (a creature 1.5 danger, a player 3), advancements 100, the first visit to a
  biome 60 and to a dimension 600 (the first world and biome don't count).
- A level up shows a banner ("Level 7! A point to place: press [P]") with a chime; the stats
  screen (P, and a shortcut on the HUD) lists the nine stats with their values and points and a +
  to place one; the HUD's XP row shows the level, the bar to the next and the points waiting.
- Dying: a tombstone (`base:tombstone`, a mossy headstone holding 45 slots) is placed where one
  died with everything carried in it, before the engine would scatter it; a quarter of the XP into
  the level is lost. Broken, it drops what's left. `/xp [amount]` gives XP or tells it.
- Seen in the game: 400 XP is level 7 with 6 points; the stats screen; dying left a tombstone at
  the spot and nothing on the ground.
- Not yet: killing XP by the creature's real strength (its kind's health isn't known when it dies);
  the tombstone crumbling once emptied (machines have no change hook yet).

**XP from animals** (2026-11-02, `mods/animals/src/xp.rs`): taming one gives 50 and 4 a level
(a level-114 cat: 506, level 1 to 7); a young one born or hatched to its owner gives them 40 if
they're here; killing one gives `xp::kill` by its level and its kind's size (a quarter power of its
mass against a 4 kg cat: a chihuahua under 1, a capybara about 2, a Utahraptor over 3), on top of
base's small kill reward. The level and kind are noted when a player's blow lands, since the
animal is gone by the time its death is told. Seen in the game: a wild capybara killed with a
sword, about 100 XP.

**5. Rested and comfort** (2026-11-02, `survival/camp.rs`):

- Comfort: 1, and one more for each kind of furniture within 7 blocks, each kind once (Valheim's):
  a fire (lava, a lit furnace, a torch), a bed, a table (crafting table), storage (chest),
  something soft (wool), flowers, a lamp. More kinds come with more furniture.
- Resting: under a roof by a fire (near a fire and sheltered) for 20 seconds, or asleep in a bed;
  then rested for eight minutes and one more for each comfort level, kept full while one rests on.
  Rested: stamina back half again as fast, XP a quarter more.
- The HUD: "Resting by the fire: 12s" while it builds; the rested icon with its minutes left and
  the comfort level.
- Seen in the game: a plank hut with a torch, bed, crafting table, chest and wool: comfort 6,
  rested after 20 seconds for 14 minutes, stamina full.

**6. Freshness and hot meals** (2026-11-02):

- Food stacks carry stamps (`base_api::MADE`, `HOT`, `FRESHNESS`, stack components): the day it
  was made, stamped the first time a player holds it; cooked food also the second it came off the
  fire, hot for five minutes, then it cools and stacks with the day's. Each second the stamps of
  what a player carries move on: fresh, stale (half the nutrients), spoiled (a quarter); stacks of
  different days don't stack.
- Eating (`base_api::eat`) takes freshness into the stomach, hot food warms (4 °C for four
  minutes), and raw, stale or spoiled food may give food poisoning (three minutes: no healing,
  stamina back half as fast, slower), less with fortitude.
- Tooltips: what a portion holds in grams (`base:nutrition`, from the food table: "200 g: P 42 ·
  E 0 · F 16 · G 0 · W 140") and how fresh it is; the base game's drumsticks are gone from them.
- Seen in the game: raw beef fresh, then stale after three days and spoiled after six; a steak
  hot from the fire.
- Not yet: meals from several ingredients (stews, pies) wait for crops and their items and
  icons; salting, smoking and drying (keeping longer); food in chests ages only from when a player
  first held it. (All three since step 6b.)

**6b. Stews, drying, smoking, salting** (2026-11-03, `vitals::meals`, `vitals::preserve`,
`mods/base/src/cooking.rs`):

- The cooking pot (iron, its own fuel): two to four foods and a bowl make a stew in 10 s. A stew
  is what went into it (`base:meal` on the stack, raw meat counted cooked): eating it puts each
  into the stomach as its own portion at x1.25, so one bowl of three foods makes one well fed;
  hot, it warms as one dish; it keeps as its shortest-keeping ingredient; the bowl comes back.
  Its tooltip shows what it holds (`base:meal_nutrition`). The stomach and its save are unchanged.
- Preserved foods, one per method, each its own row of real nutrition (USDA), the water gone so
  lighter to carry: dried meat (30 + 30 days; jerky), smoked meat (14 + 14), salted meat (45 +
  45), dried apple and dried berries (56 + 56, a game year).
- The drying rack (sticks, four slots) dries by its air (`air_at`): about a game day at 25 °C,
  humidity 0.3 and a breeze, nothing under 5 °C, above humidity 0.85 or in the rain; with a fire
  within two blocks below it smokes meat in half a day, in any weather.
- Salt: a water bucket boiled in a furnace (20 s), the bucket back (the furnace's input now
  leaves its remainder, as crafting does); raw meat and salt craft into salted meat.
- Food is stamped as the pot and the rack make it, and opening any block's inventory (a chest)
  brings its food's freshness up to date; food a player held was stamped then and ages wherever
  it is, since its day is fixed.
- Seen in the game (11-04, Tester): a pot with coal made a stew of beef, an apple and bread in
  ten seconds, using one coal; after half a day a rack over a lit furnace held smoked meat and a
  dried apple, and one in the winter air still held its mutton. Not seen yet: eating a stew, its
  tooltip.
- Not yet: roasts and pies from several ingredients (the apple pie is still a fixed row), smoked
  fish (no fish item), food going stale on a rack left in the rain.

**7. Poison, bleeding, sedation, carried weight** (2026-11-02, `survival/body.rs`, `survival/load.rs`, `base_api`):

- `base_api::poison(w, player, secs)`, `bleed(..)` and `sedate(w, player, torpor)`, for any mod
  (fortitude shortens poison and softens sedation, 2% a point; nothing in creative).
- Poisoned: no healing, 0.4% of max health a second (a spider's eye: 15 s; a venom breath on a
  player: 10 s, instead of the animals' own poison). Bleeding: no healing, 0.6% a second for
  20 s, from any blow (`damage::ATTACK`) of a fifth of max health or more. Deaths are put down to
  the worst effect: new causes `damage::POISON` and `BLEED` ("was poisoned", "bled out"), with
  freezing and heatstroke.
- Torpor: tranquilizer darts on a player add it (`knockout::add_torpor`); it wears off 3% of the
  torpor stat a second. Over a quarter: sedated (half speed, no sprinting); at the stat (200 for
  a new player, so four darts): out cold, no walking or jumping, until under half (about half a
  minute).
- Carried weight (`survival/load.rs`, `weight`): game weights, not real ones (a block of stone 0.1 kg, an
  ingot 0.2, a log 0.15, a tool 1, armor by material and piece, food its portion's grams). Over the
  weight stat (100 kg): encumbered (75% speed, actions 1.5x stamina); past half again:
  overloaded (half speed, double stamina, no sprinting). The stats screen says "Carrying 22.4 of
  100 kg".
- Speed: the speed stat times the effects' speeds, set as a `base:vitals` modifier on the
  engine's movement speed (and now and then again, as a respawn drops it).
- Bandage (`base:bandage`, 3 from a wool block): stops bleeding and heals 10; used only when
  bleeding or hurt. Its icon is the string's until one is generated (`notes/STYLE.md`).
- Seen in the game: a spider's eye poisons (the skull icon, health falling); 110 kg of stone
  encumbered, 220 overloaded, walking 8.6, 6.5 and about 4 blocks in two seconds; a heavy blow
  bleeds (the drop icon), a bandage stops it and gave health back.
- Not seen in the game: being knocked out by darts (needs aiming one at a player), a death from
  poison or bleeding (the messages are in).

**8. Animals on the shared system** (2026-11-02, `mods/animals/src/comfort.rs`):

- The air is one function for players, animals and eggs: `base_api::air_at` (climate, season,
  hour, height, sun and shade, rain or snow, water, caves, fires near by).
- Every species has the air it's comfortable in as a fact (`Biology::comfort`, on the sheet):
  chihuahua 7 to 29 °C (vets: small short-coated dogs feel the cold under 7, heat from 29), cat 5
  to 32, capybara 12 to 32 (zoos keep them over 12, they lie in water in the midday heat),
  Velociraptors 12 to 40, Utahraptor 6 to 36, wyvern -10 to 30 (the last three by analogy with
  birds of their size and their places). Fortitude points above the species' base widen it, a
  full belly warms, as a player's.
- Each animal has a core temperature once a second (`BODY`, the player's rules and lines; a
  small body cools faster) and a feeling, shown on its screen with the air and its comfort.
- Cold or hot (not just chilly or warm), wild or tame, it goes to a better place near by: random
  spots within 10 blocks, and in the cold beside the nearest fire within 16 (lava, a lit furnace,
  torches); it rests there, and looks again later. Following or staying tames stay.
- A tame out of comfort eats faster (1.5x cold or hot, 2x freezing or overheating), and freezing
  or overheating loses health as a player does, down to a tenth (it suffers, it doesn't die of
  it). Wild ones take no harm: their places suit them.
- Condition from the stomach, the player's rule: full strength from half fed, down to 40% empty
  (max health, speed, damage). Wild ones are fed.
- Cold-blooded (`Biology::cold_blooded`): no body heat, slower in the cold (40% fifteen degrees
  under its comfort's middle). No species is so far: dromaeosaurs are thought warm-blooded like
  birds, and the wyvern breathes fire.
- Eggs develop while the air at them is 25 to 40 °C (birds' and reptiles' eggs: about 26 to 38),
  a nest holding them 60% of the way to a brooding parent's 37 °C (twice as fast in it); fires
  near by warm them. Before, a torch within two blocks made any egg warm, and only snowy biomes
  were too cold.
- Seen in the game: a wild chihuahua on a winter night (-8 °C) grew cold and walked to lava 12
  blocks off; a tame one at -9 °C cold, eating faster; in snow (-14 °C) freezing, losing health.
- Not seen in the game: the heat side (a desert noon), condition on a hungry tame, eggs by the
  air (nothing shows an egg's warmth yet).
- Not yet: coats or barns for tames (shelter and fires work already), stamina for animals, poison
  and bleeding on animals through the shared effects (venom keeps the animals' own).

**9. Difficulty per world** (2026-11-02):

- `base:difficulty`, a game rule (`base_api::DIFFICULTY`, read with `base_api::difficulty()`):
  gentle, normal (the default), harsh, with `vitals::difficulty`'s numbers: how fast food digests
  and how long it keeps, how hard cold, heat, poison and bleeding hurt (gentle never below a
  tenth of max health), the XP lost on dying (a tenth, a quarter, half), what actions cost in
  stamina. Tames follow it too: how fast they eat, how hard the cold and heat hurt them.
- Picked on Create New World (the engine now offers every game rule that is a toggle or a
  choice and has a name in the language files: "Survival: Normal", "Mobs spawn: On"), changed
  later with `/rule difficulty <0-2>` or in Mod Settings, saved with the world.
- Seen in the game: `/rule difficulty 2` is harsh and still harsh after reopening the world; a
  world made with "Survival: Gentle" is gentle.
- Not seen in the game: each number's effect at gentle and harsh (they use the same code as
  normal, multiplied).

**10. Clothing** (2026-11-03, `vitals::clothing`, `mods/base/src/clothing.rs`):

- Four slots of their own under armor, beside it in the inventory: hat, coat, trousers, boots.
  No armor value; armor keeps its insulation. Fur's row left armor's table for clothing's.
- Four sets of the content plan's research (`reports/content/Clothing and armor.md`, section E,
  branch `claude/content-plan-d6bc49`), at 12 °C of comfort a clo walking (ISO 11079), a piece
  its share of the set (head 0.15, body 0.4, legs 0.3, feet 0.15): fur, a single caribou suit,
  2.5 clo (30 °C, -9 in the heat), wool 1.2 (14, -2), hide (buckskin) 0.8 (10, -1), linen 0.5
  (6, +4: with its hat cooler than bare skin in the sun). The research's other sets (double
  caribou for the Arctic winter, sheepskin, loden, a desert robe, sealskin) and a cloak slot are
  rows and a slot to add.
- Wet keeps a share by material (wool 60%, fur and hide 30%, linen 20%, armor 35%), on top of
  being wet feeling 6 °C colder. Windproof clothes stop a share of the wind chill (fur 80%, hide
  70%, wool 30%, linen 10%): `air_at` gives the air with and without the wind. Leather armor's
  heat had the wrong sign (+3, now -3).
- Materials: fur from wild furred animals (hares and foxes one, wolves and coyotes up to two,
  reindeer and jaguars up to three: `animals_core::hides`), leather from cows and deer, sheep's
  wool, and linen from string, which plant fibre from cutting tall grass (1 in 8) spins into.
  Each piece is sewn with string.
- Their tooltips say their warmth (`base:warmth`). Item icons by the game-art pass (11-04, Ronald's
  yes), also the stews', preserved foods', fibre's, salt's, bowl's, pot's and rack's; the worn
  layers and the pot's, rack's and campfire's block faces are still tinted placeholders.
- Checks (`vitals::balance`): fur holds off the cold on a -5 °C night that freezes one unclothed,
  linen is cooler than bare skin in the desert sun and fur hotter, wet wool warmer than wet linen,
  fur keeps out the wind.
- Seen in the game (11-04, Tester): fur in the four clothing slots and an iron chestplate came to
  31.6 °C cold, 9.6 kept wet, -10.6 heat, 80% windproof, and the air felt -8.2 °C instead of
  -11.8 unclothed; the inventory shows armor and clothes in two columns beside the model, the fur
  coat under the chestplate. Not seen yet: pelts dropping, plant fibre from grass.
- Not yet: clothes wearing out, dyes, flax for linen (the plants work), generated art.
- From the Researcher's round 1 (F1, `reports/Research round 1.md` on `claude/researcher-c31668`):
  used per-item warmth, windproof and warm-when-wet (`MATERIALS`, the body's shares). Not yet:
  waterproof (clothes that keep one dry, the research's sealskin and gut parka), and wetness in
  steps (DayZ's damp, wet, soaked, drenched cutting insulation by 10, 50, 90%) instead of wet or
  dry. Preserving is foods of their own with real nutrition and keeps rather than TerraFirmaCraft's
  decay multipliers (salted 0.5 and so on): our salted meat keeps over twenty times raw meat's.


**11. A fair first night** (2026-11-03, Ronald via Planner, after Tester's playtest):

- A new player arrives with a loaf and an apple in them (`vitals::stomach::ARRIVAL`, 43 units):
  max health 54 and stamina 75 of 100 instead of an empty stomach's 40, wearing off over half an
  hour as it digests. One spider at dusk had killed a fresh player in 11 s. Dying leaves the same
  light meal (step 13). The rule stays: food sets how much of the stat is reached.
- A new player also starts with full stamina (it started at 0 and took 14 s to fill).

**12. Research round 3's survival fixes** (2026-11-04, decided by Ronald via Planner; `reports/Research round 3.md`):

- **B1, a first week one can live through.** A new player arrives in plain clothes once
  (`vitals::clothing::ARRIVAL`: a linen hood, a wool sweater, buckskin trousers, moccasins, 11 °C,
  about 0.9 clo), put in the empty clothing slots on first joining (`base:dressed`). A **campfire**
  (three logs, four sticks, no stone) heats like lava (+20 °C beside it), counts as a fire for rest,
  and smokes food on a rack above it; it burns without fuel and gives light. A new world with
  seasons begins on spring's 8th day (`vitals::calendar::START_DAY`, a quarter of the year from
  midwinter): its clock moves there in its first second, so the engine's day length agrees.
- **D1, spoiled food never comes back.** The pot, the rack and the crafting grid refuse spoiled
  food. What is made of stale food starts as far through its own life as its stalest part was
  (`survival::made_for`): a half-gone beef makes a half-gone stew. Carrying the oldest age as it
  stands would spoil a fresh stew at once for an old apple, so the share of life is carried.
- **B3, clothes within reach.** Hide and wool pieces take 1, 2, 1, 1 of their material (fur and
  linen 2, 4, 3, 2), each sewn with 1, 2, 1, 1 string, and a string is 2 plant fibre. Each piece
  has its own shape in the grid, since a hat and boots of the same stuff took the same things.
- **C1.** Leather armor is hide: the same warmth, wet share and windproofing as a hide coat
  (`clothing::armor_cloth`). Armor and clothes stack.
- **B5.** Plant frost (`growth.rs`) and drying read the still air: wind chill is what a body loses.
  The animals session does eggs.
- **B6.** Meals warm by the warmest dish, not the sum (four hot steaks are +4 °C, not +16), and
  cooling food only widens the hot side of comfort, it doesn't chill one in the cold.
- The rack dries every raw meat and fish and every fruit that doesn't keep half as long as dried
  fruit already (rose hips don't need it); `preserve::PRODUCTS`.
- **U1.** Ice drops itself when mined with a pickaxe (the only blue dye).
- **T1.** Base `ru_ru` has every key `en_us` has (106 added).
- Checks (`vitals::balance`): arrival clothes 8 to 14 °C and not freezing on a plains night at the
  world's start; four hot steaks warm as one; leather armor and hide clothes alike.
- Not done from round 3: B7 (the warm effects' water use), B8 (salt from sea water only), B12 (a
  cheaper pot), the stations in the recipe viewer, "from" lines on fur, fibre and salt, the
  compost bin's drawn textures, first-week advancements.

**13. The first hour and the first death** (2026-11-04, Research round 4, H1 and H2, via Planner):

- **H1, no death spiral.** Respawning brings back the plain arrival clothes when none are worn
  (the tombstone keeps what one wore), the stomach holds the arrival meal instead of nothing, and
  stamina comes back full. After respawning a hint says where the tombstone is (x and z, kept in
  `base_api::GRAVE`). The death screen's cause is the engine's (`core:death`, not done here).
- **The campfire** is made in the inventory's 2x2 grid (two sticks over two logs) and cooks:
  a slot in, a slot out, whatever a furnace cooks of food at 1.5 times its time, no fuel; what it
  cooks comes off hot, as far along as the raw food was. It still burns without fuel.
- **Hints** (`hints.rs`): eleven lines under the crosshair for six seconds, from the player's
  state once a second, each once per player (saved, kept through death), freezing each time it
  starts, the tombstone each death; the player's own setting turns them off (`base:hints`, Mod
  Settings). Logs, a crafting table, night coming with no fire, cold, freezing, wet in cool air,
  an empty stomach, raw meat with no furnace yet, the tombstone, too hot, resting by a fire
  under a roof. A hint isn't shown for what one has already done (logs held, a fire placed, a
  furnace had). Not here: the recipe keys' hint and the taming hint (engine and animals).
- **A "Surviving" advancement group**: Hearth (place a campfire), Spun (string), Tailor (craft a
  piece of clothing), A Night In (rested), Hot Meal (eat food still hot), One Pot (a stew), Put
  By (a preserved food), A Friend (tame: the animals mod calls `base_api::milestone(w, player,
  base_api::milestones::TAME)`).
- **"Comes from" tooltips** on fur, plant fibre, salt, leather, string and wool.
- Not done here: R and U with nothing hovered (the engine's `app.rs` `recipe_keys` only looks
  under the cursor in a screen), the death screen's cause, the HUD labels (the UI session).

