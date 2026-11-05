//! Choosing between needs (`reports/Creature AI.md`, part 3, step 3): the goals for what an
//! animal does when nothing is after it (a mate, a rest, the air, water, food, a wander) used to
//! run in a fixed order, the first that had something to do winning. So a resting deer starved
//! where it lay, and one in love walked to its mate parched. Now they run in the order of how
//! pressing each is this tick (`animals_core::needs`: `hunger_score`, `thirst_score`, `LOVE`,
//! `REST`, `UNEASY`/`SUFFERING`), the one it is at already counted `KEEP_AT` more; the first that
//! has something to do still wins, and the others are told they were passed over (a rest put off
//! gets up). What comes before the band stays first whatever it scores: its upkeep, fear,
//! hunting, a knock-out, orders.
//!
//! A tame told to stay (`Need::Hold`) holds against whatever would move it (the air, water, food):
//! it scores just above the most pressing of those.
//!
//! Every animal with a goal list runs its needs through a band: the simple creatures (`wild.rs`),
//! the birds, the wolf, cat, dog, both raptors, the jaguar, the capybara and the wyvern.

use weft_sdk::ai::{Goal, Goals};
use weft_sdk::*;

use animals_core::needs::{self, KEEP_AT, LOVE, REST, SUFFERING, UNEASY, WANDER};
use vitals::temperature::Feeling;

use crate::animal::ANIMAL;
use crate::comfort::{BODY, FEELINGS};

/// What a goal in the band is for, which says how pressing it is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Need {
    Love = 1,
    Rest,
    Hold,
    Comfort,
    Drink,
    Eat,
    Wander,
    /// A herd's sentinel standing watch (`senses::watch_goal`).
    Watch,
}

/// The need whose goal took the turn last tick (0: none; not saved).
static DOING: Component<u8> = Component::named("animal_doing");

pub fn init(r: &mut Registry) {
    r.entity_component(&DOING, 0u8, 0);
    for s in &crate::species::SPECIES {
        r.attach(s.kind, &DOING);
    }
}

/// The needs' goals, run most pressing first.
#[derive(Default)]
pub struct Band {
    goals: Vec<(Need, Box<dyn Goal>)>,
}

impl Band {
    pub fn new() -> Band {
        Band { goals: Vec::new() }
    }
    pub fn with(mut self, need: Need, g: impl Goal) -> Band {
        self.goals.push((need, Box::new(g)));
        self
    }
}

/// How pressing each need is for `id` now, by need (index `Need as usize`).
fn scores(w: &World, id: u64, doing: u8) -> [f32; 9] {
    let mut s = [0.0f32; 9];
    let at = |n: Need| doing == n as u8;
    if let Some(a) = w.component(id, &ANIMAL) {
        let fed = a.food as f32 / a.stat(crate::species::FOOD).max(1.0);
        s[Need::Eat as usize] = needs::hunger_score(fed, at(Need::Eat));
        s[Need::Drink as usize] = needs::thirst_score(1.0 - a.thirst, at(Need::Drink));
    }
    s[Need::Comfort as usize] = match w
        .component(id, &BODY)
        .and_then(|b| FEELINGS.get(b.feel as usize).copied())
    {
        Some(Feeling::Freezing | Feeling::Heatstroke) => SUFFERING,
        Some(Feeling::Cold | Feeling::Hot) => UNEASY,
        _ => 0.0,
    };
    s[Need::Love as usize] = LOVE;
    s[Need::Rest as usize] = REST;
    s[Need::Wander as usize] = WANDER;
    s[Need::Watch as usize] = animals_core::roles::WATCH;
    // Staying put outranks what would move it, and nothing else.
    s[Need::Hold as usize] = s[Need::Comfort as usize]
        .max(s[Need::Drink as usize])
        .max(s[Need::Eat as usize])
        + WANDER;
    if let Some(v) = s.get_mut(doing as usize).filter(|_| doing != 0) {
        *v *= KEEP_AT;
    }
    s
}

impl Goal for Band {
    fn run(&self, w: &World, id: u64, st: &EntityState) -> bool {
        let doing = w.component(id, &DOING).unwrap_or(0);
        let s = scores(w, id, doing);
        // Most pressing first; a tie keeps the list's order.
        let mut order: [usize; 9] = std::array::from_fn(|i| i);
        let order = &mut order[..self.goals.len()];
        order.sort_by(|&a, &b| s[self.goals[b].0 as usize].total_cmp(&s[self.goals[a].0 as usize]));
        let mut took = 0u8;
        for (k, &i) in order.iter().enumerate() {
            let (need, g) = &self.goals[i];
            if g.run(w, id, st) {
                took = *need as u8;
                for &j in &order[k + 1..] {
                    self.goals[j].1.passed_over(w, id);
                }
                break;
            }
        }
        if took != doing {
            // What the fixed order never did: up from a rest, or away from a mate, for food or water.
            let (was, now) = (doing, took);
            if matches!(was, 1 | 2) && matches!(now, 5 | 6) {
                weft_sdk::info!(
                    "animal {id} leaves its {} to {} (scores: {:.2} against {:.2})",
                    if was == Need::Love as u8 { "mate" } else { "rest" },
                    if now == Need::Eat as u8 { "eat" } else { "drink" },
                    s[now as usize],
                    s[was as usize],
                );
            }
            if let Some(d) = w.component_mut(id, &DOING) {
                *d = took;
            }
        }
        took != 0
    }
    fn passed_over(&self, w: &World, id: u64) {
        for (_, g) in &self.goals {
            g.passed_over(w, id);
        }
    }
}

/// Adds the band to a goal list.
pub trait WithBand {
    fn band(self, name: &'static str, band: Band) -> Self;
}

impl WithBand for Goals {
    fn band(self, name: &'static str, band: Band) -> Goals {
        use crate::prof::Timed;
        self.timed(name, band)
    }
}

/// A goal that, passed over, calls `stop`: a rest put off for food gets up.
pub struct Stops<G, S> {
    pub goal: G,
    pub stop: S,
}

impl<G: Goal, S: Fn(&World, u64) + Send + Sync + 'static> Goal for Stops<G, S> {
    fn run(&self, w: &World, id: u64, st: &EntityState) -> bool {
        self.goal.run(w, id, st)
    }
    fn passed_over(&self, w: &World, id: u64) {
        (self.stop)(w, id);
        self.goal.passed_over(w, id);
    }
}
