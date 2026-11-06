# The wyvern

A fantasy animal asked for by Ronald (2026-11-02): a dragon that flies, can be ridden, breathes
fire, rare and dangerous, huge, "kinda like Game of Thrones". Our own design in that style, built
with the blocky-model skill at the house density (64 texels a meter).

    python build.py      lays it out, paints, animates, checks and exports it into ../../assets

`wyvern.py` holds the model, the paint and the animations; `fit.json` the fitted poses (the
folded wing, the lying pose; `--refit` or `--refit-rest` to redo them after moving bones).

## What it is

- **Body**: two hind legs and bat wings that are its arms (a short thick upper arm, a long
  forearm, a wrist with a clawed thumb, four long fingers carrying the skin), as the show's
  dragons and real bats and pterosaurs. About 3.9 m at the shoulder standing, 16 m nose to tail,
  24 m across the wings: Drogon's span is about 1.5 times his length (an orthographic sheet of
  him: Lorenzo Amendola's), the wing broad and deep, its skin reaching back to the hips. A chest
  shaped by the flight muscles: a pectoral either side, a wing-root muscle on each shoulder, the
  ribcage narrowing to the hips (Drogon's sculpts: a deep, rounded, muscled chest).
- **Standing**: broad and strong, as Drogon stands in the show (season 7): the wrists planted
  wide, the forearms sloping out from the elbows at the chest's sides (an A from the front). Each
  folded wing stands beside it as a tall pleated sail, the fingers half closed and fanned up and
  back from the wrist (the sculpts). Its skin panels are rigid, so they can't follow fingers
  closing all the way: the fan only half closes, each panel centred in the gap left, the skin
  from the last finger to the body drawn in to slivers.
- **On the ground** it goes on four: the hind feet and the wrists of the folded wings, the wings
  standing beside it as sails. The show's animators (Pixomondo) studied gorillas for the walk
  (big shoulders and long arms, walking on the knuckles) and dropped a bat's hopping gait as odd
  on a dragon; the walk was sold by the arm and hip motion. So: the wings step one after the
  other (a primate's order: left leg, right wing, right leg, left wing), the shoulders roll onto
  each planted wing; the run is a gallop, one wing then the other. Bats themselves walk in a
  lateral sequence and vampire bats bound (Riskin et al. 2006).
- **Colors**: each of the show's dragons wears one main color with an accent (Drogon black with
  red, Rhaegal green with bronze, Viserion cream with gold). A coat here is the scales, the
  belly plates and the accent (wing skin, spike tips); its genes pick the scales' color
  (`genetics/wyvern.json`) and the accent comes with it.
- **Face**: amber reptile eyes with a slit pupil on the sides of the head under a heavy brow, a
  crown of swept-back horns, teeth along both jaws, nostrils as slits on top of the nose.
- **Behaviour clips** (11-05, `src/anim.rs`): stare (the neck lowered and stretched out, the head
  level and aimed), watch (the neck drawn up, the head turning slowly), drink (the snout down at the
  water, lapping), sleep (lying, the head laid on the left wing's forearm; also in the rest file) and
  approach (a slower, lower-stepping walk, the neck out low, looking about) loop; alarm (a roar, the
  fans of the folded wings flared open beside it, the wrists down), sniff, look, scratch (the cheek
  rubbed on the left thumb's claw), stretch (both wings lifted off the ground and spread wide and up,
  the skin let out, then folded and set down), play (a bow and a bounce of the forequarters, the tail
  swishing), groom (the snout run along the left forearm), nuzzle and flinch play once; death (rolled
  onto its right side, the neck, tail and wings sprawled on the ground) holds. A hunter that doesn't
  ambush: no graze, rear, charge, stalk or pounce. No lids, so the eyes stay open asleep and dead.
  Standing on its wrists, every frame sets them back where they stand (`plant`), so moving the chest
  never sinks them; the reaches are fitted once (`fit_clips`, kept in `fit.json` "clips", `--refit-clips`).
  The idle's breath sank the wrists 4.5 and the bite's lean 12: both keep them where the stand has
  them now. The fitted fold had the wrists 1.9 in the ground: it is lifted onto it (`ground_fold`, the
  hand's turn kept, the arm turned under 1 degree), and the build asserts the stand on all four. `build.py` asserts the loops,
  each foot within 0.3 of the ground in every standing clip's keyframes (the wrists off it only in the
  stretch and the hop), nothing below -0.3 (membranes by their cut outlines, scaled), sleep and death on
  their lowest point, no wing part in the body or the other wing, and no angle jumping between keys.

## Sources

- Animation Mentor, "Raising a Dragon" (Pixomondo's Florian Friedmann on the walk).
- Riskin, Parsons, Schutt, Carter, Hermanson (2006), terrestrial locomotion of bats, J. Exp. Biol.
  209: 1725.
- Inquirer and Mental Floss on the show's references (bats, birds, lizards, chickens, elephants).
