//! The perk web (`notes/PROGRESSION.md`, "One web, like Path of Exile's passive tree"): one big
//! web with a start in the centre and a region per skill (`skills::SKILLS`, clockwise from the
//! top). A point buys a node next to one you have, so you grow outward. Small nodes give a unit
//! (`web::UNIT`, +2 %), notables three or four, keystones at the rim change how you play: a lot of
//! one family for as much of another (`web::KEYSTONE_UNITS`). A region's deeper rings need that
//! skill's level (`GATES`), so you only go deep where you actually play. Bridges between
//! neighbouring regions reward hybrids and need both skills. A rare item refunds a few nodes at a
//! time (`REFUNDS_PER_ITEM`), only ones the rest don't hang from.
//!
//! Every region is built from one template (`TEMPLATE`), so they're the same size and worth the
//! same; what differs is which families its lanes give, its notables' names and its keystone
//! (`REGIONS`). New content adds rings at the rim (`GATES` has room past the keystone) or nodes
//! off a notable: a node's id is its region's slot × 64 + its place there, so new ones never move
//! a saved build.
//!
//! The checks (`problems`): every node reachable from the start, every region inside its budget,
//! no notable or keystone off its price (`web::Node::units`), and no dominant build
//! (`dominance`: hill-climbed builds for each job; one may not win every job by over 15 %, and
//! no node may be in over 90 % of them). The sheet: `cargo run -p vitals --bin web_sheet`.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::OnceLock;

use crate::skills::{self, COUNT, SKILLS, Skill};
use crate::sockets::{self, Flow, Kind as StoneKind, RADIUS, Stone, Unique};
use crate::web::{FAMILIES, Family, KEYSTONE_UNITS, Node as Kind, family_signed, units_to_ceiling};

/// Node ids a region has room for (one `u64` of a saved build each).
pub const REGION_SLOTS: u16 = 64;
/// Words a saved build takes: the centre's, then one a region.
pub const WORDS: usize = COUNT + 1;
/// The start, in the centre: taken by everyone, free.
pub const START: u16 = 0;
/// Nodes one rare item refunds.
pub const REFUNDS_PER_ITEM: u8 = 3;

/// The skill level each ring needs (ring 1 the entry beside the centre; index 0 unused). Rings 1-8
/// are built; 9-12 are room for later ages at the rim.
pub const GATES: [u32; 13] = [0, 0, 0, 5, 10, 15, 22, 30, 40, 55, 70, 85, 100];

/// The ring a gate belongs to, for a fractional ring (a node between two rings takes the outer).
pub fn gate(ring: f32) -> u32 {
    GATES[(ring.ceil() as usize).clamp(1, GATES.len() - 1)]
}

/// Something a node does besides its units, read by the code it belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Special {
    /// Shows ley lines without a dowsing rod (the Ley lines work reads it: `has`).
    LeySight,
}

impl Special {
    pub const fn id(self) -> &'static str {
        match self {
            Special::LeySight => "ley_sight",
        }
    }
}

/// One node of the web.
#[derive(Clone, Debug)]
pub struct Node {
    pub id: u16,
    pub kind: Kind,
    /// The region it's in (`None`: the start).
    pub region: Option<Skill>,
    /// The second skill a bridge needs.
    pub bridge: Option<Skill>,
    /// Its depth in rings out from the centre (what its skill gate goes by, `GATES`), and where
    /// it's drawn (GUI px from the centre at zoom 1, y down; `web_layout`).
    pub ring: f32,
    pub x: f32,
    pub y: f32,
    /// Units it gives each family (a keystone's cost negative).
    pub gives: Vec<(Family, f32)>,
    pub special: Option<Special>,
    /// Its translation key's tail: `perk.base.<key>` (small nodes share their family's).
    pub key: String,
    /// The skill level it needs, in its region's skill (and a bridge's second skill).
    pub needs: u32,
    /// A socket's saved index (`SOCKETS`), stable like ids.
    pub socket: Option<u8>,
}

impl Node {
    /// Where it's drawn, in GUI pixels from the centre at zoom 1 (y down).
    pub fn pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    /// Its units in all, gains only (a keystone's power).
    pub fn gain(&self) -> f32 {
        self.gives.iter().map(|g| g.1.max(0.0)).sum()
    }

    /// Its net units (a keystone's power less its cost).
    pub fn net(&self) -> f32 {
        self.gives.iter().map(|g| g.1).sum()
    }

    /// Whether these skill levels let it be taken.
    pub fn open_at(&self, levels: &[u32; COUNT]) -> bool {
        let ok = |s: Option<Skill>| s.is_none_or(|s| levels[s.index()] >= self.needs);
        ok(self.region) && ok(self.bridge)
    }
}

/// GUI pixels from the centre to a ring.
pub fn radius(ring: f32) -> f32 {
    if ring <= 0.0 { 0.0 } else { RING0 + ring * RING_STEP }
}
pub const RING0: f32 = 40.0;
pub const RING_STEP: f32 = 30.0;
/// GUI pixels a lane sits beside its region's middle line.
pub const LANE_PX: f32 = 16.0;

/// The web: its nodes and the links between them.
pub struct Web {
    pub nodes: Vec<Node>,
    pub links: Vec<(u16, u16)>,
    /// The links that curve: the centre of each one's arc (`web_layout::key` order).
    pub arcs: crate::web_layout::Arcs,
    index: HashMap<u16, usize>,
    near: HashMap<u16, Vec<u16>>,
}

impl Web {
    pub fn node(&self, id: u16) -> Option<&Node> {
        self.index.get(&id).map(|&i| &self.nodes[i])
    }
    /// The nodes linked to `id`.
    pub fn next_to(&self, id: u16) -> &[u16] {
        self.near.get(&id).map_or(&[], |v| v.as_slice())
    }
}

// ------------------------------------------------------------------ the regions

/// A template node: its place in the region, ring, lane (-1 left, +1 right), kind, which lane's
/// family it gives (`A` or `B`, `M` the region's main family) and the nodes it hangs from.
struct T {
    at: u16,
    ring: f32,
    lane: f32,
    kind: Kind,
    side: Side,
    from: &'static [u16],
}
#[derive(Clone, Copy, PartialEq)]
enum Side {
    A,
    B,
    /// A notable or the keystone, named in `REGIONS`.
    Named(usize),
}

