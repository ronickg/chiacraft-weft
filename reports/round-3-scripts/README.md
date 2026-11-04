# Round 3's scripts

Read-only checks used for `reports/Research round 3.md`. Run them from the root of a checkout
(or set `CHIACRAFT_ROOT`), with `python3 -I`:

- `lang.py`: translation keys in `en` missing from `ru` (and `fr`), per mod and the engine.
- `keys.py`: translation keys used as literals in the mods' code, missing from `en`.
- `art.py`: textures that are missing, or byte-identical copies of another (needs `names.py`).
- `air.py`, `body.py`, `food.py`: the week-one air, body and food worked out per biome, mirroring
  `vitals::temperature::air`, `Body::target` and the stomach. Their constants are copied from the
  code at the time of round 3: check them against `mods/base/vitals` before trusting a rerun.
