# First hours in other games: how the primitive start works

Research for chiacraft's "earned" opening (no punching trees; pick up rocks, sticks, fibre;
knap; make cord; haft a stone axe; first fire). Written 2026-11-04.

How sure each fact is:
- **Source code** means read from the mod's own repository (the most reliable).
- **Wiki/guide** means a wiki or guide page, cited.
- **est.** means my own estimate from counting steps and clicks, because no wiki or speedrun
  gives a measured time for that step. Treat these as rough.

Nobody publishes measured "time to first stone axe" figures. Speedruns cover whole games or
bosses, not first tools. So the times below are mostly estimates, checked against the few
player reports there are.

---

## 1. TerraFirmaCraft (TFC), Minecraft mod

### TFC for 1.18 / 1.20 (current, "TFC 3.x")

**On the ground:** loose rocks, sticks and twigs, "scattered on the ground" in almost every
climate. You pick them up by clicking. The rock you find is the rock type under you
(granite, basalt, chalk and so on), so the tool you can make depends on where you are.
(Field guide, source code: `field_guide/en_us/entries/getting_started/introduction.json`)

**Punching:** logs have `requiresCorrectToolForDrops` and strength 8 (vanilla is 2). Punching a
log is slow and drops nothing. Breaking **leaves** with your fist gives sticks.
(Source code: `common/blocks/wood/Wood.java`) One guide puts it as "You cannot simply punch a
tree and start crafting planks." (holy.gg guide)

**First tool, step by step:**
1. Pick up at least 2 rocks of the same rock category, and a stick (ground or leaves).
2. Hold the rocks and right-click the air. A **5x5 knapping grid** opens. It needs 2 rocks in
   hand and uses up 1. (Source code: `tfc/knapping_types/rock.json`)
3. Click squares away until the shape matches a head. The axe head keeps 17 of 25 squares, so
   about 8 clicks. Removed squares cannot be put back. The shape can sit anywhere on the grid.
4. Craft head + stick in the 2x2 inventory grid. Heads stack, tools do not, so players knap
   spare heads.
- Heads: knife, axe, shovel, hoe, hammer, javelin. **There is no stone pickaxe.** The first
  pickaxe is copper. Rocks come in 4 categories (igneous extrusive, igneous intrusive,
  sedimentary, metamorphic) with different tool stats. (Source code: `rock_knapping/*.json`)

**First binding:** none. Head + stick just craft together. No cordage in the stone tier.

**Plant material:** the stone knife cuts grass for **straw**. Straw is kindling and pit-kiln
cover. (Field guide)

**First fire:**
1. Firestarter = 2 sticks (crafted).
2. Throw on one block: **1 log + 3 sticks**, plus up to **5 kindling** (straw, pinecones, paper).
3. Hold right-click with the firestarter on that block. Smoke, then fire. Base chance is 50%,
   and each kindling adds 10%. "It may take a couple tries." (Field guide; source code
   `ServerConfig.java`: `fireStarterChance = 0.5`)
- The log needs an axe, so fire comes after the axe.
- Heating a stick in the firepit gives 2 torches. Torches burn out.

**Then:** clay (knap 5 clay balls into molds) → **pit kiln** (a 1x1 hole, up to 4 items,
8 straw, 8 logs, light it, about 8 in-game hours to 1400 °C) → copper nuggets/ore melted in a
small vessel → poured into a pickaxe-head mold. (Field guide `pit_kiln.json`;
All The Mods Gravitas² guide) About 9-10 small ore pieces make one tool head.

**Steps to stone axe:** about 5 (rocks, stick, open grid, knap, assemble).
**Time to stone axe:** est. 1-3 min experienced, 5-15 min new (the new player has to find the
axe shape in the field guide or JEI). First copper pickaxe: est. 1-3 h experienced; a new
player often takes several sessions. (No measured source. Guides warn that the world seed
"will greatly impact how difficult progression will be": FTB wiki, TerraFirmaPunk.)

**Praised:** the field guide is in the game; the rocks under your feet set what tools you
get; "complex, beautiful, slow-paced and satisfying" (Cereal TFC modpack page); a user review
calls it "Survival Mode as it Should have Been" (Moddex). A guide says the harsh start "is the
point" and opens into "a slower and more satisfying survival progression". (holy.gg)

