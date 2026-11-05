---
name: blocky-model
description: Build, texture, rig and animate every model in the game (creatures, props, eggs, nests, items) in one block-game house style (Minecraft, Hytale, Vintage Story look) as boxes and quads with a hand-painted pixel texture, from concept art, research or a description, so they all match. Use when asked to make any model, a mob, animal, creature, character or prop "like Minecraft/Hytale", to turn concept art into a Blockbench / .bbmodel / .blockymodel / Bedrock geo model, to paint or fix a pixel texture for a box model, to animate one, or when a box model looks rough or somehow wrong next to its references, flickers, has wrong eyes, floats or sinks in a pose.
---

# Block-game models: boxes + painted pixel texture

The look (Minecraft, Hytale, Vintage Story) is **few boxes, texture does the work**. Not voxels
(thousands of small cubes look grainy and can't carry painted light), not smooth meshes.

The aim is to get a model right **the first time**: build in the defaults below from the
start, run the review checklist before showing anything, and let the user choose between options
instead of iterating by hand. Everything here is generic: a dog's ears, a bird's wings and a
fish's fins follow the same rules.

In chiacraft this skill is how **every** model is made, creature or prop, in one house style
(next section): they must sit together seamlessly, a nest next to the raptor that laid it next to
a chihuahua.

## House style (chiacraft): every model matches

The numbers are in `scripts/house.py`; build scripts import them instead of choosing their own.

- **One texel density: 32 texels a block** (a block is a meter: a texel is 3.1 cm), the same as
  the blocks and the items (Ronald, 2026-11-03: "Yes, rebuild at 32"), for every model:
  creatures, props, eggs, nests. Lay a model out in **layout units of 1/64 block** (`house.UNITS`:
  half a texel, so boxes can sit half a texel apart and poses keep their numbers), export with
  `house.SCALE` (0.25 model pixels a unit), and paint `house.TX` (0.5) texels a unit: a face's
  texels are `round(TX * size)`, at least one. A model on its own scale paints
  `house.texels(scale)` a unit. Terrain blocks are 32 × 32 (decided 2026-11-03, from the style guides in `game-art`); inventory icons come from the `game-art` skill (generated at 32 × 32), and
  items are held as those flat pictures (box models from them are set aside: "Items from icons").
- **Small creatures paint finer** (Ronald: "where 32 would lose them, a vole or a sparrow"): when
  the face that carries the eyes is under `house.MIN_FACE` (3) texels at 32, paint the least
  power-of-two multiple of 32 that gives it 3 (`house.fine(face_units)`; `texels(scale, k)`).
  Today: raven, gull, vulture and fish at 64; mouse, vole, kryptobaatar, small bird and lizard at
  128; the owl at 64 too (its face is its two big eyes, 4 by 4 texels at 32). Everything else at
  32, the chihuahua and cat included: Ronald, 2026-11-04, shown both at 32 and finer, "32, one
  look". A new animal follows the same rule. Boxes, export scale and in-game size stay the same.
- **Keep 32 for small animals** (Ronald, 2026-11-07, after the chihuahua, red fox and brown hare
  side by side at 32 and 64: "Keep 32"): every animal stays at 32 texels a block, small ones too.
  A small face (a chihuahua's 5 by 4) is not a reason to paint finer: design its eyes and marks
  for the texels it has. `fine` is only for a face that can't hold an eye at all (under
  `MIN_FACE` at 32: the birds, fish and rodents above). Don't offer a 64 version again.
- **Faces at 32 are small** (a wolf's 7 by 6 texels, a cat's 6 by 5, a chihuahua's 5 by 4): eyes
  are 2-3 texel designs with no halo (`paint.EYES` `px2`, `px3`..., or the animal's own; two front
  eyes placed with `paint.eye_corner` so they land mirror-symmetric), a mark is a texel or more
  (stripes 3+ texels apart), and toe grooves, whiskers and lip lines go where the face has no
  room for them. The painter's filters that merge thin areas (3x3 votes, blurs over a texel) erase
  one-texel marks: keep them off where a texel is the design. `scripts/render.py` draws a model
  without Blockbench for before/after sheets.
- **Real sizes**: research the real thing's measurements and build the average adult (a 20 cm
  chihuahua is 13 units at the shoulder, a 1.5 m Utahraptor 96 at the hip). Individuals differ
  by the engine's per-entity scale (sex, a size gene, growing up), never by another model.
- **Light**: only `paint.py`'s (one light from above and a bit front-left, hue-shifted ramps in
  quarter steps, ambient occlusion from the boxes, light lower near the ground). Never paint
  light, outlines or shadows by hand; designed features (eyes, toes, blotches) are areas that the
  same light then shades.
- **Colors**: an animal's or monster's look (coat colors, markings, face) is agreed first as a concept
  sprite with the `game-art` skill (`guides/4 Animals`, Part 4 says what carries over to a box
  model); then `house.MATERIALS`, natural and a little muted, the shadow hue by substance (warm for
  anything alive or earthen, purple for stone and metal, cool for eggshell). No pure black or
  white. An animal's coat comes from its genes on the client (see "Coats on the fly"); paint the
  model in its species' most common coat.
