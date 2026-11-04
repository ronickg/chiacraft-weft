//! Whether the living world's loop holds (`notes/ECOLOGY.md`, "The checks that keep it
//! balanced"): every food in a species' diet exists in the game, every plant eater finds
//! something to eat in each of its habitats in every season, every hunter has prey where it
//! lives, every biome's web is whole (plants, plant eaters, hunters, something for the dead), and
//! everything but the top hunters is somebody's prey. It reads the tables (`species::SPECIES`,
//! `vitals::forage`, `vitals::climate`), so a new species or biome is checked the moment it is
//! added.
//!
//! `gaps()` lists what is missing. Gaps that a planned species or food will fill are in
//! `WAITING`, each with what fills it; the test asks that every gap is listed there and that
//! nothing listed is no longer a gap, so the list stays the plan.

use crate::species::{OTHER_PREY, SPECIES, Species};
use vitals::climate::{self, BIOMES};
use vitals::food::Category;

pub const SEASONS: [&str; 4] = ["spring", "summer", "autumn", "winter"];

/// A gap in the web: `key` names it (the same words every time, for `WAITING`), `text` says what
/// to look at.
#[derive(Clone, Debug, PartialEq)]
pub struct Gap {
    pub key: String,
    pub text: String,
}

/// Gaps that the plan fills (`notes/ECOLOGY.md`, "The food webs and the species to add"): the
/// gap's key and what fills it. Remove an entry when what it waits for comes.
pub const WAITING: &[(&str, &str)] = &[
    // The overworld: what wave two (`notes/ECOLOGY.md`) left.
    ("herring_gull: no prey in base:ocean", "the open sea's web: its fish (later; the gull hunts on the beach meanwhile)"),
    // The Cretaceous's tier 2 and 3.
];

/// The biomes the webs are for: every climate but the caves (no web of their own) and the open
/// sea (its web, gulls and seals, comes later; its fish live in the rivers and lakes).
pub fn biomes() -> impl Iterator<Item = &'static str> {
    BIOMES
        .iter()
        .filter(|c| !c.underground && c.biome != "base:ocean")
        .map(|c| c.biome)
}

fn lives(s: &Species, biome: &str) -> bool {
    s.habitats.iter().any(|&(b, _)| b == biome)
}

fn overworld_land(biome: &str) -> bool {
    biome.starts_with("base:") && biome != "base:ocean"
}

/// How much of that diet's food the biome grows or holds in `season` (`vitals::forage`), 0 to 1.
pub fn food_in(eats: impl Fn(Category) -> bool, biome: &str, season: usize) -> f32 {
    let Some(c) = climate::of(biome) else { return 0.0 };
    vitals::forage::season_lushness(eats, biome, c.celsius, c.seasons, c.rain, Some(season)).unwrap_or(0.0)
}

/// The prey a hunter takes in this biome: species of other families living there within its
/// prey's sizes, and the farm animals' wild kinds on the overworld's land.
pub fn prey_in(s: &Species, biome: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = SPECIES
        .iter()
        .filter(|o| o.family.name != s.family.name && lives(o, biome) && s.takes_at_most(s.bio.mass, o.bio.mass))
        .map(|o| o.name)
        .collect();
    if overworld_land(biome) {
        out.extend(
            OTHER_PREY
                .iter()
                .filter(|&&(_, kg)| s.takes_at_most(s.bio.mass, kg))
                .map(|&(n, _)| n),
        );
    }
    out
}

/// A young one's mass against a grown one's, as hunters meet them: half grown, an eighth of the
/// mass. Hunters weigh prey by its own mass (`hunt.rs`), so the young of animals too big to take
/// grown (a sauropod, an iguanodont) are what keeps them in check, as they were in life.
pub const YOUNG: f32 = 0.125;

