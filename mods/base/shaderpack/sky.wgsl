// chiacraft's sky: the engine's, with the pack's atmosphere behind everything, clouds lit by the
// sun and hazed into the sky, and every color brought to the screen by `display`.
// The sky, drawn after the opaque world wherever nothing covers it. Every object is one entry of
// `sky`; things that fill the screen are a single triangle, bodies a quad, stars one quad per star
// made here from their number.

var<private> CORNER: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0),
);

struct VOut {
    @builtin(position) clip: vec4<f32>,
    // Screen position (-1..1) for what fills the screen, else the direction.
    @location(0) at: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) index: u32,
    // A star's number (stars only).
    @location(3) @interpolate(flat) star: u32,
}

fn mat(s: Sky) -> mat3x3<f32> {
    return mat3x3(s.axes[0].xyz, s.axes[1].xyz, s.axes[2].xyz);
}

fn hash(v: u32) -> u32 {
    let s = v * 747796405u + 2891336453u;
    let w = ((s >> ((s >> 28u) + 4u)) ^ s) * 277803737u;
    return (w >> 22u) ^ w;
}

fn unit(v: u32) -> f32 {
    return f32(hash(v)) / 4294967295.0;
}

fn far_point(dir: vec3<f32>) -> vec4<f32> {
    // At depth 0, the far plane: the sky is behind everything.
    let c = scene.view_proj * vec4(dir * 100.0, 1.0);
    return vec4(c.xy, 0.0, c.w);
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let s = sky[ii];
    var out: VOut;
    out.index = ii;
    out.star = 0u;
    if s.ids.y == SKY_BODY {
        let c = CORNER[vi % 6u];
        // The moon's size (`base:moon_size`: 1, 1.5 or 2 times), the bodies along the moon's way.
        var half = s.extra.x;
        if dot(mat(s) * vec3(1.0, 0.0, 0.0), scene.moon.xyz) > 0.999 {
            half *= clamp(select(fantasy_opt(OPT_MOON_SIZE), 1.0, fantasy_opt(OPT_MOON_SIZE) <= 0.0), 1.0, 2.0);
        }
        let local = vec3(1.0, (0.5 - c.y) * 2.0 * half, (c.x - 0.5) * 2.0 * half);
        let dir = mat(s) * local;
        out.clip = far_point(dir);
        out.at = dir;
        out.uv = mix(s.uv.xy, s.uv.zw, c);
    } else if s.ids.y == SKY_STARS {
        // A random point on the sphere, a random size and turn, from the star's number alone.
        let star = vi / 6u + u32(s.uv.y) * 7919u;
        let z = unit(star * 4u) * 2.0 - 1.0;
        let a = unit(star * 4u + 1u) * 6.2831853;
        let r = sqrt(1.0 - z * z);
        let toward = vec3(r * cos(a), z, r * sin(a));
        // The bright ones bigger (`star_look` picks the same brightness).
        let size = s.uv.x * (0.6 + 0.4 * unit(star * 4u + 2u)) * 2.0 * (0.85 + 0.9 * pow(unit(star * 9u + 5u), 3.0));
        let turn = unit(star * 4u + 3u) * 6.2831853;
        var side = cross(toward, vec3(0.0, 1.0, 0.0));
        if dot(side, side) < 0.001 {
            side = vec3(1.0, 0.0, 0.0);
        }
        side = normalize(side);
        let up = cross(side, toward);
        let c = CORNER[vi % 6u] - vec2(0.5);
        let q = vec2(c.x * cos(turn) - c.y * sin(turn), c.x * sin(turn) + c.y * cos(turn));
        let dir = mat(s) * (toward + (side * q.x + up * q.y) * size);
        out.clip = far_point(dir);
        out.at = dir;
        out.uv = CORNER[vi % 6u];
        out.star = star;
    } else {
        let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u)) * 2.0 - 1.0;
        // The overlay of a liquid goes over everything, the hand too; depth 1 is the near plane.
        out.clip = vec4(p, select(0.0, 1.0, s.ids.y == SKY_OVERLAY), 1.0);
        out.at = vec3(p, 1.0);
        out.uv = vec2(0.0);
    }
    return out;
}

// Clouds: rounded slabs (Complementary Reimagined's idea, written anew). The picture's cells are
// blended into soft rounded shapes that thin toward the top and bottom of a slab, and each sky
// pixel's ray takes a few dithered samples through it, gathering light and cover: soft blocky puffs,
// lit from the sun above and darker underneath, in the sky's own pass.
// uv: the camera's place in its cell (x, z), the cell in the picture (x, z). extra: cell size,
// thickness, height over the camera, how far they reach. axes[0]: the weather (`sky.rs`): x cover
// as rain comes, y how dark the undersides go, z a storm's deck.
// Ten are enough with the dithered start (the dither hides the steps); 24 cost Ronald's Mac a
// great deal at Retina size (reports/performance.md).
const CLOUD_SAMPLES: u32 = 10u;
// The slab is at least this thick (blocks), whatever the layer's own thickness.
const CLOUD_THICKNESS: f32 = 9.0;
// How far one sample looks along the ray at most (blocks): far clouds take no more samples.
const CLOUD_SPAN: f32 = 400.0;

struct CloudOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) at: vec3<f32>,
    @location(1) @interpolate(flat) index: u32,
}

struct CloudPixel {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@vertex
fn vs_clouds(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> CloudOut {
    let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u)) * 2.0 - 1.0;
    var out: CloudOut;
    out.clip = vec4(p, 0.0, 1.0);
    out.at = vec3(p, 1.0);
    out.index = ii;
    return out;
}

// 1 where cell `cell` (picture cells from the camera's) is cloud: the picture's, and as rain comes
// more of the gaps (a hash per cell, so they grow as whole cells), until a storm's deck closes
// nearly all of them.
// (`look.wgsl`'s `cloud_cell_of`, which the clouds' shadows read too.)
fn cloud_cell(s: Sky, cell: vec2<f32>, dims: vec2<f32>) -> f32 {
    return cloud_cell_of(s.ids.x, cell, s.uv.zw, dims, s.axes[0].x * 0.6 + s.axes[0].z * 0.35);
}

// A storm deck's structure at `cellp`, 0..1: one smooth field about three and a half cells across
// (the picture's cells, as the wind carries them). Where it is high the deck stands up in towers,
// where it is lowest it opens into lighter breaks. Picked cell by cell (a hash), the towers' dark
// bottoms read from below as a checkered ceiling (Ronald: "Merge, then refine storm", 11-05).
fn storm_field(s: Sky, cellp: vec2<f32>) -> f32 {
    return deck_field(cellp + s.uv.zw);
}

// How far the view at this pixel looks toward the sun's side of the sky, 0 away to 1 toward it, by
// day (`fs_clouds` sets it): the rain's deck sweeps from dark masses to a brighter side there, its
// breaks wider, as Bliss's does.
var<private> SUN_SIDE: f32 = 0.0;

// How much a storm deck stands up in a tower at a field value `n`.
fn storm_tower(n: f32) -> f32 {
    return smoothstep(0.55, 0.8, n);
}

// The deck's lumps, 0..1, varying with the height in it too, so its belly darkens by depth and not
// by cell.
fn cloud_lumps(s: Sky, cellp: vec2<f32>, along: f32) -> f32 {
    let at = cellp + s.uv.zw;
    // Forms of about fifty blocks, not lumps of twenty (Ronald's "scales", 11-05).
    return noise3(vec3(at.x / 4.5, along * 1.5, at.y / 4.5)) * 0.75 + noise3(vec3(at.x / 1.8, along * 2.5 + 3.0, at.y / 1.8)) * 0.25;
}

// Henyey-Greenstein's phase for the clouds' light, `g` forward (> 0) or back (< 0).
fn cloud_hg(mu: f32, g: f32) -> f32 {
    return (1.0 - g * g) / pow(1.0 + g * g - 2.0 * g * mu, 1.5) / 12.566;
}

