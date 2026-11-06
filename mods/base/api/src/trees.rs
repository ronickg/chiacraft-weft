//! Trees (`notes/ECOLOGY.md`, part 3), for any mod: a log names its leaves (`TREE_LEAVES`); a
//! species with a recipe (`tree_recipes`, by the Animal models session) grows from branches
//! (`branchy`), the others by a crown shape their log's tag gives (`CONE_TREE`, `FLAT_TREE`,
//! `PALM_TREE`, `SLIM_TREE`; none: round). In the wild a few lie fallen or stand as stumps. Base's worldgen grows
//! them where a biome's `tree_kinds` names their logs, and a sapling's growth step grows one in a
//! live world (`grow_tree`), whichever mod's log it is.

use crate::branchy;
use vitals::canopy::{self, Kind, Tier};
use weft_sdk::noise::hash;
use weft_sdk::*;

/// On a log, the leaves of the tree it grows into.
pub static TREE_LEAVES: Component<BlockRef> = Component::named("tree_leaves");
/// On a log, its thin branch block (the study, Ronald 11-05: "branches ... a bit like a fence"):
/// a tree grown from branches puts its wood under `BRANCH_UNDER` wide as this, a post 4 or 8 px
/// thick (`size`) with an arm to each wood neighbour (`n` `s` `e` `w` `u` `d`).
pub static TREE_BRANCH: Component<BlockRef> = Component::named("tree_branch");
/// Wood under this wide is a branch block: under `BRANCH_THIN` a 4 px one, else 8 px.
pub const BRANCH_UNDER: f32 = 0.75;
pub const BRANCH_THIN: f32 = 0.4;
/// The woods whose trees grow their thin wood as branch blocks (`base:<wood>_branch`, named on
/// the log as `TREE_BRANCH`): the broadleaf trees (Ronald, 11-05: "All broadleaf trees"); needles
/// and fronds hide a conifer's or a palm's wood, which keep their logs. Felling counts a branch
/// block a quarter of a log (`tree_recipes::felling_wood`).
pub const BRANCH_WOODS: &[&str] = &["oak", "birch", "acacia", "mesquite", "dogwood", "cherry", "ipe", "andiroba", "matamata", "brazil_nut"];
/// The world's tree style (`base:tree_style`, a world setting, `/rule base:tree_style 0|1`): 0
/// simple, Minecraft-style trees, a straight log trunk under a crown of leaves shaped by its log's
/// tag (`Crown`), no branch blocks; 1 branched, grown from the species' recipes (`tree_recipes`,
/// `colonize`) with branch blocks. Ronald, 11-05: "lets also maybe switch back to having trees
/// without our branch system just for now too to help with testing with bliss/distant horizons
/// and for android speed testing too". Simple for now; the recipes stay behind the setting.
pub static TREE_STYLE: Config<u32> = Config::named("tree_style");

/// Whether trees grow simple, Minecraft-style (`TREE_STYLE` 0).
pub fn simple_trees() -> bool {
    TREE_STYLE.get() == 0
}

/// Logs with this tag grow into cones (spruces), the others round (oaks, birches).
pub static CONE_TREE: Tag = Tag::named("cone_tree");
/// A tall bare trunk under one wide, flat crown (the acacia).
pub static FLAT_TREE: Tag = Tag::named("flat_tree");
/// A taller bare trunk under a crown of fronds that droop at their tips (palms, tree ferns).
pub static PALM_TREE: Tag = Tag::named("palm_tree");
/// A tall slim trunk under a narrow, long crown (birches).
pub static SLIM_TREE: Tag = Tag::named("slim_tree");

/// Leaves that turn differently in autumn (`vitals::wild::Turn`): evergreens never, birch-like
/// trees yellow; the rest keep `tints::FOLIAGE`. Each biome gives all three its foliage colour, and
/// the lighting session's seasonal tints change them through the year.
pub static EVERGREEN: TintKind = TintKind::named("base:evergreen");
pub static YELLOW_LEAVES: TintKind = TintKind::named("base:yellow_leaves");

/// The tint kind for leaves that turn so (`vitals::wild::turn`).
pub fn leaf_tint(turn: vitals::wild::Turn) -> &'static TintKind {
    match turn {
        vitals::wild::Turn::Evergreen => &EVERGREEN,
        vitals::wild::Turn::Yellow => &YELLOW_LEAVES,
        vitals::wild::Turn::Orange => &tints::FOLIAGE,
    }
}

