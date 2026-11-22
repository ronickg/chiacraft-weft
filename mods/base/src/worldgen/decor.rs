//! The decoration after the terrain and trees: plants, flowers, the floor's layers, clay, ponds
//! and the sea bed's plants.

use super::*;
// Ours, not `crate::blocks::MUSHROOMS` (upstream's mushroom list).
use super::statics::MUSHROOMS;

/// Plants and Clay after the terrain and trees filled the cube.
pub(super) fn decorate(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let seed = g.seed();
    let BlockPos { x: ox, y: oy, z: oz } = g.origin();
    let max_h = cols.iter().map(|c| c.height).max().unwrap_or(0);
    let min_h = cols.iter().map(|c| c.height).min().unwrap_or(0);

    // Surface decoration only near the surface, which under the sea is its water's (kelp).
    let top = if min_h < SEA { max_h.max(SEA) } else { max_h };
    if oy > top + 16 || oy + 15 < min_h - 1 {
        return;
    }
    // The world's first place, for the loose things thicker round it (`first.rs`).
    let spawn = crate::fair_spawn(seed);
    // Plant decorations
    for z in 0..16usize {
        for x in 0..16usize {
            let col = &cols[z * 16 + x];
            let (wx, wz) = (ox + x as i32, oz + z as i32);
            // Floating plants on still, shallow water (`wild.rs`); frozen water is ice. Clay lies in
            // patches under shallow inland water, most in swamps, as real clay settles in still
            // water; the sea's floor is sand.
            if col.height < SEA {
                let shore = shore_biome(g, wx, wz, col.biome);
                let floor = col.height - oy;
                if col.height >= SEA - CLAY_DEPTH && (0..16).contains(&floor) && clay(seed, wx, wz, shore) {
                    g.set(x, floor as usize, z, CLAY.id());
                }
                // Under it, plants of the bed standing in the water (eelgrass, kelp).
                bed(g, seed, [x, z], [wx, wz], col.height, col.biome);
                if col.height >= SEA - WILD_WATER_DEPTH {
                    for top in [SEA, SEA - 1] {
                        let y = top + 1 - oy;
                        if !(0..16).contains(&y) || g.get(x, y as usize, z) != 0 {
                            continue;
                        }
                        // The water it floats on: in this cube, or, on the cube's floor, the sea's
                        // surface (y 63 tops a cube, so a plant on it is always the next cube's),
                        // water unless the biome freezes it.
                        let water = if y >= 1 { WATER.is(g.get(x, y as usize - 1, z)) } else { top == SEA && !FROZEN.on_biome(col.biome) };
                        if water {
                            if let Some(s) = wild_at(seed, wx, wz, shore, wild::Ground::Water) {
                                g.set(x, y as usize, z, s);
                            }
                            break;
                        }
                    }
                }
                continue;
            }
            let y = col.height + 1 - oy;
            if !(0..16).contains(&y) {
                continue;
            }
            let y = y as usize;
            if g.get(x, y, z) != 0 {
                continue;
            }
            let h = hash(seed ^ 0xF10, wx as i64, 0, wz as i64);
            let roll = unit(h);
            // Plants go where they could be planted (soil tags); the biome decides how many.
            let b = col.biome;
            // Grass and flowers in patches, not spread evenly: a meadow noise (0 to 1, patches some
            // 14 blocks across) thickens the grass where it's high and thins it where it's low, the
            // biome's cover on average; flowers come in patches of one kind where it's highest.
            let meadow = ((fixed::perlin2(fixed::seed32(seed, 0x3EAD), wx * 16, wz * 16, 14 * 16) + 4096) as f64 / 8192.0).clamp(0.0, 1.0);
            let grass_chance = (GRASS_COVER
                .on_biome(b)
                .unwrap_or(vitals::forage::GRASS_COVER_ELSE) as f64
                * (0.35 + 1.3 * meadow))
                .min(0.9);
            let flower_chance = if meadow > FLOWER_MEADOW { 0.06 } else { 0.004 };
            // Loose rocks, flint and fallen branches (`first.rs`), on their own roll.
            let ground_u = unit(hash(seed ^ 0x6A0D, wx as i64, 0, wz as i64)) as f32;
            // What is under the lowest layer takes a look at the terrain: only if anything would grow.
            if y == 0 && roll >= (flower_chance + grass_chance) && wild_roll(seed, wx, wz) >= wild::MOST && ground_u >= vitals::first::MOST
            {
                continue;
            }
            let below = if y == 0 { crate::land::surface_block(g, wx, wz) } else { g.get(x, y - 1, z) };
            // Ponds in swamps and floodplains (`vitals::wild::POND_LANDS`): still water a block deep
            // where the ground is level, its floating plants on it.
            let near = [(1, 0), (-1, 0), (0, 1), (0, -1)].map(|(dx, dz)| (wx + dx, wz + dz, g.column(wx + dx, wz + dz)));
            // Steep ground goes bare: gravel where it falls three blocks to a neighbour (sand and the
            // frozen north keep their own ground); from four, the terrain's rock faces have made it
            // stone already (`land.rs`), and it isn't plant soil here.
            let fall = near
                .iter()
                .map(|(_, _, c)| col.height - c.height)
                .max()
                .unwrap_or(0);
            // The loose things lie on open soil, sand and gravel, thicker on steep ground and gravel
            // and round the spawn; fallen branches under the crowns; old bones instead of rocks
            // and flint in the bone lands, the frozen north among them (`vitals::first`).
            let bony = ground_u < vitals::first::MOST && vitals::first::BONE_LANDS.contains(&name_of(abi::entry_kind::BIOME, b));
            if ground_u < vitals::first::MOST
                && (bony || !FROZEN.on_biome(b))
                && (PLANT_SOIL.on_block(below) || SAND_TAG.on_block(below) || GRAVEL.is(below))
            {
                let stony = fall >= 2 || GRAVEL.is(below);
                let from_spawn = (((wx - spawn[0]) as f32).powi(2) + ((wz - spawn[2]) as f32).powi(2)).sqrt();
                if let Some(thing) = vitals::first::ground_at(ground_u, stony, under_crown(g, x, y, z), bony, from_spawn) {
                    if y >= 1 && fall >= 3 && PLANT_SOIL.on_block(below) {
                        g.set(x, y - 1, z, GRAVEL.id());
                    }
                    g.set(x, y, z, crate::first::ground_block(thing));
                    continue;
                }
            }
            if y >= 1 && fall >= 3 && PLANT_SOIL.on_block(below) && !FROZEN.on_biome(b) {
                g.set(x, y - 1, z, GRAVEL.id());
                continue;
            }
            if y >= 1
                && pond(seed, wx, wz, col.height, b)
                && PLANT_SOIL.on_block(below)
                && near.iter().all(|(_, _, c)| c.height >= col.height)
            {
                g.set(x, y - 1, z, WATER.id());
                if y >= 2 && clay(seed, wx, wz, b) {
                    g.set(x, y - 2, z, CLAY.id());
                }
                if let Some(s) = wild_at(seed, wx, wz, b, wild::Ground::Water) {
                    g.set(x, y, z, s);
                }
                continue;
            }
            // The wild plants first, on their own roll (`wild.rs`), in the frozen north too; a shore
            // is the sea's edge or a pond's (a pond's neighbour at its own height).
            let shore = (col.height == SEA && near.iter().any(|(_, _, c)| c.height < SEA))
                || near
                    .iter()
                    .any(|&(nx, nz, c)| c.height == col.height && pond(seed, nx, nz, c.height, c.biome));
            // Floor patches (`FLOOR`): a forest's leaf litter and bare dirt, the dry lands' gravel,
            // in clumps (a noise some 6 blocks across holds them, with holes), twice as thick under
            // the crowns and half in the open.
            let below = match FLOOR.on_biome(b) {
                Some(floor) if y >= 1 && !shore && (PLANT_SOIL.on_block(below) || SAND_TAG.on_block(below)) => {
                    let cover = FLOOR_COVER.on_biome(b).unwrap_or(0.0) as f64 * if under_crown(g, x, y, z) { 2.0 } else { 0.5 };
                    let patch = fixed::perlin2(fixed::seed32(seed, 0xF1C0), wx * 16, wz * 16, 6 * 16) > 0;
                    let fh = hash(seed ^ 0xF1C, wx as i64, 0, wz as i64);
                    match floor.pick(fh >> 32) {
                        Some(f) if patch && unit(fh) < 2.0 * cover && f != below => {
                            g.set(x, y - 1, z, f);
                            f
                        }
                        _ => below,
                    }
                }
                _ => below,
            };
            let ground = if PLANT_SOIL.on_block(below) {
                Some(if shore { wild::Ground::Shore } else { wild::Ground::Soil })
            } else if SAND_TAG.on_block(below) {
                Some(wild::Ground::Sand)
            } else {
                None
            };
            // A sandy lake or river shore grows the shore's reeds too, else its sand's own plants
            // (a beach's washed-up kelp).
            let sandy_shore = shore && ground == Some(wild::Ground::Sand);
            let pick = if sandy_shore { wild_at(seed, wx, wz, b, wild::Ground::Shore) } else { None };
            if let Some(s) = pick.or_else(|| ground.and_then(|gr| wild_at(seed, wx, wz, b, gr))) {
                g.set(x, y, z, s);
                // A mound's crown on it, in one place in two (`wild::upper_of`).
                if let Some(u) = crate::wild::upper_of(s, unit(hash(seed ^ 0x7E3, wx as i64, 1, wz as i64)) as f32)
                    && y + 1 < 16
                {
                    g.set(x, y + 1, z, u);
                }
                continue;
            }
            if FROZEN.on_biome(b) {
                continue;
            }
            if PLANT_SOIL.on_block(below) {
                if roll < flower_chance {
                    // The biome's own flowers (`vitals::wild::FLOWERS`), else its `FLOWERS` (another
                    // mod's biome), else dandelions and roses.
                    let kind = flower_kind(seed, wx, wz, h);
                    let flower = crate::wild::flower(name_of(abi::entry_kind::BIOME, b), wild::Ground::Soil, kind).unwrap_or_else(|| {
                        FLOWERS
                            .on_biome(b)
                            .map_or_else(|| [DANDELION.id(), ROSE.id()][kind as usize % 2], |l| l.pick(kind).unwrap_or(0))
                    });
                    if flower != 0 {
                        g.set(x, y, z, flower);
                    }
                } else if roll < flower_chance + grass_chance {
                    // Part of it two blocks tall where the biome grows it (`TALL_GRASS_SHARE`).
                    let double = TALL_GRASS_SHARE.on_biome(b).unwrap_or(0.0) as f64;
                    if y + 1 < 16 && g.get(x, y + 1, z) == 0 && unit(hash(seed ^ 0x2A55, wx as i64, 0, wz as i64)) < double {
                        g.set(x, y, z, state_with(TALL_GRASS_DOUBLE.id(), "half", "lower").unwrap_or(TALL_GRASS_DOUBLE.id()));
                        g.set(x, y + 1, z, state_with(TALL_GRASS_DOUBLE.id(), "half", "upper").unwrap_or(TALL_GRASS_DOUBLE.id()));
                    } else {
                        g.set(x, y, z, TALL_GRASS.id());
                    }
                } else {
                    // The bare soil: mushrooms at the foot of a trunk (`MUSHROOMS`), then ferns
                    // (`FERNS`), three times as thick round a trunk.
                    let trunk = beside_log(g, x, y, z);
                    let near = unit(hash(seed ^ 0xFE4, wx as i64, 0, wz as i64));
                    let mushroom = MUSHROOM_COVER.on_biome(b).filter(|_| trunk).unwrap_or(0.0) as f64;
                    let ferns = FERNS.on_biome(b).unwrap_or(0.0) as f64 * if trunk { 3.0 } else { 1.0 };
                    if near < mushroom {
                        if let Some(m) = MUSHROOMS.on_biome(b).and_then(|l| l.pick(h >> 20)) {
                            g.set(x, y, z, m);
                        }
                    } else if near < mushroom + ferns {
                        let fern = FERN_KINDS
                            .on_biome(b)
                            .and_then(|l| l.pick(h >> 20))
                            .unwrap_or(FERN.id());
                        g.set(x, y, z, fern);
                    } else if near < mushroom + ferns + BIG_LEAVES.on_biome(b).unwrap_or(0.0) as f64
                        && y + 1 < 16
                        && g.get(x, y + 1, z) == 0
                        && let Some(big) = BIG_LEAF_KINDS.on_biome(b).and_then(|l| l.pick(h >> 20))
                    {
                        // A big-leaved plant two blocks tall (`BIG_LEAVES`).
                        g.set(x, y, z, state_with(big, "half", "lower").unwrap_or(big));
                        g.set(x, y + 1, z, state_with(big, "half", "upper").unwrap_or(big));
                    }
                }
            } else if SAND_TAG.on_block(below)
                && roll < flower_chance
                && let Some(flower) =
                    crate::wild::flower(name_of(abi::entry_kind::BIOME, b), wild::Ground::Sand, flower_kind(seed, wx, wz, h))
            {
                // The desert's and the beach's flowers, on their sand.
                g.set(x, y, z, flower);
            } else if CACTUS_SOIL.on_block(below) && DRY.on_biome(b) && roll < 0.006 && y + 2 < 16 {
                let n = 1 + (h >> 40) % 3;
                for k in 0..n as usize {
                    g.set(x, y + k, z, CACTUS.id());
                }
            } else if CACTUS_SOIL.on_block(below) && DRY.on_biome(b) && roll < 0.02 {
                // Dead bushes: the desert's few sticks.
                g.set(x, y, z, DEAD_BUSH.id());
            } else if SUGAR_CANE_SOIL.on_block(below)
                && col.height == SEA
                && roll < 0.08
                && y + 2 < 16
                && [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|&(dx, dz)| g.column(wx + dx, wz + dz).height < SEA)
            {
                let n = 1 + (h >> 40) % 3;
                for k in 0..n as usize {
                    g.set(x, y + k, z, SUGAR_CANE.id());
                }
            }
        }
    }
    // Clay patches in shallow water (rivers, lakes, ocean shores), replacing the top blocks in
    // bodies of water
    for z in 0..16usize {
        for x in 0..16usize {
            let col = &cols[z * 16 + x];
            // Make sure we only generate clay patches in shallower waters
            if col.height >= SEA || col.height < SEA - 6 {
                continue;
            }
            let y = col.height - oy;
            if !(0..16).contains(&y) {
                continue;
            }
            let y = y as usize;
            let (wx, wz) = (ox + x as i32, oz + z as i32);
            // Clay patch size based on noise scale
            let clay = fixed::fbm(fixed::seed32(seed, 0xC1A7), wx * 16, wz * 16, 18 * 16, 3);
            // Threshold for clay generation
            if clay > fixed::ONE / 4 {
                let surface = g.get(x, y, z);
                if SAND.is(surface) || GRAVEL.is(surface) {
                    g.set(x, y, z, CLAY.id());
                }
            }
        }
    }
}

