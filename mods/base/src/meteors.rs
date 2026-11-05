//! Fallen stars (reports/Research round 17.md, the addendum's §2; Ronald: "Yes, as designed"): on
//! a meteor shower's nights and a lone fireball's, meteorites land near each player between
//! 23:00 and dawn, `METEOR_LANDINGS` of them, 48-160 blocks out, under the open sky on land and
//! clear of what players built and of campfires. Each leaves a scorched pit 3-5 across with 1-3
//! `base:meteorite` blocks in it, glowing hot (light 9) and cooling over a game hour, and a boom.
//!
//! Mined, a meteorite gives 1-3 lumps of meteoric iron (`METEORITE_DROPS`), iron-nickel worked
//! cold into iron-grade heads before a player can smelt (crafting's `meteoric_iron`). Terrain's
//! old craters hide meteorites too (`land.rs`), and big impacts' cores will.

use std::collections::HashMap;
use std::sync::Mutex;

use vitals::events::{self as ev, Event};
use weft_sdk::*;

use crate::blocks::{DIRT, GRASS, SNOWY_GRASS};

pub static METEORITE: Block = Block::named("meteorite");
pub static METEORIC_IRON: Item = Item::named("meteoric_iron");
static COARSE_DIRT: Block = Block::named("coarse_dirt");
static BOOM: Sound = Sound::named("weather.thunder_far");

/// A player's landings tonight: the day, and hours after 23:00 still to come, latest first.
type Landings = (u64, Vec<f32>);
static TONIGHT: Mutex<Option<HashMap<u64, Landings>>> = Mutex::new(None);

/// Landings come from this hour, for this many hours (to 05:00).
const FROM_HOUR: f32 = 23.0;
const HOURS: f32 = 6.0;
/// Heat steps a hot meteorite cools through (light 9, 6, 3, then none).
const HEAT: u32 = 3;
/// Ticks between rounds, and tries at a landing spot.
const ROUND_TICKS: u64 = 40;
const TRIES: u32 = 24;

pub fn register(r: &mut Registry) {
    // Art's crust, and while it cools its hot look, orange in the cracks (`meteorite_hot_e` glows).
    r.block(&METEORITE)
        .prop("heat", 0..=HEAT)
        .sound_group("stone")
        .texture("meteorite")
        .hardness(1.5)
        .mineable("pickaxe")
        .needs_tool_level(tool_tier::STONE)
        .states(|p, s| match p.get("heat") {
            "3" => s.texture("meteorite_hot").light(9),
            "2" => s.texture("meteorite_hot").light(6),
            "1" => s.texture("meteorite_hot").light(3),
            _ => s,
        })
        .on_scheduled(cool)
        .tag("meteorites")
        .tab(&crate::tabs::NATURAL);
    // Only mined with its tool (a hand-written table needs `Harvested` to say so).
    r.loot(&METEORITE).pool(
        Pool::new()
            .when(Cond::Harvested)
            .items("meteoric_iron", ev::METEORITE_DROPS.0, ev::METEORITE_DROPS.1),
    );
    r.item(&METEORIC_IRON)
        .tag("meteoric_iron")
        .tab(&crate::tabs::INGREDIENTS);
    r.world_tick(tick);
}

/// Ticks between a hot meteorite's heat steps: `METEORITE_COOLS` game hours over them.
fn cool_step() -> u32 {
    (ev::METEORITE_COOLS * vitals::calendar::DAY_TICKS as f32 / 24.0 / HEAT as f32).round() as u32
}

/// A hot meteorite at `pos`, cooling from light 9 over `METEORITE_COOLS` (a landing's, an impact's
/// core: Terrain's carve calls it).
pub fn hot(w: &World, pos: BlockPos) {
    let hot = state_with(METEORITE.id(), "heat", &HEAT.to_string()).unwrap_or(METEORITE.id());
    w.set_block(pos, hot);
    w.schedule_tick(pos, cool_step());
}

/// A hot meteorite cools a step.
fn cool(e: &mut BlockEvent) {
    let (w, pos) = (e.world(), e.pos());
    let heat = w.prop_int(pos, "heat").unwrap_or(0);
    if heat > 0 {
        w.prop_set(pos, "heat", &(heat - 1).to_string());
        if heat > 1 {
            w.schedule_tick(pos, cool_step());
        }
    }
}

