//! Bitmap font: an atlas image described by a `.font` file for the scripts it covers, Unifont
//! (`.hex`) for everything else. Glyphs are packed into one R8 atlas; advance widths come from the
//! inked columns so text looks proportional. A smooth font (`format = coverage`, baked from a font
//! file by a mod's tool) brings its glyphs' coverage in bigger cells and its own widths
//! (`Font::coverage`); Unifont fills in the rest, scaled up.

use rustc_hash::FxHashMap;

pub const CELL: u32 = 16;
const ATLAS_COLS: u32 = 64;

/// Where a cell of `w` x `h` atlas pixels starts. Cells are spaced with an empty pixel all around:
/// with fractional text scales a quad's edge can sample just outside its glyph, which must hit
/// blank space and not the neighboring glyph.
fn cell_origin(cell: u32, w: u32, h: u32) -> (u32, u32) {
    ((cell % ATLAS_COLS) * (w + 2) + 1, (cell / ATLAS_COLS) * (h + 2) + 1)
}

#[derive(Clone, Copy)]
pub struct Glyph {
    /// Atlas cell.
    pub cell: u32,
    /// First inked column of the cell, in atlas pixels.
    pub left: f32,
    /// In font pixels (16 a cell's height): the ink's width, where it starts from the pen, and
    /// how far the pen moves on.
    pub width: f32,
    pub bearing: f32,
    pub advance: f32,
}

impl Glyph {
    /// A bitmap glyph: its ink at the pen, the pen moving one pixel past it.
    fn bitmap(cell: u32, left: u8, advance: u8) -> Glyph {
        Glyph { cell, left: left as f32, width: advance as f32 - 1.0, bearing: 0.0, advance: advance as f32 }
    }
}

pub struct Font {
    pub glyphs: FxHashMap<char, Glyph>,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub atlas: Vec<u8>,
    pub line_height: f32,
    /// Capitals' height in font pixels, standing on the baseline (font pixel 14 of the line):
    /// for placing text by its top, as the hotbar's numbers are.
    pub caps: f32,
    /// Font pixels per pixel of the font's design (2 for the 8 pixel atlas): shadows and text
    /// placement step by this so glyph pixels stay whole.
    pub pixel: f32,
    /// Fixed cell width for monospaced text.
    pub mono: f32,
    /// A cell's size in atlas pixels: CELL x CELL for bitmaps, more for a smooth font, whose cell
    /// can be taller than the line; `unit` atlas pixels a font pixel (1 for bitmaps).
    pub cell_w: u32,
    pub cell_h: u32,
    pub unit: f32,
    /// Font pixels from the line's top to where a glyph's cell starts (negative: above the line),
    /// and its height: 0 and 16 for bitmaps.
    pub top: f32,
    pub height: f32,
    /// Coverage with soft edges, sampled smoothly (bitmaps: nearest).
    pub smooth: bool,
    fallback: Glyph,
}

impl Font {
    pub fn parse_hex(src: &str) -> Font {
        Self::new(src, None)
    }

