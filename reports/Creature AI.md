# Creature AI: how clever it is, and how to make it cleverer (11-05)

Ronald asked: "How clever is our current sort of NPC animal system? I want the logic for the AI to
be kind of clever ... I don't want it to be overly heavy because I want to be able to have lots of
mobs ... But I don't want the animals or humans to just be stupid."

The short answer: **our animals know a lot and do little with it.** Their bodies and their ecology
are simulated better than in almost any game: hunger, thirst, temperature, prey by mass, seasons,
genes, a carrying capacity per biome. Their behaviour is still a vanilla-Minecraft goal list. They
see through walls, forget everything, flee in straight lines, and herds are herds only when they
spawn. The cost is low today because the crowds are tiny (6 passive animals per player). Two of the
engine's queries grow with the square of the crowd, though, and they will hit first when the caps go up.

Part 1 reads the code. Part 2 is the research, with links. Part 3 is a plan in order, with what each
step costs per creature and what it makes visibly smarter, then a budget and how to measure it.

All costs marked "est." are estimates from reading the code. The crowd bench in step 0 is there to
replace them with numbers.

---

## Part 1: the current system

### 1.1 Architecture: a priority list, first one wins

The engine's AI is in `crates/sdk/src/ai.rs`. A kind gets a `Goals` list. Each tick, for every
entity of that kind, `Goals::tick` (ai.rs:312) first runs the **triggers** (sight, hurt, gaze,
`when`), which set **moods** (anger or fear at someone, until a tick). Then it walks the goals in
order, and **the first goal that returns true acts. The rest are skipped** (and told so via
`passed_over`). This is Minecraft's `GoalSelector` without its mutex flags: one goal at a time,
fixed priority.

The moods live in one global `Mutex<HashMap>` (ai.rs:35) and are not saved.

Every animal in the mod uses that skeleton. The simple wild creatures (`mods/animals/src/wild.rs:76-89`)
run, in this order: `animal::upkeep` (never takes the turn: rolls, ages, stats, and the body once a
second), the fear of players (`startle`, `taming::behave`), `hunt::wary`, `hunt::huddle`,
`hunt::hunt`, `breeding::behave`, `settle` (rest), `comfort::seek`, `forage::forage` (which drinks
first), and `ai::wander`.

The full creatures (wolf, jaguar, raptors, wyvern, cat, dog) use the same list plus commands,
riding and knock-out (`wolf.rs:45-59`, `raptor.rs:69-83`). Birds add flight (`birds.rs:99-109`).

Each goal keeps its own little state in a component (`HUNT: [u64; 4]`, `FORAGE: [f64; 4]`,
`DRINK`, `SEEK`, `WARY`), with "look again at tick X" timers. That gives some commitment: a
grazer walking to grass keeps walking for up to 8 s. There is no scoring anywhere. The order is
the decision.

### 1.2 Perception: distance only

- **Sight is a sphere.** `Trigger::sight` (ai.rs:115) and `seen` (ai.rs:176) test a cylinder of
  `range`, 8 up and down, shrunk by the player's `visibility` attribute. No line of sight, no view
  cone. Only `gaze` (the enderman test) raycasts.
- **Prey watch for hunters** within 10 blocks (`hunt.rs:34`, `wary` at 265), every 10 ticks.
  **Hunters look for prey** within 16 (`LOOK`), every 60 ticks. Both shrink in fog, blizzard or
  dust (`base_api::sight`). Both widen at a blood moon. Neither checks whether a wall or hill is in between.
- **Fear of players** (`taming.rs:201`, `frightened`): a player walking upright within `shy`
  blocks, or sprinting within twice that, scares it. Sneaking is safe at any distance. Shyness
  comes from the genes' temperament (`coats::temperament`, `core/src/coats.rs:1803`): 0.6x to 1.4x.
  It doesn't matter which way the animal faces, what is in between, the wind, or the noise.
- **Smell:** a hungry scavenging bird finds carrion within `needs::SCENT` (`birds.rs:210`). Every
  forager "sees and smells" dropped food within 12 (`forage.rs:45`). There are no scent trails,
  no wind direction and no noise. (`vitals::air::heading` already gives the wind's direction, so
  scent downwind is one dot product away.)
- **Memory: none.** A wild animal remembers nothing past a few-second timer. Its fear of you
  ends when `scared` reaches 0 (80 ticks, `taming.rs:44`). Shooting at a deer, or killing its
  herd-mate in front of it, changes nothing about how it treats you next time. Tames have bond
  and taming progress, and that is the only memory of a player.

### 1.3 Needs: the strong part

- **Hunger** (`animal.rs:399`, `forage.rs`): only for wild animals whose food the world grows or
  drops. Foraging starts under `HUNGRY` and stops at `FULL`. It is the hysteresis the other
  needs lack. Mouthfuls feed by how lush the plant is this season, grass gets grazed short, and dung
  drops.
- **Thirst** (`drink.rs`): by mass and diet (`needs::DRINKS`), faster out of comfort. Rain and
  wading slake it a little. A parched wild animal dies in a game day.
- **Temperature** (`comfort.rs`): the same air model as the player. Out of comfort, an animal
  samples 12 random spots within 10 blocks and walks to the best one (or beside a fire).
