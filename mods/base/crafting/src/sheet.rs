//! The crafting numbers on one page (`SHEET.md`, generated: `cargo run -p crafting --bin sheet >
//! mods/base/crafting/SHEET.md`).

use std::fmt::Write;

use crate::combine::{Part, combine};
use crate::grades::{GRADES, LEGENDARY_POINTS, LUCK, MACHINE_WORK, MEMORY_MAX};
use crate::kinds::{KINDS, Role, SHAPES};
use crate::materials::MATERIALS;
use crate::traits::TRAITS;

pub fn markdown() -> String {
    let mut s = String::new();
    let w = &mut s;
    let _ = writeln!(w, "# Crafting numbers\n");
    let _ = writeln!(w, "Generated from `mods/base/crafting` (`cargo run -p crafting --bin sheet`). The design is `notes/CRAFTING.md`.\n");

    let _ = writeln!(w, "## Grades\n");
    let _ = writeln!(w, "| # | Grade | × main stat and uses | Modifier slots |\n|---|---|---|---|");
    for g in GRADES {
        let _ = writeln!(w, "| {} | `{}` | {:.2} | {} |", g.n, g.key, g.mult, g.slots);
    }
    let luck: Vec<String> = LUCK
        .iter()
        .map(|(step, p)| format!("{step:+} {:.0}%", p * 100.0))
        .collect();
    let _ = writeln!(
        w,
        "\nPoints: material up to 40, work up to 40 (from memory up to {MEMORY_MAX}, a machine {MACHINE_WORK}), station up to 20, read in tens. Luck: {}. Grade 10 needs {LEGENDARY_POINTS} points.\n",
        luck.join(", ")
    );

    let _ = writeln!(w, "## Materials\n");
    let _ = writeln!(
        w,
        "| Material | Roles | Works as | Level | g/cm³ | Bulk | Harvest | Uses | Speed | As a haft | As a fitting | Trait |\n|---|---|---|---|---|---|---|---|---|---|---|---|"
    );
    for m in MATERIALS {
        let roles: Vec<&str> = m
            .roles
            .iter()
            .map(|r| match r {
                Role::Working => "working",
                Role::Holding => "holding",
                Role::Fitting => "fitting",
                Role::Extra => "extra",
                Role::Lining => "lining",
            })
            .collect();
        let haft = if m.roles.contains(&Role::Holding) {
            format!("uses {:+.0}%, speed {:+.0}%", m.hold_uses * 100.0, m.hold_speed * 100.0)
        } else {
            "-".into()
        };
        let fit = if m.fitting_uses > 0.0 { format!("{:.0} uses", m.fitting_uses) } else { "-".into() };
        let _ = writeln!(
            w,
            "| {} | {} | {:?} | {:.0} | {} | {} | {} | {} | {} | {} | {} | {} |",
            if crate::materials::PROVISIONAL.contains(&m.id) { format!("{} (provisional)", m.id) } else { m.id.to_string() },
            roles.join(", "),
            m.works,
            m.level(),
            m.density,
            m.bulk,
            if m.works == crate::materials::Works::None { "-".to_string() } else { m.harvest().to_string() },
            if m.works == crate::materials::Works::None { "-".to_string() } else { format!("{:.0}", m.uses()) },
            if m.works == crate::materials::Works::None { "-".to_string() } else { format!("{:.1}", m.speed()) },
            haft,
            fit,
            if m.trait_id.is_empty() { "-" } else { m.trait_id }
        );
    }

    let _ = writeln!(
        w,
        "\nHarvest, uses and speed are a working part's at Sound, from its material's level on the curve (`vitals::gear`: uses {} × g^{}, speed {} × g^{}, harvest {} at level 5 and {} a level), with each row's hand tweak (×0.75-1.25, reasons in `materials.rs`). Provisional rows stand in until their materials grow in the world. Diamond is a tier above steel, and the ladder stays open at the top: a new tier is a row.\n",
        vitals::gear::REF_USES,
        vitals::gear::USES_POWER,
        vitals::gear::REF_MINE,
        vitals::gear::MINE_POWER,
        vitals::gear::REF_HARVEST,
        vitals::gear::HARVEST_PER_LEVEL
    );
    let _ = writeln!(w, "## Traits\n");
    let _ = writeln!(w, "| Trait | Effect | When |\n|---|---|---|");
    for t in TRAITS {
        let _ = writeln!(w, "| {} | {:?} | {:?} |", t.id, t.effect, t.when);
    }

    let _ = writeln!(w, "\n## Kinds and shapes\n");
    for k in KINDS {
        let slots: Vec<String> = k
            .slots
            .iter()
            .map(|s| format!("{} ({:?})", s.name, s.role))
            .collect();
        let _ = writeln!(w, "- **{}** ({:?}, damage {}): {}", k.id, k.use_, k.damage, slots.join(", "));
        for s in SHAPES.iter().filter(|s| s.kind == k.id) {
            let _ = writeln!(
                w,
                "  - {} `{}`: {} cm³, stat ×{}, speed ×{}, damage ×{}, reach {:+}",
                s.slot, s.id, s.volume, s.stat, s.speed, s.damage, s.reach
            );
        }
    }

    let _ = writeln!(w, "\n## Example armour (a chestplate, leather straps, wool lining, Sound parts)\n");
    let _ = writeln!(w, "| Shell | Level | Armour | Health | kg | Lasts (blows) | Gives out first |\n|---|---|---|---|---|---|---|");
    for shell in ["leather", "bone", "copper", "bronze", "wrought_iron", "steel", "diamond"] {
        let parts = [
            Part { slot: "shell", shape: "chest_shell", material: shell, grade: 4 },
            Part { slot: "fastening", shape: "straps", material: "leather", grade: 4 },
            Part { slot: "lining", shape: "lining", material: "wool", grade: 4 },
        ];
        if let Ok(a) = crate::combine::armour("chestplate", &parts) {
            let weak = a.weakest().unwrap();
            let _ = writeln!(
                w,
                "| {shell} | {:.0} | {:.1} | {:.1} | {:.1} | {:.0} | {} |",
                a.level, a.armour, a.health, a.mass_kg, weak.uses, weak.slot
            );
        }
    }

    let _ = writeln!(w, "\n## Example axes (felling head, oak haft, Sound parts)\n");
    let _ = writeln!(w, "| Head | Fitting | Chops | Damage | kg | Lasts (uses) | Gives out first |\n|---|---|---|---|---|---|---|");
    for (head, fit) in [
        ("flint", "sinew"),
        ("obsidian", "birch_tar"),
        ("copper", "copper"),
        ("bronze", "bronze"),
        ("wrought_iron", "wrought_iron"),
        ("steel", "steel"),
    ] {
        let parts = [
            Part { slot: "head", shape: "felling_axe_head", material: head, grade: 4 },
            Part { slot: "haft", shape: "haft", material: "oak", grade: 4 },
            Part { slot: "fitting", shape: "lashing", material: fit, grade: 4 },
        ];
        if let Ok(t) = combine("axe", &parts) {
            let weak = t.weakest().unwrap();
            let _ =
                writeln!(w, "| {head} | {fit} | {:.2} | {:.1} | {:.2} | {:.0} | {} |", t.stat, t.damage, t.mass_kg, weak.uses, weak.slot);
        }
    }
    s
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_sheet_is_fresh() {
        assert_eq!(
            include_str!("../SHEET.md"),
            super::markdown(),
            "regenerate it: cargo run -p crafting --bin sheet > mods/base/crafting/SHEET.md"
        );
    }
}
