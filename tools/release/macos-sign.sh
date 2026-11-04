#!/usr/bin/env bash
# Signs a packed macOS build (tools/release/package.sh) with a Developer ID Application
# certificate and has Apple notarize it, so it opens without the "unidentified developer" stop.
#
#   tools/release/macos-sign.sh <in.tar.gz> <out.tar.gz>
#
# Needs, in the environment:
#   SIGN_IDENTITY   the certificate's name or SHA-1 in a keychain on the search list
#                   ("Developer ID Application: Ronald Goedeke (TEAMID)")
# and, to notarize (left out: signed only), one of:
#   NOTARY_KEY (path to the AuthKey_XXXX.p8), NOTARY_KEY_ID, NOTARY_ISSUER   App Store Connect API key
#   NOTARY_APPLE_ID, NOTARY_PASSWORD (app-specific), NOTARY_TEAM_ID         Apple ID
#   NOTARY_PROFILE                                                          a `notarytool store-credentials` profile
#
# Every Mach-O file is signed with the hardened runtime and a timestamp, including the mods'
# libraries inside run/mods/*.zip: Apple's notary looks into nested archives too. The game itself
# gets tools/release/macos/entitlements.plist: other people's mods are native libraries signed by
# someone else (library validation off), and the sandboxed mods' WebAssembly is compiled at run
# time (wasmtime). A bare executable can't have the notary's ticket stapled to it, so a player's
# Mac checks it online on the first start.
set -euo pipefail

in="${1:?usage: macos-sign.sh <in.tar.gz> <out.tar.gz>}"
out="${2:?usage: macos-sign.sh <in.tar.gz> <out.tar.gz>}"
: "${SIGN_IDENTITY:?set SIGN_IDENTITY to the Developer ID Application identity}"
here="$(cd "$(dirname "$0")" && pwd)"
entitlements="$here/macos/entitlements.plist"
in="$(cd "$(dirname "$in")" && pwd)/$(basename "$in")"
mkdir -p "$(dirname "$out")"
out="$(cd "$(dirname "$out")" && pwd)/$(basename "$out")"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
tar -xzf "$in" -C "$work"
game="$work/chiacraft"
[ -f "$game/weft" ] || { echo "no chiacraft/weft in $in" >&2; exit 1; }

sign() { codesign --force --timestamp --options runtime --sign "$SIGN_IDENTITY" "$@"; }

# The mods: unpack, sign each library, pack again with the same paths (no directory entries).
for zip in "$game"/run/mods/*.zip; do
	dir="$work/mod-$(basename "$zip" .zip)"
	mkdir "$dir"
	unzip -q "$zip" -d "$dir"
	found=0
	while IFS= read -r -d '' lib; do
		sign "$lib"
		found=1
	done < <(find "$dir" -type f \( -name '*.dylib' -o -name '*.so' \) -print0)
	[ "$found" = 1 ] || echo "note: no library in $(basename "$zip")"
	rm "$zip"
	(cd "$dir" && zip -q -r -X -D "$zip" .)
done

[ ! -f "$game/libMoltenVK.dylib" ] || sign "$game/libMoltenVK.dylib"
sign --identifier com.chiacraft.weft --entitlements "$entitlements" "$game/weft"

codesign --verify --strict --verbose=2 "$game/weft"
[ ! -f "$game/libMoltenVK.dylib" ] || codesign --verify --strict --verbose=2 "$game/libMoltenVK.dylib"
codesign -dvv "$game/weft" 2>&1 | grep -E '^(Authority|TeamIdentifier|Timestamp|Runtime)' || true

notarize=()
if [ -n "${NOTARY_KEY:-}" ]; then
	notarize=(--key "$NOTARY_KEY" --key-id "${NOTARY_KEY_ID:?}" --issuer "${NOTARY_ISSUER:?}")
elif [ -n "${NOTARY_APPLE_ID:-}" ]; then
	notarize=(--apple-id "$NOTARY_APPLE_ID" --password "${NOTARY_PASSWORD:?}" --team-id "${NOTARY_TEAM_ID:?}")
elif [ -n "${NOTARY_PROFILE:-}" ]; then
	notarize=(--keychain-profile "$NOTARY_PROFILE")
fi

state="signed (Developer ID), not notarized"
if [ ${#notarize[@]} -gt 0 ]; then
	ditto -c -k --keepParent "$game" "$work/notarize.zip"
	xcrun notarytool submit "$work/notarize.zip" "${notarize[@]}" --wait --timeout 1h \
		--output-format json > "$work/notary.json" || true
	cat "$work/notary.json"
	id="$(plutil -extract id raw -o - "$work/notary.json" 2>/dev/null || true)"
	status="$(plutil -extract status raw -o - "$work/notary.json" 2>/dev/null || true)"
	if [ "$status" != Accepted ]; then
		[ -z "$id" ] || xcrun notarytool log "$id" "${notarize[@]}" || true
		echo "notarization: ${status:-no answer}" >&2
		exit 1
	fi
	state="signed (Developer ID) and notarized"
fi
echo "macOS: $state" >> "$game/BUILD.txt"

tar -czf "$out" -C "$work" chiacraft
echo "$state: $out"
