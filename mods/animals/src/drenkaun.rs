//! Drenkaun, the Wood-Warden (`notes/PROGRESSION.md`, round 6): the old forests' boss, a huge
//! antlered guardian, the first biome boss built. Its numbers, attacks, parts and loot are data in
//! `animals_core::bosses`; this is it in the game:
//!
//! - **Summoning.** Its altar (`drenkaun_altar`), set among old trees (`TREES` logs round it),
//!   takes a forest offering (`forest_offering`) and Drenkaun rises beside it: the player picks
//!   when to fight. One at a time near an altar.
//! - **The fight** (a goal on the creature AI): it chases the nearest player and picks an attack
//!   by where they stand, each with a tell first: a **charge** with its head down (a straight run
//!   that tramples; into a trunk or a wall it is stunned), an antler **sweep** in front, a
//!   **stomp** for whoever is under or behind it, and **roots** that burst up under a player far
//!   off and hold them. Under half its health it turns fiercer (its cooldowns shorter, the roots
//!   in threes).
//! - **Stagger** (the damage step, `armour.rs`'s cap and floor): a titan loses at most its health
//!   over 120 s a second; what spills over the floor, and a share of every hit, is its stagger.
//!   Enough of it knocks it down for a few seconds; its parts break at their shares (the antlers,
//!   then its crown), each dropping a piece and softening an attack.
//! - **Loot** for each player who fought it, with bad-luck help (`Loot::chance_after`, the misses
//!   kept per player), including the Stone Age key's part, `warden_heartseed`; sometimes its calf
//!   stays behind, to tame with an offering.
//!
//! Its model and clips are `source/drenkaun` (blocky-model, concept A's colors): `rise`, a
//! `<attack>_tell` and `<attack>` for each attack, `stunned` and `down`, played by name.

use std::collections::HashMap;
use std::sync::Mutex;

use animals_core::bosses::{self, Boss, Form, STAGGER_SHARE};
use vitals::curve::Age;
use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;

pub static DRENKAUN: EntityKind = EntityKind::named("drenkaun");
pub static CALF: EntityKind = EntityKind::named("drenkaun_calf");
pub static ALTAR: Block = Block::named("drenkaun_altar");
pub static OFFERING: Item = Item::named("forest_offering");
pub static HEARTSEED: Item = Item::named("warden_heartseed");
pub static ANTLER: Item = Item::named("warden_antler");
pub static BARK: Item = Item::named("warden_bark");

static PLAYER: EntityKind = EntityKind::named("core:player");
static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");
static BUILDING: CreativeTab = CreativeTab::named("base:building");
static INGREDIENTS: CreativeTab = CreativeTab::named("base:ingredients");
static BITS: Particle = Particle::named("core:block");
/// Its health line at the top of the screen, for players near it (a place in the HUD editor).
static BAR: TextPlace = TextPlace::named("boss_health");
static LOGS: Tag = Tag::named("logs");
static DIRT: Block = Block::named("base:dirt");
static OAK_LOG: Block = Block::named("base:oak_log");
static OAK_LEAVES: Block = Block::named("base:oak_leaves");

/// Its model (`source/drenkaun`, built with the blocky-model skill on concept A's colors: about 4 m
/// at the shoulder, its antlers 4.7 blocks across), drawn at its own size.
const MODEL: &str = "drenkaun";
const TEXTURE: &str = "drenkaun/drenkaun";
pub const SCALE: f32 = 1.0;
/// Its calf: the same model, small (about 1.4 blocks tall to its antler tips).
const CALF_SCALE: f32 = 0.22;
/// Its walking speed, and its charge's times that.
const SPEED: f32 = 2.6;
const CHARGE_SPEED: f32 = 3.2;
/// The longest charge, in ticks.
const CHARGE_TICKS: u32 = 50;
/// Ticks stunned after charging into something solid.
const STUN_TICKS: u32 = 60;
/// How far it looks for players, and how long it waits with none before going back into the wood.
const SIGHT: f64 = 40.0;
const LEAVE_TICKS: u32 = 1200;
/// Ticks it takes to rise from the ground when summoned.
const RISE_TICKS: u32 = 60;
/// Logs within `TREE_RADIUS` an altar needs round it: the old forest it answers in.
pub const TREES: u32 = 8;
const TREE_RADIUS: i32 = 7;
/// No second one within this of an altar.
const ONE_AT_A_TIME: f64 = 96.0;
/// Players this near when it dies get their loot.
const FOUGHT: f64 = 48.0;
/// Seconds roots hold a player (slowed by `ROOT_SLOW`), and how wide they burst.
const ROOT_TICKS: u64 = 60;
const ROOT_SLOW: f64 = -0.75;
const ROOT_RADIUS: f64 = 2.0;
const TICKS: f32 = vitals::TICKS as f32;
/// How near its edge it comes to a player before it stops and turns to them.
const CLOSE: f64 = 1.5;

const RISING: u32 = 0;
const CHASE: u32 = 1;
const WINDUP: u32 = 2;
const ACT: u32 = 3;
const DOWN: u32 = 4;
const STUNNED: u32 = 5;

