# The coyote's source

The coyote (*Canis latrans*), the prairie's and the desert's middle hunter (`notes/ECOLOGY.md`,
"The food webs and the species to add", tier 2), made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research
alone, at the house density (64 texels a block), on the fox's build ([../fox](../fox)). The recipe
is [docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`coyote.py` lays out, paints and animates it into `out/` (not committed) and fits lying down to the
ground; `build.py` checks flicker, joints, pokes and gait and exports `models/coyote.geo.json`,
`animations/coyote[_lie].animation.json` and `textures/entity/coyote/coyote.png` into
`../../assets`, and the coat layers into `../../genetics/layers/coyote`.

## Research (2026-11-03)

- **Size**: 58-66 cm at the shoulder, 1.0-1.35 m from the nose to the tail's tip with a tail of
  about 40 cm; males 8-20 kg, females 7-18; northern ones about 18 kg, southern Mexican ones 11.5
  ([Wikipedia](https://en.wikipedia.org/wiki/Coyote)).
- **Look**: slimmer and longer-legged than a fox, a narrow pointed muzzle, large erect pointed
  ears, a bushy tail carried low (down, not level as a wolf's), a scent gland marked dark at its
  base and a black tip. Grizzled light grey and fulvous with black hairs along the back, a pale
  face mask, throat and belly; the legs, the ears' backs and the muzzle tawny. Darker and greyer up
  high, more fulvous or whitish-grey in the desert; black coyotes (a dominant black, from dogs)
  are rare. Yellow eyes with round pupils.
- **Life**: families form in midwinter; 63 days' gestation; six pups on average, by the food and
  how many coyotes are about; the pair stays together and the male feeds the pups. Most breed in
  their first winter; few live past six in the wild. Runs at up to 65 km/h; territories of 0.4-62
  km2. Its howl changes quickly in tone and pitch, with yips.
- **Food**: rabbits and hares, rodents, deer fawns, birds, reptiles, insects, carrion, fruit, and
  cats and small dogs near people (`reports/ecology/Overworld food webs.md`: meat 60%, fruit, carrion
  15% each, insects 10%; prey from 0.2% to 30% of its mass alone).
- **Eaten by** wolves and pumas; people are its main danger.

Decided: 61 cm at the shoulder and 94 cm from the nose to the rump, the tail 41 cm, carried low: the
fox's layout warped (the legs a fifth longer, everything 1.3 times bigger); yellow eyes with round
pupils; a wild hunter, not tameable; animations idle (ears turning to sounds; then the head thrown
back in a howl, the jaw opening on its yips), walk (trotting), run (galloping, the tail streaming),
lie (the head on its paws) and its attacks, pounce (the mousing leap) and bite, all from the fox.

## How it's made

- **Warped from the fox** (`warp`, `unwarp` in `coyote.py`): every box and pivot is laid out in the
  fox's units; below the chest (the legs) the space is stretched, above it lifted, then all of it
  scaled by `K`. The color rules read positions back in the fox's space; the eyes and their lids
  are placed on the warped face, with the eye's rounding.
- **The tail** is three segments at one low pitch, each 0.4 a side thinner than the one before, the
  first reaching deep into the hips so it never pokes out as it sways.
- **Coats from genes**: the tawny legs, ears' backs and muzzle are its points; a grizzled ticking
  over the back; black coyotes dark all over.
