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

Pictures: `~/Coding/chiacraft-archive/images/android-port/2026-11-05/` (`r4` title screen,
`r5` a world with the touch controls, `r6` pause, resume, menu taps and back).

### Numbers on the emulator (not a phone)

| | title screen | in a world |
|---|---|---|
| frames | ~45 fps | ~30 fps |
| where the frame goes | waiting for the emulator's display (acquire 21 ms) | acquire 23 ms, submit 9 ms; the GPU's own work under 1 ms |
| memory (PSS / RSS) | 120 / 234 MB | 325 / 440 MB |

The frame rate is the emulator's display path, not the game: the RTX 5090 does a frame's work in
under a millisecond. A phone's numbers will be different in every way; only memory says much
(440 MB resident at view distance 12 is fine for a 6-8 GB phone).

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
6. **Lifecycle**: an app killed in the background loses unsaved play (the world saves on its
   timer; save on `suspended` too). Rotation is locked to landscape.
7. **Packaging for friends**: a release signing key (not the debug key), version codes, maybe
   an AAB for a store later. Size now ~50 MB per CPU.

## Rough plan

1. Run on one real phone Ronald has (adb over USB), fix what its GPU lacks.
2. Mobile preset and a frame-time pass on that phone (`--log frame=debug`).
3. Touch UI round two with the UI worker's art: proper buttons, drag stacks, text input.
4. Save on suspend; sound check; controllers.
5. A signed APK for friends (`tools/release/android.sh` with a release key).
