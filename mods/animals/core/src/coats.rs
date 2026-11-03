//! Coats from genes (`notes/ANIMALS.md`, "Looks"). Every animal carries a genome (`GENOME`, saved,
//! and synced to everyone who sees it): drawn from its species' gene file
//! (`genetics/<species>.json`; a breed's own frequencies for a breed) when it spawns, or one
//! allele of each locus from each parent when it is born. Each client paints its coat from that
//! (`Registry::entity_skin`): the phenotype (real coat genetics, simplified, per species) on the
//! coat layers its model's build wrote (`genetics/layers/<model>`: what is coat, its light, where
//! on the body, where white, points, a mask and patterns show). Only the genome travels.
//!
//! The core every species shares (`genetics/core.json`, "Looks, part 3" in the notes): traits
//! after a species' own loci (size, shade, warmth, white, the marks' size, density, contrast,
//! edge and stretch, eyes, build), inherited alike, each showing partly by chance (its
//! heritability), shifted by sex and youth; scars from life. A species' own rules decide its
//! colors and which patterns it has; the core decides how each individual wears them.
//!
//! The website runs this very code (`mods/animals/web`, compiled to WASM: the viewer's coat
//! panel and breeding lab), so what is tried there is what the game does.

use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::species::SPECIES;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
/// An animal's genes; its coat is painted from them.
pub struct Genome {
    /// Where its patterns and white fall.
    pub seed: u32,
    /// Species + 1 (0: none yet).
    pub species: u8,
    /// Its breed + 1, in its gene file's `breeds` (0: none; `MIXED`: parents of two breeds).
    pub breed: u8,
    pub female: u8,
    /// Two per locus of the species' own, in the gene file's order: an allele's index
    /// (`NONE`: a male's missing second X, or a locus it doesn't have), or a quantitative
    /// trait's value (0-254 for 0-1).
    pub genes: [u8; 32],
    /// Two per core trait (`genetics/core.json`, in its order), as `genes`.
    pub core: [u8; 32],
    /// Its layout (`VERSION`; 0: saved before the core traits, given them at `upkeep`).
    pub version: u8,
    /// Its life stage (`animal::BABY`..), for a young one's look.
    pub stage: u8,
    /// Marks life left on it: up to four scars, each a kind (`SCARS`, 0: none) and where.
    pub scars: [u8; 8],
}

pub const MIXED: u8 = 255;
pub const NONE: u8 = 255;
/// Loci of a species' own (`Genome::genes`); the core traits' slots come after them.
pub const OWN: usize = 16;
/// Every slot: a species' own and the core.
pub const SLOTS: usize = 32;
/// The genome's layout now.
pub const VERSION: u8 = 1;

/// Scar kinds (`Genome::scars`): three raked lines of claws, a ring of tooth marks, a blade's
/// long line, a burn, torn wing skin.
pub const CLAW: u8 = 1;
pub const BITE: u8 = 2;
pub const SLASH: u8 = 3;
pub const BURN: u8 = 4;
pub const TEAR: u8 = 5;

/// `g` keeps a scar of `kind` at `place` (any byte: where on its body), the oldest of four going.
pub fn scar(g: &mut Genome, kind: u8, place: u8) {
    let free = (0..4).find(|&i| g.scars[2 * i] == 0).unwrap_or_else(|| {
        g.scars.copy_within(2..8, 0);
        3
    });
    g.scars[2 * free] = kind;
    g.scars[2 * free + 1] = place;
}

/// A genome kept before the core traits (version 0, zeros where they now are): none of them
/// yet, so they read as the middle until `upkeep` draws them.
pub fn mend(g: &mut Genome) {
    if g.version == 0 {
        g.core = [NONE; 32];
        g.stage = crate::species::ADULT;
    }
}

impl Genome {
    /// Gene `k` (0, 1) of slot `slot` (a species' own locus, or `OWN` + a core trait).
    fn get(&self, slot: usize, k: usize) -> u8 {
        if slot < OWN { self.genes[2 * slot + k] } else { self.core[2 * (slot - OWN) + k] }
    }
    fn set(&mut self, slot: usize, k: usize, v: u8) {
        if slot < OWN {
            self.genes[2 * slot + k] = v;
        } else {
            self.core[2 * (slot - OWN) + k] = v;
        }
    }
    fn pair(&self, slot: usize) -> [u8; 2] {
        [self.get(slot, 0), self.get(slot, 1)]
    }
    /// One with no genes in any slot yet.
    fn empty(species: u8, breed: u8, female: bool, seed: u32) -> Genome {
        Genome {
            seed,
            species,
            breed,
            female: female as u8,
            genes: [NONE; 32],
            core: [NONE; 32],
            version: VERSION,
            stage: crate::species::ADULT,
            ..Genome::default()
        }
    }
}

// ------------------------------------------------------------------ gene files

#[derive(Deserialize)]
struct GeneFile {
    /// Its own loci (slots 0..), then the core traits it doesn't have of its own (`core.json`,
    /// slots `OWN`..; added when it is read).
    loci: Vec<Locus>,
    /// By name, in name order (a genome keeps its breed as an index into these).
    #[serde(default)]
    breeds: BTreeMap<String, Breed>,
    /// Where wild ones start from in each habitat (`evolution::Habitat::name`), as a breed's
    /// frequencies: the kinds of a species living in different country (leopards' rosettes are
    /// rounder in East Africa, squarer in the south).
    #[serde(default)]
    habitats: HashMap<String, Breed>,
    /// The core traits' mean and spread for this species (`Ms`: its marks' size...).
    #[serde(default)]
    traits: HashMap<String, Spread>,
    /// What being male or female shifts (`male`: `{"Ct": 0.15}`, starker displays).
    #[serde(default)]
    sexes: HashMap<String, HashMap<String, f32>>,
    /// How a young one looks: shifts at birth, fading as it grows up.
    #[serde(default)]
    young: Young,
    /// Its eye colors, along the eye trait (`Ey`) from 0 to 1.
    #[serde(default)]
    eyes: Vec<[f32; 3]>,
    /// Model units to a pattern's (a big model's patterns drawn bigger to match: the wyvern's
    /// model is at the house density, 64 texels a block).
    #[serde(default = "one")]
    mark_scale: f32,
    /// Its build: per core build trait (`Nk`: neck...), the bones it scales and how far.
    #[serde(default)]
    build: HashMap<String, Stretch>,
}

/// How a build trait shows on a model: its bones drawn `1 + range` times as big at the trait's
/// top and `1 - range` at its bottom, along the model's axes as much as `axes` says (a neck
/// longer along y and z, hardly thicker across x).
#[derive(Deserialize)]
struct Stretch {
    bones: Vec<String>,
    #[serde(default = "ones")]
    axes: [f32; 3],
    range: f32,
}

fn ones() -> [f32; 3] {
    [1.0; 3]
}

fn one() -> f32 {
    1.0
}

#[derive(Deserialize, Default)]
struct Young {
    #[serde(default)]
    traits: HashMap<String, f32>,
    /// A pattern it is born with and loses (a hatchling's speckles, a cub's spots).
    #[serde(default)]
    pattern: Option<String>,
}

#[derive(Deserialize, Clone)]
struct Locus {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    about: String,
    #[serde(default)]
    alleles: Vec<Allele>,
    /// Carried on the X: males have one copy.
    #[serde(default)]
    xlinked: bool,
    /// Two values, the mean shows.
    #[serde(default)]
    quantitative: bool,
    #[serde(default)]
    mean: f32,
    #[serde(default)]
    sd: f32,
    /// Heritability: the share of its spread from genes, the rest its own chance (a core
    /// trait's when a species has it as its own; 1 if neither says).
    #[serde(default)]
    h: Option<f32>,
    /// Its slot in a genome (`Genome::get`).
    #[serde(skip)]
    slot: usize,
}

#[derive(Deserialize, Clone)]
struct Allele {
    a: String,
    #[serde(default)]
    freq: f32,
    #[serde(default)]
    about: String,
}

#[derive(Deserialize, Default)]
struct Breed {
    #[serde(default)]
    freq: HashMap<String, HashMap<String, f32>>,
    #[serde(default)]
    quantitative: HashMap<String, Spread>,
}

#[derive(Deserialize, Clone, Copy)]
struct Spread {
    mean: f32,
    sd: f32,
}

/// `species` with the core traits added after its own loci.
fn with_core(mut g: GeneFile, core: &GeneFile, name: &str) -> GeneFile {
    if g.loci.len() > OWN {
        crate::problem(format!("genetics/{name}.json: more than {OWN} loci; the rest are left out"));
        g.loci.truncate(OWN);
    }
    for (i, l) in g.loci.iter_mut().enumerate() {
        l.slot = i;
    }
    for (i, c) in core.loci.iter().enumerate().take(SLOTS - OWN) {
        if let Some(own) = g.loci.iter_mut().find(|l| l.id == c.id) {
            if !own.quantitative {
                crate::problem(format!("genetics/{name}.json: locus {} is also a core trait's id", c.id));
            }
            own.h = own.h.or(c.h);
            continue;
        }
        let mut l = c.clone();
        l.slot = OWN + i;
        if let Some(t) = g.traits.get(&c.id) {
            (l.mean, l.sd) = (t.mean, t.sd);
        }
        g.loci.push(l);
    }
    g
}

const CORE: &str = include_str!("../../genetics/core.json");

const FILES: [(&str, &str); 10] = [
    ("dog", include_str!("../../genetics/dog.json")),
    ("cat", include_str!("../../genetics/cat.json")),
    ("capybara", include_str!("../../genetics/capybara.json")),
    ("raptor", include_str!("../../genetics/raptor.json")),
    ("utahraptor", include_str!("../../genetics/utahraptor.json")),
    ("osmolskae", include_str!("../../genetics/osmolskae.json")),
    ("wyvern", include_str!("../../genetics/wyvern.json")),
    ("hare", include_str!("../../genetics/hare.json")),
    ("mountain_hare", include_str!("../../genetics/mountain_hare.json")),
    ("wood_mouse", include_str!("../../genetics/wood_mouse.json")),
];