value! {
    #[derive(Default)]
    /// Drenkaun's fight (saved, so a loaded one goes on as it was).
    pub struct Warden {
        /// Its form's age (tier).
        pub form: u32,
        pub state: u32,
        /// Ticks in its state.
        pub t: u32,
        /// The attack it is making (an index into its boss's `attacks`).
        pub attack: u32,
        pub target: u64,
        /// Ticks before each attack can come again.
        pub cooldown: [u32; 4],
        /// Stagger gathered in all (health points), and toward its next knockdown.
        pub stagger: f32,
        pub knock: f32,
        /// Parts broken, a bit each.
        pub broken: u32,
        /// Where its charge runs, and where its roots will burst (up to three).
        pub dir: [f32; 2],
        pub marks: [f32; 9],
        /// Ticks with no player near.
        pub alone: u32,
        /// Its altar.
        pub altar: [i32; 3],
    }
}

pub static WARDEN: Component<Warden> = Component::named("warden");
/// Drawn at its size since it loaded (not saved: `set_scale` isn't).
static SCALED: Component<u32> = Component::named("warden_scaled");
/// A player's misses for each boss's loot lines (bad-luck help): per boss in `BOSSES`, its loot
/// lines in order, its young last.
static LUCK: Component<[u8; 16]> = Component::named("boss_luck");
/// A calf's owner's player key (zeros while wild).
static CALF_OWNER: Component<[u8; 64]> = Component::named("calf_owner");
/// Each one's last fight state (not saved).
static LAST: Mutex<Option<HashMap<u64, Warden>>> = Mutex::new(None);
/// Who each charge has already hit (not saved).
static TRAMPLED: Mutex<Option<HashMap<u64, Vec<u64>>>> = Mutex::new(None);

fn boss() -> &'static Boss {
    bosses::boss("drenkaun").expect("drenkaun is in animals_core::bosses")
}

fn form_of(w: &Warden) -> &'static Form {
    boss().form(Age::of_tier(w.form as u8))
}

pub fn init(r: &mut Registry) {
    let b = boss();
    let stone = b.stats(b.form(Age::Stone));
    let (bw, bh) = animals_core::bodies::of(MODEL);
    r.entity_component(&WARDEN, Warden::default(), component_flags::SAVE);
    r.entity_component(&SCALED, 0, 0);
    r.entity_component(&LUCK, [0; 16], component_flags::SAVE);
    r.attach(&PLAYER, &LUCK);
    r.entity_component(&CALF_OWNER, [0; 64], component_flags::SAVE);
    r.entity(&DRENKAUN)
        .size(bw, bh)
        .max_health(stone.health)
        .walk_speed(SPEED)
        .attack_damage(0.0)
        .unpushable()
        .always_active()
        .model_file(MODEL)
        .animations(MODEL)
        .texture(TEXTURE)
        .attach(&WARDEN)
        .attach(&SCALED)
        .attach(&crate::armour::FLOOR)
        .goals(Goals::new().then(fight))
        .on_died(|w, d| died(w, d.entity, d.pos.to_array()));
    r.entity(&CALF)
        .size(bw, bh)
        .max_health(stone.health / 40.0)
        .walk_speed(SPEED)
        .passive()
        .model_file(MODEL)
        .animations(MODEL)
        .texture(TEXTURE)
        .attach(&SCALED)
        .attach(&CALF_OWNER)
        .on_interact(|e| calf_interact(&e.world(), e.entity(), e.other()))
        .goals(Goals::new().then(calf).then(ai::wander(0.3, 8.0)));
    // The damage step: the gear cap, its armour, the floor, and stagger from what spills over.
    r.modify(&chain::DAMAGE, stage::MULTIPLY, |m| {
        let q = *m.query();
        let w = m.world();
        if w.component(q.entity, &WARDEN).is_none() {
            return;
        }
        m.value = hurt(&w, q.entity, q.other, q.cause, m.value as f32) as f64;
    });

    r.text_place(&BAR)
        .at(0.5, 0.0, 0.0, 30.0)
        .scale(1.2)
        .times(0.0, 1.5, 0.4)
        .sample("hud.animals.boss_health.sample");
    r.command("drenkaun", "<action:word(state,hurt,kill,luck)> [amount:float]", command);
    r.item(&OFFERING).tab(&INGREDIENTS);
    r.item(&HEARTSEED).tab(&INGREDIENTS);
    r.item(&ANTLER).tab(&INGREDIENTS);
    r.item(&BARK).tab(&INGREDIENTS);
    // The forest's offerings: what an old wood gives.
    r.shapeless(&CRAFTING, "forest_offering", 1, &["base:acorn", "base:mushroom_red", "base:berries", "base:moss_block"], 0);
    r.block(&ALTAR)
        .textures("drenkaun_altar_top", "drenkaun_altar_side", "base:mossy_cobblestone")
        .sound_group("base:block.stone")
        .hardness(3.0)
        .mineable("pickaxe")
        .tab(&BUILDING)
        .on_use(|u| {
            let w = u.world();
            summon(&w, u.player(), u.pos().to_array())
        });
    r.shaped(
        &CRAFTING,
        "drenkaun_altar",
        1,
        &["mfm", "lml", "mmm"],
        &[('m', "base:mossy_cobblestone"), ('f', "base:flint"), ('l', "#logs")],
    );
}