// Where `along` (0 at the slab's bottom, 1 at its top as tall as a storm makes it) is within the
// cloud at `cellp`: its own height 0..1, over 1 above it. A storm's towers (`storm_field`) rise
// smoothly to the whole height; the rest of the deck fills the bottom third.
fn cloud_height(s: Sky, cellp: vec2<f32>, along: f32, storm: f32) -> f32 {
    if storm <= 0.0 {
        return along;
    }
    let span = mix(1.0 / mix(1.0, 3.0, storm), 1.0, storm_tower(storm_field(s, cellp)));
    return along / span;
}

// How dense the cloud is at `cellp` (picture cells, fractional) and `h` (0 at the slab's bottom,
// 1 at its top): the four cells about the point blended smoothly, eaten away toward the top and
// bottom so their edges round.
fn cloud_density(s: Sky, cellp: vec2<f32>, h: f32, dims: vec2<f32>) -> f32 {
    let q = cellp - vec2(0.5);
    let base = floor(q);
    let f = q - base;
    let u = f * f * (3.0 - 2.0 * f);
    let a = cloud_cell(s, base, dims);
    let b = cloud_cell(s, base + vec2(1.0, 0.0), dims);
    let c = cloud_cell(s, base + vec2(0.0, 1.0), dims);
    let d = cloud_cell(s, base + vec2(1.0, 1.0), dims);
    var cover = mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
    // Fuzzy edges (`cloud_fuzz`), only at an edge: the cores and the open sky ask no noise.
    if cover > 0.02 && cover < 0.98 {
        cover = clamp(cover + cloud_fuzz(q + s.uv.zw) * 0.45, 0.0, 1.0);
    }
    // In rain the cells give way to the deck's large forms: with the cell picture alone a rainy sky
    // stayed a blocky patchwork (the storm-light check, 11-05).
    let rainy = s.axes[0].x;
    if rainy > 0.0 {
        cover = mix(cover, smoothstep(0.25, 0.6, storm_field(s, cellp)) * 0.4 + 0.5, rainy * 0.85);
    }
    let middle = 1.0 - pow(abs(h * 2.0 - 1.0), 2.0);
    // A storm's deck is one sheet: rounded puffs would leave lens-shaped gaps where four meet, the
    // bright sky showing through them.
    // Where the storm's field is lowest the deck breaks open a little.
    let deck = s.axes[0].z * smoothstep(0.0, 0.4, middle);
    // (A wider edge, 0.3-0.7: at 0.35-0.6 the slabs' sides were hard.)
    let shaped = max(smoothstep(0.3, 0.7, cover - (1.0 - middle) * 0.5), deck);
    // In rain the deck breaks open where its large forms are lowest, wider on the sun's side (the
    // light shafts in `post/volume.wgsl` come through the same breaks).
    let wet = max(s.axes[0].x, scene.weather.w);
    if wet > 0.0 {
        return shaped * mix(1.0, 1.0 - deck_open(storm_field(s, cellp), SUN_SIDE), wet);
    }
    return shaped;
}

// ---- soft clouds (`core:soft_clouds`; the engine's `client/soft_clouds.rs`) ----
// Ronald, 11-05: "Both, a setting" (blocky by default), then "Soft clouds first", storms first.
// Traced at a quarter of the pixels each way (`fs_cloud_trace`), blended over frames
// (`fs_cloud_resolve`), drawn by `fs_clouds`. The method after Horizon Zero Dawn's (Schneider 2015)
// and Photon's: masses from a slow coverage field, billows remapped into it, detail that only wears
// their edges, a light march that doubles each step, scattering in octaves. Our own code: Photon's
// licence allows learning from it, not reuse (the Researcher's notes, side-session-notes
// soft-clouds-photon-bliss.md).

// The soft layer over its underside (blocks): fair-weather cumulus, and a storm's deck with towers.
const SOFT_THICK: f32 = 120.0;
const SOFT_STORM_THICK: f32 = 190.0;
// How far a look is followed (blocks): beyond, the clouds fade into the sky.
const SOFT_REACH: f32 = 4000.0;

// One slice `z` (0..63) of the noise volume at `xy` (texels, 0..64).
fn soft_slice(z: u32, xy: vec2<f32>) -> vec4<f32> {
    let cell = vec2<f32>(f32(z % 8u), f32(z / 8u)) * 66.0 + vec2(1.0);
    return textureSampleLevel(textures[u32(scene.soft_clouds.z)], linear_clamp, (cell + xy) / 528.0, 0.0);
}

// The noise volume at `p` (tiles: it repeats every 1 each way), between two slices.
fn soft_noise(p: vec3<f32>) -> vec4<f32> {
    let t = fract(p) * 64.0;
    let zf = t.z - 0.5 + 64.0;
    let z0 = u32(floor(zf)) % 64u;
    return mix(soft_slice(z0, t.xy), soft_slice((z0 + 1u) % 64u, t.xy), fract(zf));
}

// Where camera-relative `rel` is in the clouds' own frame (blocks, x and z): the picture's cells,
// which the wind carries, wrap every 4096 cells, so every tile here divides 49152 blocks.
fn soft_at(s: Sky, rel: vec3<f32>) -> vec2<f32> {
    return (s.uv.xy + rel.xz / s.extra.x + s.uv.zw) * s.extra.x;
}

// How much of the sky the clouds cover here, 0..1: fair weather a third, rain most, a storm all
// but its breaks; varying over a few thousand blocks (Photon's slow field, Bliss's large noise).
fn soft_cover(s: Sky, at: vec2<f32>) -> f32 {
    let wet = max(s.axes[0].x, s.axes[0].z);
    let field = soft_slice(9u, fract(at / 6144.0) * 64.0).r * 0.65 + soft_slice(27u, fract(at / 1536.0) * 64.0).r * 0.35;
    // At 0.38 a clear day's were thin wisps with no body (the first soft run, 11-05).
    // Storms at 0.92 closed into one even deck once the billows were equalized: 0.8 keeps breaks.
    // Clear weather from about 0.36 to 0.57 (0.46 +- 0.35 of the field's spread): with 0.5 +- 0.8,
    // where the slow field peaks (over 0.6) the billows all passed and a clear day stood under a
    // flat grey deck that never moved off, a birch forest's sky through its gaps grey at every
    // hour (Tester 2's looks4, 11-05, once soft clouds were the default).
    return clamp(mix(0.46, 0.8, wet) + (field - 0.5) * mix(0.35, 0.5, wet), 0.0, 1.0);
}

// The cloud's density at `at` (its frame, blocks) and `a` (0 at the layer's underside, 1 at its
// top), 0..1; `fine`: the eroding detail too (the light march leaves it out).
fn soft_density(s: Sky, at: vec2<f32>, a: f32, cover: f32, fine: bool) -> f32 {
    if a < 0.0 || a > 1.0 {
        return 0.0;
    }
    let storm = s.axes[0].z;
    let t = scene.params.w;
    // Billows about three hundred blocks across (at half that a clear day's were small wisps
    // beside Photon's cumulus, the soft-clouds sheet, 11-05), leaning a little with height, mixed
    // with the slow field so masses group.
    let billow = soft_noise(vec3(at.x / 1536.0 + a * 0.04, a * 0.3 + t * 0.0003, at.y / 1536.0)).r;
    let mass = soft_slice(41u, fract(at / 3072.0) * 64.0).r;
    // A smaller billow on them, so the puffs are cauliflowers, not smooth marshmallows (11-05).
    let small = soft_noise(vec3(at.x / 384.0 + 0.37, a * 0.8 + t * 0.001, at.y / 384.0)).r;
    let shape = billow * 0.55 + mass * 0.25 + small * 0.2;
    // Photon's remap: the billows the cover lets through, shaped from the bottom up.
    var d = clamp((shape - (1.0 - cover)) / max(cover, 0.05) * 1.6, 0.0, 1.0);
    // An egg: a flat-ish base, rounded tops; a storm's deck fills more of its height.
    d *= smoothstep(0.0, 0.08, a);
    d -= smoothstep(mix(0.35, 0.55, storm), 1.0, a) * mix(0.45, 0.4, storm);
    if fine && d > 0.0 {
        // Detail wears only the edges (Photon: by 1 - density), so the cores stay solid.
        let w = soft_noise(vec3(at.x / 128.0, a * 1.4 + t * 0.002, at.y / 128.0));
        let cells = w.g * 0.625 + w.b * 0.25 + w.a * 0.125;
        d -= (1.0 - cells) * 0.5 * (1.0 - d) * mix(0.6, 1.0, a);
    }
    return clamp(d, 0.0, 1.0);
}

