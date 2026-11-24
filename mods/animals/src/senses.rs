//! What an animal knows about the others near it, gathered once and read by every goal
//! (`reports/Creature AI.md`, part 3, step 1; Minecraft's Brain sensors): one look round every
//! `EVERY` ticks finds the nearest hunter that could take it (`hunt::wary` runs from it), how many
//! wild ones of its own kind are near and where their middle is (a pack hunts bigger prey,
//! `hunt::hunt`; a herd huddles in a blood moon, `hunt::huddle`), and for a hungry hunter the
//! nearest prey it could take with them. Before, each of those goals looked round on its own, and
//! read half a dozen records of every animal it saw.
//!
//! What another animal is to it comes from that animal's `Facts`: its kind, species, whether it
//! is tame and its mass, kept by its own upkeep (`refresh`) every `FACTS_EVERY` ticks, so a look
//! round reads one record an animal. Eggs and farm animals, which keep none, are worked out as
//! before (`facts`).

use weft_sdk::*;

use animals_core::perception::{self, NOTICED};
use animals_core::roles;
use animals_core::routines;

use crate::animal::ANIMAL;
use crate::dist2;
use crate::species::{SPECIES, Species};

value! {
    #[derive(Default)]
    /// What an animal is to the others (not saved: its upkeep keeps it).
    pub struct Facts {
        /// Its entity kind.
        pub kind: u32,
        /// Species + 1 (0: none).
        pub species: u8,
        /// Whether a player owns it (an egg: a player's nest or a tame's).
        pub tamed: u8,
        /// Whether it is an egg (food for egg eaters, a hunter to nobody).
        pub egg: u8,
        /// Whether it is still growing (a pack's hunters pick the young, `roles::prey_score`).
        pub young: u8,
        /// Its personality (`coats::personality`: temperament, curiosity, fierceness), 0..255.
        pub persona: [u8; 3],
        /// Its mass now, kg (by its genes and growth, `hunt::mass`).
        pub mass: f32,
        /// When it was worked out (0: not yet).
        pub at: u64,
    }
}

impl Facts {
    /// Temperament (calm 0 to skittish 1), curiosity and fierceness.
    pub fn persona(&self) -> [f32; 3] {
        self.persona.map(|v| v as f32 / 255.0)
    }
    pub fn species(&self) -> Option<&'static Species> {
        SPECIES.get((self.species as usize).checked_sub(1)?)
    }
    /// Its family's name: animals of one family leave each other be.
    pub fn family(&self) -> Option<&'static str> {
        self.species().map(|s| s.family.name)
    }
}

value! {
    #[derive(Default)]
    /// What it last noticed round it (not saved).
    pub struct Senses {
        /// When it looked (0: not yet).
        pub at: u64,
        /// The hunter it is most aware of (0: none, or only a noise), where that was, and how
        /// aware of danger it is (`perception::NOTICED`: it runs).
        pub threat: u64,
        pub threat_pos: [f32; 3],
        pub aware: f32,
        /// The player it is most aware of (0: none), where, and how aware (it bolts at `NOTICED`).
        pub player: u64,
        pub player_pos: [f32; 3],
        pub fright: f32,
        /// Wild ones of its kind within `KIN` (itself among them), and their middle.
        pub kin: u16,
        pub kin_mid: [f32; 3],
        /// Its herd's or pack's leader among those kin: the lowest id, itself if it is lowest
        /// (`animals_core::groups`, `group.rs`).
        pub leader: u64,
        /// A hungry hunter's nearest prey it can take with its kin near (0: none), and its mass.
        pub prey: u64,
        pub prey_kg: f32,
        /// How far it sees through the weather (`base_api::sight`), the wind's way (degrees from
        /// north) and strength (m/s) where it is, and since when.
        pub sight: f32,
        /// Whether it stands watch for its herd now (`roles::SENTINEL`), and when it last heard
        /// an alarm call (it doesn't call one back).
        pub sentinel: u8,
        pub alarm_at: u64,
        pub heading: f32,
        pub wind: f32,
        pub sight_at: u64,
        /// When its stare at the player it half noticed began (0: not staring;
        /// `perception::STARE_SECS`).
        pub stared: u64,
    }
}

/// The wind's way for a test (`/animalprof wind <degrees>`), f32 bits; NaN: the weather's own.
pub static WIND_TEST: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x7fc0_0000);

/// A sound for animals to hear: where, how far it carries, when, and who made it (0: unknown).
#[derive(Clone, Copy)]
struct Noise {
    dim: u32,
    pos: [f32; 3],
    heard: f32,
    at: u64,
    by: u64,
    /// Whether a player made it (worked out once, as it is made).
    player: bool,
}

