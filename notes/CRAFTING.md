# Crafting

Status: proposal, 2026-11-04. Nothing here is built. Four choices are matters of taste and wait for
Ronald (at the end); the rest follows from the research and the plans already made.

- **Research, with sources:** `reports/crafting/Crafting in other games.md` (Valheim, Vintage Story,
  TerraFirmaCraft, Enshrouded, V Rising, Minecraft, Factorio, Hytale).
- **Mock-up:** `notes/crafting/mockup.png`. It shows the workbench screen, stations with add-ons,
  knapping and the anvil. Its icons are today's; flint, sinew and the rest are stand-ins.
- **Related plans:**
  - `notes/CONTENT.md`: the material ladder, the stations and the crafts that wait. Its open
    decision 5 (a few stations with add-ons) and decision 6 (crafts that wait) are the same
    questions as here.
  - `notes/SURVIVAL.md`: comfort, rested, temperature.
  - Ronald's UI sheet: its Crafting screen has categories, a grid, a quantity and a Craft button.

## The idea in one paragraph

You make things at a few stations that grow as you build their add-ons beside them. The screen
shows a list of what you know how to make. You pick one, its ghost fills the grid, and Craft takes
the ingredients from your bag and the chests around you. Recipes appear as you touch their
materials. The crafts that are really hands-on (knapping flint, shaping clay, forging metal) are
short games the first few times; after that you can do them from memory. The crafts that really
wait (tanning, drying, firing, charcoal, smelting) run in the world while you do other things.
Real materials and real temperatures decide how good a thing comes out:
- the flint you found
- a hide taken in autumn
- iron worked at the right heat on a winter day

## What we have (read before designing)

- **The engine (Weft):**
  - Recipe kinds (`r.recipe_kind`), shaped and shapeless recipes of `#tags`, and variable
    recipes (`#ores/{m}`), with time, several outputs, chances, values and hidden recipes.
  - `recipe_match`, `recipe_find` and `recipe_process` for menus and machines.
  - The recipe viewer, R and U over an item (`core:recipes`, NEI style), with a tab per kind and a
    machine's arrow opening its recipes.
  - Data packages add recipes as TOML (`defs/recipes`).
  - Machines are block entities with slots, a tick and components synced to the screen.
  - Client screens can send packets to the server (`Channel`), and the world can read any block
    entity's inventory (`block_entity`, `inventory`).
  - Items have `durability`.
- **Base:**
  - the 2 × 2 grid in the inventory and the 3 × 3 crafting table, both `base:crafting`
    (`player.rs`, `recipes.rs`)
  - the furnace (`base:smelting`, `machines.rs`)
  - the cooking pot and the drying rack (`cooking.rs`)
  - the campfire on Survival's branch
  - crafting gives XP by how many things went in (`levels::xp::craft`)
  - comfort counts a crafting table as "a table"
- **The gaps:**
  - The engine has no way for a mod to list all recipes. We don't need one, because our recipes
    will be our own data table (below).
  - R shows every recipe, so it would spoil discovery. A survival handbook screen would replace it.
  - No station knows what stands near it.
  - Nothing crafts from chests, nothing queues, and nothing has quality.

None of it needs an engine change. It's all base and a new shared crate, as `notes/PLAN.md` rule 4
asks.

## What the player does

1. **Hands first.** The inventory's Crafting tab shows the hand recipes, 2 × 2: sticks, cordage
   from plant fibre, a campfire, a flake struck off a flint nodule. It's the same screen as a
   station, with fewer recipes.
2. **A workbench.** Made of logs and cordage, it replaces the crafting table (same block, renamed,
   "table" for comfort as today). It opens the workbench screen: the hand recipes and its own.
3. **Add-ons raise its level.** Each different add-on standing within 4 blocks raises the level by
   one, in any order:
   - knapping stone (stone tools)
   - chopping block (hafts, bows, furniture)
   - tanning frame (hides; also a "waiting" station)
   - sewing table (clothes)

   The screen's tab says "Workbench · level 2", and a recipe says which add-on it needs. Valheim's
   rule, but at 4 blocks instead of its 2 m, so a room can hold them. There's no roof rule, which
   is Valheim's most confusing one. Shelter matters through comfort and warmth instead.
4. **Later stations** work the same way, each with its add-ons:
   - campfire, then hearth (cooking, birch tar, fire-hardening)
   - kiln, with a chimney (hotter) and a drying shelf
   - forge, with bellows (hotter), anvil and quench trough
   - loom
   - bloomery (iron)

   They follow Content's ladder. Content owns which stations and items exist; this plan owns how
   they work.
