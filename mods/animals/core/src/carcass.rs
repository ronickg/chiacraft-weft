//! A dead animal's body (`notes/crew/plans/carcass-and-skinning.md`): it lies where the animal fell, and its look steps
//! down as it is eaten (hunters, scavengers, a player's knife) and as it rots, whole, opened, mostly bones, a pile of
//! bones, then gone. Each look is the later of the two clocks: what's been taken of what can be eaten off it, and its
//! rot (`vitals::rot::carcass_stage`, aged as its raw meat would be at its spot's air).
//!
//! What can be eaten off it: its meat (`meat::YIELD`, 45 % of live mass) and the organs and fat hunters take first,
//! about 60 % of live mass in all (a wolf pack leaves hide, bones and the gut's contents: Mech 1970, Stahler 2006).

/// Share of live mass eaten off a carcass.
pub const EDIBLE: f32 = 0.6;
/// Below this live mass an animal leaves no carcass (`meat::CARCASS_FROM_KG`: a mouse's the beetles have).
pub const FROM_KG: f32 = crate::meat::CARCASS_FROM_KG;
/// Shares of `EDIBLE` taken at which it looks opened, mostly bones, and a pile of bones.
pub const OPENED: f32 = 0.25;
pub const BONES: f32 = 0.65;
pub const PILE: f32 = 0.95;
/// What a scavenger bites off at a time, kg (a vulture swallows a few hundred grams a mouthful).
pub const BITE_KG: f32 = 0.5;
/// The live mass the placeholder bone looks are drawn at (`assets/models/carcass_*`): a white-tailed deer.
pub const REF_KG: f32 = 70.0;
/// The raw meat a carcass rots as (`vitals::food::FOODS`): venison.
pub const ROTS_AS: &str = "animals:venison";

/// How it looks, least eaten first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Look {
    Whole,
    Opened,
    Bones,
    Pile,
}

/// The look by what's been taken, `eaten` kg of an animal of `mass` kg.
pub fn by_eaten(eaten: f32, mass: f32) -> Look {
    let share = eaten / (mass * EDIBLE).max(0.001);
    if share >= PILE {
        Look::Pile
    } else if share >= BONES {
        Look::Bones
    } else if share >= OPENED {
        Look::Opened
    } else {
        Look::Whole
    }
}

/// The look by rot (`None`: gone).
pub fn by_rot(stage: vitals::rot::Carcass) -> Option<Look> {
    use vitals::rot::Carcass::*;
    match stage {
        Fresh => Some(Look::Whole),
        Bloated => Some(Look::Opened),
        Remains => Some(Look::Bones),
        Bones => Some(Look::Pile),
        Gone => None,
    }
}

/// Its look: the later of the two, and at least opened once skinned (`None`: gone, as when nothing is left).
pub fn look(eaten: f32, mass: f32, rot: vitals::rot::Carcass, skinned: bool) -> Option<Look> {
    let floor = if skinned { Look::Opened } else { Look::Whole };
    by_rot(rot).map(|r| r.max(by_eaten(eaten, mass)).max(floor))
}

/// How long its meat keeps at larder pace (`vitals::food::Food::keeps`), days fresh and stale.
pub fn keeps() -> (f32, f32) {
    vitals::food::find(ROTS_AS).map_or((2.0, 2.0), |i| vitals::food::FOODS[i].keeps)
}

/// The scale its bone looks are drawn at for an animal of `mass` kg (length goes as the cube root of mass).
pub fn scale(mass: f32) -> f32 {
    (mass / REF_KG).cbrt().clamp(0.15, 4.0)
}

// ---------------------------------------------------------------- skinning (slice 2)
//
// Ronald (11-25): "skinning only": an animal that leaves a carcass drops no hide or meat when it dies; a player kneels
// at the carcass with a knife and gets its hide and meat (`skin_*`). As RDR2's pelts: a hide is poor, good or perfect
// by the animal's own quality, how cleanly it was killed and the knife; rot spoils it (`vitals::rot::CARCASS`).

