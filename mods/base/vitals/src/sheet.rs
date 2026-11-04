//! The survival numbers as one page (`mods/base/vitals/SHEET.md`), written from this crate so it
//! can't drift: `cargo run -p vitals --bin sheet > mods/base/vitals/SHEET.md`. A test fails while
//! the page is out of date (`tests/sheet.rs`).

use std::fmt::Write;

use crate::effects;
use crate::food::{self, NUTRIENT_NAMES};
use crate::stats::{self, Growth};
use crate::{
    air, calendar, climate, clothing, crafts, difficulty, forage, levels, meals, plants, preserve, shelter, soil, temperature, wild,
};

pub fn markdown() -> String {
    let mut w = String::new();
    let _ = writeln!(w, "# Survival numbers\n");
    let _ = writeln!(
        w,
        "Written from `mods/base/vitals` (`cargo run -p vitals --bin sheet > mods/base/vitals/SHEET.md`); do not edit by hand. How the systems connect: `notes/SYSTEMS.md`; why: `notes/SURVIVAL.md`.\n"
    );
    stats_table(&mut w);
    foods(&mut w);
    cooking(&mut w);
    clothes(&mut w);
    rooms(&mut w);
    waiting(&mut w);
    felling_table(&mut w);
    effects_table(&mut w);
    climates(&mut w);
    plant_table(&mut w);
    crop_table(&mut w);
    wild_table(&mut w);
    forage_table(&mut w);
    soil_table(&mut w);
    difficulties(&mut w);
    level_table(&mut w);
    w
}

fn stats_table(w: &mut String) {
    let _ = writeln!(w, "## A player's stats\n\n| stat | level 1 | a point |\n|---|---|---|");
    for (i, name) in stats::NAMES.iter().enumerate() {
        let point = match stats::PLAYER_GROWTH[i] {
            Growth::Share(s) => format!("+{:.0}%", s * 100.0),
            Growth::Add(a) => format!("+{a}"),
        };
        let _ = writeln!(w, "| {name} | {} | {point} |", stats::PLAYER_BASE[i]);
    }
    let _ = writeln!(w, "\nComfortable air for a person, unclothed: {:?} °C.\n", temperature::PERSON);
    let arrival = crate::stomach::arrival();
    let cap = stats::PLAYER_BASE[stats::Stat::Food.index()];
    let (health, stamina) = crate::stomach::max_health_stamina(100.0, 100.0, &arrival, cap);
    let _ = writeln!(
        w,
        "A new player arrives with {} in the stomach ({:.0} units): max health {health:.0} and stamina {stamina:.0} of 100, wearing off as it digests (an empty stomach: {:.0}). A new world begins on spring's day {}, a quarter of the year from midwinter.\n",
        crate::stomach::ARRIVAL.join(" and "),
        arrival.used(),
        crate::stomach::FLOOR * 100.0,
        calendar::START_DAY + 1
    );
}

fn foods(w: &mut String) {
    let _ = writeln!(
        w,
        "## Foods\n\nA portion, its kind for the animals' diets, what it holds of each nutrient, how long it digests, and game days fresh then stale.\n\n| food | kind | g | units | {} | digests | keeps | |\n|---|---|---|---|---|---|---|---|---|---|---|---|",
        NUTRIENT_NAMES.join(" | ")
    );
    for f in food::FOODS {
        let p = f.portion();
        let notes = [
            (f.raw, "raw"),
            (f.poison >= 1.0, "poisonous"),
            (f.meal == food::Meal::Cooked, "hot when cooked"),
            (f.meal == food::Meal::Cooling, "cooling"),
        ]
        .iter()
        .filter(|(on, _)| *on)
        .map(|(_, n)| *n)
        .collect::<Vec<_>>()
        .join(", ");
        let _ = writeln!(
            w,
            "| {} | {} | {} | {:.0} | {} | {:.0} min | {} + {} | {notes} |",
            f.name,
            f.category.name(),
            f.grams,
            f.units(),
            p.iter()
                .map(|v| format!("{v:.0}"))
                .collect::<Vec<_>>()
                .join(" | "),
            f.digest_minutes(),
            f.keeps.0,
            f.keeps.1
        );
    }
    let _ = writeln!(w);
}

