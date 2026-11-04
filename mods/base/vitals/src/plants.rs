//! Every growing plant's facts from real agronomy (`reports/ecology/Plants and crops.md`, its
//! numbers in `plants.json`), and one rule for all of them (`notes/ECOLOGY.md`, part 3): a growth
//! step goes ahead at the rate warmth × water, each 0 to 1, from the air and the ground where the
//! plant stands. So the season decides whether a plant grows (winter or a dry season rests it)
//! and the growing pace how fast (`GROWING`). The base mod asks `rate` at each growth step
//! (`mods/base/src/growth.rs`).

use crate::climate::{self, Climate};
use crate::{DAY_SECS, calendar, temperature};

/// How much water a plant needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Water {
    /// Desert plants: they grow at half speed on dry ground.
    Dry,
    Moderate,
    /// Water plants: only on wet ground or by water.
    Wet,
}

/// A plant that grows, by its block.
#[derive(Clone, Copy, Debug)]
pub struct Plant {
    /// The block's full name.
    pub block: &'static str,
    /// The real plant its numbers come from (`plants.json`).
    pub real: &'static str,
    /// The air it grows in, °C: from `base`, best between `best`, nothing past `max`.
    pub base: f32,
    pub best: (f32, f32),
    pub max: f32,
    pub water: Water,
    /// Real days from sowing to harvest (a crop), or from flower to ripe fruit (a bush), at a
    /// field's usual warmth and water; 0: the game's own time (trees and columns, whose real
    /// years would take hours of play).
    pub days: f32,
    /// The seasons its last stage (the ripe fruit) can come in (`calendar` order as bits, spring
    /// 1); all four: any.
    pub ripens: u8,
    /// The air, °C, that kills it (a crop), its fruit (a bush) or its stalk down to the base (cane,
    /// cactus), from its research; `HARDY`: frost never hurts it.
    pub frost: f32,
}

/// Frost never hurts it (an oak sapling, the grass).
pub const HARDY: f32 = f32::NEG_INFINITY;

const fn p(
    block: &'static str,
    real: &'static str,
    warmth: (f32, f32, f32, f32),
    water: Water,
    days: f32,
    ripens: u8,
    frost: f32,
) -> Plant {
    Plant { block, real, base: warmth.0, best: (warmth.1, warmth.2), max: warmth.3, water, days, ripens, frost }
}

const ANY: u8 = 15;
const SUMMER: u8 = 2;
const SUMMER_AUTUMN: u8 = 2 | 4;

