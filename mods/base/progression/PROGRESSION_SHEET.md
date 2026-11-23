# Progression and balance sheet

Written from `mods/base/progression` (`cargo run -p progression --bin sheet > mods/base/progression/PROGRESSION_SHEET.md`); do not edit by hand. The plan: `notes/PROGRESSION.md`; the method: `reports/Balance model.md`; the numbers: `vitals::curve`, `vitals::combat`, `vitals::web`, `animals_core::danger`. Nothing in the game reads the curves yet: the tables show today's numbers next to what the curves would set.

## Checks

| check | result |
|---|---|
| every item and block tagged, every made one with a recipe | passes |
| every recipe in the mods' sources declared | passes |
| everything reachable from the start | passes |
| nothing at tier N needs anything above N | passes |
| each age's key by each declared route | passes |
| path parity: each age's key by each route within ×1.15 of its cheapest, in EM | passes |
| every tool-part material tagged, made from its tier or below | passes |
| each age's making inside its budget | passes |
| every size and role inside its bands, floors hold | passes |
| every species inside its bands at every level, floors hold | passes |
| tames capped by their saddle and ceiling | passes |
| tweaks inside ×0.75-×1.25 with a reason | passes |
| grade and gems at most +60% over a plain item | passes |
| stagger: same-age fights break every part, knockdowns at most a third of a fight | passes |
| bows on the curve: full draw a shot a second, a reference bow's arrows the reference's damage | passes |
| saddles lift tames by their fitting, as the tame model does, tooltip in step | passes |
| loot per hour: kill drops priced from their cheapest kill, no kill paying over its time (+30 %, +50 % more for what fights back) | passes |

## The ages

Power grows ×1.6 an age: `g(L) = 1.6^(L/10)` over area and item level L (tier × 10 + a depth of 0-9). The currency is effort minutes (EM). The reference player is same-age gear at grade 4, measured at the age's middle (depth 5). The kit is what an average player makes in the age the first time (`graph::KITS`), gathered and crafted the cheapest way, against its budget: 40 % of the age's EM (`curve::MAKING_SHARE`). Everything once is one of every made thing tagged to the age, alternatives and all, each from scratch.

| tier | age | hours | EM | g at start | reference: hit / health / armour | things tagged | made | kit EM | of budget | everything once |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | Stone Age | 6 | 360 | 1.00 | 7.6 / 25 / 32 | 550 | 159 | 48 | 33 % | 217 EM |
| 1 | Bronze Age | 12 | 720 | 1.60 | 12.1 / 40 / 51 | 52 | 35 | 150 | 52 % | 143 EM |
| 2 | Iron Age | 16 | 960 | 2.56 | 19.4 / 65 / 81 | 58 | 28 | 61 | 16 % | 270 EM |
| 3 | Age of Steam | 18 | 1080 | 4.10 | 31.1 / 104 / 130 | 38 | 32 | 94 | 22 % | 328 EM |
| 4 | Age of Lightning | 20 | 1200 | 6.55 | 49.7 / 166 / 207 | 0 | 0 | 0 | 0 % | 0 EM |
| 5 | Age of the Future | 18 | 1080 | 10.49 | 79.6 / 265 / 332 | 0 | 0 | 0 | 0 % | 0 EM |
| 6 | Age of Stars | 16 | 960 | 16.78 | 127.3 / 424 / 531 | 0 | 0 | 0 | 0 % | 0 EM |

106 hours in all.

## Fight bands and derived stats

For a same-age reference player (Ronald, 11-06: "as proposed"). Health so the reference kills it in the band's middle, armour so it cuts the reference's hit by its share (`armour / (armour + 5 × hit)`, capped at 80 %), a hit so it downs the reference in the band's hits. The cap is how many levels over it a player's gear counts at most; the floor the least seconds it dies in, whatever the gear (damage over it goes to stagger and part breaks).

| size | mass | TTK band (mid) | hits to down | hit every | armour cut | gear cap | floor | knockback resist |
|---|---|---|---|---|---|---|---|---|
| small | under 10 kg | 0-2 s (1) | 25 | 1 s | 0 % | none | none | 0 |
| medium | 10-150 kg | 3-8 s (5) | 8 | 1.2 s | 10 % | +30 | none | 0.3 |
| large | 150-2,000 kg | 20-40 s (30) | 4 | 2 s | 25 % | +20 | 10 s | 0.7 |
| huge | 2-20 t | 60-120 s (90) | 2.5 | 3 s | 35 % | +10 | 45 s | 1 |
| titan | over 20 t, or a boss | 120-300 s (210) | 1.5 | 4 s | 45 % | +5 | 120 s | 1 |

Roles shape the budget: prey health ×0.7, hit ×0.5, armour ×1; grazer health ×1, hit ×0.8, armour ×1; hunter health ×0.9, hit ×1.2, armour ×1; apex health ×1.3, hit ×1.3, armour ×1; tank health ×1, hit ×0.7, armour ×1.6.

A size's own health / armour / hit (before its role) at each age's reference level, engine points (the vitals show health ×5):

| size | Stone Age | Bronze Age | Iron Age | Age of Steam | Age of Lightning | Age of the Future | Age of Stars |
|---|---|---|---|---|---|---|---|
| small | 8 / 0 / 3 | 12 / 0 / 5 | 19 / 0 / 8 | 31 / 0 / 13 | 50 / 0 / 20 | 80 / 0 / 32 | 127 / 0 / 52 |
| medium | 34 / 4 / 6 | 55 / 7 / 10 | 87 / 11 / 16 | 140 / 17 / 26 | 224 / 28 / 41 | 358 / 44 / 66 | 573 / 71 / 106 |
| large | 171 / 13 / 10 | 273 / 20 / 16 | 437 / 32 / 26 | 699 / 52 / 42 | 1119 / 83 / 67 | 1791 / 133 / 107 | 2865 / 212 / 172 |
| huge | 444 / 20 / 15 | 710 / 33 / 23 | 1137 / 52 / 37 | 1819 / 84 / 59 | 2910 / 134 / 95 | 4656 / 214 / 152 | 7449 / 343 / 244 |
| titan | 877 / 31 / 22 | 1403 / 50 / 35 | 2244 / 79 / 56 | 3590 / 127 / 89 | 5745 / 203 / 143 | 9192 / 326 / 228 | 14707 / 521 / 365 |

## Over-tier and under-tier

Seconds to kill and the creature's hits to down the player, for a size's grazer at the Iron Age's reference level, by how many ages the player's gear is over or under it (grade 4). Over-tier gear counts only up to the size's cap, and the floor holds.

| size | -1 age | same age | +1 | +2 | +3 | +4 |
|---|---|---|---|---|---|---|
| small | 1.6 s / 16.7 | 1.0 s / 36.5 | 0.6 s / 82.0 | 0.4 s / 131.2 | 0.2 s / 209.9 | 0.2 s / 335.8 |
| medium | 8.5 s / 5.6 | 5.0 s / 11.2 | 3.0 s / 24.0 | 1.8 s / 53.8 | 1.1 s / 102.4 | 1.1 s / 102.4 |
| large | 55.2 s / 2.9 | 30.0 s / 5.5 | 17.0 s / 11.1 | 10.0 s / 23.6 | 10.0 s / 23.6 | 10.0 s / 23.6 |
| huge | 174.2 s / 1.8 | 90.0 s / 3.4 | 48.9 s / 6.5 | 48.9 s / 6.5 | 48.9 s / 6.5 | 48.9 s / 6.5 |
| titan | 426.7 s / 1.1 | 210.0 s / 2.0 | 150.4 s / 2.7 | 150.4 s / 2.7 | 150.4 s / 2.7 | 150.4 s / 2.7 |

## Species

Every wild animal rolls a level of 0-200 anywhere, and a level-200 one is 5× a level-0 one of its species (Ronald, 11-06): a wild point in health or damage adds 0.18 of the base. Its size class comes from its mass (small ones scale inside their class by the cube root of mass against 5 kg, ×0.2-×1.3: a fox beats a mouse), its role and tweaks from `animals_core::danger`. The game registers each kind with these level-0 numbers (11-07). A Stone Age species reaches the Bronze Age's band at level 30, the Iron Age's at 78, Steam's at 155.

