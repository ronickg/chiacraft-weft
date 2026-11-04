//! What every animal has (`notes/ANIMALS.md`, "Stats and levels"): its species, sex, life stage,
//! a level made of wild points and tamed points per stat, bond, imprint and food. The stats are
//! applied to the entity as attribute modifiers (health, speed) and a step of the attack damage
//! chain; the rest are kept and shown until the features that use them are built.

use weft_sdk::*;

use crate::species::{self, SPECIES, STATS};

use animals_core::life;
/// Life stages; only the chihuahua has young ones so far.
pub use animals_core::species::{ADULT, BABY};
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
        /// How thirsty, 0 slaked to 1 parched (`drink.rs`; `animals_core::needs` for how fast).
        pub thirst: f32,
        pub xp: u32,
        /// When it was born (`life::birth`: a wild one met grown has an age); 0: not known yet
        /// (saved before animals aged), given one at `upkeep`.
        pub born: u64,
        /// Its age in in-game days, as `upkeep` last saw it (for its screen).
        pub age: u16,
        /// How strong it is at its age, in percent of its prime (`life::vigor`; 0: not known
        /// yet, as at its prime).
        pub vigor: u8,
        /// When it was last seen (its upkeep ran; `life::birth`'s count, from long before the
        /// world began): the time away is caught up from it (`catch_up`); 0: not yet.
        pub seen: u64,
    }
}

impl Animal {
    pub fn species(&self) -> Option<&'static species::Species> {
        SPECIES.get((self.species as usize).checked_sub(1)?)
    }
    /// 1, plus a level for every point.
    pub fn level(&self) -> u32 {
        1 + self
            .wild
            .iter()
            .chain(&self.tamed)
            .map(|&p| p as u32)
            .sum::<u32>()
    }
    pub fn tamed_points(&self) -> u32 {
        self.tamed.iter().map(|&p| p as u32).sum()
    }
    /// Stat `i`'s value: `base * (1 + wild * wild_step) * (1 + tamed * tamed_step) * (1 + bonus)`,
    /// the bonus from imprinting (up to 20%) and bond (up to 5%).
    pub fn stat(&self, i: usize) -> f32 {
        let Some(s) = self.species() else { return 0.0 };
        let bonus = self.imprint as f32 / 100.0 * 0.2 + self.bond as f32 / 100.0 * 0.05;
        // The body's stats go with its age: growing up, its prime, old (`life::vigor`).
        let age = if BODILY.contains(&i) && self.vigor > 0 { self.vigor as f32 / 100.0 } else { 1.0 };
        s.base[i] * (1.0 + self.wild[i] as f32 * s.wild_step[i]) * (1.0 + self.tamed[i] as f32 * s.tamed_step[i]) * (1.0 + bonus) * age
    }
    /// The stat as a multiple of the species' base.
    pub fn factor(&self, i: usize) -> f32 {
        self.species().map_or(1.0, |s| self.stat(i) / s.base[i])
    }
}

/// The stats that go with its age: health, stamina, damage, speed and what it carries.
const BODILY: [usize; 5] = [species::HEALTH, species::STAMINA, species::DAMAGE, species::SPEED, species::WEIGHT];

pub static ANIMAL: Component<Animal> = Component::named("animal");
/// The stats last applied to the entity (not saved: applied again after loading).
static APPLIED: Component<[f32; 2]> = Component::named("animal_applied");
/// The size it is drawn at (`World::set_scale`; 0: not yet since it loaded).
static SCALED: Component<f32> = Component::named("animal_scaled");
/// The owner's player key (64 hex digits), zeros while wild.
pub static OWNER: Component<[u8; 64]> = Component::named("owner");

