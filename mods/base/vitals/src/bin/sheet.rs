//! Prints the survival numbers page: `cargo run -p vitals --bin sheet > mods/base/vitals/SHEET.md`.

fn main() {
    print!("{}", vitals::sheet::markdown());
}
