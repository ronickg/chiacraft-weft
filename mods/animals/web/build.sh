#!/bin/sh
# Builds the animals' logic for the website (WASM) and copies it to <dir> (default: here).
#   mods/animals/web/build.sh [dir]
set -e
cd "$(dirname "$0")/../../.."
CARGO_PROFILE_RELEASE_DEBUG=false CARGO_PROFILE_RELEASE_STRIP=true cargo build --release -p animals_web --target wasm32-unknown-unknown
out="${1:-mods/animals/web}"
cp target/wasm32-unknown-unknown/release/animals_web.wasm "$out/animals.wasm"
echo "wrote $out/animals.wasm ($(wc -c < "$out/animals.wasm" | tr -d ' ') bytes)"