pub fn init(r: &mut Registry) {
    r.entity_component(&ANIMAL, Animal::default(), component_flags::SAVE | component_flags::SYNC_MENU);
    r.entity_component(&APPLIED, [0.0; 2], 0);
    r.entity_component(&SCALED, 0.0f32, 0);
    r.entity_component(&OWNER, [0u8; 64], component_flags::SAVE | component_flags::SYNC_MENU);
    // A young one bites as hard as its size: by the area of its jaws, the square of how far it has
    // grown (a hatchling raptor a fortieth of a grown one's mass killed a cat in a PC run).
    r.modify(&chain::ATTACK_DAMAGE, stage::MULTIPLY, |m| {
        let (w, id) = (m.world(), m.query().entity);
        let Some(index) = w
            .component(id, &ANIMAL)
            .and_then(|a| (a.species as usize).checked_sub(1))
            .filter(|&i| i < SPECIES.len())
        else {
            return;
        };
        let g = crate::breeding::growth(&w, id, index) as f64;
        m.value *= (g * g).clamp(0.05, 1.0);
    });
    for s in &SPECIES {
        r.attach(s.kind, &ANIMAL);
        r.attach(s.kind, &APPLIED);
        r.attach(s.kind, &SCALED);
        r.attach(s.kind, &OWNER);
    }
    // An animal hits as hard as its damage stat.
    r.modify(&chain::ATTACK_DAMAGE, stage::MULTIPLY, |m| {
        if let Some(a) = m.world().component(m.query().entity, &ANIMAL) {
            m.value *= (a.factor(species::DAMAGE) * crate::comfort::condition(a)) as f64;
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
    key.len() == 64
        && w.component(id, &OWNER)
            .is_some_and(|k| k[..] == *key.as_bytes())
}

/// A goal that comes first for every animal of species `index`: a new one gets its wild level,
/// sex and full food; its stats are applied when they change; a tame gets hungry. Never takes
/// the turn.
pub fn upkeep(index: usize) -> impl Fn(&World, u64, &EntityState) -> bool + Send + Sync + 'static {
    move |w, id, st| {
        let Some(a) = w.component_mut(id, &ANIMAL) else { return false };
        let new = a.species == 0;
        if new {
            first_roll(w, id, index, a, st.pos);
        }
        let away = if a.seen == 0 { 0 } else { life::age(w.time(), a.seen) };
        a.seen = life::birth(w.time(), 0);
        if !new && away > life::AWAY {
            catch_up(w, id, index, a, away, st.pos);
        }
        crate::coats::upkeep(w, id, index, a, st.pos);
        crate::breeding::upkeep(w, id, index, a);
        if old_age(w, id, index, a) {
            return false;
        }
        let a = *a;
        // Its size, from its genes and sex, and how far it has grown (the engine forgets it on
        // loading: set it again).
        if let Some(g) = crate::coats::size_of(w, id) {
            let size = g * crate::breeding::growth(w, id, index);
            if w.component(id, &SCALED).is_some_and(|&s| s != size) {
                w.set_scale(id, size);
                if let Some(s) = w.component_mut(id, &SCALED) {
                    *s = size;
                }
            }
        }
        // Its body once a second (`comfort.rs`); a tame gets hungry and thirsty, faster out of
        // its comfort, and so does a wild one whose food the world has: a plant eater grazes
        // (`forage.rs`), a hunter with prey at home hunts (`hunt.rs`); both drink (`drink.rs`).
        if !new && (w.time() + id) % 20 == 0 {
            let rate = crate::comfort::upkeep(w, id, &a, st);
            if tamed(w, id) || a.species().is_some_and(|s| s.hungry_in_the_wild()) {
                hunger(w, id, &a, rate);
                crate::drink::thirst(w, id, &a, st, rate);
            }
        }
        let (health, speed) = factors(w, id, &a);
        if w.component(id, &APPLIED)
            .is_some_and(|&p| p != [health, speed])
        {
            apply(w, id, st, health, speed, if new { f32::MAX } else { 0.0 });
        }
        false
    }
}

/// A new one's wild level, sex, age and food (`roll`, `arrive`).
fn first_roll(w: &World, id: u64, index: usize, a: &mut Animal, pos: [f64; 3]) {
    *a = roll(w, index);
    arrive(w, id, index, a, pos);
}

/// Rolls `id` now if its first `upkeep` hasn't yet: what reaches an animal before its own goals
/// do (a tame, a command, its screen) finds its species, level and stats (Playtest B2: a chihuahua
/// tamed and shown after `/tk spawn` was still unrolled, level 1, every stat 0, and its screen
/// showed "{0}"). Its genes come with it, as `upkeep` gives them. True if it is an animal.
pub fn ensure_rolled(w: &World, id: u64) -> bool {
    let Some(st) = w.entity(id) else { return false };
    let Some(index) = SPECIES.iter().position(|s| s.kind.id() == st.kind) else { return false };
    let Some(a) = w.component_mut(id, &ANIMAL) else { return false };
    if a.species == 0 {
        weft_sdk::warn!("animal {id} ({}) reached before its first upkeep: rolled now", SPECIES[index].name);
        first_roll(w, id, index, a, st.pos);
        a.seen = life::birth(w.time(), 0);
        crate::coats::upkeep(w, id, index, a, st.pos);
    }
    true
}

/// A wild one of species `index`: its level (most of them low), a point per level on a random
/// stat, a sex, an adult, fed.
fn roll(w: &World, index: usize) -> Animal {
    let level = match w.random_below(100) {
        0..60 => 1 + w.random_below(30),
        60..90 => 31 + w.random_below(60),
        _ => 91 + w.random_below(MAX_WILD_LEVEL - 90),
    };
    let mut a = Animal { species: index as u8 + 1, female: w.random_below(2) as u8, stage: ADULT, ..Animal::default() };
    a.born = born_wild(w, index);
    for _ in 1..level {
        a.wild[w.random_below(STATS as u32) as usize] += 1;
    }
    a.food = a.stat(species::FOOD).round() as u16;
    a
}

/// A wild one just met (`notes/ECOLOGY.md`, parts 6 and 7): a plant eater is as fed as its place
/// feeds it now (thin in a drought), a hunter as its prey are (`crate::hunt::prey_lush`: thin in
/// the winter its prey are thin in); in its breeding season a share of them are this year's young.
fn arrive(w: &World, id: u64, index: usize, a: &mut Animal, pos: [f64; 3]) {
    let s = &SPECIES[index];
    let at = pos.map(|v| v.floor() as i32);
    let lush = if s.forages_wild() {
        Some(crate::forage::season_lush(w, s, at))
    } else if s.hungry_in_the_wild() {
        crate::hunt::prey_lush(w, s, at)
    } else {
        None
    };
    if let Some(lush) = lush {
        a.food = (a.stat(species::FOOD) * (ARRIVES_FED + (1.0 - ARRIVES_FED) * lush)).round() as u16;
        weft_sdk::debug!(
            "wild {} {id} arrives {:.0}% fed (its {} at {:.0}% here this season)",
            s.name,
            a.food as f32 / a.stat(species::FOOD) * 100.0,
            if s.forages_wild() { "food grows" } else { "prey are fed" },
            lush * 100.0
        );
    }
    let in_season = crate::breeding::season_bit(w).is_none_or(|bit| s.bio.breeds & bit != 0);
    if in_season && w.random_below(100) < YOUNG_PERCENT {
        // Born this season, up to most of the way grown.
        let age = s.growth() as u64 * w.random_below(80) as u64 / 100;
        let growing = (s.growth() as u64 - age) as u32;
        a.born = life::birth(w.time(), age);
        a.stage = life::stage(s, growing, BABY);
        if let Some(b) = w.component_mut(id, &crate::breeding::BREEDING) {
            b.growing = growing;
        }
    }
}

/// The share of wild ones met in their breeding season that are young.
const YOUNG_PERCENT: u32 = 25;
use animals_core::needs::{ARRIVES_FED, STARVES_SECS};

/// Its area was away (unloaded) for `away` ticks, and life went on there (`notes/ANIMALS.md`,
/// "Time away"): its timers run out (a mother due gives birth now), it has aged (by the world's
/// clock: a tame one whose time ran out dies as it loads, `old_age`), and a wild one that died
/// meanwhile has a successor in its place, born while away, of its place's kind as it has evolved.
fn catch_up(w: &World, id: u64, index: usize, a: &mut Animal, away: u64, pos: [f64; 3]) {
    crate::breeding::catch_up(w, id, away);
    let s = &SPECIES[index];
    let Some(seed) = w.component(id, &crate::coats::GENOME).map(|g| g.seed) else { return };
    if tamed(w, id) || life::age(w.time(), a.born) < life::lifespan(s, seed) {
        return;
    }
    let Some(pool) = crate::evolution::pool_at(w, index, pos) else { return };
    let mut r = crate::coats::world_rng(w);
    let female = r() < 0.5;
    let Some(mut g) = crate::coats::sample(index, female, &pool, &mut r) else { return };
    let age = life::successor_age(s, away, r());
    let growing = (s.growth() as u64).saturating_sub(age) as u32;
    *a = roll(w, index);
    a.female = g.female;
    a.born = life::birth(w.time(), age);
    a.seen = life::birth(w.time(), 0);
    a.stage = life::stage(s, growing, BABY);
    g.stage = a.stage;
    if let Some(old) = w.component_mut(id, &crate::coats::GENOME) {
        *old = g;
    }
    if let Some(b) = w.component_mut(id, &crate::breeding::BREEDING) {
        *b = crate::breeding::Breeding { growing, ..Default::default() };
    }
    crate::breeding::count(&crate::breeding::OLD, index);
    crate::breeding::count(&crate::breeding::BORN, index);
    weft_sdk::debug!(
        "wild {} {id} died while away ({} days); a successor {} days old took its place",
        s.name,
        away / life::DAY,
        age / life::DAY
    );
}

/// When a wild one met now was born: grown, and up to most of an average life ago.
fn born_wild(w: &World, index: usize) -> u64 {
    let age = life::wild_age(&SPECIES[index], w.random_below(10_000) as f32 / 10_000.0);
    life::birth(w.time(), age)
}

/// Its age (`notes/ANIMALS.md`, "Old age"): one from before animals aged gets one; a grown one
/// past most of its life is an elder (greying, past breeding); at the end of its own lifespan
/// (its species', give or take a fifth) it dies. True if it died.
fn old_age(w: &World, id: u64, index: usize, a: &mut Animal) -> bool {
    let s = &SPECIES[index];
    if a.born == 0 {
        let growing = w
            .component(id, &crate::breeding::BREEDING)
            .map_or(0, |b| b.growing) as u64;
        a.born = if growing > 0 { life::birth(w.time(), s.growth() as u64 - growing) } else { born_wild(w, index) };
    }
    // Sped up for testing (`/animal speed`): older by the extra ticks too.
    let n = crate::breeding::speed() as u64;
    if n > 1 {
        a.born = a.born.saturating_sub(n - 1).max(1);
    }
    let Some(seed) = w.component(id, &crate::coats::GENOME).map(|g| g.seed) else { return false };
    let (age, span) = (life::age(w.time(), a.born), life::lifespan(s, seed));
    a.stage = life::aged(a.stage, age, span);
    a.age = (age / life::DAY).min(u16::MAX as u64) as u16;
    let growing = w
        .component(id, &crate::breeding::BREEDING)
        .map_or(0, |b| b.growing);
    a.vigor = (life::vigor(s, growing, age) * 100.0)
        .round()
        .clamp(1.0, 100.0) as u8;
    if age < span {
        return false;
    }
    crate::breeding::count(&crate::breeding::OLD, index);
    weft_sdk::info!("animal {id} ({}) died of old age, {} days old", s.name, age / life::DAY);
    // Bypassing armor and leaving no scar: nothing hit it.
    w.damage(id, 1e9, 0, damage::VOID);
    true
}

/// How old it is, in in-game days, and how long its kind lives (`old_age`).
pub fn age_days(a: &Animal) -> Option<(f64, f64, u8)> {
    let s = a.species().filter(|_| a.born != 0)?;
    Some((a.age as f64, s.lifespan() as f64 / life::DAY as f64, if a.vigor == 0 { 100 } else { a.vigor }))
}

/// Max health and speed as its stats, its condition and (cold-blooded) the air say.
fn factors(w: &World, id: u64, a: &Animal) -> (f32, f32) {
    (a.factor(species::HEALTH) * crate::comfort::condition(a), a.factor(species::SPEED) * crate::comfort::speed(w, id))
}

/// Max health and speed as the stats say; `heal` adds that much health on top (a new animal
/// starts full).
fn apply(w: &World, id: u64, st: &EntityState, health: f32, speed: f32, heal: f32) {
    w.add_modifier(id, &attribute::MAX_HEALTH, "animals:stats", attr_op::MUL_BASE, health as f64 - 1.0, 0);
    w.add_modifier(id, &attribute::MOVEMENT_SPEED, "animals:stats", attr_op::MUL_BASE, speed as f64 - 1.0, 0);
    if let Some(p) = w.component_mut(id, &APPLIED) {
        *p = [health, speed];
    }
    if heal > 0.0
        && let Some(a) = w.component(id, &ANIMAL)
    {
        let mut s = *st;
        s.max_health = a.stat(species::HEALTH) * crate::comfort::condition(&a);
        s.health = (st.health + heal).min(s.max_health);
        w.set_entity(id, &s);
    }
}

/// Once a second: a tame or a wild plant eater uses a point of food every so often (its species'
/// `hunger`, `rate` times as often out of its comfort); it eats from the world (`forage.rs`). A
/// wild one with an empty stomach starves.
fn hunger(w: &World, id: u64, a: &Animal, rate: f32) {
    let Some(s) = a.species() else { return };
    // A wild one with nothing left starves to death in a game day (a tame suffers, never dies:
    // its condition falls instead).
    if a.food == 0 && !tamed(w, id) {
        if let Some(st) = w.entity(id) {
            w.damage(id, st.max_health / STARVES_SECS, 0, damage::STARVE);
        }
        return;
    }
    let every = ((s.hunger / rate).max(1.0) as u64).max(1);
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
        let (health, speed) = factors(w, id, &a);
        let gained = a.stat(species::HEALTH) * crate::comfort::condition(&a) - st.max_health;
        apply(w, id, &st, health, speed, gained);
    }
}
