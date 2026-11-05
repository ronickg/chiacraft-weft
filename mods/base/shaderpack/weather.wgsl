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

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    // Only some stretches of a column's picture show, falling with it: fewer drops, of any picture.
    let band = floor(in.uv.y * 8.0);
    if fract(sin(band * 12.9898 + f32(in.data.z & 4095u) * 78.233) * 43758.5453) * 255.0 > f32(in.data.w) {
        discard;
    }
    var color = textureSampleLevel(textures[in.data.x], nearest_clamp, fract(in.uv), 0.0);
    if color.a < 0.01 {
        discard;
    }
    let light = light_round(unpack_light(in.data.y), vec3(0.0), vec3(1e4));
    // Thin grey streaks rather than vanilla's pale-blue drops: half grey, fainter, and fading out
    // right by the eye, where a drop's texel is as big as a fingernail. Drawn as they were (the
    // texture's blue at 1.4x its alpha) they read as big pale-blue rectangles (Ronald, 11-05).
    let tint = mix(color.rgb, vec3(luma(color.rgb)) * vec3(0.95, 0.98, 1.02), 0.5);
    let near = smoothstep(1.5, 6.0, length(in.rel));
    return vec4(display(haze(tint * light * 0.8, in.rel)), min(color.a * in.alpha * 0.85 * near, 1.0));
}
