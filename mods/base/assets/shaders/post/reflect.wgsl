// Water reflections (chiacraft's shader pack turns them on, `mods/base/src/shaderpack.rs`): where
// water lies (the world image's alpha, which the pack leaves at 0 for solid things, and the water's
// surface in the depth, `translucent_depth`), the reflected ray is walked across the screen until
// it passes behind what is drawn there, and that is mirrored in the water: hills, trees, the sun.
// Where it finds nothing on the screen the water keeps the sky the pack already put in it. It must
// run before other effects: only the first sees the alpha.

const REFLECT_STEPS: i32 = 40;

fn reflect_hash(p: vec2<f32>) -> f32 {
    let q = vec2<u32>(p);
    var h = q.x * 1973u + q.y * 9277u + 26699u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    return f32((h ^ (h >> 15u)) & 1023u) / 1024.0;
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
    // A slight ripple, so the mirror is water.
    let world = pos.xz + scene.eye.xz;
    let t = p.time;
    let ripple = vec2(sin(world.x * 1.7 + t * 1.3) + sin(world.y * 2.3 - t * 1.1), sin(world.y * 1.9 + t * 1.5) + sin(world.x * 2.9 + t * 0.9));
    n = normalize(vec3(ripple.x * 0.012, 1.0, ripple.y * 0.012));
    let v = normalize(pos);
    let r = reflect(v, n);
    if r.y <= 0.0 {
        return p.color;
    }
    // Walk out along it, the steps growing, from a random start against banding. Past something
    // thin without a clean hit it keeps going; a hit is narrowed down between the last two steps.
    var step = 0.5;
    var dist = step * (0.5 + reflect_hash(p.uv * p.size));
    var before = 0.0;
    var hit = vec2(-1.0);
    var travelled = 0.0;
    for (var i = 0; i < REFLECT_STEPS; i++) {
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
        step *= 1.15;
    }
    if hit.x < 0.0 {
        return p.color;
    }
    let mirrored = textureSampleLevel(textures[p.scene], linear_clamp, hit, 0.0).rgb;
    // Less toward the screen's edges and far along the ray; more at a glancing look (Fresnel).
    let edge = smoothstep(0.0, 0.12, min(min(hit.x, 1.0 - hit.x), min(hit.y, 1.0 - hit.y)));
    // Hills across a lake are hundreds of blocks off: only the very far fades.
    let near = 1.0 - smoothstep(300.0, 700.0, travelled);
    let fresnel = 0.02 + 0.98 * pow(1.0 - max(dot(-v, n), 0.0), 5.0);
    let amount = clamp(fresnel * 0.9 + 0.1, 0.0, 0.85) * clamp(water, 0.0, 1.0) * edge * near * min(p.strength, 1.0);
    return vec4(mix(p.color.rgb, mirrored, amount), p.color.a);
}
