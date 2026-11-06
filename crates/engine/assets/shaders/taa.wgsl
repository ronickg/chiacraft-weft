// Temporal anti-aliasing (`post::Post`, `core:aa`): the world is drawn a fraction of a pixel aside
// each frame (`Scene::taa.xy`), and each frame's picture is gathered into the picture so far,
// moved by where each pixel was last frame (the motion image, `weft_render::MOTION_FORMAT`).
// A shader pack may replace this file; it keeps the entry points.
//
// `fs_taa`'s entry: `scene` this frame's world, `depth` its depth, `low` the picture so far + 1
// (0: none, start anew), `bloom` the motion image + 1 (0: none, all from depth), `size` the
// world's size, `strength` the least share of this frame (0.1: about ten frames gathered).
// `fs_taa_motion`'s: the same, writing the whole motion image for an upscaler (DLSS, Arm ASR).

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) pass_index: u32,
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    var out: VOut;
    out.clip = vec4(p * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2(p.x, 1.0 - p.y);
    out.pass_index = ii;
    return out;
}

fn taa_luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

fn taa_ycocg(c: vec3<f32>) -> vec3<f32> {
    return vec3(
        0.25 * c.r + 0.5 * c.g + 0.25 * c.b,
        0.5 * c.r - 0.5 * c.b,
        -0.25 * c.r + 0.5 * c.g - 0.25 * c.b,
    );
}

fn taa_rgb(c: vec3<f32>) -> vec3<f32> {
    return vec3(c.x + c.y - c.z, c.x + c.z, c.x - c.y - c.z);
}

// This frame's color at a texel, finite, and weighed down where it is bright (Karis): one sun
// glint or a lit leaf's edge then can't flicker the whole gathered pixel.
fn taa_texel(tex: u32, at: vec2<i32>, size: vec2<i32>) -> vec3<f32> {
    let c = textureLoad(textures[tex], clamp(at, vec2(0), size - 1), 0).rgb;
    let f = select(vec3(0.0), min(c, vec3(6.0e4)), c == c);
    return max(f, vec3(0.0));
}

fn taa_tone(c: vec3<f32>) -> vec3<f32> {
    return c / (1.0 + taa_luma(c));
}

fn taa_untone(c: vec3<f32>) -> vec3<f32> {
    return c / max(1.0 - taa_luma(c), 1e-4);
}

