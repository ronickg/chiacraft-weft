# chiacraft on Android

Ronald, 11-05: "Can we get the game compiled to Android?" ... "Yes, start it". This is where it
stands, how to build and run it, and what's left for a playable phone version.

## What works (first milestone, 2026-11-05)

Checked on the PC's Android emulator (Pixel 6 screen 2400 x 1080 held sideways, Android 15,
x86_64, the RTX 5090 through the emulator's Vulkan, gfxstream). Not yet on a phone.

- **Builds** for `aarch64-linux-android` (phones) and `x86_64-linux-android` (the emulator) into
  one debug-signed APK, without Gradle or Android Studio: `tools/release/android.sh`.
- **Starts** to the title screen in about half a second after the first unpack; the game, our
  four mods and the shader pack all load.
- **Plays a world**: `--world` from `args.txt` makes and opens one (about 12 s to the first
  frame of play on the emulator); terrain, the shader pack's look, the HUD. The account opens
  and the network inbox (iroh) starts, as on a desktop.
- **Touch** (`crates/engine/src/touch.rs`): a move stick under the left thumb (it starts where the
  finger lands, sprint past its edge), looking by dragging anywhere else, Jump, Sneak, Hit and
  Use under the right thumb, Pause, Inventory, Chat and Drop at the top middle, a hotbar slot by
  tapping it. In menus a finger is the mouse; a second finger dragged scrolls. Every button
  presses its action's own binding, so the game sees the same keys a keyboard sends.
- **UI scale for a phone**: auto picks GUI scale 4 on 2400 x 1080 (a desktop rule would pick 2,
  too small to hit); screens taller than 270 GUI pixels scroll.
- **Pause and resume**: Home pauses the world and drops the Vulkan surface; coming back makes a
  new surface and swapchain and the game goes on. The back button is Escape (closes a screen,
  opens the pause screen).
- **Files**: the APK carries the game as `assets/game.zip`, unpacked into the app's storage when
  its build changes (`<files>/game`); saves, settings and logs in `<files>/run`; mods' libraries
  per CPU in the APK's `lib/` folder (their zips carry only assets).

- **Inventory by touch** (the Inv button; the inventory and recipe panel fit the phone screen
  at GUI scale 4), back closes it, the pause button opens the pause screen.
- **Saves on going to the background**: the world's and player's files were written the moment
  Home was pressed (r7).

Pictures: `~/Coding/chiacraft-archive/images/android-port/2026-11-05/` (`r4` title screen,
`r5` a world with the touch controls, `r6` pause, resume, menu taps and back, `r7`
`grid_r7_touch.png` inventory, back and pause by touch, `grid_r7.png` the freeze below).

### Found in the second run, fixed

The first start after the emulator's cold boot: Android stopped and destroyed the activity 2 s
in, and the game froze with "chiacraft isn't responding". Android's main thread waits in
onDestroy until the native game ends, and winit 0.30 only logs Destroy. Now the game notices it,
saves and ends (a new start is a fresh process), a second `android_main` in one process ends
it too, and the manifest takes every configuration change in place so Android has fewer reasons
to recreate the activity.

