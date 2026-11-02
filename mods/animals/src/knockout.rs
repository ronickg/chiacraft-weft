//! Knock-out taming (`notes/ANIMALS.md`, "Knock out and feed"), for the raptor and big animals
//! later: hits raise an animal's torpor (a club far more than fists, a tranquilizer dart most);
//! past its torpor stat it falls unconscious. Asleep, its torpor drains: sedatives keep it under,
//! and if it wakes the tame fails and it attacks. Food put in its mouth (right click with a
//! stack) is eaten a bite a minute, each bite filling the taming bar: kibble most, its favorite
//! more than plain food, which also costs effectiveness, as does every hit while it's out.

use weft_sdk::abi::ItemStack;
use weft_sdk::*;

use crate::animal::{self, ANIMAL};
use crate::species::{self, Liking, SPECIES, Taming as Method};
use crate::taming::{self, TAMING};

value! {
    #[derive(Default)]
    /// An animal's torpor and, knocked out, what it has to eat (saved).
    pub struct Knockout {
        pub torpor: f32,
        /// Ticks knocked out + 1 (0: awake).
        pub out: u32,
        /// Food in its mouth: the item and how many.
        pub food: u32,
        pub count: u32,
        /// Ticks to the next bite.
        pub bite: u32,
        /// Ticks left angry, and at whom.
        pub angry: u32,
        pub angry_at: u64,
    }
}

pub static KNOCKOUT: Component<Knockout> = Component::named("knockout");
pub static CLUB: Item = Item::named("club");
pub static SEDATIVE: Item = Item::named("sedative");
pub static DART: Item = Item::named("tranquilizer_dart");
static DART_SHOT: Entity = Entity::named("dart");
static SHOOTER: Component<u64> = Component::named("dart_shooter");
static SLEEP: Particle = Particle::named("sleep");
static TOOLS: CreativeTab = CreativeTab::named("base:tools");
static COMBAT: CreativeTab = CreativeTab::named("base:combat");
static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");

/// Torpor a hit adds: with a club, a fist, a dart.
const CLUB_TORPOR: f32 = 20.0;
const FIST_TORPOR: f32 = 5.0;
const DART_TORPOR: f32 = 60.0;
/// A sedative raises it by this share of the maximum.
const SEDATE: f32 = 0.4;
/// Awake, torpor is gone in two minutes; out, in four.
const AWAKE_DRAIN: f32 = 1.0 / 2400.0;
const OUT_DRAIN: f32 = 1.0 / 4800.0;
/// A bite a minute.
const BITE_TICKS: u32 = 1200;
/// Thirty seconds angry.
const ANGRY_TICKS: u32 = 600;
/// Effectiveness a hit costs while it's out, and a bite of plain food or a favorite.
const HIT_WHILE_OUT: f32 = 0.05;
const PLAIN_BITE: f32 = 0.02;
const FAVORITE_BITE: f32 = 0.01;
const DART_SPEED: f64 = 30.0;

