# The gull's source

The coast's hunter and its eater of the dead (`notes/ECOLOGY.md`, "The food webs and the species to
add", tier 2: "gull"): the herring gull, made with the `blocky-model` skill on the birds' build
([../bird](../bird)), from research alone, at the house density.

## Rebuild

```
~/.venvs/blocky/bin/python ../bird/export.py gull
```

`gull.py` sets the gull's knobs and colors; `../bird/export.py` paints, checks and exports
`models/gull[_fly].geo.json`, their animations and `textures/entity/gull/gull.png` into
`../../assets`, and the coat layers into `../../genetics/layers/gull`.

## Research (2026-11-03)

- **Herring gull** (*Larus argentatus*): 55-67 cm long, a 1.25-1.55 m span, 0.7-1.5 kg (males a
  little bigger). The adult white with a pale grey back and wings, black wingtips with white
  "mirrors" and white tips, a long yellow bill with a red spot near the tip of the lower mandible
  (the chicks peck it to be fed), a pale yellow eye, pink legs and webbed feet; long narrow pointed
  wings whose black tips cross over the tail when folded. Young ones are mottled brown for three
  years. Nests in loud colonies in April and May: three eggs, 28-30 days, the young flying at 35-40
  days; breeding at four; about 12 years on average. Fish, crabs and mussels, the strandline's dead,
  eggs and young birds, insects, scraps and grain. The beaches and the sea's shore, inland along
  rivers to fields and dumps.
- **The food webs** (`reports/ecology/Overworld food webs.md`): fish 50%, carrion 25%, meat 10%,
  insects 15%; diurnal; colonies; its eggs are the fox's.

Decided: 60 cm and a 1.4 m span (the raven's layout 0.95 times its size, the wings long, narrow and
pointed), carried level on longer pink legs; the long yellow bill with the red spot, a pale eye
that blinks; animations idle (looking about in holds and snaps), walk, hop, and a wingbeat with
glides.