/// Whether an animal of species mass `kg` drops its hide and meat when it dies (too small to leave a carcass).
pub fn drops_on_death(kg: f32) -> bool {
    kg < FROM_KG
}

/// Seconds to skin a carcass of `mass` kg at a flint knife's speed (1): a hare's 5, a deer's 8, a red deer's 10.
pub const SKIN_SECS: f32 = 3.0;
pub const SKIN_SECS_PER_CBRT_KG: f32 = 1.2;
/// A knife's speed by its tier (`base_api::ITEM_LEVEL` / 10: flint, copper and bronze, iron, diamond), a time factor.
pub const KNIFE_SPEED: [f32; 4] = [1.0, 0.8, 0.65, 0.55];
/// A bare flake, slower than any knife (and it can't give a perfect hide: tier 0).
pub const FLAKE_SPEED: f32 = 1.4;
/// Blocks from the carcass the skinner must stay within (it kneels beside it).
pub const SKIN_REACH: f64 = 3.0;

/// Seconds to skin `mass` kg with a knife of `tier` (`None`: a flake).
pub fn skin_secs(mass: f32, tier: Option<u32>) -> f32 {
    let speed = tier.map_or(FLAKE_SPEED, |t| KNIFE_SPEED[(t as usize).min(KNIFE_SPEED.len() - 1)]);
    (SKIN_SECS + SKIN_SECS_PER_CBRT_KG * mass.max(0.0).cbrt()) * speed
}

/// An animal's own quality by its wild level (`animal::roll`: most are under 40, a tenth 120 or more): 1, 2 or 3.
pub const QUALITY_LEVELS: [u32; 2] = [40, 120];
/// Hits it took: up to `CLEAN_HITS` is a clean kill (a perfect hide stays possible), up to `ROUGH_HITS` a good one at
/// best, more a poor one.
pub const CLEAN_HITS: u32 = 2;
pub const ROUGH_HITS: u32 = 6;

/// How it died, for its hide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kill {
    Clean,
    Rough,
    /// Burnt, blown up, or hacked about: a poor hide at best.
    Ruined,
}

/// How a death with `hits` and the engine's damage `cause` (`abi::damage_cause`) leaves the hide.
pub fn kill(hits: u32, cause: u32) -> Kill {
    // Fire 3, lava 7, explosion 10 (`weft_abi::damage_cause`).
    if matches!(cause, 3 | 7 | 10) || hits > ROUGH_HITS {
        Kill::Ruined
    } else if hits > CLEAN_HITS {
        Kill::Rough
    } else {
        Kill::Clean
    }
}

/// A hide's quality, 1 poor, 2 good, 3 perfect: the animal's own (by `level`), capped by the kill, the knife (a flint
/// knife or a flake gives good at best) and rot (bloated: poor).
pub fn quality(level: u32, kill: Kill, knife_tier: Option<u32>, rot: vitals::rot::Carcass) -> u8 {
    let own = 1 + QUALITY_LEVELS.iter().filter(|&&l| level >= l).count() as u8;
    let by_kill = match kill {
        Kill::Clean => 3,
        Kill::Rough => 2,
        Kill::Ruined => 1,
    };
    let by_knife = if knife_tier.unwrap_or(0) == 0 { 2 } else { 3 };
    let by_rot = if rot == vitals::rot::Carcass::Fresh { 3 } else { 1 };
    own.min(by_kill).min(by_knife).min(by_rot)
}

/// How many of a hide row's `lo..=hi` a hide of `quality` gives: the fewest when poor, the most when perfect.
pub fn hide_count(lo: u32, hi: u32, quality: u8) -> u32 {
    lo + ((hi - lo) as f32 * (quality.clamp(1, 3) - 1) as f32 / 2.0).round() as u32
}

