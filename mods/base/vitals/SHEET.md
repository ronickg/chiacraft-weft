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

A portion, what it holds of each nutrient, how long it digests, and game days fresh then stale.

| food | g | units | protein | energy | fat | greens | water | digests | keeps | |
|---|---|---|---|---|---|---|---|---|---|---|
| base:apple | 180 | 18 | 1 | 25 | 0 | 5 | 154 | 9 min | 20 + 20 | cooling |
| base:bread | 250 | 25 | 22 | 122 | 8 | 7 | 90 | 26 min | 5 + 5 | hot when cooked |
| base:porkchop | 150 | 15 | 31 | 0 | 9 | 0 | 108 | 22 min | 2 + 2 | raw |
| base:cooked_porkchop | 110 | 11 | 30 | 0 | 10 | 0 | 67 | 19 min | 4 + 4 | hot when cooked |
| base:beef | 200 | 20 | 42 | 0 | 16 | 0 | 140 | 31 min | 2 + 2 | raw |
| base:cooked_beef | 150 | 15 | 42 | 0 | 16 | 0 | 88 | 28 min | 4 + 4 | hot when cooked |
| base:chicken | 300 | 30 | 56 | 0 | 45 | 0 | 206 | 53 min | 2 + 2 | raw |
| base:cooked_chicken | 220 | 22 | 60 | 0 | 30 | 0 | 131 | 43 min | 4 + 4 | hot when cooked |
| base:mutton | 150 | 15 | 26 | 0 | 24 | 0 | 96 | 26 min | 2 + 2 | raw |
| base:cooked_mutton | 110 | 11 | 28 | 0 | 18 | 0 | 63 | 22 min | 4 + 4 | hot when cooked |
| base:rotten_flesh | 120 | 12 | 18 | 0 | 12 | 0 | 78 | 18 min | 0 + 0 | raw |
| base:spider_eye | 30 | 3 | 4 | 1 | 1 | 0 | 22 | 4 min | 3 + 3 | raw, poisonous |
| base:cooked_egg | 100 | 10 | 14 | 1 | 15 | 0 | 70 | 16 min | 2 + 2 | hot when cooked |
| base:apple_pie | 200 | 20 | 5 | 74 | 25 | 4 | 104 | 25 min | 4 + 4 | hot when cooked |
| base:berries | 100 | 10 | 1 | 14 | 0 | 3 | 84 | 5 min | 6 + 6 | cooling |
| animals:kibble | 100 | 10 | 25 | 45 | 14 | 4 | 10 | 19 min | 90 + 90 |  |

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

Each biome's year mean, humidity and half its summer-to-winter range, and the air at sea level under an open sky: midwinter before dawn, midsummer mid afternoon.

| biome | mean °C | humidity | seasons ± | coldest | hottest |
|---|---|---|---|---|---|
| base:plains | 12 | 0.5 | 12 | -8 | 35 |
| base:forest | 11 | 0.65 | 10 | -6 | 31 |
| base:birch_forest | 9 | 0.6 | 12 | -10 | 31 |
| base:taiga | 0 | 0.55 | 18 | -25 | 29 |
| base:snowy_plains | -10 | 0.4 | 14 | -33 | 16 |
| base:desert | 25 | 0.1 | 9 | 5 | 48 |
| base:savanna | 26 | 0.3 | 4 | 13 | 43 |
| base:swamp | 19 | 0.9 | 8 | 6 | 35 |
| base:mountains | 6 | 0.5 | 12 | -14 | 29 |
| base:ocean | 14 | 0.8 | 6 | 3 | 29 |
| base:beach | 17 | 0.7 | 8 | 3 | 34 |
| base:deep_caves | 12 | 0.6 | 0 | 12 | 12 |
| animals:red_dunes | 28 | 0.1 | 6 | 11 | 48 |
| animals:scrubland | 26 | 0.25 | 5 | 11 | 44 |
| animals:floodplain | 24 | 0.8 | 4 | 15 | 37 |
| animals:fern_forest | 23 | 0.85 | 3 | 15 | 34 |
| animals:crags | 15 | 0.5 | 6 | 1 | 32 |

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
