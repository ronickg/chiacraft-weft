//! Typed handles and shared value types for the base game. Add `base_api` to your mod's
//! `Cargo.toml` to use them. The engine checks shared value layouts when mods load.
//!
//! Content handles use explicit `base:` names so they resolve to the base game's entries
//! in every mod. Shared vocabulary such as the `fuel` component keeps its unqualified name.

use weft_sdk::*;

/// Shared components, their value types, and food-item helpers.
pub mod components;

/// Events emitted by the base game.
pub mod events;

// Ours: the land, trees and skills the game shares with other mods.
pub mod branchy;
pub mod colonize;
pub mod grow;
pub mod land;
pub mod skills;
pub mod tree_recipes;
pub mod trees;

/// Item and block tags exposed by the base game.
pub mod tags;

/// Attributes exposed by the base game.
pub mod attributes;

/// Portions a stomach holds (`vitals::stomach::PORTIONS`).
pub const PORTIONS: usize = vitals::stomach::PORTIONS;
/// The engine counts health as the base game's 20 points, which keeps every creature's damage
/// in balance; the vitals count 100 (`notes/SURVIVAL.md`). A health point is this many.
pub const HEALTH_SCALE: f32 = 5.0;
/// A new player's stamina before the first tick sets its max (`survival/stamina.rs` clamps it):
/// over any max, so one starts full. Dying still leaves one at 0.
pub const NEW_STAMINA: f32 = 1.0e6;

value! {
    #[derive(Default)]
    /// A player's survival state (`notes/SURVIVAL.md`, the `vitals` crate): stamina, what's in the
    /// stomach, body temperature, effects, breath, XP and the points placed. Kept through death;
    /// dying empties the stomach and costs XP.
    pub struct Vitals {
        pub stamina: f32,
        /// Max health and stamina as food has them now, in the vitals' units (the HUD's numbers).
        pub max_health: f32,
        pub max_stamina: f32,
        /// Ticks before stamina comes back after spending it.
        pub rest_wait: u32,
        /// The stomach, a portion each: the food (`vitals::food::FOODS` index + 1, 0 empty), units
        /// left, quality, and the warmth of a hot (cold) meal with its seconds left.
        pub foods: [u16; PORTIONS],
        pub units: [f32; PORTIONS],
        pub quality: [f32; PORTIONS],
        pub warmth: [f32; PORTIONS],
        pub warm_secs: [f32; PORTIONS],
        /// Core temperature, °C (37 at rest).
        pub core: f32,
        /// The air's temperature where one stands, °C (`vitals::temperature::air`).
        pub outside: f32,
        /// Seconds left wet.
        pub wet: f32,
        /// A bit for each effect on, in the order of `vitals::effects::ALL`.
        pub effects: u32,
        pub air: f32,
        /// XP since level 1, and the points placed in each stat.
        pub xp: u64,
        pub points: [u16; vitals::stats::STATS],
        /// Seconds left rested, seconds rested toward it (`survival::REST_SECS`), and the comfort
        /// around one (1, and one more for each kind of furniture near).
        pub rested: f32,
        pub resting: f32,
        pub comfort: u8,
        /// Seconds left of food poisoning.
        pub sick: f32,
        /// Seconds left poisoned and bleeding; torpor (sedatives, blows), knocked out (1) once it
        /// passes the torpor stat until it falls under half; kilograms carried; the speed last set
        /// on the body (`survival/load.rs`).
        pub poison: f32,
        pub bleed: f32,
        pub torpor: f32,
        pub out: u8,
        pub load: f32,
        pub speed: f32,
        /// Today in the calendar of the dimension one is in: the day of the year + 1 (0: no
        /// calendar there), and its kind (`SEASONS`).
        pub date: u16,
        pub calendar: u8,
        /// The level last announced (a level up shows when XP passes it).
        pub level: u16,
        /// Biomes and dimensions visited, a bit each by id (exploring gives XP the first time).
        pub visited: [u32; 8],
        pub dimensions: u32,
    }
}
pub static VITALS: Component<Vitals> = Component::named("base:vitals");