**Complained about:** too slow to progress, "It takes a looooong time" to get armour, wood
locked behind the hatchet (FTB forum thread); grind at fixed points (TFC forum, "This mod needs
so much grinding or is it just me?"). Knapping itself is rarely the complaint. The complaints
are about copper, ore hunting and later chores.

### Older TFC (1.7.10 "classic", TFC+, TFC2)
- Same core opening. Rocks on the ground; hold 2 of one rock type and right-click to knap;
  the first head should be the axe. Sticks from leaves. Firepit: use the firestarter on the
  block under 3 sticks; it "may take multiple tries", and dropping straw in makes it easier.
  (TFC fandom wiki, 1.7.10; search summary, the page itself is paywalled to fetchers)
- TFC+ (TerraFirmaCraftPlus) added a firepit that needs kindling to light. (tfcplus-bids GitHub)
- TFC2 was a 1.10+ rewrite that was dropped; its ideas went into TFC for 1.12 and later.

---

## 2. No Tree Punching (NTP), Minecraft mod by AlcatrazEscapee

All of this is read from the mod's source (branch `1.20.x`).

**Rule:** blocks drop nothing if broken with the wrong tool. Config keys:
`doBlocksMineWithoutCorrectTool = false`, `doBlocksDropWithoutCorrectTool = false`. Blocks that
break instantly (grass, flowers) still break by hand.

**On the ground:** loose rocks (stone, andesite, diorite, granite, sandstone, red sandstone),
from a worldgen feature (`enableLooseRocksWorldGen = true`). Sticks come from leaves (vanilla).
Gravel can be dug by hand and gives flint. Also a shapeless recipe: 3 gravel → 2 flint.

**First tool, step by step:**
1. Get flint (dig gravel by hand, or 3 gravel → 2 flint).
2. **Knap**: right-click flint on any stone block. Each click: 40% chance the flint is used up
   (`flintKnappingConsumeChance = 0.4`); if used up, 70% chance it gives **2 flint shards**
   (`flintKnappingSuccessChance = 0.7`). So each click has 28% to give 2 shards and 12% to waste
   the flint. A knapping sound plays every click.
3. Sticks from leaves.
4. **Flint knife** = shard over stick.
5. Break grass/tall grass **with the knife**: 25% chance of **plant fibre** (loot table).
   Without a knife you get nothing (12.5% seeds).
6. **Plant string** = 3 plant fibre (shapeless). So about 12 grass blocks per string.
7. **Flint axe** = shard + string + stick (shaped: string, shard / stick).
- Flint pickaxe = 4 shards + string + 2 sticks. Shovel and hoe exist too.
- Vanilla wooden and stone axe/pickaxe recipes are **removed** (empty recipes).
- The flint axe can turn logs and planks into sticks.

**First binding:** plant string from plant fibre. The knife has to come first, so it is a
small chain: knife → fibre → string → axe.

**First fire:** the vanilla campfire recipe is removed. **Fire starter** = 2 sticks + string +
shard. A fire starter used on 1 log + 3 kindling makes a campfire
(`fireStarterCanMakeCampfire`). Each try has 30% to light (`fireStarterFireStartChance = 0.3`).

**Then:** clay tool shapes clay blocks into vessels, buckets and pots, fired in the campfire.
Saws (iron and up) make planks. Mattocks.

**Steps to flint axe:** about 7. **Time:** est. 3-5 min experienced, 10-20 min new (the new
player does not know to knap on stone or to cut grass with the knife; no in-game book, only
advancements and recipe viewers).

**Praised:** "a little more believability to your first few minutes of play" (mod page). It
is light: it is short and ends at the flint tier. Many packs use it (SevTech-style packs,
realism packs).

**Complained about:** players forked it to cut extras. "No Tree Punching: No Additional
Features", AntiTree and others say they wanted only the log rule, without the ceramics and
rock crafting (Modpack Index listings). The chance-based knapping invites spam-clicking; a
fork (Yes Flint Knapping) adds two separate chances for pack makers to tune. AntiTree changes
it so one click on stone always gives 2 shards.

---

## 3. "Primal" mods and stone-age modpacks

**PrimalCore (An_Sar, 1.11-1.12):** wooden and stone tools are swapped for **flaked flint**;
logs need a tool. Knap flint on a hard block (stone, obsidian) to get flaked flint and flint
points. Tall grass gives **plant fibres**, made into **plant cordage/twine**, which binds flint
heads to sticks. Punching a log hurts you and drops nothing. Fire comes from a **fire bow**
that uses up sticks. Every feature can be turned off in config. (9minecraft wiki mirrors,
which disagree in places; CurseForge listing)

**Primal Winter (AlcatrazEscapee):** not a primitive-start mod. It makes the whole world a
frozen wasteland with fog and blizzards. It is often packed with NTP or TFC for a harsh start.
(CurseForge)

**SevTech: Ages, Age 0** (uses Primal Tech and similar mods). Steps from a hosting guide:
1. Rocks and sticks from the ground; break grass for plant fibres.
2. 3 fibres → **plant twine**.
3. A rock → **crafting rock**.
4. Flint: a **grass fibre mesh** (2 sticks + 2 twine) sifts gravel, because gravel drops no
   flint in this pack.
5. Hit flint on stone → flaked flint, again → flint point.
6. **Flint hatchet** = flaked flint + stick + twine. Now logs drop.
7. A log in the grid becomes a **chopping block**: put a log on it and chop with the hatchet
   to get planks.
8. Chopping block + log → **work stump**. You fill its slots, then hit it with the crafting rock.
9. **Fire sticks** = 2 sticks, one in each hand, right-click.
(holy.gg SevTech guide). Progress is shown in the advancement menu (L). A player tips list
says the hatchet is "strictly a gateway" to the stump, and to upgrade to a stone axe fast;
the flint workblade cuts shrubs and leaves for twine and sticks. (borg286/sevtech2 on GitHub)
The SevTech wiki's first line: players punch trees on instinct, and it does not work.
**Steps to first axe:** about 9. **Time:** est. 10-30 min new.

**GregTech: New Horizons:** the stone age is long and hard; guides say you "barely" survive
until steam machines. Flint tools last longer but work slower than stone. No good public
source on its exact opening; the in-game quest book is the guide.

---

## 4. Vintage Story

**On the ground:** loose stones, **flint**, sticks. Sticks also come from hitting leafy
branches. (Official wiki, "Your first day")

**Punching:** felling trees needs an axe; the axe is "the primary tool used to break
wood-type blocks". No source says logs drop from a bare hand. (Wiki "Axe")

**First tool, step by step:**
1. Pick up flint (or a knappable stone: basalt, chert, granite, andesite, obsidian,
   peridotite) and a stick.
2. Shift+right-click solid ground with the flint. "Only flint will immediately create a
   knapping surface"; other stones need a second loose stone. (Wiki)
3. Pick a recipe from a menu. A **10x10 voxel grid** appears with the shape highlighted. Click
   away voxels until none are highlighted. The game stops you removing voxels you need;
   chunks that come loose break off. (Wiki "Knapping")
4. Craft head + stick (or bone) in the grid.
- Heads: axe, knife, shovel, hoe, spear; arrowheads only from flint. Obsidian > flint > other
  stone for durability. **No stone pickaxe**; the first pickaxe is copper.
- The wiki says to make an axe and a knife first.

**First binding:** none for tools. Reeds/cattails (cut with the knife) make a hand basket
(10) and reed chests (24). (Wiki)

**First fire:**
1. Knife on tall grass → **dry grass**.
2. Axe + log in the grid → firewood (4 needed).
3. Sneak+right-click the ground with dry grass to place the pit; add 4 firewood.
4. **Firestarter** = 1 dry grass + 2 sticks; each try wears it down. Hold right-click on the
   pit. (Wiki "Firepit", "Firestarter")
- Rain puts out uncovered fires. Heating sticks gives torches. Clay must be fired in a pit kiln.

**Then:** clay → pit kiln → copper from surface nuggets, panning, or ruins. Day 1 goal is
shelter, because monsters spawn at dusk. A default day is about 24 real minutes (server
defaults: time speed 60, 24 hours per day).

**Steps to stone axe:** about 4-5. **Time:** est. 2-4 min experienced, 10-20 min new.
**Copper:** an experienced player says copper "almost always" turns up "on the first day, if
not on the second" (in-game days); a forum reply says a new player "will probably be stuck at
copper for a while"; one player spent 4 hours panning for enough copper for basic tools
(VS forums). A long-form writer took "something like 80 hours" from stone tools to iron
(The Mane Quest, 2026).

**Praised:** knapping is a "first real crafting system" where you shape tools by breaking
flakes (supercraft review). Each new material brings new processes, so progress stays
satisfying (The Mane Quest). The in-game handbook exists.

**Complained about:** a new player: "I'm not feeling any fun yet", spent about an hour
without progress after a bad spawn, and disliked needing the handbook for every step
(VS forum, "Survival or bureaucracy?"). "A tedious housekeeping simulator where progress is
measured in millimeters" (forum thread title). Defenders reply that the stone age is a
tutorial, not a long stage. The handbook is "overwhelmingly voluminous" (The Mane Quest). A
player thought knapped axe heads vanished; they went into a hand basket (forum: "I cannot
knap an axe head"), so feedback on where output goes matters.

---

## 5. Valheim

**On the ground:** branches (each gives Wood) and small grey stones, both picked up with E.
Flint lies on shores and river banks; it looks like whitish stone. (allthings.how; GamesRadar)

**Punching:** guides say to pick up branches and to "ignore the full-size trees for now";
fists cannot fell trees. (allthings.how)

**First tool:**
1. Pick up a branch and a stone. Picking both up **unlocks** the recipe and shows "New crafting
   recipe". (allthings.how)
2. **Stone axe** = 5 Wood + 4 Stone, crafted from the inventory, no bench.
- **Flint axe** = 4 Wood + 6 Flint (better, needs a workbench).
- **No stone pickaxe.** The first pickaxe is antler, from the first boss's trophy.

**First binding:** none.

**First fire:** **Hammer** (3 Wood + 2 Stone) → build menu → **campfire** (5 Stone + 2 Wood).
Fires cannot go on wooden floors. (GamesRadar; gamertweak)

**Guide:** Hugin the raven appears at milestones with short hints. Reviewers like that it
stays in the theme instead of a dull tutorial; players say he "wasn't really interrupting".
He says to collect wood and stone but not how (search summary of reviews).

**Steps to stone axe:** 3. **Time:** est. 1-3 min for anyone. Speedruns kill the first boss
in about 5-6 min of a new game (speedrun.com).

**Praised:** fast, visible start; recipes unlock as you touch materials; the raven.
**Complained about:** the opening is rarely the complaint; later grind is.

---

## 6. Green Hell

**On the ground:** small stones (by boulders, in caves, on river banks), sticks, small sticks,
dry leaves, lianas hanging from big trees. (corrosionhour)

**First tool:**
- **Stone blade** = 2 small stones. "Only takes a second." (corrosionhour)
- **Basic stone axe** = 1 stick + 1 stone (several guides).
- **Stronger stone axe** = stick + stone blade + **rope**; sturdier (guides disagree on which
  recipes exist; check the game's own list).
- Crafting is done by dragging items onto a crafting area or the backpack; recipes are kept
  in a **notebook**, and some unlock by crafting skill.

**First binding:** **rope from lianas** (pull vines off big trees). Real-world and immediate.

**First fire:**
1. Small fire = 8 sticks + 6 small sticks, placed from the notebook.
2. **Hand drill** = 1 stick + 1 small stick.
3. Use the drill at the fire, drag **tinder** (bird's nest best; dry leaves, dry palm leaves,
   dry grass) into the circle. It costs **stamina** (about half the bar early) and gets faster
   as the Fire Starting skill grows. Doesn't work in rain.
4. Carry the ember to the fire quickly. (corrosionhour; Steam discussions)

**Steps to stone axe:** 2-4. **Time:** est. 1-5 min.

**Praised:** each tool feels physical and real; rope from vines and tinder by type are
believable; the skill grows with use.
**Complained about:** inventory/backpack friction (items collide, accidental crafting), crafting
one arrow at a time is tedious, "the tutorial doesn't help you at all", expect to die a lot
early, "harsh survival mechanics and a poor tutorial" (TechRaptor, Finger Guns, Vaporlens
summary).

---

## 7. Stranded Deep

**Start:** you wake on a **life raft** near an island. Rocks ("medium-sized rocks that look
like big pebbles"), sticks, palm and yucca trees.

**First tool and binding:**
1. A rock → **stone tool**.
2. Hit a **yucca** (or palm sapling) with it → **fibrous leaves** "after a few hits"; one guide
   says about 10 chops.
3. 4 fibrous leaves → **lashing**.
4. **Crude axe** = stick + rock + lashing. (TheGamer; Gfinity; community wiki index)
- The stone tool lasts about 100 hits. Trees do not grow back.

**First fire:** **campfire = 4 sticks**; craft **kindling**; at the campfire, use the kindling
by **alternating left and right mouse buttons** (friction). There is no lighter. Add 3 rocks
around it for a fire pit. (The Nerd Stash)

**Steps to crude axe:** about 4. **Time:** est. 3-10 min.

**Praised:** a short, believable chain: rock, fibre, lashing, axe. The friction minigame for fire.
**Complained about:** recipes changed across versions and guides contradict each other;
limited trees.

---

## 8. The Forest / Sons of the Forest

- **The Forest:** the **plane axe** lies in the crashed plane's aisle; you take it before
  leaving. Sticks, rocks and logs are the base materials. Crafting is done by laying items on a
  mat in the backpack (I) with a survival book (B) for recipes. A "crafted axe" is stick +
  rock (+ rope in some versions). Fire: a small fire is sticks plus the starting lighter; a
  standing fire is 3 sticks + 3 rocks. (Prima Games; Corrosion Hour)
- **Sons of the Forest:** an emergency pack gives a **tactical axe**, knife, lighter, GPS and a
  guide book. Small trees and shrubs give sticks; thick trunks give logs. (PC Gamer; wiki)
- **Lesson:** no "earned" start. The axe is free so the game can start on building and fear.

---

## 9. Rust

- You spawn with a **rock** and a **torch** (some guides add bandages). The rock hits trees
  and stone nodes slowly. **Stone hatchet** and **stone pickaxe** cost about 200 wood + 100
  stone each; no bench needed. Hemp → cloth → sleeping bag. (PC Gamer; Corrosion Hour)
- **Time to stone hatchet:** est. 2-4 min.
- **Lesson:** "rock in hand" is a believable, never-blocked start: slow, but always works.

## 10. Don't Starve

- **Flint** lies on the ground; **saplings** give twigs; **grass tufts** give cut grass. Pick by
  hand.
- **Axe** = 1 flint + 1 twig. Pickaxe = 2 flint + 2 twigs. Torch = 2 cut grass + 1 twig.
  **Campfire** = 2 logs + 3 cut grass. (Don't Starve wiki)
- **Time to axe:** under a minute.
- **Lesson:** the iconic "flint, twigs, grass" start. Everything visible, three steps, then
  danger arrives at night.

## 11. Others, briefly

- **Ancestors: The Humankind Odyssey:** you discover tool use by bashing rocks and sharpening
  sticks, with almost no hints. Critics: "an absolute headache to play for the first few
  hours", "a tedious exercise in trial-and-error" (Niche Gamer), "an awkward and repetitive
  survival slog" (PC Gamer). Shows the cost of hiding every recipe.
- **Muck:** the tutorial says: get a rock, chop a tree with it, make a workbench. Bark (from
  wood) goes into wood axe and pickaxe. Short and clear. (GamePretty)
- **Minecraft (baseline):** punch a log, planks, table, sticks, wooden pickaxe, cobble, stone
  axe. About 6 steps; top runners craft table + wooden pickaxe in about 13 s (minecraftbible
  guide, low-authority). New players are misled by the recipe book's previews (Minecraft
  feedback site post by someone who watched first-time players aged 20-30).
- **Subsistence, Dawn of Man:** no useful first-tool material found; skipped.

---

## 12. Design research on onboarding, pacing and "earned" effort

- **Cognitive load (Celia Hodent, GDC 2016, "The Gamer's Brain, Part 2"):** "3 items to
  process at the same time is THE MAXIMUM when in learning mode." Teach by doing, give each
  skill a purpose, space teaching over the first hour, do not punish while players learn, and
  "show the locks before giving the keys".
- **Opening's job (GDKeys, "How to Onboard Players"):** show the role, stakes, goal, core
  pillars and tone early. God of War's opening tree-cutting teaches the axe without combat.
  Criticises openings that delay real play for about 20 minutes.
- **Early wins (Yu-kai Chou, onboarding phase):** give quick "win-states" and one highlighted
  next action.
- **Motivation (Ryan, Rigby & Przybylski 2006, *Motivation and Emotion*):** games pull people
  through feelings of **competence**, autonomy and relatedness; competence ties to enjoyment
  and to controls feeling intuitive.
- **IKEA effect (Norton, Mochon & Ariely, *Journal of Consumer Psychology*, 2012):** people
  who built the item valued it far more, but "labor leads to love only when labor results in
  successful completion"; unfinished or destroyed work lost the effect. For us: effort must
  end in a tool the player holds, not in failure or waste.
- **Busywork:** "Gathering resources is the single least engaging grind in gaming" (AV Club).
  Crafting with "no real decisions" is "tedious busywork" (bugnet crafting article).
  Grind is tolerated more when players control its pace (Wayline).
- **Player data from the forums above:** complaints are about **long empty stretches**
  (finding copper, food, the first winter), **having to read a book for every step**, and
  **unclear feedback** (where did my axe head go?). Short physical steps like knapping are
  rarely the complaint.

---

## 13. Comparison table

| Game | First tool | Steps to stone/flint axe | Time to stone axe (new / experienced) | What makes it feel good | What players complain about |
|---|---|---|---|---|---|
| Minecraft (baseline) | wooden pickaxe (punched log) | ~6 | ~3-5 min / under 1 min (est.) | instant, no rules | none at start; recipe book misleads new players |
| TFC 1.18/1.20 | stone knife or axe (knapped) | ~5 | 5-15 min / 1-3 min (est.) | rocks underfoot set the tool; 5x5 knapping is quick; field guide in game | slow to copper and beyond; ore hunting; grind at fixed points |
| TFC classic / TFC+ | stone axe (knapped) | ~5 | similar (est.) | same | same; firestarter needs many tries |
| No Tree Punching | flint knife, then flint axe | ~7 | 10-20 min / 3-5 min (est.) | short; ends at flint; believable | chance-based knapping and 25% fibre feel random; extra content not wanted (forks) |
| PrimalCore | flaked flint hatchet | ~6 | 10-20 min / 5 min (est.) | flint, cordage, fire bow: a full chain | config-heavy, 1.12 only |
| SevTech Ages, Age 0 | flint hatchet | ~9 | 15-30 min / 5-10 min (est.) | every step physical (mesh, stump, fire sticks) | long chain; must read advancements; hatchet only a gateway |
| Vintage Story | flint knife or axe (knapped) | ~5 | 10-20 min / 2-4 min (est.) | 10x10 voxel knapping is tactile; handbook | handbook for every step; copper can take hours; "housekeeping simulator" |
| Valheim | stone axe | 3 | 1-3 min / 1 min (est.) | pick up two things → recipe unlocks; raven hints | not the opening |
| Green Hell | stone blade / stone axe | 2-4 | 2-5 min / 1 min (est.) | rope from lianas, tinder by type, skill grows | inventory clunky; poor tutorial; die a lot early |
| Stranded Deep | stone tool, then crude axe | ~4 | 5-10 min / 3 min (est.) | rock → fibre → lashing → axe is clear and real | recipes changed; guides disagree; trees don't regrow |
| The Forest / SotF | axe given (plane / pack) | 0 | 0 | starts on building, not survival chores | n/a for opening |
| Rust | rock given; stone hatchet | 2 | 3-5 min / 2-3 min (est.) | never blocked; rock always works | slow; PvP pressure |
| Don't Starve | flint axe | 3 | under 1 min | all on the ground, visible | night comes fast |
| Ancestors | sharpened stick / rock tool | unclear | long / short | discovery when it works | hidden recipes; trial and error; "slog" |

---

## 14. What makes "earned" feel good rather than tedious

1. **One new idea per step.** Each step teaches one thing (pick up, knap, cut fibre, twist cord,
   haft). Hodent: no more than 3 new things at once. TFC and Valheim stay short this way;
   SevTech's 9-step chain feels like a checklist.
2. **The materials are visible and plentiful near spawn.** Rocks, sticks, flint and grass you
   can see from where you stand (Don't Starve, TFC, Vintage Story). Searching for the first
   ingredient is dead time, not effort.
3. **Every click visibly changes something.** A knapping grid shows the head taking shape
   (TFC 5x5, about 8 clicks; VS 10x10). Hidden dice rolls with no visible progress (NTP's 28%
   per click, 25% fibre) feel like waiting, not working.
4. **Failure that teaches, never failure that deletes.** The IKEA effect vanishes when work
   is destroyed. VS stops you removing a needed voxel; TFC lets you plan; a failed fire try
   should show why (no kindling, wet) and keep the materials.
5. **The next step is always hinted.** Valheim unlocks a recipe when you touch the
   ingredients and the raven nudges. A hint chain ("you could bind this with something
   stringy") beats a handbook the player must read for every step (VS, Ancestors complaints).
6. **No hidden recipes for the core path.** Ancestors and Green Hell's notebook gaps drew the
   most "trial and error" complaints. Hide extras, never the axe.
7. **Short repeated actions, each a few seconds.** Pulling a vine (Green Hell), chopping yucca
   about 10 times (Stranded Deep), knapping about 8 clicks: fine. Holding a button for a minute
   or 12 grass-breaks per string starts to feel like grind.
8. **Each step pays off at once.** The knife immediately cuts grass; the cord immediately
   makes the axe; the axe immediately fells a tree; the fire immediately cooks and lights.
   Players praise TFC and VS for each material bringing "new processes and benefits".
9. **Keep the whole stone start short: about 5-7 steps and 5-15 minutes to the axe for a new
   player.** Valheim and Green Hell need 2-4 steps; TFC and VS about 5; SevTech 9. Defenders of
   VS call the stone age "a tutorial", and the long complaints are all about what comes after
   (copper, food, winter). The long road belongs later, not before the first axe.
10. **Make it physical and believable, and give clear feedback on output.** Fire that can fail
    (TFC 50% + 10% per kindling, Green Hell stamina) feels earned when the odds are shown and
    kindling helps. Tools should land in the hand or a visible slot (the VS "vanished axe
    head" thread). A believable rule (no punching logs, but leaves and grass break by hand)
    is accepted when the way around it is in reach within a minute.

---

## Sources

TerraFirmaCraft
- TFC source, field guide entries (introduction, firepit, pit kiln), branch 1.20.x: https://github.com/TerraFirmaCraft/TerraFirmaCraft/tree/1.20.x/src/main/resources/assets/tfc/patchouli_books/field_guide/en_us/entries/getting_started
- TFC source, log block and config: https://github.com/TerraFirmaCraft/TerraFirmaCraft/blob/1.20.x/src/main/java/net/dries007/tfc/common/blocks/wood/Wood.java , https://github.com/TerraFirmaCraft/TerraFirmaCraft/blob/1.20.x/src/main/java/net/dries007/tfc/config/ServerConfig.java
- TFC rock knapping data: https://github.com/TerraFirmaCraft/TerraFirmaCraft/blob/1.20.x/src/main/resources/data/tfc/tfc/knapping_types/rock.json
- holy.gg TFC server survival guide: https://www.holy.gg/en/post/terrafirmacraft-server-survival-guide
- All The Mods, Gravitas² beginnings: https://allthemods.github.io/alltheguides/gravitas2/beginings
- FTB wiki, Getting Started (TerraFirmaPunk): https://ftbwiki.org/Tutorial:Getting_Started_(TerraFirmaPunk)
- FTB forum, "TerraFirmaCraft opinions?": https://forum.feed-the-beast.com/threads/terrafirmacraft-opinions.51111/
- TFC forum, "This mod needs so much grinding or is it just me?": https://terrafirmacraft.com/f/topic/8677-this-mod-needs-so-much-grinding-or-is-it-just-me/
- Cereal TerraFirmaCraft modpack: https://www.modpackindex.com/modpack/51340/cereal-terrafirmacraft
- Moddex TFC review: https://moddex.gg/mod/terrafirmacraft/reviews/3391
- TFC+ bids (TFC+ firepit): https://github.com/unforbidable/tfcplus-bids

No Tree Punching and Primal mods
- NTP source (recipes, loot tables, Config.java, EventHandler.java), branch 1.20.x: https://github.com/alcatrazEscapee/no-tree-punching/tree/1.20.x
- NTP wiki home: https://github.com/alcatrazEscapee/no-tree-punching/wiki
- NTP on CurseForge: https://www.curseforge.com/minecraft/mc-mods/no-tree-punching
- 9minecraft NTP wiki: https://wiki.9minecraft.net/no-tree-punching-mod/
- Yes Flint Knapping: https://www.curseforge.com/minecraft/mc-mods/yes-flint-knapping
- AntiTree: https://www.modpackindex.com/mod/172819/antitree
- NTP: No Additional Features: https://www.modpackindex.com/mod/40464/no-tree-punching-no-additional-features
- PrimalCore wiki: https://www.9minecraft.net/primalcore-mod/wiki/
- Primal Winter: https://www.curseforge.com/minecraft/mc-mods/primal-winter
- SevTech Age 0 guide (holy.gg): https://www.holy.gg/de/post/sevtech-ages-server-setup-guide
- SevTech tips (borg286): https://github.com/borg286/sevtech2
- SevTech wiki how-to: https://sevtechages.gamepedia.com/How_to_play_guide_for_SevTech:_Ages

Vintage Story
- Wiki, Survival Guide: Your first day: https://wiki.vintagestory.at/Survival_Guide_-_Your_first_day
- Wiki, Knapping: https://wiki.vintagestory.at/index.php/Knapping
- Wiki, Firepit: https://wiki.vintagestory.at/Firepit ; Firestarter: https://wiki.vintagestory.at/index.php/Firestarter ; Axe: https://wiki.vintagestory.at/Special:MyLanguage/Axe
- Wiki, time commands (day length): https://wiki.vintagestory.at/index.php/List_of_server_commands/time
- Forum, "Survival or bureaucracy?": https://www.vintagestory.at/forums/topic/17089-survival-or-bureaucracy-new-player-struggling-to-find-the-fun-in-vintage-story-am-i-missing-something/
- Forum, "tedious housekeeping simulator": https://www.vintagestory.at/forums/topic/22218-warning-this-is-a-tedious-housekeeping-simulator-where-progress-is-measured-in-millimeters-and-the-world-is-dead/
- Forum, "I cannot knap an axe head": https://www.vintagestory.at/forums/topic/13204-i-cannot-knap-an-axe-head
- Forum, "Progression" (copper timing): https://www.vintagestory.at/forums/topic/17702-progression
- The Mane Quest, "Not Quite A Review": https://www.themanequest.com/blog/2026/8/4/not-quite-a-review-i-need-to-talk-about-vintage-story
- BisectHosting VS day 1: https://bisecthosting.com/blog/vintage-story-beginners-guide-day-1-tools-food
- Supercraft VS review: https://supercraft.host/article/vintage-story-review-hardcore-survival/

Valheim
- allthings.how, stone axe in the Meadows: https://allthings.how/valheim-how-to-make-a-stone-axe-in-the-meadows/
- GamesRadar, flint: https://gamesradar.com/valheim-flint-how-to-get
- GamesRadar, beginner tips (campfire, hammer): https://gamesradar.com/valheim-tips-beginners-guide
- gamertweak recipe list: https://gamertweak.com/valheim-crafting-recipes-list/
- speedrun.com Valheim: https://speedrun.com/valheim
- Hugin coverage: https://gamingtrend.com/the-afterlife-is-brutal-but-rewarding-valheim-first-impressions/

Green Hell
- Corrosion Hour, stone blade: https://www.corrosionhour.com/green-hell-how-to-craft-a-stone-blade/
- Corrosion Hour, fire: https://www.corrosionhour.com/green-hell-how-to-start-a-fire/
- Corrosion Hour, crafting guide: https://www.corrosionhour.com/green-hell-crafting-guide/
- Steam discussion (axe recipes): https://steamcommunity.com/app/815370/discussions/0/3814032755160356102
- TechRaptor review: https://techraptor.net/gaming/reviews/green-hell-review-scouting-didnt-prepare-me
- Finger Guns review: https://fingerguns.net/games/2021/06/14/green-hell-review-ps4-jungle-fevers-lots-of-them/
- Vaporlens summary: https://vaporlens.app/app/815370/green_hell.md

Stranded Deep
- TheGamer, lashing: https://www.thegamer.com/stranded-deep-how-to-craft-lashing/
- Gfinity, lashing: https://www.gfinityesports.com/stranded-deep/How-to-craft-Lashing-in-Stranded-Deep/
- The Nerd Stash, campfire: https://thenerdstash.com/how-to-make-a-campfire-in-stranded-deep
- GamePretty, getting started: https://gamepretty.com/stranded-deep-how-to-get-started-beginners-guide/

The Forest, Sons of the Forest, Rust, Don't Starve, others
- Prima Games, The Forest first night: https://primagames.com/tips/forest-tips-survive-your-first-night
- Corrosion Hour, The Forest beginner's guide: https://www.corrosionhour.com/the-forest-beginners-guide/
- PC Gamer, Sons of the Forest tips: https://www.pcgamer.com/sons-of-the-forest-tips-how-to-survive-your-first-hours-on-the-terrifying-island
- PC Gamer, Rust first day: https://pcgamer.com/how-to-survive-your-first-day-in-rust
- Corrosion Hour, Rust getting started: https://www.corrosionhour.com/guide-to-getting-started-in-rust/
- Don't Starve wiki, Starting Out: https://dontstarve.wiki.gg/wiki/Guides/Starting_Out:_A_Guide_For_Newbies
- Ancestors reviews: https://nichegamer.com/reviews/ancestors-the-humankind-odyssey-review , https://pcgamer.com/ancestors-the-humankind-odyssey-review , https://gamecritics.com/brad-gallaway/this-is-not-a-review-ancestors-the-humankind-odyssey/
- Muck beginner guide: https://gamepretty.com/muck-beginners-guide-how-to-survive/
- Minecraft feedback, recipe clarity for new players: https://feedback.minecraft.net/hc/en-us/community/posts/40457055432333-Improved-Recipe-Clarity-for-New-Players
- Minecraft speedrun basics: https://minecraftbible.com/guides/speedrun-basics

Design research
- Celia Hodent, The Gamer's Brain Part 2 (GDC16): https://celiahodent.com/gamers-brain-ux-onboarding/
- GDKeys, How to Onboard Players: https://gdkeys.com/onboarding-players/
- Yu-kai Chou, onboarding phase: https://yukaichou.com/gamification-study/4-experience-phases-gamification-2-onboarding-phase/
- Ryan, Rigby & Przybylski 2006: https://selfdeterminationtheory.org/player-experience-of-needs-satisfaction-pens/
- IKEA effect: https://en.wikipedia.org/wiki/IKEA_effect
- AV Club, why survival games are a bore: https://www.avclub.com/heres-why-survival-games-are-a-total-bore
- bugnet, crafting that isn't tedious: https://bugnet.io/blog/how-to-design-a-crafting-system-that-isnt-tedious
- Wayline, making the grind feel good: https://www.wayline.io/blog/make-the-grind-feel-good-designing-engaging-progression-systems