fn cooking(w: &mut String) {
    let _ = writeln!(
        w,
        "## Meals and preserving\n\nA stew (`vitals::meals`) cooks {}-{} foods in {} s in a cooking pot; raw meat in it is cooked. Each goes into the stomach as its own portion at x{} (so each counts toward well fed), the meal warms as one hot dish, and it keeps as long as its shortest-keeping ingredient. A campfire cooks what a furnace cooks of food, x{} the time, with no fuel. Nothing is made of spoiled food, and what is made of stale food starts as far through its own life as its stalest part.\n",
        meals::MIN,
        meals::MAX,
        meals::COOK_TICKS / crate::TICKS,
        meals::BONUS,
        meals::CAMPFIRE_PACE
    );
    let _ = writeln!(
        w,
        "Preserving (`vitals::preserve`): a drying rack dries in {:.0} min in the best air (25 °C, humidity 0.3, a breeze; nothing below 5 °C or above humidity 0.85, nor in the rain), and smokes in {:.0} min over a fire (lava, a lit furnace) within {} blocks below, in any weather; salting is crafted with salt, boiled out of a water bucket in a furnace in {} s.\n\n| from | dried | smoked | salted | keeps (days) |\n|---|---|---|---|---|",
        preserve::DRY_SECS / 60.0,
        preserve::SMOKE_SECS / 60.0,
        preserve::SMOKE_REACH,
        preserve::SALT_TICKS / crate::TICKS
    );
    for (i, f) in food::FOODS.iter().enumerate() {
        let made: Vec<String> = preserve::METHODS
            .iter()
            .map(|&m| preserve::product(m, i).unwrap_or("-").to_string())
            .collect();
        if made.iter().all(|m| m == "-") {
            continue;
        }
        let keeps: Vec<String> = made
            .iter()
            .filter_map(|m| food::find(m))
            .map(|p| format!("{:.0}", food::FOODS[p].keeps.0 + food::FOODS[p].keeps.1))
            .collect();
        let _ = writeln!(w, "| {} ({:.0}) | {} | {} |", f.name, f.keeps.0 + f.keeps.1, made.join(" | "), keeps.join(", "));
    }
    let _ = writeln!(w);
}

fn felling_table(w: &mut String) {
    use crate::felling::{self, LONGEST_SECS, PAST_TIER, SHEET_TREES, TIERS};
    let _ = write!(
        w,
        "## Felling\n\nAn axe at a tree's foot fells it in one chop that takes logs^{} times one log's (`vitals::felling`), with the axe's own speed on top. Each tier of axe takes trees up to a size for their wood; past it a tree still falls, {} times slower, and gives nothing. A tree that loses its foot, or a log of its trunk, by any means falls (none stands in the air); bare hands and other tools chop the foot as slowly as an axe past its tier, and as fruitlessly. Never more than {} min. Felled, a tree within its tier lies on the ground in pieces of {} logs, each cut as long as felling a tree of its logs takes; the axe wears by logs^{} for a felling or a piece. A tree falls as itself: a {} s creak, then {:.1} s for one 10 high ({}-{} s, by the square root of its height), its leaves breaking off past {}° until {:.0}% have, the rest bursting on the ground; on the way leaves and branches in its path break, and a trunk it strikes falls too if the falling tree has {}x its logs. Seconds for a typical axe of each tier (a log is hardness 2), * past its tier:\n\n| tier | harvest | takes up to | speed |",
        felling::GROWTH,
        PAST_TIER,
        LONGEST_SECS / 60.0,
        felling::PIECE,
        felling::GROWTH,
        felling::CREAK_TICKS as f32 / crate::TICKS as f32,
        felling::FALL_SECS.0,
        felling::FALL_SECS.1,
        felling::FALL_SECS.2,
        felling::SHED_FROM,
        felling::SHED_SHARE * 100.0,
        felling::KNOCK_FELLS
    );
    for (name, logs) in SHEET_TREES {
        let _ = write!(w, " {name} ({logs}) |");
    }
    let _ = write!(w, "\n|---|---|---|---|");
    for _ in SHEET_TREES {
        let _ = write!(w, "---|");
    }
    let _ = writeln!(w);
    // Bare hands and every tool but an axe: slow and fruitless at the foot (Planner, 11-05).
    let _ = write!(w, "| bare hands, other tools | - | none | 1 |");
    for &(_, logs) in SHEET_TREES {
        let secs = felling::seconds(felling::by_hand(logs), felling::log_secs(2.0, 1.0));
        let time = if secs < 120.0 { format!("{secs:.0} s") } else { format!("{:.0} min", secs / 60.0) };
        let _ = write!(w, " {time} * |");
    }
    let _ = writeln!(w);
    for t in TIERS {
        let _ = write!(w, "| {} | {} | {} logs | {} |", t.name, t.harvest, t.logs, t.speed);
        let one = felling::log_secs(2.0, t.speed);
        for &(_, logs) in SHEET_TREES {
            let f = felling::felling(logs, t.harvest);
            let secs = felling::seconds(f, one);
            let time = if secs < 120.0 { format!("{secs:.0} s") } else { format!("{:.0} min", secs / 60.0) };
            let _ = write!(w, " {time}{} |", if f.fruitful { "" } else { " *" });
        }
        let _ = writeln!(w);
    }
    let _ = writeln!(w);
}

