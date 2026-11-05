# Survival systems gap analysis

Researcher, 2026-11-05. Ronald asked: "do research for other games where they have some other cool,
unique stuff, which might be cool to add ... I'm not sure if we're missing some system still for
our game. Especially look at survival games, some common ones too."

Read first: `notes/OVERVIEW.md` (every piece and its status), `notes/SYSTEMS.md` (the table),
`notes/PROGRESSION.md` (the seven ages), the headings of `notes/ECOLOGY.md`, `notes/SURVIVAL.md`
and `notes/ANIMALS.md`, and Research round 1's feature list (`reports/Research round 1.md`, F1-F15),
so nothing here repeats it without saying so.

Status words used below:
- **has**: built and seen working.
- **part**: some of it is built, or it's designed in detail.
- **planned**: named in PROGRESSION or ECOLOGY, nothing built, no design yet.
- **lacks**: not in the notes at all.

## The short answer

chiacraft's body, food, climate, season and animal systems are already deeper than almost every
game on the list. Only TerraFirmaCraft and Vintage Story come close on food and climate, and none
has its ecology. The gaps are nearly all *outside* the player's body:
- things that come to your base;
- what you do on and in water;
- ways to move up and across the land;
- ways to know where you've been;
- other people in the world.

Six of the top 15 are things PROGRESSION already promises but hasn't shaped yet: tames at work,
settlements, ruins, the transport ladder, knowledge as a gate, and mechanical power before
electricity. The research below is mostly about *how* other games made those fun.

## 1. Top 15 to add

Ranked by what it adds against what it costs, and by how much it leans on what's already built.

| # | System | Source games | What it adds | Fit with chiacraft | Size |
|---|---|---|---|---|---|
| 1 | **Fishing, nets and traps** | Stardew Valley, Core Keeper, Valheim, Aquaculture 2, Primitive Survival | A calm food loop at every river, lake and coast; a reason to be by water at dawn | Fish already live, breed and sit in the food webs, but nothing catches them (round 3 found the beach "weak for food"). It fits the Survival tree, the stone and bronze ages (line, net, weir) and the ecology's populations | M |
| 2 | **Tames at work** | Palworld, Conan Exiles, V Rising, Once Human, Ark | Bases that run while you explore; every species worth keeping for its job | PROGRESSION names "work and farming" for tames in every age, but there's no job model. Species are data, so work traits can come from real facts | L |
| 3 | **Raids and sieges set off by progress** | Valheim, 7 Days to Die, Conan Exiles, Necesse, Factorio, MineColonies | A reason for walls, gates, light and traps; tension at home, not just away | The blood moon and its spawner exist. Boss kills and age keys are natural triggers. It must be a world rule that can be turned off | L |
| 4 | **Glider, grapple and climbing** | Enshrouded, Zelda: Breath of the Wild | Vertical play on sequoias, mesas, cliffs and ruins; a fast way down | Stamina exists, and "climbing's cost waits for ladders". A hide glider fits bronze, a grapple fits iron, and the power suit's flight sits at the top | M |
| 5 | **A map you draw, and waystones** | Xaero's/JourneyMap, Waystones, Terraria pylons, Enshrouded altars | Knowing where you've been and getting home | Expedition maps and the Explorer tree are planned, and the minimap is in STYLE. It's also the base for the network atlas (round 1, L2) | M |
| 6 | **Learning by study** | Grounded, Subnautica, No Man's Sky | Turns the "knowledge" gate into play: you examine a thing to learn what it makes | The field journal and the ruins' blueprints exist or are planned. Study can fill the journal and open recipes | M |
| 7 | **A base heart** | Enshrouded, V Rising, Rust | One block that says "this is our base": protection, tribe rights, a raid target, an upgrade path | Tribes, permissions (core next) and comfort all need a base to hang on. It also gives raids and tame jobs an anchor | M |
| 8 | **Seasonal giants and weather disasters** | Don't Starve Together, Ark, Dune: Awakening, Vintage Story | Seasons you prepare for, not just read on the HUD | Real climates and seasons exist, wildfire is planned, and biome bosses are planned. A season-bound giant per biome links the two | M |
| 9 | **Light injuries and illness** | Project Zomboid, DayZ, The Long Dark | Falls and fights leave a mark; water and meat need care | Poison, bleeding and sedation exist. Greens already "resist sickness", but there's no sickness. Round 1's F13 | S-M |
| 10 | **Hunting by wind and noise** | Dune: Awakening, 7 Days to Die, The Forest | Hunting becomes a skill, using wind the game already simulates | `vitals::air` makes each day's wind, and prey already watch for hunters. The player's scent and noise are the missing half | S |
| 11 | **Mechanical power** | Vintage Story, Create | A bronze-to-steam power step before electricity: windmills, waterwheels, treadwheels | The engine has an energy contract, and PROGRESSION puts tames on treadwheels in tier 3. Wind and rivers already exist | M |
| 12 | **Boats, then vehicles** | Valheim, Raft, Create, Satisfactory | The first rungs of the transport ladder; rivers and coasts become roads | The engine has moving structures you can stand and build on. The transport ladder is planned | L |
| 13 | **Settlers, traders and contracts** | Necesse, Terraria, MineColonies, Minecraft Comes Alive, Stardew Valley | People in the world; goals that write themselves; a place to sell and buy | PROGRESSION's settlements and traders, and CRAFTING's option C (recipes from a trader) | L |
| 14 | **Ruins with puzzles; worlds that reset** | Rust, Enshrouded, Core Keeper, Abiotic Factor | Places worth the walk, and repeatable ones for the endless gear | The ruins of the ancients and dimensions as linked worlds are planned | L |
| 15 | **Darkness and dread** | Don't Starve, Vintage Story, Project Zomboid | The dark and the deep put pressure on the mind, not only the body | Nights are dark by design and a torch must light the way. Ley lines can steady you, which links magic to survival | S-M |

