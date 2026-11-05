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
        /// Its mass now, kg (by its genes and growth, `hunt::mass`).
        pub mass: f32,
        /// When it was worked out (0: not yet).
        pub at: u64,
    }
}

impl Facts {
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
    /// What it last saw round it (not saved).
    pub struct Senses {
        /// When it looked (0: not yet).
        pub at: u64,
        /// The nearest hunter that could take it, and where it was (0: none).
        pub threat: u64,
        pub threat_pos: [f32; 3],
        /// Wild ones of its kind within `KIN` (itself among them), and their middle.
        pub kin: u16,
        pub kin_mid: [f32; 3],
        /// A hungry hunter's nearest prey it can take with its kin near (0: none), and its mass.
        pub prey: u64,
        pub prey_kg: f32,
        /// How far it sees through the weather (`base_api::sight`), and since when.
        pub sight: f32,
        pub sight_at: u64,
    }
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
        r.attach(s.kind, &FACTS);
        r.attach(s.kind, &SENSES);
    }
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
    Some(Facts { kind: e.kind, species, tamed: tamed as u8, egg: egg.is_some() as u8, mass, at: 0 })
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
    if s.sight_at == 0 || now >= s.sight_at + SIGHT_EVERY {
        s.sight = base_api::sight(w, st.pos.to_array());
        s.sight_at = now.max(1);
    }
    let watch = watch(s.sight);
    look(w, id, st, &mut s, watch);
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

/// One look round: the nearest hunter within `watch`, its kin within `KIN` (`HUDDLE_LOOK` for a
/// herd in a blood moon), and a hungry hunter's nearest prey within `hunt::LOOK`.
fn look(w: &World, id: u64, st: &EntityState, out: &mut Senses, watch: f64) {
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
        animals_core::biology::Social::Herd => 0.0,
        _ if mine.pack_prey().is_some() => KIN,
        _ => 0.0,
    };
    let hungry = hunting(w, id, mine, me.tamed != 0);
    let prey_r = if hungry { crate::hunt::LOOK * out.sight as f64 } else { 0.0 };
    let r = watch.max(kin_r).max(prey_r);
    let p = st.pos;
    let (mut threat, mut threat_d) = (None, f64::MAX);
    let (mut kin, mut mid) = (1u16, [p.x, p.y, p.z]);
    let mut prey: Vec<(u64, f32, f64)> = Vec::new();
    for t in w.targets_near(p, r, None) {
        if t == id {
            continue;
        }
        let Some(f) = facts(w, t) else { continue };
        let Some(e) = w.entity(t) else { continue };
        let d = dist2(e.pos, p);
        if f.kind == me.kind && f.tamed == 0 && d <= kin_r * kin_r {
            kin += 1;
            mid = [mid[0] + e.pos.x, mid[1] + e.pos.y, mid[2] + e.pos.z];
        }
        // Animals of one family leave each other be.
        if f.family() == Some(mine.family.name) {
            continue;
        }
        // A wild hunter that takes animals its size (a pack's kind with its pack's reach).
        if f.egg == 0
            && f.tamed == 0
            && d < threat_d
            && d <= watch * watch
            && let Some(h) = f.species()
            && h.takes_at_most(f.mass, me.mass)
        {
            (threat, threat_d) = (Some((t, e.pos)), d);
        }
        // Prey of its sizes, tames too; a player's eggs it leaves.
        if hungry && d <= prey_r * prey_r && e.health > 0.0 && !(f.egg != 0 && f.tamed != 0) && mine.takes_at_most(me.mass, f.mass) {
            prey.push((t, f.mass, d));
        }
    }
    out.threat = threat.map_or(0, |(t, _)| t);
    out.threat_pos = threat.map_or([0.0; 3], |(_, q)| [q.x as f32, q.y as f32, q.z as f32]);
    out.kin = kin;
    out.kin_mid = mid.map(|v| (v / kin as f64) as f32);
    let best = prey
        .into_iter()
        .filter(|&(_, kg, _)| mine.takes_with(me.mass, kg, kin as u32))
        .min_by(|a, b| a.2.total_cmp(&b.2));
    (out.prey, out.prey_kg) = best.map_or((0, 0.0), |(t, kg, _)| (t, kg));
}