/// A species' gene file (by `Species::name`).
fn gene_file(species: &str) -> Option<&'static GeneFile> {
    static PARSED: OnceLock<Vec<(&'static str, GeneFile)>> = OnceLock::new();
    PARSED
        .get_or_init(|| {
            let core: GeneFile = serde_json::from_str(CORE).unwrap_or_else(|e| {
                crate::problem(format!("genetics/core.json: {e}"));
                serde_json::from_str("{\"loci\":[]}").unwrap()
            });
            FILES
                .iter()
                .filter_map(|(n, json)| match serde_json::from_str(json) {
                    Ok(f) => Some((*n, with_core(f, &core, n))),
                    Err(e) => {
                        crate::problem(format!("genetics/{n}.json: {e}"));
                        None
                    }
                })
                .collect()
        })
        .iter()
        .find(|(n, _)| *n == species)
        .map(|(_, f)| f)
}

// ------------------------------------------------------------------ genomes

/// Uniform 0..1 numbers.
pub trait Rng {
    fn next(&mut self) -> f32;
}

impl<F: FnMut() -> f32> Rng for F {
    fn next(&mut self) -> f32 {
        self()
    }
}

fn gauss(r: &mut impl Rng) -> f32 {
    (-2.0 * (r.next() + 1e-9).ln()).sqrt() * (std::f32::consts::TAU * r.next()).cos()
}

/// A quantitative trait's value, 0..1, as a gene keeps it (0-254; `NONE` stays "missing").
fn unit(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 254.0).round() as u8
}

/// A quantitative trait's value back, 0..1: the middle when missing (a genome from before the
/// trait was added).
fn value(a: u8) -> f32 {
    if a == NONE { 0.5 } else { a as f32 / 254.0 }
}

/// One of `shares.len()` alleles, each as likely as its share.
fn pick(shares: &[f32], r: &mut impl Rng) -> u8 {
    let mut x = r.next() * shares.iter().sum::<f32>();
    for (i, s) in shares.iter().enumerate() {
        x -= s;
        if x <= 0.0 {
            return i as u8;
        }
    }
    shares.len().saturating_sub(1) as u8
}

/// An allele as common as it is in the gene file.
fn wild_allele(l: &Locus, r: &mut impl Rng) -> u8 {
    pick(&l.alleles.iter().map(|a| a.freq).collect::<Vec<_>>(), r)
}

/// A gene pool: per slot of a species' genome (its own loci, then the core traits), each
/// allele's share, or a quantitative trait's mean and spread (`[mean, sd]`); empty for a slot it
/// doesn't use. How common each variant is in a population; what evolves (`evolution.rs`).
pub type Pool = Vec<Vec<f32>>;

/// The breed species `index` is (+ 1, as `Genome::breed`), and its frequencies.
fn breed_of(index: usize, g: &GeneFile) -> (u8, Option<&Breed>) {
    SPECIES[index]
        .breed
        .and_then(|n| {
            g.breeds
                .iter()
                .position(|(k, _)| k == n)
                .map(|i| (i as u8 + 1, &g.breeds[n]))
        })
        .map_or((0, None), |(i, b)| (i, Some(b)))
}

/// Locus `l` as the wild has it: a breed's frequencies over a habitat's over the gene file's.
fn wild_of(l: &Locus, over: &[&Breed]) -> Vec<f32> {
    if l.quantitative {
        let q = over.iter().find_map(|b| b.quantitative.get(&l.id));
        return q.map_or(vec![l.mean, l.sd], |q| vec![q.mean, q.sd]);
    }
    l.alleles
        .iter()
        .map(|a| {
            over.iter()
                .find_map(|b| b.freq.get(&l.id).and_then(|m| m.get(&a.a)))
                .copied()
                .unwrap_or(a.freq)
        })
        .collect()
}

/// Species `index`'s pool in `habitat` as its gene file has it (its breed's frequencies for a
/// breed, else the habitat's kind): where every local pool starts.
pub fn wild_pool(index: usize, habitat: &str) -> Option<Pool> {
    let g = gene_file(SPECIES.get(index)?.name)?;
    let (_, b) = breed_of(index, g);
    let over: Vec<&Breed> = b.into_iter().chain(g.habitats.get(habitat)).collect();
    let mut pool = vec![Vec::new(); SLOTS];
    for l in &g.loci {
        pool[l.slot] = wild_of(l, &over);
    }
    Some(pool)
}

/// A new animal of species `index` drawn from `pool` (one copy on the X for males); a slot the
/// pool doesn't have yet (saved before it was added) as the gene file has it.
pub fn sample(index: usize, female: bool, pool: &Pool, r: &mut impl Rng) -> Option<Genome> {
    let g = gene_file(SPECIES.get(index)?.name)?;
    let (breed, _) = breed_of(index, g);
    let mut out = Genome::empty(index as u8 + 1, breed, female, (r.next() * 1e9) as u32);
    for l in &g.loci {
        draw(l, female, pool.get(l.slot).filter(|p| !p.is_empty()), &mut out, r);
    }
    Some(out)
}

/// Locus `l` of `out` drawn from the pool's `p` (or the gene file).
fn draw(l: &Locus, female: bool, p: Option<&Vec<f32>>, out: &mut Genome, r: &mut impl Rng) {
    if l.quantitative {
        let (mean, sd) = p.map_or((l.mean, l.sd), |p| (p.first().copied().unwrap_or(l.mean), p.get(1).copied().unwrap_or(l.sd)));
        for k in 0..2 {
            out.set(l.slot, k, unit(mean + sd * gauss(r)));
        }
    } else {
        let copies = if l.xlinked && !female { 1 } else { 2 };
        for k in 0..copies {
            let a = match p {
                Some(p) if p.len() == l.alleles.len() => pick(p, r),
                _ => wild_allele(l, r),
            };
            out.set(l.slot, k, a);
        }
    }
}

/// The pool `genomes` (of one species) make: how common each allele is among them, each
/// quantitative trait's mean and spread.
pub fn pool_of(index: usize, genomes: &[Genome]) -> Option<Pool> {
    let g = gene_file(SPECIES.get(index)?.name)?;
    let mut pool = vec![Vec::new(); SLOTS];
    for l in &g.loci {
        let values = genomes
            .iter()
            .flat_map(|x| x.pair(l.slot))
            .filter(|&a| a != NONE);
        pool[l.slot] = if l.quantitative {
            let v: Vec<f32> = values.map(value).collect();
            let n = v.len().max(1) as f32;
            let mean = v.iter().sum::<f32>() / n;
            let sd = (v.iter().map(|x| (x - mean) * (x - mean)).sum::<f32>() / n).sqrt();
            vec![mean, sd]
        } else {
            let mut count = vec![0.0; l.alleles.len()];
            for a in values {
                if let Some(c) = count.get_mut(a as usize) {
                    *c += 1.0;
                }
            }
            let n = count.iter().sum::<f32>().max(1.0);
            count.iter().map(|c| c / n).collect()
        };
    }
    Some(pool)
}

/// `pool` (of `n` animals) with genome `g`'s alleles taken out (`sign` -1: one of them died or
/// was tamed) or added (+1: one joined): each allele's share, each trait's mean, as they then are.
pub fn count_in(index: usize, pool: &mut Pool, g: &Genome, sign: f32, n: f32) {
    let Some(file) = SPECIES.get(index).and_then(|s| gene_file(s.name)) else { return };
    let total = 2.0 * n;
    for l in &file.loci {
        let Some(p) = pool.get_mut(l.slot) else { continue };
        let have = g.pair(l.slot);
        if l.quantitative {
            if let Some(mean) = p.first_mut() {
                let sum = value(have[0]) + value(have[1]);
                *mean = ((*mean * total + sign * sum) / (total + 2.0 * sign)).clamp(0.0, 1.0);
            }
            continue;
        }
        let mut counts: Vec<f32> = p.iter().map(|s| s * total).collect();
        for a in have.into_iter().filter(|&a| a != NONE) {
            if let Some(c) = counts.get_mut(a as usize) {
                *c = (*c + sign).max(0.0);
            }
        }
        let sum = counts.iter().sum::<f32>();
        if sum > 0.0 {
            *p = counts.iter().map(|c| c / sum).collect();
        }
    }
}