// Where what is seen at texel `at` (depth `d`) stood last frame, minus where it stands now, in UV;
// from the camera alone: the world holds still. The sky (depth 0) turns with the camera only, and
// the first person's hand (depth past 0.5, `vs_hand`) moves with it, so stays.
fn taa_camera_motion(at: vec2<i32>, d: f32, size: vec2<f32>) -> vec2<f32> {
    if d >= 0.5 {
        return vec2(0.0);
    }
    let uv = (vec2<f32>(at) + 0.5) / size;
    let ndc = vec2(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    let h = scene.inv_view_proj * vec4(ndc, d, 1.0);
    // Homogeneous all the way: at the sky's depth 0 `h` is a direction (w 0), which last frame's
    // camera only turns.
    let prev = scene.taa_prev * h;
    if prev.w <= 1e-9 {
        return vec2(0.0);
    }
    let pn = prev.xy / prev.w;
    // Now without the jitter: where the point is, not where this frame drew it.
    let now = ndc - scene.taa.xy;
    return (pn - now) * vec2(0.5, -0.5);
}

// The motion at texel `at`: what a surface wrote there (`@location(1)`, its depth in z matching
// the world's), else the camera's.
fn taa_motion(motion: u32, depth: u32, at: vec2<i32>, size: vec2<i32>) -> vec2<f32> {
    let p = clamp(at, vec2(0), size - 1);
    let d = textureLoad(textures[depth], p, 0).r;
    if motion != 0u {
        let m = textureLoad(textures[motion - 1u], p, 0);
        if m.z > 0.0 && abs(m.z - d) <= max(d * 4.0e-3, 1.0e-7) {
            return m.xy;
        }
    }
    return taa_camera_motion(p, d, vec2<f32>(size));
}

// The picture so far at `uv`, filtered with Catmull-Rom in five bilinear taps (Jimenez's
// "Filmic SMAA"): sharper than one bilinear tap, which would blur it a little more each frame.
fn taa_history(tex: u32, uv: vec2<f32>) -> vec4<f32> {
    let size = vec2<f32>(textureDimensions(textures[tex]));
    let p = uv * size;
    let c = floor(p - 0.5) + 0.5;
    let f = p - c;
    let w0 = f * (-0.5 + f * (1.0 - 0.5 * f));
    let w1 = 1.0 + f * f * (-2.5 + 1.5 * f);
    let w2 = f * (0.5 + f * (2.0 - 1.5 * f));
    let w3 = f * f * (-0.5 + 0.5 * f);
    let w12 = w1 + w2;
    let t0 = (c - 1.0) / size;
    let t3 = (c + 2.0) / size;
    let t12 = (c + w2 / w12) / size;
    let t = textures[tex];
    var s = textureSampleLevel(t, linear_clamp, vec2(t12.x, t0.y), 0.0) * (w12.x * w0.y);
    s += textureSampleLevel(t, linear_clamp, vec2(t0.x, t12.y), 0.0) * (w0.x * w12.y);
    s += textureSampleLevel(t, linear_clamp, vec2(t12.x, t12.y), 0.0) * (w12.x * w12.y);
    s += textureSampleLevel(t, linear_clamp, vec2(t3.x, t12.y), 0.0) * (w3.x * w12.y);
    s += textureSampleLevel(t, linear_clamp, vec2(t12.x, t3.y), 0.0) * (w12.x * w3.y);
    let w = w12.x * w0.y + w0.x * w12.y + w12.x * w12.y + w3.x * w12.y + w12.x * w3.y;
    // Its gathered count straight, not filtered (the ringing of the lobes would make it up).
    let n = textureSampleLevel(t, nearest_clamp, uv, 0.0).a;
    return vec4(max(s.rgb / w, vec3(0.0)), n);
}

// The history moved toward the box round this frame's neighbourhood (`lo`..`hi`, about `mid`) until
// it is inside: what was there last frame but is no longer (disoccluded, lit anew, moved off) gives
// way, what still is stays (Karis's clipping, in YCoCg as Salvi's).
fn taa_clip(h: vec3<f32>, mid: vec3<f32>, lo: vec3<f32>, hi: vec3<f32>) -> vec3<f32> {
    let half_ = max((hi - lo) * 0.5, vec3(1e-5));
    let centre = (hi + lo) * 0.5;
    let v = h - centre;
    let units = abs(v / half_);
    let most = max(units.x, max(units.y, units.z));
    if most > 1.0 {
        return centre + v / most;
    }
    return h;
}

@fragment
fn fs_taa(in: VOut) -> @location(0) vec4<f32> {
    let pass_ = passes[in.pass_index];
    let isize = vec2<i32>(pass_.size);
    let at = vec2<i32>(in.clip.xy);
    let uv = (vec2<f32>(at) + 0.5) / pass_.size;
    // This frame's 3x3: its mean and spread (in YCoCg, toned), and the nearest surface in it,
    // whose motion this pixel takes (an edge then moves with what is in front).
    var m1 = vec3(0.0);
    var m2 = vec3(0.0);
    var lo = vec3(1e9);
    var hi = vec3(-1e9);
    var here = vec3(0.0);
    var near = at;
    var near_d = -1.0;
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let p = at + vec2(i, j);
            let c = taa_ycocg(taa_tone(taa_texel(pass_.scene, p, isize)));
            if i == 0 && j == 0 {
                here = c;
            }
            m1 += c;
            m2 += c * c;
            lo = min(lo, c);
            hi = max(hi, c);
            let d = textureLoad(textures[pass_.depth], clamp(p, vec2(0), isize - 1), 0).r;
            if d > near_d {
                near_d = d;
                near = p;
            }
        }
    }
    let mean = m1 / 9.0;
    let sigma = sqrt(max(m2 / 9.0 - mean * mean, vec3(0.0)));
    // The variance box, inside the min-max one: tighter where the neighbourhood is mostly alike.
    let box_lo = max(lo, mean - sigma * 1.25);
    let box_hi = min(hi, mean + sigma * 1.25);
    if pass_.low == 0u || scene.taa.z < 0.5 {
        return vec4(taa_untone(taa_rgb(here)), 1.0);
    }
    let motion = taa_motion(pass_.bloom, pass_.depth, near, isize);
    let back = uv + motion;
    if any(back < vec2(0.0)) || any(back > vec2(1.0)) {
        return vec4(taa_untone(taa_rgb(here)), 1.0);
    }
    let hist = taa_history(pass_.low - 1u, back);
    let h = taa_ycocg(taa_tone(hist.rgb));
    let clipped = taa_clip(h, mean, box_lo, box_hi);
    // How many frames are gathered here: one more each frame, fewer where the history had to give
    // way (it was something else), at most 1 / `strength`.
    let gave = length(clipped - h) / max(length(box_hi - box_lo), 1e-4);
    var n = min(hist.a, 1.0 / max(pass_.strength, 0.02));
    n = mix(n, 1.0, clamp(gave * 2.0, 0.0, 1.0));
    // Fast motion: a little more of this frame, so moving land doesn't smear (each frame's history
    // is filtered once more where it is read between texels).
    let px = length(motion * pass_.size);
    n = min(n, mix(1.0 / max(pass_.strength, 0.02), 4.0, clamp(px / 16.0, 0.0, 1.0)));
    // Where rain or snow lies over it (the motion image's w, the weather's cover): that much more of
    // this frame, so their streaks don't wash out.
    var reactive = 0.0;
    if pass_.bloom != 0u {
        reactive = clamp(textureLoad(textures[pass_.bloom - 1u], at, 0).w, 0.0, 1.0);
    }
    let w = max(1.0 / (n + 1.0), reactive * 0.8);
    let out = taa_untone(taa_rgb(mix(clipped, here, w)));
    return vec4(max(out, vec3(0.0)), mix(n + 1.0, 1.0, reactive));
}

// The whole motion image for an upscaler (DLSS, Arm ASR: `post::TemporalInputs`): xy the motion
// in UV, z the depth, w 1 where a surface wrote its own, 0 from the camera.
@fragment
fn fs_taa_motion(in: VOut) -> @location(0) vec4<f32> {
    let pass_ = passes[in.pass_index];
    let isize = vec2<i32>(pass_.size);
    let at = vec2<i32>(in.clip.xy);
    let d = textureLoad(textures[pass_.depth], at, 0).r;
    if pass_.bloom != 0u {
        let m = textureLoad(textures[pass_.bloom - 1u], at, 0);
        if m.z > 0.0 && abs(m.z - d) <= max(d * 4.0e-3, 1.0e-7) {
            return vec4(m.xy, d, 1.0);
        }
    }
    return vec4(taa_motion(0u, pass_.depth, at, isize), d, 0.0);
}
