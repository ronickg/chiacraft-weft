//! Drenkaun, the Wood-Warden (`notes/PROGRESSION.md`, round 6): the old forests' boss, a huge
//! antlered guardian, the first biome boss built. Its numbers, attacks, parts and loot are data in
//! `animals_core::bosses`; what every boss shares (rising, the fight's states, stagger and part
//! breaks, loot with bad-luck help, its calf, the health line) is `boss.rs`. This is Drenkaun's own:
//!
//! - **Summoning.** Its altar (`drenkaun_altar`), set among old trees (`TREES` logs round it),
//!   takes a forest offering (`forest_offering`) and Drenkaun rises beside it.
//! - **Its attacks**, picked by where the player stands, each with a tell first: a **charge** with
//!   its head down (a straight run that tramples; into a trunk or a wall it is stunned), an antler
//!   **sweep** in front, a **stomp** for whoever is under or behind it, and **roots** that burst up
//!   under a player far off and hold them (in threes when it is fierce).
//! - **Its loot** includes the Stone Age key's part, `warden_heartseed`; its calf is tamed with a
//!   forest offering.
//!
//! Its model and clips are `source/drenkaun` (blocky-model, concept A's colors): `rise`, a
//! `<attack>_tell` and `<attack>` for each attack, `stunned` and `down`, played by name.

use animals_core::bosses::Attack;
use weft_sdk::*;

use crate::boss::{self, Ctx, Def, Step, Strike, Warden};

pub static DRENKAUN: EntityKind = EntityKind::named("drenkaun");
pub static CALF: EntityKind = EntityKind::named("drenkaun_calf");
pub static ALTAR: Block = Block::named("drenkaun_altar");
pub static OFFERING: Item = Item::named("forest_offering");
pub static HEARTSEED: Item = Item::named("warden_heartseed");
pub static ANTLER: Item = Item::named("warden_antler");
pub static BARK: Item = Item::named("warden_bark");

static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");
static BUILDING: CreativeTab = CreativeTab::named("base:building");
static INGREDIENTS: CreativeTab = CreativeTab::named("base:ingredients");
static LOGS: Tag = Tag::named("logs");
static DIRT: Block = Block::named("base:dirt");
static OAK_LOG: Block = Block::named("base:oak_log");
static OAK_LEAVES: Block = Block::named("base:oak_leaves");

/// Its charge: times its walk, and its longest run in ticks.
const CHARGE_SPEED: f32 = 3.2;
const CHARGE_TICKS: u32 = 50;
/// Logs within `TREE_RADIUS` an altar needs round it: the old forest it answers in.
pub const TREES: u32 = 8;
const TREE_RADIUS: i32 = 7;
/// Seconds roots hold a player (slowed by `ROOT_SLOW`), and how wide they burst.
const ROOT_TICKS: u64 = 60;
const ROOT_SLOW: f64 = -0.75;
const ROOT_RADIUS: f64 = 2.0;

pub static DEF: Def = Def {
    name: "drenkaun",
    entity: &DRENKAUN,
    young: &CALF,
    young_name: "drenkaun_calf",
    young_scale: 0.22,
    tame_with: &OFFERING,
    // Its model (`source/drenkaun`, built with the blocky-model skill on concept A's colors: about
    // 4 m at the shoulder), drawn at its own size.
    model: "drenkaun",
    texture: "drenkaun/drenkaun",
    scale: 1.0,
    speed: 2.6,
    dust: &DIRT,
    bits: &OAK_LOG,
    head,
    choose,
    begin,
    tell,
    act,
};

pub fn init(r: &mut Registry) {
    boss::register(r, &DEF, |w, id, st| boss::fight(&DEF, w, id, st), |w, id, st| boss::young(&DEF, w, id, st));
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
        .on_use(|u| summon(&u.world(), u.player(), u.pos().to_array()));
    r.shaped(
        &CRAFTING,
        "drenkaun_altar",
        1,
        &["mfm", "lml", "mmm"],
        &[('m', "base:mossy_cobblestone"), ('f', "base:flint"), ('l', "#logs")],
    );
}

