//! Breeding (`notes/ANIMALS.md`, "Breeding", "Babies"), for every species. A grown tame with a
//! bond of 50 or more, fed one of its favorite foods by a player who may command it, falls in
//! love for thirty seconds; two of one species and of both sexes in love go to each other (one
//! told to stay waits) and mate. The female then carries the young (`Species::gestation`) and
//! gives birth to a litter, or, a raptor, lays a clutch of eggs (`eggs.rs`: nests, incubation,
//! taking and placing eggs). Each young gets
//! one allele of each gene from each parent (`coats::breed`, so siblings differ) and each stat's
//! wild points from one of them, the stronger 55% of the time. Young are their mother's owner's,
//! follow their mother, grow from their birth size to a grown one's over `Species::growth`
//! (their favorite food speeds it), and remember their parents and generation. Afterwards both
//! parents rest, the mother for her recovery.

use std::sync::atomic::{AtomicU32, Ordering};

use animals_core::biology::seasons;
use animals_core::life::{self, LOVE_TICKS, MALE_REST};
use weft_sdk::*;

use crate::animal::{self, ANIMAL, Animal, OWNER};
use crate::coats::{self, GENOME, Genome};
use crate::species::{self, SPECIES, STATS};

value! {
    #[derive(Default)]
    /// Where an animal is in breeding and growing up, and its family; saved, and sent to whoever
    /// has its screen open.
    pub struct Breeding {
        /// Ticks left in love (0: not).
        pub love: u32,
        /// Ticks before it can fall in love again.
        pub rest: u32,
        /// Ticks left carrying young, or before laying eggs (0: not expecting).
        pub carrying: u32,
        /// Ticks left growing up (0: grown).
        pub growing: u32,
        /// 0 for one born in the wild; else one more than its parents' latest.
        pub generation: u32,
        /// Its parents (0: unknown).
        pub mother: u64,
        pub father: u64,
        /// Its parents' genes (`Genome`s; all 0: unknown).
        pub dam: [u32; 10],
        pub sire: [u32; 10],
        /// While carrying: the father, his genes and wild points.
        pub mate: u64,
        pub mate_genes: [u32; 10],
        pub mate_wild: [u8; 9],
        /// The rest of `dam`, `sire` and `mate_genes` (`pack`: genomes grew past 40 bytes; kept
        /// after older saves' fields).
        pub dam_more: More,
        pub sire_more: More,
        pub mate_more: More,
    }
}

pub static BREEDING: Component<Breeding> = Component::named("breeding");
static HEART: Particle = Particle::named("heart");

/// The bond breeding needs.
const BOND: u8 = 50;
/// Out of its breeding season a tame still comes into heat if it's well cared for: this much
/// bond, and its belly this full (a stand-in for comfort until camps have it).
const OUT_OF_SEASON_BOND: u8 = 80;
const OUT_OF_SEASON_FOOD: f32 = 0.75;

/// The season it is in this dimension, as a breeding season's bit (`seasons::bit`); `None` where
/// there are no seasons (all year is breeding season there). In a world with a wet and a dry
/// season, its first half (spring and summer) is the wet one.
pub fn season_bit(w: &World) -> Option<u8> {
    base_api::SEASONS
        .on_dimension(w.dimension())
        .filter(|&k| k > 0)
        .map(|_| seasons::bit(vitals::calendar::date(w.day_time()).season.index()))
}

/// A genome's words past its first ten (`pack`).
pub type More = [u32; 11];
const _: () = assert!(size_of::<Genome>() <= 84);

/// A genome kept as words, as `Breeding` and `eggs::Egg` keep them: its first 40 bytes (all a
/// genome had before the core traits), and the rest.
pub fn pack(g: &Genome) -> ([u32; 10], More) {
    let mut out = [0u32; 21];
    unsafe { std::ptr::copy_nonoverlapping(g as *const Genome as *const u8, out.as_mut_ptr() as *mut u8, size_of::<Genome>()) };
    let (a, b) = out.split_at(10);
    (a.try_into().unwrap(), b.try_into().unwrap())
}

/// A genome kept as words (`pack`), if any; one kept before the core traits has none of them
/// (`coats::mend`).
pub fn unpack(a: &[u32; 10], more: &More) -> Option<Genome> {
    let mut all = [0u32; 21];
    all[..10].copy_from_slice(a);
    all[10..].copy_from_slice(more);
    let mut g: Genome = unsafe { std::ptr::read_unaligned(all.as_ptr() as *const Genome) };
    coats::mend(&mut g);
    (g.species != 0).then_some(g)
}

