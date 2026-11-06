// Volumetric light (chiacraft's shader pack turns it on, `mods/base/src/shaderpack.rs`): the light
// the air sends toward the eye where the sun reaches it, marched at half the size each way by its
// half-size part (`shaders/post_low/volume.wgsl`, Research round 16 #7: about a quarter of the
// cost) and drawn here from it, upsampled by depth so beams don't bleed over silhouettes. Without
// the half-size image (an engine without `post_low`), marched here per pixel as before.
fn apply(p: PostPixel) -> vec4<f32> {
    var glow: vec3<f32>;
    if p.low != 0u && p.strength >= 0.75 {
        glow = vol_filtered(p);
    } else if p.low != 0u {
        glow = post_low_upsampled(p).rgb;
    } else {
        glow = vol_glow(p);
    }
    return vec4(p.color.rgb + glow * (vec3(1.0) - p.color.rgb * 0.7), p.color.a);
}

// The half-size glow at this pixel from the 4 x 4 texels around it, each weighted by how near its
// depth is to this pixel's: any 4 x 4 block holds each of the march's sixteen start offsets once
// (`post_low/volume.wgsl`), so their even average has no grain, and a texel on the
// far side of a silhouette counts little, so beams don't bleed over trunks (Bliss's way: a
// depth-aware filter over the fog's dither before anything else reads it).
fn vol_filtered(p: PostPixel) -> vec3<f32> {
    let low = textures[p.low - 1u];
    let dims = vec2<i32>(textureDimensions(low));
    let f = p.uv * vec2<f32>(dims) - 0.5;
    let base = vec2<i32>(floor(f)) - vec2(1);
    let here = length(post_position(p, p.uv));
    var sum = vec3(0.0);
    var weight = 0.0;
    for (var j = 0; j < 16; j++) {
        let at = clamp(base + vec2(j % 4, j / 4), vec2(0), dims - 1);
        let uv = (vec2<f32>(at) + 0.5) / vec2<f32>(dims);
        let there = length(post_position(p, uv));
        let w = 1.0 / (0.02 + abs(there - here) / max(here, 1.0)) + 1e-6;
        sum += textureLoad(low, at, 0).rgb * w;
        weight += w;
    }
    return sum / weight;
}
