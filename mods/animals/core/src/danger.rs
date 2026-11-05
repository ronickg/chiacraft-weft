//! How dangerous each species is, declared and derived (`notes/PROGRESSION.md`, "The balance
//! model"; `reports/Balance model.md`, 4.1 and 4.9): a species states its role (its size class
//! comes from its mass) and its home age; its health, armour and hit come from `vitals::combat`'s
//! bands on the curve, times its level, times bounded tweaks with reasons.
//!
//! Ronald's answers (11-06):
//! - "animals should prob have 0-200 acrso the board ... but ofc a dragon should be super strong
//!   vs othernanim,asls": every wild animal rolls a level of 0-200 anywhere; the species sets the
//!   base through size class and role; a level-200 one is about 5× a level-0 one of its kind.
//! - Tames go into later ages on saddles and gear, capped like gear.
//!
//! The game reads them (11-07, Ronald: "Yes, all of them"): each kind registers `body`'s health
//! and hit, its wild points step by `LEVEL_STEP`, and its armour is a step of the damage chain
//! (`mods/animals/src/armour.rs`).

use vitals::combat::{self, Player, Role, Size, Stats};
use vitals::curve::{Age, GRADE_CEILING, Tweak, levels_for};

use crate::species::{DAMAGE, HEALTH, SPECIES, Species};

/// Wild levels, everywhere (Ronald: "0-200 acrso the board").
pub const LEVELS: (u32, u32) = (0, 200);
/// A top level's strength against a level-0 one of the same species.
pub const TOP_TIMES: f32 = 5.0;

/// How much stronger one at `level` is than a level-0 one of its kind: straight up to
/// `TOP_TIMES` at the top, as the game's wild points add (each a share of the base, `LEVEL_STEP`).
pub fn level_times(level: u32) -> f32 {
    1.0 + (TOP_TIMES - 1.0) * level.min(LEVELS.1) as f32 / LEVELS.1 as f32
}

/// What a wild point in health or damage adds, as a share of the base: a level's points fall on
/// the nine stats alike, so a level-200 one has about 22 in each, and `TOP_TIMES` its base.
pub const LEVEL_STEP: f32 = (TOP_TIMES - 1.0) * crate::species::STATS as f32 / LEVELS.1 as f32;

/// Area levels a creature's level adds over its age's start: 0 at level 0, about 34 (three and a
/// half ages) at 200.
pub fn level_shift(level: u32) -> f32 {
    levels_for(level_times(level))
}

/// The creature level at which a species of `home` meets an age's start (a Stone Age species
/// reaches the Bronze Age's band at level 30, the Iron Age's at 79, Steam's at 155).
pub fn level_reaching(home: Age, age: Age) -> Option<u32> {
    (LEVELS.0..=LEVELS.1).find(|&lv| home.start() + level_shift(lv) >= age.start() - 1e-4)
}

/// Inside the small class (everything under 10 kg), health and bite go with mass: a fox at its
/// middle, a mouse a fifth (Ronald, 11-07: "a fox beats a mouse"). As the cube root of mass (a
/// body's length), kept between `SMALL_TIMES` so the class's band holds.
pub const SMALL_MASS: f32 = 5.0;
pub const SMALL_TIMES: (f32, f32) = (0.2, 1.3);

/// A small one's share of its class's numbers, by its mass (1 for the other classes).
pub fn mass_times(s: &Species) -> f32 {
    if size(s) == Size::Small {
        (s.bio.mass / SMALL_MASS)
            .cbrt()
            .clamp(SMALL_TIMES.0, SMALL_TIMES.1)
    } else {
        1.0
    }
}

/// Which derived stat a tweak moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    Health,
    Armour,
    Hit,
}

impl Stat {
    pub const fn name(self) -> &'static str {
        match self {
            Stat::Health => "health",
            Stat::Armour => "armour",
            Stat::Hit => "hit",
        }
    }
}

/// What a species declares for its danger: the rest is derived.
#[derive(Clone, Copy, Debug)]
pub struct Danger {
    pub species: &'static str,
    pub role: Role,
    /// The age its level 0 sits at: every wild animal today is the Stone Age's, and levels carry
    /// it up; later ages' and dimensions' creatures declare theirs.
    pub home: Age,
    pub tweaks: &'static [(Stat, Tweak)],
}

const fn d(species: &'static str, role: Role) -> Danger {
    Danger { species, role, home: Age::Stone, tweaks: &[] }
}

use Role::{Apex, Grazer, Hunter, Prey, Tank};

