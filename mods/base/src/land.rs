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
    let t = land(r.terrain(&crate::OVERWORLD), l::features());
    let t = coasts(t);
    let mut t = heights(t);
    // Giant sequoia groves, on the mountains' front slopes (`base_api::land::SEQUOIA_AREA`).
    t = t.biome(
        BiomeRule::new("sequoia_grove")
            .height(SEA + l::SEQUOIA_RISE.0..=SEA + l::SEQUOIA_RISE.1)
            .field(l::SEQUOIA_FIELD, l::SEQUOIA_GROVE..),
    );
    for (biome, bed) in l::RIVER_BEDS {
        let mut rule = BiomeRule::new(biome).deep_top(bed, SEA);
        // Swamps only on low land, where water stands.
        if biome == "swamp" {
            rule = rule.height(..=SEA + 12).continent(0.1..);
        }
        // Old growth in its patches of the temperate wet land, up to its height
        // (`base_api::land::OLD_GROWTH_AREA`).
        if biome == "old_growth_forest" {
            rule = rule
                .height(..=SEA + l::OLD_GROWTH_TOP)
                .field(l::OLD_GROWTH_FIELD, l::OLD_GROWTH_FROM..);
        }
        t = t.biome(rule);
    }
    finish(t);

    r.decoration(rock_faces).dimension(&crate::OVERWORLD);
    r.decoration(|g| l::with_mc_land(|| rock_faces(g))).dimension(&crate::MC_FOREST);
    r.decoration(cave_mouths).dimension(&crate::OVERWORLD);
    r.decoration(|g| l::with_mc_land(|| cave_mouths(g))).dimension(&crate::MC_FOREST);
    r.decoration(glaciers).dimension(&crate::OVERWORLD);
    r.decoration(|g| l::with_mc_land(|| glaciers(g))).dimension(&crate::MC_FOREST);
    r.decoration(tarn_water).dimension(&crate::OVERWORLD);
    r.decoration(|g| l::with_mc_land(|| tarn_water(g))).dimension(&crate::MC_FOREST);
    r.decoration(fray_borders).dimension(&crate::OVERWORLD);
    r.decoration(|g| l::with_mc_land(|| fray_borders(g))).dimension(&crate::MC_FOREST);
    r.decoration(sea_sand).dimension(&crate::OVERWORLD);
    r.decoration(|g| l::with_mc_land(|| sea_sand(g))).dimension(&crate::MC_FOREST);
    r.decoration(coral).dimension(&crate::OVERWORLD);
    r.decoration(landmarks).dimension(&crate::OVERWORLD);
    r.decoration(tropical_water).dimension(&crate::OVERWORLD);
}

