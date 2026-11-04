// Light shafts (chiacraft's shader pack turns them on, `mods/base/src/shaderpack.rs`): from each
// pixel toward the sun on the screen, how much bright sky lies along the way. Trees, hills and
// clouds in front of the sun are dark and cut it into rays; strongest with the sun low and in
// fog, gone at night and in rain.

const SHAFT_STEPS: i32 = 40;

fn shaft_luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

fn apply(p: PostPixel) -> vec4<f32> {
    let sun = scene.sun;
    // Under the horizon, no sky, or the eye in water or lava (their fog no longer starts at 0.6
    // of its end there).
    if sun.w <= 0.0 || abs(scene.params.x - 0.6 * scene.params.y) > 0.5 {
        return p.color;
    }
    let clip = scene.view_proj * vec4(sun.xyz * 1000.0, 1.0);
    if clip.w <= 0.0 {
        return p.color;
    }
    let ndc = clip.xy / clip.w;
    let at = vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    // Fades as the sun leaves the screen.
    let off = max(max(-at.x, at.x - 1.0), max(-at.y, at.y - 1.0));
    let on_screen = 1.0 - smoothstep(0.0, 0.6, off);
    // Low sun, fog and haze make them; rain takes them.
    let low = 1.0 - smoothstep(0.15, 0.7, sun.y);
    let air = 0.3 + 0.7 * low + 0.8 * scene.weather.z;
    let amount = min(p.strength, 1.0) * on_screen * air * (1.0 - scene.fog_color.w) * smoothstep(-0.01, 0.015, sun.y);
    if amount <= 0.001 {
        return p.color;
    }
    // What counts as bright: near the sky round the sun as it is now (a pink sunset is dimmer
    // than the noon glare), the brightest of a few spots about it.
    var bright = 0.35;
    for (var k = 0; k < 5; k++) {
        let a = f32(k) * 1.2566;
        let o = select(vec2(cos(a), sin(a)) * 0.03, vec2(0.0), k == 0);
        let c = textureSampleLevel(textures[p.scene], linear_clamp, clamp(at + o, vec2(0.0), vec2(1.0)), 0.0).rgb;
        bright = max(bright, shaft_luma(c));
    }
    let to_sun = at - p.uv;
    let step = to_sun / f32(SHAFT_STEPS);
    // Each pixel starts a little further along, at random: no even bands from even steps.
    let px = vec2<u32>(p.uv * p.size);
    var h = px.x * 1973u + px.y * 9277u + 26699u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    let jitter = f32((h ^ (h >> 15u)) & 1023u) / 1024.0;
    var uv = p.uv + step * jitter;
    var decay = 1.0;
    var lit = 0.0;
    for (var i = 0; i < SHAFT_STEPS; i++) {
        uv += step;
        let c = textureSampleLevel(textures[p.scene], linear_clamp, clamp(uv, vec2(0.0), vec2(1.0)), 0.0).rgb;
        // Only the bright: the sky about the sun, not lit ground.
        // At sunset the disc alone is that bright: then the glowing sky round it counts too.
        lit += smoothstep(bright * mix(0.6, 0.38, low), bright * mix(0.95, 0.82, low), shaft_luma(c)) * decay;
        decay *= 0.955;
    }
    lit /= f32(SHAFT_STEPS) * 0.45;
    // Brighter the nearer the sun, wide around it.
    let near = exp(-length(to_sun * vec2(p.size.x / p.size.y, 1.0)) * mix(2.4, 1.3, low));
    let tint = mix(vec3(1.0, 0.72, 0.42), vec3(1.0, 0.95, 0.85), smoothstep(0.1, 0.6, sun.y));
    let shafts = tint * lit * near * amount * mix(0.16, 0.55, low);
    // Strength 2 (`--shafts-debug`): how it sees the frame, amount in red, the sky it gathered in
    // green, how near the sun in blue.
    if p.strength > 1.5 {
        return vec4(min(amount, 1.0), min(lit, 1.0), near, 1.0);
    }
    return vec4(min(p.color.rgb + shafts * (1.0 - p.color.rgb * 0.5), vec3(1.0)), 1.0);
}
