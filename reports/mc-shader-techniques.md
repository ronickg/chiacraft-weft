# How seven Minecraft shader packs get their looks

This report reads the GLSL source of seven Iris/OptiFine packs and explains how each one makes its look, so we can compare them with chiacraft's own pack (`mods/base/shaderpack`).

**Source:** `~/Coding/chiacraft-archive/references/shaders/mc-packs-2026-11-05/src/`. The packs are BSL v10.1.8, Bliss v2.1.2 (a Chocapic13 edit), Complementary Reimagined r5.9.3, Complementary Unbound r5.9.3, Mellow v3.5, Solas V3.9 and Photon v1.4.

**How to read the paths:** every file path is relative to that pack's root folder. Each number given is the pack's default unless the text says otherwise. "Not found" means we read the code and did not find the feature; it does not mean we guessed.

## How the pack descriptions were made, and what was spot-checked

The pack descriptions were put together from a careful read of each pack, one reader per pack. A few claims were then checked by hand against the source:

- Bliss's default tonemap: `TONEMAP ToneMap_AgX_minimal` (`shaders/lib/settings.glsl:548`).
- The two Complementary packs differ in a single file.
- Photon's default tonemap is Lottes, and its clouds use 4× temporal upscaling with 40 primary steps.
- Mellow's tonemap operator is 6 (Uchimura).
- Bliss's bloomy fog line is in `dimensions/composite11.fsh:152`.

## Our pack, for contrast (`docs/shaders.md`, `docs/sky.md`, `mods/base/shaderpack`)

Our pack works like this. Lighting is forward and done in each WGSL file: the shared code is `look.wgsl` and the blocks are drawn by `world.wgsl`.

**Sky** (`look.wgsl` `palette()` and `sky_color()`)
- A hand-keyed analytic palette with five keys: day, golden hour, sunset, dusk and night.
- It also draws a sun glow, a warm side and a purple side, stars, the Milky Way and an aurora (`sky.wgsl`).

**Clouds**
- They come from a picture. The `docs/sky.md` layer (12×4-block boxes) is drawn in the pack as **rounded slabs**, raymarched with about 10 dithered samples (`sky.wgsl` `fs_clouds`). The rounded slabs are Complementary Reimagined's idea, written anew.
- They are lit with Henyey-Greenstein.
- In rain, a storm-deck field forms the deck.
- Cloud shadows on the ground come from `look.wgsl` `cloud_shadow()`.

**Fog**
- A height-and-valley fog with noise banks, lit through one shadow-map tap per step (`fog_through`).
- Haze tinted by the sky (`haze_lit`, `look_haze` 1.6).

**Post effects** (`mods/base/assets/shaders/post/*`)
- God rays (`volume.wgsl`): a shadow-map raymarch with 16 steps reaching 160 blocks.
- Screen-space shafts with 40 steps.
- Reflections: screen-space, 28 steps.
- Contact AO with 8 directions.
- Bloom: a single pass that samples three rings around the bright part.

**Display** (`look.wgsl` `display()`)
- By day the Lottes curve: contrast 1.05, midpoint 0.25 maps to 0.25, peak 8, exposure 1.3. Then saturation 1.2.
- By night Hejl 2015, plus a cool floor so nothing goes fully black.
- A warm/cool split toning in golden hour.

**LOD**
- Distant tiles go through the same `shade()` as near blocks (`world.wgsl` `fs_lod`), with a per-block texture fading to the average colour, and LOD water through the real `water()`.
- They cast into the far shadow cascade, which reaches 400 blocks (`fs_lod_shadow`).

**What our pack does not have**
- No temporal anti-aliasing and no history buffers.
- No sky lookup table or sky map.
- No puddles.
- No Purkinje shift. The word "far_look" appears nowhere in the pack.

---

## At a glance

