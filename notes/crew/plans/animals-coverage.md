# Animal AI coverage (11-25)

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
| white_tailed_deer | pass | pass (lines; water was full) | fail: only the hit one ran | - (out of the rows' range) | grid 1 |
| red_deer | pass | - (water 30 %, ate instead) | - (not hit) | pass, 4-5 blocks | grid 1 |
| reindeer | pass (2 of 3) | - (ate instead) | - (not hit) | pass, 4 blocks (one row) | grid 2 |
| capybara | pass | pass for 1 of 3 (water rose 30 to 38) | fail: only the hit one ran | pass, 3-5 blocks | grid 2 |
| protoceratops | pass | pass | fail: only the hit one ran | pass, 1-2 blocks | grid 3 |
| psittacosaurus | pass | pass (water 30 to 99) | - (not hit) | pass, 1-8 blocks | grid 3 |
| gastonia | pass | pass | - (not hit) | pass, 3 blocks | grid 4 |
| pinacosaurus | pass | pass (30 to 77, 100) | fail: ran, never stood (1900 kg) | pass, 2-4 blocks | grid 4 |
| iguanodont | pass | pass | fail: ran, never stood (4 t) | pass, 7 blocks | grid 5 |
| cedarosaurus | pass | pass (30 to 100) | - (not hit) | pass, 3-4 blocks | grid 5 |
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

## Hunters other than wolves (cov_hunters.txt)

| Species | hunts | stalks / ambush | chase | kill | share | gives up | shot |
|---|---|---|---|---|---|---|---|
| jaguar | pass (deer, 56 kg) | pass (stalk, "breaks cover", 9 blocks) | pass (60 s, off the pad) | pass (70 kg, full) | n/a | pass ("too long" at 61 s, then a second hunt killed) | grid 1 |
| coyote | pass (37: a hare) | n/a | pass (2 s) | pass (3.8 kg) | fail: 36 never joined | - | grid 2 |
| red_fox | pass (4 hunts) | n/a | pass | pass (2 mice, a vole) | n/a | - | grid 3 |
| arctic_fox | pass | n/a | pass | pass (a vole) | n/a | - | grid 3 |
| raptor | pass | n/a | pass | pass (kryptobaatar, Priscagama) | n/a | pass ("gone": the other took it) | grid 4 |
| osmolskae | pass | n/a | pass | pass (kryptobaatar) | n/a | pass ("gone") | grid 4 |
| utahraptor | pass (a pack of 3, two "join") | - | pass (10 s) | pass (Psittacosaurus) | pass (8.3 kg each of 25) | - | grid 5 |
| crocodyliform | pass (took a spectacled caiman) | pass ("breaks cover", 6 blocks) | pass | pass (25 kg, full) | n/a | - | grid 6 (dark) |
| yacare_caiman | - (ran from the crocodyliform; no prey its size) | - | - | - | n/a | - | grid 6 |
| spectacled_caiman | - (killed by the crocodyliform at 1.5 s) | - | - | - | n/a | - | grid 6 |

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

## Birds and fliers (cov_birds.txt)

| Species | flies | lands | flocks | flees | hunts / scavenges | shot |
|---|---|---|---|---|---|---|
| house_sparrow | short hops (to 72) | pass | pass, 5 blocks | pass ("bolts from player") | n/a | grid 1 |
| snow_bunting | pass (to 80) | - (comfort all run) | fail: 18 blocks apart | pass | n/a | grid 1 |
| confuciusornis | pass (to 79) | pass | pass, 5 blocks | pass | n/a | grid 1 |
| gobipteryx | pass (to 78) | pass | 20 blocks at widest | - (no bolt) | n/a | grid 1 |
| golden_eagle | fail: on foot all run | n/a | n/a | - | pass on foot (a hare; gave up one "too long") | grid 2 |
| savanna_hawk | fail: on foot all run | n/a | n/a | - | pass on foot (a mouse) | grid 2 |
| great_horned_owl | one row at 77 | - | n/a | - | pass (a mouse) | grid 3 |
| snowy_owl | fail: on foot all run | - | n/a | - | pass on foot (a mouse) | grid 3 |
| turkey_vulture | pass (to 86) | pass | - | - | pass ("smells carrion" 19-20 blocks, ate venison) | grid 4 |
| raven | fail: y 69-70 only | - | n/a | - | pass by food (29 to 100 %, no line) | grid 4 |
| herring_gull | pass (to 86) | - | 15 blocks | - | pass by food (no line) | grid 4 |

cov_birds, Mac, 345.8 s (`~/Coding/chiacraft-archive/tester-tools/runs/cov_birds_m-155832.log`; grid
`~/Coding/chiacraft-archive/images/tester/2026-11-25/cov_birds_m-160420/cov_birds_grid.jpg`, no bird
visible in any shot). Fixed (birds.rs, after 91a328628): birds of prey hunted on foot; a hungry one
now soars to search, flies at prey it sees and stoops on it. The `hit 2` killed a sparrow (1 health):
flight from a hit isn't testable on songbirds that way. Missing clips: groom, play, alarm, sniff,
stare on small_bird_fly; look, groom, alarm on early_bird_fly; alarm, sniff on owl_fly; groom, look
on gull_fly and vulture_fly; play on hare_lie; alarm, sniff, scratch, look on vole_sit; scratch on
mouse_sit (idle clips asked for in the air: the fly looks should be left out of idle life).

## Small animals and fish (cov_small.txt)

| Species | eats | drinks | wanders | flees | shot |
|---|---|---|---|---|---|
| hare | pass (tall grass) | pass (water full) | - (ate all run) | pass ("bolts") | grid 1 |
| mountain_hare | pass | pass | - (comfort) | pass | grid 1 |
| wood_mouse | pass (grass, seeds) | pass for 1 of 3 | pass (1) | pass | grid 1 |
| field_vole | pass | pass (water 30 to 84-100) | pass (1) | pass (1 of 3) | grid 1 |
| kryptobaatar | fail: never hungry | fail: never thirsty | - (rested: nocturnal) | pass | grid 2 |
| priscagama | fail: never hungry | fail: never thirsty | pass | pass | grid 2 |
| cat | pass, but on insects in the grass | pass (1 of 2) | - | pass | grid 3 |
| dog (chihuahua) | pass, but on seeds in the tall grass | pass | pass | pass (ran from a hit) | grid 3 |
| brown_trout | - | n/a | pass (swims) | - (no dart seen) | grid 4 (dark) |
| pacu | - | n/a | pass | - | grid 4 |
| lungfish | - | n/a | pass | - | grid 4 |
| lycoptera | - | n/a | 1 of 3 (two spawned on the water's top suffocated: the script's) | - | grid 4 |

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

## Big dinosaurs and bosses (cov_bosses.txt)

| Species | aggro / stands | leash | death | shot |
|---|---|---|---|---|
| iguanodont | fail: ran when hit (fixed, to re-run) | n/a | - | grid 1 |
| gastonia | fail: ran when hit (fixed, to re-run) | n/a | - | grid 1 |
| cedarosaurus | fail: ran when hit (fixed, to re-run) | n/a | - | grid 1 |
| wyvern | partial: flinched, stayed, breathed frost twice; no charge | n/a | - | grid 1 (dark) |
| drenkaun | pass ("summoned", chase, "sweep hits 1 for 14.1") | pass ("back to its place" 60 s after) | - (script: see below) | grid 1 |
| borkon | pass ("toss hits 1 for 24.9": killed the player) | pass (62 s) | - (script) | grid 1 ("You Died") |

cov_bosses, Mac, 403.9 s (`~/Coding/chiacraft-archive/tester-tools/runs/cov_bosses_m-160951.log`; grid
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

Run 2 (to come): `/animalprof love` puts every grown wild one in love, so pairs mate and the young are born
to mothers on the pad; where rows give each young one's distance to its mother; `hurry` reaches 32 blocks.

| Species | young (run 1) | keeps by its mother | grows up | shot |
|---|---|---|---|---|
| white_tailed_deer | none (out of season) | not tested | not tested | |
| reindeer | none (out of season) | not tested | not tested | |
| capybara | not in the rows | not tested | not tested | |
| grey_wolf | none (out of season) | not tested | not tested | |
| jaguar | none seen | not tested | not tested | |
| hare | 4 of 6 | wild young, no mother (2 to 20 from grown ones) | pass (74) | |
| protoceratops | not in the rows | not tested | not tested | |
| iguanodont | 1 of 4 | wild young, no mother (3 to 7) | pass (112: 263 to 430 max health) | |

Grid: `~/Coding/chiacraft-archive/images/tester/2026-11-25/cov_young_m-162151/cov_young_grid.jpg` (the pads
from afar; the young can't be told from the grown ones in it). Missing clips: the lying and sitting looks
lack their idle clips (reindeer, deer, wolf, capybara, jaguar, hare, iguanodont, protoceratops: 30 "no clip"
lines, listed for the art side).

## Taming and eggs (cov_taming.txt)

Run 1 (1ff853abf, Tester, `cov_taming_m-162426`): mostly a script failure. The testkit's `aim` + `core:use`
missed (the log's "kibble places no block": the click went to the ground), so the passive kinds were never
fed and the knock-out kinds' mouths stayed empty; the jaguar alone was fed and tamed. Every kind but the
psittacosaurus found the psittacosaurus' nest within 24 blocks (`wild_clutch` keeps nests apart), so only it
nested; its 8 eggs hatched, and the hurries after grew the hatchlings up at once (so no "(young)").
Game bug found: knocked-out raptors played groom and scratch clips while out cold (fixed: no idle clips
while out). The part 3 pad's panic (67 alarms) was its raptors and citipati hunting the hatchlings.

Run 2 (to come): `/animalprof feed` feeds through the right click's code; the passive kinds are fed only when
they ask (a creative player standing among them); bites enough for each kind's feedings (wyvern 35); one
laying kind a pad; feeding and bite lines in the log.

| Species | tamed (run 1) | lays | hatches | shot |
|---|---|---|---|---|
| grey_wolf | not fed (aim) | n/a | n/a | |
| cat | not fed (aim) | n/a | n/a | |
| chihuahua | not fed (aim) | n/a | n/a | |
| capybara | not fed (aim) | n/a | n/a | |
| raptor | not fed (aim) | kept off by another nest | - | |
| osmolskae | not fed (aim) | n/a | n/a | |
| utahraptor | not fed (aim) | n/a | n/a | |
| jaguar | pass (tamed at 100 %, 7 bonus levels) | n/a | n/a | |
| wyvern | not fed (aim) | n/a | n/a | |
| protoceratops | n/a | kept off by another nest | - | |
| psittacosaurus | n/a | pass (8 eggs) | pass (8 born) | |
| citipati | n/a | kept off by another nest | - | |
| house_sparrow | n/a | kept off by another nest | - | |

Grid: `~/Coding/chiacraft-archive/images/tester/2026-11-25/cov_taming_m-162426/cov_taming_grid.jpg`.

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
