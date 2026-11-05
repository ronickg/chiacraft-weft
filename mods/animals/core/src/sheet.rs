//! Every species' settings on one page (`mods/animals/SPECIES.md`), written from the species
//! table itself so the page can't drift from the game: what each animal is, its body, its life
//! (the real facts and the numbers derived from them), balance, taming, fighting and riding, its
//! stats, and the rules shared by all. `web/sheet.mjs` writes the page through the WASM;
//! `web/test.mjs` fails while the page is out of date.

use std::fmt::Write;

use crate::balance;
use crate::biology::{self, Biology};
use crate::evolution;
use crate::life::{self, DAY};
use crate::needs;
use crate::species::{Liking, SPECIES, STAT_NAMES, STATS, Species, TAMED_STEP, WILD_STEP};

/// A number of ticks as in-game days, as short as it reads.
fn days(t: u32) -> String {
    let d = t as f64 / DAY as f64;
    format!("{} d", trim(d, if d >= 10.0 { 1 } else { 2 }))
}

/// A real span in days, in years from a year.
fn real(d: f32) -> String {
    if d >= 365.0 { format!("{} y", trim(d as f64 / 365.0, 1)) } else { format!("{} d", trim(d as f64, 0)) }
}

fn trim(x: f64, places: usize) -> String {
    let s = format!("{x:.places$}");
    if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s }
}

fn name(s: &Species) -> String {
    match s.breed {
        Some(b) => format!("{} ({b})", s.name),
        None => s.name.to_string(),
    }
}

fn lower(x: impl std::fmt::Debug) -> String {
    format!("{x:?}").to_lowercase()
}

/// The page.
pub fn markdown() -> String {
    let mut o = String::new();
    let w = &mut o;
    let _ = writeln!(w, "# Animals: every species' settings\n");
    let _ = writeln!(
        w,
        "Written from the game's species table (`core/src/species.rs`) by `core/src/sheet.rs`; don't edit it by \
         hand. After changing a species, rewrite it:\n\n    mods/animals/web/build.sh && node mods/animals/web/sheet.mjs\n\n\
         `node mods/animals/web/test.mjs` fails while it is out of date. Time is in in-game days (24 000 ticks, 20 \
         minutes of play); \"real\" is the real animal's. Breeding numbers are derived from the real facts \
         (`core/src/biology.rs`) unless a species sets one itself (marked **set**). Why the numbers are what they \
         are: `notes/ANIMALS.md`. Coat genes: `genetics/<species>.json` and `genetics/core.json`.\n"
    );

    // At a glance.
    let _ = writeln!(w, "## At a glance\n");
    let _ = writeln!(
        w,
        "| Species | Scientific name | Rarity | Size | Height | Mass | Lives in | Diet | Taming | Carrying | Grown in | Lives | Litter | Room |"
    );
    let _ = writeln!(w, "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    for s in &SPECIES {
        let carry = if s.gestation() > 0 { days(s.gestation()) } else { format!("eggs, hatch {}", days(s.incubation())) };
        let _ = writeln!(
            w,
            "| {} | *{}* | {} | {} | {} m | {} kg | {} | {} | {} | {carry} | {} | {} | {} | {} |",
            name(s),
            s.binomial,
            s.bio.rarity.name(),
            lower(s.size),
            s.height.0,
            s.bio.mass,
            lower(s.bio.social),
            lower(s.diet()),
            lower(s.taming),
            days(s.growth()),
            days(s.lifespan()),
            litter(s),
            s.crowd(),
        );
    }
    let _ = writeln!(w);

    // Stats side by side.
    let _ = writeln!(w, "## Stats at level 1\n");
    let _ = writeln!(w, "Damage and speed are each kind's own (1 = 100%).\n");
    let _ = write!(w, "| Species |");
    for n in STAT_NAMES {
        let _ = write!(w, " {n} |");
    }
    let _ = writeln!(w, " hunger (s a point) |");
    let _ = writeln!(w, "|---|{}---|", "---|".repeat(STATS));
    for s in &SPECIES {
        let _ = write!(w, "| {} |", name(s));
        for v in s.base {
            let _ = write!(w, " {v} |");
        }
        let _ = writeln!(w, " {} |", s.hunger);
    }
    let _ = writeln!(w);

    web(w);
    for s in &SPECIES {
        species(w, s);
    }
    shared(w);
    o
}

