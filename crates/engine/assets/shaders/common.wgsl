// Shared by the engine's shaders: put in front of each one (shader packs' too).

// The uv of each face corner (`CORNERS`, from the prelude), the two triangles of a face, and
// vanilla's face shading per direction (-X +X -Y +Y -Z +Z).
var<private> UVS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(vec2(0.0, 1.0), vec2(1.0, 1.0), vec2(1.0, 0.0), vec2(0.0, 0.0));
var<private> TRI: array<u32, 6> = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u);
var<private> SHADE: array<f32, 6> = array<f32, 6>(0.6, 0.6, 0.5, 1.0, 0.8, 0.8);

// `SHADE` blended for any unit normal.
fn normal_shade(n: vec3<f32>) -> f32 {
    return n.x * n.x * 0.6 + n.z * n.z * 0.8 + n.y * n.y * select(0.5, 1.0, n.y > 0.0);
}

// How far into the fog a point at `dist` is, 0..1.
fn fog_amount(dist: f32, start: f32, end: f32) -> f32 {
    if dist <= start {
        return 0.0;
    }
    if dist >= end {
        return 1.0;
    }
    return (dist - start) / (end - start);
}

fn cloud_mix(bits: u32) -> u32 {
    let a = (bits ^ (bits >> 16u)) * 0x7feb352du;
    let b = (a ^ (a >> 15u)) * 0x846ca68bu;
    return b ^ (b >> 16u);
}

fn cloud_byte(n: u32) -> f32 {
    return f32(n & 255u) / 255.0;
}

fn texture_cloud_at(s: Sky, cell: vec2<f32>, dims: vec2<f32>) -> bool {
    let at = fract((cell + s.uv.zw + vec2(0.5)) / dims);
    return textureSampleLevel(textures[s.ids.x], nearest_clamp, at, 0.0).a >= 0.5;
}

struct CloudHit {
    color: vec4<f32>,
    distance: f32,
}

fn cloud_lobe(at: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    let offset = (at - center) / radius;
    return dot(offset, offset);
}

