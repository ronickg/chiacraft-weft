//! Borkon, the Height-Keeper (`notes/PROGRESSION.md`, round 6): the mountains' giant horned beast
//! of the high passes, the Bronze Age's boss. Its numbers, attacks, parts and loot are data in
//! `animals_core::bosses`; what every boss shares is `boss.rs`. This is Borkon's own:
//!
//! - **Summoning.** Its altar (`borkon_altar`, the dwarves' work: dressed granite and gold), set
//!   high (`HEIGHT` and up) and among bare rock (`ROCK` stone blocks round it), takes a mountain
//!   offering (`mountain_offering`: edelweiss, gentian, wool and the dwarves' gold).
//! - **Its attacks**, each with a tell first: a horn **charge** that rams and flings what it hits
//!   up and back (into a rock face it is stunned); a **toss** of the horns for whoever is close in
//!   front, thrown high; a **rockfall** where a player far off stands (it rears and stamps, stone
//!   comes down; three falls when it is fierce); and a **leap** to where its target is, up onto a
//!   ledge or down from one, landing with a quake round it. The heights are its ground: a player
//!   on a ledge above or below is leapt at.
//! - **Its loot** includes the Bronze Age key's part, `summit_stone`; its kid is tamed with a
//!   mountain offering.
//!
//! Its model and clips are `source/borkon` (blocky-model, concept A: an ibex titan, Ronald 11-05
//! "A: ibex"): `rise`, a `<attack>_tell` and `<attack>` for each attack, `stunned` and `down`,
//! played by name.

use animals_core::bosses::Attack;
use weft_sdk::*;

use crate::boss::{self, Ctx, Def, Step, Strike, Warden};

pub static BORKON: EntityKind = EntityKind::named("borkon");
pub static KID: EntityKind = EntityKind::named("borkon_kid");
pub static ALTAR: Block = Block::named("borkon_altar");
pub static OFFERING: Item = Item::named("mountain_offering");
pub static SUMMIT: Item = Item::named("summit_stone");
pub static HORN: Item = Item::named("borkon_horn");
pub static FLEECE: Item = Item::named("height_fleece");

static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");
static BUILDING: CreativeTab = CreativeTab::named("base:building");
static INGREDIENTS: CreativeTab = CreativeTab::named("base:ingredients");
static ROCKY: Tag = Tag::named("stone_ore_replaceables");
static GRAVEL: Block = Block::named("base:gravel");
static GRANITE: Block = Block::named("base:granite");
static SNOW: Block = Block::named("base:snow");

/// Its charge: times its walk, its longest run in ticks, and how much of its knockback throws up.
const CHARGE_SPEED: f32 = 3.0;
const CHARGE_TICKS: u32 = 46;
const CHARGE_TOSS: f64 = 0.8;
/// Its altar wants the heights (`HEIGHT` and up) and bare rock round it (`ROCK` stone blocks
/// within `ROCK_RADIUS`).
pub const HEIGHT: i32 = 100;
pub const ROCK: u32 = 40;
const ROCK_RADIUS: i32 = 6;
/// A rockfall's reach round each place it falls, and its slow (the rubble round your feet).
const FALL_RADIUS: f64 = 2.4;
const FALL_SLOW: f64 = -0.5;
const FALL_SLOW_TICKS: u64 = 40;
/// A leap: its flight's seconds, and the quake's reach round where it lands.
const LEAP_SECS: f64 = 1.0;
const QUAKE: f64 = 4.0;
/// Gravity, blocks a second squared (the engine's, for the leap's arc).
const GRAVITY: f64 = 32.0;
/// A target this far above or below it is on a ledge: it leaps.
const LEDGE: f64 = 3.0;

pub static DEF: Def = Def {
    name: "borkon",
    entity: &BORKON,
    young: &KID,
    young_name: "borkon_kid",
    young_scale: 0.24,
    tame_with: &OFFERING,
    // Its model (`source/borkon`, built with the blocky-model skill on concept A: about 3.3 m at
    // the shoulder, its horns' arc 5.8 m up), drawn at its own size.
    model: "borkon",
    texture: "borkon/borkon",
    scale: 1.0,
    speed: 2.4,
    dust: &GRAVEL,
    bits: &GRANITE,
    head,
    choose,
    begin,
    tell,
    act,
};

