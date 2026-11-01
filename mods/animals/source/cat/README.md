# The cat's source

An orange tabby made the way Hytale makes creatures (boxes, a few flat panels, a hand-painted
texture) with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)), from research
alone: there is no concept sheet, its colors are rules. The simplest example to copy for a new
animal; the recipe is [docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
../chihuahua/.venv/bin/python build.py      # any Python with ../chihuahua/requirements.txt
```

`cat.py` lays out, paints and animates the cat into `out/` (spec.json, texture.png; not
committed) and fits its resting poses; `build.py` runs it, checks no faces flicker and exports
`models/cat.geo.json`, `animations/cat[_sit|_lie].animation.json` and
`textures/entity/cat/cat.png` into `../../assets` (scale 0.2). For Blockbench (and a GLB for the
viewer): `python ../../../../.claude/skills/blocky-model/scripts/bb_build.py out`.
`CAT_EYES=cat_amber` (or `cat_blue`, `cat_round`) paints other eyes.

## Research (2026-11-01)

- **A real house cat**: about 23-25 cm at the shoulder, 46 cm head and body, 30 cm tail; the head
  about a quarter of the body; legs short for the body; the tail about two thirds of head and
  body. ([dimensions.com](https://www.dimensions.com/element/american-shorthair-cat),
  [a-z-animals](https://a-z-animals.com/animals/cat/))
- **Minecraft's cat** (Bedrock, [Mojang's
  sample](https://raw.githubusercontent.com/Mojang/bedrock-samples/main/resource_pack/models/entity/cat.geo.json)):
  a 5x4x5 head with a 3x2x2 muzzle and two 1x1x2 ears, a 4x6x16 body, the tail two 1x8x1
  segments, 2x2 legs (front 10 tall, back 6), 64x32 texture. The fewest boxes that read as a cat:
  small head, long body, thin long tail.
- **Hytale's cat** has unused cat and kitten models in its files; mods that use them recolor only
  the fur and keep the belly, paws and inner ears white
  ([Glymera Cats](https://www.curseforge.com/hytale/mods/glymera-cats),
  [Alec's Cats](https://hytale-hub3.pages.dev/mods/alecs-cats)).
- **Minecraft's red cat**: orange and white with green eyes; the tabby: brown stripes.
- **What reads as a cat**: triangular ears about a third of the head's height with pink insides
  and white tufts, a small muzzle with a pink nose and a mouth line, big eyes with a tall pupil,
  a tabby's M on the forehead and stripes round the legs and tail, a long tail carried up in a
  curve. It sits upright with the tail curled round on the ground, and lies as a loaf (paws
  tucked under).

Decided: real proportions for the body (back 22 units up, body 30 long, tail about 25), the head
1.3x (cute, like Hytale's creatures), chunky legs (4-6 units, not Minecraft's 2). Animations:
idle (tail sway, ear twitch, blink), walk, run, sit, lie (loaf), meow.

## How it's made

- **Colors as rules** (`area()`): coat, white (chest bib narrowing up, belly, muzzle, socks,
  paws), stripes (down the flanks, across the back, rings round legs and tail, the M, two cheek
  lines; their own crisp material so the painter doesn't merge them away), pink (inner ears,
  nose).
- **Ears**: four stacked boxes narrowing to a point, lit as one piece, one pink triangle inside,
  white tufts at the base, a cut-out panel through the middle for a smooth edge.
- **Eyes**: green round a tall pupil (`cat_green`), eyelids that blink.
- **Mouth**: the jaw opens to meow (shorter than the muzzle); a mouth box inside.
- **Tail**: four segments curving up; it swings about the axis across each segment. Sitting and
  lying it lies on the ground curling along the right side, fitted to target points
  (`fit_tail`) because hand-set angles compound into the rest curve.
- **Poses**: sit and lie fitted with forward kinematics (`fit_poses`) so the paws and body rest
  on the ground.