- **Shapes**: few chunky boxes (Hytale's way); cute exaggeration only on a creature's head, eyes
  and ears; props keep real proportions.
- **Props are entities** (`scripts/prop.py` + `to_bedrock.py`), placed by an item and broken back
  into it, not blocks with their own model: block models here take only coarse texture
  coordinates (a 128th of the texture), and a painted sheet on one came out a flat color.
- **Built from source**: every model is `mods/<mod>/source/<thing>/build.py`, which writes its
  assets and asserts the checks (`boxcheck`, `ground.joints`, `ground.pokes`, `ground.gait`).
  Never edit an exported asset by hand.

## Rules that decide the look

Geometry
- Only boxes (6 faces) and quads (one flat face, see-through pixels allowed). No spheres, wedges
  or triangles. Fewest boxes that give the silhouette; Hytale caps a model at 255 nodes; aim for
  20-60 parts on a creature.
- Keep texel density the same everywhere: UV size = face size × density (chiacraft: 32 a block
  for every model, see House style; Hytale 64 for characters and 32 for props; Minecraft 16). Stretch a box only 0.7-1.3× per axis.
  Texture sizes in multiples of 32 (Hytale) / powers of two (Minecraft).
- **No two faces in the same plane where both can be seen** (z-fighting flickers as the camera
  moves): offset touching boxes ≥ 0.5 unit; never two quads in one plane. `scripts/boxcheck.py`
  must report 0 after every change, **at rest and in every keyframe** (it checks both; build
  scripts run it and stop on a failure). Most flicker only appears in a pose, so design against it:
  - **Joined parts of a chain differ in size**: a lower leg 1 texel thinner than the upper leg
    (and a paw wider), each tail segment 0.4 a side thinner than the one before. Same-width
    neighbors share their side planes whenever the joint folds or straightens.
  - **Neighbors across a bending joint** (chest and hips, front and back fur panels) differ by
    ≥ 0.5 in width and height, so breathing or a bend never lines their faces up.
  - **No exact right angles in resting poses** (hip -90, a fold of exactly 180): they turn one
    part's face into another part's plane. Use -86, 160...
  - **Flat panels hanging beside legs** (belly fur, a bib) sit ≥ 0.5 from every leg and paw side
    plane: legs swing about X, so their side faces stay in the same planes as the panels.
  - When the check reports a pair, read which faces and which animation, then move the smaller
    part, change one size, or nudge the pose angle; run it again (fixes move collisions around).
- One part per bone, clear names (head, jaw, ear_L, tail_0, leg_FL_upper...). Rigid, no weights.
- **Every joint inside both parts it joins**: a bone's pivot must lie inside one of its own boxes
  and one of its parent's. Then the two turn about a point they share and can't come apart in
  any pose; a pivot outside either (a head hinged above its neck, a shoulder outside the chest, a
  knee below the thigh) opens a gap the moment the joint turns. Lay a neck, tail or limb out as
  a box from inside the parent to the child's joint (`base + length * direction`), and put the
  child's pivot at that end, inside the child. `ground.joints(spec)` lists every loose joint;
  build scripts assert none. (Hinges on a surface, like a tongue folding over a lip, still need
  the pivot within a quarter unit of both parts.)
- **Nothing pokes through along the body's axis**: a part that meets its neighbor at an angle (a
  neck rising into a level head, a neck into the chest, a tail segment into the one before) has
  its end hidden inside the neighbor in every pose. A corner left just outside (a sliver up to
  1.5 units) cuts slanted edges across the neighbor's faces, a lump that looks broken (both
  raptors' necks did, through the back of the head). Make the entering part slimmer than the one
  it enters and let it reach deep (a neck's end near the middle of the skull, not just past its
  back face), so the head can turn and tilt. `ground.pokes(spec, ["neck", "skull", "chest",
  "hips", "tail_"])` checks it at rest and in every keyframe; build scripts assert it's empty.
  Without the list it also shows limb joints (knee, ankle, toes) and an opening jaw: there a
  small step at the bend is the usual box joint and reads as one.

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
- **Faces only seen in poses** (the undersides of legs, tails, folded parts) are lit like back
  faces, not as near-black bottoms: they show when a pose folds them.
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
- **Dark spots on a face read as eyes**: nostrils, a mouth and pads drawn as dark squares
  turn a snout's front into a second face. Draw nostrils as thin slits (one row), a mouth as a
  thin line, and keep the real eyes the darkest, roundest marks with a catchlight.
- No one-way cut-out quads where they're seen side-on (a face-on cheek-fluff quad reads as an
  extra ear from the side): put such fringe on parts that have width.

