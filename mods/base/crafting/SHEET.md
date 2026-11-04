# Crafting numbers

Generated from `mods/base/crafting` (`cargo run -p crafting --bin sheet`). The design is `notes/CRAFTING.md`.

## Grades

| # | Grade | × main stat and uses | Modifier slots |
|---|---|---|---|
| 1 | `grade.base.crude` | 0.70 | 0 |
| 2 | `grade.base.rough` | 0.80 | 0 |
| 3 | `grade.base.plain` | 0.90 | 0 |
| 4 | `grade.base.sound` | 1.00 | 1 |
| 5 | `grade.base.good` | 1.06 | 1 |
| 6 | `grade.base.fine` | 1.12 | 1 |
| 7 | `grade.base.superior` | 1.20 | 2 |
| 8 | `grade.base.exquisite` | 1.28 | 2 |
| 9 | `grade.base.masterwork` | 1.38 | 3 |
| 10 | `grade.base.legendary` | 1.50 | 3 |

Points: material up to 40, work up to 40 (from memory up to 30, a machine 22), station up to 20, read in tens. Luck: -1 10%, +0 70%, +1 17%, +2 3%. Grade 10 needs 92 points.

## Materials

| Material | Roles | g/cm³ | Bulk | Harvest | Uses | Speed | Damage | As a haft | As a fitting | Trait |
|---|---|---|---|---|---|---|---|---|---|---|
| flint | working | 2.6 | 1.8 | 100 | 75 | 3 | 1 | - | - | keen |
| chert | working | 2.6 | 1.8 | 100 | 70 | 3 | 1 | - | - | common |
| obsidian | working | 2.4 | 1.8 | 100 | 40 | 3.2 | 1.5 | - | - | razor |
| bone | working, holding | 1.9 | 1.5 | 50 | 90 | 2.5 | 0.5 | uses +5%, speed +0% | - | tough |
| antler | working, holding | 1.9 | 1.5 | 50 | 90 | 2.5 | 0.5 | uses +8%, speed +0% | - | tough |
| stone (provisional) | working | 2.7 | 1.8 | 100 | 70 | 3 | 1 | - | - | common |
| diamond (provisional) | working | 3.5 | 1 | 300 | 1500 | 8 | 3 | - | - | - |
| copper | working, fitting | 8.96 | 1 | 150 | 150 | 5 | 1.5 | - | 120 uses | soft |
| bronze | working, fitting | 8.8 | 1 | 200 | 250 | 7 | 2 | - | 220 uses | recastable |
| wrought_iron | working, fitting | 7.7 | 1 | 220 | 275 | 7 | 2 | - | 260 uses | weldable |
| steel | working, fitting | 7.85 | 1 | 300 | 500 | 8 | 3 | - | 400 uses | tempered |
| gold | fitting | 19.3 | 1 | 0 | 30 | 6 | 0 | - | 40 uses | - |
| oak | working, holding | 0.75 | 1.6 | 0 | 60 | 2 | 0 | uses +10%, speed -3% | - | tough |
| mesquite | working, holding | 0.88 | 1.6 | 0 | 70 | 2 | 0 | uses +15%, speed -5% | - | tough |
| birch | working, holding | 0.65 | 1.6 | 0 | 55 | 2 | 0 | uses +0%, speed +5% | - | light |
| spruce | working, holding | 0.45 | 1.6 | 0 | 45 | 2 | 0 | uses -5%, speed +5% | - | light |
| plant_cordage | fitting | 1.3 | 1 | 0 | 0 | 0 | 0 | - | 30 uses | cheap |
| sinew | fitting | 1.3 | 1 | 0 | 0 | 0 | 0 | - | 60 uses | shrinks_tight |
| rawhide | fitting, holding | 1.2 | 1 | 0 | 0 | 0 | 0 | uses +0%, speed +0% | 80 uses | shrinks_tight |
| birch_tar | fitting | 1 | 1 | 0 | 0 | 0 | 0 | - | 70 uses | glued |
| leather | fitting, holding | 0.9 | 1 | 0 | 0 | 0 | 0 | uses +2%, speed +0% | 90 uses | - |