pub const PLANTS: [Plant; 43] = [
    // Frost (`reports/ecology/Plants and crops.md`, A1): spring wheat is sensitive (a hard frost,
    // about -2 °C, kills it; winter wheat, hardy, comes later); a blueberry's dormant bush is
    // hardy but its flowers and fruit die at -2; sugar cane takes none (the stalk dies, the base
    // sprouts again); a saguaro takes brief frost down to about -7.
    p("base:wheat", "spring wheat", (0.0, 15.0, 23.0, 34.0), Water::Moderate, 110.0, ANY, -2.0),
    // The crops (`crops::CROPS`), the research's middle days. Cool-season ones grow from about
    // freezing (barley, rye) or 4-7 °C (carrot, beetroot, potato) and suffer past 30; warm-season
    // ones need about 10 °C and die at a frost; sorghum and pearl millet take drought (half speed
    // on dry ground), paddy rice grows only by water. Winter rye is hardy to -30 and needs a
    // winter's cold to set grain: sown in autumn, it ripens only in summer. Cassava takes 8-18
    // months.
    p("base:barley", "barley", (0.0, 15.0, 20.0, 32.0), Water::Moderate, 105.0, ANY, -4.0),
    p("base:rye", "winter rye", (0.0, 12.0, 20.0, 30.0), Water::Moderate, 300.0, SUMMER, -30.0),
    p("base:carrots", "carrot", (4.0, 16.0, 21.0, 30.0), Water::Moderate, 95.0, ANY, -5.0),
    p("base:potatoes", "potato", (7.0, 15.0, 20.0, 30.0), Water::Moderate, 135.0, ANY, -1.0),
    p("base:beetroots", "beetroot", (4.5, 15.0, 20.0, 30.0), Water::Moderate, 72.0, ANY, -3.0),
    p("base:maize", "maize", (10.0, 24.0, 30.0, 38.0), Water::Moderate, 150.0, ANY, 0.0),
    p("base:paddy_rice", "paddy rice", (10.0, 25.0, 30.0, 35.0), Water::Wet, 120.0, ANY, 0.0),
    p("base:beans", "common bean", (10.0, 18.0, 25.0, 30.0), Water::Moderate, 100.0, ANY, 0.0),
    p("base:sorghum", "sorghum", (8.0, 27.0, 34.0, 42.0), Water::Dry, 125.0, ANY, 0.0),
    p("base:millet", "pearl millet", (10.0, 30.0, 34.0, 45.0), Water::Dry, 105.0, ANY, 0.0),
    p("base:cassava", "cassava", (12.0, 25.0, 29.0, 37.0), Water::Moderate, 390.0, ANY, 0.0),
    // Pumpkin and melon stems (the research's rows): warm-season, killed by frost; the fruit sets
    // in summer and autumn (`SUMMER_AUTUMN`, the stem's last step).
    p("base:pumpkin_stem", "pumpkin, squash", (10.0, 20.0, 30.0, 35.0), Water::Moderate, 110.0, SUMMER_AUTUMN, 0.0),
    p("base:melon_stem", "melon", (12.0, 25.0, 30.0, 38.0), Water::Moderate, 140.0, SUMMER_AUTUMN, 0.0),
    p("base:berry_bush", "blueberry", (7.0, 20.0, 26.0, 32.0), Water::Moderate, 75.0, SUMMER_AUTUMN, -2.0),
    p("base:sugar_cane", "sugar cane", (12.0, 22.0, 30.0, 38.0), Water::Wet, 0.0, ANY, -1.0),
    // Kelp (Laminaria, Saccharina): cold-water seaweed, growing best at 5-15 °C and failing
    // past about 20; it stands under water, so frost never reaches it.
    p("base:kelp", "kelp (Laminaria)", (0.0, 5.0, 15.0, 22.0), Water::Wet, 0.0, ANY, HARDY),
    p("base:cactus", "saguaro", (10.0, 25.0, 40.0, 50.0), Water::Dry, 0.0, ANY, -7.0),
    p("base:sapling", "oak", (5.0, 18.0, 28.0, 38.0), Water::Moderate, 0.0, ANY, HARDY),
    // Grazed grass growing back to tall grass: 15-21 days in spring (`vitals::forage`).
    p("base:grazed_grass", "prairie grass", (5.0, 15.0, 30.0, 40.0), Water::Moderate, 18.0, ANY, HARDY),
    // The wild plants (`vitals::wild`; the research's estimates where it has no source). A tuft
    // or a mat grazed down grows back in `days`: sedges and cottongrass as grass, white clover in
    // 20-40 days, reindeer lichen in about ten years (it grows millimetres a year and takes
    // decades to recover). The research's warmths are estimates; the tundra's own plants are
    // set to grow from about freezing, as arctic plants do, so its five-degree summer grows them.
    // Sedges are wetland plants, but they stand on the open ground here, not at the water's edge:
    // rain waters them, so a dry season rests them as it does grass.
    p("base:sedge", "sedges", (-1.0, 8.0, 22.0, 32.0), Water::Moderate, 18.0, ANY, HARDY),
    p("base:cottongrass", "cottongrass (Eriophorum)", (-2.0, 6.0, 16.0, 25.0), Water::Moderate, 18.0, ANY, HARDY),
    p("base:reindeer_lichen", "reindeer lichen (Cladonia)", (0.0, 5.0, 15.0, 25.0), Water::Dry, 3650.0, ANY, HARDY),
    p("base:clover", "white clover", (5.0, 18.0, 25.0, 32.0), Water::Moderate, 30.0, ANY, HARDY),
    // Bushes, flower to ripe fruit: lingonberries flower in June and ripen August to October;
    // cloudberries, bog plants, June to July and August, their flowers killed by a June frost;
    // crowberries, whose fruit holds over winter; hazelnuts from catkins in February and March to
    // nuts in August to October; brambles' raspberries and blackberries July to September.
    p("base:lingonberry_bush", "lingonberry", (5.0, 12.0, 20.0, 28.0), Water::Moderate, 75.0, SUMMER_AUTUMN, -3.0),
    p("base:cloudberry_bush", "cloudberry", (0.0, 8.0, 16.0, 25.0), Water::Wet, 45.0, SUMMER, -2.0),
    p("base:crowberry_bush", "crowberry", (-1.0, 6.0, 15.0, 25.0), Water::Moderate, 60.0, SUMMER_AUTUMN, HARDY),
    p("base:hazel_bush", "hazel", (5.0, 16.0, 24.0, 32.0), Water::Moderate, 180.0, SUMMER_AUTUMN, HARDY),
    p("base:bramble_bush", "raspberry and blackberry", (5.0, 16.0, 25.0, 32.0), Water::Moderate, 60.0, SUMMER_AUTUMN, -2.0),
    // Fruit and seed in their season: prickly pears flower in April and May and ripen July to
    // September, and frost below about -10 kills the pads; wild sunflowers take 125-130 days and
    // die at a light frost; the Pantanal's wild rices seed in 120-180 days as the flood peaks.
    p("base:prickly_pear", "prickly pear (Opuntia)", (10.0, 20.0, 35.0, 45.0), Water::Dry, 90.0, SUMMER_AUTUMN, -10.0),
    p("base:wild_sunflower", "wild sunflower", (6.0, 20.0, 28.0, 38.0), Water::Dry, 125.0, SUMMER_AUTUMN, -2.0),
    p("base:wild_rice", "wild rice (Oryza latifolia)", (12.0, 25.0, 32.0, 38.0), Water::Wet, 150.0, SUMMER_AUTUMN, 0.0),
    // Spreading: mushrooms come up after late-summer rain and last days (3-14 to fruit); water
    // hyacinth doubles in one to two weeks in warm water and dies at a frost.
    p("base:mushroom_brown", "forest mushrooms", (5.0, 10.0, 20.0, 30.0), Water::Moderate, 8.0, SUMMER_AUTUMN, -2.0),
    p("base:mushroom_red", "fly agaric", (5.0, 10.0, 20.0, 30.0), Water::Moderate, 8.0, SUMMER_AUTUMN, -2.0),
    p("base:water_hyacinth", "water hyacinth", (12.0, 25.0, 30.0, 38.0), Water::Wet, 10.0, ANY, -1.0),
    // Our trees' saplings, at the game's time as the oak's: mesquite, a desert legume that a
    // frost below about -15 kills; the tropical palms, killed by any frost.
    p("base:mesquite_sapling", "mesquite", (12.0, 25.0, 38.0, 48.0), Water::Dry, 0.0, ANY, -15.0),
    p("base:acuri_palm_sapling", "acuri palm (Attalea phalerata)", (15.0, 24.0, 32.0, 40.0), Water::Moderate, 0.0, ANY, 0.0),
    // The giant sequoia: the Sierra Nevada's western slopes at 1,400-2,000 m, growing from about
    // 5 degrees, best in a mild summer, hardy to about -24 under snow; its seedlings need the
    // soil kept moist (Weatherspoon 1990, Silvics of North America).
    p("base:sequoia_sapling", "giant sequoia (Sequoiadendron giganteum)", (5.0, 15.0, 24.0, 35.0), Water::Moderate, 0.0, ANY, -24.0),
    p("base:moriche_palm_sapling", "moriche palm (Mauritia flexuosa)", (15.0, 24.0, 32.0, 40.0), Water::Wet, 0.0, ANY, 0.0),
    // The Cretaceous's saplings (the animals mod's), at the game's time: its greenhouse warmth
    // suits them all; living kin give the frost they take (Araucaria about -5, ginkgo hardy, the
    // bald cypress about -15, the scale-leaved conifers estimated as araucarias, drought-hardy).
    p("animals:araucaria_sapling", "araucarian conifer", (5.0, 18.0, 28.0, 38.0), Water::Moderate, 0.0, ANY, -5.0),
    p("animals:ginkgo_sapling", "ginkgo", (5.0, 15.0, 26.0, 36.0), Water::Moderate, 0.0, ANY, HARDY),
    p("animals:swamp_cypress_sapling", "swamp cypress (Taxodioxylon)", (8.0, 20.0, 30.0, 40.0), Water::Wet, 0.0, ANY, -15.0),
    p("animals:scale_conifer_sapling", "scale-leaved conifer (Cheirolepidiaceae)", (8.0, 20.0, 32.0, 42.0), Water::Dry, 0.0, ANY, -5.0),
];