value! {
    /// A biome's climate, as a real place of its kind has it: the year's mean temperature in °C,
    /// humidity 0 dry to 1 wet (dry air swings more between day and night), half the range
    /// between summer and winter, and the share of days with rain in each season (spring
    /// first). Set on biomes (`r.biome(..).set(&CLIMATE, ..)`); body temperature and the weather
    /// read it (`notes/SURVIVAL.md`, `notes/ECOLOGY.md`). And its air (`vitals::air`): the mean
    /// wind in m/s and the days with fog in each season, and the kind of fog
    /// (`vitals::climate::Fog::index`).
    pub struct Climate { pub celsius: f32, pub humidity: f32, pub seasons: f32, pub rain: [f32; 4], pub wind: [f32; 4], pub fog: [f32; 4], pub fog_kind: u8 }
}
pub static CLIMATE: Component<Climate> = Component::named("base:climate");
/// The calendar a dimension has (`vitals::calendar`): spring, summer, autumn and winter, or a
/// wet and a dry season (the wet one in the year's first half). Without it, no seasons there. Set
/// on dimensions (`r.dimension(..).set(&SEASONS, ..)`, with `.seasons(..)` for the day's length).
pub static SEASONS: Component<u8> = Component::named("base:seasons");
pub mod calendars {
    pub const FOUR: u8 = 1;
    pub const WET_AND_DRY: u8 = 2;
    /// How much longer than half the day midsummer's daylight is (`.seasons(..)`'s swing): four
    /// seasons sixteen hours to eight; wet and dry, near the tropics, barely changing.
    pub const FOUR_SWING: f32 = 0.17;
    pub const WET_AND_DRY_SWING: f32 = 0.03;

    pub fn swing(calendar: u8) -> f32 {
        match calendar {
            FOUR => FOUR_SWING,
            WET_AND_DRY => WET_AND_DRY_SWING,
            _ => 0.0,
        }
    }
}

/// A biome's climate from the table (`vitals::climate::BIOMES`), by its full name, for setting
/// on it: every biome's numbers in one place, where the checks across systems read them.
pub fn climate(biome: &str) -> Climate {
    let c = vitals::climate::of(biome).unwrap_or_else(|| panic!("no climate for {biome} in vitals::climate::BIOMES"));
    Climate {
        celsius: c.celsius,
        humidity: c.humidity,
        seasons: c.seasons,
        rain: c.rain,
        wind: c.wind,
        fog: c.fog,
        fog_kind: c.fog_kind.index(),
    }
}

/// A biome without one: a mild, temperate place, raining a quarter of the days, a light breeze
/// and a few foggy mornings.
pub const TEMPERATE: Climate =
    Climate { celsius: 12.0, humidity: 0.5, seasons: 10.0, rain: [0.25; 4], wind: [4.0; 4], fog: [8.0; 4], fog_kind: 0 };

/// How hard survival is in this world (`vitals::difficulty::ALL`: gentle, normal, harsh), a
/// game rule picked when the world is made and changed with `/rule difficulty`.
pub static DIFFICULTY: Config<u32> = Config::named("base:difficulty");

/// This world's difficulty.
pub fn difficulty() -> vitals::difficulty::Difficulty {
    vitals::difficulty::ALL[(DIFFICULTY.get() as usize).min(vitals::difficulty::ALL.len() - 1)]
}

/// The overworld's sea level: heights for the air's temperature are counted from it.
pub const SEA: i32 = 63;
/// Metres a block of height stands for in the air's temperature: the game's mountains are a few
/// hundred blocks where real ones are kilometres (proposed).
const ALTITUDE: f32 = 6.0;

/// The air where a body stands, and what made it so (`air_at`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Air {
    pub celsius: f32,
    /// The same air out of the wind: what it is through windproof clothes
    /// (`vitals::temperature::through_clothes`).
    pub still: f32,
    /// The biome's humidity, 0 dry to 1 wet (drying racks).
    pub humidity: f32,
    /// The day's mean air here (`temperature::day_mean`): what a room's walls hold to.
    pub mean: f32,
    /// Degrees from the nearest fire (lava, a lit furnace, torches), and from the nearest one hot
    /// enough to rest by (`temperature::REST_HEAT`: not a torch).
    pub fire: f32,
    pub hearth: f32,
    /// Sky light at the eyes: under 13 a roof, 0 deep inside.
    pub sky: u32,
    pub underground: bool,
    pub open_sky: bool,
    pub raining: bool,
    pub snowing: bool,
    /// The weather's air in this biome (`World::air_at`): the wind in m/s (it chills the cold out in
    /// the open), the fog 0..1 (it shortens how far animals see, `sight`) and the snow lying.
    pub wind: f32,
    pub fog: f32,
    pub snow: f32,
    /// Today in the dimension's calendar (`vitals::calendar`), and its kind (0: none).
    pub day: u16,
    pub calendar: u8,
}

