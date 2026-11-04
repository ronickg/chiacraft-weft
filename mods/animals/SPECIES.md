# Animals: every species' settings

Written from the game's species table (`core/src/species.rs`) by `core/src/sheet.rs`; don't edit it by hand. After changing a species, rewrite it:

    mods/animals/web/build.sh && node mods/animals/web/sheet.mjs

`node mods/animals/web/test.mjs` fails while it is out of date. Time is in in-game days (24 000 ticks, 20 minutes of play); "real" is the real animal's. Breeding numbers are derived from the real facts (`core/src/biology.rs`) unless a species sets one itself (marked **set**). Why the numbers are what they are: `notes/ANIMALS.md`. Coat genes: `genetics/<species>.json` and `genetics/core.json`.

## At a glance

| Species | Scientific name | Rarity | Size | Height | Mass | Lives in | Diet | Taming | Carrying | Grown in | Lives | Litter | Room |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| dog (chihuahua) | *Canis familiaris* | common | small | 0.2 m | 2.5 kg | pack | carnivore | passive | 1.25 d | 3.5 d | 49.9 d | 1-4 | 17 |
| cat | *Felis catus* | common | small | 0.24 m | 4.5 kg | solitary | carnivore | passive | 1.28 d | 2.99 d | 49.9 d | 1-4 | 20 |
| capybara | *Hydrochoerus hydrochaeris* | common | medium | 0.56 m | 50 kg | herd | herbivore | passive | 2.28 d | 5.34 d | 30.7 d | 2-6 | 24 |
| raptor | *Velociraptor mongoliensis* | uncommon | medium | 0.5 m | 15 kg | solitary | carnivore | knockout | eggs, hatch 1.41 d | 7.54 d | 76.7 d | 1-4 | 10 |
| utahraptor | *Utahraptor ostrommaysi* | rare | large | 1.5 m | 300 kg | pack | carnivore | knockout | eggs, hatch 2.28 d | 12.3 d | 153.3 d | 1-3 | 7 |
| osmolskae | *Velociraptor osmolskae* | uncommon | medium | 0.5 m | 15 kg | solitary | carnivore | knockout | eggs, hatch 1.41 d | 7.54 d | 76.7 d | 1-4 | 10 |
| wyvern | *Wyverna montana* | legendary | large | 3.9 m | 5000 kg | solitary | carnivore | knockout | eggs, hatch 5.47 d | 32.7 d | 421.6 d | 1 | 8 |
| hare | *Lepus europaeus* | common | small | 0.3 m | 3.8 kg | solitary | herbivore | wild | 0.92 d | 2.81 d | 19.2 d | 1-4 | 26 |
| mountain_hare | *Lepus timidus* | common | small | 0.3 m | 3 kg | solitary | herbivore | wild | 1.05 d | 3.5 d | 19.2 d | 1-4 | 26 |
| wood_mouse | *Apodemus sylvaticus* | common | small | 0.05 m | 0.023 kg | solitary | herbivore | wild | 0.63 d | 3 d | 16 d | 2-6 | 30 |
| field_vole | *Microtus agrestis* | common | small | 0.05 m | 0.035 kg | solitary | herbivore | wild | 0.55 d | 3 d | 16 d | 2-5 | 30 |
| white_tailed_deer | *Odocoileus virginianus* | common | large | 0.95 m | 70 kg | herd | herbivore | wild | 2.98 d | 5.28 d | 38.3 d | 1-3 | 15 |
| red_deer | *Cervus elaphus* | uncommon | large | 0.95 m | 160 kg | herd | herbivore | wild | 3.52 d | 6.51 d | 57.5 d | 1 | 12 |
| brown_trout | *Salmo trutta* | uncommon | small | 0.37 m | 0.5 kg | solitary | carnivore | wild | eggs, hatch 0.97 d | 8.65 d | 46 d | 3-8 | 26 |
| pacu | *Piaractus mesopotamicus* | uncommon | small | 0.37 m | 4 kg | herd | omnivore | wild | eggs, hatch 0.06 d | 8.65 d | 57.5 d | 2-8 | 26 |
| red_fox | *Vulpes vulpes* | common | small | 0.38 m | 5.5 kg | pack | carnivore | wild | 1.08 d | 3.5 d | 19.2 d | 2-7 | 24 |
| arctic_fox | *Vulpes lagopus* | common | small | 0.38 m | 3.5 kg | pack | carnivore | wild | 1.08 d | 3.5 d | 19.2 d | 3-10 | 24 |
| raven | *Corvus corax* | uncommon | small | 0.34 m | 1.2 kg | solitary | carnivore | wild | 0.53 d | 6.51 d | 46 d | 2-6 | 17 |
| protoceratops | *Protoceratops andrewsi* | common | medium | 0.6 m | 83 kg | herd | herbivore | wild | eggs, hatch 1.54 d | 6 d | 53.7 d | 3-8 | 20 |
| kryptobaatar | *Kryptobaatar dashzevegi* | common | small | 0.05 m | 0.04 kg | solitary | omnivore | wild | 0.53 d | 3 d | 16 d | 1-5 | 30 |
| priscagama | *Priscagama gobiensis* | common | small | 0.04 m | 0.03 kg | solitary | carnivore | wild | eggs, hatch 1.21 d | 4.01 d | 23 d | 2-6 | 24 |
| reindeer | *Rangifer tarandus* | common | large | 1.05 m | 120 kg | herd | herbivore | wild | 3.31 d | 4.58 d | 46 d | 1 | 20 |
| coyote | *Canis latrans* | common | medium | 0.61 m | 13 kg | pack | carnivore | wild | 1.25 d | 3.5 d | 23 d | 2-8 | 22 |
| house_sparrow | *Passer domesticus* | common | small | 0.1 m | 0.03 kg | herd | omnivore | wild | 0.36 d | 2.75 d | 14 d | 2-6 | 30 |
| snow_bunting | *Plectrophenax nivalis* | common | small | 0.1 m | 0.035 kg | herd | omnivore | wild | 0.37 d | 2.75 d | 14 d | 2-7 | 30 |
| great_horned_owl | *Bubo virginianus* | uncommon | small | 0.45 m | 1.4 kg | solitary | carnivore | wild | 0.77 d | 6.51 d | 49.8 d | 1-3 | 16 |
| snowy_owl | *Bubo scandiacus* | uncommon | small | 0.45 m | 2 kg | solitary | carnivore | wild | 0.75 d | 6.51 d | 38.3 d | 2-8 | 15 |
| golden_eagle | *Aquila chrysaetos* | uncommon | medium | 0.6 m | 4.5 kg | solitary | carnivore | wild | 0.94 d | 8.33 d | 76.7 d | 1-3 | 13 |
| savanna_hawk | *Buteogallus meridionalis* | common | small | 0.6 m | 0.85 kg | solitary | carnivore | wild | 0.87 d | 6.51 d | 46 d | 1 | 18 |
| turkey_vulture | *Cathartes aura* | common | small | 0.5 m | 1.4 kg | herd | carnivore | wild | 0.89 d | 4.58 d | 61.3 d | 1-3 | 24 |
| herring_gull | *Larus argentatus* | common | small | 0.45 m | 1 kg | herd | carnivore | wild | 0.7 d | 4.58 d | 46 d | 1-4 | 24 |
| yacare_caiman | *Caiman yacare* | common | large | 2.2 m | 35 kg | herd | carnivore | wild | eggs, hatch 1.4 d | 6.25 d | 75 d | 4-8 | 26 |
| spectacled_caiman | *Caiman crocodilus* | common | large | 2.2 m | 25 kg | herd | carnivore | wild | eggs, hatch 1.4 d | 6.25 d | 75 d | 4-8 | 26 |
| grey_wolf | *Canis lupus* | uncommon | medium | 0.8 m | 42 kg | pack | carnivore | passive | 1.25 d | 6.07 d | 46 d | 2-8 | 16 |
| jaguar | *Panthera onca* | uncommon | large | 0.7 m | 85 kg | solitary | carnivore | knockout | 1.77 d | 7.54 d | 65.1 d | 1-2 | 12 |
| lungfish | *Ceratodus kempae* | uncommon | small | 0.37 m | 3 kg | solitary | carnivore | wild | eggs, hatch 0.72 d | 10 d | 76.7 d | 1-4 | 14 |
| lycoptera | *Lycoptera davidi* | common | small | 0.37 m | 0.02 kg | herd | carnivore | wild | eggs, hatch 0.19 d | 2.92 d | 15.3 d | 3-11 | 30 |
| psittacosaurus | *Psittacosaurus lujiatunensis* | common | medium | 0.5 m | 25 kg | herd | herbivore | wild | eggs, hatch 1.43 d | 6 d | 42.2 d | 3-8 | 20 |
| gastonia | *Gastonia burgei* | common | large | 1.25 m | 1900 kg | herd | herbivore | wild | eggs, hatch 1.77 d | 6 d | 95.8 d | 1-4 | 12 |
| pinacosaurus | *Pinacosaurus grangeri* | common | large | 1.25 m | 1900 kg | herd | herbivore | wild | eggs, hatch 1.77 d | 6 d | 95.8 d | 1-4 | 12 |
| iguanodont | *Iguanacolossus fortis* | common | large | 2.2 m | 4000 kg | herd | herbivore | wild | eggs, hatch 1.9 d | 6 d | 115 d | 1-3 | 12 |
| crocodyliform | *Neosuchia indet.* | uncommon | large | 2.2 m | 150 kg | solitary | carnivore | wild | eggs, hatch 1.5 d | 6.25 d | 75 d | 4-8 | 20 |
| citipati | *Citipati osmolskae* | common | medium | 0.9 m | 90 kg | solitary | herbivore | wild | eggs, hatch 1.36 d | 6 d | 57.5 d | 2-6 | 18 |
| confuciusornis | *Confuciusornis sanctus* | common | small | 0.3 m | 0.5 kg | herd | omnivore | wild | 0.7 d | 4.58 d | 38.3 d | 1-4 | 24 |
| gobipteryx | *Gobipteryx minuta* | uncommon | small | 0.3 m | 0.3 kg | herd | omnivore | wild | 0.7 d | 6.25 d | 38.3 d | 1-4 | 24 |
| cedarosaurus | *Cedarosaurus weiskopfae* | uncommon | large | 4.1 m | 10000 kg | herd | herbivore | wild | eggs, hatch 2.03 d | 6.67 d | 153.3 d | 1-2 | 10 |

## Stats at level 1

Damage and speed are each kind's own (1 = 100%).

| Species | health | stamina | food | damage | speed | weight | oxygen | torpor | fortitude | hunger (s a point) |
|---|---|---|---|---|---|---|---|---|---|---|
| dog (chihuahua) | 8 | 100 | 80 | 2 | 1 | 40 | 100 | 60 | 10 | 30 |
| cat | 6 | 80 | 60 | 1 | 1 | 30 | 80 | 40 | 10 | 40 |
| capybara | 12 | 150 | 200 | 1 | 1 | 150 | 400 | 150 | 20 | 25 |
| raptor | 14 | 150 | 150 | 4 | 1 | 140 | 150 | 200 | 15 | 20 |
| utahraptor | 60 | 300 | 400 | 10 | 1 | 400 | 200 | 700 | 30 | 12 |
| osmolskae | 14 | 150 | 150 | 4 | 1 | 140 | 150 | 200 | 15 | 20 |
| wyvern | 400 | 800 | 2000 | 30 | 1 | 1200 | 400 | 3000 | 80 | 5 |
| hare | 5 | 120 | 40 | 0.5 | 1 | 15 | 60 | 20 | 5 | 30 |
| mountain_hare | 5 | 120 | 40 | 0.5 | 1 | 13 | 60 | 20 | 5 | 30 |
| wood_mouse | 2 | 60 | 15 | 0.2 | 1 | 2 | 30 | 5 | 2 | 30 |
| field_vole | 2 | 60 | 15 | 0.2 | 1 | 3 | 30 | 5 | 2 | 30 |
| white_tailed_deer | 30 | 200 | 120 | 2 | 1 | 70 | 80 | 60 | 15 | 15 |
| red_deer | 45 | 250 | 180 | 3 | 1 | 120 | 80 | 90 | 20 | 15 |
| brown_trout | 3 | 60 | 20 | 0.3 | 1 | 5 | 400 | 10 | 3 | 30 |
| pacu | 6 | 80 | 40 | 0.5 | 1 | 20 | 400 | 15 | 4 | 25 |
| red_fox | 10 | 120 | 60 | 2 | 1 | 30 | 80 | 40 | 10 | 30 |
| arctic_fox | 8 | 120 | 60 | 1.6 | 1 | 22 | 80 | 35 | 12 | 30 |
| raven | 6 | 120 | 40 | 1 | 1 | 10 | 60 | 20 | 5 | 30 |
| protoceratops | 35 | 200 | 120 | 2 | 1 | 90 | 80 | 70 | 18 | 15 |
| kryptobaatar | 2 | 60 | 15 | 0.2 | 1 | 2 | 30 | 5 | 2 | 30 |
| priscagama | 2 | 60 | 12 | 0.2 | 1 | 2 | 40 | 5 | 2 | 40 |
| reindeer | 40 | 260 | 160 | 3 | 1 | 110 | 80 | 80 | 25 | 15 |
| coyote | 18 | 150 | 80 | 3.5 | 1 | 45 | 80 | 60 | 12 | 30 |
| house_sparrow | 2 | 60 | 15 | 0.2 | 1 | 1 | 40 | 5 | 2 | 20 |
| snow_bunting | 2 | 60 | 15 | 0.2 | 1 | 1.2 | 40 | 5 | 3 | 20 |
| great_horned_owl | 8 | 100 | 40 | 2.5 | 1 | 14 | 60 | 20 | 6 | 30 |
| snowy_owl | 10 | 110 | 45 | 3 | 1 | 20 | 60 | 24 | 8 | 30 |
| golden_eagle | 16 | 150 | 60 | 4 | 1 | 30 | 60 | 40 | 10 | 30 |
| savanna_hawk | 8 | 110 | 40 | 2 | 1 | 10 | 60 | 20 | 6 | 30 |
| turkey_vulture | 10 | 140 | 50 | 1 | 1 | 14 | 60 | 24 | 10 | 40 |
| herring_gull | 8 | 130 | 40 | 1.5 | 1 | 10 | 70 | 20 | 8 | 30 |
| yacare_caiman | 30 | 160 | 120 | 5 | 1 | 80 | 300 | 60 | 30 | 60 |
| spectacled_caiman | 26 | 150 | 100 | 4.5 | 1 | 60 | 300 | 50 | 26 | 60 |
| grey_wolf | 30 | 220 | 120 | 6 | 1 | 60 | 80 | 90 | 20 | 40 |
| jaguar | 45 | 260 | 200 | 9 | 1 | 120 | 160 | 300 | 25 | 20 |
| lungfish | 8 | 90 | 40 | 0.6 | 1 | 12 | 600 | 20 | 6 | 30 |
| lycoptera | 1 | 40 | 8 | 0.1 | 1 | 1 | 300 | 4 | 1 | 30 |
| psittacosaurus | 22 | 160 | 80 | 1.5 | 1.1 | 45 | 70 | 45 | 14 | 15 |
| gastonia | 120 | 300 | 400 | 6 | 0.8 | 600 | 200 | 400 | 60 | 12 |
| pinacosaurus | 120 | 300 | 400 | 6 | 0.8 | 600 | 200 | 400 | 60 | 12 |
| iguanodont | 150 | 400 | 600 | 8 | 1 | 800 | 250 | 500 | 50 | 12 |
| crocodyliform | 45 | 180 | 160 | 7 | 1 | 120 | 400 | 90 | 40 | 60 |
| citipati | 40 | 220 | 120 | 3 | 1.2 | 80 | 90 | 80 | 20 | 15 |
| confuciusornis | 6 | 120 | 30 | 1 | 1 | 6 | 60 | 15 | 6 | 30 |
| gobipteryx | 6 | 120 | 30 | 1 | 1 | 6 | 60 | 15 | 6 | 30 |
| cedarosaurus | 300 | 600 | 1200 | 10 | 0.8 | 2000 | 400 | 1200 | 80 | 10 |

