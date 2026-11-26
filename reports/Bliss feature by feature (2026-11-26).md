# Bliss feature by feature, against ours (2026-11-26)

Ronald, 11-26 (via Planner): "check source code and see what features are missing still, build out and then compare, as
then we know everything [is] there and close/matching". Two readers went through every Bliss program and include
(`chiacraft-archive/references/shaders/bliss/Bliss-Shader-git`, Stable 81e403e). Bliss is "© Chocapic13, all rights
reserved", so this file holds method and file:line only. Ours is at the tip of `claude/look-speed` (177eccd33). "On in
Ronald's settings" is checked against his Bliss settings (`images/bliss-stage2/2026-11-25/bliss_cap2c/pack_settings.txt`).
The 11-25 report is the earlier map; much of it has since been rebuilt (the physical sky, Bliss's clouds and the one fog
are now on by default).

**Counts:**
- Surfaces and light (part A, 78 rows): 29 matched, 17 partial, 11 missing, 8 differ, 3 differ on purpose, 9 ours only.
- Sky, air and frame (part B): 33 matched, 11 partial, 12 missing, 7 differ.

**How to check a row:**
- A pose of the 10-pose matrix (`lk_matrixD13.txt`), with `lk_match.py`: picture dE and SSIM.
- Since 11-26 also "F", the land's shade fill hue dE against Bliss's cloudy average, in debug-13 runs.
- Or the debug view a row names.

## Build order (missing and partial, by weight per cost)

Weather regressions on dev come first and are not repeated here: dust (done, f24452a2), the sky fill / snow glow
(waiting on capture 3), puddles, and the y300 band.

| # | Item | Part / row | Cost | GPU | Weight (poses) |
|---|---|---|---|---|---|
| 1 | The air dims what lies behind (atmosphereAlpha): the picture, and the sky+clouds map before its fog | B1 | S | none | high (vistas), med (F) |
| 2 | Rain on open ground as in his profile: smooth and shiny (sky Fresnel, sharp sun, SSR), not darkened | A4 | S-M | med | high (rain ground, rain vista) |
| 3 | Small fixes to the frame: luma weights 0.21/0.72/0.07 where deferred.vsh takes them; our cirrus off under Bliss's clouds; no vignette; check the shafts/volume posts against the one fog; border fog toward the clear average; Bliss's sun disc; CAS after the curve; output dither | B5, B8, B18, B19, B9, B7, B16, B20 | S each | none | med (F, vistas, dawn) |
| 4 | Cave fog and the cave kill on the fog's density | B4 | S-M (engine ~30 lines) | none | high (cave) |
| 5 | PCSS: a blocker search, then a 13-tap rotated filter | A2 | M | low-med | high (forest, noon, dawn) |
| 6 | Depth before shading (prepass or a stored sun share), then SSAO on the indirect light with its cap on sky facing, and screen-space contact shadows toward the sun (also on far land) | A1, A3 + prerequisite | M-L (engine) | low-med | high (vistas, forest, cave) |
| 7 | Near shadow map to 128 blocks | A5 | S | med | med |
| 8 | Exposure: centre 70 %, Bliss's ease; bloom levels and weights | B6, B17 | S + M | low | med (dawn, night, moving) |
| 9 | DOF (on in his settings) | B2 | M-L | med | low-med (forest, cave, shore) |
| 10 | Far-land lightmaps (LOD sky and block light) | A6 | L (engine) | none | med (vistas at night and dawn) |
| 11 | Clouds at full size for compares (his VL 1.0) | B3 | S | high | med (cloud edges) |
| 12 | The rest, low weight on the poses: water details (A9-A13), particles and entities lit as Bliss (A14-A17), glass (A8), coloured translucent shadows (A7, B15), lightning in fog (B14), moon phase side (B22), sky texture history (B11-B12), bloomy fog details (B13), underwater fog (B10) | | S-M each | low | low |
| — | Not on the matrix, later: POM, LabPBR emission, rough metals (A18-A21, no LabPBR pack in the poses), gameplay camera effects (B23), Nether and End fogs (B24) | | | | none |

After the build: one full compare over the 10 poses with dE, SSIM and F, each picture read by eye.


---

## Part A: Bliss surfaces and their light, against ours (2026-11-26)

Bliss: `chiacraft-archive/references/shaders/bliss/Bliss-Shader-git/shaders` (method only, "© Chocapic13, all rights
reserved"). Paths: `dim/` = `dimensions/`, `lib/` = `lib/`, `w0/` = `world0/`. Ours: `mods/base/shaderpack/` (world.wgsl =
W, look.wgsl = L), `mods/base/assets/shaders/post*/`, `crates/engine/src/`. Read at the worktree tip (177eccd33).

**Ronald's profile and the resource pack.** Settings that touch surfaces: POM on (depth 0.05), MATERIAL_AO on, Porosity
on, Specular_Reflections + Screen_Space_Reflections + Sky_reflection + Rough_reflections on, Roughness_Threshold 1.3,
Sun_specular_Strength 10, SSS_TYPE 3, EMISSIVE_TYPE 3, LabSSS_Curve 1.3, Emissive_Curve 0.5, TRANSLUCENT_COLORED_SHADOWS
on. Our own notes (look.rs:45-48) say his captures used no LabPBR pack. Then types 3 mean "from the pack only", so they
do nothing: no SSS, no LabPBR emission, POM / normal maps / material AO / porosity have no data. Porosity on with no
data sets porosity to 0, so **rain does not darken anything in his Bliss** (all_solid.fsh:547-553). What stays active:
sun GGX x10 on every pixel, rain-driven smoothness (sky reflection + SSR on wet open ground), coloured glass shadows,
PCSS, SSAO (indirect_effect 1), and the screen-space contact shadows. The contact shadows sit inside `SSS_TYPE != 0`, so
type 3 keeps them on.

Our default path is `land_from_sky()` (L:630: `look_sky_phys` 1 and `look_chain` 1, both default 1 in look.rs). Most of the
11-25/26 rebuild lives in that branch: `light_on_bliss`, `bliss_block_light`, `bliss_held_light`, `bliss_min_light`,
`bliss_sun_sheen`, `water_body_bliss`, `wave_bliss_normal`, `light_weather`, `sky_sun_mask`. The prior audit (11-25)
predates all of them.

Debug views that exist (`base:look_debug`): 1 sun light alone, 2 sky light alone (W:784), 6 rods/exposure, 7 shadow
coverage + cloud shadow (W:799), 8 weather / puddles (W:796), 9-10 cloud shadow map, 11-12 clouds, 13 frame-look
swatches. `look_ao_test 1` removes the vertex AO.

### 1. Feature table

