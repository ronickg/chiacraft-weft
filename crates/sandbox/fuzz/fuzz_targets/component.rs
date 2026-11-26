//! A component from another player's package: compiling, linking and running its `init` never
//! panics or hangs the game, whatever the bytes (fuel and the watchdog end `init`).
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(m) = chiacraft_sandbox::Mod::load("fuzz", data) {
        assert!(m.hooks().len() <= 256);
    }
});
