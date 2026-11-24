//! What a wild animal needs, as the numbers every part reads (`notes/ECOLOGY.md`, parts 4-7):
//! how much food fills it, by mass, the same for a kill, a mouthful off the ground and a carcass;
//! how fed one arrives; how long it lasts with nothing.
//!
//! The mod's goals (`mods/animals/src/hunt.rs`, `forage.rs`) and the species sheet read them here.

use crate::species::{SPECIES, Species};

/// A meal of this share of its own mass fills it (its whole food stat): a fox's 550 g, so a hare
/// more than fills it and a mouse is a twentieth. Real hunters eat about this at a sitting (a red
/// fox takes about a tenth of its mass a day; a wolf up to a fifth at a kill), and a grazer's tenth
/// of fresh grass is its 2-3% of dry matter.
pub const MEAL: f32 = 0.1;

/// A wild one goes after food (grazing, `forage.rs`; hunting, `hunt.rs`) under this share of its
/// food, and stops when this full.
pub const HUNGRY: f32 = 0.6;
pub const FULL: f32 = 0.95;

/// The share of natural spawns that still come where nothing grows this season (`forage.rs`):
/// animals live through a winter or a drought, fewer and thinner.
pub const SPAWN_FLOOR: f32 = 0.2;

/// The share of its food stat that `kg` of food fills in an eater of `own` kg (at most all of it).
pub fn fills(own: f32, kg: f32) -> f32 {
    if own <= 0.0 {
        return 0.0;
    }
    (kg / (own * MEAL)).min(1.0)
}

/// Share of its food a wild one has when it arrives where nothing it eats grows this season (a
/// plant eater), or where its prey are thinnest (a hunter); the rest comes with the place's food.
pub const ARRIVES_FED: f32 = 0.3;

/// Seconds a wild one lives on an empty stomach at full health before it starves (a game day).
/// A scavenger (`Species::scavenges`) never starves in the wild: a turkey vulture fasts two
/// weeks, and the world's carrion is too patchy to hold its life to a game day.
pub const STARVES_SECS: f32 = 1200.0;

/// Blocks a hungry scavenger smells carrion from, from the air or the ground (a turkey vulture
/// finds a carcass hidden under a forest's canopy by its smell, Houston 1986); it flies there.
pub const SCENT: f64 = 64.0;

/// The prey of `hunter` that live in `biome`: species of other families it takes alone, by their
/// grown masses (farm animals aren't here: they don't come and go with the seasons).
pub fn prey_in<'a>(hunter: &'a Species, biome: &'a str) -> impl Iterator<Item = &'static Species> + 'a {
    SPECIES.iter().filter(move |o| {
        o.family.name != hunter.family.name && hunter.takes(hunter.bio.mass, o.bio.mass) && o.habitats.iter().any(|&(b, _)| b == biome)
    })
}

/// The share of its prey's young a year one hunter species may take where it lives
/// (`hunter_room`): real predators take 10-36% of a prey's numbers a year, paid by the herd's own
/// increase; the rest is the herd's, and other hunters'.
pub const HUNT_SHARE: f32 = 0.3;

/// A species' share of `biome` by its habitat weight (0 where it doesn't live).
pub fn habitat_weight(s: &Species, biome: &str) -> f32 {
    s.habitats
        .iter()
        .find(|&&(b, _)| b == biome)
        .map_or(0.0, |&(_, w)| w)
}

/// What one `hunter` takes of each of its prey in `biome` a game year, and that prey's numbers
/// near and young a year (at their stocked crowds, `Species::stocked_in`, by habitat weight). It
/// eats the meat share of its food a day (`Species::hunger`), shared among its prey by their
/// biomass, each kill filling it by the prey's mass (`fills`). Empty: no prey of its sizes there.
pub fn takes_a_year(hunter: &Species, biome: &str) -> Vec<(&'static Species, f32, f32, f32)> {
    let numbers = |s: &Species| s.stocked_in(biome) as f32 * habitat_weight(s, biome);
    let prey: Vec<&'static Species> = prey_in(hunter, biome)
        .filter(|p| numbers(p) > 0.0)
        .collect();
    let biomass: f32 = prey.iter().map(|p| numbers(p) * p.bio.mass).sum();
    if biomass <= 0.0 {
        return Vec::new();
    }
    let meat: f32 = hunter
        .eats
        .iter()
        .filter(|(c, _)| matches!(c, crate::species::Category::Meat | crate::species::Category::Fish))
        .map(|(_, v)| v)
        .sum();
    let year = vitals::calendar::YEAR_DAYS as f32;
    let food_day = vitals::DAY_SECS / hunter.hunger.max(1.0) * meat;
    let full = hunter.base[crate::species::FOOD].max(1.0);
    prey.into_iter()
        .map(|p| {
            let diet = numbers(p) * p.bio.mass / biomass;
            let kills = food_day * diet / (fills(hunter.bio.mass, p.bio.mass) * full) * year;
            let young = numbers(p) * 0.5 * crate::balance::report(p).young_per_ten as f32 / 10.0 * year;
            (p, kills, numbers(p), young)
        })
        .collect()
}