pub fn init(r: &mut Registry) {
    boss::register(r, &DEF, |w, id, st| boss::fight(&DEF, w, id, st), |w, id, st| boss::young(&DEF, w, id, st));
    r.item(&OFFERING).tab(&INGREDIENTS);
    r.item(&SUMMIT).tab(&INGREDIENTS);
    r.item(&HORN).tab(&INGREDIENTS);
    r.item(&FLEECE).tab(&INGREDIENTS);
    // The heights' offerings: their flowers, wool, and the dwarves' gold.
    r.shapeless(&CRAFTING, "mountain_offering", 1, &["base:edelweiss", "base:gentian", "base:wool", "base:gold_ingot"], 0);
    r.block(&ALTAR)
        .texture("borkon_altar_side").top("borkon_altar_top").bottom("base:granite")
        .sound_group("base:block.stone")
        .hardness(4.0)
        .mined_by("pickaxe", 1)
        .tab(&BUILDING)
        .on_use(|u| summon(&u.world(), u.player(), u.pos().to_array()));
    r.shaped(
        &CRAFTING,
        "borkon_altar",
        1,
        &["gGg", "sgs", "sss"],
        &[('g', "base:granite"), ('G', "base:gold_ingot"), ('s', "base:stone_bricks")],
    );
}

/// A player used the altar at `at`: with an offering in hand, high up among bare rock, and no
/// Borkon near, it rises.
fn summon(w: &World, player: u64, at: [i32; 3]) -> bool {
    let Some(form) = boss::offering(&DEF, w, player, at) else { return false };
    if at[1] < HEIGHT {
        w.hint("hint.animals.borkon.height", &[Arg::Int(at[1] as i64), Arg::Int(HEIGHT as i64)]).to(player);
        weft_sdk::info!("borkon altar {at:?}: at {} high, {HEIGHT} wanted", at[1]);
        return true;
    }
    let rock = rock_round(w, at);
    if rock < ROCK {
        w.hint("hint.animals.borkon.rock", &[Arg::Int(rock as i64), Arg::Int(ROCK as i64)]).to(player);
        weft_sdk::info!("borkon altar {at:?}: {rock} stone round it, {ROCK} wanted");
        return true;
    }
    boss::rise(&DEF, w, player, at, form, &format!("{rock} stone round the altar at {} high", at[1]))
}

/// Bare rock round an altar (the high passes it answers in).
fn rock_round(w: &World, at: [i32; 3]) -> u32 {
    let mut n = 0;
    for dx in -ROCK_RADIUS..=ROCK_RADIUS {
        for dz in -ROCK_RADIUS..=ROCK_RADIUS {
            for dy in -3..=3 {
                if ROCKY.on_block(w.block([at[0] + dx, at[1] + dy, at[2] + dz])) {
                    n += 1;
                }
            }
        }
    }
    n
}

/// Its head (the horns' base), ahead of its middle and up.
fn head(st: &EntityState) -> [f64; 3] {
    let f = yaw_dir(st.yaw);
    let ahead = st.size.width as f64 * 0.55;
    [st.pos.x + f[0] as f64 * ahead, st.pos.y + st.size.height as f64 * 0.8, st.pos.z + f[1] as f64 * ahead]
}

/// Which attack to make now, if any: the heights first (a target on a ledge above or below it is
/// leapt at), then by where its target stands.
fn choose(x: &Ctx) -> Option<usize> {
    let sees = x.sees();
    if x.above().abs() >= LEDGE
        && x.dist <= x.reach("leap")
        && let Some(i) = x.ready("leap")
    {
        return Some(i);
    }
    if x.dist <= x.reach("toss") && x.facing > 0.4 {
        return x.ready("toss");
    }
    if (6.0..=22.0).contains(&x.dist)
        && sees
        && let Some(i) = x.ready("charge")
    {
        return Some(i);
    }
    if (9.0..=x.reach("rockfall")).contains(&x.dist)
        && let Some(i) = x.ready("rockfall")
    {
        return Some(i);
    }
    if (8.0..=x.reach("leap")).contains(&x.dist) {
        return x.ready("leap");
    }
    None
}

/// Where a rockfall comes down (under the target; in its fierce phase two more beside it) or where
/// a leap lands (the target's spot).
fn begin(x: &Ctx, c: &mut Warden) {
    let name = x.b.attacks[c.attack as usize].name;
    let t = x.t.pos;
    let mut m = [0.0; 9];
    m[..3].copy_from_slice(&[t.x as f32, t.y as f32, t.z as f32]);
    if name == "rockfall" && x.fierce {
        let side = [-c.dir[1] * 3.0, c.dir[0] * 3.0];
        m[3..6].copy_from_slice(&[t.x as f32 + side[0], t.y as f32, t.z as f32 + side[1]]);
        m[6..].copy_from_slice(&[t.x as f32 - side[0], t.y as f32, t.z as f32 - side[1]]);
    }
    if name == "rockfall" || name == "leap" {
        c.marks = m;
    }
}

