# render group

All 18 files are resolved, with no markers left. Nothing was compiled: the brief says no cargo.

## Per file

- **crates/engine/src/client/world_render.rs**: upstream's 4 mesh layers (opaque, cutout, translucent, upper face layers via `fs_layers`), body rotations, dry volumes, atmosphere binds and outline. Ours is kept on top: far terrain (lod), leaning structures, fading pipes, shadows, RT, CubeTable stats and the frame-perf occlusion pyramid. Upstream's far_lod/forest code is dropped (`mod far_lod` removed; far_lod.rs stays in the tree uncompiled). `records_of`/`groups_of` now use upstream's `page*8 + kind*4 + layer` buckets. `liquid_in()` became `.liquid`. Occlusion gained `capture_buffer` and the 5-argument `take_depth(.., atmosphere)`; the atmosphere's depth copy is its own buffer, and `has_pyramid()` is false. The shadow pass now binds BODY_ROTATIONS (identity quaternions) and DRY_VOLUMES (empty) as ranges of its offsets buffer. `texture_info` adds a zero overlay table, so the shader pack's `g.extra.x` still reads valid memory. `DEPTHS` falls back to the buckets buffer (upstream's fix).
- **crates/engine/src/gpu_layout.rs**: ours (the whole lod/lod_face/terrain header, records, tests) plus upstream's bindings (BODY_ROTATIONS, DRY_VOLUMES, CLOUD_CELLS, RAIN_DEPTH, RAIN_COVER) and their types. **Scene struct change:** all our fields are kept in their place, and upstream's new fields are appended at the end (environment, cloud: GpuSky, atmosphere_extent, cloud_lod, cloud_cache, atmosphere_cache, rain_cover, lightning_start/end), so the pack's offsets for our fields don't move. Globals `extra.x` is now our empty overlay table's start.
- **crates/engine/assets/shaders/world.wgsl**: rebuilt from upstream with ours on top. It has our LOD section, `backed` at location 13 (where upstream had lod_fade), the lean turn in `finish` followed by upstream's body rotation, and our sRGB vanilla shading and fog around upstream's lightmap, clip cube and dry-liquid clip. Translucent is wrapped in `through_world_atmosphere`. Upstream's LOD and forest entry points are dropped.
- **common.wgsl, sky.wgsl, ui.wgsl, model.wgsl**: upstream's code with ours on top. **occlusion.wgsl**: ours (the 32-tile pyramid). Its cull still reads `bucket & 4` like upstream's.
- **client/sky.rs**: upstream's atmosphere, volumetric clouds, rain and lightning, plus our soft clouds, froxels, cloud shadow and frame look. There is no `core:soft_clouds` read here; app.rs sets `soft_wanted = true`. Two comments were updated to match.
- **client/entity_render.rs**: upstream's preview, outline and worn structure, plus ours: `subject`, `preview_kind`, own shadow, hand pass, locomotion clips, first-person held scale and our thin outline look. Signatures match app.rs: `preload(gpu, assets, entities, comps, me)` and `draw_preview(.., look, worn, subject)`.
- **client/models.rs, mesher.rs (upstream's plus our BACKED bit), textures.rs, looks.rs, particles.rs, post.rs**: upstream's code with ours put back. `Post::set` was removed upstream.
- **docs/sky.md, docs/models.md**: both sides merged.

## SYNC comments in my files (7)

- crates/engine/src/client/particles.rs:366: `BLOCK_TINT` particle flag is a tag now.
- crates/engine/src/client/sky.rs:854: rain came from `WeatherLook::falls`.
- crates/engine/src/client/world_render.rs:804: drop the empty overlay table once the pack draws grass sides through the upper face layer.
- crates/engine/src/client/world_render.rs:1906: the world's opaque and cutout layers are CubeTable's (fade band).
- crates/engine/src/client/world_render.rs:2077: the TAA motion's turn is the lean's alone, without upstream's body rotation.
- crates/engine/src/client/world_render.rs:2346: upper face layers don't fade into distant terrain.
- crates/engine/src/client/world_render.rs:2862: `has_pyramid` is false; upstream's atmosphere could read our pyramid with its own offsets.

## SYNC items outside my files (not edited; for their owners)

- **crates/engine/src/client/world_render/rt.rs:621** will not compile. `self.buffers` now has 7 entries, so `let Some((_, obuf, ibuf, _, _))` must become `let Some((_, obuf, ibuf, ..))`. My edit was refused (file outside my list). Its `rt_plan` (around lines 695 and 709) also indexes `face_counts`/`quad_counts` by the old layer order (`translucent: n[1]`). Upstream's order is 0 opaque, 1 cutout, 2 translucent, 3 upper.
- **mods/base/shaderpack/world.wgsl:287** uses the old uv turn `(c + turn)`; upstream turns `c + 4u - turn`. Lines 312, 391 and 1560 read the overlay table at `g.extra.x`, which is now all zeros, so there are no overlays.
- **ui/render.rs** uses the old looks API (`slot_draw(c, stack.item)`, `ItemDraw::Icon`).
- **client/terrain_gpu.rs**: `far_lod_pipeline` is unused now (it already has a SYNC at line 437).

## For Ronald

- **Plain look:** upstream's volumetric weather clouds, rain, lightning and atmosphere are now part of the built-in sky, which changes sky cover. Clouds still write depth: upstream 077bd3318 was not fully taken, only its through-cloud tint. Our cloud wind drift was dropped for upstream's.
- **Plain look:** rotated block textures now turn the other way (upstream's uv turn fix). The pack still uses the old direction (see above).
- **Plain look:** the lightmap is upstream's, with night vision and the sunlight term, under our sRGB vanilla multiply.
- **Plain look:** a structure's faces are shaded by its body rotation.
- Overlays (grass sides) are face layers now. Far LOD grass sides lose their overlay, and the upper layers don't fade at the cube edge.
- These test flags are gone: mc-moon, mc-days, mc-clouds, `core:mc_leaves` (the far-tile leaves bit) and FACE_INNER.
- The block outline uses upstream's merged edges (no doubled edges between boxes) with our ~1 px dark look.
- Scene: upstream's fields are appended after ours. The pack reads only our fields, so their offsets are unchanged.
- Occlusion: upstream copies the full depth each frame for the atmosphere only when the atmosphere draws. Ours keeps the pyramid apart, which costs one extra full-resolution depth copy on those frames.