// ------------------------------------------------------------------ summoning

/// A player used the altar at `at`: with an offering in hand, among old trees, and no Drenkaun
/// near, it rises.
fn summon(w: &World, player: u64, at: [i32; 3]) -> bool {
    let held = w.inventory(player).get(w.selected_slot(player));
    let name = name_of(abi::entry_kind::ITEM, held.item);
    let Some(form) = boss().form_for(name) else {
        w.hint(player, "hint.animals.drenkaun.offering", &[]);
        weft_sdk::info!("drenkaun altar {at:?}: no offering in hand ({name})");
        return false;
    };
    let trees = logs_round(w, at);
    if trees < TREES {
        w.hint(player, "hint.animals.drenkaun.trees", &[Arg::Int(trees as i64), Arg::Int(TREES as i64)]);
        weft_sdk::info!("drenkaun altar {at:?}: {trees} logs round it, {TREES} wanted");
        return true;
    }
    let mid = [at[0] as f64 + 0.5, at[1] as f64, at[2] as f64 + 0.5];
    if !w
        .entities_near(mid, ONE_AT_A_TIME, Some(&DRENKAUN))
        .is_empty()
    {
        w.hint(player, "hint.animals.drenkaun.awake", &[]);
        weft_sdk::info!("drenkaun altar {at:?}: one is already awake near");
        return true;
    }
    crate::taming::take_one(w, player);
    // It rises a few blocks past the altar, away from the player.
    let p = w.entity(player).map_or(mid, |s| s.pos.to_array());
    let (dx, dz) = (mid[0] - p[0], mid[2] - p[2]);
    let len = (dx * dx + dz * dz).sqrt().max(0.1);
    let spot = [mid[0] + dx / len * 6.0, mid[1], mid[2] + dz / len * 6.0];
    let spot = ground(w, spot).unwrap_or(mid);
    let id = w.spawn(&DRENKAUN, spot);
    if id == 0 {
        return true;
    }
    let max = boss().stats(form).health;
    w.attribute_add_modifier(
        id,
        &attribute::MAX_HEALTH,
        "animals:form",
        attr_op::MULTIPLY_BASE,
        (max / boss().stats(boss().form(Age::Stone)).health) as f64 - 1.0,
        0,
    );
    w.entity_edit(id).health(max);
    w.set_component(id, &WARDEN, Warden { form: form.age.tier() as u32, state: RISING, target: player, altar: at, ..Warden::default() });
    w.action_play(id, "rise", false);
    burst(w, spot, DIRT.id(), 40, 1.5);
    for p in w.entities_near(spot, SIGHT, Some(&PLAYER)) {
        w.title(p, "title.animals.drenkaun", &[])
            .times(0.5, 3.0, 1.0);
        w.subtitle(p, "subtitle.animals.drenkaun", &[])
            .times(0.5, 3.0, 1.0);
    }
    weft_sdk::info!("drenkaun {id}: summoned at {at:?} ({} form, {max:.0} health, {trees} logs round the altar)", form.age.name());
    true
}

/// Logs round an altar (its old forest).
fn logs_round(w: &World, at: [i32; 3]) -> u32 {
    let mut n = 0;
    for dx in -TREE_RADIUS..=TREE_RADIUS {
        for dz in -TREE_RADIUS..=TREE_RADIUS {
            for dy in -2..=8 {
                if LOGS.on_block(w.block([at[0] + dx, at[1] + dy, at[2] + dz])) {
                    n += 1;
                }
            }
        }
    }
    n
}

/// The ground under (or a little over) `p`: where something can stand.
fn ground(w: &World, p: [f64; 3]) -> Option<[f64; 3]> {
    let (x, z) = (p[0].floor() as i32, p[2].floor() as i32);
    let y0 = p[1].floor() as i32;
    (y0 - 6..=y0 + 6)
        .rev()
        .find(|&y| solid(w, [x, y - 1, z]) && !solid(w, [x, y, z]) && !solid(w, [x, y + 1, z]))
        .map(|y| [x as f64 + 0.5, y as f64, z as f64 + 0.5])
}

fn solid(w: &World, p: [i32; 3]) -> bool {
    block_info(w.block(p)).is_some_and(|b| b.solid())
}

// ------------------------------------------------------------------ the fight