/// The last sounds made (blocks broken, fights), across all regions (`perception::NOISE_TICKS`
/// old at most are heard).
static NOISES: std::sync::Mutex<Vec<Noise>> = std::sync::Mutex::new(Vec::new());
const MOST_NOISES: usize = 64;

/// A sound at `pos` that carries `heard` blocks, made by `by`.
fn noise(w: &World, pos: [f64; 3], heard: f32, by: u64) {
    let player = by != 0 && w.players_iter().any(|q| q == by);
    let n = Noise { dim: w.dimension(), pos: pos.map(|v| v as f32), heard, at: w.game_time(), by, player };
    let mut v = NOISES.lock().unwrap_or_else(|e| e.into_inner());
    v.retain(|o| n.at < o.at + perception::NOISE_TICKS && n.at >= o.at);
    if v.len() >= MOST_NOISES {
        v.remove(0);
    }
    v.push(n);
}

pub static FACTS: Component<Facts> = Component::named("animal_facts");
pub static SENSES: Component<Senses> = Component::named("animal_senses");

/// Ticks between looks round, and between workings out of its facts and of the weather's sight.
pub const EVERY: u64 = 10;
const FACTS_EVERY: u64 = 100;
const SIGHT_EVERY: u64 = 100;
/// Blocks within which its kin count (a pack hunts together, `hunt::PACK_RANGE`).
pub const KIN: f64 = 16.0;

pub fn init(r: &mut Registry) {
    r.entity_component(&FACTS, Facts::default(), 0);
    r.entity_component(&SENSES, Senses::default(), 0);
    for s in &SPECIES {
        r.edit(s.kind).attach(&FACTS);
        r.edit(s.kind).attach(&SENSES);
    }
    // What animals hear besides footsteps: a block broken, a fight.
    r.on_event(&actions::BREAK_BLOCK, |b| {
        let w = b.world();
        for c in b.iter() {
            // By a player only: a plant broken off by the engine as one grazes beside it (a
            // double plant's other half, one left on bare dirt) says `player: 0`, and the grazer
            // heard its own meal and bolted (ai1124_graze9: 9 of 13 alarm calls within 1.5 s of
            // the caller starting to graze; one ran into a pit two blocks deep and starved).
            if c.player == 0 {
                continue;
            }
            let p = c.pos;
            noise(&w, [p.x as f64 + 0.5, p.y as f64 + 0.5, p.z as f64 + 0.5], perception::BREAK_HEARD, c.player);
        }
    });
    r.on_event(&actions::HURT, |b| {
        let w = b.world();
        for h in b.iter() {
            // Its own body's hurts (starving, thirst, the cold, the heat: `comfort::bodily`) are
            // no fight: a parched deer's every tick of thirst set it and its kin running and
            // calling the alarm (ai1124_savanna10: 241 alarms, none with a hunter or a player).
            if crate::comfort::bodily(h.cause) {
                continue;
            }
            if let Some(e) = w.entity(h.entity) {
                noise(&w, e.pos.to_array(), perception::FIGHT_HEARD, h.source);
            }
        }
    });
}

/// From `animal::upkeep`: its facts again when they are `FACTS_EVERY` ticks old.
pub fn refresh(w: &World, id: u64, index: usize, kind: u32) {
    let now = w.game_time();
    if w.component(id, &FACTS)
        .is_some_and(|f| f.at != 0 && now < f.at + FACTS_EVERY)
    {
        return;
    }
    let f = Facts {
        kind,
        species: index as u8 + 1,
        tamed: crate::animal::tamed(w, id) as u8,
        egg: 0,
        young: crate::breeding::young(w, id) as u8,
        persona: w
            .component(id, &crate::coats::GENOME)
            .map_or([128; 3], |g| animals_core::coats::personality(&g).map(|v| (v * 255.0) as u8)),
        mass: crate::hunt::mass(w, id).unwrap_or(SPECIES[index].bio.mass),
        at: now.max(1),
    };
    if let Some(c) = w.component_mut(id, &FACTS) {
        *c = f;
    }
}

/// What `id` is to the others: its kept facts, or for one that keeps none (an egg, a farm
/// animal, a player) worked out now. `None`: it is nothing to the food web.
pub fn facts(w: &World, id: u64) -> Option<Facts> {
    if let Some(f) = w.component(id, &FACTS).filter(|f| f.at != 0) {
        return Some(f);
    }
    let e = w.entity(id)?;
    let egg = crate::hunt::egg_of(w, id);
    let species = w
        .component(id, &ANIMAL)
        .map(|a| a.species)
        .filter(|&s| s != 0)
        .or(egg.map(|i| i as u8 + 1))
        .unwrap_or(0);
    let mass = crate::hunt::mass(w, id)?;
    let tamed = crate::animal::tamed(w, id) || crate::hunt::owned_egg(w, id);
    let young = crate::breeding::young(w, id) as u8;
    Some(Facts { kind: e.kind, species, tamed: tamed as u8, egg: egg.is_some() as u8, young, persona: [128; 3], mass, at: 0 })
}

