//! Definition files from another player: reading any of the kinds never panics.
#![no_main]

use chiacraft_packages::{BlockDef, ItemDef, Manifest, RecipeDef, TagDef};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    let _ = toml::from_str::<Manifest>(text);
    let _ = toml::from_str::<BlockDef>(text);
    let _ = toml::from_str::<ItemDef>(text);
    let _ = toml::from_str::<RecipeDef>(text);
    let _ = toml::from_str::<TagDef>(text);
});
