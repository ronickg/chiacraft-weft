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

Every species is a definition (a Rust table in the mod, or a JSON file per species: proposed),
so adding an animal is mostly numbers:

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
- **Breeding**: heat cycle (length, interval, season), courtship time, the place it needs (nest,
  den, burrow; biome; temperature), gestation or incubation time and temperature range, litter
  or clutch size, recovery time, growth time per stage.
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
3. **A run animation picked by speed**.
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

Next: the paint screen in the game (paint as a
second component the same painter reads), then sizes (real sizes, by sex and individual: the
engine's per-entity scale), breeding for the other species (only dogs breed so far), coat names
in translations.

Not yet: XP and earning points (the command gives them), naming, the higher spyglass tier, tame
caps and other world settings. Engine note: clients don't see other entities' attribute
modifiers, so the screen takes max health from the stat.