/// One region's shape: an entry, two lanes of small nodes with a notable at ring 4 and ring 7
/// each, a link across at ring 4.6, the lanes meeting at 7.6, and a keystone at the rim (8.5).
/// 16 nodes, 25 units besides the keystone.
const TEMPLATE: [T; 16] = [
    T { at: 0, ring: 1.0, lane: 0.0, kind: Kind::Small, side: Side::A, from: &[] },
    T { at: 1, ring: 2.0, lane: -1.0, kind: Kind::Small, side: Side::A, from: &[0] },
    T { at: 2, ring: 2.0, lane: 1.0, kind: Kind::Small, side: Side::B, from: &[0] },
    T { at: 3, ring: 3.0, lane: -1.0, kind: Kind::Small, side: Side::A, from: &[1] },
    T { at: 4, ring: 3.0, lane: 1.0, kind: Kind::Small, side: Side::B, from: &[2] },
    T { at: 5, ring: 4.0, lane: -1.0, kind: Kind::Notable, side: Side::Named(0), from: &[3] },
    T { at: 6, ring: 4.0, lane: 1.0, kind: Kind::Notable, side: Side::Named(1), from: &[4] },
    T { at: 7, ring: 5.0, lane: -1.3, kind: Kind::Small, side: Side::A, from: &[5] },
    T { at: 8, ring: 5.0, lane: 1.3, kind: Kind::Small, side: Side::B, from: &[6] },
    T { at: 9, ring: 4.6, lane: 0.0, kind: Kind::Small, side: Side::A, from: &[5, 6] },
    T { at: 10, ring: 6.0, lane: -1.3, kind: Kind::Small, side: Side::A, from: &[7] },
    T { at: 11, ring: 6.0, lane: 1.3, kind: Kind::Small, side: Side::B, from: &[8] },
    T { at: 12, ring: 7.0, lane: -1.0, kind: Kind::Notable, side: Side::Named(2), from: &[10] },
    T { at: 13, ring: 7.0, lane: 1.0, kind: Kind::Notable, side: Side::Named(3), from: &[11] },
    T { at: 14, ring: 7.6, lane: 0.0, kind: Kind::Small, side: Side::A, from: &[12, 13] },
    T { at: 15, ring: 8.5, lane: 0.0, kind: Kind::Keystone, side: Side::Named(4), from: &[14] },
];
/// The bridge between a region and the next clockwise: a small node at ring 5.5 between this
/// one's right lane and that one's left, and a notable past it at 6.5.
pub const BRIDGE_SMALL: u16 = 40;
pub const BRIDGE_NOTABLE: u16 = 41;
const BRIDGE_RINGS: (f32, f32) = (5.5, 6.5);

/// A named node: its key's tail and what it gives (a keystone: the family it gains and the one it
/// pays with, `web::KEYSTONE_UNITS` each).
pub struct Named {
    pub key: &'static str,
    pub gives: &'static [(Family, f32)],
    pub special: Option<Special>,
    pub keystone: Option<(Family, Family)>,
}
const fn n(key: &'static str, gives: &'static [(Family, f32)]) -> Named {
    Named { key, gives, special: None, keystone: None }
}
/// A keystone: a lot of one family for as much of another.
const fn k(key: &'static str, gain: Family, cost: Family) -> Named {
    Named { key, gives: &[], special: None, keystone: Some((gain, cost)) }
}

/// A region: its skill, its lanes' families, its four notables (left 4, right 4, left 7, right 7)
/// and its keystone, and the bridge notable to the next region clockwise.
pub struct Region {
    pub skill: Skill,
    pub a: Family,
    pub b: Family,
    pub named: [Named; 5],
    pub bridge: Named,
}

use Family::*;

