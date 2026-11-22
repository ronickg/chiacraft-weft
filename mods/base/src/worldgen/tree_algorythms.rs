//! The trees, shrubs and boulders features lay out at their starts (`Plan`): canopy trees, giants
//! and their feet, young trees, treefall gaps, lianas, fallen giants.

use super::*;

/// A young tree of the biome's kinds (`TREE_KINDS`) at a feature's start; none on a bare rock face.
pub(super) fn young_tree(s: &mut Plan) {
    if !site_open(s) {
        return;
    }
    let [x, y, z] = s.pos().to_array();
    let h = s.random();
    let log = YOUNG_KINDS
        .on_biome(s.biome())
        .or_else(|| TREE_KINDS.on_biome(s.biome()))
        .and_then(|l| l.pick(h >> 40))
        .unwrap_or(OAK_LOG.id());
    base_api::trees::young_tree_blocks(log, x, y, z, h >> 8, |x, y, z, state, force| {
        let state = made_leaves(state);
        if force {
            s.set([x, y, z], state);
        } else {
            s.put([x, y, z], state);
        }
    });
}

/// A shrub at a feature's start: a clump of the biome's shrub leaves (`SHRUB_KINDS`), one or two
/// high (`vitals::layers::shrub_height`) and one to three across; none on a bare rock face.
pub(super) fn shrub(s: &mut Plan) {
    if !site_open(s) {
        return;
    }
    let h = s.random();
    let Some(leaves) = SHRUB_KINDS
        .on_biome(s.biome())
        .and_then(|l| l.pick(h >> 40))
        .map(made_leaves)
    else {
        return;
    };
    let tall = vitals::layers::shrub_height(name_of(abi::entry_kind::BIOME, s.biome()));
    let [x, y, z] = s.pos().to_array();
    for dy in 0..tall {
        for dz in -1..=1i32 {
            for dx in -1..=1i32 {
                let side = dx.abs() + dz.abs();
                // The middle always; its sides most of the time low down, less above; corners
                // seldom, and only low.
                let bit = hash(h, dx as i64, dy as i64, dz as i64) & 15;
                let keep = match (dy, side) {
                    (_, 0) => true,
                    (0, 1) => bit < 11,
                    (0, _) => bit < 4,
                    (_, 1) => bit < 5,
                    _ => false,
                };
                if keep {
                    s.put([x + dx, y + dy, z + dz], leaves);
                }
            }
        }
    }
}

/// Whether a tree or shrub may stand at a feature's start: not on a bare rock face, nor at the foot
/// of a wall (`land::site_on_rock`, `site_under_wall`).
pub(super) fn site_open(s: &Plan) -> bool {
    let (seed, dim, pos) = (s.seed(), s.dimension(), s.pos().to_array());
    !crate::land::site_on_rock(seed, dim, pos) && !crate::land::site_under_wall(seed, dim, pos)
}

/// A boulder at a feature's start: a lumpy heap, cobbles and stone, some mossy where it's wet; none
/// on a bare rock face (`land::site_on_rock`), which is stone already.
pub(super) fn boulder(s: &mut Plan) {
    if crate::land::site_on_rock(s.seed(), s.dimension(), s.pos().to_array()) {
        return;
    }
    let [x, y, z] = s.pos().to_array();
    let h = s.random();
    let mossy = WET.on_biome(s.biome()) || WOODS.on_biome(s.biome());
    let r = 1 + (h % 3) as i32;
    for dy in -1..=r {
        for dz in -r..=r {
            for dx in -r..=r {
                let bump = (hash(h, (x + dx) as i64, (y + dy) as i64, (z + dz) as i64) % 3) as i32;
                if dx * dx + 2 * dy * dy + dz * dz > r * r + bump {
                    continue;
                }
                let pick = hash(h ^ 0xB0, (x + dx) as i64, (y + dy) as i64, (z + dz) as i64) % 4;
                let block = match pick {
                    0 if mossy => MOSSY_COBBLESTONE.id(),
                    0 | 1 => COBBLESTONE.id(),
                    _ => STONE.id(),
                };
                s.set([x + dx, y + dy, z + dz], block);
            }
        }
    }
}

