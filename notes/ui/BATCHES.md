# UI redo: what each screen batch carries

The UI redo (branch claude/ui-redo-cfcd86) restyles the screens in batches, each approved by
Ronald. Items other sessions handed over, to do in the batch named:

## HUD
- Survival's hints (`mods/base/src/hints.rs`, `screen_text` at y 0.78) are drawn in warm yellow:
  bring them to the cream text rule (UI guide, "Yellow and gold only for state").
- The R key (engine `app.rs`, `recipe_keys`) only works over an item in an open screen. In the
  game with no screen open, R should open the held item's recipes, or the whole list when nothing
  is held (Survival, round 4).
- Food buffs as tiles with timers, the level badge, the temperature gauge (from the audit).

## Death screen (menus batch)
- `core:death` shows no cause: add it, and the tombstone's location (Survival, round 4).

## Not this job (on Ronald's sheet, later features)
Character creation, compass and minimap, map, journal, skills, build menu, whistle radial,
creature quick HUD, taming screen, creature management.