/// What it saw at its last look round, looking again first when that was `EVERY` ticks ago (on
/// its own tick of the ten, so the looks of a crowd are spread out). `watch`: blocks it watches
/// for hunters (`hunt::wary`'s, with the weather's sight and a blood moon).
pub fn sense(w: &World, id: u64, st: &EntityState, watch: impl Fn(f32) -> f64) -> Senses {
    let now = w.game_time();
    let mut s = w.component(id, &SENSES).unwrap_or_default();
    if s.at != 0 && (now < s.at + EVERY || !(now + id).is_multiple_of(EVERY)) && now < s.at + 4 * EVERY {
        return s;
    }
    // Less detail where no player would see a mistake (`routines::Detail`).
    let detail = detail(w, st);
    if s.at != 0 && now < s.at + detail.every() && now < s.at + 4 * detail.every() {
        return s;
    }
    let t = crate::prof::stamp();
    if s.sight_at == 0 || now >= s.sight_at + SIGHT_EVERY {
        s.sight = base_api::sight(w, st.pos.to_array());
        let time = w.day_time();
        let hour = ((time % vitals::calendar::DAY_TICKS) as f32 / 1000.0 + 6.0) % 24.0;
        s.heading = vitals::air::heading(w.seed(), time / vitals::calendar::DAY_TICKS, hour);
        let test = f32::from_bits(WIND_TEST.load(std::sync::atomic::Ordering::Relaxed));
        if !test.is_nan() {
            s.heading = test;
        }
        s.wind = w.air_at(st.block_pos()).wind;
        s.sight_at = now.max(1);
    }
    crate::prof::slow(t, || format!("sight and wind of {id}"));
    let watch = watch(s.sight);
    // Seconds since it last looked (its awareness builds and fades by them), a second at most.
    let secs = if s.at == 0 { 0.5 } else { (now.saturating_sub(s.at) as f32 / 20.0).min(4.0) };
    let t = crate::prof::stamp();
    look(w, id, st, &mut s, watch, secs, detail == routines::Detail::Full);
    crate::prof::slow(t, || format!("look round of {id} (sight {:.2}, wind {:.1})", s.sight, s.wind));
    s.at = now.max(1);
    if let Some(c) = w.component_mut(id, &SENSES) {
        *c = s;
    }
    s
}

/// Whether a hungry hunter `s` wants prey now (`hunt::hunt` hunts under `needs::HUNGRY` and keeps
/// on to `needs::FULL`).
fn hunting(w: &World, id: u64, s: &Species, tamed: bool) -> bool {
    !tamed
        && s.hunts()
        && w.component(id, &ANIMAL)
            .is_some_and(|a| (a.food as f32) < a.stat(crate::species::FOOD) * animals_core::needs::FULL)
}

/// Blocks it notices hunters within at most, by any sense (a sprint heard, a scent on the wind):
/// the look round is one query of the entities this near.
const HUNTERS_WITHIN: f64 = 12.0;
/// Blocks a second a walker or a hunter covers sprinting, and walking at all.
const RUNNING: f32 = 0.2;
const MOVING: f32 = 0.01;
/// Blocks a tick under which a walking hunter is stalking (as quiet as a sneak).
const STALKING: f32 = 0.12;

/// How well it sees at `pos` in the light there, 0 to 1 of its range (`perception::DARK_SEEN` in
/// the dark).
fn lit(w: &World, pos: Vec3) -> f32 {
    let l = w.light(pos.block());
    let light = (weft_sdk::abi::light::block(l) as f32).max(weft_sdk::abi::light::sky(l) as f32 * w.daylight()) / 15.0;
    perception::DARK_SEEN + (1.0 - perception::DARK_SEEN) * light.clamp(0.0, 1.0)
}

/// Whether nothing opaque is between two eyes (leaves and plants it sees past).
pub fn clear(w: &World, from: Vec3, to: Vec3) -> bool {
    let v = to - from;
    let d = v.length_squared().sqrt();
    if d < 1.0 {
        return true;
    }
    let dir = v * (1.0 / d);
    let mut t = 0.0;
    for _ in 0..4 {
        let at = from + dir * t;
        let Some(hit) = w.raycast(at, dir, d - t, Ray::blocks()) else { return true };
        if block_info(hit.state).is_some_and(|b| b.opaque()) {
            return false;
        }
        // Past the plant: on from beyond its block.
        let c = Vec3::new(hit.pos.x as f64 + 0.5, hit.pos.y as f64 + 0.5, hit.pos.z as f64 + 0.5);
        let o = c - from;
        t = o.x * dir.x + o.y * dir.y + o.z * dir.z + 0.9;
        if t >= d {
            return true;
        }
    }
    true
}

