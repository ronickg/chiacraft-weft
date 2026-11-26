//! Breeding and growing up as numbers (the mod's `breeding.rs` runs them in the world; the
//! website's simulator over time): how long love, rest and laying last, a young one's life stage
//! and size as it grows, how big a litter is and which parent each stat's wild points come from,
//! and old age: how long each one lives, when it becomes an elder, how old a wild one is when met;
//! how strong it is at its age; and how often wild ones breed.

use crate::species::{ADOLESCENT, ADULT, BABY, ELDER, JUVENILE, STATS, Species};

/// Thirty seconds in love; a male rests five minutes after mating.
pub const LOVE_TICKS: u32 = 600;
pub const MALE_REST: u32 = 6_000;
/// Five minutes after mating, an egg-layer lays.
pub const LAYING: u32 = 6_000;
/// Ticks an in-game day.
pub const DAY: u64 = 24_000;

/// A young one's life stage with `growing` ticks of growing up left (an adult stays one).
pub fn stage(s: &Species, growing: u32, was: u8) -> u8 {
    match growing * 3 / s.growth().max(1) {
        _ if growing == 0 => was.max(ADULT),
        2.. => BABY,
        1 => JUVENILE,
        _ => ADOLESCENT,
    }
}

/// How big it is against a grown one, with `growing` ticks left: from its birth size up.
pub fn size(s: &Species, growing: u32) -> f32 {
    if growing == 0 {
        return 1.0;
    }
    let grown = 1.0 - growing as f32 / s.growth().max(1) as f32;
    s.birth_size + (1.0 - s.birth_size) * grown
}

/// Ticks from mating to the young born, or the eggs laid. A fish lays too: its roe lies on the
/// bottom, seen, for its incubation (`roe_develops`), then the fry come (Planner, 11-06, for
/// Ronald's "eggs you can see per species"; before, the fry simply came after the eggs' time).
pub fn carrying(s: &Species) -> u32 {
    if s.gestation() > 0 { s.gestation() } else { LAYING }
}

/// Whether a fish's roe develops where it lies: in water, whatever its warmth (a species' water
/// is its habitat's; out of the water it dries and waits).
pub fn roe_develops(in_water: bool) -> bool {
    in_water
}

/// How many young a litter or clutch has, `roll` any random number.
pub fn litter(s: &Species, roll: u32) -> u32 {
    s.litter().0 + roll % (s.litter().1 - s.litter().0 + 1)
}

/// A young one's wild points: each stat's from one parent, the stronger 55% of the time
/// (`roll(100)` a random number below 100).
pub fn wild_points(mother: &[u8; STATS], father: &[u8; STATS], mut roll: impl FnMut(u32) -> u32) -> [u8; STATS] {
    std::array::from_fn(|i| {
        let (hi, lo) = (mother[i].max(father[i]), mother[i].min(father[i]));
        if roll(100) < 55 { hi } else { lo }
    })
}

/// Births are counted from a start this long before the world's (a hundred thousand in-game
/// days), so a wild one met when a world is new can be older than the world.
pub const EPOCH: u64 = 100_000 * DAY;

/// When one `age` ticks old at tick `now` was born (`EPOCH`-based; never 0, which means unknown).
pub fn birth(now: u64, age: u64) -> u64 {
    (now + EPOCH).saturating_sub(age).max(1)
}

/// How old one born at `born` (`birth`) is at tick `now`.
pub fn age(now: u64, born: u64) -> u64 {
    (now + EPOCH).saturating_sub(born)
}

/// Old from this share of its lifespan on: an elder, greying and past breeding.
pub const ELDERLY: f64 = 0.8;

/// How long an animal lives, in ticks: its species' lifespan, give or take a fifth by its genes'
/// seed (so siblings don't die together); `u64::MAX` for a species that doesn't age.
pub fn lifespan(s: &Species, seed: u32) -> u64 {
    if s.lifespan() == 0 {
        return u64::MAX;
    }
    let h = seed.wrapping_mul(2_654_435_761) ^ seed.rotate_left(13);
    let share = 0.8 + 0.4 * (h % 10_000) as f64 / 10_000.0;
    (s.lifespan() as f64 * share) as u64
}

