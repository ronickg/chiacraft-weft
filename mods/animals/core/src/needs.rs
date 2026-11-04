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
pub const STARVES_SECS: f32 = 1200.0;

/// The prey of `hunter` that live in `biome`: species of other families it takes alone, by their
/// grown masses (farm animals aren't here: they don't come and go with the seasons).
pub fn prey_in<'a>(hunter: &'a Species, biome: &'a str) -> impl Iterator<Item = &'static Species> + 'a {
    SPECIES.iter().filter(move |o| {
        o.family.name != hunter.family.name && hunter.takes(hunter.bio.mass, o.bio.mass) && o.habitats.iter().any(|&(b, _)| b == biome)
    })
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
pub const DAY_SECS: f32 = 600.0;
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
/// can spare (`SPARE`) over what it must drink a day, on the hunger meter's clock (`DAY_SECS`).
/// `None`: it never needs to drink.
pub fn drink_lasts(s: &Species, kg: f32) -> Option<f32> {
    let drunk = litres_a_day(s, kg) * drinks(s);
    (drunk > 0.0).then(|| SPARE * kg / drunk * DAY_SECS)
}
