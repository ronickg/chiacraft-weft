//! Every species' calls, from the table in `animals_core::voices`: events
//! `animals:<kind>.hurt|death|ambient` from its family's files (`assets/sounds/voice/<family>/`),
//! pitched for the species, and its kind's sound prefix pointed at them. The engine plays them:
//! hurt and death where it is hit, the ambient call about every twelve seconds.
//!
//! On a blood moon wild wolves howl once as totality begins (`howl_at_blood_moon`,
//! `animals_core::sky`), a pack a ragged chorus over a few seconds.

use animals_core::voices;
use weft_sdk::*;

/// Of one species' call, at most this many at once: a herd doesn't drown out the world.
const AT_ONCE: u32 = 4;

/// The wolf's howl (its ambient call) and the night it last howled at a blood moon + 1 (not saved).
static HOWL: Sound = Sound::named("grey_wolf.ambient");
static HOWLED: Component<u64> = Component::named("animal_howled");
/// One in this many ticks a wolf howls, once it's time: a pack's howls spread over a few seconds.
const HOWL_ONE_IN: u32 = 30;

pub fn init(r: &mut Registry) {
    r.entity_component(&HOWLED, 0u64, 0);
    r.attach(&crate::wolf::WOLF, &HOWLED);
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
        r.sounds_on_entities(&format!("animals:{kind}"), &format!("animals:{kind}"));
    }
}

/// A goal for wolves (never taken, it only howls): a wild wolf howls once as a blood moon's
/// totality begins.
pub fn howl_at_blood_moon(w: &World, id: u64, st: &EntityState) -> bool {
    let night = w.day_time() / vitals::calendar::DAY_TICKS + 1;
    if w.component(id, &HOWLED) == Some(night)
        || !animals_core::sky::howl_time(w.seed(), w.day_time())
        || crate::animal::tamed(w, id)
        || w.random_below(HOWL_ONE_IN) != 0
    {
        return false;
    }
    if let Some(n) = w.component_mut(id, &HOWLED) {
        *n = night;
    }
    w.play_sound(&HOWL, st.pos, 1.0);
    crate::anim::act(w, id, crate::anim::Act::Howl);
    weft_sdk::info!("grey_wolf {id} howls at the blood moon");
    false
}
