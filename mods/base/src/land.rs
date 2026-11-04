//! The overworld's land (`reports/world-looks.md`, change 5): its shape, where each biome stands,
//! snow on the heights, and bare rock on steep faces. The numbers and the shapes are in
//! `base_api::land`; other mods edit the terrain by the names here (`base:desert`,
//! `base:mountains`, `base:snow_1`, the features `mountains`, `mesas`, `rivers`).

use base_api::land::{self as l, SNOW_LINES};
use weft_sdk::terrain::*;
use weft_sdk::*;

use crate::blocks::*;
use crate::worldgen::SEA;

/// The overworld's terrain.
pub fn terrain(r: &mut Registry) {
    let mut t = r
        .terrain(&crate::OVERWORLD)
        .stone("stone")
        .base_height(SEA as f64)
        .continent_scale(l::CONTINENT_SCALE)
        .sea(SEA, "water");
    // Biomes shape the land through their climate, not their borders, so nothing seams: the cold
    // flattens hills into tundra, deserts get dunes and mesas, rivers run where it's wet enough.
    for f in l::features() {
        t = t.feature(f);
    }
    // Biomes bring their climate and surface; rules only add where else they must hold. The sea
    // is the ocean's out to the coast; water inland (rivers, lakes) belongs to the land around it
    // and lies on its own bed (`RIVER_BEDS`).
    t = t
        .biome(
            BiomeRule::new("ocean")
                .height(..=SEA - 2)
                .continent(..l::OCEAN_CONTINENT),
        )
        .biome(
            BiomeRule::new("snowy_plains")
                .named("snowy_coast")
                .height(..=SEA + 1)
                .continent(..0.1),
        )
        .biome(BiomeRule::new("beach").height(..=SEA + 1).continent(..0.1));
    // Snow on the heights, the line lower in the cold and in drifts (`base_api::land::snowy`); the
    // ground is the mountains' (their climate), only its surface is snow.
    let mut low = f64::NEG_INFINITY;
    for (i, &(below, line)) in SNOW_LINES.iter().enumerate() {
        let snow = |name: String| {
            BiomeRule::new("mountains")
                .named(&name)
                .temperature(low..below)
                .surface("snow", "snow", l::SNOW_DEPTH - 1)
                .bare_above(i32::MAX)
        };
        t = t.biome(snow(format!("snow_{i}")).height(line..)).biome(
            snow(format!("snow_{i}_drift"))
                .height(line - l::SNOW_DRIFT..)
                .feature("snow_drift", l::SNOW_DRIFT_FROM..),
        );
        low = below;
    }
    // A creek in the mountains lies on gravel.
    t = t.biome(
        BiomeRule::new("mountains")
            .feature("mountains", l::MOUNTAIN_BIOME..)
            .deep_top("gravel", SEA),
    );
    // Giant sequoia groves, on the mountains' front slopes (`base_api::land::SEQUOIA_AREA`).
    t = t.biome(
        BiomeRule::new("sequoia_grove")
            .height(SEA + l::SEQUOIA_RISE.0..=SEA + l::SEQUOIA_RISE.1)
            .feature("sequoia_groves", l::SEQUOIA_GROVE..),
    );
    for (biome, bed) in l::RIVER_BEDS {
        let mut rule = BiomeRule::new(biome).deep_top(bed, SEA);
        // Swamps only on low land, where water stands.
        if biome == "swamp" {
            rule = rule.height(..=SEA + 12).continent(0.1..);
        }
        t = t.biome(rule);
    }
    t.cave_biome("deep_caves", 24, 96.0, 0.25)
        .caves(Caves::new("lava", i32::MIN, 11))
        // Distant terrain runs no decorations: it asks this what its steep faces show.
        .faces(l::SNOW_FALL, |f| {
            let [x, _, z] = f.pos();
            let rocks = l::rock_set(&l::fields(f.seed()), x * 16, z * 16, f.height(), SAND.is(f.surface_block()));
            if let Some(rock) = bare(f.seed(), f.pos(), f.block(), f.fall(), f.biome(), rocks) {
                f.set(rock);
            }
        });

    r.decorate(rock_faces).dimension(&crate::OVERWORLD);
}

