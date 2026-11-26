//! Prints the species page natively, the same text the WASM's `sheet` op gives:
//! `cargo run -p animals_core --bin sheet > mods/animals/SPECIES.md` (no WASM build needed).

fn main() {
    print!("{}", animals_core::sheet::markdown());
}
