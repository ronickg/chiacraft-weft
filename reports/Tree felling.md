# Tree felling

What Ronald asked (11-05): trees "fall down sideways and like the leaves to sort of start breaking while
it's falling"; "chopping should be pretty hard. So it takes longer as you always chop down the bottom
part"; big trees "can always be chopped down, but you won't get any resources and it just takes forever"
(he chose "Slow and fruitless"); "once it's on the ground, you still got to chop the logs in pieces, kind
of like Valheim does it"; and "even if the tree falls, it would be nice if it can break some other trees
or partially".

The research behind the pick (Valheim, the Minecraft mods, Vintage Story, how a falling tree is drawn,
game feel) is below it, with a source for every number.

## The pick

**1. Chopping (built, slice 1, `70177b92`).** The foot takes `logs^0.75` times one log's time, with the
axe's speed on top, and the engine's own mining timer runs it: while a player mines a tree's foot with an
axe a `MINING_SPEED` modifier slows them by that factor, so the cracks creep on their screen and the server
times it the same way. Sub-linear, like HT's TreeChop (`1 + 10 ln(1 + (n-1)/10)`) and Vintage Story
(`sqrt(resistance / 1.45)`), so the right axe makes a giant a job, not hopeless. Each axe tier takes trees up
to a size; past it a tree still falls, 8x slower and with no wood (Valheim's "too hard" turned into "slow and
fruitless", as Ronald chose). The numbers are in `vitals::felling` and the sheet's Felling table:

| tier (harvest) | takes up to | oak (20 logs) | birch (34) | araucaria (72) | young sequoia (165) | sequoia (1015) |
|---|---|---|---|---|---|---|
| wood, gold (0) | 40 logs | 14 s | 21 s | 5 min * | 9 min * | 36 min * |
| bone, antler (50) | 64 | 11 s | 17 s | 4 min * | 7 min * | 29 min * |
| flint, stone (100) | 96 | 10 s | 15 s | 25 s | 6 min * | 25 min * |
| copper (150) | 192 | 6 s | 9 s | 15 s | 28 s | 15 min * |
| bronze, iron (200) | 512 | 4 s | 6 s | 11 s | 20 s | 11 min * |
| steel (300) | 1536 | 4 s | 5 s | 10 s | 18 s | 70 s |
| diamond (400) | 4096 | 4 s | 5 s | 9 s | 17 s | 67 s |

`*` past the tier: no wood. The wooden axe takes the first day's common trees (oak, birch and spruce medians; Planner, 11-05). Log counts are each species' median over 40 grown trees
(`cargo run -p base_api --example tree_logs`): the ordinary trees grow 5-72 logs, a young sequoia 76-231,
a grown one 444-1421. Before, any tree up to 128 logs fell in one log's time (about 1 s with a stone axe),
and bigger ones were chopped block by block.

**2. The fall: the tree's own blocks, as one moving structure that leans (needs a small engine change).**
Every good falling tree in the research is one entity built from the tree's own blocks, rotated about the
stump's far edge (Dynamic Trees, Panda's Falling Trees, PeterWolf's, Realistic Tree Behaviour). Weft already
has the first half: `World::lift` takes blocks out of the world into a moving structure that keeps their
textures, tints and light, and `set_structure_block` changes them while it moves. What it can't do is turn:
a structure is drawn at an offset only (`world.wgsl` `finish`, `offsets[origin.w]`). The change:
- a structure gets a pose (a pivot and a rotation) next to its offset, eased on clients like its position;
- `world.wgsl` (the engine's and the base shader pack's copy, and `shadows.wgsl`) turn a structure's
  vertices and normals by it; culling widens its box;
- a leaning structure doesn't collide (it is only drawn) until it is put down;
- the SDK call: `w.body_tilt(entity, pivot, axis, degrees, ticks)`.
That is one line in `notes/ENGINE_CHANGES.md` and stays ours (Ronald: only bug fixes go upstream for now).
Why not without it: a box-model entity (a trunk box and a crown box per species and height, `set_look`,
a keyframed `fall` animation) needs no engine change, but it isn't the tree that stood there, and its
leaves can't break off one by one; re-placing the blocks at 15-degree steps is cheap but jerks and remeshes
the world at every step. The box model is the fallback if Ronald wants the engine left alone.

How it falls (numbers from Panda's and Dynamic Trees):
- away from the player (PeterWolf's; Valheim falls along the last swing), snapped to the nearest of 8 ways;
- 1 s pause with a creak, then an ease-in fall, `angle = 90 (1 - cos(t pi/2 / T))`, `T = 2.5 s x sqrt(height / 10)`
  between 2 and 5 s, so tall trees start slower; a 8-degree bounce over 0.6 s;
- leaves break as it goes (Ronald's ask): from 20 degrees on, each tick the leaves moving fastest (the
  highest, `angular speed x height`) leave the structure as particles of their own block, a share of them
  rolling their loot (saplings, sticks, fruit) where they are; at impact the rest burst (at most 400 leaf
  blocks of particles, as Dynamic Trees caps it);
- sound in three beats: the crack as it starts to lean, the rush of the fall, the thud 0.25 s before
  contact; a short camera shake within 16 blocks.

**3. The trunk on the ground, in pieces (Valheim).** It lands as horizontal logs along the fall line,
dropped onto the ground under each, marked `felled` (a prop): a felled log gives nothing when the tree lands.
The lying trunk is cut into pieces of 4 logs (Valheim's large log in 2 halves, PeterWolf's 2-3 hits a
section); chopping any log of a piece with an axe takes `4^0.75` (2.8) logs' time and breaks the piece into
its wood at the player's feet. Branch logs break off on landing as sticks. The foot's column leaves a stump
(the foot's own block stays as a stump block, cleared by digging it out). A tree past its axe's tier lands
as nothing: its trunk shatters on impact (particles, no wood), as "slow and fruitless" says.

**4. Knock-on (Valheim's chain felling, Salty's domino).** The falling trunk is tested as a few boxes along
it (Dynamic Trees: at most 24) against the world each tick:
- leaves of other trees in its path break (particles, a share drop their loot);
- a branch log (one not over a trunk's base) it hits breaks off as sticks;
- another tree's trunk: if the falling tree has at least twice its logs, that tree falls too, the same way,
  a moment later; otherwise the falling tree stops against it, leaning, and is put down there (it hung up).
- creatures and players it sweeps take `2 + logs / 10` damage once, up to 20, before the bounce
  (Dynamic Trees hurts once, by wood times speed).

**Order of work** (slices): chop time (done) -> the leaning structure in the engine and the fall with its
leaves -> the trunk on the ground in pieces -> knock-on.

---

## Research (11-04/05)

Where a number comes from decompiled or mod source, the file is linked. Lines marked "(derived)" are arithmetic from the cited numbers, not a source's claim.

### 1. Valheim

**Tree HP, tool tier, drops** (wiki table: https://valheim.fandom.com/wiki/Trees)

| Tree | Axe needed | Tree HP | Large log HP | Log (half) HP | Stump HP | Wood per small log |
|---|---|---|---|---|---|---|
| Beech | Stone | 80 | 60 | 60 | 80 | 10 Wood |
| Fir | Stone | 80 | 60 | 40 | 100 | 10 Wood |
| Pine | Stone | 120 | 60 | 40 | 120 | 15 (50% Wood / 50% Corewood) |
| Birch | Bronze | 80 | 60 | 60 | 80 | 10 (50% Wood / 50% Finewood) |
| Oak | Bronze | 200 | 160 | 140 | 160 | 25 (50% Wood / 50% Finewood) |
| Ancient (swamp) | Bronze | 80 | none | 60 | 100 | 10 (50% Wood / 50% Ancient bark) |
| Yggdrasil shoot | Black metal | 100 | 100 | 100 | 80 | 10 (50% Wood / 50% Yggdrasil wood) |
| Scorched | Stone | 200 | 90 | 90 | 80 | 10 (66.67% Ashwood / 33.33% Wood) |

- Every large log splits into 2 small logs; every stump gives 2 Wood; the standing tree itself has a 50% chance of 1-3 extras (resin, feathers, seeds): https://valheim.fandom.com/wiki/Trees, https://valheim.fandom.com/wiki/Birch
- So an Oak is 1 tree (200) -> 1 large log (160) -> 2 small logs (140 each) -> 50 wood, plus a 160 HP stump: https://valheim.fandom.com/wiki/Oak
- Tier numbers: 0 Stone axe, 1 Flint, 2 Bronze, 3 Iron, 4 Black metal. Vanilla min tiers: Beech/Pine/Fir trees and logs 0; Birch and Oak trees, logs and half logs 2; SwampTree1 (Ancient) tree 2 but its log 0: https://thunderstore.io/c/valheim/p/FixItFelix/ConfigureToolTier/
- Axe chop damage (base/max upgrade): Stone 20/29, Flint 30/39, Bronze 40/49, Iron 50/59, Black metal 60/69. Stone and Flint cut "up to Pine", Bronze/Iron "up to Birch and Oak", Black metal "up to Yggdrasil shoots". One-handed axe swing on trees is 1.3 s: https://valheim.fandom.com/wiki/Axes
- (derived) Beech with a Stone axe at skill 0: 80/20 = 4 swings for the tree, then 3 per half log; Oak with Bronze: 200/40 = 5 swings.
- "Too hard": if `hit.CheckToolTier(m_minToolTier)` fails, the game shows `DamageText.TextType.TooHard` at the hit point and does no damage; the same check guards logs: https://github.com/davrum/assembly_valheim/blob/main/TreeBase.cs, https://github.com/davrum/assembly_valheim/blob/main/TreeLog.cs. You still gain Wood cutting XP on a "too hard" hit: https://valheim.fandom.com/wiki/Wood_cutting_(skill). Stumps can also be "too hard": https://valheim.fandom.com/wiki/Stump_(Tree)
- Wood cutting skill raises chop damage up to +141% but never unlocks a harder tree; 1 XP per swing: https://valheim.fandom.com/wiki/Wood_cutting_(skill)

**Hit feedback (standing tree)** (https://github.com/davrum/assembly_valheim/blob/main/TreeBase.cs)
- Every damaging hit runs `ShakeAnimation`: for 1 s the trunk rotates by `sin(t*40)` / `cos(t*0.9*40)` degrees times `(1-t)^3 * 1.5`, i.e. a 1.5 degree wobble that decays cubically. Plus a hit effect and 100 "noise" that wakes nearby creatures.

**The fall** (same file)
- At 0 HP the standing tree is destroyed; `SpawnLog` instantiates a separate log prefab (a Unity Rigidbody) at the trunk's spot, scales its mass by tree scale, and pushes it with an impulse `hitDir * 0.2 * mass` applied 4 m up the trunk, so it tips away from the swing. A stump prefab is spawned in place. Physics does the rest.
- "A tree will fall in the direction the player is facing on the last swing, if the tree is on level ground": https://valheim.fandom.com/wiki/Wood_cutting_(skill)
- Logs are hit like trees; each hit also pushes the log (`AddForceAtPosition(hit.m_dir * pushForce * 2)`). On 0 HP a log drops its items spread along its length (`spawnDistance` 2 m) and spawns its sub-logs (the halves) at child "sub log points": https://github.com/davrum/assembly_valheim/blob/main/TreeLog.cs

**Falling logs hurt things (chain felling)**
- Logs carry an `ImpactEffect`: on collision, if relative speed > `m_minVelocity`, damage = configured damage x `LerpStep(minVelocity, maxVelocity, speed)`, re-armed every 0.5 s; it can damage players (`m_damagePlayers = true`) and any IDestructible, and carries its own tool tier: https://github.com/davrum/assembly_valheim/blob/main/ImpactEffect.cs
- The wiki: a felled tree "will fall over and collide with any objects in their path": https://valheim.fandom.com/wiki/Trees. Falling or rolled Beech trees can bring down Birch and Oak before you own a Bronze axe, and logs rolled downhill shatter into lumber: https://valheim.fandom.com/wiki/Oak, https://valheim.fandom.com/wiki/Birch
- Logs also damage other logs (a mod that makes logs die in one hit notes that damage "from other trees falling" counts): https://nexusmods.com/valheim/mods/1900. Players regularly die to falling trees: https://mein-mmo.de/en/beware-valheims-deadliest-enemies-are-trees-no-joke,648602

### 2. Minecraft mods

**Vanilla baseline**: log hardness 2; break time 3 s by hand, 1.5 wood axe, 0.75 stone, 0.6 copper, 0.5 iron, 0.4 diamond, 0.35 netherite, 0.25 gold: https://minecraft.wiki/w/Log

#### FallingTree (RakSrinaNa / RakambdaOrg)
Docs: https://github.com/RakambdaOrg/FallingTree/wiki/Settings---Trees, https://github.com/RakambdaOrg/FallingTree/wiki/Settings---Tools. Defaults from https://github.com/RakambdaOrg/FallingTree/blob/minecraft/26.3/common/src/main/java/fr/rakambda/fallingtree/common/config/real/TreeConfiguration.java and `ToolConfiguration.java` in the same folder.
- Break modes: `INSTANTANEOUS` (default: the whole tree breaks at once), `SHIFT_DOWN` (each break removes the furthest log, so you chop one spot repeatedly and the tree "shifts down"), `FALL_ITEM`, `FALL_BLOCK`, `FALL_ALL_BLOCK`.
- The FALL_* modes are not a rotating tree: each log becomes a vanilla falling block entity with a random horizontal velocity of +/-0.2 blocks/tick, so the tree crumbles straight down: https://github.com/RakambdaOrg/FallingTree/blob/minecraft/26.3/common/src/main/java/fr/rakambda/fallingtree/common/tree/breaking/FallingAnimationTreeBreakingHandler.java
- Detection: `WHOLE_TREE` (default, 6 directions), `ABOVE_CUT`, `ABOVE_Y`. `maxScanSize` 500, `maxSize` 100 (more logs -> `ABORT` by default, or `CUT` part), `minSize` 0, `maxLeafDistanceFromLog` 15, `breakOrder` FURTHEST_FIRST, `minimumLeavesAroundRequired` 1 (protects log houses), `leavesBreaking` true (fast decay), `leavesBreakingForceRadius` 0, `trunkLootPercentage` 1.0.
- Speed multiplicand (default 0 = off): break time scales with tree size; code: `speed = originalSpeed / (speedMultiplicand * logCount)`; 1 = as long as breaking every log, 0.5 = half; only in INSTANTANEOUS: https://github.com/RakambdaOrg/FallingTree/blob/minecraft/26.3/common/src/main/java/fr/rakambda/fallingtree/common/tree/TreeHandler.java
- Damage multiplicand 1.0 (tool loses 1 durability per log; 0 = just 1), rounding ROUND_DOWN; durability modes ABORT/SAVE/NORMAL (default)/BYPASS.

#### Dynamic Trees (FallingTreeEntity)
Source: https://github.com/DynamicTreesTeam/DynamicTrees/tree/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/entity
- Trees are made of branch blocks with a radius 1-8 px; you cut at the lowest branch and the whole tree (wood volume) comes down. Break hardness = log hardness (2) x `treeHardnessMultiplier` (1.0) x radius^2/64 x 8, capped at `maxTreeHardness` 20: https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/block/branch/BasicBranchBlock.java. (derived) full 8 px trunk: 16, i.e. 8x a vanilla log; 3 px sapling trunk: 2.25.
- Drops come from the tree's total wood volume (`woodVolume`), and axe damage scales with radius and volume: https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/block/branch/BranchBlock.java
- Destroy types: VOID, HARVEST, EXPLODE, FIRE, ROOT. Harvest cut from the side with mass centre >= 1 block up -> "fallover" handler; low trees -> "fling"; explosion -> "blast" (physics, dies on landing or after 200 ticks); fire -> burn: `FallingTreeEntity.java`, `animation/AnimationHandlers.java`. (The "voluntary" in DT is a leaves loot table for seeds dropping on their own, not a fall mode.)
- Drawing: on the client the entity bakes one quad list from the tree's real block models (branch quads with their connections, a cut ring at the base, the rooty soil, every leaves block with its tint) and replays it each frame with one pose transform: https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/model/FallingTreeEntityModel.java, https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/fabric/src/main/java/com/dtteam/dynamictrees/model/FallingTreeEntityModelFabric.java
- Fallover animation (`FalloverAnimationHandler.java`): pivot is the trunk edge on the far side (offset by trunk radius/16 along the tool direction). Angular speed grows each tick by `0.2 / (2 x massCenterY)` degrees, so tall trees start slower; gravity 0.03/tick for the whole entity. After 10 ticks it tests up to 24 trunk segments as small boxes against blocks; on a hit it rolls back one step and bounces with elasticity 0.25; "landed" when |speed| < 0.02. Dies at 160 degrees, on landing, or after 120 + trunkHeight ticks. (derived) mass centre 4 blocks up -> about 85 ticks (4.2 s) to reach 90 degrees.
- Damage: each living entity once, `woodVolume x |fallSpeed| x 3` x `fallingTreeDamageMultiplier` (1.0), only before the first bounce and only if > 2, with knockback away from the cutter. `enableFallingTrees` and `enableFallingTreeDamage` default true: https://github.com/DynamicTreesTeam/DynamicTrees/blob/release/1.21.1/common/src/main/java/com/dtteam/dynamictrees/config/DTConfigs.java
- Sound and leaves: a start sound (pitch by volume) at the cut, a water splash sound if a segment passes liquid, an end sound at 1.5 volume on impact. On impact each leaves block flings block-break particles with velocity = angular velocity x height of that leaf, count = fallSpeed x 5 on the first hit, 1 after, scaled by e^-bounces; capped at `maxFallingTreeLeavesParticles` 400 leaf blocks. Seeds and leaf drops fall out before it tips.

#### Panda's Falling Trees (and its fork Enhanced Falling Trees)
Source: https://github.com/ThePandaOliver/Pandas-Falling-Trees
- The entity stores the tree's block map and redraws every block each frame (`submitBlock` per block) around a pivot at the trunk's edge in the fall direction: `src/main/kotlin/client/render/TreeRenderer.kt`.
- Fall curve: angle = 90 x (1 - cos(t x pi/2 / 2.5 s)), an ease-in that starts slow (the lean) and hits fastest at the ground; then a 10 degree bounce over 1 s. Defaults `fallAnimLength` 2.5, `bounceAngleHeight` 10, `bounceAnimLength` 1: `config/client/AnimationConfig.kt`.
- Sound: fall sound on tick 1, impact sound 5 ticks (0.25 s) before the fall ends, both at 0.7 volume: `trees/GenericTree.kt`, `config/client/SoundSettingsConfig.kt`.
- The entity lives `treeLifetimeLength` 4 s, then drops all items at once. Chop slowdown: `speed / (min(16, logs-1) x 0.5 + 1)`, so at most 9x slower; max 256 logs, leaves radius 7: `config/CommonConfig.kt`, `trees/GenericTree.kt`, `config/common/tree/GenericTreeConfig.kt`.
- Enhanced Falling Trees adds leaf particles along the trunk when the tree disappears and more fall/impact sounds; it says outright it "simulates" a fall for performance, not physics: https://modrinth.com/mod/enhanced-falling-trees

#### Other one-entity fallers
- Realistic Tree Behaviour: whole tree tips in your look direction as one entity, bounces, clears foliage it hits, impact sound plus a brief camera shake, then re-places as blocks: https://modrinth.com/mod/realistic-tree-behaviour
- PeterWolf's Realistic Tree Felling: one synchronized entity rotating from the cut, falls away from the player (aim by where you stand), lands and re-places as fallen logs aligned with the fall; each fallen section takes 2-3 axe hits; part of the canopy is crushed (sticks, saplings): https://modrinth.com/mod/peterwolfs-realistic-tree-felling

#### Timber / TreeCapitator / VeinMiner style
- TreeCapitator (bspkrs) 2012 config: `logHardnessModified` 4.0 when holding a felling axe vs `logHardnessNormal` 2.0 (felling costs a 2x slower cut), `onlyDestroyUpwards` true, `destroyLeaves` true, `allowItemDamage` true (1 per block), `allowMoreBlocksThanDamage` false (tool breaks mid-tree and leaves the rest), `sneakAction` disable: https://pastebin.com/pKGtAfvp, https://www.curseforge.com/minecraft/mc-mods/treecapitator
- The family pattern: flood-fill connected logs, break them in one tick, drop items at each spot or at the cut, charge durability per block. No animation.

#### HT's TreeChop
Source: https://github.com/hammertater/treechop, README https://github.com/hammertater/treechop/blob/main/README.md
- Each hit "chops" instead of breaking: the log turns into a chopped-log block that loses radius per chop (`DEFAULT_UNCHOPPED_RADIUS` 8 px, `DEFAULT_MAX_NUM_CHOPS` 7 per block): https://github.com/hammertater/treechop/blob/main/shared/src/main/java/ht/treechop/common/block/ChoppedLogBlock.java
- Chops needed scale with log count. Default LOGARITHMIC: `chops = 1 + a x ln(1 + (blocks-1)/a)`, a = 10, rounding NEAREST. Table (a=10, NEAREST): 5 blocks -> 4 chops, 10 -> 7, 20 -> 12, 50 -> 19, 100 -> 25, 200 -> 31, 1000 -> 47. LINEAR: `chopsPerBlock` 1.0 x blocks + `baseNumChops` 0. `canRequireMoreChopsThanBlocks` false. Example: baseNumChops 8 + chopsPerBlock 0 = every tree falls after exactly 8 chops (whittling one log to nothing).
- When the count is reached the tree is "felled": all logs break at once; leaves DECAY by default (`breakOrDecayLeaves`), `maxTreeBlocks` 1024, `maxLeavesBlocks` 1024, `maxBreakLeavesDistance` 7; loot waits until felling unless `dropLootOnFirstChop`; broad axes do 5 chops per hit: https://github.com/hammertater/treechop/blob/main/shared/src/main/java/ht/treechop/common/config/ConfigHandler.java

### 3. Vintage Story
Source: https://github.com/anegostudios/vssurvivalmod/blob/master/Item/ItemAxe.cs, wiki https://wiki.vintagestory.at/Axe
- Vanilla: breaking the chosen log with an axe takes long, then the whole tree above breaks at once (no animation). Same total durability as cutting each block; the number of logs broken is capped by remaining durability. Hardwoods take 30% longer; purpleheart needs at least copper/scrap, ebony at least bronze: https://wiki.vintagestory.at/Axe
- Speed vs wood (vs leaves), durability: stone 2.6x (1.7x) 50; copper 5.2x (2.4x) 250; tin bronze 6.3x (3.0x) 400; iron 8.4x (3.2x) 900; steel 11.0x (4.0x) 1800: https://wiki.vintagestory.at/Axe
- Tree resistance: `FindTree` sums a resistance over every tree block (leaves 1, branchy leaves 2, logs 4-8 by wood tier; tier 3 softwood, 4 temperate hardwood, 5 tropical, 6 ebony), then `treeResistance = max(1, sqrt(resistance / 1.45))` divides the breaking speed. Can't cut if tool tier < woodTier - 3. (derived) a 100-resistance tree breaks 8.3x slower than one log; a 400 one 16.6x.
- `treeFellingGroupCode` (a block attribute) marks which logs and leaves belong together, so neighbouring trees of another group aren't taken; `treeFellingGroupSpreadIndex` < 2 means only the column; the search is limited to a cone (`horDist - 1 < 2 x vertDist`, or 0.5 for vertical-spread trees) and 2500 blocks; leaf groups (1-7) pick the matching canopy.
- On fell: each block breaks with dust particles in its own colour (leaves as drifting quads with wind x4, logs as cubes with gravity 0.8); leaf drop chance x0.85 per leaf (floor 0.03), branchy x0.7; a `treefell` sound plays only if > 35 blocks, volume clamp(blocks/100, 0.25, 1).
- Mods with a fall: Salty's Falling Trees animates logs and leaves as falling pieces that place as blocks or drop on impact; trees pause briefly before falling, higher parts travel farther, fall synced with the sound; domino effect (trunk must hit another trunk); `/ftfall forward|backwards|random`, `/ftdamage` 4; glass, leaves, plants, snow break in its path, wood/brick makes it "stumble"; warns redwoods "can and will fry your pc": https://mods.vintagestory.at/fallingtrees. Tree Fellas animates the tree as one asset falling in real time; logs that land become choppable logs; no damage yet: https://mods.vintagestory.at/show/mod/46225. The official forum says vanilla has no felling animation: https://www.vintagestory.at/forums/topic/1212-tree-felling-animation/

### 4. How to draw a falling tree in a block game

| Approach | Who | Looks | Costs |
|---|---|---|---|
| One entity built from the tree's own block models, rotated about the stump edge | Dynamic Trees (bakes a quad list once), Panda's Falling Trees (resubmits each block per frame), Realistic Tree Behaviour, PeterWolf's | The real tree, exact textures and tint, falls as one rigid piece; lighting is one value for the whole entity (DT uses the cut block's) | DT: one mesh build, then one draw per frame, and collision is ~24 boxes along the trunk ("about equivalent to the same number of item entities"). Panda: per-block submission each frame, fine for <= 256 logs. Needs the tree removed from the world mesh the same tick to avoid a double. |
| Per-block falling entities | FallingTree FALL_* modes, Salty's VS mod | Blocks rain down, no rotation; reads as "crumbling", not a tree falling | One entity per block; a big tree is hundreds of entities; Salty's warns redwoods "fry your pc" |
| Separate low-detail prop (Valheim) | Valheim | Standing tree mesh swapped for a log prefab with a Rigidbody; true physics, rolls, chains | A physics body per log; data authored per species |
| Voxel volume as a rigid body | Teardown | Any cut piece becomes its own object; trees and houses topple the same way | Each object is its own 3D voxel texture (1 byte per voxel, 256-entry palette), one draw call per object, rasterized as an oriented box and ray-marched inside: https://acko.net/blog/teardown-frame-teardown/, https://juandiegomontoya.github.io/teardown_breakdown.html. The engine was built around regular-grid voxel volumes for physics and graphics: https://80.lv/articles/teardown-developer-breaks-down-multiplayer-and-voxel-destruction-tech |
| Discrete voxel rotation (re-place blocks at 0, 45, 90 degrees) | (the end state in PeterWolf's and Realistic Tree Behaviour: the tree re-places as blocks after landing) | Steppy; only the final lying pose is blocks | Cheap, but mid-fall frames alias badly; used only for the landed state |

Leaves breaking off: DT spawns vanilla block particles of each leaf's own state, thrown with the tree's angular velocity x radius at impact (capped 400 leaf blocks); VS spawns colour-matched quad particles drifting with the wind on every felled leaf; Enhanced Falling Trees scatters leaf particles along the trunk when the entity disappears; PeterWolf crushes part of the canopy on impact (drops sticks/saplings) and keeps the rest near the trunk tip (sources above).

### 5. Making felling feel good
- Per-hit wobble: Valheim shakes the trunk 1 s, 1.5 degrees, decaying cubically, on every hit (TreeBase.cs above).
- Lean then fall: Panda's cos curve (slow start, fast finish over 2.5 s) and DT's angular speed scaled by 1/height (tall trees start slower) both give a lean; Salty's VS mod pauses before the fall and "higher parts travel farther" (sources above).
- Timing: 2.5 s fall + 1 s bounce of 10 degrees (Panda); DT about 3-4 s for a mid tree (derived above), bounce at 0.25 elasticity. A quick crack straight into a crash "sounds like a branch"; big trees take several seconds: https://www.asoundeffect.com/tree-felling-sound-effects/ (this page refused my fetch with a 403, so its claims here come from search-result snippets only)
- Sound in stages: crack/creak at the hinge of the cut (asoundeffect, snippet only), then the landing; OpenClonk's devs split cracking and landing and play only the crack if the tree got stuck before rotating 80 degrees: https://forum.openclonk.org/topic_show.pl?pid=19210. Panda times the impact sound 0.25 s before contact; DT plays a start sound, a splash through water, and an end sound at 1.5 volume; VS scales the `treefell` volume with tree size (sources above).
- Impact: camera shake plus thud on landing, foliage cleared where it lands: https://modrinth.com/mod/realistic-tree-behaviour. Leaf burst on impact proportional to fall speed (DT).
- Direction under the player's control: Valheim falls along the last swing; PeterWolf away from the player; Salty's VS forward/backwards/random (sources above).
- Danger: Valheim and DT make the falling trunk hurt (DT once per entity, only before the first bounce); chain felling in Valheim lets a weak axe drop a hard tree.

### What a block game can borrow
1. Gate by tier, not HP: Valheim's tree tiers (0 stone .. 4 black metal) with a "too hard" popup and XP still granted; HP sets the number of swings (4-5 for a common tree).
2. Scale chop effort with tree size but sub-linearly: TreeChop's `1 + 10 ln(1 + (n-1)/10)` or VS's `sqrt(resistance/1.45)`; avoid FallingTree's linear "every log" mode for big trees.
3. Fell as one entity built from the tree's own blocks, baked once into a mesh (DT), rotated about the far edge of the stump, ease-in fall of 2-4 s scaled by height, one small bounce, then re-place or drop.
4. Collide with a handful of boxes along the trunk only (DT: <= 24), damage once per creature by mass x speed.
5. Keep the lying trunk as logs to process (Valheim large log -> 2 halves -> wood; PeterWolf 2-3 hits per section) so felling has a second step.
6. Leave a stump (Valheim: 80-160 HP, 2 wood) and let leaves burst into tinted particles on impact, capped by count.
7. Sound in three beats: per-hit chop, crack/creak at the cut when it starts to lean, thud timed just before contact; add a short camera shake for the landing.
8. Protect builds: require leaves near the top log (FallingTree `minimumLeavesAroundRequired` 1) or a felling group (VS `treeFellingGroupCode`), and cap the scan (100-2500 blocks).
