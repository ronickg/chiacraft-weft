# Animals: every species' settings

Written from the game's species table (`core/src/species.rs`) by `core/src/sheet.rs`; don't edit it by hand. After changing a species, rewrite it:

    mods/animals/web/build.sh && node mods/animals/web/sheet.mjs

`node mods/animals/web/test.mjs` fails while it is out of date. Time is in in-game days (24 000 ticks, 20 minutes of play); "real" is the real animal's. Breeding numbers are derived from the real facts (`core/src/biology.rs`) unless a species sets one itself (marked **set**). Why the numbers are what they are: `notes/ANIMALS.md`. Coat genes: `genetics/<species>.json` and `genetics/core.json`.

## At a glance

| Species | Scientific name | Rarity | Size | Height | Mass | Lives in | Diet | Taming | Carrying | Grown in | Lives | Litter | Room |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| dog (chihuahua) | *Canis familiaris* | common | small | 0.2 m | 2.5 kg | pack | carnivore | passive | 1.25 d | 3.5 d | 49.9 d | 1-4 | 17 |
| cat | *Felis catus* | common | small | 0.24 m | 4.5 kg | solitary | carnivore | passive | 1.28 d | 2.99 d | 49.9 d | 1-4 | 13 |
| capybara | *Hydrochoerus hydrochaeris* | common | medium | 0.56 m | 50 kg | herd | herbivore | passive | 2.28 d | 5.34 d | 30.7 d | 2-6 | 16 |
| raptor | *Velociraptor mongoliensis* | uncommon | medium | 0.5 m | 15 kg | solitary | carnivore | knockout | eggs, hatch 1.41 d | 7.54 d | 76.7 d | 1-4 | 10 |
| utahraptor | *Utahraptor ostrommaysi* | rare | large | 1.5 m | 300 kg | pack | carnivore | knockout | eggs, hatch 2.28 d | 12.3 d | 153.3 d | 1-3 | 7 |
| osmolskae | *Velociraptor osmolskae* | uncommon | medium | 0.5 m | 15 kg | solitary | carnivore | knockout | eggs, hatch 1.41 d | 7.54 d | 76.7 d | 1-4 | 10 |
| wyvern | *Wyverna montana* | legendary | large | 3.9 m | 5000 kg | solitary | carnivore | knockout | eggs, hatch 5.47 d | 32.7 d | 421.6 d | 1 | 3 |
| hare | *Lepus europaeus* | common | small | 0.3 m | 3.8 kg | solitary | herbivore | wild | 0.92 d | 2.81 d | 19.2 d | 1-4 | 13 |
| mountain_hare | *Lepus timidus* | common | small | 0.3 m | 3 kg | solitary | herbivore | wild | 1.05 d | 3.5 d | 19.2 d | 1-4 | 14 |
| wood_mouse | *Apodemus sylvaticus* | common | small | 0.05 m | 0.023 kg | solitary | herbivore | wild | 0.63 d | 3 d | 16 d | 2-6 | 24 |

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
| hunts | 0.8%-80% of its mass alone (0.02-2 kg); at home: wood_mouse, base:chicken |
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
| lives in | solitary | room for 13 within 48 blocks |

**Balance** (common): a generation 4.3 d; a line four deep in 13 d (4 h of play); 6.2 young per mother in 10 days, 23 in her life.

- ✓ fourth generation: 12.8 days (want 10-25 days (common))
- ✓ grown most of its life: grows 3.0 of 50 days (want under a fifth)
- ✓ young in a life: 22.7 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 13 (want 3-30)

| Taming and food | |
|---|---|
| how | passive |
| diet | carnivore |
| eats | meat 80%, insects 10%, carrion 5%, fish 5% |
| hunts | 0.5%-40% of its mass alone (0.02-2 kg); at home: wood_mouse, base:chicken |
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
| `base:forest` | 20% |

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
| lives in | herd | room for 16 within 48 blocks |

**Balance** (common): a generation 7.6 d; a line four deep in 23 d (8 h of play); 5.4 young per mother in 10 days, 10 in her life.

