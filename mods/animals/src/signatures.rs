//! What each species does best (`notes/ANIMALS.md`, "The four animals"):
//!
//! - **Chihuahua, alarm and tracker**: a tame barks at the nearest monster that comes within
//!   range (farther with level) and marks it for five seconds; given something an animal drops
//!   (leather, a feather, wool...), or told to with the animal screen's Track and the drop in
//!   hand, it leads its owner to the nearest such animal, waiting when they fall behind.
//! - **Cat, lucky finds**: every few minutes (sooner with bond) a tame brings its owner a gift:
//!   string or a feather mostly, now and then something better, rarely something precious (more
//!   often with level and bond). The gifts are loot tables other mods can add to.
//! - **Capybara, calm aura**: around a capybara (farther with level and bond) shy animals don't
//!   run from you, taming goes a quarter faster, scared tames settle, and tames staying there
//!   slowly grow their bond.

use weft_sdk::abi::ItemStack;
use weft_sdk::*;

use crate::animal::{self, ANIMAL};
use crate::commands::{self, ORDERS};
use crate::species::SPECIES;

value! {
    #[derive(Default)]
    /// What a signature is doing (not saved: it starts over).
    pub struct Signature {
        /// Ticks to the next gift (cat); a gift on its way (cat, 1).
        pub timer: u32,
        pub carrying: u32,
        /// The monster it marked and ticks left (chihuahua); the last one it barked at.
        pub marked: u64,
        pub mark_ticks: u32,
        pub barked_at: u64,
        /// The kind tracked + 1, and ticks left tracking (chihuahua).
        pub track: u32,
        pub track_ticks: u32,
    }
}

pub static SIGNATURE: Component<Signature> = Component::named("signature");
static BARK: Sound = Sound::named("chihuahua.bark");
static MARK: Particle = Particle::named("mark");
static CALM: Particle = Particle::named("calm");
static HEART: Particle = Particle::named("heart");
static POOF: Particle = Particle::named("core:poof");
static HOSTILE: Tag = Tag::named("hostile");
/// The cat's gifts, by how rare.
static GIFTS: [LootTable; 4] = [
    LootTable::named("cat_gift_common"),
    LootTable::named("cat_gift_uncommon"),
    LootTable::named("cat_gift_rare"),
    LootTable::named("cat_gift_precious"),
];
/// What an animal drops, and the animal a chihuahua finds by it.
const SCENTS: [(&str, &str); 17] = [
    ("base:leather", "base:cow"),
    ("base:beef", "base:cow"),
    ("base:cooked_beef", "base:cow"),
    ("base:porkchop", "base:pig"),
    ("base:cooked_porkchop", "base:pig"),
    ("base:feather", "base:chicken"),
    ("base:egg", "base:chicken"),
    ("base:chicken", "base:chicken"),
    ("base:wool", "base:sheep"),
    ("base:mutton", "base:sheep"),
    ("base:cooked_mutton", "base:sheep"),
    ("base:rotten_flesh", "base:zombie"),
    ("base:bone", "base:skeleton"),
    ("base:string", "base:spider"),
    ("base:spider_eye", "base:spider"),
    ("base:gunpowder", "base:creeper"),
    ("base:arrow", "base:skeleton"),
];
/// The kinds tracked, made once (by `SCENTS`' animal).
static TRACKED: std::sync::OnceLock<Vec<(&'static str, &'static Entity)>> = std::sync::OnceLock::new();

/// Two minutes of tracking; a mark lasts five seconds.
const TRACK_TICKS: u32 = 2400;
const MARK_TICKS: u32 = 100;
/// How far a chihuahua leads, and how far ahead of its owner it waits.
const TRACK_RANGE: f64 = 96.0;
const LEAD: f64 = 8.0;
/// Five to ten minutes between gifts (halved at full bond).
const GIFT_TICKS: (u32, u32) = (6000, 12000);

