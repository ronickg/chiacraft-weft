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
// whether the light can be seen. Each pixel draws a few candidates from the light-giving blocks
// that reach its cube (`pt_lights`, from the meshes, listed per cube on the CPU), each weighted by the light it gives there
// by the lightmap's own falloff (its level less the blocks between), keeps one in proportion
// (resampled importance sampling), and sends one shadow ray to it. Averaged over frames, that is the
// share of the block light that reaches the surface: shadows behind pillars and furniture.

const PT_LIGHT_CANDIDATES: u32 = 8u;
// Half the size of a torch's flame and glow, blocks: what its shadows soften by.
const PT_FLAME: f32 = 0.2;
// The share of the lightmap's block light a shadow takes away. The rest stays: the light's further
// bounces, which the lightmap's spread stands for and one traced bounce leaves out (all of it taken,
// a block's side away from the torch came out near black in a stone cave, w59; 70 % read faint,
// Ronald chose 90 %, 11-25).
const PT_BLOCK_SHADOW: f32 = 0.9;
// The projection's near plane (`world_render`'s perspective, 0.05): the primary test starts there.
const PT_NEAR: f32 = 0.05;

// Where the lights reaching cube `c` are in `pt_lights` (start, count); count 0 when none do.
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
// x 1 seen, 0 hidden, by one light drawn as above (1 when none is near); y 1 when a light was drawn
// (else why not, for debug view 15: 0 no list for the cube, -1 none of the draws in reach, -2 no lights at all).
fn pt_block_visibility(world: vec3<f32>, rel: vec3<f32>, ng: vec3<f32>, lift: vec3<f32>, px: vec2<f32>) -> vec2<f32> {
    if pt_lights[1] == 0u {
        return vec2(1.0, -2.0);
    }
    let cell = vec3<i32>(floor(world + ng * 0.5));
    let cube = cell >> vec3(4u);
    let base = 2u + pt_lights[0] * 4u;
    // The lights that reach into this cube (the table lists them per cube, strongest first).
    let r = pt_light_find(cube);
    let total = r.y;
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
        let l = pt_lights[base + r.x + t];
        let lp = vec3<i32>(i32(l & 63u), i32((l >> 6u) & 63u), i32((l >> 12u) & 63u)) - 16 + cube * 16;
        let e = f32(max((l >> 18u) & 15u, max((l >> 22u) & 15u, (l >> 26u) & 15u)));
        let d = abs(lp - cell);
        let level = e - f32(d.x + d.y + d.z);
        if level <= 0.0 {
            continue;
        }
        // Its weight over the chance of drawing it (1 in `total`).
        let w = level * f32(total);
        w_sum += w;
        if pt_rand(px, 17u + k * 2u).x * w_sum < w {
            // A point on the light, a new one each frame (the denoiser averages them): a torch's
            // flame (bit 30) a small box above its middle, a glowing block its whole. The shadow's
            // edge softens with its distance from what casts it.
            let small = ((l >> 30u) & 1u) == 1u;
            let j = vec3(pt_rand(px, 40u + k).xy, pt_rand(px, 48u + k).x) * 2.0 - 1.0;
            chosen = vec3<f32>(lp) + select(vec3(0.5) + j * 0.45, vec3(0.5, 0.6, 0.5) + j * PT_FLAME, small);
            own = all(lp == vec3<i32>(floor(world - ng * 0.5)));
            found = true;
        }
    }
    if !found {
        return vec2(1.0, -1.0);
    }
    if own {
        return vec2(1.0, 1.0);
    }
    var to = chosen - world;
    // A light behind the face lights it only round a corner, as the lightmap spreads it: hidden.
    // But one in the layer of the surface's own block (a fire in a hole in the floor, a lamp set in
    // a wall) shines out of its open face over the surface, as the lightmap spreads it from there:
    // its point is brought out to just over the face. Drawn inside its block, every point lay under
    // the floor's plane, and the floor round the nave's fire pits kept a tenth of its light (w76).
    let behind = -dot(to, ng);
    if behind >= 0.0 {
        if behind > 1.0 {
            return vec2(0.0, 1.0);
        }
        to += ng * (behind + 0.05);
    }
    let dist = length(to);
    let dir = to / max(dist, 1e-4);
    let hit = rt_hit(rel + lift, dir, max(dist - 0.9, 0.0), 0.0);
    return vec2(select(1.0, 0.0, hit.hit && !hit.clear), 1.0);
}

