//! The species page (`mods/animals/SPECIES.md`) is what the species table says.

#[test]
fn the_species_sheet_is_up_to_date() {
    let on_disk = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../SPECIES.md")).unwrap_or_default();
    assert!(
        on_disk == animals_core::sheet::markdown(),
        "mods/animals/SPECIES.md is out of date: mods/animals/web/build.sh && node mods/animals/web/sheet.mjs"
    );
}