Third run (r8, wip8, 11-05 19:10): a cold boot started cleanly (64 s in a world, no "isn't
responding"); Home and back kept the same process with a new surface. `am start -R 2` (Android
finishes the activity and starts it again) made the new activity start in the old process: the
second-`android_main` guard ended it and Android started a fresh process, which reopened the
world where it was saved, without a freeze. The onDestroy hook itself hasn't fired in a run yet
(Android didn't destroy the activity on its own this time). Pictures: `r8/grid_r8.png`.

### The desktop is unchanged

PC looks of dev (0c8189eb) and this branch (930bb728), the vista and the birch forest at 06, 12
and 18 h with the shader pack: the same pictures (0.1-2.6 % of pixels differ, leaves in the
wind and the far fill's timing). `desktop/dev_vs_branch.png`.

### Numbers on the emulator (not a phone)

| | title screen | in a world |
|---|---|---|
| frames | ~45 fps | ~30 fps |
| where the frame goes | waiting for the emulator's display (acquire 21 ms) | acquire 23 ms, submit 9 ms; the GPU's own work under 1 ms |
| memory (PSS / RSS) | 120 / 234 MB | 325 / 440 MB |

The frame rate is the emulator's display path, not the game: the RTX 5090 does a frame's work in
under a millisecond. A phone's numbers will be different in every way; only memory says much
(440 MB resident at view distance 12 is fine for a 6-8 GB phone).

## On a real phone: Ronald's Pixel 10 (2026-11-05)

Pixel 10: Tensor G5, PowerVR D-Series DXT-48-1536, Vulkan 1.4, Android 17, 2424 x 1080. It has
everything the renderer needs (push descriptors, fillModeNonSolid, bindless). Run from the Mac:
the phone on USB, `ANDROID_HOME=~/Library/Android/sdk tools/release/android-run.sh <apk> <out> 60
--world <name>` (the Mac's GPU isn't used).

- **Speed:** 28 fps in a world with the full look (shader pack, shadows, effects), GPU-bound (world
  pass ~29 ms, shadows ~2.5 ms). It was 10-13 fps until the swapchain stopped being rebuilt every
  few frames. Without the pack: 20 fps before that fix.
- **Start:** the first start after an install compiles the pack's pipelines, about 38 s, behind the
  loading screen. After that the saved pipeline cache opens a world in 0.4 s.
- **Fixed for it:** render targets without framebuffer compression (tiles only cleared read back
  as garbage: black and random-coloured squares over the sky); `NonUniform` on sampled images
  (naga 30 leaves it off); new textures cleared; the swapchain kept on "suboptimal".
- **The water glitch (fixed, 11-05 evening):** squares at the bottom of the screen over water, the
  sand under it showing. The first-person hand, drawn in the same render pass after the water,
  writes depth and discards the texels of the body parts it hides; Imagination's driver then
  lost the water in every 32-pixel tile that geometry touched. The hand now gets a render pass of
  its own there (`Cmd::PassBreak`). Found by scripted runs on the phone (testkit APK,
  `ANDROID_TESTKIT=1 tools/release/android.sh`, `~/Coding/chiacraft-archive/tester-tools/android/
  phone_script.sh`): spectator clean, survival glitched, the hand alone off clean. Screen
  recordings before and after: `pixel10_video/`, `pixel10_video_fixed/`.
- **Where the GPU time goes (11-05, night):** the bench's vista (`bench_vista.txt`: the PC bench's
  vista, view 8, distant terrain 128, one 1024 shadow map, render scale 0.5) ran at 13 fps, world
  pass 68.7 ms. Copies of the pack with one part of `fs_lod` off at a time (`pack_variant.py`): far
  land drawn flat 5.3 ms, its texture alone 9.1, no haze 36.3, no far water 47.3, no sun shadow map
  53.4, no crowns / sky colour / cloud shadow about 60, snow and the mist march nothing; no
  distant terrain at all 87 fps. So it's the far land's per-pixel light, not its geometry. Its
  haze per vertex instead was slower (87.6 ms: the tiles have more vertices than the screen has
  pixels at 0.5).
- **Far land without `discard` (810403c2):** `fs_lod`'s discard (columns the cubes show) turned off
  PowerVR's hidden-surface removal for every far tile; with none at all the world pass was 45.7 ms.
  Tiles wholly outside the cubes' mask now draw with `fs_lod_open`, the same without it, so the
  picture is the same (fresh-world shots, old build and new: mean difference 0.04-0.15 of 255):
  world pass 50.9 ms, the frame 77 -> 59 ms, 13 -> 17 fps. (On a world saved by killed runs, and
  in both builds, near chunks showed as holes in a steep look down: test worlds want fresh names.)
- **Same-picture speed-ups (11-06)**, the bench's vista and forest (`bench_two.txt`, Medium at half
  scale, a fresh world each; side-by-sides in the crew gallery), fps vista / forest:
  - start (after the discard fix above): 17 / 18;
  - the palette read once per pixel (`look.wgsl`, 7858c9e1): 19 / 21, world 52.2 -> 45.1 and
    42.7 -> 37.3 ms;
  - small trims (far water's cloud shadow once, no far texture past 520 blocks, the haze's and
    the sky's parts with no weight skipped): world 45.1 -> 44.0 ms at the vista;
  - leaves in two passes on Android (e870c0bc: depth with the alpha test, then shaded with no
    discard where the depth is theirs): 19 / 25, forest world 37.5 -> 29.6 ms (25.5 with leaves
    drawn solid, the bound).
  - after the dev merge (simpler trees, leaf holes; 589df211: 17 / 26, vista world 49.9 ms), far
    land and far water hazed by one call instead of a copy in each path (26a4d810): 20 / 26, vista
    world 40.1 ms. Removing single parts of the haze (its veil, its sun lobe) made the shader
    slower, not faster: what cost was the second copy of the whole haze in one shader (fewer pixels
    in flight). Near water and glass the same way (74006387): no change, the same picture.
  - the far land's fixed inputs written in as constants (`lod_color`: `vs_lod` always sets no glow
    texture, block shader slot 0, no placed light, not a plant, so the compiler drops those paths,
    which it cannot see through the interpolants): vista world 39.8 -> 34.2 ms, 22 -> 25 fps in the
    `var_sweep.sh` vista bench (two runs each, within 0.1 ms).
  Tried and dropped: haze per vertex (slower). Left: half precision (no `shaderFloat16` path in
  the engine yet), the far land's crowns and cloud shadow.