/// Who takes this species where it lives, grown or young.
pub fn predators_of(s: &Species) -> Vec<&'static str> {
    SPECIES
        .iter()
        .filter(|h| {
            h.family.name != s.family.name && (h.takes_at_most(h.bio.mass, s.bio.mass) || h.takes_at_most(h.bio.mass, s.bio.mass * YOUNG))
        })
        .filter(|h| s.habitats.iter().any(|&(b, _)| lives(h, b)))
        .map(|h| h.name)
        .collect()
}

/// Whether a kind of food is there for this species somewhere it lives: plants and browse grow
/// (`vitals::forage`), meat and fish are its prey, carrion is what hunters leave.
fn food_exists(s: &Species, c: Category) -> bool {
    match c {
        Category::Meat => s.habitats.iter().any(|&(b, _)| !prey_in(s, b).is_empty()),
        Category::Fish => s
            .habitats
            .iter()
            .any(|&(b, _)| SPECIES.iter().any(|o| o.in_water() && lives(o, b))),
        Category::Carrion => s
            .habitats
            .iter()
            .any(|&(b, _)| SPECIES.iter().any(|h| h.hunts() && lives(h, b))),
        c => vitals::forage::grows(c),
    }
}

fn gap(key: String, text: String) -> Gap {
    Gap { key, text }
}

/// Everything missing from the webs.
pub fn gaps() -> Vec<Gap> {
    let mut out = Vec::new();

    for s in &SPECIES {
        // Every food in its diet is in the game.
        for &(c, share) in s.eats {
            if share > 0.0 && !food_exists(s, c) {
                out.push(gap(
                    format!("{}: no {c:?}", s.name),
                    format!(
                        "{}: eats {c:?} ({:.0}% of its food, species.rs eats), which nothing in the game gives where it lives (vitals::forage SOURCES, or prey there)",
                        s.name,
                        share * 100.0
                    ),
                ));
            }
        }
        // A plant eater finds something every season where it lives.
        if s.forages_wild() && !s.in_water() {
            for &(biome, _) in s.habitats {
                for (i, season) in SEASONS.iter().enumerate() {
                    if food_in(|k| s.eats(k), biome, i) <= 0.0 {
                        out.push(gap(
                            format!("{}: nothing to eat in {biome} in {season}", s.name),
                            format!(
                                "{}: none of its plant food grows or is found in {biome} in {season}: its diet (species.rs eats) against vitals::forage (a browse source there then, or a plant that grows then)",
                                s.name
                            ),
                        ));
                    }
                }
            }
        }
        // A hunter has prey wherever it lives (a mostly-plant eater that hunts a little needn't).
        if s.hunts() && !s.forages_wild() {
            for &(biome, _) in s.habitats {
                if prey_in(s, biome).is_empty() {
                    out.push(gap(
                        format!("{}: no prey in {biome}", s.name),
                        format!("{}: nothing it takes ({:?} of its own mass, species.rs prey) lives in {biome}", s.name, s.prey),
                    ));
                }
            }
        }
        // Everything but the top hunters is somebody's prey somewhere it lives.
        if s.plant_share() > 0.25 && predators_of(s).is_empty() {
            out.push(gap(
                format!("{}: no predator", s.name),
                format!(
                    "{}: no hunter living where it does takes {:.0} kg (the hunters' prey sizes, species.rs prey), so nothing keeps it in check but food",
                    s.name, s.bio.mass
                ),
            ));
        }
    }

    // Every biome's web is whole.
    for biome in biomes() {
        let here: Vec<&Species> = SPECIES.iter().filter(|s| lives(s, biome)).collect();
        let green = (0..4).any(|i| food_in(Category::plant, biome, i) > 0.0);
        let grazers = here.iter().any(|s| s.plant_share() >= 0.5) || overworld_land(biome);
        let hunters = here.iter().any(|s| s.hunts() && !s.forages_wild());
        let dead = here.iter().any(|s| s.eats(Category::Carrion));
        for (has, what) in [(green, "plants"), (grazers, "plant eaters"), (hunters, "hunters"), (dead, "a carrion eater")] {
            if !has {
                out.push(gap(
                    format!("{biome}: no {what}"),
                    format!("{biome}: its web has no {what} (species.rs habitats, vitals::forage SOURCES)"),
                ));
            }
        }
    }
    // A year in each biome holds: its grazers, as many as live together where commonest, can't
    // eat its grass bare in its best season.
    for biome in biomes() {
        if let Some(g) = grazing(biome)
            && g.eaten > g.regrown
        {
            out.push(gap(
                format!("{biome}: grazed bare"),
                format!(
                    "{biome}: its grazers eat {:.0} food a day within {:.0} blocks in {}, its grass grows back {:.0}: the grazers' crowds and hunger (species.rs) against vitals::forage GRASS_COVER and the grazed grass's growth (vitals::plants)",
                    g.eaten,
                    crate::life::CROWD_RADIUS,
                    SEASONS[g.season],
                    g.regrown
                ),
            ));
        }
    }
    out
}