## The food web

Each biome's web, from the species' habitats and diets and the world's plant food (`core/src/web.rs`). Plant food is how much of any plant eater's food grows or is found there in each season, against fresh growth at its best (`vitals::forage`); the farm animals' wild kinds graze every overworld land biome. Grazing: in the best season for grass, what its grazers eat a day (as many of each as live together, `crowd`, by how common there) against what its tall grass grows back within the crowd's radius.

| Biome | Plant food (spring, summer, autumn, winter) | Grazing (best season) | Plant eaters | Hunters | Carrion |
|---|---|---|---|---|---|
| base:plains | 40%, 86%, 46%, 40% | summer: eaten 5% of the regrowth | hare, wood_mouse, white_tailed_deer, house_sparrow | dog (chihuahua), cat, red_fox, raven, coyote, great_horned_owl, golden_eagle, herring_gull, grey_wolf | dog (chihuahua), cat, brown_trout, red_fox, raven, coyote, golden_eagle, turkey_vulture, herring_gull, grey_wolf |
| base:forest | 40%, 73%, 41%, 40% | summer: eaten 7% of the regrowth | hare, wood_mouse, white_tailed_deer, red_deer, house_sparrow | dog (chihuahua), cat, red_fox, raven, coyote, great_horned_owl, golden_eagle, grey_wolf | dog (chihuahua), cat, brown_trout, red_fox, raven, coyote, golden_eagle, turkey_vulture, grey_wolf |
| base:birch_forest | 40%, 62%, 40%, 40% | summer: eaten 13% of the regrowth | mountain_hare, field_vole, red_deer | dog (chihuahua), cat, red_fox, raven, great_horned_owl, golden_eagle, grey_wolf | dog (chihuahua), cat, brown_trout, red_fox, raven, golden_eagle, grey_wolf |
| base:taiga | 40%, 93%, 40%, 40% | summer: eaten 20% of the regrowth | mountain_hare, field_vole, reindeer, snow_bunting | cat, red_fox, arctic_fox, raven, great_horned_owl, snowy_owl, golden_eagle, grey_wolf | cat, brown_trout, red_fox, arctic_fox, raven, golden_eagle, grey_wolf |
| base:snowy_plains | 40%, 62%, 40%, 40% | - | mountain_hare, field_vole, reindeer, snow_bunting | arctic_fox, raven, snowy_owl, golden_eagle, grey_wolf | arctic_fox, raven, golden_eagle, grey_wolf |
| base:desert | 40%, 78%, 55%, 40% | - | hare, wood_mouse, white_tailed_deer, house_sparrow | dog (chihuahua), cat, red_fox, raven, coyote, great_horned_owl, golden_eagle, jaguar | dog (chihuahua), cat, red_fox, raven, coyote, golden_eagle, turkey_vulture |
| base:savanna | 43%, 100%, 100%, 40% | summer: eaten 2% of the regrowth | capybara, hare, wood_mouse, white_tailed_deer | dog (chihuahua), cat, great_horned_owl, savanna_hawk, yacare_caiman, spectacled_caiman, jaguar | dog (chihuahua), cat, turkey_vulture |
| base:swamp | 40%, 100%, 96%, 40% | summer: eaten 8% of the regrowth | capybara, field_vole, white_tailed_deer, house_sparrow | great_horned_owl, savanna_hawk, herring_gull, yacare_caiman, spectacled_caiman, jaguar | turkey_vulture, herring_gull |
| base:mountains | 60%, 95%, 76%, 40% | summer: eaten 27% of the regrowth | mountain_hare, field_vole, red_deer, reindeer, snow_bunting | red_fox, raven, coyote, great_horned_owl, snowy_owl, golden_eagle, grey_wolf | brown_trout, red_fox, raven, coyote, golden_eagle, turkey_vulture, grey_wolf |
| base:sequoia_grove | 40%, 40%, 40%, 40% | summer: eaten 80% of the regrowth | mountain_hare, field_vole, red_deer | red_fox, raven, coyote, great_horned_owl, golden_eagle, grey_wolf | red_fox, raven, coyote, golden_eagle, turkey_vulture, grey_wolf |
| base:beach | 40%, 40%, 50%, 40% | - | hare, wood_mouse, house_sparrow | red_fox, raven, herring_gull | red_fox, raven, herring_gull |
| animals:red_dunes | 40%, 40%, 40%, 40% | - | protoceratops, pinacosaurus, citipati, gobipteryx | raptor, osmolskae, wyvern | raptor, osmolskae, wyvern |
| animals:scrubland | 40%, 43%, 40%, 40% | - | protoceratops, psittacosaurus, gastonia, pinacosaurus, citipati, gobipteryx | raptor, utahraptor, wyvern | raptor, utahraptor, wyvern |
| animals:floodplain | 40%, 59%, 63%, 40% | - | psittacosaurus, gastonia, iguanodont, cedarosaurus | utahraptor, crocodyliform | utahraptor, crocodyliform |
| animals:fern_forest | 40%, 70%, 45%, 40% | - | psittacosaurus, gastonia, iguanodont, cedarosaurus | utahraptor, wyvern, crocodyliform | utahraptor, wyvern, crocodyliform |
| animals:crags | 40%, 66%, 65%, 40% | - | pinacosaurus, gobipteryx | wyvern | wyvern |

### Gaps in the web

What the webs still lack, and what the plan fills it with (`web::WAITING`):

- herring_gull: no prey in base:ocean: the open sea's web: its fish (later; the gull hunts on the beach meanwhile)

## dog (chihuahua)

*Canis familiaris*, Mammalia › Carnivora › Canidae › Caninae. Model `chihuahua`, entity `animals:chihuahua`.

| Where it lives | how common there |
|---|---|
| `base:plains` | commonest |
| `base:savanna` | 60% |
| `base:desert` | 40% |
| `base:forest` | 30% |
| `base:birch_forest` | 30% |

