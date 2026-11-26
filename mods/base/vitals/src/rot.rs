//! Rot by place (`notes/crew/plans/rot-plan.md`, approved 11-04, built 11-25): a food's keeps
//! (`food::Food::keeps`) are for a cool larder at `REF_C`. Where it sits speeds that up or slows it
//! down by the Q10 rule: microbes grow `Q10` times faster per 10 °C warmer (Labuza, *Shelf-life
//! dating of foods*, Q10 2-3). It fits the real gap between fridge and room: a steak keeps 3-5 days
//! at 4 °C (USDA) and under a day at 20 °C, about 5 to 1, and 2.5^1.6 = 4.3.
//!
//! The air at the food's place decides it: a chest's air (`base_api::air_at` at it, a room's
//! evened-out air inside walls, the year's mean underground), snow or ice next to it (an ice house)
//! holding it at `ICE_C`, and for what a player carries, the air the player feels. The world's
//! difficulty `keep` still stretches the keeps on top. Spoiled food is as before: the "spoiled"
//! mark, a 50% chance of poisoning, compost for 2 (`food::poisoning_chance`, `soil::SPOILED`).
//!
//! Carcasses: the numbers only (`CARCASS`, `carcass_stage`), for Animals' wave two to build the
//! body that lies where an animal fell and rots as its meat does.

/// The air the keeps in `food::FOODS` are for: a cool larder, so temperate play stays as tuned.
pub const REF_C: f32 = 15.0;
/// How many times faster food spoils per 10 °C warmer (the middle of 2-3).
pub const Q10: f32 = 2.5;
/// The fastest it gets (from 30 °C): a desert noon still leaves raw meat a day (20 minutes).
pub const FASTEST: f32 = 4.0;
/// Below this the food is frozen, and keeps for months (USDA: frozen meat 4-12 months).
pub const FROZEN_C: f32 = -2.0;
/// How fast frozen food ages, against the larder: a porkchop 80 game days, over a year.
pub const FROZEN: f32 = 0.05;
/// Snow or ice next to a store (an ice house) holds it at this; ice houses held 0-4 °C.
pub const ICE_C: f32 = 0.0;
/// Game ticks between agings of a store's food (an hour); a player's is aged every second.
pub const STORE_EVERY: u32 = 1000;

/// How many times faster than in a larder food spoils in air at `celsius`: 1 at `REF_C`,
/// `Q10` per 10 °C, at most `FASTEST`, and `FROZEN` below `FROZEN_C`.
pub fn rate(celsius: f32) -> f32 {
    if celsius < FROZEN_C {
        return FROZEN;
    }
    Q10.powf((celsius - REF_C) / 10.0).min(FASTEST)
}

/// The air food in a store keeps in: `air` there, held at `ICE_C` (if colder air doesn't freeze
/// it further) when snow or ice is next to it.
pub fn store_air(air: f32, ice_next_to: bool) -> f32 {
    if ice_next_to { air.min(ICE_C) } else { air }
}

/// Game days a food keeping `keeps` (fresh, stale) at the larder lasts in total at `celsius`.
pub fn days_at(keeps: (f32, f32), celsius: f32) -> f32 {
    (keeps.0 + keeps.1) / rate(celsius)
}

/// Ticks a food stack's making stamp moves on over `ticks` at `rate`: forward (younger) where it
/// keeps better than a larder, back (older) where it keeps worse. `carry` is the fraction left
/// over from last time, so slow rates never round away; returns (whole ticks, new carry).
pub fn shift(ticks: u64, rate: f32, carry: f32) -> (i64, f32) {
    let exact = (1.0 - rate as f64) * ticks as f64 + carry as f64;
    let whole = exact.trunc();
    (whole as i64, (exact - whole) as f32)
}

/// The places on the sheet's "where food keeps" table, and their air.
pub const PLACES: [(&str, f32); 6] = [
    ("desert noon", 35.0),
    ("summer day", 25.0),
    ("larder (as tuned)", REF_C),
    ("temperate cellar", 10.0),
    ("snow or ice store", ICE_C),
    ("frozen", -5.0),
];

/// A carcass's stages (`notes/crew/plans/rot-plan.md`, for Animals' wave two). It ages as its raw
/// meat would at its spot's air: fresh while the meat is fresh, bloated while it's stale (the meat
/// comes out stale), then remains (rotten flesh and bones, no hide), then bones only, then gone.
/// In real life (Payne 1965, a summer field): fresh 0-2 days, bloat 2-6, decay 5-11, dry 25+.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Carcass {
    Fresh,
    Bloated,
    Remains,
    Bones,
    Gone,
}

