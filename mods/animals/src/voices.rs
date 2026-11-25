//! Every species' calls, from the table in `animals_core::voices`: events
//! `animals:<kind>.hurt|death|ambient` from its family's files (`assets/sounds/voice/<family>/`),
//! pitched for the species, and its kind's sound prefix pointed at them. The engine plays them:
//! hurt and death where it is hit, the ambient call about every twelve seconds.
//!
//! Its acts (`voices::ACTS`, AI): `<kind>.alarm` with its alarm clip, `<kind>.attack` with its
//! attack clips (`on_clip`, from `anim::play`), `<kind>.eat` as it is fed (`eat`, from
//! `animal::feed`), and for the heavy ones `<kind>.step` in place of the ground's step (the engine
//! plays a kind's own step when it has one).
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
    r.edit(&crate::wolf::WOLF).attach(&HOWLED);
    for s in &crate::species::SPECIES {
        let Some(v) = voices::of(s.name) else { continue };
        let (kind, kg, carnivore) = (s.kind.path(), s.bio.mass, s.diet() == crate::species::Diet::Carnivore);
        let (alarm, attack) = voices::acts(v.family);
        if let Some(f) = voices::family(v.family) {
            sound(r, kind, "alarm", f, alarm, v.pitch);
            sound(r, kind, "attack", f, attack, v.pitch);
        }
        let eat = voices::eat_way(v.family, carnivore);
        extra(r, kind, "eat", &format!("eat/{eat}"), voices::EAT_FILES, voices::eat_pitch(kg), voices::eat_volume(kg), 16.0);
        if let Some(size) = voices::step_size(kg, carnivore) {
            extra(r, kind, "step", &format!("steps/{size}"), voices::STEP_FILES, voices::step_pitch(kg), voices::step_volume(kg), 24.0);
        }
        family_sounds(r, kind, v.family, v.pitch);
    }
    // The bosses and their young have no species: their own families, as recorded.
    for (kind, fam) in voices::YOUNG {
        family_sounds(r, kind, fam, 1.0);
    }
    for b in animals_core::bosses::BOSSES {
        extra(r, b.name, "step", "steps/huge", voices::STEP_FILES, 0.8, 2.5, 48.0);
        family_sounds(r, b.name, b.name, 1.0);
    }
    for (boss, event, n) in voices::BOSS_CALLS {
        let Some(f) = voices::family(boss) else { continue };
        sound(r, boss, event, f, n, 1.0);
    }
}

/// `animals:<kind>.hurt|death|ambient` from `family`'s files, and the kind's sounds pointed at them.
fn family_sounds(r: &mut Registry, kind: &str, family: &str, pitch: f32) {
    let Some(f) = voices::family(family) else {
        weft_sdk::error!("voice of {kind}: no family {family}");
        return;
    };
    for (event, n) in [("hurt", f.hurt), ("death", f.death), ("ambient", f.ambient)] {
        sound(r, kind, event, f, n, pitch);
    }
    r.sounds_on_entities(&format!("animals:{kind}"), &format!("animals:{kind}"));
}

/// `animals:<kind>.<event>`: `n` files `voice/<family>/<event>1..n` (none: silent).
fn sound(r: &mut Registry, kind: &str, event: &str, f: &voices::Family, n: u32, pitch: f32) {
    if n == 0 {
        return;
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
        .pitch(pitch * 0.92, pitch * 1.08)
        .voices(AT_ONCE);
}

/// `animals:<kind>.<event>` from the shared files `voice/<dir><1..n>` (eating, steps), at `pitch`
/// (±8 %) and `volume`, heard `range` blocks.
#[allow(clippy::too_many_arguments)]
fn extra(r: &mut Registry, kind: &str, event: &str, dir: &str, n: u32, pitch: f32, volume: f32, range: f32) {
    let files: Vec<String> = (1..=n).map(|i| format!("voice/{dir}{i}")).collect();
    let files: Vec<&str> = files.iter().map(String::as_str).collect();
    r.sound(Sound::generated(format!("{kind}.{event}")))
        .files(&files)
        .category(&sound_categories::CREATURES)
        .volume(volume)
        .range(range)
        .pitch(pitch * 0.92, pitch * 1.08)
        .voices(AT_ONCE);
}

/// `<name>` (`<kind>.<event>`, registered), played at `at`.
fn call(w: &World, name: String, at: Vec3) {
    static CALLS: std::sync::Mutex<Option<std::collections::HashMap<String, &'static Sound>>> = std::sync::Mutex::new(None);
    let mut calls = CALLS.lock().unwrap();
    let sound = *calls
        .get_or_insert_with(Default::default)
        .entry(name.clone())
        .or_insert_with(|| Sound::generated(name));
    w.play_sound(sound, at, 1.0);
}

/// A boss's own call (`voices::BOSS_CALLS`: summon, attack, stunned), where it stands.
pub fn boss_call(w: &World, boss: &str, event: &str, at: [f64; 3]) {
    call(w, format!("{boss}.{event}"), Vec3::from(at));
}

/// With clip `clip` (`anim::play`): a species' alarm call with its alarm, its attack call with an
/// attack (a pounce, bite or rear, or one of its species' attack clips), if its family has them.
pub fn on_clip(w: &World, id: u64, clip: &str) {
    let Some(s) = w.component(id, &crate::animal::ANIMAL).and_then(|a| a.species()) else { return };
    let Some(v) = voices::of(s.name) else { return };
    let (alarm, attack) = voices::acts(v.family);
    let event = match clip {
        "alarm" if alarm > 0 => "alarm",
        "pounce" | "bite" | "rear" if attack > 0 => "attack",
        c if attack > 0 && s.attacks.contains(&c) => "attack",
        _ => return,
    };
    let Some(st) = w.entity(id) else { return };
    call(w, format!("{}.{event}", s.kind.path()), st.pos);
}

/// As `id` is fed (`animal::feed`): its eating sound (`voices::eat_way`, pitched by its size).
pub fn eat(w: &World, id: u64) {
    let Some(s) = w.component(id, &crate::animal::ANIMAL).and_then(|a| a.species()) else { return };
    if voices::of(s.name).is_none() {
        return;
    }
    let Some(st) = w.entity(id) else { return };
    call(w, format!("{}.eat", s.kind.path()), st.pos);
}

/// A goal for wolves (never taken, it only howls): a wild wolf howls once as a blood moon's
/// totality begins.
/// A wolf's howl where it stands (a pack's rally call, `group::regroup`).
pub fn howl(w: &World, id: u64, st: &EntityState) {
    w.play_sound(&HOWL, st.pos, 1.0);
    crate::anim::act(w, id, crate::anim::Act::Howl);
}

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