| species | kg | size | role | mass × | health / armour / hit (level 0) | health at 100 / 200 | tweaks |
|---|---|---|---|---|---|---|---|
| dog | 2.5 | small | hunter | 0.79 | 4.3 / 0.0 / 2.3 | 13 / 21 |  |
| cat | 4.5 | small | hunter | 0.97 | 5.2 / 0.0 / 2.8 | 16 / 26 |  |
| capybara | 50 | medium | prey | 1.00 | 18.9 / 3.3 / 2.5 | 57 / 94 |  |
| raptor | 15 | medium | hunter | 1.00 | 20.7 / 3.3 / 6.0 | 62 / 103 | health ×0.85: hunts in packs of three to five; one alone is softer |
| utahraptor | 300 | large | hunter | 1.00 | 103.3 / 10.0 / 9.7 | 310 / 516 | health ×0.85: hunts in packs (the Stikes Quarry group); one alone is softer |
| osmolskae | 15 | medium | hunter | 1.00 | 20.7 / 3.3 / 6.0 | 62 / 103 | health ×0.85: hunts in packs of three to five; one alone is softer |
| wyvern | 5000 | huge | apex | 1.00 | 456.3 / 16.2 / 14.9 | 1369 / 2281 |  |
| hare | 3.8 | small | prey | 0.91 | 3.8 / 0.0 / 1.1 | 11 / 19 |  |
| mountain_hare | 3 | small | prey | 0.84 | 3.5 / 0.0 / 1.0 | 11 / 18 |  |
| wood_mouse | 0.023 | small | prey | 0.20 | 0.8 / 0.0 / 0.2 | 3 / 4 |  |
| field_vole | 0.035 | small | prey | 0.20 | 0.8 / 0.0 / 0.2 | 3 / 4 |  |
| white_tailed_deer | 70 | medium | prey | 1.00 | 18.9 / 3.3 / 2.5 | 57 / 94 |  |
| red_deer | 160 | large | prey | 1.00 | 94.5 / 10.0 / 4.0 | 284 / 472 |  |
| brown_trout | 0.5 | small | prey | 0.46 | 1.9 / 0.0 / 0.6 | 6 / 10 |  |
| pacu | 4 | small | prey | 0.93 | 3.9 / 0.0 / 1.1 | 12 / 19 |  |
| red_fox | 5.5 | small | hunter | 1.03 | 5.6 / 0.0 / 3.0 | 17 / 28 |  |
| arctic_fox | 3.5 | small | hunter | 0.89 | 4.8 / 0.0 / 2.6 | 14 / 24 |  |
| raven | 1.2 | small | prey | 0.62 | 2.6 / 0.0 / 0.8 | 8 / 13 |  |
| protoceratops | 83 | medium | grazer | 1.00 | 27.0 / 3.3 / 4.0 | 81 / 135 |  |
| kryptobaatar | 0.04 | small | prey | 0.20 | 0.8 / 0.0 / 0.2 | 3 / 4 |  |
| priscagama | 0.03 | small | prey | 0.20 | 0.8 / 0.0 / 0.2 | 3 / 4 |  |
| reindeer | 120 | medium | grazer | 1.00 | 27.0 / 3.3 / 4.0 | 81 / 135 |  |
| coyote | 13 | medium | hunter | 1.00 | 24.3 / 3.3 / 6.0 | 73 / 122 |  |
| house_sparrow | 0.03 | small | prey | 0.20 | 0.8 / 0.0 / 0.2 | 3 / 4 |  |
| snow_bunting | 0.035 | small | prey | 0.20 | 0.8 / 0.0 / 0.2 | 3 / 4 |  |
| great_horned_owl | 1.4 | small | hunter | 0.65 | 3.5 / 0.0 / 1.9 | 11 / 18 |  |
| snowy_owl | 2 | small | hunter | 0.74 | 4.0 / 0.0 / 2.2 | 12 / 20 |  |
| golden_eagle | 4.5 | small | hunter | 0.97 | 5.2 / 0.0 / 2.8 | 16 / 26 |  |
| savanna_hawk | 0.85 | small | hunter | 0.55 | 3.0 / 0.0 / 1.6 | 9 / 15 |  |
| turkey_vulture | 1.4 | small | prey | 0.65 | 2.7 / 0.0 / 0.8 | 8 / 14 |  |
| herring_gull | 1 | small | prey | 0.58 | 2.5 / 0.0 / 0.7 | 7 / 12 |  |
| yacare_caiman | 35 | medium | hunter | 1.00 | 24.3 / 3.3 / 6.0 | 73 / 122 |  |
| spectacled_caiman | 25 | medium | hunter | 1.00 | 24.3 / 3.3 / 6.0 | 73 / 122 |  |
| grey_wolf | 42 | medium | hunter | 1.00 | 24.3 / 3.3 / 6.0 | 73 / 122 |  |
| jaguar | 85 | medium | apex | 1.00 | 35.1 / 3.3 / 6.5 | 105 / 176 |  |
| lungfish | 3 | small | prey | 0.84 | 3.5 / 0.0 / 1.0 | 11 / 18 |  |
| lycoptera | 0.02 | small | prey | 0.20 | 0.8 / 0.0 / 0.2 | 3 / 4 |  |
| psittacosaurus | 25 | medium | grazer | 1.00 | 27.0 / 3.3 / 4.0 | 81 / 135 |  |
| gastonia | 1900 | large | tank | 1.00 | 135.0 / 16.0 / 5.7 | 405 / 675 |  |
| pinacosaurus | 1900 | large | tank | 1.00 | 135.0 / 16.0 / 5.7 | 405 / 675 |  |
| iguanodont | 4000 | huge | grazer | 1.00 | 351.0 / 16.2 / 9.2 | 1053 / 1755 |  |
| crocodyliform | 150 | large | hunter | 1.00 | 121.5 / 10.0 / 9.7 | 365 / 608 |  |
| citipati | 90 | medium | grazer | 1.00 | 27.0 / 3.3 / 4.0 | 81 / 135 |  |
| confuciusornis | 0.5 | small | prey | 0.46 | 1.9 / 0.0 / 0.6 | 6 / 10 |  |
| gobipteryx | 0.3 | small | prey | 0.39 | 1.6 / 0.0 / 0.5 | 5 / 8 |  |
| cedarosaurus | 10000 | huge | grazer | 1.00 | 351.0 / 16.2 / 11.0 | 1053 / 1755 | hit ×1.2: a tail and a stamp: a sauropod's defence is its weight |

## The combat matrix