- **Rest:** `settle` starts at a 1-in-400 chance per tick (`wild.rs:110`) and lasts 30 to 120 s.
  It has nothing to do with the hour. `Species::active` (day, night, twilight) is filled in for
  every species from research (`core/src/species.rs`, e.g. line 743: the Velociraptor is nocturnal
  by its scleral ring), but only the journal reads it (`core/src/journal.rs:80`). No animal sleeps at night.
- **Catch-up** (`animal.rs:292`, `life::AWAY`): an area unloaded for a while has its time caught
  up when it loads (aging, births, a successor for one that died). That is a good lazy simulation.

### 1.4 Social behaviour: groups at spawn, not after

- **Herds** come from spawning in groups (`Social::Herd`, `habitats.rs`). After that there is no
  cohesion at all, except `huddle` (hunt.rs:301), which pulls a straggler back to the herd's
  middle **only in a blood moon's totality**. A herd scatters with every wander.
- **No alarm.** A deer that sees a wolf runs. Its herd-mates run only when they see the wolf
  themselves. A hurt animal (`wild.rs:90`) scares only itself.
- **Packs** (`Social::Pack`: wolves, Utahraptors): `pack_near` counts kin within 16 blocks, which
  unlocks bigger prey (`takes_with`). `rally` (hunt.rs:249) sets the same target on idle kin.
  A pack chases at 1.75x speed instead of 1.4x, gets a block more reach (the "flank bites"), and
  keeps going twice as long. The Utahraptors' `aid` (raptor.rs:107) makes kin join a fight. There
  are no roles: every wolf runs at the prey's current position.
- **Parents:** a young one follows its mother if it is 3 to 32 blocks away (`breeding.rs:425`).
  Nothing defends the young, and the mother doesn't wait.

### 1.5 Predator and prey

The ecology is careful. Prey sizes are a share of the hunter's mass from research. A kill fills
the hunter by mass (`needs::fills`). Hunters eat carcasses before they hunt. A biome has room for
hunters by its prey's yearly young (`needs::hunter_room`).

The behaviour is simple:

- **Hunter:** when hungry, it picks the nearest prey it can take within 16 blocks (`hunt.rs:206`)
  and runs at its current position every 5 ticks (`commands.rs:498`). It gives up after 30 s
  (60 s for a pack). It doesn't stalk, ambush, cut off the prey, use cover, or choose the weak, old
  or young prey. It goes for the nearest.
- **Prey:** it runs to a point 10 blocks straight away from the nearest hunter (`hunt.rs:291-293`)
  and repicks the point only when the 4-second `RUN` ends. It runs into water, cliffs and other
  hunters just as readily.
- **The wrong enemy.** A wild animal that is hurt calls `taming::hurt`, which calls `scare`, then
  `flee` (taming.rs:227-241). That runs from **the nearest player**, or in a random direction.
  So a deer bitten by a wolf runs away from you, not from the wolf. For those 4 s it also ignores
  `wary`, because `taming::behave` comes earlier in the list.
- **Players are neither prey nor threat.** Hunters leave players alone ("players for now",
  ECOLOGY part 6). Monsters (`mods/base/src/mobs.rs`) are `aggressive(16..35)`: a sight sphere, a
  chase, and a melee.

### 1.6 Pathfinding

`crates/engine/src/server/path.rs` is A* over block cells with Minecraft's rules: 8 directions
with no corner cutting, a 1-block step up, a drop of up to 3, room for the body's full width (the
big Cretaceous animals), water costs +2, and no lava.

- **How far:** at most `MAX_NODES = 600` expanded cells (path.rs:41). On open ground that reaches
  maybe 40 to 100 blocks. In a forest or cave, nearer 15 to 25. Past that it returns the reachable
  cell closest to the goal.
- **How often:** a new search when the target moved more than 2 blocks or the walk got stuck, at
  most once a second per mob (`e.age + 20`, dim.rs:2089). At most 128 searches per region per
  tick (`PATHS_PER_TICK`, dim.rs:161). The searches run in parallel (rayon `par_iter`, dim.rs:2061).
- **Cost (est.):** a 600-node search is roughly 50 to 200 µs: neighbours, closures and a hash
  map. Spread over threads, 128 a tick is fine. The danger is a crowd that all re-path at once
  (a stampede), which queues for several ticks.
- **Steering:** it follows cell centres. The path isn't smoothed, so the walk zigzags along the
  diagonals. Mobs don't avoid each other.

### 1.7 How often they think, and what it costs

- **Activity:** `mark_resting` (dim.rs:2302) rests everything more than
  `core:entity_activation_distance` blocks from every player (48 by default, config.rs:653). A
  resting mob's tick and path finding run once in `core:inactive_entity_interval` ticks (20).
  Physics still runs every tick. Mods scale timers by `tick_elapsed`. That is a real two-tier AI LOD already.
- **Staggering:** most checks run on `(now + id) % N == 0`: wary every 10, huddle every 20, hunt
  looks every 60. That is time slicing, done well.
- **Populations are small today:** `core:passive` is capped at **6 per player within 64 blocks**,
  hostile at 8 (`registry.rs:1776-1778`). Passive animals don't despawn, so explored land
  accumulates them.
- **Measured:** the BENCH line reports `mods per tick: animals X ms/N calls`. Archived runs
  (`~/Coding/chiacraft-archive/tester-tools/profile`, `runs/`) show the animals mod at **0.02 to
  0.16 ms per tick** with ordinary small crowds. No bench has ever put hundreds of creatures in
  front of it. There is **no creature-crowd bench**.

