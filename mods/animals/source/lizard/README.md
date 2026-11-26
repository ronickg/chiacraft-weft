# The lizard's source

One model for *Priscagama gobiensis*, an early agamid of the Djadokhta (the Cretaceous's lizard:
`notes/ECOLOGY.md`, "The food webs and the species to add"), made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research
alone, at the house density (64 texels a block). How it lives is `src/lizard.rs` and `src/wild.rs`.

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`lizard.py` lays out, paints and animates it into `out/` (not committed) and fits basking to the
ground; `build.py` checks flicker, joints, pokes and gait and exports `models/lizard.geo.json`,
`animations/lizard[_bask].animation.json` and `textures/entity/lizard/lizard.png` into
`../../assets`, and the coat layers into `../../genetics/layers/lizard`. At this size (a texel is a
model unit) the boxes are spaced by hand: every face at least 0.35 from its neighbors' parallel
faces, the tail's segments each tilted a little further down so none is ever parallel to the next.

## Research (2026-11-03)

- **The Djadokhta's lizards**: "rather abundant and diverse", 10-100 g: *Priscagama*, *Adamisaurus*,
  *Gobinatus* and others (`reports/ecology/Cretaceous food webs.md`); eaten by dromaeosaurs,
  troodontids and oviraptorids. *Priscagama* is one of the earliest agamid-like lizards (acrodont
  teeth, as agamas'), known from skulls; the living agamas give its build: a body about 8 cm, a
  tail longer, legs sprawling out to the sides.
- **Life**: insects first; agamas bask, bob their heads and do push-ups on their front legs to warn
  rivals; they lay clutches of small eggs buried in the ground. Breeding, colors and life span have
  no record here: a desert agama's estimated.

Decided: *Priscagama* as the lizard; 1.3 times life, as the mouse, so the face has room for an eye;
sandy grey with dark blotches, a pale stripe along each side of the back, banded tail, pale below;
animations idle (head bobs and push-ups, a look round, a flick of the tail), walk and run (sprawling,
the body curving side to side), bask (flat, the legs splayed).

Painted finer (Ronald, 2026-11-03): at the house density its head was 2.4 units across and 1.5
tall, the eye a black side of the head, so it is painted `house.FINE` (4) texels a unit, 256 a
block: an agama's eye (a dark pupil in a gold-brown ring, a catchlight), eyelids that blink (a
slow blink basking), faint nostrils and the lips' line. Boxes, export scale and size unchanged.

Behaviour clips (2026-11-05, `mods/animals/src/anim.rs`): stare, watch (up on straightened front legs,
the head turning in jerks), graze (a snap at an insect), drink, sleep (flat as basking, eyes shut; in
the basking file too), approach, alarm (two push-ups, the front feet planted), sniff, look, scratch
(a hind foot to the flank), stretch, play, groom (the snout wiped on the ground), flinch and death
(over onto its back). Not a hunter (no stalk or pounce), too small to stand its ground (no rear or
charge), and no nuzzle: an agama leaves its buried eggs. No jaw, dewlap or tongue: the gape and the
tongue's flick are the head's moves. `build.py` checks each foot within 0.3 of the ground in every
keyframe, the approach's gait, only held clips and the gait looping, and the death on its lowest point.
