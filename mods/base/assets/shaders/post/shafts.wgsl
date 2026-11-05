// Light shafts (chiacraft's shader pack turns them on, `mods/base/src/shaderpack.rs`): marched at half
// the size each way by their half-size part (`shaders/post_low/shafts.wgsl`, Research round 16 #5)
// and drawn here from it, filtered: a radial blur toward the sun is soft. Without the half-size
// image, marched here per pixel as before.
fn apply(p: PostPixel) -> vec4<f32> {
    var s: vec4<f32>;
    if p.low != 0u {
        s = post_low_at(p, p.uv);
    } else {
        s = shaft_glow(p);
    }
    // Strength 2 (`--shafts-debug`): how it sees the frame.
    if s.a > 1.5 {
        return vec4(s.rgb, 1.0);
    }
    return vec4(min(p.color.rgb + s.rgb * (1.0 - p.color.rgb * 0.5), vec3(1.0)), 1.0);
}