pub fn init(r: &mut Registry) {
    r.entity_component(&KNOCKOUT, Knockout::default(), component_flags::SAVE | component_flags::SYNC_MENU);
    for s in SPECIES.iter().filter(|s| s.taming == Method::Knockout) {
        r.attach(s.kind, &KNOCKOUT);
    }
    r.item(&CLUB).max_stack(1).attack_damage(1.0).tab(&COMBAT);
    r.shaped(&CRAFTING, "club", 1, &["l", "s"], &[('l', "#logs"), ('s', "base:stick")]);
    r.item(&SEDATIVE).tab(&TOOLS);
    r.shapeless(&CRAFTING, "sedative", 2, &["base:spider_eye", "base:sugar", "base:wheat_seeds"], 0);
    r.item(&DART).tab(&COMBAT).on_use(|u| {
        let (w, player) = (u.world(), u.player());
        throw_dart(&w, player);
        if !w.creative(player) {
            u.set_stack(StackData::shrink(u.stack(), 1));
        }
        true
    });
    r.shapeless(&CRAFTING, "tranquilizer_dart", 4, &["base:arrow", "base:spider_eye"], 0);
    r.entity_component(&SHOOTER, 0u64, 0);
    r.entity(&DART_SHOT)
        .no_egg()
        .size(0.3, 0.3)
        .unpushable()
        .health(1.0)
        .speed(0.0)
        .model(Model::from(vec![Part::new("body", [0.0, 1.0, 0.0]).cube([-0.5, -0.5, -4.0], [1.0, 1.0, 8.0], [0.0, 0.0])]).uv(32, 32))
        .texture("dart")
        .attach(&SHOOTER)
        .goals(ai::Goals::new().then(dart_flies));
    r.particle(&SLEEP)
        .textures(&["sleep"])
        .life(1.5, 2.0)
        .size(0.18, 0.24)
        .gravity(-0.25)
        .drag(0.92)
        .fullbright();
    r.on(&ENTITY_HURT, |b| {
        let w = b.world();
        for h in b.iter() {
            hurt(&w, h);
        }
    });
}

fn set(w: &World, id: u64, f: impl FnOnce(&mut Knockout)) {
    if let Some(k) = w.component_mut(id, &KNOCKOUT) {
        f(k);
    }
}

/// Its torpor stat: how much it takes to knock it out.
fn max_torpor(w: &World, id: u64) -> f32 {
    w.component(id, &ANIMAL)
        .map_or(100.0, |a| a.stat(species::TORPOR))
}

/// `amount` more torpor; past the maximum, a wild one falls unconscious.
pub fn add_torpor(w: &World, id: u64, amount: f32) {
    if animal::tamed(w, id) || w.component(id, &KNOCKOUT).is_none() {
        return;
    }
    let max = max_torpor(w, id);
    let mut fell = false;
    set(w, id, |k| {
        k.torpor = (k.torpor + amount).min(max * 1.5);
        if k.out == 0 && k.torpor >= max {
            k.out = 1;
            k.bite = BITE_TICKS;
            k.angry = 0;
            fell = true;
        }
    });
    if fell {
        weft_sdk::info!("animal {id} knocked out");
        w.navigate(id, w.entity(id).map_or([0.0; 3], |e| e.pos), 0.0);
    }
}

/// Every hurt: a hit by a player raises torpor (a club or bare hands); one on an unconscious
/// animal costs effectiveness; an awake wild one turns on whoever hurt it.
fn hurt(w: &World, h: &abi::Hurt) {
    let Some(k) = w.component(h.entity, &KNOCKOUT).copied() else { return };
    if animal::tamed(w, h.entity) || h.source == 0 {
        return;
    }
    if w.entity(h.source).is_some_and(|s| s.kind == PLAYER.id()) {
        let held = w.inventory(h.source).get(w.selected_slot(h.source));
        let torpor = if held.item == CLUB.id() {
            CLUB_TORPOR
        } else if held.is_empty() {
            FIST_TORPOR
        } else {
            0.0
        };
        add_torpor(w, h.entity, torpor);
    }
    // A hit while it's out (not the one that knocked it out) costs effectiveness.
    if k.out > 10 {
        if let Some(t) = w.component_mut(h.entity, &TAMING) {
            t.effectiveness = (t.effectiveness - HIT_WHILE_OUT).max(0.0);
        }
    } else if w.component(h.entity, &KNOCKOUT).is_some_and(|k| k.out == 0) {
        anger(w, h.entity, h.source);
    }
}

/// `id` turns on `at` for a while (and its pack with it, see `raptor::aid`).
pub fn anger(w: &World, id: u64, at: u64) {
    set(w, id, |k| {
        k.angry = ANGRY_TICKS;
        k.angry_at = at;
    });
    crate::raptor::aid(w, id, at);
}

