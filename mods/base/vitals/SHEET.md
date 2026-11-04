# Survival numbers

Written from `mods/base/vitals` (`cargo run -p vitals --bin sheet > mods/base/vitals/SHEET.md`); do not edit by hand. How the systems connect: `notes/SYSTEMS.md`; why: `notes/SURVIVAL.md`.

## A player's stats

| stat | level 1 | a point |
|---|---|---|
| health | 100 | +10% |
| stamina | 100 | +10% |
| food | 100 | +10% |
| damage | 1 | +5% |
| speed | 1 | +2% |
| weight | 100 | +10% |
| oxygen | 100 | +20% |
| torpor | 200 | +10% |
| fortitude | 0 | +1 |

Comfortable air for a person, unclothed: (15.0, 28.0) °C.

## Foods

A portion, its kind for the animals' diets, what it holds of each nutrient, how long it digests, and game days fresh then stale.

| food | kind | g | units | protein | energy | fat | greens | water | digests | keeps | |
|---|---|---|---|---|---|---|---|---|---|---|---|
| base:apple | fruit | 180 | 18 | 1 | 25 | 0 | 5 | 154 | 9 min | 20 + 20 | cooling |
| base:bread | seeds | 250 | 25 | 22 | 122 | 8 | 7 | 90 | 26 min | 5 + 5 | hot when cooked |
| base:porkchop | meat | 150 | 15 | 31 | 0 | 9 | 0 | 108 | 22 min | 2 + 2 | raw |
| base:cooked_porkchop | meat | 110 | 11 | 30 | 0 | 10 | 0 | 67 | 19 min | 4 + 4 | hot when cooked |
| base:beef | meat | 200 | 20 | 42 | 0 | 16 | 0 | 140 | 31 min | 2 + 2 | raw |
| base:cooked_beef | meat | 150 | 15 | 42 | 0 | 16 | 0 | 88 | 28 min | 4 + 4 | hot when cooked |
| base:chicken | meat | 300 | 30 | 56 | 0 | 45 | 0 | 206 | 53 min | 2 + 2 | raw |
| base:cooked_chicken | meat | 220 | 22 | 60 | 0 | 30 | 0 | 131 | 43 min | 4 + 4 | hot when cooked |
| base:mutton | meat | 150 | 15 | 26 | 0 | 24 | 0 | 96 | 26 min | 2 + 2 | raw |
| base:cooked_mutton | meat | 110 | 11 | 28 | 0 | 18 | 0 | 63 | 22 min | 4 + 4 | hot when cooked |
| base:rotten_flesh | carrion | 120 | 12 | 18 | 0 | 12 | 0 | 78 | 18 min | 0 + 0 | raw |
| base:spider_eye | insects | 30 | 3 | 4 | 1 | 1 | 0 | 22 | 4 min | 3 + 3 | raw, poisonous |
| base:cooked_egg | meat | 100 | 10 | 14 | 1 | 15 | 0 | 70 | 16 min | 2 + 2 | hot when cooked |
| base:apple_pie | fruit | 200 | 20 | 5 | 74 | 25 | 4 | 104 | 25 min | 4 + 4 | hot when cooked |
| base:berries | fruit | 100 | 10 | 1 | 14 | 0 | 3 | 84 | 5 min | 6 + 6 | cooling |
| animals:kibble | meat | 100 | 10 | 25 | 45 | 14 | 4 | 10 | 19 min | 90 + 90 |  |

## Effects

| effect | group | changes |
|---|---|---|
| wet | Weather | health regen x0.75, stamina regen x0.75 |
| chilly | Weather | stamina regen x0.75 |
| cold | Weather | health regen x0, stamina regen x0.5 |
| freezing | Weather | health regen x0, stamina regen x0.5, speed x0.85, loses 0.6% of max health a second |
| warm | Weather | water used x1.5 |
| hot | Weather | stamina regen x0.75, water used x2 |
| heatstroke | Weather | stamina regen x0, water used x3, loses 0.3% of max health a second, loses 3% of max stamina a second |
| poisoned | Body | health regen x0, loses 0.4% of max health a second |
| bleeding | Body | health regen x0, loses 0.6% of max health a second |
| food_poisoning | Body | health regen x0, stamina regen x0.5, speed x0.9 |
| sedated | Body | stamina regen x0.5, speed x0.5, no sprinting |
| encumbered | Load | speed x0.75, stamina costs x1.5 |
| overloaded | Load | speed x0.5, stamina costs x2, no sprinting |
| near_fire | Camp | health regen x1.25 |
| sheltered | Camp |  |
| rested | Camp | stamina regen x1.5, XP x1.25 |
| well_fed | Camp | health regen x1.1, stamina regen x1.1 |

