# Carcasses and skinning

Ronald (11-25): "when a animal is knocked or dead … chops up the model or when animals feed on it that it becomes like a
carcass … skinning in a way like Red Dead". Branch `claude/animals-carcass` (from dev 7f9323c5c).

## How it works today (engine and mods read 11-25)

- Death is one server tick (`crates/engine/src/server/dim.rs:2076-2163`): the `DIE` question (a Deny leaves it at
  1 health), the loot table rolled and dropped, the entity removed. Mods hear the `DIE` *event* after removal (the
  body can't be read: why `hunt.rs` keeps `PREY_KG`). The 1 s death clip and poof are client-only.
- A hunter's kill is eaten at once in the `DIE` event (`hunt.rs`), by mass, shared with the pack near.
- What's left is items: hides, meat, or `animals:carcass` pieces (500 g of carrion each) from the species' loot
  table (`lib.rs`). Vultures smell dropped carrion items within 64 blocks (`birds.rs` `scent`, `forage::ground_food`).
  Items vanish after 5 minutes and are never aged on the ground.
- `vitals::rot` already has the carcass's stages by time (`Carcass::{Fresh, Bloated, Remains, Bones, Gone}`,
  `carcass_stage`, the yield per stage), unbuilt ("for Animals' wave two").
- Knocked-out animals never die (torpor), so they stay whole as they are.
- Usable SDK pieces: prop entities as eggs/nests are (`eggs.rs`), `w.set_appearance(id, kind)` to draw one kind as
  another (dog poses, bird flight), `entity_skin` repaints from a synced component (coats), client `bone_scale`
  (0 hides a bone), `w.set_scale`, `r.pose(..).speed(0).action("kneel")` + `w.set_pose` (freezes and turns the
  player), `r.camera(..)` + `w.force_camera`/`release_camera` with `fixed`/`path` shots, timed use (`use_ticks`)
  with a `before_event(USE_ITEM)` noting the target (the fire drill does this for blocks), `base:knife` (part-built,
  materials with tiers in `vitals::gear::MATERIALS`).

## The design

**A carcass is an entity**, `animals:carcass_<plan>`, spawned where an animal died, lying on its side at the yaw it
had. It is read from the `DIE` *question* (before removal: species, mass, yaw, coat), so it needs no guessing.
It keeps: species, body mass, kg eaten, when it died, how it died (for pelt quality), skinned or not.

**Stage** = the later of two clocks, both numbers in `animals_core::carcass`:
- by what's been taken: `eaten / (mass * meat::YIELD)`: whole < 0.25 ≤ opened < 0.65 ≤ mostly bones < 0.95 ≤ bone pile;
- by rot (`vitals::rot::carcass_stage` at its spot's air): fresh/bloated look whole or opened (darker when bloated),
  remains = mostly bones, bones = bone pile, gone = despawn.
A skinned carcass is at least "opened".

**Who takes from it:** the killer and its pack (their meal now writes eaten kg on the carcass; later they eat at it
over seconds), vultures and other scavengers (scent finds carcass entities as well as items, `eat` bites kg off it),
the player (skinning, then butchering takes the rest).

**Skinning (slice 2):** hold use with a knife on a carcass. Kneel pose (`speed 0`, turned to it), camera eases in
close and low beside the body (a `fixed` shot from a point beside the carcass), a knife arm loop, the carcass steps to
"opened" mid-way, then stand up and camera back. Gives a pelt/hide, meat and parts by species and mass (today's loot
rows move from death to the carcass for species big enough to leave one). A knife's tier sets the time
(flint slow … steel fast). **Pelt quality** 1-3 like RDR2: the animal's own quality (its level/genes) capped by how
it died (one clean blow or arrow: up to 3; many hits or a blunt kill: 2; blown apart/burnt/rotten: 1) and by the
knife (flint can't give a 3). Quality goes on the hide item (`ITEM_LEVEL`) and feeds clothes and trade later.

## Art needed (blocky-model, one per body plan, not per species)

Carcass models lying on their side, each in 4 looks (whole, opened with ribs, mostly bones, bone pile), scaled per
species by mass and painted from the species' coat colours (`entity_skin`), so 5 plans × 4 looks:
1. quadruped mammal (deer, wolf, cat, dog, hare scaled down, capybara, jaguar),
2. bird (wings folded, legs up),
3. biped dinosaur (raptors, citipati, iguanodont),
4. quadruped dinosaur/reptile (ankylosaurs, sauropod, caimans, crocodyliform, lizards),
5. fish (flat on its side; fish rot fast, no skinning).
Player: a kneel pose, a two-handed knife "skin" loop (~1.5 s, cuts along the body), a stand-up; added to base's
`player.animation.json` (one file per kind: base owns it). Later: a pelt over the shoulder, flies over a bloated one.

## Taken from others (method and ideas only; no assets or code)

- **RDR2:** pelt quality stars from the animal's quality, a clean kill and the right weapon; kneel + close camera;
  the skinned carcass stays in the world for scavengers and rots; big pelts carried on the shoulder/horse.
- **Butchery (jmods):** stages shown on the model as it's worked; a hook/rack to hang a carcass for a better yield
  (later, a base block).
- **Butcher's Delight:** a carcass placed on any surface is worked with any knife-tagged item (we: any `knife`).
- **Carcass (Forge):** kills leave a carcass instead of instant drops; tiered hunting knives.
- **Flaying:** carcasses rot and vanish after a set time; which tools skin is a list (we: knives, a flake slower).
- **BioButchers Delight:** carcass → skinned → skeleton looks (our stages).
- **Meat Your Maker:** predators drawn by blood scent (ours: scavengers' scent already; hunters could follow it).

## Slices

1. **Carcass entity and stages** (this branch first): the entity per plan with placeholder box looks (4 per plan,
   swapped with `set_appearance`), read at the `DIE` question, stage by eaten kg and by rot time, hunters' meals write
   eaten kg, vultures find and eat carcasses, gone at the end. Loot unchanged for now: a player's kill drops its items
   as today and leaves the carcass already "opened" (as a skinned one in RDR2). A `/animalprof carcass` line per
   carcass for checks. Tester: a kill by wolves, a vulture at a carcass, a player kill, a long wait to bones; shots
   of each stage.
2. **Skinning:** knife hold, kneel, camera, clip, yields moved from loot to the carcass, pelt quality.
3. **Art:** the blocky-model carcasses per plan and the player clips (the art crew / `blocky-model`).
4. Later: rot tint and flies, carrying a pelt, a hanging rack, hunters following blood.
