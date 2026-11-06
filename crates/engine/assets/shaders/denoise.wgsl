// Our own denoiser for the path tracer's light (`denoise::Denoiser`, `core:pt_denoiser`), in the way
// of SVGF (Schied et al. 2017, "Spatiotemporal Variance-Guided Filtering"): the path tracer hands
// its diffuse and specular light with the albedo divided out (`pt_spec_albedo` for the specular),
// so texture detail never gets blurred; each is gathered over frames along TAA's motion with depth
// and normal tests, its variance worked out from the gathered moments (spatially while the history
// is short), filtered by a few à-trous passes that stop at edges of depth, normal and light, and the
// albedo multiplied back in, added onto the scene image (which holds emission and the sky).
// Every image is at the world's size. A shader pack may replace this file; it keeps the entries.
//
// Entry fields (`DenoisePass`, `dn`), by pass:
// - `fs_temporal` (diffuse, specular gathered; alpha the frames gathered): `diffuse`/`specular` the
//   path tracer's, `normal`, `depth`, `motion` + 1, `prev_*` last frame's (`prev_diffuse` and
//   `prev_specular` its first à-trous step, `prev_len_*` its gathered light for the frame counts,
//   `prev_guide` + 1, 0: start anew), `alpha` the least share of a new frame, `max_len` the most frames;
// - `fs_moments` (the light's luminance moments, and this frame's guide for the next): as above,
//   `prev_moments` last frame's;
// - `fs_variance` (diffuse, specular with their variance in alpha): `diffuse`/`specular` the gathered,
//   `moments` this frame's;
// - `fs_atrous` (one à-trous step): `diffuse`/`specular` the last step's, `step` the tap spacing,
//   `phi_color` how far light may differ (in standard deviations), `phi_normal` the normal's power;
// - `fs_composite` (added onto the scene image): `diffuse`/`specular` the light, `albedo`, `spec`
//   (F0), `normal`, `depth`.

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

fn dn_luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

// A texel of light, finite and not past `HUGE` (one stray path can't light a whole region).
const HUGE: f32 = 6.0e4;

fn dn_light(tex: u32, at: vec2<i32>, size: vec2<i32>) -> vec4<f32> {
    let t = textureLoad(textures[tex], clamp(at, vec2(0), size - 1), 0);
    let c = select(vec3(0.0), min(max(t.rgb, vec3(0.0)), vec3(HUGE)), t.rgb == t.rgb);
    return vec4(c, select(0.0, t.a, t.a == t.a));
}

// Unit vectors in two numbers (octahedral), for the guide kept for next frame.
fn dn_oct(n: vec3<f32>) -> vec2<f32> {
    let p = n.xy / max(abs(n.x) + abs(n.y) + abs(n.z), 1e-6);
    if n.z >= 0.0 {
        return p;
    }
    return (1.0 - abs(p.yx)) * select(vec2(-1.0), vec2(1.0), p >= vec2(0.0));
}

fn dn_unoct(e: vec2<f32>) -> vec3<f32> {
    var n = vec3(e, 1.0 - abs(e.x) - abs(e.y));
    if n.z < 0.0 {
        n = vec3((1.0 - abs(n.yx)) * select(vec2(-1.0), vec2(1.0), n.xy >= vec2(0.0)), n.z);
    }
    return normalize(n);
}

fn dn_depth(depth: u32, at: vec2<i32>, size: vec2<i32>) -> f32 {
    return textureLoad(textures[depth], clamp(at, vec2(0), size - 1), 0).r;
}

// Where the surface at texel `at` (depth `d`) is, camera-relative (blocks).
fn dn_position(at: vec2<i32>, d: f32, size: vec2<f32>) -> vec3<f32> {
    let uv = (vec2<f32>(at) + 0.5) / size;
    let h = scene.inv_view_proj * vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, max(d, 1e-7), 1.0);
    return h.xyz / h.w;
}

fn dn_normal(normal: u32, at: vec2<i32>, size: vec2<i32>) -> vec4<f32> {
    let t = textureLoad(textures[normal], clamp(at, vec2(0), size - 1), 0);
    return vec4(normalize(t.xyz + vec3(0.0, 0.0, 1e-6)), clamp(t.w, 0.0, 1.0));
}

