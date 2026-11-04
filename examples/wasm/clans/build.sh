#!/bin/sh
# Builds the clans mod and writes it into its package as code/server.wasm.
# Needs `rustup target add wasm32-unknown-unknown`.
set -e
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
cargo build --release --target wasm32-unknown-unknown --manifest-path "$here/Cargo.toml"
cargo run -q --release -p chiacraft-sandbox --features pack --bin componentize --manifest-path "$root/Cargo.toml" -- \
    "$here/target/wasm32-unknown-unknown/release/clans.wasm" "$root/examples/packages/clans/code/server.wasm"
