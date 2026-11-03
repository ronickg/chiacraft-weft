# Kryptobaatar's source

One model for *Kryptobaatar dashzevegi*, the Djadokhta's commonest mammal (a multituberculate:
`notes/ECOLOGY.md`, "The food webs and the species to add"), made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research
alone, on the wood mouse's build (`../mouse`), at the house density (64 texels a block). How it lives
is `src/kryptobaatar.rs` and `src/wild.rs`.

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`kryptobaatar.py` lays out, paints and animates it into `out/` (not committed) and fits sitting up
to the ground; `build.py` checks flicker, joints, pokes and gait and exports
`models/kryptobaatar.geo.json`, `animations/kryptobaatar[_sit].animation.json` and
`textures/entity/kryptobaatar/kryptobaatar.png` into `../../assets`, and the coat layers into
`../../genetics/layers/kryptobaatar`.

## Research (2026-11-03)

- **What it was**: a multituberculate, the longest-lived mammal group (not a rodent, though it
  looked and lived like one): gnawing incisors and blade-like premolars, a broad short skull (about
  2.5 cm), a body of about 10 cm, 30-50 g (est.); "the commonest mammal" of the Djadokhta
  (Kielan-Jaworowska et al. 2003); its sister species *K. mandahuensis* at Bayan Mandahu.
- **Life**: an omnivore leaning to animal food (insects first), seeds and plants; it probably
  burrowed, and may have hopped like a jerboa (Kielan-Jaworowska & Gambaryan 1994). Eaten by
  Velociraptor, troodontids and lizards (inference, `reports/ecology/Cretaceous food webs.md`).
- **Colors and breeding**: no record; a small desert mammal's estimated.

Decided: the mouse's build (a small mammal at 1.3 times life, the eyes bigger), with a blunter,
broader snout over pale incisors, small round ears and a thicker tail; grey-brown, pale below;
animations idle, walk, run, sit (up on its haunches).
