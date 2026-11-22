# ImmersiveVFX 1.5.1 (Shinyflvres, NeoForge, MC 26.3): study for chiacraft

Method: read the shader sources (GLSL is plain text), `javap` for class constants and signatures, and the data json. Nothing was run. Class and particle names in the data are obfuscated (7-letter random tags). The readable words left are `spec`, `wind_gust`, `ground_mist`, `spore`, `dust`, `light`, `glint`, `pixel_glow`, so many facts below are inferred from numbers rather than names. Where I infer, I say so.

## 0. Architecture in one paragraph

The mod is two things glued together.

1. **A Bedrock add-on runtime** ported to Java. "Realistic Biomes" by Oreville ships as client-entity json, animation controllers, Molang scripts and 441 particle json files. The mod loads these and runs them:
   - a Molang parser/evaluator (`molang/*`);
   - a particle system with emitters (`bedrock/ParticleSystem`);
   - a state-machine host entity ("ui") that never exists in the world (`ClientEntity`).
   Java computes a handful of facts and exposes them as `q.property('p:...')` values and `v.` variables. The controllers then choose which particle emitters run.
2. **A set of real core-shader/pipeline replacements** that the Bedrock data cannot do. These are:
   - shadows and enhanced lighting;
   - water reflections;
   - post-pass distance fog;
   - sky, clouds and aurora;
   - bloom and tonemap;
   - caustics, shore foam and lava foam;
   - fire lights and "burn" overlays;
   - underwater light shafts;
   - block-place spring animation;
   - outlines.

Key point for us: the Bedrock layer is mostly **data-driven weather/ambience as particle emitters**. The ideas are good. The Molang-interpreter route is not worth copying. We would write a small per-biome "ambience table" in Rust/our mod data instead.

Config: 60-odd keys, one `preset` (IMMERSIVE, IMMERSIVE_TOON, ENHANCED (default), ENHANCED_TOON, VANILLA_PLUS, VANILLA_PLUS_TOON, CUSTOM). Toggles are packed into a bitmask `p:...` property and fed to the Molang scripts. Examples: fireflies=2, groundMist=4, weatherMist=8, specks=16, windGusts=32, leaves=64, shootingStars=128, sandWind=16 and sandStorm=32 on a separate word.

---

## 1. The profile selector (this drives everything)

**What the player sees:** each biome gets its own fog colour and distance, ambient particles and sky grade. Colour and distance change through the day and cross-fade (5 s, `SWITCH_SECONDS`) when you walk between biomes.

**Technique** (`profile/Profiles`, `ProfileTracker`, `BedrockTags`, `Metrics`):

- Every vanilla biome is mapped to a **Bedrock-style tag set** by `vfx/biome_tags.json`. Example: `minecraft:desert -> [desert, monster, overworld]`. Tags are bits in a `BitSet`. Mod biomes get tags from a convention list (name-based rules) plus `ivfx/modded/<mod>/<biome>.json` overrides. The override json only carries fogStart/fogEnd/fogColor/skyColor, blended over 2.5 s. They ship overrides for 11 snowy vanilla biomes and 3 `desolation` mod charred-forest biomes.
- A priority-ordered list of ~42 **profiles** each carries a predicate (tags + weather + indoor metrics), a fog key and a flag for "has time-of-day variants". First match wins. Order, roughly:
  1. End.
  2. Nether biomes: basalt deltas, crimson, wastes, soul sand valley, warped, then a generic nether.
  3. Deep dark, lush caves, dripstone caves, sulfur caves (underground), generic caves.
  4. Snow and blizzard.
  5. Aurora night.
  6. Cold, frozen/grove.
  7. Desert x3, mesa x3, savanna x3 (each: sand wind, sandstorm, plain).
  8. Rain, thunder.
  9. Sulfur, beach, pale garden, birch, cherry, roofed forest, taiga, forest, jungle, swamp, mushroom, ocean.
  10. Default.
- **Weather is a 5-value enum** the mod derives, not vanilla's: CLEAR, RAIN, THUNDER, AURORA, FIREWORKS. Vanilla rain and thunder are reinterpreted per biome:
  - Desert/mesa/savanna: RAIN becomes **sand wind**, THUNDER becomes **sandstorm**.
  - Cold biomes: RAIN becomes **snow**, THUNDER becomes **blizzard**.
  - Elsewhere: plain rain or thunder.
  So no new weather simulation is needed. The weather clock stays vanilla and the biome decides what it looks like.