/// Whether air at `air` °C (a night's frost where it stands) hurts it (`Plant::frost`).
pub fn frosted(p: &Plant, air: f32) -> bool {
    air < p.frost
}

/// The coldest air of the middle day of `season` in a biome's climate, °C (its nights' low).
pub fn season_low(c: &Climate, season: usize) -> f32 {
    let day = (season as u32 * calendar::SEASON_DAYS + calendar::SEASON_DAYS / 2) as f32;
    (0..24)
        .map(|h| climate::air(c, day, h as f32 + 0.5))
        .fold(f32::INFINITY, f32::min)
}

/// The plant that grows as this block.
pub fn of(block: &str) -> Option<&'static Plant> {
    PLANTS.iter().find(|p| p.block == block)
}

/// The growing pace: game days = a · real days^k, the pace the species grow up by
/// (`animals_core::biology::GROWING`, held equal by `mods/animals/core/tests/places.rs`), so a
/// field and a litter keep their real proportions.
pub const GROWING: (f32, f32) = (0.0645, 0.7);

/// The real days are a field's in its season; at the best warmth and water a plant grows this
/// much faster.
pub const BEST_OVER_USUAL: f32 = 1.0 / 0.75;

/// Game seconds a growth step takes on average at the rate 1, over `steps` steps to harvest;
/// `None`: the game's own time.
pub fn step_secs(p: &Plant, steps: u32) -> Option<f32> {
    (p.days > 0.0).then(|| GROWING.0 * p.days.powf(GROWING.1) * DAY_SECS / steps as f32 / BEST_OVER_USUAL)
}

