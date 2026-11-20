use glam::{DVec2, DVec3, IVec2, Vec2};

use crate::content::Content;

pub(crate) const CLOUD_CELL: f64 = 320.0;
pub(crate) const CLOUD_HEIGHT: f32 = 16.0;
pub(crate) const AUTOMATIC: u32 = u32::MAX;

/// A sampleable layer shared by simulation and rendering. Its drift follows elapsed world ticks,
/// so setting the time of day does not move the weather.
#[derive(Clone, Copy)]
pub(crate) struct CloudField {
    pub seed: u32,
    pub drift: f64,
    pub height: f64,
    pub rain: u16,
    pub thunder: u16,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct WeatherField {
    pub clouds: Option<CloudField>,
    pub forced: Option<u16>,
}

impl WeatherField {
    pub fn new(c: &Content, dim: u32, seed: u64, ticks: f64, forced: Option<u16>) -> Self {
        let d = &c.dims[dim as usize];
        let clouds = c.game.sky.get(dim as usize).filter(|_| d.has_weather());
        let Some(s) = clouds.and_then(|sky| {
            sky.iter()
                .rev()
                .find(|s| s.kind == weft_abi::sky_kind::CLOUDS)
        }) else {
            return Self { clouds: None, forced };
        };
        let seed = seed ^ weft_abi::fnv(d.name.as_bytes());
        let variants = weft_noise::hash(seed, ticks as i64 / 24000, 0, 0);
        let choose = |lightning: bool, roll: u32| {
            let candidates = || {
                d.weathers
                    .iter()
                    .copied()
                    .filter(|&id| c.game.weathers[id as usize].lightning() == lightning)
            };
            let total: u64 = candidates()
                .map(|id| c.game.weathers[id as usize].weight as u64)
                .sum();
            if total == 0 {
                return None;
            }
            let mut target = roll as u64 % total;
            for id in candidates() {
                let w = c.game.weathers[id as usize].weight as u64;
                if target < w {
                    return Some(id as u16);
                }
                target -= w;
            }
            None
        };
        let rain = choose(false, variants as u32).unwrap_or(0);
        let thunder = choose(true, (variants >> 32) as u32).unwrap_or(rain);
        Self {
            clouds: Some(CloudField {
                seed: (seed ^ (seed >> 32)) as u32,
                drift: ticks * s.drift as f64,
                height: s.height as f64,
                rain,
                thunder,
            }),
            forced,
        }
    }

    pub fn at(self, xz: DVec2) -> u16 {
        if let Some(id) = self.forced {
            return id;
        }
        self.clouds.map_or(0, |cloud| cloud.at(xz))
    }

    pub fn at_pos(self, pos: DVec3) -> u16 {
        if self.forced.is_none() && self.clouds.is_some_and(|cloud| pos.y >= cloud.height) {
            return 0;
        }
        self.at(DVec2::new(pos.x, pos.z))
    }
}

impl CloudField {
    pub fn coords(self, xz: DVec2) -> DVec2 {
        (xz + DVec2::new(self.drift, 0.0)) / CLOUD_CELL
    }

