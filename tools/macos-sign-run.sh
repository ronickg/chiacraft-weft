#!/bin/sh
# Cargo's runner on macOS (.cargo/config.toml): signs the game with the first Apple Development
# identity in the keychain before running it. An unsigned build is a new app to macOS every
# time, so the Keychain asked again for the account key after each rebuild; signed, "Always
# Allow" sticks. WEFT_SIGN_IDENTITY picks another identity; without one the game runs unsigned.
# Anything other than the game (tests, examples) runs as it is.
bin="$1"
if [ "$(basename "$bin")" = weft ]; then
	id="${WEFT_SIGN_IDENTITY:-$(security find-identity -v -p codesigning 2>/dev/null | sed -n 's/.*"\(Apple Development:[^"]*\)".*/\1/p' | head -n 1)}"
	if [ -n "$id" ] && ! codesign -dvv "$bin" 2>&1 | grep -qF "Authority=$id"; then
		codesign --force --sign "$id" --identifier com.chiacraft.weft "$bin" 2>/dev/null || echo "note: could not sign $bin" >&2
	fi
fi
exec "$@"
