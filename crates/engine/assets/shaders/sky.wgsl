// The sky, drawn after the opaque world wherever nothing covers it. Every object is one entry of
// `sky`; things that fill the screen are a single triangle, bodies a quad, stars one quad per star
// made here from their number.

var<private> CORNER: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0),
);

struct VOut {
    @builtin(position) clip: vec4<f32>,
    // Screen position (-1..1) for what fills the screen, else the direction.
    @location(0) at: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) index: u32,
}

fn mat(s: Sky) -> mat3x3<f32> {
    return mat3x3(s.axes[0].xyz, s.axes[1].xyz, s.axes[2].xyz);
}

fn hash(v: u32) -> u32 {
    let s = v * 747796405u + 2891336453u;
    let w = ((s >> ((s >> 28u) + 4u)) ^ s) * 277803737u;
    return (w >> 22u) ^ w;
}

fn unit(v: u32) -> f32 {
    return f32(hash(v)) / 4294967295.0;
}

fn far_point(dir: vec3<f32>) -> vec4<f32> {
    // At depth 0, the far plane: the sky is behind everything.
    let c = scene.view_proj * vec4(dir * 100.0, 1.0);
    return vec4(c.xy, 0.0, c.w);
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let s = sky[ii];
    var out: VOut;
    out.index = ii;
    if s.ids.y == SKY_BODY {
        let c = CORNER[vi % 6u];
        let local = vec3(1.0, (0.5 - c.y) * 2.0 * s.extra.x, (c.x - 0.5) * 2.0 * s.extra.x);
        let dir = mat(s) * local;
        out.clip = far_point(dir);
        out.at = dir;
        out.uv = mix(s.uv.xy, s.uv.zw, c);
    } else if s.ids.y == SKY_STARS {
        // A random point on the sphere, a random size and turn, from the star's number alone.
        let star = vi / 6u + u32(s.uv.y) * 7919u;
        let z = unit(star * 4u) * 2.0 - 1.0;
        let a = unit(star * 4u + 1u) * 6.2831853;
        let r = sqrt(1.0 - z * z);
        let toward = vec3(r * cos(a), z, r * sin(a));
        let size = s.uv.x * (0.6 + 0.4 * unit(star * 4u + 2u)) * 2.0;
        let turn = unit(star * 4u + 3u) * 6.2831853;
        var side = cross(toward, vec3(0.0, 1.0, 0.0));
        if dot(side, side) < 0.001 {
            side = vec3(1.0, 0.0, 0.0);
        }
        side = normalize(side);
        let up = cross(side, toward);
        let c = CORNER[vi % 6u] - vec2(0.5);
        let q = vec2(c.x * cos(turn) - c.y * sin(turn), c.x * sin(turn) + c.y * cos(turn));
        let dir = mat(s) * (toward + (side * q.x + up * q.y) * size);
        out.clip = far_point(dir);
        out.at = dir;
        out.uv = CORNER[vi % 6u];
    } else {
        let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u)) * 2.0 - 1.0;
        // The overlay of a liquid goes over everything, the hand too; depth 1 is the near plane.
        out.clip = vec4(p, select(0.0, 1.0, s.ids.y == SKY_OVERLAY), 1.0);
        out.at = vec3(p, 1.0);
        out.uv = vec2(0.0);
    }
    return out;
}

// Clouds: each pixel's ray walks the picture's grid between the two planes of the layer, so one
// triangle draws them however far they reach. From outside the layer a ray crosses few cells even
// at the horizon, since it soon leaves through the other plane.
// uv: the camera's place in its cell (x, z), the cell in the picture (x, z). extra: cell size,
// thickness and height over the camera. Scene environment.yz supplies cloud fog distances.
struct CloudOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) at: vec3<f32>,
    @location(1) @interpolate(flat) index: u32,
}

struct CloudPixel {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@vertex
fn vs_clouds(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> CloudOut {
    let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u)) * 2.0 - 1.0;
    var out: CloudOut;
    out.clip = vec4(p, 0.0, 1.0);
    out.at = vec3(p, 1.0);
    out.index = ii;
    return out;
}

@fragment
fn fs_clouds(in: CloudOut) -> CloudPixel {
    let dir = normalize(mat(sky[0]) * in.at);
    let hit = trace_cloud(sky[in.index], dir, scene.environment.z, scene);
    if hit.color.a <= 0.0 {
        discard;
    }
    let clip = scene.view_proj * vec4(dir * max(hit.distance, 0.001), 1.0);
    return CloudPixel(vec4(hit.color.rgb * hit.color.a, hit.color.a), clamp(clip.z / clip.w, 0.0, 1.0));
}

