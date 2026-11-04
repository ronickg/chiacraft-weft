# Research round 14: a KosmicKrisp A/B and a frame-pacing plan

Researcher, 2026-11-04, for the lag worker, Tester and Planner. Research only; no code was changed and no game or install was run. This turns round 12's findings 3 (KosmicKrisp against MoltenVK) and 4 (a CPU frame cap) into plans with exact commands.

**What was read:** `backends/vulkan/src/lib.rs` (loader, features, extensions), `crates/engine/src/app.rs` (the frame, the existing fps cap), `settings.rs`/`config.rs`, Tester's `gamerun.sh`, `bigrun.sh` and `lag-big-dev19-a.sum` (the first full-size Mac row), the installed Homebrew setup on this Mac (read-only `brew info`/`brew deps`, `strings` on `libMoltenVK.dylib`), and the web pages cited below. Pages that blocked us: Mesa's GitLab (Anubis "access denied"), Phoronix (403), the LunarG SDK getting-started page (no content), and the GitHub mirror path for Mesa (404), so **KosmicKrisp's own feature and limit list could not be read** (marked below).

## The short answer

1. **The first full-size Mac row changes what matters.** At 3456×2234 on dev `19a37dad` the game is **GPU-bound, not CPU-bound**: `acquire` is 5.9–12.7 ms of each frame because the CPU waits for a drawable the GPU hasn't freed, and the GPU time is 10.6 ms (stand) and 16.8–17.3 ms (after the flight), of which **screen effects are 5.8 and 12.4–12.7 ms** while the world pass is only about 2.4 ms. A frame cap or a driver swap won't fix that; render scale and the effects will. This row predates dev's `0a9116b0` (half-resolution effects and the other `perf` commits) and `render_scale`, so **the first thing to do is repeat the same row on today's dev** (row R1 below).
2. **KosmicKrisp is a real option on this Mac, and cheap to try.** It is Mesa's Vulkan-on-Metal driver (MIT), Vulkan 1.4 conformant, needs Apple Silicon and **macOS 26** (this Mac runs 26.5.1, arm64). Homebrew's `mesa` 26.2.6 bottle for macOS 26 builds it. Weft needs **no code change** to use it, but the A/B must **pin the driver with `VK_DRIVER_FILES` on both sides**, because installing it makes two Apple devices visible and the engine's device pick is a tie. No published speed comparison exists: LunarG says it is "close to MoltenVK" (secondary report); one emulator reports lower memory but more shader-compile hitches.
3. **The frame cap already exists** (`core:fps_cap`, 0–360 in steps of 10, a sleep at the start of the frame, `app.rs:2064`). It needs no new code to test. The Retina row shows why pacing matters but also its limit: at 95 fps the frames alternate between one and two 120 Hz ticks (p50 9.0 ms, p99 16.9 ms); a cap of 60 would make them an even 16.7 ms, but it cannot make a GPU that needs 10.6 ms finish in 8.3.

## Part A: KosmicKrisp against MoltenVK

### What it is, version, licence, how to get it

