# Path tracing: plan

Ronald, 11-06: "we do want to add path tracing as well. That's the whole reason I wanted to add it."
The goal is a **Path traced** option in Video, in the spirit of NVIDIA's RTX Remix for Minecraft:
- bounce light over several bounces;
- every emissive and block light casting shadows;
- reflections and refraction everywhere;
- light through leaves and ice;
- DLSS Ray Reconstruction on NVIDIA, our own denoiser elsewhere.

It covers the PC and any GPU with ray queries. It is off by default, part of the Max preset where the GPU supports it. The Mac and phones keep today's look.

Built on the RT work already in dev or ready: ray queries in the backend, a BLAS per cube and distant tile, a TLAS per frame, every record copied for hit lookups, `rt_sun`/`rt_hit`, soft shadows and reflections over TAA's history, LabPBR materials (the showcase blocks), TAA's jitter and motion vectors, and the DLSS hook (Streamline on Windows/Proton; NGX on Linux next).

## Architecture: a path-traced lighting pass over a rasterised G-buffer

We keep rasterising what the eye sees first; everything after is traced. Remix traces the first hit too, but our raster is cheap, already handles LOD tiles, cutout and model skinning, and keeps the Mac path shared.

1. **World pass, path-traced mode.** The pack's `world.wgsl`/`model.wgsl` write a G-buffer instead of shading:
   - albedo;
   - normal and roughness;
   - F0 and metal, emission;
   - the motion vectors TAA already writes, and depth.

   That's three more colour attachments: the backend gets multiple render targets (it has two now, colour and motion).
2. **Path-trace pass (compute, ray queries), one path a pixel at render resolution:**
   - **Direct sun and moon:** `rt_sun_soft` as now.
   - **Sky:** a few sky directions, or the sky's mip, weighted by AO from the traced bounce.
   - **Emissive and block lights:** ReSTIR DI (below).
   - **Indirect:** one diffuse or specular bounce by the material (GGX importance sampling). At the bounce hit, light it again (sun, a ReSTIR DI sample) and end the path in the radiance cache (below), which gives the further bounces almost free.
   - **Specular on smooth surfaces:** the reflection ray we have now. Through glass and water: refraction (IOR from F0: water 1.33, glass 1.5) with the surface's tint.
3. **Denoise:** DLSS Ray Reconstruction where present (it wants exactly this G-buffer: albedo, normal, roughness, specular hit distance, motion, depth). Elsewhere our own (below).
4. **Composite** into the HDR world image ahead of today's post chain: bloom, exposure, Hejl tone curve.

   Translucent things (water, glass, particles) are drawn over it as now, with traced reflection and refraction until a primary-surface-replacement step (M3).

Models and mobs: the near shadow map stands in for them today. For path tracing they get a BLAS too, rebuilt each frame from the posed cuboids (a compute expansion like `rt_expand`; few thousand triangles).

## Light cache: our own hash-grid radiance cache first (SHaRC's idea), ReSTIR GI later

| choice | for | against |
|---|---|---|
| **Hash-grid radiance cache** (world-space cells, a voxel a cell near the eye, coarser far; updated by the paths themselves each frame) | Simple and robust. Cheap multi-bounce. Fits blocks (cells line up with the world). NVIDIA's SHaRC idea, and we write our own. | Light lags a few frames after changes. Coarse light near small details (the denoiser and the one traced bounce hide most of it). |
| NRC (neural radiance cache) | best quality | CUDA/tiny-cuda-nn, NVIDIA's closed library: not an option |
| ReSTIR GI | sharp bounce light, little lag | Complex, and its temporal reuse makes ghosting. Worth it only after M2, if the cache's lag shows. |

So: the hash-grid cache in M1, ReSTIR GI as a possible M5.

**Many lights: ReSTIR DI.**
- Every emitting block (torches, lanterns, lava, amethyst, the showcase lamps) goes into a light list, gathered when cubes mesh, with a world grid for finding the nearby ones.
- Each pixel draws a few candidates by their power at it, keeps one (RIS), reuses its neighbours' and last frame's picks, and traces one shadow ray.
- Bitterli et al. 2020, our own code.

## Denoisers

| path | where | what |
|---|---|---|
| DLSS RR | NVIDIA (Windows Streamline, in dev; Linux NGX, next) | Denoise and upscale in one; DLAA, Quality or Balanced; the G-buffer feeds its guides exactly |
| Our own | every ray-query GPU | Albedo divided out, then temporal accumulation with TAA's motion and depth/normal rejection, then 3-4 à-trous passes guided by variance (SVGF-like), then albedo back in |

## Cost targets (1440p, RTX 5090)

