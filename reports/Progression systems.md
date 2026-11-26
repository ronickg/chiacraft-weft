# Progression systems: what to take, what to avoid, and gear that keeps getting better

Researcher, 2026-11-05, for Planner and Ronald, on `notes/PROGRESSION.md`. Ronald's goal for gear: "you can effectively keep on playing and you can always end up getting something better or merging something to get it better". This is across seven ages, with tech, magic and beasts braided, altar bosses, Skyrim-style skills, exploration gates, automation, dimensions and a power suit. It should last 100+ hours and take new content later without breaking balance.

The proposal at the end fits what `notes/CRAFTING.md` already decided: parts, ten grades, and "no re-rolls to buy". Path of Exile 2 is in early access and changes every patch. Its mechanics here come from guides for patches 0.1 to 0.3 and the wiki, so exact numbers may have moved; the shapes are what matter.

## The top 10 ideas to take

1. **Item level caps what can roll (Path of Exile 2).** Each item has an item level from where it dropped, and each affix tier needs a minimum item level. In chiacraft, item level = tier × 10 + a depth bonus from where the material or drop came from (boss form, ley vein, dimension depth). Every age's ceiling is fixed in data.
2. **Found crafting materials, each with one job (PoE 2's currency).** Each one adds, swaps, rerolls, locks or gambles. In chiacraft they come from ley nodes, ruins and meteorites, so exploring keeps paying after the map is "done".
3. **Certainty costs more than chance (PoE 2's essences and omens).** Each altar boss drops an essence that guarantees one property of its biome's family. Wanting frost marks becomes a reason to summon the frost mammoth.
4. **Merging two items into one (PoE 2's recombinator, Last Epoch's legendaries).** A star-age ruin gives the ancients' fusing anvil, which pools two parts' properties into one, with risk. This is Ronald's "merging something to get it better".
5. **A repeatable endgame that drops its own next step (PoE 2's Atlas and waystones).** Star charts open pockets of the ancients' worlds as linked worlds, at depths 1 to 16. Deeper charts drop deeper charts and better item levels.
6. **Bosses in harder forms with rewards that add up (Ark's gamma, beta and alpha; Terraria's Expert bags).** Each biome boss has a form per age. A higher form gives everything the lower one gives, plus more.
7. **A boss's drop opens the next material (Valheim, Terraria).** Valheim's antlers make the pickaxe for copper, and the swamp key opens the crypts with iron. In chiacraft this is the boss part of an age key: a tool or key, not just stats.
8. **Skills rise by use, perks gated by skill, points from levels (Skyrim), with its bugs fixed.**
   - Experience comes from the challenge, not from how many cheap things you make.
   - No crafting buff feeds another crafting buff.
   - Enemies scale by place and age, never by character level.
9. **Physical gates plus a book that explains them (Vintage Story), and knowledge from studying the world (Thaumcraft).** Gates are builds like a steel furnace, not "level 30". The journal explains, and ruins hold only the best blueprints. Magic research comes from studying ley nodes, relics and rare life.
10. **Budgets, curves and a graph check (WoW item budgets, GregTech's ×4 tiers, Dormans' lock-and-key graphs).** Every item's power is a budget from its tier and slot. Each tier costs a fixed multiple of the last. A test walks everything. This extends `PROGRESSION.md`'s rules 1 to 5 with item level, mark budgets and a crafting simulation.

## Path of Exile 2

### Items: rarity, affixes, tiers, item level
- **Rarity:** a normal item has no affixes. A magic item has up to 1 prefix and 1 suffix, a rare up to 3 and 3, and a unique has fixed effects ([Maxroll crafting overview](https://maxroll.gg/poe2/resources/path-of-exile-2-crafting-overview)).
- **Tiers:** each affix has tiers, and a higher tier is a higher value band. The game rolls the tier, then the value inside it ([Mobalytics, item modifiers](https://mobalytics.gg/poe-2/guides/item-modifiers), [poe2wiki, Tier](https://poe2wiki.net/wiki/Tier)).
- **Item level:** it's the level of the monster that dropped the item. It permanently caps which affix tiers the item can get, from drops or crafting ([poe2wiki, Level](https://poe2wiki.net/wiki/Level)).
- **Bases:** the base sets the affix pool. Strength bases draw armour, dexterity bases evasion, intelligence bases energy shield, and hybrid bases draw from both.

### Crafting currencies
All of these drop as loot ([Maxroll](https://maxroll.gg/poe2/resources/path-of-exile-2-crafting-overview), [poe-vault](https://www.poe-vault.com/poe2/guides/currency-and-crafting)).

| Currency | Does |
|---|---|
| Transmutation / Augmentation | normal → magic with 1 affix / adds an affix to magic |
| Regal / Alchemy | magic → rare, +1 affix / normal → rare with 4 |
| Exalted | adds a random affix to a rare |
| Chaos | removes a random affix and adds a random one |
| Annulment | removes a random affix |
| Divine | rerolls every affix's numbers |
| Fracturing | locks a random affix on a rare with 4+ affixes |
| Vaal | corrupts (an enchant, reroll up to 3 affixes, an extra socket, or nothing); no more crafting |
| Greater / Perfect versions | the added affix has a minimum level, so it can't be a low tier |

PoE 2 removed PoE 1's Scouring (wipe and restart) and Alteration (endless magic rerolls), so you can't erase and try again ([Primagames](https://primagames.com/tips/complete-beginners-guide-to-crafting-in-path-of-exile-2)). Every step commits, so every orb matters.

**Steering the gamble:**
- **Essences** make a magic item rare with one chosen affix ([Mobalytics, essences](https://mobalytics.gg/poe-2/guides/essences)).
- **Omens**, from Rituals, change the next currency use. Dextral Annulment, for example, removes only a suffix ([Mobalytics, omens](https://mobalytics.gg/poe-2/guides/omen-crafting)).
- **Desecration** (0.3) adds a hidden affix that you reveal at the Well of Souls by choosing one of three ([poe2wiki](https://www.poe2wiki.net/wiki/Desecrate)).

**Merging:** the Recombinator (0.2.0) merges two items of one class. Prefixes from both go into one pool and suffixes into another, and the result draws from those pools. It's paid in Expedition artifacts and costs more for rarer affixes. Mixing many different affixes dilutes your odds ([poe-vault](https://www.poe-vault.com/poe2/news/recombination-crafting-is-here-risky-item-creation-returns)).

Last Epoch's version is tighter. A unique with "legendary potential" 1 to 4 merges with a four-affix item, and up to that many affixes carry over, one merge per boss key ([Maxroll, Last Epoch](https://maxroll.gg/last-epoch/resources/legendary-items-crafting-guide)). The potential number caps the payoff.

**Junk still feeds crafting:** disenchanting and salvaging give shards of currency and quality scraps.

### Skill gems and the passive tree
Skills are found: you engrave an uncut gem into a skill, and a higher-level uncut gem levels it up. Support gems go in that skill's own sockets (2, up to 5 with Jeweller's orbs), each usable once per build. A shared passive tree of over a thousand nodes and a small ascendancy tree sit on top ([poe-vault gems guide](https://www.poe-vault.com/poe2/guides/gems-guide)). For chiacraft this is the model for spell tomes that roll quality, with rune "supports" slotted into a focus.

### The Atlas and waystones
- **Waystones** are consumable maps:
  - tiers 1 to 15 give area levels 65 to 79;
  - a Vaal orb can make a tier 16 (level 80);
  - area level sets monster level, which sets item level ([poe2wiki, Waystone](https://www.poe2wiki.net/wiki/Waystone)).
- **The loop sustains itself.** Harder maps drop higher waystones. Crafted waystone affixes raise both danger and loot ([Maxroll, Atlas](https://maxroll.gg/poe2/resources/atlas-of-worlds-and-mapping)).
- **The player steers it:**
  - Tablets add Breach, Ritual or Expedition to nearby maps.
  - An Atlas passive tree picks what your endgame drops.
  - Pinnacle bosses sit behind keys.
- **A death costs one of the map's portals, not gear.**

### Why there's always something better
1. **Many independent rolls.** Six affixes, each with a tier and a value, make a perfect item astronomically rare under a fixed ceiling.
2. **Currency is loot.** Even with no upgrade drop, orbs are steps toward one.
3. **The player picks the risk.** Map affixes, omens, corruption and merging trade safety for upside.

Chris Wilson's GDC 2019 talk ["Designing Path of Exile to Be Played Forever"](https://gdcvault.com/play/1025784/contactUs) is built on "multiple overlapping axes of randomness" ([80.lv](https://80.lv/articles/gdc-designing-path-of-exile)).

### How they control power creep
- **Fixed ceilings:** item level caps what can roll, so leagues add breadth, not height.
- **League resets:** 0.2.0 wiped stashes so players couldn't stockpile ([Primagames](https://primagames.com/?p=306904)).
- **Nerfs:** Wilson wrote in 2021 that power creep had hit "a breaking point" ([Reddit mirror](https://reddit.rtrace.io/r/pathofexile/comments/op73wg/some_thoughts_from_chris/h649nxj)). PoE 2's 0.2.0 nerfs and costly respecs drew a review-bomb ([GamesRadar](https://www.gamesradar.com/games/action-rpg/there-were-some-blatant-f-k-ups-path-of-exile-2-devs-rush-out-emergency-patches-after-nerfs-lead-to-review-bombing-on-steam)).
- **Fast fixes to farming exploits:** a currency farm was cut within days ([Gfinity](https://www.gfinityesports.com/article/path-of-exile-2-players-found-another-fortune-making-strategy-and-ggg-shut-it-down-fast)).

**For chiacraft:** we can't wipe players' worlds. Our controls have to be built in from the start: fixed ceilings, budgets, and a simulation that catches a too-generous source before it ships.

## Skyrim

- **Skills by use, levels by skills:**
  - Each skill-up gives character experience equal to the new skill level, and a character level needs (level + 3) × 25 ([UESP, Leveling](https://pt.uesp.net/wiki/Skyrim:Leveling)).
  - Each level gives one perk point for any tree. There are 251 perk ranks.
  - Since 1.9 a maxed skill can go "legendary": reset to 15, perks refunded, levelled again ([UESP, Perk](https://en.UESP.net/wiki/Skyrim:Perk)).
- **Smithing is a material ladder:** Ebony needs Orcish and skill 80, and Daedric needs Ebony and 90. Each material's perk also doubles how far you can temper it ([UESP, Ebony](https://www.uesp.net/wiki/Skyrim:Ebony), [Tempering](https://en.uesp.net/wiki/Skyrim:Tempering)).

**The known exploits:**
- **The restoration loop:** Fortify Restoration potions also boosted Fortify Skill enchantments, so potions and gear kept boosting each other without limit. UESP calls it game-breaking; the Unofficial Patch fixes it ([UESP](https://app.uesp.net/wiki/Skyrim:Fortify_Restoration)).
- **Alchemy-enchanting loop:** Fortify Enchanting potions make better Fortify Alchemy gear, which makes better Fortify Enchanting potions. It converges after three or four rounds, but well above intended ([UESP talk](https://app.uesp.net/wiki/Skyrim_talk:Enchanting)).
- **Grinding cheap items:** smithing experience follows the item's value, so players forge stacks of iron daggers.
- **Crafting makes you weaker:** enemies scale with character level, so players who level crafting first find enemies outgrow their combat skills ([Steam](https://steamcommunity.com/app/489830/discussions/0/3884977132825320881), [Nexus](https://forums.nexusmods.com/topic/6009123-enemy-difficultyscalingai)).

**Take:** skills by doing, perks gated by skill level, points from levels, a material ladder in Smithing, and the legendary reset for the long game.

**Fix:**
- Experience comes from the tier against your skill.
- Crafting boosts never apply to crafting-boost items.
- Enemies scale by place.

## Ark: Survival Evolved

- **Summoning:** you bring artifacts from caves and trophies from apex creatures to an obelisk. Each boss has three forms: gamma, beta and alpha.
- **Broodmother as an example:** levels 30, 50 and 70; the same three artifacts plus 0, 5 or 10 of each of four apex drops; 324k, 648k and 972k health; at most 10 players and 20 tames in the arena ([ARK wiki](https://ark.wiki.gg/wiki/Broodmother_Lysrix)).
- **Rewards:** Element, a trophy for the form, and Tek engrams. The engrams are cumulative: beta includes what gamma gives ([Guildorder](https://guildorder.com/games/ark/guides/boss-progression)).
- **Tek is gated twice:** bosses unlock it, and it runs on Element, which mostly comes from bosses. The final boss "ascends" you and raises your level cap ([Guildorder](https://guildorder.com/games/ark/wiki/boss-fights)).

**Take:**
- Forms per age with rewards that add up.
- Offerings from both exploring (caves) and hunting (apex trophies).
- An arena tame cap.
- A late fuel only bosses give, as a model for star cores.

**Avoid:** Ark's engram lists vary by map, and guides contradict each other. Keep one table per boss, in data, with a generated sheet.

## Terraria

- **The gate chain** ([wiki](https://terraria.wiki.gg/wiki/Guide:Game_progression)):
  - The Wall of Flesh starts Hardmode and drops the Pwnhammer.
  - Smashing altars with it seeds three ore tiers, each mined with the last.
  - The three mechanical bosses open chlorophyte.
  - Plantera opens the Temple and Golem, then the Cultist, then the Moon Lord.
- **Bosses change the world.** They bring NPCs (the Steampunker, the Cyborg), events and biomes.
- **Harder modes add unique loot rather than more loot.**
  - Expert bags add one Expert-only item per boss; the Demon Heart, for example, gives an extra accessory slot.
  - Master mode adds a relic and a pet ([Treasure Bag](https://terraria.wiki.gg/wiki/Treasure_Bag), [Master Mode](https://terraria.wiki.gg/wiki/Master_Mode)).
- **Reforging is seasoning:**
  - Any reforge prefix is equally likely, and you pay in coins.
  - The best, Legendary, is about +15% damage and +10% speed.
  - A Legendary copper sword never beats a plain sword of the next tier ([Reforging](https://terraria.wiki.gg/wiki/Reforging)).

**Take:**
- Keep each tier's jump bigger than the grade and mark spread inside it.
- Let a first kill change the world: a trader arrives, or a ley node wakes.

## Vintage Story

- **Physical gates:**
  - Copper and bronze are cast in crucibles and moulds.
  - Iron can't be melted. It goes through a bloomery and a bronze-or-better anvil, and its ore needs a bronze pickaxe.
  - Steel needs an iron anvil, a pulverizer and a cementation furnace loaded with 16 ingots and 40 carbon, fired about 6.67 days ([Steel making](https://wiki.vintagestory.at/Steel_making/en), [Steel](https://wiki.vintagestory.at/Steel)).
- **Knowledge isn't locked:** the in-game handbook explains processes. The gate is doing it.
- **Exploration gates the ancient tech:**
  - Ruined translocators need 2 metal parts and 3 temporal gears to fix ([Translocator](https://wiki.vintagestory.at/Translocator)).
  - Jonas parts, found only in ruins, build devices like the base-return teleporter ([Jonas part](https://wiki.vintagestory.at/Jonas_part)).
  - Lore books tell their maker's story.

**Take:**
- Each age's infrastructure gate should be a multi-day multi-block build with materials from two biomes.
- The ancients' devices (gates, suit cores) need found parts.
- The journal explains; blueprints gate only the best few things.

## Valheim

- **Bosses gate materials:**
  - Eikthyr's antlers make the copper pickaxe.
  - The Elder's key opens the iron crypts.
  - Bonemass's wishbone finds silver ([Guildorder](https://guildorder.com/games/valheim/wiki/boss-and-biome-progression)).
- **Each boss is summoned at an altar** with its biome's offerings (deer trophies, ancient seeds, withered bones) and gives a power.
- **Upgrades:** gear upgrades at its station to quality 4, once the station is raised with add-ons ([GameRant](https://gamerant.com/valheim-how-craft-upgrade-forge/)). Version 1.0 added the Forge of Potential to go further using idols, and a failure can destroy the item ([4Netplayers](https://www.4netplayers.com/en/blog/valheim/valheim-1-0-forge-of-potential-equipment-beyond-level-4-upgrade/)).
- **Death costs about 5% of skill progress,** and world settings can change that ([Freakhosting](https://help.freakhosting.com/games/valheim/configuring-skill-drain-on-death.md)).

**Take:** this is almost exactly chiacraft's boss plan. Make the drop a tool or key. Plan the endless upgrade layer from the start; Valheim bolted its on late.

## GregTech, Create, Applied Energistics, All the Mods

- **GregTech:**
  - Each voltage tier is 4× the last ([GregTech CEu docs](https://gregtechceu.github.io/GregTech-Modern/Gameplay/Electricity)).
  - Overclocking a recipe one tier up halves its time and quadruples its power, so old recipes stay useful but cost more ([IC2 forum](https://forum.industrial-craft.net/thread/11695-gregtech-5u-tech-tiers-eu-x-speed-ratio-setting)).
  - Each tier's machines need that tier's circuits, and blast furnace recipes need a minimum coil tier ([GTNH circuit notes](https://crazedd0ge.github.io/gtnhnotes/circuits)). The graph is strict, and every tier needs the one below.
- **Applied Energistics 2 limits the network on purpose:**
  - A cable carries 8 channels, a dense one 32, and a controller gives 32 per face.
  - Every storage cell, 1k to 256k, holds at most 63 item types ([AE2 guide](https://guide.appliedenergistics.org/1.18/features/me-network/storage-cells)).
  - So big storage stays a layout puzzle.
- **Create has no power tiers.** Generators add stress capacity, machines add impact, and the network stops when demand exceeds supply ([Create wiki mirror](https://git.grimmauld.de/mirrors/Create/src/commit/75145976d38eda619d8070f54ed52b0c740300e8/wiki/Rotational%20Force%20in%20Create)). You scale by building more.
- **All the Mods ends in a capstone:** the ATM Star needs materials from most of the pack's mods and dimensions ([SiriusMC guide](https://wiki.siriusmc.net/books/modpack-guides-and-tutorials/chapter/all-the-mods-10/export/html)).

**Take, by age:**
- **Steam age:** Create's stress model, with tames on treadwheels adding capacity.
- **Electric age:** GregTech's ×4 tiers with overclocking, and AE2's channel and type limits for the storage network.
- **Star age gate:** a capstone like the ATM Star that needs a piece from every path and several dimensions.

## Thaumcraft and Botania

- **Thaumcraft:**
  - Scanning things with the thaumometer gives knowledge, which you spend in a research puzzle. The Thaumonomicon fills in as you go ([FTB wiki](https://m.ftbwiki.org/Thaumometer_(Thaumcraft_6))).
  - Magic draws vis from each chunk's aura. Careless work makes flux, and flux spreads taint ([Thaumcraft wiki](https://thaumcraft-5.fandom.com/wiki/Warp,_Flux,_and_all_things_bad?oldid=1055)). Where you work matters, and power has a world cost.
- **Botania:**
  - Generating flowers make mana, pools hold it, and functional flowers spend it.
  - Terrasteel is mana-infused iron.
  - Elementium comes from trading through an elven portal: a dimension for trade, not war.
  - The Gaia Guardian's harder second form gives the best loot ([Craft Down Under](https://craftdownunder.co/guides/mods/botania)).

**Take:** research by studying the world (Runecraft and Explorer trees); a flux-like cost for heavy magitech on a ley line, as a brake on runaway power; a mana network that mirrors the item network; one trade dimension.

## Balancing for extensibility

- **Stat budgets:**
  - WoW gives each item a budget from its item level, times a slot factor (1.0 for chest and two-handers, 0.55 for rings).
  - Stats spend that budget, and the sum of their costs to the power 1.5 must fit the budget to the power 1.5, which keeps hybrid items fair ([itemization formulas](https://wow.allakhazam.com/wiki/Itemization_Formulas_(WoW)), [Item level](https://warcraft.wiki.gg/wiki/Item_level)).
  - Later WoW grew the budget 15% every 15 item levels ([bloodmallet](https://bfa.bloodmallet.com/wowtc/system/stat-scaling/)).
  - A new item needs only a level and a slot.
- **Diminishing returns:** defence as armour ÷ (armour + k) stops one stat from breaking things. Test that starter gear can't beat endgame content ([rpgcore guide](https://rpgcore.readthedocs.io/en/latest/content/progression-guide/)).
- **Narrow swings (Diablo 3's Loot 2.0):**
  - Reaper of Souls narrowed how far stats could swing, dropped fewer but better items, and skewed drops toward your class ([Diablo wiki](https://di.diablowiki.net/Loot_2.0)).
  - The reasons are in Josh Mosqueira's GDC 2015 talk ["Against the Burning Hells"](https://gdcvault.com/play/1021776/Against-the-Burning-Hells-Diablo).
  - Travis Day's GDC 2017 talk ["Rewards in Video Games"](https://gdcvault.com/play/1023948/Rewards-in-Video) covers what rewards cost a game.
- **Bad-luck protection:** WoW Legion raised legendary odds after each miss ([Wowhead](https://www.wowhead.com/news/legendary-items-soft-cap-removed-bad-luck-protection-beyond-four-legendaries-257898)), and RuneScape documents its version openly ([wiki](https://runescape.wiki/w/Bad_luck_mitigation)).
- **Progression as a graph:** Joris Dormans' lock-and-key graphs and Machinations diagrams (sources, drains, converters) are the theory behind our graph check ([thesis](https://eprints.illc.uva.nl/id/eprint/2118/), [Boris the Brave](https://www.boristhebrave.com/2021/02/27/lock-and-key-dungeons)). Model each age's economy, not only its locks. Schreiber and Romero's *Game Balance* covers pacing curves ([book](https://www.routledge.com/Game-Balance/Schreiber-Romero/p/book/9781498799577)).
- **Don't take power away:** Destiny 2's "sunsetting" capped old weapons. It was dropped after four months and fully reversed in 2024 ([Dexerto](https://www.dexerto.com/destiny/bungie-removes-destiny-2-sunsetting-1522114/)).
- **No uncapped levels:** Diablo 3's Paragon is essentially uncapped, and players call it a long grind for tiny gains ([Diablo wiki](https://diablowiki.net/Paragon), [forum](https://us.forums.blizzard.com/en/d3/t/infinite-or-finite-progression/9062)). Let the endless part be rolls approaching a fixed ceiling.

## What to avoid

1. **Boosts that feed themselves** (Skyrim's loops). Tag effects; fail any crafting boost that can apply to a crafting-boost item.
2. **Experience from quantity** (iron daggers). It scales with tier against skill.
3. **Enemies scaled by character level** (Skyrim). Scale by age, place, form and depth.
4. **Free wipe-and-retry** (PoE 1's scouring, Silent Gear's catalyst grind). Every step costs found material and commits.
5. **Power with no ceiling** (Paragon), and **taking power away** (sunsetting). Cap every number per tier; put new ages on top.
6. **Balancing by nerfing owned items** (PoE 2's 0.2.0). Players keep their worlds, so simulate before shipping.
7. **Losing 50 hours of crafting to one death.** A death in a dimension should cost the expedition, not the gear: the pack returns to the gate's hub chest, as PoE 2 costs portals, not items.
8. **Bosses farmed on the easy form.** Rewards add up across forms, so the hard form is the efficient one.
9. **One path forced, opaque tables, recipe hunts for basics.** Test each path's route and gear; keep everything in data with a sheet; blueprints gate only an age's top few items.

## A concrete proposal for chiacraft's endless gear

Parts and the ten grades from `CRAFTING.md` stay. A grade is made by the work, the material and the station, with no re-rolls. On top comes one new layer, **marks**: magic properties found or added with materials from exploring and bosses. Grades are craft; marks are the endless chase.

### 1. Item level and mark tiers
- **A part's item level** is 10 × its tier + a depth of 0–9, from where its material or drop came from:
  - surface ore: 0–2;
  - a ley vein or a far deposit: 3–5;
  - a boss form or ruin cache: 4–7;
  - a dimension at depth d: up to 9.
- **The scale:** item level runs 0 (stone) to 69 (star). A tier 7 later adds 70–79.
- **Marks** come in families (keenness, cold, haste, mana, yield, tame bond…). Each family has tiers, each with a minimum item level and a value band from the tier curve. The tier rolls first, then the value.
- **Slots:**
  - up to 2 marks from bronze, 4 from iron, and 6 (3 + 3) from the electric age;
  - one mark per family;
  - the grade's made-modifier slots stay separate.
- **Marks are data:** family, path, part kinds, weight, and tiers. A package adds marks without code.

### 2. Crafting materials, all found
| Material | Like | Does | From |
|---|---|---|---|
| Rune chalk | transmute / augment | adds a mark to a free slot | ley nodes, bronze age on |
| Ley shard | exalted | adds a random mark | ley nodes, ruins |
| Star dust | chaos | swaps one random mark | meteorites |
| Pallasite tear | divine | rerolls every mark's numbers | pallasite, rare boss drops |
| Unbinding salt | annulment | removes a random mark | salt flats, traders |
| Star-steel rivet | fracturing | locks a mark | star steel |
| Boss essence | essence | adds a mark of the boss's family at its form's tier | each altar boss and form |
| Ruin rubric | omen | steers the next craft (only the lowest mark, no repeats…) | ruins |
| Relic whisper | desecration | adds a hidden mark, revealed at a ley well by choosing one of 3 | deep ruins |
| Ley overload | Vaal | final gamble at a node: one mark up a tier, an inherent effect, nothing, or a cracked slot; then sealed | any node, with a star core |

- **Greater chalk and shards,** from boss forms and deeper charts, set a minimum mark tier.
- **Salvaging** a marked part returns shards of what was used.
- **Skills gate actions, not numbers.** Runecraft opens chalk, shards and reveals. Smithing opens rivets and fusing. Top perks add certainty, such as seeing a whisper's three choices first. No perk widens a value band, so there's no Skyrim loop.

### 3. Merging and carrying items up
- **The fusing anvil** comes from a star-age blueprint. A smaller bronze-age one fuses only parts of the same tier.
  - Two parts of the same kind go in, and one comes out.
  - Marks pool by half (prefix-like with prefix-like, suffix-like with suffix-like), and the result draws from those pools.
  - The higher grade is kept, minus one on a bad roll.
  - It costs boss essences, scaled by the summed mark tiers.
- **Relics** have a potential of 1–4, as Last Epoch's legendaries do. Fusing a relic with a marked part carries over that many marks and keeps the relic's unique effect.
- **Rebasing** carries an item to the next tier for that tier's material and a boss part.
  - It keeps the item's history, parts, grade and mark families.
  - Base stats move to the new curve, and each mark rerolls its tier under the new ceiling.
  - Nothing is ever sunset, but rebasing costs about as much as making a new item.

### 4. Bosses and bad luck
- **Forms:** each biome boss has a form per age from its first one. Offerings come from its biome, plus trophies from that age's apex creatures.
- **Rewards add up:** a form gives what lower forms give, plus its age-key part, essences, a trophy, and a loot roll at item level 10 × tier + 4–7.
- **Bad-luck help** counts per player, boss and form. A missing piece gains weight each kill and is certain by kill N (about 8, set in the table). Essences count too.
- **After a win:** a young one or an egg, as decided.

### 5. The repeatable endgame: star charts
- **What a chart is:** the ancients' coordinates, found in future-age ruins. A chart opens a pocket dimension, generated as a linked world from its seed. It has:
  - a theme;
  - a depth of 1–16;
  - a guardian at the end, an echo of a biome boss.
- **Depth sets loot:** depth d drops at item level 60 + d/2, up to 69, and deeper charts raise the share of high mark tiers.
- **Charts sustain themselves:** a chart drops charts of its depth ±1, and the guardian always drops depth + 1. Rune chalk on a chart adds danger marks (more creatures, cold, no healing near ley) that raise loot quantity and rarity.
- **The Cartography tree is the Atlas tree.** It biases which themes, materials and echoes charts roll.
- **Dying in a chart** spends one of its 3 anchors, and your pack returns to the gate chest.
- **New dimension packages** are new chart themes at a depth range.
- **The final boss** stays in the furthest dimension. After it, depth-16 charts and echoes are the open play.

### 6. Power-creep controls, in data and tested
1. **The tier dominates.** Each tier's base budget is about 1.6× the last. Grade and marks together add at most +60% over an unmarked grade-4 item, so a perfect tier-N item roughly equals a plain tier-N+1 one.
2. **A mark budget per item:** marks' values, normalised to their family's maximum and weighted by slot, with WoW's 1.5 power, must fit the tier's budget. Relics get the same budget plus a stated trade-off.
3. **Ceilings, not counters.** The top mark tier is fixed per item level. Defence has diminishing returns; resistances, speed and cooldowns have hard caps.
4. **New content adds breadth inside a band:** families, relics, chart themes and bosses. Only a new age adds height, on top. The graph check fails a mark above its band.
5. **A crafting simulation in `PROGRESSION_SHEET.md`:**
   - It plays N hours at each age and chart depth with the expected drop rates.
   - It reports the median and 90th percentile of the best item so far, and the hours to a set target.
   - It flags any source that moves these numbers more than a set percent.
6. **Sinks:** crafting materials, essences for fusing, repairs (as decided), cracked slots, consumed charts, and death drops in the open world.
7. **Enemies scale by place:** age, biome, form and depth, never character level.
8. **Linked worlds:** extend the portal admission rule from grade to item level and mark budget, so a star blade can't trivialise a friend's bronze world.
9. **Bounded bad luck:** every random reward has a certain point, and the graph check fails a table without one.
10. **Paths stay even:** the sheet fails if one path's best gear at a tier is more than about 15% ahead of the others.

### What to build first
1. Add item level, mark families and the tier budgets to the tier data crate planned in `PROGRESSION.md`, step 3.
2. Add four checks to the graph test: mark ceilings, the mark budget, bounded bad luck, and path parity.
3. Write the crafting simulation before any mark content.
4. Build a bronze-age slice (rune chalk, ley shards, two boss essences, a dozen marks) and have Tester judge whether "there's always something better" before adding the rest.