/// The air's temperature where a body `height` blocks tall stands at `pos`, the same for players,
/// animals and eggs: the biome's climate (`CLIMATE`) and season, the hour, height, sun and shade,
/// rain or snow, water, caves, and fires within `reach` blocks. `each` sees every block searched
/// (the player's comfort). `vitals::temperature::air` holds the rules.
pub fn air_at(w: &World, pos: [f64; 3], height: f32, in_water: bool, reach: i32, mut each: impl FnMut(u32)) -> Air {
    use vitals::{calendar, temperature};
    let feet = pos.map(|v| v.floor() as i32);
    let eye = [feet[0], (pos[1] + height as f64).floor() as i32, feet[2]];
    let around = around(w, feet);
    let climate = around.climate;
    let sky = w.sky_light(eye);
    let underground = sky == 0 && feet[1] < SEA - 6;
    let mut fire = 0.0f32;
    let mut hearth = 0.0f32;
    for dx in -reach..=reach {
        for dy in -reach..=reach {
            for dz in -reach..=reach {
                let state = w.block([feet[0] + dx, feet[1] + dy, feet[2] + dz]);
                each(state);
                let h = heat_of(state);
                if h > 0.0 {
                    let d = ((dx * dx + dy * dy + dz * dz) as f32).sqrt();
                    fire = fire.max(temperature::fire_heat(d) * h);
                    if h >= temperature::REST_HEAT {
                        hearth = hearth.max(temperature::fire_heat(d) * h);
                    }
                }
            }
        }
    }
    let hour = ((w.day_time() % 24000) as f32 / 1000.0 + 6.0) % 24.0;
    // Today, in this dimension's calendar (none: always a spring or autumn day).
    let kind = SEASONS.on_dimension(w.dimension()).unwrap_or(0);
    let date = calendar::date(w.day_time());
    let year = if kind > 0 { date.from_midwinter() } else { 0.25 };
    // In a forest the wind drops, and a blizzard with it (`vitals::air::shelter`).
    let calm = vitals::air::sheltered_wind(around.wind, vitals::air::shelter(around.canopy, cover(w, eye)));
    let snowing_here = around.flags & air_flags::SNOWING != 0;
    let fog = vitals::air::sheltered_fog(around.fog, around.wind, calm, snowing_here);
    let mut place = temperature::Place {
        biome_c: climate.celsius,
        humidity: climate.humidity,
        altitude: (feet[1] - SEA) as f32 * ALTITUDE,
        hour,
        open_sky: sky >= 15,
        // Rain by biome and season (`weather.rs`): here, not just somewhere in the dimension.
        raining: w.raining_at(eye),
        snowing: false,
        in_water,
        fire,
        season: temperature::season(climate.seasons, year),
        underground,
        wind: calm,
    };
    if place.raining && temperature::air(&place) < 1.0 {
        (place.raining, place.snowing) = (false, true);
    }
    Air {
        celsius: temperature::air(&place),
        still: temperature::air(&temperature::Place { wind: 0.0, ..place }),
        humidity: climate.humidity,
        mean: temperature::day_mean(&place),
        fire,
        hearth,
        sky,
        underground,
        open_sky: place.open_sky,
        raining: place.raining,
        snowing: place.snowing,
        wind: calm,
        fog,
        snow: around.snow,
        day: if kind > 0 { date.day as u16 + 1 } else { 0 },
        calendar: kind,
    }
}

/// How far an animal sees at `pos` against clear air, 1 to `vitals::air::FOG_SIGHT`: fog, a
/// blizzard or a dust storm (`World::air_at`) close it in; a blizzard less inside a forest.
pub fn sight(w: &World, pos: [f64; 3]) -> f32 {
    let p = pos.map(|v| v.floor() as i32);
    let a = w.air_at(p);
    if a.flags & air_flags::SNOWING == 0 {
        return vitals::air::sight(a.fog);
    }
    let calm = vitals::air::sheltered_wind(a.wind, vitals::air::shelter(canopy(w.biome(p)), cover(w, p)));
    vitals::air::sight(vitals::air::sheltered_fog(a.fog, a.wind, calm, true))
}

/// The air round a place, blended over its neighbours (`vitals::air::AIR_BLEND`): the climate's
/// temperature, humidity and seasons, the weather's wind, fog and snow lying, and how closed the
/// canopy is. Rain and the rest of the climate are the middle's.
pub struct Around {
    pub climate: Climate,
    pub wind: f32,
    pub fog: f32,
    pub snow: f32,
    /// The middle's `air_flags` (snowing, dust).
    pub flags: u32,
    pub canopy: f32,
}

/// The air round `pos` (`Around`): 9 biome and weather looks, so a body crossing a border feels
/// the air turn over `vitals::air::AIR_BLEND` blocks instead of at the line.
pub fn around(w: &World, pos: [i32; 3]) -> Around {
    use vitals::air::{AIR_BLEND, blend, ring};
    let points = ring(pos[0], pos[2], AIR_BLEND / 2).map(|(x, z)| {
        let p = [x, pos[1], z];
        let id = w.biome(p);
        (id, CLIMATE.on_biome(id).unwrap_or(TEMPERATE), w.air_at(p))
    });
    let mid = points[0];
    Around {
        climate: Climate {
            celsius: blend(points.map(|p| p.1.celsius)),
            humidity: blend(points.map(|p| p.1.humidity)),
            seasons: blend(points.map(|p| p.1.seasons)),
            ..mid.1
        },
        wind: blend(points.map(|p| p.2.wind)),
        fog: blend(points.map(|p| p.2.fog)),
        snow: blend(points.map(|p| p.2.snow)),
        flags: mid.2.flags,
        canopy: blend(points.map(|p| canopy(p.0))),
    }
}