    /// Unifont for everything, with the glyphs of a bitmap atlas (`.font` description and its
    /// image) drawn over it. Atlas pixels become `pixel` font pixels so both fill the 16 pixel cell.
    pub fn new(hex: &str, bitmap: Option<(&str, &crate::assets::Image)>) -> Font {
        let mut bitmaps = unifont(hex);

        let mut pixel = 1.0;
        let mut line_height = 16.0;
        if let Some((desc, img)) = bitmap {
            let mut cell = (8u32, 8u32);
            let mut grid: Vec<Vec<char>> = Vec::new();
            for line in desc.lines() {
                let Some((k, v)) = line.split_once(" = ") else { continue };
                match k.trim() {
                    "cell" => {
                        let n: Vec<u32> = v
                            .split_whitespace()
                            .filter_map(|n| n.parse().ok())
                            .collect();
                        if let [w, h] = n[..] {
                            cell = (w, h);
                        }
                    }
                    "row" => grid.push(v.chars().collect()),
                    _ => {}
                }
            }
            let scale = (CELL / cell.1.max(1)).max(1);
            if cell.0 * scale <= CELL {
                pixel = scale as f32;
                line_height = ((cell.1 + 1) * scale) as f32;
                let mut by_char: FxHashMap<char, usize> = bitmaps.iter().enumerate().map(|(i, b)| (b.0, i)).collect();
                let mut put = |ch: char, rows: [u16; 16], metrics: Option<(u8, u8)>| match by_char.get(&ch) {
                    Some(&i) => bitmaps[i] = (ch, rows, metrics),
                    None => {
                        by_char.insert(ch, bitmaps.len());
                        bitmaps.push((ch, rows, metrics));
                    }
                };
                // Vanilla-like spacing: a space is four atlas pixels, glyphs keep one between them.
                put(' ', [0; 16], Some((0, (4 * scale) as u8)));
                for (gy, row) in grid.iter().enumerate() {
                    for (gx, &ch) in row.iter().enumerate() {
                        let mut rows = [0u16; 16];
                        let (mut min_x, mut max_x) = (u32::MAX, 0);
                        for y in 0..cell.1 {
                            for x in 0..cell.0 {
                                let (px, py) = (gx as u32 * cell.0 + x, gy as u32 * cell.1 + y);
                                if px >= img.width || py >= img.height || img.rgba[((py * img.width + px) * 4 + 3) as usize] < 128 {
                                    continue;
                                }
                                min_x = min_x.min(x);
                                max_x = max_x.max(x);
                                for sy in 0..scale {
                                    for sx in 0..scale {
                                        rows[(y * scale + sy) as usize] |= 0x8000 >> (x * scale + sx);
                                    }
                                }
                            }
                        }
                        if min_x != u32::MAX {
                            put(ch, rows, Some(((min_x * scale) as u8, ((max_x - min_x + 2) * scale) as u8)));
                        }
                    }
                }
            }
        }

        let rows = (bitmaps.len() as u32 + 1).div_ceil(ATLAS_COLS);
        let atlas_width = ATLAS_COLS * (CELL + 2);
        let atlas_height = rows * (CELL + 2);
        let mut atlas = vec![0u8; (atlas_width * atlas_height) as usize];
        let mut glyphs = FxHashMap::default();
        for (n, (ch, bits, metrics)) in bitmaps.iter().enumerate() {
            let cell = n as u32 + 1;
            let (cx, cy) = cell_origin(cell, CELL, CELL);
            let (mut min_x, mut max_x) = (u32::MAX, 0);
            for (y, &row) in bits.iter().enumerate() {
                for x in 0..16u32 {
                    if row & (0x8000 >> x) != 0 {
                        atlas[((cy + y as u32) * atlas_width + cx + x) as usize] = 255;
                        min_x = min_x.min(x);
                        max_x = max_x.max(x);
                    }
                }
            }
            let (left, advance) = metrics.unwrap_or((min_x.min(255) as u8, (max_x + 2).saturating_sub(min_x).min(255) as u8));
            glyphs.insert(*ch, Glyph::bitmap(cell, left, advance));
        }
        missing_box(&mut atlas, atlas_width, CELL, CELL);
        let mono = mono_width(&glyphs);
        Font {
            glyphs,
            atlas_width,
            atlas_height,
            atlas,
            line_height,
            caps: 7.0 * pixel,
            pixel,
            mono,
            cell_w: CELL,
            cell_h: CELL,
            unit: 1.0,
            top: 0.0,
            height: CELL as f32,
            smooth: false,
            fallback: Glyph::bitmap(0, 2, 13),
        }
    }

