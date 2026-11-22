# Group "terrain": conflict resolution report

All six files have no conflict markers left. There are 3 `// SYNC:` comments. Nothing was built or run.

## crates/engine/src/client/lod.rs
- **Ours, the whole file** (`:2:`), because far terrain stays ours. The only change is an SDK-rework adaptation: `c.game.config_names` became `c.game.catalog.ids(weft_abi::entry_kind::CONFIG)`, in 2 places (`split_widths`, `Lod::distance`).
- **Upstream items our lod.rs does not have.** Other code that upstream changed may call these. The compile step must adapt that code to ours, or keep ours at those spots:
  - `Lod::new(jobs, cache_dir: PathBuf, shader: Option<u64>)`. Ours is `Lod::new(jobs)`. Upstream calls it in client/mod.rs:1040 with `gpu_terrain.lod_shader`.
  - `Lod::update(gpu, render, world, textures, seed, camera: &Camera, frame: &FrameInfo, early: &mut CommandList, far_compute: Option<PipelineId>, normal_fov, view, upload_budget: Duration, animate)`. Ours is `update(gpu, render, world, textures, seed, cam: DVec3, view)`. Upstream calls it in client/mod.rs:3291.
  - `Lod::set_columns(dim, all, cols: &[LodColumn])` takes upstream's `server::lod_tops::LodColumn`/`Span`. Ours takes `&[(i32, i32, i32, u32, u32)]`. Upstream calls it in receive.rs:271. Ours also has `set_far(dim, grid)`.
  - These are missing in ours:
    - `Lod::set_edits(dim, all, &[AreaFill])` (receive.rs:108)
    - `Lod::reload()` (app.rs:2389)
    - `Lod::save_view()` (client/mod.rs:3151)
    - `Lod::ready_for_entry(dim, seed)` (client/mod.rs:1959)
  - `LodJob::description() -> (u64, &[i32])` is missing. Ours has public fields `custom`, `done`, `live`, `columns`, `far`, `asked` and `LodJob::live()`.
  - `LodMesh.counts: Counts`, `pub type Counts = [[u32; 6]; PARTS]` and the part consts `ALWAYS`, `BESIDE_CUBES`, `SURFACE`, `PLANTS`, `FOREST` are missing. Upstream's world_render.rs:315/799/806 uses them. Ours has `LodMesh { dim, seed, key, quads, skip, y, asked }`.
  - Ours has these and upstream does not: `pub type Columns`, `MAX_LEVEL`, `SMOOTH_LEVEL`, `TileKey::smooth`, `pool()`, `fall()`, `mesh()`, `mesh_smooth()`.
- **crates/engine/src/client/lod/ (cache.rs, cover.rs, detail.rs, forest.rs).** Our lod.rs has no `mod` lines for them, so they aren't compiled and ours does without them. They should be removed at the compile step, together with upstream's `world_render/far_lod.rs`, which serves only their LOD. I did not delete them.