/// Its tells: pebbles and snow already falling where the rocks will come down, or grit kicked up
/// at its head.
fn tell(w: &World, _: u64, st: &EntityState, c: &Warden, a: &Attack) {
    if a.name == "rockfall" {
        for m in c.marks.chunks(3).filter(|m| m[1] != 0.0) {
            boss::burst(w, [m[0] as f64, m[1] as f64 + 6.0, m[2] as f64], GRAVEL.id(), 3, 1.2);
            boss::burst(w, [m[0] as f64, m[1] as f64 + 0.1, m[2] as f64], SNOW.id(), 2, FALL_RADIUS);
        }
    } else {
        boss::burst(w, head(st), GRAVEL.id(), 2, 0.8);
    }
}

/// Its attacks: the charge runs, the leap flies and lands, the toss and rockfall land at once.
fn act(w: &World, id: u64, st: &EntityState, c: &mut Warden, a: &Attack, s: Strike) -> Step {
    match a.name {
        "charge" => boss::charge(&DEF, w, id, st, c, a, s, CHARGE_SPEED, CHARGE_TICKS, CHARGE_TOSS),
        "leap" => leap(w, id, st, c, a, s),
        _ => {
            if s.first(c) {
                strike(w, id, st, c, a, s.hit);
            }
            boss::stand(w, id, st);
            if c.t >= 15 { Step::Done } else { Step::Going }
        }
    }
}

/// A leap: launched on its first tick in an arc to the marked spot, it lands (on the ground again
/// after a few ticks, or at the latest a second and a half on) with a quake round it.
fn leap(w: &World, id: u64, st: &EntityState, c: &Warden, a: &Attack, s: Strike) -> Step {
    if s.first(c) {
        let to = [c.marks[0] as f64 - st.pos.x, c.marks[1] as f64 - st.pos.y, c.marks[2] as f64 - st.pos.z];
        let t = LEAP_SECS;
        let v = [to[0] / t, (to[1] + 0.5 * GRAVITY * t * t) / t, to[2] / t];
        drop(w.entity_modify(id).velocity(v));
        weft_sdk::info!("borkon {id}: leaps {:.1} across, {:.1} up", (to[0] * to[0] + to[2] * to[2]).sqrt(), to[1]);
        return Step::Going;
    }
    let landed = c.t > 8 && st.on_ground != 0;
    if !landed && (c.t as f32) < 1.5 * boss::TICKS {
        return Step::Going;
    }
    // The quake round where it lands.
    let k = a.knockback as f64;
    let mut hits = Vec::new();
    for e in w.entities_near(st.pos, st.size.width as f64 / 2.0 + QUAKE, None) {
        let Some(o) = w.entity(e) else { continue };
        if (o.pos.y - st.pos.y).abs() > 2.5 {
            continue;
        }
        let to = [o.pos.x - st.pos.x, o.pos.z - st.pos.z];
        let len = (to[0] * to[0] + to[1] * to[1]).sqrt().max(0.1);
        hits.push((e, [to[0] / len * k, k * 0.7, to[1] / len * k]));
    }
    boss::burst(w, st.pos.to_array(), GRANITE.id(), 50, QUAKE);
    boss::land(&DEF, w, id, a, s.hit, hits);
    Step::Done
}

/// A toss or a rockfall landing.
fn strike(w: &World, id: u64, st: &EntityState, c: &Warden, a: &Attack, hit: f32) {
    let half = st.size.width as f64 / 2.0;
    let f = yaw_dir(st.yaw);
    let k = a.knockback as f64;
    let mut hits = Vec::new();
    match a.name {
        "toss" => {
            for e in w.entities_near(st.pos, half + a.reach as f64, None) {
                let Some(o) = w.entity(e) else { continue };
                let to = [o.pos.x - st.pos.x, o.pos.z - st.pos.z];
                if boss::facing(st.yaw, to) > 0.3 {
                    // Thrown high, a little forward: the horns hook under and lift.
                    hits.push((e, [f[0] as f64 * k * 0.5, k * 1.2, f[1] as f64 * k * 0.5]));
                }
            }
            boss::burst(w, head(st), GRAVEL.id(), 14, 1.6);
        }
        _ => {
            for m in c.marks.chunks(3).filter(|m| m[1] != 0.0) {
                let at = [m[0] as f64, m[1] as f64, m[2] as f64];
                boss::burst(w, [at[0], at[1] + 3.0, at[2]], GRANITE.id(), 30, FALL_RADIUS);
                boss::burst(w, at, GRAVEL.id(), 20, FALL_RADIUS);
                for e in w.entities_near(at, FALL_RADIUS, None) {
                    if e != id && !hits.iter().any(|(h, _)| *h == e) && !boss::spared(w, e) {
                        hits.push((e, [0.0, -k, 0.0]));
                        w.attribute_add_modifier(e, "animals:borkon_rubble", &Modifier::percent(&attribute::MOVEMENT_SPEED, FALL_SLOW), FALL_SLOW_TICKS);
                    }
                }
            }
        }
    }
    boss::land(&DEF, w, id, a, hit, hits);
}
