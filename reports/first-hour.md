# The first hour

Researcher, 2026-11-04, for Planner.

**Read:**
- `dev` at `8a70ba89`, and the Survival branch (`claude/survival-clothing-cooking-f0c469`), which
  is 11 commits ahead of `dev`;
- `notes/SYSTEMS.md` and `SURVIVAL.md`;
- `notes/CRAFTING.md` on `claude/crafting-d352ab` (a proposal; nothing built);
- Planner's board, `notes/crew/BOARD.md`.

**The rest of the evidence:** research rounds 3 and 4, and how five games teach the first hour
(sources at the end). No game was run.

**The one thing to know: our first hour is mostly built, but not in `dev`.** The hints, the
Surviving advancements, the campfire, the respawn kit, the spring-day-8 start, the comes-from
tooltips and the Russian HUD are all on the Survival branch, unmerged.

## 1. Our first hour, step by step

How each step goes in `dev` today, and what the Survival branch adds:

| step | in `dev` | with the Survival branch | still missing or confusing |
|---|---|---|---|
| Title, Select World | Footer says "Weft". **Address** sits next to Delete. | same | Multiplayer jargon on the first screen. |
| Create World | "Survival: Normal", no tooltip | same | Reads like a mode, not a difficulty. Nothing says Gentle can't kill you. |
| Spawn | Plains, **spring day 1**, no clothes, bread and an apple in the stomach | Spring day 8, plain clothes (about 0.9 clo) | Fine on the branch. |
| First tools | Vanilla's ladder, with advancements | same | The HUD shows **R**, but R only works over a hovered item in a screen. The wyvern's breath is also R. |
| Food | Deer, hare, trout and pacu give meat. Cooking needs a furnace (8 cobble). | The campfire cooks (1.5× slower, no fuel) | Bushes drop fruit even unripe. Wild rice and cattail feed nobody. |
| Fire | **No campfire.** Rest and warmth come from a torch, a lit furnace or lava. | 2×2 campfire (2 logs, 2 sticks); the night hint at each dusk until one is built | It never goes out (fine for now). |
| Clothes | Naked; a hide coat is 4 leather and 2 string, about 48 tall-grass breaks | Cheaper pieces; "comes from" tooltips | Fibre only from tall grass; sedge and cattail give none. |
| Shelter, rest | Rested from a roof and a fire, but nothing says so | The rest hint | Fine on the branch. |
| First night (about 10 real minutes in, like Minecraft's) | Vanilla monsters; the light meal survives one spider | The campfire's light keeps monsters off | Fine on the branch. |
| **First death** | Everything into the tombstone, **clothes included**. "You Died!" with no cause or place. Respawn with an empty stomach (40% caps) and **0 stamina**, often at night. | Plain clothes back on, the arrival meal, full stamina. A hint gives the tombstone's x and z. | No cause on the death screen. No way back to the tombstone but numbers. Nothing stops a second death on the walk back. |
| HUD | "12°C" (felt air, unlabelled), effect icons without names, P/E/F/G/W without a legend | same | Unreadable until learned. |

## 2. What other games do that we should copy

1. **Recipes appear once every ingredient has been touched, with "2 of 3" near misses shown.**
   Valheim does this, and Don't Starve keeps its fire recipes always visible.
   - *Pitfall:* Minecraft unlocks on the first ingredient, so the book floods.
   - CRAFTING.md's choice 2A is this: keep it to *all* ingredients.
2. **Hints fire once from the game's state, name the key or item, and can be read again.**
   - Valheim's raven Hugin shows each tip once, and they can be re-read in the inventory.
   - Minecraft Java has 6 hints, from moving to crafting planks.
   - *Pitfall:* tips that say what without how.
3. **Warn before the night punishes.** Don't Starve's darkness monster speaks before every hit;
   Vintage Story warns of storms in chat. Our dusk hint is that.
4. **A tombstone with mercy.** Valheim gives its "corpse run" buff for picking items up and no skill
   loss for 10 minutes after dying.
   - *Pitfall:* the death spiral, which is ours today.
5. **An honest gentle preset with a grace period.** Vintage Story's Exploration keeps your
   inventory and keeps monsters away for 5 days.

## 3. Changes, ranked by gain for effort

| # | change | owner | effort |
|---|---|---|---|
| 1 | **Merge the Survival branch into `dev`**, after one Tester run of the first hour (below). It brings 11 hints, the Surviving group, the cooking campfire, the respawn kit, spring day 8, the comes-from tooltips and the 106 Russian keys at once. | Planner, Survival, Tester | S |
| 2 | **The death screen and the way back:** the cause of death, the tombstone's place, and a marker pointing to it after respawning. Protection from monsters until the tombstone is reached or 3 minutes pass. Respawn with full stamina (dev still sets 0, `survival/mod.rs:56`). | Survival, engine/UI | S–M |
| 3 | **R and U with nothing hovered open the whole recipe list**, and the HUD's shortcut pictures carry their screens' names. Move the wyvern's breath off R (the board says V; `dev` still has R, `animals/src/breath.rs:76`). When Crafting's book lands: unlock on *all* ingredients, with "2 of 3" near misses. | engine/UI, animals, Crafting | S |
| 4 | **A readable HUD:** "feels 12°C", effect names on hover, and a nutrient legend in the stomach's tooltip. The stat descriptions already exist (`lang:289`). | Survival, UI redo | S |
| 5 | **Spring plant food in the right order.** First wild rice and cattail root become food, and sedge and cattail give fibre. *Then* unripe bushes stop dropping fruit. In the other order, spring plains have no plant food at all. | Wild plants | S |
| 6 | **Animals say what they can do.** One line when an animal is tamed: "Sneak and use it for its screen. J follow, K stay, H come." Tooltips on the spyglass, club, dart, sedative, saddle (with its bond rule) and amber. The animals mod calls `base_api::milestone` so the `tame` advancement can fire (today nothing does). | Animals | S |
| 7 | **Create World says what it means.** "Survival difficulty" with its tooltip shown. Gentle gets a grace period (no hostile monsters for the first 3 nights) and keeps clothes on death. | Survival, engine/UI | S–M |
| 8 | **The first screens tidy:** Address moves behind multiplayer, and the footer says chiacraft. | Phase 3, engine | S |

**A Tester run for change 1:** the Survival branch's build, plus a script.
1. A fresh world in plains: wait to dusk, and check the night hint fires.
2. Craft and place a campfire, and cook meat on it.
3. Die (`/tk hurt_me`), and check the respawn: plain clothes on, the arrival meal, full stamina, the
   tombstone hint.
4. The same start in taiga, to see the cold hints fire.
5. Shots at spawn, dusk, the campfire and the respawn.

## 4. Built, and a new player will likely never find it

| what | how it's found today | the cheapest way to show it | owner |
|---|---|---|---|
| The animal screen (sneak and use your tame), stances, signatures | only in the code | the tame-time line (change 6) | Animals |
| Whistles J, K, H, G, B | the key settings | on the animal screen | Animals |
| Tribes | typing `/tribe` | a Tribe button on the animal screen | Animals |
| Knock-out taming (club, dart, sedative) | the spyglass says "tamed when knocked out" | tooltips | Animals |
| Riding a Utahraptor (saddle, bond 25+) | the saddle's recipe | the saddle's tooltip, a Rider advancement | Animals |
| The Cretaceous gateway (amber frame lit by a torch) | nowhere | the amber tooltip, an advancement for the first amber | Animals |
| Coats differ by place (gene pools) | invisible | the spyglass: "born near here" | Animals |
| Smoking, salting, salt from a water bucket, stews | the rack's and pot's tooltips; U on a water bucket | register the rack's and pot's outputs as recipes, so R and U show them | Survival |
| Compost and fertile farmland | the bin's recipe; fertility never shown | tooltips on the bin and farmland | Survival, Wild plants |
| Frost kills tender crops | nowhere: they die silently | a hint at the first frosted crop; "tender" on the seed's tooltip | Wild plants |
| Ripe or unripe wild plants, grazing | looks only | ripeness in the block's name | Wild plants |
| What counts for comfort | the HUD line, only once resting | the rest hint (Survival branch), and a comfort list on hover | Survival |
| Levels and points, the stats screen | "press P" at level-up | names beside the HUD shortcuts (change 3) | Survival, UI redo |
| Letters, the mailbox, the portal recipe, linking worlds | not in `dev` (Phase 3's branch) | on merge, tooltips and a "Linked" advancement | Phase 3 |
| Nutrients P/E/F/G/W | bare letters | the legend (change 4) | Survival |

## Sources

- **Minecraft:** [tutorial hints](https://minecraft.wiki/w/Tutorial_hints), [recipe book](https://minecraft.wiki/w/Recipe_book), [day and night](https://minecraft.wiki/w/Day-night_cycle), [complaints about the book](https://feedback.minecraft.net/hc/en-us/community/posts/360017918692/comments/360002875171).
- **Vintage Story:** [your first day](https://wiki.vintagestory.at/Survival_Guide_-_Your_first_day), [temporal storms](https://wiki.vintagestory.at/Temporal_storm), [presets](https://wiki.vintagestory.at/index.php/Preset_playstyles), [the new-player complaint](https://www.vintagestory.at/forums/topic/10260-the-new-player-experience-in-this-game-is-a-bit-of-a-disaster).
- **TerraFirmaCraft:** [field guide](https://terrafirmacraft.github.io/Field-Guide/en_us/getting_started/), [firepit](https://terrafirmacraft.github.io/Field-Guide/en_us/getting_started/firepit.html).
- **Valheim:** [Hugin](https://pcinvasion.com/valheim-beginners-guide-survival-tips), [recipes from materials](https://game8.co/games/Valheim/archives/320286), [rested and comfort](https://mobalytics.gg/gamebase/guides/valheim-comfort-rested-guide), [death buffs](https://epicdope.com/valheim-effects-how-does-the-no-skill-drain-corpse-run-buff-work).
- **Don't Starve:** [crafting](https://dontstarve.wiki.gg/wiki/Crafting), [darkness](https://dontstarve.wiki.gg/wiki/Charlie/Darkness).
