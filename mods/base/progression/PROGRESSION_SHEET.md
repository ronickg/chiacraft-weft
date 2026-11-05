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
| every tool-part material tagged, made from its tier or below | passes |
| each age's making inside its budget | passes |
| every size and role inside its bands, floors hold | passes |
| every species inside its bands at every level, floors hold | passes |
| tames capped by their saddle and ceiling | passes |
| tweaks inside ×0.75-×1.25 with a reason | passes |

## The ages

Power grows ×1.6 an age: `g(L) = 1.6^(L/10)` over area and item level L (tier × 10 + a depth of 0-9). The currency is effort minutes (EM). The reference player is same-age gear at grade 4, measured at the age's middle (depth 5). The kit is what an average player makes in the age the first time (`graph::KITS`), gathered and crafted the cheapest way, against its budget: 40 % of the age's EM (`curve::MAKING_SHARE`). Everything once is one of every made thing tagged to the age, alternatives and all, each from scratch.

| tier | age | hours | EM | g at start | reference: hit / health / armour | things tagged | made | kit EM | of budget | everything once |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | Stone Age | 6 | 360 | 1.00 | 7.6 / 25 / 32 | 486 | 144 | 43 | 30 % | 247 EM |
| 1 | Bronze Age | 12 | 720 | 1.60 | 12.1 / 40 / 51 | 17 | 12 | 64 | 22 % | 18 EM |
| 2 | Iron Age | 16 | 960 | 2.56 | 19.4 / 65 / 81 | 44 | 39 | 68 | 18 % | 410 EM |
| 3 | Age of Steam | 18 | 1080 | 4.10 | 31.1 / 104 / 130 | 29 | 27 | 94 | 22 % | 512 EM |
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

## Species: today against the curve

Every wild animal rolls a level of 0-200 anywhere, and a level-200 one is 5× a level-0 one of its species (Ronald, 11-06). Its size class comes from its mass, its role and tweaks from `animals_core::danger`. Today is the species table's level-1 `base` (health, and damage as `SPECIES.md` gives it); the curve is a level-0 one of the Stone Age. A Stone Age species reaches the Bronze Age's band at level 59, the Iron Age's at 117, Steam's at 176.

