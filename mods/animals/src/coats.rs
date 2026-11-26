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

//! The rules themselves are in `animals_core::coats` (shared with the website's simulator); here
//! the genome is a component, each client paints from it, and animals get their genes.

pub use animals_core::coats::*;
// Named, as `weft_sdk` has a `Pool` of its own.
pub use animals_core::coats::{Pool, Rng};
use weft_sdk::*;

use crate::animal::Animal;
use crate::species::SPECIES;

pub static GENOME: Component<Genome> = Component::named("genome");

/// The size an animal's genes draw (`size`) and the genes it was read from (a fingerprint), not
/// saved: reading its gene file every tick for every animal cost the tick (P3), and its genes
/// rarely change (given once, `upkeep`).
static SIZE: Component<[u32; 2]> = Component::named("animal_gene_size");

pub fn init(r: &mut Registry) {
    r.entity_component(&GENOME, Genome::default(), component_flags::SAVE | component_flags::SYNC_WATCHERS | component_flags::SYNC_MENU);
    r.entity_component(&SIZE, [0u32; 2], 0);
    for (i, s) in SPECIES.iter().enumerate() {
        r.edit(s.kind).attach(&GENOME);
        r.edit(s.kind).attach(&SIZE);
        r.entity_skin(s.kind, &GENOME, move |g, skin| paint_skin(i, g, skin));
    }
    for p in animals_core::problems() {
        weft_sdk::error!("{p}");
    }
}

/// The size `id`'s genes draw (`size`): read once for the genes it has, then kept.
pub fn size_of(w: &World, id: u64) -> Option<f32> {
    let g = w.component(id, &GENOME)?;
    let print = fingerprint(&g);
    if let Some([bits, p]) = w.component(id, &SIZE)
        && bits != 0
        && p == print
    {
        return Some(f32::from_bits(bits));
    }
    let size = size(&g);
    if let Some(c) = w.component_mut(id, &SIZE) {
        *c = [size.to_bits(), print];
    }
    Some(size)
}

/// What a genome's size is read from: its species, sex and genes (FNV-1a).
fn fingerprint(g: &Genome) -> u32 {
    [g.species, g.female]
        .iter()
        .chain(&g.genes)
        .fold(0x811c_9dc5u32, |h, &b| (h ^ b as u32).wrapping_mul(0x0100_0193))
}

/// Paints an animal's coat from its genes over its kind's texture, and sets its build.
fn paint_skin(index: usize, genome: &Genome, skin: &mut Skin) {
    paint(index, genome, skin.rgba, skin.width as usize, skin.height as usize);
    if genome.species as usize == index + 1 {
        for (bone, scale) in build(genome) {
            skin.scale_bone(&bone, scale);
        }
    }
    for p in animals_core::problems() {
        weft_sdk::error!("{p}");
    }
}

/// The world's randomness as 0..1 numbers.
pub fn world_rng(w: &World) -> impl FnMut() -> f32 + '_ {
    move || w.random_below(1 << 24) as f32 / (1 << 24) as f32
}

/// Every tick, for an animal of species `index` at `pos` (`animal::upkeep`): one without genes
/// gets them from the wild around it (`evolution::pool_at`: how its kind has evolved there), of
/// its sex; one born with them takes its sex from them. One saved before the core traits (or a
/// locus its species gained since) gets those from the wild too (`upgrade`), and its genome keeps
/// its life stage (a young one looks young).
pub fn upkeep(w: &World, id: u64, index: usize, a: &mut Animal, pos: [f64; 3]) {
    let Some(g) = w.component_mut(id, &GENOME) else { return };
    if g.species as usize == index + 1 {
        a.female = g.female;
        if g.version < VERSION {
            let Some(pool) = crate::evolution::pool_at(w, index, pos) else { return };
            upgrade(index, g, &pool, &mut world_rng(w));
        }
        if g.stage != a.stage {
            g.stage = a.stage;
        }
        return;
    }
    let Some(pool) = crate::evolution::pool_at(w, index, pos) else { return };
    if let Some(mut new) = sample(index, a.female != 0, &pool, &mut world_rng(w)) {
        new.stage = a.stage;
        *g = new;
    }
}
