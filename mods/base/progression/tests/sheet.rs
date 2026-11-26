//! The progression page (`PROGRESSION_SHEET.md`) is what the code says.

#[test]
fn the_sheet_is_up_to_date() {
    let on_disk = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/PROGRESSION_SHEET.md")).unwrap_or_default();
    assert!(
        on_disk == progression::sheet::markdown(),
        "mods/base/progression/PROGRESSION_SHEET.md is out of date: cargo run -p progression --bin sheet > mods/base/progression/PROGRESSION_SHEET.md"
    );
}