Each species at level 0 (the Stone Age's start) against a reference player of each age, grade 4: seconds to kill / its hits to down the player. The checks run it at every tenth level, at grade 4 and at the +60 % ceiling.

| species | Stone Age | Bronze Age | Iron Age | Age of Steam | Age of Lightning | Age of the Future | Age of Stars |
|---|---|---|---|---|---|---|---|
| dog | 0.6 / 41 | 0.4 / 87 | 0.2 / 139 | 0.1 / 223 | 0.1 / 357 | 0.1 / 571 | 0.0 / 913 |
| cat | 0.7 / 29 | 0.4 / 66 | 0.3 / 115 | 0.2 / 183 | 0.1 / 293 | 0.1 / 469 | 0.0 / 751 |
| capybara | 2.7 / 36 | 1.6 / 81 | 1.0 / 130 | 0.8 / 164 | 0.8 / 164 | 0.8 / 164 | 0.8 / 164 |
| raptor | 3.0 / 9 | 1.8 / 18 | 1.1 / 40 | 0.9 / 60 | 0.9 / 60 | 0.9 / 60 | 0.9 / 60 |
| utahraptor | 17.2 / 4 | 10.0 / 9 | 10.0 / 12 | 10.0 / 12 | 10.0 / 12 | 10.0 / 12 | 10.0 / 12 |
| osmolskae | 3.0 / 9 | 1.8 / 18 | 1.1 / 40 | 0.9 / 60 | 0.9 / 60 | 0.9 / 60 | 0.9 / 60 |
| wyvern | 85.7 / 2 | 63.5 / 3 | 63.5 / 3 | 63.5 / 3 | 63.5 / 3 | 63.5 / 3 | 63.5 / 3 |
| hare | 0.5 / 114 | 0.3 / 182 | 0.2 / 291 | 0.1 / 465 | 0.1 / 745 | 0.0 / 1192 | 0.0 / 1906 |
| mountain_hare | 0.5 / 123 | 0.3 / 197 | 0.2 / 315 | 0.1 / 504 | 0.1 / 806 | 0.0 / 1289 | 0.0 / 2063 |
| wood_mouse | 0.1 / 518 | 0.1 / 830 | 0.0 / 1327 | 0.0 / 2124 | 0.0 / 3398 | 0.0 / 5437 | 0.0 / 8699 |
| field_vole | 0.1 / 518 | 0.1 / 830 | 0.0 / 1327 | 0.0 / 2124 | 0.0 / 3398 | 0.0 / 5437 | 0.0 / 8699 |
| white_tailed_deer | 2.7 / 36 | 1.6 / 81 | 1.0 / 130 | 0.8 / 164 | 0.8 / 164 | 0.8 / 164 | 0.8 / 164 |
| red_deer | 15.7 / 16 | 10.0 / 35 | 10.0 / 53 | 10.0 / 53 | 10.0 / 53 | 10.0 / 53 | 10.0 / 53 |
| brown_trout | 0.3 / 223 | 0.2 / 357 | 0.1 / 572 | 0.1 / 915 | 0.0 / 1464 | 0.0 / 2343 | 0.0 / 3748 |
| pacu | 0.5 / 112 | 0.3 / 179 | 0.2 / 286 | 0.1 / 458 | 0.1 / 732 | 0.0 / 1171 | 0.0 / 1874 |
| red_fox | 0.7 / 26 | 0.5 / 58 | 0.3 / 107 | 0.2 / 171 | 0.1 / 274 | 0.1 / 439 | 0.0 / 702 |
| arctic_fox | 0.6 / 33 | 0.4 / 76 | 0.2 / 125 | 0.2 / 199 | 0.1 / 319 | 0.1 / 510 | 0.0 / 816 |
| raven | 0.3 / 167 | 0.2 / 267 | 0.1 / 427 | 0.1 / 683 | 0.1 / 1094 | 0.0 / 1750 | 0.0 / 2800 |
| protoceratops | 3.9 / 16 | 2.3 / 36 | 1.4 / 81 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 |
| kryptobaatar | 0.1 / 518 | 0.1 / 830 | 0.0 / 1327 | 0.0 / 2124 | 0.0 / 3398 | 0.0 / 5437 | 0.0 / 8699 |
| priscagama | 0.1 / 518 | 0.1 / 830 | 0.0 / 1327 | 0.0 / 2124 | 0.0 / 3398 | 0.0 / 5437 | 0.0 / 8699 |
| reindeer | 3.9 / 16 | 2.3 / 36 | 1.4 / 81 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 |
| coyote | 3.5 / 9 | 2.1 / 18 | 1.3 / 40 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 |
| house_sparrow | 0.1 / 518 | 0.1 / 830 | 0.0 / 1327 | 0.0 / 2124 | 0.0 / 3398 | 0.0 / 5437 | 0.0 / 8699 |
| snow_bunting | 0.1 / 518 | 0.1 / 830 | 0.0 / 1327 | 0.0 / 2124 | 0.0 / 3398 | 0.0 / 5437 | 0.0 / 8699 |
| great_horned_owl | 0.5 / 57 | 0.3 / 106 | 0.2 / 169 | 0.1 / 271 | 0.1 / 433 | 0.0 / 693 | 0.0 / 1108 |
| snowy_owl | 0.5 / 46 | 0.3 / 94 | 0.2 / 150 | 0.1 / 240 | 0.1 / 384 | 0.0 / 615 | 0.0 / 984 |
| golden_eagle | 0.7 / 29 | 0.4 / 66 | 0.3 / 115 | 0.2 / 183 | 0.1 / 293 | 0.1 / 469 | 0.0 / 751 |
| savanna_hawk | 0.4 / 76 | 0.2 / 125 | 0.2 / 200 | 0.1 / 319 | 0.1 / 511 | 0.0 / 818 | 0.0 / 1309 |
| turkey_vulture | 0.4 / 159 | 0.2 / 254 | 0.1 / 406 | 0.1 / 649 | 0.1 / 1039 | 0.0 / 1662 | 0.0 / 2659 |
| herring_gull | 0.3 / 177 | 0.2 / 284 | 0.1 / 454 | 0.1 / 726 | 0.0 / 1162 | 0.0 / 1859 | 0.0 / 2975 |
| yacare_caiman | 3.5 / 9 | 2.1 / 18 | 1.3 / 40 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 |
| spectacled_caiman | 3.5 / 9 | 2.1 / 18 | 1.3 / 40 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 |
| grey_wolf | 3.5 / 9 | 2.1 / 18 | 1.3 / 40 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 |
| jaguar | 5.0 / 8 | 3.0 / 16 | 1.9 / 35 | 1.5 / 52 | 1.5 / 52 | 1.5 / 52 | 1.5 / 52 |
| lungfish | 0.5 / 123 | 0.3 / 197 | 0.2 / 315 | 0.1 / 504 | 0.1 / 806 | 0.0 / 1289 | 0.0 / 2063 |
| lycoptera | 0.1 / 518 | 0.1 / 830 | 0.0 / 1327 | 0.0 / 2124 | 0.0 / 3398 | 0.0 / 5437 | 0.0 / 8699 |
| psittacosaurus | 3.9 / 16 | 2.3 / 36 | 1.4 / 81 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 |
| gastonia | 25.3 / 9 | 14.0 / 20 | 10.6 / 29 | 10.6 / 29 | 10.6 / 29 | 10.6 / 29 | 10.6 / 29 |
| pinacosaurus | 25.3 / 9 | 14.0 / 20 | 10.6 / 29 | 10.6 / 29 | 10.6 / 29 | 10.6 / 29 | 10.6 / 29 |
| iguanodont | 65.9 / 5 | 48.9 / 7 | 48.9 / 7 | 48.9 / 7 | 48.9 / 7 | 48.9 / 7 | 48.9 / 7 |
| crocodyliform | 20.2 / 4 | 11.7 / 9 | 10.0 / 12 | 10.0 / 12 | 10.0 / 12 | 10.0 / 12 | 10.0 / 12 |
| citipati | 3.9 / 16 | 2.3 / 36 | 1.4 / 81 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 |
| confuciusornis | 0.3 / 223 | 0.2 / 357 | 0.1 / 572 | 0.1 / 915 | 0.0 / 1464 | 0.0 / 2343 | 0.0 / 3748 |
| gobipteryx | 0.2 / 265 | 0.1 / 424 | 0.1 / 678 | 0.1 / 1085 | 0.0 / 1736 | 0.0 / 2777 | 0.0 / 4444 |
| cedarosaurus | 65.9 / 4 | 48.9 / 5 | 48.9 / 5 | 48.9 / 5 | 48.9 / 5 | 48.9 / 5 | 48.9 / 5 |

## Weapons, tools and armour

From the curve (`vitals::gear`, Ronald, 11-07: "Yes, from the curves"): a kind swings at its speed and deals its share of the reference's damage a second at its material's item level (tier × 10 + depth). A sword of the age is the reference. A full armour set of an age gives the reference's armour and lifts health to the reference's (20 engine points at its base).

| kind | swings a second | share |
|---|---|---|
| sword | 1.6 | 1 |
| spear | 1.2 | 1 |
| axe | 0.9 | 0.9 |
| knife | 2 | 0.7 |
| club | 1 | 0.6 |
| pickaxe | 1.2 | 0.5 |
| shovel | 1 | 0.4 |
| hoe | 2 | 0.3 |
| bow | 1 | 1 |

One hit's damage (engine points) by material:

| material | age | level | sword | axe | spear | knife | pickaxe | shovel | hoe | full armour set: armour / health |
|---|---|---|---|---|---|---|---|---|---|---|
| wood | Stone Age | 0 | 3.8 | 6.0 | 5.0 | 2.1 | 2.5 | 2.4 | 0.9 | 25 / 20 |
| branch | Stone Age | 0 | 3.8 | 6.0 | 5.0 | 2.1 | 2.5 | 2.4 | 0.9 | 25 / 20 |
| oak | Stone Age | 1 | 3.9 | 6.3 | 5.2 | 2.2 | 2.6 | 2.5 | 0.9 | 26 / 21 |
| mesquite | Stone Age | 1 | 3.9 | 6.3 | 5.2 | 2.2 | 2.6 | 2.5 | 0.9 | 26 / 21 |
| birch | Stone Age | 1 | 3.9 | 6.3 | 5.2 | 2.2 | 2.6 | 2.5 | 0.9 | 26 / 21 |
| spruce | Stone Age | 1 | 3.9 | 6.3 | 5.2 | 2.2 | 2.6 | 2.5 | 0.9 | 26 / 21 |
| yew | Stone Age | 1 | 3.9 | 6.3 | 5.2 | 2.2 | 2.6 | 2.5 | 0.9 | 26 / 21 |
| elm | Stone Age | 1 | 3.9 | 6.3 | 5.2 | 2.2 | 2.6 | 2.5 | 0.9 | 26 / 21 |
| osage_orange | Stone Age | 1 | 3.9 | 6.3 | 5.2 | 2.2 | 2.6 | 2.5 | 0.9 | 26 / 21 |
| bone | Stone Age | 3 | 4.3 | 6.9 | 5.8 | 2.4 | 2.9 | 2.8 | 1.0 | 29 / 23 |
| antler | Stone Age | 3 | 4.3 | 6.9 | 5.8 | 2.4 | 2.9 | 2.8 | 1.0 | 29 / 23 |
| chert | Stone Age | 5 | 4.7 | 7.6 | 6.3 | 2.7 | 3.2 | 3.0 | 1.1 | 32 / 25 |
| flint | Stone Age | 5 | 4.7 | 7.6 | 6.3 | 2.7 | 3.2 | 3.0 | 1.1 | 32 / 25 |
| stone | Stone Age | 5 | 4.7 | 7.6 | 6.3 | 2.7 | 3.2 | 3.0 | 1.1 | 32 / 25 |
| tektite | Stone Age | 6 | 5.0 | 8.0 | 6.6 | 2.8 | 3.3 | 3.2 | 1.2 | 33 / 27 |
| obsidian | Stone Age | 6 | 5.0 | 8.0 | 6.6 | 2.8 | 3.3 | 3.2 | 1.2 | 33 / 27 |
| plant_cordage | Stone Age | 2 | 4.1 | 6.6 | 5.5 | 2.3 | 2.7 | 2.6 | 1.0 | 27 / 22 |
| sinew | Stone Age | 3 | 4.3 | 6.9 | 5.8 | 2.4 | 2.9 | 2.8 | 1.0 | 29 / 23 |
| rawhide | Stone Age | 3 | 4.3 | 6.9 | 5.8 | 2.4 | 2.9 | 2.8 | 1.0 | 29 / 23 |
| birch_tar | Stone Age | 4 | 4.5 | 7.2 | 6.0 | 2.5 | 3.0 | 2.9 | 1.1 | 30 / 24 |
| leather | Stone Age | 5 | 4.7 | 7.6 | 6.3 | 2.7 | 3.2 | 3.0 | 1.1 | 32 / 25 |
| linen | Stone Age | 3 | 4.3 | 6.9 | 5.8 | 2.4 | 2.9 | 2.8 | 1.0 | 29 / 23 |
| wool | Stone Age | 3 | 4.3 | 6.9 | 5.8 | 2.4 | 2.9 | 2.8 | 1.0 | 29 / 23 |
| fur | Stone Age | 4 | 4.5 | 7.2 | 6.0 | 2.5 | 3.0 | 2.9 | 1.1 | 30 / 24 |
| copper | Bronze Age | 12 | 6.6 | 10.5 | 8.8 | 3.7 | 4.4 | 4.2 | 1.6 | 44 / 35 |
| gold | Bronze Age | 13 | 6.9 | 11.1 | 9.2 | 3.9 | 4.6 | 4.4 | 1.7 | 46 / 37 |
| bronze | Bronze Age | 16 | 8.0 | 12.7 | 10.6 | 4.5 | 5.3 | 5.1 | 1.9 | 53 / 42 |
| meteoric_iron | Bronze Age | 18 | 8.7 | 14.0 | 11.7 | 4.9 | 5.8 | 5.6 | 2.1 | 58 / 47 |
| wrought_iron | Iron Age | 23 | 11.1 | 17.7 | 14.7 | 6.2 | 7.4 | 7.1 | 2.7 | 74 / 59 |
| iron | Iron Age | 25 | 12.1 | 19.4 | 16.2 | 6.8 | 8.1 | 7.8 | 2.9 | 81 / 65 |
| steel | Iron Age | 27 | 13.3 | 21.3 | 17.8 | 7.5 | 8.9 | 8.5 | 3.2 | 89 / 71 |
| star_steel | Iron Age | 29 | 14.7 | 23.4 | 19.5 | 8.2 | 9.8 | 9.4 | 3.5 | 98 / 78 |
| diamond | Age of Steam | 35 | 19.4 | 31.1 | 25.9 | 10.9 | 13.0 | 12.4 | 4.7 | 130 / 104 |
| peridot | Iron Age | 28 | 14.0 | 22.4 | 18.6 | 7.8 | 9.3 | 8.9 | 3.4 | 93 / 75 |
| amber | Stone Age | 4 | 4.5 | 7.2 | 6.0 | 2.5 | 3.0 | 2.9 | 1.1 | 30 / 24 |

Bows and arrows (`vitals::gear::arrow_hit`): an arrow's head sets its item level (a plain one is flint-tipped, level 5); the bow is its times (its stave's spring, grade and gems; a Sound yew bow ×1). A full draw takes 1 s and hits by its speed: less drawn, less; in the head ×2. Its level counts against a big creature at most the size's cap over it, as a blade's.