/// Every region, in `skills::SKILLS` order.
pub const REGIONS: [Region; COUNT] = [
    Region {
        skill: Skill::Melee,
        a: MeleeDamage,
        b: Stamina,
        named: [
            n("melee.heavy_blows", &[(MeleeDamage, 3.0)]),
            n("melee.second_wind", &[(Stamina, 3.0)]),
            n("melee.executioner", &[(MeleeDamage, 4.0)]),
            n("melee.battle_rhythm", &[(Stamina, 2.0), (MeleeDamage, 2.0)]),
            k("melee.berserker", MeleeDamage, Defence),
        ],
        bridge: n("bridge.weapon_master", &[(MeleeDamage, 2.0), (RangedDamage, 2.0)]),
    },
    Region {
        skill: Skill::Archery,
        a: RangedDamage,
        b: MoveSpeed,
        named: [
            n("archery.steady_aim", &[(RangedDamage, 3.0)]),
            n("archery.light_feet", &[(MoveSpeed, 3.0)]),
            n("archery.piercing_shot", &[(RangedDamage, 4.0)]),
            n("archery.skirmisher", &[(MoveSpeed, 2.0), (RangedDamage, 2.0)]),
            k("archery.glass_arrow", RangedDamage, Health),
        ],
        bridge: n("bridge.shield_archer", &[(RangedDamage, 2.0), (Defence, 2.0)]),
    },
    Region {
        skill: Skill::Defence,
        a: Defence,
        b: Health,
        named: [
            n("defence.thick_hide", &[(Defence, 3.0)]),
            n("defence.hardy", &[(Health, 3.0)]),
            n("defence.shield_wall", &[(Defence, 4.0)]),
            n("defence.unbroken", &[(Health, 4.0)]),
            k("defence.iron_wall", Defence, MoveSpeed),
        ],
        bridge: n("bridge.armourer", &[(Defence, 2.0), (CraftGrade, 2.0)]),
    },
    Region {
        skill: Skill::Smithing,
        a: CraftGrade,
        b: Smelting,
        named: [
            n("smithing.keen_eye", &[(CraftGrade, 3.0)]),
            n("smithing.bellows", &[(Smelting, 3.0)]),
            n("smithing.masters_touch", &[(CraftGrade, 4.0)]),
            n("smithing.hot_forge", &[(Smelting, 4.0)]),
            k("smithing.forge_bound", CraftGrade, Stamina),
        ],
        bridge: n("bridge.machine_forged", &[(CraftGrade, 2.0), (MachineSpeed, 2.0)]),
    },
    Region {
        skill: Skill::Engineering,
        a: MachineSpeed,
        b: Smelting,
        named: [
            n("engineering.gearwright", &[(MachineSpeed, 3.0)]),
            n("engineering.steady_fire", &[(Smelting, 3.0)]),
            n("engineering.clockwork", &[(MachineSpeed, 4.0)]),
            n("engineering.pressure", &[(Smelting, 4.0)]),
            k("engineering.overclock", MachineSpeed, CraftGrade),
        ],
        bridge: n("bridge.steam_drill", &[(MachineSpeed, 2.0), (MineSpeed, 2.0)]),
    },
    Region {
        skill: Skill::Mining,
        a: MineSpeed,
        b: Stamina,
        named: [
            n("mining.stonecutter", &[(MineSpeed, 3.0)]),
            n("mining.strong_back", &[(Stamina, 3.0)]),
            n("mining.vein_sense", &[(MineSpeed, 4.0)]),
            n("mining.tireless", &[(Stamina, 4.0)]),
            k("mining.tunnel_rat", MineSpeed, Health),
        ],
        bridge: n("bridge.quarry_master", &[(MineSpeed, 2.0), (GatherSpeed, 2.0)]),
    },
    Region {
        skill: Skill::Building,
        a: Harvest,
        b: GatherSpeed,
        named: [
            n("building.green_thumb", &[(Harvest, 3.0)]),
            n("building.quick_hands", &[(GatherSpeed, 3.0)]),
            n("building.bountiful", &[(Harvest, 4.0)]),
            n("building.builders_eye", &[(GatherSpeed, 4.0)]),
            k("building.homesteader", Harvest, MoveSpeed),
        ],
        bridge: n("bridge.kitchen_garden", &[(Harvest, 2.0), (Cooking, 2.0)]),
    },
    Region {
        skill: Skill::Alchemy,
        a: Cooking,
        b: Hunger,
        named: [
            n("alchemy.hearty_meals", &[(Cooking, 3.0)]),
            n("alchemy.iron_stomach", &[(Hunger, 3.0)]),
            n("alchemy.fine_cook", &[(Cooking, 4.0)]),
            n("alchemy.lean_living", &[(Hunger, 4.0)]),
            k("alchemy.feaster", Cooking, Hunger),
        ],
        bridge: n("bridge.infuser", &[(Cooking, 2.0), (ManaCost, 2.0)]),
    },
    Region {
        skill: Skill::Runecraft,
        a: ManaCost,
        b: CraftGrade,
        named: [
            Named { key: "runecraft.ley_sight", gives: &[(ManaCost, 3.0)], special: Some(Special::LeySight), keystone: None },
            n("runecraft.rune_carver", &[(CraftGrade, 3.0)]),
            n("runecraft.deep_runes", &[(ManaCost, 4.0)]),
            n("runecraft.glyph_weaver", &[(CraftGrade, 4.0)]),
            k("runecraft.rune_bound", CraftGrade, ManaCost),
        ],
        bridge: n("bridge.charged_runes", &[(ManaCost, 2.0), (SpellDamage, 2.0)]),
    },
    Region {
        skill: Skill::Destruction,
        a: SpellDamage,
        b: ManaCost,
        named: [
            n("destruction.kindling", &[(SpellDamage, 3.0)]),
            n("destruction.focus", &[(ManaCost, 3.0)]),
            n("destruction.storm_call", &[(SpellDamage, 4.0)]),
            n("destruction.overflow", &[(ManaCost, 4.0)]),
            k("destruction.ley_burn", SpellDamage, ManaCost),
        ],
        bridge: n("bridge.balance", &[(SpellDamage, 2.0), (Health, 2.0)]),
    },
    Region {
        skill: Skill::Restoration,
        a: Health,
        b: ManaCost,
        named: [
            n("restoration.mending", &[(Health, 3.0)]),
            n("restoration.calm_mind", &[(ManaCost, 3.0)]),
            n("restoration.renewal", &[(Health, 4.0)]),
            n("restoration.wellspring", &[(ManaCost, 4.0)]),
            k("restoration.lifeblood", ManaCost, Health),
        ],
        bridge: n("bridge.warding_light", &[(Health, 2.0), (Defence, 2.0)]),
    },
    Region {
        skill: Skill::Conjuring,
        a: TameDamage,
        b: Defence,
        named: [
            n("conjuring.bound_spirit", &[(TameDamage, 3.0)]),
            n("conjuring.ward", &[(Defence, 3.0)]),
            n("conjuring.greater_summons", &[(TameDamage, 4.0)]),
            n("conjuring.bastion", &[(Defence, 4.0)]),
            k("conjuring.warden", Defence, SpellDamage),
        ],
        bridge: n("bridge.rune_collars", &[(TameDamage, 2.0), (Defence, 2.0)]),
    },
    Region {
        skill: Skill::Taming,
        a: TamingSpeed,
        b: RideSpeed,
        named: [
            n("taming.gentle_hand", &[(TamingSpeed, 3.0)]),
            n("taming.saddle_sense", &[(RideSpeed, 3.0)]),
            n("taming.pack_leader", &[(TameDamage, 4.0)]),
            n("taming.swift_mount", &[(RideSpeed, 4.0)]),
            k("taming.beastmaster", TameDamage, MeleeDamage),
        ],
        bridge: n("bridge.tracker", &[(TamingSpeed, 2.0), (GatherSpeed, 2.0)]),
    },
    Region {
        skill: Skill::Survival,
        a: GatherSpeed,
        b: Health,
        named: [
            n("survival.woodsman", &[(GatherSpeed, 3.0)]),
            n("survival.weathered", &[(Health, 3.0)]),
            n("survival.forager", &[(Hunger, 4.0)]),
            n("survival.hard_to_kill", &[(Health, 4.0)]),
            k("survival.wild_born", Hunger, Cooking),
        ],
        bridge: n("bridge.pathfinder", &[(Hunger, 2.0), (MoveSpeed, 2.0)]),
    },
    Region {
        skill: Skill::Exploring,
        a: MoveSpeed,
        b: Stamina,
        named: [
            n("exploring.long_stride", &[(MoveSpeed, 3.0)]),
            n("exploring.wayfarer", &[(Stamina, 3.0)]),
            n("exploring.pathless", &[(MoveSpeed, 4.0)]),
            n("exploring.far_walker", &[(Stamina, 4.0)]),
            k("exploring.wanderer", MoveSpeed, Defence),
        ],
        bridge: n("bridge.caravan", &[(MoveSpeed, 2.0), (Trade, 2.0)]),
    },
    Region {
        skill: Skill::Trading,
        a: Trade,
        b: RideSpeed,
        named: [
            n("trading.haggler", &[(Trade, 3.0)]),
            n("trading.pack_train", &[(RideSpeed, 3.0)]),
            n("trading.merchant_prince", &[(Trade, 4.0)]),
            n("trading.caravaneer", &[(RideSpeed, 4.0)]),
            k("trading.silver_tongue", Trade, MeleeDamage),
        ],
        bridge: n("bridge.sellsword", &[(Trade, 2.0), (MeleeDamage, 2.0)]),
    },
];