Rested lasts 480 s and a minute more a comfort level. The core temperature: Freezing, then Cold from 33 °C, Chilly from 35 °C, Comfortable from 36.5 °C, Warm from 37.6 °C, Hot from 38.6 °C, Heatstroke from 40.1 °C.

## Climates

Each biome's real place (`reports/ecology/Biome climates.md`), year mean, humidity and half its summer-to-winter range, the air at sea level under an open sky (midwinter before dawn, midsummer mid afternoon), and the share of days with rain in spring, summer, autumn and winter.

| biome | place | mean °C | humidity | seasons ± | coldest | hottest | rain: spring | summer | autumn | winter |
|---|---|---|---|---|---|---|---|---|---|---|
| base:plains | Manhattan, Kansas (Konza Prairie) | 12.8 | 0.42 | 14.2 | -7 | 36 | 27% | 27% | 20% | 14% |
| base:forest | Kassel, Germany | 9.3 | 0.77 | 8.7 | -3 | 25 | 34% | 34% | 37% | 41% |
| base:birch_forest | Moscow | 6.3 | 0.8 | 12.9 | -10 | 26 | 29% | 30% | 32% | 47% |
| base:taiga | Fairbanks, Alaska | -2.1 | 0.51 | 19.8 | -27 | 26 | 14% | 32% | 26% | 22% |
| base:snowy_plains | Utqiaġvik, Alaska | -10.2 | 0.92 | 14.9 | -28 | 11 | 14% | 24% | 32% | 15% |
| base:desert | Hermosillo, Sonora | 25.7 | 0.18 | 8.1 | 10 | 45 | 1% | 13% | 6% | 5% |
| base:savanna | Arauca, the Colombian llanos | 27 | 0.65 | 1.6 | 21 | 36 | 40% | 70% | 45% | 11% |
| base:swamp | Nhumirim, the Pantanal | 25.1 | 0.42 | 3.4 | 16 | 38 | 18% | 33% | 20% | 7% |
| base:mountains | Davos, 1,594 m | 6 | 0.63 | 8.8 | -8 | 23 | 32% | 45% | 31% | 29% |
| base:ocean | Ponta Delgada, the Azores | 18.2 | 0.97 | 4.1 | 11 | 28 | 33% | 19% | 38% | 43% |
| base:beach | Faro, Portugal | 18.2 | 0.74 | 6.2 | 8 | 32 | 16% | 1% | 16% | 20% |
| base:deep_caves | any cave: the surface's year mean | 12 | 0.95 | 0 | 12 | 12 | 0% | 0% | 0% | 0% |
| animals:red_dunes | the Djadokhta Formation, Mongolia | 17 | 0.2 | 12 | -2 | 40 | 10% | 2% | 3% | 12% |
| animals:scrubland | the Barun Goyot Formation, Mongolia | 17 | 0.3 | 11 | -1 | 38 | 6% | 25% | 8% | 4% |
| animals:floodplain | the Cedar Mountain Formation, Utah | 21 | 0.5 | 8 | 7 | 38 | 8% | 45% | 25% | 4% |
| animals:fern_forest | the Yixian Formation, Liaoning | 10 | 0.8 | 10 | -4 | 27 | 25% | 45% | 30% | 15% |
| animals:crags | the Sevier highlands, Utah | 15 | 0.6 | 8 | 2 | 31 | 25% | 40% | 25% | 15% |

## Plants

What each growing plant needs (`vitals::plants`, from `reports/ecology/Plants and crops.md`): the air it grows in (from, best, to), its water, its real days to harvest and the game's minutes at the best rate, and where and in which seasons it grows at least a third of its best (sp, su, au, wi), with each biome's rain and warmth.

