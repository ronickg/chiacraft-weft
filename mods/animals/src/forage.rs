//! Eating from the world (`notes/ECOLOGY.md`, part 5): a hungry animal looks round for food of
//! its diet (`Species::eats`) that grows there (`vitals::forage::SOURCES`: tall grass, ripe
//! berries, leaves), walks to it and eats a mouthful at a time until it's full. What it eats
//! changes as eating it leaves it: grass is grazed short and grows back by warmth and rain
//! (`base:grazed_grass`), a berry bush starts its fruit again. So a lush place keeps its animals
//! fed and a dry season thins them, through their condition (`comfort::condition`).
//!
//! Its place feeds as many as its food grows: plant eaters come by natural spawning as often as
//! their food grows there this season (`season_lush`), and each mouthful feeds as well as the
//! plant grows today (dry-season grass is poor fodder).
//!
//! Food on the ground comes first (`notes/ECOLOGY.md`, parts 5 and 8): dropped food of its diet
//! within `ITEM_LOOK` blocks (`vitals::forage::dropped`: seeds, fruit, meat, a kill's carcass;
//! spoiled food only to carrion eaters) is eaten an item a mouthful, each filling it by its weight
//! against its own mass (`animals_core::needs::fills`): a raven makes a meal of a chop, a deer
//! hardly notices a berry. A hunter with food lying near eats that before it hunts (`hunt.rs`).
//!
//! Wild plant eaters and hunters with prey get hungry (`Species::hungry_in_the_wild`,
//! `animal::upkeep`); tames forage when hungry and not told to stay (`commands::hold` comes
//! first).

use vitals::forage::{Eaten, Source};
use weft_sdk::*;

use crate::dist2;

use crate::animal::{self, ANIMAL};
use crate::species::{self, SPECIES, Species};

/// Where it is going to eat and until when (not saved): x, y, z of the food (NaN: none near),
/// and the time it looks again (0: not foraging).
static FORAGE: Component<[f64; 4]> = Component::named("animal_forage");
/// The dropped item it is going to eat (0: a block, or nothing; not saved).
static ITEM: Component<u64> = Component::named("animal_forage_item");
/// The last place it found food growing (x, y, z, and 1 where there is one): a hungry animal
/// looks round there first, where more of it likely grows (`reports/Creature AI.md`, step 1).
/// Not saved.
static PATCH: Component<[i32; 4]> = Component::named("animal_forage_patch");
static GRAZED_GRASS: Block = Block::named("base:grazed_grass");
static DUNG: Item = Item::named("base:dung");
static DUNG_PAT: Block = Block::named("base:dung_pat");
static TALL_GRASS: Block = Block::named("base:tall_grass");
static PLANT_SOIL: Tag = Tag::named("plant_soil");

/// Blocks it looks round, and up and down.
const LOOK: i32 = 8;
const LOOK_UP: i32 = 3;
/// Blocks it sees (and smells) food on the ground from.
pub const ITEM_LOOK: f64 = 12.0;
/// It starts foraging under this share of its food, and stops when this full (`needs`).
use animals_core::needs::{FULL, HUNGRY};
/// Ticks: chewing a mouthful, walking before it looks again, wandering when nothing is near.
const CHEW: f64 = 30.0;
const WALK: f64 = 160.0;
const NOTHING: f64 = 300.0;