| Area | Feature | Bliss (file:line, method) | On in Ronald's? | Ours (file:line) | State | How to check |
|---|---|---|---|---|---|---|
| Land light | Sun and sky colours that light the land | dim/composite1.fsh:~902-903: sun from the baked light colour /80, ambient from the cloudy sky average /30 | yes | L:606-625 `land_sun`, `land_ambient` (deferred.vsh method) | matched | debug 13; dawn, noon vista dE |
| Sky light | Lightmap curve | lib/diffuse_lighting.glsl:65-76: (L^15·2 + L^2.5)·0.5·0.7, L offset by 1.05 | yes | L:2474-2501 `light_on_bliss` | matched | debug 2; forest, cave |
| Sky light | Facing weight (tops 2.5, sides 0.93, bottoms 0.2; floor 0.2+0.8(1-L)) | dim/composite1.fsh:1114-1117 | yes | L:2479-2481 | matched | debug 2; noon ground |
| Sky light | Grass/plants: sky facing forced to 1.5; sun NdotL from the quad's own normal | dim/composite1.fsh:1115; dim/all_solid.vsh:224-229 | yes | W:383 plants take an up normal, so the sun is also judged facing up | differs (unsure: vanilla cross-plant normals) | forest, noon ground (grass sides) |
| Sky light | SSAO caps the sky facing boost: occluded tops lose the 2.5 | dim/composite1.fsh:1119-1121 (indirect_effect 1) | yes | none (no AO reaches the lit pass) | missing | forest floor, cave; needs a new debug view (AO alone) |
| Sky light | Minimum light x (1 + 0.7·n·view), night vision x0.1 | dim/composite1.fsh:924; lib/diffuse_lighting.glsl:73 | yes | L:2486, L:2594 `bliss_min_light` | matched | cave, night |
| Sky light | Light-leak fix: shadow position biased by distance and snapped into the block where sky < 0.1 | lib/Shadows.glsl:4-26; dim/composite1.fsh:1011 | yes | L:1714-1745 normal offset x(2 - n.l) + slope bias | differs | cave mouth, forest (peter-panning) |
| Sun shadow | Cloud shadow on the sun | dim/composite1.fsh:1270 | yes | W:425-433, W:530-546 `cloud_shadow` | matched (the cloud map itself is the other audit's) | noon vista; debug 7, 10 |
| Sun shadow | Map extent and resolution | lib/Shadow_Params.glsl:4-26: one 2048 map over 128 blocks, log-distorted (finest at the eye) | yes | crates/engine/src/client/mod.rs:4657 near map ±80 at 2048 (0.08 block/texel), then the far map ±2000 (config.rs:643-646, ~2 blocks/texel) from 80 to 128 | partial | noon vista, forest (80-128 band blobby); debug 7 |
| Sun shadow | PCSS: 4-tap blocker search sets a 5-30 radius; 13-tap rotated filter + TAA | dim/composite.fsh:481-540; dim/composite1.fsh:641-700 | yes | L:1714-1786 a fixed 3x3 of max-pairs (gap closing), no penumbra growth | missing | forest, noon vista, dawn (long soft shadows); needs a new debug view (penumbra radius) |
| Sun shadow | Coloured translucent shadows (glass tints the sun; water casts none) | dim/composite1.fsh:661-688; w0/shadow.fsh:23-27; w0/shadow.vsh:274 | yes (TRANSLUCENT_COLORED_SHADOWS) | raster translucents cast nothing (unsure); RT keeps the colour "for later" (W:495-498) | missing | none of the 10 poses; needs a stained-glass pose |
| Sun shadow | Screen-space contact shadow: 16 steps toward the sun from depth, min() with the map, **also past the shadow distance and on DH land** | dim/composite1.fsh:376-437, 1249-1257 | yes (inside SSS_TYPE != 0) | none: post_low/contact.wgsl is an AO, not a ray toward the sun | missing | noon vista, dawn vista (far self-shadow), forest; needs a new debug view |
| Sun shadow | Fallback past the map: clamp((L-0.8)·25) | dim/composite1.fsh:1000, 1048 | yes | W:425-433 `sky_sun_mask`, W:482-490 fade over the reach's last fifth | matched | vista; debug 7 |
| Sun shadow | NdotL remap (255x-15)/240, no AO on the sun | dim/composite1.fsh:1005 | yes | L:2492 | matched | debug 1 |
| Sun shadow | Entity and player shadows | settings.glsl:139-140 | yes | model.wgsl:193 `fs_shadow` | matched | (mobs off in poses) |
| Sun shadow | Waving plants moved in the shadow pass (within 24 blocks) | w0/shadow.vsh:231-250 | yes | shadow pass shares the waving vertex shader (unsure) | matched (unsure) | forest |
| Sun shadow | Edge-on plant quads cast nothing | none | n/a | W:554-559 | ours only | noon ground |
| AO | SSAO: 7 taps, denoised, half-res upsampled, indirect only, min((1-ao²)^5 vanilla, SSAO^6) | dim/composite.fsh:239-304, 446; dim/composite1.fsh:538, 1180-1188 | yes | post_low/contact.wgsl:42-88 (1.6-64 blocks, darkens the whole colour, gentle, ≤50%) + post_low/far_ao.wgsl:72 (24+ blocks, sky light by a shadow-map guess) + vertex AO | partial | forest, cave, noon ground; needs a new debug view (AO alone) |
| AO | Vanilla AO curve | dim/composite1.fsh:1184: (1-occ²)^5 | yes | W:4 table 0.45/0.64/0.82/1 on indirect + lamps (L:2498) | differs (unsure if equivalent) | cave, forest; debug 2 + look_ao_test |
| AO | GTAO | lib/PhotonGTAO.glsl:145 (indirect_effect 2) | off | none | matched (off) | n/a |
| AO/GI | SSGI / RTAO | lib/indirect_lighting_effects.glsl:358 (indirect_effect 3/4) | off | pathtrace.wgsl (path tracer) | ours only | n/a |
| AO | Material AO (normal map's blue channel) | dim/all_solid.fsh:477-484 | on, no data | W:723 `mat.ao` | matched | n/a without maps |
| Block light | Curve (1-√(1-L))²·2, torch colour, auto brightness by exposure | lib/diffuse_lighting.glsl:24-63 | yes | L:2503-2525 `bliss_block_light` | matched | night, cave |
| Block light | Coloured light (LPV floodfill) | dim/setup.csh, dim/shadowcomp.csh, lib/lpv_*.glsl, lib/voxel_*.glsl (LPV_ENABLED) | off | the engine's RGB light levels keep their colour (L:2513-2514) | ours only | night, cave |
| Block light | Held light: max(lm, 0.9(1-d/10)^1.5) | dim/all_solid.fsh:318-330 | yes | L:2526-2534 `bliss_held_light` | matched | night |
| Emissive | LabPBR emission x auto brightness | dim/composite1.fsh:439-447, 1298 | type 3, no data: none | glow textures skipped on this path (W:736-741), matching his | matched (on purpose) | night, cave |
| Emissive | LabPBR emission channel itself | dim/all_solid.fsh:514-516 | type 3 | gpu_layout.rs:1492 `mat_at` ignores the specular alpha | missing (only matters with a LabPBR pack) | n/a |
| Emissive | Vanilla emissives (spider eyes, glint, beacon) x auto brightness | dim/all_vanilla_emissives.fsh | yes | model.wgsl:166-169, entity.wgsl:202-205 glow masks with no exposure boost | partial (unsure) | none of the poses |
| Lightning | Land lit by the strike | dim/composite1.fsh:1003 computes it but never adds it (dead code; lib/lightning_stuff.glsl:8-29) | n/a | L:1375 `lightning_glow_at`, a 180-block glow | ours only | rain vista during a strike |
| SSS | Sun SSS by blocker depth with a forward phase, max() with n.l | dim/composite1.fsh:711-737, 1261, 1288 | type 3, no data: none | W:592 `leaf_sss`, off (look_leaf_light 0) | matched (both off) | forest |
| SSS | Sky (ambient) SSS | dim/composite1.fsh:739-751, 1216-1225 | none (LabSSS 0) | none | matched (both off) | forest |
| SSS | Mob / misc-block SSS | settings.glsl:291-292 | off | none | matched (off) | n/a |
| SSS | LabPBR SSS (blue > 64) | dim/composite1.fsh:823 | no data | W:912 `mat_through` (a thickness guess) | differs | n/a |
| PBR | LabPBR read (smoothness, F0, metals 230-237, porosity) | dim/all_solid.fsh:493-553 | on, no data | gpu_layout.rs:1492-1525 `mat_at` | matched | n/a |
| PBR | Normal maps | dim/all_solid.fsh:474-486 | on, no data | gpu_layout.rs:1528 `mat_normal` (frame from screen derivatives) | matched | n/a |
| PBR | POM (adaptive step, depth write) | dim/all_solid.fsh:349-398 | on, no data | height parsed (gpu_layout.rs:1519), not used | missing | n/a without maps |
| Specular | Sun GGX x Sun_specular_Strength on every lit pixel (vanilla: rough 1, F0 0.02) | lib/specular.glsl:160-178, 291-296 | yes (x10) | W:606-616 `bliss_sun_sheen` on terrain and LOD; W:865 `mat_shine` with maps; none on entities or particles | partial | dawn (low sun on faces), noon vista dE; debug 1 |
| Specular | Sky reflection (Fresnel, lightmap fade (L-0.8)·7, roughness threshold) | lib/specular.glsl:208-255 | yes | W:865-903 `mat_shine`, maps only: sky blurred toward the zenith, open = smoothstep(0.8,1,L) | partial | rain ground (see wetness) |
| Specular | Rough reflections: one VNDF GGX sample a frame + TAA, SSR mip blur by length | lib/specular.glsl:128-160, 214-232, 266-272 | yes | W:886-893 blur toward the zenith; RT cone | partial | rain ground |
| Specular | Metals: reflection tinted by normalized albedo, diffuse kept | lib/specular.glsl:237-240 | no data | W:744, W:865-869: F0 table or albedo, diffuse x(1-0.5·metal) | differs | n/a |
| Specular | SSR on opaque surfaces, quality scaled by contribution | lib/specular.glsl:257-289 | yes | post/reflect.wgsl:116 through the negative alpha (W:926, maps with smoothness > 0.55 only) | partial | rain ground |
| Wetness | Rain makes open ground smooth: smoothness max(s, rain·clamp((sky-0.9)·10)), F0 0.02, so sky reflection, SSR and a sharp sun on all open surfaces, following rainStrength | dim/all_solid.fsh:333-338, 496-497 | yes | L:45-75 `rain_on_ground`: darkens x0.58 + sheen (0.04 + 0.3(1-n·v)^4), lingers by `ground_wet`; no SSR | partial | rain ground, rain vista; debug 8 |
| Wetness | Puddles and porosity darkening | dim/all_solid.fsh:340-344, 545-553 | porosity 0 with no data: **no darkening**; puddles only F0 0.02 (unseen) | L:76-100 puddle cells as a mirror film, ground darkened | differs (ours darker, with puddles) | rain ground dE (luma) |
| Wetness | Rain rings on water | none found (unsure) | n/a | L:109, W:1232-1236 | ours only | rain + water |
| Wetness | Snow on tops | settings.glsl:50 Snowy_Winter | off | L:3460 `snow_on` | ours only | n/a |
| Water | Waves: 3 noise layers turned by the golden angle (48x12, 12x48, 32x32) x a large modulation | lib/waterBump.glsl:31-90 | yes | W:979-1003 `wave_bliss_height`, `wave_bliss_normal` | matched | water/shore |
| Water | Waves on sides and falls flow down the slope; parallax displacement | dim/all_translucent.fsh:555-557, 169-179 | yes | W:1230: side faces keep a flat normal; no parallax | missing | water/shore (low) |
| Water | Body colour: surface albedo 0, the colour from a 2-step march (≤12 blocks, exponential steps, the shadow map per step) | dim/fogBehindTranslucent_pass.fsh:193-275; dim/all_translucent.fsh:503 | yes | W:1105-1133 `water_body_bliss`: the same march with one constant shadow | partial | water/shore dE |
| Water | Floor absorption mix(e^(-2εd), e^(-8ε), 1-e^(-0.1d)) | dim/composite1.fsh:1059-1078 | yes | W:1165-1172 | matched | water/shore |
| Water | Caustics: the wave heightmap projected along the sun, on the direct light under water | lib/waterBump.glsl:1-29; dim/composite1.fsh:1081 | yes | W:1087 `caustics` from its own noise ridges, on the floor seen through the surface only | differs | water/shore |
| Water | Reflection: Fresnel 0.02 ^5, the sky texture, faded indoors by the lightmap | dim/all_translucent.fsh:740-790 | yes | W:1811 `water_fresnel` (look_water_mirror 1), W:1287-1291 | matched | water/shore |
| Water | Sun GGX at roughness 0.035 | dim/all_translucent.fsh:781-783 | yes | W:1327-1336 | matched | water/shore, dawn |
| Water | SSR on water | dim/all_translucent.fsh:795-805 (in the pass, last frame's colour) | yes | post/reflect.wgsl:116 (post pass) | matched (other method) | water/shore |
| Water | Refraction | dim/composite3.fsh:212-219, 323: a UV offset by the tangent normal, scaled by distance | yes | W:1150-1160 a Snell ray to the depth behind, reprojected | differs on purpose (ours more real) | water/shore |
| Water | Snell's window from below | dim/all_translucent.fsh:772-775 | yes | W:1216-1220 | partial | needs an underwater pose |
| Water | Surfaces seen with the eye under water: depth guess 40(1-L)², min light x10, absorption, caustics on the sun | dim/composite1.fsh:1061-1081 | yes | L:3506 `under_water`: fog + noise beams, no caustics on surfaces | partial | needs an underwater pose |
| Water | Far (DH) water shaded as near water | dim/DH_translucent.fsh | yes | W:1830 `far_water_lit`: its own WATER_DEEP and its own GGX, not the Bliss path | partial | dawn vista, water/shore far |
| Water | Foam | none | n/a | off (look_water_foam 0) | matched | water/shore |
| Glass | Glass as reflective (F0 0.02, smooth): sky + SSR + sun GGX, alpha raised by Fresnel | dim/all_translucent.fsh:719-830; dim/all_translucent.vsh:163 | yes | W:1672 `glass_lit`: only LabPBR showcase glass mirrors; vanilla glass gets the rough sheen alone | missing | none of the poses (windows) |
| Glass | Tint: what lies behind multiplied by normalized albedo·0.9+0.1 | dim/composite3.fsh:361-363 | yes (Glass_Tint) | plain alpha blend | missing | none of the poses |
| Translucent | Lighting: direct x shadow x cloud shadow³, indirect with facing | dim/all_translucent.fsh:605-640 | yes | W:1672 via `shade_soft_lit` → `light_on_bliss` | matched | water/shore |
| Particles | Light: shadow map + lightmap fallback + cloud shadow, x(2·HG(0.65)+0.5), ambient x2.5, Bliss block curve | dim/all_particles.fsh:405-450 | yes | particle.wgsl:63-75 `particle_light`: old curve /3, half sun gated by sky, no shadow, no phase, the engine's block light | partial | forest (falling leaves), night (torch smoke) |
| Particles | Weather drawn | dim/all_particles.fsh (WEATHER: a mask only, no colour) | yes | L:2576 `light_weather` (Bliss's particle light) + drawn streaks, look_weather 1 | differs on purpose | rain vista |
| Particles | Half-transparent particles fade out over 100 blocks | dim/all_particles.fsh:466 | yes | none | missing | rain vista (low) |
| Entities | Lit per pixel like the world (PCSS, SSAO, sun GGX, Bliss fallback) | dim/composite1.fsh (entity mask 0.45) | yes | model.wgsl:133-136, entity.wgsl:146-150: light per vertex; L:1825 `sun_on_body` keeps the old sky-light gates | partial | (mobs off in poses) |
| Hand | Same lighting, held light x0.9, no SSAO | dim/all_solid.fsh:327-329 | yes | entity.wgsl:160-171 per vertex | partial | night, cave |
| Entities | Hurt tint | dim/all_translucent.fsh:513-515 | yes | entity.wgsl:150 | matched | n/a |
| Blocks | Breaking cracks | dim/composite3.fsh:357-359: mix(c·6, c, luma)·albedo | yes | entity.wgsl:223-233 multiply, darken 0.6 | differs | n/a |
| DH | Far land lit by the same deferred pass (flat normals, SSAO, SSR, contact) | dim/composite1.fsh:832-836; settings.glsl:733-735 | yes | W:1957 `lod_color` → `shade_soft_lit`; far_ao | partial (no contact or SSAO equivalents) | noon vista, dawn vista |
| DH | Far land keeps its sky and block lightmaps | dim/DH_solid.fsh:163-166 | yes | W:1962 the LOD forces sky 1 and block light 0 | missing | forest vista (far canopy floors), night vista (far lamps), dawn vista |
| DH | Albedo: flat colour + a noise texture | dim/DH_solid.fsh:118, 183 | yes | W:1983-1991 real texture to LOD_DETAIL, then its average; crown clumps W:2026-2040 | differs on purpose | noon vista |
| DH | Shadows: none (DH shadow map off), the lightmap fallback + SSRT | settings.glsl:721 | off | past 128 the sky mask (W:425) | matched (its SSRT part missing, above) | vista |
| DH | Overdraw prevention | dim/DH_solid.fsh:159-170 | yes | the engine's cube/LOD hand-off | matched (unsure) | vista seam |
| Misc | Seasonal leaf colour | settings.glsl:47 | off | seasons (mods/base vitals) | ours only | n/a |
| Misc | Wavy plants | dim/all_solid.vsh:332-366 | yes | W:207-247 wind, gusts, parting | ours only (more) | forest |
| Misc | Path tracer, RT shadows, RT reflections | none | n/a | pathtrace.wgsl; W:496-524, W:826 | ours only | n/a |
| Misc | Damage, low health, water on the lens | lib/gameplay_effects.glsl:36-108 | yes | post/nausea.wgsl, post/blindness.wgsl | out of scope (screen post, the other audit) | n/a |
| Debug | Debug views (normals, specular, direct, indirect) | settings.glsl:774-784 | n/a | look_debug 1/2 (direct, indirect) | partial (no normals, specular or AO view) | n/a |

**Counts:** matched 29, partial 17, missing 11, differs 8, differs on purpose 3, ours only 9, out of scope 1 (78 rows).

### 2. Missing and partial: what building each would mean

Cost: S < 50 lines, M 50-200, L > 200. GPU on an M-series Mac at 1080p: none / low / med / high. Weight: visual weight on the 10 poses.

**Shared prerequisite: depth before shading.** Bliss is deferred. It computes SSAO and the contact ray from depth, then lights the pixel with them. Ours is forward. The engine already has an opaque depth prepass (`core:depth_prepass`, world_render.rs:527-532, today only while tracing). Two items need it: SSAO-in-lighting and SSRT contact shadows.
- Option A: run the prepass always and add a half-res compute between prepass and shading. Ours already has post_low infrastructure for half-res work. Bind its result to the world pass.
- Option B: store the sun's share of each pixel's light (`LIGHT_SUN / LIGHT_ALL`, 1 channel) in an extra target, then darken in post. Post AO would then hit the indirect light only, and the post contact ray the sun only.
- Either is an engine change plus an ENGINE_CHANGES line. Cost: M-L in the engine. GPU: low-med (the prepass costs geometry twice on tiled GPUs; that is measured already for RT).

| # | Item | Building it (file / function) | Cost | GPU | Weight |
|---|---|---|---|---|---|
| 1 | **Screen-space contact shadows toward the sun** (SSRT), also past 128 blocks and on far land | New half-res pass after the prepass: 16 fixed-pixel steps toward the sun, thickness test 0.035. Then `min()` into `sun_shadow_once` (W:482); past the reach, into `sky_sun_mask` (W:425) and `lod_color`. Or a post pass on the sun share (option B) | M (+ prerequisite) | low-med | **high** (vistas: far hills and crowns self-shadow in Bliss, flat in ours past 128; forest) |
| 2 | **PCSS: blocker search + 13-tap rotated filter** | `shadow_in` (L:1714): add 4 blocker taps to get the depth gap, scale a 13-tap spiral (rotated per pixel by IGN, TAA averages it) between min and max radius, keep the max-pair gap closing on the centre tap. Leaf/plant lift and canopy fixes may then go | M | low-med (+13 loads, −16 today if the 4x4 goes) | **high** (forest, noon vista, dawn: tree shadows sharp at the foot, soft far from it) |
| 3 | **SSAO on the indirect light only, and its cap on the sky facing** | Replace `contact` + `far_ao` with one 7-tap half-res SSAO (CleanSample ring, radius by distance, as composite.fsh:239-304). Apply it in `light_on_bliss` to indirect + lamps (L:2498) as min(vertex AO curve, SSAO^6), and cap `facing` by mix(0.95, 2.5, f(SSAO)) | M (+ prerequisite) | low (one pass in place of two) | **high** (forest floor, cave, building corners, crowns' undersides) |
| 4 | Rain smoothness on open ground (Bliss's wetness) | `rain_on_ground` (L:45): in his profile no darkening. Smoothness = rain x clamp((sky-0.9)·10), F0 0.02, sky reflection by Schlick, a sharp sun GGX (roughness (1-s)²). Write a negative alpha (as `mirror_alpha`, W:926) so post/reflect.wgsl does SSR on wet ground. Follow `rain_amount` rather than the lingering `ground_wet` for the match (keep ours as an option) | S-M | med (SSR over all wet ground) | **high** (rain ground, rain vista) |
| 5 | Near shadow map to 128 blocks | `SHADOW_REACH` 80 → 128 (crates/engine/src/client/mod.rs:4657) with `core:shadow_size` 4096 to keep texel density (or a log distortion like Bliss's, a bigger change in `light_box` + `shadow_in`); `shadow_at` radii (L:1790) | S (4096) / M (distortion) | med (4096 map: +4x shadow fill) | med (noon vista, forest: 80-128 band) |
| 6 | Far land lightmaps | Engine: LOD meshes carry per-face sky (and block) light (crates/engine/src/client/lod.rs); `lod_color` (W:1962) stops forcing sky 1 | L (engine) | none | med (forest vista canopy floors, night vista lamps, dawn vista) |
| 7 | Coloured translucent shadows | Engine: a translucent shadow pass writing colour + depth (two depth maps as Bliss's shadowtex0/1 + shadowcolor0); `shadow_in` reads the tint as composite1.fsh:661-688; water excluded | L (engine) + S shader | low-med | low (no pose has stained glass) |
| 8 | Glass reflections and tint | `glass_lit` (W:1672): vanilla glass gets F0 0.02 smooth, sky reflection + sun GGX, alpha += Fresnel, negative alpha for SSR. Tint: multiply `behind_color` (W:1058, already bound for water) by normalized albedo·0.9+0.1 | M | low | low (no windows in poses) |
| 9 | Water body: shadow map per march step | `water_body_bliss` (W:1105): call `shadow_at` at each of the 2 sample points instead of one shadow | S | low | low-med (water/shore under trees) |
| 10 | Far water on the Bliss path | `far_water_lit` (W:1830): use `water_body_bliss` / the Bliss GGX / `water_fresnel` as near water does | S | low | low-med (dawn vista, water far) |
| 11 | Caustics from the wave heightmap | `caustics` (W:1087) → project along the sun, sum the 3 wave layers' ridges (waterBump.glsl:1-29 method); use for the floor and for surfaces with the eye under water | S | low | low (water/shore shallows) |
| 12 | Surfaces with the eye under water | `under_water` (L:3506) / `light_on_bliss`: depth guess 40(1-L)², min light x10, absorption on the surface, caustics on the sun | M | low | low (no underwater pose) |
| 13 | Waves on sides and falls; parallax | `water` (W:1230): for non-up normals shift the wave lookup by y + time·3 along the slope; parallax: offset xz by height x view | S | low | low |
| 14 | Particles lit as Bliss | `particle_light` (particle.wgsl:63) → shadow via `sun_reaches`/`shadow_at`, fallback (L-0.8)·25, cloud shadow, phase 2·HG(0.65)+0.5, ambient x2.5 through the curve, `bliss_block_light`; much of `light_weather` (L:2576) is reusable | S | low | low |
| 15 | Entities and hand as Bliss | `sun_on_body` (L:1825): Bliss's fallback, no sky gates; add `bliss_sun_sheen` per vertex or per pixel in model/entity fs | S | none-low | low |
| 16 | Particle distance fade | particle.wgsl `shade`: alpha 0.2-0.7 particles x clamp(1 - d/100) | S | none | low |
| 17 | Sun GGX on entities and particles | as item 15 | S | none | low |
| 18 | Rough reflections (VNDF) and Bliss metals | `mat_shine` (W:865): one VNDF GGX sample a frame (TAA averages it), mip blur by length in reflect.wgsl; metals tint by normalized albedo, keep diffuse | M | low-med | none (no LabPBR in the poses) |
| 19 | POM | `shade_soft_lit` before `shade_mat`: adaptive-step march on `mat.height` (35-80 steps within 25-60 blocks as his MAX_DIST 60), texture lookups at the new uv | M | med (only on map textures) | none (no LabPBR in poses) |
| 20 | LabPBR emission channel | `mat_at` (gpu_layout.rs:1492): read the specular alpha (< 255) as emission; Bliss's mix(lighting, albedo x auto brightness, e^curve) in `shade_soft_lit` | S | none | none (no pack) |
| 21 | Vanilla emissives auto brightness | model.wgsl / entity.wgsl glow: x mix(0.5, 100, exp(-10·exposure)) as Bliss (exposure from frame look, as `bliss_block_light`) | S | none | none-low |
| 22 | Plant normals (unsure) | Verify what normal vanilla gives crossed plants; if sideways, give plants their quad normal for the sun's n.l (keeping the sky's 1.5) at W:383 | S | none | low-med (noon ground, forest grass) |
| 23 | Debug views | look_debug: AO alone, normals, specular alone, PCSS radius, contact shadow alone (W:784-800) | S each | none | n/a (checking) |

**Five biggest missing items by weight:** (1) screen-space contact shadows toward the sun, including far land; (2) PCSS
penumbra; (3) SSAO on the indirect light with its sky-facing cap; (4) far-land lightmaps; (5) glass reflections and tint.
Two partials weigh as much as any of these:
- **Rain smoothness on wet ground (item 4).** In his profile it is shinier, not darker, so ours goes the opposite way.
- **The 80-128 block shadow band (item 5).** It comes from the far map, at about 2 blocks a texel.

Not verified:
- vanilla cross-plant normals;
- whether our raster shadow pass draws translucents;
- the waving vertex shader in our shadow pass;
- the DH overdraw hand-off;
- whether Bliss has rain rings on water;
- the exact vanilla AO curve equivalence.

---

## Part B: Bliss sky, air and frame against ours (2026-11-26)

Bliss: `~/Coding/chiacraft-archive/references/shaders/bliss/Bliss-Shader-git/shaders` (© Chocapic13, all rights reserved: method only, below). `B:` = `dimensions/` or `lib/` there. Ours: `mods/base/shaderpack/*.wgsl` (`look`, `sky`, `post`), `mods/base/assets/shaders/post*/`, `crates/engine/...`. Tip: `claude/look-speed` 177eccd33. "Ronald" = `bliss_cap2c/pack_settings.txt`. Off by default and not set by Ronald counts as off in Bliss.

Pass map (world0 → dimensions): world0/compositeN includes dimensions/composite(N-1), so dimensions/composite2 runs as Iris composite3 (`world0/composite3.fsh`). The names below are the dimensions/ files.

Two findings the prior audit (11-25) did not have, both checked in the source:
- **The air does dim what lies behind it.** overworld_fog.glsl:263 says "purely additive", but its air transmittance comes out as `atmosphereAlpha` (overworld_fog.glsl:342). composite2.fsh:417 multiplies the clouds' alpha by it. That alpha goes out as target 1 (composite2.fsh:446-448), and composite3.fsh:438 multiplies the *whole* picture (land too) by it (`color *= vl.a*cloudAlpha`). The same happens in the sky+clouds map: deferred.fsh:295 `sky *= atmosphereAlpha`. That map feeds the cloudy average, so it reaches the land's shade fill too. Ours never dims by the air (look.wgsl:3069-3076 comment, `one_fog` look.wgsl:3281, map look sky.wgsl:1518-1521). This bears on the warm "F" fill: in Bliss's map the sky and clouds are dimmed by the air before the fog and air light are added, so the air's blue counts for more.
- **Ronald's DOF is on** (`DOF_QUALITY=3`: 96 taps, `focal=1.9`, auto focus at the centre). Ours has none. Also, **Ronald's VL_RENDER_RESOLUTION=1.0** means Bliss draws its clouds *and* fog at full size: clouds are rendered inside composite2 (`CLOUDS_INTERSECT_TERRAIN`, composite2.fsh:146, 404), whose viewport follows VL_RENDER_RESOLUTION (composite2.vsh:51); deferred2 is disabled (shaders.properties:79). Ours traces clouds at half size (`core:cloud_res` 2).
- Small: Bliss's deferred.vsh has its own luma, 0.21 0.72 0.07 (deferred.vsh:55-57). It is used for the exposure mean (deferred.vsh:239) and for the cloudy average's min(3 luma, 2.5) (deferred.vsh:151). Ours uses Rec.601 `bliss_luma` for both (look.wgsl:132-136, post.wgsl:293, look.wgsl:621). The comment there cites color_transforms.glsl, which deferred.vsh does not include.

### 1. Feature table

| Area | Feature | Bliss (file:line, method) | On in Ronald's? | Ours (file:line) | State | How to check |
|---|---|---|---|---|---|---|
| Sky | Physical sky: Rayleigh, Mie, ozone | ROBOBO_sky.glsl:9-58, 95-165: 10-step view march, 4 light steps, sea-level coeffs, ozone at 35 km, Rayleigh phase (0.1c+0.28)/pi, HG g 0.8 | yes | look.wgsl:394-525 `atm_march` (same method, constants) | matched | noon vista, dawn vista; picture dE of the sky band |
| Sky | Light ray toward the sun along sun*0.5+0.1 | ROBOBO_sky.glsl:147 | yes | look.wgsl:497, 509 | matched | dawn, dawn vista |
| Sky | Multiple scattering approximation | ROBOBO_sky.glsl:152-160: unphased Rayleigh+Mie lit by the background (the averaged sky), x low_sun 1-1.8 | yes | look.wgsl:494, 514 `s_multi`, `atm_background` 568 | matched | dawn vista, night |
| Sky | Planet ground below the horizon | ROBOBO_sky.glsl:98-104 (SKY_GROUND: exp(-100(5y-0.1)²) cuts sun/moon, darkening 0.7) | yes | look.wgsl:495-496 | matched | noon vista (below-horizon band) |
| Sky | Sky texture (LUT) | deferred.fsh:250-271: 256x256 sphere map, jittered offsets, 10 steps, accumulated 6 %/frame (deferred.fsh:372-379) | yes | sky.wgsl:1430-1436 `fs_frame_look` rows 1-64: 128x64, horizon-bunched rows, recomputed each frame, no jitter or history; `sky_phys` look.wgsl:640 bilinear | partial | noon vista sky gradient smoothness; debug-13 swatches |
| Sky | Rain on the sky | deferred.fsh:263-264: toward 1.5 x average x (0.7T+0.3), most at the horizon, gone above the cloud plane | yes | look.wgsl:548-557 `atm_rain`, settled average 577-591 | matched | rain vista, rain ground |
| Sky | Sun and moon light colours | deferred.vsh:158-168: sun blackbody 4300 K x 250000/4000 x the air's transmittance along its path (25 steps), fade over the last 2.9°; moon 15000 K | yes (Sun_temp 4300, 250000, moon 200) | look.wgsl:606-617 `land_sun`, 626 moon | matched | dawn; debug-13 swatch 6 (land_sun) |
| Sky | Moonlight by phase | settings.glsl:336-350: moon light x abs(4-phase)/4 | yes (default) | `scene.moon.w` as moon_k (sky.wgsl:1442) (unsure what w holds per phase) | matched (unsure) | night, at full and half moon |
| Sky | Sun disc | sky_gradient.glsl:3-5, composite1.fsh:946: tiny smoothstep³ disc (about 0.5°) x DirectLightColor/0.00088 x 0.62, so it reddens with the air | yes | sky.wgsl:1319-1346: engine quad, hand core tints (golden), halo/wide glow, x16 HDR; `OPT_SUN_DISC` 1 | differs | dawn, dawn vista: disc size and colour |
| Sky | Moon disc and phases | sky_gradient.glsl:7-31, composite1.fsh:947: exp-shaped disc, mock normal lit by the phase's light vector, x land light x 20, rim 0.004 | yes | sky.wgsl:1113-1140 `bliss_moon` (phase from moon_light, waning side only) | partial | night (waxing moons light the wrong side) |
| Sky | Stars | stars.glsl:30-55, composite1.fsh:935-943: hash field x300, top 1 %, exp(-20(1-s)), x10 x clamp(-2 sun.y), turned with the day | yes | sky.wgsl:1079-1104 `bliss_stars` | matched | night clouds, night |
| Sky | Stars hidden by the moon | sky_gradient.glsl drawMoon `occludeStars` | yes | sky.wgsl:1139 (a) | matched | night |
| Sky | Milky Way, aurora, meteors, shooting stars, fireball | none | n/a | sky.wgsl:817-980; off under the Bliss path (sky.wgsl:1043) except fireball | ours only (off with Bliss path) | night |
| Sky | Rainbow | none | n/a | sky.wgsl:989 `rainbow_on` | ours only | rain vista after rain |
| Sky | Our cirrus + fake altostratus sheet | none (Bliss's altostratus is cloud layer 2) | n/a | sky.wgsl:1142-1170, still drawn under the Bliss path (`look_cirrus_off` 0) | ours only (doubles Bliss's layer 2) | noon vista high sky; debug 12 vs plain shot |
| Sky | Resource-pack sky | composite1.fsh:957-965 (RESOURCEPACK_SKY 0: off) | off | engine quads discarded under Bliss path (sky.wgsl:1300) | matched | – |
| Averages | 9-sample sky averages (clear and cloudy) | deferred.vsh:118-152: 3x3 pattern at elevations 62/41/24/6°, NaN centre adds 0, /9; cloudy = normalize x min(3 luma,2.5) x (1-0.7 rain), floor (0.5,0.75,1)x0.0005 | yes | look.wgsl:526-566 `atm_mean_dir`, `atm_mean`; `land_ambient` 618-625; sky.wgsl:1440-1528 texel 4 | partial: luma weights differ (Rec.601 vs deferred.vsh's 0.21/0.72/0.07) | F shade-fill hue dE; debug-13 swatches 1-2, 8 |
| Averages | Sky + clouds + fog map | deferred.fsh:274-297: per texel sky x clouds.a + clouds/5, then x atmosphereAlpha, then x fog.a + fog/5; clouds lit by sun x2 and skyGroundCol/30, fog air by cloudy avg x5 | yes | sky.wgsl:1470-1527: the nine looks only (no 256² map); clouds `bc_clouds`, fog `fog_march_parts`, 4 settling rounds | partial: no atmosphereAlpha dimming; no stored map (its other readers: water reflections, nether light) | F shade-fill hue dE; debug-13 swatches 3-5 |
| Averages | skyGroundCol (sky straight down) for clouds/fog | deferred.fsh:214 | yes | look.wgsl:2191-2195 `bc_ground_sky`, texel 5 | matched | debug-13 swatch 7 |
| Averages | Temporal smoothing of map and averages | deferred.fsh:372-379 (6 % a frame) | yes | none (recomputed each frame) | missing | rain onset; lightning frames |
| Clouds | Three layers: cumulus 250/500 (100 thick), altostratus 2000 | volumetricClouds.glsl:27-47, 81-224 | yes (cover 0.5/0.6/0.4) | look.wgsl:1876-1986 `bc_coverage`, `bc_cumulus`, `bc_alto` | matched | noon vista; debug 12 (layer opacities) |
| Clouds | Rain cover and density | volumetricClouds.glsl:41-47 (low → Rain_coverage, high → 0, alto → 1.5) | yes (Rain_coverage 1.1) | look.wgsl:1887-1895 | matched | rain vista |
| Clouds | Lighting: single + multiple scatter, powder, sky term on tops | volumetricClouds.glsl:235-256 | yes | look.wgsl:2002-2013 `bc_lighting` | matched | noon vista, dawn vista; debug-13 swatch 3 |
| Clouds | Two forward lobes + multi lobe, distant fog, air reddening | volumetricClouds.glsl:475-494 | yes | look.wgsl:2138-2152 | matched | dawn vista |
| Clouds | Layer-to-layer shadows (alto on cumulus, high on low) | volumetricClouds.glsl:366-380 | yes | look.wgsl:2022-2106 `bc_layer`, `bc_alto_layer` (unsure all taps) | matched (unsure) | noon vista |
| Clouds | From inside a slab | volumetricClouds.glsl:401-416 | yes | look.wgsl:2107-2118 `bc_inside_step` | matched | a pose at y 300 (needs a new pose) |
| Clouds | Clouds stop at terrain depth | composite2.fsh:404 (CLOUDS_INTERSECT_TERRAIN) | yes | sky.wgsl:506-525 (`far` from depth) | matched | dawn vista (hills in clouds) |
| Clouds | Resolution and upsample | composite2.vsh:51 at VL_RENDER_RESOLUTION; composite3.fsh:133-180 5-tap depth bilateral | Ronald 1.0: full size | soft_clouds.rs:18-23 half size (`core:cloud_res` 2); sky.wgsl:583-586 bilinear | differs | noon vista cloud edges; picture dE |
| Clouds | Temporal: TAA only, no own history | jitter + TAA | yes | sky.wgsl:551-553 (TAA on: none of its own) | matched | – |
| Clouds | Daily weather profiles | deferred.vsh:176-219, settings.glsl:414-507 | **off** (Daily_Weather=false) | none (game weather instead) | off in Bliss | – |
| Clouds | Cloud speed / drift | volumetricClouds.glsl:51 | yes | look.wgsl:1896-1898 | matched | – |
| Shadows | Cloud shadows on land | volumetricClouds.glsl:610-634: densities at planes (low +30, high +50, alto), exp2(-100 s²) | yes | look.wgsl:1988-2001 `bc_shadow_of(..., false)` | matched | noon vista (shadow patches) |
| Shadows | Cloud shadows in fog | volumetricClouds.glsl:642-664 (+30, exp) | yes | look.wgsl:1988, used in fog_march 3201 | matched | dawn vista (shafts under clouds) |
| Fog | One march: 8 steps, 11^u spacing, jittered, to far/DH end | overworld_fog.glsl:65-345 | yes (VL_SAMPLES 8) | look.wgsl:3169-3232 `fog_march_parts`, `one_fog` 3234, post.wgsl:171 | matched | dawn, dawn vista |
| Fog | Resolution | composite2.vsh:51 VL_RENDER_RESOLUTION, bilateral upsample | Ronald 1.0 (full) | per pixel before TAA (post.wgsl:179) | matched | – |
| Fog | Uniform fog with height falloff | overworld_fog.glsl:25 exp2(-0.15 h) from FOG_START_HEIGHT+3 | yes (50) | look.wgsl:3127-3133 | matched | night (uniform 0.38), debug 3 |
| Fog | Cloudy fog (noise shape/erosion, -0.05 above +20) | overworld_fog.glsl:31-35 | yes | look.wgsl:3134-3148 | matched (approximated noise) | dawn; debug 5 |
| Fog | Rain fog | overworld_fog.glsl:36: (0.5 low + high) x rain x noPuddleAreas | yes (0.3) | look.wgsl:3151-3153, `rain_fog_at` 3107 (snow line + snow flags, not biome) | matched | rain ground, rain vista |
| Fog | Time-of-day weights | climate_settings.glsl:202-235 | yes (Ronald's 8 values) | look.wgsl:3093-3106, 3128-3129 | matched | debug 4 at dawn/evening/night |
| Fog | Per-biome fog (swamp, jungle, dark forest) | climate_settings.glsl:133-196, shaders.properties:416-431 | **off** (PER_BIOME_ENVIRONMENT=false) | none | off in Bliss | forest |
| Fog | Sand and snow storms | overworld_fog.glsl:30, 38-43 (fast-moving noise, full uniform), shaders.properties:440-453 (desert + thunder, snowy + rain) | yes (on by default); colour only with PER_BIOME (off) | look.wgsl:2656-2686, 3198-3207 `storm_fog_density`, sand-tinted (comment wrongly says Bliss has none) | differs | needs a desert-storm pose |
| Fog | Fog phase and sky phase | overworld_fog.glsl:54-62, 108-116 | yes | look.wgsl:3039-3042 `bank_phase`, 3175-3177 | matched | dawn (toward sun) |
| Fog | Eye's sky light (lightLevelZero) | overworld_fog.glsl:174, 240 | yes | look.wgsl:3113-3119 `eye_sky_k` | matched | forest, cave |
| Fog | Cave kill (caveDetection x density and air) | overworld_fog.glsl:173, 216, 266; shaders.properties:491-509 | yes | none (only eye_k) | missing | cave |
| Fog | Shadow map in fog | overworld_fog.glsl:186-204 | yes | look.wgsl:3191-3194 `sun_reaches`, reach 128 | matched | forest (beams) |
| Fog | Coloured translucent shadows in fog | overworld_fog.glsl:193-199 | yes (TRANSLUCENT_COLORED_SHADOWS) | none | missing | water/shore (under stained glass: needs a new pose) |
| Fog | Lightning in fog | overworld_fog.glsl:239; lightning_stuff.glsl:44-61 (Iris bolt position) | yes | not in `fog_march` (land and sky only: look.wgsl:2483, 1393) | missing | rain vista during a flash |
| Air | Air in the march (Rayleigh + Mie x 24 x mult x Haze, below the cloud base) | overworld_fog.glsl:250-268 | yes (Haze 1.25, DH: x1) | look.wgsl:3209-3227 (`BLISS_AIR` 30) | matched | noon vista far land |
| Air | Air dims what lies behind (atmosphereAlpha) | overworld_fog.glsl:268, 342; composite2.fsh:417, 446-448; composite3.fsh:438; deferred.fsh:295 | yes | none: look.wgsl:3281 `c * through + f.rgb` (fog only) | missing | noon vista, dawn vista far land dE; F hue dE |
| Air | Border fog | composite1.fsh:1305-1323: last third of far, only level/down, colour = clear sky average (SKY_GROUND) | yes (BorderFog) | look.wgsl:3587-3606 (colour = sky in the pixel's own direction) | partial | noon vista horizon band |
| Fog | Fog behind translucents (water volume) | fogBehindTranslucent_pass.fsh:289+ | yes | other agent's scope (water) | (scope B) | water/shore |
| Water | Underwater fog | composite2.fsh:229-326, 431-443 (8-step VL with shadow map, dirt scatter); composite3.fsh:409-433 (absorption, threshold colour) | yes | look.wgsl:3486-3533 `under_water` (exponential + 6 noise beams) | partial | needs an underwater pose |
| Fog | Cave fog (blue distance fog when in a cave) | composite3.fsh:376-395 | yes (CaveFogFallOff 2.0) | none | missing | cave |
| Fog | Blindness, darkness | composite3.fsh:463-466 | yes | mods/base/assets/shaders/post/blindness.wgsl (darkness: unsure) | partial (unsure) | – |
| Fog | Bloomy fog | composite3.fsh:440, composite11.fsh:144-157: (1-T) x 1.25 x 0.75 x (1+rain) x (1-0.3 purkinje) x edge fade; x0.4 T under water | yes (BLOOMY_FOG 1.25) | post.wgsl:422-437 `bloomy_fog`, second 8-step march `bank_hidden` look.wgsl:3049 | partial: no purkinje term, no edge fade, none under water, march done twice | dawn (bank glow), night |
| Fog | Rain drops thin the bloomy fog | composite3.fsh:446-449 | yes | post.wgsl:430-435 | matched | rain ground |
| Weather | Seasons (foliage tint) | climate_settings.glsl:11-89 | **off** (Seasons undefined) | game seasons (vitals), not in the look | off in Bliss | – |
| Weather | Lightning flash on land | composite1.fsh:1003; lightning_stuff.glsl:8-28 | yes | look.wgsl:1314-1440, 2483 | matched (ours richer) | rain vista |
| Weather | Lightning bolt drawn | composite1.fsh:1300 (vanilla bolt, flat colour) | yes | look.wgsl:1393 `lightning_bolt` | ours only (own bolt) | rain vista |
| Frame | Auto exposure | deferred.vsh:225-264: 50 R2 samples over the central 70 %, 1/16 bloom mip, geometric mean, 0.95 per-frame ease, 0.18/log2(2.5L+1.045) x 0.62 | yes | post.wgsl:281-322 (8x8 centre-weighted over the whole view, tau 2.5 s down / 0.8 s up), look.wgsl:347-349 | partial (sample area, ease ~8x slower, luma weights) | cave (adaptation); noon vista dE |
| Frame | Rods: avgL2 and rod exposure | deferred.vsh:251-264 (0.07 0.22 0.71, cap 0.08, 0.985 ease) | yes | post.wgsl:288-318, look.wgsl:371-374 | matched | night; debug 6 |
| Frame | Purkinje | composite11.fsh:146-161 | yes (Fake_purkinje) | look.wgsl:353-363 `bliss_purkinje` | matched | night; debug 6 |
| Frame | Tonemap | color_transforms.glsl:34-40 (Hejl 2015 variant), applied composite11.fsh:163-170, exact sRGB | Ronald: ToneMap_Hejl2015 (default AgX minimal) | look.wgsl:3937-3942 `hejl_bliss`, chain 4055-4066 | matched | every pose |
| Frame | Saturation 0.3 (Rec.601, display space) | composite12.fsh:146, settings.glsl:554 | yes (0.3) | look.wgsl:4060-4062 (pow 1/2.2 in place of exact sRGB) | matched (approx) | noon vista dE |
| Frame | Crosstalk, luminance curve, 3-way grade, contrast | composite12.fsh:146-155 | off (defaults) | chain skips our grade (look.wgsl:4055-4066) | matched (both off) | – |
| Frame | CAS sharpening | composite12.fsh:143-145 (after the curve, display space) | yes (0.4) | post.wgsl:363-405 (HDR log luma before the curve, + motion push), 0.4 | differs | forest (leaf texel crispness) |
| Frame | Bloom | composite6-10 (quarter-size start, gaussian chain), composite11.fsh:140-154 added x 0.5E^0.2 x (1.5-2r²) | yes (1.0) | post.wgsl:232-278 pyramid 6 levels from half, chain_light post.wgsl:513-519 | partial (radius/levels) | dawn (sun glare), night lamps |
| Frame | Vignette (darkening) | none (composite11.fsh:99 only weights bloom) | n/a | mods/base/assets/shaders/post/bloom.wgsl:10-13: 20 % corners in the HDR world | ours only (Bliss has none) | any pose corner luma |
| Frame | God rays / light shafts | none apart from the fog march | n/a | post/shafts.wgsl, post/volume.wgsl (fx_shafts, fx_volume 1 by default, not gated by the one fog) | ours only (may double the one fog's light, unsure) | forest, dawn |
| Frame | Lens flare, chromatic aberration, film grain | none | n/a | none | matched (none) | – |
| Frame | Output dither | composite11.fsh:172, composite12.fsh:157 int8Dither (color_dither.glsl:33) | yes | none (unsure: engine swapchain) | missing | night sky banding (needs a debug view or zoom) |
| Frame | DOF (bokeh) | composite11.fsh:102-131, bokeh.glsl:170 (96 taps), auto focus at the centre depth eased (deferred.vsh:261) | **yes** (DOF_QUALITY 3, focal 1.9) | none | missing | forest, cave, water/shore (near focus) |
| Frame | Motion blur | final.fsh:95-99 | off | none | off in Bliss | – |
| Frame | TAA jitter | TAA_jitter.glsl:1-10 (8 fixed offsets) | yes | crates/engine/src/post.rs:142-180 (Halton 2,3) | differs (equivalent) | – |
| Frame | TAA history and clamp | composite4.fsh (3x3 min/max), composite5.fsh:200-210, 385-395 (smoothstep history filter, closest velocity, blend 0.12, more when far from box, tonemapped blend) | yes (BLEND_FACTOR 0.12) | crates/engine/assets/shaders/taa.wgsl:98-240 (Catmull-Rom, YCoCg variance clip, frame counts) | differs (ours handles more cases) | forest (leaf shimmer) |
| Frame | FXAA | none | n/a | none | matched (none) | – |
| Frame | Upscaling, render scale | res_params.glsl:5-12 TAA_UPSCALING | off | crates/engine/src/settings.rs:17 (`core:upscaler` FSR1, DLSS), `render_scale` | ours only | – |
| Frame | Gameplay effects: low health, damage, water on lens, powder-snow frost | final.fsh:101-104, gameplay_effects.glsl:47-103 | yes (Iris defaults) | none (only vanilla's hurt camera jolt: crates/engine/src/client/mod.rs:3521); nausea and sleep posts are ours | missing | needs a hurt/water-exit pose |
| Frame | Night vision, minimum light | deferred.vsh:150 | yes | ATM_FLOOR look.wgsl:405 (night vision unsure) | matched (unsure) | – |
| Debug | display_LUT, DEBUG_VIEW | composite3.fsh:469-476, settings.glsl:773-783 | off | base:look_debug 1-13 | ours only | – |
| Nether | Nether fog (plumes, ceiling smoke, haze, via sky map) | nether_fog.glsl, deferred.fsh:302-319 | yes in the Nether | none (no sky: engine fog, look.wgsl:9-11) | missing | needs a Nether pose |
| End | End fog (storm, vortex light, End lightning, orb) | end_fog.glsl, deferred.fsh:321-367 | yes in the End | none | missing | needs an End pose |

Counts (rows above; "ours only", "off in Bliss", "(scope B)" and both-none rows not counted): **matched 33** (incl. approx/unsure), **partial 11**, **missing 12**, **differs 7**.

### 2. Missing and partial: what building it means

Cost: S < 50 lines, M 50-200, L > 200. GPU on a Mac M-series at 1080p. Weight on the 10 poses.

1. **Air dims what lies behind (atmosphereAlpha).** Missing. `fog_march_parts` (look.wgsl:3169) returns the air's grey mean transmittance (product of `dot(kept, 1/3)`, as Bliss's); `one_fog` multiplies `c` by it (Bliss: land, sky and clouds alike, only with the eye in open overworld); `fs_frame_look` texel 4 (sky.wgsl:1518) multiplies `wet x cl.a + cl.rgb` by it before adding the fog. S, GPU none. Weight **high** on noon vista, dawn vista, rain vista (far land and horizon), **med** on the F fill hue (all poses' shade).
2. **DOF (Ronald's on).** Missing. A post pass after TAA, before the curve: CoC from the thin-lens formula (aperture = focal/fstop, focal 1.9 cm, f 2.8, capped 15 texels), auto focus on the centre depth eased in `fs_exposure` (spare texel), 96-tap disc gather at a mip level. Hooks: `post.wgsl` (new `fs_dof` or inside `fs` before `finish_picture`), a mip chain of the HDR picture (crates/engine/src/post.rs). M-L (taps are a table). GPU med (96 taps full size; 32 taps low). Weight: **low** on vistas (CoC under 1 px), **med** on forest, cave, water/shore when the centre is near. It matters for picture dE against Bliss captures with near focus.
3. **Clouds at full size.** Differs. Ronald's VL 1.0 makes Bliss's clouds full-size. Set `core:cloud_res` 1 for comparisons; add a 5-tap depth bilateral in `soft_pixel` (sky.wgsl:583). S, GPU **high** (4x the cloud trace; the largest single cost here). Weight **med** on noon vista and dawn vista cloud edges.
4. **Cave fog and cave kill.** Missing. Engine: a smoothed "in a cave" value (eye sky light 0 for some seconds, Bliss's OLD_CAVE_DETECTION) into `scene`. `bliss_fog_density` and the air in `fog_march_parts` x (1 - cave). In `one_fog` add Bliss's blue fog: colour (0.1, 0.2, 0.5) x distance curve (falloff 2.0) x down-weighting by look height x cave. S-M (Rust ~30 + WGSL ~25), GPU none. Weight **high** on cave, none elsewhere.
5. **Cloudy-average luma weights.** Partial. `land_ambient` (look.wgsl:621) and the exposure mean (post.wgsl:293) take 0.21/0.72/0.07; fix the comment at look.wgsl:132. S, none. Weight **med** on the F hue dE and night (blue fills read darker), low on exposure.
6. **Exposure sampling and ease.** Partial. `fs_exposure` (post.wgsl:281): sample only the central 70 %, uniform weights, ease by 0.95 per 60 fps frame (time constant about 0.33 s) on the light, not the log. S, none. Weight low on still poses; **med** for cave/forest transitions and moving shots.
7. **Sun disc.** Differs. A Bliss branch in `fs` for `is_sun` (sky.wgsl:1319) under `land_from_sky`: disc smoothstep³ between 1.8 and 3.2 sun radii x `land_sun` x the same gain (dimmed by the air, so red at dawn), no halo or core tints; or discard the quad and draw it in `gradient` as the moon is. S, none. Weight **med** on dawn and dawn vista (disc size, colour, bloom).
8. **Our cirrus over Bliss's altostratus.** Ours only, stacked. Gate `cirrus` (sky.wgsl:1142) off when `land_from_sky()`. S (one line), saves a little GPU. Weight **med** on noon vista and dawn vista high sky.
9. **Border fog colour.** Partial. look.wgsl:3604: mix toward the clear sky average (texel 4's clear part, or `atm_mean`), not `sky_color` per direction. S, none. Weight **med** on noon vista and rain vista horizon band.
10. **Underwater fog.** Partial. `under_water` (look.wgsl:3506): 8-step march to the depth with shadow-map taps (cap 32), Bliss's water+dirt extinction and dirt scatter lit by sun and cloudy average, then absorption with the max-absorb threshold. M, GPU low-med. Weight: none on the 10 poses (needs an underwater pose). Mostly the water agent's.
11. **Sky texture quality.** Partial. 256x256 with jittered offsets and a 6 % history in `fs_frame_look` (needs a persistent target, crates/engine/src/client/frame_look.rs). M, low GPU (once a frame). Weight low (smoother gradients, fewer banding steps at dusk).
12. **Map/average smoothing.** Missing. Same history as 11 for texels 3-5. S once 11 exists. Weight low (rain onset, lightning flashes on the land fill).
13. **Bloomy fog details.** Partial. `bloomy_fog` (post.wgsl:422): x (1 - 0.3 purkinje) (purkinje is already worked out in `finish_picture`, so move it to `chain_light`), x (1 - cdist¹⁵) edge fade, and under water use Bliss's 0.4 T. Store the one fog's transmittance in the pre-TAA pass's alpha instead of `bank_hidden`'s second march. S-M, GPU saves one 8-step march per pixel. Weight low-med on night and dawn.
14. **Lightning in fog.** Missing. In `fog_march_parts`, add `lightning_glow_at(dir*t)` x the fog's density per step (exp-shaped falloff from the bolt, as lightning_stuff.glsl:44-61). S, none (only during flashes). Weight low (rain vista during a flash).
15. **Coloured translucent shadows in fog.** Missing. The shadow map would need a colour target for glass/water (also the surfaces agent's coloured shadows). M-L, low-med. Weight low on the 10 poses.
16. **CAS placement.** Differs. Move `sharpen_delta` after `finish_picture`, on display values (CAS on sRGB-encoded colour, Bliss's composite12). Keep the motion push as ours. S, none. Weight low-med on forest (leaf texel contrast).
17. **Bloom radius and levels.** Partial (known, lkbl1). Start from quarter size, 7 levels, Bliss's weights (composite6-10). M, low. Weight med on dawn (glare round the sun) and night (lamps).
18. **Vignette.** Ours only. Drop the 20 % in `post/bloom.wgsl` (lines 10-13) under the Bliss path. S, none. Weight low-med on every pose's corners (picture dE).
19. **God rays and volume posts.** Ours only. `fx_shafts`/`fx_volume` still run with the one fog (unsure whether they add light twice). Gate them off under `one_fog_on()`, or compare with `--no-shafts --no-volume`. S. Weight med on forest and dawn (unsure).
20. **Output dither.** Missing. Triangular noise of 1/255 added at the end of `fs` (post.wgsl:93). S, none. Weight low (banding in night sky, dusk gradient).
21. **Sand/snow storm.** Differs. Bliss has storms (fast-moving dense noise, uniform fog at full), untinted with Ronald's settings. Ours is tinted and shaped differently; fix the comment at look.wgsl:3198. S to match. Weight none on the 10 poses.
22. **Moon phase side.** Partial. The engine sends fullness only (sky.wgsl:1108). Send the phase index (0-7) in `scene.moon` and use Bliss's 8 phase light vectors. S (Rust + WGSL). Weight low (night).
23. **Gameplay camera effects.** Missing. Low-health red edges, damage distortion, water drops after leaving water, frost: a post effect reading health/hurt/exit-water timers from the engine. M. Weight none on the 10 poses.
24. **Nether and End fogs.** Missing. Their own marches through a sky-map texel and no sky light (nether_fog.glsl 118 lines, end_fog.glsl 355). L each. Weight none on the 10 poses.

Not verified: whether `scene.moon.w` is Bliss's phase light; whether the engine's froxel pass still runs (and costs) with the one fog on; whether shafts/volume add over the one fog; whether darkness (warden) exists in ours; layer-to-layer shadow taps in `bc_layer` in full.
