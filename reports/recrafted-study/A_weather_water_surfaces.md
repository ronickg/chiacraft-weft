# Recrafted 0.2.1: weather, water and surface modules (technique notes, own words)

Scope: 06_builtin_shaders modules for precipitation, shelter, rain_puddles, lightning, fog, biome atmospheres, sky, clouds, volumetric light, water, reflections, ice, snow, sand, frosted leaves, atlas_heights. Everything below is a paraphrase of what the files do; no code is reproduced.

## 0. Shared plumbing that most modules lean on (read this first)

These are the real "big ideas". The individual effects are thin consumers of them.

1. **Post chains + persistent targets + "atlases".** Polytone runs full-screen post passes at priorities. Because chains cannot share many named targets, Recrafted packs unrelated data into a few big RGBA8 textures ("atlases") and addresses sub-rectangles via a layout include:
   - Atlas H (3104x6144): top-down heightmap depth (2048^2), its colour (2048^2), eroded height, a 1056^2 near sun-shadow map, 1024x2048 far shadow, 512^2 rain-shelter caster map, a glass map, and a min-pyramid (Hi-Z) for the heightmap.
   - Atlas S (258x321): sky-view LUT (258x192), a 128x128 cloud coverage map, a 128x96 sky model LUT, a 3x1 air-state texel.
   - Atlas R (1154x1281): biome fog/sky colour fields (320x320 each), cloud shadow (320^2), weather field (320^2) plus a 1-texel "near" summary, wake/water-wave height (512^2), cover (512^2).
   - Values are packed into 8-bit channels as 16- or 24-bit pairs. Persistence = a target flagged persistent, read as "Prev" the next frame (temporal sims, history, state machines).