/// What it makes of one hunter or walker: awareness a second by its best sense, and whether that
/// sense is sight (a line of sight still to check) or smell (which alone only alerts it).
#[derive(Clone, Copy, Default)]
struct Sensed {
    gain: f32,
    sight: f32,
    other: f32,
    hear: f32,
    smell_only: bool,
}

/// What `facing` (the way it faces) and `field` make of something at `off` (x, z) and `d`
/// blocks: seen within `seen` blocks (in view), heard within `heard`, smelled within the wind's
/// reach.
fn sense_one(out: &Senses, facing: [f32; 2], field: f32, off: [f32; 2], d: f32, seen: f32, heard: f32) -> Sensed {
    let sight = if perception::in_view(facing, off, field) { perception::gain(d, seen) } else { 0.0 };
    let hear = perception::gain(d, heard);
    let smell = perception::gain(d, perception::scent_reach(off, out.heading, out.wind));
    let other = hear.max(smell);
    Sensed { gain: sight.max(other), sight, other, hear, smell_only: hear == 0.0 && sight == 0.0 && smell > 0.0 }
}

/// Awareness `a` after `secs` of `s` (nothing sensed: it fades); smell alone alerts it but never
/// makes it bolt.
fn aware_after(a: f32, s: Option<Sensed>, secs: f32) -> f32 {
    match s.filter(|s| s.gain > 0.0) {
        Some(s) if s.smell_only => (a + s.gain * secs).min(a.max(NOTICED - 0.2)),
        Some(s) => (a + s.gain * secs).min(NOTICED + 0.5),
        None => (a - perception::FADES_PER_SEC * secs).max(0.0),
    }
}