5. **Crafting from nearby chests.** Ingredients come from your bag first, then from every chest
   within 7 blocks of the station that you could open. That's the same 7 blocks as comfort.
   - The needs list says where each one comes from ("bag 8 · chest 4").
   - The chests in use glow faintly while the screen is open.
   - Results go to the bag, or to the station's output slot when the bag is full.
   - This fixes the most-modded complaint in Valheim.
6. **A queue.** Craft 2 queues two. Each recipe has a time: hand work 1–4 s, a bow or a coat
   longer.
   - The queue keeps going while you stay within 7 blocks and pauses when you walk away.
   - Ingredients are taken when you queue, and cancelling gives them back.
   - Sub-parts are made for you one level deep: the axe needs 2 sticks, you have planks, so the
     queue makes sticks first (Factorio).
   - Shift-click crafts as many as you can.
7. **Crafts that wait** aren't in the queue. They're machines in the world, like the rack today,
   and run while you're away:
   - the tanning frame
   - the drying rack and the smokehouse
   - the kiln firing pots
   - the charcoal clamp
   - the bloomery

   Their times are Content's real times on the growing pace: a buckskin 3 minutes, sole leather 4
   game days, a kiln load 1 minute. When you come back, they report what happened ("fired 4 bowls,
   1 cracked").

## How it ties to the real world

- **Materials decide quality:**
  - Flint knaps finer than chert, obsidian is sharpest but short-lived, and a bone tool is tough
    but dull. These are Content's material rows (`vitals::gear`).
  - A hide taken in autumn or winter is prime: a warmer coat, a better grade.
  - Bronze in its real range (88–92% copper) makes a good piece; outside it, a poor one.
- **Heat is real at the forge and kiln:**
  - A piece has a temperature. The forge heats it, and the air cools it by its mass and by
    `air_at`: cold, wind and rain cool faster.
  - So a winter forge outdoors works in short bursts, and one inside a warm smithy works in long
    ones.
  - Bigger pieces hold heat longer. That's real, and it fixes the Vintage Story complaint where
    big pieces cool in 15 s.
  - Iron is worked bright orange to yellow (about 900–1250 °C), and welding needs near-white heat.
    Under dull red it stops. Copper is hammered cold and softened in the fire.
  - A kiln needs about 900 °C+ for good pottery: wood in a kiln reaches it, an open bonfire mostly
    doesn't, so more pots crack.
  - All these numbers go in the data crate with sources, checked like the survival sheet.
- **Clay dries before firing.** It dries by the air where it stands, like the drying rack today:
  slow and damp in the rain, quick in a dry wind.
- **Cold hands:**
  - When you're Freezing, hand work takes twice as long.
  - In a warm, comfortable room (Survival's comfort 3+), crafting is 25% faster: winter evenings
    by the fire were craft time.
  - Being Rested adds its XP bonus as today.
- **Seasons set the work** through the materials: hides in autumn, flax retted in summer,
  charcoal in the dry months (Content).

## Recipes are data

- **The table:** recipes live as rows in a new shared crate, `mods/base/crafting`, beside
  `vitals`. Each row holds:
  - its station and add-on
  - inputs as `#tags`, the output
  - a time
  - its skill (knapping, smithing...)
  - how quality is decided
  - what reveals it
- **Registration:** base registers every row with the engine as today, one recipe kind per
  station: `base:hand`, `base:workbench`, `base:forge`, `base:kiln`... The engine's matching and
  recipe viewer keep working, and so do data packages (`kind = "base:workbench"` in a TOML).
- **What the crate gives:**
  - the matching for discovery (which rows a picked-up item reveals)
  - quality, cooling and queue maths, all pure functions with tests
  - a generated sheet, `mods/base/crafting/SHEET.md`
- **Checks:**
  - every station's recipes reach the next tier
  - every input tag has a source (Content's check)
  - every recipe has a name key in `en` and `ru`
- **Text:** every word on these screens is a translation key, in base's lang files.

## The choices for Ronald

### 1. The screen

| | What it is | For | Against |
|---|---|---|---|
| **A. List and ghost grid** (recommended) | Categories, a list of known recipes, the picked one's ghost in the grid, the needs with counts, a quantity and Craft. Hand-filling the grid still works. | It's Ronald's UI sheet. It's quick, it shows exactly what's missing, it works with a controller, and the grid still finds unknown shapes. | The most screen to build. |
| B. List only (Valheim, V Rising) | No grid: pick, see needs, Craft. | Simplest and cleanest. | Loses the grid's discovery and the Minecraft feel. |
| C. Grid with a recipe book (Minecraft) | The grid stays the main thing; a book beside it fills it. | Closest to today. | Placing by hand is slow; queues and chests fit awkwardly. |

### 2. How recipes are found

| | What it is | For | Against |
|---|---|---|---|
| **A. By touching materials** (recommended) | A recipe appears once you've held one of each ingredient and its station exists. A recipe you've seen part of shows as a "?" silhouette, a hint of what's next. R and U show only what you know. | It's Valheim's, and it ties progress to going out and finding things. Seasons and places matter. | New players can miss a recipe for an item they haven't touched. |
| B. Everything in the handbook (Vintage Story) | All recipes visible from the start, R as today. | Nothing hidden; easy to plan. | Spoils finding things; the ladder reads like a shopping list. |
| C. Touching, plus knowledge found in the world | As A, plus some recipes (the bloomery, steel, the loom) learned from things found: an old smith's notes in a ruin, a trader, another player's world through a portal. | Gives exploring and linked worlds a reason. | Needs ruins and traders that don't exist yet; more to build. |

### 3. Hands-on making (knapping, clay, smithing)

| | What it is | For | Against |
|---|---|---|---|
| **A. By hand at first, then from memory** (recommended) | The first three of each design are a short game that can't fail: knapping a 7 × 7 flint grid, or the anvil's moves on a heat bar (TerraFirmaCraft's moves, without the maths). After three, "from memory" makes it in a few seconds at Good. Only by hand reaches Fine or Masterwork. | It feels like making a thing, never a chore, and rewards care without forcing it. | Two small games to build (clay forming later, or never). |
| B. None | Everything is list crafting with a time; quality comes from the materials and the station's level. | Least to build; fastest to play. | Loses what makes stone and metal feel different. |
| C. Always by hand (Vintage Story, TerraFirmaCraft) | Every knapped tool and every forged piece is hands-on. | The most real. | The known tedium: auto-smithing mods exist for a reason. |

### 4. Quality and wear

| | What it is | For | Against |
|---|---|---|---|
| **A. Grades from the work** (recommended) | Crude, Good, Fine, Masterwork, set by material, how well it was made and the station's level. A grade adds durability and speed (±15% a step, proposed). Tools used in recipes wear (a knife cutting hide). Repair is at the station that made it and costs a little material (a flake, a bit of metal). | Real: a better smith makes a better axe. Durability stays meaningful. | One more number on every item. |
| B. Upgrade levels (Valheim) | Craft it again at a higher station level with more material to raise it a level. | Clear path; Valheim players know it. | Not how real things are made; feels gamey. |
| C. Durability only | As today; repair at the station. | Simplest. | Nothing to get better at. |

## What gets built, in slices

Each slice is seen in the game through Tester before the next. Every string is a key, and
recipes are rows.

1. **The screen and nearby chests:** the workbench screen (choice 1) on today's recipes.
   - the inventory's Crafting tab with the hand recipes
   - ingredients from the bag and chests within 7 blocks
   - quantity and Craft
   - the `crafting` crate with today's recipes moved into it
   - a testkit script: craft from a chest, craft ×N, a missing ingredient
2. **Discovery** (choice 2):
   - known recipes per player, saved
   - the "?" hints
   - the survival handbook in place of R and U
   - XP for a first craft
3. **Stations, add-ons and the queue:**
   - workbench levels from add-ons within 4 blocks
   - the queue with sub-parts and refunds
   - cold hands and the warm-room bonus
4. **Quality, wear and repair** (choice 4), then **knapping** (choice 3). Needs Content's flint,
   chert and obsidian items and its first stone tools (Content's step 1).
5. **Crafts that wait** as machines (the tanning frame first), with Content's times and Survival's
   rack as the model.
6. **Smithing:** the forge, the anvil, heat that cools by the air. With Content's copper, bronze
   and iron steps.

## Who does what

- **Content plan:** which items, materials, stations and add-ons exist, and their numbers
  (`vitals::gear`). This plan's recipes for new items wait on its list.
- **Survival:** the pot, the rack and the campfire stay theirs. They become stations of this
  system only where Survival agrees. The campfire as the first station is the natural one.
- **UI redo:** styles the crafting screen. The mock-up's layout is the proposal to agree on.
- **Chiacraft (art):** icons for every new item, station and add-on.
- **Tester:** every slice in the game.
