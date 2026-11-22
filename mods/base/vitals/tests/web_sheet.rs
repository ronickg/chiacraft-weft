//! The skills and perk web page (`WEB_SHEET.md`) is what the code says.

#[test]
fn web_sheet_is_current() {
    let on_disk = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/WEB_SHEET.md")).unwrap_or_default();
    assert!(
        on_disk == vitals::web_sheet::markdown(),
        "mods/base/vitals/WEB_SHEET.md is out of date: cargo run -p vitals --bin web_sheet > mods/base/vitals/WEB_SHEET.md"
    );
}
