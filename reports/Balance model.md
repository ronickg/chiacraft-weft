# Balance model: one method for every system, with combat worked out

Researcher, 2026-11-05, for Planner and Ronald. Builds on `notes/PROGRESSION.md` ("Endless gear" and "How it stays balanced when things are added"), `reports/Progression systems.md`, `notes/ANIMALS.md`, `mods/animals/SPECIES.md` and `mods/base/vitals/SHEET.md`. Nothing here is built. Numbers marked "proposed" need Ronald's yes.

Ronald asked: "how do we make sure everything is sort of balanced between everything? So like that you can't just instantly kill dinosaurs when you're like super high tier, or at least not the big ones." Then he widened it: "every system needs to be well balanced, but I'm not sure how one does that." And: "it should be easy to add more content, and it sort of balances it decently well. Of course, we will need to be able to tweak certain stuff."

The answer, in one paragraph: price everything in one shared currency, give each system a budget in that currency, derive every number from a few curves instead of typing it by hand, allow hand tweaks only as bounded multipliers with a reason, and run a simulator in the tests that fails when anything leaves its band. Then let Tester's runs check that the simulator matches the game. Combat is the first and hardest case, so most of the numbers below are for combat. The same method then covers skills, loot, tames, magic, machines and exploring.

## The short version

1. **One currency:** effort minutes (EM), meaning minutes of play an average player spends to get something. Every reward, item, tame, perk point, spell, boss drop and machine has an EM price.
2. **One power curve:** power grows ×1.6 per age, smoothly by area level (`g(L) = 1.6^(L/10)`, L from 0 to 69). Gear, creatures, spells and tames all read it.
3. **One danger number per creature:** its area level, plus its size class (small, medium, large, huge, titan) and role. Its health, armour and damage come from those through time-to-kill (TTK) targets.
4. **Big creatures stay big:** against a large, huge or titan creature, your gear counts as at most 2, 1 or ½ ages above it. A damage-rate floor stops the rest. A titan takes at least about 2½ minutes whatever you wear. Small ones still die in one hit.
5. **Content declares, curves derive:** a new creature states its mass, role, home area and rarity. A new item states its tier, kind and material. A tweak is a multiplier from ×0.75 to ×1.25 with a reason, and the checks still apply after it.
6. **Checks:** a combat simulator fills a TTK matrix of every gear tier against every creature and fails outside the bands. Path parity, loot per hour, build dominance and the economy have checks of their own. All of them write sheets the way `vitals/SHEET.md` is written.

## Part 1. How games balance a whole system

### 1.1 One currency of value

