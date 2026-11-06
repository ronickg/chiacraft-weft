// Screen-space reflections on water, marched per pixel (chiacraft's shader pack turns them on).
// fx-half marched them at half the size each way, but at a glancing look the far water packs many
// screen rows into each half-size row and the mirrored far shore landed on the wrong rows as lines
// (the golden lake shot; gone with reflections off, not with contact or bloom), and marching the
// level band per pixel still left them lower down. Per pixel again until a better upsample.
// Water reflections (chiacraft's shader pack turns them on, `mods/base/src/shaderpack.rs`): where
// water lies (the world image's alpha, which the pack leaves at 0 for solid things, and the water's
// surface in the depth, `translucent_depth`), the reflected ray is walked across the screen until
// it passes behind what is drawn there, and that is mirrored in the water: hills, trees, the sun.
// Where it finds nothing it mirrors the sky as the screen shows it (clouds and all) when that part
// of the sky is on the screen, and else keeps the sky the pack already put in it. It must run
// before other effects: only the first sees the alpha.

// 28 steps growing 1.245 a step reach as far as 40 growing 1.15 did (about 860 blocks); a hit is
// still narrowed down between the last two. Fewer steps: the march over water was most of the
// Mac's effects time after a flight over the sea.
const REFLECT_STEPS: i32 = 28;

fn reflect_hash(p: vec2<f32>) -> f32 {
    let q = vec2<u32>(p);
    var h = q.x * 1973u + q.y * 9277u + 26699u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    return f32((h ^ (h >> 15u)) & 1023u) / 1024.0;
}

// The water's ripples as the pack's water draws them (`wave_height` in its `world.wgsl`, without
// the finest layer): layers of smooth noise, stretched and turned by the golden angle, sliding with
// the wind. The same hash and noise as the pack's `look.wgsl` (screen effects can't reach it), so
// the mirror wobbles with the surface. The slope at `p` (world xz) over a step widening with
// distance, flatter far off. `glance`: how steeply the eye looks down at it. At a glancing look
// water is a mirror (the slopes average out), so they are flattened there, or the far water would
// mirror random bits of sky and ground instead of the far shore.
fn reflect_hash3(p: vec3<i32>) -> f32 {
    var h = bitcast<u32>(p.x) * 73856093u ^ bitcast<u32>(p.y) * 19349663u ^ bitcast<u32>(p.z) * 83492791u;
    h = (h ^ (h >> 16u)) * 0x45d9f3bu;
    h = (h ^ (h >> 16u)) * 0x45d9f3bu;
    h = h ^ (h >> 16u);
    return f32(h & 0xffffffu) / 16777215.0;
}

fn reflect_noise3(p: vec3<f32>) -> f32 {
    let i = vec3<i32>(floor(p));
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = mix(mix(reflect_hash3(i), reflect_hash3(i + vec3(1, 0, 0)), u.x), mix(reflect_hash3(i + vec3(0, 1, 0)), reflect_hash3(i + vec3(1, 1, 0)), u.x), u.y);
    let b = mix(mix(reflect_hash3(i + vec3(0, 0, 1)), reflect_hash3(i + vec3(1, 0, 1)), u.x), mix(reflect_hash3(i + vec3(0, 1, 1)), reflect_hash3(i + vec3(1, 1, 1)), u.x), u.y);
    return mix(a, b, u.z);
}

fn reflect_layer(p: vec2<f32>, t: f32, k: f32, cell: vec2<f32>, amp: f32) -> f32 {
    let a = k * 2.39996;
    let q = mat2x2<f32>(cos(a), sin(a), -sin(a), cos(a)) * p;
    let n = reflect_noise3(vec3((q - vec2(1.6 * t, 0.0)) / cell, t * 0.08 + k * 17.0));
    return (n - 0.5) * amp;
}

fn reflect_height(p: vec2<f32>, t: f32, dir: vec2<f32>) -> f32 {
    let q = mat2x2<f32>(dir.x, -dir.y, dir.y, dir.x) * p;
    return reflect_layer(q, t, 0.0, vec2(6.0, 1.6), 0.10) + reflect_layer(q, t, 1.0, vec2(1.6, 6.0), 0.08)
        + reflect_layer(q, t, 2.0, vec2(3.2, 3.2), 0.07);
}

fn reflect_swell(p: vec2<f32>, t: f32, dist: f32, glance: f32) -> vec3<f32> {
    let w = scene.weather.xy;
    let speed = length(w);
    let dir = select(vec2(0.8, 0.6), w / max(speed, 1e-4), speed > 0.3);
    let calm = 0.7 + 0.3 * smoothstep(1.0, 8.0, speed);
    let e = 0.12 + min(dist / 200.0, 3.0);
    let h = reflect_height(p, t, dir);
    let gx = (reflect_height(p + vec2(e, 0.0), t, dir) - h) / e;
    let gz = (reflect_height(p + vec2(0.0, e), t, dir) - h) / e;
    let k = calm * mix(1.0, 0.5, smoothstep(24.0, 160.0, dist)) * mix(0.15, 1.0, smoothstep(0.03, 0.4, glance));
    return normalize(vec3(-gx * k, 1.0, -gz * k));
}

