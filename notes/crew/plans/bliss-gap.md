# Bliss gap list: method by method (2026-11-25)

Ronald's bar (via Planner, 11-25): "nearly one-to-one" with Bliss in the same world, then tweak
from there. Rule: read Bliss's code first, replace our layers with its method and its own constants,
delete what it doesn't have, and use runs only to confirm a rebuild, never to drive a search.
Bliss is "© Chocapic13, all rights reserved": below is its method in words and maths only, no code.
Its matrix refs (`images/bliss-stage2/2026-11-24/ref/bliss_matrix`) ran with Ronald's profile
(`pack_settings.txt`) and vanilla textures with no LabPBR pack, so its SSS and emissive maps read
empty there.

Score per pose: `chiacraft-archive/tester-tools/sheets/lk_match.py` (mean CIE76 dE + SSIM on luma at
480x270, logged to `images/bliss-stage2/match.csv`). Every sheet is still judged by eye first.

## Order (biggest gap first) and state

| # | Section | Biggest gaps | State |
|---|---|---|---|
| 1 | Fog and air (`## Fog`) | land dimmed by air, sun-lit air all round, hand mist banks + burn-off, rain's own light, border fog, bloomy fog cut in rain | **rebuilt** 7689bf207 (lkmxO1 confirms) |
| 2 | Sky (`## Sky`) | land's ambient from the sky average (hue x min(3 luma, 2.5)); our march out of balance (Rayleigh phase 2-3x weak, ozone ~10x, no ground darkening, ATM_SCALE 22 vs ~7.6); twilight/night floor and 15000 K moon; sun disc ~2x big; rain sky a hand slate | next |
| 3 | Lighting (`## Lighting`) | our sun cut by sky light (Bliss: shadow map only; cave walls 3 %); cave's sunlit sides swapped (check tilt sign +35 vs -35); shade a cool hand fill vs sky average lit by 4300 K; leaves backlit/boosted; AO on all light (Bliss: min(vanilla, SSAO) on indirect only, engine); no rough sun highlight; torch curve | after sky |
| 4 | Clouds (`## Clouds`) | two cumulus layers (250-350, 500-600) + altostratus 2000, 2D coverage, two-lobe light, stop at render distance, per-pixel cloud shadow 2^(-100 s^2); night clouds too bright | after lighting |
| 5 | Water and post (`## Water and post`) | bloom radius one level short (Bliss ~250/1000 px/screen-wide); DOF (profile on, autofocus); terrain sun sheen F0 0.02 x 10 + rain-mirror ground; water body in-scatter; our vignette (delete); sharper sun glint | last |

Engine pieces on this list (each its own ENGINE_CHANGES line): the eye's sky light (eyeSky^3) and
cave factor as uniforms, the froxel volume's removal (~900 WGSL + 144 Rust lines once nothing reads
it), AO split (indirect only), DOF, a wider bloom level.


---


---

## Fog

Bliss source: `references/shaders/bliss/Bliss-Shader-git/shaders` (below `B/`). Bliss is © Chocapic13, all
rights reserved: what follows is its method in plain words and its math only. Ronald's profile
(`bliss_matrix/pack_settings.txt`) is applied; anything it does not set is Bliss's default from `lib/settings.glsl`.
The reference shots were taken **with Distant Horizons on** (`bliss_matrix/README.txt`), which changes two constants
(atmosphereMult 1.0 rather than 1.5, and the march and border fog reach to `dhRenderDistance` rather than `far`).

Ours: `mods/base/shaderpack/look.wgsl` (below `L`), `post.wgsl` (`P`), `crates/engine/src/client/froxels.rs`,
`environment.rs`. The current defaults are `look_fog_one` 1, `look_mist_domes` 1 and `look_shadow_reach` 128
(`mods/base/src/look.rs:303-318`), so the live path is `one_fog`. Everything else is either dead under it or still
running beside it.

### Pass map (Bliss, world0)

| world0 program | file it includes | what it does for fog |
|---|---|---|
| deferred | `B/dimensions/deferred.fsh:273-299` | Builds the 256² "sky + clouds + fog" panel of colortex4. It runs `GetVolumetricFog` along each sky direction (suncol×2, skyGroundCol/30, averageSkyCol_Clouds×5, result /5) and is used for reflections. The seen sky does **not** take its fog from here. |
| composite | `B/dimensions/fogBehindTranslucent_pass.fsh:375-397` | On translucent pixels only: for non-water it runs the same march from the camera to the opaque surface behind. For water it runs the underwater march between the water surface and the floor. |
| composite2 | `B/dimensions/composite1.fsh:1305-1333` | Deferred lighting. Applies the **border fog** to opaque pixels, then the "fog behind translucents" from the pass above (5-tap bilateral). |
| composite3 | `B/dimensions/composite2.fsh:364-449` | **The main march**: `GetVolumetricFog(viewPos0 …)` per pixel, from the eye to the nearest surface (translucent included; the sky is clamped to the far distance). Its output is rgb = in-scattered light, a = fog transmittance. When the eye is in water this is replaced by the underwater march. |
| composite4 | `B/dimensions/composite3.fsh:231-481` | 5-tap depth-aware upsample/filter of the march, cave fog, underwater absorption, `color = color·vl.a + vl.rgb`, then writes **bloomyFogMult** to colortex7.r. |
| composite12 | `B/dimensions/composite11.fsh:144-154` | The final pass: the bloomy fog mix, then the tonemap. |

---

### 1. The march itself (steps, reach, resolution, jitter)

**Bliss's method.** `B/lib/overworld_fog.glsl:65-341`, called from `B/dimensions/composite2.fsh:416`.
- One ray per pixel from the eye to the surface. Its length is clamped to `max(far, dhRenderDistance)` with DH on
  (`:95-99`). Sky pixels are marched too, to that clamp.
- `VL_SAMPLES` = 8 steps, spaced exponentially with `expFactor` = 11:
  - position `d(u) = (11^u − 1)/10`
  - step weight `dd = 11^u · ln11 / (8·10)`
  - `u = (i + dither)/8` (`:179-180`). The weights sum to 1 along the ray.
- **Units (important):** the fog's optical depth is `density · dd · dL` with `dL = rayLength/8` (`:104-105`). Fog
  density is therefore **per 8 blocks**: σ per block = density/8. The air (section 3) uses the full length
  `dL_alternate`.
- Dither: R2 and blue noise, moved every frame (`composite2.fsh:371-372`); TAA resolves it.
- Early out when the fog's transmittance drops below 1e-5 (`:336`).
- Resolution: `VL_RENDER_RESOLUTION` defaults to 0.5 (`lib/res_params.glsl:16`), but **Ronald's profile sets 1.0**, so
  the march runs at full size. `BilateralUpscale` (`composite3.fsh:133-182`) then acts as a 5-tap filter (centre plus
  the four diagonals) that drops taps whose linear depth differs by more than `zMults.x` (DH: 0.01 of
  sqrt(depth/65000)).

**Ours.** `L:2425-2512` `one_fog`, run once per pixel before TAA in `P:121-134` `fs_pre_taa`, to the nearest
surface's depth (`P:355-367` `view_rel`).
- Steps: 8 (`look_mist_steps`), spaced 11^u, identical to Bliss.
- Jitter: interleaved gradient noise plus `frame.w`.
- Mist banks are walked only out to `MIST_MARCH_FAR` = 1500. Rain and air go all the way to the surface.
- The sky is marched to 1500 and gets **no air** (`is_sky`, `L:2455-2460`).
- No bilateral filter.

**Verdict: same in structure.** The differences:
- (a) Ours uses no /8. Our densities are per block, so a value copied from Bliss lands 8× thicker (see section 2).
- (b) Bliss adds air in front of the sky too, out to the DH distance; ours puts none on sky pixels.
- (c) There is no 5-tap depth-aware filter.

**Rebuild:** add the 5-tap filter (about +30 lines, or skip it since TAA covers most of it), and decide (b) together
with the physical sky. No engine work.

---

### 2. Density: uniform + cloudy + rain, by height and by hour

**Bliss's method.** `cloudVol` (`overworld_fog.glsl:16-48`) feeds `FogDensities` (`lib/climate_settings.glsl:204-234`,
enabled by `#define TIMEOFDAYFOG` in `deferred.fsh:139`, `composite2.fsh:144` and `fogBehindTranslucent_pass.fsh:93`).

**Fog floor.** `fogYstart = FOG_START_HEIGHT + 3` = 50 + 3 = **53** (profile 50; default 60).

**Uniform fog** = `exp2(−0.15·max(y − 53, 0))` (it halves every 6.7 blocks).

**Cloudy fog**:
- Noise `n(p)`: one texture lookup of a 512² noise, with p/18 and xz additionally ×0.5. The cells are therefore
  **36 blocks across and 18 tall**, smoothstepped between y layers (`:3-13`).
- `shape = 1 − n(pos)`.
- `erosion = 1 − n(pos·(200, 200/48, 200) − shift)`, where shift = 200·clamp((shape − 0.6)·2). In blocks this noise is
  about 0.18 blocks across and 4.3 tall, so it acts as a per-pixel grain that TAA averages into soft edges.
- `cloudy = max(clamp((shape − 0.6)·2, 0, 1) − 0.4·erosion, 0) · exp(−0.05·max(y − 73, 0))`.
  Since shape ≤ 1, its peak is about 0.8 and it is usually much lower.

**Rain fog** = `(0.5·exp2(−0.3·max(y − 53, 0)) + exp2(−0.06·max(y − 53, 0))) · rainStrength · noPuddleAreas`.
noPuddleAreas is 0 in dry and snowy biomes (`shaders.properties:456`).

**Sand or snow storm** (biome uniforms, `shaders.properties:441-450`): cloudy is replaced by a scrolling noise, and
uniform is set to 1 at every height (`:38-43`).

**Time-of-day weights.** These come from `worldTime % 24000`, not the weather (`Daily_Weather=false` in the profile;
`PER_BIOME_ENVIRONMENT=false`). Each window is a pair of linear ramps:

| Window | Rises | Falls | Weight 1 between |
|---|---|---|---|
| Morning | 22000→24000 | 0→2000 | 0 (06:00) |
| Noon | 0→2000 | 10000→12000 | 08:00 and 16:00 |
| Evening | 10000→12000 | 12000→14000 | 18:00 (peak) |
| Night | 13000→15000 | 21000→23000 | 21:00 and 03:00 |

Densities (`TOD_Fog_mult` 1), with the profile values:

| Hour weight | Uniform | Cloudy |
|---|---|---|
| Morning | **0.10** | **0.45** |
| Noon | 0.0 (default) | 0.0 (default) |
| Evening | **0.18** | **0.09** |
| Night | **0.38** | **0.24** |

- `Rain ×= RainFog_amount` = **0.3**. Rain is **not** weighted by hour.
- The density in the march is `uniform·U(t) + cloudy·C(t) + rain`, times `inACave = 1 − caveDetection`.

What that means per block (/8):
- Morning cloudy core ≤ 0.45·0.8/8 ≈ **0.045/block**.
- Morning uniform: 0.0125 below y 53, 0.0044 at y 63, 0.0016 at y 70.
- Night uniform: 0.0475 below y 53, **0.017 at y 63**, 0.008 at y 70. In this profile the night is foggier than the
  morning.
- Noon: no fog at all; only the air and the rain remain.
- Rain fog: **0.027/block** at y 63, 0.019 at y 70, 0.005 at y 100, 8e-5 at y 200.

**Ours.**

`bank_density` (`L:2338-2362`):
- Broad noise `noise3(q / (36, 18, 36))` with a linear cut from 0.55 to 1.05 (`MIST_SHAPE`).
- Erosion `noise3(q / (1.5, 3, 1.5) − 7.3·cut)` × 0.4 (`MIST_ERODE`).
- The cloudy part falls by `exp(−0.05·max(y − 73, 0))` (`MIST_SEA_LEVEL` 10 over sea 63, `MIST_SEA_FALL` 0.05).
- The even part is `0.2·exp2(−0.15·max(y − 53, 0))` (`MIST_SEA_EVEN`).
- Both are scaled by `k = air_mist() · MIST_SEA_DENSE (9) · burn`, applied **per block**.
- `burn = (1 − smoothstep(0.2, 0.45, sun.y)) · mix(0.03, 1, smoothstep(−0.1, 0.1, sun.x))`, i.e. by the sun's
  elevation and its east/west side.

The rest of the density:
- Rain: `RAIN_FOG` 0.004 · `exp2(−0.03·(y − 63))` (`L:2479`).
- Banks: only when `GROUND_MAP_ON`, `storm < 0.5` and `air_mist > 0` (`L:2431`). Air mist is the weather's
  `scene.weather.z`, so with no weather mist there are no banks at any hour.
- Storm dust and snow: none in `one_fog`. The old `fog_density_with` storm branch (`L:2023-2103`) is not in this path.

**Verdict: partly.** The shape and heights already match Bliss: the 36×18 cells, the threshold, the 53 and 73 heights,
the 0.15 and 0.05 falls, and the 0.2 even/cloudy ratio of the morning. The method around them does not:
1. **The schedule.** Bliss uses a fixed table by hour that ignores the weather. Ours depends on the weather's
   `air_mist`, has one burn curve, and has no separate uniform/cloudy weights by hour.
   - **Night:** Bliss's densest uniform (0.38) against ours at about 0.5× the morning banks, decided by sun.x.
   - **Evening:** Bliss's 0.18 / 0.09 against ours at 3 % of the morning.
   - **Morning fade:** Bliss's weight falls linearly from 06:00 (0.5 at 07:00, 0 at 08:00). Ours holds 0.85 at 07:00
     and reaches 0 at about 07:45.
2. **Units.** The comment at `L:467-470` reads Bliss's 0.45 as "a block". It is 0.45/8 (`overworld_fog.glsl:105`).
   With the natural dawn `air_mist` ≈ 0.05, ours is 0.45/block × banks, so **about 8× Bliss's density** at the same
   weight. Ronald's "too transparent" at 0.075 was judged under our lighting (section 4), so the density was tuned
   around a lighting gap.
3. **Rain fog** is about 7× thinner at the ground in ours (0.004 against 0.027 at y 63) and halves every 33 blocks
   where Bliss's halves every ~17. Bliss's is also floored at y 53 by the 0.3 exp2 term.
4. **Erosion** in ours is a 1.5-block noise; Bliss's is a sub-block grain that TAA softens.
5. **Storms** (snowStorm and sandStorm with uniform = 1 at all heights) are missing from `one_fog`.

**Rebuild:**
- Replace `burn`, `air_mist` and `MIST_SEA_DENSE` with Bliss's four-window weight table on the day's ticks. They can be
  derived from the sun's angle, `atan2(sun.y, sun.x)` → ticks; no engine work needed.
- Two weights, U and C, with Ronald's eight numbers. Densities = value/8 per block.
- Keep the weather's mist only as a `max()` floor, the way Bliss's Daily_Weather does.
- Rain: `(0.5·exp2(−0.3h) + exp2(−0.06h))·0.3/8`, with h from 53.
- Delete `MIST_SEA_DENSE`, `MIST_BANK_EVENING`, `burn`, `OPT_BANK_DENSE/FALL/TOP/EDGE`, `RAIN_FOG` and the evening-mist
  path (`evening_window`, `evening_floor`, `OPT_EVENING_MIST`).
- About −40 / +25 lines.

---

### 3. The air: Rayleigh and Mie haze in the march (`Haze_amount`)

**Bliss's method.** `overworld_fog.glsl:118-149, 256-268`.

Coefficients:
- `rC = (5.8e-6, 1.35e-5, 3.31e-5)` (the sky's Rayleigh, `settings.glsl:315-317`).
- `mC = 3e-6` grey (`fog_coefficientMie`, `:188-190`).
- `airCoef = exp2(−max(y − SEA_LEVEL(70), 0) / (8000, 1200) · (6, 7)) · 24 · atmosphereMult · Haze_amount · cloudPlaneCutoff`.
  - With DH: 24 · 1.0 · **1.25** = **30**.
  - `cloudPlaneCutoff` switches the air off above the cumulus layer's base (250, raised with the eye's altitude).
- `rL = rC·airCoef.x`.
- `m = mC·(airCoef.y + 300·fogDensity)`: the fog's droplets add Mie.
- At the ground that is Rayleigh (1.74e-4, 4.05e-4, 9.9e-4) per block and Mie 9e-5 per block.

Light:
- `Atmosphere = sunCol·sh·sunPhase·(rayPhase·rL + sunPhase·m) + averagedSky·(rL + m)·(eyeSky³·0.99 + 0.01)`, all times
  inACave.
  - `sunPhase = fogPhase(cosθ)·5` (section 4), so the Rayleigh term carries the forward fog phase × Rayleigh's phase.
  - `rayPhase(c) = (0.1·c + 0.28)/π`.
  - The Mie term carries sunPhase². The air's sun light is therefore almost all toward the sun:
    - Rayleigh × phase ≈ 1.5 at the sun, 0.02 across, 0.004 away.
    - Averaged over the sphere, sunPhase ≈ 0.54.
  - Away from the sun the air is lit by the **averaged sky** (`averageSkyCol_Clouds/30`), isotropically.
- Integrated per step as `(A − A·T)/(rL + m)` with `T = exp(−(rL + m)·dd·L)` over the **full** length. It is scaled by
  the air's own running transmittance and the fog's.
- **The air never dims what lies behind it.** It is "purely additive" (`:263`). The scene's alpha is the fog's
  transmittance only. The air's mean transmittance (`atmosphereAlpha`) only fades the clouds (`composite2.fsh:417`,
  and in the LUT the sky, `deferred.fsh:295`).

**Ours.** In `one_fog` (`L:2441-2446, 2493-2508`):
- `r_k = ATM_RAYLEIGH · AIR_THICK (18) · exp2(−(y − 63)/8000·6)`.
- `m_k = 3e-6 · 18 · (exp2(−(y − 63)/1200·7) + 300·d)`, where d is our per-block density, so the droplet Mie is 8×
  Bliss's again.
- Light: `p.sun · light_strength · lit · (0.75(1 + μ²)·r_k + HG(μ, 0.8)·m_k) + p.ambient·ext`. These are normalised
  phases, so the **sun lights the air about equally in every direction**: 0.75 to 1.5 for Rayleigh, against Bliss's
  ~0.004 to 1.5.
- No cloud-plane cutoff. No eye-sky-light factor. No cave factor.
- The land behind **is dimmed** by the air's per-channel transmittance (`land_kept`, `L:2506-2511`), added because
  "lit and never dimmed, the far land only got brighter".
- None on sky pixels.
- Tuned: `AIR_THICK` 18, not 30 (`L:291-296`), and `air·(1 − 0.5·rain)`.

Old layers this replaced, all skipped while `one_fog` is on:
- `haze_veil` (`L:2877-2904`) and `wet_depth` (`:2909-2917`).
- `haze_lit`'s whole body (`L:2924-3169`): lobe, sun tint, `HAZE_SUN_GLOW`, evening band, day air, twilight air.
- The froxels' air light.

**Verdict: partly, with the method's two key points inverted.**
1. Bliss lights the air with the sun only through its forward phase, and with the averaged sky elsewhere. Ours sun-lights
   it everywhere, which made the far land too pale and white.
2. To compensate, ours dims the land by the air and cuts the thickness to 18. Bliss never dims the land and keeps the
   full 30.

This is a stacked fix for one method gap. It is the main reason the far-land colour needs tuning at every hour.

**Rebuild:**
- Bliss's light term exactly as written: sun × shadow × sunPhase × (rayPhase·rL + sunPhase·m), plus averagedSky ×
  (rL + m) × eyeSky³.
- Drop `land_kept`. Thickness 24 × 1.25 = 30.
- Add the cloud-plane cutoff.
- Use the air's mean transmittance only to fade the clouds and sky. That is the clouds' area; check whether our clouds
  already fade.
- About ±0 lines. Delete `land_kept` and `AIR_THICK`'s tuning. No engine work.
- Needs: an "averaged sky" colour. Bliss's is the sky LUT averaged with clouds (pixel (0,37)); ours could be
  `p.ambient` or the mean of `sky_phys`.

---

### 4. The fog's lighting (sun phase, shadow taps, cloud shadow, ambient)

**Bliss's method.** `overworld_fog.glsl:54-62, 118-135, 173-174, 186-246`.

Inputs (`composite2.fsh:397-399`):
- `LightColor = lightCol.rgb/80`. lightCol is the sun's (or moon's) colour from the sky LUT. In the profile:
  `sun_illuminance` 250000, `moon_illuminance` 200, `Sun_temp` 4300 K.
- `Ambient = averageSkyCol/30`.

Phase:
- `fogPhase(c)`: with l = 1 − clamp(c·0.5 + 0.5), it is `1.5·exp2(−15·l^0.3) + exp2(−6.25·√l)`, times 5.
- Values: 12.5 at the sun, 1.7 at 30°, 0.23 across, 0.066 away.
- `cosθ = dot(sunVec, view)·lightCol.a`, where the sign flips to the moon at night.

Sky:
- `skyPhase = 5·clamp(dir.y·0.5 + 0.5)^4`.
- `skyLight = Ambient·(1 + skyPhase)`: ×6 looking up, ×1.3 level, ×1 down.

Per step:
- `sh` = one hardware-PCF tap of the shadow map. It is coloured through translucents (`TRANSLUCENT_COLORED_SHADOWS`
  = true in the profile, `:199-205`), and 1 outside the map (the map covers shadowDistance 128).
- `sh ×= cloudShadow`: each cloud layer's density along the sun at its height, summed s, then `exp(−100·s²)`
  (`lib/volumetricClouds.glsl:642-664`).
- `lighting = LightColor·sunPhase·sh + skyLight·(eyeSky³·0.99 + 0.01) + lightning`.
  - eyeSky³ is `(eyeBrightnessSmooth.y/240)^3`: under a canopy or in a ravine the fog's sky light drops with the
    **eye's** sky light.
- `color += lighting·(1 − T)·fogT`. The albedo is 1 and there are no other factors.

**Ours.** `one_fog` (`L:2432-2440, 2466-2491`):
- `sun_in = cream · light_strength · bank_phase(μ) · 0.75`. `bank_phase` equals Bliss's fogPhase×5. `cream` is the
  sun's colour moved 15 % toward a luma cream (`MIST_SEA_CREAM`).
- `sky_in = p.zenith recoloured to p.ambient's luma · (1 + 5·up^4) · 0.45 · 0.75` (`MIST_SEA_SKY`).
- The sun's share fades in over the first 8 blocks (`MIST_NEAR_SUN`).
- `lit`: one `sun_reaches` tap (`L:2178-2204`).
  - It is a hard compare, no PCF.
  - It reads the near and far cascades.
  - Within `shadow_reach` 128 it fades to lit from 96 to 128.
  - It is multiplied by `cloud_shadow` (the soft-cloud map).
- No eye-sky-light factor and no coloured shadows.
- Rain gets its own light: `rain_sky = luma(ambient)·(1, 0.96, 0.86)·0.45` and `rain_sun = cream·phase·0.2`.