| age | arrow level | hit at full draw |
|---|---|---|
| Stone Age | 5 | 7.6 |
| Bronze Age | 15 | 12.1 |
| Iron Age | 25 | 19.4 |
| Age of Steam | 35 | 31.1 |
| Age of Lightning | 45 | 49.7 |
| Age of the Future | 55 | 79.6 |
| Age of Stars | 65 | 127.3 |

## Tames by saddle

A saddle (and barding) of an age carries a tame up to that age's reference level, never past it and never below where its own level puts it (Ronald, 11-06). Its own bonuses (tamed points, imprint ×1.2, bond ×1.05) add at most ×1.6, as gear's grade and marks do. A level-0 utahraptor and wyvern, health / hit, plain and at the ceiling:

| species | no saddle | Stone Age saddle | Bronze Age saddle | Iron Age saddle | Age of Steam saddle | Age of Lightning saddle | Age of the Future saddle | Age of Stars saddle |
|---|---|---|---|---|---|---|---|---|
| utahraptor | 103 / 10 (top 165 / 16) | 131 / 12 (top 209 / 20) | 209 / 20 (top 334 / 31) | 334 / 31 (top 535 / 50) | 535 / 50 (top 856 / 80) | 856 / 80 (top 1370 / 129) | 1370 / 129 (top 2192 / 206) | 2192 / 206 (top 3507 / 330) |
| wyvern | 456 / 15 (top 730 / 24) | 577 / 19 (top 923 / 30) | 923 / 30 (top 1478 / 48) | 1478 / 48 (top 2364 / 77) | 2364 / 77 (top 3783 / 124) | 3783 / 124 (top 6052 / 198) | 6052 / 198 (top 9683 / 317) | 9683 / 317 (top 15494 / 507) |

## Biome bosses

Each boss is a titan at its form's age (`animals_core::bosses`): its numbers from the curve like any creature's, a fight of 2-5 minutes for a same-age player, the cap and the floor against over-tier gear. Seconds to kill and its hits to down the player (grade 4), by the player's age. Forms with no offering are data for later ages.

| boss | form | offering | level | health | armour | hit | Stone | Bronze | Iron | Steam | Stars |
|---|---|---|---|---|---|---|---|---|---|---|---|
| drenkaun | Stone Age | animals:forest_offering | 5 | 912 | 31.0 | 28.3 | 218 s / 1.1 | 156 s / 1.5 | 156 s / 1.5 | 156 s / 1.5 | 156 s / 1.5 |
| drenkaun | Bronze Age | - | 15 | 1459 | 49.7 | 45.3 | 444 s / 0.6 | 218 s / 1.1 | 156 s / 1.5 | 156 s / 1.5 | 156 s / 1.5 |
| drenkaun | Iron Age | - | 25 | 2334 | 79.5 | 72.4 | 952 s / 0.4 | 444 s / 0.6 | 218 s / 1.1 | 156 s / 1.5 | 156 s / 1.5 |
| drenkaun | Age of Steam | - | 35 | 3734 | 127.2 | 115.9 | 2141 s / 0.2 | 952 s / 0.4 | 444 s / 0.6 | 218 s / 1.1 | 156 s / 1.5 |

drenkaun: a titan apex; attacks charge (×0.75 hit, 1.2 s tell), sweep (×0.50 hit, 0.8 s tell), stomp (×0.40 hit, 0.9 s tell), roots (×0.25 hit, 1.4 s tell); parts left_antler at 10 %, right_antler at 20 %, crown at 30 %; knocked down by 8 % of its health in stagger; loot animals:warden_heartseed 35 % (certain by kill 3), animals:warden_antler 60 % (certain by kill 2), animals:warden_bark 100 %, leather 100 %, bone 100 %; its young animals:drenkaun_calf (30 %, certain by kill 4).

| borkon | Bronze Age | animals:mountain_offering | 15 | 1459 | 49.7 | 45.3 | 444 s / 0.6 | 218 s / 1.1 | 156 s / 1.5 | 156 s / 1.5 | 156 s / 1.5 |
| borkon | Iron Age | - | 25 | 2334 | 79.5 | 72.4 | 952 s / 0.4 | 444 s / 0.6 | 218 s / 1.1 | 156 s / 1.5 | 156 s / 1.5 |
| borkon | Age of Steam | - | 35 | 3734 | 127.2 | 115.9 | 2141 s / 0.2 | 952 s / 0.4 | 444 s / 0.6 | 218 s / 1.1 | 156 s / 1.5 |
| borkon | Age of Lightning | - | 45 | 5975 | 203.5 | 185.4 | 3936 s / 0.1 | 2141 s / 0.2 | 952 s / 0.4 | 444 s / 0.6 | 156 s / 1.5 |

borkon: a titan apex; attacks charge (×0.75 hit, 1.4 s tell), toss (×0.55 hit, 0.9 s tell), rockfall (×0.35 hit, 1.6 s tell), leap (×0.50 hit, 1.0 s tell); parts left_horn at 10 %, right_horn at 20 %, fleece at 30 %; knocked down by 8 % of its health in stagger; loot animals:summit_stone 35 % (certain by kill 3), animals:borkon_horn 60 % (certain by kill 2), animals:height_fleece 100 %, wool 100 %, raw_gold 100 %, bone 100 %; its young animals:borkon_kid (30 %, certain by kill 4).

## Loot per hour

Every kill priced in effort minutes (`crate::loot`, `vitals::loot`): its drops at their cheapest cost in the graph, against its time, which is finding one (1.5 min for a common animal, longer as its spawn chance is smaller, shared by a pack that comes at you together; 0.5 min for a mob at night), the fight at its home age (grade 4), 0.25 min to take what it leaves and 2 min to work each hide or fur (scraping, tanning, drying: `vitals::loot::PELT_MINUTES`, added to their price). A fair rate is about 1 EM a minute, within ±30 % (up to +50 % more for what fights back). Wild ones at level 0; a big one's breaks counted. Drops that kills give are priced from their cheapest kill (Planner 11-23, option A: its minutes shared among its drops by their old hand prices, `vitals::loot::KILL_DROPS`), so that kill pays 1 and the rest less: "poor" is only a poor way to spend time, "over" fails the check.

