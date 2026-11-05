//! The field journal (Ronald, 2026-11-05: "For every animal, I want a dossier"): what a player
//! can read of each species, and what they must have done to read it. A species' entry opens when
//! they first see one up close; hunting, taming (or riding) and breeding one fill in more. The facts
//! are read here from the species' own rows (`species`, `meat`, `hides`, `bodies`' sizes), never
//! copied, so the journal says what the game does.
//!
//! Each fact is a label (`journal.animals.<name>`) and a value the screen shows as words: keys
//! to translate, display names, numbers with their unit.

use crate::species::{Category, Diet, Liking, Size, Social, Species, Taming, seasons};

/// What a player has done with a species, as bits of its journal entry.
pub const SEEN: u8 = 1;
pub const HUNTED: u8 = 2;
pub const TAMED: u8 = 4;
pub const BRED: u8 = 8;
/// Every deed.
pub const ALL: u8 = SEEN | HUNTED | TAMED | BRED;
/// Each deed's name (`journal.animals.learn.<it>`: "Hunt one to learn what it drops"), by its
/// bit's order.
pub const DEEDS: [(u8, &str); 4] = [(SEEN, "seen"), (HUNTED, "hunted"), (TAMED, "tamed"), (BRED, "bred")];

/// Where a fact goes on its page: the header beside the picture (names), its chips (short
/// words), where it lives, then the survival rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Header,
    Chips,
    Lives,
    Survival,
}

/// Blocks within which an animal counts as seen up close (or through the spyglass, any range).
pub const SEEN_WITHIN: f32 = 8.0;
/// Game ticks in a day, for spans shown in days.
const DAY: f32 = vitals::calendar::DAY_TICKS as f32;

/// A fact's value, as the screen shows it.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// As written (a Latin name).
    Text(&'static str),
    /// A translation key.
    Key(String),
    /// Translation keys, shown as a list.
    Keys(Vec<String>),
    /// Display names (`item:base:fur`), shown as a list.
    Names(Vec<String>),
    /// A number with this many decimals, and its unit's key.
    Num(f32, u32, &'static str),
    /// A range of whole numbers (the same twice: one number), and its unit's key.
    Range(u32, u32, &'static str),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fact {
    /// Its label, `journal.animals.<name>`.
    pub name: &'static str,
    pub value: Value,
    /// The deed that shows it (`SEEN`, `HUNTED`...).
    pub by: u8,
    pub section: Section,
}

fn key(s: impl AsRef<str>) -> String {
    format!("journal.animals.{}", s.as_ref())
}

/// Every fact of species `s`, in the order shown: the header, the chips, where it lives, then
/// the survival rows by deed.
pub fn facts(s: &Species) -> Vec<Fact> {
    use Section::*;
    let mut f = Vec::new();
    let mut add = |name, value, by, section| f.push(Fact { name, value, by, section });
    // Seen up close: who it is, where and when it lives, what it eats, how it takes you.
    add("latin", Value::Text(s.binomial), SEEN, Header);
    add("family", Value::Key(format!("family.animals.{}", s.family.name.to_lowercase())), SEEN, Header);
    add("size", Value::Key(key(format!("size.{}", size_name(s.size)))), SEEN, Chips);
    add("mass", mass(s.bio.mass), SEEN, Chips);
    add("active", Value::Key(key(format!("active.{}", s.active.name()))), SEEN, Chips);
    add("temper", Value::Key(key(format!("temper.{}", temper(s)))), SEEN, Chips);
    add("group", group(s), SEEN, Chips);
    add("diet", Value::Key(key(format!("diet.{}", diet_name(s.diet())))), SEEN, Chips);
    add(
        "biomes",
        Value::Keys(
            s.habitats
                .iter()
                .map(|(b, _)| key(format!("biome.{}", b.replace(':', "."))))
                .collect(),
        ),
        SEEN,
        Lives,
    );
    let mut eats: Vec<&(Category, f32)> = s.eats.iter().filter(|(_, share)| *share > 0.0).collect();
    eats.sort_by(|a, b| b.1.total_cmp(&a.1));
    add(
        "eats",
        Value::Keys(
            eats.iter()
                .map(|(c, _)| key(format!("food.{}", category_name(*c))))
                .collect(),
        ),
        SEEN,
        Survival,
    );
    // Hunted: what its body gives.
    add("drops", Value::Names(drops(s)), HUNTED, Survival);
    // Tamed (or ridden): how, with what, and whether and how fast it carries you.
    add("taming", Value::Key(key(format!("taming.{}", taming_name(s.taming)))), TAMED, Survival);
    if s.taming != Taming::Wild {
        let mut foods: Vec<_> = s.foods.iter().collect();
        foods.sort_by_key(|f| (f.liking != Liking::Favorite, std::cmp::Reverse(f.food)));
        add("foods", Value::Names(foods.iter().map(|f| format!("item:{}", f.item)).collect()), TAMED, Survival);
        let rides = if s.flight > 0.0 {
            "flies"
        } else if s.seat > 0.0 {
            "rides"
        } else {
            "no"
        };
        add("riding", Value::Key(key(format!("riding.{rides}"))), TAMED, Survival);
        if s.seat > 0.0 {
            add("ride_speed", Value::Num(s.flight.max(1.0), 1, "journal.animals.unit.times_walking"), TAMED, Survival);
        }
    }
    // Bred: its seasons, its young, its life.
    add("seasons", seasons_value(s.bio.breeds), BRED, Survival);
    if s.gestation() > 0 {
        add("young", Value::Key(key("young.born")), BRED, Survival);
        add("gestation", days(s.gestation()), BRED, Survival);
    } else {
        add("young", Value::Key(key("young.eggs")), BRED, Survival);
        add("incubation", days(s.incubation()), BRED, Survival);
    }
    let (lo, hi) = s.litter();
    add("litter", Value::Range(lo, hi, "journal.animals.unit.young"), BRED, Survival);
    add("growth", days(s.growth()), BRED, Survival);
    add("lifespan", days(s.lifespan()), BRED, Survival);
    f
}

/// The species whose journal id is `id` (`id`), for a toast.
pub fn by_id(id: u32) -> Option<&'static Species> {
    crate::species::SPECIES
        .iter()
        .find(|s| self::id(s.name) == id)
}

/// Whether a player whose deeds with the species are `deeds` may read `fact`. Every fact needs
/// it seen first.
pub fn known(deeds: u8, fact: &Fact) -> bool {
    deeds & SEEN != 0 && deeds & fact.by != 0
}

fn mass(kg: f32) -> Value {
    if kg < 1.0 {
        Value::Num(kg * 1000.0, 0, "journal.animals.unit.g")
    } else if kg < 10.0 {
        Value::Num(kg, 1, "journal.animals.unit.kg")
    } else if kg < 1000.0 {
        Value::Num(kg, 0, "journal.animals.unit.kg")
    } else {
        Value::Num(kg / 1000.0, 1, "journal.animals.unit.t")
    }
}

/// A span of ticks as game days (a tenth's precision under ten).
fn days(ticks: u32) -> Value {
    let d = ticks as f32 / DAY;
    Value::Num(d, if d < 10.0 { 1 } else { 0 }, "journal.animals.unit.days")
}

fn size_name(s: Size) -> &'static str {
    match s {
        Size::Small => "small",
        Size::Medium => "medium",
        Size::Large => "large",
    }
}