/// One look round: the hunters it notices (seen in its field of view with nothing in between,
/// heard on the move, smelled downwind) and players the same way, its awareness of each building
/// up or fading; its kin within `KIN` (`HUDDLE_LOOK` for a herd in a blood moon); and a hungry
/// hunter's nearest prey within `hunt::LOOK`.
fn look(w: &World, id: u64, st: &EntityState, out: &mut Senses, watch: f64, secs: f32, full: bool) {
    let Some(me) = w
        .component(id, &FACTS)
        .filter(|f| f.at != 0)
        .or_else(|| facts(w, id))
    else {
        return;
    };
    let Some(mine) = me.species() else { return };
    let social = mine.bio.social;
    let blood = animals_core::sky::totality(w.seed(), w.day_time());
    let kin_r = match social {
        animals_core::biology::Social::Herd if blood => animals_core::sky::HUDDLE_LOOK as f64,
        animals_core::biology::Social::Herd => KIN,
        _ if mine.pack_prey().is_some() => KIN,
        _ => 0.0,
    };
    let hungry = hunting(w, id, mine, me.tamed != 0);
    let prey_r = if hungry { crate::hunt::LOOK * out.sight as f64 } else { 0.0 };
    let wild = me.tamed == 0;
    let r = HUNTERS_WITHIN.max(watch).max(kin_r).max(prey_r);
    let p = st.pos;
    let eye = st.eye();
    let facing = weft_sdk::yaw_dir(st.yaw);
    let field = if mine.hunts() { perception::HUNTER_FIELD } else { perception::PREY_FIELD };
    // A herd's sentinel notices sooner and farther (`roles::SENTINEL`).
    let eyes = if out.sentinel != 0 { roles::SENTINEL } else { 1.0 };
    // A skittish one notices sooner, a calm one later.
    let eyes = eyes * (1.0 + roles::SKITTISH * roles::lean(me.persona()[0]));
    let (mut kin, mut mid) = (1u16, [p.x, p.y, p.z]);
    let mut leader = id;
    let mut prey: Vec<(u64, f32, f64)> = Vec::new();
    // The hunters it senses, best first after the loop.
    let mut hunters: Vec<(Sensed, u64, Vec3, Vec3)> = Vec::new();
    for t in w.targets_near(p, r, None) {
        if t == id {
            continue;
        }
        let Some(f) = facts(w, t) else { continue };
        let family = f.family() == Some(mine.family.name);
        let kin_too = kin_r > 0.0 && f.kind == me.kind && f.tamed == 0;
        // A wild hunter that takes animals its size (a pack's kind with its pack's reach).
        let hunter = wild
            && !family
            && f.egg == 0
            && f.tamed == 0
            && f.species()
                .is_some_and(|h| h.takes_at_most(f.mass, me.mass));
        let food = hungry && !family && !(f.egg != 0 && f.tamed != 0) && mine.takes_at_most(me.mass, f.mass);
        if !(kin_too || hunter || food) {
            continue;
        }
        let Some(e) = w.entity(t) else { continue };
        let d2 = dist2(e.pos, p);
        if kin_too && d2 <= kin_r * kin_r {
            kin += 1;
            mid = [mid[0] + e.pos.x, mid[1] + e.pos.y, mid[2] + e.pos.z];
            leader = animals_core::groups::leader([leader, t]);
        }
        if hunter {
            let d = d2.sqrt() as f32;
            let off = [(e.pos.x - p.x) as f32, (e.pos.z - p.z) as f32];
            // The light there is read only for the hunters it notices most (below).
            let seen = if d <= watch as f32 * eyes { watch as f32 * eyes } else { 0.0 };
            // A hunter stalking slowly is as quiet as a sneak.
            let heard = match e.moved {
                m if m > RUNNING => perception::SPRINT_HEARD,
                m if m > STALKING => perception::WALK_HEARD,
                m if m > MOVING => perception::SNEAK_HEARD,
                _ => 0.0,
            };
            let mut s = sense_one(out, facing, field, off, d, seen, heard);
            (s.gain, s.sight, s.other, s.hear) = (s.gain * eyes, s.sight * eyes, s.other * eyes, s.hear * eyes);
            if s.gain > 0.0 {
                hunters.push((s, t, e.pos, e.eye()));
            }
        }
        if food && d2 <= prey_r * prey_r && e.health > 0.0 {
            // The young and the hurt count as nearer: hunters pick the weak.
            let share = if e.max_health > 0.0 { e.health / e.max_health } else { 1.0 };
            prey.push((t, f.mass, roles::prey_score(d2, f.young != 0, share)));
        }
    }
    // The most noticeable hunter, its sight only if nothing opaque is between (one or two rays a
    // look at most).
    hunters.sort_by(|a, b| b.0.gain.total_cmp(&a.0.gain));
    let mut best: Option<(Sensed, u64, Vec3)> = None;
    for (mut s, t, pos, their_eye) in hunters.into_iter().take(2) {
        if s.sight > s.other {
            // Its sight of it in the light there, and only with nothing opaque between.
            let d = dist2(pos, p).sqrt() as f32;
            // Coarse, it takes no line of sight and no light (`routines::Detail`).
            let sight = if !full {
                perception::gain(d, watch as f32)
            } else if clear(w, eye, their_eye) {
                perception::gain(d, watch as f32 * lit(w, their_eye))
            } else {
                0.0
            };
            s = Sensed { gain: sight.max(s.other), sight, smell_only: sight == 0.0 && s.hear == 0.0 && s.other > 0.0, ..s };
        }
        if s.gain > 0.0 && best.is_none_or(|b| s.gain > b.0.gain) {
            best = Some((s, t, pos));
        }
    }
    // A fight or a block broken near: alarm, from where it was (a player's is the player's).
    let now = w.game_time();
    let dim = w.dimension();
    let mut player_noise: Option<(f32, u64)> = None;
    let mut fight: Option<(f32, [f32; 3])> = None;
    for n in NOISES.lock().unwrap_or_else(|e| e.into_inner()).iter() {
        if n.dim != dim || n.by == id || now >= n.at + perception::NOISE_TICKS {
            continue;
        }
        let d = dist2(n.pos.map(f64::from), p).sqrt() as f32;
        let g = perception::gain(d, n.heard);
        if g <= 0.0 {
            continue;
        }
        if n.player {
            if player_noise.is_none_or(|b| g > b.0) {
                player_noise = Some((g, n.by));
            }
        } else if fight.is_none_or(|b| g > b.0) {
            fight = Some((g, n.pos));
        }
    }
    let hunter_sense = match (best, fight) {
        (Some(b), Some((g, _))) if g > b.0.gain => Some(Sensed { gain: g, ..Default::default() }),
        (Some(b), _) => Some(b.0),
        (None, Some((g, _))) => Some(Sensed { gain: g, ..Default::default() }),
        (None, None) => None,
    };
    out.aware = aware_after(out.aware, hunter_sense, secs);
    match (best, fight) {
        (Some((_, t, pos)), _) => {
            out.threat = t;
            out.threat_pos = [pos.x as f32, pos.y as f32, pos.z as f32];
        }
        (None, Some((_, pos))) => {
            out.threat = 0;
            out.threat_pos = pos;
        }
        // Nothing now: it remembers where the danger was while its awareness fades.
        (None, None) if out.aware <= 0.0 => out.threat = 0,
        _ => {}
    }
    if wild && mine.shy > 0.0 {
        players(w, id, st, out, mine, facing, field, secs, player_noise, full);
    }
    out.kin = kin;
    out.leader = leader;
    out.kin_mid = mid.map(|v| (v / kin as f64) as f32);
    // One adult of a herd at a time stands watch, the watch going round (by id and time).
    let herd = social == animals_core::biology::Social::Herd;
    out.sentinel = (herd && kin >= roles::SENTINEL_HERD && me.young == 0 && {
        let turn = now / roles::SENTINEL_TURN;
        (id ^ turn.wrapping_mul(0x9E37_79B9)).wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40 & 0xFFFF < (0x10000 / kin as u64)
    }) as u8;
    let best = prey
        .into_iter()
        .filter(|&(_, kg, _)| mine.takes_with(me.mass, kg, kin as u32))
        .min_by(|a, b| a.2.total_cmp(&b.2));
    (out.prey, out.prey_kg) = best.map_or((0, 0.0), |(t, kg, _)| (t, kg));
}