fn waiting(w: &mut String) {
    let _ = writeln!(
        w,
        "## Crafts that wait\n\nWhat takes days in real life takes its time on the growing pace (`vitals::crafts`; game days = {} · real days^{}), at a station that keeps working unwatched.\n\n| station | in | with | out | real days | in the game | holds |\n|---|---|---|---|---|---|---|",
        crate::plants::GROWING.0,
        crate::plants::GROWING.1
    );
    for c in crafts::CRAFTS {
        let secs = c.secs();
        let time =
            if secs < crate::DAY_SECS { format!("{:.1} min", secs / 60.0) } else { format!("{:.1} game days", secs / crate::DAY_SECS) };
        let _ = writeln!(
            w,
            "| {} | {} | {} | {} | {} | {time} | {} |",
            c.station.name(),
            c.input,
            c.with.unwrap_or("-"),
            c.output,
            c.real_days,
            c.station.holds()
        );
    }
    let _ = writeln!(w);
}

fn rooms(w: &mut String) {
    let _ = writeln!(
        w,
        "## Rooms\n\nAn enclosed space (`vitals::shelter`; up to {} blocks of air, none open to the sky) shuts out the wind, lets in its walls' share of the day's swing, and keeps {} °C over the air outside empty, or a fire's gain by its walls with a fire in it (full up to {} blocks of air, at least {:.0}% in a bigger room; never less than the fire gives beside it).\n\n| wall | a fire in the room | the day's swing let in |\n|---|---|---|",
        shelter::ROOM_MAX,
        shelter::EMPTY_ROOM,
        shelter::ROOM_FULL,
        shelter::BIG_ROOM_SHARE * 100.0
    );
    for wall in shelter::WALLS {
        let _ = writeln!(w, "| {} | +{} °C | {:.0}% |", wall.name(), wall.fire_gain(), wall.swing() * 100.0);
    }
    let _ = writeln!(w);
}