**What grows with the square of the crowd** (the scaling wall):

- `World::entities_near` / `targets_near` call `Entities::within` (`entity.rs:597`), which **scans
  every entity in the region**. There is no spatial grid.
- Each animal calls it from `wary` every 10 ticks, from `hunt`, `pack_near`, `rally`, `huddle`,
  `mate_near`, `ground_food`, and from base's systems too. So the cost is N animals × N entities.
  The dropped items count, so a field of loot makes it worse.
- At 500 animals, `wary` alone is 50 scans a tick × ~500+ entities. Est. 0.3 to 1 ms. At 2,000,
  it is about 16 times that.
- For each candidate in range, `wary` then reads several components: `ANIMAL`, then `mass`,
  which reads `GENOME`, `SIZE` and breeding growth. **Every `World::component` read allocates a
  `Vec`** (`crates/sdk/src/game.rs:317`) and crosses the C ABI.

**What spikes:**

- `drink::nearest_water` reads 33 × 33 × 7 ≈ 7,600 blocks per search (`drink.rs:22-24`, 76),
  each one an FFI `w.block` call, plus the shore checks. That is est. 0.2 to 0.5 ms per search.
  A thirsty animal searches every 10 to 20 s.
- `forage::nearest` reads 17 × 17 × 6 ≈ 1,700 blocks (forage.rs:174).
- `comfort::seek` samples 12 spots of `air_at`.
- None of these remember the answer. The pond that was there 20 s ago is searched again from scratch.

### 1.8 Rating, honestly

| | Ours | Minecraft vanilla | Ark | RDR2 animals | Rain World | TLOU / F.E.A.R. |
|---|---|---|---|---|---|---|
| Needs and ecology | **strong**: real hunger, thirst, heat, prey by mass, seasons, capacity | almost none | stats, little in the wild | mostly authored routines | food chain, dens, rain | n/a |
| Decision making | priority list | priority list + Brain (villagers, piglins) | aggro radius | authored + systemic | per-creature utility over trackers | GOAP (F.E.A.R.), designed states (TLOU) |
| Perception | spheres, no line of sight | line-of-sight checks for targets | a radius; crouch and prone shrink it | scent with wind, sound, sight | sight, noise tracker | cones, raycasts to body parts, sound |
| Memory | none | Brain memories (villagers: home, job, gossip) | none | some | last-seen tracking, per-region reputation | last known position, search |
| Groups | packs pick one target | wolves join a fight | herds, packs follow alpha | herds flee as one, predators stalk | packs, relationships | squads, flanking, callouts |
| Movement | A* 600 nodes, straight-line flight | A* | poor | animation-led | procedural bodies | cover, flanking |

**Overall:** about vanilla Minecraft's level in behaviour, well above it in what an animal knows
and needs, and above Ark. Far below RDR2, Rain World and the shooters in how animals *act* on what
they know. In one line: a deer here is the best-simulated deer in any block game, and it behaves
like a Minecraft cow with a better nose.

**What's good, worth keeping:**

- The data model: species facts from research, prey by mass, diets, comfort ranges, temperament
  from genes. These are exactly the inputs a utility system needs. Most games have to invent them.
- Commitment timers per goal, and the HUNGRY/FULL hysteresis.
- Staggered checks, the engine's resting tier, and catch-up when an area loads.
- Pack prey sizes and `rally`: the idea is right, it just needs roles.
- The forage loop changes the world (grazed grass, dung), so the ecosystem is visible.

**What's dumb, with the place:**

1. **Sees through walls** (ai.rs:176, hunt.rs:281, taming.rs:201). A deer flees a wolf behind a
   hill; a hunter targets prey in a cave below.
2. **A bitten animal flees the player, not its attacker** (taming.rs:227-241, via wild.rs:90-93),
   and that flight overrides `wary` for 4 s.
3. **Fleeing is a straight line** (hunt.rs:293), recomputed every 4 s: into lakes, off cliffs,
   into a second wolf.
4. **Chasing is pure pursuit** of the current position (commands.rs:498-500), with a path at most
   once a second. There is no lead, no cut-off, and the pack doesn't spread out.
5. **Herds don't exist after spawning** (hunt.rs:301 is the only cohesion, and only at a blood moon).
   There is no alarm call, no sentinel, and nothing protects the young.
6. **Strict first-wins order.** A resting animal keeps resting while it starves (`settle` comes
   before `forage`; `rest` lasts up to 120 s, wild.rs:110-124). An animal in love ignores thirst
   (`breeding` before `forage`). A hungry hunter never weighs risk against hunger.
7. **Day and night don't matter**, though every species has `active` filled in from research.
8. **No memory**: no last-seen position, no "this player hurt me", no "my kin died here", no
   "the pond is east". It searches the blocks again every time (drink.rs:76).
9. **Hunters pick the nearest prey**, not the weakest or the young.
10. **Engine queries grow with the square of the crowd** (`entity.rs:597`), and each component read
    allocates (game.rs:317).

---

## Part 2: the research

### Decision making

