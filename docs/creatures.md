# Making a creature

How the chihuahua was made, as a recipe for the next animal: a few dozen boxes and flat fur
panels with a hand-painted pixel texture (the look of Hytale, Minecraft and Vintage Story), built
and animated in Blockbench, exported to the Bedrock files the engine reads ([models.md](models.md)).

The rules, scripts and checklist live in the **`blocky-model` skill**:
[`.claude/skills/blocky-model/SKILL.md`](../.claude/skills/blocky-model/SKILL.md). In Claude Code,
asking for "a fox like the chihuahua" (or anything Minecraft- or Hytale-style) picks it up. Read
it first; this page is the chiacraft side.

## What you need

- Blockbench (`brew install --cask blockbench`), started with a debug port so scripts can drive
  it: `/Applications/Blockbench.app/Contents/MacOS/Blockbench --remote-debugging-port=9333 &`.
- Python 3 with `numpy pillow scipy websocket-client` (a venv is fine).
- Concept art of the animal. Best is a flat orthographic turnaround; the skill has the prompt that
  gets one from an image model.

## The steps

0. **Research** the animal (the skill's first step): real measurements, how Minecraft and
   Hytale make it, what makes it recognizable. Write it into the source README.
1. **References**: a flat turnaround (front, back, sides, top) and, if you can, a blockout
   (fur removed). Crop each view.
2. **Boxes**: lay the parts out in code as boxes and quads on bones (`head`, `jaw`, `ear_L`,
   `tail_0`, `leg_FL_upper`...), with the skill's creature defaults: designed eyes and eyelids,
   a jaw that opens, solid ears or wings, toes, designed fur edges. `boxcheck.py` must say 0.
3. **Paint**: color areas from the sheet, painted with soft light, hue-shifted shadows, ambient
   occlusion and fur streaks (`paint.py`); eyes from the eye library.
4. **Animate**: idle, walk, run, sit, lie, plus extras (blink, lick...). Fit resting poses to the
   ground (`ground.py`, `rig.fit`), swing tails about the right axis (`rig.swing`).
5. **Build in Blockbench** and look at it from every side (`bb.py`, `bbshot.py`).
6. **Let people pick**: publish the viewer (`viewer.html`) with texture variants
   (`variants.py`) for anything that's a matter of taste (eyes, coats).
7. **Export to the game** (`to_bedrock.py`), register the entity, check it in the game.

## Into the game

```
python .claude/skills/blocky-model/scripts/to_bedrock.py spec.json texture.png mods/<mod>/assets \
    --name <animal> --scale 0.17 --texture-name <mod>/<animal> --pup 0.5 --pup-head 1.3
```

writes `models/<animal>.geo.json` (and `_pup`), `animations/<animal>[_pup][_sit|_lie].animation.json`
and `textures/entity/<mod>/<animal>.png`. It takes care of what the engine needs:

- **Facing**: Bedrock models look along -Z; Blockbench's Hytale format along +Z. Turned 180°.
- **Tilted boxes**: the engine only rotates bones, so every tilted box gets its own `_tilt` bone.
- **Size and detail**: the house style (the skill's "House style", `scripts/house.py`) is one
  texel density for every model, **32 texels a block**, the same as the blocks and items (a block
  is a meter). Lay the model out in units of 1/64 block at the real average adult's size, export
  with `house.SCALE` (0.25) and paint `house.TX` (0.5) texels a unit. Small creatures whose face
  would be under 3 texels paint at 64 or 128 (`house.fine`; today the raven, gull, vulture and
  fish at 64, the mouse, vole, kryptobaatar, small bird and lizard at 128). Individuals then
  differ around it with the engine's per-entity scale (`World::set_scale`), from their size gene
  and sex (`coats::size`; the species' `dimorphism`, `size_spread` and `height` in `species.rs`).
  The chihuahua, cat, both raptors and the capybara keep their own layout scales and paint
  `house.texels(scale)` a unit, so they are at 32 too.
- **Coats**: every build writes its coat layers (`genetics/layers/<model>`) in its texture's
  layout, so a new texture size needs no code change; `cargo run -p animals_core --example
  coat_sheet -- <species> <texture.png> <out>` paints a dozen wild coats (the last few young) as
  the game does, to look at on the model.
- **Props** (nests, eggs, anything without a skeleton) are entities built with the skill's
  `prop.py`, not block models (block models here take only coarse texture coordinates).
- **Fur panels**: both sides of a flat panel get the picture (mirrored to line up).
- **Poses**: the engine plays `idle`, `walk` and `run` by itself: walk and run by speed (run
  takes over between 3 and 4.5 blocks a second), on top of the idle, so the exporter writes them
  as offsets from the idle's first frame. A pose (sit, lie) is its own file whose `idle` holds
  it, used by its own look kind (`set_look`). Actions like `lick` are in the standing file for
  `World::play_animation`, which replaces idle while it plays (so an action on a sitting dog
  stands it up for that time). Blinks are part of each idle.

Register it like the cat (`mods/animals/src/cat.rs`, the simplest) or the chihuahua (`mods/animals/src/dog.rs`): `.model_file("<animal>")`,
`.animations("<animal>[_sit|_lie]")`, `.texture("<mod>/<animal>")`, and a hitbox `.size(..)`
about the model's footprint. Bones named `head` follow where the creature looks; don't name
bones `leg0`..`leg3` unless you want the engine's own leg swing (an animation with `walk` turns
it off anyway).

Its coat comes from genes: a gene file in `mods/animals/genetics/` and its build's layers copied
to `genetics/layers/<model>` (the build scripts do it), both listed in `src/coats.rs`, with a
phenotype rule there (and in the viewer's `coats.js`).

Then give it stats: a row in `mods/animals/src/species.rs` (its place in the tree of life: family,
subfamily and scientific name, looked up, not guessed; base values of the nine stats, its diet and
favorite foods, how it is tamed; see `notes/ANIMALS.md`),
`.then(crate::animal::upkeep(<its row>)).then(crate::taming::behave(<its row>))
.then(crate::commands::act(<its row>))` first in its goals (its wild level and stats; asking
for food and running off when scared; a tame's fighting, following and staying),
`.then(crate::commands::hold)` just before wandering, an `.on_interact` that calls
`crate::screen::interact`, else `crate::taming::feed`, else `crate::commands::interact` (the
spyglass and animal screen, feeding to tame, a tame's food and follow/stay),
`crate::taming::hurt` in its `.on_hurt`, an `.attack_damage(..)` if it fights, and a `wake` that
gets it up from resting (see `cat.rs`), listed in `wake` in `lib.rs`. A
new food needs its bubble: `textures/particle/want_<item>.png` (the bubble with the item's icon).

To see it: build (`cargo build --release`), then run a script that spawns it and takes shots:

```
loaded
chat /dim base:flatlands
wait 6
idle
chat /tk spawn <mod>:<kind> ~ ~ ~3
wait 2
look 0 25
shot <animal>.png
quit
```

`cargo run --release -- --world creature-check --script that.txt` writes `run/screenshots/<animal>.png`.
F3+T reloads models, animations and textures while the game runs.

## The worked examples: the chihuahua and the cat

Both live in the `animals` mod, each with its source next to the assets:

- [mods/animals/source/dog/chihuahua](../mods/animals/source/dog/chihuahua/README.md): built from a
  concept sheet (colors read from the flat turnaround). `tools/blocky.py` lays out and paints the
  dog (eyelids, jaw and tongue, solid ears, a V bib, toes; `EYES`, `BLINKS`, `lick_keys`).
- [mods/animals/source/cat](../mods/animals/source/cat/README.md): built from research alone (no
  sheet): proportions from real cats, the look from Minecraft's and Hytale's cats; its colors are
  rules (`area()` in `cat.py`: tabby stripes, the M, white bib, belly, muzzle and socks). The
  simplest place to start a new animal: copy the folder, change the boxes, colors and poses.
- [mods/animals/source/raptor](../mods/animals/source/raptor/README.md): a feathered Velociraptor,
  the first two-legged animal (bird legs in a crouch, eyes on the sides of the head, feather
  panels for the crest, wings and tail fan). Start birds and other dinosaurs from this one.
- [mods/animals/source/capybara](../mods/animals/source/capybara/README.md): a stocky four-legged
  grazer with no tail, eyes on the sides, a chewing jaw and grazing fitted to the ground. Start
  heavy herbivores (pigs, sheep, deer, bears) from this one.

To update one in the game after changing it:

```
cd mods/animals/source/dog/chihuahua      # or cat, raptor, capybara, utahraptor
python3 -m venv .venv && .venv/bin/pip install -r requirements.txt   # once (the others use ../dog/chihuahua/.venv)
.venv/bin/python build.py
```

To see them side by side, publish the skill's `viewer.html` with an `animals.json` listing each
creature's GLB (from `bb_build.py`) and its texture options (from `variants.py`).
