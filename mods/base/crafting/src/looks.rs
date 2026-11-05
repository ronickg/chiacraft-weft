//! How an item built from parts is drawn (`notes/CRAFTING.md`, "The art format"): each kind's
//! layers on the 32 x 32 item canvas, back to front, with the parts that paint each layer's
//! regions. Parts in the data are not layers in the picture: an axe's lashing is region 1 of its
//! head layer.
//!
//! Heads never move (Chiacraft, 2026-11-04): each material's head has its own anchor, where it
//! meets the handle, and the handle layer moves so its socket meets that anchor. Handles lie on
//! the corner-to-corner diagonal. Anchors and head boxes are measured from the approved drawings
//! (`notes/ui-codex/parts/deliver/tool_sockets.json`); a handle's socket is where it meets the
//! iron head, which the handles were drawn against.

/// One layer of a kind's icon.
#[derive(Clone, Copy, Debug)]
pub struct Layer {
    pub name: &'static str,
    /// The slots whose materials paint it, region 0 first (the map's green channel).
    pub regions: &'static [&'static str],
    /// Where it sits on the canvas: x0, y0, x1, y1.
    pub area: [u8; 4],
    /// Moved so the kind's `socket` meets the working part's anchor (the handle).
    pub moves: bool,
    /// Painted per material (a head), else a shade map painted with ramps (a haft, a wooden
    /// stave).
    pub painted: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct KindLook {
    pub kind: &'static str,
    /// Where the handle layer meets the head, as drawn.
    pub socket: [f32; 2],
    /// Where each material's head meets the handle (art names: `art_name`).
    pub anchors: &'static [(&'static str, [f32; 2])],
    /// Back to front.
    pub layers: &'static [Layer],
}

pub const LOOKS: &[KindLook] = &[
    KindLook {
        kind: "axe",
        socket: [11.7, 21.3],
        anchors: &[
            ("wood", [11.8, 22.2]),
            ("stone", [11.9, 22.6]),
            ("flint", [11.7, 21.8]),
            ("copper", [11.2, 22.8]),
            ("bronze", [11.8, 22.2]),
            ("iron", [11.7, 21.3]),
            ("steel", [12.1, 21.5]),
            ("gold", [11.4, 22.6]),
            ("diamond", [11.5, 22.0]),
        ],
        layers: &[
            Layer { name: "haft", regions: &["haft"], area: [0, 0, 31, 31], moves: true, painted: false },
            Layer { name: "head", regions: &["head", "fitting"], area: [7, 0, 28, 23], moves: false, painted: true },
        ],
    },
    KindLook {
        kind: "pickaxe",
        socket: [13.4, 16.5],
        anchors: &[
            ("wood", [13.3, 16.6]),
            ("stone", [13.2, 17.5]),
            ("flint", [12.7, 17.2]),
            ("copper", [13.0, 16.1]),
            ("bronze", [13.3, 16.3]),
            ("iron", [13.4, 16.5]),
            ("steel", [13.8, 17.0]),
            ("gold", [13.9, 17.2]),
            ("diamond", [13.0, 16.0]),
        ],
        layers: &[
            Layer { name: "haft", regions: &["haft"], area: [0, 0, 31, 31], moves: true, painted: false },
            Layer { name: "head", regions: &["head", "fitting"], area: [5, 1, 30, 20], moves: false, painted: true },
        ],
    },
    KindLook {
        kind: "shovel",
        socket: [19.1, 13.4],
        anchors: &[
            ("wood", [17.0, 14.3]),
            ("stone", [16.8, 15.1]),
            ("flint", [19.1, 12.3]),
            ("copper", [19.0, 13.2]),
            ("bronze", [17.7, 14.0]),
            ("iron", [19.1, 13.4]),
            ("steel", [18.9, 13.2]),
            ("gold", [19.1, 12.4]),
            ("diamond", [19.7, 11.9]),
        ],
        layers: &[
            Layer { name: "haft", regions: &["haft"], area: [0, 0, 31, 31], moves: true, painted: false },
            Layer { name: "head", regions: &["head", "fitting"], area: [16, 1, 30, 16], moves: false, painted: true },
        ],
    },
    KindLook {
        kind: "hoe",
        socket: [15.2, 17.2],
        anchors: &[
            ("wood", [14.9, 18.1]),
            ("stone", [14.8, 18.3]),
            ("flint", [14.5, 17.4]),
            ("copper", [15.2, 18.4]),
            ("bronze", [15.2, 19.3]),
            ("iron", [15.2, 17.2]),
            ("steel", [15.4, 17.6]),
            ("gold", [15.3, 18.8]),
            ("diamond", [14.7, 19.8]),
        ],
        layers: &[
            Layer { name: "haft", regions: &["haft"], area: [0, 0, 31, 31], moves: true, painted: false },
            Layer { name: "head", regions: &["head", "fitting"], area: [10, 1, 27, 21], moves: false, painted: true },
        ],
    },
    KindLook {
        kind: "sword",
        socket: [10.8, 20.6],
        anchors: &[
            ("wood", [9.8, 21.7]),
            ("stone", [10.8, 20.5]),
            ("copper", [11.8, 19.4]),
            ("bronze", [10.8, 20.5]),
            ("iron", [10.8, 20.6]),
            ("steel", [10.5, 20.8]),
            ("gold", [11.3, 20.0]),
            ("diamond", [12.0, 20.3]),
        ],
        layers: &[
            Layer { name: "blade", regions: &["blade"], area: [6, 0, 31, 26], moves: false, painted: true },
            Layer { name: "hilt", regions: &["grip", "guard", "pommel"], area: [0, 17, 15, 31], moves: true, painted: false },
        ],
    },
    // A bow drawn from corner to corner, the string behind the stave and the grip a region of it.
    // Nothing moves: one stave drawing per shape, its wood a ramp.
    KindLook {
        kind: "bow",
        socket: [16.0, 16.0],
        anchors: &[],
        layers: &[
            Layer { name: "string", regions: &["string"], area: [0, 0, 31, 31], moves: false, painted: false },
            Layer { name: "stave", regions: &["stave", "grip"], area: [0, 0, 31, 31], moves: false, painted: false },
        ],
    },
];