static DIRT: Block = Block::named("base:dirt");
static TALL_GRASS: Block = Block::named("base:tall_grass");
/// A shelf fungus on dead wood, on a trunk's north side (base's `bracket_fungus`).
static BRACKET_FUNGUS: Block = Block::named("base:bracket_fungus");

/// A tree's crown, by its log's tag.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Crown {
    Round,
    Cone,
    Flat,
    Palm,
    Slim,
}

/// How this one grew (`h` picks it): most stand straight; some grew big, crooked or as a twin pair;
/// in the wild a few lie fallen or stand as a stump (no leaves).
#[derive(Clone, Copy, PartialEq)]
enum Form {
    Standing,
    Big,
    Crooked,
    Twin,
    Fallen,
    Stump,
    /// A standing dead trunk, bare (the taiga's fire-killed spruces, a wood's old snags).
    Snag,
}

pub struct TreeAt {
    x: i32,
    y: i32,
    z: i32,
    log: u32,
    leaves: u32,
    crown: Crown,
    form: Form,
    height: i32,
    /// The layer of its stand it grows into (`vitals::canopy`): under the canopy, in it, or over it.
    tier: Tier,
    /// The way a crooked trunk bends, a twin stands, a fallen one lies.
    lean: (i32, i32),
    /// Its seed, for a tree grown from branches (`branchy`).
    seed: u64,
}

impl TreeAt {
    /// The same tree grown into `tier` of its stand (`vitals::canopy::tier`, rolled by its site):
    /// its height and crown by the tier. A sapling's and a lone tree's are canopy trees.
    pub fn with_tier(mut self, tier: Tier) -> Self {
        self.tier = tier;
        self.height = height_of(self.crown, self.form, tier, self.seed);
        self
    }

    /// Whether it stands with a crown (not fallen, a stump or a bare snag).
    pub fn has_crown(&self) -> bool {
        !matches!(self.form, Form::Fallen | Form::Stump | Form::Snag)
    }

    /// Its height to the crown's top, above its foot.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Where its foot stands.
    pub fn pos(&self) -> [i32; 3] {
        [self.x, self.y, self.z]
    }
}

const LEANS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// A tree growing from `log` (its `tree` component), `h` deciding its height and form: a sapling's,
/// always standing.
pub fn tree_at(log: u32, x: i32, y: i32, z: i32, h: u64) -> Option<TreeAt> {
    shaped(log, x, y, z, h, false)
}

/// A tree the world grows, which may also lie fallen or stand as a stump (forest trees only).
pub fn wild_tree_at(log: u32, x: i32, y: i32, z: i32, h: u64) -> Option<TreeAt> {
    shaped(log, x, y, z, h, true)
}

fn shaped(log: u32, x: i32, y: i32, z: i32, h: u64, wild: bool) -> Option<TreeAt> {
    let leaves = TREE_LEAVES.on_block(log)?.0;
    let crown = if PALM_TREE.on_block(log) {
        Crown::Palm
    } else if FLAT_TREE.on_block(log) {
        Crown::Flat
    } else if CONE_TREE.on_block(log) {
        Crown::Cone
    } else if SLIM_TREE.on_block(log) {
        Crown::Slim
    } else {
        Crown::Round
    };
    Some(simple_at(crown, log, leaves, x, y, z, h, wild))
}

/// A tree of crown `crown` growing from `log` under `leaves` (no lookups: the canopy sheet's
/// `forest_canopy` example grows these off the game), a canopy tree until `with_tier`.
#[allow(clippy::too_many_arguments)]
pub fn simple_at(crown: Crown, log: u32, leaves: u32, x: i32, y: i32, z: i32, h: u64, wild: bool) -> TreeAt {
    let roll = (h >> 8) % 100;
    let forest = matches!(crown, Crown::Round | Crown::Cone | Crown::Slim);
    let form = match roll {
        0..4 if wild && forest => Form::Fallen,
        4..8 if wild && forest => Form::Stump,
        42..45 if wild && forest => Form::Snag,
        8..22 if crown != Crown::Palm => Form::Big,
        22..34 if crown != Crown::Cone => Form::Crooked,
        34..42 if crown == Crown::Round => Form::Twin,
        _ => Form::Standing,
    };
    let height = height_of(crown, form, Tier::Canopy, h);
    let lean = LEANS[(h >> 20) as usize % 4];
    TreeAt { x, y, z, log, leaves, crown, form, height, tier: Tier::Canopy, lean, seed: h }
}

