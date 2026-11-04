# Animals: taming, stats, breeding (design)

Status: design draft, 2026-11-01, from Ronald's answers to a 48-question interview. Steps 1-6
of the build order (the core, passive taming, commands, signatures, knock-out taming and the
raptor, riding) are built; see "Built so far" at the end. "Proposed" marks numbers and
details that still need his yes. Reference: ARK, made to
fit chiacraft (worlds owned by their players, linked by portals, the chain only where it adds
trust; see `notes/PLAN.md`).

## The decisions

| Topic | Decision |
|---|---|
| Feel | Deep, like ARK: wild levels, stat points, breeding lines, mutations. Achievement, some grind |
| Tameable | Almost every creature; bosses and a few others wild-only |
| Purpose (now) | Riding (raptor), following and fighting (dogs); more roles later |
| Players | Solo and multiplayer equally |
| Taming method | Per species: knock out and feed (big, dangerous), passive feeding (small, calm), trust over time (some) |
| Taming time | Scales with size and level: a minute or two for a small one, 5-15 min for a raptor |
| Taming food | Anything in its diet; its favorite foods are faster; crafted kibble is best |
| Taming effectiveness | Yes: damage, waking, poor food lower it; a clean tame gives bonus levels |
| Wild young | Steal eggs (parents defend them), find wild babies, raise them |
| Taming risks | Predators attack a sleeping or feeding animal; it wakes or flees; its pack defends it |
| Ownership | The tamer's; shareable with a tribe (command, ride, breed) |
| Tame cap | A world setting, none by default |
| Stats | Health, stamina, food, damage, speed, carry weight, oxygen, torpor, fortitude |
| Wild levels | Random, like ARK |
| Growth | XP from fighting, working, riding; a stat point to place each level |
| Numbers | All visible: stat values and points per stat |
| Commands | Follow, stay, wander; stances; whistles for groups; guard an area |
| Riding | A saddle crafted per species |
| Inventory | Each tame has one, limited by carry weight |
| Jobs | Gather, hunt and fetch, produce goods: designed now, built later |
| Hunger | They eat from their inventory or a trough; starving hurts and lowers bond; long neglect and they go wild (not die) |
| Bond | Matters: obedience, small stat bonuses, unlocks |
| Death | Permanent |
| Storage | A crafted capsule (like ARK's cryopod) |
| Mating | Hard and earned: adult, in heat, healthy, happy, bonded pair, the right place |
| Birth | Eggs (raptors, birds) and live births (mammals) |
| Babies | ARK-style imprinting |
| Growth time | A world setting |
| Inheritance | Each stat whole from one parent (55% the better one) |
| Mutations | ARK-style, with limits |
| Colors | Inherited coats and patterns, rare ones prized, can mutate |
| Pedigree | Full family tree; inbreeding has a cost |
| Heat | Cycles per species |
| Breeding conditions | Adult and healthy; happy and bonded; the pair has courted; the right place |
| Aging | Young, adult, elder: fertile years; no death of old age |
| Litters | Per species |
| Portals | World owners decide whether tames come through |
| Chain | Achievements now; an optional pedigree NFT later. Never in the game loop |
| Trading | Hand over a capsule; the pedigree keeps the breeder |
| PvP | World owners decide; protected by default |
| UI | An animal screen; a radial command menu and whistle keys; a spyglass to scout; a creature journal |
| Chihuahua | Alarm and tracker |
| Cat | Lucky finds |
| Capybara | Calm aura |
| Raptor | Mount; pack hunter |
| Order | This design first, then code |

## Species are data

Every species is a definition (a Rust table, `mods/animals/core/src/species.rs`), so adding an
animal is mostly numbers. **Every species' settings are on one page, `mods/animals/SPECIES.md`**,
written from that table (`node mods/animals/web/sheet.mjs`; the website tests fail while it is
out of date), so a change shows there:

- **Its place in the tree of life**, as biology names it: family (with the groups above it:
  a class and order for mammals, clades for dinosaurs), subfamily, and scientific name (genus and
  species): the chihuahua is Canis familiaris, Mammalia › Carnivora › Canidae › Caninae. The
  animal screen and the spyglass show it. **Only one species breeds together** (`same_species`):
  a house cat (Felis catus) and a lion (Panthera leo) share a family, not a genus or species,
  and would be two animals; Velociraptor mongoliensis and V. osmolskae share a genus and are two
  species that don't interbreed. Below the species: breeds (domestic only, made by people) and,
  for everything, the individual's genes (coat colors and patterns: a tabby is a pattern, not a
  breed).
- **Breeds**: a species can have breeds (the dog: the chihuahua so far). A breed is its own
  entity kind (`animals:chihuahua`), model and saddle, with the species' rules, genes and
  signature; its gene frequencies are its own (`genetics/dog.json`, `breeds`). Breeds of one
  species breed together; their puppies are mixed unless both parents are the same breed.
- **Body**: size class (small, medium, large, huge), model, sexes, life stages.
- **Diet**: carnivore, herbivore, omnivore; the foods it eats with their food value and taming
  value; its favorite foods; its kibble tier.
- **Taming**: method (passive, knockout, trust), base taming time, torpor (knockout only),
  fleeing and aggression.
- **Stats**: a base value and per-level increments (wild and tamed) for each of the nine stats.
- **Abilities**: its signature ability (below) and its jobs.
- **Riding**: rideable, saddle recipe, the level that unlocks the saddle.
- **Breeding**: as real facts (`Biology`: its mass, gestation or incubation, time to grow up,
  lifespan, time between litters, litter size, whether it lives alone, in a pack or a herd, and
  its rarity); the game's numbers follow from them (Balance, under Built so far). Still to come:
  heat cycle and season, courtship time, the place it needs (nest, den, burrow; biome;
  temperature).
- **Looks**: color regions and the colors each can take, patterns, rare colors.

## Stats and levels

Nine stats: **health**, **stamina** (running, attacking, jumping when ridden), **food** (how
much it can eat before starving), **damage**, **speed**, **carry weight**, **oxygen**
(underwater), **torpor** (how much it takes to knock out; only goes down, never placed by
players) and **fortitude** (heat, cold, poison resistance).

**Wild levels.** A wild animal spawns with a level, 1 to 150 (proposed, a world setting; ARK's
difficulty), most of them low: proposed weights 1-30 common, 31-90 uncommon, 91-150 rare. Each
level is one **wild point** placed on a random stat (never torpor alone). Its stats are known
from its level and its points, so two level-100 raptors differ: one with 30 points in speed is a
different animal from one with 30 in health.

**A stat's value** (proposed, ARK's shape):

```
value = base * (1 + wild_points * wild_step) * (1 + tamed_points * tamed_step) * (1 + bonus)
```

`wild_step` and `tamed_step` are per species and stat (about 0.2 health, 0.05 damage, 0.01
speed); `bonus` is the imprint bonus (up to +20%) plus the bond bonus (up to +5%).

**Taming bonus.** A tame gains `floor(level * effectiveness / 2)` bonus levels, placed like wild
points. A clean tame of a level-100 animal becomes level 150.

**Growth after taming.** Tames earn XP by fighting, working, being ridden and being near their
owner (slowly). Each level up gives a **tamed point** the owner places in the animal screen.
Proposed: up to 75 tamed points (a world setting). Players see every number: the value, wild
points and tamed points per stat.

## Taming

Every method ends the same way: the taming bar fills, the animal becomes the tamer's (and their
tribe's, if shared), with its level plus the taming bonus. Effectiveness starts at 100%.

### Passive (chihuahua, cat, capybara)

- Approach slowly (sneaking); running, fighting or hurting it nearby scares it, and it flees.
- When it's calm it shows that it would take food (a speech bubble with what it wants). Offer it.
  Each feeding fills the taming bar and starts a wait before the next (proposed 10-30 s).
- Feeding when it isn't asking, or something it dislikes, lowers effectiveness and can scare it.
- Small animals take 2-5 feedings; a high level takes more.

### Knock out and feed (raptor, big animals)

- **Knock out**: hit it with blunt weapons, clubs or tranquilizer darts and arrows to raise its
  torpor. Once torpor passes its maximum it falls unconscious. Damage taken while awake doesn't
  lower effectiveness; damage after it's out does.
- **Keep it under**: torpor drains over time. Feed it sleepy berries (or a sedative) to keep it
  up. If torpor reaches zero it wakes, angry, and the tame fails.
- **Feed**: put food in its inventory. It eats on its own every so often; each bite fills the
  taming bar. Favorite food and kibble fill faster and keep effectiveness higher. Effectiveness
  drops with each hit it takes and with every bite of poor food.
- **Risks**: a sleeping animal draws predators; its pack comes to defend it (raptors); the player
  defends it.
- Proposed time: a level-1 raptor about 5 minutes on kibble, a level-150 one about 15.

### Trust over time

For shy or proud species (later: wolves, horses): visit it, feed it, stay near it across in-game
days; a trust meter fills; fail by hurting or scaring it.

### Eggs and wild babies

- **Eggs**: wild nests hold eggs; take one and its parents chase you. Incubate it (the right
  temperature) and the baby that hatches is yours, imprinting on the first player who cares for
  it.
- **Wild babies**: a baby near its mother can be passively tamed fast, but the mother defends it.

### Kibble