/// A pool in words, a locus a part: `Pg sa 0.74 br 0.14 · Dk 0.31±0.12`.
pub fn describe_pool(index: usize, pool: &Pool) -> String {
    let Some(g) = SPECIES.get(index).and_then(|s| gene_file(s.name)) else { return String::new() };
    g.loci
        .iter()
        .filter_map(|l| Some((l, pool.get(l.slot).filter(|p| !p.is_empty())?)))
        .map(|(l, p)| {
            if l.quantitative {
                format!("{} {:.2}±{:.2}", l.id, p.first().unwrap_or(&0.0), p.get(1).unwrap_or(&0.0))
            } else {
                let mut a: Vec<(&str, f32)> = l
                    .alleles
                    .iter()
                    .map(|a| a.a.as_str())
                    .zip(p.iter().copied())
                    .collect();
                a.sort_by(|x, y| y.1.total_cmp(&x.1));
                let shown: Vec<String> = a
                    .iter()
                    .filter(|x| x.1 >= 0.01)
                    .map(|(n, f)| format!("{n} {f:.2}"))
                    .collect();
                format!("{} {}", l.id, shown.join(" "))
            }
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// A baby of `mother` and `father`: one allele of each locus from each (a son's only X from his
/// mother), quantitative traits drifting a little, now and then (1%) an allele mutating.
pub fn breed(mother: &Genome, father: &Genome, r: &mut impl Rng) -> Option<Genome> {
    breed_with(mother, father, 0.01, r)
}

/// `breed` with a chance of `mutation` per locus that an allele mutates.
pub fn breed_with(mother: &Genome, father: &Genome, mutation: f32, r: &mut impl Rng) -> Option<Genome> {
    let index = (mother.species as usize).checked_sub(1)?;
    let g = gene_file(SPECIES.get(index)?.name)?;
    let female = r.next() < 0.5;
    let one = |pair: [u8; 2], r: &mut dyn FnMut() -> f32| {
        if pair[1] == NONE || r() < 0.5 { pair[0] } else { pair[1] }
    };
    let breed = if mother.breed == father.breed { mother.breed } else { MIXED };
    let mut rnd = || r.next();
    let mut out = Genome::empty(mother.species, breed, female, (rnd() * 1e9) as u32);
    for l in &g.loci {
        let (m, f) = (mother.pair(l.slot), father.pair(l.slot));
        if l.quantitative {
            for (k, p) in [m, f].into_iter().enumerate() {
                let v = value(one(p, &mut rnd));
                out.set(l.slot, k, unit(v + 0.05 * gauss(&mut rnd)));
            }
            continue;
        }
        out.set(l.slot, 0, one(m, &mut rnd));
        if !l.xlinked || female {
            out.set(l.slot, 1, one(f, &mut rnd));
        }
        if rnd() < mutation {
            let k = if out.get(l.slot, 1) != NONE && rnd() < 0.5 { 1 } else { 0 };
            out.set(l.slot, k, wild_allele(l, &mut rnd));
        }
    }
    Some(out)
}

/// A genome saved before the core traits (or a locus its species gained since) given those from
/// `pool` (the wild where the animal is), and brought to the layout now.
pub fn upgrade(index: usize, g: &mut Genome, pool: &Pool, r: &mut impl Rng) {
    if g.version >= VERSION {
        return;
    }
    let Some(file) = SPECIES.get(index).and_then(|s| gene_file(s.name)) else { return };
    for l in &file.loci {
        let p = g.pair(l.slot);
        if l.slot >= OWN || p == [NONE, NONE] {
            draw(l, g.female != 0, pool.get(l.slot).filter(|p| !p.is_empty()), g, r);
        }
    }
    g.version = VERSION;
}

// ------------------------------------------------------------------ phenotypes

pub type Rgb = [f32; 3];

#[derive(Clone, Copy)]
struct Pattern {
    kind: &'static str,
    color: Rgb,
    amount: f32,
}

/// What a genome looks like: colors, patterns and white, and its name.
#[derive(Clone, Default)]
pub struct Look {
    pub name: String,
    base: Rgb,
    under: Rgb,
    pattern: Option<Pattern>,
    pattern2: Option<Pattern>,
    points: Option<Rgb>,
    points_pattern: Option<Pattern>,
    mask: Option<Rgb>,
    /// Darker or colored extremities (a colorpoint's, a capybara's darker legs): color, amount.
    ext: Option<(Rgb, f32)>,
    merle: Option<Rgb>,
    /// A tortoiseshell's other coat, in patches.
    patches: Option<Box<Look>>,
    /// How far white spotting reaches (0: none; over 1: all white).
    white: f32,
    /// How far the points' color reaches past where it starts (-1..1, 0 as the layers have it):
    /// a wyvern's accent spreading over more of its wings.
    points_reach: f32,
    /// How its marks are drawn (the core traits).
    marks: Marks,
    /// Its eye color: the core trait `Ey` along its species' eyes, unless its genes decide it
    /// (a colorpoint's blue).
    eye: Option<Rgb>,
    /// Model units to a pattern's (`GeneFile::mark_scale`).
    scale: f32,
    /// Its scars: kind and place (`Genome::scars`).
    scars: [[u8; 2]; 4],
    /// Old: a greying muzzle, the coat a little faded.
    elder: bool,
    seed: u32,
}

/// How an individual's marks are drawn, each 0..1 (0.5 as the pattern is designed): their size,
/// how much they cover, how stark they are, how ragged their edges and how drawn out along the
/// body (the core traits `Ms`, `Md`, `Ct`, `Me`, `Mx`).
#[derive(Clone, Copy)]
struct Marks {
    size: f32,
    density: f32,
    contrast: f32,
    edge: f32,
    stretch: f32,
}

impl Default for Marks {
    fn default() -> Marks {
        Marks { size: 0.5, density: 0.5, contrast: 0.5, edge: 0.5, stretch: 0.5 }
    }
}

impl Look {
    /// The color it shows at a glance, all over: what camouflages it or gives it away
    /// (`evolution.rs`).
    pub fn average(&self) -> [f32; 3] {
        let mut c = mix(self.base, self.under, 0.3);
        if let Some(p) = &self.patches {
            c = mix(c, mix(p.base, p.under, 0.3), 0.4);
        }
        for p in [self.pattern, self.pattern2, self.points_pattern]
            .into_iter()
            .flatten()
        {
            c = mix(c, p.color, 0.25 * p.amount);
        }
        if let Some(p) = self.points {
            c = mix(c, p, 0.12);
        }
        if let Some(m) = self.merle {
            c = mix(c, m, 0.3);
        }
        if let Some((e, a)) = self.ext {
            c = mix(c, e, 0.12 * a);
        }
        mix(c, WHITE, (self.white * 0.7).clamp(0.0, 1.0))
    }
}

fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}
fn lighten(c: Rgb, t: f32) -> Rgb {
    mix(c, [246.0, 240.0, 230.0], t)
}
fn darken(c: Rgb, t: f32) -> Rgb {
    mix(c, [24.0, 18.0, 18.0], t)
}
const WHITE: Rgb = [242.0, 238.0, 230.0];
/// Eyes without pigment: blue from the light the iris scatters.
const BLUE_EYE: Rgb = [92.0, 150.0, 214.0];

/// A genome read against its gene file.
struct Read<'a> {
    g: &'static GeneFile,
    genome: &'a Genome,
}

impl Read<'_> {
    fn locus(&self, id: &str) -> Option<&Locus> {
        self.g.loci.iter().find(|l| l.id == id)
    }
    fn pair(&self, l: &Locus) -> impl Iterator<Item = u8> + '_ {
        self.genome.pair(l.slot).into_iter().filter(|&a| a != NONE)
    }
    /// What shows at a locus: the most dominant allele present.
    fn shows(&self, id: &str) -> &str {
        let Some(l) = self.locus(id) else { return "" };
        let have: Vec<u8> = self.pair(l).collect();
        (0..l.alleles.len())
            .find(|a| have.contains(&(*a as u8)))
            .or(have.first().map(|&a| a as usize))
            .and_then(|a| l.alleles.get(a))
            .map_or("", |a| a.a.as_str())
    }
    /// How many copies of `allele` it has.
    fn copies(&self, id: &str, allele: &str) -> usize {
        let Some(l) = self.locus(id) else { return 0 };
        self.pair(l)
            .filter(|&a| l.alleles.get(a as usize).is_some_and(|x| x.a == allele))
            .count()
    }
    /// A quantitative trait's genes: the mean of its two values, 0..1 (what it passes on).
    fn genetic(&self, id: &str) -> f32 {
        self.locus(id)
            .map_or(0.5, |l| (value(self.genome.get(l.slot, 0)) + value(self.genome.get(l.slot, 1))) / 2.0)
    }
    /// A quantitative trait as it shows, 0..1: its genes, the animal's own chance for the share
    /// genes don't decide (from its seed: development, food; not passed on), and what its sex
    /// and youth shift.
    fn qv(&self, id: &str) -> f32 {
        let Some(l) = self.locus(id) else { return 0.5 };
        let mut v = self.genetic(id);
        let h = l.h.unwrap_or(1.0).clamp(0.05, 1.0);
        if h < 1.0 {
            let (u1, u2) = (hash3(l.slot as i32, 1, 7, self.genome.seed as i32), hash3(l.slot as i32, 2, 7, self.genome.seed as i32));
            let z = (-2.0 * (u1 + 1e-6).ln()).sqrt() * (std::f32::consts::TAU * u2).cos();
            v += l.sd * ((1.0 - h) / h).sqrt() * z;
        }
        let sex = if self.female() { "female" } else { "male" };
        v += self
            .g
            .sexes
            .get(sex)
            .and_then(|m| m.get(id))
            .copied()
            .unwrap_or(0.0);
        v += self.youth() * self.g.young.traits.get(id).copied().unwrap_or(0.0);
        v.clamp(0.0, 1.0)
    }
    /// How young it is: 1 a baby, 0 grown.
    fn youth(&self) -> f32 {
        let adult = crate::species::ADULT;
        (adult - self.genome.stage.min(adult)) as f32 / adult as f32
    }
    /// Whether the species has `id` as its own locus (not only as a core trait).
    fn own(&self, id: &str) -> bool {
        self.locus(id).is_some_and(|l| l.slot < OWN)
    }
    fn female(&self) -> bool {
        self.genome.female != 0
    }
}

fn dog(r: &Read) -> Look {
    let (brown, dil) = (r.shows("B") == "b", r.shows("D") == "d");
    let eu: Rgb = match (brown, dil) {
        (true, true) => [150.0, 128.0, 124.0],
        (true, false) => [94.0, 58.0, 40.0],
        (false, true) => [92.0, 98.0, 108.0],
        (false, false) => [38.0, 32.0, 32.0],
    };
    let eu_name = match (brown, dil) {
        (true, true) => "Lilac",
        (true, false) => "Chocolate",
        (false, true) => "Blue",
        (false, false) => "Black",
    };
    let i = r.qv("I");
    let mut pheo = if i < 0.5 {
        mix([236.0, 216.0, 180.0], [206.0, 152.0, 92.0], i * 2.0)
    } else {
        mix([206.0, 152.0, 92.0], [168.0, 84.0, 38.0], (i - 0.5) * 2.0)
    };
    if dil {
        pheo = mix(pheo, [198.0, 182.0, 166.0], 0.35);
    }
    let pheo_name = if i < 0.3 {
        "Cream"
    } else if i < 0.7 {
        "Fawn"
    } else {
        "Red"
    };
    let (e, k, a) = (r.shows("E"), r.shows("K"), r.shows("A"));
    let mut c = Look { base: eu, under: lighten(eu, 0.06), ..Look::default() };
    let mut eumelanic = false;
    let mut name;
    if e == "e" {
        (c.base, c.under) = (pheo, lighten(pheo, 0.3));
        name = pheo_name.to_string();
    } else if k == "KB" {
        name = eu_name.to_string();
        eumelanic = true;
    } else if a == "ay" {
        (c.base, c.under) = (pheo, lighten(pheo, 0.3));
        c.pattern = Some(Pattern { kind: "sable", color: eu, amount: 0.35 + 0.65 * r.qv("Sh") });
        name = format!("{pheo_name} sable");
    } else if a == "at" {
        c.points = Some(pheo);
        eumelanic = true;
        name = format!("{eu_name} and tan");
    } else {
        name = eu_name.to_string();
        eumelanic = true;
    }
    if e != "e" && k == "kbr" && a != "a" {
        if a == "ay" {
            c.pattern = Some(Pattern { kind: "brindle", color: eu, amount: 0.8 });
        } else {
            c.points_pattern = Some(Pattern { kind: "brindle", color: eu, amount: 0.7 });
        }
        name += " brindle";
    }
    if e == "Em" {
        c.mask = Some(eu);
        name += ", masked";
    }
    if r.shows("M") == "M" && eumelanic {
        c.merle = Some(mix(eu, [184.0, 186.0, 194.0], 0.6));
        // Merle dilutes the eyes' pigment too: some merles have blue eyes.
        if r.qv("Ey") > 0.6 {
            c.eye = Some(BLUE_EYE);
        }
        name = name.replacen(eu_name, &format!("{eu_name} merle"), 1);
    }
    let sp = r.copies("S", "sp").min(2);
    c.white = [0.08, 0.34, 0.66][sp];
    match sp {
        2 => name += " parti",
        1 => name += " with white",
        _ => {}
    }
    c.name = name;
    c
}

