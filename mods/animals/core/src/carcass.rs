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
/// A mouthful an eater tears off a carcass, a share of its own mass (a wolf's a kilo, a turkey vulture's 40 g), one a
/// chew (`forage.rs`), and no more than it still needs, so it feeds a few mouthfuls and leaves the rest (a fixed 0.5 kg
/// bite filled a 1.4 kg vulture, whose meal is 140 g, at once and took the rest off the body for nothing).
pub const MOUTHFUL: f32 = 0.03;
/// The least it takes in a mouthful, kg.
pub const MOUTHFUL_MIN_KG: f32 = 0.02;

/// The kg an eater of `own` kg, `full` of its food stat (0..1), takes off a carcass in a mouthful.
pub fn bite_kg(own: f32, full: f32) -> f32 {
    let need = own * crate::needs::MEAL * (1.0 - full).clamp(0.0, 1.0);
    (own * MOUTHFUL).min(need).max(MOUTHFUL_MIN_KG)
}
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

// ---------------------------------------------------------------- the bodies drawn (slice 3)
//
// A whole or opened body is one lying model per body plan (`source/carcass/body.py`), at its reference animal's size,
// scaled to each species by the cube root of its mass, and painted in the animal's own coat (`mods/animals/src`
// `carcass.rs`). The bones and the pile stay the deer's (`scale`).

/// The body plans a dead body is drawn as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plan {
    Quadruped,
    Bird,
    Biped,
    Reptile,
    Fish,
}

impl Plan {
    pub const ALL: [Plan; 5] = [Plan::Quadruped, Plan::Bird, Plan::Biped, Plan::Reptile, Plan::Fish];

    /// Its models: `carcass_<name>` whole, `carcass_<name>_open` opened.
    pub fn name(self) -> &'static str {
        match self {
            Plan::Quadruped => "quadruped",
            Plan::Bird => "bird",
            Plan::Biped => "biped",
            Plan::Reptile => "reptile",
            Plan::Fish => "fish",
        }
    }

    /// The mass of the animal its model is built at, kg: a white-tailed deer, a vulture, a raptor, a caiman, a trout.
    pub fn ref_kg(self) -> f32 {
        match self {
            Plan::Quadruped => 70.0,
            Plan::Bird => 5.0,
            Plan::Biped => 30.0,
            Plan::Reptile => 40.0,
            Plan::Fish => 3.0,
        }
    }

    /// The blood pool's scale under an opened body against the body's own (the pool is built at the deer's size).
    pub fn pool(self) -> f32 {
        match self {
            Plan::Quadruped => 1.0,
            Plan::Bird => 0.45,
            Plan::Biped => 0.8,
            Plan::Reptile => 0.9,
            Plan::Fish => 0.4,
        }
    }
}

/// The plan a species' model is drawn dead as (its living model's name, `species::Species::model`).
pub fn plan_of(model: &str) -> Plan {
    match model {
        "eagle" | "early_bird" | "gull" | "owl" | "raven" | "small_bird" | "vulture" => Plan::Bird,
        "citipati" | "raptor" | "utahraptor" | "iguanodont" | "psittacosaurus" => Plan::Biped,
        "ankylosaur" | "caiman" | "lizard" | "protoceratops" | "sauropod" | "wyvern" => Plan::Reptile,
        "fish" => Plan::Fish,
        _ => Plan::Quadruped,
    }
}

/// The scale a whole or opened body of `mass` kg is drawn at in its `plan`'s model.
pub fn body_scale(mass: f32, plan: Plan) -> f32 {
    (mass / plan.ref_kg()).cbrt().clamp(0.1, 12.0)
}

/// The coat and underside the body texture is painted in (`source/carcass/body.py`): the painter turns them to the
/// animal's own by channel ratios.
pub const COAT_RGB: [f32; 3] = [124.0, 80.0, 50.0];
pub const UNDER_RGB: [f32; 3] = [212.0, 194.0, 166.0];