Crafted food that tames faster than anything (proposed: tiers by the egg it's made from, each
species preferring one tier). Recipe: an egg, meat or a vegetable, a fiber, water, cooked in a
pot. Gives eggs a purpose and links species together (raptors lay the eggs other species'
kibble needs: proposed).

## Bond, needs and life

**Bond** (0-100): rises by feeding, petting, fighting beside you, riding, imprinting; falls by
starving, being hurt by its owner, long absence. Effects: obedience (a low-bond tame ignores
some commands and wanders), up to +5% stats at 100, unlocks (riding needs bond 25 on top of the
saddle; tricks later), breeding needs bond 50.

**Hunger**: tames eat from their own inventory, then from a trough nearby (a block that feeds
tames in range). Starving hurts slowly and lowers bond. At bond 0 and starving for a while
(proposed 3 in-game days) it **goes wild**: it leaves, unowned, with its stats. It never dies of
hunger.

**Life stages**: baby, juvenile, adolescent, adult, elder. Only adults breed; elders are less
fertile but give more XP to others nearby (wise), and never die of age. Proposed: elder after a
long time (many in-game weeks; a world setting).

**Death is permanent.** No revival. Capsules keep tames safe while you travel; protected worlds
keep others from killing them.

## Commands, riding and inventory

**Commands** (radial menu on the animal; whistle keys for every tame in range):

- **Follow** (at a set distance), **stay**, **wander** (around where you left it).
- **Stances**: passive (never fights, flees), neutral (fights back), aggressive (attacks anything
  hostile nearby), attack my target (follows your attack).
- **Whistles**: all follow, all stay, all attack my target, all passive, all come here.
- **Guard**: patrol or hold an area (a radius around a point, or your base); attack intruders by
  stance.

**Riding**: each rideable species has a saddle, unlocked at a player level (proposed) and
crafted. Saddles have quality and armor. Ridden, it uses its stamina and speed; it can sprint,
jump and attack. **Needs engine work**: Weft has no riding (mounts, passengers, rider input,
camera) yet. That is the biggest engine piece in this design.

**Inventory**: every tame has one, limited by its carry weight; overloaded, it slows down and
then can't move. It can pick up items and eat from it.

**Jobs** (designed now, built later):

| Job | What it does | Who |
|---|---|---|
| Gather | Harvest plants, wood, stone or meat with a species bonus | Big tames later |
| Hunt and fetch | Bring back game, dropped items | Dogs |
| Produce | Eggs, wool, milk, fertilizer over time | Livestock, birds |

## Breeding

Breeding is hard and earned.

**Who can breed.** A male and a female of the same species, both:

- **Adult** and not elder (elders: low fertility), **healthy** (health over 75%), **well fed**,
  not recovering from a birth.
- **Happy and bonded**: bond 50 or more with their owner (or tribe), and safe (no hostile in range
  for a while).
- **A courting pair**: they spend time near each other (proposed: a pair bond meter that fills
  while they're together and both content; some species pick a partner, and refuse others once
  paired).
- **In the right place**: the species' nest, den or burrow (a placeable block), in a biome and
  temperature it likes.
- **The female in heat.**

**Heat.** Adult females come into heat on a cycle per species, shown by behavior and particles:

| Species | Heat lasts | Every | Notes (proposed) |
|---|---|---|---|
| Chihuahua | 1 in-game day | 6 in-game days | |
| Cat | 1 day | 3 days | Often, small litters |
| Capybara | 1 day | 8 days | |
| Raptor | 2 days | once a season | Spring only |

**Mating.** When everything holds, the pair mates (an animation), then a cooldown for both.

**Birth.**

- **Mammals** carry the baby for a gestation time (proposed: chihuahua and cat 2 in-game days,
  capybara 4) shown on the mother; then the litter is born at the den.
- **Eggs**: the raptor lays a clutch in its nest. Eggs hatch after an incubation time if kept in a
  temperature range (proposed: warm: near fire or in warm biomes; too cold or hot and they don't
  progress; much too far and they die). Parents can sit on them.
- **Litters** per species (proposed): chihuahua 1-4, cat 2-5, capybara 2-6, raptor 2-4 eggs.
  Recovery before the female can breed again: chihuahua and cat 4 days, capybara 6, raptor a
  season.

## Babies and imprinting

Babies go baby, juvenile, adolescent, adult; growth time is a world setting (proposed default:
about a real hour for small species, several for big ones).

- Babies need food (from a trough, the mother, or you) and die only if hurt (permanent death);
  starving babies stop growing.
- **Imprinting**: now and then (proposed: every in-game half day) a baby asks for care: feed it a
  certain food, cuddle it, walk it a distance. Each request done raises imprint. Fully imprinted
  (100%), it grows into an adult with up to +20% to its stats and a strong bond with that player
  (+5% more when that player rides it: proposed). Missed requests are lost, not punished.

## Genetics

**Stats.** For each stat, the baby takes the wild points of one parent whole: the parent with more
points in it 55% of the time (proposed, ARK's), the other 45%. Tamed points and imprint aren't
inherited. A breeder combines the best stats of different animals over generations.

**Mutations.** At each birth, three rolls of about 2.5% each (proposed). A mutation adds 2 wild
points to a random stat and changes one color region to a new color. Each animal carries a
mutation count from its mother's line and its father's line. A parent whose line has 20 or more
mutations can't pass on more (proposed, ARK's limit), so breeders manage lines.

**Colors.** Coats come from real coat genetics (`genetics/`, `src/coats.rs`): loci with alleles
in dominance order, X-linked and quantitative traits, one allele of each locus from each parent,
mutations; see "Looks" under "Built so far".

**Evolution.** How common each variant is changes where the animals live, as in real populations
(small changes within a species, not new species; one in-game day a generation, so a habitat's
animals change over weeks). Every species has a gene pool per place (a cell of 256 blocks and its
habitat: grassland, forest, taiga, snow, desert, mountains); wild animals spawn from it. Each
generation: natural selection (a coat that stands out from the habitat's ground leaves fewer
young, by the species' `camouflage`: prey and ambush hunters most, dogs least), drift (a
population of 200), mutation, and migration from the neighboring places. Players change it: a
wild animal that dies (hunted or otherwise) or is tamed takes its genes out of its place's pool,
a released tame puts its genes in (feral dogs). Proposed next: escaped tames (neglected ones going
wild), founding breeds (a line that breeds true can be named a breed with its own frequencies),
selection on inherited stats and climate (bigger bodies and thicker fur in the cold), and, much
later, isolated pools drifting into subspecies.

**Pedigree.** Every animal knows its mother, father, their parents and so on, its generation,
its mutation counts, who tamed its wild ancestors and who bred it. **Inbreeding**: parents who
share recent ancestors have lower fertility and a chance of a small stat penalty in the baby
(proposed: the inbreeding coefficient from shared ancestors in the last 4 generations).

## The four animals

| | Chihuahua | Cat | Capybara | Raptor |
|---|---|---|---|---|
| Size | Small | Small | Medium | Medium |
| Diet | Carnivore | Carnivore | Herbivore | Carnivore |
| Favorite food | Cooked meat, bones | Fish | Melon, sugar cane | Raw prime meat |
| Taming | Passive | Passive (shy) | Passive (easy) | Knock out |
| Tame time (proposed) | 1-3 min | 2-4 min | 1-2 min | 5-15 min |
| Signature | Alarm and tracker | Lucky finds | Calm aura | Mount, pack hunter |
| Rideable | No | No | No (later?) | No (too small; the Utahraptor is the mount) |
| Fights | Yes, quick bites | Small prey | No | Yes, leaps |
| Breeding place | Den | Den (indoors, a bed) | Burrow near water | Nest, warm |
| Birth | Litter 1-4 | Litter 2-5 | Litter 2-6 | Clutch 2-4 eggs |
| Heat | Every 6 days | Every 3 days | Every 8 days | Once a spring |

**Signatures** (proposed details):

