// ---- path traced lighting (`core:path_traced`, a GPU with ray queries; reports/Path tracing plan.md) ----
//
// M0: after the world's pass, each pixel whose surface is terrain (the ray from the eye meets, at
// the depth the world drew, a block face or distant tile, not water, glass or a model) is lit again
// by tracing: the sun through `rt_sun_soft`, and one bounce ray that either reaches the sky (the
// pack's own sky light there, as the lightmap gives it) or meets a surface, which is then lit as the
// world lights it and lends its colour (light off walls and floors). Our grade stays: the light
// comes from `light_on`, the air from `haze_lit`. Temporal anti-aliasing's history averages the one
// ray a pixel into the picture. Composed by the engine with the world shader's prelude; `pt`,
// `pt_out` and `pt_in` come from it.
//
// What it hands on (the denoisers' interface, v1): per pixel the diffuse light divided by the
// albedo, and the specular light divided by the specular albedo (`pt_env_brdf`), each with the
// distance its ray went (65504: the sky), in `pt_out` three texels a pixel: [0] diffuse, [1]
// specular, both already times the air's transmittance (`haze_lit` is affine in the colour:
// haze(c) = A c + B); [2] B, and a: 0 untraced, 1 traced, 2 a debug view. The composite writes
// diffuse and specular to the engine's images and the picture itself (`pt.mode` 0), or only B
// for a denoiser to add the light onto (1).

const PT_SUN_CONE: f32 = 0.012;
const PT_REACH: f32 = 384.0;

fn pt_unpack_rgb(c: u32) -> vec3<f32> {
    return vec3<f32>(f32((c >> 16u) & 255u), f32((c >> 8u) & 255u), f32(c & 255u)) / 255.0;
}

// The biome's tint at world column (x, z) (as `world.wgsl`'s `tint_color`), gamma corrected.
fn pt_tint(tint: u32, biome: u32, x: i32, z: i32) -> vec3<f32> {
    if tint == 0u {
        return vec3(1.0);
    }
    let t = i32(g.ids.w);
    var sum = vec3(0.0);
    var n = 0.0;
    for (var k = 0; k < 4; k++) {
        let cx = x - 1 + (k & 1);
        let cz = z - 1 + (k >> 1u);
        let tx = ((cx >> 4u) % t + t) % t;
        let tz = ((cz >> 4u) % t + t) % t;
        let i = g.ids.z + u32(tz * t + tx) * 256u + u32((cz & 15) * 16 + (cx & 15));
        let c = biomes[i * g.extra.z + tint - 1u];
        if c != 0u {
            sum += pt_unpack_rgb(c);
            n += 1.0;
        }
    }
    let rgb = select(pt_unpack_rgb(biomes[biome * g.extra.z + tint - 1u]), sum / max(n, 1.0), n > 0.0);
    return pow(rgb, vec3(2.2));
}

fn pt_albedo(h: RtHit) -> vec3<f32> {
    return h.albedo.rgb * pt_tint(h.tint, h.biome, i32(floor(h.world.x)), i32(floor(h.world.z)));
}

// A cosine-weighted direction about `n` from two numbers 0..1.
fn pt_cosine(n: vec3<f32>, u: vec2<f32>) -> vec3<f32> {
    let r = sqrt(u.x);
    let a = 6.2831853 * u.y;
    let t = normalize(cross(n, select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(n.y) > 0.9)));
    let b = cross(n, t);
    return normalize(t * (r * cos(a)) + b * (r * sin(a)) + n * sqrt(max(1.0 - u.x, 0.0)));
}

// Two numbers 0..1 for pixel `px`, moving every frame (`scene.rt.y`), apart from `rt_cone`'s.
fn pt_noise(px: vec2<f32>) -> vec2<f32> {
    let k = f32(u32(scene.rt.y) % 64u);
    let p = px + vec2(5.588238 * k) + vec2(41.0, 7.0);
    let a = fract(52.9829189 * fract(dot(p, vec2(0.06711056, 0.00583715))));
    let b = fract(52.9829189 * fract(dot(p.yx + vec2(13.0, 3.0), vec2(0.06711056, 0.00583715))));
    return vec2(a, b);
}