/// A region's slot: 1 + its skill's index (0 is the centre's).
pub const fn slot(skill: Skill) -> u16 {
    skill.index() as u16 + 1
}
/// A node's id from its region and place there.
pub const fn id(skill: Skill, at: u16) -> u16 {
    slot(skill) * REGION_SLOTS + at
}

fn region_angle(i: usize) -> f32 {
    i as f32 * std::f32::consts::TAU / COUNT as f32
}

/// The start hub's gates, one a group, and the family each gives (ids `GATE_FROM` up, in this
/// order).
pub const GATES_OF: [(crate::skills::Group, Family); 5] = [
    (crate::skills::Group::Fighting, Health),
    (crate::skills::Group::Crafting, CraftGrade),
    (crate::skills::Group::Life, Stamina),
    (crate::skills::Group::Beasts, TamingSpeed),
    (crate::skills::Group::Magic, ManaCost),
];
/// The first gate's id (1-4 are the centre sockets).
pub const GATE_FROM: u16 = 5;

/// A group's gate.
pub fn gate_id(g: crate::skills::Group) -> u16 {
    GATE_FROM + GATES_OF.iter().position(|x| x.0 == g).unwrap_or(0) as u16
}

/// The first junction's id (after the gates).
pub const JUNCTION_FROM: u16 = GATE_FROM + 5;

/// Runs of neighbouring regions of one group (two or more, by index in `REGIONS`): each shares
/// one road from its gate, splitting at a junction.
pub fn junctions() -> Vec<Vec<usize>> {
    let mut runs: Vec<Vec<usize>> = Vec::new();
    for (i, r) in REGIONS.iter().enumerate() {
        match runs.last_mut() {
            Some(run) if REGIONS[run[0]].skill.group() == r.skill.group() => run.push(i),
            _ => runs.push(vec![i]),
        }
    }
    runs.into_iter().filter(|r| r.len() > 1).collect()
}

/// What a region's entry hangs from: its junction, else its group's gate.
pub fn road_to(skill: Skill) -> u16 {
    let i = skill.index();
    junctions()
        .iter()
        .position(|run| run.contains(&i))
        .map_or(gate_id(skill.group()), |j| JUNCTION_FROM + j as u16)
}

/// A point `ring` rings out at `angle` (the first web's wheel).
fn polar(ring: f32, angle: f32) -> (f32, f32) {
    let r = radius(ring);
    (r * angle.sin(), -r * angle.cos())
}

fn build() -> Web {
    build_with(crate::web_layout::STYLE)
}