// The soft clouds along `dir` from the camera, up to `far` (blocks): rgb the light they send toward
// the eye (linear), a how much of what is behind still shows.
// How far over the blocky clouds' height the soft layer's underside stands (blocks), less as rain
// comes (a storm's deck hangs low): cumulus three hundred blocks across at the blocky clouds'
// height, 150 blocks over a forest floor, filled every gap in the canopy with one flat grey base,
// looking up from under the trees (Tester 2's looks4, 11-05); higher, they stand in the sky as
// clouds with blue between them, as Bliss's do.
const SOFT_LIFT: f32 = 220.0;

fn soft_low(s: Sky) -> f32 {
    return s.extra.z + SOFT_LIFT * (1.0 - 0.7 * max(s.axes[0].x, s.axes[0].z));
}

fn soft_march(s: Sky, dir: vec3<f32>, far_in: f32, dither: f32) -> vec4<f32> {
    let storm = s.axes[0].z;
    let thick = mix(SOFT_THICK, SOFT_STORM_THICK, storm);
    let low = soft_low(s);
    let high = low + thick;
    var near = 0.0;
    var far = min(far_in, SOFT_REACH);
    if abs(dir.y) > 1e-5 {
        let a = low / dir.y;
        let b = high / dir.y;
        near = max(near, min(a, b));
        far = min(far, max(a, b));
    } else if low > 0.0 || high < 0.0 {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    if near >= far {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    // Photon's: more steps toward the horizon, where a look crosses more of the layer.
    let steps = i32(mix(36.0, 18.0, abs(dir.y)));
    let step = (far - near) / f32(steps);
    let p = palette();
    let l = light_dir();
    let mu = dot(dir, l);
    // Two lobes, forward (the silver lining toward the sun) and a little back.
    let phase = min(0.75 * cloud_hg(mu, 0.75) + 0.25 * cloud_hg(mu, -0.2), 0.6) * 12.566;
    let sun = p.sun * light_strength() * (1.0 - s.axes[0].y * 0.55);
    // Rain darkens the clouds' albedo and keeps their shapes (Photon: ×0.66); a storm's more.
    let albedo = mix(1.0, 0.66, s.axes[0].x) * mix(1.0, 0.6, storm);
    var through = 1.0;
    var light = vec3(0.0);
    for (var i = 0; i < steps; i++) {
        let t = near + (f32(i) + dither) * step;
        let rel = dir * t;
        let at = soft_at(s, rel);
        let a = (rel.y - low) / thick;
        let cover = soft_cover(s, at);
        let d = soft_density(s, at, a, cover, true);
        if d <= 0.0 {
            continue;
        }
        // Extinction per block: a dense core is gone in a few blocks.
        let sigma = d * 0.55;
        // Toward the light, five taps each twice as far as the last (3 to 48 blocks).
        var od = 0.0;
        var reach = 3.0;
        var last = 0.0;
        for (var j = 0; j < 5; j++) {
            let q = rel + l * reach;
            let qa = (q.y - low) / thick;
            od += soft_density(s, soft_at(s, q), qa, cover, false) * 0.55 * (reach - last);
            last = reach;
            reach *= 2.0;
        }
        // Scattering in octaves: each lets more light through, less of it, rounder.
        var direct = 0.0;
        var k = 1.0;
        var e = 1.0;
        for (var o = 0; o < 3; o++) {
            direct += k * exp(-od * e) * mix(1.0, phase, pow(0.6, f32(o)));
            k *= 0.5;
            e *= 0.4;
        }
        // Dark edges facing away from the sun (powder), fading out looking toward it.
        let powder = mix(1.0 - exp(-sigma * 14.0), 1.0, 0.5 * (mu * 0.5 + 0.5) * (mu * 0.5 + 0.5) + 0.3);
        // The sky's light from above, brighter on the tops; under a storm the bellies darker.
        // Grey-blue shaded bases, bright tops: evenly lit, a clear day's read as flat white cut-outs.
        let ambient = mix(p.horizon * 0.3 + p.ambient * 0.25, p.ambient * 1.2 + p.horizon * 0.2, smoothstep(0.0, 0.8, a)) * mix(1.0, 0.55, s.axes[0].y * (1.0 - a));
        let c = (sun * direct * powder * 0.9 + ambient) * albedo;
        let kept = exp(-sigma * step);
        light += through * (1.0 - kept) * c;
        through *= kept;
        if through < 0.03 {
            through = 0.0;
            break;
        }
    }
    return vec4(light, through);
}

// The trace: one look per quarter-target pixel, at a sub-pixel that moves each frame (16 places),
// dithered along the look; it stops at the world (its depth's copy).
@fragment
fn fs_cloud_trace(in: CloudOut) -> @location(0) vec4<f32> {
    let s = sky[in.index];
    let n = u32(scene.cloud_moved.w);
    let px = vec2(dpdx(in.at.x), dpdy(in.at.y));
    // A Bayer order through the 4 × 4 pixels this one stands for.
    let bayer = array<u32, 16>(0u, 8u, 2u, 10u, 12u, 4u, 14u, 6u, 3u, 11u, 1u, 9u, 15u, 7u, 13u, 5u);
    let b = bayer[n % 16u];
    let sub = (vec2(f32(b % 4u), f32(b / 4u)) + 0.5) / 4.0 - 0.5;
    let at = vec3(in.at.xy + sub * px, 1.0);
    let dir = normalize(mat(sky[0]) * at);
    var far = SOFT_REACH;
    if scene.behind.z > 0.5 {
        let size = vec2<f32>(textureDimensions(textures[u32(scene.behind.y)]));
        let uv = vec2(at.x * 0.5 + 0.5, 0.5 - at.y * 0.5);
        let d = textureLoad(textures[u32(scene.behind.y)], vec2<i32>(clamp(uv * size, vec2(0.0), size - 1.0)), 0).r;
        if d > 0.0 {
            let w = scene.inv_view_proj * vec4(at.x, at.y, d, 1.0);
            far = length(w.xyz / w.w);
        }
    }
    let h = hash(u32(in.clip.x) * 1973u + u32(in.clip.y) * 9277u);
    let dither = fract(unit(h) + f32(n) * 0.618034);
    if !open_sky() || in_liquid() {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    return soft_march(s, dir, far, dither);
}

// How far the clouds seen along `dir` stand, for reprojecting them: the layer's lower third, or
// far off where a look misses it.
fn soft_distance(s: Sky, dir: vec3<f32>) -> f32 {
    let mid = soft_low(s) + mix(SOFT_THICK, SOFT_STORM_THICK, s.axes[0].z) * 0.3;
    let t = mid / dir.y;
    return select(SOFT_REACH, clamp(t, 30.0, SOFT_REACH), t > 0.0 && abs(dir.y) > 1e-4);
}

// The resolve: this frame's trace blended into last frame's clouds where they stood then
// (reprojected: the camera's move and the wind's), held to what this frame's neighbourhood allows.
@fragment
fn fs_cloud_resolve(in: CloudOut) -> @location(0) vec4<f32> {
    let s = sky[in.index];
    let trace = u32(s.axes[1].x - 1.0);
    let at = vec2<i32>(in.clip.xy);
    let size = vec2<i32>(textureDimensions(textures[trace]));
    let now = textureLoad(textures[trace], at, 0);
    if scene.soft_clouds.w < 0.5 || scene.soft_clouds.y < 0.5 {
        return now;
    }
    let dir = normalize(mat(sky[0]) * in.at);
    let then = scene.prev_view_proj * vec4(dir * soft_distance(s, dir) + scene.cloud_moved.xyz, 1.0);
    if then.w <= 0.0 {
        return now;
    }
    let ndc = then.xy / then.w;
    let uv = vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    if any(uv < vec2(0.0)) || any(uv > vec2(1.0)) {
        return now;
    }
    var history = textureSampleLevel(textures[u32(scene.soft_clouds.y - 1.0)], linear_clamp, uv, 0.0);
    // Held within this frame's 3 × 3, widened: the trace is noisy, the clamp only stops smears.
    var lo = now;
    var hi = now;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let c = textureLoad(textures[trace], clamp(at + vec2(x, y), vec2(0), size - 1), 0);
            lo = min(lo, c);
            hi = max(hi, c);
        }
    }
    let pad = (hi - lo) * 0.5 + vec4(0.02);
    history = clamp(history, lo - pad, hi + pad);
    // Up to nine parts history: each pixel was traced once in 16 frames at its own sub-pixel.
    let w = min(1.0 - 1.0 / (scene.soft_clouds.w + 1.0), 0.9);
    return mix(now, history, w);
}

// The full picture from the resolved clouds (`fs_clouds` with soft clouds on).
fn soft_pixel(in: CloudOut, s: Sky, dir: vec3<f32>) -> CloudPixel {
    let size = vec2<f32>(textureDimensions(textures[u32(scene.behind.x)]));
    let c = textureSampleLevel(textures[u32(scene.soft_clouds.x - 1.0)], linear_clamp, in.clip.xy / size, 0.0);
    let alpha_cloud = 1.0 - c.a;
    if alpha_cloud < 0.004 {
        discard;
    }
    let seen = soft_distance(s, dir);
    let rel = dir * seen;
    var color = c.rgb / alpha_cloud;
    // Toward the far end they hand over to the sky.
    let dist = length(rel.xz) / SOFT_REACH;
    color = mix(color, sky_color(normalize(vec3(dir.x, max(dir.y, 0.0) + 1e-3, dir.z))), smoothstep(0.45, 1.0, dist));
    color = display(fog_through(rainbow_on(color, dir), dir * 220.0, 1.0));
    let alpha = alpha_cloud * (1.0 - smoothstep(0.85, 1.0, dist));
    let clip = scene.view_proj * vec4(rel, 1.0);
    return CloudPixel(vec4(color * alpha, alpha), clip.z / clip.w);
}

@fragment
fn fs_clouds(in: CloudOut) -> CloudPixel {
    let s = sky[in.index];
    let dir = normalize(mat(sky[0]) * in.at);
    if scene.soft_clouds.x > 0.5 {
        return soft_pixel(in, s, dir);
    }
    let size = s.extra.x;
    let low = s.extra.z;
    let thick = max(s.extra.y, CLOUD_THICKNESS);
    let reach = s.extra.w;
    // In a thunderstorm the slab grows: a few cells stand up as towers three times as tall.
    let storm = scene.weather.w;
    let tall = thick * mix(1.0, 3.0, storm);
    let high = low + tall;
    var near = 0.0;
    var far = reach / max(length(dir.xz), 0.0001);
    if abs(dir.y) > 0.00001 {
        near = max(near, min(low / dir.y, high / dir.y));
        far = min(far, max(low / dir.y, high / dir.y));
    } else if low > 0.0 || high < 0.0 {
        discard;
    }
    if near >= far {
        discard;
    }
    // What the opaque world shows at this pixel (its depth's copy, `scene.behind`, made before the
    // sky pass): clouds behind a hill or a wall are not marched at all, and the rest stop at it.
    // The pass writes its depth, so the GPU cannot skip those pixels by itself.
    if scene.behind.z > 0.5 {
        let d = textureLoad(textures[u32(scene.behind.y)], vec2<i32>(in.clip.xy), 0).r;
        if d > 0.0 {
            let size = vec2<f32>(textureDimensions(textures[u32(scene.behind.y)]));
            let uv = in.clip.xy / size;
            let w = scene.inv_view_proj * vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
            let solid = length(w.xyz / w.w);
            if solid <= near {
                discard;
            }
            far = min(far, solid);
        }
    }
    far = min(far, near + CLOUD_SPAN);
    let dims = vec2<f32>(f32(s.ids.z), f32(s.ids.w));
    let step = (far - near) / f32(CLOUD_SAMPLES);
    // A different start per pixel, against banding.
    let jitter = fract(sin(dot(in.clip.xy, vec2(12.9898, 78.233))) * 43758.5453);
    let p = palette();
    let lit_sky = open_sky() && !in_liquid();
    let dark = s.axes[0].y;
    let mu = max(dot(dir, scene.sun.xyz), 0.0);
    var through = 1.0;
    var light = vec3(0.0);
    var seen = far;
    // Lit as a volume (Research round 15, rec. 2; Ronald: "both, a setting", blocky by default):
    // two taps toward the light through the slab (Beer-Lambert, with a powder term), a two-lobe
    // phase for the silver lining and the backlit glow, and three octaves of multiple scattering.
    let l = light_dir();
    let mu_l = dot(dir, l);
    // The sun's side of the sky, by day: a sweep across the whole sky, not a glow about the sun.
    let flat_dir = normalize(vec2(dir.x, dir.z) + vec2(1e-4, 0.0));
    let flat_sun = normalize(vec2(scene.sun.x, scene.sun.z) + vec2(1e-4, 0.0));
    SUN_SIDE = smoothstep(-0.7, 0.9, dot(flat_dir, flat_sun)) * smoothstep(-0.02, 0.1, scene.sun.y);
    let wet = max(s.axes[0].x, storm);
    let phase = 0.7 * cloud_hg(mu_l, 0.8) + 0.3 * cloud_hg(mu_l, -0.2);
    for (var i = 0u; i < CLOUD_SAMPLES; i++) {
        let t = near + (f32(i) + jitter) * step;
        let rel = dir * t;
        let cellp = s.uv.xy + rel.xz / size;
        let along = (rel.y - low) / tall;
        let h = cloud_height(s, cellp, along, storm);
        if h > 1.0 {
            continue;
        }
        var density = cloud_density(s, cellp, h, dims);
        if density <= 0.0 {
            continue;
        }
        // A storm's deck is lumpy, its density varying from cell to cell, blended between them as
        // the density is (a step at each cell's edge showed as a seam running off to the horizon).
        if storm > 0.0 {
            density *= mix(1.0, 0.45 + 1.1 * cloud_lumps(s, cellp, along), storm * 0.8);
        }
        let a = 1.0 - exp(-density * step * 0.18);
        var c = vec3(1.0);
        if lit_sky {
            // How much of the light reaches here through the cloud toward it.
            var od = 0.0;
            for (var j = 1; j <= 2; j++) {
                let q = rel + l * (thick * 0.45 * f32(j));
                let qc = s.uv.xy + q.xz / size;
                let qh = cloud_height(s, qc, (q.y - low) / tall, storm);
                if qh <= 1.0 && qh >= 0.0 {
                    od += cloud_density(s, qc, qh, dims) * thick * 0.45 * 0.18;
                }
            }
            var scatter = 0.0;
            for (var o = 0; o < 3; o++) {
                let k = pow(0.5, f32(o));
                scatter += k * exp(-od * k) * mix(1.0, phase * 4.0, k);
            }
            let powder = 1.0 - exp(-2.0 * (od + density * 0.5));
            // Undersides darker by their albedo in a storm, not blackened.
            // By the height in the deck (the same at the same height everywhere), and darker under a
            // tower's whole column, smoothly, as the towers rise.
            var albedo = 1.0;
            if wet > 0.0 {
                let n = storm_field(s, cellp);
                let deep = mix(0.45, 1.0, smoothstep(0.0, 0.5, along)) * mix(1.0, 0.7, storm_tower(n));
                // Dark masses away from the sun, a brighter side toward it, and the thin deck about
                // the breaks there lit through (Bliss's storm: dark to bright across the sky).
                let thin = 1.0 - smoothstep(0.3, 0.6, n);
                let sweep = mix(0.6, 1.3, SUN_SIDE) * (1.0 + 0.8 * thin * SUN_SIDE);
                albedo = mix(1.0, deep, storm) * mix(1.0, sweep, wet);
            }
            let below = mix(p.horizon * 0.9 + p.ambient * 0.6, p.ambient * 1.6, h) * mix(1.0, 0.4, dark * (1.0 - h)) * albedo;
            c = below + p.sun * light_strength() * scatter * mix(0.6, 1.0, powder) * 0.55 * (1.0 - dark * 0.6) * albedo;
            c += p.glow * (pow(mu, 3.0) * 0.5 + pow(mu, 12.0) * 0.8) * light_strength() * (0.4 + 0.6 * h);
        } else {
            c = vec3(mix(0.7, 1.0, h));
        }
        if through > 0.5 && through * (1.0 - a) <= 0.5 {
            seen = t;
        }
        light += through * a * c;
        through *= 1.0 - a;
        if through < 0.02 {
            break;
        }
    }
    let alpha_cloud = 1.0 - through;
    if alpha_cloud < 0.01 {
        discard;
    }
    // The engine's color (`sky.rs`) brings rain's grey and thunder's dark; the palette the light.
    var color = s.color.rgb;
    let rel = dir * seen;
    let dist = length(rel.xz) / reach;
    if lit_sky {
        let d = clamp(scene.sun.y * 2.0 + 0.5, 0.0, 1.0);
        let dimmed = pow(vec3(d * 0.9 + 0.1, d * 0.9 + 0.1, d * 0.85 + 0.15), vec3(2.2));
        // A thunderstorm's deck floored at a mid grey: the engine darkens its colour to a fifth, and
        // the deck went near black over pale mist with a bright band of sky at the horizon between.
        // None of the packs lets a storm sky go that dark (Complementary floors it at about 0.65;
        // the Researcher's weather findings, 11-05); the bellies still darken (`dark`).
        let albedo = max(s.color.rgb / max(dimmed, vec3(0.005)), vec3(0.55, 0.58, 0.64) * scene.weather.w);
        color = albedo * light / alpha_cloud;
        color = mix(color, sky_color(normalize(vec3(dir.x, max(dir.y, 0.0) + 1e-3, dir.z))), smoothstep(mix(0.3, 0.15, wet), mix(1.0, 0.85, wet), dist));
        // The rainbow stands in the falling rain under the clouds, in front of them: drawn in the sky
        // alone, the shower's clouds hid it (the after-rain check from under the clouds, 11-05).
        // Rain's mist between the eye and the cloud, as over the land (`rain_depth`), toward the
        // colour the land's mist takes: from the vista, 14 blocks under a storm's deck, all of it in
        // view was near and dark over land kilometres off and fully misted, and the two met in a hard
        // line at the horizon (fading by the deck's reach alone could not close it).
        let misted = 1.0 - exp(-rain_depth(length(rel)));
        color = mix(color, sky_color(normalize(vec3(dir.x, max(dir.y, 0.0) * 0.35 + 1e-3, dir.z))), misted);
        // A lightning flash glows inside the deck; its bolt hangs under the deck, in front of it.
        color += lightning_in_clouds(dir) * albedo + lightning_bolt(dir);
        color = display(fog_through(rainbow_on(color, dir), dir * 220.0, 1.0));
    } else {
        color = mix(s.color.rgb * light / alpha_cloud, scene.fog_color.rgb, smoothstep(0.5, 1.0, dist));
    }
    // In rain the deck thins out over the second half of its reach into the low sky, whose colour
    // the land's mist takes too: faded only in its last fifth, a storm's deck ended in a hard
    // straight line at the horizon (Tester 2's rain and thunder shots, 11-05).
    let alpha = s.color.a * alpha_cloud * (1.0 - smoothstep(mix(0.8, 0.4, wet), 1.0, dist));
    let clip = scene.view_proj * vec4(rel, 1.0);
    return CloudPixel(vec4(color * alpha, alpha), clip.z / clip.w);
}

// A star as the pack sees it: round, of its own brightness (few bright, many faint) and color (its
// temperature: orange, white, blue-white), twinkling more low down, and showing only once the sky
// is dark enough for it (the brightest first at dusk). rgb tints, a scales the engine's star.
fn star_look(star: u32, uv: vec2<f32>, dir: vec3<f32>) -> vec4<f32> {
    let bright = pow(unit(star * 9u + 5u), 3.0);
    let warmth = unit(star * 9u + 6u);
    var tint = mix(vec3(1.0, 0.62, 0.38), vec3(1.0), smoothstep(0.0, 0.3, warmth));
    tint = mix(tint, vec3(0.66, 0.8, 1.0), smoothstep(0.55, 1.0, warmth));
    // Stars are a pixel or two across: no shape of their own, the size tells the bright ones.
    let shape = 1.0;
    let speed = 2.0 + 3.0 * unit(star * 9u + 7u);
    let twinkle = 1.0 - (0.08 + 0.2 * (1.0 - clamp(dir.y, 0.0, 1.0))) * (0.5 + 0.5 * sin(scene.params.w * speed + unit(star * 9u + 8u) * 40.0));
    let dark = smoothstep(0.05, -0.18, scene.sun.y);
    let needs = 1.0 - (0.25 + 0.75 * bright);
    let shows = smoothstep(needs, needs + 0.15, dark);
    return vec4(tint, shape * (1.0 + 1.6 * bright) * twinkle * shows);
}

// The Milky Way: a faint band across the night, fixed among the stars (it turns with the sky, as the
// sun does), mottled, with dark lanes along its middle; washed out by a bright moon and haze low down.
fn milky_way(dir: vec3<f32>) -> vec3<f32> {
    let dark = smoothstep(-0.08, -0.25, scene.sun.y);
    if dark <= 0.0 || dir.y <= 0.0 {
        return vec3(0.0);
    }
    let a = atan2(scene.sun.y, scene.sun.x);
    let local = vec3(dir.x * cos(a) + dir.y * sin(a), -dir.x * sin(a) + dir.y * cos(a), dir.z);
    let d = dot(local, normalize(vec3(0.25, 0.35, 0.9)));
    let band = exp(-(d / 0.16) * (d / 0.16));
    let mottle = noise3(local * 6.0) * 0.6 + noise3(local * 14.0) * 0.4;
    let lane = smoothstep(0.55, 0.75, noise3(local * 9.0 + vec3(3.0))) * exp(-(d / 0.05) * (d / 0.05));
    let glow = band * (0.4 + 0.8 * mottle) * (1.0 - 0.6 * lane);
    let clear = (1.0 - rain_amount()) * (1.0 - 0.6 * scene.moon.w) * smoothstep(0.0, 0.25, dir.y);
    var c = vec3(0.75, 0.78, 0.95) * glow * 0.03;
    // A magical night sky (round 17, rec. 3; `base:fx_night_sky`, 1 subtle, 2 vivid): coloured
    // clouds along the band, blue, violet and a little amber, pixel-snapped as the aurora is, and a
    // denser field of crisp pixel stars, thickest in the band. Still fainter than the moon.
    let vivid = fantasy_opt(OPT_NIGHT_SKY);
    if vivid > 0.0 {
        let q = normalize(floor(local * 180.0) / 180.0);
        let blue = smoothstep(0.45, 0.8, noise3(q * 4.0 + vec3(11.0)));
        let violet = smoothstep(0.5, 0.85, noise3(q * 5.0 + vec3(23.0)));
        let amber = smoothstep(0.62, 0.9, noise3(q * 6.0 + vec3(37.0)));
        let nebula = (vec3(0.25, 0.4, 1.0) * blue + vec3(0.65, 0.3, 0.95) * violet + vec3(1.0, 0.6, 0.3) * amber * 0.6) * band;
        c += nebula * mix(0.012, 0.03, vivid - 1.0) * (0.5 + mottle);
        // Pixel stars on a grid, a hash per cell: a few in the open sky, more in the band.
        let cell = vec3<i32>(floor(dir * 700.0));
        let h = hash(u32(cell.x * 73856093) ^ u32(cell.y * 19349663) ^ u32(cell.z * 83492791));
        let chance = mix(0.004, 0.014, vivid - 1.0) * (1.0 + 4.0 * band);
        if unit(h) < chance {
            let warmth = unit(h * 7u + 1u);
            let tint = mix(vec3(0.75, 0.85, 1.0), vec3(1.0, 0.85, 0.65), warmth);
            c += tint * mix(0.04, 0.4, pow(unit(h * 13u + 5u), 6.0));
        }
    }
    return c * dark * clear;
}

// An aurora on clear nights where the place lies under the auroral oval (`scene.ground.w`, by
// latitude): curtains of ridged noise in the northern sky, green low and purple higher up, drifting
// slowly, on a coarse grid of directions so they stay blocky.
fn aurora(dir: vec3<f32>) -> vec3<f32> {
    // An aurora storm (a sky event) reaches further and burns brighter; where it reaches comes with
    // the biomes' aurora shares the base mod widens for it.
    let storm = sky_event(4.0);
    let strength = max(scene.ground.w, storm * 0.6) * mix(1.0, 2.2, storm) * smoothstep(-0.1, -0.25, scene.sun.y) * (1.0 - rain_amount()) * (1.0 - scene.weather.z);
    if strength <= 0.0 || dir.y <= 0.02 {
        return vec3(0.0);
    }
    let d = floor(dir * 160.0) / 160.0;
    let north = smoothstep(-0.1, 0.6, -d.z);
    let t = scene.params.w;
    // On a ceiling over the land, so the curtains run along the sky and close toward the horizon.
    let p = d.xz / max(d.y, 0.05);
    let ridge = 1.0 - abs(noise3(vec3(p.x * 0.35 + t * 0.03, t * 0.05, p.y * 0.12)) * 2.0 - 1.0);
    let curtain = pow(ridge, 6.0) * (0.6 + 0.4 * noise3(vec3(p.x * 1.3, t * 0.2, p.y * 0.4)));
    let height = smoothstep(0.02, 0.15, d.y) * (1.0 - smoothstep(0.35, 0.75, d.y));
    var color = mix(vec3(0.1, 0.9, 0.45), vec3(0.55, 0.25, 0.85), smoothstep(0.15, 0.55, d.y));
    // A palette per night (`base:fx_night_sky`; Photon's thirteen): a hash of the day picks a pair,
    // green-teal, green-violet, rose-violet, now and then red.
    if fantasy_opt(OPT_NIGHT_SKY) > 0.0 {
        let night_no = u32(floor((scene.light.z + 6000.0) / 24000.0));
        let pick = hash(night_no * 2654435761u + 17u) % 8u;
        let lows = array<vec3<f32>, 8>(vec3(0.1, 0.9, 0.45), vec3(0.1, 0.85, 0.7), vec3(0.15, 0.9, 0.4), vec3(0.95, 0.35, 0.6), vec3(0.1, 0.8, 0.55), vec3(0.2, 0.95, 0.5), vec3(0.9, 0.2, 0.2), vec3(0.3, 0.8, 0.9));
        let highs = array<vec3<f32>, 8>(vec3(0.55, 0.25, 0.85), vec3(0.2, 0.5, 0.95), vec3(0.75, 0.3, 0.95), vec3(0.6, 0.3, 0.95), vec3(0.9, 0.3, 0.7), vec3(0.3, 0.7, 0.95), vec3(0.7, 0.2, 0.6), vec3(0.6, 0.4, 0.95));
        color = mix(lows[pick], highs[pick], smoothstep(0.15, 0.55, d.y));
    }
    // A storm's palette: red low, violet above.
    color = mix(color, mix(vec3(0.95, 0.15, 0.25), vec3(0.6, 0.2, 0.95), smoothstep(0.1, 0.5, d.y)), storm * 0.8);
    // A storm reaches over the whole sky, not only the north.
    let reach = mix(north, max(north, 0.6), storm);
    return color * curtain * height * reach * strength * 0.12;
}

// Shooting stars on a meteor shower's night (a sky event): pixel-snapped streaks, as many as the
// shower's strength (1 at its peak), each a fraction of a second. In twelve layers of slots of
// 1.2 s, each slot holds one streak or none, from a hash of the slot alone: at four the whole sky
// held about one at a time, and a view rarely caught it (the sky-events check, 11-05).
fn meteors(dir: vec3<f32>) -> vec3<f32> {
    let shower = sky_event(2.0);
    if shower <= 0.0 {
        return vec3(0.0);
    }
    // The shower's own clock (seconds since it began, real time: tests pin the shaders' time).
    return streaks(dir, mix(0.04, 0.8, shower), scene.sky_event.z);
}

// Pixel streaks across the night sky: `rate` the share of slots that hold one (0..1), on clock `t`.
fn streaks(dir: vec3<f32>, rate: f32, t: f32) -> vec3<f32> {
    let dark = smoothstep(-0.1, -0.25, scene.sun.y) * (1.0 - rain_amount());
    if dark <= 0.0 || dir.y <= 0.05 {
        return vec3(0.0);
    }
    let d = normalize(floor(dir * 220.0) / 220.0);
    var c = vec3(0.0);
    for (var layer = 0u; layer < 12u; layer++) {
        let at = t / 1.2 + f32(layer) * 0.0833;
        let slot = u32(floor(at));
        let h = hash(slot * 4u + layer + 7u);
        // About two a second at the peak, a few a minute on a side night.
        if unit(h) > rate {
            continue;
        }
        let f = fract(at) / 0.45;
        if f > 1.0 {
            continue;
        }
        // Where it starts (above 25°) and which way it falls, from the slot's hash.
        let az = unit(h * 3u + 1u) * 6.2831853;
        let el = mix(0.45, 1.2, unit(h * 3u + 2u));
        let start = vec3(cos(az) * cos(el), sin(el), sin(az) * cos(el));
        let side = normalize(cross(start, vec3(0.0, 1.0, 0.0)) + vec3(0.0, 1e-4, 0.0));
        let fall = normalize(side * (unit(h * 5u + 3u) * 2.0 - 1.0) - vec3(0.0, 0.6, 0.0) * start.y);
        let head = normalize(start + fall * 0.3 * f);
        // Distance to the streak between where it began and its head.
        let ab = head - start;
        let k = clamp(dot(d - start, ab) / max(dot(ab, ab), 1e-6), 0.0, 1.0);
        let off = length(d - (start + ab * k));
        let line = 1.0 - smoothstep(0.003, 0.006, off);
        // Brightest at the head, fading along the trail and as it burns out.
        let fade = k * k * (1.0 - smoothstep(0.7, 1.0, f));
        c += vec3(0.95, 0.97, 1.0) * line * fade * 0.9;
    }
    return c * dark;
}

// Shooting stars on any clear night (`base:fx_shooting_stars`): about one a minute somewhere in the
// sky, a short pixel line (Solas's). The meteor shower's streaks at a fiftieth of a peak's rate, on
// the shaders' own clock.
fn shooting_stars(dir: vec3<f32>) -> vec3<f32> {
    if fantasy_opt(OPT_SHOOTING_STARS) <= 0.0 || sky_event(2.0) > 0.0 {
        return vec3(0.0);
    }
    return streaks(dir, 0.016, scene.params.w);
}

// A fireball or a big impact on its way (`scene.sky_landing`): a burning head and a long smoky
// trail coming down toward where it lands, by day too, from 10 s before until it lands.
fn fireball(dir: vec3<f32>) -> vec3<f32> {
    let left = scene.sky_landing.w;
    if left < -0.5 || left > 10.0 {
        return vec3(0.0);
    }
    // It comes in steep from the south-west at about 60 blocks a second.
    let v = normalize(vec3(0.35, 1.0, 0.25));
    let head = scene.sky_landing.xyz + v * 60.0 * max(left, 0.0);
    let tail = head + v * 240.0;
    let d = normalize(floor(dir * 220.0) / 220.0);
    let hd = normalize(head);
    let tl = normalize(tail);
    let ab = tl - hd;
    let k = clamp(dot(d - hd, ab) / max(dot(ab, ab), 1e-6), 0.0, 1.0);
    let off = length(d - (hd + ab * k));
    let trail = (1.0 - smoothstep(0.003, 0.009, off)) * (1.0 - k);
    let glow = exp(-length(d - hd) * 90.0);
    let flash = 1.0 - smoothstep(0.0, 0.5, -left);
    return (vec3(1.0, 0.55, 0.2) * trail * 1.5 + vec3(1.0, 0.85, 0.6) * glow * 6.0) * flash;
}

// A rainbow opposite the sun while a shower passes with the sun out (Research round 15, 5): the
// primary bow 39.5-42.5° from the point opposite the sun, red outside, and a faint second one at
// 50-53.6° with its colours the other way; the sky inside the primary a little brighter. Six flat
// bands, pixel-snapped as the aurora is, to suit the blocks (Complementary's pixel rainbow). Only in
// light rain: a downpour hides the sun, and a thunderstorm makes none. Once the scene carries how
// wet the land still is (after the sync), it lingers as the rain clears.
const BOW: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
    vec3(0.45, 0.2, 0.9), vec3(0.2, 0.35, 1.0), vec3(0.25, 0.9, 0.35), vec3(1.0, 0.95, 0.25), vec3(1.0, 0.55, 0.12), vec3(1.0, 0.18, 0.1),
);

