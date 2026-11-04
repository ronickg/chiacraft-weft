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

A new player arrives with base:bread and base:apple and base:cooked_chicken in the stomach (65 units): max health 90 and stamina 82 of 100, wearing off as it digests (an empty stomach: 40). A new world begins on spring's day 8, a quarter of the year from midwinter.

## Foods

A portion, its kind for the animals' diets, what it holds of each nutrient, how long it digests, and game days fresh then stale.

| food | kind | g | units | protein | energy | fat | greens | water | digests | keeps | |
|---|---|---|---|---|---|---|---|---|---|---|---|
| base:apple | fruit | 180 | 18 | 1 | 21 | 0 | 5 | 154 | 9 min | 20 + 20 | cooling |
| base:bread | seeds | 250 | 25 | 22 | 122 | 8 | 7 | 90 | 26 min | 5 + 5 | hot when cooked |
| base:porkchop | meat | 150 | 15 | 31 | 0 | 9 | 0 | 108 | 22 min | 2 + 2 | raw |
| base:cooked_porkchop | meat | 110 | 11 | 30 | 0 | 10 | 0 | 67 | 19 min | 4 + 4 | hot when cooked |
| base:beef | meat | 200 | 20 | 42 | 0 | 16 | 0 | 140 | 31 min | 2 + 2 | raw |
| base:cooked_beef | meat | 150 | 15 | 42 | 0 | 16 | 0 | 88 | 28 min | 4 + 4 | hot when cooked |
| base:chicken | meat | 300 | 30 | 56 | 0 | 45 | 0 | 198 | 53 min | 2 + 2 | raw |
| base:cooked_chicken | meat | 220 | 22 | 60 | 0 | 30 | 0 | 131 | 43 min | 4 + 4 | hot when cooked |
| base:mutton | meat | 150 | 15 | 26 | 0 | 24 | 0 | 96 | 26 min | 2 + 2 | raw |
| base:cooked_mutton | meat | 110 | 11 | 28 | 0 | 18 | 0 | 63 | 22 min | 4 + 4 | hot when cooked |
| base:rotten_flesh | carrion | 120 | 12 | 18 | 0 | 12 | 0 | 78 | 18 min | 0 + 0 | raw |
| base:spider_eye | insects | 30 | 3 | 4 | 1 | 1 | 0 | 22 | 4 min | 3 + 3 | raw, poisonous |
| base:cooked_egg | meat | 100 | 10 | 14 | 1 | 15 | 0 | 70 | 16 min | 2 + 2 | hot when cooked |
| base:apple_pie | fruit | 200 | 20 | 4 | 65 | 25 | 4 | 105 | 24 min | 4 + 4 | hot when cooked |
| base:berries | fruit | 100 | 10 | 1 | 12 | 0 | 3 | 84 | 5 min | 6 + 6 | cooling |
| base:dried_meat | meat | 75 | 8 | 41 | 2 | 11 | 0 | 15 | 20 min | 30 + 30 |  |
| base:smoked_meat | meat | 120 | 12 | 36 | 0 | 17 | 0 | 60 | 24 min | 14 + 14 |  |
| base:salted_meat | meat | 160 | 16 | 35 | 0 | 22 | 0 | 88 | 29 min | 45 + 45 |  |
| base:dried_apple | fruit | 40 | 4 | 0 | 23 | 0 | 4 | 13 | 3 min | 56 + 56 |  |
| base:dried_berries | fruit | 30 | 3 | 1 | 22 | 1 | 2 | 4 | 2 min | 56 + 56 |  |
| base:lingonberries | fruit | 100 | 10 | 0 | 8 | 0 | 5 | 87 | 5 min | 20 + 20 | cooling |
| base:cloudberries | fruit | 100 | 10 | 2 | 2 | 1 | 16 | 85 | 6 min | 3 + 3 | cooling |
| base:crowberries | fruit | 100 | 10 | 1 | 7 | 1 | 5 | 85 | 6 min | 10 + 10 | cooling |
| base:raspberries | fruit | 100 | 10 | 1 | 5 | 1 | 9 | 86 | 5 min | 2 + 2 | cooling |
| base:huckleberries | fruit | 100 | 10 | 1 | 12 | 0 | 3 | 84 | 5 min | 5 + 5 | cooling |
| base:beach_plums | fruit | 100 | 10 | 1 | 10 | 0 | 2 | 87 | 5 min | 5 + 5 | cooling |
| base:prickly_pear_fruit | fruit | 100 | 10 | 1 | 6 | 0 | 5 | 88 | 5 min | 7 + 7 | cooling |
| base:rose_hip | fruit | 30 | 3 | 0 | 4 | 0 | 20 | 18 | 2 min | 30 + 30 | cooling |
| base:hazelnut | seeds | 30 | 3 | 4 | 2 | 18 | 3 | 2 | 10 min | 60 + 60 |  |
| base:pine_nut | seeds | 30 | 3 | 4 | 3 | 21 | 1 | 1 | 11 min | 60 + 60 |  |
| base:chia_seeds | seeds | 30 | 3 | 5 | 2 | 9 | 10 | 2 | 7 min | 365 + 365 |  |
| base:tepary_beans | seeds | 50 | 5 | 12 | 22 | 1 | 8 | 5 | 7 min | 365 + 365 |  |
| base:brazil_nut | seeds | 30 | 3 | 4 | 1 | 20 | 2 | 1 | 11 min | 180 + 180 |  |
| base:cacao_pod | fruit | 100 | 10 | 1 | 13 | 0 | 3 | 81 | 6 min | 7 + 7 | cooling |
| base:cacao_beans | seeds | 30 | 3 | 4 | 4 | 14 | 6 | 2 | 9 min | 365 + 365 |  |
| base:roasted_cacao | seeds | 30 | 3 | 4 | 4 | 14 | 6 | 1 | 9 min | 180 + 180 | hot when cooked |
| base:bananas | fruit | 150 | 15 | 2 | 30 | 0 | 5 | 112 | 11 min | 6 + 6 | cooling |
| base:sunflower_seeds | seeds | 30 | 3 | 6 | 3 | 15 | 3 | 1 | 10 min | 90 + 90 |  |
| base:kelp | water_plants | 50 | 5 | 1 | 5 | 0 | 1 | 41 | 3 min | 3 + 3 |  |
| base:cattail_root | roots | 100 | 10 | 1 | 17 | 0 | 3 | 78 | 6 min | 14 + 14 |  |
| base:cooked_wild_rice | seeds | 150 | 15 | 6 | 29 | 0 | 3 | 111 | 12 min | 3 + 3 | hot when cooked |
| base:mushroom_brown | fungi | 50 | 5 | 2 | 2 | 0 | 1 | 46 | 3 min | 3 + 3 |  |
| base:mushroom_red | fungi | 50 | 5 | 2 | 2 | 0 | 1 | 46 | 3 min | 3 + 3 | poisonous |
| animals:venison | meat | 200 | 20 | 44 | 0 | 5 | 0 | 147 | 27 min | 2 + 2 | raw |
| animals:cooked_venison | meat | 150 | 15 | 45 | 0 | 5 | 0 | 98 | 24 min | 4 + 4 | hot when cooked |
| animals:hare | meat | 150 | 15 | 33 | 0 | 3 | 0 | 109 | 20 min | 2 + 2 | raw |
| animals:cooked_hare | meat | 110 | 11 | 36 | 0 | 4 | 0 | 70 | 19 min | 4 + 4 | hot when cooked |
| animals:trout | fish | 150 | 15 | 31 | 0 | 5 | 0 | 108 | 20 min | 1 + 1 | raw |
| animals:cooked_trout | fish | 110 | 11 | 25 | 0 | 6 | 0 | 76 | 17 min | 3 + 3 | hot when cooked |
| animals:pacu | fish | 150 | 15 | 30 | 0 | 3 | 0 | 117 | 17 min | 1 + 1 | raw |
| animals:cooked_pacu | fish | 110 | 11 | 29 | 0 | 3 | 0 | 79 | 16 min | 3 + 3 | hot when cooked |
| animals:kibble | meat | 100 | 10 | 25 | 45 | 14 | 4 | 10 | 19 min | 90 + 90 |  |
| base:carrot | roots | 60 | 6 | 1 | 4 | 0 | 2 | 53 | 3 min | 20 + 20 |  |
| base:potato | roots | 170 | 17 | 3 | 30 | 0 | 7 | 135 | 11 min | 30 + 30 |  |
| base:baked_potato | roots | 140 | 14 | 4 | 30 | 0 | 4 | 105 | 10 min | 3 + 3 | hot when cooked |
| base:beetroot | roots | 80 | 8 | 1 | 5 | 0 | 3 | 70 | 4 min | 20 + 20 |  |
| base:corn | seeds | 100 | 10 | 3 | 17 | 1 | 3 | 76 | 8 min | 2 + 2 |  |
| base:roasted_corn | seeds | 100 | 10 | 3 | 19 | 2 | 3 | 73 | 8 min | 3 + 3 | hot when cooked |
| base:beans | seeds | 100 | 10 | 2 | 4 | 0 | 4 | 90 | 5 min | 5 + 5 |  |
| base:cooked_rice | seeds | 150 | 15 | 4 | 42 | 0 | 1 | 103 | 11 min | 3 + 3 | hot when cooked |
| base:cassava | roots | 100 | 10 | 1 | 36 | 0 | 4 | 60 | 7 min | 2 + 2 |  |
| base:pumpkin_slice | fruit | 200 | 20 | 2 | 12 | 0 | 3 | 183 | 9 min | 4 + 4 |  |
| base:melon_slice | fruit | 150 | 15 | 1 | 11 | 0 | 2 | 137 | 7 min | 2 + 2 | cooling |

