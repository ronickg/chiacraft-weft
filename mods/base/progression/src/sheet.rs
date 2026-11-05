//! The progression and balance page (`mods/base/progression/PROGRESSION_SHEET.md`), written from
//! the code so it can't drift: `cargo run -p progression --bin sheet >
//! mods/base/progression/PROGRESSION_SHEET.md`. A test fails while it's stale.

use std::fmt::Write;

use animals_core::danger;
use animals_core::species::SPECIES;
use vitals::combat::{self, Player, ROLES, SIZES, Size};
use vitals::curve::{self, AGES, Age, PATHS};
use vitals::web;

use crate::content::{self, Get, Kind};
use crate::graph;

pub fn markdown() -> String {
    let mut w = String::new();
    w.push_str("# Progression and balance sheet\n\n");
    w.push_str(
        "Written from `mods/base/progression` (`cargo run -p progression --bin sheet > mods/base/progression/PROGRESSION_SHEET.md`); do not edit by hand. The plan: `notes/PROGRESSION.md`; the method: `reports/Balance model.md`; the numbers: `vitals::curve`, `vitals::combat`, `vitals::web`, `animals_core::danger`. Nothing in the game reads the curves yet: the tables show today's numbers next to what the curves would set.\n\n",
    );
    let g = graph::solve();
    checks(&mut w);
    ages(&mut w, &g);
    sizes(&mut w);
    over_tier(&mut w);
    gaps(&mut w);
    species_matrix(&mut w);
    weapons(&mut w);
    tames(&mut w);
    web_prices(&mut w);
    keys(&mut w);
    tags(&mut w, &g);
    materials(&mut w);
    w
}

fn checks(w: &mut String) {
    w.push_str("## Checks\n\n| check | result |\n|---|---|\n");
    let all = crate::checks();
    for (name, problems) in &all {
        let r = if problems.is_empty() { "passes".to_string() } else { format!("**{} problems**", problems.len()) };
        let _ = writeln!(w, "| {name} | {r} |");
    }
    for (name, problems) in &all {
        if !problems.is_empty() {
            let _ = writeln!(w, "\n**{name}:**\n");
            for p in problems.iter().take(40) {
                let _ = writeln!(w, "- {p}");
            }
            if problems.len() > 40 {
                let _ = writeln!(w, "- and {} more", problems.len() - 40);
            }
        }
    }
    w.push('\n');
}

fn ages(w: &mut String, g: &graph::Solved) {
    let _ = writeln!(
        w,
        "## The ages\n\nPower grows ×{} an age: `g(L) = {}^(L/10)` over area and item level L (tier × 10 + a depth of 0-9). The currency is effort minutes (EM). The reference player is same-age gear at grade 4, measured at the age's middle (depth {}). The kit is what an average player makes in the age the first time (`graph::KITS`), gathered and crafted the cheapest way, against its budget: {:.0} % of the age's EM (`curve::MAKING_SHARE`). Everything once is one of every made thing tagged to the age, alternatives and all, each from scratch.\n",
        curve::GROWTH,
        curve::GROWTH,
        curve::REFERENCE_DEPTH,
        curve::MAKING_SHARE * 100.0
    );
    w.push_str("| tier | age | hours | EM | g at start | reference: hit / health / armour | things tagged | made | kit EM | of budget | everything once |\n|---|---|---|---|---|---|---|---|---|---|---|\n");
    let mut total = 0.0;
    for a in AGES {
        let p = Player::of(a);
        let tagged = g
            .ids
            .iter()
            .filter(|id| content::tier(id) == Some(a.tier()))
            .count();
        let (made, once) = graph::making(g, a);
        let (em, _) = graph::kit(g, a);
        let cap = a.em() * curve::MAKING_SHARE;
        total += a.hours();
        let _ = writeln!(
            w,
            "| {} | {} | {} | {:.0} | {:.2} | {:.1} / {:.0} / {:.0} | {tagged} | {made} | {em:.0} | {:.0} % | {:.0} EM |",
            a.tier(),
            a.name(),
            a.hours(),
            a.em(),
            curve::g(a.start()),
            p.hit(),
            p.health(),
            p.armour(),
            em / cap * 100.0,
            once.max(0.0)
        );
    }
    let _ = writeln!(w, "\n{total:.0} hours in all.\n");
}

