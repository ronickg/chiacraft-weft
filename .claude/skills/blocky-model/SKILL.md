---
name: blocky-model
description: Build, texture, rig and animate block-game style 3D creatures and props (Minecraft, Hytale, Vintage Story look) as boxes and quads with a hand-painted pixel texture, in Blockbench, from concept art or a description. Use when asked to make a mob, animal, creature, character or prop "like Minecraft/Hytale", to turn concept art into a Blockbench / .bbmodel / .blockymodel / Bedrock geo model, to paint or fix a pixel texture for a box model, to animate one, or when a box model looks rough, flickers, has wrong eyes, floats or sinks in a pose.
---

# Block-game models: boxes + painted pixel texture

The look (Minecraft, Hytale, Vintage Story) is **few boxes, texture does the work**. Not voxels
(thousands of small cubes look grainy and can't carry painted light), not smooth meshes.

The aim is to get a creature right **the first time**: build in the defaults below from the
start, run the review checklist before showing anything, and let the user choose between options
instead of iterating by hand. Everything here is generic: a dog's ears, a bird's wings and a
fish's fins follow the same rules.

## Rules that decide the look

Geometry
- Only boxes (6 faces) and quads (one flat face, see-through pixels allowed). No spheres, wedges
  or triangles. Fewest boxes that give the silhouette; Hytale caps a model at 255 nodes; aim for
  20-60 parts on a creature.
- Keep texel density the same everywhere: UV size = face size × density (Hytale 64 px a block
  for characters/creatures, 32 for props; Minecraft 16). Stretch a box only 0.7-1.3× per axis.
  Texture sizes in multiples of 32 (Hytale) / powers of two (Minecraft).
- **No two faces in the same plane where both can be seen** (z-fighting flickers as the camera
  moves): offset touching boxes ≥ 0.5 unit; never two quads in one plane. `scripts/boxcheck.py`
  must report 0 after every change; new parts tend to land in a neighbor's plane.
- One part per bone, clear names (head, jaw, ear_L, tail_0, leg_FL_upper...). Rigid, no weights.

Texture (paint it like an illustration; each face is a small pixel painting)
- One light, from above and a bit front-left, baked in: tops lightest, sides mid, bottoms
  darkest; light fades down each side face; soft lighter top edge, darker sides of a box.
- **Smooth, not stepped**: continuous light drawn in fine steps (a quarter ramp step, ~25 shades
  per material). 3-5 hard bands and dark outline rims between colors look rough.
- **Hue-shifted ramps**: shadows hue-shifted and a bit more saturated, lights toward yellow and a
  bit less. Warm creatures keep warm shadows; purple shadows suit stone and metal. Never pure
  black or white.
- **Big simple color areas**, one coat color whose lights turn golden, not scattered patches
  (reads as camo); drop islands smaller than a few texels.
- **Soft tufted edges** between colors (the upper color hangs into the lower in short points,
  then ~1 texel of blend). **Ambient occlusion** computed from the boxes where parts meet and
  near the ground. **Fur as gentle streaks** along the fur, not dashes, not noise.
- Pad each face's atlas island with a 1-texel gutter copied from its edge; nearest filtering.

## Creature defaults: build these in from the start

Face
- **Eyes are designed, not projected**: a pixel design stamped on the front of the head only
  (never let side or back views paint eyes or a nose onto the sides or top). Hytale's own
  creatures have big eyes (~8 texels wide on a 22-wide face): a dark ring around a near-black
  pupil, a warmer glow along the bottom, a cool white 2×2 highlight and a small grey-blue
  sparkle, set in darker fur all around with lighter fur below (`paint.EYES["hytale"]`, halo
  "round"). For a cute creature, `hytale_cute` (the same at 6 wide, between Hytale's and a cartoon
  sparkle) was the pick. Darkening only above the eye reads as a frown. Offer a mix of Hytale-style and
  simpler cute types with the switcher (Workflow 6) and let the user pick. Eye *types* are not
  expressions (closed, sleepy, happy); add expressions only when asked.
- **Mouth that can open**: muzzle/beak split into an upper part on the head and a lower jaw on
  its own bone hinged at the back (+X opens). Make the lower jaw **shorter than the upper part**
  (a slight overbite) so anything coming out of the mouth bends out of sight under the upper lip.
  A dark mouth box inside shows when open. Paint lips/beak edge, a line from the nose to the
  lip, small fangs where they fit, a pink roof and floor.
- **Tongue** (if any): a base on the jaw inside the mouth, then a chain of 2-3 bones, each hinged
  **on its top surface** at the jaw's front edge and at each joint, so it drapes and curves;
  narrow it toward the tip (e.g. 5-4-3 texels). No gap between base and first hinge, and paint
  the face under a part that bends over an edge in that part's color (or a dark band shows).
  Leave moving parts out of baked occlusion.

Appendages
- **Thin parts that must look solid** (big ears, wings, fins, frills, leaves): a core of thin
  boxes (1-2 units) stacked to the outline (shaved of fringe; mirror one side so left and right
  match), lit as one piece (no per-box edge highlights or it reads striped), plus one double-sided
  cut-out quad through the middle for the fringe. Two one-sided quads back to back show a gap.
- Same coat color as the body part they grow from (a different hue looks pasted on).
- **Cute proportions**: ears, eyes and heads read better slightly bigger than a realistic sheet;
  scale a part about its base and paint it from the sheet at the original size (map each texel
  back by 1/scale before sampling) so colors and outline still line up.
- No one-way cut-out quads where they're seen side-on (a face-on cheek-fluff quad reads as an
  extra ear from the side): put such fringe on parts that have width.

