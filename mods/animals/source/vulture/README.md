# The vulture's source

Something for the dead in the warm biomes (`notes/ECOLOGY.md`, "The food webs and the species to
add", tier 2: "vulture"), the raven's counterpart: the turkey vulture, made with the `blocky-model`
skill on the birds' build ([../bird](../bird)), from research alone, painted `house.FINE` (its bald
head is only about 4 units across).

## Rebuild

```
~/.venvs/blocky/bin/python ../bird/export.py vulture
```

`vulture.py` sets the vulture's knobs and colors; `../bird/export.py` paints, checks and exports
`models/vulture[_fly].geo.json`, their animations and `textures/entity/vulture/vulture.png` into
`../../assets`, and the coat layers into `../../genetics/layers/vulture`.

## Research (2026-11-03)

- **Turkey vulture** (*Cathartes aura*): 62-81 cm long, a 1.6-1.8 m span, 0.8-2.4 kg (about 1.4).
  Blackish-brown, the wing coverts edged brown, the flight feathers silvery grey below (two-toned
  from beneath); a small bald red head, an ivory bill with a small hook, pinkish legs; a long tail.
  It soars with its wings in a shallow V, rocking, rarely flapping, and finds carcasses by smell,
  which few birds can. Roosts in groups. 1-3 eggs (usually two) on the ground or in a cave, 38-41
  days, the young flying at 60-80 days; breeding at about three; about 16 years in the wild. From
  southern Canada to Tierra del Fuego: the llanos, the Pantanal, the desert and the prairie; it
  leaves the north in winter.
- **The food webs** (`reports/ecology/Overworld food webs.md`): carrion 95%; diurnal; roosting groups;
  the dead's eater in the warm biomes (the raven does it in the cold).

Decided: 70 cm and a 1.7 m span (the raven's layout 1.1 times its size, the wings 1.3 times the
raven's at the arm, 1.25 at the hand, fingered), a small bald red head (0.75 the raven's) on a thin
neck reaching deep into it, the bill short, curved and ivory, a dark round eye that blinks, a long
round tail, pinkish legs; it soars, the wingbeat held open three quarters of the cycle.