/// The overworld's land, for it and for the test world on the same seed (`MC_FOREST`): its stone,
/// sea and continents, the fields the rules ask for (the mountains, the sequoia groves, the snow's
/// drifts), then the features. Biomes shape the land through their climate, not their borders, so
/// nothing seams: the cold flattens hills into tundra, deserts get dunes and mesas, rivers run
/// where it's wet enough.
fn land(t: TerrainBuilder<'_>, features: Vec<Feature>) -> TerrainBuilder<'_> {
    let mut t = t.stone("stone").base_height(SEA as f64).continent_scale(l::CONTINENT_SCALE).sea(SEA, "water");
    for (name, field) in l::rule_fields() {
        t = t.field(name, field);
    }
    for f in features {
        t = t.feature(f);
    }
    t
}

/// The coasts' rules: the warm seas' reefs and lagoons, the ocean, the cold coast's snow and the
/// beach.
fn coasts(t: TerrainBuilder<'_>) -> TerrainBuilder<'_> {
    // Biomes bring their climate and surface; rules only add where else they must hold. The sea
    // is the ocean's out to the coast; water inland (rivers, lakes) belongs to the land around it
    // and lies on its own bed (`RIVER_BEDS`).
    // Warm shallow seas first (`base_api::land::reef_part`): the reef's top 2 under the surface,
    // the lagoon's floor 3-5 under, in warm water near the coasts.
    t
        .biome(
            BiomeRule::new("coral_reef")
                .height(SEA - 2..=SEA - 2)
                .temperature(l::REEF_WARM_RULE..)
                .continent(..l::REEF_COAST),
        )
        .biome(
            BiomeRule::new("lagoon")
                .height(SEA - l::REEF_LAGOON.1..=SEA - l::REEF_LAGOON.0)
                .temperature(l::REEF_WARM_RULE..)
                .continent(..l::REEF_COAST),
        )
        .biome(
            BiomeRule::new("ocean")
                .height(..=SEA - 2)
                .continent(..l::OCEAN_CONTINENT),
        )
        .biome(
            BiomeRule::new("snowy_plains")
                .named("snowy_coast")
                .height(..=SEA + 1)
                .continent(..l::BEACH_CONTINENT),
        )
        .biome(
            BiomeRule::new("beach")
                .height(..=SEA + 1)
                .continent(..l::BEACH_CONTINENT),
        )
}

/// The heights' rules: snow over its line, the mountains' own ground.
fn heights(mut t: TerrainBuilder<'_>) -> TerrainBuilder<'_> {
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
                .field(l::SNOW_DRIFT_FIELD, l::SNOW_DRIFT_FROM..),
        );
        low = below;
    }
    // A creek in the mountains lies on gravel.
    t.biome(
        BiomeRule::new("mountains")
            .field(l::MOUNTAINS_FIELD, l::MOUNTAIN_BIOME..)
            .deep_top("gravel", SEA),
    )
}

/// What every terrain on the overworld's land ends with: the deep caves, lava, the far crowns'
/// leaves and the steep faces' bare rock.
fn finish(t: TerrainBuilder<'_>) {
    t.cave_biome("deep_caves", 24, 96.0, 0.25)
        .caves(Caves::new("lava", i32::MIN, 11))
        // Distant terrain runs no decorations: it asks this what its steep faces show.
        // Far crowns' leaves by the season (a bloom over out of its months).
        .far_leaves(crate::blocks::far_leaves)
        .faces(l::SNOW_FALL, |f| {
            let [x, _, z] = f.pos();
            // A steep cell is asked once for its top and once per wall band down, the same column
            // each time: its rocks (the climate, the areas, the surface) are worked out once.
            let key = (f.seed(), x, z, f.height());
            let rocks = FACE_ROCKS.with(|c| match c.get() {
                Some((k, rocks)) if k == key => rocks,
                _ => {
                    let rocks = l::rock_set(&l::fields(f.seed()), x * 16, z * 16, f.height(), SAND.is(f.surface_block()));
                    c.set(Some((key, rocks)));
                    rocks
                }
            });
            if let Some(rock) = bare(f.seed(), f.pos(), f.block(), f.fall(), f.biome(), rocks) {
                f.set(rock);
            }
        });
}

/// The Minecraft forest test world's terrain (`base_api::land::MC_MIX`): the overworld's land, its
/// coasts and heights, and the rest shared out among Minecraft's temperate forest biomes by the mix
/// field, each with its river bed (`RIVER_BEDS`).
pub fn mc_forest_terrain(r: &mut Registry) {
    let t = land(r.terrain(&crate::MC_FOREST), l::mc_features()).field(l::MC_MIX_FIELD, l::mc_mix_field());
    let mut t = heights(coasts(t));
    let cuts = l::mc_mix_cuts();
    let mut low = f64::NEG_INFINITY;
    for (i, &(biome, _)) in l::MC_MIX.iter().enumerate() {
        let bed = l::RIVER_BEDS.iter().find(|e| e.0 == biome).map_or("sand", |e| e.1);
        let rule = BiomeRule::new(biome).named(&format!("mc_{biome}")).deep_top(bed, SEA);
        t = t.biome(if i + 1 < l::MC_MIX.len() { rule.field(l::MC_MIX_FIELD, low..cuts[i]) } else { rule });
        low = cuts[i];
    }
    finish(t);
}

/// A steep column (seed, x, z, height) and its rocks.
type FaceRocks = Option<((u64, i32, i32, i32), l::RockSet)>;

thread_local! {
    /// The last steep column's rocks (`faces`).
    static FACE_ROCKS: std::cell::Cell<FaceRocks> = const { std::cell::Cell::new(None) };
}

static WET: Tag = Tag::named("wet");
static WOODS: Tag = Tag::named("forest");

/// Whether ground that falls `fall` blocks to its lowest neighbour is a bare rock face (no soil:
/// no plants, trees or boulders stand there).
pub fn rock_face(fall: i32) -> bool {
    fall >= l::ROCK_FALL
}

/// Whether a dimension stands on the overworld's land (the overworld, and the test world on its
/// seed, `MC_FOREST`): its columns' heights are the overworld's (`OVERWORLD.column`).
pub fn overworld_land(dimension: u32) -> bool {
    dimension == crate::OVERWORLD.id() || dimension == crate::MC_FOREST.id()
}

/// The dimension whose columns a site in `dimension` stands on (the test world's own: its rivers
/// differ, `base_api::land::MC_RIVERS`).
fn land_of(dimension: u32) -> &'static Dimension {
    if dimension == crate::MC_FOREST.id() { &crate::MC_FOREST } else { &crate::OVERWORLD }
}

