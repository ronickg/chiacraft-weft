//! The ancients' ruins and the knowledge gate (`notes/PROGRESSION.md`, "How an age is gated":
//! "Knowledge: a blueprint or lore page found in that age's ruins"; Planner, 11-23: crucible, iron
//! smelting and forge locked, the Bronze and Iron vaults sealed by harvest, one blueprint per
//! player per pedestal, one Stone ruin near spawn).
//!
//! - **Blueprints** (`BLUEPRINTS`): each teaches a player the next age's key recipe. Until it is
//!   read, the crafting grid and the book won't make what it teaches, and a furnace won't take
//!   the raw ore it teaches to smelt. A blueprint is kept when read, so it can be passed on.
//! - **Ruins** (`KINDS`): the ancients built along their ley lines (`crate::ley`), so a ruin stands
//!   on a line where one passes its cell. A Stone ruin is an open ring; a Bronze ruin a
//!   half-buried vault, sealed with ward stone a copper pickaxe breaks; an Iron ruin a vault deep
//!   under a shrine, sealed with ward stone a bronze pickaxe breaks (`Kind::ward`). Each holds a
//!   pedestal that gives every player its blueprint once.
//! - **The first ruin** (`spawn_ruin`): one Stone ruin always stands within `SPAWN_REACH` blocks
//!   of the world's spawn, so no new player is kept from bronze by the seed.
//!
//! Everything here comes from the seed and the spawn alone, so the server, `/ruin locate` and the
//! checks agree without generating anything.

use crate::ley;

/// What a blueprint teaches: crafting recipes by their output, and raw ores the furnace smelts.
#[derive(Clone, Copy, Debug)]
pub struct Blueprint {
    /// Its id: the item is `base:blueprint_<id>`, its bit in a player's knowledge its index.
    pub id: &'static str,
    /// The age (tier) whose ruins hold it.
    pub age: u8,
    /// Crafting recipes' outputs it unlocks.
    pub crafts: &'static [&'static str],
    /// Raw ores a furnace takes only once it is known.
    pub smelts: &'static [&'static str],
}

impl Blueprint {
    pub fn item(&self) -> String {
        format!("base:blueprint_{}", self.id)
    }
    /// Its bit in a player's knowledge.
    pub fn bit(&self) -> u32 {
        1 << BLUEPRINTS.iter().position(|b| b.id == self.id).unwrap_or(0)
    }
}

/// The blueprints, one per age key so far: the Stone ruins' crucible (bronze), the Bronze ruins'
/// iron smelting, the Iron ruins' forge (steel).
pub const BLUEPRINTS: &[Blueprint] = &[
    Blueprint { id: "crucible", age: 0, crafts: &["base:crucible"], smelts: &[] },
    Blueprint { id: "ironwork", age: 1, crafts: &[], smelts: &["base:raw_iron"] },
    Blueprint { id: "forge", age: 2, crafts: &["base:forge"], smelts: &[] },
];

/// The blueprint by its id.
pub fn blueprint(id: &str) -> Option<&'static Blueprint> {
    BLUEPRINTS.iter().find(|b| b.id == id)
}

/// The blueprint that must be known to craft `output`, if any.
pub fn craft_lock(output: &str) -> Option<&'static Blueprint> {
    BLUEPRINTS.iter().find(|b| b.crafts.contains(&output))
}

/// The blueprint that must be known to smelt `input`, if any.
pub fn smelt_lock(input: &str) -> Option<&'static Blueprint> {
    BLUEPRINTS.iter().find(|b| b.smelts.contains(&input))
}

/// Whether knowledge bits `known` hold `b`.
pub fn knows(known: u32, b: &Blueprint) -> bool {
    known & b.bit() != 0
}

/// Every blueprint's bits: what creative and `/ruin learn all` know.
pub fn all_bits() -> u32 {
    BLUEPRINTS.iter().fold(0, |a, b| a | b.bit())
}

