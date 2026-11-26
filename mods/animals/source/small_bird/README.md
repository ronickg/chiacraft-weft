# The small bird's source

The songbird of every land biome (`notes/ECOLOGY.md`, "The food webs and the species to add", tier
2: "small bird"), the cat's, the owl's and the hawk's prey: one model for the house sparrow and the
snow bunting, made with the `blocky-model` skill on the birds' build ([../bird](../bird)), from
research alone, painted `house.FINE` (4 texels a unit) as small creatures are.

## Rebuild

```
~/.venvs/blocky/bin/python ../bird/export.py small_bird
```

`small_bird.py` sets the bird's layout, colors and knobs; `../bird/export.py` paints, checks and
exports `models/small_bird[_fly].geo.json`, their animations and
`textures/entity/small_bird/small_bird.png` into `../../assets`, and the coat layers into
`../../genetics/layers/small_bird`.

## Research (2026-11-03)

- **House sparrow** (*Passer domesticus*): 16 cm long, a 21-25 cm span, 24-40 g. A stout finch-like
  bird with a short thick conical bill. The male grey-crowned with a chestnut band from the eye
  round the nape, white cheeks, a black bib and eye line, brown back streaked black, a white wing
  bar, grey below; the female plain buff-brown. 4-5 eggs, 11-14 days' incubation, the young flying
  at about two weeks, two to four broods a summer; most live about three years. Seeds and grain,
  insects for the young. Wherever people farm and build, on every continent; in flocks.
- **Snow bunting** (*Plectrophenax nivalis*): 15-18 cm, 30-40 g; in winter buff and white, a rusty
  crown and cheeks, a streaked back, big white wing patches; breeds in the high Arctic in June (4-6
  eggs, 12-13 days, the young leaving at 12-14 days); seeds of grasses and sedges, insects in summer.
- **The food webs** (`reports/ecology/Overworld food webs.md`): seeds 45%, insects 40%, fruit 15%;
  diurnal; pairs in spring, flocks in winter; eaten by cats, hawks, owls, foxes and snakes.

Decided: the house sparrow's proportions (11 units from the bill to the tail's tip, 15 across the
open wings), the bunting the same model in its own colors from its genes; a round head, the short
thick bill, short rounded wings with a white bar, a short notched tail, short pink legs; dark eyes
with a catchlight and side eyelids that blink. Animations idle (looking about in holds and snaps),
hopping (walk and run), and the fast wingbeat of its flying model.

## How it's made

- **Its own layout** (`layout`): the raven's drawn this small would put neighboring faces closer
  than 0.3 units, which flickers; every pair of facing planes is at least 0.3 apart, each joined part
  a size apart from the next. The neck is a hidden link inside the breast and the head (it shows no
  corners, so the poke check leaves it out), the head leveled by the head alone (`neck_share` 0).
- **Coats from genes**: the male's bib is its points (black in a male sparrow, buff in a female,
  white in a bunting); the back's streaks are marks.
