# Path tracing research: doing it the best way (11-25)

Ronald asked for a check before the path tracer grows further: "do some research online just to
be sure we gonna do it optimally and well". This report has three parts:

- what papers, NVIDIA and AMD documentation, shipped games and open engines do, with sources;
- how our code compares (`claude/rt-path-tracing` 40b341fa1, `claude/pt-denoisers-13c78d`
  1df5df64e, dev's renderer and RT backend);
- what to change, ranked by picture gain per cost, each change mapped to a milestone in
  `notes/crew/plans/rtx-windows.md` (W2, M2-M6, P).

Licences:
- RTXDI, SHaRC/RTXGI, NRD and NRC are under the NVIDIA RTX SDKs licence: method only, from the
  papers and docs.
- dxvk-remix is Zlib (plus an MIT file) and may be read and reused file by file.
- Quake II RTX is GPL-2: read only.
- AMD holds a patent on hash-grid reservoir reuse (US11816781B2), and NVIDIA holds patents on
  path-space-filtering hashes. Nobody has done a patent review; one is needed before any
  commercial release.

Numbers marked *est.* are ours and need a bench.

## Summary: are we building it right?

**Mostly yes.** These parts match what shipped path tracers and the literature do:
- triangle BLAS per cube, compacted;
- an opaque and a see-through geometry apart;
- a TLAS rebuilt each frame;
- ray queries in compute;
- demodulated diffuse and specular with hit distances;
- a world-space hash-grid cache with face direction in the key;
- RIS over nearby lights with one shadow ray;
- SVGF of our own with DLSS RR beside it.

Indiana Jones, Doom: The Dark Ages and the Remix runtime itself all ship the same shape: 1 path a
pixel, 1-2 traced bounces, a SHaRC-style hash cache after that, ReSTIR DI for lights, and RR or
NRD.

**The changes worth making, roughly in order of gain per cost:**
1. **The cache only learns at the primary surface.** SHaRC learns along sparse multi-bounce
   update paths. As a result, the cells our bounce rays read (ceilings, walls behind the eye, cave
   interiors) are often empty, and our bounce hits fall back to the lightmap's sky.
2. **We trace one extra full-cost ray per pixel**, only to re-find the primary surface the
   G-buffer already has.
3. **The block-light shadow rays use the full closest-hit query**, texel work included, where an
   any-hit visibility query would do.
4. **The DLSS RR cost on 30-series is about 5.8 ms at 1080p** (NVIDIA's own table, desktop 3060).
   The laptop's Low and Medium presets should use our SVGF. RR's own model letters are D/E/F; M
   is a Super Resolution preset only.
5. **Our denoiser reprojects specular light with the surface's motion.** That ghosts reflections.
   It also has no fast history and no gradient pass, so torch and day/night changes lag.
6. **Small correctness items, each nearly free:**
   - a firefly clamp;
   - white-noise hashes everywhere (RR's rule; the sun cone and the reflection cone still use
     interleaved gradient noise);
   - a "ray shorter than the cell" rule for the cache;
   - resetting cache cells when a light block is placed or broken;
   - emissive textures at bounce hits.
7. **On our raw Vulkan backend (ash), opacity micromaps and SER are available.** wgpu exposes
   neither, but we don't trace through wgpu. Opacity micromaps cut Indiana Jones' main trace by 55
   % in foliage; SER saves 20-50 %. Both need Ada or newer, so they help the 5090, not the 3060.

## 1. Tracing voxels fast

### What others do

**Triangles vs AABBs**
- NVIDIA's advice for voxels is to mesh the visible faces into triangles.
  - The reason: with custom (AABB) primitives, the leaf test leaves the RT cores and runs on the
    shader cores.
  - In ray query, each AABB candidate is a round trip to our loop, like an any-hit.
  - Sources: [NVIDIA forum 1](https://forums.developer.nvidia.com/t/using-rtx-acceleration-for-voxel-tracing/74007), [NVIDIA forum 2](https://forums.developer.nvidia.com/t/best-practice-voxels-use-triangles-or-custom-geometry-for-calculations-not-rendering/115230).
- The only measured comparison found: custom primitives about 2x slower than hardware triangles,
  for tubes, not voxels ([arXiv 1912.08485](https://arxiv.org/pdf/1912.08485)). No published
  benchmark does a Minecraft chunk both ways.
- AABB designs that do work:
  - Teardown's successor engine: Vulkan, 8³ voxel chunks as AABBs with intersection shaders,
    DLSS RR as the denoiser ([Voxagon 2024](https://blog.voxagon.se/2024/12/29/year-summary.html)).
  - Werner et al., VMV 2024: hardware AABBs with a sparse voxel DAG (SVDAG) inside each box
    ([paper](https://diglib.eg.org/handle/10.2312/vmv20241196)).
  - Tight boxes per brick cut false hits ([NVIDIA 2023](https://research.nvidia.com/publication/2023-05_tight-bounding-boxes-voxels-and-bricks-signed-distance-field-ray-tracer)).
- Software tracing:
  - Teardown ships a voxel DDA (digital differential analyser: stepping cell by cell) with mip
    occupancy in fragment shaders; no hardware RT, no GI ([acko.net](https://acko.net/blog/teardown-frame-teardown/)).
  - Java Minecraft shader packs voxelise the world through the shadow pass and DDA it (SEUS PTGI,
    [Rethinking Voxels](https://modrinth.com/shader/rethinking-voxels)). Their known artefacts:
    a short range, light through alpha texels, and flicker.
  - Research structures: ESVO ([Laine & Karras 2010](https://www.nvidia.com/docs/IO/88889/laine2010i3d_paper.pdf)),
    SVDAGs (Kämpe et al. 2013), [HashDAG](https://github.com/Phyronnaz/HashDAG),
    [64-trees](https://dubiousconst282.github.io/2024/10/03/voxel-ray-tracing),
    [NAADF](https://www.cg.tuwien.ac.at/research/publications/2026/ulschmid-2026-naadf/)
    (Eurographics 2026: about 10x faster than DAGs for voxel GI).
  - Lumen splits near and far: precise tracing near, a coarse distance field far
    ([SIGGRAPH 2022](https://advances.realtimerendering.com/s2022/SIGGRAPH2022-Advances-Lumen-Wright%20et%20al.pdf)).
- Minecraft RTX (Bedrock, 2020):
  - a path tracer from Quake II RTX's line, A-SVGF and an irradiance cache, per a third-party
    frame analysis;
  - NVIDIA's recommended RT distance was 8 chunks, capped at 24
    ([NVIDIA](https://www.nvidia.com/en-gb/geforce/news/minecraft-with-rtx-beta-out-now-download-play),
    [MCPE-80205](https://bugs-legacy.mojang.com/browse/MCPE-80205));
  - its known bugs: lost small shadows, and emitters treated as a point in a block's corner
    ([MCPE-145467](https://bugs.mojang.com/browse/MCPE-145467)).
- Remix for Minecraft (10-28):
  - RTXDI for thousands of shadowed lights, up to 4 bounces, RTX 30 or newer
    ([NVIDIA](https://www.nvidia.com/en-au/geforce/news/remix-1-6-dlss-5-comfyui-2-0-minecraft-rtx-beta/));
  - it traces see-through texels in a loop: shadow rays multiply by what each texel lets through
    (our `RTX mod study`).

**Acceleration structures** ([NVIDIA best practices](https://developer.nvidia.com/blog/best-practices-for-using-nvidia-rtx-ray-tracing-updated/),
[compaction](https://developer.nvidia.com/blog/tips-acceleration-structure-compaction),
[Indiana Jones OMM](https://developer.nvidia.com/blog/path-tracing-optimizations-in-indiana-jones-opacity-micromaps-and-compaction-of-dynamic-blass)):
- Mark geometry opaque wherever possible, and keep the non-opaque part small and apart.
- Compaction saves 40-60 %; Indiana Jones went from 1027 to 606 MB.
- A block edit changes topology, so rebuild the cube rather than refit.
- TLAS: rebuild with fast trace, and cull instances by distance.
- Any-hit work was 17 % of Indiana Jones' main trace. Opacity micromaps took it from 7.90 to
  3.58 ms, and the SHaRC update from 1.99 to 0.89 ms (RTX 5080, 1080p). Most of the cost is the
  switching between hardware and shader, not the shader itself.

**APIs**
- wgpu (v30): ray query, AABB BLAS, compaction, and experimental RT pipelines. It has no refit and
  no opacity micromaps; SER is trunk-only, pipelines only
  ([CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md)).
- Vulkan:
  - SER is `VK_EXT_ray_tracing_invocation_reorder` and needs RT pipelines, not ray query
    ([spec](https://docs.vulkan.org/refpages/latest/refpages/source/VK_EXT_ray_tracing_invocation_reorder.html)).
  - Opacity micromaps are `VK_EXT_opacity_micromap`.
  - Mega Geometry is `VK_NV_cluster_acceleration_structure` / `VK_NV_partitioned_acceleration_structure`.

**Hardware**
- Ampere (the 3060): SER is accepted but does nothing
  ([Microsoft](https://devblogs.microsoft.com/directx/ser/)); there is no opacity micromap engine.
- Ada: twice Ampere's triangle rate, plus OMM and SER in hardware
  ([whitepaper](https://images.nvidia.com/aem-dam/Solutions/geforce/ada/nvidia-ada-gpu-architecture.pdf)).
- Blackwell: about twice Ada again, plus cluster intersection.
- SER gains: 20-50 % in path tracing; 24 % in Indiana Jones' main trace
  ([NVIDIA](https://developer.nvidia.com/blog/path-tracing-optimization-in-indiana-jones-shader-execution-reordering-and-live-state-reductions/)).
- NVIDIA publishes no rays-per-second figures. Nobody has published a 3060 Laptop path-tracing
  number; we measure our own.

### Ours, against that

| piece | ours | verdict |
|---|---|---|
| BLAS | triangles per cube, opaque and see-through geometries apart, PREFER_FAST_TRACE + ALLOW_COMPACTION, compacted when it saves over 10 % (`backends/vulkan/src/rt.rs`) | best practice |
| TLAS | rebuilt each frame, camera-cube relative, distant tiles out past 1100 blocks, masks for faces / quads / LOD | best practice |
| edits | the old BLAS casts until the new one is built, builds spread (384 a frame) | right; rebuild, not refit |
| see-through texels | ray-query candidate loop with a texel lookup (`rt_hit`, `rt_sun`), leaves' coverage from a small mip | Remix's method; the costly part (see below) |
| backend | **raw Vulkan through ash** (`backends/vulkan`), not wgpu | OMM and SER reachable when wanted |
| records copy | `RECORDS_BYTES` fixed at **512 MB** (`world_render/rt.rs`) | too big for the 6 GB laptop's Low; size it by view distance |
| primary hit | rasterised G-buffer, then **`rt_hit` from the eye to find the same surface again** (`cs_trace`) | one full closest-hit ray a pixel spent to learn what the G-buffer could store |
| far | traced to 384 blocks, faded to the lightmap's light by the edge | matches Minecraft RTX's distance cap; fine |

## 2. Direct light from many lights

### What others do

**ReSTIR DI** ([Bitterli et al. 2020](https://developer.download.nvidia.com/devblogs/ReSTIR.pdf))
- Parameters: 32 candidates; spatial reuse with 5 neighbours (3 unbiased) in a 30 px radius;
  neighbours rejected past 10 % depth or 25° normal; the history M capped at 20x.
- The biased form darkens at contacts.

**HPG 2021: the shape RTXDI ships** ([Wyman & Panteleev](https://d1qx31qr3h6wln.cloudfront.net/publications/hpg21_rearchitectingReSTIR.pdf))
- Presampled light tiles: 128 sets of 1024 lights drawn by power; each 8x8 pixel tile draws its
  candidates from one set. That turns 64M incoherent light fetches into 128k.
- 1 temporal and 1 spatial reuse, 2 visibility rays.
- 1.9-5.1 ms in all at 1080p on a 3090.

**RTXDI's defaults** (method only; [Integration.md](https://raw.githubusercontent.com/NVIDIA-RTX/RTXDI/main/Doc/Integration.md),
[NoiseAndBias.md](https://raw.githubusercontent.com/NVIDIA-RTX/RTXDI/main/Doc/NoiseAndBias.md))
- 8 local-light candidates and 1 BRDF candidate.
- Temporal: max history 20, depth threshold 0.1, normal threshold 0.5, boiling filter 0.2.
- Spatial: 2 samples in 32 px, 8 on disocclusion.
- A light index remap from last frame to this one, so reservoirs survive changes to the light
  list.
- Reusing final visibility cuts the shadow rays to about 30 %.

**ReGIR** ([RTG II ch. 23](https://boksajak.github.io/files/RTG2_ReGIRGridBasedReservoirs.pdf))
- World cells, each holding 64-512 reservoirs filled by RIS; the distance is clamped to the cell.
- About 1.5 ms and 16 MB on a 3090.
- Best for secondary hits; it ignores visibility.

**Light trees** ([Moreau et al. 2019](https://research.nvidia.com/labs/rtr/publication/moreau2019manylight),
[Conty & Kulla 2018](https://fpsunflower.github.io/ckulla/data/many-lights-hpg2018.pdf)):
ReSTIR beats them at equal time. For thousands of identical small emitters, a grid gives the
distance falloff more cheaply.

**Quake II RTX**
- Light lists per PVS cluster (the map's precomputed visibility regions).
- Since v1.2, a statistical visibility per cluster, so lights that are usually hidden get drawn
  less ([RTG II ch. 47](https://link.springer.com/chapter/10.1007/978-1-4842-7185-8_47)).
- Our 27-cube lists are the same idea.

**Capcom RE Engine** ([GDC 2026](https://docswell.com/s/CAPCOM_RandD/5DM2NL-gdc2026-implementing-real-time-path-tracing-in-re-engine))
- No ReSTIR DI: streaming RIS with 32 candidates from a 3D light grid of bitmasks.
- 2-3 shadow rays at the first bounce, 1 after.
- 8.78 ms on a 4070 Ti and 24.2 ms on a 3060, at 1080p with DLSS Performance.

### Ours

`pt_block_visibility`:
- It gathers the lights of the 27 cubes round the point and draws 8 candidates uniformly.
- Each candidate is weighted by the lightmap's own level there (emission less the Manhattan
  distance), one is kept by RIS, and one shadow ray goes to its middle.
- The result is a **visibility ratio** applied to the lightmap's unshadowed block light:
  `diffuse -= block * (1 - vis)`.

That is a known, sound estimator. It is Heitz et al. 2018's "analytic direct light times a
stochastic shadow ratio" ([paper](https://eheitzresearch.wordpress.com/705-2/)). It keeps our grade
(the lightmap's colour and reach) and lets tracing add only the shadows. Keep it.

Next to the best practice:
- **No reuse yet.** Temporal and spatial reuse is the biggest noise drop per ms: about 160
  effective candidates for one reservoir read. It is M2 already; use RTXDI's numbers above.
- **The shadow ray is `rt_hit`:** a closest-hit query with texel and record decoding at the hit.
  A visibility query (`RAY_FLAG_TERMINATE_ON_FIRST_HIT`, the alpha loop of `rt_sun`, no hit
  decoding) is cheaper and gives coloured light through stained glass for free.
- `held: array<vec4<u32>, 27>` and the linear search over it run per candidate, in every pixel.
  Presampling per cube neighbourhood or per 8x8 tile (HPG 2021) moves that out of the pixel loop.
- One ray to the block's middle gives hard shadows from a 1-block source. A point on the
  emitter's face, or a small sphere for torches, gives the soft penumbra Remix shows.
- Bounce hits get unshadowed lightmap block light (`pt_lit`). That is fine at the bounce: the
  error is blurred. ReGIR is the tool later if caves need it.

## 3. Indirect light: cache, ReSTIR GI, probes

### What others do

**SHaRC** (method only; [Integration.md](https://github.com/NVIDIAGameWorks/Spatial-Hash-Radiance-Cache/blob/main/docs/Integration.md), [repo](https://github.com/NVIDIA-RTX/SHARC))
- Three passes:
  1. **update:** sparse paths from about 1 pixel in 25 (4 %), up to 4 bounces, each vertex
     adding its radiance to its cell;
  2. **resolve:** blend into history and evict stale cells;
  3. **query:** at non-primary hits; the path ends there.
- Cell size grows logarithmically with distance from the camera.
- The key: quantised position, level, and 3 sign bits of the normal.
- 40 B a cell; about 4M cells, 160 MiB.
- Rules:
  - don't use the cache for segments shorter than a cell;
  - for glossy lobes, only past a cone-spread rule.
- Later versions added fade acceleration (reset when the light keeps dropping) and "responsive
  lighting".

**dxvk-remix** (Zlib; [RtxOptions.md](https://github.com/NVIDIAGameWorks/dxvk-remix/blob/main/RtxOptions.md))
- SHaRC is its default indirect mode: update at 1/5 resolution, 10 frames' accumulation, stale
  after 64 frames.
- Update radiance is clamped to 100x the cell's mean, and update vertices get a roughness floor
  of 0.4.

**Shipped titles**
- Indiana Jones: main trace up to 2 bounces then SHaRC; update up to 4.
- Doom: The Dark Ages: SHaRC, SER and RR
  ([NVIDIA](https://www.nvidia.com/en-us/geforce/news/doom-the-dark-ages-path-tracing-dlss-ray-reconstruction-update/)).
- Cyberpunk: ReSTIR GI since patch 2.1, and SHaRC
  ([wccftech](https://wccftech.com/cyberpunk-2077-2-1-patch-path-tracing-improvements/)).
- Capcom: its own ReSTIR GI.

**AMD GI-1.0** ([paper](https://ar5iv.labs.arxiv.org/html/2310.19855))
- The cache key adds a **"ray shorter than the cell" bit against leaks**.
- 1.9-3.4 ms at 1080p on a 3080-class GPU.

**ReSTIR GI** ([Ouyang et al. 2021](https://research.nvidia.com/publication/2021-06_restir-gi-path-resampling-real-time-path-tracing))
- 9-166x lower error than 1 spp path tracing.
- It needs Jacobians, a boiling filter, and validation when the light changes. Remix also
  randomises reprojection for RR.

**DDGI probes** ([Majercik 2019](https://research.nvidia.com/publication/2019-05_dynamic-diffuse-global-illumination-ray-traced-irradiance-fields)):
coarse, and they leak at 1-block walls. A surface hash cache fits blocks better.

**NRC** ([Müller 2021](https://arxiv.org/abs/2106.12372)): about 2.6 ms on a 3090, tensor cores,
closed DLLs. Remix now defaults to SHaRC over it. It stays the later optional row the plan
already names.

### Ours

`pt_cell`, `pt_cache_*` and `cs_cache` in `pathtrace.wgsl`.

**Where we match or beat SHaRC**
- Cells are a block wide to 24 blocks, then double with distance: the same idea as SHaRC's
  logarithmic levels.
- The key holds **one of 6 face directions**. That is better than SHaRC's sign bits, which give
  +X, +Y and +Z faces the same bits at a corner.
- Lookups are jittered along the face, SHaRC's stochastic trilinear.
- Fixed-point sums with a clamp at 64, a resolve with a 1/24 minimum blend, eviction after 240
  frames.
- 2^20 cells × 40 B = 40 MB. That fits the laptop.

**Where we differ, and should change**
1. **What trains the cache.** Ours: one pixel in four adds what its bounce brought *at the
   primary surface*. So:
   - only cells seen on screen learn;
   - a bounce ray that meets a ceiling, a wall behind the eye or a cave's far side finds no
     cell, or a stale one, and falls back to the lightmap's sky light (`pt_lit`'s else branch);
   - the multi-bounce light the cache exists for only shows where both ends are on screen.

   SHaRC's update paths store at **every vertex**, off screen too.

   Fix, cheapest first:
   - (a) at our bounce hit, also add that hit's light (its sun ray, plus the cache's or the
     lightmap's indirect) into the hit's own cell. It is a write we can afford, since the ray is
     already traced.
   - (b) a sparse update pass: 1 pixel in 25, 2-4 bounces, each vertex written. That is SHaRC's
     shape, about 0.9 ms in Indiana Jones on a 5080.
2. **Leaks at coarse cells.** Past 24 blocks, cells of 2-64 blocks hold a floor inside a wall and
   the floor outside it under the same key. Two fixes:
   - GI-1.0's rule: a bounce shorter than its cell doesn't read the cache; it uses the lightmap's
     share or traces on;
   - keep level 0 a little further out where the view is enclosed.
3. **Lag after light changes.** Placing or breaking a torch moves the light for up to 24 frames:
   the blend floor is 1/24 and the history cap 256. Two fixes:
   - reset the cells within the light's reach when a light block changes (we know exactly where);
   - SHaRC's fade acceleration (reset when a cell keeps dropping).
4. **Feedback guards.** Add Remix's two guards:
   - clamp a sample to about 100x the cell's mean before adding it;
   - a roughness floor of 0.4 for specular hits written in.
5. **Glossy reads.** A reflection ray's hit reads the cache like a diffuse one. SHaRC reads it
   only past a cone-spread rule, or at the second bounce on: below that, trace on.

**ReSTIR GI** stays M5, if the cache's lag or blur still shows after these. Its radiance would be
the cache's value at the first hit, as Cyberpunk and Capcom do.

## 4. Denoising

### What others do

**NRD** (method only; [repo](https://github.com/NVIDIA-RTX/NRD), [NRDSettings.h](https://github.com/NVIDIA-RTX/NRD/blob/master/Include/NRDSettings.h))
- Denoisers: REBLUR (recurrent blur) and RELAX (à-trous, the SVGF family, "designed for
  RTXDI"), plus SIGMA for shadows.
- Costs on a 4080 at 1440p: REBLUR diffuse+specular 2.55 ms, RELAX 3.25 ms, SIGMA 0.40 ms.

| what NRD wants | why |
|---|---|
| diffuse ÷ albedo, specular ÷ envBRDF | no texture blur |
| normalised **first-bounce** hit distance, never the path's sum | specular reprojection and blur radius |
| 2.5D or 3D motion, never INF view Z | no history resets on moving things |
| max history 30 frames, fast history 6, history fix 3 frames | stability against lag |
| **specular reprojected by its hit distance** (virtual motion), not the surface's | reflections stay put as the camera moves |
| RELAX clamps the long history to a fast one (anti-lag) | light changes show within frames |
| a history confidence from gradients at 1/5 resolution, under 5 % of the frame | lighting changes (A-SVGF's idea) |
| anti-firefly on, about +0-2 % | |

**A-SVGF** ([Schied 2018](https://cg.ivd.kit.edu/atf.php))
- It re-shades a sparse set of last frame's samples with the same random numbers; the difference
  drives how much new light a pixel takes.
- About 2 ms at 1080p. Quake II RTX ships it.

**DLSS Ray Reconstruction** ([Integration Guide, Aug 2026](https://github.com/NVIDIA/DLSS/blob/main/doc/DLSS-RR%20Integration%20Guide.pdf))

Inputs:
- noisy colour, with **no denoiser before it**;
- diffuse albedo, **not in an sRGB format**, sky 0.5;
- specular albedo from the guide's envBRDF, sky 0;
- shading normals (RGB16F, world space for its own specular motion);
- linear roughness, motion, depth;
- the specular hit distance from the primary surface, sky 65504, or specular motion vectors.

Rules:
- at least 32 jitter phases;
- white noise or hash sampling only: no checkerboard, no screen-space dithers, no shared
  patterns;
- with ReSTIR, randomised temporal reuse and permutation sampling;
- no dynamic resolution and no exposure input.

Optional inputs:
- a transparency overlay;
- colour before transparency (presets E and F);
- a responsivity mask (F).

Presets: RR's are **D, E and F**. F is the DLSS 4.5 RR transformer (August 2026, driver 580+).
**M and L are Super Resolution presets.**

**RR's cost** (NVIDIA's table, Performance mode):

| GPU | 1080p | 1440p | 4K |
|---|---|---|---|
| RTX 3060 (desktop) | **5.83 ms** | 10.27 ms | 22.92 ms |
| RTX 3070 | 3.52 ms | | |
| RTX 4070 | 1.33 ms | | |
| RTX 5090 | 0.72 ms | 1.06 ms | 2.12 ms |

About 160 MB of VRAM. The laptop 3060 will be slower still (*est.*).

Other findings:
- Digital Foundry found DLSS 4.5 looks best with the game's own denoiser off
  ([Tom's Hardware](https://www.tomshardware.com/pc-components/gpus/dlss-ray-reconstruction-might-be-living-on-borrowed-time-dlss-4-5-can-reconstruct-ray-traced-reflections-almost-perfectly-without-any-denoisers)).
- AMD: the FidelityFX Denoiser 1.3 is MIT-licensed and readable
  ([GPUOpen](https://gpuopen.com/manuals/fidelityfx_sdk/techniques/denoiser/)). FSR Ray
  Regeneration is an RDNA 4-only ML denoiser.

### Ours

`denoise.wgsl`, `denoise.rs`, `fs_pt_guides`.

**Right**
- Diffuse and specular apart, each demodulated: by albedo, and by `pt_env_brdf` (Karis' fit).
- Hit distances out: the first bounce's, sky 65504.
- Temporal along TAA's motion with depth and normal tests.
- Moments, variance, a spatial variance while the history is short.
- À-trous steps that stop at depth, normal and light edges, and SVGF's feedback from the first
  step.
- Untraced pixels kept out.
- RR fed raw light (`reconstruct` skips ours).
- RR's guides taken from the G-buffer.

**To change**
1. **Specular reprojection.** Ours takes the surface's motion for both lobes. A mirror's
   reflection then smears as the camera moves. Fix: reproject smooth specular by its hit distance
   (the virtual point behind the mirror, as ReLAX and RR's own specular motion do), and fall back
   to the surface's motion as roughness rises.
2. **Anti-lag.** We have `max_len` 32 and alpha 0.05, but no fast history and no gradients. Fix,
   in this order:
   - RELAX's clamp of the long history to a 4-6 frame fast history: cheap;
   - then A-SVGF-style confidence at 1/5 resolution, for torches placed, lava and lightning.
3. **Fireflies.** Only `HUGE` (6e4) clamps today. Fix:
   - a luminance clamp on indirect light before our SVGF, e.g. by the pixel's or the
     neighbourhood's history (*est.* tuning);
   - lighter before RR, which wants it nearly unbiased.
4. **RR's rules to check in our code**, each nearly free:
   - the albedo guide is linear, not an sRGB image (the G-buffer stores albedo in sRGB, 630cbcf67;
     check what `fs_pt_guides` hands over);
   - `rt_cone` (the sun's cone and the reflection cone) uses interleaved gradient noise, a
     screen-space pattern RR's guide rules out: move it to `pt_rand`'s pcg3d;
   - the RR model row offers F;
   - water, glass and particles go through the transparency or colour-before-transparency guides
     when M3 brings them in.

## 5. Budgets and the Low-to-Ultra ladder

**Rays per traced pixel today (`cs_trace`)**

| ray | what for | cut possible |
|---|---|---|
| 1 | re-find the primary surface | yes, from the G-buffer |
| 1 | sun or moon | none |
| 1 | diffuse bounce | none |
| 1 | the sun at the bounce hit (`pt_lit`) | none |
| 1 | block-light shadow, where there is block light | cheaper query |
| 1 | reflection, where F0 > 0 and smooth | Low: smooth only |
| 1 | the sun at the reflection's hit | none |

That is up to 7 rays a pixel; 5 on most land. Indiana Jones spends about one path of 2 bounces a
pixel.

**What shipped games do**
- 1 path a pixel at 50-67 % scale, 1-2 traced bounces, then a cache.
- Remix presets (`rtx_options.cpp`, `rtx_gpu_profile.cpp`, Zlib):
  - **Low and Medium:** 0-2 bounces, Russian roulette 0.7, sparse 0.1-0.125.
  - **High:** roulette 0.9, ReSTIR GI with pairwise bias correction.
  - **Ultra:** 1-4 bounces, a cache of important lights for next-event estimation (NEE).
  - **Auto preset:** Turing gets Low, Ampere Medium, Ada High, Blackwell Ultra, and **8 GB or
    less drops one level**. That puts our 6 GB 3060 Laptop on Low by NVIDIA's own rule.
- VRAM:
  - Portal RTX takes about 8 GB at 1080p;
  - Indiana Jones' path tracing needs 12 GB;
  - Cyberpunk Overdrive's target was a 3090 at 1080p 30 fps.

  Nobody publishes a 3060 Laptop path-tracing number. Ours, from W2, will be the first.

**Proposed ladder** (the output resolution is set apart; *est.* until benched; the laptop judges
Low, the 5090 Ultra):

| row | Low (3060 Laptop) | Medium | High | Ultra (5090) |
|---|---|---|---|---|
| render scale | 50 % | 58 % | 67 % | 67-100 % (DLAA) |
| paths a pixel | 1 | 1 | 1 | 1-2 |
| traced bounces, then the cache | 1 | 1 | 2 | 2-3 |
| cache update | at the bounce hit (fix 1a) | + sparse update 1/25 px, 2 bounces | 3 bounces | 4 bounces |
| block lights | RIS 8, temporal reuse | + spatial 1 | + spatial 2, visibility reuse | + soft emitters |
| denoiser | ours, 3 steps | ours, 4 steps (RR offered at its cost) | RR F | RR F / DLAA |
| reflections | water and smooth only (roughness < 0.2) | < 0.4 | all | all, PSR (M3) |
| leaves in indirect rays | 2-state alpha | 2-state | 2-state | coverage |
| traced reach | 192 blocks | 256 | 384 | 384 + far tiles |
| VRAM target | under 4.5 GB in all | 6 GB | 10 GB | open |

Each row shows its ms cost in the menu, per our cost rule.

## 6. Pitfalls others hit, and where we stand

| pitfall | fix in the field | ours |
|---|---|---|
| fireflies | clamp indirect light; clamp lobe probabilities (min about 0.05); roughness up after a diffuse bounce; NRD's anti-firefly | only `HUGE`; add the clamp (§4) |
| ghosting and lag | fast history, gradients, reset on cuts | missing (§4) |
| self-hits and leaks | offset along the geometric normal (RTG ch. 6, Wächter & Binder), back-face hits count as hidden, camera-relative coordinates | lift along the face normal scaled by distance; camera-cube-relative TLAS; normal-mapped bounce mirrored over the face: all good |
| emitters counted twice | MIS between NEE and BSDF hits, or no emission on BSDF hits of sampled lights (pbrt-v4) | glow textures aren't lit at bounce hits yet (M2 step 5); when added, take them off the lightmap's share there |
| alpha foliage cost | opacity micromaps (-55 % main trace); 2-state alpha on indirect rays | 2-state in `rt_hit`, coverage in `rt_sun`; opacity micromaps on Ada+ later |
| water and glass | PSR (primary surface replacement); RR guides refracted with them; NRD's disocclusion threshold 2-10x on PSR | M3 |
| sky | the sun as its own light, the sky by cosine | as we do |
| particles under RR | transparency overlay | M3/M4 |
| far precision | camera-relative positions (a SEUS-PTGI clone darkens far from spawn) | done |
| ReSTIR boiling, darkening | M cap 20, boiling filter, MIS weights, neighbour rejection; for RR, randomised reuse | M2 to do |

## Recommendations, ranked by picture gain per cost

| # | change | gain | cost | milestone |
|---|---|---|---|---|
| 1 | **Cache learns at bounce hits** (write the hit's light into its own cell); later a SHaRC-style sparse update pass, 1/25 px, 2-4 bounces | true multi-bounce in caves, rooms, under roofs; fewer lightmap fallbacks | (a) about 0 extra rays; (b) about 1 ms *est.* | M2 finish / M5 |
| 2 | **Primary surface from the G-buffer**, not a traced ray: store a "traced terrain" flag and the face normal | none in picture; about 1 of 5-7 rays a pixel saved, the dearest kind (alpha candidates) | small | M2 finish (or W2's speed check) |
| 3 | **ReSTIR DI temporal, then spatial** on today's RIS (M 20; 2 neighbours in 32 px; boiling 0.2; light index remap on edits; permutation sampling under RR) | block-light shadows clean, not noisy | about 0.3-1 ms *est.* | M2 |
| 4 | **Visibility queries for shadow rays** (terminate on first hit, alpha loop, no hit decoding); visibility reuse later | same picture, plus coloured light through stained glass | negative | M2 |
| 5 | **Cache guards:** short ray = no cache read; reset cells round a changed light; 100x-mean clamp; roughness floor 0.4; cone rule for glossy reads | no leaks at walls past 24 blocks; torches show at once | about 0 | M2 finish |
| 6 | **Denoiser: specular by hit distance, fast-history clamp, then 1/5-res confidence; firefly clamp** | reflections stay put; light changes without trails; no sparkles | about 0.3-1 ms *est.* | M3 (specular) / M4 |
| 7 | **RR by GPU**: Low and Medium (30-series) use ours, RR offered with its cost (about 6 ms on a 3060); High and Max on 40/50 use RR F; RR rules checked (linear albedo, white noise in `rt_cone`) | the laptop stays playable; RR at its best where it's cheap | about 0 | W2 / M4 |
| 8 | **Low ladder for the 6 GB laptop**: `RECORDS_BYTES` by view distance, 50 % scale, reach 192, smooth-only reflections | fits in 6 GB at playable rates | about 0 | W2 |
| 9 | **Soft block lights**: a point on the emitter's face or a sphere per torch; emissive faces merged into quads; per-cube light visibility statistics (Quake II RTX) | soft torch shadows as Remix shows; fewer wasted candidates in caves | small | M2 / P |
| 10 | **Opacity micromaps and SER** (raw Vulkan: `VK_EXT_opacity_micromap`; SER needs the tracer as an RT pipeline) | none in picture; -20-55 % trace on Ada and Blackwell, nothing on the 3060 | big (OMM build from block textures; a pipeline port) | M5 |
| 11 | ReSTIR GI on the first bounce, radiance from the cache | sharper bounce light, less lag | about 1-2 ms *est.*, complex | M5 if the cache lags |
| 12 | Keep triangle BLASes; an AABB-brick path only as a measured A/B if foliage or BLAS memory forces it | none known | big | none (on hold) |

Measure first (W2 and the PC): rays per second for primary and one-bounce diffuse in a fixed
scene, and each pass's ms in `/videocosts`. No published number replaces our own.

## Not verified

- Minecraft RTX's BLAS layout and GI method (the GTC 2020 talk is behind a login).
- Any 3060 Laptop path-tracing frame time, and RR on laptop parts.
- Cyberpunk's exact cache and NRC.
- Alan Wake 2's internals.
- NAADF's GPU figures.
- Whether opacity micromaps run on the 3060 (Ampere has no OMM hardware).
- How `fs_pt_guides` encodes the albedo for RR.