fn clothes(w: &mut String) {
    let _ = writeln!(
        w,
        "## Clothing\n\nWorn in four slots of their own under armor ({}), no armor value; armor and clothes stack, and leather armor is hide. A full set's real insulation in clo at {} °C of comfort each; a piece gives its share ({:?}, head to feet). Wet, armor keeps {:.0}% of its warmth; being wet itself feels {} °C colder. Each piece is sewn with {:?} string, a string is {} plant fibre. A new player arrives in {}.\n\n| material | clo | cold °C | heat °C | kept wet | windproof | pieces to make | from |\n|---|---|---|---|---|---|---|---|",
        clothing::PIECES.join(", "),
        clothing::PER_CLO,
        temperature::PIECE_SHARE,
        temperature::ARMOR_WET_KEEPS * 100.0,
        temperature::WET_CHILL,
        clothing::STRING,
        clothing::FIBRE_PER_STRING,
        clothing::ARRIVAL
            .iter()
            .map(|(m, p)| format!("{m} {}", clothing::PIECES[*p]))
            .collect::<Vec<_>>()
            .join(", ")
    );
    for m in clothing::MATERIALS {
        let _ = writeln!(
            w,
            "| {} | {} | {:.0} | {} | {:.0}% | {:.0}% | {:?} | {} |",
            m.name,
            m.clo,
            m.cold(),
            m.heat,
            m.wet_keeps * 100.0,
            m.windproof * 100.0,
            m.makes,
            m.from
        );
    }
    let _ = writeln!(
        w,
        "\nCloaks, in a slot of their own over clothes and armor (covering {:.0}% of the body against the wind):\n\n| cloak | cold °C | heat °C | kept wet | windproof | keeps the rain off | from |\n|---|---|---|---|---|---|---|",
        clothing::CLOAK_COVERS * 100.0
    );
    for c in clothing::CLOAKS {
        let _ = writeln!(
            w,
            "| {} | {} | {} | {:.0}% | {:.0}% | {} | {} |",
            c.name,
            c.cold,
            c.heat,
            c.wet_keeps * 100.0,
            c.windproof * 100.0,
            if c.keeps_dry { "yes" } else { "no" },
            c.from
        );
    }
    let _ = writeln!(w);
}

fn effects_table(w: &mut String) {
    let _ = writeln!(w, "## Effects\n\n| effect | group | changes |\n|---|---|---|");
    for e in effects::ALL {
        let m = e.modifiers();
        let n = effects::NONE;
        let mut parts = Vec::new();
        let mul = |parts: &mut Vec<String>, name: &str, v: f32, base: f32| {
            if v != base {
                parts.push(format!("{name} x{v}"));
            }
        };
        mul(&mut parts, "health regen", m.health_regen, n.health_regen);
        mul(&mut parts, "stamina regen", m.stamina_regen, n.stamina_regen);
        mul(&mut parts, "speed", m.speed, n.speed);
        mul(&mut parts, "stamina costs", m.stamina_cost, n.stamina_cost);
        mul(&mut parts, "XP", m.xp, n.xp);
        mul(&mut parts, "water used", m.water_use, n.water_use);
        if m.health_loss > 0.0 {
            parts.push(format!("loses {}% of max health a second", m.health_loss * 100.0));
        }
        if m.stamina_loss > 0.0 {
            parts.push(format!("loses {}% of max stamina a second", m.stamina_loss * 100.0));
        }
        if !m.can_sprint {
            parts.push("no sprinting".into());
        }
        let _ = writeln!(w, "| {} | {:?} | {} |", e.name(), e.group(), parts.join(", "));
    }
    // How the core feels, read off `temperature::feeling` a tenth of a degree at a time.
    let mut lines = Vec::new();
    let mut last = temperature::feeling(30.0);
    for tenth in 301..=430 {
        let core = tenth as f32 / 10.0;
        let f = temperature::feeling(core);
        if f != last {
            lines.push(format!("{f:?} from {core} °C"));
            last = f;
        }
    }
    let _ = writeln!(
        w,
        "\nRested lasts {:.0} s and a minute more a comfort level. The core temperature: {:?}, then {}.\n",
        effects::rested_secs(0),
        temperature::feeling(30.0),
        lines.join(", ")
    );
}