pub fn init(r: &mut Registry) {
    r.entity_component(&SIGNATURE, Signature::default(), 0);
    for s in &SPECIES {
        r.attach(s.kind, &SIGNATURE);
    }
    r.sound(&BARK)
        .files(&["chihuahua/bark1", "chihuahua/bark2", "chihuahua/bark3"])
        .category(&sound_categories::CREATURES);
    r.particle(&MARK)
        .textures(&["mark"])
        .life(0.55, 0.55)
        .size(0.35, 0.35)
        .gravity(0.0)
        .drag(1.0)
        .fullbright();
    r.particle(&CALM)
        .textures(&["calm"])
        .life(1.5, 2.5)
        .size(0.12, 0.18)
        .gravity(-0.05)
        .drag(0.95)
        .fullbright()
        .translucent();
    let gift = |item: &str, weight: u32| Entry::item(item).weight(weight);
    r.loot(&GIFTS[0]).pool(
        Pool::new()
            .entry(gift("base:string", 4))
            .entry(gift("base:feather", 4))
            .entry(gift("base:wheat_seeds", 2))
            .entry(gift("base:bone", 2))
            .entry(gift("base:rotten_flesh", 1)),
    );
    r.loot(&GIFTS[1]).pool(
        Pool::new()
            .entry(gift("base:egg", 3))
            .entry(gift("base:chicken", 3))
            .entry(gift("base:leather", 2))
            .entry(gift("base:spider_eye", 1))
            .entry(gift("base:gunpowder", 1)),
    );
    r.loot(&GIFTS[2]).pool(
        Pool::new()
            .entry(gift("base:raw_iron", 3))
            .entry(gift("base:raw_gold", 2))
            .entry(gift("base:iron_ingot", 2))
            .entry(gift("base:gold_ingot", 1))
            .entry(gift("base:apple", 2)),
    );
    r.loot(&GIFTS[3]).pool(Pool::new().entry(gift("base:diamond", 1)));
    let mut tracked: Vec<(&'static str, &'static Entity)> = Vec::new();
    for (_, kind) in SCENTS {
        if !tracked.iter().any(|(k, _)| *k == kind) {
            tracked.push((kind, Entity::generated(kind.to_string())));
        }
    }
    let _ = TRACKED.set(tracked);
}

fn set(w: &World, id: u64, f: impl FnOnce(&mut Signature)) {
    if let Some(s) = w.component_mut(id, &SIGNATURE) {
        f(s);
    }
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}

// ------------------------------------------------------------------ chihuahua

/// How far a chihuahua notices monsters: 12 blocks, up to 27 at level 150.
fn alarm_range(a: &animal::Animal) -> f64 {
    12.0 + (a.level() as f64 / 10.0).min(15.0)
}

/// A goal for chihuahuas (never takes the turn): a tame barks at the nearest monster that came
/// within range and marks it; the mark shows above it for a while.
pub fn alarm(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(sig) = w.component(id, &SIGNATURE).copied() else { return false };
    if sig.mark_ticks > 0 {
        set(w, id, |s| s.mark_ticks -= 1);
        if sig.mark_ticks % 10 == 0
            && let Some(m) = w.entity(sig.marked)
        {
            mark(w, &m);
        }
    }
    if (w.time() + id) % 10 != 0 || !animal::tamed(w, id) {
        return false;
    }
    let Some(a) = w.component(id, &ANIMAL).copied() else { return false };
    let range = alarm_range(&a);
    let nearest = w
        .near(st.pos, range, None)
        .into_iter()
        .filter_map(|t| Some((t, w.entity(t)?)))
        .filter(|(_, e)| HOSTILE.on_entity(e.kind) && e.health > 0.0)
        .min_by(|a, b| dist2(a.1.pos, st.pos).total_cmp(&dist2(b.1.pos, st.pos)));
    // A new one, and not right after the last bark (two seconds).
    if let Some((t, e)) = nearest
        && t != sig.barked_at
        && sig.mark_ticks + 40 < MARK_TICKS
    {
        bark(w, st.pos);
        mark(w, &e);
        set(w, id, |s| {
            s.barked_at = t;
            s.marked = t;
            s.mark_ticks = MARK_TICKS;
        });
    }
    false
}

fn bark(w: &World, at: [f64; 3]) {
    w.play_sound(&BARK, at, 1.0);
}

fn mark(w: &World, e: &EntityState) {
    drop(
        w.spawn_particles(&MARK, [e.pos[0], e.pos[1] + e.size[1] as f64 + 0.5, e.pos[2]])
            .count(1)
            .spread(0.0),
    );
}

