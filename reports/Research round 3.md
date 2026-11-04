# Research round 3: `dev` as one game

Researcher, 2026-11-04, for Planner.

**What was read.** `dev` (`3e8c5a8`) has the wild plants, animals wave two, Phase 6 slices 1 and
2, CI, and the Phase 4 plan (a plan only, no code yet). Clothing, stews and preserving are not in
`dev` yet. They are on `claude/survival-clothing-cooking-f0c469`, which has `dev` merged in, so
this round reads that worktree as "`dev` plus survival": the game as it will be after the next
merge.

**How it was checked.** Three reviews read the code (duplication, balance, the first hour). The
balance review worked the air and the body out in Python, from the code's own formulas. I checked
the findings the top ten rests on myself, against the code, and they held:
- when the year starts;
- that there is no campfire;
- that wild animals' only loot is hides;
- the game-meat icons nothing uses;
- the drying rack's list;
- the stew's freshness;
- that clay is never generated;
- the 83 missing Russian keys.

No game was run. The balance review read the survival worktree while its sheet was being
regenerated, so it took the numbers from the source.

Owners follow `session-crew`:
- **Survival:** clothing, freshness and cooking.
- **Animals:** wave two and the animals mod.
- **Wild plants:** the Voxel dog handoff (wild plants, worldgen).
- **Phase 3:** leftovers (links, portals, the account and linked-world screens).
- **CI.**

## The top ten