fn climates(w: &mut String) {
    let _ = writeln!(
        w,
        "## Climates\n\nEach biome's real place (`reports/ecology/Biome climates.md`), year mean, humidity and half its summer-to-winter range, the air at sea level under an open sky (midwinter before dawn, midsummer mid afternoon), and the share of days with rain in spring, summer, autumn and winter.\n\n| biome | place | mean °C | humidity | seasons ± | coldest | hottest | rain: spring | summer | autumn | winter |\n|---|---|---|---|---|---|---|---|---|---|---|"
    );
    let midwinter = calendar::MIDSUMMER + calendar::YEAR_DAYS as f32 / 2.0;
    for c in &climate::BIOMES {
        let cold = climate::air(c, midwinter, 4.0);
        let hot = climate::air(c, calendar::MIDSUMMER, 15.0);
        let rain = c.rain.map(|r| format!("{:.0}%", r * 100.0)).join(" | ");
        let _ =
            writeln!(w, "| {} | {} | {} | {} | {} | {cold:.0} | {hot:.0} | {rain} |", c.biome, c.place, c.celsius, c.humidity, c.seasons);
    }
    let _ = writeln!(w);
    let _ = writeln!(
        w,
        "## Wind and fog\n\nEach place's mean wind in m/s and its share of days with fog, in spring, summer, autumn and winter (`reports/ecology/Wind and fog.md`), and when its fog lies (`vitals::air`). A day blows from {:.0}% to {:.0}% of the mean, a quarter more mid afternoon than before dawn, {:.1} times under a thunderstorm; still fog clears from {} to {} m/s, sea fog from {} to {}. Under {} °C what falls is snow; a day of it lays {} of a deep cover and each degree of a day's mean above freezing melts {}; near players it lies as snow layers (an eighth of a block each) from {} of a cover, {} of them when deep, burying plants from {}, a column looked at every {:.0} s and a layer melting in {:.0} s; falling snow in {} to {} m/s of wind is a blizzard; a place's air is its own and its neighbours' over a band {} blocks wide, and deep in a closed forest ({} blocks in from the edge) the wind is {:.0}% of the open's, with its chill and blizzard; dry ground (humidity under {}) lifts dust from {} to {} m/s. Out in the open the wind chills the cold (the wind chill index, 10 °C and under), and in thick fog animals see {:.0}% as far.\n\n| biome | wind: spring | summer | autumn | winter | fog: spring | summer | autumn | winter | fog lies |\n|---|---|---|---|---|---|---|---|---|---|",
        air::DAY_SPREAD.0 * 100.0,
        air::DAY_SPREAD.1 * 100.0,
        air::STORM,
        air::CLEARS.0,
        air::CLEARS.1,
        air::SEA_CLEARS.0,
        air::SEA_CLEARS.1,
        air::SNOWS_BELOW,
        air::SNOWFALL,
        air::MELT,
        air::SNOW_LAYER_FROM,
        air::SNOW_LAYERS_DEEP,
        air::SNOW_BURIES,
        air::snow_visit_secs(),
        air::SNOW_MELT_SECS,
        air::BLIZZARD.0,
        air::BLIZZARD.1,
        air::AIR_BLEND,
        air::FOREST_EDGE,
        (1.0 - air::FOREST_CALM) * 100.0,
        air::DRY,
        air::DUST.0,
        air::DUST.1,
        air::FOG_SIGHT * 100.0
    );
    for c in climate::BIOMES.iter().filter(|c| !c.underground) {
        let wind = c.wind.map(|v| format!("{v:.1}")).join(" | ");
        let fog = (0..4)
            .map(|s| format!("{:.0}%", air::fog_share(c.fog, Some(s)) * 100.0))
            .collect::<Vec<_>>()
            .join(" | ");
        let kind = match c.fog_kind {
            climate::Fog::Dawn => "before dawn to mid morning",
            climate::Fog::Sea => "off the sea, any hour",
            climate::Fog::Ice => "ice fog, cold nights and the dark",
        };
        let _ = writeln!(w, "| {} | {wind} | {fog} | {kind} |", c.biome);
    }
    let _ = writeln!(w);
}