/// How closed biome `id`'s canopy is (`vitals::air::CANOPY`), 0 for open country.
pub fn canopy(id: u32) -> f32 {
    use std::sync::OnceLock;
    static IDS: OnceLock<Vec<(u32, f32)>> = OnceLock::new();
    IDS.get_or_init(|| {
        vitals::air::CANOPY
            .iter()
            .map(|&(name, s)| (Biome::generated(name.to_string()).id(), s))
            .collect()
    })
    .iter()
    .find(|c| c.0 == id)
    .map_or(0.0, |c| c.1)
}

/// How much of the ground `vitals::air::FOREST_EDGE` blocks round `eye` is under leaves or a roof
/// at that height (sky light under 15), 0 open to 1: deep in a forest all of it, at its edge half.
pub fn cover(w: &World, eye: [i32; 3]) -> f32 {
    let ring = vitals::air::ring(eye[0], eye[2], vitals::air::FOREST_EDGE);
    ring[1..]
        .iter()
        .filter(|&&(x, z)| w.sky_light([x, eye[1], z]) < 15)
        .count() as f32
        / 8.0
}

/// What the ground offers a plant at `pos` (`ground_at`): the day's mean air, how wet the soil
/// is, and the season (in a dimension with a calendar).
#[derive(Clone, Copy, Debug, Default)]
pub struct Ground {
    pub mean_air: f32,
    /// 0 dry to 1 soaked: the biome's last days of rain (`vitals::plants::soil_water`), or soaked
    /// by water within `WATER_REACH` blocks.
    pub soil: f32,
    pub season: Option<usize>,
}

/// Blocks from water within which the ground is wet whatever the weather (as farmland's).
pub const WATER_REACH: i32 = 4;

/// The ground at `pos` for a plant (`notes/ECOLOGY.md`, part 3): its biome's climate and season,
/// the height, and the rain of the last days there.
pub fn ground_at(w: &World, pos: [i32; 3]) -> Ground {
    use vitals::{calendar, plants};
    let biome = w.biome(pos);
    let c = CLIMATE.on_biome(biome).unwrap_or(TEMPERATE);
    let time = w.day_time();
    let seasons = SEASONS.on_dimension(w.dimension()).unwrap_or(0) > 0;
    let date = calendar::date(time);
    let height = (pos[1] - SEA).max(0) as f32 * ALTITUDE;
    let mean = if seasons { plants::mean_air(c.celsius, c.seasons, date.day as f32 + date.fraction) } else { c.celsius };
    let by_water = (-WATER_REACH..=WATER_REACH).any(|dx| {
        (-WATER_REACH..=WATER_REACH).any(|dz| {
            (-1..=0).any(|dy| {
                block_name(w.block([pos[0] + dx, pos[1] + dy, pos[2] + dz]))
                    .split('[')
                    .next()
                    == Some("base:water")
            })
        })
    });
    let soil = if by_water {
        1.0
    } else {
        plants::soil_water(c.rain, c.celsius, c.seasons, seasons, rain_key(w, biome), time / calendar::DAY_TICKS)
    };
    Ground { mean_air: mean - 0.0065 * height, soil, season: seasons.then(|| date.season.index() as usize) }
}

