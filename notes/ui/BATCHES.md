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

## Decided (Planner, 2026-11-04)
- R stays the recipe key; the animals session moves wyvern breath to V (F is `core:fly`).
- Numbers inside the HUD bars (Ronald's sheet) need an `at` placement in the mod UI ABI: folded
  into the next ABI change with Phase 6's API 1.2, so mods break once. Until then the numbers
  sit beside the bars, in cream.
- Food: Survival's 12-portion stomach stays (not the sheet's 3 buff tiles), toned.

## From Survival (relayed by Planner, 2026-11-04), once its branch is in dev
- cooking.rs: the campfire's machine screen, wrapped like the pot and the rack (`window`).
- death.rs (4148df51): base's own "death" panel replacing `core:death`: title, a bound line for
  the cause (`death.base.cause.*`), one for the tombstone's x/y/z, Respawn and Save and Quit.
  Restyle freely, keep the bound lines and the actions.
- death.rs's HUD element "grave" (above_hotbar, order 2): the tombstone's distance and direction
  and the grace time.
- hints.rs: the hint line at y 0.64, in cream.