/// The stand-structure kind of a crown (`vitals::canopy`): palms and flat crowns stand alone.
fn kind_of(crown: Crown) -> Option<Kind> {
    match crown {
        Crown::Round => Some(Kind::Round),
        Crown::Cone => Some(Kind::Cone),
        Crown::Slim => Some(Kind::Slim),
        Crown::Flat | Crown::Palm => None,
    }
}

/// How high a tree grows to its crown's top: by its crown and tier (`vitals::canopy::crown`), a
/// big one a block more.
fn height_of(crown: Crown, form: Form, tier: Tier, h: u64) -> i32 {
    let big = (form == Form::Big) as i32;
    match kind_of(crown) {
        Some(k) => {
            let c = canopy::crown(k, tier);
            c.low + (h % c.spread.max(1) as u64) as i32 + big
        }
        None if crown == Crown::Palm => 7 + (h % 3) as i32,
        None => 5 + (h % 2) as i32 + big,
    }
}

/// Calls `put(x, y, z, state, force)` for every block of the tree.
pub fn tree_blocks(t: &TreeAt, mut put: impl FnMut(i32, i32, i32, u32, bool)) {
    let log = t.log;
    match t.form {
        // A trunk lying on the ground along its lean, 3 to 5 long, its log turned on its side.
        Form::Fallen => {
            let axis = if t.lean.0 != 0 { "x" } else { "z" };
            let lying = state_with(log, "axis", axis).unwrap_or(log);
            for k in 0..3 + t.height % 3 {
                put(t.x + t.lean.0 * k, t.y, t.z + t.lean.1 * k, lying, false);
            }
            // Shelf fungus on its north side, on half of them.
            if t.seed >> 30 & 1 == 0 {
                put(t.x + t.lean.0, t.y, t.z + t.lean.1 - 1, BRACKET_FUNGUS.id(), false);
            }
            return;
        }
        // A bare trunk three to six high, a stub of a branch near its top.
        Form::Snag => {
            let top = t.y + 3 + t.height % 4;
            for y in t.y..top {
                put(t.x, y, t.z, log, true);
            }
            let axis = if t.lean.0 != 0 { "x" } else { "z" };
            put(t.x + t.lean.0, top - 2, t.z + t.lean.1, state_with(log, "axis", axis).unwrap_or(log), false);
            // Shelf fungus rotting it, on its north side, on half of them.
            if t.seed >> 30 & 1 == 0 {
                put(t.x, t.y + 1 + (t.seed >> 31 & 1) as i32, t.z - 1, BRACKET_FUNGUS.id(), false);
            }
            put(t.x, t.y - 1, t.z, DIRT.id(), true);
            return;
        }
        Form::Stump => {
            for y in t.y..t.y + 1 + t.height % 2 {
                put(t.x, y, t.z, log, true);
            }
            if t.seed >> 30 & 1 == 0 {
                put(t.x, t.y, t.z - 1, BRACKET_FUNGUS.id(), false);
            }
            put(t.x, t.y - 1, t.z, DIRT.id(), true);
            return;
        }
        _ => {}
    }
    // A species with a recipe grows from branches (`branchy`), its log turned along each limb. A
    // giant's species stands here as its young one, the world's as a sapling's: the grown giants
    // are the giant scatter's (`giant_tree_blocks`), too tall for a tree's site.
    let name = block_name(log).split('[').next().unwrap_or("");
    // A broad-crowned species grows by space colonization (`colonize`, Ronald's hybrid, 11-05).
    // (A giant's species in a tree's site grows its young recipe, below: the grown giant is the
    // giant scatter's.)
    let giant_young = crate::tree_recipes::YOUNG.iter().any(|y| y.log == name);
    // Simple trees (`TREE_STYLE`): every species by its crown shape, below.
    let simple = simple_trees();
    // Its form (rolled in `shaped`): big and old, crooked or a twin, or as its recipe grows.
    let variant = match t.form {
        Form::Big => crate::tree_recipes::Variant::Big,
        Form::Crooked => crate::tree_recipes::Variant::Crooked,
        Form::Twin => crate::tree_recipes::Variant::Twin,
        _ => crate::tree_recipes::Variant::Standing,
    };
    if let Some(c) = crate::tree_recipes::colonized(name)
        .filter(|_| !giant_young && !simple)
        .map(|c| crate::tree_recipes::tiered_colonized(&crate::tree_recipes::varied_colonized(c, variant), t.tier))
    {
        let c = &c;
        if let Some(branch) = TREE_BRANCH.on_block(log).map(|b| b.0) {
            put_with_branches(&crate::colonize::blocks(c, t.seed), None, log, t.leaves, branch, t.x, t.y, t.z, &mut put);
            put(t.x, t.y - 1, t.z, DIRT.id(), true);
            return;
        }
        for (dx, dy, dz, part, _) in crate::colonize::blocks(c, t.seed) {
            let (x, y, z) = (t.x + dx, t.y + dy, t.z + dz);
            match part {
                branchy::Part::Log { axis } => {
                    let along = ["x", "y", "z"][axis as usize];
                    put(x, y, z, state_with(log, "axis", along).unwrap_or(log), true)
                }
                branchy::Part::Leaves => put(x, y, z, t.leaves, false),
            }
        }
        put(t.x, t.y - 1, t.z, DIRT.id(), true);
        return;
    }
    let recipe = crate::tree_recipes::sapling_of(name)
        .filter(|_| !simple)
        .map(|r| crate::tree_recipes::tiered(&crate::tree_recipes::varied(r, variant), t.tier));
    let recipe = recipe.as_ref();
    if let (Some(recipe), Some(branch)) = (recipe, TREE_BRANCH.on_block(log).map(|b| b.0)) {
        grow_with_branches(recipe, log, t.leaves, branch, t.x, t.y, t.z, t.seed, &mut put);
        put(t.x, t.y - 1, t.z, DIRT.id(), true);
        return;
    }
    if let Some(recipe) = recipe {
        branchy::grow(recipe, t.x, t.y, t.z, t.seed, |x, y, z, part| match part {
            branchy::Part::Log { axis } => {
                let along = ["x", "y", "z"][axis as usize];
                put(x, y, z, state_with(log, "axis", along).unwrap_or(log), true)
            }
            branchy::Part::Leaves => put(x, y, z, t.leaves, false),
        });
        put(t.x, t.y - 1, t.z, DIRT.id(), true);
        return;
    }
    simple_standing(t, &mut put);
    put(t.x, t.y - 1, t.z, DIRT.id(), true);
}