// How much of what it shows water mirrors at the angle it is seen at (the pack's `water_fresnel`,
// the same): a fourth power from 0.04, muted navy water as Bliss's in its own world (Far view,
// 11-06; it was a near mirror at an ordinary look, ^2 from 0.08).
fn reflect_fresnel(ndv: f32) -> f32 {
    return 0.04 + 0.96 * pow(1.0 - clamp(ndv, 0.0, 1.0), 4.0);
}

// Where `rel` (camera relative) lands on the screen, and its depth there.
fn reflect_project(rel: vec3<f32>) -> vec4<f32> {
    let c = scene.view_proj * vec4(rel, 1.0);
    if c.w <= 0.0 {
        return vec4(-1.0);
    }
    let ndc = c.xyz / c.w;
    return vec4(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, ndc.z, 1.0);
}

// How much the screen's reflections may show at `pos` (camera relative): none toward where the
// cubes end. Past them the water is distant terrain, which is left out of these reflections (for
// their cost), so near water mirrored the far shore's trees right up to that square's edge and the
// far water beyond did not: a hard-edged rectangle of reflection, its sides straight (Ronald's
// lake on the Mac, at a smaller view distance; the LOD worker's sea views, 11-05). The view
// distance in chunks is the pack's 15th option (`core:view_distance`, `mods/base/src/look.rs`).
fn reflect_reach(pos: vec3<f32>) -> f32 {
    let chunks = scene.options[3].z;
    if chunks <= 0.0 {
        return 1.0;
    }
    let end = chunks * 16.0;
    return 1.0 - smoothstep(end - 72.0, end - 24.0, max(abs(pos.x), abs(pos.z)));
}

