// Far shade from the sky (chiacraft's shader pack turns it on, `mods/base/src/shaderpack.rs`): how
// much of the sky a point 24 blocks off and further cannot see for what stands round it, worked out
// at half the size each way (`shaders/post_low/far_ao.wgsl`) and averaged here over the block of
// half-size texels on this pixel's own surface, so its ordered start pattern leaves no grain and a
// darkening keeps to its surface. Without the half-size image, worked out here from four turns.
fn apply(p: PostPixel) -> vec4<f32> {
    var keep: f32;
    if p.low != 0u {
        keep = far_ao_filtered(p, select(4, 2, p.strength < 0.75));
    } else {
        // At full size (the half-size parts off, as the Max preset has them) no block to average
        // over: four rings evenly turned, 28 samples, so no pattern is left either.
        keep = (far_ao(p, 0.0) + far_ao(p, 0.25) + far_ao(p, 0.5) + far_ao(p, 0.75)) * 0.25;
    }
    return vec4(p.color.rgb * keep, p.color.a);
}

// The half-size share kept at this pixel from the `n` x `n` texels round it (4, or 2 at Low) on its
// own surface: any such block holds each of the start turns once, so their average has no grain.
fn far_ao_filtered(p: PostPixel, n: i32) -> f32 {
    let low = textures[p.low - 1u];
    let dims = vec2<i32>(textureDimensions(low));
    let f = p.uv * vec2<f32>(dims) - 0.5;
    let base = vec2<i32>(floor(f)) - vec2(n / 2 - 1);
    let here = length(post_position(p, p.uv));
    var sum = 0.0;
    var weight = 0.0;
    for (var j = 0; j < n * n; j++) {
        let at = clamp(base + vec2(j % n, j / n), vec2(0), dims - 1);
        let uv = (vec2<f32>(at) + 0.5) / vec2<f32>(dims);
        let there = length(post_position(p, uv));
        let w = select(1e-4, 1.0, abs(there - here) < 0.08 * here + 0.5);
        sum += textureLoad(low, at, 0).r * w;
        weight += w;
    }
    return sum / weight;
}