| mode | GPU frame | how |
|---|---|---|
| Path traced + DLSS RR Quality (render 960p) | about 6-8 ms (120+ fps) | about 1 path a pixel, 2 rays a bounce, cache after one bounce |
| Path traced + our denoiser, native | about 10-12 ms (80+ fps) | the same at 1440p, plus about 1.5 ms of denoising |
| Today's hybrid (traced shadows and reflections) | +0.2-0.8 ms over maps | as measured 11-06 |

Each milestone gets a bench in the showcase hall and at NVIDIA's spots: one dev run, one branch run, per the crew's rule.

## Milestones

Each one ends in a gallery sheet against NVIDIA's demo shots. The showcase session has the 5 matched cameras in their world.

1. **M0: G-buffer and path-trace skeleton.**
   - Multiple render targets in the backend; the pack's G-buffer outputs; the compute pass with sun, sky and one diffuse bounce, raw 1 spp.
   - A debug view per buffer.
   - "Path traced" in Video, off by default.
2. **M1: radiance cache and our denoiser.** Multi-bounce light in caves and rooms, stable in motion.
3. **M2: ReSTIR DI for block and emissive lights.** Every torch and lantern shadowed; NVIDIA's storeroom and nave spots.
4. **M3: specular and transmission.**
   - Reflections and refraction everywhere (rough GGX, glass and water IOR).
   - A primary-surface-replacement light: the surface seen in a mirror or through water lit as a first hit.
   - Models and mobs in the BLAS.
5. **M4: DLSS RR on the path-traced output.** Guides from the G-buffer (Linux NGX and Windows Streamline); sheets RR against our own denoiser.
6. **M5: light through leaves and ice, ReSTIR GI if the cache lags, a speed pass.** "Path traced" goes into the Max preset.

## M2 design: block and emissive lights (11-06)

Today a traced pixel gets its block light from the lightmap (`light_on`'s yzw), unshadowed and flat; glowing textures (`in.glow`) lose their glow when traced. M2 lights them by tracing:

1. **The light list.** When a cube is meshed, the client gathers its emitting blocks: each block state's emission is in `content::Tight::light` (rgb, 0-15). One entry per emitter: position in the cube (12 bits), rgb emission, and later the face mask a glowing texture shows. It is uploaded with the cube's BLAS into a light buffer, and the cube's TLAS row gets the range (start, count). A world grid finds the cubes round a point: a GPU hash from cube coordinate to row, filled when the TLAS is made.
2. **Candidates (RIS).** Per pixel, a few candidates (4-8) from the lights of the 27 cubes round the surface, weighted by emission × 1/d² × the cosine; keep one.
3. **Reuse (ReSTIR, Bitterli et al. 2020; our own code).** The kept sample's reservoir goes into a per-pixel buffer. Next frame it is reused through TAA's motion (temporal, M cap 20) and from a few neighbours (spatial, with depth and normal tests).
4. **One shadow ray** to the chosen light (`rt_hit` to its block, the block itself let through), then the light's emission × the reservoir's weight.
5. **Into the picture.** The traced block light replaces the lightmap's block share (yzw) in `lit`, as the bounce replaced the sky's. The lightmap's share stays in the cache for bounce hits, so block light bounces too. Glowing textures add their emission in the composite (the G-buffer gets an emission channel: spec.a is free once sss moves).
6. **Checks.** NVIDIA's storeroom (lanterns, amethyst) and nave (fires), the showcase hall's lamps. Against the raster: block light by the lightmap should come close in open rooms, sharper with shadows behind pillars.

Costs to watch: the candidate search and the shadow ray, about one ray a pixel. Reservoirs are 16 bytes a pixel.

## Split

To go faster, a second session takes **the denoiser track**. It is independent once M0 fixes the G-buffer layout:
- our SVGF-style denoiser;
- DLSS RR's guides on the path-traced output;
- the Linux NGX upscaler.

I'd keep the path tracer itself: G-buffer, integrator, cache, ReSTIR DI, transmission. Two writers stay apart:
- the second session: `post.rs`, the DLSS hook, `backends/vulkan/src/ngx.rs` and new denoiser shaders;
- me: the world pass outputs, `world_render/rt.rs`, the path-trace compute shaders and the pack's G-buffer code.

## Open choices for Ronald

1. Should path tracing keep our stylised palette (Lottes/Hejl curve, Bliss-like grade)? Or should it aim at Remix's neutral photoreal look? I'd keep ours: the same grade, and the light becomes physical.
2. Should "Path traced" sit in Max wherever the GPU can trace? Or should it stay a separate opt-in until M4? I'd keep it opt-in until M4.