/// A standing tree's trunk and crown by its crown shape (simple trees, and every species without
/// a recipe): no lookups, so the canopy sheet (`forest_canopy`) grows the same trees off the game.
pub fn simple_standing(t: &TreeAt, mut put: impl FnMut(i32, i32, i32, u32, bool)) {
    let log = t.log;
    let top = t.y + t.height;
    // A crooked trunk steps aside once, two or three blocks under its top, and its crown with it.
    let bend = if t.form == Form::Crooked { top - 2 - t.height % 2 } else { i32::MAX };
    let (cx, cz) = if t.form == Form::Crooked { (t.x + t.lean.0, t.z + t.lean.1) } else { (t.x, t.z) };
    crown(t, cx, cz, top, &mut put);
    for y in t.y..top {
        let (x, z) = if y >= bend { (cx, cz) } else { (t.x, t.z) };
        put(x, y, z, log, true);
    }
    // A twin: a second, shorter trunk beside the first, with its own crown.
    if t.form == Form::Twin {
        let (sx, sz) = (t.x + t.lean.0 + t.lean.1, t.z + t.lean.1 - t.lean.0);
        let second = top - 1 - t.height % 2;
        crown(t, sx, sz, second, &mut put);
        for y in t.y..second {
            put(sx, y, sz, log, true);
        }
    }
}