// What the water mirrors at this pixel, premultiplied by how much (rgb), and how much (a): 0 where
// it mirrors nothing. Premultiplied, a shore's half-size texels average to less mirror, not to black.
fn reflect_result(p: PostPixel) -> vec4<f32> {
    let water = p.color.a;
    // Not water, the eye in a liquid, or nothing drawn.
    if water < 0.02 || abs(scene.params.x - 0.6 * scene.params.y) > 0.5 {
        return vec4(0.0);
    }
    if post_depth(p, p.uv) <= 0.0 {
        return vec4(0.0);
    }
    let pos = post_position(p, p.uv);
    // The surface's slope from its neighbours' depth: only what lies flat, below the eye.
    let px = 1.0 / p.size;
    let right = post_position(p, p.uv + vec2(px.x, 0.0)) - pos;
    let down = post_position(p, p.uv + vec2(0.0, px.y)) - pos;
    var n = normalize(cross(down, right) + vec3(0.0, 1e-6, 0.0));
    if dot(n, pos) > 0.0 {
        n = -n;
    }
    if n.y < 0.9 || pos.y > -0.3 {
        return vec4(0.0);
    }
    let v = normalize(pos);
    n = reflect_swell(pos.xz + scene.eye.xz, scene.params.w, length(pos), -v.y);
    var r = reflect(v, n);
    // A swell tilted so far that the ray would go down: mirror as if it were flat.
    if r.y <= 0.0 {
        n = vec3(0.0, 1.0, 0.0);
        r = reflect(v, n);
    }
    let fresnel = reflect_fresnel(dot(-v, n));
    // Strength under 0.75 (the Low setting): as strong, in 16 longer steps that reach as far.
    let cheap = p.strength < 0.75;
    let mask = smoothstep(0.02, 0.3, water) * select(min(p.strength, 1.0), 1.0, cheap) * reflect_reach(pos);
    if mask <= 0.0 {
        return vec4(0.0);
    }
    // Walk out along it, the steps growing, from a random start against banding. Past something
    // thin without a clean hit it keeps going; a hit is narrowed down between the last two steps.
    let steps = select(REFLECT_STEPS, 16, cheap);
    let grow = select(1.245, 1.5, cheap);
    var step = 0.5;
    var dist = step * (0.5 + reflect_hash(p.uv * p.size));
    var before = 0.0;
    var hit = vec2(-1.0);
    var travelled = 0.0;
    for (var i = 0; i < steps; i++) {
        let at = pos + r * dist;
        let s = reflect_project(at);
        if s.w < 0.0 || s.x < 0.0 || s.x > 1.0 || s.y < 0.0 || s.y > 1.0 {
            break;
        }
        let d = post_depth(p, s.xy);
        // Reverse Z: what is drawn there is nearer than the ray's point, so the ray went behind it.
        if d > s.z && d > 0.0 && length(at) - length(post_position(p, s.xy)) < step * 2.5 + 1.0 {
            var lo = before;
            var hi = dist;
            for (var k = 0; k < 5; k++) {
                let mid = (lo + hi) * 0.5;
                let m = reflect_project(pos + r * mid);
                if post_depth(p, m.xy) > m.z {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            hit = reflect_project(pos + r * hi).xy;
            travelled = hi;
            break;
        }
        before = dist;
        dist += step;
        step *= grow;
    }
    if hit.x < 0.0 {
        // Nothing in the way: the sky that way, from the screen where it shows there (its clouds,
        // the sun), so the water is not a flat painted blue.
        let far = reflect_project(r * 4096.0);
        if far.w < 0.0 || far.x < 0.0 || far.x > 1.0 || far.y < 0.0 || far.y > 1.0 {
            return vec4(0.0);
        }
        // Where something is drawn at that spot (land at the horizon, a trunk), look a little
        // higher up the screen for the sky, and give it less the further up it was: switching
        // straight from the screen's sky to the pack's own at that spot drew hard straight lines
        // in the water, level along the horizon and upright along trunks (Ronald's lake, the LOD
        // worker's sea views, 11-05).
        var at = far.xy;
        var found = post_depth(p, at) <= 0.0;
        var reach = 0.0;
        for (var k = 1; k <= 6 && !found; k++) {
            reach = f32(k) / 6.0;
            at = vec2(far.x, far.y - reach * 0.06);
            found = at.y > 0.0 && post_depth(p, at) <= 0.0;
        }
        if !found {
            return vec4(0.0);
        }
        let sky = textureSampleLevel(textures[p.scene], linear_clamp, at, 0.0).rgb;
        let edge = smoothstep(0.0, 0.1, min(min(at.x, 1.0 - at.x), min(at.y, 1.0 - at.y)));
        // The pack's own sky is already in the water at this share; this only replaces it.
        let amount = clamp(fresnel, 0.0, 0.97) * mask * edge * (1.0 - reach);
        return vec4(sky * amount, amount);
    }
    // Blurred more the farther the ray went before it hit (a reflection in rippled water softens
    // with distance): the hit and four samples about it. One sharp sample turned dark tree crowns
    // over a bright sky into a hard black-and-white mosaic looking down at a lake (Tester 2).
    let spread = clamp(travelled * 0.0006, 0.002, 0.014) * vec2(p.size.y / p.size.x, 1.0);
    var mirrored = textureSampleLevel(textures[p.scene], linear_clamp, hit, 0.0).rgb * 2.0;
    mirrored += textureSampleLevel(textures[p.scene], linear_clamp, clamp(hit + vec2(spread.x, 0.0), vec2(0.0), vec2(1.0)), 0.0).rgb;
    mirrored += textureSampleLevel(textures[p.scene], linear_clamp, clamp(hit - vec2(spread.x, 0.0), vec2(0.0), vec2(1.0)), 0.0).rgb;
    mirrored += textureSampleLevel(textures[p.scene], linear_clamp, clamp(hit + vec2(0.0, spread.y), vec2(0.0), vec2(1.0)), 0.0).rgb;
    mirrored += textureSampleLevel(textures[p.scene], linear_clamp, clamp(hit - vec2(0.0, spread.y), vec2(0.0), vec2(1.0)), 0.0).rgb;
    mirrored /= 6.0;
    // Less toward the screen's edges and far along the ray; as much as the water mirrors at that
    // angle (Fresnel: little looking down, nearly all at a glancing look), the sky's share of
    // which the water already shows.
    let edge = smoothstep(0.0, 0.12, min(min(hit.x, 1.0 - hit.x), min(hit.y, 1.0 - hit.y)));
    // Hills across a lake are hundreds of blocks off: only the very far fades.
    let near = 1.0 - smoothstep(300.0, 700.0, travelled);
    // The mask is the water's own opacity: low only in clear shallows seen from above. A little more
    // than the Fresnel share: the water's own body under it is dark, so what it mirrors carries.
    // That extra fades looking steeply down, where the water shows its body more than what it mirrors.
    let steep = smoothstep(0.5, 0.9, dot(-v, n));
    let amount = clamp(fresnel + 0.15 * (1.0 - steep), 0.0, 0.97) * mask * edge * near;
    return vec4(mirrored * amount, amount);
}

fn apply(p: PostPixel) -> vec4<f32> {
    let r = reflect_result(p);
    return vec4(p.color.rgb * (1.0 - r.a) + r.rgb, p.color.a);
}
