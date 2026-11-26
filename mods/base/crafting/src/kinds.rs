//! Kinds of item built from parts and the part shapes that fit them (`notes/CRAFTING.md`, "Kinds
//! and their parts"). A kind is its slots; a shape is one way to make a slot's part, with its
//! size and what it does to the numbers (4B: "part shapes with trade-offs").

/// What a part does in its item (`notes/CRAFTING.md`, "How stats combine").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Sets the numbers: head, blade, point, stave, shell.
    Working,
    /// Multiplies them: haft, grip, shaft.
    Holding,
    /// Holds the others together: a lashing, glue, rivets, a wedge; a trait and its own wear.
    Fitting,
    /// Adds to the look and a little to the numbers: a guard, a pommel.
    Extra,
    /// Pads armour and keeps its wearer warm: Survival's clothing in numbers (wool, linen, fur).
    Lining,
}

/// Seconds a chopping tool's hit takes a raised shield down for (vanilla's axe, and Weft's
/// `comp::DISABLE_BLOCKING` on its own axes).
pub const CHOP_DISABLES_BLOCKING: f32 = 5.0;

/// What a kind's working part is for, and so its main stat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    Chop,
    Mine,
    Dig,
    Till,
    Cut,
    Thrust,
    /// A bow: its power is its stave's spring.
    Shoot,
    /// Armour: its shell stops blows (`armour`).
    Guard,
}

#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub name: &'static str,
    pub role: Role,
}

#[derive(Clone, Copy, Debug)]
pub struct Kind {
    pub id: &'static str,
    pub key: &'static str,
    pub use_: Use,
    pub slots: &'static [Slot],
    /// Damage of the kind itself, before its head's material (a sword's is high, a hoe's low).
    pub damage: f32,
}

const fn s(name: &'static str, role: Role) -> Slot {
    Slot { name, role }
}

const TOOL: &[Slot] = &[s("head", Role::Working), s("haft", Role::Holding), s("fitting", Role::Fitting)];
/// Armour: the shell stops blows, the fastening holds it together (and wears first), the lining
/// pads and warms (`notes/CRAFTING.md`, "Kinds and their parts").
const ARMOUR: &[Slot] = &[s("shell", Role::Working), s("fastening", Role::Fitting), s("lining", Role::Lining)];

pub const KINDS: &[Kind] = &[
    Kind { id: "axe", key: "kind.base.axe", use_: Use::Chop, slots: TOOL, damage: 5.0 },
    Kind { id: "pickaxe", key: "kind.base.pickaxe", use_: Use::Mine, slots: TOOL, damage: 2.0 },
    Kind { id: "shovel", key: "kind.base.shovel", use_: Use::Dig, slots: TOOL, damage: 1.5 },
    Kind { id: "hoe", key: "kind.base.hoe", use_: Use::Till, slots: TOOL, damage: 1.0 },
    Kind {
        id: "knife",
        key: "kind.base.knife",
        use_: Use::Cut,
        slots: &[s("blade", Role::Working), s("grip", Role::Holding), s("fitting", Role::Fitting)],
        damage: 2.5,
    },
    Kind {
        id: "spear",
        key: "kind.base.spear",
        use_: Use::Thrust,
        slots: &[s("point", Role::Working), s("shaft", Role::Holding), s("fitting", Role::Fitting)],
        damage: 4.0,
    },
    Kind {
        id: "sword",
        key: "kind.base.sword",
        use_: Use::Cut,
        slots: &[s("blade", Role::Working), s("guard", Role::Extra), s("grip", Role::Holding), s("pommel", Role::Extra)],
        damage: 4.0,
    },
    Kind {
        id: "bow",
        key: "kind.base.bow",
        use_: Use::Shoot,
        slots: &[s("stave", Role::Working), s("grip", Role::Holding), s("string", Role::Fitting)],
        damage: 6.0,
    },
    // An arrow: only its head is a part (its material sets the arrow's item level,
    // `vitals::gear::arrow_level`); a stick and a feather make the rest, four at a time at the
    // assembly row (`base`'s `parts::fletched`). Never built as a tool.
    Kind { id: "arrow", key: "kind.base.arrow", use_: Use::Shoot, slots: &[s("head", Role::Working)], damage: 0.0 },
    // Armour, head to feet (`materials::ARMOUR`, `vitals::gear::SLOT_SHARE`).
    Kind { id: "helmet", key: "kind.base.helmet", use_: Use::Guard, slots: ARMOUR, damage: 0.0 },
    Kind { id: "chestplate", key: "kind.base.chestplate", use_: Use::Guard, slots: ARMOUR, damage: 0.0 },
    Kind { id: "leggings", key: "kind.base.leggings", use_: Use::Guard, slots: ARMOUR, damage: 0.0 },
    Kind { id: "boots", key: "kind.base.boots", use_: Use::Guard, slots: ARMOUR, damage: 0.0 },
];