pub fn init(r: &mut Registry) {
    r.entity_component(&FORAGE, [0.0; 4], 0);
    r.entity_component(&ITEM, 0u64, 0);
    r.entity_component(&PATCH, [0i32; 4], 0);
    for s in &SPECIES {
        r.edit(s.kind).attach(&FORAGE);
        r.edit(s.kind).attach(&ITEM);
        r.edit(s.kind).attach(&PATCH);
    }
    // A place holds as many as it feeds (`notes/ECOLOGY.md`, part 7): plant eaters come as often
    // as their food grows there now, so fewer in a drought or a winter.
    // And none come past its room (`life::room_for`): the engine's spawner caps only all creatures
    // together, so packs were added where their kind already filled the place.
    r.before_event(&question::NATURAL_SPAWN, |q| {
        let kind = q.query().other as u32;
        let Some(s) = SPECIES.iter().find(|s| s.kind.id() == kind) else {
            return Verdict::Abstain;
        };
        let w = q.world();
        let coming = q.query().amount.max(1.0) as u32;
        let near = kind_near(&w, s, q.pos().to_array());
        let crowd = crate::species::crowd_at(&w, s, q.pos());
        if !animals_core::life::room_for(near, coming, crowd) {
            weft_sdk::info!("no room for {coming} {} at {:?}: {near:.1} near, room for {crowd}", s.name, q.pos());
            return Verdict::Deny;
        }
        if !s.forages_wild() {
            return Verdict::Abstain;
        }
        let floor = animals_core::needs::SPAWN_FLOOR;
        let come = floor + (1.0 - floor) * season_lush(&w, s, q.pos().to_array());
        if (w.random_below(10_000) as f32) < come * 10_000.0 { Verdict::Abstain } else { Verdict::Deny }
    });
}

/// How much its kind crowds the spot `pos` (`life::crowding` each, within `life::CROWD_RADIUS`):
/// what natural spawning weighs against its room (`life::room_for`; `/animal room` shows it).
pub fn kind_near(w: &World, s: &Species, pos: [i32; 3]) -> f64 {
    w.entities_near(pos.map(|v| v as f64 + 0.5), animals_core::life::CROWD_RADIUS, Some(s.kind))
        .into_iter()
        .filter_map(|o| w.component(o, &ANIMAL))
        .map(|o| animals_core::life::crowding(o.stage))
        .sum()
}

/// How much of its food its biome grows through this season, 0 to 1
/// (`vitals::forage::season_lushness`); 1 for one that doesn't forage.
pub fn season_lush(w: &World, s: &animals_core::species::Species, pos: [i32; 3]) -> f32 {
    let biome = w.biome(pos);
    let c = base_api::CLIMATE
        .on_biome(biome)
        .unwrap_or(base_api::TEMPERATE);
    let season = crate::breeding::season_bit(w).map(|bit| bit.trailing_zeros() as usize);
    let name = name_of(abi::entry_kind::BIOME, biome);
    vitals::forage::season_lushness(|k| s.eats(k), name, c.celsius, c.seasons, c.rain, season).unwrap_or(1.0)
}

/// How much of its food the place at `pos` grows now, 0 to 1 (`vitals::forage::lushness`); 1 for
/// one that doesn't forage.
pub fn lush(w: &World, s: &Species, pos: [i32; 3]) -> f32 {
    let g = base_api::ground_at(w, pos);
    let biome = name_of(abi::entry_kind::BIOME, w.biome(pos));
    vitals::forage::lushness(|c| s.eats(c), biome, g.mean_air, g.soil).unwrap_or(1.0)
}

/// Whether it is foraging or drinking (a resting animal settles only when it isn't: hungry or
/// thirsty, it eats and drinks first).
pub fn busy(w: &World, id: u64) -> bool {
    // A search left from before it was fed (a tame held by its orders, whose forage goal doesn't
    // run to clear it) isn't one: only a hungry one forages.
    let hungry = w
        .component(id, &ANIMAL)
        .is_some_and(|a| (a.food as f32) < a.stat(species::FOOD) * FULL);
    crate::drink::busy(w, id)
        || (hungry
            && w.component(id, &FORAGE)
                .is_some_and(|f| f[3] != 0.0 && !f[0].is_nan()))
}

/// What it would eat of the block `state`, if anything: the first of the state's food rows
/// (`rows`) its diet and the season allow, as `vitals::forage::of` picks.
fn food(s: &Species, state: u32, season: Option<usize>) -> Option<&'static Source> {
    rows(state, |r| {
        r.iter()
            .map(|&i| &vitals::forage::SOURCES[i as usize])
            .find(|src| s.eats(src.category) && src.in_season(season))
    })
}