/// Its goal, every tick: rise, chase, wind up and make an attack, or lie stunned or knocked down.
fn fight(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(mut c) = w.component(id, &WARDEN) else { return false };
    let e = w.tick_elapsed(id);
    if w.component(id, &SCALED) != Some(1) {
        w.set_scale(id, SCALE);
        w.set_component(id, &SCALED, 1);
        // A titan stands its ground: no knockback (`core:knockback_resistance`; before it the
        // goal undid it, PC look boss_drenkaun1: twenty hits had flung it off its arena).
        w.attribute_add_modifier(
            id,
            &attribute::KNOCKBACK_RESISTANCE,
            "animals:size",
            attr_op::ADD,
            bosses::SIZE.band().knockback as f64,
            0,
        );
    }
    for cd in c.cooldown.iter_mut() {
        *cd = cd.saturating_sub(e);
    }
    c.t = c.t.saturating_add(e);
    let b = boss();
    // The nearest player it can fight; none for a while and it goes back into the wood.
    let target = nearest_player(w, st);
    match target {
        Some(p) => {
            c.target = p;
            c.alone = 0;
        }
        None => {
            c.alone += e;
            if c.alone >= LEAVE_TICKS {
                weft_sdk::info!("drenkaun {id}: no one to fight, back into the wood");
                burst(w, st.pos.to_array(), OAK_LEAVES.id(), 40, 2.0);
                w.despawn(id);
                LAST.lock()
                    .unwrap()
                    .get_or_insert_with(HashMap::new)
                    .remove(&id);
                return true;
            }
        }
    }
    let fierce = st.health < st.max_health * b.fierce_at;
    // Its health line, once a second, for the players near.
    if (w.game_time() + id).is_multiple_of(20) {
        let state = match c.state {
            DOWN => "hud.animals.boss_health.down",
            STUNNED => "hud.animals.boss_health.stunned",
            _ if fierce => "hud.animals.boss_health.fierce",
            _ => "hud.animals.boss_health.fighting",
        };
        let pct = (100.0 * st.health / st.max_health.max(1.0)).ceil() as i64;
        for p in w.entities_near(st.pos, SIGHT, Some(&PLAYER)) {
            drop(w.show_at(
                p,
                &BAR,
                "hud.animals.boss_health",
                &[
                    Arg::Key("title.animals.drenkaun"),
                    Arg::Int(st.health.ceil() as i64),
                    Arg::Int(st.max_health.ceil() as i64),
                    Arg::Int(pct),
                    Arg::Key(state),
                ],
            ));
        }
    }
    match c.state {
        RISING => {
            stand(w, id, st);
            if c.t >= RISE_TICKS {
                go(&mut c, CHASE);
            }
        }
        DOWN | STUNNED => {
            stand(w, id, st);
            let ticks = if c.state == DOWN { (b.down_secs * TICKS) as u32 } else { STUN_TICKS };
            if c.t >= ticks {
                w.action_play(id, "", false);
                go(&mut c, CHASE);
            }
        }
        CHASE => {
            let Some(t) = target.and_then(|p| w.entity(p)) else {
                stand(w, id, st);
                w.set_component(id, &WARDEN, c);
                return true;
            };
            let to = [t.pos.x - st.pos.x, t.pos.z - st.pos.z];
            let dist = (to[0] * to[0] + to[1] * to[1]).sqrt();
            let half = st.size.width as f64 / 2.0;
            let facing = facing(st.yaw, to);
            match choose(b, &c, dist - half, facing, w, st, &t) {
                Some(i) => {
                    let a = &b.attacks[i];
                    c.attack = i as u32;
                    c.dir = [(to[0] / dist.max(0.1)) as f32, (to[1] / dist.max(0.1)) as f32];
                    if a.name == "roots" {
                        c.marks = marks(st, &t, fierce);
                    }
                    w.navigate(id, st.pos, 0.0);
                    w.action_play(id, &format!("{}_tell", a.name), false);
                    go(&mut c, WINDUP);
                }
                // Close enough: it stops and turns to them rather than walk into them (PC look
                // boss_drenkaun1: it stood on the player's spot and every swing missed).
                None if dist - half < CLOSE => {
                    stand(w, id, st);
                    face(w, id, [(to[0] / dist.max(0.1)) as f32, (to[1] / dist.max(0.1)) as f32]);
                }
                None => {
                    w.navigate(id, t.pos, 1.0);
                }
            }
        }
        WINDUP => {
            let a = &b.attacks[c.attack as usize];
            stand(w, id, st);
            face(w, id, c.dir);
            if a.name == "roots" {
                for m in c.marks.chunks(3).filter(|m| m[1] != 0.0) {
                    burst(w, [m[0] as f64, m[1] as f64 + 0.1, m[2] as f64], DIRT.id(), 3, 0.6);
                }
            } else {
                burst(w, head(st), OAK_LEAVES.id(), 2, 0.8);
            }
            if c.t as f32 >= a.windup * TICKS {
                w.action_play(id, a.name, false);
                if a.name == "charge" {
                    TRAMPLED
                        .lock()
                        .unwrap()
                        .get_or_insert_with(HashMap::new)
                        .insert(id, Vec::new());
                }
                go(&mut c, ACT);
            }
        }
        ACT => {
            let a = b.attacks[c.attack as usize];
            let hit = stats(&c).hit * a.hit * weakened(b, &c, a.name);
            let cd = (a.cooldown * TICKS * if fierce { bosses::FIERCE } else { 1.0 }) as u32;
            match a.name {
                "charge" => {
                    w.walk(id, c.dir, CHARGE_SPEED, false);
                    face(w, id, c.dir);
                    trample(w, id, st, hit, a.reach as f64, a.knockback);
                    // Into a trunk or a wall: stunned, its window.
                    let ahead = [
                        (st.pos.x + c.dir[0] as f64 * (st.size.width as f64 / 2.0 + 0.6)).floor() as i32,
                        (st.pos.y + 1.0).floor() as i32,
                        (st.pos.z + c.dir[1] as f64 * (st.size.width as f64 / 2.0 + 0.6)).floor() as i32,
                    ];
                    let blocked = c.t > 6 && solid(w, ahead) && solid(w, [ahead[0], ahead[1] + 1, ahead[2]]);
                    if blocked || c.t >= CHARGE_TICKS {
                        TRAMPLED
                            .lock()
                            .unwrap()
                            .get_or_insert_with(HashMap::new)
                            .remove(&id);
                        c.cooldown[c.attack as usize] = cd;
                        if blocked {
                            weft_sdk::info!("drenkaun {id}: charged into {:?}, stunned", ahead);
                            burst(w, head(st), OAK_LOG.id(), 20, 1.0);
                            w.action_play(id, "stunned", true);
                            go(&mut c, STUNNED);
                        } else {
                            go(&mut c, CHASE);
                        }
                    }
                }
                _ => {
                    // The others land at once, then a breath before it moves again.
                    if c.t == e.max(1) || c.t <= e {
                        strike(w, id, st, &c, &a, hit);
                        c.cooldown[c.attack as usize] = cd;
                    }
                    stand(w, id, st);
                    if c.t >= 15 {
                        go(&mut c, CHASE);
                    }
                }
            }
        }
        _ => go(&mut c, CHASE),
    }
    w.set_component(id, &WARDEN, c);
    remember(id, c);
    true
}

