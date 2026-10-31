//! A signature envelope from another player: parsing never panics, and what parses writes back
//! the same.
#![no_main]

use chiacraft_packages::signature::{Envelope, fingerprint};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    if let Ok(e) = Envelope::parse(text) {
        assert_eq!(Envelope::parse(&e.text()), Ok(e.clone()));
        let _ = fingerprint(&e.creator);
    }
});
