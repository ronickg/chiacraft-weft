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
    // Base's hostile mobs, on the same curve (`vitals::mobs`); armour pays off against them.
    for m in vitals::mobs::MOBS {
        same_age(m.size(), m.role, m.level(), m.stats(), m.name, &mut out);
        let hits = vitals::mobs::armoured_hits(m);
        // Only for mobs priced against a fresh player: priced against the reference (option B)
        // they take the band's hits from it by design.
        if vitals::mobs::MEETS == vitals::mobs::Meets::Fresh && hits < vitals::mobs::ARMOURED_HITS {
            out.push(format!(
                "{}: the {} reference in its armour goes down in {hits:.1} of its hits, under {}",
                m.name,
                m.home.name(),
                vitals::mobs::ARMOURED_HITS
            ));
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
    for m in vitals::mobs::MOBS {
        for (stat, t) in m.tweaks {
            if !t.ok() {
                out.push(format!("{}'s {stat:?} tweak ×{} (\"{}\") is out of bounds or gives no reason", m.name, t.x, t.why));
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

/// Gear's grade and its gems together stay under the curve's ceiling (`curve::GRADE_CEILING`,
/// `notes/PROGRESSION.md` "Endless gear": "grade plus marks add at most +60%"): every kind at
/// every grade, its sockets full of the best gem roll there is (a whole band on each stat it takes).
pub fn gem_ceiling() -> Vec<String> {
    let mut out = Vec::new();
    for k in crafting::KINDS {
        for g in crafting::GRADES {
            let best = vitals::sockets::best_units() * 3.0;
            let boosts: Vec<(vitals::web::Family, f32)> = vitals::sockets::GEAR_FAMILIES
                .iter()
                .map(|f| (*f, best))
                .collect();
            for (i, m) in crafting::gems::multipliers(k, g.n, &boosts)
                .iter()
                .enumerate()
            {
                if g.mult * m > vitals::curve::GRADE_CEILING + 1e-4 {
                    out.push(format!("{} at grade {}: stat {i} x{:.2}", k.id, g.n, g.mult * m));
                }
            }
        }
    }
    out
}

/// Most of a fight a creature may spend knocked down (`vitals::combat::KNOCK_*`).
pub const DOWN_SHARE: f32 = 1.0 / 3.0;

/// Stagger on every size that has a floor, at every age's levels: a same-age fight reaches every
/// break, and knockdowns (their rest between) keep it down at most `DOWN_SHARE` of a fight, same
/// age or over-tier at grade 4 and at the ceiling.
pub fn stagger() -> Vec<String> {
    let mut out = Vec::new();
    for size in SIZES.into_iter().filter(|&s| combat::staggers(s)) {
        for role in ROLES {
            for age in AGES {
                for depth in [0, 5, 9] {
                    let level = age.start() + depth as f32;
                    let them = combat::derive(size, role, level);
                    let h = them.health;
                    let mut fights = vec![("same-age".to_string(), combat::fight(Player { level, times: 1.0 }, size, level, them))];
                    for gear in AGES.iter().filter(|a| a.tier() > age.tier()) {
                        for times in [1.0, GRADE_CEILING] {
                            fights.push((
                                format!("{} gear ×{times}", gear.name()),
                                combat::fight(Player { level: gear.reference(), times }, size, level, them),
                            ));
                        }
                    }
                    for (i, (who, f)) in fights.iter().enumerate() {
                        // What it would have lost without the floor, over the fight: the overflow.
                        let over = h * (f.ttk / f.raw.max(1e-6) - 1.0).max(0.0);
                        let stagger = combat::stagger_gain(h, over);
                        if i == 0 && combat::breaks_at(stagger, h) < combat::BREAKS.len() {
                            out.push(format!(
                                "a {} {} (level {level:.0}): a same-age fight gathers {:.0}% of its health in stagger, short of its last break",
                                size.name(),
                                role.name(),
                                100.0 * stagger / h
                            ));
                        }
                        let knocks = (stagger / (combat::KNOCK_AT * h)).floor();
                        let room = 1.0
                            + ((f.ttk - combat::KNOCK_SECS) / (combat::KNOCK_SECS + combat::KNOCK_REST))
                                .max(0.0)
                                .floor();
                        let down = knocks.min(room) * combat::KNOCK_SECS;
                        if down > DOWN_SHARE * f.ttk {
                            out.push(format!(
                                "a {} {} (level {level:.0}), {who}: down {down:.0} s of a {:.1} s fight",
                                size.name(),
                                role.name(),
                                f.ttk
                            ));
                        }
                    }
                }
            }
        }
    }
    out
}

/// Bows on the curve (`vitals::gear`, "bows and arrows"): a full draw takes the bow's spread's
/// seconds, a plain arrow sits at the Stone Age's reference, the best stave's spring is the
/// reference bow's (a Sound yew bow has times 1), and a reference bow's arrows at every age deal
/// the reference's damage a second, as its sword does.
pub fn ranged() -> Vec<String> {
    use vitals::gear::{ARROW_LEVEL, FULL_DRAW_SECS, arrow_hit, draw_power, spread};
    let mut out = Vec::new();
    let Some(bow) = spread("bow") else {
        return vec!["no bow spread in `vitals::gear::SPREADS`".into()];
    };
    if (bow.speed * FULL_DRAW_SECS - 1.0).abs() > 1e-4 || (draw_power(FULL_DRAW_SECS) - 1.0).abs() > 1e-4 {
        out.push(format!(
            "a full draw takes {FULL_DRAW_SECS} s (power {:.2}), but the bow's spread shoots {} a second",
            draw_power(FULL_DRAW_SECS),
            bow.speed
        ));
    }
    if (ARROW_LEVEL - Age::Stone.reference()).abs() > 1e-4 {
        out.push(format!("a plain arrow is item level {ARROW_LEVEL}, not the Stone Age's reference {}", Age::Stone.reference()));
    }
    let best = crafting::materials::SPRING
        .iter()
        .map(|s| s.1)
        .fold(0.0_f32, f32::max);
    if (best - 1.0).abs() > 1e-4 {
        out.push(format!("the springiest stave is ×{best}, not the reference bow's ×1"));
    }
    // Arrows of other heads: every material that makes an arrow's head is an arrowhead, and every
    // age with working materials has an arrowhead of its own, so a bow keeps up age by age.
    use vitals::gear::{ARROWHEADS, arrow_level};
    let heads: Vec<&str> = crafting::MATERIALS
        .iter()
        .filter(|m| crafting::materials::fits(m, "arrow", "head", crafting::Role::Working))
        .map(|m| m.id)
        .collect();
    for m in &heads {
        if !ARROWHEADS.contains(m) {
            out.push(format!("{m} makes an arrow's head but isn't in vitals::gear::ARROWHEADS"));
        }
    }
    for m in ARROWHEADS {
        if !heads.contains(m) {
            out.push(format!("{m} is an arrowhead but makes no arrow's head"));
        }
        if vitals::gear::material(m).is_none() {
            out.push(format!("arrowhead {m} has no item level in vitals::gear"));
        }
    }
    if (arrow_level(0) - ARROW_LEVEL).abs() > 1e-4 {
        out.push(format!("a flint arrow is item level {}, not a plain arrow's {ARROW_LEVEL}", arrow_level(0)));
    }
    for age in AGES {
        let made = crafting::MATERIALS
            .iter()
            .filter(|m| m.roles.contains(&crafting::Role::Working))
            .filter_map(|m| vitals::gear::material(m.id))
            .any(|l| age_of(l.level()) == age);
        let headed = (0..ARROWHEADS.len() as u32).any(|h| age_of(arrow_level(h)) == age);
        if made && !headed {
            out.push(format!("{}: working materials but no arrowhead of the age", age.name()));
        }
    }
    for age in AGES {
        let p = Player::of(age);
        let dps = arrow_hit(age.reference(), 1.0) / FULL_DRAW_SECS;
        let want = p.hit() * combat::REF_RATE * bow.share;
        if (dps / want - 1.0).abs() > 0.01 {
            out.push(format!("{}: the reference bow's arrows deal {dps:.2} a second, not the reference's {want:.2}", age.name()));
        }
    }
    out
}

/// The ages' lang keys (`age.base.<key>` in base's lang), tier by tier.
const AGE_KEYS: [&str; 7] = ["stone", "bronze", "iron", "steam", "lightning", "future", "stars"];

/// Saddles lift tames (Planner 11-23, option A: a saddle's fitting sets its level): the plain one
/// is leather at the Stone reference; each fitting is a crafting material with a fitting's role,
/// no lower than the one before; the game's lift (`vitals::gear::saddle_lift` on health, hit and
/// armour) is what the checks' tame model gives (`danger::tame_with`); a saddle never carries a
/// tame past it; and the saddle's tooltip line names each fitting's material, age and level.
pub fn saddles() -> Vec<String> {
    use vitals::gear::{SADDLE_FITTINGS, SADDLE_PLAIN_LEVEL, saddle_level, saddle_lift};
    let mut out = Vec::new();
    if SADDLE_FITTINGS.first() != Some(&"leather") || (saddle_level(0) - Age::Stone.reference()).abs() > 1e-4 {
        out.push(format!("the plain saddle isn't leather at the Stone reference (level {})", saddle_level(0)));
    }
    if (SADDLE_PLAIN_LEVEL - Age::Stone.reference()).abs() > 1e-4 {
        out.push(format!("a plain saddle is level {SADDLE_PLAIN_LEVEL}, not the Stone reference"));
    }
    let lang = include_str!("../../assets/lang/en_us.lang");
    for (i, m) in SADDLE_FITTINGS.iter().enumerate() {
        let fit = i as u32;
        match crafting::materials::material(m) {
            Some(mat) if mat.roles.contains(&crafting::Role::Fitting) => {}
            _ => out.push(format!("saddle fitting {m} is no crafting material with a fitting's role")),
        }
        if i > 0 && saddle_level(fit) + 1e-4 < saddle_level(fit - 1) {
            out.push(format!("saddle fitting {m} (level {}) is below the one before it", saddle_level(fit)));
        }
        let level = saddle_level(fit);
        let line = format!(
            "{{if:value=={i}}}{{lang:material.base.{m}}} fittings · {{lang:age.base.{}}} · item level {level:.0}{{/if}}",
            AGE_KEYS[age_of(level).tier() as usize]
        );
        if !lang.contains(&line) {
            out.push(format!("base's en_us tagged.base.saddle_fit has no `{line}`"));
        }
    }
    for s in SPECIES.iter() {
        let Some(d) = danger::danger(s.name) else { continue };
        for level in [0, 100, 200] {
            let own = d.area_level(level);
            let Some(plain) = danger::tame_with(s, level, None, 1.0) else { continue };
            for fit in 0..SADDLE_FITTINGS.len() as u32 {
                let at = saddle_level(fit);
                let Some(on) = danger::tame_with(s, level, Some(at), 1.0) else { continue };
                let lift = saddle_lift(at, own);
                for (stat, a, b) in [("health", on.health, plain.health), ("hit", on.hit, plain.hit), ("armour", on.armour, plain.armour)] {
                    if b > 1e-6 && ((a / b) / lift - 1.0).abs() > 0.02 {
                        out.push(format!(
                            "a {} tame (level {level}) on a {} saddle: the game lifts its {stat} ×{lift:.3}, the model ×{:.3}",
                            s.name,
                            SADDLE_FITTINGS[fit as usize],
                            a / b
                        ));
                    }
                }
                let Some(fights_at) = danger::tame_level(s, level, None).map(|o| o.max(at)) else { continue };
                if fights_at > own.max(at) + 1e-4 {
                    out.push(format!("a {} tame on a {} saddle fights past it", s.name, SADDLE_FITTINGS[fit as usize]));
                }
            }
        }
    }
    out
}
