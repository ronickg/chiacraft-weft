# Rot: plan and numbers (Survival, 2026-11-04, for Planner, before building)

**Approved by Planner (2026-11-04), held for Ronald's next session; listed in BOARD.md's held list.**
1. The pace: the keeps stay as tuned; only where food sits changes the rate (the Q10 table).
2. Carcasses: Survival puts their numbers (stages, ages, yields) in `mods/base/vitals`. Animals
   wave two builds the carcass entity in `mods/animals` next session and reads them from there.

Base: dev 01180dd6. Builds on freshness (`vitals::food` keeps, `survival/food.rs` stamps), the air
at a block (`base_api::air_at`, rooms in `survival/shelter`), the compost bin and fertility
(`vitals::soil`), and the animals' carrion eating (`animals/src/forage.rs`).

## What's there now

- Every food has `keeps: (fresh, stale)` in game days (a porkchop 2 + 2, bread 5 + 5, an apple
  20 + 20, jerky 30 + 30). It ages by a day stamp (`MADE`), at the same rate everywhere: in a hand,
  a chest, a cave or the desert sun.
- Spoiled food stays its item with the "spoiled" mark. Eating it has a 50% chance of poisoning, and
  it composts for 2 (`soil::SPOILED`).
- Dropped items vanish after 5 minutes (engine `ITEM_LIFETIME`), so food on the ground never gets
  old enough to rot. Rule 4 says leave the engine alone, so I plan to keep this.
- A dead animal drops its meat at once. There is no body.

## One decision first: the pace

Planner's brief says "spoils on the growing pace". But the keeps already count real days as game
days (a game day is 20 minutes), and that's been played and tuned. On the growing pace, a real
day of raw meat at room temperature would be 0.065 game days, about 77 seconds. **I propose to
keep the keeps as they are** and add only what changes them: where the food is.

## The numbers (`vitals::rot`, new)

Spoilage speeds up with warmth by the Q10 rule. Microbes grow Q10 = 2 to 3 times faster per
10 °C (Labuza, *Shelf-life dating of foods*). This fits the real gap between room and
refrigerator: a steak keeps 3-5 days at 4 °C (USDA) and under a day at 20 °C, a ratio of about 5,
and 2.5^1.6 = 4.3.

| Number | Value | Why |
|---|---|---|
| `REF_C`: the air the current keeps are for | 15 °C | A cool larder. Temperate play stays about as it is. |
| `Q10` | 2.5 | The middle of 2-3. |
| `FASTEST` | ×4 (from 30 °C) | So a desert noon still leaves raw meat 20 minutes. |
| `FROZEN_C` / `FROZEN` | below −2 °C: ×0.05 | Frozen meat keeps 4-12 months (USDA); 80 game days for a porkchop, over a year. |
| Snow or ice next to a container (an ice house) | holds it at 0 °C: ×0.25 | Ice houses held 0-4 °C. |
| Underground (a cellar) | the biome's year-round mean (`air_at` already) | Root cellars hold 0-10 °C. |
| A room | the room's evened-out air (`shelter`), as players feel it | |
| Carried | the air the player feels (`Vitals::outside`) | |
| Salt | no extra rule: salted meat is its own food (56 + 56 days) | Already in `preserve`. |

Rate = 2.5^((T − 15)/10), clamped to ×4, and ×0.05 when frozen.

| Air | Rate | Porkchop (2 + 2 d) | Bread (5 + 5 d) | Apple (20 + 20 d) |
|---|---|---|---|---|
| 35 °C, desert noon | ×4 | 1 d (20 min) | 2.5 d | 10 d |
| 25 °C, summer | ×2.5 | 1.6 d | 4 d | 16 d |
| 15 °C, larder (as now) | ×1 | 4 d (80 min) | 10 d | 40 d |
| 10 °C, temperate cellar | ×0.63 | 6.3 d | 16 d | 63 d |
| 0 °C, snow or ice store | ×0.25 | 16 d | 40 d | 160 d |
| −5 °C, frozen | ×0.05 | 80 d | 200 d | 800 d |

The world's difficulty `keep` multiplier still applies on top.

## Rot

- **On spoiling,** a food becomes rot:
  - meat, fish and eggs become `base:rotten_flesh` (it exists and composts, and it's carrion for
    carrion eaters);
  - everything else (plants, bread, meals) becomes `base:rot` (new).
  - The count stays the same. The "spoiled X" items go away, so the inventory shows plainly what
    has rotted.
- **Rot feeds the soil:** it composts for 2 in the bin (as spoiled food does now), and spread on
  farmland it raises fertility by one, as dung does (`soil::FEEDS`).
- **Rotten flesh** keeps its compost value (and its poisoning when eaten).

## Carcasses (cross-mod: `mods/animals`)

A dead animal leaves its body (a carcass entity lying where it fell) instead of its drops.
- **Butchering** (use with an axe or knife) gives what the kill gives now: meat by `meat.rs`,
  hide, bones. The meat is stamped as old as the carcass.
- **It rots as its meat does:** the carcass ages as raw meat at its spot's air (the table above). It
  is fresh while the meat would be fresh, and bloated while stale (the meat comes out stale). Then
  it's remains: butchering gives rotten flesh and bones, no hide. After the same time again it's
  bones only, and then it's gone.
  - In real life (Payne 1965, a summer field): fresh 0-2 days, bloat 2-6, decay 5-11, dry remains
    25+. That's the same order as the meat's keeps.
- **Scavengers:** carrion eaters (fox, raven, wyvern, chihuahua, Utahraptor) go for a carcass
  within their food range, as they go for dropped carrion now. Each bite takes one meat item's
  worth from it.
- **The question:** Animals wave two owns hunting and the animal goals. Should I build the carcass
  in `mods/animals`, or give them the numbers (`vitals::rot` and a `carcass_stage(age, air)` helper)?

## How freshness counts (the build)

Slowing by part of a day needs finer stamps. I plan to:
- move `MADE` from whole days to game minutes, reading old day stamps as day × 1440;
- have a container's system (on chests, barrels, the pot and so on) push its foods' stamps forward
  each game hour by (1 − rate) × the hour, catching up the time it was unloaded, as the waiting
  crafts do;
- age a player's inventory the same way, by the air the player feels.
Stacks still merge to the older stamp.

## Checks (`vitals` tests, `balance.rs`)

1. The rate is 1 at 15 °C, rises with warmth, is at most 4, and is at most 0.05 frozen.
2. A temperate cellar keeps raw meat at least twice as long as a summer day outside.
3. A desert noon leaves raw meat fresh for at least 10 minutes.
4. Every food spoils into rot or rotten flesh, and both compost (`soil::COMPOSTS`).
5. A carcass is never fresher than meat dropped at the same time, at the same spot.
6. The fridge-to-room ratio is 3 to 6 (the USDA steak, from the Q10).
7. The sheet gets a "where food keeps" table: the one above.

`notes/SYSTEMS.md` gets a "Rot" row linking freshness, temperature, soil and the animals' diets;
`pieces.json` gets a part.