/// The plant of the bed (eelgrass, kelp; `vitals::wild::Ground::Bed`) on a column whose floor is
/// at `floor` under the sea, in this cube's part of it: in the water just over the floor, kelp on
/// up to its height, a block below the surface at most.
pub(super) fn bed(g: &mut Gen, seed: u64, [x, z]: [usize; 2], [wx, wz]: [i32; 2], floor: i32, biome: u32) {
    let roll = wild_roll(seed, wx, wz);
    if roll >= wild::MOST {
        return;
    }
    let Some(i) = wild::pick(name_of(abi::entry_kind::BIOME, biome), wild::Ground::Bed, roll) else { return };
    let plant = &wild::WILD[i];
    let depth = SEA - floor;
    if !wild::depths(plant.block).contains(&depth) {
        return;
    }
    let s = crate::wild::state(i);
    if s == u32::MAX {
        return;
    }
    let tall =
        if plant.form == wild::Form::Column { 1 + (hash(seed ^ 0xCE1, wx as i64, 0, wz as i64) % (depth - 1) as u64) as i32 } else { 1 };
    let oy = g.origin().y;
    for y in floor + 1..=floor + tall {
        let ly = y - oy;
        if (0..16).contains(&ly) && WATER.is(g.get(x, ly as usize, z)) {
            // A kelp column's top shows its tip (`wild.rs`, `top`); the blocks under it the stalk.
            let here = if plant.form == wild::Form::Column && y < floor + tall { state_with(s, "top", "false").unwrap_or(s) } else { s };
            g.set(x, ly as usize, z, here);
        }
    }
}

