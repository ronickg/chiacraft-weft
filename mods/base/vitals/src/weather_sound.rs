//! How loud the weather is where the player stands (`notes/SYSTEMS.md`, weather sound), after
//! Vintage Story's model: a few looped beds whose volumes ease toward targets the weather sets,
//! muffled by how much the player is under cover; thunder a one-shot heard a distance's delay
//! after its flash. The weather itself (rain, thunder, wind and snow by biome and season) comes
//! from `climate` and `air` through the engine; this only turns it into levels.
//!
//! A block is a meter here, as everywhere in the game.

/// Speed of sound in air, m/s: thunder is heard `distance / SOUND_SPEED` seconds after its
/// flash (3 s a kilometre, the old rule of thumb).
pub const SOUND_SPEED: f32 = 343.0;

/// How far away lightning strikes, metres: never closer than this, never heard past the far
/// one (real thunder carries about 15 km; past 6 km it is too low and too late to place).
pub const STRIKE: (f32, f32) = (150.0, 6000.0);

/// Closer than this a strike cracks; farther it only rumbles (the air takes the highs first).
pub const CRACK_WITHIN: f32 = 1200.0;

/// Where a strike lands for a random `u` in 0..1: evenly over the ground around the player, so
/// far strikes are more common than near ones (the area grows with the distance squared).
pub fn strike_distance(u: f32) -> f32 {
    let (a, b) = (STRIKE.0 * STRIKE.0, STRIKE.1 * STRIKE.1);
    (a + (b - a) * u.clamp(0.0, 1.0)).sqrt()
}

/// Seconds between a strike's flash and its thunder.
pub fn thunder_delay(metres: f32) -> f32 {
    metres.max(0.0) / SOUND_SPEED
}

/// Thunder's volume at `metres`: full within 300 m, falling like a line source's (1/√d) past
/// it, not to nothing at the far edge.
pub fn thunder_volume(metres: f32) -> f32 {
    (300.0 / metres.max(300.0)).sqrt().clamp(0.0, 1.0)
}

/// Its low-pass cutoff in Hz: clear close by, about 400 Hz at 6 km.
pub fn thunder_lowpass(metres: f32) -> f32 {
    let t = ((metres - STRIKE.0) / (STRIKE.1 - STRIKE.0)).clamp(0.0, 1.0);
    20_000.0 * (400.0f32 / 20_000.0).powf(t.sqrt())
}

/// Its playback speed: a far one a little lower.
pub fn thunder_pitch(metres: f32) -> f32 {
    1.0 - 0.2 * ((metres - STRIKE.0) / (STRIKE.1 - STRIKE.0)).clamp(0.0, 1.0)
}

/// The cover over the player, as the ears hear it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cover {
    /// Share of the places sampled around the head that are open to the sky, 0..1: rain and
    /// wind are heard from there.
    pub open: f32,
    /// 0..1: a roof the rain falls on close above the head, 1 right on it (`ROOF_UP` blocks
    /// away or more: 0). Rain drums on it.
    pub roof: f32,
    /// Share of the sampled places with leaves overhead, 0..1: wind in the trees.
    pub leaves: f32,
}

/// Blocks above the head a roof still drums.
pub const ROOF_UP: i32 = 8;

/// How far around the head the sky is sampled, blocks (`Cover::open`).
pub const SAMPLE_RINGS: [i32; 2] = [3, 7];

/// Through walls, a bed is still heard at this share of its open level (a window, a door).
pub const THROUGH_WALLS: f32 = 0.2;

/// Low-pass cutoff for beds by how open it is: about 500 Hz shut in, clear in the open.
pub fn muffle(open: f32) -> f32 {
    500.0 * 40.0f32.powf(open.clamp(0.0, 1.0))
}

/// The wind is heard from this speed (m/s) and at full from that (a strong breeze to a gale).
pub const WIND_HEARD: (f32, f32) = (1.5, 16.0);

/// What the weather beds should play at now.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Levels {
    pub rain: f32,
    pub heavy: f32,
    pub roof: f32,
    pub wind: f32,
    pub trees: f32,
    /// Playback speed of both wind beds: higher in a gale.
    pub wind_pitch: f32,
    /// Low-pass cutoff for the beds heard from outside.
    pub lowpass: f32,
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The beds' levels from the weather where the player is (`rain` and `thunder` 0..1 as the
/// engine eases them, `wind` m/s, `snowing` 0..1) and the cover over them. Snow falls without a
/// sound; a thunderstorm's rain is heavy.
pub fn levels(rain: f32, thunder: f32, wind: f32, snowing: f32, cover: Cover) -> Levels {
    let wet = (rain * (1.0 - snowing)).clamp(0.0, 1.0);
    let heavy = thunder.clamp(0.0, 1.0);
    let heard = THROUGH_WALLS + (1.0 - THROUGH_WALLS) * cover.open;
    let blow = smooth(WIND_HEARD.0, WIND_HEARD.1, wind);
    // Indoors the wind is a far moan, but under trees it is all around.
    let wind_heard = blow * (0.15 + 0.85 * cover.open);
    Levels {
        rain: wet * (1.0 - heavy * 0.7) * heard,
        heavy: wet * heavy * heard,
        roof: wet * cover.roof * (1.0 - cover.open * 0.5) * (0.7 + 0.3 * heavy),
        wind: wind_heard * (1.0 - cover.leaves * 0.6),
        trees: wind_heard * cover.leaves,
        wind_pitch: 0.85 + 0.35 * blow,
        lowpass: muffle(cover.open),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thunder_comes_late_and_quiet_from_far() {
        assert!((thunder_delay(1000.0) - 2.915).abs() < 0.01);
        assert!(thunder_volume(STRIKE.1) < thunder_volume(STRIKE.0));
        assert!(thunder_lowpass(STRIKE.1) < 500.0 && thunder_lowpass(STRIKE.0) > 19_000.0);
        assert_eq!(strike_distance(0.0), STRIKE.0);
        assert!((strike_distance(1.0) - STRIKE.1).abs() < 0.5);
        // Evenly over the ground: half the strikes are past about 70% of the far edge.
        assert!(strike_distance(0.5) > STRIKE.1 * 0.7);
    }

    #[test]
    fn cover_muffles_and_snow_is_silent() {
        let open = Cover { open: 1.0, roof: 0.0, leaves: 0.0 };
        let shut = Cover { open: 0.0, roof: 1.0, leaves: 0.0 };
        let out = levels(1.0, 0.0, 0.0, 0.0, open);
        let inside = levels(1.0, 0.0, 0.0, 0.0, shut);
        assert!(out.rain > inside.rain && inside.roof > out.roof);
        assert!(inside.lowpass < 1000.0 && out.lowpass > 19_000.0);
        assert_eq!(levels(1.0, 0.0, 0.0, 1.0, open).rain, 0.0);
        let storm = levels(1.0, 1.0, 20.0, 0.0, open);
        assert!(storm.heavy > storm.rain && storm.wind > 0.9);
        assert_eq!(levels(0.0, 0.0, 0.5, 0.0, open).wind, 0.0);
    }
}