pub fn kind(id: &str) -> Option<&'static Kind> {
    KINDS.iter().find(|k| k.id == id)
}

impl Kind {
    /// Its working slot's name (every kind has one).
    pub fn working(&self) -> &'static str {
        self.slots
            .iter()
            .find(|s| s.role == Role::Working)
            .map_or("", |s| s.name)
    }

    /// For armour, its slot head to feet (0-3).
    pub fn armour_slot(&self) -> Option<usize> {
        crate::materials::ARMOUR.iter().position(|a| *a == self.id)
    }
}

/// One way to make a slot's part: its size, and what it does to the item's numbers (×). The
/// trade-offs of 4B: a bearded axe head hits harder and swings slower than a felling one.
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub id: &'static str,
    pub key: &'static str,
    pub kind: &'static str,
    pub slot: &'static str,
    /// cm³ of a metal part this shape; stone and bone are `Material::bulk` times it.
    pub volume: f32,
    pub stat: f32,
    pub speed: f32,
    pub damage: f32,
    /// Blocks of reach it adds (a long shaft).
    pub reach: f32,
}

const fn sh(id: &'static str, kind: &'static str, slot: &'static str, volume: f32, nums: (f32, f32, f32, f32)) -> Shape {
    Shape { id, key: id, kind, slot, volume, stat: nums.0, speed: nums.1, damage: nums.2, reach: nums.3 }
}