Spawns by day on grass, snow or sand, in groups of 2-3 (pack), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.2 m at the shoulder |
| males against females | 1.04x as tall |
| size spread (an individual's, either way) | 20% |
| newborn's size | 100% of grown (its own young model) |

| Life | real | in the game |
|---|---|---|
| mass | 2.5 kg | |
| carries her young | 63 d | 1.25 d |
| litter or clutch | 3 on average | 1-4 |
| between litters | 180 d | rests 2.5 d |
| breeds in | all year |  |
| comfortable in | 7 to 29 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 300 d | 3.5 d |
| lives | 13 y | 49.9 d, give or take a fifth; an elder from 80% of it |
| lives in | pack | room for 17 within 48 blocks |

**Balance** (common): a generation 4.7 d; a line four deep in 14 d (5 h of play); 6.7 young per mother in 10 days, 24 in her life.

- ✓ fourth generation: 14.2 days (want 10-25 days (common))
- ✓ grown most of its life: grows 3.5 of 50 days (want under a fifth)
- ✓ young in a life: 24.2 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 17 (want 3-30)

| Taming and food | |
|---|---|
| how | passive |
| diet | carnivore |
| eats | meat 40%, carrion 20%, insects 15%, fruit 15%, seeds 10% |
| hunts | 0.8%-80% of its mass alone (0.02-2 kg); at home: wood_mouse, field_vole, brown_trout, raven, house_sparrow, great_horned_owl, savanna_hawk, turkey_vulture, herring_gull, base:chicken |
| a meal that fills it | 0.25 kg; its biggest prey at home fills it in one kill |
| drinks | 100% of its 0.226 L a day; a drink lasts 17 min, thirsty after 8 |
| feedings to tame at level 1 | 4 (2.5x at level 150) |
| scared by a walking player within | 3 blocks (twice that running) |
| eats (food points) | `base:beef` 10; `base:porkchop` 10; `base:chicken` 8; `base:mutton` 8; `base:cooked_beef` 20, favorite; `base:cooked_porkchop` 20, favorite; `base:cooked_chicken` 16, favorite; `base:cooked_mutton` 16, favorite; `base:bone` 5, favorite; `animals:kibble` 30, kibble |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: bite |
| reach past its body | 0.8 blocks |
| signature | alarm |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.3 |

## cat

*Felis catus*, Mammalia › Carnivora › Felidae › Felinae. Model `cat`, entity `animals:cat`.

| Where it lives | how common there |
|---|---|
| `base:plains` | commonest |
| `base:forest` | 60% |
| `base:birch_forest` | 60% |
| `base:savanna` | 50% |
| `base:desert` | 40% |
| `base:taiga` | 20% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.24 m at the shoulder |
| males against females | 1.08x as tall |
| size spread (an individual's, either way) | 8% |
| newborn's size | 35% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 4.5 kg | |
| carries her young | 65 d | 1.28 d |
| litter or clutch | 3 on average | 1-4 |
| between litters | 200 d | rests 2.78 d |
| breeds in | spring, summer and autumn, most in spring | out of it, only a tame well cared for |
| comfortable in | 5 to 32 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 240 d | 2.99 d |
| lives | 13 y | 49.9 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 20 **set** within 48 blocks |

**Balance** (common): a generation 4.3 d; a line four deep in 13 d (4 h of play); 6.2 young per mother in 10 days, 23 in her life.

- ✓ fourth generation: 12.8 days (want 10-25 days (common))
- ✓ grown most of its life: grows 3.0 of 50 days (want under a fifth)
- ✓ young in a life: 22.7 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 20 (want 3-30)

| Taming and food | |
|---|---|
| how | passive |
| diet | carnivore |
| eats | meat 80%, insects 10%, carrion 5%, fish 5% |
| hunts | 0.5%-40% of its mass alone (0.02-2 kg); at home: wood_mouse, field_vole, brown_trout, raven, house_sparrow, snow_bunting, great_horned_owl, savanna_hawk, turkey_vulture, herring_gull, base:chicken |
| a meal that fills it | 0.45 kg; its biggest prey at home fills it in one kill |
| drinks | 50% of its 0.383 L a day; a drink lasts 35 min, thirsty after 18 |
| feedings to tame at level 1 | 5 (2.5x at level 150) |
| scared by a walking player within | 6 blocks (twice that running) |
| eats (food points) | `base:beef` 8; `base:mutton` 8; `base:egg` 6; `base:cooked_beef` 14; `base:cooked_mutton` 14; `base:chicken` 10, favorite; `base:cooked_chicken` 16, favorite; `animals:kibble` 30, kibble |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: swipe, bite |
| reach past its body | 0.8 blocks |
| signature | gifts |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.5 |

## capybara

*Hydrochoerus hydrochaeris*, Mammalia › Rodentia › Caviidae › Hydrochoerinae. Model `capybara`, entity `animals:capybara`.

| Where it lives | how common there |
|---|---|
| `base:swamp` | commonest |
| `base:savanna` | 80% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | medium |
| height (grown, average) | 0.56 m at the shoulder |
| males against females | 0.97x as tall |
| size spread (an individual's, either way) | 8% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 50 kg | |
| carries her young | 140 d | 2.28 d |
| litter or clutch | 4 on average | 2-6 |
| between litters | 1 y | rests 5.07 d |
| breeds in | all year, most in spring |  |
| comfortable in | 12 to 32 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 1.5 y | 5.34 d |
| lives | 8 y | 30.7 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 24 **set** within 48 blocks |

**Balance** (common): a generation 7.6 d; a line four deep in 23 d (8 h of play); 5.4 young per mother in 10 days, 10 in her life.

- ✓ fourth generation: 22.9 days (want 10-25 days (common))
- ✓ grown most of its life: grows 5.3 of 31 days (want under a fifth)
- ✓ young in a life: 10.4 (want 6-40)
- ✓ litter: 2-6 (want 1-12, fewest first)
- ✓ room near: 24 (want 3-30)

| Taming and food | |
|---|---|
| how | passive |
| diet | herbivore |
| eats | grass 70%, water plants 20%, seeds 5%, fruit 5% |
| a meal that fills it | 5 kg |
| drinks | 100% of its 3.347 L a day; a drink lasts 22 min, thirsty after 11 |
| its food grows at home (spring/summer/autumn/winter) | swamp 40/100/96/40%, savanna 43/100/100/40% |
| feedings to tame at level 1 | 3 (2.5x at level 150) |
| scared by a walking player within | nothing |
| eats (food points) | `base:wheat` 10; `base:wheat_seeds` 3; `base:bread` 15; `base:hay_bale` 90; `base:apple` 15, favorite; `base:sugar_cane` 12, favorite; `base:berries` 8, favorite; `animals:kibble` 30, kibble |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.8 blocks |
| signature | calm |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.8 |

## raptor

*Velociraptor mongoliensis*, Dinosauria › Theropoda › Dromaeosauridae › Velociraptorinae. Model `raptor`, entity `animals:raptor`.

| Where it lives | how common there |
|---|---|
| `animals:red_dunes` | commonest |
| `animals:scrubland` | 40% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | medium |
| height (grown, average) | 0.5 m at the hip |
| males against females | 1x as tall |
| size spread (an individual's, either way) | 8% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 15 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 74 d | 1.41 d |
| litter or clutch | 3 on average | 1-4 |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 12 to 40 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 2.5 y | 7.54 d |
| lives | 20 y | 76.7 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 10 within 48 blocks |

**Balance** (uncommon): a generation 9.2 d; a line four deep in 28 d (9 h of play); 4.7 young per mother in 10 days, 25 in her life.

- ✓ fourth generation: 27.6 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 7.5 of 77 days (want under a fifth)
- ✓ young in a life: 25.3 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 10 (want 3-30)

| Taming and food | |
|---|---|
| how | knockout |
| diet | carnivore |
| eats | meat 70%, carrion 25%, insects 5% |
| hunts | 0.2%-600% of its mass alone (0.03-90 kg); at home: protoceratops, kryptobaatar, psittacosaurus, citipati, gobipteryx |
| a meal that fills it | 1.5 kg; its biggest prey at home fills it in one kill |
| drinks | 50% of its 1.133 L a day; a drink lasts 40 min, thirsty after 20 |
| feedings to tame at level 1 | 10 (2.5x at level 150) |
| scared by a walking player within | 8 blocks (twice that running) |
| eats (food points) | `base:chicken` 10; `base:mutton` 10; `base:porkchop` 12; `base:beef` 15, favorite; `animals:kibble` 30, kibble |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: bite, kick |
| reach past its body | 0.8 blocks |
| signature | pack |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 1 |

## utahraptor

*Utahraptor ostrommaysi*, Dinosauria › Theropoda › Dromaeosauridae › Dromaeosaurinae. Model `utahraptor`, entity `animals:utahraptor`.

| Where it lives | how common there |
|---|---|
| `animals:floodplain` | commonest |
| `animals:scrubland` | 50% |
| `animals:fern_forest` | 30% |

Spawns by day on grass, snow or sand, in groups of 2-3 (pack), at 0.4% of spawn attempts that pick it where commonest (rare).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 1.5 m at the hip |
| males against females | 1x as tall |
| size spread (an individual's, either way) | 8% |
| newborn's size | 15% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 300 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 140 d | 2.28 d |
| litter or clutch | 2 on average | 1-3 |
| between litters | 1.5 y | rests 7.62 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 6 to 36 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 4.9 y | 12.3 d |
| lives | 40 y | 153.3 d, give or take a fifth; an elder from 80% of it |
| lives in | pack | room for 7 within 48 blocks |

**Balance** (rare): a generation 14.8 d; a line four deep in 44 d (15 h of play); 2.5 young per mother in 10 days, 28 in her life.

- ✓ fourth generation: 44.3 days (want 35-70 days (rare))
- ✓ grown most of its life: grows 12.3 of 153 days (want under a fifth)
- ✓ young in a life: 28.1 (want 6-40)
- ✓ litter: 1-3 (want 1-12, fewest first)
- ✓ room near: 7 (want 3-30)

| Taming and food | |
|---|---|
| how | knockout |
| diet | carnivore |
| eats | meat 80%, carrion 15%, fish 5% |
| hunts | 1.0%-300% of its mass alone (3.00-900 kg); at home: protoceratops, lungfish, psittacosaurus, gastonia, pinacosaurus, iguanodont, crocodyliform, citipati |
| a meal that fills it | 30 kg; its biggest prey at home fills it in one kill |
| drinks | 80% of its 16.79 L a day; a drink lasts 34 min, thirsty after 17 |
| feedings to tame at level 1 | 18 (2.5x at level 150) |
| scared by a walking player within | 10 blocks (twice that running) |
| eats (food points) | `base:chicken` 10; `base:porkchop` 15; `base:beef` 20, favorite; `base:mutton` 15, favorite; `animals:kibble` 30, kibble |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: bite, kick |
| reach past its body | 1.2 blocks |
| signature | mount |
| ridden | seat 1.55 up, 0 forward |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.8 |

## osmolskae

*Velociraptor osmolskae*, Dinosauria › Theropoda › Dromaeosauridae › Velociraptorinae. Model `raptor`, entity `animals:osmolskae`.

| Where it lives | how common there |
|---|---|
| `animals:red_dunes` | commonest |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | medium |
| height (grown, average) | 0.5 m at the hip |
| males against females | 1x as tall |
| size spread (an individual's, either way) | 8% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 15 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 74 d | 1.41 d |
| litter or clutch | 3 on average | 1-4 |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 12 to 40 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 2.5 y | 7.54 d |
| lives | 20 y | 76.7 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 10 within 48 blocks |

**Balance** (uncommon): a generation 9.2 d; a line four deep in 28 d (9 h of play); 4.7 young per mother in 10 days, 25 in her life.

- ✓ fourth generation: 27.6 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 7.5 of 77 days (want under a fifth)
- ✓ young in a life: 25.3 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 10 (want 3-30)

| Taming and food | |
|---|---|
| how | knockout |
| diet | carnivore |
| eats | meat 70%, carrion 25%, insects 5% |
| hunts | 0.2%-600% of its mass alone (0.03-90 kg); at home: protoceratops, kryptobaatar, citipati, gobipteryx |
| a meal that fills it | 1.5 kg; its biggest prey at home fills it in one kill |
| drinks | 50% of its 1.133 L a day; a drink lasts 40 min, thirsty after 20 |
| feedings to tame at level 1 | 10 (2.5x at level 150) |
| scared by a walking player within | 8 blocks (twice that running) |
| eats (food points) | `base:chicken` 10; `base:mutton` 10; `base:porkchop` 12; `base:beef` 15, favorite; `animals:kibble` 30, kibble |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: bite, kick |
| reach past its body | 0.8 blocks |
| signature | pack |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 1 |

## wyvern

*Wyverna montana*, Fantasy › Draconia › Wyvernidae › Wyverninae. Model `wyvern`, entity `animals:wyvern`.

| Where it lives | how common there |
|---|---|
| `animals:crags` | commonest |
| `animals:fern_forest` | 33% |
| `animals:red_dunes` | 33% |
| `animals:scrubland` | 33% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 0.06% of spawn attempts that pick it where commonest (legendary).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 3.9 m at the shoulder |
| males against females | 1.08x as tall |
| size spread (an individual's, either way) | 12% |
| newborn's size | 8% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 5000 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 1.2 y | 5.47 d |
| litter or clutch | 1 on average | 1 |
| between litters | 3 y | rests 15.2 d |
| breeds in | early in the wet season | out of it, only a tame well cared for |
| comfortable in | -10 to 30 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 20 y | 32.7 d |
| lives | 110 y | 421.6 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 8 **set** within 48 blocks |

**Balance** (legendary): a generation 38.4 d; a line four deep in 115 d (38 h of play); 0.6 young per mother in 10 days, 20 in her life.

- ✓ fourth generation: 115.1 days (want 80-160 days (legendary))
- ✓ grown most of its life: grows 32.7 of 422 days (want under a fifth)
- ✓ young in a life: 19.7 (want 6-40)
- ✓ litter: 1-1 (want 1-12, fewest first)
- ✓ room near: 8 (want 3-30)

| Taming and food | |
|---|---|
| how | knockout |
| diet | carnivore |
| eats | meat 80%, fish 10%, carrion 10% |
| hunts | 0.1%-60% of its mass alone (5.00-3000 kg); at home: raptor, utahraptor, osmolskae, protoceratops, psittacosaurus, gastonia, pinacosaurus, crocodyliform, citipati |
| a meal that fills it | 500 kg; its biggest prey at home fills it in one kill |
| drinks | 100% of its 211.207 L a day; a drink lasts 36 min, thirsty after 18 |
| feedings to tame at level 1 | 45 (2.5x at level 150) |
| scared by a walking player within | nothing |
| eats (food points) | `base:chicken` 10; `base:porkchop` 15; `base:beef` 20; `base:mutton` 20, favorite; `base:cooked_mutton` 30, favorite; `animals:kibble` 30, kibble |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: bite, claw, tail |
| reach past its body | 5 blocks |
| signature | mount |
| ridden | seat 3.65 up, 1.55 forward |
| flies ridden | 8x its walking speed |
| camouflage (cost of a coat that stands out) | 0.2 |

## hare

*Lepus europaeus*, Mammalia › Lagomorpha › Leporidae › Leporinae. Model `hare`, entity `animals:hare`.

| Where it lives | how common there |
|---|---|
| `base:plains` | commonest |
| `base:desert` | commonest |
| `base:forest` | 60% |
| `base:savanna` | 40% |
| `base:beach` | 40% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.3 m at the shoulder |
| males against females | 0.98x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 3.8 kg | |
| carries her young | 42 d | 0.92 d |
| litter or clutch | 3 on average | 1-4 |
| between litters | 60 d | rests 0.83 d |
| breeds in | spring and summer, most in spring | out of it, only a tame well cared for |
| comfortable in | -5 to 30 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 220 d | 2.81 d |
| lives | 5 y | 19.2 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 26 **set** within 48 blocks |

**Balance** (common): a generation 3.7 d; a line four deep in 11 d (4 h of play); 14.2 young per mother in 10 days, 18 in her life.

- ✓ fourth generation: 11.2 days (want 10-25 days (common))
- ✓ grown most of its life: grows 2.8 of 19 days (want under a fifth)
- ✓ young in a life: 17.8 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 26 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | grass 55%, leaves 25%, roots 10%, seeds 5%, fruit 5% |
| a meal that fills it | 0.38 kg |
| drinks | 30% of its 0.329 L a day; a drink lasts 58 min, thirsty after 29 |
| its food grows at home (spring/summer/autumn/winter) | plains 40/86/46/40%, desert 40/78/55/40%, forest 40/73/41/40%, savanna 43/100/100/40%, beach 40/40/50/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 12 blocks (twice that running) |
| eats (food points) | `base:wheat` 8; `base:wheat_seeds` 2; `base:apple` 10, favorite; `base:berries` 6, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.8 blocks |
| signature | bolt |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 1 |

## mountain_hare

*Lepus timidus*, Mammalia › Lagomorpha › Leporidae › Leporinae. Model `hare`, entity `animals:mountain_hare`.

| Where it lives | how common there |
|---|---|
| `base:taiga` | commonest |
| `base:birch_forest` | commonest |
| `base:snowy_plains` | 75% |
| `base:mountains` | 75% |
| `base:sequoia_grove` | 30% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.3 m at the shoulder |
| males against females | 0.98x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 3 kg | |
| carries her young | 50 d | 1.05 d |
| litter or clutch | 2.5 on average | 1-4 |
| between litters | 70 d | rests 0.97 d |
| breeds in | spring and summer, most in spring | out of it, only a tame well cared for |
| comfortable in | -15 to 24 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 300 d | 3.5 d |
| lives | 5 y | 19.2 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 26 **set** within 48 blocks |

**Balance** (common): a generation 4.5 d; a line four deep in 14 d (5 h of play); 12.3 young per mother in 10 days, 15 in her life.

- ✓ fourth generation: 13.6 days (want 10-25 days (common))
- ✓ grown most of its life: grows 3.5 of 19 days (want under a fifth)
- ✓ young in a life: 14.6 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 26 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | grass 40%, leaves 50%, fruit 5%, lichen 5% |
| a meal that fills it | 0.3 kg |
| drinks | 30% of its 0.266 L a day; a drink lasts 56 min, thirsty after 28 |
| its food grows at home (spring/summer/autumn/winter) | taiga 40/93/40/40%, birch_forest 40/62/40/40%, snowy_plains 40/62/40/40%, mountains 60/95/76/40%, sequoia_grove 40/40/40/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 12 blocks (twice that running) |
| eats (food points) | `base:crowberries` 4; `base:lingonberries` 6, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.8 blocks |
| signature | bolt |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 1 |

## wood_mouse

*Apodemus sylvaticus*, Mammalia › Rodentia › Muridae › Murinae. Model `mouse`, entity `animals:wood_mouse`.

| Where it lives | how common there |
|---|---|
| `base:plains` | commonest |
| `base:forest` | commonest |
| `base:desert` | 60% |
| `base:savanna` | 60% |
| `base:beach` | 50% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.05 m at the shoulder |
| males against females | 1x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 35% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 0.023 kg | |
| carries her young | 25 d | 0.63 d |
| litter or clutch | 4.5 on average | 2-6 |
| between litters | 30 d | rests 0.42 d |
| breeds in | spring, summer and autumn, most in summer | out of it, only a tame well cared for |
| comfortable in | -5 to 32 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 60 d | 3 d **set** |
| lives | 1.5 y | 16 d **set**, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 30 **set** within 48 blocks |

**Balance** (common): a generation 3.6 d; a line four deep in 11 d (4 h of play); 38.3 young per mother in 10 days, 38 in her life.

- ✓ fourth generation: 10.9 days (want 10-25 days (common))
- ✓ grown most of its life: grows 3.0 of 16 days (want under a fifth)
- ✓ young in a life: 37.6 (want 6-40)
- ✓ litter: 2-6 (want 1-12, fewest first)
- ✓ room near: 30 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | seeds 45%, insects 20%, fruit 15%, grass 10%, fungi 5%, roots 5% |
| a meal that fills it | 0.002 kg |
| drinks | 20% of its 0.003 L a day; a drink lasts 52 min, thirsty after 26 |
| its food grows at home (spring/summer/autumn/winter) | plains 40/86/46/40%, forest 40/73/41/40%, desert 40/78/55/40%, savanna 43/100/100/40%, beach 40/40/50/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 6 blocks (twice that running) |
| eats (food points) | `base:wheat_seeds` 4; `base:wheat` 6; `base:berries` 4, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.5 blocks |
| signature | dart |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 1 |

## field_vole

*Microtus agrestis*, Mammalia › Rodentia › Cricetidae › Arvicolinae. Model `vole`, entity `animals:field_vole`.

| Where it lives | how common there |
|---|---|
| `base:taiga` | commonest |
| `base:birch_forest` | commonest |
| `base:snowy_plains` | 80% |
| `base:mountains` | 80% |
| `base:swamp` | 60% |
| `base:sequoia_grove` | commonest |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.05 m at the shoulder |
| males against females | 1x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 35% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 0.035 kg | |
| carries her young | 21 d | 0.55 d |
| litter or clutch | 4.5 on average | 2-5 **set** |
| between litters | 25 d | rests 0.35 d |
| breeds in | all year, most in spring and summer |  |
| comfortable in | -15 to 28 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 35 d | 3 d **set** |
| lives | 2 y | 16 d **set**, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 30 **set** within 48 blocks |

**Balance** (common): a generation 3.5 d; a line four deep in 11 d (4 h of play); 39 young per mother in 10 days, 38 in her life.

- ✓ fourth generation: 10.6 days (want 10-25 days (common))
- ✓ grown most of its life: grows 3.0 of 16 days (want under a fifth)
- ✓ young in a life: 38.2 (want 6-40)
- ✓ litter: 2-5 (want 1-12, fewest first)
- ✓ room near: 30 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | grass 60%, leaves 15%, roots 15%, seeds 5%, insects 5% |
| a meal that fills it | 0.004 kg |
| drinks | 10% of its 0.005 L a day; a drink lasts 108 min, thirsty after 54 |
| its food grows at home (spring/summer/autumn/winter) | taiga 40/65/40/40%, birch_forest 40/62/40/40%, snowy_plains 40/50/40/40%, mountains 40/70/40/40%, swamp 40/100/96/40%, sequoia_grove 40/40/40/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 5 blocks (twice that running) |
| eats (food points) | `base:wheat` 6; `base:wheat_seeds` 3; `base:sunflower_seeds` 6, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.5 blocks |
| signature | dart |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 1 |

## white_tailed_deer

*Odocoileus virginianus*, Mammalia › Artiodactyla › Cervidae › Capreolinae. Model `deer`, entity `animals:white_tailed_deer`.

| Where it lives | how common there |
|---|---|
| `base:plains` | commonest |
| `base:savanna` | 80% |
| `base:desert` | 50% |
| `base:forest` | 50% |
| `base:swamp` | 40% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 0.95 m at the shoulder |
| males against females | 1.08x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 40% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 70 kg | |
| carries her young | 200 d | 2.98 d |
| litter or clutch | 1.8 on average | 1-3 |
| between litters | 1 y | rests 5.07 d |
| breeds in | autumn | out of it, only a tame well cared for |
| comfortable in | -15 to 30 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 1.5 y | 5.28 d |
| lives | 10 y | 38.3 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 15 within 48 blocks |

**Balance** (common): a generation 8.3 d; a line four deep in 25 d (8 h of play); 2.5 young per mother in 10 days, 6 in her life.

- ✓ fourth generation: 24.8 days (want 10-25 days (common))
- ✓ grown most of its life: grows 5.3 of 38 days (want under a fifth)
- ✓ young in a life: 6.3 (want 6-40)
- ✓ litter: 1-3 (want 1-12, fewest first)
- ✓ room near: 15 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | leaves 50%, grass 20%, fruit 15%, seeds 10%, fungi 5% |
| a meal that fills it | 7 kg |
| drinks | 70% of its 4.531 L a day; a drink lasts 33 min, thirsty after 17 |
| its food grows at home (spring/summer/autumn/winter) | plains 40/86/46/40%, savanna 43/100/100/40%, desert 40/78/55/40%, forest 40/73/41/40%, swamp 40/100/96/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 16 blocks (twice that running) |
| eats (food points) | `base:wheat` 8; `base:apple` 12, favorite; `base:berries` 6, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 1 blocks |
| signature | bolt |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.9 |

## red_deer

*Cervus elaphus*, Mammalia › Artiodactyla › Cervidae › Cervinae. Model `deer`, entity `animals:red_deer`.

| Where it lives | how common there |
|---|---|
| `base:forest` | commonest |
| `base:birch_forest` | 80% |
| `base:mountains` | 80% |
| `base:sequoia_grove` | 80% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 0.95 m at the shoulder |
| males against females | 1.1x as tall |
| size spread (an individual's, either way) | 30% |
| newborn's size | 40% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 160 kg | |
| carries her young | 250 d | 3.52 d |
| litter or clutch | 1 on average | 1 |
| between litters | 1 y | rests 3 d **set** |
| breeds in | autumn | out of it, only a tame well cared for |
| comfortable in | -20 to 25 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 2 y | 6.51 d |
| lives | 15 y | 57.5 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 12 within 48 blocks |

**Balance** (uncommon): a generation 10 d; a line four deep in 30 d (10 h of play); 1.5 young per mother in 10 days, 6 in her life.

- ✓ fourth generation: 30.1 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 6.5 of 57 days (want under a fifth)
- ✓ young in a life: 6.1 (want 6-40)
- ✓ litter: 1-1 (want 1-12, fewest first)
- ✓ room near: 12 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | grass 50%, leaves 40%, fruit 5%, seeds 5% |
| a meal that fills it | 16 kg |
| drinks | 80% of its 9.535 L a day; a drink lasts 31 min, thirsty after 16 |
| its food grows at home (spring/summer/autumn/winter) | forest 40/73/41/40%, birch_forest 40/62/40/40%, mountains 40/78/74/40%, sequoia_grove 40/40/40/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 20 blocks (twice that running) |
| eats (food points) | `base:wheat` 10; `base:apple` 12, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 1 blocks |
| signature | bolt |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.9 |

## brown_trout

*Salmo trutta*, Actinopterygii › Salmoniformes › Salmonidae › Salmoninae. Model `fish`, entity `animals:brown_trout`.

| Where it lives | how common there |
|---|---|
| `base:forest` | commonest |
| `base:birch_forest` | commonest |
| `base:mountains` | commonest |
| `base:taiga` | 80% |
| `base:plains` | 60% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.37 m at the fork |
| males against females | 1x as tall |
| size spread (an individual's, either way) | 20% |
| newborn's size | 10% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 0.5 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 45 d | 0.97 d |
| litter or clutch | 8 on average | 3-8 **set** |
| between litters | 1 y | rests 5.07 d |
| breeds in |  | out of it, only a tame well cared for |
| comfortable in | 4 to 20 °C air | cold-blooded: slower in the cold |
| grows up in | 3 y | 8.65 d |
| lives | 12 y | 46 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 26 **set** within 48 blocks |

**Balance** (uncommon): a generation 9.9 d; a line four deep in 30 d (10 h of play); 10.3 young per mother in 10 days, 29 in her life.

- ✓ fourth generation: 29.6 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 8.7 of 46 days (want under a fifth)
- ✓ young in a life: 29.1 (want 6-40)
- ✓ litter: 3-8 (want 1-12, fewest first)
- ✓ room near: 26 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | insects 65%, fish 30%, carrion 5% |
| a meal that fills it | 0.05 kg |
| drinks | never: it lives in the water |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 6 blocks (twice that running) |
| eats (food points) |  |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.5 blocks |
| signature | dart |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 1 |

## pacu

*Piaractus mesopotamicus*, Actinopterygii › Characiformes › Serrasalmidae › Colossomatinae. Model `fish`, entity `animals:pacu`.

| Where it lives | how common there |
|---|---|
| `base:swamp` | commonest |
| `base:savanna` | 80% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.37 m at the fork |
| males against females | 1x as tall |
| size spread (an individual's, either way) | 40% |
| newborn's size | 5% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 4 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 1 d | 0.06 d |
| litter or clutch | 6 on average | 2-8 |
| between litters | 1 y | rests 5.07 d |
| breeds in |  | out of it, only a tame well cared for |
| comfortable in | 20 to 32 °C air | cold-blooded: slower in the cold |
| grows up in | 3 y | 8.65 d |
| lives | 15 y | 57.5 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 26 within 48 blocks |

**Balance** (uncommon): a generation 9 d; a line four deep in 27 d (9 h of play); 9.4 young per mother in 10 days, 35 in her life.

- ✓ fourth generation: 26.9 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 8.7 of 57 days (want under a fifth)
- ✓ young in a life: 35.1 (want 6-40)
- ✓ litter: 2-8 (want 1-12, fewest first)
- ✓ room near: 26 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | omnivore |
| eats | fruit 30%, seeds 15%, insects 35%, fish 20% |
| a meal that fills it | 0.4 kg |
| drinks | never: it lives in the water |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 5 blocks (twice that running) |
| eats (food points) |  |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.5 blocks |
| signature | dart |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.8 |

## red_fox

*Vulpes vulpes*, Mammalia › Carnivora › Canidae › Caninae. Model `fox`, entity `animals:red_fox`.

| Where it lives | how common there |
|---|---|
| `base:forest` | commonest |
| `base:birch_forest` | commonest |
| `base:taiga` | 80% |
| `base:mountains` | 80% |
| `base:plains` | 60% |
| `base:desert` | 40% |
| `base:beach` | 40% |
| `base:sequoia_grove` | 60% |

Spawns by day on grass, snow or sand, in groups of 2-3 (pack), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.38 m at the shoulder |
| males against females | 1.05x as tall |
| size spread (an individual's, either way) | 12% |
| newborn's size | 25% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 5.5 kg | |
| carries her young | 52 d | 1.08 d |
| litter or clutch | 5 on average | 2-7 |
| between litters | 1 y | rests 2.5 d **set** |
| breeds in | winter | out of it, only a tame well cared for |
| comfortable in | -20 to 30 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 300 d | 3.5 d |
| lives | 5 y | 19.2 d, give or take a fifth; an elder from 80% of it |
| lives in | pack | room for 24 **set** within 48 blocks |

**Balance** (common): a generation 4.6 d; a line four deep in 14 d (5 h of play); 12.6 young per mother in 10 days, 15 in her life.

- ✓ fourth generation: 13.7 days (want 10-25 days (common))
- ✓ grown most of its life: grows 3.5 of 19 days (want under a fifth)
- ✓ young in a life: 14.9 (want 6-40)
- ✓ litter: 2-7 (want 1-12, fewest first)
- ✓ room near: 24 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | meat 50%, fruit 20%, insects 15%, carrion 15% |
| hunts | 0.4%-80% of its mass alone (0.02-4 kg); at home: hare, mountain_hare, wood_mouse, field_vole, brown_trout, raven, house_sparrow, snow_bunting, great_horned_owl, snowy_owl, turkey_vulture, herring_gull, base:chicken |
| a meal that fills it | 0.55 kg; its biggest prey at home fills it in one kill |
| drinks | 50% of its 0.459 L a day; a drink lasts 36 min, thirsty after 18 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 10 blocks (twice that running) |
| eats (food points) | `base:chicken` 10; `base:beef` 10; `base:berries` 6, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: pounce, bite |
| reach past its body | 0.6 blocks |
| signature | pounce |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.6 |

## arctic_fox

*Vulpes lagopus*, Mammalia › Carnivora › Canidae › Caninae. Model `fox`, entity `animals:arctic_fox`.

| Where it lives | how common there |
|---|---|
| `base:snowy_plains` | commonest |
| `base:taiga` | 30% |

Spawns by day on grass, snow or sand, in groups of 2-3 (pack), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.38 m at the shoulder |
| males against females | 1.05x as tall |
| size spread (an individual's, either way) | 35% |
| newborn's size | 25% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 3.5 kg | |
| carries her young | 52 d | 1.08 d |
| litter or clutch | 7 on average | 3-10 |
| between litters | 1 y | rests 5.07 d |
| breeds in | spring | out of it, only a tame well cared for |
| comfortable in | -35 to 18 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 300 d | 3.5 d |
| lives | 5 y | 19.2 d, give or take a fifth; an elder from 80% of it |
| lives in | pack | room for 24 **set** within 48 blocks |

**Balance** (common): a generation 4.6 d; a line four deep in 14 d (5 h of play); 10.6 young per mother in 10 days, 12 in her life.

- ✓ fourth generation: 13.7 days (want 10-25 days (common))
- ✓ grown most of its life: grows 3.5 of 19 days (want under a fifth)
- ✓ young in a life: 12.5 (want 6-40)
- ✓ litter: 3-10 (want 1-12, fewest first)
- ✓ room near: 24 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | meat 55%, carrion 25%, fruit 10%, insects 5%, fish 5% |
| hunts | 0.4%-100% of its mass alone (0.01-4 kg); at home: mountain_hare, field_vole, brown_trout, raven, snow_bunting, great_horned_owl, snowy_owl, base:chicken |
| a meal that fills it | 0.35 kg; its biggest prey at home fills it in one kill |
| drinks | 30% of its 0.306 L a day; a drink lasts 57 min, thirsty after 29 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 10 blocks (twice that running) |
| eats (food points) | `base:chicken` 10; `base:beef` 10; `base:berries` 6, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: pounce, bite |
| reach past its body | 0.6 blocks |
| signature | pounce |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.6 |

## raven

*Corvus corax*, Aves › Passeriformes › Corvidae › Corvinae. Model `raven`, entity `animals:raven`.

| Where it lives | how common there |
|---|---|
| `base:taiga` | commonest |
| `base:mountains` | 80% |
| `base:forest` | 60% |
| `base:birch_forest` | 60% |
| `base:snowy_plains` | 60% |
| `base:desert` | 40% |
| `base:plains` | 30% |
| `base:beach` | 30% |
| `base:sequoia_grove` | 80% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.34 m at the crown |
| males against females | 1.08x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 1.2 kg | |
| carries her young | 0 d | 0.53 d **set** |
| litter or clutch | 4.5 on average | 2-6 |
| between litters | 1 y | rests 5.07 d |
| breeds in | spring | out of it, only a tame well cared for |
| comfortable in | -30 to 32 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 2 y | 6.51 d |
| lives | 12 y | 46 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 17 within 48 blocks |

**Balance** (uncommon): a generation 7.6 d; a line four deep in 23 d (8 h of play); 7.1 young per mother in 10 days, 22 in her life.

- ✓ fourth generation: 22.7 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 6.5 of 46 days (want under a fifth)
- ✓ young in a life: 21.6 (want 6-40)
- ✓ litter: 2-6 (want 1-12, fewest first)
- ✓ room near: 17 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | carrion 40%, meat 20%, insects 15%, seeds 15%, fruit 10% |
| hunts | 1.0%-10% of its mass alone (0.01-0 kg); at home: wood_mouse, field_vole, house_sparrow, snow_bunting |
| a meal that fills it | 0.12 kg; its biggest prey at home in 3.4 kills |
| drinks | 70% of its 0.117 L a day; a drink lasts 22 min, thirsty after 11 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 14 blocks (twice that running) |
| eats (food points) | `base:beef` 10; `base:wheat_seeds` 2; `base:rotten_flesh` 8, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | yes:  |
| reach past its body | 0.4 blocks |
| signature | soar |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.2 |

## protoceratops

*Protoceratops andrewsi*, Dinosauria › Ceratopsia › Protoceratopsidae › Protoceratopsinae. Model `protoceratops`, entity `animals:protoceratops`.

| Where it lives | how common there |
|---|---|
| `animals:red_dunes` | commonest |
| `animals:scrubland` | 60% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | medium |
| height (grown, average) | 0.6 m at the hip |
| males against females | 1.05x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 15% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 83 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 83 d | 1.54 d |
| litter or clutch | 12 on average | 3-8 **set** |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 8 to 40 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 3 y | 6 d **set** |
| lives | 14 y | 53.7 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 20 **set** within 48 blocks |

**Balance** (common): a generation 7.8 d; a line four deep in 23 d (8 h of play); 10.3 young per mother in 10 days, 38 in her life.

- ✓ fourth generation: 23.4 days (want 10-25 days (common))
- ✓ grown most of its life: grows 6.0 of 54 days (want under a fifth)
- ✓ young in a life: 38.1 (want 6-40)
- ✓ litter: 3-8 (want 1-12, fewest first)
- ✓ room near: 20 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | leaves 75%, roots 15%, seeds 10% |
| a meal that fills it | 8.3 kg |
| drinks | 40% of its 5.282 L a day; a drink lasts 59 min, thirsty after 29 |
| its food grows at home (spring/summer/autumn/winter) | red_dunes 40/40/40/40%, scrubland 40/40/40/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 12 blocks (twice that running) |
| eats (food points) | `animals:fern` 8; `base:wheat` 8; `animals:cycad_seed` 10, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 1 blocks |
| signature | herd |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.9 |

## kryptobaatar

*Kryptobaatar dashzevegi*, Mammalia › Multituberculata › Djadochtatheriidae › Djadochtatheriinae. Model `kryptobaatar`, entity `animals:kryptobaatar`.

| Where it lives | how common there |
|---|---|
| `animals:red_dunes` | commonest |
| `animals:scrubland` | commonest |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.05 m at the shoulder |
| males against females | 1x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 35% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 0.04 kg | |
| carries her young | 20 d | 0.53 d |
| litter or clutch | 3.5 on average | 1-5 |
| between litters | 30 d | rests 0.42 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 5 to 40 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 60 d | 3 d **set** |
| lives | 2 y | 16 d **set**, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 30 **set** within 48 blocks |

**Balance** (common): a generation 3.5 d; a line four deep in 11 d (4 h of play); 31.7 young per mother in 10 days, 31 in her life.

- ✓ fourth generation: 10.6 days (want 10-25 days (common))
- ✓ grown most of its life: grows 3.0 of 16 days (want under a fifth)
- ✓ young in a life: 31.1 (want 6-40)
- ✓ litter: 1-5 (want 1-12, fewest first)
- ✓ room near: 30 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | omnivore |
| eats | insects 60%, seeds 25%, fruit 10%, roots 5% |
| a meal that fills it | 0.004 kg |
| drinks | 10% of its 0.005 L a day; a drink lasts 110 min, thirsty after 55 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 6 blocks (twice that running) |
| eats (food points) | `base:wheat_seeds` 4; `base:wheat` 6; `base:berries` 4, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.5 blocks |
| signature | dart |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 1 |

## priscagama

*Priscagama gobiensis*, Reptilia › Squamata › Priscagamidae › Priscagaminae. Model `lizard`, entity `animals:priscagama`.

| Where it lives | how common there |
|---|---|
| `animals:red_dunes` | commonest |
| `animals:scrubland` | commonest |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.04 m at the shoulder |
| males against females | 1.05x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 20% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 0.03 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 60 d | 1.21 d |
| litter or clutch | 4.5 on average | 2-6 |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 12 to 42 °C air | cold-blooded: slower in the cold |
| grows up in | 1 y | 4.01 d |
| lives | 6 y | 23 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 24 **set** within 48 blocks |

**Balance** (common): a generation 5.5 d; a line four deep in 16 d (5 h of play); 7.5 young per mother in 10 days, 11 in her life.

- ✓ fourth generation: 16.4 days (want 10-25 days (common))
- ✓ grown most of its life: grows 4.0 of 23 days (want under a fifth)
- ✓ young in a life: 10.8 (want 6-40)
- ✓ litter: 2-6 (want 1-12, fewest first)
- ✓ room near: 24 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | insects 90%, leaves 10% |
| a meal that fills it | 0.003 kg |
| drinks | 10% of its 0.001 L a day; a drink lasts 534 min, thirsty after 267 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 5 blocks (twice that running) |
| eats (food points) | `base:spider_eye` 4, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.4 blocks |
| signature | bask |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 1 |

## reindeer

*Rangifer tarandus*, Mammalia › Artiodactyla › Cervidae › Capreolinae. Model `reindeer`, entity `animals:reindeer`.

| Where it lives | how common there |
|---|---|
| `base:snowy_plains` | commonest |
| `base:taiga` | 60% |
| `base:mountains` | 30% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 1.05 m at the shoulder |
| males against females | 1.12x as tall |
| size spread (an individual's, either way) | 15% |
| newborn's size | 35% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 120 kg | |
| carries her young | 230 d | 3.31 d |
| litter or clutch | 1 on average | 1 |
| between litters | 1 y | rests 1.67 d **set** |
| breeds in | autumn | out of it, only a tame well cared for |
| comfortable in | -45 to 18 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 1.6 y | 4.58 d **set** |
| lives | 12 y | 46 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 20 **set** within 48 blocks |

**Balance** (common): a generation 7.9 d; a line four deep in 24 d (8 h of play); 2 young per mother in 10 days, 6 in her life.

- ✓ fourth generation: 23.7 days (want 10-25 days (common))
- ✓ grown most of its life: grows 4.6 of 46 days (want under a fifth)
- ✓ young in a life: 6.5 (want 6-40)
- ✓ litter: 1-1 (want 1-12, fewest first)
- ✓ room near: 20 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | lichen 40%, grass 35%, leaves 20%, fungi 5% |
| a meal that fills it | 12 kg |
| drinks | 100% of its 7.36 L a day; a drink lasts 24 min, thirsty after 12 |
| its food grows at home (spring/summer/autumn/winter) | snowy_plains 40/62/40/40%, taiga 40/80/40/40%, mountains 60/95/76/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 18 blocks (twice that running) |
| eats (food points) | `base:mushroom_brown` 8 |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 1 blocks |
| signature | crater |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.7 |

## coyote

*Canis latrans*, Mammalia › Carnivora › Canidae › Caninae. Model `coyote`, entity `animals:coyote`.

| Where it lives | how common there |
|---|---|
| `base:plains` | commonest |
| `base:desert` | commonest |
| `base:forest` | 40% |
| `base:mountains` | 40% |
| `base:sequoia_grove` | 60% |

Spawns by day on grass, snow or sand, in groups of 2-3 (pack), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | medium |
| height (grown, average) | 0.61 m at the shoulder |
| males against females | 1.05x as tall |
| size spread (an individual's, either way) | 20% |
| newborn's size | 25% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 13 kg | |
| carries her young | 63 d | 1.25 d |
| litter or clutch | 6 on average | 2-8 |
| between litters | 1 y | rests 5.07 d |
| breeds in | winter | out of it, only a tame well cared for |
| comfortable in | -30 to 36 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 300 d | 3.5 d |
| lives | 6 y | 23 d, give or take a fifth; an elder from 80% of it |
| lives in | pack | room for 22 **set** within 48 blocks |

**Balance** (common): a generation 4.7 d; a line four deep in 14 d (5 h of play); 7.9 young per mother in 10 days, 12 in her life.

- ✓ fourth generation: 14.2 days (want 10-25 days (common))
- ✓ grown most of its life: grows 3.5 of 23 days (want under a fifth)
- ✓ young in a life: 11.8 (want 6-40)
- ✓ litter: 2-8 (want 1-12, fewest first)
- ✓ room near: 22 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | meat 60%, fruit 15%, carrion 15%, insects 10% |
| hunts | 0.1%-40% of its mass alone (0.01-5 kg); at home: cat, hare, mountain_hare, wood_mouse, field_vole, brown_trout, raven, house_sparrow, snow_bunting, great_horned_owl, snowy_owl, golden_eagle, turkey_vulture, herring_gull, base:chicken |
| a meal that fills it | 1.3 kg; its biggest prey at home fills it in one kill |
| drinks | 100% of its 0.996 L a day; a drink lasts 20 min, thirsty after 10 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 14 blocks (twice that running) |
| eats (food points) | `base:beef` 12; `base:chicken` 10, favorite; `base:berries` 4 |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: pounce, bite |
| reach past its body | 0.7 blocks |
| signature | howl |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.6 |

## house_sparrow

*Passer domesticus*, Aves › Passeriformes › Passeridae › Passerinae. Model `small_bird`, entity `animals:house_sparrow`.

| Where it lives | how common there |
|---|---|
| `base:plains` | commonest |
| `base:desert` | 50% |
| `base:beach` | 50% |
| `base:forest` | 40% |
| `base:swamp` | 30% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.1 m at the crown |
| males against females | 1.02x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 60% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 0.03 kg | |
| carries her young | 0 d | 0.36 d **set** |
| litter or clutch | 4.5 on average | 2-6 |
| between litters | 100 d | rests 1.39 d |
| breeds in | spring and summer, most in spring | out of it, only a tame well cared for |
| comfortable in | -25 to 38 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 300 d | 2.75 d **set** |
| lives | 3 y | 14 d **set**, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 30 **set** within 48 blocks |

**Balance** (common): a generation 3.5 d; a line four deep in 10 d (3 h of play); 22.8 young per mother in 10 days, 19 in her life.

- ✓ fourth generation: 10.4 days (want 10-25 days (common))
- ✓ grown most of its life: grows 2.8 of 14 days (want under a fifth)
- ✓ young in a life: 19.3 (want 6-40)
- ✓ litter: 2-6 (want 1-12, fewest first)
- ✓ room near: 30 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | omnivore |
| eats | seeds 60%, insects 30%, fruit 10% |
| a meal that fills it | 0.003 kg |
| drinks | 100% of its 0.004 L a day; a drink lasts 11 min, thirsty after 5 |
| its food grows at home (spring/summer/autumn/winter) | plains 40/86/45/40%, desert 40/78/55/40%, beach 40/40/40/40%, forest 40/73/40/40%, swamp 40/100/96/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 8 blocks (twice that running) |
| eats (food points) | `base:wheat_seeds` 2; `base:berries` 3, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.2 blocks |
| signature | flock |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.6 |

## snow_bunting

*Plectrophenax nivalis*, Aves › Passeriformes › Calcariidae › . Model `small_bird`, entity `animals:snow_bunting`.

| Where it lives | how common there |
|---|---|
| `base:snowy_plains` | commonest |
| `base:mountains` | 40% |
| `base:taiga` | 30% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.1 m at the crown |
| males against females | 1.03x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 60% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 0.035 kg | |
| carries her young | 0 d | 0.37 d **set** |
| litter or clutch | 5 on average | 2-7 |
| between litters | 1 y | rests 2.5 d **set** |
| breeds in | summer | out of it, only a tame well cared for |
| comfortable in | -40 to 18 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 300 d | 2.75 d **set** |
| lives | 3 y | 14 d **set**, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 30 **set** within 48 blocks |

**Balance** (common): a generation 3.5 d; a line four deep in 10 d (3 h of play); 15.7 young per mother in 10 days, 13 in her life.

- ✓ fourth generation: 10.5 days (want 10-25 days (common))
- ✓ grown most of its life: grows 2.8 of 14 days (want under a fifth)
- ✓ young in a life: 13.2 (want 6-40)
- ✓ litter: 2-7 (want 1-12, fewest first)
- ✓ room near: 30 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | omnivore |
| eats | seeds 40%, insects 30%, grass 15%, leaves 10%, fruit 5% |
| a meal that fills it | 0.004 kg |
| drinks | 100% of its 0.005 L a day; a drink lasts 11 min, thirsty after 5 |
| its food grows at home (spring/summer/autumn/winter) | snowy_plains 40/50/40/40%, mountains 40/78/74/40%, taiga 40/93/40/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 8 blocks (twice that running) |
| eats (food points) | `base:wheat_seeds` 2; `base:berries` 3, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.2 blocks |
| signature | flock |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.6 |

## great_horned_owl

*Bubo virginianus*, Aves › Strigiformes › Strigidae › Striginae. Model `owl`, entity `animals:great_horned_owl`.

| Where it lives | how common there |
|---|---|
| `base:forest` | commonest |
| `base:birch_forest` | 80% |
| `base:taiga` | 80% |
| `base:plains` | 80% |
| `base:mountains` | 60% |
| `base:desert` | 60% |
| `base:swamp` | 60% |
| `base:savanna` | 40% |
| `base:sequoia_grove` | 80% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.45 m at the crown |
| males against females | 0.95x as tall |
| size spread (an individual's, either way) | 15% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 1.4 kg | |
| carries her young | 0 d | 0.77 d **set** |
| litter or clutch | 2 on average | 1-3 |
| between litters | 1 y | rests 5.07 d |
| breeds in | winter | out of it, only a tame well cared for |
| comfortable in | -35 to 38 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 2 y | 6.51 d |
| lives | 13 y | 49.8 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 16 within 48 blocks |

**Balance** (uncommon): a generation 8.1 d; a line four deep in 24 d (8 h of play); 3.4 young per mother in 10 days, 11 in her life.

- ✓ fourth generation: 24.2 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 6.5 of 50 days (want under a fifth)
- ✓ young in a life: 11.4 (want 6-40)
- ✓ litter: 1-3 (want 1-12, fewest first)
- ✓ room near: 16 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | meat 90%, insects 5%, fish 5% |
| hunts | 0.5%-180% of its mass alone (0.01-3 kg); at home: dog, wood_mouse, field_vole, brown_trout, raven, house_sparrow, snow_bunting, savanna_hawk, turkey_vulture, herring_gull, base:chicken |
| a meal that fills it | 0.14 kg; its biggest prey at home fills it in one kill |
| drinks | 100% of its 0.134 L a day; a drink lasts 16 min, thirsty after 8 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 14 blocks (twice that running) |
| eats (food points) | `base:beef` 10; `base:chicken` 10, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | yes:  |
| reach past its body | 0.5 blocks |
| signature | silent |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.7 |

## snowy_owl

*Bubo scandiacus*, Aves › Strigiformes › Strigidae › Striginae. Model `owl`, entity `animals:snowy_owl`.

| Where it lives | how common there |
|---|---|
| `base:snowy_plains` | commonest |
| `base:taiga` | 30% |
| `base:mountains` | 30% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.45 m at the crown |
| males against females | 0.95x as tall |
| size spread (an individual's, either way) | 12% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 2 kg | |
| carries her young | 0 d | 0.75 d **set** |
| litter or clutch | 6 on average | 2-8 |
| between litters | 1 y | rests 5.07 d |
| breeds in | spring | out of it, only a tame well cared for |
| comfortable in | -50 to 16 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 2 y | 6.51 d |
| lives | 10 y | 38.3 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 15 within 48 blocks |

**Balance** (uncommon): a generation 8 d; a line four deep in 24 d (8 h of play); 8.6 young per mother in 10 days, 21 in her life.

- ✓ fourth generation: 24.1 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 6.5 of 38 days (want under a fifth)
- ✓ young in a life: 20.7 (want 6-40)
- ✓ litter: 2-8 (want 1-12, fewest first)
- ✓ room near: 15 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | meat 100% |
| hunts | 0.5%-120% of its mass alone (0.01-2 kg); at home: field_vole, brown_trout, raven, snow_bunting, turkey_vulture, base:chicken |
| a meal that fills it | 0.2 kg; its biggest prey at home fills it in one kill |
| drinks | 100% of its 0.185 L a day; a drink lasts 16 min, thirsty after 8 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 16 blocks (twice that running) |
| eats (food points) | `base:beef` 10; `base:chicken` 10, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | yes:  |
| reach past its body | 0.5 blocks |
| signature | silent |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.7 |

## golden_eagle

*Aquila chrysaetos*, Aves › Accipitriformes › Accipitridae › Aquilinae. Model `eagle`, entity `animals:golden_eagle`.

| Where it lives | how common there |
|---|---|
| `base:mountains` | commonest |
| `base:plains` | 60% |
| `base:desert` | 60% |
| `base:taiga` | 50% |
| `base:snowy_plains` | 30% |
| `base:forest` | 30% |
| `base:birch_forest` | 30% |
| `base:sequoia_grove` | 30% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | medium |
| height (grown, average) | 0.6 m at the crown |
| males against females | 0.92x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 4.5 kg | |
| carries her young | 0 d | 0.94 d **set** |
| litter or clutch | 2 on average | 1-3 |
| between litters | 1 y | rests 5.07 d |
| breeds in | spring | out of it, only a tame well cared for |
| comfortable in | -40 to 38 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 4.1 y | 8.33 d **set** |
| lives | 20 y | 76.7 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 13 within 48 blocks |

**Balance** (uncommon): a generation 10.2 d; a line four deep in 31 d (10 h of play); 3.3 young per mother in 10 days, 18 in her life.

- ✓ fourth generation: 30.6 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 8.3 of 77 days (want under a fifth)
- ✓ young in a life: 17.6 (want 6-40)
- ✓ litter: 1-3 (want 1-12, fewest first)
- ✓ room near: 13 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | meat 85%, carrion 15% |
| hunts | 1.0%-150% of its mass alone (0.04-7 kg); at home: dog, cat, hare, mountain_hare, brown_trout, red_fox, arctic_fox, raven, great_horned_owl, snowy_owl, turkey_vulture, herring_gull, base:chicken |
| a meal that fills it | 0.45 kg; its biggest prey at home fills it in one kill |
| drinks | 100% of its 0.383 L a day; a drink lasts 18 min, thirsty after 9 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 24 blocks (twice that running) |
| eats (food points) | `base:beef` 12; `base:chicken` 10, favorite; `base:rotten_flesh` 4 |

| Fighting, riding, the wild | |
|---|---|
| fights | yes:  |
| reach past its body | 0.6 blocks |
| signature | stoop |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.4 |

## savanna_hawk

*Buteogallus meridionalis*, Aves › Accipitriformes › Accipitridae › Buteoninae. Model `eagle`, entity `animals:savanna_hawk`.

| Where it lives | how common there |
|---|---|
| `base:savanna` | commonest |
| `base:swamp` | 80% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.6 m at the crown |
| males against females | 0.95x as tall |
| size spread (an individual's, either way) | 40% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 0.85 kg | |
| carries her young | 0 d | 0.87 d **set** |
| litter or clutch | 1 on average | 1 |
| between litters | 1 y | rests 2.5 d **set** |
| breeds in | winter | out of it, only a tame well cared for |
| comfortable in | 5 to 42 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 2 y | 6.51 d |
| lives | 12 y | 46 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 18 within 48 blocks |

**Balance** (common): a generation 8.3 d; a line four deep in 25 d (8 h of play); 3 young per mother in 10 days, 9 in her life.

- ✓ fourth generation: 24.8 days (want 10-25 days (common))
- ✓ grown most of its life: grows 6.5 of 46 days (want under a fifth)
- ✓ young in a life: 9.0 (want 6-40)
- ✓ litter: 1-1 (want 1-12, fewest first)
- ✓ room near: 18 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | meat 70%, insects 20%, fish 10% |
| hunts | 0.5%-60% of its mass alone (0.00-1 kg); at home: wood_mouse, field_vole, house_sparrow |
| a meal that fills it | 0.085 kg; its biggest prey at home in 2.4 kills |
| drinks | 100% of its 0.086 L a day; a drink lasts 15 min, thirsty after 7 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 16 blocks (twice that running) |
| eats (food points) | `base:beef` 8; `base:chicken` 8, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | yes:  |
| reach past its body | 0.5 blocks |
| signature | stoop |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.4 |

## turkey_vulture

*Cathartes aura*, Aves › Accipitriformes › Cathartidae › . Model `vulture`, entity `animals:turkey_vulture`.

| Where it lives | how common there |
|---|---|
| `base:savanna` | commonest |
| `base:swamp` | commonest |
| `base:desert` | 80% |
| `base:plains` | 60% |
| `base:forest` | 30% |
| `base:mountains` | 30% |
| `base:sequoia_grove` | 40% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.5 m at the crown |
| males against females | 1x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 1.4 kg | |
| carries her young | 0 d | 0.89 d **set** |
| litter or clutch | 2 on average | 1-3 |
| between litters | 1 y | rests 3.33 d **set** |
| breeds in | spring | out of it, only a tame well cared for |
| comfortable in | -12 to 42 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 3 y | 4.58 d **set** |
| lives | 16 y | 61.3 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 24 **set** within 48 blocks |

**Balance** (common): a generation 6.4 d; a line four deep in 19 d (6 h of play); 4.7 young per mother in 10 days, 21 in her life.

- ✓ fourth generation: 19.1 days (want 10-25 days (common))
- ✓ grown most of its life: grows 4.6 of 61 days (want under a fifth)
- ✓ young in a life: 21.1 (want 6-40)
- ✓ litter: 1-3 (want 1-12, fewest first)
- ✓ room near: 24 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | carrion 95%, fruit 5% |
| a meal that fills it | 0.14 kg |
| drinks | 100% of its 0.134 L a day; a drink lasts 16 min, thirsty after 8 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 16 blocks (twice that running) |
| eats (food points) | `base:rotten_flesh` 10, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.4 blocks |
| signature | scent |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.2 |

## herring_gull

*Larus argentatus*, Aves › Charadriiformes › Laridae › Larinae. Model `gull`, entity `animals:herring_gull`.

| Where it lives | how common there |
|---|---|
| `base:beach` | commonest |
| `base:ocean` | 60% |
| `base:plains` | 10% |
| `base:swamp` | 20% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.45 m at the crown |
| males against females | 1.05x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 1 kg | |
| carries her young | 0 d | 0.7 d **set** |
| litter or clutch | 3 on average | 1-4 |
| between litters | 1 y | rests 5.07 d |
| breeds in | spring | out of it, only a tame well cared for |
| comfortable in | -15 to 34 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 4 y | 4.58 d **set** |
| lives | 12 y | 46 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 24 **set** within 48 blocks |

**Balance** (common): a generation 6 d; a line four deep in 18 d (6 h of play); 4.3 young per mother in 10 days, 14 in her life.

- ✓ fourth generation: 17.9 days (want 10-25 days (common))
- ✓ grown most of its life: grows 4.6 of 46 days (want under a fifth)
- ✓ young in a life: 13.9 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 24 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | fish 45%, carrion 25%, meat 10%, insects 10%, seeds 5%, fruit 5% |
| hunts | 0.5%-30% of its mass alone (0.00-0 kg); at home: wood_mouse, field_vole, house_sparrow |
| a meal that fills it | 0.1 kg; its biggest prey at home in 2.9 kills |
| drinks | 100% of its 0.099 L a day; a drink lasts 15 min, thirsty after 8 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 10 blocks (twice that running) |
| eats (food points) | `base:beef` 8; `base:bread` 6, favorite; `base:rotten_flesh` 6 |

| Fighting, riding, the wild | |
|---|---|
| fights | yes:  |
| reach past its body | 0.5 blocks |
| signature | colony |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.2 |

## yacare_caiman

*Caiman yacare*, Reptilia › Crocodylia › Alligatoridae › Caimaninae. Model `caiman`, entity `animals:yacare_caiman`.

| Where it lives | how common there |
|---|---|
| `base:swamp` | commonest |
| `base:savanna` | 30% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 2.2 m at the length |
| males against females | 1.2x as tall |
| size spread (an individual's, either way) | 20% |
| newborn's size | 15% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 35 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 73 d | 1.4 d |
| litter or clutch | 30 on average | 4-8 **set** |
| between litters | 1 y | rests 16.7 d **set** |
| breeds in | spring and summer, most in summer | out of it, only a tame well cared for |
| comfortable in | 20 to 36 °C air | cold-blooded: slower in the cold |
| grows up in | 7 y | 6.25 d **set** |
| lives | 35 y | 75 d **set**, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 26 **set** within 48 blocks |

**Balance** (common): a generation 7.9 d; a line four deep in 24 d (8 h of play); 3.5 young per mother in 10 days, 19 in her life.

- ✓ fourth generation: 23.7 days (want 10-25 days (common))
- ✓ grown most of its life: grows 6.2 of 75 days (want under a fifth)
- ✓ young in a life: 19.1 (want 6-40)
- ✓ litter: 4-8 (want 1-12, fewest first)
- ✓ room near: 26 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | fish 60%, meat 30%, insects 10% |
| hunts | 0.5%-100% of its mass alone (0.17-35 kg); at home: dog, cat, hare, pacu, great_horned_owl, savanna_hawk, turkey_vulture, herring_gull, base:chicken |
| a meal that fills it | 3.5 kg; its biggest prey at home fills it in one kill |
| drinks | 100% of its 0.486 L a day; a drink lasts 108 min, thirsty after 54 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 8 blocks (twice that running) |
| eats (food points) | `base:beef` 14; `base:chicken` 12, favorite; `base:porkchop` 12 |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: bite |
| reach past its body | 1 blocks |
| signature | ambush |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.8 |

## spectacled_caiman

*Caiman crocodilus*, Reptilia › Crocodylia › Alligatoridae › Caimaninae. Model `caiman`, entity `animals:spectacled_caiman`.

| Where it lives | how common there |
|---|---|
| `base:savanna` | commonest |
| `base:swamp` | 50% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 2.2 m at the length |
| males against females | 1.2x as tall |
| size spread (an individual's, either way) | 20% |
| newborn's size | 15% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 25 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 73 d | 1.4 d |
| litter or clutch | 25 on average | 4-8 **set** |
| between litters | 1 y | rests 16.7 d **set** |
| breeds in | spring and summer, most in summer | out of it, only a tame well cared for |
| comfortable in | 20 to 36 °C air | cold-blooded: slower in the cold |
| grows up in | 7 y | 6.25 d **set** |
| lives | 35 y | 75 d **set**, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 26 **set** within 48 blocks |

**Balance** (common): a generation 7.9 d; a line four deep in 24 d (8 h of play); 3.5 young per mother in 10 days, 19 in her life.

- ✓ fourth generation: 23.7 days (want 10-25 days (common))
- ✓ grown most of its life: grows 6.2 of 75 days (want under a fifth)
- ✓ young in a life: 19.1 (want 6-40)
- ✓ litter: 4-8 (want 1-12, fewest first)
- ✓ room near: 26 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | fish 60%, meat 30%, insects 10% |
| hunts | 0.5%-100% of its mass alone (0.12-25 kg); at home: dog, cat, hare, pacu, great_horned_owl, savanna_hawk, turkey_vulture, herring_gull, base:chicken |
| a meal that fills it | 2.5 kg; its biggest prey at home fills it in one kill |
| drinks | 100% of its 0.359 L a day; a drink lasts 105 min, thirsty after 52 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 8 blocks (twice that running) |
| eats (food points) | `base:beef` 14; `base:chicken` 12, favorite; `base:porkchop` 12 |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: bite |
| reach past its body | 1 blocks |
| signature | ambush |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.8 |

## grey_wolf

*Canis lupus*, Mammalia › Carnivora › Canidae › Caninae. Model `wolf`, entity `animals:grey_wolf`.

| Where it lives | how common there |
|---|---|
| `base:taiga` | commonest |
| `base:forest` | 60% |
| `base:birch_forest` | 60% |
| `base:mountains` | 50% |
| `base:snowy_plains` | 40% |
| `base:plains` | 10% |
| `base:sequoia_grove` | 30% |

Spawns by day on grass, snow or sand, in groups of 2-3 (pack), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | medium |
| height (grown, average) | 0.8 m at the shoulder |
| males against females | 1.07x as tall |
| size spread (an individual's, either way) | 20% |
| newborn's size | 20% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 42 kg | |
| carries her young | 63 d | 1.25 d |
| litter or clutch | 5.5 on average | 2-8 |
| between litters | 1 y | rests 5.07 d |
| breeds in | winter | out of it, only a tame well cared for |
| comfortable in | -40 to 28 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 1.8 y | 6.07 d |
| lives | 12 y | 46 d, give or take a fifth; an elder from 80% of it |
| lives in | pack | room for 16 **set** within 48 blocks |

**Balance** (uncommon): a generation 7.3 d; a line four deep in 22 d (7 h of play); 7.9 young per mother in 10 days, 24 in her life.

- ✓ fourth generation: 22.0 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 6.1 of 46 days (want under a fifth)
- ✓ young in a life: 24.3 (want 6-40)
- ✓ litter: 2-8 (want 1-12, fewest first)
- ✓ room near: 16 (want 3-30)

| Taming and food | |
|---|---|
| how | passive |
| diet | carnivore |
| eats | meat 85%, carrion 15% |
| hunts | 2.0%-300% of its mass alone (0.84-126 kg); at home: cat, hare, mountain_hare, white_tailed_deer, red_deer, raven, reindeer, great_horned_owl, snowy_owl, golden_eagle, turkey_vulture, herring_gull, base:chicken, base:pig, base:sheep |
| a meal that fills it | 4.2 kg; its biggest prey at home fills it in one kill |
| drinks | 100% of its 2.861 L a day; a drink lasts 22 min, thirsty after 11 |
| feedings to tame at level 1 | 10 (2.5x at level 150) |
| scared by a walking player within | 20 blocks (twice that running) |
| eats (food points) | `base:beef` 10; `base:porkchop` 10; `base:chicken` 6; `base:cooked_beef` 14; `base:cooked_mutton` 14; `base:mutton` 12, favorite; `animals:kibble` 30, kibble |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: pounce, bite |
| reach past its body | 0.9 blocks |
| signature | pack |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.5 |

## jaguar

*Panthera onca*, Mammalia › Carnivora › Felidae › Pantherinae. Model `jaguar`, entity `animals:jaguar`.

| Where it lives | how common there |
|---|---|
| `base:swamp` | commonest |
| `base:savanna` | 60% |
| `base:desert` | 5% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 0.7 m at the shoulder |
| males against females | 1.1x as tall |
| size spread (an individual's, either way) | 15% |
| newborn's size | 12% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 85 kg | |
| carries her young | 100 d | 1.77 d |
| litter or clutch | 1.6 on average | 1-2 |
| between litters | 1.9 y | rests 4.17 d **set** |
| breeds in | all year, most in spring and summer |  |
| comfortable in | 8 to 38 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 2.5 y | 7.54 d |
| lives | 17 y | 65.1 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 12 **set** within 48 blocks |

**Balance** (uncommon): a generation 9.3 d; a line four deep in 28 d (9 h of play); 2.5 young per mother in 10 days, 11 in her life.

- ✓ fourth generation: 27.9 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 7.5 of 65 days (want under a fifth)
- ✓ young in a life: 11.3 (want 6-40)
- ✓ litter: 1-2 (want 1-12, fewest first)
- ✓ room near: 12 (want 3-30)

| Taming and food | |
|---|---|
| how | knockout |
| diet | carnivore |
| eats | meat 85%, fish 15% |
| hunts | 1.0%-150% of its mass alone (0.85-128 kg); at home: dog, capybara, hare, white_tailed_deer, pacu, red_fox, raven, coyote, great_horned_owl, golden_eagle, savanna_hawk, turkey_vulture, herring_gull, yacare_caiman, spectacled_caiman, base:chicken, base:pig, base:sheep |
| a meal that fills it | 8.5 kg; its biggest prey at home fills it in one kill |
| drinks | 100% of its 5.397 L a day; a drink lasts 24 min, thirsty after 12 |
| feedings to tame at level 1 | 14 (2.5x at level 150) |
| scared by a walking player within | 12 blocks (twice that running) |
| eats (food points) | `base:chicken` 8; `base:porkchop` 14; `base:beef` 18, favorite; `base:mutton` 14; `animals:kibble` 30, kibble |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: swipe, bite |
| reach past its body | 1 blocks |
| signature | skull_bite |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.8 |

## lungfish

*Ceratodus kempae*, Sarcopterygii › Dipnoi › Ceratodontidae › . Model `fish`, entity `animals:lungfish`.

| Where it lives | how common there |
|---|---|
| `animals:floodplain` | commonest |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.37 m at the fork |
| males against females | 1x as tall |
| size spread (an individual's, either way) | 60% |
| newborn's size | 10% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 3 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 30 d | 0.72 d |
| litter or clutch | 6 on average | 1-4 **set** |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 14 to 32 °C air | cold-blooded: slower in the cold |
| grows up in | 5 y | 10 d **set** |
| lives | 20 y | 76.7 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 14 within 48 blocks |

**Balance** (uncommon): a generation 11 d; a line four deep in 33 d (11 h of play); 4.7 young per mother in 10 days, 24 in her life.

- ✓ fourth generation: 32.9 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 10.0 of 77 days (want under a fifth)
- ✓ young in a life: 24.1 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 14 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | insects 50%, water plants 20%, fish 30% |
| a meal that fills it | 0.3 kg |
| drinks | never: it lives in the water |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 6 blocks (twice that running) |
| eats (food points) |  |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.5 blocks |
| signature | dart |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 1 |

## lycoptera

*Lycoptera davidi*, Actinopterygii › Osteoglossomorpha › Lycopteridae › . Model `fish`, entity `animals:lycoptera`.

| Where it lives | how common there |
|---|---|
| `animals:fern_forest` | commonest |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.37 m at the fork |
| males against females | 1x as tall |
| size spread (an individual's, either way) | 50% |
| newborn's size | 10% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 0.02 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 5 d | 0.19 d |
| litter or clutch | 8 on average | 3-11 |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 4 to 24 °C air | cold-blooded: slower in the cold |
| grows up in | 1 y | 2.92 d **set** |
| lives | 4 y | 15.3 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 30 **set** within 48 blocks |

**Balance** (common): a generation 3.4 d; a line four deep in 10 d (3 h of play); 13.1 young per mother in 10 days, 12 in her life.

- ✓ fourth generation: 10.1 days (want 10-25 days (common))
- ✓ grown most of its life: grows 2.9 of 15 days (want under a fifth)
- ✓ young in a life: 12.3 (want 6-40)
- ✓ litter: 3-11 (want 1-12, fewest first)
- ✓ room near: 30 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | insects 100% |
| a meal that fills it | 0.002 kg |
| drinks | never: it lives in the water |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 6 blocks (twice that running) |
| eats (food points) |  |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.5 blocks |
| signature | dart |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 1 |

## psittacosaurus

*Psittacosaurus lujiatunensis*, Dinosauria › Ceratopsia › Psittacosauridae › . Model `psittacosaurus`, entity `animals:psittacosaurus`.

| Where it lives | how common there |
|---|---|
| `animals:fern_forest` | commonest |
| `animals:floodplain` | 30% |
| `animals:scrubland` | 30% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | medium |
| height (grown, average) | 0.5 m at the hip |
| males against females | 1.05x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 15% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 25 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 75 d | 1.43 d |
| litter or clutch | 12 on average | 3-8 **set** |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 0 to 32 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 4 y | 6 d **set** |
| lives | 11 y | 42.2 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 20 **set** within 48 blocks |

**Balance** (common): a generation 7.7 d; a line four deep in 23 d (8 h of play); 10.3 young per mother in 10 days, 29 in her life.

- ✓ fourth generation: 23.0 days (want 10-25 days (common))
- ✓ grown most of its life: grows 6.0 of 42 days (want under a fifth)
- ✓ young in a life: 28.6 (want 6-40)
- ✓ litter: 3-8 (want 1-12, fewest first)
- ✓ room near: 20 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | leaves 60%, seeds 30%, fruit 10% |
| a meal that fills it | 2.5 kg |
| drinks | 100% of its 1.794 L a day; a drink lasts 21 min, thirsty after 10 |
| its food grows at home (spring/summer/autumn/winter) | fern_forest 40/40/40/40%, floodplain 40/40/40/40%, scrubland 40/40/40/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 12 blocks (twice that running) |
| eats (food points) | `animals:fern` 8; `animals:ginkgo_seed` 8; `base:berries` 10, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 1 blocks |
| signature | herd |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.9 |

## gastonia

*Gastonia burgei*, Dinosauria › Ankylosauria › Polacanthidae › . Model `ankylosaur`, entity `animals:gastonia`.

| Where it lives | how common there |
|---|---|
| `animals:floodplain` | commonest |
| `animals:scrubland` | 30% |
| `animals:fern_forest` | 30% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 1.25 m at the hip |
| males against females | 1.03x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 10% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 1900 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 100 d | 1.77 d |
| litter or clutch | 10 on average | 1-4 **set** |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 8 to 38 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 6 y | 6 d **set** |
| lives | 25 y | 95.8 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 12 **set** within 48 blocks |

**Balance** (common): a generation 8 d; a line four deep in 24 d (8 h of play); 4.7 young per mother in 10 days, 33 in her life.

- ✓ fourth generation: 24.1 days (want 10-25 days (common))
- ✓ grown most of its life: grows 6.0 of 96 days (want under a fifth)
- ✓ young in a life: 33.2 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 12 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | leaves 90%, fruit 5%, seeds 5% |
| a meal that fills it | 190 kg |
| drinks | 100% of its 88.412 L a day; a drink lasts 32 min, thirsty after 16 |
| its food grows at home (spring/summer/autumn/winter) | floodplain 40/40/40/40%, scrubland 40/40/40/40%, fern_forest 40/40/40/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 8 blocks (twice that running) |
| eats (food points) | `animals:fern` 10; `animals:cycad_seed` 12, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 1.5 blocks |
| signature | herd |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.9 |

## pinacosaurus

*Pinacosaurus grangeri*, Dinosauria › Ankylosauria › Ankylosauridae › . Model `ankylosaur`, entity `animals:pinacosaurus`.

| Where it lives | how common there |
|---|---|
| `animals:scrubland` | commonest |
| `animals:red_dunes` | 50% |
| `animals:crags` | 20% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 1.25 m at the hip |
| males against females | 1.03x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 10% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 1900 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 100 d | 1.77 d |
| litter or clutch | 10 on average | 1-4 **set** |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 0 to 40 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 6 y | 6 d **set** |
| lives | 25 y | 95.8 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 12 **set** within 48 blocks |

**Balance** (common): a generation 8 d; a line four deep in 24 d (8 h of play); 4.7 young per mother in 10 days, 33 in her life.

- ✓ fourth generation: 24.1 days (want 10-25 days (common))
- ✓ grown most of its life: grows 6.0 of 96 days (want under a fifth)
- ✓ young in a life: 33.2 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 12 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | leaves 80%, fruit 10%, roots 5%, insects 5% |
| a meal that fills it | 190 kg |
| drinks | 100% of its 88.412 L a day; a drink lasts 32 min, thirsty after 16 |
| its food grows at home (spring/summer/autumn/winter) | scrubland 40/40/40/40%, red_dunes 40/40/40/40%, crags 40/40/40/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 8 blocks (twice that running) |
| eats (food points) | `animals:fern` 10; `base:berries` 12, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 1.5 blocks |
| signature | herd |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.9 |

## iguanodont

*Iguanacolossus fortis*, Dinosauria › Ornithopoda › Styracosterna › . Model `iguanodont`, entity `animals:iguanodont`.

| Where it lives | how common there |
|---|---|
| `animals:floodplain` | commonest |
| `animals:fern_forest` | 60% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 2.2 m at the hip |
| males against females | 1.03x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 10% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 4000 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 110 d | 1.9 d |
| litter or clutch | 12 on average | 1-3 **set** |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 4 to 38 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 7 y | 6 d **set** |
| lives | 30 y | 115 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 12 **set** within 48 blocks |

**Balance** (common): a generation 8.2 d; a line four deep in 24 d (8 h of play); 3.8 young per mother in 10 days, 32 in her life.

- ✓ fourth generation: 24.5 days (want 10-25 days (common))
- ✓ grown most of its life: grows 6.0 of 115 days (want under a fifth)
- ✓ young in a life: 32.3 (want 6-40)
- ✓ litter: 1-3 (want 1-12, fewest first)
- ✓ room near: 12 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | leaves 85%, seeds 10%, fruit 5% |
| a meal that fills it | 400 kg |
| drinks | 100% of its 172.778 L a day; a drink lasts 35 min, thirsty after 17 |
| its food grows at home (spring/summer/autumn/winter) | floodplain 40/40/40/40%, fern_forest 40/40/40/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 10 blocks (twice that running) |
| eats (food points) | `animals:fern` 10; `animals:cycad_seed` 14, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 2 blocks |
| signature | herd |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.9 |

## crocodyliform

*Neosuchia indet.*, Crocodylomorpha › Crocodyliformes › Neosuchia › . Model `caiman`, entity `animals:crocodyliform`.

| Where it lives | how common there |
|---|---|
| `animals:floodplain` | commonest |
| `animals:fern_forest` | 40% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 2.2 m at the length |
| males against females | 1.2x as tall |
| size spread (an individual's, either way) | 67% |
| newborn's size | 15% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 150 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 80 d | 1.5 d |
| litter or clutch | 25 on average | 4-8 **set** |
| between litters | 1 y | rests 16.7 d **set** |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 20 to 36 °C air | cold-blooded: slower in the cold |
| grows up in | 10 y | 6.25 d **set** |
| lives | 50 y | 75 d **set**, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 20 **set** within 48 blocks |

**Balance** (uncommon): a generation 8 d; a line four deep in 24 d (8 h of play); 3.5 young per mother in 10 days, 19 in her life.

- ✓ fourth generation: 24.0 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 6.2 of 75 days (want under a fifth)
- ✓ young in a life: 19.1 (want 6-40)
- ✓ litter: 4-8 (want 1-12, fewest first)
- ✓ room near: 20 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | carnivore |
| eats | fish 60%, meat 30%, carrion 10% |
| hunts | 0.3%-100% of its mass alone (0.45-150 kg); at home: lungfish, psittacosaurus, confuciusornis |
| a meal that fills it | 15 kg; its biggest prey at home fills it in one kill |
| drinks | 100% of its 1.799 L a day; a drink lasts 125 min, thirsty after 63 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 6 blocks (twice that running) |
| eats (food points) | `animals:trout` 10; `animals:venison` 14, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | yes: bite |
| reach past its body | 1.2 blocks |
| signature | ambush |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.8 |

## citipati

*Citipati osmolskae*, Dinosauria › Theropoda › Oviraptoridae › Citipatiinae. Model `citipati`, entity `animals:citipati`.

| Where it lives | how common there |
|---|---|
| `animals:red_dunes` | commonest |
| `animals:scrubland` | 70% |

Spawns by day on grass, snow or sand, in groups of 1 (solitary), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | medium |
| height (grown, average) | 0.9 m at the hip |
| males against females | 1.05x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 12% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 90 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 70 d | 1.36 d |
| litter or clutch | 18 on average | 2-6 **set** |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 2 to 40 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 3 y | 6 d **set** |
| lives | 15 y | 57.5 d, give or take a fifth; an elder from 80% of it |
| lives in | solitary | room for 18 **set** within 48 blocks |

**Balance** (common): a generation 7.6 d; a line four deep in 23 d (8 h of play); 7.5 young per mother in 10 days, 30 in her life.

- ✓ fourth generation: 22.8 days (want 10-25 days (common))
- ✓ grown most of its life: grows 6.0 of 57 days (want under a fifth)
- ✓ young in a life: 30.0 (want 6-40)
- ✓ litter: 2-6 (want 1-12, fewest first)
- ✓ room near: 18 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | leaves 40%, seeds 25%, fruit 10%, meat 15%, insects 10% |
| hunts | 0.0%-5% of its mass alone (0.04-4 kg); at home: kryptobaatar, gobipteryx |
| a meal that fills it | 9 kg |
| drinks | 100% of its 5.681 L a day; a drink lasts 24 min, thirsty after 12 |
| its food grows at home (spring/summer/autumn/winter) | red_dunes 40/40/40/40%, scrubland 40/40/40/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 12 blocks (twice that running) |
| eats (food points) | `animals:cycad_seed` 8; `base:berries` 10, favorite; `base:wheat_seeds` 3 |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 1 blocks |
| signature | herd |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.9 |

## confuciusornis

*Confuciusornis sanctus*, Dinosauria › Avialae › Confuciusornithidae › . Model `early_bird`, entity `animals:confuciusornis`.

| Where it lives | how common there |
|---|---|
| `animals:fern_forest` | commonest |
| `animals:floodplain` | 60% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 2% of spawn attempts that pick it where commonest (common).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.3 m at the crown |
| males against females | 1.05x as tall |
| size spread (an individual's, either way) | 15% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 0.5 kg | |
| carries her young | 0 d | 0.7 d **set** |
| litter or clutch | 3 on average | 1-4 |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 0 to 34 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 1 y | 4.58 d **set** |
| lives | 10 y | 38.3 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 24 **set** within 48 blocks |

**Balance** (common): a generation 5.7 d; a line four deep in 17 d (6 h of play); 4.3 young per mother in 10 days, 11 in her life.

- ✓ fourth generation: 17.1 days (want 10-25 days (common))
- ✓ grown most of its life: grows 4.6 of 38 days (want under a fifth)
- ✓ young in a life: 11.3 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 24 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | omnivore |
| eats | fish 40%, seeds 40%, insects 20% |
| a meal that fills it | 0.05 kg |
| drinks | 100% of its 0.053 L a day; a drink lasts 14 min, thirsty after 7 |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 10 blocks (twice that running) |
| eats (food points) | `animals:cycad_seed` 4; `base:wheat_seeds` 3, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | yes:  |
| reach past its body | 0.5 blocks |
| signature | flock |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.6 |

## gobipteryx

*Gobipteryx minuta*, Dinosauria › Avialae › Enantiornithes › . Model `early_bird`, entity `animals:gobipteryx`.

| Where it lives | how common there |
|---|---|
| `animals:crags` | commonest |
| `animals:scrubland` | commonest |
| `animals:red_dunes` | 50% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | small |
| height (grown, average) | 0.3 m at the crown |
| males against females | 1.05x as tall |
| size spread (an individual's, either way) | 15% |
| newborn's size | 30% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 0.3 kg | |
| carries her young | 0 d | 0.7 d **set** |
| litter or clutch | 3 on average | 1-4 |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 0 to 40 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 1 y | 6.25 d **set** |
| lives | 10 y | 38.3 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 24 **set** within 48 blocks |

**Balance** (uncommon): a generation 7.4 d; a line four deep in 22 d (7 h of play); 4.3 young per mother in 10 days, 11 in her life.

- ✓ fourth generation: 22.1 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 6.2 of 38 days (want under a fifth)
- ✓ young in a life: 10.6 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 24 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | omnivore |
| eats | insects 50%, seeds 40%, leaves 10% |
| a meal that fills it | 0.03 kg |
| drinks | 100% of its 0.033 L a day; a drink lasts 13 min, thirsty after 7 |
| its food grows at home (spring/summer/autumn/winter) | crags 40/40/40/40%, scrubland 40/40/40/40%, red_dunes 40/40/40/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 10 blocks (twice that running) |
| eats (food points) | `animals:cycad_seed` 4; `base:wheat_seeds` 3, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 0.5 blocks |
| signature | flock |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.6 |

## cedarosaurus

*Cedarosaurus weiskopfae*, Dinosauria › Sauropoda › Brachiosauridae › . Model `sauropod`, entity `animals:cedarosaurus`.

| Where it lives | how common there |
|---|---|
| `animals:floodplain` | commonest |
| `animals:fern_forest` | 50% |

Spawns by day on grass, snow or sand, in groups of 3-5 (herd), at 0.8% of spawn attempts that pick it where commonest (uncommon).

| Body | |
|---|---|
| size class | large |
| height (grown, average) | 4.1 m at the shoulder |
| males against females | 1.03x as tall |
| size spread (an individual's, either way) | 10% |
| newborn's size | 5% of grown |

| Life | real | in the game |
|---|---|---|
| mass | 10000 kg | |
| lays | | 0.25 d after mating |
| eggs hatch in | 120 d | 2.03 d |
| litter or clutch | 25 on average | 1-2 **set** |
| between litters | 1 y | rests 5.07 d |
| breeds in | the wet season | out of it, only a tame well cared for |
| comfortable in | 6 to 40 °C air | seeks shade, water or warmth outside it; a tame eats faster, far outside loses health |
| grows up in | 10 y | 6.67 d **set** |
| lives | 40 y | 153.3 d, give or take a fifth; an elder from 80% of it |
| lives in | herd | room for 10 **set** within 48 blocks |

**Balance** (uncommon): a generation 8.9 d; a line four deep in 27 d (9 h of play); 2.8 young per mother in 10 days, 33 in her life.

- ✓ fourth generation: 26.8 days (want 20-40 days (uncommon))
- ✓ grown most of its life: grows 6.7 of 153 days (want under a fifth)
- ✓ young in a life: 32.7 (want 6-40)
- ✓ litter: 1-2 (want 1-12, fewest first)
- ✓ room near: 10 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | leaves 95%, seeds 5% |
| a meal that fills it | 1000 kg |
| drinks | 100% of its 394.126 L a day; a drink lasts 38 min, thirsty after 19 |
| its food grows at home (spring/summer/autumn/winter) | floodplain 40/40/40/40%, fern_forest 40/40/40/40% |
| feedings to tame at level 1 | 0 (2.5x at level 150) |
| scared by a walking player within | 6 blocks (twice that running) |
| eats (food points) | `animals:fern` 10; `animals:araucaria_cone` 16, favorite |

| Fighting, riding, the wild | |
|---|---|
| fights | no |
| reach past its body | 3 blocks |
| signature | herd |
| ridden | no |
| flies ridden | no |
| camouflage (cost of a coat that stands out) | 0.9 |

## Shared by every species

| Levels | health / stamina / food / damage / speed / weight / oxygen / torpor / fortitude |
|---|---|
| a wild point adds (share of the base) | 0.2 / 0.1 / 0.1 / 0.05 / 0.01 / 0.02 / 0.1 / 0.06 / 0.05 |
| a tamed point adds | 0.1 / 0.1 / 0.1 / 0.04 / 0.015 / 0.04 / 0.1 / 0 / 0.05 |

| Pacing: real days to game days (`biology.rs`) | |
|---|---|
| carrying, incubating | 0.056 · real days^0.75 |
| growing up | 0.0645 · real days^0.7 |
| a life | 0.0105 · real days^1 |
| rest between litters | 0.0139 · real days^1 |
| room near | 13 · (mass / 4 kg)^-0.2, x1.2 a pack, x2 a herd, at least 3 |

| Hunger (`needs.rs`) | |
|---|---|
| a meal that fills it | 10% of its mass: a kill, a mouthful off the ground, by mass |
| a wild one arrives fed | 30% where its food grows least (or its prey are thinnest), all of it where most |
| starving, a wild one dies in | 1200 s on an empty stomach |
| water needed a day | 0.099 · kg^0.9 L (a mammal's), x0.2 cold-blooded; a species drinks its share of it (`DRINKS`) |
| a drink lasts | 15% of its mass spared, a real day of drinking 600 s of play |
| thirsty | under 50% water, drinking 25% a gulp every 1.5 s until 95%; parched, a wild one dies in 1200 s |

| Breeding and life (`life.rs`) | |
|---|---|
| in love for | 30 s |
| a male rests after mating | 5 min |
| a wild one falls in love, with room | 0.5 times a day |
| breeding when at its room | 10% of that |
| no breeding past | 3x its room |
| deaths of want, at twice its room | 10% of grown ones a day (elders x3, young x2) |
| elders crowd | half |
| time away caught up past | 30 s |

| Evolution (`evolution.rs`) | |
|---|---|
| a place | 256 blocks a side, per habitat |
| a generation | 1 d |
| population a place | 200 |
| selection strength | 0.6 |
| mutation a gene a generation | 0.002 |
| migration from neighbors | 0.02 |
| bigger in the cold (Bergmann) | 0.15 |
| a new place's history | 300 generations (a revisited one catches up 120) |

| Spawning (`habitats.rs`, `biology.rs`) | |
|---|---|
| common | 2% of attempts that pick it, where commonest |
| uncommon | 0.8% of attempts that pick it, where commonest |
| rare | 0.4% of attempts that pick it, where commonest |
| legendary | 0.06% of attempts that pick it, where commonest |
| solitary | groups of 1 |
| pack | groups of 2-3 |
| herd | groups of 3-5 |

| Balance targets (`balance.rs`): a line four generations deep | |
|---|---|
| common | 10-25 d |
| uncommon | 20-40 d |
| rare | 35-70 d |
| legendary | 80-160 d |