## crates/engine/src/client/terrain_gpu.rs
- **Ours stayed**: `describe_lod`, `crown_of`, `custom_heights`, `lod_desc` (descriptions made on LOD's threads), the CPU-feature heights in the job stride, `lod_desc_key: Option<(u32, u64)>`, `LodJob::live` filtering, smooth flag in tile jobs, `describe -> (words, custom)`.
- **Upstream's came in**:
  - `precompile`, `PIPELINES`, `shader()`, and the fields `far_lod_pipeline` and `lod_shader`
  - entries `catalog` config lookups
  - `crate::features::warm`, in place of `warm_scatters`
  - `crate::pool` for the unpack pool
  - the `area::catch_up` signature
  - the `log::debug!` line in `tiles`
- **`far_lod_enabled()` now returns `false`**, so the `lod_far` pipeline is never made: our terrain.wgsl has no `lod_far` entry. SYNC at **line 437**.
- **SYNC at line 164 (`describe_lod`)**: upstream replaced `DimensionDef::scatters`/`ScatterDef` with `features::FeatureDef`, using `Spread::Grid` + `Chance::Density` + `Shape::Plan`. The scatter section must read features instead. The far sites' hash in `terrain.wgsl` `lod` must also match `features`' grid starts. The content.rs owner should know about this.

## crates/engine/assets/shaders/terrain.wgsl
- **Ours, plus one upstream hunk.** Our `lod` entry and its helpers stay, and so does our record format. That one hunk is 9065e3223 in `generate`: a cave biome with threshold <= -1 fills every deep cube. GPU must equal the CPU, and upstream's `server/terrain.rs` has the same rule.
- **Dropped**, all LOD-only:
  - upstream's `lod_far`, with `FarSample` and the `far_*` functions
  - their reworked `lod` (two-pass caves, `lod_cave`, `cave_plane`, `lod_floor`)
  - the removal of `carved_at` and of the 64-bit hash helpers

  This matches the render group keeping ours in world.wgsl and gpu_layout.rs. gpu_layout must keep ours for `terrain::lod`, `d::FACE_FALL`, `d::SCATTER_OFF`, `d::CONT_SCALE`, `words::CUSTOM`, `words::SCATTER_BIOME`, `lod::SHAPES` and `crown`.

## crates/engine/src/server/terrain.rs
- **Upstream's file is the base** (terrain as an `entry_kind::TERRAIN` entry, `ColumnCache` from `weft_noise::cache`). Ours is written against it:
  - `Feature.custom`, plus `height_with` and `height_at`
  - `Mask` (scatter thinning); `from_steps` now takes `&[abi::ExprStep]`
  - `Faces`, `FarLeaves`
  - `Terrain.faces`, `far_leaves` and `continent_scale`
  - `Terrain::fields(seed)`, `field_values`
  - `standard_fields(continent_scale)`, plus `noise_field`
  - `one_column`/`ONE_COLUMN`, and the fast path on top of upstream's new `surface_block`
  - `Clone` for `Terrain`, written by hand with fresh caches, because upstream's `ColumnCache` isn't Clone; ours needs it for the content switch
  - `Clone` derives on `Field`, `Feature`, `Rule`, `Caves`, `CaveBiome`
- **`build()` reads our values as entry components.**
  - `t::CONTINENT_SCALE` (f64, at least 1 block, else an error).
  - `t::FACES` (callback) with `t::FACE_FALL` (i32).
  - `t::FAR_LEAVES` (callback).
  - A landform takes its height from the callback named `t::CPU_HEIGHT` + its full name; its `height` is then empty.
- **Tests** were ported to upstream's `Ops` entry helper: `far_surface_block_is_the_column_s_own_before_caves`, `standard_fields_match_the_cpu_fields`, `continent_scale_defaults_and_follows_its_component`.
- **Needed in crates/abi** (abi group), all in `comp::terrain`:
  - `CONTINENT_SCALE = "terrain.continent_scale"`
  - `FACES = "terrain.faces"`
  - `FACE_FALL = "terrain.face_fall"`
  - `FAR_LEAVES = "terrain.far_leaves"`
  - `CPU_HEIGHT = "terrain.cpu_height."` (a prefix)
  - and the structs kept: `FeatureArgs`, `FaceArgs`, `FarLeavesArgs`

  The old `TerrainDesc`/`terrain_op`/`FeatureDesc` are no longer used by these files.
- **Needed elsewhere**:
  - `weft_noise::fixed::Fields { seed, continent_scale }`, `Fields::with_continent_scale` and `CONTINENT_SCALE` (noise/src/fixed.rs, continent-scale row).
  - `DimensionDef::custom_cache` must become a `weft_noise::cache::ColumnCache`; `server::terrain::ColumnCache` is gone.

## crates/sdk/src/terrain.rs
- **Upstream's file is the base** (`Landform`, entry builder, `comps::terrain`). Ours is on top:
  - `Landform::custom(fn)` replaces our `Feature::custom`. `landform()` sends its callback as a per-name `CPU_HEIGHT` component, and an unnamed one gets a generated name.
  - `TerrainBuilder::continent_scale`, `faces(fall, f)`, `far_leaves(f)`, through a local `mod ours` of `Component` statics, so comps.rs stays untouched.
  - `Face` (its `surface_block` now goes through `Generation`, not `host().dim_surface_block`).
- **SYNC at line 689**: our `ScatterBuilder` with `thin`/`thin_in` went with upstream's scatter builder, which features replaced. Thinning needs a home on the feature builder (sdk/feature.rs, not my file). Our mods that call `.thin(...)`, `Feature::custom`, `Feature::new` or `r.scatter(...)` must move to `Landform::custom`/`Landform::new` and features.

## docs/worldgen.md
- **Upstream's text is the base** (landforms, `spacing`, `plan`, edits as entries). Our lines kept on top:
  - the `land.rs` example, now saying `Landform::custom`
  - the climate range note
  - runtime biome tints (`set_biome_tints`), appended to upstream's tint line
  - `seasons(..)`
  - the `continent_scale` paragraph and its table row
  - the forest canopy paragraph
  - underwater plants
- The scatter reach stays upstream's "32 and 16, `reach_y`". Our "96" was a scatter limit, and upstream's features set reach per feature.

## For Ronald
- **New underground fill.** Upstream's 9065e3223 is in, on the GPU and the CPU: a cave biome with threshold -1 fills every cube deep enough, as a fallback. It changes underground biomes only where a mod registers such a cave biome. Upstream's base worldgen does; that's the mods group's to list.
- **Upstream's new LOD is held out, as decided**, pending the A/B: GPU coarse terrain (`lod_far`), adaptive LODs, distant forest impostors, two-pass cave roofs and open shafts in far terrain, water LODs and the LOD disk cache. Far terrain looks as it does today.
- **Far trees may be placed wrong until fixed.** Upstream replaced scatters (trees, boulders) with "features". Our far crowns read the old scatters (SYNC in terrain_gpu.rs), so until that is ported, far trees may be missing or off-site against the near ones.
- **Scatter thinning is unported** (groves and glades that far forests copy exactly). It has no SDK home yet (SYNC in sdk/terrain.rs).