/// A goal for animals tamed by knocking out, first: unconscious it lies still (eating, losing
/// torpor, waking when it's gone); angry it attacks; awake its torpor wears off.
pub fn behave(index: usize) -> impl Fn(&World, u64, &EntityState) -> bool + Send + Sync + 'static {
    move |w, id, st| {
        let Some(k) = w.component(id, &KNOCKOUT).copied() else { return false };
        let max = max_torpor(w, id);
        if k.out == 0 {
            if k.torpor > 0.0 {
                set(w, id, |k| k.torpor = (k.torpor - max * AWAKE_DRAIN).max(0.0));
            }
            if k.angry > 0 {
                set(w, id, |k| k.angry -= 1);
                let alive = w.entity(k.angry_at).is_some_and(|e| e.health > 0.0);
                if alive {
                    crate::wake(w, id, index);
                    crate::commands::fight(w, id, st, k.angry_at);
                    return true;
                }
                set(w, id, |k| k.angry = 0);
            }
            return false;
        }
        // Unconscious.
        crate::sleep(w, id, index);
        if w.navigating(id) {
            w.navigate(id, st.pos, 0.0);
        }
        if (w.time() + id) % 30 == 0 {
            drop(
                w.spawn_particles(&SLEEP, [st.pos[0], st.pos[1] + st.size[1] as f64 + 0.2, st.pos[2]])
                    .count(1)
                    .spread(0.1),
            );
        }
        let torpor = k.torpor - max * OUT_DRAIN;
        if torpor <= 0.0 {
            wake_up(w, id, index, st);
            return true;
        }
        set(w, id, |k| {
            k.torpor = torpor;
            k.bite = k.bite.saturating_sub(1);
            k.out = k.out.saturating_add(1);
        });
        if k.bite <= 1 && k.count > 0 {
            bite(w, id, st);
        }
        true
    }
}

/// It wakes before it was tamed: the taming is lost, and it turns on the nearest player.
fn wake_up(w: &World, id: u64, index: usize, st: &EntityState) {
    let near = w
        .players()
        .into_iter()
        .filter_map(|p| Some((p, w.entity(p)?)))
        .filter(|(_, e)| dist2(e.pos, st.pos) < 16.0 * 16.0)
        .min_by(|a, b| dist2(a.1.pos, st.pos).total_cmp(&dist2(b.1.pos, st.pos)))
        .map(|(p, _)| p);
    spill(w, id, st);
    set(w, id, |k| {
        k.out = 0;
        k.torpor = 0.0;
    });
    if let Some(t) = w.component_mut(id, &TAMING) {
        t.progress = 0.0;
        t.effectiveness = 1.0;
        t.feedings = 0;
    }
    crate::wake(w, id, index);
    weft_sdk::info!("animal {id} woke up");
    if let Some(p) = near {
        anger(w, id, p);
    }
}

/// The food left in its mouth falls out.
fn spill(w: &World, id: u64, st: &EntityState) {
    let Some(k) = w.component(id, &KNOCKOUT).copied() else { return };
    if k.count > 0 {
        w.spawn_item([st.pos[0], st.pos[1] + 0.3, st.pos[2]], ItemStack::new(k.food, k.count));
    }
    set(w, id, |k| {
        k.food = 0;
        k.count = 0;
    });
}