/// How warm enough it is, 0 to 1, in the day's mean air: rising from `base` to the best, falling
/// to nothing at `max`.
pub fn warmth(p: &Plant, air: f32) -> f32 {
    if air <= p.base || air >= p.max {
        0.0
    } else if air < p.best.0 {
        (air - p.base) / (p.best.0 - p.base)
    } else if air <= p.best.1 {
        1.0
    } else {
        (p.max - air) / (p.max - p.best.1)
    }
}

/// How well watered, 0 to 1, on ground that wet (`soil_water`).
pub fn watered(p: &Plant, soil: f32) -> f32 {
    let ramp = |from: f32, to: f32| ((soil - from) / (to - from)).clamp(0.0, 1.0);
    match p.water {
        Water::Dry => 0.5 + 0.5 * ramp(0.0, 0.2),
        Water::Moderate => ramp(0.05, 0.4),
        Water::Wet => ramp(0.3, 0.7),
    }
}

/// The chance a growth step goes ahead: warmth × water.
pub fn rate(p: &Plant, air: f32, soil: f32) -> f32 {
    warmth(p, air) * watered(p, soil)
}

/// Whether its last stage may come in `season` (0 spring; `None`: no seasons there).
pub fn ripens_in(p: &Plant, season: Option<usize>) -> bool {
    season.is_none_or(|s| p.ripens >> s & 1 != 0)
}