2. **Top-down orthographic heightmap viewpoint** (viewpoints/heightmap.json): a second camera at y=320 looking straight down, ortho extent 256/512/1024 blocks chosen by render distance, snapped to a texel grid so it does not shimmer, solid+cutout terrain only, no entities, 4096^2 resolution, refreshed on a slow interval (update_interval 200, activation gated by which modules need it). The result is the world's "sky visibility oracle": anything can ask "is this point open to the sky", "what is the highest surface here", or "trace a ray over the terrain". It is used by shelter, puddles, reflections, volumetric light, foam, underwater light shafts, fog ground height.
3. **Surface map (surface_map.json):** a Polytone feature that exposes, per columns across render distance, the motion-blocking-no-leaves height and chosen biome attributes (fog end, fog start, fog colour, sky colour, plus climate). Post shaders read these as a ring-buffer indexed by column. This is how biome data reaches a screen-space shader without sampling vanilla biome blending.
4. **Weather field** (post_shaders/weather_field.json, always-on-shared): a 320x320 texture, 4 blocks per texel (1280-block span, recentred on the camera in 4-block steps). Per cell it stores: precipitates? (1/0 from biome downfall) and an encoded **snow line altitude** (biome temperature below 0.15 means snow always; above that the line is sea level + 17 + (temp-0.15)*800 blocks). rain at a point = precipitates AND y below the line (hard step); snow = precipitates AND y above the line (smoothstep with a 40-block soft band). A 32-cell strip around the camera is summarised into a single "everything nearby is the same" texel, so the heavy per-drop shaders can skip the texture fetch when the whole neighbourhood is uniform. This is the "rain falls as snow higher up the mountain" trick, per column, no CPU work.
5. **Global expressions (CPU-side, per tick):**
   - `wind_speed`: 0.40 + 0.15*sin(t*0.01309) + 0.10*sin(t*0.004712), times (1 + rain clamp). Two slow sinusoids = gusting; rain doubles it. `wind_x = -1`, `wind_z = 0`: the prevailing wind is a fixed constant direction (west), only speed varies. `water_wind`: nearly the same two-sine form (amplitudes 0.094 and 0.0625 around 1.0) times (1 + rain). `water_phase`: integrates a wave clock at 1 + 1.2*storm per tick-gated step so waves speed up (not jump) in storms.
   - `air_state`: packs two 10-bit values in one float: fog amount (view distance vs vanilla fog window, with rain shortening fog end from fe0 down to max(96, fe0-256*rain)) and a "daylight dim" curve (piecewise linear sunrise 22330-133, day, sunset 11867-13670). Cheap way to ship several uniforms through one number.
   - `lightmap_sky`, `sky_attr`, `fog_attr`, `cloud_rgb/a/height`: vanilla visual environment attributes (sky/fog/cloud colour, cloud height, sky light) re-exported as post uniforms.
   - `tick_gate`, `warmup` (counts ticks after join; atlases are force-built for the first 1200 ticks so late-activated modules don't pop).
6. **Alpha/marker tagging** (the single most reusable trick for engine-side thinking): terrain shaders cannot add passes, so they smuggle material info into spare bits.
   - Opaque terrain alpha carries flags: 254/255 = snow top face, 253/255 = sand top face; post passes find these pixels and replace/augment shading.
   - Water is written into the translucent target with alpha 254/255 and near-white texture; glass/ice with other tags; tags also go into low bits of translucent *depth* (RC_TTAG_WATER/GLASS/ICE/OTHER plus a light level code).
   - Geometry marks: custom block models contain elements with tiny fractional coordinates (e.g. an up-face split at x=8.09 on snow/sand, ice at 128/2048 of a texel, frost faces at codes 147/157/32768ths of a block). The vertex shader decodes "this vertex sits on a magic fractional offset" into a flag and passes it to the fragment shader. Pure modelling-side tag channel.
   - Vanilla lightning-bolt geometry is hijacked the same way (see 4).
7. **World normals:** a separate module (world_normal) writes world-space normals of translucent surfaces into a target (octahedral-like codec) so water/ice/glass post passes can shade analytically.
8. **Fog is moved out of terrain shaders.** Terrain fog start/end are set to ~1e9 when the post fog owns the environment; fog is applied once per pixel in a post pass (see 5).

## 1. precipitation

**Sees:** individual 3D rain streaks and snow flakes falling around the player, wind-slanted, gusting, in rolling fronts; distant rain as a veil; rain seen through water from below; no vanilla rain sprites.

**Technique:** vanilla rain/snow sprite quads are neutered (rain particle life/alpha forced to 0, drip/splash particles hidden while raining, 1x1 placeholder textures). Everything is a post pass chain (priority 88):
- Rain near the camera (<~36 blocks, 3 layers): a procedural field of drops in three skewed/rotated lattices (cell sizes ~1.2-1.6 blocks, different fall speeds ~14.7, 21.0, 29.0 b/s, per-layer yaw so the lattices do not align). Each drop's position is a hash of its integer cell plus time-scrolled offset (a wrap period so it repeats only over thousands of blocks). Each drop is drawn analytically per pixel as a capsule/line segment with a motion-blur tail (tail length = fall velocity * shutter time ~0.08 s), a thickness of 1/16 block, edge antialiasing by pixel footprint, 4-level pixel quantisation to keep the pixel-art look. Drops refract the background ("lens" mode): colour = mix of refracted sky from the sky LUT and scene, with a Fresnel-ish glint when looking toward the sun or moon, and a forward-scatter veil term (mu^6) for halo. Held-light torch glints and placed-light glints are added.
- Which drops exist is decided by culling passes: depth is reduced to small tiles (precip_tdepth at 0.07-0.26 scale), per tile a candidate drop list is built (`rain_tiles` at 0.52 scale writes up to ~54 entries per 8x8-texel cell, each entry a packed cell offset, layer, visibility and 4 sub-bin coverage bits), summed (`precip_tsum`), sorted/lined (`rain_tlist`, `rain_tline`) and masked (`rain_tmask`). The final pass only loops the tile's list (cap ~54, early-out when accumulated alpha > 0.985). It is a compute-style binning pipeline built from fragment passes.
- Each candidate drop is tested: (a) weather field (rain only below snow line, precipitating biome), (b) **shelter** (section 2), (c) a moving rain-front noise (value noise ~225-cell period, scale 4 x wind cell size, advected by wind velocity) that thins drops to 28%-100% density, so rain arrives in bands and curtains, (d) player eye in air only (not underwater).
- Far rain (beyond the geometric range): two cylindrical shells at radius ~23 and ~30 blocks textured from a 514x1542 sheet (`weather_far.png`: separate sheet blocks for rain, "beads" and snow) tiled 10x around, scrolled along fall direction, fading in over the drop range.
- Snow: same tile-list architecture (`snowfall`, `precip_tlist`) with a different flake model: three layers of 2-block cells, falling 1040-2240 ticks/block-ish with large lateral drift (DX 530-1142), per-layer gust amplitude (0.55/1.0/1.7) and phase, flakes 3/16 block, rolled quads with a 5/9-filled diagonal/plus pixel mask, sun forward-scatter, shadow-map lit, thunder adds 0.6 gust. Reach 24 blocks.
- Rain from below water: `rain_window` re-draws the rain as seen through Snell's window (refracting the view ray, tracing it over the heightmap to the sky, using shelter/open-column tests) with 3 shells and a total-internal-reflection tilt.
- Air haze: precipitation raises the aerosol rate in the fog march (see 5) and a veil lifts airlight; the unused RC_RAIN_VEIL option is off.

**Drivers:** `PolyRain` (rain level, window = clamp(2*rain)), `PolyBiomePrecip` (downfall>0 at player), `PolySnowLine` (camera biome temperature -> snow altitude), weather field, wind (fixed direction, speed gusting; rain streaks lean using wind and "swell" sinusoids with two frequencies), sky/fog attribute colours, sun/moon angle, held light, shadows config, volumetric clouds flag (changes lightning/cloud interplay), thunder strength. Activation: rain>0 AND sky type overworld-like, or first 1200 ticks.

**Cost:** heavy. ~15 passes; several quarter/half-res; main rain pass is full-res with up to ~54 entries per pixel worst case plus a 200-step cap per layer march. Config says "medium", on in "complete" preset only.

**Steal:** (1) a hashed 3-lattice drop field evaluated analytically, no particles, no state; (2) tile-binned drop lists to keep the per-pixel loop short; (3) rain-front noise advected by wind so density varies in sheets; (4) rain vs snow decided per world column by a snow-altitude derived from biome temperature; (5) drops refract sky colour (cheap "wet glass" look); (6) hide vanilla rain by zeroing particle life rather than removing it.

## 2. shelter

**Sees:** rain/snow stops under roofs, overhangs, trees, and also under *slanted* roofs (rain comes in at an angle, so the dry zone is offset downwind).

**Technique:** builds a 512x512 "caster" map (stored in atlas H) from the top-down heightmap: for each texel, the highest relevant occluder along the **slanted rain direction** (the direction is fixed by a constant wind drift plus a build slope ~1.96 blocks up per block across). It is computed with a logarithmic doubling scheme: an init pass accumulates 1, 2, 4, 8, 16 texel hops along the slanted line, taking the dominant occluder each time (4 nested levels in one pass); a "step" pass re-expands 3 more doublings per frame so the range keeps growing and the map converges over frames. A drop or surface point is then in the lee if its height is above that stored line (plus 0.15 block bias). A second pass `ground_sky` keeps a per-ground-column smoothed "sky height" and a "rain reach" scalar (0 = in lee, 1 = reached) that moves by a fixed step per tick toward the instantaneous answer: so ground wetness fades in/out slowly under cover. A small meta texel decides when to rebuild (on heightmap update, first frames, or when rain has just begun).

**Drivers:** global expression `shelter_age` (counts up only when precipitation is needed: rain with overworld sky, or puddle sim still active); heightmap update flag; config precipitation/rain_puddles (module is on if either).

**Cost:** moderate: 4 small (512^2) passes, run only while wet. 

**Steal:** do shelter as a 2D slanted-line occluder map, not per-drop ray casting; log-doubling to propagate; temporal smoothing of "reach" per column.

## 3. rain_puddles

**Sees:** after rain, flat ground goes darker and glossy, with irregular puddles that mirror the sky, ripple rings under falling rain, edge roughness, footprint trails in wet ground and drips from the player; things dry out in sun.

**Technique:** screen-space post pass on the scene with depth. It reconstructs world position from depth, uses the alpha tag to know sand/snow tops, and tests "up face". Puddle mask = threshold of a two-scale fbm (macro ~1.8 blocks, micro ~0.45 blocks, micro weight 0.3) evaluated on world XZ quantised to 1/16 block (so edges are pixel-art aligned). The threshold ("level") rises with accumulated wetness: area fraction = 0.30 + 0.70*wet^0.564, so puddles grow in area as the ground soaks, smoothly. The level is further reduced by the "open to sky" factor (from `ground_sky`, so covered/sheltered ground gets fewer puddles) and by lee share. On puddle pixels: darken albedo with a total-internal-reflection model for porous material (wet = dry*(1-f)/(1-dry*f), f=0.32); build a surface normal from rim roughness, trail "film" noise, wave slope (shared water waves with wave phase), optional wind ripple (disabled), and rain-ring slope; reflect the view ray; reflection colour = sky LUT in reflected direction, attenuated by sky visibility traced through the heightmap Hi-Z (so you see reflected roofs/trees dark, not sky), Fresnel weighted, plus lamp glints. Ring ripples are a hashed cell field (0.75-block cells, 7.5 s life, 4 slots per cell, rate derived from rain landing flux so density matches the visible drop density) giving packet-shaped height rings.
- **Persistent puddlemap sim** (512x512, tiled around the player): footprints and entity prints wet/dry the ground (dry gain 3, depth up to 0.88, refill speed 1/680 per tick), player drips deposit 0.45 within radius 4 blocks. A tile-list pass (`trail_tiles`) bins stamps so the shading pass reads few texels. Same sim framework is reused for snow/sand tracks (section 12).
- Wall wicking: vertical faces get a wet band that wicks up from the puddle plane.

**Drivers:** global expressions `puddle_wet` (+0.006/tick while rain at full, drying 0.00019 at night to 0.00056 in full sun: so ~170 ticks to soak, ~1800-5000 ticks to dry; sun factor from time of day), `puddle_active` (1600-tick timer so the sim keeps running after rain), `rain`, weather field (rain at the player's feet only where it actually rains: snow areas get no puddles), water phase, fog/sky attrs, held light, swimming (also activates it).

**Cost:** low-moderate (config "low"): one full-res shading pass, a few small sim passes. The Hi-Z sky visibility trace is the heavy part per puddle pixel.

**Steal:** puddle = threshold of fbm with wetness-driven threshold (animates growth for free); quantise the noise lookup to texel grid; sky-exposure mask so indoor floors stay dry; mirror reflection modulated by heightmap sky visibility; darken-then-gloss model; ring ripple count derived from actual rain flux.

## 4. lightning

**Sees:** forked bolts at the exact vanilla strike position, multi-stage flicker, cloud illumination, thunder-sky brightening.

**Technique (very clever):** the vanilla bolt mesh is still submitted, but its vertex shader recognises the bolt by its unique vertex colour, sends all normal geometry off-screen, and instead encodes the bolt's *world block position* into a few "marker" pixels of a tiny row of the weather target (a special lattice of vertices at 0.27-offset positions are decoded; the fragment shader writes an encoded coordinate into 2 pixels per slot, up to 16 slots). A tiny stateful pass (`lightning_state`, persistent) reads those pixels each frame, keeps a slot table (position, start tick, missing-frames counter, "live"), restarts a bolt if the position changes or if it vanished for 2 frames, and keeps it live up to 24 ticks. The draw pass (priority 87) then renders each live bolt as a **billboard-ish plane facing the camera direction toward the bolt base** by ray-plane intersection per pixel (4 sub-samples per pixel), looking up a pre-baked bitmap: a 2856x3960 animated atlas with 6 bolt shapes x 7 time layers (flash stages), 4 texels per block, ~408x660 texels per cell (102x165 blocks). Shape and mirror chosen by hash of position+start tick, stage by age (0-20 ticks, each layer has an energy factor: pre-flash 0.22, main 2.0, then 1.1, 0.9, 0.5, 0.4, 0.3). Blended additively over the scene with distance extinction in rain and glass transmission. If volumetric clouds are off, a cloud glow term (blue-white, bright rim near the bolt top) is added at the cloud deck; when volumetric clouds are on, the cloud march itself reads the lightning state to light the cloud interior (module `lightning_cloud_light`).

**Drivers:** vanilla lightning entity (via its mesh), rain level, cloud deck height, volumetric clouds flag.

**Cost:** cheap (one full-screen pass with a small loop; the state pass is 16x4).

**Steal:** reading gameplay events out of vanilla geometry through vertex-shader marker pixels; baked bolt atlas with energy ramp; additive cloud glow.

## 5. atmospheric_fog

**Sees:** haze that follows the real sky colour, thicker low near the ground (valley mist), per-biome visibility, rain/storm closing in, horizon blending into the sky.

**Technique:** terrain/entity/particle fog is switched off (fog start/end replaced by 1e9 in include) and replaced by one full-screen pass (priority 82, overworld only; other dimensions get a simpler `dimension_air` pass). The pass:
- Per pixel, marches the view ray through up to 6 segments in a **ground-hugging exponential layer** (density falls off with height above local ground, using the surface heightmap ground height for the column under each segment, not just camera Y), integrates optical depth analytically per segment, adds a flat precipitation term (rain rate or, where the weather field says snow, snow rate with an "unresolved" boost) in the slab from ground to a cloud deck height, then maps depth to opacity (clamped linear in the default model). Fog "end distance" drives the rate: rate = 1/(end - start).
- **Biome fog distance field:** a persistent 320x(3x320) texture recentred with the camera that stores, per 4-block cell, log2 of the biome's fog span, ground height, and fog end, blended by a 1D triangular blur (separable: `biome_field` then `biome_field_v`) so neighbouring biomes mix their visibility smoothly instead of popping at borders. Also holds blended sky and fog colour fields.
- Haze colour is not a flat fog colour: it is a **"painted sky"** colour looked up in the sky-model LUT in the view direction (so haze toward the sun is warm and toward the zenith blue), tinted by the biome sky/fog attribute palette and a slight cool aerial cast, with a rain lift.
- Cloud-edge handling: reads vanilla clouds depth, includes a marker pixel handshake so it only skips fog where clouds really are in front.
- A "cover" pass samples the field under the camera for indoors/caves to avoid haze leaking.

**Drivers:** `PolyAirState` (fog amount + daylight dim), rain, sky/fog colour attributes, biome fog_end/fog_start (set in biome_atmospheres), eye medium (air vs water), precipitation (adds aerosol), weather field snow line, view distance, lightmap sky.

**Cost:** moderate (config "medium"): 1 full-res pass with 6 segments, plus 2 small field passes and sky LUT bake.

**Steal:** height-dependent analytic fog using real ground height; fog colour from the sky model along the view ray; biome-blended visibility stored as log-span in a camera-centred field; rain shrinking fog end by a fixed formula.

## 6. biome_atmospheres

**Sees:** each climate has its own haze distance and tint: swamps/jungle/dark forest/taiga very short sight, snowy biomes grey and short, plains/desert/badlands long and clear; storms darken and shorten everything.

**Technique:** pure data (no shader): Polytone biome modifiers that override the vanilla fog/sky/cloud/sky-light environment attributes with **expressions evaluated every tick of rain**. Per biome group a single formula: fog end = view distance * lerp(clearVis, lerp(rainyVis, stormVis, thunder u), rain t); fog start = 0; fog/sky/cloud colours are lerps between three hex palettes (clear, rainy, storm) on t = min(2*rain,1) and u = max(2*(rain-0.5),0); cloud alpha 0.8 -> 0.95 -> 1.0; sky light factor 1.0 -> 0.55 -> 0.30. A dimension modifier removes vanilla's own rain/thunder overrides and zeroes sunrise glow. Also a smoothed `rain_outdoor` expression (lerp toward min(skyLight/5,1) at 0.2 per tick) so indoor players get less fog tightening.
Visibility fractions (clear/rainy/storm of view distance): plains/savanna/hills/ocean/beach/river/caves 1.0/0.75/0.5; desert same but fog colour sand (clear E F E4C8 ... storm brown 5A4A30); badlands 1/1/1; meadow/cherry 1/0.7/0.7; modded fallback 1/0.7/0.7; forests 0.5 flat; swamp and snowy 0.25 flat; jungle and dark forest/taiga 0.26/0.18/0.18.
Config default on, low cost, all presets.

**Steal:** a three-state (clear/rain/storm) palette and visibility table per biome class driven by one rain float; separate "outdoor" smoothing so interiors do not fog.

## 7. atmosphere

**Sees:** a physically based sky (blue zenith, orange horizon, sunsets with deep twilight, moon with blue tint), stars and celestial bodies hidden below the true horizon, no vanilla sunrise strip.

**Technique:** the sky mesh is replaced: the sky vertex shader maps the dome fan into a full-screen quad and computes per-fragment view rays from an inverse view-projection (two depth planes), the fragment shader just looks up a colour from the sky model for that direction. The model is single-scatter Rayleigh+Mie+ozone atmosphere math (planet radius 6360 km, top 6420 km, Rayleigh scale height 8.5 km, Mie 4 km with phase g 0.508, an ozone layer centred 25 km, 16 view steps, multiple-scatter fudge 0.0765, desaturation 0.594, sun intensity 55) **baked into a small LUT** (128x96 model LUT, 258x192 painted LUT, two planes around the sun azimuth) each frame, with eye-height bias so the horizon dips as you climb. A "painter" then blends the LUT with the biome sky and fog colours (palette structure weight 0.6) and applies exposure curves (sun and moon exposure ramps with 3.6 stops of twilight, horizon exposure). Sun disc dims in rain (0.6) / storm (0.3) / snow (0.2). Stars and sun/moon textures are multiplied by a horizon occluder (smoothstep around y=-0.009 with 0.035 width, with camera-height bias against a 64 block anchor over 512 blocks), so they set behind the true horizon not behind terrain.

**Drivers:** sun angle, day time, rain, `sky_attr`/`fog_attr`, camera altitude, eye medium (under water the vanilla path is used).

**Cost:** low (config "low"): LUT bake once per frame at small res, sky pixel is a texture fetch.

**Steal:** bake the atmosphere into a tiny azimuth-relative LUT and sample it everywhere (sky, haze, rain airlight, reflections, puddles); tint it with biome palette; horizon occluder for stars.

## 8. volumetric_clouds

**Sees:** vanilla voxel-style clouds but as lit 3D slabs: shaded undersides, lit tops, silver linings toward the sun, sun-free gaps, overcast in storms, cloud shadows on the ground, lit by lightning.

**Technique:** vanilla cloud geometry is thrown away in the cloud vertex shader. Post chain (priority 83, overworld): (1) cloud shadow map: 320x320 texels at 8 blocks (2560 block span, snapped at 512-block steps), each texel walks the sun-ward ray through the coverage grid with a DDA over 16-block cells (<=48 columns) integrating a closed-form vertical density profile, giving an optical depth -> shadow. Blurred, stored in atlas R, used by terrain shading, volumetric light, reflections. (2) A cloud-depth pass at half-res. (3) The march at **half resolution**: ray vs a slab 42 blocks thick (cloud height -6 to +36), 4-8 view steps depending on span (~2.5 block spacing), 2 light steps of 22 blocks toward the sun, jittered start by interleaved gradient noise. Density is **voxelised**: the coverage map (128x128 cells of 16 blocks, wind-scrolled) is sampled per cell so clouds stay blocky Minecraft-style; vertical profile = smoothstep in at the bottom, plateau, soft top that depends on coverage (denser cells are taller). Lighting: two-lobe Henyey-Greenstein phase (forward 0.55 mix with a back lobe -0.2 at 28%), multiple-scattering attenuation factor 0.65, sigma 0.45, rim fade. (4) Composite: bilateral/depth-aware upsample of half-res clouds into the scene, with translucent, particle and weather layers sorted behind/in front of the cloud depth so rain and glass interleave correctly.
- **Coverage logic:** coverage uses low/high thresholds 0.25/0.8 on large noise (period ~296 blocks, domain-warped by 237 blocks) that **rain widens toward overcast**; a morph speed 0.0025 evolves the pattern; a "sun clear" mask carves a hole in cover around the sun/moon disc (clear radius depends on elevation) so the sun is always visible except in storms (rain 0.5 factor).
- Clouds read the lightning state for internal flashes.

**Drivers:** cloud colour/alpha/height attributes (set by biome_atmospheres per weather), rain, game time (20 fps tick clock), sun angle, wind (fixed direction * 0.5 speed), air state.

**Cost:** heavy (config "high"): a half-res march with ~8 samples + 2 light samples, a shadow map DDA, composite pass. Fully bounded step counts keep it stable.

**Steal:** voxelised coverage map + 3D slab, 2-lobe phase, half-res + depth-aware upsample, ground cloud-shadow map by sun-ward DDA, sun-clearing hole, rain-driven coverage window.

## 9. volumetric_lighting

**Sees:** god rays through trees/windows in the air, light shafts underwater, shafts in storms, colour from the sun/fog.

**Technique:** half-res raymarch (priority 85) of in-scattered sunlight along each pixel ray: 40-80 steps (0.5 steps/block, denser near the camera: 1/block within 32 blocks), max 160 blocks (512 in storms with 28 sky steps), IGN jitter, voxel-snapped sample positions (16 per block) so shafts look pixel-art. Sun visibility per sample uses the **near sun-shadow map** (a sun-aligned ortho depth render around the player, snapped to a 1/16 grid so it does not swim), then the far shadow in atlas H (heightfield caster lines), plus cloud shadow map, plus stained-glass tint from a glass map. Separate underwater variant: a water-surface-aware march (window length through the surface, wave caustic modulated, TIR) with 24-40 steps. Then **temporal accumulation**: history target (persistent, half-res) reprojected with up to 16 frames age, depth-aware blend weights (min 0.125, gamma 1.25), plus a 33x1 persistent state texel for shadow-map movement compensation; composite upsamples with a depth sigma (0.05) and adds in/out-scatter colour.

**Drivers:** sun angle (end dimension uses its own), rain/storm distance, fog colour, held light (torch glow), eye medium, sea level, wave phase, config shadows.

**Cost:** heavy ("high").

**Steal:** shadow-map-driven froxel-free ray march with jitter + reprojection; snap shadow-camera origin to a sub-texel grid; voxel-snapped sample coordinates; separate cheap far-shadow heightfield.

## 10. underwater

**Sees:** water-colour extinction with depth (red dies first), Snell's window overhead, mirror-like total internal reflection outside the window, light dimming with depth, drifting marine snow, caustic light on the seabed, surface seen from below with ripples.

**Technique:** two chains. `underwater_surface` (priority 75) re-shades the *surface seen from below* in the translucent layer. `underwater` (priority 84) fully re-composes the scene when the camera fog type is water: per-channel Beer-Lambert extinction (hue-driven clarity: murky vs clear water chooses bulk and layer extinction coefficients; base colour mixes), downwelling light attenuated by depth below the surface Y (from sea level and the actual surface), caustics decoded from the caustic target, light shafts if volumetric lighting is on, and **marine snow**: two layers of hashed 3D cells (periods ~212x120x159 etc.), 36 and 20 steps, particle size 1/16 block, sway and tumble, glow depending on light, near fade 0.25-1.0 and far fade 24-42 blocks. Mirror/TIR: view rays that hit the surface outside the ~48 degree cone are mirrored using screen-space and heightmap lookups with a tilt constant, the window region refracts to the sky LUT / sunlit hemisphere.

**Drivers:** `fogType` water, sea level, wave phase, biome water tint (hue), rain (surface waves), volumetric lighting flag, sky/fog attributes.

**Cost:** moderate-heavy only while submerged.

**Steal:** Snell's window + TIR with a tilt tweak, hue-derived clarity, marine snow as hashed lattice.

## 11. procedural_water (+ translucent_lighting_water)

**Sees:** animated, wind-responsive waves with believable normals, refraction, foam, caustics, wakes behind swimmers/boats, glass and ice that stays clean next to water.

**Technique:**
- Terrain water is rendered near-white with alpha 254/255 so the water becomes a flagged region of the translucent buffer; post pass `water_flag` (priority 8) detects it, and `water` (60) first blends it with fog/vertex tint at fixed alpha (0.70) for sheets, then `water_shade`.
- Waves: analytic sum of 5 octaves of directional gravity waves (wavenumber doubling each octave, dispersion omega = sqrt(g k), each octave's direction rotated ~20 degrees from the previous), steepening with wind (wind01 in [1,3], WIND_STEEPEN 1.4), anti-aliased by shrinking amplitude when wavelength approaches pixel footprint; a far set of 3 octaves; phase clock from `water_phase`. Normals come from the analytic derivative.
- Body (`water_body`, priority 72): per pixel refracts the view ray (IOR 1.333), marches the refracted ray against the scene/heightmap bed (8 steps + 3 binary refinement steps, max 32 blocks), applies depth-wise Beer extinction, caustics, foam (whitening with sun/shadow history), Fresnel mix with reflection, fog.
- **Caustics** (`water_caustics`, currently gated off via activation 0.0 in this release but the include exists): for each seabed pixel finds the wave-surface point that focuses toward it by Newton iterations on the wave lens (3 iterations), computes focus intensity from the Jacobian, quantises with a 4x4 Bayer dither, adds chromatic dispersion per channel (~0.007), so you see sharp bright nets at pixel-art resolution.
- **Wake simulation:** a persistent 512x512 wave-equation grid (height + velocity packed 16-bit), 6 sim passes per frame (sub-stepped), damping 0.9966, sponge boundary 96 texels, driven by entity stamps: an "entity occupancy" probe pass renders entity silhouettes into a target, `entity_stamp` converts them to impulses (radius 0.55 block, impulse proportional to speed, lead position ahead of motion, ring shaped), `wake_optics` turns height into normals/foam into atlas R which the water shader adds to the wave slope and caustics.
- **Neighbour-aware block models** (translucent_lighting_water): glass, stained glass and ice blockstates are composite models whose six faces switch between "plain", "split" (neighbour is water source with level 0) and "full" (neighbour water two high) variants by testing the adjacent block; the variants use separate textures/tags so water-glass interfaces do not z-fight or tint twice, and the translucent tag (low depth bits) tells the post chain which faces are glass vs ice vs water.
- Water particles replaced: fishing bobber ghost particles with a custom lifetime/decay; vanilla splash particles hidden.

**Drivers:** wind01 (water_wind: 1.0 +/-9%/6%, plus rain), water_phase, sea level, fog attr, shadows, volumetric lighting, rain, entity positions.

**Cost:** moderate overall (config "medium") but wake sim and caustics are the heavy parts when active.

**Steal:** wave field as an analytic spectrum with AA; wake grid driven by entity occupancy stamping into a wave PDE; face-variant blockstates keyed on neighbour water; tag bits in depth.

## 12. reflections

**Sees:** water, ice and glass mirror sky, clouds, terrain, lamps, and sun glare.

**Technique:** pass `reflect` over the translucent layer (set up by a lamp-source pre-pass). The reflected ray is **traced through the top-down heightmap** rather than screen space: 128 steps with a texel-sized step that grows 10%, 8 refinement steps, max 160 blocks; a Hi-Z variant (min pyramid, 1024 iterations, top level 3, up to 640 blocks) is used for sky-visibility queries. Misses fall back to sky LUT (and cloud coverage sampling + cloud shadow). Wall reflections gated by normal-y 0.35. Fresnel F0: water 0.02, ice 0.25, glass 0.25, metal 0.30. Sun gets a sharp core (power 125, strength 10) plus halo (0.5/0.4) and warm glare colour. Local lamps: `reflect_src` builds a bit-mask from the atlas X light grid and `reflect_lamps` builds a 768x928 persistent lamp grid so lit block-light sources reflect in water as glints (non-trivially cheap point-light reflection). Wave normal amplitude at 0.35 for reflections. Ultra reflections is a disabled overlay (condition false).

**Cost:** heavy ("high"), on in "balanced" and "complete" presets.

**Steal:** heightmap ray tracing for reflections (stable and off-screen-capable, no SSR disocclusion); Hi-Z min pyramid; lamp glint grid.

## 13. atlas_heights

Utility module: three small chains that fill atlas H: copy views (heightmap depth+colour+near+far shadow into their atlas regions, priority 14), `hm_pyramid` (builds a 512^2 persistent first min-level and packs lower levels into the atlas, 13), and a glass-height copy when shadows are off (49). A debug view chain exists (disabled). Needed by reflections, puddles, VL, precipitation, water+fog. No config of its own. Steal: build the Hi-Z min pyramid of the heightmap once and share.

## 14. procedural_snow

**Sees:** snow tops become a lumpy, wind-drifted surface with grain and pixel-quantised shading, rim light and cavities, footprints that compress and refill, sparkle-free matte white; layered snow blocks have flat tops (heights 2..14/16 models).

**Technique:** terrain fragment writes flat snow albedo (0.97 white scaled so texture detail is preserved) and flags snow top faces by alpha 254/255. Post `snow_shade` (priority 45) uses depth to recover world position, then evaluates a 128-block tiling "surface" function at 16 texels/block: wind-aligned anisotropic drifts (noise frequencies 8x14 and 19x32 over the tile = stretched along wind), lumps (40, 85 per tile), fine grain (427), mounds (32), converted to height and slope; shading uses slope, sun and moon direction, shadow map (sun/shadow strength 0.45/0.5), cavity darkening, LOD beyond 24-56 blocks to cheap. A **footprint sim** (`snow_sim`, persistent 512x513 map, 6-block stamp radius, refill 1/2400 per tick = two minutes, entity/leg depth 1.0, jittery width and crumb noise) fed by `ground_probe` (a ground-contact pass that reads item/entity depth under feet and a `ground_landing` history to detect new steps) writes depressions; `trail_tiles` bins them; shade adds rim highlights around prints by sampling 8 directions (rim height 0.06, radii 3-6 texels, random rim variation 0.55-1.45). Pixelised (quantised to 6 levels) output.

**Drivers:** snow block tags/marks; wind (fixed prevailing direction, via wind include); sun angle; shadow config; rain.

**Cost:** low ("low"), complete preset only.

**Steal:** tag top faces with alpha, redo shading in post with a tiling procedural height field; refill-over-time footprint map keyed to ground contact; wind-stretched noise for drifts.

## 15. procedural_sand (+ sand_ctm, snow_sand)

**Sees:** dunes with sharp wind-aligned ridges, fine ripples, footprints in sand, red sand variant, smooth blends at sand edges, shared lighting with snow.

**Technique:** analogous to snow: sand fragment forced to flat albedo (sand 0.86/0.81/0.64; red 0.65/0.46/0.27; discrimination by green/red ratio < 0.83) and flagged 253/255 on top faces; post `sand_shade`. Surface: three bands of "sastrugi"-like relief on a 128-block tile: swells (9x21 elongated along wind), dunes (47 per tile) with an **asymmetric sawtooth profile** (crest at 0.55, long windward slope, short lee slope, warp noise 0.6/0.19), and ripples (67x31 anisotropic, slope 0.035). Slope bounded 0.025-0.165 and modulated by a coarse "wave density". Warm-tone slope shift (sunlit vs shaded faces shift hue). Footprint sim identical to snow but refill 1/5280 or 1/2400 and a "path" mode (radius 5.5, depth 0.6) and entity-specific bowls (bowl radius 4, depth 0.8). **sand_ctm** (requires overlay_textures): 17 OptiFine-CTM tiles per sand type, textured with six marker colours that the sand shader recognises (219,207,164 etc.), so connected-edge tiles blend sand into neighbours while still being procedurally shaded. **snow_sand** = shared `ground_shade` pass when both are on, merged so only one pass handles both flags.

**Cost:** low.

**Steal:** asymmetric dune profile aligned to prevailing wind; marker-colour textures to carry CTM edge info through to a post shader.

## 16. procedural_ice (disabled in this release)

Overlay condition is `false`, so inactive in 0.2.1. Technique visible in files: ice blocks flagged by geometry marker offsets (128/2048 texel code); fragment shader treats each face as a thick slab and procedurally renders internal structure from face-space coordinates: cell-based branching **cracks** (cell 2 blocks, length 6, width 0.5, bevel 0.2, tone 0.2), scratches, bump and undulation layers, grain, grazing-angle fade (23.5 down to 2.0), relief shading from the sun path; with neighbour-aware "submerged" models. Reflection F0 0.25, frost opacity 0.28 in reflections. Steal: cracks as hashed cell polylines with bevel normal.

## 17. frosted_leaves (+ frosted_leaves_bushy)

**Sees:** leaves with snow on top get a white frost cover with ragged edges, tufts under overhangs, tone varying with the leaf texture.

**Technique:** it is decided by **blockstate expression on the block above**: if the block above is any snow or powder snow, the leaf block switches to a `*_frost` model (snow-on-top face), else normal. No biome-temperature test at all; snow cover is the physical trigger (and snow only exists where weather/snow line allowed it). Bushy variant adds tuft submodels chosen by checking air above (tufts-up) or air/snow below (tuft-down), so the extra fluff only renders on exposed sides. Frost faces and tufts carry the fractional geometry marker; the fragment shader derives per-pixel frost from the leaf texture luminance (low-luma gaps stay green, thresholds 0.32-0.74), quantised on a 16/block grid with a 4-octave gradient noise (ragged 2.0, fade 6.0, solid 2.0), five-tone ramp from 0.80 to 0.97 albedo with cold tint (snow_cool), lightmap and directional face shading. Down faces excluded.

**Cost:** low.

**Steal:** swap block model by neighbour state instead of shading; use texture luminance as the frost mask; marker offsets.

## 18. Config options (assets/recrafted/polytone/config_entries; presets classic / balanced / complete)

| id | default | cost tag | presets (classic/balanced/complete) |
|---|---|---|---|
| precipitation | true | medium | off/off/on |
| rain_puddles | true | low | off/off/on |
| lightning | true | low | on/on/on |
| atmospheric_fog | true | medium | off/off/on |
| biome_atmospheres | true | low | on/on/on |
| atmosphere | true | low | off/off/on |
| volumetric_clouds | true | high | off/off/on |
| volumetric_lighting | true | high | off/off/on |
| procedural_water | true | medium | off/off/on |
| reflections | true | high | off/on/on |
| ultra_reflections | false | high | off/off/off (overlay condition literally false) |
| procedural_snow | true | low | off/off/on |
| procedural_sand | true | low | off/off/on |
| frosted_leaves | true | low | (no preset map) |
| bushy_leaves | true | medium | (no preset map) |
| overlay_textures | true | low | needed by sand_ctm |

Modules without own config: shelter (on if precipitation OR rain_puddles), atlas_heights (on if reflections OR rain_puddles OR volumetric_lighting OR precipitation OR (procedural_water AND atmospheric_fog)), translucent_lighting_water (on if (colored_lighting OR shadows) AND procedural_water), procedural_snow_sand (both snow and sand), procedural_ice (disabled), underwater (procedural_water AND atmospheric_fog), sand_ctm (procedural_sand AND overlay_textures), frosted_leaves_bushy (frosted_leaves AND bushy_leaves), biome_atmospheres folder loads if biome_atmospheres OR atmospheric_fog.

## 19. Biggest ideas to carry to our engine

1. Top-down heightmap + Hi-Z min pyramid as the universal "sky openness / reflection / shelter" oracle.
2. Per-column weather field with snow altitude from biome temperature.
3. Slanted-line shelter map built by log-doubling.
4. Wetness as a global scalar that integrates up in rain and down in sun, feeding a fbm-threshold puddle mask.
5. Analytic lattice rain/snow with tile-binned lists, wind-front noise, sky-refraction look.
6. Hijacking gameplay data (lightning positions) via geometry marker pixels.
7. Atmosphere baked to a tiny LUT reused by sky, fog, rain, reflections.
8. Voxelised coverage cloud slab with 2-lobe phase, half-res + depth-aware upsample, shadow map via DDA.
9. Wave PDE wake driven by entity occupancy stamps; analytic gravity-wave spectrum.
10. Terrain writes flags in alpha; post replaces snow/sand with 128-block tiling procedural relief and a persistent footprint map that refills.