/// A giant of the biome's `GIANT_KINDS` at a feature's start; none on a bare rock face.
pub(super) fn giant(s: &mut Plan) {
    if !site_open(s) {
        return;
    }
    let h = s.random();
    let Some(log) = GIANT_KINDS
        .on_biome(s.biome())
        .and_then(|l| l.pick(h >> 40))
    else {
        return;
    };
    let [x, y, z] = s.pos().to_array();
    if TREEFALL
        .on_biome(s.biome())
        .and_then(|share| in_gap(s.seed(), x, z, share, GAP_MARGIN_GIANT))
        .is_some()
    {
        return;
    }
    // One sequoia in twelve lies fallen, mossy, along the ground (the overworld's: its height is
    // known).
    if (h >> 4).is_multiple_of(12) && s.dimension() == crate::OVERWORLD.id() && Block::lookup("base:sequoia_log") == Some(log) {
        fallen_giant(s, h);
        return;
    }
    let mut crown = Vec::new();
    let mut limbs = Vec::new();
    let climbers = LIANAS.on_biome(s.biome()).is_some();
    let brazil = Block::lookup("base:brazil_nut_log") == Some(log);
    // A Brazil nut stands on a mound of its own roots, its buttresses round its foot.
    if brazil {
        giant_foot(s, [x, y, z], h);
    }
    base_api::trees::giant_tree_blocks(log, x, y, z, h >> 8, |x, y, z, state, force| {
        if climbers && LEAVES.on_block(state) {
            crown.push([x, y, z]);
        }
        if brazil && matches!(state_prop(state, "axis"), Some("x" | "z")) {
            limbs.push([x, y, z]);
        }
        let state = made_leaves(state);
        if force {
            s.set([x, y, z], state);
        } else {
            s.put([x, y, z], state);
        }
    });
    lianas(s, &crown, y, h);
    // Bromeliads on one limb block in five.
    for (k, &[x, y, z]) in limbs.iter().enumerate() {
        if hash(h, 0xB20, k as i64, 0) % 5 == 0 {
            s.put([x, y + 1, z], BROMELIAD.id());
        }
    }
}

/// A Brazil nut's foot (Ronald, 11-04: "C's giants"): a low mound of its roots, up to two blocks
/// high and six out, topped with leaf litter, and four to six plank buttresses radiating from the
/// trunk, four to six high at it and stepping down to one, three to six blocks out.
pub(super) fn giant_foot(s: &mut Plan, [x, y, z]: [i32; 3], h: u64) {
    const TRUNK: f32 = 1.5;
    for dz in -6..=6i32 {
        for dx in -6..=6i32 {
            let d = ((dx * dx + dz * dz) as f32).sqrt();
            let rise = (2.2 - (d - TRUNK).max(0.0) / 2.2) as i32;
            for dy in 0..rise {
                let top = dy + 1 == rise;
                s.put([x + dx, y + dy, z + dz], if top { LEAF_LITTER.id() } else { DIRT.id() });
            }
        }
    }
    let fins = 4 + hash(h, 0xB21, 0, 0) % 3;
    let turn = (hash(h, 0xB22, 0, 0) % 360) as f32;
    for k in 0..fins {
        let angle = (turn + k as f32 * 360.0 / fins as f32 + (hash(h, 0xB23, k as i64, 0) % 30) as f32).to_radians();
        let (dx, dz) = (angle.cos(), angle.sin());
        let len = 3 + (hash(h, 0xB24, k as i64, 0) % 4) as i32;
        let tall = 4 + (hash(h, 0xB25, k as i64, 0) % 3) as i32;
        for step in 0..len {
            let r = TRUNK + 0.5 + step as f32;
            let (fx, fz) = (x + (dx * r).round() as i32, z + (dz * r).round() as i32);
            let up = (tall - tall * step / len).max(1);
            for dy in 0..up {
                s.set([fx, y + dy, fz], BRAZIL_NUT_BUTTRESS.id());
            }
        }
    }
}

