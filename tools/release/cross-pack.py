#!/usr/bin/env python3
"""Makes the folder `weft dist` makes, for a build for another system that can't run here (a
Windows build cross-compiled on a Mac): the executable, the engine's assets in assets/core, and
every mod not marked `dist = false` packed into run/mods/<mod>.zip with its library.

    tools/release/cross-pack.py <build dir> <exe name> <out folder>

e.g. tools/release/cross-pack.py target/x86_64-pc-windows-gnu/dist weft.exe dist/chiacraft

Run it from the repository root. It mirrors crates/app/src/main.rs `make_dist` (including
mod-abi.txt, the hash crates/abi computes from its own source) and
crates/engine/src/modloader.rs `pack`: each zip holds the mod's library at the top (the file name
the game looks for on that system: <mod>.dll, lib<mod>.so or lib<mod>.dylib), then everything
under the mod's assets/ at the same paths. Unlike the engine it leaves out .DS_Store files.
"""

import re
import shutil
import sys
import zipfile
from pathlib import Path


def library(build: Path, exe: str, name: str) -> Path:
    if exe.endswith(".exe"):
        file = f"{name}.dll"
    elif "apple-darwin" in str(build):
        file = f"lib{name}.dylib"
    else:
        file = f"lib{name}.so"
    return build / file


def abi_version(root: Path) -> str:
    """crates/abi's ABI_VERSION: FNV-1a over the files its `concat!(include_str!(...))` names."""
    src = root / "crates" / "abi" / "src"
    line = next(l for l in (src / "lib.rs").read_text().splitlines() if "pub const ABI_VERSION" in l)
    h = 0xCBF29CE484222325
    for name in re.findall(r'include_str!\("([^"]+)"\)', line):
        for b in (src / name).read_bytes():
            h = ((h ^ b) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"{h:016x}"


def marked_dev(mod: Path) -> bool:
    toml = mod / "Cargo.toml"
    if not toml.is_file():
        return False
    return any(line.replace(" ", "") == "dist=false" for line in toml.read_text().splitlines())


def pack(mod: Path, lib: Path, out: Path) -> Path:
    if not lib.is_file():
        sys.exit(f"cross-pack: {lib} is not built")
    out.mkdir(parents=True, exist_ok=True)
    path = out / f"{mod.name}.zip"
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.write(lib, lib.name)
        assets = mod / "assets"
        if assets.is_dir():
            for f in sorted(assets.rglob("*")):
                if f.is_file() and f.name != ".DS_Store":
                    z.write(f, f.relative_to(mod).as_posix())
    return path


def main() -> None:
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    build, exe, out = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
    root = Path.cwd()
    if not (root / "Cargo.toml").is_file() or not (root / "mods" / "base").is_dir():
        sys.exit("cross-pack: run from the repository root")
    if not (build / exe).is_file():
        sys.exit(f"cross-pack: {build / exe} is not built")
    out.mkdir(parents=True, exist_ok=True)
    shutil.copy2(build / exe, out / exe)
    (out / "mod-abi.txt").write_text(abi_version(root))
    shutil.copytree(
        root / "crates" / "engine" / "assets",
        out / "assets" / "core",
        ignore=shutil.ignore_patterns(".DS_Store"),
        dirs_exist_ok=True,
    )
    for mod in sorted(p for p in (root / "mods").iterdir() if p.is_dir()):
        if marked_dev(mod):
            continue
        print(pack(mod, library(build, exe, mod.name), out / "run" / "mods"))


if __name__ == "__main__":
    main()