/// How many of `hunter` its prey in `biome` feed (`HUNT_SHARE` of each prey's young a year);
/// `None`: no prey of its sizes lives there.
pub fn hunter_room(hunter: &Species, biome: &str) -> Option<u32> {
    takes_a_year(hunter, biome)
        .into_iter()
        .filter(|&(_, kills, _, _)| kills > 0.0)
        .map(|(_, kills, _, young)| HUNT_SHARE * young / kills)
        .reduce(f32::min)
        .map(|n| n.floor() as u32)
}

/// How many of its biggest prey at home fill `hunter` (`fills`): one hunt, or several small
/// ones. `None`: no prey of its sizes lives where it does (it waits for them to come).
pub fn kills_to_fill(hunter: &Species) -> Option<f32> {
    let biggest = hunter
        .habitats
        .iter()
        .flat_map(|&(biome, _)| prey_in(hunter, biome).map(|o| o.bio.mass))
        .chain(
            crate::species::OTHER_PREY
                .iter()
                .filter(|_| hunter.habitats.iter().any(|&(b, _)| b.starts_with("base:")))
                .filter(|&&(_, kg)| hunter.takes(hunter.bio.mass, kg))
                .map(|&(_, kg)| kg),
        )
        .reduce(f32::max)?;
    Some(1.0 / fills(hunter.bio.mass, biggest))
}

// ---------------------------------------------------------------- water (part 4)

/// A mammal's water need a day in litres, `WATER.0 · kg^WATER.1` (`notes/ECOLOGY.md`, part 4): a
/// mouse's is a seventh of its mass a day, a deer's a fifteenth.
pub const WATER: (f32, f32) = (0.099, 0.9);
/// A cold-blooded one's water need against a mammal's of its mass (desert lizards' water flux in
/// the field is about a fifth of a mammal's, Nagy).
pub const COLD_BLOODED_WATER: f32 = 0.2;
/// The share of its mass in water it can go without before it is parched (a mammal is in danger
/// past 10-15% of its mass lost: the top of that, so a tame isn't always at its bowl).
pub const SPARE: f32 = 0.15;
/// Seconds of play a real day of drinking takes: the hunger meter's clock (a deer's food stat, a
/// meal of a tenth of its mass, is three days of its grazing and lasts it 30 minutes).
pub const DRINK_DAY_SECS: f32 = 600.0;
/// It goes to drink under this share of its water, and drinks until this full.
pub const THIRSTY: f32 = 0.5;
pub const SLAKED: f32 = 0.95;
/// A gulp: this share of its water, every `GULP_TICKS`.
pub const GULP: f32 = 0.25;
pub const GULP_TICKS: u32 = 30;
/// Seconds a wild one lives parched at full health before thirst kills it (as starving).
pub const PARCHED_SECS: f32 = STARVES_SECS;

/// The share of its water each species drinks; the rest comes from its food and its own body.
/// A species not here drinks all of it (a dog, the capybara by its rivers, the wyvern). From what's
/// known of each one or its nearest living kin: voles and desert rodents live on the water in
/// their food (kangaroo rats never drink), hares on greens and dew, foxes and cats get much of
/// theirs from prey, deer drink every day; the dinosaurs as their living kin (birds) and the
/// desert hunters they lived as. Fish live in it (`Species::in_water`).
pub const DRINKS: &[(&str, f32)] = &[
    ("cat", 0.5),
    ("raptor", 0.5),
    ("osmolskae", 0.5),
    ("utahraptor", 0.8),
    ("hare", 0.3),
    ("mountain_hare", 0.3),
    ("wood_mouse", 0.2),
    ("field_vole", 0.1),
    ("white_tailed_deer", 0.7),
    ("red_deer", 0.8),
    ("red_fox", 0.5),
    ("arctic_fox", 0.3),
    ("raven", 0.7),
    // A desert browser: most of its water from its plants, the rest at the wet season's ponds.
    ("protoceratops", 0.4),
    ("kryptobaatar", 0.1),
    ("priscagama", 0.1),
    // The red dunes' omnivore: its water from leaves, seeds and prey, as Protoceratops's.
    ("citipati", 0.4),
];

/// The share of its water `s` drinks (`DRINKS`; 0 for fish, which live in it).
pub fn drinks(s: &Species) -> f32 {
    if s.in_water() {
        return 0.0;
    }
    DRINKS.iter().find(|d| d.0 == s.name).map_or(1.0, |d| d.1)
}

