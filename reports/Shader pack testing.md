# Shader pack testing: how others test, and the bugs they shipped

Research for chiacraft's shader pack (`mods/base/shaderpack/checks/README.md` is the checklist made
from it). Gathered 2026-11-04 from pack changelogs, issue trackers, engine docs and talks. No pack
publishes a test checklist; what they test is read from the fixes they shipped. Items marked
*unsourced* are standard practice that no source here states.

## How packs and engines test

- **What gets fixed, so what to test**: the End and the Nether (Atherial, Complementary #380), high
  altitude NaNs and sky darkening underground (Solas 2.2b, 3.6), 4K "sparkly colors" (Bliss 2.1.1),
  handheld lights on AMD (Solas 2.7), puddles without rain and rain at night (Bliss), fog behind
  glass (Bliss 2.1.0/2.1.1), Distant Horizons and TAAU (Bliss, Solas), light shafts broken on many
  NVIDIA GPUs (Complementary r2.2.1), Apple Silicon (Solas 1.7, 1.8, 2.1; Photon does not support
  Metal).
- **Compiler and driver traps** (shaderLABS wiki): normalizing a zero vector gives NaN; float
  equality misbehaves on NVIDIA; texture sampling outside `main()` on AMD; Apple Silicon crashes on
  global-scope calls and arrays of arrays; clamp the scene buffer or NaNs appear in black areas.
- **Validation**: naga's WGSL front end validates fully; `naga file.wgsl` alone only validates;
  its own tests compare snapshots. spirv-val and glslang on outputs *(unsourced)*.
- **Debugging**: Vulkan validation layers with Debug Printf, RenderDoc's NaN/Inf overlay,
  MoltenVK's `MVK_CONFIG_DEBUG` to compare the SPIR-V with the MSL it made.
- **Golden images**: render reference scenes and compare within a tolerance (for example every
  channel within 3/255 and under 0.2% of pixels off), writing the diff on failure; software
  renderers round differently, so give them a wider budget.
- **NaN and Inf**: from `sqrt`/`log` of negatives, division by zero, uninitialised targets, zero
  `normalize`; one bad pixel spreads through blurs. Fix the source. **MoltenVK compiles with fast
  math by default** (`MVK_CONFIG_FAST_MATH_ENABLED`), so `isnan` guards can vanish: clamp instead.
- **Costs** (shaderLABS): `exp2`, `log2`, `inversesqrt`, `sin`, `cos` are hardware ops; special
  functions run at a quarter rate on NVIDIA. No published frame budgets.

## Known bugs and edge cases

- **Fog**: missing behind glass or translucents; border fog through glass; water fog differing in
  and out of water; too thick in caves and the Nether; overexposed or flickering noise; not dense in
  storms; the eye in water, lava or powder snow (`isEyeInWater` 1, 2, 3). Banding in ray-marched
  fog wants jittered steps *(unsourced)*.
- **Light shafts as a screen pass** (GPU Gems 3, ch. 13): the sun near perpendicular or off screen
  sends its screen position far away (clamp or fade); occluders crossing the screen edge flicker;
  brightness-based masks leak where non-sky things are bright (snow, sand, lamps); even steps make
  patterns (jitter); without depth, background rays show in front of near things. The sun behind
  the camera needs its own test. A Lux issue reports a black flash looking at the sun.
- **Waving plants**: huge `sin` arguments make foliage jump on some backends (Vitrail v0.7.3: hash
  vertex bits instead); wave only listed blocks; potted flowers waved (Solas); a tall plant is two
  block models, so both halves need one wave anchored in the world.
- **Snow and wetness**: puddles without rain; wetness lagging rain; snow and wet under roofs or on
  walls without a sky-light mask and a normal test *(unsourced)*.
- **Rain**: rain indoors (a long vanilla problem, MC-169351; Ebin-Shaders #42); rain too bright at
  night (Bliss 2.1.1); lighting bugs in water while raining (Complementary #365, open).
- **Sky**: mirrored moon phases (Solas 3.2); stars ignoring settings or drawn on top (Photon #665);
  failing outside the Overworld (Complementary #376).
- **Colour**: night vision, blindness and darkness effects to test.
- **Precision**: OptiFine's `frameTimeCounter` wraps at 3600 s and anything not periodic pops;
  float time loses fractional bits as it grows (keep `sin` inputs within a period, reduce time on the
  CPU in doubles); a fog that snapped when time was reset (The Witness).
- **Platforms**: MoltenVK fast math, an fp16 transcendental fix, Apple Silicon workarounds for cube
  gradients and arrayed depth; WGSL requires derivatives and `textureSample` in uniform control flow.
- **UI and other**: the hand and banners, very bright players holding lanterns with dynamic lights
  (Photon #677, #671), enchantment glint overflowing (Photon #690), render scale above 1x.

## Sources

- Modrinth changelogs: https://api.modrinth.com/v2/project/solas-shader/version ,
  https://api.modrinth.com/v2/project/bliss-shader/version ,
  https://api.modrinth.com/v2/project/complementary-reimagined/version ,
  https://api.modrinth.com/v2/project/photon-shader/version , https://modrinth.com/shader/atherial-shaders
- Issues: https://github.com/ComplementaryDevelopment/ComplementaryReimagined/issues ,
  https://github.com/sixthsurge/photon , https://github.com/aurealis-graphics/lux-shader/issues/5 ,
  https://github.com/BruceKnowsHow/Ebin-Shaders/issues/42 , https://bugs-legacy.mojang.com/browse/MC-169351 ,
  https://mojira.dev/MC-267864
- shaderLABS: https://shaderlabs.org/wiki/Compiler_Behavior_Notes , https://shaderlabs.org/wiki/Shader_Optimization ,
  https://shaderlabs.org/wiki/Shader_Tricks , https://shaderlabs.org/wiki/Performance_Tips
- OptiFine shader doc: https://raw.githubusercontent.com/sp614x/optifine/master/OptiFineDoc/doc/shaders.txt
- GPU Gems 3, ch. 13: https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-13-volumetric-light-scattering-post-process
- Vitrail v0.7.3: https://github.com/avpbynf/Vitrail-Shaders/releases/tag/v0.7.3-beta
- MoltenVK: https://raw.githubusercontent.com/KhronosGroup/MoltenVK/main/Docs/MoltenVK_Configuration_Parameters.md ,
  https://raw.githubusercontent.com/KhronosGroup/MoltenVK/main/Docs/MoltenVK_Runtime_UserGuide.md
- Vulkan Debug Printf: https://github.com/KhronosGroup/Vulkan-ValidationLayers/blob/main/docs/debug_printf.md
- NaN propagation (Unity HDRP): https://docs.unity3d.com/Packages/com.unity.render-pipelines.high-definition@17.0/manual/Post-Processing-Propagating-NaNs.html
- naga: https://github.com/gfx-rs/wgpu/tree/trunk/naga ; WGSL uniformity: https://www.w3.org/TR/WGSL/#uniformity
- Golden images: https://docs.rs/fenestra-shell/0.21.0/fenestra_shell/testing/ ,
  https://docs.vulkan.org/tutorial/latest/ML_Inference/Desktop_Applications/04_ci_render_validation.html
- Float precision: https://community.arm.com/groups/arm-mali-graphics/blog/2011/09/06/at-home-on-the-range--why-floating-point-formats-matter-in-graphics
