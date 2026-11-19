//! Low vegetation contributes to the surface color without more geometry.
use std::hash::{DefaultHasher, Hash};

use glam::{IVec2, Vec3};
use rustc_hash::FxHashMap;
use weft_abi::{Callback, ModId, RenderLayer, SurfaceCoverArgs, SurfaceCoverSample};

use super::{CELLS, Columns, GRID, TileKey, corner_y, sample_offset};
use crate::client::textures::GameTextures;
use crate::client::{looks, mesher};
use crate::content::Content;

pub(super) type Colors = [[Option<u16>; GRID]; GRID];

#[derive(Clone, Copy)]
struct Material {
    rgb: Vec3,
    opacity: f32,
    tint: u8,
}

impl Material {
    fn color(self, c: &Content, biome: u16) -> Vec3 {
        let tint = c.kind_color(self.tint, Some(biome));
        self.rgb * Vec3::from_array([16, 8, 0].map(|s| ((tint >> s) & 255) as f32 / 255.0))
    }
}

pub(super) struct Cover {
    hooks: Vec<(ModId, Callback)>,
    ground: FxHashMap<u32, Material>,
    plants: Vec<Option<Material>>,
}

impl Cover {
    pub(super) fn new(
        c: &Content,
        dim: u32,
        faces: &[[(u32, u32); 6]],
        textures: &GameTextures,
        signature: &mut DefaultHasher,
    ) -> Option<Self> {
        let def = &c.dims[dim as usize];
        let hooks: Vec<_> = def
            .features
            .iter()
            .filter_map(|f| f.surface_cover)
            .collect();
        if hooks.is_empty() {
            return None;
        }
        let terrain = def.terrain.as_ref()?;
        // A callback and its data can change even if the terrain description is identical.
        for m in &c.mods {
            (&m.id, &m.version, m.revision).hash(signature);
        }
        for biome in &c.biomes {
            biome.tints.hash(signature);
        }
        let mut cache = FxHashMap::default();
        let mut material = |(tex, tint)| {
            let (rgb, opacity) = *cache.entry(tex).or_insert_with(|| {
                let pixels = textures.impostor_texels(tex);
                pixels.hash(signature);
                let mut rgb = Vec3::ZERO;
                let mut alpha = 0.0;
                for [r, g, b, a] in pixels {
                    rgb += Vec3::new(r as f32, g as f32, b as f32) * a as f32;
                    alpha += a as f32;
                }
                (rgb / (255.0 * alpha.max(1.0)), alpha / (16.0 * 255.0))
            });
            Material { rgb, opacity, tint: tint as u8 }
        };
        let mut ground = FxHashMap::default();
        for state in std::iter::once(terrain.stone).chain(
            terrain
                .rules
                .iter()
                .flat_map(|r| [r.top, r.deep_top.map(|v| v.0)].into_iter().flatten()),
        ) {
            ground
                .entry(state)
                .or_insert_with(|| material(faces[state as usize][3]));
        }
        let looks = looks::current();
        let plants = c
            .states
            .iter()
            .enumerate()
            .map(|(id, state)| {
                if state.layer != RenderLayer::Cutout || state.solid() {
                    return None;
                }
                // For a tall plant use the canopy, whose texture can differ from its stem.
                let model = looks.drawn(id as u32, state);
                let (tex, dir) = model
                    .boxes
                    .iter()
                    .flat_map(|b| {
                        (0..6).filter_map(move |dir| {
                            mesher::box_face(b, dir)?;
                            Some((b.max[1], b.textures[dir], dir))
                        })
                    })
                    .max_by(|a, b| a.0.total_cmp(&b.0))
                    .map(|(_, tex, dir)| (tex, dir))?;
                let first = mesher::face_layers(c, state, tex & crate::gpu_layout::face::TEX_MASK, dir)[0];
                Some(material((first.tex, first.tint as u32)))
            })
            .collect();
        Some(Self { hooks, ground, plants })
    }

    pub(super) fn colors(&self, c: &Content, seed: u64, key: TileKey, words: &[u32], columns: &Columns) -> Colors {
        let mut colors = [[None; GRID]; GRID];
        let s = 1 << key.level;
        let side = s.min(2);
        let n = (side * side) as usize;
        let origin = key.origin();
        let mut samples = Vec::new();
        let mut cells = Vec::new();
        for j in 0..CELLS {
            for i in 0..CELLS {
                let k = sample_offset(i, j);
                let state = words[k] >> 16;
                let Some(&ground) = self.ground.get(&state) else { continue };
                let p = origin + IVec2::new(i * s, j * s);
                if words[k + 2] != 0 {
                    continue;
                }
                let y = corner_y(words, i, j);
                let biome = words[k + 1] as u16;
                cells.push((i, j, ground.color(c, biome)));
                let stride = s / side;
                for dz in 0..side {
                    for dx in 0..side {
                        let x = p.x + dx * stride;
                        let z = p.y + dz * stride;
                        let h = weft_noise::hash(seed, x as i64, 0, z as i64);
                        let bx = x + (h as i32 & (stride - 1));
                        let bz = z + ((h >> 32) as i32 & (stride - 1));
                        let edited = columns
                            .get(&IVec2::new(bx, bz))
                            .is_some_and(|col| col.top != 0 || !col.spans.is_empty());
                        let ground = if edited { 0 } else { state };
                        samples.push(SurfaceCoverSample { surface: [bx, y, bz].into(), biome: biome as u32, ground, plant: 0 });
                    }
                }
            }
        }
        if samples.is_empty() {
            return colors;
        }
        let mut args = SurfaceCoverArgs { seed, samples: samples.as_mut_ptr(), len: samples.len() };
        for &(owner, cb) in &self.hooks {
            crate::host::invoke(owner, cb, &mut args);
        }
        for ((i, j, ground), samples) in cells.into_iter().zip(samples.chunks_exact(n)) {
            let mut delta = Vec3::ZERO;
            for sample in samples {
                if let Some(plant) = self.plants.get(sample.plant as usize).copied().flatten() {
                    // A crossed plant shows more than one face from an oblique view.
                    let cover = (plant.opacity * 1.6).min(0.8);
                    delta += (plant.color(c, sample.biome as u16) - ground) * cover;
                }
            }
            if delta != Vec3::ZERO {
                let rgb = (ground + delta / n as f32) * Vec3::new(31.0, 63.0, 31.0);
                let rgb = rgb.round().as_uvec3();
                colors[i as usize + 1][j as usize + 1] = Some(((rgb.x << 11) | (rgb.y << 5) | rgb.z) as u16);
            }
        }
        colors
    }
}