pub fn init(r: &mut Registry) {
    r.entity_component(&BREEDING, Breeding::default(), component_flags::SAVE | component_flags::SYNC_MENU);
    for s in &SPECIES {
        r.attach(s.kind, &BREEDING);
    }
    // Counted for `/animal census`: deaths other than of old age (counted in `animal::old_age`).
    // (By its kind: its components are gone by the time the event comes.)
    r.on(&ENTITY_DIED, |b| {
        for d in b.iter() {
            if d.cause != damage::VOID
                && let Some(i) = SPECIES.iter().position(|s| s.kind.id() == d.kind)
            {
                count(&DIED, i);
            }
        }
    });
}

// ------------------------------------------------------------------ testing: speed and census

/// How many times faster than the clock animals live (`/animal speed`, for testing: their
/// timers, aging and wild breeding; not saved).
static SPEED: AtomicU32 = AtomicU32::new(1);
/// The species counted.
const KINDS: usize = SPECIES.len();
/// Per species since the census was last reset: born, died of old age, died otherwise.
pub static BORN: [AtomicU32; KINDS] = [const { AtomicU32::new(0) }; KINDS];
pub static OLD: [AtomicU32; KINDS] = [const { AtomicU32::new(0) }; KINDS];
pub static DIED: [AtomicU32; KINDS] = [const { AtomicU32::new(0) }; KINDS];

pub fn speed() -> u32 {
    SPEED.load(Ordering::Relaxed).max(1)
}

pub fn set_speed(n: u32) {
    SPEED.store(n.clamp(1, 10_000), Ordering::Relaxed);
}

pub fn count(what: &[AtomicU32; KINDS], index: usize) {
    if let Some(c) = what.get(index) {
        c.fetch_add(1, Ordering::Relaxed);
    }
}

// ------------------------------------------------------------------ every tick

/// Every tick, for an animal of species `index` (`animal::upkeep`): its timers, its life stage
/// from how far it has grown, and a mother's young born (or eggs laid) when the time comes.
pub fn upkeep(w: &World, id: u64, index: usize, a: &mut Animal) {
    let s = &SPECIES[index];
    // Ticks since it last ticked: one with no player near rests, ticking now and then
    // (`World::tick_elapsed`), and its timers run as fast all the same.
    let e = w.tick_elapsed(id);
    let n = speed() * e;
    let Some(b) = w.component_mut(id, &BREEDING) else { return };
    // Love isn't sped up: they still have to walk to each other.
    b.love = b.love.saturating_sub(e);
    b.rest = b.rest.saturating_sub(n);
    b.growing = b.growing.saturating_sub(n);
    a.stage = life::stage(s, b.growing, a.stage);
    if b.love > 0 && (w.time() + id) % 20 == 0 {
        hearts(w, id, 1);
    }
    if b.carrying > 0 {
        b.carrying = b.carrying.saturating_sub(n);
        if b.carrying == 0 {
            let b = *b;
            deliver(w, id, index, &b);
        }
    }
    let b = *b;
    // Checked as often in game time at any speed (`/animal speed`), or a sped-up herd falls in
    // love all at once.
    let every = (WILD_LOVE_EVERY / speed() as u64).max(1);
    if (w.time() + id) % every < e as u64 && !crate::animal::tamed(w, id) {
        let days = (every * speed() as u64) as f64 / life::DAY as f64;
        let (kin, near) = kin_near(w, id, index, a);
        let roll = w.random_below(1_000_000) as f64 / 1_000_000.0;
        if roll < life::want_chance(days, near, s.crowd(), a.stage) {
            weft_sdk::info!("wild {} {id} died of want ({near:.1} of its kind near, room for {})", s.name, s.crowd());
            w.damage(id, 1e9, 0, damage::STARVE);
            return;
        }
        wild_love(w, id, index, a, &b, &kin, near, days);
    }
}

/// Its kind within `life::CROWD_RADIUS` blocks of wild one `id` (itself among them), and how
/// much they crowd it (`life::crowding`).
fn kin_near(w: &World, id: u64, index: usize, a: &Animal) -> (Vec<u64>, f64) {
    let Some(st) = w.entity(id) else { return (Vec::new(), 0.0) };
    let kin: Vec<u64> = w
        .near(st.pos, life::CROWD_RADIUS, Some(SPECIES[index].kind))
        .into_iter()
        .filter(|&o| {
            w.component(o, &ANIMAL)
                .is_some_and(|o| o.species == a.species)
        })
        .collect();
    let near = kin
        .iter()
        .filter_map(|&o| w.component(o, &ANIMAL))
        .map(|o| life::crowding(o.stage))
        .sum();
    (kin, near)
}