Provisional rows stand in for today's tools until Content plan's decisions 1 and 2 (the ladder replacing Weft's tiers, and what diamond becomes).

## Traits

| Trait | Effect | When |
|---|---|---|
| keen | Stat(1.15) | Soft |
| common | RepairCost(0.5) | Always |
| razor | Damage(1.25) | Always |
| brittle | Wear(2.0) | Always |
| tough | NoChips | Stone |
| soft | RepairCost(0.0) | Always |
| recastable | Recast | Always |
| weldable | Weld | Always |
| tempered | Wear(0.7) | Always |
| light | Speed(1.05) | Always |
| shrinks_tight | Wear(0.5) | Dry |
| glued | Wear(1.0) | Wet |
| cheap | Wear(2.0) | Wet |

## Kinds and shapes

- **axe** (Chop, damage 5): head (Working), haft (Holding), fitting (Fitting)
  - head `felling_axe_head`: 150 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - head `bearded_axe_head`: 170 cm³, stat ×0.9, speed ×0.95, damage ×1.2, reach +0
  - haft `haft`: 450 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - haft `long_haft`: 600 cm³, stat ×1, speed ×0.9, damage ×1.15, reach +0.3
  - fitting `lashing`: 5 cm³, stat ×1, speed ×1, damage ×1, reach +0
- **pickaxe** (Mine, damage 2): head (Working), haft (Holding), fitting (Fitting)
  - head `pick_head`: 180 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - haft `haft`: 450 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - fitting `lashing`: 5 cm³, stat ×1, speed ×1, damage ×1, reach +0
- **shovel** (Dig, damage 1.5): head (Working), haft (Holding), fitting (Fitting)
  - head `shovel_blade`: 200 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - haft `haft`: 450 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - fitting `lashing`: 5 cm³, stat ×1, speed ×1, damage ×1, reach +0
- **hoe** (Till, damage 1): head (Working), haft (Holding), fitting (Fitting)
  - head `hoe_blade`: 120 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - haft `haft`: 450 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - fitting `lashing`: 5 cm³, stat ×1, speed ×1, damage ×1, reach +0
- **knife** (Cut, damage 2.5): blade (Working), grip (Holding), fitting (Fitting)
  - blade `knife_blade`: 25 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - grip `grip`: 60 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - fitting `lashing`: 3 cm³, stat ×1, speed ×1, damage ×1, reach +0
- **spear** (Thrust, damage 4): point (Working), shaft (Holding), fitting (Fitting)
  - point `leaf_point`: 40 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - point `barbed_point`: 45 cm³, stat ×0.9, speed ×1, damage ×1.15, reach +0
  - shaft `shaft`: 700 cm³, stat ×1, speed ×1, damage ×1, reach +1
  - fitting `lashing`: 4 cm³, stat ×1, speed ×1, damage ×1, reach +0
- **sword** (Cut, damage 4): blade (Working), guard (Extra), grip (Holding), pommel (Extra)
  - blade `straight_blade`: 140 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - blade `long_blade`: 180 cm³, stat ×1, speed ×0.9, damage ×1.1, reach +0.5
  - blade `short_blade`: 100 cm³, stat ×1, speed ×1.15, damage ×0.9, reach -0.3
  - grip `grip`: 80 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - guard `bar_guard`: 20 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - pommel `round_pommel`: 15 cm³, stat ×1, speed ×1.02, damage ×1, reach +0

## Example axes (felling head, oak haft, Sound parts)

| Head | Fitting | Chops | Damage | kg | Lasts (uses) | Gives out first |
|---|---|---|---|---|---|---|
| flint | sinew | 2.91 | 6.0 | 1.05 | 60 | fitting |
| obsidian | birch_tar | 3.10 | 8.1 | 0.99 | 22 | head |
| copper | copper | 4.85 | 6.5 | 1.73 | 120 | fitting |
| bronze | bronze | 6.79 | 7.0 | 1.70 | 220 | fitting |
| wrought_iron | wrought_iron | 6.79 | 7.0 | 1.53 | 260 | fitting |
| steel | steel | 7.76 | 8.0 | 1.55 | 571 | fitting |
