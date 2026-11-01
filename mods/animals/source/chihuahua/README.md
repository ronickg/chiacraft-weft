# The chihuahua's source

Everything needed to rebuild the dog: a papillon-style long-haired chihuahua made the way Hytale
makes creatures, a few dozen boxes and flat fur panels with a hand-painted pixel texture. The
recipe for making another animal like it is [docs/creatures.md](../../../../docs/creatures.md); the
rules and scripts are the `blocky-model` skill in
[.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md).

## Rebuild

```
python3 -m venv .venv && .venv/bin/pip install -r requirements.txt   # once
.venv/bin/python build.py
```

`build.py` paints the model and its animations (`tools/blocky.py`, into `out/d_blocky/`, not
committed), checks no faces flicker, at rest or in any keyframe (the skill's `boxcheck.py`) and exports the game's files into
`../../assets` with the skill's `to_bedrock.py` (scale 0.17, a puppy at half size with a 1.3x
head). Commit what it writes there; F3+T reloads them in a running game.

To open or edit it in Blockbench (and get Hytale `.blockymodel`/`.blockyanim`, a `.glb` and a
`.bbmodel`), start Blockbench with a debug port and build it there:

```
/Applications/Blockbench.app/Contents/MacOS/Blockbench --remote-debugging-port=9333 &
.venv/bin/python tools/bb.py 'let p = Plugins.all.find(p => p.id == "hytale_plugin"); await p.download(true); return p.installed'   # once
.venv/bin/python tools/bb_build.py          # into out/d_blocky/
.venv/bin/python tools/bbshot.py shots.png  # four views of it
```

## Files

- `ref/turnaround_flat.png`: the flat turnaround the dog is built from (six orthographic views,
  flat colors); `ref/views/f_*.png` are its front, back, sides and top cut out (the guide lines
  painted out), which is all the build reads. `ref/blockout_fur.png`: the same views with the fur
  removed, used for measuring parts.
- `tools/blocky.py`: the model. Element layout (`elements()`: body, head, upper muzzle, jaw,
  mouth, tongue, solid ears, eyelids, bib, fringes, tail, legs, paws), painting (`smooth_colors`,
  designed eyes `EYES`, paws `stamp_paw`, mouth `paint_mouth`), the skeleton with jaw, tongue and
  eyelid bones, and the animations (`ANIMS`, `mouth()`, `eyelids()`, `BLINKS`, `lick_keys`).
  Landmarks and the fitted sit/lie values are in `L`.
- `tools/voxdog.py`: the base skeleton, the palette and the poses (idle, walk, run, sit, lie;
  `tail_wag` swings the tail about the axis across each segment).
- `tools/ground.py`: each part's lowest point in a pose, to fit sit and lie to the ground.
- Flicker is checked with the skill's `boxcheck.py` (rest pose and every keyframe), which `build.py` runs.
- `tools/bb.py`, `bb_build.py`, `bbshot.py`: drive Blockbench (build, export, screenshots).

## How it's made (and what each fix was)

- **Texture**: the sheet only decides which color area each texel is in (one brown coat whose
  lights turn golden, white, pink, nose); each area is painted with continuous light in fine
  hue-shifted steps, soft tufted edges, ambient occlusion from the boxes and gentle fur streaks.
- **Eyes**: `between` (default), a blend of the Hytale corgi's eye and a cute sparkle; the others
  in `EYES` (`BLOCKY_EYES=<name>`) are kept to compare. Eyelids (`eyelid_L`, `eyelid_R`) lie
  inside the head and swing down over the eyes to blink (-90 on X).
- **Ears**: 1.35x the sheet's, a stack of thin boxes plus one cut-out fringe panel (two panels
  back to back left a gap), lit as one piece.
- **Mouth**: the jaw opens (hinged at the back), shorter than the upper muzzle so the tongue
  bends out of sight under the lip; the tongue is three parts hinged on their top surface,
  narrowing to the tip.
- **Bib**: locks whose tips meet in a V between the front legs.
- **Paws**: three toes and pads.
- **Poses**: sit and lie fitted with forward kinematics so every paw rests on the ground; the
  tail stays upright when sitting (it cancels the body's tilt).

Earlier versions and the other approaches tried (voxels, AI meshes) are in `~/Coding/voxel-dog`
on the Mac it was made on, with its full notes; nothing here depends on it.