| # | id | kind | owner | effort | finding | fix |
|---|---|---|---|---|---|---|
| 1 | B1 | balance | Survival | S–M | **A new player freezes in week one in 6 of 10 biomes.** The year starts 7 days after midwinter, the player arrives with no clothes, and there is no fire without 8 cobblestone. Plains: Freezing until 10:00 on day 0, 69 freezing hours in the week. Snowy plains: dead about 5 minutes after spawning. | Start players in plain clothes (about 1 clo); add a campfire; start worlds 7 days later (mid-spring) |
| 2 | B2 | balance | Animals, Survival | M | **Hunting wild animals gives no food.** The only loot is hides (`animals_core::hides`). Taiga, snowy plains, desert and beach then have no protein at all, since farm animals only spawn on grass. Icons for venison, hare, trout and pacu, raw and cooked, are drawn and unused. | Meat loot from real carcass yields, `FOODS` rows from USDA data, `raw_meat`/`cooked_meat` tags so cooking, drying, smoking and stews take them |
| 3 | D1 | bug | Survival | S | **Spoiled food comes back fresh.** The pot stamps a stew "made today, hot" whatever went in. The rack stamps its product fresh. Salting turns any raw meat, spoiled or not, into a 90-day food. | Refuse spoiled inputs, or carry the oldest ingredient's age forward |
| 4 | B3 | balance | Survival, Wild plants | S | **Clothes are out of reach in week one.** A linen set needs 38 string: about 114 fibre, about 900 tall grass. Hide or wool needs 11 leather or 11 fleeces, which is 11 cows or sheep. The snowy biomes have no fibre at all: no tall grass, and sedge and cottongrass drop nothing. | Fewer pieces of material for hide and wool, to real hide counts; sedge, cottongrass and cattail give fibre; 2 fibre a string |
| 5 | T1 | translation | Survival, Phase 3 | S–M | **A Russian player meets an English HUD at once.** Base `ru_ru` lacks 83 keys: every effect, the "feels like" line, seasons, shortcuts, all 18 stats, levels, and the freshness and nutrition tooltip lines. The engine's `ru` lacks 100 text keys (account, friends, links, join sheet, multiplayer), `fr` 86. | Fill base `ru` first (the HUD), then the engine's screens |
| 6 | B4 | balance | Wild plants, Survival | S | **Spring's wild food is wrong both ways.** Unripe bushes still give a handful out of season, which the rule forbids. Wild rice, cattail and over-wintered berries, which are real spring food, give nothing a player can eat. | Unripe bushes drop nothing; wild rice gets a `FOODS` row; cattail gives a root; lingonberry and crowberry generate ripe in winter and spring |
| 7 | U1 | unreachable | Wild plants, Animals, Portals | S each | **Items with no source in survival.** Clay is never generated, so bricks (the Building tab's icon), slime blocks and sticky pistons can't be made. Ice drops nothing, and it is the only blue dye. Arrows have no recipe, and tranquilizer darts need them. Portal blocks have no recipe. | Clay under shallow water; ice drops itself with a pickaxe; an arrow recipe; Ronald decides on portals |
| 8 | C1 | clash | Survival | S | **Leather is warm two ways, and they stack.** Leather armor gives 8 °C, keeps 35% when wet and stops no wind (`temperature.rs:163`). A hide coat from the same leather gives 9.6 °C, keeps 30% wet and stops 70% of the wind (`clothing.rs:47`). | Derive armor's warmth from the hide material, and decide whether they stack |
| 9 | Q1 | CI | CI | S | **CI skips the new tests.** `ci.yml` runs wallet, account, packages, engine, vitals and `animals_core`. It does not run `chiacraft-sandbox`, the packages mod, `base_api` or `base`. Since tests run only on CI, Phase 6's tests and survival's new ones run nowhere. | Add the four packages to the test steps |
| 10 | B5 | bug | Survival, Wild plants, Animals | S | **Wind chill cools things that have no skin.** Plant frost (`growth.rs:55`), egg warmth (`eggs.rs:380`) and drying (`cooking.rs:245`) read the felt temperature, so a windy −1 °C night kills plants hardy to −2 °C. Wind chill is the heat a body loses; an object doesn't fall below the air. | Read `.still` for objects |

---

## 1. Balance across systems

### Where the numbers put a new player

**The calendar.**
- A day is 1200 s and a season 14 days; midsummer is day 21 of a 56-day year
  (`vitals::calendar`), so midwinter is day 49.
- A world starts at day 0, 06:00, which is spring's first day but only **7 days after
  midwinter**. In a real year that is early March.
- The season's pull on day 0 is −0.71 of the biome's swing, easing to about −0.1 by day 7.

**The body.** Unclothed, the player is comfortable from 15 to 28 °C of air:

| state | air below |
|---|---|
| Chilly | 13.6 °C |
| Cold | 9.3 °C |
| Freezing | 3.6 °C |

- Freezing takes 0.6% of max health a second. With the arrival meal (bread and an apple, max
  health 54), that is about 2 minutes to cool, then 167 s to die on normal.
- On the hot side: Hot above 33 °C, Heatstroke above 38 °C.

**Heat.**
- Lava gives +20 °C beside it, a lit furnace +14, a torch +5. There is no campfire.
- A roof removes sun, rain and wind but never warms the air.
- Below y 57 with no sky, a cave holds the biome's year mean.

**Clothes.**
- A full set warms by: linen 6 °C, hide 9.6, wool 14.4, fur 30.
- Each set takes 11 pieces of material and 5 string.

**Food.**
- There is no hunger death. An empty stomach floors max health and stamina at 40%.
- Holding protein at half (max health 70) takes about 1.7 cooked steaks a game day.
- Holding energy at half takes about 1.6 loaves, or 63 handfuls of berries. Those won't fit: they
  digest in 5.5 minutes and are 84% water.

**Wild hunters** leave players alone for now (`hunt.rs`). The vanilla night monsters are the only
danger in week one.

### Each biome's first week

The temperatures are 03:00 still air / windy, and 15:00, on day 0 → day 6.

| biome | air | verdict | numbers |
|---|---|---|---|
| plains | −3.4/−8.8 and 12.8 → 5.1 and 21.5 | **freezes** | Freezing until 10:00 on day 0, 69 freezing hours in the week. Food is fine. Wheat sown before about day 5 dies on −2 °C nights. |
| forest | −0.8/−4.3 and 10.6 → 4.5 and 15.9 | **freezes** | 75 h freezing; food fine |
| birch forest | −6.5/−9.6 and 4.6 → 1.2 and 12.6 | **freezes** | 117 h, freezing at 15:00 on day 0; food fine |
| taiga | −21.6/−28.9 and −6.4 → −9.7 and 5.8 | **freezes all week** | Freezing even beside a furnace (−7.6 °C) and in a cave (−2.1). No farm animals on podzol and no meat from hunting, so only berries (max health about 41) |
| snowy plains | −23.6/−35.6 and −14 → −14.7 and −4.8 | **dies in 5 minutes** | A cave with a furnace reaches only Cold. No fibre, no meat |
| mountains | −5.0/−9.3 and 8.1 → 0.2 and 13.5 at sea level (y 130 is 2.6 °C colder) | **freezes** | 108 h; food fine |
| desert | 12.2 and 31.2 → 17.1 and 36.2 | fine for heat, **weak for food** | Hot afternoons from day 3 (18 h). Prickly pear is the only food (about 1% of sand); no protein |
| savanna | 21.1 and 33.7 → 22.1 and 34.7 | **fine, nearly too easy** | 29 hot hours (stamina regen −25%); farm animals |
| swamp | 16.5 and 32.1 → 18.5 and 34.2 | **fine** | farm animals; cattails give nothing |
| beach | 9.7 and 21.3 → 13.4 and 25.1 | warm enough, **weak for food** | No edible plant, no farm animals, no fishing |

Stews (×1.25, well fed from three foods) need a 7-iron pot. Drying runs near zero below 5 °C, so
in temperate week one it takes 4–6 days (about 1.5 in the desert). Smoking needs a furnace or lava
under the rack. Preserving helps from day 2–3 on, but changes none of these verdicts.

### What three changes do together

Week-one hours, freezing / cold / hot, from the same script:

| change | plains | forest | birch | mountains | taiga | snowy |
|---|---|---|---|---|---|---|
| none (now) | 69/30/0 | 75/43/0 | 117/39/0 | 108/38/0 | 168/0/0 | 168/0/0 |
| plain clothes, 1 clo | 0/17/0 | 0/0/0 | 0/33/0 | 0/39/0 | 111/36/0 | 168/0/0 |
| plain clothes, and the year starting 7 days later | 0/0/0 | 0/0/0 | 0/0/0 | 0/0/0 | 6/21/0 | 79/60/0 |
| a full fur set | 0/0/0 | 0/0/0 | 0/0/0 | 0/0/0 | 0/9/0 | 0/40/0 |

With plain clothes and the later start, the temperate biomes become a cold first week rather than
a lethal one. Taiga and the snowy plains stay hard, and still need fire and fur, which is right
for Fairbanks and Utqiaġvik in April. The Long Dark and Vintage Story both start players dressed
(round 1, F1).

### The changes, with their real numbers

**B1. Survival, S–M. Start warm enough to live.**
1. **Plain clothes on arrival.** A `clothing::ARRIVAL` beside `stomach::ARRIVAL` in `Vitals::new`.
   - It is worth about 1 clo: ordinary indoor clothing is 0.8–1.2 clo (ISO 9920).
   - Either a "plain" material, or a hide coat, trousers and boots with a linen hat, which comes to
     about 8.7 °C.
2. **A campfire.**
   - `"base:campfire" => 1.0` in `base_api::heat_of`'s table: +20 °C beside it, enough to smoke
     too (`SMOKE_HEAT` 0.5).
   - It counts as a fire for camp and rest (`survival/camp.rs`).
   - Made from logs and sticks, with no stone.
   - A real open fire felt at 1–2 m is worth about +15–25 °C.
3. **The year starts at spring's 8th day**, `from_midwinter` 0.25. It is a world-start offset in
   `vitals::calendar`, and it changes nothing else in the calendar. Plains' day-0 mean goes from
   2.8 to 12.8 °C.
   - **Check with the breeding seasons and plants:** `places.rs` holds eggs and seasons against
     the calendar.

**B2. Animals with Survival, M. Meat from wild animals.**
- A `MEAT` table beside `HIDES` in `animals_core`, at real carcass yields: about 40–50% of live
  mass is meat, so a 70 kg deer gives about 30 kg. The game's portion count follows from it, as
  hides' does.
- `FOODS` rows from USDA data:
  - venison (raw: water 73.6, protein 21.8, fat 2.4 per 100 g);
  - hare (72.8, 21.8, 2.3);
  - trout;
  - pacu.
- Each gets the `raw_meat/<m>` and `cooked_meat/<m>` tags, so smelting, salting, drying, smoking
  and stews take them with no new code.
- The icons are drawn already: `mods/animals/assets/textures/item/{venison,cooked_venison,hare,cooked_hare,trout,cooked_trout,pacu,cooked_pacu}.png`.
- Fish need a way to be caught (fishing, round 1's F8). Until then their meat comes only from
  hunting them in shallows.

**B3. Survival, S. Clothes within reach.**
- `vitals::clothing::MAKES` goes from (2, 4, 3, 2) pieces to (1, 2, 1, 1) for hide and wool.
  - A real buckskin suit takes 3–4 deer hides.
  - One fleece knits more than a sweater.
  - Fur can stay at 11 pelts, about 4–5 reindeer, near a real caribou-skin suit.
- String: 2 fibre a string. Sedge, cottongrass and cattail give fibre; they are real cordage
  plants.
- Then a linen set is about 300 grass, and the snowy plains have fibre.

**B4. Wild plants, S. Spring's real food.**
- `wild.rs`/`blocks.rs`: an unripe bush drops nothing. This ends out-of-season cloudberries,
  raspberries and hazelnuts.
- `wild_rice` gets a `FOODS` row: USDA cooked wild rice is water 73.9, protein 4.0, carbs 21.3,
  fat 0.3 and fibre 1.8 per 100 g. Then `cooked_rice.png`, also drawn and unused, has an item to
  be.
- `cattail` gives `base:cattail_root`. Its numbers are an estimate, about 17 g starch per 100 g
  fresh rhizome. It is the classic spring survival starch.
- Lingonberry and crowberry generate ripe in winter and spring: both really hold their fruit over
  winter.

**B5. Survival, Wild plants, Animals, S. Wind chill only on bodies.** Read `.still`, not `.celsius`,
in:
- `growth.rs:55` (plant frost);
- `eggs.rs:380` (egg warmth);
- `cooking.rs:245` (drying).

**B6. Survival, S. Meal warmth from the strongest dish, not the sum.**
- `Stomach::meal_warmth` adds every portion: four hot steaks give +16 °C.
- Five handfuls of lingonberries in the taiga, the only food there, give −10 °C and turn Cold into
  Freezing.
- Take the largest warming and the largest cooling, and let cooling foods cool only above the
  comfort range.

### Smaller balance points

- **B7. Survival, S.** The water use of Warm, Hot and Heatstroke (×1.5, ×2, ×3) does nothing: only
  `sheet.rs` reads `water_use`. Wire it into the stomach's water, or drop it from the sheet so
  nobody tunes it.
- **B8. Survival, S.** Salt boils out of any water bucket (`cooking.rs:66`), though its comment says
  sea water. Take only water from ocean or beach biomes, or say fresh water gives none.
- **B9. Wild plants, S.** The `animal_spawn` tag is only `grass`, so pigs, cows, sheep and chickens
  never spawn on podzol or snowy grass. The taiga and snowy plains are tagged for animals, but
  their ground isn't.
- **B10. Animals, S.** The capybara lives in `base:forest`, whose climate is Kassel's beech forest.
  That isn't a fact: capybaras are South American wetland and savanna animals.
- **B11. Animals, S.** `SPAWN_FLOOR` (0.2) is a loose constant in `mods/animals/src/forage.rs`.
  Move it to `animals_core`.
- **B12. Survival, S.** The cooking pot costs 7 iron (`cooking.rs:68`), and salt needs an iron
  bucket, so hot food and preserving wait on iron. A clay or stone pot (clay first needs U1) would
  put stews in week one.

---

## 2. Code that clashes across sessions

**D1. Bug, Survival, S. Spoiled food comes back fresh.**
- `cooking::stew` (`mods/base/src/cooking.rs:155-173`) builds the stew from the food indexes
  alone. It stamps it `MADE` today and `HOT`, with no look at its ingredients' freshness.
- The rack stamps its product fresh as of today (`cooking.rs:261`).
- The salted-meat recipe (`cooking.rs:67`) makes a 90-day food out of any raw meat.
- **Fix:** refuse spoiled inputs in the pot's and the rack's slot checks, and give the product the
  oldest ingredient's age, as Vintage Story's meals do (round 1).

**C1. Clash, Survival, S. Leather's two warmth numbers.**
- Leather armor (`vitals/src/temperature.rs:163`, `:175`, and `clothing.rs:84-90` for wind)
  against hide clothes (`vitals/src/clothing.rs:47`).
- Both are made from `base:leather`, and they stack.
- **Fix:** derive the armor's warmth, wetness and wind from the hide material, and decide whether
  armor over a coat stacks. Real layers do, at less than the sum.

**C2. Clash, Survival vs Wild plants, S. The drying rack knows two foods.**
- `vitals/src/preserve.rs:35-45` dries only `base:apple` and `base:berries`.
- Lingonberries, cloudberries, crowberries, raspberries, rose hips and mushrooms
  (`vitals/src/food.rs:340-430`) sit on the rack forever at zero (`cooking.rs:257`), while the
  tooltip promises "berries" (`en_us.lang:423`).
- **Fix:** a "dries to" field on the `FOODS` row, or dry any watery fruit or fungus. Let the rack
  take only what can dry.

**C3. Clash, Wild plants, M. The berry bush is coded twice.**
- The old bush has its own code in five places:
  - registration and loot (`blocks.rs:571-596`);
  - frost (`growth.rs`);
  - planting (`items.rs:130`);
  - forage (`vitals/src/forage.rs:84`);
  - placement (`worldgen.rs:422`).
- All of it is what `vitals::wild::WILD` and `wild.rs:52-93` already do for every bush.
- **Fix:** one row in `WILD`, and the special cases go.

**C4. Clash, Survival, S. Every food is listed twice.**
- `items.rs:74-109` is a hand list of food names, with the old hunger and saturation numbers, that
  must mirror `vitals::food::FOODS`.
- The testkit check (`gameplay.rs:1209`) goes one way only: an item without a row fails, a row
  without an item passes.
- **Fix:** register the base foods by looping over `FOODS`'s `base:` rows, with the meat tags as a
  field on the row. B2's new meats then need no second list.

**C5. Clash, Animals vs Wild plants and Survival, M. What hand-feeding means is apart from diets.**
- Each species' taming foods (`animals_core/src/species.rs`, e.g. reindeer `:1630`, red fox
  `:1241`, white-tailed deer `:1003`) are their own list.
- None of the new foods is on one: lingonberries, crowberries, hazelnuts, dried meat, mushrooms.
- Nothing ties a taming food to the diet.
- **Fix:** a check in `animals_core` that every taming food's category is in the diet, then add the
  new foods where the species really eats them. Reindeer and lichen is the obvious one.

**C6. Clash, Animals, Wild plants and Survival, S. Who eats what off the ground.**
- Wild rice gives `base:rice`, which only animals can eat (`forage.rs:202`). That is B4's row.
- Kibble is filed as meat (`food.rs:458`), so plant eaters never take it off the ground, though
  `taming.rs:58` says every species takes it.
- Animals take poison off the ground: a dropped fly agaric or spider eye is eaten.
- Preserved meat reaches meat eaters as meat. Whether a fox should eat salted meat is Ronald's call.

**C7. Tidy, Survival, Animals, Wild plants, S–M. Name matching left beside the new table.**
- Survival's commit `d19c9e6` made `heat_of` a table per block state (round 1, P1). Three
  neighbours still compare names:
  - `survival/camp.rs:18-31` does it for every block of a 15³ scan, with its own fire list;
  - animals' `forage.rs:108-112` reads names and scans the 54 forage rows by string;
  - `ground_at`'s water test (`api/src/lib.rs`) checks 162 names per growth step, now for 19 more
    plant kinds, where the rest of the code uses the `water` tag.
- **Fix:** camp reads the heat table; forage gets a table per state; `ground_at` uses the tag.

**C8. Tidy, all, S. The repeats from round 1, still there and grown.**
- **Hex** is now written seven times. New since round 1: `server/persist.rs:294-302`, which is
  public, skips bad pairs quietly and can panic on non-ASCII.
- **Stores** still write JSON straight over the file: `links.rs:221`, `friends.rs:66`,
  `saved_worlds.rs:56`, `package_trust.rs:102-110`. Meanwhile `packages::store` (`store.rs:297`)
  and Phase 6's sandbox store (`storage.rs:79`) each write a temporary file and rename it. Two
  sessions wrote the safe way separately. **Fix:** one shared write, R1.
- **Constants:** `DAY_SECS` is 1200 in `vitals` and 600 in `animals_core::needs`. `FLOOR = 0.4` is
  copied from `vitals/src/stomach.rs:14` into `animals/src/comfort.rs:47`. `dist2` is written seven
  times in animals.

**C9. Tidy, Wild plants, S. Stand-in forage rows now doubled.** `forage.rs:91` and `:103` make
cactus bear pads and prickly pears, and `:127` puts mushrooms in podzol. The real rows are `:170`,
`:177` and `:181`. Drop the stand-ins.

**C10. Tidy, Survival, S. Loose numbers and two spellings.**
- **Weights:** `load.rs:59` weighs a stew at a fixed 0.5 kg, though `meals::grams` knows its real
  weight (up to about 0.8 kg). Clothing weights live in `load.rs:62-65` instead of `vitals`.
- **Feet slot:** `SHOES` (slot, texture) against `boots` (pieces, items).
- **Fibre:** `plant_fibre` against `fiber` (nutrition).

**Q1. CI, S. Tests nobody runs.**
- `.github/workflows/ci.yml:54, 57, 67` leave out:
  - `chiacraft-sandbox` (`tests/sandbox.rs`, and 3 tests in `storage.rs`);
  - the packages mod (2 tests in `sandboxed.rs`);
  - `base_api` (survival's new full-stamina test);
  - `base` (`player.rs`).
- Add them. Ronald's rule leaves CI the only place tests run.

**Checked and fine.**
- Every item and block the wild plants and survival added has a key in both base lang files.
- No literal key in base, its API or animals is missing from English.
- No `#[allow(dead_code)]`, and no TODO pointing at finished work.
- The `notes/SYSTEMS.md` rows checked all name code that exists.
- Phase 6 already does what round 1 asked: fuel with an epoch watchdog, `StoreLimits` with
  `trap_on_grow_failure`, and a versioned `wit/v1.0`.

---

## 3. What a new player meets in the first hour

In the order it happens:

1. **Cold, at once.** B1. Nothing points the player at clothing or fire before the first night:
   the start has an empty inventory, and no advancement covers either.
2. **The HUD in English for a Russian player.** T1:
   - base `ru_ru.lang` lacks 83 text keys. First ten: `hud.base.bar_value`,
     `hud.base.clock_day`, `hud.base.clock_night`, `hud.base.comfort`, `hud.base.effect.bleeding`,
     `hud.base.effect.chilly`, `hud.base.effect.cold`, `hud.base.effect.encumbered`,
     `hud.base.effect.food_poisoning`, `hud.base.effect.freezing`.
   - Then all 18 `stat.base.*`, the 7 `levels.base.*`, `tagged.base.freshness` and
     `tagged.base.nutrition`.
   - The clothing and cooking keys are all there.
3. **Killing a deer gives leather and nothing to eat.** B2.
4. **A torch under the drying rack does nothing.**
   - For camp and rest a torch is a fire (`survival/camp.rs:20`). For smoking it isn't: only a
     furnace or lava reaches `SMOKE_HEAT`.
   - The rack's tooltip says "over a fire below".
   - **Fix:** the campfire (B1), and the tooltip names what counts.
5. **The new stations can't be looked up.** The cooking pot and the drying rack aren't in the
   recipe viewer: no `.recipes()` at `cooking.rs:76` and `:111`. Fur, plant fibre and salt have no
   tooltip saying where they come from, and salt (a water bucket in a furnace) is impossible to
   guess. **Fix (Survival, S):** register both stations with the viewer, and give the three
   materials a "from" line.
6. **Things that can't be had** (U1):

   | item | why | owner | fix |
   |---|---|---|---|
   | **clay** (so bricks, slime blocks, sticky pistons) | never generated | Wild plants | patches under shallow water and in swamps, as real clay lies |
   | **ice** (so blue dye: blue glass and the blue lamp) | drops nothing (`blocks.rs:244`) | Survival | drops itself with a pickaxe, or a second blue dye (woad, cornflower) |
   | **arrows** (so tranquilizer darts) | skeletons only, no recipe, no bow | Animals | an arrow recipe from stick, feather and flint, or darts without arrows |
   | **portal block** | creative only (`portals/src/lib.rs:82-87`) | Ronald's call, then Phase 3 | can survival owners craft one? |
   | **fish eggs** (`brown_trout_egg`, `pacu_egg`) | registered but nameless: no `item.animals.*` key, so the raw id shows | Animals | add the keys |
   | mossy cobblestone, snow block, leaves, piston heads | low priority | — | snow block is the "summit" advancement's icon; leaves need shears, which don't exist |

7. **Art that is drawn but not used** (S each):
   - **Compost bin (Survival):** `compost_bin_side.png` and `compost_bin_top_{empty,filling,full}.png`
     exist, but `blocks.rs:599-613` still shows planks, dirt and podzol. `notes/STYLE.md:176` lists
     them as still to make.
   - **Grazed tufts (Wild plants):** `sedge_grazed`, `clover_grazed`, `cottongrass_grazed` and
     `reindeer_lichen_grazed` exist, but `wild.rs:47-49` shows one texture for both ages.
   - **The game meats and cooked rice:** B2, B4.
8. **Art that reads wrong:**
   - eight pairs of HUD effect icons are the same file, so good and bad states look alike:
     near fire = heatstroke, rested = sedated, poisoned = food poisoning, encumbered = overloaded,
     chilly = cold, wet = the air bubble, well fed = full, warm = the day icon;
   - the bowl and the stew are hard to tell apart;
   - the tombstone, seen at the first death, is a box of mossy cobblestone.

   Owner: art (the `game-art` skill). M for the icons.
9. **No guidance into the new loop.**
   - The 19 advancements (`advancements.rs:65-83`) follow vanilla's ladder. None covers clothes,
     fire, camp, stew, salt, the rack or taming.
   - **Fix (Survival, S–M):** five advancements as a first-week path: fire → clothes → a stew → a
     preserved food → a tame. Each hint says where the materials come from.
10. **Commands where a screen belongs.** Tribes are only `/tribe` and `/tribeinvite`
    (`animals/src/commands.rs:127-129`), and `/rename` stands in for an anvil. Later.

**Other translation gaps (T2):**
- **Animals and portals have no `ru`** (263 and 17 keys). The rule asks only for `en` there, so
  this is Ronald's call, but the animal screens will be English for Russian players.
- **Engine `ru`** lacks 100 text keys, `fr` 86. First ten in `ru`: `account.backed_up`,
  `account.backup_needed`, `account.error`, `account.fingerprint`, `account.hide_words`,
  `account.identity`, `account.none`, `account.restore`, `account.restore_hint`,
  `account.restore_title`. Owner: Phase 3 (they are its screens).
- **English sent to players:** only the `/animal` debug command (`screen.rs:541, 576, 588`), so
  low priority.

The key diffs and the week-one numbers came from small read-only scripts, kept in
`reports/round-3-scripts/` (see its README) so the next merge can be checked the same way:
- `lang.py`: the key sets compared;
- `keys.py`: keys used in code;
- `art.py`: missing and identical textures;
- `air.py`, `body.py`, `food.py`: the week-one air, body and food.

Worth turning into a CI check: "every key in `en` is in `ru`", reported, not failing, until T1 is
done.

---

## By owner

| owner | ids |
|---|---|
| Survival | B1, B3 (string, pieces), B5, B6, B7, B8, B12, D1, C1, C2, C4, C7 (camp), C10, T1 (base `ru`), the stations in the recipe viewer, the compost textures, ice, first-week advancements |
| Animals | B2 (with Survival for the food rows), B10, B11, C5, C6 (kibble, poison), C7 (forage), C8 (`DAY_SECS`, `FLOOR`, `dist2`), arrows or darts, fish-egg names |
| Wild plants (Voxel dog handoff) | B4, B9, B3 (fibre from sedge, cottongrass, cattail), B5 (`growth.rs`), C3, C9, C7 (`ground_at`), clay, grazed textures |
| Phase 3 leftovers | T2 (engine `ru`/`fr` for its screens), C8 (atomic stores = R1, hex) |
| CI | Q1, the lang-diff report |
| Ronald | the portal recipe; `ru` for animals and portals; whether salted meat is fox food; whether armor and clothes stack |
| Art | the duplicate effect icons, the bowl and stew, the tombstone |

## Not checked

- No game was run, so these are from the code, not seen on screen.
- The week-one hours come from a script that mirrors `temperature::air` and `Body::target`. A
  short Tester run in plains and taiga at spring day 1 would confirm the two ends.
- The survival branch was being worked on while it was read. If its numbers changed since,
  B1–B6's hours move with them.