// Where what is seen at `at` was last frame, minus where it is now, in UV: what the surface wrote
// in the motion image (its depth tag matching), else the camera's (as `taa.wgsl`).
fn dn_motion(motion: u32, depth: u32, at: vec2<i32>, size: vec2<i32>) -> vec2<f32> {
    let p = clamp(at, vec2(0), size - 1);
    let d = textureLoad(textures[depth], p, 0).r;
    if motion != 0u {
        let m = textureLoad(textures[motion - 1u], p, 0);
        if m.z > 0.0 && abs(m.z - d) <= max(d * 4.0e-3, 1.0e-7) {
            return m.xy;
        }
    }
    if d >= 0.5 {
        return vec2(0.0);
    }
    let fs = vec2<f32>(size);
    let uv = (vec2<f32>(p) + 0.5) / fs;
    let ndc = vec2(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    let h = scene.inv_view_proj * vec4(ndc, d, 1.0);
    let prev = scene.taa_prev * h;
    if prev.w <= 1e-9 {
        return vec2(0.0);
    }
    let now = ndc - scene.taa.xy;
    return (prev.xy / prev.w - now) * vec2(0.5, -0.5);
}

// Last frame's four texels round where this one's surface was, each weighed bilinearly and kept
// only where last frame saw the same surface there: about the same distance along the view (the
// guide's z against where this point stood from last frame's camera) and facing about the same way.
struct Reprojected {
    base: vec2<i32>,
    w: vec4<f32>,
    ok: bool,
}

fn dn_reproject(d: DenoisePass, at: vec2<i32>) -> Reprojected {
    var r: Reprojected;
    r.base = vec2(0);
    r.w = vec4(0.0);
    r.ok = false;
    let isize = vec2<i32>(d.size);
    let dep = dn_depth(d.depth, at, isize);
    if dep <= 0.0 || d.prev_guide == 0u || d.reset != 0u {
        return r;
    }
    let m = dn_motion(d.motion, d.depth, at, isize);
    let uv = (vec2<f32>(at) + 0.5) / d.size;
    let back = (uv + m) * d.size - 0.5;
    let base = vec2<i32>(floor(back));
    let f = back - floor(back);
    let p = dn_position(at, dep, d.size);
    let z_then = (scene.taa_prev * vec4(p, 1.0)).w;
    let n = dn_normal(d.normal, at, isize).xyz;
    let bil = vec4((1.0 - f.x) * (1.0 - f.y), f.x * (1.0 - f.y), (1.0 - f.x) * f.y, f.x * f.y);
    var w = vec4(0.0);
    for (var k = 0; k < 4; k++) {
        let q = base + vec2(k & 1, k >> 1u);
        if any(q < vec2(0)) || any(q >= isize) {
            continue;
        }
        let g = textureLoad(textures[d.prev_guide - 1u], q, 0);
        if g.z <= 0.0 {
            continue;
        }
        let same_depth = abs(g.z - z_then) <= 0.03 * z_then + 0.05;
        let same_facing = dot(dn_unoct(g.xy), n) > 0.85;
        if same_depth && same_facing {
            w[k] = bil[k];
        }
    }
    let s = w.x + w.y + w.z + w.w;
    if s < 1e-3 {
        return r;
    }
    r.base = base;
    r.w = w / s;
    r.ok = true;
    return r;
}

fn dn_fetch(tex: u32, r: Reprojected) -> vec4<f32> {
    var s = vec4(0.0);
    for (var k = 0; k < 4; k++) {
        if r.w[k] > 0.0 {
            s += textureLoad(textures[tex], r.base + vec2(k & 1, k >> 1u), 0) * r.w[k];
        }
    }
    return s;
}

struct Pair {
    @location(0) diffuse: vec4<f32>,
    @location(1) specular: vec4<f32>,
}

@fragment
fn fs_temporal(in: VOut) -> Pair {
    let d = dn[in.pass_index];
    let isize = vec2<i32>(d.size);
    let at = vec2<i32>(in.clip.xy);
    var out: Pair;
    if dn_depth(d.depth, at, isize) <= 0.0 {
        out.diffuse = vec4(0.0);
        out.specular = vec4(0.0);
        return out;
    }
    let cd = dn_light(d.diffuse, at, isize).rgb;
    let cs = dn_light(d.specular, at, isize).rgb;
    let r = dn_reproject(d, at);
    if !r.ok {
        out.diffuse = vec4(cd, 1.0);
        out.specular = vec4(cs, 1.0);
        return out;
    }
    // Last frame's light after its first à-trous step (SVGF's history: less noise carried on), the
    // frames gathered from the gathered light's alpha.
    let hd = dn_fetch(d.prev_diffuse, r);
    let hs = dn_fetch(d.prev_specular, r);
    // Smooth surfaces' reflections move with the eye, not the surface: they keep fewer frames.
    let rough = dn_normal(d.normal, at, isize).w;
    let len_d = min(dn_fetch(d.prev_len_diffuse, r).a + 1.0, d.max_len);
    let len_s = min(dn_fetch(d.prev_len_specular, r).a + 1.0, mix(4.0, d.max_len, clamp(rough * 2.0, 0.0, 1.0)));
    out.diffuse = vec4(mix(hd.rgb, cd, max(1.0 / len_d, d.alpha)), len_d);
    out.specular = vec4(mix(hs.rgb, cs, max(1.0 / len_s, d.alpha)), len_s);
    return out;
}

struct MomentsOut {
    @location(0) moments: vec4<f32>,
    @location(1) guide: vec4<f32>,
}

@fragment
fn fs_moments(in: VOut) -> MomentsOut {
    let d = dn[in.pass_index];
    let isize = vec2<i32>(d.size);
    let at = vec2<i32>(in.clip.xy);
    var out: MomentsOut;
    let dep = dn_depth(d.depth, at, isize);
    if dep <= 0.0 {
        out.moments = vec4(0.0);
        out.guide = vec4(0.0);
        return out;
    }
    let nr = dn_normal(d.normal, at, isize);
    let p = dn_position(at, dep, d.size);
    out.guide = vec4(dn_oct(nr.xyz), (scene.view_proj * vec4(p, 1.0)).w, nr.w);
    let ld = dn_luma(dn_light(d.diffuse, at, isize).rgb);
    let ls = dn_luma(dn_light(d.specular, at, isize).rgb);
    var m = vec4(ld, ld * ld, ls, ls * ls);
    let r = dn_reproject(d, at);
    if r.ok {
        let len = min(dn_fetch(d.prev_len_diffuse, r).a + 1.0, d.max_len);
        m = mix(dn_fetch(d.prev_moments, r), m, max(1.0 / len, 0.2));
    }
    out.moments = m;
    return out;
}

// How alike two surfaces are for a filter: by distance off this one's plane, and by facing.
fn dn_edge(nc: vec3<f32>, pc: vec3<f32>, zc: f32, nq: vec3<f32>, pq: vec3<f32>, phi_normal: f32) -> f32 {
    let plane = abs(dot(nc, pq - pc)) / (0.02 * zc + 0.02);
    return exp(-plane) * pow(max(dot(nc, nq), 0.0), phi_normal);
}

@fragment
fn fs_variance(in: VOut) -> Pair {
    let d = dn[in.pass_index];
    let isize = vec2<i32>(d.size);
    let at = vec2<i32>(in.clip.xy);
    var out: Pair;
    let dep = dn_depth(d.depth, at, isize);
    let hd = dn_light(d.diffuse, at, isize);
    let hs = dn_light(d.specular, at, isize);
    if dep <= 0.0 {
        out.diffuse = vec4(0.0);
        out.specular = vec4(0.0);
        return out;
    }
    let m = textureLoad(textures[d.moments], at, 0);
    var v = max(vec2(m.y - m.x * m.x, m.w - m.z * m.z), vec2(0.0));
    if hd.a < 4.0 {
        // Too few frames gathered: the moments of the 7x7 alike texels round it instead, made
        // larger while the history is this short.
        let nc = dn_normal(d.normal, at, isize).xyz;
        let pc = dn_position(at, dep, d.size);
        let zc = length(pc);
        var sum = vec4(0.0);
        var wsum = 0.0;
        for (var y = -3; y <= 3; y++) {
            for (var x = -3; x <= 3; x++) {
                let q = at + vec2(x, y);
                let dq = dn_depth(d.depth, q, isize);
                if dq <= 0.0 {
                    continue;
                }
                let w = dn_edge(nc, pc, zc, dn_normal(d.normal, q, isize).xyz, dn_position(clamp(q, vec2(0), isize - 1), dq, d.size), 64.0);
                let lq = vec2(dn_luma(dn_light(d.diffuse, q, isize).rgb), dn_luma(dn_light(d.specular, q, isize).rgb));
                sum += vec4(lq.x, lq.x * lq.x, lq.y, lq.y * lq.y) * w;
                wsum += w;
            }
        }
        let s = sum / max(wsum, 1e-6);
        v = max(vec2(s.y - s.x * s.x, s.w - s.z * s.z), vec2(0.0)) * (4.0 / max(hd.a, 1.0));
    }
    out.diffuse = vec4(hd.rgb, v.x);
    out.specular = vec4(hs.rgb, v.y);
    return out;
}

// The variance round `at`, blurred over 3x3 (a steadier measure for the light test).
fn dn_variance_blur(tex: u32, at: vec2<i32>, size: vec2<i32>) -> f32 {
    let k = array<f32, 3>(0.25, 0.5, 0.25);
    var s = 0.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            s += dn_light(tex, at + vec2(x, y), size).a * k[x + 1] * k[y + 1];
        }
    }
    return s;
}

