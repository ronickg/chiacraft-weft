chiacraft
=========

A block game on the Weft engine. Needs a GPU with Vulkan (on a Mac it comes with the game).
BUILD.txt says which commit this is.

Windows (x64)
  Unpack the zip anywhere and run chiacraft\weft.exe. Windows SmartScreen may say it doesn't
  know the app, as it is not signed: "More info", then "Run anyway".

Linux (x64)
  tar xf chiacraft-*-linux-x64.tar.gz, then run chiacraft/weft.
  Needs your GPU's Vulkan driver (Mesa, or NVIDIA's own), the Vulkan loader (libvulkan1 /
  vulkan-icd-loader), ALSA (libasound2) and X11 or Wayland with libxkbcommon.

macOS (Apple silicon: macos-arm64; Intel: macos-x64), macOS 12 or newer
  tar xf chiacraft-*-macos-*.tar.gz (or double-click it), then run chiacraft/weft.
  An unsigned build is stopped by macOS the first time:
    - either, in Terminal:  xattr -dr com.apple.quarantine chiacraft
    - or try to open weft once, then System Settings > Privacy & Security > "Open Anyway"
      (on macOS 14 and older, Control-click weft > Open works too).
  A signed and notarized build (its BUILD.txt says so) opens without this.

Where the game keeps your worlds, settings and logs
  Windows: %APPDATA%\Weft   Linux: ~/.local/share/weft   macOS: ~/Library/Application Support/Weft
