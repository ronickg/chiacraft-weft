//! Where each species lives (`notes/ANIMALS.md`, "Places, food and sedation"): its habitats are
//! facts in its row (`Species::habitats`, biomes with how common it is in each), and natural
//! spawning follows them: a rule per species and biome, as often as its rarity has it there
//! (`Rarity::spawn_chance`), in groups as its social life has them (`Social::group`), on the
//! ground animals stand on, by day.

use weft_sdk::*;

use crate::species::SPECIES;

/// The ground a wild animal may spawn on: grass, snowy grass, sand, red sand.
const GROUND: &str = "habitat_ground";

/// The tag a biome is given so rules can name it (spawn rules match biomes by tag).
fn tag(biome: &str) -> String {
    format!("habitat/{}", biome.replace(':', "/"))
}

pub fn init(r: &mut Registry) {
    r.tag(GROUND, &["base:grass", "base:snowy_grass", "base:sand", "animals:red_sand"]);
    let mut tagged: Vec<&str> = Vec::new();
    for s in &SPECIES {
        for &(biome, share) in s.habitats {
            if !tagged.contains(&biome) {
                r.tag_on(Target::Biome(biome), &tag(biome));
                tagged.push(biome);
            }
            if s.in_water() {
                continue; // fish come by their own spawner, into water (`fish.rs`)
            }
            let (few, most) = s.bio.social.group();
            r.spawn(s.kind)
                .biome(&format!("#{}", tag(biome)))
                .ground(&format!("#{GROUND}"))
                .light(7, 15)
                .group(few, most)
                .chance(s.bio.rarity.spawn_chance() * share);
        }
    }
}

/// The land within `radius` of `at` by biome (a column every 32 blocks, as the generator makes
/// it), a line for the biomes (each with its nearest spot) and one per species with the share of
/// the land it lives on.
pub fn survey(w: &World, at: [i32; 2], radius: i32) -> Vec<String> {
    let dim = Dimension::generated(name_of(abi::entry_kind::DIMENSION, w.dimension()).to_string());
    // Each biome: columns sampled, and the nearest one (distance squared, x, z).
    let mut biomes: Vec<(String, u32, (i64, i32, i32))> = Vec::new();
    let mut total = 0;
    for z in (-radius..=radius).step_by(32) {
        for x in (-radius..=radius).step_by(32) {
            let c = dim.column(w.seed(), at[0] + x, at[1] + z);
            let name = name_of(abi::entry_kind::BIOME, c.biome).to_string();
            let spot = ((x as i64).pow(2) + (z as i64).pow(2), at[0] + x, at[1] + z);
            match biomes.iter_mut().find(|b| b.0 == name) {
                Some(b) => (b.1, b.2) = (b.1 + 1, b.2.min(spot)),
                None => biomes.push((name, 1, spot)),
            }
            total += 1;
        }
    }
    biomes.sort_by(|a, b| b.1.cmp(&a.1));
    let pct = |n: f32| format!("{:.1}%", n * 100.0 / total.max(1) as f32);
    let mut lines = vec![
        biomes
            .iter()
            .map(|(n, k, (_, x, z))| format!("{n} {} (nearest {x} {z})", pct(*k as f32)))
            .collect::<Vec<_>>()
            .join(", "),
    ];
    for s in &SPECIES {
        // Land weighted by how common it is in each biome.
        let land: f32 = s
            .habitats
            .iter()
            .map(|&(b, share)| {
                biomes
                    .iter()
                    .find(|x| x.0 == b)
                    .map_or(0.0, |x| x.1 as f32 * share)
            })
            .sum();
        lines.push(format!("{}: {} of the land (weighted by how common it is there)", s.kind_name(), pct(land)));
    }
    lines
}