    fn at(self, xz: DVec2) -> u16 {
        let at = self.coords(xz);
        let origin = at.floor();
        let cell = origin.as_ivec2();
        let (shape, details) = cloud_hashes(self.seed, cell);
        if details >> 24 < 48 {
            return 0;
        }
        let center = cloud_center(shape);
        let bound = 0.475 - (center - 0.5).abs().max_element();
        let major = bound.min(0.34 + 0.07 * byte(details));
        let radii = Vec2::new(major, major * (0.64 + 0.30 * byte(details >> 8)));
        let offset = (at - origin).as_vec2() - center;
        let direction = cloud_axis(shape >> 30);
        let local = Vec2::new(offset.dot(direction), offset.dot(direction.perp())) / radii;
        let distance = |center: Vec2, radius: f32| (local - center).length_squared() / (radius * radius);
        let core = distance(Vec2::new(0.0, 0.08), 0.70);
        let left = distance(Vec2::new(-0.47, 0.13 + 0.14 * byte(details >> 16)), 0.42);
        let right = distance(Vec2::new(0.43, -0.07 - 0.17 * byte(shape >> 16)), 0.49);
        if core.min(left).min(right) >= 0.90 {
            return 0;
        }
        match shape & 255 {
            v if v >= 218 => self.thunder,
            v if v >= 155 => self.rain,
            _ => 0,
        }
    }
}

fn byte(n: u32) -> f32 {
    (n & 255) as f32 / 255.0
}

fn cloud_center(shape: u32) -> Vec2 {
    Vec2::new(byte(shape >> 8), byte(shape >> 16)) * 0.44 + 0.28
}

fn cloud_axis(turn: u32) -> Vec2 {
    use std::f32::consts::FRAC_1_SQRT_2 as DIAGONAL;
    [Vec2::X, Vec2::splat(DIAGONAL), Vec2::Y, Vec2::new(-DIAGONAL, DIAGONAL)][turn as usize]
}

// Match the weather cloud parameters in common.wgsl. A cloud stays inside its own cell,
// so a sample needs only one cell and neighboring weather types have a gap between them.
fn cloud_hashes(seed: u32, cell: IVec2) -> (u32, u32) {
    let h = seed ^ (cell.x as u32).wrapping_mul(0x9e3779b1) ^ (cell.y as u32).wrapping_mul(0x85ebca77);
    let mix = |v: u32| {
        let v = (v ^ (v >> 16)).wrapping_mul(0x7feb352d);
        let v = (v ^ (v >> 15)).wrapping_mul(0x846ca68b);
        v ^ (v >> 16)
    };
    (mix(h), mix(h ^ 0xa341316c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_weather_leaves_holes_and_stays_below_the_cloud_layer() {
        let cloud = CloudField { seed: 42, drift: 24.0, height: 200.0, rain: 1, thunder: 2 };
        let field = WeatherField { clouds: Some(cloud), forced: None };
        let mut seen = [false; 3];
        for z in -32..32 {
            for x in -32..32 {
                let xz = DVec2::new(x as f64 * 72.0, z as f64 * 72.0);
                let pos = DVec3::new(xz.x, 64.0, xz.y);
                let weather = field.at_pos(pos);
                seen[weather as usize] = true;
                assert_eq!(field.at_pos(DVec3::new(xz.x, cloud.height, xz.y)), 0);
                assert_eq!(WeatherField { forced: Some(1), ..field }.at_pos(pos), 1);
                assert_eq!(WeatherField { forced: Some(0), ..field }.at_pos(pos), 0);
            }
        }
        assert_eq!(seen, [true; 3]);
    }

    #[test]
    fn a_cloud_keeps_its_weather_and_leaves_a_gap_at_cell_boundaries() {
        let cloud = CloudField { seed: 42, drift: 24.0, height: 200.0, rain: 1, thunder: 2 };
        let world_at = |cell: IVec2, fraction: DVec2| (cell.as_dvec2() + fraction) * CLOUD_CELL - DVec2::new(cloud.drift, 0.0);
        let mut wet = 0;
        for x in -20..20 {
            let cell = IVec2::new(x, 2);
            let (shape, _) = cloud_hashes(cloud.seed, cell);
            let core = cloud.at(world_at(cell, cloud_center(shape).as_dvec2()));
            wet += usize::from(core != 0);
            for dz in 1..10 {
                for dx in 1..10 {
                    let other = cloud.at(world_at(cell, DVec2::new(dx as f64, dz as f64) * 0.1));
                    assert!(other == 0 || other == core);
                }
            }
            assert_eq!(cloud.at(world_at(cell, DVec2::new(0.99, 0.5))), 0);
        }
        assert!(wet > 0);
    }
}