/// Its life stage at `age` ticks old: a grown one past `ELDERLY` of its `lifespan` is an elder.
pub fn aged(stage: u8, age: u64, lifespan: u64) -> u8 {
    if stage >= ADULT && age as f64 >= ELDERLY * lifespan as f64 { ELDER } else { stage }
}

/// How old a wild one is when first met, `roll` 0..1: grown, and up to most of an average life.
pub fn wild_age(s: &Species, roll: f32) -> u64 {
    let grown = s.growth() as u64;
    let most = (s.lifespan() as f64 * 0.7) as u64;
    grown + (most.saturating_sub(grown) as f64 * roll.clamp(0.0, 1.0) as f64) as u64
}

/// Whether one at `stage` may fall in love (grown, not old).
pub fn can_breed(stage: u8) -> bool {
    stage == ADULT
}

/// How strong one is at its age, against its prime (1): growing up from 0.4 at birth, young
/// adults a little short of their prime, full strength until the elder years, then fading to
/// 0.6 at an average life's end. Health, stamina, damage, speed and what it carries go with it.
pub fn vigor(s: &Species, growing: u32, age: u64) -> f32 {
    if growing > 0 {
        let grown = 1.0 - growing as f32 / s.growth().max(1) as f32;
        return 0.4 + 0.5 * grown;
    }
    if s.lifespan() == 0 {
        return 1.0;
    }
    let (t, grown) = (age as f32 / s.lifespan() as f32, s.growth() as f32 / s.lifespan() as f32);
    let elderly = ELDERLY as f32;
    if t < grown * 2.0 {
        0.9 + 0.1 * ((t - grown) / grown).clamp(0.0, 1.0)
    } else if t < elderly {
        1.0
    } else {
        (1.0 - 0.4 * (t - elderly) / (1.0 - elderly)).max(0.5)
    }
}

/// Blocks round a wild one within which its kind count as near (`Species::crowd`).
pub const CROWD_RADIUS: f64 = 48.0;
/// How often a grown, rested wild one with room around falls in love: about every other day.
pub const WILD_LOVE_PER_DAY: f64 = 0.5;

/// How much one at `stage` crowds its kind (`love_chance`): an elder half, so the next
/// generation is born before the old one dies (counted fully, a group of elders stopped all
/// breeding and died out together: the pen test in `notes/ANIMALS.md`).
pub fn crowding(stage: u8) -> f64 {
    if stage >= ELDER { 0.5 } else { 1.0 }
}

/// The chance that a grown, rested wild one falls in love within `days`, with `near` of its kind
/// near (itself among them, each counted by `crowding`) and room for `crowd`: high when few,
/// none when crowded (logistic growth: a population grows fast when small and levels off).
pub fn love_chance(days: f64, near: f64, crowd: u32) -> f64 {
    let full = near / crowd.max(1) as f64;
    if full >= CROWDED {
        return 0.0;
    }
    // A little even when full: ages overlap, so there are young grown ones when the old die (none
    // at all past the crowd, a generation filled a pen at once, aged together and died out).
    let room = (1.0 - full).max(BREEDING_WHEN_FULL);
    (WILD_LOVE_PER_DAY * days * room).clamp(0.0, 1.0)
}

/// How much of the room's breeding goes on when its kind are already at their crowd.
pub const BREEDING_WHEN_FULL: f64 = 0.1;
/// No breeding at all past this many times its crowd.
pub const CROWDED: f64 = 3.0;

