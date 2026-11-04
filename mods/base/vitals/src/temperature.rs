//! Body temperature, for players and warm-blooded animals alike: the air where one stands, what
//! one feels of it (insulation, fortitude, fat, a hot meal, being wet), and the core temperature
//! moving toward where that leaves it. Cold-blooded animals take the air's temperature and slow
//! down below what they like.

use crate::stats;

/// Where one stands, as the world tells it.
#[derive(Clone, Copy, Debug)]
pub struct Place {
    /// The biome's mean temperature in °C.
    pub biome_c: f32,
    /// The biome's humidity, 0 dry to 1 wet: dry air swings more between day and night.
    pub humidity: f32,
    /// Metres above sea level.
    pub altitude: f32,
    /// The hour, 0 to 24.
    pub hour: f32,
    /// The sky overhead is open (the sun reaches).
    pub open_sky: bool,
    pub raining: bool,
    pub snowing: bool,
    pub in_water: bool,
    /// Degrees a fire adds here (`fire_heat`).
    pub fire: f32,
    /// Degrees the season adds to the biome's mean (`season`; 0 until worlds have seasons).
    pub season: f32,
    /// Deep under ground: the air keeps the year's mean, no day, sun or weather.
    pub underground: bool,
    /// The wind in m/s (at 10 m): out in the open it chills the cold (`air::feels`).
    pub wind: f32,
}

/// The air's temperature where one stands, in °C.
pub fn air(p: &Place) -> f32 {
    if p.underground {
        return p.biome_c + p.fire;
    }
    // Coldest before dawn, warmest mid afternoon; deserts swing far more than the sea: half a
    // day's range from 2.5 °C (an island in the Atlantic) to 9 (fitted to the real places,
    // `reports/ecology/Biome climates.md`).
    let swing = 2.5 + 6.5 * (1.0 - p.humidity.clamp(0.0, 1.0));
    let day = (std::f32::consts::TAU * (p.hour - 15.0) / 24.0).cos();
    let mut t = p.biome_c + p.season + swing * day - 0.0065 * p.altitude.max(0.0);
    if p.open_sky && (7.0..19.0).contains(&p.hour) && !p.raining && !p.snowing {
        t += 3.0 * day.max(0.0); // in the sun
    }
    if p.open_sky && p.raining {
        t -= 3.0;
    }
    if p.open_sky && p.snowing {
        t -= 5.0;
    }
    if p.open_sky && !p.in_water {
        t = crate::air::feels(t, p.wind);
    }
    if p.in_water {
        t = t.min(p.biome_c - 4.0); // water is cooler than the air, and takes heat fast
    }
    t + p.fire
}

/// The day's mean air where one stands: the biome's mean, the season, the height (what a room's
/// heavy walls hold to, `shelter::Room::air`).
pub fn day_mean(p: &Place) -> f32 {
    if p.underground { p.biome_c } else { p.biome_c + p.season - 0.0065 * p.altitude.max(0.0) }
}

/// Degrees the season adds to a biome's mean: `swing` its half range between summer and winter
/// (a taiga's 18, a jungle's 2), `year` 0 to 1 from midwinter (coldest at 0, warmest at 0.5).
pub fn season(swing: f32, year: f32) -> f32 {
    -swing * (std::f32::consts::TAU * year).cos()
}

/// How much each kind of fire heats the air near it, a share of lava's (`base_api::heat_of` names
/// the blocks): lava and a campfire 1, a lit furnace 0.7, a torch 0.25.
pub const LAVA_HEAT: f32 = 1.0;
pub const FURNACE_HEAT: f32 = 0.7;
pub const TORCH_HEAT: f32 = 0.25;
/// The heat a fire needs to rest by (camp's "near a fire", being rested): a torch's isn't a fire
/// (integration review: a torch-lit room gave rest).
pub const REST_HEAT: f32 = 0.7;
/// Blocks around a player searched for fires and comfort, once a second; animals, eggs and plants
/// search fewer (`FIRE_REACH`), as there are many more of them. A fire warms nothing past 8.
pub const PLAYER_FIRE_REACH: i32 = 7;
pub const FIRE_REACH: i32 = 3;

/// Degrees a fire adds `distance` blocks away: 20 °C beside it, nothing past 8 blocks (proposed).
pub fn fire_heat(distance: f32) -> f32 {
    20.0 * (1.0 - (distance - 1.0).max(0.0) / 7.0).clamp(0.0, 1.0)
}