/// The leaves of a crown whose trunk tops out at `top`, over (`cx`, `cz`).
fn crown(t: &TreeAt, cx: i32, cz: i32, top: i32, put: &mut impl FnMut(i32, i32, i32, u32, bool)) {
    let leaves = t.leaves;
    match t.crown {
        Crown::Palm => {
            // A tuft on top, and fronds out four blocks each way that droop as they go, their
            // corners shorter.
            put(cx, top, cz, leaves, false);
            put(cx, top + 1, cz, leaves, false);
            for (dx, dz) in LEANS {
                for k in 1..=4 {
                    put(cx + dx * k, top + 1 - k.min(3), cz + dz * k, leaves, false);
                }
            }
            for (dx, dz) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                for k in 1..=2 {
                    put(cx + dx * k, top + 1 - k, cz + dz * k, leaves, false);
                }
            }
        }
        Crown::Flat => {
            // One wide flat layer on the trunk's top, a small one over its middle; a big one wider.
            let wide = if t.form == Form::Big { 13 } else { 10 };
            for (y, r2) in [(top, wide), (top + 1, 2)] {
                for dz in -4i32..=4 {
                    for dx in -4i32..=4 {
                        let d = dx * dx + dz * dz;
                        if d <= r2 && (d < r2 || hash(1, (cx + dx) as i64, y as i64, (cz + dz) as i64) & 1 == 0) {
                            put(cx + dx, y, cz + dz, leaves, false);
                        }
                    }
                }
            }
        }
        Crown::Cone => {
            // Rings stepping in toward a spire from where the needles start (a quarter of the way
            // up, three tenths on an emergent: barer trunks stood as poles, the PC look 11-06),
            // every other ring pulled in: a ruffled spire whose ledges catch the sun, like
            // Minecraft's spruces, sized by its tier.
            let c = canopy::crown(Kind::Cone, t.tier);
            let bare = ((t.height as f32 * if t.tier == Tier::Emergent { 0.3 } else { 0.25 }).round() as i32).max(1);
            let base = c.radius + if t.form == Form::Big { 0.4 } else { 0.0 } + jitter(t.seed, 1) * 0.3;
            let len = (t.height + 1 - bare).max(1);
            for dy in bare..=t.height {
                let k = t.height - dy;
                let mut r = base * (k as f32 + 0.6) / len as f32;
                if k % 2 == 1 && k > 1 {
                    r -= 0.7;
                }
                ring(cx, t.y + dy, cz, r, 0.3, t.seed, leaves, put);
            }
            put(cx, t.y + t.height + 1, cz, leaves, false);
        }
        Crown::Round | Crown::Slim => {
            // A dome (`dome`), narrower at its top, its rim ragged; a birch's taller and slimmer.
            // Some canopy crowns and every emergent carry side lobes at their shoulders, as an old
            // oak's big limbs each hold a clump: lumpy outlines that shade each other, so a stand
            // reads as crowns, not one mat (`vitals::canopy`).
            let kind = if t.crown == Crown::Slim { Kind::Slim } else { Kind::Round };
            let c = canopy::crown(kind, t.tier);
            let big = t.form == Form::Big;
            let r = c.radius + if big { 0.4 } else { 0.0 } + jitter(t.seed, 2) * 0.3;
            let depth = c.depth + big as i32;
            let mid = if kind == Kind::Slim { 0.5 } else { 0.6 };
            dome(cx, top, cz, r, depth, mid, t.seed, leaves, put);
            let lobes = match t.tier {
                Tier::Emergent => c.lobes,
                Tier::Canopy if c.lobes > 0 && unit(t.seed, 3) < canopy::CANOPY_LOBE => 1,
                _ => 0,
            };
            let turn = unit(t.seed, 4) * std::f32::consts::TAU;
            for i in 0..lobes {
                let a = turn + i as f32 * std::f32::consts::TAU / lobes as f32 + jitter(t.seed, 10 + i as u64) * 0.6;
                let out = r * 0.8 + unit(t.seed, 20 + i as u64) * 0.6;
                let (lx, lz) = (cx + (a.cos() * out).round() as i32, cz + (a.sin() * out).round() as i32);
                let ly = top - 2 + (hash(t.seed, i as i64, 7, 0) % 4) as i32;
                let lr = r * 0.62 + unit(t.seed, 30 + i as u64) * 0.4;
                dome(lx, ly, lz, lr, 4, mid, t.seed ^ (i as u64 + 1) * 0x9E37, leaves, put);
            }
        }
    }
}

/// -1..1 from `seed`, its `k`th draw.
fn jitter(seed: u64, k: u64) -> f32 {
    unit(seed, k) * 2.0 - 1.0
}

/// 0..1 from `seed`, its `k`th draw.
fn unit(seed: u64, k: u64) -> f32 {
    (hash(seed, k as i64, 0x7E3, 0) >> 40) as f32 / (1u64 << 24) as f32
}