- **Settings on the phone** (`sweep.sh`, the same bench): sun shadows cost the forest 4 ms and
  the vista nothing; the screen effects about 2 ms; distant terrain 64 instead of 128 gives 20 / 20;
  the Low preset 23 / 27. The **Phone** preset (`video.rs`, 9b46a8c5: Low's distances, no sun
  shadows, Medium's effects) is offered on phones and chosen only by the player: the default stays
  the full look (Ronald: optimise, don't tune down).
- **Debugging on the phone:** `VALIDATION_LAYERS=<Khronos android-binaries folder>
  tools/release/android.sh` packs the validation layer; `--gpu-debug` in args.txt turns it on;
  `adb shell setprop debug.vulkan.khronos_validation.validate_sync true` adds sync checks. It
  found the swapchain loop at once; the compression bug it couldn't see (bisected with a copy of
  the pack, `--shader-pack dbg`, its `pack.cfg` switches one at a time).
- **Workgroup arrays:** three compute shaders had `ArrayStride` on workgroup arrays (naga 30), which
  the Pixel's driver did not ignore: wrong terrain (ground floating in the air, underground faces
  showing through). The backend now drops it; validation is clean apart from the rotation note.

## Build and run

On the PC everything is in `~/Android/Sdk` (command-line tools, platform 35, build tools 35.0.1,
NDK 28.2, the emulator and an Android 15 x86_64 image), with `cargo-ndk` and the two Rust targets.

```bash
# from the repository root; ABIS default arm64-v8a, PROFILE default dist
ABIS="arm64-v8a x86_64" PROFILE=release tools/release/android.sh
```

The APK lands in `dist/chiacraft-<commit>-android.apk`. On a phone: `adb install -r` it (USB
debugging on), or copy it over and allow installing from that app.

On the emulator (needs a queue hold, `t2q.py hold 15 "android emulator"`, and a display: the host
GPU won't run headless; `~/chiacraft-android/screen.sh` starts a virtual KDE screen and writes its
`DISPLAY` to `~/chiacraft-android/display`):

```bash
DISPLAY=$(cat ~/chiacraft-android/display) tools/release/android-run.sh <apk> <out> 60 --world droid1 --log frame=debug
```

`TAPS`, `SWIPES` and `KEYS` drive it (device pixels; see the script's head). Out come
screenshots, logcat, the game's log and `dumpsys meminfo`.

## What needed changing, and why

All engine changes are in `notes/ENGINE_CHANGES.md` (8b). The ones that came out of the first runs:

- **No push descriptors on the emulator.** Its Vulkan (gfxstream) is 1.3 but has no
  `VK_KHR_push_descriptor`; the backend now uses descriptor sets from per-frame pools there.
  Desktop GPUs still push. Most phones (Adreno, recent Mali) have the extension.
- **libc++_shared.so** goes in the APK: a C++ part of a dependency links it.
- **mimalloc's thread-locals** must use the dynamic model: Android refuses to load a library with
  initial-exec TLS (`local_dynamic_tls` on Android only).
- **CMake under cargo-ndk**: zlib-ng (under flate2) needs the NDK's toolchain file per CPU, and
  aws-lc-sys uses its own C builder (`AWS_LC_SYS_CMAKE_BUILDER=0`). Both set in the script.
- **No system clipboard**: arboard has no Android; copy and paste work inside the game.
- **The account** opens, but Android has no OS keystore in the wallet yet (`store/os.rs` falls
  through to "no OS keystore on this system"): where its secret is kept on Android needs a look
  (Android Keystore through JNI) before anyone's real account goes on a phone.

## What's missing for a playable phone version

1. **A real phone.** Everything above is the emulator. Things a phone may lack: `fillModeNonSolid`
   (Mali has none; the game requires it), `VK_KHR_push_descriptor` (handled), 16-bit storage,
   `D32_SFLOAT` depth (fine on both), 2048 bindless textures (`MAX_TEXTURES`). The first phone run
   tells us which.
2. **A mobile performance preset**: lower view distance (6-8), the shader pack's shadows and
   effects off or reduced, render scale below 1, a 30 fps cap option, far terrain off. The base
   game already picks its first preset by `core:gpu_class`; Android needs its own row.
3. **Touch UI, the rest**: dragging stacks (now: tap to pick up, tap to put down), long-press for
   half a stack, the on-screen keyboard for text (world names, chat; NativeActivity has no IME
   through winit 0.30, needs a little Java or GameActivity), hiding the keyboard hints the HUD
   shows ("E Inventory"), button art in the game's UI style (now plain squares), a setting for
   look speed and button size, buttons out of the way of the HUD's left column.
4. **Controllers**: winit 0.30 has no gamepad events on Android; gilrs or the activity's input
   events directly.
5. **Sound**: cpal's AAudio backend builds and the game's audio starts (all 215 sound events
   found); not listened to yet (the emulator ran with `-no-audio`).
6. **Lifecycle**: going to the background saves the world and players at once (an app killed
   there loses nothing since); not yet seen in a run that Android actually kills. Rotation is
   locked to landscape.
7. **Packaging for friends**: a release signing key (not the debug key), version codes, maybe
   an AAB for a store later. Size now ~50 MB per CPU.

## Rough plan

1. Run on one real phone Ronald has (adb over USB), fix what its GPU lacks.
2. Mobile preset and a frame-time pass on that phone (`--log frame=debug`).
3. Touch UI round two with the UI worker's art: proper buttons, drag stacks, text input.
4. Sound check; controllers.
5. A signed APK for friends (`tools/release/android.sh` with a release key).
