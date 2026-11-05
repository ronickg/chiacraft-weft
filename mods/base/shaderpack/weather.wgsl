// chiacraft's rain and snow: lit and graded like the world.
// Rain and snow: a strip standing in each column around the camera, one instance each. It turns
// toward the camera about the vertical only, so it stays upright wherever the camera looks.

var<private> CORNER: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0),
);

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) data: vec4<u32>,
    @location(2) alpha: f32,
    @location(3) dist: f32,
    @location(4) rel: vec3<f32>,
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let w = columns[ii];
    let c = CORNER[vi % 6u];
    let across = normalize(vec2(w.place.z, -w.place.x) + vec2(1e-4, 0.0));
    var world = vec3(w.place.x + across.x * (c.x - 0.5), mix(w.place.w, w.place.y, c.y), w.place.z + across.y * (c.x - 0.5));
    // Drops come in on the wind: higher up they are further upwind (rain falls about 7 m/s).
    world = vec3(world.x - wind_now().x / 7.0 * (world.y - w.place.w), world.y, world.z - wind_now().y / 7.0 * (world.y - w.place.w));
    var out: VOut;
    out.clip = scene.view_proj * vec4(world, 1.0);
    out.uv = vec2(w.uv.x + c.x * 0.25, w.uv.y + (c.y - 1.0) * w.uv.z);
    out.data = w.data;
    out.alpha = w.uv.w;
    out.dist = length(world);
    out.rel = world;
    return out;
}

// A hash of a column's seed and a cell in it, 0..1.
fn fall_hash(seed: u32, cell: vec2<i32>) -> f32 {
    var h = seed ^ (u32(cell.x) * 0x27d4eb2du) ^ (u32(cell.y) * 0x165667b1u);
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    h = h ^ (h >> 15u);
    return f32(h & 0xffffffu) / 16777215.0;
}

// Drawn here, not from the picture's texels: as texels, rain's drops were thin vertical rectangles
// with hard edges, a stretch here and there of each strip, and snow's flakes a pixel or two, lost
// against a grey sky (Tester 2's weather sheet, 11-05). Rain is fine streaks, three lanes to a
// block, each a drop blurred by its fall (a block or so long, soft across and at its ends); snow
// round flakes a few centimetres across, tumbling a little as they come down. Both thinner than a
// pixel far off go fainter rather than wider, so a column far away is a soft veil, not a bar.
// Which is which by the precipitation's density (the engine's rain 0.1, snow 0.5; a mod's own
// picture, like ash, as flakes).
@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    // Across the strip (0..1 block, with snow's drift) and down it (blocks, falling with time).
    let x = fract(in.uv.x * 4.0);
    let y = in.uv.y * 4.0;
    // Blocks a pixel covers across the strip (before any branch: derivatives need every pixel).
    let px = max(fwidth(in.uv.x * 4.0), 1e-4);
    let snow = in.data.w >= 64u;
    let seed = in.data.z;
    var cover = 0.0;
    if snow {
        // Cells a quarter block each way; a flake in some of them, somewhere in the cell.
        let q = vec2(x * 4.0, y * 4.0);
        let cell = vec2<i32>(floor(q));
        let h = fall_hash(seed, cell);
        if h < 0.45 {
            let h2 = fall_hash(seed + 17u, cell);
            let h3 = fall_hash(seed + 31u, cell);
            let sway = sin(y * 2.3 + h * 40.0) * 0.12;
            let at = vec2(0.25 + 0.5 * h2 + sway, 0.25 + 0.5 * h3);
            let r = mix(0.025, 0.05, h * 2.0) * 4.0;
            let d = length((fract(q) - at) * vec2(1.0, 1.0));
            let soft = max(r, px * 4.0 * 0.8);
            cover = (1.0 - smoothstep(soft * 0.5, soft, d)) * min((r * r) / (soft * soft), 1.0);
        }
    } else {
        // Three lanes a block; in each a drop every three and a half blocks down.
        let lane = floor(x * 3.0);
        let lx = fract(x * 3.0);
        let h = fall_hash(seed, vec2(i32(lane), 0));
        let h2 = fall_hash(seed + 7u, vec2(i32(lane), 0));
        let period = 3.5;
        let k = floor(y / period + h);
        let hk = fall_hash(seed + 13u, vec2(i32(lane), i32(k)));
        let len = 0.8 + 0.7 * hk;
        let along = fract(y / period + h) * period / len;
        if along < 1.0 {
            let cx = 0.25 + 0.5 * h2;
            let dx = (lx - cx) / 3.0;
            let w = 0.012;
            let soft = max(w, px * 0.8);
            let across = exp(-(dx / soft) * (dx / soft) * 2.0);
            let ends = smoothstep(0.0, 0.25, along) * smoothstep(1.0, 0.55, along);
            cover = across * ends * (w / soft);
        }
    }
    if cover < 0.004 {
        discard;
    }
    let light = light_round(unpack_light(in.data.y), vec3(0.0), vec3(1e4));
    // Drops a pale grey-blue, flakes white; fading out right by the eye.
    let tint = select(vec3(0.78, 0.84, 0.92), vec3(1.0, 1.0, 1.04), snow);
    let near = smoothstep(select(1.5, 1.0, snow), select(6.0, 3.0, snow), length(in.rel));
    let strength = select(0.6, 0.95, snow);
    return vec4(display(haze(tint * light * select(1.0, 1.15, snow), in.rel)), min(cover * in.alpha * strength * near, 1.0));
}
