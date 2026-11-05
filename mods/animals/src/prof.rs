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
    r.command("animalprof", "<what:word(on,off,dump,starve,count,bite)>", |cmd| {
        let w = cmd.world();
        let text = match cmd.text("what").unwrap_or("") {
            "on" => {
                ON.store(true, Relaxed);
                report(&w);
                "animal goal timer on".to_string()
            }
            "dump" => report(&w),
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