/// Every species' role, in the species table's order (a check fails when one is missing). Prey
/// flee; grazers stand by their herd; hunters hunt; apex hunters are the top of their place; tanks
/// are armoured.
pub static DANGER: [Danger; 46] = [
    d("dog", Hunter),
    d("cat", Hunter),
    d("capybara", Prey),
    Danger { tweaks: &[(Stat::Health, Tweak::new(0.85, "hunts in packs of three to five; one alone is softer"))], ..d("raptor", Hunter) },
    Danger {
        tweaks: &[(Stat::Health, Tweak::new(0.85, "hunts in packs (the Stikes Quarry group); one alone is softer"))],
        ..d("utahraptor", Hunter)
    },
    Danger {
        tweaks: &[(Stat::Health, Tweak::new(0.85, "hunts in packs of three to five; one alone is softer"))],
        ..d("osmolskae", Hunter)
    },
    d("wyvern", Apex),
    d("hare", Prey),
    d("mountain_hare", Prey),
    d("wood_mouse", Prey),
    d("field_vole", Prey),
    d("white_tailed_deer", Prey),
    d("red_deer", Prey),
    d("brown_trout", Prey),
    d("pacu", Prey),
    d("red_fox", Hunter),
    d("arctic_fox", Hunter),
    d("raven", Prey),
    d("protoceratops", Grazer),
    d("kryptobaatar", Prey),
    d("priscagama", Prey),
    d("reindeer", Grazer),
    d("coyote", Hunter),
    d("house_sparrow", Prey),
    d("snow_bunting", Prey),
    d("great_horned_owl", Hunter),
    d("snowy_owl", Hunter),
    d("golden_eagle", Hunter),
    d("savanna_hawk", Hunter),
    d("turkey_vulture", Prey),
    d("herring_gull", Prey),
    d("yacare_caiman", Hunter),
    d("spectacled_caiman", Hunter),
    d("grey_wolf", Hunter),
    d("jaguar", Apex),
    d("lungfish", Prey),
    d("lycoptera", Prey),
    d("psittacosaurus", Grazer),
    d("gastonia", Tank),
    d("pinacosaurus", Tank),
    d("iguanodont", Grazer),
    d("crocodyliform", Hunter),
    d("citipati", Grazer),
    d("confuciusornis", Prey),
    d("gobipteryx", Prey),
    Danger {
        tweaks: &[(Stat::Hit, Tweak::new(1.2, "a tail and a stamp: a sauropod's defence is its weight"))],
        ..d("cedarosaurus", Grazer)
    },
];

/// A species' danger row.
pub fn danger(species: &str) -> Option<&'static Danger> {
    DANGER.iter().find(|d| d.species == species)
}

/// Its size class, from its mass.
pub fn size(s: &Species) -> Size {
    Size::of_mass(s.bio.mass)
}

impl Danger {
    /// Its tweak on `stat` (1 if none).
    pub fn tweak(&self, stat: Stat) -> f32 {
        self.tweaks
            .iter()
            .filter(|(s, _)| *s == stat)
            .map(|(_, t)| t.x)
            .product()
    }

    /// The area level one at `level` fights at.
    pub fn area_level(&self, level: u32) -> f32 {
        self.home.start() + level_shift(level)
    }
}

/// A wild one of `s` at `level`: derived, then tweaked.
pub fn stats(s: &Species, level: u32) -> Option<Stats> {
    let d = danger(s.name)?;
    let mut st = combat::derive(size(s), d.role, d.area_level(level));
    let m = mass_times(s);
    st.health *= d.tweak(Stat::Health) * m;
    st.armour *= d.tweak(Stat::Armour);
    st.hit *= d.tweak(Stat::Hit) * m;
    Some(st)
}

/// A level-0 one's numbers, what the game registers its kind with (its level's points multiply
/// them, `LEVEL_STEP`); a species with no row keeps its table's.
pub fn body(s: &Species) -> Stats {
    stats(s, 0).unwrap_or(Stats { health: s.base[HEALTH], armour: 0.0, hit: s.base[DAMAGE], interval: 1.0 })
}

// ------------------------------------------------------------------ bosses

/// The biome bosses (`crate::bosses`): titans whatever their mass, at their form's age.
pub fn boss_size() -> Size {
    crate::bosses::SIZE
}

/// A boss's form at `age` (its last past its last): its area level and numbers, derived like
/// every creature's and tweaked within the range.
pub fn boss(name: &str, age: Age) -> Option<(f32, Stats)> {
    let b = crate::bosses::boss(name)?;
    let f = b.form(age);
    Some((b.level(f), b.stats(f)))
}

// ------------------------------------------------------------------ tames