/// How many of a meat row's `lo..=hi` portions are left when `eaten` kg of `mass` kg's edible share is gone, at rot
/// `stage` (`vitals::rot::CARCASS`' meat share): none once eaten to the bones.
pub fn meat_count(lo: u32, hi: u32, eaten: f32, mass: f32, rot: vitals::rot::Carcass) -> u32 {
    let left = (1.0 - eaten / (mass * EDIBLE).max(0.001)).clamp(0.0, 1.0);
    let share = vitals::rot::CARCASS
        .iter()
        .find(|c| c.0 == rot)
        .map_or(0.0, |c| c.2);
    ((lo + hi) as f32 / 2.0 * left * share).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use vitals::rot::Carcass;

    #[test]
    fn eaten_steps_it_down() {
        // A 70 kg deer: 42 kg can be eaten off it; a wolf's 5 kg meal leaves it whole, a pack of five opens it.
        assert_eq!(by_eaten(5.0, 70.0), Look::Whole);
        assert_eq!(by_eaten(25.0, 70.0), Look::Opened);
        assert_eq!(by_eaten(30.0, 70.0), Look::Bones);
        assert_eq!(by_eaten(42.0, 70.0), Look::Pile);
        assert!(OPENED < BONES && BONES < PILE && PILE <= 1.0);
    }

    #[test]
    fn the_later_clock_wins() {
        assert_eq!(look(0.0, 70.0, Carcass::Remains, false), Some(Look::Bones));
        assert_eq!(look(42.0, 70.0, Carcass::Fresh, false), Some(Look::Pile));
        assert_eq!(look(0.0, 70.0, Carcass::Fresh, true), Some(Look::Opened));
        assert_eq!(look(0.0, 70.0, Carcass::Gone, false), None);
    }

    #[test]
    fn skinning_by_knife_kill_and_rot() {
        // A deer with a steel knife in about 6 s, with a flake in about 11; a hare in under 5.
        assert!((skin_secs(70.0, Some(3)) - 4.4).abs() < 0.2);
        assert!(skin_secs(70.0, None) > 10.0 && skin_secs(3.0, Some(2)) < 5.0);
        // One clean arrow on a level-130 deer with an iron knife: perfect; the same with a flint knife: good.
        assert_eq!(quality(130, kill(1, 12), Some(2), Carcass::Fresh), 3);
        assert_eq!(quality(130, kill(1, 12), Some(0), Carcass::Fresh), 2);
        // Hacked at ten times, or burnt, or bloated: poor; a level-10 one is poor however it's killed.
        assert_eq!(quality(130, kill(10, 2), Some(3), Carcass::Fresh), 1);
        assert_eq!(quality(130, kill(1, 3), Some(3), Carcass::Fresh), 1);
        assert_eq!(quality(130, kill(1, 2), Some(3), Carcass::Bloated), 1);
        assert_eq!(quality(10, kill(1, 2), Some(3), Carcass::Fresh), 1);
        assert_eq!((hide_count(1, 3, 1), hide_count(1, 3, 2), hide_count(1, 3, 3)), (1, 2, 3));
        // A whole fresh deer gives its meat; half eaten, half; to the bones or rotten to remains, none... remains half.
        assert_eq!(meat_count(3, 4, 0.0, 70.0, Carcass::Fresh), 4);
        assert_eq!(meat_count(3, 4, 21.0, 70.0, Carcass::Fresh), 2);
        assert_eq!(meat_count(3, 4, 42.0, 70.0, Carcass::Fresh), 0);
        assert!(drops_on_death(0.5) && !drops_on_death(2.5));
    }

    #[test]
    fn rots_as_a_meat_and_scales_by_mass() {
        assert!(vitals::food::find(ROTS_AS).is_some());
        assert!((scale(REF_KG) - 1.0).abs() < 1e-5);
        assert!(scale(2.5) < 0.4 && scale(500.0) > 1.8);
        assert!(FROM_KG >= 1.0 && EDIBLE > crate::meat::YIELD);
    }
}