/// The chance that a wild one at `stage` dies of want within `days` with `near` of its kind
/// round it (counted by `crowding`) against its `crowd`: none with room, more the more crowded,
/// the old and the young first (hunger and illness take them). With births going on a little
/// even when full, this keeps a population turning over at its crowd (overlapping generations)
/// instead of one generation filling the place, aging together and dying out.
pub fn want_chance(days: f64, near: f64, crowd: u32, stage: u8) -> f64 {
    let over = near / crowd.max(1) as f64 - 1.0;
    if over <= 0.0 {
        return 0.0;
    }
    let frail = match stage {
        s if s >= ELDER => 3.0,
        s if s < ADULT => 2.0,
        _ => 1.0,
    };
    (DEATHS_WHEN_CROWDED * days * over * frail).clamp(0.0, 1.0)
}

/// Of a crowd twice its size, the share of grown ones dying of want a day.
pub const DEATHS_WHEN_CROWDED: f64 = 0.1;

/// Whether a group of `coming` finds room where `near` of its kind already live (counted by
/// `crowding`, within `CROWD_RADIUS`) with room for `crowd`: natural spawning brings none past the
/// room, so a place fills to its crowd and no further. Without it the engine's spawner, which caps
/// only all creatures together, added packs to a place already full, and they died of want
/// (`want_chance`) as fast as they came. A group bigger than the room still comes to an empty
/// place.
pub fn room_for(near: f64, coming: u32, crowd: u32) -> bool {
    near + coming as f64 <= crowd.max(coming) as f64
}

/// Ticks in a second.
pub const SECOND: u64 = 20;

/// Whether an animal's once-a-second upkeep (its body, hunger and thirst) runs this tick, and
/// how many seconds it covers: `time` is the world's tick, `elapsed` the ticks since the animal
/// last ticked (`World::tick_elapsed`). It runs on the tick a second turns over for it
/// (`(time + id) % SECOND < elapsed`) and covers what passed (`elapsed / SECOND`, at least one).
/// One no player is near rests and ticks only every `core:inactive_entity_interval` ticks: run on
/// `(time + id) % 20 == 0` alone, it lived a second for every wake whatever the interval (half as
/// fast at 40, a third at 60; at 30 two wakes in three were missed).
pub fn seconds_due(time: u64, id: u64, elapsed: u32) -> Option<f32> {
    let e = elapsed.max(1) as u64;
    ((time + id) % SECOND < e).then(|| (e as f32 / SECOND as f32).max(1.0))
}

/// How many of its every-`every`-seconds steps fall in the `secs` seconds (`seconds_due`) up to
/// tick `time` (a point of food used every so many seconds, `animal::hunger`): the windows of
/// successive upkeeps meet, so none is missed or counted twice.
pub fn steps_in(time: u64, secs: f32, every: u64) -> u64 {
    let period = every.max(1) * SECOND;
    let window = (secs * SECOND as f32).round() as u64;
    time / period - time.saturating_sub(window) / period
}

/// Ticks since an animal was last seen (its area loaded) past which its time away is caught up
/// on load, as lazy simulations do (Vintage Story's crops, Minecraft's Elapsed): timers run out,
/// young are born, it ages, and a wild one that died meanwhile has a successor.
pub const AWAY: u64 = 600;

/// How old a wild one's successor is (born while its area was away, `away` ticks), `roll` 0..1:
/// any age up to the time away or most of a life.
pub fn successor_age(s: &Species, away: u64, roll: f32) -> u64 {
    let most = away.min((s.lifespan() as f64 * 0.8) as u64);
    (most as f64 * roll.clamp(0.0, 1.0) as f64) as u64
}

/// The air an egg develops in, °C (eggs of birds and reptiles: about 26 to 38).
pub const EGG_AIR: (f32, f32) = (25.0, 40.0);
/// A brooding parent's warmth, °C, and how far a nest takes an egg toward it, both ways.
pub const BROODING: f32 = 37.0;
pub const NEST_HOLDS: f32 = 0.6;

/// The air at an egg in `air`, in a nest or not.
pub fn egg_air(air: f32, nest: bool) -> f32 {
    if nest { air + (BROODING - air) * NEST_HOLDS } else { air }
}