/// The food rows (`vitals::forage::SOURCES`) that block state `state` is, by its block and age:
/// read from its name once a state, not compared by name for every block a forager looks over
/// (C7).
fn rows<R>(state: u32, f: impl FnOnce(&[u8]) -> R) -> R {
    thread_local! {
        static ROWS: std::cell::RefCell<std::collections::HashMap<u32, Vec<u8>>> = Default::default();
    }
    ROWS.with(|m| {
        let mut m = m.borrow_mut();
        let r = m.entry(state).or_insert_with(|| {
            let name = block_name(state)
                .split('[')
                .next()
                .unwrap_or("")
                .to_string();
            let age: Option<u8> = state_prop(state, "age").and_then(|a| a.parse().ok());
            vitals::forage::SOURCES
                .iter()
                .enumerate()
                .filter(|(_, src)| src.block == name && (src.age.is_none() || src.age == age))
                .map(|(i, _)| i as u8)
                .collect()
        });
        f(r)
    })
}

/// The season now (0 spring), where there are seasons.
fn season(w: &World) -> Option<usize> {
    crate::breeding::season_bit(w).map(|bit| bit.trailing_zeros() as usize)
}

/// Blocks round the patch it last found food in that it looks first, and how far it goes back to
/// one.
const PATCH_LOOK: i32 = 3;
const RECALL: i32 = 2 * LOOK;

/// Food of its diet: round the patch it last found some in, if that is within `RECALL` blocks and
/// still has some, else the nearest within `LOOK` blocks; remembered as its patch.
fn find_food(w: &World, id: u64, s: &Species, pos: [f64; 3], driven: bool) -> Option<[i32; 3]> {
    let p = pos.map(|v| v.floor() as i32);
    let patch = w
        .component(id, &PATCH)
        .filter(|k| k[3] != 0 && (k[0] - p[0]).pow(2) + (k[2] - p[2]).pow(2) <= RECALL * RECALL);
    let found = patch
        .and_then(|k| nearest(w, s, [k[0], k[1], k[2]], PATCH_LOOK, p, false))
        .or_else(|| nearest(w, s, p, LOOK, p, true))?;
    // Ground that frightened its kind (a kill there, `memory`) it keeps off unless hunger drives it.
    let at = [found[0] as f64 + 0.5, found[1] as f64, found[2] as f64 + 0.5];
    if !driven && crate::memory::fear_at(w, at) >= animals_core::memory::AVOID {
        weft_sdk::debug!("animal {id} keeps off feared ground to eat at {found:?}");
        return None;
    }
    if let Some(c) = w.component_mut(id, &PATCH) {
        *c = [found[0], found[1], found[2], 1];
    }
    Some(found)
}

/// The nearest food of its diet to `p` within `r` blocks of `center` it can reach from the
/// ground; the ground's own roots, insects and lichen only when nothing richer is in sight
/// (`Source::fallback`), and only with `ground`.
fn nearest(w: &World, s: &Species, center: [i32; 3], r: i32, p: [i32; 3], ground_too: bool) -> Option<[i32; 3]> {
    let now = season(w);
    let mut best: Option<([i32; 3], i32)> = None;
    let mut ground: Option<([i32; 3], i32)> = None;
    for dx in -r..=r {
        for dz in -r..=r {
            for dy in -2..=LOOK_UP {
                let at = [center[0] + dx, center[1] + dy, center[2] + dz];
                let (ox, oy, oz) = (at[0] - p[0], at[1] - p[1], at[2] - p[2]);
                let d = ox * ox + oz * oz + oy * oy * 4;
                if best.is_some_and(|(_, b)| d >= b) {
                    continue;
                }
                match food(s, w.block(at), now) {
                    Some(src) if !src.fallback() => best = Some((at, d)),
                    Some(_) if ground_too && ground.is_none_or(|(_, b)| d < b) => ground = Some((at, d)),
                    _ => {}
                }
            }
        }
    }
    best.or(ground).map(|(at, _)| at)
}