/// The web laid out in a style (`web_layout`): the game's is `web_layout::STYLE`; the sheet and the
/// layout pictures build the others to compare.
pub fn build_with(style: crate::web_layout::Style) -> Web {
    let mut nodes = vec![Node {
        id: START,
        kind: Kind::Small,
        region: None,
        bridge: None,
        ring: 0.0,
        x: 0.0,
        y: 0.0,
        gives: Vec::new(),
        special: None,
        key: "start".into(),
        needs: 0,
        socket: None,
    }];
    let mut links = Vec::new();
    for (i, r) in REGIONS.iter().enumerate() {
        let mid = region_angle(i);
        for t in &TEMPLATE {
            let (gives, key, special) = match t.side {
                Side::A => (vec![(r.a, 1.0)], format!("family.{}", r.a.id()), None),
                Side::B => (vec![(r.b, 1.0)], format!("family.{}", r.b.id()), None),
                Side::Named(j) if r.named[j].keystone.is_some() => {
                    let (gain, cost) = r.named[j].keystone.unwrap();
                    (vec![(gain, KEYSTONE_UNITS), (cost, -KEYSTONE_UNITS)], r.named[j].key.to_string(), None)
                }
                Side::Named(j) => (r.named[j].gives.to_vec(), r.named[j].key.to_string(), r.named[j].special),
            };
            let angle = mid + t.lane * LANE_PX / radius(t.ring);
            let (x, y) = polar(t.ring, angle);
            nodes.push(Node {
                id: id(r.skill, t.at),
                kind: t.kind,
                region: Some(r.skill),
                bridge: None,
                ring: t.ring,
                x,
                y,
                gives,
                special,
                key,
                needs: gate(t.ring),
                socket: None,
            });
            if t.from.is_empty() {
                // A region's entry hangs from its road: its group's junction where neighbouring
                // regions of one group share one, else its group's gate at the start hub.
                links.push((road_to(r.skill), id(r.skill, t.at)));
            }
            for &f in t.from {
                links.push((id(r.skill, f), id(r.skill, t.at)));
            }
        }
        // The bridge to the next region clockwise.
        let next = REGIONS[(i + 1) % COUNT].skill;
        let edge = mid + std::f32::consts::PI / COUNT as f32;
        let (small, notable) = (id(r.skill, BRIDGE_SMALL), id(r.skill, BRIDGE_NOTABLE));
        nodes.push(Node {
            id: small,
            kind: Kind::Small,
            region: Some(r.skill),
            bridge: Some(next),
            ring: BRIDGE_RINGS.0,
            x: polar(BRIDGE_RINGS.0, edge).0,
            y: polar(BRIDGE_RINGS.0, edge).1,
            gives: vec![(r.bridge.gives[0].0, 1.0)],
            special: None,
            key: format!("family.{}", r.bridge.gives[0].0.id()),
            needs: gate(BRIDGE_RINGS.0),
            socket: None,
        });
        nodes.push(Node {
            id: notable,
            kind: Kind::Notable,
            region: Some(r.skill),
            bridge: Some(next),
            ring: BRIDGE_RINGS.1,
            x: polar(BRIDGE_RINGS.1, edge).0,
            y: polar(BRIDGE_RINGS.1, edge).1,
            gives: r.bridge.gives.to_vec(),
            special: r.bridge.special,
            key: r.bridge.key.to_string(),
            needs: gate(BRIDGE_RINGS.1),
            socket: None,
        });
        links.push((id(r.skill, 8), small));
        links.push((id(next, 7), small));
        links.push((small, notable));
        // Its socket, deep in the region between the lanes (from the link across at 4.6).
        let deep = id(r.skill, REGION_SOCKET);
        nodes.push(socket_node(deep, Some(r.skill), None, REGION_SOCKET_RING, mid, 4 + i as u8));
        links.push((id(r.skill, 9), deep));
        // Every second bridge has one past its notable.
        if i % 2 == 0 {
            let by = id(r.skill, BRIDGE_SOCKET);
            nodes.push(socket_node(by, Some(r.skill), Some(next), BRIDGE_SOCKET_RING, edge, 20 + (i / 2) as u8));
            links.push((notable, by));
        }
    }
    // The start hub: a gate a group (Ronald, 11-05: "A, with clearer islands": a small start hub
    // with a handful of exits, not 16 spokes from the centre).
    for (g, &(group, fam)) in GATES_OF.iter().enumerate() {
        let (x, y) = polar(0.6, g as f32 * std::f32::consts::TAU / GATES_OF.len() as f32);
        nodes.push(Node {
            id: gate_id(group),
            kind: Kind::Small,
            region: None,
            bridge: None,
            ring: 0.6,
            x,
            y,
            gives: vec![(fam, 1.0)],
            special: None,
            key: format!("family.{}", fam.id()),
            needs: 0,
            socket: None,
        });
        links.push((START, gate_id(group)));
    }
    // The junctions: a road from a gate splitting near the islands, giving the gate's family.
    for (j, run) in junctions().iter().enumerate() {
        let group = REGIONS[run[0]].skill.group();
        let fam = GATES_OF
            .iter()
            .find(|g| g.0 == group)
            .map_or(Family::Stamina, |g| g.1);
        let angle = run.iter().map(|&i| region_angle(i)).sum::<f32>() / run.len() as f32;
        let (x, y) = polar(1.5, angle);
        nodes.push(Node {
            id: JUNCTION_FROM + j as u16,
            kind: Kind::Small,
            region: None,
            bridge: None,
            ring: 1.5,
            x,
            y,
            gives: vec![(fam, 1.0)],
            special: None,
            key: format!("family.{}", fam.id()),
            needs: 0,
            socket: None,
        });
        links.push((gate_id(group), JUNCTION_FROM + j as u16));
    }
    // Four near the centre, between region pairs a quarter round apart, from both entries.
    for (j, &r) in CENTRE_SOCKETS.iter().enumerate() {
        let sid = 1 + j as u16;
        let angle = region_angle(r) + std::f32::consts::PI / COUNT as f32;
        nodes.push(socket_node(sid, None, None, CENTRE_SOCKET_RING, angle, j as u8));
        links.push((id(REGIONS[r].skill, 2), sid));
        links.push((id(REGIONS[(r + 1) % COUNT].skill, 1), sid));
    }
    let mut arcs = crate::web_layout::Arcs::new();
    crate::web_layout::lay(style, &mut nodes, &mut links, &mut arcs);
    let index = nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
    let mut near: HashMap<u16, Vec<u16>> = HashMap::new();
    for &(a, b) in &links {
        near.entry(a).or_default().push(b);
        near.entry(b).or_default().push(a);
    }
    Web { nodes, links, arcs, index, near }
}

/// Where the sockets are: deep in each region (its place 20, ring 5.6), past every second
/// bridge's notable (place 42, ring 7.3), and four near the centre (ids 1-4, ring 2.6) between
/// the regions after these, reached from either side's ring-2 lane node.
pub const REGION_SOCKET: u16 = 20;
pub const BRIDGE_SOCKET: u16 = 42;
const REGION_SOCKET_RING: f32 = 5.6;
const BRIDGE_SOCKET_RING: f32 = 7.3;
const CENTRE_SOCKET_RING: f32 = 2.6;
const CENTRE_SOCKETS: [usize; 4] = [1, 5, 9, 13];
/// Sockets a saved build holds stones for (28 used; the rest room for more at the rim).
pub const SOCKETS: usize = 48;

fn socket_node(id: u16, region: Option<Skill>, bridge: Option<Skill>, ring: f32, angle: f32, socket: u8) -> Node {
    let (x, y) = polar(ring, angle);
    Node {
        id,
        kind: Kind::Socket,
        region,
        bridge,
        ring,
        x,
        y,
        gives: Vec::new(),
        special: None,
        key: "socket".into(),
        needs: if region.is_some() { gate(ring) } else { 0 },
        socket: Some(socket),
    }
}

/// The web, built once.
pub fn web() -> &'static Web {
    static WEB: OnceLock<Web> = OnceLock::new();
    WEB.get_or_init(build)
}

// ------------------------------------------------------------------ a build

/// A player's taken nodes, a bit each by id (the start always counts as taken).
pub type Taken = [u64; WORDS];

pub fn has(taken: &Taken, id: u16) -> bool {
    id == START
        || taken
            .get(id as usize / 64)
            .is_some_and(|w| w & 1 << (id % 64) != 0)
}
pub fn set(taken: &mut Taken, id: u16, on: bool) {
    if let Some(w) = taken.get_mut(id as usize / 64) {
        if on {
            *w |= 1 << (id % 64);
        } else {
            *w &= !(1 << (id % 64));
        }
    }
}
/// The nodes taken, the start not counted.
pub fn count(taken: &Taken) -> u32 {
    web()
        .nodes
        .iter()
        .filter(|n| n.id != START && has(taken, n.id))
        .count() as u32
}

