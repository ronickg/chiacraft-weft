# The raven's source

The common raven (*Corvus corax*), the living world's carrion bird of the cold biomes, where there
are no vultures (`notes/ECOLOGY.md`, "The food webs and the species to add"), made with the
`blocky-model` skill ([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md))
from research alone, at the house density (64 texels a block). How it lives and flies is
`src/raven.rs`.

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`raven.py` lays out, paints and animates two models on one texture into `out/` (not committed):
standing with the wings folded along its sides (`raven`: idle, walk, run), and flying with them
open (`raven_fly`: flap, the look `raven.rs` puts on while it flies), so no wing has to fold
through the body. `build.py` checks flicker, joints, pokes and gait on both and exports
`models/raven[_fly].geo.json`, `animations/raven[_fly].animation.json` and
`textures/entity/raven/raven.png` into `../../assets`, and the coat layers into
`../../genetics/layers/raven`.

## Research (2026-11-03)

- **Size**: 54-67 cm from the bill to the tail's tip, a 1-1.5 m span, 0.7-2 kg (about 1.2); the
  biggest songbird. Black all over with a blue-purple gloss, a heavy bill, shaggy throat feathers,
  a wedge-shaped tail; dark brown eyes. White (leucistic) ravens turn up very rarely
  ([Wikipedia](https://en.wikipedia.org/wiki/Common_raven)).
- **Life**: pairs for life, holding a territory; the young gather in flocks and winter roosts. Lays
  3-7 eggs from late winter to April, the female sitting 18-21 days; the young fly at five or six
  weeks and breed from three or four. 10-15 years in the wild, up to 23.
- **Food**: carrion first (it follows wolves), eggs and nestlings, small rodents, insects, grain,
  berries and scraps; from the Arctic to the desert, mountains and coasts. Eagle-owls and goshawks
  take some.
- **The food webs** (`reports/ecology/Overworld food webs.md`): the forest, birch forest, taiga,
  snowy plains, mountains, desert, plains and beach; carrion, eggs, insects and scraps.

Decided (Ronald): the common raven, on the ground and in the air, no nests yet (its young come out
as fledglings); dark brown eyes with a catchlight. 40 texels long and 76 across the open wings, the
head a little bigger than life.
