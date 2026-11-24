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

/// How long a wild one lives on an empty stomach at full health, on its body's reserves: a
/// mammal's fasting endurance grows with its mass to the power 0.44 (Lindstedt & Boyce 1985,
/// "Seasonality, fasting endurance, and body size in mammals"), so a deer lasts weeks of the
/// calendar where a mouse lasts days. `FAST_DAYS` game days at `FAST_MASS` kg (a white-tailed
/// deer: 6 game days, about 39 days of the 56-day year), `FAST_POWER` of the mass ratio, within
/// `FAST_RANGE` game days (a small one still a day; the biggest a season). A cold-blooded one
/// burns about a fifth of what a mammal of its mass does at rest (as it drinks, `COLD_BLOODED_WATER`)
/// and fasts `FAST_COLD_BLOODED` times as long: a caiman goes months without a meal.
/// A scavenger (`Species::scavenges`) never starves in the wild: a turkey vulture fasts two
/// weeks, and the world's carrion is too patchy to hold its life to its fasting.
pub const FAST_DAYS: f32 = 6.0;
pub const FAST_MASS: f32 = 70.0;
pub const FAST_POWER: f32 = 0.44;
pub const FAST_RANGE: (f32, f32) = (1.0, vitals::calendar::SEASON_DAYS as f32);
pub const FAST_COLD_BLOODED: f32 = 5.0;

/// Game days one of `mass` kg lives on an empty stomach (`FAST_DAYS`).
pub fn fast_days(mass: f32, cold_blooded: bool) -> f32 {
    let days = FAST_DAYS * (mass.max(0.001) / FAST_MASS).powf(FAST_POWER);
    (days * if cold_blooded { FAST_COLD_BLOODED } else { 1.0 }).clamp(FAST_RANGE.0, FAST_RANGE.1)
}

/// Game days `s` lives on an empty stomach at full health (`fast_days`).
pub fn fasts(s: &Species) -> f32 {
    fast_days(s.bio.mass, s.bio.cold_blooded)
}

/// Seconds `s` lives on an empty stomach at full health before it starves (`fasts`).
pub fn starves_secs(s: &Species) -> f32 {
    fasts(s) * vitals::DAY_SECS
}

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
/// Seconds a wild one lives parched at full health before thirst kills it: a game day, whatever
/// its size (water, unlike fat, isn't stored for long).
pub const PARCHED_SECS: f32 = vitals::DAY_SECS;

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
/// A rest gives way to a need that drives it (`memory::DRIVEN`, thirst or hunger): it scores at
/// most this share of that need, under it even while resting (`KEEP_AT`). A sleepy night's rest
/// (`REST` + `routines::SLEEPY`, held at `KEEP_AT`: 0.96) outscored thirst until a deer was all
/// but dry: one lay a whole game day 33 blocks from water while its water fell from 69 % to 8 %,
/// and died parched (ai1124_savanna8).
pub const REST_YIELDS: f32 = 0.75;
const _: () = assert!(REST_YIELDS * KEEP_AT < 1.0);

/// A rest that scores `rest` against the most pressing of thirst and hunger, `pressing`: as it
/// is, or under `pressing` once that drives it (`REST_YIELDS`).
pub fn rest_under(rest: f32, pressing: f32) -> f32 {
    if pressing >= crate::memory::DRIVEN { rest.min(pressing * REST_YIELDS) } else { rest }
}

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

    #[test]
    fn a_sleepy_rest_gives_way_to_real_thirst() {
        let night = REST + crate::routines::SLEEPY;
        // Mild thirst waits for the morning; real thirst gets up, even from a rest under way.
        assert_eq!(rest_under(night, thirst_score(0.4, false)), night);
        let parched = thirst_score(0.1, false);
        assert!(rest_under(night, parched) * KEEP_AT < parched);
        assert!(rest_under(night, hunger_score(0.05, false)) * KEEP_AT < hunger_score(0.05, false));
    }
}

// ------------------------------------------------------------------ stuck, and the way out

