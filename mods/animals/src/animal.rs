//! What every animal has (`notes/ANIMALS.md`, "Stats and levels"): its species, sex, life stage,
//! a level made of wild points and tamed points per stat, bond, imprint and food. The stats are
//! applied to the entity as attribute modifiers (health, speed) and a step of the attack damage
//! chain; the rest are kept and shown until the features that use them are built.

use weft_sdk::*;

use crate::species::{self, SPECIES, STATS};

/// Life stages; only the chihuahua has young ones so far.
pub const BABY: u8 = 0;
pub const JUVENILE: u8 = 1;
pub const ADOLESCENT: u8 = 2;
pub const ADULT: u8 = 3;
pub const STAGE_NAMES: [&str; 5] = ["baby", "juvenile", "adolescent", "adult", "elder"];

/// Wild levels: most low, a few high (proposed in the design; a world setting later).
pub const MAX_WILD_LEVEL: u32 = 150;
/// Tamed points an animal can get by leveling (proposed; a world setting later).
pub const MAX_TAMED_POINTS: u32 = 75;

value! {
    #[derive(Default)]
    /// An animal's own state, saved with it, and sent to whoever has its screen open.
    pub struct Animal {
        /// Species + 1 (0: not rolled yet).
        pub species: u8,
        pub female: u8,
        pub stage: u8,
        /// 0 to 100.
        pub bond: u8,
        /// 0 to 100 (babies, later).
        pub imprint: u8,
        /// Follow, stay, wander (later).
        pub command: u8,
        /// Tamed points earned and not placed yet.
        pub unspent: u16,
        /// Points per stat: from the wild, and placed by its owner.
        pub wild: [u8; STATS],
        pub tamed: [u8; STATS],
        /// Food now (the food stat is how much it holds).
        pub food: u16,
        pub xp: u32,
    }
}

impl Animal {
    pub fn species(&self) -> Option<&'static species::Species> {
        SPECIES.get((self.species as usize).checked_sub(1)?)
    }
    /// 1, plus a level for every point.
    pub fn level(&self) -> u32 {
        1 + self.wild.iter().chain(&self.tamed).map(|&p| p as u32).sum::<u32>()
    }
    pub fn tamed_points(&self) -> u32 {
        self.tamed.iter().map(|&p| p as u32).sum()
    }
    /// Stat `i`'s value: `base * (1 + wild * wild_step) * (1 + tamed * tamed_step) * (1 + bonus)`,
    /// the bonus from imprinting (up to 20%) and bond (up to 5%).
    pub fn stat(&self, i: usize) -> f32 {
        let Some(s) = self.species() else { return 0.0 };
        let bonus = self.imprint as f32 / 100.0 * 0.2 + self.bond as f32 / 100.0 * 0.05;
        s.base[i] * (1.0 + self.wild[i] as f32 * s.wild_step[i]) * (1.0 + self.tamed[i] as f32 * s.tamed_step[i]) * (1.0 + bonus)
    }
    /// The stat as a multiple of the species' base.
    pub fn factor(&self, i: usize) -> f32 {
        self.species()
            .map_or(1.0, |s| self.stat(i) / s.base[i])
    }
}

pub static ANIMAL: Component<Animal> = Component::named("animal");
/// The stats last applied to the entity (not saved: applied again after loading).
static APPLIED: Component<[f32; 2]> = Component::named("animal_applied");
/// The owner's player key (64 hex digits), zeros while wild.
pub static OWNER: Component<[u8; 64]> = Component::named("owner");

pub fn init(r: &mut Registry) {
    r.entity_component(&ANIMAL, Animal::default(), component_flags::SAVE | component_flags::SYNC_MENU);
    r.entity_component(&APPLIED, [0.0; 2], 0);
    r.entity_component(&OWNER, [0u8; 64], component_flags::SAVE | component_flags::SYNC_MENU);
    for s in &SPECIES {
        r.attach(s.kind, &ANIMAL);
        r.attach(s.kind, &APPLIED);
        r.attach(s.kind, &OWNER);
    }
    // An animal hits as hard as its damage stat.
    r.modify(&chain::ATTACK_DAMAGE, stage::MULTIPLY, |m| {
        if let Some(a) = m.world().component(m.query().entity, &ANIMAL) {
            m.value *= a.factor(species::DAMAGE) as f64;
        }
    });
}

/// Whether `id` is tamed.
pub fn tamed(w: &World, id: u64) -> bool {
    w.component(id, &OWNER).is_some_and(|k| k[0] != 0)
}