- **Cave detection without a cave tag** (`Metrics`, `Rays`): a 40-direction Fibonacci-sphere probe. 2 rays updated per tick, so the full sphere refreshes every 20 ticks. Max ray length 100 blocks, solid-block raycast. It summarises three numbers:
  - `outsideExposure` = fraction of rays that escape;
  - `stoneFactor` = fraction hitting stone/deepslate/tuff/sculk;
  - `enclosureVolume` = estimated cavity size, computed from mean ray length (the code carries a 4pi-style sphere-volume constant).
  Thresholds: cave profile if enclosure > 30 and exposure < 0.2, or exposure < 0.1 and stone factor > 0.7, with tags checked as well. This is cheap and reliable. It gives nice transitions: a big cave opening stays "outside".
- The anchor point used for tags is the player, but with a 3-block hysteresis radius so biome borders don't flicker.
- **Time-of-day fog variants**: `Profiles.phase(time)` appends a suffix to the fog key by tick:

  | ticks | phase |
  |---|---|
  | 13700-23000 | night |
  | 23000-23500 | pre-dawn A |
  | 23500-24000 | pre-dawn B |
  | 0-500 | dawn |
  | 500-11500 | day |
  | 11500-12000 | late day |
  | 12000-12500 | dusk A |
  | 12500-12800 | dusk B |
  | 12800-13000 | dusk C |
  | 13000-13700 | dusk D |

  Each combination has its own hand-picked fog colour in `vfx/fogs.json` (771 entries).

**Cost:** negligible (a rolling 2-ray raycast per tick).
**Idea for us:** biome tag bitset, weather reinterpretation per biome, ray-sphere enclosure metric, phase-suffixed colour table. All trivially portable.

## 2. Fog (ground fog, ambient fog, distance fog)

Three separate systems.

### 2a. Vanilla fog replacement table (`fog/FogTable`, `vfx/fogs.json`, `FogRendererMixin`)

Each entry is `air / water / weather` x `[start, end, hexColour, relativeFlag]`. Relative flag 1 means the numbers are fractions of render distance. Sample numbers (blocks, absolute):

| profile | start | end | colour / note |
|---|---|---|---|
| default overworld day | 8 | 192 | sky blue |
| desert day | 8 | 192 | F2CF85 |
| desert sand wind / sandstorm | 2 | 96 | E8C97D |
| mesa sandstorm | 2 | 100 | B5743A |
| savanna storm | 2 | 64 | AA6D47 |
| rain | 2 | 48 | 345375 |
| thunder | 2 | 32 | 333B47 |
| blizzard | 2 | 38 | 5E76A0 |
| cold clear | - | 128 | E6EBF4 |
| nether wastes | 2 | 110 | - |
| End | 2 | 148 | 010013 |
| caves | 8 | 96-256 | - |
| aurora / fireworks | 1020 | 1028 | effectively off |

- Water fog is a separate triple per biome, e.g. swamp 4C6559, warm ocean 32A598, frozen 14559B, always 2..64.
- The `weather` triple differs from `air` only in a few cases, e.g. the ground-mist profile (air 8E8884 fading in to 3B3D42 in rain).
- **Fog modes**: IMMERSIVE (fixed distances), **IMMERSIVE_EXTENDED (default; ends doubled: desert 96 becomes 192)**, CHUNK_DISTANCE (fractions like 0.02..0.8 of render distance), OFF. The mode picks a suffix on the key.
- Rain and storm profiles are so short (48, 32) that they look like a wall. Sandstorm fog is the *same* distance as plain sand wind (96). The visual difference must come from the particle sheet (see section 5).

### 2b. Atmospheric fog (config `atmosphericFog`, default **false**; `vfx_distance_fog.fsh`)

A full-screen post pass.