- **Chihuahua, alarm and tracker**: barks when a hostile comes within range (more range with
  level), and the bark marks it for you for a few seconds. **Track**: give it something an animal
  dropped (or a player's item) and it leads you toward that kind or person. Later: sniffs out
  buried things.
- **Cat, lucky finds**: now and then (more often with bond) it brings you a gift: string, a
  feather, a rare item from a loot table that grows with its level; on rare occasions something
  special.
- **Capybara, calm aura**: animals within range calm down: wild ones flee less and tolerate you,
  passive tames go faster, scared tames settle; tames resting near it recover bond. The aura
  grows with level and bond. The capybara famously gets along with everything.
- **Raptor, mount and pack hunter**: rideable with a saddle: fast, jumps high, a leap attack.
  Wild raptors hunt in packs and come to each other's aid; tamed raptors near each other fight
  better together (a small pack bonus: proposed).

## Players, tribes and worlds

**Ownership.** A tame belongs to its tamer; it can be shared with a **tribe** (a group of players
in a world; proposed: tribes are a world feature, later linked to DIDs) so members can command,
ride and breed it.

**World settings** (the owner decides, chiacraft's rule), with defaults:

| Setting | Default |
|---|---|
| Tame cap per player or tribe | None |
| Wild level range | 1-150 |
| Taming speed, breeding interval, growth speed | 1x |
| Tames from other worlds through portals | Owner's choice per world or link (proposed default: off) |
| Highest level of a visiting tame | No limit |
| Others can hurt or steal your tames (PvP) | No (protected) |

Death is permanent everywhere (Ronald's decision, not a setting).

**Capsules.** A crafted item that holds one tame, with everything about it (stats, pedigree,
inventory, bond). Release it to place it out again. Used to travel, through portals where allowed,
and to trade: handing over the capsule moves ownership; the pedigree keeps the breeder's name.

**Portals.** A tame comes through only if the destination allows it (and within its level cap);
otherwise the capsule stays sealed there and the animal comes back with you.

## On chain

Following `notes/PLAN.md` ("the chain only where it adds trust"):

- **Now: achievements** anchored to the player's DID, for example the first tame, the first
  tame of each species, a level-150 tame, the first mutation, a 10-generation line, a rare color,
  a fully imprinted adult, every species in the journal.
- **Later: a pedigree certificate**, an optional NFT a breeder mints for a prized animal (its
  stats, colors, mutation counts, parents, breeder), so its line can be proven anywhere.
- **Never** in the game loop: taming, breeding and stats are all off chain.

## What the player sees

- **Animal screen** (sneak and use): name (rename), species, sex, age stage, level and XP, every
  stat with its value and wild/tamed points (place points here), bond, imprint, hunger,
  inventory, color regions, mutation counts, pedigree (opens a family tree), owner and tribe,
  stance and command.
- **Radial menu** (hold a key on an animal): follow, stay, wander, stance, guard, ride, feed,
  open inventory, allow mating, rename.
- **Whistle keys** for every tame in range.
- **Spyglass**: a crafted item; looking through it at a wild animal shows its species, sex,
  level and, at a higher tier (proposed), its stat points and colors.
- **Creature journal**: an entry per species, filled in as you meet, tame and breed it: diet and
  favorite foods, how to tame it, where it lives, its heat and breeding needs, the colors and
  mutations you've seen, your best one of each.
- **Above the animal**: its name; a heat, pregnancy or imprint-request icon when it matters.

## What it needs from the engine

Most of this fits in the `animals` mod with Weft's SDK as it is: components for every animal's
state (saved), entity inventories, custom screens and menus, attack and damage, attribute
modifiers (bond and aura bonuses), biome and time lookups, looks for poses. Engine work (proposed,
general features worth offering upstream):

1. **Riding**: mounts, passengers, rider input and camera. The biggest piece.
2. **One-off animations over the pose**: bark, lick, mate, eat while sitting (see the earlier
   engine notes in `notes/HANDOFF.md`).
3. **A run animation picked by speed** (done: a model's `run` takes over from `walk` between 3 and
   4.5 blocks a second; both are written as offsets from the idle, which Weft plays under them).
4. **A scale per kind**: babies grow smoothly instead of a model file per size.
5. Projectiles for tranquilizer darts (if Weft's aren't enough).

## Building it (proposed order)

1. **Core**: species data, the animal state (level, points, sex, age, owner, bond, hunger), stats
   applied to entities, the animal screen, the spyglass. Wild levels on spawn.
2. **Taming**: passive (chihuahua, cat, capybara), effectiveness, bonus levels, favorite foods,
   kibble.
3. **Commands**: follow, stay, wander, stances, whistles, guard; tribes.
4. **Signatures**: alarm and tracker, lucky finds, calm aura.
5. **Knockout taming and the raptor**: torpor, sedatives, clubs, darts; packs.
6. **Riding** (engine) and the raptor's saddle.
7. **Breeding**: heat, courtship, dens and nests, gestation, eggs and incubation, litters.
8. **Babies**: growth, imprinting; **genetics**: inheritance, mutations, colors, pedigree.
9. **Capsules**, portals, trading; achievements on chain.
10. **Jobs**, the journal, the pedigree NFT.

## Places, food and sedation (design, 2026-11-02)

Ronald: "certain animals will only spawn in certain biomes; raptors in their own dino
dimension; food sources tied to taming and to how animals' stats change; a realistic way to
make narcotics. When does it get too complex?"

**When it's too much.** A new idea stays only if the player can see it (the animal's screen, the
spyglass, the world), can act on it (it changes where to hunt, what to feed, whom to breed), it
feeds a system we have (habitat, genes and evolution, stats, food, the species' facts) instead of
starting another, and it says itself in one sentence ("raptors from the red desert are paler and
smaller"). Depth over complexity: few rules that combine. Out by this test: each food shifting
particular stats (a food x stat x species table nobody can see or plan around).

**One hub: the place.** A biome's climate (temperature, humidity, which the engine has) and what
grows there; everything below reads from it.

- **Where each lives**: a species' habitats are facts in its row (biome tags, and a dimension),
  and spawning follows them (the engine spawns by biome tag already).
- **The dino dimension** (decided: dinosaurs and wyverns; the overworld keeps today's animals):
  its own biomes after the places these animals lived: arid dunes and red beds (Velociraptor:
  the Djadokhta Formation, Mongolia), a semi-arid floodplain with conifers and ferns
  (Utahraptor: the Cedar Mountain Formation, Utah), fern and conifer forests, and high crags for
  wyverns. Same systems, its own data.
- **Climate shapes them** (evolution, already: bigger in the cold, Bergmann; camouflage): add
  shorter ears, tails and limbs in the cold (Allen's rule, on the build genes' bones) and darker
  coats where it's humid (Gloger's rule).
- **Food** (decided: wild animals eat in the world): a diet is food categories (grass, leaves,
  fruit, seeds, insects, meat, fish, carrion), with its favorites; every food item and food
  source belongs to one. Herbivores graze grass and browse leaves, carnivores hunt smaller
  animals; a biome's food sets how many it holds and how well fed they are.
- **Diet changes an animal through its condition** (decided: one number): well fed on its foods
  it reaches the size and strength its genes allow and grows up on time; poorly fed it stays
  smaller and weaker, growing slowly. Shown on its screen. Generations change through the local
  gene pool, as now.
- **Sedation by dose and weight** (decided): as wildlife capture does it, a dart's drug acts
  after a delay and the dose needed scales with body mass (mg/kg), which every species has as a
  fact; too much for a small one can kill it, an antidote wakes one. Made from real plants:
  opium poppy (the strong one; etorphine is of its family), datura or henbane (strong, risky),
  valerian root (mild, to keep one under). No sleeping berries.

**Order**: 1. habitats as species facts and spawning by them; 2. the dino dimension and its
biomes, with the dinosaurs and wyverns moved there; 3. food categories, plants and items, wild
grazing and hunting, condition; 4. sedatives from plants by dose; 5. Allen's and Gloger's rules.

**1. Habitats** (built 2026-11-02; Ronald: "some animals have multiple habitats, some many;
following realism; we'll create the new biomes"). Each species' row lists its biomes, each with
how common it is there against where it is commonest (`Species::habitats`), and spawning follows
them (`src/habitats.rs`): a rule per species and biome, at its rarity's chance there
(`Rarity::spawn_chance`: common 2%, uncommon 0.8%, rare 0.4%, legendary 0.06% of attempts that
pick it), in groups by its social life (`Social::group`: alone 1, pack 2-3, herd 3-5), by day
on grass, snow or sand. The habitats, from where each lives or lived:

- chihuahua: plains, savanna, desert, forests (Chihuahua, Mexico: dry country and people; not
  the cold); cat: plains, forests, savanna, desert, a little taiga (the African wildcat of desert
  and savanna edges; feral nearly everywhere but the deep cold); capybara: swamp, savanna, a
  little forest (the llanos' savannas, flooded grasslands and swamps).
- The dinosaurs and wyverns: in the Cretaceous since (below).

Two new overworld biomes in base: **savanna** (warm and dry, between the desert and the forests;
yellow grass, few trees; 6% of the land) and **swamp** (mild and wet lowland near sea level; dark
olive grass and water; 1.6%). Before them hot and wet land barely existed (desert 0.5%), so the
capybara, raptors and Utahraptor would have had nowhere. `/animal habitats [radius]` surveys the
land round the player from the world's generator: each biome's share and nearest spot, and how
much of the land each species lives on. Still to come: standing water in swamps (a terrain
feature), and a capybara keeping near water.

**2. The Cretaceous** (built 2026-11-02; Ronald: a built gateway, "The Cretaceous", mostly real
plants). A dimension (`animals:cretaceous`, `src/cretaceous.rs`) warm everywhere, with five
biomes of its own and the sea: **red dunes** (red sand over red sandstone, wind-blown dunes: the
Djadokhta Formation's), **scrubland** (dry, open, a few conifers), **floodplain** (low and wet:
ferns and horsetails; the Cedar Mountain Formation's floodplain, whose soils swelled and cracked
with wet and dry seasons), **fern forest** (conifers over ferns) and **crags** (high rock). Of its
land about a quarter scrubland, a sixth each dunes and fern forest, a tenth crags and floodplain.
Mostly real: grass as in the overworld, ferns and horsetails over it, spruce for the conifers;
no flowers (a fern where base would put one) and no cacti (red sand isn't `sand`). Who lives
there: Velociraptor on the dunes and scrubland, V. osmolskae on the dunes, the Utahraptor on the
floodplain, scrubland and in the forest, wyverns on the crags and a third as often elsewhere. In
the game Utahraptors turned up on the floodplain within two minutes.

**The gateway** (`src/gateway.rs`). Amber (fossil resin, from `amber_ore` in the rock: three
small seams a cube below y 64 in the overworld, six in the Cretaceous) makes amber blocks (four
each). An upright frame of them round an opening 2-21 wide and 3-21 tall, lit with a torch,
fills with the gateway; two seconds in it and you are in the other world at the same x and z, on
the ground, out of a gateway within 24 blocks there or a new one built in front of you (2 by 3,
on an amber floor), so the way back is always there. Tested both ways from the overworld
(`mods/animals/tests/gateway.txt`): lit, across, a gateway built there, back out beside the
first. Not yet: tame animals coming along, a gateway going out when its frame breaks.

Sources: [canid immobilization](https://pmc.ncbi.nlm.nih.gov/articles/PMC3141559),
[muskoxen](https://openpolar.no/Record/ftpubmed:oai:pubmedcentral.nih.gov:5645868),
[black rhinoceros darting](https://rhinoresourcecenter.com/wp-content/uploads/2006/12/1652709686.pdf),
[diet and carotenoids in birds](https://cibnor.repositorioinstitucional.mx/jspui/handle/1001/2353),
[depth and complexity](https://giantbrain.co.uk/2022/07/14/the-complexity-of-depth/).

## Open questions

- Tribes: a world feature only, or tied to player DIDs across worlds?
- Is the in-game day/season long enough for heat cycles to feel right (Weft's day length)?
- Kibble: how many tiers, and which species' eggs make which?
- Capsule: durability, a cost to use, how many a player can carry?
- Should elders eventually pass a trait to babies (like a "wise" line)?
- Cat and capybara: anything beyond their signatures (the cat was "idk yet")?

## Built so far

**Step 1, the core** (2026-11-01, `mods/animals/src`):

- `species.rs`: the four species as rows: size, diet, taming method, the nine stats' base values,
  wild and tamed steps (one shared table for now: health 0.2/0.1, damage 0.05/0.04, speed
  0.01/0.015...), hunger rate.
- `animal.rs`: the `animal` component (saved; sent to whoever has its screen open): species,
  sex, life stage, bond, imprint, command, wild and tamed points per stat, unspent points, food,
  XP. A new animal rolls its level (60% 1-30, 30% 31-90, 10% 91-150) and places a point per level
  on a random stat. Health and speed are attribute modifiers (`animals:stats`, applied again
  after loading); damage is a step of `chain::ATTACK_DAMAGE`; the rest are kept and shown. Tames
  use food at their species' rate (meat feeds a chihuahua); nothing happens at 0 yet. The owner
  is the `owner` component (a player key) on all four now, not only the dog.
- `screen.rs`: the animal screen (sneak-use your own, or anyone's in creative): heading, sex,
  stage, tamed or wild, health, food, bond, a table of every stat (value, wild, tamed) with a
  button to place an unspent point (not on torpor; up to 75). The scout screen (species, level,
  sex, stage, size, diet, how it is tamed) through the spyglass: use it on an animal up to 64
  blocks away (a ray against the animals' boxes, stopped by blocks), or right click one with it.
  `/animal [show|tame|points N|wild]` (game masters) for testing.

**Step 2, passive taming** (2026-11-01, `taming.rs`, diets in `species.rs`):

- Each species has a diet (what it eats, its favorites, food points per item) and how many
  feedings of plain food tame one at level 1 (chihuahua 4, cat 5, capybara 3; a level-150 one
  takes 2.5 times as many). Plain food counts 1, a favorite 1.5, kibble 2, and what it asked
  for a quarter more. No fish or melons exist yet, so the cat's favorite is chicken and the
  capybara's apples and sugar cane.
- A wild one asks when a player is within 5 blocks: a speech bubble with the food above it
  (`textures/particle/want_*.png`, made from base's item icons), for 20 seconds, standing
  still; after each feeding it waits 10-30 seconds before asking again.
- Mistakes: feeding it when it isn't asking (it eats, a third of the time it runs off) or
  offering food it dislikes (another species' food) cost 10% effectiveness; hurting it 15% and
  it runs off. Shyness: within `shy` blocks a player walking upright (not sneaking) scares it,
  running twice as far (cat 6, chihuahua 3, capybara never).
- Tamed: the player's, bond 25, `floor(level * effectiveness / 2)` bonus levels as wild points
  on random stats (not torpor). The chihuahua's old bone taming is gone (bones are a favorite).
- **Kibble** (`animals:kibble`): an egg, raw meat or an apple/sugar cane, wheat and a water
  bucket make four; every species takes it. Tiers by egg and a cooking pot come later.
- The scout screen shows the favorite food, the taming bar and effectiveness, and what it asks
  for. `/animal here` brings the animal in front of you (testing).

**Step 3, commands and tribes** (2026-11-01, `commands.rs`):

- Commands (`Animal::command`): **follow** (keeps 3 blocks away; catches up from far off),
  **stay** (at a point, going back there), **wander** (within 12 blocks of where it was left),
  **guard** (an area of 12 blocks around a point). A new tame stays where it was tamed.
- Stances (`orders` component): **passive** (never fights, runs from harm), **neutral** (fights
  back and defends its owner), **aggressive** (also attacks monsters within 10 blocks),
  **assist** (also attacks what its owner hits). Guarding, a tame that fights attacks monsters in
  the area. Fighting species: chihuahua (2 damage), cat (1), raptor (4); the capybara never
  fights (always passive). Damage is the kind's times the damage stat; the engine paces bites.
- On the animal screen: the orders line and buttons for every command and stance, and Tribe
  (share it with your tribe, or stop). Plain right click: food it eats feeds a hungry tame (+2
  bond), anything else switches follow and stay (the chihuahua sits when staying).
- Whistles, to every tame in 32 blocks you may command: J all follow, K all stay, H all come here
  (they gather around you and stay), G attack what you look at, B all passive.
- Tribes (on players, saved): `/tribe create <name>` (a random id and the name), `/tribeinvite
  <player>`, `/tribe join` (the invite), `/tribe leave`, `/tribe info`. Members command, whistle
  to and open the screen of tames shared with the tribe; points are still the owner's.
- Resting animals (cat, capybara, raptor) get up to follow or fight (`wake`).

Checked in the game with the chihuahua: the buttons, following, an aggressive dog biting a
zombie, the stay whistle. Not yet tried: tribes with two players, guarding, the other animals'
commands.

**Step 4, signatures** (2026-11-01, `signatures.rs`):

- **Chihuahua alarm**: a tame barks (`sounds/chihuahua/bark1-3.wav`, synthesized yips) at the
  nearest monster within 12 blocks (up to 27 with level), at most every two seconds and once per
  monster, and marks it with a red "!" for five seconds.
- **Chihuahua tracking**: given something an animal drops (right click with leather, a feather,
  wool, string, gunpowder...; or the animal screen's "Sniff out" with it in hand, for meat and
  bones it would eat), it leads its owner to the nearest such animal within 96 blocks, waiting
  when it is 8 blocks ahead, and barks and marks it when there. Two minutes at most; it gives up
  (a puff) if there is none. Tracking players comes later.
- **Cat lucky finds**: every 5-10 minutes (half that at full bond) a tame walks to its owner
  (within 24 blocks) and drops a gift from loot tables `animals:cat_gift_common` (string,
  feathers...), `_uncommon` (eggs, chicken, leather...), `_rare` (raw iron and gold, ingots,
  apples) and `_precious` (diamonds): rare 4% rising to 16.5% with level and bond, precious 0.2%
  to 0.7% with level. Other mods can add to the tables.
- **Capybara calm aura**: 6 blocks, up to 20 with level and bond, from any capybara (wild ones
  too): shy animals don't run from you, passive taming goes 25% faster, scared animals settle
  (a tame's running off stops), tames staying there gain a point of bond about every minute.
  Pale motes drift around it.
- Every screen shows the species' signature; `/animal gift` makes a cat's next gift come now.

Checked in the game: the mark over a zombie, a dog leading to a sheep and waiting, three gifts
from a cat, a shy cat that stays beside a capybara (and runs without one).

**Step 5, knock-out taming and the raptor** (2026-11-01, `knockout.rs`, `raptor.rs`):

- **Torpor**: a player's hit adds 5 bare-handed, 20 with a **club** (`animals:club`, a log on a
  stick); a **tranquilizer dart** (`animals:tranquilizer_dart`, an arrow and a spider eye make
  four; thrown with use) stings for 1 and adds 60. Awake, torpor wears off in two minutes. Past
  the torpor stat the animal falls unconscious (its resting pose, "z"s above it).
- **Keeping it under**: out, torpor drains in four minutes; a **sedative** (`animals:sedative`,
  a spider eye, sugar and seeds make two; right click it in) adds 40%. If torpor runs out it
  wakes: the taming is lost, food left in its mouth falls out, and it attacks the nearest player.
- **Feeding**: right click it with food it eats: the stack goes in its mouth (more of the same
  can be added). A bite a minute; each fills the bar like a passive feeding (kibble 2, favorite
  1.5, plain 1, over 10 feedings at level 1, 2.5 times at 150), and costs 2% effectiveness
  plain, 1% favorite, none for kibble. A hit while it's out costs 5%. Tamed by the nearest
  player, with bonus levels.
- **Angry**: a wild raptor hurt while awake goes for whoever did it for 30 seconds, and every
  wild raptor within 16 blocks joins in. Now and then (about every five minutes) a pack hunts a
  chicken, sheep or pig nearby.
- **Pack bonus**: a tamed raptor hits 10% harder for each other tamed raptor within 8 blocks, up
  to three.
- The scout screen shows torpor and what it is eating; `/animal ko` and `/animal bite` (testing).

Checked in the game: six darts knocking out a level-48 raptor, a level-30 one tamed on kibble
(15 bonus levels), three raptors attacking together when one was darted. Not yet: sedatives,
waking up, hunting, the club, the pack bonus. A proper inventory for food (a menu) instead of
the stack in its mouth could come with tame inventories.

**Step 6, riding** (2026-11-01, engine and `riding.rs`):

- **Engine** (general, worth offering to Weft upstream): `World::mount(rider, mount, seat)`,
  `dismount`, `mount_of`, `rider_of`, `keys`; a `core:mount` field tells clients who rides what;
  the move message carries the player's held keys (`abi::held_keys`); a seated pose
  (`Pose::Sit`, `abi::pose::SIT`: legs forward, arms forward, the hips on the seat, eyes 0.97
  above it). A riding client sits on its mount as drawn (no walking, no view bobbing) and the
  server takes its seat on the mount as the server has it; riders and mounts don't push each
  other; a dead or removed mount leaves its rider on foot; non-player riders are seated by the
  server. Not saved: everyone is on foot after a restart.
- **Mod**: a species with a `seat` (the Utahraptor, 1.55 blocks) can be saddled with its saddle
  (`animals:utahraptor_saddle`: three leather and two string), and its owner or a tribe member gets
  on with an empty hand once it is grown and its bond is 25 or more. Ridden it walks where the
  rider steers (WASD), facing where they look while walking, runs 1.7 times as fast sprinting,
  jumps half again as high; sneaking gets off beside it.

Checked in the game: getting on, riding toward a landmark, the seated pose from the front and
from the saddle, getting off. Not yet: other players seeing a rider (multiplayer), the saddle
on the model, stamina while ridden, attacking from the saddle. A mount still turns its head and
body to look at the rider standing still.

**The Utahraptor** (2026-11-01, `utahraptor.rs`, `source/utahraptor`): the Velociraptor is too
small to ride at its real size (turkey-sized), so the mount is a new species, the Utahraptor:
1.5 m at the hip, about 4.7 m long, built with the skill from research (Carpenter's emu-coated,
vulture-winged, alligator-eyed restoration) on the Velociraptor's skeleton: heavier everywhere,
big slit-pupiled eyes, shaggy locks over the thighs, broad wing feathers, three-toed feet with
the raised sickle claw, and six coats to choose from in the viewer (umber, emu, desert, forest,
ash, rufous). Large, carnivore, tamed by knocking out (torpor 700: about a dozen darts; 18
feedings at level 1), base health 60, damage 10; alone or in pairs, rarer than Velociraptors;
hunts sheep, pigs and cows now and then, and the others of its kind join a fight. Checked in the
game: standing beside a Velociraptor, resting, ridden (front view, from the saddle, moving).

**Looks, part 1: coat genetics and painting, in the viewer** (2026-11-01; decided with Ronald:
real coat genetics, a paint screen, paint is not inherited, the viewer first, then the game):

- Every creature's build writes layers next to its texture (`layers/`, skill `layers.py`): coat
  or fixed, light, underside, position, where white, points, a mask, extremities and patterns
  show. Gene files per species in `mods/animals/genetics/` (see its README); phenotype rules and
  the renderer in the skill's `coats.js`.
- Dog (the chihuahua is a breed: `dog.json`, its frequencies under `breeds`): E (red, mask), K (solid black, brindle), A (fawn/sable, black and tan, recessive
  black), B (chocolate), D (blue, lilac), S (white: a little, or parti), M (merle), red intensity
  and sable shading. Cat: O on the X (red, cream; tortoiseshell only in females), B (chocolate,
  cinnamon), D, A (tabby or solid), mackerel/classic/spotted/ticked, S (bicolor, van), W (all
  white), colorpoint, warmth. Capybara: shade, redness, belly, rare leucism. Raptors: pigment,
  pattern (bars, bands, rosettes, speckle, plain), saddle, melanism, leucism, darkness.
- Babies get one allele of each locus from each parent (a son's X from his mother), quantitative
  traits drift a little, 1% of alleles mutate: babies look like their parents, hidden recessives
  surprise.
- Paint (any color, pink included): coat, underside, markings and points colors, and a stencil
  (stripes, spots, patches); "Wash off" removes it. Not inherited.
- The viewer: Natural, Random, Mother, Father, Breed, the paint row, the genotype under the
  description. The chihuahua's eye picker and the Utahraptor's fixed coats are gone (genes now).
- The chihuahua is grouped under dog: species "dog", breed "chihuahua" (`species.rs` `breed`,
  the code in `dog.rs`, the source in `source/dog/chihuahua`); its screen reads "Chihuahua (dog)".
  The next dog breed is a new entity, model and row with `name: "dog"`.
- Taxonomy (2026-11-01): every species row has its family (`species::Family`, with the groups
  above it), subfamily and binomial; breeding is by species. A second Velociraptor species,
  V. osmolskae (named 2008 from skull bones found at Bayan Mandahu, Inner Mongolia; a 2013 study
  puts it nearer Linheraptor, so it may leave the genus): the raptor's body and behavior, a paler
  sand coat (`source/raptor` paints both), its own gene file (`osmolskae.json`), spawning on
  desert sand. Its own pack: the pack bonus and aid count only its species.

**Looks, part 2: genes in the game** (2026-11-01). Every animal has a genome (`coats.rs`,
component `animals:genome`, saved and synced to whoever sees it, 40 bytes): drawn from its
species' gene file (its breed's frequencies for a breed) when it spawns, of its sex; one allele of
each locus from each parent when it is born (dogs breed only male with female now). Each client
paints the coat from the genome on the layers its model's build wrote (`genetics/layers/<model>`,
embedded in the mod) through the engine's new skin hook (`Registry::entity_skin`: a mod paints an
entity's own texture from a component, on the client; only the bytes travel). The phenotype rules
and renderer are `coats.js`'s, ported to Rust: change both together. The animal screen and the
spyglass show the coat ("Coat: Red sable, masked parti"). The chihuahua's 96 hand-made coat looks
are gone (its looks are now age and pose only). `/animal love` (testing): every grown dog near you
is yours, wanders and falls in love.

**Looks, part 3: every animal unique (plan, 2026-11-02; asked for by Ronald: "every animal is
unique, that's super important").** What real animals show, and what follows for ours:

- *Three things make a look.* The kind (species, subspecies, breed) decides which patterns turn
  up and how often: leopards' rosettes are rounder in East Africa, squarer in the south, bigger in
  Asia, and fur greyer in cold places. The genes decide the pattern and its character. Chance in
  development decides where each mark lands.
- *Patterns are a few major genes plus many small ones.* In dogs, a few genes switch white
  spotting on or off (MITF), or decide stripes against swirls in cats (Taqpep, Dkk4). Many small
  genes set how much: MITF variants explain whether a cow is spotted but not how much white each
  one has (polygenic). Giraffe calves inherit their mothers' spot *shape* (round, jagged-edged;
  Lee et al. 2018), not where the spots are.
- *Placement is chance.* The marks are laid down in the embryo by a reaction-diffusion process
  (Turing; Dkk4 is the cat's inhibitor) or by migrating pigment cells. A clone of a calico cat
  (CC, 2001) looked nothing like its donor, so even identical genes don't give identical marks.
  Marks are fixed for life (photo-ID of giraffes, leopards and tigers relies on it). A few change
  with age: ocellated lizards go from spots to a labyrinth, each scale flipping by its neighbours.
- *Dalmatians* are extreme white (MITF) with ticking (dark spots coming back into white, roan
  locus at USH2A) and a modifier that makes the ticks big, round and solid (not yet found).
- *Selection acts on it*: giraffe calves with bigger, irregular spots survived better.

What we have: kinds (species, breeds, local gene pools that evolve), major genes, a few
quantitative genes (shade, red intensity, sable, size), and a seed per animal that places every
mark (siblings with the same genes still differ). Size is inherited and evolves (Bergmann). What's
missing is the middle layer: every spotted cat's spots are the same size, every brindle the same
stripes, and the wyvern has no markings at all.

**The design: one core, every species speaking it** (Ronald: "this should be a core thing").
Real species each have their own genes and their own interactions between them, but every
animal is built the same way: a kind, genes with a big effect, many genes with a small one,
chance, sex and age, and what life does to it. So:

1. **Core traits** (`genetics/core.json`): quantitative traits every species carries, after its
   own loci, inherited alike (one value from each parent, a little drift, evolving in local
   pools): size, shade, warmth, underside lightness, how much white; the marks' size, density,
   contrast, edge (smooth to ragged) and stretch (round to long); eye color; build (head, neck,
   tail, limbs, girth, horns). A species' gene file sets each one's mean and spread, a breed or
   lineage its own, so a cheetah-like cat has small dense spots and a leopard-like one big
   rosettes; a locus of the species' own with the same id replaces the core one.
2. **Chance**: each core trait has a heritability; the rest of its variation is the animal's own
   (from its seed), not passed on, as nutrition and development give real animals. With the
   seed placing every mark, two siblings with the same genes still differ in shade, size and
   marks.
3. **One vocabulary for the look** (`Look`: base and underside, patterns, points, mask,
   extremities, merle, patches, accent, white, eyes), painted by one painter. Species with simple
   genes say what each allele does in their gene file (a scale color, a pattern kind), read by
   one generic reader; species whose genes interact (dogs, cats: one gene hiding another) keep
   their rules in code, as their genetics is. A new species needs its gene file and its model's
   layers, nothing else.
4. **Sex and age**: a gene file's `sexes` (a male's offsets: brighter, more contrast) and `young`
   (a juvenile's look: spots a hatchling or cub loses as it grows) shift the traits. Old age
   (a greying muzzle) needs animals to age past adulthood: later.
5. **Eyes**: the layers mark the iris (`props2` blue); the eye trait picks along the species' eye
   colors, a species' genes may tie it (blue eyes with merle or much white).
6. **Build**: the engine scales single bones of one animal (synced, like its size); a gene file
   maps the build traits onto bones (neck: the neck bones' length).
7. **Scars**: what life does, saved with the animal: a badly hurt animal may keep a scar (a pale
   line through the coat, a torn wing, a notched ear, a chipped horn), painted where it fell.
8. **Wyvern lineages**: wyverns of a few kinds, each its own breed (they interbreed, as
   subspecies do), living in their own country, with their own colors, markings, breath and
   build: every one still marked differently.
9. **Dog spotting**: extreme white, ticking, the marks' size for Dalmatian spots; a Dalmatian
   breed once a bigger dog body exists.
10. **The viewer** (`coats.js`, the same rules) gets a "roll" that shows a dozen individuals of
    one kind side by side.

*Built* (2026-11-02): 1-4 (the core traits in every genome, chance by heritability, sex and
youth; eye colors chosen, not painted yet), 8 (crag, frost, sand, wood and storm wyverns, with
bands, blotches, mottling or stripes, an accent that reaches further or less; wyverns now live in
the cold, the deserts and the forests too, rarer than in the mountains) and 10. Genomes grew
from 40 to 84 bytes: one saved before gets its core traits from the wild when it loads, and the
genes kept in breeding records and eggs keep their old place with the rest after.

*Eyes* (2026-11-02): every animal's eyes in its own color along its species' range (dogs dark
brown to amber, cats copper to green, raptors amber to yellow, wyverns gold to red), painted on
the iris the layers mark, keeping its painted light. Genes decide it outright where they do in
life: colorpoint cats always blue-eyed, white cats and some merle dogs blue. The capybara's eyes
are plain dark, nothing to recolor.

*Build* (2026-11-02): every animal's head, neck, tail, legs, girth and (wyverns) horns a little
bigger or smaller by its genes and chance (a few percent; a wyvern's horns up to 30%). The engine
draws single bones of one entity scaled (`Skin::scale_bone`, set by the coat painter on the
client, so only the genes travel): a bone's own boxes about its pivot, its children kept their
size and moved with its ends, the animal raised or lowered by how far that moves its lowest
point at rest. Each gene file's `build` names the bones.

*Scars* (2026-11-02; `scars.rs`): a blow that takes more than a twelfth of an animal's health
may leave a scar (likelier the harder it was, at most six in ten), of what dealt it: an animal's
claws (or a cat's swipe, a raptor's kick) three raked lines, its teeth a ring of tooth marks, a
blade or a tail one long line, fire or a blast a burn; a winged one's wing skin tears instead a
third of the time (holes in it). Up to four, the oldest going, kept in the genome (synced, saved,
not inherited); its coat's name says "scarred". Each is painted flat on the body's surface where
its place falls (the back and flanks; a tear on the wing skin), sized to the body: healed skin
paler and pinker than the coat, a burn dark with a pale rim. `/animal scar` gives the animal looked
at one of each.

*Crisp marks* (2026-11-02): looked up how games do coats (tint regions in ARK and Red Dead,
grey-and-tint in Hytale, gene-driven overlay layers in horse games, Spore's procedural painting).
Ours combines all of them; the one thing pixel art asks for that it lacked was clean clusters.
Every mark is now snapped to three tones (none, an edge tone, full) and lone texels taken into
their neighbours'. Still worth doing: designed markings for what noise can't place (a tabby's
M, eye stripes, a blaze, socks, a wyvern's face mask), and cell-grown rosettes and spots.

*One copy of the rules, on the website too* (2026-11-02; asked for by Ronald: "compile it to WASM
so the website mimics the real logic, to test breeding, family trees, time"). The mod's pure
logic is its own crate, `mods/animals/core` (`animals_core`: the species table, genes and
breeding, coats, builds, scars, evolution's generations, growing up and litters), used by the mod
in the game and, through `mods/animals/web` (a JSON API, `web/build.sh`), compiled to WASM for
the viewer. The viewer's coat panel and How it works run on it, and its **Breeding lab** runs a
population on the game's clock: wild animals from any habitat, matings the game would allow (it
refuses the same pairs), carrying, litters and clutches, eggs hatching, growing up, families
(grandparents to young), scars and deaths, breeding by themselves up to a cap, and old age (not in
the game yet: a setting there), with a habitat's pool evolving beside it. The JS copy of the rules
(`coats.js`) is now only the skill's standalone version.

The lab plays time forward (6 hours to 30 days a second) with charts of who is alive and how
many are born and die a day, shows the animal picked live, and has a balance table (each
species' numbers and what they add up to: young per female per 10 days, days and play hours to a
4th generation). `web/test.mjs` runs its checks on the rules through the WASM. What they found:
the 1% mutation chance per gene in every young (about 1 young in 10 carries one somewhere) is far
above life's and makes true-breeding lines leak; populations left alone overshot their cap and,
with old age, died out together (a cohort aging as one), fixed in the simulator by logistic
breeding and lifespans that vary.

**Old age** (2026-11-02; Ronald: "animals should die"). Every animal ages on the game's tick clock
(not the sky's: `/time` doesn't age them) from when it was born (`Animal::born`, counted from a
start long before the world's, so a wild one met in a new world can be older than it; a wild one
is met at a random grown age up to most of an average life). For the last fifth of its own
lifespan (its species', give or take a fifth by its genes' seed, so siblings don't die together)
it is an **elder**: a grizzled muzzle, the coat a little faded, past breeding. At the end it dies
(`animal::old_age`). Its screen says how old it is and how long its kind lives; `/animal age
<days>` makes one older (testing). Lifespans, in in-game days (20 minutes each): dogs and cats
50, capybaras 31, raptors 77, Utahraptors 153, wyverns 422 (derived from real lifespans since:
Balance, below). The lab runs the same rule (or none,
or a span of its own).

**Balance research** (2026-11-02). How games and life pace breeding:
- Minecraft (the same 20-minute day): parents wait 5 minutes (a quarter day), babies grow up in a
  day. ARK at 1x: a raptor hatches in 2 hours and grows up in 37, a wyvern in about 4 days; most
  servers breed 10-50 times faster. RimWorld cats: 10-day gestation, 1-3 kittens, adult at 24
  days of a 60-day year. Sims 4 pets: about 25 adult days, then 6-11 elder days. Niche: 24 days.
- Real ratios (dog gestation 1): cat 1.0, capybara 2.2, a troodontid's incubation about 1.2;
  maturity: cat 0.8, capybara about 1.8, raptors 3 or more, a condor about 8. Ours keep that
  order, compressed about 60-100 times.
- Recommended (in-game days): dogs and cats carrying 1-1.5, litters of about 3, rest 2-3, grow up
  in 2.5-4, live 40-60; capybaras 2-2.5, about 4, 4-6, 4-5, 30-45; raptors 1-1.5, 2-4 eggs, 4,
  5-8, 60-100; Utahraptors 2-3, 1-3, 6-8, 10-15, 120-200; wyverns 5-7, 1, 14-20, 25-40, 400+.
- Targets: a pet pair to the 4th generation in 12-16 days (now 12), 6-10 young per female per 10
  days (cats 11.7, dogs 8.3), a recessive coat in two litters of carriers 80% of the time.
- Evolution: at our selection strength 0.6 a whole region changes coat in about 12 in-game days
  (4 hours of play); 0.25-0.3 makes it 25-40 days, over several sessions. The mutation rate
  (0.002 a gene in the wild, 1% in every young bred) is 100-1000 times life's.

Sources: [Minecraft breeding](https://minecraft.wiki/w/Breeding), [ARK breeding](https://ark.wiki.gg/wiki/Breeding),
[RimWorld cat](https://rimworldwiki.com/wiki/Cat), [Sims 4 pet life stages](https://sims.fandom.com/wiki/Life_stages_(pet)),
[Niche](https://niche.fandom.com/wiki/Nicheling), [dog litters (Borge 2011)](https://pubmed.ncbi.nlm.nih.gov/21196028/),
[cat](https://en.wikipedia.org/wiki/Cat), [capybara](https://en.wikipedia.org/wiki/Capybara),
[Troodon incubation (Varricchio 2018)](https://www.nature.com/articles/s41598-018-30085-6),
[Deinonychus age (Erickson 2009)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2756958),
[peppered moth](https://www.wikiwand.com/en/Peppered_moth_evolution).

**Wild breeding, crowding, strength by age** (2026-11-02; Ronald: "animals in the wild should
mate, I don't want them to die out; age should be part of the stats"). Wild animals now breed on
their own: a grown, rested wild one, now and then (about every other day with room), falls in love
with a wild mate within 24 blocks, the more readily the fewer of its kind within 48 blocks against
its species' crowd (now derived: dogs 17, cats 13, capybaras 16, raptors 12, Utahraptors 7,
wyverns 3), a little even
when full, never past three times it (`life::love_chance`; elders crowd half). When crowded, wild
ones die of want, the old and the young first (`life::want_chance`). Without that last part a pen
filled with one generation that aged together and died out (cats and chihuahuas in the first
in-game test); with it, generations overlap and every species keeps going (four pens, 150 sped-up
days, `mods/animals/tests`). Strength follows age (`life::vigor`): 40% at birth, growing up,
young adults 90%, full in their prime, fading from the elder years to 60% at an average life's
end; health, stamina, damage, speed and carrying go with it (shown on its screen). Testing:
`/animal speed <n>`, `/animal census`, `/animal reset` (`mods/animals/tests/README.md`).

**Time away** (2026-11-02; Ronald: "my animals should die even while I'm gone"). Animals only
live where a player is: the engine spawns them 20-45 blocks from players (at most 6 passive ones
within 64) and ticks only loaded areas. So what happens elsewhere is caught up when an area loads,
as lazy simulations do (Vintage Story catches crops, births and spoilage up on load; Minecraft's
Elapsed and KeepMeGrowing grow crops and young by the time since the chunk unloaded; Dwarf
Fortress simulates a whole history when a world is made). Each animal keeps when it was last seen
(`Animal::seen`); past half a minute away (`life::AWAY`), on its next tick its timers run on (a
mother due gives birth), it has aged by the world's clock, a tame one whose time ran out dies of
old age, and a wild one that died meanwhile has a successor in its place: born while away, of its
place's kind as it has evolved since (`animal::catch_up`). A place's gene pool used for the first
time starts as if it had evolved since the world began (up to 300 generations, `HISTORY`; a
revisited one catches up to 120). Not caught up: births and deaths of want while away (a herd
comes back the size it was). `/animal away <days>` makes the animals round the player as if that
long had passed (`mods/animals/tests/away.txt`). Owners aren't told yet when a pet died away (the
engine has no way to send a player a chat line).

Sources: [Vintage Story forum](https://www.vintagestory.at/forums/topic/12926-how-close-do-i-need-for-things-to-keep-growingburningcooking-ect/),
[Elapsed](https://www.modpackindex.com/mod/208669/elapsed), [KeepMeGrowing](https://modrinth.com/plugin/keep-me-growing),
[Cattlestrophic!](https://modrinth.com/mod/cattlestrophic) (old age for Minecraft's animals).

**Balance: numbers from real facts** (2026-11-02; Ronald: "lots of animals, how do we keep it
balanced? Is there a function one can run for everything?"). Games that stay balanced with many
creatures don't tune each one by hand: they derive the numbers from a few inputs by one formula
(ARK's per-species multipliers on a global rate, card games' cost curves, Magic's and
Hearthstone's "vanilla test"), and check every one against targets. Life has such a formula:
spans scale with body mass by quarter powers (lifespan and time to maturity ~ M^1/4, metabolism ~
M^3/4: Kleiber's law; gestation weaker). So a species is now written as life has it
(`core/src/biology.rs`, `Biology`): its mass, real days carried or incubated, to grow up, of life
and between litters, its litter, its social life and its rarity. One pacing for every species
turns real days into game days, compressing long spans more than short ones (game days = a ·
real days^k: carrying 0.056·d^0.75, growing up 0.0645·d^0.7, a life 0.0105·d, rest 0.0139·d;
fitted so the first seven land on the research's recommendations above). The room near (`crowd`)
falls with mass (land per animal grows with it, ~M^-0.2) and rises for packs and herds. Any
number can still be set by hand (`Tune`), and shows as set.

Then the checks (`core/src/balance.rs`), on every species, a new one included: a line four
generations deep in 10-25 in-game days for a common species, 20-40 uncommon, 35-70 rare, 80-160
legendary; grown up in under a fifth of its life; 6-40 young in a mother's breeding life; a
sensible litter and room; and in the simulator a wild group left alone at its room (no newcomers,
as in a pen) lasts 150 days or a life and a half in two runs of three (`web/test.mjs`). The lab's
balance table shows the facts, the numbers derived, what they come to for a player and the checks.

| | real: mass, young, grown, life | game days: carrying, rest, grown, life | litter | room | 4th gen |
|---|---|---|---|---|---|
| dog (chihuahua), common | 2.5 kg, 63 d, 300 d, 13 y | 1.3, 2.5, 3.5, 50 | 1-4 | 17 | 14 d |
| cat, common | 4.5 kg, 65 d, 240 d, 13 y | 1.3, 2.8, 3.0, 50 | 1-4 | 13 | 13 d |
| capybara, common | 50 kg, 140 d, 1.5 y, 8 y | 2.3, 5.1, 5.3, 31 | 2-6 | 16 | 23 d |
| raptors, uncommon | 15 kg, eggs 74 d, 2.5 y, 20 y | hatch 1.4, 5.1, 7.5, 77 | 1-4 | 12 | 28 d |
| Utahraptor, rare | 300 kg, eggs 140 d, 4.9 y, 40 y | hatch 2.3, 7.6, 12.3, 153 | 1-3 | 7 | 44 d |
| wyvern, legendary | 5 t, eggs 1.2 y, 20 y, 110 y | hatch 5.5, 15.2, 32.7, 422 | 1 | 3 | 115 d |

What the simulator found: small rooms die out. Left alone, a cat group with room for 8 died out
within 150 days in half the runs, with 12 in none; a Utahraptor group of 4 in half, of 6 in one in
eight (a minimum viable population, as conservation biology calls it). So rooms start from 13
cats' worth, not 8. In the open the engine keeps spawning newcomers, so only pens depend on it.

*Next*: the rest of the balance (selection 0.25-0.3, fewer mutations; combat stats by a cost
curve the same way), then Dalmatian spotting.

Sources: Lee, Cavener, Bond 2018 (PeerJ 6:e5690, giraffe spots); Kaelin et al. 2021 (Nature
Communications, tabby genes); Manukyan et al. 2017 (Nature 544:173, lizard scales); Shin et al.
2002 (cloned cat CC); Massey University 2025 (Science Advances, Holstein MITF and KIT); the USH2A
roan studies (PMC7986734, PMC7987146) for roan and Dalmatian spots.

**Evolution, part 1: local gene pools and natural selection** (2026-11-01; `evolution.rs`). Each
species' pool per place (256-block cell and habitat, from the biome's tags) starts as its gene
file and evolves a generation an in-game day, when next needed (up to 60 at once), as numbers:
200 animals drawn from it, parents chosen by fitness (1 − 0.6 × camouflage × how far the coat's
overall color is from the habitat's ground), their young making the next pool (drift), 0.2%
mutation per locus, 2% from the neighboring places' pools. Wild animals spawn from their place's
pool. Saved in the world (`mods/animals/pools.json`, through the bridge's new `data_dir`).
`/animal evolve [count] [habitat]` runs generations where you stand and prints each pool. Checked:
after 100 generations raptors are all sandy in the desert, red cats rise from 25% to 73% there,
white spotting near fixed in snow, brown raptors and dark dogs rise in forest; 200 animals keep
variation (40 fixed nearly every locus by chance).

**Evolution, part 2: players' part** (2026-11-01). A wild animal that dies (any cause: the death
event's position, as the dead one is nowhere any more) or is tamed leaves its place's pool, as one
animal of its 200 (`evolution::count`, `coats::count_in`); the animal screen's new Release (the
owner only) lets a tame go: wild again with its levels, bond, orders and tribe share gone, its
saddle dropped, taming starting over, and its genes join the pool where it is. Checked in the game:
ten raptors killed and a chihuahua tamed each left their pool, the released one joined it.

**Sizes** (2026-11-01). Every species at its real size, a block a meter: the chihuahua rebuilt
to 20 cm at the shoulder (×0.72) and the cat to 24 cm (×0.87), so the chihuahua is now smaller
than the cat; capybara (56 cm), Velociraptor (0.5 m at the hip) and Utahraptor (1.5 m) already
were. Individuals differ around that (`coats::size`): a size gene `Sz` in every gene file
(quantitative: a baby takes one value from each parent, so big parents have big young), spread
±20% for chihuahuas (15-23 cm), ±8% for the rest, and sex: males 4% taller than females in dogs,
8% in cats, capybara females 3% the bigger, none for the dinosaurs (unknown). The engine's new
per-entity scale (`World::set_scale`: model, box and eyes; quicker steps for small ones) draws
it, the riding seat follows it, and the screens show it ("Size: 21 cm at the shoulder", "Grows
to ..." for young ones). Size evolves: mammals have more young the bigger they are in snow and
taiga and the smaller in the desert (Bergmann's rule, `evolution::fitness`). Genomes from before
read the new gene as average.

**Breeding, for every species** (2026-11-01; `breeding.rs`, replacing the dog's own). A grown
tame with bond 50 or more fed a favorite food by someone who may command it is in love for 30
seconds (hearts); two of one species and both sexes in love meet (one told to stay waits) and
mate. Mammals carry the young (dog and cat a day, capybara two) and give birth to a litter
(dog 1-4, cat 2-5, capybara 2-6); raptors lay a clutch of 2-4 eggs after five minutes
(`animals:raptor_egg`, a code model), which hatch after a day (the Utahraptor's two), twice as
fast within two blocks of a torch or furnace, not at all in snow or taiga without one. Each young
gets genes from both (`coats::breed`: siblings differ) and each stat's wild points from one
parent, the stronger 55% of the time; it is its mother's owner's, follows its mother, and grows
from its birth size (a third or so; the chihuahua keeps its puppy model) over three days (dog,
cat), four (capybara, Velociraptor) or twelve (Utahraptor), a favorite food speeding it. Mothers
rest after a birth (two to four days), fathers five minutes. Every animal keeps its generation
and parents' genes; the screens show "In love", "Expecting young in N min", "Grows up in N min",
"Generation 2 · mother: ... · father: ...". Checked in the game: a capybara litter of 6, a cat
litter of 4 from one pair (black torbie, brown tabby, red tabby, white), two raptor eggs that
hatched (rufous barred, ash spotted). `/animal love` and `/animal hurry` (births, hatching and
growing up within a second) for testing. Not yet: heat cycles, dens and nests, imprinting,
inbreeding costs.

**Eggs and nests** (2026-11-02; `eggs.rs`). Raptor eggs are items you can carry: right click one
(or the nest it's in) to take it, use it on the ground or a nest to put it down; whoever puts it
down owns the young that hatches (taming by raising). Taking an egg from the wild sets the wild
ones of its kind within 24 blocks on the thief. Wild females now and then (about every two days,
while their kind isn't crowded and no nest is near) build a nest and lay a clutch fathered by
their place's gene pool; tamed mothers lay into a nest within eight blocks. Eggs hatch twice as
fast in a nest or by a torch or furnace, not at all in snow or taiga without. The nest is a prop
(an entity, placed with a nest item made from sticks and wheat): an earthen dish about a meter
across in a sloped rim, as Troodon (the raptors' close relative) dug them; the egg is about 15 cm
long, pointed below and blunt above, set a quarter into the ground (onto the dish's floor in a
nest) leaning its top toward the nest's middle, as Troodon's were; colored and marked
(Deinonychus's eggshell held both birds' pigments). Both built with the skill's full workflow
(`source/nest`, `source/egg`); of the options in the viewer (the nest bare, lined with grass or
grassy-rimmed; the egg blotched blue-green, speckled cream, rufous or plain green) Ronald chose
the bare nest and blotched eggs (`CHOSEN` in each build). `/animal nest`: the nearest wild female nests
now.

**House style** (2026-11-02, decided with Ronald): every model is made with the blocky-model
skill in one style so they match: 64 texels a block for every model (Hytale's density for
characters), real sizes, the skill's light and materials (`scripts/house.py`), props as entities.
The five animals predate it (19 to 131 texels a block) and are to be rebuilt to it.

**The wyvern** (2026-11-02; `wyvern.rs`, `source/wyvern`; asked for by Ronald: a wyvern that
flies, can be ridden, breathes fire, rare and dangerous, huge, like Game of Thrones'). A
fantasy animal, kept apart from the real ones: its family is placed under "Fantasy"
(*Wyverna montana* · Fantasy › Draconia › Wyvernidae › Wyverninae). About 3.9 m at the shoulder,
16 m long, 24 m across the wings (reworked after Ronald found it looked wrong, from the show's
Drogon: a long forearm and four long fingers, a muscled chest, standing broad on wrists planted
wide, each folded wing a tall pleated sail; `source/wyvern/README.md`): two heavy hind legs, bat
wings for arms; on the ground it goes on four, on its feet and the wrists of its folded wings. Built with the skill at the house
density (64 a block). Painted with plated scales (of four ways to draw them in the viewer, Ronald
chose plated, 2026-11-02): big armor-like scales each with a dark rim, plates across the belly,
throat and under the tail, veined wing skin with a darker trailing edge, ringed horns paling to
the tip, amber slit-pupil eyes under the brow, teeth along both jaws. Animations: idle, walk and run
(the wings step one after the other, as the show's dragons, animated from gorillas; the run a
gallop), breathe (rears, lunges with the jaws wide, sweeps), fly, glide, and lying down (rest,
knocked out). Its feet don't slide: each foot's path is set and the limb's angles solved to it
every frame. Genes (`genetics/wyvern.json`): what it breathes (fire, frost, lightning, venom;
the commoner dominant), its scales' color (slate, bronze, green, black, red, rare cream), how
dark, how big; the coat line names them ("Slate, breathes fire"). Each scale color brings its
accent, as the show's dragons wear them (black with red wings, green with bronze, cream with
gold): the game repaints each wyvern from its genes on its coat layers, the accent on the wing
skin and spike tips as the look's points. Rare and alone in the mountains (`#mountain`); a wild
one goes for any player within 14 blocks (24 near a nest) and hunts sheep, pigs and cows as the
Utahraptor does. Knocked out to tame; 400 health, 30 damage. Its eggs are about 3.4 times a
raptor's and its nests as many times bigger (any species' nest is now scaled to its eggs).
Rideable on the ground with its own saddle (more leather and an iron ingot). The engine now
flies entities (`World::fly`) and plays a model's `run` by speed (the walk and run written as
offsets from the idle).

**Riding in flight** (2026-11-02; `riding.rs`, engine). A tamed, saddled wyvern's rider sits at
the base of its neck between the shoulder spikes, as the show's riders do (the engine's seat can
now be ahead of a mount's middle: `World::mount_at`). Jump takes off. In the air it flies where the
rider looks while they go forward (look up to climb, down to dive), sideways with the side keys,
climbs with jump and comes down with sneak, faster sprinting (about 18 blocks a second, 28
sprinting), and hovers with no key held; it beats its wings, gliding as it dives. It lands when it
touches the ground; a rider who came down holding sneak lets go before sneaking gets them off.
Without its rider in the air it stops flying. Riding anything big, the third-person camera pulls
back by twice the mount's height so it clears it. Checked in the game: take off, climb, fly, dive,
land, get off; a wild one killed a player in survival who came near.

**The breath** (2026-11-02; `breath.rs`). What a wyvern breathes is in its genes (fire, frost,
lightning, venom). Its rider makes it breathe with a key (V, rebindable) where they look; a wild one angry at
something 5 to 16 blocks off breathes at it now and then. On the ground it rears, draws breath and
lunges (`breathe`), the stream pouring from its mouth from 0.95 to 2.6 seconds; in the air it
breathes as it flies, at once; then eight seconds before the next. The stream reaches 14 blocks in
a cone: fire burns and sets alight, scorches grass to dirt, burns leaves, plants and snow away and
melts ice; frost hurts less, slows (60% for four seconds) and puts fire out, freezes water and
frosts grass; lightning strikes the nearest thing in it hard and arcs on to two more; venom hurts
and poisons for five seconds, withering grass and leaves. Each its own particles (flame, frost
mist, sparks in a jagged bolt, a green spray). `/animal fire|frost|lightning|venom` sets a
wyvern's breath (testing). Engine: mods can set entities alight (`World::ignite`); a mob standing
still now brings its body round to where it looks (vanilla's body rotation), so one that turned to
face something faces it whole. Checked in the game: each breath ridden, and a wild one breathing
fire at a player and killing them, and breathing in flight (the stream carries the wyvern's own
speed, or it would outfly its fire).

**Attacks** (2026-11-02; asked for by Ronald). Every animal that fights shows its hits: each
species lists its attacks (`Species::attacks`), and `commands::fight` plays one at random when a
hit lands (the engine paces hits by the kind's attack speed). The chihuahua bites (ears flat, a
lunge and snap); the cat swipes (a forepaw raised to the face and raked down, a hiss) or bites;
both raptors bite or kick with the sickle claw (rearing back on one leg, the other struck forward);
the wyvern bites (the neck drawn back and lunging), claws (a wing-arm lifted 3 m and brought down
on what's ahead) or lashes its tail. The capybara never fights. How far past its box a species
strikes is `Species::reach` (the wyvern's head is 7 blocks ahead of its middle: 5 blocks).

Next for the wyvern: wild ones flying, its sounds (a roar, wingbeats, the breath), its own egg
icon.

Next otherwise: rebuild the five animals to the house density; the paint screen in the game
(paint as a second component the same painter reads), coat names in translations.

Not yet: XP and earning points (the command gives them), naming, the higher spyglass tier, tame
caps and other world settings. Engine note: clients don't see other entities' attribute
modifiers, so the screen takes max health from the stat.

## Breeding seasons (2026-11-02)

Ronald: seasons tied in with mating and breeding; each species breeds in its real season
(`notes/SURVIVAL.md`, "Seasons").

- A species' facts say the seasons it comes into heat in and its peak (`Biology::breeds`, `peak`):
  dogs all year (unlike wolves, about twice a year whenever); cats spring to autumn, most in
  spring (long-day breeders, anestrous in winter's short days); capybaras all year, most in spring
  (most births as the rains begin, April and May in the llanos); the Velociraptors and
  Utahraptor the wet season, most at its start (no record: their nesting colonies came back year
  after year as birds' do, and their kin time hatching to the growing season); the wyvern early
  in the wet season (as the great birds of prey, courting in late winter, laying in spring). In a
  world with a wet and a dry season, spring and summer are the wet one.
- Wild ones fall in love only in their season, twice as readily in its peak
  (`Biology::season_love`). A tame fed its favorite food out of season comes into heat only if
  well cared for: bond 80 and its belly three quarters full (a stand-in for comfort until camps
  have it). Where there are no seasons (the flatlands) all year is breeding season.
- The animal screen says when its kind breeds and whether it's that season now; SPECIES.md has a
  "breeds in" row.
- Not yet: the website's simulator (`web/`) breeds without seasons.