// `c` the sky or cloud behind (linear) with the bow on it in `dir`: tinted by its band (on a white
// shower cloud a bow only added on top showed no colour) and brightened. Ronald, 11-05: "Rainbow
// stronger first": at a quarter of the sun's light, added, it was a faint thread on the sheet.
fn rainbow_on(c: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    let r = rain_amount();
    // A shower with the sun out: the rain coming and going (a downpour hides the sun), none in a
    // thunderstorm.
    let showers = smoothstep(0.03, 0.15, r) * (1.0 - smoothstep(0.65, 0.97, r)) * (1.0 - scene.weather.w);
    // The sun out and under 42°, so the bow stands above the horizon.
    let low_sun = smoothstep(0.0, 0.08, scene.sun.y) * (1.0 - smoothstep(0.5, 0.67, scene.sun.y));
    let strength = showers * low_sun * light_strength();
    if strength <= 0.0 || dir.y <= 0.0 {
        return c;
    }
    let d = normalize(floor(dir * 100.0) / 100.0);
    let deg = degrees(acos(clamp(dot(d, -scene.sun.xyz), -1.0, 1.0)));
    // It thins toward the horizon, where less rain lies along the look.
    let k = strength * smoothstep(0.0, 0.06, d.y);
    var band = vec3(0.0);
    var weight = 0.0;
    let one = (deg - 39.5) / (3.0 / 6.0);
    if one >= 0.0 && one < 6.0 {
        band = BOW[u32(one)];
        weight = 1.0;
    }
    let two = (deg - 50.0) / (3.6 / 6.0);
    if two >= 0.0 && two < 6.0 {
        band = BOW[5u - u32(two)];
        weight = 0.45;
    }
    // Inside the bow the sky is lit a little by the light it bends there; between the two bows it
    // is darker (Alexander's band).
    var out = c * (1.0 + 0.18 * k * (1.0 - smoothstep(30.0, 40.0, deg)));
    out *= 1.0 - 0.12 * k * smoothstep(42.5, 43.5, deg) * (1.0 - smoothstep(49.0, 50.0, deg));
    let a = k * weight;
    return mix(out, out * band * 1.6, a * 0.55) + band * luma(palette().sun) * a * 0.35;
}

