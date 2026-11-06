//! Everything the engine's shaders and its Rust code must agree on, written once here: bit
//! layouts and mode numbers (`<GROUP>_<NAME>` constants), the structs of GPU buffers and each
//! shader's bindings. `prelude(shader)` puts them in front of the shader, so an engine shader, a
//! shader pack's replacement and a snippet only write functions.

macro_rules! shared {
    ($($(#[$doc:meta])* $group:ident { $($(#[$cdoc:meta])* $name:ident = $v:expr),* $(,)? })*) => {
        $($(#[$doc])* pub mod $group { $($(#[$cdoc])* pub const $name: u32 = $v;)* })*

        /// The constants as WGSL.
        pub fn wgsl() -> String {
            let mut s = String::new();
            $($(s.push_str(&format!("const {}_{}: u32 = {}u;\n", stringify!($group).to_uppercase(), stringify!($name), $group::$name));)*)*
            s
        }
    };
}

shared! {
    /// Block face record, four words. Word 0: x, y, z in the cube (4 bits each), direction,
    /// corner AO (2 bits per corner), tint kind. Word 1: texture, biome. Words 2 and 3: corner
    /// light, 16 bits per corner.
    face {
        POS_MASK = 15,
        Y_SHIFT = 4,
        Z_SHIFT = 8,
        DIR_SHIFT = 12,
        DIR_MASK = 7,
        AO_SHIFT = 16,
        AO_MASK = 3,
        TINT_SHIFT = 24,
        /// Quarter turns of the picture.
        TURN_SHIFT = 28,
        TURN_MASK = 3,
        /// Word 0: a see-through face with another of the same block behind it (its own back
        /// face not drawn, `CULL_SAME`): what shows through its holes is that block's leaves, not
        /// what lies past the crown, so packs and the built-in shader fill them (Minecraft's fancy
        /// leaves draw those faces).
        BACKED = 0x8000,
        TEX_MASK = 0xFFFF,
        BIOME_SHIFT = 16,
        /// Corner light: a channel per 4 bits, a corner per 16.
        LIGHT_MASK = 15,
        CORNER_SHIFT = 16,
    }
    /// Free-form quad record, eight words: four corners (10 bits per axis, 1/16 block from -2),
    /// texture and biome (like a face's word 1), uv (5 bits each, 1/16) and tint kind with
    /// `FLAT`, corner light (like a face's). Twelve free bits hold 1/8ths of those uv units, for
    /// an unwrap on a big texture: 8 of each word's top, the rest beside the tint. A shader
    /// pack's older copy goes by the whole units and is none the worse for it.
    quad {
        POS_MASK = 1023,
        POS_BITS = 10,
        /// Added to a corner in 1/16 block: corners reach 2 blocks outside the cube.
        POS_OFFSET = 32,
        UV_MASK = 31,
        UV_BITS = 5,
        UV_FINE_BITS = 3,
        UV_FINE_LOW_SHIFT = 30,
        UV_FINE_HIGH_SHIFT = 20,
        TINT_SHIFT = 24,
        /// Tint flag: no directional shading (plants).
        FLAT = 16,
    }
    /// Distant terrain record (`client::lod`), four words in the faces' pages: x, z in the tile
    /// (11 bits each), direction, tint kind; texture and biome like a face's; y (signed 16 bits)
    /// and the box's height; its width and depth.
    ///
    /// Direction `GRID` is a cell of a far tile's smooth ground instead (`lod_grid_cell`): word 2
    /// is its record's place in the tile, whose first `GRID_WORDS` words are the heights and
    /// biomes of its `GRID_SIDE` x `GRID_SIDE` corners (from one before the tile's first, `lod_grid`),
    /// word 3 its cell (i, j) and level. Its corners morph to the next level's shape with distance
    /// (`lod_corner`).
    lod_face {
        POS_MASK = 2047,
        Z_SHIFT = 11,
        DIR_SHIFT = 22,
        TINT_SHIFT = 25,
        HALF_SHIFT = 16,
        HALF_MASK = 0xFFFF,
        GRID = 6,
        GRID_SIDE = 34,
        GRID_WORDS = 34 * 34,
        CELL_MASK = 63,
        CELL_J_SHIFT = 6,
        LEVEL_SHIFT = 12,
        /// chiacraft: word 0's top bit: the face is a tree crown's (`client::lod`), for packs that
        /// shade far canopies apart from the ground.
        CROWN_SHIFT = 31,
        /// chiacraft: word 0's bits 29-30: how much the land round shuts the face's sky out, 0 to 3
        /// (`client::lod`'s `openness`), for packs that darken valleys and the gaps between crowns.
        OCC_SHIFT = 29,
        OCC_MASK = 3,
    }
    /// Tint kinds of faces and quads (0 none); mods' kinds follow the engine's.
    tint {
        MASK = 15,
    }
    /// Entity box modes (`entity.wgsl`).
    box_mode {
        SKIN = 0,
        FACES = 1,
        SOLID = 2,
        FLAMES = 3,
        TEXEL = 4,
        FILLED = 5,
        QUAD = 6,
    }
    /// `Quad::layer` of a texture animated as a strip: the frame, the next and how many there are.
    ui_strip {
        BITS = 11,
        MASK = 2047,
        COUNT_SHIFT = 22,
    }
    /// UI quad modes (`ui.wgsl`).
    ui_mode {
        SOLID = 0,
        TEXTURE = 1,
        ICON = 2,
        FONT = 3,
        GUI = 4,
    }
    /// Particle data word y: texture index and shader slot.
    particle {
        TEX_MASK = 0xFFFF,
        SLOT_SHIFT = 24,
    }
    /// What each entry of the sky list draws (`sky.wgsl`).
    sky {
        BODY = 0,
        STARS = 1,
        DOME = 2,
        GRADIENT = 3,
        GLOW = 4,
        OVERLAY = 5,
        CLOUDS = 6,
    }
}

/// Terrain on the GPU (`terrain.wgsl`, `client::terrain_gpu`): the layout of the description,
/// the noise seeds, record sizes and what comes back, for both sides.
pub mod terrain {
    use weft_noise::fixed::salt;

    shared! {
        /// Header words of the description (`terrain_gpu::describe`).
        d {
            STONE = 0, BASE = 1, HAS_SEA = 2, SEA = 3, WATER = 4, HAS_CAVES = 5, LAVA = 6,
            LAVA_FROM = 7, LAVA_TO = 8, FIELDS = 9, FEATURES = 10, RULES = 11, CAVE_BIOMES = 12,
            /// The noise seeds, in `SEEDS` order.
            SEEDS = 13,
            FIELD_OFF = 21, FEAT_OFF = 22, RULE_OFF = 23, CAVE_OFF = 24,
            /// Distant terrain's scattered things (`describe_lod`).
            SCATTER_OFF = 25,
            /// Distant terrain: from what fall (`lod::fall`) the ground is a bare face, where no
            /// scattered things stand (`Terrain::faces`; `i32::MAX`: none).
            FACE_FALL = 26,
            /// The continent noise's scale (Q4, `Terrain::continent_scale`).
            CONT_SCALE = 27,
            /// The header's length.
            LEN = 28,
        }
        /// Words per record of the description and per cube made. A rule is: biome, lowest and
        /// highest surface height, the word where its conditions start and how many there
        /// are, top block, filler block, depth, under block and layers, deep top block
        /// and the height below which it applies, ice block, a flag for bare stone and the
        /// height above which that is.
        words {
            /// An expression step: kind, the noise seed (made from its salt), then `a`, `b`, `c`.
            STEP = 5,
            /// A field: where its steps start (a word index in the description) and how many.
            FIELD = 2,
            /// A feature: start and count of its height's steps, and of its mask's (count 0: none),
            /// then its place among the features made on the CPU (`Feature::custom`), whose heights
            /// come with each job (`CUSTOM`, or a tile's `lod::SAMPLES`) instead of its steps
            /// (-1: none).
            FEATURE = 5,
            RULE = 15,
            /// A condition of a rule: the field, from, to.
            COND = 3,
            CAVE = 5,
            /// Per scatter: cell, reach, hash seed (two words), biomes, its thinning mask's steps
            /// (where they start and how many, `STEP` each) and keep. After it the mask's steps,
            /// a `SCATTER_BIOME` per biome, then the ground it stands on (`ScatterDef::ground`) as
            /// state ranges: how many, then first and count each (none: any).
            SCATTER = 8,
            /// Per scatter and biome: threshold (two words), thinned, then `lod::SHAPES` crowns
            /// its shape callback made there, two words each (`crown`).
            SCATTER_BIOME = 3 + 2 * 8,
            /// Per CPU-made feature (`Feature::custom`) and cube: heights at the 256 columns and
            /// the 25 corners caves measure from, after the job's own words.
            CUSTOM = 281,
            OUT = 2560,
        }
        /// Sizes of the shader's own arrays: the values of the fields at one column, and what an
        /// expression keeps on its stack.
        limit {
            FIELDS = 32,
            DEPTH = 16,
        }
        /// Where things are in a cube made (`words::OUT`): blocks two per word from 0, then biomes
        /// (cave biomes included) two per word, column heights, column biomes two per word.
        out {
            BIOMES = 2048,
            HEIGHTS = 2176,
            COLUMN_BIOMES = 2432,
        }
        /// Distant terrain tiles (`client::lod`): samples per side (a tile's 32 cells and one
        /// around it), all of them.
        lod {
            SIDE = 34,
            SAMPLES = 34 * 34,
            /// Words per sample coming back: the ground (height and top, biome and under) and
            /// the crown over it (`lod_out`).
            WORDS = 4,
            /// Crowns sampled per scatter and biome; each site takes one by its hash.
            SHAPES = 8,
        }
        /// A sampled crown (`terrain_gpu::describe_lod`), first word: radius, top above the site,
        /// its lowest leaves, the widest layer (fifteenths of the way from the lowest to the top),
        /// how round its top is (its width halfway from the widest layer to the top, fifteenths
        /// of the radius), the trunk's width. Second word: its leaves, its trunk's block. A first
        /// word of 0: nothing there.
        crown {
            RADIUS_MASK = 63,
            TOP_SHIFT = 6,
            TOP_MASK = 255,
            BOTTOM_SHIFT = 14,
            BOTTOM_MASK = 127,
            WIDEST_SHIFT = 21,
            ROUND_SHIFT = 25,
            FIFTEENTHS = 15,
            TRUNK_SHIFT = 29,
            TRUNK_MASK = 7,
            BLOCK_MASK = 0xFFFF,
            TRUNK_BLOCK_SHIFT = 16,
        }
        /// A sample's crown, words 2 and 3 of it: the crown's top block (signed 16 bits) and its
        /// leaves (none: no crown); how many blocks it reaches down under its top, the trunk
        /// standing in its cell (its place in the cell, two bits each, and width; 0: none) and
        /// the trunk's block.
        lod_out {
            HALF_SHIFT = 16,
            HALF_MASK = 0xFFFF,
            DEPTH_MASK = 255,
            TRUNK_DZ_SHIFT = 8,
            TRUNK_DX_SHIFT = 10,
            TRUNK_W_SHIFT = 12,
            PLACE_MASK = 3,
            TRUNK_W_MASK = 15,
        }
        /// Expression step kinds.
        expr {
            CONST = weft_abi::expr::CONST,
            X = weft_abi::expr::X,
            Z = weft_abi::expr::Z,
            FIELD = weft_abi::expr::FIELD,
            ADD = weft_abi::expr::ADD,
            SUB = weft_abi::expr::SUB,
            MUL = weft_abi::expr::MUL,
            DIV = weft_abi::expr::DIV,
            REM = weft_abi::expr::REM,
            MIN = weft_abi::expr::MIN,
            MAX = weft_abi::expr::MAX,
            ABS = weft_abi::expr::ABS,
            RAMP = weft_abi::expr::RAMP,
            NOISE = weft_abi::expr::NOISE,
            SITE = weft_abi::expr::SITE,
        }
    }

    /// The noise seeds the description carries from `d::SEEDS` on, the shader's `S_<name>`:
    /// made from the salts the CPU uses.
    /// The last three are the standard fields' own noise, which distant terrain's thinning masks
    /// read (`continent()`, `temperature()`, `humidity()`) where the terrain's fields may be pruned.
    pub const SEEDS: [(&str, u64); 8] = [
        ("C1", salt::CAVE_A),
        ("C2", salt::CAVE_B),
        ("C3", salt::CAVERN),
        ("C4", salt::CAVERN_ROOF),
        ("LAKE", salt::LAVA),
        ("CONT", salt::CONTINENT),
        ("TEMP", salt::TEMPERATURE),
        ("HUM", salt::HUMIDITY),
    ];

    /// What goes in front of `terrain.wgsl`: the constants, the seeds' indices, the bindings
    /// and the push constants.
    pub fn prelude() -> String {
        let mut s = wgsl();
        s.push_str(&format!(
            "const ONE: i32 = {};
",
            weft_noise::fixed::ONE
        ));
        for (i, (name, _)) in SEEDS.iter().enumerate() {
            s.push_str(&format!(
                "const S_{name}: u32 = {i}u;
"
            ));
        }
        s.push_str(concat!(
            "// The description (`terrain_gpu::describe`).
",
            "@group(1) @binding(0) var<storage, read> desc: array<i32>;
",
            "// Per cube: x, y, z, unused.
",
            "@group(1) @binding(1) var<storage, read> jobs: array<i32>;
",
            "// Per cube `WORDS_OUT` words (`OUT_*`); per distant terrain tile its samples.
",
            "@group(1) @binding(2) var<storage, read_write> outp: array<u32>;
",
            "struct Params {
    cubes: u32,
    custom: u32,
}
",
            "var<immediate> params: Params;
",
        ));
        s
    }

    /// The bindings `prelude` declares, in order.
    pub mod binding {
        pub const DESC: u32 = 0;
        pub const JOBS: u32 = 1;
        pub const OUT: u32 = 2;
    }
}

/// A unit cube's faces (-X +X -Y +Y -Z +Z), four corners each counter-clockwise seen from
/// outside: bottom-left, bottom-right, top-right, top-left in texture space. The shaders get it
/// as `CORNERS`.
pub const CORNERS: [[[i32; 3]; 4]; 6] = weft_abi::FACE_CORNERS;

/// A block face record (`face`): (x, y, z) in the cube, `ao` 2 bits per corner, `light` per
/// corner (`corner_light`).
#[inline]
pub fn face_record(pos: [i32; 3], dir: u32, ao: u32, tint: u32, turn: u32, tex: u32, biome: u32, light: [u32; 4]) -> [u32; 4] {
    let w0 = pos[0] as u32
        | (pos[1] as u32) << face::Y_SHIFT
        | (pos[2] as u32) << face::Z_SHIFT
        | dir << face::DIR_SHIFT
        | ao << face::AO_SHIFT
        | tint << face::TINT_SHIFT
        | (turn & face::TURN_MASK) << face::TURN_SHIFT;
    [w0, tex | biome << face::BIOME_SHIFT, light[0] | light[1] << face::CORNER_SHIFT, light[2] | light[3] << face::CORNER_SHIFT]
}

/// The direction of a face record.
#[inline]
pub fn face_dir(f: &[u32; 4]) -> usize {
    ((f[0] >> face::DIR_SHIFT) & face::DIR_MASK) as usize
}

/// Corner light of a face or quad from a cell's light (8 bits per channel), 0..15 each.
#[inline]
pub fn corner_light(channels: [u32; 4]) -> u32 {
    channels
        .iter()
        .enumerate()
        .fold(0, |w, (i, &c)| w | c.min(face::LIGHT_MASK) << (i * 4))
}

/// A quad corner (`quad`), in blocks from the cube's corner.
#[inline]
pub fn quad_pos(p: [f32; 3]) -> u32 {
    let e = |v: f32| ((v * 16.0).round() as i32 + quad::POS_OFFSET as i32).clamp(0, quad::POS_MASK as i32) as u32;
    e(p[0]) | e(p[1]) << quad::POS_BITS | e(p[2]) << (2 * quad::POS_BITS)
}

/// Where the middle of a quad is, in 1/32 block from the cube's corner, whatever else is on its
/// corners' words.
pub fn quad_middle(q: &[u32; 8]) -> [i32; 3] {
    let at = |w: u32, axis: u32| ((w >> (axis * quad::POS_BITS)) & quad::POS_MASK) as i32 - quad::POS_OFFSET as i32;
    std::array::from_fn(|a| at(q[0], a as u32) + at(q[2], a as u32))
}

/// A quad's uv word, and what goes on top of each of its corners' words: `uv` (u0, v0, u1, v1, 16
/// across a texture), tint kind and flags.
#[inline]
pub fn ui_strip_record(frame: u32, next: u32, count: u32) -> u32 {
    frame.min(ui_strip::MASK) | next.min(ui_strip::MASK) << ui_strip::BITS | count << ui_strip::COUNT_SHIFT
}

pub fn quad_uv(uv: [f32; 4], tint: u32) -> (u32, [u32; 4]) {
    let e = uv.map(|v| ((v * 8.0).round() as i32).clamp(0, (quad::UV_MASK as i32 + 1) * 8 - 1) as u32);
    let [whole, fine] = [e.map(|v| v >> quad::UV_FINE_BITS), e.map(|v| v & ((1 << quad::UV_FINE_BITS) - 1))];
    let fine = fine[0] | fine[1] << quad::UV_FINE_BITS | fine[2] << (2 * quad::UV_FINE_BITS) | fine[3] << (3 * quad::UV_FINE_BITS);
    let word = whole[0]
        | whole[1] << quad::UV_BITS
        | whole[2] << (2 * quad::UV_BITS)
        | whole[3] << (3 * quad::UV_BITS)
        | (fine >> 8) << quad::UV_FINE_HIGH_SHIFT
        | tint << quad::TINT_SHIFT;
    (word, std::array::from_fn(|i| ((fine >> (2 * i)) & 3) << quad::UV_FINE_LOW_SHIFT))
}

/// A distant terrain record (`lod_face`): the `dir` face of a box at `pos` in the tile, `size`
/// blocks.
#[inline]
pub fn lod_record(dir: u32, pos: [i32; 3], size: [i32; 3], tex: u32, tint: u32, biome: u32) -> [u32; 4] {
    let h = lod_face::HALF_SHIFT;
    [
        pos[0] as u32 | (pos[2] as u32) << lod_face::Z_SHIFT | dir << lod_face::DIR_SHIFT | tint << lod_face::TINT_SHIFT,
        tex | biome << face::BIOME_SHIFT,
        (pos[1] as u32 & lod_face::HALF_MASK) | (size[1] as u32) << h,
        size[0] as u32 | (size[2] as u32) << h,
    ]
}

/// A cell of a far tile's smooth ground (`lod_face::GRID`): cell (i, j) of a level `level` tile,
/// the `k`th record of the tile, its top block's texture, tint kind and biome.
pub fn lod_grid_cell(i: u32, j: u32, level: u32, k: u32, tex: u32, tint: u32, biome: u32) -> [u32; 4] {
    let s = 1 << level;
    [
        i * s | (j * s) << lod_face::Z_SHIFT | lod_face::GRID << lod_face::DIR_SHIFT | tint << lod_face::TINT_SHIFT,
        tex | biome << face::BIOME_SHIFT,
        k,
        i | j << lod_face::CELL_J_SHIFT | level << lod_face::LEVEL_SHIFT,
    ]
}

/// A far tile's corner for its smooth ground: the top block's y and the biome.
pub fn lod_grid(y: i32, biome: u32) -> u32 {
    (y as u32 & lod_face::HALF_MASK) | biome << lod_face::HALF_SHIFT
}

/// Distant terrain's vertices, for `vs_lod` (engine's and packs'): needs `records`.
const LOD_CORNER: &str = "
// A distant terrain vertex: where it is in its tile, its normal, the face's direction (a smooth
// cell's is up) and the biome its tint is from.
struct LodCorner {
    local: vec3<f32>,
    normal: vec3<f32>,
    dir: u32,
    biome: u32,
}

fn lod_grid_word(base: u32, i: i32, j: i32) -> u32 {
    let n = i32(LOD_FACE_GRID_SIDE) - 1;
    return records[base + u32(clamp(j + 1, 0, n)) * LOD_FACE_GRID_SIDE + u32(clamp(i + 1, 0, n))];
}

fn lod_grid_y(base: u32, i: i32, j: i32) -> f32 {
    return f32(bitcast<i32>(lod_grid_word(base, i, j) << LOD_FACE_HALF_SHIFT) >> LOD_FACE_HALF_SHIFT);
}

// Corner (`i`, `j`) of a smooth tile, `m` of the way to the next level's shape: on its grid of
// even corners, odd ones lie on the line or square between them (CDLOD's morph). That level's
// tiles have these very corners, so tiles meet without a seam and a level swap doesn't jump.
fn lod_grid_morphed(base: u32, i: i32, j: i32, m: f32) -> f32 {
    let h = lod_grid_y(base, i, j);
    let oi = (i & 1) != 0;
    let oj = (j & 1) != 0;
    var t = h;
    if oi && oj {
        t = 0.25 * (lod_grid_y(base, i - 1, j - 1) + lod_grid_y(base, i + 1, j - 1) + lod_grid_y(base, i - 1, j + 1) + lod_grid_y(base, i + 1, j + 1));
    } else if oi {
        t = 0.5 * (lod_grid_y(base, i - 1, j) + lod_grid_y(base, i + 1, j));
    } else if oj {
        t = 0.5 * (lod_grid_y(base, i, j - 1) + lod_grid_y(base, i, j + 1));
    }
    return mix(h, t, m);
}

fn lod_box_normal(dir: u32) -> vec3<f32> {
    switch dir {
        case 0u: { return vec3(-1.0, 0.0, 0.0); }
        case 1u: { return vec3(1.0, 0.0, 0.0); }
        case 2u: { return vec3(0.0, -1.0, 0.0); }
        case 3u: { return vec3(0.0, 1.0, 0.0); }
        case 4u: { return vec3(0.0, 0.0, -1.0); }
        default: { return vec3(0.0, 0.0, 1.0); }
    }
}

// Corner `c` of record `f`; `at` is its tile's origin from the camera. A level's tiles are shown
// from about 4 to 8 of their widths away; a smooth one has the next level's shape from 8 on, its
// own from 6 in.
fn lod_corner(f: u32, c: u32, at: vec3<f32>) -> LodCorner {
    let w0 = records[f * 4u];
    let w1 = records[f * 4u + 1u];
    let w2 = records[f * 4u + 2u];
    let w3 = records[f * 4u + 3u];
    let dir = lod_dir(w0);
    if dir != LOD_FACE_GRID {
        let lo = lod_pos(w0, w2) - vec3(0.0, 0.125, 0.0);
        return LodCorner(lo + CORNERS[dir * 4u + c] * lod_size(w2, w3), lod_box_normal(dir), dir, w1 >> FACE_BIOME_SHIFT);
    }
    let base = (f - w2) * 4u;
    let s = f32(1u << ((w3 >> LOD_FACE_LEVEL_SHIFT) & 7u));
    let k = CORNERS[3u * 4u + c];
    let i = i32(w3 & LOD_FACE_CELL_MASK) + i32(k.x);
    let j = i32((w3 >> LOD_FACE_CELL_J_SHIFT) & LOD_FACE_CELL_MASK) + i32(k.z);
    let xz = vec2(f32(i), f32(j)) * s;
    let width = 32.0 * s;
    let m = clamp((length(at.xz + xz) - 6.0 * width) / (2.0 * width), 0.0, 1.0);
    let y = lod_grid_morphed(base, i, j, m);
    let dx = lod_grid_y(base, i + 1, j) - lod_grid_y(base, i - 1, j);
    let dz = lod_grid_y(base, i, j + 1) - lod_grid_y(base, i, j - 1);
    let normal = normalize(vec3(-dx, 2.0 * s, -dz));
    let biome = lod_grid_word(base, i, j) >> LOD_FACE_HALF_SHIFT;
    return LodCorner(vec3(xz.x, y + 1.0 - 0.125, xz.y), normal, 3u, biome);
}
";

/// The WGSL side of the records above: `CORNERS` and their decoders.
fn records() -> String {
    let corners: Vec<String> = CORNERS
        .iter()
        .flatten()
        .map(|c| format!("vec3({:.1}, {:.1}, {:.1})", c[0] as f32, c[1] as f32, c[2] as f32))
        .collect();
    let mut s = format!("var<private> CORNERS: array<vec3<f32>, 24> = array<vec3<f32>, 24>({});\n", corners.join(", "));
    let dirs: Vec<String> = weft_abi::FACE_DIRS
        .iter()
        .map(|d| format!("vec3({:.1}, {:.1}, {:.1})", d[0] as f32, d[1] as f32, d[2] as f32))
        .collect();
    s.push_str(&format!("var<private> FACE_DIRS: array<vec3<f32>, 6> = array<vec3<f32>, 6>({});\n", dirs.join(", ")));
    s.push_str(concat!(
        "fn face_pos(w0: u32) -> vec3<f32> {\n",
        "    return vec3<f32>(f32(w0 & FACE_POS_MASK), f32((w0 >> FACE_Y_SHIFT) & FACE_POS_MASK), f32((w0 >> FACE_Z_SHIFT) & FACE_POS_MASK));\n",
        "}\n",
        "fn face_dir(w0: u32) -> u32 {\n    return (w0 >> FACE_DIR_SHIFT) & FACE_DIR_MASK;\n}\n",
        "// AO per corner, 0 darkest to 3 none.\n",
        "fn face_ao(w0: u32) -> vec4<u32> {\n",
        "    let ao = w0 >> FACE_AO_SHIFT;\n",
        "    return vec4<u32>(ao & FACE_AO_MASK, (ao >> 2u) & FACE_AO_MASK, (ao >> 4u) & FACE_AO_MASK, (ao >> 6u) & FACE_AO_MASK);\n",
        "}\n",
        "// Corner `c`'s light from a record's two light words, the channels 0..1.\n",
        "fn corner_light(lo: u32, hi: u32, c: u32) -> vec4<f32> {\n",
        "    let l = (select(lo, hi, c >= 2u) >> ((c & 1u) * FACE_CORNER_SHIFT)) & 65535u;\n",
        "    return vec4<f32>(f32(l & FACE_LIGHT_MASK), f32((l >> 4u) & FACE_LIGHT_MASK), f32((l >> 8u) & FACE_LIGHT_MASK), f32((l >> 12u) & FACE_LIGHT_MASK)) / 15.0;\n",
        "}\n",
        "fn quad_pos(w: u32) -> vec3<f32> {\n",
        "    let p = vec3<u32>(w, w >> QUAD_POS_BITS, w >> (2u * QUAD_POS_BITS)) & vec3(QUAD_POS_MASK);\n",
        "    return (vec3<f32>(p) - f32(QUAD_POS_OFFSET)) / 16.0;\n",
        "}\n",
        "// u0, v0, u1, v1, in whole 1/16ths of a texture.\n",
        "fn quad_uv(w: u32) -> vec4<f32> {\n",
        "    let b = QUAD_UV_BITS;\n",
        "    return vec4<f32>(vec4<u32>(w, w >> b, w >> (2u * b), w >> (3u * b)) & vec4(QUAD_UV_MASK)) / 16.0;\n",
        "}\n",
        "// The same to the 1/8th, from the uv word and the four corners' words.\n",
        "fn quad_uv_fine(w: u32, c0: u32, c1: u32, c2: u32, c3: u32) -> vec4<f32> {\n",
        "    let s = QUAD_UV_FINE_LOW_SHIFT;\n",
        "    let f = (c0 >> s) | (c1 >> s) << 2u | (c2 >> s) << 4u | (c3 >> s) << 6u | ((w >> QUAD_UV_FINE_HIGH_SHIFT) & 15u) << 8u;\n",
        "    let b = QUAD_UV_FINE_BITS;\n",
        "    let fine = vec4<u32>(f, f >> b, f >> (2u * b), f >> (3u * b)) & vec4(7u);\n",
        "    return quad_uv(w) + vec4<f32>(fine) / (8.0 * 16.0);\n",
        "}\n",
        "fn lod_pos(w0: u32, w2: u32) -> vec3<f32> {\n",
        "    return vec3<f32>(f32(w0 & LOD_FACE_POS_MASK), f32(bitcast<i32>(w2 << LOD_FACE_HALF_SHIFT) >> LOD_FACE_HALF_SHIFT), f32((w0 >> LOD_FACE_Z_SHIFT) & LOD_FACE_POS_MASK));\n",
        "}\n",
        "fn lod_size(w2: u32, w3: u32) -> vec3<f32> {\n",
        "    return vec3<f32>(f32(w3 & LOD_FACE_HALF_MASK), f32(w2 >> LOD_FACE_HALF_SHIFT), f32(w3 >> LOD_FACE_HALF_SHIFT));\n",
        "}\n",
        "fn lod_dir(w0: u32) -> u32 {\n    return (w0 >> LOD_FACE_DIR_SHIFT) & FACE_DIR_MASK;\n}\n",
        "fn lod_tint(w0: u32) -> u32 {\n    return (w0 >> LOD_FACE_TINT_SHIFT) & TINT_MASK;\n}\n",
        "fn lod_crown(w0: u32) -> bool {\n    return (w0 >> LOD_FACE_CROWN_SHIFT) != 0u;\n}\n",
        "fn lod_occ(w0: u32) -> u32 {\n    return (w0 >> LOD_FACE_OCC_SHIFT) & LOD_FACE_OCC_MASK;\n}\n",
    ));
    s
}

/// A host-visible buffer per frame in flight (`slots`), holding at least `needed` elements of
/// `stride` bytes; a slot too small is replaced by one grown to the next power of two.
pub fn frame_buffer(
    gpu: &mut dyn weft_render::Backend,
    slots: &mut Vec<(weft_render::BufferId, u32)>,
    slot: usize,
    needed: u32,
    stride: u64,
    usage: u32,
    label: &str,
) -> weft_render::BufferId {
    use weft_render::usage::HOST;
    while slots.len() < gpu.frames_in_flight() {
        slots.push((gpu.create_buffer(stride * 1024, usage | HOST, label), 1024));
    }
    if slots[slot].1 < needed {
        gpu.destroy_buffer(slots[slot].0);
        let cap = needed.next_power_of_two();
        slots[slot] = (gpu.create_buffer(stride * cap as u64, usage | HOST, label), cap);
    }
    slots[slot].0
}

/// WGSL spelling of a field type.
pub trait Wgsl {
    const TY: &'static str;
    /// Alignment and size in a WGSL storage buffer.
    const LAYOUT: (usize, usize);
}

macro_rules! wgsl_types {
    ($($t:ty => $s:literal, $a:literal, $n:literal),* $(,)?) => { $(impl Wgsl for $t { const TY: &'static str = $s; const LAYOUT: (usize, usize) = ($a, $n); })* };
}
wgsl_types! {
    f32 => "f32", 4, 4, u32 => "u32", 4, 4, i32 => "i32", 4, 4,
    [f32; 2] => "vec2<f32>", 8, 8, [f32; 3] => "vec3<f32>", 16, 12, [f32; 4] => "vec4<f32>", 16, 16,
    [u32; 4] => "vec4<u32>", 16, 16, [i32; 4] => "vec4<i32>", 16, 16,
    [[f32; 4]; 3] => "array<vec4<f32>, 3>", 16, 48, [[f32; 4]; 4] => "mat4x4<f32>", 16, 64, [[f32; 4]; 6] => "array<vec4<f32>, 6>", 16, 96, [[f32; 4]; 8] => "array<vec4<f32>, 8>", 16, 128,
}

/// Declares a GPU buffer struct: the Rust type and, from the same fields, its WGSL text
/// (`wgsl()`), under the WGSL name given after `as`.
macro_rules! gpu_struct {
    ($(#[$m:meta])* pub struct $name:ident as $wgsl:literal { $($(#[$fm:meta])* pub $f:ident: $t:ty),* $(,)? }) => {
        $(#[$m])*
        #[repr(C)]
        #[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct $name { $($(#[$fm])* pub $f: $t),* }

        // The shader reads every field where Rust puts it, and arrays of it by the same stride.
        const _: () = {
            let (mut off, mut align) = (0usize, 4usize);
            $(
                let (a, n) = <$t as Wgsl>::LAYOUT;
                off = off.div_ceil(a) * a;
                assert!(off == core::mem::offset_of!($name, $f), concat!(stringify!($name), ".", stringify!($f), " sits elsewhere in WGSL"));
                off += n;
                if a > align {
                    align = a;
                }
            )*
            assert!(off.div_ceil(align) * align == core::mem::size_of::<$name>(), concat!(stringify!($name), " has another size in WGSL: pad it"));
        };

        impl $name {
            pub fn wgsl() -> String {
                let mut s = format!("struct {} {{\n", $wgsl);
                $(s.push_str(&format!("    {}: {},\n", stringify!($f), <$t as Wgsl>::TY));)*
                s.push_str("}\n");
                s
            }
        }
    };
}

gpu_struct! {
    /// What every world pass sees of the frame: camera, fog, light, time. One buffer a frame
    /// for the world, entities, models and particles; the hand and the inventory figure have
    /// their own camera.
    pub struct Scene as "Scene" {
        pub view_proj: [[f32; 4]; 4],
        /// Its inverse: from the screen back to camera-relative positions (screen effects).
        pub inv_view_proj: [[f32; 4]; 4],
        /// Camera right and up in world space (particle billboards).
        pub right: [f32; 4],
        pub up: [f32; 4],
        /// Linear rgb; w how hard it rains, 0..1 (shader packs).
        pub fog_color: [f32; 4],
        /// x fog start, y fog end, z daylight 0..1, w seconds (shader effects).
        pub params: [f32; 4],
        /// x the dimension's ambient light 0..1, y the Brightness setting 0 (moody) .. 1
        /// (bright), z game ticks with the fraction of the current one (animated textures;
        /// wraps every 2^20).
        pub light: [f32; 4],
        /// xyz toward the sun (unit), w how far it is above the horizon (0 under it or in a
        /// dimension without sky light). For shader packs' shadows; follows the day, whatever
        /// bodies mods put in the sky.
        pub sun: [f32; 4],
        /// x and y the wind where the player is along the world's x and z (m/s, the way it
        /// blows), z how foggy it is there (0..1), w how hard it thunders (0..1); from a mod's
        /// weather (`set_air`), still and clear without one. For shader packs.
        pub weather: [f32; 4],
        /// The camera in the world: x and z wrapped every 8192 blocks (enough for noise to tile
        /// unseen), y as it is; w the moon's light by its phase (1 full, 0 new: vanilla's eight,
        /// the phase the sky's moon shows). For shader packs.
        pub eye: [f32; 4],
        /// Where the player is: x snow lying on the ground (0..1), y how much of what falls is
        /// snow, z how much dust blows (0..1), w how strong an aurora is on clear nights (0..1);
        /// from a mod's weather (`set_air`). For shader packs.
        pub ground: [f32; 4],
        /// From camera-relative positions to the shadow map's clip space (`sun_shadows` packs):
        /// x and y -1..1 across the map (texel row 0 at y -1), z reverse depth along the light.
        pub shadow_view_proj: [[f32; 4]; 4],
        /// x the shadow map's bindless index, y its size in texels, z 1 while it holds this
        /// frame's shadows (0: none, fall back), w the blocks a texel covers.
        pub shadow: [f32; 4],
        /// The world as it was before what is seen through it was drawn (`water_behind` packs):
        /// x its color's bindless index, y its depth's, z 1 while they hold this frame's (0: none),
        /// w unused. Only what is drawn after (water, glass) may read them.
        pub behind: [f32; 4],
        /// The far shadow cascade, as `shadow_view_proj` and `shadow` for the near one: coarser,
        /// 400 blocks to each side, where the near one ends.
        pub shadow_far_view_proj: [[f32; 4]; 4],
        pub shadow_far: [f32; 4],
        /// The sky's cloud layer, for packs' cloud shadows: x its picture's bindless index + 1 (0:
        /// no clouds), y the blocks a picture cell covers, z its underside's height over the camera
        /// (blocks), w how much of the picture's gaps the weather fills (0..1, a hash per cell
        /// decides which).
        pub clouds: [f32; 4],
        /// Where the camera is under it: x and z (in `xy`) its place in its cell, 0..1, and the
        /// cell in the picture (`zw`), as the sky's cloud pass reads them.
        pub cloud_cells: [f32; 4],
        /// xyz toward the moon where the sky draws it (it lags the day by its phase: up all night
        /// when full, only by day when new), w the light it gives: its phase times how high it
        /// stands, 0 below the horizon. For packs' night light and moon shadows.
        pub moon: [f32; 4],
        /// The values of the settings the pack names in its `pack.cfg` (`options = a, b, ...`), 32
        /// at most (16 before chiacraft's fantasy settings), four to a row in that order
        /// (`packs::pack_options`); 0 for one not found.
        pub options: [[f32; 4]; 8],
        /// The light the player holds (a torch, a lantern), for packs' dynamic light: xyz where it
        /// is (camera relative, by the hand), w unused.
        pub hand: [f32; 4],
        /// Its light per channel (r, g, b) as block light levels 0..1 (a torch 14/15); zeros
        /// when what is held gives none. It falls off like block light, a level a block.
        pub hand_light: [f32; 4],
        /// Where the player is: x how closed a forest's canopy is (0 open sky to 1 a rainforest's;
        /// a mod's air, `air_flags::CANOPY_SHIFT`), for packs' light under the leaves; y how wet
        /// the ground is, 0..1: it follows what falls up and dries over minutes after (a mod's air
        /// says how fast, `air_flags::DRYING_SHIFT`); z how far puddles have grown, 0..1: over
        /// about ten minutes of rain, drying three times slower than the ground after; w where rain
        /// turns to snow up a mountain, blocks above the sea (a mod's air,
        /// `air_flags::SNOWLINE_SHIFT`; 400 for none).
        pub canopy: [f32; 4],
        /// Soft clouds (`core:soft_clouds`, with a pack whose sky has `fs_cloud_trace` and
        /// `fs_cloud_resolve`; chiacraft): x the bindless index + 1 of this frame's clouds, traced
        /// at a quarter of the pixels each way and blended over frames, which `fs_clouds` draws
        /// from (0: off, the blocky clouds); y the index + 1 of last frame's, the history
        /// `fs_cloud_resolve` blends into; z the bindless index of the noise volume
        /// (`soft_clouds::noise_atlas`: 64 slices of 64², eight to a row, each in a 66² cell with
        /// a wrapped border, so a linear sample within a cell tiles); w how many frames the history
        /// holds (0: none yet, take this frame's alone).
        pub soft_clouds: [f32; 4],
        /// Last frame's `view_proj` (camera relative then), for reprojecting the clouds.
        pub prev_view_proj: [[f32; 4]; 4],
        /// xyz: where last frame's camera stands from this one's, less how far the wind carried
        /// the clouds since (blocks): a cloud at camera-relative `p` now stood at `p + xyz` from
        /// last frame's camera. w: the frame's number (for the trace's jitter).
        pub cloud_moved: [f32; 4],
        /// What happens in the sky (chiacraft, a mod's `set_sky_event`): x the kind
        /// (`weft_abi::sky_event`: 0 none, 1 blood moon, 2 meteors, 3 fireball, 4 aurora storm),
        /// y how strongly (0..1), z seconds since it began, w unused.
        pub sky_event: [f32; 4],
        /// What falls from the sky (a mod's `sky_landing`): xyz where it lands (camera relative),
        /// w seconds until it does (negative after; -1e6 when nothing falls).
        pub sky_landing: [f32; 4],
        /// What the pack's look works out once a frame (`client::frame_look`, a pack whose sky
        /// has `fs_frame_look`; chiacraft): x the bindless index + 1 of its row of texels (0:
        /// none this frame, each shader works them out itself), yzw unused.
        pub frame_look: [f32; 4],
        /// The last lightning flash where the player is (a weather with `lightning`, chiacraft):
        /// x seconds since it began (1e4 when none yet; real time, not the shaders' clock, which
        /// tests pin), y a random number 0..1 of its own (for where it strikes), z how hard it
        /// thunders now (as `weather.w`), w where the camera faced as it began (radians,
        /// clockwise from -z, as `atan2(forward.x, -forward.z)`), so a pack can aim strikes into
        /// view and keep them put while the player turns. The engine's own flash only brightens the
        /// daylight for 0.15 s; packs draw the bolt and the flicker from this.
        pub lightning: [f32; 4],
        /// How this frame is drawn: x 1 when into the HDR world (`post::hdr`: linear light past 1,
        /// the picture finished in the effects' last pass, so a pack's surfaces leave its tone
        /// curve to that pass); 0 into the screen or a preview. y 1 when the last frame drew the
        /// HDR world's bloom pyramid (`post::bloom`), which the last pass adds, so a pack's own
        /// bloom effect leaves its glow out. z the pixels a mist volume cell covers each way
        /// (`client::froxels`: 8, or 4 at ultra), for a pack that spends more on finer cells. w,
        /// chiacraft: the frame's number, 0 to 63 and round again, for dithers that change each
        /// frame (a temporal blend's history averages them out; without one they shimmer finely).
        pub frame: [f32; 4],
        /// The soft clouds' shadow on the land (`client::cloud_shadow`, with the HDR world; a pack
        /// whose sky has `fs_cloud_shadow`): x the map's bindless index + 1 (0: none this frame), y
        /// its texels each way, zw unused.
        pub cloud_shadow: [f32; 4],
        /// The mist's volume (`client::froxels`, `core:froxels`, with a pack whose sky has
        /// `fs_froxel_light` and `fs_froxel_sum`; chiacraft): x the bindless index + 1 of the sums
        /// from the eye (0: none this frame), y the index of each cell's own, zw the cells across
        /// the view; 32 slices deep, eight to a row of the image.
        pub froxels: [f32; 4],
        /// How a world drawn smaller than the screen is scaled up: x `core:upscaler` (0 bilinear, 1
        /// FSR 1's EASU, 2 Snapdragon GSR 1; `world_upscaled` in common.wgsl), yzw unused.
        pub upscale: [f32; 4],
        /// Ray-traced sun and moon shadows (`core:rt_shadows`, a GPU with ray queries): x 1 when
        /// the world's shader traces them this frame (its prelude then has `rt_sun`; the near shadow
        /// map then holds the models alone, the far one the terrain as before), y a frame count
        /// (to 2^24, for noise that moves every frame), z 1 for soft edges from more rays a pixel
        /// (`core:rt_shadows` 2: `rt_sun_soft`'s `fine`), w unused.
        pub rt: [f32; 4],
        /// Temporal anti-aliasing (`post::Temporal`, `core:aa`): xy this frame's jitter in NDC
        /// (`view_proj` has it, `taa_prev` not; a clip position's `xy / w - taa.xy` is where it
        /// would be without), z 1 when `taa_prev` holds last frame's camera (0: start anew), w
        /// the shaders' clock (`params.w`) last frame, for what moves by time (the wind).
        pub taa: [f32; 4],
        /// Last frame's `view_proj` without its jitter, taking this frame's camera-relative
        /// positions: where a still point stood on last frame's screen. A shader writing motion
        /// (`@location(1)`, `weft_render::MOTION_FORMAT`) puts a point's position last frame
        /// through it.
        pub taa_prev: [[f32; 4]; 4],
        /// What temporal anti-aliasing writes this frame, for the screen effects after it: x the
        /// bindless index + 1 of its count image (r the frames gathered in each pixel, g how far
        /// the pixel moved this frame in pixels: a gathered picture is softer where it moves, and
        /// a pack may sharpen it more there); 0 without. y 1 when the world's pass has the motion
        /// image (`core:aa` 1): shaders skip their motion's work without it. zw unused.
        pub taa_frame: [f32; 4],
        /// Under the backend's upscaler (DLSS, `post::set_upscaling`): x the texture mip bias, in
        /// levels (NVIDIA's log2(render / output) - 1; 0 without), which a pack's world shader adds
        /// to its block textures' level; yzw unused.
        pub dlss: [f32; 4],
    }
}

gpu_struct! {
    /// One thing drawn in the sky. The first entry is always the gradient, and its `axes` turn a
    /// screen position into a direction for everything that fills the screen.
    pub struct GpuSky as "Sky" {
        /// Columns: its own frame in the world, x toward it (`sky.wgsl`).
        pub axes: [[f32; 4]; 3],
        /// Picture rectangle: u0, v0, u1, v1. Dome: x 1 for a box layout. Glow: x how far along
        /// the sunset it is, 0..1. Stars: x half their size, y the seed.
        pub uv: [f32; 4],
        /// Linear rgb and opacity.
        pub color: [f32; 4],
        /// x bindless picture (`u32::MAX` none), y `sky` kind, z shader slot, w 1 if additive.
        pub ids: [u32; 4],
        /// x half size across (a body's tangent). Overlay: x tiles across, y 1 for the screen's
        /// lower part only, z where its frame starts in the picture.
        pub extra: [f32; 4],
    }
}

gpu_struct! {
    /// What only the world shader needs.
    pub struct Globals as "Globals" {
        /// Camera position inside its cube (0..16) in xyz.
        pub cam_frac: [f32; 4],
        /// z biome count, w blended color tiles.
        pub ids: [u32; 4],
        /// Origins are relative to this cube.
        pub cam_cube: [i32; 4],
        /// x start of the overlay table in `frames` (per texture: overlay texture + 1, or 0),
        /// y how far across from the camera the cubes end, fading into distant terrain before it
        /// (`WorldRenderer::lod_fade`, 0: no fade), z tint kinds, w half the width of `lod_mask` in the
        /// low 24 bits and `core:far_look` in the top 8 (chiacraft).
        pub extra: [u32; 4],
    }
}

gpu_struct! {
    /// A box of an entity or a held item (`entity.wgsl`): its unit cube's placement and how it
    /// is textured (`box_mode`).
    pub struct GpuBox as "Box" {
        /// Unit cube -> camera-relative world, rows of a 3x4 matrix.
        pub rows: [[f32; 4]; 3],
        /// Box size in texels (w, h, d) for the classic unwrap; `QUAD`: x its shade.
        pub dims: [f32; 4],
        /// Texel origin and texture size; `QUAD`: the part of its picture, 0..1.
        pub uvo: [f32; 4],
        /// Tint (rgb) and hurt flash (a).
        pub color: [f32; 4],
        /// Skin texture, `box_mode`, face textures packed 16 bit.
        pub ids: [u32; 4],
        /// x last face textures, y light as the world stores it (all ones: full light); block
        /// faces: z the faces tinted by w (0xRRGGBB).
        pub ids2: [u32; 4],
    }
}

gpu_struct! {
    /// One model drawn (`model.wgsl`).
    pub struct Instance as "Instance" {
        /// First bone matrix, first box and box count, first mesh corner of its kind (filled in
        /// when queued).
        pub bones: u32,
        pub cuboids: u32,
        pub cuboid_count: u32,
        pub vertices: u32,
        /// Tint (rgb), and the strength of `effect`.
        pub color: [f32; 3],
        pub strength: f32,
        pub tex: u32,
        /// Extra model pixels on every side (worn layers).
        pub inflate: f32,
        pub tex_size: [f32; 2],
        /// Bit per part role this layer draws, and bit per bone (the first 32) it draws too.
        pub roles: u32,
        pub bone_mask: u32,
        /// Entity shader slot (`shaders/entity/<name>.wgsl`, 0 none) and glow mask texture (all
        /// ones: none).
        pub shader: u32,
        pub glow: u32,
        /// Light as the world stores it, all ones: full light.
        pub light: u32,
        /// Entity effect slot (`shaders/entity_effect/<name>.wgsl`, 0 none).
        pub effect: u32,
        // Whole 16 bytes: WGSL rounds the array stride up to them.
        pub pad0: u32,
        pub pad1: u32,
    }
}

gpu_struct! {
    pub struct GpuCuboid as "Cuboid" {
        /// Corner in model pixels and the bone moving it.
        pub origin: [f32; 3],
        pub bone: u32,
        pub size: [f32; 3],
        pub inflate: f32,
        pub faces: [[f32; 4]; 6],
        pub role: u32,
        // Three words, not a vec3: WGSL would align that to 16 bytes.
        pub pad0: u32,
        pub pad1: u32,
        pub pad2: u32,
    }
}

gpu_struct! {
    pub struct GpuCorner as "Corner" {
        pub pos: [f32; 3],
        pub bone: u32,
        pub uv: [f32; 2],
        pub role: u32,
        pub pad: u32,
    }
}

gpu_struct! {
    /// A bone's transform, three rows.
    pub struct GpuBone as "Bone" {
        pub rows: [[f32; 4]; 3],
    }
}

gpu_struct! {
    /// A particle as `particle_sim.wgsl` keeps it.
    pub struct GpuSimParticle as "SimParticle" {
        /// Position from the simulation's origin, and edge length in blocks.
        pub pos_size: [f32; 4],
        /// Velocity in blocks a second, and seconds it has lived.
        pub vel_age: [f32; 4],
        /// x how long it lives.
        pub life: [f32; 4],
        /// x kind (16 bits) and flags (`particle_sim.wgsl`), y tint 0xRRGGBB, z bindless texture
        /// of block dust, w its uv rectangle as `GpuParticle`.
        pub data: [u32; 4],
    }
}

gpu_struct! {
    /// What `particle_sim.wgsl` needs of a kind, for this frame's step.
    pub struct GpuParticleKind as "ParticleKind" {
        /// x gravity, y speed kept this step, z speed kept on the ground this step, w the seconds
        /// of its motion it is drawn stretched over (0: a square).
        pub motion: [f32; 4],
        /// x color, y color at the end, z `abi::particle_flags`, w shader slot.
        pub data: [u32; 4],
        /// x first in `frames`, y how many, none for block dust; z first in `rules`, w how many.
        pub frames: [u32; 4],
        /// What one turning into it starts with: life and size, each from and to.
        pub start: [f32; 4],
        /// How the wind carries it (`registry::ParticleDef`): x the rate it takes the air's speed
        /// (0: not carried), y its sink through still air, z its flutter, w the wind that lifts it
        /// once landed.
        pub air: [f32; 4],
    }
}

gpu_struct! {
    /// A kind's `registry::ParticleRule`.
    pub struct GpuParticleRule as "ParticleRule" {
        /// x event | place << 4 | tag << 8 | action << 16, y the share a bounce keeps (f32
        /// bits), z the kind it turns into.
        pub data: [u32; 4],
    }
}

/// `occlusion.wgsl`'s push constants.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OcclusionArgs {
    /// Building a level: the size of the one it is made from, where that starts and where it goes.
    pub size: [u32; 2],
    pub src: u32,
    pub dst: u32,
    /// Testing: how many records there are.
    pub count: u32,
    pub pad: u32,
}

/// What `occlusion.wgsl` goes by this frame.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OcclusionView {
    /// The last frame's camera: what it saw is what is known.
    pub prev: [[f32; 4]; 4],
    /// This frame's sides of the view.
    pub planes: [[f32; 4]; 4],
    /// From this frame's camera to that one's (xyz), and how much bigger cubes are for it (w).
    pub moved: [f32; 4],
    pub cam_frac: [f32; 4],
    pub cam_cube: [i32; 4],
    /// The depth's size.
    pub size: [u32; 2],
    /// Records made by the CPU that aren't cubes.
    pub skip: [u32; 2],
    pub levels: u32,
    /// Whether there is a last frame to go by.
    pub hiz: u32,
    pub pad: [u32; 2],
}

/// A place in `occlusion.wgsl`'s table: a cube's draws of one layer of one kind on one page.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CubeEntry {
    pub cube: [i32; 3],
    /// `page * 4 + kind * 2 + layer`, `u32::MAX` for a free place.
    pub bucket: u32,
    pub start: u32,
    /// Faces by direction rank, 16 bits each; quads: how many, in the first.
    pub counts: [u32; 3],
}

/// `particle_sim.wgsl`'s push constants.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleSimArgs {
    pub first: u32,
    pub count: u32,
    pub dt: f32,
    pub pad: f32,
    /// Added to positions when the origin moved.
    pub shift: [f32; 4],
    /// From the origin to where drawing wants it: minus the camera.
    pub rel: [f32; 4],
    pub origin: [i32; 4],
    /// The cube in the middle of `cells`.
    pub center: [i32; 4],
    /// The wind where the player is (m/s along x and z, the way it blows; `Scene::weather`), z
    /// how hard it rains (0..1), w the shaders' seconds (`Scene::params.w`) the gusts move by.
    pub wind: [f32; 4],
}

gpu_struct! {
    /// A quad of redstone wire: a piece of it, or an arm of one.
    pub struct GpuWireQuad as "WireQuad" {
        /// The cell, and w its face (3 bits) and which part of it (`wire.wgsl`).
        pub pos: [i32; 4],
        /// x texture index, y light (as `GpuParticle`), z its piece's number among the levels.
        pub data: [u32; 4],
    }
}

gpu_struct! {
    pub struct GpuWeather as "WeatherColumn" {
        /// The column's middle relative to the camera in x and z, its strip's bottom and top in y.
        /// As x, bottom, z, top.
        pub place: [f32; 4],
        /// x its left in the picture, y its top, z how much of the picture high, w alpha.
        pub uv: [f32; 4],
        /// x texture index, y light of the column (as `GpuParticle`), z a number of its own, w how
        /// much of the picture shows (0..255).
        pub data: [u32; 4],
    }
}

gpu_struct! {
    pub struct GpuParticle as "Particle" {
        /// Center relative to the camera, and edge length in blocks.
        pub pos_size: [f32; 4],
        /// x color 0xRRGGBBAA, y texture (16 bits) | shader slot << 24, z uv rectangle as four
        /// bytes (u0 v0 u1 v1, /255), w light: sky, red, green, blue bytes (0..15), all ones =
        /// unlit.
        pub data: [u32; 4],
        /// x life fraction 0..1; y 1 for a card that turns in 3D (a fluttering kind,
        /// `ParticleDef::flutter`), 0 one facing the camera, 2 + its length (blocks) for a streak
        /// along its motion (`ParticleDef::streak`); z the card's heading and w how far it is
        /// tipped about it (radians; 0 lying flat), or for a streak the motion's heading and
        /// climb.
        pub extra: [f32; 4],
    }
}

gpu_struct! {
    /// A UI quad (`ui.wgsl`).
    pub struct Quad as "Quad" {
        /// Corner, and the two edge vectors, in physical pixels.
        pub p0: [f32; 2],
        pub u: [f32; 2],
        pub v: [f32; 2],
        /// Text effect slot (top bit set on shadows) or GUI fill slot.
        pub fx: u32,
        /// Glyph number in its text, or how far an animated texture has blended into its next frame.
        pub index: f32,
        pub uv: [f32; 4],
        pub color: u32,
        pub tex: u32,
        /// A texture animated as a strip of frames: `ui_strip_record` (0: still). For text
        /// (`FONT`): 1 if the font is smooth, sampled linearly.
        pub layer: u32,
        pub mode: u32,
        /// Visible rectangle in physical pixels; empty (all zero) means unclipped.
        pub clip: [f32; 4],
    }
}

gpu_struct! {
    pub struct Screen as "Screen" {
        /// Pixels, seconds since start, GUI scale.
        pub size: [f32; 4],
    }
}

gpu_struct! {
    /// One screen effect pass (`post.wgsl`).
    pub struct Pass as "Pass" {
        /// Bindless index of the image to read, the effect's slot, its strength and the time.
        pub scene: u32,
        pub slot: u32,
        pub strength: f32,
        pub time: f32,
        pub size: [f32; 2],
        /// Bindless index of the world's depth (reverse Z: 0 is the far plane, the sky).
        pub depth: u32,
        pub pad: u32,
        /// Bindless index + 1 of this effect's own image at half the size each way, which its
        /// `shaders/post_low/<name>.wgsl` drew before the effects (0: none; chiacraft).
        pub low: u32,
        pub spare: u32,
        /// With the HDR world (chiacraft): bindless index + 1 of its bloom, the widest level of the
        /// pyramid the pack's `fs_bloom_down`/`fs_bloom_up` drew from the world this frame, and of
        /// this frame's exposure texel (`fs_exposure`); 0 without. In the pyramid's own passes `low`
        /// is the second image they read and `strength` the seconds since the last frame.
        pub bloom: u32,
        pub exposure: u32,
    }
}

gpu_struct! {
    /// One pass of the path tracer's denoiser (`denoise.wgsl`, `denoise::Denoiser`): bindless
    /// indices of what it reads (their meaning by pass is in the shader's header), + 1 where 0 means
    /// none.
    pub struct DenoisePass as "DenoisePass" {
        pub diffuse: u32,
        pub specular: u32,
        pub albedo: u32,
        pub normal: u32,
        /// The G-buffer's F0.
        pub spec: u32,
        pub depth: u32,
        /// The motion image + 1.
        pub motion: u32,
        pub prev_diffuse: u32,
        pub prev_specular: u32,
        pub prev_moments: u32,
        /// Last frame's guide + 1 (0: no last frame, start anew).
        pub prev_guide: u32,
        pub moments: u32,
        /// The à-trous step's tap spacing, in texels.
        pub step: u32,
        /// 1: nothing of last frame is used (a teleport, a new world).
        pub reset: u32,
        pub size: [f32; 2],
        /// The least share of a new frame, and the most frames gathered.
        pub alpha: f32,
        pub max_len: f32,
        /// The à-trous light test (standard deviations) and normal test (power).
        pub phi_color: f32,
        pub phi_normal: f32,
    }
}

/// Group 1 of each engine shader, in order: a constant per binding (`bind::world::RECORDS`),
/// the WGSL name and type the prelude declares.
macro_rules! bindings {
    ($($module:ident $file:literal { $($konst:ident $name:literal: $ty:literal),* $(,)? })*) => {
        /// Binding numbers of each engine shader's group 1, by name.
        pub mod bind {
            $(pub mod $module { bindings!(@number 0u32; $($konst)*); })*
        }

        fn bindings(shader: &str) -> &'static [(&'static str, &'static str)] {
            match shader {
                $($file => &[$(($name, $ty)),*],)*
                _ => &[],
            }
        }
    };
    (@number $n:expr; $first:ident $($rest:ident)*) => {
        pub const $first: u32 = $n;
        bindings!(@number $n + 1; $($rest)*);
    };
    (@number $n:expr;) => {};
}

bindings! {
    world "world.wgsl" {
        G "g": "Globals",
        ORIGINS "origins": "array<vec4<i32>>",
        RECORDS "records": "array<u32>",
        // Per biome a color per tint kind (`g.extra.z` kinds), 0xRRGGBB. Then the blended
        // colors of loaded block columns, 0xFFRRGGBB, in 16x16 tiles addressed by column
        // position modulo g.ids.w.
        BIOMES "biomes": "array<u32>",
        // Two words per texture, see `GameTextures::world_tables`.
        TEX_INFO "tex_info": "array<u32>",
        // Animated frames: texture | ticks << 16; then the overlay table.
        FRAMES "frames": "array<u32>",
        // Camera-relative positions of moving structures drawn this frame (0: none, the
        // world's cubes): a cube's `origin.w` picks one, its `origin.xyz` is then the cube in
        // the structure. Its w is the world's light where the structure is, as bits.
        OFFSETS "offsets": "array<vec4<f32>>",
        // Glow mask texture per bindless texture index, all ones: none.
        GLOW_MAP "glow_map": "array<u32>",
        // Block shader slot (`shaders/block/<name>.wgsl`) per texture id, 0 none.
        TEX_SHADER "tex_shader": "array<u32>",
        SCENE "scene": "Scene",
        // A bit per cube column around the camera's, addressed modulo 2 * g.extra.w + 1:
        // its cubes show its ground, distant terrain is not drawn there (`client::lod`).
        LOD_MASK "lod_mask": "array<u32>",
        // Which blocks around the camera cast shadows, for shader packs that call `solid_at`
        // (`client::world_render::Solid`); a single word otherwise.
        SOLID "solid": "array<u32>",
        // Material maps per bindless color index (chiacraft): two words each, its LabPBR
        // specular map's index and its normal map's, all ones: none. `mat_at` reads them.
        MAT_MAP "mat_map": "array<u32>",
    }
    // `item_boxes`: item models in their own space, kept between frames
    // (`EntityRenderer::item_model`); an item instance is a box in `boxes` whose rows place
    // the model, `ids.z` its first box, `color.a` the hurt flash and `ids2.y` the light.
    entity "entity.wgsl" {
        BOXES "boxes": "array<Box>",
        SCENE "scene": "Scene",
        GLOW_MAP "glow_map": "array<u32>",
        ITEM_BOXES "item_boxes": "array<Box>",
    }
    model "model.wgsl" {
        INSTANCES "instances": "array<Instance>",
        SCENE "scene": "Scene",
        CUBOIDS "cuboids": "array<Cuboid>",
        CORNERS "corners": "array<Corner>",
        BONES "bones": "array<Bone>",
        // The same bones last frame, at the same indices, taking this frame's camera-relative
        // positions (this frame's where an entity is new), for the motion image.
        PREV_BONES "prev_bones": "array<Bone>",
    }
    particle "particle.wgsl" {
        PARTICLES "particles": "array<Particle>",
        SCENE "scene": "Scene",
    }
    occlusion "occlusion.wgsl" {
        DEPTHS "depths": "array<f32>",
        ORIGINS "origins": "array<vec4<i32>>",
        DRAWS "draws": "array<u32>",
        VIEW "view": "View",
        TABLE "table": "array<Entry>",
        BUCKETS "buckets": "array<Bucket>",
        OUT_DRAWS "out_draws": "array<u32>",
        OUT_ORIGINS "out_origins": "array<vec4<i32>>",
        FACES_DRAWN "faces_drawn": "Counter",
    }
    particle_sim "particle_sim.wgsl" {
        SIM "sim": "array<SimParticle>",
        DST "dst": "array<Particle>",
        KINDS "kinds": "array<ParticleKind>",
        RULES "rules": "array<ParticleRule>",
        FRAMES "frames": "array<u32>",
        CELLS "cells": "array<u32>",
    }
    wire "wire.wgsl" {
        QUADS "quads": "array<WireQuad>",
        LEVELS "levels": "array<u32>",
        SCENE "scene": "Scene",
    }
    weather "weather.wgsl" {
        COLUMNS "columns": "array<WeatherColumn>",
        SCENE "scene": "Scene",
    }
    sky "sky.wgsl" {
        OBJECTS "sky": "array<Sky>",
        SCENE "scene": "Scene",
    }
    ui "ui.wgsl" {
        QUADS "quads": "array<Quad>",
        SCREEN "screen": "Screen",
    }
    post "post.wgsl" {
        PASSES "passes": "array<Pass>",
        // The frame the world was drawn with (all zero without one): where the sun is on the
        // screen, the weather, for effects such as light shafts.
        SCENE "scene": "Scene",
    }
    // Temporal anti-aliasing (`post::Post`): the same as the screen effects', so their pass
    // entries serve.
    taa "taa.wgsl" {
        PASSES "passes": "array<Pass>",
        SCENE "scene": "Scene",
    }
    upscale_guides "upscale_guides.wgsl" {
        PASSES "passes": "array<Pass>",
        SCENE "scene": "Scene",
    }
    // The path tracer's denoiser (`denoise::Denoiser`).
    denoise "denoise.wgsl" {
        PASSES "dn": "array<DenoisePass>",
        SCENE "scene": "Scene",
    }
}

/// Whether the block at `p` (block coordinates) casts a shadow, as far as the client has it
/// within about 112 blocks of the camera; false beyond, for unloaded cubes and while no shader
/// pack asks (`client::world_render::Solid`).
const SOLID_AT: &str = "
fn solid_at(p: vec3<i32>) -> bool {
    if arrayLength(&solid) < 4096u * 132u {
        return false;
    }
    let c = p >> vec3(4u);
    let s = u32(c.y & 15) * 256u + u32(c.z & 15) * 16u + u32(c.x & 15);
    if solid[s * 4u + 3u] == 0u || bitcast<i32>(solid[s * 4u]) != c.x || bitcast<i32>(solid[s * 4u + 1u]) != c.y || bitcast<i32>(solid[s * 4u + 2u]) != c.z {
        return false;
    }
    let l = p & vec3(15);
    let i = u32((l.y * 16 + l.z) * 16 + l.x);
    return ((solid[4096u * 4u + s * 128u + i / 32u] >> (i % 32u)) & 1u) != 0u;
}
";

/// `mat_at(layer, uv, albedo)`: the material of a block texel (chiacraft), decoded from its
/// texture's LabPBR 1.3 maps (`mat_map`, `assets::MatMaps`); a texture without maps is rough
/// stone (F0 0.04, a flat normal), so a pack calling it changes nothing for those. `mat_normal`
/// tilts a face's normal by the map, from the screen's derivatives (fragment shaders only).
const MAT_AT: &str = "
struct Mat {
    // LabPBR's perceptual smoothness, and roughness = (1 - smoothness)^2.
    smoothness: f32,
    rough: f32,
    // Reflectance at normal incidence, rgb (a metal's colour, or 0.04 for most stone and glass).
    f0: vec3<f32>,
    // 1 for a metal (no diffuse light of its own), 0 otherwise.
    metal: f32,
    // 0-1: how much water darkens and glosses it in rain; and its subsurface share.
    porosity: f32,
    sss: f32,
    // The normal in the texture's frame: x along u, y along v (down the picture), z out.
    n: vec3<f32>,
    ao: f32,
    height: f32,
    // Whether the texture has maps at all.
    has: bool,
}

fn mat_none() -> Mat {
    return Mat(0.0, 1.0, vec3(0.04), 0.0, 0.0, 0.0, vec3(0.0, 0.0, 1.0), 1.0, 1.0, false);
}

// LabPBR's metals 230-237 as F0: iron, gold, aluminium, chrome, copper, lead, platinum, silver.
fn mat_metal_f0(id: u32, albedo: vec3<f32>) -> vec3<f32> {
    switch id {
        case 230u: { return vec3(0.56, 0.57, 0.58); }
        case 231u: { return vec3(1.00, 0.71, 0.29); }
        case 232u: { return vec3(0.91, 0.92, 0.92); }
        case 233u: { return vec3(0.55, 0.56, 0.55); }
        case 234u: { return vec3(0.95, 0.64, 0.54); }
        case 235u: { return vec3(0.63, 0.63, 0.63); }
        case 236u: { return vec3(0.67, 0.64, 0.58); }
        case 237u: { return vec3(0.95, 0.93, 0.88); }
        default: { return albedo; }
    }
}

fn mat_at(layer: u32, uv: vec2<f32>, albedo: vec3<f32>) -> Mat {
    var m = mat_none();
    if layer * 2u + 1u >= arrayLength(&mat_map) {
        return m;
    }
    let si = mat_map[layer * 2u];
    let ni = mat_map[layer * 2u + 1u];
    if si == 0xffffffffu {
        return m;
    }
    m.has = true;
    let sp = textureSampleLevel(textures[si], nearest_clamp, uv, 0.0);
    m.smoothness = sp.r;
    m.rough = (1.0 - sp.r) * (1.0 - sp.r);
    let g = u32(round(sp.g * 255.0));
    if g >= 230u {
        m.metal = 1.0;
        m.f0 = mat_metal_f0(g, albedo);
    } else {
        m.f0 = vec3(sp.g);
    }
    let b = sp.b * 255.0;
    m.porosity = select(0.0, b / 64.0, b <= 64.5);
    m.sss = select(0.0, (b - 65.0) / 190.0, b > 64.5);
    if ni != 0xffffffffu {
        let nt = textureSampleLevel(textures[ni], nearest_clamp, uv, 0.0);
        let xy = nt.rg * 2.0 - 1.0;
        m.n = vec3(xy, sqrt(max(1.0 - dot(xy, xy), 0.0)));
        m.ao = nt.b;
        m.height = nt.a;
    }
    return m;
}

// The face's normal `n` tilted by the material's, at camera-relative `rel` and texture `uv`
// (the frame from the screen's derivatives: fragment shaders only).
fn mat_normal(m: Mat, n: vec3<f32>, rel: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    let dp1 = dpdx(rel);
    let dp2 = dpdy(rel);
    let du1 = dpdx(uv);
    let du2 = dpdy(uv);
    if m.n.z > 0.9999 {
        return n;
    }
    let p2 = cross(dp2, n);
    let p1 = cross(n, dp1);
    let t = p2 * du1.x + p1 * du2.x;
    let b = p2 * du1.y + p1 * du2.y;
    let k = inverseSqrt(max(max(dot(t, t), dot(b, b)), 1e-12));
    return normalize(t * k * m.n.x + b * k * m.n.y + n * m.n.z);
}
";

/// Every GPU struct as WGSL.
fn structs() -> String {
    [
        Scene::wgsl(),
        Globals::wgsl(),
        GpuBox::wgsl(),
        Instance::wgsl(),
        GpuCuboid::wgsl(),
        GpuCorner::wgsl(),
        GpuBone::wgsl(),
        GpuParticle::wgsl(),
        GpuSimParticle::wgsl(),
        GpuParticleKind::wgsl(),
        GpuParticleRule::wgsl(),
        GpuWireQuad::wgsl(),
        GpuWeather::wgsl(),
        GpuSky::wgsl(),
        Quad::wgsl(),
        Screen::wgsl(),
        Pass::wgsl(),
        DenoisePass::wgsl(),
    ]
    .concat()
}

/// What goes in front of an engine shader (or a pack's replacement of it): the constants, every
/// GPU struct, the shader's bindings, then `common` (lighting, fog, samplers and the structs
/// snippets get).
pub fn prelude(shader: &str, common: &str) -> String {
    let mut s = wgsl();
    s.push_str(&records());
    s.push_str(&structs());
    for (i, (name, ty)) in bindings(shader).iter().enumerate() {
        s.push_str(&format!("@group(1) @binding({i}) var<storage, read> {name}: {ty};\n"));
    }
    if shader == "world.wgsl" {
        s.push_str(SOLID_AT);
        s.push_str(LOD_CORNER);
        s.push_str(MAT_AT);
        s.push_str(if RT_PRELUDE.load(std::sync::atomic::Ordering::Relaxed) { RT_SUN } else { RT_SUN_OFF });
    }
    s.push_str(CALM_OFF);
    s.push_str(DRY_OFF);
    s.push_str(common);
    s.push('\n');
    s
}

static RT_PRELUDE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the world shader composed next traces its sun shadows (`core:rt_shadows` on a GPU
/// with ray queries): its prelude then has the scene's bindings and `rt_sun` traces; without, the
/// same function says the light is open, and nothing in the shader needs ray queries.
pub fn set_rt_prelude(on: bool) {
    RT_PRELUDE.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// The world shader's group 1 bindings of ray tracing's scene (`client::world_render::rt`): the
/// TLAS, the frame's instance table, the see-through records' copies.
/// Kept well past the world's own (`bindings!`, numbered in order), so new ones there never meet
/// them.
pub mod rt_bind {
    pub const TLAS: u32 = 30;
    pub const INSTANCES: u32 = 31;
    pub const RECORDS: u32 = 32;
}

const RT_SUN_OFF: &str = "
fn rt_sun(rel: vec3<f32>, n: vec3<f32>, l: vec3<f32>, plant: bool) -> vec3<f32> {
    return vec3(1.0);
}
fn rt_cone(l: vec3<f32>, radius: f32, px: vec2<f32>) -> vec3<f32> {
    return l;
}
fn rt_sun_soft(rel: vec3<f32>, n: vec3<f32>, l: vec3<f32>, plant: bool, radius: f32, px: vec2<f32>, fine: bool) -> vec3<f32> {
    return vec3(1.0);
}
";

/// `rt_sun(rel, n, l, plant)`: how much of the light coming along `l` (unit, toward the sun or
/// the moon) reaches the surface at `rel` (camera relative) facing `n`, rgb, 0 blocked to 1 open:
/// one ray through the scene's cubes and distant terrain. A leaf's or a plant's see-through texels
/// let it by, a stained pane colours it, and water (and any tinted see-through block) lets it
/// through whole, as the shadow maps did. From a plant (`plant`) the free quads (other plants)
/// don't shade it, or a meadow would shade itself in specks.
const RT_SUN: &str = "
enable wgpu_ray_query;
const RT_ALPHA_MIP: f32 = 2.0;
@group(1) @binding(30) var rt_scene: acceleration_structure;
@group(1) @binding(31) var<storage, read> rt_inst: array<vec4<u32>>;
@group(1) @binding(32) var<storage, read> rt_records: array<u32>;

// Whether real cubes stand in the column of the distant terrain's hit at `rel` (`Lod::mask`):
// then the tile isn't there.
fn rt_lod_hidden(rel: vec3<f32>) -> bool {
    let half = i32(g.extra.w & 0xFFFFFFu);
    let d = vec2<i32>(floor((rel.xz + g.cam_frac.xz) / 16.0));
    if abs(d.x) <= half && abs(d.y) <= half {
        let col = d + g.cam_cube.xz;
        let w = 2 * half + 1;
        let i = u32(((col.y % w) + w) % w * w + ((col.x % w) + w) % w);
        return ((lod_mask[i / 32u] >> (i % 32u)) & 1u) != 0u;
    }
    return false;
}

// A direction toward a point of the light's disc round `l` (unit), `radius` radians across from its
// middle, for the pixel at `px` (`@builtin(position)`): a different point each frame of temporal
// anti-aliasing's jitter (`scene.taa.xy`), so its history averages one ray a pixel into the
// penumbra a light of that size casts (soft shadows from one ray). `l` itself without the jitter.
fn rt_cone(l: vec3<f32>, radius: f32, px: vec2<f32>) -> vec3<f32> {
    return rt_cone_at(l, radius, px, 0.0);
}

// The same, turned by `turn` (0..1 of a circle) and its distance from the middle mirrored for odd
// halves, for further rays of one pixel spread over the disc (`rt_sun_soft`).
fn rt_cone_at(l: vec3<f32>, radius: f32, px: vec2<f32>, turn: f32) -> vec3<f32> {
    let j = scene.taa.xy;
    if j.x == 0.0 && j.y == 0.0 {
        return l;
    }
    // Interleaved gradient noise, moved on every frame (`scene.rt.y`, a frame count: TAA's jitter
    // has only a few phases, and a pattern repeating every 8 frames left a grain its history kept).
    let k = f32(u32(scene.rt.y) % 64u);
    let p = px + vec2(5.588238 * k);
    var u1 = fract(52.9829189 * fract(dot(p, vec2(0.06711056, 0.00583715))));
    if fract(turn * 2.0) >= 0.5 {
        u1 = 1.0 - u1;
    }
    let u2 = fract(52.9829189 * fract(dot(p.yx + vec2(17.0, 31.0), vec2(0.06711056, 0.00583715))) + turn);
    let r = radius * sqrt(u1);
    let a = 6.2831853 * u2;
    let t = normalize(cross(l, select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(l.y) > 0.9)));
    let b = cross(l, t);
    return normalize(l + (t * cos(a) + b * sin(a)) * r);
}

// `rt_sun` toward a light of disc `radius` (radians), soft-edged: with temporal anti-aliasing's
// jitter on, one ray to a point of the disc that moves every frame, the history averaging them into
// the penumbra. `fine`: two rays to opposite sides of the disc, and where they disagree (a penumbra)
// two more across it, smooth within the frame but about three times the cost (PC, 11-06: +0.7-2.5
// ms at 1440p against +0.2-0.7). Without the jitter one ray to the middle.
fn rt_sun_soft(rel: vec3<f32>, n: vec3<f32>, l: vec3<f32>, plant: bool, radius: f32, px: vec2<f32>, fine: bool) -> vec3<f32> {
    let j = scene.taa.xy;
    if j.x == 0.0 && j.y == 0.0 {
        return rt_sun(rel, n, l, plant);
    }
    let a = rt_sun(rel, n, rt_cone_at(l, radius, px, 0.0), plant);
    if !fine {
        return a;
    }
    let b = rt_sun(rel, n, rt_cone_at(l, radius, px, 0.5), plant);
    if all(abs(a - b) < vec3(0.05)) {
        return (a + b) * 0.5;
    }
    let c = rt_sun(rel, n, rt_cone_at(l, radius, px, 0.25), plant);
    let d = rt_sun(rel, n, rt_cone_at(l, radius, px, 0.75), plant);
    return (a + b + c + d) * 0.25;
}

struct RtTexel {
    c: vec4<f32>,
    // The block's texture is tinted (water, a leaf's biome colour).
    tinted: bool,
}

// The texel a see-through record shows where a ray met it: record `rec` of the copy at word
// `base`, kind 0 a face, 1 a free quad, its triangle `tri` (0: corners 0 1 2, 1: 0 2 3) at
// barycentrics `b`. Its alpha from a smaller mip (a few texels wide): a leaf's single-texel holes,
// lined up through a crown's faces, let the light by in pixel-wide specks; the coverage round the
// point gives the soft dapple a filtered shadow map shows.
fn rt_texel(base: u32, kind: u32, rec: u32, tri: u32, b: vec2<f32>) -> RtTexel {
    var uv: array<vec2<f32>, 4>;
    var tex: u32;
    var tinted: bool;
    if kind == 0u {
        let w0 = rt_records[base + rec * 4u];
        tex = rt_records[base + rec * 4u + 1u] & FACE_TEX_MASK;
        tinted = ((w0 >> FACE_TINT_SHIFT) & TINT_MASK) != 0u;
        let turn = (w0 >> FACE_TURN_SHIFT) & FACE_TURN_MASK;
        for (var k = 0u; k < 4u; k++) {
            uv[k] = UVS[(k + turn) % 4u];
        }
    } else {
        let at = base + rec * 8u;
        tex = rt_records[at + 4u] & FACE_TEX_MASK;
        let w5 = rt_records[at + 5u];
        tinted = ((w5 >> QUAD_TINT_SHIFT) & TINT_MASK) != 0u;
        let q = quad_uv_fine(w5, rt_records[at], rt_records[at + 1u], rt_records[at + 2u], rt_records[at + 3u]);
        uv = array<vec2<f32>, 4>(vec2(q.x, q.w), vec2(q.z, q.w), vec2(q.z, q.y), vec2(q.x, q.y));
    }
    let k1 = select(1u, 2u, tri == 1u);
    let k2 = select(2u, 3u, tri == 1u);
    let at = uv[0] * (1.0 - b.x - b.y) + uv[k1] * b.x + uv[k2] * b.y;
    let t = textures[tex_info[tex * 4u]];
    let c = textureSampleLevel(t, nearest_clamp, at, 0.0);
    return RtTexel(vec4(c.rgb, textureSampleLevel(t, linear_mipped, at, RT_ALPHA_MIP).a), tinted);
}

fn rt_sun(rel: vec3<f32>, n: vec3<f32>, l: vec3<f32>, plant: bool) -> vec3<f32> {
    // Off the surface so it never meets itself: out along its normal (not a plant's blade, which
    // faces either way and isn't in the plants' rays), then a hair toward the light.
    let lift = select(n * 0.004, vec3(0.0), plant) + l * 0.002;
    let mask = select(0xFFu, 0xFFu & ~2u, plant);
    var rq: ray_query;
    // The TLAS is in the space of the camera's cube (`world_render::rt`), the camera at `g.cam_frac`.
    rayQueryInitialize(&rq, rt_scene, RayDesc(RAY_FLAG_TERMINATE_ON_FIRST_HIT, mask, 0.0, 16384.0, rel + lift + g.cam_frac.xyz, l));
    var through = vec3(1.0);
    while rayQueryProceed(&rq) {
        let h = rayQueryGetCandidateIntersection(&rq);
        let row = rt_inst[h.instance_custom_data];
        if row.z == 2u {
            if !rt_lod_hidden(rel + lift + l * h.t) {
                rayQueryConfirmIntersection(&rq);
            }
            continue;
        }
        let rec = h.primitive_index / 2u;
        let texel = rt_texel(row.x, row.z, rec, h.primitive_index % 2u, h.barycentrics);
        if rec >= row.y {
            // Translucent: water and tinted panes let it through; stained glass colours it.
            if !texel.tinted {
                through *= mix(vec3(1.0), texel.c.rgb, texel.c.a);
                if max(through.r, max(through.g, through.b)) < 0.02 {
                    rayQueryConfirmIntersection(&rq);
                }
            }
        } else {
            // Cutout: as much gets by as the texture's coverage there leaves open.
            through *= clamp(1.0 - texel.c.a, 0.0, 1.0);
            if max(through.r, max(through.g, through.b)) < 0.02 {
                rayQueryConfirmIntersection(&rq);
            }
        }
    }
    if rayQueryGetCommittedIntersection(&rq).kind != RAY_QUERY_INTERSECTION_NONE {
        return vec3(0.0);
    }
    return through;
}
";

/// What goes in front of `rt_expand.wgsl` (ray tracing's scene, `world_render::rt`): the
/// constants and record decoders, the job, what it reads and writes.
pub fn rt_expand_prelude() -> String {
    let mut s = wgsl();
    s.push_str(&records());
    s.push_str(concat!(
        "struct RtJob {\n    src: u32,\n    count: u32,\n    dst: u32,\n    kind: u32,\n    copy: u32,\n    copy_from: u32,\n    pad: vec2<u32>,\n}\n",
        "var<immediate> job: RtJob;\n",
        "@group(1) @binding(0) var<storage, read> records: array<u32>;\n",
        "@group(1) @binding(1) var<storage, read_write> positions: array<f32>;\n",
        "@group(1) @binding(2) var<storage, read_write> rt_records: array<u32>;\n",
    ));
    s
}
/// `CALM` in every shader: false, but true in the world's calm pipelines (`WorldRenderer::set_calm`),
/// used while the scene has no fog in the air (`scene.weather.z` 0), no thunder (`scene.lightning.z`
/// under 0.05), no held light (`scene.hand_light` 0) and the eye is in no liquid. A pack leaves
/// out what does nothing then behind `if !CALM`: a smaller shader for the same picture (Android
/// port: on the Pixel 10 the rare paths' size alone cost the vista a fifth of its world pass).
pub const CALM_OFF: &str = "const CALM: bool = false;\n";
pub const CALM_ON: &str = "const CALM: bool = true;\n";
/// `DRY` in every shader: false, but true in the world's calm-and-dry pipelines (level 2 of
/// `WorldRenderer::set_calm`), used while the scene is calm and also has no rain
/// (`scene.fog_color.w` 0), dry ground (`scene.canopy.y` 0), no puddles (`scene.canopy.z` 0) and no
/// snow lying where the player is (`scene.ground.x` 0). A pack leaves out what those draw behind
/// `if !DRY` (Android port: 1.8 ms of the Pixel 10's vista).
pub const DRY_OFF: &str = "const DRY: bool = false;\n";
pub const DRY_ON: &str = "const DRY: bool = true;\n";

/// What goes in front of `occlusion.wgsl`.
pub fn occlusion_prelude() -> String {
    let mut s = String::from(
        "struct View {\n    prev: mat4x4<f32>,\n    planes: array<vec4<f32>, 4>,\n    moved: vec4<f32>,\n    cam_frac: vec4<f32>,\n    cam_cube: vec4<i32>,\n    size: vec2<u32>,\n    skip: vec2<u32>,\n    levels: u32,\n    hiz: u32,\n    pad: vec2<u32>,\n}\nstruct Entry {\n    cube: vec3<i32>,\n    bucket: u32,\n    start: u32,\n    counts: array<u32, 3>,\n}\nstruct Bucket {\n    count: atomic<u32>,\n    base: u32,\n}\nstruct Counter {\n    count: atomic<u32>,\n}\nstruct Args {\n    size: vec2<u32>,\n    src: u32,\n    dst: u32,\n    count: u32,\n    pad: u32,\n}\nvar<immediate> args: Args;\n",
    );
    for (i, (name, ty)) in bindings("occlusion.wgsl").iter().enumerate() {
        let access = if matches!(*name, "origins" | "view" | "table") { "read" } else { "read_write" };
        s.push_str(&format!("@group(1) @binding({i}) var<storage, {access}> {name}: {ty};\n"));
    }
    s
}

/// What goes in front of `particle_sim.wgsl`: the constants, the structs, what it reads and writes.
pub fn particle_sim_prelude(window_reach: i32) -> String {
    let mut s = wgsl();
    s.push_str(&structs());
    for (i, (name, ty)) in bindings("particle_sim.wgsl").iter().enumerate() {
        let access = if matches!(*name, "sim" | "dst") { "read_write" } else { "read" };
        s.push_str(&format!("@group(1) @binding({i}) var<storage, {access}> {name}: {ty};\n"));
    }
    use weft_abi::{particle_event as pe, particle_flags as pf, particle_then as pt, particle_when as pw};
    s.push_str(&format!(
        "struct SimArgs {{\n    first: u32,\n    count: u32,\n    dt: f32,\n    pad: f32,\n    shift: vec4<f32>,\n    rel: vec4<f32>,\n    origin: vec4<i32>,\n    center: vec4<i32>,\n    wind: vec4<f32>,\n}}\nvar<immediate> args: SimArgs;\nconst WINDOW_REACH: i32 = {window_reach};\nconst PARTICLE_FLAG_COLLIDE: u32 = {}u;\nconst PARTICLE_FLAG_FULLBRIGHT: u32 = {}u;\nconst PARTICLE_FLAG_SHRINK: u32 = {}u;\nconst PARTICLE_FLAG_FLOATS: u32 = {}u;\n",
        pf::COLLIDE, pf::FULLBRIGHT, pf::SHRINK, pf::FLOATS
    ));
    let consts = [
        ("EVENT_LAND", pe::LAND),
        ("EVENT_WALL", pe::WALL),
        ("EVENT_CEILING", pe::CEILING),
        ("EVENT_ENTER_LIQUID", pe::ENTER_LIQUID),
        ("EVENT_LEAVE_LIQUID", pe::LEAVE_LIQUID),
        ("EVENT_EXPIRE", pe::EXPIRE),
        ("WHEN_ANY", pw::ANY),
        ("WHEN_LIQUID", pw::LIQUID),
        ("WHEN_TAG", pw::TAG),
        ("THEN_DIE", pt::DIE),
        ("THEN_STOP", pt::STOP),
        ("THEN_SLIDE", pt::SLIDE),
        ("THEN_BOUNCE", pt::BOUNCE),
        ("THEN_BECOME", pt::BECOME),
        ("THEN_BECOME_STILL", pt::BECOME_STILL),
    ];
    for (name, v) in consts {
        s.push_str(&format!("const {name}: u32 = {v}u;\n"));
    }
    s
}

#[cfg(test)]
mod quad_uv_tests {
    use super::*;

    #[test]
    fn quad_uv_keeps_a_fine_unwrap() {
        let (w, on) = quad_uv([0.25, 16.0, 1.25, 31.875], quad::FLAT);
        let m = quad::UV_MASK;
        // Whole units where they always were, for what reads only those.
        assert_eq!([w & m, (w >> 5) & m, (w >> 10) & m, (w >> 15) & m], [0, 16, 1, 31]);
        assert_eq!((w >> quad::TINT_SHIFT) & 31, quad::FLAT);
        let fine = on[0] >> 30 | (on[1] >> 30) << 2 | (on[2] >> 30) << 4 | (on[3] >> 30) << 6 | ((w >> 20) & 15) << 8;
        assert_eq!([fine & 7, (fine >> 3) & 7, (fine >> 6) & 7, (fine >> 9) & 7], [2, 0, 2, 7]);
    }

    #[test]
    fn quad_middle_ignores_the_uv_on_the_corners() {
        let word = |p: [f32; 3]| quad_pos(p) | 3 << quad::UV_FINE_LOW_SHIFT;
        let (a, c) = (word([1.0, 2.0, -1.0]), word([2.0, 2.0, 0.0]));
        assert_eq!(quad_middle(&[a, 0, c, 0, 0, 0, 0, 0]), [48, 64, -16]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Records pack into the fields the constants say, and the prelude's decoders of them
    /// compile against the same constants (the pictures of the visual tests check the rest).
    #[test]
    fn records_round_trip() {
        let f = face_record([3, 14, 9], 5, 0b10_01_11_00, 7, 2, 1234, 42, [0x1234, 0xFFFF, 0, 0xABCD]);
        assert_eq!(face_dir(&f), 5);
        assert_eq!(f[0] & face::POS_MASK, 3);
        assert_eq!((f[0] >> face::Y_SHIFT) & face::POS_MASK, 14);
        assert_eq!((f[0] >> face::Z_SHIFT) & face::POS_MASK, 9);
        assert_eq!((f[0] >> face::TINT_SHIFT) & tint::MASK, 7);
        assert_eq!(f[1] & face::TEX_MASK, 1234);
        assert_eq!(corner_light([200, 3, 15, 16]), 15 | 3 << 4 | 15 << 8 | 15 << 12);
        let q = quad_pos([-2.0, 0.5, 61.9375]);
        assert_eq!([q & quad::POS_MASK, (q >> quad::POS_BITS) & quad::POS_MASK, q >> (2 * quad::POS_BITS)], [0, 40, 1023]);
        let l = lod_record(4, [2047, -300, 5], [16, 7, 32], 9, 3, 2);
        assert_eq!(l[0] & lod_face::POS_MASK, 2047);
        assert_eq!((l[0] >> lod_face::Z_SHIFT) & lod_face::POS_MASK, 5);
        assert_eq!((l[0] >> lod_face::DIR_SHIFT) & face::DIR_MASK, 4);
        assert_eq!(((l[2] << 16) as i32) >> 16, -300);
        assert_eq!([l[3] & 0xFFFF, l[2] >> 16, l[3] >> 16], [16, 7, 32]);
        let src = format!(
            "{}{}const F0: u32 = {}u;\nconst Q: u32 = {}u;\nconst L0: u32 = {}u;\nconst L2: u32 = {}u;\nconst L3: u32 = {}u;\n\
             @compute @workgroup_size(1) fn main() {{ _ = face_pos(F0); _ = face_dir(F0); _ = face_ao(F0); _ = quad_pos(Q); _ = lod_pos(L0, L2); _ = lod_size(L2, L3); _ = corner_light(1u, 2u, 3u); _ = quad_uv(Q); _ = quad_uv_fine(Q, Q, Q, Q, Q); _ = lod_dir(L0); _ = lod_tint(L0); }}\n",
            wgsl(),
            records(),
            f[0],
            q,
            l[0],
            l[2],
            l[3]
        );
        let module = naga::front::wgsl::parse_str(&src).expect("records parse");
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&module)
            .expect("records validate");
    }

    /// Distant terrain's vertex helper compiles against the records' decoders, and a smooth
    /// cell's words hold what it reads.
    #[test]
    fn lod_corner_compiles() {
        let cell = lod_grid_cell(31, 5, 4, 300, 77, 2, 9);
        assert_eq!((cell[0] >> lod_face::DIR_SHIFT) & face::DIR_MASK, lod_face::GRID);
        assert_eq!([cell[0] & lod_face::POS_MASK, (cell[0] >> lod_face::Z_SHIFT) & lod_face::POS_MASK], [31 * 16, 5 * 16]);
        assert_eq!(
            [cell[3] & lod_face::CELL_MASK, (cell[3] >> lod_face::CELL_J_SHIFT) & lod_face::CELL_MASK, cell[3] >> lod_face::LEVEL_SHIFT],
            [31, 5, 4]
        );
        assert_eq!(((lod_grid(-7, 3) << 16) as i32) >> 16, -7);
        assert_eq!(lod_face::GRID_WORDS % 4, 0);
        let src = format!(
            "{}{}@group(0) @binding(0) var<storage, read> records: array<u32>;\n{LOD_CORNER}\n\
             @vertex fn main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {{ let k = lod_corner(vi / 4u, vi % 4u, vec3(1.0)); return vec4(k.local + k.normal, f32(k.dir + k.biome)); }}\n",
            wgsl(),
            records(),
        );
        let module = naga::front::wgsl::parse_str(&src).expect("lod_corner parses");
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&module)
            .expect("lod_corner validates");
    }

    /// The terrain shader with its prelude validates, and the header has room for every seed.
    #[test]
    fn terrain_prelude() {
        use terrain::*;
        assert_eq!(d::SEEDS as usize + SEEDS.len(), d::FIELD_OFF as usize);
        assert!(d::SCATTER_OFF < d::LEN);
        let src = prelude() + include_str!("../assets/shaders/terrain.wgsl");
        let module = naga::front::wgsl::parse_str(&src).expect("terrain shader parses");
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&module)
            .expect("terrain shader validates");
    }

    /// naga's layout of every generated struct matches the Rust one, so a field the WGSL rules
    /// align differently (a lone vec3) can't slip in.
    #[test]
    fn layouts_match_rust() {
        let src = format!("{}{}{}", wgsl(), records(), structs());
        let module = naga::front::wgsl::parse_str(&src).expect("structs parse");
        let mut layouter = naga::proc::Layouter::default();
        layouter.update(module.to_ctx()).unwrap();
        let sizes = [
            ("Scene", size_of::<Scene>()),
            ("Globals", size_of::<Globals>()),
            ("Box", size_of::<GpuBox>()),
            ("Instance", size_of::<Instance>()),
            ("Cuboid", size_of::<GpuCuboid>()),
            ("Corner", size_of::<GpuCorner>()),
            ("Bone", size_of::<GpuBone>()),
            ("Particle", size_of::<GpuParticle>()),
            ("Sky", size_of::<GpuSky>()),
            ("Quad", size_of::<Quad>()),
            ("Screen", size_of::<Screen>()),
            ("Pass", size_of::<Pass>()),
        ];
        for (name, rust) in sizes {
            let (h, _) = module
                .types
                .iter()
                .find(|(_, t)| t.name.as_deref() == Some(name))
                .expect(name);
            assert_eq!(layouter[h].size as usize, rust, "{name}");
        }
    }
}

/// A transform as the three rows the shaders read.
pub fn rows(m: glam::Mat4) -> [[f32; 4]; 3] {
    [m.row(0).to_array(), m.row(1).to_array(), m.row(2).to_array()]
}