pub fn look(kind: &str) -> Option<&'static KindLook> {
    LOOKS.iter().find(|l| l.kind == kind)
}

/// The name a material's drawings go by: woods share the wooden head, wrought iron is "iron".
pub fn art_name(material: &str) -> &str {
    match material {
        "oak" | "birch" | "spruce" | "mesquite" | "branch" => "wood",
        "wrought_iron" => "iron",
        m => m,
    }
}

/// How far the moving layers shift (whole pixels) for a working part of `material`: its head's
/// anchor less the handle's socket. Nothing for a material with no anchor yet.
pub fn shift(kind: &str, material: &str) -> [i32; 2] {
    let Some(l) = look(kind) else { return [0, 0] };
    let art = drawn(kind, material);
    match l.anchors.iter().find(|(m, _)| *m == art) {
        Some((_, a)) => [(a[0] - l.socket[0]).round() as i32, (a[1] - l.socket[1]).round() as i32],
        None => [0, 0],
    }
}

/// A layer's textures, named by the shape of the part in its region 0 (so a bearded axe head
/// and a felling one each have their own): painted, `base:parts/<kind>/<shape>/<art name>`, used
/// as it is; or a shade map, `base:parts/<kind>/<shape>`, painted with ramps.
pub fn painted(kind: &str, shape: &str, material: &str) -> String {
    format!("base:parts/{kind}/{shape}/{}", drawn(kind, material))
}

/// The drawing a working part of `material` uses on `kind`: its own where it's drawn (the kind's
/// anchors list the drawn ones), else the nearest drawn one until its art comes (meteoric iron as
/// iron, star steel as steel, chert, obsidian and tektite as flint, bone and antler, and flint on a
/// sword, as stone).
pub fn drawn<'a>(kind: &str, material: &'a str) -> &'a str {
    let Some(l) = look(kind) else { return art_name(material) };
    let has = |a: &str| l.anchors.is_empty() || l.anchors.iter().any(|(m, _)| *m == a);
    let mut art = art_name(material);
    for _ in 0..3 {
        if has(art) {
            return art;
        }
        art = match art {
            "meteoric_iron" => "iron",
            "star_steel" => "steel",
            "chert" | "obsidian" | "tektite" => "flint",
            "flint" | "bone" | "antler" => "stone",
            _ => "iron",
        };
    }
    art
}

pub fn map(kind: &str, shape: &str) -> String {
    format!("base:parts/{kind}/{shape}")
}

pub fn ramp(material: &str) -> String {
    format!("base:ramps/{material}")
}