/// A day's mean air at sea level on day `day` of the year (from 0, with its fraction).
pub fn mean_air(c_celsius: f32, c_seasons: f32, day: f32) -> f32 {
    c_celsius + temperature::season(c_seasons, calendar::from_midwinter(day))
}

/// Days of rain the ground remembers.
pub const SOIL_DAYS: u64 = 8;

/// How wet a biome's ground is on `day` (days since the world began), 0 dry to 1 soaked: the last
/// `SOIL_DAYS` days' rain (`climate::spell`, a longer spell more), each day's drying faster in
/// warmer air. Worked out from the rain schedule, so nothing is stored. `calendar`: whether the
/// dimension has seasons. Ground by water is soaked whatever the weather (the game's side).
pub fn soil_water(rain: [f32; 4], celsius: f32, seasons: f32, calendar: bool, key: u64, day: u64) -> f32 {
    let mut wet = 0.0;
    let mut keep = 1.0;
    for back in 0..SOIL_DAYS.min(day + 1) {
        let d = day - back;
        let of_year = (d % calendar::YEAR_DAYS as u64) as f32;
        let season = calendar.then_some((of_year as u32 / calendar::SEASON_DAYS) as usize);
        if let Some((from, to)) = climate::spell(rain, season, key, d) {
            wet += keep * ((to - from) / climate::SPELL_HOURS.1).max(0.5);
        }
        // A day keeps 80% of the water at 0 °C, a third at 30 °C.
        let air = if calendar { mean_air(celsius, seasons, of_year + 0.5) } else { celsius };
        keep *= (0.8 - 0.015 * air.max(0.0)).clamp(0.33, 0.8);
    }
    (wet / 1.5).min(1.0)
}

/// A plant's rate there through season `season` (0 spring), on average: the middle day's air and
/// the season's soil, for the sheet and the checks.
pub fn season_rate(p: &Plant, c: &Climate, season: usize) -> f32 {
    season_rate_in(p, c.celsius, c.seasons, c.rain, season)
}

/// `season_rate` in a climate of that mean, seasons and rain (a biome's `base_api::Climate`).
pub fn season_rate_in(p: &Plant, celsius: f32, seasons: f32, rain: [f32; 4], season: usize) -> f32 {
    let first = season as u64 * calendar::SEASON_DAYS as u64;
    let days = first + calendar::YEAR_DAYS as u64..first + calendar::YEAR_DAYS as u64 + calendar::SEASON_DAYS as u64;
    let n = days.end - days.start;
    days.map(|d| {
        let air = mean_air(celsius, seasons, (d % calendar::YEAR_DAYS as u64) as f32 + 0.5);
        let soil = if p.water == Water::Wet { 1.0 } else { soil_water(rain, celsius, seasons, true, 7, d) };
        rate(p, air, soil)
    })
    .sum::<f32>()
        / n as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warmth_and_water_shape_the_rate() {
        let wheat = of("base:wheat").unwrap();
        assert_eq!(rate(wheat, 18.0, 0.8), 1.0);
        assert_eq!(rate(wheat, -2.0, 1.0), 0.0, "no growth in frost");
        assert_eq!(rate(wheat, 18.0, 0.0), 0.0, "no growth on dry ground");
        assert!(rate(wheat, 7.5, 1.0) > 0.4 && rate(wheat, 7.5, 1.0) < 0.6);
    }

    #[test]
    fn rain_wets_the_ground_and_heat_dries_it() {
        let wet = climate::of("base:savanna").unwrap();
        let dry = climate::of("base:desert").unwrap();
        let soil = |c: &Climate, day: u64| soil_water(c.rain, c.celsius, c.seasons, true, 3, day);
        // The llanos in their rains (summer) against the desert's dry spring.
        let summer = (calendar::YEAR_DAYS + calendar::SEASON_DAYS + 7) as u64;
        let spring = (calendar::YEAR_DAYS + 7) as u64;
        assert!(soil(wet, summer) > soil(dry, spring) + 0.2);
    }
}