@fragment
fn fs_atrous(in: VOut) -> Pair {
    let d = dn[in.pass_index];
    let isize = vec2<i32>(d.size);
    let at = vec2<i32>(in.clip.xy);
    var out: Pair;
    let dep = dn_depth(d.depth, at, isize);
    let cd = dn_light(d.diffuse, at, isize);
    let cs = dn_light(d.specular, at, isize);
    if dep <= 0.0 {
        out.diffuse = cd;
        out.specular = cs;
        return out;
    }
    let ncr = dn_normal(d.normal, at, isize);
    let nc = ncr.xyz;
    let pc = dn_position(at, dep, d.size);
    let zc = length(pc);
    let lc = vec2(dn_luma(cd.rgb), dn_luma(cs.rgb));
    let sigma = sqrt(max(vec2(dn_variance_blur(d.diffuse, at, isize), dn_variance_blur(d.specular, at, isize)), vec2(0.0)));
    let phi = d.phi_color * sigma + vec2(1e-4);
    // The B3 spline's 5 taps a side, spread `step` texels apart.
    let h = array<f32, 5>(1.0 / 16.0, 1.0 / 4.0, 3.0 / 8.0, 1.0 / 4.0, 1.0 / 16.0);
    var sd = vec3(0.0);
    var ss = vec3(0.0);
    var vd = 0.0;
    var vs = 0.0;
    var wd = 0.0;
    var ws = 0.0;
    let step = i32(d.step);
    for (var y = -2; y <= 2; y++) {
        for (var x = -2; x <= 2; x++) {
            let q = at + vec2(x, y) * step;
            if any(q < vec2(0)) || any(q >= isize) {
                continue;
            }
            let dq = dn_depth(d.depth, q, isize);
            if dq <= 0.0 {
                continue;
            }
            let nqr = dn_normal(d.normal, q, isize);
            let geo = dn_edge(nc, pc, zc, nqr.xyz, dn_position(q, dq, d.size), d.phi_normal) * h[x + 2] * h[y + 2];
            let qd = dn_light(d.diffuse, q, isize);
            let qs = dn_light(d.specular, q, isize);
            let w1 = geo * exp(-abs(dn_luma(qd.rgb) - lc.x) / phi.x);
            // Specular light also only between alike roughness: a mirror next to rough stone
            // keeps its own reflection.
            let w2 = geo * exp(-abs(dn_luma(qs.rgb) - lc.y) / phi.y) * exp(-abs(nqr.w - ncr.w) * 8.0);
            sd += qd.rgb * w1;
            vd += qd.a * w1 * w1;
            wd += w1;
            ss += qs.rgb * w2;
            vs += qs.a * w2 * w2;
            ws += w2;
        }
    }
    out.diffuse = vec4(sd / max(wd, 1e-6), vd / max(wd * wd, 1e-12));
    out.specular = vec4(ss / max(ws, 1e-6), vs / max(ws * ws, 1e-12));
    return out;
}