/// The animal a chihuahua finds by `item`, if it has a scent.
fn scent(item: u32) -> Option<(u32, &'static Entity)> {
    let name = item_name(item);
    let kind = SCENTS.iter().find(|(i, _)| *i == name)?.1;
    let tracked = TRACKED.get()?;
    let i = tracked.iter().position(|(k, _)| *k == kind)?;
    Some((i as u32, tracked[i].1))
}

/// Its owner gives a tame chihuahua `item` to track: true if it has a scent (the chihuahua
/// sniffs it and sets off).
pub fn track(w: &World, id: u64, item: u32) -> bool {
    let Some((i, _)) = scent(item) else { return false };
    if w.component(id, &ANIMAL).and_then(|a| a.species()).is_none_or(|s| s.name != "chihuahua") {
        return false;
    }
    set(w, id, |s| {
        s.track = i + 1;
        s.track_ticks = TRACK_TICKS;
    });
    if let Some(st) = w.entity(id) {
        bark(w, st.pos);
    }
    true
}

/// A goal for chihuahuas, before commands: tracking, it heads for the nearest animal of the
/// kind, waits when its owner falls behind, and barks and marks it when there.
pub fn tracking(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(sig) = w.component(id, &SIGNATURE).copied().filter(|s| s.track > 0) else { return false };
    if sig.track_ticks == 0 || w.component(id, &ORDERS).is_some_and(|o| o.target != 0) {
        set(w, id, |s| s.track = 0);
        return false;
    }
    set(w, id, |s| s.track_ticks -= 1);
    if (w.time() + id) % 10 != 0 {
        return true;
    }
    let Some(kind) = TRACKED.get().and_then(|t| t.get(sig.track as usize - 1)).map(|t| t.1) else { return false };
    let found = w
        .near(st.pos, TRACK_RANGE, Some(kind))
        .into_iter()
        .filter_map(|t| Some((t, w.entity(t)?)))
        .min_by(|a, b| dist2(a.1.pos, st.pos).total_cmp(&dist2(b.1.pos, st.pos)));
    let Some((t, e)) = found else {
        // Nothing to find: it gives up.
        drop(
            w.spawn_particles(&POOF, [st.pos[0], st.pos[1] + 0.5, st.pos[2]])
                .count(4)
                .spread(0.2),
        );
        set(w, id, |s| s.track = 0);
        return false;
    };
    if dist2(e.pos, st.pos) < 3.0 * 3.0 {
        bark(w, st.pos);
        set(w, id, |s| {
            s.track = 0;
            s.marked = t;
            s.mark_ticks = MARK_TICKS;
        });
        w.navigate(id, st.pos, 0.0);
        return true;
    }
    crate::wake(w, id, 0);
    let owner = commands::owner(w, id).and_then(|p| w.entity(p));
    if let Some(o) = owner
        && dist2(o.pos, st.pos) > LEAD * LEAD
        && dist2(st.pos, e.pos) < dist2(o.pos, e.pos)
    {
        // Ahead of its owner: waits, looking back.
        w.navigate(id, st.pos, 0.0);
        return true;
    }
    w.navigate(id, e.pos, 1.2);
    true
}

// ------------------------------------------------------------------ cat