/// Each stage: its name, and what butchering it gives against a fresh kill (meat, hide, bones;
/// meat at stage `Remains` comes out as rotten flesh).
pub const CARCASS: [(Carcass, &str, f32, f32, f32); 5] = [
    (Carcass::Fresh, "fresh", 1.0, 1.0, 1.0),
    (Carcass::Bloated, "bloated", 1.0, 1.0, 1.0),
    (Carcass::Remains, "remains", 0.5, 0.0, 1.0),
    (Carcass::Bones, "bones", 0.0, 0.0, 1.0),
    (Carcass::Gone, "gone", 0.0, 0.0, 0.0),
];
/// After spoiling, remains last as long again as the meat's whole life, and bones as long again.
pub const REMAINS_LIVES: f32 = 1.0;
pub const BONES_LIVES: f32 = 1.0;

/// A carcass's stage at `larder_days` of age counted at larder pace (each day's real days times
/// `rate` at its spot, as a food's stamp), for meat keeping `keeps` (`food::Food::keeps`).
pub fn carcass_stage(keeps: (f32, f32), larder_days: f32) -> Carcass {
    let life = keeps.0 + keeps.1;
    if larder_days < keeps.0 {
        Carcass::Fresh
    } else if larder_days < life {
        Carcass::Bloated
    } else if larder_days < life * (1.0 + REMAINS_LIVES) {
        Carcass::Remains
    } else if larder_days < life * (1.0 + REMAINS_LIVES + BONES_LIVES) {
        Carcass::Bones
    } else {
        Carcass::Gone
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::food::{self, Freshness};

    /// The plan's check 1: 1 at the larder, rising with warmth, at most `FASTEST`, at most
    /// `FROZEN` frozen.
    #[test]
    fn rate_rises_with_warmth() {
        assert!((rate(REF_C) - 1.0).abs() < 1e-6);
        let mut last = 0.0;
        for c in -1..=50 {
            let r = rate(c as f32);
            assert!(r >= last, "{c} °C: {r} < {last}");
            assert!(r <= FASTEST);
            last = r;
        }
        assert!(rate(-10.0) <= FROZEN && rate(-3.0) <= FROZEN);
        assert!(store_air(20.0, true) <= ICE_C && store_air(-10.0, true) == -10.0);
    }

    fn porkchop() -> (f32, f32) {
        food::FOODS[food::find("base:porkchop").expect("porkchop")].keeps
    }

    /// Checks 2 and 3: a temperate cellar keeps raw meat at least twice as long as a summer day,
    /// and a desert noon leaves it fresh at least 10 minutes.
    #[test]
    fn cellar_beats_summer_and_desert_leaves_minutes() {
        let k = porkchop();
        assert!(days_at(k, 10.0) >= 2.0 * days_at(k, 25.0));
        let fresh_secs = k.0 / rate(35.0) * crate::DAY_SECS;
        assert!(fresh_secs >= 600.0, "raw meat fresh {fresh_secs} s at a desert noon");
    }

    /// Check 6: fridge to room (4 °C to 20 °C, the USDA steak) is 3 to 6.
    #[test]
    fn fridge_to_room_ratio() {
        let ratio = rate(20.0) / rate(4.0);
        assert!((3.0..=6.0).contains(&ratio), "{ratio}");
    }

    /// Ties rot to temperature: a stamp shifted by the rate gives the same freshness as the keeps
    /// stretched by it, so cold stores, rooms and hot sun come out in that order.
    #[test]
    fn shifted_stamps_order_cold_room_hot() {
        let k = porkchop();
        let day = crate::calendar::DAY_TICKS;
        let used = |celsius: f32| {
            let (s, _) = shift(3 * day, rate(celsius), 0.0);
            (3 * day) as i64 - s
        };
        let (cold, room, hot) = (used(0.0), used(REF_C), used(35.0));
        assert!(cold < room && room < hot);
        let fresh = |t: i64| food::freshness_of(k, t as f32 / crate::TICKS as f32, 1.0);
        assert_eq!(fresh(cold), Freshness::Fresh);
        assert_eq!(fresh(room), Freshness::Stale);
        assert_eq!(fresh(hot), Freshness::Spoiled);
        // Slow rates add up through the carry, never rounding away.
        let (mut total, mut carry) = (0i64, 0.0f32);
        for _ in 0..1000 {
            let (s, c) = shift(1, 0.25, carry);
            total += s;
            carry = c;
        }
        assert!((total - 750).abs() <= 1, "{total}");
    }

    /// Check 5 in its vitals half: a carcass is never fresher than meat of the same age, and goes
    /// through every stage in order.
    #[test]
    fn carcass_never_fresher_than_meat() {
        let k = porkchop();
        let mut last = 0;
        for i in 0..400 {
            let d = i as f32 * 0.05;
            let stage = carcass_stage(k, d);
            let at = CARCASS.iter().position(|c| c.0 == stage).unwrap();
            assert!(at >= last);
            last = at;
            let meat = food::freshness_of(k, d * crate::DAY_SECS, 1.0);
            match meat {
                Freshness::Fresh => {}
                Freshness::Stale => assert_ne!(stage, Carcass::Fresh),
                Freshness::Spoiled => assert!(at >= 2),
            }
        }
        assert_eq!(last, CARCASS.len() - 1);
    }
}