/// Ticks between a wild one's chances to fall in love (`wild_love`).
const WILD_LOVE_EVERY: u64 = 200;

/// A grown, rested wild one, now and then, falls in love with a wild mate near it, the more
/// readily the fewer of its kind are around (`life::love_chance`): wild animals keep their kind
/// going as old age takes them, and stop when it's crowded. They then meet and mate as tames in
/// love do (`behave`).
#[allow(clippy::too_many_arguments)]
fn wild_love(w: &World, id: u64, index: usize, a: &Animal, b: &Breeding, kin: &[u64], near: f64, days: f64) {
    let ready = |a: &Animal, b: &Breeding| life::can_breed(a.stage) && b.growing == 0 && b.love == 0 && b.rest == 0 && b.carrying == 0;
    if !ready(a, b) {
        return;
    }
    let s = &SPECIES[index];
    let Some(st) = w.entity(id) else { return };
    // Only in its breeding season, twice as readily in its peak.
    let roll = w.random_below(1_000_000) as f64 / 1_000_000.0;
    if roll >= life::love_chance(days, near, s.crowd()) * s.bio.season_love(season_bit(w)) {
        return;
    }
    let mate = kin.iter().copied().find(|&o| {
        o != id
            && !crate::animal::tamed(w, o)
            && w.entity(o)
                .is_some_and(|m| (m.pos[0] - st.pos[0]).powi(2) + (m.pos[2] - st.pos[2]).powi(2) < 24.0 * 24.0)
            && match (w.component(o, &ANIMAL), w.component(o, &BREEDING)) {
                (Some(oa), Some(ob)) => oa.female != a.female && ready(oa, ob),
                _ => false,
            }
    });
    let Some(mate) = mate else { return };
    for o in [id, mate] {
        if let Some(b) = w.component_mut(o, &BREEDING) {
            b.love = LOVE_TICKS;
        }
    }
    weft_sdk::info!("wild {} {id} and {mate} in love", s.name);
}

/// Its area was away (unloaded) for `away` ticks (`animal::catch_up`): its timers have run on (a
/// mother due gives birth on its next tick), and love has passed.
pub fn catch_up(w: &World, id: u64, away: u64) {
    let Some(b) = w.component_mut(id, &BREEDING) else { return };
    let away = away.min(u32::MAX as u64) as u32;
    b.love = 0;
    b.rest = b.rest.saturating_sub(away);
    b.growing = b.growing.saturating_sub(away);
    if b.carrying > 0 {
        b.carrying = b.carrying.saturating_sub(away).max(1);
    }
}

/// How big it is against a grown one: from its birth size up, as it grows.
pub fn growth(w: &World, id: u64, index: usize) -> f32 {
    let s = &SPECIES[index];
    w.component(id, &BREEDING)
        .map_or(1.0, |b| life::size(s, b.growing))
}

/// Still growing up.
pub fn young(w: &World, id: u64) -> bool {
    w.component(id, &BREEDING).is_some_and(|b| b.growing > 0)
}

fn hearts(w: &World, id: u64, count: u32) {
    let Some(st) = w.entity(id) else { return };
    drop(
        w.spawn_particles(&HEART, [st.pos[0], st.pos[1] + st.size[1] as f64 + 0.1, st.pos[2]])
            .count(count)
            .spread(0.3)
            .velocity([0.0, 0.6, 0.0]),
    );
}

// ------------------------------------------------------------------ love and mating

/// A player who may command `id` gives it one of its favorite foods (`commands::interact`): a
/// grown one (not an elder) with bond enough, not resting or expecting, falls in love in its
/// breeding season, or out of it if well cared for (`OUT_OF_SEASON_BOND`, `OUT_OF_SEASON_FOOD`); a
/// young one grows a tenth of what it has left. True if it took the food.
pub fn fed(w: &World, id: u64, player: u64) -> bool {
    let Some(a) = w.component(id, &ANIMAL).copied() else { return false };
    let in_season = a
        .species()
        .is_none_or(|s| s.bio.season_love(season_bit(w)) > 0.0);
    let cared = a.bond >= OUT_OF_SEASON_BOND && a.food as f32 >= a.stat(species::FOOD) * OUT_OF_SEASON_FOOD;
    let Some(b) = w.component_mut(id, &BREEDING) else { return false };
    if b.growing > 0 {
        b.growing -= b.growing / 10;
    } else if b.love == 0 && b.rest == 0 && b.carrying == 0 && a.bond >= BOND && life::can_breed(a.stage) && (in_season || cared) {
        b.love = LOVE_TICKS;
        weft_sdk::info!("animal {id} in love");
    } else {
        return false;
    }
    crate::taming::take_one(w, player);
    hearts(w, id, 5);
    true
}

