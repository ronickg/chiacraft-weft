//! Coats from genes (`notes/ANIMALS.md`, "Looks"). Every animal carries a genome (`GENOME`, saved,
//! and synced to everyone who sees it): drawn from its species' gene file
//! (`genetics/<species>.json`; a breed's own frequencies for a breed) when it spawns, or one
//! allele of each locus from each parent when it is born. Each client paints its coat from that
//! (`Registry::entity_skin`): the phenotype (real coat genetics, simplified, per species) on the
//! coat layers its model's build wrote (`genetics/layers/<model>`: what is coat, its light, where
//! on the body, where white, points, a mask and patterns show). Only the genome travels.
//!
//! These are the rules of the viewer's `coats.js` (`.claude/skills/blocky-model/scripts`), where
//! coats are tried out: change both together.

use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use serde::Deserialize;
use weft_sdk::*;

use crate::animal::Animal;
use crate::species::SPECIES;

value! {
    #[derive(Default)]
    /// An animal's genes; its coat is painted from them.
    pub struct Genome {
        /// Where its patterns and white fall.
        pub seed: u32,
        /// Species + 1 (0: none yet).
        pub species: u8,
        /// Its breed + 1, in its gene file's `breeds` (0: none; `MIXED`: parents of two breeds).
        pub breed: u8,
        pub female: u8,
        /// Two per locus, in the gene file's order: an allele's index (`NONE`: a male's missing
        /// second X), or a quantitative trait's value (0-255 for 0-1).
        pub genes: [u8; 32],
    }
}

pub static GENOME: Component<Genome> = Component::named("genome");
pub const MIXED: u8 = 255;
const NONE: u8 = 255;

pub fn init(r: &mut Registry) {
    r.entity_component(
        &GENOME,
        Genome::default(),
        component_flags::SAVE | component_flags::SYNC_WATCHERS | component_flags::SYNC_MENU,
    );
    for (i, s) in SPECIES.iter().enumerate() {
        r.attach(s.kind, &GENOME);
        r.entity_skin(s.kind, &GENOME, move |g, skin| paint(i, g, skin));
    }
}

// ------------------------------------------------------------------ gene files

#[derive(Deserialize)]
struct GeneFile {
    loci: Vec<Locus>,
    /// By name, in name order (a genome keeps its breed as an index into these).
    #[serde(default)]
    breeds: BTreeMap<String, Breed>,
}

#[derive(Deserialize)]
struct Locus {
    id: String,
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
}

#[derive(Deserialize)]
struct Allele {
    a: String,
    #[serde(default)]
    freq: f32,
}

#[derive(Deserialize, Default)]
struct Breed {
    #[serde(default)]
    freq: HashMap<String, HashMap<String, f32>>,
    #[serde(default)]
    quantitative: HashMap<String, Spread>,
}

#[derive(Deserialize)]
struct Spread {
    mean: f32,
    sd: f32,
}

const FILES: [(&str, &str); 6] = [
    ("dog", include_str!("../genetics/dog.json")),
    ("cat", include_str!("../genetics/cat.json")),
    ("capybara", include_str!("../genetics/capybara.json")),
    ("raptor", include_str!("../genetics/raptor.json")),
    ("utahraptor", include_str!("../genetics/utahraptor.json")),
    ("osmolskae", include_str!("../genetics/osmolskae.json")),
];