/// Whether (`x`, `z`) lies in a treefall gap: each 48-block cell holds one at chance `share`, a
/// round opening 8 to 20 blocks across somewhere in it (its own and its neighbours' cells looked
/// at, so a gap may cross a cell's edge). `Some(true)` inside it, `Some(false)` in the ring
/// `margin` wide round it where no tree may stand, `None` elsewhere.
pub(super) fn in_gap(seed: u64, x: i32, z: i32, share: f32, margin: i32) -> Option<bool> {
    const CELL: i32 = 48;
    let (cx, cz) = (x.div_euclid(CELL), z.div_euclid(CELL));
    for dz in -1..=1 {
        for dx in -1..=1 {
            let (gx, gz) = (cx + dx, cz + dz);
            let h = hash(seed ^ 0x6A9F, gx as i64, 0, gz as i64);
            if (h & 1023) as f32 / 1024.0 >= share {
                continue;
            }
            let px = gx * CELL + ((h >> 10) % CELL as u64) as i32;
            let pz = gz * CELL + ((h >> 20) % CELL as u64) as i32;
            let r = 4 + ((h >> 30) % 7) as i32;
            let (ex, ez) = (x - px, z - pz);
            let d2 = ex * ex + ez * ez;
            if d2 <= (r + margin) * (r + margin) {
                return Some(d2 <= r * r);
            }
        }
    }
    None
}

/// What stands where a canopy tree would in a treefall gap: one in three a trunk lying fallen
/// (five to nine long, the way it fell), one in three a young tree of its kind coming up, the rest
/// open ground for the light (the ferns and big leaves the floor's layers put).
pub(super) fn gap_growth(s: &mut Plan, log: u32, [x, y, z]: [i32; 3], h: u64) {
    match (h >> 50) % 3 {
        0 => {
            let (dx, dz) = [(1, 0), (-1, 0), (0, 1), (0, -1)][((h >> 40) % 4) as usize];
            let lying = state_with(log, "axis", if dx != 0 { "x" } else { "z" }).unwrap_or(log);
            for k in 0..5 + ((h >> 44) % 5) as i32 {
                s.put([x + dx * k, y, z + dz * k], lying);
            }
        }
        1 => base_api::trees::young_tree_blocks(log, x, y, z, h >> 8, |x, y, z, state, force| {
            let state = made_leaves(state);
            if force {
            s.set([x, y, z], state);
        } else {
            s.put([x, y, z], state);
        }
        }),
        _ => {}
    }
}

/// Lianas hanging from a tree's crown (`LIANAS`, the share of trees that carry them): two to four
/// columns from the underside of the crown, three to twelve long, stopping two above the ground;
/// one in three a loop (Ronald, 11-04: "ropes and loops"), running on two or three blocks at its
/// foot and back up a few, as a monkey-ladder liana sags between two holds.
pub(super) fn lianas(s: &mut Plan, crown: &[[i32; 3]], ground: i32, h: u64) {
    let Some(share) = LIANAS.on_biome(s.biome()) else { return };
    if crown.is_empty() || (hash(h, 0x11A, 0, 0) & 1023) as f32 / 1024.0 >= share {
        return;
    }
    // The crown's lowest leaf in each column it covers (a map: a crown has hundreds of leaves,
    // and every column of cubes runs its shape again).
    let mut by_column: std::collections::HashMap<(i32, i32), i32> = std::collections::HashMap::new();
    for &[x, y, z] in crown {
        by_column
            .entry((x, z))
            .and_modify(|low| *low = (*low).min(y))
            .or_insert(y);
    }
    let mut lowest: Vec<[i32; 3]> = by_column.into_iter().map(|((x, z), y)| [x, y, z]).collect();
    lowest.sort_unstable();
    let count = 2 + hash(h, 0x11B, 0, 0) % 3;
    for k in 0..count {
        let r = hash(h, 0x11C, k as i64, 0);
        let [x, top, z] = lowest[(r % lowest.len() as u64) as usize];
        let len = 3 + ((r >> 16) % 10) as i32;
        let bottom = (top - len).max(ground + 2);
        for y in (bottom..top).rev() {
            s.put([x, y, z], LIANA.id());
        }
        if (r >> 32) % 3 == 0 {
            let (dx, dz) = [(1, 0), (-1, 0), (0, 1), (0, -1)][((r >> 36) % 4) as usize];
            let run = 2 + ((r >> 40) % 2) as i32;
            for k in 1..=run {
                s.put([x + dx * k, bottom, z + dz * k], LIANA.id());
            }
            let rise = 2 + ((r >> 44) % 4) as i32;
            for y in bottom + 1..(bottom + 1 + rise).min(top) {
                s.put([x + dx * run, y, z + dz * run], LIANA.id());
            }
        }
    }
}

