//! Loot per hour (`reports/Balance model.md`, 5.5; `vitals::loot`): every kill priced in effort
//! minutes against the time it takes. A wild animal's loot is its meat, hide or carcass pieces
//! (`animals_core::meat`, `hides`) and, for one big enough to stagger, a part from each break of a
//! same-age fight (`animals_core::parts::BREAKS`); a hostile mob's is its vanilla table
//! (`vitals::mobs::LOOT`). Each item is priced at its cheapest cost in the progression graph. The
//! time is finding one (by rarity, shared by a pack that comes at you together; mobs come to you at
//! night), the fight at its home age (`vitals::combat::fight`, the reference player at grade 4), and
//! taking what it leaves.

use animals_core::danger;
use animals_core::meat;
use animals_core::species::{Social, Species};
use vitals::combat::{self, Player, Role};
use vitals::curve::{Age, Em};
use vitals::loot;

use std::collections::HashMap;

use crate::content::{self, Get};
use crate::graph::Solved;

/// One kind of kill and what it comes to.
pub struct Kill {
    pub who: String,
    pub age: Age,
    pub role: Role,
    /// Minutes to find a group, and the kills in it.
    pub find: f32,
    pub kills: f32,
    pub fight_secs: f32,
    /// Its expected drops: item and how many a kill.
    pub drops: Vec<(String, f32)>,
}

impl Kill {
    pub fn minutes(&self) -> f32 {
        loot::kill_minutes(self.find, self.kills, self.fight_secs)
    }

    /// What its drops are worth (an unpriced one counts 0; `unpriced` names it).
    pub fn value(&self, g: &Solved) -> Em {
        self.drops
            .iter()
            .filter_map(|(id, n)| g.cost_of(id).map(|c| c * n))
            .sum()
    }

    pub fn unpriced(&self, g: &Solved) -> Vec<&str> {
        self.drops
            .iter()
            .filter(|(id, _)| g.cost_of(id).is_none())
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// EM earned a minute.
    pub fn rate(&self, g: &Solved) -> f32 {
        self.value(g) / self.minutes()
    }

    /// Whether it fights back (a grazer defends its herd; only prey flees).
    pub fn fights_back(&self) -> bool {
        self.role != Role::Prey
    }
}

fn mean((lo, hi): (u32, u32)) -> f32 {
    (lo + hi) as f32 / 2.0
}

/// A grown wild one's drops, a kill.
pub fn animal_drops(s: &Species) -> Vec<(String, f32)> {
    let mut out: Vec<(String, f32)> = Vec::new();
    if let Some((_, m, lo, hi)) = meat::drops().find(|d| d.0 == s.name) {
        out.push((format!("animals:{m}"), mean((lo, hi))));
    }
    if let Some(c) = meat::carcass(s.name) {
        out.push((meat::CARCASS.to_string(), mean(c)));
    }
    for &(_, item, lo, hi) in animals_core::hides::HIDES.iter().filter(|h| h.0 == s.name) {
        out.push((item.to_string(), mean((lo, hi))));
    }
    if combat::staggers(danger::size(s)) {
        for n in 1..=combat::BREAKS.len() as u32 {
            if let Some((item, lo, hi)) = animals_core::parts::break_drop(s.name, n) {
                match out.iter_mut().find(|(id, _)| id == item) {
                    Some(d) => d.1 += mean((lo, hi)),
                    None => out.push((item.to_string(), mean((lo, hi)))),
                }
            }
        }
    }
    out
}

/// Every kind of kill: each wild species at level 0 at its home age, each hostile mob at night.
pub fn kills() -> Vec<Kill> {
    let mut out = Vec::new();
    for (s, d) in danger::rows() {
        let Some(d) = d else { continue };
        let Some(f) = danger::fight(s, 0, d.home, 1.0) else { continue };
        let kills = match s.bio.social {
            Social::Pack => mean(s.bio.social.group()),
            _ => 1.0,
        };
        out.push(Kill {
            who: s.name.to_string(),
            age: d.home,
            role: d.role,
            find: s.bio.rarity.find_minutes(),
            kills,
            fight_secs: f.ttk,
            drops: animal_drops(s),
        });
    }
    for m in vitals::mobs::MOBS {
        let f = combat::fight(Player { level: m.home.reference(), times: 1.0 }, m.size(), m.level(), m.stats());
        let drops = vitals::mobs::LOOT
            .iter()
            .filter(|l| l.0 == m.name)
            .map(|&(_, item, lo, hi, chance)| (format!("base:{item}"), mean((lo, hi)) * chance))
            .collect();
        out.push(Kill {
            who: m.name.to_string(),
            age: m.home,
            role: m.role,
            find: loot::NIGHT_FIND_MINUTES,
            kills: 1.0,
            fight_secs: f.ttk,
            drops,
        });
    }
    out
}

/// What each kill drop costs a piece: its cheapest kill's minutes, shared among that kill's drops
/// by count × weight (`vitals::loot::shares`).
pub fn prices() -> HashMap<String, Em> {
    let mut out: HashMap<String, Em> = HashMap::new();
    for k in kills() {
        for (id, p) in loot::shares(k.minutes(), &k.drops) {
            let e = out.entry(id.to_string()).or_insert(p);
            *e = e.min(p);
        }
    }
    out
}

/// The kill an item is cheapest from.
pub fn cheapest(item: &str) -> Option<String> {
    kills()
        .into_iter()
        .filter_map(|k| {
            let p = loot::shares(k.minutes(), &k.drops).find(|d| d.0 == item)?.1;
            Some((k.who.clone(), p))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(who, _)| who)
}

/// No kill pays over its band (`vitals::loot::band`); every kill drop has a weight (unless made or
/// gathered too) and a price.
pub fn check(g: &Solved) -> Vec<String> {
    let mut out = Vec::new();
    // The weights and the tags agree: what a kill drops has a weight, and is priced from kills.
    for &(id, _) in loot::KILL_DROPS {
        if !matches!(content::rule(id).map(|r| r.get), Some(Get::Kill)) {
            out.push(format!("{id} has a kill weight but isn't tagged Kill"));
        }
    }
    for k in kills() {
        for (id, _) in &k.drops {
            // One also made or gathered (a spider's string) keeps that price; its share weighs 1.
            let elsewhere = matches!(content::rule(id).map(|r| r.get), Some(Get::Made | Get::World(..)));
            if loot::weight(id).is_none() && !elsewhere {
                out.push(format!("{}: drops {id}, which has no weight in vitals::loot::KILL_DROPS", k.who));
            }
        }
        for id in k.unpriced(g) {
            out.push(format!("{}: drops {id}, which has no price in the graph", k.who));
        }
        if k.drops.is_empty() {
            continue;
        }
        // Over the band, a kill pays more than its time: a faucet to close. Under it, it's only a
        // poor way to spend time (carrion from an owl), listed in the sheet.
        let (_, hi) = loot::band(k.fights_back());
        let r = k.rate(g);
        if r > hi {
            out.push(format!(
                "{} ({}): {:.1} EM of loot in {:.1} min, {:.2} EM a minute, over {hi:.2}",
                k.who,
                k.age.name(),
                k.value(g),
                k.minutes(),
                r
            ));
        }
    }
    out
}