/// A goal for every species, after its commands: one in love goes to a mate in love (one of its
/// species and the other sex, grown) and, meeting, they mate; a young one keeps near its mother.
pub fn behave(index: usize) -> impl Fn(&World, u64, &EntityState) -> bool + Send + Sync + 'static {
    move |w, id, st| {
        let Some(b) = w.component(id, &BREEDING).copied() else { return false };
        if b.growing > 0 {
            return follow_mother(w, id, index, st, &b);
        }
        if b.love == 0 {
            return false;
        }
        let female = |o: u64| w.component(o, &ANIMAL).map(|a| a.female);
        let Some(mine) = w.component(id, &ANIMAL).and_then(|a| a.species()) else { return false };
        let mate = w.near(st.pos, 24.0, None).into_iter().find(|&o| {
            o != id
                && female(o) != female(id)
                && w.component(o, &ANIMAL)
                    .and_then(|a| a.species())
                    .is_some_and(|s| species::same_species(s, mine))
                && w.component(o, &BREEDING)
                    .is_some_and(|m| m.love > 0 && m.growing == 0)
        });
        let Some(mate) = mate else { return false };
        let Some(ms) = w.entity(mate) else { return false };
        let d2 = (ms.pos[0] - st.pos[0]).powi(2) + (ms.pos[2] - st.pos[2]).powi(2);
        if d2 > 1.5 * 1.5 {
            if crate::animal::tamed(w, id) && crate::commands::command(w, id) == crate::commands::STAY {
                return false;
            }
            crate::wake(w, id, index);
            w.navigate(id, ms.pos, 1.0);
            return true;
        }
        // They meet: the female does the rest.
        if female(id) == Some(1) {
            mated(w, id, mate, index);
        }
        true
    }
}

/// Mother `id` and father `mate` have mated: she carries his young (or will lay his eggs); both
/// rest.
fn mated(w: &World, id: u64, mate: u64, index: usize) {
    let s = &SPECIES[index];
    let (Some(genes), Some(wild)) = (w.component(mate, &GENOME).copied(), w.component(mate, &ANIMAL).map(|a| a.wild)) else {
        return;
    };
    if let Some(b) = w.component_mut(id, &BREEDING) {
        b.love = 0;
        b.carrying = life::carrying(s);
        b.rest = b.carrying + s.recovery();
        (b.mate, b.mate_wild) = (mate, wild);
        (b.mate_genes, b.mate_more) = pack(&genes);
    }
    if let Some(b) = w.component_mut(mate, &BREEDING) {
        (b.love, b.rest) = (0, MALE_REST);
    }
    hearts(w, id, 7);
    weft_sdk::info!("animal {id} mated with {mate}");
}

/// A young one keeps within three blocks of its mother, while she is near.
fn follow_mother(w: &World, id: u64, index: usize, st: &EntityState, b: &Breeding) -> bool {
    if crate::animal::tamed(w, id) && crate::commands::command(w, id) != crate::commands::WANDER {
        return false;
    }
    let Some(m) = w.entity(b.mother).filter(|_| b.mother != 0) else { return false };
    let d2 = (m.pos[0] - st.pos[0]).powi(2) + (m.pos[2] - st.pos[2]).powi(2);
    if d2 < 3.0 * 3.0 || d2 > 32.0 * 32.0 {
        return false;
    }
    if (w.time() + id) % 10 == 0 || !w.navigating(id) {
        crate::wake(w, id, index);
        w.navigate(id, m.pos, 1.2);
    }
    true
}

// ------------------------------------------------------------------ births and eggs

/// What a young one is: its genes, family and wild points.
pub struct Young {
    pub index: usize,
    pub genome: Genome,
    pub mother: u64,
    pub father: u64,
    pub generation: u32,
    /// Its parents' genes (species 0: unknown).
    pub dam: Genome,
    pub sire: Genome,
    pub wild: [u8; STATS],
}