| kill | age | role | find (min) | kills | fight (s) | drops | loot (EM) | minutes | EM a minute | band |
|---|---|---|---|---|---|---|---|---|---|---|
| brown_trout | Stone Age | prey | 3.7 | 1.0 | 0.3 | 1.0 animals:trout | 4.0 | 4.0 | 1.00 | ok |
| pacu | Stone Age | prey | 3.7 | 1.0 | 0.5 | 1.5 animals:pacu | 4.0 | 4.0 | 1.00 | ok |
| gastonia | Stone Age | tank | 1.5 | 1.0 | 25.3 | 9.5 animals:carcass, 3.0 animals:scute | 2.2 | 2.2 | 1.00 | ok |
| pinacosaurus | Stone Age | tank | 1.5 | 1.0 | 25.3 | 9.5 animals:carcass, 3.0 animals:scute | 2.2 | 2.2 | 1.00 | ok |
| zombie | Stone Age | hunter | 0.5 | 1.0 | 3.5 | 1.0 rotten_flesh | 0.81 | 0.8 | 1.00 | ok |
| creeper | Stone Age | prey | 0.5 | 1.0 | 2.7 | 1.0 gunpowder | 0.80 | 0.8 | 1.00 | ok |
| reindeer | Stone Age | grazer | 1.5 | 1.0 | 3.9 | 3.5 animals:venison, 2.5 fur | 6.7 | 6.8 | 0.98 | ok |
| white_tailed_deer | Stone Age | prey | 1.5 | 1.0 | 2.7 | 3.5 animals:venison, 1.5 hide | 4.4 | 4.8 | 0.92 | ok |
| mountain_hare | Stone Age | prey | 1.5 | 1.0 | 0.5 | 1.5 animals:hare, 1.0 fur | 3.4 | 3.8 | 0.91 | ok |
| hare | Stone Age | prey | 1.5 | 1.0 | 0.5 | 1.5 animals:hare, 1.0 fur | 3.4 | 3.8 | 0.91 | ok |
| arctic_fox | Stone Age | hunter | 1.5 | 2.5 | 0.6 | 1.5 animals:carcass, 1.5 fur | 3.5 | 3.9 | 0.90 | ok |
| coyote | Stone Age | hunter | 1.5 | 2.5 | 3.5 | 1.5 animals:carcass, 1.5 fur | 3.5 | 3.9 | 0.89 | ok |
| cedarosaurus | Stone Age | grazer | 3.7 | 1.0 | 65.9 | 16.5 animals:carcass, 5.0 hide | 13 | 15.1 | 0.85 | ok |
| iguanodont | Stone Age | grazer | 1.5 | 1.0 | 65.9 | 12.5 animals:carcass, 2.0 animals:claw | 2.4 | 2.8 | 0.85 | ok |
| red_fox | Stone Age | hunter | 1.5 | 2.5 | 0.7 | 1.5 animals:carcass, 1.0 fur | 2.4 | 2.9 | 0.83 | ok |
| grey_wolf | Stone Age | hunter | 3.7 | 2.5 | 3.5 | 2.5 animals:carcass, 2.0 fur | 4.7 | 5.8 | 0.81 | ok |
| red_deer | Stone Age | prey | 3.7 | 1.0 | 15.7 | 4.5 animals:venison, 2.0 hide | 5.8 | 8.3 | 0.71 | ok |
| skeleton | Stone Age | hunter | 0.5 | 1.0 | 2.8 | 1.0 arrow, 1.0 bone | 0.52 | 0.8 | 0.66 | poor |
| jaguar | Stone Age | apex | 3.7 | 1.0 | 5.0 | 3.5 animals:carcass, 2.5 fur | 5.9 | 9.1 | 0.65 | poor |
| capybara | Stone Age | prey | 1.5 | 1.0 | 2.7 | 2.5 animals:carcass, 1.0 hide | 2.5 | 3.8 | 0.65 | poor |
| spider | Stone Age | hunter | 0.5 | 1.0 | 2.8 | 1.0 string, 0.3 spider_eye | 0.40 | 0.8 | 0.50 | poor |
| utahraptor | Stone Age | hunter | 7.5 | 2.5 | 17.2 | 5.5 animals:carcass, 1.0 animals:claw, 3.0 feather | 1.7 | 3.5 | 0.47 | poor |
| crocodyliform | Stone Age | hunter | 3.7 | 1.0 | 20.2 | 4.5 animals:carcass, 2.0 animals:scute | 1.2 | 4.3 | 0.28 | poor |
| protoceratops | Stone Age | grazer | 1.5 | 1.0 | 3.9 | 3.5 animals:carcass | 0.45 | 1.8 | 0.25 | poor |
| citipati | Stone Age | grazer | 1.5 | 1.0 | 3.9 | 3.5 animals:carcass | 0.45 | 1.8 | 0.25 | poor |
| dog | Stone Age | hunter | 1.5 | 2.5 | 0.6 | 1.5 animals:carcass | 0.19 | 0.9 | 0.22 | poor |
| yacare_caiman | Stone Age | hunter | 1.5 | 1.0 | 3.5 | 2.5 animals:carcass | 0.32 | 1.8 | 0.18 | poor |
| spectacled_caiman | Stone Age | hunter | 1.5 | 1.0 | 3.5 | 2.5 animals:carcass | 0.32 | 1.8 | 0.18 | poor |
| psittacosaurus | Stone Age | grazer | 1.5 | 1.0 | 3.9 | 2.5 animals:carcass | 0.32 | 1.8 | 0.18 | poor |
| cat | Stone Age | hunter | 1.5 | 1.0 | 0.7 | 1.5 animals:carcass | 0.19 | 1.8 | 0.11 | poor |
| herring_gull | Stone Age | prey | 1.5 | 1.0 | 0.3 | 1.0 animals:carcass | 0.13 | 1.8 | 0.07 | poor |
| turkey_vulture | Stone Age | prey | 1.5 | 1.0 | 0.4 | 1.0 animals:carcass | 0.13 | 1.8 | 0.07 | poor |
| wyvern | Stone Age | apex | 50.0 | 1.0 | 85.7 | 13.5 animals:carcass, 1.5 animals:scute, 1.0 animals:claw | 2.6 | 51.7 | 0.05 | poor |
| lungfish | Stone Age | prey | 3.7 | 1.0 | 0.5 | 1.5 animals:carcass | 0.19 | 4.0 | 0.05 | poor |
| golden_eagle | Stone Age | hunter | 3.7 | 1.0 | 0.7 | 1.5 animals:carcass | 0.19 | 4.0 | 0.05 | poor |
| raptor | Stone Age | hunter | 3.7 | 1.0 | 3.0 | 1.5 animals:carcass | 0.19 | 4.0 | 0.05 | poor |
| osmolskae | Stone Age | hunter | 3.7 | 1.0 | 3.0 | 1.5 animals:carcass | 0.19 | 4.0 | 0.05 | poor |
| raven | Stone Age | prey | 3.7 | 1.0 | 0.3 | 1.0 animals:carcass | 0.13 | 4.0 | 0.03 | poor |
| great_horned_owl | Stone Age | hunter | 3.7 | 1.0 | 0.5 | 1.0 animals:carcass | 0.13 | 4.0 | 0.03 | poor |
| snowy_owl | Stone Age | hunter | 3.7 | 1.0 | 0.5 | 1.0 animals:carcass | 0.13 | 4.0 | 0.03 | poor |
| wood_mouse | Stone Age | prey | 1.5 | 1.0 | 0.1 | - | -0.00 | 1.8 | -0.00 | no loot |
| field_vole | Stone Age | prey | 1.5 | 1.0 | 0.1 | - | -0.00 | 1.8 | -0.00 | no loot |
| kryptobaatar | Stone Age | prey | 1.5 | 1.0 | 0.1 | - | -0.00 | 1.8 | -0.00 | no loot |
| priscagama | Stone Age | prey | 1.5 | 1.0 | 0.1 | - | -0.00 | 1.8 | -0.00 | no loot |
| house_sparrow | Stone Age | prey | 1.5 | 1.0 | 0.1 | - | -0.00 | 1.8 | -0.00 | no loot |
| snow_bunting | Stone Age | prey | 1.5 | 1.0 | 0.1 | - | -0.00 | 1.8 | -0.00 | no loot |
| savanna_hawk | Stone Age | hunter | 1.5 | 1.0 | 0.4 | - | -0.00 | 1.8 | -0.00 | no loot |
| lycoptera | Stone Age | prey | 1.5 | 1.0 | 0.1 | - | -0.00 | 1.8 | -0.00 | no loot |
| confuciusornis | Stone Age | prey | 1.5 | 1.0 | 0.3 | - | -0.00 | 1.8 | -0.00 | no loot |
| gobipteryx | Stone Age | prey | 3.7 | 1.0 | 0.2 | - | -0.00 | 4.0 | -0.00 | no loot |

What kills drop, priced from their cheapest kill; the weight is the old hand price (option B, to come back to):

| drop | weight (old EM) | EM now | cheapest from |
|---|---|---|---|
| animals:carcass | 4.0 | 0.13 | gastonia |
| animals:venison | 3.0 | 0.35 | reindeer |
| animals:hare | 3.0 | 0.81 | mountain_hare |
| animals:trout | 3.0 | 4.0 | brown_trout |
| animals:pacu | 3.0 | 2.7 | pacu |
| animals:claw | 10 | 0.41 | iguanodont |
| animals:scute | 10 | 0.32 | gastonia |
| hide | 2.0 | 2.1 | cedarosaurus |
| fur | 2.0 | 2.2 | arctic_fox |
| feather | 2.0 | 0.19 | utahraptor |
| bone | 2.0 | 0.40 | skeleton |
| arrow | 2.0 | 0.13 | skeleton |
| gunpowder | 2.0 | 0.80 | creeper |
| spider_eye | 2.0 | 0.96 | spider |
| rotten_flesh | 2.0 | 0.81 | zombie |

## The perk web's prices

