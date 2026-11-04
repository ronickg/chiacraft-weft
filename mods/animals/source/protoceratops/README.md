# Protoceratops's source

One model for *Protoceratops andrewsi*, the Djadokhta's plant eater and the Velociraptors' prey
(`notes/ECOLOGY.md`, "The food webs and the species to add"), made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research
alone, at the house density (64 texels a block). The recipe is
[docs/creatures.md](../../../../docs/creatures.md); how it lives is `src/protoceratops.rs` and
`src/wild.rs`.

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`protoceratops.py` lays out, paints and animates it into `out/` (not committed) and fits lying
down and cropping ferns to the ground; `build.py` checks flicker, joints, pokes and gait and exports
`models/protoceratops.geo.json`, `animations/protoceratops[_lie].animation.json` and
`textures/entity/protoceratops/protoceratops.png` into `../../assets`, and the coat layers into
`../../genetics/layers/protoceratops`.

## Research (2026-11-03)

- **Size**: about 1.8 m long (up to 2.5) and 0.6 m at the hips, 62-104 kg as an adult; a big head
  with a parrot-like beak, a bump on the nose, flaring cheek bones and a frill over the neck; a deep
  tail with tall spines along its top; four legs, the hind ones longer
  ([Wikipedia](https://en.wikipedia.org/wiki/Protoceratops); `reports/ecology/Cretaceous food
  webs.md`).
- **Life**: "extremely common", the Djadokhta's dominant animal (Bayn Dzak has been called the
  *Protoceratops* fauna: Longrich 2010). Groups of mixed ages and small herds (Hone et al. 2014); a
  nest of 15 juveniles; buried, soft-shelled eggs, 12 or more a clutch (Norell et al. 2020), about 83
  days to hatch (Erickson et al. 2017). Low browse, chopped with a strong bite (Button & Zanno 2019).
- **Hunted by**: Velociraptor (the Fighting Dinosaurs, Tugrikin Shireh, 1971); tooth-marked jaws at
  Bayan Mandahu (Hone et al. 2010).
- **Colors**: no record. Here a dune animal's: sandy to ochre, pebbled with small scales, dark
  bands across the back and tail, pale below.

Decided: the real size, the head a little bigger than life; the frill a plate rising up and back
over the neck; amber-brown eyes on the sides; wild, in herds; animations idle (cropping ferns,
looking about), walk, run (a heavy trot), lie (on its belly, the legs folded under).