/// The air at an egg in `air`: in a brooded nest (a bird's, `nests::site`) a sitting parent's
/// whatever the air, else as `egg_air`.
pub fn egg_air_of(air: f32, nest: bool, brooded: bool) -> f32 {
    if nest && brooded { BROODING } else { egg_air(air, nest) }
}

/// Whether an egg develops in that air (`egg_air`).
pub fn develops(egg_air: f32) -> bool {
    (EGG_AIR.0..=EGG_AIR.1).contains(&egg_air)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A far animal lives as fast as a near one (`seconds_due`, `steps_in`): ticking only every
    /// 20, 30, 40, 60 or 200 ticks (`core:inactive_entity_interval`), or every tick, its upkeep
    /// covers a minute in a minute, its points of food are used on time, and a countdown run
    /// down by the ticks elapsed (taming's fright and wait, a dog's sitting, breeding) ends
    /// within one wake of when it is due.
    #[test]
    fn a_far_animal_keeps_time() {
        for interval in [1u64, 7, 20, 30, 40, 60, 200] {
            for id in [0u64, 3, 17, 1234] {
                let (mut secs, mut food, mut last, mut left, mut ended) = (0.0f32, 0u64, 0u64, 600u32, None);
                // Ticks as the engine wakes a resting entity: when (time + id) % interval == 0.
                for time in 1..=1200u64 {
                    if (time + id) % interval != 0 {
                        continue;
                    }
                    let elapsed = if last == 0 { 1 } else { (time - last) as u32 };
                    last = time;
                    left = left.saturating_sub(elapsed);
                    if left == 0 && ended.is_none() {
                        ended = Some(time);
                    }
                    if let Some(s) = seconds_due(time, id, elapsed) {
                        secs += s;
                        food += steps_in(time, s, 5);
                    }
                }
                assert!(
                    (secs - 60.0).abs() <= interval as f32 / SECOND as f32 + 1.0,
                    "interval {interval}, id {id}: {secs} s of upkeep in 60"
                );
                assert!(food.abs_diff(12) <= 1 + interval / 100, "interval {interval}, id {id}: {food} points of food in a minute, not 12");
                let ended = ended.expect("the countdown ended");
                assert!(ended >= 600 && ended < 600 + 2 * interval, "interval {interval}, id {id}: a 600-tick countdown ended at {ended}");
            }
        }
    }

    /// Natural spawning fills a place to its crowd and no further (the 48-minute PC soak: wolves
    /// at 21-52 where there is room for 16, dying of want): a grey wolf's pack comes while it
    /// fits, not into a place already full, and a pack bigger than the room still comes to an
    /// empty one.
    #[test]
    fn natural_spawns_stop_at_the_room() {
        let wolf = crate::species::SPECIES
            .iter()
            .find(|s| s.name == "grey_wolf")
            .expect("the grey wolf's row");
        let (crowd, (few, most)) = (wolf.crowd(), wolf.bio.social.group());
        assert!(room_for(0.0, most, crowd), "a pack of {most} doesn't fit an empty place with room for {crowd}");
        assert!(room_for((crowd - few) as f64, few, crowd), "the last {few} of the room were turned away");
        assert!(!room_for(crowd as f64, few, crowd), "a pack of {few} came to a full place ({crowd})");
        assert!(!room_for((crowd - few) as f64 + 0.5, few, crowd), "a pack came past the room");
        // Every species: a place at its crowd takes no more, and one where none live takes a group.
        for s in &crate::species::SPECIES {
            let (few, most) = s.bio.social.group();
            assert!(room_for(0.0, most, s.crowd()), "{}: no room for its group in an empty place", s.name);
            assert!(!room_for(s.crowd() as f64, few.max(1), s.crowd()), "{}: room past its crowd", s.name);
            // Nothing that comes is past what dies of want (`want_chance` is 0 up to the crowd).
            assert_eq!(want_chance(1.0, s.crowd() as f64, s.crowd(), ADULT), 0.0);
        }
    }
}
