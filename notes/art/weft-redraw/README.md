# The Weft sync's art, redrawn in ours (11-22)

The scripts that made the approved set, kept so a piece can be remade. They ran from `run/art/wc/`
(outside git), with the generated sprites (`*_px.png`) and the cut folders beside them; every
image is in `~/Coding/chiacraft-archive/images/art-weft-content/2026-11-22/`.

- `potions.py`: one generated flask (`potion_base_a_px.png`) re-shaded to each effect's colour from
  `mods/base/src/effects.rs`, and the empty `glass_bottle`. A shape per effect: generate more flasks
  and pick a base per potion here.
- `shears.py`: gold and diamond shears from the iron pair, blades mapped by brightness rank onto the
  item guide's ramps.
- `shield.py`: the shield model's 64 x 64 texture, painted in `models/item/shield.geo.json`'s layout.
- `fx_run.sh` and its request `.txt` files: the effect icon and UI icon sheets (one codex run each);
  `gridicons.py` cuts a sheet by grid cell (`SIZE`, `MARGIN`, `BIGGEST`, `SILHOUETTE`).
- `defringe.py`: magenta-hued edge pixels off a cut sprite.
- `pairs.py`, `page.py`: the before/after sheets and the review page.
- `install_map.py`: which redrawn file replaces which of the sync's added textures (`added.txt` from
  `git diff --name-only --diff-filter=A ae261d346 dev -- mods`), and what's left over.

Blocks: `notes/blocks/batches/weft_plants.tsv`, `notes/blocks/weft_tulips.py`,
`notes/blocks/weft_plants_post.py`, `notes/blocks/weft_machines.py`.