/// A player used the altar at `at`: with an offering in hand, among old trees, and no Drenkaun
/// near, it rises.
fn summon(w: &World, player: u64, at: [i32; 3]) -> bool {
    let Some(form) = boss::offering(&DEF, w, player, at) else { return false };
    let trees = logs_round(w, at);
    if trees < TREES {
        w.hint(player, "hint.animals.drenkaun.trees", &[Arg::Int(trees as i64), Arg::Int(TREES as i64)]);
        weft_sdk::info!("drenkaun altar {at:?}: {trees} logs round it, {TREES} wanted");
        return true;
    }
    boss::rise(&DEF, w, player, at, form, &format!("{trees} logs round the altar"))
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

/// Its head, ahead of its middle and up.
fn head(st: &EntityState) -> [f64; 3] {
    let f = yaw_dir(st.yaw);
    let ahead = st.size.width as f64 * 0.55;
    [st.pos.x + f[0] as f64 * ahead, st.pos.y + st.size.height as f64 * 0.85, st.pos.z + f[1] as f64 * ahead]
}

/// Which attack to make now, if any: by where its target stands.
fn choose(x: &Ctx) -> Option<usize> {
    if x.dist <= x.reach("stomp") && x.facing < 0.2 {
        return x.ready("stomp");
    }
    if x.dist <= x.reach("sweep") && x.facing > 0.5 {
        return x.ready("sweep");
    }
    let sees = x.sees();
    if (6.0..=22.0).contains(&x.dist)
        && sees
        && let Some(i) = x.ready("charge")
    {
        return Some(i);
    }
    if (9.0..=x.reach("roots")).contains(&x.dist) && sees {
        return x.ready("roots");
    }
    None
}

/// Where roots will burst: under the target, and in its fierce phase two more on the line to it.
fn begin(x: &Ctx, c: &mut Warden) {
    if x.b.attacks[c.attack as usize].name != "roots" {
        return;
    }
    let (st, t) = (x.st, x.t);
    let mut m = [0.0; 9];
    let at = |s: f64| [(st.pos.x + (t.pos.x - st.pos.x) * s) as f32, t.pos.y as f32, (st.pos.z + (t.pos.z - st.pos.z) * s) as f32];
    m[..3].copy_from_slice(&at(1.0));
    if x.fierce {
        m[3..6].copy_from_slice(&at(0.66));
        m[6..].copy_from_slice(&at(1.33));
    }
    c.marks = m;
}

/// Its tells: the roots' places stirring, or leaves shaken from its antlers.
fn tell(w: &World, _: u64, st: &EntityState, c: &Warden, a: &Attack) {
    if a.name == "roots" {
        for m in c.marks.chunks(3).filter(|m| m[1] != 0.0) {
            boss::burst(w, [m[0] as f64, m[1] as f64 + 0.1, m[2] as f64], DIRT.id(), 3, 0.6);
        }
    } else {
        boss::burst(w, head(st), OAK_LEAVES.id(), 2, 0.8);
    }
}

/// Its attacks: the charge runs; the sweep, stomp and roots land at once, then a breath.
fn act(w: &World, id: u64, st: &EntityState, c: &mut Warden, a: &Attack, s: Strike) -> Step {
    if a.name == "charge" {
        return boss::charge(&DEF, w, id, st, c, a, s, CHARGE_SPEED, CHARGE_TICKS, 0.4);
    }
    if s.first(c) {
        strike(w, id, st, c, a, s.hit);
    }
    boss::stand(w, id, st);
    if c.t >= 15 { Step::Done } else { Step::Going }
}

/// A sweep, stomp or roots landing.
fn strike(w: &World, id: u64, st: &EntityState, c: &Warden, a: &Attack, hit: f32) {
    let half = st.size.width as f64 / 2.0;
    let f = yaw_dir(st.yaw);
    let k = a.knockback as f64;
    let mut hits = Vec::new();
    match a.name {
        "sweep" => {
            for e in w.entities_near(st.pos, half + a.reach as f64, None) {
                let Some(s) = w.entity(e) else { continue };
                let to = [s.pos.x - st.pos.x, s.pos.z - st.pos.z];
                if boss::facing(st.yaw, to) > 0.3 {
                    // Thrown aside and back, the way the antlers swing.
                    let side = [-f[1] as f64, f[0] as f64];
                    hits.push((e, [(side[0] * 0.7 + f[0] as f64 * 0.5) * k, k * 0.35, (side[1] * 0.7 + f[1] as f64 * 0.5) * k]));
                }
            }
            boss::burst(w, head(st), OAK_LEAVES.id(), 16, 2.0);
        }
        "stomp" => {
            for e in w.entities_near(st.pos, half + a.reach as f64, None) {
                let Some(s) = w.entity(e) else { continue };
                let to = [s.pos.x - st.pos.x, s.pos.z - st.pos.z];
                let len = (to[0] * to[0] + to[1] * to[1]).sqrt().max(0.1);
                if (s.pos.y - st.pos.y).abs() < 2.0 {
                    hits.push((e, [to[0] / len * k, k * 0.6, to[1] / len * k]));
                }
            }
            boss::burst(w, st.pos.to_array(), DIRT.id(), 40, half + a.reach as f64);
        }
        _ => {
            for m in c.marks.chunks(3).filter(|m| m[1] != 0.0) {
                let at = [m[0] as f64, m[1] as f64, m[2] as f64];
                boss::burst(w, at, OAK_LOG.id(), 24, ROOT_RADIUS);
                for e in w.entities_near(at, ROOT_RADIUS, None) {
                    if e != id && !hits.iter().any(|(h, _)| *h == e) && !boss::spared(w, e) {
                        hits.push((e, [0.0, k, 0.0]));
                        w.attribute_add_modifier(e, &attribute::MOVEMENT_SPEED, "animals:warden_roots", attr_op::MULTIPLY_BASE, ROOT_SLOW, ROOT_TICKS);
                    }
                }
            }
        }
    }
    boss::land(&DEF, w, id, a, hit, hits);
}
