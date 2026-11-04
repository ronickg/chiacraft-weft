//! Prints the crafting numbers page: `cargo run -p crafting --bin sheet > mods/base/crafting/SHEET.md`.

fn main() {
    print!("{}", crafting::sheet::markdown());
}