/// A biome's own rain in this world (`vitals::climate::spell`): the weather and the ground read
/// the same schedule.
pub fn rain_key(w: &World, biome: u32) -> u64 {
    w.seed() ^ (biome as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

/// How much a block heats the air near it (1 lava's). Looked up by name once per block state and
/// kept in a table (`HEATS`): the air's scan (7×7×7 a player a second) and the animals' search for
/// a fire (33×7×33) ask it for every block they pass.
pub fn heat_of(state: u32) -> f32 {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU8, Ordering};
    // Hundredths of heat + 1 per state; 0 not looked up yet.
    static HEATS: OnceLock<Box<[AtomicU8]>> = OnceLock::new();
    let table = HEATS.get_or_init(|| (0..HEAT_STATES).map(|_| AtomicU8::new(0)).collect());
    let Some(slot) = table.get(state as usize) else { return heat_by_name(state) };
    match slot.load(Ordering::Relaxed) {
        0 => {
            let h = heat_by_name(state);
            slot.store((h * 100.0).round().clamp(0.0, 254.0) as u8 + 1, Ordering::Relaxed);
            h
        }
        v => (v - 1) as f32 / 100.0,
    }
}

/// Block states `heat_of` keeps in its table (a byte each); any past it are looked up by name.
const HEAT_STATES: usize = 1 << 20;

fn heat_by_name(state: u32) -> f32 {
    // The state's name without its properties (`base:lava[level=0]`).
    match block_name(state).split('[').next().unwrap_or("") {
        "base:lava" | "base:campfire" => vitals::temperature::LAVA_HEAT,
        "base:furnace" if state_prop(state, "lit") == Some("true") => vitals::temperature::FURNACE_HEAT,
        "base:torch" | "base:wall_torch" => vitals::temperature::TORCH_HEAT,
        _ => 0.0,
    }
}

impl Vitals {
    /// A new player: breath full, stamina full (the first tick brings it down to what the stomach
    /// allows), the body at 37 °C, a light meal in the stomach (`vitals::stomach::arrival`).
    pub fn new(air: f32) -> Vitals {
        let mut v = Vitals { core: vitals::temperature::CORE, air, stamina: NEW_STAMINA, ..Vitals::default() };
        v.set_stomach(&vitals::stomach::arrival());
        v
    }

    pub fn stomach(&self) -> vitals::stomach::Stomach {
        let mut s = vitals::stomach::Stomach::default();
        for (i, p) in s.portions.iter_mut().enumerate() {
            *p = vitals::stomach::Portion {
                food: self.foods[i],
                units: self.units[i],
                quality: self.quality[i],
                warmth: self.warmth[i],
                warm_secs: self.warm_secs[i],
            };
        }
        s
    }

    pub fn set_stomach(&mut self, s: &vitals::stomach::Stomach) {
        for (i, p) in s.portions.iter().enumerate() {
            (self.foods[i], self.units[i], self.quality[i], self.warmth[i], self.warm_secs[i]) =
                (p.food, p.units, p.quality, p.warmth, p.warm_secs);
        }
    }

    /// The stats' values from the points placed.
    pub fn stats(&self) -> [f32; vitals::stats::STATS] {
        vitals::stats::player(&self.points.map(u32::from))
    }

    /// The stomach's size (the food stat).
    pub fn capacity(&self) -> f32 {
        self.stats()[vitals::stats::Stat::Food.index()]
    }

    pub fn has(&self, e: vitals::effects::Effect) -> bool {
        self.effects & (1 << e.index()) != 0
    }

    pub fn effects(&self) -> impl Iterator<Item = vitals::effects::Effect> + '_ {
        vitals::effects::ALL.into_iter().filter(|&e| self.has(e))
    }
}

/// Value chains exposed by the base game.
pub mod chains;

/// HUD elements exposed by the base game.
pub mod hud;

// `Food`, `FOOD`, `FUEL` and `FLAMMABLE` are in `components`; what a food gives comes from the
// `vitals` food table by the item's name (`eat` below), `Food`'s hunger and saturation are the
// base game's old numbers, kept for other mods.
// SYNC: `components::FoodItem::food` still eats with upstream's hunger `components::eat`; point it
// at ours (`eat` here) or keep our food on `.on_use(base_api::eat)`.

value! {
    /// Protection while worn: about 4% less damage per point, less against big hits unless
    /// `toughness` holds them back, the way modern's armor works.
    pub struct Armor { pub points: u32, pub toughness: f32 }
}
value! {
    /// What a portion of a food holds, in grams (its size in units of 10 g), from the vitals'
    /// food table: shown in its tooltip.
    pub struct Nutrition { pub units: u16, pub protein: u16, pub energy: u16, pub fat: u16, pub greens: u16, pub water: u16 }
}
pub static NUTRITION: Component<Nutrition> = Component::named("base:nutrition");
/// A food stack's day of making + 1 (0: not stamped yet), the second it came hot off the fire
/// (gone once it cools, `vitals::food::HOT_FOR`), and how fresh it is now (`freshness`): stamped
/// in a player's inventory as it ages, so stacks of a day stack and the tooltip can say it.
pub static MADE: Component<u32> = Component::named("base:made");
pub static HOT: Component<u32> = Component::named("base:hot");
pub static FRESHNESS: Component<u8> = Component::named("base:freshness");
/// A stew's ingredients (`vitals::meals::pack`), and what it holds, shown in its tooltip; both on
/// the stack, as a stew is what went into it.
pub static MEAL: Component<u64> = Component::named("base:meal");
pub static MEAL_NUTRITION: Component<Nutrition> = Component::named("base:meal_nutrition");
/// The empty bowl a stew is eaten from.
pub static BOWL: Item = Item::named("base:bowl");

value! {
    /// A piece of clothing's warmth (`vitals::clothing`), for its tooltip: degrees of cold and heat
    /// insulation, the share kept wet and the share of the wind stopped, in percent.
    pub struct Warmth { pub cold: u32, pub heat: i32, pub wet: u32, pub wind: u32 }
}
pub static WARMTH: Component<Warmth> = Component::named("base:warmth");
pub mod freshness {
    pub const FRESH: u8 = 0;
    pub const STALE: u8 = 1;
    pub const SPOILED: u8 = 2;
    pub const HOT: u8 = 3;
}