/// Whether a scatter's site (`Site::pos`, the block above the ground) in the overworld stands on a
/// bare rock face, for trees and boulders to skip: a site can't see the cube, so this asks the
/// terrain for the neighbouring columns.
pub fn site_on_rock(seed: u64, dimension: u32, [x, _, z]: [i32; 3]) -> bool {
    if !overworld_land(dimension) {
        return false;
    }
    let dim = land_of(dimension);
    let h = dim.column(seed, x, z).height;
    let fall = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .map(|&(dx, dz)| h - dim.column(seed, x + dx, z + dz).height)
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
    if !overworld_land(dimension) {
        return false;
    }
    let dim = land_of(dimension);
    let h = dim.column(seed, x, z).height;
    let high = |dx: i32, dz: i32| dim.column(seed, x + dx, z + dz).height - h >= WALL_RISE;
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
        // The ground the borders fray into (`fray_borders`).
        "sand" => SAND.id(),
        "dirt" => DIRT.id(),
        "grass" => GRASS.id(),
        "podzol" => PODZOL.id(),
        "snowy_grass" => SNOWY_GRASS.id(),
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
    if !overworld_land(g.dimension()) {
        return top;
    }
    let c = g.column(wx, wz);
    let fall = fall(g, wx, wz, c.height);
    if fall >= l::SNOW_FALL
        && let Some(face) = bare(g.seed(), [wx, c.height, wz], top, fall, c.biome, rocks(g, wx, wz, c.height, top))
    {
        return face;
    }
    // Then the border's fray (`fray_borders`), on soil.
    if SOIL.contains(&top)
        && let Some((frayed, _)) = l::fray(&l::fields(g.seed()), wx * 16, wz * 16, c.height)
    {
        return rock(frayed);
    }
    top
}

/// The ground a border's fray may change (`fray_borders`).
static SOIL: std::sync::LazyLock<[u32; 5]> = std::sync::LazyLock::new(|| [GRASS.id(), SNOWY_GRASS.id(), PODZOL.id(), SAND.id(), DIRT.id()]);