/// Where gems sit on each kind's 32 px icon, the centre of its first, second and third socket
/// (measured on the iron heads, the hilt's guard and pommel, and the armour shells' icons).
pub const GEM_SPOTS: &[(&str, [[i32; 2]; 3])] = &[
    ("axe", [[19, 9], [22, 6], [16, 12]]),
    ("pickaxe", [[18, 9], [22, 6], [14, 12]]),
    ("shovel", [[23, 8], [25, 5], [21, 11]]),
    ("hoe", [[18, 9], [21, 6], [15, 12]]),
    ("sword", [[9, 22], [4, 27], [12, 19]]),
    // Kinds drawn by their plain icon until their parts have art: the knife's bolster, the
    // spear's socket under its point, the bow's grip.
    ("knife", [[12, 20], [8, 24], [16, 16]]),
    ("spear", [[22, 11], [19, 14], [16, 17]]),
    ("bow", [[7, 12], [5, 16], [10, 8]]),
    ("helmet", [[16, 13], [12, 16], [20, 16]]),
    ("chestplate", [[15, 13], [10, 10], [20, 10]]),
    ("leggings", [[12, 8], [19, 8], [15, 4]]),
    ("boots", [[11, 10], [20, 10], [15, 6]]),
];

/// How a shell is built, and so drawn (Ronald, 11-05: "Fix all as recommended"): hardened leather
/// and rawhide as the approved leather set, bone and antler as a lamellar of laced slats, diamond as
/// the approved diamond set's quilted scales, the metals as plate (the approved iron set).
pub fn shell_family(material: &str) -> &'static str {
    match material {
        "leather" | "rawhide" => "leather",
        "bone" | "antler" => "bone",
        "diamond" => "diamond",
        _ => "plate",
    }
}

/// Worn armour drawn from its parts (`abi::comp::WORN_LOOK`): its shell family's shade map per worn
/// layer in the wearer's skin layout (`base:entity/armor/parts/<family>_<1|2>`, made from that
/// family's approved set: region 0 its shell, 1 its fastening, 2 its lining, each pixel the nearest
/// step of its source material's ramp), painted with the three parts' ramps; leggings are the inner
/// layer (2), the rest the outer (1).
pub fn worn_look(slot: usize, shell: &str, fastening: &str, lining: &str, gems: &[&str]) -> String {
    let layer = if slot == 2 { 2 } else { 1 };
    let family = shell_family(shell);
    let mut look = format!("base:entity/armor/parts/{family}_{layer}@{},{},{}", ramp(shell), ramp(fastening), ramp(lining));
    for (i, g) in gems.iter().enumerate() {
        if let Some(spot) = WORN_GEM_SPOTS.get(slot).and_then(|s| s.get(i)) {
            let (x, y) = (spot[0] - GEM_SIZE / 2, spot[1] - GEM_SIZE / 2);
            look += &format!(";base:entity/armor/parts/gem_set@{}+{x},{y}", ramp(&format!("gem_{g}")));
        }
    }
    look
}

/// Where gems sit on worn armour, the centre of each socket's in the 128 x 64 skin layout, head to
/// feet: the helmet's brow, the chest, the thigh, the shin.
pub const WORN_GEM_SPOTS: [[[i32; 2]; 3]; 4] =
    [[[24, 19], [20, 19], [28, 19]], [[48, 46], [43, 46], [53, 46]], [[12, 44], [12, 50], [12, 56]], [[12, 58], [10, 54], [14, 54]]];

/// The gem drawing's size: a shade map with the gem in its top-left corner
/// (`base:parts/gem/set`), moved to its socket's spot.
pub const GEM_SIZE: i32 = 5;

/// The look layer of a gem `gem` (`crafting::gems::GEMS`) in socket `i` of an item of `kind`:
/// the gem's map painted with its ramp (`base:ramps/gem_<gem>`) at the socket's spot; none for a
/// kind without spots (drawn by its plain icon).
pub fn gem_look(kind: &str, i: usize, gem: &str) -> Option<String> {
    let spot = GEM_SPOTS.iter().find(|(k, _)| *k == kind)?.1.get(i)?;
    let (x, y) = (spot[0] - GEM_SIZE / 2, spot[1] - GEM_SIZE / 2);
    Some(format!("base:parts/gem/set@{}+{x},{y}", ramp(&format!("gem_{gem}"))))
}