// ---- the lamps' own light (stage 2 of the Remix match, `core:pt_emissive` above 0) ----
//
// In place of the lightmap's block light (its level falling a step a block, then shadowed), the
// light from the lamp itself: I colour cos / (d^2 + r^2), I the light's level times its strength
// (`light_strength`: a torch 1, fire 2.5, lava 2, glowstone 1.5) times `core:pt_emissive`, r its
// size (a flame's small, a glowing block's half a block). Calibrated so that `PT_NEE_AT` blocks off
// (fewer for a dim light) a light of strength 1 gives what the lightmap gives there (the look's own
// curve and colour: `light_on` of the light's level less that), and from there it falls off as light
// does, nearer up to `PT_NEE_HOT` times that. (Matched 1 block off, a torch lit a pool a block or
// two wide and the room went dark past it, s2b cave_torch.) The candidates are drawn as
// above (8, then one in proportion to its estimate here: resampled importance sampling, so a near
// fire beats a far torch) and one shadow ray goes to it. Where no light of the cube's list reaches,
// the lightmap's block light stays (lights past the lists, the far land).

// The light's size squared (blocks): a glowing block's, a flame's.
const PT_NEE_R2_BLOCK: f32 = 0.25;
const PT_NEE_R2_FLAME: f32 = 0.04;
// Where the lamps' light matches the lightmap's (blocks off the light), and how much brighter it
// may get nearer.
const PT_NEE_AT: f32 = 5.0;
const PT_NEE_HOT: f32 = 3.0;
// The most the lamps' light gives a surface, its brightest channel, its hue kept (the world's light
// units: 1 a full lightmap). Past it the display's curve took red to 1 and green after it, and the
// fire's orange went yellow-white on the pillar and wall (s2c fire18000).
const PT_NEE_MAX: f32 = 1.5;
// How far the normal map may change the lamps' light (a share either way): the cosine is the face's,
// times 1 plus the map's difference to it, within this. (Its cosine itself, even 0.3 of its tilt,
// mottled a wall the fire's light grazed: s2a and s2b fire18000.)
const PT_NEE_DETAIL: f32 = 0.2;

// The falloff at `d2` (squared distance) from a light of size `r2` whose match lies `at` blocks off.
fn pt_nee_falloff(d2: f32, r2: f32, at: f32) -> f32 {
    return min((at * at + r2) / (d2 + r2), PT_NEE_HOT);
}

// Over how many levels at the end of its reach a lamp's light fades out.
const PT_NEE_FADE: f32 = 3.0;

// How much of a light of level `e` reaches a surface `to` off its middle (1 well within the
// lightmap's reach, 0 past it), by the surface's own distance in blocks along the axes.
fn pt_nee_fade(e: f32, to: vec3<f32>) -> f32 {
    let d = max(abs(to) - 0.5, vec3(0.0));
    return clamp((e - (d.x + d.y + d.z)) / PT_NEE_FADE, 0.0, 1.0);
}

// How far above the lightmap's own block light at a surface a field of lamps' light may sum (a lamp
// with two or more glowing neighbours, bit 31 of its listed word: a lava pit, a glowstone wall).
// Each lamp is matched to the lightmap as if alone, and the lightmap takes the strongest light
// where tracing adds them up: a 10 x 7 lava pit lit the floor round it white, a third over the
// room's lightmap (s2o lava_glow). A lamp alone keeps its own light (capped for all, it took a
// third of the alcove fire's off its pillar and wall, s2p fire18000). Shadows still take the
// light down freely.
const PT_NEE_SUM: f32 = 2.0;
// Whether the lamp just shaded (`pt_block_direct`) is one of a field.
var<private> PT_NEE_FIELD: bool = false;

fn pt_nee_sum_cap(c: vec3<f32>, block: vec3<f32>) -> vec3<f32> {
    let w = vec3(0.2126, 0.7152, 0.0722);
    let most = PT_NEE_SUM * dot(block, w);
    return c * min(1.0, most / max(dot(c, w), 1e-6));
}

// The lamps' light within `PT_NEE_MAX`, its hue kept.
fn pt_nee_cap(c: vec3<f32>) -> vec3<f32> {
    return c * min(1.0, PT_NEE_MAX / max(max(c.r, max(c.g, c.b)), 1e-4));
}

// The cosine for the lamps' light: the face's own, with the normal map's detail within
// `PT_NEE_DETAIL`.
fn pt_nee_cos(n: vec3<f32>, ng: vec3<f32>, dir: vec3<f32>) -> f32 {
    let c = dot(ng, dir);
    return max(c, 0.0) * clamp(1.0 + dot(n, dir) - c, 1.0 - PT_NEE_DETAIL, 1.0 + PT_NEE_DETAIL);
}

// A lamp as drawn: its block, its word in the lists (levels, the flame bit) and its strength.
struct PtLamp {
    lp: vec3<i32>,
    word: u32,
    strength: f32,
}