Big bodies
- **A torso reads as muscle, not one box**: a core, then the masses that carry the animal on
  top of it (pectorals either side of the breastbone, shoulder or wing-root muscles widening the
  top, the ribcage narrowing back to the hips, a keel under a flier's chest). Still few boxes, each
  a few units out from its neighbors' planes. One big box for a 4 m chest looks like a crate.
- **Stance width decides strength**: an animal standing on its forelimbs (a dragon on its
  wrists, a gorilla on its knuckles) plants them wider than its shoulders, the limbs sloping out
  from the elbows (an A from the front). Straight down from the shoulders they read as stilts.

Wings (bats, dragons, pterosaurs: arms that are wings)
- **Proportions from the animal, not from a guess**: a bat's or dragon's wing is a short, thick
  upper arm, a long forearm, and fingers each about as long as the forearm or longer, the skin
  reaching back to the hips; a dragon's span is 1.4-1.5 times its nose-to-tail length. The
  forearm must be long enough to reach the ground from an elbow at the height of the back, or it
  can't walk on its wrists with the wings folded; short fingers can't fold into anything but a
  cape. Sweep the arms a little forward in flight: the leading edge angles forward.
- **Skin as panels**: one cut-out quad per gap between fingers, riding the first of the two, and
  one from the last finger back to the body; each in its own plane inside the fingers' thickness,
  a unit apart (they never share a plane, even mid-beat).
- **A rigid panel can't follow fingers closing**: turning it about the wrist moves its whole
  edge off its own finger (by the angle it turned: on long fingers, metres). So close the fan
  only partly (half the spread), turn each panel by half the closing so it sits centred in the
  gap left (reaching a little past both fingers, over its neighbors, a unit apart), and draw in
  (scale to a sliver toward a pivot inside the hand or body) the skin that can't fit: the panel
  from the last finger to the body.
- **Folded on the ground, a sail**: the wrist planted wide and ahead of the chest, the forearm
  rising to the elbow by the chest's side, the fingers half closed and fanned up and back from
  the wrist into a tall pleated sail beside the body (the show's dragons). Fit it (wrist, elbow,
  first and last finger tips as targets) with every corner of the arm, hand and fingers above the
  ground: pinning one point (the thumb claw's tip) lets the rest hang through it.

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
  (`ground.lowest`) and a grid search (`rig.fit`), never by eye. **Search a wide enough range**
  (a body tilt of 15-55°, not just the steep end) and print the fitted lowest points: if the
  front feet float, the range or the weights are wrong (weight the feet more). For a sitting
  quadruped the tilt follows from leg length: with the rump on the ground the shoulders must sit at
  front-leg height, so short front legs mean a lower tilt. feet at 0, nothing below 0, the
  body resting where it should. Count fringes separately (they may brush the ground). Numbers
  tuned for one model never carry over to another.
- **Tails, trunks and other chains in resting poses**: don't hand-tune them (each joint's
  turn compounds the rest pose's curve); set target points for each segment's end (on the
  ground, curling along one side toward the front paws for a sitting or loafing cat) and solve
  the joint angles with forward kinematics and an optimizer (scipy `minimize`, Powell), with a
  penalty for going below the ground (`fit_tail` in the cat).
- **Two-legged creatures** (birds, dinosaurs): model the leg straight (thigh, shin, foot, toes,
  each thinner than the last) and give every pose a crouched stance on top (knee forward, ankle
  back, toes flat: the angles sum to 0), with the drop to the ground from forward kinematics. Hold
  the head above the back with a raised neck and a counter-rotated head (a fixed offset added in
  every pose). A bird's head moves in holds and snaps (keys that hold, then jump), not smooth
  waves; walking, it bobs forward and back.
- **Eyes on the sides** (birds, dinosaurs, horses, rabbits): stamp them on the head's side faces;
  their eyelids hinge on the side and swing down about Z (+90 on the +x side, -90 on the other),
  lying inside the head when open.
- **Gaits move forward**: a foot in the air swings forward, a foot on the ground pushes back.
  With the swing's sign flipped against the lift, the creature moonwalks (easy to get wrong:
  +X pitches the foot back). Check with `ground.gait(spec, "walk", feet)`: positive for every
  foot, in walk and run; build scripts assert it. Short legs need a clear lift (a big knee
  bend, a small swing) and fine keyframes (20-24 a cycle), or the foot only shuffles and the
  check can't tell forward from back.
- **Feet that don't slide** (big creatures, where sliding shows): give each foot a path (on the
  ground moving back at the ground speed, then lifted and swung forward) and solve the limb's
  angles to it every frame (least squares on the model's bones, warm-started from the last frame,
  with terms keeping heels, toe tips and wrists above the ground); the wyvern's `solve_gait`.
  Solve the right limbs as left ones in the mirrored body (its turns' y and z, and its sideways
  shift, negated), so the shoulders' roll and the weight shifting over a planted limb are in the
  solve and nothing slides.
  Weft plays a walk 0.6 clip seconds per block walked, so a planted foot moves back UNITS / 0.6
  units per clip second, and a gait's clip length is its stride over that. A `run` takes over from
  the walk by speed; both are exported as offsets from the idle (`to_bedrock.LAYERED`), which the
  engine keeps playing under them.
