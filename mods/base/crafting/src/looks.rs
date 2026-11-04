//! How an item built from parts is drawn (`notes/CRAFTING.md`, "The art format"): each kind's
//! layers on the 32 x 32 item canvas, back to front, with the parts that paint each layer's
//! regions. Parts in the data are not layers in the picture: an axe's lashing is region 1 of its
//! head layer. Sockets and boxes are Chiacraft's, settled with Ronald 2026-11-04.

/// One layer of a kind's icon.
#[derive(Clone, Copy, Debug)]
pub struct Layer {
    pub name: &'static str,
    /// The slots whose materials paint it, region 0 first (the map's green channel).
    pub regions: &'static [&'static str],
    /// Where it sits on the canvas: x0, y0, x1, y1.
    pub area: [u8; 4],
}

#[derive(Clone, Copy, Debug)]
pub struct KindLook {
    pub kind: &'static str,
    /// Where the working part meets the rest.
    pub socket: [f32; 2],
    /// Back to front.
    pub layers: &'static [Layer],
}

pub const LOOKS: &[KindLook] = &[
    KindLook {
        kind: "axe",
        socket: [14.9, 14.8],
        layers: &[
            Layer { name: "haft", regions: &["haft"], area: [0, 10, 17, 31] },
            Layer { name: "head", regions: &["head", "fitting"], area: [13, 0, 31, 20] },
        ],
    },
    KindLook {
        kind: "sword",
        socket: [8.1, 23.4],
        layers: &[
            Layer { name: "blade", regions: &["blade"], area: [6, 0, 31, 26] },
            Layer { name: "hilt", regions: &["grip", "guard", "pommel"], area: [0, 17, 15, 31] },
        ],
    },
];

pub fn look(kind: &str) -> Option<&'static KindLook> {
    LOOKS.iter().find(|l| l.kind == kind)
}

/// A layer's textures: a painted one for a material (`base:parts/<kind>/<layer>/<material>`, used
/// as it is) or the shade map (`base:parts/<kind>/<layer>`) painted with ramps.
pub fn painted(kind: &str, layer: &str, material: &str) -> String {
    format!("base:parts/{kind}/{layer}/{material}")
}

pub fn map(kind: &str, layer: &str) -> String {
    format!("base:parts/{kind}/{layer}")
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
        }
    }
}