/// Whether leaves hang over this spot of the cube (a tree's crown, a shrub), looking up to the
/// cube's top.
pub(super) fn under_crown(g: &mut Gen, x: usize, y: usize, z: usize) -> bool {
    (y + 1..16).any(|k| LEAVES.on_block(g.get(x, k, z)))
}

/// Whether a trunk stands beside this spot of the cube (the four sides and the corners).
pub(super) fn beside_log(g: &mut Gen, x: usize, y: usize, z: usize) -> bool {
    (-1..=1i32).any(|dz| {
        (-1..=1i32).any(|dx| {
            let (nx, nz) = (x as i32 + dx, z as i32 + dz);
            (0..16).contains(&nx) && (0..16).contains(&nz) && LOGS.on_block(g.get(nx as usize, y, nz as usize))
        })
    })
}

/// Whether clay lies under still water at this column of `biome`'s: in 4 x 4 patches, half the
/// swamp's floor, a fifth of other inland water's, none under the sea (sand there).
pub(super) fn clay(seed: u64, wx: i32, wz: i32, biome: u32) -> bool {
    let share = match name_of(abi::entry_kind::BIOME, biome) {
        "base:swamp" => 0.5,
        "base:ocean" | "base:beach" => 0.0,
        _ => 0.2,
    };
    unit(hash(seed ^ 0xC1A, (wx >> 2) as i64, 0, (wz >> 2) as i64)) < share
}

