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
- **Size**: the engine has no per-kind scale; `--scale` is baked into every position (model
  pixels per Blockbench unit; 0.17 makes the chihuahua the size of the old one, ~0.6 blocks).
- **Fur panels**: both sides of a flat panel get the picture (mirrored to line up).
- **Poses**: the engine plays `idle` and `walk` by itself. A pose (sit, lie) is its own file whose
  `idle` holds it, used by its own look kind (`set_look`). `run` and actions like `lick` are in
  the standing file for `World::play_animation`, which replaces idle while it plays (so an action
  on a sitting dog stands it up for that time). Blinks are part of each idle.

Register it like the chihuahua (`mods/chihuahua/src/lib.rs`): `.model_file("<animal>")`,
`.animations("<animal>[_sit|_lie]")`, `.texture("<mod>/<animal>")`, and a hitbox `.size(..)`
about the model's footprint. Bones named `head` follow where the creature looks; don't name
bones `leg0`..`leg3` unless you want the engine's own leg swing (an animation with `walk` turns
it off anyway).

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

## The chihuahua, as the worked example

Its source is in the mod: [mods/chihuahua/source](../mods/chihuahua/source/README.md).
`tools/blocky.py` lays out and paints the dog (51 boxes and quads with eyelids, jaw and tongue;
`EYES`, `BLINKS`, `lick_keys`), `tools/voxdog.py` has the poses, `tools/bb_build.py` builds it in
Blockbench. To update the dog in the game after changing it:

```
cd mods/chihuahua/source
python3 -m venv .venv && .venv/bin/pip install -r requirements.txt   # once
.venv/bin/python build.py
```

For a new animal, copy that folder next to the new mod's assets and change `blocky.py`'s
elements, painting and poses; `build.py` names the files.