/// Part shapes drawn from one shade map painted with their material's ramp, as a loose part and
/// (a shell) as the built piece (`base:parts/armour/<shape>`, made from the approved icons): armour's
/// shells, straps (their buckles a second region: the straps' own metal, else wrought iron) and
/// linings. Ronald, 11-05: a shell's icon shows its material.
pub const PART_MAPS: [&str; 6] = ["helmet_shell", "chest_shell", "leg_shell", "boot_shell", "straps", "lining"];

/// The look of a part of `shape` made of `material` (the engine's `core:look`), if it's drawn
/// from a map.
pub fn part_look(shape: &str, material: &str) -> Option<String> {
    if !PART_MAPS.contains(&shape) {
        return None;
    }
    let metal = crate::materials::material(material).is_some_and(|m| m.works == crate::materials::Works::Metal || m.id == "gold");
    let buckle = if shape != "straps" {
        String::new()
    } else if metal {
        format!(",{}", ramp(material))
    } else {
        format!(",{}", ramp("wrought_iron"))
    };
    // A shell is drawn as its family is built: a leather cap, bone slats.
    let drawn = match (shell_family(material), shape) {
        ("leather", "helmet_shell") => "cap_shell".to_string(),
        ("bone", s) if s.ends_with("_shell") => format!("bone_{s}"),
        _ => shape.to_string(),
    };
    Some(format!("base:parts/armour/{drawn}@{}{buckle}", ramp(material)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_region_is_a_slot_and_every_slot_is_drawn() {
        for l in LOOKS {
            let k = crate::kinds::kind(l.kind).unwrap();
            let drawn: Vec<&str> = l
                .layers
                .iter()
                .flat_map(|x| x.regions.iter().copied())
                .collect();
            for s in k.slots {
                assert_eq!(drawn.iter().filter(|d| **d == s.name).count(), 1, "{}: {} drawn once", l.kind, s.name);
            }
            assert_eq!(drawn.len(), k.slots.len(), "{}: no region without a slot", l.kind);
            for x in l.layers {
                assert!(x.area[0] < x.area[2] && x.area[1] < x.area[3] && x.area[2] < 32 && x.area[3] < 32);
            }
            assert!(l.layers.iter().filter(|x| x.moves).count() <= 1, "{}: one layer moves", l.kind);
        }
    }

    #[test]
    fn heads_stay_handles_move() {
        // Iron is what the handles were drawn against: no shift; the others a pixel or two.
        assert_eq!(shift("axe", "wrought_iron"), [0, 0]);
        for l in LOOKS {
            for (m, _) in l.anchors {
                let s = shift(l.kind, m);
                assert!(s[0].abs() <= 3 && s[1].abs() <= 3, "{} {m}: {s:?}", l.kind);
            }
        }
        assert_eq!(painted("axe", "felling_axe_head", "oak"), "base:parts/axe/felling_axe_head/wood");
        // Until their art comes, undrawn heads use the nearest drawn one.
        assert_eq!(painted("pickaxe", "pick_head", "meteoric_iron"), "base:parts/pickaxe/pick_head/iron");
        assert_eq!(painted("sword", "straight_blade", "flint"), "base:parts/sword/straight_blade/stone");
        assert_eq!(painted("axe", "felling_axe_head", "obsidian"), "base:parts/axe/felling_axe_head/flint");
        assert_eq!(part_look("chest_shell", "bronze").unwrap(), "base:parts/armour/chest_shell@base:ramps/bronze");
        assert_eq!(part_look("straps", "leather").unwrap(), "base:parts/armour/straps@base:ramps/leather,base:ramps/wrought_iron");
        assert!(part_look("haft", "oak").is_none());
        assert_eq!(part_look("helmet_shell", "leather").unwrap(), "base:parts/armour/cap_shell@base:ramps/leather");
        assert_eq!(part_look("leg_shell", "bone").unwrap(), "base:parts/armour/bone_leg_shell@base:ramps/bone");
        assert!(worn_look(2, "diamond", "leather", "wool", &[]).starts_with("base:entity/armor/parts/diamond_2@"));
        assert_eq!(gem_look("sword", 0, "diamond").unwrap(), "base:parts/gem/set@base:ramps/gem_diamond+7,20");
        assert!(gem_look("axe", 3, "diamond").is_none() && gem_look("sword", 3, "diamond").is_none());
        assert_eq!(
            worn_look(1, "bronze", "gold", "wool", &["amber"]),
            "base:entity/armor/parts/plate_1@base:ramps/bronze,base:ramps/gold,base:ramps/wool;base:entity/armor/parts/gem_set@base:ramps/gem_amber+46,44"
        );
    }
}