/// The biome water at a column belongs to for its plants: the land's beside it. Water deeper
/// than a block is the ocean to the world (`BiomeRule`), but a swamp's pond or a floodplain's
/// river is theirs; open water far from land stays its own.
pub(super) fn shore_biome(g: &Gen, wx: i32, wz: i32, own: u32) -> u32 {
    [(3, 0), (-3, 0), (0, 3), (0, -3), (6, 0), (-6, 0), (0, 6), (0, -6)]
        .iter()
        .map(|&(dx, dz)| g.column(wx + dx, wz + dz))
        .find(|c| c.height >= SEA)
        .map_or(own, |c| c.biome)
}

/// Whether the column at `wx`, `wz` (ground at `height`, in `biome`) is a pond's: round ponds of
/// radius 2 to 4, one in the share of 8 x 8 cells `vitals::wild::POND_LANDS` gives its biome, on
/// ground above the sea (the swamps' own low land; where it isn't level, the caller leaves it dry).
pub(super) fn pond(seed: u64, wx: i32, wz: i32, height: i32, biome: u32) -> bool {
    if height < SEA {
        return false;
    }
    let name = name_of(abi::entry_kind::BIOME, biome);
    let Some(&(_, share)) = wild::POND_LANDS.iter().find(|(b, _)| *b == name) else { return false };
    let (cx, cz) = (wx.div_euclid(8), wz.div_euclid(8));
    (-1..=1).any(|i| {
        (-1..=1).any(|k| {
            let (ox, oz) = (cx + i, cz + k);
            let h = hash(seed ^ 0x90D, ox as i64, 0, oz as i64);
            if unit(h) >= share as f64 {
                return false;
            }
            let (px, pz) = (ox * 8 + 2 + ((h >> 20) % 4) as i32, oz * 8 + 2 + ((h >> 28) % 4) as i32);
            let r = 2 + ((h >> 36) % 3) as i32;
            (wx - px).pow(2) + (wz - pz).pow(2) <= r * r
        })
    })
}