fn sizes(w: &mut String) {
    w.push_str("## Fight bands and derived stats\n\nFor a same-age reference player (Ronald, 11-06: \"as proposed\"). Health so the reference kills it in the band's middle, armour so it cuts the reference's hit by its share (`armour / (armour + 5 × hit)`, capped at 80 %), a hit so it downs the reference in the band's hits. The cap is how many levels over it a player's gear counts at most; the floor the least seconds it dies in, whatever the gear (damage over it goes to stagger and part breaks).\n\n");
    w.push_str("| size | mass | TTK band (mid) | hits to down | hit every | armour cut | gear cap | floor | knockback resist |\n|---|---|---|---|---|---|---|---|---|\n");
    for s in SIZES {
        let b = s.band();
        let mass = match s {
            Size::Small => "under 10 kg",
            Size::Medium => "10-150 kg",
            Size::Large => "150-2,000 kg",
            Size::Huge => "2-20 t",
            Size::Titan => "over 20 t, or a boss",
        };
        let _ = writeln!(
            w,
            "| {} | {mass} | {}-{} s ({}) | {} | {} s | {:.0} % | {} | {} | {} |",
            s.name(),
            b.ttk.0,
            b.ttk.1,
            b.mid,
            b.hits,
            b.interval,
            b.cut * 100.0,
            b.cap.map_or("none".into(), |c| format!("+{c}")),
            b.floor.map_or("none".into(), |f| format!("{f} s")),
            b.knockback
        );
    }
    w.push_str("\nRoles shape the budget: ");
    let roles: Vec<String> = ROLES
        .iter()
        .map(|r| {
            let (h, x, a) = r.times();
            format!("{} health ×{h}, hit ×{x}, armour ×{a}", r.name())
        })
        .collect();
    let _ = writeln!(w, "{}.\n", roles.join("; "));
    w.push_str("A size's own health / armour / hit (before its role) at each age's reference level, engine points (the vitals show health ×5):\n\n| size |");
    for a in AGES {
        let _ = write!(w, " {} |", a.name());
    }
    w.push_str("\n|---|");
    for _ in AGES {
        w.push_str("---|");
    }
    w.push('\n');
    for s in SIZES {
        let _ = write!(w, "| {} |", s.name());
        for a in AGES {
            let st = combat::derive_base(s, a.reference());
            let _ = write!(w, " {:.0} / {:.0} / {:.0} |", st.health, st.armour, st.hit);
        }
        w.push('\n');
    }
    w.push('\n');
}

fn over_tier(w: &mut String) {
    w.push_str("## Over-tier and under-tier\n\nSeconds to kill and the creature's hits to down the player, for a size's grazer at the Iron Age's reference level, by how many ages the player's gear is over or under it (grade 4). Over-tier gear counts only up to the size's cap, and the floor holds.\n\n| size | -1 age | same age | +1 | +2 | +3 | +4 |\n|---|---|---|---|---|---|---|\n");
    let at = Age::Iron.reference();
    for s in SIZES {
        let them = combat::derive(s, combat::Role::Grazer, at);
        let _ = write!(w, "| {} |", s.name());
        for d in [-1.0f32, 0.0, 1.0, 2.0, 3.0, 4.0] {
            let f = combat::fight(Player { level: at + d * curve::LEVELS_PER_AGE, times: 1.0 }, s, at, them);
            let _ = write!(w, " {:.1} s / {:.1} |", f.ttk, f.hits);
        }
        w.push('\n');
    }
    w.push('\n');
}

fn gaps(w: &mut String) {
    let _ = writeln!(
        w,
        "## Species\n\nEvery wild animal rolls a level of {}-{} anywhere, and a level-{} one is {}× a level-0 one of its species (Ronald, 11-06): a wild point in health or damage adds {:.2} of the base. Its size class comes from its mass (small ones scale inside their class by the cube root of mass against {} kg, ×{}-×{}: a fox beats a mouse), its role and tweaks from `animals_core::danger`. The game registers each kind with these level-0 numbers (11-07). A Stone Age species reaches the Bronze Age's band at level {}, the Iron Age's at {}, Steam's at {}.\n",
        danger::LEVELS.0,
        danger::LEVELS.1,
        danger::LEVELS.1,
        danger::TOP_TIMES,
        danger::LEVEL_STEP,
        danger::SMALL_MASS,
        danger::SMALL_TIMES.0,
        danger::SMALL_TIMES.1,
        danger::level_reaching(Age::Stone, Age::Bronze).unwrap_or(0),
        danger::level_reaching(Age::Stone, Age::Iron).unwrap_or(0),
        danger::level_reaching(Age::Stone, Age::Steam).unwrap_or(0),
    );
    w.push_str("| species | kg | size | role | mass × | health / armour / hit (level 0) | health at 100 / 200 | tweaks |\n|---|---|---|---|---|---|---|---|\n");
    for s in SPECIES.iter() {
        let Some(d) = danger::danger(s.name) else { continue };
        let (Some(st), Some(mid), Some(top)) = (danger::stats(s, 0), danger::stats(s, 100), danger::stats(s, 200)) else { continue };
        let _ = writeln!(
            w,
            "| {} | {} | {} | {} | {:.2} | {:.1} / {:.1} / {:.1} | {:.0} / {:.0} | {} |",
            s.name,
            s.bio.mass,
            danger::size(s).name(),
            d.role.name(),
            danger::mass_times(s),
            st.health,
            st.armour,
            st.hit,
            mid.health,
            top.health,
            crate::matrix::tweak_text(s.name)
        );
    }
    w.push('\n');
}