/// Its last state, for when it has died (its components are gone by then: boss_drenkaun1's death
/// lines read a fresh one's).
fn remember(id: u64, c: Warden) {
    LAST.lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .insert(id, c);
}

fn go(c: &mut Warden, state: u32) {
    c.state = state;
    c.t = 0;
}

fn stats(c: &Warden) -> vitals::combat::Stats {
    boss().stats(form_of(c))
}

/// A softer attack for each part broken that weakens it.
fn weakened(b: &Boss, c: &Warden, attack: &str) -> f32 {
    b.parts
        .iter()
        .enumerate()
        .filter(|(i, p)| c.broken & (1 << i) != 0 && p.weakens == Some(attack))
        .map(|_| bosses::BROKEN)
        .product()
}

/// Which attack to make now, if any: by where its target stands (`dist` past its body, `facing`
/// the cosine of the angle off its nose).
fn choose(b: &Boss, c: &Warden, dist: f64, facing: f64, w: &World, st: &EntityState, t: &EntityState) -> Option<usize> {
    let ready = |name: &str| {
        b.attacks
            .iter()
            .position(|a| a.name == name)
            .filter(|&i| c.cooldown[i] == 0)
    };
    let reach = |name: &str| b.attack(name).map_or(0.0, |a| a.reach as f64);
    if dist <= reach("stomp") && facing < 0.2 {
        return ready("stomp");
    }
    if dist <= reach("sweep") && facing > 0.5 {
        return ready("sweep");
    }
    let sees = sees(w, st, t);
    if (6.0..=22.0).contains(&dist)
        && sees
        && let Some(i) = ready("charge")
    {
        return Some(i);
    }
    if (9.0..=reach("roots")).contains(&dist) && sees {
        return ready("roots");
    }
    None
}

/// Whether nothing solid stands between its eyes and `t`'s middle.
fn sees(w: &World, st: &EntityState, t: &EntityState) -> bool {
    let from = [st.pos.x, st.pos.y + st.size.eye as f64, st.pos.z];
    let to = [t.pos.x, t.pos.y + t.size.height as f64 * 0.5, t.pos.z];
    let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    len < 1.0 || w.raycast(from, d.map(|v| v / len), len, false).is_none()
}

/// The cosine of the angle between where it faces (`yaw`) and `to`.
fn facing(yaw: f32, to: [f64; 2]) -> f64 {
    let f = yaw_dir(yaw);
    let len = (to[0] * to[0] + to[1] * to[1]).sqrt().max(1e-6);
    (f[0] as f64 * to[0] + f[1] as f64 * to[1]) / len
}

fn face(w: &World, id: u64, dir: [f32; 2]) {
    drop(w.entity_edit(id).yaw((-dir[0]).atan2(-dir[1])));
}

fn stand(w: &World, id: u64, st: &EntityState) {
    if w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
}

/// Its head, ahead of its middle and up (the reindeer's, scaled).
fn head(st: &EntityState) -> [f64; 3] {
    let f = yaw_dir(st.yaw);
    let ahead = st.size.width as f64 * 0.55;
    [st.pos.x + f[0] as f64 * ahead, st.pos.y + st.size.height as f64 * 0.85, st.pos.z + f[1] as f64 * ahead]
}

/// Where roots will burst: under the target, and in its fierce phase two more on the line to it.
fn marks(st: &EntityState, t: &EntityState, fierce: bool) -> [f32; 9] {
    let mut m = [0.0; 9];
    let at = |s: f64| [(st.pos.x + (t.pos.x - st.pos.x) * s) as f32, t.pos.y as f32, (st.pos.z + (t.pos.z - st.pos.z) * s) as f32];
    m[..3].copy_from_slice(&at(1.0));
    if fierce {
        m[3..6].copy_from_slice(&at(0.66));
        m[6..].copy_from_slice(&at(1.33));
    }
    m
}

