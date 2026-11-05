//! Prints the progression and balance page: `cargo run -p progression --bin sheet >
//! mods/base/progression/PROGRESSION_SHEET.md`.

fn main() {
    print!("{}", progression::sheet::markdown());
}
