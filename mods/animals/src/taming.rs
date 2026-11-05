//! Passive taming (`notes/ANIMALS.md`, "Taming"): come close slowly, wait until the animal asks
//! for food (a bubble with what it wants above it), and feed it. Each feeding fills its taming bar
//! (more with its favorite food, most with kibble) and starts a wait before it asks again.
//! Feeding it when it isn't asking, food it dislikes, or hurting it lowers the taming's
//! effectiveness and can scare it off; a clean tame gives bonus levels. Knock-out taming (the
//! raptor) comes later.

use weft_sdk::abi::ItemStack;
use weft_sdk::*;

use crate::animal::{self, ANIMAL, OWNER};
use crate::species::{self, Liking, SPECIES, STATS, Taming as Method};

value! {
    /// A wild animal's taming so far, saved with it.
    pub struct Taming {
        /// The bar, 0 to 1: tamed at 1.
        pub progress: f32,
        /// 1 at first; down with every mistake.
        pub effectiveness: f32,
        pub feedings: u32,
        /// Ticks before it asks again, asking, and running off scared.
        pub wait: u32,
        pub asking: u32,
        pub scared: u32,
        /// What it asks for: its food + 1.
        pub want: u32,
    }
}

pub static TAMING: Component<Taming> = Component::named("taming");
/// Each food's bubble (by item name), made once.
static BUBBLES: std::sync::OnceLock<Vec<(&'static str, &'static Particle)>> = std::sync::OnceLock::new();
static HEART: Particle = Particle::named("heart");
pub static KIBBLE: Item = Item::named("kibble");
static FOOD_TAB: CreativeTab = CreativeTab::named("base:food");
static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");
static POOF: Particle = Particle::named("core:poof");

/// Twenty seconds asking, ten to thirty between.
const ASK_TICKS: u32 = 400;
const WAIT_TICKS: (u32, u32) = (200, 600);
/// Four seconds running off.
const SCARED_TICKS: u32 = 80;
/// How close a player must be for it to ask.
const ASK_RANGE: f64 = 5.0;
/// Effectiveness lost to a mistake, and to being hurt.
const MISTAKE: f32 = 0.1;
const HURT: f32 = 0.15;

pub const FRESH: Taming = Taming { progress: 0.0, effectiveness: 1.0, feedings: 0, wait: 0, asking: 0, scared: 0, want: 0 };

pub fn init(r: &mut Registry) {
    r.entity_component(&TAMING, FRESH, component_flags::SAVE | component_flags::SYNC_MENU | component_flags::SYNC_WATCHERS);
    for s in &SPECIES {
        r.attach(s.kind, &TAMING);
    }
    // Kibble (`notes/ANIMALS.md`, "Kibble"): an egg, meat or a plant, wheat for fiber, and water;
    // every species takes it, and it tames fastest. Tiers by egg and cooking in a pot come with
    // more eggs.
    r.item(&KIBBLE).tab(&FOOD_TAB);
    r.tag("kibble_meat", &["base:beef", "base:porkchop", "base:chicken", "base:mutton"]);
    r.tag("kibble_plant", &["base:apple", "base:sugar_cane"]);
    for filling in ["#kibble_meat", "#kibble_plant"] {
        r.shapeless(&CRAFTING, "kibble", 4, &["base:egg", filling, "base:wheat", "base:water_bucket"], 0);
    }
    // A bubble with each food in it, shown above an animal asking for it.
    let mut bubbles: Vec<(&'static str, &'static Particle)> = Vec::new();
    for f in SPECIES.iter().flat_map(|s| s.foods) {
        if bubbles.iter().any(|(item, _)| *item == f.item) {
            continue;
        }
        let name = bubble(f.item);
        let p = Particle::generated(name.clone());
        r.particle(p)
            .textures(&[&name])
            .life(1.1, 1.1)
            .size(0.45, 0.45)
            .gravity(0.0)
            .drag(1.0)
            .fullbright();
        bubbles.push((f.item, p));
    }
    let _ = BUBBLES.set(bubbles);
}