/// The time has come for mother `id`: her litter is born around her, or her clutch laid.
fn deliver(w: &World, id: u64, index: usize, b: &Breeding) {
    let s = &SPECIES[index];
    let (Some(mother), Some(father), Some(st), Some(a)) =
        (w.component(id, &GENOME).copied(), unpack(&b.mate_genes, &b.mate_more), w.entity(id), w.component(id, &ANIMAL).copied())
    else {
        return;
    };
    let mut r = coats::world_rng(w);
    let n = life::litter(s, w.random_below(u32::MAX));
    let father_gen = w.component(b.mate, &BREEDING).map_or(0, |f| f.generation);
    let owner = w.component(id, &OWNER).copied().filter(|o| o[0] != 0);
    let mut clutch = Vec::new();
    for k in 0..n {
        let Some(genome) = coats::breed(&mother, &father, &mut r) else { continue };
        let wild = life::wild_points(&a.wild, &b.mate_wild, |n| w.random_below(n));
        let y = Young {
            index,
            genome,
            mother: id,
            father: b.mate,
            generation: b.generation.max(father_gen) + 1,
            dam: mother,
            sire: father,
            wild,
        };
        let angle = k as f64 / n as f64 * std::f64::consts::TAU;
        let at = [st.pos[0] + angle.cos() * 0.6, st.pos[1], st.pos[2] + angle.sin() * 0.6];
        if s.gestation() > 0 {
            born(w, at, &y, owner);
        } else {
            clutch.push(y);
        }
    }
    crate::eggs::lay(w, st.pos, &clutch, owner);
    hearts(w, id, 9);
    let what = if s.gestation() > 0 { "young born" } else { "eggs laid" };
    weft_sdk::info!("animal {id}: {n} {what}");
}

/// A young one comes into the world at `at`: its genes, its stats from its parents', growing
/// up from now, its owner's (and wandering after its mother) if its mother was someone's.
pub fn born(w: &World, at: [f64; 3], y: &Young, owner: Option<[u8; 64]>) -> u64 {
    let s = &SPECIES[y.index];
    let id = w.spawn(s.kind, at);
    if id == 0 {
        return 0;
    }
    count(&BORN, y.index);
    if let Some(g) = w.component_mut(id, &GENOME) {
        *g = y.genome;
    }
    if let Some(a) = w.component_mut(id, &ANIMAL) {
        let mut new = Animal {
            species: y.index as u8 + 1,
            female: y.genome.female,
            stage: animal::BABY,
            born: life::birth(w.time(), 0),
            bond: if owner.is_some() { 25 } else { 0 },
            wild: y.wild,
            ..Animal::default()
        };
        new.food = new.stat(species::FOOD).round() as u16;
        *a = new;
    }
    if let Some(b) = w.component_mut(id, &BREEDING) {
        *b = Breeding { growing: s.growth(), generation: y.generation, mother: y.mother, father: y.father, ..Breeding::default() };
        (b.dam, b.dam_more) = pack(&y.dam);
        (b.sire, b.sire_more) = pack(&y.sire);
    }
    if let Some(o) = owner {
        if let Some(own) = w.component_mut(id, &OWNER) {
            *own = o;
        }
        crate::xp::raised(w, &o);
        crate::commands::tamed(w, id);
        crate::commands::set_command(w, id, crate::commands::WANDER, None);
        crate::dog::on_tamed(w, id);
    }
    let coat = coats::phenotype(&y.genome)
        .map(|l| l.name)
        .unwrap_or_default();
    let whose = if owner.is_some() { "tamed" } else { "wild" };
    weft_sdk::info!("{} {id} born to {} and {} ({whose}): {coat}", s.kind_name(), y.mother, y.father);
    id
}

// ------------------------------------------------------------------ testing

/// `/animal love` (for testing): every grown animal within 12 blocks of `player` becomes theirs,
/// bonded enough, wanders and falls in love.
pub fn love_near(w: &World, player: u64) {
    let Some(p) = w.entity(player) else { return };
    for id in w.near(p.pos, 12.0, None) {
        if w.component(id, &BREEDING).is_none_or(|b| b.growing > 0) {
            continue;
        }
        if !crate::animal::tamed(w, id) {
            crate::taming::tame(w, id, player, 0.0);
        }
        crate::commands::set_command(w, id, crate::commands::WANDER, None);
        if let Some(a) = w.component_mut(id, &ANIMAL) {
            a.bond = a.bond.max(BOND);
        }
        if let Some(b) = w.component_mut(id, &BREEDING) {
            (b.love, b.rest) = (LOVE_TICKS, 0);
        }
    }
}

/// `/animal hurry` (for testing): every mother near `player` gives birth or lays within a
/// second, every egg hatches, every young one grows up.
pub fn hurry_near(w: &World, player: u64) {
    let Some(p) = w.entity(player) else { return };
    for id in w.near(p.pos, 16.0, None) {
        if let Some(b) = w.component_mut(id, &BREEDING) {
            b.carrying = b.carrying.min(20);
            b.growing = b.growing.min(20);
        }
    }
    crate::eggs::hurry(w, p.pos);
}