fn tick(w: &World) {
    if !w.game_time().is_multiple_of(ROUND_TICKS) || w.dimension() != crate::OVERWORLD.id() {
        return;
    }
    let landings = match crate::events::tonight(w) {
        Some(Event::Meteors { peak: true }) => ev::METEOR_LANDINGS[0],
        Some(Event::Meteors { peak: false }) => ev::METEOR_LANDINGS[1],
        Some(Event::Fireball) => ev::METEOR_LANDINGS[2],
        _ => return,
    };
    let since = (ev::hour_of(w.day_time()) - FROM_HOUR).rem_euclid(24.0);
    if since >= HOURS {
        return;
    }
    let day = w.day_time() / vitals::calendar::DAY_TICKS;
    for player in w.players() {
        let due = {
            let mut tonight = TONIGHT.lock().unwrap();
            let tonight = tonight.get_or_insert_with(HashMap::new);
            let (d, hours) = tonight.entry(player).or_insert((u64::MAX, Vec::new()));
            if *d != day {
                // Tonight's landings for this player, at hours spread over the night.
                let n = landings.0 + w.random_below(landings.1 - landings.0 + 1);
                *hours = (0..n).map(|_| unit(w) * HOURS).collect();
                hours.sort_by(|a, b| b.total_cmp(a));
                *d = day;
            }
            match hours.last() {
                Some(&h) if h <= since => {
                    hours.pop();
                    true
                }
                _ => false,
            }
        };
        if due {
            land_near(w, player);
        }
    }
}

/// A meteorite lands near `player`, where it may.
fn land_near(w: &World, player: u64) {
    if w.outside_survival(player) {
        return;
    }
    let Some(e) = w.entity(player) else { return };
    for _ in 0..TRIES {
        let angle = unit(w) as f64 * std::f64::consts::TAU;
        let reach = ev::METEOR_REACH.0 as f64 + (ev::METEOR_REACH.1 - ev::METEOR_REACH.0) as f64 * unit(w) as f64;
        let (x, z) = ((e.pos.x + angle.cos() * reach).floor() as i32, (e.pos.z + angle.sin() * reach).floor() as i32);
        let Some(top) = crate::events::open_ground(w, x, e.pos.y.floor() as i32, z) else { continue };
        if crate::events::near_built(w, top, ev::METEOR_CLEAR) || crate::mobs::campfire_near(w, top.center()).is_some() {
            continue;
        }
        let n = land(w, top);
        weft_sdk::info!("meteors: a fallen star at {top:?} for {player}, {n} meteorites");
        return;
    }
}

/// The pit at ground `top`: the 3 x 3 middle a block lower, scorched 2 round, 1-3 hot meteorites
/// in its floor, and the boom. How many meteorites.
fn land(w: &World, top: BlockPos) -> u32 {
    let scorch = |p: BlockPos| {
        let s = w.block(p);
        if GRASS.is(s) || SNOWY_GRASS.is(s) || DIRT.is(s) {
            w.set_block(p, COARSE_DIRT.id());
        }
    };
    for dz in -2..=2i32 {
        for dx in -2..=2i32 {
            let p = top.offset(dx, 0, dz);
            if dx.abs() <= 1 && dz.abs() <= 1 {
                // The pit: its top gone, its floor scorched.
                if w.block_flags(p) & abi::block_flags::LIQUID == 0 {
                    w.set_block(p, 0);
                    w.set_block(p.above(), 0);
                }
                scorch(p.below());
            } else if dx.abs() + dz.abs() < 4 {
                scorch(p);
            }
        }
    }
    let n = 1 + w.random_below(3);
    let spots = [(0, 0), (1, 0), (0, 1), (-1, 0), (0, -1)];
    for &(dx, dz) in spots.iter().take(n as usize) {
        hot(w, top.offset(dx, -1, dz));
    }
    w.play_sound(&BOOM, top.center(), 4.0);
    n
}

fn unit(w: &World) -> f32 {
    (w.random() >> 40) as f32 / (1u64 << 24) as f32
}