fn species_matrix(w: &mut String) {
    w.push_str("## The combat matrix\n\nEach species at level 0 (the Stone Age's start) against a reference player of each age, grade 4: seconds to kill / its hits to down the player. The checks run it at every tenth level, at grade 4 and at the +60 % ceiling.\n\n| species |");
    for a in AGES {
        let _ = write!(w, " {} |", a.name());
    }
    w.push_str("\n|---|");
    for _ in AGES {
        w.push_str("---|");
    }
    w.push('\n');
    for s in SPECIES.iter() {
        let _ = write!(w, "| {} |", s.name);
        for a in AGES {
            match crate::matrix::cell(s, 0, a) {
                Some(f) => {
                    let _ = write!(w, " {:.1} / {:.0} |", f.ttk, f.hits);
                }
                None => w.push_str(" - |"),
            }
        }
        w.push('\n');
    }
    w.push('\n');
}

fn weapons(w: &mut String) {
    w.push_str("## Weapons, tools and armour\n\nFrom the curve (`vitals::gear`, Ronald, 11-07: \"Yes, from the curves\"): a kind swings at its speed and deals its share of the reference's damage a second at its material's item level (tier × 10 + depth). A sword of the age is the reference. A full armour set of an age gives the reference's armour and lifts health to the reference's (20 engine points at its base).\n\n| kind | swings a second | share |\n|---|---|---|\n");
    for sp in vitals::gear::SPREADS {
        let _ = writeln!(w, "| {} | {} | {} |", sp.kind, sp.speed, sp.share);
    }
    w.push_str("\nOne hit's damage (engine points) by material:\n\n| material | age | level |");
    let kinds = ["sword", "axe", "spear", "knife", "pickaxe", "shovel", "hoe"];
    for k in kinds {
        let _ = write!(w, " {k} |");
    }
    w.push_str(" full armour set: armour / health |\n|---|---|---|");
    for _ in kinds {
        w.push_str("---|");
    }
    w.push_str("---|\n");
    for m in vitals::gear::MATERIALS {
        let _ = write!(w, "| {} | {} | {:.0} |", m.id, Age::of_tier(m.tier).name(), m.level());
        for k in kinds {
            let _ = write!(w, " {:.1} |", vitals::gear::weapon(k, m.level()).map_or(0.0, |x| x.0));
        }
        let (a, v) = (0..4)
            .map(|slot| vitals::gear::armour(slot, m.level()))
            .fold((0.0, 0.0), |t, x| (t.0 + x.0, t.1 + x.1));
        let _ = writeln!(w, " {a:.0} / {:.0} |", combat::REF_HEALTH + v);
    }
    w.push('\n');
}

fn tames(w: &mut String) {
    let _ = writeln!(
        w,
        "## Tames by saddle\n\nA saddle (and barding) of an age carries a tame up to that age's reference level, never past it and never below where its own level puts it (Ronald, 11-06). Its own bonuses (tamed points, imprint ×{}, bond ×{}) add at most ×{}, as gear's grade and marks do. A level-0 utahraptor and wyvern, health / hit, plain and at the ceiling:\n",
        1.0 + danger::IMPRINT,
        1.0 + danger::BOND,
        danger::TAME_CEILING
    );
    w.push_str("| species | no saddle |");
    for a in AGES {
        let _ = write!(w, " {} saddle |", a.name());
    }
    w.push_str("\n|---|---|");
    for _ in AGES {
        w.push_str("---|");
    }
    w.push('\n');
    for name in ["utahraptor", "wyvern"] {
        let Some(s) = SPECIES.iter().find(|s| s.name == name) else { continue };
        let cell = |saddle: Option<Age>| -> String {
            match (danger::tame(s, 0, saddle, 1.0), danger::tame(s, 0, saddle, danger::TAME_CEILING)) {
                (Some(p), Some(t)) => format!("{:.0} / {:.0} (top {:.0} / {:.0})", p.health, p.hit, t.health, t.hit),
                _ => "-".into(),
            }
        };
        let _ = write!(w, "| {name} | {} |", cell(None));
        for a in AGES {
            let _ = write!(w, " {} |", cell(Some(a)));
        }
        w.push('\n');
    }
    w.push('\n');
}

