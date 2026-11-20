// Shared cache readers for the sky and translucent world.
fn cloud_cache_index(cell: vec2<i32>, band: i32, side: i32) -> u32 {
    let p = cell & vec2(side - 1);
    return u32((band * side + p.y) * side + p.x);
}

fn cloud_brick_index(cell: vec2<i32>, band: i32, side: i32) -> u32 {
    let count = side / CLOUD_BRICK;
    let p = (cell & vec2(side - 1)) / CLOUD_BRICK;
    return u32(5 * side * side + (band * count + p.y) * count + p.x);
}

fn cloud_cache_value(view: Scene, cell: vec2<i32>, band: i32) -> u32 {
    let side = i32(view.atmosphere_cache.x);
    let relative = cell - view.cloud_cache.xy;
    if any(relative < vec2(0)) || any(relative >= vec2(side)) {
        return CLOUD_UNCACHED;
    }
    if cloud_cells[cloud_brick_index(cell, band, side)] == 0u {
        return CLOUD_EMPTY_BRICK;
    }
    return cloud_cells[cloud_cache_index(cell, band, side)];
}

const RAIN_STRIDE: u32 = 8u;
const RAIN_SLICES: u32 = 16u;
const RAIN_SLICE: f32 = 96.0;

fn rain_at_cell(cell: u32, range: f32) -> vec4<f32> {
    let z = clamp(min(range / RAIN_SLICE, (range - 36.0) / (RAIN_SLICE - 36.0)), 0.0, f32(RAIN_SLICES));
    let layer = u32(z);
    let offset = cell * (RAIN_SLICES + 1u);
    let a = rain_depth[offset + layer];
    let b = rain_depth[offset + min(layer + 1u, RAIN_SLICES)];
    let first = vec4(unpack2x16float(a.x), unpack2x16float(a.y));
    let second = vec4(unpack2x16float(b.x), unpack2x16float(b.y));
    return mix(first, second, fract(z));
}

fn rain_at(view: Scene, pixel: vec2<f32>, range: f32) -> vec4<f32> {
    if range <= 36.0 {
        return vec4(0.0);
    }
    let taps = atmosphere_filter(pixel, vec4(view.atmosphere_grid.xy, RAIN_STRIDE, view.atmosphere_cache.z));
    let i = taps.index;
    return mat4x4(rain_at_cell(i.x, range), rain_at_cell(i.y, range), rain_at_cell(i.z, range), rain_at_cell(i.w, range)) * taps.weight;
}

fn resolve_rain(s: Sky, dir: vec3<f32>, limit: f32, cloud: CloudHit, view: Scene, pixel: vec2<f32>) -> vec4<f32> {
    if view.atmosphere_cache.y == 0u {
        let color = composite_rain_veil(s, dir, limit, cloud, view);
        return vec4(color.rgb * color.a, color.a);
    }
    let front = min(cloud.optical_distance, limit);
    if cloud.color.a >= CLOUD_OPAQUE {
        let rain = rain_at(view, pixel, front);
        return rain + vec4(cloud.color.rgb * cloud.color.a, cloud.color.a) * (1.0 - rain.a);
    }
    let rain = rain_at(view, pixel, limit);
    if cloud.color.a <= 0.0 {
        return rain;
    }
    let before_cloud = rain_at(view, pixel, front);
    let color = mix(rain.rgb, before_cloud.rgb, cloud.color.a) + cloud.color.rgb * cloud.color.a * (1.0 - before_cloud.a);
    return vec4(color, cloud.color.a + (1.0 - cloud.color.a) * rain.a);
}
