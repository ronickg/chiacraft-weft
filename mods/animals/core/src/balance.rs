//! Whether a species' numbers are in balance with the rest (`notes/ANIMALS.md`, "Balance"): what
//! they come to for a player (days to breed a line four generations deep, young a mother has in ten
//! days and in her life), each against a target for its rarity. The website's tests run these on
//! every species (`web/test.mjs`), the lab shows them (its balance table), so a new species whose
//! facts or `Tune` throw it out of line is caught before it reaches the game.

use crate::biology::Rarity;
use crate::life::{self, DAY};
use crate::species::Species;

/// What a species' numbers come to, in game days.
#[derive(Clone, Copy, Debug)]
pub struct Report {
    /// Mating to a grown young one that can breed itself (carrying, hatching, growing up).
    pub generation: f64,
    /// From a first pair to their great-grandchildren (three generations on).
    pub fourth: f64,
    /// Young a mother has in ten days at her fastest.
    pub young_per_ten: f64,
    /// Young an average mother has in her breeding life (grown up to elderly).
    pub lifetime_young: f64,
}

pub fn report(s: &Species) -> Report {
    let day = |t: u32| t as f64 / DAY as f64;
    // Eggs (a fish's roe too) hatch after their incubation; live young have none.
    let hatching = if s.gestation() > 0 { 0 } else { s.incubation() };
    let generation = day(life::carrying(s)) + day(hatching) + day(s.growth());
    let cycle = day(life::carrying(s) + s.recovery()).max(0.01);
    let litter = (s.litter().0 + s.litter().1) as f64 / 2.0;
    let breeding = (day(s.lifespan()) * life::ELDERLY - day(s.growth())).max(0.0);
    Report { generation, fourth: generation * 3.0, young_per_ten: 10.0 / cycle * litter, lifetime_young: breeding / cycle * litter }
}

/// Days a line may take to reach a fourth generation, by rarity: a pet's an afternoon's play or
/// two (an in-game day is twenty minutes), a dragon's a long project.
pub fn fourth_target(r: Rarity) -> (f64, f64) {
    match r {
        Rarity::Common => (10.0, 25.0),
        Rarity::Uncommon => (20.0, 40.0),
        Rarity::Rare => (35.0, 70.0),
        Rarity::Legendary => (80.0, 160.0),
    }
}

/// A check passed or failed, with what it found.
#[derive(Clone, Debug)]
pub struct Check {
    pub name: &'static str,
    pub ok: bool,
    pub found: String,
    pub target: String,
}

/// Every balance check on a species.
pub fn checks(s: &Species) -> Vec<Check> {
    let r = report(s);
    let (lo, hi) = fourth_target(s.bio.rarity);
    let day = |t: u32| t as f64 / DAY as f64;
    let (grow, live) = (day(s.growth()), day(s.lifespan()));
    let (few, most) = s.litter();
    vec![
        Check {
            name: "fourth generation",
            ok: (lo..=hi).contains(&r.fourth),
            found: format!("{:.1} days", r.fourth),
            target: format!("{lo}-{hi} days ({})", s.bio.rarity.name()),
        },
        // Most of a life grown: a young one that is a child half its life is no pet to keep.
        Check {
            name: "grown most of its life",
            ok: grow < live / 5.0,
            found: format!("grows {grow:.1} of {live:.0} days"),
            target: "under a fifth".into(),
        },
        // Enough young that the wild can replace its dead (`life::want_chance` takes some), not
        // so many that a pen floods.
        Check {
            name: "young in a life",
            ok: (6.0..=40.0).contains(&r.lifetime_young),
            found: format!("{:.1}", r.lifetime_young),
            target: "6-40".into(),
        },
        Check {
            name: "litter",
            ok: few >= 1 && few <= most && most <= 12,
            found: format!("{few}-{most}"),
            target: "1-12, fewest first".into(),
        },
        Check { name: "room near", ok: (3..=30).contains(&s.crowd()), found: s.crowd().to_string(), target: "3-30".into() },
    ]
}