// Vanilla's sky is a disc 16 blocks above the eye fading into fog with distance.
fn gradient(dir: vec3<f32>, s: Sky) -> vec4<f32> {
    // Under water, the water's own color all round (`look.wgsl`), not the engine's dark fog.
    if in_water() {
        return vec4(display(underwater_color()), 1.0);
    }
    // In another liquid (lava): its fog all round (the engine draws the gradient alone then).
    if s.extra.x > 0.5 {
        return vec4(scene.fog_color.rgb, 1.0);
    }
    if open_sky() && !in_liquid() {
        var c = sky_color(dir) + milky_way(dir) + aurora(dir) + meteors(dir) + shooting_stars(dir) + fireball(dir) + lightning_bolt(dir);
        // The rainbow in the rain, in front of the high cirrus.
        c = rainbow_on(cirrus(c, dir), dir);
        if dir.y < 0.0 {
            // From below sea level, vanilla's dark of the void.
            c *= 1.0 - s.uv.x * smoothstep(0.0, 0.08, -dir.y);
        }
        // The fog on the land hides the horizon and fades upward.
        c = fog_through(c, dir * 220.0, 1.0);
        // A held torch's glow in the mist goes on over the sky (`haze_lit` gives it the land's).
        c += held_in_fog(dir * 48.0, 1.0);
        return vec4(display(c), 1.0);
    }
    if dir.y <= 0.0 {
        // Under the horizon: fog, and from below sea level the dark of vanilla's void.
        let dark = s.uv.x * smoothstep(0.0, 0.08, -dir.y);
        return vec4(scene.fog_color.rgb * (1.0 - dark), 1.0);
    }
    return vec4(mix(s.color.rgb, scene.fog_color.rgb, clamp(0.0625 / dir.y, 0.0, 1.0)), 1.0);
}

