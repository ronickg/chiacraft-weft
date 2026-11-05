//! The crowd bench's per-goal timer (`reports/Creature AI.md`, part 3, step 0): after
//! `/animalprof on` (which also starts the rows over), every goal wrapped by `Timed::timed` adds
//! the nanoseconds it took, how often it ran and how often it took the turn to its name's row;
//! `/animalprof dump` logs the rows since as one `ANIMAL_GOALS` line (µs per tick and per call)
//! and starts them over. Off, a goal costs one atomic load more. `/animalprof starve` sets every wild animal within 96 blocks hungry and thirsty, so
//! the bench's second pass runs foraging, drinking and hunting.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::time::Instant;

use weft_sdk::ai::{Goal, Goals};
use weft_sdk::*;

/// Whether the timer runs (`/animalprof on|off`).
static ON: AtomicBool = AtomicBool::new(false);
/// The goal names, in the order first seen, and their sums: nanoseconds, runs, and runs that
/// took the turn.
const ROWS: usize = 32;
static NAMES: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());
static NANOS: [AtomicU64; ROWS] = [const { AtomicU64::new(0) }; ROWS];
static CALLS: [AtomicU64; ROWS] = [const { AtomicU64::new(0) }; ROWS];
static TOOK: [AtomicU64; ROWS] = [const { AtomicU64::new(0) }; ROWS];
/// The longest single run, nanoseconds (a spike: one search, one burst).
static MAX: [AtomicU64; ROWS] = [const { AtomicU64::new(0) }; ROWS];
/// The tick the rows started at.
static SINCE: AtomicU64 = AtomicU64::new(0);

fn row(name: &'static str) -> usize {
    let mut names = NAMES.lock().unwrap();
    names.iter().position(|&n| n == name).unwrap_or_else(|| {
        names.push(name);
        (names.len() - 1).min(ROWS - 1)
    })
}

/// While the timer is on: a stamp for `slow`.
pub fn stamp() -> Option<Instant> {
    ON.load(Relaxed).then(Instant::now)
}

/// While the timer is on, logs `what` if it took more than 20 ms since `t` (`stamp`): where a spike
/// in a goal comes from.
pub fn slow(t: Option<Instant>, what: impl FnOnce() -> String) {
    if let Some(t) = t
        && t.elapsed().as_millis() >= 20
    {
        weft_sdk::warn!("ANIMAL_SLOW {} ms: {}", t.elapsed().as_millis(), what());
    }
}

/// A goal, timed under `name` while the timer is on.
pub struct Time<G> {
    row: usize,
    goal: G,
}

impl<G: Goal> Goal for Time<G> {
    fn run(&self, w: &World, id: u64, st: &EntityState) -> bool {
        if !ON.load(Relaxed) {
            return self.goal.run(w, id, st);
        }
        let t = Instant::now();
        let took = self.goal.run(w, id, st);
        let ns = t.elapsed().as_nanos() as u64;
        NANOS[self.row].fetch_add(ns, Relaxed);
        MAX[self.row].fetch_max(ns, Relaxed);
        CALLS[self.row].fetch_add(1, Relaxed);
        TOOK[self.row].fetch_add(took as u64, Relaxed);
        took
    }
    fn passed_over(&self, w: &World, id: u64) {
        self.goal.passed_over(w, id)
    }
}