/// Keys are `shape.base.<id>`; `key` holds the id and `Shape::name_key` builds it.
pub const SHAPES: &[Shape] = &[
    // Axes: a felling head bites deep; a bearded one is a weapon too.
    sh("felling_axe_head", "axe", "head", 150.0, (1.0, 1.0, 1.0, 0.0)),
    sh("bearded_axe_head", "axe", "head", 170.0, (0.9, 0.95, 1.2, 0.0)),
    sh("pick_head", "pickaxe", "head", 180.0, (1.0, 1.0, 1.0, 0.0)),
    sh("shovel_blade", "shovel", "head", 200.0, (1.0, 1.0, 1.0, 0.0)),
    sh("hoe_blade", "hoe", "head", 120.0, (1.0, 1.0, 1.0, 0.0)),
    sh("knife_blade", "knife", "blade", 25.0, (1.0, 1.0, 1.0, 0.0)),
    sh("leaf_point", "spear", "point", 40.0, (1.0, 1.0, 1.0, 0.0)),
    sh("barbed_point", "spear", "point", 45.0, (0.9, 1.0, 1.15, 0.0)),
    // Swords: a long blade reaches, a short one is quick.
    sh("straight_blade", "sword", "blade", 140.0, (1.0, 1.0, 1.0, 0.0)),
    sh("long_blade", "sword", "blade", 180.0, (1.0, 0.9, 1.1, 0.5)),
    sh("short_blade", "sword", "blade", 100.0, (1.0, 1.15, 0.9, -0.3)),
    // Holding parts: length is reach and swing.
    sh("haft", "axe", "haft", 450.0, (1.0, 1.0, 1.0, 0.0)),
    sh("haft", "pickaxe", "haft", 450.0, (1.0, 1.0, 1.0, 0.0)),
    sh("haft", "shovel", "haft", 450.0, (1.0, 1.0, 1.0, 0.0)),
    sh("haft", "hoe", "haft", 450.0, (1.0, 1.0, 1.0, 0.0)),
    sh("long_haft", "axe", "haft", 600.0, (1.0, 0.9, 1.15, 0.3)),
    sh("grip", "knife", "grip", 60.0, (1.0, 1.0, 1.0, 0.0)),
    sh("grip", "sword", "grip", 80.0, (1.0, 1.0, 1.0, 0.0)),
    sh("shaft", "spear", "shaft", 700.0, (1.0, 1.0, 1.0, 1.0)),
    // Fittings and extras.
    sh("lashing", "axe", "fitting", 5.0, (1.0, 1.0, 1.0, 0.0)),
    sh("lashing", "pickaxe", "fitting", 5.0, (1.0, 1.0, 1.0, 0.0)),
    sh("lashing", "shovel", "fitting", 5.0, (1.0, 1.0, 1.0, 0.0)),
    sh("lashing", "hoe", "fitting", 5.0, (1.0, 1.0, 1.0, 0.0)),
    sh("lashing", "knife", "fitting", 3.0, (1.0, 1.0, 1.0, 0.0)),
    sh("lashing", "spear", "fitting", 4.0, (1.0, 1.0, 1.0, 0.0)),
    sh("bar_guard", "sword", "guard", 20.0, (1.0, 1.0, 1.0, 0.0)),
    sh("round_pommel", "sword", "pommel", 15.0, (1.0, 1.02, 1.0, 0.0)),
    // Bows: a long self stave draws heavy; a short one is quick and light.
    sh("self_stave", "bow", "stave", 400.0, (1.0, 1.0, 1.0, 0.0)),
    sh("short_stave", "bow", "stave", 280.0, (1.0, 1.15, 0.85, 0.0)),
    sh("grip", "bow", "grip", 40.0, (1.0, 1.0, 1.0, 0.0)),
    sh("bowstring", "bow", "string", 2.0, (1.0, 1.0, 1.0, 0.0)),
    // An arrowhead: a few grams of flint, bone or metal.
    sh("arrowhead", "arrow", "head", 2.0, (1.0, 1.0, 1.0, 0.0)),
    // Armour: a shell of its slot's size (a metal coat about 1.2 litres of iron, 9 kg; a leather
    // one under a kilogram), straps or rivets, and a lining (cloth's own density).
    sh("helmet_shell", "helmet", "shell", 300.0, (1.0, 1.0, 1.0, 0.0)),
    sh("chest_shell", "chestplate", "shell", 1200.0, (1.0, 1.0, 1.0, 0.0)),
    sh("leg_shell", "leggings", "shell", 900.0, (1.0, 1.0, 1.0, 0.0)),
    sh("boot_shell", "boots", "shell", 400.0, (1.0, 1.0, 1.0, 0.0)),
    sh("straps", "helmet", "fastening", 10.0, (1.0, 1.0, 1.0, 0.0)),
    sh("straps", "chestplate", "fastening", 30.0, (1.0, 1.0, 1.0, 0.0)),
    sh("straps", "leggings", "fastening", 20.0, (1.0, 1.0, 1.0, 0.0)),
    sh("straps", "boots", "fastening", 10.0, (1.0, 1.0, 1.0, 0.0)),
    sh("lining", "helmet", "lining", 400.0, (1.0, 1.0, 1.0, 0.0)),
    sh("lining", "chestplate", "lining", 2000.0, (1.0, 1.0, 1.0, 0.0)),
    sh("lining", "leggings", "lining", 1500.0, (1.0, 1.0, 1.0, 0.0)),
    sh("lining", "boots", "lining", 500.0, (1.0, 1.0, 1.0, 0.0)),
];

impl Shape {
    pub fn name_key(&self) -> String {
        format!("shape.base.{}", self.id)
    }
}

/// The shapes that fit `kind`'s slot `slot`.
pub fn shapes_for<'a>(kind: &'a str, slot: &'a str) -> impl Iterator<Item = &'static Shape> + 'a {
    SHAPES
        .iter()
        .filter(move |s| s.kind == kind && s.slot == slot)
}

pub fn shape(kind: &str, id: &str) -> Option<&'static Shape> {
    SHAPES.iter().find(|s| s.kind == kind && s.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_slot_has_a_shape_and_one_working_part() {
        for k in KINDS {
            assert_eq!(k.slots.iter().filter(|s| s.role == Role::Working).count(), 1, "{}", k.id);
            for s in k.slots {
                assert!(shapes_for(k.id, s.name).next().is_some(), "{}: no shape for {}", k.id, s.name);
            }
        }
        for s in SHAPES {
            let k = kind(s.kind).unwrap_or_else(|| panic!("{}: no kind {}", s.id, s.kind));
            assert!(k.slots.iter().any(|x| x.name == s.slot), "{}: {} has no slot {}", s.id, s.kind, s.slot);
        }
    }

    #[test]
    fn shapes_trade_off() {
        // A shape that does more of one thing does less of another.
        for s in SHAPES {
            let up = s.stat > 1.0 || s.damage > 1.0 || s.speed > 1.0 || s.reach > 0.0;
            let down = s.stat < 1.0 || s.damage < 1.0 || s.speed < 1.0 || s.reach < 0.0 || s.volume > 160.0;
            assert!(!up || down || s.slot == "shaft" || s.slot == "pommel", "{} only gains", s.id);
        }
    }
}