// Match the cell layout in weather.rs. Intersect the low envelope before sampling density.
fn cloud_volume(s: Sky, dir: vec3<f32>, cell: vec2<f32>, start: f32, end: f32, view: Scene) -> CloudHit {
    let grid = vec2<i32>(cell) + vec2<i32>(s.uv.zw);
    let h = s.ids.z ^ (bitcast<u32>(grid.x) * 0x9e3779b1u) ^ (bitcast<u32>(grid.y) * 0x85ebca77u);
    let details = cloud_mix(h ^ 0xa341316cu);
    if details >> 24u < 48u {
        return CloudHit(vec4(0.0), 0.0);
    }
    let shape = cloud_mix(h);
    let center = vec2(cloud_byte(shape >> 8u), cloud_byte(shape >> 16u)) * 0.44 + 0.28;
    let edge = abs(center - 0.5);
    let major = min(0.475 - max(edge.x, edge.y), 0.34 + 0.07 * cloud_byte(details));
    let radii = vec2(major, major * (0.64 + 0.30 * cloud_byte(details >> 8u)));
    let diagonal = 0.7071067812;
    let axes = array(vec2(1.0, 0.0), vec2(diagonal), vec2(0.0, 1.0), vec2(-diagonal, diagonal));
    let axis = axes[shape >> 30u];
    let across = vec2(-axis.y, axis.x);
    let roll = shape & 255u;
    let half_height = select(5.0, select(6.0, 8.0, roll >= 218u), roll >= 155u);
    let offset = s.uv.xy - cell - center + dir.xz * (start / s.extra.x);
    let p = vec3(vec2(dot(offset, axis), dot(offset, across)) / radii, (dir.y * start - s.extra.z) / half_height - 1.0);
    let ray = vec3(vec2(dot(dir.xz, axis), dot(dir.xz, across)) / (radii * s.extra.x), dir.y / half_height);
    // The envelope is looser vertically so the shoulders keep their rounded tops.
    let bound = vec3(1.0, 1.0, 0.67);
    let origin = p * bound;
    let direction = ray * bound;
    let inv_ray2 = 1.0 / dot(direction, direction);
    let mid = -dot(origin, direction) * inv_ray2;
    let closest = origin + direction * mid;
    let depth = (1.0 - dot(closest, closest)) * inv_ray2;
    if depth <= 0.0 {
        return CloudHit(vec4(0.0), 0.0);
    }
    let radius = sqrt(depth);
    let entry = max(0.0, mid - radius);
    let exit = min(end - start, mid + radius);
    if entry >= exit {
        return CloudHit(vec4(0.0), 0.0);
    }
    var base = s.color.rgb;
    if s.ids.w != 0u && roll >= 155u {
        base = select(s.axes[0].xyz, s.axes[1].xyz, roll >= 218u);
    }
    let count = select(8u, 4u, start + entry > 900.0);
    let stride = (exit - entry) / f32(count);
    let phase = cloud_byte(details >> 16u) * 6.2831853;
    let left = vec2(-0.47, 0.13 + 0.14 * cloud_byte(details >> 16u));
    let right = vec2(0.43, -0.07 - 0.17 * cloud_byte(shape >> 16u));
    var light = 0.0;
    var opacity = 0.0;
    var front = entry;
    for (var i = 0u; i < count; i++) {
        let q = p + ray * (entry + (f32(i) + 0.5) * stride);
        let core = cloud_lobe(q.xy, vec2(0.0, 0.08), 0.70);
        let body = min(core, min(cloud_lobe(q.xy, left, 0.42), cloud_lobe(q.xy, right, 0.49)));
        let interior = 1.0 - body - q.z * q.z;
        let folds = sin(12.0 * q.x + 7.0 * q.y + phase) * 0.65 + sin(9.0 * q.y - 4.0 * q.x - phase) * 0.35;
        let density = smoothstep(0.0, 0.49, interior * (1.0 + folds * 0.24));
        let absorbed = (1.0 - opacity) * (1.0 - exp(-density * stride * 0.29));
        if opacity == 0.0 && absorbed > 0.0 {
            front = entry + f32(i) * stride;
        }
        let shade = clamp(0.88 + q.z * 0.26 + folds * 0.07 + q.x * 0.08, 0.50, 1.12);
        light += absorbed * shade;
        opacity += absorbed;
    }
    let distance = start + front;
    let fog = fog_amount(distance, view.environment.y, view.environment.z);
    let alpha = opacity * s.color.a * (1.0 - smoothstep(0.6, 1.0, fog));
    let color = mix(base * (light / max(opacity, 0.00001)), view.fog_color.rgb, smoothstep(0.0, 1.0, fog));
    return CloudHit(vec4(color, alpha), distance);
}

// Trace up to the first cloud surface, stopping at the fragment when testing world geometry.
fn trace_cloud(s: Sky, dir: vec3<f32>, limit: f32, view: Scene) -> CloudHit {
    let size = s.extra.x;
    let low = s.extra.z;
    let reach = view.environment.z;
    let high = low + s.extra.y;
    var near = 0.0;
    var far = min(reach, limit);
    if abs(dir.y) > 0.00001 {
        near = max(near, min(low / dir.y, high / dir.y));
        far = min(far, max(low / dir.y, high / dir.y));
    } else if low > 0.0 || high < 0.0 {
        return CloudHit(vec4(0.0), 0.0);
    }
    if near >= far {
        return CloudHit(vec4(0.0), 0.0);
    }
    let dims = vec2<f32>(f32(s.ids.z), f32(s.ids.w));
    let start = s.uv.xy + dir.xz * near / size;
    var cell = floor(start);
    let turn = sign(dir.xz);
    let delta = size / max(abs(dir.xz), vec2(0.000001));
    var next = near + select(start - cell, cell + vec2(1.0) - start, dir.xz > vec2(0.0)) * delta;
    var t = near;
    // 0: came in through a plane, 1 and 2: through a side along x or z.
    var side = 0u;
    // From inside a cloud, it isn't what is seen.
    let procedural = s.ids.x == 0xffffffffu;
    var within = !procedural && near == 0.0 && texture_cloud_at(s, cell, dims);
    var hit = false;
    let steps = u32(ceil((far - near) / size * (abs(dir.x) + abs(dir.z)))) + 2u;
    for (var i = 0u; i < steps; i++) {
        if procedural {
            let volume = cloud_volume(s, dir, cell, t, min(far, min(next.x, next.y)), view);
            if volume.color.a > 0.003 {
                return volume;
            }
        } else {
            let filled = texture_cloud_at(s, cell, dims);
            if filled && !within {
                hit = true;
                break;
            }
            within = within && filled;
        }
        if next.x < next.y {
            t = next.x;
            next.x += delta.x;
            cell.x += turn.x;
            side = 1u;
        } else {
            t = next.y;
            next.y += delta.y;
            cell.y += turn.y;
            side = 2u;
        }
        if t >= far {
            break;
        }
    }
    if !hit {
        return CloudHit(vec4(0.0), 0.0);
    }
    // Lit from above, like blocks.
    var shade = select(0.9, 0.8, side == 1u);
    if side == 0u {
        shade = select(1.0, 0.7, dir.y > 0.0);
    }
    let fog = fog_amount(t, view.environment.y, reach);
    let color = mix(s.color.rgb * shade, view.fog_color.rgb, smoothstep(0.0, 1.0, fog));
    let alpha = s.color.a * (1.0 - smoothstep(0.6, 1.0, fog));
    return CloudHit(vec4(color, alpha), t);
}

