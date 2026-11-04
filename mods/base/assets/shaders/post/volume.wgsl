// Volumetric light (chiacraft's shader pack turns it on, `mods/base/src/shaderpack.rs`): the
// sunlight the air between the eye and what it sees scatters toward the eye, where the sun reaches
// that air through the gaps the shadow maps leave: beams through the trees, a bright mist toward a
// low sun, dark lanes behind hills. Each pixel's view is walked in a few jittered steps; more of it
// in fog and at dawn and dusk, none at night or in rain.

const VOL_STEPS: i32 = 16;
// How far along the view the air is walked (blocks): the near and far shadow maps cover it.
const VOL_REACH: f32 = 160.0;

// 1 where the sun reaches camera-relative `rel` (the near shadow map, else the far one), 1 also out
// of both maps' reach.
fn vol_lit(rel: vec3<f32>) -> f32 {
    if scene.shadow.z > 0.5 {
        let c = scene.shadow_view_proj * vec4(rel, 1.0);
        let ndc = c.xyz / c.w;
        if abs(ndc.x) < 0.98 && abs(ndc.y) < 0.98 && ndc.z > 0.0 && ndc.z < 1.0 {
            let at = vec2<i32>(vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * scene.shadow.y);
            let d = textureLoad(textures[u32(scene.shadow.x)], at, 0).r;
            return select(0.0, 1.0, d <= ndc.z + 0.0005);
        }
    }
    if scene.shadow_far.z > 0.5 {
        let c = scene.shadow_far_view_proj * vec4(rel, 1.0);
        let ndc = c.xyz / c.w;
        if abs(ndc.x) < 0.98 && abs(ndc.y) < 0.98 && ndc.z > 0.0 && ndc.z < 1.0 {
            let at = vec2<i32>(vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * scene.shadow_far.y);
            let d = textureLoad(textures[u32(scene.shadow_far.x)], at, 0).r;
            return select(0.0, 1.0, d <= ndc.z + 0.0004);
        }
    }
    return 1.0;
}

fn apply(p: PostPixel) -> vec4<f32> {
    let sun = scene.sun;
    let day = smoothstep(-0.02, 0.06, sun.y);
    // At night, in rain, or with the eye in a liquid (its fog no longer starts at 0.6 of its end).
    if day <= 0.0 || scene.fog_color.w > 0.95 || abs(scene.params.x - 0.6 * scene.params.y) > 0.5 {
        return p.color;
    }
    // Along the view to what is drawn there, or the sky.
    var end = VOL_REACH;
    var dir: vec3<f32>;
    if post_depth(p, p.uv) > 0.0 {
        let pos = post_position(p, p.uv);
        end = min(length(pos), VOL_REACH);
        dir = pos / max(length(pos), 1e-4);
    } else {
        let far = scene.inv_view_proj * vec4(p.uv.x * 2.0 - 1.0, 1.0 - p.uv.y * 2.0, 1e-6, 1.0);
        dir = normalize(far.xyz / far.w);
    }
    let px = vec2<u32>(p.uv * p.size);
    var h = px.x * 7919u + px.y * 104729u + 31337u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    let jitter = f32((h ^ (h >> 15u)) & 1023u) / 1024.0;
    // Strength under 0.75 (the Video screen's Low): as strong, in half the steps.
    let cheap = p.strength < 0.75;
    let steps = select(VOL_STEPS, VOL_STEPS / 2, cheap);
    var lit = 0.0;
    for (var i = 0; i < steps; i++) {
        let t = (f32(i) + jitter) / f32(steps) * end;
        lit += vol_lit(dir * t);
    }
    lit /= f32(steps);
    // Toward the sun the air glows most (Henyey-Greenstein, g 0.65), a little all round.
    let g = 0.65;
    let mu = dot(dir, sun.xyz);
    let phase = (1.0 - g * g) / pow(1.0 + g * g - 2.0 * g * mu, 1.5) * 0.25 + 0.15;
    // Thin air by day, thicker in fog and with the sun low.
    let low = 1.0 - smoothstep(0.1, 0.5, sun.y);
    let density = (0.0025 + 0.004 * low + 0.03 * scene.weather.z) * (1.0 - scene.fog_color.w);
    let air = 1.0 - exp(-density * end);
    let color = mix(vec3(1.0, 0.62, 0.35), vec3(1.0, 0.95, 0.85), smoothstep(0.05, 0.5, sun.y));
    let glow = color * lit * air * phase * day * select(min(p.strength, 1.0), 1.0, cheap) * 0.9;
    return vec4(p.color.rgb + glow * (vec3(1.0) - p.color.rgb * 0.6), p.color.a);
}
