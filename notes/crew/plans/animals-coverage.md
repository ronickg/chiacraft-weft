# Animal AI coverage (11-25)

"not tested" marks anything the runs did not show; "pass" only what a log line or a where row showed.

Planner's task (from Ronald: "with animals, then you need to test more than just wolves"): every species'
AI checked on the Mac, one check per behaviour group (`mods/testkit/scripts/cov_*.txt`, written by
`tools/animals/coverage_scripts.py`), the results here, the failures fixed biggest first.

Marks: **pass** (seen, the log line or the where rows show it), **fail** (tried, didn't happen or went
wrong), **-** (not tested in these runs), **n/a** (not this species' behaviour). The Cretaceous's
species are checked on flattened overworld pads (there is no command to send a player there).

Runs: commit 1ff853abf (branch claude/animals-coverage), seed 1, fresh worlds, `mod:animals=debug`.

## Grazers and herds (cov_grazers.txt)

| Species | eats | drinks | flees | herd holds | shot |
|---|---|---|---|---|---|
| white_tailed_deer | pass | pass (lines; water was full) | fail: only the hit one ran | not tested (out of the rows' range) | grid 1 |
| red_deer | pass | not tested (water 30 %, ate instead) | not tested (not hit) | pass, 4-5 blocks | grid 1 |
| reindeer | pass (2 of 3) | not tested (ate instead) | not tested (not hit) | pass, 4 blocks (one row) | grid 2 |
| capybara | pass | pass for 1 of 3 (water rose 30 to 38) | fail: only the hit one ran | pass, 3-5 blocks | grid 2 |
| protoceratops | pass | pass | fail: only the hit one ran | pass, 1-2 blocks | grid 3 |
| psittacosaurus | pass | pass (water 30 to 99) | not tested (not hit) | pass, 1-8 blocks | grid 3 |
| gastonia | pass | pass | not tested (not hit) | pass, 3 blocks | grid 4 |
| pinacosaurus | pass | pass (30 to 77, 100) | fail: ran, never stood (1900 kg) | pass, 2-4 blocks | grid 4 |
| iguanodont | pass | pass | fail: ran, never stood (4 t) | pass, 7 blocks | grid 5 |
| cedarosaurus | pass | pass (30 to 100) | not tested (not hit) | pass, 3-4 blocks | grid 5 |
| citipati | pass (seeds off the ground) | pass | pass (ran) | n/a (two kept within 2) | grid 6 |

cov_grazers, Mac, 494.9 s (`~/Coding/chiacraft-archive/tester-tools/runs/cov_grazers_m-154047.log`;
grid `~/Coding/chiacraft-archive/images/tester/2026-11-25/cov_grazers_m-154912/cov_grazers_grid.jpg`,
poor: tall grass hid the animals and parts 5-6 were at dusk; the scripts now clear it and set noon).
Fixed on the branch (e91a79201, to re-run): a hit animal now calls the alarm and its herd runs with
it; a grown plant eater of 150 kg or more (red deer, the ankylosaurs, iguanodont, sauropod) turns on
what hit it and charges instead of running. Also seen: 12 pose clips missing (the client's "no clip
for action": groom on deer_lie; watch, graze, drink on capybara_sit; watch, scratch, groom, look on
reindeer_lie; play, stretch on psittacosaurus_lie; groom, scratch, look on ankylosaur_lie; scratch
on citipati_lie): art for blocky-model.

Run 2 (b7ef3aaf7, `r2_grazers_m-172424`, 486 s). All six hit animals ran ("flees from what hurt it") and five
called the alarm with the right count of hearers, but only the reindeer's hearers ran: the others stood
staring at the player. The shy check (`taming::behave`) stopped them to stare before `hunt::wary` could run
them off with their herd; fixed (a fresh alarm goes first: `senses::alarmed`). Every pad drank to full. Grazing
was not shown: the pads had no grass plants since e91a79201 (tall grass hid the shots), and the turf's roots
and insects are a top-up only (`fills_to`); the pads now have tall grass on their far strip. Herds held
together (3 to 28 blocks).

| Species | eat | drink | flee when hit | herd runs on the alarm | holds together | shot |
|---|---|---|---|---|---|---|
| red_deer | not tested (no grass) | pass | pass | fail (stared; fixed) | pass | r2 |
| white_tailed_deer | not tested | pass | not tested | not tested | pass (up to 20) | r2 |
| reindeer | not tested | pass | pass | pass (2 ran) | pass (2-9) | r2 |
| protoceratops | not tested | pass | pass | fail (stared; fixed) | pass (up to 25) | r2 |
| pinacosaurus | not tested | pass | pass | fail (stared; fixed) | pass | r2 |
| cedarosaurus | not tested | pass | pass (young: runs) | fail (stared; fixed) | pass | r2 |
| citipati | not tested | pass | pass | n/a (not a herd kind) | pass (up to 24) | r2 |

Grid r2: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r2_grazers_m-172424/r2_grazers_grid.jpg`.

Run 3 (46f12cbd7, `r11_grazers_m`, 486 s, script pass). Every pad grazes: 171 "eats base:tall_grass /
base:grass" lines (deer 18, reindeer and capybara 12, protoceratops and psittacosaurus 49, the ankylosaurs
24, iguanodont and cedarosaurus 52, citipati 16). Each hit calls the alarm, and the herd runs with the hit one.
Few hearers log "runs from" (they run through the alarm, not their own look round), so this was read from the
where rows before and after: the red deer moved 20 blocks with the hit one, protoceratops 6-10, iguanodont 83
12, capybara 34 7-9. Capybara 32 stayed: she is the mother of the hit young 33 and stands between (a
mother over `GUARDS_KG`), by design. The hit gastonia turned on the player and charged ("turns on it",
"holds charge"). The hit iguanodont was a young one, so it ran, by design.

| Species | eat | drink | flee when hit | herd runs on the alarm | shot |
|---|---|---|---|---|---|
| white_tailed_deer | pass | pass (r2) | not tested (not hit) | not tested; bolted from the player on its own | r11 1 |
| red_deer | pass | pass | pass | pass (both moved 20 blocks) | r11 1 |
| reindeer | pass | pass (r2) | not tested (not hit in r11) | pass (r2) | r11 2 |
| capybara | pass | pass | pass (the young) | pass (34 ran; the mother stood guard) | r11 2 |
| protoceratops | pass | pass | pass | pass (both moved 6-10) | r11 3 |
| psittacosaurus | pass | pass | not tested (not hit); bolted on its own | not tested | r11 3 |
| gastonia | pass | pass | pass: turns on it and charges | n/a (stands) | r11 4 |
| pinacosaurus | pass | pass | pass (r2) | not tested in r11 | r11 4 |
| iguanodont | pass | pass | pass (a young one runs) | pass (83 moved 12) | r11 5 |
| cedarosaurus | pass | pass | not tested (not hit); bolted on its own | not tested | r11 5 |
| citipati | pass | pass | pass | n/a (not a herd kind) | r11 6 |

Grid r11: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r11_grazers_m-183049/r11_grazers_grid.jpg`.

## Hunters other than wolves (cov_hunters.txt)

| Species | hunts | stalks / ambush | chase | kill | share | gives up | shot |
|---|---|---|---|---|---|---|---|
| jaguar | pass (deer, 56 kg) | pass (stalk, "breaks cover", 9 blocks) | pass (60 s, off the pad) | pass (70 kg, full) | n/a | pass ("too long" at 61 s, then a second hunt killed) | grid 1 |
| coyote | pass (37: a hare) | n/a | pass (2 s) | pass (3.8 kg) | fail: 36 never joined | not tested | grid 2 |
| red_fox | pass (4 hunts) | n/a | pass | pass (2 mice, a vole) | n/a | not tested | grid 3 |
| arctic_fox | pass | n/a | pass | pass (a vole) | n/a | not tested | grid 3 |
| raptor | pass | n/a | pass | pass (kryptobaatar, Priscagama) | n/a | pass ("gone": the other took it) | grid 4 |
| osmolskae | pass | n/a | pass | pass (kryptobaatar) | n/a | pass ("gone") | grid 4 |
| utahraptor | pass (a pack of 3, two "join") | not tested | pass (10 s) | pass (Psittacosaurus) | pass (8.3 kg each of 25) | not tested | grid 5 |
| crocodyliform | pass (took a spectacled caiman) | pass ("breaks cover", 6 blocks) | pass | pass (25 kg, full) | n/a | not tested | grid 6 (dark) |
| yacare_caiman | not tested (ran from the crocodyliform; no prey its size) | not tested | not tested | not tested | n/a | not tested | grid 6 |
| spectacled_caiman | not tested (killed by the crocodyliform at 1.5 s) | not tested | not tested | not tested | n/a | not tested | grid 6 |

cov_hunters, Mac, 557.2 s (`~/Coding/chiacraft-archive/tester-tools/runs/cov_hunters_m-154912.log`;
grid `~/Coding/chiacraft-archive/images/tester/2026-11-25/cov_hunters_m-155831/cov_hunters_grid.jpg`, parts 4-6
dark). Fixed (91a328628): coyote 36 ate the insects in the grass 33 times, from 28 to 90 %, and never
hunted or joined its pack: a fallback food (insects, roots, lichen) now fills a hunter only to its
share of the diet. Seen and left: the raptors stopped hunting with two prey left, both young (a few
grams: under what they take); kills of mice and voles printed "0.0 of 0.0 kg" (now grams). Missing
clips: pounce on jaguar; graze on coyote, both foxes, both raptors (they don't graze: the anim asks
for one); bite on raptor_rest; look, scratch on hare_lie; play on vole_sit, kryptobaatar_sit,
psittacosaurus_lie, caiman_bask; groom, sniff, play on lizard_bask; watch, scratch, groom, look,
stretch on capybara_sit; groom, sniff on caiman_bask.

Run 2 (b7ef3aaf7, `r2_hunters_m-171045`, 557 s). The jaguar stalked, broke cover and killed a deer in 20 s; a
coyote killed a hare in 6 s (the other gave up on it, gone); the foxes and the osmolskae took two small prey
each and none filled up on insects (fills_to holds); the three utahraptors hunted as a pack and shared 8.3 kg
each; the crocodyliform broke cover and took the yacare caiman (in its range, 17.9 kg) over the capybaras.
The raptor chased a kryptobaatar 60 s, gave up ("too long") and tried again: small fast prey escaping, left as
is. The two caimans never hunted: their prey is 0.17 to 35 kg and the capybaras (50 kg) are too big, so
there was no prey for them on the pad (run 3: their own pad, with hares and pacu).

| Species | hunts | stalk / ambush | chase | kill, share | gives up | shot |
|---|---|---|---|---|---|---|
| jaguar | pass | pass ("breaks cover", 8.9 blocks) | pass | pass (70 kg deer, full) | n/a | r2 1 |
| coyote | pass (two, each alone) | not tested | pass | pass (3.8 kg hare) | pass ("gone") | r2 2 |
| red_fox | pass | not tested | pass | pass (2 voles) | n/a | r2 3 |
| arctic_fox | pass | not tested | pass | pass (2 mice) | n/a | r2 3 |
| raptor | pass | not tested | pass | none in 60 s | pass ("too long") | r2 4 (dark) |
| osmolskae | pass | not tested | pass | pass (a priscagama, a kryptobaatar) | n/a | r2 4 (dark) |
| utahraptor | pass (a pack of 3, two join) | not tested | pass | pass (8.3 kg each) | n/a | r2 5 |
| crocodyliform | pass | pass ("breaks cover", 8.1 blocks) | pass | pass (35 kg caiman, full) | n/a | r2 6 |
| yacare_caiman | not tested (no prey its size) | not tested | not tested | not tested | not tested | r2 6 |
| spectacled_caiman | not tested (no prey its size) | not tested | not tested | not tested | not tested | r2 6 |

Grid r2: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r2_hunters_m-171045/r2_hunters_grid.jpg`.

Run 3 (8fe0410c3, `r7_hunters_m-181008`, 649 s): every hunter but the utahraptors and the caimans killed and
ate again: the jaguar (stalk, cover, a deer), a coyote (a hare; the other ate its share off the ground), both
foxes (a vole and mice; the red fox first gave one up, gone), the raptor (three small prey), the osmolskae,
and the crocodyliform (broke cover, a capybara). The utahraptor pack chased a psittacosaurus 60 s, gave up
("too long") and began again: no kill this time (run 2: a kill shared three ways). The caimans had prey their
size (hares, pacu) but didn't hunt: the yacare rested (caimans hunt by night, and at 29 % it wasn't driven)
and the spectacled was young (its prey sizes go by its mass). Run 4: their pad at night, both grown first.
Grid r3: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r7_hunters_m-181008/r7_hunters_grid.jpg`.

Run 4 (46f12cbd7, `r11_hunters_m`, 651 s, script pass). Eleven kills, all by hunters. The coyotes took a hare.
The foxes took two voles and a mouse, and the raptor and osmolskae three kryptobaatars. The utahraptors hunted as
a pack of 3 and shared a psittacosaurus (8.3 kg each), and the crocodyliform stalked, broke cover and took a
capybara. The grown caimans hunted at night: both prowled, broke cover and took a pacu each. The jaguar
stalked, broke cover and wounded its deer five times, but the deer ran 60 blocks off the pad: two "too long"
give-ups and no kill in its 110 s (runs 1 and 2 killed). Found and fixed (d62af1ac5): a kill
fed its kind's grown mass whatever the prey weighed, because the dead can't be read at the death event. The
crocodyliform hunted a young 2.7 kg capybara and ate 50 kg. The hunter now keeps its prey's mass from the
start of the hunt. New in the warnings: 10 "neighbor update storm" lines (the pad and pond fills). Shot 7 is
black: night, nothing lit.

| Species | hunts | stalk / ambush | chase | kill, eat, share | gives up | shot |
|---|---|---|---|---|---|---|
| jaguar | pass | pass (stalk, "breaks cover" 8.9 blocks) | pass (5 wounds) | not tested (deer escaped; runs 1-2 pass) | pass ("too long", hunts again) | r4 1 (no animal in view) |
| coyote | pass | n/a | pass | pass (hare, full) | pass ("gone") | r4 2 |
| red_fox, arctic_fox | pass | n/a | pass | pass (2 voles, a mouse) | pass ("gone") | r4 3 |
| raptor, osmolskae | pass | n/a | pass | pass (3 kryptobaatars) | pass ("gone") | r4 4 |
| utahraptor | pass (a pack of 3) | n/a | pass | pass (8.3 kg each) | n/a | r4 5 |
| crocodyliform | pass | pass (stalk, 9 blocks) | pass | pass, but young prey eaten as grown (fixed d62af1ac5) | n/a | r4 6 |
| yacare_caiman | pass (at night) | pass (2.2-6.8 blocks) | pass | pass (a pacu) | pass ("gone", then the next) | r4 7 (black) |
| spectacled_caiman | pass (at night) | pass (8 blocks) | pass | pass (a pacu) | n/a | r4 7 (black) |

Grid r4: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r11_hunters_m-191009/r11_hunters_grid.jpg`.

## Birds and fliers (cov_birds.txt)

| Species | flies | lands | flocks | flees | hunts / scavenges | shot |
|---|---|---|---|---|---|---|
| house_sparrow | short hops (to 72) | pass | pass, 5 blocks | pass ("bolts from player") | n/a | grid 1 |
| snow_bunting | pass (to 80) | not tested (comfort all run) | fail: 18 blocks apart | pass | n/a | grid 1 |
| confuciusornis | pass (to 79) | pass | pass, 5 blocks | pass | n/a | grid 1 |
| gobipteryx | pass (to 78) | pass | 20 blocks at widest | not tested (no bolt) | n/a | grid 1 |
| golden_eagle | fail: on foot all run | n/a | n/a | not tested | pass on foot (a hare; gave up one "too long") | grid 2 |
| savanna_hawk | fail: on foot all run | n/a | n/a | not tested | pass on foot (a mouse) | grid 2 |
| great_horned_owl | one row at 77 | not tested | n/a | not tested | pass (a mouse) | grid 3 |
| snowy_owl | fail: on foot all run | not tested | n/a | not tested | pass on foot (a mouse) | grid 3 |
| turkey_vulture | pass (to 86) | pass | not tested | not tested | pass ("smells carrion" 19-20 blocks, ate venison) | grid 4 |
| raven | fail: y 69-70 only | not tested | n/a | not tested | pass by food (29 to 100 %, no line) | grid 4 |
| herring_gull | pass (to 86) | not tested | 15 blocks | not tested | pass by food (no line) | grid 4 |

cov_birds, Mac, 345.8 s (`~/Coding/chiacraft-archive/tester-tools/runs/cov_birds_m-155832.log`; grid
`~/Coding/chiacraft-archive/images/tester/2026-11-25/cov_birds_m-160420/cov_birds_grid.jpg`, no bird
visible in any shot). Fixed (birds.rs, after 91a328628): birds of prey hunted on foot; a hungry one
now soars to search, flies at prey it sees and stoops on it. The `hit 2` killed a sparrow (1 health):
flight from a hit isn't testable on songbirds that way. Missing clips: groom, play, alarm, sniff,
stare on small_bird_fly; look, groom, alarm on early_bird_fly; alarm, sniff on owl_fly; groom, look
on gull_fly and vulture_fly; play on hare_lie; alarm, sniff, scratch, look on vole_sit; scratch on
mouse_sit (idle clips asked for in the air: the fly looks should be left out of idle life).

Run 2 (b7ef3aaf7, `r2_birds_m-170124`, 346 s). Songbirds as in run 1. The birds of prey now take to the air
for prey ("flies at its prey", "soars to look for prey") but killed nothing: the eagle and both owls glided
down onto where the prey had been, landed short, lifted again (about 14 times each), the eagle once hurting a
hare on the ground. Fixed: in the air it stays on the prey, comes down low over it from 6 blocks
(`roles::STOOP_LOW`) and strikes it within 1.6 blocks once a second ("strikes N from the air"); the kill is
hunt.rs's as on foot. Scavengers: one vulture and both gulls ate the deer's venison; the second vulture came
when it was eaten; the raven found the food but couldn't reach it ("can't reach the food"). Eight clips were
still asked of birds in flight (held states and a clip playing as it took off): held states now wait for the
ground, and taking off stops what played. The pads' shots were hidden by a hill south of the pad (now cleared).

| Species | air hunt (run 2) | kill from the air |
|---|---|---|
| golden_eagle | pass ("flies at its prey", 3 hunts, a hare hurt) | fail: none (fixed, to re-run) |
| savanna_hawk | soared once, no prey taken | not tested |
| great_horned_owl | pass ("flies at its prey", hunts a mouse and a vole) | fail: none (fixed, to re-run) |
| snowy_owl | pass (as the great horned owl) | fail: none (fixed, to re-run) |
| turkey_vulture | pass (smells carrion, eats 200 g venison) | n/a |
| raven | soars for prey; can't reach the food | not tested |
| herring_gull | pass (eats venison off the ground) | n/a |

Grid r2: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r2_birds_m-170124/r2_birds_grid.jpg`.

Run 3 (8fe0410c3, `r7_birds_m-175912`, 346 s): kills from the air. The golden eagle flew at a hare 14 blocks
off and struck it three times from the air ("strikes 45 from the air"), killing it; the great horned owl
struck a vole at night and killed it. The hawk and the snowy owl soared to look but struck nothing in the
time. Both vultures smelled the deer and ate its venison; the raven and the gull ate (where rows) without a
smell line. Songbirds bolted from the walk-in (4 to 19 blocks off) and flew up to 74-81. One "no clip" in
the whole run, none on a flying look (run 2: 8): the fix holds.

| Species | air hunt (run 3) | kill from the air |
|---|---|---|
| golden_eagle | pass | pass (a hare, 3 strikes) |
| savanna_hawk | soared, no strike | not tested |
| great_horned_owl | pass | pass (a vole) |
| snowy_owl | soared, no strike | not tested |
| turkey_vulture | pass (both smell and eat) | n/a |
| songbirds (4 kinds) | pass (bolt from the walk-in, fly) | n/a |

Grid r3: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r7_birds_m-175912/r7_birds_grid.jpg`.

## Small animals and fish (cov_small.txt)

| Species | eats | drinks | wanders | flees | shot |
|---|---|---|---|---|---|
| hare | pass (tall grass) | pass (water full) | not tested (ate all run) | pass ("bolts") | grid 1 |
| mountain_hare | pass | pass | not tested (comfort) | pass | grid 1 |
| wood_mouse | pass (grass, seeds) | pass for 1 of 3 | pass (1) | pass | grid 1 |
| field_vole | pass | pass (water 30 to 84-100) | pass (1) | pass (1 of 3) | grid 1 |
| kryptobaatar | fail: never hungry | fail: never thirsty | not tested (rested: nocturnal) | pass | grid 2 |
| priscagama | fail: never hungry | fail: never thirsty | pass | pass | grid 2 |
| cat | pass, but on insects in the grass | pass (1 of 2) | not tested | pass | grid 3 |
| dog (chihuahua) | pass, but on seeds in the tall grass | pass | pass | pass (ran from a hit) | grid 3 |
| brown_trout | not tested | n/a | pass (swims) | not tested (no dart seen) | grid 4 (dark) |
| pacu | not tested | n/a | pass | not tested | grid 4 |
| lungfish | not tested | n/a | pass | not tested | grid 4 |
| lycoptera | not tested | n/a | 1 of 3 (two spawned on the water's top suffocated: the script's) | not tested | grid 4 |

cov_small, Mac, 326.5 s (`~/Coding/chiacraft-archive/tester-tools/runs/cov_small_m-160421.log`; grid
`~/Coding/chiacraft-archive/images/tester/2026-11-25/cov_small_m-160950/cov_small_grid.jpg`). Fixed:
insect eaters (kryptobaatar, Priscagama) were neither plant eaters nor hunters with prey, so they
never got hungry or thirsty: one living half or more on what the ground grows (insects, roots,
plants) now does (the mod's `Species::hungry_in_the_wild`); the cats' insects and the dogs' seeds are
top-up foods, now eaten only below their diet share (91a328628); the script spawns fish in the water.
A `hit 2` kills a 1-2 health animal (a mouse, a kryptobaatar). Missing clips: look, groom on
vole_sit; scratch, groom on lizard_bask; graze on chihuahua_adult and cat; groom, look, scratch,
stare, alarm on cat_sit; groom on cat_lie; idle clips on the fish (flinch, sniff, groom, look, play,
scratch on lungfish, pacu, lycoptera, brown_trout, fish_flop): fish should get no idle life.

Run 2 (b7ef3aaf7, `r2_small_m-171615`, 327 s). Hares, mountain hares, mice and voles drank to full; hares,
mice, voles, kryptobaatars and priscagamas ate; the insect eaters now get hungry (mice and voles 22-27 % at
the end; 946ea9fee holds). No "runs from" after the walk-in on land (not tested: the hits killed the
young hare and kryptobaatar hit, 1 health each). A wild cat hunted, killed and ate a chihuahua (2.5 kg, in its
prey sizes; a question for Planner: should hunters leave the dogs alone?). Fish: 8 of 9 alive at y 68-69,
swimming; the ninth lycoptera was spawned outside the pool and suffocated (script, fixed). A hit trout swam
on: fish now dart off from what hurts them ("fish N hurt by M: darts off"), and darting from a walker is
logged. Fish don't eat or drink and don't get hungry, by design. Vole 12's "health 0/0" is a young vole's
0.4 rounded.

| Species | eat | drink | wander | flee | shot |
|---|---|---|---|---|---|
| hare | pass (tall grass) | pass | pass | not tested | r2 1 |
| mountain_hare | not tested (no eat line) | pass | pass | not tested | r2 1 |
| wood_mouse | pass (grass), gets hungry | pass | pass | not tested | r2 1 |
| field_vole | pass (one), gets hungry | pass | pass | not tested | r2 1 |
| kryptobaatar | pass | pass | pass | not tested | r2 2 |
| priscagama | pass | pass | pass | not tested | r2 2 |
| cat | pass (killed a chihuahua) | pass | pass | pass ("runs from" when hit) | r2 3 |
| chihuahua | pass | pass | pass | pass (ran from the cat) | r2 3 |
| fish (4 kinds) | n/a | n/a | pass (swim) | fail: no dart when hit (fixed, to re-run) | r2 4 (dark) |

Grid r2: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r2_small_m-171615/r2_small_grid.jpg`.

Run 3 (46f12cbd7, `r11_small_m`, 327 s, script pass). All nine fish alive and in the pond to the end. When the
player walks in, all nine log "darts off from a walker" within 0.8 s, and the hit trout logs "fish 81 hurt by
1: darts off"; they move 1-3 blocks. The where rows read "doing -" for fish: they swim by `fish.rs`, outside
the needs band, so this is expected. Land: hares, mice and voles eat (tall grass on the strip, turf) and drink;
a kryptobaatar was killed by a hunter on the pad (core:attack). Nothing new in the warnings.

| Species | eat | drink | wander | flee | shot |
|---|---|---|---|---|---|
| hare, mountain_hare | pass (tall grass) | pass | pass | not tested on land (no walk-in "runs from") | r11 1 |
| wood_mouse, field_vole | pass | pass | pass | not tested | r11 1 |
| kryptobaatar | pass | pass | pass | prey: killed by a hunter | r11 2 |
| priscagama | pass (run 2) | pass (run 2) | pass | not tested | r11 2 |
| cat, chihuahua | pass | pass | pass | pass (run 2) | r11 3 |
| brown_trout, pacu, lungfish, lycoptera | n/a (by design) | n/a | pass (swim, all 9 alive) | pass (dart from a walker and from a hit) | r11 4 |

Grid r11: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r11_small_m-182115/r11_small_grid.jpg`.

## Big dinosaurs and bosses (cov_bosses.txt)

Run 2 (b7ef3aaf7, `r2_bosses_m-165534`, 408 s). The bosses: summon, aggro, death with loot and Drenkaun's
leash all seen; no blow lowered a boss's health (no "hit" line: the aimed blows never reached it; run 3 adds
`/boss hurt`, the same damage step), and Borkon's one toss (24.9) killed the player, so his second summon and
leash never came. The big plant eaters: the iguanodont and the gastonia "turned on it" but only stood, and
the gastonia then bolted from the player: e91a79201 only set its senses, and nothing made it charge, while
the shy check bolted from the player it had turned on. Fixed: `hunt::turn_on`, a charge at what hurt it
for `roles::TURN_SECS` (15 s), striking within `CHARGE` once a second ("charges 1 (it hurt it)"), letting go
when it is gone, a spectator or past 24 blocks ("lets 1 go"); no bolting from it meanwhile. The
cedarosaurus was young (103/192) and ran: right for a young one.

| Species | aggro / stands | leash | death | shot |
|---|---|---|---|---|
| iguanodont | fail: "turns on it", no charge (fixed, to re-run) | n/a | not tested | r2 1 |
| gastonia | fail: "turns on it", then bolted (fixed, to re-run) | n/a | not tested | r2 2 |
| cedarosaurus | pass (young: ran) | n/a | not tested | r2 3 |
| wyvern | flinched, breathed fire twice; no charge (a knock-out kind, left as is) | n/a | not tested | r2 4 (dark) |
| drenkaun | pass ("sweep hits 1 for 14.1") | pass ("back to its place" 60 s after) | pass ("died", loot); damage from blows not tested | r2 5 |
| borkon | pass ("toss hits 1 for 24.9": killed the player) | not tested (player dead) | pass ("died", loot); damage from blows not tested | r2 6 |

Grid r2: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r2_bosses_m-165534/r2_bosses_grid.jpg`.

Run 3 (8fe0410c3, `r7_bosses_m-175323`, 426 s): the grown gastonia "turns on it", "charges 1 (it hurt it)"
seven times a second apart and "lets 1 go" when the player turns spectator, with no bolt: the fix holds.
The iguanodont and the cedarosaurus were young (spawned in their season) and ran, right for the young; run 4
grows them first. `/boss hurt 40` went through each boss's damage step (Drenkaun took 7.5 and 4.1 of 40 after
its armour, Borkon 12.1 and 6.0), then death with loot; Drenkaun's leash again. Borkon's toss (24.9) killed
the player in one hit once more: the script gives no armour, so it is a balance question (should one
toss kill an unarmoured player at full health?), not a coverage failure.

| Species | aggro / stands (run 3) | death |
|---|---|---|
| gastonia | pass ("charges", "lets 1 go") | n/a |
| iguanodont | not tested (young: ran) | n/a |
| cedarosaurus | not tested (young: ran) | n/a |
| drenkaun | pass | pass (damage through its step, death, loot; leash) |
| borkon | pass | pass (damage, death, loot) |

Grid r3: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r7_bosses_m-175323/r7_bosses_grid.jpg`.

Run 4 (46f12cbd7, `r11_bosses_m`, 434 s, script pass). All three big plant eaters were grown this time,
and each turned on the player and charged it once a second until it let go: the iguanodont 9 times, the
gastonia 9 and the cedarosaurus 7, with no bolting. The wyvern breathed fire when hit, then flew out of its
pad's shot. Drenkaun and Borkon: `/boss hurt 40` went through the damage step (Drenkaun took 7.5 then 3.8,
Borkon 12.1 then 6.0: the second hit lands for half, as in run 3), then death with loot; Drenkaun's second
summon went back to its place with no one to fight. Borkon's toss (24.9) killed the unarmoured player again.

| Species | aggro / stands (run 4) | leash | death | shot |
|---|---|---|---|---|
| iguanodont | pass (turns on it, 9 charges, lets go) | n/a | n/a | r4 1 |
| gastonia | pass (9 charges, lets go) | n/a | n/a | r4 2 |
| cedarosaurus | pass (7 charges, lets go) | n/a | n/a | r4 3 |
| wyvern | breathes fire when hit (a knock-out kind, as designed) | n/a | n/a | r4 4 (flew out of shot) |
| drenkaun | pass | pass | pass (damage step, death, loot); a real blow not tested | r4 5 |
| borkon | pass (toss 24.9 kills an unarmoured player) | not tested (player dead) | pass; a real blow not tested | r4 6 |

Grid r4: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r11_bosses_m-185756/r11_bosses_grid.jpg`.

Borkon's toss (Planner, 11-25: make it survivable from full health, armour counting): fixed in 5b36e74c2.
One boss blow on a player now takes at most `animals_core::bosses::PLAYER_BLOW` (80 %) of the player's max
health before the player's armour, so the 24.9 toss takes an unarmoured player (20) to 16 and leaves 4; armour
takes its share after that, and armour's health raises the cap. The knock-back is unchanged.

| What | before | now | checked |
|---|---|---|---|
| borkon toss on an unarmoured player at full health | 24.9: dead (runs 3, 4) | 16.0: lives at 20 % | not tested in a run yet (expect "toss hits 1 for 16.0" and no "player 1 died") |

Run 1: cov_bosses, Mac, 403.9 s (`~/Coding/chiacraft-archive/tester-tools/runs/cov_bosses_m-160951.log`; grid
`~/Coding/chiacraft-archive/images/tester/2026-11-25/cov_bosses_m-161637/cov_bosses_grid.jpg`). The big plant
eaters ran from the hit (this run was before e91a79201, which has them turn and charge). The bosses'
deaths weren't reached, the script's fault: its blows went over the boss (looking up), and a boss
left alone despawns at its altar (the leash), so `/boss kill` after it found "no boss near". The
script now aims its blows, kills first, then summons again for the leash. The wyvern (a knock-out
kind) has no answer to a hit but its breath; left as is.

## Young (cov_young.txt)

Run 1 (1ff853abf, Tester, `cov_young_m-162151`): a script failure, not a game one. Young came only from
`/tk spawn`'s wild ones (a quarter of those met in their breeding season, which summer is not for deer,
reindeer or wolves), and a wild young one has no mother to keep by; `/animal hurry` reached 16 blocks, so the
young at a pad's edge stayed young; the where rows' 40 blocks missed the deer, capybara and protoceratops.
What it did show: hares 4 of 6 and an iguanodont 1 of 4 young, and those `hurry` reached grew up (hare 74
health 10/10 to 10/12, iguanodont 112 263/263 to 264/430).

