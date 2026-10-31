//! A manifest from another player: parsing never panics, and whatever parses is canonical and
//! within the limits.
#![no_main]

use chiacraft_packages::identity::{FileManifest, MAX_FILES, MAX_TOTAL_BYTES, check_file};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    if let Ok(m) = FileManifest::parse(text) {
        assert_eq!(m.text(), text);
        assert!(m.files.len() <= MAX_FILES);
        assert!(m.files.iter().map(|f| f.size).sum::<u64>() <= MAX_TOTAL_BYTES);
        for f in &m.files {
            assert!(check_file(&f.path, f.size).is_ok());
            assert!(!f.path.split('/').any(|s| s == ".." || s.is_empty()));
        }
    }
});