// The specular albedo: how much of the light round it a surface of reflectance `f0` and perceptual
// roughness `rough` mirrors toward an eye at `nv` (n.v), averaged over the lobe. Karis' fit (2014,
// "Physically Based Shading on Mobile") of the split-sum's scale and bias. The denoisers divide and
// multiply by the same (their `pt_spec_albedo`).
fn pt_env_brdf(f0: vec3<f32>, rough: f32, nv: f32) -> vec3<f32> {
    let c0 = vec4(-1.0, -0.0275, -0.572, 0.022);
    let c1 = vec4(1.0, 0.0425, 1.04, -0.04);
    let r = rough * c0 + c1;
    let a004 = min(r.x * r.x, exp2(-9.28 * nv)) * r.x + r.y;
    let ab = vec2(-1.04, 1.04) * a004 + r.zw;
    return f0 * ab.x + ab.y;
}

fn pt_schlick(f0: vec3<f32>, c: f32) -> vec3<f32> {
    return f0 + (vec3(1.0) - f0) * pow(1.0 - clamp(c, 0.0, 1.0), 5.0);
}

// ---- the radiance cache (M1; NVIDIA's SHaRC idea, our own code) ----
//
// World-space cells, a block wide near the eye and twice as wide each doubling of the distance past
// 24 blocks, one per face direction, in a hash table of `PT_CACHE` cells (`pt_cache`, ten words a
// cell: [0, N) keys, [N, 5N) this frame's sums r g b and count, [5N, 9N) the resolved light r g b
// and its history as f32 bits, [9N, 10N) frames since last touched). Each traced pixel (one in four
// a frame) adds the indirect light it found (what its bounce ray brought, or the sky's share where
// the ray left) to its cell; `cs_cache` folds the sums into the resolved light. A bounce ray that
// meets a surface then takes that surface's indirect light from its cell instead of the lightmap's
// sky: light that has bounced any number of times, a few frames late.

const PT_PROBES: u32 = 8u;
const PT_MIN_HISTORY: f32 = 4.0;
const PT_EVICT: u32 = 240u;

fn pt_hash(x: u32) -> u32 {
    let s = x * 747796405u + 2891336453u;
    let w = ((s >> ((s >> 28u) + 4u)) ^ s) * 277803737u;
    return (w >> 22u) ^ w;
}

// The cell for a surface at `world` facing `n`, `dist` from the eye: its slot's hash and its key
// (never 0, the empty slot's).
fn pt_cell(world: vec3<f32>, n: vec3<f32>, dist: f32) -> vec2<u32> {
    let level = u32(clamp(floor(log2(max(dist, 24.0) / 24.0)), 0.0, 6.0));
    let c = vec3<i32>(floor((world + n * 0.01) / f32(1u << level)));
    let a = abs(n);
    var axis = select(select(4u, 2u, a.y >= a.z), 0u, a.x >= a.y && a.x >= a.z);
    if dot(n, vec3(f32(axis == 0u), f32(axis == 2u), f32(axis == 4u))) < 0.0 {
        axis += 1u;
    }
    let h = pt_hash(bitcast<u32>(c.x) ^ pt_hash(bitcast<u32>(c.y) ^ pt_hash(bitcast<u32>(c.z) ^ pt_hash(axis | (level << 3u)))));
    return vec2(h, pt_hash(h ^ 0x9e3779b9u) | 1u);
}

// The cell's slot, -1 if it has none (and `insert` false, or the probes are full).
fn pt_cache_find(cell: vec2<u32>, insert: bool) -> i32 {
    for (var p = 0u; p < PT_PROBES; p++) {
        let i = (cell.x + p) % PT_CACHE;
        let k = atomicLoad(&pt_cache[i]);
        if k == cell.y {
            return i32(i);
        }
        if k == 0u {
            if !insert {
                return -1;
            }
            let r = atomicCompareExchangeWeak(&pt_cache[i], 0u, cell.y);
            if r.exchanged || r.old_value == cell.y {
                return i32(i);
            }
        }
    }
    return -1;
}

fn pt_cache_add(i: u32, light: vec3<f32>) {
    let q = vec3<u32>(clamp(light, vec3(0.0), vec3(64.0)) * 256.0);
    let a = PT_CACHE + i * 4u;
    atomicAdd(&pt_cache[a], q.r);
    atomicAdd(&pt_cache[a + 1u], q.g);
    atomicAdd(&pt_cache[a + 2u], q.b);
    atomicAdd(&pt_cache[a + 3u], 1u);
}