/// Its awareness of players (`taming::behave` bolts at `NOTICED`): one walking upright is seen
/// within its shy distance (twice that sprinting, half crouching), in the light, in its field of
/// view and with nothing opaque between; heard walking, sprinting, a little sneaking; smelled
/// downwind. One standing still it doesn't see (it may still smell him).
#[allow(clippy::too_many_arguments)]
fn players(
    w: &World,
    id: u64,
    st: &EntityState,
    out: &mut Senses,
    mine: &'static Species,
    facing: [f32; 2],
    field: f32,
    secs: f32,
    noise: Option<(f32, u64)>,
    full: bool,
) {
    let p = st.pos;
    let far = perception::SPRINT_HEARD.max(perception::SCENT) as f64;
    let mut shy = None;
    let mut best: Option<(Sensed, u64, Vec3, Vec3)> = None;
    for q in w.players_iter() {
        let Some(e) = w.entity(q) else { continue };
        let d2 = dist2(e.pos, p);
        if d2 > far * far || w.game_mode(q) == weft_sdk::abi::game_mode::SPECTATOR {
            continue;
        }
        let shy = *shy.get_or_insert_with(|| mine.shy * crate::taming::shyness(w, id));
        let d = d2.sqrt() as f32;
        let moving = e.moved > MOVING;
        let (sprint, sneak) = (e.sprinting != 0, e.pose == weft_sdk::abi::pose::CROUCH);
        let seen = if !moving {
            0.0
        } else {
            shy * if sprint { 2.0 } else { 1.0 }
                * if sneak { perception::SNEAK_SEEN } else { 1.0 }
                * if full { lit(w, e.eye()) } else { 1.0 }
        };
        let heard = match (moving, sprint, sneak) {
            (false, ..) => 0.0,
            (true, true, _) => perception::SPRINT_HEARD,
            (true, false, true) => perception::SNEAK_HEARD,
            _ => perception::WALK_HEARD,
        };
        let off = [(e.pos.x - p.x) as f32, (e.pos.z - p.z) as f32];
        // What it and its kind here remember of him (`memory`): one who hurt them is noticed
        // sooner and farther off, one who fed them later and nearer.
        let (sooner, farther) = crate::memory::wary_of(w, id, mine.index(), p.to_array(), q);
        let mut s = sense_one(out, facing, field, off, d, seen * farther, heard * farther.min(1.5));
        (s.gain, s.sight, s.other, s.hear) = (s.gain * sooner, s.sight * sooner, s.other * sooner, s.hear * sooner);
        if s.gain > 0.0 && best.is_none_or(|b| s.gain > b.0.gain) {
            best = Some((s, q, e.pos, e.eye()));
        }
    }
    let mut sensed = best.map(|(s, q, pos, their_eye)| {
        let s = if full && s.sight > s.other && !clear(w, st.eye(), their_eye) {
            Sensed { gain: s.other, sight: 0.0, smell_only: s.hear == 0.0 && s.other > 0.0, ..s }
        } else {
            s
        };
        (s, q, pos)
    });
    // A block broken near it by a player, or a fight one started.
    if let Some((g, q)) = noise
        && sensed.is_none_or(|b| g > b.0.gain)
        && let Some(e) = w.entity(q)
    {
        sensed = Some((Sensed { gain: g, ..Default::default() }, q, e.pos));
    }
    out.fright = aware_after(out.fright, sensed.map(|b| b.0), secs);
    if out.fright < perception::STARE {
        out.stared = 0;
    }
    match sensed {
        Some((_, q, pos)) => {
            out.player = q;
            out.player_pos = [pos.x as f32, pos.y as f32, pos.z as f32];
        }
        None if out.fright <= 0.0 => out.player = 0,
        None => {}
    }
}

