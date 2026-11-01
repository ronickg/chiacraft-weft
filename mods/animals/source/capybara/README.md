# The capybara's source

The world's largest rodent, calm and always chewing, made the way Hytale makes creatures with the
`blocky-model` skill ([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)),
from research alone. The recipe is [docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
../chihuahua/.venv/bin/python build.py      # any Python with ../chihuahua/requirements.txt
```

`capybara.py` lays out, paints and animates it into `out/` (not committed) and fits its sit, lie
and grazing to the ground; `build.py` checks flicker, joints and gait and exports
`models/capybara.geo.json`, `animations/capybara[_sit|_lie].animation.json` and
`textures/entity/capybara/capybara.png` into `../../assets` (scale 0.34: about half a block at the
shoulder, 1.1 blocks long). For Blockbench and a GLB:
`python ../../../../.claude/skills/blocky-model/scripts/bb_build.py out`.

## Research (2026-11-01)

- **The real animal**: 50-62 cm at the shoulder, 100-130 cm long, about 60 kg; a barrel body on
  short legs, the hind legs a little longer; a vestigial tail (none to see); a long, blunt head
  with the eyes, small round ears and nostrils near the top, so it can watch and breathe with
  the rest underwater; slightly webbed feet, four toes in front and three behind, with short
  claws; coarse reddish-brown fur, lighter underneath
  ([Wikipedia](https://en.wikipedia.org/wiki/Capybara), [Encyclopedia.com](https://www.encyclopedia.com/environment/encyclopedias-almanacs-transcripts-and-maps/capybaras-hydrochaeridae),
  [SeaWorld](https://seaworld.org/animals/facts/mammals/capybara/)).
- **Behavior**: calm; grazes slowly, lifting its head now and then; chews constantly (its
  incisors grow all the time); rests lying down with the head held up, or sitting on riverbanks.
  Males have a dark, bare scent gland on top of the snout (the morrillo).
- **In games**: Minecraft has no capybara (mods add one); Hytale neither: a mod recolors the boar
  ([Capy Camp](https://www.curseforge.com/hytale/mods/capy-camp)).

Decided: real proportions (back 25.5 up, 54 long with the head), the head 1.15x and brick-shaped,
small eyes and round ears high on the sides, slit nostrils on top of the snout, the morrillo,
dark webbed feet with toes; animations idle (chewing, looking about, then grazing with the head
down at the grass), walk (slow), run, sit on its haunches, lie flat with the head up.

## How it's made

- **Coat**: reddish brown, a darker back, a lighter belly; the snout's dark patch; dark, bare
  feet with webbed toes painted on (4 in front, 3 behind) and claws.
- **Face**: eyes on the sides (with eyelids that swing down about z to blink), small and dark
  with a catchlight; nostrils as slits, not holes (two dark squares and a mouth line read as a
  second face); a split lip and a hint of the incisors.
- **Chewing**: the jaw (shorter than the muzzle) works in every pose except running.
- **Grazing**: the neck and head angles that bring the mouth to the grass are fitted with
  forward kinematics (`fit_poses`), as are the sit and the lie.
- **Gait**: short legs need a clear lift (knee 45 walking, 62 running) and a small swing, or
  the foot only shuffles and the gait check can't tell forward from back.
