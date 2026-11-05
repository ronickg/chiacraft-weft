# Research round 19: what could make the Mac 10× faster, and what can't

Researcher, 2026-11-05, for Planner, Lighting and Distant terrain. Ronald asked: "Find some more articles or thesis or stuff on this topic which could potentially like 100x stuff still? Without really losing quality". The topic is our speed: far terrain and rendering, above all on his Mac (M3 Max, MoltenVK, 3456×2234).

**What was read:**
- Four sub-agents read papers, theses, SIGGRAPH/HPG/GDC talks, Apple's WWDC sessions, and the MoltenVK source and issues, as text. They covered voxel and far-terrain rendering, GPU terrain generation, temporal reuse and upscaling, and Apple GPU specifics.
- I checked their claims that touch our code against the code (dev `d21a7627`).
- Sources are listed by section at the end (about 60 links).

**Where the frame stands:** Ronald's Mac run (dev `10e54a0c`, 3456×2234 at the auto scale of 0.67, creative flight):
- **GPU:** 16.2 ms of a 16.7 ms frame. The world pass took 7.6 ms and "effects" 6.8 ms (that part also holds the water copy, sky, clouds and water: round 16, M1).
- **Far terrain:** 15–22 ms of CPU heights plus 5.5–16 ms of meshing per tile. The far view is full at 11 s, and a 3,000-block teleport fills in about 9 s.
- **Already done since round 16:**
  - half-size shafts and volumetric light (`post_low`, 404bae57);
  - the far shadow map anchored to a 32-block grid (7942bde7);
  - mist down to 8 steps;
  - contact shadows off by default and shafts on Low;
  - far tiles requested coarse first, on a pool of their own (lod-loading).

## The short answer

1. **No single technique gives 100× here without losing the look.** Every 100×-class number found is a narrow case: memory not speed, or a baseline far worse than ours, or a kind of geometry we don't have (the table "Which big numbers hold"). What the sources support for our frame is about 2–3× on the Mac's GPU time from stacking four or five changes, plus about 10× on how fast far terrain fills in.
2. **The biggest Mac win may be a bug-sized one in our own code.**
   - Since the Weft sync, the cubes are culled on the GPU and drawn with count draws (`world_render.rs:1647`, `occlusion.wgsl`).
   - MoltenVK has no `drawIndirectCount`, so on the Mac every count draw issues all of its `max` records (`backends/vulkan/src/lib.rs:1855`, `world_render.rs:1756-1766`). `max` is every loaded cube's slots × 3: about 33,000–40,000 Metal draws a frame at view 12, nearly all of them empty.
   - That fits the Mac's world pass jumping from 2.3 ms (round 16's calibration) to 7.6 ms.
   - The culling already packs each bucket's visible records at its front (`occlusion.wgsl:101`). So drawing only last frame's count plus a margin would cut those draws to the visible few thousand.
   - Unmeasured: it needs a Mac row and a Metal capture first.
3. **The next biggest is temporal reuse, which we have none of:**
   - **An upscaler of our own (TAAU)** at 0.5 render scale instead of 0.67 with bilinear: Epic's TSR halves the frame at 4K, which shades 44% fewer pixels than now, with better edges than bilinear.
   - **Mist and light in the air marched in a froxel volume, accumulated over frames** (Frostbite 2015, AC4 2014): about 45× fewer samples than our per-pixel march.
   - **Clouds refreshed a sixteenth at a time** (Nubis; Toft et al. measured 40×, against a much worse baseline than ours).
4. **Apple's own rules give smaller sure wins:** draw opaque before anything that discards, don't split the world pass for the water copy, and specialise the merged effects shader (Apple measured 4.6× on an uber-shader, on other content).
5. **Far terrain:** no source gives 10× for graph-based rivers and lakes like ours on the GPU. A world-scale cache of the features per region would do it more cheaply: our 11 µs per column against Génevaux's about 1 µs points to redundant work. Then evaluate columns on the GPU from that cache. Expect about 3–10× on fill-in time, not on frames.

## Ranked: gain for the effort, weighted toward the Mac's GPU time

Gains are Mac ms out of today's 16.2, from the sources' multipliers applied to our measured parts. Every one needs a Mac row to confirm. "Agent hours" are for one worker, writing and checking on the PC.
- **Engine** means `crates/` (and an ENGINE_CHANGES line).
- **Pack** means our shader pack and `mods/base`.
- Every row works through MoltenVK unless it says otherwise.

