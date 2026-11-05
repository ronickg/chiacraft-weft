//! A material's trait (`notes/CRAFTING.md`, "Materials and traits"): an effect base knows, with
//! its numbers and when it applies. A package gives its material a trait from this list, so it
//! writes no code.

/// When a trait works.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When {
    Always,
    /// The item (or its wearer) is dry: no rain, not in water.
    Dry,
    /// The item is wet.
    Wet,
    /// On thin trees, hide and flesh: what a sharp edge is for.
    Soft,
    /// On stone and ore.
    Stone,
}

/// What it does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    /// Its part's main stat (chop, mine, damage) ×.
    Stat(f32),
    /// Damage ×.
    Damage(f32),
    /// Wear on its part ×.
    Wear(f32),
    /// Repairing its part costs × the material.
    RepairCost(f32),
    /// Its part is never chipped: blows on stone don't wear it.
    NoChips,
    /// A broken part can be melted and cast whole (bronze).
    Recast,
    /// A broken part is mended whole by forge-welding (wrought iron).
    Weld,
    /// Its part's speed × (light wood).
    Speed(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trait {
    pub id: &'static str,
    /// Its name's key; `trait.base.<id>.desc` says what it does.
    pub key: &'static str,
    pub effect: Effect,
    pub when: When,
}

const fn t(id: &'static str, key: &'static str, effect: Effect, when: When) -> Trait {
    Trait { id, key, effect, when }
}

pub const TRAITS: &[Trait] = &[
    t("keen", "trait.base.keen", Effect::Stat(1.15), When::Soft),
    t("common", "trait.base.common", Effect::RepairCost(0.5), When::Always),
    t("razor", "trait.base.razor", Effect::Damage(1.25), When::Always),
    // Razor's other half: obsidian wears fast (two traits would read as one in the tooltip).
    t("brittle", "trait.base.brittle", Effect::Wear(2.0), When::Always),
    t("tough", "trait.base.tough", Effect::NoChips, When::Stone),
    t("soft", "trait.base.soft", Effect::RepairCost(0.0), When::Always),
    t("recastable", "trait.base.recastable", Effect::Recast, When::Always),
    t("weldable", "trait.base.weldable", Effect::Weld, When::Always),
    t("tempered", "trait.base.tempered", Effect::Wear(0.7), When::Always),
    // Star steel's etched plates: no rust, and repairs at half the material.
    t("patterned", "trait.base.patterned", Effect::RepairCost(0.5), When::Always),
    t("light", "trait.base.light", Effect::Speed(1.05), When::Always),
    t("shrinks_tight", "trait.base.shrinks_tight", Effect::Wear(0.5), When::Dry),
    t("glued", "trait.base.glued", Effect::Wear(1.0), When::Wet),
    t("cheap", "trait.base.cheap", Effect::Wear(2.0), When::Wet),
];

pub fn trait_of(id: &str) -> Option<&'static Trait> {
    TRAITS.iter().find(|t| t.id == id)
}

/// The traits a material brings: its own, and obsidian's wear with its edge.
pub fn of_material(id: &str) -> Vec<&'static Trait> {
    let Some(m) = crate::materials::material(id) else { return Vec::new() };
    let mut out: Vec<&'static Trait> = trait_of(m.trait_id).into_iter().collect();
    if m.trait_id == "razor" {
        out.extend(trait_of("brittle"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_unique() {
        for (i, a) in TRAITS.iter().enumerate() {
            assert!(TRAITS[i + 1..].iter().all(|b| b.id != a.id), "{} twice", a.id);
            assert!(a.key.ends_with(a.id));
        }
    }

    #[test]
    fn obsidian_is_sharp_and_short_lived() {
        let t = of_material("obsidian");
        assert!(t.iter().any(|t| t.effect == Effect::Damage(1.25)));
        assert!(t.iter().any(|t| t.effect == Effect::Wear(2.0)));
    }
}