/// The nearest player it can fight (in survival, alive, in sight range).
fn nearest_player(w: &World, st: &EntityState) -> Option<u64> {
    w.entities_near(st.pos, SIGHT, Some(&PLAYER))
        .into_iter()
        .filter(|&p| !w.outside_survival(p))
        .filter_map(|p| w.entity(p).filter(|s| s.health > 0.0).map(|s| (p, s)))
        .min_by(|a, b| dist2(a.1.pos.to_array(), st.pos.to_array()).total_cmp(&dist2(b.1.pos.to_array(), st.pos.to_array())))
        .map(|(p, _)| p)
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

/// A charge's tick: what is in front of it takes the hit and is thrown, once a charge.
fn trample(w: &World, id: u64, st: &EntityState, hit: f32, reach: f64, knockback: f32) {
    let half = st.size.width as f64 / 2.0;
    let f = yaw_dir(st.yaw);
    let front = [st.pos.x + f[0] as f64 * half, st.pos.y, st.pos.z + f[1] as f64 * half];
    let mut guard = TRAMPLED.lock().unwrap();
    let done = guard
        .get_or_insert_with(HashMap::new)
        .entry(id)
        .or_default();
    for e in w.entities_near(front, half + reach, None) {
        if e == id || done.contains(&e) || spared(w, e) {
            continue;
        }
        let Some(s) = w.entity(e) else { continue };
        if (s.pos.y - st.pos.y).abs() > st.size.height as f64 {
            continue;
        }
        done.push(e);
        w.damage(e, hit, id, damage_cause::ATTACK);
        w.push(e, [f[0] as f64 * knockback as f64, knockback as f64 * 0.4, f[1] as f64 * knockback as f64]);
        weft_sdk::info!("drenkaun {id}: charge hits {e} for {hit:.1}");
    }
}

/// A sweep, stomp or roots landing.
fn strike(w: &World, id: u64, st: &EntityState, c: &Warden, a: &bosses::Attack, hit: f32) {
    let half = st.size.width as f64 / 2.0;
    let f = yaw_dir(st.yaw);
    let mut hits = Vec::new();
    match a.name {
        "sweep" => {
            for e in w.entities_near(st.pos, half + a.reach as f64, None) {
                let Some(s) = w.entity(e) else { continue };
                let to = [s.pos.x - st.pos.x, s.pos.z - st.pos.z];
                if e != id && facing(st.yaw, to) > 0.3 {
                    // Thrown aside and back, the way the antlers swing.
                    let side = [-f[1] as f64, f[0] as f64];
                    let k = a.knockback as f64;
                    hits.push((e, [(side[0] * 0.7 + f[0] as f64 * 0.5) * k, k * 0.35, (side[1] * 0.7 + f[1] as f64 * 0.5) * k]));
                }
            }
            burst(w, head(st), OAK_LEAVES.id(), 16, 2.0);
        }
        "stomp" => {
            for e in w.entities_near(st.pos, half + a.reach as f64, None) {
                let Some(s) = w.entity(e) else { continue };
                let to = [s.pos.x - st.pos.x, s.pos.z - st.pos.z];
                let len = (to[0] * to[0] + to[1] * to[1]).sqrt().max(0.1);
                let k = a.knockback as f64;
                if e != id && (s.pos.y - st.pos.y).abs() < 2.0 {
                    hits.push((e, [to[0] / len * k, k * 0.6, to[1] / len * k]));
                }
            }
            burst(w, st.pos.to_array(), DIRT.id(), 40, half + a.reach as f64);
        }
        _ => {
            for m in c.marks.chunks(3).filter(|m| m[1] != 0.0) {
                let at = [m[0] as f64, m[1] as f64, m[2] as f64];
                burst(w, at, OAK_LOG.id(), 24, ROOT_RADIUS);
                for e in w.entities_near(at, ROOT_RADIUS, None) {
                    if e != id && !hits.iter().any(|(h, _)| *h == e) {
                        hits.push((e, [0.0, a.knockback as f64, 0.0]));
                        w.attribute_add_modifier(
                            e,
                            &attribute::MOVEMENT_SPEED,
                            "animals:warden_roots",
                            attr_op::MULTIPLY_BASE,
                            ROOT_SLOW,
                            ROOT_TICKS,
                        );
                    }
                }
            }
        }
    }
    for (e, push) in hits {
        if spared(w, e) {
            continue;
        }
        w.damage(e, hit, id, damage_cause::ATTACK);
        w.push(e, push);
        weft_sdk::info!("drenkaun {id}: {} hits {e} for {hit:.1}", a.name);
    }
}

/// What its attacks never hit: another Drenkaun, and its calves (PC look boss_drenkaun_hud1: a
/// second one trampled the first one's calf).
fn spared(w: &World, e: u64) -> bool {
    w.component(e, &WARDEN).is_some() || w.entity(e).is_some_and(|s| s.kind == CALF.id())
}

fn burst(w: &World, at: [f64; 3], state: u32, count: u32, spread: f64) {
    drop(
        w.spawn_particles(&BITS, at)
            .block(state)
            .count(count)
            .spread(spread as f32)
            .outward(),
    );
}

// ------------------------------------------------------------------ the damage step and stagger

/// A hit of `raw` on Drenkaun `id`: what it takes after the gear cap, its armour and the floor;
/// its stagger grows by what spilled over and a share of what it took, knocking it down and
/// breaking its parts.
fn hurt(w: &World, id: u64, other: u64, cause: u32, raw: f32) -> f32 {
    let Some(mut c) = w.component(id, &WARDEN) else { return raw };
    let Some(st) = w.entity(id) else { return raw };
    if cause == damage_cause::VOID || raw <= 0.0 {
        return raw;
    }
    // Rising out of the ground it can't be hurt yet.
    if c.state == RISING {
        return 0.0;
    }
    let b = boss();
    let f = form_of(&c);
    let level = b.level(f);
    let size = bosses::SIZE;
    let mut hit = raw * crate::armour::player_cap(w, other, cause, size, level);
    let capped = hit;
    if DamageCause::flags_of(cause) & damage_flags::BYPASSES_ARMOR == 0 {
        hit *= 1.0 - vitals::combat::armour_cut(b.stats(f).armour, hit);
    }
    let (took, over, _) = crate::armour::floor(w, id, size, st.max_health, hit);
    let gained = over + took * STAGGER_SHARE;
    c.stagger += gained;
    c.knock += gained;
    weft_sdk::info!(
        "drenkaun {id}: hit {raw:.2}, capped {capped:.2}, armour {hit:.2}, took {took:.2}, stagger {:.1} ({:.0}% of its health), health {:.1}/{:.1}",
        c.stagger,
        100.0 * c.stagger / st.max_health,
        (st.health - took).max(0.0),
        st.max_health
    );
    // Its parts, in order.
    for (i, p) in b.parts.iter().enumerate() {
        if c.broken & (1 << i) == 0 && c.stagger >= p.at * st.max_health {
            c.broken |= 1 << i;
            let at = head(&st);
            if let Some(item) = item_id(p.drop) {
                w.spawn_item(at, ItemStack::new(item, p.count));
            }
            burst(w, at, OAK_LOG.id(), 30, 1.5);
            for pl in w.entities_near(st.pos, SIGHT, Some(&PLAYER)) {
                w.subtitle(pl, &format!("subtitle.animals.drenkaun.{}", p.name), &[])
                    .times(0.2, 2.0, 0.5);
            }
            weft_sdk::info!("drenkaun {id}: its {} breaks", p.name);
        }
    }
    // A knockdown, unless it is already down.
    if c.knock >= b.knockdown * st.max_health && c.state != DOWN {
        c.knock = 0.0;
        TRAMPLED
            .lock()
            .unwrap()
            .get_or_insert_with(HashMap::new)
            .remove(&id);
        w.action_play(id, "down", true);
        go(&mut c, DOWN);
        weft_sdk::info!("drenkaun {id}: knocked down");
    }
    w.set_component(id, &WARDEN, c);
    remember(id, c);
    took
}

fn item_id(name: &str) -> Option<u32> {
    let id = Item::generated(name.to_string()).id();
    (id != 0).then_some(id)
}

// ------------------------------------------------------------------ loot and the calf

/// It died at `pos`: every player who fought it near gets their roll of its loot, with bad-luck
/// help, and its calf may stay behind.
fn died(w: &World, id: u64, pos: [f64; 3]) {
    let b = boss();
    let last = LAST
        .lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .remove(&id);
    let c = last
        .or_else(|| w.component(id, &WARDEN))
        .unwrap_or_default();
    let f = form_of(&c);
    let index = bosses::BOSSES
        .iter()
        .position(|x| x.name == b.name)
        .unwrap_or(0);
    let lines: Vec<&bosses::Loot> = b.loot_for(f).collect();
    let mut calf = false;
    for p in w.entities_near(pos, FOUGHT, Some(&PLAYER)) {
        let mut luck = w.component(p, &LUCK).unwrap_or([0; 16]);
        let mut got = Vec::new();
        let base = index * 8;
        for (i, l) in lines.iter().enumerate().take(7) {
            let misses = luck[base + i] as u32;
            if roll(w) < l.chance_after(misses) {
                luck[base + i] = 0;
                let n = l.count_at(roll(w));
                if let Some(item) = item_id(l.item) {
                    w.spawn_item([pos[0], pos[1] + 1.0, pos[2]], ItemStack::new(item, n));
                }
                got.push(format!("{} x{n}", l.item));
            } else {
                luck[base + i] = luck[base + i].saturating_add(1);
                got.push(format!("{} missed ({} in a row)", l.item, luck[base + i]));
            }
        }
        let y = base + 7;
        if roll(w) < b.young.chance_after(luck[y] as u32) {
            luck[y] = 0;
            calf = true;
        } else {
            luck[y] = luck[y].saturating_add(1);
        }
        w.set_component(p, &LUCK, luck);
        // A socket stone for the perk web (`vitals::sockets`, a boss's find: one a kill, rolled at
        // its form's age).
        if base_api::skills::drop_stone(w, base_api::skills::StoneSource::Boss, f.age.tier(), [pos[0], pos[1] + 1.0, pos[2]]) {
            got.push("a socket stone".into());
        }
        weft_sdk::info!("drenkaun {id}: loot for player {p}: {}", got.join(", "));
    }
    if calf {
        let at = ground(w, pos).unwrap_or(pos);
        let calf = w.spawn(&CALF, at);
        weft_sdk::info!("drenkaun {id}: its calf {calf} stays behind");
    }
    weft_sdk::info!("drenkaun {id}: died ({} form, stagger {:.0}, parts broken {:03b})", f.age.name(), c.stagger, c.broken);
    TRAMPLED
        .lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .remove(&id);
}

fn roll(w: &World) -> f32 {
    w.random_below(1_000_000) as f32 / 1_000_000.0
}

/// A calf: offered a forest offering, it takes to that player and follows them.
fn calf_interact(w: &World, id: u64, player: u64) -> bool {
    let held = w.inventory(player).get(w.selected_slot(player));
    if held.item != OFFERING.id() {
        return false;
    }
    let owner = w.component(id, &CALF_OWNER).unwrap_or([0; 64]);
    if owner != [0; 64] {
        return false;
    }
    let key = crate::player_key(player);
    let mut k = [0u8; 64];
    for (i, b) in key.bytes().take(64).enumerate() {
        k[i] = b;
    }
    w.set_component(id, &CALF_OWNER, k);
    crate::taming::take_one(w, player);
    if let Some(st) = w.entity(id) {
        drop(
            w.spawn_particles(&crate::HEART, [st.pos.x, st.pos.y + 1.2, st.pos.z])
                .count(6)
                .spread(0.5),
        );
    }
    w.hint(player, "hint.animals.drenkaun_calf.tamed", &[]);
    weft_sdk::info!("drenkaun calf {id}: tamed by player {player}");
    true
}

/// A calf's goal: drawn small; a tamed one follows its owner.
fn calf(w: &World, id: u64, st: &EntityState) -> bool {
    if w.component(id, &SCALED) != Some(1) {
        w.set_scale(id, CALF_SCALE);
        w.set_component(id, &SCALED, 1);
    }
    let owner = w.component(id, &CALF_OWNER).unwrap_or([0; 64]);
    if owner == [0; 64] {
        return false;
    }
    let Some((p, ps)) = w
        .players_iter()
        .find(|&p| {
            crate::player_key(p)
                .as_bytes()
                .iter()
                .take(64)
                .eq(owner.iter().take_while(|&&b| b != 0))
        })
        .and_then(|p| w.entity(p).map(|s| (p, s)))
    else {
        return false;
    };
    let _ = p;
    let d = dist2(ps.pos.to_array(), st.pos.to_array());
    if d > 16.0 && d < 48.0 * 48.0 {
        w.navigate(id, ps.pos, 1.2);
        return true;
    }
    if d <= 9.0 {
        stand(w, id, st);
    }
    d <= 16.0
}

// ------------------------------------------------------------------ the check command

/// `/drenkaun state|hurt <n>|kill|luck`, for the checks (`scripts/drenkaun.txt`): the nearest
/// one's fight state; a hit of `n` from the player as their held weapon's (through the cap, its
/// armour and the floor); its death with loot as from the void; the player's misses.
fn command(cmd: &mut ServerCommand) {
    let (w, player) = (cmd.world(), cmd.player());
    if cmd.text("action") == Some("luck") {
        let luck = w.component(player, &LUCK).unwrap_or([0; 16]);
        cmd.reply("command.animals.line", &[Arg::Text(&format!("misses {:?}", &luck[..8]))]);
        return;
    }
    let Some(me) = w.entity(player) else { return };
    let Some(id) = w
        .entities_near(me.pos, 64.0, Some(&DRENKAUN))
        .into_iter()
        .min_by(|a, b| {
            let d = |e: u64| {
                w.entity(e)
                    .map_or(f64::MAX, |s| dist2(s.pos.to_array(), me.pos.to_array()))
            };
            d(*a).total_cmp(&d(*b))
        })
    else {
        cmd.reply("command.animals.line", &[Arg::Text("no drenkaun near")]);
        return;
    };
    match cmd.text("action").unwrap_or("state") {
        "hurt" => {
            let n = cmd.float("amount").unwrap_or(10.0) as f32;
            w.damage(id, n, player, damage_cause::ATTACK);
        }
        "kill" => {
            w.damage(id, 1.0e6, player, damage_cause::VOID);
        }
        _ => {}
    }
    let c = w.component(id, &WARDEN).unwrap_or_default();
    let st = w.entity(id);
    let line = format!(
        "drenkaun {id}: state {} t {} health {:.1}/{:.1} stagger {:.1} knock {:.1} broken {:03b} cooldowns {:?}",
        ["rising", "chase", "windup", "act", "down", "stunned"]
            .get(c.state as usize)
            .unwrap_or(&"?"),
        c.t,
        st.map_or(0.0, |s| s.health),
        st.map_or(0.0, |s| s.max_health),
        c.stagger,
        c.knock,
        c.broken,
        c.cooldown
    );
    weft_sdk::info!("{line}");
    cmd.reply("command.animals.line", &[Arg::Text(&line)]);
}