// A soft high layer of thin cloud, far above the blocky ones (Research round 15, rec. 2 slice 4:
// Bliss's altostratus plane at y 2000): wisps on a plane, drifting slowly with the wind, lit by the
// sky and taking the sunset's colours and the moon's light, thicker under rain. Behind the blocky
// clouds it gives the sky depth.
fn cirrus(c: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    if dir.y <= 0.02 {
        return c;
    }
    let p = palette();
    let t = scene.params.w;
    let at = dir.xz / dir.y * 2000.0 + scene.eye.xz + wind_now() * t * 6.0;
    // Even both ways: stretched along one axis its wisps fanned out from the sun like a warp.
    let q = at / 600.0;
    let n = noise3(vec3(q, t * 0.002)) * 0.6 + noise3(vec3(q * 2.7 + 13.0, t * 0.003)) * 0.3 + noise3(vec3(q * 7.1 + 41.0, 0.0)) * 0.1;
    let rain = rain_amount();
    // Gone toward the horizon, where the plane's texture crowds into streaks.
    let cover = smoothstep(0.55 - 0.25 * rain, 0.85, n) * smoothstep(0.12, 0.4, dir.y);
    let mu = max(dot(dir, light_dir()), 0.0);
    let lit = p.horizon * 0.6 + p.ambient * 0.8 + p.sun * light_strength() * 0.35 + p.glow * pow(mu, 4.0) * light_strength() * 0.8;
    return mix(c, lit, cover * 0.55);
}