// The cloud has already been blended into the background. Tinting a fragment behind it by the
// same cloud color and opacity gives the same result as blending the cloud after that fragment.
fn through_cloud(c: vec4<f32>, rel: vec3<f32>, view: Scene) -> vec4<f32> {
    let s = view.cloud;
    let low = s.extra.z;
    if s.color.a <= 0.0 || max(0.0, rel.y) < low || min(0.0, rel.y) > low + s.extra.y {
        return c;
    }
    let dist = length(rel);
    let cloud = trace_cloud(s, rel / max(dist, 0.000001), dist, view).color;
    return vec4(mix(c.rgb, cloud.rgb, cloud.a), c.a);
}

fn brightness_curve(level: f32) -> f32 {
    return level / (4.0 - 3.0 * level);
}

// Light as the world stores it, a byte each for sky, red, green, blue (0..15), to 0..1.
fn unpack_light(l: u32) -> vec4<f32> {
    let v = vec4<f32>(f32(l & 255u), f32((l >> 8u) & 255u), f32((l >> 16u) & 255u), f32(l >> 24u));
    return min(v, vec4(15.0)) / 15.0;
}

// Vanilla 26.1's lightmap: dim ambient, sky light that turns blue and weaker at night, warm block
// light that whitens towards both ends, then the Brightness setting bends it up keeping the hue.
// `day` daylight 0..1, `ambient` the dimension's floor 0..1, `bright` the Brightness setting,
// `night` night vision 0..1: lit everywhere as under the open sky at noon, lamps adding their
// warmth on top (not vanilla's raising of each color to full, which makes a torch at night look
// like a dark patch).
fn lightmap(light: vec4<f32>, day: f32, ambient: f32, bright: f32, night: f32, sunlight: f32) -> vec3<f32> {
    let d = clamp((mix(day, 1.0, night) - 0.12) / 0.88, 0.0, 1.0);
    let sky = brightness_curve(mix(light.x, 1.0, night)) * mix(0.24, 1.0, d) * mix(sunlight, 1.0, night);
    var c = vec3(ambient) + mix(vec3(0.478, 0.478, 1.0), vec3(1.0), d) * sky;
    let block = light.yzw;
    if any(block > vec3(0.0)) {
        // Vanilla's warmth of block light: yellower the dimmer it is, white at full, by the
        // level of its brightest channel so a colored lamp only grows brighter toward it.
        let level = max(max(block.x, block.y), block.z);
        let tint = vec3(1.0, (level * 0.6 + 0.4) * 0.6 + 0.4, level * level * 0.6 + 0.4);
        // Added to the sky's, each channel on its own: a lamp only ever brightens, and a
        // colored one shows by day too.
        c += min(tint * vec3(brightness_curve(block.x), brightness_curve(block.y), brightness_curve(block.z)) * 1.4, vec3(0.85));
    }
    // Light adds up past what a screen shows: what is bright bends toward full instead of
    // being cut there, so sun and lamp together are brighter than the sun alone.
    let knee = 0.8;
    c = select(c, knee + (1.0 - knee) * (1.0 - exp(-(c - knee) / (1.0 - knee))), c > vec3(knee));
    c = clamp(c, vec3(0.0), vec3(1.0));
    let m = max(max(c.x, c.y), c.z);
    if m > 1e-4 {
        let one_minus_m = 1.0 - m;
        let m2 = one_minus_m * one_minus_m;
        let m4 = m2 * m2;
        let lifted = c * ((1.0 - m4) / m);
        return mix(c, lifted, bright);
    }
    return c;
}