fn diet_name(d: Diet) -> &'static str {
    match d {
        Diet::Carnivore => "carnivore",
        Diet::Omnivore => "omnivore",
        Diet::Herbivore => "herbivore",
    }
}

fn taming_name(t: Taming) -> &'static str {
    match t {
        Taming::Passive => "passive",
        Taming::Knockout => "knockout",
        Taming::Wild => "wild",
    }
}

/// The food categories' names (`journal.animals.food.<it>`).
pub fn category_name(c: Category) -> &'static str {
    match c {
        Category::Grass => "grass",
        Category::Leaves => "leaves",
        Category::WaterPlants => "water_plants",
        Category::Fruit => "fruit",
        Category::Seeds => "seeds",
        Category::Roots => "roots",
        Category::Lichen => "lichen",
        Category::Fungi => "fungi",
        Category::Insects => "insects",
        Category::Meat => "meat",
        Category::Fish => "fish",
        Category::Carrion => "carrion",
    }
}

/// How it takes a player near: a hunter that hunts, one that fights back, one that flees, or calm.
fn temper(s: &Species) -> &'static str {
    if s.hunts() && s.fights {
        "hunter"
    } else if s.fights {
        "fights_back"
    } else if s.shy > 0.0 {
        "shy"
    } else {
        "calm"
    }
}

/// Alone, or in a pack or herd of so many.
fn group(s: &Species) -> Value {
    let (lo, hi) = s.bio.social.group();
    let unit = match s.bio.social {
        Social::Solitary => return Value::Key(key("group.alone")),
        Social::Pack => "journal.animals.unit.pack",
        Social::Herd => "journal.animals.unit.herd",
    };
    Value::Range(lo, hi, unit)
}

/// What one gives when it dies: its meat and its hide (display names).
fn drops(s: &Species) -> Vec<String> {
    let meat = crate::meat::MEAT
        .iter()
        .filter(|(n, _)| *n == s.name)
        .map(|(_, m)| format!("item:animals:{m}"));
    let hide = crate::hides::HIDES
        .iter()
        .filter(|h| h.0 == s.name)
        .map(|h| format!("item:{}", h.1));
    meat.chain(hide).collect()
}

fn seasons_value(bits: u8) -> Value {
    if bits == 0 {
        return Value::Key(key("seasons.none"));
    }
    if bits == seasons::ALL_YEAR {
        return Value::Key(key("seasons.all_year"));
    }
    Value::Keys(
        seasons::NAMES
            .iter()
            .enumerate()
            .filter(|(i, _)| bits & seasons::bit(*i as u32) != 0)
            .map(|(_, n)| key(format!("season.{n}")))
            .collect(),
    )
}

