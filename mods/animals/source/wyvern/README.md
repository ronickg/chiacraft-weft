# The wyvern

A fantasy animal asked for by Ronald (2026-11-02): a dragon that flies, can be ridden, breathes
fire, rare and dangerous, huge, "kinda like Game of Thrones". Our own design in that style, built
with the blocky-model skill at the house density (64 texels a meter).

    python build.py      lays it out, paints, animates, checks and exports it into ../../assets

`wyvern.py` holds the model, the paint and the animations; `fit.json` the fitted poses (the
folded wing, the lying pose; `--refit` or `--refit-rest` to redo them after moving bones).

## What it is

- **Body**: two hind legs and bat wings that are its arms (an upper arm, a forearm, a wrist with a
  clawed thumb, three long fingers carrying the skin), as the show's dragons and real bats and
  pterosaurs. About 3.9 m at the shoulder standing, 16 m nose to tail, 17 m across the wings.
- **On the ground** it goes on four: the hind feet and the wrists of the folded wings, the wings
  closed round it like a cape. The show's animators (Pixomondo) studied gorillas for the walk
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

## Sources

- Animation Mentor, "Raising a Dragon" (Pixomondo's Florian Friedmann on the walk).
- Riskin, Parsons, Schutt, Carter, Hermanson (2006), terrestrial locomotion of bats, J. Exp. Biol.
  209: 1725.
- Inquirer and Mental Floss on the show's references (bats, birds, lizards, chickens, elephants).
