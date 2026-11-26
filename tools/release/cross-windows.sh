#!/usr/bin/env bash
# Builds and packs the Windows x64 game on a Mac (or Linux) without GitHub Actions: the GNU
# toolchain (mingw-w64) instead of MSVC, so nothing needs Microsoft's SDK or its licence.
#
#   rustup target add x86_64-pc-windows-gnu
#   brew install mingw-w64 nasm            # Linux: the distribution's mingw-w64 and nasm
#   tools/release/cross-windows.sh [label]
#
# Same as CI: the dist profile, no debug info, then tools/release/package.sh, which packs with
# tools/release/cross-pack.py (a Windows weft.exe can't run here to do `weft dist`). The archive
# lands in $OUT_DIR (default dist/). Niced and 4 jobs, so the machine stays usable.
set -euo pipefail

target=x86_64-pc-windows-gnu
[ -f Cargo.toml ] && [ -d mods/base ] || { echo "run from the repository root" >&2; exit 1; }
command -v x86_64-w64-mingw32-gcc > /dev/null || { echo "no mingw-w64: brew install mingw-w64" >&2; exit 1; }
rustup target list --installed | grep -qx "$target" || { echo "rustup target add $target" >&2; exit 1; }

# aws-lc-sys (the wallet's TLS) runs bindgen, whose clang needs mingw's C headers.
include="$(dirname "$(x86_64-w64-mingw32-gcc -print-file-name=include/stdlib.h 2> /dev/null)")"
[ -f "$include/stdlib.h" ] || include="$(x86_64-w64-mingw32-gcc -print-sysroot)/x86_64-w64-mingw32/include"
[ -f "$include/stdlib.h" ] || { echo "can't find mingw's stdlib.h" >&2; exit 1; }
export BINDGEN_EXTRA_CLANG_ARGS_x86_64_pc_windows_gnu="--target=x86_64-w64-mingw32 -isystem $include"
export CARGO_PROFILE_DIST_DEBUG=0 CARGO_PROFILE_DIST_STRIP=debuginfo

nice -n 19 cargo build --profile dist --target "$target" -j "${JOBS:-4}"
TARGET="$target" tools/release/package.sh windows-x64 "${1:-}"