## Meals and preserving

A stew (`vitals::meals`) cooks 2-4 foods in 10 s in a cooking pot; raw meat in it is cooked. Each goes into the stomach as its own portion at x1.25 (so each counts toward well fed), the meal warms as one hot dish, and it keeps as long as its shortest-keeping ingredient. A campfire cooks what a furnace cooks of food, x1.5 the time, with no fuel. Nothing is made of spoiled food, and what is made of stale food starts as far through its own life as its stalest part.

Preserving (`vitals::preserve`): a drying rack dries in 20 min in the best air (25 °C, humidity 0.3, a breeze; nothing below 5 °C or above humidity 0.85, nor in the rain), and smokes in 10 min over a fire (lava, a lit furnace) within 2 blocks below, in any weather; salting is crafted with salt, boiled out of a water bucket in a furnace in 20 s.

| from | dried | smoked | salted | keeps (days) |
|---|---|---|---|---|
| base:apple (40) | base:dried_apple | - | - | 112 |
| base:porkchop (4) | base:dried_meat | base:smoked_meat | base:salted_meat | 60, 28, 90 |
| base:beef (4) | base:dried_meat | base:smoked_meat | base:salted_meat | 60, 28, 90 |
| base:chicken (4) | base:dried_meat | base:smoked_meat | base:salted_meat | 60, 28, 90 |
| base:mutton (4) | base:dried_meat | base:smoked_meat | base:salted_meat | 60, 28, 90 |
| base:berries (12) | base:dried_berries | - | - | 112 |
| base:lingonberries (40) | base:dried_berries | - | - | 112 |
| base:cloudberries (6) | base:dried_berries | - | - | 112 |
| base:crowberries (20) | base:dried_berries | - | - | 112 |
| base:raspberries (4) | base:dried_berries | - | - | 112 |
| base:huckleberries (10) | base:dried_berries | - | - | 112 |
| base:beach_plums (10) | base:dried_berries | - | - | 112 |
| base:prickly_pear_fruit (14) | base:dried_berries | - | - | 112 |
| base:cacao_pod (14) | base:dried_berries | - | - | 112 |
| base:bananas (12) | base:dried_berries | - | - | 112 |
| animals:venison (4) | base:dried_meat | base:smoked_meat | base:salted_meat | 60, 28, 90 |
| animals:hare (4) | base:dried_meat | base:smoked_meat | base:salted_meat | 60, 28, 90 |
| animals:trout (2) | base:dried_meat | base:smoked_meat | base:salted_meat | 60, 28, 90 |
| animals:pacu (2) | base:dried_meat | base:smoked_meat | base:salted_meat | 60, 28, 90 |
| base:pumpkin_slice (8) | base:dried_berries | - | - | 112 |
| base:melon_slice (4) | base:dried_berries | - | - | 112 |

## Clothing

Worn in four slots of their own under armor (hat, coat, trousers, boots), no armor value; armor and clothes stack, and leather armor is hide. A full set's real insulation in clo at 12 °C of comfort each; a piece gives its share ([0.15, 0.4, 0.3, 0.15], head to feet). Wet, armor keeps 35% of its warmth; being wet itself feels 6 °C colder. Each piece is sewn with [1, 2, 1, 1] string, a string is 2 plant fibre. A new player arrives in linen hat, wool coat, hide trousers, hide boots.

| material | clo | cold °C | heat °C | kept wet | windproof | pieces to make | from |
|---|---|---|---|---|---|---|---|
| fur | 2.5 | 30 | -9 | 30% | 80% | [2, 4, 3, 2] | pelts of furred wild animals |
| wool | 1.2 | 14 | -2 | 60% | 30% | [1, 2, 1, 1] | sheep's wool |
| hide | 0.8 | 10 | -1 | 30% | 70% | [1, 2, 1, 1] | leather |
| linen | 0.5 | 6 | 4 | 20% | 10% | [2, 4, 3, 2] | plant fibre spun and woven |

Cloaks, in a slot of their own over clothes and armor (covering 60% of the body against the wind):

| cloak | cold °C | heat °C | kept wet | windproof | keeps the rain off | from |
|---|---|---|---|---|---|---|
| wool_cloak | 4 | -1 | 60% | 30% | no | sheep's wool |
| fur_robe | 8 | -3 | 30% | 60% | no | pelts |
| rain_cape | 1 | 0 | 100% | 50% | yes | leather |

## Rooms

An enclosed space (`vitals::shelter`; up to 400 blocks of air, none open to the sky) shuts out the wind, lets in its walls' share of the day's swing, and keeps 2 °C over the air outside empty, or a fire's gain by its walls with a fire in it (full up to 60 blocks of air, at least 40% in a bigger room; never less than the fire gives beside it).

| wall | a fire in the room | the day's swing let in |
|---|---|---|
| log | +25 °C | 40% |
| earth | +25 °C | 15% |
| snow | +25 °C | 90% |
| plank | +16 °C | 60% |
| stone | +16 °C | 15% |
| hide | +12 °C | 90% |
| glass | +12 °C | 90% |

## Crafts that wait

What takes days in real life takes its time on the growing pace (`vitals::crafts`; game days = 0.0645 · real days^0.7), at a station that keeps working unwatched.

| station | in | with | out | real days | in the game | holds |
|---|---|---|---|---|---|---|
| tanning_frame | #hides | - | base:leather | 3 | 2.8 min | 1 |
| tanning_pit | #hides | #bark_tannin | base:sole_leather | 365 | 4.0 game days | 8 |
| charcoal_clamp | #logs | - | base:coal | 7 | 5.0 min | 16 |

## Felling

