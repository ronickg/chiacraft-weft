# Build brief: fossils, phase 1 (fossils and amber finds in the world)

For a worker. Design: `reports/Fossils, DNA and cloning.md` (sections 4, 5.1, 5.2, 10, 11).
Decided by Ronald on 11-06: Jurassic Park style fiction (DNA in fossils and amber), the ice age
first, T. rex as the late capstone. Building waits on Ronald's go for this phase.

## What done looks like

A player in the hot, dry land sees bone in the rock of a mesa or canyon wall. A brush digs it out
in about five seconds and gives "Velociraptor fossil"; a pickaxe smashes it into bone fragments
instead. Amber ore now and then holds something: an insect, a plant, a feather, or a biting insect
that fed on a dinosaur ("Amber with a blood-fed insect: Protoceratops"). The first fossil of a
species notes it in the field journal. A fossil placed on the ground stands as a display. There is
no DNA, no lab and no quality yet: those are phases 2-4.

## Read first

- `reports/Fossils, DNA and cloning.md`: sections 4, 5.1, 5.2 and 10.
- `base_api::land` (`mods/base/api/src/land.rs`): `rock_set`, `strata`, `STRATA`, `face_ore`,
  `ore_line`, and the test `face_ores_come_in_their_shares`: the pattern fossils follow.
- `mods/base/src/land.rs`: `rock_faces` and `bare` (how faces become rock, `r.decoration`).
- `mods/animals/src/gateway.rs`: amber ore and amber.
- `mods/animals/src/eggs.rs`: one item per species, made in a loop (`ITEMS`), the model for the
  fossil items.
- `mods/animals/src/journal.rs` and `animals_core::journal`: the deeds (`SEEN`, `HUNTED`,
  `TAMED`, `BRED`) a fossil deed joins.
- `mods/animals/core/src/species.rs`: each Cretaceous species' formation is in its comments
  (Djadokhta, Bayan Mandahu, Cedar Mountain, Yixian) and its `Rarity`.
- `docs/modding.md` (blocks, items, loot with `Cond::ToolTag`, `BLOCK_BROKEN`), `docs/worldgen.md`.
- The skills: `block-textures` (fossil blocks), `game-art` (icons), `blocky-model` (the display).

## You own / you don't

- You own: `animals_core::fossils` (new), `mods/animals/src/fossils.rs` (new), the amber
  inclusions in `gateway.rs`, the fossil deed in the journal, their lang lines, art and checks.