/// Seconds a food stack has existed (as far as its stamps say), now at day time `now` (ticks),
/// and whether it's still hot.
pub fn food_age(stack: &ItemStack, now: u64) -> (f32, bool) {
    let secs = (now / 20) as u32;
    if let Some(hot) = HOT.on_stack_own(stack) {
        let age = secs.saturating_sub(hot) as f32;
        return (age, age < vitals::food::HOT_FOR);
    }
    let made = MADE.on_stack_own(stack).unwrap_or(0);
    if made == 0 {
        return (0.0, false);
    }
    let days = (now / vitals::calendar::DAY_TICKS).saturating_sub(made as u64 - 1);
    (days as f32 * vitals::DAY_SECS, false)
}

/// Eating for `.on_use`: the food goes into the stomach (`vitals::food::FOODS` knows it by the
/// item's name), as good as it is fresh (stale half, spoiled a quarter), hot food warming; raw,
/// stale or spoiled food may give food poisoning (fortitude resists it). One item less. Refused
/// when there's no room for it, or the item isn't a food the vitals know. With `.use_ticks(32)`
/// it takes as long as the base game's food.
pub fn eat(u: &mut ItemUse) -> bool {
    let stack = u.stack();
    let Some(food) = vitals::food::find(weft_sdk::item_name(stack.item)) else { return false };
    let w = u.world();
    let f = &vitals::food::FOODS[food];
    let (age, hot) = food_age(&stack, w.day_time());
    let fresh = vitals::food::freshness(f, age, difficulty().keep);
    let roll = w.random_below(1000) as f32 / 1000.0;
    let Some(v) = w.component_mut(u.entity(), &VITALS) else { return false };
    let mut s = v.stomach();
    if s.eat(food, fresh.quality(), if hot { 0.0 } else { vitals::food::HOT_FOR }, v.capacity())
        .is_err()
    {
        return false;
    }
    v.set_stomach(&s);
    let fortitude = v.stats()[vitals::stats::Stat::Fortitude.index()];
    let resist = vitals::stats::fortitude_resists(fortitude);
    if f.poison >= 1.0 {
        // A poisonous thing (a spider's eye): poisoned, not sick.
        v.poison = v.poison.max(vitals::food::POISON_SECS * (1.0 - resist));
    } else if roll < vitals::food::poisoning_chance(f, fresh) * (1.0 - resist) {
        v.sick = v.sick.max(vitals::food::SICK_SECS);
    }
    u.set_stack(consume_one(stack));
    true
}

/// Eating a stew for `.on_use`: its ingredients go into the stomach together (`vitals::meals`), as
/// good as the stew is fresh (it keeps as its shortest-keeping ingredient), the cooking's bonus on
/// top; hot, it warms. The bowl is left. Refused when it doesn't all fit.
pub fn eat_meal(u: &mut ItemUse) -> bool {
    let stack = u.stack();
    let foods = vitals::meals::ingredients(MEAL.on_stack_own(&stack).unwrap_or(0));
    if foods.is_empty() {
        return false;
    }
    let w = u.world();
    let (age, hot) = food_age(&stack, w.day_time());
    let fresh = vitals::food::freshness_of(vitals::meals::keeps(&foods), age, difficulty().keep);
    let roll = w.random_below(1000) as f32 / 1000.0;
    let Some(v) = w.component_mut(u.entity(), &VITALS) else { return false };
    let mut s = v.stomach();
    if s.eat_meal(&foods, fresh.quality(), if hot { 0.0 } else { vitals::food::HOT_FOR }, v.capacity())
        .is_err()
    {
        return false;
    }
    v.set_stomach(&s);
    let resist = vitals::stats::fortitude_resists(v.stats()[vitals::stats::Stat::Fortitude.index()]);
    if roll < vitals::meals::poisoning_chance(&foods, fresh) * (1.0 - resist) {
        v.sick = v.sick.max(vitals::food::SICK_SECS);
    }
    u.set_stack(weft_sdk::consume_one(stack));
    true
}

/// Poisons `player` for `secs` (venom, a poisonous thing eaten), less with fortitude; nothing in
/// creative. Poison takes health a while (`vitals::effects::Effect::Poisoned`).
pub fn poison(w: &World, player: u64, secs: f32) {
    with_body(w, player, |v, resist| v.poison = v.poison.max(secs * (1.0 - resist)));
}

/// `player` bleeds for `secs` (a deep wound); a bandage stops it.
pub fn bleed(w: &World, player: u64, secs: f32) {
    with_body(w, player, |v, _| v.bleed = v.bleed.max(secs));
}

/// `amount` more torpor in `player` (a dart, a blow), less with fortitude: past their torpor stat
/// they're knocked out until it falls under half.
pub fn sedate(w: &World, player: u64, amount: f32) {
    with_body(w, player, |v, resist| v.torpor += amount * (1.0 - resist));
}

fn with_body(w: &World, player: u64, f: impl FnOnce(&mut Vitals, f32)) {
    if w.outside_survival(player) {
        return;
    }
    let Some(v) = w.component_mut(player, &VITALS) else { return };
    let resist = vitals::stats::fortitude_resists(v.stats()[vitals::stats::Stat::Fortitude.index()]);
    f(v, resist);
}