fn atmosphere_limit(dir: vec3<f32>, depth: f32) -> f32 {
    let slope = (scene.view_proj * vec4(dir, 0.0)).w;
    // Infinite reverse Z, with the optional translation from camera bobbing.
    let at_origin = scene.view_proj[3];
    let limit = (at_origin.z - depth * at_origin.w) / max(depth * slope, 1e-8);
    return select(limit, min(limit, scene.params.y), scene.environment.w >= 1.0);
}

fn atmosphere_at(s: Sky, dir: vec3<f32>, depth: f32, pixel: vec2<f32>) -> vec4<f32> {
    let limit = atmosphere_limit(dir, depth);
    return resolve_rain(s, dir, limit, trace_cloud(s, dir, limit, scene), scene, pixel);
}

@fragment
fn fs_atmosphere(in: CloudOut) -> @location(0) vec4<f32> {
    let pixel = vec2<u32>(in.clip.xy);
    let depth = world_depth[pixel.y * u32(sky[0].extra.x) + pixel.x];
    return atmosphere_at(sky[in.index], normalize(mat(sky[0]) * in.at), depth, in.clip.xy);
}

fn atmosphere_sample_pixel(cell: vec2<u32>, size: vec2<u32>, stride: u32) -> vec2<u32> {
    return min(cell * stride + vec2(stride / 2u), size - vec2(1u));
}

var<workgroup> occupied_cloud_brick: atomic<u32>;
var<workgroup> keep_cloud_brick: u32;

@compute @workgroup_size(8, 8)
fn cache_cloud_density(
    @builtin(workgroup_id) brick: vec3<u32>,
    @builtin(local_invocation_id) lane: vec3<u32>,
    @builtin(local_invocation_index) index: u32,
) {
    let side = i32(scene.atmosphere_cache.x);
    let start = scene.cloud_cache.xy + vec2<i32>(brick.xy) * CLOUD_BRICK;
    let previous = scene.cloud_cache.zw;
    // The toroidal buffer leaves overlapping world bricks at the same addresses.
    if index == 0u {
        keep_cloud_brick = u32(previous.x != -2147483647 && all(start >= previous) && all(start < previous + vec2(side)));
        atomicStore(&occupied_cloud_brick, 0u);
    }
    if workgroupUniformLoad(&keep_cloud_brick) != 0u {
        return;
    }
    let cell = start + vec2<i32>(lane.xy);
    // Floor division is required on the negative side of the weather origin.
    let origin = (cell - (cell % 24 + 24) % 24) / 24;
    let local_cell = vec2<f32>(cell - origin * 24);
    let band = i32(brick.z);
    let packed = pack2x16float(cloud_cell_density(scene.cloud.ids.z, origin, local_cell, band));
    cloud_cells_out[cloud_cache_index(cell, band, side)] = packed;
    if (packed & 0xffffu) != 0u {
        atomicOr(&occupied_cloud_brick, 1u);
    }
    workgroupBarrier();
    if index == 0u {
        cloud_cells_out[cloud_brick_index(start, band, side)] = atomicLoad(&occupied_cloud_brick);
    }
}

fn atmosphere_ray(pixel: vec2<f32>, size: vec2<u32>) -> vec3<f32> {
    let ndc = (pixel / vec2<f32>(size)) * vec2(2.0, -2.0) + vec2(-1.0, 1.0);
    return normalize(mat(sky[0]) * vec3(ndc, 1.0));
}

@compute @workgroup_size(8, 8)
fn cache_rain_depth(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = scene.atmosphere_grid.xy;
    let grid = (size + (RAIN_STRIDE - 1u)) / RAIN_STRIDE;
    if any(id.xy >= grid) {
        return;
    }
    let pixel = vec2<f32>(atmosphere_sample_pixel(id.xy, size, RAIN_STRIDE)) + 0.5;
    let dir = atmosphere_ray(pixel, size);
    let s = scene.cloud;
    let reach = min(1536.0, scene.environment.z);
    let ceiling = s.extra.z + s.extra.y * 0.27 + 18.0;
    var enter = 36.0;
    var leave = select(reach, min(reach, scene.params.y), scene.environment.w >= 1.0);
    if dir.y > 0.0 {
        leave = min(leave, ceiling / dir.y);
    } else if dir.y < 0.0 {
        enter = max(enter, ceiling / dir.y);
    } else if ceiling < 0.0 {
        leave = 0.0;
    }
    let rainbow = rainbow_radiance(s, dir);
    let offset = (id.y * grid.x + id.x) * (RAIN_SLICES + 1u);
    var rain = vec4(0.0);
    rain_depth_out[offset] = vec2(0u);
    for (var layer = 1u; layer <= RAIN_SLICES; layer += 1u) {
        let start = f32(layer - 1u) * RAIN_SLICE;
        let end = min(start + RAIN_SLICE, leave);
        for (var t = start; t < end && rain.a < 0.88; t += 32.0) {
            let a = max(t, enter);
            let b = min(t + 32.0, end);
            if b > a {
                let medium = rain_medium(s, dir, (a + b) * 0.5, reach, rainbow, scene);
                let absorption = (1.0 - rain.a) * (1.0 - exp(-medium.a * (b - a)));
                rain += vec4(medium.rgb, 1.0) * absorption;
            }
        }
        rain_depth_out[offset + layer] = vec2(pack2x16float(rain.rg), pack2x16float(rain.ba));
    }
}