// ---------------------------------------------------------------- rot drawn
//
// A body shows its rot (the stages are `vitals::rot::carcass_stage`'s): fresh, it is the animal's coat; bloating, the
// skin and fur go a dull green-black (the marbling of decay) step by step; gone to remains, a dried grey-brown hide.
// Blow flies hover over it while it bloats and rots (they come within minutes of a death and are thickest at bloat and
// active decay; few stay at dry remains, none at bones).

/// Steps the rot is drawn in (each a repaint of the coat): 0 fresh, 1.. through the bloat, `ROT_STEPS` at remains.
pub const ROT_STEPS: u8 = 6;
/// The colour a bloating body turns toward and how far at its last step (0..1), and a dry one's.
pub const BLOAT_RGB: [f32; 3] = [66.0, 78.0, 46.0];
pub const BLOAT_TINT: f32 = 0.45;
pub const DRY_RGB: [f32; 3] = [118.0, 104.0, 84.0];
pub const DRY_TINT: f32 = 0.55;
/// How much darker it is at the full tint (wet, then dark dried skin), a white texel's (a dark one's less: a white belly
/// goes a dirty grey-green, not mint).
pub const ROT_DARKEN: f32 = 0.42;
/// Flies over a deer-sized body at once while it bloats (as many again per body as big by area), and the share left at
/// remains.
pub const FLIES: f32 = 10.0;
pub const FLIES_MAX: u8 = 24;
pub const DRY_FLIES: f32 = 0.3;

/// The rot step drawn at `larder_days` of age for meat keeping `keeps` (`keeps()`).
pub fn rot_step(keeps: (f32, f32), larder_days: f32) -> u8 {
    let t = (larder_days - keeps.0) / keeps.1.max(1e-3);
    if t <= 0.0 {
        0
    } else if t >= 1.0 {
        ROT_STEPS
    } else {
        1 + (t * (ROT_STEPS - 1) as f32) as u8
    }
}

/// Turns a painted body (`rgba`, every opaque texel: coat, meat and bones alike) to rot `step`'s colour, keeping each
/// texel's brightness.
pub fn rot_tint(rgba: &mut [u8], step: u8) {
    if step == 0 {
        return;
    }
    let (to, k) = if step >= ROT_STEPS { (DRY_RGB, DRY_TINT) } else { (BLOAT_RGB, BLOAT_TINT * step as f32 / (ROT_STEPS - 1) as f32) };
    let lum = |c: [f32; 3]| 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2];
    let full = k / BLOAT_TINT.max(DRY_TINT);
    let lt = lum(to);
    for px in rgba.as_chunks_mut::<4>().0 {
        if px[3] == 0 {
            continue;
        }
        let l = lum([px[0] as f32, px[1] as f32, px[2] as f32]);
        let dark = 1.0 - ROT_DARKEN * full * (0.35 + 0.65 * l / 255.0);
        for c in 0..3 {
            let v = px[c] as f32 * (1.0 - k) + l * to[c] / lt * k;
            px[c] = (v * dark).round().clamp(0.0, 255.0) as u8;
        }
    }
}

