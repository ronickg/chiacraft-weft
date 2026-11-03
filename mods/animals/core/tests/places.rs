//! Checks across systems (`notes/SYSTEMS.md`): the species' facts (habitats, comfort, breeding
//! seasons, eggs) against the world's climates and calendar (`vitals::climate`,
//! `vitals::calendar`) and the body's rules (`vitals::temperature`). Each failure says what
//! broke and which numbers to look at.

use animals_core::life;
use animals_core::species::{SPECIES, Species};
use vitals::calendar::SEASON_DAYS;
use vitals::climate::{self, Climate};
use vitals::temperature::{Body, Feeling, feeling};

/// The middle day of season `index` (0 spring), and each of its hours.
fn day_hours(index: u32) -> impl Iterator<Item = (f32, f32)> {
    let day = (index * SEASON_DAYS + SEASON_DAYS / 2) as f32;
    (0..24).map(move |h| (day, h as f32 + 0.5))
}

/// Share of the hours of the middle days of `seasons` (bits, spring first) that `bad` is true.
fn share(c: &Climate, seasons: u8, bad: impl Fn(f32) -> bool) -> f32 {
    let hours: Vec<f32> = (0..4)
        .filter(|s| seasons >> s & 1 != 0)
        .flat_map(day_hours)
        .map(|(day, hour)| climate::air(c, day, hour))
        .collect();
    hours.iter().filter(|&&a| bad(a)).count() as f32 / hours.len().max(1) as f32
}

/// Its body, fed (as wild ones are) and with no points in fortitude.
fn body(s: &Species) -> Body {
    Body { comfort: s.bio.comfort, fat: 0.5, ..Body::default() }
}

fn uneasy(body: &Body, air: f32) -> bool {
    matches!(feeling(body.target(air)), Feeling::Cold | Feeling::Freezing | Feeling::Hot | Feeling::Heatstroke)
}

fn report(problems: Vec<String>) {
    assert!(problems.is_empty(), "\n{}\n", problems.join("\n"));
}

#[test]
fn every_habitat_has_a_climate() {
    let mut problems = Vec::new();
    for s in &SPECIES {
        for (biome, _) in s.habitats {
            if climate::of(biome).is_none() {
                problems.push(format!(
                    "{}: lives in {biome}, which has no climate in vitals::climate::BIOMES (add its row; its mod sets it with base_api::climate)",
                    s.name
                ));
            }
        }
    }
    report(problems);
}

/// A wild animal is mostly at ease where it lives: cold or hot (seeking comfort, a tame eating
/// faster) at most a quarter of the year's hours in its commonest habitat, half in the others.
#[test]
fn each_species_is_at_home_in_its_habitats() {
    let mut problems = Vec::new();
    for s in SPECIES.iter().filter(|s| !s.bio.cold_blooded) {
        let b = body(s);
        for &(biome, weight) in s.habitats {
            let Some(c) = climate::of(biome) else { continue };
            let limit = if weight >= 1.0 { 0.25 } else { 0.5 };
            let off = share(c, 0b1111, |a| uneasy(&b, a));
            if off > limit {
                problems.push(format!(
                    "{}: cold or hot {:.0}% of the year in {biome} (at most {:.0}%): its comfort {:?} (species.rs) against the biome's climate (vitals::climate)",
                    s.name,
                    off * 100.0,
                    limit * 100.0,
                    s.bio.comfort
                ));
            }
        }
    }
    report(problems);
}

/// A species that lays has eggs that develop in a nest in each of its habitats for at least half
/// of the day in its likeliest breeding season (every breeding season without one).
#[test]
fn eggs_hatch_where_and_when_they_are_laid() {
    let mut problems = Vec::new();
    for s in SPECIES.iter().filter(|s| s.gestation() == 0) {
        let when = if s.bio.peak != 0 { s.bio.peak } else { s.bio.breeds };
        for season in (0..4).filter(|i| when >> i & 1 != 0) {
            for &(biome, _) in s.habitats {
                let Some(c) = climate::of(biome) else { continue };
                let cold = share(c, 1 << season, |a| !life::develops(life::egg_air(a, true)));
                if cold > 0.5 {
                    problems.push(format!(
                        "{}: eggs in a nest in {biome} develop only {:.0}% of the day in season {season} (its breeding season, biology.rs): life::EGG_AIR {:?} against the biome's climate",
                        s.name,
                        (1.0 - cold) * 100.0,
                        life::EGG_AIR
                    ));
                }
            }
        }
    }
    report(problems);
}

/// Players and animals share the nine stats by position (`[_; STATS]` tables on both sides).
#[test]
fn the_stats_are_the_same_nine() {
    assert_eq!(
        animals_core::species::STAT_NAMES,
        vitals::stats::NAMES,
        "the animals' stats (species.rs STAT_NAMES) and the player's (vitals::stats::NAMES) must be the same, in the same order"
    );
}

/// Crops grow by the pace the species grow up by (`notes/ECOLOGY.md`, "The pacing"), so a field
/// and a litter keep their real proportions.
#[test]
fn plants_and_animals_grow_at_one_pace() {
    let (a, k) = animals_core::biology::GROWING;
    assert_eq!(
        (a as f32, k as f32),
        vitals::plants::GROWING,
        "vitals::plants::GROWING and animals_core::biology::GROWING must be the same pace"
    );
}

/// A plant eater that forages in the wild finds its food in each of its habitats for at least
/// half the year: some food of its diet grows back there at a third of its best or more, or
/// browse is found there (`vitals::forage::season_lushness`), in two seasons or more.
#[test]
fn plant_eaters_find_food_where_they_live() {
    let mut problems = Vec::new();
    for s in SPECIES.iter().filter(|s| s.forages_wild()) {
        for &(biome, _) in s.habitats {
            let Some(c) = climate::of(biome) else { continue };
            let seasons = (0..4)
                .filter(|&season| {
                    vitals::forage::season_lushness(|k| s.eats(k), biome, c.celsius, c.seasons, c.rain, Some(season)).unwrap_or(0.0)
                        >= 1.0 / 3.0
                })
                .count();
            if seasons < 2 {
                problems.push(format!(
                    "{}: its food grows in {biome} in only {seasons} season(s): its diet (species.rs eats) against vitals::forage and vitals::plants",
                    s.name
                ));
            }
        }
    }
    report(problems);
}