- It rebuilds view position from depth, then applies exponential haze. The haze is clearly derived from Complementary's atmospheric fog.
- Near/LOD mode: `1 - 2^(-max(d-40,0)*(0.4+0.4*rain)/atmDist)`.
- Far mode: `1 - exp(-(d*(0.001-0.0007*rain))^(2-rain^2) * d * renderDistFactor)`.
- Multiplied by an **altitude factor** that is 1 at sea level (y 63.1) and falls quadratically to 0 over 90 (near) or 60 (far) blocks. A "camera altitude" term keeps mountain tops from vanishing.
- Multiplied by an eye-brightness/**cave factor** so interiors get no haze.
- Haze colour = the same sky model's horizon colour (day/night mix, 2.5x brighter at night when low), tonemapped.
- Final outer "border" fade to the sky colour in the last distance: `1 - exp(-3*(r/R)^4)`, to the 4th or 16th power, so the edge is sharp-ish.
- Clever part: it reads four colour copies (before, mid, late, final) and a depth copy so that **stuff drawn after fog** (clouds, translucents, particles) is subtracted out and re-added unfogged.
- **Cost:** moderate. One full-screen pass plus four sampled copies.

### 2c. Ground mist and "low-lying fog" (`client/LowFog`, particle `ground_mist`, default on)

- **LowFog:** up to **1100 camera-facing flat sheets** (shared 64x96 texture `low_fog.png`) within **64 blocks**, hugging terrain. Base size 2, distance scaling 0.1. The band top is at 0.35 block, growing 0.05/s of age. Tinted 35% toward the fog colour. Sheets sit a sheet-base 0.05 above ground with 0.35 spread.
- **Ground-mist particles:** the Bedrock profile `vxdvzqa_fqjwcke` (the swamp/forest-floor mist profile, fog end 32) has `ground_mist_sensor`, `ground_mist_emitter` and `pixel_glow`. The sensor decides where mist should form.
- **Cost:** cheap. 1100 quads, CPU-built.

## 3. Water: foam, caustics, light shafts, reflections, colour

### 3a. Caustics and shore foam (`client/SurfaceEffects`, `vfx_surface.fsh`; defaults **on**)

The best-built part of the mod.

- **CPU scan** every 10 ticks of a **49x49 column grid** (radius 24) around the camera. For each column it records water-top height, the floor under it, and surface height.
- **Caustics:** extra additive quads laid on the sea floor (strength 0.5, fade-in 0.6 s, fade out 16 to 23 blocks from camera).
  - The shader quantises UV to the 16x16 block texel grid, so caustics are **pixel-art cells**.
  - Value = two layers of **Voronoi cells**, using F2-F1 distance for the bright edge lines: `1 - smoothstep(0, 0.2, d2 - d1)`.
  - The two layers are at scale 0.75 and 1.35, animated over time at 0.7x and 1.05x speed, mixed 0.65/0.35.
  - The result is squared and boosted so edges are crisp, then multiplied by vertex light colour (so it respects day/night and sky light), and fogged out.
- **Shore foam:** a quad per shoreline cell with an 8-bit **neighbour mask in vertex alpha**. Bits 1,2,4,8 mark the four edges and 16,32,64,128 the corners. The fragment computes distance-to-edge in texels, `d = min over set bits`.
  - Then a **sine wave moving along the shoreline** (`phase = t*1.25 + (u+v)*0.045`) sets the foam front: `front = floor(2 + 4*wave + jitter)`.
  - Inner `d < front-2` is opaque white (0.85 alpha). The band to `front` is pale blue (0.62). An outer ring `d` 6-7 appears only on the retreating half of the wave, as sparse dots.
  - Random sparkle pixels sit just beyond the front. `jitter` is a hash per 4-texel run, so the edge is ragged but stays pixel-aligned.
  - The result looks like a retreating-wave animated 16 px foam fringe. Alpha fades with `foamFade`.
- **Lava foam** (default on) reuses the same shader with `kind < 0.75`. Cream core (1.0, 0.96, 0.70), orange rim, and sparse bright flecks. It is slower (`t*0.35`). Scan range 12 above / 24 below the camera.
- **Cost:** moderate. Scan is 2401 columns / 10 ticks; draw is a few thousand small quads in the translucent pass. Fragment cost for caustics is 2x9 cell lookups.

### 3b. Underwater light shafts (`VfxWaterShafts`, `vfx_water_shafts.fsh`; default on, intensity 150% in IMMERSIVE preset)

- Full-screen **raymarch with 12 samples** per pixel. Samples are spaced quadratically (`t*t*maxDistance`) with a per-pixel dither. Max distance 96, min 24 blocks, brightness 0.2.
- Each sample asks "is this point below the water surface?" (surface height is a uniform, scanned 48 blocks up).
- Then it samples a **2-octave tiling noise** at the point *projected up along the sun direction to the surface* (`slope * h`). That gives refracted caustic-shaped beams.
- If shadows are on, it also tests the shadow map so shafts break around land and roofs.
- Accumulates a smoothstep-weighted integral and multiplies by `0.5 + 0.25*(dot(viewDir, sunDir)+1)` (forward-scatter bias).
- **Cost:** moderate, a 12-tap pass.

### 3c. Reflections (`VfxReflections`, in `vfx_shadow.glsl`; water and ice, defaults on, strength 100%, quality NORMAL; LOW/NORMAL/HIGH)

- Screen-space reflections: **ray march in world space** with step doubling, up to **38 samples** and a refinement count by quality. Hits are validated by depth error. Edge fade `1 - pow(maxcdist, 50)`.
- Misses fall back to the **procedural sky** (the same Complementary-like model) scaled by skylight.
- Fresnel `(f^3*0.85+0.15)`, halved when seen from below, 0.7 for ice. Ice is found by testing UV against 5 atlas rects for the ice sprites.
- **Water colour mapping** (default on): water colour is derived from the biome tint, `sqrt(tint)*(1,.85,.8)*.375`, modulated by the texture luminance. Opacity is `0.98*(0.6+0.4*waterFog)`, where `waterFog = 1 - exp((eyeDist-floorDist)*0.075)` uses scene depth, so shallows go clear. Fresnel pushes alpha to 1 at grazing angles. Done in-place in the terrain shader with a `VFX_REFLECT` define.
- **Cost:** heavy-ish (the one notable per-pixel loop).

### 3d. "Water leaves" (default on)
Falling leaves landing on water. Not examined in depth.

## 4. Light and shadow: "pixelated shadows" without Iris

This is the technique most worth stealing. **It is a single shadow map re-rendered from the sun, applied inside the modified vanilla terrain shader.** There is no deferred pipeline, which is where the speed claim comes from.

`VfxShadows`, `vfx_shadow.glsl`, `vfx_terrain.fsh`, `vfx_sodium_terrain.fsh`, `ShadowSections`, `ShadowCull`, `Sodium*Lists`:

- **How it works without Iris:** it changes render pipelines in place (`VfxShadows.modify(pipeline, id)`). Vanilla/Sodium terrain, entity, block and particle shaders are swapped for `vfx_*` versions with `#define`s (`VFX_MARK`, `VFX_REFLECT`, `ALPHA_CUTOUT`), so each draw that is already happening also reads the shadow map. A parallel "caster" pipeline (`vfx_shadow_caster`, plus a cutout variant for leaves) is run into a depth texture. For this it re-submits the already-built chunk sections that lie inside the light frustum (via Sodium's visible-section lists, plus a Sodium frustum mixin that substitutes a sun-view frustum). So there is **no second chunk meshing**, only a second draw of existing buffers.
- **Resolution:** `16 texels per block` of coverage, clamped to 512..4096. Default distance 32 blocks (config 'shadowDistance'; IMMERSIVE preset 256). Margin 16, reach 256 away from the sun and 96 toward it. Only the sections within 14 section-radius of the camera are drawn.
- **Update policy:** a map is only re-rendered when a section changes (a `dirty` flag from chunk updates), or the sun angle crosses a quantised **ANGLE_STEP**. The previous map is kept and the two are cross-faded over 1.5 s, so there is no popping. Two map objects, `VfxShadowMap` and `VfxShadowMapPrev`.
- **"Pixelated" part:** `vfx_texel_snap`. The fragment's world position is moved to the **centre of its texture texel** using screen-space derivatives. Then the shadow is looked up there. So shadow edges step in exact 1/16-block pixels that match the block texture grid. With 16 shadow texels per block the two grids align, and the shadow looks like pixel art. Bias scales with distance: `(0.12 + 0.0008*dist^0.75)*(2-0.95*NdotL)`.
- **Filtering:** 5-tap plus-shaped PCF, offset `0.75 texel` for normal blocks. For foliage (flagged in an emissive/foliage atlas channel) an 8-tap rotated disc is used, walking along the light vector, so leaves get soft dappled light.
- Edge fade to "lit" in the outer 80-95% of the range. Rain shadow reduction x0.4. Mob shadows toggle (entities cast into the same map).
- **Shadow modes:** NORMAL keeps vanilla lightmap/face shading and darkens by shade. **ENHANCED (default)** replaces the lighting equation (`vfx_enhanced_color`):
  - Colour sources: warm block light `(0.1775, 0.104, 0.077)`, sun colour, ambient.
  - Sunlit amount reduced by shadow; a direction term `0.75 + 0.25*N.y`; then a Lottes-style tonemap (`xa/(xa*0.919+0.7188)`), blended 80/20 with a simple Reinhard-ish curve.
  - Note: it un-bakes vanilla's per-face shading first (`vfx_vanilla_unshade`: up 1.0, down 2.0, z 1.25, x 1.667) so shadows do not double-darken.
- **Emissive via the albedo atlas:** a one-channel atlas (`VfxEmissive`) stores glow band (upper nibble) and foliage flag (lower nibble) per texel, built from the resource packs. Used for bloom, self-lit blocks and emissive alpha.
- **Cost:** moderate. One depth pass of nearby sections only when something changed; per-pixel 5-8 taps. Cheap because it is cached and cross-faded.
- A separate `VfxLodLight` pass applies the same light to distant-horizons style LOD terrain (depth + lightmap textures).

## 5. Weather: rain, snow, blizzard, sand wind, sandstorm

### Heavy rain (particle `cvsmalc`, plus controller `dsdacm`)

- Vanilla rain rendering is cancelled (`WeatherEffectRendererMixin`). `weatherVisuals` is VANILLA, ENHANCED or CUSTOM (default CUSTOM). `VfxWeatherPass` draws the particles in their own pass near the end of the frame.
- One emitter, parented to the camera: **750 particles/s** at full rate (`1000/2.7*0.75`), max 1000 alive.
- Spawned on a **disc of radius 20 at 40 blocks above** the player, aimed straight down.
- Speed 24-32 plus acceleration -4, so about 2.7 s of life.
- Each drop is a 3 x (0.75-1.25) block **y-aligned streak billboard**, blend material, grey-blue `(150,164,202)` alpha 0.125. It is very faint per drop; density makes it read as heavy rain.
- **Impact without collision queries:** `expire_if_not_in_blocks [air, tall_grass, short_grass]`. When a drop leaves those blocks it dies and fires an expiration event that spawns a **splash**.
- A second "sensor" emitter (`jnlldsq`, 80/s) fires a parametric random-walk probe 40 steps long in a downward cone. It finds the nearest non-air block and spawns splashes there, for ground impact variety.
- Rate scale `qxeeelp` comes from the `particleRate` config bits: MAXIMUM=1, REDUCED=0.5, NONE=0.
- Thunder uses the same machinery with a darker fog (32 blocks) and its own emitters (`mbelegb`).
- The `Weather Mist` toggle adds low mist during rain.

### Snow / blizzard (`vcppzvp_*`)

- Snow specks: disc of radius 6-24, spawned on **snow-layer blocks** (via `expire_if_not_in_blocks [snow_layer]`, at 0.9 of lifetime), so snow gets kicked up from the actual snowy ground.
- Wind emitter: 80/s, lifespan 1 s. Blizzard fog is 38 blocks, blue 5E76A0.
- Option `snowAndSandParticleStyle` HD or VANILLA (default VANILLA).

### Sand wind and sandstorm (desert, mesa, savanna)

How it is decided: **vanilla RAIN in a tagged biome becomes sand wind, THUNDER becomes sandstorm**, if `sandWind`/`sandStorm` are on. The `clunyu_ollhdpx` controller has one state per biome kind (desert, mesa, savanna). It enters a state when `init && enabled && biomeTag && weatherFlag` is true and stays until the condition ends AND a weather counter hits 9 (the hysteresis delays the stop). On entry it starts one big emitter, parented to the camera locator.

What is drawn:

- **The sandstorm cloud emitters are the three particle files that this jar does not ship** (`asxjyhw` desert, `wvxxhia` mesa, `phnlqvk` savanna are referenced by the controller but absent from `ivfx/particles`). I could not read the sheet geometry. Everything below is the *wind* and *speck* layer that is present.
- **Fog for both the wind and the storm:** short and tinted. Desert E8C97D at 96 (192 extended), mesa B5743A at 100, savanna AA6D47 at 64. The time-of-day variants keep it tinted day and night.
- **Specks (`wngibvq_*_spec`):** rate 20/s (desert), 35/s (savanna), max 1000. Disc radius 4-16 (desert) or 4-24, offset y -5..+20, 3-5 s life, `particles_alpha`, pale `(255,249,214)`, size about 0.03 block. Plus a 5/s **glint** emitter on the additive material.
- **Gusts (`wind_gust`):** 2/s, max 40. **Long streaks, 5-53 blocks wide by 0.25 high**, `lookat_direction`, additive, tinted `(255,217,147)`.
  - The disc has radius 10-24 at y 4-8 and direction +x. Life 1.5-2 s, speed 1.
  - Texture `rb_cf` is a 240x1620 flipbook of 36 frames (240x45 each).
- **Wind specs (`wind_spec`):** the fast sand curtain.
  - A box emitter 64 blocks wide, placed 40 blocks upwind of the player.
  - 3/s with a 1-4 s random sleep (so they come in bursts); speed 20-25, direction x 17.
  - 3.75 s life, a slight acceleration `(0.25,-0.2,0)`, fade-in at 40% and fade-out at the last 40% with Hermite blend.
  - Texture `rb_cl` (256x128) and `vanilla` variant `rb_cm`.
  - Speed times lifetime gives about 80 blocks of travel, so a sheet crosses the whole view in under 4 s.
- **Per-biome tint:** desert (227,219,176) wind-spec, savanna (99,95,44) alpha 0.5 and wind spec from a parametric sine path (`yeylzfy` speed, a slow vertical sine whose frequency changes with age).
- **Savanna "sandstorm"** is really **dust plus blown leaves**. A finder emitter (`bxnhhwt`, 1000/s for 0.2 s) fills a box of 70x20x70 and keeps only particles that land in acacia/oak leaves. At 0.9 of life those spawn two follow-up emitters, so leaves shed from nearby trees.
- **Hidden probe particles** (size 0, `particles_opaque`): one random-walk particle per sample expires when its block is not in a list (`[sand, sandstone]` for desert, `[grass_block]` for savanna). The surviving position is used to place a visible particle (`tejcfbz_hkgzkdv`, additive tex `rb_by`, a 196x196 sprite) at a *ground-hugging position*. This is how Bedrock gets a block query without scripting. In our engine we would just call a world query.

Cost: **moderate**. Steady particle counts of roughly 100 to 1500 instances; additive overdraw of big streaks is the main cost.

## 6. Sky, clouds, aurora, meteors, fireworks

### Sky (SkyMode: VANILLA = "IVFX Vanilla", REALISTIC = default in Immersive preset only)

- **VANILLA:** the Bedrock sky. A box-model sky (`models/sky.json`) plus a colour atlas: `vfx/sky.json` has **30 profiles x 8 rows x 15 columns** of hex colours.
  - The columns are time keyframes, in ticks: 18000, 22298, 23000, 23500, 0, 500, 6000, 11198, 11500, 12000, 12500, 12800, 13000, 13700, 18000.
  - The 8 rows are sky bands (zenith to horizon, sun glow and so on).
  - A second file `sky_textures.json` encodes the same as a 15x8 atlas per profile.
  - Biomes pick a profile, e.g. desert gets warm dawns.
  - This is a lerp-in-time colour ramp per biome, pure data.
- **REALISTIC** (`vfx_realistic.glsl`, `vfx_sky.fsh`): a closed-form procedural sky **adapted from Complementary Shaders** (the structure and constants match: `noonFactor`, `sunVisibility`, `nightFactor`, glare formula, "sunsetDownP"). It computes zenith/middle/horizon colours from the vanilla sky colour, rain, and sun height; adds a sun glare `0.075/(1-(0.925)*cos^p)`; a below-horizon scatter mix; then a Lottes-ish tonemap. Rain removes 75% of glare and desaturates. The same function is reused by fog, clouds and reflections, so everything agrees on the sky.

### Clouds (`VfxClouds`, `vfx_clouds.fsh`; rendering OFF / FANCY / FLAT / SHADED, default FANCY in config and SHADED in Immersive; formation LEGACY / MODERN default MODERN)

- A real **mesh of cloud cells** extruded from a cloud-map texture (`legacy.png` 128x128, `modern.png` 256x256). Cells have a per-vertex style colour: shade, weight and offset bytes.
- Constants: reference 128, model base 1.875, scroll speed 0.03, modern radius 4.358 and legacy 5.272 (in cell units), max radius 1024, fade 0.85.
- `cloudWeight()` turns cell distance into a density falloff so edges are soft. Modern is a smooth `t^1.5` falloff.
- Shading (SHADED mode): ambient + sun-lit colour by day/sunset/night/rain, with a silver-lining term from sun direction. Clouds fade into the sky colour with distance (`(4000-d)/4000` squared).
- A **coverage pass** (`CLOUD_COVERAGE` define) writes cloud alpha so the fog pass can skip fogging over clouds.
- **Cost:** cheap (mesh rebuilt on scroll, no raymarch).

### Aurora (`VfxAurora`, `vfx_aurora.fsh`; default on)

- **Fullscreen sky pass, additive, 16 layers, ridge-noise curtains.** For each pixel it intersects the view ray with parallel horizontal planes. Each layer samples a 128x128 noise texture, takes `ridge = (1-2|n-0.5|)^16` (a thin bright line where the noise crosses 0.5), and multiplies it by two finer noise samples for fray.
- Colour mixes magenta-violet `(7,2.2,12)` at the top of the curtain to green `(6,16,12)` at the bottom, `low^4` weighted. Layers are dithered per pixel.
- Visible mask: `sqrt(up*1.5-0.225)*(1-0.9*up)`, so only 15% above the horizon up to roughly zenith, faded at the zenith.
- Uniforms: fade 6 s (`FADE_SECONDS`), brightness 0.6, slow drift in noise space.
- **When:** `SkyEvents`: when the clock passes tick 14000 on a CLEAR night, **50% chance** an event lasts **3520 ticks**. The event is an aurora in cold biomes (frozen, snowy, grove, peaks; profile id -2, weather AURORA between ticks 13700 and 23000) and a meteor shower window of 1000 ticks elsewhere. Fireworks: at tick 18000 a **2% chance** (guaranteed on Dec 31) of 600 ticks of fireworks with bursts every 15 ticks. Config: `auroraBorealis`, `meteorShower`, `fireworks`, `alwaysWinter`.
- **Cost:** moderate (16 texture-pair samples per sky pixel, only when the aurora is visible and only on sky pixels).

### Shooting stars
Bedrock particles (`shootingStars` toggle).

## 7. Particles per biome (the 441 json files)

**Schema:** standard Bedrock particle json: `description.identifier` and `basic_render_parameters {material, texture}`. Materials are `particles_alpha`, `particles_add`, `particles_opaque`, `particles_blend`.

Components used and how often (of 441):

| component | files |
|---|---|
| lifetime_expression | 441 |
| appearance_billboard | 400 |
| emitter_initialization (sets the rate scale `v.qxeeelp ??= 1`) | 319 |
| emitter_lifetime_looping / once | 304 / 135 |
| initial_speed | 252 |
| appearance_tinting | 243 |
| expire_if_not_in_blocks | 243 |
| rate_steady / rate_instant | 241 / 200 |
| motion_dynamic | 215 |
| lifetime_events | 180 |
| shape_point / disc / box / sphere | 177 / 103 / 73 / 8 |
| motion_parametric | 71 |
| motion_collision | 6 |

Facts:

- 63 `spec` (ambient specks, usually in a day variant and a night variant), 23 `wind_gust`, 10 `mist`, 8 `spore`, 6 `dust`, 4 `light`.
- Textures are the 62 tiny `rb_*.png` sprites. Many are 1x1 solid pixels (for square pixel specks), 4x16 to 9x36 flipbooks, and a few big ones (`rb_ch` 1280x1080, `rb_cw` 1416x256).
- Names are group-hashed. Controller groups: weather, dust, specks, wind gusts, "feature finders", sandstorm.
- **Per-biome set** (from controller states): jungle, swamp, taiga, forest, birch, cherry grove, roofed forest, mushroom island (spores with `mushroom_spore_emitter`), pale garden, ocean, beach, deep dark, lush caves, dripstone, nether biomes (ash-like), End. Each has a day and a night speck variant, so fireflies at night and pollen by day.
- Interesting numbers: spore and pollen motes are also done on the Java side (`AmbientMotes`: **max 600 motes, radius 11 blocks around player**, y offset -6..+7, lifetimes in seconds with a sine drift) as a cheap alternative.
- `particleRate` MAXIMUM / REDUCED (x0.5) / NONE.

## 8. Fire (`DungeonsFire`, `vfx_fire.fsh`, `vfx_fire_light.fsh`, `BaseFireBlockMixin`; default **on**)

- `BaseFireBlockMixin` suppresses vanilla smoke so the replacement is not doubled.
- Three procedural kinds, all drawn as camera-facing or surface-aligned pixel quads:
  1. **Smoke**: pixel discs shaded by `1 - 0.2*y - 0.12*d^2`. 288 per second per cap, max 2160. Emitted within **128 blocks**, smoke considered near within 10.
  2. **Embers**: max 1600.
  3. **Burn**: a charred overlay on blocks that are burning (up to **6000** cells, budget 600 per update, within **72 blocks**). It is drawn from a *heat field*: `vein*0.62 + wide*0.3 + detail*0.12 + pulse` where `vein = 1-|2*valueNoise-1|` gives thin crack lines. Thresholds map heat to colours: >0.93 pale yellow, >0.85 orange, >0.78 dark red, otherwise charcoal with alpha. Blocks actually on fire shift by -0.14 so they glow more. So burning blocks get **glowing lava-crack veins that crawl** (pulse phase per pixel).
- **Fire light:** `VfxFireLight` gathers up to **32 lights** (grid cell size 8, rebuilt every 4 ticks, 32 chunks scanned per tick) at **quarter resolution**. A screen-space pass adds a tinted surface glow `falloff^2 * facing` plus a soft volumetric halo, `exp(-r^2/(0.34R)^2)` wide plus a tight core, along each view ray (a closest-approach-to-light formula, no raymarch). Cheap and pretty.

## 9. Bouncy block placement (`SmoothBlockPlace`, mixin `BlockStatePredictionHandlerMixin`; default on)

- When the client predicts a placement, the real block is **hidden** until its chunk section recompiles, and a "moving block" copy is drawn with a **damped spring** transform for **0.42 s**.
- Scale is a squash-and-stretch spring with settle time 0.075 s and angular frequency omega 26.18 rad/s (about 4.2 Hz). Stretch amplitude 0.26.
- A tilt spring (omega 20.9, settle 0.09, amplitude 0.1) leans the block about an axis from the placed face normal.
- **Groups:** consecutive placements with the same prediction sequence number are one group (a fill or a bridge-click run). Spread 0.13 with lag 0.9 staggers members so a row of blocks ripples.
- Limit 64 live placements. Handoff timeout 1.5 s (so a lost server confirmation can't leave a ghost); a 150 ms Sodium handoff delay.
- A cheap, high-feel effect. The key trick is hiding the real block until its mesh is ready, then handing over with no pop.

## 10. Other features not in the advertised list

- **Bloom** (default on, intensity 100): 8-tile pyramid at 1/4 res (down 4x then ATLAS_SCALE 4), masks from the emissive atlas with several glow classes (bright-only, enhanced, whole block, all). Composite max mix 1.0. Moderate.
- **Tonemap + saturation 1.056** (`VfxTonemap`), only in ENHANCED shadow mode.
- **Colour grading** OFF / WARM / AUTUMN (3 fixed 4-tuple matrices selected via properties).
- **Natural foliage colours** (default on): remaps biome grass/foliage tints to less saturated, temperature/humidity-dependent colours (`NaturalFoliage`, pale-garden tint 1.34/1.07/1.16).
- **Outlines**: block and mob outlines (default off) and **enchantment outlines** (default on): edge-detect passes for enchanted item glints (dilate rows/tiles).
- **End**: End colour grading and "End aura" (`vfx_end.fsh`), custom ambient light `(0.2176, 0.1632, 0.3424)`.
- **Light rays** (god rays, defaults on, intensity 170 in the Immersive preset), driven by `SunProbe` (a ray from the player toward the sun, checking for blocking up to 40 blocks) and `vfx/light_rays.json` (a 29-colour palette).
- **Debug UI**: `BiomeDebugScreen` for editing fog/sky per biome live.
- **Per-dimension toggles** for modded dimensions (`VfxDimensions`).

## 11. Where the "2-3x faster than Iris+Complementary" claim comes from (inferred; the jar gives no numbers)

- No G-buffer or deferred lighting. Lighting is done in place in the one forward terrain shader, with one cached shadow map.
- Shadow map is re-rendered only on change, covers 32 blocks by default, and reuses existing chunk buffers.
- Fog, bloom, fire light and underwater shafts all work at a quarter of the resolution or use few samples (12 for shafts, 38 max for SSR).
- Weather and ambience are CPU particles in the existing batch path, not per-pixel noise.
- The sky model is closed form.
- Atmospheric fog is off by default, and the heaviest options (SSR high quality, 256-block shadows) are only in the Immersive preset.

## 12. Steal list for chiacraft (ranked, no code copied)

1. **Biome tag bitset + weather reinterpretation** (RAIN to sand wind/snow, THUNDER to sandstorm/blizzard) and a short per-profile fog table with time-of-day colour keyframes. Cheap, big mood win.
2. **Ray-sphere cave/exposure probe** (40 rays, 2 per tick) to switch from "outside" to "indoor" ambience.
3. **Texel-snapped shadow lookup + one cached sun shadow map** with cross-fade. This is the "pixel shadow" look.
4. **Shore foam with an 8-neighbour mask and a travelling sine front, all in 16 px texel space; Voronoi caustics on the floor.** Both are pure shader arithmetic on a CPU-scanned 49x49 grid.
5. **Aurora as a 16-layer ridge-noise sky pass**, scheduled as a 50%-per-clear-night event.
6. **Rain as a camera-parented streak emitter** (750/s, disc r20 at +40 up, speed 24-32) with impact via heightmap, not collision.
7. **Smooth block-place spring** with the real block hidden until its mesh is ready.
8. **Burn overlay** (crack-vein heat field, charcoal to orange to yellow) and quarter-res fire lights.
9. **Falling leaves from nearby leaf blocks** (find leaf blocks in a 70x20x70 box, shed from them).
10. Do **not** copy: the Molang/Bedrock interpreter and the block-probe-by-particle hack. Use direct world queries.

## 13. Licence note
"All Rights Reserved", credits Oreville's Bedrock add-on. Treat everything here as a description of ideas only. Write our own shaders and tables. The colours, noise textures, fog colour tables and particle numbers are creative content and should not be copied.