| plant | real plant | °C | water | real days | minutes | grows |
|---|---|---|---|---|---|---|
| base:wheat | spring wheat | 0, 15-23, 34 | Moderate | 110 | 26 | plains (su au), forest (su au), birch_forest (su au), taiga (su), savanna (su au), swamp (au), mountains (su au), ocean (sp su au wi), beach (au wi), floodplain (au), fern_forest (su au), crags (su au) |
| base:berry_bush | blueberry | 7, 20-26, 32 | Moderate | 75 | 20 | plains (su), forest (su), birch_forest (su), taiga (su), savanna (sp su au), swamp (su au), mountains (su), ocean (su au wi), beach (au), scrubland (su), floodplain (su au), fern_forest (su), crags (su au) |
| base:sugar_cane | sugar cane | 12, 22-30, 38 | Wet | - | the game's | plains (su), forest (su), birch_forest (su), taiga (su), desert (sp su au wi), savanna (sp su au wi), swamp (sp su au wi), ocean (sp su au), beach (sp su au), red_dunes (sp su au), scrubland (sp su au), floodplain (sp su au), fern_forest (su), crags (sp su au) |
| base:cactus | saguaro | 10, 25-40, 50 | Dry | - | the game's | plains (su), forest (su), birch_forest (su), taiga (su), desert (sp su au), savanna (sp su au wi), swamp (sp su au wi), ocean (sp su au), beach (sp su au), red_dunes (su), scrubland (su), floodplain (sp su au), fern_forest (su), crags (su) |
| base:sapling | oak | 5, 18-28, 38 | Moderate | - | the game's | plains (su au), forest (su), birch_forest (su), taiga (su), savanna (sp su au), swamp (su au), mountains (su), ocean (sp su au wi), beach (au), scrubland (su), floodplain (su au), fern_forest (su au), crags (su au) |
| base:grazed_grass | prairie grass | 5, 15-30, 40 | Moderate | 18 | 7 | plains (su au), forest (su au), birch_forest (su), taiga (su), savanna (sp su au), swamp (su au), mountains (su), ocean (sp su au wi), beach (au wi), scrubland (su), floodplain (su au), fern_forest (su au), crags (su au) |

## Food in the world

What foraging animals eat of the world's blocks (`vitals::forage`), how much a mouthful feeds (a capybara holds 200), what eating it leaves, and what brings it back.