fn cat(r: &Read) -> Look {
    let dil = r.shows("D") == "d";
    let (eu, eu_name): (Rgb, &str) = match (r.shows("B"), dil) {
        ("b", false) => ([92.0, 58.0, 42.0], "Chocolate"),
        ("b", true) => ([170.0, 152.0, 148.0], "Lilac"),
        ("bl", false) => ([130.0, 80.0, 50.0], "Cinnamon"),
        ("bl", true) => ([190.0, 164.0, 144.0], "Fawn"),
        (_, false) => ([44.0, 38.0, 38.0], "Black"),
        (_, true) => ([112.0, 118.0, 128.0], "Blue"),
    };
    let (pheo, pheo_name): (Rgb, &str) = if dil { ([234.0, 204.0, 156.0], "Cream") } else { ([214.0, 124.0, 56.0], "Red") };
    let ground = mix(lighten(eu, 0.5), [200.0, 150.0, 92.0], 0.35 + 0.4 * r.qv("Ru"));
    let o = r.copies("O", "O");
    let red = if r.female() { o == 2 } else { o == 1 };
    let tortie = r.female() && o == 1;
    let tabby = r.shows("A") == "A" || red;
    let kind = if r.shows("Ta") == "Ta" {
        "ticked"
    } else if r.shows("Sp") == "Sp" {
        "spotted"
    } else if r.shows("Mc") == "Mc" {
        "mackerel"
    } else {
        "classic"
    };
    let tabby_name = format!("{kind} tabby");
    let red_coat = Look {
        base: pheo,
        under: lighten(pheo, 0.3),
        pattern: Some(Pattern { kind, color: darken(pheo, 0.38), amount: 1.0 }),
        ..Look::default()
    };
    let plain = if tabby {
        Look { base: ground, under: lighten(ground, 0.3), pattern: Some(Pattern { kind, color: eu, amount: 1.0 }), ..Look::default() }
    } else {
        Look { base: eu, under: lighten(eu, 0.05), ..Look::default() }
    };
    let (mut c, mut name) = if red {
        (red_coat, format!("{pheo_name} {tabby_name}"))
    } else if tortie {
        let n = format!("{eu_name} {}", if tabby { "torbie" } else { "tortoiseshell" });
        (Look { patches: Some(Box::new(red_coat)), ..plain }, n)
    } else {
        let n = if tabby { format!("{} {tabby_name}", if eu_name == "Black" { "Brown" } else { eu_name }) } else { eu_name.to_string() };
        (plain, n)
    };
    if r.copies("C", "cs") == 2 {
        // Colorpoint: a pale body, the color on the points.
        let point = if red { pheo } else { eu };
        c = Look {
            base: mix(c.base, [238.0, 226.0, 206.0], 0.78),
            under: [242.0, 234.0, 220.0],
            ext: Some((point, 1.0)),
            pattern: c.pattern.map(|p| Pattern { amount: 0.2, ..p }),
            ..Look::default()
        };
        let seal = if eu_name == "Black" { "Seal" } else { eu_name };
        name = format!("{seal} point{}", if red { " (flame)" } else { "" });
        // The same gene keeps the eyes' pigment from forming: always blue.
        c.eye = Some(BLUE_EYE);
    }
    let s = r.copies("S", "S").min(2);
    c.white = [0.03, 0.36, 0.7][s];
    match s {
        2 => name += ", van",
        1 => name += " with white",
        _ => {}
    }
    if r.shows("W") == "W" {
        c.white = 1.2;
        name = "White".into();
        // White cats' eyes are often blue (no pigment cells reached them).
        if r.qv("Ey") > 0.45 {
            c.eye = Some(BLUE_EYE);
        }
    }
    c.name = name;
    c
}

fn capybara(r: &Read) -> Look {
    let (dk, ru, bl) = (r.qv("Dk"), r.qv("Ru"), r.qv("Bl"));
    if r.copies("Le", "le") == 2 {
        return Look { name: "Leucistic".into(), base: [226.0, 208.0, 176.0], under: [238.0, 226.0, 204.0], ..Look::default() };
    }
    let mut base = mix([190.0, 142.0, 96.0], [84.0, 56.0, 40.0], dk);
    base = mix(base, if ru > 0.5 { [150.0, 80.0, 42.0] } else { [118.0, 108.0, 98.0] }, (ru - 0.5).abs() * 0.7);
    let shade = if dk < 0.33 {
        "Light tan"
    } else if dk < 0.66 {
        "Brown"
    } else {
        "Dark brown"
    };
    let tint = if ru > 0.62 {
        ", reddish"
    } else if ru < 0.38 {
        ", greyish"
    } else {
        ""
    };
    Look {
        name: format!("{shade}{tint}"),
        base,
        under: lighten(base, 0.12 + 0.4 * bl),
        ext: Some((darken(base, 0.4), 0.7)),
        ..Look::default()
    }
}

/// Hares (the brown hare and the mountain hare share these loci): agouti fur grizzled with dark tips
/// over a white belly, grey-brown to rufous; the mountain hare's white coat (dominant, as the
/// snowshoe hare's winter-white allele), the brown hare's tail black on top; black very rarely.
fn hare(r: &Read) -> Look {
    let ru = r.qv("Ru");
    let tail = (r.shows("Bt") == "Bt").then_some(([40.0, 34.0, 32.0], 1.0));
    if r.copies("Mel", "mel") == 2 {
        return Look { name: "Black".into(), base: [58.0, 50.0, 46.0], under: [84.0, 74.0, 66.0], ext: tail, ..Look::default() };
    }
    if r.shows("Wi") == "Wi" {
        return Look {
            name: "White (winter coat)".into(),
            base: [234.0, 232.0, 226.0],
            under: [242.0, 240.0, 234.0],
            ext: tail,
            ..Look::default()
        };
    }
    let base = mix([150.0, 128.0, 98.0], [178.0, 116.0, 66.0], ru);
    let name = if ru > 0.62 {
        "Rufous brown"
    } else if ru < 0.38 {
        "Grey-brown"
    } else {
        "Brown"
    };
    Look {
        name: name.into(),
        base,
        under: [236.0, 230.0, 216.0],
        pattern: Some(Pattern { kind: "ticked", color: darken(base, 0.4), amount: 0.6 }),
        ext: tail,
        ..Look::default()
    }
}

/// Wood mice: warm sandy brown to grey-brown above, sharply grey-white below; albinos very rarely
/// (recessive).
fn mouse(r: &Read) -> Look {
    if r.copies("Al", "al") == 2 {
        return Look { name: "Albino".into(), base: [236.0, 230.0, 222.0], under: [242.0, 238.0, 232.0], ..Look::default() };
    }
    let ru = r.qv("Ru");
    let base = mix([128.0, 112.0, 96.0], [166.0, 112.0, 66.0], ru);
    let name = if ru > 0.62 {
        "Sandy brown"
    } else if ru < 0.38 {
        "Grey-brown"
    } else {
        "Brown"
    };
    Look { name: name.into(), base, under: [220.0, 216.0, 206.0], ..Look::default() }
}

/// Dromaeosaurs: feather colors of ground birds (every raptor species has these loci).
fn raptor(r: &Read) -> Look {
    let (pigment, mut name): (Rgb, String) = match r.shows("Pg") {
        "ru" => ([152.0, 86.0, 50.0], "Rufous".into()),
        "gr" => ([112.0, 110.0, 108.0], "Ash".into()),
        "sa" => ([194.0, 160.0, 110.0], "Sandy".into()),
        _ => ([104.0, 80.0, 60.0], "Umber".into()),
    };
    let dk = r.qv("Dk");
    let mut base = if dk > 0.5 { darken(pigment, (dk - 0.5) * 0.7) } else { lighten(pigment, (0.5 - dk) * 0.5) };
    let mut under = mix(lighten(base, 0.5), [226.0, 210.0, 178.0], 0.4);
    let kind = match r.shows("Pt") {
        "bars" => Some(("bars", ", barred")),
        "bands" => Some(("bands", ", banded")),
        "spots" => Some(("rosettes", ", spotted")),
        "speckle" => Some(("speckle", ", speckled")),
        _ => None,
    };
    let mut pattern = kind.map(|(k, n)| {
        name += n;
        Pattern { kind: k, color: darken(base, 0.5), amount: 1.0 }
    });
    let mut saddle = None;
    if r.copies("Sd", "sd") == 2 {
        saddle = Some(Pattern { kind: "saddle", color: darken(base, 0.55), amount: 0.9 });
        name += ", saddled";
    }
    if r.copies("Mel", "mel") == 2 {
        (base, under) = ([44.0, 38.0, 36.0], [74.0, 66.0, 60.0]);
        pattern = pattern.map(|p| Pattern { color: [24.0, 20.0, 20.0], amount: 0.4, ..p });
        name = "Melanistic".into();
    }
    if r.copies("Lc", "lc") == 2 {
        (base, under) = ([228.0, 218.0, 200.0], [244.0, 238.0, 226.0]);
        pattern = pattern.map(|p| Pattern { color: [196.0, 184.0, 166.0], amount: 0.6, ..p });
        name = "Leucistic".into();
    }
    Look { name, base, under, pattern, pattern2: saddle, ..Look::default() }
}

