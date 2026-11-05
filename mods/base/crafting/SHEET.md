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

| Material | Roles | Works as | Level | g/cm³ | Bulk | Harvest | Uses | Speed | As a haft | As a fitting | Trait |
|---|---|---|---|---|---|---|---|---|---|---|---|
| flint | working | Edge | 5 | 2.6 | 1.8 | 100 | 75 | 3.0 | - | - | keen |
| chert | working | Edge | 5 | 2.6 | 1.8 | 100 | 70 | 3.0 | - | - | common |
| tektite | working | Edge | 6 | 2.4 | 1.8 | 110 | 81 | 3.1 | - | - | razor |
| obsidian | working | Edge | 6 | 2.4 | 1.8 | 110 | 81 | 3.1 | - | - | razor |
| bone | working, holding, extra | Bone | 3 | 1.9 | 1.5 | 80 | 82 | 2.8 | uses +5%, speed +0% | - | tough |
| antler | working, holding, extra | Bone | 3 | 1.9 | 1.5 | 80 | 82 | 2.8 | uses +8%, speed +0% | - | tough |
| stone (provisional) | working | Ground | 5 | 2.7 | 1.8 | 100 | 75 | 3.0 | - | - | common |
| diamond | working | Gem | 35 | 3.5 | 1 | 400 | 781 | 8.6 | - | - | - |
| copper | working, fitting, extra | Metal | 12 | 8.96 | 1 | 170 | 124 | 3.8 | - | 120 uses | soft |
| bronze | working, fitting, extra | Metal | 16 | 8.8 | 1 | 210 | 205 | 4.4 | - | 220 uses | recastable |
| wrought_iron | working, fitting, extra | Metal | 23 | 7.7 | 1 | 280 | 228 | 5.6 | - | 260 uses | weldable |
| meteoric_iron | working, fitting, extra | Metal | 18 | 7.9 | 1 | 230 | 189 | 4.7 | - | 220 uses | - |
| steel | working, fitting, extra | Metal | 27 | 7.85 | 1 | 320 | 356 | 6.5 | - | 400 uses | tempered |
| star_steel | working, fitting, extra | Metal | 29 | 7.9 | 1 | 340 | 409 | 6.9 | - | 500 uses | patterned |
| gold | fitting, extra | None | 13 | 19.3 | 1 | - | - | - | - | 40 uses | - |
| oak | working, holding, extra | Wood | 1 | 0.75 | 1.6 | 60 | 57 | 2.6 | uses +10%, speed -3% | - | tough |
| mesquite | working, holding, extra | Wood | 1 | 0.88 | 1.6 | 60 | 68 | 2.6 | uses +15%, speed -5% | - | tough |
| birch | working, holding, extra | Wood | 1 | 0.65 | 1.6 | 60 | 57 | 2.6 | uses +0%, speed +5% | - | light |
| yew (provisional) | working, holding, extra | Wood | 1 | 0.67 | 1.6 | 60 | 57 | 2.6 | uses +5%, speed +0% | - | - |
| elm (provisional) | working, holding, extra | Wood | 1 | 0.56 | 1.6 | 60 | 57 | 2.6 | uses +5%, speed +0% | - | - |
| osage_orange (provisional) | working, holding, extra | Wood | 1 | 0.86 | 1.6 | 60 | 63 | 2.6 | uses +8%, speed +0% | - | - |
| spruce | working, holding, extra | Wood | 1 | 0.45 | 1.6 | 60 | 45 | 2.6 | uses -5%, speed +5% | - | light |
| branch | holding | None | 0 | 0.6 | 1.6 | - | - | - | uses -10%, speed +0% | - | - |
| plant_cordage | fitting | None | 2 | 1.3 | 1 | - | - | - | - | 50 uses | cheap |
| sinew | fitting | None | 3 | 1.3 | 1 | - | - | - | - | 60 uses | shrinks_tight |
| rawhide | fitting, holding, working | Skin | 3 | 1.2 | 1 | 80 | 65 | 2.8 | uses +0%, speed +0% | 80 uses | shrinks_tight |
| birch_tar | fitting | None | 4 | 1 | 1 | - | - | - | - | 70 uses | glued |
| leather | fitting, holding, working, lining | Skin | 5 | 0.9 | 1 | 100 | 75 | 3.0 | uses +2%, speed +0% | 90 uses | - |
| linen | lining | None | 3 | 0.5 | 1 | - | - | - | - | - | - |
| wool | lining | None | 3 | 0.3 | 1 | - | - | - | - | - | - |
| fur | lining | None | 4 | 0.4 | 1 | - | - | - | - | - | - |

