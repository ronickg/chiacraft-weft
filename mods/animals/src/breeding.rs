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
    }
}

pub static BREEDING: Component<Breeding> = Component::named("breeding");
static HEART: Particle = Particle::named("heart");

/// Thirty seconds in love; a male rests five minutes after mating.
const LOVE_TICKS: u32 = 600;
const MALE_REST: u32 = 6_000;
/// The bond breeding needs.
const BOND: u8 = 50;
/// Five minutes after mating, a raptor lays.
const LAYING: u32 = 6_000;

const _: () = assert!(size_of::<Genome>() <= 40);

/// A genome kept as words, as `Breeding` and `eggs::Egg` keep them.
pub fn pack(g: &Genome) -> [u32; 10] {
    let mut out = [0u32; 10];
    unsafe { std::ptr::copy_nonoverlapping(g as *const Genome as *const u8, out.as_mut_ptr() as *mut u8, size_of::<Genome>()) };
    out
}

/// A genome kept as words (`pack`), if any.
pub fn unpack(b: &[u32; 10]) -> Option<Genome> {
    let g: Genome = unsafe { std::ptr::read_unaligned(b.as_ptr() as *const Genome) };
    (g.species != 0).then_some(g)
}

pub fn init(r: &mut Registry) {
    r.entity_component(&BREEDING, Breeding::default(), component_flags::SAVE | component_flags::SYNC_MENU);
    for s in &SPECIES {
        r.attach(s.kind, &BREEDING);
    }
}

// ------------------------------------------------------------------ every tick

/// Every tick, for an animal of species `index` (`animal::upkeep`): its timers, its life stage
/// from how far it has grown, and a mother's young born (or eggs laid) when the time comes.
pub fn upkeep(w: &World, id: u64, index: usize, a: &mut Animal) {
    let s = &SPECIES[index];
    let Some(b) = w.component_mut(id, &BREEDING) else { return };
    b.love = b.love.saturating_sub(1);
    b.rest = b.rest.saturating_sub(1);
    b.growing = b.growing.saturating_sub(1);
    a.stage = match b.growing * 3 / s.growth.max(1) {
        _ if b.growing == 0 => a.stage.max(animal::ADULT),
        2.. => animal::BABY,
        1 => animal::JUVENILE,
        _ => animal::ADOLESCENT,
    };
    if b.love > 0 && (w.time() + id) % 20 == 0 {
        hearts(w, id, 1);
    }
    if b.carrying > 0 {
        b.carrying -= 1;
        if b.carrying == 0 {
            let b = *b;
            deliver(w, id, index, &b);
        }
    }
}

/// How big it is against a grown one: from its birth size up, as it grows.
pub fn growth(w: &World, id: u64, index: usize) -> f32 {
    let s = &SPECIES[index];
    let Some(b) = w.component(id, &BREEDING).filter(|b| b.growing > 0) else { return 1.0 };
    let grown = 1.0 - b.growing as f32 / s.growth.max(1) as f32;
    s.birth_size + (1.0 - s.birth_size) * grown
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
/// grown one with bond enough, not resting or expecting, falls in love; a young one grows a
/// tenth of what it has left. True if it took the food.
pub fn fed(w: &World, id: u64, player: u64) -> bool {
    let Some(a) = w.component(id, &ANIMAL).copied() else { return false };
    let Some(b) = w.component_mut(id, &BREEDING) else { return false };
    if b.growing > 0 {
        b.growing -= b.growing / 10;
    } else if b.love == 0 && b.rest == 0 && b.carrying == 0 && a.bond >= BOND {
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
        let mate = w.near(st.pos, 8.0, None).into_iter().find(|&o| {
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
        b.carrying = if s.gestation > 0 { s.gestation } else { LAYING };
        b.rest = b.carrying + s.recovery;
        (b.mate, b.mate_genes, b.mate_wild) = (mate, pack(&genes), wild);
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
    pub dam: [u32; 10],
    pub sire: [u32; 10],
    pub wild: [u8; STATS],
}

/// The time has come for mother `id`: her litter is born around her, or her clutch laid.
fn deliver(w: &World, id: u64, index: usize, b: &Breeding) {
    let s = &SPECIES[index];
    let (Some(mother), Some(father), Some(st), Some(a)) =
        (w.component(id, &GENOME).copied(), unpack(&b.mate_genes), w.entity(id), w.component(id, &ANIMAL).copied())
    else {
        return;
    };
    let mut r = coats::world_rng(w);
    let n = s.litter.0 + w.random_below(s.litter.1 - s.litter.0 + 1);
    let father_gen = w.component(b.mate, &BREEDING).map_or(0, |f| f.generation);
    let owner = w.component(id, &OWNER).copied().filter(|o| o[0] != 0);
    let mut clutch = Vec::new();
    for k in 0..n {
        let Some(genome) = coats::breed(&mother, &father, &mut r) else { continue };
        // Each stat's wild points from one parent, the stronger 55% of the time.
        let wild = std::array::from_fn(|i| {
            let (hi, lo) = (a.wild[i].max(b.mate_wild[i]), a.wild[i].min(b.mate_wild[i]));
            if w.random_below(100) < 55 { hi } else { lo }
        });
        let y = Young {
            index,
            genome,
            mother: id,
            father: b.mate,
            generation: b.generation.max(father_gen) + 1,
            dam: pack(&mother),
            sire: b.mate_genes,
            wild,
        };
        let angle = k as f64 / n as f64 * std::f64::consts::TAU;
        let at = [st.pos[0] + angle.cos() * 0.6, st.pos[1], st.pos[2] + angle.sin() * 0.6];
        if s.gestation > 0 {
            born(w, at, &y, owner);
        } else {
            clutch.push(y);
        }
    }
    crate::eggs::lay(w, st.pos, &clutch, owner);
    hearts(w, id, 9);
    let what = if s.gestation > 0 { "young born" } else { "eggs laid" };
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
    if let Some(g) = w.component_mut(id, &GENOME) {
        *g = y.genome;
    }
    if let Some(a) = w.component_mut(id, &ANIMAL) {
        let mut new = Animal {
            species: y.index as u8 + 1,
            female: y.genome.female,
            stage: animal::BABY,
            bond: if owner.is_some() { 25 } else { 0 },
            wild: y.wild,
            ..Animal::default()
        };
        new.food = new.stat(species::FOOD).round() as u16;
        *a = new;
    }
    if let Some(b) = w.component_mut(id, &BREEDING) {
        *b = Breeding {
            growing: s.growth,
            generation: y.generation,
            mother: y.mother,
            father: y.father,
            dam: y.dam,
            sire: y.sire,
            ..Breeding::default()
        };
    }
    if let Some(o) = owner {
        if let Some(own) = w.component_mut(id, &OWNER) {
            *own = o;
        }
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