/// The bubble particle of a food item (`base:cooked_beef`: `want_cooked_beef`).
fn bubble(item: &str) -> String {
    format!("want_{}", item.split_once(':').map_or(item, |(_, n)| n))
}

/// A goal for wild animals of species `index`, after `animal::upkeep`: running off when
/// scared, standing still while asking for food, asking when a player is near and calm. One that
/// is never tamed (`Taming::Wild`: a hare) only runs off.
pub fn behave(index: usize) -> impl Fn(&World, u64, &EntityState) -> bool + Send + Sync + 'static {
    move |w, id, st| {
        let s = &SPECIES[index];
        let wild = s.taming == Method::Wild;
        if !(s.taming == Method::Passive || wild) || animal::tamed(w, id) {
            return false;
        }
        let Some(t) = w.component(id, &TAMING) else { return false };
        if t.scared > 0 {
            set(w, id, |t| t.scared = t.scared.saturating_sub(w.tick_elapsed(id)));
            if (w.game_time() + id) % 20 == 0 || !w.navigating(id) {
                flee(w, id, st);
            }
            return true;
        }
        if (w.game_time() + id) % 10 == 0
            && s.shy > 0.0
            && frightened(w, st, s.shy * shyness(w, id))
            && !crate::signatures::calm(w, st.pos.to_array())
        {
            scare(w, id);
            return true;
        }
        if wild {
            return false;
        }
        if t.asking > 0 {
            set(w, id, |t| {
                t.asking = t.asking.saturating_sub(w.tick_elapsed(id) as _);
                if t.asking == 0 {
                    t.wait = WAIT_TICKS.0 + w.random_below(WAIT_TICKS.1 - WAIT_TICKS.0);
                }
            });
            let food = s.foods.get(t.want as usize - 1).map(|f| f.item);
            let bubble = BUBBLES
                .get()
                .and_then(|b| b.iter().find(|(item, _)| Some(*item) == food));
            if (w.game_time() + id) % 20 == 0
                && let Some((_, p)) = bubble
            {
                drop(
                    w.spawn_particles(p, [st.pos.x, st.pos.y + st.size.height as f64 + 0.45, st.pos.z])
                        .count(1)
                        .spread(0.0),
                );
            }
            if w.navigating(id) {
                w.navigate(id, st.pos, 0.0);
            }
            return true;
        }
        if t.wait > 0 {
            set(w, id, |t| t.wait = t.wait.saturating_sub(w.tick_elapsed(id)));
            return false;
        }
        if (w.game_time() + id) % 20 == 0 && near_player(w, st).is_some() {
            // Its favorite half the time, else anything it eats (kibble it never asks for:
            // that it always takes).
            let pick: Vec<usize> = (0..s.foods.len())
                .filter(|&i| s.foods[i].liking != Liking::Kibble)
                .filter(|&i| w.random_below(2) == 0 || s.foods[i].liking == Liking::Favorite)
                .collect();
            let pick = if pick.is_empty() { vec![0] } else { pick };
            let want = pick[w.random_below(pick.len() as u32) as usize];
            set(w, id, |t| {
                t.asking = ASK_TICKS;
                t.want = want as u32 + 1;
            });
        }
        false
    }
}

/// `id` was let go: taming it starts again.
pub fn released(w: &World, id: u64) {
    set(w, id, |t| *t = FRESH);
}

fn set(w: &World, id: u64, f: impl FnOnce(&mut Taming)) {
    if let Some(t) = w.component_mut(id, &TAMING) {
        f(t);
    }
}

/// The nearest player within asking range.
fn near_player(w: &World, st: &EntityState) -> Option<u64> {
    w.players()
        .into_iter()
        .filter_map(|p| Some((p, dist2(&w.entity(p)?, st))))
        .filter(|&(_, d)| d < ASK_RANGE * ASK_RANGE)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(p, _)| p)
}

fn dist2(a: &EntityState, b: &EntityState) -> f64 {
    crate::dist2(a.pos, b.pos)
}