An axe at a tree's foot fells it in one chop that takes logs^0.75 times one log's (`vitals::felling`), with the axe's own speed on top. Each tier of axe takes trees up to a size for their wood; past it a tree still falls, 8 times slower, and gives nothing. A tree that loses its foot, or a log of its trunk, by any means falls (none stands in the air); bare hands and other tools chop the foot as slowly as an axe past its tier, and as fruitlessly, but keep the wood of a tree of up to 10 logs at the hand's pace (a stopgap until the first hours give a hand-made axe). Never more than 60 min. Felled, a tree within its tier lies on the ground in pieces of 4 logs, each cut as long as felling a tree of its logs takes; the axe wears by logs^0.75 for a felling or a piece. A tree falls as itself: a 1 s creak, then 2.5 s for one 10 high (2-5 s, by the square root of its height), its leaves breaking off past 20° until 70% have, the rest bursting on the ground; on the way leaves and branches in its path break, and a trunk it strikes falls too if the falling tree has 2x its logs, or else it catches there and settles as blocks; a creature it hits takes 2 + a tenth of its logs, up to 20. Seconds for a typical axe of each tier (a log is hardness 2), * past its tier:

| tier | harvest | takes up to | speed | oak (20) | birch (34) | araucaria (72) | young sequoia (165) | sequoia (1015) |
|---|---|---|---|---|---|---|---|---|
| bare hands, other tools | - | 10 logs (stopgap) | 1 | 4 min * | 6 min * | 10 min * | 18 min * | 60 min * |
| wood, gold | 0 | 40 logs | 2 | 14 s | 21 s | 5 min * | 9 min * | 36 min * |
| bone, antler | 50 | 64 logs | 2.5 | 11 s | 17 s | 4 min * | 7 min * | 29 min * |
| flint, chert, obsidian, stone | 100 | 96 logs | 2.91 | 10 s | 15 s | 25 s | 6 min * | 25 min * |
| copper | 150 | 192 logs | 4.85 | 6 s | 9 s | 15 s | 28 s | 15 min * |
| bronze, iron, wrought iron | 200 | 512 logs | 6.79 | 4 s | 6 s | 11 s | 20 s | 11 min * |
| steel, diamond (old) | 300 | 1536 logs | 7.76 | 4 s | 5 s | 10 s | 18 s | 70 s |
| diamond | 400 | 4096 logs | 8 | 4 s | 5 s | 9 s | 17 s | 67 s |

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
| base:tropical_forest | Manaus, the central Amazon | 27.4 | 0.75 | 1 | 22 | 36 | 27% | 56% | 57% | 25% |
| base:mountains | Davos, 1,594 m | 6 | 0.63 | 8.8 | -8 | 23 | 32% | 45% | 31% | 29% |
| base:sequoia_grove | Giant Forest, Sequoia National Park, 1,721 m | 10.4 | 0.49 | 8.4 | -4 | 28 | 20% | 5% | 10% | 24% |
| base:ocean | Ponta Delgada, the Azores | 18.2 | 0.97 | 4.1 | 11 | 28 | 33% | 19% | 38% | 43% |
| base:coral_reef | Key West, the Florida Reef Tract | 26.1 | 0.97 | 4.2 | 19 | 36 | 15% | 33% | 30% | 16% |
| base:lagoon | Key West, the Florida Keys' back-reef lagoon | 26.1 | 0.97 | 4.2 | 19 | 36 | 15% | 33% | 30% | 16% |
| base:beach | Faro, Portugal | 18.2 | 0.74 | 6.2 | 8 | 32 | 16% | 1% | 16% | 20% |
| base:deep_caves | any cave: the surface's year mean | 12 | 0.95 | 0 | 12 | 12 | 0% | 0% | 0% | 0% |
| animals:red_dunes | the Djadokhta Formation, Mongolia | 17 | 0.2 | 12 | -2 | 40 | 10% | 2% | 3% | 12% |
| animals:scrubland | the Barun Goyot Formation, Mongolia | 17 | 0.3 | 11 | -1 | 38 | 6% | 25% | 8% | 4% |
| animals:floodplain | the Cedar Mountain Formation, Utah | 21 | 0.5 | 8 | 7 | 38 | 8% | 45% | 25% | 4% |
| animals:fern_forest | the Yixian Formation, Liaoning | 10 | 0.8 | 10 | -4 | 27 | 25% | 45% | 30% | 15% |
| animals:crags | the Sevier highlands, Utah | 15 | 0.6 | 8 | 2 | 31 | 25% | 40% | 25% | 15% |

## Wind and fog

Each place's mean wind in m/s and its share of days with fog, in spring, summer, autumn and winter (`reports/ecology/Wind and fog.md`), and when its fog lies (`vitals::air`). A day blows from 45% to 155% of the mean, a quarter more mid afternoon than before dawn, 1.8 times under a thunderstorm; still fog clears from 3.5 to 7 m/s, sea fog from 9 to 14. Under 1 °C what falls is snow; a day of it lays 0.4 of a deep cover and each degree of a day's mean above freezing melts 0.05; near players it lies as snow layers (an eighth of a block each) from 0.2 of a cover, 4 of them when deep, burying plants from 2, a column looked at every 27 s and a layer melting in 20 s in air of 8 °C or warmer (fewer the colder, none in a frost); falling snow in 6 to 12 m/s of wind is a blizzard; a place's air is its own and its neighbours' over a band 32 blocks wide, and deep in a closed forest (12 blocks in from the edge) the wind is 25% of the open's, with its chill and blizzard; dry ground (humidity under 0.35) lifts dust from 7 to 11 m/s. Out in the open the wind chills the cold (the wind chill index, 10 °C and under), and in thick fog animals see 35% as far.

| biome | wind: spring | summer | autumn | winter | fog: spring | summer | autumn | winter | fog lies |
|---|---|---|---|---|---|---|---|---|---|
| base:plains | 3.8 | 3.0 | 3.0 | 3.1 | 6% | 6% | 7% | 9% | before dawn to mid morning |
| base:forest | 2.5 | 2.0 | 2.4 | 3.0 | 6% | 5% | 18% | 12% | before dawn to mid morning |
| base:birch_forest | 1.5 | 1.1 | 1.4 | 1.6 | 2% | 3% | 3% | 2% | before dawn to mid morning |
| base:taiga | 2.3 | 2.0 | 1.5 | 1.1 | 2% | 3% | 8% | 9% | ice fog, cold nights and the dark |
| base:snowy_plains | 5.3 | 5.3 | 6.1 | 6.0 | 29% | 51% | 32% | 20% | off the sea, any hour |
| base:desert | 2.3 | 2.6 | 1.6 | 1.4 | 0% | 0% | 0% | 0% | before dawn to mid morning |
| base:savanna | 1.5 | 1.2 | 1.2 | 1.6 | 1% | 3% | 2% | 0% | before dawn to mid morning |
| base:swamp | 3.5 | 2.9 | 2.9 | 3.6 | 3% | 4% | 6% | 5% | before dawn to mid morning |
| base:tropical_forest | 1.8 | 1.5 | 1.4 | 1.7 | 3% | 2% | 2% | 4% | before dawn to mid morning |
| base:mountains | 2.4 | 2.7 | 2.2 | 1.7 | 2% | 1% | 3% | 3% | before dawn to mid morning |
| base:sequoia_grove | 2.0 | 1.8 | 1.6 | 1.8 | 3% | 1% | 2% | 4% | before dawn to mid morning |
| base:ocean | 3.1 | 2.5 | 2.7 | 3.4 | 2% | 1% | 0% | 0% | off the sea, any hour |
| base:coral_reef | 4.8 | 4.0 | 4.3 | 4.8 | 1% | 0% | 0% | 1% | off the sea, any hour |
| base:lagoon | 4.5 | 3.8 | 4.0 | 4.5 | 1% | 0% | 0% | 1% | off the sea, any hour |
| base:beach | 4.1 | 3.7 | 3.4 | 3.8 | 1% | 2% | 2% | 3% | before dawn to mid morning |
| animals:red_dunes | 4.5 | 3.5 | 3.3 | 2.8 | 0% | 0% | 0% | 0% | before dawn to mid morning |
| animals:scrubland | 3.8 | 3.2 | 2.9 | 2.5 | 1% | 1% | 1% | 0% | before dawn to mid morning |
| animals:floodplain | 2.5 | 2.5 | 2.0 | 1.8 | 2% | 1% | 2% | 5% | before dawn to mid morning |
| animals:fern_forest | 2.8 | 2.0 | 1.8 | 1.8 | 2% | 4% | 7% | 2% | before dawn to mid morning |
| animals:crags | 4.5 | 3.7 | 3.7 | 3.7 | 5% | 1% | 3% | 7% | before dawn to mid morning |