/// Wyverns: scales of a big reptile's colors, and what it breathes (named with the coat).
fn wyvern(r: &Read) -> Look {
    // Scales and an accent (the wing skin, the spikes' tips), as the show's dragons wear them:
    // black with red, green with bronze, cream with gold (`source/wyvern`'s `COATS`).
    let (scales, accent, name): (Rgb, Rgb, &str) = match r.shows("Sc") {
        "bronze" => ([132.0, 92.0, 52.0], [96.0, 54.0, 38.0], "Bronze"),
        "green" => ([58.0, 78.0, 52.0], [150.0, 104.0, 52.0], "Green"),
        "black" => ([40.0, 38.0, 42.0], [128.0, 36.0, 30.0], "Black"),
        "red" => ([118.0, 40.0, 34.0], [52.0, 26.0, 24.0], "Red"),
        "white" => ([222.0, 212.0, 190.0], [196.0, 150.0, 70.0], "Cream"),
        _ => ([92.0, 92.0, 98.0], [112.0, 70.0, 60.0], "Slate"),
    };
    let dk = r.qv("Dk");
    let base = if dk > 0.5 { darken(scales, (dk - 0.5) * 0.6) } else { lighten(scales, (0.5 - dk) * 0.4) };
    // The belly plates a muted, lighter brown of the scales (a black one's dark, as Drogon's).
    let under = mix(lighten(base, 0.3), [170.0, 140.0, 110.0], 0.4);
    let breath = match breath(r.genome) {
        "" => "fire",
        b => b,
    };
    // Dark markings over the scales (on cream scales, in the accent).
    let ink = if r.shows("Sc") == "white" { mix(accent, base, 0.3) } else { darken(base, 0.5) };
    let (kind, marked) = match r.shows("Mk") {
        "bands" => (Some("bands"), ", banded"),
        "blotches" => (Some("blotches"), ", blotched"),
        "mottle" => (Some("mottle"), ", mottled"),
        "stripes" => (Some("streaks"), ", striped"),
        _ => (None, ""),
    };
    Look {
        name: format!("{name}{marked}, breathes {breath}"),
        base,
        under,
        points: Some(accent),
        points_reach: (r.qv("Ac") - 0.5) * 2.0,
        pattern: kind.map(|kind| Pattern { kind, color: ink, amount: 0.85 }),
        ..Look::default()
    }
}

/// What a wyvern with `genome` breathes ("fire", "frost", "lightning", "venom"; "" for other
/// species).
pub fn breath(genome: &Genome) -> &'static str {
    let Some(s) = (genome.species as usize)
        .checked_sub(1)
        .and_then(|i| SPECIES.get(i))
    else {
        return "";
    };
    let Some(g) = gene_file(s.name) else { return "" };
    match (Read { g, genome }).shows("Br") {
        "frost" => "frost",
        "lightning" => "lightning",
        "venom" => "venom",
        "fire" => "fire",
        _ => "",
    }
}

/// Gives a wyvern two copies of the breath allele `kind` (testing: `/animal fire`).
pub fn set_breath(genome: &mut Genome, kind: &str) -> bool {
    let Some(s) = (genome.species as usize)
        .checked_sub(1)
        .and_then(|i| SPECIES.get(i))
    else {
        return false;
    };
    let Some(g) = gene_file(s.name) else { return false };
    let Some(l) = g.loci.iter().find(|l| l.id == "Br") else { return false };
    let Some(a) = l.alleles.iter().position(|a| a.a == kind) else { return false };
    genome.set(l.slot, 0, a as u8);
    genome.set(l.slot, 1, a as u8);
    true
}

/// How big an adult with `genome` is against its species' average (its model): its size gene
/// (`Sz`, spread by the species' `size_spread`) and its sex (`dimorphism`, males against
/// females; the average stays the model's).
pub fn size(genome: &Genome) -> f32 {
    let Some(s) = (genome.species as usize)
        .checked_sub(1)
        .and_then(|i| SPECIES.get(i))
    else {
        return 1.0;
    };
    // As it shows: its genes and its own chance (how well it grew).
    let sz = gene_file(s.name).map_or(0.5, |g| Read { g, genome }.qv("Sz"));
    let sex = if genome.female != 0 { s.dimorphism.sqrt().recip() } else { s.dimorphism.sqrt() };
    (1.0 + s.size_spread * (sz - 0.5) * 2.0) * sex
}

/// Its size genes (`Sz`), 0..1: small to big for its kind (what it passes on).
pub fn size_gene(genome: &Genome) -> f32 {
    let Some(s) = (genome.species as usize)
        .checked_sub(1)
        .and_then(|i| SPECIES.get(i))
    else {
        return 0.5;
    };
    gene_file(s.name).map_or(0.5, |g| Read { g, genome }.genetic("Sz"))
}

/// What `genome` looks like (none before it has genes).
pub fn phenotype(genome: &Genome) -> Option<Look> {
    let s = SPECIES.get((genome.species as usize).checked_sub(1)?)?;
    let r = Read { g: gene_file(s.name)?, genome };
    let mut look = match s.name {
        "dog" => dog(&r),
        "cat" => cat(&r),
        "capybara" => capybara(&r),
        "wyvern" => wyvern(&r),
        "hare" | "mountain_hare" => hare(&r),
        "wood_mouse" => mouse(&r),
        _ => raptor(&r),
    };
    core(&r, &mut look);
    look.scars = std::array::from_fn(|i| [genome.scars[2 * i], genome.scars[2 * i + 1]]);
    if look.scars.iter().any(|s| s[0] != 0) {
        look.name += ", scarred";
    }
    look.seed = genome.seed;
    if let Some(p) = &mut look.patches {
        p.seed = genome.seed;
    }
    Some(look)
}

/// The core traits on a species' look, the same for every species: how light and warm this
/// one's colors are, how far its white reaches, how its marks are drawn, its eyes, and the
/// pattern a young one is born with.
fn core(r: &Read, look: &mut Look) {
    let g = r.g;
    let recolor = |f: &dyn Fn(Rgb) -> Rgb, look: &mut Look| {
        look.base = f(look.base);
        look.under = f(look.under);
        look.points = look.points.map(f);
        if let Some(p) = look.patches.as_deref_mut() {
            (p.base, p.under) = (f(p.base), f(p.under));
        }
    };
    if !r.own("Dk") {
        // A species without shade genes of its own still has lighter and darker ones.
        let dk = r.qv("Dk");
        recolor(&|c| if dk > 0.5 { darken(c, (dk - 0.5) * 0.5) } else { lighten(c, (0.5 - dk) * 0.35) }, look);
    }
    let t = (r.qv("Wm") - 0.5) * 2.0;
    let warm = |c: Rgb| [c[0] * (1.0 + 0.07 * t), c[1] * (1.0 + 0.015 * t), c[2] * (1.0 - 0.09 * t)];
    recolor(&warm, look);
    for p in [&mut look.pattern, &mut look.pattern2, &mut look.points_pattern]
        .into_iter()
        .flatten()
    {
        p.color = warm(p.color);
    }
    if look.white > 0.0 && look.white < 1.0 {
        look.white = (look.white * (0.6 + 0.8 * r.qv("Wh"))).min(0.97);
    }
    look.scale = g.mark_scale;
    look.marks = Marks { size: r.qv("Ms"), density: r.qv("Md"), contrast: r.qv("Ct"), edge: r.qv("Me"), stretch: r.qv("Mx") };
    if look.eye.is_none() && !g.eyes.is_empty() {
        let e = r.qv("Ey") * (g.eyes.len() - 1) as f32;
        let i = (e.floor() as usize).min(g.eyes.len() - 1);
        look.eye = Some(mix(g.eyes[i], g.eyes[(i + 1).min(g.eyes.len() - 1)], e - i as f32));
    }
    look.elder = r.genome.stage >= crate::species::ELDER;
    let youth = r.youth();
    if let Some(kind) = g.young.pattern.as_deref()
        && youth > 0.0
        && look.pattern2.is_none()
    {
        look.pattern2 = Some(Pattern { kind, color: lighten(look.base, 0.5), amount: youth });
    }
}

// ------------------------------------------------------------------ described (for the website)

/// One locus of an animal, as the website shows it: its alleles and the one that shows, or a
/// trait's genes and how it shows.
#[derive(Serialize)]
pub struct LocusView {
    pub id: String,
    pub name: String,
    pub about: String,
    /// One of the core traits every species carries.
    pub core: bool,
    pub quantitative: bool,
    pub alleles: Vec<String>,
    pub shows: String,
    pub meaning: String,
    pub genetic: f32,
    pub shown: f32,
    pub heritability: f32,
}

/// What an animal looks like, as the website shows it.
#[derive(Serialize)]
pub struct LookView {
    pub name: String,
    pub base: Rgb,
    pub under: Rgb,
    pub points: Option<Rgb>,
    pub pattern: Option<(String, Rgb)>,
    pub pattern2: Option<(String, Rgb)>,
    pub merle: Option<Rgb>,
    pub patches: Option<Rgb>,
    pub eye: Option<Rgb>,
    pub white: f32,
    /// Size, density, contrast, edge, stretch.
    pub marks: [f32; 5],
    pub scars: Vec<u8>,
    /// Its colors at a glance (what camouflages it).
    pub average: Rgb,
}

