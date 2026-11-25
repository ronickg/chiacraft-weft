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
| house_sparrow | - | - | - | - | n/a | |
| snow_bunting | - | - | - | - | n/a | |
| confuciusornis | - | - | - | - | n/a | |
| gobipteryx | - | - | - | - | n/a | |
| golden_eagle | - | - | n/a | - | - | |
| savanna_hawk | - | - | n/a | - | - | |
| great_horned_owl | - | - | n/a | - | - | |
| snowy_owl | - | - | n/a | - | - | |
| turkey_vulture | - | - | - | - | - | |
| raven | - | - | n/a | - | - | |
| herring_gull | - | - | - | - | - | |

## Small animals and fish (cov_small.txt)

| Species | eats | drinks | wanders | flees | shot |
|---|---|---|---|---|---|
| hare | - | - | - | - | |
| mountain_hare | - | - | - | - | |
| wood_mouse | - | - | - | - | |
| field_vole | - | - | - | - | |
| kryptobaatar | - | - | - | - | |
| priscagama | - | - | - | - | |
| cat | - | - | - | - | |
| dog (chihuahua) | - | - | - | - | |
| brown_trout | - | n/a | - | - | |
| pacu | - | n/a | - | - | |
| lungfish | - | n/a | - | - | |
| lycoptera | - | n/a | - | - | |

## Big dinosaurs and bosses (cov_bosses.txt)

| Species | aggro / stands | leash | death | shot |
|---|---|---|---|---|
| iguanodont | - | n/a | - | |
| gastonia | - | n/a | - | |
| cedarosaurus | - | n/a | - | |
| wyvern | - | n/a | - | |
| drenkaun | - | - | - | |
| borkon | - | - | - | |

## Young (cov_young.txt)

| Species | young spawned | keeps by a grown one | grows up | shot |
|---|---|---|---|---|
| white_tailed_deer | - | - | - | |
| reindeer | - | - | - | |
| capybara | - | - | - | |
| grey_wolf | - | - | - | |
| jaguar | - | - | - | |
| hare | - | - | - | |
| protoceratops | - | - | - | |
| iguanodont | - | - | - | |

## Taming and eggs (cov_taming.txt)

| Species | tamed | lays | hatches | shot |
|---|---|---|---|---|
| grey_wolf | - | n/a | n/a | |
| cat | - | n/a | n/a | |
| chihuahua | - | n/a | n/a | |
| capybara | - | n/a | n/a | |
| raptor | - | - | - | |
| osmolskae | - | n/a | n/a | |
| utahraptor | - | n/a | n/a | |
| jaguar | - | n/a | n/a | |
| wyvern | - | n/a | n/a | |
| protoceratops | n/a | - | - | |
| psittacosaurus | n/a | - | - | |
| citipati | n/a | - | - | |
| house_sparrow | n/a | - | - | |

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
