// Far relief (chiacraft's shader pack turns it on, `mods/base/src/shaderpack.rs`): the far land
// shaded by its own shapes, worked out at half size (`shaders/post_low/relief.wgsl`, where the why
// is) and upsampled by depth here. Without the half-size image, worked out here.
// The sun's part takes what the sun gives a far face, about half its light at midday (the rest the
// sky's); the ambient part all of it.
const RELIEF_SUN_SHARE: f32 = 0.55;

fn apply(p: PostPixel) -> vec4<f32> {
    var d: vec2<f32>;
    if p.low != 0u {
        d = post_low_upsampled(p).rg;
    } else {
        d = relief_dark(p);
    }
    let sun = mix(1.0 - RELIEF_SUN_SHARE * smoothstep(0.0, 0.15, scene.sun.w), 1.0, d.y);
    return vec4(p.color.rgb * d.x * sun, p.color.a);
}