// Vanilla's sunset fan: an oval lying on the horizon on the side of the body.
fn glow(dir: vec3<f32>, s: Sky) -> vec4<f32> {
    let toward = dot(dir, s.axes[0].xyz);
    if toward <= 0.0 {
        return vec4(0.0);
    }
    let x = dot(dir, s.axes[2].xyz) / toward / 1.2;
    let y = dir.y / toward / max(0.4 * s.color.a, 0.001);
    var a = s.color.a * max(0.0, 1.0 - sqrt(x * x + y * y));
    if open_sky() {
        a *= 0.25;
    }
    return vec4(display(s.color.rgb), a);
}

// Panorama: longitude and latitude. Box: three by two faces; first row down, up, south; second
// west, north, east.
fn dome_uv(l: vec3<f32>, s: Sky) -> vec2<f32> {
    if s.uv.x == 0.0 {
        return vec2(atan2(l.x, -l.z) / 6.2831853 + 0.5, 0.5 - asin(clamp(l.y, -1.0, 1.0)) / 3.1415927);
    }
    let m = abs(l);
    var face: vec2<f32>;
    var cell: vec2<f32>;
    if m.y >= m.x && m.y >= m.z {
        face = vec2(l.x, select(-l.z, l.z, l.y > 0.0)) / m.y;
        cell = vec2(select(0.0, 1.0, l.y > 0.0), 0.0);
    } else if m.z >= m.x {
        face = vec2(select(l.x, -l.x, l.z > 0.0), -l.y) / m.z;
        cell = select(vec2(1.0, 1.0), vec2(2.0, 0.0), l.z > 0.0);
    } else {
        face = vec2(select(-l.z, l.z, l.x > 0.0), -l.y) / m.x;
        cell = select(vec2(0.0, 1.0), vec2(2.0, 1.0), l.x > 0.0);
    }
    return (cell + face * 0.5 + 0.5) / vec2(3.0, 2.0);
}

