# The first hours

Status: **decided by Ronald** (11-05, relayed by Planner): "B: about 13-15 min", "Knapping
screen, then book", "Fire bed + bow drill", "Knapped stone pick, no chopper". Slices 1-3 built
(`mods/base/src/first.rs`, `knapping.rs`, `firebed.rs`; numbers `vitals::first`), waiting for PC
runs. The research behind it is `reports/The first hours.md`.

**As built, where it differs from the proposal below:**
- The fire bed is three sticks and a fibre (the 2 × 2 holds four).
- The knapping screen opens by sneaking and striking a flint nodule on the ground with a rock. It
  knaps an axe head (8 squares away), a pick head (13) or a shovel blade (8). A click inside the
  outline breaks the nodule into a flake. No knife blade: a knife is a flake on a stick.
- After a head of a shape has been held, flint makes it from the book or the grid (1 flint an
  axe head, 2 side by side a pick head, 2 stacked a shovel blade), from memory. It's instant;
  a craft time waits for the book's queue.
- No shaved-haft step: the stick in the hafting recipe is the haft (`branch` wood).
- The lit bed becomes today's campfire. Today's campfire recipe (two logs, two sticks) still
  places a lit one; lighting it with the drill is Survival's campfire, for later.

Ronald (11-05): "you can't just punch down a tree. You first got to get a rock, and then if you
want to create an axe, well, what do you need for axe? ... wood, sticks, you need ropes." And:
"a bit of more survivally, but takes a bit longer ... you got to feel like you earned it."

## Why now: dev has no way in

Dev (7e5740a1) has a dead end for a new survival player:
- bare hands fell trees fruitlessly, so there are no logs;
- planks, the table, every tool and the campfire need logs;
- stone needs a pickaxe;
- every tool recipe is 3 × 3, so it needs the table.

The `hint.base.logs` hint still says to punch a trunk. This design is the way in.

## The path, in one picture

`notes/first-hours/path.png` (source `path.html`).

```
look around ─┬─ cobbles (hammerstones) ─┐
             ├─ flint / chert nodules ───┼─ strike ─> sharp flake ─ cuts grass ─> fibre ─ twist ─> cordage ─┐
             ├─ fallen branches, leaves ─┴─────────── sticks ── shave with the flake ─> haft ───────────────┤
             └─────────────────────────── knap the nodule (first time by hand) ─> axe head ─────────────────┴─ lash ─> STONE AXE
sticks + tinder (fibre, birch bark) + bow drill (2 sticks + cordage) ─> the first fire
```

## 1. What lies on the ground

Small blocks on the surface, picked up by hand at once, like flowers. They don't grow back
(a later option: a river bank brings new cobbles each spring).

| Thing | Where | Gives | Real basis |
|---|---|---|---|
| **Loose cobble** | anywhere with rock under the soil; thicker on slopes, river banks and under cliffs; none on the tundra, savanna and swamp (bone starts there, `CONTENT.md` decision 4) | 1 cobble of the local rock (granite, limestone, basalt ...): a hammerstone, a ring for the fire, a throw | river cobbles and scree are every knapper's hammerstones |
| **Flint or chert nodule** | in gravel and limestone country, river banks and beaches; obsidian in the desert and on ocean islands | 1 nodule: the core you knap | flint weathers out of chalk and limestone; chert of the plains' gravel (`CONTENT.md`, "Where the materials are") |
| **Fallen branch** | under trees, more under old and dying ones | 1-2 sticks | deadfall, not cut wood: what people burned and hafted first |
| Grass, sedge, cattail, fern, liana (there now) | as now | fibre (below) | nettle, grass, lianas, bast |

**At the spawn, always:** within 24 blocks at least 4 cobbles, 2 nodules, 3 fallen branches and
grass. A worldgen check holds it, for the spawn and for 50 random temperate spots.

**Gravel** broken by hand or with a shovel gives a nodule 10% of the time (Minecraft's flint
from gravel).

## 2. What bare hands do

- **A trunk:** nothing comes off. The first try shows the hint "Bare hands won't fell a tree.
  Look on the ground: stones, flint and fallen branches." (The slow, fruitless chop stays as
  Felling built it.)
- **Leaves:** a stick 20% of the time by hand (5% today). With the leaves' 0.3 s break, about 1.5
  s a stick.
- **Grass and other fibre plants:** fibre 12.5% by hand (as now), **50% with a sharp flake or a
  knife**: pulling grass works, cutting it works four times better.
