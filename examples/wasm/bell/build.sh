#!/bin/sh
# Builds the bell mod and writes it into its package as code/server.wasm.
# Needs `rustup target add wasm32-unknown-unknown`.
set -e
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
cargo build --target wasm32-unknown-unknown --manifest-path "$here/Cargo.toml"
cargo run -q -p chiacraft-sandbox --features pack --bin componentize --manifest-path "$root/Cargo.toml" -- \
    "$here/target/wasm32-unknown-unknown/debug/bell.wasm" "$root/examples/packages/bell/code/server.wasm"
