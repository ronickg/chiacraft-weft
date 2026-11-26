#!/usr/bin/env bash
# Makes the DLSS test kit for Windows (on the Linux PC): one folder with the game cross-compiled
# for Windows (with the `streamline` feature), NVIDIA's Streamline runtime beside it, Bliss's
# imported world, the scripts, the runner and the checklist.
#
#   tools/dlss-kit/make_kit.sh <build dir> <streamline zip> <mc-world dir> <kit dir>
#   e.g. tools/dlss-kit/make_kit.sh ~/win/target/x86_64-pc-windows-gnu/release \
#        ~/sdk/streamline-2.14.16/streamline-sdk-v2.14.16.zip ~/mc-world ~/win/chiacraft-dlss-kit
#
# Run it from the repository root of the commit that was built. NVIDIA's files come only from the
# official Streamline release zip (github.com/NVIDIA-RTX/Streamline): its production DLLs go to
# chiacraft/streamline/, the development ones to chiacraft/streamline-dev/, with NVIDIA's licence
# texts. None of it is ever committed.
set -euo pipefail

build="${1:?build dir}"
zip="${2:?streamline zip}"
mcworld="${3:?mc-world dir}"
kit="${4:?kit dir}"
[ -f Cargo.toml ] && [ -d mods/base ] || { echo "run from the repository root" >&2; exit 1; }
[ -f "$build/weft.exe" ] || { echo "$build/weft.exe is not built" >&2; exit 1; }

rm -rf "$kit"
mkdir -p "$kit"
game="$kit/chiacraft"
# The game folder as a player's: weft.exe, assets/core and the mods (tools/release/cross-pack.py),
# plus the test kit mod the scripts need (left out of players' copies).
python3 -I tools/release/cross-pack.py "$build" weft.exe "$game"
python3 -I - "$build" "$game" <<'EOF'
import sys
from pathlib import Path
sys.path.insert(0, "tools/release")
import importlib.util
spec = importlib.util.spec_from_file_location("cross_pack", "tools/release/cross-pack.py")
cp = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cp)
build, game = Path(sys.argv[1]), Path(sys.argv[2])
print(cp.pack(Path("mods/testkit"), cp.library(build, "weft.exe", "testkit"), game / "run" / "mods"))
EOF
# mingw's runtime DLLs the build imports (libstdc++ and what it needs), found in mingw's own
# folder and followed through their own imports.
mingw_bin=$(dirname "$(x86_64-w64-mingw32-gcc -print-file-name=libstdc++-6.dll)")
[ -f "$mingw_bin/libstdc++-6.dll" ] || mingw_bin=/usr/x86_64-w64-mingw32/bin
imports() { x86_64-w64-mingw32-objdump -p "$1" | sed -n 's/^\s*DLL Name: //p'; }
todo=("$game/weft.exe" "$build"/*.dll)
while [ ${#todo[@]} -gt 0 ]; do
	f=${todo[0]}
	todo=("${todo[@]:1}")
	for dll in $(imports "$f"); do
		if [ -f "$mingw_bin/$dll" ] && [ ! -f "$game/$dll" ]; then
			cp "$mingw_bin/$dll" "$game/"
			echo "copied $dll"
			todo+=("$game/$dll")
		fi
	done
done
echo "weft.exe imports:"; x86_64-w64-mingw32-objdump -p "$game/weft.exe" | grep "DLL Name" | sed 's/^/  /'

# NVIDIA's runtime: production (signed, what a release would carry) and development.
tmp=$(mktemp -d)
unzip -q "$zip" 'bin/x64/*' 'license.txt' -d "$tmp"
mkdir -p "$game/streamline" "$game/streamline-dev"
for f in sl.interposer.dll sl.common.dll sl.dlss.dll sl.dlss_d.dll nvngx_dlss.dll nvngx_dlssd.dll nvngx_dlss.license.txt; do
	cp "$tmp/bin/x64/$f" "$game/streamline/"
	cp "$tmp/bin/x64/development/$f" "$game/streamline-dev/"
done
cp "$tmp/license.txt" "$game/streamline/streamline-license.txt"
cp "$tmp/license.txt" "$game/streamline-dev/streamline-license.txt"
rm -rf "$tmp"

# Bliss's world (Ronald's own copy, local only), imported by the game into each fresh world.
cp -r "$mcworld" "$kit/mc-world"

here=$(dirname "$0")
cp "$here/run_kit.ps1" "$here/kit_mcworld.txt" "$here/kit_spotc.txt" "$here/copy-to-windows.sh" "$here/fetch-results.sh" "$kit/"
cp "$here/CHECKLIST.txt" "$kit/CHECKLIST.txt"
{
	echo "chiacraft DLSS kit"
	echo "commit $(git rev-parse HEAD 2> /dev/null || cat .commit 2> /dev/null || echo unknown)"
	echo "made $(date -Iseconds) on $(hostname)"
	echo "streamline zip $(basename "$zip") sha256 $(sha256sum "$zip" | cut -d' ' -f1)"
} > "$kit/BUILD.txt"
du -sh "$kit"
