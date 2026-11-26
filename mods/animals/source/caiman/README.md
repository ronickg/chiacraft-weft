# The caiman's source

The water's hunter of the swamp and the savanna (`notes/ECOLOGY.md`, "The food webs and the species
to add", tier 2: "caiman"), the jaguar's main prey in the Pantanal and the capybara's danger: one
model for the yacare caiman and the spectacled caiman, made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research
alone, at the house density, on the lizard's build ([../lizard](../lizard)).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`caiman.py` lays out, paints and animates it into `out/` (not committed) and fits basking to the
ground; `build.py` checks flicker, joints, pokes and gait and exports `models/caiman.geo.json`,
`animations/caiman[_bask].animation.json` and `textures/entity/caiman/caiman.png` into
`../../assets`, and the coat layers into `../../genetics/layers/caiman`.

## Research (2026-11-03)

- **Yacare caiman** (*Caiman yacare*): males 2-2.5 m (up to 3), females about 1.4 m; 14-58 kg. Dark
  olive to near black, dark bands across the back and the tail, dark bars along the lower jaw,
  yellowish below; the back armored with rows of bony plates; a broad flat snout with teeth showing
  along the jaws; the eyes and nostrils high on the head, so it lies in the water with only them
  out. Olive-gold eyes with slit pupils. Nests in the rains: 20-35 eggs in a mound of rotting plants
  the female guards, 70-75 days; most eggs are eaten. Females lay at about 1.2 m, six to eight
  years old; 30-40 years. Fish, snails, crabs, water birds, rodents, the young of capybaras. Tens of
  thousands at the Pantanal's dry-season pools; the jaguar's main prey there.
- **Spectacled caiman** (*Caiman crocodilus*): 1.4-2.5 m, 7-40 kg; olive, paler and with fewer bands,
  a bony ridge between the eyes like the bridge of spectacles. The llanos' rivers and lagoons.
- **The food webs** (`reports/ecology/Overworld food webs.md`): fish 60%, meat 30%, insects (snails,
  crabs) 10%; a nocturnal hunter that basks by day; prey from 1% to 100% of its mass.
- Cold-blooded: it basks, gapes to cool, and goes into the water when hot.

Decided: 2.2 m from the snout to the tail's tip, half of it tail: the lizard's layout with a
crocodilian's proportions (a long flat snout on the skull, a raised nose, a jaw on its own bone
shorter than the snout with a pink mouth inside and teeth along both jaws, a broad flat body, a
tall tail in four segments each a size smaller, short sprawled legs), drawn 5.2 times bigger; the
spectacled caiman the same model drawn smaller and paler by its genes. Animations idle (still, the
jaws slowly gaping and closing, the head lifting so the lower jaw stays clear of the ground, a slow
tail sweep, blinks), walk (the high walk), run (a rush), bask (flat on the bank, the jaws agape, a
slow blink) and its attack, bite (the jaws wide, a sideways snap).

Behaviour clips (`mods/animals/src/anim.rs`, 11-05): held stare, watch, drink, sleep (also in the
basking file) and the gaits stalk (the belly a hand off the ground, the legs sprawled, a slow creep)
and approach loop; pounce (a lunge on straightened legs, the jaws flung open, the tail driving),
alarm (a hiss up on straight legs, the tail curled), sniff, look, scratch (a hind foot at the flank:
it can't reach its jaw), stretch (a gape, then legs and tail), play (a tail swish), groom and nuzzle
(head nudges) and flinch play once; death rolls it onto its back. `build.py` grounds the gaits, the
alarm, the pounce and each frame of the death, and asserts the feet, the stalk's belly and which
clips loop.

## How it's made

- **Laid out in the lizard's space**: every box and pivot in `caiman.py` is in the lizard's units
  and scaled by `K` before painting; the color rules read positions back (`p / K`). The skull is
  sized to whole texels once scaled, so the eyelids line up with the painted eye; the lids are a
  quarter texel bigger all round than the eye, so a closed lid hides it from any side.
- **The tail** is straight, each segment reaching deep into the one before and swung only a few
  degrees on it, so no segment pokes out of the last.