/// A fallen giant: a mossy trunk three wide lying 14 to 22 long through its site, following the
/// ground under it (half sunk in it, only filling air), its root end a little wider.
pub(super) fn fallen_giant(s: &mut Plan, h: u64) {
    let [x, _, z] = s.pos().to_array();
    let (ax, az, axis) = if h >> 12 & 1 == 0 { (1, 0, "x") } else { (0, 1, "z") };
    let log = state_with(MOSSY_SEQUOIA_LOG.id(), "axis", axis).unwrap_or(MOSSY_SEQUOIA_LOG.id());
    let half = 7 + ((h >> 16) % 5) as i32;
    let seed = s.seed();
    for k in -half..=half {
        let (cx, cz) = (x + ax * k, z + az * k);
        let ground = crate::OVERWORLD.column(seed, cx, cz).height;
        let r2 = if k <= -half + 2 { 5 } else { 2 };
        for dy in -2i32..=2 {
            for d in -2i32..=2 {
                if d * d + dy * dy > r2 {
                    continue;
                }
                // Across the trunk: the other axis.
                let (px, pz) = (cx + az * d, cz + ax * d);
                s.put([px, ground + 1 + dy, pz], log);
            }
        }
    }
}

/// A tree of the biome's kinds (`TREE_KINDS`) at a feature's start; none on a bare rock face
/// (`land::site_on_rock`).
pub(super) fn tree(s: &mut Plan) {
    let log = TREE_KINDS
        .on_biome(s.biome())
        .and_then(|l| l.pick(s.random() >> 40))
        .unwrap_or(OAK_LOG.id());
    let log = mc_forest_kind(s, log);
    tree_of(s, log);
}

/// In the Minecraft forest test world (`MC_FOREST`) the old growth stands as Minecraft's forest:
/// oaks and birches, no spruces (Planner, 11-06, for like-for-like looks against Bliss's forest).
/// A spruce becomes an oak, or two in five a birch, so the old growth's oak 6 : spruce 2 : birch 1
/// comes to Minecraft's 4 : 1.
pub(super) fn mc_forest_kind(s: &mut Plan, log: u32) -> u32 {
    if s.dimension() != crate::MC_FOREST.id()
        || name_of(abi::entry_kind::BIOME, s.biome()) != "base:old_growth_forest"
        || log != crate::blocks::SPRUCE_LOG.id()
    {
        return log;
    }
    if s.random() % 5 < 2 { crate::blocks::BIRCH_LOG.id() } else { OAK_LOG.id() }
}

/// A wild cherry at a grove's site (`CHERRY_SITES`).
pub(super) fn cherry(s: &mut Plan) {
    if let Some(log) = Block::lookup("base:cherry_log") {
        tree_of(s, log);
    }
}

/// A tree of `log` at a feature's start; none on a bare rock face.
pub(super) fn tree_of(s: &mut Plan, log: u32) {
    if !site_open(s) {
        return;
    }
    let [x, y, z] = s.pos().to_array();
    let h = s.random();
    // In a treefall gap no canopy tree stands: one lies fallen, or young growth comes up.
    if let Some(inside) = TREEFALL
        .on_biome(s.biome())
        .and_then(|share| in_gap(s.seed(), x, z, share, GAP_MARGIN))
    {
        if inside {
            gap_growth(s, log, [x, y, z], h);
        }
        return;
    }
    // Its layer of the stand (`vitals::canopy`): under the canopy, in it, or over it.
    let roll = (hash(h, 0x71E2, 0, 0) >> 40) as f32 / (1u64 << 24) as f32;
    let tier = vitals::canopy::tier(name_of(abi::entry_kind::BIOME, s.biome()), roll);
    let Some(t) = wild_tree_at(log, x, y, z, h >> 32).map(|t| t.with_tier(tier)) else { return };
    let mut crown = Vec::new();
    let climbers = LIANAS.on_biome(s.biome()).is_some();
    tree_blocks(&t, |x, y, z, state, force| {
        if climbers && LEAVES.on_block(state) {
            crown.push([x, y, z]);
        }
        let state = made_leaves(state);
        if force {
            s.set([x, y, z], state);
        } else {
            s.put([x, y, z], state);
        }
    });
    lianas(s, &crown, y, h);
}