Feet and fur
- **Feet are designed**: toes as grooves on the front running a little over the top, pads
  underneath (hooves: a split line; birds: toes as thin boxes). Size the foot so toes split
  evenly (8 texels = three 2-texel toes and two grooves). Force an area's color when the sheet
  can't see it (foot tops hide under the body in a top view).
- **Fringes whose shape matters** (a chest bib ending in a point, a mane, a beard, tail feathers)
  get a designed outline: locks a few texels wide whose tips meet in a V or fan. End the solid
  box behind above the tips, or its flat edge is what shows. White over white hides an outline:
  darken the very tips a little. Tall light quads need a gentle light falloff and soft streaks,
  or their lower half fills with grey that looks like dirt.

Animation
- Rotations relative to the parent, one Euler order (Blockbench and Hytale: R = Rz·Ry·Rx);
  combine as quaternions (`rig.quat`, `rig.euler`, round-trips).
- **Swing about the axis across the part** (`rig.swing`): tails, necks, antennae, trunks. A part
  pointing up that swings about the world's vertical only twists in place.
- **Fit resting poses to the ground** (sit, lie, crouch, perch) with forward kinematics
  (`ground.lowest`) and a grid search (`rig.fit`), never by eye: feet at 0, nothing below 0, the
  body resting where it should. Count fringes separately (they may brush the ground). Numbers
  tuned for one model never carry over to another.
- **Feet flat**: a foot's pitch cancels the sum of its parents' pitches. Likewise any part that
  should keep its own direction when a pose tilts the body (an upright tail when sitting, a
  level head) cancels the parent's tilt; a fixed angle added on top of the tilt folds it into
  the body. Check each resting pose from the side for parts sunk into the body.
- Idle breathes and moves something alive (pant, an ear twitch, the tail); keep the mouth open
  for panting creatures, nearly shut when resting.