| # | change | the source's number, and on what | expected on the Mac | risk to the look | where | agent hours |
|---|---|---|---|---|---|---|
| 1 | **Mac count draws: issue only what was drawn.** Read the bucket counts back 2 frames late (they are atomics already) and pass min(max, count × 1.25 + 64) as the draw count. Records past the count are zero, so a few extra are free. Alternatively, try MoltenVK main, which now advertises `drawIndirectCount` (but implements it with a compute encoder that ends and restarts the render pass). | no paper: MoltenVK encodes each indirect record as a Metal draw on the CPU (`MVKCmdDraw.mm`); ours are about 35k a frame on the Mac | up to most of the 7.6 → 2–3 ms world-pass gap, plus CPU time (**measure first**) | a frame of missing cubes when the count jumps (a fast turn): cover with the margin, or draw everything for a frame after a big camera turn | `crates/engine/src/client/world_render.rs:1640-1660, 1756`, `backends/vulkan/src/lib.rs:1855` | 4–8 |
| 2 | **Our own temporal upscaler (TAAU)** with Karis-style neighbourhood clipping: jittered projection, a velocity buffer, history. The world at 0.5 instead of 0.67. | TSR: 4K native 57.5 → 1080p→4K 33.4 ms ("half"); FSR 2 Performance shades 25% of pixels at 0.6–2.3 ms | 44% fewer pixels for all per-pixel work: about 4–6 ms of the 14.4 in world plus effects, less the TAAU's own 0.5–1 ms | ghosting on fast-moving things, softness in motion; better edges than today's bilinear | engine `post.rs`, `app.rs` (`world_frame`), the client camera's jitter; pack (velocity from the world shaders) | 16–24 |
| 3 | **Mist and the light in the air in a froxel volume** (about 160×90×64), marched once a frame with jitter and 5% accumulated (Frostbite); fog then reads one 3D tap. First step: mist at half size on the `post_low` hook with jitter and history (Photon does this). | Frostbite: the whole volumetric system 0.83–2.95 ms on PS4; 0.92M cells against our 3.47 Mpx × 8 steps ≈ 28M samples | mist's +7.6 ms (now with 8 steps, perhaps +5) → about 1 ms; volumetric light and shafts share it | trails behind moving light (the sun moves slowly); froxel blur near the eye | pack `look.wgsl` `fog_through` (651 on), `post/volume.wgsl`; engine for a 3D texture and its pass | 8–12 (half size), 24–40 (froxels) |
| 4 | **Draw opaque first, then what discards, then frag_depth, then translucent**: the fade band's meshes drawn with discard only where they are in the band (about 40% of near meshes use it, `world_render.rs` fade pipes), cutout leaves after all opaque, clouds without `frag_depth` if possible | Apple: discard and depth writes make "feedback fragments" that skip HSR; opaque before alpha-tested (WWDC20 10632, 10602); no number | unknown, possibly large: the world pass is 3× the PC's share on the Mac | none (order only) | engine `world_render.rs` (bucket order, fade pipes), pack `world.wgsl:763-800`, `sky.wgsl:110` | 4–8 |
| 5 | **Specialise the merged effects shader**: a pipeline per set of active effects through WGSL `override`/spec constants instead of a switch in one uber-shader | Apple: a material pass 58 → 12.5 ms (4.6×) on pre-M3 hardware, Blender content (WWDC23 10127); M3's Dynamic Caching shrinks the gap | part of the effects pass: perhaps 0.5–2 ms (**measure**) | none | engine `post.rs`, `assets/shaders/post.wgsl` (the `post_apply` switch) | 4–8 |
| 6 | **The water copy without splitting the world pass**: copy before the world pass, or end it once and draw water and clouds after (Apple: "batch copies before rendering") | Apple: five round trips to one flush (WWDC23 10125); our split is about 166 MB a frame of stores and loads (my arithmetic) | about 0.4–1 ms | water behind water may differ; clouds lose the copied depth (pair with a depth test) | engine `app.rs:3050-3090`, `post.rs` `copy_behind`; pack `sky.wgsl:378, 492` | 8–12 |
| 7 | **Clouds refreshed a quarter or a sixteenth of the pixels a frame**, the rest reprojected (needs #2's history) | Toft et al. 2016: 297.7 ms (full, 128 steps) → 2.4 ms (quarter, 8 steps, TAA), about 40×; Nubis "under 2 ms" | clouds' share (unmeasured, in "effects"): perhaps 0.5–1.5 ms | smearing of fast clouds | pack `sky.wgsl` `fs_clouds`; engine history buffer | 8–16 after #2 |
| 8 | **A sky-view LUT** (192×108) and an aerial-perspective volume (32×32×32) instead of sky and haze per pixel | Hillaire 2020: sky-view 0.27 ms on an iPhone 6s (secondhand) | 0.2–0.5 ms, and haze cheaper per fragment | none if matched | pack `look.wgsl` `sky_color`, `haze_lit` | 8–12 |
| 9 | **Far features cached per region at world scale**: river and lake segments, lake levels, mesa masks, made once and shared by every far tile. Profile first: 11 µs a column against Génevaux's about 1 µs. Then columns on the GPU from that cache, in the existing LOD compute pass, with integer and fixed-point decisions so the GPU and CPU agree. | Génevaux 2013: about 1 s per 1024² samples on one CPU core; GPU noise 2.4–4.7 G samples/s (GTX 580); FastNoise2 SIMD about 6.5× | fill-in time 11 s → about 2–4 s first, more with GPU columns (CPU, not frame time) | none if exact (whole-block heights) | base mod `land/net.rs` (`region_at`), engine `client/lod.rs` `custom_heights`, `terrain_gpu.rs` | 8–16 (cache), 40–80 (GPU columns) |
| 10 | **Fewer, smaller draws for near cubes**: 8-byte faces (from 16); later a compute pass that writes visible faces into one buffer drawn in a few draws (Meshelium-style); mesh shaders once MoltenVK merges `VK_EXT_mesh_shader` (#2842, open) | Meshelium: 1.5× at 12 chunks, 5.3× at 64 (RX 9070 XT, the developer's own figures); on M1, mesh shaders measured slower than draws (Tellusim, an Apple forum thread) | half the face bandwidth now; the draw part is mostly #1 | none | engine `mesher.rs`, `gpu_layout.rs`, `world.wgsl` vertex pulling | 12–20 (8-byte faces) |

**Do first:** #1 (a small change, possibly the largest), #4 (order only), then #2, which #3 and #7 need, and #5. Before any of them: one Mac row with `MVK_CONFIG_PERFORMANCE_TRACKING=1` or a Metal frame capture, to split the world pass's 7.6 ms. It needs Ronald's yes for a Mac run.

Not reachable through MoltenVK today:
- **Metal's variable-rate shading** (rasterisation rate maps): MoltenVK doesn't map `VK_KHR_fragment_shading_rate` (issue #1710, open).
- **MetalFX upscaling**: possible in theory through `VK_EXT_metal_objects`, but nobody has done it, and M1 forum anecdotes put its cost at 4–10 ms.
- **Metal's indirect command buffers**: MoltenVK never uses them.
- **Mesh shaders**: PR #2842 is still open.

A native Metal backend would get these, but going native measured only 16–24% faster in Dolphin and Godot: not a multiple.

## Which big numbers hold, and where

| claim | what it is really | for us |
|---|---|---|
| SVDAG: 28–576× fewer nodes, 38× less memory (Kämpe 2013) | memory against pointer octrees; ray speed is about equal to triangle ray tracing | our far terrain is coarse boxes: no gain in speed |
| Ray boxes: 20× over rasterised meshes (Majercik 2018) | only for individually oriented voxels that may all change every frame | not our blocks |
| Ray-casting heightfields 2× (Dick 2009) | on rough 1 m terrain; on smooth terrain the rasteriser was several times faster, and cost scales with pixels | our far land is smooth boxes at 3.5 Mpx: rasterising wins |
| Nanite's software rasteriser 3–4× | only for sub-pixel triangles | our LOD tiles are already coarse |
| Visibility buffer +115% | only with 2–8× MSAA on bandwidth-limited GPUs; "little to no net benefit" at one sample (Burns & Hunt 2013); Apple's HSR already shades opaque once | skip |
| Clouds 40× (Toft 2016) | from 128 steps at full resolution; we already take 10 | about 2–5× for our clouds |
| Froxel fog about 45× fewer samples | sample count against a per-pixel march; a froxel costs more to light than one march step | about 5× for mist in practice |
| Uber-shader 4.6× (Apple) | a Blender material pass on pre-M3 hardware | smaller for a switch over five effects on M3 |
| Mesh shaders 5.3× (Meshelium) | 64 chunks, draw-bound, AMD | on Apple M1 they measured slower than draws |
| FastFlow 26–52× (Jain 2024) | flow routing and erosion simulations against CPU code | would change the land: not for our graph rivers |
| MetalFX "half the time" | a diagram, no numbers; M1 anecdotes of 4–10 ms per upscale | unproven through MoltenVK |

Stacking the realistic ones: #1 (perhaps −4 ms), #2 (−4 to −6), #3 in mist (−4), #4–#6 (−1 to −3). Together that takes the Mac from 16.2 ms toward 6–8 ms in clear air and keeps mist from doubling it: about 2–2.5× on the GPU, not 100×. On far terrain, #9 is the one change with a real 10× in reach, on how fast it appears.

## Sources

**Voxel and far-terrain rendering:**
- Laine & Karras 2010, ESVO: https://www.nvidia.com/docs/IO/88889/laine2010i3d_paper.pdf
- Kämpe, Sintorn, Assarsson 2013, SVDAG: https://www.cse.chalmers.se/~uffe/HighResolutionSparseVoxelDAGs.pdf, https://research.chalmers.se/en/publication/182658
- Villanueva et al. 2016, SSVDAG: https://vicserver.crs4.it/vic/data/papers/i3d2016-symmetry-dags.pdf
- Dolonius et al. 2017, compressed colours: https://research.chalmers.se/en/publication/508251
- Careil, Billeter, Eisemann 2020, HashDAG: https://diglib.eg.org/handle/10.1111/cgf13916, https://github.com/Phyronnaz/HashDAG
- Aokana 2025: https://arxiv.org/html/2505.02017v1
- Majercik et al. 2018, ray-box: https://research.nvidia.com/labs/rtr/publication/majercik2018raybox
- Nousiainen 2019 thesis: https://core.ac.uk/download/227720821.pdf
- Dick, Krüger, Westermann 2009: https://www.cs.cit.tum.de/fileadmin/w00cfj/cg/Research/Publications/2009/GPU_Ray-Casting/EG09AreasTerrain.pdf
- hybrid raster and ray-cast terrain: https://www.iadisportal.org/digital-library/gpu-aware-hybrid-terrain-rendering
- Teardown frame teardown: https://acko.net/blog/teardown-frame-teardown/
- Nanite (secondary): https://trickybitsblog.github.io/2024/04/20/nanite.html, https://vrarwiki.com/wiki/Nanite
- meshoptimizer clusterlod: https://zeux.io/2025/09/30/billions-of-triangles-in-minutes/
- Bevy virtual geometry: https://jms55.github.io/posts/2024-11-14-virtual-geometry-bevy-0-15
- Burns & Hunt 2013, visibility buffer: https://jcgt.org/published/0002/02/04/paper.pdf
- MoltenVK mesh shader PR #2842: https://github.com/KhronosGroup/MoltenVK/pull/2842, issue #2375
- Meshelium: https://modrinth.com/mod/meshelium
- Tellusim compute raster: https://tellusim.com/compute-raster/
- Apple forum on mesh shaders on M1: https://developer.apple.com/forums/thread/722047

**Far terrain generation:**
- Jain et al. 2024, FastFlow: https://people.cs.uct.ac.za/~Jgain/wp-content/papercite-data/pdf/jain2024.pdf
- Génevaux et al. 2013: https://www.cs.purdue.edu/homes/bbenes/papers/Genevaux13ToG.pdf
- Cordonnier et al. 2018: https://www.cs.purdue.edu/homes/bbenes/papers/Cordonnier18TVCG.pdf
- Štava et al. 2008: https://www.cs.purdue.edu/homes/bbenes/papers/Stava08SCA.pdf
- Horizon Zero Dawn GPU placement (GDC 2017): https://www.guerrilla-games.com/media/News/Files/GDC2017_VanMuijden_GPUBasedProceduralPlacementInHorizonZeroDawn.pdf
- Minecraft density functions: https://minecraft.wiki/w/Density_function
- FastNoise2: https://github.com/Auburn/FastNoise2
- GPU simplex noise: https://arxiv.org/pdf/1204.1461
- Quilez on analytic noise derivatives: https://iquilezles.org/articles/morenoise
- WGSL floating-point primer: https://github.com/gpuweb/cts/blob/main/docs/fp_primer.md
- rust-gpu: https://rust-gpu.github.io

**Temporal reuse and upscaling:**
- Epic TSR: https://dev.epicgames.com/documentation/en-us/unreal-engine/temporal-super-resolution-in-unreal-engine
- AMD FSR 2: https://gpuopen.com/manuals/fidelityfx_sdk/techniques/super-resolution-temporal/
- Salvi, temporal supersampling (GDC 2016): https://developer.download.nvidia.com/gameworks/events/GDC2016/msalvi_temporal_supersampling.pdf
- Hillaire 2015, Frostbite volumetrics: https://www.slideshare.net/DICEStudio/physically-based-and-unified-volumetric-rendering-in-frostbite
- Wronski 2014, AC4 volumetric fog: https://www.scribd.com/doc/259208414/Assassin-s-Creed-4
- Nubis: https://www.guerrilla-games.com/read/nubis-evolved
- Toft et al. 2016: https://ar5iv.labs.arxiv.org/html/1609.05344
- Photon shader settings: https://github.com/sixthsurge/photon/blob/main/shaders/shaders.properties
- Battlefield 1 checkerboard: https://www.slideshare.net/slideshow/4k-checkerboard-in-battlefield-1-and-mass-effect-andromeda/72810615
- Metal rasterisation rates: https://developer.apple.com/documentation/metal/rendering-at-different-rasterization-rates.md
- MoltenVK issue #1710: https://github.com/KhronosGroup/MoltenVK/issues/1710
- MetalFX (WWDC22): https://developer.apple.com/videos/play/wwdc2022/10103/
- MetalFX cost anecdotes: https://developer.apple.com/forums/thread/728545
- GTAO: https://research.activision.com/publications/archives/practical-real-time-strategies-for-accurate-indirect-occlusion, https://github.com/GameTechDev/XeGTAO
- DDGI: https://research.nvidia.com/publication/2019-05_dynamic-diffuse-global-illumination-ray-traced-irradiance-fields

**Apple GPUs and MoltenVK:**
- WWDC20 10632: https://developer.apple.com/videos/play/wwdc2020/10632/
- WWDC20 10602: https://developer.apple.com/videos/play/wwdc2020/10602/
- WWDC23 10125: https://developer.apple.com/videos/play/wwdc2023/10125/
- WWDC23 10127: https://developer.apple.com/videos/play/wwdc2023/10127/
- WWDC19 601: https://developer.apple.com/videos/play/wwdc2019/601/
- Dynamic Caching tech talk: https://developer.apple.com/videos/play/tech-talks/111375/
- TBDR: https://developer.apple.com/documentation/metal/tailor-your-apps-for-apple-gpus-and-tile-based-deferred-rendering
- texture compression: https://developer.apple.com/documentation/metal/optimizing-texture-data, https://developer.apple.com/videos/play/tech-talks/10876
- MoltenVK draw commands: https://raw.githubusercontent.com/KhronosGroup/MoltenVK/main/MoltenVK/MoltenVK/Commands/MVKCmdDraw.mm
- MoltenVK device features: https://raw.githubusercontent.com/KhronosGroup/MoltenVK/main/MoltenVK/MoltenVK/GPUObjects/MVKDevice.mm
- MoltenVK pipeline barriers: https://raw.githubusercontent.com/KhronosGroup/MoltenVK/main/MoltenVK/MoltenVK/Commands/MVKCmdPipeline.mm
- MoltenVK release notes: https://raw.githubusercontent.com/KhronosGroup/MoltenVK/main/Docs/Whats_New.md
- Dolphin Metal against Vulkan: https://gist.github.com/dvessel/b060edb3e7e9ac242ee1e45ea0bb8b12
- Godot 4.4 Metal: https://godotengine.org/article/dev-snapshot-godot-4-4-dev-1/, https://github.com/godotengine/godot/pull/98212
- KosmicKrisp (XDC 2025): https://www.lunarg.com/wp-content/uploads/2025/10/XDC-2025-KosmicKrisp-Overview.pdf

## Not checked

- Nothing was run. The Mac numbers come from Ronald's run (Planner's relay) and round 16's calibration.
- #1's cause (empty Metal draws) is read from our code and MoltenVK's source and is not yet measured on the Mac.
- Several talks were only readable through secondary summaries: Nanite, Nubis, Wronski, Hillaire 2020 and Horizon Zero Dawn. Their numbers are marked so in the sources above.