// The moon as a round disc lit by the sun (Bliss's moon is a sphere too), its markings from the
// full moon of the base game's picture, softened: the phase is where the sun stands, so it matches
// the night's light (the moon lags the day by its phase, `mods/base/src/lib.rs`). `local` is the
// pixel's direction in the body's frame (x toward it). By day a pale, see-through disc, as the real
// moon on a blue sky, only its lit part; at night the dark part keeps a faint earthshine. rgb is
// linear light before `display`, a how much of the pixel it covers.
fn moon_body(s: Sky, local: vec3<f32>) -> vec4<f32> {
    // The quad's half size as `vs` draws it (`base:moon_size`).
    var half = s.extra.x;
    half *= clamp(select(fantasy_opt(OPT_MOON_SIZE), 1.0, fantasy_opt(OPT_MOON_SIZE) <= 0.0), 1.0, 2.0);
    // Where in the quad, -1..1 along its up and its right; the picture's moon fills its middle half.
    let q = local.yz / max(local.x * half, 1e-5);
    let d = q / 0.5;
    let r2 = dot(d, d);
    let cover = 1.0 - smoothstep(0.94, 1.0, sqrt(r2));
    if cover <= 0.0 {
        return vec4(0.0);
    }
    // The sphere's face toward the eye, in the world, and how the sun lights it.
    let n = mat(s) * vec3(-sqrt(max(1.0 - r2, 0.0)), d.x, d.y);
    let lit = smoothstep(-0.04, 0.14, dot(n, scene.sun.xyz));
    // The full moon's picture (frame 0, top left) where this pixel lies: u along the quad's right,
    // v down its up, as `vs` lays the picture out.
    var albedo = vec3(0.85, 0.87, 0.9);
    if s.ids.x != 0xffffffffu {
        let c = vec2(q.y * 0.5 + 0.5, 0.5 - q.x * 0.5);
        let at = c * (s.uv.zw - s.uv.xy);
        albedo = textureSampleLevel(textures[s.ids.x], linear_clamp, at, 0.0).rgb;
    }
    // A little darker toward the rim (the real moon barely darkens; a hint reads as round).
    let rim = mix(0.78, 1.0, sqrt(max(1.0 - r2, 0.0)));
    let day = smoothstep(-0.05, 0.15, scene.sun.y);
    let shine = albedo * s.color.rgb * rim * (lit + 0.05 * (1.0 - day));
    // By day only the lit part shows, pale: a third of its night light over the blue.
    let a = cover * mix(1.0, lit, day);
    return vec4(shine * mix(1.0, 0.32, day), a);
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let s = sky[in.index];
    if s.ids.y == SKY_OVERLAY {
        // Vanilla's: four tiles across the screen, sliding with the look. A texture with
        // animation frames shows its first.
        if s.extra.y != 0.0 && in.at.y > -0.25 {
            return vec4(0.0);
        }
        let tile = fract((in.at.xy * vec2(0.5, -0.5) + 0.5) * s.extra.x + s.uv.xy) * s.uv.zw + vec2(0.0, s.extra.z);
        let c = textureSampleLevel(textures[s.ids.x], nearest_clamp, tile, 0.0) * s.color;
        return vec4(c.rgb * c.a, c.a);
    }
    var dir = in.at;
    if s.ids.y == SKY_GRADIENT || s.ids.y == SKY_GLOW || s.ids.y == SKY_DOME {
        dir = mat(sky[0]) * in.at;
    }
    dir = normalize(dir);
    if s.ids.y == SKY_GRADIENT {
        return gradient(dir, s);
    }
    if s.ids.y == SKY_GLOW {
        let c = glow(dir, s);
        return vec4(c.rgb * c.a, c.a);
    }
    let local = dir * mat(s);
    var uv = in.uv;
    var albedo = vec4(1.0);
    if s.ids.y == SKY_DOME {
        uv = dome_uv(local, s);
        if s.ids.x != 0xffffffffu {
            albedo = textureSampleLevel(textures[s.ids.x], linear_clamp, uv, 0.0);
        }
    } else if s.ids.x != 0xffffffffu {
        albedo = textureSampleLevel(textures[s.ids.x], nearest_clamp, uv, 0.0);
    }
    var c = sky_shade(s.ids.z, SkyPixel(dir, local, uv, albedo, vec4(s.color.rgb, 1.0), scene.params.z, scene.params.w));
    var a = c.a * s.color.a;
    var rgb = c.rgb;
    if s.ids.y == SKY_STARS && open_sky() && !in_liquid() {
        let star = star_look(in.star, in.uv, dir);
        rgb *= star.rgb;
        a *= star.a;
    }
    if open_sky() && !in_liquid() {
        // Bodies shine through the tone curve; stars dim in the haze low down.
        // The sun shines through the tone curve; the moon keeps its markings.
        var shape = 1.0;
        var tint_after = vec3(1.0);
        // Which body this is, by where the body itself stands (`axes[0]` points at it), not by how
        // near the pixel is to the sun: tested per pixel, a moon within 25° of the sun took the sun's
        // branch, and every other moon kept vanilla's square picture, which by day went white
        // through the curve as a big white square or, low down, a skewed one (Tester 2's
        // "square sun", 11-05: the moon of summer day 5, up by day).
        let is_sun = s.ids.y == SKY_BODY && dot(s.axes[0].xyz, scene.sun.xyz) > 0.9995;
        let is_moon = s.ids.y == SKY_BODY && !is_sun && dot(s.axes[0].xyz, scene.moon.xyz) > 0.9995;
        if is_moon {
            let m = moon_body(s, local);
            rgb = m.rgb;
            a = m.a * s.color.a;
        }
        if is_sun {
            // The sun as a round disc with a soft glow (Bliss's), not the picture's square: that
            // square, about 20° across with a pale border, went white through the curve, washed
            // out the golden hour (and the water's path with it), and turned into a diamond on the
            // leaned path. White-gold by day, a deep orange low down, the glow gone before the
            // quad's edges.
            let half = max(abs(s.uv.zw - s.uv.xy) * 0.5, vec2(1e-4));
            let r = length((in.uv - (s.uv.xy + s.uv.zw) * 0.5) / half);
            let disc = 1.0 - smoothstep(0.12, 0.15, r);
            let halo = exp(-r * 6.0) * (1.0 - smoothstep(0.55, 0.95, r)) * 0.7;
            let g = golden();
            rgb = mix(vec3(1.0, 0.96, 0.86), vec3(1.0, 0.52, 0.2), g) * (disc * mix(3.0, 1.5, g) + halo);
            shape = smoothstep(0.0, 0.004, disc + halo);
        }
        // A blood moon: copper, then a deep red disc, dimmer in totality.
        if is_moon && blood_moon() > 0.0 {
            // Tinted after the curve as well: the moon's bright picture went white through it, and a
            // copper moon showed only a faint pink (the sky-events check, 11-05).
            rgb *= blood_tint() * mix(1.0, 0.45, smoothstep(0.5, 1.0, blood_moon()));
            tint_after = blood_tint();
        }
        if is_moon {
            shape = smoothstep(0.0, 0.004, a);
        }
        a *= 1.0 - rain_amount() * 0.9;
        // `display` lifts black a little: outside the sun's shape nothing may be added.
        rgb = display(rgb) * shape * tint_after;
    }
    return vec4(rgb * a, select(a, 0.0, s.ids.w != 0u));
}

// What the look works out once a frame (`client::frame_look`), drawn before the world: the palette,
// a member a texel (`look.wgsl`'s `palette`).
@fragment
fn fs_frame_look(in: CloudOut) -> @location(0) vec4<f32> {
    let p = palette_made();
    switch u32(in.clip.x) {
        case 0u: { return vec4(p.zenith, 1.0); }
        case 1u: { return vec4(p.horizon, 1.0); }
        case 2u: { return vec4(p.glow, 1.0); }
        case 3u: { return vec4(p.sun, 1.0); }
        case 4u: { return vec4(p.ambient, 1.0); }
        default: { return vec4(0.0); }
    }
}