fn distant_cloud_history(dir: vec3<f32>, plane: f32, grid: vec2<u32>, stride: u32, size: vec2<u32>) -> CloudHistory {
    let position = dir * plane + scene.atmosphere_shift.xyz;
    let clip = scene.atmosphere_previous * vec4(position, 1.0);
    let pixel = (clip.xy / max(clip.w, 0.001) * vec2(0.5, -0.5) + 0.5) * vec2<f32>(size);
    if clip.w <= 0.0 || any(pixel < vec2(0.0)) || any(pixel >= vec2<f32>(size)) {
        return CloudHistory(0u, 0u, 0u);
    }
    let cell = min(vec2<u32>(pixel) / stride, grid - 1u);
    let center = vec2<f32>(atmosphere_sample_pixel(cell, size, stride)) + 0.5;
    // Avoid dragging the previous silhouette across the current sample footprint.
    if any(abs(pixel - center) > vec2(f32(stride) * 0.38)) {
        return CloudHistory(0u, 0u, 0u);
    }
    return atmosphere_history[cell.y * grid.x + cell.x];
}

// Trace a sparse grid of rays. The two packed halves hold premultiplied RGB and opacity.
@compute @workgroup_size(8, 8)
fn sample_atmosphere(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = scene.atmosphere_grid.xy;
    let stride = scene.atmosphere_grid.z;
    let cells = (size + vec2(stride - 1u)) >> vec2(firstTrailingBit(stride));
    if any(id.xy >= cells) {
        return;
    }
    let p = atmosphere_sample_pixel(id.xy, size, stride);
    let pixel = vec2<f32>(p) + 0.5;
    let dir = atmosphere_ray(pixel, size);
    let depth = world_depth[p.y * size.x + p.x];
    let limit = atmosphere_limit(dir, depth);
    let s = scene.cloud;
    let a = s.extra.z / select(dir.y, 1e-8, dir.y == 0.0);
    let b = (s.extra.z + s.extra.y) / select(dir.y, 1e-8, dir.y == 0.0);
    let distant = stride == 4u && dir.y != 0.0 && min(a, b) > max(1536.0, scene.cloud_lod.y) && max(a, b) < min(limit, scene.environment.z);
    let phase = scene.atmosphere_cache.w;
    var age = 0u;
    var cloud: CloudHit;
    if distant && phase != 0u && (id.x + id.y + phase) % 3u != 0u {
        let history = distant_cloud_history(dir, (a + b) * 0.5, cells, stride, size);
        if history.history_age > 0u && history.history_age < 3u {
            let color = vec4(unpack2x16float(history.cloud_rg), unpack2x16float(history.cloud_ba));
            // The whole distant slab sits beyond the rain reach.
            cloud = CloudHit(color, 0.0, min(a, b));
            age = history.history_age + 1u;
        }
    }
    if age == 0u {
        cloud = trace_cloud(s, dir, limit, scene);
        age = select(0u, 1u, distant);
    }
    let c = resolve_rain(s, dir, limit, cloud, scene, pixel);
    let index = id.y * cells.x + id.x;
    atmosphere_out[index] = AtmosphereSample(pack2x16float(c.rg), pack2x16float(c.ba), depth);
    if stride == 4u {
        var remembered = CloudHistory(0u, 0u, 0u);
        if age != 0u {
            remembered = CloudHistory(pack2x16float(cloud.color.rg), pack2x16float(cloud.color.ba), age);
        }
        atmosphere_history_out[index] = remembered;
    }
}

@fragment
fn fs_resolve_atmosphere(in: CloudOut) -> @location(0) vec4<f32> {
    let grid = scene.atmosphere_grid;
    let pixel = vec2<u32>(in.clip.xy);
    let depth = world_depth[pixel.y * grid.x + pixel.x];
    let taps = atmosphere_filter(in.clip.xy, grid);
    let i = taps.index;
    let result = filter_atmosphere(scene, depth, taps, false, atmosphere[i.x], atmosphere[i.y], atmosphere[i.z], atmosphere[i.w]);
    if result.a >= 0.0 {
        return result;
    }
    return atmosphere_at(sky[in.index], normalize(mat(sky[0]) * in.at), depth, in.clip.xy);
}