/// A goal for cats, before commands: a tame now and then brings its owner a gift.
pub fn gifts(w: &World, id: u64, st: &EntityState) -> bool {
    if !animal::tamed(w, id) {
        return false;
    }
    let Some(sig) = w.component(id, &SIGNATURE).copied() else { return false };
    let Some(a) = w.component(id, &ANIMAL).copied() else { return false };
    if sig.carrying == 0 {
        if sig.timer == 0 {
            let wait = GIFT_TICKS.0 + w.random_below(GIFT_TICKS.1 - GIFT_TICKS.0);
            set(w, id, |s| s.timer = (wait as f32 * (1.0 - a.bond as f32 / 200.0)) as u32);
        } else if sig.timer == 1 {
            set(w, id, |s| {
                s.timer = 0;
                s.carrying = 1;
            });
        } else {
            set(w, id, |s| s.timer -= 1);
        }
        return false;
    }
    let Some(o) = commands::owner(w, id).and_then(|p| Some((p, w.entity(p)?))) else { return false };
    let (player, p) = o;
    if dist2(p.pos, st.pos) > 24.0 * 24.0 {
        // Too far to bring it: it waits until they come by.
        return false;
    }
    if dist2(p.pos, st.pos) > 1.8 * 1.8 {
        crate::wake(w, id, 1);
        if (w.time() + id) % 10 == 0 || !w.navigating(id) {
            w.navigate(id, p.pos, 1.1);
        }
        return true;
    }
    for stack in w.roll_loot(&GIFTS[gift_tier(w, &a)], &no_loot_ctx()) {
        w.spawn_item([p.pos[0], p.pos[1] + 0.3, p.pos[2]], stack);
    }
    weft_sdk::info!("cat {id} brought {player} a gift");
    set(w, id, |s| s.carrying = 0);
    drop(
        w.spawn_particles(&HEART, [st.pos[0], st.pos[1] + 0.6, st.pos[2]])
            .count(3)
            .spread(0.3)
            .velocity([0.0, 0.6, 0.0]),
    );
    true
}

/// How rare a gift: precious 0.2% (up to 0.7% at level 150), rare 4% (up to 16.5% with level
/// and bond), uncommon 26%, else common.
fn gift_tier(w: &World, a: &animal::Animal) -> usize {
    let r = w.random_below(1000);
    let (level, bond) = (a.level(), a.bond as u32);
    if r < 2 + level / 30 {
        3
    } else if r < 40 + level / 2 + bond / 2 {
        2
    } else if r < 300 {
        1
    } else {
        0
    }
}

fn no_loot_ctx() -> LootCtx {
    LootCtx { tool: ItemStack::EMPTY, killer: 0, by_player: false, on_fire: false, mined: 0 }
}

/// A cat's next gift comes now (testing).
pub fn gift_now(w: &World, id: u64) {
    set(w, id, |s| s.timer = 1);
}

// ------------------------------------------------------------------ capybara

/// How far a capybara's calm reaches: 6 blocks, up to 20 with level and bond.
fn aura(a: &animal::Animal) -> f64 {
    (6.0 + a.level() as f64 / 15.0 + a.bond as f64 / 20.0).min(20.0)
}

/// Whether `pos` is within some capybara's calm.
pub fn calm(w: &World, pos: [f64; 3]) -> bool {
    w.near(pos, 20.0, Some(&crate::capybara::CAPYBARA))
        .into_iter()
        .any(|c| {
            let (Some(e), Some(a)) = (w.entity(c), w.component(c, &ANIMAL)) else { return false };
            dist2(e.pos, pos) < aura(a).powi(2)
        })
}

/// A goal for capybaras (never takes the turn): once a second, the animals around it settle;
/// tames staying there grow their bond now and then. A few motes show the calm.
pub fn calm_aura(w: &World, id: u64, st: &EntityState) -> bool {
    if (w.time() + id) % 20 != 0 {
        return false;
    }
    let Some(a) = w.component(id, &ANIMAL).copied() else { return false };
    let radius = aura(&a);
    for other in w.near(st.pos, radius, None) {
        if other == id || w.component(other, &ANIMAL).is_none() {
            continue;
        }
        if let Some(t) = w.component_mut(other, &crate::taming::TAMING) {
            t.scared = t.scared.min(10);
        }
        if let Some(o) = w.component_mut(other, &ORDERS) {
            o.flee = 0;
        }
        // About every minute, a point of bond for a tame staying here.
        if animal::tamed(w, other)
            && commands::command(w, other) == commands::STAY
            && w.random_below(60) == 0
            && let Some(b) = w.component_mut(other, &ANIMAL)
        {
            b.bond = b.bond.saturating_add(1).min(100);
        }
    }
    if w.random_below(3) == 0 {
        let a = w.random_below(628) as f64 / 100.0;
        let r = 1.0 + w.random_below(100) as f64 / 100.0 * (radius / 3.0);
        drop(
            w.spawn_particles(&CALM, [st.pos[0] + a.cos() * r, st.pos[1] + 0.4, st.pos[2] + a.sin() * r])
                .count(1)
                .spread(0.2),
        );
    }
    false
}