/// Where to run from `from`: `dist` blocks away along the way most nearly straight away from it
/// that has ground to stand on half way and at the end (no drop of more than `needs::FLEE_DROP`, so it can step back up; no water), trying
/// ways turned up to 110° off; straight away when none has.
pub fn escape(w: &World, st: &EntityState, from: [f64; 3], dist: f64, herd: Option<[f64; 3]>) -> [f64; 3] {
    let (mut x, mut z) = (st.pos.x - from[0], st.pos.z - from[2]);
    // A herd animal runs toward its herd's middle too, unless that is toward the hunter: the others
    // between it and the danger (`roles::SELFISH`).
    if let Some(mid) = herd {
        let n = (x * x + z * z).sqrt().max(0.01);
        let (mx, mz) = (mid[0] - st.pos.x, mid[2] - st.pos.z);
        let m = (mx * mx + mz * mz).sqrt();
        if m > 2.0 && (mx * x + mz * z) / (m * n) > -0.3 {
            let k = roles::SELFISH as f64;
            (x, z) = (x / n + k * mx / m, z / n + k * mz / m);
        }
    }
    let away = if x * x + z * z > 0.01 { z.atan2(x) } else { w.random_below(628) as f64 / 100.0 };
    let straight = [st.pos.x + away.cos() * dist, st.pos.y, st.pos.z + away.sin() * dist];
    for turn in [0.0f64, 35.0, -35.0, 70.0, -70.0, 110.0, -110.0] {
        let a = away + turn.to_radians();
        let mut y = st.pos.y.floor() as i32;
        let mut at = None;
        for step in [dist / 2.0, dist] {
            let (px, pz) = (st.pos.x + a.cos() * step, st.pos.z + a.sin() * step);
            let (bx, bz) = (px.floor() as i32, pz.floor() as i32);
            match crate::comfort::ground(w, bx, y, bz) {
                Some(g) if y - g <= animals_core::needs::FLEE_DROP && !crate::comfort::WATER.at(w, [bx, g, bz]) => {
                    y = g;
                    at = Some([px, g as f64, pz]);
                }
                _ => {
                    at = None;
                    break;
                }
            }
        }
        if let Some(to) = at {
            return to;
        }
    }
    straight
}

/// It has half noticed something at `at`: it stops and turns to look that way, staring (what the
/// player sees before it bolts).
pub fn stare(w: &World, id: u64, st: &EntityState, at: [f32; 3]) {
    face(w, id, st, at);
    crate::anim::hold(w, id, crate::anim::Act::Stare);
}

/// Whether it still stares at the player it half noticed: from when its awareness rose past
/// `perception::STARE`, for `perception::STARE_SECS`.
pub fn staring(w: &World, id: u64) -> bool {
    let now = w.game_time();
    let Some(s) = w.component_mut(id, &SENSES) else { return false };
    if s.stared == 0 {
        s.stared = now.max(1);
    }
    now.saturating_sub(s.stared) < (perception::STARE_SECS * 20.0) as u64
}

/// It stops and turns to face `at`.
pub fn face(w: &World, id: u64, st: &EntityState, at: [f32; 3]) {
    if w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
    let (x, z) = (at[0] as f64 - st.pos.x, at[2] as f64 - st.pos.z);
    if x * x + z * z > 0.01 {
        // `yaw_dir`'s inverse: facing (x, z) is (-sin yaw, -cos yaw).
        w.entity_modify(id).yaw((-x).atan2(-z) as f32);
    }
}

/// It has run from the danger (`player`: from a player): its awareness of it falls to
/// `perception::ALERT`, so it runs again only once it notices again.
pub fn ran(w: &World, id: u64, player: bool) {
    if let Some(s) = w.component_mut(id, &SENSES) {
        let a = if player { &mut s.fright } else { &mut s.aware };
        *a = a.min(perception::ALERT);
    }
}

/// Its herd's middle from its last look, if it is a herd's and has kin near (`escape`'s selfish
/// herd).
pub fn herd_mid(w: &World, id: u64) -> Option<[f64; 3]> {
    let s = w.component(id, &SENSES)?;
    let herd = w
        .component(id, &FACTS)
        .and_then(|f| f.species())
        .is_some_and(|sp| sp.bio.social == animals_core::biology::Social::Herd);
    (herd && s.kin >= 2).then(|| s.kin_mid.map(f64::from))
}

/// It runs from `threat` at `pos`: it calls, and its wild kin within `roles::ALARM` blocks know at
/// once (unless it ran on a call itself). How many heard.
pub fn alarm(w: &World, id: u64, st: &EntityState, threat: u64, pos: [f32; 3]) -> u32 {
    let now = w.game_time();
    if w.component(id, &SENSES)
        .is_some_and(|s| s.alarm_at != 0 && now < s.alarm_at + roles::ALARM_QUIET)
    {
        return 0;
    }
    let Some(kind) = SPECIES
        .iter()
        .find(|s| s.kind.id() == st.kind)
        .map(|s| s.kind)
    else {
        return 0;
    };
    if !SPECIES
        .iter()
        .any(|s| s.kind.id() == st.kind && s.bio.social != animals_core::biology::Social::Solitary)
    {
        return 0;
    }
    let mut heard = 0;
    for o in w.entities_near(st.pos, roles::ALARM, Some(kind)) {
        if o == id || crate::animal::tamed(w, o) {
            continue;
        }
        if let Some(s) = w.component_mut(o, &SENSES) {
            s.aware = s.aware.max(NOTICED);
            s.threat = threat;
            s.threat_pos = pos;
            s.alarm_at = now.max(1);
            heard += 1;
        }
    }
    heard
}

