# The fish's source

One model for the brown trout (*Salmo trutta*) and the pacu (*Piaractus mesopotamicus*), the middle
of the living world's water webs (`notes/ECOLOGY.md`, "The food webs and the species to add"), made
with the `blocky-model` skill ([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md))
from research alone, at the house density (64 texels a block). The recipe is
[docs/creatures.md](../../../../docs/creatures.md); how fish spawn and swim is `src/fish.rs`.

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`fish.py` lays out, paints and animates it into `out/` (not committed) and fits the stranded pose to
the ground; `build.py` checks flicker, joints and pokes and exports `models/fish.geo.json`,
`animations/fish[_flop].animation.json` and `textures/entity/fish/fish.png` into `../../assets`,
and the coat layers into `../../genetics/layers/fish`. `FISH_COAT=pacu` paints a pacu's colors, to
compare in the viewer (the game paints every fish's coat from its genes).

## Research (2026-11-03)

- **Brown trout**: a river fish of about 30-40 cm and 0.3-1 kg (lake fish far bigger); an
  olive-brown back, golden-bronze flanks, black spots and red ones ringed pale, a small adipose fin;
  insects (larvae and those fallen on the water), crustaceans, snails and small fish; cool, clear
  water (best at 4-19 degrees). It spawns in gravel in late autumn and winter.
- **Pacu**: the Pantanal's and the Paraná basin's deep-bodied pacu, typically 40-50 cm and a few
  kilograms, dark grey above, silver with an orange throat and belly; it eats fruit and seeds fallen
  into the floods (and spreads the forest's seeds), insects, snails and small fish; it spawns in the
  rivers as the floods begin.
- **The food webs** (`reports/ecology/Overworld food webs.md`): fish are the caiman's, the jaguar's,
  the gull's and the sea eagle's food, and the cat's by water.

Decided: the trout's shape at real size (about 35 cm), the pacu the same model drawn deeper and
bigger by its genes; cut-out fins with rays; animations idle (holding in the current, the pectoral
fins sculling), walk (swimming), run (darting) and flop (stranded, on its side, the tail slapping).

## How it's made

- **The fins** are double-sided cut-out panels through the body's middle (the tail's fan with a
  shallow fork, the dorsal sail, the adipose lobe, the anal and pelvic fans) and two paddle-shaped
  pectorals.
- **The pacu's shape** comes from its genes: the girth trait (`Gt`) scales the body's bones in
  height (about 1.3 times), the size gene the whole fish (about 1.35 times).
- **Stranded on its side**, the tail bends about its own x axis (the body's roll turns it upward),
  so it slaps along the ground instead of into it.

Painted finer (Ronald, 2026-11-03): its head is 2.8 units across and the eye filled most of its
side, so it is painted `house.FINE` (4) texels a unit, 256 a block: a round eye in proportion (a
gold ring, a black pupil, a catchlight), round spots, finer fin outlines. Boxes, export scale and
size unchanged.