/// Server to client: a hint from any mod (`hint`), as its lang key and number arguments, tab
/// separated.
pub static HINT_TEXT: weft_sdk::Channel<String> = weft_sdk::Channel::named("base:hint_text");

/// Shows `player` a hint: the line under the crosshair the base game's hints use (`hints.rs`), a
/// few seconds, one at a time, off when the player's `base:hints` setting is. `key` is a lang key
/// of any mod (`hint.animals.tamed`), `args` fill its `{0}`, `{1}`... The caller decides how often
/// (the base game's own show once per player).
pub fn hint(w: &World, player: u64, key: &str, args: &[i64]) {
    let mut text = key.to_string();
    for a in args {
        text.push('\t');
        text.push_str(&a.to_string());
    }
    w.send(player, &HINT_TEXT, &text);
}

/// Ley sight (`notes/PROGRESSION.md`, "Round 6"): how clearly a player sees the ley lines without
/// a dowsing rod, 0 (not at all) to 1, on the player and sent to its owner. Base registers and
/// attaches it and draws the lines while it is over 0 (as faint as the rod's at 1, out to
/// `vitals::ley::SHOWN_SIGHT`); the magic path's perk (the perk web) sets it. The network itself
/// is `vitals::ley`: `ley::strength(w.seed(), x, z)` is what magic, mana and machines draw on.
pub static LEY_SIGHT: Component<f32> = Component::named("base:ley_sight");

/// Whose tame a creature is, as its watchers' clients may know: the owner's player entity while
/// that player is in the world, 0 otherwise. Base registers it (sent to every watcher) and its map
/// marks the local player's tames in their own colour; a mod with tames attaches it to its kinds
/// and keeps it (animals: `animal::mark_owner`).
pub static TAME_OF: Component<u64> = Component::named("base:tame_of");

/// Things a player has done that other mods know of and the base game's advancements read
/// (`advancements.rs`, the "Surviving" group): a bit each (`milestones`), kept through death.
pub static MILESTONES: Component<u32> = Component::named("base:milestones");
pub mod milestones {
    /// Tamed an animal (the animals mod).
    pub const TAME: u32 = 1;
    /// Saw a blood moon through to its end without dying (base's sky events).
    pub const RED_NIGHT: u32 = 2;
    /// Out under the open sky on a meteor shower's peak night, and on an aurora storm's.
    pub const WISH_UPON: u32 = 4;
    pub const SKY_ON_FIRE: u32 = 8;
}

/// Marks a milestone (`milestones::TAME`...) for `player`: the advancement for it comes within a
/// second.
pub fn milestone(w: &World, player: u64, m: u32) {
    if let Some(done) = w.component_mut(player, &MILESTONES) {
        *done |= m;
    }
}

value! {
    /// Where a player's tombstone was put when they last died (`set` 0: none yet), for the hint
    /// after respawning.
    pub struct Grave { pub x: i32, pub y: i32, pub z: i32, pub set: u32 }
}
pub static GRAVE: Component<Grave> = Component::named("base:grave");

/// Gives `player` XP for what they did (`vitals::levels::xp`): other mods call it for theirs
/// (taming, raising young, hatching eggs). Rested raises it; nothing in creative.
pub fn give_xp(w: &World, player: u64, amount: u64) {
    if amount == 0 || w.outside_survival(player) {
        return;
    }
    let Some(v) = w.component_mut(player, &VITALS) else { return };
    let rate = vitals::effects::combine(v.effects()).xp;
    v.xp += (amount as f32 * rate).round() as u64;
}

/// Whether there's room in `player`'s stomach for all of a stew.
pub fn has_room_for_meal(w: &World, player: u64, stack: &ItemStack) -> bool {
    let foods = vitals::meals::ingredients(MEAL.on_stack_own(stack).unwrap_or(0));
    w.component(player, &VITALS)
        .is_some_and(|v| v.stomach().room(v.capacity()) + 1e-4 >= vitals::meals::units(&foods))
}