You can only compare a perk, a sword and a tame if they're priced in the same unit. Ian Schreiber's *Game Balance Concepts* course calls this the cost curve: put every cost and every benefit into one resource, then require benefit = cost along a curve ([Level 3: Transitive Mechanics and Cost Curves](https://gamebalanceconcepts.wordpress.com/2010/07/21/level-3-transitive-mechanics-and-cost-curves/); a summary is [here](https://bowling-bash.blogspot.com/2011/03/literature-game-balance-ch3-transitive.html), and the book is Schreiber and Romero's [*Game Balance*](https://www.routledge.com/Game-Balance/Schreiber-Romero/p/book/9781498799577)). An increasing curve favours the late game, and a decreasing one favours the early game.

- **Magic: the Gathering** prices every card in mana, and cards are judged against what that mana usually buys. A separate Play Design team is responsible for power level and playtests a year ahead in the "Future Future League" ([mtg.wiki, Play Design](https://mtg.wiki/page/Play_Design), [FFL](https://mtg.wiki/page/FFL), [Play Design lessons learned](https://magic.wizards.com/en/news/feature/play-design-lessons-learned-2019-11-18)).
- **World of Warcraft** gives each item a stat budget from its item level and slot. Stats spend that budget, so a new item needs only a level and a slot ([itemization formulas](https://wow.allakhazam.com/wiki/Itemization_Formulas_(WoW)), [Item level](https://warcraft.wiki.gg/wiki/Item_level)).

For a survival game the natural currency is time. Players feel hours, not points. chiacraft already has a time budget per age (6, 12, 16, 18, 20, 18 and 16 hours, `PROGRESSION.md`). So the currency is **effort minutes**, with a second, derived unit for strength: **power**, which is the age curve.

### 1.2 Budgets per system

Once a currency exists, each system gets a budget: how much power it may add, and at what effort price.
- **Items:** WoW's item budgets (above). Diablo III's Loot 2.0 narrowed how far stats could swing, and dropped fewer but better items ([Diablo wiki](https://di.diablowiki.net/Loot_2.0)). Josh Mosqueira's GDC 2015 talk ["Against the Burning Hells"](https://gdcvault.com/play/1021776/Against-the-Burning-Hells-Diablo) tells why.
- **Skill trees:** Path of Exile prices its passive tree in points and walking distance. A notable costs its own point plus the small nodes on the way, and a keystone pays for its power with a drawback. Chris Wilson's GDC 2019 talk ["Designing Path of Exile to Be Played Forever"](https://gdcvault.com/play/1025784/Designing-Path-of-Exile-to) covers the long-term plan behind it.
- **Bonuses that stack:** Path of Exile adds "increased" bonuses together and multiplies "more" bonuses. Additive pools inside a category stop a tree from compounding itself. The Skyrim loops in `reports/Progression systems.md` show what happens without that rule.

**Checking for dominant builds.** Slay the Spire's team logged every run and watched two numbers per card: how often players pick it when offered, and how often it's in winning decks ([GDC 2019, "Metrics Driven Design and Balance"](https://www.gdcvault.com/browse/gdc-19/play/1025731); [Game Developer write-up](https://www.gamedeveloper.com/design/learn-i-slay-the-spire-i-s-metrics-driven-approach-to-game-balancing-at-gdc-2019)). Before there are players, a simulator can do the same thing. "Evolving the Hearthstone Meta" used simulated games and an evolutionary search to find small card changes that pulled deck win rates toward 50% ([arXiv 1907.01623](https://arxiv.org/abs/1907.01623)). For a skill web, that means: generate many builds with a hill-climber, score each one on the jobs the game asks for (combat TTK, taming time, crafting hours, travel), and flag any build that beats every other one on everything, or any node that nearly every optimised build takes.

### 1.3 Feedback loops

Marc LeBlanc's GDC 1999 talk on feedback systems is the standard frame ([Wikipedia](https://en.wikipedia.org/wiki/Marc_LeBlanc), [8kindsoffun](https://www.8kindsoffun.com/)).
- **Positive feedback** amplifies the leader: guns on the back of the cars in a racing game.
- **Negative feedback** pulls the field together: guns on the front.

A survival game's core loop is positive: better gear kills faster, which brings more loot, which buys better gear. That's the fun of progress, but unchecked it snowballs out of an age in an afternoon. The usual fixes are:
- **Ceilings on the reward:** Path of Exile's area level caps the item level of drops ([PoE wiki, Monster](https://www.poewiki.net/wiki/Monster), [monster level](https://pathofexile.fandom.com/wiki/Monster_level)).
- **Rising costs:** repairs, fuel and upkeep.
- **Catch-up for whoever trails:** Palworld gives a Pal 1.5× to 6× experience when it is 1 to 21+ levels behind its player ([Palworld wiki, Level](https://palworld.wiki.gg/wiki/Level)). Bad-luck protection is catch-up for loot.

### 1.4 Path parity

When several routes lead to the same goal, each needs about the same price. League of Legends flags a champion for change by win rate weighted by ban rate, with different signals for average, elite and pro play ([Champion Balance Framework](https://nexus.leagueoflegends.com/en-au/2019/05/dev-champion-balance-framework/), [2020 update](https://www.leagueoflegends.com/en-us/news/dev/dev-balance-framework-update/)). Factorio's developers changed recipes so the copper-to-iron ratio a factory needs stays about the same in normal and expensive modes ([Friday Facts #217](https://cf-www.factorio.com/blog/post/fff-217)). For chiacraft the routes are tech, magic and beasts through each age, so the measure is hours per age key by each route.

### 1.5 Loot, faucets and sinks

Raph Koster describes Star Wars Galaxies as a "faucet-drain economy": money flows in from sources and out through sinks ([Astromech Stats: Economy Stats](https://www.raphkoster.com/games/essays/astromech-stats-economy-stats/)). When the faucets win you get "mudflation", where old items and materials become worthless ([Boing Boing on Koster's essay](https://boingboing.net/2007/01/19/mudflation-inflation.html)). Koster's advice is to watch the economy's statistics closely and tune every sink against every faucet. In a game without trading the same holds for materials and items. Each activity (exploring, bosses, farming, automation) is a faucet. Its **expected value per hour** in the shared currency should be comparable, with a premium for risk. Repairs, consumed crafting materials, offerings, fuel and death drops are the sinks.

### 1.6 Automation against hand work

In Factorio the first assembling machine crafts at 0.5 speed, half the player's hand speed of 1.0, and the later ones reach 0.75 and 1.25 ([Factorio wiki](https://wiki.factorio.com/Assembling_machine_1/fr); [forum figures](https://forums.factorio.com/viewtopic.php?p=386364)). A machine wins anyway because it runs unattended, in parallel and without stopping. That's the shape to copy: the first machine of an age isn't faster than your hands, but it pays back its build cost within a short, known time. After that it wins on scale.

### 1.7 Difficulty bands and old content

Jenova Chen's thesis "Flow in Games" puts the player in a band between anxiety and boredom, and lets the player steer within it through choices rather than hidden adjustment ([jenovachen.com](https://www.jenovachen.com/flowingames/missionstatement.htm)). Games keep old content worth doing in four ways:
- **Opt-in heat:** Hades' Pact of Punishment lets the player add Heat, and pays bounties per weapon at Heat levels ([Hades wiki, Heat](https://hades.fandom.com/wiki/Heat), [RPG Site guide](https://www.rpgsite.net/feature/10287-hades-pact-of-punishment-heat-modifiers-and-how-to-maximize-your-rewards)).
- **Harder forms with unique loot:** Terraria's Expert and Master modes (below) and Ark's boss forms.
- **Downscaling the player to the content:** Guild Wars 2 lowers your level to the zone's. ESO's One Tamriel scales every zone to the player ([UESP, One Tamriel](https://en.m.uesp.net/wiki/Online:One_Tamriel), [Massively OP](https://massivelyop.com/2016/06/19/mmo-mechanics-zonation-and-scaling-mechanics-in-mmorpgs/)). Players argue about how much progress that takes away.
- **Capping the advantage instead of the player:** Destiny 2's contest mode treats players as at most a set power below each encounter, whatever their gear ([Stevivor](https://stevivor.com/news/destiny-2-beyond-light-deep-stone-crypt-contest-mode-detailed-bungie)).

### 1.8 Tools

| Tool | What it does | Who |
|---|---|---|
| Spreadsheets and cost curves | One sheet per system, priced in the currency | Schreiber and Romero; every studio |
| Machinations | Diagrams of sources, drains and converters, run as Monte Carlo simulations ([Dormans, AIIDE](https://ojs.aaai.org/index.php/AIIDE/article/view/12477), [Machinations on Monte Carlo personas](https://machinations.io/articles/balancing-f2p-economies-simulating-player-personas-and-progression-curves-with-machinations)) | Free-to-play economies |
| Monte Carlo loot sims | Thousands of simulated players; report the median and the 90th percentile, not the mean | Diablo, Path of Exile, our crafting sim |
| Agent simulations | Bots play the rules and a search changes numbers toward a target | The Hearthstone paper above |
| Generated sheets and tests | The data writes a sheet, and tests fail when it's stale or out of band | chiacraft's `vitals/SHEET.md` and `SPECIES.md` already do this |
| Telemetry | Logs of where players die, what they pick, and how long things take. Microsoft's user-research lab for Halo 3 tracked where players died down to the square foot ([Wired](https://wirednews.pages.dev/2007/08/ff-halo-2)) | Bungie, Mega Crit, Riot |
| Playtest loops | Short cycles with real players. Hades' long early access reworked content with player feedback ([GameDaily](https://www.gamedaily.biz/?p=588)) | Supergiant; MTG's FFL |

### 1.9 What studios do, in one line each

- **Slay the Spire:** log every run, and watch pick rate and the win rate of decks with the card.
- **Path of Exile:** area level caps loot, rarity mods give monsters a budget, and league resets clear the economy (we can't wipe worlds).
- **Hades:** opt-in difficulty with rewards per band, and an early access feedback loop.
- **Factorio:** ratios kept stable across modes, and machines that win by running unattended.
- **League of Legends:** published thresholds that trigger a change.
- **Magic: the Gathering:** one currency (mana), plus a team that owns power level and tests it a year ahead.
- **World of Warcraft:** item budgets per level and slot. Its level squish was needed because ever-growing numbers became unreadable ([Blizzard Watch](https://blizzardwatch.com/2020/09/24/world-warcraft-need-level-squish/), [Massively OP](https://massivelyop.com/?p=297815)).

## Part 2. Keeping big creatures dangerous: what other games do

| Game | How | What works | What fails |
|---|---|---|---|
| **Monster Hunter** | No character levels: "it's only your gear that you can improve" (Tsujimoto, [GameBanshee](https://gamebanshee.com/k493y)). Hitzones set the share of damage each body part takes per damage type; low values make blades bounce; parts break for rewards and staggers ([Kiranico guide](https://mhworld.kiranico.com/guide/understanding-monster), [Fextralife, resistances](https://monsterhunterrise.wiki.fextralife.com/Elemental+Resistances)) | Big monsters are fights about where you hit. Gear cuts time but never removes the fight | Late gear trivialises the early monsters. That's accepted because they aren't revisited much |
| **Ark** | Random wild levels (up to 150 by default, set by the difficulty), with stats from base × levels × taming effectiveness × imprint ([Creature stats calculation](https://ark.wiki.gg/wiki/Creature_stats_calculation), [difficulty guide](https://xgamingserver.com/docs/ark-survival-evolved/difficulty-levels)) | Every wild animal is an individual. High levels are prizes | Breeding stacks mutations (2 levels each, a counter limit of 20 per side, [Mutations](https://ark.wiki.gg/wiki/Mutations)) into tames that dwarf the wild. Bosses need arena tame caps |
| **Valheim** | Danger by biome, further from the centre; a creature with 1 star has 2× health and 150% damage, and 2 stars 3× and 200%; +30% health per extra nearby player ([Creatures](https://valheim.fandom.com/wiki/Creatures)) | Clear bands: you know the Plains will kill you | Old biomes become trivial, with no reason to go back except materials |
| **Path of Exile** | Area level sets monster level, which sets life, damage and the item level of drops. Magic and rare monsters get mods and drop higher item levels ([Monster](https://www.poewiki.net/wiki/Monster), [monster level](https://pathofexile.fandom.com/wiki/Monster_level)). Armour reduces a hit by A/(A + 5 × damage) in PoE 1 and A/(A + 10 × damage) in PoE 2, capped at 90% ([Armour](https://www.poewiki.net/wiki/Armour), [PoE 2 Armour](https://poe2wiki.net/wiki/Armour)) | One number per place; loot tied to the place | Builds out-scale everything, which only resets and nerfs control |
| **Elden Ring and Dark Souls** | Defence by curve, then damage negation per type (standard, slash, strike, pierce, magic, fire, lightning, holy); poise and stance, with bosses at about 80-120 poise against 15-65 for normal enemies ([Damage types](https://eldenring.wiki.fextralife.com/Damage+Types), [Damage negation](https://eldenring.wiki.gg/wiki/Damage_negation), [Stance](https://eldenring.wiki.gg/wiki/Stance)) | Types and stance make big enemies different, not just bigger | Over-levelling still melts bosses; the game relies on the player not over-levelling |
| **Skyrim and Oblivion** | Enemies and loot scale with the character's level. Skyrim locks a place's level when you first enter it ([UESP, Leveling](https://pt.uesp.net/wiki/Skyrim:Leveling)) | Every place stays relevant | Oblivion's bandits in glass armour ([Steam thread](https://steamcommunity.com/app/22330/discussions/0/5383446733857395241)); levelling crafting first makes you weaker ([Nexus](https://forums.nexusmods.com/topic/6009123-enemy-difficultyscalingai)) |
| **World of Warcraft** | Item budgets by level; a level and item squish when numbers grew too big | Budgets make new items easy | Growth without a ceiling needed a squish, then grew again ([Massively OP](https://massivelyop.com/?p=218228)) |
| **Terraria** | Expert doubles enemy health and damage; Master triples them and adds knockback resistance (Blue Slime: 25, 50, 75 health) ([Expert](https://terraria.wiki.gg/wiki/Expert_Mode), [Master](https://terraria.wiki.gg/wiki/Master_Mode)) | Opt-in bands with unique loot per band | Flat multipliers, the same for small and big enemies |
| **Destiny 2** | Your effective power is capped relative to the activity; enemies above you deal more and take less | The cleanest "over-tier stops helping" rule | Players feel their grind didn't count ([GamesRadar](https://www.gamesradar.com/the-destiny-2-forge-farm-fiasco-is-another-reminder-that-its-power-grind-needs-to-change)) |

**What to take.**
- From Monster Hunter, fights about parts and types.
- From Path of Exile, one level per place that also caps loot.
- From Valheim and Path of Exile, danger set by place.
- From Destiny, a cap on how much over-tiered gear counts, applied only to big creatures so the grind still feels good on small ones.
- From Elden Ring, stance and types.
- From Terraria, opt-in bands.

**What to avoid.**
- Skyrim's scaling with the character.
- Ark's unbounded breeding.
- WoW's growth with no ceiling.

## Part 3. Combat maths

### 3.1 Damage formulas

- **Flat (subtractive):** damage − armour. Easy to read, but it breaks at the edges: armour above the hit makes you immune, and a small hit against heavy armour does nothing. It's useful as a threshold, like Monster Hunter's bounce.
- **Percentage:** damage × (1 − r) with r from gear. It's linear, so stacking to 100% means immunity, and it needs a hard cap.
- **Ratio with diminishing returns:** r = A / (A + K). League of Legends uses damage × 100/(100 + armour), so effective health = health × (1 + armour/100). Each point of armour adds the same effective health, and stacking never reaches immunity ([LoL wiki, Armor](https://wiki.leagueoflegends.com/en-us/Armor), [Life](https://wiki.leagueoflegends.com/en-us/Life)).
- **Ratio against the hit:** r = A / (A + K × hit), Path of Exile's form. Armour stops small hits well and big hits badly. Against a creature this makes small weapons bounce off heavy hide. Against the player it lets a titan's stomp go through armour that shrugs off a fox. It's also scale-free: if armour and hits both grow ×1.6 per age, a same-age fight comes out at the same percentage in every age.
- **Penetration** removes part of the target's armour before the ratio (percent or flat). It's the job of piercing weapons and of later-age tools.
- **Resistances by type** are a percentage per type (fire, cold, shock, poison, ley), with a cap (PoE uses 75%) and weaknesses below zero.

### 3.2 Size and weight

Big animals in fiction shrug off small weapons and don't fly back when hit. Games do it with:
- **knockback resistance** (Terraria's Master mode adds it per mode);
- **poise or stance** (Elden Ring);
- **hitzones and bounces** (Monster Hunter);
- **parts to break and weak points.**

The rule that falls out is that size should change *how* a fight goes, not just how long it lasts.

### 3.3 Effective health and time-to-kill

Effective health (EHP) is health divided by the share of damage that gets through. TTK is EHP divided by the attacker's damage per second (DPS). Balancing by TTK is easier to reason about than balancing by health: "a large dinosaur takes about 30 seconds of fighting" is a design goal, and health is just whatever number delivers it. The mirror number is **hits to down the player**: how many of the creature's hits a same-age player in same-age armour takes before falling.

### 3.4 Why linear growth breaks, and the fix

If each age adds a fixed amount (+10 damage), the early ages are huge jumps and the late ones are nothing, and old gear's relative standing changes all the time. If each age multiplies by a fixed ratio (×1.6), every age feels the same size, and the gap between age N and N+1 is always 1.6×. Ratios keep the shape. Then two things must hold:
- **The tier must dominate the roll:** a perfect item of age N is about a plain item of age N+1. Terraria's best reforge never beats the next tier ([Reforging](https://terraria.wiki.gg/wiki/Reforging)), and our existing rule caps grade plus marks at +60%.
- **Every number needs a ceiling** (`reports/Progression systems.md`, "Power-creep controls"), or the curve becomes WoW's 1,000-item-level problem.

## Part 4. chiacraft's combat model

### 4.1 One number for danger

- **Area level L (0-69):** 10 × the place's age tier, plus a depth of 0-9. It's the same scale as item level (`PROGRESSION.md`). Biome, height, depth underground, distance from spawn, a boss's form and a star chart's depth all set it. The world generator writes it per region (proposed: from climate and terrain, like `land::fields(seed)`), and the spyglass and journal show it.
- **Size class**, from mass, so it's a fact rather than a choice:

  | Class | Mass | Today's examples (`SPECIES.md`) |
  |---|---|---|
  | small | under 10 kg | hare, fox, owls, fish |
  | medium | 10-150 kg | raptor, coyote, wolf, jaguar, protoceratops |
  | large | 150-2,000 kg | red deer, utahraptor, gastonia |
  | huge | 2-20 t | wyvern, iguanodont, cedarosaurus |
  | titan | over 20 t, or any boss | biome bosses, seasonal giants |

  The code's `Size` enum has small, medium and large today, so huge and titan are new.
- **Role**, which shapes the budget without adding to it:

  | Role | Health | Damage | Notes |
  |---|---|---|---|
  | prey | ×0.7 | ×0.5 | flees |
  | grazer | ×1.0 | ×0.8 | defends its herd |
  | hunter | ×0.9 | ×1.2 | |
  | apex | ×1.3 | ×1.3 | priced as half a size class up |
  | tank | ×1.0 | ×0.7 | armour ×1.6, like the ankylosaurs gastonia and pinacosaurus |

- **Wild level (Ark's)** stays as the individual roll inside that band (see 4.9).

So a creature's danger is (L, size, role), plus its roll.

### 4.2 The curve

`g(L) = 1.6^(L/10)`: 1.0 at the start of stone, 2.56 at iron, 16.8 at star, and 24 at L 69. Each area level is about +4.8%. It's the only growth curve in combat. Tier hours and the economy have their own curves (Part 5).

### 4.3 The reference player

The reference player is a player of the age with same-age gear at grade 4 (Sound), no marks, no perks and no food buffs. All targets are measured against them. It's calibrated to today's numbers: the engine counts health in 20 points, and the vitals show 100 (`HEALTH_SCALE = 5` in `mods/base/api`).

| At area level L | Value |
|---|---|
| hit | 6 × g(L) at one hit a second (about a stone sword landing most swings) |
| health | 20 × g(L) engine points: 100 shown at stone, about 256 at iron, about 1,050 in the future age. Above the base 20, it comes from the age's armour and the Defence and Survival web, not from character level |
| armour | 25 × g(L), in the new formula's points (below) |

### 4.4 TTK and hits-to-down targets, and the stats they give

Proposed targets for a same-age reference player:

| Size | TTK band (mid) | Hits to down the player | Creature's hit interval | Creature armour cuts the player's hit by |
|---|---|---|---|---|
| small | under 2 s (1), one or two hits | 25 | 1 s | 0% |
| medium | 3-8 s (5) | 8 | 1.2 s | 10% |
| large | 20-40 s (30) | 4 | 2 s | 25% |
| huge | 60-120 s (90) | 2.5 | 3 s | 35% |
| titan / boss | 120-300 s (210) | 1.5, telegraphed | 4 s | 45% |

From these the stats follow, with no hand numbers:
- **health** = TTK mid × the reference hit × (1 − its armour cut) × role;
- **armour** is solved so the ratio formula gives its cut against the reference hit;
- **its hit** is solved so the reference player falls in that many hits through their own armour.

Computed (engine points; `g` at L 5, 25 and 55):

| Size | L 5 (stone): health / armour / hit | L 25 (iron) | L 55 (future) |
|---|---|---|---|
| small | 8 / 0 / 3 | 19 / 0 / 8 | 80 / 0 / 32 |
| medium | 34 / 4 / 6 | 87 / 11 / 16 | 358 / 44 / 66 |
| large | 171 / 13 / 10 | 437 / 32 / 26 | 1,791 / 133 / 107 |
| huge | 444 / 20 / 15 | 1,137 / 52 / 37 | 4,656 / 214 / 152 |
| titan | 877 / 31 / 22 | 2,244 / 80 / 56 | 9,192 / 326 / 228 |

Against today's table: the raptor has 14 health (34 here), the utahraptor 60 (171 to 273, depending on its area), gastonia 120, the wyvern 400 and cedarosaurus 300 (444 or more as a huge one). The big animals are well under target today: a stone sword (5 damage at 1.6 swings a second) kills a cedarosaurus in under 40 seconds. Ronald's worry is already true for them.

### 4.5 Over-tier and under-tier: the size cap

Two rules, both from size class:
1. **Item-level clamp.** Against a creature at area L, your weapon's and armour's item level counts as at most L + a cap: small unlimited, medium +30, large +20, huge +10, titan +5. It's Destiny's power cap, applied only to big creatures, so the grind still pays everywhere else.
2. **Damage-rate floor.** A large creature loses at most a tenth of its health a second, a huge one 1/45 and a titan 1/120, from all sources together: players, tames, fire, traps. Damage over the rate isn't lost: it fills the stagger meter, which causes knockdowns and part breaks. Over-tier players knock things down more often and break parts faster, which means more part loot, but they don't kill faster. This also caps multiplayer burst.

Plus the fight's texture:
- **Knockback resistance** by size: 0, 0.3, 0.7, 1 and 1.
- **Stagger thresholds** on the curve.
- **Weak points** at ×1.5, and armoured plates at ×0.5 per damage type (Monster Hunter's hitzones).
- **Titans and bosses lock their health** at 66% and 33% until a named part breaks, so the fight's phases can't be skipped.

Under-tier needs no extra rule: the ratio formula already makes a smaller hit go through less armour. TTK (seconds) and hits to down a player, by the player's ages over or under the creature, for a creature at L 25:

| Size | −1 age | same age | +1 | +2 | +3 | +6 |
|---|---|---|---|---|---|---|
| small | 1.6 s / 12 | 1.0 / 25 | 0.6 / 56 | 0.4 / 105 | 0.2 / 168 | 0.1 / 688 |
| medium | 8.5 / 4.1 | 5.0 / 8 | 3.0 / 17 | 1.8 / 37 | 1.1 / 82 | 1.1 / 82 |
| large | 55 / 2.1 | 30 / 4 | 17 / 7.9 | 10 / 16 | 10 / 16 | 10 / 16 |
| huge | 174 / 1.4 | 90 / 2.5 | 49 / 4.7 | 49 / 4.7 | 49 / 4.7 | 49 / 4.7 |
| titan | 427 / 0.9 | 210 / 1.5 | 150 / 2.0 | 150 / 2.0 | 150 / 2.0 | 150 / 2.0 |

So a star-age player still one-shots a stone hare and kills a raptor in a second. A stone utahraptor takes 10 seconds and is no threat. A stone-age huge sauropod takes about 50 seconds and can still drop you in five hits. A stone-age titan boss takes 2½ minutes and two hits flatten you. The other way round, a stone player who walks into an iron-age forest meets large animals that take a minute to kill and down them in two hits. That's a wall, and the right one.

### 4.6 The damage formula (proposed)

It replaces the modern-armour step in `mods/base/src/player.rs`, which uses points out of 25 with toughness. One function in `vitals::combat` runs for both players and creatures:

1. **Raw hit R:** weapon (or creature) base × g(clamped item level) × grade × (1 + additive bonuses within a category) × (more bonuses between categories) × crit.
2. **Type and part:** R × the part's multiplier for the hit's type (cut, pierce, blunt, fire, cold, shock, poison, ley). Weak points ×1.5, plates ×0.5, the default part 1.0.
3. **Penetration:** A_eff = max(0, A × (1 − pen%) − pen_flat). Pierce weapons get 30%, picks and war hammers flat penetration, and later ages' tools more.
4. **Armour:** physical types × (1 − A_eff / (A_eff + 5R)), capped at 80%.
5. **Resistance:** elemental types × (1 − res), with res between −100% and +75%.
6. **The world:** difficulty's `harm` (0.5, 1 or 1.5, already in vitals) on damage to players.
7. **Rate floor and phase locks** on large, huge and titan targets, with the overflow going to stagger.

### 4.7 A worked example across three ages

A large hunter, the utahraptor's shape, met at L 5 (stone), L 25 (iron) and L 55 (future). Health 171, 437 and 1,791; armour 12.6, 32.4 and 132.6.

| Fight | Hit | Armour cut | Damage a hit | TTK |
|---|---|---|---|---|
| Stone player, flint spear (pierce 30%) vs L 5 | 7.6 | 19% | 6.2 | 28 s |
| Iron player, steel sword (cut) vs L 25 | 19.4 | 25% | 14.6 | 30 s |
| Future player, ley blade (pen 20%) vs L 55 | 79.6 | 21% | 62.8 | 28 s |
| Future player vs the stone one (clamped to item level 25) | 19.4 | 9% | 17.6 | 10 s (the floor) |
| Stone player vs the iron one | 7.6 | 37% | 4.8 | 92 s |

Its bite on a same-age player is 12, 31 and 128. After armour that's about 8, 21 and 85, so three bites down you in every age. On a stone player the iron one's bite is 31 against 25 health: one bite. The future player shrugs off the stone one, which is fine for a large creature. A huge or titan one would still hurt (the table above).

### 4.8 Bosses

Bosses are titans with forms. Form k sits at area level 10 × (its first age + k) + 7, so each form is one age up and near the top of its age. Two things about them can be priced from the same budget:
- **Arena rules:** a tame cap (as Ark has) and +50% health per extra player (proposed; Terraria's Expert bosses also gain health per extra player), held in check by the rate floor.
- **Part breaks:** each break adds a guaranteed part to the loot. That makes over-tier play feel rewarding without being fast.

### 4.9 Wild levels and tames

**Wild levels.** Today a wild point adds 20% health and 5% damage, and levels run 1-150 (`SPECIES.md`). A level-150 animal can have about 4.8× the health of a level-1 one, which is nearly two ages of power. There are two options:
- **(A)** Keep Ark's feel and let the sheet show the tail.
- **(B, recommended)** Scale the steps so that a top roll is about one age stronger (×1.6 health and ×1.6 damage over the median). "A level-140 raptor" then means "a raptor from the next age": a known prize, and still inside the checks.

Either way the TTK check runs at the median roll, and the 95th percentile may reach at most twice the band's top.

**Tames** use the same curves. A same-age tame of size s equals a wild one of size s at the same area: the same derived health, armour and hit. A tame's damage goes through the same formula, clamp and rate floor, so a pack of star-age tames can't melt a titan either. Its budget:
- **Tamed points, imprint (+20%, as decided) and bond (+5%) together add at most +60%** over a clean wild tame of its level. That's the same ceiling gear has for grade and marks. Ark's mutations (+2 levels each, counter 20) count inside it, so breeding lines chase a ceiling instead of breaking through it. Today's 75 tamed points at +10% health each could reach ×8.5, so the steps need normalising, as with wild points.
- **Keeping pace across ages.** A tame's band is its birth or taming area. Two ways forward, for Ronald to pick:
  - **(A)** Breeding carries it up: young born in a nest of the next age's materials, fed that age's kibble, are born in that age's band. Old lines stay alive by breeding them on, which suits "Deep, like ARK".
  - **(B)** Saddles and barding of an age raise a grown tame's effective tier by one, like Pokémon badges in reverse. This is close to the rebasing Ronald turned down for gear.
- **Catch-up:** tames below their owner's age get Palworld's extra experience (1.5× to 6×).
- **Parity:** a player plus a same-age tame deals about 1.6-1.8× a solo player's damage. The beasts path pays for it by spending web points in the Beasts region instead of on weapon perks, so the path-parity check (5.4) holds.

## Part 5. How chiacraft balances every system

### 5.1 The currency

- **Effort minutes (EM):** the expected minutes of an average player to get something. That includes gathering at the sheet's rates, travel, crafting time, station time and risk. An age's whole budget is its hours: stone 360 EM, bronze 720, iron 960, steam 1,080, electric 1,200, future 1,080, star 960.
- **Power:** the age curve g(L). An item, tame or spell has a power multiple of the same-age reference.

Every reward has both an EM price and a power value, so a perk, a tame and a sword can be compared on one line of the sheet.

### 5.2 Budgets per system (proposed)

| System | Power budget | Price in EM | Notes |
|---|---|---|---|
| Gear | Base from the curve × its kind's spread (a heavy axe trades speed for damage). Grade + marks at most +60% | A plain same-age weapon is about 2.5% of the age's EM (9 min at stone, 24 at iron). A full armour set about 8% | Spreads are budgets: damage × speed × reach ≈ 1 (WoW's rule) |
| Perk web | A small node is 1 unit: +2% to one family (melee damage, taming speed). A notable is 3-4 units for 1 point plus the walk. A keystone nets 0 (power +X with a cost of about X) | Points come from levels; a level's EM follows the level table in vitals | Bonuses add within a family and multiply between families. A fully specialised region is worth at most ×1.6 in its speciality (one age) |
| Everything together | Gear × web × tame bond × food at most ×2.5 over the reference (two ages), checked | | For large, huge and titan creatures the rate floor still holds |
| Spells | Damage per mana on the same curve. A same-age focus at full mana uptime gives the reference DPS × its kind's spread. Area spells pay √targets out of the budget | Tomes and foci are priced like gear | Spells roll quality like weapons, inside the same +60% |
| Tames | As 4.9 | Taming a same-age large one is about 3-6% of the age's EM, a breeding line 10-25 in-game days (`balance.rs` targets) | |
| Boss drops | A form's loot is at item level 10t + 4-7. Its age-key part is required | A first kill, including offerings, is about 10-15% of the age's EM | Bad-luck help is certain by a set kill count |
| Machines | An age's first machine works at 0.5-1× hand speed, unattended | Pays back its EM within 2 hours of that age; works at 4× or more of hand speed by the end of its age and 10× or more on the previous age's materials | Old materials get cheap on purpose; old content stays worth it for unique drops, below |
| Exploring | Item level from depth; ruins hold blueprints and relics | Expected value per hour within ±30% of farming's, plus a risk premium of up to +50% for expected death loss | Risk is priced: drop-everything deaths are a cost in EM |

### 5.3 Loops, parity, old content

- **Positive loops are bounded by place:** loot from area L can't exceed item level L + 9, so snowballing inside an age stops at the age's ceiling.
- **Negative loops:** repairs, fuel, death drops, offerings and the rate floors.
- **Catch-up:** bad-luck help, Palworld-style tame experience, and the tribe's shared journal (knowledge, not age).
- **Path parity:** each age's key by the tech, magic and beasts routes costs within ±15% EM of the others. The reference fighter of each path (sword, spell, tame) meets the same TTK bands within ±15%.
- **Old content stays worth it:**
  - higher boss forms in old biomes;
  - old-biome materials needed in small amounts later (Terraria's way);
  - seasonal giants and blood moons that use the place's own level;
  - opt-in danger marks on star charts (Hades' Heat).
- **Linked worlds:** when you visit a friend's world, the item-level clamp applies to the whole world (Guild Wars 2's downscaling). That extends the portal admission rule.

### 5.4 Adding content: declare a few things, the curves do the rest

A new entry states only the facts. Everything else is derived:
- **Creature:** mass, role, home area, rarity, path, damage types, part layout.
- **Item:** tier, kind, material, damage type.
- **Perk node:** kind, region, skill gate, family.
- **Boss:** biome, first age, parts, offerings.

Tweaks are multipliers on derived values. Each tweak:
- is limited to ×0.75 to ×1.25;
- carries a `why`;
- is listed in the sheet with its reason;
- still has to pass every check after it.

A tweak outside the limits fails the build. If a designer really needs more, that's a sign the role or the curve is wrong, and the fix goes there. Base's species can stay in the Rust table (`species.rs`) with the same fields. Packages use TOML, as `notes/CRAFTING.md` already plans for materials and parts.

A creature:
```toml
# defs/creatures/frillback.toml
id = "animals:frillback"
mass_kg = 2600            # size from mass: huge
role = "grazer"           # prey | grazer | hunter | apex | tank
home_area = 23            # iron age, depth 3
rarity = "uncommon"
path = "beasts"           # which path's route uses it
attacks = ["blunt"]
parts = { head = { blunt = 0.5 }, frill = { cut = 1.5, breaks = true }, flank = {} }

[tweak]
health = { x = 0.85, why = "herds of 3-5 fight together; one is softer" }
```

An item part:
```toml
# defs/parts/steel_war_axe_head.toml
id = "base:steel_war_axe_head"
kind = "axe_head"         # kind spread: heavy, damage x1.25, speed x0.8
tier = 2
material = "steel"        # trait from the materials table: tempered
damage = "cut"

[tweak]
speed = { x = 0.9, why = "long haft; reads slow in the hand" }
```

A perk node:
```toml
# defs/web/beasts_pack_leader.toml
id = "web:beasts/pack_leader"
kind = "notable"          # small = 1 unit, notable = 3-4, keystone = 0 net
region = "beasts"
needs = { taming = 40 }
gives = [{ family = "tame_damage", units = 3 }]
next_to = ["web:beasts/whistle_1", "web:beasts/whistle_2"]
```

A boss:
```toml
# defs/bosses/sand_wyrm.toml
id = "bosses:sand_wyrm"
biome = "desert"
size = "titan"            # bosses are always titans
first_age = 1             # forms at area 17, 27, 37 ...
parts = ["jaw", "dorsal_plates", "tail"]
phase_locks = [{ at = 0.66, part = "dorsal_plates" }, { at = 0.33, part = "jaw" }]
offerings = ["base:sand_glass", "animals:dune_viper_fang"]
age_key_part = "bosses:wyrm_fang"
loot = "loot/sand_wyrm"   # item level 10t + 4-7 per form; bad-luck help certain by kill 8
```

### 5.5 The checks and sheets

| Check | Fails when | Writes |
|---|---|---|
| **Combat matrix** | For every gear tier (0-6, at grades 4 and 10, with and without a maxed web) against every creature at its median and 95th-percentile roll: TTK is outside its band at the same age, below the floor anywhere, or hits-to-down is outside its band | The TTK and hits-to-down matrix, one row per species, in `SPECIES.md`; the per-size tables in vitals' `SHEET.md` ("Combat") |
| **Tweaks** | A multiplier is outside ×0.75-1.25 or has no reason | The tweak list with reasons |
| **Path parity** | An age key by one path costs over 15% more EM than another; a path's reference fighter is outside the TTK bands | `PROGRESSION_SHEET.md` |
| **Build dominance** | A hill-climbed build wins every scored job by over 15%, or one node is in over 90% of the optimised builds (Slay the Spire's pick rate) | The top builds and node pick rates |
| **Loot per hour** | An activity's risk-priced value per hour is outside ±30% of its age's mean | EV per hour per activity and age |
| **Crafting simulation (Monte Carlo)** | Hours to an age's target gear are outside its budget at the median or 90th percentile; one source moves them by more than a set percent | Median and 90th percentile per age (already planned in `PROGRESSION.md`) |
| **Sinks and faucets** | A material accumulates without a sink over N simulated hours | Inflow and outflow per material per age |
| **Ceilings** | Any stat stack is over ×2.5 the reference; a resistance is over 75%; armour's cut is over 80% | |
| **Progression graph** | As `PROGRESSION.md` rule 3 | |

The combat simulator is pure maths over the derived stats: for each pair it computes hits, armour, the rate floor and phase locks. It's cheap enough to run every pair on every build. A second, slower test drives the real AI in a headless testkit world for a few sample pairs and compares the time against the simulator.

### 5.6 Where the numbers live

- **`mods/base/vitals/src/curve.rs`:** g(L), item level and area level, each age's hours (EM budgets), and the clamp caps.
- **`mods/base/vitals/src/combat.rs`:** the damage formula, damage types, the reference player, the TTK and hits-to-down bands, rate floors, and the combat simulator.
- **`mods/base/vitals/src/web.rs`:** perk units and the family ceilings.
- **`mods/animals/core/src/danger.rs`:** size from mass (a five-class `Size`), roles, derived creature stats, parts, the wild-roll and tamed-point budgets, and boss forms. `animals_core` already depends on `vitals`, so it reads the curve from there, and the website's WASM build gets the same numbers.
- **The tier data crate planned in `PROGRESSION.md` (step 3):** EM prices, loot tables, sinks and faucets, and path parity, with `PROGRESSION_SHEET.md`. It doesn't exist yet.

Mods keep only behaviour: `mods/base/src/player.rs` calls `vitals::combat` instead of its own armour maths, and the animals' attack code reads the derived hit.

### 5.7 The playtest loop with Tester

The simulator says what should happen. Tester checks that the game does it:
1. **A script per band:**
   - `/tk spawn` a creature at a set area level (a `/tk` option to force L);
   - `/tk give` a reference kit;
   - fight with `hold attack` and `look`;
   - log time and hits taken.
   Run it on the PC queue (`t2q.py`), with ticks frozen for shots.
2. **A combat log** (proposed: written by base on every death and kill to `run/saves/<world>/balance.log`, with time, item levels, damage done and taken) becomes our telemetry. Mega Crit's lesson is to log everything a run does.
3. **Compare** the logged TTK with the simulator's per pair. A gap over 25% means the AI, reach or hit detection is off, not the curve. That's a bug for the AI's owner.
4. **Ronald plays** a slice per age: a stone-age day, a bronze boss. His notes ("too tanky", "felt cheap") become tweaks with reasons, or changes to a band.
5. **The sheet records** each round's measured TTK next to the target, so drift shows up.

### 5.8 Questions for Ronald

1. **Bands:** a large dinosaur at 20-40 s, a huge one at 60-120 s, a titan or boss at 2-5 min, for a same-age player. Are those right?
2. **Floors:** a stone titan takes at least 2½ minutes even in star gear, a huge one about 50 s, a large one 10 s. Is that right, or should over-tier count for less (or more)?
3. **Wild levels:** keep Ark's big tail (A), or make a top roll worth about one age (B)?
4. **Tames across ages:** carried up by breeding in the next age's nests (A), or by saddles that raise a tier (B)?
5. **Scale of numbers:** health grows from 100 shown at stone to about 1,050 in the future age (×1.6 an age). Is that fine, or should the display divide by the age?

## Sources

Balancing methods
- Schreiber, [Level 3: Transitive Mechanics and Cost Curves](https://gamebalanceconcepts.wordpress.com/2010/07/21/level-3-transitive-mechanics-and-cost-curves/); notes [here](https://bowling-bash.blogspot.com/2011/03/literature-game-balance-ch3-transitive.html); Schreiber and Romero, [*Game Balance*](https://www.routledge.com/Game-Balance/Schreiber-Romero/p/book/9781498799577)
- Giovannetti, GDC 2019, ["Slay the Spire: Metrics Driven Design and Balance"](https://www.gdcvault.com/browse/gdc-19/play/1025731); [Game Developer](https://www.gamedeveloper.com/design/learn-i-slay-the-spire-i-s-metrics-driven-approach-to-game-balancing-at-gdc-2019)
- Wilson, GDC 2019, ["Designing Path of Exile to Be Played Forever"](https://gdcvault.com/play/1025784/Designing-Path-of-Exile-to)
- Mosqueira, GDC 2015, ["Against the Burning Hells"](https://gdcvault.com/play/1021776/Against-the-Burning-Hells-Diablo); [Loot 2.0](https://di.diablowiki.net/Loot_2.0)
- LeBlanc, GDC 1999, feedback systems: [Wikipedia](https://en.wikipedia.org/wiki/Marc_LeBlanc), [8kindsoffun](https://www.8kindsoffun.com/)
- Riot, [Champion Balance Framework](https://nexus.leagueoflegends.com/en-au/2019/05/dev-champion-balance-framework/), [2020 update](https://www.leagueoflegends.com/en-us/news/dev/dev-balance-framework-update/)
- Wizards, [Play Design](https://mtg.wiki/page/Play_Design), [Future Future League](https://mtg.wiki/page/FFL), [Play Design lessons learned](https://magic.wizards.com/en/news/feature/play-design-lessons-learned-2019-11-18)
- WoW: [itemization formulas](https://wow.allakhazam.com/wiki/Itemization_Formulas_(WoW)), [Item level](https://warcraft.wiki.gg/wiki/Item_level), [level squish](https://blizzardwatch.com/2020/09/24/world-warcraft-need-level-squish/), [Massively OP](https://massivelyop.com/?p=297815), [the earlier squish](https://massivelyop.com/?p=218228)
- Factorio, [Friday Facts #217](https://cf-www.factorio.com/blog/post/fff-217), [Assembling machine 1](https://wiki.factorio.com/Assembling_machine_1/fr), [forum on speeds](https://forums.factorio.com/viewtopic.php?p=386364)
- Hades: [Heat](https://hades.fandom.com/wiki/Heat), [RPG Site](https://www.rpgsite.net/feature/10287-hades-pact-of-punishment-heat-modifiers-and-how-to-maximize-your-rewards), [GameDaily on early access](https://www.gamedaily.biz/?p=588)
- Koster, [Astromech Stats: Economy Stats](https://www.raphkoster.com/games/essays/astromech-stats-economy-stats/); [mudflation](https://boingboing.net/2007/01/19/mudflation-inflation.html)
- Dormans, [Simulating Mechanics to Study Emergence in Games](https://ojs.aaai.org/index.php/AIIDE/article/view/12477); [Machinations, Monte Carlo personas](https://machinations.io/articles/balancing-f2p-economies-simulating-player-personas-and-progression-curves-with-machinations)
- de Mesentier Silva et al., [Evolving the Hearthstone Meta](https://arxiv.org/abs/1907.01623)
- Wired, [Halo 3: How Microsoft Labs Invented a New Science of Play](https://wirednews.pages.dev/2007/08/ff-halo-2)
- Chen, [Flow in Games](https://www.jenovachen.com/flowingames/missionstatement.htm)

Games
- Monster Hunter: [Kiranico, Understanding Monster](https://mhworld.kiranico.com/guide/understanding-monster), [Fextralife, resistances](https://monsterhunterrise.wiki.fextralife.com/Elemental+Resistances), [Tsujimoto via GameBanshee](https://gamebanshee.com/k493y)
- Ark: [Creature stats calculation](https://ark.wiki.gg/wiki/Creature_stats_calculation), [Imprinting](https://ark.wiki.gg/wiki/Imprinting), [Mutations](https://ark.wiki.gg/wiki/Mutations), [difficulty](https://xgamingserver.com/docs/ark-survival-evolved/difficulty-levels)
- Valheim: [Creatures](https://valheim.fandom.com/wiki/Creatures)
- Path of Exile: [Armour](https://www.poewiki.net/wiki/Armour), [PoE 2 Armour](https://poe2wiki.net/wiki/Armour), [Monster](https://www.poewiki.net/wiki/Monster), [Monster level](https://pathofexile.fandom.com/wiki/Monster_level)
- Elden Ring: [Damage types](https://eldenring.wiki.fextralife.com/Damage+Types), [Damage negation](https://eldenring.wiki.gg/wiki/Damage_negation), [Stance](https://eldenring.wiki.gg/wiki/Stance), [Poise](https://eldenring.wiki.gg/wiki/Poise)
- Skyrim and Oblivion: [UESP, Leveling](https://pt.uesp.net/wiki/Skyrim:Leveling), [Steam](https://steamcommunity.com/app/22330/discussions/0/5383446733857395241), [Nexus](https://forums.nexusmods.com/topic/6009123-enemy-difficultyscalingai)
- Terraria: [Expert Mode](https://terraria.wiki.gg/wiki/Expert_Mode), [Master Mode](https://terraria.wiki.gg/wiki/Master_Mode), [Reforging](https://terraria.wiki.gg/wiki/Reforging)
- Destiny 2: [contest mode](https://stevivor.com/news/destiny-2-beyond-light-deep-stone-crypt-contest-mode-detailed-bungie), [GamesRadar on the power grind](https://www.gamesradar.com/the-destiny-2-forge-farm-fiasco-is-another-reminder-that-its-power-grind-needs-to-change)
- ESO and Guild Wars 2: [One Tamriel](https://en.m.uesp.net/wiki/Online:One_Tamriel), [Massively OP on zone scaling](https://massivelyop.com/2016/06/19/mmo-mechanics-zonation-and-scaling-mechanics-in-mmorpgs/)
- League of Legends: [Armor](https://wiki.leagueoflegends.com/en-us/Armor), [Life](https://wiki.leagueoflegends.com/en-us/Life)
- Palworld: [Level](https://palworld.wiki.gg/wiki/Level), [Pal Condensation](https://palworld.wiki.gg/wiki/Pal_Condensation)