| | BSL | Bliss | Complementary R / U | Mellow | Solas | Photon |
|---|---|---|---|---|---|---|
| Sky model | analytic gradient | single-scattering raymarch into a 256² map, accumulated over frames | analytic 3-colour gradient from vanilla `skyColor` | 2-colour analytic gradient, keyframed | analytic, keyframed, per-biome | Bruneton precomputed scattering lookup table (3D, shipped pre-baked) |
| Clouds | 1 volumetric slab, ≤32 steps, no march toward the light | 3 layers (2 volumetric, 1 flat sheet), 15 steps each, powder and multi-scattering | R: blocky rounded slab, first hit only; U: 4-octave noise slab, no march toward the light | flat 2D by default (volumetric option, 12 steps) | volumetric, ≤28 steps, 1 light tap per step, plus a 2D high layer | 5 physically based layers, 40/20 steps, 8 multiple-scattering octaves, refreshed at 1/16 per frame |
| Cloud shadows | option, off by default | yes, also in fog | option, off by default | not found | yes, also in god rays | yes, 512² map, plus god rays from it |
| God rays | shadow-map march, 7 steps | shadow-map march, 8 exponential steps, half resolution | shadow-map march, 10+1 steps | screen space, 4 taps, off by default | shadow-map march, 8 exponential steps (to 256 blocks) | shadow-map march at half resolution, ≤25 steps |
| Shadow filtering | 9-tap PCF | PCSS-like: 4 blocker samples + 13-tap PCF + 16-step contact shadows | 4 rotated sample pairs + hardware PCF | static (skylight threshold); dynamic shadows off by default | 4 taps rotated by blue noise | PCSS (Vogel 3 + 6–12) + screen-space traced shadows past the shadow distance |
| AO | SSAO, 8 taps | SSAO, 7 samples | SSAO, 8 taps | SSAO, off by default | SSAO, 8 taps | GTAO at half resolution, temporal |
| Coloured light | floodfill, off by default | floodfill, off by default | floodfill, off by default (forced off on Mac) | floodfill or screen space, off by default | floodfill (forced off on Mac) | floodfill, off by default; Photonics ray-tracing hooks |
| Tonemap | custom Reinhard-like with highlight desaturation | AgX minimal (Ronald's own settings: Hejl 2015) | modified Lottes | Uchimura | Hable / Uncharted 2 | Lottes |
| Bloom | 7 levels, mixed in at 0.2 | 7 levels, added, plus "bloomy fog" | 7 levels, mixed in at 0.12, plus "bloom fog" | 3 tiny tiles, mixed in | 5 levels, mixed in at about 0.15 | 6 mips, mixed in at 0.12 |
| TAA | yes, after FXAA | yes | yes, then FXAA | off | yes | yes, then FXAA |
| Distant Horizons / Voxy | both, dithered ring | Distant Horizons only, hard cut | both, dithered fade | both, dithered (Distant Horizons) | both, dithered (Distant Horizons) | both, dithered (interleaved gradient noise) |

---

## BSL v10.1.8

BSL is old-school and cheap. It uses no lookup tables. Every look is a slider-set RGB colour, kept as its square root and squared before it is used.

**Pass layout:**
- **gbuffers:** forward lighting through `shaders/lib/lighting/forwardLighting.glsl`.
- **deferred:** SSAO.
- **deferred1:** AO blur, fog, the deferred sky and the volumetric clouds.
- **composite:** light shafts.
- **composite1:** the shafts blurred with 4 taps and added.
- **composite4 and 5:** the bloom atlas, then mix, exposure, grade and tonemap.
- **composite6 and 7:** FXAA, then TAA.
- **final:** sharpening.

### Sky (`shaders/lib/atmospherics/sky.glsl`, `lib/color/lightColor.glsl`, `lib/color/skyColor.glsl`)
- **Model:** an analytic exponential gradient, `exp(-(1-(1-VoU)^curve)/0.35)`. The sky colour (96,160,255) is compressed by `x/sqrt(x²+1)`.
- **Sunset:**
  - The warm light colour (255,160,80)×1.2 is raised to the power `4-sunSkyVisibility`.
  - It is blended in by `sunMix`, which is `((VoL*.5+.5)*(1-VoU))^(2-vis)` times a factor that fades out at noon.
  - Raising the colour to a power is what pushes it to deep orange-red at the horizon. It costs almost nothing.
- **Stars:** three hashed noises multiplied together on a 1024 grid, thresholded at 0.8125.
- **Sun and moon:** vanilla textures by default. An optional disc, size 0.007, sits in `lib/atmospherics/sunmoon.glsl`.
- **Aurora:** off by default. When on, 24 planar layers of `noisetex.b`.

### Clouds (`lib/atmospherics/clouds.glsl`)
- **Shape:** one volumetric slab at `cloudHeight`, 60 blocks thick.
- **Steps:** at most 32, each about 30 blocks long. The start is a Bayer offset that shifts by the golden ratio every frame, and TAA resolves the noise.
- **Noise:** only the 512² `tex/noise.png`. Perlin in R, Worley in G, "blocky" in A, and the B channel faked into 3D by blending slices.
- **Lighting:**
  - No march toward the light and no Beer's law.
  - `heightGradient^(…)` times `(1-noise^k)`, which acts like a powder term.
  - A forward lobe `pow(halfVoL,6)`.
- **Cloud shadows** (`lib/lighting/shadows.glsl` `GetCloudShadow`): one projected noise sample. Off by default.

### Fog and volumetrics (`lib/atmospherics/fog.glsl`, `lightShafts.glsl`, `weatherDensity.glsl`)
- **Distance fog:** `1-exp(-2*fog^(…))`.
  - ×4 at night (`FOG_DENSITY_NIGHT`); this fades out at sunrise and is the only morning fog.
  - ×1.5 in rain.
  - Per-biome weights (cold, dry, damp) through custom biome uniforms in `shaders.properties`.
- **Height fog:** `exp2(-max(y-62,0)/128)`.
- **God rays** (composite): 7 samples at `exp2(i+dither)-0.95`, reaching 128 blocks, stored as `pow(vl/32,0.25)`, then blurred with 4 taps. Strength is 0.25 in the morning, 0.10 by day and 0.5 at night.

### Lighting
- **Shadows:**
  - Map 2048², distance 256.
  - Distortion `distb*bias+(1-bias)` with bias = 1−25.6/distance; z ×0.2.
  - 9-tap PCF.
  - Coloured shadows through stained glass.
  - A 12-tap subsurface blur on leaves.
- **AO:** SSAO with 4 directions × 2 taps, blurred with 4 depth-aware taps. Vanilla AO is kept too.
- **Combined light:**
  - `mix(ambient*sky, light, shadow*NoL)*sky²`.
  - Block light (255,212,160)×0.85, squared.
  - The held light is merged with a smooth max.
- **Coloured light:** a 256³ floodfill in a compute pass. Off by default.
- **Nights:** dark and blue. Night light is (96,192,255)×0.3, scaled by moon phase and squared. Low-light desaturation (`DESATURATION` 1.5) pulls dark colours toward blue.
- **Exposure:** fixed ×4. Auto exposure is off by default.

### Water (`program/gbuffers_water.glsl`, `lib/reflections/*`, `lib/atmospherics/waterFog.glsl`)
- **Waves:** procedural Worley from the noise texture at scales /256 and /48, with a 4-step parallax and finite-difference normals.
- **Reflections:** screen-space, 30 steps plus 4 refinements. The fallback is sky + clouds + aurora, scaled by skylight².
- **Fresnel and sun:** Schlick fresnel with 0.02 base, and a GGX sun highlight.
- **Off by default:** refraction and caustics. The caustics, when on, are drawn in the shadow pass.
- **Foam:** not found.

### Weather
- **Rain:**
  - Light colours are desaturated toward a weather colour per biome.
  - Direct light ×(1−0.95·rain).
  - Clouds get more coverage and turn 40% darker.
- **Puddles and ripples:** these exist (`lib/reflections/rainPuddles.glsl`) but only with ADVANCED_MATERIALS, which is off by default.
- **Lightning:** only the bolt is recoloured. A sky flash is not found.

### Distant Horizons and Voxy
- **Distant Horizons:** an overlap ring dithered with Bayer8.
  - Vanilla terrain is discarded beyond `far+(dither-0.75)*16`.
  - Distant Horizons terrain is discarded inside that.
- **Voxy:** depth is merged in deferred1, with no dither.
- **On LOD terrain:**
  - LOD is lit by the same lighting function, with blue-noise albedo variation.
  - SSAO at a larger radius.
  - Fog ×FOG_DENSITY_LOD.
  - **Shadows:** a 16-step screen-space march along the light against the LOD depth; LOD has no shadow map.
  - Clouds reach out to the LOD distance.
  - LOD water gets screen-space reflections.

### Post (`program/composite4.glsl`, `composite5.glsl`, `composite7.glsl`, `final.glsl`)
- **Tonemap:** custom "BSLTonemap".
  - Exposure ×4.
  - A crosstalk matrix (0.03/0.01) so highlights drift toward white.
  - `x/sqrt(x²+1)` with white curve 2, then the inverse matrix.
  - A Reinhard-like curve, not ACES or AgX.
- **Bloom:** 7 levels in one atlas, each a 36-tap Gaussian, weighted 4…1, then `mix(color, bloom, 0.2)`. It is mixed in, not added.
- **Anti-aliasing:** FXAA, then TAA. TAA uses 8 jitter points, Catmull-Rom history and YCoCg clipping, with history weight 0.7–0.9. Then a 4-tap sharpen at 0.125.

### What BSL does well
Art direction through a handful of squared colour sliders. Sunsets from raising a colour to a power. Per-biome weather tints. Seven-sample god rays that look good.

---

## Bliss v2.1.2 (Chocapic13 edit): the look ours is modelled on

**Ronald's own settings differ from the stock defaults** (`~/Coding/chiacraft-archive/references/shaders/bliss_v2.1.1_ronald_settings.txt`, v2.1.1):

| Setting | Ronald | Default |
|---|---|---|
| `TONEMAP` | `ToneMap_Hejl2015` | `ToneMap_AgX_minimal` |
| `SATURATION` | 0.3 | 0 |
| `Sun_temp` | 4300 | 6300 |
| `sun_illuminance` | 250000 | 128000 |
| `moon_illuminance` | 200 | 50 |

His other settings:
- `Haze_amount` 1.25, `RainFog_amount` 0.3, `BLOOMY_FOG` 1.25.
- VL at full resolution.
- `SHARPENING` 0.4.
- `Daily_Weather=false`.

**Where the code is:** `shaders/world0/*` include `shaders/dimensions/*`, with the numbers shifted by one. Settings are in `shaders/lib/settings.glsl`.

**Pass layout:**
- **deferred:** the 256² sky map and a 256² sky+clouds+fog map in `colortex4`; light colours and exposure go into single pixels.
- **deferred2:** clouds at half resolution.
- **composite:** fog behind translucents.
- **composite1:** SSAO and the PCSS blocker search.
- **composite2:** the main deferred lighting.
- **composite3:** VL fog at half resolution (full resolution for Ronald).
- **composite4:** bilateral upsample, refraction, blend.
- **composite6:** TAA.
- **composite7–11:** bloom.
- **composite12:** DOF, bloom, exposure, Purkinje shift, tonemap.
- **composite13:** sharpening and grade.

### Sky (`shaders/lib/ROBOBO_sky.glsl`, `lib/sky_gradient.glsl`, `lib/stars.glsl`)
- **Model:** a real single-scattering raymarch (Rayleigh, Mie and ozone). There is no Bruneton or Hillaire lookup chain.
  - Planet radius 6731 km, atmosphere 110 km, scale heights 8 km and 1.2 km.
  - Rayleigh (5.8e-6, 1.35e-5, 3.31e-5); Mie 3e-6 with g 0.8; ozone absorption.
  - 10 view steps and 4 light steps per texel.
- **Cost trick:** the 256×256 equirectangular map is jittered and accumulated over frames at 6% per frame. Screen pixels only read it.
- **Sunset:** physical, plus two Bliss tweaks.
  - Sun colour = `blackbody(Sun_temp)*illuminance*transmittance`.
  - In-scattered light uses a shortened, raised sun path (`sunVector*0.5+0.1`), so less of the glow is absorbed at low sun. The twilight comes out brighter and more saturated.
  - A fake multiple-scattering ambient is scaled by `low_sun` (1 to 1.8).
  - Ozone gives the blue-violet band.
- **Sun disc:** about 0.96–1.71°, raised to the power 3, so it reads big.
- **Moon:** a fake sphere lit by phase; its light is `blackbody(15000K)·50`.
- **Stars:** hashed value noise at 300× scale, threshold 0.99.
- **Aurora:** not found.

### Clouds (`lib/volumetricClouds.glsl`)
- **Layers:**
  - L0 cumulus at 250–350 m.
  - L1 cumulus at 500–600 m.
  - L2 altostratus as a flat sheet at 2000 m.
- **Steps:** 15 per layer (`HQ_CLOUDS`), at half resolution, upsampled bicubically.
- **Noise:** only textures. `texture/noises.png` (512²) for coverage, in two octaves. 3D value noise packed into 2D through a y-slice offset of 193 gives the detail; the erosion is `cov - noise⁴*0.5`.
- **Lighting** (`DoCloudLighting`):
  - 3 shadow taps toward the sun per step, at 20, 40–60 and 60–100 m.
  - The upper layers shadow the lower ones.
  - Direct light = `sunScatter*exp(-10·od) + sunMultiScatter*exp(-3·od)*powder`, with powder `1-exp(-10·density)`.
  - Phase: two forward Henyey-Greenstein lobes (0.85 and 0.75). The multiple-scattering term adds a weaker back lobe.
  - Ambient ramps from bottom to top.
  - Distant clouds pass through Rayleigh extinction, so they redden and fade near the horizon.
- **Cloud shadows** (`GetCloudShadow`): one coverage sample per layer, `exp2(-100·s²)`. Applied to direct light, subsurface scattering and **every VL fog step**.
- **Daily weather:** 10 day profiles chosen by `worldDay%10` (`DAY0..9`) set coverage, density and fog per layer. In rain, L0 goes to coverage 0.95 and density 1.

### Fog and volumetrics (`lib/overworld_fog.glsl`, `lib/climate_settings.glsl`)
- **The march:** one shadow-space march, `VL_SAMPLES` 8 exponentially spaced steps (expFactor 11) out to `far`, with bilateral upsampling.
- **Two media in the same march:**
  - **Fog:**
    - A height gradient `exp2(-0.15*(y-63))`, scaled by a density for the time of day.
    - Cloudy noise fog that thins with height (`exp(-0.05*(y-83))`).
    - Rain fog.
  - **Atmospheric haze:**
    - Uses the sky's Rayleigh and Mie coefficients, with a height falloff from `SEA_LEVEL` 70, ×24×1.5×`Haze_amount`.
    - Lit with the Rayleigh phase plus a sharp forward lobe ×5, and shadow-mapped.
    - **This gives the warm glow on distant terrain toward a low sun.**
- **Time of day** (`FogDensities`, uniform/cloudy):

| Time | Uniform | Cloudy |
|---|---|---|
| Morning | 0.01 | 0.05 |
| Noon | 0 | 0 |
| Evening | 0.05 | 0 |
| Night | 0.2 | 0.05 |

- **Per biome** (60 s transitions):
  - Swamp: 0.3 density, colour (0.15,0.25,0.05).
  - Jungle: denser.
  - Dark forest: tinted blue.
  - Sand and snow storms: scrolling noise.
- **Bloomy fog** (`dimensions/composite11.fsh:152`): `col = mix(col, bloom, (1-fogTransmittance)*BLOOMY_FOG*0.75*(1+rain))`. Fogged areas show the blurred bloom image instead of the sharp scene, which gives the soft, glowing haze.

### Lighting (`dimensions/composite1.fsh`, `lib/Shadow_Params.glsl`, `lib/PhotonGTAO.glsl`)
- **Shadows:**
  - Distortion `log(len*b+a)*k`, k 1.8; sun path rotation −35.
  - 4 blocker-search samples, then a 13-tap PCF with radius 5–30 scaled by blocker depth.
  - 16-step screen-space contact shadows.
  - Beyond the shadow distance, a fallback from the lightmap.
- **AO:** SSAO with 7 samples, combined with vanilla AO through power curves. GTAO, SSGI and RTAO are options.
- **Ambient light:** 9 samples of the sky+cloud map, so the ambient colour always matches the sky.
- **Torches:**
  - Colour (1.0, 0.75, 0.65).
  - Torches are boosted up to ×30 and emissives up to ×100 when exposure is low, so they still read in daylight.
- **Coloured light:** a 128³ floodfill. Off by default.
- **Nights:**
  - Auto exposure: 50 samples from the 1/32 bloom level, a geometric mean, 95% history per frame. `exposure = 0.18/log2(L*2.5+1.045)*0.62`. It only partly compensates, so nights stay dark.
  - **Purkinje shift** (composite11): blends toward `lum*(0.4,0.7,1.0)*5` in low light. Nights read blue rather than black.

### Water (`lib/waterBump.glsl`, `dimensions/all_translucent.fsh`)
- **Waves:** a procedural height field from 3 texture-noise octaves (48×12, 12×48, 32×32), rotated by the golden angle, modulated by large-scale noise. The finite-difference step grows with distance. Parallax.
- **Reflections:**
  - Screen-space, 15–30 steps, reading the previous TAA frame.
  - The fallback is the sky/cloud/fog map, or the lightmap indoors.
  - GGX sun highlight and a Snell's window.
- **Colour:** water has no albedo at all. Colour is pure absorption `(0.263, 0.0565, 0.0101)` plus a scattering "dirt" term.
- **Caustics:** noise projected along the sun.
- **Foam:** not found.

### Weather
- **Puddles:** noise-shaped, upward faces only, open sky, not in dry biomes. They darken by porosity.
- **Not found:** ripples and snow cover.
- **Lightning** (`lib/lightning_stuff.glsl`): a point light (77,153,255) with a 300 m falloff, lighting surfaces, fog **and** clouds.

### Distant Horizons and Voxy
- **Distant Horizons:** the terrain writes the same gbuffers, so deferred lighting, SSAO, fog and clouds all apply to it.
  - The overlap is a **hard discard** (inside far−32 for solids, far−64 for water), not a dither.
  - LOD gets contact shadows (step ×6), cloud shadows and per-block procedural noise.
- **Voxy:** not found.

### Post
- **Tonemap:** AgX minimal by default, with an `agxLook` saturation of 1.25 that adds to the vividness. Ronald's profile and "OLD_BLISS" use **Hejl 2015** (`hdr*0.85`, white 3.0).
- **Grade:** contrast-adaptive sharpening 0.35, then saturation `c+(c-luma)*SATURATION`.
- **Bloom:**
  - 7 levels from 1/4 to 1/256.
  - Weights 1, 1/2, 1/3, 1/5.5, 1/8, 1/10, 1/12.
  - **No threshold.** Added at `0.5·strength·exposure^0.2`.
- **TAA:** blend 0.12, with closest-depth velocity.

### What Bliss does well
- One sky map that feeds sky, ambient, reflections and clouds, so all the colours agree.
- Haze that shares the sky's coefficients and is shadow-mapped, which gives the warm distance glow.
- Bloomy fog.
- Clouds with powder and multiple scattering.
- Morning and night fog tables.
- Purkinje nights.

---

## Complementary Reimagined and Complementary Unbound r5.9.3

**The two packs are the same code.** `diff -rq` finds one differing file, `shaders/lib/common.glsl`, and one line in it: `SHADER_STYLE` is 1 in Reimagined and 4 in Unbound. That switches these defaults:

| Setting | Reimagined | Unbound |
|---|---|---|
| WATER_STYLE | 1 | 3 |
| CLOUD_STYLE | 1 | 3 |
| AURORA_STYLE | 1 | 2 |
| SUN_MOON_STYLE | 1 | 2 |
| RAINBOW_STYLE | 1 | 4 |
| sunPathRotation | 0 | −40 |
| Glowing ores | off | on |

**Lighting is forward.** `DoLighting` runs in the gbuffers passes.

**Pass layout:**
- **deferred1:** SSAO, fog, LOD shadows, aurora, clouds.
- **composite:** reflections at half resolution.
- **composite1:** refraction and the light shafts. **This is the first place colour becomes linear (`pow 2.2`).**
- **composite4:** the bloom atlas.
- **composite5:** tonemap.
- **composite6:** TAA.
- **composite7:** FXAA.
- **final:** sharpening.

**The lighting is mixed in a gamma-like space:** `sqrt(pow2(dirShade*AO)*(block+pow2(scene)+minLight)+emission²)` (`shaders/lib/lighting/mainLighting.glsl`). This is a large part of the clean, soft "Complementary" look.

### Sky (`lib/atmospherics/sky.glsl`, `lib/colors/skyColors.glsl`)
- **Model:** an analytic three-colour gradient (up, middle, down) built from vanilla's `skyColor` uniform, so biome sky colours carry through. Noon up is `skyColor^2.9*(0.85,0.92,0.81)`.
- **"Fake scattering" toward the ground:** separate curves per channel (r², √g, ∛b).
- **Sunset band:** `(1.45,0.86,0.5)` weighted by `pow2(1-|VdotU|)` and `(0.7+VdotS·0.3)`.
- **Sun glare:** a hyperbolic lobe, `v/(1-(1-v)*VdotS^k)-v` with v 0.075.
- **Gradient dither:** 1/128 against banding.
- **Sun and moon:**
  - Unbound: a procedural sun disc, and a moon with 3-octave crater noise.
  - Reimagined: recolours the vanilla sun texture.
- **Stars:** 1024 cells, threshold 0.7.
- **Aurora:** 25 samples; blocky in Reimagined, smooth in Unbound. By default only in snowy biomes at full moon.
- **Rainbow:** analytic, after rain.

### Clouds (`lib/atmospherics/clouds/*`)
**Reimagined** (`reimaginedClouds.glsl`). Our clouds borrowed this idea.
- **Density:** a binary test on the blue channel of `textures/cloud-water.png` (256²). The cells are rounded with SixthSurge's rounded-cell function (roundness 0.125).
- **Slab:** ±4.2 blocks, with a `pow8` vertical threshold.
- **Steps:** the march stops at the **first hit**. `planeDistance/8` steps, at least 12.
- **Lighting:**
  - Height shading.
  - **2 jittered taps of the 2D map toward the light**: `light = 2 - s1 - s2`.
  - A forward boost `VdotS*0.25`.
  - Clouds darken to 0.25 where the terrain's shadow map covers them.
- **Not used:** Beer's law and powder.

**Unbound** (`unboundClouds.glsl`)
- **Density:** `Noise3D`, a pseudo-3D lookup into the 128² noise.png with 128 z-slices. 4 octaves at ×3 frequency each, squared. Base density 0.8, +0.4×rain.
- **Steps:** fixed 32-block steps; the march exits at alpha 0.9.
- **Lighting:** by height only, with no march toward the light.

**Cloud shadows** (`lib/lighting/cloudShadows.glsl`): off by default.

### Fog and volumetrics (`lib/atmospherics/fog/mainFog.glsl`)
- **Cave fog:** below y 62, colour (0.13,0.13,0.15).
- **Atmospheric height fog:** `1-exp(-pow(d·(0.001-0.0007·rain), 2-rain²)·d·192/far)`. Full below y 63, fading over 60 blocks above it.
- **Border fog:** `1-exp(-3·(d/far)^16)` toward `GetSky()`, so terrain melts into the sky.
- **Biome storm fog:** sandstorm or snowstorm fog in dry or snowy biomes during rain.
- **Morning fog:** not found.
- **Bloom fog** (`fog/bloomFog.glsl`):
  - composite1 multiplies the colour by `1+fog·mult`.
  - The bloom pass blurs it.
  - composite5 divides it back out.
  - The result is a haze glow that grows with distance. mult is 8 in rain, 3 at night and 14 in caves and underwater.
- **Light shafts** (`volumetricLight/volumetricLight.glsl`):
  - A world-space shadow-map march with 10 near samples plus 1 far sample, spaced by t².
  - Stained glass tints them.
  - Weakest at noon, strongest at sunrise and sunset. The noon colour is a separate bluish (0.4,0.75,1.3).
  - **Scene-aware:** every ~66 ms, 25 shadow-map texels plus a stored height decide whether the player is under canopy, and the shaft factor ramps up or down by 1/255.

### Lighting (`lib/lighting/*`)
- **Shadows:**
  - Hardware PCF plus a rotated disc (2/4/8/16 pairs by quality) using interleaved gradient noise rotated by the golden ratio each frame.
  - Not PCSS. A variable penumbra is faked from the gradient of vanilla AO; in rain the radius grows 2–3×.
  - Normal bias `(0.12+0.0008·d^1.5)`. Positions are pulled toward the block centre in caves to stop light leaks.
- **SSAO:** 8 taps. Vanilla AO is reshaped.
- **Colours:**
  - Hand-tuned tables in `lib/colors/lightAndAmbientColors.glsl`. Noon sun `(0.65,0.55,0.375)*2.05`.
  - Block light `(0.1775,0.104,0.077)`, suppressed in daylight.
- **Face shading:** directional, top 1.0 and sides 0.75–0.9, plus fake bounce light.
- **Coloured light "ACT":**
  - The shadow-pass vertex shader `imageStore`s block IDs into an r16ui volume.
  - A compute pass spreads it: 6 neighbours ÷ 6.42, ping-pong rgba16f, reprojected.
  - **Off by default and forced off on macOS.**
- **Nights:** moderately dark. Night ambient `0.9·(0.09,0.12,0.17)·(1.55+…)`. Cave minimum 0.0056.
- **Exposure:** no auto exposure. The smoothed eye sky-light fakes adaptation.

### Water (`lib/materials/specificMaterials/translucents/water.glsl`)
- **Reimagined:** the vanilla texture squared × biome tint.
- **Unbound:**
  - A flat `0.375×tint`.
  - Normals from the RG channels of `cloud-water.png` in 4 octaves.
  - 4-iteration parallax from the alpha channel.
- **Reflections:** screen-space, 30 steps (stride ×2, 6 refinements), reading the deferred1 scene. The fallback is sky + re-marched clouds + aurora, scaled by the sky lightmap.
- **Refraction:** a screen-space offset, accepted only if the target pixel is also water.
- **Foam:** where the surface-to-floor gap is small.
- **Caustics:** baked into the shadow colour.

### Weather
- **Rain changes:**
  - The sky: flatter and desaturated.
  - The light colour.
  - Shadows: softer.
  - Fog and bloom fog: more.
  - Unbound clouds: +0.4 density.
- **Puddles:** off by default.
- **Wet darkening:** not found.
- **IMPROVED_RAIN:** relights the rain streaks in composite.
- **Lightning:** only the bolt. A world flash is not found.

### Distant Horizons and Voxy
- **Distant Horizons:**
  - LOD fades in with a dithered alpha `smoothstep(far·0.4, far·0.6)`.
  - Normal terrain dithers out with `exp(-3(d/far)^16)`.
  - LOD colours get a procedural noise (4 cells per block).
  - **LOD shadows:** 32 screen-space steps of 2.5 blocks along the light through `dhDepthTex`, giving `colour *= 0.5+0.5·shadow`.
  - Fog switches to its LOD formula.
- **Voxy:** the LOD shadow is reprojected from the previous frame.

### Post (`program/composite4-7`, `final`)
- **Bloom:** 7 tiles, each a 7×7 binomial blur, **mixed in at 0.12**.
- **Tonemap:** a modified Lottes 2016 curve.
  - Contrast 1.05, hdrMax 8, mid 0.25 maps to 0.25.
  - A dark lift: 75% untonemapped below luminance 0.1.
  - A path to white.
  - Dark colours desaturated by 25%.
  - Saturation +0.07 after. (Our day curve copies these numbers.)
- **TAA:** after the tonemap, on LDR, blend 0.35–0.7.
- **Then:** FXAA 3.11, then a 4-tap sharpen.

### What Complementary does well
- One `GetSky()` reused for the sky, fog, the clouds' far fade and reflections.
- Art-directed colour tables.
- Gamma-space mixing.
- Hand-coded per-block "integrated PBR".
- Bloom fog.
- Scene-aware shafts.
- Distant light bokeh.
- End flashes and the Nether storm.

---

## Mellow v3.5

`program/*.fsh` are already-inlined copies. The readable sources are in `shaders/global/` and `shaders/lib/`. The pack is forward-lit, cheap and hand-tuned.

**Pass layout:**
- **gbuffers:** all lighting.
- **deferred1:** sky, clouds, SSAO, fog, godrays.
- **composite2–7:** bloom.
- **composite8:** TAA, exposure, Purkinje shift, tonemap, LUT.
- **composite9–11:** SMAA.
- **final:** CAS sharpening and the grade.

### Sky (`global/atmosphere/sky.glsl`, `global/lighting/light_colors.vsh`, `lib/colors.glsl`)
- **Model:** two colours, `mix(SKY_TOP, SKY_GROUND+glare, fogify(upDot+0.1, 0.03))`, with `fogify = w/(x²+w)`.
- **Keyframes:** sunrise, noon, sunset and night, through custom uniforms. Six colour-scheme presets.
- **Sunset:** an orange glare `(0.8,0.45,0)·(VdotL·.5+.5)²·(sunset+sunrise)²` added at the horizon.
- **Stars:** 512 cells.
- **Aurora:** 2 samples, only in snowy biomes.
- **Rainbow:** analytic, after rain.

### Clouds (`global/atmosphere/clouds.glsl`)
- **Default "Flat":** a 2D noise layer projected onto a dome.
  - Noise from `img/cloud_noise+normal.png` (256²), 2 octaves.
  - Self-shadow from one sample offset toward the sun.
  - A Cornette-Shanks-like phase with g 0.7.
  - **Animation speed is 0.**
- **"Volumetric" option:**
  - At y 1000–1500, ≤12 steps.
  - Detail eroded with the 64³ `img/worley_perlin.bin`.
  - Beer's law with a multiple-scattering term `fms/(1-fms)`.
- **"Blocky" option:** a texel DDA through `clouds.png`, 16 steps, with 6 steps toward the sun.
- **Cloud shadows:** not found.

### Fog and volumetrics (`global/atmosphere/fog.glsl`)
- **Single scattering with Beer's law:** absorption 0.0025, scattering 0.02, `T = exp(-0.0225·min(d,128)·density)`. The density falls off between y 60 and 100, so it sits in valleys.
- **In-scatter:** sun colour × (phase g 0.6 + multiple scattering) + `SKY_GROUND/π`.
- **One uniform for all of it:** `fogAmount = sunrise + night + 1 + rain + 30·underwater`. Fog roughly doubles at dawn and at night.
- **Godrays:** screen space, 4 taps. Off by default.

### Lighting
- **Dynamic shadows: off by default.** The "static shadow" `smoothstep(0.85, 0.96, skylight)` cuts the sun wherever skylight is below about 0.9. **This is a big part of the soft, mellow look.**
- **Ambient:** 60% grey, 40% the sky gradient picked by the normal's up-ness.
- **When dynamic shadows are on:** 512 px over 96 blocks, 4-tap or 8-tap Vogel PCF. PCSS not found.
- **Nights:** moon about a tenth of the sun. Readable, blue-grey.
- **Exposure:** a fixed 1.7.

### Water (`global/water.glsl`)
- **Waves:** a vertex sine wave (amplitude 0.1) plus two scrolling normal textures.
- **Reflections:** the default mode 3 is **"Flipped Image"**. One screen sample at `ViewPos + R·(50+far/4)` from the previous frame. Very cheap.
- **Not found:** refraction, caustics, foam.

### Weather
- **Rain particles:** desaturated and tilted.
- **Sky:** desaturated 0.5 and darkened 0.33.
- **Puddles:** a noise mask on upward faces.
- **Lightning:** adds white ambient light within about 100 blocks.

### Distant Horizons and Voxy
- **Distant Horizons:** a dithered handover at `far−16`.
- **On LOD:**
  - Static shadows.
  - Blocky 1/8-block albedo noise.
  - SSAO forced on at 2× radius.
- **Voxy:** supported, with no dither.

### Post (`lib/tonemap_operators.glsl`)
- **Tonemap:** **Uchimura** (`TONEMAP_OPERATOR 6`; P 1, a 1, m 0.22, l 0.4, c 1.33). ACES, Reinhard-Jodie, Hejl, Lottes, AgX Punchy and Khronos Neutral are also offered.
- **Grade:** vibrance 1.2, optional 512² LUTs, optional Purkinje.
- **Bloom:** three tiny tiles of 128, 64 and 32 px, with 13-tap downsamples. Mixed in by luminance at about 0.4. This gives a soft overall haze.
- **Off by default:** TAA and SMAA.
- **CAS:** 0.2.

### What Mellow does well
A coherent palette. Ambient from the sky gradient. A soft look with no shadow map. Haze-style bloom with Uchimura and vibrance. Seasons: an HSV shift of foliage over a 120-day cycle.

---

## Solas V3.9

**Lighting is forward:** one function, `lib/lighting/gbuffersLighting.glsl`. Paths in this section are relative to `shaders/`.

**Pass layout:**
- **deferred1:** sky, clouds, aurora, SSAO, fog.
- **composite1:** VL.
- **composite3:** refraction.
- **composite13 and 14:** bloom and tonemap.
- **composite15:** TAA.
- **final:** sharpening.

### Sky (`lib/atmosphere/sky.glsl`, `lib/color/lightColor.glsl`, `lib/atmosphere/sunMoon.glsl`)
- **Model:** analytic. The normalised vanilla `skyColor`, × a warm tint by time of day, × a colour per biome (desert, cherry, swamp, mushroom, jungle), × a rain colour.
- **Sunset gradient:** `pow(1-y, 3-1.5·VoS)·(0.5-0.3·√timeBrightness)·0.6`, plus a **narrow green band** at twilight, `exp(-(y-0.1)²/0.025)·0.2`.
- **Light colours:** six keyframes (sunrise, morning, day, evening, sunset, night) for both direct and ambient light.
- **Sun:** `max(VoS^1024-0.4,0)·16`.
- **Stars:** 2 hashed layers, plus shooting stars, a Milky Way texture (`tex/milkyWay.png`, 512×1024) and a 3-octave nebula.
- **Aurora** (`lib/atmosphere/aurora.glsl`): volumetric, 8+8·kp samples. A "kp index" from `worldDay` sets its strength. It tints the clouds and the scene light too.

### Clouds (`lib/atmosphere/volumetricClouds.glsl`, `planarClouds.glsl`, `lib/lighting/cloudShadows.glsl`)
- **Volumetric layer:**
  - At 190, about 113 blocks thick.
  - ≤28 steps of 48 blocks, jittered with blue noise shifted by the golden ratio.
  - Only the 512² `tex/noise.png`, with slices faked into 3D.
- **Lighting:**
  - **One sample toward the light per step.**
  - Beer `exp(-lightNoise·(4-T))`, powder `1-exp(-n⁴·0.75)`, forward lobe `pow8(halfVoL)`.
  - The final colour is pushed 40% toward the atmosphere colour, a cheap aerial perspective.
- **Dynamic weather:** a sum of sines over `worldDay` changes amount, height (±75), scale and thickness each day.
- **Indoors:** the march stops on the shadow map, so clouds don't show through ceilings.
- **High 2D layer:** one offset light sample.
- **Cloud shadows:** the density sampled at 4 heights; `exp(-2n)` on the ground and **`exp(-3.5n)` inside VL, which gives the crepuscular rays.**

### Fog and volumetrics (`lib/atmosphere/fog.glsl`, `lib/atmosphere/volumetrics.glsl`)
- **Fog:** `1-exp(-0.005·d·k)`.
  - Denser at sunrise, sunset and night.
  - Biome additions: desert +0.10, swamp +0.15, jungle +0.25. Rain +0.25.
  - **Height falloff around y = 90 + noise·10 + T·25, which acts as morning valley fog.**
- **Far fade:** to the atmosphere colour as `pow(d/far, 4)`.
- **VL:**
  - 8 samples, +4 at sunrise and sunset, +3 in rain.
  - Steps double in length, so they reach about 256 blocks.
  - Cloud shadow at every step.
  - A "closed space" boost from 9 depth taps plus eye brightness, stronger indoors and in forests.

### Lighting
- **Shadows** (`lib/lighting/shadows.glsl`):
  - 2048 map, distance 192, distortion bias 0.867.
  - **Only 4 taps** rotated by blue noise.
  - Light-leak fix pulls positions toward block centres.
- **Screen-space shadows** (`lib/lighting/screenSpaceShadows.glsl`): 16 steps ×√2, on LOD only.
- **SSAO:** 8 taps, blurred with 4 taps.
- **Block light:** (255,195,135)².
- **Floodfill** (`shadowcomp.csh`, `lib/vx/*`):
  - 192×96×192, one step per frame, ÷7.01.
  - Sources write `pow(colour, 1.6)` and surfaces read `pow(·, 1/1.6)` to stretch the range.
  - Also used as LPV fog. **Forced off on Mac (`MC_OS_MAC`).**
- **Nights:** moon about (0.01,0.03,0.08) and ambient about (0.003,0.010,0.027): blue and fairly dark.
- **Auto exposure:** not found.

### Water (`lib/water/waterNormals.glsl`, `lib/pbr/waterReflection.glsl`)
- **Waves:** 2 noise taps (/256, /48) moving in opposite directions, with 4-step parallax.
- **Colour by depth:** absorption in which red falls off first.
- **Reflections:** screen-space, 30 steps plus 6 refinements, from a scene copy that includes sky and clouds.
- **Refraction:** in composite3.
- **Caustics:** the curvature of the height field ×48, put into `shadowcolor0`.
- **Foam:** not found.

### Weather
- **Rain:**
  - Sky ×weather colour.
  - Direct light ×(1−0.75w).
  - More cloud coverage, fog +0.25.
  - Puddles (`lib/pbr/rainPuddles.glsl`) with cell ripples.
- **Rainbow:** when `wetness > 0` and rain = 0.
- **Lightning** (`lib/lighting/lightning.glsl`): `exp(-24(1-x))` over 256 blocks.
- **Snow:** not found.

### Distant Horizons and Voxy
- **Distant Horizons:** a Bayer8 dither over a 16-block band.
- **On LOD:**
  - The fake skylight shadow and cloud shadows.
  - 3D blue-noise albedo.
  - SSAO and screen-space shadows.
  - Fog ×2.
- **Voxy:** shadow-mapped (`lib/lighting/shadowsVoxy.glsl`).

### Post (`lib/post/tonemap.glsl`, `computeBloom.glsl`, `getBloom.glsl`, `lib/antialiasing/taa.glsl`)
- **Tonemap:** Uncharted 2 / Hable with A .40, B .20, C .25, D .15, E .034, F .35, input ×3.6, white 10.
- **Bloom:** 5 tiles of 36 taps each, lerped in at about 0.15.
- **TAA:** on LDR.
- **Lens flare:** on.
- **Oddity:** a guard macro, `SOLAS_BY_SEPTONIOUS`, zeroes the sharpening and breaks the TAA jitter if it is changed. It looks like an anti-tamper check.

### What Solas does well
- Cheap, keyframed atmosphere colours.
- Clouds that change every day, with their shadows tied into the god rays.
- Exponentially spaced VL with an indoor boost.
- Night-sky extras.

---

## Photon v1.4

Photon is the most physically based of the seven.

**Pass layout:**
- **prepare:** cloud shadow map and coverage map.
- **deferred:** a 192×108 sky map.
- **deferred1:** clouds at low resolution.
- **deferred2:** cloud upscaling.
- **deferred3:** GTAO.
- **deferred4_a:** SH compute.
- **deferred4:** shading.
- **composite0:** VL at 0.5 scale.
- **composite1:** translucents.
- **composite4:** TAA and exposure.
- **composite5–18:** bloom.
- **composite19:** grade and tonemap.
- **composite21:** FXAA.
- **final:** CAS.

The working colour space is Rec.2020.

### Sky (`shaders/include/sky/atmosphere.glsl`, `sky.glsl`, `program/d0_sky_map.*`)
- **Model:** Bruneton precomputed scattering with Zombye's ν mapping.
  - The table **ships pre-baked** as `image/atmosphere/scattering.dat`, a 3D RGB16F texture of 32×64×32, at ground level only.
  - The μ axis is stretched near the horizon "to admire the sunset".
  - Transmittance is analytic (Chapman approximation); no transmittance table is used.
- **Constants:** ozone with a 4-Gaussian profile; sunlight from AM0 via CIE 2006.
- **Art-directed twilight:**
  - A saturation boost of `0.10+0.20·blue_hour`.
  - Tints for morning/evening and for the blue hour.
  - A sun-exposure "magic brightness" ×(1 + 0.5·(sunrise+sunset) + 40·blue_hour), used *instead of* auto exposure.
- **Ambient:** a compute pass takes 256 samples of the sky map into L2 spherical harmonics, evaluated with the GTAO bent normal.
- **Sun and moon:** sun disc 2°, with limb darkening. Moon 3°, procedurally textured.
- **Stars:** blackbody colours, 4500–8500 K.
- **Aurora:** 64 steps. Rare (1 day in 5 in snowy biomes); 13 colour pairs chosen per day.

### Clouds (`include/sky/clouds/*`, `include/weather/clouds.glsl`)
**Layers**, raymarched on a spherical planet at 10× block scale:

| Layer | Height | Primary steps | Light steps |
|---|---|---|---|
| Cumulus / stratocumulus / stratus | 1200 m | 40 horizon / 20 zenith | 6 (+2 ambient) |
| Altocumulus | 3200 m | 12/6 | 4 |
| Cumulus congestus (ring 30–50 km out) | base 400 m, about 12.8 km tall | 20 | 6 |
| Cirrus / cirrocumulus | 6000 m | planar | 6 |
| Noctilucent | 80 km | planar, twilight only | — |

- **Noise:**
  - The 2D noise texture sets coverage.
  - Two 64³ 3D Worley textures carve the detail (`worley_bubbly.dat`, `worley_swirley.dat`).
  - Cirrus uses curl-noise warping.
  - Coverage is precomputed into a distorted 150 km map.
- **Lighting:**
  - Beer's law with an energy-conserving integral `(1−T)/σ`.
  - **8 multiple-scattering octaves.**
  - Phase: max(Klein–Nishina, HG 0.8) mixed with a back lobe.
  - Powder `π·d/(d+0.15)`.
  - Ground bounce with albedo 0.4.
- **Aerial perspective:** a transmittance ratio between the camera and the cloud, distance doubled.
- **Temporal reconstruction:**
  - `CLOUDS_TEMPORAL_UPSCALING 4`: buffers at 1/4 resolution, each pixel refreshed every 16 frames in a checkerboard.
  - Reprojected by the cloud's apparent distance.
  - Catmull-Rom history, accumulated over up to 20 frames.
- **Cloud shadows:** a 512² map covering 256 blocks. **PCSS penumbras grow up to ×5 under cloud shadow.**
- **Crepuscular rays:** a march through the cloud shadow map, 20 steps at the horizon and 4 at the zenith.
- **Weather state machine** (`weather/core.glsl`):
  - Temperature, humidity and wind come from slow noise over the world's age (about 600 s scale), biased by biome, time of day and wetness.
  - They choose the cloud types, coverage, "wind-torn" detail, cirrus amount and the fog.

### Fog and volumetrics (`include/fog/overworld/*`, `include/weather/fog.glsl`, `program/c0_vl.fsh`)
- **Two media:**
  - "Rayleigh": starts 30 blocks above sea level, half-life 30.
  - "Mie": starts 7 above sea level, **half-life 7**, modulated by `4·noise²`. **This is the morning ground fog.**
- **Mie densities:**

| Time or weather | Density |
|---|---|
| Morning | 0.007 |
| Noon | 0.0001 |
| Evening | 0.005 |
| Rain | 0.03 |

  Humidity adds up to 8× the noon value. Rayleigh colour and density vary per biome.
- **VL:** shadow-map march at half resolution, `8+0.1·length` steps up to 25, 4 multiple-scattering octaves, HG 0.7·HG(0.5)+0.3·HG(−0.2).
- **Analytic airmass fog:** used behind translucents, over the sky map and in reflections.
- **Border fog:** `exp2(-8(d/far)^8)`.

### Lighting (`include/lighting/*`, `program/d4_deferred_shading.fsh`)
- **Shadows:**
  - Quartic distortion.
  - PCSS: 3 blocker samples, then 6–12 PCF samples with an early out.
  - **Screen-space ray-traced shadows past the shadow distance (`SHADOW_SSRT`, 10 steps).**
  - Subsurface scattering from the blocker depth.
- **AO:** GTAO with 2 slices × 3 steps at half resolution, temporal over 10 frames, plus a bent normal.
- **Block light:** (1.0,0.75,0.63)×6, with a hotspot term.
- **Coloured light:** an LPV floodfill, 128³. Off by default.
- **`photonics/`:** hooks for the external Photonics ray-tracing mod. Not in-pack ray tracing.
- **Nights:** moonlight about 1/10 of sunlight before transmittance, plus a 0.15 cave floor and a Purkinje shift (intensity 0.05) outdoors.
- **Exposure:** off by default (manual EV 0). A 32-bin histogram mode is optional.

### Water (`include/surface/water_normal.glsl`, `include/lighting/specular_lighting.glsl`, `include/misc/distant_water.glsl`)
- **Waves:** 3 octaves of Gerstner-like `sin²` waves, rotated by the golden angle.
- **Colour:** Beer absorption (0.39,0.14,0.07), adjusted by the biome water colour through −log.
- **Reflections:** screen-space, 16 steps, reading the previous frame. The fallback is the sky map, cut by `skylight^12`.
- **Refraction:** a screen-space offset.
- **Caustics:** from the area ratio of refracted positions in the shadow pass. Off by default.
- **Foam:** an "edge highlight" near shores rather than real foam.

### Weather
- **Puddles** (`include/surface/rain_puddles.glsl`): ripples, f0 0.02, darkening 0.33–0.67.
- **Biome gating:** rain, snow or sandstorm.
- **Lightning:** only brightens the clouds.

### Distant Horizons and Voxy (`include/misc/lod_mod_support.glsl`)
- **LOD terrain** writes the same gbuffer and is fully deferred-shaded.
- **Overlap:** an interleaved-gradient-noise dither between far−32 and far−16.
- **Depth:** a combined depth buffer for GTAO, screen-space reflections and screen-space traced shadows.
- **On LOD:** screen-space traced shadows plus cloud shadows, and LOD water uses the same absorption model.

### Post (`include/post_processing/tonemap_operators.glsl`)
- **Tonemap:** Lottes with contrast 1.5, shoulder 0.91, white 8, mid-in 0.26 mapped to 0.32.
- **Grade:** HSL hue-isolated saturation (teal +0.10), a gain S-curve, white balance.
- **Bloom:** 6 mips, a 13-tap downsample and tent upsample, mixed in at 0.12.
- **TAA:** variance clipping, R2 jitter.
- **Sharpening and AA:** FXAA, then CAS 0.5.

### What Photon does well
- A weather state machine that varies the clouds and fog without rain.
- Layered physically based clouds made affordable by 1/16-per-frame upscaling.
- One sky map that feeds reflections, SH ambient and border fog.
- 7-block Mie ground fog.
- PCSS that widens under cloud shadow.
- Screen-space traced shadows on LOD.

---

## Patterns across the packs

1. **Lighting is forward in five of the six** (BSL, Complementary, Mellow, Solas and mostly Bliss for translucents). Only Photon, and Bliss for opaque surfaces, defer the lighting. Our forward pack is the norm, not a handicap.
2. **Every sky except Photon's and Bliss's is an art-directed analytic gradient.** Bliss's physical sky is a cheap single-scattering raymarch into a 256² map accumulated over frames, not a lookup-table chain.
3. **Low step counts plus a dither that moves every frame plus TAA** are how every pack makes clouds and god rays cheap: 7–10 god-ray steps, 15–32 cloud steps. Without TAA we get no temporal resolve, so our dithers must be stable or blurred.
4. **Bloom is a 5–7 level mip chain with no threshold, *mixed* in at 0.12–0.2.** Four packs do it that way. The result is a soft haze over the whole picture, not just glow around bright spots. Bliss and Complementary also push bloom into fogged areas.
5. **One sky function or map is reused** for the fog colour, the reflection fallback and the clouds' far fade (all six). Bliss and Photon also feed it into ambient light.
6. **Distant Horizons shadows** are always screen-space marches (BSL 16 steps, Complementary 32, Solas 16, Photon screen-space traced shadows, Bliss contact shadows). No pack draws the LOD into a shadow map. Our far 400-block cascade with `fs_lod_shadow` is ahead of all of them there.
7. **Coloured floodfill light** exists in all six but is **off by default everywhere**, and forced off on macOS in Complementary and Solas.

---

## Ideas worth taking (ranked)

Ranked by value to our Bliss-like look against cost. Costs are per pixel unless the text says otherwise. Our Mac budget (`reports/performance.md`) argues for cheap wins first.

1. **Bloomy fog: bloom into the fog** (Bliss `dimensions/composite11.fsh:152`, Complementary `lib/atmospherics/fog/bloomFog.glsl`)
   - **What:** where fog or haze covers the scene, mix toward the blurred image: `mix(col, bloom, (1-fogT)·k·(1+rain))`. Distance and rain read as soft glowing air, which is a big part of Bliss's look.
   - **Cost:** given a proper bloom (idea 2), one mix plus a fog-transmittance value. The fog pass would write it to alpha or a small target. Zero extra passes.
2. **A mip-chain bloom with no threshold, mixed in** (BSL, Complementary, Photon, Solas, Bliss)
   - **What:** replace our single-pass three-ring, thresholded `bloom.wgsl` with 5–7 downsample levels (13-tap CoD:AW-style downsample, tent upsample). Blend `mix(scene, bloom, 0.12–0.2)`, or add it at about 0.5·exposure^0.2 as Bliss does.
   - **Cost:** about 6 small passes from 1/2 down to 1/64 resolution, 13 taps each. It costs less than a full-resolution ring sampler at Retina size. Needs engine support for multi-pass post effects at lower resolutions, which is an engine change.
3. **Cloud shadows inside the god rays and fog** (Solas `lib/lighting/cloudShadows.glsl` `exp(-3.5n)` in VL; Bliss `GetCloudShadow` at every VL step; Photon crepuscular rays through the cloud shadow map)
   - **What:** our `volume.wgsl` and `look.wgsl` `fog_through` / `sun_reaches` test only the terrain shadow maps. Multiplying each step by `cloud_shadow(rel)` gives beams that fall from cloud gaps onto the land, the "Bliss moving shade" in the air too.
   - **Cost:** about 4 texel reads (`cloud_cell_of` × 4 blended cells) × 16 VL steps. Could be cut by sampling every other step, or by a 1-tap version.
4. **Cloud lighting: light taps toward the sun, powder and a multi-scattering term** (Bliss `lib/volumetricClouds.glsl` `DoCloudLighting`; Complementary's 2-tap `light = 2 - s1 - s2`; Solas's 1 tap per step)
   - **What:** inside `fs_clouds`, take 1–2 samples of the cloud picture offset toward the sun per step. Light = `exp(-10·od) + 0.5·exp(-3·od)·(1-exp(-10·density))`, with our existing HG lobes (two forward lobes, 0.85 and 0.35, plus a weak back lobe). This gives silver edges and dark bellies instead of only height shading.
   - **Cost:** +1–2 texture taps × 10 steps.
5. **Haze lit with a forward Mie lobe and the shadow maps** (Bliss `lib/overworld_fog.glsl`, its atmospheric haze; Mellow `global/atmosphere/fog.glsl` single scattering, g 0.6)
   - **What:** our `haze_lit` tints by the sky behind. Bliss lights its haze with the Rayleigh phase plus a sharp forward lobe ×5, and shadow-maps it. The result is a warm glow over distant terrain toward a low sun, with dark lanes behind hills. Apply the lobe to the haze term only; the shadow taps already exist in `fog_through`.
   - **Cost:** arithmetic only for the lobe. The shadow part reuses the existing per-step taps.
6. **Daily weather variety** (Bliss `DAY0..9` by `worldDay%10`; Solas sum of sines over `worldDay`; Photon `include/weather/core.glsl` temperature/humidity/wind noise)
   - **What:** per-day cloud coverage, cloud height and thickness, and morning-fog density, set on the CPU side from the day number, the biome and humidity.
   - **Cost:** **zero GPU.** A few values written by `mods/base/src/weather.rs` / `sky.rs`, which already pass cover and fog through `scene.weather`.
7. **Purkinje night shift** (Bliss composite11, `lum*(0.4,0.7,1.0)*5`; Photon intensity 0.05; Mellow option)
   - **What:** in low light, blend toward a blue-tinted luminance. This replaces the hand-made cool floor and the 60% near-black desaturation in our `display()` with one principled step. Nights stay dark (our rule) but read blue, not black or muddy.
   - **Cost:** a few ALU ops in `display()`.
8. **Morning and evening valley fog from a short-half-life Mie layer plus a time-of-day table** (Photon: Mie half-life 7 blocks, with the Mie density table under its fog section; Bliss `FogDensities` morning 0.01/0.05, night 0.2; Solas valley height falloff)
   - **What:** our `fog_density` already lies on the ground. The ideas are a density keyed to time of day (thick at dawn, near zero at noon, some at dusk and night) and a 7-block exponential half-life above the floor, which makes thin mist that hills stand out of.
   - **Cost:** arithmetic. The density table can live on the CPU.
9. **Sky ambient from a small sky map with SH** (Photon `deferred4_a` sky SH compute; Bliss 9-sample ambient from its sky map)
   - **What:** shade-side light (`look_fill`) takes the sky's actual colour from the sky's directions: orange-pink at sunset, deep blue at noon. Right now it comes from palette constants.
   - **Cost:** one 192×108 sky render + a 256-sample SH reduction per frame, or every few frames. Shading then evaluates 9 coefficients. Needs an engine compute or pre-pass, so medium cost and an engine change. A cheaper approximation is 3 `sky_color()` evaluations (up, horizon toward the sun, horizon away from it) per frame, done on the CPU and passed as uniforms.
10. **Screen-space shadows past the far cascade** (Complementary 32 steps × 2.5 blocks through LOD depth; BSL 16; Photon screen-space traced shadows, 10 steps)
    - **What:** we already cast LOD into a 400-block far map. Past it, LOD lies flat. A short march along the light in depth space would give far hills a shaded side at a low sun.
    - **Cost:** 16–32 depth taps on LOD pixels beyond 400 blocks only, as a post effect over the depth buffer.
11. **Light shafts that know where you are** (Complementary `volumetricLight.glsl`: 25 shadow texels plus a stored height every ~66 ms, ramped by 1/255)
    - **What:** we already have `scene.canopy` from the biome. Complementary instead measures the actual cover above the player, which also works for clearings, caves, houses and lone trees.
    - **Cost:** about 25 taps once per frame, into one value. An engine-side uniform would be best.
12. **Rayleigh-reddened distant clouds and cheap aerial perspective on clouds** (Bliss clouds through Rayleigh extinction; Solas's 40% push toward the atmosphere colour)
    - **What:** far clouds near the horizon fade toward the sky colour and warm at sunset, rather than staying white.
    - **Cost:** one mix in `fs_clouds`.
13. **A two-tone sunset made by raising the light colour to a power** (BSL `lightSun^(4-sunSkyVis)`; Solas's twilight green band `exp(-(y-0.1)²/0.025)·0.2`)
    - **What:** cheap, controllable horizon colour richness. The green band is a subtle "real twilight" cue that none of our keys has.
    - **Cost:** arithmetic in `sky_color()`.
14. **Caustics through a shadow colour target** (Solas, BSL, Complementary: the curvature of the water height field written into `shadowcolor0` in the shadow pass)
    - **What:** sunlit caustic patterns on riverbeds and shallow seabeds, and on anything under water the sun reaches.
    - **Cost:** the engine's shadow pass would need a colour target, which is an engine change. Then one extra tap where shading reads the shadow. Our `world.wgsl` has a caustic already (checked with grep); this would make it follow the actual sun and waves.
15. **Temporal accumulation (TAA or history) for clouds and VL** (Photon 1/16-per-frame cloud refresh; every pack's golden-ratio-shifted dither)
    - **What:** this would let clouds take more light steps and VL more steps at the same cost, and remove dither noise.
    - **Cost:** history buffers and reprojection in the engine. Large, and an engine change. The highest long-term value, but the most work. Photon's cloud upscaling is the template.

**Ideas judged not worth taking:**
- Bruneton lookup tables. Bliss's vivid look comes from a single-scattering sky plus hand tweaks, and our keyed palette already matches it artistically.
- Floodfill coloured light. It is off by default everywhere and forced off on Mac by two packs.
- Mellow's "flipped image" reflections. Our screen-space reflections are better.
- Hable or Uchimura curves. Lottes plus our grade is where Complementary and Photon also landed.