/// Points a node costs: one, a socket `SOCKET_POINTS`.
pub fn cost(n: &Node) -> u32 {
    if n.kind == Kind::Socket { SOCKET_POINTS } else { 1 }
}
/// Points a socket costs (a stone is worth more than a node: the dominance check found a socket
/// at one point in every build).
pub const SOCKET_POINTS: u32 = 3;

/// Points spent on a build.
pub fn spent(taken: &Taken) -> u32 {
    web()
        .nodes
        .iter()
        .filter(|n| n.id != START && has(taken, n.id))
        .map(cost)
        .sum()
}

/// Whether a special is in the build.
pub fn has_special(taken: &Taken, s: Special) -> bool {
    web()
        .nodes
        .iter()
        .any(|n| n.special == Some(s) && has(taken, n.id))
}

/// Units a build gives each family (`web::FAMILIES` order).
pub fn units(taken: &Taken) -> [f32; FAMILIES.len()] {
    let mut u = [0.0; FAMILIES.len()];
    for n in &web().nodes {
        if has(taken, n.id) {
            for &(f, x) in &n.gives {
                u[f.index()] += x;
            }
        }
    }
    u
}

/// The stones in a build's sockets, by socket index (`Node::socket`).
pub type Stones = [Option<Stone>; SOCKETS];

fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).hypot(a.1 - b.1) <= RADIUS
}

/// Units a build gives each family with the stones in its sockets taken: a mind-stone's rolls, a
/// flow-stone's work on the small nodes taken in its reach, a relic's own.
pub fn units_with(taken: &Taken, stones: &Stones) -> [f32; FAMILIES.len()] {
    let web = web();
    let mut u = units(taken);
    for n in &web.nodes {
        let Some(i) = n.socket else { continue };
        let Some(st) = stones.get(i as usize).copied().flatten() else { continue };
        if !has(taken, n.id) {
            continue;
        }
        for &(f, x) in st.rolls.iter().flatten() {
            u[f.index()] += x;
        }
        let here = n.pos();
        let reach = || {
            web.nodes
                .iter()
                .filter(move |m| m.id != START && m.id != n.id && has(taken, m.id) && near(m.pos(), here))
        };
        match (st.kind, st.flow) {
            (_, Some(Flow::Amplify(x))) => {
                for m in reach().filter(|m| m.kind == Kind::Small) {
                    for &(f, g) in &m.gives {
                        u[f.index()] += g * x;
                    }
                }
            }
            (_, Some(Flow::Turn(to))) => {
                for m in reach().filter(|m| m.kind == Kind::Small) {
                    for &(f, g) in &m.gives {
                        u[f.index()] -= g;
                        u[to.index()] += g;
                    }
                }
            }
            (StoneKind::Unique(Unique::Remaru), _) => u[Family::ManaCost.index()] += sockets::REMARU_UNITS,
            (StoneKind::Unique(Unique::Emakaun), _) => {
                u[Family::Health.index()] += reach().count() as f32 * sockets::LIFEKEEP_PER_NODE;
            }
            _ => {}
        }
    }
    u
}

/// `times` with the stones counted.
pub fn times_with(taken: &Taken, stones: &Stones, f: Family) -> f32 {
    family_signed(units_with(taken, stones)[f.index()])
}

/// `has_special` with the relics counted (Remaru gives ley sight).
pub fn has_special_with(taken: &Taken, stones: &Stones, s: Special) -> bool {
    has_special(taken, s)
        || (s == Special::LeySight
            && web().nodes.iter().any(|n| {
                n.socket.is_some_and(|i| {
                    has(taken, n.id)
                        && stones
                            .get(i as usize)
                            .copied()
                            .flatten()
                            .is_some_and(|st| st.kind == StoneKind::Unique(Unique::Remaru))
                })
            }))
}

/// What a build multiplies a family by (capped at one age, a keystone's cost down to the floor).
pub fn times(taken: &Taken, f: Family) -> f32 {
    family_signed(units(taken)[f.index()])
}

/// Why a node can't be taken now, or `None` if it can.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Taken,
    Unknown,
    NoPoints,
    NotNextTo,
    Skill,
}

pub fn can_take(taken: &Taken, id: u16, levels: &[u32; COUNT], points: u32) -> Result<(), Refusal> {
    let web = web();
    let Some(n) = web.node(id) else { return Err(Refusal::Unknown) };
    if has(taken, id) {
        return Err(Refusal::Taken);
    }
    if !web.next_to(id).iter().any(|&o| has(taken, o)) {
        return Err(Refusal::NotNextTo);
    }
    if !n.open_at(levels) {
        return Err(Refusal::Skill);
    }
    if points < cost(n) {
        return Err(Refusal::NoPoints);
    }
    Ok(())
}

/// Whether a taken node can be refunded: the rest all still reach the start without it.
pub fn can_refund(taken: &Taken, id: u16) -> bool {
    if id == START || !has(taken, id) {
        return false;
    }
    let mut without = *taken;
    set(&mut without, id, false);
    let reached = reach(&without);
    web()
        .nodes
        .iter()
        .all(|n| !has(&without, n.id) || reached.contains(&n.id))
}

/// The taken nodes that reach the start through taken nodes.
fn reach(taken: &Taken) -> HashSet<u16> {
    let web = web();
    let mut seen = HashSet::from([START]);
    let mut todo = VecDeque::from([START]);
    while let Some(at) = todo.pop_front() {
        for &o in web.next_to(at) {
            if has(taken, o) && seen.insert(o) {
                todo.push_back(o);
            }
        }
    }
    seen
}

// ------------------------------------------------------------------ the checks

/// The most a region gives in all its families together: an age and a quarter (the speciality
/// rule, one age in any one family, is checked family by family; this bounds a bigger region's
/// travel nodes and its bridge).
pub const REGION_TOTAL: f32 = 37.5;

/// Units each region gives (its own nodes and its bridges), nets (keystones count as nothing).
pub fn region_units(skill: Skill) -> f32 {
    web()
        .nodes
        .iter()
        .filter(|n| n.region == Some(skill))
        .map(|n| n.net())
        .sum()
}

