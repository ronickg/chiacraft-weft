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
    fn rots_as_a_meat_and_scales_by_mass() {
        assert!(vitals::food::find(ROTS_AS).is_some());
        assert!((scale(REF_KG) - 1.0).abs() < 1e-5);
        assert!(scale(2.5) < 0.4 && scale(500.0) > 1.8);
        assert!(FROM_KG >= 1.0 && EDIBLE > crate::meat::YIELD);
    }
}