fn web_prices(w: &mut String) {
    let _ = writeln!(
        w,
        "## The perk web's prices\n\nA unit is +{:.0} % to one family; a small node gives 1, a notable 3-4, a keystone nets 0. Inside a family bonuses add, between families they multiply. A family tops out at ×{} ({:.0} units: one age); everything together (gear, web, a tame's bond, food) at ×{} (two ages). Families: {}. The nodes, skills and their checks: `mods/base/vitals/WEB_SHEET.md`.\n",
        web::UNIT * 100.0,
        web::FAMILY_CEILING,
        web::units_to_ceiling(),
        curve::TOTAL_CEILING,
        web::FAMILIES
            .iter()
            .map(|f| f.name())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

fn keys(w: &mut String) {
    w.push_str("## Age keys by path\n\nMoving to the next age takes a key, one thing from each gate (materials and tools, a biome boss, knowledge, infrastructure), by a route for each path. What exists so far:\n\n| age | tech | magic | beasts | not built yet |\n|---|---|---|---|---|\n");
    for a in AGES {
        let routes = graph::routes(a);
        let cell = |p: curve::Path| -> String {
            match routes.iter().find(|(q, _)| *q == p).and_then(|(_, r)| *r) {
                Some(items) => items.join(", "),
                None => "-".into(),
            }
        };
        let missing = graph::KEYS
            .iter()
            .find(|k| k.age == a)
            .map_or("all of it".to_string(), |k| k.missing.join("; "));
        let _ = writeln!(
            w,
            "| {} | {} | {} | {} | {missing} |",
            a.name(),
            cell(curve::Path::Tech),
            cell(curve::Path::Magic),
            cell(curve::Path::Beasts)
        );
    }
    w.push('\n');
}

fn tags(w: &mut String, g: &graph::Solved) {
    w.push_str("## What exists, by age and path\n\nEvery named item and block, by its tag (`content::RULES`). EM is the cheapest way to have one; \"-\" not had as an item.\n\n");
    for a in AGES {
        let ids: Vec<&String> = g
            .ids
            .iter()
            .filter(|id| content::tier(id) == Some(a.tier()))
            .collect();
        if ids.is_empty() {
            continue;
        }
        let _ = writeln!(w, "### {} ({})\n", a.name(), ids.len());
        for p in PATHS {
            let mine: Vec<&&String> = ids
                .iter()
                .filter(|id| content::rule(id).is_some_and(|r| r.path == p))
                .collect();
            if mine.is_empty() {
                continue;
            }
            let mut kinds: Vec<Kind> = mine
                .iter()
                .filter_map(|id| content::rule(id).map(|r| r.kind))
                .collect();
            kinds.sort();
            kinds.dedup();
            let _ = writeln!(w, "**{}** ({})\n", p.name(), mine.len());
            for k in kinds {
                let list: Vec<String> = mine
                    .iter()
                    .filter(|id| content::rule(id).is_some_and(|r| r.kind == k))
                    .map(|id| {
                        let r = content::rule(id).map(|r| r.get);
                        match (r, g.cost.get(id.as_str())) {
                            (Some(Get::Not), _) | (_, None) => format!("{} (-)", short(id)),
                            (_, Some(c)) => format!("{} ({})", short(id), em(*c)),
                        }
                    })
                    .collect();
                let _ = writeln!(w, "- {}: {}", k.name(), list.join(", "));
            }
            w.push('\n');
        }
        let notes: Vec<String> = content::RULES
            .iter()
            .filter(|r| r.tier == a.tier() && !r.note.is_empty())
            .map(|r| format!("{}: {}", r.pats.join(", "), r.note))
            .collect();
        if !notes.is_empty() {
            w.push_str("Notes:\n");
            for n in notes {
                let _ = writeln!(w, "- {n}");
            }
            w.push('\n');
        }
    }
}

fn short(id: &str) -> &str {
    id.strip_prefix("base:").unwrap_or(id)
}

fn em(c: f32) -> String {
    if c < 1.0 {
        format!("{c:.2}")
    } else if c < 10.0 {
        format!("{c:.1}")
    } else {
        format!("{c:.0}")
    }
}

fn materials(w: &mut String) {
    w.push_str("## Tool parts' materials\n\nEach `crafting::MATERIALS` row's age, and what a part of it is made from in the game today (\"-\": nothing yet).\n\n| material | age | from |\n|---|---|---|\n");
    for m in crafting::MATERIALS {
        match content::material(m.id) {
            Some(t) => {
                let _ = writeln!(
                    w,
                    "| {} | {} | {} |",
                    m.id,
                    t.tier().map_or("untagged", |x| Age::of_tier(x).name()),
                    if t.from.is_empty() { "-" } else { t.from }
                );
            }
            None => {
                let _ = writeln!(w, "| {} | untagged | - |", m.id);
            }
        }
    }
    w.push('\n');
}