    /// A smooth font: the glyphs of a coverage atlas (`format = coverage`, `cell = w h`,
    /// `scale = n` atlas pixels a font pixel, `baseline = y` in the cell, then `g <code> <col> <row> <left> <width> <bearing> <advance>`,
    /// all in atlas pixels; see `mods/base/source/ui/font_bake.py`), and Unifont, scaled up, for the
    /// rest. Its baseline sits where the bitmap font's does (font pixel 14 of the line).
    /// A pixel font baked one design pixel a font pixel says so: `pixel = 1` (font pixels a design
    /// pixel, 2 by default), `line = n` (the line's height in font pixels, 18 by default) and
    /// `sharp = true` (sampled nearest, not smoothly), and `caps = n` (capitals' height, font pixels).
    pub fn coverage(hex: &str, desc: &str, img: &crate::assets::Image) -> Option<Font> {
        let (mut size, mut cell_h, mut baseline, mut unit) = (0u32, 0u32, 0.0f32, 0u32);
        let (mut pixel, mut line_height, mut smooth, mut caps) = (2.0f32, 18.0f32, true, 11.5f32);
        let mut baked: Vec<(char, u32, u32, f32, f32, f32, f32)> = Vec::new();
        for line in desc.lines() {
            if let Some(v) = line.strip_prefix("cell = ") {
                let n: Vec<u32> = v
                    .split_whitespace()
                    .filter_map(|n| n.parse().ok())
                    .collect();
                if let [w, h] = n[..] {
                    (size, cell_h) = (w, h);
                }
            } else if let Some(v) = line.strip_prefix("scale = ") {
                unit = v.trim().parse().ok()?;
            } else if let Some(v) = line.strip_prefix("baseline = ") {
                baseline = v.trim().parse().ok()?;
            } else if let Some(v) = line.strip_prefix("pixel = ") {
                pixel = v.trim().parse().ok()?;
            } else if let Some(v) = line.strip_prefix("line = ") {
                line_height = v.trim().parse().ok()?;
            } else if let Some(v) = line.strip_prefix("caps = ") {
                caps = v.trim().parse().ok()?;
            } else if let Some(v) = line.strip_prefix("sharp = ") {
                smooth = v.trim() != "true";
            } else if let Some(v) = line.strip_prefix("g ") {
                let f: Vec<&str> = v.split_whitespace().collect();
                let [code, col, row, left, width, bearing, advance] = f[..] else { continue };
                let Some(ch) = u32::from_str_radix(code, 16).ok().and_then(char::from_u32) else { continue };
                let n = |s: &str| s.parse::<f32>().ok();
                let (Some(col), Some(row), Some(left), Some(width), Some(bearing), Some(advance)) =
                    (n(col), n(row), n(left), n(width), n(bearing), n(advance))
                else {
                    continue;
                };
                baked.push((ch, col as u32, row as u32, left, width, bearing, advance));
            }
        }
        if unit == 0 {
            unit = size / CELL;
        }
        if unit == 0 || size < CELL * unit || cell_h < size.min(CELL * unit) || baked.is_empty() {
            return None;
        }
        let k = unit as f32;
        let ours: rustc_hash::FxHashSet<char> = baked.iter().map(|b| b.0).collect();
        let fill: Vec<_> = unifont(hex)
            .into_iter()
            .filter(|b| !ours.contains(&b.0))
            .collect();
        let rows = (baked.len() as u32 + fill.len() as u32 + 1).div_ceil(ATLAS_COLS);
        let atlas_width = ATLAS_COLS * (size + 2);
        let atlas_height = rows * (cell_h + 2);
        let mut atlas = vec![0u8; (atlas_width * atlas_height) as usize];
        let mut glyphs = FxHashMap::default();
        let mut cell = 1;
        for &(ch, col, row, left, width, bearing, advance) in &baked {
            let (cx, cy) = cell_origin(cell, size, cell_h);
            for y in 0..cell_h {
                for x in 0..size {
                    let (px, py) = (col * size + x, row * cell_h + y);
                    if px < img.width && py < img.height {
                        atlas[((cy + y) * atlas_width + cx + x) as usize] = img.rgba[((py * img.width + px) * 4) as usize];
                    }
                }
            }
            glyphs.insert(ch, Glyph { cell, left, width: width / k, bearing: bearing / k, advance: advance / k });
            cell += 1;
        }
        // Unifont's 16 pixel glyphs scaled up, standing on the same baseline (its row 14).
        let scale = unit;
        let rise = (baseline - 14.0 * k).round().max(0.0) as u32;
        for (ch, bits, metrics) in fill {
            let (cx, cy) = cell_origin(cell, size, cell_h);
            let (mut min_x, mut max_x) = (u32::MAX, 0);
            for (y, &row) in bits.iter().enumerate() {
                for x in 0..16u32 {
                    if row & (0x8000 >> x) == 0 {
                        continue;
                    }
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let ay = rise + y as u32 * scale + sy;
                            if ay < cell_h {
                                atlas[((cy + ay) * atlas_width + cx + x * scale + sx) as usize] = 255;
                            }
                        }
                    }
                }
            }
            let (left, advance) = metrics.unwrap_or((min_x.min(255) as u8, (max_x + 2).saturating_sub(min_x).min(255) as u8));
            let g = Glyph::bitmap(cell, left, advance);
            glyphs.insert(ch, Glyph { left: g.left * scale as f32, ..g });
            cell += 1;
        }
        missing_box(&mut atlas, atlas_width, size, cell_h);
        let mono = mono_width(&glyphs);
        let fallback = Glyph { cell: 0, left: 2.0 * k, width: 12.0, bearing: 0.0, advance: 13.0 };
        Some(Font {
            glyphs,
            atlas_width,
            atlas_height,
            atlas,
            line_height,
            caps,
            pixel,
            mono,
            cell_w: size,
            cell_h,
            unit: k,
            top: 14.0 - baseline / k,
            height: cell_h as f32 / k,
            smooth,
            fallback,
        })
    }

    #[inline]
    pub fn glyph(&self, c: char) -> Glyph {
        self.glyphs.get(&c).copied().unwrap_or(self.fallback)
    }

    pub fn uv(&self, g: Glyph) -> [f32; 4] {
        let (cx, cy) = cell_origin(g.cell, self.cell_w, self.cell_h);
        let k = self.unit;
        let x0 = cx as f32 + g.left;
        let x1 = x0 + g.width * k;
        [
            x0 / self.atlas_width as f32,
            cy as f32 / self.atlas_height as f32,
            x1 / self.atlas_width as f32,
            (cy + self.cell_h) as f32 / self.atlas_height as f32,
        ]
    }
}