- You don't own: base's rock faces and ores (`land.rs`, `base_api::land`: read them, change them
  only if the decoration order forces it, below), the egg looks (session "Eggs: a distinct look per
  species" is changing egg art; leave eggs alone), the ley lines.

## The parts

### 1. The numbers and the rules: `animals_core::fossils` (new module)

Pure, no SDK, testable like `face_ore`.

- `FORMATIONS`: each formation, its host rocks and its species (by `Species::name`):

  | Formation | Host rocks (base's names) | Species |
  |---|---|---|
  | Djadokhta (with Bayan Mandahu) | `sandstone`, `ochre_sandstone`, `cream_sandstone`, `red_sandstone` | raptor, osmolskae, protoceratops, citipati, pinacosaurus, kryptobaatar, priscagama, gobipteryx |
  | Cedar Mountain | `banded_clay` | utahraptor, gastonia, iguanodont, cedarosaurus, crocodyliform |
  | Yixian | `limestone` | psittacosaurus, confuciusornis |

  Fish (lungfish, Lycoptera) and the wyvern: none in phase 1.
- Species weights within a formation by rarity: common 4, uncommon 2, rare 1 (a constant
  `RARITY_WEIGHT`).
- `FACE_FOSSIL`: share and knot size per host rock, as `FACE_*` in base_api: start at
  `(0.006, 1.5)` for the sandstones and banded clay, `(0.003, 1.5)` for limestone (its faces are
  everywhere in the temperate land; fossils should feel like the badlands' thing). A canyon wall of
  2,000 face blocks then shows about a dozen.
- `fossil_at(seed, [x, y, z], rock) -> Option<&'static str>`: the species whose fossil is at that
  block of that rock, or none. Deterministic from the seed and position (a 3D noise over its
  `ore_line` threshold for the share, then a hash of the position for the species), so server and
  clients agree. Use a salt of its own, not base's.
- `AMBER_FINDS`: what breaking amber ore may give besides its amber, as chances per ore block:
  plant 6%, plain insect 5%, blood-fed insect 3%, feather 2% (one roll; 84% nothing extra). A
  blood-fed insect or feather names a species: in the Cretaceous, one living in the biome where it
  was found (its habitats); in the overworld, any species of the three formations, by
  `RARITY_WEIGHT`.
- `BRUSH_SECS`: about 5 seconds for a fossil block with a brush (the brush's mining speed is
  derived from the fossil blocks' hardness so this holds). `BRUSH_USES`: 64.
- `FRAGMENTS`: a smashed fossil gives 1-2 bone fragments; 3 fragments make 1 bone.

### 2. The blocks: `mods/animals/src/fossils.rs`

- Six fossil blocks, one per host rock: `fossil_sandstone`, `fossil_ochre_sandstone`,
  `fossil_cream_sandstone`, `fossil_red_sandstone`, `fossil_banded_clay`, `fossil_limestone`.
  Hardness and sound as their rock. `mineable("pickaxe")` and `mineable("brush")`.
- Breaking:
  - with a brush (`Cond::ToolTag("tools/brush")` in the loot, or the tool in `BLOCK_BROKEN`'s
    `item`): the species' fossil item from `fossil_at` at that position (the handler drops it,
    since the species comes from the position);
  - with anything else: 1-2 bone fragments and the host rock (a loot table with
    `Cond::NotToolTag("tools/brush")`).
- Placing them in the world: a decoration in the animals mod for the overworld that, after base's
  `rock_faces`, swaps host rock for its fossil block where `fossil_at` says so. That covers both the
  face and the rock behind it (faces are rock down to 16 blocks), so a player who digs in finds
  more. **Check the order first**: the decoration must run after base's `rock_faces` (which turns
  the faces to rock). If a mod's decorations can't be ordered after base's, the fallback is a hook
  in base's `bare()`/`face_ore` that asks for fossil names; ask Planner who owns base's `land.rs`
  before touching it.
- Not in the Cretaceous in phase 1 (its rocks are alive with these animals; Jurassic fossils there
  are a later idea), and not in `MC_FOREST`.

### 3. The items

- **Brush**: a tool of class `brush`, Stone Age, from a stick and plant fibre (or feathers): both
  recipes. Durability `BRUSH_USES`. It mines only fossil blocks (nothing else is
  `mineable/brush`).
- **Bone fragments**: from a smashed fossil; 3 make a bone (base's `bone`) at the crafting grid.
  Later they add 1% genome each (phase 2), so keep them as their own item.
- **Fossils**: one item per species, `fossil_<species>`, made in a loop like eggs' `ITEMS`
  (15 species). There's no translated per-stack name, so a name per item is the way: "Velociraptor
  fossil" in en and ru. Icons: three shared looks to start (a claw for the theropods, a bone for the
  big plant eaters, a small bone for the small animals and birds), from the `game-art` skill.
  Kinds (skull, tooth, egg) and quality come in phase 4: don't add them now.
- **Amber finds**: `amber_insect`, `amber_plant` (no species), and per species
  `amber_blooded_<species>` and `amber_feather_<species>` in the same loop. Four icons. The insect
  is a biting midge or a mosquito (Planner's pick; both are known from Cretaceous amber); its name
  comes from the names tool only if it's an invented one.
- **The display**: each fossil item places a small display block of its own
  (`.places(&FOSSIL_DISPLAY_<species>)`, made in the same loop), sharing one box model per icon
  look (`blocky-model`: a bone on a low stone plinth), broken back into the item. Amber finds place
  a small amber block showing the inclusion (one model, four textures). Mounted skeletons are
  phase 4.

### 4. The journal

- A new deed `FOUND = 16` in `animals_core::journal` (`DEEDS` gets `(FOUND, "found")`): set when a
  player picks up their first fossil or amber find of a species (in the drop handler, or on
  pickup). It opens the species' entry like `SEEN` does, with its toast. For species no one can
  see alive in the overworld, this is how the journal fills from home.
- Its line in the journal: "Found as a fossil" (en, ru), and the formation's name on the entry
  ("Djadokhta Formation: wind-blown dunes"), from `FORMATIONS`.

### 5. Text

Every string a translation key, en and ru in `mods/animals/assets/lang/`: the six blocks, brush,
bone fragments, the 15 fossils and their displays, the amber finds, the deed, the formations.
Names of real things stay real (Djadokhta, Cedar Mountain, Yixian); nothing invented needs the
names tool unless you invent the insect's name.

### 6. Tagging and the project's books

- `mods/base/progression/src/content.rs`: the fossil blocks (Ground, `Not`, "breaks into a fossil
  with a brush, fragments with a pick"), brush (tier 0, Beasts, Tool, Made), bone fragments
  (Material), fossils and amber finds (Beasts, Material, found), the displays (Building). A recipe
  row in `graph.rs` for the brush and fragments-to-bone. The progression tests fail until tagged.
- `notes/overview/pieces.json`: a part under "animals": "Fossils, DNA and cloning", status
  partial, note "phase 1: fossils and amber finds in the world (brush, journal, display); the lab
  is phases 2-6", code and the report as docs; then `python3 notes/overview/build.py`.
- `notes/SYSTEMS.md`: a row "Fossils" (code, numbers, reads: rock faces, strata, species'
  rarity, habitats; feeds: the journal, later the genome vials), and a line in "what to check
  when one changes" for the strata and the species list.
- `mods/animals/SPECIES.md`: add each species' formation and host rocks to its section through
  `core/src/sheet.rs`, then regenerate (`mods/animals/web/build.sh && node
  mods/animals/web/sheet.mjs`).
- `notes/ANIMALS.md`: a short "Fossils" section pointing at the report.

### 7. Art

- Six block textures: bone showing in each host rock, painted over that rock's own tile so they
  match (`block-textures` skill; it shows a preview sheet). Ronald approves the batch.
- Icons: brush, bone fragments, three fossil looks, four amber finds (`game-art`).
- The display models (`blocky-model`): a bone on a plinth in three looks, an amber lump.
- Keep every generated image (archive rule), post sheets to the crew gallery.

## Checks (written now, run by CI; no tests on the Mac)

In `animals_core` (`tests/fossils.rs` or in-module):
- Fossils come in their shares: over a large sample of positions per host rock, `fossil_at`'s
  share is within 25% of `FACE_FOSSIL`'s.
- Every species in `FORMATIONS` exists in `SPECIES`, lays its formation's host rocks into the
  strata (each host rock is in `base_api::land::STRATA` or is limestone), and is reachable: over
  many positions every species of a formation turns up, in roughly its rarity weight.
- No fossils in rocks that hold none (granite, slate, karst's weathered limestone, stone).
- `fossil_at` is the same for the same seed and position, and differs between seeds.
- Amber finds: the chances add up to under 1; a blood-fed find in the Cretaceous only names a
  species living in that biome.
- The journal: `FOUND` fits in the deeds byte with the others; a note of `FOUND` is read back.

In the mod: the progression tags (the existing tests), lang keys present in en and ru for every
new block and item (as the other lang checks do).

Before "ready to merge": niced `cargo check --workspace --all-targets` and clippy, merge current
dev, ENGINE_CHANGES only if you had to touch `crates/` (you shouldn't).

## The game run (Tester)

Never launch the game yourself. Send Tester (or the PC queue, `t2q.py look`) your worktree and a
script, fresh world names, looks-off/no-rendering-change noted. A script along these lines:

1. `loaded`, `chat /rule core:mob_spawning 0`, survival off (`/tk mode creative` while setting up).
2. Find a strata face (base_api's `land_find` example prints where canyons and mesas are), or
   `/tk fill` a small wall of
   `animals:fossil_red_sandstone` beside the player on flat ground, then `/tk mode survival`.
3. `/tk give animals:brush`, look at the fossil (`look <yaw> <pitch>`: negative pitch looks down),
   `hold attack 6`, `shot`: the fossil item dropped, the journal toast.
4. `/tk give base:iron_pickaxe`, break another: fragments and rock.
5. `/tk give animals:amber_ore 64`, place and break a stack with a pickaxe: count the finds in the
   run log.
6. Place a fossil: `shot` of the display.
7. A natural face: teleport to a canyon wall found by `fossil_at` near spawn (write a small
   `cargo run -p animals_core --example fossil_find -- <seed>` that prints the nearest fossil
   faces, as base_api's examples do) and `shot` the wall from 8 blocks.

What to look at: the drops (run log), the toasts, the six textures in daylight, how many fossils
one canyon wall shows (should read as "a few, worth looking for", not a polka-dot wall).

## Report to Planner

SendMessage to Planner (`local_a93a48fb-ed3e-4140-9fea-72a4beba73c9`) at each finished part and
with any question: the state, the commit (copied from git output), what was checked and how, what
wasn't, and the paths of the art sheets and shots.