/// Every problem the web has; empty when it's sound.
pub fn problems() -> Vec<String> {
    let web = web();
    let mut out = Vec::new();
    // Ids are unique and in their region's slots.
    let mut ids = HashSet::new();
    for n in &web.nodes {
        if !ids.insert(n.id) {
            out.push(format!("node id {} twice", n.id));
        }
        if let Some(r) = n.region
            && n.id / REGION_SLOTS != slot(r)
        {
            out.push(format!("{} is outside {}'s slots", n.key, r.id()));
        }
    }
    if web.nodes.len() as u16 > REGION_SLOTS * WORDS as u16 {
        out.push("more nodes than a saved build holds".into());
    }
    // Every node reachable from the start, rising in skill: never behind a node needing more.
    let mut need: HashMap<u16, u32> = HashMap::from([(START, 0)]);
    let mut todo = VecDeque::from([START]);
    while let Some(at) = todo.pop_front() {
        let here = need[&at];
        for &o in web.next_to(at) {
            let n = web.node(o).unwrap();
            let v = here.max(n.needs);
            if need.get(&o).is_none_or(|&old| v < old) {
                need.insert(o, v);
                todo.push_back(o);
            }
        }
    }
    for n in &web.nodes {
        match need.get(&n.id) {
            None => out.push(format!("{} can't be reached", n.key)),
            Some(&v) if v > n.needs => out.push(format!("{} needs skill {} but sits behind {}", n.key, n.needs, v)),
            _ => {}
        }
    }
    // Prices: a small node one unit, a notable 3-4, a keystone as much cost as power.
    for n in &web.nodes {
        if n.id == START {
            continue;
        }
        let (lo, hi) = n.kind.units();
        match n.kind {
            Kind::Keystone => {
                if n.net().abs() > 1e-4 || n.gain() != KEYSTONE_UNITS {
                    out.push(format!("keystone {} nets {} (should be 0, power {KEYSTONE_UNITS})", n.key, n.net()));
                }
            }
            _ if n.net() < lo - 1e-4 || n.net() > hi + 1e-4 => {
                out.push(format!("{} gives {} units, its kind {lo}-{hi}", n.key, n.net()));
            }
            _ => {}
        }
        if n.gives.is_empty() && n.kind != Kind::Socket {
            out.push(format!("{} gives nothing", n.key));
        }
    }
    // A region's budget: everything in it together at most one age in its speciality.
    for s in SKILLS {
        let u = region_units(s);
        if u > REGION_TOTAL + 1e-4 {
            out.push(format!("region {} gives {u} units, over its budget of {REGION_TOTAL}", s.id()));
        }
        for f in FAMILIES {
            let fu: f32 = web
                .nodes
                .iter()
                .filter(|n| n.region == Some(s) && n.kind != Kind::Keystone)
                .flat_map(|n| n.gives.iter())
                .filter(|g| g.0 == f)
                .map(|g| g.1)
                .sum();
            // Every socket of the region holding the best stone for this family.
            let fu = fu + region_sockets(s) as f32 * sockets::best_units();
            if fu > units_to_ceiling() + 1e-4 {
                out.push(format!("region {} gives {fu} units of {}", s.id(), f.name()));
            }
        }
    }
    let d = dominance();
    out.extend(d.problems);
    out
}

// ------------------------------------------------------------------ build dominance

/// A job a build is scored on: how much each family is worth to it.
pub struct Job {
    pub name: &'static str,
    pub weights: &'static [(Family, f32)],
}

/// The jobs (`reports/Balance model.md` 5.5, "Build dominance"): one per way to play today and
/// tomorrow.
pub const JOBS: [Job; 8] = [
    Job { name: "fighter", weights: &[(MeleeDamage, 1.0), (Defence, 0.7), (Health, 0.5), (Stamina, 0.3)] },
    Job { name: "archer", weights: &[(RangedDamage, 1.0), (MoveSpeed, 0.6), (Health, 0.3)] },
    Job { name: "tamer", weights: &[(TameDamage, 1.0), (TamingSpeed, 0.8), (RideSpeed, 0.5)] },
    Job { name: "miner", weights: &[(MineSpeed, 1.0), (Stamina, 0.5), (GatherSpeed, 0.4)] },
    Job { name: "smith", weights: &[(CraftGrade, 1.0), (Smelting, 0.6), (MachineSpeed, 0.3)] },
    Job { name: "farmer and cook", weights: &[(Harvest, 1.0), (Cooking, 0.8), (Hunger, 0.6)] },
    Job { name: "mage", weights: &[(SpellDamage, 1.0), (ManaCost, 0.8), (Health, 0.3)] },
    Job { name: "explorer", weights: &[(MoveSpeed, 1.0), (Stamina, 0.6), (Hunger, 0.5), (Trade, 0.3)] },
];

/// Points the dominance check spends (about level 60) and the skill it assumes everywhere (the
/// Iron Age's end: keystones open).
pub const CHECK_POINTS: u32 = 60;
pub const CHECK_SKILL: u32 = 45;

/// A build's score on a job: the log of what its families multiply, weighted (so ×1.2 in one and
/// ×1.2 in another is worth as much as ×1.44 in one).
pub fn score(taken: &Taken, job: &Job) -> f32 {
    let mut u = units(taken);
    let worth = |u: &[f32; FAMILIES.len()]| -> f32 {
        job.weights
            .iter()
            .map(|&(f, w)| w * family_signed(u[f.index()]).ln())
            .sum()
    };
    // Each socket taken holds the best stone for the job: a top-level mind-stone in whichever
    // family gains it most (stones swap freely, so a build always has its best).
    let sockets = web()
        .nodes
        .iter()
        .filter(|n| n.socket.is_some() && has(taken, n.id))
        .count();
    for _ in 0..sockets {
        let best = job
            .weights
            .iter()
            .map(|&(f, _)| {
                let mut t = u;
                t[f.index()] += sockets::best_units();
                (worth(&t), f)
            })
            .max_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((_, f)) = best {
            u[f.index()] += sockets::best_units();
        }
    }
    worth(&u)
}

/// The middle of a region's island (the average of its nodes, its bridge's left out).
pub fn region_centre(skill: Skill) -> (f32, f32) {
    let ns: Vec<&Node> = web()
        .nodes
        .iter()
        .filter(|n| n.region == Some(skill) && n.bridge.is_none())
        .collect();
    let k = ns.len().max(1) as f32;
    (ns.iter().map(|n| n.x).sum::<f32>() / k, ns.iter().map(|n| n.y).sum::<f32>() / k)
}

