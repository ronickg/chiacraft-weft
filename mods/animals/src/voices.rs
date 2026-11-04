//! Every species' calls, from the table in `animals_core::voices`: events
//! `animals:<kind>.hurt|death|ambient` from its family's files (`assets/sounds/voice/<family>/`),
//! pitched for the species, and its kind's sound prefix pointed at them. The engine plays them:
//! hurt and death where it is hit, the ambient call about every twelve seconds.

use animals_core::voices;
use weft_sdk::*;

/// Of one species' call, at most this many at once: a herd doesn't drown out the world.
const AT_ONCE: u32 = 4;

pub fn init(r: &mut Registry) {
    for s in &crate::species::SPECIES {
        let Some(v) = voices::of(s.name) else { continue };
        let Some(f) = voices::family(v.family) else {
            weft_sdk::error!("voice of {}: no family {}", s.name, v.family);
            continue;
        };
        let kind = s.kind.path();
        for (event, n) in [("hurt", f.hurt), ("death", f.death), ("ambient", f.ambient)] {
            if n == 0 {
                continue;
            }
            let files: Vec<String> = (1..=n)
                .map(|i| format!("voice/{}/{event}{i}", f.name))
                .collect();
            let files: Vec<&str> = files.iter().map(String::as_str).collect();
            r.sound(Sound::generated(format!("{kind}.{event}")))
                .files(&files)
                .category(&sound_categories::CREATURES)
                .volume(f.volume)
                .range(f.range)
                .pitch(v.pitch * 0.92, v.pitch * 1.08)
                .voices(AT_ONCE);
        }
        r.sounds_on(Target::Entity(&format!("animals:{kind}")), &format!("animals:{kind}"));
    }
}
