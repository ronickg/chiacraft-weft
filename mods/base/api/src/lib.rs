//! What other mods use of the base game, as types: its events with their values, components,
//! value chains, recipe kinds, equipment slots, creative tabs and dimensions. A mod adds
//! `base_api` to its `Cargo.toml` and gets these checked when it compiles, without depending on
//! the game's code; the engine still compares the value layouts when the mods load.
//!
//! Names here are full (`base:..`): the statics compile into whichever mod uses them.

use weft_sdk::*;

value! {
    /// Fired when a furnace finishes one item.
    pub struct Smelted { pub furnace: u64, pub input: u32, pub output: ItemStack }
}
pub static SMELTED: Event<Smelted> = Event::named("base:smelted");

value! {
    /// A furnace's state, synced to whoever has it open.
    pub struct Smelting { pub burn: u32, pub burn_max: u32, pub cook: u32, pub cook_max: u32, pub cooking_item: u32 }
}
pub static SMELTING_STATE: Component<Smelting> = Component::named("base:smelting");

/// Portions a stomach holds (`vitals::stomach::PORTIONS`).
pub const PORTIONS: usize = vitals::stomach::PORTIONS;
/// The engine counts health as the base game's 20 points, which keeps every creature's damage
/// in balance; the vitals count 100 (`notes/SURVIVAL.md`). A health point is this many.
pub const HEALTH_SCALE: f32 = 5.0;

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
    Climate { celsius: c.celsius, humidity: c.humidity, seasons: c.seasons, rain: c.rain, wind: c.wind, fog: c.fog, fog_kind: c.fog_kind.index() }
}

/// A biome without one: a mild, temperate place, raining a quarter of the days, a light breeze
/// and a few foggy mornings.
pub const TEMPERATE: Climate = Climate { celsius: 12.0, humidity: 0.5, seasons: 10.0, rain: [0.25; 4], wind: [4.0; 4], fog: [8.0; 4], fog_kind: 0 };

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
    /// Degrees from the nearest fire (lava, a lit furnace, torches).
    pub fire: f32,
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
    let climate = CLIMATE.on_biome(w.biome(feet)).unwrap_or(TEMPERATE);
    let sky = w.sky_light(eye);
    let underground = sky == 0 && feet[1] < SEA - 6;
    let mut fire = 0.0f32;
    for dx in -reach..=reach {
        for dy in -reach..=reach {
            for dz in -reach..=reach {
                let state = w.get([feet[0] + dx, feet[1] + dy, feet[2] + dz]);
                each(state);
                let h = heat_of(state);
                if h > 0.0 {
                    let d = ((dx * dx + dy * dy + dz * dz) as f32).sqrt();
                    fire = fire.max(temperature::fire_heat(d) * h);
                }
            }
        }
    }
    let hour = ((w.day_time() % 24000) as f32 / 1000.0 + 6.0) % 24.0;
    // Today, in this dimension's calendar (none: always a spring or autumn day).
    let kind = SEASONS.on_dimension(w.dimension()).unwrap_or(0);
    let date = calendar::date(w.day_time());
    let year = if kind > 0 { date.from_midwinter() } else { 0.25 };
    let weather = w.air_at(eye);
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
        wind: weather.wind,
    };
    if place.raining && temperature::air(&place) < 1.0 {
        (place.raining, place.snowing) = (false, true);
    }
    Air {
        celsius: temperature::air(&place),
        fire,
        sky,
        underground,
        open_sky: place.open_sky,
        raining: place.raining,
        snowing: place.snowing,
        wind: weather.wind,
        fog: weather.fog,
        snow: weather.snow,
        day: if kind > 0 { date.day as u16 + 1 } else { 0 },
        calendar: kind,
    }
}

/// How far an animal sees at `pos` against clear air, 1 to `vitals::air::FOG_SIGHT`: fog, a
/// blizzard or a dust storm (`World::air_at`) close it in.
pub fn sight(w: &World, pos: [f64; 3]) -> f32 {
    vitals::air::sight(w.air_at(pos.map(|v| v.floor() as i32)).fog)
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
        (-WATER_REACH..=WATER_REACH).any(|dz| (-1..=0).any(|dy| block_name(w.get([pos[0] + dx, pos[1] + dy, pos[2] + dz])).split('[').next() == Some("base:water")))
    });
    let soil = if by_water { 1.0 } else { plants::soil_water(c.rain, c.celsius, c.seasons, seasons, rain_key(w, biome), time / calendar::DAY_TICKS) };
    Ground { mean_air: mean - 0.0065 * height, soil, season: seasons.then(|| date.season.index() as usize) }
}

