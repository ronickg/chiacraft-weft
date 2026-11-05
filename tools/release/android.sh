#!/usr/bin/env bash
# Builds chiacraft for Android and packs a debug-signed APK, without Gradle, Android Studio or CI:
# cargo-ndk builds the game (crates/android, libchiacraft.so, the NativeActivity's library) and
# every mod not marked `dist = false` as .so files per CPU; aapt2, zipalign and apksigner from the
# SDK's build tools make the APK. Set up once (the test PC has it in ~/Android/Sdk):
#
#   SDK command-line tools from developer.android.com, then
#   sdkmanager "platform-tools" "platforms;android-35" "build-tools;35.0.1" "ndk;28.2.13676358"
#   rustup target add aarch64-linux-android x86_64-linux-android
#   cargo install cargo-ndk
#
#   tools/release/android.sh [label]
#
# ABIS: the CPUs, "arm64-v8a" (phones, the default) and/or "x86_64" (the emulator on a PC).
# PROFILE: dist (default, what players get) or release (quicker, the crew's).
# ANDROID_HOME (default ~/Android/Sdk), ANDROID_NDK_HOME (default its newest ndk/), JOBS (4).
# The APK lands in $OUT_DIR (default dist/) as chiacraft-<label>-android.apk, signed with a debug
# key made once in ~/.android/debug.keystore (as Android Studio does). Install: adb install -r.
# Niced, so the machine stays usable. Run it from the repository root.
set -euo pipefail

[ -f Cargo.toml ] && [ -d mods/base ] || { echo "run from the repository root" >&2; exit 1; }
sdk="${ANDROID_HOME:-$HOME/Android/Sdk}"
[ -d "$sdk/platforms" ] || { echo "no Android SDK in $sdk (set ANDROID_HOME)" >&2; exit 1; }
ndk="${ANDROID_NDK_HOME:-$(ls -d "$sdk"/ndk/* 2> /dev/null | sort -V | tail -1)}"
[ -d "$ndk" ] || { echo "no NDK: sdkmanager \"ndk;28.2.13676358\"" >&2; exit 1; }
tools="$(ls -d "$sdk"/build-tools/* | sort -V | tail -1)"
api="${ANDROID_PLATFORM:-35}"
jar="$sdk/platforms/android-$api/android.jar"
[ -f "$jar" ] || { echo "no $jar: sdkmanager \"platforms;android-$api\"" >&2; exit 1; }
command -v cargo-ndk > /dev/null || { echo "cargo install cargo-ndk" >&2; exit 1; }
# Android 11: AAudio, and Vulkan 1.3 drivers come with Android 13 or later anyway.
min_sdk=30
profile="${PROFILE:-dist}"
abis="${ABIS:-arm64-v8a}"
commit="${COMMIT:-$(git rev-parse HEAD 2> /dev/null || echo unknown)}"
label="${1:-${commit:0:8}}"
out_dir="${OUT_DIR:-dist}"
stage="$out_dir/android-stage"
apk="$out_dir/chiacraft-$label-android.apk"
target_dir="${CARGO_TARGET_DIR:-target}"

triple() {
	case "$1" in
		arm64-v8a) echo aarch64-linux-android ;;
		x86_64) echo x86_64-linux-android ;;
		armeabi-v7a) echo armv7-linux-androideabi ;;
		x86) echo i686-linux-android ;;
		*) echo "unknown ABI $1" >&2; exit 1 ;;
	esac
}
for abi in $abis; do
	rustup target list --installed | grep -qx "$(triple "$abi")" || { echo "rustup target add $(triple "$abi")" >&2; exit 1; }
done

# The game's files first (it also lists the mods to build).
stamp="$label-$(date -u +%Y%m%d%H%M%S)"
python3 -I tools/release/android-pack.py "$stage" "$stamp"
packages=(-p chiacraft-android)
while read -r m; do packages+=(-p "$m"); done < "$stage/mods.txt"

