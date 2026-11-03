# The field vole's source

The field vole (*Microtus agrestis*), the living world's rodent of the taiga, birch woods, tundra,
mountains and marshes (`notes/ECOLOGY.md`, "The food webs and the species to add"), made with the
`blocky-model` skill ([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md))
from research alone, at the house density (64 texels a block). Its body is the wood mouse's layout
(`../mouse`), with a vole's own head, ears, tail and coat. The recipe is
[docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`vole.py` lays out, paints and animates it into `out/` (not committed) and fits sitting up to the
ground; `build.py` checks flicker, joints, pokes and gait and exports `models/vole.geo.json`,
`animations/vole[_sit].animation.json` and `textures/entity/vole/vole.png` into `../../assets`, and
the coat layers into `../../genetics/layers/vole`. `VOLE_EYES=vole_glint` or `vole_black` paints
the other eyes, to compare in the viewer.

## Research (2026-11-03)

- **The real animal**: 8-13 cm head and body, the tail only 3-4 cm, 20-50 g; dark grey-brown,
  its hair longer and shaggier than the common vole's, the ears densely haired; a blunt round
  face, small eyes. Grasses, herbs, root tubers and moss, bark in winter, a few insect larvae. More
  active by day than the common vole; surface runs through tall grass. Breeds all year, even under
  snow, most in spring and summer: a three-week gestation, up to seven litters of four to six,
  mature soon after leaving the nest at three weeks; about two years at most. Numbers cycle.
  Eaten by owls, kestrels, weasels, stoats, foxes and snakes
  ([Wikipedia](https://en.wikipedia.org/wiki/Field_vole)).
- **The food webs** (`reports/ecology/Overworld food webs.md`): the north's rodent, standing in for
  the lemmings (the tundra's fox and owl live on them), the snow vole and the marsh rats.

Decided: the mouse's 1.3 times life; a short tail, small furred ears, a blunt muzzle, a dark coat;
small bead eyes (Ronald's pick); the mouse's animations (sniffing, scurrying, a dash, sitting up).