/// An animal's loci and look, for the website.
pub fn describe(genome: &Genome) -> Option<(Vec<LocusView>, LookView)> {
    let s = SPECIES.get((genome.species as usize).checked_sub(1)?)?;
    let r = Read { g: gene_file(s.name)?, genome };
    let loci =
        r.g.loci
            .iter()
            .filter(|l| genome.pair(l.slot) != [NONE, NONE])
            .map(|l| {
                let alleles = r
                    .pair(l)
                    .map(|a| {
                        l.alleles
                            .get(a as usize)
                            .map_or("?".into(), |x| x.a.clone())
                    })
                    .collect();
                let shows = if l.quantitative { String::new() } else { r.shows(&l.id).to_string() };
                let meaning = l
                    .alleles
                    .iter()
                    .find(|a| a.a == shows)
                    .map_or(String::new(), |a| a.about.clone());
                LocusView {
                    id: l.id.clone(),
                    name: l.name.clone(),
                    about: l.about.clone(),
                    core: l.slot >= OWN,
                    quantitative: l.quantitative,
                    alleles: if l.quantitative { Vec::new() } else { alleles },
                    shows,
                    meaning,
                    genetic: if l.quantitative { r.genetic(&l.id) } else { 0.0 },
                    shown: if l.quantitative { r.qv(&l.id) } else { 0.0 },
                    heritability: l.h.unwrap_or(1.0),
                }
            })
            .collect();
    let look = phenotype(genome)?;
    let m = look.marks;
    let view = LookView {
        name: look.name.clone(),
        base: look.base,
        under: look.under,
        points: look.points,
        pattern: look.pattern.map(|p| (p.kind.to_string(), p.color)),
        pattern2: look.pattern2.map(|p| (p.kind.to_string(), p.color)),
        merle: look.merle,
        patches: look.patches.as_deref().map(|p| p.base),
        eye: look.eye,
        white: look.white,
        marks: [m.size, m.density, m.contrast, m.edge, m.stretch],
        scars: look
            .scars
            .iter()
            .map(|s| s[0])
            .filter(|&k| k != 0)
            .collect(),
        average: look.average(),
    };
    Some((loci, view))
}

/// Sets locus `id` of `genome` to `values` (allele names, or a trait's 0..1 values): one copy each
/// (a male's X-linked locus takes one). For trying edge cases; `None` if the locus or a value is
/// unknown.
pub fn set_locus(genome: &mut Genome, id: &str, values: &[serde_json::Value]) -> Option<()> {
    let s = SPECIES.get((genome.species as usize).checked_sub(1)?)?;
    let l = gene_file(s.name)?.loci.iter().find(|l| l.id == id)?;
    if values.is_empty() || values.len() > 2 {
        return None;
    }
    let mut set = [NONE, NONE];
    for (k, v) in values.iter().enumerate() {
        set[k] = if l.quantitative {
            unit(v.as_f64()? as f32)
        } else {
            l.alleles
                .iter()
                .position(|a| Some(a.a.as_str()) == v.as_str())? as u8
        };
    }
    if l.xlinked && genome.female == 0 {
        set[1] = NONE;
    }
    genome.set(l.slot, 0, set[0]);
    genome.set(l.slot, 1, if values.len() == 1 && !l.xlinked { set[0] } else { set[1] });
    Some(())
}

impl Genome {
    /// Its bytes, as saved and sent (`size_of::<Genome>()` of them).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = vec![0u8; size_of::<Genome>()];
        // A plain `repr(C)` struct of bytes and one u32: every byte is defined but the padding,
        // which `Default` zeroed for every genome made here.
        unsafe { std::ptr::copy_nonoverlapping(self as *const Genome as *const u8, out.as_mut_ptr(), out.len()) };
        out
    }
    /// A genome from its bytes (`to_bytes`); one too short is padded with zeros.
    pub fn from_bytes(b: &[u8]) -> Genome {
        let mut all = vec![0u8; size_of::<Genome>()];
        let n = b.len().min(all.len());
        all[..n].copy_from_slice(&b[..n]);
        unsafe { std::ptr::read_unaligned(all.as_ptr() as *const Genome) }
    }
}

// ------------------------------------------------------------------ patterns in 3D (model units)

fn hash3(x: i32, y: i32, z: i32, s: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374761393)
        ^ (y as u32).wrapping_mul(668265263)
        ^ (z as u32).wrapping_mul(2147483647)
        ^ (s as u32).wrapping_mul(1274126177);
    h = (h ^ (h >> 13)).wrapping_mul(1103515245);
    ((h ^ (h >> 16)) as f64 / 4294967296.0) as f32
}

fn noise3(x: f32, y: f32, z: f32, s: i32) -> f32 {
    let (xi, yi, zi) = (x.floor(), y.floor(), z.floor());
    let f = |t: f32| t * t * (3.0 - 2.0 * t);
    let (u, v, w) = (f(x - xi), f(y - yi), f(z - zi));
    let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let (xi, yi, zi) = (xi as i32, yi as i32, zi as i32);
    let c = |dx, dy, dz| hash3(xi + dx, yi + dy, zi + dz, s);
    l(l(l(c(0, 0, 0), c(1, 0, 0), u), l(c(0, 1, 0), c(1, 1, 0), u), v), l(l(c(0, 0, 1), c(1, 0, 1), u), l(c(0, 1, 1), c(1, 1, 1), u), v), w)
}