- **Blinking**: an eyelid quad per eye on its own bone, hinged at the eye's top, the same size
  as the eye, painted as the closed eye looks (the face's fur from the front view, a dark lash
  line along the bottom, a soft crease above). In the rest pose it lies flat *inside* the head;
  a blink swings it down over the eye (-90° on X) and back in ~0.18 s. Rotation only, so every
  format plays it, and a missing keyframe leaves the eyes open. Add keyframes at the blink's
  edges and middle (even steps are too coarse); blink in idle, sit and lie, not walk or run.
  If eye designs are swapped as texture variants, keep the lid size fixed (one eye type's) so
  every variant shares the UV layout.
- **Licking** (or any quick tongue move): animate the jaw/tongue chain from (phase, value)
  keys with fine sampling (~0.05 s): jaw opens, tongue out level, tip curls up over the nose
  (-X), sweeps side to side (Y), folds back under the upper jaw (tip 180°) to hide; squint the
  eyes (lids closed) at the peak. Hide parts by folding them inside other boxes, not by scale.
- **Animation set** for a creature: idle, walk, run, sit, lie as a base; then cheap, charming
  extras that reuse the same bones: blink, lick, head tilt, sniff, shake off, scratch, play bow,
  sleep, eat. Offer the list and let the user pick.

## Workflow

1. **References.** Best: a flat orthographic turnaround (front, back, left, right, top, bottom),
   same scale, flat unlit colors, plain grey background; plus a blockout (fur removed, parts by
   role). If the user has only concept art, give them this prompt for an image model:
   > Technical model sheet for 3D reconstruction of the attached character. Same character and
   > colors. Six views in a 3x2 grid, each large: FRONT, LEFT, RIGHT, BACK, TOP, BOTTOM. True
   > orthographic, no perspective, identical pose and scale, feet on one ground line, thin black
   > horizontal guide lines at ground, back, head top and ear tips. Flat unlit color-ID coloring:
   > no shading, highlights, outlines or shadows; only these colors: <palette>. Plain #808080
   > background, no text.

   Crop views (background by color distance + flood fill; paint out guide lines first).
2. **Lay out boxes** in code: elements `{name, bone, lo, hi, quad?, cut?, rot?, origin?, src?}`
   in units (y up, facing +z), including the defaults above (jaw, mouth box, solid appendage
   cores, designed fringes, feet). `boxcheck.py` → 0.
3. **Areas, not colors, from the sheet**: per texel, the view facing that face, snapped to the
   materials allowed on that part, smoothed (`majority`). Cut quads from silhouettes or designed
   outlines.
4. **Paint**: `paint.paint_face_smooth` (tones from `paint.tones`, `paint.occlusion`), then
   designed features (`paint.stamp_eyes`, `paint.stamp` for toes, pads, mouth), `paint.pack`.
5. **Build in Blockbench** (scriptable, exports every format):
   - `brew install --cask blockbench`; run
     `/Applications/Blockbench.app/Contents/MacOS/Blockbench --remote-debugging-port=9333 &`.
   - `scripts/bb.py 'return ...JS...'` runs JS inside Blockbench (`newProject(Formats.x)`,
     `new Group({name, origin}).init().addTo(parent)`, `new Cube({name, from, to, origin, rotation,
     box_uv:false, faces:{north:{uv,texture}...}}).init()`, `new Texture({name}).fromDataURL(...).add(false)`,
     `new Animation({name, length, loop:"loop"}).add()`,
     `anim.getBoneAnimator(group).addKeyframe({channel:"rotation", time, data_points:[{x,y,z}]})`).
   - Hytale: install the store plugin once
     (`let p = Plugins.all.find(p => p.id=="hytale_plugin"); await p.download(true)`) → formats
     `hytale_character` (64 px/block), `hytale_prop` (32), codec `blockymodel`. After
     `newProject` a settings dialog opens: `Dialog.open.cancel()`.
   - Export: `Codecs.blockymodel.compile()`, `Codecs.project.compile()` (.bbmodel),
     `await Codecs.gltf.compile({animations: Animation.all, encoding:"binary"})` (ArrayBuffer;
     base64 in chunks), `Codecs.bedrock.compile()`. Hytale animations: replace
     `Blockbench.export` with a capture of `opts.content`, call `anim.save()` per animation.
   - The glTF export **pads the texture** (e.g. 512×96 → 512×128); anything that swaps textures
     on the GLB must use the padded size (`variants.py` does).
6. **Show it, and let the user pick** instead of fine-tuning by hand: publish
   `scripts/viewer.html` (as the page, with `model.json` = `{"glb": base64}` and optional
   `variants.json` from `scripts/variants.py`). It has a button per animation and per texture
   variant. For anything subjective (eye type, coat pattern, markings), paint 5-10 variants that
   differ in kind, not just tweaks, on the same UV layout and let the user choose.
   When they like two, offer 2-3 blends between them (size of one, colors and ring of the
   other) next to both originals, and make the pick the model's default.
7. **Review checklist** before showing (`scripts/bbshot.py` for four angles; animation frames
   via `Modes.options.animate.select(); anim.select(); Timeline.setTime(t); Animator.preview();`):
   - [ ] `boxcheck` 0; move the camera: nothing flickers
   - [ ] two eyes on the front only; no stray eye/nose texels on sides or top; face reads cute
   - [ ] no part reads wrong side-on (stray flaps); thin parts solid, no gaps between layers
   - [ ] colors: one coat hue across head, ears, body; no grey/dirty shading on light areas;
         no colors bleeding onto the wrong part
   - [ ] mouth: opens, no dark gaps at hinges, tongue bend hidden, tongue narrows
   - [ ] feet: toes and pads; flat on the ground in every resting pose; nothing below ground;
         no part (tail, ears) folded into the body in any pose
   - [ ] swinging parts swing, not twist
   - [ ] compare with a screenshot of the target game's own creature at the same angle
8. **Save a version before each big change** (`<out>_vN/` with the scripts that built it and a
   README on restoring it): users often want to compare or revert.

## Scripts

- `scripts/bb.py` — run JS in a running Blockbench over CDP (`BB_PORT`, default 9333).
- `scripts/bbshot.py out.png` — the open Blockbench project from four angles, side by side.
- `scripts/boxcheck.py spec.json` — coplanar overlapping faces (z-fighting); exit 1 if any.
- `scripts/paint.py` — `tones`, `occlusion`, `paint_face_smooth` (default look), `EYES` (ten eye
  types: six Hytale-style, four cute) and `stamp_eyes`, `stamp`, `majority`, `pack`;
  `ramp`/`paint_face` (flat five-step look).
- `scripts/rig.py` — `quat`, `qmul`, `qmat`, `euler`, `swing` (wag about the axis across a
  part), `fit` (grid search for pose parameters).
- `scripts/ground.py` — `lowest(spec, pose, groups)`: each group's lowest world y in a pose.
- `scripts/variants.py model.glb out.json name=tex.png ...` — texture variants padded to the
  GLB's texture size, as data URLs for the viewer.
- `scripts/viewer.html` — the viewer page (animations + variant switcher, three.js r147).
- `scripts/to_bedrock.py spec.json texture.png <assets> --name <animal> --scale <s> --texture-name <mod>/<animal> [--pup 0.5]`
  — the game files (Bedrock `.geo.json`, per-pose `.animation.json`, texture) for Weft/chiacraft
  or Minecraft Bedrock: turns the model to face -Z, gives tilted boxes their own bones, bakes the
  scale, paints both sides of flat panels, one animation file per pose (`idle` holds it).

## Into chiacraft (Weft)

Follow `docs/creatures.md` in the chiacraft repository: export with `to_bedrock.py`, register the
entity (`.model_file`, `.animations` per pose, `.texture`, `.size`), and check it in the game
with a `--script` that spawns it and takes shots (`look yaw pitch`: negative pitch looks down).
Engine limits that shape the model: bones rotate but single boxes don't; no per-kind scale; only
`idle` and `walk` play by themselves (poses are look kinds with their own file); a played
animation replaces idle while it plays; no Molang; cut-out alpha only (no blending).

Worked example: the chiacraft chihuahua (a papillon), source in the repo at
`mods/chihuahua/source` (its README explains it): `tools/blocky.py` builds 51 boxes/quads on 27
bones with all the defaults above (eyelids, jaw, tongue, solid ears, toes, a V bib),
`tools/bb_build.py` builds it in Blockbench, `tools/ground.py` and the fitted `sit_*`/`lie_*`
values, `build.py` exports it into the mod. Copy that folder to start a new animal.

## Sources

- Hytale art director's modeling guide: https://hytale.com/news/2025/12/an-introduction-to-making-models-for-hytale
  (cubes and quads only, 0.7-1.3× stretch, 64/32 px density, multiples of 32, paint shadows/AO/
  highlights in, no noise, no flat surfaces, no pure black/white, colored shadows).
- Blockbench Hytale plugin notes: https://cdn.jsdelivr.net/gh/JannisX11/blockbench-plugins@master/plugins/hytale_plugin/about.md
  (255 nodes, UVs locked to face size).
- Hue shifting for block-game skins: https://www.planetminecraft.com/blog/skinning-understanding-shading-hue-shifting-and-colors-5562100/
