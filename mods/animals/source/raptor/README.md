# The velociraptor's source

A feathered Velociraptor the way the real animal looked (not the film's scaly giant), made the
way Hytale makes creatures with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)), from research
alone. The first two-legged animal: copy this folder for birds and other dinosaurs. The recipe is
[docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
../chihuahua/.venv/bin/python build.py      # any Python with ../chihuahua/requirements.txt
```

`raptor.py` lays out, paints and animates it into `out/` (not committed) and fits its stance and
resting pose to the ground; `build.py` checks flicker (rest and every keyframe) and exports
`models/raptor.geo.json`, `animations/raptor[_rest].animation.json` and
`textures/entity/raptor/raptor.png` into `../../assets` (scale 0.3: about half a block at the hip,
1.7 blocks long). For Blockbench and a GLB:
`python ../../../../.claude/skills/blocky-model/scripts/bb_build.py out`.

## Research (2026-11-01)

- **The real animal** (Velociraptor mongoliensis): about 0.5 m at the hip, 1.8-2 m long, 15-20 kg
  (turkey-sized; the film's raptors are far bigger); a long, low skull (up to 25 cm) with an
  upturned snout; the tail stiffened by bony rods, though it could bend sideways in an S
  ([Fossil Wiki](https://fossil.fandom.com/wiki/Velociraptor),
  [Prehistoric Wiki](https://prehistoric.fandom.com/wiki/Velociraptor)).
- **Feathers**: quill knobs on a forearm (2007) prove wing feathers; its relatives preserve long
  feathers on arms and tail and down on the body
  ([Dromaeosauridae](https://fossil.fandom.com/wiki/Dromaeosauridae),
  [Paraves](https://en.wikipedia.org/wiki/Paraves)).
- **Feet**: the second toe, with the sickle claw, was held off the ground; it walked on the third
  and fourth.
- **Hytale**: its raptor is the Cave Raptor, an aggressive, fast pack hunter that leaps
  ([The Spike](https://www.thespike.gg/hytale/beginner-guides/dinosaurs-in-hytale)).
  **Minecraft** has none; mods lean feathered
  ([Dominion Craft](https://modrinth.com/mod/dominion-craft)).

Decided: real proportions (hip about 29 units up in its crouch, body 23 long, tail 44, half the
length), the head 1.2x for Hytale's chunky look; feathered, colored like a hawk (rust back with
dark bars, sandy flanks, cream belly), a red crest; eyes on the sides of the head (a predator
bird's), yellow round a black pupil, in a dark stripe; animations idle (a bird's head: still,
then a snap to a new look), walk (head bobbing), run (leaning forward, tail straight out), rest
(legs folded under like a resting bird), screech.

## How it's made

- **Stance**: bird legs (thigh, shin, foot, toes) in a crouch, knees forward, ankles back, toes
  flat (the angles sum to 0), set in every pose (`stand()`); the drop that puts the toes on the
  ground comes from forward kinematics. The neck is raised and the head held level (`NECK`),
  so the head rides above the back on an S-curve.
- **Eyes on the sides**: stamped on the skull's east and west faces; their eyelids are hinged on
  the side of the head and swing down about Z (+90 on the left, -90 on the right), placed with
  the painted eye's rounding.
- **Feathers**: the crest, the wing feathers on each forearm and the tail fan are cut-out panels;
  the body feathers are painted (soft coat, dark bars on the back and tail).
- **Mouth**: an upturned snout (tilted), a jaw that hangs half a unit below the head and opens
  for the screech, teeth painted along both edges, a mouth box inside.
- **Flicker**: 0 at rest and in every keyframe; each tail segment is 0.6 a side thinner than the
  one before, leg parts all differ in width.