targets=()
for abi in $abis; do targets+=(-t "$abi"); done
export ANDROID_NDK_HOME="$ndk" ANDROID_HOME="$sdk"
# aws-lc-sys (the wallet's TLS) builds Android with CMake by default, which can't find a make
# program under cargo-ndk; its own C builder needs nothing more.
export AWS_LC_SYS_CMAKE_BUILDER=0
# C libraries built with CMake (zlib-ng under flate2) get the NDK's toolchain, set for each CPU
# (its file alone would build 32-bit ARM). The cmake crate knows it by this file name.
for abi in $abis; do
	t="$(triple "$abi")"
	file="$PWD/$stage/cmake-$abi/android.toolchain.cmake"
	mkdir -p "$(dirname "$file")"
	printf 'set(ANDROID_ABI %s)\nset(ANDROID_PLATFORM android-%s)\ninclude(%s/build/cmake/android.toolchain.cmake)\n' "$abi" "$min_sdk" "$ndk" > "$file"
	export "CMAKE_TOOLCHAIN_FILE_${t//-/_}=$file"
done
export CARGO_PROFILE_DIST_DEBUG=0 CARGO_PROFILE_DIST_STRIP=debuginfo
export CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_PROFILE_RELEASE_STRIP=debuginfo
nice -n 19 cargo ndk "${targets[@]}" -P "$min_sdk" build --profile "$profile" -j "${JOBS:-4}" "${packages[@]}"

# Native libraries: lib/<abi>/libchiacraft.so and each mod's lib<mod>.so beside it.
for abi in $abis; do
	built="$target_dir/$(triple "$abi")/$profile"
	mkdir -p "$stage/apk/lib/$abi"
	for name in chiacraft $(cat "$stage/mods.txt"); do
		[ -f "$built/lib$name.so" ] || { echo "android: $built/lib$name.so was not built" >&2; exit 1; }
		cp "$built/lib$name.so" "$stage/apk/lib/$abi/"
	done
	# C++ parts of dependencies link the NDK's shared C++ library, which the APK must carry.
	cxx="$ndk/toolchains/llvm/prebuilt/$(uname -s | tr A-Z a-z)-x86_64/sysroot/usr/lib/$(triple "$abi" | sed 's/armv7/arm/')/libc++_shared.so"
	[ -f "$cxx" ] || { echo "android: no $cxx" >&2; exit 1; }
	cp "$cxx" "$stage/apk/lib/$abi/"
done

version_code="$(git rev-list --count HEAD 2> /dev/null || echo 1)"
"$tools/aapt2" compile --dir "$stage/res" -o "$stage/res.zip"
"$tools/aapt2" link -o "$stage/unaligned.apk" \
	--manifest tools/release/android/AndroidManifest.xml -I "$jar" \
	-A "$stage/apk-assets" -R "$stage/res.zip" --auto-add-overlay \
	--min-sdk-version "$min_sdk" --target-sdk-version "$api" \
	--version-code "$version_code" --version-name "$label" \
	--debug-mode -0 zip -0 stamp
python3 -I - "$stage/unaligned.apk" "$stage/apk" << 'PY'
import sys, zipfile
from pathlib import Path
apk, root = sys.argv[1], Path(sys.argv[2])
with zipfile.ZipFile(apk, "a", zipfile.ZIP_DEFLATED) as z:
    for f in sorted(root.rglob("*.so")):
        z.write(f, f.relative_to(root).as_posix())
PY
"$tools/zipalign" -f -P 16 4 "$stage/unaligned.apk" "$stage/aligned.apk"

keystore="${ANDROID_KEYSTORE:-$HOME/.android/debug.keystore}"
if [ ! -f "$keystore" ]; then
	mkdir -p "$(dirname "$keystore")"
	keytool -genkeypair -keystore "$keystore" -storepass android -keypass android -alias androiddebugkey \
		-keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Android Debug,O=Android,C=US"
fi
mkdir -p "$out_dir"
"$tools/apksigner" sign --ks "$keystore" --ks-pass pass:android --key-pass pass:android --out "$apk" "$stage/aligned.apk"
"$tools/apksigner" verify "$apk"
du -h "$apk"
echo "$apk"
