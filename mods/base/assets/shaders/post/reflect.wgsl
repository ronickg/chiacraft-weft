// Water reflections (chiacraft's shader pack turns them on, `mods/base/src/shaderpack.rs`): where
// water lies (the world image's alpha, which the pack leaves at 0 for solid things, and the water's
// surface in the depth, `translucent_depth`), the reflected ray is walked across the screen until
// it passes behind what is drawn there, and that is mirrored in the water: hills, trees, the sun.
// Where it finds nothing it mirrors the sky as the screen shows it (clouds and all) when that part
// of the sky is on the screen, and else keeps the sky the pack already put in it. It must run
// before other effects: only the first sees the alpha.

const REFLECT_STEPS: i32 = 40;

fn reflect_hash(p: vec2<f32>) -> f32 {
    let q = vec2<u32>(p);
    var h = q.x * 1973u + q.y * 9277u + 26699u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    return f32((h ^ (h >> 15u)) & 1023u) / 1024.0;
}

// The water's swells as the pack's water draws them (`wave_height` in its `world.wgsl`, without the
// chop): the slope at `p` (world xz), flatter far off. The mirror wobbles with the surface.
// `glance`: how steeply the eye looks down at it. At a glancing look water is a mirror (the swells'
// slopes average out), so they are flattened there, or the far water would mirror random bits of
// sky and ground instead of the far shore.
fn reflect_swell(p: vec2<f32>, t: f32, dist: f32, glance: f32) -> vec3<f32> {
    let w = scene.weather.xy;
    let speed = length(w);
    let dir = select(vec2(0.8, 0.6), w / max(speed, 1e-4), speed > 0.3);
    let side = vec2(-dir.y, dir.x);
    let d1 = normalize(dir + side * 0.35);
    let d2 = normalize(dir - side * 0.85);
    let calm = 0.6 + 0.4 * smoothstep(1.0, 8.0, speed);
    let g = d1 * cos(dot(p, d1) * 0.9 - t * 0.85) * 0.054 + d2 * cos(dot(p, d2) * 1.37 - t * 1.1 + 1.7) * 0.055;
    let k = calm * mix(1.0, 0.25, smoothstep(24.0, 140.0, dist)) * mix(0.15, 1.0, smoothstep(0.03, 0.4, glance));
    return normalize(vec3(-g.x * k, 1.0, -g.y * k));
}

// How much of what it shows water mirrors at the angle it is seen at (the pack's `water` uses the
// same): a near mirror at an ordinary look across it, as in the reference pictures.
fn reflect_fresnel(ndv: f32) -> f32 {
    return 0.08 + 0.92 * pow(1.0 - clamp(ndv, 0.0, 1.0), 2.5);
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

fn apply(p: PostPixel) -> vec4<f32> {
    let water = p.color.a;
    // Not water, the eye in a liquid, or nothing drawn.
    if water < 0.02 || abs(scene.params.x - 0.6 * scene.params.y) > 0.5 {
        return p.color;
    }
    if post_depth(p, p.uv) <= 0.0 {
        return p.color;
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
        return p.color;
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
    let mask = smoothstep(0.02, 0.3, water) * select(min(p.strength, 1.0), 1.0, cheap);
    // Walk out along it, the steps growing, from a random start against banding. Past something
    // thin without a clean hit it keeps going; a hit is narrowed down between the last two steps.
    let steps = select(REFLECT_STEPS, 16, cheap);
    let grow = select(1.15, 1.5, cheap);
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
        if far.w < 0.0 || far.x < 0.0 || far.x > 1.0 || far.y < 0.0 || far.y > 1.0 || post_depth(p, far.xy) > 0.0 {
            return p.color;
        }
        let sky = textureSampleLevel(textures[p.scene], linear_clamp, far.xy, 0.0).rgb;
        let edge = smoothstep(0.0, 0.1, min(min(far.x, 1.0 - far.x), min(far.y, 1.0 - far.y)));
        // The pack's own sky is already in the water at this share; this only replaces it.
        let amount = clamp(fresnel, 0.0, 0.97) * mask * edge;
        return vec4(mix(p.color.rgb, sky, amount), p.color.a);
    }
    let mirrored = textureSampleLevel(textures[p.scene], linear_clamp, hit, 0.0).rgb;
    // Less toward the screen's edges and far along the ray; as much as the water mirrors at that
    // angle (Fresnel: little looking down, nearly all at a glancing look), the sky's share of
    // which the water already shows.
    let edge = smoothstep(0.0, 0.12, min(min(hit.x, 1.0 - hit.x), min(hit.y, 1.0 - hit.y)));
    // Hills across a lake are hundreds of blocks off: only the very far fades.
    let near = 1.0 - smoothstep(300.0, 700.0, travelled);
    // The mask is the water's own opacity: low only in clear shallows seen from above. A little more
    // than the Fresnel share: the water's own body under it is dark, so what it mirrors carries.
    let amount = clamp(fresnel + 0.15, 0.0, 0.97) * mask * edge * near;
    return vec4(mix(p.color.rgb, mirrored, amount), p.color.a);
}