/// The share of a spoiled carcass's weight an insect eater gets from it (its insects).
const CARRION_INSECTS: f32 = 0.1;

/// Grams of the dropped item `id` that `s` eats: food of its diet (spoiled food as carrion).
fn edible(w: &World, s: &animals_core::species::Species, id: u64) -> Option<f32> {
    let stack = w.dropped_item(id)?;
    let name = name_of(abi::entry_kind::ITEM, stack.item);
    let food = vitals::food::FOODS.iter().find(|f| f.name == name);
    // A poisonous one (a fly agaric, a spider's eye) it leaves where it lies, and salted meat,
    // too salty for an animal; dried and smoked meat a meat eater takes as it raids a camp (C6).
    // A scavenger's stomach takes rotten flesh (a vulture's acid kills what's in it).
    if food.is_some_and(|f| f.poison > 0.0 && !(s.scavenges() && f.category == vitals::food::Category::Carrion))
        || name == "base:salted_meat"
    {
        return None;
    }
    let spoiled = food.is_some_and(|f| {
        let (age, _) = base_api::food_age(&stack, w.day_time());
        vitals::food::freshness(f, age, base_api::difficulty().keep) == vitals::food::Freshness::Spoiled
    });
    let (c, grams) = vitals::forage::dropped(name, spoiled)?;
    // To a scavenger a kill's meat lying there is carrion.
    let c = if s.scavenges() && c == vitals::food::Category::Meat { vitals::food::Category::Carrion } else { c };
    // Kibble is every species' (`taming.rs`), though the food table files it as meat (C6); a
    // carcass gone off is the insects' as well, its beetles and grubs an insect eater's (the
    // Djadokhta's bones are bored by them), a share of it.
    let insects = c == vitals::food::Category::Carrion && s.eats(vitals::food::Category::Insects);
    if insects && !s.eats(c) {
        return Some(grams * CARRION_INSECTS);
    }
    (s.eats(c) || name == "animals:kibble").then_some(grams)
}

/// The nearest dropped food of its diet within `radius` blocks of `pos`: the item, where it is.
pub fn ground_food(w: &World, s: &animals_core::species::Species, pos: [f64; 3], radius: f64) -> Option<(u64, [f64; 3])> {
    w.entities_near(pos, radius, None)
        .into_iter()
        .filter(|&t| edible(w, s, t).is_some())
        .filter_map(|t| Some((t, w.entity(t)?.pos.to_array())))
        .min_by(|a, b| dist2(a.1, pos).total_cmp(&dist2(b.1, pos)))
}

/// Within reach of its mouth: two blocks for even a small one standing beside it.
fn reach(at: [f64; 3], pos: [f64; 3]) -> bool {
    let (dx, dz) = (at[0] - pos[0], at[2] - pos[2]);
    dx * dx + dz * dz <= 4.0 && (at[1] - pos[1]).abs() <= 2.5
}

/// It eats one of the dropped item `item` (`grams` each): as much of its food as the item's
/// weight is of a meal of its mass (`needs::fills`).
fn eat_item(w: &World, id: u64, a: &animal::Animal, item: u64, grams: f32) {
    let Some(mut stack) = w.dropped_item(item) else { return };
    let own = crate::hunt::mass(w, id).unwrap_or_else(|| a.species().map_or(1.0, |s| s.bio.mass));
    let max = a.stat(species::FOOD);
    let points = (max * animals_core::needs::fills(own, grams / 1000.0))
        .round()
        .max(1.0) as u16;
    animal::feed(w, id, points);
    stack.count -= 1;
    w.set_dropped_item(item, stack);
    weft_sdk::debug!(
        "animal {id} eats {} off the ground ({grams} g, it {own:.3} kg): food {} of {max:.0}",
        name_of(abi::entry_kind::ITEM, stack.item),
        a.food.saturating_add(points).min(max as u16)
    );
    if a.species().is_some_and(|s| s.scavenges()) {
        weft_sdk::info!("scavenger {id} eats {} ({grams} g)", name_of(abi::entry_kind::ITEM, stack.item));
    }
}