/// A biome's own rain in this world (`vitals::climate::spell`): the weather and the ground read
/// the same schedule.
pub fn rain_key(w: &World, biome: u32) -> u64 {
    w.seed() ^ (biome as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

/// How much a block heats the air near it (1 lava's).
pub fn heat_of(state: u32) -> f32 {
    // The state's name without its properties (`base:lava[level=0]`).
    match block_name(state).split('[').next().unwrap_or("") {
        "base:lava" => 1.0,
        "base:furnace" if state_prop(state, "lit") == Some("true") => 0.7,
        "base:torch" | "base:wall_torch" => 0.25,
        _ => 0.0,
    }
}

impl Vitals {
    /// A new player: breath full, the body at 37 °C, an empty stomach.
    pub fn new(air: f32) -> Vitals {
        Vitals { core: vitals::temperature::CORE, air, ..Vitals::default() }
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

value! {
    /// A player's advancements, a bit each in the order of the base game's list.
    pub struct Advancements { pub done: u64 }
}
pub static ADVANCEMENTS: Component<Advancements> = Component::named("base:advancements");

value! {
    /// Where a player respawns: the foot of the bed they last used and its dimension, when `set`
    /// is 1. Kept through death.
    pub struct BedSpawn { pub bed: [i32; 3], pub dim: u32, pub set: u32 }
}
pub static BED_SPAWN: Component<BedSpawn> = Component::named("base:bed_spawn");

value! {
    /// Marks an item as food (what it gives comes from the `vitals` food table, by the item's
    /// name); `hunger` and `saturation` are the base game's old numbers, kept for other mods.
    pub struct Food { pub hunger: u32, pub saturation: f32 }
}
value! {
    /// Protection while worn: about 4% less damage per point, less against big hits unless
    /// `toughness` holds them back, the way modern's armor works.
    pub struct Armor { pub points: u32, pub toughness: f32 }
}

// The common vocabulary the base game reads; any mod may set these on its own content.
pub static FOOD: Component<Food> = Component::named("food");

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
    if let Some(hot) = HOT.stack_only(stack) {
        let age = secs.saturating_sub(hot) as f32;
        return (age, age < vitals::food::HOT_FOR);
    }
    let made = MADE.stack_only(stack).unwrap_or(0);
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
    let Some(v) = w.component_mut(u.player(), &VITALS) else { return false };
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
    u.set_stack(stack.shrink(1));
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
    if w.creative(player) {
        return;
    }
    let Some(v) = w.component_mut(player, &VITALS) else { return };
    let resist = vitals::stats::fortitude_resists(v.stats()[vitals::stats::Stat::Fortitude.index()]);
    f(v, resist);
}

/// Gives `player` XP for what they did (`vitals::levels::xp`): other mods call it for theirs
/// (taming, raising young, hatching eggs). Rested raises it; nothing in creative.
pub fn give_xp(w: &World, player: u64, amount: u64) {
    if amount == 0 || w.creative(player) {
        return;
    }
    let Some(v) = w.component_mut(player, &VITALS) else { return };
    let rate = vitals::effects::combine(v.effects()).xp;
    v.xp += (amount as f32 * rate).round() as u64;
}

/// Whether there's room in `player`'s stomach for a portion of that item.
pub fn has_room(w: &World, player: u64, item: u32) -> bool {
    let Some(food) = vitals::food::find(weft_sdk::item_name(item)) else { return false };
    w.component(player, &VITALS)
        .is_some_and(|v| v.stomach().room(v.capacity()) + 1e-4 >= vitals::food::FOODS[food].units())
}
/// Burn time in ticks.
pub static FUEL: Component<u32> = Component::named("fuel");
pub static ARMOR: Component<Armor> = Component::named("armor");
/// How readily fire takes it.
pub static FLAMMABLE: Component<u32> = Component::named("flammable");

/// A furnace's cooking time: base is the recipe's time, `entity` the furnace, `item` the input.
pub static SMELT_TIME: Chain = Chain::named("base:smelt_time");

pub static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");
pub static SMELTING: RecipeKind = RecipeKind::named("base:smelting");

/// The player's equipment slots, head to feet.
pub static HEAD: EquipmentSlot = EquipmentSlot::named("base:head");
pub static CHEST: EquipmentSlot = EquipmentSlot::named("base:chest");
pub static LEGS: EquipmentSlot = EquipmentSlot::named("base:legs");
pub static FEET: EquipmentSlot = EquipmentSlot::named("base:feet");

pub static OVERWORLD: Dimension = Dimension::named("base:overworld");
pub static FLATLANDS: Dimension = Dimension::named("base:flatlands");

/// What is in the sky of the game's dimensions. Any of them can be changed or taken out by a mod:
/// `r.sky_body(&OVERWORLD, &MOON).size(40.0)`, `r.remove_sky(&OVERWORLD, &STARS)`.
pub static SUN: SkyObject = SkyObject::named("base:sun");
pub static MOON: SkyObject = SkyObject::named("base:moon");
pub static STARS: SkyObject = SkyObject::named("base:stars");
pub static CLOUDS: SkyObject = SkyObject::named("base:clouds");

/// Vanilla's creative tabs.
pub mod tabs {
    use weft_sdk::CreativeTab;
    pub static BUILDING: CreativeTab = CreativeTab::named("base:building");
    pub static COLORED: CreativeTab = CreativeTab::named("base:colored");
    pub static NATURAL: CreativeTab = CreativeTab::named("base:natural");
    pub static FUNCTIONAL: CreativeTab = CreativeTab::named("base:functional");
    pub static TOOLS: CreativeTab = CreativeTab::named("base:tools");
    pub static COMBAT: CreativeTab = CreativeTab::named("base:combat");
    pub static FOOD: CreativeTab = CreativeTab::named("base:food");
    pub static INGREDIENTS: CreativeTab = CreativeTab::named("base:ingredients");
    pub static REDSTONE: CreativeTab = CreativeTab::named("base:redstone");
}
