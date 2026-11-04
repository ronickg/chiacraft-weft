// Contact shadows (chiacraft's shader pack turns them on, `mods/base/src/shaderpack.rs`): how much
// of the space around each point is closed in by what is drawn near it, from the world's depth.
// Corners, the ground under an overhang, the foot of a wall and the gaps between leaves darken a
// little more than the blocks' own corner shading does; open ground and the sky stay as they are.
// Kept gentle on purpose (2 to 4% in shaded spots, Ronald's choice on 2026-11-04): the sun shadows
// do the visible shading, and stronger darkening here once darkened grass and the held hand.

const CONTACT_DIRS: i32 = 8;

fn contact_hash(p: vec2<f32>) -> f32 {
    let q = vec2<u32>(p);
    var h = q.x * 2654435u + q.y * 9781u + 1013u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    return f32((h ^ (h >> 15u)) & 1023u) / 1024.0;
}

// The surface's normal at `uv` from its neighbours' depth, taking on each axis the neighbour on the
// same surface (the nearer in depth), so block edges do not smear it.
fn contact_normal(p: PostPixel, uv: vec2<f32>, pos: vec3<f32>) -> vec3<f32> {
    let px = 1.0 / p.size;
    let l = post_position(p, uv - vec2(px.x, 0.0)) - pos;
    let r = post_position(p, uv + vec2(px.x, 0.0)) - pos;
    let u = post_position(p, uv - vec2(0.0, px.y)) - pos;
    let d = post_position(p, uv + vec2(0.0, px.y)) - pos;
    let dx = select(-l, r, dot(r, r) < dot(l, l));
    let dy = select(-u, d, dot(d, d) < dot(u, u));
    // On a thin edge (a grass blade, a flower) both neighbours are on something else: no slope.
    let gap = length(pos) * 0.08 + 0.3;
    if min(dot(l, l), dot(r, r)) > gap * gap || min(dot(u, u), dot(d, d)) > gap * gap {
        return vec3(0.0);
    }
    var n = normalize(cross(dy, dx) + vec3(0.0, 1e-6, 0.0));
    if dot(n, pos) > 0.0 {
        n = -n;
    }
    return n;
}

fn apply(p: PostPixel) -> vec4<f32> {
    let depth = post_depth(p, p.uv);
    // The sky, or the eye in a liquid.
    if depth <= 0.0 || abs(scene.params.x - 0.6 * scene.params.y) > 0.5 {
        return p.color;
    }
    let pos = post_position(p, p.uv);
    let dist = length(pos);
    // The hand (drawn with its own projection, within reach of the eye) and far away (noise).
    if dist < 1.6 || dist > 64.0 {
        return p.color;
    }
    let n = contact_normal(p, p.uv, pos);
    if dot(n, n) < 0.5 {
        return p.color;
    }
    // Look about a block around: its size on the screen at this distance.
    let reach = 1.2;
    let c = scene.view_proj * vec4(pos + vec3(reach, 0.0, 0.0), 1.0);
    let c0 = scene.view_proj * vec4(pos, 1.0);
    let radius = clamp(length(c.xy / c.w - c0.xy / c0.w) * 0.5, 2.0 / p.size.y, 0.12);
    let turn = contact_hash(p.uv * p.size) * 6.2831853;
    var closed = 0.0;
    for (var i = 0; i < CONTACT_DIRS; i++) {
        let a = turn + f32(i) * 6.2831853 / f32(CONTACT_DIRS);
        let dir = vec2(cos(a), sin(a) * p.size.x / p.size.y);
        // Two distances along each direction.
        for (var k = 1; k <= 2; k++) {
            let at = p.uv + dir * radius * (f32(k) * 0.5);
            if at.x < 0.0 || at.x > 1.0 || at.y < 0.0 || at.y > 1.0 {
                continue;
            }
            if post_depth(p, at) <= 0.0 {
                continue;
            }
            let to = post_position(p, at) - pos;
            let len = length(to);
            // What rises above the surface close by closes it in; far behind or below, not.
            let rise = max(dot(n, to / max(len, 1e-4)) - 0.35, 0.0);
            closed += rise * (1.0 - smoothstep(reach * 0.6, reach * 1.6, len));
        }
    }
    closed /= f32(CONTACT_DIRS) * 2.0;
    let fade = 1.0 - smoothstep(24.0, 64.0, dist);
    let dark = 1.0 - clamp(closed * 2.8, 0.0, 0.5) * fade * min(p.strength, 1.0);
    return vec4(p.color.rgb * dark, p.color.a);
}