// What a lamp is likely to give a surface at `world` (normals `n`, `ng`): the draws' weight, as the
// light itself (`pt_block_direct`) without its colour or its shadow; 0 none.
fn pt_nee_target(world: vec3<f32>, n: vec3<f32>, ng: vec3<f32>, lamp: PtLamp) -> f32 {
    let l = lamp.word;
    let e = f32(max((l >> 18u) & 15u, max((l >> 22u) & 15u, (l >> 26u) & 15u)));
    let small = ((l >> 30u) & 1u) == 1u;
    let fade = pt_nee_fade(e, vec3<f32>(lamp.lp) + 0.5 - world);
    if fade <= 0.0 {
        return 0.0;
    }
    if all(lamp.lp == vec3<i32>(floor(world - ng * 0.5))) {
        // The surface is the light's own block.
        return lamp.strength * e / 15.0 * PT_NEE_HOT * fade;
    }
    var to = vec3<f32>(lamp.lp) + select(vec3(0.5), vec3(0.5, 0.6, 0.5), small) - world;
    let behind = -dot(to, ng);
    if behind > 1.0 {
        return 0.0;
    }
    if behind >= 0.0 {
        to += ng * (behind + 0.05);
    }
    let d2 = dot(to, to);
    let r2 = select(PT_NEE_R2_BLOCK, PT_NEE_R2_FLAME, small);
    let cs = pt_nee_cos(n, ng, to * inverseSqrt(max(d2, 1e-6)));
    if cs <= 0.0 {
        return 0.0;
    }
    return lamp.strength * e / 15.0 * cs * pt_nee_falloff(d2, r2, clamp(e - 1.0, 1.0, PT_NEE_AT)) * fade;
}

// ---- the lamps' draws reused (ReSTIR, `core:pt_restir`: `PtArgs::flags` bit 1) ----
//
// Each pixel keeps the lamp it drew and that draw's weight (a reservoir, 16 bytes: the lamp's block
// from the surface's and its word, its strength and how many draws it stands for, its weight, and
// which block face it was for). Before its shadow ray it merges last frame's at its own pixel and
// at two near ones, each kept only for the same face (its own) or a face of the same way within 2
// blocks (a neighbour's), weighed again for this surface: many more lamps tried for the same one
// ray, so less noise where several light a room. (Biased as the cheap ReSTIR is: no visibility in
// the weights; the denoiser keeps the rest smooth.)

// How many draws a reservoir may stand for (about 8 frames of fresh draws).
const PT_RES_CAP: f32 = 64.0;
// How far off the near pixels a reservoir reuses lie (pixels, each way).
const PT_RES_NEAR: f32 = 6.0;
// The draws the pixel's reservoir stood for this frame (debug view 20).
var<private> PT_RES_M: f32 = 0.0;
// Debug view 21: for the pixel's first fresh draw, which of `pt_nee_target`'s tests pass (red in
// reach, green not behind the face, blue facing it).
var<private> PT_NEE_DBG: vec3<f32> = vec3(0.0);

fn pt_nee_tests(world: vec3<f32>, n: vec3<f32>, ng: vec3<f32>, lamp: PtLamp) -> vec3<f32> {
    let l = lamp.word;
    let e = f32(max((l >> 18u) & 15u, max((l >> 22u) & 15u, (l >> 26u) & 15u)));
    let small = ((l >> 30u) & 1u) == 1u;
    let fade = pt_nee_fade(e, vec3<f32>(lamp.lp) + 0.5 - world);
    var to = vec3<f32>(lamp.lp) + select(vec3(0.5), vec3(0.5, 0.6, 0.5), small) - world;
    let behind = -dot(to, ng);
    if behind >= 0.0 {
        to += ng * (behind + 0.05);
    }
    let cs = pt_nee_cos(n, ng, normalize(to));
    return vec3(select(0.0, 1.0, fade > 0.0), select(0.0, 1.0, behind <= 1.0), select(0.0, 1.0, cs > 0.0));
}

// The face a reservoir is for: its block's low 9 bits an axis, the face's way (1-6).
fn pt_res_check(cell: vec3<i32>, ng: vec3<f32>) -> u32 {
    let a = abs(ng);
    var face = select(1u, 2u, ng.x > 0.0);
    if a.y > a.x && a.y >= a.z {
        face = select(3u, 4u, ng.y > 0.0);
    } else if a.z > a.x && a.z > a.y {
        face = select(5u, 6u, ng.z > 0.0);
    }
    let c = vec3<u32>(cell) & vec3(511u);
    return c.x | (c.y << 9u) | (c.z << 18u) | (face << 27u);
}

// A reservoir's block less this one's (each axis wrapped in 9 bits).
fn pt_res_diff(theirs: u32, mine: u32) -> vec3<i32> {
    let d = (vec3<i32>(vec3(theirs, theirs >> 9u, theirs >> 18u) & vec3(511u)) - vec3<i32>(vec3(mine, mine >> 9u, mine >> 18u) & vec3(511u)) + 256) & vec3(511);
    return d - 256;
}

