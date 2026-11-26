//! The survival numbers page (`SHEET.md`) is what the code says.

#[test]
fn the_sheet_is_up_to_date() {
    let on_disk = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/SHEET.md")).unwrap_or_default();
    assert!(
        on_disk == vitals::sheet::markdown(),
        "mods/base/vitals/SHEET.md is out of date: cargo run -p vitals --bin sheet > mods/base/vitals/SHEET.md"
    );
}