## Plants

What each growing plant needs (`vitals::plants`, from `reports/ecology/Plants and crops.md`): the air it grows in (from, best, to), its water, the frost that kills it (or its fruit, or its stalk to the base), its real days to harvest and the game's minutes at the best rate, and where and in which seasons it grows at least a third of its best (sp, su, au, wi; marked * where that season's nights freeze it), with each biome's rain and warmth.

| plant | real plant | °C | water | frost | real days | minutes | grows |
|---|---|---|---|---|---|---|---|
| base:wheat | spring wheat | 0, 15-23, 34 | Moderate | -2 °C | 110 | 26 | plains (su au), forest (su au), birch_forest (su au), taiga (su), savanna (su au), swamp (au), tropical_forest (su au), mountains (su au), ocean (sp su au wi), coral_reef (au), lagoon (au), beach (au wi), floodplain (au), fern_forest (su au), crags (su au) |
| base:barley | barley | 0, 15-20, 32 | Moderate | -4 °C | 105 | 25 | plains (au), forest (su au), birch_forest (su au), taiga (su), mountains (su au), ocean (sp su au wi), beach (au wi), floodplain (au), fern_forest (su au), crags (su au) |
| base:rye | winter rye | 0, 12-20, 30 | Moderate | -30 °C | 300 | 52 | plains (au), forest (su au), birch_forest (su au), taiga (su), mountains (su au), ocean (sp su au wi), beach (au wi), floodplain (au), fern_forest (su au), crags (su au) |
| base:carrots | carrot | 4, 16-21, 30 | Moderate | -5 °C | 95 | 23 | plains (au), forest (su au), birch_forest (su), taiga (su), mountains (su), ocean (sp su au wi), beach (au), floodplain (au), fern_forest (su au), crags (su au) |
| base:potatoes | potato | 7, 15-20, 30 | Moderate | -1 °C | 135 | 30 | plains (au), forest (su au), birch_forest (su), taiga (su), mountains (su), ocean (sp su au wi), beach (au), floodplain (au), fern_forest (su au), crags (su au) |
| base:beetroots | beetroot | 4.5, 15-20, 30 | Moderate | -3 °C | 72 | 19 | plains (au), forest (su au), birch_forest (su), taiga (su), mountains (su), ocean (sp su au wi), beach (au wi), floodplain (au), fern_forest (su au), crags (su au) |
| base:maize | maize | 10, 24-30, 38 | Moderate | 0 °C | 150 | 32 | plains (su), forest (su), birch_forest (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), ocean (su au), coral_reef (su au), lagoon (su au), scrubland (su), floodplain (su au), fern_forest (su), crags (su) |
| base:paddy_rice | paddy rice | 10, 25-30, 35 | Wet | 0 °C | 120 | 28 | plains (su), forest (su), birch_forest (su), taiga (su), desert (sp su au wi), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), sequoia_grove (su), ocean (sp su au), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au), red_dunes (sp su au), scrubland (sp su au), floodplain (sp su au), fern_forest (su), crags (sp su au) |
| base:beans | common bean | 10, 18-25, 30 | Moderate | 0 °C | 100 | 24 | plains (su), forest (su), birch_forest (su), taiga (su), savanna (au), swamp (au), tropical_forest (au), mountains (su), ocean (sp su au wi), coral_reef (au), lagoon (au), beach (au), floodplain (au), fern_forest (su), crags (su au) |
| base:sorghum | sorghum | 8, 27-34, 42 | Dry | 0 °C | 125 | 28 | plains (su), forest (su), birch_forest (su), taiga (su), desert (sp su au), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), sequoia_grove (su), ocean (sp su au wi), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au), red_dunes (su), scrubland (su), floodplain (sp su au), fern_forest (su), crags (su au) |
| base:millet | pearl millet | 10, 30-34, 45 | Dry | 0 °C | 105 | 25 | plains (su), forest (su), birch_forest (su), desert (sp su au), savanna (sp su au wi), swamp (sp su au), tropical_forest (sp su au wi), ocean (su au), coral_reef (sp su au wi), lagoon (sp su au wi), beach (su au), red_dunes (su), scrubland (su), floodplain (su au), fern_forest (su), crags (su) |
| base:tepary_beans | tepary bean (Phaseolus acutifolius) | 12, 25-32, 42 | Dry | 0 °C | 75 | 20 | plains (su), forest (su), birch_forest (su), desert (sp su au), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), ocean (sp su au), coral_reef (sp su au wi), lagoon (sp su au wi), beach (su au), red_dunes (su), scrubland (su), floodplain (sp su au), fern_forest (su), crags (su) |
| base:cassava | cassava | 12, 25-29, 37 | Moderate | 0 °C | 390 | 63 | plains (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), ocean (su au), coral_reef (su au), lagoon (su au), scrubland (su), floodplain (su au), fern_forest (su), crags (su) |
| base:pumpkin_stem | pumpkin, squash | 10, 20-30, 35 | Moderate | 0 °C | 110 | 26 | plains (su), forest (su), birch_forest (su), taiga (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), ocean (su au wi), coral_reef (su au), lagoon (su au), beach (au), scrubland (su), floodplain (su au), fern_forest (su), crags (su au) |
| base:melon_stem | melon | 12, 25-30, 38 | Moderate | 0 °C | 140 | 31 | plains (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), ocean (su au), coral_reef (su au), lagoon (su au), scrubland (su), floodplain (su au), fern_forest (su), crags (su) |
| base:berry_bush | blueberry | 7, 20-26, 32 | Moderate | -2 °C | 75 | 20 | plains (su), forest (su), birch_forest (su), taiga (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), mountains (su), ocean (su au wi), coral_reef (au), lagoon (au), beach (au), scrubland (su), floodplain (su au), fern_forest (su), crags (su au) |
| base:sugar_cane | sugar cane | 12, 22-30, 38 | Wet | -1 °C | - | the game's | plains (su), forest (su), birch_forest (su), taiga (su), desert (sp su au wi), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), sequoia_grove (su), ocean (sp su au), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au), red_dunes (sp su au), scrubland (sp su au), floodplain (sp su au), fern_forest (su), crags (sp su au) |
| base:kelp | kelp (Laminaria) | 0, 5-15, 22 | Wet | hardy | - | the game's | plains (sp au), forest (sp su au), birch_forest (sp su au), taiga (sp su au), snowy_plains (su), desert (wi), mountains (sp su au), sequoia_grove (sp su au wi), ocean (sp au wi), beach (sp au wi), red_dunes (sp au wi), scrubland (sp au wi), floodplain (wi), fern_forest (sp su au), crags (sp au wi) |
| base:cactus | saguaro | 10, 25-40, 50 | Dry | -7 °C | - | the game's | plains (su), forest (su), birch_forest (su), taiga (su), desert (sp su au), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), sequoia_grove (su), ocean (sp su au), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au), red_dunes (su), scrubland (su), floodplain (sp su au), fern_forest (su), crags (su) |
| base:sapling | oak | 5, 18-28, 38 | Moderate | hardy | - | the game's | plains (su au), forest (su), birch_forest (su), taiga (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), mountains (su), ocean (sp su au wi), coral_reef (su au), lagoon (su au), beach (au), scrubland (su), floodplain (su au), fern_forest (su au), crags (su au) |
| base:grazed_grass | prairie grass | 5, 15-30, 40 | Moderate | hardy | 18 | 7 | plains (su au), forest (su au), birch_forest (su), taiga (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), mountains (su), ocean (sp su au wi), coral_reef (su au), lagoon (su au), beach (au wi), scrubland (su), floodplain (su au), fern_forest (su au), crags (su au) |
| base:sedge | sedges | -1, 8-22, 32 | Moderate | hardy | 18 | 7 | plains (sp su au), forest (sp su au), birch_forest (su au), taiga (su), snowy_plains (su), savanna (au), tropical_forest (au), mountains (su au), ocean (sp su au wi), coral_reef (au), lagoon (au), beach (au wi), floodplain (au), fern_forest (su au), crags (su au wi) |
| base:cottongrass | cottongrass (Eriophorum) | -2, 6-16, 25 | Moderate | hardy | 18 | 7 | plains (sp au), forest (sp su au wi), birch_forest (su au), taiga (su), snowy_plains (su), mountains (sp su au), sequoia_grove (wi), ocean (sp au wi), beach (au wi), fern_forest (su au), crags (au wi) |
| base:reindeer_lichen | reindeer lichen (Cladonia) | 0, 5-15, 25 | Dry | hardy | 3650 | 301 | plains (sp au), forest (sp su au), birch_forest (sp su au), taiga (su au), snowy_plains (su), mountains (sp su au), sequoia_grove (sp su au wi), ocean (sp au wi), beach (sp au wi), red_dunes (sp au wi), scrubland (sp au wi), floodplain (au wi), fern_forest (sp su au), crags (sp au wi) |
| base:dune_grass | American beachgrass (Ammophila breviligulata) | 5, 18-26, 35 | Dry | hardy | 30 | 10 | plains (sp su au), forest (su au), birch_forest (su), taiga (su), desert (sp au wi), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), mountains (su), sequoia_grove (su), ocean (sp su au wi), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au wi), red_dunes (sp su au), scrubland (sp su au), floodplain (sp su au wi), fern_forest (su au), crags (sp su au) |
| base:clover | white clover | 5, 18-25, 32 | Moderate | hardy | 30 | 10 | plains (su au), forest (su), birch_forest (su), taiga (su), savanna (sp su au), swamp (au), tropical_forest (su au wi), mountains (su), ocean (sp su au wi), coral_reef (au), lagoon (au), beach (au), scrubland (su), floodplain (au), fern_forest (su au), crags (su au) |
| base:lingonberry_bush | lingonberry | 5, 12-20, 28 | Moderate | -3 °C | 75 | 20 | plains (au), forest (su au), birch_forest (su au), taiga (su), mountains (su), ocean (sp su au wi), beach (au wi), floodplain (au), fern_forest (su au), crags (su au) |
| base:cloudberry_bush | cloudberry | 0, 8-16, 25 | Wet | -2 °C | 45 | 14 | plains (sp au), forest (sp su au), birch_forest (sp su au), taiga (su), snowy_plains (su), desert (wi), mountains (sp su au), sequoia_grove (sp su au wi*), ocean (sp su au wi), beach (sp au wi), red_dunes (sp au wi*), scrubland (sp au wi), floodplain (sp au wi), fern_forest (sp su au), crags (sp au wi) |
| base:crowberry_bush | crowberry | -1, 6-15, 25 | Moderate | hardy | 60 | 17 | plains (sp au), forest (sp su au wi), birch_forest (su au), taiga (su), snowy_plains (su), mountains (su au), sequoia_grove (wi), ocean (sp au wi), beach (au wi), fern_forest (su au), crags (au wi) |
| base:hazel_bush | hazel | 5, 16-24, 32 | Moderate | hardy | 180 | 37 | plains (su au), forest (su au), birch_forest (su), taiga (su), savanna (su au), swamp (au), tropical_forest (au), mountains (su), ocean (sp su au wi), coral_reef (au), lagoon (au), beach (au), floodplain (au), fern_forest (su au), crags (su au) |
| base:huckleberry_bush | black huckleberry (Vaccinium membranaceum) | 3, 12-22, 30 | Moderate | -2 °C | 70 | 19 | plains (au), forest (su au), birch_forest (su au), taiga (su), mountains (su au), ocean (sp su au wi), beach (au wi), floodplain (au), fern_forest (su au), crags (su au) |
| base:beach_plum_bush | beach plum (Prunus maritima) | 5, 16-26, 34 | Dry | -2 °C | 100 | 24 | plains (sp su au), forest (su au), birch_forest (su), taiga (su), desert (sp au wi), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), mountains (su), sequoia_grove (su), ocean (sp su au wi), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au wi), red_dunes (sp su au), scrubland (sp su au), floodplain (sp su au wi), fern_forest (su au), crags (sp su au) |
| base:bramble_bush | raspberry and blackberry | 5, 16-25, 32 | Moderate | -2 °C | 60 | 17 | plains (su au), forest (su au), birch_forest (su), taiga (su), savanna (sp su au), swamp (au), tropical_forest (su au wi), mountains (su), ocean (sp su au wi), coral_reef (au), lagoon (au), beach (au), scrubland (su), floodplain (au), fern_forest (su au), crags (su au) |
| base:prickly_pear | prickly pear (Opuntia) | 10, 20-35, 45 | Dry | -10 °C | 90 | 23 | plains (su au), forest (su), birch_forest (su), taiga (su), desert (sp su au wi), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), mountains (su), sequoia_grove (su), ocean (sp su au wi), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au), red_dunes (sp su), scrubland (sp su au), floodplain (sp su au), fern_forest (su), crags (su au) |
| base:chia | desert chia (Salvia columbariae) | 5, 15-28, 38 | Dry | -3 °C | 90 | 23 | plains (sp su au), forest (su au), birch_forest (su), taiga (su), desert (sp su au wi), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), mountains (su), sequoia_grove (sp su), ocean (sp su au wi), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au wi), red_dunes (sp su au), scrubland (sp su au), floodplain (sp su au wi), fern_forest (su au), crags (sp su au) |
| base:wild_sunflower | wild sunflower | 6, 20-28, 38 | Dry | -2 °C | 125 | 28 | plains (su au), forest (su), birch_forest (su), taiga (su), desert (sp su au wi), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), mountains (su), sequoia_grove (su), ocean (sp su au wi), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au wi), red_dunes (sp su au), scrubland (sp su au), floodplain (sp su au), fern_forest (su), crags (sp su au) |
| base:wild_rice | wild rice (Oryza latifolia) | 12, 25-32, 38 | Wet | 0 °C | 150 | 32 | plains (su), forest (su), birch_forest (su), desert (sp su au wi), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), sequoia_grove (su), ocean (sp su au), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au), red_dunes (sp su au), scrubland (sp su au), floodplain (sp su au), fern_forest (su), crags (su) |
| base:mushroom_brown | forest mushrooms | 5, 10-20, 30 | Moderate | -2 °C | 8 | 4 | plains (au), forest (su au), birch_forest (su au), taiga (su), mountains (su au), ocean (sp su au wi), beach (au wi), floodplain (au), fern_forest (su au), crags (su au) |
| base:mushroom_red | fly agaric | 5, 10-20, 30 | Moderate | -2 °C | 8 | 4 | plains (au), forest (su au), birch_forest (su au), taiga (su), mountains (su au), ocean (sp su au wi), beach (au wi), floodplain (au), fern_forest (su au), crags (su au) |
| base:water_hyacinth | water hyacinth | 12, 25-30, 38 | Wet | -1 °C | 10 | 5 | plains (su), forest (su), birch_forest (su), desert (sp su au wi), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), sequoia_grove (su), ocean (sp su au), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au), red_dunes (sp su au), scrubland (sp su au), floodplain (sp su au), fern_forest (su), crags (su) |
| base:mesquite_sapling | mesquite | 12, 25-38, 48 | Dry | -15 °C | - | the game's | plains (su), forest (su), birch_forest (su), desert (sp su au), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), ocean (sp su au), coral_reef (sp su au wi), lagoon (sp su au wi), beach (su au), red_dunes (su), scrubland (su), floodplain (sp su au), fern_forest (su), crags (su) |
| base:brazil_nut_sapling | Brazil nut (Bertholletia excelsa) | 18, 25-32, 38 | Moderate | 0 °C | - | the game's | plains (su), desert (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), coral_reef (su au), lagoon (su au), scrubland (su), floodplain (su), crags (su) |
| base:cacao_sapling | cacao (Theobroma cacao) | 15, 22-30, 35 | Moderate | 0 °C | - | the game's | plains (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), ocean (su au), coral_reef (su au), lagoon (su au), scrubland (su), floodplain (su au), fern_forest (su), crags (su) |
| base:andiroba_sapling | andiroba (Carapa guianensis) | 18, 24-32, 38 | Moderate | 0 °C | - | the game's | plains (su), desert (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), coral_reef (su au), lagoon (su au), scrubland (su), floodplain (su), crags (su) |
| base:matamata_sapling | matamatá (Eschweilera coriacea) | 18, 24-32, 38 | Moderate | 0 °C | - | the game's | plains (su), desert (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), coral_reef (su au), lagoon (su au), scrubland (su), floodplain (su), crags (su) |
| base:ipe_sapling | golden ipê (Handroanthus serratifolius) | 15, 22-32, 40 | Dry | -1 °C | - | the game's | plains (su), birch_forest (su), desert (sp su au), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), ocean (su au), coral_reef (sp su au wi), lagoon (sp su au wi), beach (su au), red_dunes (su), scrubland (su), floodplain (sp su au), fern_forest (su), crags (su) |
| base:acuri_palm_sapling | acuri palm (Attalea phalerata) | 15, 24-32, 40 | Moderate | 0 °C | - | the game's | plains (su), desert (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), ocean (su), coral_reef (su au), lagoon (su au), scrubland (su), floodplain (su au), crags (su) |
| base:sequoia_sapling | giant sequoia (Sequoiadendron giganteum) | 5, 15-24, 35 | Moderate | -24 °C | - | the game's | plains (su au), forest (su au), birch_forest (su), taiga (su), savanna (su au), swamp (su au), tropical_forest (su au wi), mountains (su), ocean (sp su au wi), coral_reef (au), lagoon (au), beach (au wi), scrubland (su), floodplain (su au), fern_forest (su au), crags (su au) |
| base:moriche_palm_sapling | moriche palm (Mauritia flexuosa) | 15, 24-32, 40 | Wet | 0 °C | - | the game's | plains (su), desert (sp su au wi), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), ocean (sp su au), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au), red_dunes (sp su au), scrubland (sp su au), floodplain (sp su au), fern_forest (su), crags (su) |
| animals:araucaria_sapling | araucarian conifer | 5, 18-28, 38 | Moderate | -5 °C | - | the game's | plains (su au), forest (su), birch_forest (su), taiga (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), mountains (su), ocean (sp su au wi), coral_reef (su au), lagoon (su au), beach (au), scrubland (su), floodplain (su au), fern_forest (su au), crags (su au) |
| animals:ginkgo_sapling | ginkgo | 5, 15-26, 36 | Moderate | hardy | - | the game's | plains (su au), forest (su au), birch_forest (su), taiga (su), savanna (sp su au), swamp (su au), tropical_forest (su au wi), mountains (su), ocean (sp su au wi), coral_reef (su au), lagoon (su au), beach (au wi), scrubland (su), floodplain (su au), fern_forest (su au), crags (su au) |
| animals:swamp_cypress_sapling | swamp cypress (Taxodioxylon) | 8, 20-30, 40 | Wet | -15 °C | - | the game's | plains (sp su au), forest (su), birch_forest (su), taiga (su), desert (sp su au wi), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), mountains (su), sequoia_grove (su), ocean (sp su au wi), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au wi), red_dunes (sp su au), scrubland (sp su au), floodplain (sp su au wi), fern_forest (su), crags (sp su au) |
| animals:scale_conifer_sapling | scale-leaved conifer (Cheirolepidiaceae) | 8, 20-32, 42 | Dry | -5 °C | - | the game's | plains (su au), forest (su), birch_forest (su), taiga (su), desert (sp su au wi), savanna (sp su au wi), swamp (sp su au wi), tropical_forest (sp su au wi), mountains (su), sequoia_grove (su), ocean (sp su au wi), coral_reef (sp su au wi), lagoon (sp su au wi), beach (sp su au), red_dunes (sp su au), scrubland (sp su au), floodplain (sp su au), fern_forest (su), crags (sp su au) |