/// What one has on and in one.
#[derive(Clone, Copy, Debug, Default)]
pub struct Body {
    /// Comfortable air, °C (a person 15 to 28 unclothed; `PERSON`).
    pub comfort: (f32, f32),
    /// Degrees of cold and heat insulation from what one wears (armor `set_insulation`, clothes
    /// `clothing::Worn`), or a coat; and the cold insulation still kept wet (wool keeps most of
    /// its warmth soaked, linen almost none).
    pub cold_insulation: f32,
    pub heat_insulation: f32,
    pub wet_cold_insulation: f32,
    /// The fortitude stat.
    pub fortitude: f32,
    /// Fat's fullness in the stomach, 0 to 1: up to 4 °C warmer in the cold.
    pub fat: f32,
    /// Degrees the warmest meal warms one (`Stomach::meal_warming`), and the coolest cools one
    /// (`meal_cooling`, 0 or less): cooling only widens the hot side, it doesn't chill one in the
    /// cold.
    pub meal: f32,
    pub meal_cool: f32,
    /// Wet: insulation is mostly lost.
    pub wet: bool,
}

pub const PERSON: (f32, f32) = (15.0, 28.0);
/// Degrees colder being wet feels, whatever one wears (water on the skin takes heat away).
pub const WET_CHILL: f32 = 6.0;
/// A body's core temperature at rest.
pub const CORE: f32 = 37.0;

impl Body {
    /// The air's range that feels comfortable to this body.
    pub fn comfort_range(&self) -> (f32, f32) {
        let wide = stats::fortitude_degrees(self.fortitude);
        let cold = if self.wet { self.wet_cold_insulation.min(self.cold_insulation) } else { self.cold_insulation };
        let lo = self.comfort.0 - wide - cold - 4.0 * self.fat.clamp(0.0, 1.0) - self.meal + if self.wet { WET_CHILL } else { 0.0 };
        let hi = self.comfort.1 + wide + self.heat_insulation - self.meal - self.meal_cool.min(0.0);
        (lo, hi)
    }

    /// The core temperature this air brings the body to, given time.
    pub fn target(&self, air: f32) -> f32 {
        let (lo, hi) = self.comfort_range();
        if air < lo {
            CORE - 0.35 * (lo - air)
        } else if air > hi {
            CORE + 0.3 * (air - hi)
        } else {
            CORE
        }
    }
}

/// The core after `secs` toward `target`: half way in a minute for a 70 kg body, slower for a
/// bigger one (a quarter power of the mass, as heat flows through skin).
pub fn step_core(core: f32, target: f32, secs: f32, mass_kg: f32) -> f32 {
    let half = 60.0 * (mass_kg.max(0.1) / 70.0).powf(0.25);
    core + (target - core) * (1.0 - 0.5f32.powf(secs / half))
}

/// How the core temperature feels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feeling {
    Freezing,
    Cold,
    Chilly,
    Comfortable,
    Warm,
    Hot,
    Heatstroke,
}

/// From the core temperature: chilly below 36.5 °C, cold below 35 (hypothermia), freezing below
/// 33; warm over 37.5, hot over 38.5, heatstroke over 40, as medicine draws the lines.
pub fn feeling(core: f32) -> Feeling {
    match core {
        c if c < 33.0 => Feeling::Freezing,
        c if c < 35.0 => Feeling::Cold,
        c if c < 36.5 => Feeling::Chilly,
        c if c > 40.0 => Feeling::Heatstroke,
        c if c > 38.5 => Feeling::Hot,
        c if c > 37.5 => Feeling::Warm,
        _ => Feeling::Comfortable,
    }
}

/// Cold and heat insulation of a full set of armor of that material (proposed): leather is hide
/// (`clothing::armor_cloth`), metal cold to wear in snow and hot in the sun. A piece gives its share (`PIECE_SHARE`). Fur,
/// wool, hide and linen are clothes (`clothing`), worn under armor.
pub fn set_insulation(material: &str) -> (f32, f32) {
    match material {
        "leather" => crate::clothing::armor_cloth(material).map_or((0.0, 0.0), |m| (m.cold(), m.heat)),
        "iron" | "chainmail" => (4.0, -4.0),
        "gold" => (3.0, -5.0),
        "diamond" => (5.0, -2.0),
        _ => (0.0, 0.0),
    }
}

/// Each piece's share of a set's insulation: helmet, chestplate, leggings, boots (and a hat, a
/// coat, trousers, boots of clothes).
pub const PIECE_SHARE: [f32; 4] = [0.15, 0.4, 0.3, 0.15];
/// The share of armor's cold insulation kept wet.
pub const ARMOR_WET_KEEPS: f32 = 0.35;

/// The air one feels out in the wind, through clothes that stop `windproof` of it (0 to 1): the
/// wind chill (`chilled`, `air::feels`) only on what still gets through to the skin.
pub fn through_clothes(still: f32, chilled: f32, windproof: f32) -> f32 {
    chilled + (still - chilled) * windproof.clamp(0.0, 1.0)
}

/// A cold-blooded animal's speed in that air: full at its liking and above, slower below, down to
/// 40% fifteen degrees colder.
pub fn ectotherm_speed(air: f32, likes: f32) -> f32 {
    (1.0 - 0.6 * ((likes - air) / 15.0).clamp(0.0, 1.0)).clamp(0.4, 1.0)
}