/// A dome of leaves `depth` layers deep from `top` down over (`cx`, `cz`), `r` wide at its widest
/// (at `mid`, a share of its depth from the top), its sides steep and its top and foot drawn in a
/// little (a rounded box, as Minecraft's oaks: the steep sides are what shade), its rim ragged
/// (`ring`).
#[allow(clippy::too_many_arguments)]
fn dome(cx: i32, top: i32, cz: i32, r: f32, depth: i32, mid: f32, seed: u64, leaves: u32, put: &mut impl FnMut(i32, i32, i32, u32, bool)) {
    for k in 0..depth.max(1) {
        let t = (k as f32 + 0.5) / depth.max(1) as f32;
        let f = 1.0 - ((t - mid).abs() / 0.65).powi(3);
        ring(cx, top - k, cz, r * f.max(0.0).cbrt(), 0.9, seed, leaves, put);
    }
}

/// A layer of leaves round (`cx`, `cz`) out to `r`, each column's edge moved by up to `ragged`
/// either way so the rim is ragged; its middle always.
#[allow(clippy::too_many_arguments)]
fn ring(cx: i32, y: i32, cz: i32, r: f32, ragged: f32, seed: u64, leaves: u32, put: &mut impl FnMut(i32, i32, i32, u32, bool)) {
    let reach = (r + ragged).ceil().max(0.0) as i32;
    for dz in -reach..=reach {
        for dx in -reach..=reach {
            let (x, z) = (cx + dx, cz + dz);
            let edge = r + ragged * ((hash(seed ^ 0xC0, x as i64, y as i64, z as i64) & 0xFF) as f32 / 255.0 - 0.55);
            if dx == 0 && dz == 0 || ((dx * dx + dz * dz) as f32) <= edge.max(0.0).powi(2) {
                put(x, y, z, leaves, false);
            }
        }
    }
}

/// A grown giant of `log` (its recipe's full size, `tree_recipes::of`; a sequoia's 60 blocks), for
/// a scatter of its own: nothing for a log without a recipe. Calls `put(x, y, z, state, force)`.
pub fn giant_tree_blocks(log: u32, x: i32, y: i32, z: i32, h: u64, mut put: impl FnMut(i32, i32, i32, u32, bool)) {
    let Some(leaves) = TREE_LEAVES.on_block(log).map(|l| l.0) else { return };
    // Simple trees (`TREE_STYLE`): a big emergent of its crown shape, half again as tall.
    if simple_trees() {
        if let Some(mut t) = shaped(log, x, y, z, h, false) {
            t.form = Form::Big;
            let mut t = t.with_tier(Tier::Emergent);
            t.height = t.height * 3 / 2;
            tree_blocks(&t, put);
        }
        return;
    }
    let name = block_name(log).split('[').next().unwrap_or("");
    // A colonized giant (the Brazil nut): its trunk from its recipe, its crown by colonization.
    if let Some(c) = crate::tree_recipes::colonized(name) {
        let bark = state_with(log, "axis", "bark");
        if let Some(branch) = TREE_BRANCH.on_block(log).map(|b| b.0) {
            put_with_branches(&crate::colonize::blocks(c, h), Some(bark.unwrap_or(log)), log, leaves, branch, x, y, z, &mut put);
            put(x, y - 1, z, DIRT.id(), true);
            return;
        }
        for (dx, dy, dz, part, _) in crate::colonize::blocks(c, h) {
            let (x, y, z) = (x + dx, y + dy, z + dz);
            match part {
                branchy::Part::Log { axis: 1 } => put(x, y, z, bark.unwrap_or(log), true),
                branchy::Part::Log { axis } => {
                    let along = ["x", "y", "z"][axis as usize];
                    put(x, y, z, state_with(log, "axis", along).unwrap_or(log), true)
                }
                branchy::Part::Leaves => put(x, y, z, leaves, false),
            }
        }
        put(x, y - 1, z, DIRT.id(), true);
        return;
    }
    let Some(recipe) = crate::tree_recipes::of(name) else { return };
    // The upright trunk in bark all round: its flared foot spreads out at the ground, where an
    // upright log's top would show its rings (the PC look, 11-04).
    let bark = state_with(log, "axis", "bark");
    branchy::grow(recipe, x, y, z, h, |x, y, z, part| match part {
        branchy::Part::Log { axis: 1 } => put(x, y, z, bark.unwrap_or(log), true),
        branchy::Part::Log { axis } => {
            let along = ["x", "y", "z"][axis as usize];
            put(x, y, z, state_with(log, "axis", along).unwrap_or(log), true)
        }
        branchy::Part::Leaves => put(x, y, z, leaves, false),
    });
    put(x, y - 1, z, DIRT.id(), true);
}