/// Flies over a body at rot `step`, drawn as `look`, at `scale` of its plan's reference animal.
pub fn flies(step: u8, look: Look, scale: f32) -> u8 {
    let n = match (step, look) {
        (0, _) | (_, Look::Bones | Look::Pile) => return 0,
        (s, _) if s >= ROT_STEPS => FLIES * DRY_FLIES,
        _ => FLIES,
    };
    (n * scale * scale).round().clamp(1.0, FLIES_MAX as f32) as u8
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
/// Blocks from a reference-sized body's centre the skinner kneels (times its `body_scale`, at least `KNEEL_MIN`): the
/// hands of the `skin` clip reach about 0.55 blocks ahead, a lying deer is about 0.3 blocks to its middle (r1b's
/// skinner knelt where they clicked, a body's width short of it).
pub const KNEEL_FROM: f64 = 0.8;
pub const KNEEL_MIN: f64 = 0.45;

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
    fn mouthfuls_by_the_eater() {
        // A turkey vulture (1.4 kg) takes 40 g a mouthful and is full after a few, its 140 g meal; never more than it
        // needs; a wolf about a kilo.
        let v = bite_kg(1.4, 0.0);
        assert!((0.03..0.06).contains(&v), "{v}");
        let mouthfuls = (1.4 * crate::needs::MEAL / v).ceil();
        assert!((3.0..=5.0).contains(&mouthfuls), "{mouthfuls}");
        assert!(bite_kg(1.4, 0.9) <= 1.4 * crate::needs::MEAL * 0.1 + MOUTHFUL_MIN_KG);
        assert!((0.9..1.5).contains(&bite_kg(40.0, 0.0)));
        assert_eq!(bite_kg(40.0, 1.0), MOUTHFUL_MIN_KG);
    }

    #[test]
    fn rot_steps_and_flies() {
        let k = keeps();
        assert_eq!(rot_step(k, 0.0), 0);
        assert_eq!(rot_step(k, k.0 + k.1 * 0.01), 1);
        assert_eq!(rot_step(k, k.0 + k.1 * 0.99), ROT_STEPS - 1);
        assert_eq!(rot_step(k, k.0 + k.1), ROT_STEPS);
        let mut last = 0;
        for i in 0..100 {
            let s = rot_step(k, i as f32 * 0.1);
            assert!(s >= last);
            last = s;
        }
        // The tint darkens and pulls toward green while bloating; nothing while fresh, transparent untouched.
        let mut px = [200, 150, 100, 255, 9, 9, 9, 0];
        rot_tint(&mut px, 0);
        assert_eq!(px, [200, 150, 100, 255, 9, 9, 9, 0]);
        rot_tint(&mut px, ROT_STEPS - 1);
        assert!(px[0] < 200 && px[1] < 150 && px[4..] == [9, 9, 9, 0]);
        assert_eq!(flies(0, Look::Whole, 1.0), 0);
        assert_eq!(flies(3, Look::Bones, 1.0), 0);
        assert!(flies(ROT_STEPS, Look::Opened, 1.0) < flies(3, Look::Opened, 1.0));
        assert!(flies(3, Look::Whole, 0.3) >= 1 && flies(3, Look::Whole, 5.0) <= FLIES_MAX);
    }

    #[test]
    fn eaten_steps_it_down() {
        // A 70 kg deer: 42 kg can be eaten off it; a wolf's 5 kg meal leaves it whole, a pack of five opens it.
        assert_eq!(by_eaten(5.0, 70.0), Look::Whole);
        assert_eq!(by_eaten(25.0, 70.0), Look::Opened);
        assert_eq!(by_eaten(30.0, 70.0), Look::Bones);
        assert_eq!(by_eaten(42.0, 70.0), Look::Pile);
        const { assert!(OPENED < BONES && BONES < PILE && PILE <= 1.0) };
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
        assert!((body_scale(70.0, Plan::Quadruped) - 1.0).abs() < 1e-5);
        assert!(body_scale(2.5, Plan::Quadruped) < 0.4 && body_scale(500.0, Plan::Quadruped) > 1.8);
        const { assert!(FROM_KG >= 1.0 && EDIBLE > crate::meat::YIELD) };
    }

    #[test]
    fn every_species_has_a_plan_and_a_sane_size() {
        for s in &crate::species::SPECIES {
            let plan = plan_of(s.model);
            let k = body_scale(s.bio.mass, plan);
            assert!((0.1..=12.0).contains(&k), "{}: {k}", s.name);
        }
        assert_eq!(plan_of("deer"), Plan::Quadruped);
        assert_eq!(plan_of("vulture"), Plan::Bird);
        assert!((body_scale(70.0, Plan::Quadruped) - 1.0).abs() < 1e-6);
    }
}
