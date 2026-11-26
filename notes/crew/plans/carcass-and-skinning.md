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

## Where it stands

- Slice 1 (912b84f55): built; Mac run cov_carcass_r1 sent 11-25.
- Slice 2 (Ronald 11-25: "Yes, skinning only"): built on the same branch. Click to start (RDR2 holds the button; ours
  runs on once started, stopped by walking off), kneel pose `skinning` with base's `skin` clip, camera `skinning`
  (low beside the right shoulder), knife tier from `base_api::ITEM_LEVEL`, hide quality on `animals:pelt_quality`,
  hints `hint.animals.pelt_1..3`. The progression sheet's kill yields stay true (skinning gives the same hide, meat or
  carcass pieces as the death loot did, at good quality).
- Runs: r4 (f0c719804) had wolves kill and open a carcass (14.4/38.1 kg), vultures eat at one, rot Opened 3 d →
  Bones 5 d → Pile 9 d → gone 13 d, skinning (2 hides quality 2 with flint, 4 venison), no drops at kills; but the
  body stood up (the held death clip doesn't play on a swapped appearance) and the camera sat in the arm. r5
  (1804dc789): the client rolls whole/opened bodies onto their side (`SHOWN` synced, root bone turned and lifted),
  the camera stands out past the arm; shots 2, 4 (lying), 5 (ribcage), 6 (pile), 8 (kneel with knife), 9 (skinned
  ribcage) pass. r5's wolves wounded the deer but didn't kill it in the time (r4 did): a chance, not a regression.
- r6 (Planner on r5's shots): an opened body lies on a pool of blood and guts (`carcass_gore`, a second entity, until
  the art slice draws the opened body); the skinning camera a three-quarter view from the right with the knife and
  the body in frame; part 1's deer wounded to 30 % before the wolves come.
- r6 (a1eff5304): part 1's wolves killed and opened their deer (carcass, meal, Opened); the pool shows under opened
  bodies (shots 1, 4, 8) and is gone at Bones and after skinning; the camera shows the kneeling skinner side-on with
  the knife and the body. The red was too bright (darkened after). The body lies at the pool's edge, not its middle:
  rolling the model about its root moves it off its entity's centre, which the art slice's lying models fix.
- Slice 3, first batch (Planner on r6: the kneel disjointed, the whole deer legs-up, the body off its pool): the
  `skin` clip kneels on the shins (hips at 3 px, legs flat behind; the torso, which hangs from the neck, leans only
  10-12 degrees, so it stays on the hips). Whole and opened bodies are now one lying model per body plan
  (`source/carcass/body.py`: four-legged mammal, bird, two-legged dinosaur, reptile on its belly, fish; opened: the
  upper side cut open, ribs across the meat, guts beside it), centred on the carcass, scaled by the cube root of mass
  against each plan's reference animal (`animals_core::carcass::Plan`), and painted in the animal's own coat and
  underside (its genes' phenotype read at death into `carcass_coat`, since replaced by the whole genome: see the third
  part). Sheet: `source/carcass/sheet.py` (archived in
  ~/Coding/chiacraft-archive/images/carcass/2026-11-26/). cov_carcass part 5 kills one of each plan for shots.
- r7 (cd142572e): nothing sinks (low shots of each stage on the grass), each plan in its own coat; the knife missed
  the deer (it lay 2 blocks off the aim). r9 (e2549bca5, every shot with the HUD hidden): skinning runs (7.9 s, flint,
  1 hide at quality 1 from a level-33 deer, 4 venison; the venison taken counts as eaten, so it steps to Bones); the
  kneel view shows the deer on its side, belly and legs toward the skinner, on its pool; the five plans' opened bodies
  rest on the ground.
- Planner on r9: the kneel fell apart (Weft's humanoid has no parent bones and its body hangs from the neck: r9's
  10 degree torso and 38 degree head tipped apart over flat legs). Refitted by `mods/base/source/player/skin_pose.py`:
  torso, head and arms turned 30 degrees about the hips with position keys holding each joint, knees down behind,
  hands at the body. Slice 3, second part: bones and pile per body plan (`body.py`: a ribcage on its side, skull,
  legs' long bones with their hooves or claws, a little meat, a scrap of the animal's own hide or feathers that the
  coat painter turns to its colour; the pile a skull, a few long bones and the hide), replacing slice 1's one
  placeholder pair. cov_carcass shots 14 (bones) and 15 (pile) of each plan's row.
- Slice 3, third part (Planner 11-26, "spotting and patterns on dead coats next"): the carcass keeps the animal's
  whole genome (`carcass_genome`, copied at the DIE question), and the client paints it with the living coat painter
  (`animals_core::coats::paint_dead`) on the carcass models' own layers (`genetics/layers/carcass/`, written by
  `body.py layers`: each coat texel's model position, folded from lying back to standing so stripes, rosettes, white
  feet and blazes land where they grow, its underside share, white and points zones), keeping the shared texture's
  painted light by its brightness. Offline: `cargo run -p animals_core --bin carcass_coats` then `sheet.py --coats`
  (jaguar rosettes, tabby and white cats, striped raptors, archived).