/// Unifont's glyphs (`.hex`): each character's rows of up to 16 pixels, and its first inked column
/// and advance where they are not taken from the ink.
fn unifont(hex: &str) -> Vec<(char, [u16; 16], Option<(u8, u8)>)> {
    let mut bitmaps = Vec::new();
    for line in hex.lines() {
        if line.starts_with('#') {
            continue;
        }
        let Some((cp, hex)) = line.split_once(':') else { continue };
        let Some(ch) = u32::from_str_radix(cp, 16).ok().and_then(char::from_u32) else { continue };
        let bpr = match hex.len() {
            32 => 1,
            64 => 2,
            _ => continue,
        };
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .filter_map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok())
            .collect();
        if bytes.len() != 16 * bpr {
            continue;
        }
        let mut rows = [0u16; 16];
        for (y, row) in rows.iter_mut().enumerate() {
            *row = if bpr == 1 { (bytes[y] as u16) << 8 } else { (bytes[y * 2] as u16) << 8 | bytes[y * 2 + 1] as u16 };
        }
        let metrics = (ch == ' ')
            .then_some((0, 4))
            .or_else(|| rows.iter().all(|&r| r == 0).then_some((0, bpr as u8 * 4)));
        bitmaps.push((ch, rows, metrics));
    }
    bitmaps
}

/// Cell 0 is drawn as a visible box, shown when a glyph is missing.
fn missing_box(atlas: &mut [u8], atlas_width: u32, size: u32, cell_h: u32) {
    let (ox, oy) = cell_origin(0, size, cell_h);
    let t = (size.min(cell_h) / CELL).max(1);
    let size = size.min(cell_h);
    for i in 0..size {
        for j in [2 * t, size - 3 * t] {
            for w in 0..t {
                atlas[((oy + j + w) * atlas_width + ox + i.clamp(2 * t, size - 3 * t)) as usize] = 255;
                atlas[((oy + i.clamp(2 * t, size - 3 * t)) * atlas_width + ox + j + w) as usize] = 255;
            }
        }
    }
}

/// Cell width for fixed-width text: the widest digit.
fn mono_width(glyphs: &FxHashMap<char, Glyph>) -> f32 {
    ('0'..='9')
        .filter_map(|c| glyphs.get(&c))
        .map(|g| g.advance)
        .fold(0.0, f32::max)
        .max(1.0)
}