- **Gravel, sand, dirt:** by hand, as now.

## 3. Knapping

Two kinds, as in real life: quick flakes struck in the world, and shaped heads made by hand the
first time (CRAFTING.md's 3A: hands-on the first time only).

- **A sharp flake:** hold a cobble, use it on a nodule lying on the ground or on a stone block,
  or put a cobble and a nodule in the 2 × 2 grid.
  - Each strike is about 1.5 s of holding use, with a crack and chips flying.
  - Flint gives a flake 80% of the time, chert 75%, obsidian 70%; a miss is a crack and nothing.
    Plain stone (granite, basalt) gives one only 25%, and a poor one: it says so.
  - The flake is a cutting tool in the hand: grass to fibre, a stick shaved to a haft, meat off
    a carcass, bark strips. It wears out in about 40 uses.
- **A head** (axe, knife, shovel, pick): the knapping screen.
  - A 5 × 5 nodule: click away the squares outside the head's outline; a wrong click spoils it.
    It's TerraFirmaCraft's rock knapping, smaller, with the outline shown the first time.
  - About 12 clicks, 20-40 s the first time.
  - After the first good head of a shape, it's in the book: 8 s from a nodule and a cobble.
  - The work score (CRAFTING.md) comes from how clean it was: a first try is Rough to Sound,
    a careful one Good.

## 4. Fibre into cordage

- **Cordage** (today's `string`, which the parts already take as plant cordage): 2 fibre twisted
  for 3 s in the 2 × 2 grid.
- Other cordage later, no new step for the axe: sinew from a hunted animal (its trait, shrinks
  tight), bast stripped from a linden or willow log with the axe, a liana cut whole in the
  tropics.

## 5. Hafting

All in the inventory's 2 × 2, so no table is needed before the first axe.

| Tool | Head | Haft | Lashing | Time | What it does |
|---|---|---|---|---|---|
| **Stone axe** | knapped axe head | a shaved stick | 2 cordage | 6 s | fells trees up to 96 logs (Felling's stone tier), strips bark |
| **Stone knife** | a flake, or a knapped blade | a shaved stick, short | 1 cordage | 4 s | cuts grass, butchers, carves; lasts 3× a loose flake |
| **Stone shovel** | knapped flat blade | a shaved stick | 2 cordage | 6 s | digs soil, sand and gravel faster |
| **Stone pick** | knapped pointed head | a shaved stick | 2 cordage | 6 s | breaks stone (the game needs it; really an antler pick or a hammerstone) |

- **The haft** is a stick shaved with the flake, 4 s: a part with its own wood material,
  "branch" (green wood, 10% less durable than a split haft). Hafts split from a log at the table
  (as now) replace it later.
- **The parts are CRAFTING.md's:** head, haft, fitting (plant cordage: "cheap", rots when wet).
  So the first axe is already a parts tool, with a grade and a history ("made on day 1").
- **The fitting wears first:** re-lash with fresh cordage, as with any parts tool.

## 6. The first fire

- **A fire bed:** 4 sticks and 1 tinder (fibre, dry grass or birch bark) placed on the ground.
  It isn't lit.
- **A bow drill:** 2 sticks and 1 cordage. Hold use on the fire bed: smoke, then an ember.
  - 6 s, 70% of tries take; a hand drill (2 sticks, no cordage) takes 10 s and 35%.
  - Wet tinder or rain on the bed: no ember, and the hint says why.
- **Lit, it's today's campfire.** Logs on it later (Survival's fuel, when it has one).
- **Today's recipe stays** (2 logs and 2 sticks), but it needs lighting too, with the drill.
- The dusk hint names the drill.

## 7. What comes after

| Tier (harvest) | First item | Opens |
|---|---|---|
| Hands | flake, hafted stone tools | logs, planks, the workbench (the table), stone |
| Stone and flint (100) | stone axe, pick, knife, shovel | trees to 96 logs, stone, the furnace |
| Bone and antler (50) | bone knife and awl; an antler flaker (better knapping grades) | hides, sinew, sewing (Survival's clothes) |
| Fire and clay | the kiln, ground stone axe (tougher) | pottery, charcoal |
| Copper (150) | native copper, cold-hammered | young sequoias (192 logs) |
| Bronze (200) | casting | |
| Iron (220), steel (300) | bloomery, forge | the grown sequoias |

- **Vanilla's grid tools go.** Wooden and cobblestone tools from the 3 × 3 grid are replaced by
  parts tools. A wooden club and digging stick stay as tier 0, carved from a log.
- **Hints, in order** (each once):

  | When | Hint |
  |---|---|
  | 20 s in | Your hands can't fell a tree. Look on the ground: stones, flint and fallen branches. |
  | a cobble and a nodule held | Strike the flint with a stone (use it on the flint) for a sharp flake. |
  | a flake held | A flake cuts grass for fibre. Two fibres twist into cordage. |
  | a nodule, cordage and a stick held | Knap an axe head from the flint, then lash it to a shaved stick. |
  | the first axe | Chop the foot of a tree. Bigger trees take longer. |
  | dusk, no fire near | Night is coming. A fire bed (4 sticks, tinder) and a bow drill make a fire. |

## The timing

A new player who doesn't know the way, in a temperate spawn; then one who does. The first day
runs at half speed from 10:00 to dusk, about 13 real minutes.

| Step | New | Knows the way | How the time is set |
|---|---|---|---|
| Look around, pick up 2 cobbles and a nodule | 2.5 min | 0.5 | spawn guarantee, 24 blocks |
| 3 sticks (branches, leaves) | 1.5 | 0.5 | 1-2 per branch, 20% per leaf |
| First flake (a couple of misses) | 1 | 0.1 | 80% per 1.5 s strike |
| 8 fibre (16 grass cuts with the flake) | 1.5 | 1 | 50% per cut: 3 cordage and the tinder |
| Twist 3 cordage | 0.5 | 0.2 | 3 s each |
| Knap an axe head (one spoiled try) | 2.5 | 0.5 | the screen, ~12 clicks |
| Shave a haft | 0.3 | 0.1 | 4 s |
| Lash the axe | 0.5 | 0.1 | 6 s |
| Reading the hints and the book, getting lost | 3 | 0 | |
| **To the stone axe** | **about 13 min** | **about 3 min** | |
| Fire: 4 more sticks, tinder, a bow drill, the ember | 3 | 1 | |
| **To the first fire** | **about 16 min** | **about 4 min** | dusk at about 13 |

So a new player has the axe as the sun goes down, and the fire just after: the first night is
the deadline you feel.

## Choices for Ronald

1. **The pace.**
   - A. About 8-10 minutes: any loose stone knaps, no nodules.
   - **B. About 13-15 minutes (recommended):** cobbles are hammers, flint and chert are cores.
   - C. About 20+ minutes (TerraFirmaCraft): B, plus the haft must dry a game day, and fibre
     must be retted.
2. **Knapping a head.**
   - **A. The knapping screen the first time, then the book (recommended,** his 3A).
   - B. Strikes in the world only: a head is a lucky flake.
3. **The first fire.**
   - **A. A fire bed lit by a bow or hand drill (recommended).**
   - B. Today's campfire, lit when placed.
4. **Stone for the pick.**
   - **A. A knapped stone pick (recommended):** less real, but stone and the furnace open on day 1.
   - B. Antler picks only (real): stone waits for the first deer.
5. **A hand chopper** (an unhafted hand axe, the oldest tool): fells trees up to 8 logs slowly,
   for its wood, before you have cordage.
   - **A. No (recommended):** the axe is the moment.
   - B. Yes: a gentler start for the lost player.

## Built in slices (after his yes)

Numbers in `mods/base/vitals` (`vitals::first`), strings as keys (en, ru), each slice with a
check and a PC run from a fresh world timing the steps.

1. **The way in:** loose cobbles, nodules and fallen branches on the ground (with the spawn
   check), leaves' sticks, the flake by striking, fibre by cutting, cordage, the 2 × 2 hafting of
   the axe with a shaved stick, the bare-hand hint. Today's dead end is gone after this slice.
2. **The knapping screen** (with UI): heads by hand the first time, the book after.
3. **The first fire:** the fire bed, the drills, tinder, the dusk hint.
4. **The rest of the kit:** knife, shovel, pick; vanilla's grid tools go; the hints in order;
   the recipe book's hand tab.

**Who else:** Felling (axes, chop time; nothing changes for it), Plants (the ground items'
places, beside its layers), UI (the knapping screen, hints, the book), Art (icons and the ground
blocks: cobble, nodule, branch, flake, heads, haft, bow drill, fire bed, tinder).