/// A goal in the needs band (`Need::Watch`): a herd's sentinel stands with its head up, turning to
/// look round now and then; it takes the turn while it is on watch.
pub fn watch_goal(w: &World, id: u64, st: &EntityState) -> bool {
    if w.component(id, &SENSES).is_none_or(|s| s.sentinel == 0) {
        return false;
    }
    if w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
    if (w.game_time() + id).is_multiple_of(40) {
        let turn = (w.random_below(240) as f32 - 120.0).to_radians();
        w.entity_modify(id).yaw(st.yaw + turn);
    }
    crate::anim::hold(w, id, crate::anim::Act::Watch);
    true
}

/// A goal in the needs band (`Need::Investigate`): a curious one (`roles::CURIOUS`) that is calm
/// comes to look at a player standing still within `LOOKS_FROM` times its shy distance, to
/// `COMES_TO` of it (nearer the more curious), and stands looking at him; it takes the turn while
/// it does. Once he moves, its senses take over (`taming::behave`).
pub fn investigate(w: &World, id: u64, st: &EntityState) -> bool {
    use crate::choose::Need;
    let now = w.game_time();
    let at_it = crate::choose::doing(w, id) == Need::Investigate as u8;
    if !(now + id).is_multiple_of(20) {
        return at_it;
    }
    let Some(me) = w
        .component(id, &FACTS)
        .filter(|f| f.at != 0 && f.tamed == 0)
    else {
        return false;
    };
    let Some(mine) = me.species() else { return false };
    let [_, curious, _] = me.persona();
    if curious < roles::CURIOUS
        || mine.shy <= 0.0
        || w.component(id, &SENSES)
            .is_some_and(|s| s.fright >= perception::STARE)
    {
        return false;
    }
    let shy = mine.shy * crate::taming::shyness(w, id);
    let facing = weft_sdk::yaw_dir(st.yaw);
    let field = if mine.hunts() { perception::HUNTER_FIELD } else { perception::PREY_FIELD };
    let near = w
        .players_iter()
        .filter(|&q| w.game_mode(q) != weft_sdk::abi::game_mode::SPECTATOR)
        .filter_map(|q| Some((q, w.entity(q)?)))
        .filter(|(_, e)| e.moved <= MOVING)
        .map(|(q, e)| (q, dist2(e.pos, st.pos).sqrt() as f32, e))
        .filter(|(_, d, e)| {
            *d <= shy * roles::LOOKS_FROM
                && perception::in_view(facing, [(e.pos.x - st.pos.x) as f32, (e.pos.z - st.pos.z) as f32], field)
                && clear(w, st.eye(), e.eye())
        })
        .min_by(|a, b| a.1.total_cmp(&b.1));
    let Some((q, d, e)) = near else { return false };
    let to = shy * roles::COMES_TO * (1.5 - curious);
    if d > to + 1.0 {
        if !at_it {
            weft_sdk::info!("{} {id} comes to look at player {q} ({d:.1} blocks off, curiosity {curious:.2})", mine.name);
        }
        let k = ((d - to) / d) as f64;
        w.navigate(id, [st.pos.x + (e.pos.x - st.pos.x) * k, e.pos.y, st.pos.z + (e.pos.z - st.pos.z) * k], 0.6);
        // Slowly, the head low (held till its next look, in 20 ticks).
        crate::anim::hold(w, id, crate::anim::Act::Approach);
    } else {
        stare(w, id, st, [e.pos.x as f32, e.pos.y as f32, e.pos.z as f32]);
    }
    true
}

/// How much detail `st`'s senses get now (`routines::Detail`): by the nearest player, and whether
/// it is in a player's view.
pub fn detail(w: &World, st: &EntityState) -> routines::Detail {
    use animals_core::routines::{COARSE_WITHIN, Detail, FULL_WITHIN, SEEN_WITHIN, VIEW};
    let mut best = Detail::Far;
    for q in w.players_iter() {
        let Some(p) = w.entity(q) else { continue };
        let d2 = dist2(p.pos, st.pos);
        if d2 <= FULL_WITHIN * FULL_WITHIN {
            return Detail::Full;
        }
        let off = [(st.pos.x - p.pos.x) as f32, (st.pos.z - p.pos.z) as f32];
        if d2 <= SEEN_WITHIN * SEEN_WITHIN && perception::in_view(weft_sdk::yaw_dir(p.yaw), off, VIEW) {
            return Detail::Full;
        }
        if d2 <= COARSE_WITHIN * COARSE_WITHIN {
            best = Detail::Coarse;
        }
    }
    best
}
