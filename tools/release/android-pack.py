#!/usr/bin/env python3
"""Makes the game's files for the Android APK (tools/release/android.sh):

    tools/release/android-pack.py <stage> <stamp>

<stage>/apk-assets/game.zip   the folder `weft dist` makes, without the executable: the
                              engine's assets in assets/core and every mod not marked
                              `dist = false` packed into run/mods/<mod>.zip, its assets only
<stage>/apk-assets/game.stamp <stamp>: the game unpacks game.zip again when it changes
<stage>/res/                  tools/release/android/res (the theme, the splash picture) and
                              mipmap-*/ic_launcher.png, the launcher icon (the engine's icon.png)

The mods' libraries don't go in their zips: the APK carries them per CPU in lib/<abi>/ beside
libchiacraft.so, and the game looks there (crates/android, `paths::set_library_dir`), so one
game.zip serves every CPU. Run it from the repository root.
"""

import os
import shutil
import sys
import zipfile
from pathlib import Path


def dist_mods(root: Path) -> list[Path]:
    def marked_dev(mod: Path) -> bool:
        toml = mod / "Cargo.toml"
        return toml.is_file() and any(line.replace(" ", "") == "dist=false" for line in toml.read_text().splitlines())

    # ANDROID_TESTKIT=1: the test kit too, for scripted runs on a phone (`--script`).
    extra = {"testkit"} if os.environ.get("ANDROID_TESTKIT") == "1" else set()
    return [
        m
        for m in sorted((root / "mods").iterdir())
        if m.is_dir() and (m / "Cargo.toml").is_file() and (not marked_dev(m) or m.name in extra)
    ]


def add_tree(z: zipfile.ZipFile, src: Path, prefix: str) -> int:
    n = 0
    for f in sorted(src.rglob("*")):
        if f.is_file() and f.name != ".DS_Store":
            z.write(f, f"{prefix}/{f.relative_to(src).as_posix()}")
            n += 1
    return n


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    stage, stamp = Path(sys.argv[1]), sys.argv[2]
    root = Path.cwd()
    if not (root / "Cargo.toml").is_file() or not (root / "mods" / "base").is_dir():
        sys.exit("android-pack: run from the repository root")
    assets = stage / "apk-assets"
    shutil.rmtree(stage, ignore_errors=True)
    assets.mkdir(parents=True)
    with zipfile.ZipFile(assets / "game.zip", "w", zipfile.ZIP_DEFLATED) as game:
        n = add_tree(game, root / "crates" / "engine" / "assets", "assets/core")
        for mod in dist_mods(root):
            # A mod's zip as `modloader::pack` makes it, without the library.
            packed = stage / "mods" / f"{mod.name}.zip"
            packed.parent.mkdir(parents=True, exist_ok=True)
            with zipfile.ZipFile(packed, "w", zipfile.ZIP_DEFLATED) as z:
                if (mod / "assets").is_dir():
                    add_tree(z, mod / "assets", "assets")
            # Stored: it is a zip already.
            game.write(packed, f"run/mods/{mod.name}.zip", compress_type=zipfile.ZIP_STORED)
            n += 1
            print(f"  mod {mod.name}: {packed.stat().st_size // 1024} KB")
    (assets / "game.stamp").write_text(stamp + "\n")
    print(f"game.zip: {n} files, {(assets / 'game.zip').stat().st_size // (1024 * 1024)} MB")
    # The theme and the splash picture (tools/release/android/res), then the launcher icon.
    shutil.copytree(root / "tools" / "release" / "android" / "res", stage / "res")
    icon = root / "crates" / "engine" / "assets" / "icon.png"
    for density in ["mdpi", "hdpi", "xhdpi", "xxhdpi", "xxxhdpi"]:
        d = stage / "res" / f"mipmap-{density}"
        d.mkdir(parents=True, exist_ok=True)
        shutil.copy2(icon, d / "ic_launcher.png")
    # The names of the mods to build, for android.sh.
    (stage / "mods.txt").write_text("".join(f"{m.name}\n" for m in dist_mods(root)))


if __name__ == "__main__":
    main()
