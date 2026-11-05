// Screen-space reflections on water (chiacraft's shader pack turns them on): what the water mirrors is
// marched at half the size each way (`shaders/post_low/reflect.wgsl`, where the whole method is
// described) and laid over the water here, upsampled by depth; without the half-size image,
// marched here per pixel as before. Only the first effect sees the image's alpha (the water's
// share), so this stays first.
fn apply(p: PostPixel) -> vec4<f32> {
    // Not water: nothing to lay over.
    if p.color.a < 0.02 {
        return p.color;
    }
    var r: vec4<f32>;
    if p.low != 0u {
        r = post_low_upsampled(p);
    } else {
        r = reflect_result(p);
    }
    return vec4(p.color.rgb * (1.0 - r.a) + r.rgb, p.color.a);
}
