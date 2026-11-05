# Citipati's source

One model for *Citipati osmolskae*, the oviraptorid of the Gobi's red dunes that sat on its nests
like a bird (`notes/ECOLOGY.md`, "The food webs and the species to add"; its rows in
`reports/ecology/Cretaceous food webs.md`: the dunes' omnivore, 90 kg, its brooding nests the
player sees), made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research, at
the house density (32 texels a block: the skull's side, which carries the eye, is 7 by 6 texels).
The recipe is [docs/creatures.md](../../../../docs/creatures.md). Not yet in the game: no entity
registers it yet (`src/`).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`citipati.py` lays out, paints and animates it into `out/` (not committed) and fits its stance,
pecking, run and brooding to the ground; `build.py` checks flicker, joints, pokes and gait and
exports `models/citipati.geo.json`, `animations/citipati[_lie].animation.json` and
`textures/entity/citipati/citipati.png` into `../../assets`, and the coat layers into
`../../genetics/layers/citipati`. Look at it with the skill's `render.py`
(`render.py out/spec.json out/texture.png shot.png --ppb 300 --views 3q,side,head`; its `3q` and
`head` views look up from below: a negative pitch looks down on it).

## Research (2026-11-04)

- **Size**: 2.5-2.9 m long and 75-110 kg, emu-sized; among the largest oviraptorosaurs before
  *Gigantoraptor* ([Wikipedia](https://en.wikipedia.org/wiki/Citipati); Norell et al. 2018). No
  hip height published: about 0.9 m here, an emu's, from its long legs (the shin longer than the
  thigh, as in all oviraptorids).
- **Head**: a short, deep, toothless beak, parrot-like, and an unusually short skull under a tall
  crest of the premaxilla and nasals, "superficially similar to that of a modern cassowary"
  (lower in *C. osmolskae*, taller and squarer in the Zamyn Khondt oviraptorid, IGM 100/42).
- **Feathers**: long feathers on the arms and the tail, the tail ending in a pygostyle that carried
  a fan of large tail feathers (rectrices); long pennaceous wing feathers (Hopp & Orsen 2004). A
  muscular, flexible tail, perhaps for display (Persons et al. 2014).
- **Brooding**: "Big Mama" (IGM 100/979) sits on an elongatoolithid clutch, the forelimbs folded
  and the hind legs crouched, breast, belly and feet on the eggs (Norell et al. 1995, *Nature*);
  "Big Auntie" (IGM 100/1004) has its arms spread symmetrically on both sides of the nest, over
  its rim, more spread than a bird's (Norell et al. 2018, *American Museum Novitates* 3899). Up to
  22 eggs a clutch, in rings of up to three layers, each about 18 cm long. *Oviraptor* was no egg
  thief: it sat on its own eggs.
- **Life**: common at Ukhaa Tolgod (Djadokhta); an omnivore or plant eater by inference (leaves,
  seeds, fruit, some small animals, insects, eggs: the food webs report's diet row); raptors take
  its young and eggs. It nests in spring, after the dunes' winter rains.
- **Colors**: no record.

Decided: the real size (hips about 58 units up in its stance, the crest's top about 105, 175
units along the spine from the beak to the fan's tip), the head a little bigger than life; a
dune bird's colors, our choice: sandy-buff body feathers with a rufous back and nape, a paler
throat, breast and belly, darker brown wings and tail fan with dark bars, a red crest over a dark
slate beak, grey-brown scaly feet with dark claws; the crest three stacked boxes, each narrower
and shorter, rising straight from the beak in front and stepping down behind; the neck two
segments (a bird's S); the wing feathers two cut-out panels a side (secondaries on the forearm,
primaries on the hand, ragged tips), hanging over the flanks when folded; the tail fan two
cut-out halves in a shallow V, so it shows from the side; an amber eye (no record; a ground
bird's), 3 by 2 texels with a catchlight forward, mirrored on the right; eyelids a little over the
eye all round. Animations: idle (pecking at the ground with quick jabs, the beak fitted to the
ground, then up to look about with a bird's head snaps, held and jumped, and two blinks, then
down again), walk (with a bird's head bob), run (leaning, the neck out, the wings a little out),
lie (brooding: sat on its folded legs, the breast on the ground, the arms spread out to both sides
and turned so their feathers fan down to the ground, as in the nesting fossils; fitted with
`ground.lowest` and `rig.fit`). Behaviour clips (`src/anim.rs`, in the standing file; sleep in the lying one too): stare, graze
(pecking, the beak's tip fitted to the ground with the legs straight, not the idle's over-tipped head), drink,
sniff, look and watch (a bird's head snaps), scratch and groom (preening a lifted wing), stretch (wings up,
then the wing-and-leg stretch), alarm (a hiss, wings flared), rear (a mother's display: tall, wings spread
wide, tail and fan raised, the feet planted), play (hops), nuzzle, flinch, sleep (brooding, the head turned
back over the shoulder), death (onto its side), approach and charge (gaits, grounded). No stalk or pounce.