## Crops

What each crop is sown from and gives ripe (`vitals::crops`; how it grows is its plants row): picked unripe it gives back what sowed it, ripe its harvest and, with a seed of its own, up to 3 more seed. Its ripe harvest tires or feeds the farmland under it. Broken tall grass in the biomes listed gives one of their crops' seed a time in 16.

| crop | sown from | ripe gives | stages | soil | wild seed in |
|---|---|---|---|---|---|
| base:wheat | base:wheat_seeds | base:wheat 1-1 | 8 | Tires | tall grass anywhere |
| base:barley | base:barley_seeds | base:barley 1-1 | 4 | Tires | base:taiga, base:mountains, base:plains, base:birch_forest |
| base:rye | base:rye_seeds | base:rye 1-1 | 4 | Tires | base:taiga, base:birch_forest, base:plains, base:mountains |
| base:carrots | base:carrot | base:carrot 1-3 | 4 | Tires | base:plains, base:forest, base:birch_forest, base:taiga, base:mountains |
| base:potatoes | base:potato | base:potato 1-3 | 4 | Tires | base:plains, base:forest, base:birch_forest, base:taiga, base:mountains |
| base:beetroots | base:beetroot | base:beetroot 1-2 | 4 | Tires | base:plains, base:forest, base:birch_forest, base:taiga, base:mountains |
| base:maize | base:maize_seeds | base:corn 1-2 | 4 | Tires | base:plains, base:forest, base:savanna |
| base:paddy_rice | base:rice_seeds | base:rice_seeds 1-3 | 4 | Tires | base:swamp, base:savanna |
| base:beans | base:bean_seeds | base:beans 1-2 | 4 | Feeds | base:plains, base:forest, base:savanna, base:mountains |
| base:tepary_beans | base:tepary_beans | base:tepary_beans 1-3 | 4 | Feeds | base:desert |
| base:sorghum | base:sorghum_seeds | base:sorghum 1-1 | 4 | Tires | base:savanna, base:plains, base:desert |
| base:millet | base:millet_seeds | base:millet 1-1 | 4 | Tires | base:savanna, base:desert |
| base:cassava | base:cassava | base:cassava 1-3 | 4 | Tires | base:savanna, base:swamp |
| base:pumpkin_stem | base:pumpkin_seeds | base:pumpkin_seeds 1-1 | 4 | Tires | base:plains, base:forest, base:savanna |
| base:melon_stem | base:melon_seeds | base:melon_seeds 1-1 | 4 | Tires | base:savanna, base:plains |

