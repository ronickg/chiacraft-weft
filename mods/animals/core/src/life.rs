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

/// Ticks from mating to the young born, or the eggs laid.
pub fn carrying(s: &Species) -> u32 {
    if s.gestation() > 0 { s.gestation() } else { LAYING }
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

/// Whether an egg develops in that air (`egg_air`).
pub fn develops(egg_air: f32) -> bool {
    (EGG_AIR.0..=EGG_AIR.1).contains(&egg_air)
}