/// Who eats what where (`web.rs`): each biome's plant food by season, its plant eaters, hunters
/// and carrion eaters, and the web's gaps with what fills them.
fn web(w: &mut String) {
    use vitals::food::Category;
    let _ = writeln!(
        w,
        "## The food web
"
    );
    let _ = writeln!(
        w,
        "Each biome's web, from the species' habitats and diets and the world's plant food (`core/src/web.rs`). \
         Plant food is how much of any plant eater's food grows or is found there in each season, against fresh growth \
         at its best (`vitals::forage`); the farm animals' wild kinds graze every overworld land biome. Grazing: in \
         the best season for grass, what its grazers eat a day (as many of each as live together, `crowd_in`, the \
         biome's stocking) against what its tall grass grows back within the crowd's radius, real grazing's quarter \
         to half (`vitals::forage::GRAZED`, checked), and the most of that ground their dung pats cover at once \
         (`vitals::soil`), in the season they lie longest.\n"
    );
    let _ =
        writeln!(w, "| Biome | Plant food (spring, summer, autumn, winter) | Grazing (best season) | Plant eaters | Hunters | Carrion |");
    let _ = writeln!(w, "|---|---|---|---|---|---|");
    let names = |f: &dyn Fn(&Species) -> bool, biome: &str| {
        let v: Vec<String> = SPECIES
            .iter()
            .filter(|s| s.habitats.iter().any(|&(b, _)| b == biome) && f(s))
            .map(name)
            .collect();
        if v.is_empty() { "**none**".to_string() } else { v.join(", ") }
    };
    for biome in crate::web::biomes() {
        let food: Vec<String> = (0..4)
            .map(|i| format!("{:.0}%", 100.0 * crate::web::food_in(Category::plant, biome, i)))
            .collect();
        let grazing = crate::web::grazing(biome).map_or("-".to_string(), |g| {
            format!(
                "{}: eaten {:.0}% of the regrowth; dung pats {:.2}% of the ground",
                crate::web::SEASONS[g.season],
                100.0 * g.eaten / g.regrown.max(1e-6),
                100.0 * g.pat_cover
            )
        });
        let _ = writeln!(
            w,
            "| {biome} | {} | {grazing} | {} | {} | {} |",
            food.join(", "),
            names(&|s| s.plant_share() >= 0.5, biome),
            names(&|s| s.hunts() && !s.forages_wild(), biome),
            names(&|s| s.eats(Category::Carrion), biome)
        );
    }
    let gaps = crate::web::gaps();
    let _ = writeln!(w, "\n### Gaps in the web\n");
    if gaps.is_empty() {
        let _ = writeln!(w, "None.\n");
        return;
    }
    let _ = writeln!(w, "What the webs still lack, and what the plan fills it with (`web::WAITING`):\n");
    for g in &gaps {
        let why = crate::web::WAITING
            .iter()
            .find(|&&(k, _)| k == g.key)
            .map_or("**not planned**".to_string(), |&(_, why)| why.to_string());
        let _ = writeln!(w, "- {}: {why}", g.key);
    }
    let _ = writeln!(w);
}

/// Its breeding season, in words ("spring, summer and autumn, most in spring"; for one of the
/// Cretaceous, whose year is a wet and a dry season, "the wet season").
fn breeding_season(s: &Species) -> String {
    use crate::biology::seasons;
    let b = &s.bio;
    if s.habitats
        .iter()
        .any(|(biome, _)| biome.starts_with("animals:"))
    {
        let (wet, dry) = (b.breeds & seasons::WET, b.breeds & !seasons::WET);
        return match (wet, dry) {
            (seasons::SPRING, 0) => "early in the wet season".into(),
            (_, 0) => "the wet season".into(),
            (0, _) => "the dry season".into(),
            _ => "all year".into(),
        };
    }
    let names = |bits: u8| -> Vec<&str> {
        (0..4)
            .filter(|&i| bits & seasons::bit(i) != 0)
            .map(|i| seasons::NAMES[i as usize])
            .collect()
    };
    let list = |n: Vec<&str>| match n.len() {
        0 => String::new(),
        1 => n[0].to_string(),
        _ => format!("{} and {}", n[..n.len() - 1].join(", "), n[n.len() - 1]),
    };
    let all = if b.breeds == seasons::ALL_YEAR { "all year".to_string() } else { list(names(b.breeds)) };
    if b.peak != 0 && b.peak != b.breeds { format!("{all}, most in {}", list(names(b.peak))) } else { all }
}

