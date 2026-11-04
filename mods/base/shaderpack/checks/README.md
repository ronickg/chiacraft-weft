# Checking chiacraft's shader pack

What to run before a change to the pack (`mods/base/shaderpack/`, `mods/base/assets/shaders/post/`)
reaches players, and the bugs other packs shipped, so ours does not. Research behind it:
`reports/Shader pack testing.md`.

## 1. Offline, every change

- **Validate with naga** the way the engine composes each file (prelude, `look.wgsl`, the file,
  snippet dispatch): any parse or validation error makes the engine fall back to its own shader.
- **Compile the result for Metal** (`xcrun metal -std=metal3.0`): the Mac runs it through MoltenVK.
- The checker used so far composes the six pack files and the screen pass with the light shafts,
  writes MSL and leaves the `.metal` files to compile; it lives outside the repo (a small crate on
  `weft_engine::gpu_layout::prelude`). Adding it as a `vitals`-style bin is a follow-up.

## 2. In the game: `edge-cases.txt`

```
./target/release/weft --world edge_$RANDOM --script mods/base/shaderpack/checks/edge-cases.txt \
  --shot-dir <dir> --size 640x360 --shader-pack chiacraft
```

A fresh world name each time. On the Mac only with nothing else running, and stop at the first
stall (a hung GPU once took the whole Mac down). It takes about a minute and saves:

| shot | what can go wrong there |
|---|---|
| 01 sun behind | shafts or glow drawn with the sun behind the camera |
| 02 straight down, 03 straight up, in fog | a zero vector normalized: NaN, black or flickering pixels |
| 04 sunset, 04b golden hour through posts | no rays at a low sun; the whole view bleached |
| 05 closed stone room in thick fog | fog filling a room or a cave |
| 06 under water | the pack's fog or shafts drawn under water |
| 07 high above the fog | the fog layer following the eye up |
| 08 30,000 blocks out, in a gale | precision: noise or waving jumping far from spawn |
| 09 snow beside a roof | snow lying under the roof; snow drawn as rain |
| 10 night rain under a roof | shafts at night; rain under the roof |
| 11 the flatlands | another dimension |

Then look for pure black pixels in the world (a NaN usually shows as one; the block outline is
black too, so look at where they are) and look at each shot. Test runs pin the shaders' clock, so
motion (wind, ripples, drifting fog) does not show in shots: watch it in a normal game.

## 3. Known traps (from other packs' bugs)

- **Zero vectors**: `normalize` of a flattened or crossed vector that can be zero (looking straight
  down, a squashed quad): add a small offset first. MoltenVK compiles with fast math, so `isnan`
  guards may be optimised away: clamp instead.
- **Huge sine arguments**: world positions far from spawn and the growing clock fed into `sin`
  lose precision (foliage jumps): wrap positions (the wind takes them within 4096 blocks).
- **The clock**: it counts up from start; after a day of play the fastest waves can stutter.
  Not fixed (the engine's clock would have to wrap on a period every effect shares).
- **Light shafts without depth**: brightness stands in for "sky", so very bright snow or sand
  near the sun can leak a little, and rays can show in front of near objects. Judged against the
  sky round the sun now; jittered so even steps make no bands; off at night, in rain, under water.
- **Fog in every material**: blocks, mobs, held models and particles each take the sky light where
  they are, so caves stay clear; translucent blocks are fogged like opaque ones.
- **Tall stacked plants** (sugar cane) sway segment by segment and can zigzag in a gale: the shader
  cannot tell where a stack is rooted.
- **Weather at biome borders**: the air of the biome the player stands in colours the whole view,
  and changes over a few seconds when crossing.
