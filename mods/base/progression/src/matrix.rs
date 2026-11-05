//! The combat matrix (`reports/Balance model.md`, 5.5): every gear age against every size and
//! role, and against every species at every tenth level, through `vitals::combat`'s simulator.
//! Fails when a same-age fight leaves its band, a fight inside an age runs past twice its band's
//! top, over-tier gear beats a floor, a titan stops being a threat over-tier, a tame outgrows its
//! saddle or ceiling, or a tweak is out of bounds.

use animals_core::danger::{self, DANGER, Stat};
use animals_core::species::{SPECIES, Species};
use vitals::combat::{self, Player, ROLES, Role, SIZES, Size};
use vitals::curve::{AGES, Age, GRADE_CEILING, age_of};

/// Levels the species checks step through.
pub const LEVEL_STEP: u32 = 10;
/// A titan over-tier still downs a player in this many hits at most (Ronald, 11-06: "can still
/// down you in two hits"), by the size's own numbers at grade 4: a role moves it (a grazer's
/// softer hit takes about three).
pub const TITAN_HITS: f32 = 2.5;

fn same_age(size: Size, role: Role, level: f32, them: combat::Stats, who: &str, out: &mut Vec<String>) {
    let b = size.band();
    let exact = combat::fight(Player { level, times: 1.0 }, size, level, them);
    if !(b.ttk.0..=b.ttk.1).contains(&exact.ttk) {
        out.push(format!(
            "{who} ({} {}, level {level:.0}): a same-age fight takes {:.1} s, outside {}-{} s",
            size.name(),
            role.name(),
            exact.ttk,
            b.ttk.0,
            b.ttk.1
        ));
    }
    let age = age_of(level);
    let mid = combat::fight(Player::of(age), size, level, them);
    if mid.ttk > 2.0 * b.ttk.1 {
        out.push(format!(
            "{who} ({} {}, level {level:.0}): the {}'s reference takes {:.1} s, over twice the band's {} s",
            size.name(),
            role.name(),
            age.name(),
            mid.ttk,
            b.ttk.1
        ));
    }
    for gear in AGES.iter().filter(|a| a.tier() > age.tier()) {
        for times in [1.0, GRADE_CEILING] {
            let f = combat::fight(Player { level: gear.reference(), times }, size, level, them);
            if let Some(floor) = b.floor
                && f.ttk + 1e-3 < floor
            {
                out.push(format!(
                    "{who} ({}, level {level:.0}): {} gear ×{times} kills it in {:.1} s, under the {floor} s floor",
                    size.name(),
                    gear.name(),
                    f.ttk
                ));
            }
        }
    }
}

/// Every size and role at every age's levels.
pub fn sizes() -> Vec<String> {
    let mut out = Vec::new();
    for age in AGES {
        for depth in 0..10 {
            let level = age.start() + depth as f32;
            let them = combat::derive_base(Size::Titan, level);
            for gear in AGES.iter().filter(|a| a.tier() > age.tier()) {
                let f = combat::fight(Player::of(*gear), Size::Titan, level, them);
                if f.hits > TITAN_HITS {
                    out.push(format!(
                        "a titan (level {level:.0}): {} gear takes {:.1} of its hits, over {TITAN_HITS}",
                        gear.name(),
                        f.hits
                    ));
                }
            }
        }
    }
    for size in SIZES {
        for role in ROLES {
            for age in AGES {
                for depth in 0..10 {
                    let level = age.start() + depth as f32;
                    same_age(size, role, level, combat::derive(size, role, level), "a creature", &mut out);
                }
            }
        }
    }
    out
}

/// Every species at every tenth level, tweaks and all.
pub fn species() -> Vec<String> {
    let mut out = Vec::new();
    for s in SPECIES.iter() {
        let Some(d) = danger::danger(s.name) else {
            out.push(format!("{} has no danger row (add it to `animals_core::danger::DANGER`)", s.name));
            continue;
        };
        let size = danger::size(s);
        for level in (danger::LEVELS.0..=danger::LEVELS.1).step_by(LEVEL_STEP as usize) {
            let Some(them) = danger::stats(s, level) else { continue };
            same_age(size, d.role, d.area_level(level), them, s.name, &mut out);
        }
    }
    out
}

/// Tweaks are inside ×0.75-×1.25, say why, and name a real species.
pub fn tweaks() -> Vec<String> {
    let mut out = Vec::new();
    for d in &DANGER {
        if !SPECIES.iter().any(|s| s.name == d.species) {
            out.push(format!("danger row {} names no species", d.species));
        }
        for (stat, t) in d.tweaks {
            if !t.ok() {
                out.push(format!("{}'s {} tweak ×{} (\"{}\") is out of bounds or gives no reason", d.species, stat.name(), t.x, t.why));
            }
        }
    }
    out
}

/// A saddle carries a tame to its age and no further; its own bonuses stop at the ceiling.
pub fn tames() -> Vec<String> {
    let mut out = Vec::new();
    for s in SPECIES.iter() {
        let Some(d) = danger::danger(s.name) else { continue };
        for saddle in AGES {
            for level in [0, 100, 200] {
                let own = d.area_level(level);
                let Some(at) = danger::tame_level(s, level, Some(saddle)) else { continue };
                if at > own.max(saddle.reference()) + 1e-3 {
                    out.push(format!("a {} tame on a {} saddle fights at level {at:.0}, past its saddle", s.name, saddle.name()));
                }
                let (Some(top), Some(plain)) = (danger::tame(s, level, Some(saddle), 99.0), danger::tame(s, level, Some(saddle), 1.0))
                else {
                    continue;
                };
                if top.health > plain.health * danger::TAME_CEILING * 1.0001 {
                    out.push(format!("a {} tame's bonuses pass the ×{} ceiling", s.name, danger::TAME_CEILING));
                }
            }
        }
    }
    out
}

/// A species' fight against a gear age at level 0, for the sheet.
pub fn cell(s: &Species, level: u32, age: Age) -> Option<combat::Fight> {
    danger::fight(s, level, age, 1.0)
}

/// Its tweaks, for the sheet.
pub fn tweak_text(species: &str) -> String {
    danger::danger(species)
        .map(|d| {
            d.tweaks
                .iter()
                .map(|(stat, t)| format!("{} ×{}: {}", stat.name(), t.x, t.why))
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default()
}

/// The stat a tweak moves, for the sheet's column names.
pub fn stat_name(s: Stat) -> &'static str {
    s.name()
}
