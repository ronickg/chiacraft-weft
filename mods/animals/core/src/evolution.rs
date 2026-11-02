//! Evolution (`notes/ANIMALS.md`, "Evolution"): how common each gene variant is changes where the
//! animals live, as it does in real populations. Every species has a gene pool per place (a cell
//! of 256 x 256 blocks and its habitat): wild animals spawning there are drawn from it
//! (`coats::upkeep`), and it starts as the species' gene file has it. Each in-game day is a
//! generation: a population of the pool lives, the ones whose coat stands out from the habitat
//! (sandy raptors on grass, dark ones on snow) leave fewer young (natural selection, by the
//! species' `camouflage`), chance decides much in a small population (drift), alleles now and
//! then mutate, and a little of each neighboring place's pool mixes in (migration). Mammals also
//! grow bigger where it is cold and smaller where it is hot (Bergmann's rule). So raptors in
//! deserts turn sandy over a few weeks, ones in forests dark, and neighbors stay alike.
//!
//! Players change them too: a wild animal that dies (hunted or otherwise) or is tamed takes its
//! genes out of its place's pool, and a tame its owner releases puts its genes in (`count`).
//!
//! Pools evolve when they are next needed (up to `CATCH_UP` generations at once), as numbers, not
//! animals, so a pool far from everyone costs nothing. They are saved in the world
//! (`mods/animals/pools.json`, through the engine's bridge).

use crate::coats::{self, Pool};
use crate::species::SPECIES;

/// Blocks a side of a pool's place.
pub const CELL: f64 = 256.0;
/// Ticks a generation (an in-game day).
pub const GENERATION: u64 = 24_000;
/// Animals a generation: a small population, so chance matters (drift) without wiping out
/// every rare variant within weeks.
pub const POPULATION: usize = 200;
/// How much standing out from the habitat costs, for a species with `camouflage` 1: a coat as
/// far from it as colors go leaves this much fewer young.
pub const SELECTION: f32 = 0.6;
/// The chance per locus and young that an allele mutates.
pub const MUTATION: f32 = 0.002;
/// How much size matters to a mammal in the cold (or heat): at the size gene's ends, this much
/// more (or fewer) young in snow.
pub const BERGMANN: f32 = 0.15;
/// The share of each generation from the neighboring places.
pub const MIGRATION: f32 = 0.02;
/// Generations a pool catches up at once when it is needed again after a long time.
pub const CATCH_UP: u64 = 60;

/// Where animals live, as their coats are seen against it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Habitat {
    Grassland,
    Forest,
    Taiga,
    Snow,
    Desert,
    Mountains,
}

pub const HABITATS: [Habitat; 6] =
    [Habitat::Grassland, Habitat::Forest, Habitat::Taiga, Habitat::Snow, Habitat::Desert, Habitat::Mountains];

impl Habitat {
    pub fn name(self) -> &'static str {
        match self {
            Habitat::Grassland => "grassland",
            Habitat::Forest => "forest",
            Habitat::Taiga => "taiga",
            Habitat::Snow => "snow",
            Habitat::Desert => "desert",
            Habitat::Mountains => "mountains",
        }
    }

    pub fn named(name: &str) -> Option<Habitat> {
        HABITATS.into_iter().find(|h| h.name() == name)
    }

    /// How cold it is: 1 in snow and taiga, -1 in the desert's heat.
    pub fn cold(self) -> f32 {
        match self {
            Habitat::Snow | Habitat::Taiga => 1.0,
            Habitat::Mountains => 0.5,
            Habitat::Desert => -1.0,
            Habitat::Grassland | Habitat::Forest => 0.0,
        }
    }

    /// What an animal is seen against there: the ground and plants around it, as a predator or
    /// prey sees them.
    pub fn background(self) -> [f32; 3] {
        match self {
            Habitat::Grassland => [138.0, 128.0, 84.0],
            Habitat::Forest => [78.0, 70.0, 52.0],
            Habitat::Taiga => [72.0, 68.0, 60.0],
            Habitat::Snow => [232.0, 236.0, 240.0],
            Habitat::Desert => [208.0, 184.0, 136.0],
            Habitat::Mountains => [128.0, 124.0, 118.0],
        }
    }
}

// ------------------------------------------------------------------ generations

/// How many young an animal of species `index` leaves, relative: less the more its coat stands
/// out; and for mammals, more the bigger it is in the cold and the smaller in the heat (Bergmann's
/// rule: a big body loses less heat).
pub fn fitness(index: usize, g: &coats::Genome, h: Habitat) -> f32 {
    let s = &SPECIES[index];
    let Some(look) = coats::phenotype(g) else { return 1.0 };
    let (c, bg) = (look.average(), h.background());
    let d = ((c[0] - bg[0]).powi(2) + (c[1] - bg[1]).powi(2) + (c[2] - bg[2]).powi(2)).sqrt() / 441.7;
    let mut f = (1.0 - SELECTION * s.camouflage * d).max(0.05);
    if s.family.above.contains(&"Mammalia") {
        f *= 1.0 + BERGMANN * h.cold() * (coats::size_gene(g) - 0.5) * 2.0;
    }
    f
}

/// One generation of species `index`'s pool in habitat `h`: `POPULATION` animals drawn from it,
/// parents chosen by fitness, their young making the next pool.
pub fn generation(index: usize, pool: &Pool, h: Habitat, r: &mut impl coats::Rng) -> Option<Pool> {
    let adults: Vec<(coats::Genome, f32)> = (0..POPULATION)
        .filter_map(|i| coats::sample(index, i % 2 == 0, pool, r))
        .map(|g| (g, fitness(index, &g, h)))
        .collect();
    let parent = |female: bool, r: &mut dyn FnMut() -> f32| {
        let of: Vec<&(coats::Genome, f32)> = adults
            .iter()
            .filter(|a| (a.0.female != 0) == female)
            .collect();
        let mut x = r() * of.iter().map(|a| a.1).sum::<f32>();
        for a in &of {
            x -= a.1;
            if x <= 0.0 {
                return Some(a.0);
            }
        }
        of.last().map(|a| a.0)
    };
    let mut rnd = || r.next();
    let young: Vec<coats::Genome> = (0..POPULATION)
        .filter_map(|_| {
            let (m, f) = (parent(true, &mut rnd)?, parent(false, &mut rnd)?);
            coats::breed_with(&m, &f, MUTATION, &mut rnd)
        })
        .collect();
    coats::pool_of(index, &young)
}

/// `pool` with `share` of `other` mixed in (allele shares, a trait's mean and spread alike).
pub fn mix_in(pool: &mut Pool, other: &Pool, share: f32) {
    for (a, b) in pool.iter_mut().zip(other) {
        if a.len() == b.len() {
            for (x, y) in a.iter_mut().zip(b) {
                *x += (y - *x) * share;
            }
        }
    }
}