/// A kind of ruin, one per age so far.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kind {
    /// Its id (`/ruin locate`, `/ruin goto <id>`, lang `ruin.base.<id>`).
    pub id: &'static str,
    /// The age whose blueprint it holds.
    pub age: u8,
    /// Its grid: one chance of a ruin per cell of this many blocks.
    pub cell: i32,
    /// The chance a cell has one (where a line passes near enough).
    pub chance: f32,
    /// How far from the cell's own spot a line may be to draw the ruin onto it (blocks); with no
    /// line that near the cell has none.
    pub pull: f64,
    /// The harvest (`abi::tool_tier`, `crate::gear::harvest`) its ward stone takes; 0: open.
    pub ward: u32,
    /// How far below the ground its vault's floor lies (blocks).
    pub depth: i32,
    /// What finding it and getting in costs, in effort minutes (the blueprint's price), by hand:
    /// the walk to one at this spacing with the rod or a map (a ring a few minutes from spawn), the
    /// way in (a vault's roof broken, a deep vault's stair) and the dangers of the far land; about
    /// a boss part's price at the age past the Stone (Borkon's Summit Stone 60, an Iron form's 80).
    pub find_em: f32,
}

/// The ruins: the Stone Age's open rings, the Bronze Age's half-buried vaults (a copper
/// pickaxe's harvest, `gear::harvest` of copper's level, 170), the Iron Age's deep vaults (a
/// bronze pickaxe's, 210, as iron ore).
pub const KINDS: &[Kind] = &[
    Kind { id: "ring", age: 0, cell: 384, chance: 0.7, pull: 140.0, ward: 0, depth: 0, find_em: 12.0 },
    Kind { id: "vault", age: 1, cell: 768, chance: 0.75, pull: 200.0, ward: WARD_BRONZE, depth: 4, find_em: 30.0 },
    Kind { id: "deep", age: 2, cell: 1024, chance: 0.8, pull: 260.0, ward: WARD_IRON, depth: 22, find_em: 45.0 },
];

/// The Bronze vaults' ward: a copper pickaxe breaks it (copper's harvest).
pub const WARD_BRONZE: u32 = 170;
/// The Iron vaults' ward: a bronze pickaxe breaks it (bronze's, iron ore's own).
pub const WARD_IRON: u32 = crate::gear::IRON_ORE_HARVEST;

/// The kind by id.
pub fn kind(id: &str) -> Option<&'static Kind> {
    KINDS.iter().find(|k| k.id == id)
}

/// The kind whose ruins hold `age`'s blueprint.
pub fn kind_of_age(age: u8) -> Option<&'static Kind> {
    KINDS.iter().find(|k| k.age == age)
}

/// The first Stone ruin stands at most this far from the world's spawn (blocks; Planner, 11-23:
/// "within about 300 blocks of world spawn").
pub const SPAWN_REACH: f64 = 300.0;
/// Where it stands with no line near: this far from spawn, the way the seed says (blocks): far
/// enough to be a walk, near enough to see from a hill.
pub const SPAWN_DIST: f64 = 160.0;
/// No two ruins stand nearer than this (blocks): a deeper kind keeps the lighter ones away, and
/// none sits on a waystone's node.
pub const APART: f64 = 48.0;
/// How far a ruin reaches from its centre (blocks, each axis): what a cube asks about.
pub const REACH: i32 = 6;

/// One ruin: its kind (a `KINDS` index), where it stands (its centre's column) and its cell
/// (the spawn's ruin has cell `(i32::MIN, i32::MIN)`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ruin {
    pub kind: usize,
    pub x: i32,
    pub z: i32,
    pub cell: (i32, i32),
}

impl Ruin {
    pub fn kind(&self) -> &'static Kind {
        &KINDS[self.kind]
    }
    /// Its turn, a quarter 0-3 from where it stands (doors and stairs face that way).
    pub fn turn(&self) -> u8 {
        (mix((self.x as u32 as u64) | ((self.z as u32 as u64) << 32)) & 3) as u8
    }
    /// A number for it that the pedestal's givings are kept by.
    pub fn key(&self) -> u32 {
        let h = mix((self.x as u32 as u64) | ((self.z as u32 as u64) << 32) ^ ((self.kind as u64) << 60));
        (h as u32) | 1
    }
    fn dist(&self, x: f64, z: f64) -> f64 {
        ((self.x as f64 - x).powi(2) + (self.z as f64 - z).powi(2)).sqrt()
    }
}

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn hash(seed: u64, i: i32, j: i32, salt: u64) -> u64 {
    mix(seed ^ mix((i as u32 as u64) | ((j as u32 as u64) << 32)) ^ mix(salt))
}

const SALT_RUIN: u64 = 0x50A1_0001;
const SALT_SPAWN: u64 = 0x50A1_0002;

