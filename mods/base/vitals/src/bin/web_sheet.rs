//! Prints the skills and perk web page: `cargo run -p vitals --bin web_sheet > mods/base/vitals/WEB_SHEET.md`.

fn main() {
    print!("{}", vitals::web_sheet::markdown());
}
