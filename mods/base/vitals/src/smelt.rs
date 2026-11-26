//! Copper's ore, smelt and casting (`notes/COPPER.md`, slice 2): malachite in the dry lands and on
//! beaches, a furnace that melts copper only when it burns hot (charcoal, with bellows beside
//! it), and fired-clay moulds beside a hot furnace that copper ingots are cast in.
//!
//! Real numbers (`reports/content/Tools and materials.md`, C1; `reports/first-hours-research/
//! prehistory.md`): copper melts at 1,085 °C and malachite reduces with charcoal from about
//! 700 °C, but a wood fire in a shaft reaches 600-800 °C; forced air (a blowpipe, then bellows)
//! over charcoal brought it past 1,100 °C. Tin melts at 232 °C, any fire does. Casting into an
//! open or bivalve mould is minutes; the mould is reused until it cracks.

use crate::DAY_SECS;
use crate::plants::GROWING;

/// Fuel that burns at least this long burns hot: charcoal (`base:coal`, 1,600 ticks) and what
/// burns longer (a coal block, lava). Wood, sticks and planks (100-300) don't.
pub const HOT_FUEL_TICKS: u32 = 1600;

/// The metals whose raw ore (`#raw_ores/<m>`) melts only in a hot fire: hot fuel burning and
/// bellows beside the furnace (any of its six neighbours). Tin melts in any fire. Iron waits for
/// the bloomery (`notes/CONTENT.md` step 7), and smelts as before until then.
pub const HOT_METALS: [&str; 1] = ["copper"];

/// Whether `metal`'s ore needs the hot fire.
pub fn needs_hot_fire(metal: &str) -> bool {
    HOT_METALS.contains(&metal)
}

/// Whether a furnace burning fuel of `burn_max` ticks, with or without bellows beside it, is hot
/// enough for copper.
pub fn hot(burn_max: u32, bellows: bool) -> bool {
    bellows && burn_max >= HOT_FUEL_TICKS
}

/// The lands where copper's shallow ore is malachite, the green carbonate that weathers out of
/// copper ore near the surface in dry country (`metals.rs`' veins by biome tag; the desert's,
/// `notes/CONTENT.md` "Where things are"). Elsewhere copper ore stays `copper_ore`.
pub const MALACHITE_LANDS: [&str; 2] = ["dry", "beach"];

/// The moulds' shapes: one fired-clay block for each head copper is cast into.
pub const MOULDS: [&str; 3] = ["felling_axe_head", "pick_head", "shovel_blade"];

/// Ingots one cast takes: the book's three for a head (`parts.rs`), so the two ways cost the same.
pub const CAST_INGOTS: u32 = 3;

/// A cast's real time: pouring and cooling, an hour or so.
pub const CAST_REAL_DAYS: f32 = 0.05;

/// A cast's seconds in the game, on the growing pace as the other crafts that wait.
pub fn cast_secs() -> f32 {
    GROWING.0 * CAST_REAL_DAYS.powf(GROWING.1) * DAY_SECS
}

/// A cast head's grade points: the mould's work (a machine's, `crafting::grades::MACHINE_WORK`)
/// and the hot furnace as its station. With copper's 20 material points: 52, a Good head, one up
/// from a cold-hammered one; the longer way pays.
pub const CAST_WORK: u32 = 22;
pub const CAST_STATION: u32 = 10;

/// How near a player stands to a furnace holding copper that isn't hot enough to be told why.
pub const HEAT_HINT_REACH: f64 = 8.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_charcoal_and_bellows_melt_copper() {
        assert!(needs_hot_fire("copper") && !needs_hot_fire("tin") && !needs_hot_fire("iron"));
        // Sticks 100, logs and planks 300, charcoal 1,600 (`items.rs`, `blocks.rs`, `recipes.rs`).
        assert!(!hot(300, true), "wood with bellows is not enough");
        assert!(!hot(1600, false), "charcoal without bellows is not enough");
        assert!(hot(1600, true));
    }

    #[test]
    fn a_cast_is_short() {
        let s = cast_secs();
        assert!((3.0..60.0).contains(&s), "a cast takes {s} s");
    }

    #[test]
    fn casting_beats_cold_hammering() {
        // Cold hammering at skill 0: work from memory 10, the hand's bonus 10, no station.
        let cold = crate::first::COPPER_HEAD_MATERIAL_POINTS + 10 + crate::first::BY_HAND_BONUS;
        let cast = crate::first::COPPER_HEAD_MATERIAL_POINTS + CAST_WORK + CAST_STATION;
        assert!(cast / 10 > cold / 10, "cast {cast} vs cold {cold}");
    }
}