/// A bite of the food in its mouth: toward taming it, less with poorer food.
fn bite(w: &World, id: u64, st: &EntityState) {
    let Some(a) = w.component(id, &ANIMAL).copied() else { return };
    let Some(k) = w.component(id, &KNOCKOUT).copied() else { return };
    let (Some(s), Some(t)) = (a.species(), w.component(id, &TAMING).copied()) else { return };
    let Some((_, food)) = s.food(k.food) else { return };
    let needed = s.feedings * (1.0 + a.level() as f32 / 100.0);
    let cost = match food.liking {
        Liking::Eats => PLAIN_BITE,
        Liking::Favorite => FAVORITE_BITE,
        Liking::Kibble => 0.0,
    };
    let progress = (t.progress + food.liking.taming() / needed).min(1.0);
    let effectiveness = (t.effectiveness - cost).max(0.0);
    if let Some(t) = w.component_mut(id, &TAMING) {
        t.progress = progress;
        t.effectiveness = effectiveness;
        t.feedings += 1;
    }
    set(w, id, |k| {
        k.count -= 1;
        k.bite = BITE_TICKS;
    });
    animal::feed(w, id, food.food);
    if progress >= 1.0 {
        // Tamed by whoever is nearest (the tamer, as a rule).
        let tamer = w
            .players()
            .into_iter()
            .filter_map(|p| Some((p, w.entity(p)?)))
            .min_by(|a, b| dist2(a.1.pos, st.pos).total_cmp(&dist2(b.1.pos, st.pos)))
            .map(|(p, _)| p);
        spill(w, id, st);
        set(w, id, |k| {
            k.out = 0;
            k.torpor = 0.0;
        });
        if let Some(p) = tamer {
            taming::tame(w, id, p, effectiveness);
        }
    }
}

/// A player right clicks an unconscious animal: a sedative keeps it under; food it eats goes in
/// its mouth (the held stack, if its mouth is empty or holds the same). True if it was used.
pub fn feed(w: &World, id: u64, player: u64) -> bool {
    let Some(k) = w.component(id, &KNOCKOUT).copied().filter(|k| k.out != 0) else { return false };
    let inv = w.inventory(player);
    let slot = w.selected_slot(player);
    let held = inv.get(slot);
    if held.item == SEDATIVE.id() {
        let max = max_torpor(w, id);
        set(w, id, |k| k.torpor = (k.torpor + max * SEDATE).min(max * 1.5));
        taming::take_one(w, player);
        return true;
    }
    let Some(s) = w.component(id, &ANIMAL).and_then(|a| a.species()) else { return false };
    if held.is_empty() || s.food(held.item).is_none() || (k.count > 0 && k.food != held.item) {
        return false;
    }
    set(w, id, |k| {
        k.food = held.item;
        k.count += held.count;
    });
    if !w.creative(player) {
        inv.set(slot, ItemStack::EMPTY);
    }
    true
}

/// An unconscious animal's next bite comes now (testing).
pub fn bite_now(w: &World, id: u64) {
    set(w, id, |k| k.bite = 1);
}

// ------------------------------------------------------------------ darts

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}

/// A dart from `player`'s eyes where they look.
fn throw_dart(w: &World, player: u64) {
    let Some(p) = w.entity(player) else { return };
    let (sy, cy) = (p.yaw as f64).sin_cos();
    let (sp, cp) = (p.pitch as f64).sin_cos();
    let dir = [-sy * cp, sp, -cy * cp];
    let eye = [p.pos[0] + dir[0] * 0.6, p.pos[1] + p.size[2] as f64 - 0.1 + dir[1] * 0.6, p.pos[2] + dir[2] * 0.6];
    let dart = w.spawn(&DART_SHOT, eye);
    if let Some(s) = w.component_mut(dart, &SHOOTER) {
        *s = player;
    }
    if let Some(mut s) = w.entity(dart) {
        s.vel = dir.map(|d| d * DART_SPEED);
        s.yaw = p.yaw;
        w.set_entity(dart, &s);
    }
}

/// A dart in flight: hitting something it stings (a little damage) and sedates; gone when it
/// hits or lands.
fn dart_flies(w: &World, id: u64, st: &EntityState) -> bool {
    let shooter = w.component(id, &SHOOTER).copied().unwrap_or(0);
    let hit = w
        .near_bodies(st.pos, 0.8)
        .into_iter()
        .find(|&e| e != id && e != shooter && w.entity(e).is_some_and(|s| s.kind != st.kind));
    if let Some(target) = hit {
        w.damage(target, 1.0, shooter, damage::ATTACK);
        add_torpor(w, target, DART_TORPOR);
        w.remove(id);
    } else if st.flags & entity_flags::BLOCKED != 0 || st.on_ground != 0 || st.age > 100 {
        w.remove(id);
    }
    true
}