## Wild plants

What the world grows besides grass and trees (`vitals::wild`): how each grows, the ground it stands on, what picking it gives, and the biomes it covers that share of its ground in.

| block | grows | ground | gives | cover | found in |
|---|---|---|---|---|---|
| base:sedge | Tuft | Soil | - | 12.0% | snowy_plains, taiga, swamp |
| base:cottongrass | Tuft | Soil | - | 8.0% | snowy_plains, taiga |
| base:reindeer_lichen | Tuft | Soil | - | 5.0% | snowy_plains, taiga, mountains |
| base:clover | Tuft | Soil | - | 3.0% | plains, forest, birch_forest, mountains |
| base:berry_bush | Bush | Soil | base:berries | 0.4% | forest, birch_forest, taiga |
| base:lingonberry_bush | Bush | Soil | base:lingonberries | 1.0% | taiga, birch_forest, mountains |
| base:cloudberry_bush | Bush | Soil | base:cloudberries | 0.6% | snowy_plains, taiga |
| base:crowberry_bush | Bush | Soil | base:crowberries | 1.2% | snowy_plains, taiga, mountains |
| base:hazel_bush | Bush | Soil | base:hazelnut | 0.6% | forest, birch_forest, plains |
| base:bramble_bush | Bush | Soil | base:raspberries | 0.6% | forest, birch_forest, plains, taiga, mountains |
| base:prickly_pear | Ripens | Sand | base:prickly_pear_fruit | 1.0% | desert |
| base:wild_sunflower | Ripens | Soil | base:sunflower_seeds | 1.0% | plains |
| base:wild_rice | Ripens | Shore | base:rice | 10.0% | swamp, savanna |
| base:mushroom_brown | Spreads | Soil | base:mushroom_brown | 0.6% | forest, birch_forest, taiga, mountains |
| base:mushroom_red | Spreads | Soil | base:mushroom_red | 0.3% | forest, birch_forest, taiga |
| base:water_hyacinth | Spreads | Water | - | 8.0% | swamp, savanna |
| base:cattail | Still | Shore | base:cattail_root | 15.0% | swamp, plains, forest, taiga, savanna |
| base:reed | Still | Shore | - | 12.0% | plains, forest, birch_forest, taiga, swamp (4.0%) |
| base:lily_pad | Still | Water | - | 12.0% | plains, forest, birch_forest, swamp, taiga, savanna |
| base:seagrass | Still | Bed | - | 20.0% | ocean, beach |
| base:kelp | Column | Bed | base:kelp | 8.0% | ocean |
| animals:cycad | Still | Soil | animals:cycad_seed | 1.0% | red_dunes, scrubland, floodplain, fern_forest, crags |
| animals:flowering_shrub | Still | Soil | - | 2.0% | floodplain, scrubland |
| animals:water_lily | Still | Water | - | 6.0% | floodplain, red_dunes |
| animals:rotten_log | Log | Soil | - | 0.4% | floodplain, fern_forest |
| animals:termite_mound | Mound | Soil | - | 0.3% | red_dunes, scrubland |
| base:kelp_wrack | Mat | Sand | base:kelp | 3.0% | beach |
| base:dune_grass | Tuft | Sand | - | 10.0% | beach |
| base:beach_plum_bush | Bush | Sand | base:beach_plums | 0.6% | beach |
| base:huckleberry_bush | Bush | Soil | base:huckleberries | 1.0% | mountains |
| base:chia | Ripens | Sand | base:chia_seeds | 2.0% | desert |
| base:dwarf_birch | Still | Soil | - | 2.0% | snowy_plains |
| base:staghorn_coral | Still | Bed | - | 9.0% | coral_reef |
| base:elkhorn_coral | Still | Bed | - | 6.0% | coral_reef |
| base:sea_fan | Still | Bed | - | 5.0% | coral_reef |
| base:finger_coral | Still | Bed | - | 5.0% | coral_reef, lagoon |
| base:sea_rod | Still | Bed | - | 4.0% | coral_reef, lagoon |
| base:turtle_grass | Still | Bed | - | 20.0% | lagoon |

