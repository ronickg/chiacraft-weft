# Shadow maps: how Minecraft shader packs do it

Research for chiacraft's sun shadows (engine: `render_shadows` in `crates/engine/src/client/mod.rs`,
`draw_shadow` in `world_render.rs`; pack: `shadow_at` in `mods/base/shaderpack/look.wgsl`).
Gathered 2026-11-04 from the source of Complementary Reimagined, Photon and Bliss, the shaderLABS
Shadow-Tutorial, OptiFine's `shaders.txt`, Microsoft's shadow-map article and MJP's survey.

**Ours**: one 2048² reverse-Z map, an orthographic box 80 blocks to each side of the camera and
400 deep along the light, snapped to whole texels: 0.078 blocks a texel.

## Distortion (not used yet)

All the packs warp the map's clip xy so texels are small near the player: `xy / (length(xy) * k +
1 - k)`; Photon k = 0.85 (with a quartic length so corners warp less), Complementary
`1 - 25.6 / shadowDistance`, shaderLABS `xy / (r + 0.1)`, Bliss a logarithm. One texel's size is
then the plain texel times `f² / (1 - k)` (k 0.85: 0.012 blocks at the centre, 0.5 at the edge),
and every bias must scale by the same factor. They need it for boxes of ±128 to ±256 blocks; at
±80 our plain texel is already fine. Warping is per vertex (fine for one-block quads), and it
slides against the world as the camera moves unless centred on the snapped centre.

## Bias (applied)

- Packs shift the receiver along the flat face normal, about a texel, up to twice that where the
  light grazes the face (Complementary `N * (0.12 + 0.0008 * d^1.5) * (2 - 0.95 NdotL)`, Photon the
  same idea), with no or a tiny constant depth bias. Ours: normal offset `1.2 texel * (2 - NdotL)`,
  depth bias 0.03 blocks plus a slope term (tan of the light's angle to the face, capped at 4).
- A large constant depth bias (ours was 0.33 blocks, about 4 texels) makes shadows float off their
  objects.
- Cracks between neighbouring meshes let thin lines of light through a shadow; ours closes them by
  taking, for each tap, the nearest caster of a 2×2 of texels.
- Light leaking under block edges in caves: packs pull the sample toward the block's centre where
  sky light is low. Ours keeps it dark there by the sky light instead.
- Foliage: alpha-tested in the shadow pass (ours too); packs skip the normal offset for plants and
  widen their filter.

## Soft edges

Packs use the hardware comparison sampler (free bilinear PCF) and rotated Vogel or spiral taps
with interleaved gradient noise, 4 to 16 taps of 2 to 3 texels; PCSS (contact hardening) where
they have TAA to hide its noise. Ours: a 3×3 of `textureLoad`s, no TAA. A comparison sampler is a
possible upgrade.

## Range, fading and low sun (applied)

- `shadowDistance` (Iris) is the box's half width, 64 to 256 in pack profiles. Shadows fade to the
  light-level shading over the last 10 to 15% of the box, by the larger of box edge and round
  distance. Ours: the larger of the two, fading from 85% to 97%.
- Shadows fade as the light nears the horizon (Complementary `|2 sunVisibility - 1|⁴`), which
  hides long smears and the swap from the sun to the moon. Ours: from 0.06 to 0.02 of the light's
  height, so golden hour keeps its long shadows (a first try from 0.16 faded them away).
- Cascades: no Minecraft pack has them (Iris gives one shadow pass).
- The sun turns about 0.3° a second, so a map could be redrawn every few frames, sampling with the
  matrix it was drawn with.

## Sources

- https://github.com/shaderLABS/Shadow-Tutorial
- https://github.com/ComplementaryDevelopment/ComplementaryReimagined (`lib/lighting/shadowSampling.glsl`, `mainLighting.glsl`)
- https://github.com/sixthsurge/photon (`include/lighting/shadows/`)
- https://github.com/X0nk/Bliss-Shader (`lib/Shadow_Params.glsl`, `lib/Shadows.glsl`)
- https://github.com/sp614x/optifine/blob/master/OptiFineDoc/doc/shaders.txt
- https://learn.microsoft.com/en-us/windows/win32/dxtecharts/common-techniques-to-improve-shadow-depth-maps
- https://mynameismjp.wordpress.com/2013/09/10/shadow-maps/