A unit is +2 % to one family; a small node gives 1, a notable 3-4, a keystone nets 0. Inside a family bonuses add, between families they multiply. A family tops out at ×1.6 (30 units: one age); everything together (gear, web, a tame's bond, food) at ×2.5 (two ages). Families: melee damage, ranged damage, spell damage, defence, health, tame damage, taming speed, craft grade, gathering speed, mining speed, move speed, mana cost, stamina, slower hunger, cooking, harvest, riding speed, trade, smelting speed, machine speed. The nodes, skills and their checks: `mods/base/vitals/WEB_SHEET.md`.

## Age keys by path

Moving to the next age takes a key, one thing from each gate (materials and tools, a biome boss, knowledge, infrastructure), by a route for each path. What exists so far:

| age | tech | magic | beasts | not built yet |
|---|---|---|---|---|
| Stone Age | base:pickaxe, base:furnace, animals:warden_heartseed | base:dowsing_rod, animals:warden_heartseed | animals:club, animals:tranquilizer_dart, animals:nest, animals:warden_heartseed | knowledge: a ruin's blueprint or lore page |
| Bronze Age | animals:summit_stone | animals:summit_stone | animals:summit_stone | materials and tools: the Bronze Age's own; knowledge: a ruin's blueprint or lore page; infrastructure: a working setup from the Bronze Age |
| Iron Age | - | - | - | all of it |
| Age of Steam | - | - | - | all of it |
| Age of Lightning | - | - | - | all of it |
| Age of the Future | - | - | - | all of it |
| Age of Stars | - | - | - | all of it |

What each route costs from scratch (`graph::route_cost`): its items at their cheapest, plus every tool and station on the way once. Path parity: a route at most ×1.15 its age's cheapest. Own: without what every route of the age takes (its boss part), what tells the paths apart.

| age | path | EM | × cheapest | own EM | tools and stations |
|---|---|---|---|---|---|
| Stone Age | tech | 48 | 1.01 | 2.9 | crafting_table, axe, sharp_flake |
| Stone Age | beasts | 47 | 1.00 | 2.3 | crafting_table, axe, sharp_flake |
| Stone Age | magic | 50 | 1.07 | 5.5 | sharp_flake |
| Bronze Age | tech | 60 | 1.00 | 0.00 |  |
| Bronze Age | beasts | 60 | 1.00 | 0.00 |  |
| Bronze Age | magic | 60 | 1.00 | 0.00 |  |

## What exists, by age and path

Every named item and block, by its tag (`content::RULES`). EM is the cheapest way to have one; "-" not had as an item.

### Stone Age (550)

**tech** (46)

- material: gem_amber (2.0), gem_tektite (5.1)
- tool: arrow (0.13), axe (0.48), bow (0.48), glider (8.2), hoe (0.70), knife (0.43), pickaxe (0.48), shovel (0.48), spear (0.70), sword (0.70)
- armour: boots (0.70), chestplate (0.70), helmet (0.70), leggings (0.70)
- station: furnace (1.2)
- building: slate_tiles (0.16)
- part: part_arrowhead (0.22), part_bar_guard (0.22), part_barbed_point (0.22), part_bearded_axe_head (0.22), part_boot_shell (0.22), part_bowstring (0.22), part_chest_shell (0.22), part_felling_axe_head (0.22), part_grip (0.22), part_haft (0.22), part_helmet_shell (0.22), part_hoe_blade (0.22), part_knife_blade (0.22), part_lashing (0.22), part_leaf_point (0.22), part_leg_shell (0.22), part_lining (0.22), part_long_blade (0.22), part_long_haft (0.22), part_pick_head (0.22), part_round_pommel (0.22), part_self_stave (0.22), part_shaft (0.22), part_short_blade (0.22), part_short_stave (0.22), part_shovel_blade (0.22), part_straight_blade (0.22), part_straps (0.22)

**magic** (4)

- tool: dowsing_rod (5.2)
- building: ley_stone (3.0)
- sky: eclipse_ash (40)
- technical: waystone (-)

**beasts** (45)

- ground: animals:amber_ore (-), animals:termite_mound (0.30)
- material: animals:claw (0.41), animals:scute (0.32)
- beast: animals:amber (2.0), animals:amber_block (8.1), animals:brown_trout_egg (4.0), animals:carcass (0.13), animals:cedarosaurus_egg (4.0), animals:citipati_egg (4.0), animals:club (0.40), animals:confuciusornis_egg (4.0), animals:crocodyliform_egg (4.0), animals:gastonia_egg (4.0), animals:gobipteryx_egg (4.0), animals:golden_eagle_egg (4.0), animals:great_horned_owl_egg (4.0), animals:herring_gull_egg (4.0), animals:house_sparrow_egg (4.0), animals:iguanodont_egg (4.0), animals:kibble (0.62), animals:lungfish_egg (4.0), animals:lycoptera_egg (4.0), animals:nest (0.45), animals:pacu_egg (4.0), animals:pinacosaurus_egg (4.0), animals:priscagama_egg (4.0), animals:protoceratops_egg (4.0), animals:psittacosaurus_egg (4.0), animals:raptor_egg (4.0), animals:raven_egg (4.0), animals:savanna_hawk_egg (4.0), animals:sedative (0.73), animals:snow_bunting_egg (4.0), animals:snowy_owl_egg (4.0), animals:spectacled_caiman_egg (4.0), animals:tranquilizer_dart (0.31), animals:turkey_vulture_egg (4.0), animals:utahraptor_egg (4.0), animals:utahraptor_saddle (8.1), animals:velociraptor_osmolskae_egg (4.0), animals:wyvern_egg (4.0), animals:yacare_caiman_egg (4.0)
- technical: animals:gateway (-), animals:termite_mound_upper (-)

**shared** (455)

- ground: animals:red_sand (0.10), animals:red_sandstone (0.15), banded_clay (0.15), banded_clay_coal_ore (-), brain_coral_block (0.15), clay (0.10), coal_ore (-), coarse_dirt (0.10), cobblestone (0.15), coral_rubble (0.15), cream_sandstone (0.15), deep_leaf_litter (0.10), dirt (0.10), fallen_branch (-), glacier_ice (0.15), granite (0.15), grass (0.10), gravel (0.10), ice (0.15), impact_glass (0.10), lake_ice (0.15), leaf_litter (0.10), limestone (0.15), loose_flint (-), loose_rock (-), loose_tektite (-), moss_block (0.10), mossy_cobblestone (0.15), ochre_sandstone (0.15), podzol (0.10), porites_coral_block (0.15), red_sandstone (0.15), sand (0.10), sandstone (0.15), slate (0.15), snow (0.10), snow_layer (0.10), snowy_grass (0.10), staghorn_coral_block (0.15), suevite (0.15), weathered_limestone (0.15)
- wood: animals:araucaria_leaves (0.10), animals:araucaria_log (0.30), animals:araucaria_sapling (0.10), animals:ginkgo_leaves (0.10), animals:ginkgo_log (0.30), animals:ginkgo_sapling (0.10), animals:rotten_log (0.30), animals:scale_conifer_leaves (0.10), animals:scale_conifer_log (0.30), animals:scale_conifer_sapling (0.10), animals:swamp_cypress_leaves (0.10), animals:swamp_cypress_log (0.30), animals:swamp_cypress_sapling (0.10), animals:tempskya_leaves (0.10), animals:tempskya_log (0.30), acacia_branch (0.20), acacia_leaves (0.10), acacia_log (0.30), acacia_planks (0.09), acuri_palm_leaves (0.10), acuri_palm_log (0.30), acuri_palm_planks (0.09), acuri_palm_sapling (0.10), andiroba_branch (0.20), andiroba_leaves (0.10), andiroba_log (0.30), andiroba_planks (0.09), andiroba_sapling (0.10), birch_branch (0.20), birch_leaves (0.10), birch_log (0.30), birch_log_hollow (0.31), birch_planks (0.09), birch_wood (0.31), brazil_nut_branch (0.20), brazil_nut_buttress (0.30), brazil_nut_leaves (0.10), brazil_nut_log (0.30), brazil_nut_planks (0.09), brazil_nut_sapling (0.10), bush_leaves (0.10), cacao_leaves (0.10), cacao_log (0.30), cacao_planks (0.09), cacao_sapling (0.10), cherry_branch (0.20), cherry_leaves (0.10), cherry_log (0.30), cherry_planks (0.09), cherry_sapling (0.10), creosote_leaves (0.10), dogwood_branch (0.20), dogwood_leaves (0.10), dogwood_log (0.30), ipe_branch (0.20), ipe_leaves (0.10), ipe_log (0.30), ipe_planks (0.09), ipe_sapling (0.10), juniper_leaves (0.10), matamata_branch (0.20), matamata_leaves (0.10), matamata_log (0.30), matamata_planks (0.09), matamata_sapling (0.10), mesquite_branch (0.20), mesquite_leaves (0.10), mesquite_log (0.30), mesquite_planks (0.09), mesquite_sapling (0.10), moriche_palm_leaves (0.10), moriche_palm_log (0.30), moriche_palm_planks (0.09), moriche_palm_sapling (0.10), mossy_sequoia_log (0.30), oak_branch (0.20), oak_leaves (0.10), oak_log (0.30), oak_log_hollow (0.31), oak_planks (0.09), oak_wood (0.31), sapling (0.10), sequoia_leaves (0.10), sequoia_log (0.30), sequoia_planks (0.09), sequoia_sapling (0.10), spruce_leaves (0.10), spruce_log (0.30), spruce_log_hollow (0.31), spruce_planks (0.09), spruce_wood (0.31), stripped_acacia_log (0.30), stripped_acuri_palm_log (0.30), stripped_andiroba_log (0.30), stripped_birch_log (0.30), stripped_brazil_nut_log (0.30), stripped_cacao_log (0.30), stripped_cherry_log (0.30), stripped_dogwood_log (0.30), stripped_ipe_log (0.30), stripped_matamata_log (0.30), stripped_mesquite_log (0.30), stripped_moriche_palm_log (0.30), stripped_oak_log (0.30), stripped_sequoia_log (0.30), stripped_spruce_log (0.30), willow_leaves (0.10)
- plant: animals:cycad (0.10), animals:fern (0.10), animals:flowering_shrub (0.10), animals:horsetail (0.10), animals:water_lily (0.10), alligator_flag (0.10), alpenrose (0.10), alpine_aster (0.10), arctic_coltsfoot (0.10), arctic_poppy (0.10), banana (0.10), beach_plum_bush (0.10), bellflower (0.10), berry_bush (0.10), black_eyed_susan (0.10), blazing_star (0.10), bluegrass (-), bracket_fungus (0.10), bramble_bush (0.10), brittlebush (0.10), bromeliad (0.10), bunchberry (0.10), bush_stem (-), butterfly_milkweed (0.10), cattail (0.10), chanterelle (0.10), chanterelle_mushroom (-), cloudberry_bush (0.10), clover (0.10), cottongrass (0.10), crowberry_bush (0.10), dandelion (0.10), dead_bush (0.10), desert_marigold (0.10), dryads_saddle_mushroom (-), dune_grass (0.10), dung_pat (0.10), dwarf_birch (0.10), edelweiss (0.10), elkhorn_coral (0.10), fern (0.10), finger_coral (0.10), fireweed (0.10), foxfire (0.10), foxglove (0.10), gentian (0.10), globeflower (0.10), globemallow (0.10), glow_mycena (0.10), goblins_gold (0.10), gold_poppy (0.10), hazel_bush (0.10), heliconia (0.10), honey_fungus (0.10), horned_poppy (0.10), hottentot_fig (0.10), huckleberry_bush (0.10), inula (-), labrador_tea (0.10), lavender (-), leadplant (0.10), lesser_celandine (0.10), liana (0.10), lily_of_the_valley (0.10), lily_pad (0.10), lingonberry_bush (0.10), lousewort (0.10), lungwort (0.10), lupine (0.10), may_lily (0.10), morning_glory (0.10), moss (-), ocotillo (0.10), pencil_flower (0.10), philodendron (0.10), porcini (0.10), porcini_mushroom (-), prickly_pear (0.10), prickly_rose (0.10), purple_coneflower (0.10), pygmy_buttercup (0.10), raspberry_leaves (-), reed (0.10), reindeer_lichen (0.10), rose (0.10), saxifrage (0.10), sea_daffodil (0.10), sea_fan (0.10), sea_holly (0.10), sea_rocket (0.10), sea_rod (0.10), seagrass (0.10), sedge (0.10), senna (0.10), sensitive_plant (0.10), short_grass (-), snowberry_leaves (-), staghorn_coral (0.10), sweet_woodruff (0.10), sword_fern (0.10), tall_grass (0.10), tall_grass_double (0.10), tall_tallgrass (-), toadstool_mushroom (-), tulip_orange (-), tulip_pink (-), tulip_purple (-), tulip_red (-), tulip_white (-), tulip_yellow (-), turtle_grass (0.10), twinflower (0.10), verbascum (-), water_hyacinth (0.10), water_primrose (0.10), wild_sunflower (0.10), wood_anemone (0.10), wood_cranesbill (0.10)
- crop: barley (0.20), barley_seeds (0.20), bean_seeds (0.20), beans (0.20), beetroot (0.20), beetroots (0.20), cactus (0.20), carrot (0.20), carrots (0.20), cassava (0.20), chia (0.20), chia_seeds (0.20), corn (0.20), maize (0.20), maize_seeds (0.20), melon (0.20), millet (0.20), millet_seeds (0.20), potato (0.20), potatoes (0.20), pumpkin (0.20), rice (0.20), rice_seeds (0.20), rye (0.20), rye_seeds (0.20), sorghum (0.20), sorghum_seeds (0.20), sugar_cane (0.20), tepary_beans (0.20), wheat (0.20), wheat_seeds (0.20), wild_rice (0.20)
- food: animals:araucaria_cone (0.20), animals:cooked_hare (0.98), animals:cooked_pacu (2.8), animals:cooked_trout (4.2), animals:cooked_venison (0.52), animals:cycad_seed (0.20), animals:ginkgo_seed (0.20), animals:hare (0.81), animals:pacu (2.7), animals:trout (4.0), animals:venison (0.35), acorn (0.20), acuri_nut (0.20), andiroba_seed (0.20), apple (0.20), apple_pie (2.7), baked_potato (0.37), bananas (0.20), beach_plums (0.20), beef (3.0), berries (0.20), brazil_nut (0.20), bread (0.65), cacao_beans (0.25), cacao_pod (0.20), cattail_root (0.20), cherry (0.20), chicken (3.0), cloudberries (0.20), cooked_beef (3.2), cooked_chicken (3.2), cooked_dryads_saddle (-), cooked_egg (2.2), cooked_mutton (3.2), cooked_porkchop (3.2), cooked_rice (0.37), cooked_wild_rice (0.37), crowberries (0.20), dried_apple (0.25), dried_berries (0.25), dried_meat (0.40), dryads_saddle (-), hazelnut (0.20), huckleberries (0.20), ipe_seed (0.20), kelp (0.20), lingonberries (0.20), matamata_fruit (0.20), melon_seeds (0.03), melon_slice (0.01), mesquite_pod (0.20), moriche_fruit (0.20), mushroom_brown (0.20), mushroom_red (0.20), mutton (3.0), pine_nut (0.20), porkchop (3.0), prickly_pear_fruit (0.20), pumpkin_seeds (0.03), pumpkin_slice (0.01), raspberries (0.20), raspberry (-), roasted_cacao (0.42), roasted_corn (0.37), rose_hip (0.20), sequoia_cone (0.20), smoked_meat (0.52), snowberry (-), sunflower_seeds (0.20), toadstool (-)
- material: animals:forest_offering (0.75), animals:warden_antler (15), animals:warden_bark (15), bandage (0.14), bone (0.40), book (3.3), bowl (0.08), clay_ball (-), cloth (0.30), coal (0.33), coal_block (3.0), compost (0.45), dung (2.0), egg (2.0), feather (0.19), flint (0.20), fur (2.2), gunpowder (0.80), hide (2.1), leather (2.6), oak_bark (0.08), paper (0.22), plant_fibre (0.02), rock (0.10), rotten_flesh (0.81), sharp_flake (0.25), sole_leather (2.4), spider_eye (0.96), stick (0.05), string (0.08), sugar (0.25), tektite (5.0)
- clothing: fur_boots (4.5), fur_coat (9.0), fur_hat (4.5), fur_robe (11), fur_trousers (6.8), hide_boots (4.4), hide_coat (8.8), hide_hat (4.4), hide_trousers (6.6), linen_boots (0.73), linen_coat (1.4), linen_hat (0.73), linen_trousers (1.1), rain_cape (13), wool_boots (0.90), wool_cloak (1.3), wool_coat (1.8), wool_hat (0.90), wool_trousers (1.4)
- station: animals:drenkaun_altar (1.9), bow_drill (0.23), campfire_laid (0.75), charcoal_clamp (1.1), compost_bin (0.66), cooking_pot (0.75), drying_rack (0.30), fire_bed (0.22), hand_drill (0.15), stew (0.80), tanning_frame (0.58), tanning_pit (0.66)
- building: bed (1.5), bookshelf (11), chest (0.75), crafting_table (0.40), hay_bale (1.9), ladder (0.13), oak_door (0.19), wool (0.38)
- light: torch (0.11)
- part: animals:warden_heartseed (45)
- technical: campfire (-), farmland (-), grazed_grass (-), lava (-), maize_top (-), melon_stem (-), millet_top (-), paddy_rice (-), pumpkin_stem (-), sorghum_top (-), tombstone (-), water (-), packages:unknown (-), portals:portal (-)

Notes:
- base:short_grass, base:tall_tallgrass, base:bluegrass, base:tulip_*, base:inula, base:verbascum, base:lavender, base:moss, base:bush_stem, base:raspberry_leaves, base:snowberry_leaves, base:porcini_mushroom, base:chanterelle_mushroom, base:toadstool_mushroom, base:dryads_saddle_mushroom: upstream's grasses, flowers, moss, berry bushes and forest mushrooms: no worldgen of ours grows them yet
- base:raspberry, base:snowberry, base:toadstool, base:dryads_saddle, base:cooked_dryads_saddle: upstream's berries and mushrooms (food rows in `vitals::food`): from its bushes and fungi, which don't grow yet; the dryad's saddle cooks over a fire
- base:clay_ball: upstream's clay ball: our clay drops clay, so none comes in yet (4 make a clay block)
- base:water, base:lava: liquids, carried in buckets
- base:farmland, base:grazed_grass, base:*_top, base:*_stem, base:paddy_rice, base:tombstone: growing stages, tilled or grazed ground, the grave a death leaves
- base:campfire: a laid campfire, lit
- base:loose_rock, base:loose_flint, base:loose_tektite, base:fallen_branch: picked up as a rock, flint, tektite or sticks
- base:coal_ore, base:banded_clay_coal_ore: breaks into coal
- animals:amber_ore: breaks into amber
- animals:gateway, animals:termite_mound_upper: the Cretaceous gateway is built of amber blocks; a mound's top
- portals:portal, packages:unknown: a linked world's door; a missing package's stand-in
- base:eclipse_ash: falls in an eclipse
- base:dowsing_rod: senses the ley lines: the Stone Age's magic route
- base:ley_stone: the ancients' carved stone round the waystones, some 600 blocks apart
- base:waystone: a sleeping waystone on a great ley crossing: not carried off, woken later
- base:axe, base:pickaxe, base:shovel, base:hoe, base:knife, base:spear, base:sword, base:bow: built from parts: its parts' materials set its real age (`MATERIALS`)
- base:helmet, base:chestplate, base:leggings, base:boots: built from a shell, fastening and lining: its shell sets its real age; a leather-look shell (leather, rawhide, bone)
- base:part_*: a part's material sets its real age (`MATERIALS`)
- base:gem_tektite, base:gem_amber: cut gems: a boost in a tool's, weapon's or armour's socket
- base:arrow: dropped by skeletons, or fletched from an arrowhead part, a stick and a feather (its head's material its age)
- base:furnace: the Stone Age key's infrastructure: what it makes is the Bronze Age's
- base:cooking_pot, base:stew: a clay pot (Ronald, 11-07: "move them to fit")
- animals:claw, animals:scute: a large animal's stagger breaks (animals_core::parts): a fight with a Utahraptor or an ankylosaur
- animals:drenkaun_altar: a biome boss's altar: the player picks when to fight (round 6)
- animals:forest_offering: the old forest's offering: calls Drenkaun, tames its calf
- animals:warden_heartseed: Drenkaun's loot, certain by the third kill: the Stone Age key's boss part
- animals:warden_antler, animals:warden_bark: Drenkaun's part breaks and loot
- animals:kibble: an egg, food, grain and fibre (no iron bucket; Ronald, 11-07: "move them to fit")

### Bronze Age (52)

**tech** (46)

- ground: copper_ore (-), gold_ore (-), granite_gold_ore (-), granite_iron_ore (-), iron_ore (-), meteorite (-), red_sandstone_iron_ore (-), tin_ore (-)
- material: bronze_ingot (7.3), copper_ingot (1.7), gold_block (29), gold_ingot (3.2), meteoric_iron (10), raw_copper (1.5), raw_gold (3.0), raw_tin (5.0), tin_ingot (5.2)
- armour: boots_scale (-), chestplate_scale (-), helmet_scale (-), leggings_scale (-)
- station: crucible (2.2)
- building: blue_glazed_tiles (0.32), blue_stained_glass (0.47), bricks (0.27), bronze_plate (7.3), checker_tiles (0.19), copper_block (15), frosted_glass (0.42), glass (0.27), green_glazed_tiles (0.33), green_stained_glass (0.52), leaded_glass (2.2), limestone_tiles (0.20), polished_limestone (0.19), red_glazed_tiles (0.30), red_stained_glass (0.42), smoked_glass (0.65), stone (0.32), stone_bricks (0.33), verdigris_copper (-)
- light: blue_lamp (1.4), bronze_lantern (31), glowstone (1.2), green_lamp (1.4), red_lamp (1.3)

**magic** (1)

- sky: pallasite (20)

**shared** (5)

- material: animals:borkon_horn (20), animals:height_fleece (20), animals:mountain_offering (3.8)
- station: animals:borkon_altar (5.1)
- part: animals:summit_stone (60)

Notes:
- base:copper_ore, base:tin_ore: breaks into raw copper or raw tin (a stone pickaxe): copper in cold, dry and beach land, tin on beaches and in mountains (`metals.rs`)
- base:iron_ore, base:granite_iron_ore, base:red_sandstone_iron_ore, base:meteorite, base:gold_ore, base:granite_gold_ore: breaks into raw iron, meteoric iron or raw gold (a stone pickaxe)
- base:pallasite: a stony-iron meteorite's olivine, found where one fell
- base:helmet_scale, base:chestplate_scale, base:leggings_scale, base:boots_scale: a bronze or copper shell's worn look: no copper or bronze in the game yet (only `/gear`)
- base:meteoric_iron: hammered cold, iron-grade before a player can smelt (Research round 17)
- base:raw_gold, base:gold_ingot, base:gold_block: cast and soft: the Bronze Age's (Ronald, 11-07: "move them to fit")
- base:verdigris_copper: a copper block weathered in the open air (`vitals::weathering`); breaks into the copper block
- animals:borkon_altar: the Height-Keeper's altar: stone bricks and gold (the Bronze Age's)
- animals:mountain_offering: the heights' offering, with a gold ingot: calls Borkon, tames its kid
- animals:summit_stone: Borkon's loot, certain by the third kill: the Bronze Age key's boss part
- animals:borkon_horn, animals:height_fleece: Borkon's part breaks and loot

### Iron Age (58)

**tech** (22)

- ground: redstone_ore (-), star_steel_ore (-)
- material: bucket (3.5), gem_peridot (10), iron_block (11), iron_ingot (1.2), lava_bucket (4.1), raw_iron (1.0), star_steel_ingot (12), star_steel_lump (12), steel_ingot (2.3), water_bucket (3.7)
- tool: grappling_hook (3.8)
- armour: boots_mail (2.8), chestplate_mail (4.0), helmet_mail (2.8), leggings_mail (4.0), shield (1.7)
- station: forge (6.1), woodcutter (1.5)
- building: music_player (1.9), steel_plate (2.3)

**magic** (32)

- food: glass_bottle (-), potion_blindness (-), potion_fire_resistance (-), potion_haste (-), potion_health_boost (-), potion_hunger (-), potion_instant_damage (-), potion_instant_health (-), potion_invisibility (-), potion_jump_boost (-), potion_levitation (-), potion_mining_fatigue (-), potion_nausea (-), potion_night_vision (-), potion_poison (-), potion_regeneration (-), potion_resistance (-), potion_slow_falling (-), potion_slowness (-), potion_speed (-), potion_strength (-), potion_water_breathing (-), potion_weakness (-), potion_wither (-)
- material: aruduk (60), emakaun (60), maunduk (60), remaru (60), unlearning_stone (20)
- light: star_lantern (62)
- sky: peridot (10), starheart (60)

**beasts** (1)

- beast: animals:wyvern_saddle (15)

**shared** (3)

- food: salt (3.9), salted_meat (4.3)
- tool: animals:spyglass (2.6)

Notes:
- base:shield: upstream's shield: planks round an iron boss, so the Iron Age's (whole, not built from parts)
- base:woodcutter: upstream's woodcutter: an iron blade on planks; cuts beams and eighths
- base:music_player: upstream's music player: planks round an iron works
- base:potion_*, base:glass_bottle: upstream's potions and their bottle: no brewing yet, only creative or commands
- base:redstone_ore, base:star_steel_ore: breaks into its ore (an iron pickaxe)
- base:helmet_mail, base:chestplate_mail, base:leggings_mail, base:boots_mail: the same armour with an iron or steel shell, an item of its own only for its worn look (`parts::ARMOUR_LOOKS`)
- base:star_lantern: a starheart in iron
- base:unlearning_stone: a peridot ground with tektites: gives back perk web nodes (vitals::perks)
- base:maunduk, base:aruduk, base:remaru, base:emakaun: stones for the web's sockets (vitals::sockets), found, never made: in pallasite today; bosses, ley nodes, ruins and rare animals add theirs (an earlier tier then)
- base:salt, base:salted_meat: salt is boiled from a water bucket (iron)
- animals:wyvern_saddle: a dragon-sized saddle has an iron frame

### Age of Steam (38)

**tech** (37)

- ground: diamond_ore (-)
- material: diamond (10), diamond_block (90), gem_diamond (10)
- armour: boots_gem (20), chestplate_gem (30), helmet_gem (20), leggings_gem (30)
- building: polished_obsidian (1.1)
- machine: anchor (9.9), assembler (10), assembly_tool (1.3), comparator (3.3), copper_lamp (7.2), dispenser (1.6), floater (10), golden_lamp (13), grab_tool (-), guide_rail (1.2), helm (0.55), lever (0.25), linear_guide (7.8), observer (3.1), piston (2.6), redstone (0.50), redstone_block (4.6), redstone_lamp (3.2), redstone_torch (0.60), repeater (2.7), rotor (15), slime_block (1.0), sticky_piston (3.7), stone_button (0.37), thruster (10), wheel (-), wheel_large (-), wheel_small (-)

**shared** (1)

- ground: obsidian (1.0)

Notes:
- base:thruster, base:helm, base:assembler, base:rotor, base:linear_guide, base:guide_rail, base:assembly_tool: upstream's ships and vehicles: redstone, pistons and iron, the Age of Steam's like our machines
- base:wheel_small, base:wheel, base:wheel_large, base:grab_tool: upstream's wheels take dyed wool (`#wool/*`), which our game doesn't make; the grab tool has no recipe
- base:diamond_ore: breaks into a diamond (a steel pickaxe: harvest 300)
- base:copper_lamp: the golden lamp's twin in copper; a redstone machine (the Age of Steam's)
- base:obsidian: a steel pickaxe or better (harvest 300)
- base:helmet_gem, base:chestplate_gem, base:leggings_gem, base:boots_gem: a diamond-scale shell's worn look
- base:redstone, base:redstone_block, base:redstone_torch, base:repeater, base:comparator, base:lever, base:stone_button, base:redstone_lamp, base:piston, base:sticky_piston, base:slime_block, base:observer, base:dispenser, base:floater, base:anchor, base:golden_lamp: redstone ore needs an iron pickaxe; machines are the Age of Steam's