// The light with the albedo back in, added onto the scene image (emission and the sky are there):
// drawn with `Blend::Additive`, which scales by this alpha and keeps the image's own (the water mask).
@fragment
fn fs_composite(in: VOut) -> @location(0) vec4<f32> {
    let d = dn[in.pass_index];
    let isize = vec2<i32>(d.size);
    let at = vec2<i32>(in.clip.xy);
    let dep = dn_depth(d.depth, at, isize);
    if dep <= 0.0 {
        return vec4(0.0);
    }
    let nr = dn_normal(d.normal, at, isize);
    let v = -normalize(dn_position(at, dep, d.size));
    let albedo = textureLoad(textures[d.albedo], at, 0).rgb;
    let f0 = textureLoad(textures[d.spec], at, 0).rgb;
    let diffuse = dn_light(d.diffuse, at, isize).rgb * albedo;
    let specular = dn_light(d.specular, at, isize).rgb * pt_spec_albedo(f0, nr.w, abs(dot(nr.xyz, v)));
    return vec4(diffuse + specular, 1.0);
}

// The check before the path tracer exists (`Denoiser::run_test`, `WEFT_PT_DENOISE_TEST`): a light
// of mean 1 a pixel, new each frame (`step` the frame), heavy-tailed as one path a pixel is
// (-log of a uniform), and the normals from the depth (the nearer neighbour each way, as
// `upscale_guides.wgsl`), roughness 1. `diffuse` the picture (taken as the albedo), `depth`.
fn dn_hash(x: u32) -> u32 {
    var h = x * 747796405u + 2891336453u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    return (h >> 22u) ^ h;
}