fn litter(s: &Species) -> String {
    match s.litter() {
        (a, b) if a == b => a.to_string(),
        (a, b) => format!("{a}-{b}"),
    }
}

fn species(w: &mut String, s: &Species) {
    let b: &Biology = &s.bio;
    let t = &s.tune;
    let set = |on: bool| if on { " **set**" } else { "" };
    let _ = writeln!(w, "## {}\n", name(s));
    let tree: Vec<&str> = s
        .family
        .above
        .iter()
        .copied()
        .chain([s.family.name, s.subfamily])
        .collect();
    let _ = writeln!(w, "*{}*, {}. Model `{}`, entity `animals:{}`.\n", s.binomial, tree.join(" › "), s.model, s.breed.unwrap_or(s.name));

    let (few, most) = s.bio.social.group();
    let group = if few == most { few.to_string() } else { format!("{few}-{most}") };
    let _ = writeln!(w, "| Where it lives | how common there |\n|---|---|");
    for &(biome, share) in s.habitats {
        let _ = writeln!(
            w,
            "| `{biome}` | {} |",
            if share >= 1.0 { "commonest".to_string() } else { format!("{}%", trim(share as f64 * 100.0, 0)) }
        );
    }
    let _ = writeln!(
        w,
        "\nSpawns by day on grass, snow or sand, in groups of {group} ({}), at {}% of spawn attempts that pick it where commonest ({}).\n",
        lower(s.bio.social),
        trim(s.bio.rarity.spawn_chance() as f64 * 100.0, 2),
        s.bio.rarity.name()
    );

    let _ = writeln!(w, "| Body | |\n|---|---|");
    let _ = writeln!(w, "| size class | {} |", lower(s.size));
    let _ = writeln!(w, "| height (grown, average) | {} m at the {} |", s.height.0, s.height.1);
    let _ = writeln!(w, "| males against females | {}x as tall |", s.dimorphism);
    let _ = writeln!(w, "| size spread (an individual's, either way) | {}% |", trim(s.size_spread as f64 * 100.0, 0));
    let _ = writeln!(
        w,
        "| newborn's size | {}% of grown{} |",
        trim(s.birth_size as f64 * 100.0, 0),
        if s.birth_size >= 1.0 { " (its own young model)" } else { "" }
    );
    let _ = writeln!(w);

    let _ = writeln!(w, "| Life | real | in the game |\n|---|---|---|");
    let _ = writeln!(w, "| mass | {} kg | |", b.mass);
    if b.gestation > 0.0 || s.gestation() > 0 {
        let _ = writeln!(w, "| carries her young | {} | {}{} |", real(b.gestation), days(s.gestation()), set(t.gestation.is_some()));
    } else {
        let _ = writeln!(w, "| lays | | {} after mating |", days(life::LAYING));
        let _ = writeln!(w, "| eggs hatch in | {} | {}{} |", real(b.incubation), days(s.incubation()), set(t.incubation.is_some()));
    }
    let _ = writeln!(w, "| litter or clutch | {} on average | {}{} |", b.litter, litter(s), set(t.litter.is_some()));
    let _ = writeln!(w, "| between litters | {} | rests {}{} |", real(b.interval), days(s.recovery()), set(t.recovery.is_some()));
    let out_of_season = if b.breeds == crate::biology::seasons::ALL_YEAR { "" } else { "out of it, only a tame well cared for" };
    let _ = writeln!(w, "| breeds in | {} | {out_of_season} |", breeding_season(s));
    let _ = writeln!(
        w,
        "| comfortable in | {} to {} °C air | {} |",
        b.comfort.0,
        b.comfort.1,
        if b.cold_blooded {
            "cold-blooded: slower in the cold"
        } else {
            "seeks shade, water or warmth outside it; a tame eats faster, far outside loses health"
        }
    );
    let _ = writeln!(w, "| grows up in | {} | {}{} |", real(b.maturity), days(s.growth()), set(t.growth.is_some()));
    let _ = writeln!(
        w,
        "| lives | {} | {}{}, give or take a fifth; an elder from {}% of it |",
        real(b.lifespan),
        days(s.lifespan()),
        set(t.lifespan.is_some()),
        trim(life::ELDERLY * 100.0, 0)
    );
    let _ = writeln!(
        w,
        "| lives in | {} | room for {}{} within {} blocks |",
        lower(b.social),
        s.crowd(),
        set(t.crowd.is_some()),
        life::CROWD_RADIUS
    );
    let _ = writeln!(w);

    let r = balance::report(s);
    let _ = writeln!(
        w,
        "**Balance** ({}): a generation {}; a line four deep in {} ({} h of play); {} young per mother in 10 days, {} in her life.\n",
        b.rarity.name(),
        format!("{} d", trim(r.generation, 1)),
        format!("{} d", trim(r.fourth, 0)),
        trim(r.fourth / 3.0, 0),
        trim(r.young_per_ten, 1),
        trim(r.lifetime_young, 0)
    );
    for c in balance::checks(s) {
        let _ = writeln!(w, "- {} {}: {} (want {})", if c.ok { "✓" } else { "✗" }, c.name, c.found, c.target);
    }
    let _ = writeln!(w);

    let _ = writeln!(w, "| Taming and food | |\n|---|---|");
    let _ = writeln!(w, "| how | {} |", lower(s.taming));
    let _ = writeln!(w, "| diet | {} |", lower(s.diet()));
    let eats = s
        .eats
        .iter()
        .map(|(c, share)| format!("{} {:.0}%", c.name().replace('_', " "), share * 100.0))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(w, "| eats | {eats} |");
    if s.hunts() {
        let (lo, hi) = (s.bio.mass * s.prey.0, s.bio.mass * s.prey.1);
        let prey = s.prey_at_home().collect::<Vec<_>>().join(", ");
        let prey = if prey.is_empty() { "none yet: it waits for its prey to come to the world".to_string() } else { prey };
        let _ = writeln!(
            w,
            "| hunts | {:.1}%-{:.0}% of its mass alone ({lo:.2}-{hi:.0} kg); at home: {prey} |",
            s.prey.0 * 100.0,
            s.prey.1 * 100.0
        );
    }
    // A meal by mass (`needs::fills`): what fills it, and how many of its biggest prey at home.
    let kills = match needs::kills_to_fill(s) {
        Some(k) if s.hungry_in_the_wild() && !s.forages_wild() => {
            if k <= 1.0 {
                "; its biggest prey at home fills it in one kill".to_string()
            } else {
                format!("; its biggest prey at home in {} kills", trim(k as f64, 1))
            }
        }
        _ => String::new(),
    };
    let _ = writeln!(w, "| a meal that fills it | {} kg{kills} |", trim((s.bio.mass * needs::MEAL) as f64, 3));
    // Water (`needs::drink_lasts`): how much it drinks of what it needs, and how long a drink lasts.
    let need = needs::litres_a_day(s, s.bio.mass);
    let drinks = match needs::drink_lasts(s, s.bio.mass) {
        Some(secs) => format!(
            "{}% of its {} L a day; a drink lasts {} min, thirsty after {}",
            trim(needs::drinks(s) as f64 * 100.0, 0),
            trim(need as f64, 3),
            trim(secs as f64 / 60.0, 0),
            trim(secs as f64 * (1.0 - needs::THIRSTY) as f64 / 60.0, 0)
        ),
        None if s.in_water() => "never: it lives in the water".to_string(),
        None => "never".to_string(),
    };
    let _ = writeln!(w, "| drinks | {drinks} |");
    // How much of its food grows at home, season by season (`vitals::forage::season_lushness`):
    // how many come there and how fed (`notes/ECOLOGY.md`, part 7).
    if s.forages_wild() {
        let homes = s
            .habitats
            .iter()
            .filter_map(|&(biome, _)| {
                let c = vitals::climate::of(biome)?;
                let by = (0..4)
                    .map(|season| {
                        vitals::forage::season_lushness(|k| s.eats(k), biome, c.celsius, c.seasons, c.rain, Some(season)).unwrap_or(0.0)
                    })
                    .map(|l| format!("{:.0}", l * 100.0))
                    .collect::<Vec<_>>()
                    .join("/");
                Some(format!("{} {by}%", biome.split(':').nth(1).unwrap_or(biome)))
            })
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(w, "| its food grows at home (spring/summer/autumn/winter) | {homes} |");
    }
    let _ = writeln!(w, "| feedings to tame at level 1 | {} (2.5x at level 150) |", s.feedings);
    let _ = writeln!(
        w,
        "| scared by a walking player within | {} |",
        if s.shy > 0.0 { format!("{} blocks (twice that running)", s.shy) } else { "nothing".into() }
    );
    let foods: Vec<String> = s
        .foods
        .iter()
        .map(|f| {
            let like = match f.liking {
                Liking::Eats => "",
                Liking::Favorite => ", favorite",
                Liking::Kibble => ", kibble",
            };
            format!("`{}` {}{like}", f.item, f.food)
        })
        .collect();
    let _ = writeln!(w, "| eats (food points) | {} |", foods.join("; "));
    let _ = writeln!(w);

    let _ = writeln!(w, "| Fighting, riding, the wild | |\n|---|---|");
    let _ = writeln!(w, "| fights | {} |", if s.fights { format!("yes: {}", s.attacks.join(", ")) } else { "no".into() });
    let _ = writeln!(w, "| reach past its body | {} blocks |", s.reach);
    let _ = writeln!(w, "| signature | {} |", s.signature);
    let _ =
        writeln!(w, "| ridden | {} |", if s.seat > 0.0 { format!("seat {} up, {} forward", s.seat, s.seat_forward) } else { "no".into() });
    let _ = writeln!(w, "| flies ridden | {} |", if s.flight > 0.0 { format!("{}x its walking speed", s.flight) } else { "no".into() });
    let _ = writeln!(w, "| camouflage (cost of a coat that stands out) | {} |", s.camouflage);
    let _ = writeln!(w);

    if s.wild_step != WILD_STEP || s.tamed_step != TAMED_STEP {
        let _ = writeln!(w, "Its own steps a level (as a share of the base): wild {:?}, tamed {:?}.\n", s.wild_step, s.tamed_step);
    }
}

