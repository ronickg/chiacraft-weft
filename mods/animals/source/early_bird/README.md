# The early bird's source

One model for the Cretaceous's early birds (`notes/ECOLOGY.md`, the Cretaceous's tier 2): the
Jehol's *Confuciusornis sanctus* and the Gobi's enantiornithine *Gobipteryx minuta*, on the birds'
build ([../bird/bird.py](../bird/bird.py): the raven's layout with knobs), made with the
`blocky-model` skill. `early_bird.py` is a page of numbers and color rules; how they live is
`src/early_bird.rs` and `src/birds.rs`.

## Rebuild

```
~/.venvs/blocky/bin/python ../bird/export.py early_bird
```

It writes `models/early_bird.geo.json` and `early_bird_fly.geo.json`, their animations and
`textures/entity/early_bird/early_bird.png` into `../../assets`, and the coat layers into
`../../genetics/layers/early_bird`, after the build's checks (flicker, joints, pokes, a walk that
goes forward).

## Research (2026-11-04)

- **Confuciusornis**: the commonest vertebrate of the Yixian (`reports/ecology/Cretaceous food
  webs.md`: 14 collections), "abundant", many killed together in flocks by eruptions; about the
  size of a crow, 0.2-1.5 kg (est.); the first bird with a toothless, horny beak; a big clawed
  finger at the bend of each wing; males with two long ribbon-like tail feathers. Fish remains in
  one specimen, seeds or plants proposed. Its color cells (melanosomes) suggest mottled grey-black
  and brown plumage with pale spots on the wings (Li et al. 2018, *Nature Communications*).
  Hunted by *Sinocalliopteryx* (two in its gut), *Microraptor*, pterosaurs.
- **Gobipteryx**: an enantiornithine ("opposite bird") of the Djadokhta, 0.2-0.5 kg (est.),
  insects and seeds (inference), nesting on the ground; eggs with embryos are known. Its colors
  have no record.

Decided: one model, the raven's layout 0.85 times its size (smaller made the beak and jaw
flicker on this build), a short deep beak, broad wings with the fingered tip, a long notched tail
for the streamers; painted at 64 a block (the head's side would be 2 texels at 32), a 2 by 2 dark
eye. Confuciusornis's colors as its color cells suggest; Gobipteryx sandier and smaller by its
genes. Neither fishes on this build (a bird here takes its food on the ground).