static WET: Tag = Tag::named("wet");
static WOODS: Tag = Tag::named("forest");

/// Whether ground that falls `fall` blocks to its lowest neighbour is a bare rock face (no soil:
/// no plants, trees or boulders stand there).
pub fn rock_face(fall: i32) -> bool {
    fall >= l::ROCK_FALL
}

/// Whether a scatter's site (`Site::pos`, the block above the ground) in the overworld stands on a
/// bare rock face, for trees and boulders to skip: a site can't see the cube, so this asks the
/// terrain for the neighbouring columns.
pub fn site_on_rock(seed: u64, dimension: u32, [x, _, z]: [i32; 3]) -> bool {
    if dimension != crate::OVERWORLD.id() {
        return false;
    }
    let h = crate::OVERWORLD.column(seed, x, z).height;
    let fall = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .map(|&(dx, dz)| h - crate::OVERWORLD.column(seed, x + dx, z + dz).height)
        .max()
        .unwrap_or(0);
    rock_face(fall)
}

/// Whether a scatter's site in the overworld stands at the foot of a wall: ground `WALL_RISE` or
/// more blocks higher within `WALL_REACH` (a karst tower, a cliff, a rock face), where a tree's
/// crown would press against the rock; trees and shrubs skip it, boulders (fallen rock) don't.
/// It looks along the eight directions at 2 and 4 blocks, and beside the site: a crown reaches
/// three to five blocks (the first rule, 2 blocks, left the towers' crowns as they were).
pub fn site_under_wall(seed: u64, dimension: u32, [x, _, z]: [i32; 3]) -> bool {
    if dimension != crate::OVERWORLD.id() {
        return false;
    }
    let h = crate::OVERWORLD.column(seed, x, z).height;
    let high = |dx: i32, dz: i32| crate::OVERWORLD.column(seed, x + dx, z + dz).height - h >= WALL_RISE;
    const DIRS: [(i32, i32); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];
    DIRS[..4].iter().any(|&(dx, dz)| high(dx, dz))
        || [WALL_REACH / 2, WALL_REACH]
            .iter()
            .any(|&d| DIRS.iter().any(|&(dx, dz)| high(dx * d, dz * d)))
}

/// A wall this many blocks over a site's ground, this near, keeps trees off it (`site_under_wall`).
pub const WALL_RISE: i32 = 4;
pub const WALL_REACH: i32 = 4;