fn smooth(v: f32, a: f32, b: f32) -> f32 {
    let t = ((v - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn band(t: f32, width: f32) -> f32 {
    let f = t - t.floor();
    smooth(f, 0.0, 0.08) * (1.0 - smooth(f, width - 0.08, width))
}

/// How much a pattern covers a texel (0..1): `p` in model units (x side to side, y up, z along
/// the body; divided by the species' `mark_scale`), `u` how far toward the underside, `m` how
/// this individual's marks are drawn: bigger marks are the pattern drawn on a smaller body,
/// stretched ones on a body shortened along its length, ragged ones on a warped body, and
/// denser ones with wider marks.
fn pattern(kind: &str, p: [f32; 3], u: f32, s: i32, m: &Marks) -> f32 {
    let k = 2f32.powf((m.size - 0.5) * 1.6);
    let st = 2f32.powf((m.stretch - 0.5) * 1.2);
    let q = [p[0] / k * st.sqrt(), p[1] / k * st.sqrt(), p[2] / (k * st)];
    let warp = 0.5 + 3.0 * m.edge;
    let n = |o: i32| (noise3(q[0] * 0.4, q[1] * 0.4, q[2] * 0.4, s + o) - 0.5) * warp;
    let [x, y, z] = [q[0] + n(11), q[1] + n(12), q[2] + n(13)];
    let d = m.density - 0.5;
    let w = |width: f32| (width * (1.0 + 1.1 * d)).clamp(0.1, 0.85);
    match kind {
        "mackerel" | "bars" => band((z + 1.6 * (y * 0.45).sin() + 0.5 * x.abs()) / 5.5, w(0.32)),
        "classic" => band(((z - 2.0).hypot(y - 14.0)) / 6.5 + 0.15 * noise3(x * 0.3, y * 0.3, z * 0.3, s), w(0.42)),
        "spotted" => {
            band((z + 1.6 * (y * 0.45).sin()) / 5.5, w(0.4))
                * smooth(noise3(x * 0.55, y * 0.55, z * 0.55, s), 0.45 - 0.3 * d, 0.55 - 0.3 * d)
        }
        "ticked" => 0.35 * smooth(noise3(x * 1.4, y * 1.4, z * 1.4, s), 0.55 - 0.3 * d, 0.7 - 0.3 * d),
        "bands" => band((z + 0.4 * y) / 7.0, w(0.38)),
        "streaks" => band((y + 1.5 * noise3(z * 0.08, x * 0.2, 0.0, s)) / 4.5, w(0.3)),
        "rosettes" => {
            let (cx, cy, cz) = ((x / 4.0).floor(), (y / 4.0).floor(), (z / 4.0).floor());
            if hash3(cx as i32, cy as i32, cz as i32, s) > 0.55 + 0.6 * d {
                return 0.0;
            }
            let (dx, dz, dy) = (x - (cx + 0.5) * 4.0, z - (cz + 0.5) * 4.0, (y - (cy + 0.5) * 4.0) * 0.6);
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            smooth(dist, 0.6, 0.8) * (1.0 - smooth(dist, 1.5, 1.7))
        }
        "blotches" => {
            // Solid spots, one at a jittered place in each cell (a reptile's blotches).
            let c = |v: f32| (v / 6.0).floor();
            let (cx, cy, cz) = (c(x), c(y), c(z));
            let mut best = 0.0f32;
            for (ox, oy, oz) in [(0, 0, 0), (1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1), (0, 1, 0), (0, -1, 0)] {
                let (i, j, l) = (cx as i32 + ox, cy as i32 + oy, cz as i32 + oz);
                if hash3(i, j, l, s + 21) > 0.5 + 0.6 * d {
                    continue;
                }
                let at = |o: i32, base: i32| (base as f32 + 0.2 + 0.6 * hash3(i, j, l, s + o)) * 6.0;
                let dist = ((x - at(22, i)).powi(2) + ((y - at(23, j)) * 0.8).powi(2) + (z - at(24, l)).powi(2)).sqrt();
                let r = 1.4 + 1.2 * hash3(i, j, l, s + 25);
                best = best.max(1.0 - smooth(dist, r - 0.3, r + 0.3));
            }
            best
        }
        "mottle" => {
            let v = noise3(x * 0.22, y * 0.22, z * 0.22, s + 31) * 0.65 + noise3(x * 0.6, y * 0.6, z * 0.6, s + 32) * 0.35;
            smooth(v, 0.55 - 0.25 * d, 0.62 - 0.25 * d)
        }
        "speckle" => (hash3(x.floor() as i32, y.floor() as i32, z.floor() as i32, s) < 0.14 * (1.0 + 2.0 * d).max(0.2)) as u8 as f32,
        "saddle" => smooth(0.55 + 0.3 * d - u, 0.0, 0.25),
        "sable" => smooth(0.75 + 0.3 * d - u, 0.0, 0.5) * (0.55 + 0.45 * noise3(x * 0.6, y * 0.6, z * 0.6, s)),
        "brindle" => band((z + 2.5 * noise3(x * 0.25, y * 0.25, z * 0.25, s) * 3.0) / 2.2, w(0.3)),
        "patches" => smooth(noise3(x * 0.15, y * 0.15, z * 0.15, s), 0.5 - 0.25 * d, 0.56 - 0.25 * d),
        _ => 0.0,
    }
}

// ------------------------------------------------------------------ crisp marks

/// How much of a mark a texel takes, as pixel art draws it: none, an edge tone, or all of it.
const TONES: [f32; 3] = [0.0, 0.5, 1.0];

/// A cover (0..1) as one of `TONES`' steps.
fn tone_step(v: f32) -> u8 {
    if v < 0.35 {
        0
    } else if v < 0.65 {
        1
    } else {
        2
    }
}

fn tones3(v: f32) -> f32 {
    TONES[tone_step(v) as usize]
}

/// Each coat texel's marks as `TONES` steps: its pattern, its second pattern, its points'
/// pattern, and (2) whether a tortoiseshell's other coat covers it. A pattern's cover is snapped
/// to its steps relative to its strongest (ticking is faint everywhere), then lone texels (unlike
/// every neighbour on its face) take their neighbours' step: pixel art's clean clusters, no stray
/// pixels.
fn crisp_marks(l: &Layers, look: &Look, w: usize, h: usize) -> Vec<[u8; 4]> {
    let [lo, hi] = l.bounds;
    let s = look.seed as i32;
    let m = &look.marks;
    let peak = |kind: &str| if kind == "ticked" { 0.35 } else { 1.0 };
    let step = |pt: Option<Pattern>, p: [f32; 3], u: f32| pt.map_or(0, |pt| tone_step(pattern(pt.kind, p, u, s, m) / peak(pt.kind)));
    let mut marks = vec![[0u8; 4]; l.w * l.h];
    for y in 0..h {
        for x in 0..w {
            let i = y * l.w + x;
            if l.info[i * 4] < 128 {
                continue;
            }
            let u = l.info[i * 4 + 2] as f32 / 255.0;
            let p = [0, 1, 2].map(|k| (lo[k] + l.pos[i * 4 + k] as f32 / 255.0 * (hi[k] - lo[k])) / look.scale.max(0.01));
            let other = look
                .patches
                .as_deref()
                .filter(|_| pattern("patches", p, u, s + 7, m) > 0.5);
            let pat = other.map_or(look.pattern, |o| o.pattern);
            marks[i] = [step(pat, p, u), step(look.pattern2, p, u), step(look.points_pattern, p, u), if other.is_some() { 2 } else { 0 }];
        }
    }
    let read = marks.clone();
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let i = y * l.w + x;
            if l.info[i * 4] < 128 {
                continue;
            }
            let near: Vec<usize> = [i - 1, i + 1, i - l.w, i + l.w]
                .into_iter()
                .filter(|&j| l.info[j * 4] >= 128)
                .collect();
            if near.len() < 3 {
                continue;
            }
            for k in 0..4 {
                let mine = read[i][k];
                if near.iter().any(|&j| read[j][k] == mine) {
                    continue;
                }
                let mut count = [0; 3];
                for &j in &near {
                    count[read[j][k] as usize] += 1;
                }
                marks[i][k] = (0..3).max_by_key(|&v| (count[v], 2 - v)).unwrap_or(0) as u8;
            }
        }
    }
    marks
}

// ------------------------------------------------------------------ scars

/// A scar placed on a model: its kind, its middle (on the body), the surface's normal there, the
/// way it runs and across it (along the surface), its length (a bite's or burn's radius) and its
/// width, in model units.
struct ScarShape {
    kind: u8,
    c: [f32; 3],
    n: [f32; 3],
    d: [f32; 3],
    across: [f32; 3],
    len: f32,
    w: f32,
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}
fn norm3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a).max(1e-6);
    a.map(|v| v / l)
}
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// Where `look`'s scars fall on a model's layers: each on a coat texel its place picks (a tear on
/// the wing skin, the accent; the others where patterns show), running a way its place picks,
/// sized to the body.
fn scar_shapes(l: &Layers, look: &Look) -> Vec<ScarShape> {
    let [lo, hi] = l.bounds;
    let size = len3(sub3(hi, lo));
    let seed = look.seed as i32;
    let texels = l.w * l.h;
    let mut out = Vec::new();
    for [kind, place] in look.scars {
        if kind == 0 {
            continue;
        }
        let fits = |i: usize| {
            l.info[i * 4] >= 128
                && if kind == TEAR { l.props[i * 4 + 1] > 150 } else { l.props2[i * 4 + 1] > 80 && l.props[i * 4 + 1] < 100 }
        };
        let count = (0..texels).filter(|&i| fits(i)).count();
        if count == 0 {
            continue;
        }
        let h = |k: i32| hash3(place as i32, k, 99, seed);
        let pick = ((h(1) * count as f32) as usize).min(count - 1);
        let Some(i) = (0..texels).filter(|&i| fits(i)).nth(pick) else { continue };
        let at = |i: usize| [0, 1, 2].map(|k| lo[k] + l.pos[i * 4 + k] as f32 / 255.0 * (hi[k] - lo[k]));
        let c = at(i);
        // The surface there: from the texels beside and below it on the same face.
        let near = |j: usize| (j < texels && l.info[j * 4] >= 128 && len3(sub3(at(j), c)) < size * 0.02).then(|| sub3(at(j), c));
        let (right, down) = (near(i + 1).or_else(|| near(i.wrapping_sub(1))), near(i + l.w).or_else(|| near(i.wrapping_sub(l.w))));
        let n = match (right, down) {
            (Some(a), Some(b)) if len3(cross3(a, b)) > 1e-6 => norm3(cross3(a, b)),
            _ => norm3([h(5) - 0.5, h(6) - 0.5, h(7) - 0.5]),
        };
        let d = norm3(cross3(n, norm3([h(2) - 0.5, (h(3) - 0.5) * 1.4, h(4) - 0.5])));
        let across = cross3(n, d);
        let (len, w) = match kind {
            CLAW => (0.11, 0.006),
            BITE => (0.055, 0.011),
            BURN => (0.07, 0.0),
            TEAR => (0.08, 0.008),
            _ => (0.15, 0.006),
        };
        out.push(ScarShape { kind, c, n, d, across, len: size * len * (0.8 + 0.4 * h(8)), w: (size * w).max(1.0) });
    }
    out
}

impl ScarShape {
    /// How much of a texel at `p` (model units) it covers, 0..1.
    fn cover(&self, p: [f32; 3], seed: i32) -> f32 {
        // On the surface it lies on, and a little round its edges.
        let depth = dot3(sub3(p, self.c), self.n);
        if depth.abs() > self.len * 0.5 {
            return 0.0;
        }
        let p = [0, 1, 2].map(|k| p[k] - self.n[k] * depth);
        let line = |off: f32, len: f32, w: f32| {
            let r = [0, 1, 2].map(|k| p[k] - self.c[k] - self.across[k] * off);
            let t = dot3(r, self.d).clamp(-len / 2.0, len / 2.0);
            let dist = len3([0, 1, 2].map(|k| r[k] - self.d[k] * t));
            // Narrowing toward its ends.
            let w = w * (1.0 - (2.0 * t / len).powi(2)).max(0.0).sqrt();
            1.0 - smooth(dist, w * 0.5, w.max(0.01))
        };
        match self.kind {
            CLAW => (-1..=1)
                .map(|k| line(k as f32 * self.w * 3.0, self.len * (1.0 - 0.15 * (k as f32).abs()), self.w))
                .fold(0.0, f32::max),
            BITE => {
                // Two arcs of tooth marks round where the jaws closed.
                let r = sub3(p, self.c);
                let (a, b) = (dot3(r, self.d), dot3(r, self.across));
                let ring = (a.hypot(b) - self.len).abs();
                let angle = b.atan2(a);
                let tooth = ((angle * 9.0 / std::f32::consts::PI).rem_euclid(2.0) < 1.0) as u8 as f32;
                tooth * (1.0 - smooth(ring, self.w * 0.6, self.w * 1.2))
            }
            BURN => {
                let dist = len3(sub3(p, self.c));
                let k = 3.0 / self.len.max(0.01);
                let edge = self.len * (0.7 + 0.6 * noise3(p[0] * k, p[1] * k, p[2] * k, seed + 41));
                1.0 - smooth(dist, edge * 0.85, edge)
            }
            TEAR => {
                let k = 2.0 / self.w.max(0.01);
                line((noise3(p[0] * k, p[1] * k, p[2] * k, seed + 43) - 0.5) * self.w * 2.0, self.len, self.w * 1.6)
            }
            _ => line(0.0, self.len, self.w * 1.3),
        }
    }
}

// ------------------------------------------------------------------ light (the skill's paint.tones)

fn rgb_hsv([r, g, b]: Rgb) -> [f32; 3] {
    let (r, g, b) = (r / 255.0, g / 255.0, b / 255.0);
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let d = mx - mn;
    let mut h = 0.0;
    if d > 0.0 {
        h = if mx == r {
            ((g - b) / d) % 6.0
        } else if mx == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
    }
    [((h / 6.0) + 1.0) % 1.0, if mx > 0.0 { d / mx } else { 0.0 }, mx]
}

fn hsv_rgb(h: f32, s: f32, v: f32) -> Rgb {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - f * s), v * (1.0 - (1.0 - f) * s));
    let [r, g, b] = match (i as i32).rem_euclid(6) {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    };
    [r * 255.0, g * 255.0, b * 255.0]
}

/// A shade `k` ramp steps from a color (negative darker), hue-shifted: shadows warm and a little
/// more saturated, lights toward yellow and a little less.
fn tone(rgb: Rgb, k: f32) -> Rgb {
    let [h, s, v] = rgb_hsv(rgb);
    let toward = |h0: f32, target: f32, amt: f32| {
        let d = ((target - h0 + 0.5) % 1.0 + 1.0) % 1.0 - 0.5;
        ((h0 + d.signum() * d.abs().min(amt)) % 1.0 + 1.0) % 1.0
    };
    if k < 0.0 {
        hsv_rgb(toward(h, 0.02, 0.025 * -k), (s + 0.06 * -k).min(1.0), (v * (1.0 - 0.15 * -k)).max(0.08))
    } else {
        hsv_rgb(toward(h, 0.12, 0.02 * k), s * (1.0 - 0.13 * k), (v * (1.0 + 0.09 * k)).min(0.96))
    }
}

