# The wood mouse's source

The wood mouse (*Apodemus sylvaticus*), the living world's rodent of fields, woods, the desert's
scrub and the dunes (`notes/ECOLOGY.md`, "The food webs and the species to add"), made with the
`blocky-model` skill ([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md))
from research alone, at the house density (64 texels a block). The field vole, the north's rodent,
gets its own model. The recipe is [docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`mouse.py` lays out, paints and animates it into `out/` (not committed) and fits sitting up to the
ground; `build.py` checks flicker, joints, pokes and gait and exports `models/mouse.geo.json`,
`animations/mouse[_sit].animation.json` and `textures/entity/mouse/mouse.png` into `../../assets`,
and the coat layers into `../../genetics/layers/mouse`. `MOUSE_EYES=mouse_bead` paints the other
eye, a one-texel bead, to compare in the viewer.

## Research (2026-11-03)

- **The real animal**: about 9 cm head and body (8-11), the tail as long again, 23 g; warm sandy
  brown above, sharply grey-white below; big black eyes and big round ears; long hind feet. Almost
  wholly nocturnal, burrowing, a builder of nests; eats seeds (oak, beech, ash, lime, hawthorn,
  sycamore), snails and insects in late spring, berries, fungi and roots later. Breeds February to
  October, a 23-27 day gestation, independent at three weeks, mature at about two months. Eaten
  by foxes, owls and other birds of prey, weasels, snakes, cats and dogs
  ([Wikipedia](https://en.wikipedia.org/wiki/Wood_mouse)).
- **The food webs** (`reports/ecology/Overworld food webs.md`): the rodent is the cat's and the
  chihuahua's main food and stands in for the deer mouse, the kangaroo rat, cane mice and dune mice.

Decided: at 64 texels a block a real one is six texels long, with no room for a face, so the body
is about 1.3 times life and the head, eyes and ears bigger still; big black eyes with a catchlight
(Ronald's pick); animations idle (sniffing, the head turning, the ears twitching), walk
(scurrying), run (a bounding dash) and sit (up on its haunches, nibbling).

## How it's made

- **Widths across the body** are one table, `G`: at this size every part is a texel or two wide,
  and the hips, chest, haunches, shins, feet and front legs are spaced so no two faces that can
  meet in a pose share a plane (`boxcheck` at rest and in every frame).
- **Legs**: the front paw hinges at its back edge and curls its toes up as it lifts; the hind hock
  folds back with the foot kept flat (folding it forward made the lifted foot slide backwards).
- **A short neck** on the chest reaches deep into the head, so the head turns and nods on it
  without its corners cutting the chest.
- **Coat**: the colors as rules (`area`), recolored per animal from its genes in the game
  (`genetics/wood_mouse.json`, `core/src/coats.rs` `mouse`): grey-brown to sandy, paler in the
  desert, albinos very rarely; the tail dark above and pale below.

Painted finer (Ronald, 2026-11-03): at the house density its head was 2.5 units across, the eye
a black side of the head and every face one flat color, so it is painted `house.FINE` (4) texels
a unit, 256 a block, with a designed eye, eyelids that blink (idle and sit), a nose and a split lip.
The boxes, the export scale and its size in the game are unchanged.