- Rot drawn (Planner 11-26, "rot tint and flies" first of the four): the body's coat is repainted at each rot step
  (`animals_core::carcass::rot_step`: 0 fresh, 1-5 through the bloat, 6 at remains), turning toward a dull green-black
  as it bloats and a dry grey-brown at remains, bright texels darkening most (`rot_tint`, `BLOAT_RGB`, `DRY_RGB`,
  `ROT_DARKEN`, pale bellies too, by `DRY_DARKEN` at remains), the opened flesh and ribs rotting with it, a dull
  red-brown through the bloat and a dark dried brown at remains (`MEAT_ROT_RGB`, `MEAT_DRY_RGB`; Planner on
  sheet_rot: the meat stayed fresh red, the bellies went mint); the blood pool under it too (`gore_tint`: darker
  red-brown through the bloat, a dry dark stain under the remains' bones, gone with the pile; `has_gore`, synced as
  `carcass_gore_rot`; offline `carcass_coats --gore`); blow flies dart over it while it bloats (`FLIES` over a deer-sized body, as many again per body as big
  by area, `DRY_FLIES` of them at remains, none at bones), drawn on each client as short-lived particles
  (`carcass_fly`). The step and the flies ride in the synced `Dead` (`carcass_genome`) with the genes, so a step
  repaints the coat. `/animalprof carcass` lists both. Offline: `sheet.py --rot`. The golden eagle's coat now has its
  coverts' paler mottling (living and dead: a plain brown read flat).
- Vultures (Planner 11-26, second of the four): a carcass is eaten a mouthful at a time, each the eater's own
  (`animals_core::carcass::bite_kg`: `MOUTHFUL` 3 % of its mass, no more than it still needs, at least
  `MOUTHFUL_MIN_KG`; a turkey vulture 40 g, a wolf a kilo), one a chew, so a 1.4 kg vulture feeds four mouthfuls to
  its 140 g meal; the fixed 0.5 kg bite filled it at once and took the rest off the body for nothing. Check:
  cov_carcass_vultures (part 2 alone, watched 4 minutes).
- Hide quality (Planner 11-26, fourth of the four): the skinned stack carries `base:hide_quality` (base_api, was
  `animals:pelt_quality`), kept through tanning (`crafts.rs`: leather of one quality stacks alone) and sewing
  (`clothing.rs` `crafted`, the crafting grid and the book: a garment as good as its worst hide, an unknown one as
  good); fur and leather clothes and cloaks warm by `vitals::clothing::HIDE_WARMTH` (poor 0.85, good 1, perfect
  1.15) in `survival/temperature.rs`. The tooltip (`tagged.base.hide_quality`, en and ru) gives stars and the warmth.
- Blood trails (11-26, branch claude/animals-blood, "hunters following blood"): a blow struck by something of a fifth
  of its max health or more opens a wound on a wild animal (as a player's), bleeding 30 s at a fifth and longer for
  heavier, 120 s at most, 0.4 % of max health a second (`animals_core::blood`): one hit for 62.5 % or more bleeds out
  and leaves its carcass where it fell. Bleeding as it goes it drops a spot of blood (`blood_spot`, a flat splash drawn 1.8 times its 0.4-block model, `SPOT_SCALE`) every
  3 blocks along its way (`SPOT_GAP`, at most 2 a second), shrinking as it dries over 5 minutes, then gone. A hungry hunter with no prey in sight
  smells spots 24 blocks off, walks to the nearest and on along the trail spot by spot in the order they fell, to prey it takes or to a carcass
  (`hunt.rs`, before kill sites). Check: cov_blood (`/animalprof gash`, `/animalprof blood`).
- Pickup shown (11-26, branch claude/animals-pickup): what a skinning takes off is tossed from the body toward the
  skinner (`animals_core::carcass::OFF_SPEED`, `OFF_UP`), fanned apart, and picked up `OFF_PICKUP_TICKS` (1.25 s)
  later, so it is seen to come off and land. cov_carcass_skin shots 8b_off, 8c_off.
- Laid clear of walls (11-26, pt_rain_4: a deer that died against a stone wall lay half in it, out of the knife's
  reach): `animals_core::carcass::clear_spot` moves a carcass to the nearest spot its lying box is clear of solid
  blocks, up to `CLEAR_REACH` (1.5 blocks, or half its width), toward the side the animal stood. cov_carcass_wall.
- Left for later: species without stagger-break rows drop nothing on a break.