// The lamps' light at a surface at `world` (`rel` camera relative), its shading normal `n` and
// face's `ng`: rgb in the world's light units, a as `pt_block_visibility`'s y (1 lamps in the
// list, drawn or none lighting this face; 0 no list for the cube, -2 no lights at all). `at`: the pixel whose reservoir
// it keeps and reuses (`0xffffffff` none: the bounce's surface), `pix` that pixel.
fn pt_block_direct(world: vec3<f32>, rel: vec3<f32>, n: vec3<f32>, ng: vec3<f32>, lift: vec3<f32>, px: vec2<f32>, at: u32, pix: vec2<i32>) -> vec4<f32> {
    let reuse = at != 0xffffffffu && (pt.flags & 2u) != 0u;
    PT_RES_M = 0.0;
    if pt_lights[1] == 0u {
        return vec4(0.0, 0.0, 0.0, -2.0);
    }
    let cell = vec3<i32>(floor(world + ng * 0.5));
    let cube = cell >> vec3(4u);
    let base = 2u + pt_lights[0] * 4u;
    let strengths = base + pt_lights[1];
    let r = pt_light_find(cube);
    let total = r.y;
    var y = PtLamp(vec3(0), 0u, 0.0);
    var y_p = 0.0;
    var w_sum = 0.0;
    var m = 0.0;
    var found = false;
    // Fresh draws among the cube's list, each as likely.
    if total > 0u {
        for (var k = 0u; k < PT_LIGHT_CANDIDATES; k++) {
            let t = min(u32(pt_rand(px, 16u + k * 2u).x * f32(total)), total - 1u);
            let l = pt_lights[base + r.x + t];
            let lamp = PtLamp(vec3<i32>(i32(l & 63u), i32((l >> 6u) & 63u), i32((l >> 12u) & 63u)) - 16 + cube * 16, l, f32(pt_lights[strengths + r.x + t]) * 0.25);
            m += 1.0;
            if k == 0u && pt.debug == 21u {
                PT_NEE_DBG = pt_nee_tests(world, n, ng, lamp) * select(1.0, 0.5, lamp.strength <= 0.0);
            }
            let p = pt_nee_target(world, n, ng, lamp);
            if p <= 0.0 {
                continue;
            }
            // Its weight over the chance of drawing it (1 in `total`).
            let w = p * f32(total);
            w_sum += w;
            if pt_rand(px, 17u + k * 2u).x * w_sum < w {
                y = lamp;
                y_p = p;
                found = true;
            }
        }
    }
    // Last frame's at this pixel and two near ones.
    let check = pt_res_check(cell, ng);
    // Each reused reservoir's draws and block (for the count below).
    var res_m = array<f32, 3>(0.0, 0.0, 0.0);
    var res_dc = array<vec3<i32>, 3>(vec3(0), vec3(0), vec3(0));
    if reuse {
        for (var j = 0u; j < 3u; j++) {
            var q = pix;
            if j > 0u {
                let o = (pt_rand(px, 60u + j).xy * 2.0 - 1.0) * PT_RES_NEAR;
                q = clamp(pix + vec2<i32>(round(o)), vec2(0), vec2<i32>(pt.size) - 1);
            }
            let v = pt_res_prev[u32(q.y) * pt.size.x + u32(q.x)];
            if v.w == 0u || (v.w >> 27u) != (check >> 27u) {
                continue;
            }
            let dc = pt_res_diff(v.w, check);
            if (j == 0u && any(dc != vec3(0))) || any(abs(dc) > vec3(2)) {
                continue;
            }
            let lamp = PtLamp(cell + dc + vec3<i32>(i32(v.x & 63u), i32((v.x >> 6u) & 63u), i32((v.x >> 12u) & 63u)) - 32, v.x, f32(v.y & 15u) * 0.25);
            let mq = min(f32(v.y >> 4u), PT_RES_CAP);
            res_m[j] = mq;
            res_dc[j] = dc;
            let p = pt_nee_target(world, n, ng, lamp);
            let w = p * bitcast<f32>(v.z) * mq;
            if w <= 0.0 {
                continue;
            }
            w_sum += w;
            if pt_rand(px, 70u + j).x * w_sum < w {
                y = lamp;
                y_p = p;
                found = true;
            }
        }
    }
    if !found || y_p <= 0.0 {
        PT_RES_M = m;
        // Lamps in the list but none lighting this face (all behind it or out of reach): none of
        // their light, not the lightmap's. The lightmap's floods round a wall, so falling back to
        // it lit a wall's near face from a torch behind it (s2l wall); it stays only where the
        // cube has no list (`0`).
        return vec4(0.0, 0.0, 0.0, select(1.0, 0.0, total == 0u));
    }
    // The draws the chosen lamp stands for: this pixel's fresh ones, and each reused reservoir's
    // only where that lamp could light the reservoir's own surface (its block, this face). Counting
    // every reservoir's draws darkened the fires' light by 5-20 % (s2h, s2j nave12800 and
    // fire18000): a near surface that can't see the chosen lamp (it lies behind it) could never
    // have drawn it.
    for (var j = 0u; j < 3u; j++) {
        if res_m[j] > 0.0 && (all(res_dc[j] == vec3(0)) || pt_nee_target(world + vec3<f32>(res_dc[j]), n, ng, y) > 0.0) {
            m += res_m[j];
        }
    }
    PT_RES_M = m;
    // (RIS: what was chosen, over its estimate, times the draws' mean weight.)
    let ris = w_sum / (m * y_p);
    let l = y.word;
    PT_NEE_FIELD = (l >> 31u) == 1u;
    let lp = y.lp;
    if reuse {
        let o = lp - cell + 32;
        if all(o >= vec3(0)) && all(o < vec3(64)) {
            let word = u32(o.x) | (u32(o.y) << 6u) | (u32(o.z) << 12u) | (l & 0xFFFC0000u);
            let q = u32(round(y.strength * 4.0)) & 15u;
            pt_res[at] = vec4(word, q | (u32(min(m, PT_RES_CAP)) << 4u), bitcast<u32>(ris), check);
        }
    }
    let small = ((l >> 30u) & 1u) == 1u;
    let fade = pt_nee_fade(f32(max((l >> 18u) & 15u, max((l >> 22u) & 15u, (l >> 26u) & 15u))), vec3<f32>(lp) + 0.5 - world);
    let chosen_s = y.strength * fade;
    // Its colour where it matches the lightmap, in the world's light (the look's curve and tint),
    // facing it.
    let level = vec3(f32((l >> 18u) & 15u), f32((l >> 22u) & 15u), f32((l >> 26u) & 15u));
    let at_d = clamp(max(level.r, max(level.g, level.b)) - 1.0, 1.0, PT_NEE_AT);
    let up = vec3(0.0, 1.0, 0.0);
    // Its hue as the lightmap has it a block off, its brightness as there: matched `at_d` blocks
    // off the colour itself drifted (a fire's green and blue run out first).
    let dark = light_on(up, vec4(0.0), 0.0, 1.0, world, rel);
    let near = max(light_on(up, vec4(0.0, max(level - 1.0, vec3(0.0)) / 15.0), 0.0, 1.0, world, rel) - dark, vec3(0.0));
    let far = max(light_on(up, vec4(0.0, max(level - at_d, vec3(0.0)) / 15.0), 0.0, 1.0, world, rel) - dark, vec3(0.0));
    let one_off = near * (dot(far, vec3(0.2126, 0.7152, 0.0722)) / max(dot(near, vec3(0.2126, 0.7152, 0.0722)), 1e-4));
    if all(lp == vec3<i32>(floor(world - ng * 0.5))) {
        return vec4(pt_nee_cap(one_off * chosen_s * PT_NEE_HOT * ris * pt.emissive), 1.0);
    }
    // A point on the light, a new one each frame (as `pt_block_visibility`), and one shadow ray.
    let jit = vec3(pt_rand(px, 40u).xy, pt_rand(px, 48u).x) * 2.0 - 1.0;
    var to = vec3<f32>(lp) + vec3(0.5, 0.6, 0.5) + jit * PT_FLAME - world;
    var short = 0.9;
    // How far the light is for its falloff: from the block's middle, as `pt_nee_target` and the
    // 5-block match measure it (only which point the ray aims at moves to a face).
    var mid = to;
    if !small {
        // A glowing block shines from its faces: one of the three facing this surface, drawn by
        // how much of it faces it (so on average the whole block's light), a point on it, and
        // the ray stopping at it. A face buried in the floor or ceiling then shades as it should
        // and an open one lights even at a grazing look; aimed at a point inside the block, the
        // ray's last stretch ran through the floor or ceiling beside lava in a pit or glowstone
        // in a ceiling, which took the pit's light and cut hard wedges into the walls (s2i
        // lava_glow).
        let c = vec3<f32>(lp) + 0.5 - world;
        mid = c;
        let mb = -dot(c, ng);
        if mb >= 0.0 {
            mid += ng * (mb + 0.05);
        }
        let a = abs(c);
        let u = pt_rand(px, 56u).x * (a.x + a.y + a.z);
        var axis = vec3(0.0, 0.0, 1.0);
        if u < a.x {
            axis = vec3(1.0, 0.0, 0.0);
        } else if u < a.x + a.y {
            axis = vec3(0.0, 1.0, 0.0);
        }
        to = c - axis * sign(c) * 0.5 + (vec3(1.0) - axis) * jit * 0.48;
        short = 0.03;
    }
    let behind = -dot(to, ng);
    if behind >= 0.0 {
        to += ng * (behind + 0.05);
    }
    let d2 = dot(to, to);
    let dist = sqrt(d2);
    let dir = to / max(dist, 1e-4);
    let cs = pt_nee_cos(n, ng, dir);
    if cs <= 0.0 {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    let hit = rt_hit(rel + lift, dir, max(dist - short, 0.0), 0.0);
    if hit.hit && !hit.clear {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    let r2 = select(PT_NEE_R2_BLOCK, PT_NEE_R2_FLAME, small);
    let f = one_off * chosen_s * cs * pt_nee_falloff(select(d2, dot(mid, mid), !small), r2, at_d);
    return vec4(pt_nee_cap(f * ris * pt.emissive), 1.0);
}

// ---- plain (`core:pt_plain`): vanilla's math, and only what tracing adds to it ----
//
// The world's pass without a pack (the engine's `world.wgsl`): the texel times its tint, its face's
// shade and the lightmap's curve, multiplied in sRGB values as vanilla does; no grade, no haze, no
// specular. Tracing adds two things: the sky level the lightmap's flood fill guesses becomes the
// sky's share the surface sees (a ray that gets out counts whole, one that meets a surface counts
// that surface's own sky light, so a wall over open ground stays as vanilla's and a cave's goes
// dark by its shape), and the sun's shadow (a face the traced sun misses loses `PT_PLAIN_SUN` of
// the daylight). The block light is shadowed as in the look's path. For Ronald's stage-1 compare
// (ours plain against vanilla): PT that differs from vanilla only where tracing does.
const PT_PLAIN_SUN: f32 = 0.3;

fn pt_srgb_encode(l: vec3<f32>) -> vec3<f32> {
    let c = max(l, vec3(0.0));
    return select(1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3(0.0031308));
}

fn pt_srgb_decode(s: vec3<f32>) -> vec3<f32> {
    let c = max(s, vec3(0.0));
    return select(pow((c + 0.055) / 1.055, vec3(2.4)), c / 12.92, c <= vec3(0.04045));
}

// Vanilla's lightmap (`common.wgsl`) at `light` (sky, then the block light's channels, 0..1).
fn pt_lightmap(light: vec4<f32>) -> vec3<f32> {
    return lightmap(light, scene.params.z, scene.light.x, scene.light.y, scene.light.w, scene.environment.x);
}

// The plain pixel at `at` (see above): `h` the surface met, `n` its normal, `ng` its face's, `sun`
// the traced sun on it (0..1, 0 facing away), `albedo` the G-buffer's (linear, tinted).
fn pt_plain(at: u32, px: vec2<f32>, h: RtHit, ng: vec3<f32>, lift: vec3<f32>, sun: f32, albedo: vec3<f32>, glow: f32) {
    // The sky's share: one cosine ray a frame about the face (the denoisers average them).
    let b = rt_hit(h.rel + lift, pt_cosine(ng, pt_noise(px)), 96.0, 2.0);
    let access = select(1.0, clamp(b.light.x, 0.0, 1.0), b.hit);
    let blk = h.light.yzw;
    let none = pt_lightmap(vec4(0.0));
    var light = mix(pt_lightmap(vec4(0.0, blk)), pt_lightmap(vec4(1.0, blk)), access);
    // The sun's shadow on the daylight the surface sees.
    let day = max(pt_lightmap(vec4(1.0, 0.0, 0.0, 0.0)) - none, vec3(0.0));
    light -= day * access * PT_PLAIN_SUN * clamp(light_strength(), 0.0, 1.0) * (1.0 - clamp(sun, 0.0, 1.0));
    // The block light, as much of it as reaches the surface.
    let block = max(pt_lightmap(vec4(0.0, blk)) - none, vec3(0.0));
    if max(block.r, max(block.g, block.b)) > 1e-4 {
        let vis = pt_block_visibility(h.world, h.rel, ng, lift, px).x;
        light -= block * ((1.0 - vis) * PT_BLOCK_SHADOW);
    }
    // Debug 17 and 18 (plain only): the sky's share as traced (grey), the traced sun on the face
    // (yellow; blue where the face looks away from it).
    if pt.debug == 17u || pt.debug == 18u {
        var out = vec3(access);
        if pt.debug == 18u {
            out = select(vec3(0.0, 0.0, 0.3), vec3(1.0, 0.9, 0.0) * clamp(sun, 0.0, 1.0), dot(ng, light_dir()) > 0.0);
        }
        pt_out[at] = vec4(out, 0.0);
        pt_out[at + 2u] = vec4(0.0, 0.0, 0.0, 2.0);
        return;
    }
    light = clamp(light, vec3(0.0), vec3(1.0)) * normal_shade(ng);
    // A glowing texel at least its glow, as the engine's `max(light, tint * e)` (the G-buffer's
    // alpha under plain, `gbuffer` in world.wgsl; the tint is in the albedo): lit as plain
    // colour a campfire's flame came out dimmer than raster's (fire2, 11-26).
    light = max(light, vec3(glow));
    let surface = pt_srgb_decode(pt_srgb_encode(albedo) * light);
    // The light alone for the denoisers (the surface over its albedo), no specular, no air.
    pt_out[at] = vec4(surface / max(albedo, vec3(1e-4)), select(PT_SKY_T, max(b.t, 1e-3), b.hit));
    pt_out[at + 1u] = vec4(0.0);
    pt_out[at + 2u] = vec4(select(surface, vec3(0.0), pt.mode == 1u), 1.0);
}

@compute @workgroup_size(8, 8)
fn cs_trace(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= pt.size.x || id.y >= pt.size.y {
        return;
    }
    let at = (id.y * pt.size.x + id.x) * 3u;
    // No lamp kept here unless the lamps' light below keeps one (`pt_block_direct`).
    pt_res[id.y * pt.size.x + id.x] = vec4(0u);
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
    // From the near plane (the projection's 0.05), as the world draws: a face through the eye
    // itself is clipped there, and traced from the eye it stopped every ray crossing it. A slab made
    // a full block by the showcase import's full shapes stood against the nave's camera, and half
    // the screen kept the world's pixel (debug 16 blue, the parity5 nave's seam, 11-25).
    // And past a face met from behind (the world culls back faces, the rays don't): the eye on a
    // block's corner starts the ray inside that block, and it met the block's inner faces first.
    var h = rt_hit_mask(-v * PT_NEAR, -v, dist * 1.02 + 0.2 - PT_NEAR, 0.0, 3u);
    for (var k = 0; k < 2; k++) {
        let met = length(h.rel);
        if !h.hit || h.clear || dot(h.n, v) >= 0.0 || met >= dist - 0.02 * dist - 0.15 {
            break;
        }
        h = rt_hit_mask(h.rel - v * 0.01, -v, dist * 1.02 + 0.2 - met - 0.01, 0.0, 3u);
    }
    // Terrain at the depth drawn there: anything else (water, glass, a model, a particle) keeps the
    // world's own pixel.
    if !h.hit || h.clear || abs(length(h.rel) - dist) > 0.02 * dist + 0.15 {
        // Debug 16: why the trace kept the world's pixel: red no hit, green a clear pane, blue
        // a hit nearer than the depth, yellow one farther (the nave's half-screen seam, 11-25).
        if pt.debug == 16u {
            var why = vec3(1.0, 0.0, 0.0);
            if h.hit && h.clear {
                why = vec3(0.0, 1.0, 0.0);
            } else if h.hit {
                why = select(vec3(1.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0), length(h.rel) < dist);
            }
            pt_out[at] = vec4(why, 0.0);
            pt_out[at + 2u] = vec4(0.0, 0.0, 0.0, 2.0);
        }
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
        if pt.debug == 16u {
            pt_out[at] = vec4(1.0, 0.0, 1.0, 0.0);
            pt_out[at + 2u] = vec4(0.0, 0.0, 0.0, 2.0);
        }
        return;
    }
    // Debug 8-12, timing only (a bench's probes): the trace stopped after its primary test (8), the
    // sun's ray (9), the bounce and the cache (10), or with no specular ray (11); 12 before any ray, above; 13 the whole trace without the air's
    // two `haze_lit`s. The picture they
    // leave is no use.
    if pt.debug == 8u {
        return;
    }
    // Debug 16: grey where the trace runs (the reasons it doesn't are above).
    if pt.debug == 16u {
        pt_out[at] = vec4(0.2, 0.2, 0.2, 0.0);
        pt_out[at + 2u] = vec4(0.0, 0.0, 0.0, 2.0);
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
    if pt.plain != 0u {
        pt_plain(at, px, h, ng, lift, sun, albedo, textureLoad(textures[pt.albedo], ip, 0).a);
        return;
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
        // Stage 2 step 2 (`core:pt_bounce_lights`, flags bit 0): the surface met is lit by the
        // lamps as the pixel is (`pt_block_direct`: its own shadow ray), in place of its lightmap's
        // block light; a lamp's own block met gives its glow. The wall a fire lights lights the floor.
        if pt.emissive > 0.0 && (pt.flags & 1u) != 0u {
            let b_block = max(light_on(b.n, vec4(0.0, b.light.yzw), 0.0, 1.0, b.world, b.rel) - light_on(b.n, vec4(0.0), 0.0, 1.0, b.world, b.rel), vec3(0.0));
            if max(b_block.r, max(b_block.g, b_block.b)) > 1e-4 {
                let e = pt_block_direct(b.world, b.rel, b.n, b.n, b.n * 0.02, px + vec2(37.0, 53.0), 0xffffffffu, vec2(0));
                if e.a > 0.5 {
                    bounce += pt_albedo(b) * (select(e.rgb, pt_nee_sum_cap(e.rgb, b_block), PT_NEE_FIELD) - b_block);
                }
            }
        }
        // Above 0 on every traced pixel: the denoisers read diffuse's a > 0 as "traced".
        diffuse_t = max(b.t, 1e-3);
    }
    // A ray that gets out brings the sky's light from its own way (`light_on`'s tops 1, sides 0.75,
    // bottoms 0.5 about the ray), not the face's: the face's tilt is already in which rays get out,
    // and with the face's own share a wall counted it twice. NVIDIA's cathedral, out of the sun at
    // 07:00, came out near black against the world's 0.75 (parity5, 11-25: facade 18 % against 37 %).
    var sky_way = sky_open;
    if !b.hit {
        sky_way = max(light_on(bdir, vec4(1.0, 0.0, 0.0, 0.0), 0.0, 1.0, h.world, h.rel) - light_on(bdir, vec4(0.0), 0.0, 1.0, h.world, h.rel), vec3(0.0));
    }
    let indirect = select(sky_way, bounce, b.hit);
    var diffuse = lit - sky_part + indirect;
    // The block light, shadowed: its share of the world's light, as much of it as reaches the
    // surface (one light drawn among those near, one shadow ray; frames average it). The
    // lightmap's block light keeps its colour and reach; the held light is not a block's.
    let block = max(light_on(n, h.light, 0.0, 1.0, h.world, h.rel) - light_on(n, vec4(h.light.x, 0.0, 0.0, 0.0), 0.0, 1.0, h.world, h.rel), vec3(0.0));
    var block_vis = vec2(1.0, 0.0);
    var direct = vec3(0.0);
    if max(block.r, max(block.g, block.b)) > 1e-4 {
        if pt.emissive > 0.0 {
            // The lamps' own light in place of the lightmap's, where a light of the list reaches.
            let e = pt_block_direct(h.world, h.rel, n, ng, lift, px, id.y * pt.size.x + id.x, vec2<i32>(id.xy));
            block_vis = vec2(select(0.0, 1.0, max(e.r, max(e.g, e.b)) > 0.0), e.a);
            if e.a > 0.5 {
                direct = select(e.rgb, pt_nee_sum_cap(e.rgb, block), PT_NEE_FIELD);
                diffuse += direct - block;
            } else {
                direct = block;
            }
        } else {
            block_vis = pt_block_visibility(h.world, h.rel, ng, lift, px);
            diffuse -= block * ((1.0 - block_vis.x) * PT_BLOCK_SHADOW);
        }
    }
    // Into the cache: the indirect light found here (one pixel in four a frame).
    if ((id.x + id.y * 3u + u32(scene.rt.y)) & 3u) == 0u {
        let i = pt_cache_find(pt_cell(h.world, ng, dist, pt_noise(px + vec2(7.0, 5.0))), true);
        if i >= 0 {
            pt_cache_add(u32(i), indirect);
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
    if (pt.debug != 0u && pt.debug < 7u) || pt.debug >= 14u {
        var out = vec3(0.0);
        switch pt.debug {
            case 1u: { out = albedo; }
            case 2u: { out = n * 0.5 + 0.5; }
            case 3u: { out = albedo * (lit - sky_part); }
            case 4u: { out = bounce; }
            case 5u: { out = specular * s_alb; }
            case 6u: { out = albedo * pt_cached(h.world, ng, dist, pt_noise(px + vec2(3.0, 11.0))).rgb; }
            // 14: the lightmap's block light at the surface (as light, without the texture).
            case 14u: { out = block; }
            // 19: the lamps' light as traced (`core:pt_emissive`; the lightmap's where none reaches).
            case 19u: { out = direct; }
            // 20: the draws the lamps' reservoir stood for (ReSTIR): black 0, white `PT_RES_CAP`.
            case 20u: { out = vec3(PT_RES_M / PT_RES_CAP); }
            // 21: the first fresh draw's tests (`PT_NEE_DBG`; half bright when its strength is 0).
            case 21u: { out = PT_NEE_DBG; }
            // 15: the traced block lights: green seen, red hidden; no light drawn: blue no list for
            // the cube, cyan none in reach, magenta no lights at all; black no block light.
            case 15u: {
                if max(block.r, max(block.g, block.b)) > 1e-4 {
                    if block_vis.y > 0.5 {
                        out = select(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), block_vis.x > 0.5);
                    } else if block_vis.y > -0.5 {
                        out = vec3(0.0, 0.0, 1.0);
                    } else if block_vis.y > -1.5 {
                        out = vec3(0.0, 1.0, 1.0);
                    } else {
                        out = vec3(1.0, 0.0, 1.0);
                    }
                }
            }
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
    if pt.plain != 0u {
        return vec4(kind.rgb + light, 0.0);
    }
    return vec4(display(kind.rgb + light), 0.0);
}