/// Whether `player` (by key) owns `id`.
pub fn owns(w: &World, id: u64, player: u64) -> bool {
    let key = crate::player_key(player);
    key.len() == 64 && w.component(id, &OWNER).is_some_and(|k| k[..] == *key.as_bytes())
}

/// A goal that comes first for every animal of species `index`: a new one gets its wild level,
/// sex and full food; its stats are applied when they change; a tame gets hungry. Never takes
/// the turn.
pub fn upkeep(index: usize) -> impl Fn(&World, u64, &EntityState) -> bool + Send + Sync + 'static {
    move |w, id, st| {
        let Some(a) = w.component_mut(id, &ANIMAL) else { return false };
        let new = a.species == 0;
        if new {
            *a = roll(w, index);
        }
        let a = *a;
        if !new && tamed(w, id) && (w.time() + id) % 20 == 0 {
            hunger(w, id, &a);
        }
        let (health, speed) = (a.factor(species::HEALTH), a.factor(species::SPEED));
        if w.component(id, &APPLIED).is_some_and(|&p| p != [health, speed]) {
            apply(w, id, st, health, speed, if new { f32::MAX } else { 0.0 });
        }
        false
    }
}

/// A wild one of species `index`: its level (most of them low), a point per level on a random
/// stat, a sex, an adult, fed.
fn roll(w: &World, index: usize) -> Animal {
    let level = match w.random_below(100) {
        0..60 => 1 + w.random_below(30),
        60..90 => 31 + w.random_below(60),
        _ => 91 + w.random_below(MAX_WILD_LEVEL - 90),
    };
    let mut a = Animal {
        species: index as u8 + 1,
        female: w.random_below(2) as u8,
        stage: ADULT,
        ..Animal::default()
    };
    for _ in 1..level {
        a.wild[w.random_below(STATS as u32) as usize] += 1;
    }
    a.food = a.stat(species::FOOD).round() as u16;
    a
}

/// Max health and speed as the stats say; `heal` adds that much health on top (a new animal
/// starts full).
fn apply(w: &World, id: u64, st: &EntityState, health: f32, speed: f32, heal: f32) {
    w.add_modifier(id, &attribute::MAX_HEALTH, "animals:stats", attr_op::MUL_BASE, health as f64 - 1.0, 0);
    w.add_modifier(id, &attribute::MOVEMENT_SPEED, "animals:stats", attr_op::MUL_BASE, speed as f64 - 1.0, 0);
    if let Some(p) = w.component_mut(id, &APPLIED) {
        *p = [health, speed];
    }
    if heal > 0.0 && let Some(a) = w.component(id, &ANIMAL) {
        let mut s = *st;
        s.max_health = a.stat(species::HEALTH);
        s.health = (st.health + heal).min(s.max_health);
        w.set_entity(id, &s);
    }
}

/// Once a second: a tame uses a point of food every so often (its species' `hunger`). Starving
/// and eating from troughs come with commands and needs.
fn hunger(w: &World, id: u64, a: &Animal) {
    let Some(s) = a.species() else { return };
    let every = (s.hunger.max(1.0) as u64).max(1);
    if a.food > 0 && (w.time() / 20) % every == 0 {
        if let Some(a) = w.component_mut(id, &ANIMAL) {
            a.food -= 1;
        }
    }
}

/// Feeds `id` `amount` points of food, up to what it holds.
pub fn feed(w: &World, id: u64, amount: u16) {
    if let Some(a) = w.component_mut(id, &ANIMAL) {
        let max = a.stat(species::FOOD).round() as u16;
        a.food = a.food.saturating_add(amount).min(max);
    }
}

/// The owner places one of its unspent points on stat `i` (never torpor).
pub fn place_point(w: &World, id: u64, player: u64, i: usize) {
    if i >= STATS || i == species::TORPOR || !owns(w, id, player) {
        return;
    }
    let Some(a) = w.component_mut(id, &ANIMAL) else { return };
    if a.unspent == 0 || a.tamed_points() >= MAX_TAMED_POINTS || a.tamed[i] == u8::MAX {
        return;
    }
    a.unspent -= 1;
    a.tamed[i] += 1;
    // More health heals that much.
    let a = *a;
    if let Some(st) = w.entity(id) {
        let gained = a.stat(species::HEALTH) - st.max_health;
        apply(w, id, &st, a.factor(species::HEALTH), a.factor(species::SPEED), gained);
    }
}