// An animal whose walks to food or water end where they began is stuck: in a cave under the
// water it wants (the path can't find the way round that far), or a pit deeper than it steps
// up. Each walk that ended short gave the place up and handed the turn to a rest, so it lay down,
// got up and lay down again on one block till it died (ai1124_graze9, graze10). It looks for a
// way out: legs along its own level, a different way each time; and from a pit those can't leave,
// it leaps, as a deer clears a fence of 2.4 m.

/// Walks ended short within `STUCK_NEAR` blocks of where the first did before it looks for a way
/// out, and those before it leaps.
pub const STUCK_AFTER: u8 = 2;
pub const LEAP_AFTER: u8 = 6;
pub const STUCK_NEAR: f64 = 4.0;
/// A walk to food or water whose time is up (`forage::WALK`, `drink::WALK`) has failed if it
/// moved less than this from where it set off, whether or not the engine still walks it: herd
/// mates jostling in a pit kept each other 'moving', so the engine's own stuck check (no move for
/// 2 s) never ended their walks and none was ever counted (ai1124_pit4b, cave2).
pub const WALK_MOVED: f64 = 2.0;
/// A leg out: blocks along its own level, the ways (of 8, turned on each try), and the game
/// seconds it has to walk it.
pub const WAY_OUT_LEG: f64 = 12.0;
pub const WAY_OUT_WAYS: u8 = 8;
pub const WAY_OUT_SECS: f32 = 10.0;
/// The way out: before a leg along its level, a stuck animal searches the ground it can walk or
/// leap to from where it stands (stepping up one, dropping at most `FLEE_DROP` + 2), up to
/// `WAY_UP_CELLS` places within `WAY_UP_REACH` blocks, for the place nearest the food or water it
/// couldn't reach (with none known, the highest), and walks `WAY_OUT_LEG` steps of that way. The engine's paths search 600 places toward the goal and stop at the nearest, so from a
/// cave under a pond the best they find is under the pond (ai1124_cave3b: deer 10 blocks under
/// it walked legs along the cave floor and leapt for 160 s, and died of thirst there).
pub const WAY_UP_CELLS: usize = 4000;
pub const WAY_UP_REACH: i32 = 40;
/// A way out toward food or water it couldn't reach must end this many blocks nearer to it.
pub const WAY_CLOSER: f64 = 2.0;
/// A leap: onto ground at most `LEAP_MOST` blocks above its feet (a walker's jump clears 1.25; a
/// deer clears a fence of 2.4 m) and `LEAP_REACH` off, over the edge by `LEAP_CLEAR`; the
/// world's pull (the engine's dimension gravity, blocks a second squared) and a share more for
/// the air's drag. With no such ground near it leaps `LEAP_MOST` up along the way it tried.
pub const LEAP_MOST: i32 = 2;
pub const LEAP_REACH: i32 = 3;
pub const LEAP_CLEAR: f64 = 0.7;
pub const GRAVITY: f64 = 32.0;
pub const LEAP_DRAG: f64 = 1.15;
/// A leap goes straight up and across only once its feet are over the ledge (`stuck::going`), at
/// this many blocks a second for each block still to go: pushed across from the start, the air's
/// drag (vanilla's, keeping 0.91 of it a tick) stopped it short and against the wall it rose
/// along, and it fell back in (ai1124_pit5e: twenty leaps 1 across and 2 up, none out).
pub const LEAP_OVER: f64 = 7.0;
/// Leaps from one stuck place that leave it there before it stops leaping (till it is somewhere
/// else): a deer on level ground leapt 70 times at a rim it could step onto (ai1124_savanna17).
pub const LEAP_TRIES: u8 = 3;
/// Shut in: the ground it can walk to from where it stands is fewer places than this (a pit 3 x 3
/// is 9, a shaft's foot 3). Only then does a stuck animal look for a way out and leap
/// (`stuck::enclosed`); on open ground it gives the place up and wanders on.
pub const TRAP_CELLS: usize = 48;
/// Game ticks its drink or eat goal may go unrun before a walk it had set off on counts as cut
/// short by another goal, not by the ground (`stuck::ran`): goals run every tick while they have
/// the turn.
pub const RAN_GAP: f64 = 10.0;
/// The steepest drop a running animal takes (`senses::escape`): it can step back up one, not two.
pub const FLEE_DROP: i32 = 1;
const _: () = assert!(HEAL_AFTER > 1.0);
const _: () = assert!(STUCK_AFTER < LEAP_AFTER && LEAP_AFTER - STUCK_AFTER <= WAY_OUT_WAYS);

