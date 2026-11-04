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
    let art = art_name(material);
    match l.anchors.iter().find(|(m, _)| *m == art) {
        Some((_, a)) => [(a[0] - l.socket[0]).round() as i32, (a[1] - l.socket[1]).round() as i32],
        None => [0, 0],
    }
}

/// A layer's textures, named by the shape of the part in its region 0 (so a bearded axe head
/// and a felling one each have their own): painted, `base:parts/<kind>/<shape>/<art name>`, used
/// as it is; or a shade map, `base:parts/<kind>/<shape>`, painted with ramps.
pub fn painted(kind: &str, shape: &str, material: &str) -> String {
    format!("base:parts/{kind}/{shape}/{}", art_name(material))
}

pub fn map(kind: &str, shape: &str) -> String {
    format!("base:parts/{kind}/{shape}")
}

pub fn ramp(material: &str) -> String {
    format!("base:ramps/{material}")
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
    }
}
