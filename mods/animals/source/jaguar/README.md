# The jaguar's source

The jaguar (*Panthera onca*), the top hunter of the swamp and the savanna, which in the Pantanal
eats mostly caimans, capybaras and fish (`notes/ECOLOGY.md`, "The food webs and the species to add",
tier 1): a full creature, made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research
alone, at the house density (64 texels a block), on the cat's build ([../cat](../cat)).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`jaguar.py` lays out, paints and animates it into `out/` (not committed) and fits sitting and lying
to the ground; `build.py` checks flicker, joints, pokes and gait and exports `models/jaguar.geo.json`,
`animations/jaguar[_sit|_lie].animation.json` and `textures/entity/jaguar/jaguar.png` into
`../../assets`, and the coat layers into `../../genetics/layers/jaguar`.

## Research (2026-11-03)

- **Jaguar** (*Panthera onca*): 57-81 cm at the shoulder, 1.12-1.85 m head and body, a 45-75 cm tail
  (the shortest of the big cats); usually 56-96 kg, up to 158, females 10-20% smaller, the biggest in
  the Pantanal. More robust than a leopard: stockier limbs, a squarer head, a strong bite (it kills
  by biting through the skull or a turtle's shell). Pale yellow to reddish gold, whitish below, black
  spots that become rosettes on the sides, bigger, darker and fewer than a leopard's, with thicker
  lines and often a spot in the middle. Black jaguars carry a dominant allele (a deletion in MC1R)
  ([Wikipedia](https://en.wikipedia.org/wiki/Jaguar)).
- **Life**: gestation 91-111 days; one or two cubs in the Pantanal, which stay in the den six months
  and with their mother nearly two years; females mature at about 2.5 years, males at 3-4; rarely
  past 11 in the wild, 22 kept. Prey of 1-130 kg, 45-85 kg liked best; capybaras and giant anteaters
  most selected; in the Pantanal mostly caimans, capybaras and fish, a kill every 1-16 days. Strong
  swimmers; dense forest, wetlands and swamps; home ranges of 15-25 km2 in the Pantanal, the densest
  anywhere.
- **The food webs** (`reports/ecology/Overworld food webs.md`): meat 85%, fish 15%; prey from 5% to
  300% of its mass; nocturnal and crepuscular (largely by day in the Pantanal); alone.

Decided: 70 cm at the shoulder, 1.5 m from the nose to the rump, a 60 cm tail: the cat's layout with
a jaguar's build (a longer back, a deeper, broader chest, thick legs and big paws, a short thick tail
carried low at one pitch, small round ears, a broad muzzle), the head 0.6 of the cat's (still a
little bigger than life, Hytale's way), warped into the house's units (the legs shortened, everything
2.25 times bigger). Golden with thick rosettes round darker centers, solid spots on the head, legs and
belly, the tail ringed toward a black tip, black ear backs with a pale spot; amber eyes with round
pupils (a big cat's), front lids a quarter texel bigger than the eye. A full creature, tamed by
knock-out as the raptors are (too dangerous to feed from the hand). Animations idle, walk, run, sit
and lie (fitted: a low sit for its short legs, the tail laid back on the ground), roar (its sawing
call), and the attacks swipe and bite.

## How it's made

- **From the cat's units** (`place`, `unplace`): the head and the bones under it scaled `HS` about
  the head's pivot, then the cat's legs squeezed (`D`) and everything scaled `K`. The color rules read
  positions back in the cat's units; the eyes and their lids are placed on the scaled face.
- **Rosettes** (`rosettes`): a jittered cell each, a broken thick ring round a darker center, often a
  spot inside; solid spots on the head, legs and belly. In the game the genes draw them (`rosettes`
  and `spotted` patterns, `mark_scale` 2.2 for its size).
- **The cat's build never checked pokes**; this one does: the hips reach deep into the chest, the
  neck is slimmer than the head, the tail's segments share one pitch.