| species | kg | size | role | today health | curve health (level 0) | ×  | today damage | curve hit (level 0) | curve health at 100 / 200 | tweaks |
|---|---|---|---|---|---|---|---|---|---|---|
| dog | 2.5 | small | hunter | 8 | 5 | 0.7 | 2 | 2.9 | 12 / 27 |  |
| cat | 4.5 | small | hunter | 6 | 5 | 0.9 | 1 | 2.9 | 12 / 27 |  |
| capybara | 50 | medium | prey | 12 | 19 | 1.6 | 1 | 2.5 | 42 / 94 |  |
| raptor | 15 | medium | hunter | 14 | 21 | 1.5 | 4 | 6.0 | 46 / 103 | health ×0.85: hunts in packs of three to five; one alone is softer |
| utahraptor | 300 | large | hunter | 60 | 103 | 1.7 | 10 | 9.7 | 231 / 516 | health ×0.85: hunts in packs (the Stikes Quarry group); one alone is softer |
| osmolskae | 15 | medium | hunter | 14 | 21 | 1.5 | 4 | 6.0 | 46 / 103 | health ×0.85: hunts in packs of three to five; one alone is softer |
| wyvern | 5000 | huge | apex | 400 | 456 | 1.1 | 30 | 14.9 | 1020 / 2281 |  |
| hare | 3.8 | small | prey | 5 | 4 | 0.8 | 0.5 | 1.2 | 9 / 21 |  |
| mountain_hare | 3 | small | prey | 5 | 4 | 0.8 | 0.5 | 1.2 | 9 / 21 |  |
| wood_mouse | 0.023 | small | prey | 2 | 4 | 2.1 | 0.2 | 1.2 | 9 / 21 |  |
| field_vole | 0.035 | small | prey | 2 | 4 | 2.1 | 0.2 | 1.2 | 9 / 21 |  |
| white_tailed_deer | 70 | medium | prey | 30 | 19 | 0.6 | 2 | 2.5 | 42 / 94 |  |
| red_deer | 160 | large | prey | 45 | 94 | 2.1 | 3 | 4.0 | 211 / 472 |  |
| brown_trout | 0.5 | small | prey | 3 | 4 | 1.4 | 0.3 | 1.2 | 9 / 21 |  |
| pacu | 4 | small | prey | 6 | 4 | 0.7 | 0.5 | 1.2 | 9 / 21 |  |
| red_fox | 5.5 | small | hunter | 10 | 5 | 0.5 | 2 | 2.9 | 12 / 27 |  |
| arctic_fox | 3.5 | small | hunter | 8 | 5 | 0.7 | 1.6 | 2.9 | 12 / 27 |  |
| raven | 1.2 | small | prey | 6 | 4 | 0.7 | 1 | 1.2 | 9 / 21 |  |
| protoceratops | 83 | medium | grazer | 35 | 27 | 0.8 | 2 | 4.0 | 60 / 135 |  |
| kryptobaatar | 0.04 | small | prey | 2 | 4 | 2.1 | 0.2 | 1.2 | 9 / 21 |  |
| priscagama | 0.03 | small | prey | 2 | 4 | 2.1 | 0.2 | 1.2 | 9 / 21 |  |
| reindeer | 120 | medium | grazer | 40 | 27 | 0.7 | 3 | 4.0 | 60 / 135 |  |
| coyote | 13 | medium | hunter | 18 | 24 | 1.3 | 3.5 | 6.0 | 54 / 122 |  |
| house_sparrow | 0.03 | small | prey | 2 | 4 | 2.1 | 0.2 | 1.2 | 9 / 21 |  |
| snow_bunting | 0.035 | small | prey | 2 | 4 | 2.1 | 0.2 | 1.2 | 9 / 21 |  |
| great_horned_owl | 1.4 | small | hunter | 8 | 5 | 0.7 | 2.5 | 2.9 | 12 / 27 |  |
| snowy_owl | 2 | small | hunter | 10 | 5 | 0.5 | 3 | 2.9 | 12 / 27 |  |
| golden_eagle | 4.5 | small | hunter | 16 | 5 | 0.3 | 4 | 2.9 | 12 / 27 |  |
| savanna_hawk | 0.85 | small | hunter | 8 | 5 | 0.7 | 2 | 2.9 | 12 / 27 |  |
| turkey_vulture | 1.4 | small | prey | 10 | 4 | 0.4 | 1 | 1.2 | 9 / 21 |  |
| herring_gull | 1 | small | prey | 8 | 4 | 0.5 | 1.5 | 1.2 | 9 / 21 |  |
| yacare_caiman | 35 | medium | hunter | 30 | 24 | 0.8 | 5 | 6.0 | 54 / 122 |  |
| spectacled_caiman | 25 | medium | hunter | 26 | 24 | 0.9 | 4.5 | 6.0 | 54 / 122 |  |
| grey_wolf | 42 | medium | hunter | 30 | 24 | 0.8 | 6 | 6.0 | 54 / 122 |  |
| jaguar | 85 | medium | apex | 45 | 35 | 0.8 | 9 | 6.5 | 78 / 176 |  |
| lungfish | 3 | small | prey | 8 | 4 | 0.5 | 0.6 | 1.2 | 9 / 21 |  |
| lycoptera | 0.02 | small | prey | 1 | 4 | 4.2 | 0.1 | 1.2 | 9 / 21 |  |
| psittacosaurus | 25 | medium | grazer | 22 | 27 | 1.2 | 1.5 | 4.0 | 60 / 135 |  |
| gastonia | 1900 | large | tank | 120 | 135 | 1.1 | 6 | 5.7 | 302 / 675 |  |
| pinacosaurus | 1900 | large | tank | 120 | 135 | 1.1 | 6 | 5.7 | 302 / 675 |  |
| iguanodont | 4000 | huge | grazer | 150 | 351 | 2.3 | 8 | 9.2 | 785 / 1755 |  |
| crocodyliform | 150 | large | hunter | 45 | 122 | 2.7 | 7 | 9.7 | 272 / 608 |  |
| citipati | 90 | medium | grazer | 40 | 27 | 0.7 | 3 | 4.0 | 60 / 135 |  |
| confuciusornis | 0.5 | small | prey | 6 | 4 | 0.7 | 1 | 1.2 | 9 / 21 |  |
| gobipteryx | 0.3 | small | prey | 6 | 4 | 0.7 | 1 | 1.2 | 9 / 21 |  |
| cedarosaurus | 10000 | huge | grazer | 300 | 351 | 1.2 | 10 | 11.0 | 785 / 1755 | hit ×1.2: a tail and a stamp: a sauropod's defence is its weight |

## The combat matrix