/// How far the ground of column (`wx`, `wz`), `height` high, falls to its lowest neighbour.
pub fn fall(g: &Gen, wx: i32, wz: i32, height: i32) -> i32 {
    [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .map(|&(dx, dz)| height - g.column(wx + dx, wz + dz).height)
        .max()
        .unwrap_or(0)
}

/// What a rock face makes of `block` at `pos` in `biome`, if it's soil or stone that goes bare
/// there (`fall` blocks to the lowest neighbour); `None` keeps it. The face shows its column's
/// rocks (`rocks`, `base_api::land::rock_set`): strata, slate, granite or limestone in patches.
fn bare(seed: u64, pos: [i32; 3], block: u32, fall: i32, biome: u32, rocks: l::RockSet) -> Option<u32> {
    let face = |pos| rock(rocks.block(seed, pos, WET.on_biome(biome) || WOODS.on_biome(biome)));
    if fall >= l::SNOW_FALL && SNOW.is(block) && fall < rocks.bare_from() {
        return Some(face(pos));
    }
    if fall < rocks.bare_from() {
        return None;
    }
    let soil = [GRASS.id(), SNOWY_GRASS.id(), DIRT.id(), PODZOL.id(), SNOW.id(), GRAVEL.id(), SAND.id(), SANDSTONE.id(), STONE.id()];
    soil.contains(&block).then(|| face(pos))
}

/// A rock's block by its name in `base_api::land`.
fn rock(name: &str) -> u32 {
    match name {
        "granite" => GRANITE.id(),
        "slate" => SLATE.id(),
        "limestone" => LIMESTONE.id(),
        "sandstone" => SANDSTONE.id(),
        "ochre_sandstone" => OCHRE_SANDSTONE.id(),
        "red_sandstone" => RED_SANDSTONE.id(),
        "cream_sandstone" => CREAM_SANDSTONE.id(),
        "banded_clay" => BANDED_CLAY.id(),
        "weathered_limestone" => WEATHERED_LIMESTONE.id(),
        "cobblestone" => COBBLESTONE.id(),
        "mossy_cobblestone" => MOSSY_COBBLESTONE.id(),
        "banded_clay_coal_ore" => BANDED_CLAY_COAL_ORE.id(),
        "red_sandstone_iron_ore" => RED_SANDSTONE_IRON_ORE.id(),
        "granite_gold_ore" => GRANITE_GOLD_ORE.id(),
        "granite_iron_ore" => GRANITE_IRON_ORE.id(),
        _ => STONE.id(),
    }
}

/// The rocks of column (`wx`, `wz`)'s face, its top being `top`.
fn rocks(g: &Gen, wx: i32, wz: i32, height: i32, top: u32) -> l::RockSet {
    l::rock_set(&l::fields(g.seed()), wx * 16, wz * 16, height, SAND.is(top))
}

/// The block at the top of column (`wx`, `wz`) once its face went bare: what a decoration in the
/// cube above reads instead of the terrain's (`Gen::surface_block`).
pub fn surface_block(g: &Gen, wx: i32, wz: i32) -> u32 {
    let top = g.surface_block(wx, wz);
    if g.dimension() != crate::OVERWORLD.id() {
        return top;
    }
    let c = g.column(wx, wz);
    let fall = fall(g, wx, wz, c.height);
    if fall < l::SNOW_FALL {
        return top;
    }
    bare(g.seed(), [wx, c.height, wz], top, fall, c.biome, rocks(g, wx, wz, c.height, top)).unwrap_or(top)
}

/// Steep faces go bare: where the ground falls `ROCK_FALL` blocks or more to a neighbour, its soil
/// and stone are rock as far down as the face shows, the column's own (`rock_set`): the mesas'
/// strata, slate in the cold, granite high up, limestone on the low cliffs, each in patches with
/// stone or cobbles; snow slides off from `SNOW_FALL`. Before the plants (`worldgen::decorate`), which then find no soil there.
fn rock_faces(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let seed = g.seed();
    let [ox, oy, oz] = g.origin();
    let max_h = cols.iter().map(|c| c.height).max().unwrap_or(0);
    let min_h = cols.iter().map(|c| c.height).min().unwrap_or(0);
    if oy > max_h || oy + 15 < min_h - l::ROCK_MAX_DEPTH {
        return;
    }
    for z in 0..16usize {
        for x in 0..16usize {
            let col = cols[z * 16 + x];
            let (wx, wz) = (ox + x as i32, oz + z as i32);
            let h = col.height;
            if h < SEA || h < oy || h - l::ROCK_MAX_DEPTH > oy + 15 {
                continue;
            }
            let fall = fall(g, wx, wz, h);
            if fall < l::SNOW_FALL {
                continue;
            }
            let set = rocks(g, wx, wz, h, g.surface_block(wx, wz));
            let depth = fall.clamp(l::ROCK_DEPTH, l::ROCK_MAX_DEPTH);
            for wy in (h - depth).max(oy)..=h.min(oy + 15) {
                let y = (wy - oy) as usize;
                if let Some(rock) = bare(seed, [wx, wy, wz], g.get(x, y, z), fall, col.biome, set) {
                    g.set(x, y, z, rock);
                }
            }
        }
    }
}