## 2. Already covered

These came up in nearly every game, and chiacraft has them, often deeper:

- **Temperature and clothing** (Tough As Nails, Valheim, The Long Dark): **has**. Real climates, wind, wet clothes and clo values; few games go this far.
- **Food, nutrition, cooking and spoilage** (TerraFirmaCraft, Valheim, Farmer's Delight): **has**. Real nutrients, stews, drying, smoking and salting, freshness stamps.
- **Thirst**: **has**, as water in the stomach.
- **Rested and comfort** (Valheim): **has**, but which furniture counts is still open (see "Decorating and the home").
- **Seasons with gameplay** (Don't Starve, TerraFirmaCraft, Serene Seasons): **has**. Day length, temperatures, breeding, rain and crops by season.
- **Taming, breeding, genetics, imprinting** (Ark): **has**, except imprinting and care.
- **Pets with personality** (Palworld's partner skills): **has**, as signatures: barking, gifts, a calm aura.
- **Riding**: **part**. Only the Utahraptor so far.
- **Bestiary** (Pokémon, No Man's Sky): **part**, as the field journal.
- **Crafting quality and parts** (Tinkers', Dwarf Fortress): **part**, with grades and parts in CRAFTING.
- **Skills by use** (Skyrim, Vintage Story's XSkills): **planned** in PROGRESSION.
- **Bosses summoned at altars** (Core Keeper, Valheim, Terraria): **planned**.
- **Sky events**: **has**. The blood moon, meteor showers and impacts.
- **Crafts that take real days** (TerraFirmaCraft, Vintage Story): **has**. The tanning pit and the charcoal clamp.
- **Rooms**: **has**. Shelter is a flood fill, which most games fake.
- **Death and a run back**: **has**.

## 3. The systems, one by one

### 1. Fishing, nets and traps (lacks)
Stardew Valley makes fish depend on season, weather, hour and place. That turns the calendar into
a checklist, and its fish bundles give collectors a goal
([Stardew wiki, River Fish Bundle](https://stardewvalleywiki.com/River_Fish_Bundle)). Core Keeper
ties the catch to the water's type, upgrades rods by tier, and makes fish a cooking ingredient
whose effects carry into dishes
([fishing](https://xgamingserver.com/blog/core-keeper-fishing-guide/),
[cooking](https://gameplay.tips/guides/core-keeper-comprehensive-guide-to-cooking.html)). On
Minecraft, Aquaculture 2 adds 30+ fish tied to biomes
([CurseForge](https://www.curseforge.com/minecraft/mc-mods/aquaculture)), and Primitive
Survival's traps were the 1.0M-download mod round 1 cited (F8).

Players love fishing because it's restful between fights and it rewards knowing the world.
chiacraft has the hard half already: real fish species that school, court and breed, with
populations. What's missing is a way to catch them.
- **Stone age:** a line and bone hook, and a basket trap in a stream.
- **Bronze age:** a net, and a weir across a river that fills over real days, like the tanning pit.
- **Later:** fish farms in pens (the ecology's capacity rules apply).

Catches should come from the real fish population near the hook, so overfishing a pond shows.
Skip a Stardew-style bar minigame; a bite you answer in time is enough (see section 4).

### 2. Tames at work (planned)
Palworld's hit system is work suitability: each species has jobs (mining, watering, handiwork,
transport) at levels, works stations on its own, and some boost others' work
([Palworld wiki](https://palworld.wiki.gg/wiki/Work_Suitability)). Conan Exiles' thralls staff
crafting stations and guard bases. V Rising's servants are sent on timed hunts that return loot,
and their blood quality sets how well they do
([PCGamesN](https://www.pcgamesn.com/v-rising/servants-best-how-to-get)). Once Human keeps its
Deviants in a containment unit at base, and they need rotating to stay sharp
([guide](https://theriagames.com/guide/once-human-isolated-securement-unit/)). Ark's mounts each
have a field use: one gathers metal, one carries heavy loads, one breaks stone.

Players love this because a tame stops being a pet in a pen and becomes a reason to go catch the
next species. chiacraft's species are real, so their work can come from fact:
- dogs herd and guard, and capybaras are calm workers that settle a pen;
- a raptor hunts and fetches;
- a big herbivore ploughs and pulls;
- a bird scouts or carries a letter.

**Design suggestion:** a `work` row on each species (job, level), read by stations within a base
heart's reach. Add expeditions in V Rising's style for "each player at their own pace": send a tame
out for real hours, and it comes back with goods from its habitat. Its genes (strength, speed)
should matter, which makes breeding for work worth doing.

### 3. Raids and sieges set off by progress (part: the blood moon)
- **Valheim:** raids come only after a boss kill, each tied to the boss you last beat, so the threat grows with you ([Prima](https://primagames.com/gaming/valheim-what-are-raids-and-how-to-survive-them-guide)).
- **7 Days to Die:** its blood moon sends a horde straight at you every seventh night from 22:00 to 04:00, scaled to your progress, and its frequency and spread are world settings ([7DtD wiki](https://7daystodie.wiki.gg/wiki/Blood_Moon_Horde)).
- **Conan Exiles:** the Purge fills a hidden meter from gathering, crafting and killing; surviving it gives purge-only thralls ([G-Portal](https://www.g-portal.com/wiki/en/what-is-a-purge-and-how-does-this-gameplay-mechanic-work/)).
- **Necesse:** raids skip settlements under 3 settlers, scale with their number, and come only while a team member is online ([Necesse wiki](https://necessewiki.com/Raid)).
- **Factorio:** the cleanest cause and effect. Pollution that reaches enemy nests is what they spend on attacks ([Factorio wiki](https://wiki.factorio.com/Pollution)).
- **MineColonies:** raids average one every 14 nights and can be switched off ([config](https://minecolonies.com/wiki/misc/configfile)).

Players love raids because a base finally has to *do* something. They hate them when raids come
from nowhere or wreck hours of building. Round 1 found raids sixth on Enshrouded's feedback board,
and Vintage Story's storms are the cautionary tale.

chiacraft has the blood moon's own spawner and monsters that wait at a fire's edge.
**Suggestion:** raids come from what the player does, and the world is real, so the attackers can
be too:
- wolves in a hard winter come for the livestock;
- a biome boss's kin come after its kill;
- in the steam age, industry's noise and smoke draw creatures, as Factorio's pollution does.

Keep Necesse's rules: never while you're offline, never at a base too small to matter, and a world
rule to turn raids off. They target a base heart (system 7), never a random wall.

### 4. Glider, grapple and climbing (lacks)
Enshrouded's glider and grappling hook are its main movement tools: it has no mounts, so these
plus fast travel cover the map. The glider comes early from simple materials
([PinDrop](https://pindrop.gg/enshrouded/guides/glider-grappling-hook)). In Breath of the Wild,
you can climb almost any surface on a stamina wheel. Its developers added the paraglider because
climbing wouldn't be worth it if getting down weren't easy
([Zelda Dungeon](https://www.zeldadungeon.net/wiki/Paraglider)). Critics credit the pair with
turning every cliff into a puzzle
([GamesRadar](https://gamesradar.com/the-legend-of-zelda-breath-of-the-wilds-climbing-is-the-secret-ingredient-that-finally-adds-real-adventure-to-open-world-gaming)).

Players love the feeling of a peak as a launch point. chiacraft has 60-block sequoias, mesas on
tiers, highlands and cubic chunks with no height limit, but moving through them is still
Minecraft walking. Stamina already exists. Steps up the ages:
- **Stone age:** climbing with picks or claws that drains stamina; the stamina stat would finally matter.
- **Bronze age:** a hide-and-wood glider, its range set by the day's wind from `vitals::air`. Gliding with the wind should feel different from gliding into it.
- **Iron age:** a grapple.
- **Later:** flying mounts, and the power suit's flight.

The engine needs climbing on walls (a body pose and a movement state) and glide physics.

### 5. A map you draw, and waystones (planned)
Xaero's World Map and JourneyMap are in nearly every Minecraft modpack, which shows how much
players miss a map. Waystones adds teleport stones you find and activate, and a stone at your
death spot for the run back ([CurseForge](https://www.curseforge.com/minecraft/mc-mods/waystones)).
Terraria's pylons let you teleport only between happy towns in their biome, and not during an
invasion, which ties travel to settling the land
([Terraria wiki](https://terraria.wiki.gg/wiki/Guide:NPC_Happiness)). Enshrouded's flame altars do
the same for its fast travel and push back its Shroud as you upgrade them
([wiki](https://enshrouded.wiki.gg/wiki/Flame_Altar)).

PROGRESSION plans expedition maps and an Explorer tree, and round 1 (F11) wanted "a map you draw".
**Suggestion:**
- The map fills only where you've walked, as a fog of war.
- It's an item a tribe can copy and trade, so drawing it is exploring.
- Waystones are ancient ley nodes you wake, which ties travel to the lore and the magic path; teleporting costs mana or a crystal.
- The same map grows into the network atlas of linked worlds (round 1, L2).

### 6. Learning by study (part: the journal)
- **Grounded:** a resource analyzer turns a new item into recipes and raises your Brainpower, which opens more ([Grounded wiki](https://grounded.wiki.gg/wiki/Resource_Analyzer_(Grounded)), [Xbox Wire](https://news.xbox.com/en-us/2025/07/29/getting-started-in-grounded-2/)).
- **Subnautica:** you scan wreck fragments to learn a blueprint, three for the scanner room ([wikily](https://wikily.gg/subnautica/items/basemaproom/)).
- **No Man's Sky:** you scan and name species, upload them for pay, and show them on a projector at base ([Echoes update](https://nomanssky.com/echoes-update)).

Players love finding a thing and *learning* from it. It makes every new biome a page of the
journal and not just loot.

PROGRESSION's knowledge gate is blueprints and lore pages from ruins. **Suggestion:** widen it.
- Examining a new plant, ore or creature at a study desk fills its journal page.
- A filled page opens what it makes: a new dye, a remedy, a better hide treatment.
- First discovery is the part only chiacraft can do: the first player to record a species in a world, or across linked worlds, is named on its page and can sign it with their DID.

### 7. A base heart (lacks)
- **Rust:** the tool cupboard grants building rights in its radius and holds the upkeep. Bases decay when it runs dry, so abandoned forts clear themselves ([Facepunch wiki](https://wiki.facepunch.com/rust/the_tool_cupboard)).
- **V Rising:** the castle heart runs on blood essence, sets how many servants and how much floor you can have, and the castle decays if it starves ([guide](https://xgamingserver.com/blog/v-rising-castle-building-guide/)).
- **Enshrouded:** the flame altar is fast travel, base claim and progress meter in one, upgraded with rescued survivors.

chiacraft will need this anyway:
- permissions on servers;
- tribes sharing "bases and storage, by rank";
- tame jobs needing a reach;
- raids needing a target;
- comfort needing a home.

**Suggestion:** a hearth stone you upgrade each age, with a claim radius set by its tier and tribe
ranks to build and open chests. Upkeep and decay should be only a server rule (see section 4).

### 8. Seasonal giants and weather disasters (part)
Don't Starve Together gives each season its giant: Deerclops late in winter, the Moose/Goose in
spring, Bearger in autumn. Each drops what you need for the next season, such as the eyeball for
an umbrella ([DST seasons](https://dontstarve.wiki.gg/wiki/Seasons/DST)).
- **Ark:** heatwaves and cold snaps swing a map's temperature, and one creature can only be tamed in a heatwave ([Dot Esports](https://dotesports.com/ark/news/how-to-survive-a-heat-wave-in-scorched-earth-ark-survival-ascended)).
- **Dune: Awakening:** sandstorms you shelter from in a closed building, and worms that come to footsteps on sand ([PC Gamer](https://www.pcgamer.com/spice-worms-and-water-first-details-on-the-upcoming-dune-mmos-survival-systems)).
- **Vintage Story:** temporal storms every ten days or so, growing longer over time. Players asked for a way to turn them off ([wiki](https://wiki.vintagestory.at/Temporal_Stability)).

chiacraft's seasons are real, so its disasters can be too, each from the biome's real place:
- wildfire in the savanna's dry season (planned);
- blizzards in the taiga;
- floods when a river's catchment gets a wet spell;
- heat waves on the mesas.

**Suggestions:**
- Each biome boss could appear in its season unbidden, or be easier to summon then, as DST's giants do.
- Weather can buff, too: a thunderstorm charges ley nodes for magic, and some creatures can only be tamed or found in one kind of weather, as in Ark.
- A forecast (round 1's F7) is what turns a disaster from unfair into a plan.

### 9. Light injuries and illness (lacks)
Project Zomboid tracks wounds per body part. Fractures need splints, dirty bandages bring
infection, and first aid skill speeds healing ([PZ first aid](https://pzfans.com/first-aid/)).
DayZ has cholera from unsafe water, salmonella from raw or spoiled meat, and wound infection in two
stages. Boiling water, cooking meat and clean rags prevent them
([Prima chart](https://primagames.com/tips/dayz-sickness-chart-all-diseases-and-cures)).

Players love this because each choice about water and meat matters, and a fall from a cliff is a
story, not just a number. chiacraft has the pieces:
- poison and bleeding as effects;
- nutrients;
- greens that "resist sickness";
- a freshness stamp on all food.

**Suggestion (off by default, as round 1's F13 said):**
- Spoiled meat and raw river water carry a mild sickness (nausea, slower digestion) that greens and boiling prevent.
- A hard fall can sprain a leg (slower, no sprint) until it's splinted and rested.
- Keep it to three or four conditions. Project Zomboid's full body chart is too much next to the stomach (see section 4).

### 10. Hunting by wind and noise (part: prey wariness)
- **Dune: Awakening:** its worms come to footsteps on sand: "If you walk on sand, they will come" ([PC Gamer](https://www.pcgamer.com/spice-worms-and-water-first-details-on-the-upcoming-dune-mmos-survival-systems)).
- **7 Days to Die:** a heat map adds up noise from mining, forges and fighting, and calls screamers when it fills ([7DtD wiki](https://7daystodie.wiki.gg/wiki/Heatmap)).
- **The Forest:** its cannibals watch and probe before they attack, and react to how you treat them ([Wikipedia](https://en.wikipedia.org/wiki/The_Forest_(video_game))).

chiacraft's prey already flee hunters and watch for them, and on a blood moon they watch 1.5x as
far. The world already has each day's wind. **Suggestion:**
- Prey smell the player downwind at a long range and upwind barely at all.
- Running, armour and metal tools make noise; crouching in tall grass or leaf litter hides you.
- The Survival tree's tracking perks show tracks and dung pats (which already exist) as trails to the herd.

It's small to build, makes every hunt a puzzle, and uses wind no other block game models.

### 11. Mechanical power (planned in tier 3)
Vintage Story's windmills turn axles, gears and clutches that drive a quern, a helve hammer and a
pulverizer. Its windmill gives nothing on a calm day however well it's built
([VS wiki](https://wiki.vintagestory.at/Special:MyLanguage/Mechanical_Power),
[windmill](https://wiki.vintagestory.at/index.php/Windmill)). Create built Minecraft's biggest
mod around turning power: waterwheels and windmills give stress units, too much load stalls the
line, and the same parts build moving bridges, elevators and trains
([Create guide](https://createmod.com/guides/getting-started-with-create-mod)).

Players love it because you build and watch it work. chiacraft's ages jump from muscle to steam,
and mechanical power fills bronze and iron:
- a waterwheel on a real river, which runs low in the dry season;
- a windmill whose output follows the day's wind;
- a treadwheel walked by a tame.

The engine's energy contract can carry it. Real wind and rivers make siting a mill a real choice.

### 12. Boats, then vehicles (planned)
Valheim's longship opens its ocean biomes. Raft is a whole game on a moving base whose sails only
work with or across the wind ([Wikipedia](https://en.wikipedia.org/wiki/Raft_(video_game))).
Create's trains and Satisfactory's trucks, trains and hypertubes make transport itself the
automation ([Satisfactory wiki](https://satisfactory.wiki.gg/wiki/Hyper_Tube)). Round 1 found
vehicles the top request in Unturned (57%).

The engine already has moving structures you can stand and build on, which is most of the hard
part. **Suggestion:** start the transport ladder with water:
- **Bronze age:** a dugout canoe from a felled log, which fits felling.
- **Iron age:** a sail boat built as a moving structure, its speed set by the wind's direction.
- **Steam age:** a cart behind a pack animal, as round 1's F14 suggested, then rails.

Rivers already run down valleys to the sea, so they're the first roads.

### 13. Settlers, traders and contracts (planned)
- **Necesse:** settlers who work around their house, and an elder who hands out quests per settlement ([settlement](https://necessewiki.com/Settlement), [quests](https://necessewiki.com/Quest)).
- **Terraria:** NPCs move in when you build a room, and their happiness by biome and neighbour sets prices and pylons.
- **MineColonies:** happiness from housing, food and safety ([MineColonies wiki](https://minecolonies.com/wiki/systems/happinessandsaturation)).
- **Minecraft Comes Alive:** villagers who marry and raise children; 26.4M downloads ([CurseForge](https://www.curseforge.com/minecraft/mc-mods/minecraft-comes-alive-mca)).
- **Stardew Valley:** its community-centre bundles turn collecting into a town's restoration ([Stardew wiki](https://stardewvalley.fandom.com/wiki/Bundles)).

PROGRESSION has "NPC camps and towns that trade, give contracts and sell knowledge" growing from a
trading post to a spaceport. **Suggestions:**
- Start small: a travelling trader on a pack animal who comes in the right season and buys what the local climate makes.
- Contracts from the ecology: "the village needs 20 dried fish before winter".
- Settlers who move in once you have a lit, roofed room (the room flood fill exists) and work like tames.

### 14. Ruins with puzzles; worlds that reset (planned)
Rust's monuments have hazards, keycard puzzles and a patrolling tank, and they're where players
meet ([Rust monuments](https://guildorder.com/games/rust/wiki/monuments-and-loot-tiers)).
Enshrouded's vaults are short puzzle dungeons with a chest at the end. Abiotic Factor's portal
worlds are optional pocket levels that need a key item to enter, and their resources reset twice a
week ([Abiotic Factor wiki](https://abioticfactor.wiki.gg/wiki/Portal_Worlds)). Raft's story
islands each hold the radio code to the next.

PROGRESSION plans the ancients' ruins with blueprints, relics and lore pages that point to the next
ruin, much as Raft does. **Two additions:**
- **Puzzles:** use the redstone and machines the engine has (doors, levers, ley conduits), so a ruin is solved, not just looted.
- **Resetting worlds:** small dimensions on the portal system that reset on a timer, giving the endless gear a renewable source without stripping the overworld.

### 15. Darkness and dread (lacks)
Don't Starve's sanity falls in the dark, near monsters and on bad food. At half, shadow creatures
appear, and in full darkness an unseen thing attacks
([Sanity](https://dontstarve.wiki.gg/wiki/Sanity),
[Charlie](https://dontstarve.wiki.gg/wiki/Charlie/Darkness)). Vintage Story's temporal stability
drops underground and near rifts; low stability makes the world shimmer and calls drifters
([VS wiki](https://wiki.vintagestory.at/Temporal_Stability)). Project Zomboid's moodles add
boredom and unhappiness that books and the outdoors lift
([moodles](https://pzfans.com/moodles/)).

chiacraft has decided nights stay dark and a torch must light the way; monsters already need light
6 or less. **Suggestion, kept light:**
- **Dread rises:** in the dark, deep underground, and alone at night.
- **Dread falls:** by a fire, near tames, rested, and near a ley node, which gives the magic path a survival use.
- **Effects:** high dread blurs the edge of the view, makes sounds that aren't there, and slows stamina's return, never damage.

Since animals already huddle and howl on a blood moon, the player's own nerves are the missing half
of that night.

### Smaller ones worth noting
- **Cooking buffs** (Core Keeper, Valheim): **part**. chiacraft's meals are nutrition; a few dishes could also carry a short effect, such as a spiced stew giving warmth for a winter trip. Farmer's Delight has 224M downloads ([CurseForge](https://www.curseforge.com/projects/398521)), so players want cooking depth.
- **Decorating and the home** (Valheim, Terraria): **part**. Comfort counts, but which furniture is still open. Make it Valheim's way: one point per kind of thing (bed, chair, rug, banner, trophy), so a varied home beats a stack of one item ([Valheim comfort](https://mobalytics.gg/gamebase/guides/valheim-comfort-rested-guide)). Trophies from biome bosses fit here.
- **Crafting station upgrades** (Valheim, Enshrouded): **part**. A station that grows by adding tool racks and an anvil beside it is a cheap way to show an age's progress at home.
- **Companions with orders** (Sons of the Forest's Kelvin): **has** for tames. Kelvin's notepad of plain orders such as "get logs" is a good model for the commands screen ([wiki](https://sonsoftheforest.wiki.gg/wiki/Kelvin)).
- **Mount abilities** (Palworld, Ark): **part**. Each mount should do one thing nothing else can: a raptor sprints and leaps, a big grazer carries a load, a flyer glides.
- **Pollution** (Factorio): **lacks**. In the steam and electric ages, smoke and waste could lower nearby soil fertility and drive wildlife off. The ecology loop would show it with no extra rules, as a natural brake on factories.
- **A storyteller** (RimWorld): **lacks**. Cassandra, Phoebe and Randy pace events differently and can be switched mid-game with no penalty ([RimWorld wiki](https://rimworldwiki.com/wiki/AI_Storytellers)). One world setting choosing how raids, disasters and sky events are spaced would be cheap.
- **Water quality** (DayZ, Dune): **part**. Boiling and a still fit system 9. Dune's windtraps and dew collectors fit the desert and mesa biomes ([awakening.wiki](https://awakening.wiki/Hydration)).
- **Festivals** (Stardew Valley): **lacks**. Once settlements exist, the real calendar and the meteor showers already give dates for them.

## 4. Probably skip, and why

- **Server wipes** (Rust, Once Human's seasons): chiacraft's worlds are persistent, linked and tied to identity on Chia. A wipe breaks every portal and pedigree. A server could choose it, but not the base game.
- **Full-loot PvP, offline raiding, clan wars over territory** (Rust, Conan Exiles, Dune's Deep Desert): Ronald chose "each player at their own pace", and tribes share by choice. These fit a server package with its own rules, not the base game.
- **Building decay and upkeep** (Rust): as a default it punishes the builders chiacraft wants. Keep it as an option for public servers, to clear abandoned bases.
- **Structural integrity** (Valheim, 7 Days to Die): players of block games expect free building, sequoias already need special felling, and with no height limit a support check on every block costs time on every change. Guides even disagree on what Valheim's colours mean, which shows how hard it is to read ([Mobalytics](https://mobalytics.gg/gamebase/guides/valheim-general-building-guide)).
- **A full body-part chart and blood types** (Project Zomboid, DayZ): too fiddly beside a stomach of real nutrients. Keep system 9 to three or four conditions.
- **Heavy sanity with hallucinated monsters that hurt you** (Don't Starve): it fights the calm, Valheim-spirited tone. Dread stays a pressure, not a damage source.
- **A fishing minigame bar** (Stardew Valley): it's often the most complained-about part of Stardew's fishing. A bite you answer in time keeps fishing calm.
- **Underwater bases** (Subnautica): the engine's water would need sealed rooms that stay dry, air supply and pressure. Breath exists, but it's a big job for a niche. Revisit when an ocean dimension is on the table.
- **Tower defence as its own mode**: fold traps and turrets into raids instead.
- **Item-scarcity economies** (Rust, Dune): chiacraft's trade comes from real climates (round 1, L5). Invented scarcity isn't needed.