/// Whether there's room in `player`'s stomach for a portion of that item.
pub fn has_room(w: &World, player: u64, item: u32) -> bool {
    let Some(food) = vitals::food::find(weft_sdk::item_name(item)) else { return false };
    w.component(player, &VITALS)
        .is_some_and(|v| v.stomach().room(v.capacity()) + 1e-4 >= vitals::food::FOODS[food].units())
}
// SYNC: upstream made armor attributes (`attributes::ARMOR`, `ARMOR_TOUGHNESS`, given worn by
// `Modifier::add(..).worn()`); ours is still this stack component, as a piece built from parts
// (`parts.rs`) carries its own points on its stack. Move ours onto the attributes once a stack
// can carry its own modifiers.
pub static ARMOR: Component<Armor> = Component::named("armor");
/// A weapon's or tool's item level (`vitals::curve`: tier × 10 + depth, its working material's):
/// on today's tools as their item's, on a tool built from parts as its stack's. Against a big
/// creature it counts at most its size's cap over the creature (`vitals::combat::Band::cap`).
pub static ITEM_LEVEL: Component<f32> = Component::named("base:item_level");
/// A saddle's fitting (`vitals::gear::SADDLE_FITTINGS`' index; 0 or none: plain leather). Base's
/// assembly row fits one (`parts.rs`); the animals mod reads it when it saddles a tame. Items
/// tagged `saddle` take it.
pub static SADDLE_FIT: Component<u32> = Component::named("base:saddle_fit");
/// The tag every saddle item carries, for the assembly row.
pub const SADDLE_TAG: &str = "saddle";

/// A saddle stack of `item` fitted with `fit` (`vitals::gear::SADDLE_FITTINGS`' index): its fitting
/// and its item level on it. Fitting in base's assembly row and unsaddling a tame both make it.
pub fn saddle_stack(item: u32, fit: u32) -> ItemStack {
    ItemStack::new(item, 1)
        .with(&SADDLE_FIT, fit)
        .with(&ITEM_LEVEL, vitals::gear::saddle_level(fit))
}

/// A saddle stack's fitting (0: plain).
pub fn saddle_fit(stack: &ItemStack) -> u32 {
    SADDLE_FIT.on_stack(stack).unwrap_or(0)
}

/// An arrow's head: its place in `vitals::gear::ARROWHEADS` plus one (none or 0: a plain
/// flint-tipped arrow). Base's assembly row makes them (`parts::fletched`); a bow reads the arrow's
/// `ITEM_LEVEL`.
pub static ARROW_HEAD: Component<u32> = Component::named("base:arrow_head");

/// `count` arrows headed with `vitals::gear::ARROWHEADS[head]`: the head and its item level on
/// them (base's `parts` adds the look).
pub fn arrow_stack(head: u32, count: u32) -> ItemStack {
    ItemStack::new(items::ARROW.id(), count)
        .with(&ARROW_HEAD, head + 1)
        .with(&ITEM_LEVEL, vitals::gear::arrow_level(head))
}

/// Clothes' slots, head to feet, worn under armor (`vitals::clothing`); armor's are in `slots`.
pub static HAT: EquipmentSlot = EquipmentSlot::named("base:hat");
pub static COAT: EquipmentSlot = EquipmentSlot::named("base:coat");
pub static TROUSERS: EquipmentSlot = EquipmentSlot::named("base:trousers");
pub static SHOES: EquipmentSlot = EquipmentSlot::named("base:shoes");
pub const CLOTHES: [&EquipmentSlot; 4] = [&HAT, &COAT, &TROUSERS, &SHOES];
/// A cloak, over clothes and armor (`vitals::clothing::CLOAKS`).
pub static CLOAK: EquipmentSlot = EquipmentSlot::named("base:cloak");

/// The Minecraft forest test world (`land::MC_MIX`): the overworld's land on the same seed under
/// Minecraft's temperate forest mix, for like-for-like comparisons with the Minecraft shader shots.
/// Reached with `/dim base:mc_forest`; no normal world goes there.
pub static MC_FOREST: Dimension = Dimension::named("base:mc_forest");

/// Dimensions registered by the base game.
pub mod dimensions;

/// Vanilla's entity kinds.
pub mod entities;

/// What is in the sky of the game's dimensions. Any of them can be changed or taken out by a mod:
/// `r.edit(OVERWORLD.sky(&MOON)).size(40.0)`, `r.remove(OVERWORLD.sky(&STARS))`.
pub mod sky_objects;

/// Vanilla's blocks.
pub mod blocks;

/// Vanilla's menus.
pub mod menus;

/// Vanilla's items.
pub mod items;

/// Vanilla's equipment slots.
pub mod slots;

/// Vanilla's recipe types.
pub mod recipes;

/// Vanilla's creative tabs.
pub mod tabs;

#[cfg(test)]
mod tests {
    use super::*;

    /// A new player starts with full stamina (Tester, 2026-11-03: it started at 0 and took 14 s to
    /// fill), however many points are placed in it.
    #[test]
    fn a_new_player_starts_with_full_stamina() {
        let most = vitals::stats::player(&[u32::from(u16::MAX); vitals::stats::STATS])[vitals::stats::Stat::Stamina.index()];
        assert!(Vitals::new(10.0).stamina >= most, "a new player's stamina {} under a max of {most}", Vitals::new(10.0).stamina);
    }

    /// A new player arrives with its light meal in it.
    #[test]
    fn a_new_player_arrives_fed() {
        assert_eq!(Vitals::new(10.0).stomach(), vitals::stomach::arrival());
        assert!(vitals::stomach::arrival().used() > 0.0);
    }
}