| fact | source |
|---|---|
| Mesa driver `kosmickrisp`, Vulkan on Metal 4, by LunarG; **MIT**; Apple Silicon M1 and later; **macOS 26+**; no iOS, no Intel | [LunarG product page](https://www.lunarg.com/products/kosmickrisp/), [Mesa docs](https://docs.mesa3d.org/drivers/kosmickrisp.html) |
| Vulkan 1.3 conformant (Khronos-listed), then **Vulkan 1.4 conformant**; LunarG: "performance has tripled" since the 1.3 conformance; optimisation "ongoing" | [1.4 conformance post](https://www.lunarg.com/kosmickrisp-achieves-vulkan-1-4-conformance-on-apple-silicon/) (its header and body disagree on the year, 2026 vs 2025: don't rely on its date) |
| In the **LunarG Vulkan SDK 1.4.363.0** as "a Vulkan 1.4-conformant ICD" next to MoltenVK; the SDK page also says GPU-assisted validation works with KosmicKrisp and not with MoltenVK | [SDK 1.4.363.0 release](https://www.lunarg.com/lunarg-releases-vulkan-sdk-1-4-363-0/) |
| **Homebrew `mesa` 26.2.6**, bottled for `arm64_tahoe` only; its formula sets `vulkan_drivers = (MacOS.version >= :tahoe) ? "kosmickrisp,swrast" : "swrast"` (with a comment that KosmicKrisp needs Metal 4); it depends on `molten-vk`; deps not yet installed here: libclc, spirv-llvm-translator, spirv-tools and the X libs | [formula source](https://raw.githubusercontent.com/Homebrew/homebrew-core/HEAD/Formula/m/mesa.rb), `brew info mesa` / `brew deps mesa` on this Mac |
| Source build: `meson setup … -Dplatforms=macos -Dvulkan-drivers=kosmickrisp -Dgallium-drivers= -Dopengl=false -Dzstd=disabled --prefer-static`, needs Xcode tools, meson 1.9.1+, llvm 20.1.8+, libclc, spirv-llvm-translator, spirv-tools | [Mesa docs](https://docs.mesa3d.org/drivers/kosmickrisp.html) |
| Select a driver with `VK_DRIVER_FILES` pointing at its manifest (LunarG: `libkosmickrisp_icd.json`) and confirm with `vulkaninfo` | [1.4 conformance post](https://www.lunarg.com/kosmickrisp-achieves-vulkan-1-4-conformance-on-apple-silicon/) |
| Debug knobs: `MESA_KK_DEBUG=msl` (log generated Metal shaders), `MESA_KK_GPU_CAPTURE=1` (Xcode capture), `MESA_KK_DISABLE_WORKAROUNDS`, `MESA_KK_EXPERIMENTAL` | [Mesa docs](https://docs.mesa3d.org/drivers/kosmickrisp.html) |
| Another user's measurements (xemu, M2 Ultra, Mesa 26.3.0-rc1): 0.4–0.6 GB less peak memory; slowest pipeline compile warm 118 → 10 ms (MoltenVK 136 → 5 ms); **11 compile hitches on a cold walk against 6 on MoltenVK**; an occasional occlusion test reads empty under heavy host load; wireframe draws render filled; first launch of a new build 1.6–2.2 s slower. They keep MoltenVK as default. | [xemu v0.16 notes](https://github.com/MichaelJSr/xemu-macos/releases/tag/v0.16) |
| "Close to MoltenVK, not far behind native Metal" | Phoronix, reported in a search summary only (page blocked): unconfirmed |

**Signed or from source?** I found no statement about signing. Homebrew's bottle is Homebrew's own build (ad-hoc signed, as all its bottles). LunarG's SDK is a downloaded installer: expect a quarantine flag (`xattr -dr com.apple.quarantine <folder>` if macOS blocks the library), and I could not confirm whether it is notarised. A source build is not worth it for a test.

**Options, in order of cost:**
1. **`brew install mesa`** (downloads bottles of mesa and its few missing deps; llvm 22 is already here). It writes the KosmicKrisp and the software-rasteriser (lavapipe) manifests next to the loader's search path, so **it changes what an unpinned run uses** (see "the trap" below). Installing software is Ronald's call.
2. **The LunarG SDK macOS installer** into `~/VulkanSDK/1.4.363.0`: user space, leaves Homebrew alone, also gives `vulkaninfo`. Exact installer steps unconfirmed (the page that lists them blocked us).
3. Source build: no.

### How Weft picks the driver with no code (`backends/vulkan/src/lib.rs:201-235`)

1. **`libMoltenVK.dylib` beside the executable goes first** and stands in for the loader, so **ICD selection never runs**. Tester's builds (`exp/*/target/release`) have no such file (checked on all of them), so they take path 2. A release build that bundles MoltenVK would ignore every variable below.
2. Otherwise `ash::Entry::load()` (the system name `libvulkan.dylib`, which dyld doesn't find under `/opt/homebrew`), then the fallback loop tries `/opt/homebrew/lib/libvulkan.1.dylib`, the Homebrew loader (1.4.363.0). That loader reads the manifests it knows: today only `/opt/homebrew/etc/vulkan/icd.d/MoltenVK_icd.json` (library `../../../lib/libMoltenVK.dylib`, `is_portability_driver: true`).
3. **`VK_DRIVER_FILES`** ("force the loader to use the specific driver JSON files"; `VK_ADD_DRIVER_FILES` adds; `VK_ICD_FILENAMES` is the deprecated older name; all ignored for privileged processes; separators `:` in the loader's Unix examples, macOS not named) ([Loader docs](https://raw.githubusercontent.com/KhronosGroup/Vulkan-Loader/main/docs/LoaderInterfaceArchitecture.md)). `VK_LOADER_DRIVERS_SELECT='*kosmic*'` (a glob on the manifest name) is the other way. `VK_LOADER_DEBUG=driver` makes the loader print which driver it loaded; `gamerun.sh` sends stderr into the run's log, so it lands there.
4. The engine then picks the device with the best type score (`lib.rs:336-345`): discrete 3, integrated 2, other 1, **first wins a tie**. Apple GPUs are integrated in both drivers, so with both manifests visible the choice depends on enumeration order. It logs only `GPU: <device name>` (`lib.rs:351`), which is probably "Apple M3 Max" for both drivers (unconfirmed), so **the log doesn't say which driver ran**.
5. **The trap:** after `brew install mesa` the Homebrew loader lists MoltenVK, KosmicKrisp and lavapipe. An unpinned run takes whichever comes first, so a later "MoltenVK" row could silently run something else. Pin **every** row with `VK_DRIVER_FILES`, and grep `VK_LOADER_DEBUG=driver` output for the library name.

### What Weft needs from the driver, and what KosmicKrisp might lack

From `backends/vulkan/src/lib.rs` (`create`), with the engine's own check: **a missing feature stops the start with a message naming it** (`"<device> lacks what the game needs: …"`, `lib.rs:392`). So the first probe is simply starting the game with the pinned driver.

| need | where | note |
|---|---|---|
| Vulkan 1.3 API; instance + `KHR_portability_enumeration` if offered | `:285` | the engine adds it only if the loader lists it; KosmicKrisp is not a portability driver |
| features: multiDrawIndirect, drawIndirectFirstInstance, samplerAnisotropy, fillModeNonSolid, shaderInt16, storageBuffer16BitAccess | `:365-411` | 1.4 conformance doesn't require all of these: **unconfirmed** |
| descriptor indexing: runtimeDescriptorArray, partially bound, **sampled-image update-after-bind**, non-uniform indexing of sampled images; **65,536 sampled images per set and per stage** (`MAX_TEXTURES`, checked at `:356-359`) | `:369-392` | optional in Vulkan 1.2 and 1.4; **the likeliest gap**, unconfirmed (Mesa's source blocked us) |
| scalarBlockLayout, dynamicRendering, synchronization2 | `:380-394` | core in 1.3; fine |
| `VK_KHR_push_descriptor` (**required**, `:419`) | `:419` | LunarG lists push descriptors in KosmicKrisp's early work; core in 1.4 |
| `VK_KHR_portability_subset` only if the driver has it | `:423-428` | |
| D32_SFLOAT depth, B8G8R8A8_SRGB swapchain, timestamp queries (`cmd_write_timestamp2` TOP_OF_PIPE/ALL_COMMANDS) | `:15, 664, 1593` | the bench's GPU parts use timestamps; check they read plausible |
| `draw_indirect_count` only if present, never issued by the engine | `:409` | MoltenVK splits the render pass on it; irrelevant here |
| not used: wireframe (`PolygonMode::FILL` only), occlusion queries, MSAA (1 sample) | `:1432, :1318` | so two of xemu's caveats don't touch us |

Shader path: naga produces SPIR-V (`naga` in `Cargo.toml`), which KosmicKrisp compiles to Metal Shading Language; MoltenVK uses SPIRV-Cross. Different compilers can disagree on edge cases: compare shots, not just speed.

**Pipeline cache interaction.** The `claude/pipeline-cache` branch (`b1867503`) is **not in dev** (dev still passes `PipelineCache::null()`, `lib.rs:1496, 1524`). If it merges: it keeps one `pipelines.bin` and loads it only when the header matches "this very driver and device's", so two drivers sharing a cache directory overwrite each other's file and every switch starts cold. Use one run dir per driver, and read a warm run (the second), because KosmicKrisp showed more compile hitches when cold.

### Exact commands for Tester (a plan; nothing here has been run)

Rules that apply: one game at a time, only through `bigrun.sh` (so the stall guard runs), and **a Mac run on a new GPU driver needs Ronald's clear yes** for that run (crew rule and the GPU-hang memory). Start small.

```bash
# Once (needs Ronald's yes: installs software). Option 1:
brew install mesa
ls "$(brew --prefix)/share/vulkan/icd.d/" "$(brew --prefix)/etc/vulkan/icd.d/"
#   expect a kosmickrisp manifest (name unconfirmed: libkosmickrisp_icd.json or kosmickrisp_icd.<arch>.json)

export MVK_ICD=/opt/homebrew/etc/vulkan/icd.d/MoltenVK_icd.json
export KK_ICD=$(ls "$(brew --prefix)"/share/vulkan/icd.d/*kosmic*.json | head -1)
export EXP=~/Coding/chiacraft-archive/tester-tools/exp/<the build of dev's tip>   # no libMoltenVK.dylib beside it

# Probe 0 (no game): which devices each pinning shows. Needs vulkan-tools (brew install vulkan-tools) or the SDK.
VK_DRIVER_FILES=$KK_ICD  vulkaninfo --summary | head -40
VK_DRIVER_FILES=$MVK_ICD vulkaninfo --summary | head -40
VK_DRIVER_FILES=$KK_ICD  vulkaninfo | grep -E "maxDescriptorSetUpdateAfterBindSampledImages|maxPerStageDescriptorUpdateAfterBindSampledImages|descriptorBindingSampledImageUpdateAfterBind|runtimeDescriptorArray|scalarBlockLayout|pushDescriptor|KHR_present_wait|KHR_present_id"
# Compare the two limits with the game's need: both counts must be >= 65536.

# Probe 1: does the engine start? Small window, the standard script, driver pinned, loader choice logged.
cd ~/Coding/chiacraft-archive/tester-tools
SIZE=1280x720 VK_DRIVER_FILES=$KK_ICD VK_LOADER_DEBUG=driver ./bigrun.sh $EXP kk-probe
#   success: BENCH lines in runs/lag-kk-probe.sum. Failure to start prints "lacks what the game needs: ..." in the log: stop, report the names.

# The A/B at Ronald's size: same build, script, seed; each arm twice (a = cold, b = warm).
for arm in a b; do
  VK_DRIVER_FILES=$MVK_ICD VK_LOADER_DEBUG=driver ./bigrun.sh $EXP mvk-$arm
  VK_DRIVER_FILES=$KK_ICD  VK_LOADER_DEBUG=driver ./bigrun.sh $EXP kk-$arm
done
# Then the same four with the world cheaper, to separate driver cost from effects cost:
#   append:  --config core:render_scale=1   (S0)   and   --config core:render_scale=0.67   (A0)
```
Notes for Tester: set `VK_DRIVER_FILES` on the same line as `bigrun.sh` (it flows through `gamerun.sh` into `weft`; verified in `gamerun.sh:13`); don't run both arms back to back without the load check that `bigrun.sh` already does (it waits for load under 15); capture `/usr/bin/top -l 1 -pid <weft pid>` or `ps -o rss=` once near the end for memory, since memory is where KosmicKrisp is reported better.

### What to compare, and when to adopt

- **From the BENCH lines:** gpu ms per part (`world`, `effects`, `shadows`), p50, p99, p99.9, max, frames over 33 ms, `acquire` ms, time from process start to the first frame (log timestamps), and the "slow frame" lines. Compare the **b** (warm) runs; list the **a** (cold) hitch counts separately.
- **From shots:** three testkit shots per driver (title, day with trees, water), checked for visible differences: shader compilers can differ on precision (shadows, clouds, water), and KosmicKrisp must not crash or go black.
- **Adopt as an option only if** it starts without a missing feature, no GPU stall or device loss appears in the logs of four runs, and it is at least about 10% better on gpu ms or p99 on the Retina row **twice**, with matching shots. Otherwise record the result and stop; the cost of being wrong is a GPU-hang risk on Ronald's Mac.
- **If adopted, shipping is a separate piece** (effort M): the load order (`lib.rs:201`) would prefer KosmicKrisp on macOS 26+ arm64 when it is bundled and fall back to MoltenVK on any failure; both libraries and the loader go in the app bundle ("bundle KosmicKrisp and the Vulkan loader in your application bundle", LunarG); licence texts for Mesa's MIT parts and its dependencies must ship. MoltenVK stays for Intel and older macOS.

## Part B: a frame cap, from today's code and the first big row

### What exists

`frame()` (`app.rs:2063-2071`): `core:fps_cap` (0 = off; 0–360, step 10), `sleep(1/cap − elapsed since the last frame began)` at the **start** of the frame, before input and update. Run it today with `--config core:fps_cap=60`. Vsync is `FIFO` with 3 swapchain images and 2 frames in flight (`lib.rs:12, 729`).

### What the Retina row says (dev19, 3456×2234, GPU-bound)

| pass | fps | avg / p50 / p99 ms | acquire ms | gpu ms (effects) |
|---|---|---|---|---|
| stand | 95 | 10.58 / 9.04 / 16.88 | 5.86 | 10.56 (5.77) |
| walk | 102 | 9.84 / 8.82 / 15.97 | 5.13 | 9.81 (5.21) |
| view_far | 119 | 8.38 / 8.32 / 9.18 | 0.01 | 7.02 (2.05) |
| after_flight | 60 | 16.79 / 16.67 / 22.87 | 11.91 | 16.78 (12.37) |
| settled | 58 | 17.28 / 16.76 / 24.30 | 12.65 | 17.27 (12.66) |

- **Quantised frames.** In `stand` p50 9.04 ms and the mean 10.58 fit about 78% of frames at one tick (8.33 ms) and 22% at two (16.7 ms): the display shows frames on a fixed 120 Hz grid in this window, so a GPU time over 8.33 ms alternates between 8.3 and 16.7 (Apple's "present evenly" point, WWDC21 10147). `after_flight` sits at exactly 60: every frame misses the first tick.
- **A cap doesn't lower GPU time**; `acquire` already is the back-pressure. What a cap changes is evenness: `stand` at a cap of 60 would be an even 16.7 ms instead of 8.3/16.7 alternating at an average of 95 fps; `after_flight` is already at 60 and a cap changes nothing.
- **Hypotheses to test, not facts:** (1) with the GPU idle 37% of the time under a 60 cap the GPU may clock down and take longer per frame (round 12's duty-cycle hypothesis); (2) the window is composited, and fullscreen may behave differently (Apple forum 733033: direct mode was worse on some macOS 13–14 releases).
- **The real cost is the effects**, 5.8 ms (stand) to 12.7 ms (flight) at full resolution on this build. Half-resolution effects (dev's `0a9116b0`) and `render_scale` (default auto 2/3 on high-density screens) are expected to cut that sharply: **first repeat this row on today's dev**, because the cap question is different if stand is then under 8.3 ms.

### Plan (in order; a, b, c need no new code)

1. **Row R1: the same big row on dev's tip** (default settings, `render_scale` auto, perf merged) and **S0** (`--config core:render_scale=1`). If stand and walk now have GPU under about 7 ms, pacing is a smaller issue: stop at step 3.
2. **Rows C120, C60, C40** with the existing cap: `./bigrun.sh $EXP cap60 --config core:fps_cap=60` (likewise 120 and 40), vsync on, driver pinned to MoltenVK. Read p50, p99, p99.9, frames over 20 ms, and whether `after_flight` changes.
3. **Metal HUD** during one row each (`MTL_HUD_ENABLED=1`; log to file with `MTL_HUD_LOG_ENABLED=1`): it shows whether the window is "direct" or "composited" and the real present intervals (Apple's Metal HUD docs). Repeat one row in fullscreen (`F11`, the setting `core:fullscreen`) to see whether the 8.3/16.7 pattern changes.
4. **Pacing metrics in BENCH (small, the lag worker's file `bench.rs`).** Add per-pass counts of frame intervals in bins by display ticks, for example under 12.5 ms, 12.5–20.8, 20.8–29.2, over 29.2, plus the percentage near a tick. Averages hide the 8/16 alternation that is the complaint.
5. **If the existing cap helps but wobbles (a 25 ms frame now and then), improve how it waits (S):** sleep only to **about 1.5 ms before** the target interval and let the swapchain's own wait (`acquire`) finish the job, instead of sleeping the whole share (macOS timer overshoot of a millisecond or more turns a 16.7 ms target into a missed tick, which is a 25 ms frame). *Inference from the 120 Hz grid above; test it.*
6. **If a fixed cap is not enough: an automatic tick-snapped cap (S–M).** A setting `core:fps_cap = auto` that picks the interval from {120, 60, 40, 30} by the 90th percentile of the last second's work time: step down when over about 90% of a tick, step back up only after about 3 s under about 65% of the next tick, so it never oscillates. Take the tick from `winit` (`current_monitor().refresh_rate_millihertz()`). *My design; no source describes this exact scheme.* Test on the rows above, including `after_flight`.
7. **Only if 6 isn't smooth: know the real present times (M).** Enable `VK_KHR_present_id` and `VK_KHR_present_wait` in the Vulkan backend (not enabled today: `dev_exts` has only swapchain and push descriptor, `lib.rs:419`) and `vkWaitForPresentKHR` on frame n−1 after submitting n. MoltenVK 1.4.2 exports both (confirmed with `strings` on the installed `libMoltenVK.dylib`; also `VK_GOOGLE_display_timing`, which MoltenVK only records). Whether KosmicKrisp has them is unconfirmed, so feature-test at start and fall back. This also gives the bench true on-screen intervals.
8. **Not worth it:** MoltenVK has no `presentAfterMinimumDuration`, no `CAMetalDisplayLink` and ignores `desiredPresentTime` (round 12), so Apple's own adaptive pacing recipe is out; and two swapchain images instead of three (MoltenVK's guide recommends three for fullscreen).

### Settings screen

If a cap ships, say it in plain words: "Frame rate: off, 120, 60, 40, auto", with the Low quality preset on auto. The UI session's Video screen already has presets; a row there needs the setting's name `core:fps_cap` (steps of 10 today, so 40, 60, 120 exist; "auto" would be a new value).

## Not found, not checked

- KosmicKrisp's real feature and limit list (Mesa's GitLab and its GitHub mirror path did not load): whether it has the 65,536-image update-after-bind set, scalar block layout, `present_wait`. **Probe 0 answers it in minutes.**
- The ICD manifest file name after `brew install mesa`, and whether the LunarG installer is notarised.
- Any real speed comparison on a game: none published.
- Whether dev's tip already has GPU time under 8.3 ms at Retina (the rows above are from an older build).
- Whether the macOS 26 windowed mode here is fixed 120 Hz or adaptive: inferred from the quantised frame times, to be shown by the Metal HUD.
- Nothing was installed, downloaded as software, or run.

## Sources

- [LunarG KosmicKrisp](https://www.lunarg.com/products/kosmickrisp/), [1.4 conformance](https://www.lunarg.com/kosmickrisp-achieves-vulkan-1-4-conformance-on-apple-silicon/), [1.3 conformance](https://www.lunarg.com/lunarg-achieves-vulkan-1-3-conformance-with-kosmickrisp-on-apple-silicon/), [SDK 1.4.363.0](https://www.lunarg.com/lunarg-releases-vulkan-sdk-1-4-363-0/), [XDC 2025 overview](https://www.lunarg.com/wp-content/uploads/2025/10/XDC-2025-KosmicKrisp-Overview.pdf)
- [Mesa docs: KosmicKrisp](https://docs.mesa3d.org/drivers/kosmickrisp.html), [Homebrew mesa formula](https://raw.githubusercontent.com/Homebrew/homebrew-core/HEAD/Formula/m/mesa.rb)
- [Vulkan-Loader interface docs](https://raw.githubusercontent.com/KhronosGroup/Vulkan-Loader/main/docs/LoaderInterfaceArchitecture.md)
- [xemu-macos v0.16 notes](https://github.com/MichaelJSr/xemu-macos/releases/tag/v0.16), [Phoronix: parity](https://www.phoronix.com/news/KosmicKrisp-Parity) and [1.4 conformance](https://www.phoronix.com/news/KosmicKrisp-Vulkan-1.4-2026) (blocked: summaries only)
- Frame pacing: [WWDC21 10147](https://developer.apple.com/videos/play/wwdc2021/10147/), [Apple forum 733033](https://developer.apple.com/forums/thread/733033), [Zed on 120 fps](https://zed.dev/blog/120fps), [Metal HUD](https://developer.apple.com/tutorials/data/documentation/xcode/monitoring-your-metal-apps-graphics-performance.md), round 12's `reports/Research round 12.md` for the MoltenVK facts
- Local: `backends/vulkan/src/lib.rs`, `crates/engine/src/app.rs`, `tester-tools/{gamerun,bigrun}.sh`, `runs/lag-big-dev19-a.sum`