// What snippets get (`shaders.rs`), here so a shader pack replacing an engine shader keeps them.

// What a block shader (`shaders/block/<name>.wgsl`, `fn shade(p: BlockPixel) -> vec4<f32>`) gets;
// it returns the lit color before fog.
struct BlockPixel {
    // 0..1 across the face.
    uv: vec2<f32>,
    // The texel (with its overlay) and the light on it (shading, AO, tint, glow), both linear.
    albedo: vec4<f32>,
    light: vec3<f32>,
    // Block coordinates of the pixel (in a moving structure, the structure's own).
    pos: vec3<f32>,
    time: f32,
}

// What an entity shader (`fn shade(p: EntityPixel) -> vec4<f32>`) gets; it returns the lit
// color, alpha below 0.5 cuts the pixel out.
struct EntityPixel {
    uv: vec2<f32>,
    // Skin texel and the light on it (shading, hurt flash and glow included), linear.
    albedo: vec4<f32>,
    light: vec3<f32>,
    time: f32,
}

// What a text effect (`shaders/fx/<name>.wgsl`, `fn effect(g: Glyph) -> Glyph`) may change.
struct Glyph {
    // Added to this corner of the glyph, physical pixels.
    offset: vec2<f32>,
    // Linear color and alpha.
    color: vec4<f32>,
    time: f32,
    // Number of the glyph in its text, for waves running along it.
    index: f32,
    // Which corner, 0..1 in each axis.
    corner: vec2<f32>,
    // Physical pixels per GUI pixel.
    scale: f32,
    // The dark copy drawn under the text.
    shadow: bool,
}

// A themed panel filled by a shader (`shaders/gui/<name>.wgsl`, `fn fill(p: GuiPixel) -> vec4<f32>`).
struct GuiPixel {
    // 0..1 across the panel.
    uv: vec2<f32>,
    // Panel size in physical pixels.
    size: vec2<f32>,
    // The theme color, linear.
    color: vec4<f32>,
    time: f32,
}

// What a screen effect (`shaders/post/<name>.wgsl`, `fn apply(p: PostPixel) -> vec4<f32>`) gets.
struct PostPixel {
    // 0..1 across the screen, top left first.
    uv: vec2<f32>,
    // The image so far at `uv`, linear.
    color: vec4<f32>,
    // 0..1 as the mod set it, for fading in and out.
    strength: f32,
    time: f32,
    // Pixels.
    size: vec2<f32>,
    scene: u32,
}

// What a sky shader (`shaders/sky/<name>.wgsl`, `fn shade(p: SkyPixel) -> vec4<f32>`) gets; it
// returns the color and how much it covers, and the opacity is applied after it.
struct SkyPixel {
    // Unit direction of the pixel in the world, and in the object's own frame, which turns with
    // its orbit (x points at a body).
    dir: vec3<f32>,
    local: vec3<f32>,
    // 0..1 across the quad, or across the picture of a dome.
    uv: vec2<f32>,
    // The picture's sample (white without one) and the object's color, both linear.
    albedo: vec4<f32>,
    color: vec4<f32>,
    daylight: f32,
    time: f32,
}

// What a particle shader (`shaders/particle/<name>.wgsl`, `fn shade(p: ParticleFrag) -> vec4<f32>`)
// gets; it returns the color before lighting and fog.
struct ParticleFrag {
    // 0..1 across the quad.
    uv: vec2<f32>,
    // The texture sample and the kind's color at this age, both linear.
    albedo: vec4<f32>,
    color: vec4<f32>,
    // 0 at birth, 1 at the end of its life.
    life: f32,
    time: f32,
}
fn quat_vector(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let t = 2.0 * cross(q.xyz, v);
    return v + q.w * t + cross(q.xyz, t);
}