- ✓ fourth generation: 22.9 days (want 10-25 days (common))
- ✓ grown most of its life: grows 5.3 of 31 days (want under a fifth)
- ✓ young in a life: 10.4 (want 6-40)
- ✓ litter: 2-6 (want 1-12, fewest first)
- ✓ room near: 16 (want 3-30)

| Taming and food | |
|---|---|
| how | passive |
| diet | herbivore |
| eats | grass 75%, water plants 20%, fruit 5% |
| its food grows at home (spring/summer/autumn/winter) | swamp 8/59/45/0%, savanna 43/82/75/19%, forest 11/73/41/0% |
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
| hunts | 0.2%-600% of its mass alone (0.03-90 kg); at home: none yet: it waits for its prey to come to the world |
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
| hunts | 1.0%-300% of its mass alone (3.00-900 kg); at home: none yet: it waits for its prey to come to the world |
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
| hunts | 0.2%-600% of its mass alone (0.03-90 kg); at home: none yet: it waits for its prey to come to the world |
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
| lives in | solitary | room for 3 within 48 blocks |

**Balance** (legendary): a generation 38.4 d; a line four deep in 115 d (38 h of play); 0.6 young per mother in 10 days, 20 in her life.

- ✓ fourth generation: 115.1 days (want 80-160 days (legendary))
- ✓ grown most of its life: grows 32.7 of 422 days (want under a fifth)
- ✓ young in a life: 19.7 (want 6-40)
- ✓ litter: 1-1 (want 1-12, fewest first)
- ✓ room near: 3 (want 3-30)

| Taming and food | |
|---|---|
| how | knockout |
| diet | carnivore |
| eats | meat 80%, fish 10%, carrion 10% |
| hunts | 0.1%-60% of its mass alone (5.00-3000 kg); at home: raptor, utahraptor, osmolskae |
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
| lives in | solitary | room for 13 within 48 blocks |

**Balance** (common): a generation 3.7 d; a line four deep in 11 d (4 h of play); 14.2 young per mother in 10 days, 18 in her life.

- ✓ fourth generation: 11.2 days (want 10-25 days (common))
- ✓ grown most of its life: grows 2.8 of 19 days (want under a fifth)
- ✓ young in a life: 17.8 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 13 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | grass 60%, leaves 30%, roots 10% |
| its food grows at home (spring/summer/autumn/winter) | plains 13/53/46/0%, desert 40/40/40/40%, forest 40/73/41/0%, savanna 43/82/75/40%, beach 14/20/50/33% |
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
| lives in | solitary | room for 14 within 48 blocks |

**Balance** (common): a generation 4.5 d; a line four deep in 14 d (5 h of play); 12.3 young per mother in 10 days, 15 in her life.

- ✓ fourth generation: 13.6 days (want 10-25 days (common))
- ✓ grown most of its life: grows 3.5 of 19 days (want under a fifth)
- ✓ young in a life: 14.6 (want 6-40)
- ✓ litter: 1-4 (want 1-12, fewest first)
- ✓ room near: 14 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | grass 40%, leaves 55%, lichen 5% |
| its food grows at home (spring/summer/autumn/winter) | taiga 40/62/40/40%, birch_forest 40/62/40/0%, snowy_plains 40/40/40/40%, mountains 40/70/40/40% |
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
| lives in | solitary | room for 24 **set** within 48 blocks |

**Balance** (common): a generation 3.6 d; a line four deep in 11 d (4 h of play); 38.3 young per mother in 10 days, 38 in her life.

- ✓ fourth generation: 10.9 days (want 10-25 days (common))
- ✓ grown most of its life: grows 3.0 of 16 days (want under a fifth)
- ✓ young in a life: 37.6 (want 6-40)
- ✓ litter: 2-6 (want 1-12, fewest first)
- ✓ room near: 24 (want 3-30)

| Taming and food | |
|---|---|
| how | wild |
| diet | herbivore |
| eats | seeds 45%, insects 20%, fruit 15%, grass 10%, fungi 5%, roots 5% |
| its food grows at home (spring/summer/autumn/winter) | plains 13/53/46/0%, forest 11/73/41/0%, desert 40/40/40/40%, savanna 43/82/75/19%, beach 14/40/50/33% |
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
