// Volumetric light (chiacraft's shader pack turns it on, `mods/base/src/shaderpack.rs`): the light
// the air sends toward the eye where the sun reaches it, marched at half the size each way by its
// half-size part (`shaders/post_low/volume.wgsl`, Research round 16 #7: about a quarter of the
// cost) and drawn here from it, upsampled by depth so beams don't bleed over silhouettes. Without
// the half-size image (an engine without `post_low`), marched here per pixel as before.
fn apply(p: PostPixel) -> vec4<f32> {
    var glow: vec3<f32>;
    if p.low != 0u {
        glow = post_low_upsampled(p).rgb;
    } else {
        glow = vol_glow(p);
    }
    return vec4(p.color.rgb + glow * (vec3(1.0) - p.color.rgb * 0.7), p.color.a);
}