**Utility AI.** Each candidate action scores itself from a few inputs ("considerations"), each
through a response curve (linear, logistic, quadratic) normalised to 0..1. The scores multiply,
and the highest wins, with a bonus for the action already running so it doesn't flicker. Dave
Mark's **Infinite Axis Utility System** is the standard form: data-driven, any number of
considerations per action, and a compensation factor so actions with many considerations aren't
punished ([IAUS at gameai.com](https://gameai.com/iaus.php);
[GDC 2015 "Building a Better Centaur: AI at Massive Scale", Mark and Lewis](https://www.gdcvault.com/play/1021848);
[GDC 2012 "Embracing the Dark Art of Mathematical Modeling in AI"](https://gdcvault.com/play/1015421/Embracing-the-Dark-Art-of);
[overview](https://en.wikipedia.org/wiki/Utility_system)). The Sims is utility AI over needs:
objects advertise how much they satisfy each motive, and a Sim picks the best one for its current
needs ([Zubek, "Needs-Based AI", Game Programming Gems 8, draft](http://robert.zubek.net/publications/Needs-based-AI-draft.pdf)).

It costs a handful of multiplies per action, and it fits us best: our needs are already numbers 0..1.

**GOAP.** Jeff Orkin's planner for F.E.A.R.: actions with preconditions and effects, and A* over
world states finds a sequence that reaches a goal. The famous result is that the soldiers' state
machine was only three states (Goto, Animate, UseSmartObject), with the planner doing the rest
([Orkin, "Three States and a Plan", GDC 2006, slides](https://pages.cs.wisc.edu/~dyer/cs540/handouts/gdc2006_orkin_jeff_fear.pdf);
[GDC Vault](https://gdcvault.com/play/1013282/Three-States-and-a-Plan)). It is powerful for
multi-step jobs, and costly when replanned often for hundreds of agents.

**HTN planning.** It decomposes a task into authored sub-tasks ("hunt" → "stalk, rush, bite").
It is cheaper and easier to control than GOAP, because designers write the recipes.
[Guerrilla used it in Killzone 2's bots](https://www.guerrilla-games.com/media/News/Files/VUA07_Verweij_Hierarchically-Layered-MP-Bot_System.pdf)
and in Horizon Zero Dawn's machines. HZD combined it with utility scoring and with **group
agents**: a herd is an agent that holds shared knowledge and hands roles to its members
([The AI of Horizon Zero Dawn](https://www.guerrilla-games.com/read/the-ai-of-horizon-zero-dawn);
[HTN Planning in Decima](https://guerrilla-games.com/read/htn-planning-in-decima);
[GDC 2018, Berteling](https://gdcvault.com/play/1024912/Beyond-Killzone-Creating-New-AI)).

**Behaviour trees.** Selectors and sequences, made popular by Halo 2
([Isla, "Handling Complexity in the Halo 2 AI"](https://gamedeveloper.com/programming/gdc-2005-proceeding-handling-complexity-in-the-i-halo-2-i-ai)).
Our goal list is the flattest possible tree. A tree adds little over utility for animals, and is
useful for scripted NPC jobs.

**Minecraft's Brain** (villagers, piglins, axolotls): *sensors* refresh *memories* on a timer,
and *tasks* read the shared memories, so expensive sensing is done once per mob and not once per
goal. It also has an activity *schedule* ([Brain class, Yarn docs](https://maven.fabricmc.net/docs/yarn-1.21.3+build.1/net/minecraft/entity/ai/brain/Brain.html)).
That is the shape we want: sense once, decide from memory.

### Animals in games

**Rain World** is the closest thing to what Ronald describes. The creatures exist for their own
sake, not as obstacles. They roam on their own and keep living off-screen, and designers steer
them mostly through level shape and "room attractiveness"
([Game Developer: Rain World's ecosystem](https://gamedeveloper.com/design/crafting-the-complex-chaotic-ecosystem-of-i-rain-world-i-)).
Technically, each creature keeps **relationships** (Eats, Afraid, Attacks, Pack, Ignores, each
0..1) toward the creatures it has noticed. It runs **trackers** (threat, prey, noise, rain,
injury, aggression, friend), and its relationships move with **reputation per region and
community** and with **personality traits** (aggression, bravery, dominance, energy, nervousness,
sympathy) ([Rain World wiki: Behavior](https://rainworld.miraheze.org/wiki/Behavior)). The
animation is procedural, so a simple decision looks alive
([GDC 2016 animation talk](https://gamedeveloper.com/art/video-animating-i-rain-world-i-and-its-many-squishy-stretchy-creatures)).

**Red Dead Redemption 2:** about 200 species, each modelled on real behaviour. Herds move, grizzlies
bluff-charge, predators wait and stalk, and carcasses draw scavengers. Hunting uses scent carried
by the wind ([GDC 2020, O'Dwyer](https://gdconf.com/article/learn-how-rockstar-breathed-life-into-the-wildlife-of-red-dead-redemption-2-at-gdc/);
[The Conversation via WEF](https://www.weforum.org/stories/emerging-technologies/red-dead-redemption-2-virtual-ecology-is-making-game-worlds-eerily-like-our-own)).
Much of its convincingness is animation and authored routines, which is costly content and not costly CPU.

**Ark** has no design docs. Players report a per-species aggro radius, shrunk by going prone or
crouching, and larger around nests
([Steam discussion](https://steamcommunity.com/app/346110/discussions/0/617335934135515454)).
That is about our level. **Far Cry 2** left predators out to keep balance simple
([GamesRadar](https://gamesradar.com/far-cry-2-exclusive-dev-team-interview/4)). Later games
made animals a systemic hazard that attacks enemy camps too. The lesson: **hunters that attack
anyone, not just the player, make stories.**

### Perception and memory

- **Cones, raycasts and alertness.** *Splinter Cell: Blacklist* used a main cone, a wider
  peripheral box, and raycasts to several bones on the player's body. It built awareness up over
  time instead of switching it on ([Walsh, GDC 2014 "Modeling AI Perception and Awareness"](https://www.gdcvault.com/play/1020436/Modeling-AI-Perception-and-Awareness);
  [Game AI Pro 2 chapter](https://www.oreilly.com/library/view/game-ai-pro/9781482254792/K23980_C028.xhtml)).
  The talk's key point: what matters is what *the player* thinks the guard could see. *The Last of
  Us* raycasts to the head, or the chest when crouched
  ([TLOU Human Enemy AI, GDC](https://gdcvault.com/play/1020338/contactUs)).
- **Two brains.** *Alien: Isolation*'s director knows where the player is and only *hints* to the
  alien. A "menace gauge" makes it back off ([The Perfect Organism](https://www.gamedeveloper.com/design/the-perfect-organism-the-ai-of-alien-isolation)).
  It is cheap and convincing, and useful for monsters and night hunts.
- **Occupancy maps:** a probability map of where the target may be now, spread out over time,
  is what lets a guard search sensibly ([Isla, "Third Eye Crime", AIIDE 2013](https://dblp1.uni-trier.de/rec/conf/aiide/Isla13.html)).
- **Scent maps:** Brogue lays the player's scent on every tile they can see, stamped with a global
  counter instead of decaying every tile, and a tracker climbs the gradient. It costs one cheap
  write per visible tile and nothing to age ([Brogue wiki: Scent](https://brogue.wiki/wiki/Scent);
  [RogueBasin: tracking by scent and sound](https://chizaruu.github.io/roguebasin/tracking_by_scent_and_sound)).
- **Influence maps:** a coarse grid where agents add threat, food or territory and it spreads.
  Many agents read one shared map instead of each querying the world. Dave Mark's modular version
  is built for many agents at once ([Game AI Pro 2, ch. 30, PDF](https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter30_Modular_Tactical_Influence_Maps.pdf);
  [GDC 2018](https://gdcvault.com/play/1025243/Spatial-Knowledge-Representation-through-Modular)).
- **Remembering the player.** Shadow of Mordor's Nemesis system: an enemy that beat you or
  escaped remembers it, rises in rank and taunts you. Small saved records make huge stories
  ([GDC 2018, Hoge](https://gdcvault.com/play/1025150/Helping-Players-Hate-(or-Love))). Rain
  World's reputation is the animal version.

### Movement and crowds

- **Steering and boids:** separation, alignment and cohesion from a few neighbours give flocks and
  herds with no planner ([Reynolds 1987](https://www.cs.princeton.edu/courses/archive/spr01/cs598b/papers/reynolds87.pdf)).
  Couzin's model adds zones (repulsion, then alignment, then attraction) and a blind spot behind.
  Small changes to the zones flip a group between swarm, mill and march
  ([Couzin et al. 2002](https://kops.uni-konstanz.de/entities/publication/1adea651-afe4-45f3-826b-429d36d9c7dd/full)).
  It costs a few vector sums over 6 to 8 neighbours, and needs a spatial grid to find them.
- **Flow fields:** one field per goal for a whole crowd (Supreme Commander 2). It is tiled so it
  is built only where needed ([Emerson, Game AI Pro ch. 23, PDF](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter23_Crowd_Pathfinding_and_Steering_Using_Flow_Field_Tiles.pdf)).
  For us, a herd going to water shares one search.

### Doing less work: AI level of detail

- **Assassin's Creed Unity** had about 10,000 crowd NPCs from 40 real AIs and 120 full models.
  Far ones ran a simplified brain and were swapped for full ones near the player
  ([Cournoyer, GDC 2015 "Massive Crowd on Assassin's Creed Unity: AI Recycling"](https://gdcvault.com/play/1022141/Massive-Crowd-on-Assassin-s)).
- **The LOD Trader:** treat AI detail as a budget. Each agent's level comes from how noticeable a
  mistake would be at its distance and visibility, and the budget goes where it shows
  ([Sunshine-Hill, Game AI Pro ch. 14](https://gameaipro.com/)).
- Rain World and our own `catch_up` show the far end: unseen creatures are simulated abstractly,
  or caught up on load.

### Personality, emotion and needs

- **Dwarf Fortress:** about 50 personality facets and 30 values per creature, hidden 0..100.
  Needs come from the facets, and memories shift the facets over time
  ([Adams, "Characterization and Emergent Narrative in Dwarf Fortress"](https://wileywiggins.com//documents/dorf/Adams%20-%20Characterization%20and%20Emergent%20Narrative%20in%20Dwarf%20F.pdf);
  [wiki: Personality facet](https://dwarffortresswiki.org/index.php/DF2014:Personality_facet)).
  Personality is free at runtime: a few weights on the utility curves.
- **The Sims:** needs decay, objects advertise, and the best one is chosen
  ([Zubek](http://robert.zubek.net/publications/Needs-based-AI-draft.pdf)).

### Academic work on believable animals and ecosystems

- **Tu and Terzopoulos, "Artificial Fishes"** (SIGGRAPH 1994): perception with a 300° field of view
  limited by occlusion and distance, plus intentions from hunger, libido and fear. It was the first
  convincing animal agent ([SIGGRAPH summary](https://education.siggraph.org/static/HyperGraph/animation/art_life/fish.htm)).
- **Blumberg, "Action-Selection in Hamsterdam: Lessons from Ethology"** (SAB 1994) and the MIT
  Synthetic Characters group: ethology-style action selection. Behaviours compete by motivation
  times releasing stimulus, with inhibition and **fatigue/boredom so one behaviour doesn't hog
  the animal**, and the winner keeps a "hysteresis" bonus
  ([MIT Characters publications](https://characters.media.mit.edu/publications.html)). This is
  the academic root of utility AI for animals, and it fixes our "rests while starving" fault.
- **EcoSim** (Gras et al., *Artificial Life* 2009): hundreds of thousands of individual predators
  and prey, each choosing actions from a small fuzzy cognitive map (fear, hunger, distance to
  food, mates and predators), with realistic population patterns emerging
  ([paper](https://strathprints.strath.ac.uk/90261/1/Gras-etal-AL-2009-An-individual-based-evolving-predator-prey-ecosystem-simulation.pdf)).
  It shows that cheap weighted maps per agent scale to huge counts.
- **Field ecology** gives the group behaviours to copy:
  - the **selfish herd**: each animal moves toward the middle to put others between itself and the
    hunter ([Hamilton 1971](https://en.wikipedia.org/wiki/Selfish_herd_theory));
  - **vigilance and sentinels**: in a bigger group each animal looks up less, yet the group spots a
    hunter sooner ([many-eyes effect](https://en.wikipedia.org/wiki/Vigilance_(behavioural_ecology)));
  - **alarm calls** pass the warning on faster than sight does.

---

## Part 3: the plan for chiacraft

The principle: **sense once, remember, decide by score, act through what we have.** Each step
stands alone and is visible in the game. Costs are per creature, averaged over ticks, for an
active one (near a player). The engine's resting tier covers the far ones already.

### 0. A crowd bench first (cost: none)

Today there are no numbers for crowds. We need a testkit script, run by Tester 2 on the PC (never on the Mac):

1. Use a fresh world on flat open land, with `core:mob_spawning 0`, spectator mode, and a fixed time.
2. Spawn a mix: deer and reindeer in herds, hares, foxes, wolves in packs, ravens, a few water
   ponds. Use 100, 250, 500 and 1,000, all within 40 blocks so they stay active, and a second set
   of 1,000 beyond 48 blocks for the resting tier.
3. Run `bench 10` at each size. Read `server avg/max`, `mods per tick: animals X ms/N calls` and
   `entities N` from the BENCH line.
4. Run twice more: once hungry (`/tk attr` food low), so foraging, drinking and hunting run, and
   once idle.

Add a small per-goal timer in the animals mod behind a debug setting: nanoseconds summed per goal
name, logged once a BENCH. Then each step below is checked against the same lines. **Every step
ships with its bench delta.**

### 1. Shared sensing and a spatial grid (cost goes *down*)

- **Engine (a Weft bug-level fix, with a line in ENGINE_CHANGES):** give `Entities::within` a
  cell index, 16-block columns rebuilt or updated each tick. A query then costs O(entities in
  the nearby cells) instead of O(all). Est. 60 to 100 lines.
- **Or mod-side, if the engine stays untouched:** the animals mod builds its own grid once per
  tick, at the first upkeep of the tick: one `entities_in` over the loaded area plus one
  `w.entity` per entity, about 0.1 ms at 500.
- **A per-animal "senses" record**, refreshed every 10 ticks, staggered, and stored in one
  component: nearest threat, nearest prey, kin count and centre, nearest player and their stance.
  All goals read it instead of querying. This is Minecraft's Brain sensors.
- **Cache `mass`** in that record. Add a non-allocating component read to the SDK (a fixed-size
  stack buffer instead of `vec!`).
- **Remember the water and food spots** found (last 2 to 3 per animal, with a tick). Search the
  blocks only when a remembered one fails.

It makes nothing visibly smarter, but it makes everything after it affordable. Expect `wary`,
`hunt` and pack calls to fall from O(N) to O(k), and water searches to drop by 5 to 10x.

### 2. A utility layer over the goals (est. 1 µs a decision, every 10 to 20 ticks: ~0.1 µs a tick)

Keep the hard priorities where they are right: upkeep, knocked out, ridden, commands, and
fleeing danger. Replace the fixed order of the **needs band** (drink, forage, comfort, rest,
breed, wander, later socialise and sentinel) with a scored choice:

- Each action is a product of curves over inputs the mod already has: thirst, food share,
  comfort off by °C, time of day against `Species::active`, season, love, how far the resource is
  (from the memory in step 1), and danger near (from the senses).
- The running action gets a +10 to 25% bonus (hysteresis), and its score decays the longer it
  runs (Blumberg's fatigue). A rest ends when thirst climbs.
- The curves are data per species family in `animals_core`, next to the needs numbers, so the
  sheets can show them.

This visibly fixes: animals resting while starving, lovers ignoring thirst, a deer grazing 2
blocks from a wolf it hasn't "decided" to fear. It is the one change that makes everything else
read as intent.

### 3. Perception that can be fooled (est. 1 to 3 µs a tick)

- **Line of sight** for the senses' candidates only: one raycast to the eye, budgeted (at most N
  raycasts a tick per region, round-robin), and cached for 10 ticks.
- **A view cone** for most animals: wide for prey (about 300°, Tu and Terzopoulos's fish), narrow
  for hunters (about 120°). A blind spot behind.
- **Awareness that builds up** (0..1, Splinter Cell style): fast when close and in the open, slow
  in the dark, in tall grass, or sneaking. The animal's head turns to look first. The player sees
  the "it noticed me" moment before it bolts.
- **Noise events:** sprinting, a block breaking, combat, an explosion, a bark. Each is pushed to a
  per-region ring buffer (position, loudness, tick, who) and read through the grid. Sound goes
  round walls cheaply if loudness falls off by distance and drops through solid ground (one
  raycast count).
- **Scent downwind:** an animal smells a player or hunter when they are within `scent_range ×
  wind factor`, and the wind factor is the dot product with `vitals::air::heading`. Approaching
  upwind becomes the hunter's skill, as in RDR2.
- **Fixes:** flee from **whoever hurt it** (the hurt event's `source`), not the nearest player.

The visible result: sneaking up behind a deer, from downwind, through grass, works. Walls and
hills hide you. A deer looks up, stares, then runs.

### 4. Memory (est. a few bytes; ~0.2 µs a tick)

- **Per animal**, a fixed array of 4 memories: (who or what, where, when, how it felt: fear,
  anger, trust), saved with the animal. It covers last-seen threat positions (it searches or
  avoids there), "this player hurt me", and "this player fed me".
- **Per area**, a coarse **danger map**: an 8 × 8 block cell in a small hash per region, where a
  kill or a hunt adds fear that decays over game days. Grazers weigh it in their utility, so
  they avoid the clearing where wolves killed. Hunters weigh it the other way, around a kill site
  or a waterhole.
- **Reputation per species and area** (Rain World): a player who hunts deer in a valley finds its
  deer warier: shy distance × (1 + bad reputation), sharing with the herd. A player who feeds them
  calms them. It decays over weeks. Cost: one number per (species, region).

The visible result: animals learn you. A hunted valley gets harder, and a fed herd lets you close.

### 5. Herds and packs with roles (est. 2 to 4 µs a tick for herd members)

- **Cohesion:** Reynolds and Couzin over the 6 nearest kin from the grid every 5 to 10 ticks,
  only while not fleeing or foraging far. Herds drift together and turn together.
- **Alarm:** the first to notice a hunter (or you) calls, through a voice and the noise buffer.
  Kin within 24 blocks go to full awareness and flee *together*, away from the threat and toward
  the herd's middle (the selfish herd), not straight off a cliff.
- **Smarter flight:** pick the best of 6 to 8 directions away, scored by open ground, no water or
  drop, away from other threats, toward kin and cover. Hold the choice for the run.
- **Sentinel:** in a resting or grazing herd, one adult is "on watch": it looks up and scans with a
  wider awareness. The role rotates every 20 to 40 s. You see heads up in the herd.
- **The young:** young keep to the middle and mothers stay between a threat and their calf. Big
  plant eaters (aurochs-sized, Protoceratops, the ankylosaur) **turn and face** a lone hunter
  instead of running, or charge it.
- **Packs:** the leader picks the prey by a score (young, old, hurt, stragglers: off the herd's
  middle). The others take **flank slots**: target position plus an offset of 60 to 120° to each
  side, ahead of the prey's run. Their pursuit **leads** the target (aim at pos + vel × t).
  One wolf drives while two cut off.
- **Ambushers** (jaguar, caiman): stalk low within cover until `burst range`, then charge with
  a short time limit. The caiman waits submerged at a waterhole from the step 1 memory.

The visible result: real herd panic, a pack that splits around a deer, a jaguar that appears from
the grass, and a mother that turns on you.

### 6. Personality from genes (cost: none)

`Tp` (temperament) already sets shyness. Extend it to 2 or 3 traits from the genes (bold/shy,
curious/aloof, and aggressive for hunters), inherited and shown in the journal. They scale utility
weights: a bold deer lets you closer and investigates noises; a shy one sentinels more; a bold
wolf leads the hunt. Rain World and Dwarf Fortress show this costs nothing at runtime and makes
every animal a little different. Breeding then selects for temperament, which ties into
`evolution.rs`.

### 7. Day routines (cost: none)

Read `Species::active` in the utility: a diurnal animal's rest climbs at night, and it seeks
shelter (cover, the herd's middle) to sleep. Crepuscular animals peak at dawn and dusk, and the
nocturnal raptors and owls hunt at night. Add "go to water at dawn and dusk" for grazers. Hunters
learn waterholes are where prey come (step 4). The world then has a rhythm: herds at the river at
dusk, wolves out at night.

### 8. AI LOD tiers (cost goes down)

| Tier | Where | What runs |
|---|---|---|
| A, full | within 32 blocks or in the player's view (facing within ~70° and LOS on the sample) | everything, senses every 10 ticks |
| B, coarse | 32-48 blocks, out of view | senses every 40 ticks, no LOS (use the danger map), decisions every 40 |
| C, resting | past 48 (the engine's) | upkeep every 20 ticks, no new paths unless fleeing |
| D, unloaded | not loaded | `catch_up` on load (have) |

Use the LOD Trader's rule: spend detail where a mistake would be seen. Group plans (step 9) let
a whole herd in tier B cost about one animal.

### 9. Shared group plans (cost goes down with group size)

A herd or pack gets a **group record**, as HZD's group agents do: leader, centre, the current
plan (graze here, go to water, migrate, flee), and the target. Only the leader searches for water,
paths and threats in tier B. Followers steer to the leader's path with their cohesion offsets,
which is one A* for the herd. Migration (ECOLOGY's "Later": herds toward wet biomes in the dry
season) becomes a group plan with a far target. A tiled flow field to a waterhole is an option
if herds get big.

### 10. Small movement fixes (cost: small)

- String-pull paths (skip cells that have line of sight to a later one) so the walk doesn't zigzag.
- Separation from kin so a pack doesn't stack on one block.
- Re-path a chaser sooner when its target turns hard: the engine's 1-second repath is long for a sprinting deer.

### The budget

At 20 ticks a second a tick is 50 ms, and the server has to fit chunk work, physics, sync and
base's systems in it. A target that leaves room:

| | Count | Budget a tick | Per creature-tick |
|---|---|---|---|
| Active animals (tier A/B) | 500 | **≤ 3 ms** | ≤ 6 µs |
| Resting animals (tier C) | 2,000 | ≤ 0.5 ms | ≤ 0.25 µs |
| Townsfolk (later) | 100 | ≤ 1.5 ms | ≤ 15 µs |
| Path searches | ≤ 128/tick | parallel, ≤ 2 ms wall | |

After steps 1 to 9, an active animal is est. about 5 µs a tick: upkeep ~1, senses ~1.5, line of
sight ~0.5, utility ~0.2, cohesion ~0.5 and its action ~1. That fits 500 active animals in about 2.5 ms, *if* step 1 removes the square-of-the-crowd queries.
Without step 1, 500 is already near the budget, and 1,000 is over it.

**How to measure it:** the step 0 bench, at each size, before and after each step. The pass line
is the BENCH `mods per tick: animals ≤ 3 ms` and `server max` with no tick over 50 ms at 500
active. The per-goal timer shows which step regressed.

### People: dwarves, elves and humans in living towns

`notes/PROGRESSION.md` wants towns with NPCs going about their day, traders, smiths, reputation
per people, and guards and ambushes. The same parts carry over, with one more layer:

- **Utility for choosing, HTN-lite for doing.** A villager scores what to do now: work, eat,
  sleep, socialise, flee, guard. The chosen job runs a short authored recipe ("smith: fetch ore
  → smelt → forge → sell"), as HZD and Killzone did. GOAP is not needed. Recipes are cheaper and
  easier to debug.
- **Schedules:** time blocks per role and people (Minecraft's villagers, The Sims). Dwarves keep
  hall hours underground, elves are about at dusk. These are inputs to the utility, not a hard
  script, so a fire or a raid overrides them.
- **Smart objects:** workstations, beds, taverns and markets *advertise* what they satisfy and
  hold the animations, as in The Sims. Adding a forge adds behaviour without new AI code. This is
  also how F.E.A.R. kept its state machine at three states.
- **Social memory:** each NPC keeps a few memories, as the animals do (step 4), about players and
  other NPCs: "helped me", "stole", "killed my kin". A **town record**, the group agent, holds
  reputation per player and spreads gossip. A memory shared at the tavern raises or lowers the
  town's reputation the next day. That is Nemesis-scale storytelling at the cost of a few saved
  records, and it is what PROGRESSION's "reputation per people" needs.
- **LOD for towns:** a town the player isn't in runs as numbers (stock, food, who is where by
  schedule), like Rain World off-screen. When the player arrives, NPCs are placed where their
  schedules say ("alibis") and run in full. Only the town near the player costs per NPC.
- **Budget:** 100 townsfolk active in one town at ≤ 15 µs each is 1.5 ms. Most of that is
  perception and talk, since jobs mostly wait on animations.

### Order and what each buys

| Step | Cost per active creature | What visibly changes |
|---|---|---|
| 0. Crowd bench | 0 | numbers instead of guesses |
| 1. Grid, senses, memory of spots | cost down | room for 500+ animals |
| 2. Utility over the needs band | ~0.1 µs | no more resting while starving; animals look like they decide |
| 3. LOS, cones, awareness, noise, scent and wind; flee the attacker | ~1-3 µs | stalking works; hills and grass hide you; "it noticed me" |
| 4. Memories, danger map, reputation | ~0.2 µs | animals learn you and avoid kill sites |
| 5. Herd and pack roles | ~2-4 µs | herd panic, sentinels, flanking wolves, ambushing jaguars, protective mothers |
| 6. Personality from genes | 0 | every animal a bit different, bred temperaments |
| 7. Day routines | 0 | dusk at the river, wolves by night |
| 8. AI LOD tiers | cost down | crowds far away cost little |
| 9. Group plans, migration | cost down | herds that travel; one path per herd |
| 10. Path smoothing, separation | small | smoother, less robotic movement |
| People | ≤ 15 µs per NPC | living towns that remember you |

Steps 1 and 2 are the core. Step 1 makes a big crowd affordable, and step 2 makes behaviour read
as intent. Steps 3 and 5 are what players will notice and talk about. Each one is a branch of its
own, with its row in `notes/overview/pieces.json` and `notes/SYSTEMS.md` and its bench delta. The
curves and constants go in `animals_core`, per the working rules, so the sheets can show them.
