# The eagle's source

The day's big hunter of the open land (`notes/ECOLOGY.md`, "The food webs and the species to add",
tier 2: "eagle"): one model for the golden eagle and the savanna hawk, made with the `blocky-model`
skill on the birds' build ([../bird](../bird)), from research alone, at the house density.

## Rebuild

```
~/.venvs/blocky/bin/python ../bird/export.py eagle
```

`eagle.py` sets the eagle's knobs and colors; `../bird/export.py` paints, checks and exports
`models/eagle[_fly].geo.json`, their animations and `textures/entity/eagle/eagle.png` into
`../../assets`, and the coat layers into `../../genetics/layers/eagle`.

## Research (2026-11-03)

- **Golden eagle** (*Aquila chrysaetos*): 66-102 cm long, a 1.8-2.3 m span, 3-6.7 kg (females about
  a third heavier). Dark brown with a golden crown and nape, paler upperwing coverts, legs feathered
  to the toes, a grey hooked bill with a yellow cere, yellow feet, brown eyes under a heavy brow; long
  broad wings with fingered tips, held in a shallow V when it soars. 1-4 eggs (usually two) in March
  and April, 41-45 days' incubation, the young flying at ten or eleven weeks; adult plumage and
  breeding at four or five; about 20 years in the wild. Hares, rabbits, marmots and ground
  squirrels, grouse, foxes, young deer and sheep, small dogs and cats; carrion in winter. Mountains,
  prairie, desert canyons, taiga and the tundra's edge across the north.
- **Savanna hawk** (*Buteogallus meridionalis*): 46-61 cm, about 0.85 kg; rufous-cinnamon, finely
  barred below, the back greyer, long yellow legs, a black tail band. One egg, about 39 days. Rodents,
  lizards, snakes, frogs, crabs and insects, often following fires. The llanos' and the Pantanal's
  open grassland and marsh edges, where there are no golden eagles.
- **The food webs** (`reports/ecology/Overworld food webs.md`): meat 85%, carrion 10%, fish 5%; prey
  from 1% to 150% of its mass; diurnal; pairs.

Decided: the golden eagle's proportions (the raven's layout 1.35 times its size: about 85 cm long, a
2 m span: the wings 1.25 times the raven's at the arm and 1.2 at the hand, a third broader, fingered),
the savanna hawk the same model drawn about two thirds as big by its size genes and rufous by its
coat genes; a strong hooked bill (a hook box turned down over the jaw) with a yellow cere, an amber
eye with side lids that blink, golden hackles, feathered trousers, yellow toes, a square tail with a
dark band. Animations idle (looking about in holds and snaps), walk, hop, and a slow wingbeat held
open in a long glide.
