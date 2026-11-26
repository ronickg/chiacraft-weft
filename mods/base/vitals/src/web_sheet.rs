//! The skills and the perk web as one page (`mods/base/vitals/WEB_SHEET.md`), written from this
//! crate so it can't drift: `cargo run -p vitals --bin web_sheet > mods/base/vitals/WEB_SHEET.md`.
//! A test fails while it's stale.

use std::fmt::Write;

use crate::curve::{AGES, Age};
use crate::perks::{self, CHECK_POINTS, CHECK_SKILL, GATES, JOBS, REFUNDS_PER_ITEM, REGIONS};
use crate::skills::{self, SKILLS, gain};
use crate::web::{self, FAMILIES, Node as Kind};

pub fn markdown() -> String {
    let mut s = String::new();
    let w = &mut s;
    let _ = writeln!(w, "# Skills and the perk web\n");
    let _ = writeln!(
        w,
        "Written from `mods/base/vitals` (`cargo run -p vitals --bin web_sheet > mods/base/vitals/WEB_SHEET.md`); do not edit by hand. The design: `notes/PROGRESSION.md`, \"Skills and perks (like Skyrim)\"; the prices: `reports/Balance model.md` 5.2.\n"
    );

    let _ = writeln!(w, "## Skills\n");
    let _ = writeln!(
        w,
        "A skill runs 0-{}, {:.1} levels an age. Its levels cost effort minutes (EM): {:.0} % of each age's hours for that age's levels, so a main skill reaches 100 in {:.1} hours of doing it. Work below your skill teaches half as much every {:.1} levels below (one age below: a quarter). Each skill-up gives the player {} XP times the new level (Skyrim's), toward the character level; each character level gives {} perk point.\n",
        skills::MAX,
        skills::PER_AGE,
        skills::SHARE * 100.0,
        skills::em_to(skills::MAX as f32) / 60.0,
        skills::HALVING,
        skills::UP_XP,
        web::POINTS_PER_LEVEL
    );
    let _ = writeln!(w, "| Skill | Group | Path | Raised by |");
    let _ = writeln!(w, "|---|---|---|---|");
    for sk in SKILLS {
        let by = if sk.by().is_empty() { "nothing yet (data for later)" } else { sk.by() };
        let _ = writeln!(w, "| {} | {} | {} | {} |", sk.name(), sk.group().name(), sk.path().name(), by);
    }
    let _ = writeln!(w, "\n### What each age's levels cost\n");
    let _ = writeln!(w, "| Age | Skill levels | EM per level | EM from 0 | Hours from 0 |");
    let _ = writeln!(w, "|---|---|---|---|---|");
    for (i, a) in AGES.iter().enumerate() {
        let (lo, hi) = (i as f32 * skills::PER_AGE, (i + 1) as f32 * skills::PER_AGE);
        let _ = writeln!(
            w,
            "| {} | {:.0}-{:.0} | {:.1} | {:.0} | {:.1} |",
            a.name(),
            lo,
            hi,
            (skills::em_to(hi) - skills::em_to(lo)) / skills::PER_AGE,
            skills::em_to(hi),
            skills::em_to(hi) / 60.0
        );
    }
    let _ = writeln!(w, "\n### What doing pays (EM, before tier against skill)\n");
    let stone = skills::em_to(skills::PER_AGE);
    let rows: [(&str, f32); 9] = [
        ("a stone block mined (hardness 1.5)", gain::mine(1.5)),
        ("an iron ore mined (hardness 3)", gain::ore(3.0)),
        ("a reference hit landed (Stone Age)", gain::hit(crate::combat::Player::of(Age::Stone).hit(), Age::Stone)),
        ("a reference hit taken (Stone Age)", gain::taken(crate::combat::Player::of(Age::Stone).hit(), Age::Stone)),
        ("a tool part of 3 inputs made", gain::craft(3)),
        ("a food cooked", gain::cook(1)),
        ("a block placed", gain::PLACE),
        ("a ripe crop harvested", gain::HARVEST),
        ("a large animal tamed", gain::tame(gain::TAME_MINUTES[2])),
    ];
    let _ = writeln!(w, "| Doing | EM | To the Stone Age's top level ({:.0} EM), at full rate |", stone);
    let _ = writeln!(w, "|---|---|---|");
    for (what, em) in rows {
        let _ = writeln!(w, "| {what} | {em:.3} | {:.0} |", stone / em);
    }
    let _ = writeln!(
        w,
        "\nAt full rate means work at or above one's skill: Stone Age work at skill 7 teaches {:.0} %, at 14 {:.0} %.\n",
        skills::against(skills::for_age(Age::Stone), 7.0) * 100.0,
        skills::against(skills::for_age(Age::Stone), 14.0) * 100.0
    );

    let _ = writeln!(w, "## The web\n");
    let web = perks::web();
    let nodes = web.nodes.len() - 1;
    let count = |k: Kind| {
        web.nodes
            .iter()
            .filter(|n| n.id != perks::START && n.kind == k)
            .count()
    };
    let _ = writeln!(
        w,
        "{nodes} nodes ({} small, {} notables, {} keystones, {} sockets) in {} regions around a free start, {} links. A small node gives {:.0} % to its family, a notable 3-4 times that, a keystone {:.0} % to one family for as much off another. A family tops out at ×{:.1} (one age), a keystone's cost at ×{:.1}. A rare item refunds {} nodes, each one the rest don't hang from.\n",
        count(Kind::Small),
        count(Kind::Notable),
        count(Kind::Keystone),
        count(Kind::Socket),
        REGIONS.len(),
        web.links.len(),
        web::UNIT * 100.0,
        web::KEYSTONE_UNITS * web::UNIT * 100.0,
        web::FAMILY_CEILING,
        web::FAMILY_FLOOR,
        REFUNDS_PER_ITEM
    );
    let _ = writeln!(w, "Skill level each ring needs (rings 9-12 are room for later ages at the rim):\n");
    let _ = writeln!(
        w,
        "| Ring | {} |",
        (1..GATES.len())
            .map(|r| r.to_string())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let _ = writeln!(w, "|---|{}", "---|".repeat(GATES.len() - 1));
    let _ = writeln!(
        w,
        "| Skill | {} |\n",
        GATES[1..]
            .iter()
            .map(|g| g.to_string())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let _ = writeln!(w, "### Regions\n");
    let _ = writeln!(w, "| Region | Left lane | Right lane | Notables (ring 4, ring 7) | Keystone | Units | Bridge to the next |");
    let _ = writeln!(w, "|---|---|---|---|---|---|---|");
    for r in &REGIONS {
        let ks = r.named[4].keystone.unwrap();
        let notables: Vec<String> = r.named[..4]
            .iter()
            .map(|n| {
                let gives: Vec<String> = n
                    .gives
                    .iter()
                    .map(|g| format!("+{:.0} {}", g.1, g.0.name()))
                    .collect();
                let sp = n.special.map_or(String::new(), |s| format!(", {}", s.id()));
                format!("{} ({}{sp})", n.key.split('.').nth(1).unwrap_or(n.key), gives.join(", "))
            })
            .collect();
        let bridge: Vec<String> = r
            .bridge
            .gives
            .iter()
            .map(|g| format!("+{:.0} {}", g.1, g.0.name()))
            .collect();
        let _ = writeln!(
            w,
            "| {} | {} | {} | {} | {} (+{:.0} {}, -{:.0} {}) | {:.0} | {} ({}) |",
            r.skill.name(),
            r.a.name(),
            r.b.name(),
            notables.join("; "),
            r.named[4].key.split('.').nth(1).unwrap_or(""),
            web::KEYSTONE_UNITS,
            ks.0.name(),
            web::KEYSTONE_UNITS,
            ks.1.name(),
            perks::region_units(r.skill),
            r.bridge.key.split('.').nth(1).unwrap_or(""),
            bridge.join(", ")
        );
    }
    let _ = writeln!(
        w,
        "\nA region's budget is {:.0} units (one age in its speciality); keystones count as nothing, their power is their cost.\n",
        web::units_to_ceiling()
    );

    let _ = writeln!(w, "### Families\n");
    let _ = writeln!(w, "| Family | Units in the web | In the game |");
    let _ = writeln!(w, "|---|---|---|");
    for f in FAMILIES {
        let u: f32 = web
            .nodes
            .iter()
            .flat_map(|n| n.gives.iter())
            .filter(|g| g.0 == f && g.1 > 0.0)
            .map(|g| g.1)
            .sum();
        let _ = writeln!(w, "| {} | {u:.0} | {} |", f.name(), if f.in_game() { "yes" } else { "later" });
    }

    sockets_section(w);

    let _ = writeln!(w, "\n## Checks\n");
    let p = perks::problems();
    if p.is_empty() {
        let _ = writeln!(
            w,
            "Every node reachable from the start without passing a deeper gate, every price and region budget kept, no dominant build: no problems.\n"
        );
    } else {
        for x in &p {
            let _ = writeln!(w, "- **{x}**");
        }
        let _ = writeln!(w);
    }
    let d = perks::dominance();
    let _ = writeln!(w, "### Build dominance\n");
    let _ = writeln!(
        w,
        "Each job's build is hill-climbed with {CHECK_POINTS} points at skill {CHECK_SKILL} everywhere, then scored on every job (the weighted log of what its families multiply, shown as ×). A build may not win every job by over 15 %, and no node may be in over 90 % of the builds.\n"
    );
    let _ = writeln!(w, "| Build (points spent) | {} |", JOBS.iter().map(|j| j.name).collect::<Vec<_>>().join(" | "));
    let _ = writeln!(w, "|---|{}", "---|".repeat(JOBS.len()));
    for (i, (b, sc)) in d.builds.iter().enumerate() {
        let _ = writeln!(
            w,
            "| {} ({}) | {} |",
            JOBS[i].name,
            perks::count(b),
            sc.iter()
                .map(|x| format!("×{:.2}", x.exp()))
                .collect::<Vec<_>>()
                .join(" | ")
        );
    }
    let _ = writeln!(w, "\nMost picked nodes:\n");
    let _ = writeln!(w, "| Node | Builds |");
    let _ = writeln!(w, "|---|---|");
    for &(id, c) in d.picks.iter().take(8) {
        let n = web.node(id).unwrap();
        let _ = writeln!(w, "| {} {} (ring {}) | {c} of {} |", n.region.map_or("", |r| r.id()), n.key, n.ring, d.builds.len());
    }
    s
}

fn sockets_section(w: &mut String) {
    use crate::sockets::{self, KINDS, SOURCES};
    let _ = writeln!(
        w,
        "\n### Sockets and stones\n\nRonald, 11-05: \"insert items which generate boosts and you find those items in the world, relics or whatever\". A socket costs {} points (at one, the dominance check found sockets in every build); four sit near the centre (ring 2.6), one deep in every region (5.6) and one past every second bridge (7.3). Stones are found, never made, and swap freely, so the checks assume the best stone for the job in every socket taken: a top-level mind-stone, {:.0} units in one family.\n",
        perks::SOCKET_POINTS,
        sockets::best_units()
    );
    let _ = writeln!(w, "| Stone | What it does |");
    let _ = writeln!(w, "|---|---|");
    for k in KINDS {
        let what = match k {
            sockets::Kind::Mind => {
                "1-3 boosts to families, its band split among them, each rolled 60-100 % of its share in half units".to_string()
            }
            sockets::Kind::Flow => format!(
                "the small nodes taken within {:.0} px of its socket: +{:.0}-{:.0} % of their units (two in three), or turned to one family",
                sockets::RADIUS,
                sockets::AMPLIFY_MIN * 100.0,
                sockets::AMPLIFY_MAX * 100.0
            ),
            sockets::Kind::Unique(sockets::Unique::Remaru) => format!("ley sight, and +{:.0} units of mana cost", sockets::REMARU_UNITS),
            sockets::Kind::Unique(sockets::Unique::Emakaun) => {
                format!("+{} units of health for every node taken in its reach", sockets::LIFEKEEP_PER_NODE)
            }
        };
        let _ = writeln!(w, "| {} | {what} |", k.id());
    }
    let _ = writeln!(w, "\nA mind-stone's band by the age it's found in (units at the age's middle):\n");
    let _ = writeln!(
        w,
        "| {} |",
        AGES.iter()
            .map(|a| a.name())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let _ = writeln!(w, "|{}", "---|".repeat(AGES.len()));
    let _ = writeln!(
        w,
        "| {} |\n",
        AGES.iter()
            .map(|a| format!("{:.1}", sockets::band(a.reference())))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let _ = writeln!(w, "| Found at | Mind | Flow | Unique | Depth | Chance each time |");
    let _ = writeln!(w, "|---|---|---|---|---|---|");
    for f in SOURCES {
        let _ = writeln!(
            w,
            "| {} | {:.0} % | {:.0} % | {:.0} % | {} | {:.0} % |",
            f.source.id(),
            f.odds[0] * 100.0,
            f.odds[1] * 100.0,
            f.odds[2] * 100.0,
            f.depth,
            f.chance * 100.0
        );
    }
}