Each species at level 0 (the Stone Age's start) against a reference player of each age, grade 4: seconds to kill / its hits to down the player. The checks run it at every tenth level, at grade 4 and at the +60 % ceiling.

| species | Stone Age | Bronze Age | Iron Age | Age of Steam | Age of Lightning | Age of the Future | Age of Stars |
|---|---|---|---|---|---|---|---|
| dog | 0.7 / 27 | 0.4 / 62 | 0.3 / 111 | 0.2 / 177 | 0.1 / 283 | 0.1 / 453 | 0.0 / 725 |
| cat | 0.7 / 27 | 0.4 / 62 | 0.3 / 111 | 0.2 / 177 | 0.1 / 283 | 0.1 / 453 | 0.0 / 725 |
| capybara | 2.7 / 36 | 1.6 / 81 | 1.0 / 130 | 0.8 / 164 | 0.8 / 164 | 0.8 / 164 | 0.8 / 164 |
| raptor | 3.0 / 9 | 1.8 / 18 | 1.1 / 40 | 0.9 / 60 | 0.9 / 60 | 0.9 / 60 | 0.9 / 60 |
| utahraptor | 17.2 / 4 | 10.0 / 9 | 10.0 / 12 | 10.0 / 12 | 10.0 / 12 | 10.0 / 12 | 10.0 / 12 |
| osmolskae | 3.0 / 9 | 1.8 / 18 | 1.1 / 40 | 0.9 / 60 | 0.9 / 60 | 0.9 / 60 | 0.9 / 60 |
| wyvern | 85.7 / 2 | 63.5 / 3 | 63.5 / 3 | 63.5 / 3 | 63.5 / 3 | 63.5 / 3 | 63.5 / 3 |
| hare | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| mountain_hare | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| wood_mouse | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| field_vole | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| white_tailed_deer | 2.7 / 36 | 1.6 / 81 | 1.0 / 130 | 0.8 / 164 | 0.8 / 164 | 0.8 / 164 | 0.8 / 164 |
| red_deer | 15.7 / 16 | 10.0 / 35 | 10.0 / 53 | 10.0 / 53 | 10.0 / 53 | 10.0 / 53 | 10.0 / 53 |
| brown_trout | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| pacu | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| red_fox | 0.7 / 27 | 0.4 / 62 | 0.3 / 111 | 0.2 / 177 | 0.1 / 283 | 0.1 / 453 | 0.0 / 725 |
| arctic_fox | 0.7 / 27 | 0.4 / 62 | 0.3 / 111 | 0.2 / 177 | 0.1 / 283 | 0.1 / 453 | 0.0 / 725 |
| raven | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| protoceratops | 3.9 / 16 | 2.3 / 36 | 1.4 / 81 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 |
| kryptobaatar | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| priscagama | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| reindeer | 3.9 / 16 | 2.3 / 36 | 1.4 / 81 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 |
| coyote | 3.5 / 9 | 2.1 / 18 | 1.3 / 40 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 |
| house_sparrow | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| snow_bunting | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| great_horned_owl | 0.7 / 27 | 0.4 / 62 | 0.3 / 111 | 0.2 / 177 | 0.1 / 283 | 0.1 / 453 | 0.0 / 725 |
| snowy_owl | 0.7 / 27 | 0.4 / 62 | 0.3 / 111 | 0.2 / 177 | 0.1 / 283 | 0.1 / 453 | 0.0 / 725 |
| golden_eagle | 0.7 / 27 | 0.4 / 62 | 0.3 / 111 | 0.2 / 177 | 0.1 / 283 | 0.1 / 453 | 0.0 / 725 |
| savanna_hawk | 0.7 / 27 | 0.4 / 62 | 0.3 / 111 | 0.2 / 177 | 0.1 / 283 | 0.1 / 453 | 0.0 / 725 |
| turkey_vulture | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| herring_gull | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| yacare_caiman | 3.5 / 9 | 2.1 / 18 | 1.3 / 40 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 |
| spectacled_caiman | 3.5 / 9 | 2.1 / 18 | 1.3 / 40 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 |
| grey_wolf | 3.5 / 9 | 2.1 / 18 | 1.3 / 40 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 | 1.0 / 60 |
| jaguar | 5.0 / 8 | 3.0 / 16 | 1.9 / 35 | 1.5 / 52 | 1.5 / 52 | 1.5 / 52 | 1.5 / 52 |
| lungfish | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| lycoptera | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| psittacosaurus | 3.9 / 16 | 2.3 / 36 | 1.4 / 81 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 |
| gastonia | 25.3 / 9 | 14.0 / 20 | 10.6 / 29 | 10.6 / 29 | 10.6 / 29 | 10.6 / 29 | 10.6 / 29 |
| pinacosaurus | 25.3 / 9 | 14.0 / 20 | 10.6 / 29 | 10.6 / 29 | 10.6 / 29 | 10.6 / 29 | 10.6 / 29 |
| iguanodont | 65.9 / 5 | 48.9 / 7 | 48.9 / 7 | 48.9 / 7 | 48.9 / 7 | 48.9 / 7 | 48.9 / 7 |
| crocodyliform | 20.2 / 4 | 11.7 / 9 | 10.0 / 12 | 10.0 / 12 | 10.0 / 12 | 10.0 / 12 | 10.0 / 12 |
| citipati | 3.9 / 16 | 2.3 / 36 | 1.4 / 81 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 | 1.1 / 102 |
| confuciusornis | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| gobipteryx | 0.6 / 104 | 0.3 / 166 | 0.2 / 265 | 0.1 / 425 | 0.1 / 680 | 0.1 / 1087 | 0.0 / 1740 |
| cedarosaurus | 65.9 / 4 | 48.9 / 5 | 48.9 / 5 | 48.9 / 5 | 48.9 / 5 | 48.9 / 5 | 48.9 / 5 |

## Today's weapons against the reference

Damage a second of today's swords and axes (`items.rs`) against the reference hit a second of their age (6 × g(L) at one hit a second).

| weapon | tier | damage × swings | a second | reference | × |
|---|---|---|---|---|---|
| base:wood_sword | 0 | 4 × 1.6 | 6.4 | 7.6 | 0.84 |
| base:stone_sword | 0 | 5 × 1.6 | 8.0 | 7.6 | 1.05 |
| base:iron_sword | 2 | 6 × 1.6 | 9.6 | 19.4 | 0.49 |
| base:gold_sword | 2 | 4 × 1.6 | 6.4 | 19.4 | 0.33 |
| base:diamond_sword | 3 | 7 × 1.6 | 11.2 | 31.1 | 0.36 |
| base:stone_axe | 0 | 9 × 0.8 | 7.2 | 7.6 | 0.95 |
| base:iron_axe | 2 | 9 × 0.9 | 8.1 | 19.4 | 0.42 |
| base:diamond_axe | 3 | 9 × 1 | 9.0 | 31.1 | 0.29 |
| base:wood_axe | 0 | 7 × 0.8 | 5.6 | 7.6 | 0.74 |

## Tames by saddle

A saddle (and barding) of an age carries a tame up to that age's reference level, never past it and never below where its own level puts it (Ronald, 11-06). Its own bonuses (tamed points, imprint ×1.2, bond ×1.05) add at most ×1.6, as gear's grade and marks do. A level-0 utahraptor and wyvern, health / hit, plain and at the ceiling:

| species | no saddle | Stone Age saddle | Bronze Age saddle | Iron Age saddle | Age of Steam saddle | Age of Lightning saddle | Age of the Future saddle | Age of Stars saddle |
|---|---|---|---|---|---|---|---|---|
| utahraptor | 103 / 10 (top 165 / 16) | 131 / 12 (top 209 / 20) | 209 / 20 (top 334 / 31) | 334 / 31 (top 535 / 50) | 535 / 50 (top 856 / 80) | 856 / 80 (top 1370 / 129) | 1370 / 129 (top 2192 / 206) | 2192 / 206 (top 3507 / 330) |
| wyvern | 456 / 15 (top 730 / 24) | 577 / 19 (top 923 / 30) | 923 / 30 (top 1478 / 48) | 1478 / 48 (top 2364 / 77) | 2364 / 77 (top 3783 / 124) | 3783 / 124 (top 6052 / 198) | 6052 / 198 (top 9683 / 317) | 9683 / 317 (top 15494 / 507) |

## The perk web's prices

A unit is +2 % to one family; a small node gives 1, a notable 3-4, a keystone nets 0. Inside a family bonuses add, between families they multiply. A family tops out at ×1.6 (30 units: one age); everything together (gear, web, a tame's bond, food) at ×2.5 (two ages). Families: melee damage, ranged damage, spell damage, defence, health, tame damage, taming speed, craft grade, gathering speed, mining speed, move speed, mana cost. No node exists yet.

## Age keys by path

Moving to the next age takes a key, one thing from each gate (materials and tools, a biome boss, knowledge, infrastructure), by a route for each path. What exists so far:

| age | tech | magic | beasts | not built yet |
|---|---|---|---|---|
| Stone Age | base:pickaxe, base:furnace | - | animals:club, animals:tranquilizer_dart, animals:nest | magic route: a dowsing rod (sensing ley lines); a biome boss's part; knowledge: a ruin's blueprint or lore page |
| Bronze Age | - | - | - | all of it |
| Iron Age | - | - | - | all of it |
| Age of Steam | - | - | - | all of it |
| Age of Lightning | - | - | - | all of it |
| Age of the Future | - | - | - | all of it |
| Age of Stars | - | - | - | all of it |

## What exists, by age and path

Every named item and block, by its tag (`content::RULES`). EM is the cheapest way to have one; "-" not had as an item.

### Stone Age (486)

**tech** (45)

- tool: arrow (2.0), axe (0.48), bow (0.48), hoe (0.70), knife (0.43), pickaxe (0.48), shovel (0.48), spear (0.70), stone_axe (0.60), stone_hoe (0.45), stone_pickaxe (0.60), stone_shovel (0.30), stone_sword (0.40), sword (0.70), wood_axe (0.41), wood_hoe (0.33), wood_pickaxe (0.41), wood_shovel (0.24), wood_sword (0.28)
- armour: leather_boots (9.1), leather_chestplate (18), leather_helmet (11), leather_leggings (16)
- station: furnace (1.2)
- part: part_bar_guard (0.22), part_barbed_point (0.22), part_bearded_axe_head (0.22), part_bowstring (0.22), part_felling_axe_head (0.22), part_grip (0.22), part_haft (0.22), part_hoe_blade (0.22), part_knife_blade (0.22), part_lashing (0.22), part_leaf_point (0.22), part_long_blade (0.22), part_long_haft (0.22), part_pick_head (0.22), part_round_pommel (0.22), part_self_stave (0.22), part_shaft (0.22), part_short_blade (0.22), part_short_stave (0.22), part_shovel_blade (0.22), part_straight_blade (0.22)

**magic** (1)

- sky: eclipse_ash (40)

**beasts** (33)

- ground: animals:amber_ore (-), animals:termite_mound (0.30)
- beast: animals:amber (2.0), animals:amber_block (8.1), animals:brown_trout_egg (4.0), animals:carcass (4.0), animals:cedarosaurus_egg (4.0), animals:citipati_egg (4.0), animals:club (0.40), animals:confuciusornis_egg (4.0), animals:crocodyliform_egg (4.0), animals:gastonia_egg (4.0), animals:gobipteryx_egg (4.0), animals:iguanodont_egg (4.0), animals:lungfish_egg (4.0), animals:lycoptera_egg (4.0), animals:nest (0.45), animals:pacu_egg (4.0), animals:pinacosaurus_egg (4.0), animals:priscagama_egg (4.0), animals:protoceratops_egg (4.0), animals:psittacosaurus_egg (4.0), animals:raptor_egg (4.0), animals:sedative (1.2), animals:spectacled_caiman_egg (4.0), animals:tranquilizer_dart (1.0), animals:utahraptor_egg (4.0), animals:utahraptor_saddle (7.7), animals:velociraptor_osmolskae_egg (4.0), animals:wyvern_egg (4.0), animals:yacare_caiman_egg (4.0)
- technical: animals:gateway (-), animals:termite_mound_upper (-)

**shared** (407)

- ground: animals:red_sand (0.10), animals:red_sandstone (0.15), banded_clay (0.15), banded_clay_coal_ore (-), brain_coral_block (0.15), clay (0.10), coal_ore (-), coarse_dirt (0.10), cobblestone (0.15), coral_rubble (0.15), cream_sandstone (0.15), deep_leaf_litter (0.10), dirt (0.10), fallen_branch (-), glacier_ice (0.15), granite (0.15), grass (0.10), gravel (0.10), ice (0.15), impact_glass (0.10), lake_ice (0.15), leaf_litter (0.10), limestone (0.15), loose_flint (-), loose_rock (-), loose_tektite (-), moss_block (0.10), mossy_cobblestone (0.15), ochre_sandstone (0.15), podzol (0.10), porites_coral_block (0.15), red_sandstone (0.15), sand (0.10), sandstone (0.15), slate (0.15), snow (0.10), snow_layer (0.10), snowy_grass (0.10), staghorn_coral_block (0.15), suevite (0.15), weathered_limestone (0.15)
- wood: animals:araucaria_leaves (0.10), animals:araucaria_log (0.30), animals:araucaria_sapling (0.10), animals:ginkgo_leaves (0.10), animals:ginkgo_log (0.30), animals:ginkgo_sapling (0.10), animals:rotten_log (0.30), animals:scale_conifer_leaves (0.10), animals:scale_conifer_log (0.30), animals:scale_conifer_sapling (0.10), animals:swamp_cypress_leaves (0.10), animals:swamp_cypress_log (0.30), animals:swamp_cypress_sapling (0.10), animals:tempskya_leaves (0.10), animals:tempskya_log (0.30), acacia_branch (0.20), acacia_leaves (0.10), acacia_log (0.30), acacia_planks (0.09), acuri_palm_leaves (0.10), acuri_palm_log (0.30), acuri_palm_planks (0.09), acuri_palm_sapling (0.10), andiroba_branch (0.20), andiroba_leaves (0.10), andiroba_log (0.30), andiroba_planks (0.09), andiroba_sapling (0.10), birch_branch (0.20), birch_leaves (0.10), birch_log (0.30), birch_planks (0.09), brazil_nut_branch (0.20), brazil_nut_buttress (0.30), brazil_nut_leaves (0.10), brazil_nut_log (0.30), brazil_nut_planks (0.09), brazil_nut_sapling (0.10), cacao_leaves (0.10), cacao_log (0.30), cacao_planks (0.09), cacao_sapling (0.10), creosote_leaves (0.10), dogwood_branch (0.20), dogwood_leaves (0.10), dogwood_log (0.30), ipe_branch (0.20), ipe_leaves (0.10), ipe_log (0.30), ipe_planks (0.09), ipe_sapling (0.10), juniper_leaves (0.10), matamata_branch (0.20), matamata_leaves (0.10), matamata_log (0.30), matamata_planks (0.09), matamata_sapling (0.10), mesquite_branch (0.20), mesquite_leaves (0.10), mesquite_log (0.30), mesquite_planks (0.09), mesquite_sapling (0.10), moriche_palm_leaves (0.10), moriche_palm_log (0.30), moriche_palm_planks (0.09), moriche_palm_sapling (0.10), mossy_sequoia_log (0.30), oak_branch (0.20), oak_leaves (0.10), oak_log (0.30), oak_planks (0.09), sapling (0.10), sequoia_leaves (0.10), sequoia_log (0.30), sequoia_planks (0.09), sequoia_sapling (0.10), spruce_leaves (0.10), spruce_log (0.30), spruce_planks (0.09), stripped_acacia_log (0.30), stripped_acuri_palm_log (0.30), stripped_andiroba_log (0.30), stripped_birch_log (0.30), stripped_brazil_nut_log (0.30), stripped_cacao_log (0.30), stripped_dogwood_log (0.30), stripped_ipe_log (0.30), stripped_matamata_log (0.30), stripped_mesquite_log (0.30), stripped_moriche_palm_log (0.30), stripped_oak_log (0.30), stripped_sequoia_log (0.30), stripped_spruce_log (0.30), willow_leaves (0.10)
- plant: animals:cycad (0.10), animals:fern (0.10), animals:flowering_shrub (0.10), animals:horsetail (0.10), animals:water_lily (0.10), alligator_flag (0.10), alpenrose (0.10), alpine_aster (0.10), arctic_coltsfoot (0.10), arctic_poppy (0.10), banana (0.10), beach_plum_bush (0.10), bellflower (0.10), berry_bush (0.10), black_eyed_susan (0.10), blazing_star (0.10), bracket_fungus (0.10), bramble_bush (0.10), brittlebush (0.10), bromeliad (0.10), bunchberry (0.10), butterfly_milkweed (0.10), cattail (0.10), chanterelle (0.10), cloudberry_bush (0.10), clover (0.10), cottongrass (0.10), crowberry_bush (0.10), dandelion (0.10), dead_bush (0.10), desert_marigold (0.10), dune_grass (0.10), dung_pat (0.10), dwarf_birch (0.10), edelweiss (0.10), elkhorn_coral (0.10), fern (0.10), finger_coral (0.10), fireweed (0.10), foxfire (0.10), foxglove (0.10), gentian (0.10), globeflower (0.10), globemallow (0.10), glow_mycena (0.10), goblins_gold (0.10), gold_poppy (0.10), hazel_bush (0.10), heliconia (0.10), honey_fungus (0.10), horned_poppy (0.10), hottentot_fig (0.10), huckleberry_bush (0.10), labrador_tea (0.10), leadplant (0.10), lesser_celandine (0.10), liana (0.10), lily_of_the_valley (0.10), lily_pad (0.10), lingonberry_bush (0.10), lousewort (0.10), lungwort (0.10), lupine (0.10), may_lily (0.10), morning_glory (0.10), ocotillo (0.10), pencil_flower (0.10), philodendron (0.10), porcini (0.10), prickly_pear (0.10), prickly_rose (0.10), purple_coneflower (0.10), pygmy_buttercup (0.10), reed (0.10), reindeer_lichen (0.10), rose (0.10), saxifrage (0.10), sea_daffodil (0.10), sea_fan (0.10), sea_holly (0.10), sea_rocket (0.10), sea_rod (0.10), seagrass (0.10), sedge (0.10), senna (0.10), sensitive_plant (0.10), staghorn_coral (0.10), sweet_woodruff (0.10), sword_fern (0.10), tall_grass (0.10), tall_grass_double (0.10), turtle_grass (0.10), twinflower (0.10), water_hyacinth (0.10), water_primrose (0.10), wild_sunflower (0.10), wood_anemone (0.10), wood_cranesbill (0.10)
- crop: barley (0.20), barley_seeds (0.20), bean_seeds (0.20), beans (0.20), beetroot (0.20), beetroots (0.20), cactus (0.20), carrot (0.20), carrots (0.20), cassava (0.20), chia (0.20), chia_seeds (0.20), corn (0.20), maize (0.20), maize_seeds (0.20), melon (0.20), millet (0.20), millet_seeds (0.20), potato (0.20), potatoes (0.20), pumpkin (0.20), rice (0.20), rice_seeds (0.20), rye (0.20), rye_seeds (0.20), sorghum (0.20), sorghum_seeds (0.20), sugar_cane (0.20), tepary_beans (0.20), wheat (0.20), wheat_seeds (0.20), wild_rice (0.20)
- food: animals:araucaria_cone (0.20), animals:cooked_hare (3.2), animals:cooked_pacu (3.2), animals:cooked_trout (3.2), animals:cooked_venison (3.2), animals:cycad_seed (0.20), animals:ginkgo_seed (0.20), animals:hare (3.0), animals:pacu (3.0), animals:trout (3.0), animals:venison (3.0), acorn (0.20), acuri_nut (0.20), andiroba_seed (0.20), apple (0.20), apple_pie (2.7), baked_potato (0.37), bananas (0.20), beach_plums (0.20), beef (3.0), berries (0.20), brazil_nut (0.20), bread (0.65), cacao_beans (0.25), cacao_pod (0.20), cattail_root (0.20), chicken (3.0), cloudberries (0.20), cooked_beef (3.2), cooked_chicken (3.2), cooked_egg (2.2), cooked_mutton (3.2), cooked_porkchop (3.2), cooked_rice (0.37), cooked_wild_rice (0.37), crowberries (0.20), dried_apple (0.25), dried_berries (0.25), dried_meat (3.0), hazelnut (0.20), huckleberries (0.20), ipe_seed (0.20), kelp (0.20), lingonberries (0.20), matamata_fruit (0.20), melon_seeds (0.03), melon_slice (0.01), mesquite_pod (0.20), moriche_fruit (0.20), mushroom_brown (0.20), mushroom_red (0.20), mutton (3.0), pine_nut (0.20), porkchop (3.0), prickly_pear_fruit (0.20), pumpkin_seeds (0.03), pumpkin_slice (0.01), raspberries (0.20), roasted_cacao (0.42), roasted_corn (0.37), rose_hip (0.20), sequoia_cone (0.20), smoked_meat (3.2), sunflower_seeds (0.20)
- material: bandage (0.14), bone (2.0), book (3.2), bowl (0.08), cloth (0.30), coal (0.33), coal_block (3.0), compost (0.45), dung (2.0), egg (2.0), feather (2.0), flint (0.20), fur (2.0), gunpowder (2.0), hide (2.0), leather (2.5), oak_bark (0.08), paper (0.22), plant_fibre (0.02), rock (0.10), rotten_flesh (2.0), sharp_flake (0.25), sole_leather (2.2), spider_eye (2.0), stick (0.05), string (0.08), sugar (0.25), tektite (5.0)
- clothing: fur_boots (4.1), fur_coat (8.2), fur_hat (4.1), fur_robe (10), fur_trousers (6.2), hide_boots (4.1), hide_coat (8.2), hide_hat (4.1), hide_trousers (6.2), linen_boots (0.73), linen_coat (1.4), linen_hat (0.73), linen_trousers (1.1), rain_cape (13), wool_boots (0.90), wool_cloak (1.3), wool_coat (1.8), wool_hat (0.90), wool_trousers (1.4)
- station: bow_drill (0.23), campfire_laid (0.75), charcoal_clamp (1.1), compost_bin (0.66), drying_rack (0.30), fire_bed (0.22), hand_drill (0.15), tanning_frame (0.58), tanning_pit (0.66)
- building: bed (1.5), bookshelf (10), chest (0.75), crafting_table (0.40), hay_bale (1.9), ladder (0.13), wool (0.38)
- light: torch (0.11)
- technical: campfire (-), farmland (-), grazed_grass (-), lava (-), maize_top (-), melon_stem (-), millet_top (-), paddy_rice (-), pumpkin_stem (-), sorghum_top (-), tombstone (-), water (-), packages:unknown (-), portals:portal (-)

Notes:
- base:water, base:lava: liquids, carried in buckets
- base:farmland, base:grazed_grass, base:*_top, base:*_stem, base:paddy_rice, base:tombstone: growing stages, tilled or grazed ground, the grave a death leaves
- base:campfire: a laid campfire, lit
- base:loose_rock, base:loose_flint, base:loose_tektite, base:fallen_branch: picked up as a rock, flint, tektite or sticks
- base:coal_ore, base:banded_clay_coal_ore: breaks into coal
- animals:amber_ore: breaks into amber
- animals:gateway, animals:termite_mound_upper: the Cretaceous gateway is built of amber blocks; a mound's top
- portals:portal, packages:unknown: a linked world's door; a missing package's stand-in
- base:eclipse_ash: falls in an eclipse
- base:axe, base:pickaxe, base:shovel, base:hoe, base:knife, base:spear, base:sword, base:bow: built from parts: its parts' materials set its real age (`MATERIALS`)
- base:part_*: a part's material sets its real age (`MATERIALS`)
- base:arrow: dropped by skeletons; no recipe yet
- base:furnace: the Stone Age key's infrastructure: what it makes is the Bronze Age's

### Bronze Age (17)

**tech** (16)

- ground: granite_iron_ore (-), iron_ore (-), meteorite (-), red_sandstone_iron_ore (-)
- material: meteoric_iron (10)
- building: blue_stained_glass (0.47), bricks (0.27), glass (0.27), green_stained_glass (0.52), red_stained_glass (0.42), stone (0.32), stone_bricks (0.33)
- light: blue_lamp (1.4), glowstone (1.2), green_lamp (1.4), red_lamp (1.3)

**magic** (1)

- sky: pallasite (20)

Notes:
- base:iron_ore, base:granite_iron_ore, base:red_sandstone_iron_ore, base:meteorite: breaks into raw iron or meteoric iron (a stone pickaxe)
- base:pallasite: a stony-iron meteorite's olivine, found where one fell
- base:meteoric_iron: hammered cold, iron-grade before a player can smelt (Research round 17)

### Iron Age (44)

**tech** (34)

- ground: diamond_ore (-), gold_ore (-), granite_gold_ore (-), redstone_ore (-), star_steel_ore (-)
- material: bucket (3.5), gold_block (29), gold_ingot (3.2), iron_block (11), iron_ingot (1.2), lava_bucket (4.1), raw_gold (3.0), raw_iron (1.0), star_steel_ingot (12), star_steel_lump (12), water_bucket (3.7)
- tool: gold_axe (9.7), gold_hoe (6.5), gold_pickaxe (9.7), gold_shovel (3.3), gold_sword (6.4), iron_axe (3.6), iron_hoe (2.5), iron_pickaxe (3.6), iron_shovel (1.3), iron_sword (2.4)
- armour: gold_boots (13), gold_chestplate (25), gold_helmet (16), gold_leggings (22), iron_boots (4.7), iron_chestplate (9.4), iron_helmet (5.9), iron_leggings (8.2)

**magic** (3)

- light: star_lantern (62)
- sky: peridot (10), starheart (60)

**beasts** (2)

- beast: animals:kibble (1.5), animals:wyvern_saddle (14)

**shared** (5)

- tool: animals:spyglass (2.6)
- station: cooking_pot (8.2), salt (3.9), salted_meat (6.9), stew (3.4)

Notes:
- base:gold_ore, base:granite_gold_ore, base:diamond_ore, base:redstone_ore, base:star_steel_ore: breaks into its ore (an iron pickaxe)
- base:raw_gold, base:gold_ingot, base:gold_block: gold ore needs an iron pickaxe today; the plan casts gold in the Bronze Age
- base:star_lantern: a starheart in iron
- base:cooking_pot, base:stew, base:salt, base:salted_meat: the pot is iron; salt is boiled from a water bucket
- animals:kibble: takes a water bucket (iron)
- animals:wyvern_saddle: a dragon-sized saddle has an iron frame

### Age of Steam (29)

**tech** (28)

- material: diamond (10), diamond_block (90)
- tool: diamond_axe (30), diamond_hoe (20), diamond_pickaxe (30), diamond_shovel (10), diamond_sword (20)
- armour: diamond_boots (40), diamond_chestplate (80), diamond_helmet (50), diamond_leggings (70)
- machine: anchor (9.9), comparator (3.3), copper_lamp (-), dispenser (1.6), floater (10), golden_lamp (13), lever (0.25), observer (3.1), piston (2.6), redstone (0.50), redstone_block (4.6), redstone_lamp (3.2), redstone_torch (0.60), repeater (2.7), slime_block (1.0), sticky_piston (3.7), stone_button (0.37)

**shared** (1)

- ground: obsidian (1.0)

Notes:
- base:copper_lamp: no recipe yet (no copper in the game)
- base:obsidian: a diamond pickaxe
- base:diamond_pickaxe, base:diamond_axe, base:diamond_shovel, base:diamond_hoe, base:diamond_sword: diamond is a tier above steel (`crafting::materials`)
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
| copper | Bronze Age | - |
| bronze | Bronze Age | - |
| wrought_iron | Iron Age | base:iron_ingot |
| meteoric_iron | Bronze Age | base:meteoric_iron |
| steel | Iron Age | - |
| star_steel | Iron Age | base:star_steel_ingot |
| gold | Iron Age | base:gold_ingot |
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