@fragment
fn fs_test(in: VOut) -> Pair {
    let d = dn[in.pass_index];
    let isize = vec2<i32>(d.size);
    let at = vec2<i32>(in.clip.xy);
    var out: Pair;
    let dep = dn_depth(d.depth, at, isize);
    if dep <= 0.0 {
        out.diffuse = vec4(0.0);
        out.specular = vec4(0.0, 0.0, 1.0, 1.0);
        return out;
    }
    let u = f32(dn_hash(u32(at.x) + dn_hash(u32(at.y) + dn_hash(d.step))) >> 8u) / 16777216.0;
    let l = min(-log(max(1.0 - u, 1e-6)), 12.0);
    out.diffuse = vec4(vec3(l), 0.0);
    let c = dn_position(at, dep, d.size);
    let l_ = dn_position(clamp(at - vec2(1, 0), vec2(0), isize - 1), dn_depth(d.depth, at - vec2(1, 0), isize), d.size);
    let r_ = dn_position(clamp(at + vec2(1, 0), vec2(0), isize - 1), dn_depth(d.depth, at + vec2(1, 0), isize), d.size);
    let u_ = dn_position(clamp(at - vec2(0, 1), vec2(0), isize - 1), dn_depth(d.depth, at - vec2(0, 1), isize), d.size);
    let w_ = dn_position(clamp(at + vec2(0, 1), vec2(0), isize - 1), dn_depth(d.depth, at + vec2(0, 1), isize), d.size);
    let dx = select(c - l_, r_ - c, length(r_ - c) < length(c - l_));
    let dy = select(c - u_, w_ - c, length(w_ - c) < length(c - u_));
    var n = normalize(cross(dy, dx));
    if dot(n, c) > 0.0 {
        n = -n;
    }
    out.specular = vec4(n, 1.0);
    return out;
}
