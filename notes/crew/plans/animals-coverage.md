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
| white_tailed_deer | - | - | - | - | |
| red_deer | - | - | - | - | |
| reindeer | - | - | - | - | |
| capybara | - | - | - | - | |
| protoceratops | - | - | - | - | |
| psittacosaurus | - | - | - | - | |
| gastonia | - | - | - | - | |
| pinacosaurus | - | - | - | - | |
| iguanodont | - | - | - | - | |
| cedarosaurus | - | - | - | - | |
| citipati | - | - | - | n/a | |

## Hunters other than wolves (cov_hunters.txt)

| Species | hunts | stalks / ambush | chase | kill | share | gives up | shot |
|---|---|---|---|---|---|---|---|
| jaguar | - | - | - | - | n/a | - | |
| coyote | - | - | - | - | - | - | |
| red_fox | - | - | - | - | n/a | - | |
| arctic_fox | - | - | - | - | n/a | - | |
| raptor | - | - | - | - | n/a | - | |
| osmolskae | - | - | - | - | n/a | - | |
| utahraptor | - | - | - | - | - | - | |
| crocodyliform | - | - | - | - | n/a | - | |
| yacare_caiman | - | - | - | - | n/a | - | |
| spectacled_caiman | - | - | - | - | n/a | - | |

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