/// `Goals::then`, timed.
pub trait Timed {
    fn timed(self, name: &'static str, g: impl Goal) -> Self;
}

impl Timed for Goals {
    fn timed(self, name: &'static str, g: impl Goal) -> Goals {
        self.then(Time { row: row(name), goal: g })
    }
}

/// The rows since they last started over, as one line, and a fresh start.
fn report(w: &World) -> String {
    let now = w.game_time();
    let ticks = now.saturating_sub(SINCE.swap(now, Relaxed)).max(1) as f64;
    let names = NAMES.lock().unwrap().clone();
    let mut total = 0.0;
    let mut rows = Vec::new();
    for (i, name) in names.iter().enumerate().take(ROWS) {
        let (ns, calls, took, max) =
            (NANOS[i].swap(0, Relaxed), CALLS[i].swap(0, Relaxed), TOOK[i].swap(0, Relaxed), MAX[i].swap(0, Relaxed));
        if calls == 0 {
            continue;
        }
        let us = ns as f64 / 1000.0;
        total += us;
        rows.push(format!(
            "{name} {:.1}us/t {:.2}us/call {}/{} max {:.0}us",
            us / ticks,
            us / calls as f64,
            took,
            calls,
            max as f64 / 1000.0
        ));
    }
    format!("ANIMAL_GOALS {:.0} ticks | total {:.1} us/tick | {}", ticks, total / ticks, rows.join(" | "))
}

pub fn init(r: &mut Registry) {
    // The rows in the same order every run.
    row("upkeep");
    r.command("animalprof", "<what:word(on,off,dump,starve,count,bite,wind,senses,kill,fear,water)> [value:float]", |cmd| {
        let w = cmd.world();
        let text = match cmd.text("what").unwrap_or("") {
            "on" => {
                ON.store(true, Relaxed);
                report(&w);
                "animal goal timer on".to_string()
            }
            "dump" => report(&w),
            "water" => {
                // Every wild animal within 96 blocks has this share of its water.
                let Some(me) = w.entity(cmd.player()) else { return };
                let v = cmd.float("value").unwrap_or(0.5) as f32;
                let mut n = 0;
                for id in w.entities_near(me.pos, 96.0, None) {
                    if let Some(a) = w.component_mut(id, &crate::animal::ANIMAL)
                        && a.species != 0
                    {
                        a.thirst = (1.0 - v).clamp(0.0, 1.0);
                        n += 1;
                    }
                }
                format!("{n} animals at {:.0}% water", v * 100.0)
            }
            "kill" => {
                // The player kills the wild animal nearest him (a hunter's kill: fear there, and
                // its kind's reputation of him falls).
                let Some(me) = w.entity(cmd.player()) else { return };
                let near = w
                    .entities_near(me.pos, 48.0, None)
                    .into_iter()
                    .filter(|&id| {
                        w.component(id, &crate::animal::ANIMAL)
                            .is_some_and(|a| a.species != 0)
                            && !crate::animal::tamed(&w, id)
                    })
                    .filter_map(|id| Some((crate::dist2(w.entity(id)?.pos, me.pos), id)))
                    .min_by(|a, b| a.0.total_cmp(&b.0));
                let Some((_, id)) = near else { return };
                w.damage(id, 1e6, cmd.player(), damage_cause::ATTACK);
                format!("KILL {id}")
            }
            "fear" => {
                // The fear of the ground where the player stands, and red deer's (and hares')
                // reputation of him there.
                let Some(me) = w.entity(cmd.player()) else { return };
                let p = me.pos.to_array();
                let rep = |name: &str| {
                    crate::species::SPECIES
                        .iter()
                        .position(|s| s.name == name)
                        .map_or(0.0, |i| crate::memory::reputation(&w, i, p, cmd.player()))
                };
                format!(
                    "ANIMAL_FEAR here {:.2}, red_deer reputation {:.2}, hare {:.2}",
                    crate::memory::fear_at(&w, p),
                    rep("red_deer"),
                    rep("hare")
                )
            }
            "wind" => {
                // The wind's way the animals smell by (degrees clockwise from north; none: the
                // weather's own), for a test.
                let v = cmd.float("value").map_or(f32::NAN, |v| v as f32);
                crate::senses::WIND_TEST.store(v.to_bits(), Relaxed);
                format!("animals' wind toward {v}")
            }
            "senses" => {
                // What the animals nearest the player have noticed: awareness of hunters and of
                // players, by whom.
                let Some(me) = w.entity(cmd.player()) else { return };
                let mut near: Vec<(f64, u64)> = w
                    .entities_near(me.pos, 48.0, None)
                    .into_iter()
                    .filter_map(|id| Some((crate::dist2(w.entity(id)?.pos, me.pos), id)))
                    .filter(|&(_, id)| w.component(id, &crate::senses::SENSES).is_some())
                    .collect();
                near.sort_by(|a, b| a.0.total_cmp(&b.0));
                let rows: Vec<String> = near
                    .iter()
                    .take(8)
                    .filter_map(|&(d2, id)| {
                        let s = w.component(id, &crate::senses::SENSES)?;
                        let kind = name_of(abi::entry_kind::ENTITY, w.entity(id)?.kind);
                        Some(format!(
                            "{id} {kind} at {:.1}: player {} fright {:.2}, hunter {} aware {:.2}, kin {}{}, wind {:.0} {:.1} m/s, persona {}, about {:.2} doing {}",
                            d2.sqrt(),
                            s.player,
                            s.fright,
                            s.threat,
                            s.aware,
                            s.kin,
                            if s.sentinel != 0 { " sentinel" } else { "" },
                            s.heading,
                            s.wind,
                            w.component(id, &crate::senses::FACTS)
                                .map(|f| f.persona().map(|v| format!("{v:.2}")).join("/"))
                                .unwrap_or_default(),
                            crate::routines::activity_of(&w, id),
                            ["-", "love", "rest", "hold", "comfort", "drink", "eat", "wander", "watch", "investigate"]
                                .get(crate::choose::doing(&w, id) as usize)
                                .unwrap_or(&"?")
                        ))
                    })
                    .collect();
                format!("ANIMAL_SENSES {}", rows.join(" | "))
            }
            "bite" => {
                // The hunter nearest the player bites the animal nearest it (`taming::hurt` with
                // the hunter as the source: the bitten one runs from it).
                let Some(me) = w.entity(cmd.player()) else { return };
                let near = w.entities_near(me.pos, 48.0, None);
                let species = |id: u64| {
                    w.component(id, &crate::animal::ANIMAL)
                        .and_then(|a| a.species())
                };
                let by_distance = |from: Vec3| {
                    let mut v: Vec<(u64, f64)> = near
                        .iter()
                        .filter_map(|&id| Some((id, crate::dist2(w.entity(id)?.pos, from))))
                        .collect();
                    v.sort_by(|a, b| a.1.total_cmp(&b.1));
                    v
                };
                let Some((hunter, _)) = by_distance(me.pos)
                    .into_iter()
                    .find(|&(id, _)| species(id).is_some_and(|s| s.hunts()))
                else {
                    return;
                };
                let Some(at) = w.entity(hunter).map(|e| e.pos) else { return };
                let Some((prey, _)) = by_distance(at)
                    .into_iter()
                    .find(|&(id, _)| id != hunter && species(id).is_some_and(|s| !s.hunts()))
                else {
                    return;
                };
                w.damage(prey, 1.0, hunter, damage_cause::ATTACK);
                format!("BITE {hunter} bites {prey}")
            }
            "off" => {
                ON.store(false, Relaxed);
                "animal goal timer off".to_string()
            }
            "starve" => {
                let Some(me) = w.entity(cmd.player()) else { return };
                let mut n = 0;
                for id in w.entities_near(me.pos, 96.0, None) {
                    if crate::animal::tamed(&w, id) {
                        continue;
                    }
                    if let Some(a) = w.component_mut(id, &crate::animal::ANIMAL)
                        && a.species != 0
                    {
                        a.food = (a.stat(crate::species::FOOD) * 0.15) as u16;
                        a.thirst = 0.75;
                        n += 1;
                    }
                }
                format!("{n} animals hungry and thirsty")
            }
            _ => {
                let Some(me) = w.entity(cmd.player()) else { return };
                let n = w
                    .entities_near(me.pos, 256.0, None)
                    .into_iter()
                    .filter(|&id| {
                        w.component(id, &crate::animal::ANIMAL)
                            .is_some_and(|a| a.species != 0)
                    })
                    .count();
                format!("ANIMAL_COUNT {n}")
            }
        };
        weft_sdk::info!("{text}");
        cmd.reply("command.animals.line", &[Arg::Text(&text)]);
    });
}