/// The nearest point on a ley line within `reach` of (`x`, `z`), not on a node.
fn on_line(seed: u64, x: f64, z: f64, reach: f64) -> Option<[f64; 2]> {
    ley::lines_near(seed, x as i32, z as i32, reach as i32 + 1)
        .iter()
        .map(|l| {
            // Not on a node: along the line, at least `APART` from either end.
            let len = l.length().max(1.0);
            let (at, _) = l.nearest(x, z);
            let t =
                ((at[0] - l.from[0] as f64) * l.dir()[0] + (at[1] - l.from[1] as f64) * l.dir()[1]).clamp(APART, (len - APART).max(APART));
            let p = [l.from[0] as f64 + l.dir()[0] * t, l.from[1] as f64 + l.dir()[1] * t];
            let d = ((p[0] - x).powi(2) + (p[1] - z).powi(2)).sqrt();
            (p, d)
        })
        .filter(|&(_, d)| d <= reach)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(p, _)| p)
}

/// Kind `k`'s ruin in cell (`i`, `j`), if it has one, before others keep it away.
fn in_cell(seed: u64, k: usize, i: i32, j: i32) -> Option<Ruin> {
    let kind = &KINDS[k];
    let h = hash(seed, i, j, SALT_RUIN + k as u64);
    if (h % 1000) as f32 >= kind.chance * 1000.0 {
        return None;
    }
    let inset = kind.cell / 6;
    let span = (kind.cell - 2 * inset) as u64;
    let x = (i * kind.cell + inset + ((h >> 10) % span) as i32) as f64;
    let z = (j * kind.cell + inset + ((h >> 36) % span) as i32) as f64;
    let [px, pz] = on_line(seed, x, z, kind.pull)?;
    Some(Ruin { kind: k, x: px.round() as i32, z: pz.round() as i32, cell: (i, j) })
}

/// Kind `k`'s ruins (by cell) within `r` blocks (each axis) of (`x`, `z`), before others keep them
/// away.
fn raw_near(seed: u64, k: usize, x: i32, z: i32, r: i32) -> Vec<Ruin> {
    let c = KINDS[k].cell;
    let mut out = Vec::new();
    for j in (z - r).div_euclid(c)..=(z + r).div_euclid(c) {
        for i in (x - r).div_euclid(c)..=(x + r).div_euclid(c) {
            if let Some(ruin) = in_cell(seed, k, i, j)
                && (ruin.x - x).abs() <= r
                && (ruin.z - z).abs() <= r
            {
                out.push(ruin);
            }
        }
    }
    out
}

/// Whether `ruin` stands: no deeper kind's ruin, no waystone and not the spawn's ruin (`first`)
/// within `APART` of it.
fn stands(seed: u64, ruin: &Ruin, first: Option<Ruin>) -> bool {
    let reach = APART as i32 + 1;
    let deeper = (ruin.kind + 1..KINDS.len()).any(|k| {
        raw_near(seed, k, ruin.x, ruin.z, reach)
            .iter()
            .any(|o| o.dist(ruin.x as f64, ruin.z as f64) < APART)
    });
    let stone = ley::waystones_near(seed, ruin.x, ruin.z, reach)
        .iter()
        .any(|n| ruin.dist(n.x as f64, n.z as f64) < APART);
    let near_first = first.is_some_and(|f| f.dist(ruin.x as f64, ruin.z as f64) < APART);
    !deeper && !stone && !near_first
}

/// Every ruin within `r` blocks (each axis) of (`x`, `z`), the spawn's (`first`, from
/// `spawn_ruin`) among them when given and near.
pub fn ruins_near(seed: u64, first: Option<Ruin>, x: i32, z: i32, r: i32) -> Vec<Ruin> {
    let mut out: Vec<Ruin> = (0..KINDS.len())
        .flat_map(|k| raw_near(seed, k, x, z, r))
        .filter(|ruin| stands(seed, ruin, first))
        .collect();
    if let Some(first) = first
        && (first.x - x).abs() <= r
        && (first.z - z).abs() <= r
    {
        out.push(first);
    }
    out
}

/// The nearest ruin of kind `k` to (`x`, `z`), looking `cells` of its grid out.
pub fn nearest(seed: u64, first: Option<Ruin>, k: usize, x: i32, z: i32, cells: i32) -> Option<Ruin> {
    ruins_near(seed, first, x, z, KINDS[k].cell * cells)
        .into_iter()
        .filter(|r| r.kind == k)
        .min_by(|a, b| {
            a.dist(x as f64, z as f64)
                .total_cmp(&b.dist(x as f64, z as f64))
        })
}