/// A species' gene file (by `Species::name`).
fn gene_file(species: &str) -> Option<&'static GeneFile> {
    static PARSED: OnceLock<Vec<(&'static str, GeneFile)>> = OnceLock::new();
    PARSED
        .get_or_init(|| {
            FILES
                .iter()
                .filter_map(|(n, json)| match serde_json::from_str(json) {
                    Ok(f) => Some((*n, f)),
                    Err(e) => {
                        weft_sdk::error!("genetics/{n}.json: {e}");
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

/// The world's randomness as 0..1 numbers.
pub fn world_rng(w: &World) -> impl FnMut() -> f32 + '_ {
    move || w.random_below(1 << 24) as f32 / (1 << 24) as f32
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

/// A gene pool: per locus of a species' gene file, each allele's share, or a quantitative
/// trait's mean and spread (`[mean, sd]`). How common each variant is in a population; what
/// evolves (`evolution.rs`).
pub type Pool = Vec<Vec<f32>>;

/// The breed species `index` is (+ 1, as `Genome::breed`), and its frequencies.
fn breed_of(index: usize, g: &GeneFile) -> (u8, Option<&Breed>) {
    SPECIES[index]
        .breed
        .and_then(|n| g.breeds.iter().position(|(k, _)| k == n).map(|i| (i as u8 + 1, &g.breeds[n])))
        .map_or((0, None), |(i, b)| (i, Some(b)))
}

/// Species `index`'s pool as its gene file has it (its breed's frequencies for a breed): where
/// every local pool starts.
pub fn wild_pool(index: usize) -> Option<Pool> {
    let g = gene_file(SPECIES.get(index)?.name)?;
    let (_, b) = breed_of(index, g);
    Some(
        g.loci
            .iter()
            .take(16)
            .map(|l| {
                if l.quantitative {
                    let (mean, sd) = b
                        .and_then(|b| b.quantitative.get(&l.id))
                        .map_or((l.mean, l.sd), |q| (q.mean, q.sd));
                    vec![mean, sd]
                } else {
                    let f = b.and_then(|b| b.freq.get(&l.id));
                    l.alleles
                        .iter()
                        .map(|a| f.and_then(|m| m.get(&a.a)).copied().unwrap_or(a.freq))
                        .collect()
                }
            })
            .collect(),
    )
}

/// A new animal of species `index` drawn from `pool` (one copy on the X for males).
pub fn sample(index: usize, female: bool, pool: &Pool, r: &mut impl Rng) -> Option<Genome> {
    let g = gene_file(SPECIES.get(index)?.name)?;
    let (breed, _) = breed_of(index, g);
    let mut genes = [NONE; 32];
    for (i, l) in g.loci.iter().enumerate().take(16) {
        let p = pool.get(i)?;
        if l.quantitative {
            let (mean, sd) = (p.first().copied().unwrap_or(l.mean), p.get(1).copied().unwrap_or(l.sd));
            for k in 0..2 {
                genes[2 * i + k] = unit(mean + sd * gauss(r));
            }
        } else {
            let copies = if l.xlinked && !female { 1 } else { 2 };
            for k in 0..copies {
                genes[2 * i + k] = if p.len() == l.alleles.len() { pick(p, r) } else { wild_allele(l, r) };
            }
        }
    }
    Some(Genome { seed: (r.next() * 1e9) as u32, species: index as u8 + 1, breed, female: female as u8, genes })
}

/// The pool `genomes` (of one species) make: how common each allele is among them, each
/// quantitative trait's mean and spread.
pub fn pool_of(index: usize, genomes: &[Genome]) -> Option<Pool> {
    let g = gene_file(SPECIES.get(index)?.name)?;
    Some(
        g.loci
            .iter()
            .enumerate()
            .take(16)
            .map(|(i, l)| {
                let values = genomes
                    .iter()
                    .flat_map(|x| [x.genes[2 * i], x.genes[2 * i + 1]])
                    .filter(|&a| a != NONE);
                if l.quantitative {
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
                }
            })
            .collect(),
    )
}

/// `pool` (of `n` animals) with genome `g`'s alleles taken out (`sign` -1: one of them died or
/// was tamed) or added (+1: one joined): each allele's share, each trait's mean, as they then are.
pub fn count_in(index: usize, pool: &mut Pool, g: &Genome, sign: f32, n: f32) {
    let Some(file) = SPECIES.get(index).and_then(|s| gene_file(s.name)) else { return };
    let total = 2.0 * n;
    for ((i, l), p) in file.loci.iter().enumerate().zip(pool.iter_mut()) {
        let have = [g.genes[2 * i], g.genes[2 * i + 1]];
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
        .zip(pool)
        .map(|(l, p)| {
            if l.quantitative {
                format!("{} {:.2}±{:.2}", l.id, p.first().unwrap_or(&0.0), p.get(1).unwrap_or(&0.0))
            } else {
                let mut a: Vec<(&str, f32)> = l.alleles.iter().map(|a| a.a.as_str()).zip(p.iter().copied()).collect();
                a.sort_by(|x, y| y.1.total_cmp(&x.1));
                let shown: Vec<String> = a.iter().filter(|x| x.1 >= 0.01).map(|(n, f)| format!("{n} {f:.2}")).collect();
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
    let mut genes = [NONE; 32];
    let mut rnd = || r.next();
    for (i, l) in g.loci.iter().enumerate().take(16) {
        let (m, f) = ([mother.genes[2 * i], mother.genes[2 * i + 1]], [father.genes[2 * i], father.genes[2 * i + 1]]);
        if l.quantitative {
            for (k, p) in [m, f].into_iter().enumerate() {
                let v = value(one(p, &mut rnd));
                genes[2 * i + k] = unit(v + 0.05 * gauss(&mut rnd));
            }
            continue;
        }
        genes[2 * i] = one(m, &mut rnd);
        if !l.xlinked || female {
            genes[2 * i + 1] = one(f, &mut rnd);
        }
        if rnd() < mutation {
            let k = if genes[2 * i + 1] != NONE && rnd() < 0.5 { 1 } else { 0 };
            genes[2 * i + k] = wild_allele(l, &mut rnd);
        }
    }
    let breed = if mother.breed == father.breed { mother.breed } else { MIXED };
    Some(Genome { seed: (rnd() * 1e9) as u32, species: mother.species, breed, female: female as u8, genes })
}

/// Every tick, for an animal of species `index` at `pos` (`animal::upkeep`): one without genes
/// gets them from the wild around it (`evolution::pool_at`: how its kind has evolved there), of
/// its sex; one born with them takes its sex from them.
pub fn upkeep(w: &World, id: u64, index: usize, a: &mut Animal, pos: [f64; 3]) {
    let Some(g) = w.component_mut(id, &GENOME) else { return };
    if g.species as usize == index + 1 {
        a.female = g.female;
        return;
    }
    let Some(pool) = crate::evolution::pool_at(w, index, pos) else { return };
    if let Some(new) = sample(index, a.female != 0, &pool, &mut world_rng(w)) {
        *g = new;
    }
}

// ------------------------------------------------------------------ phenotypes

type Rgb = [f32; 3];

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
    seed: u32,
}

impl Look {
    /// The color it shows at a glance, all over: what camouflages it or gives it away
    /// (`evolution.rs`).
    pub fn average(&self) -> [f32; 3] {
        let mut c = mix(self.base, self.under, 0.3);
        if let Some(p) = &self.patches {
            c = mix(c, mix(p.base, p.under, 0.3), 0.4);
        }
        for p in [self.pattern, self.pattern2, self.points_pattern].into_iter().flatten() {
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

/// A genome read against its gene file.
struct Read<'a> {
    g: &'a GeneFile,
    genome: &'a Genome,
}

impl Read<'_> {
    fn locus(&self, id: &str) -> Option<(usize, &Locus)> {
        self.g.loci.iter().enumerate().find(|(_, l)| l.id == id)
    }
    fn pair(&self, i: usize) -> impl Iterator<Item = u8> + '_ {
        [self.genome.genes[2 * i], self.genome.genes[2 * i + 1]]
            .into_iter()
            .filter(|&a| a != NONE)
    }
    /// What shows at a locus: the most dominant allele present.
    fn shows(&self, id: &str) -> &str {
        let Some((i, l)) = self.locus(id) else { return "" };
        let have: Vec<u8> = self.pair(i).collect();
        (0..l.alleles.len())
            .find(|a| have.contains(&(*a as u8)))
            .or(have.first().map(|&a| a as usize))
            .and_then(|a| l.alleles.get(a))
            .map_or("", |a| a.a.as_str())
    }
    /// How many copies of `allele` it has.
    fn copies(&self, id: &str, allele: &str) -> usize {
        let Some((i, l)) = self.locus(id) else { return 0 };
        self.pair(i)
            .filter(|&a| l.alleles.get(a as usize).is_some_and(|x| x.a == allele))
            .count()
    }
    /// A quantitative trait's value, 0..1.
    fn qv(&self, id: &str) -> f32 {
        self.locus(id)
            .map_or(0.5, |(i, _)| (value(self.genome.genes[2 * i]) + value(self.genome.genes[2 * i + 1])) / 2.0)
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
    let pheo_name = if i < 0.3 { "Cream" } else if i < 0.7 { "Fawn" } else { "Red" };
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
    let shade = if dk < 0.33 { "Light tan" } else if dk < 0.66 { "Brown" } else { "Dark brown" };
    let tint = if ru > 0.62 { ", reddish" } else if ru < 0.38 { ", greyish" } else { "" };
    Look {
        name: format!("{shade}{tint}"),
        base,
        under: lighten(base, 0.12 + 0.4 * bl),
        ext: Some((darken(base, 0.4), 0.7)),
        ..Look::default()
    }
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

/// How big an adult with `genome` is against its species' average (its model): its size gene
/// (`Sz`, spread by the species' `size_spread`) and its sex (`dimorphism`, males against
/// females; the average stays the model's).
pub fn size(genome: &Genome) -> f32 {
    let Some(s) = (genome.species as usize).checked_sub(1).and_then(|i| SPECIES.get(i)) else { return 1.0 };
    let sz = size_gene(genome);
    let sex = if genome.female != 0 { s.dimorphism.sqrt().recip() } else { s.dimorphism.sqrt() };
    (1.0 + s.size_spread * (sz - 0.5) * 2.0) * sex
}

/// Its size gene (`Sz`), 0..1: small to big for its kind.
pub fn size_gene(genome: &Genome) -> f32 {
    let Some(s) = (genome.species as usize).checked_sub(1).and_then(|i| SPECIES.get(i)) else { return 0.5 };
    gene_file(s.name).map_or(0.5, |g| Read { g, genome }.qv("Sz"))
}

/// What `genome` looks like (none before it has genes).
pub fn phenotype(genome: &Genome) -> Option<Look> {
    let s = SPECIES.get((genome.species as usize).checked_sub(1)?)?;
    let r = Read { g: gene_file(s.name)?, genome };
    let mut look = match s.name {
        "dog" => dog(&r),
        "cat" => cat(&r),
        "capybara" => capybara(&r),
        _ => raptor(&r),
    };
    look.seed = genome.seed;
    if let Some(p) = &mut look.patches {
        p.seed = genome.seed;
    }
    Some(look)
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
    l(
        l(l(c(0, 0, 0), c(1, 0, 0), u), l(c(0, 1, 0), c(1, 1, 0), u), v),
        l(l(c(0, 0, 1), c(1, 0, 1), u), l(c(0, 1, 1), c(1, 1, 1), u), v),
        w,
    )
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
/// the body), `u` how far toward the underside.
fn pattern(kind: &str, p: [f32; 3], u: f32, s: i32) -> f32 {
    let [x, y, z] = p;
    match kind {
        "mackerel" | "bars" => band((z + 1.6 * (y * 0.45).sin() + 0.5 * x.abs()) / 5.5, 0.32),
        "classic" => band(((z - 2.0).hypot(y - 14.0)) / 6.5 + 0.15 * noise3(x * 0.3, y * 0.3, z * 0.3, s), 0.42),
        "spotted" => band((z + 1.6 * (y * 0.45).sin()) / 5.5, 0.4) * smooth(noise3(x * 0.55, y * 0.55, z * 0.55, s), 0.45, 0.55),
        "ticked" => 0.35 * smooth(noise3(x * 1.4, y * 1.4, z * 1.4, s), 0.55, 0.7),
        "bands" => band((z + 0.4 * y) / 7.0, 0.38),
        "rosettes" => {
            let (cx, cy, cz) = ((x / 4.0).floor(), (y / 4.0).floor(), (z / 4.0).floor());
            if hash3(cx as i32, cy as i32, cz as i32, s) > 0.55 {
                return 0.0;
            }
            let (dx, dz, dy) = (x - (cx + 0.5) * 4.0, z - (cz + 0.5) * 4.0, (y - (cy + 0.5) * 4.0) * 0.6);
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            smooth(d, 0.6, 0.8) * (1.0 - smooth(d, 1.5, 1.7))
        }
        "speckle" => (hash3(x.floor() as i32, y.floor() as i32, z.floor() as i32, s) < 0.14) as u8 as f32,
        "saddle" => smooth(0.55 - u, 0.0, 0.25),
        "sable" => smooth(0.75 - u, 0.0, 0.5) * (0.55 + 0.45 * noise3(x * 0.6, y * 0.6, z * 0.6, s)),
        "brindle" => band((z + 2.5 * noise3(x * 0.25, y * 0.25, z * 0.25, s) * 3.0) / 2.2, 0.3),
        "patches" => smooth(noise3(x * 0.15, y * 0.15, z * 0.15, s), 0.5, 0.56),
        _ => 0.0,
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
            include_bytes!(concat!("../genetics/layers/", $name, "/info.png")).as_slice(),
            include_bytes!(concat!("../genetics/layers/", $name, "/pos.png")).as_slice(),
            include_bytes!(concat!("../genetics/layers/", $name, "/props.png")).as_slice(),
            include_bytes!(concat!("../genetics/layers/", $name, "/props2.png")).as_slice(),
        ], include_str!(concat!("../genetics/layers/", $name, "/layers.json")))),*]
    };
}

type ModelLayers = (&'static str, [&'static [u8]; 4], &'static str);
const MODEL_LAYERS: [ModelLayers; 5] = model_layers!("chihuahua", "cat", "capybara", "raptor", "utahraptor");

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
                        weft_sdk::error!("genetics/layers/{name}: unreadable");
                    }
                    (*name, l)
                })
                .collect()
        })
        .iter()
        .find(|(n, _)| *n == model)
        .and_then(|(_, l)| l.as_ref())
}

/// Paints the coat of an animal of species `index` from its genome over its kind's texture.
fn paint(index: usize, genome: &Genome, skin: &mut Skin) {
    if genome.species as usize != index + 1 {
        return;
    }
    let (Some(look), Some(l)) = (phenotype(genome), layers(SPECIES[index].model)) else { return };
    render(l, skin, &look);
}

fn render(l: &Layers, skin: &mut Skin, look: &Look) {
    let [lo, hi] = l.bounds;
    let s = look.seed as i32;
    let (w, h) = (l.w.min(skin.width as usize), l.h.min(skin.height as usize));
    for y in 0..h {
        for x in 0..w {
            let i = (y * l.w + x) * 4;
            let o = (y * skin.width as usize + x) * 4;
            if l.info[i] < 128 {
                continue;
            }
            let level = l.info[i + 1] as f32 / 10.0;
            let u = l.info[i + 2] as f32 / 255.0;
            let p = [0, 1, 2].map(|k| lo[k] + l.pos[i + k] as f32 / 255.0 * (hi[k] - lo[k]));
            let (white, points, mask) = (l.props[i] as f32 / 255.0, l.props[i + 1] as f32 / 255.0, l.props[i + 2] as f32 / 255.0);
            let (ext, zone) = (l.props2[i] as f32 / 255.0, l.props2[i + 1] as f32 / 255.0);
            let (mut b, mut un, mut pat) = (look.base, look.under, look.pattern);
            if let Some(other) = look.patches.as_deref()
                && pattern("patches", p, u, s + 7) > 0.5
            {
                (b, un, pat) = (other.base, other.under, other.pattern);
            }
            let mut c = mix(b, un, u);
            if let Some(m) = look.merle {
                c = mix(c, m, 0.8 * (1.0 - smooth(noise3(p[0] * 0.3, p[1] * 0.3, p[2] * 0.3, s + 3), 0.58, 0.64)));
            }
            for pt in [pat, look.pattern2].into_iter().flatten() {
                c = mix(c, pt.color, pattern(pt.kind, p, u, s) * zone * pt.amount);
            }
            if let Some(mut pc) = look.points {
                if let Some(pp) = look.points_pattern {
                    pc = mix(pc, pp.color, pattern(pp.kind, p, u, s) * pp.amount);
                }
                c = mix(c, pc, smooth(points, 0.4, 0.62));
            }
            if let Some(m) = look.mask {
                c = mix(c, m, 0.9 * mask);
            }
            if let Some((color, amount)) = look.ext {
                c = mix(c, color, smooth(ext * amount, 0.25, 0.9));
            }
            if look.white > 0.0 {
                let wn = white + (noise3(p[0] * 0.22, p[1] * 0.22, p[2] * 0.22, s + 5) - 0.5) * 0.3;
                let thr = 1.0 - look.white;
                c = mix(c, WHITE, smooth(wn, thr, thr + 0.05));
            }
            let t = tone(c, -3.0 + level * 0.25);
            for k in 0..3 {
                skin.rgba[o + k] = t[k].round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}