fn shared(w: &mut String) {
    let _ = writeln!(w, "## Shared by every species\n");
    let _ = writeln!(w, "| Levels | {} |\n|---|---|", STAT_NAMES.join(" / "));
    let _ = writeln!(w, "| a wild point adds (share of the base) | {} |", steps(&WILD_STEP));
    let _ = writeln!(w, "| a tamed point adds | {} |", steps(&TAMED_STEP));
    let _ = writeln!(w);

    let pace = |p: (f64, f64)| format!("{} · real days^{}", p.0, p.1);
    let _ = writeln!(w, "| Pacing: real days to game days (`biology.rs`) | |\n|---|---|");
    let _ = writeln!(w, "| carrying, incubating | {} |", pace(biology::CARRYING));
    let _ = writeln!(w, "| growing up | {} |", pace(biology::GROWING));
    let _ = writeln!(w, "| a life | {} |", pace(biology::LIVING));
    let _ = writeln!(w, "| rest between litters | {} |", pace(biology::RESTING));
    let _ = writeln!(
        w,
        "| room near | {} · (mass / 4 kg)^{}, x{} a pack, x{} a herd, at least 3 |",
        biology::ROOM,
        biology::ROOM_BY_MASS,
        biology::Social::Pack.room(),
        biology::Social::Herd.room()
    );
    let _ = writeln!(w);

    let _ = writeln!(w, "| Hunger (`needs.rs`) | |\n|---|---|");
    let _ = writeln!(
        w,
        "| a meal that fills it | {}% of its mass: a kill, a mouthful off the ground, by mass |",
        trim(needs::MEAL as f64 * 100.0, 0)
    );
    let _ = writeln!(
        w,
        "| a wild one arrives fed | {}% where its food grows least (or its prey are thinnest), all of it where most |",
        trim(needs::ARRIVES_FED as f64 * 100.0, 0)
    );
    let _ = writeln!(w, "| starving, a wild one dies in | {} s on an empty stomach |", needs::STARVES_SECS);
    let _ = writeln!(
        w,
        "| water needed a day | {} · kg^{} L (a mammal's), x{} cold-blooded; a species drinks its share of it (`DRINKS`) |",
        needs::WATER.0,
        needs::WATER.1,
        needs::COLD_BLOODED_WATER
    );
    let _ = writeln!(
        w,
        "| a drink lasts | {}% of its mass spared, a real day of drinking {} s of play |",
        trim(needs::SPARE as f64 * 100.0, 0),
        needs::DRINK_DAY_SECS
    );
    let _ = writeln!(
        w,
        "| thirsty | under {}% water, drinking {}% a gulp every {} s until {}%; parched, a wild one dies in {} s |",
        trim(needs::THIRSTY as f64 * 100.0, 0),
        trim(needs::GULP as f64 * 100.0, 0),
        trim(needs::GULP_TICKS as f64 / 20.0, 1),
        trim(needs::SLAKED as f64 * 100.0, 0),
        needs::PARCHED_SECS
    );
    let _ = writeln!(w);

    let _ = writeln!(w, "| Breeding and life (`life.rs`) | |\n|---|---|");
    let _ = writeln!(w, "| in love for | {} s |", life::LOVE_TICKS / 20);
    let _ = writeln!(w, "| a male rests after mating | {} min |", life::MALE_REST / 20 / 60);
    let _ = writeln!(w, "| a wild one falls in love, with room | {} times a day |", life::WILD_LOVE_PER_DAY);
    let _ = writeln!(w, "| breeding when at its room | {}% of that |", trim(life::BREEDING_WHEN_FULL * 100.0, 0));
    let _ = writeln!(w, "| no breeding past | {}x its room |", life::CROWDED);
    let _ = writeln!(
        w,
        "| deaths of want, at twice its room | {}% of grown ones a day (elders x3, young x2) |",
        trim(life::DEATHS_WHEN_CROWDED * 100.0, 0)
    );
    let _ = writeln!(w, "| elders crowd | half |");
    let _ = writeln!(w, "| time away caught up past | {} s |", life::AWAY / 20);
    let _ = writeln!(w);

    let _ = writeln!(w, "| Evolution (`evolution.rs`) | |\n|---|---|");
    let _ = writeln!(w, "| a place | {} blocks a side, per habitat |", evolution::CELL);
    let _ = writeln!(w, "| a generation | {} |", days(evolution::GENERATION as u32));
    let _ = writeln!(w, "| population a place | {} |", evolution::POPULATION);
    let _ = writeln!(w, "| selection strength | {} |", evolution::SELECTION);
    let _ = writeln!(w, "| mutation a gene a generation | {} |", evolution::MUTATION);
    let _ = writeln!(w, "| migration from neighbors | {} |", evolution::MIGRATION);
    let _ = writeln!(w, "| bigger in the cold (Bergmann) | {} |", evolution::BERGMANN);
    let _ =
        writeln!(w, "| a new place's history | {} generations (a revisited one catches up {}) |", evolution::HISTORY, evolution::CATCH_UP);
    let _ = writeln!(w);

    let _ = writeln!(w, "| Spawning (`habitats.rs`, `biology.rs`) | |\n|---|---|");
    for r in [biology::Rarity::Common, biology::Rarity::Uncommon, biology::Rarity::Rare, biology::Rarity::Legendary] {
        let _ = writeln!(w, "| {} | {}% of attempts that pick it, where commonest |", r.name(), trim(r.spawn_chance() as f64 * 100.0, 2));
    }
    for x in [biology::Social::Solitary, biology::Social::Pack, biology::Social::Herd] {
        let (few, most) = x.group();
        let _ = writeln!(w, "| {} | groups of {} |", lower(x), if few == most { few.to_string() } else { format!("{few}-{most}") });
    }
    let _ = writeln!(w);

    let _ = writeln!(w, "| Balance targets (`balance.rs`): a line four generations deep | |\n|---|---|");
    for r in [biology::Rarity::Common, biology::Rarity::Uncommon, biology::Rarity::Rare, biology::Rarity::Legendary] {
        let (lo, hi) = balance::fourth_target(r);
        let _ = writeln!(w, "| {} | {lo}-{hi} d |", r.name());
    }
}

fn steps(v: &[f32; STATS]) -> String {
    v.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(" / ")
}