/// The Stone ruin that always stands near the world's spawn: on the nearest line where one
/// passes within `SPAWN_REACH`, otherwise `SPAWN_DIST` out the way the seed says; never on a
/// waystone, and on dry ground where `dry(x, z)` finds any (the game's: above the sea).
pub fn spawn_ruin(seed: u64, spawn: [i32; 2], dry: &dyn Fn(i32, i32) -> bool) -> Ruin {
    let (sx, sz) = (spawn[0] as f64, spawn[1] as f64);
    let ring = KINDS.iter().position(|k| k.age == 0).unwrap_or(0);
    let at = |x: f64, z: f64| Ruin { kind: ring, x: x.round() as i32, z: z.round() as i32, cell: (i32::MIN, i32::MIN) };
    let fits = |x: f64, z: f64| {
        let far = (x - sx).hypot(z - sz);
        (MIN_FROM_SPAWN..=SPAWN_REACH - REACH as f64).contains(&far)
            && dry(x.round() as i32, z.round() as i32)
            && !ley::waystones_near(seed, x as i32, z as i32, APART as i32 + 1)
                .iter()
                .any(|n| ((n.x as f64 - x).powi(2) + (n.z as f64 - z).powi(2)).sqrt() < APART)
    };
    // A line near enough: the ancients' own place.
    if let Some([x, z]) = on_line(seed, sx, sz, SPAWN_REACH - REACH as f64)
        && fits(x, z)
    {
        return at(x, z);
    }
    // Otherwise round the spawn, the seed's way first, at a walk, then nearer and further.
    let a0 = (hash(seed, 0, 0, SALT_SPAWN) % 3600) as f64 / 3600.0 * std::f64::consts::TAU;
    for d in [SPAWN_DIST, SPAWN_DIST * 0.6, SPAWN_DIST * 1.5] {
        for turn in 0..16 {
            let a = a0 + turn as f64 / 16.0 * std::f64::consts::TAU;
            let (x, z) = (sx + d * a.cos(), sz + d * a.sin());
            if fits(x, z) {
                return at(x, z);
            }
        }
    }
    at(sx + SPAWN_DIST * a0.cos(), sz + SPAWN_DIST * a0.sin())
}

/// The first ruin stands at least this far from spawn (blocks): a walk, not a step.
pub const MIN_FROM_SPAWN: f64 = 48.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_blueprint_has_its_ruin_and_bit() {
        for (i, b) in BLUEPRINTS.iter().enumerate() {
            assert_eq!(b.bit(), 1 << i);
            assert!(kind_of_age(b.age).is_some(), "{} has no ruin of age {}", b.id, b.age);
            assert!(!b.crafts.is_empty() || !b.smelts.is_empty(), "{} teaches nothing", b.id);
        }
    }

    #[test]
    fn spawn_ruin_is_near_spawn() {
        for seed in 1..=8u64 {
            for spawn in [[0, 0], [1200, -800], [-3000, 2500], [77, 4096]] {
                let r = spawn_ruin(seed, spawn, &|_, _| true);
                let d = r.dist(spawn[0] as f64, spawn[1] as f64);
                assert!(d <= SPAWN_REACH, "seed {seed} spawn {spawn:?}: {d}");
                assert!(d >= 40.0, "seed {seed} spawn {spawn:?}: on the spawn ({d})");
            }
        }
    }

    #[test]
    fn ruins_keep_apart() {
        for seed in 1..=3u64 {
            let all = ruins_near(seed, Some(spawn_ruin(seed, [0, 0], &|_, _| true)), 0, 0, 3000);
            assert!(all.iter().filter(|r| r.kind == 0).count() > 20, "seed {seed}: few rings");
            for (k, kind) in KINDS.iter().enumerate().skip(1) {
                assert!(all.iter().any(|r| r.kind == k), "seed {seed}: no {}", kind.id);
            }
            for (i, a) in all.iter().enumerate() {
                for b in &all[i + 1..] {
                    assert!(a.dist(b.x as f64, b.z as f64) >= APART - 1.0, "seed {seed}: {a:?} and {b:?}");
                }
            }
        }
    }

    #[test]
    fn wards_follow_the_pickaxes() {
        let h = |m: &str| crate::gear::harvest(crate::gear::material(m).unwrap().level());
        assert!(h("flint") < WARD_BRONZE && h("copper") >= WARD_BRONZE);
        assert!(h("copper") < WARD_IRON && h("bronze") >= WARD_IRON);
    }
}
