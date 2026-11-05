// Screen-space reflections on water (chiacraft's shader pack turns them on): what the water mirrors is
// marched at half the size each way (`shaders/post_low/reflect.wgsl`, where the whole method is
// described) and laid over the water here, upsampled by depth; without the half-size image,
// marched here per pixel as before. Only the first effect sees the image's alpha (the water's
// share), so this stays first.

// The half-size reflections round `p`: only from those of the four that were water (`a` not -1)
// and lie about as far off as this pixel. Where none does, at the shore and along the far water
// (where every screen row lies much farther than the one below it, so a half-size pixel took its
// water from another distance or none), it is marched here instead: laid over from the land's
// nothing, those rows came out as bright and dark lines along the far shore (fx-half's lake shots).
fn reflect_upsampled(p: PostPixel) -> vec4<f32> {
    let low = textures[p.low - 1u];
    let dims = vec2<i32>(textureDimensions(low));
    let f = p.uv * vec2<f32>(dims) - 0.5;
    let base = vec2<i32>(floor(f));
    let t = fract(f);
    let here = length(post_position(p, p.uv));
    var sum = vec4(0.0);
    var weight = 0.0;
    for (var j = 0; j < 4; j++) {
        let o = vec2(j % 2, j / 2);
        let at = clamp(base + o, vec2(0), dims - 1);
        let r = textureLoad(low, at, 0);
        let uv = (vec2<f32>(at) + 0.5) / vec2<f32>(dims);
        let off = abs(length(post_position(p, uv)) - here) / max(here, 1.0);
        if r.a < 0.0 || off > 0.04 {
            continue;
        }
        let bil = mix(1.0 - t.x, t.x, f32(o.x)) * mix(1.0 - t.y, t.y, f32(o.y));
        let w = (bil + 0.05) / (0.01 + off);
        sum += r * w;
        weight += w;
    }
    if weight <= 0.0 {
        return reflect_result(p);
    }
    return sum / weight;
}

fn apply(p: PostPixel) -> vec4<f32> {
    // Not water: nothing to lay over.
    if p.color.a < 0.02 {
        return p.color;
    }
    var r: vec4<f32>;
    if p.low != 0u {
        r = reflect_upsampled(p);
    } else {
        r = reflect_result(p);
    }
    return vec4(p.color.rgb * (1.0 - r.a) + r.rgb, p.color.a);
}
