# Skills and the perk web

Written from `mods/base/vitals` (`cargo run -p vitals --bin web_sheet > mods/base/vitals/WEB_SHEET.md`); do not edit by hand. The design: `notes/PROGRESSION.md`, "Skills and perks (like Skyrim)"; the prices: `reports/Balance model.md` 5.2.

## Skills

A skill runs 0-100, 14.3 levels an age. Its levels cost effort minutes (EM): 25 % of each age's hours for that age's levels, so a main skill reaches 100 in 26.5 hours of doing it. Work below your skill teaches half as much every 7.1 levels below (one age below: a quarter). Each skill-up gives the player 8 XP times the new level (Skyrim's), toward the character level; each character level gives 1 perk point.

| Skill | Group | Path | Raised by |
|---|---|---|---|
| Melee | Fighting | shared | hitting creatures in melee |
| Archery and throwing | Fighting | shared | hitting with arrows and thrown things |
| Defence | Fighting | shared | taking hits |
| Smithing and tinkering | Crafting | tech | making tools, weapons, armour and their parts; smelting |
| Engineering | Life | tech | nothing yet (data for later) |
| Mining and prospecting | Life | tech | breaking stone and ore |
| Building and farming | Crafting | shared | placing blocks; sowing and harvesting crops |
| Alchemy and cooking | Crafting | shared | cooking |
| Runecraft and enchanting | Crafting | magic | nothing yet (data for later) |
| Destruction | Magic | magic | nothing yet (data for later) |
| Restoration | Magic | magic | nothing yet (data for later) |
| Conjuring and wards | Magic | magic | nothing yet (data for later) |
| Taming and riding | Beasts | beasts | taming and riding animals |
| Survival | Life | shared | chopping trees; making the other things one needs |
| Exploring and cartography | Life | shared | finding new biomes and dimensions |
| Trading | Life | shared | nothing yet (data for later) |

### What each age's levels cost

| Age | Skill levels | EM per level | EM from 0 | Hours from 0 |
|---|---|---|---|---|
| Stone Age | 0-14 | 6.3 | 90 | 1.5 |
| Bronze Age | 14-29 | 12.6 | 270 | 4.5 |
| Iron Age | 29-43 | 16.8 | 510 | 8.5 |
| Age of Steam | 43-57 | 18.9 | 780 | 13.0 |
| Age of Lightning | 57-71 | 21.0 | 1080 | 18.0 |
| Age of the Future | 71-86 | 18.9 | 1350 | 22.5 |
| Age of Stars | 86-100 | 16.8 | 1590 | 26.5 |

### What doing pays (EM, before tier against skill)

| Doing | EM | To the Stone Age's top level (90 EM), at full rate |
|---|---|---|
| a stone block mined (hardness 1.5) | 0.017 | 5400 |
| an iron ore mined (hardness 3) | 0.133 | 675 |
| a reference hit landed (Stone Age) | 0.050 | 1800 |
| a reference hit taken (Stone Age) | 0.033 | 2700 |
| a tool part of 3 inputs made | 0.600 | 150 |
| a food cooked | 0.333 | 270 |
| a block placed | 0.025 | 3600 |
| a ripe crop harvested | 0.100 | 900 |
| a large animal tamed | 12.000 | 8 |

At full rate means work at or above one's skill: Stone Age work at skill 7 teaches 100 %, at 14 51 %.

## The web

288 nodes (192 small, 80 notables, 16 keystones) in 16 regions around a free start, 336 links. A small node gives 2 % to its family, a notable 3-4 times that, a keystone 40 % to one family for as much off another. A family tops out at ×1.6 (one age), a keystone's cost at ×0.6. A rare item refunds 3 nodes, each one the rest don't hang from.

Skill level each ring needs (rings 9-12 are room for later ages at the rim):

| Ring | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Skill | 0 | 0 | 5 | 10 | 15 | 22 | 30 | 40 | 55 | 70 | 85 | 100 |

### Regions

| Region | Left lane | Right lane | Notables (ring 4, ring 7) | Keystone | Units | Bridge to the next |
|---|---|---|---|---|---|---|
| Melee | melee damage | stamina | heavy_blows (+3 melee damage); second_wind (+3 stamina); executioner (+4 melee damage); battle_rhythm (+2 stamina, +2 melee damage) | berserker (+20 melee damage, -20 defence) | 30 | weapon_master (+2 melee damage, +2 ranged damage) |
| Archery and throwing | ranged damage | move speed | steady_aim (+3 ranged damage); light_feet (+3 move speed); piercing_shot (+4 ranged damage); skirmisher (+2 move speed, +2 ranged damage) | glass_arrow (+20 ranged damage, -20 health) | 30 | shield_archer (+2 ranged damage, +2 defence) |
| Defence | defence | health | thick_hide (+3 defence); hardy (+3 health); shield_wall (+4 defence); unbroken (+4 health) | iron_wall (+20 defence, -20 move speed) | 30 | armourer (+2 defence, +2 craft grade) |
| Smithing and tinkering | craft grade | smelting speed | keen_eye (+3 craft grade); bellows (+3 smelting speed); masters_touch (+4 craft grade); hot_forge (+4 smelting speed) | forge_bound (+20 craft grade, -20 stamina) | 30 | machine_forged (+2 craft grade, +2 machine speed) |
| Engineering | machine speed | smelting speed | gearwright (+3 machine speed); steady_fire (+3 smelting speed); clockwork (+4 machine speed); pressure (+4 smelting speed) | overclock (+20 machine speed, -20 craft grade) | 30 | steam_drill (+2 machine speed, +2 mining speed) |
| Mining and prospecting | mining speed | stamina | stonecutter (+3 mining speed); strong_back (+3 stamina); vein_sense (+4 mining speed); tireless (+4 stamina) | tunnel_rat (+20 mining speed, -20 health) | 30 | quarry_master (+2 mining speed, +2 gathering speed) |
| Building and farming | harvest | gathering speed | green_thumb (+3 harvest); quick_hands (+3 gathering speed); bountiful (+4 harvest); builders_eye (+4 gathering speed) | homesteader (+20 harvest, -20 move speed) | 30 | kitchen_garden (+2 harvest, +2 cooking) |
| Alchemy and cooking | cooking | slower hunger | hearty_meals (+3 cooking); iron_stomach (+3 slower hunger); fine_cook (+4 cooking); lean_living (+4 slower hunger) | feaster (+20 cooking, -20 slower hunger) | 30 | infuser (+2 cooking, +2 mana cost) |
| Runecraft and enchanting | mana cost | craft grade | ley_sight (+3 mana cost, ley_sight); rune_carver (+3 craft grade); deep_runes (+4 mana cost); glyph_weaver (+4 craft grade) | rune_bound (+20 craft grade, -20 mana cost) | 30 | charged_runes (+2 mana cost, +2 spell damage) |
| Destruction | spell damage | mana cost | kindling (+3 spell damage); focus (+3 mana cost); storm_call (+4 spell damage); overflow (+4 mana cost) | ley_burn (+20 spell damage, -20 mana cost) | 30 | balance (+2 spell damage, +2 health) |
| Restoration | health | mana cost | mending (+3 health); calm_mind (+3 mana cost); renewal (+4 health); wellspring (+4 mana cost) | lifeblood (+20 mana cost, -20 health) | 30 | warding_light (+2 health, +2 defence) |
| Conjuring and wards | tame damage | defence | bound_spirit (+3 tame damage); ward (+3 defence); greater_summons (+4 tame damage); bastion (+4 defence) | warden (+20 defence, -20 spell damage) | 30 | rune_collars (+2 tame damage, +2 defence) |
| Taming and riding | taming speed | riding speed | gentle_hand (+3 taming speed); saddle_sense (+3 riding speed); pack_leader (+4 tame damage); swift_mount (+4 riding speed) | beastmaster (+20 tame damage, -20 melee damage) | 30 | tracker (+2 taming speed, +2 gathering speed) |
| Survival | gathering speed | health | woodsman (+3 gathering speed); weathered (+3 health); forager (+4 slower hunger); hard_to_kill (+4 health) | wild_born (+20 slower hunger, -20 cooking) | 30 | pathfinder (+2 slower hunger, +2 move speed) |
| Exploring and cartography | move speed | stamina | long_stride (+3 move speed); wayfarer (+3 stamina); pathless (+4 move speed); far_walker (+4 stamina) | wanderer (+20 move speed, -20 defence) | 30 | caravan (+2 move speed, +2 trade) |
| Trading | trade | riding speed | haggler (+3 trade); pack_train (+3 riding speed); merchant_prince (+4 trade); caravaneer (+4 riding speed) | silver_tongue (+20 trade, -20 melee damage) | 30 | sellsword (+2 trade, +2 melee damage) |

A region's budget is 30 units (one age in its speciality); keystones count as nothing, their power is their cost.

### Families

| Family | Units in the web | In the game |
|---|---|---|
| melee damage | 41 | yes |
| ranged damage | 41 | later |
| spell damage | 39 | later |
| defence | 74 | yes |
| health | 41 | yes |
| tame damage | 41 | later |
| taming speed | 13 | yes |
| craft grade | 70 | later |
| gathering speed | 25 | later |
| mining speed | 39 | yes |
| move speed | 48 | yes |
| mana cost | 61 | later |
| stamina | 31 | yes |
| slower hunger | 38 | yes |
| cooking | 39 | yes |
| harvest | 37 | yes |
| riding speed | 22 | later |
| trade | 39 | later |
| smelting speed | 22 | later |
| machine speed | 39 | later |

## Checks

Every node reachable from the start without passing a deeper gate, every price and region budget kept, no dominant build: no problems.

### Build dominance

Each job's build is hill-climbed with 60 points at skill 45 everywhere, then scored on every job (the weighted log of what its families multiply, shown as ×). A build may not win every job by over 15 %, and no node may be in over 90 % of the builds.

| Build (points spent) | fighter | archer | tamer | miner | smith | farmer and cook | mage | explorer |
|---|---|---|---|---|---|---|---|---|
| fighter (60) | ×2.62 | ×1.27 | ×1.02 | ×1.09 | ×1.04 | ×1.00 | ×1.22 | ×1.12 |
| archer (55) | ×1.43 | ×2.14 | ×1.00 | ×1.04 | ×1.00 | ×1.04 | ×1.22 | ×1.70 |
| tamer (35) | ×1.03 | ×1.00 | ×2.05 | ×1.02 | ×1.00 | ×1.00 | ×1.00 | ×1.01 |
| miner (51) | ×1.22 | ×1.01 | ×1.05 | ×2.05 | ×1.02 | ×1.07 | ×1.00 | ×1.41 |
| smith (43) | ×1.04 | ×1.00 | ×1.00 | ×1.04 | ×2.19 | ×1.00 | ×1.02 | ×1.00 |
| farmer and cook (28) | ×1.00 | ×1.00 | ×1.00 | ×1.00 | ×1.00 | ×1.95 | ×1.03 | ×1.10 |
| mage (45) | ×1.32 | ×1.15 | ×1.00 | ×1.00 | ×1.00 | ×1.00 | ×2.31 | ×1.00 |
| explorer (60) | ×1.27 | ×1.38 | ×1.04 | ×1.29 | ×1.00 | ×1.18 | ×1.00 | ×2.58 |

Most picked nodes:

| Node | Builds |
|---|---|
| melee family.melee_damage (ring 1) | 3 of 8 |
| melee family.stamina (ring 2) | 3 of 8 |
| melee family.stamina (ring 3) | 3 of 8 |
| melee melee.second_wind (ring 4) | 3 of 8 |
| melee family.stamina (ring 5) | 3 of 8 |
| melee family.stamina (ring 6) | 3 of 8 |
| melee melee.battle_rhythm (ring 7) | 3 of 8 |
| defence family.defence (ring 1) | 3 of 8 |