- **Grazing, drinking, sniffing the ground**: fit the neck and head angles that bring the mouth
  to the ground (about half a unit up) with forward kinematics, like the resting poses; then
  blend into it inside the idle (look about, head down, chew, head up) so the engine plays it
  without code.
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
  every variant shares the UV layout. **Place the lid with the same rounding as the painted eye**
  (`stamp_eyes` rounds the eye to whole texels; compute the lid box from that, not from the eye's
  nominal center), or the closed lid sits half a row off and the eye peeks out under the lash.
- **Licking** (or any quick tongue move): animate the jaw/tongue chain from (phase, value)
  keys with fine sampling (~0.05 s): jaw opens, tongue out level, tip curls up over the nose
  (-X), sweeps side to side (Y), folds back under the upper jaw (tip 180°) to hide; squint the
  eyes (lids closed) at the peak. Hide parts by folding them inside other boxes, not by scale.
- **Skin that contracts** (a bat or dragon wing's inner membrane, between the arm and the body)
  is the one part that scales: folded, it is too big to tuck anywhere (hanging down it cuts
  through the thighs, folded in it pokes out under the belly), so in resting poses its bone
  scales to a sliver toward a pivot inside the body (`"scale": {bone: (x, y, z)}` in a frame;
  Bedrock and Blockbench play scale keys) and it opens to full size in flight.
- **Animation set** for a creature: idle, walk, run, sit, lie as a base; then cheap, charming
  extras that reuse the same bones: blink, lick, head tilt, sniff, shake off, scratch, play bow,
  sleep, eat. Offer the list and let the user pick.
- **Attacks** for anything that fights, one or more (the game plays one at random on each hit):
  short (0.5-1 s), a wind-up, the strike and a recovery from (phase, value) keys, with fine
  keyframes (24-30). A bite: drawn back, a lunge with the jaw opening on the way and snapping shut
  at the strike, the body leaning in. A swipe: the weight back, a forepaw raised nearly straight
  to the face with the paw curled, raked down and across. A kick (raptors): rearing back on one
  leg, the other drawn up and struck forward. A limb that strikes the ground (a dragon's claw):
  solve it to its targets like a gait's foot. Ears flat, a hiss, the head following the strike
  sell it. Run `boxcheck` on them too: a raised limb sweeps through the body's planes.

## Workflow

0. **Research the animal first** (web search and fetch), before laying out a single box, and
   write what you found into the creature's source README:
   - **The real animal**: shoulder height, head-and-body length, tail length, head size against
     the body (e.g. a house cat: 23-25 cm at the shoulder, 46 cm head and body, 30 cm tail, head
     about a quarter of the body). Turn them into units (pick the shoulder height in units, scale
     the rest), so the body reads as that animal before any styling.
   - **Its block-game versions**: Minecraft's model is public: fetch
     `https://raw.githubusercontent.com/Mojang/bedrock-samples/main/resource_pack/models/entity/<animal>.geo.json`
     for its boxes (what the fewest boxes that still read as the animal are). Hytale's creatures
     (screenshots, its blog, mods that show unused models): head size, eyes, which areas are white
     or recolorable. Vintage Story too when it has the animal.
   - **What makes it recognizable**: ear shape, muzzle, markings (a tabby's M and stripes), eye
     color and pupil shape, tail carriage, typical poses (a cat sits upright with its tail around
     its paws, lies as a loaf).
   - **For a fantasy creature, the best-known screen version is the real animal**: find its
     orthographic sheets (ArtStation's "render view" turnarounds), sculpts and stills, and
     measure from them as from a field guide: span against length, each limb segment against the
     others, how it stands, how wide. Look at images with the built-in browser (an image search;
     open a result's image address to see it large), and turn 3D models on Sketchfab in the
     page's viewer to their front, side and top (downloading one needs the user's OK and a look
     at its license). Write the ratios into the README; they decide bone lengths.
   - Then decide: real proportions for the body, cute exaggeration only on head, eyes and ears
     (Hytale's way), chunky limbs (not Minecraft's 2-texel sticks), and list the creature's
     signature animations.
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
   - [ ] `boxcheck` 0 at rest and in poses; look from below and from the side at every resting
         pose: nothing flickers
   - [ ] two eyes on the front only; no stray eye/nose texels on sides or top; face reads cute;
         a blink frame closes the eyes completely
   - [ ] no part reads wrong side-on (stray flaps); thin parts solid, no gaps between layers;
         `ground.joints` empty: nothing comes apart at a joint (look at neck, shoulders, knees, tail
         while the animations play); `ground.pokes` empty along the axis: no neck corner through
         the head or chest, no tail segment through the one before (look at the head from behind
         and above while it turns)
   - [ ] colors: one coat hue across head, ears, body; no grey/dirty shading on light areas;
         no colors bleeding onto the wrong part
   - [ ] mouth: opens, no dark gaps at hinges, tongue bend hidden, tongue narrows
   - [ ] feet: toes and pads; flat on the ground in every resting pose; nothing below ground;
         no part (tail, ears) folded into the body in any pose
   - [ ] swinging parts swing, not twist; walk and run go forward (`ground.gait` positive)
   - [ ] compare with a screenshot of the target game's own creature at the same angle
   - [ ] held against the references in flat views (`bbshot.py --angles front_flat,side_flat,top`,
         the same views as the reference sheet): the ratios match (span, limb lengths, stance
         width, chest depth), standing and in flight
   - [ ] quads (wing skin, fins) checked by their cut outline, not their box: a panel's bounding
         box can dip below the ground while its shape doesn't (and the reverse)
8. **Never throw away a generated image** (Ronald, 2026-11-04: "never throw away generated images
   pls so I can also check back"): every render, review shot, before/after sheet, variant and
   texture tried, the before versions too. Copy them to
   `/Users/ronald/Coding/chiacraft-archive/images/<work>/<date>/` (e.g. `models-32/2026-11-04/`:
   sheets, renders, variants, coats, textures before and now, with a README saying what is where)
   before a scratch folder can be cleared.
9. **Save a version before each big change** (`<out>_vN/` with the scripts that built it and a
   README on restoring it): users often want to compare or revert. Show the new one next to the
   old one in the viewer ("<name> (before)").
10. **When the user says something looks wrong** (and can't say what): look up the references
   again, shoot ours in the same flat views, and list the differences as numbers (a span 1.06 times
   the length against 1.4, a forearm too short to reach the ground). Fix the cause (bone lengths,
   part sizes) and refit the poses; adjusting a pose around a wrong skeleton never gets there.

## Coats on the fly (genes, breeding, paint)

A creature's look can vary per animal without repainting it: the painter records, per texel,
whether it is coat or fixed (eyes, nose, teeth, claws), its light level, how far toward the
underside it is, its position on the body, and where white spotting, points (tan points, a
colorpoint's dark), a face mask and patterns show (`layers.py`: `paint_face_smooth(out=...)`, a
`Recorder` per build, `layers/info|pos|props|props2.png` next to the texture). A gene file per
species (loci, alleles in dominance order with wild frequencies, X-linked and codominant loci,
quantitative traits) and `coats.js` (random animals, breeding with one allele from each parent,
phenotype rules per species from real coat genetics, a renderer with 3D patterns and the same
hue-shifted light ramps) paint any genotype on those layers; `coatdata.py` packs it for the
viewer's coat panel (Natural, Random, Mother, Father, Breed, Paint). When a creature gets its
layers: map each material to a role (`("coat", toward the underside 0..1, light shift)` or
"fixed"), mark everything stamped after painting that isn't a darkening as fixed (the recorder
does), and write its `layer_props` (where white starts: paws, chest, belly, muzzle; points;
mask; extremities; where patterns show). A painted pattern (a tabby's stripes) maps to the
plain coat: the genes draw patterns now. In the chiacraft game the build copies its layers to
`mods/animals/genetics/layers/<model>` and the mod's shared crate (`mods/animals/core`) paints
every animal's coat from its genome on the client. That crate also compiles to WASM
(`mods/animals/web/build.sh` writes `animals.wasm`); the viewer then runs the game's own rules
through `scripts/animals.js` (coat panel, How it works) and `scripts/lab.js` (the breeding lab:
matings, litters, eggs, growing up, families, old age if set, evolution; a balance table of
each species' real facts, the numbers the game derives from them and the balance checks). `coats.js` is the
standalone version, for a creature with no game behind it.

**Every animal unique: the core traits.** Real looks come in three layers: the kind (species,
breed, the population of a country) decides which patterns turn up; genes decide the pattern
(a few big ones) and its character (many small ones); chance in development decides where each
mark lands, so even clones differ. The core every species shares (`genetics/core.json`, merged
after a species' own loci by `Coats.withCore`) carries the small genes: size, shade, warmth, how
far white reaches, the marks' size, density, contrast, edge and stretch, eye color, build. Each
has a heritability: the rest of its spread is the animal's own chance from its seed. A gene file
sets their means (`traits`), what sex and youth shift (`sexes`, `young` with a pattern a young
one loses), its eye colors, its countries' kinds (`habitats`) and `mark_scale` (a big model
draws its marks bigger). So a species needs only its big genes and its colors; never give a
species' own locus an id the core uses (the cat's tabby `Mc` once hid the core's contrast). The
viewer's **Roll 12** shows a dozen of one kind side by side: check that no two look alike.
A creature whose `layer_props` leaves `pattern` at 0 shows no markings at all, whatever its
genes: mark where they show (the back and flanks fully, the belly and thin skin faintly). Eyes
vary too: while a recorder exists, `stamp_eyes` marks the iris (its saturated mid-light colors,
or the design's own `iris` keys) and the layers keep it (`props2` blue); the game paints it in
the animal's eye color, keeping its painted light. A design stamped by hand (outside
`paint.stamp_eyes`) calls `paint.mark_iris` itself. Builds vary too (a gene file's `build`: the
bones each build trait scales): name a model's bones so a trait can find them (one bone per neck
or tail segment, the jaw and eyelids under the head); in the viewer a bone whose rotated boxes
became meshes (`_tilt` in the game) can't be scaled. Scars (what life left: claw lines, tooth
marks, a blade's line, a burn, torn wing skin) are painted the same way, flat on the surface where
they fall; the viewer's **Scar** button adds one. Marks are drawn crisp, as pixel art draws
them: each pattern's cover (and points, mask, extremities, scars) snapped to three tones (none,
one edge tone, full), then lone texels unlike every neighbour taken into their neighbours'
(`crisp_marks` / `crispMarks`). Soft blends read as mud and stray pixels as noise at 32-64
texels a block; the viewer's **Crisp marks** switch shows the difference.

## Props (no skeleton)

Nests, eggs, furniture, anything placed or lying about: `scripts/prop.py`.

1. **Research** the real thing, as for an animal (its size, shape, materials, colors), and write it
   into the build's docstring or README.
2. **Lay out** elements in texel units centered on the origin at the ground (`prop.el`: boxes,
   quads with `cut`, tilts about one axis). Same rules: no coplanar faces (stagger even hidden
   bottoms), neighbors differ in size, fewest boxes.
3. **Areas**: `area(e, face, p, ...)` gives a material per texel from its position (`p`), e.g. a
   darker scrape in a nest's middle, blotches from fixed 3D spots so they run over edges.
4. **Paint** with `prop.paint(elements, area, [house.material(...)], coat=(soft materials,),
   cut=...)`: the creatures' light, occlusion and smooth painting.
5. **Export**: `prop.entity_spec` then `to_bedrock.export(..., house.SCALE, "<mod>/<thing>",
   pose_files={"": {}})`; run `boxcheck` on the spec. Paint a 16x16 inventory icon in the same
   colors.
6. **Register** an entity (`.model_file`, `.texture`, `.size`, `.passive().unpushable()`, no
   spawn egg), an item that places it, and drop the item when it is broken.

**Props get the whole workflow**, not just the painter: research with reference pictures, a
design, `boxcheck`, four-angle review shots after every change, and options for the user to pick
(geometry options as separate entries, colorings as texture variants). The first nest skipped
that and came out looking like a wooden crate with polka dots. What fixed it:

- **Earth, stone, wood** need their own detail, quietly: soil in soft clumps two to four texels
  across in shades just either side of the base, a lighter dry crust on top, a rare speck (a
  tiny stone). Never a regular math pattern (sin grids read as dots), never big contrasty
  blotches (camouflage), never a bare gradient (plastic). Run `smooth=1` (a `majority` pass) to
  drop single-texel noise.
- **Mounds and slopes**: three or four stacked boxes, each narrower and higher than the one
  below, and light them as one piece (`paint(down=...)`), or every step gets its own bright edge
  and it reads as stacked planks.
- **Lay materials as the real thing has them**: stems running round a dish (as an animal turns
  while lining it), an egg's marks gathered at its blunt end, darker marks over paler deeper
  ones, bark along a twig; things kicked out of a scrape land outside its rim.
- **Small props** (an egg is 10 units, 5 texels at 32 a block): taper both ends a unit a layer so it reads
  round, keep marks one or two texels (hairlines don't read at this size), and review it scaled
  up six times in a copy of the spec (Blockbench frames tiny models too small to see).
- **Show a prop as it is used** (a nest with eggs in it, half sunk and leaning as they are laid).

Worked examples: `mods/animals/source/nest` (`build.py --variants`: a bare, lined and grassy
scrape, each also with eggs) and `mods/animals/source/egg` (`build.py --variants`: four markings
on one layout).

## Items from icons (held models)

Set aside for now (2026-11-03): Ronald wants items flat, as in Minecraft. The tools' models,
made the way below for all 25 tools, are in `mods/base/source/items/tools.py` (one layout per
kind, each tier's finish), built and shot in Blockbench by `bb_items.py`; they write nothing into
the game until run, and the icons themselves come from the `game-art` skill.

Items (tools, weapons, food) are 3D box models drawn in the hand, made from their inventory icon
(the `game-art` skill: Ronald generates the icons, Claude cuts them). The model doesn't copy the
icon one to one, but must read as the same thing. Worked example:
`mods/base/source/items/stone_sword/` (`build.py`, `viewer.html`, `shots.py`).

What the guides say (Hytale's modeling guide, Blockbench's style guide, hand-painted weapon
critiques): weapons and tools at 64 texels a block, textures in multiples of 32; few boxes,
detail in the texture; a slant is a turned box, never stair steps; never stretch a texture;
light painted into the texture face by face; no pure black or white, colored shadows; deliberate
shades in a short ramp, not noise; a blade brightest toward its point; lit edges.

1. **The icon upright**: cut the item from its sheet (`sheet.background`; diagonal items reach
   into each other's columns, so take connected shapes), turn it so its axis is vertical (PIL
   `rotate(+45)` for one lying handle bottom-left), and print each row's width and extent: the
   parts (blade, guard, grip, pommel) show as steps in the width.
2. **Sizes**: widths from the icon, scaled so the parts keep their ratios. **Lengths not**: icons
   draw items short and stubby to fill a square (the first sword read as a dagger). Use real
   proportions: a one-handed sword ~1.35 blocks (87 texels), the blade 70% of it (Hytale's own
   are ~72 texels).
3. **Boxes**: one per part, neighbors differing in size; a flat blade 3 thick with a ridge box 5
   thick down its middle (the icon's light and dark halves are a bevel); a point from a square
   turned 45 degrees (side = width / sqrt 2), half sunk in the blade and 0.5 thinner so no faces
   share a plane. 7 boxes for the sword.
4. **Texture: "refined"** (`refined_faces`), the one to use. Per texel, in ramp steps (`tones`,
   0.25 a step), then snapped to the ramp:
   - each material's ramp from the icon: the part's lit pixels (55th percentile up) as its base;
     stone's shadow hue its own (0.07: toward purple a warm grey passes through red);
   - the icon's own light and marks: (luminance / base - 1) / 0.15, clipped to 1.5, rounded to
     half steps (clean clusters); skipped on small round parts (grip, pommel), whose icon light
     belongs to a 3/4 view;
   - face light from the icon's top left: up +0.5, south 0, north -0.25, west +0.25, east
     -0.5, down -1;
   - bevels: +0.5 along each side face's top row, -0.5 along its bottom, +-0.25 left and
     right; a light falloff down each side (+-0.3); occlusion where parts meet (-1.25 x
     `occlusion`);
   - the blade: +0.75 from guard to point; cutting edges +0.75, with chips (-0.75) at fixed
     heights so both faces agree; cord on a grip as rows (groove -1, lit top +0.5), never at a
     slant (that turns into a checkerboard);
   - total clipped to +-2: stronger pushes the ramp to cream and maroon.
5. **Sampling the icon**: per part, its front face to that part's box in the icon, inside the
   dark outline (erode the alpha ~19 px; drop the outline's red-purple and the magenta tint);
   side faces take the color just inside that edge. Where the model is longer than the icon,
   repeat the icon's middle back and forth (a mirror, never a jump: a jump leaves a seam).
6. **Review**: `shots.py` renders the viewer from any cameras (3/4, the icon's angle, close-ups
   of a face); compare side by side with the upright icon. Look for seams, outline colors on an
   edge, a part copying the icon's 3/4 light, colors drifting from the icon.
7. **Into the engine**: `build.py` writes `models/item/<name>.json` (Blockbench's Java Block/Item
   format: 16 units a block, a texel 0.25, `rotation` for the turned boxes) and its texture
   (`textures/item/<name>_model.png`). The engine reads face UVs to 1/128 of the texture, so
   textures up to 128 texels a side map texel for texel. As in Hytale, the slot shows the item's
   picture (`textures/item/<name>.png`, the icon) and the hand and ground the model
   (`Looks::slot_draw`). A `handheld` item's model is held like its sprite, so the build lays it
   in the 16x16 square as the sprite lies: the sword turned -45 degrees about its middle, handle
   bottom left (the points, turned +45, come out straight); a long sword's diagonal just fits.

## Scripts

Run them with `~/.venvs/blocky/bin/python` (Homebrew's Python 3.14 with scipy added; it sees the
system numpy and Pillow). Homebrew's own `python3` has no scipy and refuses installs.

- `scripts/house.py` — the house style's numbers: `DENSITY` (32 texels a block), `UNITS` (64
  layout units a block), `SCALE`, `TX` (texels a unit), `texels(scale, k)`, `fine(face_units)`,
  `MIN_FACE`, `ICON`, `MATERIALS` and `material(name)` (tones with the material's shadow hue).
- `scripts/render.py spec.json texture.png out.png [--scale s] [--ppb n] [--views 3q,side,head]`
  — a model drawn flat with its texture (no Blockbench): review shots and before/after sheets at
  the same size in the world, so densities compare.
- `scripts/prop.py` — props without a skeleton: `el`, `paint` (`down` lights stacked boxes as one
  piece, `smooth` drops specks), `entity_spec` (see Props).

- `scripts/bb.py` — run JS in a running Blockbench over CDP (`BB_PORT`, default 9333).
- `scripts/bbshot.py out.png [--anim name t] [--angles a,b]` — the open Blockbench project from
  four angles side by side (framed by its bounds), at an animation's frame; `--angles` picks others:
  `front_flat`, `side_flat`, `top`, `back` (flat views to hold against a reference sheet).
- `scripts/boxcheck.py spec.json` — coplanar overlapping faces (z-fighting) at rest and, with
  animations in the spec, in every keyframe (`check_posed`); exit 1 if any.
- `scripts/paint.py` — `tones`, `occlusion`, `paint_face_smooth` (default look), `EYES` (ten eye
  types: six Hytale-style, four cute) and `stamp_eyes`, `stamp`, `majority`, `pack`;
  `ramp`/`paint_face` (flat five-step look).
- `scripts/rig.py` — `quat`, `qmul`, `qmat`, `euler`, `swing` (wag about the axis across a
  part), `fit` (grid search for pose parameters).
- `scripts/ground.py` — `lowest(spec, pose, groups)`: each group's lowest world y in a pose;
  `joints(spec)`: joints whose pivot isn't inside both parts (parts that come apart);
  `pokes(spec, parts)`: corners poking out of a neighbor at an angled joint, at rest and in every
  keyframe (`python ground.py spec.json neck skull ...` prints them);
  `gait(spec, anim, feet)`: whether a walk or run moves forward (positive per foot).
- `scripts/variants.py model.glb out.json name=tex.png ...` — texture variants padded to the
  GLB's texture size, as data URLs for the viewer.
- `scripts/layers.py` — per-texel layers for coats painted on the fly (see above).
- `scripts/coats.js` — genes (random, breed), phenotypes per species, the coat renderer.
- `scripts/coatdata.py layers genes.json out.json` — a creature's layers and genes for the viewer.
- `scripts/viewer.html` — the viewer page: a switcher between creatures (`animals.json`), a
  button per animation and per texture variant, and the coat panel (three.js r147); per entry a
  person for scale, zoom, and playback rates for gaits (see its header). Turntable off by default.
- `scripts/bb_build.py <dir>` — builds `<dir>/spec.json` + `texture.png` in a running Blockbench
  as a Hytale Character and saves `<name>.blockymodel`, `anims/*.blockyanim`, `<name>.glb`,
  `<name>.bbmodel`.
- `scripts/to_bedrock.py spec.json texture.png <assets> --name <animal> --scale <s> --texture-name <mod>/<animal> [--pup 0.5]`
  — the game files (Bedrock `.geo.json`, per-pose `.animation.json`, texture) for Weft/chiacraft
  or Minecraft Bedrock: turns the model to face -Z, gives tilted boxes their own bones, bakes the
  scale, paints both sides of flat panels, one animation file per pose (`idle` holds it).

## Into chiacraft (Weft)

Follow `docs/creatures.md` in the chiacraft repository: export with `to_bedrock.py`, register the
entity (`.model_file`, `.animations` per pose, `.texture`, `.size`), and check it in the game
with a `--script` that spawns it and takes shots (`look yaw pitch`: negative pitch looks down).
Engine limits that shape the model: bones rotate but single boxes don't; a per-entity scale
(`World::set_scale`) for individuals, none per kind (bake the size in); only `idle`, `walk` and
`run` play by themselves (walk and run by speed, on top of the idle: `to_bedrock` writes them as
offsets from it; poses are look kinds with their own file); scale keys play; a played
animation replaces idle while it plays; no Molang; cut-out alpha only (no blending).

Worked examples, in the chiacraft repo's `animals` mod (each README explains it):
- `mods/animals/source/cat`: built from research alone (no concept sheet), one script
  (`cat.py`): boxes, color areas as rules, eyes, poses fitted to the ground (legs with `rig.fit`,
  the tail on ground targets), animations; `build.py` exports it. **Start a new animal by copying
  this folder.**
- `mods/animals/source/raptor`: a feathered Velociraptor, the two-legged example (bird legs,
  eyes on the sides, feather panels). Start birds and dinosaurs here.
- `mods/animals/source/capybara`: a stocky grazer (no tail, side eyes, chewing, grazing fitted
  to the ground). Start heavy herbivores here.
- `mods/animals/source/wyvern`: a huge winged dragon (the show's dragons): four-fingered bat
  wings with skin panels, the folded sail, IK-solved gaits on feet and wrists, a fitted lying pose,
  scale keys, genes with an accent color, a breath animation. Start winged creatures here.
- `mods/animals/source/nest` and `mods/animals/source/egg`: props (see Props).
- `mods/animals/source/dog/chihuahua`: built from a concept sheet (`tools/blocky.py`, colors read
  from the flat turnaround), with eyelids, jaw, tongue, solid ears, toes, a V bib.

## Sources

- Hytale art director's modeling guide: https://hytale.com/news/2025/12/an-introduction-to-making-models-for-hytale
  (cubes and quads only, 0.7-1.3× stretch, 64/32 px density, multiples of 32, paint shadows/AO/
  highlights in, no noise, no flat surfaces, no pure black/white, colored shadows).
- Blockbench Hytale plugin notes: https://cdn.jsdelivr.net/gh/JannisX11/blockbench-plugins@master/plugins/hytale_plugin/about.md
  (255 nodes, UVs locked to face size).
- Hue shifting for block-game skins: https://www.planetminecraft.com/blog/skinning-understanding-shading-hue-shifting-and-colors-5562100/
