# Group "backend": merge report

Files done (no conflict markers left, no `// SYNC:` comments added):
backends/vulkan/src/lib.rs, crates/render/src/lib.rs, crates/android/Cargo.toml, crates/android/src/lib.rs, crates/app/src/main.rs.
I followed the verdicts in LEDGER.md sections 12 and 13. No cargo was run, so nothing here is compile-checked.

## backends/vulkan/src/lib.rs
- **Upstream's file is the base.** It brings Vulkan 1.1 render passes and framebuffers (PassKey), sync1 barriers, per-frame descriptor sets (no push descriptors), samplers in set 2, multi_draw and draw compaction, set_surface, `turned` (the compositor turns the picture), precompile threads, and the pipeline cache file (`BackendDesc::cache`, saved once per frame).
- **Ours put back on top**, ported to upstream's model:
  - Upscalers: DLSS/NGX/Streamline (mods ngx, streamline, upscale; record_upscale; Texture.usage; create_upscale_target; Streamline interposer in load_vulkan).
  - RT and path tracing: mod rt; ray-query features and extensions; BLAS batching; TLAS bound through descriptor sets (ACCELERATION_STRUCTURE pool size and writes in bind_buffers), because upstream has no push descriptors; FillBuffer; Compare Equal.
  - MRT: the motion image and PassExtras as extra color attachments of a render pass; writes_location masks; create_pipeline_mrt.
  - fix-vulkan-sync:
    - layout_stages barriers in sync1 flags
    - WEFT_ALL_BARRIERS
    - the acquire wait at COLOR_ATTACHMENT_OUTPUT
    - early writes kept on the CPU (Early, early_wait)
  - fix-vulkan-oom: spill memory (spill_types, allocate, the spill test).
  - Diagnostics: MoltenVK error messenger, submit-split log (SubmitSplit, log_submit), device_class and device_memory_mb, and the driver log line.
  - depth-water-shafts: depth is sampled and bindless, in SHADER_READ_ONLY between passes; CopyDepthToBuffer copies from its real layout.
- **Let go** (verdicts):
  - Android: pre-rotation, image compression control, hand_own_pass and the PassBreak handling, our indirect_count path, the workgroup ArrayStride fix, textures cleared at creation, and surface_lost/restored (replaced by upstream's set_surface).
  - render-scale-cache: our pipeline cache load/save. **Upstream's `BackendDesc::cache` was chosen.** It saves whenever a pipeline was added, at most once a frame. That covers world open, and Android backgrounding as well, because the data is already on disk. Our UUID header check is gone, since the driver validates the data itself.
  - fix-vulkan-lifetime.
  - Our Vulkan 1.3 requirement: dynamic rendering, sync2 and push descriptors.
  - The instance now asks for the loader's version, capped at 1.3. Devices need 1.1, and RT needs 1.2.

## crates/render/src/lib.rs
- **Ours kept:** the upscaler API (with `BackendDesc::upscale_standin` added after upstream's `cache`), the RT and path-tracing API (accel, FillBuffer, Compare::Equal), MRT, and device_class and device_memory_mb.
- **Upstream's taken whole.** Nothing of upstream's was removed.
- **Ours dropped:**
  - pipeline_cache_data and load_pipeline_cache
  - the Android API: Blend::NoColor, Cmd::PassBreak, Backend::indirect_count, hand_own_pass, surface_lost and surface_restored

## crates/android/Cargo.toml
- Upstream's exactly: weft_android, lib "main".

## crates/android/src/lib.rs
- **Upstream's file**, plus one line of ours in the Options literal (the field is ours, phase2-packages; Options needs it): `package_dirs: vec![data.join("packages"), game.join("packages")]`.
- **Ours dropped:**
  - the STARTED re-entry guard and the process::exit
  - stdio_to_logcat
  - `--env` in args.txt (upstream reads options.txt)
  - own_dir and game.stamp
  - lib "chiacraft"

## crates/app/src/main.rs
- **Upstream's subcommand CLI:** `server`, `pack-mod`, `dist`, `submit-mod`, `export-models`, `optimize-assets`, `test-all`.
- **Ours on top:**
  - package_dirs and the `--packages` flag
  - a new `sign-package <dir>` subcommand, replacing our `--sign-package` flag
  - test_mods marked(play=false)
  - run_dir/packages put first
  - the `exit()` helper that flushes the log
- **Dropped:** the flags `--no-indirect-count`, `--no-indirect-cap`, `--server`, `--pack-mod`, `--dist` and `--sign-package`.

## Follow-ups for other groups (files not mine)
- `crates/render/src/rt.rs:107`: `Slab::new()` must become `Slab::default()`, because Slab moved to weft_render and has no `new`.
- `crates/engine/src/app.rs`: drop load_pipeline_cache, save_pipeline_cache and PIPELINE_CACHE. Use upstream's `set_surface` instead of surface_lost and surface_restored.
- `world_render.rs` and `entity_render.rs`: drop the calls to `indirect_count`, `hand_own_pass`, `Cmd::PassBreak` and `Blend::NoColor` (the cutout split, capped draws and the hand pass).
- `backends/wgpu/src/lib.rs` (new from upstream) must handle our extra Cmd variants and trait methods (upscale, RT, MRT, FillBuffer and so on), or have defaults for them.
- Callers of the old CLI flags:
  - `tools/release/package.sh`: `--dist` becomes `dist`.
  - `mods/testkit` gameplay.rs: `--server` becomes the `server` subcommand, as the first argument. As a flag it now goes to the mods and starts a client.
  - The package_trust.rs doc: `weft --sign-package` becomes `weft sign-package`.
- `tools/release/android.sh` (`-p chiacraft-android`) no longer matches. Upstream's `.github/android/build.py` uses lib "main".
- `notes/ENGINE_CHANGES.md`: update the rows that were let go (Android rows, render-scale-cache, fix-vulkan-lifetime).

## For Ronald
- **Android is upstream's now.** We lose:
  - pre-rotation, so the compositor turns the picture again (about 1.95 ms a frame on the Pixel 10)
  - the Pixel 10 fixes: the hand in its own pass (water squares), cutout in two passes, the compression-off switch, the workgroup ArrayStride fix, textures cleared at creation
  - our optional-drawIndirectCount path (upstream has its own draw compaction)
  - the second-start guard, args.txt and `--env`
  - Recheck on the Pixel 10.
- **The Vulkan backend is upstream's Vulkan 1.1 render-pass design now.** Our RT, path tracing, DLSS, MRT/TAA motion, spill memory, sync tightening and diagnostics were ported onto it, but nothing is verified. Risky spots:
  - the barrier stages, now in sync1 flags
  - the TLAS, now bound through descriptor sets
  - RT needs a device on 1.2
  - the instance API version goes up to 1.3
  - Needs a check, and a look and bench on the PC.
- **Pipeline cache:** upstream's file (saved each frame a pipeline was added) replaces ours.
- **The CLI moved to subcommands:** `weft server`, `weft dist`, `weft pack-mod`, and our `weft sign-package`. Scripts using the old flags need updating.
- No gameplay content or look change comes in through these files.