// The resolved light of slot `i` (rgb) and how many samples it has (a).
fn pt_cache_get(i: u32) -> vec4<f32> {
    let r = PT_CACHE * 5u + i * 4u;
    return vec4(
        bitcast<f32>(atomicLoad(&pt_cache[r])),
        bitcast<f32>(atomicLoad(&pt_cache[r + 1u])),
        bitcast<f32>(atomicLoad(&pt_cache[r + 2u])),
        bitcast<f32>(atomicLoad(&pt_cache[r + 3u])),
    );
}

// The indirect light the cache holds for a surface (rgb; a: 1 found, 0 none yet).
fn pt_cached(world: vec3<f32>, n: vec3<f32>, dist: f32) -> vec4<f32> {
    let i = pt_cache_find(pt_cell(world, n, dist), false);
    if i < 0 {
        return vec4(0.0);
    }
    let c = pt_cache_get(u32(i));
    return select(vec4(0.0), vec4(c.rgb, 1.0), c.a >= PT_MIN_HISTORY);
}

// A surface met by a ray, lit as the world lights it: its block light, the sun through one more
// ray, and its indirect light from the cache (the lightmap's sky light until the cache has it).
fn pt_lit(b: RtHit, l: vec3<f32>) -> vec3<f32> {
    var s = 0.0;
    if dot(b.n, l) > 0.0 && light_strength() > 0.0 {
        s = dot(rt_sun(b.rel, b.n, l, false), vec3(1.0 / 3.0));
    }
    let c = pt_cached(b.world, b.n, length(b.rel));
    if c.a > 0.5 {
        return pt_albedo(b) * (light_on(b.n, vec4(0.0, b.light.yzw), s, 1.0, b.world, b.rel) + c.rgb);
    }
    return pt_albedo(b) * light_on(b.n, b.light, s, 1.0, b.world, b.rel);
}

// Folds each cell's sums into its light (more samples this frame, a bigger step; never under 1/24,
// so it follows changes within a second), and frees the cells no path has touched for `PT_EVICT`
// frames.
@compute @workgroup_size(64)
fn cs_cache(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= PT_CACHE || atomicLoad(&pt_cache[i]) == 0u {
        return;
    }
    let a = PT_CACHE + i * 4u;
    let r = PT_CACHE * 5u + i * 4u;
    let age = PT_CACHE * 9u + i;
    let count = atomicLoad(&pt_cache[a + 3u]);
    if count == 0u {
        let old = atomicAdd(&pt_cache[age], 1u);
        if old + 1u > PT_EVICT {
            atomicStore(&pt_cache[i], 0u);
            atomicStore(&pt_cache[r + 3u], 0u);
            atomicStore(&pt_cache[age], 0u);
        }
        return;
    }
    let n = f32(count);
    let sum = vec3(f32(atomicLoad(&pt_cache[a])), f32(atomicLoad(&pt_cache[a + 1u])), f32(atomicLoad(&pt_cache[a + 2u]))) / 256.0;
    let c = pt_cache_get(i);
    let w = max(n / (c.a + n), 1.0 / 24.0);
    let light = mix(c.rgb, sum / n, w);
    atomicStore(&pt_cache[r], bitcast<u32>(light.r));
    atomicStore(&pt_cache[r + 1u], bitcast<u32>(light.g));
    atomicStore(&pt_cache[r + 2u], bitcast<u32>(light.b));
    atomicStore(&pt_cache[r + 3u], bitcast<u32>(min(c.a + n, 256.0)));
    for (var k = 0u; k < 4u; k++) {
        atomicStore(&pt_cache[a + k], 0u);
    }
    atomicStore(&pt_cache[age], 0u);
}

const PT_SKY_T: f32 = 65504.0;