/// A young tree of `log` (the understory: in clearings and at a wood's edge), `h` its seed: a
/// species with a recipe grows a small one of its own shape (`tree_recipes::understory`, within
/// `UNDERSTORY_REACH`); the rest a trunk two to four high under a small crown of its leaves, a
/// spire for the cone trees, a ragged ball for the rest. None for palms and flat-topped trees, which grow up in the open.
/// Calls `put(x, y, z, state, force)` for every block.
pub fn young_tree_blocks(log: u32, x: i32, y: i32, z: i32, h: u64, mut put: impl FnMut(i32, i32, i32, u32, bool)) {
    let Some(leaves) = TREE_LEAVES.on_block(log).map(|l| l.0) else { return };
    if PALM_TREE.on_block(log) || FLAT_TREE.on_block(log) {
        return;
    }
    // A species with a recipe grows its young from it, small (`tree_recipes::understory`), so a
    // young oak reads as an oak and a young birch as a birch.
    let name = block_name(log).split('[').next().unwrap_or("");
    if let Some(recipe) = crate::tree_recipes::of(name).filter(|_| !simple_trees()) {
        let young = crate::tree_recipes::understory(recipe);
        branchy::grow(&young, x, y, z, h, |x, y, z, part| match part {
            branchy::Part::Log { axis } => {
                let along = ["x", "y", "z"][axis as usize];
                put(x, y, z, state_with(log, "axis", along).unwrap_or(log), true)
            }
            branchy::Part::Leaves => put(x, y, z, leaves, false),
        });
        return;
    }
    let height = 2 + (h % 3) as i32;
    let top = y + height;
    if CONE_TREE.on_block(log) {
        // Rings of one round the trunk from its first block up, and a tip on top.
        for k in 1..height {
            for (dx, dz) in LEANS {
                if k % 2 == 1 || hash(h, dx as i64, k as i64, dz as i64) & 1 == 0 {
                    put(x + dx, y + k, z + dz, leaves, false);
                }
            }
        }
        put(x, top, z, leaves, false);
        put(x, top + 1, z, leaves, false);
    } else {
        for cy in top - 1..=top {
            for dz in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx.abs() + dz.abs() == 2 && hash(h, dx as i64, cy as i64, dz as i64) & 3 != 0 {
                        continue;
                    }
                    put(x + dx, cy, z + dz, leaves, false);
                }
            }
        }
        put(x, top + 1, z, leaves, false);
    }
    for k in 0..height {
        put(x, y + k, z, log, true);
    }
}

/// Places a tree of `log` into a live world (saplings). Checks for room first.
pub fn grow_tree(w: World, p: [i32; 3], log: u32) {
    let Some(t) = tree_at(log, p[0], p[1], p[2], w.random()) else { return };
    for y in t.y + 1..t.y + t.height {
        if w.block([t.x, y, t.z]) != 0 {
            return;
        }
    }
    tree_blocks(&t, |x, y, z, s, overwrite| {
        let cur = w.block([x, y, z]);
        if overwrite || cur == 0 || TALL_GRASS.is(cur) {
            w.set_block([x, y, z], s);
        }
    });
}

/// A tree grown from `recipe` whose thin wood is `branch` blocks (`TREE_BRANCH`), each joined to
/// its wood neighbours; the rest logs along their limbs and leaves.
#[allow(clippy::too_many_arguments)]
fn grow_with_branches(
    recipe: &branchy::Recipe,
    log: u32,
    leaves: u32,
    branch: u32,
    x: i32,
    y: i32,
    z: i32,
    seed: u64,
    put: &mut impl FnMut(i32, i32, i32, u32, bool),
) {
    let widths: std::collections::HashMap<_, _> = branchy::wood_widths(recipe, seed)
        .into_iter()
        .map(|(x, y, z, w)| ((x, y, z), w))
        .collect();
    let b: Vec<_> = branchy::blocks(recipe, seed)
        .into_iter()
        .map(|(x, y, z, p)| (x, y, z, p, widths.get(&(x, y, z)).copied().unwrap_or(1.0)))
        .collect();
    put_with_branches(&b, None, log, leaves, branch, x, y, z, put);
}