fn plant_table(w: &mut String) {
    let _ = writeln!(
        w,
        "## Plants\n\nWhat each growing plant needs (`vitals::plants`, from `reports/ecology/Plants and crops.md`): the air it grows in (from, best, to), its water, the frost that kills it (or its fruit, or its stalk to the base), its real days to harvest and the game's minutes at the best rate, and where and in which seasons it grows at least a third of its best (sp, su, au, wi; marked * where that season's nights freeze it), with each biome's rain and warmth.\n\n| plant | real plant | °C | water | frost | real days | minutes | grows |\n|---|---|---|---|---|---|---|---|"
    );
    const SHORT: [&str; 4] = ["sp", "su", "au", "wi"];
    for p in &plants::PLANTS {
        let minutes = plants::step_secs(p, 1).map_or("the game's".into(), |s| format!("{:.0}", s / 60.0));
        let grows = climate::BIOMES
            .iter()
            .filter(|c| !c.underground)
            .filter_map(|c| {
                let seasons: Vec<String> = (0..4)
                    .filter(|&s| plants::season_rate(p, c, s) >= 0.33)
                    .map(|s| if plants::frosted(p, plants::season_low(c, s)) { format!("{}*", SHORT[s]) } else { SHORT[s].to_string() })
                    .collect();
                (!seasons.is_empty()).then(|| format!("{} ({})", c.biome.split(':').nth(1).unwrap_or(c.biome), seasons.join(" ")))
            })
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(
            w,
            "| {} | {} | {}, {}-{}, {} | {:?} | {} | {} | {minutes} | {grows} |",
            p.block,
            p.real,
            p.base,
            p.best.0,
            p.best.1,
            p.max,
            p.water,
            if p.frost > plants::HARDY { format!("{} °C", p.frost) } else { "hardy".into() },
            if p.days > 0.0 { format!("{}", p.days) } else { "-".into() }
        );
    }
    let _ = writeln!(w);
}

fn crop_table(w: &mut String) {
    let _ = writeln!(
        w,
        "## Crops\n\nWhat each crop is sown from and gives ripe (`vitals::crops`; how it grows is its plants row): picked unripe it gives back what sowed it, ripe its harvest and, with a seed of its own, up to {} more seed. Its ripe harvest tires or feeds the farmland under it. Broken tall grass in the biomes listed gives one of their crops' seed a time in {:.0}.\n\n| crop | sown from | ripe gives | stages | soil | wild seed in |\n|---|---|---|---|---|---|",
        crate::crops::EXTRA_SEEDS,
        1.0 / crate::crops::WILD_SEED_CHANCE
    );
    for c in &crate::crops::CROPS {
        let wild = if c.wild.is_empty() { "tall grass anywhere".to_string() } else { c.wild.join(", ") };
        let _ =
            writeln!(w, "| {} | {} | {} {}-{} | {} | {:?} | {wild} |", c.block, c.seed, c.gives, c.count.0, c.count.1, c.ripe + 1, c.soil);
    }
    let _ = writeln!(w);
}