Harvest, uses and speed are a working part's at Sound, from its material's level on the curve (`vitals::gear`: uses 53 × g^1.5, speed 2.5 × g^0.75, harvest 100 at level 5 and 10 a level), with each row's hand tweak (×0.75-1.25, reasons in `materials.rs`). Provisional rows stand in until their materials grow in the world. Diamond is a tier above steel, and the ladder stays open at the top: a new tier is a row.

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
| patterned | RepairCost(0.5) | Always |
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
- **bow** (Shoot, damage 6): stave (Working), grip (Holding), string (Fitting)
  - stave `self_stave`: 400 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - stave `short_stave`: 280 cm³, stat ×1, speed ×1.15, damage ×0.85, reach +0
  - grip `grip`: 40 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - string `bowstring`: 2 cm³, stat ×1, speed ×1, damage ×1, reach +0
- **helmet** (Guard, damage 0): shell (Working), fastening (Fitting), lining (Lining)
  - shell `helmet_shell`: 300 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - fastening `straps`: 10 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - lining `lining`: 400 cm³, stat ×1, speed ×1, damage ×1, reach +0
- **chestplate** (Guard, damage 0): shell (Working), fastening (Fitting), lining (Lining)
  - shell `chest_shell`: 1200 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - fastening `straps`: 30 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - lining `lining`: 2000 cm³, stat ×1, speed ×1, damage ×1, reach +0
- **leggings** (Guard, damage 0): shell (Working), fastening (Fitting), lining (Lining)
  - shell `leg_shell`: 900 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - fastening `straps`: 20 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - lining `lining`: 1500 cm³, stat ×1, speed ×1, damage ×1, reach +0
- **boots** (Guard, damage 0): shell (Working), fastening (Fitting), lining (Lining)
  - shell `boot_shell`: 400 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - fastening `straps`: 10 cm³, stat ×1, speed ×1, damage ×1, reach +0
  - lining `lining`: 500 cm³, stat ×1, speed ×1, damage ×1, reach +0

## Example armour (a chestplate, leather straps, wool lining, Sound parts)

| Shell | Level | Armour | Health | kg | Lasts (blows) | Gives out first |
|---|---|---|---|---|---|---|
| leather | 5 | 12.6 | 2.1 | 1.7 | 75 | shell |
| bone | 3 | 11.5 | 1.2 | 4.0 | 82 | shell |
| copper | 12 | 17.6 | 6.1 | 11.4 | 90 | fastening |
| bronze | 16 | 21.2 | 9.0 | 11.2 | 90 | fastening |
| wrought_iron | 23 | 29.5 | 15.6 | 9.9 | 90 | fastening |
| steel | 27 | 35.6 | 20.5 | 10.0 | 90 | fastening |
| diamond | 35 | 51.8 | 33.4 | 4.8 | 90 | fastening |

## Example axes (felling head, oak haft, Sound parts)

| Head | Fitting | Chops | Damage | kg | Lasts (uses) | Gives out first |
|---|---|---|---|---|---|---|
| flint | sinew | 2.89 | 7.6 | 1.05 | 60 | fitting |
| obsidian | birch_tar | 3.00 | 9.9 | 0.99 | 44 | head |
| copper | copper | 3.70 | 10.5 | 1.73 | 120 | fitting |
| bronze | bronze | 4.26 | 12.7 | 1.70 | 220 | fitting |
| wrought_iron | wrought_iron | 5.46 | 17.7 | 1.53 | 251 | head |
| steel | steel | 6.28 | 21.3 | 1.55 | 559 | head |