Old layers, skipped with fog_one but still compiled and used by froxels:
- `fog_light` (`L:2123-2174`): two HG lobes 0.8/0.1 ×0.45, `MIST_GOLD` warmth, a dawn/evening tint split, leaf light,
  a 0.55 sky cut.
- `mist_bounce` (`L:2789-2798`).

**Verdict: partly.** The phase and the sky's up-weight are Bliss's. On top of them sit five factors Bliss does not
have: the cream tint, the zenith recolour, the 0.45 sky share, the overall 0.75 `k`, and the near-sun fade. Rain has
its own 0.45/0.2 light. The eye-sky³ is missing.

**Rebuild:**
- Use `p.sun` (or `sun_kelvin` × strength) × phase × shadow, plus `ambient·(1 + 5·up^4)·eyeSky³`, with albedo 1.
- One light for mist and rain alike.
- Delete `MIST_SEA_CREAM`, `MIST_SEA_SKY.z`, `k`, `MIST_NEAR_SUN`, `rain_sky`, `rain_sun`, `OPT_BANK_SUN/SHADE`, the
  `OPT_MIST_TINT` and `OPT_MIST_DAWN_TINT` paths, and `fog_light`, `MIST_GOLD` and `mist_bounce` (together with the
  froxels).
- Engine: a smoothed **eye sky-light** uniform (Bliss's `eyeBrightnessSmooth`), a few lines in the client plus a scene
  field. Coloured shadow taps belong to the lighting area.
- About −120 WGSL lines.

---

### 5. Rain fog

**Bliss's method.** This is the density term from section 2 (0.3, by height, not by hour), lit like all fog (section 4).
In rain the sun's colour and the ambient are already the rainy LUT's, and the cloud shadow gives the shafts. Bloomy fog
also grows in rain (section 8).

**Ours.** `RAIN_FOG` 0.004·exp2(−0.03·above sea) in `one_fog`, with its own light (`L:2448-2449, 2486`), and air ×
(1 − 0.5·rain).

The engine also has a rain mist (`environment.rs:236-259`): a distance-fog layer, `core:rain_mist`, ending at 170
blocks (110 in a thunderstorm), starting at 0.12 of that, its colour 80 % grey. Under fog_one the pack ignores its
colour and only uses its weight to move the border fog's end (`L:2933-2938`). It has already forced one special case
(`in_liquid`, `L:13-21`).

**Verdict: partly.**
- About 7× thinner at the ground with a slower height fall.
- Separately lit.
- Air cut by half.
- Bloomy fog cut where Bliss's grows.

**Rebuild:**
- Bliss's rain term and the shared light. Drop the rain's own light and the `(1 − 0.5·rain)` on the air.
- Gate it by Bliss's noPuddleAreas equivalent: biome precipitation is rain; no rain fog in snow or dry biomes.
- Retire `core:rain_mist`'s effect on the pack. In the engine it can stay for packs without a march; add a small engine
  check if it must be off. About −15 lines.

---

### 6. Border fog where the terrain ends

**Bliss's method.** `composite1.fsh:1305-1323` (opaque, in the lighting pass, before the VL fog). The same is done in
`all_translucent.fsh:688-827` and `DH_translucent.fsh:402-481`.
- `fog = smoothstep(1, 0, clamp(3·(1 − dist/dhRenderDistance), 0, 1))`: none to 2/3 of the distance, full at the end.
- Times `exp(−10·(4·clamp(dir.y, 0, 1))²)`: only near-level or downward looks. At 6° up it is about 0.2.
- Not on the sky, and not under water.
- The colour is the plain sky (`skyFromTex`, no fog and no clouds) in the pixel's own direction.
- The VL fog and air are then added over it.

**Ours.** `haze_lit` under fog_one (`L:2925-2944`):
- `edge = smoothstep(0.84·end, end, dist)`, where end = the engine's fog end, moved to 1500 in rain.
- Mixes toward `sky_color(dir with y×0.35)`, a flattened direction.
- No upward-angle term.
- Applied in the surface shaders, so it comes before `one_fog` too, which matches Bliss's order.

**Verdict: partly.** Ours fades over the last 16 % against Bliss's last 33 %, uses a flattened direction, and has no
angle cut.

**Rebuild:** Bliss's two factors and the real direction (`sky_phys(dir)`). About ±5 lines. No engine work.

---

### 7. Caves

**Bliss's method.**
- **In the march:** density × `inACave = 1 − caveDetection` (`overworld_fog.glsl:173, 222`).
  - `caveDetection` is a smoothed flag (3 s in, 1 s out) that turns on when the eye's sky light is 0 and the cave
    "mood" is climbing (`shaders.properties:488-510`).
  - The air is also × inACave.
  - The fog's and the air's sky light are × eyeSky³.
  - While the eye's sky light is 0 the fog does not absorb: `totalAbsorbance *= mix(1, T, eyeSky³)`.
- **Cave fog** (`composite3.fsh:376-395`, `CAVE_FOG` on, before the VL blend):
  - Colour (0.1, 0.2, 0.5) × `(1 − (1 − u²)^CaveFogFallOff)`, with u = dist/far (the vanilla `far`, not DH) and the
    **profile's falloff 2.0**.
  - Times `exp(−7·clamp(dir.y·0.5 + 0.5))·0.999 + 0.001`: ~1 looking down, 0.03 level, ~0 up.
  - **Added** (not mixed) × caveDetection.
  - `CAVE_FOG_DARKEN_SKY` is off.

**Ours.**
- `one_fog` has **no cave guard**. `fs_pre_taa` checks only `open_sky()`, which means "the dimension has a sun".
- `bank_density` has no lower bound. At dawn its even part and banks are full below y 73, so caves and ravines under
  ~y 90 fill with dawn mist.
- The air's `p.ambient` sky light runs underground too.
- The old `fog_through` used the surface's sky light (`sky`), but that is skipped now.
- No cave fog.

**Verdict: missing** (and a regression under fog_one: the dawn mist in caves).

**Rebuild:**
- Engine: a smoothed cave flag, or simply the eye-sky-light uniform from section 4. In-cave = eye sky light ≈ 0 for a
  few seconds; it can skip Bliss's mood trick.
- Pack: density and air × (1 − cave), sky light × eyeSky³, and Bliss's additive blue cave fog. About +20 lines in the
  pack and +15 in the engine.

---

### 8. Bloomy fog (fog mixed toward bloom)

**Bliss's method.**

`bloomyFogMult` (`composite3.fsh:318-480`):
- Starts as `vl.a`, the **fog's** transmittance only, without the air.
- In rain, `mix(vl.a, 0.5·vl.a + 0.5, rain)`.
- × (1 − (5·rainDrop)²) where raindrops cover the screen.
- × 0.4 under water, and 0 in powder snow.

Final (`composite11.fsh:152-154`):
- `a = clamp((1 − mult)·BLOOMY_FOG·0.75·(1 + rain)·(1 − 0.3·purkinje), 0, 1)·clamp(1 − cdist(uv)^15)`.
  - BLOOMY_FOG = **1.25** in the profile, so 0.94 at full fog.
  - Rain doubles the factor while halving the opacity: about the same at full rain.
- `col = mix(col, bloom, a) + bloom·lightScat`.

**Ours.** Two inconsistent sources, plus a base mix (`P:426-433` chain path; `P:496-511` other path):
1. `mist_over`: the **froxel volume's** opacity (`P:369-387`) × 0.6. The froxels are built from the old
   `fog_density_with` model, not the fog that is drawn.
2. `bank_hidden` (`L:2379-2420`): a **second 8-step march** of the banks alone (no rain, no other terms), ×
   `BANK_BLOOMY` 0.94.
3. Both are × (1 − 0.85·rain). Bliss's goes up in rain.
4. Non-chain path: `BLOOM_MIX` 0.1·(1 + air) everywhere.

There is no screen-edge falloff, no purkinje term and no underwater ×0.4.

**Verdict: ours more complex and wrong in rain.**

**Rebuild:**
- `one_fog` writes its fog transmittance (without the air) to the alpha of its output, or to a small R8 target. Post
  reads it.
- Bliss's formula with the rain, raindrop, underwater and edge terms.
- Delete `bank_hidden`, `mist_over` and `BANK_BLOOMY`, and the base `BLOOM_MIX` coupling to `air`.
- About −70 lines.
- Engine: maybe the pre-TAA pass's alpha path. The colour target's alpha may already be free; otherwise one R8 target
  (small).

---

### 9. Under water

**Bliss's method.**

March (`composite2.fsh:229-325, 431-443`, replacing the fog when `isEyeInWater`):
- 8 steps (11^u) to min(dist, **32**).
- σ = Water_Absorb + Dirt_Absorb·(Dirt_Amount + 0.01) = (0.2629, 0.0565, 0.0101) + (0.65, 0.85, 1.05)·0.15 =
  **(0.360, 0.184, 0.168)** per block. Scatter = 0.15·0.7/π = **0.0334**.
- Per step:
  - ambient (`averageSkyCol_Clouds/30`) · exp(−σ·f) · (dir.y·0.3 + 0.7)^1.5
  - plus sun · shadow · cloud shadow · fogPhase×5 · exp(−1.1·σ·f) · caustics(mix(0.25, 10, near-bubble) + 0.75) ·
    |sun.y|
  - f is the step's 0..1 fraction of the ray, not blocks.
  - Everything × clamp(eyeSky/240, 0.1, 1), then × scatter.
  - Integrated as `(L − L·T)/σ` with T = exp(−σ·dd·len).

Absorption (`composite3.fsh:410-433`, before the blend):
- `absorb = exp(−σ·dist)` and `maxAbsorb = exp(−8σ)`.
- `fade = exp(−0.001·d²)`, where d = length(x, max(−y, 0), z), so looking up the vertical distance is free. Fade is 0.5
  at 26 blocks.
- `color = mix(sun.y·(dir.y·0.3 + 0.7)^1.5·maxAbsorb, color·mix(maxAbsorb, absorb, mean(absorb)), fade)`, then + the
  march.
- Bloomy fog × 0.4.

Translucent water seen from above: `fogBehindTranslucent_pass.fsh:393` runs the same scatter between the surface and
the floor. That belongs to the water area.

**Ours.** `under_water` (`L:2848-2872`):
- `mix(c·mix(0.6, 1, down), teal(0.04, 0.19, 0.15)·light·1.8·mix(0.35, 1, down), 1 − exp(−dist/18))`.
  - Grey extinction at 1/18 per block.
  - A fixed colour.
  - `down` = exp(−depth below the surface/28).
- Plus 6 taps of ridged noise "beams" projected to the surface, no shadow (`L:2856-2869`).
- Skipped by `one_fog` in liquid.

**Verdict: missing in method** (ours is hand-built).

**Rebuild:** Bliss's 8-step scatter march with σ and scatter as above, shadow and phase, plus the closed-form
absorption mix. It can live in `fs_pre_taa`'s liquid branch. About +45 / −30 lines. The caustics term can reuse our
water caustics. No engine work.

---

### 10. Resolution and cost; layers still running beside `one_fog`

**Bliss:** one march per pixel (full size in this profile), one for fog behind translucents, and the sky LUT's march
for reflections.

**Ours, with the defaults:**
- `one_fog` per pixel (full size), plus `bank_hidden`, a second 8-step march per pixel in post.
- **The froxel volume is still built every misty frame.** `froxels_wanted = core:froxels && hdr`
  (`crates/engine/src/app.rs:3234`, and the Max preset sets `FROXELS` 2 at `mods/base/src/video.rs:251`). It is read
  only by `mist_over` (bloomy fog) and `mist_kept` (`world.wgsl:1209`, the water's floor), both from the **old**
  density model.
- `held_in_fog` and `placed_in_fog` (`L:3176-3221`) also read the old `fog_density`.

**Dead under fog_one but compiled:**
- `fog_through*` (`L:2212-2285`)
- `froxel_fog` (`2653-2704`)
- `bank_march` (`2513-2616`)
- `haze_veil` / `wet_depth` / `haze_lit`'s body (~280 lines)
- `fog_density_with` / `fog_banks*` / `fog_light` / `mist_bounce` (~180)
- The `MIST_*` dome/wisp/evening constants
- `sky.wgsl`'s `fog_through_sky` callers

**Verdict: ours more complex**, and it pays for two layers (froxels and `bank_hidden`) that only feed side effects.

**Rebuild:** once sections 2-8 land:
- Delete `froxels.rs`, `fs_froxel_light/sum` and the scene field, and the old fog model (≈ 900 WGSL lines, 144 Rust).
- Feed `mist_kept` and the torch-in-fog from `one_fog`'s density function (`placed_in_fog`/`held_in_fog` keep their
  closed form with the new σ).
- Engine: remove the froxels pass (≈ −200 lines with its app/client wiring); a small `ENGINE_CHANGES` row. It saves the
  froxel passes' GPU time on every misty frame.

---

### Which of ours Bliss's single method replaces

| Ours | Replaced by (Bliss) |
|---|---|
| Mist banks: `bank_density` and its burn-off curve, `air_mist` gate, `MIST_SEA_DENSE`/`EVENING` | Uniform + cloudy fog with the four-window hour table (fixed, not weather) |
| The evening mist layer, `evening_window`, `BAND_HDR` far band | The Evening weights (0.18 / 0.09) |
| `one_fog`'s air with sun-lit isotropic phases, `land_kept`, `AIR_THICK` 18 | The additive air: sun only through the forward phase, averaged sky elsewhere, ×30, no land dimming |
| `haze_veil`, `haze_lit` (lobe, sun glow, day air, twilight air, far band), `wet_depth` | The same air term plus the border fog |
| Froxels (`froxels.rs`, `froxel_*`, `fog_density_with`, `fog_light`, `mist_bounce`, `fog_through*`) | The march (no volume) |
| `bank_march`, `bank_hidden`, `mist_over` | The march's stored fog transmittance |
| Rain's own light, `RAIN_FOG`, engine `core:rain_mist` | The rain density term, lit like all fog |
| `under_water` teal + noise beams | The 8-step water scatter march + absorption mix |
| (none) | Cave guard + additive cave fog |

### Ranked biggest visual gaps (fog and atmosphere)

1. **The air's light.** Ours sun-lights the air in every direction, then dims the land and thins the air to 18 to
   compensate. Bliss adds the air without dimming the land: lit by the sun only toward it, by the averaged sky
   elsewhere, ×30. This sets the far-land colour at every hour.
2. **The fog schedule and units.** Bliss runs a fixed hour table that ignores the weather (night uniform 0.38 the
   densest, evening 0.18/0.09, none at noon). Ours is driven by `air_mist` with one burn curve, and is about 8× too
   dense because it ignores Bliss's /8 step.
3. **The fog's light.** The cream tint, the 0.45 sky share, the 0.75 `k` and the near-sun fade are ours, not Bliss's.
   The eye-sky³ factor is missing.
4. **Rain fog** is ~7× thinner and separately lit. Its bloomy fog is cut where Bliss's grows.
5. **Bloomy fog** comes from a second march and stale froxels, not the drawn fog's transmittance.
6. **Caves.** Dawn mist fills them and the sky lights the air underground. Bliss's blue cave fog is missing.
7. **Under water** is hand-built teal against Bliss's absorbing scatter march.
8. **The border fog band** is narrower and flattened (minor).

---

## Sky

Bliss paths are relative to `.../Bliss-Shader-git/shaders/` (the overworld passes are thin `world0/*.fsh` wrappers that include `dimensions/*`). Ours are relative to the worktree: `look.wgsl` and `sky.wgsl` are in `mods/base/shaderpack/`, and `look.rs` is `mods/base/src/look.rs`. Values from Ronald's profile are marked **(profile)**; every other constant is Bliss's default from `lib/settings.glsl`.

**Units.** Bliss stores its sky LUT as `raw/4000 x 150` and draws it `/30`. The drawn sky is therefore `raw/800`, where `raw` is the march's radiance with a sun of `blackbody(4300) x 250000` **(profile)**. Its direct sunlight on land is `sunColorBase x T / 2133` (deferred.vsh:165, then x150 for storage and /80 in composite1.fsh:908). Ours uses `BLISS_UNITS = 41`, so ours x 41 = Bliss (look.wgsl:250). In our units, Bliss's sunlight comes out at about 1.9 at noon, which matches our palette's 1.92. Its drawn sky would be **S x 250000/800/41 = 7.6 x blackbody(4300) x S**, where S is the march's scattering per unit sun.

**Default state of ours (look.rs:260-325).** These are on: `look_sky_phys` 1, `look_chain` 1, `look_sun_disc` 1, `look_sky_lobe` 1, `look_sun_glow` 1, `look_twilight` 0.5, `look_evening_away` 1 and `look_chain_night` 0.08. These are off: `look_dawn_sky` 0 and `look_horizon_lift` 0. So the physical sky is what you see. The roughly 220-line hand sky (`sky_color`, look.wgsl:1263-1484) is dead code by default. The hand palette still lights the land and still feeds the sky through several side doors.

---

### 1. The sky colour model (the atmosphere march)

**Bliss's method.** `lib/ROBOBO_sky.glsl:95-165`, called from `dimensions/deferred.fsh:260`.

Geometry and density:
- **Planet:** radius 6731e3 m (sic, not 6371). The atmosphere is 110e3 m thick (ROBOBO:14-16).
- **Eye:** at `planetRadius + eyeAltitude`, where eyeAltitude is the camera's y in blocks, counted as metres (ROBOBO:106).
- **Ground:** the planet is intersected at radius x 0.998 (ROBOBO:111). The march runs from the eye to the atmosphere's top, or to the ground.
- **Density at radius r** (ROBOBO:52-59):
  - Rayleigh: `exp(-(r - R)/8000)`.
  - Mie: `exp(-(r - R)/1200)`.
  - Ozone: `exp(-max(0, 35 km - h)/5 km) x exp(-max(0, h - 35 km)/15 km)`, which peaks at 1 at 35 km.

Coefficients, per metre (settings.glsl:315-320, ROBOBO:9-32):
- **Rayleigh:** (5.8e-6, 1.35e-5, 3.31e-5).
- **Mie:** 3e-6 grey. Extinction equals scattering, so there is no absorption.
- **Ozone:** cross-section (4.511e-21, 3.285e-21, 1.968e-22) x (2.5035e25 x 8e-6 x 0.2e-6) = **(1.81e-7, 1.32e-7, 7.9e-9)**. Its column is about 20 km of peak, so the zenith optical depth is about 0.004 at most. That is close to none.

Phases (ROBOBO:38-46):
- **Rayleigh:** Bliss uses a linear stand-in, `(0.1 cos + 0.28)/pi`. Forward that gives 0.121, at 90 degrees 0.089, and backward 0.057. The physical `3/(16 pi)(1 + cos^2)` gives 0.060, 0.030 and 0.060. **Bliss's Rayleigh is therefore 2-3x the physical value and leans toward the sun.**
- **Mie:** Henyey-Greenstein, `(1 - g^2)/(4 pi) x (1 + g^2 - 2 g cos)^-1.5`, with **g = 0.8**, for both the sun and the moon (ROBOBO:121-122).

The march (ROBOBO:116-160):
- **Steps:** 10 view steps, with the start offset by `0.34 x blueNoise` (ROBOBO:119). Each light ray takes 4 steps.
- **Per step:** with `od = beta x airmass` and `T_step = exp(-od)`, the visible weight is `(T_step - 1)/(-od) x T_view x GroundDarkening` (ROBOBO:140-142).
- **The sun term:** `beta_s x airmass x phaseSun x visible x T_light(p, sun*0.5 + 0.1) x planetGround` (ROBOBO:147).
  - The light ray goes along a fake direction: half the sun vector plus 0.1 on each axis, not normalised. This makes the low sun's path shallower ("make sunrise less blue").
  - The ray is not stopped by the planet. Instead, its density blows up below the ground, so the sky keeps sun until about `sun.y = -0.2` (about 11.5 degrees under).
  - Light-path transmittance is `exp(-od/ln2)`, which is 1.44x the optical depth (ROBOBO:89-91).
- **The moon term:** the same, at `-sun` and along the true direction (ROBOBO:150). Bliss's moon always stands opposite the sun.
- **Multiple scattering (the fake):** `scatteringAmbient += beta_s x airmass x visible x low_sun` (ROBOBO:156). It has no phase and no light transmittance. The term is later multiplied by `background`:
  - `low_sun = clamp(3(1 - |sun.y|) - high_sun, 1, 1.8)`, with `high_sun = clamp((sun.y + 0.6)^5, 0, 1) x 3` (ROBOBO:130-131). That is 1.8 from about 24 degrees up through dusk, and 1 at noon.
  - **`background = averageSkyCol x 4000/2`** (deferred.fsh:260). That is 0.75 of last frame's averaged sky in raw units, so the air is lit by the sky's own mean colour. This feedback is blue x Rayleigh-blue.
  - Its floor comes from `averageSkyCol >= minimumlight = (0.5, 0.75, 1.0) x min(MIN_LIGHT_AMOUNT, 0.0005)` (deferred.vsh:149-152). In raw units that is a background of at least (0.5, 0.75, 1.0) x 1.0. This floor is the night sky (section 6).
- **The sum:** `sky = scatteringSun x sunColorBase + scatteringAmbient x background + scatteringMoon x moonColorBase` (ROBOBO:162).
  - `sunColorBase = blackbody(Sun_temp=4300) x 250000` **(profile)**. Linear, that is (1.0, 0.668, 0.426) x 250000.
  - `moonColorBase = blackbody(15000) x 200` **(profile)** `x |4 - moonPhase|/4`. Linear, that is about (0.448, 0.599, 1.0) (settings.glsl:322-351, util.glsl:95-108).
  - **The same warm sun lights both Rayleigh and Mie.**

Darkening below the horizon (SKY_GROUND is on, settings.glsl:355; ROBOBO:98-104):
- `planetGround = exp(-100 x max(-5y + 0.1, 0)^2)` and `GroundDarkening = max(0.7 planetGround + 0.3, clamp(2 sun.y, 0, 1))`.
- Below the horizon, the sun and moon terms go to 0 and the ambient term falls to 0.3 (unless the sun is high).

**The baked texture.** deferred.fsh:247-267:
- 256x256, equirectangular. u is `atan(-x, -z)/2pi + 0.5`, and v is linear in y: `0.5y + 0.5` (sky_gradient.glsl:135-142, util.glsl:127-131).
- Recomputed every frame and **blended into history at 6 % per frame** (deferred.fsh:371-377), which averages out the blue-noise step jitter.
- A second 256x256 holds sky + clouds + VL fog, made at the same time (deferred.fsh:271-297). Reflections and the ambient average use it.

**How the drawn sky reads it** (composite1.fsh:953):
- `skyFromTex(dir)/30 x Sky_Brightness`, with `Sky_Brightness = 1`, bilinear.
- Stars, sun and moon are added under it (section 4), and the clouds are composited over.

**Ours.** look.wgsl:275-438, baked by sky.wgsl:1325-1333 (`fs_frame_look`, rows 1-64: 128x64 directions, rows bunched to the horizon as v = sqrt(elevation)). It is read through `sky_phys` (look.wgsl:422-438) from `sky_color` (look.wgsl:1247-1271).

What is the same:
- Rayleigh (5.8, 13.5, 33.1)e-6, scale heights 8 km and 1.2 km, Mie 3e-6, HG g 0.8.
- 10 view steps and 4 light steps.

What differs:
- Planet 6371e3 with a 100 km top (look.wgsl:300-301). The eye is at `eye.y + 100` (look.wgsl:368).
- Mie extinction is 1.11x scattering (look.wgsl:339).
- **Ozone uses real-air coefficients (0.65, 1.881, 0.085)e-6** (look.wgsl:304) in a tent 25 +/- 15 km, which is about **10x Bliss's**. The zenith green optical depth is about 0.03 against Bliss's 0.003. At twilight (about 19 air masses through the layer) ozone tints our upper sky purple-blue where Bliss's has almost none.
- **The physical Rayleigh phase** `3/(16 pi)(1 + mu^2)` (look.wgsl:376). It is 2-3x weaker than Bliss's and has no lean toward the sun.
- The sun's path uses `exp(-0.5 x OD)` along the true direction (`ATM_SUN_PATH`, look.wgsl:313/393). **`atm_toward` (look.wgsl:353-365) has no ground test, and `atm_density` clamps h < 0 to sea level** (look.wgsl:335-336). With the sun under the horizon, a high point's light ray crosses the planet at sea-level density and keeps about e^-3 of the red. Check whether this leaves a red afterglow that Bliss's planet cut-off would not have.
- **Hand layer 1: the sun's colour on the sky.** Rayleigh gets a white sun (`ATM_SUN = 1`, look.wgsl:309) that blends to 4300 K only with the sun low (`ray_k`, smoothstep 0.1-0.35). Mie gets the 4300 K colour at the white sun's luma, x 0.6 with the sun low (look.wgsl:404-412). Bliss puts 4300 K on all of it.
- **Hand layer 2: multiple scattering.** It is `(beta_r + beta_m) x T_view x T_sun x low x 0.25/(4 pi)`, in the sun's colour (look.wgsl:398, 412). Bliss's is lit by the sky's own average with no sun transmittance, so it is bluer and survives after sunset.
- **Hand layer 3: the scale.** `ATM_SCALE = 22` was fitted to the old palette (look.wgsl:318). Bliss's own number in our units is 250000/800/41 = **7.6 x the 4300 K colour (luma 0.72, so about 5.5)**. Our Rayleigh phase is 2-3x weaker, which roughly cancels the difference for the blue. **For Mie it does not: our sun glow is about 3-4x too strong against the blue.** That fits the history of the "blown white by the sun" fixes and the 0.6 Mie cut.
- No ground darkening, no jitter or history (not needed at this size), and no second sky + clouds texture.

**Verdict: partly.** The skeleton is Bliss's, but the parts that set the balance differ: the Rayleigh phase, a warm sun on all of it, the sky-lit multiple scattering, ozone, the fake sun path, the ground, and the scale.

**Rebuild:**
1. Use Bliss's Rayleigh phase `(0.1 mu + 0.28)/pi`.
2. Use a single sun colour, `sun_kelvin_linear()`, for Rayleigh, Mie and the multiple term. Delete `ATM_SUN`, `ray_k`, `warm`, `low_sun` and the 0.6.
3. Use Bliss's ozone (1.81e-7, 1.32e-7, 7.9e-9) with the 35 km exponential profile.
4. Light ray: Bliss's flattened direction `normalize(sun x 0.5 + 0.1)`, or keep the true one with a planet test and the extinction scaled as Bliss's. Either way, stop it at the ground. Delete `ATM_SUN_PATH`.
5. Multiple scattering: `beta_s x airmass x visible x low_sun x background`. `background` is the averaged sky of section 3, x0.75, with the floor (0.5, 0.75, 1.0) x k. Delete `ATM_MULTI`.
6. Use Bliss's `GroundDarkening`.
7. Derive `ATM_SCALE` from the units: 250000/800/41, about 7.6 with the 4300 K colour.
8. Planet radius and top: Bliss's 6731/110 km are harmless either way.

**Engine work:** none. Everything is in `fs_frame_look`. The feedback needs last frame's average, which is either one texel of the same `frame_look` target read as last frame's (the engine would have to keep the previous `frame_look` texture) or recomputed from 9 `atm_sky` calls in that texel. The second needs no engine work at all.

---

### 2. The sun's and the moon's light (sunColor, moonColor, temperature, handover)

**Bliss's method.** dimensions/deferred.vsh:156-168:
- `skyAbsorb` is the transmittance from `calculateAtmosphere(..., viewVector = sunVec, 25 steps)`: along the true sun direction, eye to the atmosphere's top, planet at R x 0.998, `exp(-OD)`.
- `sunColor = sunColorBase/4000 x skyAbsorb`, which is **blackbody(4300) x 250000 x T(path)/4000** **(profile)**. Both brightness and hue fall physically as the sun drops. With these coefficients, T at 10 degrees up is about (0.76, 0.54, 0.22), and at 0 degrees about (0.13, 0.014, 0).
- `moonColor = moonColorBase/4000`: blackbody(15000) x 200 x |4 - phase|/4 **(profile)**. It passes through no air, and is 1/1515 of the sun by luma at full moon.
- **Handover:** `sunVis = (clamp(sunElev, 0, 0.05)/0.05)^2` and `moonVis` the same for -sunElev. `lightSourceColor = sunVis > 1e-5 ? sunColor x sunVis : moonColor x moonVis`. The sun fades out over its last 2.9 degrees and the moon fades in over the first 2.9 degrees under.
- On land this becomes `DirectLightColor = lightCol/80` (composite1.vsh:52-53, composite1.fsh:908).
- Nothing else changes the colour: no per-hour keys and no tints. (`PER_BIOME_ENVIRONMENT` is false **(profile)**.)

**Ours.** The palette keys (look.wgsl:852-1025), baked into `frame_look` row 0 (sky.wgsl:1335-1341) and read by `palette_now` (look.wgsl:709-817).
- **Brightness:** hand keys for day (`sun_hue x 1.92`), warm (x2.0), golden (1.55, 0.88, 0.36), sunset, dusk (0) and night (0.17, 0.21, 0.30), blended by `sun.y` (look.wgsl:873-948). Morning sun gets an extra x(1, 0.88, 0.72) (look.wgsl:938-940).
- **Hue:** `sun_kelvin_linear() x sky_transmittance(sun.y)` (look.wgsl:740-744, 823-836). `sky_transmittance` is a closed form, Kasten-Young air mass on Bliss's sea-level coefficients, and **is rescaled to the palette's luma**. So the low sun keeps the hand brightness where Bliss's dims about 5-10x.
- **Stacked on top:**
  - The blue-hour dimming of the evening sun (`OPT_LOW_SUN_KEEP` 0.6, look.wgsl:795-797).
  - Rain x(1 - 0.9r) (look.wgsl:991).
  - Storybook moonlight tint (look.wgsl:1019-1020).
  - The chain's rain grey (look.wgsl:808-809).
  - `chain_night` x0.08 at night (look.wgsl:810-815).
- **Strength over the day:** `light_strength()` (look.wgsl:1211-1218). The sun runs smoothstep(-0.04, 0.05). The moon runs smoothstep(0.06, 0.2, -e) x `moon.w`, so it is full only at 11.5 degrees under and **leaves a dark gap from about -2 to -6 degrees**, which Bliss fills with the moon at once.
- **Moon colour:** the hand (0.17, 0.21, 0.30) x 0.08 x the moon factor. Brightness: `moon.w` = phase (0.5 + 0.5 cos(2 pi phase/8)) x how high it stands (client/mod.rs:3967-4002). Bliss's is linear in phase, |4 - p|/4, and ignores height.
- **Moon direction:** ours orbits with a phase lag (lib.rs:252-262). Bliss's is always -sun.

**Verdict: ours more complex.** The hue is roughly Bliss's, but the brightness is hand-made at every hour, and five or more layers sit on top.

**Rebuild:**
- In `fs_frame_look`'s texel 3: `p.sun = sun_kelvin_linear() x T_march(sun) x 250000/2133/41` (about 2.86 x 4300 K x T), with T marched by the same `atm_sky` code toward the sun (25 steps, planet-cut).
- Moon: `(0.448, 0.599, 1.0) x 200 x |4 - phase|/4/2133/41`, with Bliss's handover of 2.9 degrees each side.
- **Delete:**
  - The sun members of `day`, `warm`, `golden`, `sunset`, `dusk` and `night` (only the sky's own terms stay).
  - The morning warming, `sky_transmittance`, `OPT_LOW_SUN_KEEP` and storybook moonlight.
  - `chain_night` on `p.sun`.
  - Rain's x0.1 (Bliss scales the sun in rain only through the clouds' shadow, which is the clouds' area).
- Keep our moon orbit (a design choice), but light the land from `scene.moon` with Bliss's colour.

**Engine work:** none.

---

### 3. The ambient colour (AmbientColor, the sky's average as light)

**Bliss's method.** dimensions/deferred.vsh:116-152:
- **Sampling:** nine directions are averaged from the sky + clouds + fog texture and from the sky-only texture. Each is `pos = (0, 1, 0) + normalize(s) x (0.3183, 0.9)`, with s from a 3x3 pattern ((-1, -0.3), (0, 0), (1, -0.3), (+/-1, -0.5), (0, -0.5), (+/-1, -1), (0, -1)). The directions lie in the x-y plane, the sun's east-west plane, from about y = 0.1 to y = 1 (0.74 at most for the off-centre ones). So the average includes the sun's side and the far side.
- **The averages:** `avgClouds = 1.5 x mean/150`, and the same for `avgSky`.
- **AmbientColor:** `averageSkyCol_Clouds = max(normalize(avgClouds) x min(3 x luma(avgClouds), 2.5) x (1 - 0.7 rain), minimumlight)`. The hue is the sky's own, with clouds and fog in it. **The brightness is 3x the luma, capped at 2.5, so it is flat through most of the day.** `minimumlight = (0.5, 0.75, 1.0) x 0.0005`.
- **The sky's own average:** `averageSkyCol = max(avgSky x PLANET_GROUND_BRIGHTNESS(1), minimumlight)`. It feeds the march's `background` (section 1) and the rain sky (section 7).
- **Storage:** written to the LUT x `AmbientLight_RGB` (1, 1, 1), and temporally blended at 6 % (deferred.fsh:218-222, 371-377).
- **Use:** `AmbientLightColor = averageSkyCol_Clouds/30` in composite1.fsh:909, then `doIndirectLighting(AmbientLightColor x skylight, ...)` with the lightmap curve `(L^15 x 2 + L^2.5) x 0.5 x ambient_brightness x 0.7` (diffuse_lighting.glsl:65-76; composite1.fsh:1128). Clouds and fog take `skyGroundCol/30` (the texel straight down), and reflections take `skyCloudsFromTex(reflected)/30` (specular.glsl:246-248).

**Ours.** `p.ambient` comes from the hand keys (look.wgsl:873-948): day `fill_hue x 0.3` (0.85, 0.95, 1.2), warm, golden (0.26, 0.25, 0.30), sunset, dusk (0.03, 0.07, 0.25) and night (0.012, 0.017, 0.030). **It is not taken from the physical sky at all.** Stacked on top, in order:
1. `hdr_scale` 0.45-0.65 by canopy and sun height (look.wgsl:719-721).
2. The lightning add (look.wgsl:726-729, and again 1000).
3. `OPT_SKY_HUE` (off by default, look.wgsl:750-756).
4. The morning `TWILIGHT_RISE` 0.7-2.6 (look.wgsl:761-766).
5. The blue hour, `OPT_TWILIGHT` 0.5: ambient = deep blue x 0.5 (look.wgsl:770-790).
6. `OPT_EVENING_SHADE` 0.5 (look.wgsl:800-806).
7. The evening twilight's fixed (0.42, 0.52, 0.72) (look.wgsl:970-975).
8. Moon x1.6-6 (look.wgsl:977-993).
9. Rain grey (look.wgsl:989-993).
10. Storybook lavender (look.wgsl:1015-1021).
11. The chain's 40 % grey by day (`CHAIN_AMBIENT_GREY`, look.wgsl:808).
12. The chain's rain warm grey (look.wgsl:806-808).
13. `chain_night` x0.08 (look.wgsl:810-815).

That is about 13 layers. Then `light_on` uses the lightmap curve in the HDR world `/2` (look.wgsl:1811-1840), which is near Bliss's (the sky light belongs to another section).

**Verdict: ours more complex. This is the biggest single gap in this area.** The light in the shade has a hand colour at every hour instead of the sky's.

**Rebuild:**
- In `fs_frame_look`'s texel 4, evaluate `atm_sky` at Bliss's nine directions (9 marches of 10x4 steps, for one pixel; cheap).
- Then `ambient = max(normalize(avg) x min(3 x luma(avg), 2.5) x (1 - 0.7 rain), (0.5, 0.75, 1) x 0.0005)`, in Bliss's units /30, then /41.
- Feed the same average back as section 1's `background`.
- Clouds: Bliss's average includes them (grey-white), which is what `CHAIN_AMBIENT_GREY` imitates. Approximate it with the cloud cover mixing the average toward the clouds' grey, or later bake a coarse sky + clouds row.
- **Delete** layers 1, 3-7, 9-13 and the keys' ambient members.

**Engine work:** none for the sky-only average. A real sky + clouds average needs the soft clouds marched into a small texture before `frame_look`: about 1-2 days (engine pass order plus pack). Optional: a one-texel history blend (6 %/frame) needs `frame_look` kept across frames, about half a day of engine work.

---

### 4. Sun disc

**Bliss's method.** sky_gradient.glsl:3-5, called at composite1.fsh:946:
- `disc = DirectLightColor/0.0008821203 x 0.62 x smoothstep(cos(0.0093084 x 3.2), cos(0.0093084 x 1.8), cos theta)^3`.
- That is a disc of radius about 1 degree (full) to 1.7 degrees (edge), **at 703x the land's direct light, in that light's own colour**: 4300 K x the air's transmittance, so it is red and dim when low.
- No halo. The glow is only the Mie lobe in the sky texture (section 1).
- Cut by `atmosphereGround = 1 - exp2(-50 x clamp(y + 0.025, 0, 1)^2)` (composite1.fsh:931, 949). Clouds are composited over it (composite1.fsh:966-968). The bloom does the rest.

**Ours.** sky.wgsl:1225-1253:
- The engine's sun body is a 33-degree quad (lib.rs:240-244). The disc is `r < 0.12-0.15` of the half-size, a radius of about 2-2.5 degrees.
- Hand core tints mix white-gold and deep orange by `golden()`, plus a variant mix by `OPT_SUN_DISC`. The ring tint is (1, 0.52, 0.2).
- x `mix(3, 1.5, golden)` x **16 ("hot")**.
- A halo `exp(-6r)` and a wide glow `exp(-3r) x 0.9` over about 16 degrees.
- **Over that** is vanilla's sunset fan, `glow()` at 25 % in the open sky (sky.wgsl:1082-1094).
- Its brightness is not tied to `p.sun` or its colour.
- Before the physical sky, the old ten layers round the sun did this job. With `sky_phys` on they are in the dead path (look.wgsl:1418-1460: `sun_glow`, the Mie lobes, `OPT_SUN_GLOW`).

**Verdict: ours more complex.** The disc is bigger, hand-tinted and haloed twice, over a physical Mie lobe that already gives the glow.

**Rebuild:**
- In the sky pass, draw `p.sun x 703 x smoothstep(cos(1.71 deg), cos(0.96 deg), dot(dir, sun))^3 x atmosphereGround`, so the disc follows the light it casts.
- Drop the sun body's drawing in the open sky.
- **Delete** the core and ring tints, `golden()` in the disc, `hot`, the halo, the `wide` glow and `OPT_SUN_DISC`. Draw vanilla's fan at 0 when `sky_phys` is on.

**Engine work:** none (the pack can draw nothing for the body).

---

### 5. Moon and its phases

**Bliss's method.** sky_gradient.glsl:7-32, called at composite1.fsh:947 with `Color = DirectLightColor x 20`:
- **Shape:** `Shape = clamp(exp(1 - 1000(1 + dot(sun, dir))) - 1.5, 0, 25)`. That is a disc of radius about 2 degrees opposite the sun, peaking at e - 1.5 = 1.22.
- **Lighting:** a fake normal is built from `shape2 = exp(-10 x Shape)^0.15 x 255`. **Eight fixed light directions, one per `moonPhase`:** (-1, -0.5, 1), (-1, -0.5, 0.35), (-1, -0.5, 0.2), (-1, -0.5, 0.1), (1, 0.25, -1), (1, 0.25, 0.1), (1, 0.25, 0.2), (1, 0.25, 0.35). They are not taken from the sun.
- **Lit part:** `Shape x clamp(dot(n, L)/5, 0, 1.5)^5 x moonColor x 20`.
- **Grey floor:** `clamp(4 x Shape x (shape2/200)^2, 0, 1) x 0.004`.
- It hides the stars behind it (`occludeStars *= max(1 - 5 x Shape, 0)`). There is no texture.
- **Glow:** only the moon's Mie lobe in the sky texture (section 1), in moonColor (15000 K).

**Ours.** sky.wgsl:1124-1154 (`moon_body`) and 1220-1224:
- A lit sphere: the phase comes from the true sun direction, with the vanilla full-moon texture, a rim darkening and earthshine 0.05. By day it is pale and see-through.
- Size: the engine's 23-degree quad, with the picture filling half, a **radius of about 5.8 degrees (about 3x Bliss's)**, times `base:moon_size`.
- Colour: the body's own colour, not the moon's light.
- Glow: s_moon in `atm_sky` (white, `ATM_MOON = 8e-4`, which is Bliss's 1/1250 sun:moon ratio, look.wgsl:310, 399, 413), **plus a hand glow** `pow(mu, 24) x (0.05, 0.07, 0.12) x moon.w` (look.wgsl:1254-1257).
- The blood moon tint is ours (an event).

**Verdict: ours more complex, and better made** (a real phase from the sun, a texture). The visible gaps are its size (3x), a brightness not tied to the moon's light, and a white glow plus a hand glow where Bliss has one blue-white Mie lobe.

**Rebuild:**
- Keep the sphere.
- Scale its brightness by the moon's light (section 2) in place of `s.color`.
- Set `moon_size` toward a radius of about 2 degrees, if Ronald wants Bliss's look.
- Colour s_moon by (0.448, 0.599, 1.0), and delete the hand glow (look.wgsl:1254-1257 and 1465-1469).
- Optional: hide the stars behind the disc.

**Engine work:** none (the quad size is set in `lib.rs`).

---

### 6. Stars and the night sky

**Bliss's method:**
- **Stars** (stars.glsl:21-56, composite1.fsh:937-943):
  - The view direction is rotated in the world's x-y plane by `2.39996 - (worldTime + 24000 x worldDay)/24000` radians, which is **1 radian per day**.
  - `field = bilinear over x, y of clamp((hash13(floor(dir x 300)) - 0.99)/0.01, 0, 1)`. That is cells of about 0.19 degrees, and only about 1 % of them light up.
  - `star = exp(-20 x (1 - field))`.
  - `Background += star x 10 x clamp(-2 sun.y, 0, 1)`. They fade in from sunset and are full at 30 degrees under. They are white and do not twinkle.
  - x `atmosphereGround`. The sky texture is then **added** on top. Clouds are composited over.
- **The night sky:** the moon's single scattering (15000 K, x phase) plus the multiple term on the floor `background = (0.5, 0.75, 1.0) x 1.0` (raw units). This gives a dim Rayleigh-blue night that brightens toward the horizon by itself, even with no moon. It also takes `low_sun` 1-1.8.
- **Twilight (not a separate effect):** it falls out of the fake sun path (the sky still lit to about 11.5 degrees under), the sky-lit multiple scattering, the moon at -sun and the star fade.

**Ours:**
- **Stars:** 1500 engine star quads (lib.rs:238) turning with the sky. `star_look` (sky.wgsl:786-800) gives a per-star colour (orange, white, blue-white), size by brightness, a twinkle by height, and a fade by `night_sky_dark` = smoothstep(-0.28, -0.45) with `OPT_TWILIGHT` on (16-27 degrees under; look.wgsl:157-163).
- **Night sky floor:** with `sky_phys`, `palette` zenith/horizon x `smoothstep(-0.02, -0.18)` x `ATM_NIGHT_FLOOR` (look.wgsl:1249-1253). These are the hand night key (0.006, 0.010, 0.030)/(0.032, 0.044, 0.085) x `chain_night` 0.08, **plus the hand blue-hour colours.**
- **Twilight colours in the sky:**
  - `OPT_TWILIGHT` deep (0.08, 0.3, 1.0) on zenith and horizon (look.wgsl:770-783).
  - The evening blue (0.72, 0.86, 1.1)/(0.30, 0.48, 1.0) (look.wgsl:956-962).
  - The twilight blue (look.wgsl:966-975).
  - The dusk key's pink horizon.
  - All of these go through that floor term into the drawn physical sky at dusk.
- **Night grade:** the sky also takes `display`'s night rods (`SKY_ROD_SHARE` 0.7, sky.wgsl:1179). That belongs to the grade section.
- **Ours only:** the Milky Way, aurora, meteors, shooting stars and the fireball (sky.wgsl:803-966). Keep them; they are ours by design.

**Verdict: ours more complex.** The stars are fine (ours have more character). The night and twilight sky is a hand floor and hand blue-hour colours added onto a march that lacks Bliss's sky-lit floor term.

**Rebuild:**
- Get the night and twilight sky from section 1's multiple term with the (0.5, 0.75, 1.0) floor and the moon in 15000 K.
- **Delete** `ATM_NIGHT_FLOOR`'s palette add (look.wgsl:1249-1253) and the zenith/horizon/glow members of the night, dusk, sunset and golden keys. Also delete the evening-blue and twilight-blue sky overrides (look.wgsl:951-976), `OPT_TWILIGHT`'s sky part and `chain_night` on the sky.
- Star fade: Bliss's `clamp(-2 sun.y, 0, 1)` x a brightness of 10/800 relative to the sky, letting the physical sky hide them early, in place of the -16 to -27 degree window.

**Engine work:** none.

---

### 7. Rain sky

**Bliss's method** (deferred.fsh:262-264): `sky = mix(sky, averageSkyCol x 4000 x (0.7 skyAbsorb + 0.3), (1 - exp(-5 x clamp(0.9 - y, 0, 1)^2)) x clamp(1 - max(eyeY - cloudHeight, 0)/200, 0, 1) x rain)`. The sky goes toward its own average, tinted by the air's transmittance along the view, most at the horizon. It fades out once the eye is above the cloud layer.

**Ours** (look.wgsl:1258-1267): `mix(c, luma(c) x (0.98, 0.97, 0.9) x 0.35 - 0.12 x storm x luma, rain x (1 - 0.6 smoothstep(-0.1, 0.9, y)))`. It is a hand warm slate at 0.35. The palette's rain greys (look.wgsl:983-995) also feed the haze and clouds.

**Verdict: partly.** Same idea, hand colour.

**Rebuild:** use Bliss's formula with the section 3 average. Delete the 0.35 slate and the storm term.

**Engine work:** none.

---

### 8. Sky reflected and used elsewhere

**Bliss's method:**
- Water and metal reflections: `skyCloudsFromTex(reflected)/30`, the sky + clouds texture (specular.glsl:246-248; all_translucent.fsh:785).
- Far border fog: `skyFromTex(dir)/30` (composite1.fsh:1319).
- Clouds and fog lit by `skyGroundCol/30` (the texel looking straight down) and `averageSkyCol_Clouds` (deferred.fsh:289-292).

**Ours:** `sky_color()` is used everywhere (world.wgsl:763, 812, 841, 1157, 1687; pathtrace.wgsl:357, 370; the haze, look.wgsl:2938-3159), so it reads the physical sky. **But** clouds, cirrus, particles and the haze still light from `p.zenith`, `p.horizon`, `p.glow` and `p.ambient` (sky.wgsl:457, 471, 727, 1072-1077; look.wgsl:2160, 2438-2448, 2553, 2834; particle.wgsl:67). Those are the hand keys.

**Verdict: partly.**

**Rebuild:** once sections 1 and 3 land, define `p.zenith = atm(up)`, `p.horizon = atm(horizon toward the side away from the sun)` and `p.glow = atm(toward the sun, low)`, all from the same march, so every reader follows the physical sky. Then `palette_made`'s keys can go entirely (look.wgsl:852-1025, about 170 lines). The dead hand sky (look.wgsl:1272-1484, about 210 lines) and its constants (look.wgsl:501-573: `SUN_GOLD`, `SKY_AWAY*`, `SKY_DAWN_LINE`, `SKY_LOBE`, `SUN_BAND*`, `SUN_ROUND_SEEN`, `SUN_MIE_BROAD_CUT`, `SKY_BAND_BLUE_SEEN`), `dawn_sky()`, `bliss_low_sun()` and `sun_glow()` go too. Water's reflection of clouds is the clouds and water sections' business.

**Engine work:** none.

---

### 9. Lightning and rainbow

**Bliss's method.** lightning_stuff.glsl:9-61; shaders.properties:399-407:
- `lightningEffect = (77, 153, 255) x lightningFlash`. The flash is 1 while a bolt entity exists (Iris), or is detected from a grey sky (OptiFine).
- It is a **point light at the bolt**: `exp(-10 x (1 - max(1 - d/300, 0)))` on land (x NdotL x lightmap.y^10), 400 blocks in the clouds, and `exp(-15 x ...)` in the fog.
- No bolt is drawn by the shader (vanilla's bolt entity is). There is no rainbow.

**Ours:**
- A global flash: `lightning_flash()` adds (0.5, 0.55, 0.7) to ambient (look.wgsl:726-729, 1000) and to zenith and horizon (look.wgsl:1001-1002).
- A drawn bolt and a cloud glow (look.wgsl:1078-1209).
- A rainbow (sky.wgsl:968-1009).

**Verdict:** different, and ours is extra. Bliss's flash is positional and blue (0.30, 0.60, 1.0).

**Rebuild (low priority):** make our flash a point light at the bolt with Bliss's falloff (we have `lightning_toward` and the distance), in Bliss's blue. Drop the zenith and horizon adds (the march will not carry them). Keep the bolt and the rainbow (features).

**Engine work:** the flash's world position. The scene has a direction and distance now, so this is about half a day if a position is needed.

---

### Ranked: the biggest visual gaps in this area

1. **The land's light is not the sky's** (sections 2-3). Bliss's ambient is the sky (+ clouds) average's own hue x min(3 x luma, 2.5), and its sunlight is 4300 K x 250000 x the marched transmittance, dimming physically when low. Ours are hand keys with about 13 ambient and 6 sun layers per hour. This touches every surface at every hour, most of all golden hour and twilight.
2. **The march's balance** (section 1). Ours has the physical Rayleigh phase (Bliss's is 2-3x and leans to the sun), a white sun on Rayleigh plus a 0.6 Mie cut (Bliss: one 4300 K sun), a sun-coloured multiple term (Bliss: lit by the sky's own average, bluer, lasting past sunset), 10x ozone, no ground darkening, no planet cut on the light ray, and a hand `ATM_SCALE` 22 (Bliss's units give about 7.6 x 4300 K). The net result is a Mie glow about 3-4x too strong against the blue.
3. **The twilight and night sky** (section 6). Hand blue-hour colours and a palette floor are added onto the march, where Bliss gets them from the (0.5, 0.75, 1.0) sky-lit floor and a 15000 K moon. Stars fade on our own window.
4. **The sun disc** (section 4). It is 2x the size, hand-tinted, x16, with two halos and vanilla's fan over the physical lobe. Bliss's disc is the sunlight's own colour x 703, with no halo.
5. **Rain sky** (section 7): a hand slate where Bliss mixes toward the sky's average x (0.7T + 0.3).
6. **Small:** the moon is 3x Bliss's radius, its glow white plus a hand glow; the moonlight handover leaves a gap at -2 to -6 degrees; lightning is a global flash, not positional.

**Net deletions once 1-4 land:** the palette's keys and per-hour fixes (look.wgsl:852-1025 and much of 709-817), the dead hand sky and its constants (look.wgsl:501-573 and 1272-1484), and the sun body's tints and halos. That is about -450 lines. There is no engine work except optional history and sky + clouds averaging, about 1-2 days.

---

## Lighting

Read 2026-11-25. Bliss = `chiacraft-archive/references/shaders/bliss/Bliss-Shader-git/shaders` (method and numbers only, "© Chocapic13, all rights reserved"); paths below are relative to that `shaders/` folder. The main lighting pass is `dimensions/composite1.fsh` (world0/composite1 includes it); its inputs are made in `dimensions/deferred.vsh/.fsh` (sun, moon, sky-average colours, exposure) and `dimensions/composite.fsh` (SSAO and the PCSS blocker search, written to colortex3/14). Ours = `mods/base/shaderpack/look.wgsl` (L), `world.wgsl` (W), post passes in `mods/base/assets/shaders/post{,_low}/`.

Ronald's profile (`bliss_matrix/pack_settings.txt`) over Bliss's defaults. The matrix refs ran with **vanilla textures, no LabPBR pack** (README: Bliss_v2.1.2, mc-dh, no resource pack named). That matters a lot here: every LabPBR-driven feature in the profile reads an empty specular map:

| Profile setting | What it does with no LabPBR pack |
|---|---|
| SSS_TYPE=3 (LabPBR only) | `all_solid.fsh:531`: SSS amount = specular.b = 0 → **no leaf/plant SSS at all, neither sun nor sky** |
| LabSSS_Curve=1.3 | multiplies a 0, no effect |
| EMISSIVE_TYPE=3 (LabPBR only) | `all_solid.fsh:514`: emission = specular.a = 0 → **no emissive boost**; lamps glow only by their own block light |
| Emissive_Curve=0.5 | no effect |
| MATERIAL_AO | `all_solid.fsh:477`: albedo × (normal.b·0.5+0.5); default normal map b = 1 → no effect |
| Sun_specular_Strength=10 | **does act**: a rough GGX sun lobe on every surface (F0 0.02, roughness 1), see §17 |
| TRANSLUCENT_COLORED_SHADOWS | acts (vanilla glass, water, ice, leaves? no: only translucent pass) |
| Sun_temp 4300, sun_illuminance 250000, moon_illuminance 200 | act |

(Assumption: Iris's default for a missing `_s` map is all zeros. If it were not, SSS_TYPE 3 would still be 0 from the b channel unless ≥ 65/255.)

So the earlier report's line "Bliss: leaf SSS by blocker depth, sky SSS" is true of Bliss's defaults, **not of Ronald's Bliss**. In the reference shots leaves are lit like any block: max(sun·NdotL·shadow, 0) + sky light.

---

### 1. Direct light colour and strength (sun, moon)

**Bliss's method**
- `deferred.vsh` (main, after the sky averages): sun colour = sunColorBase / 4000 × the atmosphere's transmittance along the sun's path (`calculateAtmosphere`, 25 steps, the same Rayleigh/Mie/ozone model as the sky texture). sunColorBase = blackbody(Sun_temp) × sun_illuminance (`settings.glsl:343`). Moon = blackbody(Moon_temp) × moon_illuminance × phase/4 / 4000, phase brightness = |4 − moonPhase|/4 (`settings.glsl:345-348`), no atmosphere.
- Which is on: sunVis = (clamp(sunElevation, 0, 0.05)/0.05)², moonVis the same on −sunElevation; light = sunVis > 0 ? sun·sunVis : moon·moonVis.
- `composite1.fsh:908`: DirectLightColor = that / 80.
- blackbody (`util.glsl:95`): t = T^-1.5, lt = ln T; r = 220000·t + 0.5804; g = 0.3923·lt − 2.4455 (T > 6500: 138039·t + 0.7216); b = 0.7608·lt − 5.6808; clamp 0..1; sRGB→linear.
- Numbers (profile): 4300 K → linear **1.0, 0.668, 0.426**; ×250000/4000/80 = **0.78 × transmittance** (noon ≈ 0.94, 0.87, 0.72 → about 0.73, 0.45, 0.24, a strongly orange sun). Moon 15000 K → linear 0.45, 0.60, 1.0; ×200/4000/80 = 6.3e-4 × phase: **1/1250 of the sun** before the sun's absorption. Defaults would be 6300 K (near white) and 128000.

**Ours**
- L:852-1026 `palette_made`: five hand keys (day/golden/sunset/dusk/night), each with its own sun RGB and brightness (day `sun_hue × 1.92`, L:873; golden 1.55 0.88 0.36; sunset 1.50 0.82 0.34; dusk 0; night 0.17 0.21 0.30), blended by sun height; `OPT_SUN_WARMTH`, a morning warm-up (L:~916 `p.sun *= mix(1, (1,0.88,0.72), morning)`).
- L:735-739: in the HDR world the hue is replaced by `sun_kelvin_linear()` (1.0 0.668 0.426, L:823) × `sky_transmittance` (closed-form Kasten–Young air mass with Bliss's coefficients, L:831), **at the palette's hand-keyed luma**.
- Stacked on top: blue-hour sun dim (`OPT_LOW_SUN_KEEP`, L:778), rain greying (L:795-800, chain), rain ×(1−0.9r) (L:~993), chain-night ×0.08 (L:802-808), `light_strength()` ramp smoothstep(−0.04, 0.05, e) (L:1211) and at night moon.w × blood-moon cut.
- Moon: own lagging orbit (`mods/base/src/lib.rs:250`), strength = phase × height (`scene.moon.w`), night key colour 0.17 0.21 0.30.

**Verdict: partly.** Hue at noon now matches; brightness at every hour is hand-keyed (five keys + four per-hour fixes), where Bliss's falls out of one blackbody × one transmittance × sunVis. Rebuild: p.sun = blackbody(4300)·0.78·T(path)·sunVis (×BLISS_UNITS scale), moon = (0.45,0.60,1.0)·6.3e-4·phase·moonVis; delete the sun members of the five keys, the morning warm-up, the low-sun keep, the chain rain greying of the sun (the transmittance through the deck should be the cloud shadow's job). If `look_sky_phys` lands, reuse its transmittance (the same march) instead of the closed form. Pack only, ~-60 lines. Our moon's lagging orbit is a design choice; keep it, but drop the height factor (Bliss's moon light depends on phase only).

### 2. Facing term (NdotL)

**Bliss** `composite1.fsh:1005`: NdotL = clamp((dot(n, L)·255 − 15)/240, 0, 1) (a cut of grazing light below dot 0.059), with the bumped normal; the shadow lookup uses the flat normal's version (:1007). No wrap.

**Ours** L:1752 `sun_facing`: (dot + w)/(1 + w), w = 0.45·(1 − smoothstep(0.05, 0.3, |sun.y|)), a wrap over tops at low sun ("golden hour warm ground"); ×mix(0.75, 1, ao) (L:1866).

**Verdict: ours more complex.** Replace with Bliss's line; delete the wrap (it lights faces Bliss leaves dark at golden hour) and the AO on the sun (§13). -6 lines.

### 3. Sun shadow map: size, reach, distortion, bias, leak fix

**Bliss**
- `settings.glsl:125-126`: one map, 2048², shadowDistance 128; `Shadow_Params.glsl`: radial log distortion, k = 1.8, d0 = 0.04, d1 = 0.61, a = e^d0, b = (e^d1 − a)·150/128; distort = ln(|xy|·b + a)·k. At the centre ≈ 14× magnified (≈110 texels per block, my estimate), at the edge ≈ 6.5 texels per block. Depth scaled ×1/6.
- Bias (`Shadows.glsl`, "GriAndEminShadowFix", :4-26): offset the lookup along the **flat** normal by max(1 − max(1 − dist/128, 0), 0.05) × a small low-res boost; where sky light < 0.1 replace that with a snap toward the block's centre (0.25 − 0.5·fract(pos + bias·0.25)) so caves don't leak at block seams.
- Fade (`composite1.fsh:1016-1019`, OPTIMIZED_SHADOW_DISTANCE 1): smoothstep of min((1 − d/(128+16))·5, 1): out over the last ~20 %.

**Ours** L:1498-1600: two orthographic maps (engine `client/mod.rs:4597` SHADOW_REACH 80, depth 400; far map `core:shadow_far`, 400+), 2048 default (`core:shadow_size` 1024-4096), ≈12.8 texels per block everywhere in the near map. Bias: normal offset 1.2·texel·(2 − ndl) plus slope bias (0.03 + 0.5·texel·slope)/depth; hand-over to the far map; `low` fade with the light under 2-6° (L:1532). Extras: plant lookups lifted 0.7 block toward the sun (W:409, 477), edge-on plant quads discarded from the caster pass (W:487-496).

**Verdict: different, close in effect.** Ours is uniformly coarser near the player and reaches further. Not a big visual gap by itself. Optional engine work (medium): a distorted single map like Bliss's (shadow vertex shader + this lookup), which would also make the plant lift and edge-on discard unnecessary (they exist to hide texel-scale stripes). The block-centre snap for sky < 0.1 is the piece we lack and need for §8: ~10 lines in `shadow_in`.

### 4. Shadow filter (PCSS)

**Bliss** `composite.fsh:477-540` (blocker search, quarter of the work) + `composite1.fsh:620-690` `ComputeShadowMap`:
- Search: 4 spiral taps (VPS_Search_Samples) of raw depth within Max_Shadow_Filter_Radius 30; count blockers; average blocker depth; filter radius = clamp((z − blocker)·1500, 0, Max_Filter_Depth 35)/35 · (30 − 5) + 5. Leaves facing away get min radius 50 (only with SSS > 0, so not in the profile).
- Filter: 13 taps (SHADOW_FILTER_SAMPLE_COUNT) on a golden-ish spiral (θ = i·2.26π, r = sqrt(i/N)), radius ×0.3, hardware compare (sampler2DShadow), offset scale radius·distort·d0·k/res. My estimate: 5 → about 0.014 block, 30 → about 0.08 block of penumbra at any distance. Upsampled with a 4-tap depth-aware filter (`BilateralUpscale_REUSE_Z`, :545).
- So Bliss's shadows are **sharp**, softening a little with blocker distance; plus TAA.

**Ours** L:1538-1567: fixed 3×3 of a 4×4 with "nearest of each pair" crack closing (≈1.5 texels ≈ 0.12 block). Far map 2×2 bilinear.

**Verdict: partly (no blocker term), small visual gap.** The earlier report's idea that a blocker-scaled filter would replace the canopy cover/lift fixes does not hold: Bliss's penumbra is tiny. Leave the filter; low priority. If done: 4-tap blocker search + 13-tap spiral, ~40 lines, cost +~0.2 ms.

### 5. Contact shadows (screen-space ray toward the sun)

**Bliss** `composite1.fsh:398-435` `SSRT_Shadows`, used at :1248-1258 (SCREENSPACE_CONTACT_SHADOWS on by default): 16 steps in screen space toward the sun, step = 3 pixels (6 in DH range), dithered start; a hit when the depth buffer is in front within a 3.5 % linear-depth threshold. Shadows = min(shadow map, SSRT) **everywhere**, near and far (it also gives far hills some shadow past the map).

**Ours** none in raster. `post/contact.wgsl` + `post_low/contact.wgsl:42-92` is an **AO**, not a shadow: 8 directions × 2 distances within 1.2 blocks, "rise above the surface" test, darkening up to 0.5, fading 24-64 blocks, multiplying the **whole** colour (sun, sky, lamps, already-applied fog). The RT path (`rt_sun_soft`) is the only true contact ray.

**Verdict: missing (and ours' "contact" is a different thing).** Rebuild: a 16-step screen-space sun ray in the deferred/lit pass (needs the depth buffer in the lit pass → if our world pass is forward, it goes as a post pass writing a sun-visibility mask the world pass can't use; realistic option: a pre-pass at half size after depth, like `post_low`, read by the forward world shader next frame or by a deferred split). Engine work: medium-large (a depth pre-pass or deferred lighting). Delete `contact` as AO once §13's SSAO exists.

### 6. Coloured translucent shadows (profile: on)

**Bliss** `composite1.fsh:640-672`: three maps: shadowtex0 (all), shadowtex1 (opaque only), shadowcolor0 (translucent colour+alpha). Per tap: depthDiff = (clamp((opaque − z·0.6)·2, 0, 1))²; opaque = max(tex0 compare, depthDiff); tint = max(normalize(rgb + 1e-4), max(opaque, 1 − α')) · max(α', backface·(1 − depthDiff)), α' = (1 − α⁵)^0.2; final tint = mix(tint, 1, opaque·depthDiff); DirectLightColor ×= mix(1, avg tint, fade). Hue only (normalised), not darkness.

**Ours** none in raster (`sun_shadow_near` notes RT keeps a colour "for later").

**Verdict: missing.** Engine work: a second shadow pass for translucent geometry into an RGBA target + its own depth (medium, 1-2 days); shader ~25 lines. Visible only near stained glass, water and ice; not in the matrix shots' main gaps.

### 7. Past the shadow's reach

**Bliss** `composite1.fsh:1000, 1048`: fallback mask = clamp((sky − 0.8)·25, 0, 1) (full sun at sky ≥ 0.84 after the lightmap offset, none ≤ 0.8), mixed in by the fade of §3; for water pixels the leak fix instead. Then min with SSRT (§5) and × cloud shadow.

**Ours** W:425-440 `sky_sun_mask` smoothstep(0.86, 0.94, sky) × cloud shadow, used only when `OPT_SHADOW_REACH` > 0 (fade reach·0.8 → reach+16); otherwise the far map to 400+ blocks.

**Verdict: same when OPT_SHADOW_REACH = 128** (thresholds a hair different: use clamp((sky−0.8)·25) on Bliss's offset lightmap). Keep the far map as our own extra if Ronald wants; for the 1:1 match the reach option should be on by default in the Bliss look.

### 8. Sky light gating of the sun (the cave mouth)

**Bliss**: **none.** Inside the map, the sun is shadow map × NdotL × (SSRT) × cloud shadow; sky light plays no part (OLD_LIGHTLEAK_FIX is off, `settings.glsl:678`). Leaks are stopped by the map itself plus the bias of §3. The eye-brightness term `lightLeakFix` = clamp((eyeBrightnessSmooth.y/240 + sky)², 0, 1) (:905) multiplies only the sun SSS (zero in the profile) and the under-water depth estimate.

**Ours**: three stacked sky-light gates on the sun:
1. W:470-484 `sun_shadow_mapped`: shadow = mix(smoothstep(0.75, 1, sky), **min(map, smoothstep(0.3, 0.8, sky))**, trust) (canopy shifts it to 0.15-0.5).
2. L:1863-1865 `light_on`: sun × **open = smoothstep(0.5, 0.95, sky)** (canopy: 0.25-0.55).
3. W:443-468 RT path: cover smoothstep(0.3, 0.8, sky), and "deep under cover: no sun to trace".
At sky 0.6 (level 9) that is 0.65 × 0.04 ≈ **3 % of the sun**; at 0.8, 1 × 0.5.

**Verdict: ours more complex and the main cause of the cave gap.** Delete gates 1-3 (inside the map's trust); add Bliss's block-centre snap where sky < 0.1 (§3) and keep the map's own occlusion. Test the leak cases the gates were added for (traced shadow gaps, caves under hills, forest floors) on the matrix cave and forest shots. Pack only, about −15 lines.

### 9. Sky light: the lightmap curve

**Bliss** `composite1.fsh:810`: lightmap = min(max(lm − 0.05, 0)·1.06, 1) (both channels). `diffuse_lighting.glsl:65-75`: curve = (L^15·2 + L^2.5)·0.5; indirect = ambient × facing × curve × ambient_brightness (1.0) × 0.7, + minimum light (§12).

**Ours** L:1806-1816: (L^(15·steep)·2 + L^(2.5·steep)) / mix(3, 2, bliss_sky); bliss_sky = smoothstep(−0.1, 0.02, sun.y) (by day /2 like Bliss, by night /3); `OPT_SHADE` exponent knob; no −0.05·1.06 offset; no ×0.7.

**Verdict: same curve, plus a day/night divisor.** Rebuild: Bliss's offset + curve + 0.7 at all hours; delete the sliding divisor (the night's brightness belongs to the moon-lit sky colour, §11, and the exposure). −4 lines.

### 10. Sky light: facing (hemisphere) term

**Bliss** `composite1.fsh:1113-1121`: dirY = (n / (|nx|+|ny|+|nz|)).y·1.5 (bumped normal); grass = 1.5; skylight = max((flat n.y·0.5 + 0.5)^0.1 + dirY, **0.2 + (1 − L)·0.8**); with SSAO: min(skylight, mix(0.95, 2.5, (1 − (1 − ssao)^0.5)²)). Gives tops 2.5, sides 0.93, bottoms 0.2 in the open, but **bottoms and everything else rise toward ≥ 1 where sky light is low** (in caves the light comes from all round), and occluded spots lose the top boost.

**Ours** L:1817-1829: hemi = mix(0.75 + 0.25n.y, 0.6 + 0.4n.y, rain); by clear day mixed to tops 2.5/sides 0.93/bottoms 0.2 by n.y (linear, not the L1-normalised form); rain keeps the old split. Plus: cloud shadow on the sky light ×mix(0.6, 1) (L:1835), canopy under-leaves tint and the sun-through-canopy fill 0.045 (L:1841), golden-hour glow fill (L:1848-1852), `under_leaves` (L:659).

**Verdict: partly, with four stacked layers.** Rebuild: Bliss's formula exactly (incl. the low-L floor and the SSAO cap once §13 exists), same at all weathers; delete the rain hemisphere, the cloud-shadow-on-sky, the canopy sun fill, the golden glow fill and `under_leaves`/`under_tint` (Bliss's forest floor is just curve × facing × AO). About −30 lines.

### 11. Ambient (sky light) colour

**Bliss** `deferred.vsh` (main): average of 9 directions of the **sky-with-clouds** texture in the upper sky (up tilted by (0.318, 0.9)·normalised offsets), ×1.5/150; then ambient = normalize(avg) × min(luma(avg)·3, 2.5) × (1 − 0.7·rain), floored by (0.5, 0.75, 1)·0.0005; `composite1.fsh:909`: /30. So: the hue is whatever the sky and clouds are (with a 4300 K sun lighting the air and the clouds, a pale, near-neutral blue-grey), and the brightness is capped (flat by day). Night: the moon-lit sky's average, same cap.

**Ours** L:709-817: hand key ambient per hour (day `fill_hue × 0.3`, fill_hue cool 0.85 0.95 1.2, L:872-873), then: `OPT_SKY_LIGHT`/`hdr_scale` 0.45-0.65 by hour and canopy (L:717-719), lightning add (L:724), `OPT_SKY_HUE` to the zenith/horizon hue (L:744-750), morning twilight rise ×2.6 (L:756-762), blue-hour deep blue (L:766-785), evening shade (L:788-793), chain: 40 % greyed (`CHAIN_AMBIENT_GREY`, L:795), rain warm grey (L:798), chain night ×0.08 (L:802), night ×1.6-6 by moon (L:979), rain/storm greying (L:~990-994).

**Verdict: ours far more complex, and still bluer than Bliss's** (cave shade: Bliss 37 31 25 … 85 75 67, R > B; ours 57 61 72 … 95 100 111, B > R; lkmxM1 vs ref). Rebuild: ambient = normalize(avg of 9 sky-with-clouds samples) × min(luma·3, 2.5) × (1 − 0.7 rain) / 30 (in our units), from the physical sky (`atm_sky`, which already lights the air with the 4300 K sun) and the cloud layer's colour. Delete every line listed above except lightning. Needs the physical sky (`look_sky_phys`) on and a cloud term in its average: pack work, about −120 lines net; one frame-look texel more.

### 12. Minimum light

**Bliss** `composite1.fsh:924`: MinimumLightColor = 1 + 0.7·dot(n, view dir) (0.3 on faces toward the eye … 1.7 away); `diffuse_lighting.glsl:73`: indirect += that × max(MIN_LIGHT_AMOUNT·0.01, nightVision·0.1) = **0.01, neutral white**, then × AO with the rest. Under water (eye in water) it is 10.

**Ours** L:1875-1876: floor = max(brightness_curve(gamma setting)·0.25, mix(0.006, 0.03, dimension light)) × **(0.85, 0.9, 1.0)** × ao; also in `light_round`.

**Verdict: partly (blue tint, no view term).** Use Bliss's neutral 0.01 × (1 + 0.7·n·v) × AO; keep the Brightness setting as a multiplier. −2 lines. Part of the cave's blue cast.

### 13. Ambient occlusion

**Bliss**
- Vanilla AO: `all_solid.vsh:199` occlusion o = 1 − vertex colour alpha; `composite1.fsh:1183`: curve (1 − o²)^5.
- SSAO (`composite.fsh:239-303`, indirect_effect 1 = default): 7 taps on the CleanSample spiral (θ = i·2.26π), radius ∝ projection/(3·(1 + clamp(z²/50, 0, 5)))·0.3, max distance² ∝ z²·…/mix(4, 50, …); occlusion += clamp(dot(dir, n), 0, 1)·(1 − d²/max²); ao = max(1 − occ·AO_Strength(1)/n, 0); 4-tap depth-aware blur in composite1.
- Combined: AO = **min((1 − o²)^5, ssao^6)**, applied to the **indirect only** (sky + block light + minimum), never to the sun (`AO_in_sunlight` is defined but unused).

**Ours** W:4 vertex AO table 0.45, 0.64, 0.82, 1.0 (linear factor); applied in `light_on` to sky ×ao, sun ×mix(0.75, 1, ao), lamps ×mix(0.5, 1, ao), floor ×ao. Post: `contact` (AO-like, whole colour, ≤ 0.5, 1.6-64 blocks, §5) and `far_ao` (24 blocks → haze end, guesses sunlit faces from the maps, whole colour divided by the haze's share; `OPT_AO_POWER`). `OPT_AO_TEST` knob.

**Verdict: different.** Rebuild: one half-size 7-tap SSAO (we have the half-size `post_low` machinery) whose result the lit pass reads, AO = min((1−o²)^5, ssao^6) on sky + lamps + minimum only; o from our corner levels (0, 1/3, 2/3, 1 × Bliss's ~0.8 range: check vanilla's vertex alpha steps). Delete AO on the sun, `contact` and `far_ao` (Bliss has nothing past its SSAO's screen reach; DH land gets SSAO too). Engine: the lit pass must read a screen texture made before it → a depth pre-pass or deferred split (shared with §5). Medium-large; the biggest structural item in this area.

### 14. Leaves and plants: subsurface light

**Bliss (Ronald's profile)**: **none** (SSS_TYPE 3 with no LabPBR, see top). Leaves: max(sun·NdotL·shadow, 0) + sky light × facing × AO. Plants (grass mask 0.60): sky facing forced to the top value 1.5+… (`composite1.fsh:1115`), otherwise like blocks. (Bliss's default SSS_TYPE 1 method, for the record: sun SSS by blocker depth exp(−7·(1 − …)), absorption exp(−max(luma − albedo·(1, 1.1, 1.2), 0)·(20 − 19·depth)), phase 1 + 6·(2^(−25√(1−μ)) + 0.5e^(−10(1−μ))), × density^LabSSS_Curve, combined as max(NdotL·shadow, SSS); sky SSS from the SSAO pass. Not in the refs.)

**Ours**: `leaf_sss` (W:525, off by default, `OPT_LEAF_LIGHT`), `leaf_through` for plants and far crowns (L:1797, W:571; back-light pow 4 ×1.6 + 0.12), extra sun on leaf faces ×`OPT_LEAF_SUN` (W:551-558), the non-HDR back-light + hold-back + desaturation (W:573-590), the lime/yellow cuts in `shade_soft_lit` (W:650-670), `OPT_LEAF_HOLES`.

**Verdict: ours more complex than the reference, which has none.** For the 1:1 match: delete `leaf_through`, the leaf-sun extra, the lime cuts and hold-backs; leaves and plants lit like blocks, plants with the top's sky facing. Keep `leaf_sss` as an off-by-default option (it is Bliss's own SSS type 1 if Ronald ever wants it). About −60 lines. Big effect on forest shots (back-lit crowns glowing where Bliss's are dark).

### 15. Block light (torches) and held light

**Bliss** `diffuse_lighting.glsl:24-63`: curve = (1 − √(1 − L))²·2 (L = 1 → 2, 0.5 → 0.17, 0.25 → 0.036); colour (TORCH_R/G/B) **1.0, 0.75, 0.65**, used as linear, one colour; × autoBrightness = mix(1, 30, clamp(e^(−10·exposure))) (bright views ≈ ×6, dark ×1, my estimate from its exposure curve) — so torches stay visible by day; **added** to the sky light (no max). LPV coloured light off by default. Held light (`all_solid.fsh:325`): lightmap = max(lm, 0.9·(1 − d/10)^1.5) (HANDHELD_LIGHT_RANGE 10), through the same curve. × AO with the indirect.

**Ours** L:1732-1750 `block_light`: engine curve L/(4 − 3L), ^0.9, per-channel RGB (coloured light), white light warmed from (1, 0.56, 0.26) to (1, 0.82, 0.62) by level, ×1.4, flicker; L:1786 `block_by_day` cuts it to 0.25 in daylit open air; × mix(0.5, 1, ao); held light L:1761 (level − d/15, facing 0.5-1, pool fade 2.5-10 blocks), max with placed light.

**Verdict: partly; ours more complex; opposite day rule.** Rebuild: Bliss's curve, (1, 0.75, 0.65) for white light (coloured light stays ours), × autoBrightness from the exposure instead of `block_by_day`, added; held light = max(level, 0.9·(1 − d/10)^1.5). Delete the warm ramp, ^0.9, ×1.4 and the day cut; flicker optional (ours only). −15 lines. Night-torch shots: Bliss's pool is small and steep, ours wide.

### 16. Emissive blocks

**Bliss (profile)**: none (EMISSIVE_TYPE 3, no LabPBR); a lamp face is lit only by its own block light (curve 2 × autoBrightness). (Method, for the record: `composite1.fsh:438-446` mix(lighting, albedo·Emissive_Brightness·mix(5, 100, e^(−10·exposure)), emission^Emissive_Curve).)

**Ours** W:641-644: light = max(light, tint^2.2·glow·1.7) from glow textures.

**Verdict: ours extra.** For the match: glow off (or Bliss's method if a glow mask is kept: mix by glow^0.5 with the exposure-scaled brightness). Small.

### 17. Sun specular on land

**Bliss** `specular.glsl:180-298`, profile Specular_Reflections on, Sun_specular_Strength 10, Rough_reflections on, Roughness_Threshold 1.3. With vanilla textures: roughness = (1 − 0)² = 1, F0 = 0.02. Sky and screen reflections are skipped (hasReflections = F0·(1 − 1·1.3) < 0.01). Only the sun lobe remains: GGX with r = rough^2.5 = 1 → D = 1/π; F = F0 + (1 − F0)·2^((−5.55473·LH − 6.98316)·LH); vis = 1/(LH²·(1 − r/4) + r/4); value = NdotL·D·F·vis; added as DirectLightColor·shadow·NdotL·value·10, **not multiplied by albedo** (×"Metals" = 1). My numbers: sun behind the eye ≈ +6 % of NdotL² sun, white-orange; looking toward the sun at grazing LH ≈ 0.2 ≈ **3.9 × NdotL² sun**: a strong untextured sheen on every sunlit surface seen against the sun (golden hour, dawn shots).

**Ours** W:789 `mat_shine` only where a texture has LabPBR maps (`mat.has`); ordinary blocks have no sun specular. Water/film have their own.

**Verdict: missing.** Rebuild: for every lit pixel without maps, add sun·shadow·NdotL²·GGX(r=1, F0=0.02)·10 after albedo. Pack only, ~15 lines, cheap. Check against the sun-facing refs (sunup_h06).

### 18. Cloud shadows on light

**Bliss** `composite1.fsh:1269-1273`: cloudShadow from the cloud density (`GetCloudShadow`) multiplies the sun (and sun SSS) only; the sky light dims through the ambient average instead (clouds in it, ×(1 − 0.7·rain)).

**Ours** cloud shadow on the sun (W:466, 484), and also ×mix(0.6, 1, soft_cloud_shadow) on the sky light (L:1835).

**Verdict: ours extra layer.** Delete the sky-light one when §11 lands. −3 lines.

### 19. Caves and light leaks

**Bliss**: no lightmap gating of the sun (§8); leaks handled by the shadow bias and the sky < 0.1 block-centre snap; eye-brightness only on SSS and water. In caves the sky facing term rises to ≥ 1 all round (§10), AO and the 0.01 neutral minimum light; the exposure lifts the dark view; a blue cave *fog* (fog section).

**Ours**: three sky-light gates on the sun (§8), canopy-bent thresholds, `under_leaves`' cave exception, a cool floor light (§12).

**Verdict: ours more complex.** Covered by §8, §10, §12.

### 20. Night: moon light, minimum, Purkinje

**Bliss**: moon as §1 (1/1250 of the sun at full, phase-linear, no height factor; moonVis ramps with the sun's elevation 0 → −0.05); ambient = the moon-lit sky's average (§11); minimum 0.01 (§12); Purkinje in `composite11.fsh:146-161`: purkinje = clamp(exposure², 0, 1)·clamp(rodExp/(1 + rodExp)·1, 0, 1), rodExp from the view's blue-weighted mean (`deferred.vsh`: max(0.012/log2(avgB + 1.002) − 0.1, 0)·1.2); lum = col·(0.15, 0.3, 0.55), rodLum = col·(0.85, 0.7, 0.45)·200; rodCurve = mix(1, rodLum/(2.5 + rodLum), purkinje); col = mix(lum·(0.4, 0.7, 1.0)·5, col, rodCurve).

**Ours**: night key sun 0.17 0.21 0.30 and ambient 0.012 0.017 0.030 (L:~893), ambient ×1.6-6 by moon (L:979), chain-night ×0.08 (L:802-808), rain night dim; `bliss_purkinje` (L:260-276) with the chain, `display`'s night blue off in the chain (L:3338).

**Verdict: Purkinje same (with the chain); light partly.** Rebuild with §1 and §11 (moon from blackbody, ambient from the moon-lit physical sky); delete the night key, the ×1.6-6 boost and the ×0.08 chain-night factor. Pack only, −20 lines.

### 21. What the deferred pass has that we lack, in one list

Screen-space AO read by the lighting (§13), a screen-space sun ray (§5), coloured translucent shadows (§6), the GGX sun lobe on all surfaces (§17), autoBrightness on block light (§15), the low-sky-light facing floor (§10), the ambient from the sky-with-clouds average (§11). Lightning flash light (`Iris_Lightningflash` × sky^10) we have in our own form; border fog and under-water absorption belong to other sections.

---

### The cave mouth (cave_c4_y90): why Bliss's stone is warm and sunlit and ours cold and unlit

Measured (ref vs lkmxM1, sRGB, 13-px patches): right-hand stone in the sun **214 189 151 vs 95 100 111**; floor sun patch, lower right **214 185 138 vs 57 61 71**; dirt cliff face **222 152 65 vs 124 74 36**; shaded left wall 37 31 25 / 58 49 41 vs 78 83 93 / 84 88 99; floor centre 85 75 67 vs 57 61 72.

1. **No sun on those faces in ours.** Bliss lights a face wherever its shadow map says lit (§8). Ours multiplies the sun by two sky-light gates (W:484 min(map, smoothstep(0.3, 0.8, sky)); L:1864 smoothstep(0.5, 0.95, sky)): at a cave-mouth wall with sky level 8-11 that is 0-30 % of the sun, deeper 0. **But** the dirt cliff high on the right (open to the sky, gates fully open) is unlit in ours too, so a second cause is likely: **the sun stands on another side**. In the ref the faces turned toward +z (the right-hand walls, seen from a camera facing −x) take the sun; in ours they don't, and the faces toward the camera on the left read bright. Check first (minutes): one noon shot of a lone pillar in both, or print `scene.sun` at the shot. Our path is Rx(+35°) on the sun's frame (`client/sky.rs:65`, `mods/base/src/lib.rs:249` tilt 35 → noon direction (0, 0.82, +0.57)); Bliss's is OptiFine's sunPathRotation −35. If they lean opposite ways at this spot, flip our tilt's sign.
2. **The sun's colour.** Bliss's sunlit stone is beige-orange because its direct light is the 4300 K blackbody × transmittance (≈0.73, 0.45, 0.24) and, at ~40:1 over the sky light on a cave wall (my estimate: sun 0.78·T·NdotL against ambient ≤ 0.083 × 0.93 × curve(0.6) 0.14 × 0.7), the wall shows almost pure sun colour. Ours has the same hue now (L:735), so once the sun reaches the wall this part follows.
3. **The shade's colour.** Bliss's shade is warm-neutral (R > B) because its ambient is the hue of a sky and clouds lit by that same orange sun (§11), with a neutral minimum light (§12). Ours is a hand cool fill (0.85 0.95 1.2), 40 % greyed, plus the `OPT_SKY_HUE` pull toward the zenith's blue, plus a blue-tinted floor light: blue-grey.
4. **Brightness of the shade.** Bliss's cave shade is darker (37-85) than ours (57-100): its sky facing has no 0.75 minimum for sides in the open-sky formula but its AO (min of (1−o²)^5 and SSAO^6) darkens corners hard, and its exposure is set by a scene that now contains bright sunlit stone.

Fix order for this shot: (a) settle the sun's side; (b) delete the sky-light gates on the sun (§8) with the block-centre snap for sky < 0.1; (c) ambient from the sky-with-clouds average (§11) and a neutral minimum (§12).

---

### Ranked: biggest visual gaps in this area

1. **Sun gated by sky light** (W:484, L:1864, RT cover): cave mouths, ravines, overhangs and forest gaps lose the sun Bliss gives them. Delete; pack only.
2. **Sun direction in the cave shot**: the lit sides are swapped vs Bliss; verify the path's lean sign (minutes) before any tuning.
3. **Ambient colour**: hand cool fill + ten stacked fixes vs Bliss's sky-with-clouds average (warm-neutral under a 4300 K sun). Needs the physical sky on.
4. **Leaves/plants**: Ronald's Bliss has no SSS at all; our back-light glow, leaf-sun extra and lime cuts make crowns glow where Bliss's are plain. Delete.
5. **AO**: Bliss's min(vanilla^5-curve, SSAO^6) on indirect only vs our vertex AO on everything + whole-colour contact/far AO. Needs the lit pass to read a screen texture (engine).
6. **Sun specular**: Bliss's rough GGX ×10 on every surface (strong sheen toward the sun); we have none on ordinary blocks.
7. **Block light**: Bliss's steep curve, pale (1, 0.75, 0.65), ×exposure boost by day, added; ours wide, orange, cut by day.
8. **Screen-space sun ray and coloured translucent shadows**: missing; engine work, smaller on the matrix shots.
9. PCSS: small gap (Bliss's penumbra is under 0.1 block).

---

## Clouds

Bliss's paths are under `.../Bliss-Shader-git/shaders/` (`VC` means `lib/volumetricClouds.glsl`). Ours are under
`mods/base/shaderpack/` and `crates/engine/src/`. Profile values come from Ronald's `pack_settings.txt`; every other
value is Bliss's default from `lib/settings.glsl`. Bliss's method is described here, not copied (© Chocapic13, all
rights reserved). Its `texture/noises.png` is its own asset too, so we make our own noise with the same job.

**Correction to the 11-25 report:** in today's Bliss git the clouds are **not** drawn at half size in `deferred2`.
`deferred2.fsh:101`, `composite1.fsh:175` and `composite2.fsh:146` all define `CLOUDS_INTERSECT_TERRAIN`, which
turns off the half-size path (`deferred2.fsh:123`) and its bicubic upsample (`composite1.fsh:968`). The clouds are
marched in `composite2` together with the volumetric fog, at `VL_RENDER_RESOLUTION`, which is **1.0** in the
profile (default 0.5). That means full size, per pixel. See section 9.

---

### 1. The layers: heights and thickness

**Bliss's method** (`VC:27-35`, `settings.glsl:378-391`)
- There are three layers, at absolute world heights (sea level 63):
  - Layer 0 (small cumulus): from `CloudLayer0_height` **250** up 100, to 350.
  - Layer 1 (large cumulus): from `max(CloudLayer1_height 500, 350)` up 100, to 600.
  - Layer 2 (altostratus): a flat plane at `max(2000, 600)` = **2000**, 5 blocks thick in effect (`VC:454`: step `5/|dir.y|`).
- Each layer is marched on its own (`VC:524-561`): 15 steps (`HQ_CLOUDS`: min 15, max 15) from where the ray enters
  the slab (`layerStartingPosition`, `VC:401-416`). The step is `dir * 100/|dir.y|/steps`, so a ray always crosses
  the whole 100-block slab, however far away that is.
- Step count by view height: `clamp(15/sqrt(2^viewY), 0, 15)`, so 15 at the horizon and about 10.6 at the zenith (`VC:444`).
- Earth curvature (`VC:450`): `dir.y += 0.025 * heightRelativeToClouds`, where `heightRelativeToClouds = clamp(1 - max(camY - 250, 0)/100, 0, 1)`.
  A horizontal look therefore still meets layer 0, at about `(250 - camY)/0.025`, which is about 7.6 km from y 60.
- Blend order (`VC:563-596`) depends on where the camera is: below layer 0 the layers blend front to back, above it
  back to front. The altostratus goes behind the cumulus when you are under 2000, in front of them when above.
- Overdraw cut: when layer 0 is opaque (`a < 1e-5`) and the camera is below layer 1, layers 1 and 2 are skipped.

**Ours** (`sky.wgsl:227-228, 341-343, 345-362`; `look.wgsl:1073-1076`; `registry.rs:739`; `sky.rs:829`)
- There is one soft layer. Its underside sits at the sky entry's height **224.33** + `core:cloud_height` (0) + `SOFT_LIFT`
  **220**, so about **444**, and it is **120** thick (to about 564).
  - In rain the lift drops to `220*(1 - 0.7*wet)`, putting the underside near **290**. A storm's slab grows to 190 thick.
- Steps: `mix(36, 18, |dir.y|)` between the slab's two planes, capped at `SOFT_REACH` **4000** blocks.
- There is no curvature, so a horizontal look never meets the layer.
- The altostratus is not a volume: `cirrus()` (`sky.wgsl:1050-1080`) mixes a lit colour into the sky from 2D noise
  on a plane at 2000 (HDR: a second sheet of 2400-block noise × 0.45, and a cirrus wisp layer × 0.35).
- The old blocky clouds (`fs_clouds` 611-781, `cloud_density` 181-217, `storm_field`, `cloud_lumps`, `deck_*`) are still in
  behind `core:soft_clouds`, which defaults to 1 (`config.rs:665`).

**Verdict: partly.**
- Ours is one layer standing between Bliss's two. It has no small low cumulus at 250-350, and in rain it moves down
  and thickens, where Bliss keeps the heights and changes only cover and density.
- There is no real altostratus volume.

**Rebuild:**
- Two slabs at 250-350 and 500-600 (absolute y), each with 15 steps of `100/|y|/15` from slab entry. Add the 0.025
  curvature lift and the blend order by camera height.
- Delete `SOFT_LIFT`'s rain drop, `SOFT_STORM_THICK` and the storm tower height (`cloud_height`). Rain changes cover
  and density only (section 2).
- Write the altostratus as a one-sample layer (section 4) and delete `cirrus()`'s HDR branch.
- Remove the blocky clouds' code (≈250 lines) or keep it LDR-only.
- Engine: `soft_low` gets its height from `extra.z`. The two layers' heights can be pack constants, as Bliss's are
  absolute. Small cost (shader only, about +80 lines and -150 lines).

---

### 2. Coverage, and how rain and the day change it

**Bliss's method** (`VC:19-47`; profile)
- `Daily_Weather` is **false** in the profile, so the coverages are fixed: L0 **0.5**, L1 **0.6**, L2 **0.4** (profile).
  Densities are the defaults: L0 **0.5**, L1 **0.5**, L2 **0.1**.
- Rain blends each value toward its rain value with `rainStrength`:

  | | coverage | density |
  |---|---|---|
  | L0 | 0.5 → `Rain_coverage` **1.1** | 0.5 → 1.0 |
  | L1 | 0.6 → **0** (the large cumulus disappear) | 0.5 → 0 |
  | L2 | 0.4 → 1.5 | 0.1 → 0.05 |

- Rain also drops the flat-bottom cut. The `Baseshape` term is multiplied by `(1 - rainStrength)` (`VC:137, 150`), so the
  rain deck has flat, hanging bases.
- The altostratus's sunlight is multiplied by `(1 + 3*rain)` (`VC:559`).
- Nothing depends on the hour. With `Daily_Weather` on (Bliss's default, off in the profile) a per-day table in the
  sky LUT's pixels (1,1) and (2,1) would set the coverages and densities instead.
- Wind: `cloud_movement = (worldTime + (worldDay mod 100)*24000)/24 * Cloud_Speed(1)`, which is 1000 blocks per game
  day (`VC:51`). L1 moves at twice that, along swapped and negated axes. The erosion noise drifts at a quarter of it.

**Ours** (`sky.wgsl:258-277`; `sky.rs:877-935`)
- `soft_cover = clamp(mix(0.32, 0.88 (chain; else 0.8), wet) + (field - 0.5) * mix(0.35, 0.5, wet))`, where:
  - `field` is two slices of our 3D noise at 6144 and 1536 blocks (0.65/0.35);
  - `wet = max(axes[0].x, axes[0].z)`;
  - the engine's weather sets `axes[0] = (rain*0.75 + thunder*0.25, rain*0.6 + thunder*0.4, thunder)`.
- Extra rain layers:
  - darkening `axes[0].y` (×(1 - 0.55y) on the sun, ×mix(1, 0.55, y(1 - a)) on the ambient);
  - albedo ×0.4 (chain) or 0.66, and ×0.6 in a storm (`sky.wgsl:418`);
  - the lowered deck (above);
  - the low-horizon rain mist in `soft_pixel` (`sky.wgsl:595-596`).
- The blocky path adds its own deck, its towers, the deck's breaks toward the sun (`SUN_SIDE`) and the grey palettes
  from `sky.rs`.

**Verdict: partly, and ours is more complex.**
- Ours is one cover number that goes from 0.32 to 0.88. Bliss has three per-layer values, and the large layer
  disappears in rain while the low one closes (cover 1.1 on a formula whose noise term is mostly negative, so there
  are still breaks).

**Rebuild:**
- `LAYERn_COVERAGE` and `LAYERn_DENSITY` become constants blended by one `rain` input, as in the table above.
- Delete `soft_cover`'s field, the 0.32/0.88 history, `axes[0].y` darkening, the rain albedo, the storm albedo and the
  deck drop.
- The wind (1000 blocks/day, L1 ×2 the other way, erosion ×¼) replaces `s.uv.zw`'s drift for the soft clouds.
- Engine: `axes[0]` only needs the rain strength, smoothed as Minecraft's `rainStrength` is. Thunder needs no deck of its own.
- Small cost.

---

### 3. Shape: coverage noise, 3D erosion, remaps

**Bliss's method** (`VC:54-66, 81-223`, `fbmAmount` **0.5**)

*Coverage, 2D (`cloudCov`).*
- Layer 0:
  - `large = B(xz/5000)`, `small = R(xz/500)`, both moving with the wind.
  - `cov = |large*2 - 1.2|*0.5 - (1 - small)`.
  - `layer0 = min(cov + COV0, inside-slab ramps)`.
  - Top rounding: `Top = max(y - (top - 75), 0)/200 + max(y - (top - 10), 0)/15`.
  - Base: `Base = max(bottom + 12.5 - y, 0)/50`.
  - `final = max(layer0 - Top - Base*(1 - rain), 0)`.
- Layer 1:
  - `large = B(-zx/10000)`, `small = B(-zx/2500)` (wind ×2).
  - `cov = |large - 0.8| - small`, then `+ COV1 - 0.5`.
  - The top term is squared (`Top²`); the base uses 15.5 instead of 12.5.
- (`R`, `B` are channels of a 512² tileable 2D noise texture.)

*Erosion (`cloudVol`, LoD 1 in the view march, `VC:183-210`).*
- `samplePos = pos * (1, 1/48, 1)/4`, moved by `-wind/4`, then sheared with height by `((y - (bottom + 20))/20)^1.5` blocks
  in x and z, so the tops lean.
- The 3D noise is the 2D texture stacked in z with a 193-texel offset per slice and linear blending between slices
  (`densityAtPos`: `pos/18`, `xz*0.5`).
  - In world terms one texel covers about **0.72 blocks** in layer 0 (×200) and about 1.44 in layer 1 (×100). The
    second octave is ×600 / ×450.
  - It is fine-grained noise near block scale, not billow shapes. It frays edges into grain that TAA averages to soft wisps.
- `n = (1 - N(×200|100))*sqrt(1 - cov) + |N(×600|450) - (1 - clamp((top - y)/100))| * 0.75 * (1 - cov)`.
- `cloud = max(cov - (n²)² * fbmAmount, 0)`.
  - Erosion is strong only where the cover is thin, and cores stay solid.
  - The term `log(fbmPower1/2)` is computed but unused.
- Light taps and shadows use LoD 0, which skips the second octave.
- The base fades in with height: extinction is multiplied by `clamp((y - bottom)/25, 0, 1)²` (`VC:357`).

**Ours** (`sky.wgsl:279-315`; `soft_clouds.rs:104-155`)
- The noise is a 64³ Perlin-Worley volume:
  - r: Perlin fbm remapped by Worley, rank-equalised;
  - g, b, a: Worley at 4, 8 and 16 cells.
- Shape (HDR):
  - `billow` at **2048** blocks, `mass` (a 2D slice) at 4096, `small` at 384, weighted 0.5/0.4/0.1;
  - remap `(shape - (1 - cover))/cover*1.6`;
  - base `smoothstep(0, 0.08, a)`;
  - top cut `smoothstep(0.35|0.55, 1, a)*0.45|0.4`.
- Erosion: Worley fbm at 224 blocks × 0.35 × (1 - d) × mix(0.6, 1, a).
- Plus the LDR-only alternatives (1024 / 2048 / 128 / 0.5) and the time-animated noise z (`t*0.0003`).
- Extinction is `d*0.55` per block.

**Verdict: different method.**
- Bliss's cumulus come from a *2D coverage map* (blob size set by the 500-block R noise, grouping by the 5000-block B
  noise), cut by height ramps, and frayed by near-block-scale grain that only bites thin parts.
- Ours is the Horizon/Photon 3D billow remap with large Worley cauliflower. Silhouettes, edge texture and puff sizes
  differ.

**Rebuild:**
- Replace `soft_cover` + `soft_density` with Bliss's `cov → height ramps → erosion` chain per layer (≈60 lines).
- Make our own 512² tileable 2D noise with two smooth channels (one with blob-sized features for `small`, one broad for
  `large`) and a grain channel for the stacked-slice erosion. The 3D Perlin-Worley atlas can stay for the grain or go.
- Delete the HDR/LDR scale pairs, the cauliflower `small` billow, the equalising rank pass (if the new noise is made
  to range) and the animated z.
- Engine: a new noise texture in `soft_clouds.rs` (+60 lines, −50).
- Medium cost. This is the single biggest change to the look of individual clouds.

---

### 4. Altostratus (layer 2)

**Bliss's method** (`VC:69-79, 298-333`)
- Density:
  - `large = max(1 - B((xz + wind)/100000) + COV2 - 0.7, 0)`;
  - `medium = 1 - B((xz - wind)/7500 + (-large, 1 - large)/5)`;
  - `shape = max(large - 0.4*medium*clamp(1.5 - large, 0, 1), 0)`;
  - density `= shape²`, × L2 density (0.1).
- One sample on the plane, with step length `5/|y|`.
- Two light taps along the sun at `(1 + j*dither) / (|sun.y*0.5|³*0.995 + 0.005)` blocks. That is about 1-2 blocks with
  a high sun and up to about 200 with a low sun.
- `od = Σ density × (1 - |sun.y|)`.
- Lit by the same `DoCloudLighting` with `skyScatter` 0.5. In rain the sun term is ×(1 + 3 rain).
- Drawn only on the camera's side of the plane.
- It shadows the cumulus (`VC:376-380`: `alto(at the sun ray's 2000 crossing) * 0.1 * (1 - |sunY|)`) and the land (section 10).

**Ours:** `cirrus()` (`sky.wgsl:1050-1080`)
- 2D noise at 600 (wisps) and 2400 (sheet) on a 2000 plane.
- Colour `p.horizon*0.6 + p.ambient*0.8 + p.sun*0.35 (+ HG 0.6 forward)`, mixed in at 0.35 / 0.45.
- No extinction of the sky behind, no light taps, no shadow.

**Verdict: missing (faked).**
- **Rebuild:** a one-sample volume (≈30 lines) through the shared lighting function, and delete both cirrus branches.
  Its shadow onto the cumulus and the land comes for free once section 10 is rebuilt. Small cost.

---

### 5. Lighting: scattering, powder, phase, light taps

**Bliss's method** (`VC:229-256, 343-386, 468-499`)
- Phase:
  - `phaseg(x, g) = (1 - g²)/4 * (1 + g² - 2gx)^-1.5 / 3.14`.
  - Single scattering: `mieDay = phaseg(μ, 0.85) + phaseg(μ, 0.75)`.
  - Multiple scattering: `mieMulti = phaseg(μ, 0.35) + 0.5*phaseg(-μ, 0.35)`.
  - `sunScatter = L * mieDay * 3.14`.
  - `sunMulti = L * mieMulti * 3.14 * 2`.
  - `L = lightCol/80`: the sun's colour through the air (`sunColorBase/4000 * skyAbsorb * sunVis`, `deferred.vsh:160-168`),
    or the moon's when the sun is down.
- A fake Rayleigh on far clouds (`VC:491-497`):
  - `scatter = distantfog * exp(-10000 * rC * exp(-5|y|))`, with `rC = (5.8e-6, 1.35e-5, 3.31e-5) * 3`.
  - At the horizon that is (0.84, 0.67, 0.37): far clouds go orange and dim. At y 0.3 it is about (0.96, 0.91, 0.80).
- Three light taps per sample at 20, 40-60 and 60-100 blocks along the sun (dither scales the gaps), LoD 0.
  - `od = Σ density × layerDensity`. The densities are summed, *not* × step length.
  - Plus the shadow of layer 1 on layer 0: `2 × L1density × density` at the point where the sun ray reaches layer 1's base.
  - Plus the altostratus shadow (section 4).
- `DoCloudLighting`:
  - `powder = 1 - exp(-10 ρ)`, with ρ = cumulus × layer density.
  - `direct = sunScatter * exp(-10 od) + sunMulti * exp(-3 od) * powder`.
  - `indirect = sky * mix(1, 2*(1 - sqrt(skyScatter³ * ρ)), distantfog^(1 - 0.5 rain))`, where `skyScatter = clamp((top - y)/100)`.
  - So tops take 2× the sky light, and bottoms `2(1 - sqrt ρ)`.
  - The result is `direct + indirect`. There is no albedo factor.
- Integration: `C += (light - light*exp(-step*ρ_e)) * T`, `T *= exp(-step*ρ_e)`. Here `ρ_e` is the base-faded extinction,
  the step is `90/|y|/15`, and the march stops when T < 1e-5.

**Ours** (`sky.wgsl:345-490`): the same idea with stacked parts.
- Phase: `0.75 HG(0.75) + 0.25 HG(-0.2)`, capped at 0.6 in LDR.
- Light taps: 5 at 3, 6, 12, 24 and 48 blocks, `od` × 0.55 × segment length.
- Scattering: three octaves (k 0.5, e 0.4, phase pulled toward 1 by 0.6^o).
- Powder: `mix(1 - exp(-14σ), 1, 0.5(μ/2 + 0.5)² + 0.3)`, × 0.9.
- Ambient: a palette blend by height (`p.horizon*0.3 + p.ambient*0.25` → `p.ambient*1.2 + p.horizon*0.2`).
- Layers stacked on that:
  1. a sun-strength floor down to −4° (`sky.wgsl:384-387`);
  2. `CLOUD_SUN_HDR`, `CLOUD_SKY_HDR` (both 1.0, dead);
  3. half desaturation of the sun from 6° to 17° (`:405`);
  4. a "cream away from a low sun" tint (`:409-413`);
  5. `cloud_low_sun` ambient ×0.72 (`:457`);
  6. LDR `cloud_sun_tint` gold;
  7. the HDR "deep" fill: `sun*exp(-0.12 od)*(1 - 0.5 exp(-8σ))*0.35`, plus the sky from below (`:459-469`);
  8. the HDR silver rim: `sun*HG(0.75)*12.566*0.5*(1 - d)²*exp(-0.2 od)` (`:472-476`);
  9. the rain and storm albedos (section 2);
  10. the night cuts (section 7).

**Verdict: ours is more complex.**
- The two lobes, the 3-tap od, powder only on the multiple term, the `2(1 - sqrt)` sky gradient and the fake Rayleigh
  cover what our deep fill, rim, tints, desaturation and low-sun fixes were trying to reach.
- **Rebuild:** one `cloud_light(ρ, od, skyScatter)` as above (≈25 lines). Delete items 1-10 and the octave loop (≈ −70 lines).
- Inputs: our sun colour through the air (`palette_now` sun) and the sky's hemispherical average (section 6).

---

### 6. Ambient from the sky, and layer-to-layer sky occlusion

**Bliss's method**
- The sky light on clouds is `averageSkyCol/30` (`composite2.fsh:398`): the mean of the sky LUT over the
  hemisphere *without* clouds (`deferred.vsh:119-152`), × `PLANET_GROUND_BRIGHTNESS` 1.
- It is scaled by the hour (`VC:499`): `mix(Sky_Brightness 1, 1.25 - 0.75(1 - sunY⁺)⁵, distantfog)`, which is 0.5 at
  sunrise and 1.25 at noon.
- Layer 1 shades layer 0's sky light (`VC:337-338`):
  - `skyOcc = mix(1, (1 - L1density)*0.8 + 0.2, (1 - 2^(-5 u²)) * distantfog)`;
  - `u` is layer 1's density straight above the sample.
- The clouds also go into the sky LUT. `deferred.fsh:270-299` composites sky + clouds + fog into a second LUT whose
  average `averageSkyCol_Clouds = normalize(avg) * min(3 luma, 2.5) * (1 - 0.7 rain)` (`deferred.vsh:151`) lights the
  fog and the world's sky light (`indirect_lighting_effects.glsl:396`). Overcast therefore dims the world's ambient on
  its own.

**Ours:**
- Palette mixes (`p.horizon`, `p.ambient`) by height, with HDR "deep" sky from below.
- A local sky-light cut under the cloud shadow, ×mix(0.6, 1, shadow) (`look.wgsl:1838-1841`), stands in for the cloudy LUT.

**Verdict: partly.**
- **Rebuild:** feed `sky_avg` (the clear hemisphere mean) to the clouds with Bliss's hour factor, and add the L1→L0
  sky occlusion (needs two layers).
- Larger, engine work: a small sky-with-clouds LUT (Bliss's 256² sphere map) and its average. It would replace
  `look.wgsl:1840` and the rain hemisphere fixes listed in the light section. That is about one low-res pass of the
  march per frame (Bliss marches it with the same `renderClouds`). Medium cost.

---

### 7. Night, moon light, rain darkening

**Bliss's method**
- No night branch. The light is `moonColor * moonVis` (`moonColorBase/4000`, `moon_illuminance` **200** profile) through
  the same function, and the sky light is the night sky's average, so the clouds come out barely above the sky
  without any tuning.
- `WsunVec` is flipped to the moon (`VC:428`).
- Rain darkening comes only from:
  - higher L0 density (1.0) and cover;
  - the indirect exponent `distantfog^(1 - 0.5 rain)`;
  - the darker rain sky (`deferred.fsh:264`: the sky blends toward `averageSkyCol*4000*(0.7 skyAbsorb + 0.3)` near the
    horizon by `rain × heightRelativeToClouds`).
- There is no albedo cut.

**Ours:**
- Night: sun ×0.06 (chain) or 0.2 below −7° (`sky.wgsl:393-400`); sky ×0.4 (`night_amb`, `:461`), also on the deep fill.
- Rain: albedo ×0.4 or 0.66, storm ×0.6, `axes[0].y` darkening on sun and ambient, a lower deck, misted low horizon.

**Verdict: ours is more complex.**
- These are all fixes for not having Bliss's inputs. Delete them once sections 5-6 are rebuilt and the moon's colour
  feeds `L`. Small cost.

---

### 8. How far they reach and how they fade into the sky

**Bliss's method**
- `CLOUDS_INTERSECT_TERRAIN` (`VC:287-304, 346-351`): the march stops where the ray passes the pixel's depth.
  `maxdist = far + 80` (DH: `dhRenderDistance + 512`).
- For a **sky pixel** the depth is the far plane, so its length is about `far`, which is below `maxdist`. **The clouds
  end at about the render distance.** With 32 chunks they stop about 600 blocks out and leave a clear pale band above
  the horizon, as seen in the 11-06 test (memory: `--mc-clouds` in `sky.rs:907`).
  - With DH they reach DH's far plane instead.
- `distantfog` (SKY_GROUND on, `VC:483-489`): `mix(1, 1 - 2^(-|y|^mix(1.5, 4, rain) * mix(100, 35, rain)), heightRelativeToClouds)`.
  - It does *not* fade alpha. It scales the sun terms (through `scatter`) and the sky term's gradient, so low clouds
    turn flat sky-lit grey near the horizon, while near clouds keep their opacity.
  - In rain the band is much wider (exponent 4, rate 35).
- In `composite2` the clouds sit behind all the volumetric fog: `fog.rgb + clouds.rgb * fog.a * atmosphereAlpha`.
  The fog's air haze is cut above the cloud base (`overworld_fog.glsl`: `cloudPlaneCutoff`).

**Ours** (`sky.wgsl:525-530, 576-604`; `look.wgsl:1076`)
- Reach is `SOFT_REACH` **4000** blocks.
- Colour fades to `sky_color` by `smoothstep(0.45, 1, dist/4000)`, and alpha by `smoothstep(0.5 (HDR), 1, dist)`.
- Rain: a low-horizon mix to a squashed sky colour (`1 - smoothstep(0.02, 0.22, dir.y)`).
- `fog_through_sky(color, dir*220)`.
- The rainbow is drawn over them, and the clouds are written at `soft_distance` depth.

**Verdict: different.**
- Our clouds run to 4 km and fade to sky colour.
- Bliss's end at the render distance (or DH distance), have no alpha fade, and the far ones are lit by sky only with an
  orange fake-Rayleigh sun term.
- **Rebuild:**
  - End the march at the frame's far distance (view distance or LOD distance) as Bliss does: `far = depth or view distance`.
  - Add `distantfog` and the Rayleigh term in the lighting, and composite under the fog march.
  - Delete the sky-colour fade, the alpha fade, the rain horizon mist and `fog_through_sky(…220)`.
  - Whether to copy the clear band at 32 chunks is a decision for Ronald: it is what Bliss does without DH.
- Small cost.

---

### 9. Resolution, jitter, upsample, temporal

**Bliss's method** (`composite2.fsh:366-447`, `composite3.fsh:134-170, 306-316, 438-439`, `composite2.vsh:51`)
- The march runs in `composite2` at `VL_RENDER_RESOLUTION` = **1.0** (profile): one ray per pixel.
- Dither:
  - start offset `Dither.y` = blue noise + frame × 0.618;
  - light-tap jitter `Dither.x = 1 - R2(pixel + 2*frame)`.
- Output: fog+clouds rgb and fog alpha in colortex0, cloud alpha in colortex14.
- Upsample (`composite3`): a 5-tap depth-aware filter (centre + 4 diagonals, kept where `|Δdepth| < threshold`),
  applied to both. At 1.0 scale this is a light cross-shaped spatial denoise.
- Then the main TAA, with no cloud history of its own.
- The half-size `deferred2` path (`CLOUDS_QUALITY` 0.5, bicubic in `composite1`) is compiled out.

**Ours** (`sky.wgsl:494-573, 576-581`; `soft_clouds.rs:20-24`; `config.rs:669`)
- Trace at `core:cloud_res` 2 (half each way), with a 4×4 Bayer sub-pixel per frame.
- Dither: a hash + frame × 0.618.
- With TAA the frame passes through as is. Without TAA, a 3×3 clamp and up to 0.9 history.
- `soft_pixel` reads it with a bilinear tap.

**Verdict: partly.**
- **Rebuild:** trace at full size, fused into the fog march (section 11), with the 5-tap depth-aware filter on its
  output, and delete `fs_cloud_resolve` and the trace and history targets.
- Cost: about 4× the trace pixels. Bliss affords that with 15 steps per layer, 3 cheap 2D-texture taps and early-outs
  outside slabs, against our 18-36 steps × (1 + 5 taps) of 3D-atlas reads.
- On the Mac budget a `cloud_res` 1/2 setting (cost shown) is needed. Engine: `soft_clouds.rs` shrinks, and the
  composite moves into the fog pass. Medium cost.

---

### 10. Cloud shadows on the land and in the fog

**Bliss's method** (`VC:610-668`; `composite1.fsh:1270`, `overworld_fog.glsl:215`)
- No shadow map. Per pixel, the sun ray from the point is followed to one plane per layer: L0 at `height + 30`, L1 at
  `height + 50` (fog: +30), L2 at 2000.
- `s = clamp(Σ cov-density(LoD 0) × layerDensity + alto × L2density × (1 - |sunY|), 0, 1)`.
- Land: `2^(-100 s²)`. Fog: `e^(-100 s²)`. Both × `CLOUD_SHADOW_STRENGTH` 1.
  - The core goes to **0**, and the edge is soft over `s` 0.05-0.2.
- It multiplies only the sun (shadow and SSS), never the sky light. Translucents use `shadow³`.
- No fade by sun height. It works with the moon too (signed `WsunVec`).
- In the fog every step takes it, so shafts form under the gaps.

**Ours** (`sky.wgsl:1351-1378`; `look.wgsl:1650-1709, 1840, 2475, 2597, 2771`; `cloud_shadow.rs`, 512² map)
- A 4096-block map (8 blocks per texel) made each frame from a 6-step march through the slab:
  `through = max(exp(-2.5 od), 0.12 exp(-0.2 od))` (a floor of 0.12 plus a slow tail).
- Read with `smoothstep(0.03, 0.15, l.y)` fade-in.
- Extra: sky light ×mix(0.6, 1) under the shadow.
- It feeds the land, the mist march, the banks and the froxels.
- The blocky path keeps its own `cloud_shadow` from cells (`1 - 0.72 smoothstep(cover)`).

**Verdict: ours is more complex.**
- The look differs too: ours never goes darker than 0.12 sun, cuts sky light, and loses penumbra detail below 8 blocks.
- **Rebuild:** `cloud_shadow(rel)` = Bliss's per-pixel plane lookup on the new coverage function (2-3 texture reads
  per layer), `2^(-100 s²)`.
  - Delete `fs_cloud_shadow`, `cloud_shadow.rs` (−100 lines of engine), the 0.12 floor, the 2.5× od, the sky-light cut
    at `look.wgsl:1840` and the sun-height fade.
  - Cheaper code and the same per-pixel cost class as Bliss.
- Small to medium (engine deletion).

---

### 11. Sun shafts through clouds in rain

**Bliss's method**
- Not a separate effect.
- In rain L0 has cover 1.1 and density 1.0. Its noise term `|L*2 - 1.2|*0.5 - (1 - S)` still leaves gaps where `S` is low.
- The fog march (`overworld_fog.glsl`) multiplies the sun in each step by the cloud shadow (section 10), and the rain
  fog (`RainFog_amount` **0.3** profile) gives it something to light. Shafts come out under the gaps.
- The altostratus's ×(1 + 3 rain) brightens the high sheet seen through them.

**Ours** (`look.wgsl:2475-2494`, commit cf764277e)
- The rain fog is lit by `rain_sky + rain_sun * lit`, with `lit` carrying `cloud_shadow`. That is the same idea.
- The blocky path adds `deck_open` breaks wider toward the sun (`SUN_SIDE`) and `post/volume` shafts.

**Verdict: same idea.**
- The shafts' shape follows our cover and shadow, so it gets fixed by sections 2, 3 and 10.
- Delete `deck_open`, `SUN_SIDE` and the sweep once the blocky path goes. No extra cost.

---

### 12. Order in the frame (where the clouds are drawn)

**Bliss's method:** clouds and fog come out of one pass. The clouds sit behind the fog, and both are applied over the
lit scene and the sky with `color*fogA*cloudA + vl.rgb` (`composite3.fsh:438-439`). The sky behind comes from
`composite1` with no clouds.

**Ours:** the clouds are drawn in the sky pass (`fs_clouds` → `soft_pixel`) with their own depth, then the world's
fog march treats sky pixels at `MIST_MARCH_FAR`.

**Verdict: partly.**
- **Rebuild:** this goes with section 9. Engine: the fog pass binds the cloud trace, or the trace becomes part of the fog
  march (Bliss's way). The `soft_distance` depth write can then go. Medium cost, shared with the fog section.

---

### Biggest visual gaps in the clouds, ranked

1. **Layer structure:** one layer at about 444-564 (rain about 290) against Bliss's two cumulus layers (250-350 and
   500-600) plus a lit altostratus at 2000. In rain Bliss's large layer goes away while the low one closes flat-based.
2. **Cloud shape:** Bliss's cumulus come from 2D coverage noise (500 and 5000 blocks) cut by height ramps and frayed by
   near-block grain. Ours are 2048-block 3D billows with Worley cauliflower. Silhouettes and edges don't match.
3. **Lighting:** ten stacked fixes (deep fill, rim, tints, desaturation, night and rain cuts, octaves) instead of
   Bliss's two lobes, `exp(-10od) + exp(-3od)·powder`, `2(1-√ρ)` sky light and fake-Rayleigh horizon. Rebuilding is
   about −70 lines.
4. **Reach and horizon:** ours run to 4 km and fade to sky colour. Bliss's end at the render distance (DH: DH's), keep
   full alpha and go sky-lit and orange near the horizon.
5. **Cloud shadows:** ours floor at 0.12, cut sky light and are built from a 4096-block map. Bliss's are per pixel,
   `2^(-100s²)`, a black core and sun only.
6. **Resolution:** full size with a 5-tap bilateral filter and TAA, against our half size with bilinear sampling.
7. **Clouds in the sky light:** overcast dims the world's ambient through the sky LUT in Bliss; ours fakes it locally.

---

## Water and post

Bliss is "© Chocapic13, all rights reserved": below is its method in words and maths only.
Bliss paths are under `references/shaders/bliss/Bliss-Shader-git/shaders/`. `world0/compositeN` includes
`dimensions/composite(N-1)`, so "dims/compositeN" below is the file holding the code. Ours are at the tip of
`claude/look-list` (a3b17ae32), defaults from `mods/base/src/look.rs:268-320` (chain 1, curve 1, bloom 1,
bloom_add 1, sharpen 0.4, glare 0, water_absorb 1, water_foam 0, water_mirror 1, water_body 0.35, fog_one 1,
sky_phys 1, chain_night 0.08).

Ronald's profile changes that matter here: TONEMAP Hejl2015, SATURATION 0.3, SHARPENING 0.4, BLOOMY_FOG 1.25,
**DOF_QUALITY 3 (on)**, focal 1.9, Specular_Reflections / Screen_Space_Reflections / Sky_reflection /
Rough_reflections on, **Sun_specular_Strength 10**, Roughness_Threshold 1.3, Porosity on. All else is Bliss's default.

---

### Part A: water

#### A1. Waves and normals

**Bliss's method** (`lib/waterBump.glsl:31-91`, `dims/all_translucent.fsh:551-566`, `:169-177`)
- The height is the sum of three taps of a 512² blurred noise texture (`noises.png`, blue channel). Each tap is
  turned by the golden angle (2.39996 rad) from the one before. The tile sizes are 48×12, 12×48 and 32×32 blocks.
  A large-scale mask W = max(noise(pos/600), 0.1) adds an offset of W/2 to each tap and scales the result:
  h = (Σ taps)/60 · W. The water is calm where W is small and rougher where it is large.
- The taps slide in noise-UV by t·0.035·WATER_WAVE_SPEED(1). That is about 1.7 blocks/s along a 48-block tile.
- The normal comes from forward differences over a step Δ = min(dist/224, 3) + 0.15, which is wider far away
  and acts as the distance filter. n = normalize(dx, dz, 1 − (dx+dz)²). It is then amplified 10× (WATER_WAVE_STRENGTH 1):
  n' = 10·n + (0,0,−9).
- Parallax: before the normal is taken, the lookup point moves along the view by h·2·(view.xy·½·22·1.5), a
  one-step parallax (PW_DEPTH 1.5, PW_POINTS 2).
- Water on a slope (flowing water's sides): the lookup point is pushed along the face's xz by (y + 3t), so it flows downhill.
- The normal's tangent xy/3 is stored for refraction.

**Ours** (`mods/base/shaderpack/world.wgsl:866-915`, the same code again in `assets/shaders/post/reflect.wgsl:27-75`)
- Four layers of procedural value noise (`noise3`): cells 6×1.6, 1.6×6, 3.2×3.2 and 0.8×0.8 blocks, amplitudes
  0.10, 0.08, 0.07 and 0.022, golden-angle turned, sliding at 1.6 blocks/s.
- Extra layers stacked on top: the whole set is turned to face the wind; the amplitude rises with wind speed
  (0.7→1.0); the finest layer fades out between 16 and 56 blocks; the slope is scaled by
  k = mix(1, 0.5, 24-160 blocks) × mix(0.4, 1, glance); the step is 0.12 + min(dist/200, 3), about Bliss's Δ;
  rain rings are added to the slope (`rain_rings`, :917-938).
- No parallax, no large-scale calm/rough mask, no flow on sloped water.
- `reflect.wgsl` repeats the first three layers with its own copy of the hash and noise, and flattens them more
  at a glance (mix(0.15, 1)).

**Verdict: partly. Ours has more layers.** The pattern is the same idea; the missing visual is Bliss's large patches of
calm and choppy water (W). Rebuild:
- Move to Bliss's three-layer height with the W mask and the 10× amplified normal.
- Drop the glance flattening and the extra fine layer. Keep the rain rings and the wind turn; they are ours.
- Have `reflect.wgsl` stop re-deriving the normal. Either the water pass writes its normal (or tangent xy, as
  Bliss does) into the world image's spare channels, or SSR moves into the water pass (see A5).
- Parallax: about 5 lines. All of this is pack work; the normal in a G-buffer channel needs a small engine change.

#### A2. Water colour: absorption and scattering (seen from above)

**Bliss's method**
- The surface itself has no colour: water albedo is 0 and alpha is 1/255 (`all_translucent.fsh:500-505`; no
  Vanilla_like_water, no BIOME_TINT_WATER). All colour comes from the bottom and the body.
- The bottom (`dims/composite1.fsh:1059-1082, 1302`), shaded in the deferred pass:
  - The estimated depth is d = Vdiff·|view.y|, with Vdiff = (distance from surface to floor) × mix(5, 2, lit),
    so 2× outdoors.
  - ε = water (0.2629, 0.0565, 0.01011) + dirt (0.65, 0.85, 1.05) × (Dirt_Amount 0.14 + 0.01).
    That gives ε = (0.3604, 0.1840, 0.1676) per block.
  - Floor × [mix(exp(−2εd), exp(−8ε), 1 − exp(−0.1d))]: absorbed by depth, but never darker than an
    8-block floor.
  - The sun on the floor is multiplied by mix(1, caustics + 0.25, clamp(d, 0, 1)) (A6).
- The body's light (`dims/fogBehindTranslucent_pass.fsh:193-282, 318-397`), at half size and bilaterally
  upsampled in composite1:1325-1331, is a 2-step exponentially spaced march (rayMarchSampleCount 2, base 11)
  from the surface to the floor:
  - Ray length is 2×the true distance, capped at 12 blocks.
  - Here dirt = 0.14 + 0.1 = 0.24: ε = (0.419, 0.2605, 0.262) and σ_s = 0.24·0.7/3.14 = 0.0535 per channel.
  - Per step: light = [ambient·(n_y·0.3+0.7)·exp(−d_end·t·ε) + sun·shadowmap(coloured through glass)·
    cloudshadow·phase·exp(−d_sun·t·ε·1.1)]·σ_s.
  - The step is integrated analytically: (light − light·e^{−ε·dt})/ε, times the transmittance so far.
  - phase = fogPhase(V·L)×5, a sharp forward lobe: exp2(−15·(1−μ')^0.3)·1.5 + sqrt(exp2(−12.5·sqrt(1−μ'))).
  - The ambient is the sky's average with clouds (/30) × the sky lightmap curve (1−(1−(1−lm)^0.5)^3).
    The minimum is (0.01, 0.2, 0.4)·0.03.
  - Result: floor·T + in-scatter. The body's colour emerges from σ_s/ε ≈ (0.128, 0.205, 0.204), a cyan-teal
    lit by the sky's average and, toward the sun, by a bright forward glow.

**Ours** (`world.wgsl:1016-1069` `water_under`, `:1071-1217` `water`, `:1671` `WATER_DEEP`,
`look.wgsl:583` `water_body`)
- With `look_water_absorb` 1 the absorption is Bliss's exactly: the same ε, d = 2·through·|v.y|, and the same mix
  toward exp(−8ε). **Same.**
- The body's light ("fill") is a hand palette, not a march:
  - shallow teal (0.07, 0.27, 0.20) to deep navy `WATER_DEEP` (0.018, 0.045, 0.085) by probed block depth
    (1 − e^{−0.35·depth}, up to 8 blocks probed);
  - × the biome tint (half-desaturated, 15-35 %) × the texture's luma pattern (±20 % within 8-24 blocks);
  - × `lit` (light_on with an up normal: sun with shadow, no phase);
  - 40 % toward grey;
  - × `water_body` 0.35;
  - then fill = that × (1 − e^{−ε·min(through,12)}).
- Stacked fixes: the depth probe fade at 72-104 blocks (PROBE_REACH); the bottom veiled again by `haze_veil` and
  `mist_kept`; the old 5-constant path (gone = exp(−(0.42, 0.11, 0.07)·through)) is still there when absorb is 0.
- Far water (`world.wgsl:1676-1700` `far_water_lit`) is WATER_DEEP × lit × body, a flat colour.

**Verdict: absorption the same; body light missing (ours is a hand palette, 5-6 stacked colour fixes).**
Rebuild:
- In `water_under`, replace `deep`/`scatter` with Bliss's closed form. It needs no march if you take one
  sun-shadow tap at the floor, or the surface's own shadow:
  in = σ_s·(ambient_sky·(0.3n_y+0.7)·A + sun·shadow·phase(V·L)·5·S)·(1 − e^{−εL})/ε, with L = min(2·through, 12)
  and A, S the mean extinctions exp(−εL/2) and exp(−1.1εL/2).
- Delete `WATER_DEEP`, the shallow teal, the biome tint, the pattern, the 40 % grey, `water_body` and the
  `look_water_absorb` switch (keep Bliss's path only).
- Far water uses the same formula with L = 12.
- Pack only, about −40 / +25 lines. Bliss's two march steps with real shadow-map taps are a +15-line option.

#### A3. Refraction

**Bliss** (`dims/composite3.fsh:212-226`): a screen-UV offset of tangentNormal·mix(0.01, 0.1, (1 − dist/128)²),
rejected where the moved texel is not translucent. It is a cheap wobble, strong near and nearly none far.

**Ours** (`world.wgsl:1022-1037`): a real Snell bend (n = 1.33), marched up to 6 blocks, re-projected, rejected if
it lands above the water.

**Verdict: ours more physical, keep.** The visible gap: Bliss's offset uses the 10× amplified wave normal, so its
near water wobbles more. Tune only if the A/B shows it.

#### A4. Reflections: Fresnel, sky reflection, the sun's highlight

**Bliss** (`dims/all_translucent.fsh:717-830`)
- Water is hardcoded smoothness 1, f0 0.02, roughness 0.
- Fresnel = mix(f0, 1, (1 + n·v)^5), with n·v negative toward the eye.
- Sky reflection = the baked sky-with-clouds texture (skyCloudsFromTex, /30) in the reflected direction, with no
  lift. "indoors" uses the sky lightmap: indoors = (1 − (1 − clamp((lm − 0.6)·3))^0.5)^2. Below that, the water's
  own lit colour stands in (0 for water).
- Sun highlight = Direct_lighting × GGX(n, v, l, max(roughness, 0.035), f0):
  - r' = 0.035^2.5 = 2.3e-4, D = r'/(π(N·H²(r'−1)+1)²), so the peak is about 1390;
  - F is Schlick in the exp2 form f0 + (1−f0)·2^{(−5.55473·L·H − 6.98316)·L·H};
  - visibility 1/(L·H²(1−k)+k) with k = r'/4;
  - × N·L, which Direct_lighting already holds with the sun's shadow and cloud shadow³.
  - No cap. A very sharp, very bright sun path that bloom then spreads.
- Water's sun path does not use Sun_specular_Strength (that is for LabPBR terrain, A7).
- Compose: alpha = F; rgb = (sky or SSR)·F + sun. The body is under it with weight (1 − F).

**Ours** (`world.wgsl:1088-1216`, `:1657-1668`)
- Fresnel: with `look_water_mirror` 1 the same (0.02 + 0.98(1−c)^5); the ^4-from-0.04 path is still kept as an option.
- Sky: `sky_color` in the reflected direction with r.y·lift (lift 1 with mirror on, else 0.85), y ≥ 0.03,
  × mix(0.15, 1, brightness_curve(sky light)), our indoors.
- Clouds come in only where the SSR pass finds the screen's sky (A5). (unsure) whether `sky_color` holds the
  soft clouds.
- Sun: a GGX with a² = 0.0016 (α 0.04) × F × 0.25, plus a "sparkle" lobe a² = 0.00012 × 0.02 near only (12-40 blocks).
  It is capped at 40, and at 1.5 in golden hour, × palette sun × shadow × a horizon fade.
- RT reflections option (ours only).
- Foam, now off by default.

**Verdict: Fresnel same; sun highlight different (ours wider, dimmer, capped, two lobes); the sky lacks clouds off-screen.**
Rebuild:
- Use one GGX at α² = 0.035^2.5 with Bliss's Schlick-exp2 F and its visibility term, × sun × shadow, no cap.
- Delete the sparkle lobe, the golden-hour cap, the lift and the ^4 path.
- Reflected sky with clouds: needs a sky-and-clouds lookup in the reflected direction. Bliss bakes one small texture
  per frame. Ours has the physical sky view texture (`look_sky_phys`), but clouds are a separate pass.
  Engine/pack work: a low-res cloud layer into that texture (medium cost).

#### A5. Screen-space reflections on water

**Bliss** (`dims/all_translucent.fsh:250-303, 795-807`)
- Done in the water's own fragment shader.
- A clip-space march of mix(15, SSR_STEPS 30, F) steps toward the screen edge, against a quarter-resolution
  depth (colortex4.a), with a jittered start and a thickness test.
- A hit samples the previous frame's TAA output (colortex5) at the reprojected position. It is unblurred, and
  replaces the sky entirely (Reflections.a = 1).
- A miss uses the sky texture. Final: mix(sky, SSR, hit)·F·visibility.

**Ours** (`assets/shaders/post/reflect.wgsl`)
- A post pass after TAA.
- It finds water by the alpha mask, then re-derives the surface normal from depth plus its own copy of the waves.
- The march: 28 world-space steps growing ×1.245 (16 ×1.5 on Low), then 5 bisection steps.
- A hit blends 5 taps spread by the distance travelled (0.002-0.014 UV).
- Stacked on top:
  - fades by screen edge, by travel (300-700 blocks) and by the view distance's square;
  - "+0.15 extra when not Bliss's mirror" (off now);
  - a miss searches up to 6 steps up the screen for sky pixels, else keeps the pack's sky.
- Composition: col·(1 − a) + mirrored·a. That multiplies the body under the water by (1 − F) a second time,
  since `water()` already weighted it by (1 − F), and leaves F(1 − F) of the old sky in.

**Verdict: ours more complex; one composition error (double Fresnel on the body).** Rebuild:
- Smallest: in `apply`, subtract the sky's share and add the mirrored one, instead of lerping.
  That is col − a·sky_pack + a·mirrored, or write the water's F and sky into the world image so the pass can
  swap only them.
- Larger and closer: march in the water pass itself, against a half/quarter depth copy of the opaque world.
  The engine already copies "behind" (`scene.behind`). Then delete reflect.wgsl's wave copy, the normal-from-depth,
  the 5-tap blur and the upward sky search. Medium engine work: the water pass reads the behind depth plus last
  frame's TAA picture.

#### A6. Caustics

**Bliss** (`lib/waterBump.glsl:1-29`, used in `dims/composite1.fsh:1081` and `dims/composite2.fsh:302-305`)
- The floor point is projected along the sun to y = 0 (p − sun/sun.y·p.y). The same three wave taps are folded:
  ((|2n − 1|)·2 − 1)², summed.
- C = exp((1 + 5√W)·(Σ/3 − 0.5)).
- On the floor the sun is × mix(1, C·WATER_CAUSTICS_BRIGHTNESS(1) + 0.25, clamp(depth, 0, 1)). It darkens between
  the lines as well as brightening them.
- In the under-water march: × C·mix(0.25, 10, near-eye "bubble" e^{−7·(1−dist/16)}) + 0.75. These are beams near the eye.

**Ours** (`world.wgsl:1004-1010`, `:1062-1066`)
- Two noise ridge sets, pow 6 and pow 8, on the floor's xz only (no sun projection).
- Added to the bottom: bottom·gone·C·light·shadow·e^{−0.25·through}·smoothstep(0.1, 0.8, through)·0.9.
  It only brightens.

**Verdict: different.** Rebuild: use Bliss's sun-projected form from the shared wave height (after A1), and make it
multiplicative on the floor's sunlit share. With only the screen copy of the floor, approximate that as
bottom·(1 + k·(C·1 + 0.25 − 1))·sun_share. Same size, pack only.

#### A7. Reflections on terrain (LabPBR switches in the profile) and rain puddles

**Bliss** (`lib/specular.glsl:180-298`, `dims/composite1.fsh:1293-1296`, `dims/all_solid.fsh:330-345, 494-497, 546-553`)
- Every terrain pixel gets F0 = max(spec.g, 0.02). Without a LabPBR pack, smoothness is 0, so roughness = 1.
- **The sun's specular is added to all lit terrain regardless:** Diffuse(sun·shadow·N·L) × GGX(r = 1, f0 0.02) ×
  Sun_specular_Strength **10**.
  - With r = 1: D = 1/π, k = 0.25.
  - Toward a frontal sun the term is about 0.064·N·L × sun. That is comparable to a dark albedo's diffuse (+30-40 % on grass).
  - Toward a low sun at grazing L·H, F (Schlick exp2) rises toward 0.3-0.5 and the term grows several times.
    Terrain facing a low sun gets a bright, broad sheen.
- Sky reflection and SSR: only where F0·(1 − R·1.3) > 0.01, so smoothness above about 0.38.
  - That never happens without LabPBR except in rain: rain sets smoothness ≥ rainStrength·(sky lm−0.9)·10 on
    open ground, so wet ground becomes a smooth mirror with F0 0.02.
  - With Rough_reflections the ray uses VNDF sampling of the roughness. SSR blurs by mip with the ray length.
- Puddles: Puddle_shape = clamp(lm − exp(−15·noise(xz·0.02)^5)) × up-facing × rain flattens the normal.
  With Porosity and no LabPBR, porosity = 0, so no darkening.

**Ours** (`world.wgsl:778-862` mat_shine and mirror_alpha, only for textures with LabPBR maps (`gpu_layout.rs:1443-1504`);
`world.wgsl:688-733` wet sheen and puddles)
- No specular at all on map-less terrain.
- Rain: ground darkened ×0.58 plus a sheen (0.04 + 0.3(1−n·v)^4)·sky; hashed 3-block puddle cells mirroring
  `sky_color` (no SSR on them unless mirror_alpha).

**Verdict: missing (the sun sheen on all terrain, and wet-ground SSR); rain puddles ours-different.**
This is a probable unlisted gap in low-sun shots. Rebuild:
- In `surface_light`, add Bliss's GGX(r = 1, f0 0.02) × 10 × the sun's direct light for every lit pixel.
  About 15 lines, cost negligible. Note it belongs to the lighting section too.
- Rain: replace the darkening, sheen and puddle cells with Bliss's (smoothness by rain, F0 0.02, flattened puddle
  normal, no darkening without porosity). Write −F into `mirror_alpha` so reflect.wgsl traces wet ground.
  Pack only, about −30 / +20.

#### A8. The surface seen from below (Snell's window)

**Bliss** (`all_translucent.fsh:772-775, 788`):
- From under water, Fresnel = clamp(1.5 + n·v)^25: total reflection outside the window and see-through inside.
- What it reflects is exp(−8ε)·clamp(sun.y·light), the deep water's colour.
- Inside the window the real sky and clouds show through (refracted by the UV offset).

**Ours** (`world.wgsl:1090-1094`): opaque. The colour is `underwater_color()`, a hand teal
(0.040, 0.19, 0.15)·(ambient·1.4 + sun·0.5)·1.8, × mix(0.6, 2.4, smoothstep(0.55, 0.85, −v.y)).
The sky is never seen through it.

**Verdict: missing.** Rebuild: alpha = 1 − F with Bliss's ^25 window, the reflected colour exp(−8ε)·sun.y·light,
and see-through to the sky behind. The translucent pass already blends. About 10 lines; delete `underwater_color`'s
use here.

#### A9. The view from under water (eye in water)

**Bliss** (`dims/composite3.fsh:409-433`, `dims/composite2.fsh:229-310, 431-442`, `composite3.fsh:445-449` bloom)
- Scene × absorption: exp(−ε·dist) blended with an 8-block floor by the scene's own brightness.
  Mixed toward sun.y·(n_y·0.3+0.7)^1.5·exp(−8ε) by a fog fade exp(−0.001·dist²).
  Here dist uses only the downward part of the view.
- Plus an 8-step march to 32 blocks: ambient plus sun·shadow map·cloud shadow·phase·caustics(bubble).
  It gives sun beams and in-scatter, with σ_s = 0.15·0.7/3.14.
- The fog alpha passed to bloom is × 0.4, so heavy bloomy fog under water. Minimum light ×10 under water.

**Ours** (`look.wgsl:2832-2875` `underwater_color`, `under_water`)
- A hand teal fog: mix(c·mix(0.6, 1, down), teal·mix(0.35, 1, down), 1 − e^{−dist/18}), with depth falloff
  exp(−(surface − y)/28).
- Plus 6 noise "beams" along the ray. No shadow map, no phase.
- No bloomy fog under water: `fs_pre_taa` skips in liquid, and `mist_over`/`bank_hidden` give 0.

**Verdict: ours simpler and hand-coloured; bloomy fog missing.** Rebuild:
- Use Bliss's absorption + an 8-step march (ε, σ_s, phase, a sun shadow tap per step, caustic beams) in
  `fs_pre_taa`'s liquid branch, which today returns early. The one-fog march already has this shape.
- Pass a fog transmittance of 0.4·… to the bloomy-fog mix (B3).
- Delete `underwater_color` and the 6 noise beams. About +40 / −35 lines, pack only.

---

### Part B: the post chain, pass by pass

Bliss's order, after the world is lit and fogged:

| Step | Bliss file | What it does |
|---|---|---|
| 1 | dims/composite3 | refraction, translucent blend, fog, bloomy-fog alpha into colortex7.r |
| 2 | dims/composite5 | TAA → colortex5 (fp10 dither) |
| 3 | dims/composite6/7 | bloom down to 1/2, 1/4 |
| 4 | dims/composite8/9 | separable Gaussians per tile |
| 5 | dims/composite10 | bicubic upsample-sum of 7 levels |
| 6 | dims/composite11 | DOF → bloomy fog mix + bloom add → × exposure → Purkinje → Hejl → sRGB → int8 dither |
| 7 | dims/composite12 | sharpening → saturation → (luma curve, grade: off) → int8 dither |
| 8 | final | water-on-camera, damage effects (motion blur off) |

Exposure itself is computed in `dims/deferred.vsh` from last frame's bloom tiles.

Ours (`crates/engine/src/post.rs:899-1340`):
1. `fs_pre_taa`: the one-fog march.
2. TAA (`crates/engine/assets/shaders/taa.wgsl`).
3. The effects in order: reflect, contact, far_ao, volume, shafts, bloom (`mods/base/src/shaderpack.rs:112-118`).
   The last effect's pass also runs `hdr_light` (chain_light) → `finish_picture` (`mods/base/shaderpack/post.wgsl:84-101`).
4. Bloom pyramid and exposure (`post.wgsl:151-252`), drawn after the effects' entries but read by the last effect
   pass of the same frame.

#### B1. Auto exposure

**Bliss's method** (`dims/deferred.vsh:222-268`, stored in colortex4 texel (10,37), read in composite11.vsh:21)
- Each frame, 50 points (R2 sequence, a new set each frame) in the centre 70 % of the screen sample a blurred bloom
  tile from last frame (the 1/16-1/32 level).
- Geometric mean of luma (Rec.601: 0.299, 0.587, 0.114).
- Smoothing: L = mix(mean_now, L_prev, 0.95) per frame. That is linear, symmetric and frame-rate dependent:
  about 0.33 s at 60 fps. Exposure_Speed is listed in the menu but unused in the code.
- Curve: E = 0.18 / log2(2.5·L + 1.045) × 0.62 × EXPOSURE_MULTIPLIER(1). No clamp beyond ≥ 0.
- Rods: L2 = the geometric mean of min(dot(c, (0.07, 0.22, 0.71)), 0.08), a per-sample clamp on a scotopic-like
  weight, smoothed 0.985/frame. rodE = max(0.012/log2(L2 + 1.002) − 0.1, 0)·1.2.

**Ours, chain on** (`post.wgsl:229-252` measure; `post.wgsl:416-439` and `look.wgsl:246-275` curve)
- An 8×8 fixed grid on the narrowest pyramid level (1/64). Centre-weighted 1 → 0.3 from r 0.1 to 0.75.
  Log-mean of Rec.709 luma.
- Eased in log space: τ 0.8 s brighter, 2.5 s darker, by real frame time.
- Curve: `bliss_exposure` = 2.2·log2(2.5·0.35·41 + 1.045)/log2(2.5·mean·41 + 1.045), then × 0.389 inside `hejl_bliss`.
  In our units that is 4.46/log2(102.5m + 1.045) against Bliss's 4.58/log2(…), 2.6 % lower. **Same curve.**
  BLISS_UNITS = 41 is the unit conversion.
- Layers stacked on it:
  - `look_chain_night` 0.08 scales the night's sun, sky light, zenith, horizon and glow (`look.wgsl:795-812`).
    It is a fix for our night light units, not an exposure step.
  - CHAIN_AMBIENT_GREY and rain greying in the same place (lighting, not post).
- The non-chain path (`post.wgsl:259-287, 442-571`: day/night power, 0.83, ceilings, rain cap, land dim, dusk lift,
  dawn lift in finish_picture:3393-3401, sky saturation and dim) is dead code with chain = 1.

**Verdict: curve same; measuring nearly the same; easing different (asymmetric log vs symmetric linear 5 %/frame);
rod mean different.** Rebuild:
- Match the easing (mix 0.95 per 1/60 s, or its time-based equivalent exp(−dt/0.33)) and Rec.601 luma.
- Measure from the 1/16-1/32 level, centre 70 %, unweighted.
- Add a second texel for Bliss's rod mean (per-sample min 0.08 of (0.07, 0.22, 0.71)).
  Engine: one more channel in `fs_exposure`'s 1×1 target (b is free). About 15 lines.
- Delete the whole non-chain `hdr_light` body, `auto_exposure`, `exposure()`, the dawn lift, the glare, the land and
  sky dims and the `display()` night blue: about −250 lines once chain = 1 is final.
- `look_chain_night` should go when our night light matches Bliss's moon/sun ratio (moon_illuminance 200 /
  sun_illuminance 250000, about 1/1250, per Ronald's profile). That is a lighting task.

#### B2. Bloom

**Bliss's method** (`dims/composite6.fsh`, `composite7.fsh` downsample; `composite8.fsh`, `composite9.fsh` Gaussians;
`composite10.fsh:81-112` upsample; `composite11.fsh:79-154` mix)
- Source: the TAA output (pre-exposure HDR), no threshold, no Karis average.
- Down: a 13-tap box (½ on the 4 diagonals at ±1, ⅛ on the ±2 cross and the ±2 diagonals, ⅛ the centre) to ½,
  then ¼ per axis, then tiles down to 1/128.
- Each tile is blurred by a separable Gaussian, weight exp(−4α·i²) on bilinear pairs (2 texels a tap). Per tile:

  | Level (per axis) | α | Taps | σ, own texels | σ, full-res px |
  |---|---|---|---|---|
  | 1/4 | none | 0 | none | none |
  | 1/8 | 0.16 | ±3 | ≈1.8 | ≈14 |
  | 1/16 | 0.035 | ±6 | ≈3.8 | ≈60 |
  | 1/32 | 0.0085 | ±12 | ≈7.7 | ≈245 |
  | 1/64 | 0.002 | ±28/30 | ≈16 | ≈1000 |
  | 1/128 | 0.0005 | ±60 | ≈32 | screen-wide |

- Up: a bicubic (B-spline) tap of each of 7 levels (½ … 1/128), weights 1, ½, ⅓, 1/5.5, ⅛, 1/10, 1/12.
  Normalised: 0.43, 0.215, 0.143, 0.078, 0.054, 0.043, 0.036 (the ×3 and ÷3 cancel).
- Mix (composite11): col = mix(col, bloom, VL_abs) + bloom·lightScat.
  - lightScat = clamp(BLOOM_STRENGTH(1)·0.5·E^0.2, 0, 1)·(1.5 − 2r²), with r² = |uv − ½|²: 1.5 at the centre,
    0.5 at the corners. That is Bliss's only "vignette", and it only scales bloom.
  - Then × E.

**Ours** (`post.wgsl:151-222`, `crates/engine/src/post.rs:851-890, 1273-1316`; `core:bloom` 6 levels)
- 6 levels ½ … 1/64 per axis. The 13-tap Jimenez down, with a Karis average on the first. A 3×3 tent up.
- Each up level = (own + 1.3·tent(wider))/2.3. Weights ≈ 0.435, 0.246, 0.139, 0.079, 0.044, 0.058, close to
  Bliss's per-level shares.
- But no extra Gaussian per level. The widest effective blur is about σ 100 px at 1080p, against Bliss's 245/1000/
  screen-wide on its last three levels. Bliss's glow is several times wider: a big soft halo round the sun, bright sky
  and lamps, and a general veil.
- Chain mix (`post.wgsl:427-437`): the add term is the same as Bliss's
  (0.5·E_bliss^0.2 clamped × (1.5 − 2r²) × `look_bloom`).
- BLOOM_SPREAD 1.3 is a tuning constant.

**Verdict: partly. The per-level weights match; the radius is far too small; one level short.** Rebuild:
- After each down level ≥ 1/8, add Bliss's separable Gaussian at its α and tap count. That is 2 extra passes per level,
  or fold the H and V into the down and up passes.
- Add a 1/128 level (`bloom_levels` 7).
- Up-sample with weights 1, ½, ⅓ … in one final sum, bicubic, instead of the chained tent and 1.3 spread.
  Delete `BLOOM_SPREAD` and the Karis average (Bliss has none; keep only if fireflies show).
- Engine: about 10 extra small passes, all at ≤ 1/8 size, cheap. The pyramid code in post.rs grows by about 40 lines.
  This is the "lkbl1" item Planner held.

#### B3. Bloomy fog (post side)

**Bliss** (`dims/composite3.fsh:437-449, 480`, `composite11.fsh:145-154`)
- Fog alpha a = the fog march's transmittance vl.a; in rain a = mix(vl.a, ½vl.a + ½, rain); × 0.4 under water;
  × (1 − (5·raindrop)²) where drops are on screen; 0 in powder snow.
- VL_abs = clamp((1 − a)·BLOOMY_FOG(**1.25**)·0.75·(1 + rain)·(1 − 0.3·purkinje), 0, 1) × (1 − cdist^15),
  where cdist = max |uv − ½|·2, a fade only at the very screen edge.
- col = mix(col, bloom, VL_abs).
- So up to 0.94 of the picture becomes the (wide) bloom where fog hides everything. More in rain (×2) and under water.

**Ours, chain** (`post.wgsl:427-432`)
- Two mixes:
  - (a) froxel mist cover × 0.6, cut 85 % in rain (froxels are off with fog_one, so about 0);
  - (b) `bank_hidden`, a **second 8-step march** of the mist banks alone (`look.wgsl:2379-…`), × 0.94, × (1 − 0.85·rain),
    0 in storms.
- The one-fog march's own transmittance (air, rain fog, banks together) is not used. Under water: none.

**Verdict: ours more complex and opposite in rain.** Rebuild:
- `one_fog` writes its transmittance into the pre-TAA image's alpha, or a small R8 target; it is already one march.
- chain_light uses Bliss's single formula with rain ×(1 + rain) and water 0.4.
- Delete `mist_over`'s use, `bank_hidden` (−60 lines, and one march per pixel saved), and the rain cuts.
- Engine: the pre-TAA pass gets a second output, or alpha survives TAA. TAA already carries alpha (the water mask),
  so a separate target is cleaner. Small.

#### B4. Purkinje (night)

**Bliss** (`composite11.fsh:145-160`):
- purk = clamp(E², 0, 1)·clamp(rodE/(1 + rodE)·Purkinje_strength 1).
- lum = c·(0.15, 0.3, 0.55); rodLum = c·(0.85, 0.7, 0.45)·200; curve = clamp(mix(1, rodLum/(2.5 + rodLum), purk)).
- col = mix(lum·(0.4, 0.7, 1.0)·5, col, curve), on exposed light before the tone curve.

**Ours, chain** (`look.wgsl:260-272` `bliss_purkinje`): the same formula. But rodE comes from min(mean, 0.08) of the
luma mean, not Bliss's per-sample scotopic clamp mean (B1).

**Verdict: same formula; input partly.** Fix with B1's rod texel. Delete the non-chain night blue in `display()`
(`look.wgsl:3341-3373`), the storybook rods, the cool floor and the near-black desaturation (`:3477-3518`): dead with the chain.

#### B5. Tone curve (ToneMap_Hejl2015)

**Bliss** (`lib/color_transforms.glsl:34-40`, applied in `composite11.fsh:164`):
- v = 0.85·x, w = 3 (in v units, so x ≈ 3.53 maps to white). a = 1.75v + 0.05.
- f(v) = (v·a + 0.004)/(v·(a + 0.55) + 0.0491) − 0.0821 + 0.000633604888.
- Out = f(v)/f(3), then the exact piecewise sRGB encode (12.92 / 1.055·x^(1/2.4) − 0.055). So the curve's output,
  which already has a gamma baked in, is gamma-encoded again: deep darks, steep mids.

**Ours** (`look.wgsl:3271-3276` `hejl_bliss`): the same formula on x·0.389. The output is taken as linear and the
swapchain sRGB-encodes it. **Same.** (The old `hejl` with 1.425 and pow 2.2, `lottes` and `agx` are kept for the
non-chain path.)

**Verdict: same.** Delete `lottes`, `hejl`, `agx` and the `look_curve` switch when the non-chain path goes (−70 lines).

#### B6. Saturation and grading

**Bliss** (`composite12.fsh:120-128, 142-150`):
- After the curve, in sRGB-coded values: c + (c − luma601(c))·(SATURATION **0.3** − luma·CROSSTALK 0) = 1.3×
  away from grey.
- LUMINANCE_CURVE, COLOR_GRADING_ENABLED and CONTRAST 1 are off. Nothing else.

**Ours, chain** (`look.wgsl:3383-3391`): 1.3× on pow(t, 1/2.2) with Rec.601 luma, then back to linear, at all hours,
before the sharpen. Bliss sharpens first. Plus a blood moon red cast (ours, gameplay).

**Verdict: same (tiny gamma-approximation difference).** The non-chain grade (white balance, split warm, leaf yellow,
near-black cut, sky saturation) is dead with the chain: delete (−120 lines).

#### B7. Sharpening (SHARPENING 0.4)

**Bliss** (`composite12.fsh:103-118`), after the tone curve in display space:
- Four bilinear taps at the half-texel diagonals; each is a 2×2 average including the centre, so their mean is a
  3×3 tent blur T.
- contrast k = 1 − luma(Σ|tap − m1|)/5. Large local contrast means less sharpening.
- Algebraically out = c + (4/3)·S·k·(c − T) = c + **0.53·k·(c − T)** at S = 0.4. An unsharp mask scaled down at edges,
  not AMD's CAS.

**Ours** (`post.wgsl:298-338` `sharpen_delta`, added at `post.wgsl:98` before `hdr_light`)
- AMD-CAS-style weights from the min/max of a 5-tap cross on log2(1 + luma), in **HDR before exposure, bloom and
  the curve**.
- Plus a motion push up to 2× at 6 px/frame, clamped to the neighbourhood.

**Verdict: different method, different space.** Rebuild:
- Bliss's tent unsharp mask on the finished (curved, saturated-before-not-after) value.
- Without a new pass: run the exposure, Purkinje and Hejl on the 4 diagonal bilinear HDR taps inline in
  `finish_picture` (4 more curve evaluations, cheap), or add a final pass after the last effect (small engine work).
- Keep the motion push only if TAA stays ours (B8). Then saturation after it, as Bliss.

#### B8. TAA

**Bliss** (`dims/composite5.fsh:291-395`)
- Velocity from the nearest of 5 depth taps (2-texel cross).
- The current frame is sampled de-jittered, with a smoothstep-filtered bilinear: a slight softening.
- Box = average of the 3×3 min/max and the 5-tap plus min/max, in RGB. A clamp, not a clip.
- History via a 5-tap Catmull-Rom with **sharpness 0.75** (standard is 0.5), so a sharper history.
- Blend = BLEND_FACTOR 0.12 + luma(min(max(clamped − hist, 0)/hist, 1)): more current where the history was clamped.
- Blended in reversible tonemap space c/(1 + luma). Off-screen: current. Hand: velocity-based. fp10 dither on output.

**Ours** (`crates/engine/assets/shaders/taa.wgsl:164-256`; `core:taa_blend` 0.1 in `crates/engine/src/config.rs:769`)
- Karis-toned YCoCg variance box (mean ± 1.25σ inside min/max), clipped toward the centre.
- Raw jittered current texel.
- Standard Catmull-Rom (0.5).
- A frame count (up to 10, so a minimum blend 0.09), reset by the clip amount (gave·2), raised with motion
  (n ≥ 4 at 8 px).
- Rain reactive mask (×0.8). A far-end exception for the LOD dither. Alpha (water mask) carried.

**Verdict: ours more complex but handles more (rain, LOD fade, upscalers).** The visual differences:
- Bliss is softer in still frames (de-jittered smooth bilinear current, min-max clamp);
- Bliss is sharper in history (c 0.75);
- Bliss ghosts a little more.
- Rebuild only if A/B shots show a sharpness gap: try c = 0.75 in `taa_history` (1 line) and blend 0.12.
  Keep the rest (engine-owned, used by DLSS paths). Low priority.

#### B9. Vignette, glare, lens

**Bliss**: no darkening vignette. Only bloom is centre-weighted (1.5 − 2r², B2). No glare, no lens dirt, no chromatic
aberration. Gameplay: water drops on the camera after leaving water (WATER_ON_CAMERA_EFFECT, `final.fsh` via
`lib/gameplay_effects.glsl:63-86`); low-health and damage effects.

**Ours**:
- `assets/shaders/post/bloom.wgsl:10-13`: in the HDR world the bloom *effect* still applies a **20 % corner
  darkening**: ×(1 − 0.2·smoothstep(0.55, 1.15, d)), d the aspect-corrected radius×2.
  About −17 % at the left and right edges and −20 % in the corners. It is on whenever `fx_bloom` > 0 (default 1).
- Veiling glare (`post.wgsl:543-544`) is only in the non-chain path, and `look_glare` is 0.

**Verdict: ours extra (a visible one).** Delete the vignette line in bloom.wgsl's HDR branch, or make that branch
return the colour unchanged. 1 line, no engine work. The water-on-camera drops are missing (optional, gameplay).

#### B10. Depth of field (DOF_QUALITY 3, focal 1.9): **on in Ronald's profile**

**Bliss** (`composite11.fsh:100-131`, `lib/bokeh.glsl:170` 96 samples, `settings.glsl:528-535`, autofocus `deferred.vsh:259-261`)
- DOF_QUALITY ≥ 0 turns it on; 3 means 96 bokeh offsets.
- MANUAL_FOCUS −2 (default) autofocuses on the centre pixel's depth. Eased by
  clamp(DoF_Adaptation_Speed(1)·exp(1 − 0.016/frameTime)/(6 + depth·far)), so faster for near targets.
- Circle of confusion (UV): pcoc = min(|A·(f/100)·(z − focus)| / (z·(focus − f/100)), 15 texels), with
  f = focal 1.9, fstop 2.8, so A = f/fstop = 0.679 and f/100 = 0.019 blocks.
- The colour is the mean of 96 taps of the TAA picture at offset·pcoc·(anamorphic 1, aspect), each read at mip
  LOD pcoc·200. Before bloom and exposure.
- Size: focus 300 blocks, a tree at 20 blocks: pcoc ≈ 6·10⁻⁴ UV, about 1.2 px radius at 1080p (a soft
  foreground). Focus 5 blocks, far land: about 5 px radius. The cap is 15 texels.
- In vista shots (focus far) the near ground and trees are slightly soft. In close shots the far land is clearly blurred.

**Ours**: none.

**Verdict: missing.** The refs carry this softness, so it is part of "look the same". Rebuild:
- One full-screen pass before the bloom mix: autofocus texel (centre depth, eased) in the exposure target's spare channel.
- Bliss's CoC formula; a 32-64-tap disc on a mipped copy of the TAA output (the bloom pyramid's ½ and ¼ levels can
  stand in for the mips).
- Engine: a mip chain or reuse of the pyramid plus one pass, small to medium. About 60 lines.
- Worth an A/B on the matrix first: it may explain some of the "ours is crisper/noisier" notes.

#### B11. Dithering

**Bliss**: a triangular white-noise dither of ±1 LSB.
- fp10 mantissa-relative on the TAA output (`composite5.fsh:432`);
- int8 (±1/256) after the tone curve (`composite11.fsh:172`) and again after sharpening and saturation (`composite12.fsh:157`).

**Ours**: none in the HDR chain. Float16 targets, an 8-bit swapchain.

**Verdict: missing.** Possible banding in sky gradients and fog at dusk. Add ±1/255 triangular noise in
`finish_picture` before the clamp (3 lines), moving with `scene.frame.w`.

#### B12. Motion blur

Off in both (MOTION_BLUR commented). Nothing to do.

---

### Ranked: the biggest visual gaps in this area

1. **Bloom radius (B2).** Bliss's Gaussian-blurred levels reach σ ≈ 250 / 1000 px / screen-wide; our pyramid tops out
   near 100 px. Its wide halo round the sun, sky and lamps, and its general soft veil, are what most of "Bliss's glow"
   is. It also feeds the bloomy fog.
2. **DOF is on in Ronald's profile (B10).** It is missing in ours: soft foregrounds in vistas, blurred backgrounds in close shots.
3. **Sun sheen on all terrain (A7).** Sun_specular_Strength 10 with F0 0.02 and roughness 1 gives a strong
   low-sun-facing sheen and frontal brightening; ours has none on map-less blocks. Rain-wet ground as an SSR
   mirror is also missing.
4. **Water body colour (A2).** A hand palette (navy, teal, tint, pattern, grey, ×0.35) against Bliss's physical
   in-scatter with a forward sun lobe. The absorption already matches.
5. **The corner vignette in bloom.wgsl (B9).** −20 % corners that Bliss doesn't have; a 1-line fix.
6. **Water's sun path (A4).** Bliss's α ≈ 0.015 uncapped GGX against our wider, capped two-lobe glint.
   The reflected sky lacks clouds off-screen.
7. **Bloomy fog (B3).** Not from the real fog transmittance, cut in rain where Bliss doubles it, none under water;
   a second mist march to delete.
8. **Sharpening (B7).** Wrong space (HDR before the curve) and method. Then: under-water view and Snell's window
   (A8, A9), caustics (A6), no dither (B11), TAA sharpness (B8), double-Fresnel in reflect.wgsl (A5).
   Exposure, Purkinje, Hejl and saturation (B1, B4-B6) are already the same: what is left there is deleting about
   450 lines of the dead non-chain path.

### Seen on the matrix (lkmxO1, Planner's read, 11-25)

- **Underwater** (underwater_shore_y90): Bliss's is a dark, murky green-brown; ours a bright mint. Bliss's
  8-step water march (σ (0.360, 0.184, 0.168), scatter 0.0334) and its absorption mix in place of our teal
  `under_water` and its noise beams (Fog, section on underwater).
- **Shore water** (shore_noon_y0): Bliss's bright, rippled and reflective; ours a flat deep blue. Water body
  colour (A2), sun glint (A4) and the reflected sky with clouds above.
- **Caves** (cave_c4): Bliss's walls catch warm sun patches and a warm bounce; ours are a cool blue-grey with
  no sun patch. Lighting batch, first item: the tilt sign (+35 vs −35), the sun cut by sky light, the hand
  fill.
- **Night sky**: our stars and aurora far brighter than Bliss's near-starless navy. Bliss's stars fade in by
  clamp(−2 sun.y) at 10/800 of its sky's scale (Sky, section 6).