## Food in the world

What foraging animals eat of the world's blocks (`vitals::forage`), how much a mouthful feeds (a capybara holds 200), what eating it leaves, and what brings it back.

| block | kind | food | eaten | grows back as |
|---|---|---|---|---|
| base:tall_grass | grass | 10 | becomes base:grazed_grass | base:grazed_grass |
| base:berry_bush (age 3) | fruit | 8 | back to age 1 | base:berry_bush |
| base:oak_leaves | leaves | 4 | stays | always there in base:forest, base:plains, base:swamp (spring, summer, autumn), worth 40% of fresh growth |
| base:birch_leaves | leaves | 4 | stays | always there in base:forest, base:birch_forest (spring, summer, autumn), worth 40% of fresh growth |
| base:spruce_leaves | leaves | 4 | stays | always there in base:taiga, base:snowy_plains, base:mountains, base:sequoia_grove (all year), worth 40% of fresh growth |
| base:acacia_leaves | leaves | 4 | stays | always there in base:savanna (all year), worth 40% of fresh growth |
| base:cactus | leaves | 3 | stays | always there in base:desert (all year), worth 40% of fresh growth |
| base:tall_grass | seeds | 3 | stays | always there in base:plains, base:forest, base:birch_forest, base:taiga, base:savanna, base:swamp, base:mountains, base:sequoia_grove, base:beach (summer, autumn), worth 40% of fresh growth |
| base:dead_bush | seeds | 2 | stays | always there in base:desert (all year), worth 40% of fresh growth |
| base:cactus | fruit | 3 | stays | always there in base:desert (summer, autumn), worth 40% of fresh growth |
| animals:fern | leaves | 4 | stays | always there in animals:red_dunes, animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (all year), worth 40% of fresh growth |
| base:tall_grass | grass | 3 | stays | always there in base:plains, base:forest, base:birch_forest, base:savanna, base:swamp, base:mountains, base:sequoia_grove, animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (winter), worth 40% of fresh growth |
| base:oak_leaves | leaves | 2 | stays | always there in base:forest, base:plains, base:swamp (winter), worth 40% of fresh growth |
| base:birch_leaves | leaves | 2 | stays | always there in base:forest, base:birch_forest (winter), worth 40% of fresh growth |
| base:oak_leaves | seeds | 3 | stays | always there in base:forest, base:plains, base:swamp (autumn, winter), worth 40% of fresh growth |
| base:snowy_grass | lichen | 3 | stays | always there in base:snowy_plains (all year), worth 40% of fresh growth |
| base:podzol | lichen | 3 | stays | always there in base:taiga (all year), worth 40% of fresh growth |
| base:stone | lichen | 2 | stays | always there in base:mountains (all year), worth 40% of fresh growth |
| base:podzol | fungi | 2 | stays | always there in base:taiga (summer, autumn), worth 40% of fresh growth |
| base:oak_log | fungi | 2 | stays | always there in base:forest, base:plains, base:swamp (summer, autumn), worth 40% of fresh growth |
| base:birch_log | fungi | 2 | stays | always there in base:forest, base:birch_forest (summer, autumn), worth 40% of fresh growth |
| base:grass | roots | 2 | stays | always there in base:plains, base:forest, base:birch_forest, base:savanna, base:swamp, base:mountains, base:sequoia_grove, animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (all year), worth 40% of fresh growth |
| base:sand | roots | 1 | stays | always there in base:desert, base:beach (all year), worth 40% of fresh growth |
| animals:red_sand | roots | 1 | stays | always there in animals:red_dunes (all year), worth 40% of fresh growth |
| base:grass | insects | 2 | stays | always there in base:plains, base:forest, base:birch_forest, base:savanna, base:swamp, base:mountains, base:sequoia_grove, animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (spring, summer, autumn), worth 40% of fresh growth |
| base:sand | insects | 2 | stays | always there in base:desert, base:beach (all year), worth 40% of fresh growth |
| animals:red_sand | insects | 2 | stays | always there in animals:red_dunes (all year), worth 40% of fresh growth |
| base:podzol | insects | 2 | stays | always there in base:taiga (summer), worth 40% of fresh growth |
| base:snowy_grass | insects | 2 | stays | always there in base:snowy_plains (summer), worth 40% of fresh growth |
| base:sugar_cane | water_plants | 4 | stays | always there in base:plains, base:forest, base:birch_forest, base:savanna, base:swamp, base:beach (all year), worth 40% of fresh growth |
| animals:horsetail | water_plants | 4 | stays | always there in animals:red_dunes, animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (all year), worth 40% of fresh growth |
| base:sedge (age 1) | grass | 8 | back to age 0 | base:sedge |
| base:cottongrass (age 1) | grass | 6 | back to age 0 | base:cottongrass |
| base:reindeer_lichen (age 1) | lichen | 6 | back to age 0 | base:reindeer_lichen |
| base:clover (age 1) | leaves | 8 | back to age 0 | base:clover |
| base:lingonberry_bush (age 3) | fruit | 6 | back to age 1 | base:lingonberry_bush |
| base:cloudberry_bush (age 3) | fruit | 6 | back to age 1 | base:cloudberry_bush |
| base:crowberry_bush (age 3) | fruit | 6 | back to age 1 | base:crowberry_bush |
| base:hazel_bush (age 3) | seeds | 8 | back to age 1 | base:hazel_bush |
| base:bramble_bush (age 3) | fruit | 8 | back to age 1 | base:bramble_bush |
| base:prickly_pear (age 1) | fruit | 6 | back to age 0 | base:prickly_pear |
| base:wild_sunflower (age 1) | seeds | 6 | back to age 0 | base:wild_sunflower |
| base:wild_rice (age 1) | seeds | 6 | back to age 0 | base:wild_rice |
| base:prickly_pear | leaves | 4 | stays | always there in base:desert (all year), worth 40% of fresh growth |
| base:wild_rice | grass | 4 | stays | always there in base:swamp, base:savanna (spring, summer, autumn), worth 40% of fresh growth |
| base:crowberry_bush | fruit | 2 | stays | always there in base:snowy_plains, base:taiga, base:mountains (winter), worth 40% of fresh growth |
| base:lingonberry_bush | fruit | 2 | stays | always there in base:taiga, base:birch_forest, base:mountains (winter), worth 40% of fresh growth |
| base:mushroom_brown | fungi | 5 | stays | always there in base:forest, base:birch_forest, base:taiga, base:mountains (summer, autumn), worth 40% of fresh growth |
| base:water_hyacinth | water_plants | 6 | stays | always there in base:swamp, base:savanna (all year), worth 40% of fresh growth |
| base:cattail | water_plants | 5 | stays | always there in base:swamp, base:plains, base:forest, base:taiga, base:savanna (spring, summer, autumn), worth 40% of fresh growth |
| animals:cycad | leaves | 3 | stays | always there in animals:red_dunes, animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (all year), worth 40% of fresh growth |
| animals:cycad | seeds | 3 | stays | always there in animals:red_dunes, animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (summer, autumn), worth 40% of fresh growth |
| animals:flowering_shrub | leaves | 4 | stays | always there in animals:floodplain, animals:scrubland (spring, summer, autumn), worth 40% of fresh growth |
| animals:water_lily | water_plants | 5 | stays | always there in animals:floodplain, animals:red_dunes (all year), worth 40% of fresh growth |
| base:kelp_wrack | water_plants | 4 | stays | always there in base:beach (all year), worth 40% of fresh growth |
| base:turtle_grass | water_plants | 5 | stays | always there in base:lagoon (all year), worth 40% of fresh growth |
| animals:rotten_log | insects | 3 | stays | always there in animals:floodplain, animals:fern_forest (all year), worth 40% of fresh growth |
| base:dune_grass (age 1) | grass | 4 | back to age 0 | base:dune_grass |
| base:beach_plum_bush (age 3) | fruit | 6 | back to age 1 | base:beach_plum_bush |
| base:huckleberry_bush (age 3) | fruit | 8 | back to age 1 | base:huckleberry_bush |
| base:chia (age 1) | seeds | 6 | back to age 0 | base:chia |
| base:dwarf_birch | leaves | 4 | stays | always there in base:snowy_plains (spring, summer, autumn), worth 40% of fresh growth |
| base:dwarf_birch | leaves | 2 | stays | always there in base:snowy_plains (winter), worth 40% of fresh growth |
| animals:termite_mound | insects | 3 | stays | always there in animals:red_dunes, animals:scrubland (all year), worth 40% of fresh growth |
| animals:termite_mound_upper | insects | 3 | stays | always there in animals:red_dunes, animals:scrubland (all year), worth 40% of fresh growth |
| base:mesquite_leaves | leaves | 4 | stays | always there in base:desert, base:savanna (spring, summer, autumn), worth 40% of fresh growth |
| base:mesquite_leaves | seeds | 4 | stays | always there in base:desert, base:savanna (summer, autumn), worth 40% of fresh growth |
| base:acuri_palm_leaves | seeds | 5 | stays | always there in base:swamp (autumn, winter), worth 40% of fresh growth |
| base:moriche_palm_leaves | fruit | 5 | stays | always there in base:savanna, base:swamp (spring, summer), worth 40% of fresh growth |
| animals:araucaria_leaves | leaves | 5 | stays | always there in animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (all year), worth 40% of fresh growth |
| animals:araucaria_leaves | seeds | 4 | stays | always there in animals:scrubland, animals:floodplain, animals:fern_forest, animals:crags (autumn), worth 40% of fresh growth |
| animals:ginkgo_leaves | leaves | 4 | stays | always there in animals:fern_forest (spring, summer, autumn), worth 40% of fresh growth |
| animals:ginkgo_leaves | fruit | 4 | stays | always there in animals:fern_forest (autumn), worth 40% of fresh growth |
| animals:swamp_cypress_leaves | leaves | 3 | stays | always there in animals:floodplain (spring, summer, autumn), worth 40% of fresh growth |
| animals:scale_conifer_leaves | leaves | 2 | stays | always there in animals:red_dunes, animals:scrubland, animals:crags (all year), worth 40% of fresh growth |
| animals:tempskya_leaves | leaves | 2 | stays | always there in animals:floodplain, animals:fern_forest (all year), worth 40% of fresh growth |