/// A tame's own bonuses together (tamed points, imprint, bond, Ark's mutations among them) add at
/// most this over a clean wild one of its level: the same ceiling gear has for grade and marks.
pub const TAME_CEILING: f32 = GRADE_CEILING;
/// Imprint (as decided) and a bond's share.
pub const IMPRINT: f32 = 0.2;
pub const BOND: f32 = 0.05;

/// A tame of `s` at `level`, on a saddle (and barding) of `saddle`'s age if any, with `bonus`
/// (its tamed points, imprint and bond, as a multiplier). The saddle carries it up to its age's
/// reference level, never past it, and never below where it already is (Ronald, 11-06: tames go
/// into later ages on saddles and gear, capped like gear); the bonus is capped at `TAME_CEILING`.
/// Its hits still go through the target's over-tier cap and rate floor (`combat::fight`), so a
/// pack of star-age tames can't melt a titan either.
pub fn tame(s: &Species, level: u32, saddle: Option<Age>, bonus: f32) -> Option<Stats> {
    let d = danger(s.name)?;
    let own = d.area_level(level);
    let at = saddle.map_or(own, |a| own.max(a.reference()));
    let times = bonus.clamp(1.0, TAME_CEILING);
    let m = mass_times(s);
    let mut st = combat::derive(size(s), d.role, at);
    st.health *= d.tweak(Stat::Health) * times * m;
    st.armour *= d.tweak(Stat::Armour) * times;
    st.hit *= d.tweak(Stat::Hit) * times * m;
    Some(st)
}

/// The level a tame fights at, as a player of it would: what its saddle carried it to.
pub fn tame_level(s: &Species, level: u32, saddle: Option<Age>) -> Option<f32> {
    let own = danger(s.name)?.area_level(level);
    Some(saddle.map_or(own, |a| own.max(a.reference())))
}

/// A same-age reference fight against one of `s` at `level`, by a player of `age` with all their
/// gear's numbers `times` (1: grade 4, `GRADE_CEILING`: a perfect one).
pub fn fight(s: &Species, level: u32, age: Age, times: f32) -> Option<combat::Fight> {
    let d = danger(s.name)?;
    let them = stats(s, level)?;
    Some(combat::fight(Player { level: age.reference(), times }, size(s), d.area_level(level), them))
}

/// Every species, with its danger row (the check names those without one).
pub fn rows() -> impl Iterator<Item = (&'static Species, Option<&'static Danger>)> {
    SPECIES.iter().map(|s| (s, danger(s.name)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_species_has_a_role() {
        for (s, d) in rows() {
            assert!(d.is_some(), "{} has no danger row", s.name);
        }
        for d in &DANGER {
            assert!(SPECIES.iter().any(|s| s.name == d.species), "{} is no species", d.species);
            for (_, t) in d.tweaks {
                assert!(t.ok(), "{}: {t:?}", d.species);
            }
        }
    }

    #[test]
    fn a_top_level_is_five_times_a_bottom_one() {
        let s = SPECIES.iter().find(|s| s.name == "grey_wolf").unwrap();
        let (lo, hi) = (stats(s, 0).unwrap(), stats(s, 200).unwrap());
        assert!((hi.health / lo.health - TOP_TIMES).abs() < 1e-3);
        assert!((level_times(100) - 3.0).abs() < 1e-5);
        assert!((LEVEL_STEP - 0.18).abs() < 1e-5);
        assert_eq!(level_reaching(Age::Stone, Age::Bronze), Some(30));
        assert_eq!(level_reaching(Age::Stone, Age::Lightning), None);
    }

    #[test]
    fn a_fox_beats_a_mouse() {
        let of = |n: &str| SPECIES.iter().find(|s| s.name == n).unwrap();
        let (fox, mouse) = (body(of("red_fox")), body(of("wood_mouse")));
        assert!(fox.health > 3.0 * mouse.health && fox.hit > 3.0 * mouse.hit);
    }

    #[test]
    fn a_saddle_carries_a_tame_up_but_no_further() {
        let s = SPECIES.iter().find(|s| s.name == "utahraptor").unwrap();
        let own = tame_level(s, 0, None).unwrap();
        assert_eq!(tame_level(s, 0, Some(Age::Iron)).unwrap(), Age::Iron.reference());
        assert_eq!(tame_level(s, 200, Some(Age::Bronze)).unwrap(), own + level_shift(200));
        let capped = tame(s, 0, None, 9.0).unwrap().health / tame(s, 0, None, 1.0).unwrap().health;
        assert!((capped - TAME_CEILING).abs() < 1e-4);
    }
}