/// How far its temperament (`coats::temperament`) moves its kind's shy distance: a calm one (0)
/// lets a walker come to 0.6 of it, a skittish one (1) runs at 1.4 times it.
fn shyness(w: &World, id: u64) -> f32 {
    w.component(id, &crate::coats::GENOME)
        .map_or(1.0, |g| 0.6 + 0.8 * crate::coats::temperament(&g))
}

/// A player walking upright within `shy` blocks, or running within twice that.
fn frightened(w: &World, st: &EntityState, shy: f32) -> bool {
    let shy = shy as f64;
    w.players()
        .into_iter()
        .filter_map(|p| w.entity(p))
        .any(|p| {
            let d2 = dist2(&p, st);
            let sneaking = p.pose == abi::pose::CROUCH;
            let walking = p.moved > 0.01;
            (p.sprinting != 0 && d2 < 4.0 * shy * shy) || (!sneaking && walking && d2 < shy * shy)
        })
}

/// Scared: stops asking and runs off for a while.
fn scare(w: &World, id: u64) {
    set(w, id, |t| {
        t.scared = SCARED_TICKS;
        t.asking = 0;
        t.wait = t.wait.max(WAIT_TICKS.0);
    });
    if let Some(st) = w.entity(id) {
        flee(w, id, &st);
    }
}

/// Away from the nearest player, else anywhere.
fn flee(w: &World, id: u64, st: &EntityState) {
    let away = w
        .players()
        .into_iter()
        .filter_map(|p| w.entity(p))
        .min_by(|a, b| dist2(a, st).total_cmp(&dist2(b, st)))
        .map(|p| (st.pos.x - p.pos.x, st.pos.z - p.pos.z))
        .filter(|&(x, z)| x * x + z * z > 0.01);
    let (x, z) = away.unwrap_or_else(|| {
        let a = w.random_below(628) as f64 / 100.0;
        (a.cos(), a.sin())
    });
    let n = (x * x + z * z).sqrt().max(0.01);
    w.navigate(id, [st.pos.x + x / n * 8.0, st.pos.y, st.pos.z + z / n * 8.0], 1.6);
}

/// It was hurt: a taming under way loses effectiveness, and it runs off.
pub fn hurt(w: &World, id: u64) {
    let passive = w
        .component(id, &ANIMAL)
        .and_then(|a| a.species())
        .is_some_and(|s| matches!(s.taming, Method::Passive | Method::Wild));
    if !passive || animal::tamed(w, id) {
        return;
    }
    set(w, id, |t| {
        if t.progress > 0.0 {
            t.effectiveness = (t.effectiveness - HURT).max(0.0);
        }
    });
    scare(w, id);
}

/// A player right clicks a wild animal with `held`: food it eats goes toward taming it. True if
/// the click was used.
pub fn feed(w: &World, id: u64, player: u64) -> bool {
    let Some(a) = w.component(id, &ANIMAL) else { return false };
    let (Some(s), Some(t)) = (a.species(), w.component(id, &TAMING)) else { return false };
    if s.taming != Method::Passive || animal::tamed(w, id) || t.scared > 0 {
        return false;
    }
    let Some(st) = w.entity(id) else { return false };
    let held = w.inventory(player).get(w.selected_slot(player));
    let Some((index, food)) = s.food(held.item) else {
        // Something another animal eats: it turns away, a little put off.
        if !held.is_empty() && species::anyone_eats(held.item) {
            mistake(w, id, &st, true);
            return true;
        }
        return false;
    };
    take_one(w, player);
    animal::feed(w, id, food.food);
    if t.asking == 0 {
        // Fed when it wasn't asking: it eats, warily.
        mistake(w, id, &st, w.random_below(3) == 0);
        return true;
    }
    // What it asked for counts a quarter more (kibble is best anyway).
    let wanted = t.want as usize == index + 1;
    let needed = s.feedings * (1.0 + a.level() as f32 / 100.0);
    // Near a capybara it takes to you a quarter faster.
    let calm = if crate::signatures::calm(w, st.pos.to_array()) { 1.25 } else { 1.0 };
    let gain = food.liking.taming() * if wanted { 1.25 } else { 1.0 } * calm / needed;
    let wait = WAIT_TICKS.0 + w.random_below(WAIT_TICKS.1 - WAIT_TICKS.0);
    set(w, id, |t| {
        t.progress = (t.progress + gain).min(1.0);
        t.feedings += 1;
        t.asking = 0;
        t.wait = wait;
    });
    hearts(w, st.pos.to_array(), 2);
    if t.progress + gain >= 1.0 {
        tame(w, id, player, t.effectiveness);
    }
    true
}