/// Healing: an animal watered (its water at `THIRSTY` or more) and unhurt for `HEAL_AFTER`
/// seconds heals its whole health in `HEAL_DAYS` game days while fed (its food at `HUNGRY` of its
/// most or more), slower as it grows thin, not at all at `HEAL_THIN` (as thin as a wild one
/// arrives where nothing grows, `ARRIVES_FED`): a thin animal heals too, only slower. Without it a
/// deer bitten to half by a wolf or a fall stayed at half for good, and the next blow killed it;
/// with only the fed healing, a winter's deer grazing between 55 and 95 % healed part of the time.
pub const HEAL_DAYS: f32 = 1.0;
pub const HEAL_AFTER: f32 = 10.0;
pub const HEAL_THIN: f32 = ARRIVES_FED;
const _: () = assert!(HEAL_THIN < HUNGRY);

/// The share of its full healing (`heal_per_sec`) an animal at `fed` (its share of its food) and
/// `water` (its share of its water), last hurt `since` seconds ago, heals at. A starving or
/// parched one is hurt every second (`starves_secs`, `PARCHED_SECS`) and empty besides, so it
/// never heals what it loses.
pub fn heal_share(fed: f32, water: f32, since: f32) -> f32 {
    if water < THIRSTY || since < HEAL_AFTER {
        return 0.0;
    }
    ((fed - HEAL_THIN) / (HUNGRY - HEAL_THIN)).clamp(0.0, 1.0)
}

/// Health a second an animal of `max` health heals.
pub fn heal_per_sec(max: f32) -> f32 {
    max / (HEAL_DAYS * vitals::DAY_SECS)
}

/// The push (blocks a second) of a leap that lands `dx, dz` off and `dy` up: up fast enough to
/// clear the edge by `LEAP_CLEAR`, and across in the time it is in the air.
pub fn leap(dx: f64, dy: f64, dz: f64) -> [f64; 3] {
    let rise = dy.max(0.0) + LEAP_CLEAR;
    let up = (2.0 * GRAVITY * rise).sqrt();
    let air = up / GRAVITY + (2.0 * LEAP_CLEAR / GRAVITY).sqrt();
    [dx / air * LEAP_DRAG, up * LEAP_DRAG, dz / air * LEAP_DRAG]
}

#[cfg(test)]
mod leaping {
    use super::*;

    #[test]
    fn a_leap_out_of_a_pit_clears_its_rim() {
        // Out of a pit 2 deep, 1.5 blocks to the rim: up past 2.7 blocks, as a deer's fence.
        let [x, up, _] = leap(1.5, 2.0, 0.0);
        assert!(up * up / (2.0 * GRAVITY) > 2.0 + LEAP_CLEAR);
        assert!(x > 0.0 && x < up);
    }
}

#[cfg(test)]
mod healing {
    use super::*;

    #[test]
    fn a_fed_unhurt_animal_heals_in_a_day_a_thin_one_slower() {
        // Half its health back in half a day, the whole in a day.
        let max = 20.0;
        assert!((heal_per_sec(max) * vitals::DAY_SECS * HEAL_DAYS - max).abs() < 1e-3);
        assert_eq!(heal_share(FULL, 1.0, HEAL_AFTER), 1.0);
        assert_eq!(heal_share(HUNGRY, 1.0, 60.0), 1.0);
        assert_eq!(heal_share(FULL, 1.0, HEAL_AFTER - 1.0), 0.0, "just hurt");
        assert_eq!(heal_share(FULL, THIRSTY - 0.01, 60.0), 0.0, "thirsty");
        // Thin, slower; at `HEAL_THIN` and under, not at all.
        let half = heal_share((HUNGRY + HEAL_THIN) / 2.0, 1.0, 60.0);
        assert!((half - 0.5).abs() < 1e-4, "{half}");
        assert_eq!(heal_share(HEAL_THIN, 1.0, 60.0), 0.0);
        assert_eq!(heal_share(0.0, 1.0, 60.0), 0.0);
    }
}