/// A tree's blocks with each wood block's width (`colonize::blocks`, `branchy::wood_widths`), its
/// thin wood as `branch` blocks joined to their wood neighbours, the rest logs and leaves.
#[allow(clippy::too_many_arguments)]
fn put_with_branches(
    b: &[(i32, i32, i32, branchy::Part, f32)],
    upright: Option<u32>,
    log: u32,
    leaves: u32,
    branch: u32,
    x: i32,
    y: i32,
    z: i32,
    put: &mut impl FnMut(i32, i32, i32, u32, bool),
) {
    // No bare wood on the crown's top (`tree_recipes::capped`).
    let b = &crate::tree_recipes::capped(b);
    let wood: std::collections::HashSet<_> = b
        .iter()
        .filter(|e| matches!(e.3, branchy::Part::Log { .. }))
        .map(|e| (e.0, e.1, e.2))
        .collect();
    for &(dx, dy, dz, part, w) in b {
        match part {
            branchy::Part::Leaves => put(x + dx, y + dy, z + dz, leaves, false),
            branchy::Part::Log { axis } => {
                if w >= BRANCH_UNDER {
                    let along = ["x", "y", "z"][axis as usize];
                    let state = match upright {
                        Some(u) if axis == 1 => u,
                        _ => state_with(log, "axis", along).unwrap_or(log),
                    };
                    put(x + dx, y + dy, z + dz, state, true);
                    continue;
                }
                let mut s = state_with(branch, "size", if w < BRANCH_THIN { "4" } else { "8" }).unwrap_or(branch);
                for (name, (ox, oy, oz)) in
                    [("n", (0, 0, -1)), ("s", (0, 0, 1)), ("e", (1, 0, 0)), ("w", (-1, 0, 0)), ("u", (0, 1, 0)), ("d", (0, -1, 0))]
                {
                    if wood.contains(&(dx + ox, dy + oy, dz + oz)) {
                        s = state_with(s, name, "1").unwrap_or(s);
                    }
                }
                put(x + dx, y + dy, z + dz, s, true);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every simple crown in every layer of its stand stays inside the tree scatters' reach (8
    /// sideways, `worldgen.rs`) and under their 32 up, or a cube's edge would cut it off.
    #[test]
    fn simple_crowns_within_reach() {
        for crown in [Crown::Round, Crown::Cone, Crown::Slim, Crown::Flat, Crown::Palm] {
            for tier in [Tier::Under, Tier::Canopy, Tier::Emergent] {
                for seed in 0..400u64 {
                    let h = hash(seed, 3, 5, 7) >> 32;
                    let t = simple_at(crown, 1, 2, 0, 0, 0, h, false).with_tier(tier);
                    let mut leaves = 0;
                    simple_standing(&t, |x, y, z, s, _| {
                        assert!(x.abs() <= 8 && z.abs() <= 8 && (0..32).contains(&y), "{crown:?} {tier:?} {seed}: {x} {y} {z}");
                        leaves += (s == 2) as u32;
                    });
                    assert!(leaves > 0, "{crown:?} {tier:?}: a crown");
                }
            }
        }
    }

    /// The layers stand apart: an emergent's top over every canopy tree's of its kind, an
    /// under-storey tree's under them.
    #[test]
    fn layers_stand_apart() {
        for crown in [Crown::Round, Crown::Cone, Crown::Slim] {
            let top = |tier: Tier| {
                (0..200u64)
                    .map(|s| {
                        simple_at(crown, 1, 2, 0, 0, 0, hash(s, 1, 1, 1) >> 32, false)
                            .with_tier(tier)
                            .height()
                    })
                    .fold((i32::MAX, i32::MIN), |(lo, hi), v| (lo.min(v), hi.max(v)))
            };
            let (u, c, e) = (top(Tier::Under), top(Tier::Canopy), top(Tier::Emergent));
            assert!(u.1 < c.1 && c.1 <= e.0, "{crown:?}: under {u:?} canopy {c:?} emergent {e:?}");
        }
    }
}