## Tool parts' materials

Each `crafting::MATERIALS` row's age, and what a part of it is made from in the game today ("-": nothing yet).

| material | age | from |
|---|---|---|
| flint | Stone Age | base:flint |
| chert | Stone Age | - |
| tektite | Stone Age | base:tektite |
| obsidian | Stone Age | - |
| bone | Stone Age | base:bone |
| antler | Stone Age | - |
| stone | Stone Age | base:cobblestone |
| diamond | Age of Steam | base:diamond |
| copper | Bronze Age | base:copper_ingot |
| bronze | Bronze Age | base:bronze_ingot |
| wrought_iron | Iron Age | base:iron_ingot |
| meteoric_iron | Bronze Age | base:meteoric_iron |
| steel | Iron Age | base:steel_ingot |
| star_steel | Iron Age | base:star_steel_ingot |
| gold | Bronze Age | base:gold_ingot |
| oak | Stone Age | base:oak_planks |
| mesquite | Stone Age | base:mesquite_planks |
| birch | Stone Age | base:birch_planks |
| yew | Stone Age | - |
| elm | Stone Age | - |
| osage_orange | Stone Age | - |
| spruce | Stone Age | base:spruce_planks |
| branch | Stone Age | base:stick |
| plant_cordage | Stone Age | base:string |
| sinew | Stone Age | - |
| rawhide | Stone Age | - |
| birch_tar | Stone Age | - |
| leather | Stone Age | base:sole_leather |
| linen | Stone Age | base:cloth |
| wool | Stone Age | base:wool |
| fur | Stone Age | base:fur |