/// A mistake: effectiveness down, and maybe it runs off.
fn mistake(w: &World, id: u64, st: &EntityState, scares: bool) {
    set(w, id, |t| t.effectiveness = (t.effectiveness - MISTAKE).max(0.0));
    drop(
        w.spawn_particles(&POOF, [st.pos.x, st.pos.y + 0.5, st.pos.z])
            .count(4)
            .spread(0.2),
    );
    if scares {
        scare(w, id);
    }
}

/// `id` becomes `player`'s, with `floor(level * effectiveness / 2)` bonus levels placed on
/// random stats (never torpor).
pub fn tame(w: &World, id: u64, player: u64, effectiveness: f32) {
    // Its level first: the bonus levels and the XP go by it.
    crate::animal::ensure_rolled(w, id);
    // Out of the wild: its genes leave its place's pool.
    if !crate::animal::tamed(w, id) {
        crate::evolution::count(w, id, -1.0);
    }
    let key = crate::player_key(player);
    if let Some(o) = w.component_mut(id, &OWNER)
        && key.len() == 64
    {
        o.copy_from_slice(key.as_bytes());
    }
    if let Some(a) = w.component_mut(id, &ANIMAL) {
        let bonus = (a.level() as f32 * effectiveness.clamp(0.0, 1.0) / 2.0) as u32;
        for _ in 0..bonus {
            let mut i = w.random_below(STATS as u32 - 1) as usize;
            if i >= species::TORPOR {
                i += 1;
            }
            a.wild[i] = a.wild[i].saturating_add(1);
        }
        a.bond = a.bond.max(25);
        weft_sdk::info!("animal {id} tamed at {:.0}%: {bonus} bonus levels, now level {}", effectiveness * 100.0, a.level());
        let level = a.level();
        crate::xp::tamed(w, player, level);
        crate::journal::note_animal(w, player, id, animals_core::journal::TAMED);
        // The first time this player tames one: how to give it orders (a line under the
        // crosshair, `base_api::hint`), then Survival's "A Friend" advancement
        // (`base_api::milestones`), which also marks that they have.
        let first = w
            .component(player, &base_api::MILESTONES)
            .is_some_and(|m| m & base_api::milestones::TAME == 0);
        if first {
            base_api::hint(w, player, "hint.animals.tamed", &[]);
        }
        base_api::milestone(w, player, base_api::milestones::TAME);
    }
    set(w, id, |t| *t = FRESH);
    crate::commands::tamed(w, id);
    crate::dog::on_tamed(w, id);
    if let Some(st) = w.entity(id) {
        hearts(w, st.pos.to_array(), 7);
    }
}

fn hearts(w: &World, at: [f64; 3], count: u32) {
    drop(
        w.spawn_particles(&HEART, [at[0], at[1] + 0.6, at[2]])
            .count(count)
            .spread(0.3)
            .velocity([0.0, 0.6, 0.0]),
    );
}

/// One of the held item used up (unless in creative).
pub fn take_one(w: &World, player: u64) {
    if w.outside_survival(player) {
        return;
    }
    let inv = w.inventory(player);
    let slot = w.selected_slot(player);
    let mut s = inv.get(slot);
    s.count = s.count.saturating_sub(1);
    inv.set(slot, if s.count == 0 { ItemStack::EMPTY } else { s });
}