fn wild_table(w: &mut String) {
    let _ = writeln!(
        w,
        "## Wild plants\n\nWhat the world grows besides grass and trees (`vitals::wild`): how each grows, the ground it stands on, what picking it gives, and the biomes it covers that share of its ground in.\n\n| block | grows | ground | gives | cover | found in |\n|---|---|---|---|---|---|"
    );
    for p in &wild::WILD {
        let found = p
            .found
            .iter()
            .map(|b| {
                let short = b.split(':').nth(1).unwrap_or(b);
                let c = wild::cover_in(p, b);
                if c == p.cover { short.to_string() } else { format!("{short} ({:.1}%)", c * 100.0) }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let _ =
            writeln!(w, "| {} | {:?} | {:?} | {} | {:.1}% | {found} |", p.block, p.form, p.ground, p.gives.unwrap_or("-"), p.cover * 100.0);
    }
    let _ = writeln!(w);
}

fn forage_table(w: &mut String) {
    let _ = writeln!(
        w,
        "## Food in the world\n\nWhat foraging animals eat of the world's blocks (`vitals::forage`), how much a mouthful feeds (a capybara holds 200), what eating it leaves, and what brings it back.\n\n| block | kind | food | eaten | grows back as |\n|---|---|---|---|---|"
    );
    for f in &forage::SOURCES {
        let block = match f.age {
            Some(a) => format!("{} (age {a})", f.block),
            None => f.block.to_string(),
        };
        let eaten = match f.eaten {
            forage::Eaten::Becomes(b) => format!("becomes {b}"),
            forage::Eaten::Age(a) => format!("back to age {a}"),
            forage::Eaten::Stays => "stays".into(),
        };
        let back = match f.renews {
            Some(p) => p.to_string(),
            None => {
                let when = if f.seasons == 15 {
                    "all year".to_string()
                } else {
                    (0..4)
                        .filter(|s| f.seasons >> s & 1 != 0)
                        .map(|s| calendar::SEASONS[s].name())
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                format!("always there in {} ({when}), worth {:.0}% of fresh growth", f.found.join(", "), forage::BROWSE * 100.0)
            }
        };
        let _ = writeln!(w, "| {block} | {} | {} | {eaten} | {back} |", f.category.name(), f.food);
    }
    let _ = writeln!(
        w,
        "\nOn the ground, animals eat dropped food of their diet one item a mouthful: every food (the foods table, by its kind and portion) and these seeds; spoiled food only carrion eaters. An item fills an animal by its weight against a meal of a tenth of the animal's mass (`animals_core::needs`).\n\n| item | kind | grams |\n|---|---|---|"
    );
    for (item, c, grams) in forage::DROPPED {
        let _ = writeln!(w, "| {item} | {} | {grams} |", c.name());
    }
    let _ = writeln!(w);
}

fn soil_table(w: &mut String) {
    let _ = writeln!(
        w,
        "## Soil and compost\n\nFarmland's fertility (`vitals::soil`): how fast crops grow at each level, what wears it, what gives it back; and what a compost bin takes.\n\n| fertility | crops grow |\n|---|---|"
    );
    for f in (0..=soil::MAX).rev() {
        let _ = writeln!(w, "| {f} | {:.0}% |", soil::growth(f) * 100.0);
    }
    let feeds = soil::FEEDS
        .iter()
        .map(|(n, v)| format!("{n} +{v}"))
        .collect::<Vec<_>>()
        .join(", ");
    let composts = soil::COMPOSTS
        .iter()
        .map(|(n, v)| format!("{n} {v}"))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(
        w,
        "\nA ripe harvest of {} takes a level, of {} gives one back; {feeds}; a season unplanted (about {:.0} game days, at grass's rate) gives one back. A compost bin fills in {} steps: spoiled food {}, {composts}; full, it rots into compost in about {:.0} minutes.\n",
        crops(crate::crops::Soil::Tires),
        crops(crate::crops::Soil::Feeds),
        soil::FALLOW_SECS / crate::DAY_SECS,
        soil::BIN_FULL,
        soil::SPOILED,
        soil::compost_secs() / 60.0
    );
}

/// The crops that leave the soil so, by name.
fn crops(soil: crate::crops::Soil) -> String {
    crate::crops::CROPS
        .iter()
        .filter(|c| c.soil == soil)
        .map(|c| c.block)
        .collect::<Vec<_>>()
        .join(", ")
}

fn difficulties(w: &mut String) {
    let _ = writeln!(
        w,
        "## Difficulty (the `base:difficulty` rule)\n\n| | digests | keeps | harm | can kill | XP lost on death | stamina costs | first nights without monsters |\n|---|---|---|---|---|---|---|---|"
    );
    for d in difficulty::ALL {
        let _ = writeln!(
            w,
            "| {} | x{} | x{} | x{} | {} | {:.0}% | x{} | {} |",
            d.name,
            d.digest,
            d.keep,
            d.harm,
            if d.deadly { "yes" } else { "no, stops at a tenth" },
            d.death_xp * 100.0,
            d.stamina_cost,
            d.grace_nights
        );
    }
    let _ = writeln!(w);
}

fn level_table(w: &mut String) {
    let _ = writeln!(w, "## Levels\n\n| level | XP to the next | XP from level 1 |\n|---|---|---|");
    for l in [1, 2, 5, 10, 25, 50, 75, 99, 100] {
        let _ = writeln!(w, "| {l} | {} | {} |", levels::to_next(l), levels::total_to(l));
    }
}
