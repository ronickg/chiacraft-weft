// Far shade from the sky, its half-size part (`post_low`; filtered and applied in `post/far_ao.wgsl`).
// How much of the sky a point far off cannot see for what stands round it: the gaps between tree
// crowns, the foot of a hill's step, the ground under a far overhang. The blocks' own corner shading
// and the contact shadows (`contact`, gone past 64 blocks) are a block wide; past them nothing
// shaded those gaps, and a far canopy's floor read as bright as open grass (the vista's twilight
// gaps 70 against Bliss's 60, Trees' stage diff, 11-06). Bliss's way, our own code: a few samples
// round each point in a ring a fixed share of the screen wide, so the ring in blocks grows with the
// distance, and what rises above the point's surface inside it closes it in. It darkens the sky's
// light only, as near as a screen pass can tell (where the shadow maps say the sun reaches a face
// turned to it, little), never the sky or the clouds.

const FAR_AO_SAMPLES: i32 = 7;
// The ring's radius, a share of the screen's height.
const FAR_AO_RADIUS: f32 = 0.022;
// How dark a fully closed-in point goes (0..1 of its light kept at the most closed).
const FAR_AO_STRENGTH: f32 = 0.6;

// The surface's normal at `uv` from its neighbours' depth, the neighbour on the same surface on
// each axis (the contact shadows' way), or none on a thin edge (a leaf's, a blade's).
fn far_ao_normal(p: PostPixel, uv: vec2<f32>, pos: vec3<f32>) -> vec3<f32> {
    let px = 1.0 / p.size;
    let l = post_position(p, uv - vec2(px.x, 0.0)) - pos;
    let r = post_position(p, uv + vec2(px.x, 0.0)) - pos;
    let u = post_position(p, uv - vec2(0.0, px.y)) - pos;
    let d = post_position(p, uv + vec2(0.0, px.y)) - pos;
    let dx = select(-l, r, dot(r, r) < dot(l, l));
    let dy = select(-u, d, dot(d, d) < dot(u, u));
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

// 1 where the sun reaches camera-relative `rel` by the shadow maps (the near one, else the far
// one), 1 also out of their reach. With traced shadows (`scene.rt`) the near map holds the models
// alone, so the far one (the terrain) is asked as well and the darker counts (light in the air's
// way, `post_low/volume.wgsl`): else terrain within the near map's reach counted as sunlit.
fn far_ao_sun(rel: vec3<f32>) -> f32 {
    let traced = scene.rt.x > 0.5;
    var lit = 1.0;
    if scene.shadow.z > 0.5 {
        let c = scene.shadow_view_proj * vec4(rel, 1.0);
        let ndc = c.xyz / c.w;
        if abs(ndc.x) < 0.98 && abs(ndc.y) < 0.98 && ndc.z > 0.0 && ndc.z < 1.0 {
            let at = vec2<i32>(vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * scene.shadow.y);
            let d = textureLoad(textures[u32(scene.shadow.x)], at, 0).r;
            lit = select(0.0, 1.0, d <= ndc.z + 0.0005);
            if !traced || lit < 0.5 {
                return lit;
            }
        }
    }
    if scene.shadow_far.z > 0.5 {
        let c = scene.shadow_far_view_proj * vec4(rel, 1.0);
        let ndc = c.xyz / c.w;
        if abs(ndc.x) < 0.98 && abs(ndc.y) < 0.98 && ndc.z > 0.0 && ndc.z < 1.0 {
            let at = vec2<i32>(vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * scene.shadow_far.y);
            let d = textureLoad(textures[u32(scene.shadow_far.x)], at, 0).r;
            return min(lit, select(0.0, 1.0, d <= ndc.z + 0.0004));
        }
    }
    return lit;
}

// What share of this pixel's light it keeps (1 open), its ring turned by `turn` (0..1) or, with
// `turn` under 0, by the half-size image's ordered pattern.
fn far_ao(p: PostPixel, turn_in: f32) -> f32 {
    let depth = post_depth(p, p.uv);
    // The sky, or the eye in a liquid.
    if depth <= 0.0 || abs(scene.params.x - 0.6 * scene.params.y) > 0.5 {
        return 1.0;
    }
    let pos = post_position(p, p.uv);
    let dist = length(pos);
    // From where the contact shadows fade (24-64 blocks), to before the far haze's edge (the land
    // hands over to the sky there).
    let end = scene.params.y;
    let reach = smoothstep(24.0, 64.0, dist) * (1.0 - smoothstep(end * 0.6, end * 0.84, dist));
    // Clouds write depth: their underside less a margin (the pack's own test, `sky_or_cloud`).
    if reach <= 0.0 || (scene.clouds.x > 0.5 && pos.y > scene.clouds.z - 20.0) {
        return 1.0;
    }
    let n = far_ao_normal(p, p.uv, pos);
    if dot(n, n) < 0.5 {
        return 1.0;
    }
    // The ring in blocks: the screen's share at this distance (a block's width on screen).
    let c0 = scene.view_proj * vec4(pos, 1.0);
    let c1 = scene.view_proj * vec4(pos + scene.right.xyz, 1.0);
    let per_block = max(length(c1.xy / c1.w - c0.xy / c0.w) * 0.5, 1e-5);
    let ring = FAR_AO_RADIUS * 2.0 / per_block;
    // Start turned by a 4x4 ordered pattern over this half-size image (2x2 at Low), the same every
    // frame: the full-size part averages each block by depth, so every turn meets in a pixel.
    let cheap = p.strength < 0.75;
    let px = vec2<u32>(p.uv * p.size * 0.5);
    let bayer = array<u32, 16>(0u, 8u, 2u, 10u, 12u, 4u, 14u, 6u, 3u, 11u, 1u, 9u, 15u, 7u, 13u, 5u);
    let bayer2 = array<u32, 4>(0u, 2u, 3u, 1u);
    var turn = (f32(bayer[(px.y & 3u) * 4u + (px.x & 3u)]) + 0.5) / 16.0;
    if cheap {
        turn = (f32(bayer2[(px.y & 1u) * 2u + (px.x & 1u)]) + 0.5) / 4.0;
    }
    if turn_in >= 0.0 {
        turn = turn_in;
    }
    let aspect = p.size.y / p.size.x;
    var closed = 0.0;
    for (var i = 0; i < FAR_AO_SAMPLES; i++) {
        // A Vogel spiral: evenly over the disc.
        let r = sqrt((f32(i) + 0.5) / f32(FAR_AO_SAMPLES));
        let a = (f32(i) * 2.3999632 + turn * 6.2831853);
        let at = p.uv + vec2(cos(a) * aspect, sin(a)) * r * FAR_AO_RADIUS;
        if at.x < 0.0 || at.x > 1.0 || at.y < 0.0 || at.y > 1.0 || post_depth(p, at) <= 0.0 {
            continue;
        }
        let to = post_position(p, at) - pos;
        let d2 = dot(to, to);
        if d2 > ring * ring {
            continue;
        }
        // What stands above this point, by the world's height, closes it in, nearer the more: the
        // ground between far crowns, with crowns round it; not a crown's own top, whose bumps and
        // leaf holes read concave by the depth's normal and took full shade at twilight (crowns
        // 53 -> 27 at 19:00 against Bliss's 55, Trees' crowngap read of the first try).
        if to.y > 1.0 {
            let rise = to.y * inverseSqrt(max(d2, 1e-6));
            closed += smoothstep(0.2, 0.8, rise) * (1.0 - d2 / (ring * ring));
        }
    }
    var ao = 1.0 - clamp(closed / f32(FAR_AO_SAMPLES) * FAR_AO_STRENGTH, 0.0, 0.8);
    // Or as strong as Bliss's (`base:look_ao_power`, the shader pack's option 41, read raw: 0 as
    // made): what is left open to the sky raised to that power (Bliss's SSAO^6 on the sky light,
    // `composite1.fsh:1180-1188`), so a gap half closed in goes near black. Its crown interiors, the
    // ground between crowns and the shaded sides went navy-black where ours stayed lit (controlled
    // frames, 11-22: noon shade 0 19 65 against ours 15 32 67).
    let power = scene.options[10][1];
    if power > 0.0 {
        ao = pow(clamp(1.0 - closed / f32(FAR_AO_SAMPLES), 0.0, 1.0), power);
    }
    // The sky's light only: a face the sun reaches keeps most of its light (the sun's), by day.
    let sun = scene.sun;
    let day = smoothstep(0.0, 0.15, sun.y);
    let sunlit = far_ao_sun(pos) * smoothstep(0.0, 0.3, dot(n, sun.xyz)) * day * (1.0 - clamp(scene.fog_color.w, 0.0, 1.0));
    ao = mix(ao, 1.0, sunlit * 0.8);
    return mix(1.0, ao, reach * select(min(p.strength, 1.0), 1.0, cheap));
}

fn low(p: PostPixel) -> vec4<f32> {
    return vec4(far_ao(p, -1.0), 0.0, 0.0, 1.0);
}
