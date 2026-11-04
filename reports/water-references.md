# Water: reference pictures and how the best packs do it

Researcher, 2026-11-04, for Lighting ("our water should look as good as the best packs'").

**The pictures:**
- 19 pictures, downloaded with Ronald's yes, for internal reference only.
- Small copies are in `water-references/`. The 1280-wide set for comparison sheets, the originals and `sources.md` (a link for each picture) are in `~/Coding/chiacraft-archive/references/water/`.
- They were picked by eye from the packs' Modrinth galleries, which mostly have no captions.
- Rethinking Voxels and SEUS PTGI had no water shots in their galleries.
- No official gallery had a close shot of waves breaking on sand: the shore picture is a coast seen from the air.

| scene | pictures |
|---|---|
| lake at noon | `lake_noon_bsl`, `lake_noon_photon`, `lake_noon_bliss` |
| golden hour, sunset | `golden_hour_complementary`, `golden_hour_complementary-unbound`, `golden_hour_photon`, `golden_hour_bsl` |
| overcast, rain | `overcast_photon`, `overcast_swamp_bliss`, `after_rain_complementary` (a rainbow) |
| night | `night_river_photon` (an aurora over a river), `night_ocean_complementary-unbound` |
| shore | `shore_photon` (a coast from the air) |
| river | `river_photon` (a valley at sunset), `river_canal_photon` |
| clear shallows | `shallows_photon` (the bottom and lilies), `shallows_above_photon` (sun sparkle, the bottom showing) |
| under water | `underwater_bliss` (light shafts), `underwater_complementary` (a reef) |

## How they do it (ideas only)

**Licences.** Complementary forbids modifying or redistributing its code. Photon allows reading and limited redistribution. Bliss is all rights reserved. BSL's and SEUS's source isn't public.

**(a) Refraction: bending what's under the surface.**
- **Photon** ([c1_blend_layers.fsh](https://github.com/sixthsurge/photon/blob/main/shaders/program/c1_blend_layers.fsh)) draws water into its own layer, with its tangent normal kept. A later pass reads the clean opaque image at `uv + n.xy / viewDist · min(thickness, 8) · 0.1`, falls back to the straight uv when the offset lands in front of the water, then blends the water on top.
- **Bliss** does the same.
- **Complementary** ([refraction.glsl](https://github.com/ComplementaryDevelopment/ComplementaryReimagined/blob/main/shaders/lib/materials/materialMethods/refraction.glsl)) has no copy of the scene before water. In a post pass it moves the lookup into the already-blended image by a scrolling noise, scaled by distance and the depth behind the water, within a water mask.
- *Cheapest for us:* Complementary's, as a post effect, if the engine gives the pack a water mask and the opaque-only depth.

**(b) Caustics.**
- **Bliss** ([waterBump.glsl](https://github.com/X0nk/Bliss-Shader/blob/Stable/shaders/lib/waterBump.glsl), applied in `composite1.fsh`) needs no shadow map:
  - each underwater point is projected along the sun to the water plane (`pos − sun/sun.y · pos.y`);
  - three rotated noise octaves are sampled there, and light is multiplied by `exp()` of their sum;
  - this fades in with depth below the surface.
- **Photon** ([shadow.fsh](https://github.com/sixthsurge/photon/blob/main/shaders/program/shadow.fsh)) bends the light by the wave normal in the shadow pass, and measures how light bunches with `dFdx`/`dFdy`, the ratio of areas. It is off by default, and needs water in a colour channel of the shadow map.
- *Cheapest for us:* Bliss's, in the forward pass for surfaces under water.

**(c) Foam.**
- **Complementary** (`water.glsl`, [link](https://github.com/ComplementaryDevelopment/ComplementaryReimagined/blob/main/shaders/lib/materials/specificMaterials/translucents/water.glsl)):
  - rebuilds the height of the ground behind each water pixel from the opaque-only depth;
  - foam = `(clamp((threshold + gap) / threshold))²`, with the threshold varied by noise, on top faces only;
  - less where there's little sky light; foam also dims the reflection.
- **Photon:** a thin "edge highlight", `(1 − 2 · thickness · |dir.y|)³`.
- **Around objects:** no pack has foam for objects as such. Being depth-based, anything in the opaque depth (rocks, posts, a boat) gets it too.
- **Bliss:** has none.

**(d) Under water: fog and light.**
- **Photon** ([water_fog_vl.glsl](https://github.com/sixthsurge/photon/blob/main/shaders/include/fog/water_fog_vl.glsl), [settings.glsl](https://github.com/sixthsurge/photon/blob/main/shaders/settings.glsl)):
  - marches 16–25 steps;
  - absorption per metre (0.20, 0.08, 0.04) under water and (0.39, 0.14, 0.07) seen from above, scattering 0.03 and 0.01;
  - a forward-scattering phase function;
  - sunlight's path through the water taken from the shadow map;
  - light shafts from scrolling caustic noise.
  - **Snell's window** comes from the Fresnel term with air/water = 1/1.333: the surface goes mirror-like past the critical angle.
- **Complementary** ([waterFog.glsl](https://github.com/ComplementaryDevelopment/ComplementaryReimagined/blob/main/shaders/lib/atmospherics/fog/waterFog.glsl)) is simpler: `1 − exp(−(d/48)²)` tinted (0.80, 0.87, 0.97), with shafts from noise in the shadow colour.
- **Bliss** fakes Snell's window as `pow(clamp(1.5 + N·V), 25)`.

**(e) Waves.**
- **Photon** ([water_normal.glsl](https://github.com/sixthsurge/photon/blob/main/shaders/include/surface/water_normal.glsl)):
  - three octaves of `sin²` waves at Gerstner speeds (w = √(gk)), each octave turned by the golden angle;
  - normals by finite differences;
  - weaker where there's little sky (caves), stronger in rain (0.04 + 0.15 · rain);
  - following the flow on flowing water.
- **Complementary:** a normal texture scrolled by wind at four scales, plus raindrop ripples.
- **Nobody moves the geometry by default.** Normals carry it all.

**(f) Colour by depth and biome.**
- **Photon:** Beer–Lambert over the water's thickness, with absorption nudged by the biome's water tint. The same numbers feed the caustics and the light under water, so it all agrees.
- **Complementary:** alpha rising as `1 − exp(−0.075 · depth behind)`, with the biome colour and a little hue noise.

**(g) Reflections.**
- Everyone uses screen-space reflections with a sky fallback, scaled by sky light so caves don't reflect sky.
- **Photon** reads last frame's colour for hits, and treats rough surfaces with GGX rays.
- **Complementary** blurs by mip level with roughness and distance, and fades at the screen's edges.

## What our engine would need, in order of how many of these it unlocks

1. **Opaque-only depth behind the water.** Every one of foam, colour by depth, refraction and Bliss-style caustics needs it.
2. **A water mask, or water drawn to its own target with a blend pass after.** The mask is enough for Complementary-style refraction; the separate target gives Photon-style refraction.
3. **The water pass's normal or mask in a target** later passes can read.
4. **Water in a colour channel of the shadow map**, with shadow depth with and without water, for Photon-style caustics and shafts. Bliss-style caustics avoid this.
5. **Last frame's colour,** for reflections of what's off the reflected ray's screen path.

So a good order for Lighting:
1. waves and colour by depth (needs 1);
2. foam (needs 1);
3. refraction (needs 1 and 2);
4. caustics the Bliss way (needs 1);
5. under-water fog with Photon's coefficients and Snell's window, in the pack;
6. shadow-map caustics and shafts last, if wanted.
