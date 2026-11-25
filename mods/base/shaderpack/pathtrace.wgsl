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
// A hash of the pixel and the frame (Jarzynski and Olano 2020's pcg3d): its two numbers don't go
// together. Interleaved gradient noise's did, and steered the bounce and reflection rays alike
// along a diagonal: the faces showed diagonal hatching where they should show plain grain for
// temporal anti-aliasing and the denoisers to settle (NVIDIA's storeroom, pt1_hatching_crop).
fn pt_noise(px: vec2<f32>) -> vec2<f32> {
    var v = vec3<u32>(vec2<u32>(max(floor(px), vec2(0.0))), u32(scene.rt.y));
    v = v * 1664525u + 1013904223u;
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v = v ^ (v >> vec3(16u));
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    return vec2<f32>(v.xy >> vec2(8u)) / 16777216.0;
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
// (never 0, the empty slot's). `u` (two numbers 0..1, different each pixel and frame) moves the
// point up to a cell's width along the face first, so neighbouring cells blend under temporal
// anti-aliasing instead of showing as squares of light (the cathedral's pillar, pt1_rtxd_m2c).
fn pt_cell(world: vec3<f32>, n: vec3<f32>, dist: f32, u: vec2<f32>) -> vec2<u32> {
    let level = u32(clamp(floor(log2(max(dist, 24.0) / 24.0)), 0.0, 6.0));
    let size = f32(1u << level);
    let t = normalize(cross(n, select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(n.y) > 0.9)));
    let b = cross(n, t);
    let at = world + (t * (u.x - 0.5) + b * (u.y - 0.5)) * size;
    let c = vec3<i32>(floor((at + n * 0.01) / size));
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
fn pt_cached(world: vec3<f32>, n: vec3<f32>, dist: f32, u: vec2<f32>) -> vec4<f32> {
    let i = pt_cache_find(pt_cell(world, n, dist, u), false);
    if i < 0 {
        return vec4(0.0);
    }
    let c = pt_cache_get(u32(i));
    return select(vec4(0.0), vec4(c.rgb, 1.0), c.a >= PT_MIN_HISTORY);
}

// A surface met by a ray, lit as the world lights it: its block light, the sun through one more
// ray, and its indirect light from the cache (the lightmap's sky light until the cache has it).
// `light_on` with the sun as traced: its direct part not gated by the lightmap's sky light. The
// world lets the sun in only where the sky light is high (`light_on`'s `open`), to keep its shadow
// map's gaps out of caves; the traced shadow has no gaps, and with the gate the sun through a 2x2
// roof hole never reached the cave floor (w41 cave_sky). The ambient-occlusion term is 1 here.
fn pt_light_on(n: vec3<f32>, light: vec4<f32>, sun: f32, pos: vec3<f32>, rel: vec3<f32>) -> vec3<f32> {
    let c = light_on(n, light, 0.0, 1.0, pos, rel);
    let sun_part = palette().sun * light_strength() * sun_facing(n) * sun;
    LIGHT_SUN = sun_part;
    return c + sun_part;
}

fn pt_lit(b: RtHit, l: vec3<f32>, u: vec2<f32>) -> vec3<f32> {
    var s = 0.0;
    if dot(b.n, l) > 0.0 && light_strength() > 0.0 {
        s = dot(rt_sun(b.rel, b.n, l, false), vec3(1.0 / 3.0)) * models_shadow(b.rel, b.n) * cloud_shadow(b.rel);
    }
    let c = pt_cached(b.world, b.n, length(b.rel), u);
    if c.a > 0.5 {
        return pt_albedo(b) * (pt_light_on(b.n, vec4(0.0, b.light.yzw), s, b.world, b.rel) + c.rgb);
    }
    return pt_albedo(b) * pt_light_on(b.n, b.light, s, b.world, b.rel);
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

// Two numbers 0..1 for pixel `px`, this frame, and a draw `salt` of its own (`pt_noise`'s hash with
// the salt in the frame's word): each of the block lights' draws apart from the others.
fn pt_rand(px: vec2<f32>, salt: u32) -> vec2<f32> {
    var v = vec3<u32>(vec2<u32>(max(floor(px), vec2(0.0))), u32(scene.rt.y) ^ (salt * 0x9E3779B9u));
    v = v * 1664525u + 1013904223u;
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v = v ^ (v >> vec3(16u));
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    return vec2<f32>(v.xy >> vec2(8u)) / 16777216.0;
}

// ---- block lights (M2; reports/Path tracing plan.md) ----
//
// The lightmap's block light stays the light's colour and reach (our grade); what tracing adds is
// whether the light can be seen. Each pixel draws a few candidates from the light-giving blocks in
// the 27 cubes round it (`pt_lights`, from the meshes), each weighted by the light it gives there
// by the lightmap's own falloff (its level less the blocks between), keeps one in proportion
// (resampled importance sampling), and sends one shadow ray to it. Averaged over frames, that is the
// share of the block light that reaches the surface: shadows behind pillars and furniture.

const PT_LIGHT_CANDIDATES: u32 = 8u;

// Where cube `c`'s lights are in `pt_lights` (start, count); count 0 when it has none.
fn pt_light_find(c: vec3<i32>) -> vec2<u32> {
    let slots = pt_lights[0];
    var i = ((u32(c.x) * 73856093u) ^ (u32(c.y) * 19349663u) ^ (u32(c.z) * 83492791u)) & (slots - 1u);
    for (var p = 0u; p < 32u; p++) {
        let at = 2u + i * 4u;
        let x = pt_lights[at];
        if x == 0x80000000u {
            return vec2(0u);
        }
        if x == u32(c.x) && pt_lights[at + 1u] == u32(c.y) && pt_lights[at + 2u] == u32(c.z) {
            let v = pt_lights[at + 3u];
            return vec2(v & 0xFFFFFu, v >> 20u);
        }
        i = (i + 1u) & (slots - 1u);
    }
    return vec2(0u);
}

// How much of the block light reaches a surface at `world` (`rel` camera relative) facing `ng`:
// x 1 seen, 0 hidden, by one light drawn as above (1 when none is near); y 1 when a light was drawn.
fn pt_block_visibility(world: vec3<f32>, rel: vec3<f32>, ng: vec3<f32>, lift: vec3<f32>, px: vec2<f32>) -> vec2<f32> {
    if pt_lights[1] == 0u {
        return vec2(1.0, 0.0);
    }
    let cell = vec3<i32>(floor(world + ng * 0.5));
    let cube = cell >> vec3(4u);
    let base = 2u + pt_lights[0] * 4u;
    // The cubes round with lights, first: drawing the cube at random among all 27 found a light in
    // a quarter of frames in NVIDIA's storeroom, and a frame with none counted as seen, so the
    // shadows all but vanished (pt1_rtxd_m10v).
    var held: array<vec4<u32>, 27>;
    var n = 0u;
    var total = 0u;
    for (var ci = 0u; ci < 27u; ci++) {
        let r = pt_light_find(cube + vec3<i32>(i32(ci % 3u) - 1, i32((ci / 3u) % 3u) - 1, i32(ci / 9u) - 1));
        if r.y > 0u {
            held[n] = vec4(r.x, r.y, ci, total);
            total += r.y;
            n += 1u;
        }
    }
    if total == 0u {
        return vec2(1.0, 0.0);
    }
    var chosen = vec3(0.0);
    var own = false;
    var w_sum = 0.0;
    var found = false;
    for (var k = 0u; k < PT_LIGHT_CANDIDATES; k++) {
        // One of the `total` lights, each as likely.
        let t = min(u32(pt_rand(px, 16u + k * 2u).x * f32(total)), total - 1u);
        var c = 0u;
        for (var j = 1u; j < n; j++) {
            if held[j].w <= t {
                c = j;
            }
        }
        let ci = held[c].z;
        let at = cube + vec3<i32>(i32(ci % 3u) - 1, i32((ci / 3u) % 3u) - 1, i32(ci / 9u) - 1);
        let l = pt_lights[base + held[c].x + (t - held[c].w)];
        let lp = vec3<i32>(i32(l & 15u), i32((l >> 4u) & 15u), i32((l >> 8u) & 15u)) + at * 16;
        let e = f32(max((l >> 12u) & 15u, max((l >> 16u) & 15u, (l >> 20u) & 15u)));
        let d = abs(lp - cell);
        let level = e - f32(d.x + d.y + d.z);
        if level <= 0.0 {
            continue;
        }
        // Its weight over the chance of drawing it (1 in `total`).
        let w = level * f32(total);
        w_sum += w;
        if pt_rand(px, 17u + k * 2u).x * w_sum < w {
            chosen = vec3<f32>(lp) + 0.5;
            own = all(lp == vec3<i32>(floor(world - ng * 0.5)));
            found = true;
        }
    }
    if !found {
        return vec2(1.0, 0.0);
    }
    if own {
        return vec2(1.0, 1.0);
    }
    let to = chosen - world;
    let dist = length(to);
    let dir = to / max(dist, 1e-4);
    // A light behind the face lights it only round a corner, as the lightmap spreads it: hidden.
    if dot(dir, ng) <= 0.0 {
        return vec2(0.0, 1.0);
    }
    let hit = rt_hit(rel + lift, dir, max(dist - 0.9, 0.0), 0.0);
    return vec2(select(1.0, 0.0, hit.hit && !hit.clear), 1.0);
}

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
    // Debug 7: the picture, with a line where the traced land ends.
    if pt.debug == 7u && abs(dist - PT_REACH) < 1.5 {
        pt_out[at] = vec4(1.0, 0.0, 1.0, 0.0);
        pt_out[at + 2u] = vec4(0.0, 0.0, 0.0, 2.0);
        return;
    }
    if dist > PT_REACH || pt.debug == 12u {
        return;
    }
    let v = -rel / dist;
    // Out to the farthest hit the test below accepts: one past it fails either way (the same
    // pixels traced), and the ray walks no further. The cubes' faces and quads alone: a distant
    // tile it met would be no pixel traced either (the G-buffer has none), and the hidden ones
    // under the cubes cost each ray their shader's asking. The world's `pt_takes` asks the same.
    let h = rt_hit_mask(vec3(0.0), -v, dist * 1.02 + 0.2, 0.0, 3u);
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
    // Only where the G-buffer drew (the cubes' opaque and cutout faces): a distant tile inside the
    // reach met there has no G-buffer, and traced with no albedo a far peak came out black
    // (pt1_dark_peak_crop); it keeps the world's pixel, as the denoisers skip it.
    if dot(gb_n.xyz, gb_n.xyz) < 0.25 {
        return;
    }
    // Debug 8-12, timing only (a bench's probes): the trace stopped after its primary test (8), the
    // sun's ray (9), the bounce and the cache (10), or with no specular ray (11); 12 before any ray, above; 13 the whole trace without the air's
    // two `haze_lit`s. The picture they
    // leave is no use.
    if pt.debug == 8u {
        return;
    }
    let ng = h.n;
    let n = normalize(gb_n.xyz);
    // Rays leave a little off the face, more with the distance: the hit point comes from the
    // primary ray's length, less exact than the world's own points.
    let lift = ng * (0.01 + dist * 3e-4);
    let rough_p = gb_n.w;
    let f0 = gb_s.rgb;
    let l = light_dir();
    var sun = 0.0;
    if dot(ng, l) > 0.0 && light_strength() > 0.0 {
        sun = dot(rt_sun_soft(h.rel + lift, ng, l, false, PT_SUN_CONE, px, false), vec3(1.0 / 3.0));
        // As the world's traced sun (`world.wgsl` `sun_shadow_near`): the models' shadows (mobs
        // and players are not in the traced scene) and the clouds'. Without them a rainy day's
        // ground stayed sunlit where the world's lay under its clouds (w33).
        sun *= models_shadow(h.rel, n) * cloud_shadow(h.rel);
    }
    // Diffuse. The world's light (its sky light from the lightmap); the sky's share of it, as the
    // lightmap gives it (already lowered under cover) and as the open sky would give it.
    let lit = pt_light_on(n, h.light, sun, h.world, h.rel);
    if pt.debug == 9u {
        pt_out[at] = vec4(lit, 1.0);
        return;
    }
    // The lamps' and the sky's shares of it for the night's blue (`display`), kept before the calls
    // below, which light the sky alone and set it to 1: traced, a fire-lit wall at night went
    // white-blue where the world draws it orange (w12, the nave at 18000).
    let share = NIGHT_SKY_SHARE;
    let dark = light_on(n, vec4(0.0), 0.0, 1.0, h.world, h.rel);
    let sky_part = max(light_on(n, vec4(h.light.x, 0.0, 0.0, 0.0), 0.0, 1.0, h.world, h.rel) - dark, vec3(0.0));
    let sky_open = max(light_on(n, vec4(1.0, 0.0, 0.0, 0.0), 0.0, 1.0, h.world, h.rel) - dark, vec3(0.0));
    // One bounce in place of the lightmap's sky: a surface met gives its own light, and a ray that
    // gets out gives the open sky's: the rays themselves say how much of the sky is hidden. (With
    // the lightmap's lowered share there, cover counted twice and a roofless ruin's walls came out
    // a quarter darker than the world draws them, NVIDIA's cathedral, pt1_rtxd_d3.)
    // About the normal-mapped normal, mirrored back over the face where that sends it in: a bumpy
    // texture's rays went into its own block and darkened it in patches (the storeroom's gold).
    var bdir = pt_cosine(n, pt_noise(px));
    if dot(bdir, ng) < 0.0 {
        bdir -= 2.0 * dot(bdir, ng) * ng;
    }
    let b = rt_hit(h.rel + lift, bdir, 96.0, 2.0);
    var bounce = vec3(0.0);
    var diffuse_t = PT_SKY_T;
    if b.hit {
        bounce = pt_lit(b, l, pt_noise(px + vec2(3.0, 11.0)));
        // Above 0 on every traced pixel: the denoisers read diffuse's a > 0 as "traced".
        diffuse_t = max(b.t, 1e-3);
    }
    let indirect = select(sky_open, bounce, b.hit);
    var diffuse = lit - sky_part + indirect;
    // The block light, shadowed: its share of the world's light, as much of it as reaches the
    // surface (one light drawn among those near, one shadow ray; frames average it). The
    // lightmap's block light keeps its colour and reach; the held light is not a block's.
    let block = max(light_on(n, h.light, 0.0, 1.0, h.world, h.rel) - light_on(n, vec4(h.light.x, 0.0, 0.0, 0.0), 0.0, 1.0, h.world, h.rel), vec3(0.0));
    if max(block.r, max(block.g, block.b)) > 1e-4 {
        let block_vis = pt_block_visibility(h.world, h.rel, ng, lift, px);
        diffuse -= block * (1.0 - block_vis.x);
    }
    // Into the cache: the indirect light found here (one pixel in four a frame).
    if ((id.x + id.y * 3u + u32(scene.rt.y)) & 3u) == 0u {
        let i = pt_cache_find(pt_cell(h.world, ng, dist, pt_noise(px + vec2(7.0, 5.0))), true);
        if i >= 0 {
            pt_cache_add(u32(i), indirect);
        }
        // And at the surface the bounce met, one ray further (SHaRC's update along the path): the
        // cells the bounce rays read are mostly off screen (a cave's ceiling, the walls behind the
        // eye), and learning only where the eye looks left them empty, their light the lightmap's
        // flat sky (reports/Path tracing research.md, 1).
        if b.hit && !b.clear {
            let bd = length(b.rel);
            let d2 = pt_cosine(b.n, pt_noise(px + vec2(19.0, 23.0)));
            let b2 = rt_hit(b.rel + b.n * (0.01 + bd * 3e-4), d2, 96.0, 2.0);
            var found = vec3(0.0);
            if b2.hit {
                found = pt_lit(b2, l, pt_noise(px + vec2(29.0, 31.0)));
            } else {
                let dark_b = light_on(b.n, vec4(0.0), 0.0, 1.0, b.world, b.rel);
                found = max(light_on(b.n, vec4(1.0, 0.0, 0.0, 0.0), 0.0, 1.0, b.world, b.rel) - dark_b, vec3(0.0));
            }
            let j = pt_cache_find(pt_cell(b.world, b.n, bd, pt_noise(px + vec2(37.0, 41.0))), true);
            if j >= 0 {
                pt_cache_add(u32(j), found);
            }
        }
    }
    if pt.debug == 10u {
        pt_out[at] = vec4(diffuse, 1.0);
        return;
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
        if a < 0.5 && horizon > 0.0 && pt.debug != 11u {
            var dir = rt_cone(r, 0.01 + a * 0.5, px + vec2(17.0, 29.0));
            if dot(dir, ng) <= 0.0 {
                dir = r;
            }
            let m = rt_hit(h.rel + lift, dir, 384.0, a * 6.0);
            var seen = sky_color(dir);
            specular_t = PT_SKY_T;
            if m.hit {
                seen = haze_lit(pt_lit(m, l, pt_noise(px + vec2(13.0, 2.0))), dir * m.t, m.light.x);
                specular_t = m.t;
            }
            env = mix(seen, env, smoothstep(0.1, 0.5, a));
        }
        let gloss = 1.0 - clamp(a * 1.6, 0.0, 0.85);
        specular = (pt_schlick(f0, nv) * env * gloss * horizon + shine) / max(s_alb, vec3(1e-3));
    }
    // Toward the edge, the world's own light (the lightmap's sky, no bounce) takes over, so the land
    // past `PT_REACH` follows on without a step: traced, the near land is darker and greener than
    // the world draws it (pt-denoisers' spot C check, 11-06).
    let edge = smoothstep(PT_REACH * 0.75, PT_REACH, dist);
    diffuse = mix(diffuse, lit, edge);
    specular *= 1.0 - edge;
    // Rain on the ground as the world's pass has it (`rain_on_ground`: wet, darker, the sky's sheen,
    // puddles): traced, the ground stayed dry and bright in the rain (w27). It is affine in the
    // colour: its factor on both lights, what it adds into the specular.
    if ground_wet() > 0.0 || scene.canopy.z > 0.0 {
        let leafy = h.quad || h.tint == 2u; // (2: foliage, `world.wgsl` TINT_FOLIAGE)
        let added = rain_on_ground(vec3(0.0), n, h.rel, h.world, h.light.x, leafy);
        let k = rain_on_ground(vec3(1.0), n, h.rel, h.world, h.light.x, leafy) - added;
        diffuse *= k;
        specular = specular * k + added / max(s_alb, vec3(1e-3));
    }
    if pt.debug != 0u && pt.debug < 7u {
        var out = vec3(0.0);
        switch pt.debug {
            case 1u: { out = albedo; }
            case 2u: { out = n * 0.5 + 0.5; }
            case 3u: { out = albedo * (lit - sky_part); }
            case 4u: { out = bounce; }
            case 5u: { out = specular * s_alb; }
            case 6u: { out = albedo * pt_cached(h.world, ng, dist, pt_noise(px + vec2(3.0, 11.0))).rgb; }
            default: {}
        }
        pt_out[at] = vec4(out, 0.0);
        pt_out[at + 2u] = vec4(0.0, 0.0, 0.0, 2.0);
        return;
    }
    // The picture's own (mode 0): the surface's light through the air as the world's pass puts it
    // (`haze_lit` on the colour itself) and finished as it finishes it (`display`: the night's
    // blue, or the LDR tone curve). As an affine map from white (`through`) the air's greying in
    // the rain, which mixes each colour toward its own grey, was lost: the traced ground stayed
    // warm where the world's went grey (w29). One call, where the map took two.
    // Under a denoiser (mode 1) the air as that map: its light alone (`under`, which
    // `fs_pt_finish` adds the denoised light to and finishes) and what it lets through.
    NIGHT_SKY_SHARE = share;
    var dif = diffuse;
    var spe = specular;
    var under: vec3<f32>;
    if pt.mode == 1u {
        let air = haze_lit(vec3(0.0), h.rel, h.light.x);
        let through = haze_lit(vec3(1.0), h.rel, h.light.x) - air;
        dif *= through;
        spe *= through;
        under = air;
    } else {
        let surface = diffuse * albedo + specular * s_alb;
        // (Debug 13, timing only: without the air; a branch, as a `select` works out both.)
        if pt.debug == 13u {
            under = display(surface);
        } else {
            under = display(haze_lit(surface, h.rel, h.light.x));
        }
    }
    // The light alone for the denoisers (with the air's share through it under one, mode 1).
    pt_out[at] = vec4(dif, diffuse_t);
    pt_out[at + 1u] = vec4(spe, specular_t);
    // The share rides in the kind (1 to 1.25: traced) for `fs_pt_finish`.
    pt_out[at + 2u] = vec4(under, 1.0 + 0.25 * share);
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

// After a denoiser: each traced pixel the air's light (`pt_in` [2]) and the denoised light it
// wrote (`textures[pt.light]`), finished as the world's pass finishes (`display`).
@fragment
fn fs_pt_finish(in: PtOut) -> @location(0) vec4<f32> {
    let px = vec2<u32>(in.clip.xy);
    let kind = pt_in[(px.y * pt.size.x + px.x) * 3u + 2u];
    if kind.a < 0.5 || kind.a > 1.5 {
        discard;
    }
    let light = textureLoad(textures[pt.light], vec2<i32>(px), 0).rgb;
    // The pixel's sky share as `cs_trace` found it: left at 1, the whole denoised light went
    // the night's blue, a fire's with it.
    NIGHT_SKY_SHARE = clamp((kind.a - 1.0) * 4.0, 0.0, 1.0);
    return vec4(display(kind.rgb + light), 0.0);
}
