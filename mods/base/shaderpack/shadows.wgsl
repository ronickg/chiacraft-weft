// Not shipped: sun shadows traced block by block through `solid_at`. Taken out of `world.wgsl`
// after they hung the GPU (every pixel, plants' many layers too, walked up to 96 blocks, and in
// the open nearly every walk went the whole way). Before they go back: far fewer steps, an early
// way out above the terrain, and a measured frame time on a small window first.

// ---- sun shadows ----

// Kept small on purpose: every pixel of a block walks at most this far, so the shadows cost a
// bounded, modest amount on any GPU (a long walk per pixel can stall a laptop's whole screen).
// SHADOWS false turns them off altogether.
const SHADOWS: bool = false;
const SHADOW_STEPS: u32 = 32u;
const SHADOW_REACH: f32 = 28.0;
// Pixels further than this from the eye are not traced; the sky light stands in.
const SHADOW_DISTANCE: f32 = 64.0;

// 1 where nothing stands between `p` (world blocks) and the light, walking block by block toward it
// (Amanatides and Woo). Past the blocks the client keeps, the sky light decides instead.
fn traced_shadow(p: vec3<f32>, l: vec3<f32>) -> f32 {
    var cell = vec3<i32>(floor(p));
    let step = vec3<i32>(sign(l));
    let inv = 1.0 / max(abs(l), vec3(1e-5));
    let fr = p - floor(p);
    var next = select(fr, vec3(1.0) - fr, l > vec3(0.0)) * inv;
    var t = 0.0;
    for (var i = 0u; i < SHADOW_STEPS; i++) {
        if next.x < next.y && next.x < next.z {
            t = next.x;
            next.x += inv.x;
            cell.x += step.x;
        } else if next.y < next.z {
            t = next.y;
            next.y += inv.y;
            cell.y += step.y;
        } else {
            t = next.z;
            next.z += inv.z;
            cell.z += step.z;
        }
        if t > SHADOW_REACH {
            break;
        }
        if solid_at(cell) {
            return 0.0;
        }
    }
    return 1.0;
}

fn sun_shadow(in: VOut, n: vec3<f32>) -> f32 {
    let l = light_dir();
    // Plants (drawn in many layers over each other) are never traced: the sky light shades them.
    if !SHADOWS || in.kind.z != 0u {
        return smoothstep(0.75, 1.0, in.light.x);
    }
    if in.kind.y != 0u || light_strength() <= 0.0 || in.light.x < 0.45 || in.dist > SHADOW_DISTANCE {
        return 1.0;
    }
    // Off the face toward the light, so a block never shades itself; plants from inside their cell.
    var start = in.pos + l * 0.02;
    if in.kind.z == 0u {
        if dot(n, l) <= 0.0 {
            return 1.0;
        }
        start += n * 0.01;
    }
    let s = traced_shadow(start, l);
    return mix(s, 1.0, smoothstep(SHADOW_DISTANCE * 0.75, SHADOW_DISTANCE, in.dist));
}