/// Steep faces go bare: where the ground falls `ROCK_FALL` blocks or more to a neighbour, its soil
/// and stone are rock as far down as the face shows, the column's own (`rock_set`): the mesas'
/// strata, slate in the cold, granite high up, limestone on the low cliffs, each in patches with
/// stone or cobbles; snow slides off from `SNOW_FALL`. Before the plants (`worldgen::decorate`), which then find no soil there.
fn rock_faces(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let seed = g.seed();
    let [ox, oy, oz] = g.origin().to_array();
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

/// Biome borders fray (`base_api::land::fray`): near a border that changes the ground, the
/// neighbour's top and under block in patches, on soil only (rock faces, water and what stands
/// on the ground stay). After the rock faces, before the plants, which then grow on what it left.
fn fray_borders(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let [ox, oy, oz] = g.origin().to_array();
    let f = l::fields(g.seed());
    for z in 0..16usize {
        for x in 0..16usize {
            let h = cols[z * 16 + x].height;
            if h < SEA || h < oy || h - 2 > oy + 15 {
                continue;
            }
            let (wx, wz) = (ox + x as i32, oz + z as i32);
            let Some((top, under)) = l::fray(&f, wx * 16, wz * 16, h) else { continue };
            for wy in (h - 2).max(oy)..=h.min(oy + 15) {
                let y = (wy - oy) as usize;
                if SOIL.contains(&g.get(x, y, z)) {
                    g.set(x, y, z, rock(if wy == h { top } else { under }));
                }
            }
        }
    }
}

/// Sand on the low coast by the sea that the beach rule no longer reaches
/// (`base_api::land::sea_sand`, `BEACH_CONTINENT`): its grass and dirt, `SEA_SAND_DEEP` blocks,
/// become sand. After the borders' fray, before the plants (which then find sand).
fn sea_sand(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let [ox, oy, oz] = g.origin().to_array();
    if oy > SEA + 1 || oy + 15 < SEA - 1 - l::SEA_SAND_DEEP {
        return;
    }
    let heights: [i32; 256] = std::array::from_fn(|i| cols[i].height);
    let sand = l::sea_sand(&l::fields(g.seed()), ox, oz, &heights);
    for (i, _) in sand.iter().enumerate().filter(|s| *s.1) {
        let (x, z, h) = (i % 16, i / 16, heights[i]);
        for wy in (h - l::SEA_SAND_DEEP + 1).max(oy)..=h.min(oy + 15) {
            let y = (wy - oy) as usize;
            if SOIL.contains(&g.get(x, y, z)) {
                g.set(x, y, z, SAND.id());
            }
        }
    }
}

/// Cave mouths that read from the surface (`base_api::land::MOUTH_MIN`, `MOUTH_LIP`): where the
/// engine's caves open the ground (a tunnel grazing it leaves a slit a block or two wide in the
/// grass), the walls of the opening and a lip round it are the column's rock, and right beside it
/// the slit widens a block where nothing stands on the ground. After the rock faces, before the
/// borders' fray and the plants. Every block is decided by its column and the columns round it
/// (the caves' copy in `base_api::land::caves`), and a widened block only where the block over
/// it is in the same cube, so neighbouring cubes agree.
fn cave_mouths(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let [ox, oy, oz] = g.origin().to_array();
    let max_h = cols.iter().map(|c| c.height).max().unwrap_or(0);
    let min_h = cols.iter().map(|c| c.height).min().unwrap_or(0);
    // The surface and the few blocks under it.
    if oy > max_h || oy + 15 < min_h - 4 || max_h <= SEA {
        return;
    }
    let seed = g.seed();
    // The columns round the cube (as far as a mouth's reach and its five by five) and their
    // cells' corners.
    const M: i32 = l::MOUTH_LIP + 2;
    const N: i32 = 16 + 2 * M;
    let mut heights = std::collections::HashMap::new();
    let mut height = |g: &Gen, x: i32, z: i32| {
        *heights
            .entry((x, z))
            .or_insert_with(|| g.column(x, z).height)
    };
    let mut caves = l::caves::Caves::over(seed, (ox - M, ox + 16 + M), (oz - M, oz + 16 + M), (min_h.max(SEA) - 8, max_h + 8));
    let mut open = vec![false; (N * N) as usize];
    for j in 0..N {
        for i in 0..N {
            let (x, z) = (ox - M + i, oz - M + j);
            let h = height(g, x, z);
            if h <= SEA {
                continue;
            }
            let (gx, gz) = (x.div_euclid(4) * 4, z.div_euclid(4) * 4);
            let corners = [height(g, gx, gz), height(g, gx + 4, gz), height(g, gx, gz + 4), height(g, gx + 4, gz + 4)];
            open[(j * N + i) as usize] = caves.mouth(x, z, h, corners);
        }
    }
    if !open.iter().any(|&o| o) {
        return;
    }
    let at = |i: i32, j: i32| (0..N).contains(&i) && (0..N).contains(&j) && open[(j * N + i) as usize];
    // A mouth: enough open columns in its five by five.
    let mouth = |i: i32, j: i32| {
        at(i, j)
            && (-2..=2)
                .flat_map(|b| (-2..=2).map(move |a| (a, b)))
                .filter(|&(a, b)| at(i + a, j + b))
                .count()
                >= l::MOUTH_MIN
    };
    let soil = [GRASS.id(), SNOWY_GRASS.id(), PODZOL.id(), SAND.id(), DIRT.id(), GRAVEL.id(), STONE.id()];
    for z in 0..16i32 {
        for x in 0..16i32 {
            let (i, j) = (x + M, z + M);
            // How near a mouth (Chebyshev, blocks): 0 in one.
            let Some(near) =
                (0..=l::MOUTH_LIP).find(|&d| (-d..=d).any(|b| (-d..=d).any(|a| (a.abs() == d || b.abs() == d) && mouth(i + a, j + b))))
            else {
                continue;
            };
            let (wx, wz) = (ox + x, oz + z);
            if near == l::MOUTH_LIP
                && (noise::fixed::hash(noise::fixed::seed32(seed, 0x30C7), wx, 0, wz) & 4095) >= (l::MOUTH_LIP_PATCHES * 4096.0) as u32
            {
                continue;
            }
            let h = height(g, wx, wz);
            let col = cols[(z * 16 + x) as usize];
            let set = rocks(g, wx, wz, h, g.surface_block(wx, wz));
            let mossy = WET.on_biome(col.biome) || WOODS.on_biome(col.biome);
            // The rock: in the mouth its walls four deep, beside it three, at the lip's edge the top.
            let deep = match near {
                0 => 4,
                1 => 3,
                _ => 1,
            };
            for wy in (h - deep + 1).max(oy)..=h.min(oy + 15) {
                let y = (wy - oy) as usize;
                if soil.contains(&g.get(x as usize, y, z as usize)) {
                    g.set(x as usize, y, z as usize, rock(set.block(seed, [wx, wy, wz], mossy)));
                }
            }
            // Right beside a mouth the slit widens a block, where nothing stands on the ground.
            if near == 1 && h > oy && h < oy + 15 {
                let y = (h - oy) as usize;
                if g.get(x as usize, y + 1, z as usize) == 0 {
                    g.set(x as usize, y, z as usize, 0);
                }
            }
        }
    }
}

/// Landmarks built from blocks (`base_api::land::landmarks`): natural arches of their land's rock
/// and sea-cliff waterfalls, each cube its part. A waterfall is put as the engine's water settles
/// on its own: the spring a source, the block in front of it flowing (amount 7, `drop_off` 1),
/// under that a falling column down to the sea's surface, which doesn't spread over the sea.
fn landmarks(g: &mut Gen) {
    let [ox, oy, oz] = g.origin().to_array();
    let seed = g.seed();
    let f = l::fields(seed);
    let inside = |x: i32, y: i32, z: i32| (ox..ox + 16).contains(&x) && (oy..oy + 16).contains(&y) && (oz..oz + 16).contains(&z);
    for a in l::landmarks::arches_near(&f, ox + 8, oz + 8, 9) {
        let (x0, x1, z0, z1, top) = a.reach();
        if top < oy || a.base - 48 > oy + 15 {
            continue;
        }
        for wz in z0.max(oz)..=z1.min(oz + 15) {
            for wx in x0.max(ox)..=x1.min(ox + 15) {
                let ground = g.column(wx, wz).height;
                for wy in oy..oy + 16 {
                    if a.holds(wx, wy, wz, ground) {
                        g.set((wx - ox) as usize, (wy - oy) as usize, (wz - oz) as usize, rock(a.rocks.block(seed, [wx, wy, wz], false)));
                    }
                }
            }
        }
    }
    // The meteorites buried under old craters (`base_api::land::crater_meteorites`): First
    // hours' block (with the sky events), none until it is there.
    static METEORITE: Block = Block::named("meteorite");
    let meteorite = METEORITE.id();
    for [mx, my, mz] in l::crater_meteorites(&f, ox + 8, oz + 8, 8 + (l::CRATER_RADIUS.1 as f64 * l::CRATER_METEORITE_SPREAD) as i32 + 1) {
        if meteorite != u32::MAX && inside(mx, my, mz) {
            g.set((mx - ox) as usize, (my - oy) as usize, (mz - oz) as usize, meteorite);
        }
    }
    let water = WATER.id();
    for w in l::landmarks::waterfalls_near(&f, ox + 8, oz + 8, 9) {
        let [sx, sy, sz] = w.spring;
        if inside(sx, sy, sz) {
            g.set((sx - ox) as usize, (sy - oy) as usize, (sz - oz) as usize, water);
        }
        let [fx, fz] = w.front;
        for wy in (SEA + 1)..=sy {
            if inside(fx, wy, fz) {
                // The front: flowing, amount 7; under it falling.
                let state = if wy == sy { water + 1 } else { water + 8 };
                g.set((fx - ox) as usize, (wy - oy) as usize, (fz - oz) as usize, state);
            }
        }
    }
}

/// Glaciers in the mountains' high valleys (`base_api::land::glacier`): the top
/// `GLACIER_THICK` blocks of a valley's floor glacier ice, under the snow. After the rock faces.
fn glaciers(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let [ox, oy, oz] = g.origin().to_array();
    let max_h = cols.iter().map(|c| c.height).max().unwrap_or(0);
    let min_h = cols.iter().map(|c| c.height).min().unwrap_or(0);
    // High ground only: the lowest snow line less the glaciers' reach under it.
    let lowest = SNOW_LINES.iter().map(|&(_, line)| line).min().unwrap_or(0) - l::GLACIER_BELOW;
    if max_h < lowest || oy > max_h || oy + 15 < min_h - l::GLACIER_THICK {
        return;
    }
    let f = l::fields(g.seed());
    // Which columns are a glacier's floor, from the heights alone; then the ice.
    let mut floors = Vec::new();
    {
        let mut heights = std::collections::HashMap::new();
        let mut ground = |x: i32, z: i32| {
            *heights
                .entry((x, z))
                .or_insert_with(|| g.column(x, z).height)
        };
        for z in 0..16i32 {
            for x in 0..16i32 {
                let h = cols[(z * 16 + x) as usize].height;
                if h < lowest || h < oy || h - l::GLACIER_THICK >= oy + 16 {
                    continue;
                }
                if l::glacier(&f, ox + x, oz + z, h, &mut ground) {
                    floors.push((x as usize, z as usize, h));
                }
            }
        }
    }
    for (x, z, h) in floors {
        for wy in (h - l::GLACIER_THICK + 1).max(oy)..=h.min(oy + 15) {
            let y = (wy - oy) as usize;
            if g.get(x, y, z) != 0 {
                g.set(x, y, z, GLACIER_ICE.id());
            }
        }
    }
}

/// The tropical forest's still water (`base_api::land::tropical_water`): flooded forest by the
/// rivers and winding igarapés on the low floor, at the sea's level so it stays put. The ground
/// over that level goes where nothing stands on it (a trunk keeps its ground: the trees stand on
/// mounds in the water; the block over the ground must be in the same cube to be seen); the
/// block at the sea's level and one under it where deep are water whatever stands on them (under
/// kept ground it is a closed pocket). Before the plants.
fn tropical_water(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let tropical = crate::worldgen::TROPICAL_FOREST.id();
    let [ox, oy, oz] = g.origin().to_array();
    if !cols.iter().any(|c| c.biome == tropical) || oy > SEA + l::IGAPO_RISE.max(l::IGARAPE_RISE) || oy + 15 < SEA - 1 {
        return;
    }
    let f = l::fields(g.seed());
    for z in 0..16i32 {
        for x in 0..16i32 {
            let col = cols[(z * 16 + x) as usize];
            let h = col.height;
            if col.biome != tropical {
                continue;
            }
            let (wx, wz) = (ox + x, oz + z);
            let Some(depth) = l::tropical_water(&f, wx, wz, h) else { continue };
            let (xu, zu) = (x as usize, z as usize);
            // The ground over the sea's level, unless something stands on it.
            if h > SEA && h >= oy && h < oy + 15 && g.get(xu, (h + 1 - oy) as usize, zu) == 0 {
                for wy in (SEA + 1).max(oy)..=h {
                    g.set(xu, (wy - oy) as usize, zu, 0);
                }
            }
            for wy in (SEA - depth + 1)..=SEA {
                if (oy..oy + 16).contains(&wy) {
                    g.set(xu, (wy - oy) as usize, zu, WATER.id());
                }
            }
        }
    }
}

/// The tarns' water (`base_api::land::tarn_level`, `TARN_*`): from the bowl's bed up to its level,
/// still; the ring round it holds it. Their streams (`stream_bed`): a block of still water on the
/// channel's bed, falling where it drops. A cave under the bed is closed with stone so the water can't
/// run out. After the glaciers, before the plants.
fn tarn_water(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let [ox, oy, oz] = g.origin().to_array();
    let lowest = cols.iter().map(|c| c.height).min().unwrap_or(0);
    // Tarns stand high: under the lowest snow line, in the mountains; nothing here under the sea.
    if oy + 15 < lowest - 1 || oy + 15 <= SEA {
        return;
    }
    let f = l::fields(g.seed());
    for z in 0..16i32 {
        for x in 0..16i32 {
            let h = cols[(z * 16 + x) as usize].height;
            if h - 1 > oy + 15 {
                continue;
            }
            // A tarn's stream: a block of still water on its bed.
            let level = match l::tarn_level(&f, ox + x, oz + z) {
                Some(level) => level,
                None => match l::stream_bed(&f, ox + x, oz + z) {
                    Some(bed) => bed + 1,
                    None => continue,
                },
            };
            let (xu, zu) = (x as usize, z as usize);
            for wy in (h - 1)..=level {
                if !(oy..oy + 16).contains(&wy) {
                    continue;
                }
                let y = (wy - oy) as usize;
                if wy > h {
                    g.set(xu, y, zu, WATER.id());
                } else if g.get(xu, y, zu) == 0 {
                    g.set(xu, y, zu, STONE.id());
                }
            }
        }
    }
}

/// Coral (`base_api::land::REEF_*`; Art's blocks): the reef's top and the block under it coral, in
/// patches of brain, staghorn and porites coral, where the reef itself stands (`reef_top`; natural
/// shallows at its depth share its biome's name, not its coral); single coral heads on the
/// lagoon's floor; coral rubble in the reef's gaps and on its landward edge.
fn coral(g: &mut Gen) {
    let Some(&cols) = g.columns() else { return };
    let (reef, lagoon) = (crate::worldgen::CORAL_REEF.id(), crate::worldgen::LAGOON.id());
    let [ox, oy, oz] = g.origin().to_array();
    if oy > SEA || oy + 15 < SEA - 8 || !cols.iter().any(|c| c.biome == reef || c.biome == lagoon) {
        return;
    }
    let seed = g.seed();
    let f = l::fields(seed);
    let kinds = [BRAIN_CORAL_BLOCK.id(), STAGHORN_CORAL_BLOCK.id(), PORITES_CORAL_BLOCK.id()];
    for z in 0..16usize {
        for x in 0..16usize {
            let col = cols[z * 16 + x];
            let (wx, wz) = (ox + x as i32, oz + z as i32);
            let pick = noise::fixed::perlin2(noise::fixed::seed32(seed, 0xC07A), wx * 16, wz * 16, 9 * 16);
            let kind = kinds[if pick < -600 {
                0
            } else if pick < 700 {
                1
            } else {
                2
            }];
            let h = col.height;
            let mut put = |wy: i32, block: u32| {
                if (oy..oy + 16).contains(&wy) {
                    g.set(x, (wy - oy) as usize, z, block);
                }
            };
            if col.biome == reef && l::reef_top(&f, wx * 16, wz * 16) {
                put(h, kind);
                put(h - 1, kind);
            } else if (col.biome == reef || col.biome == lagoon || col.biome == crate::worldgen::OCEAN.id())
                && l::reef_rubble(&f, wx * 16, wz * 16)
            {
                put(h, CORAL_RUBBLE.id());
            } else if col.biome == lagoon && noise::fixed::hash(noise::fixed::seed32(seed, 0xC07B), wx, 0, wz) % 1000 < 25 {
                // A head on the sand, in the water.
                put(h + 1, kind);
            }
        }
    }
}
