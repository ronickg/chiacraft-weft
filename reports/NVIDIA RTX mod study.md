# NVIDIA's RTX mod for Minecraft: how it works, and how we'd do the same

Study for chiacraft's ultra look on the PC (RTX 5090). Started 2026-11-05 from the mod's and the
runtime's public source, read on GitHub. The one PC try (2026-11-06, under Wine) didn't start the
renderer, and Ronald chose to skip a live test on Windows, so there are no shots or frame times of
our own: see [The PC test](#the-pc-test). What we take from it, ranked: [What we take](#what-we-take-ranked-by-look-gain-per-cost). Study only: none of its code, shaders, configs or art go into our
repository or game. We learn how it's built, then write our own.

## What it is

- **Name**: *RTX Remix for Minecraft*, version 0.1.0 (beta), released by NVIDIA on 2026-10-23 as
  part of RTX Remix 1.6 ([NVIDIA's page][nv]).
- **For**: Minecraft Java 1.21.1, Fabric (loader 0.16.5+, Fabric API 0.116.10+), Java 21.
- **Source**: [NVIDIAGameWorks/rtx-remix-minecraft][repo], Apache-2.0. It's the Java mod plus a
  small JNI bridge (`rtxremix-java/`) over the RTX Remix C API.
- **The renderer**: the RTX Remix runtime, [NVIDIAGameWorks/dxvk-remix][dxvk] (zlib licence).
  It's a fork of DXVK, which normally turns Direct3D 9 games into Vulkan. Here it's used through
  its C API (`remix_c.h`), not as a D3D9 wrapper. The release ships it prebuilt (pinned by
  `packman/remix-sdk.packman.xml`), together with closed NVIDIA parts: DLSS (Super Resolution,
  Ray Reconstruction, Frame Generation), NRD, the Neural Radiance Cache (`NRC_Vulkan.dll`, built on
  tiny-cuda-nn) and SHaRC.
- **Runs on**: Windows 10/11 only, on GeForce RTX 30 or newer, driver 616.92+. The bridge loads
  the runtime's `d3d9.dll`, so it needs Windows Java. On Linux, people run the whole Windows game
  under Proton: the world comes out upside down with no UI unless the GL memory import is marked
  dedicated (a community mixin, issue [#41][i41]); plain Wine does not start the renderer at all
  (our try, below).
- **Extras**: the *RTX Remix Materials* pack (LabPBR normal and specular maps, plus their own
  `_sss` subsurface maps) and the *RTX Remix Showcase World* by BlockWorks, which the renderer
  was tuned for. Both are on CurseForge and inside the release zip.

### What the release really ships (checked 11-05)

- The release zip (1.03 GB, sha256 `0a4b84fc…01fa`) holds only the installer and the mod jar.
  The materials pack (0.58.0) and the showcase world (v0.11.1) are separate downloads on
  CurseForge.
- The jar's 312 classes match the source at the release commit `170197a` (main has 4 classes
  more since). Inside it, `native/remix-sdk-win64` and `-arm64` carry the runtime:
  `d3d9.dll` (309 MB, the shaders built in), `nvngx_dlssd.dll` (Ray Reconstruction),
  `nvngx_dlss.dll`, `nvngx_dlssg.dll` (frame generation), `nvngx_dlssnr.dll` (266 MB, DLSS 5's
  neural rendering), `NRD.dll`, `NRC_Vulkan.dll`, `libxess.dll`, NVRTC and USD libraries, and
  `remixbridge.dll` (the JNI bridge).
- **The runtime is not the public dxvk-remix.** It reports `remix-main+71a581da`, built from
  NVIDIA's internal tree; that commit isn't in the public repository. Its C API is version 0.16,
  the public header's is 0.6. Retained scene instances (`SceneInstanceAdd`), the GL interop
  (`dxvk_GetInteropInfo`), config layers and texture updates are only in the private build, and
  so are ten of the options the mod sets: `gpuResidentScene`,
  `materials.enableUvBoundsLooping` (block textures in an atlas), `neuralRadianceCache.
  wrapPositionsToSceneBounds`, `sparseRendering.enableRtxdiReuseForInactivePixels` and others
  (their names are in the shipped `d3d9.dll`, not in the public source). So the public source
  shows the path tracer's passes, but not the Minecraft-specific runtime work.

## The big picture

Minecraft keeps running as usual: its game logic, its chunk meshing workers, its OpenGL UI. The
mod takes the 3D world away from OpenGL and gives it to Remix, which path-traces it in Vulkan.

```
Minecraft (Java, OpenGL)                       RTX Remix runtime (C++, Vulkan)
──────────────────────────                     ────────────────────────────────
section meshes from vanilla's workers  ──►  mesh pages ──► one BLAS each (built once)
entities, block entities, particles    ──►  instances (retained, or per frame) ─► TLAS
atlas sprites + LabPBR maps            ──►  textures + materials
block lights (rules per block state)   ──►  lights (sphere/rect/disk), drawn per frame
sky dome (GL compute, Rayleigh+Mie)    ──►  dome light
camera                                 ──►  SetupCamera ─► path trace ─► image in Vulkan
GL draws the UI onto that image  ◄──  shared image (EXT_memory_object + EXT_semaphore)
                                      ──►  Present
```

Everything goes over about 30 C calls: `CreateMesh`, `CreateMaterial`, `CreateTextureFromMemory`,
`UpdateTextureFromMemory`, `SceneInstanceAdd/Update/Remove` (instances kept from frame to frame),
`DrawInstance` (one frame only), `CreateLight` / `DrawLightInstance`, `SetupCamera`,
`SetConfigVariable` and the config layers, `Submit`, `Present`, and a few interop calls
(`dxvk_GetInteropInfo`, `dxvk_GetVkImage`).

## The Java side: what the mod feeds the renderer

About 75 mixins hook vanilla. The parts that matter for us:

**Terrain.** Vanilla's chunk workers still build section meshes. A low-priority executor merges a
whole chunk column into one asset, split by kind into four categories (solid, cutout with
alpha test, translucent, overlay decals) and into pages capped in bytes. Each page is one Remix
mesh, and with `rtx.minimizeBlasMerging` each mesh gets **its own BLAS, built once and never
rebuilt** (without it, small meshes are merged into one BLAS that is rebuilt every frame). Pages
sit in the scene as retained instances; moving the camera never changes who owns what. Changed
columns are found by a fingerprint per category, so only the changed category is rebuilt. Uploads
go through a per-frame budget on the render thread, with up to 256 MB of meshes pending.
Opacity micromaps are off ("Minecraft's high-volume terrain workload").

**No far terrain.** There is no LOD. The scene is what Minecraft loaded (32 chunks at most, 512
blocks), and the renderer fades terrain into the sky over 70 to 80% of the render distance
(`rtx.viewDistance.*`, a stable world-space noise for rays after the first).

**Entities and particles.** Entities vanilla already draws are captured in place, so each model
is posed once. Entities outside the view are still swept up, nearest first under a budget,
because a path tracer needs them for shadows and reflections. Particles are built on a worker,
one frame behind the scene. First-person arms and held items go through Remix's separate
view-model camera.

**Materials.** Every atlas sprite gets its own material, and the atlas gets PBR, emissive and
luminance textures. LabPBR maps are converted (`LabPBRConverter`). Animated sprites are
forwarded as texture updates. Without the materials pack, vanilla textures get simple built-in
materials. Biome tints travel as vertex colour.

**Lights.** Block lights come from resource-pack rules keyed by block state, and each one is
drawn again every frame, so not drawing one culls it. Culling: the chunk-column box, then a flood
through connected sections, then a reach test by the light's power (brighter lights reach
further). Lanterns and sea pickles glow through their surface instead of having a separate light.
The new Copper Beacon is a disk light that really lights rain and walls.

**Sky and clouds.** The sky is a Rayleigh and Mie panorama made by GL compute shaders from the
time of day, read back and published as a dome light; rain blends an overcast dome over it. The
sun's path is tilted, "matching popular shader packs (BSL, Complementary)". Clouds are real
meshes: marching cubes over a density field, the same on every client from the world time.

**Looks by situation.** Remix config layers fade in and out: dawn, day, dusk, night, rain,
thunder, under water, caves, biomes, and zones a map author places. They mostly change exposure,
bloom and fog. A few numbers worth knowing: `rtx.sceneScale = 0.01` (Remix's fog grid was made for
games measured in centimetres), fog to 256 blocks, auto exposure between -2.5 and +1.5 EV, a high
bloom threshold by day and a low one at night, and local tonemapping.

**Engineering around Java.** Much of the work is keeping Minecraft from stalling the renderer:
chunk delivery and unloads are paced by measured render-thread cost, hot vanilla paths are made
allocation-free (the collector's pauses stretched frames), and section meshes live in C-allocated
buffers outside the JDK's limit.

## The GPU side: what Remix does each frame

From `RtxContext::injectRTX` and `dispatchPathTracing` in dxvk-remix, in order:

1. **Volumetrics**: a froxel grid (camera-aligned 3D cells) lit by its own light sampling. God
   rays, haze, underwater fog.
2. **Sparse rendering**: picks which pixels get traced this frame (Minecraft sets quality 0.3) and
   packs them together. It only runs with DLSS Ray Reconstruction and the neural cache on.
3. **G-buffer by ray tracing**: primary rays instead of rasterising. *PSR* (primary surface
   replacement) follows perfect mirrors and glass: the surface seen in a mirror or through water
   becomes the pixel's surface, so reflections and refractions get full lighting and denoising.
4. **RTXDI** (ReSTIR for direct light): every pixel picks a few of thousands of lights and reuses
   good picks from its neighbours and from the last frame, then traces one shadow ray.
5. **NEE cache**: a world-space cache of which lights matter where, for the bounced rays.
6. **Integrate direct and indirect**: shade the first hit, then the indirect path (up to about
   four bounces, NVIDIA says). Paths end early in a radiance cache: **NRC** (a small neural
   network trained on the fly, CUDA) or **SHaRC** (a hash grid in world space).
7. **ReSTIR GI**: reuses indirect light samples between pixels and frames.
8. **Demodulate, then denoise**: divide out the albedo, denoise (NRD's ReBLUR/ReLAX, or let DLSS-RR
   do it all), then put the albedo back in **composite**.
9. **Upscale**: DLSS-RR (denoise and upscale in one network), or DLSS, XeSS, NIS, TAA-U.
10. **Post**: bloom, motion blur, auto exposure, tonemapping (ACES, local tonemap), lens
    effects, sRGB with dither, then DLSS frame generation and the blit to the game's image.

The shaders are in Slang. Remix has both ray-query compute versions and ray-tracing-pipeline
versions (raygen, closest-hit, miss) of its passes, and can use Shader Execution Reordering on
RTX 40 and newer.

Two details that matter for blocks:
- **See-through texels in ray queries** (`algorithm/visibility.slangh`). Cutout and translucent
  triangles go into the BLAS as non-opaque, so the GPU hands each one back to the shader as a
  candidate. For a shadow ray, the shader looks up the texel, multiplies the ray's colour by what
  that texel lets through, and keeps going; only when almost nothing gets through does it accept
  the hit. That one loop gives leaves their dappled shadows and stained glass its coloured ones.
- **Primary rays through glass and water** (`algorithm/resolve.slangh`): up to 128 see-through
  hits per primary ray (32 for later bounces), sorted into four distance bins and blended, so
  layered glass and particles look right without sorting meshes.

So the mod is **full path tracing**, not hybrid: no rasterised world at all. It leans on three
NVIDIA-only parts to be fast enough: DLSS-RR (denoising and upscaling), NRC (bounce light) and
frame generation.

## What it costs

- The release is 1.03 GB, and about 3 GB stays installed. 16 GB of RAM is recommended.
- No measured frame times have been published yet (searched 11-05). Issue #39 reports 16 fps
  windowed against 52 fps full screen at the same size on one machine.
- The CPU side is a large part of the work: much of the mod is pacing, budgets and
  allocation-free rewrites to keep Java from stalling the renderer.
- We have no measurements of our own: the Wine try didn't render, and the live test was skipped
  (11-06).

## What looks best (from the source; not seen in a run)

- Coloured light through glass and water, and coloured shadows from them (`enableDirect/Indirect
  TranslucentShadows`).
- Real reflections and refractions on water and glass through PSR, lit like the rest of the scene.
- Bounce light: the colour of a lit wall on its neighbours, and dark corners in caves without
  ambient occlusion tricks.
- Many lights that all cast shadows (torches, lava, the beacon), thanks to RTXDI.
- Volumetric light through the fog and under water.
- Clouds as volume-like meshes that cast shadows.

Weak spots it admits: nothing past the render distance, other mods' rendering breaks, and things
the showcase world never visits have had less care.

## How we could do the same in Weft

**Where we stand.** Our engine has its own renderer: `weft_render` (a backend trait) and
`backends/vulkan` (Vulkan 1.3 through ash 0.38, dynamic rendering, bindless textures, push
descriptors). Shaders are WGSL, turned into SPIR-V by naga 30. It rasterises; shadows come from
shadow maps (near and far cascades) and, in packs, a voxel march through `solid_at` (blocks
about 112 around the camera). Far land comes from GPU-made LOD tiles (`client/lod.rs`). Nothing
uses ray tracing yet.

**What helps us.**
- naga 30 already reads ray queries in WGSL (`enable wgpu_ray_query;`, the
  `acceleration_structure` and `ray_query` types) and writes them to SPIR-V. So a pack's
  `world.wgsl` or a compute pass could trace rays without leaving WGSL. It has no full
  ray-tracing pipeline in WGSL yet, so we'd build on **ray queries**, inline in compute and
  fragment shaders. That's also simpler to fit into our backend.
- Our world is voxels, and our meshes are already built per chunk. One BLAS per chunk section
  (opaque, cutout and translucent as separate geometries), built when the section is meshed and
  kept until it changes, is exactly the scheme Remix ended up with (`minimizeBlasMerging`).
  The TLAS (a few thousand instances) is rebuilt every frame and costs little.
  Remix builds every BLAS with `PREFER_FAST_BUILD | ALLOW_UPDATE` (`rtx_accel_manager.cpp`),
  because the D3D9 games it was made for stream geometry each frame. Our sections change
  rarely, so ours should be `PREFER_FAST_TRACE | ALLOW_COMPACTION`, compacted after the build.
- We already have far terrain, which Remix doesn't. LOD tiles can get coarse BLASes of their own,
  so far hills cast shadows and show in reflections.

**Which GPUs.** Ray tracing is an ultra setting, off unless the device has
`VK_KHR_acceleration_structure` and `VK_KHR_ray_query`:
- the PC (RTX 5090), and any RTX 20 or newer or RDNA2 or newer card: yes;
- the Mac: no. MoltenVK doesn't expose Vulkan ray tracing, so the Mac keeps today's look;
- Android: some new phones expose ray queries, but they're far too slow for this. Off.
The look without ray tracing stays the reference, and every ray-traced step must look like a
better version of it, not a different game.

**Steps, each its own branch, each benched and shot on the PC.**

1. **Ray tracing in the backend.** Turn on the extensions and buffer device address when
   present; add BLAS and TLAS (create, build, compact, update) to the `Backend` trait; put the
   TLAS into the world's bindings; a capability flag the settings read. Check: a debug view
   showing hit distance per pixel.
2. **Scene upload.** A BLAS per meshed section (and per LOD tile), the TLAS each frame, models
   and held items as instances. Materials as a bindless table: albedo, our normal and specular
   maps, emission. Cutout leaves alpha-tested in the ray query loop. Check: build time per frame
   and VRAM, at 16 and 32 chunks.
3. **Ray-traced sun and moon shadows.** One ray per pixel toward the sun, a little jittered for
   soft edges, then a small temporal and spatial filter. It replaces the shadow maps on ultra and
   reaches as far as the LOD tiles. This is the cheapest big win, and a pack can call it the way
   it calls `solid_at` now.
4. **Reflections on water, glass and polished blocks.** One ray from the surface, shaded with our
   own world shading at the hit and the sky where it misses, denoised by roughness. It replaces
   screen-space reflections, so things off screen show too.
5. **Bounce light.** One diffuse ray per pixel (or per two), ending in a world-space hash-grid
   radiance cache we write ourselves (the idea behind SHaRC, published by NVIDIA), with a
   temporal and an à-trous spatial denoiser of our own. Then caves and rooms get coloured bounce
   light without ambient tricks.
6. **Many lights.** Our own ReSTIR for block lights (the papers are public: Bitterli et al.
   2020, and NVIDIA's RTXDI notes), so every torch casts shadows. Only after 3 to 5 look right.
7. **Denoising and upscaling with DLSS-RR**: optional. It's NVIDIA's closed SDK (it has Linux and
   Vulkan builds), with its own licence terms and attribution, and it's what makes Remix's sparse
   tracing work. Before we use it, Ronald decides on the licence. Our own denoisers from steps 3
   to 5 come first either way.

**A first sketch of the backend side** (our own design, to be refined on its branch):

```rust
// weft_render: new, only when Backend::ray_tracing() is true
pub struct BlasDesc<'a> {
    pub vertices: BufferId, pub vertex_stride: u32, pub vertex_count: u32, // positions as f32x3
    pub indices: BufferId, pub index_count: u32,
    pub opaque: bool, // false: cutout/translucent, hands candidates to the shader
}
fn create_blas(&mut self, geometries: &[BlasDesc], label: &str) -> BlasId; // fast trace + compaction
fn destroy_blas(&mut self, blas: BlasId);
// Per frame: the instances (BLAS, 3x4 transform, custom index for the material table, mask).
fn build_tlas(&mut self, tlas: TlasId, instances: &[TlasInstance]); // recorded as a Cmd before the passes
```

WGSL then gets `@group(0) @binding(N) var scene_tlas: acceleration_structure;` in the world
prelude, and a helper in `common.wgsl`, such as `sun_visible(p, n) -> vec3<f32>`
(colour let through), written once with the see-through loop above. A pack calls it where it
calls `solid_at` now. Our chunk meshes have no float vertex buffer: a section is a page of
packed face records (four words a face, `gpu_layout::face`, free quads eight words) that
`vs_face` and `vs_quad` turn into corners. So a small compute pass expands a section's records
into float positions (four corners a face, one shared index pattern for every quad) in a
scratch buffer, the BLAS is built from that, and the scratch is reused; the record's index is
the primitive index, so a hit looks its face record up again for texture, tint and light. Remix
does the same kind of expansion for its games (`pass/interleave_geometry.comp.slang`).

We don't copy Remix's full path tracer. A hybrid (rasterised first hit, rays for shadows,
reflections and bounce light) fits our look and our shader packs, costs far less than a full
path tracer, and lets the Mac keep the same picture without the rays.

## What we take, ranked by look gain per cost

Steps 1 and 2 (ray tracing in the backend, scene upload) come first and show nothing on their own;
everything below needs them. Then, best gain for the cost first:

1. **Ray-traced sun and moon shadows** (step 3). One ray a pixel and a small denoiser. It gives
   sharp-near, soft-far shadows with no shadow-map texels, acne or swimming, reaching as far as the
   LOD tiles. With the see-through loop (multiplying through glass, leaves and water) it also gives
   coloured shadows. It replaces a pass we already pay for, so the net cost is small.
2. **Reflections on water and glass** (step 4). One ray, only on reflective pixels, so it's cheap
   on land and big on lakes and windows. Things off screen and behind the camera show, which no
   screen-space trick does. The cost is shading at the hit: reuse our world shading with a cheaper
   light model.
3. **Bounce light** (step 5). The biggest change of picture (coloured light off walls, dark caves
   without ambient tricks), and the costliest: a ray a pixel or less, our own hash-grid radiance
   cache, and a temporal and spatial denoiser. Most of the risk is in the denoiser, so it gets its
   own branch and many looks.
4. **Shadowed block lights, ReSTIR** (step 6). Every torch casts shadows. It's high in both cost and
   work, and only worth it once 1 to 3 look right; nights stay dark (our rule), so it shows mostly
   in caves and builds.
5. **DLSS-RR** (step 7). It needs Ronald's licence decision, NVIDIA cards only. Our own denoisers
   come first.

Ideas worth trying without rays, for every GPU:
- **Coloured shadows through stained glass** in today's shadow map: a second, coloured map of
  translucent casters, as Minecraft packs do.
- **Fade far terrain into the sky** where the world ends; Remix fades from 70 to 80% of its view
  distance. We have LOD instead, but the same fade fits the LOD's far edge.
- **A tilted sun path**, as in Remix's sky and the BSL and Complementary packs: lower noon light and
  longer shadows. That's a look choice for Ronald.

## The PC test

**First try, 2026-11-06, plain Wine: the renderer does not start.** Windows Prism Launcher 11.1.1
and Microsoft OpenJDK 21.0.13 in a Wine 11.21 prefix on the PC (CachyOS, RTX 5090, driver
615.78.08), Ronald's account, MC 1.21.1 + Fabric 0.16.5 + the mod's release jar + Fabric API,
materials pack on, on a rootful Xwayland at 2560×1440. The jar loads, the Remix API starts
(runtime log: Vulkan device up with `VK_KHR_external_semaphore_win32`), then:

- **GL interop fails**: `glImportSemaphoreWin32HandleEXT` on Remix's "dxvk done" semaphore gives
  `GL_INVALID_VALUE` (0x501); the interop info also comes back with no image handle and a 0×0 size.
  Wine's OpenGL doesn't hand the Win32 handles of its own Vulkan objects to the host GL driver.
  The mod then shuts Remix down and Minecraft runs in plain OpenGL (title screen in the gallery).
- **No DLSS**: NGX reports "not available on this hardware/platform" for every feature. Plain Wine
  has no `nvngx`/NVAPI path, so DLSS-RR and NRC, which the defaults lean on (sparse rendering at
  0.3 needs DLSS-RR), would be missing even if the interop worked.

Logs: `chiacraft-archive/references/rtx-mod/run-logs/2026-11-06/` (`latest.log`,
`remix-dxvk.log`); shot: `chiacraft-archive/images/rtx-mod-study/2026-11-06/fail1.png`.

**Skipped** (Ronald, 11-06: "Skip the live test"). Seeing it render would need native Windows on the
PC (it dual-boots), with Ronald at it, or Proton plus a third-party fix mixin, which this study
doesn't run (downloads only from the mod's official source). If it's ever wanted, the scene script is
in
`chiacraft-archive/tester-tools/rtx-mod/rtx_scenes.txt`:
- the MC comparison's vista, forest, water and snow places, with `/rtx off` A/B pairs;
- close-ups of a pool and metal blocks, coloured glass, wall and overhang shadows, and a white room
  with a red wall, by day and by torch;
- frame times at 1440p from F3 and `nvidia-smi`, on each `/rtx preset`.

The study's conclusions about how it works and what we'd build don't depend on this run; the run would
add how it looks next to the MC shader packs and what it costs on a 5090.

[nv]: https://www.nvidia.com/en-us/geforce/news/remix-1-6-dlss-5-comfyui-2-0-minecraft-rtx-beta/
[repo]: https://github.com/NVIDIAGameWorks/rtx-remix-minecraft
[dxvk]: https://github.com/NVIDIAGameWorks/dxvk-remix
[i41]: https://github.com/NVIDIAGameWorks/rtx-remix-minecraft/issues/41
