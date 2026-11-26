//! Each biome's prey and what its hunters take a game year (`web::predation`).
//!
//!     cargo run -p animals_core --example predation
fn main() {
    for biome in animals_core::web::biomes() {
        let p = animals_core::web::predation(biome);
        if p.is_empty() {
            continue;
        }
        let rooms: Vec<String> = animals_core::species::SPECIES
            .iter()
            .filter(|h| h.hunts() && animals_core::needs::habitat_weight(h, biome) > 0.0)
            .map(|h| format!("{} {}/{}", h.name, h.crowd_in(biome), h.crowd()))
            .collect();
        println!("{biome}  hunters' room/crowd: {}", rooms.join(", "));
        for x in p {
            let by: Vec<String> = x.taken.iter().map(|(h, t)| format!("{h} {t:.1}")).collect();
            println!(
                "  {:20} numbers {:5.1} young/yr {:7.1} taken/yr {:6.1} = {:4.0}% of young  ({})",
                x.prey,
                x.numbers,
                x.young,
                x.taken(),
                x.share() * 100.0,
                by.join(", ")
            );
        }
    }
}