Run 2 (b7ef3aaf7, `r2_young_m-164843`, 471 s): mating, births, keeping by the mother and growing up all
seen on every pad but the jaguars'. Unpaired grown ones on each pad (about a third) are expected: pairs meet.
The jaguars never mated, most likely (to confirm in the next run's love lines) because love scores 0.5, and a jaguar's sleepy rest at noon 0.8 (0.96 held), so a cat put
in love in its sleepy hours slept through its 30 s of love; a tame pair fed at noon would too. Fixed:
`needs::IN_LOVE` (0.97) for one in love now. Rows now give each one's sex, and `/animalprof love` logs who
fell in love and why the others didn't.

| Species | mated | young born | keeps by its mother (5 rows) | grows up | shot |
|---|---|---|---|---|---|
| white_tailed_deer | pass (2 pairs) | pass (5) | pass (1-3 blocks) | pass (12/12 to 12/26) | r2 1 |
| reindeer | pass (2 pairs) | pass (2) | pass (1-2) | pass (27/56 to 30/125) | r2 1 |
| capybara | pass (2 pairs) | pass (4) | pass (1-2) | pass (12/13 to 13/28) | r2 2 |
| grey_wolf | pass (1 pair) | pass (3) | pass (1-3) | pass (24/27 to 26/61) | r2 2 |
| jaguar | pass in run 3 (2 pairs at noon) | pass (3 cubs) | pass (1-3, one 8) | pass (none young after the hurry) | r3 3 |
| hare | pass (2 pairs) | pass (4) | pass (1-5) | pass (3/3 to 3/7) | r2 3 |
| protoceratops | pass (1 pair) | pass (5 eggs hatched) | pass (1-3) | pass (21/21 to 21/46) | r2 4 (dark) |
| iguanodont | pass (1 pair) | pass (1 egg hatched) | pass (2-3) | pass (353/545 to 379/1226) | r2 4 (dark) |

Grown ones keep their health and only their maximum rises (a young one grown by `hurry` heals as any
other). The dark pad at x 600 (every run): a hill over the 15 cleared blocks; pads now clear the sky to 140.
Grid r2: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r2_young_m-164843/r2_young_grid.jpg`.

Run 3 (8fe0410c3, `r7_young_m-174610`, 471 s): the jaguars, in love at noon, now mated (2 pairs, 3 cubs, kept
1 to 3 blocks from their mothers) and grew up; the fix holds. Every other pad mated, bore young that kept by
their mothers, and grew up, again. The love lines say why the others didn't: young ones and an elder
protoceratops. The four wolves (on the capybara pad) didn't mate this time (run 2: they did).
Grid r3: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r7_young_m-174610/r7_young_grid.jpg`.

Run 1 grid: `~/Coding/chiacraft-archive/images/tester/2026-11-25/cov_young_m-162151/cov_young_grid.jpg`.

## Taming and eggs (cov_taming.txt)

Run 1 (1ff853abf, Tester, `cov_taming_m-162426`): mostly a script failure. The testkit's `aim` + `core:use`
missed (the log's "kibble places no block": the click went to the ground), so the passive kinds were never
fed and the knock-out kinds' mouths stayed empty; the jaguar alone was fed and tamed. Every kind but the
psittacosaurus found the psittacosaurus' nest within 24 blocks (`wild_clutch` keeps nests apart), so only it
nested; its 8 eggs hatched, and the hurries after grew the hatchlings up at once (so no "(young)").
Game bug found: knocked-out raptors played groom and scratch clips while out cold (fixed: no idle clips
while out). The part 3 pad's panic (67 alarms) was its raptors and citipati hunting the hatchlings.

Grid r1: `~/Coding/chiacraft-archive/images/tester/2026-11-25/cov_taming_m-162426/cov_taming_grid.jpg`.

Run 2 (b7ef3aaf7, `r2_taming_m-164051`, 652 s): the feeding and bite lines all came. Passive: the cat was tamed
in 3 feeds as it asked (39, 78, 100 %, one about every 25 s); the wolf (18 %), capybara (69 %) and chihuahua
(47 %) were fed as they asked but then walked out of the 5 blocks a player must be within for them to ask
(the capybara ran from the wolf on their shared pad): a script limit, not a bug. Knock-out: raptor,
osmolskae and utahraptor out cold, given kibble, bit to 100 % and tamed; the jaguar reached 73 % in the
script's 12 bites and the wyvern 69 % in 35 (about 2 % a bite): too few bites, not a bug. No idle clips
from animals out cold (the fix holds). Laying: every kind nested on its own pad, the eggs hatched, and the
hatchlings kept 1 to 5 blocks from their mothers over 3 rows (one citipati 10 once, resting). Hatchlings read
"doing love" in the rows: a young one's love goal is following its mother (the rows now say "follow").
Shots 4 and 7 came out near black at midday (not looked into; the other pads are lit).


| Species | tamed | lays | hatches, keeps by mother | shot |
|---|---|---|---|---|
| grey_wolf | not tested (18 %, walked off) | n/a | n/a | r2 1 |
| cat | pass (3 feeds, "tamed at 100 %") | n/a | n/a | r2 2 |
| chihuahua | not tested (47 %, walked off) | n/a | n/a | r2 2 |
| capybara | not tested (69 %, fled the wolf) | n/a | n/a | r2 1 |
| raptor | pass (6 bites) | pass (3 eggs) | pass (3 born, 1-2 blocks) | r2 3, 7 |
| osmolskae | pass (6 bites) | n/a | n/a | r2 3 |
| utahraptor | pass (8 bites) | n/a | n/a | r2 3 |
| jaguar | not tested (73 % at the last bite; run 1 tamed it) | n/a | n/a | r2 3 |
| wyvern | not tested (69 % at the last bite) | n/a | n/a | r2 3 |
| protoceratops | n/a | pass (6 eggs) | pass (6 born, 1-5 blocks) | r2 4 (dark) |
| psittacosaurus | n/a | pass (6 eggs) | pass (6 born, 1-3 blocks) | r2 5 |
| citipati | n/a | pass (5 eggs) | pass (5 born, 1-3; one 10 once) | r2 6 |
| house_sparrow | n/a | pass (2 eggs) | pass (2 born, 1-5) | r2 8 |

Grid r2: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r2_taming_m-164051/r2_taming_grid.jpg`.

Run 3 (8fe0410c3, `r7_taming_m-173815`, 792 s). Penned beside the player, the passive kinds ask and are fed:
capybara tamed in 3 feeds, chihuahua in 5; the wolf reached 70 % in 4 feeds and the cat 44 % (asks 17 to
50 s apart: the window too short). Knocked out and fed: osmolskae, jaguar and wyvern (34 bites) tamed; the
raptor and utahraptor stopped at 88 %, one to three bites short (their levels vary the bites needed). Laying:
protoceratops, citipati, raptor and sparrow nested and hatched (young 1 block from their mothers); the one
psittacosaurus female didn't nest, with no line to say why (now logged: "doesn't nest: <why>"). Run 4: 210 s
of asking a pad, 30 bites for the raptors, four of each laying kind.

| Species | tamed | lays | hatches, keeps by mother | shot |
|---|---|---|---|---|
| grey_wolf | not tested (70 % in the time) | n/a | n/a | r3 1 |
| capybara | pass (3 feeds) | n/a | n/a | r3 2 |
| cat | not tested (44 % in the time; run 2: pass) | n/a | n/a | r3 3 |
| chihuahua | pass (5 feeds) | n/a | n/a | r3 3 |
| raptor | not tested (88 %, bites short; run 2: pass) | pass | pass (1 born) | r3 4, 8 |
| osmolskae | pass | n/a | n/a | r3 4 |
| utahraptor | not tested (88 %, bites short; run 2: pass) | n/a | n/a | r3 4 |
| jaguar | pass (8 bites) | n/a | n/a | r3 4 |
| wyvern | pass (34 bites, 26 bonus levels) | n/a | n/a | r3 4 |
| protoceratops | n/a | pass (6) | pass (6, 1 block) | r3 5 |
| psittacosaurus | n/a | not tested (no nest; run 2: pass) | not tested | r3 6 |
| citipati | n/a | pass (4) | pass (4) | r3 7 |
| house_sparrow | n/a | pass (2) | pass (2, 1 block) | r3 9 |

Grid r3: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r7_taming_m-173815/r7_taming_grid.jpg`.

Run 4 (46f12cbd7, `r11_taming_m`, 1035 s, script pass). Everything but the wolf tamed: capybara (3 asked
feeds), cat (6), chihuahua (3); knocked out and fed, raptor (7 bites), osmolskae (7), utahraptor (10), jaguar
(14), wyvern (40). The wolf took 8 asked feeds at 7 % each to 57 % and then its pad's time ran out: it needs
about 14 asks at 10-30 s apart, so the script now feeds it for 480 s (not run yet). Every laying kind nested and
hatched, the psittacosaurus too (5 eggs, 5 born). One female nested per pad: the others are within 24 blocks of
that nest, which is the rule (`wild_clutch`), so not a bug. Shots fine.

| Species | tamed | lays | hatches, keeps by mother | shot |
|---|---|---|---|---|
| grey_wolf | not tested (57 %, rising 7 % a feed; script lengthened) | n/a | n/a | r4 1 |
| capybara | pass (3 feeds) | n/a | n/a | r4 2 |
| cat | pass (6 feeds) | n/a | n/a | r4 3 |
| chihuahua | pass (3 feeds) | n/a | n/a | r4 3 |
| raptor | pass (7 bites) | pass (2) | pass (2) | r4 4, 8 |
| osmolskae | pass (7 bites) | n/a | n/a | r4 4 |
| utahraptor | pass (10 bites) | n/a | n/a | r4 4 |
| jaguar | pass (14 bites) | n/a | n/a | r4 4 |
| wyvern | pass (40 bites) | n/a | n/a | r4 4 |
| protoceratops | n/a | pass (5) | pass (5, 1-3 blocks) | r4 5 |
| psittacosaurus | n/a | pass (5) | pass (5) | r4 6 |
| citipati | n/a | pass (4) | pass (4) | r4 7 |
| house_sparrow | n/a | pass (3) | pass (3) | r4 9 |

Grid r4: `~/Coding/chiacraft-archive/images/tester/2026-11-25/r11_taming_m-184927/r11_taming_grid.jpg`.

## Sounds

Every species' calls come from a shared voice family (`animals_core::voices`, files in
`mods/animals/assets/sounds/voice/<family>/`), pitched for the species: none has calls of its own.

- **Silent:** the fish (brown_trout, pacu, lungfish, lycoptera), by design. **Drenkaun and Borkon**
  have no voice at all (no hurt, death or call; only their stone sound group).
- **Borrowed:** borkon_kid and drenkaun_calf use the reindeer's voice (`voices.rs`).
- **Shared families, no own sounds:** canine (dog, grey_wolf, coyote), fox (both), cat, big_cat
  (jaguar), deer (white-tailed, red, reindeer), rodent (capybara, wood_mouse, field_vole,
  kryptobaatar), hare (both; hurt and death only), songbird (sparrow, bunting, confuciusornis,
  gobipteryx), hawk (golden_eagle, savanna_hawk), owl (both), gull, corvid (raven), hiss
  (turkey_vulture, both caimans, priscagama), bellow (crocodyliform, psittacosaurus,
  protoceratops, gastonia, pinacosaurus, iguanodont, cedarosaurus), small_dino (raptor,
  osmolskae, utahraptor, citipati), dragon (wyvern). The most stretched: the Cretaceous's plant
  eaters all on one bellow (a 25 kg Psittacosaurus to a 10 t sauropod), the vulture on a hiss.