@compute @workgroup_size(8, 8)
fn cs_trace(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= pt.size.x || id.y >= pt.size.y {
        return;
    }
    let at = (id.y * pt.size.x + id.x) * 3u;
    pt_out[at] = vec4(0.0);
    pt_out[at + 1u] = vec4(0.0);
    pt_out[at + 2u] = vec4(0.0);
    let ip = vec2<i32>(id.xy);
    let d = textureLoad(textures[pt.depth], ip, 0).r;
    if d <= 0.0 {
        return;
    }
    let px = vec2<f32>(id.xy) + 0.5;
    let ndc = vec2(px.x / f32(pt.size.x) * 2.0 - 1.0, 1.0 - px.y / f32(pt.size.y) * 2.0);
    let c = scene.inv_view_proj * vec4(ndc, d, 1.0);
    let rel = c.xyz / c.w;
    let dist = length(rel);
    // The far land keeps the world's own light (traced shadows hand it to the far map from 256
    // blocks too): its rays are the dearest and its detail the least.
    if dist > PT_REACH {
        return;
    }
    let v = -rel / dist;
    let h = rt_hit(vec3(0.0), -v, dist * 1.1 + 2.0, 0.0);
    // Terrain at the depth drawn there: anything else (water, glass, a model, a particle) keeps the
    // world's own pixel.
    if !h.hit || h.clear || abs(h.t - dist) > 0.02 * dist + 0.15 {
        return;
    }
    // The surface as the G-buffer has it (its shading normal, roughness, F0); the face's own normal
    // for the rays' starts.
    let gb_n = textureLoad(textures[pt.normal], ip, 0);
    let gb_s = textureLoad(textures[pt.spec], ip, 0);
    let albedo = textureLoad(textures[pt.albedo], ip, 0).rgb;
    let ng = h.n;
    let n = select(ng, normalize(gb_n.xyz), dot(gb_n.xyz, gb_n.xyz) > 0.25);
    let rough_p = gb_n.w;
    let f0 = gb_s.rgb;
    let l = light_dir();
    var sun = 0.0;
    if dot(ng, l) > 0.0 && light_strength() > 0.0 {
        sun = dot(rt_sun_soft(h.rel, ng, l, false, PT_SUN_CONE, px, false), vec3(1.0 / 3.0));
    }
    // Diffuse. The world's light (its sky light from the lightmap), and its sky's share alone.
    let lit = light_on(n, h.light, sun, 1.0, h.world, h.rel);
    let sky_part = max(lit - light_on(n, vec4(0.0, h.light.yzw), sun, 1.0, h.world, h.rel), vec3(0.0));
    // One bounce: open sky keeps the sky's share; a surface takes its place, lit as the world.
    let b = rt_hit(h.rel + ng * 0.01, pt_cosine(n, pt_noise(px)), 96.0, 2.0);
    var bounce = vec3(0.0);
    var diffuse = lit;
    var diffuse_t = PT_SKY_T;
    if b.hit {
        bounce = pt_lit(b, l);
        diffuse = lit - sky_part + bounce;
        // Above 0 on every traced pixel: the denoisers read diffuse's a > 0 as "traced".
        diffuse_t = max(b.t, 1e-3);
    }
    // Into the cache: the indirect light found here (one pixel in four a frame).
    if ((id.x + id.y * 3u + u32(scene.rt.y)) & 3u) == 0u {
        let i = pt_cache_find(pt_cell(h.world, ng, dist), true);
        if i >= 0 {
            pt_cache_add(u32(i), select(sky_part, bounce, b.hit));
        }
    }
    // Specular, where the texture has maps (F0 above 0), as the world's `mat_shine`: the sun's
    // highlight (GGX, Smith's visibility) and what lies the mirrored way, traced where smooth, the
    // sky or the room's light where rough.
    var specular = vec3(0.0);
    var specular_t = 0.0;
    let nv = max(dot(n, v), 1e-4);
    let s_alb = pt_env_brdf(f0, rough_p, nv);
    if max(f0.r, max(f0.g, f0.b)) > 0.0 {
        let a = rough_p * rough_p;
        var shine = vec3(0.0);
        let ndl = max(dot(n, l), 0.0);
        if ndl > 0.0 && scene.sun.w > 0.0 {
            let hv = normalize(v + l);
            let ndh = max(dot(n, hv), 0.0);
            let a2 = max(a, 0.03) * max(a, 0.03);
            let dd = ndh * ndh * (a2 - 1.0) + 1.0;
            let dist_d = a2 / (3.14159265 * dd * dd);
            let vis = 0.5 / max(ndl * sqrt(nv * nv * (1.0 - a2) + a2) + nv * sqrt(ndl * ndl * (1.0 - a2) + a2), 1e-4);
            shine = min(dist_d * vis, 60.0) * pt_schlick(f0, max(dot(v, hv), 0.0)) * ndl * palette().sun * light_strength() * sun;
        }
        let r = reflect(-v, n);
        let sky_seen = mix(sky_color(r), sky_color(vec3(0.0, 1.0, 0.0)), clamp(a * 2.0, 0.0, 1.0));
        let open = smoothstep(0.8, 1.0, h.light.x);
        var env = mix(diffuse * 0.5, sky_seen, open * open);
        // The face's own horizon: a normal map tilts some texels (mortar joints, bevels) until they
        // mirror along the face or into it, where the groove and the block itself hide the sky. The
        // stone bricks' joints lit up as blue lines from the sky (pt1_rtxd_pt, the cathedral).
        let horizon = smoothstep(-0.05, 0.3, dot(r, ng));
        if a < 0.5 && horizon > 0.0 {
            var dir = rt_cone(r, 0.01 + a * 0.5, px + vec2(17.0, 29.0));
            if dot(dir, ng) <= 0.0 {
                dir = r;
            }
            let m = rt_hit(h.rel + ng * 0.01, dir, 384.0, a * 6.0);
            var seen = sky_color(dir);
            specular_t = PT_SKY_T;
            if m.hit {
                seen = haze_lit(pt_lit(m, l), dir * m.t, m.light.x);
                specular_t = m.t;
            }
            env = mix(seen, env, smoothstep(0.1, 0.5, a));
        }
        let gloss = 1.0 - clamp(a * 1.6, 0.0, 0.85);
        specular = (pt_schlick(f0, nv) * env * gloss * horizon + shine) / max(s_alb, vec3(1e-3));
    }
    if pt.debug != 0u {
        var out = vec3(0.0);
        switch pt.debug {
            case 1u: { out = albedo; }
            case 2u: { out = n * 0.5 + 0.5; }
            case 3u: { out = albedo * (lit - sky_part); }
            case 4u: { out = bounce; }
            case 5u: { out = specular * s_alb; }
            case 6u: { out = albedo * pt_cached(h.world, ng, dist).rgb; }
            default: {}
        }
        pt_out[at] = vec4(out, 0.0);
        pt_out[at + 2u] = vec4(0.0, 0.0, 0.0, 2.0);
        return;
    }
    // The air between the eye and the surface, as an affine map of the colour.
    let air = haze_lit(vec3(0.0), h.rel, h.light.x);
    let through = haze_lit(vec3(1.0), h.rel, h.light.x) - air;
    let dif = through * diffuse;
    let spe = through * specular;
    pt_out[at] = vec4(dif, diffuse_t);
    pt_out[at + 1u] = vec4(spe, specular_t);
    // The picture's own (mode 0), or what the denoiser adds onto (1).
    let under = select(air + dif * albedo + spe * s_alb, air, pt.mode == 1u);
    pt_out[at + 2u] = vec4(under, 1.0);
}

struct PtOut {
    @builtin(position) clip: vec4<f32>,
}

@vertex
fn vs_pt(@builtin(vertex_index) vi: u32) -> PtOut {
    let p = vec2(f32((vi << 1u) & 2u), f32(vi & 2u));
    var out: PtOut;
    out.clip = vec4(p * 2.0 - 1.0, 0.0, 1.0);
    return out;
}

struct PtFrag {
    @location(0) color: vec4<f32>,
    @location(1) diffuse: vec4<f32>,
    @location(2) specular: vec4<f32>,
}

// The traced pixels over the world's (others left as they are), and their light for the
// denoisers (0 where untraced).
@fragment
fn fs_pt(in: PtOut) -> PtFrag {
    let px = vec2<u32>(in.clip.xy);
    let at = (px.y * pt.size.x + px.x) * 3u;
    let kind = pt_in[at + 2u];
    if kind.a < 0.5 {
        discard;
    }
    if kind.a > 1.5 {
        return PtFrag(vec4(pt_in[at].rgb, 0.0), vec4(0.0), vec4(0.0));
    }
    return PtFrag(vec4(kind.rgb, 0.0), pt_in[at], pt_in[at + 1u]);
}