// Vanilla's sky is a disc 16 blocks above the eye fading into fog with distance.
fn gradient(dir: vec3<f32>, s: Sky) -> vec4<f32> {
    if dir.y <= 0.0 {
        // Under the horizon: fog, and from below sea level the dark of vanilla's void.
        let dark = s.uv.x * smoothstep(0.0, 0.08, -dir.y);
        return vec4(scene.fog_color.rgb * (1.0 - dark) * s.color.a, s.color.a);
    }
    return vec4(mix(s.color.rgb, scene.fog_color.rgb, clamp(0.0625 / dir.y, 0.0, 1.0)) * s.color.a, s.color.a);
}

// Vanilla's sunset fan: an oval lying on the horizon on the side of the body.
fn glow(dir: vec3<f32>, s: Sky) -> vec4<f32> {
    let toward = dot(dir, s.axes[0].xyz);
    if toward <= 0.0 {
        return vec4(0.0);
    }
    let x = dot(dir, s.axes[2].xyz) / toward / 1.2;
    let y = dir.y / toward / max(0.4 * s.color.a, 0.001);
    let a = s.color.a * max(0.0, 1.0 - sqrt(x * x + y * y));
    return vec4(s.color.rgb, a);
}

// Panorama: longitude and latitude. Box: three by two faces; first row down, up, south; second
// west, north, east.
fn dome_uv(l: vec3<f32>, s: Sky) -> vec2<f32> {
    if s.uv.x == 0.0 {
        return vec2(atan2(l.x, -l.z) / 6.2831853 + 0.5, 0.5 - asin(clamp(l.y, -1.0, 1.0)) / 3.1415927);
    }
    let m = abs(l);
    var face: vec2<f32>;
    var cell: vec2<f32>;
    if m.y >= m.x && m.y >= m.z {
        face = vec2(l.x, select(-l.z, l.z, l.y > 0.0)) / m.y;
        cell = vec2(select(0.0, 1.0, l.y > 0.0), 0.0);
    } else if m.z >= m.x {
        face = vec2(select(l.x, -l.x, l.z > 0.0), -l.y) / m.z;
        cell = select(vec2(1.0, 1.0), vec2(2.0, 0.0), l.z > 0.0);
    } else {
        face = vec2(select(-l.z, l.z, l.x > 0.0), -l.y) / m.x;
        cell = select(vec2(0.0, 1.0), vec2(2.0, 1.0), l.x > 0.0);
    }
    return (cell + face * 0.5 + 0.5) / vec2(3.0, 2.0);
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let s = sky[in.index];
    if s.ids.y == SKY_OVERLAY {
        // Vanilla's: four tiles across the screen, sliding with the look. A texture with
        // animation frames shows its first.
        if s.extra.y != 0.0 && in.at.y > -0.25 {
            return vec4(0.0);
        }
        let tile = fract((in.at.xy * vec2(0.5, -0.5) + 0.5) * s.extra.x + s.uv.xy) * s.uv.zw + vec2(0.0, s.extra.z);
        let c = textureSampleLevel(textures[s.ids.x], nearest_clamp, tile, 0.0) * s.color;
        return vec4(c.rgb * c.a, c.a);
    }
    var dir = in.at;
    if s.ids.y == SKY_GRADIENT || s.ids.y == SKY_GLOW || s.ids.y == SKY_DOME {
        dir = mat(sky[0]) * in.at;
    }
    dir = normalize(dir);
    let mist = scene.environment.w;
    if s.ids.y == SKY_GRADIENT {
        let c = gradient(dir, s);
        return vec4(mix(c.rgb, scene.fog_color.rgb * c.a, mist), c.a);
    }
    if s.ids.y == SKY_GLOW {
        let c = glow(dir, s);
        let a = c.a * (1.0 - mist);
        return vec4(c.rgb * a, a);
    }
    let local = dir * mat(s);
    var uv = in.uv;
    var albedo = vec4(1.0);
    if s.ids.y == SKY_DOME {
        uv = dome_uv(local, s);
        if s.ids.x != 0xffffffffu {
            albedo = textureSampleLevel(textures[s.ids.x], linear_clamp, uv, 0.0);
        }
    } else if s.ids.x != 0xffffffffu {
        albedo = textureSampleLevel(textures[s.ids.x], nearest_clamp, uv, 0.0);
    }
    let c = sky_shade(s.ids.z, SkyPixel(dir, local, uv, albedo, vec4(s.color.rgb, 1.0), scene.params.z, scene.params.w));
    let a = c.a * s.color.a * (1.0 - mist);
    return vec4(c.rgb * a, select(a, 0.0, s.ids.w != 0u));
}