// ------------------------------------------------------------------ layers and the painter

/// A model's coat layers (`layers.py` in the skill): per texel, `info` (coat, light level, how far
/// toward the underside), `pos` (where on the body, scaled into `bounds`), `props` (white,
/// points, mask) and `props2` (extremity, where patterns show).
struct Layers {
    w: usize,
    h: usize,
    info: Vec<u8>,
    pos: Vec<u8>,
    props: Vec<u8>,
    props2: Vec<u8>,
    bounds: [[f32; 3]; 2],
}

macro_rules! model_layers {
    ($($name:literal),*) => {
        [$(($name, [
            include_bytes!(concat!("../../genetics/layers/", $name, "/info.png")).as_slice(),
            include_bytes!(concat!("../../genetics/layers/", $name, "/pos.png")).as_slice(),
            include_bytes!(concat!("../../genetics/layers/", $name, "/props.png")).as_slice(),
            include_bytes!(concat!("../../genetics/layers/", $name, "/props2.png")).as_slice(),
        ], include_str!(concat!("../../genetics/layers/", $name, "/layers.json")))),*]
    };
}

type ModelLayers = (&'static str, [&'static [u8]; 4], &'static str);
const MODEL_LAYERS: [ModelLayers; 8] = model_layers!("chihuahua", "cat", "capybara", "raptor", "utahraptor", "wyvern", "hare", "mouse");

fn decode(png_bytes: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    let mut d = png::Decoder::new(std::io::Cursor::new(png_bytes));
    d.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA | png::Transformations::STRIP_16);
    let mut r = d.read_info().ok()?;
    let mut buf = vec![0; r.output_buffer_size()?];
    let info = r.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());
    (info.color_type == png::ColorType::Rgba).then_some((info.width as usize, info.height as usize, buf))
}

/// A model's layers, decoded on first use.
fn layers(model: &str) -> Option<&'static Layers> {
    static DECODED: OnceLock<Vec<(&'static str, Option<Layers>)>> = OnceLock::new();
    DECODED
        .get_or_init(|| {
            MODEL_LAYERS
                .iter()
                .map(|(name, pngs, json)| {
                    let read = || {
                        #[derive(Deserialize)]
                        struct Meta {
                            bounds: [[f32; 3]; 2],
                        }
                        let meta: Meta = serde_json::from_str(json).ok()?;
                        let [info, pos, props, props2] = pngs.map(decode);
                        let (w, h, info) = info?;
                        Some(Layers { w, h, info, pos: pos?.2, props: props?.2, props2: props2?.2, bounds: meta.bounds })
                    };
                    let l = read();
                    if l.is_none() {
                        crate::problem(format!("genetics/layers/{name}: unreadable"));
                    }
                    (*name, l)
                })
                .collect()
        })
        .iter()
        .find(|(n, _)| *n == model)
        .and_then(|(_, l)| l.as_ref())
}

/// Paints the coat of an animal of species `index` from its genome over its kind's texture
/// (`rgba`, rows of `width` pixels, `height` of them).
pub fn paint(index: usize, genome: &Genome, rgba: &mut [u8], width: usize, height: usize) {
    if genome.species as usize != index + 1 {
        return;
    }
    let mut genome = *genome;
    mend(&mut genome);
    let (Some(look), Some(l)) = (phenotype(&genome), layers(SPECIES[index].model)) else { return };
    render(l, rgba, width, height, &look);
}

/// How `genome`'s body is built: each bone its build traits scale, and how much (the traits as
/// they show, each through its species' `build`).
pub fn build(genome: &Genome) -> Vec<(String, [f32; 3])> {
    let Some(g) = (genome.species as usize)
        .checked_sub(1)
        .and_then(|i| SPECIES.get(i))
        .and_then(|s| gene_file(s.name))
    else {
        return Vec::new();
    };
    let r = Read { g, genome };
    let mut out: Vec<(String, [f32; 3])> = Vec::new();
    let mut ids: Vec<&String> = g.build.keys().collect();
    ids.sort();
    for id in ids {
        let st = &g.build[id];
        let t = (r.qv(id) - 0.5) * 2.0 * st.range;
        let scale = st.axes.map(|a| 1.0 + t * a);
        for bone in &st.bones {
            match out.iter_mut().find(|(b, _)| b == bone) {
                Some((_, s)) => *s = [s[0] * scale[0], s[1] * scale[1], s[2] * scale[2]],
                None => out.push((bone.clone(), scale)),
            }
        }
    }
    out
}

fn render(l: &Layers, rgba: &mut [u8], width: usize, height: usize, look: &Look) {
    let [lo, hi] = l.bounds;
    let s = look.seed as i32;
    let (w, h) = (l.w.min(width), l.h.min(height));
    let at = |x: usize, y: usize| ((y * l.w + x) * 4, (y * width + x) * 4);
    let lum = |c: &[u8]| 0.3 * c[0] as f32 + 0.59 * c[1] as f32 + 0.11 * c[2] as f32;
    // The eyes' iris in its color, keeping its painted light: the brightest iris texel gets the
    // color, darker ones (the rim, the shade under the lid) darker shades of it.
    let brightest = look.eye.map_or(0.0, |_| {
        (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| l.props2[at(x, y).0 + 2] > 127)
            .map(|(x, y)| lum(&rgba[at(x, y).1..]))
            .fold(0.0, f32::max)
    });
    let scars = scar_shapes(l, look);
    let marks = crisp_marks(l, look, w, h);
    for y in 0..h {
        for x in 0..w {
            let (i, o) = at(x, y);
            let mark = marks[y * l.w + x];
            if let Some(eye) = look.eye
                && l.props2[i + 2] > 127
                && brightest > 0.0
            {
                let k = lum(&rgba[o..]) / brightest;
                for c in 0..3 {
                    rgba[o + c] = (eye[c] * k).round().clamp(0.0, 255.0) as u8;
                }
                continue;
            }
            if l.info[i] < 128 {
                continue;
            }
            let level = l.info[i + 1] as f32 / 10.0;
            let u = l.info[i + 2] as f32 / 255.0;
            let p = [0, 1, 2].map(|k| (lo[k] + l.pos[i + k] as f32 / 255.0 * (hi[k] - lo[k])) / look.scale.max(0.01));
            let (white, points, mask) = (l.props[i] as f32 / 255.0, l.props[i + 1] as f32 / 255.0, l.props[i + 2] as f32 / 255.0);
            let (ext, zone) = (l.props2[i] as f32 / 255.0, l.props2[i + 1] as f32 / 255.0);
            let m = &look.marks;
            // Stark marks show fully, faint ones blend into the coat.
            let contrast = 0.45 + 1.1 * m.contrast;
            let (mut b, mut un, mut pat) = (look.base, look.under, look.pattern);
            if let Some(other) = look.patches.as_deref()
                && mark[3] == 2
            {
                (b, un, pat) = (other.base, other.under, other.pattern);
            }
            let mut c = mix(b, un, u);
            if let Some(mc) = look.merle {
                let k = 0.3 / 2f32.powf((m.size - 0.5) * 1.6);
                c = mix(c, mc, 0.8 * (1.0 - smooth(noise3(p[0] * k, p[1] * k, p[2] * k, s + 3), 0.58, 0.64)));
            }
            for (k, pt) in [pat, look.pattern2].into_iter().enumerate() {
                if let Some(pt) = pt {
                    c = mix(c, pt.color, (TONES[mark[k] as usize] * zone * pt.amount * contrast).min(1.0));
                }
            }
            if let Some(mut pc) = look.points {
                if let Some(pp) = look.points_pattern {
                    pc = mix(pc, pp.color, (TONES[mark[2] as usize] * pp.amount * contrast).min(1.0));
                }
                let at = 0.4 - 0.3 * look.points_reach;
                c = mix(c, pc, tones3(smooth(points, at, at + 0.22)));
            }
            if let Some(mk) = look.mask {
                c = mix(c, mk, 0.9 * tones3(mask));
            }
            if look.elder {
                // Grey hairs on the muzzle, and the whole coat a little faded.
                let grey = (c[0] + c[1] + c[2]) / 3.0;
                c = mix(c, [grey, grey, grey], 0.2);
                let hair = if hash3(x as i32, y as i32, 3, s + 61) > 0.45 { 1.0 } else { 0.4 };
                c = mix(c, [214.0, 210.0, 204.0], 0.6 * hair * tones3(mask));
            }
            if let Some((color, amount)) = look.ext {
                c = mix(c, color, tones3(smooth(ext * amount, 0.25, 0.9)));
            }
            if look.white > 0.0 {
                // White's edges as ragged as the marks'.
                let wn = white + (noise3(p[0] * 0.22, p[1] * 0.22, p[2] * 0.22, s + 5) - 0.5) * (0.15 + 0.3 * m.edge);
                let thr = 1.0 - look.white;
                c = mix(c, WHITE, smooth(wn, thr, thr + 0.05));
            }
            let at_rest = [0, 1, 2].map(|k| lo[k] + l.pos[i + k] as f32 / 255.0 * (hi[k] - lo[k]));
            let mut torn = false;
            for sc in &scars {
                let a = tones3(sc.cover(at_rest, s));
                if a <= 0.0 {
                    continue;
                }
                c = match sc.kind {
                    // Scorched dark in the middle, its edge pale.
                    BURN => mix(c, mix(mix(darken(c, 0.5), [124.0, 62.0, 52.0], 0.35), lighten(c, 0.3), (1.0 - a) * 0.8), a.min(1.0)),
                    TEAR => {
                        torn |= a > 0.5;
                        mix(c, darken(c, 0.45), a)
                    }
                    // Healed skin, paler and pinker than the coat, where the hair grows back white.
                    _ => mix(c, mix(lighten(c, 0.4), [214.0, 182.0, 172.0], 0.4), a * 0.85),
                };
            }
            if torn {
                rgba[o + 3] = 0;
                continue;
            }
            let t = tone(c, -3.0 + level * 0.25);
            for k in 0..3 {
                rgba[o + k] = t[k].round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}