/// A biome's grass and its grazers in its best season for grass (`grazing`).
#[derive(Clone, Copy, Debug)]
pub struct Grazing {
    pub season: usize,
    /// Food a day the grazed grass gives back within `life::CROWD_RADIUS` blocks.
    pub regrown: f32,
    /// Food a day its grazers eat of it there, as many of each as live together (`crowd`, by how
    /// common it is there) and the grass's share of their diets.
    pub eaten: f32,
}

/// The tall grass's balance in a biome's best season for grass (`None`: no grass, or nothing
/// grazes it): the area within `life::CROWD_RADIUS` blocks, its grass cover
/// (`vitals::forage::grass_grows`), a grazed tuft growing back by the grazed grass's plant at the
/// season's rate (`vitals::plants`) and feeding as a mouthful of fresh grass does then.
pub fn grazing(biome: &str) -> Option<Grazing> {
    let c = climate::of(biome)?;
    let cover = vitals::forage::grass_grows(biome);
    let plant = vitals::plants::of("base:grazed_grass")?;
    let tuft = vitals::forage::SOURCES
        .iter()
        .find(|s| s.block == "base:tall_grass" && s.category == Category::Grass)?;
    let secs = vitals::plants::step_secs(plant, 1)?;
    let grazers: Vec<(&Species, f32)> = SPECIES
        .iter()
        .filter_map(|s| {
            let share = s
                .eats
                .iter()
                .find(|&&(k, _)| k == Category::Grass)
                .map(|&(_, v)| v)?;
            let weight = s
                .habitats
                .iter()
                .find(|&&(b, _)| b == biome)
                .map(|&(_, w)| w)?;
            Some((s, share * weight))
        })
        .collect();
    if cover <= 0.0 || grazers.is_empty() {
        return None;
    }
    let (season, rate) = (0..4)
        .map(|i| (i, vitals::plants::season_rate_in(plant, c.celsius, c.seasons, c.rain, i)))
        .fold((0, 0.0f32), |a, b| if b.1 > a.1 { b } else { a });
    let blocks = std::f32::consts::PI * (crate::life::CROWD_RADIUS as f32).powi(2) * cover;
    let mouthful = tuft.food as f32 * (0.4 + 0.6 * rate);
    let regrown = blocks * mouthful * rate * vitals::DAY_SECS / secs;
    let eaten = grazers
        .iter()
        .map(|&(s, share)| s.crowd() as f32 * share * vitals::DAY_SECS / s.hunger.max(1.0))
        .sum();
    Some(Grazing { season, regrown, eaten })
}

/// The gaps not in `WAITING`, and the `WAITING` entries no longer gaps: what the test asks to be
/// empty.
pub fn problems() -> Vec<String> {
    let found = gaps();
    let mut out: Vec<String> = found
        .iter()
        .filter(|g| !WAITING.iter().any(|&(k, _)| k == g.key))
        .map(|g| g.text.clone())
        .collect();
    for &(key, why) in WAITING {
        if !found.iter().any(|g| g.key == key) {
            out.push(format!("web::WAITING lists \"{key}\" (waiting for {why}), but it is no longer a gap: remove it"));
        }
    }
    out
}