/// Litres of water a day an animal of `s` weighing `kg` needs, drunk or not (`WATER`).
pub fn litres_a_day(s: &Species, kg: f32) -> f32 {
    WATER.0 * kg.max(0.001).powf(WATER.1) * if s.bio.cold_blooded { COLD_BLOODED_WATER } else { 1.0 }
}

/// Seconds a full drink lasts an animal of `s` weighing `kg`, from slaked to parched: the water it
/// can spare (`SPARE`) over what it must drink a day, on the hunger meter's clock (`DRINK_DAY_SECS`; not the calendar's day, `vitals::DAY_SECS`).
/// `None`: it never needs to drink.
pub fn drink_lasts(s: &Species, kg: f32) -> Option<f32> {
    let drunk = litres_a_day(s, kg) * drinks(s);
    (drunk > 0.0).then(|| SPARE * kg / drunk * DRINK_DAY_SECS)
}

// ------------------------------------------------------------------ choosing between needs

// What an animal does when nothing is after it (`reports/Creature AI.md`, part 3, step 3): each
// need scores itself 0 to 1 from how pressing it is, and the most pressing that has something to
// do goes first (`mods/animals/src/choose.rs`), not a fixed order. So a starving deer gets up to
// eat, and one parched leaves its mate to drink. Hunger and thirst grow as the square of how far
// past their start they are (Dave Mark's quadratic response curve): mild hunger waits for a rest
// to end, real hunger doesn't.

/// What a need scores when it is barely pressing (hunger just under `HUNGRY`, thirst just under
/// `THIRSTY`) and when it is all there is (empty, parched).
pub const EAT_FLOOR: f32 = 0.3;
pub const DRINK_FLOOR: f32 = 0.35;
/// A mate in its season, and a rest: what a hungry or thirsty one sets aside only once the need
/// presses (the square past about 0.25 and 0.36 of the way to empty).
pub const LOVE: f32 = 0.5;
pub const REST: f32 = 0.45;
/// The air: cold or hot, and freezing or in a heatstroke.
pub const UNEASY: f32 = 0.4;
pub const SUFFERING: f32 = 0.85;
/// Wandering, last of all.
pub const WANDER: f32 = 0.01;
/// What it is doing already counts this much more (Blumberg's hysteresis: it doesn't flicker
/// between two needs near the same).
pub const KEEP_AT: f32 = 1.2;

/// How pressing hunger is at `fed` (its share of its food): 0 when fed (`HUNGRY` or more and not
/// eating already), `EAT_FLOOR` just under, 1 empty.
pub fn hunger_score(fed: f32, eating: bool) -> f32 {
    need_score(fed, HUNGRY, EAT_FLOOR, eating)
}

/// `hunger_score` from `start` instead of `HUNGRY` (a herd member grazing with its leader,
/// `groups::JOIN_EAT`).
pub fn hunger_score_from(fed: f32, start: f32, eating: bool) -> f32 {
    need_score(fed, start.max(HUNGRY), EAT_FLOOR, eating)
}

/// How pressing thirst is at `water` (its share of its water): as hunger, from `THIRSTY`.
pub fn thirst_score(water: f32, drinking: bool) -> f32 {
    need_score(water, THIRSTY, DRINK_FLOOR, drinking)
}

/// `thirst_score` from `start` instead of `THIRSTY` (a plant eater's dawn and dusk drink,
/// `routines::DRINKS_AT`).
pub fn thirst_score_from(water: f32, start: f32, drinking: bool) -> f32 {
    need_score(water, start.max(THIRSTY), DRINK_FLOOR, drinking)
}

fn need_score(have: f32, start: f32, floor: f32, at_it: bool) -> f32 {
    if have >= start && !at_it {
        return 0.0;
    }
    let past = ((start - have) / start).clamp(0.0, 1.0);
    floor + (1.0 - floor) * past * past
}

#[cfg(test)]
mod choosing {
    use super::*;

    #[test]
    fn real_hunger_beats_a_rest_and_real_thirst_beats_love() {
        // Mild hunger waits for its rest; a starving one gets up (the rest held at `KEEP_AT`).
        assert!(hunger_score(0.5, false) < REST * KEEP_AT);
        assert!(hunger_score(0.1, false) > REST * KEEP_AT);
        // Thirst leaves a mate only once it presses.
        assert!(thirst_score(0.45, false) < LOVE);
        assert!(thirst_score(0.15, false) > LOVE);
        // Fed and watered, neither asks for the turn; a meal under way keeps going.
        assert_eq!(hunger_score(0.8, false), 0.0);
        assert!(hunger_score(0.8, true) > 0.0);
    }
}