/// A goal: hungry, it goes to the nearest food of its diet (on the ground first, then growing) and
/// eats there; it takes the turn while it does. With nothing near it lets the next goal
/// (wandering) take it elsewhere. Thirst is weighed against it by the needs band (`choose.rs`,
/// `drink::drink`).
pub fn eat(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(a) = w.component(id, &ANIMAL) else { return false };
    let Some(s) = a.species() else { return false };
    let Some(mut f) = w.component(id, &FORAGE) else { return false };
    let full = a.food as f32 / a.stat(species::FOOD).max(1.0);
    // Only an animal that gets hungry, and whose food the world grows or drops.
    let grows = s.eats.iter().any(|&(c, _)| vitals::forage::grows(c));
    let drops = s.eats.iter().any(|&(c, _)| vitals::forage::dropped_kind(c));
    // The cheap reasons first: most animals most of the time aren't hungry.
    // A herd member grazes with its leader (`groups::JOIN_EAT`).
    let hungry =
        if crate::group::leader_doing(w, id) == Some(crate::choose::Need::Eat as u8) { animals_core::groups::JOIN_EAT } else { HUNGRY };
    let fed = full >= FULL || (f[3] == 0.0 && full >= hungry);
    if fed || !((s.hungry_in_the_wild() || animal::tamed(w, id)) && (grows || drops)) {
        if f[3] != 0.0 {
            w.component_mut(id, &FORAGE).map(|f| *f = [0.0; 4]);
            w.component_mut(id, &ITEM).map(|i| *i = 0);
        }
        return false;
    }
    // Walking a way out of where it is stuck (`stuck`).
    if crate::stuck::going(w, id) {
        return true;
    }
    let now = w.game_time() as f64;
    // Going to food on the ground, or eating it.
    let item = w.component(id, &ITEM).unwrap_or(0);
    if item != 0 {
        if let (Some(e), Some(grams)) = (w.entity(item), edible(w, s, item)) {
            if reach(e.pos.to_array(), st.pos.to_array()) {
                if now < f[3] {
                    crate::anim::hold(w, id, crate::anim::Act::Graze);
                    return true; // chewing
                }
                eat_item(w, id, &a, item, grams);
                crate::anim::hold(w, id, crate::anim::Act::Graze);
                f[3] = now + CHEW;
                w.component_mut(id, &FORAGE).map(|c| *c = f);
                return true;
            }
            if now < f[3] {
                return true; // walking to it
            }
        }
        // Gone (eaten, taken) or out of reach still: it looks again.
        w.component_mut(id, &ITEM).map(|i| *i = 0);
        f[3] = 0.0;
    }
    let target = (!f[0].is_nan() && f[3] != 0.0).then(|| [f[0], f[1], f[2]]);
    let close = target.is_some_and(|t| {
        let (dx, dz) = (t[0] + 0.5 - st.pos.x, t[2] + 0.5 - st.pos.z);
        // Within reach of its mouth: two blocks for even a small one standing beside it.
        dx * dx + dz * dz <= 4.0 && (t[1] - st.pos.y).abs() <= 2.5
    });
    if close && let Some(t) = target {
        if now < f[3] {
            crate::anim::hold(w, id, crate::anim::Act::Graze);
            return true; // chewing
        }
        let at = t.map(|v| v as i32);
        if let Some(src) = food(s, w.block(at), season(w)) {
            // Growing food feeds as well as it grows now: dry-season grass is poor fodder.
            let fodder = if src.renews.is_some() { 0.4 + 0.6 * lush(w, s, at) } else { 1.0 };
            animal::feed(w, id, (src.food as f32 * fodder).round().max(1.0) as u16);
            weft_sdk::debug!(
                "animal {id} eats {} at {at:?} ({:.0}% as good as at its best; food {} of {:.0})",
                src.block,
                fodder * 100.0,
                a.food,
                a.stat(species::FOOD)
            );
            match src.eaten {
                Eaten::Becomes(name) => {
                    if let Some(b) = becomes(name) {
                        w.set_block(at, b);
                    }
                }
                Eaten::Age(age) => {
                    w.prop_set(at, "age", &age.to_string());
                }
                Eaten::Stays => {}
            }
            if w.random_below(vitals::soil::MOUTHFULS_A_PAT) == 0 {
                drop_pat(w, st.pos);
            }
            crate::anim::hold(w, id, crate::anim::Act::Graze);
            f[3] = now + CHEW;
            w.component_mut(id, &FORAGE).map(|c| *c = f);
            return true;
        }
    } else if now < f[3] {
        // Walking to it; or nothing near, wandering on.
        return target.is_some();
    } else if target.is_some() && !w.navigating(id) {
        // Its walk ended short of the food: it can't get there. It gives that patch up and lets
        // its wander take it elsewhere, not walking at it again and again (graze5: a deer stood
        // "eating" a game day at one spot while its food went from 55 % to 4 % and its thirst
        // past saving).
        weft_sdk::debug!("animal {id} can't reach the food at {target:?}: looks elsewhere");
        w.component_mut(id, &PATCH).map(|k| *k = [0; 4]);
        w.component_mut(id, &FORAGE)
            .map(|c| *c = [f64::NAN, 0.0, 0.0, now + NOTHING]);
        // Ended short where the last did too: it is stuck, and looks for a way out (`stuck`).
        return crate::stuck::short(w, id, st);
    }
    if let Some((t, p)) = ground_food(w, s, st.pos.to_array(), ITEM_LOOK) {
        w.navigate(id, p, 1.0);
        w.component_mut(id, &ITEM).map(|i| *i = t);
        w.component_mut(id, &FORAGE)
            .map(|c| *c = [p[0], p[1], p[2], now + WALK]);
        return true;
    }
    match grows
        .then(|| find_food(w, id, s, st.pos.to_array(), animals_core::needs::hunger_score(full, true) >= animals_core::memory::DRIVEN))
        .flatten()
    {
        Some(p) => {
            w.navigate(id, [p[0] as f64 + 0.5, p[1] as f64, p[2] as f64 + 0.5], 1.0);
            f = [p[0] as f64, p[1] as f64, p[2] as f64, now + WALK];
        }
        None => f = [f64::NAN, 0.0, 0.0, now + NOTHING],
    }
    w.component_mut(id, &FORAGE).map(|c| *c = f);
    !f[0].is_nan()
}

/// The block a food becomes when eaten (`Eaten::Becomes`).
fn becomes(name: &str) -> Option<u32> {
    match name {
        "base:grazed_grass" => Some(GRAZED_GRASS.id()),
        _ => None,
    }
}

/// A grazer's dung (`vitals::soil`): a pat where it stands on the ground (on the soil, in the air
/// or the grass there), none within `PAT_SPACING` of another; off the soil, the dung as an item.
fn drop_pat(w: &World, pos: Vec3) {
    let [x, y, z] = pos.block().to_array();
    let feet = w.block([x, y, z]);
    let open = feet == 0 || TALL_GRASS.is(feet) || GRAZED_GRASS.is(feet);
    if !(open && PLANT_SOIL.on_block(w.block([x, y - 1, z]))) {
        w.spawn_item(pos, ItemStack::new(DUNG.id(), 1));
        return;
    }
    let r = vitals::soil::PAT_SPACING;
    let crowded = (-1..=1).any(|dy| (-r..=r).any(|dz| (-r..=r).any(|dx| DUNG_PAT.is(w.block([x + dx, y + dy, z + dz])))));
    if !crowded {
        w.set_block([x, y, z], DUNG_PAT.id());
    }
}
