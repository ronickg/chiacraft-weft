#!/usr/bin/env bash
# Packs a built game into one archive a player unpacks and runs: the `weft` executable, the
# engine's assets (assets/core), our mods each packed into run/mods/<mod>.zip (base carries the
# chiacraft shader pack inside its library and writes it out on the first start), and on macOS
# MoltenVK, which stands in for Vulkan there.
#
# It packs the build in target/$PROFILE (default dist, what players get; PROFILE=release packs
# a crew build). With TARGET set (a Rust target triple, e.g. x86_64-pc-windows-gnu for Windows
# cross-compiled on a Mac), it packs target/$TARGET/$PROFILE with tools/release/cross-pack.py,
# which makes the same folder without running the game.
#
#   cargo build --profile dist
#   tools/release/package.sh <platform> [label]
#
# <platform> is linux-x64, windows-x64, macos-arm64 or macos-x64 and ends up in the file name;
# [label] (default: the commit) too: chiacraft-<label>-<platform>.tar.gz (.zip on Windows) in
# $OUT_DIR (default dist/). On macOS set MOLTENVK_DYLIB (and MOLTENVK_LICENSE) to the
# libMoltenVK.dylib to bundle. Run it from the repository root. It runs the game only with
# `--dist`, which writes the folder and quits: no window, no GPU.
#
# Prints the archive's path last, and writes `archive=` and `name=` to $GITHUB_OUTPUT in CI.
set -euo pipefail

platform="${1:?usage: tools/release/package.sh <platform> [label]}"
# The commit checked out (on a run by hand with another `ref`, GITHUB_SHA is the workflow's, not
# this one), else GitHub's, else none (a tree without .git, as on the test PC).
commit="$(git rev-parse HEAD 2> /dev/null || echo "${GITHUB_SHA:-unknown}")"
label="${2:-${commit:0:8}}"
out_dir="${OUT_DIR:-dist}"
name="chiacraft-$label-$platform"
folder="$out_dir/chiacraft"

case "$platform" in
	windows-*) exe=weft.exe ;;
	*) exe=weft ;;
esac
profile="${PROFILE:-dist}"
build="${CARGO_TARGET_DIR:-target}${TARGET:+/$TARGET}/$profile"
bin="$build/$exe"
[ -f Cargo.toml ] && [ -d mods/base ] || { echo "run from the repository root" >&2; exit 1; }
[ -x "$bin" ] || [ -f "$bin" ] || { echo "$bin is missing: cargo build --profile $profile${TARGET:+ --target $TARGET} first" >&2; exit 1; }

rm -rf "$folder"
mkdir -p "$out_dir"
# The engine's own packer: the executable, assets/core and every mod not marked `dist = false`
# (the test kit, testmod), each with its library from beside the executable (target/$profile).
if [ -n "${TARGET:-}" ]; then
	python3 -I tools/release/cross-pack.py "$build" "$exe" "$folder"
else
	"$bin" --dist "$folder"
fi

# What a player's copy needs, checked here rather than on their machine.
fail() { echo "package: $*" >&2; exit 1; }
[ -f "$folder/$exe" ] || fail "no $exe in $folder"
[ -f "$folder/assets/core/icon.png" ] || fail "no engine assets in $folder/assets/core"
# Every mod in mods/ is packed, except the development ones marked `dist = false`.
[ -f "$folder/run/mods/base.zip" ] || fail "base was not packed"
for dir in mods/*/; do
	m=$(basename "$dir")
	[ -f "$dir/Cargo.toml" ] || continue
	if tr -d ' ' < "$dir/Cargo.toml" | grep -qx 'dist=false'; then
		[ ! -e "$folder/run/mods/$m.zip" ] || fail "development mod $m was packed"
	else
		[ -f "$folder/run/mods/$m.zip" ] || fail "mod $m was not packed (is its library missing from target/$profile?)"
	fi
done
ls "$folder/run/mods"

if [[ "$platform" == macos-* ]]; then
	[ -f "${MOLTENVK_DYLIB:-}" ] || fail "set MOLTENVK_DYLIB to the libMoltenVK.dylib to bundle"
	cp "$MOLTENVK_DYLIB" "$folder/libMoltenVK.dylib"
	[ -z "${MOLTENVK_LICENSE:-}" ] || cp "$MOLTENVK_LICENSE" "$folder/MoltenVK-LICENSE"
fi

cp LICENSE "$folder/LICENSE"
cp tools/release/README.txt "$folder/README.txt"
{
	echo "chiacraft $label ($platform)"
	echo "commit $commit"
	echo "built $(date -u +%Y-%m-%dT%H:%MZ)"
	[ -z "${TARGET:-}" ] || echo "target $TARGET, cross-compiled on $(uname -s)"
} > "$folder/BUILD.txt"

# Windows gets a zip (Explorer opens it); the others a tar, which keeps the permission to run.
if [[ "$platform" == windows-* ]]; then
	archive="$out_dir/$name.zip"
	rm -f "$archive"
	if command -v 7z > /dev/null; then
		(cd "$out_dir" && 7z a -tzip -bso0 "$name.zip" chiacraft)
	elif command -v zip > /dev/null; then
		(cd "$out_dir" && zip -q -r -X "$name.zip" chiacraft)
	else
		pwsh -NoProfile -Command "Compress-Archive -Path '$folder' -DestinationPath '$archive'"
	fi
else
	archive="$out_dir/$name.tar.gz"
	tar -czf "$archive" -C "$out_dir" chiacraft
fi

du -sh "$folder" "$archive"
if [ -n "${GITHUB_OUTPUT:-}" ]; then
	echo "archive=$archive" >> "$GITHUB_OUTPUT"
	echo "name=$name" >> "$GITHUB_OUTPUT"
fi
echo "$archive"
