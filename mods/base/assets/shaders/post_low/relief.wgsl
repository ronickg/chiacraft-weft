// Far relief (chiacraft's shader pack turns it on, `mods/base/src/shaderpack.rs`; Far view, 11-06): the
// land from about 200 to 2000 blocks out, where the sun shadows have ended, shaded by its own shapes
// from the world's depth, worked out here at half size (upsampled by depth in `post/relief.wgsl`).
// Bliss + Distant Horizons' far hills read as sunlit terrace tops over near-black steps, a texture
// like a canopy, where ours stood soft and even (the vista's hill at noon): its far land takes a
// screen-space ambient occlusion with a near-constant screen radius and a shadow marched toward the
// sun on the depth (Bliss's `SSAO` and `SSRT_Shadows` on DH's depth, learnt from, not taken). Two
// parts: how much of the light round a point is closed in by the land beside it (r), and whether the
// sun is hidden from it by land between (g, 1 lit, 0 hidden).

const RELIEF_AO_SAMPLES: i32 = 8;
const RELIEF_SUN_STEPS: i32 = 14;

fn relief_hash(p: vec2<f32>) -> f32 {
    let q = vec2<u32>(p);
    var h = q.x * 1973u + q.y * 9277u + 26699u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    return f32((h ^ (h >> 15u)) & 1023u) / 1024.0;
}

// How far out the far relief counts (0 none, 1 all): from 150-300 blocks, where the sun shadows'
// far cascade and the near land's own shade end, easing out from 500 to 2000 as the haze takes the
// land (darkened under it, the haze went a dirty grey where Bliss's far land is a soft blue: Look,
// 11-06). Long before the haze hands the land over to the sky (`haze_lit`'s `edge`, from 0.84 of
// the far end).
fn relief_weight(dist: f32) -> f32 {
    return smoothstep(150.0, 300.0, dist) * (1.0 - smoothstep(500.0, 2000.0, dist));
}

// A cloud, not land: over the clouds' underside less a margin (the clouds write their depth; the
// pack's `sky_or_cloud`, which the engine's own post shader lacks). Camera relative.
fn relief_cloud(pos: vec3<f32>) -> bool {
    return scene.clouds.x > 0.5 && pos.y > scene.clouds.z - 20.0;
}

// The surface's normal at `uv` from its neighbours' depth, each axis taking the neighbour nearer in
// depth (the same surface), so the edges of far boxes don't smear it.
fn relief_normal(p: PostPixel, uv: vec2<f32>, pos: vec3<f32>) -> vec3<f32> {
    let px = 1.0 / p.size;
    let l = post_position(p, uv - vec2(px.x, 0.0)) - pos;
    let r = post_position(p, uv + vec2(px.x, 0.0)) - pos;
    let u = post_position(p, uv - vec2(0.0, px.y)) - pos;
    let d = post_position(p, uv + vec2(0.0, px.y)) - pos;
    let dx = select(-l, r, dot(r, r) < dot(l, l));
    let dy = select(-u, d, dot(d, d) < dot(u, u));
    var n = normalize(cross(dy, dx) + vec3(0.0, 1e-6, 0.0));
    if dot(n, pos) > 0.0 {
        n = -n;
    }
    return n;
}

fn relief_on_screen(uv: vec2<f32>) -> bool {
    return uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0;
}

// (ambient left 0..1, sun left 0..1) at this pixel.
fn relief_dark(p: PostPixel) -> vec2<f32> {
    let depth = post_depth(p, p.uv);
    // The sky, or the eye in a liquid.
    if depth <= 0.0 || abs(scene.params.x - 0.6 * scene.params.y) > 0.5 {
        return vec2(1.0);
    }
    let pos = post_position(p, p.uv);
    let dist = length(pos);
    let w = relief_weight(dist);
    if w <= 0.0 || relief_cloud(pos) {
        return vec2(1.0);
    }
    let n = relief_normal(p, p.uv, pos);
    let cheap = p.strength < 0.75;
    let noise = relief_hash(p.uv * p.size);
    // Ambient occlusion: samples in a spiral about 2% of the screen's height across, what rises over
    // the surface within an eighth of the distance closing it in (Bliss's DH radius is about that).
    let samples = select(RELIEF_AO_SAMPLES, RELIEF_AO_SAMPLES / 2, cheap);
    let reach2 = (0.12 * dist) * (0.12 * dist);
    var closed = 0.0;
    var counted = 0.0;
    for (var i = 0; i < samples; i++) {
        let f = (f32(i) + noise) / f32(samples);
        let a = (f32(i) * 2.3999632 + noise * 6.2831853);
        let at = p.uv + vec2(cos(a) * p.size.y / p.size.x, sin(a)) * 0.022 * sqrt(f);
        if !relief_on_screen(at) || post_depth(p, at) <= 0.0 {
            continue;
        }
        let seen = post_position(p, at);
        let v = seen - pos;
        let d2 = dot(v, v);
        if d2 < 1e-4 || relief_cloud(seen) {
            continue;
        }
        counted += 1.0;
        closed += clamp(dot(v * inverseSqrt(d2), n), 0.0, 1.0) * clamp(1.0 - d2 / reach2, 0.0, 1.0);
    }
    let ambient = 1.0 - clamp(closed / max(counted, 1.0) * 1.6, 0.0, 0.75) * w;
    // The sun hidden by land between: marched toward it in steps about 6 pixels long on the screen,
    // a hit where the land drawn there stands in front of the ray by no more than 3.5% of its
    // distance. Faces turned from the sun are dark already (their own shading): not marched.
    var sun = 1.0;
    let to_sun = scene.sun.xyz;
    if scene.sun.w > 0.0 && dot(n, to_sun) > 0.05 {
        let steps = select(RELIEF_SUN_STEPS, RELIEF_SUN_STEPS / 2, cheap);
        // A pixel's size in blocks here, and the step along the sun's way that moves about 6 of them
        // across the screen (more where the sun's way runs toward or from the eye).
        let c0 = scene.view_proj * vec4(pos, 1.0);
        let c1 = scene.view_proj * vec4(pos + scene.right.xyz, 1.0);
        let pixel = 1.0 / max(length((c1.xy / c1.w - c0.xy / c0.w) * 0.5 * p.size), 1e-4);
        let across = max(length(to_sun - dot(to_sun, pos / dist) * pos / dist), 0.2);
        let step = 6.0 * pixel / across * select(1.0, 2.0, cheap);
        for (var k = 1; k <= steps; k++) {
            let q = pos + n * pixel + to_sun * step * (f32(k) - 0.5 + noise);
            let c = scene.view_proj * vec4(q, 1.0);
            if c.w <= 0.0 {
                break;
            }
            let at = vec2(c.x / c.w * 0.5 + 0.5, 0.5 - c.y / c.w * 0.5);
            if !relief_on_screen(at) {
                break;
            }
            if post_depth(p, at) <= 0.0 {
                continue;
            }
            let ray = length(q);
            let there = post_position(p, at);
            if relief_cloud(there) {
                continue;
            }
            let seen = length(there);
            if seen < ray - pixel && ray - seen < 0.035 * ray {
                sun = 0.0;
                break;
            }
        }
    }
    return vec2(ambient, mix(1.0, sun, w));
}

fn low(p: PostPixel) -> vec4<f32> {
    let d = relief_dark(p);
    return vec4(d.x, d.y, 0.0, 1.0);
}