/// Sockets in a region (its own and its bridge's).
pub fn region_sockets(skill: Skill) -> usize {
    web()
        .nodes
        .iter()
        .filter(|n| n.region == Some(skill) && n.socket.is_some())
        .count()
}

/// The dominance check's result.
pub struct Dominance {
    /// Each job's hill-climbed build, and its score on every job.
    pub builds: Vec<(Taken, Vec<f32>)>,
    /// How many of the builds take each node (the start not counted).
    pub picks: Vec<(u16, usize)>,
    pub problems: Vec<String>,
}

/// Hill-climb a build for a job: from the start, repeatedly take the best nearby path (one to
/// three nodes) by score gained per point, until `points` are spent or nothing helps.
pub fn climb(job: &Job, points: u32, skill: u32) -> Taken {
    let levels = [skill; COUNT];
    let web = web();
    let mut taken: Taken = [0; WORDS];
    let mut left = points;
    while left > 0 {
        let base = score(&taken, job);
        let mut best: Option<(f32, Vec<u16>)> = None;
        // Paths of up to three new nodes out from the build.
        let mut paths: Vec<Vec<u16>> = Vec::new();
        for n in &web.nodes {
            if can_take(&taken, n.id, &levels, SOCKET_POINTS).is_ok() {
                paths.push(vec![n.id]);
            }
        }
        for _ in 1..3 {
            let mut more = Vec::new();
            for p in &paths {
                if path_cost(p) >= left {
                    continue;
                }
                let mut t = taken;
                for &i in p {
                    set(&mut t, i, true);
                }
                let last = *p.last().unwrap();
                for &o in web.next_to(last) {
                    if !has(&t, o) && web.node(o).unwrap().open_at(&levels) {
                        let mut q = p.clone();
                        q.push(o);
                        more.push(q);
                    }
                }
            }
            paths.extend(more);
        }
        for p in paths {
            if path_cost(&p) > left {
                continue;
            }
            let mut t = taken;
            for &i in &p {
                set(&mut t, i, true);
            }
            let per = (score(&t, job) - base) / path_cost(&p) as f32;
            if best.as_ref().is_none_or(|b| per > b.0 + 1e-6) {
                best = Some((per, p));
            }
        }
        // Nothing left helps this job: the rest of the points stay unspent.
        let Some((per, p)) = best else { break };
        if per <= 1e-6 {
            break;
        }
        left -= path_cost(&p);
        for i in p {
            set(&mut taken, i, true);
        }
    }
    taken
}

fn path_cost(p: &[u16]) -> u32 {
    p.iter().map(|&i| web().node(i).map_or(1, cost)).sum()
}

/// Runs the build dominance check.
pub fn dominance() -> Dominance {
    let builds: Vec<Taken> = JOBS
        .iter()
        .map(|j| climb(j, CHECK_POINTS, CHECK_SKILL))
        .collect();
    let scored: Vec<(Taken, Vec<f32>)> = builds
        .iter()
        .map(|b| (*b, JOBS.iter().map(|j| score(b, j)).collect()))
        .collect();
    let mut problems = Vec::new();
    for (i, (_, s)) in scored.iter().enumerate() {
        let wins_all = (0..JOBS.len()).all(|j| {
            let others = scored
                .iter()
                .enumerate()
                .filter(|(o, _)| *o != i)
                .map(|(_, b)| b.1[j])
                .fold(f32::MIN, f32::max);
            // Scores are logs: winning by 15 % is ln(1.15) more.
            s[j] > others + 1.15f32.ln()
        });
        if wins_all {
            problems.push(format!("the {} build wins every job by over 15 %", JOBS[i].name));
        }
    }
    let mut picks: Vec<(u16, usize)> = web()
        .nodes
        .iter()
        .filter(|n| n.id != START)
        .map(|n| (n.id, builds.iter().filter(|b| has(b, n.id)).count()))
        .filter(|p| p.1 > 0)
        .collect();
    picks.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for &(id, c) in &picks {
        if c as f32 > 0.9 * builds.len() as f32 {
            problems.push(format!("{} is in {c} of {} builds", web().node(id).unwrap().key, builds.len()));
        }
    }
    Dominance { builds: scored, picks, problems }
}

/// The skill levels from effort minutes per skill.
pub fn levels(em: &[f32; COUNT]) -> [u32; COUNT] {
    std::array::from_fn(|i| skills::level_of(em[i]).0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_web_is_sound() {
        let p = problems();
        assert!(p.is_empty(), "{}", p.join("\n"));
    }

    #[test]
    fn taking_and_refunding() {
        let mut t: Taken = [0; WORDS];
        let low = [0; COUNT];
        // From the start the road runs through the group's gate (and a junction where regions
        // share one) to the island's entry (the islands web, 11-05).
        let gate = gate_id(Skill::Melee.group());
        let entry = id(Skill::Melee, 0);
        assert_eq!(can_take(&t, entry, &low, 5), Err(Refusal::NotNextTo));
        assert_eq!(can_take(&t, gate, &low, 0), Err(Refusal::NoPoints));
        assert!(can_take(&t, gate, &low, 1).is_ok());
        set(&mut t, gate, true);
        let road: Vec<u16> = web()
            .next_to(entry)
            .iter()
            .copied()
            .filter(|&o| o >= JUNCTION_FROM && web().next_to(o).contains(&gate))
            .collect();
        for &j in &road {
            set(&mut t, j, true);
        }
        assert!(can_take(&t, entry, &low, 1).is_ok());
        set(&mut t, entry, true);
        set(&mut t, id(Skill::Melee, 1), true);
        set(&mut t, id(Skill::Melee, 3), true);
        assert_eq!(can_take(&t, id(Skill::Melee, 5), &low, 1), Err(Refusal::Skill));
        assert!(!can_refund(&t, entry) && !can_refund(&t, gate));
        assert!(can_refund(&t, id(Skill::Melee, 3)));
        assert!((times(&t, Family::MeleeDamage) - 1.06).abs() < 1e-5);
        assert_eq!(count(&t), 4 + road.len() as u32);
    }
}