| block | kind | food | eaten | grows back as |
|---|---|---|---|---|
| base:tall_grass | grass | 10 | becomes base:grazed_grass | base:grazed_grass |
| base:berry_bush (age 3) | fruit | 8 | back to age 1 | base:berry_bush |
| base:oak_leaves | leaves | 4 | stays | always there in base:forest, base:plains, base:swamp (spring, summer, autumn), worth 40% of fresh growth |
| base:birch_leaves | leaves | 4 | stays | always there in base:forest, base:birch_forest (spring, summer, autumn), worth 40% of fresh growth |
| base:spruce_leaves | leaves | 4 | stays | always there in base:taiga, base:snowy_plains, base:mountains (all year), worth 40% of fresh growth |
| base:acacia_leaves | leaves | 4 | stays | always there in base:savanna (all year), worth 40% of fresh growth |
| base:cactus | leaves | 3 | stays | always there in base:desert (all year), worth 40% of fresh growth |
| base:tall_grass | seeds | 3 | stays | always there in base:plains, base:forest, base:birch_forest, base:taiga, base:savanna, base:swamp, base:mountains, base:beach (summer, autumn), worth 40% of fresh growth |
| base:dead_bush | seeds | 2 | stays | always there in base:desert (all year), worth 40% of fresh growth |
| base:cactus | fruit | 3 | stays | always there in base:desert (summer, autumn), worth 40% of fresh growth |
| animals:fern | leaves | 4 | stays | always there in animals:red_dunes, animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (all year), worth 40% of fresh growth |
| base:tall_grass | grass | 3 | stays | always there in base:plains, base:forest, base:birch_forest, base:savanna, base:swamp, base:mountains, animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (winter), worth 40% of fresh growth |
| base:oak_leaves | leaves | 2 | stays | always there in base:forest, base:plains, base:swamp (winter), worth 40% of fresh growth |
| base:birch_leaves | leaves | 2 | stays | always there in base:forest, base:birch_forest (winter), worth 40% of fresh growth |
| base:oak_leaves | seeds | 3 | stays | always there in base:forest, base:plains, base:swamp (autumn, winter), worth 40% of fresh growth |
| base:snowy_grass | lichen | 3 | stays | always there in base:snowy_plains (all year), worth 40% of fresh growth |
| base:podzol | lichen | 3 | stays | always there in base:taiga (all year), worth 40% of fresh growth |
| base:stone | lichen | 2 | stays | always there in base:mountains (all year), worth 40% of fresh growth |
| base:podzol | fungi | 2 | stays | always there in base:taiga (summer, autumn), worth 40% of fresh growth |
| base:oak_log | fungi | 2 | stays | always there in base:forest, base:plains, base:swamp (summer, autumn), worth 40% of fresh growth |
| base:birch_log | fungi | 2 | stays | always there in base:forest, base:birch_forest (summer, autumn), worth 40% of fresh growth |
| base:grass | roots | 2 | stays | always there in base:plains, base:forest, base:birch_forest, base:savanna, base:swamp, base:mountains, animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (all year), worth 40% of fresh growth |
| base:sand | roots | 1 | stays | always there in base:desert, base:beach (all year), worth 40% of fresh growth |
| animals:red_sand | roots | 1 | stays | always there in animals:red_dunes (all year), worth 40% of fresh growth |
| base:grass | insects | 2 | stays | always there in base:plains, base:forest, base:birch_forest, base:savanna, base:swamp, base:mountains, animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (spring, summer, autumn), worth 40% of fresh growth |
| base:sand | insects | 2 | stays | always there in base:desert, base:beach (all year), worth 40% of fresh growth |
| animals:red_sand | insects | 2 | stays | always there in animals:red_dunes (all year), worth 40% of fresh growth |
| base:podzol | insects | 2 | stays | always there in base:taiga (summer), worth 40% of fresh growth |
| base:snowy_grass | insects | 2 | stays | always there in base:snowy_plains (summer), worth 40% of fresh growth |
| base:sugar_cane | water_plants | 4 | stays | always there in base:plains, base:forest, base:birch_forest, base:savanna, base:swamp, base:beach (all year), worth 40% of fresh growth |
| animals:horsetail | water_plants | 4 | stays | always there in animals:red_dunes, animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (all year), worth 40% of fresh growth |

## Soil and compost

Farmland's fertility (`vitals::soil`): how fast crops grow at each level, what wears it, what gives it back; and what a compost bin takes.

| fertility | crops grow |
|---|---|
| 3 | 100% |
| 2 | 73% |
| 1 | 47% |
| 0 | 20% |

A ripe harvest of base:wheat takes a level; base:compost +2, base:dung +1; a season unplanted (about 14 game days, at grass's rate) gives one back. A compost bin fills in 7 steps: spoiled food 2, base:dung 2, base:rotten_flesh 2, base:oak_leaves 1, base:birch_leaves 1, base:spruce_leaves 1, base:acacia_leaves 1, base:sapling 1, base:wheat_seeds 1, base:wheat 1; full, it rots into compost in about 23 minutes.

## Difficulty (the `base:difficulty` rule)

| | digests | keeps | harm | can kill | XP lost on death | stamina costs |
|---|---|---|---|---|---|---|
| gentle | x0.75 | x1.5 | x0.5 | no, stops at a tenth | 10% | x0.8 |
| normal | x1 | x1 | x1 | yes | 25% | x1 |
| harsh | x1.25 | x0.6 | x1.5 | yes | 50% | x1.15 |

## Levels

| level | XP to the next | XP from level 1 |
|---|---|---|
| 1 | 20 | 0 |
| 2 | 34 | 20 |
| 5 | 95 | 177 |
| 10 | 237 | 919 |
| 25 | 863 | 8554 |
| 50 | 2338 | 46897 |
| 75 | 4199 | 126982 |
| 99 | 6275 | 251114 |
| 100 | 6367 | 257389 |