/// Every translation key the journal's facts use, for the lang files' check.
pub fn keys() -> Vec<String> {
    let mut k: Vec<String> = Vec::new();
    for s in crate::species::SPECIES.iter() {
        for f in facts(s) {
            k.push(key(f.name));
            match f.value {
                Value::Key(x) => k.push(x),
                Value::Keys(xs) => k.extend(xs),
                Value::Num(_, _, u) | Value::Range(_, _, u) => k.push(u.to_string()),
                Value::Text(_) | Value::Names(_) => {}
            }
        }
    }
    k.extend(DEEDS.iter().map(|(_, d)| key(format!("learn.{d}"))));
    k.sort();
    k.dedup();
    k
}

/// The species' journal names, as a stable id (FNV-1a, 24 bits): a saved entry survives new
/// species and a changed order.
pub fn id(name: &str) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in name.bytes() {
        h = (h ^ b as u32).wrapping_mul(0x0100_0193);
    }
    h & 0x00ff_ffff
}

/// Saved entries: `(id << 8) | deeds`, 0 for none.
pub const ENTRIES: usize = 128;

/// A player's deeds with species `name` in `entries`.
pub fn deeds(entries: &[u32; ENTRIES], name: &str) -> u8 {
    let id = id(name);
    entries
        .iter()
        .find(|&&e| e != 0 && e >> 8 == id)
        .map_or(0, |&e| e as u8)
}

/// Adds `deed` to species `name` in `entries`; whether it was new.
pub fn note(entries: &mut [u32; ENTRIES], name: &str, deed: u8) -> bool {
    let id = id(name);
    if let Some(e) = entries.iter_mut().find(|e| **e != 0 && **e >> 8 == id) {
        let new = (*e as u8) & deed != deed;
        *e |= deed as u32;
        return new;
    }
    if let Some(e) = entries.iter_mut().find(|e| **e == 0) {
        *e = (id << 8) | deed as u32;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::species::SPECIES;

    #[test]
    fn ids_are_distinct_and_fit() {
        let mut ids: Vec<u32> = SPECIES.iter().map(|s| id(s.name)).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), SPECIES.len(), "two species share a journal id");
        assert!(SPECIES.len() <= ENTRIES, "more species than journal entries");
        assert!(ids.iter().all(|&i| i != 0));
    }

    #[test]
    fn notes_add_up() {
        let mut e = [0u32; ENTRIES];
        assert_eq!(deeds(&e, "capybara"), 0);
        assert!(note(&mut e, "capybara", SEEN));
        assert!(!note(&mut e, "capybara", SEEN));
        assert!(note(&mut e, "capybara", TAMED));
        assert!(note(&mut e, "grey_wolf", HUNTED));
        assert_eq!(deeds(&e, "capybara"), SEEN | TAMED);
        assert_eq!(deeds(&e, "grey_wolf"), HUNTED);
    }

    #[test]
    fn every_species_has_its_basics_and_its_young() {
        for s in SPECIES.iter() {
            let f = facts(s);
            for name in ["latin", "mass", "biomes", "active", "diet", "temper", "drops", "seasons", "young", "litter", "lifespan"] {
                assert!(f.iter().any(|x| x.name == name), "{}: no {name}", s.name);
            }
            assert!(f.iter().filter(|x| x.by == SEEN).count() >= 6, "{}: few facts when seen", s.name);
            // The sections in their page order, so the screen can lay them out as they come.
            let order: Vec<Section> = f.iter().map(|x| x.section).collect();
            assert!(order.windows(2).all(|w| w[0] as u8 <= w[1] as u8), "{}: sections out of order", s.name);
        }
        let deer = SPECIES.iter().find(|s| s.name == "red_deer").unwrap();
        let drops = facts(deer).into_iter().find(|x| x.name == "drops").unwrap();
        assert_eq!(drops.value, Value::Names(vec!["item:animals:venison".into(), "item:base:hide".into()]));
    }

    #[test]
    fn a_fact_needs_its_deed_and_a_sighting() {
        let f = Fact { name: "drops", value: Value::Text(""), by: HUNTED, section: Section::Survival };
        assert!(!known(HUNTED, &f), "hunted unseen (a shot from afar) still needs a look");
        assert!(!known(SEEN, &f));
        assert!(known(SEEN | HUNTED, &f));
    }

    #[test]
    fn every_key_is_in_english_and_russian() {
        for (name, lang) in
            [("en_us", include_str!("../../assets/lang/en_us.lang")), ("ru_ru", include_str!("../../assets/lang/ru_ru.lang"))]
        {
            let have: std::collections::HashSet<&str> = lang
                .lines()
                .filter_map(|l| l.split_once(" = ").map(|(k, _)| k.trim()))
                .collect();
            let missing: Vec<String> = keys()
                .into_iter()
                .filter(|k| !have.contains(k.as_str()))
                .collect();
            assert!(missing.is_empty(), "{name} lacks {missing:?}");
        }
    }
}