On the ground, animals eat dropped food of their diet one item a mouthful: every food (the foods table, by its kind and portion) and these seeds; spoiled food only carrion eaters. An item fills an animal by its weight against a meal of a tenth of the animal's mass (`animals_core::needs`).

| item | kind | grams |
|---|---|---|
| base:wheat_seeds | seeds | 10 |
| base:wheat | seeds | 50 |
| base:acorn | seeds | 20 |
| base:rice | seeds | 30 |
| animals:cycad_seed | seeds | 20 |
| base:mesquite_pod | seeds | 30 |
| base:acuri_nut | seeds | 30 |
| base:moriche_fruit | fruit | 50 |
| base:brazil_nut | seeds | 30 |
| base:cacao_pod | fruit | 40 |
| base:andiroba_seed | seeds | 20 |
| base:matamata_fruit | seeds | 20 |
| animals:araucaria_cone | seeds | 50 |
| animals:ginkgo_seed | fruit | 10 |

## Soil and compost

Farmland's fertility (`vitals::soil`): how fast crops grow at each level, what wears it, what gives it back; and what a compost bin takes.

| fertility | crops grow |
|---|---|
| 3 | 100% |
| 2 | 73% |
| 1 | 47% |
| 0 | 20% |

A ripe harvest of base:wheat, base:barley, base:rye, base:carrots, base:potatoes, base:beetroots, base:maize, base:paddy_rice, base:sorghum, base:millet, base:cassava, base:pumpkin_stem, base:melon_stem takes a level, of base:beans, base:tepary_beans gives one back; base:compost +2, base:dung +1; a season unplanted (about 14 game days, at grass's rate) gives one back. A compost bin fills in 7 steps: spoiled food 2, base:dung 2, base:rotten_flesh 2, base:oak_leaves 1, base:birch_leaves 1, base:spruce_leaves 1, base:acacia_leaves 1, base:sapling 1, base:wheat_seeds 1, base:wheat 1; full, it rots into compost in about 23 minutes.

## Difficulty (the `base:difficulty` rule)

| | digests | keeps | harm | can kill | XP lost on death | stamina costs | first nights without monsters |
|---|---|---|---|---|---|---|---|
| gentle | x0.75 | x1.5 | x0.5 | no, stops at a tenth | 10% | x0.8 | 3 |
| normal | x1 | x1 | x1 | yes | 25% | x1 | 0 |
| harsh | x1.25 | x0.6 | x1.5 | yes | 50% | x1.15 | 0 |

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
